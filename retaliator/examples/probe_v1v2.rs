#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Game, Player};
fn main() {
    for g in 0..2 {
        let v2_blue = g == 0;
        let mut game = Game::new();
        while !game.is_over() {
            let v2_moves = (game.position().to_move() == Player::Blue) == v2_blue;
            let mv = if v2_moves {
                retaliator::search::best_move(game.position())
            } else {
                v1base::best_move(game.position())
            };
            game.play(mv.unwrap()).unwrap();
        }
        println!("game {} v2={}: blue={:.0} red={:.0} barea={:.1} rarea={:.1}",
            g + 1, if v2_blue { "blue" } else { "red" },
            game.position().score(Player::Blue).to_f64(),
            game.position().score(Player::Red).to_f64(),
            game.position().area(Player::Blue).to_f64(),
            game.position().area(Player::Red).to_f64());
    }
}
