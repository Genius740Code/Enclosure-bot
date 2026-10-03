//! probe_r_reinforce — v9 item 4 REINFORCE-LINES gate harness (lane R).
//!
//! One dose variable: R_REINFORCE (or the <w> CLI arg) — the weight on
//! measured Δdurable area (area of ours no single legal enemy cut can pop;
//! see eval_reinforce.rs) added to candidate selection, full-horizon points,
//! gated to moves that gain area or reach the enemy (DEADWOOD respected).
//!
//! Subcommands:
//!   selftest                 — dose-machinery sanity (durable_area etc.)
//!   identity [min]           — OFF-identity: control == shipped picks
//!   h2h <w>                  — dose vs control, 5 openings x 2 chairs
//!   league <w>               — dose AND control vs scoutbase, skips 0/10/20/30
//!   gauge <w>                — dose vs greedy 1-ply, 8 games (4 Blue / 4 Red)
//!   v1 <w>                   — dose AND control vs v1base, 5 openings x 2 chairs
//!   corpus <dir>             — survived/durable share per side, recorded games
//!   gates <w>                — BARS + selftest + identity + every gate + verdict
//!
//! All games deterministic (fixed openings/skips, no RNG); tiebreaks by move
//! index inside the searches. Gates tracked per side: survived / (survived +
//! popped) — the c-autopsy metric (ours 5.3%, VladNet 50-55%, GB 88-90%).

#[path = "../src/eval_reinforce.rs"]
mod eval_reinforce;
#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v1base.rs"]
mod v1base;

use meridian_engine::{Edge, Game, Move, Player, Point, Position, Score};
use std::cell::RefCell;

type Picker = fn(&Position) -> Option<Move>;

thread_local! {
    /// The dose weight for this run (fixed before any game starts).
    static DOSE_W: RefCell<f64> = const { RefCell::new(0.0) };
}

fn dose_w() -> f64 {
    DOSE_W.with(|w| *w.borrow())
}

fn dose_pick(position: &Position) -> Option<Move> {
    eval_reinforce::reinforce_best_move(position, dose_w())
}

/// The control: this lane's skeleton copy with the dose off. Must equal
/// `retaliator::search::best_move` byte for byte (identity check).
fn control_pick(position: &Position) -> Option<Move> {
    eval_reinforce::best_move(position)
}

fn shipped_pick(position: &Position) -> Option<Move> {
    retaliator::search::best_move(position)
}

fn scout_pick(position: &Position) -> Option<Move> {
    scoutbase::best_move(position)
}

fn v1_pick(position: &Position) -> Option<Move> {
    v1base::best_move(position)
}

/// 1-ply greedy: maximize own enclosed area after the move, ties by move id
/// (the gauge.rs standard).
fn greedy(position: &Position) -> Option<Move> {
    let mover = position.to_move();
    let mut best: Option<(f64, usize, Move)> = None;
    for mv in position.legal_moves().iter() {
        let mut after = position.clone();
        after.apply_unchecked(mv);
        let v = after.area(mover).to_f64();
        let id = mv.index();
        if best.is_none_or(|(bv, bid, _)| v > bv || (v == bv && id < bid)) {
            best = Some((v, id, mv));
        }
    }
    best.map(|(_, _, mv)| mv)
}

/// Survival bookkeeping for one side of one game: the c-autopsy metric.
#[derive(Clone, Copy, Default)]
struct Surv {
    final_area: f64,
    popped: f64,
}

impl Surv {
    fn share(&self) -> f64 {
        let d = self.final_area + self.popped;
        if d > 0.0 {
            self.final_area / d
        } else {
            0.0
        }
    }
    fn add(&mut self, other: Surv) {
        self.final_area += other.final_area;
        self.popped += other.popped;
    }
    fn pct(&self) -> f64 {
        100.0 * self.share()
    }
}

/// Play one game from `start`: side A (dose or control arm) vs side B.
/// Returns (a_score, b_score, a_surv, b_surv). Pops are the victim's area
/// drops inflicted by the other side's actions.
fn play(mut game: Game, a_blue: bool, a: Picker, b: Picker) -> (f64, f64, Surv, Surv) {
    let mut popped = [0.0f64; 2];
    while !game.is_over() {
        let mover = game.position().to_move();
        let a_moves = (mover == Player::Blue) == a_blue;
        let before = [
            game.position().area(Player::Blue).to_f64(),
            game.position().area(Player::Red).to_f64(),
        ];
        let mv = if a_moves { a(game.position()) } else { b(game.position()) };
        let Some(mv) = mv else { break };
        game.play(mv).expect("probe moves are legal");
        let after = [
            game.position().area(Player::Blue).to_f64(),
            game.position().area(Player::Red).to_f64(),
        ];
        let mi = usize::from(mover == Player::Red);
        for p in 0..2 {
            if p != mi && after[p] < before[p] {
                popped[p] += before[p] - after[p];
            }
        }
    }
    let fa = [
        game.position().area(Player::Blue).to_f64(),
        game.position().area(Player::Red).to_f64(),
    ];
    let mut bs = game.position().score(Player::Blue).to_f64();
    let mut rs = game.position().score(Player::Red).to_f64();
    if !game.is_over() {
        // A picker ran dry mid-game (draw by rule): scores so far still rank.
        bs = 0.0;
        rs = 0.0;
    }
    let (asc, bsc, sa, sb) = if a_blue {
        (bs, rs, Surv { final_area: fa[0], popped: popped[0] }, Surv { final_area: fa[1], popped: popped[1] })
    } else {
        (rs, bs, Surv { final_area: fa[1], popped: popped[1] }, Surv { final_area: fa[0], popped: popped[0] })
    };
    (asc, bsc, sa, sb)
}

/// The standard forced-opening set (probe_v3all / probe_b_h2h).
const OPENINGS: [Option<usize>; 5] = [None, Some(4864), Some(5589), Some(11723), Some(9199)];

fn start(opening: Option<usize>) -> Game {
    let mut game = Game::new();
    if let Some(id) = opening {
        game.play(Move::from_index(id).expect("opening id")).expect("opening is legal");
    }
    game
}

/// Skip prelude for the league gate: shipped-search self-play from the empty
/// start, identical for both arms (rows stay comparable with the scoreboard).
fn skipped(skip: usize) -> Game {
    let mut game = Game::new();
    for _ in 0..skip {
        if game.is_over() {
            break;
        }
        let mv = shipped_pick(game.position()).expect("self-play has a move");
        game.play(mv).expect("self-play moves are legal");
    }
    game
}

fn margin(asc: f64, bsc: f64) -> f64 {
    if asc.abs() > 0.0 {
        (asc - bsc) / asc.abs() * 100.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// selftest
// ---------------------------------------------------------------------------

fn edge(a: (i8, i8), b: (i8, i8)) -> Edge {
    Edge::between(
        Point::new(a.0, a.1).expect("on board"),
        Point::new(b.0, b.1).expect("on board"),
    )
    .expect("king step")
}

fn setup_pos(blue: &[Edge], red: &[Edge], ap: u8) -> Position {
    Position::setup(blue, red, ap, [Score::ZERO; 2], &[]).expect("editor position")
}

fn selftest() -> bool {
    let mut ok = true;
    let check = |name: &str, cond: bool, ok: &mut bool| {
        println!("  {name}: {}", if cond { "PASS" } else { "FAIL" });
        if !cond {
            *ok = false;
        }
    };

    // t1: start position — no area, nothing durable.
    let s = Position::new();
    check(
        "t1 start durable 0/0",
        eval_reinforce::durable_area(&s, Player::Blue) == 0.0
            && eval_reinforce::durable_area(&s, Player::Red) == 0.0,
        &mut ok,
    );

    // t2: single-touch triangle near Red — every edge legally cuttable by
    // one Red edge ending on it: durable must read 0.0.
    let tri = setup_pos(
        &[edge((0, 0), (3, 0)), edge((3, 0), (3, 3)), edge((3, 3), (0, 0))],
        &[edge((6, 0), (9, 0))],
        6,
    );
    let area = tri.area(Player::Blue).to_f64();
    let dur = eval_reinforce::durable_area(&tri, Player::Blue);
    println!("    triangle area={area} durable={dur}");
    check("t2 single-touch triangle durable == 0", dur == 0.0 && area == 4.5, &mut ok);

    // t3: same triangle + a chord — the cut now pops only half; durable
    // must read the surviving 2.25 exactly.
    let thick = setup_pos(
        &[
            edge((0, 0), (3, 0)),
            edge((3, 0), (3, 3)),
            edge((3, 3), (0, 0)),
            edge((3, 0), (0, 3)),
        ],
        &[edge((6, 0), (9, 0))],
        6,
    );
    let dur3 = eval_reinforce::durable_area(&thick, Player::Blue);
    println!("    chorded triangle area={} durable={dur3}", thick.area(Player::Blue).to_f64());
    check("t3 chorded triangle durable == 2.25", (dur3 - 2.25).abs() < 1e-9, &mut ok);

    // t4: far single-touch triangle — Red cannot reach it; the engine's
    // legality (reachability) is part of durability, so it reads in full.
    let far = setup_pos(
        &[edge((-9, 0), (-6, 0)), edge((-6, 0), (-6, 3)), edge((-6, 3), (-9, 0))],
        &[edge((6, 0), (9, 0))],
        6,
    );
    let dur4 = eval_reinforce::durable_area(&far, Player::Blue);
    check("t4 unreachable triangle durable == area", dur4 == far.area(Player::Blue).to_f64(), &mut ok);

    // t5: invariants + dose-off identity over a real shipped self-play line.
    let mut game = Game::new();
    for i in 0..12 {
        if game.is_over() {
            break;
        }
        let mv = shipped_pick(game.position()).expect("move");
        game.play(mv).expect("legal");
        let pos = game.position();
        for p in [Player::Blue, Player::Red] {
            let d = eval_reinforce::durable_area(pos, p);
            let a = pos.area(p).to_f64();
            if !(d.is_finite() && d >= -1e-9 && d <= a + 1e-9) {
                ok = false;
                println!("  t5 INVARIANT FAIL at action {}: d={d} a={a}", i + 1);
            }
        }
    }
    let pos = game.position().clone();
    let control = control_pick(&pos);
    let shipped = shipped_pick(&pos);
    let off = eval_reinforce::reinforce_best_move(&pos, 0.0);
    check(
        "t5 dose-off == control == shipped",
        control == shipped && off == shipped,
        &mut ok,
    );
    println!("  t5 invariants (0 <= durable <= area, both sides, 12 actions): done");

    ok
}

// ---------------------------------------------------------------------------
// identity
// ---------------------------------------------------------------------------

fn cmd_identity(min: usize) -> bool {
    let min = min.max(239);
    let mut checked = 0usize;
    let mut bad = 0usize;
    for opening in OPENINGS {
        // Two lines per opening: scoutbase self-play and control self-play.
        for line in 0..2 {
            let mut game = start(opening);
            while !game.is_over() && checked < min {
                let shipped = shipped_pick(game.position());
                let control = control_pick(game.position());
                checked += 1;
                if control != shipped {
                    bad += 1;
                    println!(
                        "MISMATCH actions_played={}: control={:?} shipped={:?}",
                        game.position().actions_played(),
                        control.map(|mv| mv.index()),
                        shipped.map(|mv| mv.index())
                    );
                }
                let mv = match line {
                    0 => scout_pick(game.position()),
                    _ => control_pick(game.position()),
                };
                let Some(mv) = mv else { break };
                game.play(mv).expect("identity line moves are legal");
            }
            if checked >= min {
                break;
            }
        }
        if checked >= min {
            break;
        }
    }
    println!("identity: {}/{} control == shipped (bar: 0 mismatches over >= 239)", checked - bad, checked);
    bad == 0 && checked >= 239
}

// ---------------------------------------------------------------------------
// gates
// ---------------------------------------------------------------------------

struct GateOut {
    name: &'static str,
    pass: bool,
    detail: String,
}

fn cmd_h2h() -> (GateOut, Surv, Surv) {
    println!("--- h2h: dose vs control (v8), 5 openings x 2 chairs ---");
    let mut wins = [0usize; 2]; // [blue-chair dose wins, red-chair dose wins]
    let mut rows = 0usize;
    let (mut sa, mut sb) = (Surv::default(), Surv::default());
    for opening in OPENINGS {
        for dose_blue in [true, false] {
            let (asc, bsc, a, b) = play(start(opening), dose_blue, dose_pick, control_pick);
            sa.add(a);
            sb.add(b);
            let won = asc > bsc;
            if won {
                wins[usize::from(!dose_blue)] += 1;
            }
            rows += 1;
            println!(
                "  open={opening:?} dose={} {asc:.0}-{bsc:.0} m={:+.1}% {} | surv dose={:.1}% ctrl={:.1}%",
                if dose_blue { "blue" } else { "red" },
                margin(asc, bsc),
                if won { "DOSE" } else { "ctrl" },
                a.pct(),
                b.pct(),
            );
        }
    }
    let total = wins[0] + wins[1];
    let pass = total >= 6 && wins[0] >= 2 && wins[1] >= 2;
    println!(
        "  h2h chairs: dose Blue {}/5, Red {}/5, total {total}/10 (bar: >=6/10, no chair <2/5)",
        wins[0], wins[1]
    );
    (
        GateOut {
            name: "h2h",
            pass,
            detail: format!("dose {total}/10 (B {}/5, R {}/5)", wins[0], wins[1]),
        },
        sa,
        sb,
    )
}

fn cmd_league() -> (GateOut, Surv, Surv) {
    println!("--- league: dose and control vs scoutbase, skips 0/10/20/30 x 2 colors ---");
    let mut dm_sum = 0.0;
    let mut cm_sum = 0.0;
    let mut n = 0;
    let mut worst = f64::INFINITY;
    let (mut sa, mut sc) = (Surv::default(), Surv::default());
    for skip in [0usize, 10, 20, 30] {
        for dose_blue in [true, false] {
            let g = || skipped(skip);
            let (ars, ass, a, _) = play(g(), dose_blue, dose_pick, scout_pick);
            let (crs, css, c, _) = play(g(), dose_blue, control_pick, scout_pick);
            sa.add(a);
            sc.add(c);
            let dm = margin(ars, ass);
            let cm = margin(crs, css);
            dm_sum += dm;
            cm_sum += cm;
            n += 1;
            worst = worst.min(dm);
            println!(
                "  skip={skip:>2} dose={} | dose {ars:.0}-{ass:.0} m={dm:+.1}% | control {crs:.0}-{css:.0} m={cm:+.1}% | diff {:+.1}pp",
                if dose_blue { "blue" } else { "red" },
                dm - cm,
            );
        }
    }
    let (avg_d, avg_c) = (dm_sum / n as f64, cm_sum / n as f64);
    let pass = avg_d > avg_c && worst > -300.0;
    println!(
        "  league AVG dose {avg_d:+.1}% vs control {avg_c:+.1}%, worst dose row {worst:+.1}% (bar: avg >, no row < -300%)",
    );
    (
        GateOut {
            name: "league",
            pass,
            detail: format!("dose {avg_d:+.1}% vs ctrl {avg_c:+.1}%, worst {worst:+.1}%"),
        },
        sa,
        sc,
    )
}

fn cmd_gauge() -> (GateOut, Surv, Surv) {
    println!("--- gauge: dose vs greedy 1-ply, 8 games (4 Blue / 4 Red) ---");
    let mut wins = 0usize;
    let (mut sa, mut sb) = (Surv::default(), Surv::default());
    for g in 0..8 {
        let dose_blue = g % 2 == 0;
        let (asc, bsc, a, b) = play(Game::new(), dose_blue, dose_pick, greedy);
        sa.add(a);
        sb.add(b);
        let won = asc > bsc;
        wins += usize::from(won);
        println!(
            "  game {g}: dose={} {asc:.0}-{bsc:.0} m={:+.1}% {} | surv dose={:.1}% greedy={:.1}%",
            if dose_blue { "blue" } else { "red" },
            margin(asc, bsc),
            if won { "WIN" } else { "loss" },
            a.pct(),
            b.pct(),
        );
    }
    let pass = wins >= 7;
    println!("  gauge: dose {wins}/8 (bar: >=7/8)");
    (GateOut { name: "gauge", pass, detail: format!("{wins}/8") }, sa, sb)
}

fn cmd_v1() -> (GateOut, Surv, Surv) {
    println!("--- v1: dose and control vs v1base, 5 openings x 2 chairs each ---");
    let (mut dw, mut cw) = (0usize, 0usize);
    let (mut sa, mut sc) = (Surv::default(), Surv::default());
    for opening in OPENINGS {
        for our_blue in [true, false] {
            let g = || start(opening);
            let (ars, ass, a, _) = play(g(), our_blue, dose_pick, v1_pick);
            let (crs, css, c, _) = play(g(), our_blue, control_pick, v1_pick);
            dw += usize::from(ars > ass);
            cw += usize::from(crs > css);
            sa.add(a);
            sc.add(c);
            println!(
                "  open={opening:?} us={} | dose {ars:.0}-{ass:.0} | control {crs:.0}-{css:.0}",
                if our_blue { "blue" } else { "red" },
            );
        }
    }
    let pass = dw >= cw;
    println!("  v1: dose {dw}/10 vs control {cw}/10 (bar: dose >= control)");
    (GateOut { name: "v1", pass, detail: format!("dose {dw}/10, ctrl {cw}/10") }, sa, sc)
}

// ---------------------------------------------------------------------------
// corpus: survived + durable share on recorded games (metric baselines)
// ---------------------------------------------------------------------------

fn cmd_corpus(dir: &str) {
    println!("--- corpus: per-side survival and durable share ({dir}) ---");
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .expect("corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".json"))
        .collect();
    files.sort();
    // winner vs loser splits
    let (mut w_surv, mut l_surv) = (Vec::new(), Vec::new());
    let (mut w_dura, mut l_dura) = (Vec::new(), Vec::new());
    for name in &files {
        let text = std::fs::read_to_string(format!("{dir}/{name}")).expect("readable");
        let j: serde_json::Value = serde_json::from_str(&text).expect("json");
        let Some(ids) = j["moves"].as_array().map(|a| a.iter().filter_map(|v| v.as_u64()).collect::<Vec<_>>()) else {
            println!("  {name}: no moves array, skipped");
            continue;
        };
        let winner = match j["result"].as_str() {
            Some("blue") => Some(Player::Blue),
            Some("red") => Some(Player::Red),
            _ => None,
        };
        let blue_name = j["blue"].as_str().unwrap_or("?").to_string();
        let red_name = j["red"].as_str().unwrap_or("?").to_string();
        let mut game = Game::new();
        let mut popped = [0.0f64; 2];
        let mut dura = [0.0f64; 2];
        let mut dura_n = [0.0f64; 2];
        for (i, id) in ids.iter().enumerate() {
            let mv = Move::from_index(*id as usize).expect("site id");
            let before = [
                game.position().area(Player::Blue).to_f64(),
                game.position().area(Player::Red).to_f64(),
            ];
            let mover = game.position().to_move();
            game.play(mv).expect("site moves are legal");
            let after = [
                game.position().area(Player::Blue).to_f64(),
                game.position().area(Player::Red).to_f64(),
            ];
            let mi = usize::from(mover == Player::Red);
            for p in 0..2 {
                if p != mi && after[p] < before[p] {
                    popped[p] += before[p] - after[p];
                }
            }
            // durable share sampled every 4 actions (both sides)
            if i % 4 == 0 && !game.is_over() {
                for (p, player) in [Player::Blue, Player::Red].iter().enumerate() {
                    let a = game.position().area(*player).to_f64();
                    if a > 0.0 {
                        dura[p] += eval_reinforce::durable_area(game.position(), *player) / a;
                        dura_n[p] += 1.0;
                    }
                }
            }
        }
        let fin = [
            game.position().area(Player::Blue).to_f64(),
            game.position().area(Player::Red).to_f64(),
        ];
        let surv = [
            Surv { final_area: fin[0], popped: popped[0] },
            Surv { final_area: fin[1], popped: popped[1] },
        ];
        let dshare = [
            if dura_n[0] > 0.0 { dura[0] / dura_n[0] } else { 0.0 },
            if dura_n[1] > 0.0 { dura[1] / dura_n[1] } else { 0.0 },
        ];
        println!(
            "  {name}: {blue_name} vs {red_name} | result={} | surv B={:.1}% R={:.1}% | durable B={:.2} R={:.2}",
            j["result"].as_str().unwrap_or("?"),
            surv[0].pct(),
            surv[1].pct(),
            dshare[0],
            dshare[1],
        );
        if let Some(w) = winner {
            let l = w.opponent();
            let (wi, li) = (usize::from(w == Player::Red), usize::from(l == Player::Red));
            w_surv.push(surv[wi].share());
            l_surv.push(surv[li].share());
            w_dura.push(dshare[wi]);
            l_dura.push(dshare[li]);
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    println!(
        "  split ({} games): winner surv {:.1}% vs loser {:.1}% | winner durable {:.3} vs loser {:.3}",
        w_surv.len(),
        100.0 * mean(&w_surv),
        100.0 * mean(&l_surv),
        mean(&w_dura),
        mean(&l_dura),
    );
}

// ---------------------------------------------------------------------------
// gates orchestrator
// ---------------------------------------------------------------------------

fn cmd_gates(w: f64) {
    println!("REINFORCE-LINES D1 gates — REINFORCE_W = {w}");
    println!("BARS (stated before the run):");
    println!("  selftest : all machinery checks pass");
    println!("  identity : 0 mismatches over >= 239 positions");
    println!("  h2h      : dose >= 6/10 vs control, no chair < 2/5");
    println!("  league   : AVG(dose) > AVG(control), no dose row < -300%");
    println!("  gauge    : dose >= 7/8 vs greedy");
    println!("  v1       : dose wins >= control wins");
    println!("  survival : pooled dose surv > control surv (mechanism moves)");
    println!();
    DOSE_W.with(|c| *c.borrow_mut() = w);

    let mut gates: Vec<GateOut> = Vec::new();
    let (mut dose_surv, mut ctrl_surv) = (Surv::default(), Surv::default());

    println!("=== selftest ===");
    gates.push(GateOut { name: "selftest", pass: selftest(), detail: String::new() });

    println!("\n=== identity ===");
    gates.push(GateOut { name: "identity", pass: cmd_identity(239), detail: String::new() });

    println!("\n=== gates ===");
    let (g, a, b) = cmd_h2h();
    dose_surv.add(a);
    ctrl_surv.add(b); // h2h opponent IS the control
    gates.push(g);
    let (g, a, b) = cmd_league();
    dose_surv.add(a);
    ctrl_surv.add(b); // league control arm (vs scoutbase)
    gates.push(g);
    let (g, a, _greedy) = cmd_gauge();
    dose_surv.add(a); // greedy's survival is reported in the rows, not pooled
    gates.push(g);
    let (g, a, b) = cmd_v1();
    dose_surv.add(a);
    ctrl_surv.add(b); // v1 control arm (vs v1base)
    gates.push(g);

    let surv_pass = dose_surv.share() > ctrl_surv.share();
    println!("\n=== survival (pooled over h2h+league+gauge+v1, dose side vs control side) ===");
    println!(
        "  dose: final {:.0} + popped {:.0} = {:.1}% | control: final {:.0} + popped {:.0} = {:.1}% (bar: dose > control)",
        dose_surv.final_area,
        dose_surv.popped,
        dose_surv.pct(),
        ctrl_surv.final_area,
        ctrl_surv.popped,
        ctrl_surv.pct(),
    );

    println!("\n=== VERDICT (W={w}) ===");
    for g in &gates {
        println!("  {:<8} {} {}", g.name, if g.pass { "PASS" } else { "FAIL" }, g.detail);
    }
    println!(
        "  {:<8} {} dose {:.1}% vs ctrl {:.1}%",
        "survival",
        if surv_pass { "PASS" } else { "FAIL" },
        dose_surv.pct(),
        ctrl_surv.pct(),
    );
    let all = gates.iter().all(|g| g.pass) && surv_pass;
    println!(
        "\nOVERALL: {} — REINFORCE_W={w}",
        if all { "GO (all bars pass)" } else { "NO-GO (>=1 bar failed)" }
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("gates");
    // Weight for the dose subcommands: CLI arg first, then R_REINFORCE env.
    let env_w = || {
        let r = eval_reinforce::Reinforce::from_env();
        if r.on() { Some(r.w) } else { None }
    };
    let w: f64 = match cmd {
        "h2h" | "league" | "gauge" | "v1" | "gates" => {
            args.get(2).and_then(|s| s.parse::<f64>().ok()).or_else(env_w).unwrap_or(0.0)
        }
        _ => 0.0,
    };
    match cmd {
        "selftest" => {
            let ok = selftest();
            std::process::exit(i32::from(!ok));
        }
        "identity" => {
            let min: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(239);
            std::process::exit(i32::from(!cmd_identity(min)));
        }
        "h2h" => {
            DOSE_W.with(|c| *c.borrow_mut() = w);
            cmd_h2h();
        }
        "league" => {
            DOSE_W.with(|c| *c.borrow_mut() = w);
            cmd_league();
        }
        "gauge" => {
            DOSE_W.with(|c| *c.borrow_mut() = w);
            cmd_gauge();
        }
        "v1" => {
            DOSE_W.with(|c| *c.borrow_mut() = w);
            cmd_v1();
        }
        "corpus" => cmd_corpus(args.get(2).expect("corpus dir").as_str()),
        "gates" => cmd_gates(w),
        other => {
            eprintln!("unknown subcommand {other:?} (selftest|identity|h2h|league|gauge|v1|corpus|gates)");
            std::process::exit(2);
        }
    }
}
