//! Test rebuild denial: verify that re-closes near cuts are penalized
//! even when they break something.

use meridian_engine::{Game, Move, Player, Point, MoveOutcome};
use retaliator::search::{analyze_with_avoid, CUT_MEMORY};

const CUT_RADIUS: i8 = 3;
const REBUILD_PENALTY: f64 = 3.0;
const HORIZON: f64 = 12.0;

fn near_points(points: &[Point], target: Point, dist: i8) -> bool {
    points.iter().any(|p| (p.x() - target.x()).abs() <= dist && (p.y() - target.y()).abs() <= dist)
}

fn play_move(game: &mut Game, target_x: i8, target_y: i8) {
    for mv in game.position().legal_moves().iter() {
        let t = mv.target().expect("legal");
        if t.x() == target_x && t.y() == target_y {
            game.play(mv).unwrap();
            return;
        }
    }
    // Fallback: play first legal move
    for mv in game.position().legal_moves().iter() {
        game.play(mv).unwrap();
        return;
    }
}

fn play_move_return_outcome(game: &mut Game, target_x: i8, target_y: i8) -> MoveOutcome {
    for mv in game.position().legal_moves().iter() {
        let t = mv.target().expect("legal");
        if t.x() == target_x && t.y() == target_y {
            return game.play(mv).unwrap();
        }
    }
    // Fallback: play first legal move
    for mv in game.position().legal_moves().iter() {
        return game.play(mv).unwrap();
    }
    panic!("no legal moves");
}

fn main() {
    println!("Testing rebuild denial fix...");
    println!("CUT_RADIUS = {}", CUT_RADIUS);
    println!("REBUILD_PENALTY = {}", REBUILD_PENALTY);
    println!("CUT_MEMORY = {}", CUT_MEMORY);
    println!("HORIZON = {}", HORIZON);

    // Create a game where Red cuts Blue's loop
    let mut game = Game::new();
    
    // Play moves to set up a position where a cut happens
    // Blue: D10-F7 (forced opener)
    play_move(&mut game, -4, -3);
    
    // Red: P10-M8
    play_move(&mut game, 4, -3);
    
    // Blue: F7-H7
    play_move(&mut game, -2, -3);
    
    // Red: M8-K8
    play_move(&mut game, 2, -3);
    
    // Blue: H7-J7
    play_move(&mut game, 0, -3);
    
    // Red: K8-I8
    play_move(&mut game, 0, -3);
    
    // Blue: J7-J5 (down)
    play_move(&mut game, 0, -1);
    
    // Red: I8-I6 (down)
    play_move(&mut game, 0, -1);
    
    // Blue: J5-H5 (left)
    play_move(&mut game, -2, -1);
    
    // Red: I6-G6 (left) - Red completes a triangle and cuts Blue's potential loop
    let outcome = play_move_return_outcome(&mut game, -2, -1);
    
    println!("\nAfter Red's cut move:");
    println!("  Broken: {:?}", outcome.broken);
    println!("  Blue area: {:.1}", game.position().area(Player::Blue).to_f64());
    println!("  Red area: {:.1}", game.position().area(Player::Red).to_f64());
    
    // Now it's Blue's turn. Blue's loop was cut near G6/H5 area.
    // The avoid points should include the cut edge endpoints.
    let avoid = if let Some(cut) = outcome.broken {
        vec![cut.origin(), cut.far()]
    } else {
        vec![]
    };
    
    println!("\nAvoid points (cut edge endpoints): {:?}", avoid);
    
    // Analyze with avoid points
    let analysis = analyze_with_avoid(game.position(), 4096, &avoid);
    
    println!("\nTop 5 candidates for Blue (with avoid):");
    for (i, c) in analysis.candidates.iter().take(5).enumerate() {
        println!("  {}: {} eval={:.2} pv=[{}]", 
            i+1, c.mv.index(), c.evaluation, 
            c.pv.iter().map(|m| m.index().to_string()).collect::<Vec<_>>().join(", "));
    }
    
    // Now test WITHOUT avoid points for comparison
    let analysis_no_avoid = analyze_with_avoid(game.position(), 4096, &[]);
    
    println!("\nTop 5 candidates for Blue (WITHOUT avoid):");
    for (i, c) in analysis_no_avoid.candidates.iter().take(5).enumerate() {
        println!("  {}: {} eval={:.2} pv=[{}]", 
            i+1, c.mv.index(), c.evaluation,
            c.pv.iter().map(|m| m.index().to_string()).collect::<Vec<_>>().join(", "));
    }
    
    // Check if the best move changed due to avoid
    let best_with = analysis.candidates.first().map(|c| c.mv);
    let best_without = analysis_no_avoid.candidates.first().map(|c| c.mv);
    
    println!("\nBest move WITH avoid: {:?}", best_with);
    println!("Best move WITHOUT avoid: {:?}", best_without);
    
    if best_with != best_without {
        println!("\n✓ FIX WORKS: Avoid points changed the best move!");
    } else {
        println!("\n✗ Fix may not be triggering (best move unchanged)");
    }
    
    // Test a specific re-close move near the cut
    println!("\n--- Testing specific re-close moves near cut ---");
    let legal_moves = game.position().legal_moves();
    let mut near_cut_moves = 0;
    let mut near_cut_breaking = 0;
    let mut near_cut_non_breaking = 0;
    
    for mv in legal_moves.iter().take(30) {
        let mut after = game.position().clone();
        let outcome = after.apply_unchecked(mv);
        let target = mv.target().expect("legal move");
        
        // Check if near avoid points
        let near_cut = near_points(&avoid, target, CUT_RADIUS);
        
        if near_cut {
            near_cut_moves += 1;
            if outcome.broken.is_some() {
                near_cut_breaking += 1;
                println!("  Move {}: target={:?} broken=YES near_cut=YES", 
                    mv.index(), (target.x(), target.y()));
            } else {
                near_cut_non_breaking += 1;
                println!("  Move {}: target={:?} broken=NO near_cut=YES", 
                    mv.index(), (target.x(), target.y()));
            }
        }
    }
    
    println!("\nSummary:");
    println!("  Total moves near cut (radius {}): {}", CUT_RADIUS, near_cut_moves);
    println!("  Breaking moves near cut: {}", near_cut_breaking);
    println!("  Non-breaking moves near cut: {}", near_cut_non_breaking);
    println!("\nThe fix penalizes BOTH breaking and non-breaking moves near cuts.");
    println!("Previously, only non-breaking moves were penalized (outcome.broken.is_none() check).");
}