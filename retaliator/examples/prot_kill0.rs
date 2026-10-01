//! PROT-BLOCK kill-0: MESH_B_SEIZE behind Red-mesh-entry trigger, rule 6 kills entry-4.
//! Bars: 9/9 games deny the 4th Red entry AND Red area @16 <= 10.0 AND Blue
//! doom exposure @9 <= 4.5, on mirror-scope games (Red mirror-mesh + pocket
//! seek, Blue seize-when-armed else greedy).

use retaliator::search;
use meridian_engine::{Game, Move, Player, Point, Position, notation};

fn cheby(a: Point, b: Point) -> i8 {
    (a.x() - b.x()).abs().max((a.y() - b.y()).abs())
}

/// Worst one-action pop of `victim`'s area (frozen copy of search::max_pop).
fn max_pop(pos: &Position, victim: Player) -> f64 {
    let held = pos.area(victim).to_f64();
    let mut worst: f64 = 0.0;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        next.apply_unchecked(mv);
        worst = worst.max(held - next.area(victim).to_f64());
    }
    worst.max(0.0)
}

fn sq(s: &str) -> Point {
    notation::parse_square(s).expect("square")
}

fn mv_text(mv: Move) -> String {
    notation::move_text(mv.source, mv.target().expect("target"))
}

/// Greedy by own-area gain, ties to lowest index. Deterministic.
fn greedy(pos: &Position, mover: Player) -> Option<Move> {
    let mut legal: Vec<Move> = pos.legal_moves().iter().collect();
    legal.sort_by_key(|m| m.index());
    let base = pos.area(mover).to_f64();
    let mut best: Option<(f64, Move)> = None;
    for mv in legal {
        let mut after = pos.clone();
        after.apply_unchecked(mv);
        let gain = after.area(mover).to_f64() - base;
        best = Some(match best {
            Some((g, bm)) => {
                if gain.total_cmp(&g) == std::cmp::Ordering::Greater {
                    (gain, mv)
                } else {
                    (g, bm)
                }
            }
            None => (gain, mv),
        });
    }
    best.map(|(_, m)| m)
}

fn main() {
    let center = sq("G11");
    let seize: [(Point, Point); 4] = [(sq("D10"), sq("G10")), (sq("G10"), sq("J11")), (sq("D10"), sq("G13")), (sq("G13"), sq("J11"))];
    let in_pocket = |p: Point| cheby(p, center) <= 2;
    let control = std::env::var("PROT_CONTROL").is_ok();
    let mut all_pass = true;
    for g in 0..9usize {
        let mut game = Game::new();
        let mut played_seize = [false; 4];
        let mut entries = 0u32;
        let mut denials = 0u32;
        let mut red16 = f64::NAN;
        let mut blue9 = f64::NAN;
        while game.moves().len() < 40 && !game.position().is_finished() {
            let pos = game.position().clone();
            let mover = pos.to_move();
            if mover == Player::Blue {
                let armed = !control && pos.nodes(Player::Red).iter().any(|n| in_pocket(n));
                let mut done = false;
                if armed {
                    for i in 0..4 {
                        if played_seize[i] {
                            continue;
                        }
                        if let Some(mv) = Move::between(seize[i].0, seize[i].1) {
                            if pos.check_move(mv).is_ok() {
                                game.play(mv).expect("seize legal");
                                played_seize[i] = true;
                                done = true;
                                break;
                            }
                        }
                    }
                }
                if !done {
                    let mv = greedy(&pos, mover).expect("blue move");
                    game.play(mv).expect("legal");
                }
            } else {
                if let Some(pm) = search::mesh_prefix(game.position(), game.moves()) {
                    let tgt = pm.target().expect("target");
                    if !control && in_pocket(tgt) && entries >= 3 {
                        denials += 1;
                    } else {
                        game.play(pm).expect("prefix legal");
                        if in_pocket(tgt) {
                            entries += 1;
                        }
                    }
                } else {
                    let mut legal: Vec<Move> = pos.legal_moves().iter().collect();
                    legal.sort_by_key(|m| m.index());
                    let base = pos.area(Player::Red).to_f64();
                    let mut scored: Vec<(i8, f64, usize, Move)> = legal
                        .into_iter()
                        .map(|mv| {
                            let mut after = pos.clone();
                            after.apply_unchecked(mv);
                            let tgt = mv.target().expect("target");
                            (cheby(tgt, center), base - after.area(Player::Red).to_f64(), mv.index(), mv)
                        })
                        .collect();
                    scored.sort_by(|a, b| {
                        a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2))
                    });
                    let pref = g % scored.len().min(3).max(1);
                    let mut done = false;
                    for i in pref..scored.len() {
                        let mv = scored[i].3;
                        let tgt = mv.target().expect("target");
                        if !control && in_pocket(tgt) && entries >= 3 {
                            denials += 1;
                            continue;
                        }
                        game.play(mv).expect("legal");
                        if in_pocket(tgt) {
                            entries += 1;
                        }
                        done = true;
                        break;
                    }
                    if !done {
                        panic!("red has no admissible move");
                    }
                }
            }
            let n = game.moves().len();
            if n == 9 {
                blue9 = max_pop(game.position(), Player::Blue);
            }
            if n == 16 {
                red16 = game.position().area(Player::Red).to_f64();
            }
        }
        let ok = denials >= 1 && red16 <= 10.0 && blue9 <= 4.5;
        all_pass &= ok;
        println!(
            "game {g}: denials={denials} red16={red16:.1} blue9={blue9:.1} {}",
            if ok { "PASS" } else { "FAIL" }
        );
        let _ = mv_text;
    }
    println!("{}", if all_pass { "PROT-KILL0 GO" } else { "PROT-KILL0 KILL" });
}
