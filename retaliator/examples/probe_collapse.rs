use meridian_engine::{Game, Player};
#[path = "support/scoutbase.rs"]
mod scoutbase;
fn main() {
    let mut game = Game::new();
    for _ in 0..10 {
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    let mut t = 10;
    let (mut cut_us, mut cut_them) = (0, 0);
    while !game.is_over() {
        let red = game.position().to_move() == Player::Red;
        let mv = if red {
            retaliator::search::best_move(game.position()).unwrap()
        } else {
            scoutbase::best_move(game.position()).unwrap()
        };
        let r0 = game.position().area(Player::Red).to_f64();
        let oc = game.play(mv).unwrap();
        let r1 = game.position().area(Player::Red).to_f64();
        t += 1;
        if oc.broken.is_some() {
            if red { cut_them += 1; } else { cut_us += 1; }
        }
        // our (red) turns where OUR area drops, or enemy breaks
        if t >= 45 && t <= 85 && ((!red && oc.broken.is_some()) || (red && r1 < r0 - 0.5)) {
            println!("t={t} {} red_area {r0:.1}->{r1:.1} mv={} kind={:?}",
                if red { "us " } else { "OPP" }, mv.index(), oc.kind);
        }
    }
    println!("final: cuts by us={cut_them}, cuts suffered={cut_us}");
}
