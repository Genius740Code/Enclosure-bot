//! Lane E v7 line-strength census (C2 spec, measurement-only, NO eval change).
//!
//! Corpus: h2h gate lines (5 openings x 2 colors, variant vs shipped) PLUS
//! league genuine rows (skip0 both colors, variant vs scoutbase), winner AND
//! loser lines (win control). Per CLOSE (Connect, gain >= 2.0) by EITHER
//! side, log at close time: gain; touches = max(source degree before,
//! target degree after) — the shared-node count with prior structure, same
//! definition as the E-term code; bank-rate = own score delta over the next
//! 6 own-actions / gain; survived = own area at game end still >= 50% of
//! the post-close peak; dose + game result + color + side.
//! Output: one JSONL row per close on stdout; human summary on stderr.
//! Analysis = survival by (gain, touches, bank-rate) stratified on game
//! result; min n=200 closes per dose before any claim (spec gate).
//! A "strength" predictor only counts if it flips a pick — this probe
//! builds no term, so pick-flip is N/A by construction (reported as such).
//! Usage: E_DENY=0.25 cargo run --release --example probe_b_census > census.jsonl

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Move, MoveKind, Player, Point, Position};

/// Degree of `player`'s node at `point` (same definition as E-term code).
fn node_degree(position: &Position, player: Player, point: Point) -> u32 {
    position
        .edges(player)
        .iter()
        .filter(|edge| edge.origin() == point || edge.far() == point)
        .count() as u32
}

fn variant_pick(position: &Position) -> Option<Move> {
    let y = eval_phases::Deny::from_env();
    if y.on() {
        eval_phases::deny_best_move_with_avoid(position, &[], y)
    } else {
        eval_phases::best_move(position)
    }
}

struct Snap {
    action: u16,
    mover: Player,
    blue_score: f64,
    red_score: f64,
    blue_area: f64,
    red_area: f64,
}

struct Close {
    game: String,
    side: String,
    color: String,
    action: u16,
    gain: f64,
    touches: u32,
    score_at: f64,
}

/// Play one game; `a` moves when (to_move == Blue) == a_blue. Record the
/// per-action trajectory plus every close (either side).
fn play(
    game_id: &str,
    mut game: Game,
    a_blue: bool,
    a: fn(&Position) -> Option<Move>,
    b: fn(&Position) -> Option<Move>,
    closes: &mut Vec<Close>,
    snaps: &mut Vec<Snap>,
) -> (f64, f64) {
    let base = snaps.len();
    let (mut n_close_a, mut n_close_b) = (0usize, 0usize);
    while !game.is_over() {
        let pos = game.position();
        let mover = pos.to_move();
        let a_moves = (mover == Player::Blue) == a_blue;
        let area_before = pos.area(mover).to_f64();
        let score_before = pos.score(mover).to_f64();
        let mv = (if a_moves { a } else { b })(pos).unwrap();
        let src = mv.source;
        let deg_src = node_degree(pos, mover, src);
        let tgt = mv.target().expect("legal moves end on the board");
        let act = u16::from(pos.actions_played());
        let oc = game.play(mv).unwrap();
        let pos2 = game.position();
        let gain = pos2.area(mover).to_f64() - area_before;
        if oc.kind == MoveKind::Connect && gain >= 2.0 {
            let touches = deg_src.max(node_degree(pos2, mover, tgt));
            closes.push(Close {
                game: game_id.to_string(),
                side: if a_moves { "a" } else { "b" }.to_string(),
                color: if mover == Player::Blue { "blue" } else { "red" }.to_string(),
                action: act,
                gain,
                touches,
                score_at: score_before,
            });
            if a_moves { n_close_a += 1; } else { n_close_b += 1; }
        }
        snaps.push(Snap {
            action: act + 1,
            mover,
            blue_score: pos2.score(Player::Blue).to_f64(),
            red_score: pos2.score(Player::Red).to_f64(),
            blue_area: pos2.area(Player::Blue).to_f64(),
            red_area: pos2.area(Player::Red).to_f64(),
        });
    }
    let _ = (base, n_close_a, n_close_b);
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}

fn score_of(s: &Snap, p: Player) -> f64 {
    if p == Player::Blue { s.blue_score } else { s.red_score }
}

fn area_of(s: &Snap, p: Player) -> f64 {
    if p == Player::Blue { s.blue_area } else { s.red_area }
}

fn main() {
    let dose = std::env::var("E_DENY").unwrap_or_else(|_| "control".to_string());
    let shipped = retaliator::search::best_move as fn(&Position) -> Option<Move>;
    let scout = scoutbase::best_move as fn(&Position) -> Option<Move>;
    let mut closes: Vec<Close> = Vec::new();
    let mut snaps: Vec<Snap> = Vec::new();
    // (game id, close-base index, snap-base index, a-won, a-is-variant)
    let mut games: Vec<(String, usize, usize, bool)> = Vec::new();

    // H2H gate lines: variant (a) vs shipped (b).
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let id = format!("h2h-{open:?}-{}", if a_blue { "blue" } else { "red" });
            let (cb, sb) = (closes.len(), snaps.len());
            let (asc, bsc) = play(&id, game, a_blue, variant_pick, shipped, &mut closes, &mut snaps);
            println!("{id}: {asc:.0}-{bsc:.0} {}", if asc > bsc { "A WINS" } else { "b wins" });
            games.push((id, cb, sb, asc > bsc));
        }
    }
    // League genuine rows: skip0, variant (a) vs scoutbase (b), both colors.
    for a_blue in [true, false] {
        let game = Game::new();
        let id = format!("skip0-{}", if a_blue { "blue" } else { "red" });
        let (cb, sb) = (closes.len(), snaps.len());
        let (asc, bsc) = play(&id, game, a_blue, variant_pick, scout, &mut closes, &mut snaps);
        println!("{id}: {asc:.0}-{bsc:.0} {}", if asc > bsc { "A WINS" } else { "b wins" });
        games.push((id, cb, sb, asc > bsc));
    }

    // Emit JSONL: resolve bank-rate + survival from the trajectory.
    // Per game, snaps[base..] are in action order; close.action = pre-move
    // actions_played, so its own snap index = first snap with action > close.action.
    let mut n = 0usize;
    for (gid, cb, sb, a_won) in &games {
        let _ = gid;
        let traj = &snaps[*sb..];
        for c in closes[*cb..].iter() {
            // Own snap: first trajectory entry after the close action.
            let own_idx = traj.iter().position(|s| s.action > c.action).unwrap_or(traj.len().saturating_sub(1));
            let p = if c.color == "blue" { Player::Blue } else { Player::Red };
            // Bank-rate: own score 6 own-actions later (or game end).
            let mut seen = 0usize;
            let mut later = traj.len() - 1;
            for (i, s) in traj.iter().enumerate().skip(own_idx) {
                if s.mover == p {
                    seen += 1;
                    if seen == 6 {
                        later = i;
                        break;
                    }
                }
            }
            let bank_rate = (score_of(&traj[later], p) - c.score_at) / c.gain.max(1e-9);
            // Survival: final area >= 50% of post-close peak.
            let peak = traj[own_idx..].iter().map(|s| area_of(s, p)).fold(0.0f64, f64::max);
            let survived = area_of(&traj[traj.len() - 1], p) >= 0.5 * peak;
            let a_side = c.side == "a";
            let won = if a_side { *a_won } else { !*a_won };
            println!(
                "{{\"game\":\"{}\",\"side\":\"{}\",\"color\":\"{}\",\"action\":{},\"gain\":{:.2},\"touches\":{},\"bank_rate\":{:.3},\"survived\":{},\"dose\":\"{}\",\"result\":\"{}\"}}",
                c.game, c.side, c.color, c.action, c.gain, c.touches, bank_rate, survived, dose,
                if won { "win" } else { "loss" },
            );
            n += 1;
        }
    }
    eprintln!("census: {n} closes (dose {dose}); n>=200 gate for any claim: {}", if n >= 200 { "MET" } else { "NOT MET" });
}
