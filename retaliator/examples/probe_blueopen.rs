#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Game, Move, Player};
fn play(mut game: Game) -> (f64, f64) {
    while !game.is_over() {
        let v3_moves = game.position().to_move() == Player::Blue;
        let mv = if v3_moves {
            retaliator::search::best_move(game.position())
        } else {
            v1base::best_move(game.position())
        };
        game.play(mv.unwrap()).unwrap();
    }
    (game.position().score(Player::Blue).to_f64(), game.position().score(Player::Red).to_f64())
}
fn main() {
    // candidate Blue action-1 ids (None = unforced): E12, F13, D13, F7, E8, G10
    for open in [None, Some(13892), Some(16780), Some(16058), Some(1979), Some(4145), Some(9560)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        let (b, r) = play(game);
        println!("open={open:?} blue={b:.0} red={r:.0} margin={:+.1}%", (b - r) / b * 100.0);
    }
}
