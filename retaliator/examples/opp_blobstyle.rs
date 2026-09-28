//! Lane D2 style-mimic: the mega-blob swallow — xmybot / Great Barrier /
//! Atlas v2, C2 autopsy S1 (`research/c-site-losses-2.md` §4.1 on
//! `origin/lane-c-autopsy`).
//!
//! Signature: area stays ~0 for 40+ actions while one giant loop is drawn
//! far from the enemy lines (an open path encloses nothing), no small loop
//! is ever closed, then ONE giant Connect banks 100+ area and the hold
//! phase repairs the boundary forever — score compounds at every scoring
//! event while the enemy's cuts arrive a turn late (rule 6 shields the
//! cutting edge). Growth rings afterwards widen the loop (xmybot 180 -> 257
//! between acts 60-80).
//!
//! Blue's ring walls off the far west band (x in [-9,-3], 108 area), Red's
//! the far east mirror; ring2/3 push the far wall to the midline and across
//! (162 / 216) once the previous ring is closed and banking.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::notation::parse_square;
use meridian_engine::{Edge, Move, MoveKind, Player, Position};

/// Blue ring 1: the far west band (x in [-9,-3], 108 area — GB's site loop
/// size), drawn 9+ from Red's start, beyond a 2-ply bot's routing radius.
const BLUE_RING: [&str; 16] = [
    "A10-A13", "A13-A16", "A16-A19", "A19-D19", "D19-G19", "G19-G16", "G16-G13", "G13-G10",
    "G10-G7", "G7-G4", "G4-G1", "G1-D1", "D1-A1", "A1-A4", "A4-A7", "A7-A10",
];
/// Red ring 1: the far east band mirror (x in [3,9]).
const RED_RING: [&str; 16] = [
    "S10-S13", "S13-S16", "S16-S19", "S19-P19", "P19-M19", "M19-M16", "M16-M13", "M13-M10",
    "M10-M7", "M7-M4", "M4-M1", "M1-P1", "P1-S1", "S1-S4", "S4-S7", "S7-S10",
];
/// Blue partner wall: a parallel polyline one step OUTSIDE the enemy-facing
/// G wall and along the north/south row ends. Any edge cutting the wall then
/// touches two of ours (illegal), and a lone partner cut drops nothing —
/// the invincible-thicket hold of the site blobs (C2 §4.2a).
const BLUE_PARTNER: [&str; 15] = [
    "G19-H18", "H18-H15", "H15-H12", "H12-H9", "H9-H6", "H6-H3", "H3-G1", "G19-F18", "F18-D18",
    "D18-B18", "B18-A16", "G1-F2", "F2-D2", "D2-B2", "B2-A4",
];
/// Red partner wall mirror (outside the enemy-facing M wall, L column).
const RED_PARTNER: [&str; 15] = [
    "M19-L18", "L18-L15", "L15-L12", "L12-L9", "L9-L6", "L6-L3", "L3-M1", "M19-N18", "N18-P18",
    "P18-R18", "R18-S16", "M1-N2", "N2-P2", "P2-R2", "R2-S4",
];
/// Blue ring 2: far wall to x=0 (162); ring 3: to x=3 (216).
const BLUE_RING2: [&str; 8] =
    ["G19-J19", "J19-J16", "J16-J13", "J13-J10", "J10-J7", "J7-J4", "J4-J1", "J1-G1"];
const BLUE_RING3: [&str; 8] =
    ["J19-M19", "M19-M16", "M16-M13", "M13-M10", "M10-M7", "M7-M4", "M4-M1", "M1-J1"];
const RED_RING2: [&str; 8] =
    ["M19-J19", "J19-J16", "J16-J13", "J13-J10", "J10-J7", "J7-J4", "J4-J1", "J1-M1"];
const RED_RING3: [&str; 8] =
    ["J19-G19", "G19-G16", "G16-G13", "G13-G10", "G10-G7", "G7-G4", "G4-G1", "G1-J1"];
/// The style never closes small; the fallback floor is a big-loop-only floor.
const MIN_CLOSE: f64 = 25.0;

/// Growth gates: each further layer is only drawn while the previous one is
/// closed and banking (ring 1 = 108; +partner strip ~124; ring 2 = 162+;
/// ring 3 = 216+), so a contested mimic spends its actions on the ring war,
/// not on dead growth.
const GATE_PARTNER: f64 = 80.0;
const GATE_RING2: f64 = 100.0;
const GATE_RING3: f64 = 170.0;

pub fn best_move(position: &Position) -> Option<Move> {
    let me = position.to_move();
    let area = position.area(me).to_f64();
    let mut rings: [&[&str]; 4] = match me {
        Player::Blue => [&BLUE_RING, &BLUE_PARTNER, &BLUE_RING2, &BLUE_RING3],
        Player::Red => [&RED_RING, &RED_PARTNER, &RED_RING2, &RED_RING3],
    };
    if area < GATE_RING3 {
        rings[3] = &[];
    }
    if area < GATE_RING2 {
        rings[2] = &[];
    }
    if area < GATE_PARTNER {
        rings[1] = &[];
    }
    // Draw / repair / grow: the first ring edge not owned yet that is legal
    // now. Owned edges are skipped (done), blocked ones (enemy anchor or
    // fresh cut edge) get an anchor-clearing cut, else are retried later.
    for ring in rings {
        for text in ring {
            let Some((mv, edge)) = parse_edge(text) else { continue };
            if position.edges(me).contains(edge) {
                continue;
            }
            if position.legal_moves().contains(mv) {
                return Some(mv);
            }
            if let Some(clear) = clearing_move(position, mv, me) {
                return Some(clear);
            }
        }
    }
    fallback(position, me)
}

/// A blocked slot is usually an enemy node anchored on the line with two
/// edges (the slot then touches 2+ opponent edges and cannot be re-placed).
/// Cutting one enemy edge with an endpoint on or beside the slot line opens
/// it for the next action. The popper of the most enemy area wins ties.
fn clearing_move(position: &Position, slot: Move, me: Player) -> Option<Move> {
    let opp = me.opponent();
    let target = slot.target()?;
    let edge = Edge::between(slot.source, target)?;
    let mut line = vec![slot.source, target];
    line.extend(edge.interior_points());
    let near = |p: meridian_engine::Point| {
        line.iter().any(|q| (p.x() - q.x()).abs() <= 1 && (p.y() - q.y()).abs() <= 1)
    };
    let mut best: Option<(f64, Move)> = None;
    for mv in position.legal_moves().iter() {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        let Some(broken) = outcome.broken else { continue };
        let (a, b) = broken.endpoints();
        if !near(a) && !near(b) {
            continue;
        }
        let popped = position.area(opp).to_f64() - after.area(opp).to_f64();
        if best.is_none_or(|(p, _)| popped > p) {
            best = Some((popped, mv));
        }
    }
    best.map(|(_, mv)| mv)
}

/// Scout's wall extending, minus any close under [`MIN_CLOSE`] area: the
/// style banks nothing but the giant loop. If every candidate closes small,
/// take the smallest close (least damage).
fn fallback(position: &Position, me: Player) -> Option<Move> {
    let area_before = position.area(me).to_f64();
    let analysis = scoutbase::analyze(position, scoutbase::MOVE_BUDGET);
    let mut least_bad: Option<(f64, Move)> = None;
    for candidate in &analysis.candidates {
        if is_small_close(position, candidate, me, area_before) {
            let mut after = position.clone();
            let gain = (|| {
                for &mv in &candidate.pv {
                    after.apply_unchecked(mv);
                }
                after.area(me).to_f64() - area_before
            })();
            if least_bad.is_none_or(|(g, _)| gain < g) {
                least_bad = Some((gain, candidate.mv));
            }
            continue;
        }
        return Some(candidate.mv);
    }
    least_bad.map(|(_, mv)| mv)
}

/// Whether the candidate's principal variation closes a loop worth less
/// than [`MIN_CLOSE`] area (the tiny-close habit this style refuses).
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

/// "A10-A13" -> (the move, the edge it places).
fn parse_edge(text: &str) -> Option<(Move, Edge)> {
    let (from, to) = text.split_once('-')?;
    let a = parse_square(from)?;
    let b = parse_square(to)?;
    Some((Move::between(a, b)?, Edge::between(a, b)?))
}

/// Smoke test: one game against Scout's search with each color, tracing
/// this style's own actions (number, kind, area after) and the ring state.
fn main() {
    for g in 0..2 {
        let me_blue = g == 0;
        let mut game = meridian_engine::Game::new();
        let mut trace = String::new();
        while !game.is_over() {
            let me_moves = (game.position().to_move() == Player::Blue) == me_blue;
            let own = (u16::from(game.position().actions_played()) + 2) / 2;
            let me = if me_blue { Player::Blue } else { Player::Red };
            let rings: [&[&str]; 4] = if me_blue {
                [&BLUE_RING, &BLUE_PARTNER, &BLUE_RING2, &BLUE_RING3]
            } else {
                [&RED_RING, &RED_PARTNER, &RED_RING2, &RED_RING3]
            };
            let missing: Vec<usize> = rings
                .iter()
                .map(|ring| {
                    ring.iter()
                        .filter(|text| {
                            parse_edge(text).is_none_or(|(_, e)| !game.position().edges(me).contains(e))
                        })
                        .count()
                })
                .collect();
            let mv = if me_moves {
                best_move(game.position())
            } else {
                scoutbase::best_move(game.position())
            };
            let text = format!("{:?}-{:?}", mv.unwrap().source, mv.unwrap().target());
            let outcome = game.play(mv.unwrap()).unwrap();
            let tag = if me_moves { "me" } else { "sc" };
            let area = game.position().area(me).to_f64();
            let kind = match outcome.kind {
                MoveKind::Connect => "C",
                MoveKind::Capture => "X",
                MoveKind::Extend => ".",
            };
            let cut = if outcome.broken.is_some() { "cut!" } else { "" };
            trace.push_str(&format!("\n{tag} a{own} {text} {kind}{cut} area={area:.0} miss={missing:?}"));
        }
        let bs = game.position().score(Player::Blue).to_f64();
        let rs = game.position().score(Player::Red).to_f64();
        println!("game {} me={}: blue={bs:.0} red={rs:.0}\n{trace}", g + 1, if me_blue { "blue" } else { "red" });
    }
}
