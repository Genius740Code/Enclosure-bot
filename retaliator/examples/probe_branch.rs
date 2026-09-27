use meridian_engine::{Game, Player};
fn main() {
    let mut game = Game::new();
    let mut total = 0u64; let mut n = 0u32; let mut max = 0usize; let mut over2048 = 0u32;
    while !game.is_over() {
        let c = game.legal_moves().len();
        total += c as u64; n += 1; max = max.max(c);
        if c > 2048 { over2048 += 1; }
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
        let _ = Player::Blue;
    }
    println!("positions: {n}, avg legal: {:.0}, max legal: {max}, positions over first-ply budget (2048): {over2048}", total as f64 / n as f64);
}
