//! Lane A probe: search_deep (depth 3, 2s budget) vs v1
//!
//! Runs 10 games alternating colors.

use meridian_engine::{Game, Move, Player, Position};

#[path = "support/v1base.rs"]
mod v1base;

/// The candidate: real alpha-beta with move ordering, depth 3, 2s budget, DEFAULT_WIDTH.
fn deep_budget(position: &Position) -> Option<Move> {
    retaliator::search_deep::best_move_capped(
        position,
        retaliator::search_deep::DEFAULT_DEPTH,
        &[],
        &retaliator::search_deep::Budget::Ms(retaliator::search_deep::DEFAULT_BUD_MS),
        retaliator::search_deep::DEFAULT_WIDTH,
    )
}

/// The v1 baseline.
fn v1(position: &Position) -> Option<Move> {
    v1base::best_move(position)
}

fn play(mut game: Game, a_blue: bool, a: fn(&Position) -> Option<Move>, b: fn(&Position) -> Option<Move>) -> (f64, f64) {
    while !game.is_over() {
        let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
        game.play(if a_moves { a(game.position()) } else { b(game.position()) }.unwrap()).unwrap();
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}

fn main() {
    println!("== h2h: search_deep (depth 3, 2s budget) vs v1, 10 games ==");
    let (mut w, mut n) = (0, 0);
    let (mut w_blue, mut n_blue, mut w_red, mut n_red) = (0, 0, 0, 0);
    for g in 0..10 {
        let deep_blue = g % 2 == 0;
        let mut game = Game::new();
        let t = std::time::Instant::now();
        let (dsc, v1sc) = play(game, deep_blue, deep_budget, v1);
        n += 1;
        let won = dsc > v1sc;
        if won { w += 1; }
        if deep_blue { n_blue += 1; if won { w_blue += 1; } } else { n_red += 1; if won { w_red += 1; } }
        println!(
            "game {}: deep={} {:.0}-{:.0} {} ({:.0}s)",
            g + 1,
            if deep_blue { "blue" } else { "red" },
            dsc, v1sc,
            if won { "DEEP WINS" } else { "v1 wins" },
            t.elapsed().as_secs_f64()
        );
    }
    println!("==> h2h search_deep vs v1: {w}/{n} (blue {w_blue}/{n_blue}, red {w_red}/{n_red})");
}