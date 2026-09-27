#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Game, Move, Player};
fn play(mut game: Game, v2_blue: bool) -> (f64, f64) {
    while !game.is_over() {
        let v2_moves = (game.position().to_move() == Player::Blue) == v2_blue;
        let mv = if v2_moves {
            retaliator::search::best_move(game.position())
        } else {
            v1base::best_move(game.position())
        };
        game.play(mv.unwrap()).unwrap();
    }
    let v2s = game.position().score(if v2_blue { Player::Blue } else { Player::Red }).to_f64();
    let v1s = game.position().score(if v2_blue { Player::Red } else { Player::Blue }).to_f64();
    (v2s, v1s)
}
fn main() {
    // Site forced openings (blue's first action) + empty start.
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for v2_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let (v2s, v1s) = play(game, v2_blue);
            println!("open={open:?} v2={} v2s={v2s:.0} v1s={v1s:.0} {}",
                if v2_blue { "blue" } else { "red" },
                if v2s > v1s { "V2 WINS" } else { "v1 wins" });
        }
    }
}
