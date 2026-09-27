use meridian_engine::{Game, Player};
fn greedy(position: &meridian_engine::Position) -> Option<meridian_engine::Move> {
    let mover = position.to_move();
    let mut best: Option<(f64, usize, meridian_engine::Move)> = None;
    for mv in position.legal_moves().iter() {
        let mut after = position.clone();
        after.apply_unchecked(mv);
        let v = after.area(mover).to_f64();
        let id = mv.index();
        if best.is_none_or(|(bv, bid, _)| v > bv || (v == bv && id < bid)) {
            best = Some((v, id, mv));
        }
    }
    best.map(|(_, _, mv)| mv)
}
fn main() {
    // retaliator blue vs greedy red; log both areas every 10 turns
    let mut game = Game::new();
    println!("turn blue_area red_area");
    let mut t = 0;
    while !game.is_over() {
        let mv = if game.position().to_move() == Player::Blue {
            retaliator::search::best_move(game.position())
        } else {
            greedy(game.position())
        };
        game.play(mv.unwrap()).unwrap();
        t += 1;
        if t % 10 == 0 {
            println!("{t} {:.1} {:.1}",
                game.position().area(Player::Blue).to_f64(),
                game.position().area(Player::Red).to_f64());
        }
    }
    println!("final b={:.0} r={:.0}", game.position().score(Player::Blue).to_f64(), game.position().score(Player::Red).to_f64());
}
