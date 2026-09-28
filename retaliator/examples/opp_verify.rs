//! Lane O (v7) step 1: verify collapser repro + CAPTURE_W ablation at tip.
//! Config: skip=0, ret=Blue, 8 solos, n=8 per arm. No eval-code changes:
//! baseline uses retaliator::search, ablation arm uses opp_nocap (search
//! copy with CAPTURE_W=0.0).

mod opp_collapser;
mod opp_nocap;

use meridian_engine::notation::parse_square;
use meridian_engine::{Game, Move, Player};

const SOLOS: [&str; 8] = [
    "D10-F11", "D10-C7", "A10-C11", "A10-D13", "D10-E12", "A10-B13", "D10-G10", "A10-C7",
];

fn solo_move(game: &Game, text: &str) -> Option<Move> {
    let (from, to) = text.split_once('-')?;
    let mv = Move::between(parse_square(from)?, parse_square(to)?)?;
    if game.legal_moves().contains(mv) {
        Some(mv)
    } else {
        game.legal_moves().iter().next()
    }
}

fn play(game: &mut Game, ret_blue: bool, solo: &str, nocap: bool) -> (f64, f64) {
    let mut first = true;
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if first {
            first = false;
            solo_move(game, solo)
        } else if ret_moves {
            if nocap {
                opp_nocap::best_move(game.position())
            } else {
                retaliator::search::best_move(game.position())
            }
        } else {
            opp_collapser::best_move(game.position())
        };
        game.play(mv.unwrap()).unwrap();
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss)
}

fn arm(label: &str, nocap: bool) {
    let mut w = 0;
    let mut l = 0;
    let mut sum = 0.0;
    for (g, solo) in SOLOS.iter().enumerate() {
        let mut game = Game::new();
        let (rs, ss) = play(&mut game, true, solo, nocap);
        let margin = (rs - ss) / rs * 100.0;
        if rs > ss { w += 1; } else { l += 1; }
        sum += margin;
        println!("  {} game{} margin={:+.1}% {}", label, g + 1, margin, if rs > ss { "WIN" } else { "LOSS" });
    }
    println!("{} skip=0 ret=Blue {}-{} avg={:+.1}% (n=8)", label, w, l, sum / 8.0);
}

fn main() {
    println!("=== VERIFY @ tip (skip=0 ret=Blue, 8 solos) ===");
    arm("BASELINE", false);
    arm("NOCAP(CAPTURE_W=0)", true);
}
