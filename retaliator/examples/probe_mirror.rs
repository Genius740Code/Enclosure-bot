#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Player};
fn main() {
    for g in 0..2 {
        let mut game = Game::new();
        let mut brk = 0;
        while !game.is_over() {
            let mv = scoutbase::best_move(game.position()).unwrap();
            if game.play(mv).unwrap().broken.is_some() { brk += 1; }
        }
        println!("mirror {}: blue={:.0} red={:.0} barea={:.1} rarea={:.1} breaks={brk}",
            g + 1,
            game.position().score(Player::Blue).to_f64(),
            game.position().score(Player::Red).to_f64(),
            game.position().area(Player::Blue).to_f64(),
            game.position().area(Player::Red).to_f64());
    }
}
