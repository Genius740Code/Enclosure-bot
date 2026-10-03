//! p1_pair2: P1 pair-regret census on our double-action turns (v9 analysis lane).
//!
//! Question (idea-backlog P1 step-0): on our 2-action turns the bot plans
//! greedily per action — move 1 picked by the search (whose PV sees the pair),
//! move 2 re-planned from scratch. Does the jointly best pair beat
//! greedy-then-replan, how often, and by how much?
//!
//! Method: at every our-to-move turn START (actions_left_in_turn == 2) in the
//! 46-game local corpus, replayed from the recorded moves —
//! - GREEDY: mv1 = the search's top pick (WIDTH=8, fixed budget, deterministic),
//!   mv2 = a fresh re-plan from the post-mv1 position (current behaviour).
//! - JOINT: the same re-planned mv2 after EACH of the top-8 root candidates;
//!   the best pair wins. Regret = joint pair value - greedy pair value
//!   (non-negative by construction: the greedy pair is in the candidate set).
//! - Pair value = sign(mover) * value(after pair) + horizon_extension(root,
//!   after pair) - doom discount at the post-pair position (the search's own
//!   selection machinery; frozen verbatim copies, same practice as
//!   cut_census.rs). Also recorded: does the re-planned mv2 equal the root
//!   PV's assumed mv2 (the mechanism check)?
//!
//! Usage: cargo run --release --example p1_pair2 -- <corpus dir>
//! Read-only: replays recorded games, changes no bot behaviour, no src edits.

use meridian_engine::{Game, Move, MoveKind, Outcome, Player, Point, Position};
use retaliator::search::evaluate;
use std::cmp::Reverse;

// FROZEN constants (values verbatim from search.rs @ 2a41473).
const MOVE_BUDGET: usize = 4096;
const SMALLEST_BUDGET: usize = 16;
const WIDTH: usize = 8;
const HORIZON: f64 = 12.0;
const HORIZON_WEIGHT: f64 = 0.5;
const CONTACT_PENALTY: f64 = 1.0;
const DEADWOOD_PENALTY: f64 = 2.0;
const DEADWOOD_ENEMY_DIST: i8 = 3;
const CUT_RADIUS: i8 = 3;
const REBUILD_PENALTY: f64 = 3.0;
const FRESH_RADIUS: i8 = 2;
const FRESH_PENALTY: f64 = 1.5;
const IDLE_ROOM: f64 = 0.5;
const IDLE_ENEMY_DIST: i8 = 3;
const IDLE_PENALTY: f64 = 2.0;
const FIGHT_DIST: i8 = 4;
const FIGHT_HEAT: usize = 10;
const REMOTE_DIST: i8 = 5;
const REMOTE_BONUS: f64 = 1.0;
const DENSE_DIST: i8 = 1;
const DENSE_COUNT: u32 = 2;
const DENSE_BONUS: f64 = 1.0;
const PATIENCE_MAX_GAIN: f64 = 2.0;
const PATIENCE_WINDOW: u8 = 12;
const PATIENCE_PENALTY: f64 = 2.0;

/// Joint candidates per double turn (top root candidates).
const JOINT_K: usize = 8;
/// Regret thresholds (selection units; value() ~ area x hz scale).
const REGRET_SMALL: f64 = 0.5;
const REGRET_BIG: f64 = 5.0;

/// Measure one of OUR double turns from the turn root.
/// Returns (joint-greedy pair regret, re-planned mv2 == root PV's assumed mv2).
fn measure_turn(root: &Position, me: Player) -> (f64, bool) {
    let candidates = analyze_full(root, MOVE_BUDGET);
    let Some((mv1_g, pv_g)) = candidates.first() else {
        return (0.0, true);
    };
    let pos1 = position_after_pv(root, &[*mv1_g]);
    // Greedy: mv2 re-planned from scratch (current behaviour).
    let Some(mv2_g) = analyze_with_width(&pos1, MOVE_BUDGET, WIDTH, &[]) else {
        return (0.0, true);
    };
    let pos2_g = position_after_pv(&pos1, &[mv2_g]);
    let v_greedy = pair_value(root, &pos2_g, me);

    // Mechanism check: does the re-plan equal the root PV's assumed mv2?
    let pv_assumed = pv_g.get(1).copied();
    let mv2_matches_pv = pv_assumed == Some(mv2_g);

    // Joint: the same re-planned mv2 after each of the top-K root candidates.
    let mut v_joint = v_greedy;
    for (mv1_k, _) in candidates.iter().take(JOINT_K) {
        let pos1_k = position_after_pv(root, &[*mv1_k]);
        let Some(mv2_k) = analyze_with_width(&pos1_k, MOVE_BUDGET, WIDTH, &[]) else {
            continue;
        };
        let pos2_k = position_after_pv(&pos1_k, &[mv2_k]);
        let v_k = pair_value(root, &pos2_k, me);
        if v_k > v_joint {
            v_joint = v_k;
        }
    }
    ((v_joint - v_greedy).max(0.0), mv2_matches_pv)
}

/// FROZEN verbatim `analyze_with_avoid` @ 2a41473 (the pieces this census
/// needs: candidates with PVs, D1 tie-break, all selection terms).
fn analyze_full(position: &Position, budget: usize) -> Vec<(Move, Vec<Move>)> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, &[]);
    let width = first_actions.len().min(WIDTH);
    let reply_budget = budget
        .checked_sub(first_actions.len())
        .map(|r| r / width.max(1))
        .unwrap_or(usize::MAX);
    let mut scored: Vec<(f64, Move, Vec<Move>)> = Vec::new();
    for (mv, after, _, _) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false, &[]);
        let mut pv = vec![mv];
        let mut evaluation = value(&after);
        if let Some((reply, _, reply_value, _)) = replies.first() {
            evaluation = *reply_value;
            pv.push(*reply);
        }
        let end = position_after_pv(position, &pv);
        let mut adjusted = sign(mover) * evaluation + horizon_extension(position, &end, mover);
        {
            let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
            let mut probe = position.clone();
            let oc = probe.apply_unchecked(mv);
            if oc.broken.is_none() {
                let tgt = mv.target().expect("legal moves end on the board");
                if near_fresh_enemy(position, tgt) {
                    adjusted -= FRESH_PENALTY * hz_sel;
                }
            }
        }
        {
            let doom_at = if end.to_move() == opp {
                Some(&end)
            } else if after.to_move() == opp {
                Some(&after)
            } else {
                None
            };
            if let Some(pos) = doom_at {
                let events_left = f64::from(pos.scoring_events_left());
                let hz_doom = events_left.min(HORIZON);
                let tail = (events_left - hz_doom) * 0.5;
                adjusted -= 1.0 * max_pop(pos, mover) * (hz_doom + tail);
            }
        }
        scored.push((adjusted, mv, pv));
    }
    scored.sort_by(|a, b| {
        let gap = (b.0 - a.0).abs();
        if gap < 1e-9 {
            let da = distance_to_center(&a.1);
            let db = distance_to_center(&b.1);
            da.total_cmp(&db).then(a.1.index().cmp(&b.1.index()))
        } else {
            b.0.total_cmp(&a.0).then(a.1.index().cmp(&b.1.index()))
        }
    });
    scored.into_iter().map(|(_, mv, pv)| (mv, pv)).collect()
}

/// FROZEN verbatim `analyze_with_avoid` @ 2a41473 with the reply width raised
/// to `width_param`; returns only the top pick.
fn analyze_with_width(
    position: &Position,
    budget: usize,
    width_param: usize,
    avoid: &[Point],
) -> Option<Move> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, avoid);
    let width = first_actions.len().min(width_param);
    let reply_budget = budget
        .checked_sub(first_actions.len())
        .map(|r| r / width.max(1))
        .unwrap_or(usize::MAX);
    let mut scored: Vec<(f64, Move)> = Vec::new();
    for (mv, after, _, _) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false, &[]);
        let mut evaluation = value(&after);
        if let Some((reply, _, reply_value, _)) = replies.first() {
            evaluation = *reply_value;
        }
        let end = position_after_pv(position, &[mv]);
        let mut adjusted = sign(mover) * evaluation + horizon_extension(position, &end, mover);
        {
            let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
            let mut probe = position.clone();
            let oc = probe.apply_unchecked(mv);
            if oc.broken.is_none() {
                let tgt = mv.target().expect("legal moves end on the board");
                if near_points(avoid.iter().copied(), tgt, CUT_RADIUS) {
                    adjusted -= REBUILD_PENALTY * hz_sel;
                }
                if near_fresh_enemy(position, tgt) {
                    adjusted -= FRESH_PENALTY * hz_sel;
                }
            }
        }
        {
            let doom_at = if end.to_move() == opp {
                Some(&end)
            } else if after.to_move() == opp {
                Some(&after)
            } else {
                None
            };
            if let Some(pos) = doom_at {
                let events_left = f64::from(pos.scoring_events_left());
                let hz_doom = events_left.min(HORIZON);
                let tail = (events_left - hz_doom) * 0.5;
                adjusted -= 1.0 * max_pop(pos, mover) * (hz_doom + tail);
            }
        }
        scored.push((adjusted, mv));
    }
    scored.sort_by(|a, b| {
        let gap = (b.0 - a.0).abs();
        if gap < 1e-9 {
            let da = distance_to_center(&a.1);
            let db = distance_to_center(&b.1);
            da.total_cmp(&db).then(a.1.index().cmp(&b.1.index()))
        } else {
            b.0.total_cmp(&a.0).then(a.1.index().cmp(&b.1.index()))
        }
    });
    scored.first().map(|(_, mv)| *mv)
}

fn position_after_pv(position: &Position, pv: &[Move]) -> Position {
    let mut pos = position.clone();
    for mv in pv {
        pos.apply_unchecked(*mv);
    }
    pos
}

/// The pair outcome value from the mover's perspective at the turn root:
/// honest leaf value after the pair + horizon extension - doom discount at
/// the post-pair position (where the opponent is to move).
fn pair_value(root: &Position, after: &Position, mover: Player) -> f64 {
    let opp = mover.opponent();
    let mut v = sign(mover) * value(after) + horizon_extension(root, after, mover);
    if after.to_move() == opp {
        let events_left = f64::from(after.scoring_events_left());
        let hz_doom = events_left.min(HORIZON);
        let tail = (events_left - hz_doom) * 0.5;
        v -= 1.0 * max_pop(after, mover) * (hz_doom + tail);
    }
    v
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = args.first().map(String::as_str).unwrap_or("/home/genius74o/game");
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .expect("corpus dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            (name.starts_with("chall-") || name.starts_with("R-chall-")) && name.ends_with(".moves.json")
        })
        .collect();
    files.sort();

    let mut turns_total = 0usize;
    let mut mismatch = 0usize; // re-planned mv2 != root PV's assumed mv2
    let mut regret_gt_small = 0usize;
    let mut regret_gt_big = 0usize;
    let mut regrets: Vec<f64> = Vec::new();
    let mut per_chair: [(usize, usize, usize, usize, Vec<f64>); 2] = Default::default();

    for file in &files {
        let raw = std::fs::read_to_string(file).expect("read corpus file");
        let d: serde_json::Value = serde_json::from_str(&raw).expect("parse corpus file");
        let name = file.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let me = if name.contains("botIsblue") {
            Player::Blue
        } else if name.contains("botIsred") {
            Player::Red
        } else {
            continue;
        };
        let chair = if me == Player::Blue { 0 } else { 1 };
        let actions: Vec<(Move, Player)> = if let Some(seq) = d.get("seq").and_then(|s| s.as_array()) {
            decode_seq(seq, me)
        } else if let Some(ids) = d["moves"].as_array() {
            decode_site(ids)
        } else {
            continue;
        };

        // Replay to each of OUR double turns.
        let mut game = Game::new();
        for (mv, mover) in &actions {
            if game.is_over() {
                break;
            }
            let pos = game.position();
            let expected = pos.to_move();
            if *mover != expected {
                break; // recorded/replay desync — skip the rest of this game
            }
            // OUR double-turn start: to_move == us, 2 actions left.
            if expected == me && pos.actions_left_in_turn() == 2 {
                let (regret, mv2_matches_pv) = measure_turn(&pos, me);
                turns_total += 1;
                per_chair[chair].0 += 1;
                if !mv2_matches_pv {
                    mismatch += 1;
                    per_chair[chair].1 += 1;
                }
                if regret > REGRET_SMALL {
                    regret_gt_small += 1;
                    per_chair[chair].2 += 1;
                }
                if regret > REGRET_BIG {
                    regret_gt_big += 1;
                    per_chair[chair].3 += 1;
                }
                regrets.push(regret);
                per_chair[chair].4.push(regret);
            }
            if game.position().check_move(*mv).is_err() {
                break; // recorded move illegal under replay — stop here
            }
            game.play(*mv).expect("recorded move legal");
        }
    }

    let pct = |n: usize, d: usize| if d == 0 { 0.0 } else { n as f64 / d as f64 * 100.0 };
    let mean = |v: &[f64]| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
    let p90 = |v: &[f64]| {
        if v.is_empty() {
            0.0
        } else {
            let mut s = v.to_vec();
            s.sort_by(|a, b| b.total_cmp(a));
            let idx = ((s.len() as f64) * 0.90).floor() as usize;
            s.get(idx).copied().unwrap_or(0.0)
        }
    };
    let max = |v: &[f64]| v.iter().copied().fold(0.0f64, f64::max);

    println!("P1 pair-regret census (our double turns, {dir}):");
    println!(
        "double turns {} | re-plan != PV-assumed mv2: {} ({:.1}%)",
        turns_total,
        mismatch,
        pct(mismatch, turns_total)
    );
    println!(
        "regret joint-greedy > {REGRET_SMALL}: {} ({:.1}%) | > {REGRET_BIG}: {} ({:.1}%)",
        regret_gt_small,
        pct(regret_gt_small, turns_total),
        regret_gt_big,
        pct(regret_gt_big, turns_total)
    );
    println!(
        "regret size: mean {:.3} | p90 {:.3} | max {:.3}",
        mean(&regrets),
        p90(&regrets),
        max(&regrets)
    );
    for (chair, row) in per_chair.iter().enumerate() {
        let cname = if chair == 0 { "us=Blue" } else { "us=Red" };
        println!(
            "{cname}: turns {} | mismatch {} ({:.1}%) | regret > {REGRET_SMALL} {} ({:.1}%) | > {REGRET_BIG} {} | mean {:.3} | p90 {:.3} | max {:.3}",
            row.0,
            row.1,
            pct(row.1, row.0),
            row.2,
            pct(row.2, row.0),
            row.3,
            mean(&row.4),
            p90(&row.4),
            max(&row.4)
        );
    }
    println!(
        "VERDICT: {}",
        if regret_gt_big > 0 && pct(mismatch, turns_total) >= 25.0 {
            "D1 GO — regret big and frequent"
        } else if regret_gt_big > 0 {
            "D1 KILL — regret big but re-plan tracks the PV (not frequent)"
        } else {
            "D1 KILL — regret rare/small (kill-0: regret rare/small)"
        }
    );
}

/// Decode a local sparring seq: [{"who","color","moves":[{from,to}]}...].
/// JS frame (0..18) -> Rust frame: subtract 9 from both axes.
/// Entries with an explicit "color" use it; without one, "bot" is the file's
/// own bot (`me`) and "chall" the opponent (R-chall-* files carry no color).
fn decode_seq(seq: &[serde_json::Value], me: Player) -> Vec<(Move, Player)> {
    let mut out = Vec::new();
    for entry in seq {
        let who = entry.get("who").and_then(|w| w.as_str()).unwrap_or("");
        let explicit = entry.get("color").and_then(|c| c.as_str());
        let player = match explicit {
            Some(c) if c.contains("blue") => Player::Blue,
            Some(c) if c.contains("red") => Player::Red,
            _ => {
                if who.contains("bot") {
                    me
                } else {
                    me.opponent()
                }
            }
        };
        if let Some(moves) = entry.get("moves").and_then(|m| m.as_array()) {
            for m in moves {
                let from = m.get("from").and_then(|p| p.as_array());
                let to = m.get("to").and_then(|p| p.as_array());
                if let (Some(from), Some(to)) = (from, to) {
                    if let (Some(fx), Some(fy), Some(tx), Some(ty)) = (
                        from.first().and_then(|v| v.as_f64()),
                        from.get(1).and_then(|v| v.as_f64()),
                        to.first().and_then(|v| v.as_f64()),
                        to.get(1).and_then(|v| v.as_f64()),
                    ) {
                        if let (Some(a), Some(b)) = (
                            Point::new(fx as i8 - 9, fy as i8 - 9),
                            Point::new(tx as i8 - 9, ty as i8 - 9),
                        ) {
                            if let Some(mv) = Move::between(a, b) {
                                out.push((mv, player));
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Decode a site-format game (flat move ids): Red iff flat index in {1,2} mod 4
/// (side_to_move: total 0 -> Blue, then pairs Red/Blue starting at index 1).
fn decode_site(ids: &[serde_json::Value]) -> Vec<(Move, Player)> {
    let mut out = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        let Some(idx) = id.as_u64() else { continue };
        let Some(mv) = Move::from_index(idx as usize) else { continue };
        let player = if i == 0 || (i > 0 && ((i - 1) / 2) % 2 == 1) {
            Player::Blue
        } else {
            Player::Red
        };
        out.push((mv, player));
    }
    out
}

// ==== FROZEN verbatim copies from retaliator/src/search.rs @ 2a41473 ====
// (bot behaviour unchanged; needed because `ranked` and friends are private).

#[allow(dead_code)]
fn horizon_extension(before: &Position, end: &Position, mover: Player) -> f64 {
    let gained = (end.area(mover).to_f64() - before.area(mover).to_f64()).max(0.0);
    let destroyed =
        (before.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    (gained + destroyed) * (events - events.min(HORIZON)) * HORIZON_WEIGHT
}

#[allow(dead_code)]
fn max_pop(pos: &Position, victim: Player) -> f64 {
    let held = pos.area(victim).to_f64();
    let mut worst: f64 = 0.0;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        let _ = next.apply_unchecked(mv);
        worst = worst.max(held - next.area(victim).to_f64());
    }
    worst.max(0.0)
}

#[allow(dead_code)]
fn value(position: &Position) -> f64 {
    let drawn = matches!(position.outcome_given(&position.legal_moves()), Some(Outcome::Draw(_)));
    if drawn { 0.0 } else { evaluate(position) }
}

#[allow(dead_code)]
struct Ranked {
    mv: Move,
    after: Position,
    value: f64,
    priority: f64,
}

#[allow(dead_code)]
fn ranked(
    position: &Position,
    budget: usize,
    first: bool,
    avoid: &[Point],
) -> Vec<(Move, Position, f64, f64)> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let room_before = room(position, mover);
    let heat = fight_heat(position, mover, opp);
    let mut moves: Vec<Move> =
        position.legal_moves().iter().filter(|&mv| !repeats_a_connection(position, mv)).collect();
    moves.sort_by_key(|mv| (Reverse(length(*mv)), mv.index()));

    let mut tried: Vec<Ranked> = moves
        .into_iter()
        .take(budget)
        .map(|mv| {
            let mut after = position.clone();
            let outcome = after.apply_unchecked(mv);
            let value = value(&after);
            let mut priority = sign(mover) * value;
            if first && after.to_move() == mover && !after.is_finished() {
                priority += loop_bonus(position, mv, &after, room_before);
            }
            priority += horizon_extension(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
            let hz = f64::from(position.scoring_events_left()).min(HORIZON);
            if first {
                if heat >= FIGHT_HEAT
                    && outcome.broken.is_none()
                    && !near_enemy_node(position, opp, target, REMOTE_DIST)
                {
                    priority += REMOTE_BONUS * hz;
                }
                if outcome.broken.is_none() && near_fresh_enemy(position, target) {
                    priority -= FRESH_PENALTY * hz;
                }
                if near_own_count(position, mover, target) >= DENSE_COUNT {
                    priority += DENSE_BONUS * hz;
                }
                if outcome.broken.is_none()
                    && own_gain <= 0.0
                    && room(&after, mover) - room_before < IDLE_ROOM
                    && !near_enemy_node(position, opp, target, IDLE_ENEMY_DIST)
                {
                    priority -= IDLE_PENALTY * hz;
                }
                if outcome.broken.is_none() && near_points(avoid.iter().copied(), target, CUT_RADIUS) {
                    priority -= REBUILD_PENALTY * hz;
                }
                if outcome.kind == MoveKind::Connect
                    && own_gain < PATIENCE_MAX_GAIN
                    && position.actions_played() < PATIENCE_WINDOW
                {
                    priority -= PATIENCE_PENALTY * hz;
                }
                if outcome.broken.is_none() && near_enemy_node(position, opp, target, 1) {
                    priority -= CONTACT_PENALTY * hz;
                }
                if outcome.kind == MoveKind::Connect
                    && own_gain <= 0.0
                    && !near_enemy_node(position, opp, target, DEADWOOD_ENEMY_DIST)
                {
                    priority -= DEADWOOD_PENALTY * hz;
                }
            }
            Ranked { mv, after, value, priority }
        })
        .collect();
    tried.sort_by(|a, b| b.priority.total_cmp(&a.priority).then(a.mv.index().cmp(&b.mv.index())));
    tried.into_iter().map(|r| (r.mv, r.after, r.value, r.priority)).collect()
}

#[allow(dead_code)]
fn near_points(mut points: impl Iterator<Item = Point>, target: Point, dist: i8) -> bool {
    points.any(|p| (p.x() - target.x()).abs() <= dist && (p.y() - target.y()).abs() <= dist)
}

#[allow(dead_code)]
fn fight_heat(position: &Position, mover: Player, opp: Player) -> usize {
    let own: Vec<Point> = position.nodes(mover).iter().collect();
    let foe: Vec<Point> = position.nodes(opp).iter().collect();
    let mut heat = 0;
    for a in &own {
        for b in &foe {
            if (a.x() - b.x()).abs() <= FIGHT_DIST && (a.y() - b.y()).abs() <= FIGHT_DIST {
                heat += 1;
            }
        }
    }
    heat
}

#[allow(dead_code)]
fn near_fresh_enemy(position: &Position, target: Point) -> bool {
    position.shielded_edges().any(|edge| {
        near_points([edge.origin(), edge.far()].into_iter(), target, FRESH_RADIUS)
    })
}

#[allow(dead_code)]
fn near_own_count(position: &Position, player: Player, target: Point) -> u32 {
    position
        .nodes(player)
        .iter()
        .filter(|node| {
            (node.x() - target.x()).abs() <= DENSE_DIST && (node.y() - target.y()).abs() <= DENSE_DIST
        })
        .count() as u32
}

#[allow(dead_code)]
fn near_enemy_node(position: &Position, player: Player, target: Point, dist: i8) -> bool {
    near_points(position.nodes(player).iter(), target, dist)
}

#[allow(dead_code)]
fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

#[allow(dead_code)]
fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
}

#[allow(dead_code)]
fn distance_to_center(mv: &Move) -> f64 {
    let target = match mv.target() {
        Some(t) => t,
        None => return f64::MAX,
    };
    let cx = 9.0f64;
    let cy = 9.0f64;
    let dx = (target.x() as f64 - cx).abs();
    let dy = (target.y() as f64 - cy).abs();
    dx.max(dy)
}

#[allow(dead_code)]
fn loop_bonus(position: &Position, mv: Move, after: &Position, room_before: f64) -> f64 {
    let mover = position.to_move();
    let target = mv.target().expect("legal moves end on the board");
    let mut triangle: f64 = 0.0;
    for edge in position.edges(mover).iter().filter(|edge| edge.has_endpoint(mv.source)) {
        let (origin, far) = edge.endpoints();
        let third = if origin == mv.source { far } else { origin };
        let closes = Move::between(target, third).is_some_and(|closing| after.check_move(closing).is_ok());
        if closes {
            triangle = triangle.max(f64::from(cross(mv.source, target, third).abs()) * 0.5);
        }
    }
    let added_room = (room(after, mover) - room_before).max(0.0).min(triangle);
    added_room * f64::from(after.scoring_events_left().min(HORIZON as u8))
}

#[allow(dead_code)]
fn room(position: &Position, player: Player) -> f64 {
    let mut points: Vec<Point> = position.nodes(player).iter().collect();
    points.sort_by_key(|point| (point.x(), point.y()));
    if points.len() < 3 {
        return 0.0;
    }
    let mut hull = half_hull(points.iter().copied());
    hull.extend(half_hull(points.iter().rev().copied()));
    let first = hull[0];
    hull.windows(2).map(|pair| cross(first, pair[0], pair[1]) as f64).sum::<f64>().abs() * 0.5
}

#[allow(dead_code)]
fn half_hull(points: impl Iterator<Item = Point>) -> Vec<Point> {
    let mut chain: Vec<Point> = Vec::new();
    for point in points {
        while chain.len() >= 2 && cross(chain[chain.len() - 2], chain[chain.len() - 1], point) <= 0 {
            chain.pop();
        }
        chain.push(point);
    }
    chain.pop();
    chain
}

#[allow(dead_code)]
fn cross(a: Point, b: Point, c: Point) -> i32 {
    i32::from(b.x() - a.x()) * i32::from(c.y() - a.y()) - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
}

#[allow(dead_code)]
fn sign(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}
