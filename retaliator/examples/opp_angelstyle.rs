//! Lane D style-mimic: AngelBot WASM (1613) — alpha-beta middleweight.
//!
//! Per `research/rival-analysis.md` §4: no fixed book, first close at own
//! action ~12 as blue / 6 as red, 14-15 closes/game at ~5.8 area/loop —
//! between the big-loop bankers (~10/close) and the rush poppers (~1.8/close).
//! This mimic is Scout's search with a moderate close-size floor, so the
//! lane-D tournament isolates the close threshold (Scout 0 / Angel 3 / GB 8).

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Game, Move, MoveKind, Player, Position};

/// Only close loops worth at least this much area (AngelBot ~5.8/loop).
const MIN_CLOSE: f64 = 3.0;

pub fn best_move(position: &Position) -> Option<Move> {
    let me = position.to_move();
    let area_before = position.area(me).to_f64();
    let analysis = scoutbase::analyze(position, scoutbase::MOVE_BUDGET);
    for candidate in &analysis.candidates {
        if is_small_close(position, candidate, me, area_before) {
            continue;
        }
        return Some(candidate.mv);
    }
    analysis.candidates.first().map(|c| c.mv)
}

/// Whether the candidate's principal variation closes a loop worth less than
/// [`MIN_CLOSE`] area (the tiny-close habit this style refuses).
fn is_small_close(position: &Position, candidate: &scoutbase::Candidate, me: Player, area_before: f64) -> bool {
    let mut after = position.clone();
    let mut closes = false;
    for &mv in &candidate.pv {
        let outcome = after.apply_unchecked(mv);
        closes |= outcome.kind == MoveKind::Connect;
    }
    let gain = after.area(me).to_f64() - area_before;
    closes && gain < MIN_CLOSE
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
