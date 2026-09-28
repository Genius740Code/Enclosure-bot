//! Gauge (Lane B): `eval_phases` vs greedy max-area 1-ply (the blob
//! archetype), alternating colors. Reports scores, margins, breaks. Same
//! structure as `gauge.rs`, with the variant wired in.
//!
//! B-3 farm-cycle routing (2026-09-28): OUR side's anti-rebuild avoid set
//! is fed live from the game (the `lib.rs` `replay()` wiring, memory
//! generalized). `B3_MEM` = the cut-memory window in actions — 0/absent =
//! no avoid = the published doom-OFF control (byte-identical); 6 = the
//! shipped site wiring; the B-3 sweep runs 12/20/40. `B3_SCALE` = 0 flat
//! routing only, 1 = + gain-scaled penalty keeping the counter-cut
//! exemption, 2 = + gain-scaled penalty on all first actions on cut
//! ground. Usage: `B3_MEM=20 B3_SCALE=2 cargo run --release --example gauge_b [games]`

#[path = "../src/eval_phases.rs"]
mod eval_phases;
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point, Position};

/// 1-ply greedy: maximize own enclosed area after the move, ties by move id.
fn greedy(position: &Position) -> Option<Move> {
    let mover = position.to_move();
    let mut best: Option<(f64, usize, Move)> = None;
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

/// Endpoints of `me`'s edges cut within the last `mem` actions: the shipped
/// anti-rebuild wiring (`lib.rs` `replay()`), memory generalized (`B3_MEM`).
/// `mem == 0` = no avoid set (the control).
fn avoid_points(cuts: &[(u16, Player, Edge)], me: Player, played: u16, mem: usize) -> Vec<Point> {
    let mut avoid = Vec::new();
    if mem == 0 {
        return avoid;
    }
    for &(action, owner, cut) in cuts.iter().rev() {
        if played - action > mem as u16 {
            break;
        }
        if owner == me {
            avoid.push(cut.origin());
            avoid.push(cut.far());
        }
    }
    avoid
}

#[derive(Default)]
struct MissedCloses {
    laneb_pot_sum: u32,
    laneb_turns: u32,
    laneb_closes: u32,
    greedy_pot_sum: u32,
    greedy_turns: u32,
    greedy_closes: u32,
}

fn play_game(laneb_blue: bool, mem: usize, scale: u8) -> (f64, f64, u32, u32, MissedCloses) {
    let mut game = Game::new();
    let (mut breaks_r, mut breaks_g) = (0u32, 0u32);
    let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
    let mut mc = MissedCloses::default();
    while !game.is_over() {
        let to_move = game.position().to_move();
        let played = u16::from(game.position().actions_played());
        let laneb_moves =
            to_move == Player::Blue && laneb_blue || to_move == Player::Red && !laneb_blue;
        
        // Track one-move potential at start of turn
        let pot_me = eval_phases::one_move_potential(game.position(), to_move);
        if laneb_moves {
            mc.laneb_pot_sum += pot_me;
            mc.laneb_turns += 1;
        } else {
            mc.greedy_pot_sum += pot_me;
            mc.greedy_turns += 1;
        }
        
        let mv = if laneb_moves {
            if mem == 0 {
                eval_phases::best_move(game.position())
            } else {
                let avoid = avoid_points(&cuts, to_move, played, mem);
                match scale {
                    0 => eval_phases::best_move_with_avoid(game.position(), &avoid),
                    1 => eval_phases::farm_best_move_with_avoid(
                        game.position(),
                        &avoid,
                        eval_phases::Rebuild::ScaledExempt,
                    ),
                    _ => eval_phases::farm_best_move_with_avoid(
                        game.position(),
                        &avoid,
                        eval_phases::Rebuild::ScaledAll,
                    ),
                }
            }
        } else {
            greedy(game.position())
        };
        let Some(mv) = mv else { break };
        let area_before = game.position().area(to_move).to_f64();
        let oc = game.play(mv).expect("bot moves are legal");
        let area_after = game.position().area(to_move).to_f64();
        let gain = area_after - area_before;
        
        // Count area-gaining Connect moves as "closes"
        if oc.kind == MoveKind::Connect && gain > 0.0 {
            if laneb_moves {
                mc.laneb_closes += 1;
            } else {
                mc.greedy_closes += 1;
            }
        }
        
        if let Some(cut) = oc.broken {
            cuts.push((played, to_move.opponent(), cut));
        }
        if oc.broken.is_some() {
            if laneb_moves {
                breaks_r += 1;
            } else {
                breaks_g += 1;
            }
        }
    }
    let b = game.position().score(Player::Blue).to_f64();
    let r = game.position().score(Player::Red).to_f64();
    (b, r, breaks_r, breaks_g, mc)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(2);
    let mem: usize = std::env::var("B3_MEM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let scale: u8 = std::env::var("B3_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    println!("B3 config: mem={mem} scale={scale}");
    let mut wins_r = 0;
    let mut total_laneb_missed = 0.0;
    let mut total_greedy_missed = 0.0;
    for g in 0..games {
        let laneb_blue = g % 2 == 0;
        let t = std::time::Instant::now();
        let (b, r, breaks_r, breaks_g, mc) = play_game(laneb_blue, mem, scale);
        let (rs, gs) = if laneb_blue { (b, r) } else { (r, b) };
        let margin = (rs - gs) / rs * 100.0;
        if rs > gs {
            wins_r += 1;
        }
        let laneb_avg_pot = if mc.laneb_turns > 0 { mc.laneb_pot_sum as f64 / mc.laneb_turns as f64 } else { 0.0 };
        let greedy_avg_pot = if mc.greedy_turns > 0 { mc.greedy_pot_sum as f64 / mc.greedy_turns as f64 } else { 0.0 };
        let laneb_missed = laneb_avg_pot - mc.laneb_closes as f64;
        let greedy_missed = greedy_avg_pot - mc.greedy_closes as f64;
        total_laneb_missed += laneb_missed;
        total_greedy_missed += greedy_missed;
        println!(
            "game {}: laneb={} blue={:.1} red={:.1} margin={:+.1}% breaks R={} G={} | missed: laneb={laneb_missed:.1} greedy={greedy_missed:.1} (pot: laneb={laneb_avg_pot:.1} greedy={greedy_avg_pot:.1}, closes: laneb={} greedy={}) ({:.0}s)",
            g + 1,
            if laneb_blue { "blue" } else { "red" },
            b,
            r,
            margin,
            breaks_r,
            breaks_g,
            mc.laneb_closes,
            mc.greedy_closes,
            t.elapsed().as_secs_f64()
        );
    }
    println!("laneb wins: {wins_r}/{games}");
    println!("==> missed-closes/game: laneb={:.2} greedy={:.2}", total_laneb_missed / games as f64, total_greedy_missed / games as f64);
}
