//! probe_poprice.rs — POP-PRICE decision-node ledger on 3a414aad (Actions 56, 57, 64, 80, 84, 88, 89)

use std::cmp::Reverse;
use std::fs;
use meridian_engine::{Game, Move, MoveKind, Outcome, Player, Point, Position};

// Constants exactly matching retaliator/src/search.rs
pub const MOVE_BUDGET: usize = 4096;
const WIDTH: usize = 8;
const HORIZON: f64 = 12.0;
const ROOM_WEIGHT: f64 = 0.4;
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
const DOOM_W: f64 = 1.0;
const CAPTURE_W: f64 = 3.0;
const PATIENCE_MAX_GAIN: f64 = 2.0;
const PATIENCE_WINDOW: u8 = 12;
const PATIENCE_PENALTY: f64 = 2.0;

fn sign(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}

fn cross(a: Point, b: Point, c: Point) -> i32 {
    i32::from(b.x() - a.x()) * i32::from(c.y() - a.y()) - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
}

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

fn one_edge_nodes(position: &Position, player: Player) -> u32 {
    use std::collections::HashMap;
    let mut degree: HashMap<usize, u32> = HashMap::new();
    for edge in position.edges(player).iter() {
        for end in [edge.origin(), edge.far()] {
            *degree.entry(end.index()).or_insert(0) += 1;
        }
    }
    position.nodes(player).iter().filter(|n| degree.get(&n.index()) == Some(&1)).count() as u32
}

pub fn evaluate(position: &Position) -> f64 {
    let events_left = f64::from(position.scoring_events_left());
    let worth = |player| {
        position.score(player).to_f64()
            + position.area(player).to_f64() * events_left.min(HORIZON)
            + room(position, player) * ROOM_WEIGHT * events_left.min(1.0)
    };
    worth(Player::Blue) - worth(Player::Red)
        + CAPTURE_W
            * (one_edge_nodes(position, Player::Red) as f64
                - one_edge_nodes(position, Player::Blue) as f64)
}

fn value(position: &Position) -> f64 {
    let drawn = matches!(position.outcome_given(&position.legal_moves()), Some(Outcome::Draw(_)));
    if drawn { 0.0 } else { evaluate(position) }
}

fn horizon_extension(before: &Position, end: &Position, mover: Player) -> f64 {
    let gained = (end.area(mover).to_f64() - before.area(mover).to_f64()).max(0.0);
    let destroyed =
        (before.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    (gained + destroyed) * (events - events.min(HORIZON)) * HORIZON_WEIGHT
}

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

fn near_points(mut points: impl Iterator<Item = Point>, target: Point, dist: i8) -> bool {
    points.any(|p| (p.x() - target.x()).abs() <= dist && (p.y() - target.y()).abs() <= dist)
}

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

fn near_fresh_enemy(position: &Position, target: Point) -> bool {
    position.shielded_edges().any(|edge| {
        near_points([edge.origin(), edge.far()].into_iter(), target, FRESH_RADIUS)
    })
}

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

fn near_enemy_node(position: &Position, player: Player, target: Point, dist: i8) -> bool {
    near_points(position.nodes(player).iter(), target, dist)
}

fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
}

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

#[derive(Debug, Clone)]
struct MoveDetails {
    mv: Move,
    notation: String,
    priority_rank: usize,
    in_beam: bool,
    honest_eval: f64,
    horizon_tail: f64,
    doom_charge: f64,
    rebuild_penalty: f64,
    fresh_penalty: f64,
    total_adjusted: f64,
    area_gain: f64,
    is_close: bool,
    e_events: u8,
}

fn evaluate_candidate(
    position: &Position,
    mv: Move,
    rank: usize,
    in_beam: bool,
    avoid: &[Point],
    reply_budget: usize,
) -> MoveDetails {
    let mover = position.to_move();
    let opp = mover.opponent();
    let notation = format!("{}-{}", mv.source, mv.target().unwrap());

    let mut after = position.clone();
    let oc = after.apply_unchecked(mv);
    let area_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
    let is_close = area_gain > 0.001 || oc.kind == MoveKind::Connect;

    // Search reply for honest 2-ply evaluation
    let replies = ranked_moves(&after, reply_budget, false, &[]);
    let (honest_eval, end_pos) = if let Some((reply_mv, _, reply_val, _)) = replies.first() {
        let mut end = after.clone();
        end.apply_unchecked(*reply_mv);
        (*reply_val, end)
    } else {
        (value(&after), after.clone())
    };

    let horizon_tail = horizon_extension(position, &end_pos, mover);

    let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
    let target = mv.target().expect("legal moves end on the board");
    let mut rebuild_pen = 0.0;
    let mut fresh_pen = 0.0;
    if oc.broken.is_none() {
        if near_points(avoid.iter().copied(), target, CUT_RADIUS) {
            rebuild_pen = REBUILD_PENALTY * hz_sel;
        }
        if near_fresh_enemy(position, target) {
            fresh_pen = FRESH_PENALTY * hz_sel;
        }
    }

    let doom_at = if end_pos.to_move() == opp {
        Some(&end_pos)
    } else if after.to_move() == opp {
        Some(&after)
    } else {
        None
    };

    let doom_charge = if let Some(pos) = doom_at {
        let hz_doom = f64::from(pos.scoring_events_left()).min(HORIZON);
        DOOM_W * max_pop(pos, mover) * hz_doom
    } else {
        0.0
    };

    let total_adjusted = sign(mover) * honest_eval + horizon_tail - rebuild_pen - fresh_pen - doom_charge;

    MoveDetails {
        mv,
        notation,
        priority_rank: rank,
        in_beam,
        honest_eval,
        horizon_tail,
        doom_charge,
        rebuild_penalty: rebuild_pen,
        fresh_penalty: fresh_pen,
        total_adjusted,
        area_gain,
        is_close,
        e_events: position.scoring_events_left(),
    }
}

fn ranked_moves(
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

    struct Ranked {
        mv: Move,
        after: Position,
        value: f64,
        priority: f64,
    }

    let hz = f64::from(position.scoring_events_left()).min(HORIZON);

    let mut tried: Vec<Ranked> = moves
        .into_iter()
        .take(budget)
        .map(|mv| {
            let mut after = position.clone();
            let outcome = after.apply_unchecked(mv);
            let val = value(&after);
            let mut priority = sign(mover) * val;
            if first && after.to_move() == mover && !after.is_finished() {
                priority += loop_bonus(position, mv, &after, room_before);
            }
            priority += horizon_extension(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();

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
            Ranked { mv, after, value: val, priority }
        })
        .collect();

    tried.sort_by(|a, b| b.priority.total_cmp(&a.priority).then(a.mv.index().cmp(&b.mv.index())));
    tried.into_iter().map(|r| (r.mv, r.after, r.value, r.priority)).collect()
}

fn main() {
    let path = "research/games/3a414aad-5aeb-4c16-87c1-f89e87966b93.json";
    let s = fs::read_to_string(path).expect("read game json");
    let d: serde_json::Value = serde_json::from_str(&s).expect("parse json");
    let move_indices: Vec<usize> = d["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();

    let target_actions = vec![56, 57, 64, 80, 84, 88, 89];

    let mut game = Game::new();
    let mut avoid: Vec<Point> = Vec::new();

    println!("==========================================================================================");
    println!("POP-PRICE DECISION-NODE LEDGER: Game 3a414aad (7 re-close nodes)");
    println!("==========================================================================================");

    let mut flip_count = 0;

    for (i, &idx) in move_indices.iter().enumerate() {
        let action = i + 1;
        let pos = game.position().clone();
        let mv = Move::from_index(idx).expect("valid move");

        if target_actions.contains(&action) {
            println!("\n------------------------------------------------------------------------------------------");
            println!("NODE: Action {} (Blue to move, E = {} scoring events left)", action, pos.scoring_events_left());
            println!("------------------------------------------------------------------------------------------");

            let first_actions = ranked_moves(&pos, MOVE_BUDGET / 2, true, &avoid);
            let width = first_actions.len().min(WIDTH);
            let reply_budget = (MOVE_BUDGET - first_actions.len()) / width.max(1);

            let chosen_mv = mv;
            let chosen_notation = format!("{}-{}", chosen_mv.source, chosen_mv.target().unwrap());

            // Rank of chosen move
            let chosen_rank = first_actions.iter().position(|(m, _, _, _)| *m == chosen_mv).map(|p| p + 1).unwrap_or(999);
            let chosen_in_beam = chosen_rank <= WIDTH;

            let chosen_details = evaluate_candidate(&pos, chosen_mv, chosen_rank, chosen_in_beam, &avoid, reply_budget);

            // Find best non-close alternative
            let mut best_non_close: Option<MoveDetails> = None;
            for (rank_0, (cand_mv, _, _, _)) in first_actions.iter().enumerate() {
                let r = rank_0 + 1;
                let in_beam = r <= WIDTH;
                let mut after_probe = pos.clone();
                let oc = after_probe.apply_unchecked(*cand_mv);
                let gain = after_probe.area(Player::Blue).to_f64() - pos.area(Player::Blue).to_f64();
                let is_close = gain > 0.001 || oc.kind == MoveKind::Connect;

                if !is_close {
                    let details = evaluate_candidate(&pos, *cand_mv, r, in_beam, &avoid, reply_budget);
                    if best_non_close.as_ref().map_or(true, |best| details.total_adjusted > best.total_adjusted) {
                        best_non_close = Some(details);
                    }
                }
            }

            let bnc = best_non_close.expect("found at least one non-close alternative");

            let gap = chosen_details.total_adjusted - bnc.total_adjusted;
            let eval_gap = chosen_details.honest_eval - bnc.honest_eval;
            let tail_gap = chosen_details.horizon_tail - bnc.horizon_tail;
            let doom_gap = bnc.doom_charge - chosen_details.doom_charge; // positive if chosen has more doom (subtracted)
            let rebuild_gap = (bnc.rebuild_penalty + bnc.fresh_penalty) - (chosen_details.rebuild_penalty + chosen_details.fresh_penalty);

            println!("  [CHOSEN RE-CLOSE]: {} (rank={}, in_beam={}) gain=+{:.1}",
                chosen_details.notation, chosen_details.priority_rank, chosen_details.in_beam, chosen_details.area_gain);
            println!("    honest_eval={:.2}, tail={:.2}, doom_charge={:.2}, rebuild_pen={:.2}, fresh_pen={:.2} => adjusted={:.2}",
                chosen_details.honest_eval, chosen_details.horizon_tail, chosen_details.doom_charge,
                chosen_details.rebuild_penalty, chosen_details.fresh_penalty, chosen_details.total_adjusted);

            println!("  [BEST NON-CLOSE ]: {} (rank={}, in_beam={}) gain=+{:.1}",
                bnc.notation, bnc.priority_rank, bnc.in_beam, bnc.area_gain);
            println!("    honest_eval={:.2}, tail={:.2}, doom_charge={:.2}, rebuild_pen={:.2}, fresh_pen={:.2} => adjusted={:.2}",
                bnc.honest_eval, bnc.horizon_tail, bnc.doom_charge,
                bnc.rebuild_penalty, bnc.fresh_penalty, bnc.total_adjusted);

            println!("  GAP (chosen - non_close): {:+.2}", gap);
            println!("    Decomposition: eval_gap={:+.2}, tail_gap={:+.2}, doom_gap(net)={:+.2}, rebuild_pen_gap(net)={:+.2}",
                eval_gap, tail_gap, doom_gap, rebuild_gap);

            // D1 Analysis:
            // Under D1, the re-close is priced for the 100% pop on the next enemy turn.
            // Pop wipes out the gained area immediately, so the horizon_tail is 0.0 (phantom tail deleted),
            // and the doom charge fully accounts for the area loss: D1_doom = DOOM_TAIL_W * pop * hz.
            // With gained area lost, D1 adjusted for re-close removes the tail_gap and adds the pop doom.
            let d1_reclose_adjusted = chosen_details.total_adjusted - chosen_details.horizon_tail - chosen_details.area_gain * f64::from(pos.scoring_events_left()).min(HORIZON) * HORIZON_WEIGHT;
            let d1_flips = d1_reclose_adjusted < bnc.total_adjusted;

            println!("  D1 PRICING CHECK: d1_reclose_adj={:.2} vs non_close={:.2} => FLIPS? {}",
                d1_reclose_adjusted, bnc.total_adjusted, if d1_flips { "YES (Flips to non-close)" } else { "NO (Re-close still wins)" });

            if d1_flips {
                flip_count += 1;
            }
        }

        let outcome = game.play(mv).expect("legal move");
        if let Some(cut) = outcome.broken {
            if pos.to_move().opponent() == Player::Blue {
                avoid.push(cut.origin());
                avoid.push(cut.far());
            }
        }
        if avoid.len() > 12 {
            avoid.drain(0..avoid.len() - 12);
        }
    }

    println!("\n==========================================================================================");
    println!("POP-PRICE SUMMARY & KILL-0 VERDICT");
    println!("==========================================================================================");
    println!("Total nodes analyzed: 7");
    println!("Nodes where D1 flips re-close to non-close: {} / 7", flip_count);
    let kill_0 = flip_count < 3;
    println!("Kill-0 condition (flips < 3/7): {}", if kill_0 { "TRIGGERED (KILL-0)" } else { "PASSED (SURVIVES)" });
}
