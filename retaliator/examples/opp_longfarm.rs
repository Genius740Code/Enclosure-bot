//! Lane D2 style-mimic: the long farm — 20-40-action farm cycles that
//! outlast CUT_MEMORY 6 (c-blunder-catalog §4 H1 + §3.2; capybara
//! `b4dd04bf`'s fingerprint in `research/g-capybara.md` §4).
//!
//! Site signature: the farmer's own web is durable (VladNet area 48-51.8
//! from act 40, alive all game; capy's center mesh: length-3 edges heading
//! INWARD to the J files), and against a rebuilder it farms 30-41 cuts/game
//! re-cutting EXACTLY the edges the rebuilder re-closes — "identical move
//! ids on both sides of the cycle: the rebuilder rebuilds, capy re-cuts the
//! same ground". Pop ids are reused across 20-40-action spans (17112 x4,
//! 12114 x3, 2707 x3, 452 x3) — far beyond the deployed CUT_MEMORY 6, so
//! the avoid set forgets the cycle and our re-closes get popped (C2:
//! "the avoid set forgets cycles that run for dozens of actions, and
//! re-closes on known farm ground still outrank fresh ground").
//!
//! ONE variable vs the Vlad farmer (`opp_vladstyle.rs`): **the no-decay
//! farm ledger** — every edge the mimic has cut is remembered forever, and
//! the moment the opponent re-closes any of them (it is back in their
//! edges), the mimic re-cuts that same ground on priority. The cycle length
//! is whatever the rebuilder's re-close cadence makes it (12-24 actions on
//! site); the mimic's half is always prompt.
//!
//! `avoid` is the shipped wiring's own-cut-endpoint list (CUT_MEMORY 6):
//! the site farmer rebuilds rarely (0-2 repeated closes/game, capy §8), so
//! a re-close on freshly popped ground is refused unless it banks ≥ 10 —
//! the same rule the sac-style applies to its own sacrifice. The farm
//! re-cuts themselves are untouched by it (the cycle's whole point).

#[path = "support/scoutbase.rs"]
mod scoutbase;

use std::cell::RefCell;

use meridian_engine::notation::parse_square;
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point, Position};

/// A re-close on freshly popped ground must bank at least this much —
/// the site farmer rebuilds rarely (0-2 repeated closes per game).
const BIG_RECLOSE: f64 = 10.0;

thread_local! {
    /// The farm ledger: every edge the mimic has ever cut (no decay), tracked
    /// by diffing the opponent's edges between the mimic's turns — the enemy
    /// never cuts its own edges, so a disappeared enemy edge was the mimic's
    /// cut. Cleared on a new game (the action count going backwards).
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
}

#[derive(Default)]
struct Ledger {
    /// The action count at the mimic's last call, to detect a new game.
    prev_seen: Option<u16>,
    /// The opponent's edges at the mimic's last call (to diff its cuts).
    prev_opp: Vec<Edge>,
    /// Every edge the mimic has cut, oldest first. No decay.
    farm: Vec<Edge>,
}

/// Updates the ledger with the mimic's own cuts since its last call, and
/// returns the full farm ledger.
fn note_cuts(position: &Position, me: Player) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        let played = u16::from(position.actions_played());
        if ledger.prev_seen.is_some_and(|p| played < p) {
            ledger.farm.clear();
            ledger.prev_opp.clear();
        }
        ledger.prev_seen = Some(played);
        let now: Vec<Edge> = position.edges(me.opponent()).iter().collect();
        let prev = std::mem::take(&mut ledger.prev_opp);
        for edge in prev.iter() {
            if !now.contains(edge) && !ledger.farm.contains(edge) {
                ledger.farm.push(*edge);
            }
        }
        ledger.prev_opp = now;
    })
}

/// The opponent's current edges that sit on farm ground (the mimic cut them
/// before and the rebuilder re-closed them).
fn farm_targets(position: &Position, me: Player) -> Vec<Edge> {
    LEDGER.with(|ledger| {
        let ledger = ledger.borrow();
        position
            .edges(me.opponent())
            .iter()
            .filter(|edge| ledger.farm.contains(edge))
            .collect()
    })
}

/// Blue's center mesh (Lane G: `D10-D13 D10-G8 G8-J10 J10-G13 G8-J11
/// G13-J11 G13-D10 J11-M13…`) — length-3 edges heading inward to the J
/// files, the durable web the farmer banks from.
const BLUE_MESH: [&str; 8] =
    ["D10-D13", "D10-G8", "G8-J10", "J10-G13", "G8-J11", "G13-J11", "G13-D10", "J11-M13"];
/// Red's center mesh mirror (`P10-M8 M8-J10 J10-M13 M8-J11 M13-J11
/// M13-P11 M13-P10 P11-M8…`).
const RED_MESH: [&str; 8] =
    ["P10-M8", "M8-J10", "J10-M13", "M8-J11", "M13-J11", "M13-P11", "M13-P10", "P11-M8"];

pub fn best_move(position: &Position, avoid: &[Point]) -> Option<Move> {
    let me = position.to_move();
    note_cuts(position, me);
    // 1. THE FARM: the rebuilder rebuilt, the farmer re-cuts the same
    //    ground. The fattest farm-ground cut; ties by move index.
    if let Some(mv) = farm_cut(position, me) {
        return Some(mv);
    }
    // 2. Build: the center-mesh book (the durable web, first not-owned legal
    //    edge; blocked slots are retried later).
    let mesh: &[&str] = if me == Player::Blue { &BLUE_MESH } else { &RED_MESH };
    for text in mesh {
        let Some((mv, edge)) = parse_book_edge(text) else { continue };
        if position.edges(me).contains(edge) {
            continue;
        }
        if position.legal_moves().contains(mv) {
            return Some(mv);
        }
    }
    // 3. Max-pop cut: the cut destroying the most enemy area (Vlad's rule).
    let opp = me.opponent();
    let area_opp = position.area(opp).to_f64();
    let mut best_cut: Option<(f64, Move)> = None;
    // 4. Max-gain close: banks the web (48-51 from act 40 on site), minus
    //    re-closes on freshly popped ground unless they bank big (the site
    //    farmer rebuilds rarely).
    let area_me = position.area(me).to_f64();
    let mut best_close: Option<(f64, Move)> = None;
    for mv in moves(position) {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        if outcome.broken.is_some() {
            let popped = area_opp - after.area(opp).to_f64();
            if best_cut.is_none_or(|(p, _)| popped > p) {
                best_cut = Some((popped, mv));
            }
        } else if outcome.kind == MoveKind::Connect {
            let gain = after.area(me).to_f64() - area_me;
            if gain > 0.0 && touches_popped(mv, avoid) && gain < BIG_RECLOSE {
                continue; // freshly popped ground: let it stand (rare rebuild)
            }
            if gain > 0.0 && best_close.is_none_or(|(g, _)| gain > g) {
                best_close = Some((gain, mv));
            }
        }
    }
    if let Some((_, mv)) = best_cut {
        return Some(mv);
    }
    if let Some((_, mv)) = best_close {
        return Some(mv);
    }
    // 5. No cut, no close: extend walls with Scout's search.
    scoutbase::best_move(position)
}

/// The fattest legal cut that re-cuts farm ground (the cycle: identical move
/// ids on both sides). Ties by move index.
fn farm_cut(position: &Position, me: Player) -> Option<Move> {
    let targets = farm_targets(position, me);
    if targets.is_empty() {
        return None;
    }
    let opp = me.opponent();
    let mut best: Option<(f64, Move)> = None;
    for mv in position.legal_moves().iter() {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        let Some(broken) = outcome.broken else { continue };
        if !targets.contains(&broken) {
            continue;
        }
        let popped = position.area(opp).to_f64() - after.area(opp).to_f64();
        if best.is_none_or(|(p, _)| popped > p) {
            best = Some((popped, mv));
        }
    }
    best.map(|(_, mv)| mv)
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

/// Whether the move lands on a point where the mover's own edges were
/// recently cut (the shipped wiring's avoid list).
fn touches_popped(mv: Move, avoid: &[Point]) -> bool {
    let Some(target) = mv.target() else { return false };
    avoid.contains(&mv.source) || avoid.contains(&target)
}

/// "D10-D13" -> (the move, the edge it places).
fn parse_book_edge(text: &str) -> Option<(Move, Edge)> {
    let (from, to) = text.split_once('-')?;
    let a = parse_square(from)?;
    let b = parse_square(to)?;
    Some((Move::between(a, b)?, Edge::between(a, b)?))
}

/// Smoke test: one game against Scout's search with each color, tracing this
/// style's own actions (number, kind, area after), the farm re-cuts, and the
/// cycle spans (first cut of a farm edge -> last re-cut, in own actions).
fn main() {
    for g in 0..2 {
        let me_blue = g == 0;
        let me = if me_blue { Player::Blue } else { Player::Red };
        let mut game = Game::new();
        let mut trace = String::new();
        let mut first_cut: std::collections::HashMap<Edge, u16> = std::collections::HashMap::new();
        let mut last_cut: std::collections::HashMap<Edge, u16> = std::collections::HashMap::new();
        let mut farm_cuts = 0usize;
        // (action index, owner of the broken edge, the broken edge) — the
        // shipped wiring's cut history, replayed for the avoid list.
        let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
        while !game.is_over() {
            let mover = game.position().to_move();
            let played = u16::from(game.position().actions_played());
            let me_moves = (mover == Player::Blue) == me_blue;
            let own = (played + 2) / 2;
            let farm_before = if me_moves { Some(farm_targets(game.position(), me)) } else { None };
            let avoid = if me_moves {
                let mut avoid = Vec::new();
                for &(action, owner, cut) in cuts.iter().rev() {
                    if played - action > 6 {
                        break;
                    }
                    if owner == mover {
                        avoid.push(cut.origin());
                        avoid.push(cut.far());
                    }
                }
                avoid
            } else {
                Vec::new()
            };
            let mv = if me_moves {
                best_move(game.position(), &avoid)
            } else {
                scoutbase::best_move(game.position())
            };
            let text = format!("{:?}-{:?}", mv.unwrap().source, mv.unwrap().target());
            let outcome = game.play(mv.unwrap()).unwrap();
            if let Some(cut) = outcome.broken {
                cuts.push((played, mover.opponent(), cut));
            }
            if !me_moves {
                continue;
            }
            let kind = match outcome.kind {
                MoveKind::Connect => "C",
                MoveKind::Capture => "X",
                MoveKind::Extend => ".",
            };
            let cut = if outcome.broken.is_some() { "cut!" } else { "" };
            let farm = farm_before.is_some_and(|targets| {
                outcome.broken.is_some_and(|broken| targets.contains(&broken))
            });
            if farm {
                farm_cuts += 1;
                if let Some(broken) = outcome.broken {
                    first_cut.entry(broken).or_insert(own);
                    last_cut.insert(broken, own);
                }
            }
            let area = game.position().area(me).to_f64();
            let tag = if farm { " FARM" } else { "" };
            trace.push_str(&format!("\nme a{own} {text} {kind}{cut} area={area:.0}{tag}"));
        }
        let mut spans: Vec<String> = first_cut
            .iter()
            .map(|(edge, first)| {
                let last = last_cut[edge];
                format!("{edge:?} a{first}->a{last}")
            })
            .collect();
        spans.sort();
        let bs = game.position().score(Player::Blue).to_f64();
        let rs = game.position().score(Player::Red).to_f64();
        println!(
            "game {} me={}: blue={bs:.0} red={rs:.0} farm_cuts={farm_cuts}\ncycle spans: {}\n{trace}",
            g + 1,
            if me_blue { "blue" } else { "red" },
            spans.join(", ")
        );
    }
}
