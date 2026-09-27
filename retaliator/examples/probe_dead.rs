use meridian_engine::{Game, Player};
fn main() {
    let mut game = Game::new();
    let (mut dead_b, mut dead_r) = (0, 0);
    let (mut tot_b, mut tot_r) = (0, 0);
    while !game.is_over() {
        let pos = game.position().clone();
        let mover = pos.to_move();
        let a = retaliator::search::analyze(&pos, 4096);
        let c = &a.candidates[0];
        // replay full pv to measure total effect of the chosen line
        let mut end = pos.clone();
        for &m in &c.pv { end.apply_unchecked(m); }
        let gain = end.area(mover).to_f64() - pos.area(mover).to_f64();
        let destroyed = (pos.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
        let scored = (end.score(mover).to_f64() - pos.score(mover).to_f64()).abs();
        let dead = gain.abs() < 0.01 && destroyed < 0.01 && scored < 0.01;
        if mover == Player::Blue { tot_b += 1; dead_b += dead as u32; }
        else { tot_r += 1; dead_r += dead as u32; }
        game.play(c.mv).unwrap();
    }
    println!("blue dead picks: {dead_b}/{tot_b}, red dead picks: {dead_r}/{tot_r}");
    println!("final b={:.0} r={:.0}", game.position().score(Player::Blue).to_f64(), game.position().score(Player::Red).to_f64());
}
