//! probe_r_reinforce: v9 REINFORCE D1 gate runner (roadmap item 4).
//!
//! Dose under test: `eval_reinforce::REINFORCE_DENSE_W` — the DENSE_BONUS
//! extension toward building behind 2+ shared-node walls, then expanding
//! far / banking behind from shared anchors (structural only; DEADWOOD and
//! IDLE respected inside `reinforce_structure`).
//!
//! Laws enforced here (v9-roadmap): one variable per dose (the weight);
//! full-horizon points (the term scales with hz like DENSE itself);
//! deterministic tiebreak by move index (untouched comparators); n>=8 both
//! colors vs v1+scoutbase+greedy; per-chair E-6 tables (every gate judges
//! each chair vs control at the same color); kill-0/OFF-identity before any
//! strength claim; scoreboard row only from these on-disk logs.
//!
//! Modes (arg 1, default `all`):
//!   selftest  pure-fn sanity checks
//!   identity  OFF-identity: baseline twins vs the LIVE shipped engine,
//!             >=30 positions, byte-identical picks + candidates (HARD gate)
//!   census    flip census across a weight grid (inertness detector: the
//!             swept H1 term's NO-GO was zero flips at every weight)
//!   h2h       dose vs control (live engine), 10 games, both colors
//!   suite     dose AND control each vs v1base/scoutbase/greedybase,
//!             8 games both colors per opponent (E-6 + survival metric)
//!   league    league_mesh structure, routed dose and routed control,
//!             skip={0,10,20,30} x colors vs scoutbase
//!   gauge     vs greedy 1-ply, 6 games, dose and control
//!
//! Usage: cargo run --release --example probe_r_reinforce -- [mode]

#[path = "../src/eval_reinforce_dense.rs"]
mod eval_reinforce;

#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/scoutbase.rs"]
mod scoutbase;
// greedy: 1-ply max-area (the gauge archetype), verbatim from gauge.rs —
// support/greedybase.rs is not on this lane's base.

use eval_reinforce::REINFORCE_DENSE_W;
use meridian_engine::{Game, Move, Player, Position};

/// 1-ply greedy: maximize own enclosed area after the move, ties by move id.
/// (Verbatim from the shipped gauge.rs probe — the blob archetype.)
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

/// Fixed node budget for every deterministic gate (h2h_base convention).
const BUDGET: usize = 4096;
/// Safety cap: no probe game may run past this many actions.
const MOVE_CAP: usize = 600;

type Picker = fn(&Game) -> Option<Move>;

// --- our side -----------------------------------------------------------

fn dose_fixed(game: &Game) -> Option<Move> {
    eval_reinforce::analyze_with_avoid(game.position(), BUDGET, &[])
        .candidates
        .first()
        .map(|c| c.mv)
}

fn control_fixed(game: &Game) -> Option<Move> {
    retaliator::search::analyze_with_avoid(game.position(), BUDGET, &[])
        .candidates
        .first()
        .map(|c| c.mv)
}

fn dose_routed(game: &Game) -> Option<Move> {
    eval_reinforce::best_move_routed(game.position(), game.moves(), &[])
}

fn control_routed(game: &Game) -> Option<Move> {
    retaliator::search::best_move_routed(game.position(), game.moves(), &[])
}

// --- opponents ----------------------------------------------------------

fn v1_pick(game: &Game) -> Option<Move> {
    v1base::best_move(game.position())
}

fn scout_pick(game: &Game) -> Option<Move> {
    scoutbase::best_move(game.position())
}

fn greedy_pick(game: &Game) -> Option<Move> {
    greedy(game.position())
}

// --- game runner with survival tracking ---------------------------------

/// One played game from our side's perspective. `survival` is the c-autopsy
/// metric: survived / (survived + popped), where popped is every later
/// area decrease after a gain (re-closes re-count — matches the site
/// autopsy method). Our baseline on site was 5.3% (VladNet 50-55%,
/// GB 88-90%).
struct Played {
    our_score: f64,
    opp_score: f64,
    our_final_area: f64,
    popped: f64,
    our_moves: u32,
    dense_landings: u32,
    actions: u32,
    capped: bool,
}

impl Played {
    fn survival(&self) -> f64 {
        let survived = self.our_final_area;
        let produced = survived + self.popped;
        if produced <= 0.0 {
            0.0
        } else {
            survived / produced
        }
    }
}

fn run_game(our_blue: bool, our: Picker, opp: Picker, skip: usize) -> Played {
    let mut game = Game::new();
    // Neutral setup moves: the LIVE shipped engine plays the prefix for
    // both variants (league_mesh precedent) — identical start position.
    for _ in 0..skip {
        if game.is_over() {
            break;
        }
        let mv = retaliator::search::best_move(game.position()).expect("setup move");
        game.play(mv).expect("setup moves are legal");
    }
    let us = if our_blue { Player::Blue } else { Player::Red };
    let mut area = game.position().area(us).to_f64();
    let mut popped = 0.0f64;
    let mut our_moves = 0u32;
    let mut dense_landings = 0u32;
    let mut actions = 0u32;
    let mut capped = false;
    while !game.is_over() {
        if game.moves().len() >= MOVE_CAP {
            capped = true;
            break;
        }
        let to_move = game.position().to_move();
        let our_turn = (to_move == Player::Blue) == our_blue;
        let mv = if our_turn { our(&game) } else { opp(&game) };
        let Some(mv) = mv else { break };
        game.play(mv).expect("bot moves are legal");
        actions += 1;
        let a = game.position().area(us).to_f64();
        if a < area {
            popped += area - a;
        }
        area = a;
        if our_turn {
            our_moves += 1;
            // Dense-landing freq (the DENSE anchor, same formula for both
            // variants so the counts are comparable): own nodes within
            // Chebyshev 1 of the target after the move.
            if let Some(t) = mv.target() {
                let near = game
                    .position()
                    .nodes(us)
                    .iter()
                    .filter(|p| (p.x() - t.x()).abs() <= 1 && (p.y() - t.y()).abs() <= 1)
                    .count();
                if near >= 2 {
                    dense_landings += 1;
                }
            }
        }
    }
    let our_score = game.position().score(us).to_f64();
    let opp_score = game.position().score(us.opponent()).to_f64();
    Played {
        our_score,
        opp_score,
        our_final_area: game.position().area(us).to_f64(),
        popped,
        our_moves,
        dense_landings,
        actions,
        capped,
    }
}

/// Plays `games` alternating colors and returns per-chair (Blue, Red) wins
/// plus pooled metrics. `skips` spreads the openings so the deterministic
/// games differ (league skip-prefix precedent).
fn run_match(our: Picker, opp: Picker, skips: &[usize]) -> (u32, u32, Vec<Played>) {
    let mut blue_wins = 0u32;
    let mut red_wins = 0u32;
    let mut all = Vec::new();
    for (i, &skip) in skips.iter().enumerate() {
        let our_blue = i % 2 == 0;
        let played = run_game(our_blue, our, opp, skip);
        if played.our_score > played.opp_score {
            if our_blue {
                blue_wins += 1;
            } else {
                red_wins += 1;
            }
        }
        all.push(played);
    }
    (blue_wins, red_wins, all)
}

fn pooled(plays: &[Played]) -> (f64, f64, f64, u32, u32) {
    let survived: f64 = plays.iter().map(|p| p.our_final_area).sum();
    let popped: f64 = plays.iter().map(|p| p.popped).sum();
    let survival = if survived + popped > 0.0 {
        survived / (survived + popped)
    } else {
        0.0
    };
    let dense: f64 = plays.iter().map(|p| p.dense_landings).sum::<u32>() as f64;
    let moves: f64 = plays.iter().map(|p| p.our_moves).sum::<u32>() as f64;
    let avg_margin = plays
        .iter()
        .map(|p| if p.our_score > 0.0 { (p.our_score - p.opp_score) / p.our_score * 100.0 } else { 0.0 })
        .sum::<f64>()
        / plays.len() as f64;
    let dense_freq = if moves > 0.0 { dense / moves } else { 0.0 };
    (survival, avg_margin, dense_freq, plays.len() as u32, moves as u32)
}

// --- position generation (deterministic) --------------------------------

/// Diverse deterministic positions: sampled prefixes of (a) a fixed-budget
/// live self-play game and (b) a routed (mesh-opening) live self-play game.
/// Every position is real reachable game state; no RNG anywhere.
fn sample_positions(n: usize) -> Vec<(Position, Vec<Move>)> {
    let mut out = Vec::new();
    let take_every = |game: &mut Game, step: usize, out: &mut Vec<(Position, Vec<Move>)>| {
        let mut played = 0usize;
        while !game.is_over() && out.len() < n {
            let mv = retaliator::search::analyze_with_avoid(game.position(), BUDGET, &[])
                .candidates
                .first()
                .map(|c| c.mv);
            let Some(mv) = mv else { break };
            game.play(mv).expect("legal");
            played += 1;
            if played % step == 0 {
                out.push((game.position().clone(), game.moves().to_vec()));
            }
        }
    };
    // (a) fixed-budget lineage from the plain start.
    let mut game = Game::new();
    take_every(&mut game, 3, &mut out);
    // (b) routed lineage: mesh openings on both colors (live engine).
    let mut game = Game::new();
    let mut played = 0usize;
    while !game.is_over() && out.len() < n {
        let mv = retaliator::search::best_move_routed(game.position(), game.moves(), &[]);
        let Some(mv) = mv else { break };
        game.play(mv).expect("legal");
        played += 1;
        if played % 3 == 0 {
            out.push((game.position().clone(), game.moves().to_vec()));
        }
    }
    out
}

// --- modes --------------------------------------------------------------

fn mode_selftest() {
    println!("selftest: pure-fn sanity (full unit suite: cargo test --example probe_r_reinforce)");
    let position = Position::new();
    let mv = Move::between(
        meridian_engine::Point::new(-6, 0).expect("D10"),
        meridian_engine::Point::new(-4, -3).expect("F7"),
    )
    .expect("D10-F7");
    let mut after = position.clone();
    after.apply_unchecked(mv);
    let baseline = eval_reinforce::baseline_analyze_with_avoid(&position, 512, &[]);
    let dose = eval_reinforce::analyze_with_avoid(&position, 512, &[]);
    println!(
        "selftest: start pos baseline pick {:?} dose pick {:?} (baseline nodes {})",
        baseline.candidates.first().map(|c| c.mv.index()),
        dose.candidates.first().map(|c| c.mv.index()),
        baseline.nodes
    );
    assert!(baseline.candidates.first().is_some(), "baseline must pick on the start");
    assert!(dose.candidates.first().is_some(), "dose must pick on the start");
    assert_eq!(baseline.candidates.first().map(|c| c.mv.index()), dose.candidates.first().map(|c| c.mv.index()), "blue opener is forced for both");
    println!("selftest: OK");
}

fn mode_identity() {
    println!("=== OFF-identity: eval_reinforce baseline twins vs LIVE shipped engine ===");
    println!("REINFORCE_DENSE_W (dose) = {REINFORCE_DENSE_W}; baseline runs 0.0");
    let positions = sample_positions(32);
    println!("positions: {} (deterministic lineages, fixed budget {BUDGET})", positions.len());
    assert!(positions.len() >= 30, "need >=30 positions, got {}", positions.len());
    let mut pick_ok = 0;
    let mut cand_ok = 0;
    for (i, (pos, _hist)) in positions.iter().enumerate() {
        let live = retaliator::search::analyze_with_avoid(pos, BUDGET, &[]);
        let base = eval_reinforce::baseline_analyze_with_avoid(pos, BUDGET, &[]);
        let pick_same = live.candidates.first().map(|c| c.mv) == base.candidates.first().map(|c| c.mv);
        let mut list_same = live.candidates.len() == base.candidates.len();
        if list_same {
            for (a, b) in live.candidates.iter().zip(base.candidates.iter()) {
                if a.mv != b.mv || a.evaluation.to_bits() != b.evaluation.to_bits() {
                    list_same = false;
                    break;
                }
            }
        }
        if live.nodes != base.nodes || live.evaluation.to_bits() != base.evaluation.to_bits() {
            list_same = false;
        }
        if pick_same {
            pick_ok += 1;
        } else {
            println!(
                "  MISMATCH pick #{i}: live={:?} baseline={:?}",
                live.candidates.first().map(|c| c.mv.index()),
                base.candidates.first().map(|c| c.mv.index())
            );
        }
        if list_same {
            cand_ok += 1;
        } else {
            println!("  MISMATCH candidate list #{i} (nodes live={} base={})", live.nodes, base.nodes);
        }
    }
    println!(
        "OFF-identity: picks {pick_ok}/{} byte-identical, full candidate lists {cand_ok}/{} byte-identical",
        positions.len(),
        positions.len()
    );
    // Routed/timed spot check (machine-dependent path — report, not gate).
    let mut routed_ok = 0;
    let mut routed_n = 0;
    for (pos, hist) in positions.iter().take(10) {
        if pos.actions_played() % 2 == 1 {
            continue; // timed think only on our turn; either is fine — sample both
        }
        routed_n += 1;
        let live = retaliator::search::best_move_routed(pos, hist, &[]);
        let base = eval_reinforce::baseline_best_move_routed(pos, hist, &[]);
        if live == base {
            routed_ok += 1;
        }
    }
    println!("routed/timed spot check (report only): {routed_ok}/{routed_n} identical");
    println!(
        "GATE: {}",
        if pick_ok == positions.len() && cand_ok == positions.len() { "PASS" } else { "FAIL" }
    );
}

fn mode_census() {
    println!("=== flip census: dose pick vs OFF pick across weights ===");
    let positions = sample_positions(36);
    println!("positions: {} (deterministic), budget {BUDGET}", positions.len());
    let grid = [0.25f64, 0.5, 1.0, 2.0, 4.0];
    let mut off_picks = Vec::new();
    for (pos, _) in &positions {
        off_picks.push(
            eval_reinforce::analyze_with_weight(pos, BUDGET, &[], 0.0)
                .candidates
                .first()
                .map(|c| c.mv),
        );
    }
    for w in grid {
        let mut flips = 0;
        for ((pos, _), off) in positions.iter().zip(off_picks.iter()) {
            let on = eval_reinforce::analyze_with_weight(pos, BUDGET, &[], w)
                .candidates
                .first()
                .map(|c| c.mv);
            if on != *off {
                flips += 1;
            }
        }
        println!("W={w:<5}: flips {flips}/{} ({:.1}%)", positions.len(), flips as f64 / positions.len() as f64 * 100.0);
    }
    println!(
        "compiled D1 weight: REINFORCE_DENSE_W = {REINFORCE_DENSE_W} (census at other weights is ungated context)"
    );
}

fn mode_h2h() {
    println!("=== h2h: dose vs control (LIVE shipped engine), 10 games, both colors ===");
    let skips = [0usize, 5, 10, 15, 20, 25, 30, 35, 40, 45];
    let (b, r, plays) = run_match(dose_fixed, control_fixed, &skips);
    for (i, p) in plays.iter().enumerate() {
        println!(
            "game {:2}: {} score {:4.0} vs {:4.0} margin {:+.1}% survival {:.1}% dense {:.0}% ({} actions)",
            i + 1,
            if i % 2 == 0 { "doseBlue" } else { "doseRed" },
            p.our_score,
            p.opp_score,
            (p.our_score - p.opp_score) / p.our_score.max(1.0) * 100.0,
            p.survival() * 100.0,
            if p.our_moves > 0 { p.dense_landings as f64 / p.our_moves as f64 * 100.0 } else { 0.0 },
            p.actions
        );
    }
    let (surv, margin, dense, n, mv) = pooled(&plays);
    println!("h2h: dose {}/10 (Blue {b}/5, Red {r}/5)", (b + r));
    println!(
        "h2h pooled: survival {:.1}% avg margin {margin:+.1}% dense-freq {:.1}% ({n} games, {mv} moves)",
        surv * 100.0,
        dense * 100.0
    );
    println!("GATE (h2h >= 6/10): {}", if b + r >= 6 { "PASS" } else { "FAIL" });
}

fn mode_suite() {
    println!("=== suite: dose AND control each vs v1/scout/greedy, 8 games both colors ===");
    let skips = [0usize, 5, 10, 15, 20, 25, 30, 35];
    let opps: [(&str, Picker); 3] = [("v1", v1_pick), ("scout", scout_pick), ("greedy", greedy_pick)];
    for (name, opp) in opps {
        let (db, dr, dplays) = run_match(dose_fixed, opp, &skips);
        let (cb, cr, cplays) = run_match(control_fixed, opp, &skips);
        let (ds, dm, dd, _, _) = pooled(&dplays);
        let (cs, cm, cd, _, _) = pooled(&cplays);
        println!("{name}: dose {}/8 (B {db}/4 R {dr}/4) vs control {}/8 (B {cb}/4 R {cr}/4)", db + dr, cb + cr);
        println!(
            "  E-6 per chair: dose B {db}/4 R {dr}/4 | control B {cb}/4 R {cr}/4 | delta B {:+} R {:+}",
            db as i32 - cb as i32,
            dr as i32 - cr as i32
        );
        println!(
            "  dose:    survival {:.1}% margin {:+.1}% dense {:.1}%",
            ds * 100.0,
            dm,
            dd * 100.0
        );
        println!(
            "  control: survival {:.1}% margin {:+.1}% dense {:.1}%",
            cs * 100.0,
            cm,
            cd * 100.0
        );
    }
}

fn mode_league() {
    println!("=== league: league_mesh structure, routed, vs scoutbase ===");
    println!("rows: skip x color x variant; margins from our perspective");
    let skips = [0usize, 10, 20, 30];
    let mut dose_margins = Vec::new();
    let mut control_margins = Vec::new();
    for &skip in &skips {
        for &our_blue in &[true, false] {
            let d = run_game(our_blue, dose_routed, scout_pick, skip);
            let c = run_game(our_blue, control_routed, scout_pick, skip);
            let dm = if d.our_score > 0.0 { (d.our_score - d.opp_score) / d.our_score * 100.0 } else { 0.0 };
            let cm = if c.our_score > 0.0 { (c.our_score - c.opp_score) / c.our_score * 100.0 } else { 0.0 };
            println!(
                "skip={skip} {} dose: {:4.0} vs {:4.0} margin {dm:+.1}% (survival {:.1}%{}) | control: {:4.0} vs {:4.0} margin {cm:+.1}% (survival {:.1}%{})",
                if our_blue { "blue" } else { "red " },
                d.our_score,
                d.opp_score,
                d.survival() * 100.0,
                if d.capped { " CAPPED" } else { "" },
                c.our_score,
                c.opp_score,
                c.survival() * 100.0,
                if c.capped { " CAPPED" } else { "" }
            );
            dose_margins.push(dm);
            control_margins.push(cm);
        }
    }
    let davg = dose_margins.iter().sum::<f64>() / dose_margins.len() as f64;
    let dworst = dose_margins.iter().cloned().fold(f64::INFINITY, f64::min);
    let cavg = control_margins.iter().sum::<f64>() / control_margins.len() as f64;
    let cworst = control_margins.iter().cloned().fold(f64::INFINITY, f64::min);
    println!("league AVG margin: dose {davg:+.1}% (worst {dworst:+.1}%) | control {cavg:+.1}% (worst {cworst:+.1}%)");
    println!(
        "GATE (AVG > -9.9% AND worst > -300%): {}",
        if davg > -9.9 && dworst > -300.0 { "PASS" } else { "FAIL" }
    );
}

fn mode_gauge() {
    println!("=== gauge: vs greedy 1-ply, 6 games, dose and control ===");
    let mut dwin = 0;
    let mut cwin = 0;
    for g in 0..6 {
        let our_blue = g % 2 == 0;
        let d = run_game(our_blue, dose_fixed, greedy_pick, 0);
        let c = run_game(our_blue, control_fixed, greedy_pick, 0);
        if d.our_score > d.opp_score {
            dwin += 1;
        }
        if c.our_score > c.opp_score {
            cwin += 1;
        }
        println!(
            "game {}: {} dose {:4.0} vs {:4.0} | control {:4.0} vs {:4.0}",
            g + 1,
            if our_blue { "blue" } else { "red " },
            d.our_score,
            d.opp_score,
            c.our_score,
            c.opp_score
        );
    }
    println!("gauge: dose {dwin}/6, control {cwin}/6");
    println!("GATE (dose 6/6): {}", if dwin == 6 { "PASS" } else { "FAIL" });
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    println!("probe_r_reinforce: v9 REINFORCE D1 (DENSE extension, kill-0 GO 5e43810)");
    println!("mode: {mode}");
    let t0 = std::time::Instant::now();
    match mode.as_str() {
        "selftest" => mode_selftest(),
        "identity" => mode_identity(),
        "census" => mode_census(),
        "h2h" => mode_h2h(),
        "suite" => mode_suite(),
        "league" => mode_league(),
        "gauge" => mode_gauge(),
        "all" => {
            mode_selftest();
            println!();
            mode_identity();
            println!();
            mode_census();
            println!();
            mode_h2h();
            println!();
            mode_suite();
            println!();
            mode_gauge();
            println!();
            mode_league();
        }
        other => panic!("unknown mode {other}"),
    }
    println!("mode {mode} done in {:.0}s", t0.elapsed().as_secs_f64());
}
