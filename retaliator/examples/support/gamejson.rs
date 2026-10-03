//! Shared game-JSON loader for read-only census probes (lane-v9 analysis).
//!
//! Handles both recorded formats:
//! - **site**: `{"moves": [site ids...], "blue": name, "red": name, ...}` —
//!   ids decode with `Move::from_index`.
//! - **local sparring**: `{"seq": [{who, color?, moves: [{from:[x,y], to:[x,y]}], dt}...]}` —
//!   explicit coordinates in the JS engine frame (0..18 on both axes);
//!   the Rust `Point` frame is the same grid shifted by (-9, -9): the JS
//!   standard start `[[0,9],[3,9]] / [[18,9],[15,9]]` is the Rust start
//!   `(-9,0)-(-6,0) / (6,0)-(9,0)` (verified against `position.rs::new`).
//!
//! "Our" color: the site bot name (contains "riposte", case-insensitive) or
//! the local filename (`*Isblue*` / `*Isred*`); falls back to the recorded
//! `bot` color, then Blue. The opponent name comes from the file name
//! (`chall-<bot>Is...` / `R-chall-<bot>Is...-sNNNN`) or the site JSON.

use meridian_engine::{Move, Player, Point};
use serde_json::Value;
use std::path::Path;

pub struct RecordedGame {
    /// File stem or site game id, for logs.
    pub name: String,
    /// Our color ("me" from the recorder's perspective).
    pub me: Player,
    /// Opponent name for per-opponent breakdowns.
    pub opponent: String,
    /// Every action in order: (move, mover).
    pub actions: Vec<(Move, Player)>,
}

/// Loads one game JSON; `None` when the file is neither format (skipped, logged).
pub fn load(path: &Path) -> Option<RecordedGame> {
    let raw = std::fs::read_to_string(path).ok()?;
    let d: Value = serde_json::from_str(&raw).ok()?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    if let Some(ids) = d["moves"].as_array() {
        // Site format.
        let blue_name = d["blue"].as_str().unwrap_or("blue").to_string();
        let red_name = d["red"].as_str().unwrap_or("red").to_string();
        let me = if blue_name.to_lowercase().contains("riposte") {
            Player::Blue
        } else if red_name.to_lowercase().contains("riposte") {
            Player::Red
        } else if stem.to_lowercase().contains("isblue") {
            Player::Blue
        } else if stem.to_lowercase().contains("isred") {
            Player::Red
        } else {
            Player::Blue
        };
        let opponent = if me == Player::Blue { red_name.clone() } else { blue_name.clone() };
        let mut actions = Vec::with_capacity(ids.len());
        for id in ids {
            let index = id.as_u64()? as usize;
            let mv = Move::from_index(index).or_else(|| {
                panic!("site id {index} does not decode in {}", path.display())
            })?;
            let mover = unknown_mover(&actions);
            actions.push((mv, mover));
        }
        return Some(RecordedGame { name: stem, me, opponent, actions });
    }
    if let Some(seq) = d["seq"].as_array() {
        // Local sparring format. The bot's color: entries with who == "bot"
        // carry a color on chall-* files; otherwise the file name decides.
        let mut bot_color = None;
        for turn in seq {
            if turn["who"].as_str() == Some("bot") {
                if let Some(color) = turn["color"].as_str() {
                    bot_color = Some(if color == "blue" { Player::Blue } else { Player::Red });
                }
                break;
            }
        }
        let me = bot_color.unwrap_or_else(|| {
            if stem.to_lowercase().contains("isred") { Player::Red } else { Player::Blue }
        });
        let mut actions = Vec::new();
        for turn in seq {
            let mover = match turn["who"].as_str() {
                Some("bot") => me,
                Some("chall") => me.opponent(),
                _ => continue,
            };
            let Some(moves) = turn["moves"].as_array() else { continue };
            for step in moves {
                let (Some(from), Some(to)) = (step["from"].as_array(), step["to"].as_array()) else {
                    continue;
                };
                let (fx, fy) = (from[0].as_f64()?, from[1].as_f64()?);
                let (tx, ty) = (to[0].as_f64()?, to[1].as_f64()?);
                // JS frame (0..18) -> Rust frame (-9..9) on both axes.
                let a = Point::new(fx as i8 - 9, fy as i8 - 9).expect("seq point on board");
                let b = Point::new(tx as i8 - 9, ty as i8 - 9).expect("seq point on board");
                let mv = Move::between(a, b)
                    .unwrap_or_else(|| panic!("seq move {}-{} does not decode in {}", fx, fy, path.display()));
                actions.push((mv, mover));
            }
        }
        let opponent = opponent_from_stem(&stem);
        return Some(RecordedGame { name: stem, me, opponent, actions });
    }
    None
}

/// The mover for site ids: the engine's turn structure is deterministic by
/// action count (blue opens, then red/blue pairs), so the mover follows the
/// same rule as `search.rs::side_to_move`.
fn unknown_mover(actions: &[(Move, Player)]) -> Player {
    let total = actions.len();
    if total == 0 {
        Player::Blue
    } else if ((total - 1) / 2) % 2 == 0 {
        Player::Red
    } else {
        Player::Blue
    }
}

/// `chall-aggro-botIsblue` / `R-chall-random-botIsblue-s1000` -> `aggro` / `random`.
fn opponent_from_stem(stem: &str) -> String {
    let body = stem.strip_prefix("R-chall-").or_else(|| stem.strip_prefix("chall-")).unwrap_or(stem);
    let body = body.split("-botIs").next().unwrap_or(body);
    // Drop a trailing seed suffix (-s1000).
    let body = match body.rsplit_once("-s") {
        Some((head, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => head,
        _ => body,
    };
    body.to_string()
}

/// Enumerates game JSON files from the arguments: a directory contributes its
/// `*.json` entries (sorted for determinism); a file is used as-is.
pub fn game_files(args: &[String]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for arg in args {
        let path = Path::new(arg);
        if path.is_dir() {
            let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(path)
                .expect("read dir")
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
                .collect();
            entries.sort();
            files.extend(entries);
        } else {
            files.push(path.to_path_buf());
        }
    }
    files
}
