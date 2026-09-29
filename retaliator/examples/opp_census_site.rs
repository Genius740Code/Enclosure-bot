//! Lane O fallback: run the GB unbreakable-share census spec
//! (`research/v7-gb-census-spec.md`) against local site games.
//! Replays each game in /tmp/opencode/sitegames/games via engine only,
//! computes share(P) per spec, prediction test: share(winner) > share(loser).
//! Reports n, ties, NaN games, replay errors separately.

use meridian_engine::{Game, Move, Player, Position};
use std::collections::HashMap;

const GAMES_DIR: &str = "/tmp/opencode/sitegames/games";

fn thickness(pos: &Position, player: Player) -> f64 {
    let mut degree: HashMap<usize, u32> = HashMap::new();
    for edge in pos.edges(player).iter() {
        for end in [edge.origin(), edge.far()] {
            *degree.entry(end.index()).or_insert(0) += 1;
        }
    }
    let edges = pos.edges(player);
    if edges.is_empty() {
        return 0.0;
    }
    let thick = edges
        .iter()
        .filter(|e| {
            degree.get(&e.origin().index()).unwrap_or(&0) >= &2
                && degree.get(&e.far().index()).unwrap_or(&0) >= &2
        })
        .count();
    thick as f64 / edges.len() as f64
}

fn main() {
    let mut files: Vec<String> = std::fs::read_dir(GAMES_DIR)
        .expect("games dir")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|f| f.ends_with(".json"))
        .collect();
    files.sort();
    let (mut agree, mut decided, mut ties, mut nan, mut errs, mut n) = (0, 0, 0, 0, 0, 0);
    let mut fam: HashMap<String, (u32, u32)> = HashMap::new();
    for f in &files {
        let text = std::fs::read_to_string(format!("{GAMES_DIR}/{f}")).unwrap();
        let g: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => {
                errs += 1;
                continue;
            }
        };
        let moves = match g["moves"].as_array() {
            Some(m) => m,
            None => {
                errs += 1;
                continue;
            }
        };
        let winner = match g["result"].as_str() {
            Some("blue") => Player::Blue,
            Some("red") => Player::Red,
            _ => {
                errs += 1;
                continue;
            }
        };
        let mut game = Game::new();
        let mut acc = [0.0f64; 2];
        let mut tot = [0.0f64; 2];
        let mut ok = true;
        for mv in moves {
            let idx = match mv.as_u64() {
                Some(i) => i as usize,
                None => {
                    ok = false;
                    break;
                }
            };
            let m = match Move::from_index(idx) {
                Some(m) => m,
                None => {
                    ok = false;
                    break;
                }
            };
            match game.play(m) {
                Ok(outcome) => {
                    if let Some(scored) = outcome.scored {
                        let gains = [scored[0].to_f64(), scored[1].to_f64()];
                        for (i, pl) in [Player::Blue, Player::Red].iter().enumerate() {
                            if gains[i] > 0.0 {
                                acc[i] += gains[i] * thickness(game.position(), *pl);
                                tot[i] += gains[i];
                            }
                        }
                    }
                }
                Err(_) => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            errs += 1;
            continue;
        }
        n += 1;
        let share = |i: usize| if tot[i] > 0.0 { acc[i] / tot[i] } else { f64::NAN };
        let (sb, sr) = (share(0), share(1));
        let key = format!(
            "{} v {}",
            g["blue"].as_str().unwrap_or("?"),
            g["red"].as_str().unwrap_or("?")
        );
        if sb.is_nan() || sr.is_nan() {
            nan += 1;
            println!("{f} {key} share B={sb:.2} R={sr:.2} NaN");
            continue;
        }
        let (ws, ls) = if winner == Player::Blue { (sb, sr) } else { (sr, sb) };
        decided += 1;
        let verdict = if ws > ls {
            agree += 1;
            let e = fam.entry(key.clone()).or_insert((0, 0));
            e.0 += 1;
            e.1 += 1;
            "AGREE"
        } else if ws < ls {
            let e = fam.entry(key.clone()).or_insert((0, 0));
            e.1 += 1;
            "DISAGREE"
        } else {
            ties += 1;
            let e = fam.entry(key.clone()).or_insert((0, 0));
            e.1 += 1;
            "TIE"
        };
        println!("{f} {key} winner={} shareB={sb:.3} shareR={sr:.3} {verdict}",
            if winner == Player::Blue { "blue" } else { "red" });
    }
    println!("=== CENSUS-SITE: n={n} decided={decided} agree={agree} ties={ties} nan={nan} replay-errors={errs}");
    let mut fams: Vec<_> = fam.into_iter().collect();
    fams.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
    for (k, (a, d)) in fams {
        println!("  fam {k}: agree {a}/{d}");
    }
}
