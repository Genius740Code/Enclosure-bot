//! Lane O (v7) style-mimic: the COLLAPSER — reproduces the worst league
//! collapse rows (skip=10/20/30 red -111%) from `league-scoreboard.md`.
//!
//! This is the scout rush-closer style (floor 0) that proved most dangerous
//! in Lane D (13-11 overall, collapse lines -855, -661, -426).
//! Signature: closes early and often (actions 2-3), many small loops
//! compounding, then a 30+ pop at ~t=60 followed by steady erosion.
//! The 2-ply search cannot see the compounding pops coming (horizon=12).

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Move, Player, Position};

/// The COLLAPSER is the scoutbase rush-closer: closes at floor 0,
/// builds compounding loops, delivers the 30+ pop at ~t=60.
pub fn best_move(position: &Position) -> Option<Move> {
    scoutbase::best_move(position)
}

/// Smoke test: trace the collapse pattern against retaliator search.
fn main() {
    // This is just scoutbase - the collapse emerges from the matchup
    // Run probe_league or probe_collapse to see it
    println!("COLLAPSER = scoutbase rush-closer (floor 0)");
    println!("Run: cargo run --example probe_league");
    println!("     cargo run --example probe_collapse");
}