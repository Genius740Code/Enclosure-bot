//! Retaliator, a WebAssembly bot for Constellation.
//!
//! The site sends a request as JSON and reads a JSON reply. `"type": "move"` asks for a move and
//! `"type": "analysis"` for the best moves with their values. Both give the position as `start`
//! and the `moves` played since. This file reads requests and writes replies, and ends with the
//! three functions the site calls. The strategy is in `search.rs`.
//! https://constellation.blueshrimp.uk/bot-api.md has the details.

pub mod search;

use std::cell::RefCell;

use meridian_engine::{Game, Move, Point, notation};
use serde_json::{Value, json};

use search::Analysis;

/// How many moves an analysis lists.
const LISTED_CANDIDATES: usize = 5;
const DEFAULT_VISITS: u64 = 240;

/// The reply to one request: a move, an analysis, or an error.
pub fn answer(request: &Value) -> Value {
    reply(request).unwrap_or_else(|error| json!({"error": error}))
}

fn reply(request: &Value) -> Result<Value, String> {
    let (game, avoid) = replay(request)?;
    match request["type"].as_str() {
        Some("move") => {
            // v6-mesh: route through the mesh-prefix entry so the forced
            // opening plays on site; history comes from the request's moves.
            let best = search::best_move_routed(game.position(), game.moves(), &avoid)
                .ok_or("The game is over.")?;
            Ok(json!({"move": best.index()}))
        }
        Some("analysis") => {
            let visits = request["limits"]["visits"].as_u64().unwrap_or(DEFAULT_VISITS).min(search::MOVE_BUDGET as u64);
            Ok(json!({"analysis": analysis_json(&search::analyze_with_avoid(game.position(), visits as usize, &avoid))}))
        }
        _ => Err("type must be move or analysis".into()),
    }
}

/// The request's position (`start`, then the move IDs in `moves`), plus the
/// points near our loops the enemy cut in the last few actions — ground the
/// search routes away from instead of rebuilding on.
fn replay(request: &Value) -> Result<(Game, Vec<Point>), String> {
    let start = request["start"].as_str().ok_or("start is required: a position such as [B:A10-D10 R:P10-S10]")?;
    let mut game = Game::from_position(notation::parse_setup(start).map_err(|error| error.to_string())?);
    let moves = request["moves"].as_array().ok_or("moves is required: an array of move IDs")?;
    let mut ids = Vec::with_capacity(moves.len());
    for id in moves {
        let mv = id.as_u64().and_then(|id| usize::try_from(id).ok()).and_then(Move::from_index);
        ids.push(mv.ok_or(format!("{id} is not a move ID"))?);
    }
    // Our edges the enemy cut near the end of the line: rebuilding there is farmed.
    // `broken` is always the mover's opponent's edge; collect them with their
    // action index, then keep the recent cuts of our (final) side's edges.
    let mut cuts = Vec::new();
    for (i, &mv) in ids.iter().enumerate() {
        let mover = game.position().to_move();
        let outcome = game.play(mv).map_err(|error| error.to_string())?;
        if let Some(cut) = outcome.broken {
            cuts.push((i, mover, cut));
        }
    }
    let ours = game.position().to_move();
    let mut avoid = Vec::new();
    for (i, mover, cut) in cuts.iter().rev() {
        if ids.len() - i > search::CUT_MEMORY {
            break;
        }
        if mover.opponent() == ours {
            avoid.push(cut.origin());
            avoid.push(cut.far());
        }
    }
    Ok((game, avoid))
}

fn analysis_json(analysis: &Analysis) -> Value {
    let candidates: Vec<Value> = analysis
        .candidates
        .iter()
        .take(LISTED_CANDIDATES)
        .map(|candidate| {
            json!({
                "move": candidate.mv.index(),
                "evaluation": candidate.evaluation,
                "pv": candidate.pv.iter().map(|mv| mv.index()).collect::<Vec<_>>(),
                "visits": candidate.visits,
            })
        })
        .collect();
    json!({
        "evaluation": analysis.evaluation,
        "unit": "points",
        "depth": analysis.depth,
        "nodes": analysis.nodes,
        "candidates": candidates,
    })
}

// How the site calls the module: version 1 of the bot interface.

thread_local! {
    /// The latest reply: its length as a little-endian u32, then the JSON. It stays until the next
    /// request.
    static REPLY: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn meridian_abi() -> u32 {
    1
}

/// Room for a request of `len` bytes, which the site then writes there.
#[unsafe(no_mangle)]
pub extern "C" fn meridian_alloc(len: usize) -> *mut u8 {
    std::mem::ManuallyDrop::new(Vec::<u8>::with_capacity(len)).as_mut_ptr()
}

/// Answers the request the site wrote, and returns where the reply is.
///
/// # Safety
/// `ptr` and `len` must be a buffer from [`meridian_alloc`], holding the request.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn meridian_run(ptr: *mut u8, len: usize) -> *const u8 {
    let request = unsafe { Vec::from_raw_parts(ptr, len, len) };
    let reply = match serde_json::from_slice(&request) {
        Ok(request) => answer(&request),
        Err(error) => json!({"error": format!("Invalid JSON: {error}")}),
    };
    let body = reply.to_string().into_bytes();
    REPLY.with_borrow_mut(|out| {
        out.clear();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out.as_ptr()
    })
}
