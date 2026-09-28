//! Lane D2 trace probe: one full game, style vs baseline, with a per-action
//! trace (own action, move text, kind, both areas, both banks) — the
//! action-numbered evidence for the research files.
//!
//! Usage: `probe_d2_trace <longfarm|sac|blob> <v2|v2a> <blue|red>`.

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v2base.rs"]
mod v2base;
#[path = "opp_blobstyle.rs"]
mod opp_blobstyle;
#[path = "opp_sacstyle.rs"]
mod opp_sacstyle;
#[path = "opp_longfarm.rs"]
mod opp_longfarm;

use meridian_engine::notation::parse_square;
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point, Position};

const SOLO: &str = "D10-F11";

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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let style = args.first().map(String::as_str).unwrap_or("longfarm");
    let base = args.get(1).map(String::as_str).unwrap_or("v2a");
    let style_blue = !args.get(2).map(String::as_str).unwrap_or("blue").eq("red");
    let style_move = |pos: &Position, avoid: &[Point], me: Player| -> Option<Move> {
        let _ = me;
        match style {
            "longfarm" => opp_longfarm::best_move(pos, avoid),
            "sac" => opp_sacstyle::best_move(pos, avoid),
            _ => opp_blobstyle::best_move(pos),
        }
    };
    let base_move = |pos: &Position, avoid: &[Point]| match base {
        "v2" => v2base::best_move(pos),
        _ => v2base::best_move_with_avoid(pos, avoid),
    };

    let mut game = Game::new();
    let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
    let mut first = true;
    while !game.is_over() {
        let mover = game.position().to_move();
        let played = u16::from(game.position().actions_played());
        let own = (played + 2) / 2;
        let style_moves = (mover == Player::Blue) == style_blue;
        let avoid = avoid_points(&cuts, mover, played);
        let mv = if first {
            first = false;
            let (from, to) = SOLO.split_once('-').unwrap();
            let solo = Move::between(parse_square(from).unwrap(), parse_square(to).unwrap()).unwrap();
            if game.legal_moves().contains(solo) { Some(solo) } else { game.legal_moves().iter().next() }
        } else if style_moves {
            style_move(game.position(), &avoid, mover)
        } else {
            base_move(game.position(), &avoid)
        };
        let text = format!("{:?}->{:?}", mv.unwrap().source, mv.unwrap().target());
        let outcome = game.play(mv.unwrap()).unwrap();
        if let Some(cut) = outcome.broken {
            cuts.push((played, mover.opponent(), cut));
        }
        let tag = if style_moves { "ST" } else { "BS" };
        let kind = match outcome.kind {
            MoveKind::Connect => "C",
            MoveKind::Capture => "X",
            MoveKind::Extend => ".",
        };
        let cut = if outcome.broken.is_some() { "cut!" } else { "    " };
        println!(
            "{tag} a{own} {text} {kind} {cut} | area {:.0}/{:.0} bank {:.0}/{:.0}",
            game.position().area(Player::Blue).to_f64(),
            game.position().area(Player::Red).to_f64(),
            game.position().score(Player::Blue).to_f64(),
            game.position().score(Player::Red).to_f64(),
        );
    }
    let bs = game.position().score(Player::Blue).to_f64();
    let rs = game.position().score(Player::Red).to_f64();
    println!(
        "final blue={bs:.0} red={rs:.0} style({style} as {}) {:+.0}",
        if style_blue { "blue" } else { "red" },
        if style_blue { bs - rs } else { rs - bs }
    );
}
