use meridian_engine::Game;
fn main() {
    let mut game = Game::new();
    let mut checked = 0; let mut subopt = 0;
    while !game.is_over() {
        let pos = game.position().clone();
        let mover = pos.to_move();
        let opp = mover.opponent();
        // all breaking first actions with their destroyed area
        let mut breaks: Vec<(usize, f64)> = vec![];
        for mv in pos.legal_moves().iter() {
            let mut after = pos.clone();
            let oc = after.apply_unchecked(mv);
            if oc.broken.is_some() {
                let d = (pos.area(opp).to_f64() - after.area(opp).to_f64()).max(0.0);
                breaks.push((mv.index(), d));
            }
        }
        let mv = retaliator::search::best_move(&pos).unwrap();
        if breaks.len() >= 2 {
            let mut after = pos.clone();
            let oc = after.apply_unchecked(mv);
            if oc.broken.is_some() {
                let picked = (pos.area(opp).to_f64() - after.area(opp).to_f64()).max(0.0);
                let maxd = breaks.iter().map(|(_, d)| *d).fold(0.0f64, f64::max);
                checked += 1;
                if maxd - picked > 1.0 {
                    subopt += 1;
                    if subopt <= 8 {
                        println!("action {}: picked break destroys {:.1}, best available {:.1} ({} options)", pos.actions_played() + 1, picked, maxd, breaks.len());
                    }
                }
            }
        }
        game.play(mv).unwrap();
    }
    println!("break decisions with 2+ options: {checked}, picked below-max by >1: {subopt}");
}
