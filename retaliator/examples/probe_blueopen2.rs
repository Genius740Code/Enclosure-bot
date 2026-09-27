#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/v2base.rs"]
mod v2base;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Move, Player, Position};
fn play(mut game: Game, red: fn(&Position) -> Option<Move>) -> (f64, f64) {
    while !game.is_over() {
        let mv = if game.position().to_move() == Player::Blue {
            retaliator::search::best_move(game.position())
        } else {
            red(game.position())
        };
        game.play(mv.unwrap()).unwrap();
    }
    (game.position().score(Player::Blue).to_f64(), game.position().score(Player::Red).to_f64())
}
fn main() {
    for (rn, r) in [("v1", v1base::best_move as fn(&Position) -> Option<Move>), ("v2", v2base::best_move), ("scout", scoutbase::best_move)] {
        for open in [None, Some(1979), Some(13892), Some(4145)] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let (b, rr) = play(game, r);
            println!("vs {rn} open={open:?} blue={b:.0} red={rr:.0} margin={:+.1}%", (b - rr) / b * 100.0);
        }
    }
}
