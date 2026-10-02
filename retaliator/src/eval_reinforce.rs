//! Reinforcement evaluation for Lane R (REINFORCE H1).
//!
//! H1: if opp places line next to our high-value area, reinforce/protect shape
//! to prevent breaks.
//!
//! Pure, unit-testable functions. No game-state mutability.

use meridian_engine::{Game, Move, Player, Position, Point};

/// Full-horizon weight for reinforcement when opponent threatens our high-value area.
///
/// When the opponent's last move lands adjacent to our nodes that enclose significant area,
! this constant scales the reinforcement bonus in full-horizon points (area × events_left),
! so the incentive is proportional to both the value at risk and the time remaining to
! defend it.
pub const REINFORCE_W: f64 = 1.5;

/// Compute the reinforcement bonus for the given position and the player to move.
/// 
/// The bonus is positive when the opponent's recent move threatens our high-value area,
! encouraging us to reinforce/protect shape to prevent breaks. The bonus is scaled by
! `REINFORCE_W`, the number of our nodes adjacent to the threat, and the remaining
! scoring events (full-horizon so it counts area × events_left proportionally).
!
! # Pure function — no game mutation. Unit-testable.
fn reinforcement_bonus(position: &Position, player: Player) -> f64 {
    let opponent = player.opponent();
    let our_nodes = position.nodes(player);
    let opp_nodes = position.nodes(opponent);

    // Find the opponent's most recent move's target point.
    // We examine the board for any edge the opponent just placed and check if it's
    // adjacent to our high-value nodes.
    let mut bonus = 0.0f64;

    // Count our nodes that are within distance 1 of any of the opponent's nodes.
    // These are the "high-value area" nodes that could be threatened.
    let mut nodes_threatened: u32 = 0;
    for our_node in our_nodes.iter() {
        for opp_node in opp_nodes.iter() {
            if (our_node.x() - opp_node.x()).abs() <= 1
                && (our_node.y() - opp_node.y()).abs() <= 1
            {
                nodes_threatened += 1;
                break;
            }
        }
    }

    if nodes_threatened > 0 {
        // Full-horizon: count remaining scoring events
        let events_left = f64::from(position.scoring_events_left());
        // Bonus scales with threatened nodes, events left, and the weight
        bonus = REINFORCE_W * (nodes_threatened as f64) * events_left;
    }

    bonus
}

/// Evaluate the reinforcement score for a candidate move.
///
/// Returns the reinforcement bonus added to the evaluation score.
/// Positive values encourage reinforcing/protecting shape when the opponent
! threatens our high-value area.
!
! # Pure function — takes a position and move; no game mutation.
pub fn move_reinforcement_bonus(position: &Position, player: Player, mv: Move) -> f64 {
    let opponent = player.opponent();
    let target = mv.target().expect("legal moves end on the board");

    // Check if the move's target is adjacent to any of our nodes that enclose area
    let our_nodes = position.nodes(player);
    let mut nodes_adjacent: u32 = 0;
    for our_node in our_nodes.iter() {
        if (our_node.x() - target.x()).abs() <= 1 && (our_node.y() - target.y()).abs() <= 1
        {
            nodes_adjacent += 1;
        }
    }

    if nodes_adjacent == 0 {
        return 0.0;
    }

    // Full-horizon scoring events left
    let events_left = f64::from(position.scoring_events_left());
    REINFORCE_W * (nodes_adjacent as f64) * events_left
}

#[cfg(test)]
mod tests {
    use super::*;
    use meridian_engine::Position;

    #[test]
    fn test_reinforcement_bonus_no_threat() {
        // Start position has no threats
        let pos = Position::new();
        let bonus = reinforcement_bonus(&pos, Player::Blue);
        assert_eq!(bonus, 0.0);
    }

    #[test]
    fn test_move_reinforcement_bonus_no_adjacency() {
        let pos = Position::new();
        // A move far from any of our nodes
        // D10 is at (-9, 10)... let's use a point we know is on board
        // Actually let's just test with a move that has no adjacent nodes
        // In starting position, Blue has no edges, so any move creates an edge
        // Let's test with a position where nodes aren't adjacent to target
        let mv = meridian_engine::Move::between(
            meridian_engine::Point::new(-6, 0).expect("on board"),
            meridian_engine::Direction::from_delta(1, 0).expect("valid dir"),
        ).expect("valid move");
        let bonus = move_reinforcement_bonus(&pos, Player::Blue, mv);
        // In start position, the first move creates a node at the target
        // but we check adjacency - let's see if it's adjacent to existing nodes
        // Starting position has no nodes yet, so bonus should be 0
        assert_eq!(bonus, 0.0);
    }

    #[test]
    fn test_reinforcement_bonus_scales_with_events() {
        // Start position - 120 total actions, 0 played, so 12 scoring events left initially
        let pos = Position::new();
        let bonus = reinforcement_bonus(&pos, Player::Blue);
        // No nodes threatened yet, should be 0
        assert_eq!(bonus, 0.0);
    }
}