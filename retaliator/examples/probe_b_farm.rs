//! Lane B3 farm-cycle confirm probe (c-site-losses-2 H1). Does the scaled
//! rebuild routing + extended cut memory actually flip the farmed
//! re-closes the site losses are made of? Three measurements per game:
//!
//! (1) FIDELITY: replay the site moves; the DEPLOYED decision process
//!     (shipped `search::best_move_with_avoid` on the lib.rs mem-6 avoid
//!     reconstruction) must re-pick the site's move at every our-action
//!     (v4 games: 59-60/60 per c-site-losses-2 §0) — this validates the
//!     probe's avoid reconstruction against `lib.rs` exactly.
//! (2) CONFIRM TARGETS (C2 H1): 446956a1 act 51 (1-based; 0-based 50) =
//!     re-close 16013 +4.5, popped at [52] by 17112 (4th re-pop of the
//!     same id); f9c819ed act 113 (0-based 112) = re-close 16970 +10.5
//!     (4th re-close of the same id). "The counter must make any
//!     fresh-ground move outrank it": per config, is the pick flipped off
//!     the re-close and off the cut ground?
//! (3) WHOLE-GAME SCORECARD: at EVERY our-action where the CONTROL
//!     (doom OFF, no avoid) picks a cut-ground close (Connect with gain
//!     > 0 landing on ground cut within 40 actions — the farmed
//!     re-close), how often does each config pick something else, and is
//!     the new pick off the config's own cut ground? Fixed site
//!     trajectory, per-action comparison (the probe_b_close precedent).
//!
//! Configs: ctrl (no avoid, doom OFF) / dep (deployed: doom ON, mem 6,
//! flat) / m{6,12,20,40}f (flat routing) / m{..}se (+ gain-scaled,
//! counter-cut exemption kept) / m{..}s (+ gain-scaled on all first
//! actions — the farm-cycle form). Usage:
//! `cargo run --release --example probe_b_farm` (built-in targets), or
//! `cargo run --release --example probe_b_farm -- GAME.json blue|red ACT0BASED`

#[path = "../src/eval_phases.rs"]
mod eval_phases;
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point, Position};

/// Matches `eval_phases`' shipped CUT_RADIUS (anti-rebuild routing).
const CUT_RADIUS: i8 = 3;
const WIDEST: usize = 40;

/// Endpoints of `me`'s edges cut within the last `mem` actions: the
/// `lib.rs` `replay()` wiring, memory generalized. `mem == 0` = none.
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

/// Whether `target` is within Chebyshev `CUT_RADIUS` of any avoid point.
fn on_ground(avoid: &[Point], target: Point) -> bool {
    avoid
        .iter()
        .any(|p| (p.x() - target.x()).abs() <= CUT_RADIUS && (p.y() - target.y()).abs() <= CUT_RADIUS)
}

/// (own area gain, is a Connect) of `mv` for the player to move.
fn apply_info(position: &Position, mv: Move) -> (f64, bool) {
    let mover = position.to_move();
    let mut after = position.clone();
    let oc = after.apply_unchecked(mv);
    let gain = after.area(mover).to_f64() - position.area(mover).to_f64();
    (gain, oc.kind == MoveKind::Connect)
}

fn pick_of(
    position: &Position,
    avoid: &[Point],
    mem: usize,
    scale: u8,
) -> Option<Move> {
    match (mem, scale) {
        (0, _) => eval_phases::best_move(position),
        (_, 0) => eval_phases::best_move_with_avoid(position, avoid),
        (_, 1) => eval_phases::farm_best_move_with_avoid(position, avoid, eval_phases::Rebuild::ScaledExempt),
        _ => eval_phases::farm_best_move_with_avoid(position, avoid, eval_phases::Rebuild::ScaledAll),
    }
}

/// One config column: (name, mem, scale).
const CONFIGS: [(&str, usize, u8); 10] = [
    ("m6f", 6, 0),
    ("m12f", 12, 0),
    ("m20f", 20, 0),
    ("m40f", 40, 0),
    ("m6se", 6, 1),
    ("m12se", 12, 1),
    ("m20se", 20, 1),
    ("m40se", 40, 1),
    ("m6s", 6, 2),
    ("m12s", 12, 2),
    // m20s / m40s handled via args? No — add them below to keep the table complete.
];

fn run_game(path: &str, me: Player, confirm_at: usize, label: &str) {
    let Ok(body) = std::fs::read_to_string(path) else {
        println!("=== {label}: SKIP ({path} not found) ===");
        return;
    };
    let d: serde_json::Value = serde_json::from_str(&body).unwrap();
    let ids: Vec<usize> = d["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    let mut game = Game::new();
    let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
    let mut our_actions = 0usize;
    let mut deployed_match = 0usize;
    let mut disease = 0usize;
    // (flips, of which off own ground) per config, plus m20s/m40s appended.
    let mut flips = vec![(0usize, 0usize); CONFIGS.len() + 2];
    println!("=== {label}: us={} {} site moves ===", if me == Player::Blue { "blue" } else { "red" }, ids.len());
    for (i, &id) in ids.iter().enumerate() {
        let pos = game.position();
        let to_move = pos.to_move();
        let played = u16::from(pos.actions_played());
        if to_move == me {
            our_actions += 1;
            let avoid: Vec<(usize, Vec<Point>)> =
                [6usize, 12, 20, 40].iter().map(|&m| (m, avoid_points(&cuts, me, played, m))).collect();
            let av = |m: usize| avoid.iter().find(|(mm, _)| *mm == m).unwrap().1.clone();
            // Fidelity: the deployed process must re-pick the site move.
            let dep = retaliator::search::best_move_with_avoid(pos, &av(6));
            let site_mv = Move::from_index(id).unwrap();
            if dep == Some(site_mv) {
                deployed_match += 1;
            }
            // Control pick + the disease test (cut-ground close, window 40).
            let ctrl = eval_phases::best_move(pos);
            let (d_gain, d_connect, d_ground) = match ctrl {
                Some(mv) => {
                    let (g, c) = apply_info(pos, mv);
                    let tgt = mv.target().expect("legal moves end on the board");
                    (g, c, on_ground(&av(WIDEST), tgt))
                }
                None => (0.0, false, false),
            };
            let is_disease = d_connect && d_gain > 0.0 && d_ground;
            // Config picks.
            let mut all: Vec<(&str, usize, u8, Option<Move>)> = Vec::new();
            for &(name, m, s) in CONFIGS.iter() {
                all.push((name, m, s, pick_of(pos, &av(m), m, s)));
            }
            all.push(("m20s", 20, 2, pick_of(pos, &av(20), 20, 2)));
            all.push(("m40s", 40, 2, pick_of(pos, &av(40), 40, 2)));
            if is_disease {
                disease += 1;
                for (k, (_, m, _, pick)) in all.iter().enumerate() {
                    if let Some(mv) = pick {
                        if *mv != ctrl.unwrap() {
                            let tgt = mv.target().expect("legal moves end on the board");
                            let off = !on_ground(&av(*m), tgt);
                            flips[k].0 += 1;
                            if off {
                                flips[k].1 += 1;
                            }
                        }
                    }
                }
            }
            if i == confirm_at {
                let (sg, _) = apply_info(pos, site_mv);
                let stgt = site_mv.target().expect("legal moves end on the board");
                let sgr: Vec<char> = [6usize, 12, 20, 40]
                    .iter()
                    .map(|&m| if on_ground(&av(m), stgt) { 'Y' } else { 'n' })
                    .collect();
                println!(
                    "confirm act {} (0-based): site move {id} gain {sg:+.1} on-ground mem6/12/20/40: {} | deployed picks it: {} | control picks: {}",
                    i,
                    sgr.iter().collect::<String>(),
                    if dep == Some(site_mv) { "YES" } else { "NO" },
                    ctrl.map(|m| m.index().to_string()).unwrap_or_else(|| "none".into()),
                );
                println!("  config  pick  gain  on-own-ground  flipped-off-site-move");
                for (name, m, _, pick) in &all {
                    match pick {
                        Some(mv) => {
                            let (g, _) = apply_info(pos, *mv);
                            let tgt = mv.target().expect("legal moves end on the board");
                            println!(
                                "  {name:6}  {:5}  {g:+.1}  {}  {}",
                                mv.index(),
                                if on_ground(&av(*m), tgt) { "YES" } else { "no " },
                                if *mv != site_mv { "FLIPPED" } else { "kept   " },
                            );
                        }
                        None => println!("  {name:6}  none"),
                    }
                }
            }
        }
        let oc = game.play(Move::from_index(id).expect("site id")).expect("site moves are legal");
        if let Some(cut) = oc.broken {
            cuts.push((played, to_move.opponent(), cut));
        }
    }
    println!(
        "deployed fidelity: {deployed_match}/{our_actions} our-actions re-picked",
    );
    println!(
        "farmed re-closes the control wants (Connect gain>0 on ground-40): {disease}; per config: flips/off-ground",
    );
    let names: Vec<&str> = CONFIGS
        .iter()
        .map(|(n, _, _)| *n)
        .chain(["m20s", "m40s"])
        .collect();
    for (k, name) in names.iter().enumerate() {
        println!("  {name:6}: {}/{} off-ground", flips[k].0, flips[k].1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 4 {
        let me = if args[2] == "blue" { Player::Blue } else { Player::Red };
        let act: usize = args[3].parse().expect("0-based action");
        run_game(&args[1], me, act, &format!("{} @{}", args[1], act));
        return;
    }
    // C2 H1 confirm targets (c-site-losses-2 §6): the site games live
    // outside the repo (lane C2's fetch); skip gracefully if absent.
    run_game(
        "/tmp/opencode/sitegames/losses/446956a1-d0bb-42fe-a3d4-94569603d0e9.json",
        Player::Red,
        50,
        "446956a1 act51 re-close 16013 (capybara v4)",
    );
    run_game(
        "/tmp/opencode/sitegames/losses/f9c819ed-001d-4330-b869-6b8d237c7073.json",
        Player::Blue,
        112,
        "f9c819ed act113 re-close 16970 (H3 cluster)",
    );
}
