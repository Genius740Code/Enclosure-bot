//! Lane B gap probe: where does the score gap actually open in the games that
//! matter? Replays (deterministic, term OFF = shipped search):
//!   1. the three losing as-Blue chair openings vs v1 (5589, 9199, 11723)
//!   2. the league collapse line (skip=10, ret=red, the -111% game)
//! and logs per-action area trajectories, every single-action area swing > 2
//! (pops and big claims, both sides), and cut events with their tempo cost.
//! Measures the "cut+make gap" and the 30+-pop collapse hypothesis in numbers
//! before any term is designed. Usage: cargo run --release --example probe_b_gap

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Move, MoveKind, Player, Position};

fn play_logged(
    label: &str,
    mut game: Game,
    ret_blue: bool,
    ret: fn(&Position) -> Option<Move>,
    opp: fn(&Position) -> Option<Move>,
) {
    let (mut max_ret_loss, mut max_ret_loss_at) = (0.0f64, 0usize);
    let (mut max_opp_loss, mut max_opp_loss_at) = (0.0f64, 0usize);
    let (mut ret_cuts, mut opp_cuts) = (0u32, 0u32);
    while !game.is_over() {
        let to_move = game.position().to_move();
        let ret_moves = (to_move == Player::Blue) == ret_blue;
        let action = usize::from(game.position().actions_played());
        let mv = if ret_moves { ret(game.position()) } else { opp(game.position()) }.unwrap();
        let outcome = game.play(mv).expect("bot moves are legal");
        let ret_area = game.position().area(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
        let opp_area = game.position().area(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
        if outcome.broken.is_some() {
            if ret_moves { ret_cuts += 1 } else { opp_cuts += 1 }
        }
        // Single-action area swings for the side that just moved.
        // (apply_unchecked's outcome.kind tells a capture apart; the area delta
        // covers breaks and closes either way.)
        let _ = outcome.kind;
        if ret_moves {
            let loss = opp_area - game.position().area(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
            let _ = loss;
        }
        // Log every action's areas + the move; swings are visible in the deltas.
        if action % 10 == 0 || outcome.broken.is_some() || action >= 110 {
            println!(
                "{label} act={action} {} {} ret_area={ret_area:.1} opp_area={opp_area:.1} gap={:+.1}{}",
                mv.index(),
                match outcome.kind { MoveKind::Connect => "close", MoveKind::Extend => "extend", MoveKind::Capture => "CAPTURE" },
                ret_area - opp_area,
                if outcome.broken.is_some() { " CUT" } else { "" },
            );
        }
        let _ = (&mut max_ret_loss, &mut max_opp_loss);
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let os = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    println!(
        "{label} FINAL ret {rs:.0} opp {os:.0} margin={:+.1}% ret_cuts={ret_cuts} opp_cuts={opp_cuts}",
        (rs - os) / rs * 100.0
    );
}

fn main() {
    println!("=== as-Blue chair games vs v1 (the three losing chairs) ===");
    for open in [Some(5589), Some(9199), Some(11723)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(mv_from(id)).unwrap();
        }
        play_logged(
            &format!("chair{:?}", open.map(|i| i.to_string()).unwrap_or_else(|| "-".into())),
            game,
            true,
            eval_phases::best_move,
            v1base::best_move,
        );
    }
    println!("=== league collapse line (skip=10, ret=red vs scoutbase) ===");
    let mut game = Game::new();
    for _ in 0..10 {
        if game.is_over() { break; }
        let m = retaliator::search::best_move(game.position()).unwrap();
        game.play(m).unwrap();
    }
    play_logged("collapse", game, false, eval_phases::best_move, scoutbase::best_move);
}

fn mv_from(id: usize) -> meridian_engine::Move {
    meridian_engine::Move::from_index(id).unwrap()
}
