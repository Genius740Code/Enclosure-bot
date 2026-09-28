//! Lane O (v7) step 2: DEDICATED CONTEST move generation vs the COLLAPSER.
//!
//! Not an eval tweak: generates the explicit contest set against enemy
//! big-close construction sites and force-picks on FULL-HORIZON merit
//! (BONUS=0). If the search already prices these right, the contest set
//! never wins the merit test -> KILL with numbers.
//!
//! Contest classes (one variable = the generation, shared merit test):
//! - CUT_SITE: our move cuts a foe edge adjacent (Chebyshev <=2) to foe
//!   fresh/shielded wall = cutting at the corridor root / 2nd-wall joint.
//! - OCCUPY: non-breaking move landing within 2 of foe fresh/shielded wall
//!   = occupying the pre-build site (exactly what FRESH_PENALTY forbids).
//!
//! Metrics per game: W/L + margin (tempo cost), # contest picks by class,
//! foe max area, BIG_CLOSE flag (foe area >= 25 at any snapshot).
//! Conversion = share of games with no BIG_CLOSE.

mod opp_collapser;

use meridian_engine::notation::parse_square;
use meridian_engine::{Game, Move, Player, Point, Position};

const SOLOS: [&str; 8] = [
    "D10-F11", "D10-C7", "A10-C11", "A10-D13", "D10-E12", "A10-B13", "D10-G10", "A10-C7",
];
const SITE_DIST: i8 = 2;
const BIG_CLOSE: f64 = 25.0;

fn solo_move(game: &Game, text: &str) -> Option<Move> {
    let (from, to) = text.split_once('-')?;
    let mv = Move::between(parse_square(from)?, parse_square(to)?)?;
    if game.legal_moves().contains(mv) {
        Some(mv)
    } else {
        game.legal_moves().iter().next()
    }
}

fn near(a: Point, b: Point, dist: i8) -> bool {
    (a.x() - b.x()).abs() <= dist && (a.y() - b.y()).abs() <= dist
}

/// Fresh/shielded wall endpoints (foe construction sites). Mirrors the
/// search's `near_fresh_enemy` approximation (no owner filter available).
fn site_points(pos: &Position) -> Vec<Point> {
    let mut pts = Vec::new();
    for edge in pos.shielded_edges().chain(pos.fresh_edges()) {
        pts.push(edge.origin());
        pts.push(edge.far());
    }
    pts
}

/// Full-horizon mover-relative points of `after` (uncapped events).
fn full_value(before: &Position, after: &Position, mover: Player) -> f64 {
    let ev = f64::from(after.scoring_events_left());
    let lead = |p: &Position| {
        (p.score(Player::Blue).to_f64() - p.score(Player::Red).to_f64())
            + (p.area(Player::Blue).to_f64() - p.area(Player::Red).to_f64()) * ev
    };
    let s = if mover == Player::Blue { 1.0 } else { -1.0 };
    s * (lead(after) - lead(before))
}

#[derive(Clone, Copy, PartialEq)]
enum ContestClass {
    CutSite,
    Occupy,
}

/// Baseline pick + explicit contest set; contest wins only on full-horizon
/// merit (BONUS=0). Returns (move, contest class if contest won).
fn contest_best_move(pos: &Position) -> (Option<Move>, Option<ContestClass>) {
    let mover = pos.to_move();
    let baseline = retaliator::search::best_move(pos);
    let mut base_val = f64::NEG_INFINITY;
    if let Some(mv) = baseline {
        let mut after = pos.clone();
        after.apply_unchecked(mv);
        base_val = full_value(pos, &after, mover);
    }
    let sites = site_points(pos);
    let mut best: Option<(Move, ContestClass, f64)> = None;
    for mv in pos.legal_moves().iter() {
        let target = match mv.target() {
            Some(t) => t,
            None => continue,
        };
        if !sites.iter().any(|&s| near(s, target, SITE_DIST)) {
            continue;
        }
        let mut after = pos.clone();
        let outcome = after.apply_unchecked(mv);
        let class = if outcome.broken.is_some() {
            ContestClass::CutSite
        } else {
            ContestClass::Occupy
        };
        let v = full_value(pos, &after, mover);
        if v >= base_val && best.map_or(true, |(_, _, bv)| v > bv) {
            best = Some((mv, class, v));
        }
    }
    match best {
        Some((mv, class, _)) => (Some(mv), Some(class)),
        None => (baseline, None),
    }
}

fn play(solo: &str, ret_blue: bool, contest: bool) -> (f64, f64, u32, u32, u32, f64, bool) {
    let mut game = Game::new();
    let mut first = true;
    let mut cuts = 0;
    let mut occupies = 0;
    let mut our_cuts = 0;
    let mut foe_max_area = 0.0f64;
    let foe = if ret_blue { Player::Red } else { Player::Blue };
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if first {
            first = false;
            solo_move(&game, solo)
        } else if ret_moves {
            if contest {
                let (mv, class) = contest_best_move(game.position());
                match class {
                    Some(ContestClass::CutSite) => cuts += 1,
                    Some(ContestClass::Occupy) => occupies += 1,
                    None => {}
                }
                mv
            } else {
                retaliator::search::best_move(game.position())
            }
        } else {
            opp_collapser::best_move(game.position())
        };
        let mover_is_ret = (game.position().to_move() == Player::Blue) == ret_blue;
        let pre_actions = game.position().actions_played();
        let outcome = game.play(mv.unwrap()).unwrap();
        if mover_is_ret && pre_actions > 0 && outcome.broken.is_some() {
            our_cuts += 1;
        }
        foe_max_area = foe_max_area.max(game.position().area(foe).to_f64());
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss, cuts, occupies, our_cuts, foe_max_area, foe_max_area >= BIG_CLOSE)
}

fn arm(contest: bool, ret_blue: bool) {
    let label = format!(
        "{} ret={}",
        if contest { "CONTEST" } else { "BASELINE" },
        if ret_blue { "Blue" } else { "Red" }
    );
    let mut w = 0;
    let mut l = 0;
    let mut sum = 0.0;
    let mut converted = 0;
    let mut tcuts = 0u32;
    for (g, solo) in SOLOS.iter().enumerate() {
        let (rs, ss, cuts, occupies, our_cuts, foe_max, big) = play(solo, ret_blue, contest);
        let margin = (rs - ss) / rs * 100.0;
        if rs > ss {
            w += 1;
        } else {
            l += 1;
        }
        if !big {
            converted += 1;
        }
        sum += margin;
        tcuts += our_cuts;
        println!(
            "  {} game{} margin={:+.1}% contest cut={} occupy={} ourCuts={} foeMaxArea={:.0} bigClose={} {}",
            label, g + 1, margin, cuts, occupies, our_cuts, foe_max, big,
            if rs > ss { "WIN" } else { "LOSS" }
        );
    }
    println!(
        "{} {}-{} avg={:+.1}% ourCutsAvg={:.1}/game conversion={}/8 (n=8)",
        label, w, l, sum / 8.0, tcuts as f64 / 8.0, converted
    );
}

fn main() {
    println!("=== CONTEST-GEN vs COLLAPSER (skip=0, 8 solos, BONUS=0) ===");
    arm(false, true);
    arm(true, true);
    arm(false, false);
    arm(true, false);
}
