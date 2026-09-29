//! playbook_miner.rs - Detailed game analysis for VladNet and GB/GB2 games.

use std::fs;
use std::path::Path;
use meridian_engine::{Game, Move, Player, Point};
use retaliator::search::{analyze_with_avoid, MOVE_BUDGET};

#[derive(Debug, Clone)]
struct MoveEvent {
    action: usize,
    player: Player,
    is_rival: bool,
    mv: Move,
    notation: String,
    kind: String,
    blue_area: f64,
    red_area: f64,
    rival_area: f64,
    our_area: f64,
    rival_gain: f64,
    our_gain: f64,
    opp_broken_loss: Option<f64>, // area lost by opponent due to this move
    own_broken_loss: Option<f64>, // area lost by mover due to this move
}

struct GameReport {
    id: String,
    rival_name: String,
    rival_color: Player,
    our_color: Player,
    winner: String,
    blue_score: f64,
    red_score: f64,
    events: Vec<MoveEvent>,
}

fn analyze_file(path: &str, rival_name: &str) -> GameReport {
    let s = fs::read_to_string(path).expect("read game json");
    let d: serde_json::Value = serde_json::from_str(&s).expect("parse json");
    let full_id = d["id"].as_str().unwrap().to_string();
    let blue_bot = d["blue"].as_str().unwrap_or("").to_string();
    let red_bot = d["red"].as_str().unwrap_or("").to_string();

    let (rival_color, our_color) = if blue_bot.to_lowercase().contains(&rival_name.to_lowercase()) {
        (Player::Blue, Player::Red)
    } else {
        (Player::Red, Player::Blue)
    };

    let move_indices: Vec<usize> = d["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();

    let mut game = Game::new();
    let mut events = Vec::new();

    for (i, &idx) in move_indices.iter().enumerate() {
        let action = i + 1;
        let mover = game.position().to_move();
        let is_rival = mover == rival_color;

        let b0 = game.position().area(Player::Blue).to_f64();
        let r0 = game.position().area(Player::Red).to_f64();

        let mv = Move::from_index(idx).expect("valid move index");
        let notation = format!("{}-{}", mv.source, mv.target().unwrap());

        let outcome = game.play(mv).expect("legal move");

        let b1 = game.position().area(Player::Blue).to_f64();
        let r1 = game.position().area(Player::Red).to_f64();

        let rival_area = if rival_color == Player::Blue { b1 } else { r1 };
        let our_area = if our_color == Player::Blue { b1 } else { r1 };

        let rival_prev = if rival_color == Player::Blue { b0 } else { r0 };
        let our_prev = if our_color == Player::Blue { b0 } else { r0 };

        let rival_gain = rival_area - rival_prev;
        let our_gain = our_area - our_prev;

        let (opp_broken_loss, own_broken_loss) = if outcome.broken.is_some() {
            if is_rival {
                // Rival broke something
                let opp_loss = our_prev - our_area;
                let own_loss = rival_prev - rival_area;
                (
                    if opp_loss > 0.001 { Some(opp_loss) } else { None },
                    if own_loss > 0.001 { Some(own_loss) } else { None },
                )
            } else {
                // We broke something
                let opp_loss = rival_prev - rival_area;
                let own_loss = our_prev - our_area;
                (
                    if opp_loss > 0.001 { Some(opp_loss) } else { None },
                    if own_loss > 0.001 { Some(own_loss) } else { None },
                )
            }
        } else {
            (None, None)
        };

        events.push(MoveEvent {
            action,
            player: mover,
            is_rival,
            mv,
            notation,
            kind: format!("{:?}", outcome.kind),
            blue_area: b1,
            red_area: r1,
            rival_area,
            our_area,
            rival_gain,
            our_gain,
            opp_broken_loss,
            own_broken_loss,
        });
    }

    let winner = d["result"].as_str().unwrap_or("unknown").to_string();
    let blue_score = game.position().score(Player::Blue).to_f64();
    let red_score = game.position().score(Player::Red).to_f64();

    GameReport {
        id: full_id,
        rival_name: if rival_color == Player::Blue { blue_bot } else { red_bot },
        rival_color,
        our_color,
        winner,
        blue_score,
        red_score,
        events,
    }
}

fn sample_reply_habits(path: &str, rival_color: Player, sample_actions: &[usize]) {
    let s = fs::read_to_string(path).expect("read game json");
    let d: serde_json::Value = serde_json::from_str(&s).expect("parse json");
    let move_indices: Vec<usize> = d["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();

    let mut game = Game::new();
    let mut avoid: Vec<Point> = Vec::new();

    for (i, &idx) in move_indices.iter().enumerate() {
        let action = i + 1;
        let mover = game.position().to_move();
        let mv = Move::from_index(idx).expect("valid move index");

        if mover == rival_color && sample_actions.contains(&action) {
            let analysis = analyze_with_avoid(game.position(), MOVE_BUDGET, &avoid);
            let greedy_cand = analysis.candidates.first();
            let greedy_mv = greedy_cand.map(|c| c.mv);
            let greedy_not = greedy_mv.map(|m| format!("{}-{}", m.source, m.target().unwrap())).unwrap_or("none".into());
            let actual_not = format!("{}-{}", mv.source, mv.target().unwrap());
            let matched = greedy_mv == Some(mv);
            let cand_len = analysis.candidates.len();
            println!(
                "  Action {:2} ({:?}): greedy_first={} actual={} matched={} (cand_count={})",
                action, mover, greedy_not, actual_not, if matched { "YES" } else { "NO" }, cand_len
            );
        }

        let outcome = game.play(mv).expect("legal move");
        if let Some(cut) = outcome.broken {
            if mover == rival_color {
                avoid.push(cut.origin());
                avoid.push(cut.far());
            }
        }
        if avoid.len() > 12 {
            avoid.drain(0..avoid.len() - 12);
        }
    }
}

fn print_quantified_analysis(rep: &GameReport) {
    let short_id = &rep.id[..8];
    println!("=======================================================");
    println!("GAME {} — Rival: {} ({:?}) vs Riposte v7 ({:?})", short_id, rep.rival_name, rep.rival_color, rep.our_color);
    println!("Result: Winner={}, Scores: Blue={:.1}, Red={:.1}", rep.winner, rep.blue_score, rep.red_score);
    println!("-------------------------------------------------------");

    // 1. Opener
    println!("1. OPENER:");
    println!("  First 6 game actions:");
    for ev in rep.events.iter().take(6) {
        let mover_str = if ev.is_rival { "RIVAL" } else { "US   " };
        println!("    Action {:2} [{:?} | {}]: {} ({}) -> b_area={:.1} r_area={:.1}",
            ev.action, ev.player, mover_str, ev.notation, ev.kind, ev.blue_area, ev.red_area);
    }
    println!("  Rival first 6 moves:");
    let rival_moves: Vec<_> = rep.events.iter().filter(|e| e.is_rival).take(6).collect();
    for ev in &rival_moves {
        println!("    Action {:2}: {} ({}) gain={:+.1} rival_tot_area={:.1}",
            ev.action, ev.notation, ev.kind, ev.rival_gain, ev.rival_area);
    }

    // 2. First wall move + size
    println!("2. FIRST WALL MOVE & MAJOR WALLS:");
    let first_wall = rep.events.iter().find(|e| e.is_rival && e.kind == "Connect" && e.rival_gain > 0.001);
    if let Some(fw) = first_wall {
        println!("    First wall close: Action {:2}, Move {}, gain={:.1} area, total rival area={:.1}",
            fw.action, fw.notation, fw.rival_gain, fw.rival_area);
    } else {
        println!("    First wall close: NONE");
    }
    let first_major = rep.events.iter().find(|e| e.is_rival && e.rival_gain >= 10.0);
    if let Some(fm) = first_major {
        println!("    First major bank (gain>=10): Action {:2}, Move {}, gain={:.1} area, total rival area={:.1}",
            fm.action, fm.notation, fm.rival_gain, fm.rival_area);
    } else {
        println!("    First major bank (gain>=10): NONE");
    }

    // 3. Break timing vs us
    println!("3. BREAK TIMING VS US:");
    let mut our_last_close = 0;
    let mut rival_breaks = 0;
    let mut rival_break_damage = 0.0;
    for ev in &rep.events {
        if !ev.is_rival && ev.kind == "Connect" && ev.our_gain > 0.001 {
            our_last_close = ev.action;
        }
        if ev.is_rival {
            if let Some(dmg) = ev.opp_broken_loss {
                rival_breaks += 1;
                rival_break_damage += dmg;
                let lag = if our_last_close > 0 { ev.action as isize - our_last_close as isize } else { -1 };
                println!("    Rival Break at Action {:2}: Move {} dealt -{:.1} loss to us (lag from our last close at {}: {} actions)",
                    ev.action, ev.notation, dmg, our_last_close, lag);
            }
        }
    }
    println!("    Total breaks dealt by rival: {}, total dmg=-{:.1}", rival_breaks, rival_break_damage);

    // Our breaks on rival
    let mut our_breaks = 0;
    let mut our_break_damage = 0.0;
    for ev in &rep.events {
        if !ev.is_rival {
            if let Some(dmg) = ev.opp_broken_loss {
                our_breaks += 1;
                our_break_damage += dmg;
                println!("    Our Break at Action {:2}: Move {} dealt -{:.1} loss to rival",
                    ev.action, ev.notation, dmg);
            }
        }
    }
    println!("    Total breaks dealt by us: {}, total dmg=-{:.1}", our_breaks, our_break_damage);

    // 4. Bank sizes at 40/60/80
    println!("4. BANK SIZES AT 40/60/80:");
    for &act in &[40, 60, 80] {
        if let Some(ev) = rep.events.get(act - 1) {
            println!("    At Action {:2}: Rival Area={:.1}, Our Area={:.1} (Blue={:.1}, Red={:.1})",
                act, ev.rival_area, ev.our_area, ev.blue_area, ev.red_area);
        }
    }

    // 5. Bank fragility
    println!("5. BANK FRAGILITY:");
    let cat_breaks_rival_suffered = rep.events.iter().filter(|e| !e.is_rival && e.opp_broken_loss.map_or(false, |d| d >= 10.0)).count();
    let cat_breaks_we_suffered = rep.events.iter().filter(|e| e.is_rival && e.opp_broken_loss.map_or(false, |d| d >= 10.0)).count();
    let peak_rival_area = rep.events.iter().map(|e| e.rival_area).fold(0.0, f64::max);
    let final_rival_area = rep.events.last().map(|e| e.rival_area).unwrap_or(0.0);
    let peak_our_area = rep.events.iter().map(|e| e.our_area).fold(0.0, f64::max);
    let final_our_area = rep.events.last().map(|e| e.our_area).unwrap_or(0.0);

    println!("    Rival bank: peak={:.1}, final={:.1}, breaks suffered={}, total area lost={:.1}, catastrophic breaks(>=10)={}",
        peak_rival_area, final_rival_area, our_breaks, our_break_damage, cat_breaks_rival_suffered);
    println!("    Our bank:   peak={:.1}, final={:.1}, breaks suffered={}, total area lost={:.1}, catastrophic breaks(>=10)={}",
        peak_our_area, final_our_area, rival_breaks, rival_break_damage, cat_breaks_we_suffered);
}

fn main() {
    let files = vec![
        ("research/games/3a414aad-5aeb-4c16-87c1-f89e87966b93.json", "VladNet"),
        ("research/games/1c69056c-dd8a-4dc4-b62c-4846d08e92c8.json", "VladNet"),
        ("research/games/30c7653b-6d57-43d7-a361-d6080b07e266.json", "Great Barrier"),
        ("research/games/ad65f054-597e-47f1-b1f8-84f0d8ca3c54.json", "Great Barrier"),
        ("research/games/d2d4b4fd-9713-4fdc-82d9-26232a31784a.json", "Great Barrier"),
    ];

    let mut reports = Vec::new();
    for (f, r) in &files {
        let rep = analyze_file(f, r);
        print_quantified_analysis(&rep);
        reports.push(rep);
    }

    println!("\n=======================================================");
    println!("6. REPLY HABITS (GREEDY-FIRST % ON 20 SAMPLED POSITIONS)");
    println!("=======================================================");

    println!("\n--- VladNet (20 sampled positions: 10 from 3a414aad, 10 from 1c69056c) ---");
    // Midgame actions where VladNet moves
    // 3a414aad (VladNet Red): moves on 2,3, 6,7, 10,11, 14,15, 18,19, 22,23, 26,27, 30,31, 34,35, 38,39, 42,43, 46,47, 50,51, 54,55
    let vlad_red_samples = vec![14, 18, 22, 26, 30, 34, 38, 42, 46, 50];
    sample_reply_habits("research/games/3a414aad-5aeb-4c16-87c1-f89e87966b93.json", Player::Red, &vlad_red_samples);

    // 1c69056c (VladNet Blue): moves on 1, 4,5, 8,9, 12,13, 16,17, 20,21, 24,25, 28,29, 32,33, 36,37, 40,41, 44,45, 48,49, 52,53
    let vlad_blue_samples = vec![12, 16, 20, 24, 28, 32, 36, 40, 44, 48];
    sample_reply_habits("research/games/1c69056c-dd8a-4dc4-b62c-4846d08e92c8.json", Player::Blue, &vlad_blue_samples);

    println!("\n--- Great Barrier (20 sampled positions across 30c7653b, ad65f054, d2d4b4fd) ---");
    // 30c7653b (GB Blue): 7 positions
    let gb_blue_samples = vec![12, 16, 24, 32, 40, 48, 56];
    println!("GB Blue (30c7653b):");
    sample_reply_habits("research/games/30c7653b-6d57-43d7-a361-d6080b07e266.json", Player::Blue, &gb_blue_samples);

    // ad65f054 (GB Red): 7 positions
    let gb_red_samples = vec![14, 22, 30, 38, 46, 54, 62];
    println!("GB Red (ad65f054):");
    sample_reply_habits("research/games/ad65f054-597e-47f1-b1f8-84f0d8ca3c54.json", Player::Red, &gb_red_samples);

    // d2d4b4fd (GB 2.0 Red): 6 positions
    let gb2_red_samples = vec![14, 22, 30, 38, 46, 54];
    println!("GB 2.0 Red (d2d4b4fd):");
    sample_reply_habits("research/games/d2d4b4fd-9713-4fdc-82d9-26232a31784a.json", Player::Red, &gb2_red_samples);
}
