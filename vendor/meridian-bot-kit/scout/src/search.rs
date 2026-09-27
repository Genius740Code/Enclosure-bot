//! Scout's strategy: a short search over the next two actions.
//!
//! Scout values a position by the points each player has, plus the area each encloses now, counted
//! as if held for up to 12 more scoring events, plus a little for the room their nodes span.
//!
//! It tries up to half its budget of its own actions, longest edges first, and keeps the best 8.
//! For each of those it tries the replies with the rest of the budget and takes the best one: the
//! opponent's move if the turn ended, or Scout's own second action if not. The value after that
//! reply decides.

use std::cmp::Reverse;

use meridian_engine::{Move, Outcome, Player, Point, Position};

/// Positions searched for a move in a game. Analysis asks for its own number, up to this.
pub const MOVE_BUDGET: usize = 4096;
const SMALLEST_BUDGET: usize = 16;
/// How many first actions get their replies searched.
const WIDTH: usize = 8;
/// Enclosed area counts as if held for at most this many more scoring events.
const HORIZON: u8 = 12;
/// What the room a player's nodes span is worth, per unit of area, against area enclosed.
const ROOM_WEIGHT: f64 = 0.4;

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
    /// Blue's lead after this move and the best reply.
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
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true);
    let mut nodes = first_actions.len();
    let mut depth = usize::from(nodes > 0);
    let width = first_actions.len().min(WIDTH);
    let reply_budget = (budget - nodes) / width.max(1);

    let mut candidates = Vec::new();
    for (mv, after, mut evaluation) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false);
        nodes += replies.len();
        let mut pv = vec![mv];
        if let Some((reply, _, after_reply)) = replies.first() {
            evaluation = *after_reply;
            pv.push(*reply);
            depth = 2;
        }
        candidates.push(Candidate { mv, evaluation, pv, visits: 1 + replies.len() });
    }
    let side = sign(position.to_move());
    candidates
        .sort_by(|a, b| (side * b.evaluation).total_cmp(&(side * a.evaluation)).then(a.mv.index().cmp(&b.mv.index())));
    let evaluation = candidates.first().map_or_else(|| value(position), |best| best.evaluation);
    Analysis { evaluation, depth, nodes, candidates }
}

/// Blue's lead as Scout sees it. Positive favours Blue.
pub fn evaluate(position: &Position) -> f64 {
    let events_left = f64::from(position.scoring_events_left());
    let worth = |player| {
        position.score(player).to_f64()
            + position.area(player).to_f64() * events_left.min(f64::from(HORIZON))
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
fn ranked(position: &Position, budget: usize, plan_loops: bool) -> Vec<(Move, Position, f64)> {
    let mover = position.to_move();
    let room_before = room(position, mover);
    let mut moves: Vec<Move> =
        position.legal_moves().iter().filter(|&mv| !repeats_a_connection(position, mv)).collect();
    moves.sort_by_key(|mv| (Reverse(length(*mv)), mv.index()));

    let mut tried: Vec<(f64, (Move, Position, f64))> = moves
        .into_iter()
        .take(budget)
        .map(|mv| {
            let mut after = position.clone();
            after.apply_unchecked(mv);
            let value = value(&after);
            let mut priority = sign(mover) * value;
            if plan_loops && after.to_move() == mover && !after.is_finished() {
                priority += loop_bonus(position, mv, &after, room_before);
            }
            (priority, (mv, after, value))
        })
        .collect();
    tried.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.0.index().cmp(&b.1.0.index())));
    tried.into_iter().map(|(_, entry)| entry).collect()
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
    added_room * f64::from(after.scoring_events_left().min(HORIZON))
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
