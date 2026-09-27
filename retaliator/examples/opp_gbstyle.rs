//! Lane D style-mimic: Great Barrier (1907) — big-loop banker + fixed book.
//!
//! Per `research/rival-analysis.md` §4: walls ~12 own-actions with long
//! diagonals (two NE/SE diagonal walls), first close at own action ~13-15,
//! then banks only big loops (~10 area/close, 10-13 closes/game). This mimic
//! plays the observed 13-own-action book (mirrored by color), then runs
//! Scout's search but refuses any close worth less than [`MIN_CLOSE`] area —
//! the "banks big" half of the style.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::notation::parse_square;
use meridian_engine::{Game, Move, MoveKind, Player, Position};

/// Blue's book: solo (own action 1) + 12 long diagonal walls (own actions 2-13).
const BLUE_SOLO: &str = "D10-F11";
const BLUE_BOOK: [&str; 12] = [
    "D10-E12", "D10-F13", "E12-G15", "F13-H16", "G15-I18", "H16-J19", "I18-J19", "D10-F7",
    "D10-E8", "E8-G5", "F7-H4", "G5-I2",
];
/// Red's book: 13 walls (own actions 1-13), the color mirror.
const RED_BOOK: [&str; 13] = [
    "P10-O12", "P10-N13", "O12-M15", "N13-L16", "M15-K18", "L16-J19", "K18-J19", "P10-N7",
    "P10-O8", "O8-M5", "N7-L4", "M5-K2", "L4-J1",
];
/// Only close loops worth at least this much area (GB banks ~10/loop).
const MIN_CLOSE: f64 = 8.0;

pub fn best_move(position: &Position) -> Option<Move> {
    let me = position.to_move();
    // Own action number, 1-based: solo is 1, then two per 2-action turn.
    let own = (u16::from(position.actions_played()) + 2) / 2;
    let slot = match (me, own) {
        (Player::Blue, 1) => Some(BLUE_SOLO),
        (Player::Blue, 2..=13) => Some(BLUE_BOOK[(own - 2) as usize]),
        (Player::Red, 1..=13) => Some(RED_BOOK[(own - 1) as usize]),
        _ => None,
    };
    if let Some(text) = slot {
        if let Some(mv) = parse_move(text) {
            if position.legal_moves().contains(mv) {
                return Some(mv);
            }
        }
    }
    fallback(position)
}

/// Scout's search, minus the small-loop habit: the first ranked candidate
/// whose plan does not end in a sub-[`MIN_CLOSE`] close. Tiebreak by move
/// index (candidates are already index-sorted within equal value).
fn fallback(position: &Position) -> Option<Move> {
    let me = position.to_move();
    let area_before = position.area(me).to_f64();
    let analysis = scoutbase::analyze(position, scoutbase::MOVE_BUDGET);
    for candidate in &analysis.candidates {
        if is_small_close(position, candidate, me, area_before) {
            continue;
        }
        return Some(candidate.mv);
    }
    analysis.candidates.first().map(|c| c.mv)
}

/// Whether the candidate's principal variation closes a loop worth less than
/// [`MIN_CLOSE`] area (the tiny-close habit this style refuses).
fn is_small_close(position: &Position, candidate: &scoutbase::Candidate, me: Player, area_before: f64) -> bool {
    let mut after = position.clone();
    let mut closes = false;
    for &mv in &candidate.pv {
        let outcome = after.apply_unchecked(mv);
        closes |= outcome.kind == MoveKind::Connect;
    }
    let gain = after.area(me).to_f64() - area_before;
    closes && gain < MIN_CLOSE
}

/// "D10-E12" -> the move from D10 to E12. Legality is checked by the engine.
fn parse_move(text: &str) -> Option<Move> {
    let (from, to) = text.split_once('-')?;
    Move::between(parse_square(from)?, parse_square(to)?)
}

/// Smoke test: one game against Scout's search with each color, with a trace
/// of this style's own actions (own action, move, kind, area after).
fn main() {
    for g in 0..2 {
        let me_blue = g == 0;
        let mut game = Game::new();
        let mut trace = String::new();
        let mut area_after = 0.0;
        while !game.is_over() {
            let me_moves = (game.position().to_move() == Player::Blue) == me_blue;
            let own = (u16::from(game.position().actions_played()) + 2) / 2;
            let mv = if me_moves {
                best_move(game.position())
            } else {
                scoutbase::best_move(game.position())
            };
            let outcome = game.play(mv.unwrap()).unwrap();
            if me_moves {
                area_after = game.position().area(if me_blue { Player::Blue } else { Player::Red }).to_f64();
                trace.push_str(&format!("a{}:{:?}/{}@{:.0} ", own, outcome.kind, if outcome.broken.is_some() { "X" } else { "." }, area_after));
            }
        }
        let bs = game.position().score(Player::Blue).to_f64();
        let rs = game.position().score(Player::Red).to_f64();
        println!("game {} me={}: blue={:.0} red={:.0}", g + 1, if me_blue { "blue" } else { "red" }, bs, rs);
        println!("  trace: {trace}");
    }
}
