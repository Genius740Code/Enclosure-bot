//! mesh_verify: port check for the v6 mesh8 prefix routing (NOT a gate).
//! 1. Routed game-start Blue move is D10-D13 (prefix beats opener).
//! 2. Routed bot vs PASSIVE opponent, both colors: inj==8 with no stall.
//!    (Pair-turn aware: sides act from to_move(), never from move count.)
//! 3. Informational: vs greedy (stall allowed, just report).
//! Usage: cargo run --release --example mesh_verify

use meridian_engine::{Game, Move, Player, Position, notation};
use retaliator::search;

fn text(mv: Move) -> String {
    match mv.target() {
        Some(t) => notation::move_text(mv.source, t),
        None => format!("off-board"),
    }
}

fn main() {
    // 1. Precedence at game start.
    let g = Game::new();
    let routed = search::best_move_routed(g.position(), g.moves(), &[]).expect("move");
    println!("routed start = {} (want D10-D13)", text(routed));
    assert_eq!(text(routed), "D10-D13", "prefix beats opener");

    // 2. inj==8 vs passive, both colors.
    for retaliator_blue in [true, false] {
        let ours = if retaliator_blue { Player::Blue } else { Player::Red };
        let mut game = Game::new();
        let (mut inj, mut stall) = (0usize, false);
        while !game.is_over() {
            let mv = if game.position().to_move() == ours {
                if search::mesh_prefix(game.position(), game.moves()).is_some() {
                    inj += 1;
                } else if inj < 8 {
                    stall = true;
                }
                search::best_move_routed(game.position(), game.moves(), &[])
            } else {
                first_legal(game.position())
            };
            let Some(mv) = mv else { break };
            game.play(mv).expect("legal");
            if game.moves().len() > 60 {
                break;
            }
        }
        println!(
            "as {} vs passive: inj={}/8 stall={}",
            if retaliator_blue { "blue" } else { "red" },
            inj,
            stall
        );
        assert_eq!(inj, 8, "prefix must inject all 8 vs passive");
        assert!(!stall, "no stall expected vs passive");
    }

    // 3. Informational vs greedy.
    for retaliator_blue in [true, false] {
        let ours = if retaliator_blue { Player::Blue } else { Player::Red };
        let mut game = Game::new();
        let (mut inj, mut stall_at) = (0usize, None::<usize>);
        while !game.is_over() {
            let mv = if game.position().to_move() == ours {
                if search::mesh_prefix(game.position(), game.moves()).is_some() {
                    inj += 1;
                } else if inj < 8 && stall_at.is_none() {
                    stall_at = Some(game.moves().len());
                }
                search::best_move_routed(game.position(), game.moves(), &[])
            } else {
                greedy(game.position())
            };
            let Some(mv) = mv else { break };
            game.play(mv).expect("legal");
            if game.moves().len() > 60 {
                break;
            }
        }
        println!(
            "as {} vs greedy: inj={}/8 stall_at={:?}",
            if retaliator_blue { "blue" } else { "red" },
            inj,
            stall_at
        );
    }
    println!("mesh_verify OK");
}

fn first_legal(position: &Position) -> Option<Move> {
    position.legal_moves().iter().min_by_key(|mv| mv.index())
}

/// 1-ply greedy, ties by move id (same as gauge).
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
