//! Lane H-verify -- the genuine `probe_league` gate with the mesh8 forced-start prefix
//! wired in on our side, both colors. The mesh8 adoption (lane-h bf3c9aa) was gated on
//! h2h sweeps + projections; the genuine 8-game league gate was never run. This probe
//! mirrors `probe_league.rs`'s structure (skip=0/10/20/30 pre-moves, both colors, us vs
//! scoutbase) with ONE change: OUR side's play is the mesh8 forced prefix (MESH-B as
//! Blue / MESH-R as Red) from our side's first own move in the game, handing to
//! `retaliator::search::best_move` after the 8th entry (or first illegal) -- the
//! `Bot::forced` pattern from `probe_mesh.rs`. The opponent's pre-moves stay bare
//! `retaliator::search::best_move` (the genuine league structure; our engine plays both
//! sides during the skip), and post-skip it plays scoutbase. Because our side's first own
//! move is engine act 1/2 regardless of skip, the prefix injects from the game's start on
//! our side (own moves 1-8 -- the forced-start pattern, "wrap at match start"); the
//! opponent's retaliator-search pre-moves may make a later prefix entry illegal, which
//! TRUNCATES the prefix (stall recorded, never silent -- the probe_mesh lessons).
//!
//! PROBE-ONLY: no `retaliator/src/*` changes; the shipped search (DOOM_W=1.0 state at
//! bf3c9aa, the -20.0% control state) is untouched.
//!
//! Margin = (ours - theirs) / ours * 100, ret perspective (probe_league's formula, with a
//! 1.0 floor so a wipeout can't divide by zero -- identical for any score >= 1).
//! Control (league-scoreboard baseline): AVG -20.0%, genuine blue +29.4% / red +38.1%,
//! collapse red -111.2 / -111.2 / -111.4 (skip=10/20/30).
//! ACCEPT bar: AVG > -9.9% AND no row worse than -300.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{notation, Game, Move, Player, Position};

type SearchFn = fn(&Position) -> Option<Move>;

const MESH_B: &str = "D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10 J11-M13";
const MESH_R: &str = "P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8";

fn v3(pos: &Position) -> Option<Move> {
    retaliator::search::best_move(pos)
}

fn parse_line(text: &str) -> Vec<Move> {
    text.split_whitespace()
        .map(|t| {
            let (a, b) = t.split_once('-').expect("move is SRC-TGT");
            Move::between(
                notation::parse_square(a).expect("source square"),
                notation::parse_square(b).expect("target square"),
            )
            .expect("1-3 king steps")
        })
        .collect()
}

/// A player: an optional forced prefix (own moves, in order) + a search hand-off.
/// Verbatim from probe_mesh.rs (the v1 off-by-one + bluechair no-op lessons): entry 0 is
/// ALWAYS the next prefix move to play; the first illegal entry truncates the prefix and
/// search takes over for good; injections counted, stalls never silent.
struct Bot {
    prefix: Vec<Move>,
    search: SearchFn,
    forced_idx: usize,
    expected: usize,
    inj: usize,
    /// First prefix slot that could not be played: (slot, engine act, why).
    stall: Option<(usize, u8, String)>,
}

impl Bot {
    fn forced(prefix: Vec<Move>, search: SearchFn) -> Bot {
        let expected = prefix.len();
        Bot { prefix, search, forced_idx: 0, expected, inj: 0, stall: None }
    }
    fn next(&mut self, pos: &Position, act1: u8) -> Move {
        if self.forced_idx < self.prefix.len() {
            let mv = self.prefix[self.forced_idx];
            match pos.check_move(mv) {
                Ok(_) => {
                    self.forced_idx += 1;
                    self.inj += 1;
                    return mv;
                }
                Err(why) => {
                    if self.stall.is_none() {
                        self.stall = Some((self.forced_idx, act1, format!("{why}")));
                    }
                    // Truncate: the prefix is dead, search takes over for good.
                    self.forced_idx = self.prefix.len();
                }
            }
        }
        (self.search)(pos).expect("search returns a move")
    }
}

fn stall_txt(stall: &Option<(usize, u8, String)>) -> String {
    match stall {
        None => String::new(),
        Some((slot, act, why)) => format!(" STALL[#{slot}@act{act}:{why}]"),
    }
}

fn main() {
    // Id decode gate (probe_mesh: D10-D13 = 16058) -- a mismatch means the notation parse
    // is wrong, abort.
    let mesh_b = parse_line(MESH_B);
    let mesh_r = parse_line(MESH_R);
    assert_eq!(mesh_b.len(), 8, "mesh8 = 8 blue entries");
    assert_eq!(mesh_r.len(), 8, "mesh8 = 8 red entries");
    assert_eq!(mesh_b[0].index(), 16058, "D10-D13 must decode to 16058");

    let mut sum = 0.0;
    let mut n = 0usize;
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            // Our side: mesh8 forced prefix + v3, from our side's first own move in the
            // game (both colors). Opponent: bare v3 pre-moves (the genuine structure),
            // scoutbase after the skip.
            let mut us =
                Bot::forced(if ret_blue { mesh_b.clone() } else { mesh_r.clone() }, v3);
            let mut game = Game::new();
            let mut pre = 0usize;
            while pre < skip && !game.is_over() {
                let pos = game.position();
                let ours = (pos.to_move() == Player::Blue) == ret_blue;
                let act1 = pos.actions_played() + 1;
                let mv = if ours { us.next(&pos, act1) } else { v3(&pos).expect("pre-move") };
                game.play(mv).unwrap();
                pre += 1;
            }
            while !game.is_over() {
                let pos = game.position();
                let ours = (pos.to_move() == Player::Blue) == ret_blue;
                let act1 = pos.actions_played() + 1;
                let mv = if ours {
                    us.next(&pos, act1)
                } else {
                    scoutbase::best_move(&pos).expect("scout move")
                };
                game.play(mv).unwrap();
            }
            let fin = game.position();
            let rs = fin.score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
            let ss = fin.score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
            let m = (rs - ss) / rs.max(1.0) * 100.0;
            println!(
                "skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% inj={}/{}{}",
                if ret_blue { "blue" } else { "red" },
                us.inj,
                us.expected,
                stall_txt(&us.stall)
            );
            sum += m;
            n += 1;
        }
    }
    println!("AVG margin (ret perspective): {:.1}%", sum / n as f64);
}
