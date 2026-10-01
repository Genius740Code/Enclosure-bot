//! Lane D2 capybara fingerprint check: replay the capybara net v5 lines
//! (Lane G's `research/g-capybara.md`) against our deployed searches, and
//! record the divergence points — missed close? farmed re-close?
//!
//! Capybara side (deterministic, from Lane G):
//! - Opening book: the center mesh (Blue `D10-D13 D10-G8 G8-J10 J10-G13
//!   G8-J11 G13-J11 G13-D10 J11-M13`, Red the mirror) — "length-3 edges
//!   heading INWARD to the J files" (Lane G §4 item 4; the same mesh Lane H
//!   shipped as the v6 prefix).
//! - Continuation: the adaptive mid-weight steady banker — closes its best
//!   available loop every opportunity with NO floor (close gains
//!   `[1.25, 13.75, 1.25, 1.75, 1.25, 13.75…]`, Lane G §4), else extends
//!   walls with Scout's search (the ambient ~24 cuts/game ride along).
//! - THE ADAPTIVE FARM SWITCH (Lane G §5 + H2): when the opponent re-closes
//!   the same edge ≥ 2x (the farm-cycle detector, c-blunder-catalog §2.4),
//!   re-cut re-closed ground on priority — capy `b4dd04bf`: 41 cuts re-cutting
//!   exactly the edges riposte v4 re-closed ("identical move ids on both
//!   sides of the cycle").
//!
//! Our side: the deployed search (`v5base::best_move_with_avoid` — master's
//! current tip, the v8 stack, with live avoid; the untimed probe path) and
//! the v2+avoid arm (CUT_MEMORY 6). The v8 site entry (`best_move_routed`,
//! mesh8 prefix + timed think) is not run here — the timed think costs
//! 2-4.8s/move, ~1h per matchup; it is Lane B/X's next test.
//!
//! Divergence points recorded per game: our re-closes that got farmed
//! (popped by a capybara farm cut within 2 actions), our close cadence vs
//! capybara's (the missed-close question), and the center-mesh suppression
//! (capybara's largest loop / area trajectory).
//!
//! Usage: `cargo run --release --example probe_capyfingerprint -- [arms...]`
//! (default: all arms).

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v2base.rs"]
mod v2base;
#[path = "support/v5base.rs"]
mod v5base;

use std::cell::RefCell;

use meridian_engine::notation::parse_square;
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point, Position};

/// Reasonable Blue solos (only the starting edge's ENDPOINTS A10/D10 can
/// extend) — the same list `probe_match_d2` verified legal.
const SOLOS: [&str; 8] = [
    "D10-F11", "D10-C7", "A10-C11", "A10-D13", "D10-E12", "A10-B13", "D10-G10", "A10-C7",
];

/// Capybara's center mesh as Blue (Lane G §4 item 4).
const MESH_BLUE: [&str; 8] =
    ["D10-D13", "D10-G8", "G8-J10", "J10-G13", "G8-J11", "G13-J11", "G13-D10", "J11-M13"];
/// Red mirror (`P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8…`).
const MESH_RED: [&str; 8] =
    ["P10-M8", "M8-J10", "J10-M13", "M8-J11", "M13-J11", "M13-P11", "M13-P10", "P11-M8"];

thread_local! {
    /// Capybara's farm-cycle detector state: the edges it has cut, and how
    /// many times the opponent re-closed each of them. Cleared on a new game.
    static DETECTOR: RefCell<Detector> = RefCell::new(Detector::default());
}

#[derive(Default)]
struct Detector {
    /// The action count at capybara's last call, to detect a new game.
    prev_seen: Option<u16>,
    /// The opponent's edges at capybara's last call (to diff both sides).
    prev_opp: Vec<Edge>,
    /// Every edge capybara has cut (its farm ledger, no decay).
    cut: Vec<Edge>,
    /// Per-edge count of the opponent's re-closes of capybara's cut ground.
    recloses: Vec<(Edge, u32)>,
    /// The edges capybara cut in FARM mode (the switch on), with the action
    /// index — the divergence check distinguishes these from ambient cuts.
    farm_hits: Vec<(Edge, u16)>,
}

/// Updates the detector with capybara's cuts and the opponent's re-closes
/// since its last call.
fn note_state(position: &Position, me: Player) {
    DETECTOR.with(|state| {
        let mut state = state.borrow_mut();
        let played = u16::from(position.actions_played());
        if state.prev_seen.is_some_and(|p| played < p) {
            state.cut.clear();
            state.recloses.clear();
            state.prev_opp.clear();
            state.farm_hits.clear();
        }
        state.prev_seen = Some(played);
        let now: Vec<Edge> = position.edges(me.opponent()).iter().collect();
        let prev = std::mem::take(&mut state.prev_opp);
        for edge in prev.iter() {
            if !now.contains(edge) && !state.cut.contains(edge) {
                // The enemy never cuts its own edges: a disappeared one was
                // capybara's cut.
                state.cut.push(*edge);
            }
        }
        for edge in now.iter() {
            if state.cut.contains(edge) && !prev.contains(edge) {
                // The rebuilder rebuilt capybara's cut ground: count it.
                match state.recloses.iter_mut().find(|(e, _)| e == edge) {
                    Some((_, n)) => *n += 1,
                    None => state.recloses.push((*edge, 1)),
                }
            }
        }
        state.prev_opp = now;
    })
}

/// Whether the farm switch is ON: the opponent re-closed the same edge ≥ 2x.
fn farm_mode(position: &Position, me: Player) -> bool {
    DETECTOR.with(|state| {
        let state = state.borrow();
        let now: Vec<Edge> = position.edges(me.opponent()).iter().collect();
        state
            .recloses
            .iter()
            .any(|(edge, n)| *n >= 2 && now.contains(edge))
    })
}

/// Capybara's move: farm re-cut when the switch is on, else the fattest
/// close (no floor), else the mesh book while it lasts, else Scout's search.
fn capy_move(position: &Position) -> Option<Move> {
    let me = position.to_move();
    note_state(position, me);
    // 1. The adaptive farm switch: re-cut the re-closed ground (the fattest
    //    farm-ground cut; ties by move index).
    if farm_mode(position, me) {
        if let Some(mv) = farm_cut(position, me) {
            return Some(mv);
        }
    }
    // 2. The fattest available close, no floor (capy re-closes tiny: gains
    //    [1.25, 13.75, ...]).
    let opp = me.opponent();
    let area_me = position.area(me).to_f64();
    let mut best_close: Option<(f64, Move)> = None;
    for mv in moves(position) {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        if outcome.kind == MoveKind::Connect {
            let gain = after.area(me).to_f64() - area_me;
            if gain > 0.0 && best_close.is_none_or(|(g, _)| gain > g) {
                best_close = Some((gain, mv));
            }
        }
    }
    if let Some((_, mv)) = best_close {
        return Some(mv);
    }
    // 3. The center-mesh book while it lasts (the durable web).
    let mesh: &[&str] = if me == Player::Blue { &MESH_BLUE } else { &MESH_RED };
    for text in mesh {
        let Some((mv, edge)) = parse_book_edge(text) else { continue };
        if position.edges(me).contains(edge) {
            continue;
        }
        if position.legal_moves().contains(mv) {
            return Some(mv);
        }
    }
    // 4. Extend walls with Scout's search (ambient cuts ride along).
    scoutbase::best_move(position)
}

/// The fattest legal cut that re-cuts capybara's cut ground. Ties by move
/// index. Records the farm hit (edge + action) for the divergence check.
fn farm_cut(position: &Position, me: Player) -> Option<Move> {
    let ledger: Vec<Edge> = DETECTOR.with(|state| state.borrow().cut.clone());
    if ledger.is_empty() {
        return None;
    }
    let opp = me.opponent();
    let mut best: Option<(f64, Move, Edge)> = None;
    for mv in position.legal_moves().iter() {
        let mut after = position.clone();
        let outcome = after.apply_unchecked(mv);
        let Some(broken) = outcome.broken else { continue };
        if !ledger.contains(&broken) {
            continue;
        }
        let popped = position.area(opp).to_f64() - after.area(opp).to_f64();
        if best.is_none_or(|(p, _, _)| popped > p) {
            best = Some((popped, mv, broken));
        }
    }
    if let Some((_, mv, broken)) = best {
        DETECTOR.with(|state| {
            state
                .borrow_mut()
                .farm_hits
                .push((broken, u16::from(position.actions_played())));
        });
        return Some(mv);
    }
    None
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

/// "D10-D13" -> (the move, the edge it places).
fn parse_book_edge(text: &str) -> Option<(Move, Edge)> {
    let (from, to) = text.split_once('-')?;
    let a = parse_square(from)?;
    let b = parse_square(to)?;
    Some((Move::between(a, b)?, Edge::between(a, b)?))
}

/// The endpoints of `mover`'s edges cut in the last [`CUT_MEMORY`] actions:
/// the shipped anti-rebuild wiring.
fn avoid_points(cuts: &[(u16, Player, Edge)], mover: Player, played: u16) -> Vec<Point> {
    let mut avoid = Vec::new();
    for &(action, owner, cut) in cuts.iter().rev() {
        if played - action > v2base::CUT_MEMORY as u16 {
            break;
        }
        if owner == mover {
            avoid.push(cut.origin());
            avoid.push(cut.far());
        }
    }
    avoid
}

#[derive(Default)]
struct Side {
    closes: u32,
    cuts: u32,
    first_close: Option<u16>,
    /// Re-closes of capybara's cut ground (ours only).
    recloses: u32,
    /// Re-closes popped by a capybara farm cut within 2 actions (ours only).
    farmed: u32,
}

/// One game: capybara vs one of our arms. Returns the game plus both sides'
/// stats and the action numbers of the divergence points.
fn play(base: &dyn Fn(&Position, &[Point], &[Move]) -> Option<Move>, solo: &str, capy_blue: bool) -> (f64, f64, Side, Side, Vec<u16>, Vec<u16>, f64) {
    let mut game = Game::new();
    let mut history: Vec<Move> = Vec::new();
    let (mut base_side, mut capy_side) = (Side::default(), Side::default());
    let (mut farmed_acts, mut reclose_acts) = (Vec::new(), Vec::new());
    let (mut cuts, mut reclose_t) = (Vec::new(), Vec::new());
    let mut first = true;
    let mut largest_capy: f64 = 0.0;
    while !game.is_over() {
        let mover = game.position().to_move();
        let played = u16::from(game.position().actions_played());
        let own = (played + 2) / 2;
        let base_moves = (mover == Player::Blue) != capy_blue;
        let avoid = avoid_points(&cuts, mover, played);
        let mv = if first {
            first = false;
            let (from, to) = solo.split_once('-').unwrap();
            let solo = Move::between(parse_square(from).unwrap(), parse_square(to).unwrap()).unwrap();
            if game.legal_moves().contains(solo) { Some(solo) } else { game.legal_moves().iter().next() }
        } else if base_moves {
            base(game.position(), &avoid, &history)
        } else {
            capy_move(game.position())
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        history.push(mv.unwrap());
        if let Some(cut) = outcome.broken {
            cuts.push((played, mover.opponent(), cut));
        }
        let side = if base_moves { &mut base_side } else { &mut capy_side };
        if outcome.kind == MoveKind::Connect {
            side.closes += 1;
            if side.first_close.is_none() {
                side.first_close = Some(own);
            }
        }
        if outcome.broken.is_some() {
            side.cuts += 1;
        }
        if base_moves {
            // Did our move re-close capybara's cut ground? (The placed edge
            // is back on farm ground.)
            let ledger: Vec<Edge> =
                DETECTOR.with(|state| state.borrow().cut.clone());
            if ledger.contains(&outcome.placed) {
                base_side.recloses += 1;
                reclose_acts.push(own);
                reclose_t.push((outcome.placed, own));
            }
        } else {
            let capy_area = game.position().area(mover).to_f64();
            if capy_area > largest_capy {
                largest_capy = capy_area;
            }
            // Did capybara's farm cut pop a re-close of ours within 2 actions?
            if let Some(broken) = outcome.broken {
                if farm_was_cut(&broken) {
                    for &(edge, act) in reclose_t.iter().rev() {
                        if own - act > 2 {
                            break;
                        }
                        if edge == broken {
                            base_side.farmed += 1;
                            farmed_acts.push(act);
                            break;
                        }
                    }
                }
            }
        }
    }
    let blue = game.position().score(Player::Blue).to_f64();
    let red = game.position().score(Player::Red).to_f64();
    let (base_score, capy_score) = if capy_blue { (red, blue) } else { (blue, red) };
    (base_score - capy_score, capy_score, base_side, capy_side, reclose_acts, farmed_acts, largest_capy)
}

/// Whether `broken` is an edge capybara cut in FARM mode (the switch on) —
/// the farm hits are recorded in the detector; ambient cuts are not.
fn farm_was_cut(broken: &Edge) -> bool {
    DETECTOR.with(|state| state.borrow().farm_hits.iter().any(|(edge, _)| edge == broken))
}

fn main() {
    let mut arms: Vec<(&str, &dyn Fn(&Position, &[Point], &[Move]) -> Option<Move>)> = vec![
        ("v8", &|pos, avoid, _| v5base::best_move_with_avoid(pos, avoid)),
        ("v2a", &|pos, avoid, _| v2base::best_move_with_avoid(pos, avoid)),
    ];
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        arms.retain(|(name, _)| args.iter().any(|a| a == name));
    }
    for (arm_name, arm) in arms {
        let mut margins = Vec::new();
        for g in 0..8 {
            let capy_blue = g < 4;
            let (margin, capy_score, base, capy, recloses, farmed, largest) =
                play(arm, SOLOS[g], capy_blue);
            println!(
                "{arm_name} vs capy game {} capy={}: margin={:+.0} base[closes={} cuts={} first={} reclose={} farmed={}] capy[closes={} cuts={} first={} largest={:.0}]",
                g + 1,
                if capy_blue { "blue" } else { "red" },
                margin,
                base.closes,
                base.cuts,
                base.first_close.map_or("-".into(), |a| a.to_string()),
                base.recloses,
                base.farmed,
                capy.closes,
                capy.cuts,
                capy.first_close.map_or("-".into(), |a| a.to_string()),
                largest,
            );
            if !farmed.is_empty() {
                println!("  farmed re-closes at own actions {farmed:?}");
            }
            margins.push(margin);
        }
        let wins = margins.iter().filter(|m| **m > 0.0).count();
        let draws = margins.iter().filter(|m| **m == 0.0).count();
        let avg = margins.iter().sum::<f64>() / margins.len() as f64;
        println!(
            "== {arm_name} vs capy: W-L {wins}-{} D{draws} avg_margin={avg:+.0}",
            8 - draws - wins,
        );
    }
}
