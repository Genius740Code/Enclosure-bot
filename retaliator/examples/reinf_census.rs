//! REINFORCE-LINES kill-0: unbreakable-share vs win census on corpus games.
//! For every wall/extend action by our bot landing next to 2+ own nodes,
//! does the structure survive unbroken to game end? Split by win/loss.

use meridian_engine::{Game, Move, MoveKind, Player, Point, notation};
use serde_json::Value;
use std::fs;

fn js_pt(a: &[f64]) -> Point {
    Point::new((a[0].round() as i64 - 9) as i8, (9 - a[1].round() as i64) as i8).expect("on board")
}

fn main() {
    let m = Move::from_index(16058).expect("id");
    assert_eq!(notation::move_text(m.source, m.target().expect("t")), "D10-D13");
    let dir = std::env::args().nth(1).expect("corpus dir");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".moves.json"))
        .collect();
    files.sort();
    // (chair Blue/Red, won?) -> (builds, survived)
    let mut stats = [[[0u64; 2]; 2]; 2];
    for name in &files {
        let our_blue = name.contains("botIsblue");
        let ci = if our_blue { 0 } else { 1 };
        let ours = if our_blue { Player::Blue } else { Player::Red };
        let j: Value = serde_json::from_str(&fs::read_to_string(format!("{dir}/{name}")).unwrap()).unwrap();
        let bs = j["scores"]["blue"].as_f64().unwrap_or(0.0);
        let rs = j["scores"]["red"].as_f64().unwrap_or(0.0);
        let won = if our_blue { bs > rs } else { rs > bs };
        let wi = if won { 1 } else { 0 };
        let mut game = Game::new();
        let mut builds: Vec<(usize, Vec<Point>)> = Vec::new();
        let mut breaks: Vec<(usize, Point, Point)> = Vec::new();
        let mut idx = 0usize;
        for t in j["seq"].as_array().cloned().unwrap_or_default() {
            let Some(mvs) = t.get("moves").and_then(|v| v.as_array()) else { continue };
            for m in mvs {
                let from = js_pt(&m["from"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>());
                let to = js_pt(&m["to"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>());
                let mv = Move::between(from, to).expect("king step");
                let mover = game.position().to_move();
                let oc = game.play(mv).expect("legal replay");
                if mover == ours
                    && (oc.kind == MoveKind::Connect || oc.kind == MoveKind::Extend)
                {
                    let near: Vec<Point> = game.position().nodes(ours).iter().filter(|n| (n.x() - to.x()).abs() <= 1 && (n.y() - to.y()).abs() <= 1).collect();
                    if near.len() >= 2 {
                        builds.push((idx, near));
                    }
                }
                if let Some(cut) = oc.broken {
                    breaks.push((idx, cut.origin(), cut.far()));
                }
                idx += 1;
            }
        }
        let mut surv = 0u64;
        for (bidx, near) in &builds {
            let mut ok = true;
            for (br_idx, o, f) in &breaks {
                if br_idx <= bidx {
                    continue;
                }
                if near.iter().any(|n| *n == *o || *n == *f) {
                    ok = false;
                    break;
                }
            }
            surv += ok as u64;
        }
        stats[ci][wi][0] += builds.len() as u64;
        stats[ci][wi][1] += surv;
        println!("{name}: builds={} survived={surv} won={won}", builds.len());
    }
    println!("-- survival = no later cut touching build nodes --");
    for (ci, ch) in ["Blue", "Red"].iter().enumerate() {
        for (wi, w) in ["loss", "win"].iter().enumerate() {
            let (b, s) = (stats[ci][wi][0], stats[ci][wi][1]);
            println!("CHAIR {ch} {w}: {s}/{b} = {:.1}%", 100.0 * s as f64 / b.max(1) as f64);
        }
    }
    let b: u64 = stats.iter().map(|c| c.iter().map(|w| w[0]).sum::<u64>()).sum();
    let s: u64 = stats.iter().map(|c| c.iter().map(|w| w[1]).sum::<u64>()).sum();
    println!("POOLED {s}/{b} = {:.1}%", 100.0 * s as f64 / b.max(1) as f64);
}
