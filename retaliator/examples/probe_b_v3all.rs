//! Lane B round robin: `eval_phases` (legal-cuts-only vulnerability) vs the
//! v1 and v2 baselines over the site's forced openings + empty start (same
//! set as `probe_v3all`, 10 games per matchup). This is the direct Blue-chair
//! comparable: the shipped baseline is v3-vs-v1 4/10 (as Blue 1/4 from forced
//! openings), v3-vs-v2 7/10. Usage: cargo run --release --example probe_b_v3all

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/v2base.rs"]
mod v2base;
use meridian_engine::{Game, Move, Player};
fn play(mut game: Game, a_blue: bool, a: fn(&meridian_engine::Position) -> Option<Move>, b: fn(&meridian_engine::Position) -> Option<Move>) -> (f64, f64) {
    while !game.is_over() {
        let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
        game.play(if a_moves { a(game.position()) } else { b(game.position()) }.unwrap()).unwrap();
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}
fn wra(f: fn(&meridian_engine::Position) -> Option<Move>) -> fn(&meridian_engine::Position) -> Option<Move> { f }
fn main() {
    let laneb = wra(eval_phases::best_move);
    let v1 = wra(v1base::best_move);
    let v2 = wra(v2base::best_move);
    for (aname, a, bname, b) in [("laneB", laneb, "v1", v1), ("laneB", laneb, "v2", v2)] {
        let (mut w, mut n) = (0, 0);
        let (mut wb, mut nb, mut wr, mut nr) = (0, 0, 0, 0);
        let (mut sum_b, mut sum_r) = (0.0, 0.0);
        for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
            for a_blue in [true, false] {
                let mut game = Game::new();
                if let Some(id) = open {
                    game.play(Move::from_index(id).unwrap()).unwrap();
                }
                let (asc, bsc) = play(game, a_blue, a, b);
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
                println!("{aname} vs {bname} open={open:?} a={} {asc:.0}-{bsc:.0} margin={m:+.1}% {}", if a_blue { "blue" } else { "red" }, if asc > bsc { "A WINS" } else { "b wins" });
            }
        }
        println!("==> {aname} vs {bname}: {w}/{n} | as blue {wb}/{nb} avg margin {:+.1}% | as red {wr}/{nr} avg margin {:+.1}%", sum_b / nb as f64, sum_r / nr as f64);
    }
}
