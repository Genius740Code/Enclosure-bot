//! Lane B v1 head-to-head: `eval_phases` (legal-cuts-only vulnerability) vs the
//! frozen v1 baseline (`support/v1base.rs`), over the site's forced openings +
//! empty start (same 10-game set as `probe_v3all`). v1 is the live site bot and
//! the Blue-chair record vs v1 is problem #1, so this is the gate that counts.
//! Usage: cargo run --release --example probe_b_v1

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Game, Move, Player, Position};

fn play(mut game: Game, a_blue: bool, a: fn(&Position) -> Option<Move>, b: fn(&Position) -> Option<Move>) -> (f64, f64) {
    while !game.is_over() {
        let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
        game.play(if a_moves { a(game.position()) } else { b(game.position()) }.unwrap()).unwrap();
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}
fn wra(f: fn(&Position) -> Option<Move>) -> fn(&Position) -> Option<Move> { f }

fn main() {
    let laneb = wra(eval_phases::best_move);
    let v1 = wra(v1base::best_move);
    let (mut w, mut n) = (0, 0);
    let (mut wb, mut nb, mut wr, mut nr) = (0, 0, 0, 0);
    let (mut sum_b, mut sum_r) = (0.0, 0.0);
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let t = std::time::Instant::now();
            let (asc, bsc) = play(game, a_blue, laneb, v1);
            n += 1;
            let m = (asc - bsc) / asc * 100.0;
            if a_blue {
                nb += 1;
                sum_b += m;
                if asc > bsc { w += 1; wb += 1; }
            } else {
                nr += 1;
                sum_r += m;
                if asc > bsc { w += 1; wr += 1; }
            }
            println!(
                "laneB vs v1 open={open:?} a={} {asc:.0}-{bsc:.0} margin={m:+.1}% {} ({:.0}s)",
                if a_blue { "blue" } else { "red" },
                if asc > bsc { "A WINS" } else { "b wins" },
                t.elapsed().as_secs_f64()
            );
        }
    }
    println!(
        "==> laneB vs v1: {w}/{n} | as blue {wb}/{nb} avg margin {:+.1}% | as red {wr}/{nr} avg margin {:+.1}%",
        sum_b / nb as f64,
        sum_r / nr as f64
    );
}
