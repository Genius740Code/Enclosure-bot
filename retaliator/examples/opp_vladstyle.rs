//! Lane D style-mimic: VladNet (1584) — small-loop popper + max-pop cutter.
//!
//! Per `research/rival-analysis.md` §4-5: closes its first loop at action 2
//! and keeps popping (30-32 tiny loops/game), and against a rebuilder it
//! actively demolishes — 41 cuts in `f61a06ec`. This mimic always takes the
//! cut that pops the most enemy area whenever a cut is available, else pops
//! the fattest tiny loop it can, else hands the action to Scout's search.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Game, Move, MoveKind, Player, Position};

pub fn best_move(position: &Position) -> Option<Move> {
    let me = position.to_move();
    let opp = me.opponent();
    let area_opp = position.area(opp).to_f64();
    let area_me = position.area(me).to_f64();

    // 1. Max-pop cut: the cut destroying the most enemy area; ties by move index.
    let mut best_cut: Option<(f64, Move)> = None;
    // 2. Max-gain close: pops tiny loops fast (first close at action 2).
    let mut best_close: Option<(f64, Move)> = None;
    for mv in moves(position) {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        if outcome.broken.is_some() {
            let popped = area_opp - after.area(opp).to_f64();
            if best_cut.is_none_or(|(p, _)| popped > p) {
                best_cut = Some((popped, mv));
            }
        } else if outcome.kind == MoveKind::Connect {
            let gain = after.area(me).to_f64() - area_me;
            if gain > 0.0 && best_close.is_none_or(|(g, _)| gain > g) {
                best_close = Some((gain, mv));
            }
        }
    }
    if let Some((_, mv)) = best_cut {
        return Some(mv);
    }
    if let Some((_, mv)) = best_close {
        return Some(mv);
    }
    // 3. No cut, no close: extend walls with Scout's search.
    scoutbase::best_move(position)
}

/// The mover's legal moves without the second listing of each own connection.
fn moves(position: &Position) -> Vec<Move> {
    position
        .legal_moves()
        .iter()
        .filter(|&mv| !repeats_a_connection(position, mv))
        .collect()
}

/// A connection between two of the mover's nodes is listed from both ends. This is the second.
fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

/// Smoke test: one game against Scout's search with each color.
fn main() {
    for g in 0..2 {
        let me_blue = g == 0;
        let mut game = Game::new();
        while !game.is_over() {
            let me_moves = (game.position().to_move() == Player::Blue) == me_blue;
            let mv = if me_moves {
                best_move(game.position())
            } else {
                scoutbase::best_move(game.position())
            };
            game.play(mv.unwrap()).unwrap();
        }
        let bs = game.position().score(Player::Blue).to_f64();
        let rs = game.position().score(Player::Red).to_f64();
        println!("game {} me={}: blue={:.0} red={:.0}", g + 1, if me_blue { "blue" } else { "red" }, bs, rs);
    }
}
