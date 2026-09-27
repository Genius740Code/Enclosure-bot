use meridian_engine::Game;
fn main() {
    let mut game = Game::new();
    let mut checked = 0; let mut vindicated = 0; let mut real_miss = 0;
    while !game.is_over() {
        let pos = game.position().clone();
        let mover = pos.to_move();
        let opp = mover.opponent();
        let mut breaks: Vec<(usize, f64)> = vec![];
        for mv in pos.legal_moves().iter() {
            let mut after = pos.clone();
            let oc = after.apply_unchecked(mv);
            if oc.broken.is_some() {
                // post-reply surviving destruction: enemy's best reply, then measure
                let mut best_reply_d = 0.0f64;
                let mut any = false;
                for r in after.legal_moves().iter() {
                    let mut e2 = after.clone();
                    e2.apply_unchecked(r);
                    best_reply_d = best_reply_d.max((pos.area(opp).to_f64() - e2.area(opp).to_f64()).max(0.0));
                    let _ = any;
                }
                // surviving = immediate minus best enemy recovery... approximate: use min over replies? use max surviving:
                let mut min_d = f64::INFINITY;
                let mut n = 0;
                for r in after.legal_moves().iter().take(64) {
                    let mut e2 = after.clone();
                    e2.apply_unchecked(r);
                    min_d = min_d.min((pos.area(opp).to_f64() - e2.area(opp).to_f64()).max(0.0));
                    n += 1;
                }
                if n == 0 { min_d = 0.0; }
                let imm = (pos.area(opp).to_f64() - after.area(opp).to_f64()).max(0.0);
                breaks.push((mv.index(), imm * 1000.0 + min_d));
                let _ = best_reply_d;
            }
        }
        let mv = retaliator::search::best_move(&pos).unwrap();
        if breaks.len() >= 2 {
            let mut after = pos.clone();
            let oc = after.apply_unchecked(mv);
            if oc.broken.is_some() {
                let imm = (pos.area(opp).to_f64() - after.area(opp).to_f64()).max(0.0);
                let mut min_d = f64::INFINITY; let mut n = 0;
                for r in after.legal_moves().iter().take(64) {
                    let mut e2 = after.clone();
                    e2.apply_unchecked(r);
                    min_d = min_d.min((pos.area(opp).to_f64() - e2.area(opp).to_f64()).max(0.0));
                    n += 1;
                }
                if n == 0 { min_d = 0.0; }
                let pk = imm * 1000.0 + min_d;
                let mx = breaks.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
                checked += 1;
                if (mx - pk) > 1000.0 {
                    // picked smaller immediate break: check surviving values
                    let pk_surv = pk % 1000.0;
                    let mx_surv = breaks.iter().map(|(_, v)| v % 1000.0).fold(0.0f64, f64::max);
                    if mx_surv - pk_surv > 1.0 { real_miss += 1; } else { vindicated += 1; }
                }
            }
        }
        game.play(mv).unwrap();
    }
    println!("checked={checked} vindicated(small break survives better)={vindicated} real_miss={real_miss}");
}
