//! Retaliator's strategy: Scout-shaped two-action search with three valuation fixes.
//!
//! The search skeleton (ranked first actions, searched replies, horizon-capped
//! static evaluation) follows the site's own Scout bot. The fixes come from our
//! self-play evidence:
//!
//! 1. **Break-first retaliation.** The static evaluation counts a popped enemy
//!    loop only up to `HORIZON` scoring events, so early-game breaks are
//!    underpriced by up to 3x. Both the candidate ranking and the final pick
//!    add the uncapped remainder (`destroyed * (events - capped) * weight`).
//!    A pure break-first 1-ply bot went 6-0 against our old greedy valuation,
//!    so the weight is deliberately large.
//! 2. **Contact avoidance.** A first action that ends next to an enemy node
//!    without cutting anything just donates a target (the neck/aggro family
//!    lives on this). Penalized.
//! 3. **Dead-wood discount.** A reinforcement between two own nodes that adds
//!    no area, far from the enemy, is tempo burned in the back: the safe loop
//!    already banks every turn, and one cut zeroes a loop no matter how
//!    reinforced it looks. Penalized, so the bot expands instead.

use std::cmp::Reverse;

use meridian_engine::{Move, MoveKind, Outcome, Player, Point, Position};

/// Positions searched for a move in a game. Analysis asks for its own number, up to this.
pub const MOVE_BUDGET: usize = 4096;
const SMALLEST_BUDGET: usize = 16;
/// How many first actions get their replies searched.
const WIDTH: usize = 8;
/// Enclosed area counts as if held for at most this many more scoring events.
const HORIZON: f64 = 12.0;
/// What the room a player's nodes span is worth, per unit of area, against area enclosed.
const ROOM_WEIGHT: f64 = 0.4;
/// How much of the beyond-horizon denial (popped enemy area times the scoring
/// events past the horizon) counts, in both ranking and final selection.
const RETALIATION_WEIGHT: f64 = 0.5;
/// First-action penalty for ending next to an enemy node without cutting anything.
const CONTACT_PENALTY: f64 = 1.0;
/// Penalty for a no-area reinforcement between own nodes far from the enemy.
const DEADWOOD_PENALTY: f64 = 2.0;
/// Beyond this Chebyshev distance to the nearest enemy node, a no-area
/// reinforcement counts as dead wood in the back.
const DEADWOOD_ENEMY_DIST: i8 = 3;

pub struct Analysis {
    /// Blue's lead after the best move and its reply.
    pub evaluation: f64,
    /// 2 when replies were searched.
    pub depth: usize,
    /// Positions searched.
    pub nodes: usize,
    /// The best moves first.
    pub candidates: Vec<Candidate>,
}

pub struct Candidate {
    pub mv: Move,
    /// Blue's lead after this move and the best reply (honest static value;
    /// ordering uses the retaliation-adjusted score, kept separately).
    pub evaluation: f64,
    /// This move and the best reply.
    pub pv: Vec<Move>,
    /// 1, plus the replies searched.
    pub visits: usize,
}

pub fn best_move(position: &Position) -> Option<Move> {
    analyze(position, MOVE_BUDGET).candidates.first().map(|candidate| candidate.mv)
}

pub fn analyze(position: &Position, budget: usize) -> Analysis {
    let mover = position.to_move();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true);
    let mut nodes = first_actions.len();
    let mut depth = usize::from(nodes > 0);
    let width = first_actions.len().min(WIDTH);
    let reply_budget = (budget - nodes) / width.max(1);

    // (adjusted score for ordering, candidate with honest static evaluation).
    let mut scored: Vec<(f64, Candidate)> = Vec::new();
    for (mv, after, _, _) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false);
        nodes += replies.len();
        let mut pv = vec![mv];
        let mut evaluation = value(&after);
        if let Some((reply, _, reply_value, _)) = replies.first() {
            evaluation = *reply_value;
            pv.push(*reply);
            depth = 2;
        }
        let end = position_after_pv(position, &pv);
        let adjusted = sign(mover) * evaluation + retaliation_remainder(position, &end, mover);
        let visits = 1 + replies.len();
        scored.push((adjusted, Candidate { mv, evaluation, pv, visits }));
    }
    scored.sort_by(|a, b| {
        b.0.total_cmp(&a.0).then(a.1.mv.index().cmp(&b.1.mv.index()))
    });
    let candidates = scored.into_iter().map(|(_, c)| c).collect::<Vec<_>>();
    let evaluation = candidates.first().map_or_else(|| value(position), |best| best.evaluation);
    Analysis { evaluation, depth, nodes, candidates }
}

/// Replays a principal variation from `position`.
fn position_after_pv(position: &Position, pv: &[Move]) -> Position {
    let mut pos = position.clone();
    for &mv in pv {
        pos.apply_unchecked(mv);
    }
    pos
}

/// The beyond-horizon part of a denial, mover-relative: enemy area destroyed
/// along the line, times the scoring events past the horizon. The static
/// evaluation already counts destruction up to the horizon; this is the rest.
fn retaliation_remainder(before: &Position, end: &Position, mover: Player) -> f64 {
    let destroyed =
        (before.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    destroyed * (events - events.min(HORIZON)) * RETALIATION_WEIGHT
}

struct Ranked {
    mv: Move,
    after: Position,
    value: f64,
    priority: f64,
}

/// Blue's lead as Scout sees it. Positive favours Blue.
pub fn evaluate(position: &Position) -> f64 {
    let events_left = f64::from(position.scoring_events_left());
    let worth = |player| {
        position.score(player).to_f64()
            + position.area(player).to_f64() * events_left.min(HORIZON)
            + room(position, player) * ROOM_WEIGHT * events_left.min(1.0)
    };
    worth(Player::Blue) - worth(Player::Red)
}

/// [`evaluate`], except that a drawn game is worth 0.
fn value(position: &Position) -> f64 {
    let drawn = matches!(position.outcome_given(&position.legal_moves()), Some(Outcome::Draw(_)));
    if drawn { 0.0 } else { evaluate(position) }
}

/// The mover's actions, best first, with the position after each and its value. Only the first
/// `budget` of them, longest edges first, are tried.
fn ranked(position: &Position, budget: usize, first: bool) -> Vec<(Move, Position, f64, f64)> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let room_before = room(position, mover);
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
            // Fix 1: retaliation, in ranking as well as selection.
            priority += retaliation_remainder(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            if first && outcome.broken.is_none() {
                // Fix 2: don't graze the enemy frontier for free.
                if near_enemy_node(position, opp, target, 1) {
                    priority -= CONTACT_PENALTY;
                }
                // Fix 3: don't reinforce the back; expand instead. A Connect
                // that adds no area far from the enemy burns tempo: the safe
                // loop already banks, and redundancy never survives a cut.
                let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
                if outcome.kind == MoveKind::Connect
                    && own_gain <= 0.0
                    && !near_enemy_node(position, opp, target, DEADWOOD_ENEMY_DIST)
                {
                    priority -= DEADWOOD_PENALTY;
                }
            }
            Ranked { mv, after, value, priority }
        })
        .collect();
    tried.sort_by(|a, b| b.priority.total_cmp(&a.priority).then(a.mv.index().cmp(&b.mv.index())));
    tried.into_iter().map(|r| (r.mv, r.after, r.value, r.priority)).collect()
}

/// Whether `target` is within Chebyshev `dist` of any of `player`'s nodes.
fn near_enemy_node(position: &Position, player: Player, target: Point, dist: i8) -> bool {
    position.nodes(player).iter().any(|node| {
        (node.x() - target.x()).abs() <= dist && (node.y() - target.y()).abs() <= dist
    })
}

/// A connection between two of the mover's nodes is listed from both ends. This is the second.
fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
}

/// For a first action the second can close into a triangle: the room it adds, up to that
/// triangle's area, for the scoring events left. It keeps such plans ahead of long edges that
/// cannot be closed in time.
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

/// The area of the convex hull of a player's nodes: room to grow into.
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

/// One side of the convex hull of points sorted left to right (Andrew's monotone chain), without
/// its last point, which starts the other side.
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

/// Twice the signed area of the triangle `a`, `b`, `c`: positive when it turns counter-clockwise.
fn cross(a: Point, b: Point, c: Point) -> i32 {
    i32::from(b.x() - a.x()) * i32::from(c.y() - a.y()) - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
}

/// 1 for Blue and -1 for Red, to turn Blue's lead into the mover's.
fn sign(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}
