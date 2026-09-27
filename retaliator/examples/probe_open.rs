use meridian_engine::{Game, Move};
fn pt(mv: Move) -> String {
    let s = mv.source; let t = mv.target().unwrap();
    let n = |p: meridian_engine::Point| format!("{}{}", (b'A' + p.x() as u8 + 9) as char, p.y() + 10);
    format!("{}-{}", n(s), n(t))
}
fn main() {
    let mut game = Game::new();
    for _ in 0..6 {
        let mv = retaliator::search::best_move(game.position()).unwrap();
        println!("action {} {:?} plays {}", game.position().actions_played() + 1, game.position().to_move(), pt(mv));
        game.play(mv).unwrap();
    }
}
