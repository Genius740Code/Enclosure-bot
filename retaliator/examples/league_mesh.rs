//! league_mesh: genuine 8-game probe_league structure, our side routed
//! (mesh8 prefix + search, no avoid — same harness as the published
//! -9.9%/-20.0% controls for comparability; harness-gap caveat applies).
//! DOOM_W variant is compile-time (see search.rs): run once per value.
//! Usage: cargo run --release --example league_mesh
//! Accept bar: AVG better than -9.9% AND no collapse row worse than -300.

#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Player};

fn play(mut game: Game, ret_blue: bool) -> (f64, f64) {
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if ret_moves {
            retaliator::search::best_move_routed(game.position(), game.moves(), &[])
        } else {
            scoutbase::best_move(game.position())
        };
        game.play(mv.unwrap()).unwrap();
        if game.moves().len() > 600 {
            break;
        }
    }
    let rs = game
        .position()
        .score(if ret_blue { Player::Blue } else { Player::Red })
        .to_f64();
    let ss = game
        .position()
        .score(if ret_blue { Player::Red } else { Player::Blue })
        .to_f64();
    (rs, ss)
}

fn main() {
    let mut sum = 0.0;
    let mut n = 0;
    let mut worst = 0.0f64;
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() {
                    break;
                }
                let mv = retaliator::search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                continue;
            }
            let t = std::time::Instant::now();
            let (rs, ss) = play(game, ret_blue);
            let m = (rs - ss) / rs * 100.0;
            println!(
                "skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% ({:.0}s)",
                if ret_blue { "blue" } else { "red" },
                t.elapsed().as_secs_f64()
            );
            sum += m;
            n += 1;
            worst = worst.min(m);
        }
    }
    println!("AVG margin (ret perspective): {:.1}%", sum / n as f64);
    println!("worst row: {worst:.1}%");
}
