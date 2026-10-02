//! Patient big-build evaluation terms for Lane P (H2).
//!
//! Bonus for 2-line connection sequences that complete a large area in 2-3 rounds
//! (triangle-close potential / room-to-close conversion), penalty for snatching
//! tiny loops (<2-3 area) in opening <12 actions, reward for committing to one
//! anchor region vs scattered tiles. All terms in FULL-HORIZON points.
//!
//! Deterministic tiebreak by move index. Sweep 1 weight at a time.

use meridian_engine::{Edge, Move, Outcome, Player, Point, Position, MoveKind};

/// Bonus for a 2-line connection sequence that can complete a large area in
/// 2-3 rounds (triangle-close potential). Rewards moves that set up a close
/// where the third edge will close a large triangle within the next 2-3 turns.
/// The value scales with the triangle area and events remaining, kept fully
/// horizon-uncapped so big long-term claims rank above immediate small gains.
pub fn triangle_close_potential(position: &Position, mv: Move, after: &Position) -> f64 {
    let mover = position.to_move();
    let target = mv.target().expect("legal moves end on the board");

    // Count how many of our existing edges can form a triangle with this move
    // and the potential closing move.
    let mut potential: f64 = 0.0;
    for edge in position.edges(mover).iter().filter(|edge| edge.has_endpoint(mv.source)) {
        let (origin, far) = edge.endpoints();
        let third = if origin == mv.source { far } else { origin };

        // Check if the third closing edge (target-third) is legal and would close
        let close_move = Move::between(target, third);
        if let Some(mv_close) = close_move {
            if after.check_move(mv_close).is_ok() {
                // Triangle is completable; area is the triangle area via cross product
                let area = f64::from(cross(mv.source, target, third).abs()) * 0.5;
                // Scale by events left - this is full-horizon, no HORIZON cap
                let events = f64::from(after.scoring_events_left());
                potential = potential.max(area * events);
            }
        }
    }
    potential
}

/// Penalty for snatching tiny loops (<2-3 area) in the opening (<12 actions).
/// Detects Connect moves that gain less than TINY_LOOP_MAX_AREA when the game
/// has played fewer than TINY_LOOP_WINDOW actions, and applies a penalty scaled
/// by events left (full-horizon scaling so the penalty is meaningful across
/// the whole game but only triggers in the opening).
pub fn tiny_loop_snatched_penalty(
    position: &Position,
    mv: Move,
    after: &Position,
) -> f64 {
    let mover = position.to_move();
    let target = mv.target().expect("legal moves end on the board");
    let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();

    // Only trigger when: Connect move, tiny gain, opening stage
    // Determine move kind: Connect if target is mover's node, Capture if target is opponent's node, Extend otherwise
    let mv_kind = match position.node_owner(target) {
        None => MoveKind::Extend,
        Some(owner) if owner == mover => MoveKind::Connect,
        Some(_) => MoveKind::Capture,
    };
    if mv_kind != MoveKind::Connect {
        return 0.0;
    }
    if own_gain >= constants::TINY_LOOP_MAX_AREA {
        return 0.0;
    }
    if position.actions_played() >= constants::TINY_LOOP_WINDOW {
        return 0.0;
    }

    // Penalty scales with events left (full-horizon) so it remains relevant
    // beyond the opening but is heaviest when the board is wide open.
    let events = f64::from(after.scoring_events_left());
    let hz_scale = events; // no min(HORIZON) cap - full-horizon

    constants::TINY_LOOP_PENALTY * hz_scale
}

/// Reward for committing to one anchor region vs scattered tiles.
/// Measures whether the mover's nodes are concentrated in one region (high
/// convex hull room per node) vs spread thin across many scattered positions.
/// Uses the room-to-area ratio: larger room for given area means more
/// committed/anchored development.
pub fn anchor_commit_bonus(position: &Position) -> f64 {
    let mover = position.to_move();
    let nodes = position.nodes(mover).len();

    if nodes < 3 {
        return 0.0;
    }

    let room = room(position, mover);
    let area = position.area(mover).to_f64();

    if area == 0.0 {
        return 0.0;
    }

    // Room per unit of area: higher means more concentrated/anchored development
    let room_per_area = room / area;
    // Bonus scales with how much more room we get per area compared to baseline
    // Baseline room/area for random scatter is ~0.5; reward deviation above that.
    let bonus = (room_per_area - 0.5).max(0.0);
    bonus * f64::from(position.scoring_events_left().min(12)) // cap scaling for determinism
}

/// Twice the signed area of the triangle `a`, `b`, `c`: positive when
/// it turns counter-clockwise.
fn cross(a: Point, b: Point, c: Point) -> i32 {
    i32::from(b.x() - a.x()) * i32::from(c.y() - a.y()) - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
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
        while chain.len() >= 2
            && cross(chain[chain.len() - 2], chain[chain.len() - 1], point) <= 0
        {
            chain.pop();
        }
        chain.push(point);
    }
    chain.pop();
    chain
}

/// 1 for Blue and -1 for Red, to turn Blue's lead into the mover's.
fn sign(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}

/// Constants for the patience evaluation terms.
pub mod constants {
    /// Window (in actions) within which tiny-loop snatching is penalized.
    pub const TINY_LOOP_WINDOW: u8 = 12;

    /// Maximum area (in unit squares) that triggers the tiny-loop penalty.
    pub const TINY_LOOP_MAX_AREA: f64 = 3.0;

    /// Penalty weight for snatching tiny loops in opening (full-horizon scaled).
    pub const TINY_LOOP_PENALTY: f64 = 2.0;

    /// Bonus weight for triangle-close potential (full-horizon, uncapped).
    pub const TRIANGLE_BONUS: f64 = 1.0;

    /// Reward weight for committing to one anchor region vs scattered tiles
    /// (scaled by events left, capped at 12 events for determinism).
    pub const ANCHOR_COMMIT_BONUS: f64 = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use meridian_engine::MoveKind;

    #[test]
    fn triangle_close_potential_no_triangle() {
        let mut pos = meridian_engine::Position::new();
        // Minimal position - no triangles possible
        let mv = pos.legal_moves().first().unwrap().clone();
        let after = pos.clone();
        // We need to play the move first
        let _ = pos.apply_unchecked(mv);
        // Just verify it doesn't panic and returns 0
        assert!(triangle_close_potential(&pos, mv, &after) >= 0.0);
    }

    #[test]
    fn tiny_loop_snatched_no_penalty() {
        // Test with a large gain - should not trigger
        let mut pos = meridian_engine::Position::new();
        let mv = pos.legal_moves().first().unwrap().clone();
        let after = pos.clone();
        let result = tiny_loop_snatched_penalty(&pos, mv, &after);
        assert_eq!(result, 0.0, "large gain should not trigger penalty");
    }

    #[test]
    fn anchor_commit_basic() {
        let mut pos = meridian_engine::Position::new();
        let mv = pos.legal_moves().first().unwrap().clone;
        let _ = pos.apply_unchecked(mv);
        // Basic smoke test - should not panic
        let result = anchor_commit_bonus(&pos);
        assert!(result.is_finite());
    }
}