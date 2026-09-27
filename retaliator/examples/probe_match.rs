#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Player};
fn main() {
    for g in 0..2 {
        let ret_blue = g == 0;
        let mut game = Game::new();
        let (mut brk_r, mut brk_s) = (0, 0);
        while !game.is_over() {
            let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
            let mv = if ret_moves {
                retaliator::search::best_move(game.position())
            } else {
                scoutbase::best_move(game.position())
            };
            let oc = game.play(mv.unwrap()).unwrap();
            if oc.broken.is_some() {
                if ret_moves { brk_r += 1; } else { brk_s += 1; }
            }
        }
        let bs = game.position().score(Player::Blue).to_f64();
        let rs = game.position().score(Player::Red).to_f64();
        let ba = game.position().area(Player::Blue).to_f64();
        let ra = game.position().area(Player::Red).to_f64();
        println!("game {} ret={}: blue_score={:.0} red_score={:.0} blue_area={:.1} red_area={:.1} breaks R={} S={}",
            g + 1, if ret_blue { "blue" } else { "red" }, bs, rs, ba, ra, brk_r, brk_s);
    }
}
