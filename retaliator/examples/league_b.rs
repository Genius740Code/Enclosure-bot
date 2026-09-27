//! League (Lane B): `eval_phases` (legal-cuts-only vulnerability) vs scoutbase,
//! 8 games (skip 0/10/20/30 x 2 colors), same structure as `probe_league.rs`
//! with the in-game variant wired in. The skip prelude stays the shipped
//! search's self-play (unchanged openings, so the collapse-line rows stay
//! comparable with the scoreboard baseline); the controlled comparison is
//! `probe_b_h2h`. Usage: cargo run --release --example league_b

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Player};
fn play(mut game: Game, ret_blue: bool) -> (f64, f64) {
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if ret_moves {
            eval_phases::best_move(game.position())
        } else {
            scoutbase::best_move(game.position())
        };
        game.play(mv.unwrap()).unwrap();
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss)
}
fn main() {
    let mut sum = 0.0; let mut n = 0;
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() { break; }
                let mv = retaliator::search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() { continue; }
            let t = std::time::Instant::now();
            let (rs, ss) = play(game, ret_blue);
            let m = (rs - ss) / rs * 100.0;
            println!("skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% ({:.0}s)", if ret_blue { "blue" } else { "red" }, t.elapsed().as_secs_f64());
            sum += m; n += 1;
        }
    }
    println!("AVG margin (ret perspective): {:.1}%", sum / n as f64);
}
