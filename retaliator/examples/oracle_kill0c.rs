//! oracle_kill0c: MIRROR-ORACLE kill-0 on the RECORDED mirror games.
//!
//! The v9-ideas #4 premise ("Blue @30 = 24-29 vs Red 37.3 in mirrors") was
//! measured on the recorded v7-vs-v6 mirrors. oracle_kill0b's scripted-Red
//! harness (mesh + prot_kill0 continuation) does NOT reproduce it — its Red
//! continuation is weaker than the recorded Red. This probe replays the
//! RECORDED mirror Red lines instead: 5 openers = the 5 recorded
//! "Riposte v7 vs Riposte v6" games, Red = its recorded moves where legal
//! (else mesh_prefix, else the g-seed continuation; fallbacks counted).
//!
//! Blue modes: 0 = CURRENT (WIDTH=8 + mesh_prefix, per-action re-plan),
//! 1 = BEAM64 (WIDTH=64, no prefixes, per-action re-plan).
//! Bar (spec): Blue @30 area >= 35.0. With 5 recorded openers the >= 8/9 bar
//! is reported as x/5 plus the 9-opener synthetic result from oracle_kill0b.
//!
//! Read-only: no src changes; recorded games from lane-v8-autopsy
//! (research/games-ref/, site format, ids decoded with Move::from_index).

use meridian_engine::{Game, Move, MoveKind, Outcome, Player, Point, Position};
use retaliator::search::{evaluate, mesh_prefix};
use std::cmp::Reverse;

// FROZEN constants (values verbatim from search.rs @ 2a41473).
const MOVE_BUDGET: usize = 4096;
const SMALLEST_BUDGET: usize = 16;
const WIDTH: usize = 8;
const WIDTH64: usize = 64;
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

const BAR_AREA: f64 = 35.0;
const MEASURE_AT: u8 = 30;

/// Red's recorded moves from a site-format game (Red's slots in order).
fn recorded_red(path: &str) -> Option<Vec<Move>> {
    let raw = std::fs::read_to_string(path).ok()?;
    let d: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let ids = d["moves"].as_array()?;
    let mut red_moves = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        // mover of flat index i: Red iff i in {1,2} mod 4 (side_to_move).
        if i > 0 && ((i - 1) / 2) % 2 == 0 {
            if let Some(mv) = Move::from_index(id.as_u64()? as usize) {
                red_moves.push(mv);
            }
        }
    }
    Some(red_moves)
}

/// Red's action: recorded move where legal, else mesh_prefix, else the
/// deterministic continuation (nearest-to-centre, seed `g`).
fn red_play(pos: &Position, history: &[Move], recorded: &[Move], slot: usize, g: usize) -> Option<Move> {
    if let Some(mv) = recorded.get(slot) {
        if pos.check_move(*mv).is_ok() {
            return Some(*mv);
        }
    }
    if let Some(pm) = mesh_prefix(pos, history) {
        return Some(pm);
    }
    let center = Point::new(9, 0).expect("centre");
    let mut legal: Vec<Move> = pos.legal_moves().iter().collect();
    legal.sort_by_key(|m| m.index());
    let base = pos.area(Player::Red).to_f64();
    let mut scored: Vec<(i8, f64, usize, Move)> = legal
        .into_iter()
        .map(|mv| {
            let mut after = pos.clone();
            after.apply_unchecked(mv);
            let tgt = mv.target().expect("target");
            (cheby(tgt, center), base - after.area(Player::Red).to_f64(), mv.index(), mv)
        })
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2)));
    let pref = g % scored.len().min(9).max(1);
    scored.get(pref).map(|(_, _, _, mv)| *mv)
}

fn cheby(a: Point, b: Point) -> i8 {
    (a.x() - b.x()).abs().max((a.y() - b.y()).abs())
}

/// One recorded-mirror game. `mode`: 0 = CURRENT Blue, 1 = BEAM64 (re-plan),
/// 2 = BEAM64 SOLVED line (the turn-start search's PV played out without
/// re-planning — what a fixed MESH_B_ORACLE table would play).
/// Returns (Blue area @30, Blue score @30, Red area @30, Red fallbacks, red slot).
fn play_one(mode: usize, recorded: &[Move], g: usize) -> (f64, f64, f64, u32, usize) {
    let mut game = Game::new();
    let mut slot = 0usize; // Red's action counter (into the recorded line)
    let mut fallbacks = 0u32;
    let mut pending_pv: Option<Vec<Move>> = None;
    while game.moves().len() < usize::from(MEASURE_AT) && !game.is_over() {
        let pos = game.position();
        let mover = pos.to_move();
        let history = game.moves().to_vec();
        let Some(mv) = (if mover == Player::Red {
            pending_pv = None;
            let r = red_play(pos, &history, recorded, slot, g);
            if r.is_some() {
                // count a fallback when the recorded move was NOT used
                let used_recorded = recorded.get(slot).is_some_and(|rm| pos.check_move(*rm).is_ok());
                if !used_recorded {
                    fallbacks += 1;
                }
                slot += 1;
            }
            r
        } else {
            match mode {
                0 => match mesh_prefix(pos, &history) {
                    Some(pm) => Some(pm),
                    None => analyze_with_width(pos, MOVE_BUDGET, WIDTH, &[]),
                },
                1 => analyze_with_width(pos, MOVE_BUDGET, WIDTH64, &[]),
                2 => match pending_pv.take() {
                    Some(pv) => pv
                        .get(1)
                        .copied()
                        .or_else(|| analyze_with_width(pos, MOVE_BUDGET, WIDTH64, &[])),
                    None => analyze_pv_width(pos, MOVE_BUDGET, WIDTH64)
                        .map(|pv| {
                            let first = pv.first().copied();
                            pending_pv = Some(pv);
                            first
                        })
                        .flatten(),
                },
                _ => analyze_with_width(pos, MOVE_BUDGET, WIDTH64, &[]),
            }
        }) else {
            break;
        };
        game.play(mv).expect("legal");
    }
    let pos = game.position();
    (
        pos.area(Player::Blue).to_f64(),
        pos.score(Player::Blue).to_f64(),
        pos.area(Player::Red).to_f64(),
        fallbacks,
        slot,
    )
}

/// FROZEN verbatim `analyze_with_avoid` @ 2a41473 with the reply width raised
/// to `width_param`; returns the top candidate's PV.
fn analyze_pv_width(position: &Position, budget: usize, width_param: usize) -> Option<Vec<Move>> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, &[]);
    let width = first_actions.len().min(width_param);
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
    scored.first().map(|(_, _, pv)| pv.clone())
}

/// FROZEN verbatim `analyze_with_avoid` @ 2a41473 with the reply width raised.
fn analyze_with_width(position: &Position, budget: usize, width_param: usize, avoid: &[Point]) -> Option<Move> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, avoid);
    let width = first_actions.len().min(width_param);
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
    scored.first().map(|(_, mv, _)| *mv)
}

fn position_after_pv(position: &Position, pv: &[Move]) -> Position {
    let mut pos = position.clone();
    for mv in pv {
        pos.apply_unchecked(*mv);
    }
    pos
}

fn main() {
    let mirrors = [
        "16840108-93ce-4c18-934d-3590e4cc693f",
        "21fc2945-7a56-4fe5-ba19-3604e7357a6f",
        "b4462808-e94a-4269-9ecf-80447d4972b1",
        "c20c750b-3c06-49af-8da8-6eb0e4a76dea",
        "c3c8cf5b-858d-40b4-a721-e7aca788d8bf",
    ];
    let dir = std::env::var("ORACLE_RED_DIR").unwrap_or_else(|_| "../research/games-ref".into());
    println!(
        "MIRROR-ORACLE kill-0 (recorded mirrors): {n} openers = the 5 recorded \
         v7-vs-v6 mirrors, Red = recorded moves (fallback mesh/g-continuation), \
         Blue @{} area >= {BAR_AREA}",
        MEASURE_AT,
        n = mirrors.len()
    );
    println!("============================================================");
    let names = [
        "CURRENT (w8 + mesh, re-plan)",
        "BEAM64 (w64, no prefixes, re-plan)",
        "BEAM64-SOLVED (w64 PV played out, no re-plan)",
    ];
    let mut pass_any = [0usize; 3];
    for mode in 0..3 {
        println!("\n-- {} --", names[mode]);
        let mut pass = 0usize;
        for (g, id) in mirrors.iter().enumerate() {
            let path = format!("{dir}/{id}.json");
            let Some(recorded) = recorded_red(&path) else {
                println!("opener {g} ({id}): no recorded Red moves, skipped");
                continue;
            };
            let (area, score, red_area, fb, slot) = play_one(mode, &recorded, g);
            let status = if area >= BAR_AREA { "PASS" } else { "fail" };
            if area >= BAR_AREA {
                pass += 1;
            }
            println!(
                "opener {g} ({}…): Blue @30 area = {area:6.1} | score = {score:8.1} | Red @30 = {red_area:6.1} | recorded {slot}/{} used, fb {fb} {status}",
                &id[..8],
                recorded.len()
            );
        }
        pass_any[mode] = pass;
        println!("=> {}: Blue @30 >= {BAR_AREA} in {pass}/{} recorded openers", names[mode], mirrors.len());
    }
    println!("\n============================================================");
    println!(
        "CURRENT {}/{} | BEAM64 {}/{} | BEAM64-SOLVED {}/{}",
        pass_any[0], mirrors.len(), pass_any[1], mirrors.len(), pass_any[2], mirrors.len()
    );
    if pass_any[1] * 9 >= 8 * mirrors.len() as usize {
        println!(
            "VERDICT: KILL-0 MET on the recorded mirrors — beam-64 Blue reaches \
             @30 >= {BAR_AREA} in {}/{}; dose spec may be written.",
            pass_any[1],
            mirrors.len()
        );
    } else {
        println!(
            "VERDICT: KILL — beam-64 Blue cannot reach @30 >= {BAR_AREA} on the \
             recorded mirrors ({}/{}); the offline-solution premise fails.",
            pass_any[1],
            mirrors.len()
        );
    }
}

// ==== FROZEN verbatim copies from retaliator/src/search.rs @ 2a41473 ====

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
