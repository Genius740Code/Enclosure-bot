//! Lane D2 tournament: next-gen style mimics (C2 site signatures) vs the
//! shipped search + v1 + v2.
//!
//! Same protocol as `probe_match_opp.rs`: round-robin, 8 games per matchup
//! (4 with each color), games differ only in Blue's solo action, tiebreak by
//! move index. Margins/W-L are from the BASELINE's perspective (ours).
//! A `v2+avoid` arm (shipped anti-rebuild wiring, CUT_MEMORY 6) runs against
//! every style so farm-cycle tricks can be measured against the routing we
//! actually deploy.
//!
//! Usage: `cargo run --release --example probe_match_d2 -- [styles...]`
//! (default: the D2 styles; `old` adds the lane-D styles for re-baselining).

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/v2base.rs"]
mod v2base;
#[path = "opp_gbstyle.rs"]
mod opp_gbstyle;
#[path = "opp_vladstyle.rs"]
mod opp_vladstyle;
#[path = "opp_angelstyle.rs"]
mod opp_angelstyle;
#[path = "opp_blobstyle.rs"]
mod opp_blobstyle;

use std::collections::HashMap;

use meridian_engine::notation::parse_square;
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point, Position};

/// Reasonable Blue solos (only the starting edge's ENDPOINTS A10/D10 can
/// extend); one per game — the same list `probe_match_opp` verified legal.
const SOLOS: [&str; 8] = [
    "D10-F11", "D10-C7", "A10-C11", "A10-D13", "D10-E12", "A10-B13", "D10-G10", "A10-C7",
];

type Engine<'a> = &'a dyn Fn(&Position, &[Point]) -> Option<Move>;

#[derive(Default, Clone, Copy)]
struct Stats {
    closes: u32,
    cuts: u32,
    first_close: Option<u16>,
}

struct GameOut {
    /// Score diff from the baseline's perspective.
    margin: f64,
    /// The baseline's own score (margin as a % of it).
    ret_score: f64,
    ret: Stats,
    opp: Stats,
    ret_area: f64,
    opp_area: f64,
}

fn solo_move(game: &Game, text: &str) -> Option<Move> {
    let (from, to) = text.split_once('-')?;
    let mv = Move::between(parse_square(from)?, parse_square(to)?)?;
    if game.legal_moves().contains(mv) {
        Some(mv)
    } else {
        game.legal_moves().iter().next()
    }
}

/// The endpoints of `mover`'s edges cut in the last [`CUT_MEMORY`] actions:
/// the shipped anti-rebuild wiring (`lib.rs` `replay()`).
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

fn play(ret_blue: bool, solo: &str, ret: Engine, opp: Engine) -> GameOut {
    let mut game = Game::new();
    let (mut ret_stats, mut opp_stats) = (Stats::default(), Stats::default());
    // (action index, owner of the broken edge, the broken edge).
    let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
    let mut first = true;
    while !game.is_over() {
        let mover = game.position().to_move();
        let ret_moves = (mover == Player::Blue) == ret_blue;
        let played = u16::from(game.position().actions_played());
        let own = (played + 2) / 2;
        let avoid = avoid_points(&cuts, mover, played);
        let mv = if first {
            first = false;
            solo_move(&game, solo)
        } else if ret_moves {
            ret(game.position(), &avoid)
        } else {
            opp(game.position(), &avoid)
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        if let Some(cut) = outcome.broken {
            cuts.push((played, mover.opponent(), cut));
        }
        let stats = if ret_moves { &mut ret_stats } else { &mut opp_stats };
        if outcome.kind == MoveKind::Connect {
            stats.closes += 1;
            if stats.first_close.is_none() {
                stats.first_close = Some(own);
            }
        }
        if outcome.broken.is_some() {
            stats.cuts += 1;
        }
    }
    let blue = game.position().score(Player::Blue).to_f64();
    let red = game.position().score(Player::Red).to_f64();
    let (ret_score, opp_score) = if ret_blue { (blue, red) } else { (red, blue) };
    let (ret_area, opp_area) = if ret_blue {
        (game.position().area(Player::Blue).to_f64(), game.position().area(Player::Red).to_f64())
    } else {
        (game.position().area(Player::Red).to_f64(), game.position().area(Player::Blue).to_f64())
    };
    GameOut { margin: ret_score - opp_score, ret_score, ret: ret_stats, opp: opp_stats, ret_area, opp_area }
}

fn fmt_stats(s: &Stats) -> String {
    format!(
        "close={} cut={} first={}",
        s.closes,
        s.cuts,
        s.first_close.map_or("-".into(), |a| a.to_string())
    )
}

/// W-L / avg margin over the 8 games, plus the loss pattern (own-action
/// numbered, per side) so the "how" is readable.
fn matchup(name: &str, ret: Engine, opp: Engine, opp_name: &str) {
    let mut outs = Vec::new();
    for g in 0..8 {
        let ret_blue = g < 4;
        let out = play(ret_blue, SOLOS[g], ret, opp);
        println!(
            "{name} vs {opp_name} game {} base={}: margin={:+.0} base[{:.1} {}] {opp_name}[{:.1} {}]",
            g + 1,
            if ret_blue { "blue" } else { "red" },
            out.margin,
            out.ret_area,
            fmt_stats(&out.ret),
            out.opp_area,
            fmt_stats(&out.opp),
        );
        outs.push(out);
    }
    let wins = outs.iter().filter(|o| o.margin > 0.0).count();
    let draws = outs.iter().filter(|o| o.margin == 0.0).count();
    let avg_pct = outs.iter().map(|o| o.margin / o.ret_score * 100.0).sum::<f64>() / outs.len() as f64;
    let avg_diff = outs.iter().map(|o| o.margin).sum::<f64>() / outs.len() as f64;
    let side_avg = |side: fn(&GameOut) -> Stats| -> (f64, f64, f64, u32) {
        let lost: Vec<_> = outs.iter().filter(|o| o.margin < 0.0).collect();
        if lost.is_empty() {
            return (0.0, 0.0, 0.0, 0);
        }
        let n = lost.len() as f64;
        (
            lost.iter().map(|o| side(o).closes as f64).sum::<f64>() / n,
            lost.iter().map(|o| side(o).cuts as f64).sum::<f64>() / n,
            lost.iter().map(|o| side(o).first_close.map_or(0.0, |a| a as f64)).sum::<f64>() / n,
            lost.len() as u32,
        )
    };
    let (rl, rc, rf, rn) = side_avg(|o| o.ret);
    let (ol, oc, of_, on) = side_avg(|o| o.opp);
    println!(
        "== {name} vs {opp_name}: W-L {wins}-{wins_loss} D{draws} avg_margin={avg_pct:+.1}% avg_diff={avg_diff:+.0} | in {rn} losses: base[close={rl:.1} cut={rc:.1} first={rf:.1}] | in {on} losses: {opp_name}[close={ol:.1} cut={oc:.1} first={of_:.1}]",
        wins_loss = 8 - draws - wins,
    );
}

fn main() {
    let mut styles: HashMap<&str, Engine> = HashMap::new();
    styles.insert("blob", &|pos, _| opp_blobstyle::best_move(pos) as Option<Move>);
    let mut old: Vec<(&str, Engine)> = vec![
        ("gb", &|pos, _| opp_gbstyle::best_move(pos)),
        ("vlad", &|pos, _| opp_vladstyle::best_move(pos)),
        ("angel", &|pos, _| opp_angelstyle::best_move(pos)),
        ("scout", &|pos, _| scoutbase::best_move(pos)),
    ];
    // Add each new mimic here as it lands:
    // ("sac", &|pos, avoid| opp_sacstyle::best_move(pos, avoid)),
    // ("longfarm", &|pos, avoid| opp_longfarm::best_move(pos, avoid)),
    let d2: Vec<(&str, Engine)> = vec![("blob", &|pos, _| opp_blobstyle::best_move(pos))];
    let mut chosen: Vec<(&str, Engine)> = Vec::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        chosen = d2;
    } else {
        for arg in &args {
            if arg == "old" {
                chosen.append(&mut old);
            } else if let Some(style) = styles.get(arg.as_str()) {
                chosen.push((arg.as_str(), *style));
            }
        }
    }
    let baselines: [(&str, Engine); 3] = [
        ("v3", &|pos, _| retaliator::search::best_move(pos)),
        ("v1", &|pos, _| v1base::best_move(pos)),
        ("v2", &|pos, _| v2base::best_move(pos)),
    ];
    for (style_name, style) in chosen {
        for (base_name, base) in baselines {
            // Baseline as `ret`: margin and W-L read from our perspective.
            matchup(base_name, base, style, style_name);
        }
        // Anti-rebuild arm: v2 with the shipped avoid wiring fed live.
        matchup("v2+avoid", &|pos, avoid| v2base::best_move_with_avoid(pos, avoid), style, style_name);
    }
}
