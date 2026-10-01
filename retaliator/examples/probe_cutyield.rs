//! Lane 3 CUT-YIELD census (kill-0 step for v9 roadmap): measure the corpus's
//! zero-yield cuts and whether NOT playing them flips game outcomes.
//!
//! A **zero-yield cut** is a first action that breaks an enemy edge while
//! destroying 0 enemy area and gaining 0 own area: the cut costs a tempo and
//! produces nothing on either axis. The proposed dose (ZERO_CUT_PENALTY=1.0
//! x hz) would penalize exactly these first actions.
//!
//! Kill-0 question (roadmap lane 3): if we DON'T play the zero-yield cut,
//! does the game outcome flip? 0 flips = Lane-E disease (the term never
//! changes anything — cf. PATIENCE off-ablation, CUT_MEMORY 6->15, both
//! byte-identical in every gate); >=10% of zero-yield cuts flipping =
//! proceed to dose.
//!
//! Read-only. Two modes:
//! - `count` — census only: cuts by us/opponent, zero-yield cuts by
//!   us/opponent, per-opponent breakdown. Fast.
//! - `flip` (default) — census + counterfactual replay per our zero-yield
//!   cut. At each recorded zero-yield-cut decision point two continuations
//!   run from the same position:
//!     * CONTROL plays the plain current-engine best move (fixed budget);
//!     * DOSE-CF plays the best NON-zero-yield alternative (the dose's
//!       intent, implemented as "zero-yield cuts are not playable").
//!   Both continue with the current engine on our side; the opponent
//!   replays its recorded moves where still legal, else an engine stand-in
//!   (fallbacks counted). A **flip** = outcome class (win/loss/tie) differs
//!   between DOSE-CF and CONTROL — clean attribution, same continuation,
//!   one move different. Also reported: DOSE-CF vs the recorded game
//!   (inflated by engine drift: the recorded corpus was played by older bot
//!   versions), and the pick census (does the current engine also play a
//!   zero-yield cut here — the dose has a target only at those points).
//!
//! Usage: cargo run --release --example probe_cutyield -- <path-or-dir>... [count|flip [cap]]
//! Engine calls use fixed node budgets (deterministic; no clock dependence).

#[path = "support/gamejson.rs"]
mod gamejson;

use gamejson::{game_files, RecordedGame};
use meridian_engine::{Game, Move, Player, Position, notation};
use retaliator::search;
use std::collections::BTreeMap;

/// Deterministic analysis budget for the decision point (never analyze_timed).
const DECISION_BUDGET: usize = 4096;
/// Cheaper deterministic budget for counterfactual continuation moves.
const REPLAY_BUDGET: usize = 1024;
/// Counterfactual replays per game by default (bounded cost; reported if hit).
const DEFAULT_CAP: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OutcomeClass {
    Win,
    Loss,
    Tie,
}

fn outcome_class(ours: f64, theirs: f64) -> OutcomeClass {
    if ours > theirs {
        OutcomeClass::Win
    } else if ours < theirs {
        OutcomeClass::Loss
    } else {
        OutcomeClass::Tie
    }
}

/// Zero-yield first action: breaks an enemy edge, destroys 0 enemy area,
/// gains 0 own area (both against a 1e-9 float tolerance).
fn zero_yield(pos: &Position, mv: Move) -> bool {
    let mut after = pos.clone();
    let oc = after.apply_unchecked(mv);
    if oc.broken.is_none() {
        return false;
    }
    let mover = pos.to_move();
    let gained = after.area(mover).to_f64() - pos.area(mover).to_f64();
    let destroyed = pos.area(mover.opponent()).to_f64() - after.area(mover.opponent()).to_f64();
    destroyed <= 1e-9 && gained <= 1e-9
}

/// Best candidate (current engine, best first) whose first action is NOT a
/// zero-yield cut: the move the dose would produce.
fn best_non_zero_yield(a: &search::Analysis, pos: &Position) -> Option<Move> {
    a.candidates.iter().find(|c| !zero_yield(pos, c.mv)).map(|c| c.mv)
}

/// Fallback when every searched candidate is a zero-yield cut (rare): best
/// non-zero-yield legal move by real production (gained + destroyed), ties by
/// move index. Deterministic.
fn fallback_alt(pos: &Position) -> Option<Move> {
    let mover = pos.to_move();
    let opp = mover.opponent();
    let mut best: Option<(f64, Move)> = None;
    for mv in pos.legal_moves().iter() {
        if zero_yield(pos, mv) {
            continue;
        }
        let mut after = pos.clone();
        let _ = after.apply_unchecked(mv);
        let key = (after.area(mover).to_f64() - pos.area(mover).to_f64())
            + (pos.area(opp).to_f64() - after.area(opp).to_f64());
        let better = match best {
            None => true,
            Some((bk, bm)) => {
                key > bk + 1e-9 || ((key - bk).abs() <= 1e-9 && mv.index() < bm.index())
            }
        };
        if better {
            best = Some((key, mv));
        }
    }
    best.map(|(_, mv)| mv)
}

/// Current engine pick at a fixed replay budget (deterministic).
fn analyze_pick(pos: &Position) -> Option<Move> {
    search::analyze_with_avoid(pos, REPLAY_BUDGET, &[]).candidates.first().map(|c| c.mv)
}

/// Counterfactual continuation from `start` (the position before the recorded
/// zero-yield cut): `first` plays now; the rest continues with the current
/// engine on our side and the opponent's recorded moves where still legal
/// (else the engine stand-in, counted as a fallback). Returns final scores.
fn counterfactual(
    start: &Game,
    first: Move,
    recorded: &[(Move, Player)],
    me: Player,
    fallbacks: &mut usize,
) -> (f64, f64) {
    let mut cf = start.clone();
    cf.play(first).expect("counterfactual first move is legal at the decision point");
    while !cf.is_over() {
        let pos = cf.position().clone();
        let mover = pos.to_move();
        let total = pos.actions_played() as usize;
        let mv = if mover == me {
            analyze_pick(&pos)
        } else {
            // Opponent's recorded move at this action index (the turn
            // structure is deterministic by action count, so the recorded
            // action at index `total` is the opponent's).
            let theirs =
                recorded.get(total).filter(|(_, m)| *m != me).map(|(mv, _)| mv).copied();
            match theirs.filter(|mv| pos.check_move(*mv).is_ok()) {
                Some(mv) => Some(mv),
                None => {
                    *fallbacks += 1;
                    analyze_pick(&pos)
                }
            }
        };
        let Some(mv) = mv else { break };
        cf.play(mv).expect("engine pick is legal in counterfactual");
    }
    let p = cf.position();
    (p.score(me).to_f64(), p.score(me.opponent()).to_f64())
}

#[derive(Default)]
struct Totals {
    games: usize,
    actions: usize,
    cuts_us: usize,
    cuts_opp: usize,
    zy_us: usize,
    zy_opp: usize,
    destroyed_us: f64,
    destroyed_opp: f64,
    // Flip census (our zero-yield cuts, our decisions).
    zy_measured: usize,
    /// Decision points where the current engine's top pick is ALSO a
    /// zero-yield cut (the dose has a target only at those points).
    zy_target: usize,
    /// Zero-yield cuts in games we won / lost (rate-by-outcome split).
    zy_us_win: usize,
    zy_us_loss: usize,
    /// Our cuts in games we won / lost.
    cuts_us_win: usize,
    cuts_us_loss: usize,
    /// Outcome flips vs the CONTROL continuation (clean attribution).
    flip_target: usize,
    /// Outcome flips vs the recorded game (inflated by engine drift).
    flip_recorded: usize,
    /// Fallbacks to the engine stand-in for opponent moves.
    fallbacks: usize,
}

fn census_game(recorded: &RecordedGame, flip: bool, cap: usize, cap_hit: &mut bool, totals: &mut Totals) -> Vec<String> {
    let me = recorded.me;
    let opp = me.opponent();
    let mut game = Game::new();
    let mut zero_yield_at: Vec<usize> = Vec::new();
    let mut cuts_us = 0usize;
    let mut cuts_opp = 0usize;
    let mut zy_us = 0usize;
    let mut zy_opp = 0usize;
    let mut destroyed_us = 0.0f64;
    let mut destroyed_opp = 0.0f64;
    let mut lines: Vec<String> = Vec::new();

    // Pass 1: replay, count cuts and zero-yield cuts.
    for (k, &(mv, mover)) in recorded.actions.iter().enumerate() {
        let before = game.position().clone();
        let my0 = before.area(mover).to_f64();
        let their0 = before.area(mover.opponent()).to_f64();
        let oc = game.play(mv).expect("recorded move is legal");
        let after = game.position();
        let gained = after.area(mover).to_f64() - my0;
        let destroyed = (their0 - after.area(mover.opponent()).to_f64()).max(0.0);
        if oc.broken.is_some() {
            if mover == me {
                cuts_us += 1;
                destroyed_us += destroyed;
                if destroyed <= 1e-9 && gained <= 1e-9 {
                    zy_us += 1;
                    zero_yield_at.push(k);
                }
            } else {
                cuts_opp += 1;
                destroyed_opp += destroyed;
                if destroyed <= 1e-9 && gained <= 1e-9 {
                    zy_opp += 1;
                }
            }
        }
    }
    let final_pos = game.position();
    let recorded_class =
        outcome_class(final_pos.score(me).to_f64(), final_pos.score(opp).to_f64());
    // Zero-yield rate by outcome: do junk cuts correlate with losses? All of
    // a game's cuts share its recorded outcome.
    let (cuts_us_win, cuts_us_loss, zy_us_win, zy_us_loss) = match recorded_class {
        OutcomeClass::Win => (cuts_us, 0, zero_yield_at.len(), 0),
        OutcomeClass::Loss => (0, cuts_us, 0, zero_yield_at.len()),
        OutcomeClass::Tie => (0, 0, 0, 0),
    };
    lines.push(format!(
        "{}: {} actions | cuts us {cuts_us} (zy {zy_us}) / opp {cuts_opp} (zy {zy_opp}) | destroyed us {destroyed_us:.1} opp {destroyed_opp:.1} | final {blue:.0}-{red:.0} {recorded_class:?}",
        recorded.name,
        recorded.actions.len(),
        blue = final_pos.score(me).to_f64(),
        red = final_pos.score(opp).to_f64(),
    ));

    // Pass 2: flip census at our zero-yield decision points.
    let mut per_game = PerGameFlips::default();
    if flip {
        if zero_yield_at.len() > cap {
            *cap_hit = true;
        }
        for &k in zero_yield_at.iter().take(cap) {
            // Re-replay to the decision point.
            let mut game = Game::new();
            for &(mv, _) in recorded.actions.iter().take(k) {
                game.play(mv).expect("recorded move is legal");
            }
            let pos_before = game.position().clone();
            let recorded_mv = recorded.actions[k].0;
            let a = search::analyze_with_avoid(&pos_before, DECISION_BUDGET, &[]);
            let top = a.candidates.first().map(|c| c.mv);
            let top_same = top == Some(recorded_mv);
            let top_zy = top.map(|mv| zero_yield(&pos_before, mv)).unwrap_or(false);
            if top_zy {
                per_game.zy_target += 1;
            }
            let dose_first =
                best_non_zero_yield(&a, &pos_before).or_else(|| fallback_alt(&pos_before));
            let Some(dose_first) = dose_first else { continue };
            let Some(control_first) = top else { continue };
            let (our_dose, opp_dose) =
                counterfactual(&game, dose_first, &recorded.actions, me, &mut per_game.fallbacks);
            let (our_ctrl, opp_ctrl) =
                counterfactual(&game, control_first, &recorded.actions, me, &mut per_game.fallbacks);
            let dose_class = outcome_class(our_dose, opp_dose);
            let ctrl_class = outcome_class(our_ctrl, opp_ctrl);
            let flip_target = dose_class != ctrl_class;
            let flip_recorded = dose_class != recorded_class;
            if flip_target {
                per_game.flip_target += 1;
            }
            if flip_recorded {
                per_game.flip_recorded += 1;
            }
            let txt = notation::move_text(
                recorded_mv.source,
                recorded_mv.target().unwrap_or(recorded_mv.source),
            );
            let dose_txt = notation::move_text(
                dose_first.source,
                dose_first.target().unwrap_or(dose_first.source),
            );
            let mut flags = Vec::new();
            if top_same {
                flags.push("top=same");
            }
            if top_zy {
                flags.push("top=zy");
            }
            if flip_target {
                flags.push("FLIP");
            }
            lines.push(format!(
                "    zy@{:3} {} dose={} | outcome rec {recorded_class:?} dose {dose_class:?} ctrl {ctrl_class:?} | margin rec {:+.0} dose {:+.0} ctrl {:+.0} {}",
                k + 1,
                txt,
                dose_txt,
                final_pos.score(me).to_f64() - final_pos.score(opp).to_f64(),
                our_dose - opp_dose,
                our_ctrl - opp_ctrl,
                flags.join(" "),
            ));
        }
        lines.push(format!(
            "    flip census: {}/{} zero-yield cuts measured (cap {cap}) | targets {} | flips ctrl {} recorded {}",
            zero_yield_at.len().min(cap),
            zero_yield_at.len(),
            per_game.zy_target,
            per_game.flip_target,
            per_game.flip_recorded,
        ));
    }

    totals.games += 1;
    totals.actions += recorded.actions.len();
    totals.cuts_us += cuts_us;
    totals.cuts_opp += cuts_opp;
    totals.zy_us += zy_us;
    totals.zy_opp += zy_opp;
    totals.zy_us_win += zy_us_win;
    totals.zy_us_loss += zy_us_loss;
    totals.cuts_us_win += cuts_us_win;
    totals.cuts_us_loss += cuts_us_loss;
    totals.destroyed_us += destroyed_us;
    totals.destroyed_opp += destroyed_opp;
    if flip {
        totals.zy_measured += zero_yield_at.len().min(cap);
        totals.zy_target += per_game.zy_target;
        totals.flip_target += per_game.flip_target;
        totals.flip_recorded += per_game.flip_recorded;
        totals.fallbacks += per_game.fallbacks;
    }
    lines
}

#[derive(Default)]
struct PerGameFlips {
    zy_target: usize,
    flip_target: usize,
    flip_recorded: usize,
    fallbacks: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode_idx = args.iter().position(|a| a == "count" || a == "flip");
    let (flip, cap): (bool, usize) = match mode_idx {
        Some(i) => {
            let m = args[i] == "flip";
            let cap = args
                .get(i + 1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(DEFAULT_CAP);
            (m, cap)
        }
        None => (true, DEFAULT_CAP),
    };
    let paths: Vec<String> = args
        .into_iter()
        .filter(|a| a != "count" && a != "flip")
        .filter(|a| a.parse::<usize>().is_err())
        .collect();
    assert!(!paths.is_empty(), "usage: probe_cutyield <path-or-dir>... [count|flip [cap]]");

    let files = game_files(&paths);
    let mut cap_hit = false;
    let mut totals = Totals::default();
    let mut per_opp: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();

    for file in &files {
        let Some(recorded) = gamejson::load(file) else {
            println!("skip (neither format): {}", file.display());
            continue;
        };
        let lines = census_game(&recorded, flip, cap, &mut cap_hit, &mut totals);
        for line in &lines {
            println!("{line}");
        }
        let row = per_opp.entry(recorded.opponent.clone()).or_default();
        // Per-opponent tallies come from the same totals; re-derive cheaply
        // by re-counting from the recorded game (cheap: no engine search).
        let me = recorded.me;
        let mut game = Game::new();
        for &(mv, mover) in &recorded.actions {
            let before = game.position().clone();
            let my0 = before.area(mover).to_f64();
            let their0 = before.area(mover.opponent()).to_f64();
            let oc = game.play(mv).expect("recorded move is legal");
            let after = game.position();
            let gained = after.area(mover).to_f64() - my0;
            let destroyed = (their0 - after.area(mover.opponent()).to_f64()).max(0.0);
            if oc.broken.is_some() {
                let zy = destroyed <= 1e-9 && gained <= 1e-9;
                if mover == me {
                    row.0 += 1;
                    row.2 += usize::from(zy);
                } else {
                    row.1 += 1;
                    row.3 += usize::from(zy);
                }
            }
        }
    }

    println!("\n=== CUT-YIELD census ({}) ===", if flip { "flip" } else { "count" });
    println!("games {}, actions {}", totals.games, totals.actions);
    println!(
        "cuts: us {} / opp {} | zero-yield: us {} / opp {} | destroyed from cuts: us {:.1} / opp {:.1}",
        totals.cuts_us, totals.cuts_opp, totals.zy_us, totals.zy_opp, totals.destroyed_us, totals.destroyed_opp
    );
    println!(
        "zero-yield rate by outcome (our cuts): in wins {}/{} = {:.1}% | in losses {}/{} = {:.1}% | overall {:.1}%",
        totals.zy_us_win,
        totals.cuts_us_win,
        pct(totals.zy_us_win, totals.cuts_us_win),
        totals.zy_us_loss,
        totals.cuts_us_loss,
        pct(totals.zy_us_loss, totals.cuts_us_loss),
        pct(totals.zy_us, totals.cuts_us)
    );
    println!("\n-- per opponent (cuts us / cuts opp | zero-yield us / zero-yield opp) --");
    for (name, (cu, co, zu, zo)) in &per_opp {
        println!("{name:14} {cu:3} / {co:3} | {zu:3} / {zo:3}");
    }
    if flip {
        println!("\n-- flip census (our zero-yield cuts) --");
        println!(
            "measured {}/{} (cap {cap}) | current engine also plays a zero-yield cut at {} decision points (dose targets)",
            totals.zy_measured, totals.zy_us, totals.zy_target
        );
        println!(
            "outcome flips vs CONTROL: {}/{} = {:.1}% of measured | {:.1}% of all recorded zero-yield cuts | {:.1}% of dose targets",
            totals.flip_target,
            totals.zy_measured,
            pct(totals.flip_target, totals.zy_measured),
            pct(totals.flip_target, totals.zy_us),
            pct(totals.flip_target, totals.zy_target)
        );
        println!(
            "outcome flips vs RECORDED (drift-inflated): {}/{} = {:.1}% of measured | engine stand-in fallbacks {}",
            totals.flip_recorded,
            totals.zy_measured,
            pct(totals.flip_recorded, totals.zy_measured),
            totals.fallbacks
        );
        println!("\n=== Kill-0 gate (roadmap: >=10% of zero-yield cuts flipping) ===");
        let gate = pct(totals.flip_target, totals.zy_us);
        if totals.flip_target == 0 {
            println!("0 flips — Lane-E disease: zero-yield cuts do not flip outcomes. KILL the dose.");
        } else if gate >= 10.0 {
            println!("{gate:.1}% >= 10% — PROCEED to dose (ZERO_CUT_PENALTY=1.0 x hz).");
        } else {
            println!("{gate:.1}% < 10% — below the gate; coordinator decides (weak signal).");
        }
    }
    if cap_hit {
        println!("\nNOTE: per-game cap {cap} hit — flip rate measured on the capped sample.");
    }
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { n as f64 / d as f64 * 100.0 }
}
