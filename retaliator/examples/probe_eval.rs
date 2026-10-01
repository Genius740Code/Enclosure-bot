//! Lane B eval probe (2026-10-01): the mission measurements, on the shipped
//! `search::best_move` / `search::analyze` (doom ON — the site bot).
//!
//! Modes (argv[1], default `all`):
//!
//! - `census` — phase-aware term snapshot + single-term ablation over a
//!   league-style corpus (8 games: skip 0/10/20/30 x both colors, shipped
//!   vs scoutbase). For each sampled decision point: `search::analyze`
//!   lists the top candidates, `replay_adjusted` re-scores them with one
//!   term on at a time, and the probe counts pick flips. A fidelity line
//!   first proves the replay reproduces the shipped ordering (argmax of
//!   the default-term replay == analyze's pick) before any flip counts.
//! - `cutline` — THE key hypothesis, quantified: at spread decision
//!   points, enumerate two-action lines, take the best cut+make line
//!   (destroys enemy area AND gains own) and the best make-only line by
//!   the shipped static value, play BOTH out to game end with the shipped
//!   search for both sides, and measure the TRUE banked-score swing
//!   (final margin difference). The shipped pricing's denial component
//!   (`destroyed x beyond-horizon events x 0.5`) is linear in a denial
//!   weight w, so each point yields an implied w* where the priced swing
//!   matches the true swing; the aggregate answers whether the symmetric
//!   0.5 is right (w* > 1: denial deserves more; < 1: less).
//!
//! Usage: cargo run --release --example probe_eval [census|cutline|all]

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Move, Player, Position};
use retaliator::search;

use eval_phases::{ProbeTerms, TermSnapshot};

fn sgn(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}

/// The shipped side's decision points (turn starts: two actions in hand).
fn is_our_turn_start(pos: &Position, shipped_blue: bool) -> bool {
    (pos.to_move() == Player::Blue) == shipped_blue && pos.actions_left_in_turn() == 2
}

// ---------------------------------------------------------------------
// census mode
// ---------------------------------------------------------------------

/// Ablation configurations: one term on at a time (plus the deny/claim
/// sweep), against the default (shipped arithmetic).
fn ablation_terms() -> Vec<(&'static str, ProbeTerms)> {
    let mut base = ProbeTerms::default();
    base.close_w = 2.0;
    let mut close_half = ProbeTerms::default();
    close_half.close_w = 0.5;
    let mut mob = ProbeTerms::default();
    mob.mob_w = 0.05;
    let mut vuln = ProbeTerms::default();
    vuln.vuln_edge_w = 4.0;
    let mut node = ProbeTerms::default();
    node.node_w = 3.0;
    let mut frontier = ProbeTerms::default();
    frontier.frontier_w = 2.0;
    let mut deny_half = ProbeTerms::default();
    deny_half.deny_w = 0.5;
    let mut deny_two = ProbeTerms::default();
    deny_two.deny_w = 2.0;
    let mut deny_zero = ProbeTerms::default();
    deny_zero.deny_w = 0.0;
    let mut claim_zero = ProbeTerms::default();
    claim_zero.claim_w = 0.0;
    vec![
        ("base", ProbeTerms::default()),
        ("close2", base),
        ("close0.5", close_half),
        ("mob0.05", mob),
        ("vuln4", vuln),
        ("node3", node),
        ("frontier2", frontier),
        ("deny0.5", deny_half),
        ("deny2", deny_two),
        ("deny0", deny_zero),
        ("claim0", claim_zero),
    ]
}

fn census() {
    let configs = ablation_terms();
    let mut flips = vec![0u32; configs.len()];
    let mut fidelity_ok = 0u32;
    let mut fidelity_total = 0u32;
    let mut snap_n = [0u32; 3];
    let mut snap_sum = [[0.0f64; 12]; 3]; // per phase: area/closable/mob/cut/front/nodes x 2
    let mut turn = 0u32;

    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() {
                    break;
                }
                let mv = search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                continue;
            }
            let mut sampled = 0u32;
            while !game.is_over() {
                let pos = game.position().clone();
                let shipped_turn = is_our_turn_start(&pos, ret_blue);
                if shipped_turn {
                    sampled += 1;
                    turn += 1;
                    if turn % 2 == 1 {
                        let s: TermSnapshot = eval_phases::term_snapshot(&pos);
                        let ph = s.phase.index();
                        snap_n[ph] += 1;
                        for p in 0..2 {
                            snap_sum[ph][0 + p] += s.area[p];
                            snap_sum[ph][2 + p] += s.closable[p] as f64;
                            snap_sum[ph][4 + p] += s.mobility[p] as f64;
                            snap_sum[ph][6 + p] += s.cuttable[p] as f64;
                            snap_sum[ph][8 + p] += s.frontier[p] as f64;
                            snap_sum[ph][10 + p] += s.nodes[p] as f64;
                        }
                    }
                    if pos.actions_played() >= 12 {
                        let analysis = search::analyze(&pos, search::MOVE_BUDGET);
                        let cands: Vec<(Move, f64, Vec<Move>)> = analysis
                            .candidates
                            .iter()
                            .map(|c| (c.mv, c.evaluation, c.pv.clone()))
                            .collect();
                        if let Some(pick) = analysis.candidates.first().map(|c| c.mv) {
                            if !cands.is_empty() {
                                let argmax = |terms: &ProbeTerms| -> Move {
                                    let mut best: Option<(f64, usize, Move)> = None;
                                    for (mv, evaluation, pv) in cands.iter() {
                                        let adj = eval_phases::replay_adjusted(&pos, *mv, *evaluation, pv, terms);
                                        let id = mv.index();
                                        if best.is_none_or(|(b, bi, _)| adj > b || (adj == b && id < bi)) {
                                            best = Some((adj, id, *mv));
                                        }
                                    }
                                    best.expect("nonempty candidates").2
                                };
                                let base = &configs[0].1;
                                fidelity_total += 1;
                                if argmax(base) == pick {
                                    fidelity_ok += 1;
                                } else {
                                    println!(
                                        "REPLAY DRIFT at actions={} phase={}: replay picks {}, analyze picks {}",
                                        pos.actions_played(),
                                        eval_phases::phase(&pos).name(),
                                        argmax(base).index(),
                                        pick.index()
                                    );
                                }
                                for (i, (_, terms)) in configs.iter().enumerate().skip(1) {
                                    if argmax(terms) != pick {
                                        flips[i] += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                let mv = if shipped_turn {
                    search::best_move(&pos)
                } else {
                    scoutbase::best_move(&pos)
                };
                game.play(mv.unwrap()).unwrap();
            }
            println!("corpus game done: skip={skip} ret={} ({sampled} shipped turn-starts)", if ret_blue { "blue" } else { "red" });
        }
    }

    println!();
    println!("replay fidelity: {fidelity_ok}/{fidelity_total} positions (default terms reproduce the shipped pick)");
    println!("single-term pick flips over {fidelity_total} decision points:");
    for (i, (name, _)) in configs.iter().enumerate().skip(1) {
        println!("  {name:<10} flips {:3} ({:5.1}%)", flips[i], 100.0 * flips[i] as f64 / fidelity_total.max(1) as f64);
    }
    println!();
    println!("term snapshot means per phase ([Blue, Red]):");
    for (ph, name) in ["opening", "middlegame", "endgame"].iter().enumerate() {
        if snap_n[ph] == 0 {
            println!("  {name}: (no sampled points)");
            continue;
        }
        let n = snap_n[ph] as f64;
        let s = &snap_sum[ph];
        println!(
            "  {name:<11} n={:3} area [{:5.1},{:5.1}] closable [{:4.1},{:4.1}] mobility [{:5.1},{:5.1}] cuttable [{:4.1},{:4.1}] frontier [{:4.1},{:4.1}] nodes [{:4.1},{:4.1}]",
            snap_n[ph],
            s[0] / n, s[1] / n,
            s[2] / n, s[3] / n,
            s[4] / n, s[5] / n,
            s[6] / n, s[7] / n,
            s[8] / n, s[9] / n,
            s[10] / n, s[11] / n,
        );
    }
}

// ---------------------------------------------------------------------
// cutline mode: the key hypothesis, quantified
// ---------------------------------------------------------------------

/// A two-action line from a turn-start decision point.
#[derive(Clone)]
struct Line {
    pv: Vec<Move>,
    /// Own area gained over the line.
    gain: f64,
    /// Enemy area destroyed over the line.
    destroyed: f64,
    /// Mover-relative shipped static value at the line's leaf.
    value: f64,
    end: Position,
}

/// The mover's two-action lines: EVERY legal first action that breaks an
/// enemy edge (the cut pool — cut+make lines are the rare type, so none
/// are discarded), plus the top 8 area-gaining firsts by value (the make
/// pool), each answered by the best second action by the shipped static
/// value.
fn enumerate_lines(pos: &Position) -> Vec<Line> {
    let mover = pos.to_move();
    let s = sgn(mover);
    struct First {
        mv: Move,
        broke: bool,
        gain: f64,
        v: f64,
        after: Position,
    }
    let mut cut: Vec<First> = Vec::new();
    let mut make: Vec<First> = Vec::new();
    for mv in pos.legal_moves().iter() {
        let mut after = pos.clone();
        let oc = after.apply_unchecked(mv);
        let gain = after.area(mover).to_f64() - pos.area(mover).to_f64();
        let v = s * search::evaluate(&after);
        let first = First { mv, broke: oc.broken.is_some(), gain, v, after };
        if first.broke {
            cut.push(first);
        } else if first.gain > 0.5 {
            make.push(first);
        }
    }
    let mut by_v = |mut v: Vec<First>| {
        v.sort_by(|a, b| b.v.total_cmp(&a.v).then(a.mv.index().cmp(&b.mv.index())));
        v.truncate(8);
        v
    };
    // Cut+make lines are the rare type, so the cut pool keeps 3x the make
    // pool's width; both are sorted by first-action value first.
    let mut cut = cut;
    cut.sort_by(|a, b| b.v.total_cmp(&a.v).then(a.mv.index().cmp(&b.mv.index())));
    cut.truncate(24);
    let mut lines: Vec<Line> = Vec::new();
    for first in cut.into_iter().chain(by_v(make)) {
        if lines.iter().any(|l: &Line| l.pv[0].index() == first.mv.index()) {
            continue; // a first can sit in both pools
        }
        let mut best: Option<(f64, usize, Move)> = None;
        for reply in first.after.legal_moves().iter() {
            let mut end = first.after.clone();
            end.apply_unchecked(reply);
            let v = s * search::evaluate(&end);
            let id = reply.index();
            if best.is_none_or(|(b, bi, _)| v > b || (v == b && id < bi)) {
                best = Some((v, id, reply));
            }
        }
        let pv = match best {
            Some((_, _, reply)) => vec![first.mv, reply],
            None => vec![first.mv], // no replies: the 1-action line
        };
        let mut end = pos.clone();
        for &mv in &pv {
            end.apply_unchecked(mv);
        }
        let gain = end.area(mover).to_f64() - pos.area(mover).to_f64();
        let destroyed =
            (pos.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
        let value = s * search::evaluate(&end);
        lines.push(Line { pv, gain, destroyed, value, end });
    }
    lines
}

/// Plays the position out with the shipped search for BOTH sides; the
/// final score margin from `mover`'s perspective.
fn playout_margin(pos: &Position, mover: Player) -> f64 {
    let mut game = Game::from_position(pos.clone());
    while !game.is_over() {
        let Some(mv) = search::best_move(game.position()) else { break };
        if game.play(mv).is_err() {
            break;
        }
    }
    let p = game.position();
    p.score(mover).to_f64() - p.score(mover.opponent()).to_f64()
}

/// Decision points: every 4th shipped-side turn start in the middlegame
/// window [16, 96), every 2nd in the endgame window [98, 118) — deduped
/// globally, because this harness's blue-side games from different skips
/// are known to converge to identical lines.
fn cutline() {
    struct Row {
        phase: &'static str,
        events_left: u8,
        gain_c: f64,
        dest_c: f64,
        gain_m: f64,
        margin_c: f64,
        margin_m: f64,
        true_swing: f64,
        a_diff: f64,
        b_diff: f64,
        implied_w: Option<f64>,
        shipped_pick_cutmake: bool,
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut skipped = [0u32; 3];
    let mut no_cutmake = [0u32; 3];
    let mut no_make = [0u32; 3];
    let mut seen: std::collections::HashSet<Position> = std::collections::HashSet::new();

    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() {
                    break;
                }
                let mv = search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                continue;
            }
            let mut targets: Vec<Position> = Vec::new();
            let (mut mid_n, mut end_n) = (0u32, 0u32);
            while !game.is_over() {
                let pos = game.position().clone();
                let shipped_turn = is_our_turn_start(&pos, ret_blue);
                if shipped_turn {
                    let a = pos.actions_played();
                    if (16..96).contains(&a) && {
                        mid_n += 1;
                        mid_n % 3 == 0
                    } {
                        targets.push(pos.clone());
                    }
                    if a >= 98 && {
                        end_n += 1;
                        true
                    } {
                        targets.push(pos.clone());
                    }
                }
                let mv = if shipped_turn {
                    search::best_move(&pos)
                } else {
                    scoutbase::best_move(&pos)
                };
                game.play(mv.unwrap()).unwrap();
            }
            for pos in targets {
                if !seen.insert(pos.clone()) {
                    continue; // identical line already measured
                }
                let ph = eval_phases::phase(&pos);
                if ph == eval_phases::Phase::Opening {
                    continue;
                }
                let mover = pos.to_move();
                let lines = enumerate_lines(&pos);
                let best_of = |pred: &dyn Fn(&Line) -> bool| -> Option<Line> {
                    lines
                        .iter()
                        .filter(|l| pred(l))
                        .max_by(|x, y| x.value.total_cmp(&y.value))
                        .cloned()
                };
                let is_cutmake = |l: &Line| l.destroyed >= 0.5 && l.gain >= 0.5;
                let is_makeonly = |l: &Line| l.destroyed < 0.5 && l.gain >= 0.5;
                let has_cutmake = lines.iter().any(|l| is_cutmake(l));
                let has_make = lines.iter().any(|l| is_makeonly(l));
                let (Some(best_c), Some(best_m)) = (best_of(&is_cutmake), best_of(&is_makeonly)) else {
                    skipped[ph.index()] += 1;
                    if !has_cutmake {
                        no_cutmake[ph.index()] += 1;
                    }
                    if !has_make {
                        no_make[ph.index()] += 1;
                    }
                    continue;
                };
                let pick_cutmake = search::analyze(&pos, search::MOVE_BUDGET)
                    .candidates
                    .first()
                    .is_some_and(|c| {
                        let mut end = pos.clone();
                        for &mv in &c.pv {
                            end.apply_unchecked(mv);
                        }
                        let gain = end.area(mover).to_f64() - pos.area(mover).to_f64();
                        let destroyed =
                            (pos.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
                        destroyed >= 0.5 && gain >= 0.5
                    });
                let margin_c = playout_margin(&best_c.end, mover);
                let margin_m = playout_margin(&best_m.end, mover);
                let true_swing = margin_c - margin_m;
                // pricing: A = mover-relative leaf value (w-independent),
                // B = destroyed x beyond-horizon events x HORIZON_WEIGHT (0.5)
                let price = |l: &Line| {
                    let ev = f64::from(l.end.scoring_events_left());
                    let beyond = ev - ev.min(12.0);
                    (l.value, l.destroyed * beyond * 0.5)
                };
                let (a_c, b_c) = price(&best_c);
                let (a_m, b_m) = price(&best_m);
                let a_diff = a_c - a_m;
                let b_diff = b_c - b_m;
                let implied_w = if b_diff.abs() > 1.0 {
                    Some((true_swing - a_diff) / b_diff)
                } else {
                    None
                };
                rows.push(Row {
                    phase: ph.name(),
                    events_left: pos.scoring_events_left(),
                    gain_c: best_c.gain,
                    dest_c: best_c.destroyed,
                    gain_m: best_m.gain,
                    margin_c,
                    margin_m,
                    true_swing,
                    a_diff,
                    b_diff,
                    implied_w,
                    shipped_pick_cutmake: pick_cutmake,
                });
            }
        }
    }

    println!("cut+make vs make-only — true banked-score swing (playouts: shipped search both sides)");
    println!(
        "n={} decision points (skipped {}/{} mid/end: no cut+make line {}/{}, no make-only {}/{})",
        rows.len(),
        skipped[1],
        skipped[2],
        no_cutmake[1],
        no_cutmake[2],
        no_make[1],
        no_make[2]
    );
    println!(
        "{:<11} {:>3} {:>20} {:>18} {:>8} {:>8} {:>7} {:>6}",
        "phase", "ev", "cut g/d -> margin", "make g -> margin", "swing", "A diff", "B diff", "pick"
    );
    for r in &rows {
        println!(
            "{:<11} {:>3} {:>6.1}/{:<5.1}->{:>6.0} {:>6.1}->{:>7.0} {:>+8.0} {:>+8.1} {:>+7.1} {:>6}",
            r.phase,
            r.events_left,
            r.gain_c,
            r.dest_c,
            r.margin_c,
            r.gain_m,
            r.margin_m,
            r.true_swing,
            r.a_diff,
            r.b_diff,
            if r.shipped_pick_cutmake { "cm" } else { "-" }
        );
    }
    println!();
    for name in ["middlegame", "endgame"] {
        let cell: Vec<&Row> = rows.iter().filter(|r| r.phase == name).collect();
        if cell.is_empty() {
            println!("{name}: no points");
            continue;
        }
        let n = cell.len() as f64;
        let mean = |f: fn(&Row) -> f64| cell.iter().map(|r| f(r)).sum::<f64>() / n;
        let swing_mean = mean(|r| r.true_swing);
        let a_mean = mean(|r| r.a_diff);
        let b_mean = mean(|r| r.b_diff);
        let swing_pos = cell.iter().filter(|r| r.true_swing > 0.0).count();
        let mut ws: Vec<f64> = cell.iter().filter_map(|r| r.implied_w).collect();
        ws.sort_by(|a, b| a.total_cmp(b));
        let (w_med, w_lo, w_hi) = if ws.is_empty() {
            (f64::NAN, f64::NAN, f64::NAN)
        } else {
            (ws[ws.len() / 2], ws[ws.len() / 4], ws[3 * ws.len() / 4])
        };
        // trimmed mean of w* (drop the most chaotic decile each side)
        let w_trim = if ws.len() >= 10 {
            let k = ws.len() / 10;
            ws[k..ws.len() - k].iter().sum::<f64>() / (ws.len() - 2 * k) as f64
        } else {
            f64::NAN
        };
        let agree = |w: f64| {
            cell.iter()
                .filter(|r| (r.a_diff + w * r.b_diff > 0.0) == (r.true_swing > 0.0))
                .count()
        };
        let picked_cm = cell.iter().filter(|r| r.shipped_pick_cutmake).count();
        println!(
            "{name}: n={} | true swing mean {swing_mean:+.0} (cut+make better in {swing_pos}/{}) | priced: A(w-independent) mean {a_mean:+.0}, B(w x destroyed x beyond x 0.5) mean {b_mean:+.0} | priced swing at w=1: {:+.0} | implied w* median {w_med:+.2} [p25 {w_lo:+.2}, p75 {w_hi:+.2}], trimmed mean {w_trim:+.2} (n_w={}) | sign agree w=1: {}/{} | shipped already picks the cut+make line: {picked_cm}/{}",
            cell.len(),
            a_mean + b_mean,
            ws.len(),
            agree(1.0),
            cell.len(),
            cell.len(),
            cell.len()
        );
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    let term = std::env::args().nth(2);
    match mode.as_str() {
        "census" => census(),
        "cutline" => cutline(),
        "league" => league_gate(term.as_deref().unwrap_or("base")),
        "gauge" => gauge_gate(term.as_deref().unwrap_or("base")),
        "h2h" => h2h_gate(term.as_deref().unwrap_or("base")),
        _ => {
            census();
            println!();
            cutline();
        }
    }
}

// ---------------------------------------------------------------------
// gate modes: the replay variant as a playing bot (one term at a time)
// ---------------------------------------------------------------------

fn terms_by_name(name: &str) -> ProbeTerms {
    ablation_terms()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| t)
        .unwrap_or_default()
}

/// The shipped bot with `terms` applied: the shipped opener, then
/// `search::analyze`'s top candidates re-scored by `replay_adjusted`
/// (which reproduces the shipped pick exactly with default terms —
/// probe_eval census fidelity 199/199 over the corpus).
fn variant_best_move(pos: &Position, terms: &ProbeTerms) -> Option<Move> {
    if let Some(open) = eval_phases::opener(pos) {
        return Some(open);
    }
    let analysis = search::analyze(pos, search::MOVE_BUDGET);
    let mut best: Option<(f64, usize, Move)> = None;
    for c in &analysis.candidates {
        let adj = eval_phases::replay_adjusted(pos, c.mv, c.evaluation, &c.pv, terms);
        let id = c.mv.index();
        if best.is_none_or(|(b, bi, _)| adj > b || (adj == b && id < bi)) {
            best = Some((adj, id, c.mv));
        }
    }
    best.map(|(_, _, mv)| mv)
}

/// League gate: the probe_league pattern — 8 games vs scoutbase, skip
/// prelude in shipped self-play, both colors, AVG margin from the
/// variant's perspective. With `base` this must reproduce probe_league's
/// rows exactly (the harness self-check).
fn league_gate(term: &str) {
    let terms = terms_by_name(term);
    let mut sum = 0.0;
    let mut n = 0;
    let mut wins = 0;
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() {
                    break;
                }
                let mv = search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                continue;
            }
            let t = std::time::Instant::now();
            while !game.is_over() {
                let pos = game.position().clone();
                let our_turn = (pos.to_move() == Player::Blue) == ret_blue;
                let mv = if our_turn {
                    variant_best_move(&pos, &terms)
                } else {
                    scoutbase::best_move(&pos)
                };
                game.play(mv.unwrap()).unwrap();
            }
            let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
            let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
            let m = (rs - ss) / rs * 100.0;
            if rs > ss {
                wins += 1;
            }
            println!("league[{term}] skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% ({:.0}s)", if ret_blue { "blue" } else { "red" }, t.elapsed().as_secs_f64());
            sum += m;
            n += 1;
        }
    }
    println!("league[{term}] AVG margin: {:+.1}% | W/L {wins}/{n}", sum / n as f64);
}

/// 1-ply greedy (the gauge's blob archetype), identical to gauge.rs.
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

/// Gauge gate: the variant vs greedy, alternating colors.
fn gauge_gate(term: &str) {
    let terms = terms_by_name(term);
    let games: usize = std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(6);
    let mut wins = 0;
    for g in 0..games {
        let us_blue = g % 2 == 0;
        let t = std::time::Instant::now();
        let mut game = Game::new();
        while !game.is_over() {
            let pos = game.position().clone();
            let our_turn = (pos.to_move() == Player::Blue) == us_blue;
            let Some(mv) = (if our_turn {
                variant_best_move(&pos, &terms)
            } else {
                greedy(&pos)
            }) else {
                break;
            };
            game.play(mv).unwrap();
        }
        let b = game.position().score(Player::Blue).to_f64();
        let r = game.position().score(Player::Red).to_f64();
        let (rs, gs) = if us_blue { (b, r) } else { (r, b) };
        if rs > gs {
            wins += 1;
        }
        println!(
            "gauge[{term}] game {}: us={} {rs:.0}-{gs:.0} margin={:+.1}% ({:.0}s)",
            g + 1,
            if us_blue { "blue" } else { "red" },
            (rs - gs) / rs * 100.0,
            t.elapsed().as_secs_f64()
        );
    }
    println!("gauge[{term}]: {wins}/{games}");
}

/// h2h gate: the variant vs the shipped search over the site's openings
/// (the probe_b_v3all pattern, against `search::best_move` itself). With
/// `base` both sides play identically — every opening must give the
/// same score both ways around (the self-play line); anything else is
/// replay drift.
fn h2h_gate(term: &str) {
    let terms = terms_by_name(term);
    let mut wins = 0;
    let mut n = 0;
    let (mut wb, mut nb, mut wr, mut nr) = (0, 0, 0, 0);
    let (mut sum_b, mut sum_r) = (0.0, 0.0);
    for open in [None, Some(4864usize), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            while !game.is_over() {
                let pos = game.position().clone();
                let a_turn = (pos.to_move() == Player::Blue) == a_blue;
                let mv = if a_turn {
                    variant_best_move(&pos, &terms)
                } else {
                    search::best_move(&pos)
                };
                game.play(mv.unwrap()).unwrap();
            }
            let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
            let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
            let m = (asc - bsc) / asc * 100.0;
            n += 1;
            if asc > bsc {
                wins += 1;
                if a_blue { wb += 1 } else { wr += 1 }
            }
            if a_blue {
                nb += 1;
                sum_b += m;
            } else {
                nr += 1;
                sum_r += m;
            }
            println!(
                "h2h[{term}] open={open:?} a={} {asc:.0}-{bsc:.0} margin={m:+.1}% {}",
                if a_blue { "blue" } else { "red" },
                if asc > bsc { "A WINS" } else { "b wins" }
            );
        }
    }
    println!(
        "h2h[{term}]: {wins}/{n} | as blue {wb}/{nb} avg {:+.1}% | as red {wr}/{nr} avg {:+.1}%",
        sum_b / nb.max(1) as f64,
        sum_r / nr.max(1) as f64
    );
}
