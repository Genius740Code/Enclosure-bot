use meridian_engine::{Game, Player};
#[path = "support/scoutbase.rs"]
mod scoutbase;
fn main() {
    let mut game = Game::new();
    for _ in 0..10 {
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    let mut t = 10;
    while !game.is_over() {
        let mv = if game.position().to_move() == Player::Red {
            retaliator::search::best_move(game.position()).unwrap()
        } else {
            scoutbase::best_move(game.position()).unwrap()
        };
        game.play(mv).unwrap();
        t += 1;
        if t % 20 == 0 {
            println!("t={t} b={:.1}/{:.0} r={:.1}/{:.0}",
                game.position().area(Player::Blue).to_f64(), game.position().score(Player::Blue).to_f64(),
                game.position().area(Player::Red).to_f64(), game.position().score(Player::Red).to_f64());
        }
    }
}
