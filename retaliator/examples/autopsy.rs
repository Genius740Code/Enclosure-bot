use meridian_engine::{Game, Move, Player};
fn main() {
    let path = std::env::args().nth(1).expect("game json path");
    let me = std::env::args().nth(2).expect("our color: blue|red");
    let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let ids: Vec<usize> = d["moves"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
    let me_p = if me == "blue" { Player::Blue } else { Player::Red };
    let mut game = Game::new();
    println!("move blue_area red_area blue_score red_score | our moves annotated");
    let mut n = 0;
    for id in ids {
        let mv = Move::from_index(id).expect("site id");
        let ours = game.position().to_move() == me_p;
        let b0 = game.position().area(Player::Blue).to_f64();
        let r0 = game.position().area(Player::Red).to_f64();
        let oc = game.play(mv).expect("site moves are legal");
        n += 1;
        let b1 = game.position().area(Player::Blue).to_f64();
        let r1 = game.position().area(Player::Red).to_f64();
        // enemy breaks of OUR loops / our breaks, and big area swings
        let my_area = if me_p == Player::Blue { b1 } else { r1 };
        let my_prev = if me_p == Player::Blue { b0 } else { r0 };
        let swing = my_area - my_prev;
        if ours {
            let opp = me_p.opponent();
            let od = (if opp == Player::Blue { b0 } else { r0 }) - (if opp == Player::Blue { b1 } else { r1 });
            let flag = if oc.broken.is_some() { format!(" BREAK(opp{od:+.1})") } else { String::new() };
            println!("  [{n}] us mv={id} kind={:?} myArea{swing:+.1}{flag}", oc.kind);
        } else if oc.broken.is_some() {
            println!("  [{n}] OPP BREAK myArea{swing:+.1} id={id}");
        }
        if n % 10 == 0 || game.is_over() {
            println!("= after {n}: areas b={b1:.1} r={r1:.1} scores b={:.0} r={:.0}",
                game.position().score(Player::Blue).to_f64(),
                game.position().score(Player::Red).to_f64());
        }
    }
}
