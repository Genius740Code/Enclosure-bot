//! Tests for the Retaliator bot: protocol shape, legality, ABI round-trip.

use meridian_engine::{Game, Move, notation};
use serde_json::json;

use retaliator::{answer, search};

fn start_game() -> Game {
    Game::from_position(notation::parse_setup("[B:A10-D10 R:P10-S10]").expect("start parses"))
}

#[test]
fn move_reply_is_legal_on_start() {
    let game = start_game();
    let legal: Vec<usize> = game.legal_moves().iter().map(|mv| mv.index()).collect();
    assert!(!legal.is_empty());
    let reply = answer(&json!({
        "type": "move",
        "start": "[B:A10-D10 R:P10-S10]",
        "moves": [],
        "limits": {"moveTimeMs": 20000},
        "seed": 7,
    }));
    let id = reply["move"].as_u64().expect("reply has a move") as usize;
    assert!(legal.contains(&id), "move {id} must be legal");
}

#[test]
fn move_reply_is_legal_mid_game() {
    // A real line: blue opens D10-D13, red answers; then it is blue's second action.
    let mut game = start_game();
    for id in [16058usize] {
        let mv = Move::from_index(id).expect("move id");
        game.play(mv).expect("line is legal");
    }
    let moves: Vec<usize> = game.moves().iter().map(|mv| mv.index()).collect();
    let legal: Vec<usize> = game.legal_moves().iter().map(|mv| mv.index()).collect();
    let reply = answer(&json!({
        "type": "move",
        "start": "[B:A10-D10 R:P10-S10]",
        "moves": moves,
        "limits": {"moveTimeMs": 20000},
        "seed": 7,
    }));
    let id = reply["move"].as_u64().expect("reply has a move") as usize;
    assert!(legal.contains(&id), "move {id} must be legal");
}

#[test]
fn analysis_reply_shape() {
    let game = start_game();
    let legal: Vec<usize> = game.legal_moves().iter().map(|mv| mv.index()).collect();
    let reply = answer(&json!({
        "type": "analysis",
        "start": "[B:A10-D10 R:P10-S10]",
        "moves": [],
        "limits": {"moveTimeMs": 30000, "visits": 240},
        "seed": 7,
    }));
    let a = &reply["analysis"];
    assert_eq!(a["unit"], "points");
    let cands = a["candidates"].as_array().expect("candidates");
    assert!(!cands.is_empty() && cands.len() <= 32);
    for c in cands {
        let id = c["move"].as_u64().expect("candidate move") as usize;
        assert!(legal.contains(&id), "candidate {id} must be legal");
        let pv = c["pv"].as_array().expect("pv");
        assert!(!pv.is_empty() && pv.len() <= 30);
        assert_eq!(pv[0].as_u64().unwrap() as usize, id, "pv starts with the move");
    }
    // The pv must stay legal when replayed.
    let mut g = start_game();
    let pv_ids: Vec<usize> =
        cands[0]["pv"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
    for id in pv_ids {
        g.play(Move::from_index(id).expect("pv id")).expect("pv stays legal");
    }
}

#[test]
fn cut_history_still_answers_legally() {
    // A real cut-containing line (from the Great Barrier loss): the replay
    // must collect avoid points and still return a legal move.
    let d: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("/tmp/opencode/game-f4b7f187-37e3-403d-a578-70f6cfd0cda1.json").unwrap(),
    )
    .unwrap();
    let all: Vec<usize> =
        d["moves"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
    let moves = &all[..40];
    let mut game = start_game();
    for &id in moves {
        game.play(Move::from_index(id).unwrap()).unwrap();
    }
    let legal: Vec<usize> = game.legal_moves().iter().map(|mv| mv.index()).collect();
    let reply = answer(&json!({
        "type": "move",
        "start": "[B:A10-D10 R:P10-S10]",
        "moves": moves,
        "limits": {"moveTimeMs": 20000},
        "seed": 7,
    }));
    let id = reply["move"].as_u64().expect("reply has a move") as usize;
    assert!(legal.contains(&id), "move {id} must be legal with cut history");
}

#[test]
fn bad_requests_error_cleanly() {
    let reply = answer(&json!({"type": "move"}));
    assert!(reply["error"].is_string());
    let reply = answer(&json!({
        "type": "move",
        "start": "[B:A10-D10 R:P10-S10]",
        "moves": [99999],
        "limits": {"moveTimeMs": 20000},
        "seed": 1,
    }));
    assert!(reply["error"].is_string());
}

#[test]
fn search_finds_a_move_quickly() {
    use std::time::Instant;
    let game = start_game();
    let t = Instant::now();
    let mv = search::best_move(game.position()).expect("a move");
    assert!(game.legal_moves().contains(mv));
    assert!(t.elapsed().as_secs() < 20, "move budget must fit the site limit");
}
