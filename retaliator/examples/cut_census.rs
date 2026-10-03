//! CUT-YIELD kill-0: flip census for ZERO_CUT_PENALTY=1.0 x hz on
//! broken + 0-destroyed + 0-gain first actions.

use retaliator::search::{CUT_MEMORY, evaluate};
use meridian_engine::{Game, Move, MoveKind, Outcome, Player, Point, Position, notation};
use serde_json::Value;
use std::cmp::Reverse;
use std::fs;

// Frozen verbatim copies from retaliator/src/search.rs @ base. Bot behavior unchanged.

#[allow(dead_code)]
const HORIZON: f64 = 12.0;

#[allow(dead_code)]
const HORIZON_WEIGHT: f64 = 0.5;

#[allow(dead_code)]
const CONTACT_PENALTY: f64 = 1.0;

#[allow(dead_code)]
const DEADWOOD_PENALTY: f64 = 2.0;

#[allow(dead_code)]
const DEADWOOD_ENEMY_DIST: i8 = 3;

#[allow(dead_code)]
const CUT_RADIUS: i8 = 3;

#[allow(dead_code)]
const REBUILD_PENALTY: f64 = 3.0;

#[allow(dead_code)]
const FRESH_RADIUS: i8 = 2;

#[allow(dead_code)]
const FRESH_PENALTY: f64 = 1.5;

#[allow(dead_code)]
const IDLE_ROOM: f64 = 0.5;

#[allow(dead_code)]
const IDLE_ENEMY_DIST: i8 = 3;

#[allow(dead_code)]
const IDLE_PENALTY: f64 = 2.0;

#[allow(dead_code)]
const FIGHT_DIST: i8 = 4;

#[allow(dead_code)]
const FIGHT_HEAT: usize = 10;

#[allow(dead_code)]
const REMOTE_DIST: i8 = 5;

#[allow(dead_code)]
const REMOTE_BONUS: f64 = 1.0;

#[allow(dead_code)]
const DENSE_DIST: i8 = 1;

#[allow(dead_code)]
const DENSE_COUNT: u32 = 2;

#[allow(dead_code)]
const DENSE_BONUS: f64 = 1.0;

#[allow(dead_code)]
const DOOM_W: f64 = 1.0;

#[allow(dead_code)]
const PATIENCE_MAX_GAIN: f64 = 2.0;

#[allow(dead_code)]
const PATIENCE_WINDOW: u8 = 12;

#[allow(dead_code)]
const PATIENCE_PENALTY: f64 = 2.0;

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
            // Fix 1: full-horizon claims and denial, in ranking as well as selection.
            priority += horizon_extension(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
            // All priorities run in full-horizon points (area x events), so
            // fixed penalties must scale the same way or they never fire.
            let hz = f64::from(position.scoring_events_left()).min(HORIZON);
            if first {
                // Two-front play: hot position + far target = safe banking.
                if heat >= FIGHT_HEAT
                    && outcome.broken.is_none()
                    && !near_enemy_node(position, opp, target, REMOTE_DIST)
                {
                    priority += REMOTE_BONUS * hz;
                }
                // Fresh/shielded walls: don't fight what can't be cut yet.
                if outcome.broken.is_none()
                    && near_fresh_enemy(position, target)
                {
                    priority -= FRESH_PENALTY * hz;
                }
                // Thicket density: land next to 2+ own nodes.
                if near_own_count(position, mover, target) >= DENSE_COUNT {
                    priority += DENSE_BONUS * hz;
                }
                // Do-nothing filter.
                if outcome.broken.is_none()
                    && own_gain <= 0.0
                    && room(&after, mover) - room_before < IDLE_ROOM
                    && !near_enemy_node(position, opp, target, IDLE_ENEMY_DIST)
                {
                    priority -= IDLE_PENALTY * hz;
                }
                // Anti-rebuild routing: our loops were cut near these points
                // recently; re-closing there gets farmed. Counter-cutting
                // (breaking something) is exempt. (Rival-analysis steal #1.)
                if outcome.broken.is_none() && near_points(avoid.iter().copied(), target, CUT_RADIUS) {
                    priority -= REBUILD_PENALTY * hz;
                }
                // Patience: don't snatch tiny loops in the opening while the
                // board is wide open; set up bigger closes instead (GB delays
                // its first close to action ~13 and banks ~10/loop).
                if outcome.kind == MoveKind::Connect
                    && own_gain < PATIENCE_MAX_GAIN
                    && position.actions_played() < PATIENCE_WINDOW
                {
                    priority -= PATIENCE_PENALTY * hz;
                }
                // Fix 2: don't graze the enemy frontier for free.
                if outcome.broken.is_none()
                    && near_enemy_node(position, opp, target, 1)
                {
                    priority -= CONTACT_PENALTY * hz;
                }
                // Fix 3: don't reinforce the back; expand instead. A Connect
                // that adds no area far from the enemy burns tempo: the safe
                // loop already banks, and redundancy never survives a cut.
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
        next.apply_unchecked(mv);
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
            (node.x() - target.x()).abs() <= DENSE_DIST
                && (node.y() - target.y()).abs() <= DENSE_DIST
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

#[allow(dead_code)]
fn ranked_zcp(
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
            // Fix 1: full-horizon claims and denial, in ranking as well as selection.
            priority += horizon_extension(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
            // All priorities run in full-horizon points (area x events), so
            // fixed penalties must scale the same way or they never fire.
            let hz = f64::from(position.scoring_events_left()).min(HORIZON);
            if first {
                // Two-front play: hot position + far target = safe banking.
                if heat >= FIGHT_HEAT
                    && outcome.broken.is_none()
                    && !near_enemy_node(position, opp, target, REMOTE_DIST)
                {
                    priority += REMOTE_BONUS * hz;
                }
                // Fresh/shielded walls: don't fight what can't be cut yet.
                if outcome.broken.is_none()
                    && near_fresh_enemy(position, target)
                {
                    priority -= FRESH_PENALTY * hz;
                }
                // Thicket density: land next to 2+ own nodes.
                if near_own_count(position, mover, target) >= DENSE_COUNT {
                    priority += DENSE_BONUS * hz;
                }
                // Do-nothing filter.
                if outcome.broken.is_none()
                    && own_gain <= 0.0
                    && room(&after, mover) - room_before < IDLE_ROOM
                    && !near_enemy_node(position, opp, target, IDLE_ENEMY_DIST)
                {
                    priority -= IDLE_PENALTY * hz;
                }
                // Anti-rebuild routing: our loops were cut near these points
                // recently; re-closing there gets farmed. Counter-cutting
                // (breaking something) is exempt. (Rival-analysis steal #1.)
                if outcome.broken.is_none() && near_points(avoid.iter().copied(), target, CUT_RADIUS) {
                    priority -= REBUILD_PENALTY * hz;
                }
                // Patience: don't snatch tiny loops in the opening while the
                // board is wide open; set up bigger closes instead (GB delays
                // its first close to action ~13 and banks ~10/loop).
                if outcome.kind == MoveKind::Connect
                    && own_gain < PATIENCE_MAX_GAIN
                    && position.actions_played() < PATIENCE_WINDOW
                {
                    priority -= PATIENCE_PENALTY * hz;
                }
                // Fix 2: don't graze the enemy frontier for free.
                if outcome.broken.is_none()
                    && near_enemy_node(position, opp, target, 1)
                {
                    priority -= CONTACT_PENALTY * hz;
                }
                // Fix 3: don't reinforce the back; expand instead. A Connect
                // that adds no area far from the enemy burns tempo: the safe
                // loop already banks, and redundancy never survives a cut.
                if outcome.kind == MoveKind::Connect
                    && own_gain <= 0.0
                    && !near_enemy_node(position, opp, target, DEADWOOD_ENEMY_DIST)
                {
                    priority -= DEADWOOD_PENALTY * hz;
                }
                if outcome.broken.is_some()
                    && position.area(opp).to_f64() - after.area(opp).to_f64() <= 0.0
                    && own_gain <= 0.0
                {
                    priority -= 1.0 * hz;
                }
            }
            Ranked { mv, after, value, priority }
        })
        .collect();
    tried.sort_by(|a, b| b.priority.total_cmp(&a.priority).then(a.mv.index().cmp(&b.mv.index())));
    tried.into_iter().map(|r| (r.mv, r.after, r.value, r.priority)).collect()
}
fn js_pt(a: &[f64]) -> Point {
    Point::new((a[0].round() as i64 - 9) as i8, (9 - a[1].round() as i64) as i8).expect("on board")
}

fn main() {
    let m = Move::from_index(16058).expect("id");
    assert_eq!(notation::move_text(m.source, m.target().expect("t")), "D10-D13");
    let dir = std::env::args().nth(1).expect("corpus dir");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".moves.json"))
        .collect();
    files.sort();
    let mut zyc = 0u64;
    let mut flips = 0u64;
    let mut roots = 0u64;
    let mut chair = [(0u64, 0u64, 0u64); 2];
    for name in &files {
        let our_blue = name.contains("botIsblue");
        let ci = if our_blue { 0 } else { 1 };
        let ours = if our_blue { Player::Blue } else { Player::Red };
        let opp = ours.opponent();
        let j: Value = serde_json::from_str(&fs::read_to_string(format!("{dir}/{name}")).unwrap()).unwrap();
        let mut game = Game::new();
        let mut cuts: Vec<(usize, Player, Point, Point)> = Vec::new();
        let mut idx = 0usize;
        for t in j["seq"].as_array().cloned().unwrap_or_default() {
            let Some(mvs) = t.get("moves").and_then(|v| v.as_array()) else { continue };
            for m in mvs {
                let from = js_pt(&m["from"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>());
                let to = js_pt(&m["to"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>());
                let mv = Move::between(from, to).expect("king step");
                let mover = game.position().to_move();
                if mover == ours && !game.position().is_finished() {
                    let root = game.position().clone();
                    let mut avoid: Vec<Point> = Vec::new();
                    for (cix, cmover, o, f) in cuts.iter().rev() {
                        if idx - cix > CUT_MEMORY {
                            break;
                        }
                        if cmover.opponent() == ours {
                            avoid.push(*o);
                            avoid.push(*f);
                        }
                    }
                    let legal = root.legal_moves().len();
                    if legal > 0 {
                        roots += 1;
                        let r = ranked(&root, legal, true, &avoid);
                        let rz = ranked_zcp(&root, legal, true, &avoid);
                        if !r.is_empty() && !rz.is_empty() {
                            let mut plain_zyc = false;
                            let (pmv, pafter, _, _) = &r[0];
                            let mut probe = root.clone();
                            let oc = probe.apply_unchecked(*pmv);
                            let dest = root.area(opp).to_f64() - pafter.area(opp).to_f64();
                            let gain = pafter.area(ours).to_f64() - root.area(ours).to_f64();
                            if oc.broken.is_some() && dest <= 0.0 && gain <= 0.0 {
                                plain_zyc = true;
                                zyc += 1;
                                chair[ci].0 += 1;
                            }
                            if plain_zyc && rz[0].0 != *pmv {
                                flips += 1;
                                chair[ci].1 += 1;
                            }
                            chair[ci].2 += 1;
                        }
                    }
                }
                let oc = game.play(mv).expect("legal replay");
                if let Some(cut) = oc.broken {
                    cuts.push((idx, mover, cut.origin(), cut.far()));
                }
                idx += 1;
            }
        }
        println!("{name}: done");
    }
    for (ci, ch) in ["Blue", "Red"].iter().enumerate() {
        println!(
            "CHAIR {ch}: zero-yield-cut picks {} flips {} ({:.1}%) over {} roots",
            chair[ci].0,
            chair[ci].1,
            100.0 * chair[ci].1 as f64 / chair[ci].0.max(1) as f64,
            chair[ci].2
        );
    }
    println!("FLIP {flips}/{zyc} = {:.1}% over {roots} roots", 100.0 * flips as f64 / zyc.max(1) as f64);
    let _ = MoveKind::Connect;
    let _ = evaluate(&Game::new().position().clone());
}
