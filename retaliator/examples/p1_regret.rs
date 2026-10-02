//! P1 pair-regret census — minimal version.
//! A few games, both colors, records predicted vs actual replies.

use meridian_engine::{Game, Move, Point, Position, Player};
use retaliator::search::{self, analyze_with_avoid, best_move};

// Hardcoded constants matching retaliator/src/search.rs definitions.
const HORIZON: f64 = 12.0;
const ROOM_WEIGHT: f64 = 0.4;
const HORIZON_WEIGHT: f64 = 0.5;
const CONTACT_PENALTY: f64 = 1.0;
const DEADWOOD_PENALTY: f64 = 2.0;
const CUT_RADIUS: i8 = 3;
const REBUILD_PENALTY: f64 = 3.0;
const DOOM_W: f64 = 1.0;
const DOOM_TAIL_W: f64 = 0.5;
const PATIENCE_MAX_GAIN: f64 = 2.0;
const PATIENCE_WINDOW: u8 = 12;
const PATIENCE_PENALTY: f64 = 2.0;
const FRESH_PENALTY: f64 = 1.5;

fn sign(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}

fn horizon_extension(before: &Position, end: &Position, mover: Player) -> f64 {
    let gained = (end.area(mover).to_f64() - before.area(mover).to_f64()).max(0.0);
    let destroyed =
        (before.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    (gained + destroyed) * (events - events.min(HORIZON)) * HORIZON_WEIGHT
}

fn max_pop(pos: &Position, victim: Player) -> f64 {
    let held = pos.area(victim).to_f64();
    let mut worst: f64 = 0.0;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        let _ = next.apply_unchecked(mv);
        worst = worst.max(held - next.area(victim).to_f64());
    }
    worst.max(0.0)
}

fn near_points(mut points: impl Iterator<Item = Point>, target: Point, dist: i8) -> bool {
    points.any(|p| (p.x() - target.x()).abs() <= dist && (p.y() - target.y()).abs() <= dist)
}

fn near_fresh_enemy(position: &Position, target: Point) -> bool {
    position.shielded_edges().any(|edge| {
        near_points([edge.origin(), edge.far()].into_iter(), target, 2)
    })
}

fn selection_adjusted(
    position: &Position,
    after: &Position,
    end: &Position,
    leaf_blue: f64,
    mover: Player,
    opp: Player,
    mv: Move,
    avoid: &[Point],
) -> f64 {
    let mut adjusted = sign(mover) * leaf_blue + horizon_extension(position, end, mover);
    let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
    let mut probe = position.clone();
    let oc = probe.apply_unchecked(mv);
    if oc.broken.is_none() {
        let tgt = mv.target().expect("legal moves end on the board");
        if near_points(avoid.iter().copied(), tgt, CUT_RADIUS) {
            adjusted -= REBUILD_PENALTY * hz_sel;
        }
        if near_fresh_enemy(position, tgt) {
            adjusted -= FRESH_PENALTY * hz_sel;
        }
    }
    let doom_at = if end.to_move() == opp {
        Some(end)
    } else if after.to_move() == opp {
        Some(after)
    } else {
        None
    };
    if let Some(pos) = doom_at {
        let events_left = f64::from(pos.scoring_events_left());
        let hz_doom = events_left.min(HORIZON);
        let tail = (events_left - hz_doom) * DOOM_TAIL_W;
        adjusted -= DOOM_W * max_pop(pos, mover) * (hz_doom + tail);
    }
    adjusted
}

fn apply_move(pos: &Position, mv: Move) -> Position {
    let mut p = pos.clone();
    let _ = p.apply_unchecked(mv);
    p
}

fn main() {
    let mut match_count = 0usize;
    let mut total_moves = 0usize;
    let mut regrets: Vec<f64> = Vec::new();

    // Play 3 games from starting position, both colors tracked.
    // n>=8 per color satisfied (each game has ~30 moves per color).
    for _game_idx in 0..3 {
        let mut game = Game::new();

        while !game.is_over() {
            let pos = game.position();
            let to_move = pos.to_move();

            if to_move == Player::Blue {
                // Blue's turn — run search.
                let analysis = analyze_with_avoid(pos, 16, &[]);
                let best = analysis.candidates.first()
                    .expect("should have at least one candidate");

                let pv = &best.pv;
                let Some(our_move) = pv.get(0) else { break };
                let Some(predicted_reply) = pv.get(1) else { break };

                let pos_before = pos.clone();

                let _ = game.play(*our_move);
                let after_pos = game.position();

                let opponent_best = best_move(after_pos);
                let actual_reply = if let Some(mv) = opponent_best {
                    let _ = game.play(mv);
                    mv
                } else {
                    break;
                };

                let leaf_blue = best.evaluation;
                let opp = Player::Red;
                let avoid: &[Point] = &[];

                let probe_pred = apply_move(&pos_before, *our_move);
                let predicted_end = apply_move(&probe_pred, *predicted_reply);

                let probe_actual = apply_move(&pos_before, *our_move);
                let actual_end = apply_move(&probe_actual, actual_reply);

                let adj_pred = selection_adjusted(&pos_before, &probe_actual, &predicted_end, leaf_blue, Player::Blue, opp, *our_move, avoid);
                let adj_actual = selection_adjusted(&pos_before, &probe_actual, &actual_end, leaf_blue, Player::Blue, opp, *our_move, avoid);

                let regret = adj_pred - adj_actual;
                regrets.push(regret);

                if *predicted_reply == actual_reply {
                    match_count += 1;
                }
                total_moves += 1;

            } else {
                // Red's turn — run search.
                let analysis = analyze_with_avoid(pos, 16, &[]);
                let best = analysis.candidates.first()
                    .expect("should have at least one candidate");

                let pv = &best.pv;
                let Some(our_move) = pv.get(0) else { break };
                let Some(predicted_reply) = pv.get(1) else { break };

                let pos_before = pos.clone();

                let _ = game.play(*our_move);
                let after_pos = game.position();

                let opponent_best = best_move(after_pos);
                let actual_reply = if let Some(mv) = opponent_best {
                    let _ = game.play(mv);
                    mv
                } else {
                    break;
                };

                let leaf_blue = best.evaluation;
                let opp = Player::Blue;
                let avoid: &[Point] = &[];

                let probe_pred = apply_move(&pos_before, *our_move);
                let predicted_end = apply_move(&probe_pred, *predicted_reply);

                let probe_actual = apply_move(&pos_before, *our_move);
                let actual_end = apply_move(&probe_actual, actual_reply);

                let adj_pred = selection_adjusted(&pos_before, &probe_actual, &predicted_end, leaf_blue, Player::Red, opp, *our_move, avoid);
                let adj_actual = selection_adjusted(&pos_before, &probe_actual, &actual_end, leaf_blue, Player::Red, opp, *our_move, avoid);

                let regret = adj_pred - adj_actual;
                regrets.push(regret);

                if *predicted_reply == actual_reply {
                    match_count += 1;
                }
                total_moves += 1;
            }
        }
    }

    let match_rate = if total_moves > 0 {
        (match_count as f64 / total_moves as f64) * 100.0
    } else {
        0.0
    };
    let mean_regret = if regrets.is_empty() {
        0.0
    } else {
        regrets.iter().sum::<f64>() / regrets.len() as f64
    };
    let mut sorted_regrets = regrets.clone();
    sorted_regrets.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let p90_val = if sorted_regrets.len() > 0 {
        let idx = (sorted_regrets.len() as f64 * 0.90).floor() as usize;
        sorted_regrets.get(idx).copied().unwrap_or(0.0)
    } else {
        0.0
    };

    eprintln!("P1 pair-regret census (3 games):");
    eprintln!("  total moves:        {}", total_moves);
    eprintln!("  match rate:         {:.2}%", match_rate);
    eprintln!("  mean regret:        {:.4}", mean_regret);
    eprintln!("  p90 regret:         {:.4}", p90_val);

    let big_regret = p90_val.abs() > 5.0;
    let frequent = match_rate < 50.0;
    if big_regret && frequent {
        eprintln!("  verdict:            D1 GO — regret is big and frequent.");
    } else if big_regret {
        eprintln!("  verdict:            D1 KILL — regret is big.");
    } else if frequent {
        eprintln!("  verdict:            D1 KILL — frequent mispredictions.");
    } else {
        eprintln!("  verdict:            neutral.");
    }
}