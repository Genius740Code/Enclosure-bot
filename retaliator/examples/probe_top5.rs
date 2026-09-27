use meridian_engine::{Game, Player};
fn main() {
    let mut game = Game::new();
    for _ in 0..40 {
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    let pos = game.position().clone();
    let mover = pos.to_move();
    let opp = mover.opponent();
    let events = pos.scoring_events_left() as f64;
    println!("actions={} to_move={:?} events_left={events} scores b={:.0} r={:.0} areas b={:.1} r={:.1}",
        pos.actions_played(), mover,
        pos.score(Player::Blue).to_f64(), pos.score(Player::Red).to_f64(),
        pos.area(Player::Blue).to_f64(), pos.area(Player::Red).to_f64());
    let a = retaliator::search::analyze(&pos, 4096);
    println!("--- top candidates (picked order) ---");
    for c in a.candidates.iter().take(5) {
        let mut after = pos.clone();
        let oc = after.apply_unchecked(c.mv);
        let end = after.clone();
        let destroyed = (pos.area(opp).to_f64() - end.area(opp).to_f64()).max(0.0);
        let gain = end.area(mover).to_f64() - pos.area(mover).to_f64();
        let statik = retaliator::search::evaluate(&end);
        println!("mv={} eval={:+.1} static={:+.1} kind={:?} broke={} destroyed={:.1} gain={:+.1}",
            c.mv.index(), c.evaluation, statik, oc.kind, oc.broken.is_some(), destroyed, gain);
    }
    println!("--- best non-breaking moves by static eval ---");
    let mut nb: Vec<(f64, usize, f64)> = vec![];
    for mv in pos.legal_moves().iter() {
        let mut after = pos.clone();
        let oc = after.apply_unchecked(mv);
        if oc.broken.is_none() {
            let gain = after.area(mover).to_f64() - pos.area(mover).to_f64();
            nb.push((retaliator::search::evaluate(&after), mv.index(), gain));
        }
    }
    nb.sort_by(|x, y| y.0.total_cmp(&x.0));
    for (s, id, g) in nb.iter().take(5) {
        println!("mv={id} static={s:+.1} gain={g:+.1}");
    }
}
