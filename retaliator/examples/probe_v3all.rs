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
    let v3 = wra(retaliator::search::best_move);
    let v1 = wra(v1base::best_move);
    let v2 = wra(v2base::best_move);
    for (aname, a, bname, b) in [("v3", v3, "v1", v1), ("v3", v3, "v2", v2)] {
        let (mut w, mut n) = (0, 0);
        for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
            for a_blue in [true, false] {
                let mut game = Game::new();
                if let Some(id) = open {
                    game.play(Move::from_index(id).unwrap()).unwrap();
                }
                let (asc, bsc) = play(game, a_blue, a, b);
                n += 1;
                if asc > bsc { w += 1; }
                println!("{aname} vs {bname} open={open:?} a={} {asc:.0}-{bsc:.0} {}", if a_blue { "blue" } else { "red" }, if asc > bsc { "A WINS" } else { "b wins" });
            }
        }
        println!("==> {aname} vs {bname}: {w}/{n}");
    }
}
