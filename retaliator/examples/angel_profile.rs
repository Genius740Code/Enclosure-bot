//! angel_profile: the rising-area circle census on recorded games (v9 lane 7).
//!
//! Port of the 2a41473 JS instrument's core to the site-format v8 games:
//! a **circle** = one applied move that raises the mover's banked territory
//! area (the same quantity the game scores on — no proxy drift). Per side per
//! game: closes (area rises), area/close, first close action, pop rate (share
//! of circles whose area later fell back below their close level). Final
//! scores are printed against the recorded `score` field for checking.
//!
//! Usage: cargo run --release --example angel_profile -- <path-or-dir>...
//! Read-only: replays recorded games, changes no bot behaviour, no src edits.

#[path = "support/gamejson.rs"]
mod gamejson;

use gamejson::{game_files, RecordedGame};
use meridian_engine::Player;

#[derive(Default)]
struct CircleAcc {
    closes: usize,
    area_sum: f64,
    first_close: Option<usize>,
    popped: usize,
    /// (close action, close level, popped flag)
    circles: Vec<(usize, f64, bool)>,
}

impl CircleAcc {
    fn on_area(&mut self, action: usize, area: f64, last: f64) -> f64 {
        if area > last + 1e-9 {
            self.closes += 1;
            self.area_sum += area - last;
            if self.first_close.is_none() {
                self.first_close = Some(action);
            }
            self.circles.push((action, area, false));
            area
        } else if area < last - 1e-9 {
            for (_, level, popped) in self.circles.iter_mut() {
                if !*popped && *level > area + 1e-9 {
                    *popped = true;
                    self.popped += 1;
                }
            }
            area
        } else {
            last
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(!args.is_empty(), "usage: angel_profile <path-or-dir>...");
    let files = game_files(&args);

    let mut totals: [CircleAcc; 2] = Default::default(); // [Blue, Red]
    let mut games = 0usize;

    for file in &files {
        let Some(recorded) = gamejson::load(file) else {
            println!("skip (neither format): {}", file.display());
            continue;
        };
        let mut game = meridian_engine::Game::new();
        let mut last_area = [0.0f64; 2];
        let mut per_game: [CircleAcc; 2] = Default::default();
        for (mv, mover) in &recorded.actions {
            if game.is_over() {
                break;
            }
            if game.position().to_move() != *mover {
                break;
            }
            if game.position().check_move(*mv).is_err() {
                break;
            }
            game.play(*mv).expect("recorded move legal");
            let idx = if *mover == Player::Blue { 0 } else { 1 };
            let area = game.position().area(*mover).to_f64();
            last_area[idx] = per_game[idx].on_area(game.moves().len(), area, last_area[idx]);
        }
        games += 1;
        for idx in 0..2 {
            totals[idx].closes += per_game[idx].closes;
            totals[idx].area_sum += per_game[idx].area_sum;
            totals[idx].popped += per_game[idx].popped;
            totals[idx].circles.extend(per_game[idx].circles.iter().cloned());
            if let Some(fc) = per_game[idx].first_close {
                totals[idx].first_close = Some(match totals[idx].first_close {
                    None => fc,
                    Some(prev) => prev.min(fc),
                });
            }
        }
        let b = game.position().score(Player::Blue).to_f64();
        let r = game.position().score(Player::Red).to_f64();
        println!(
            "{}: final {b:.1}-{r:.1} | closes blue {} (area {:.1}, popped {}/{}) red {} (area {:.1}, popped {}/{}) | first close blue {} red {}",
            recorded.name,
            per_game[0].closes,
            per_game[0].area_sum,
            per_game[0].popped,
            per_game[0].circles.len(),
            per_game[1].closes,
            per_game[1].area_sum,
            per_game[1].popped,
            per_game[1].circles.len(),
            per_game[0].first_close.map(|a| a.to_string()).unwrap_or("-".into()),
            per_game[1].first_close.map(|a| a.to_string()).unwrap_or("-".into()),
        );
    }

    println!("\n=== RISING-AREA circle census (totals over {games} games) ===");
    for (idx, acc) in totals.iter().enumerate() {
        let cname = if idx == 0 { "Blue" } else { "Red" };
        println!(
            "{cname}: closes {} ({:.2}/game) | area/close {:.2} | earliest first close act {} | pop rate {:.2} ({}/{})",
            acc.closes,
            acc.closes as f64 / games as f64,
            if acc.closes == 0 { 0.0 } else { acc.area_sum / acc.closes as f64 },
            acc.first_close.map(|a| a.to_string()).unwrap_or("-".into()),
            if acc.circles.is_empty() {
                0.0
            } else {
                acc.popped as f64 / acc.circles.len() as f64
            },
            acc.popped,
            acc.circles.len()
        );
    }
}
