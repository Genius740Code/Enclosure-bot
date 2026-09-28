//! Lane D2 style-mimic: AngelBot WASM's bank-sacrifice tempo trade — C2
//! autopsy §3 (`research/c-site-losses-2.md` on `origin/lane-c-autopsy`).
//!
//! Signature (site games `1c7b1aa1`, `37946f6f`, `4929b951`): banks early
//! and often (close floor 3.0, double-duty close+cut preferred), farms every
//! enemy balloon with the max-pop cut, and — the twist this probe isolates —
//! lets its own already-banked loops STAND POPPED: re-closing ground the
//! enemy just cut is tempo donated for nothing (the bank is cumulative;
//! cutting a banked loop recovers no score), so it re-closes only BIG
//! elsewhere. `avoid` is the shipped wiring's own-cut-endpoint list.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Game, Move, MoveKind, Player, Point, Position};

/// Banking floor: closes below this never rank (AngelBot ~5.8 area/loop).
const MIN_CLOSE: f64 = 3.0;
/// A re-close on freshly popped ground must bank at least this much — small
/// re-inflations are the tempo trap, big ones are worth the actions.
const BIG_RECLOSE: f64 = 10.0;

pub fn best_move(position: &Position, avoid: &[Point]) -> Option<Move> {
    let me = position.to_move();
    let opp = me.opponent();
    let area_me = position.area(me).to_f64();
    let area_opp = position.area(opp).to_f64();

    // (ranking value, bank gain, the move) — closes, double-duty
    // (close + cut) ranked best.
    let mut best_close: Option<(f64, f64, Move)> = None;
    // (enemy area popped, the move) — real pops only, never a wasted cut.
    let mut best_cut: Option<(f64, Move)> = None;
    for mv in moves(position) {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        let popped = area_opp - after.area(opp).to_f64();
        if outcome.kind == MoveKind::Connect {
            let gain = after.area(me).to_f64() - area_me;
            if gain < MIN_CLOSE {
                continue;
            }
            let on_popped = touches(mv, avoid);
            if on_popped && gain < BIG_RECLOSE {
                continue; // the bank-sacrifice: let the popped loop stand
            }
            let value = gain + popped; // close+cut in one action, like AngelBot
            if best_close.is_none_or(|(v, _, _)| value > v) {
                best_close = Some((value, gain, mv));
            }
        } else if outcome.broken.is_some() && popped > 0.25 && best_cut.is_none_or(|(p, _)| popped > p) {
            best_cut = Some((popped, mv));
        }
    }
    // Bank first (score compounds every event); a pop bigger than the bank
    // is the one exception — never let an enemy balloon live.
    if let (Some((popped, cut)), Some(((_, gain, _))) ) = (best_cut, best_close) {
        if popped > gain {
            return Some(cut);
        }
    }
    if let Some((_, _, close)) = best_close {
        return Some(close);
    }
    if let Some((_, cut)) = best_cut {
        return Some(cut);
    }
    fallback(position, avoid)
}

/// Scout's wall extending, minus small closes and minus re-closes at
/// freshly popped ground (the bank-sacrifice rule holds in the fallback too).
fn fallback(position: &Position, avoid: &[Point]) -> Option<Move> {
    let me = position.to_move();
    let area_before = position.area(me).to_f64();
    let analysis = scoutbase::analyze(position, scoutbase::MOVE_BUDGET);
    for candidate in &analysis.candidates {
        if touches(candidate.mv, avoid) && is_small_close(position, candidate, me, area_before, BIG_RECLOSE) {
            continue;
        }
        if is_small_close(position, candidate, me, area_before, MIN_CLOSE) {
            continue;
        }
        return Some(candidate.mv);
    }
    analysis.candidates.first().map(|c| c.mv)
}

/// Whether the candidate's principal variation closes a loop worth less
/// than `floor` area.
fn is_small_close(position: &Position, candidate: &scoutbase::Candidate, me: Player, area_before: f64, floor: f64) -> bool {
    let mut after = position.clone();
    let mut closes = false;
    for &mv in &candidate.pv {
        let outcome = after.apply_unchecked(mv);
        closes |= outcome.kind == MoveKind::Connect;
    }
    let gain = after.area(me).to_f64() - area_before;
    closes && gain < floor
}

/// The mover's legal moves without the second listing of each own connection.
fn moves(position: &Position) -> Vec<Move> {
    position
        .legal_moves()
        .iter()
        .filter(|&mv| !repeats_a_connection(position, mv))
        .collect()
}

/// A connection between two of the mover's nodes is listed from both ends. This is the second.
fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

/// Whether the move touches a point where our edges were recently cut.
fn touches(mv: Move, avoid: &[Point]) -> bool {
    let Some(target) = mv.target() else { return false };
    avoid.contains(&mv.source) || avoid.contains(&target)
}

/// Smoke test: two games against Scout's search (one per color), tracing
/// own actions (number, kind, banked score so far, area).
fn main() {
    for g in 0..2 {
        let me_blue = g == 0;
        let me = if me_blue { Player::Blue } else { Player::Red };
        let mut game = Game::new();
        let mut avoid: Vec<Point> = Vec::new();
        let mut cuts: Vec<(u16, Player, meridian_engine::Edge)> = Vec::new();
        let mut trace = String::new();
        while !game.is_over() {
            let mover = game.position().to_move();
            let played = u16::from(game.position().actions_played());
            let own = (played + 2) / 2;
            let me_moves = mover == me;
            avoid.clear();
            if me_moves {
                for &(action, owner, cut) in cuts.iter().rev() {
                    if played - action > 6 {
                        break;
                    }
                    if owner == mover {
                        avoid.push(cut.origin());
                        avoid.push(cut.far());
                    }
                }
            }
            let mv = if me_moves {
                best_move(game.position(), &avoid)
            } else {
                scoutbase::best_move(game.position())
            };
            let outcome = game.play(mv.unwrap()).unwrap();
            if let Some(cut) = outcome.broken {
                cuts.push((played, mover.opponent(), cut));
            }
            if me_moves {
                let kind = match outcome.kind {
                    MoveKind::Connect => "C",
                    MoveKind::Capture => "X",
                    MoveKind::Extend => ".",
                };
                let area = game.position().area(me).to_f64();
                let bank = game.position().score(me).to_f64();
                trace.push_str(&format!("a{own}:{kind}({area:.0}/{bank:.0}) "));
            }
        }
        let bs = game.position().score(Player::Blue).to_f64();
        let rs = game.position().score(Player::Red).to_f64();
        println!("game {} me={}: blue={bs:.0} red={rs:.0}", g + 1, if me_blue { "blue" } else { "red" });
        println!("  trace: {trace}");
    }
}
