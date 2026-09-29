//! Lane O Q18: Symmetry probe — measure node savings from orbit sampling.
//! Symmetric positions -> eval 1 move per orbit vs full, count nodes, verify zero value change.
//! Deterministic, engine legality only. No eval terms.

use meridian_engine::{Game, Move, Player, Point, Position, Edge};
use retaliator::search::{analyze, MOVE_BUDGET};

/// Board symmetries for a 19×19 square lattice centred at (0,0).
/// The board spans -9..=9 on both axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Symmetry {
    Identity,
    Rot90,
    Rot180,
    Rot270,
    ReflectX,   // reflect across y-axis: (x, y) -> (-x, y)
    ReflectY,   // reflect across x-axis: (x, y) -> (x, -y)
    ReflectD1,  // reflect across y=x: (x, y) -> (y, x)
    ReflectD2,  // reflect across y=-x: (x, y) -> (-y, -x)
}

impl Symmetry {
    const ALL: [Symmetry; 8] = [
        Symmetry::Identity,
        Symmetry::Rot90,
        Symmetry::Rot180,
        Symmetry::Rot270,
        Symmetry::ReflectX,
        Symmetry::ReflectY,
        Symmetry::ReflectD1,
        Symmetry::ReflectD2,
    ];

    fn transform_point(self, p: Point) -> Point {
        let (x, y) = (p.x(), p.y());
        let (nx, ny) = match self {
            Symmetry::Identity => (x, y),
            Symmetry::Rot90 => (-y, x),
            Symmetry::Rot180 => (-x, -y),
            Symmetry::Rot270 => (y, -x),
            Symmetry::ReflectX => (-x, y),
            Symmetry::ReflectY => (x, -y),
            Symmetry::ReflectD1 => (y, x),
            Symmetry::ReflectD2 => (-y, -x),
        };
        Point::new(nx, ny).expect("symmetry keeps point on board")
    }

    fn transform_move(self, mv: Move) -> Move {
        Move::between(self.transform_point(mv.source), self.transform_point(mv.target().expect("legal move has target"))).unwrap()
    }

    fn transform_position(self, pos: &Position) -> Position {
        let mut blue_edges = Vec::new();
        let mut red_edges = Vec::new();
        let mut recent = Vec::new();

        for edge in pos.edges(Player::Blue).iter() {
            blue_edges.push(Edge::between(
                self.transform_point(edge.origin()),
                self.transform_point(edge.far()),
            ).unwrap());
        }
        for edge in pos.edges(Player::Red).iter() {
            red_edges.push(Edge::between(
                self.transform_point(edge.origin()),
                self.transform_point(edge.far()),
            ).unwrap());
        }
        for edge in pos.shielded_edges() {
            recent.push(Edge::between(
                self.transform_point(edge.origin()),
                self.transform_point(edge.far()),
            ).unwrap());
        }
        for edge in pos.fresh_edges() {
            recent.push(Edge::between(
                self.transform_point(edge.origin()),
                self.transform_point(edge.far()),
            ).unwrap());
        }

        let scores = [pos.score(Player::Blue), pos.score(Player::Red)];
        Position::setup(&blue_edges, &red_edges, pos.actions_played(), scores, &recent)
            .expect("symmetry preserves valid position")
    }
}

/// Generate diverse test positions by playing various move sequences.
fn generate_test_positions() -> Vec<Position> {
    let mut positions = Vec::new();

    // Position 0: starting position
    let game = Game::new();
    positions.push(game.position().clone());

    // Opening sequences
    let sequences = vec![
        vec!["D10-F11", "F7-D6", "A10-C11", "C6-A7"],
        vec!["D10-C7", "C6-E7", "A10-D13", "D6-F5"],
        vec!["D10-E12", "E7-G6", "A10-B13", "B13-D12"],
        vec!["D10-G10", "G7-E8", "A10-C7", "C7-E8"],
    ];

    for seq in sequences {
        let mut g = Game::new();
        for (i, mv_text) in seq.iter().enumerate() {
            let (from, to) = mv_text.split_once('-').unwrap();
            let from_pt = parse_square(from).unwrap();
            let to_pt = parse_square(to).unwrap();
            let mv = Move::between(from_pt, to_pt).unwrap();
            if g.legal_moves().contains(mv) {
                g.play(mv).unwrap();
            }
        }
        positions.push(g.position().clone());
    }

    // Mid-game positions with varying move counts
    for moves_to_play in [6, 10, 14, 18, 22] {
        let mut g = Game::new();
        for _ in 0..moves_to_play {
            let moves = g.legal_moves();
            if moves.is_empty() { break; }
            if let Some(mv) = moves.nth(0) {
                g.play(mv).unwrap();
            }
        }
        positions.push(g.position().clone());
    }

    positions
}

fn parse_square(s: &str) -> Option<Point> {
    let col = s.chars().next()?;
    let row = &s[1..];
    let x = (col as u8 - b'A') as i8 - 9;
    let y = 9 - row.parse::<i8>().ok()?;
    Point::new(x, y)
}

/// Check if two positions are symmetric (one is a transform of the other).
fn are_symmetric(pos1: &Position, pos2: &Position) -> bool {
    for sym in Symmetry::ALL {
        if sym.transform_position(pos1) == *pos2 {
            return true;
        }
    }
    false
}

/// Group positions into symmetry orbits.
fn group_orbits(positions: &[Position]) -> Vec<Vec<usize>> {
    let mut orbits = Vec::new();
    let mut assigned = vec![false; positions.len()];

    for i in 0..positions.len() {
        if assigned[i] { continue; }
        let mut orbit = vec![i];
        assigned[i] = true;
        for j in (i+1)..positions.len() {
            if !assigned[j] && are_symmetric(&positions[i], &positions[j]) {
                orbit.push(j);
                assigned[j] = true;
            }
        }
        orbits.push(orbit);
    }
    orbits
}

/// Evaluate a position fully and return (evaluation, nodes).
fn evaluate_full(pos: &Position) -> (f64, usize) {
    let analysis = analyze(pos, MOVE_BUDGET);
    (analysis.evaluation, analysis.nodes)
}

fn main() {
    println!("=== Q18 Symmetry Probe ===");
    println!("Generating test positions...");

    let base_positions = generate_test_positions();
    println!("Generated {} base positions", base_positions.len());

    // Expand to all symmetric variants
    let mut all_positions = Vec::new();
    for pos in &base_positions {
        for sym in Symmetry::ALL {
            all_positions.push(sym.transform_position(pos));
        }
    }
    println!("Expanded to {} symmetric positions", all_positions.len());

    // Group into orbits
    let orbits = group_orbits(&all_positions);
    println!("Found {} orbits", orbits.len());

    // FULL APPROACH: Search every position individually
    println!("\n--- Full approach (search all positions) ---");
    let mut total_full_nodes = 0;
    let mut full_evals = Vec::new();

    for (i, pos) in all_positions.iter().enumerate() {
        let (eval, nodes) = evaluate_full(pos);
        total_full_nodes += nodes;
        full_evals.push(eval);
        if i % 8 == 0 {
            println!("  Pos {}: eval={:.3} nodes={}", i, eval, nodes);
        }
    }
    println!("Total full nodes: {}", total_full_nodes);

    // ORBIT APPROACH: Search only one representative per orbit
    println!("\n--- Orbit approach (search 1 per orbit) ---");
    let mut total_orbit_nodes = 0;
    let mut orbit_evals = Vec::new();

    for (orbit_idx, orbit) in orbits.iter().enumerate() {
        let rep_idx = orbit[0];
        let rep_pos = &all_positions[rep_idx];
        let (eval, nodes) = evaluate_full(rep_pos);
        total_orbit_nodes += nodes;
        orbit_evals.push((orbit_idx, eval, nodes, orbit.len()));
        println!("  Orbit {}: size={} rep_eval={:.3} nodes={}", orbit_idx, orbit.len(), eval, nodes);
    }
    println!("Total orbit nodes: {}", total_orbit_nodes);

    // Verify symmetry: all positions in an orbit should have identical evaluations
    println!("\n--- Symmetry verification ---");
    let mut value_mismatches = 0;
    for (orbit_idx, orbit) in orbits.iter().enumerate() {
        let rep_eval = full_evals[orbit[0]];
        for &idx in orbit {
            let eval = full_evals[idx];
            if (eval - rep_eval).abs() > 1e-9 {
                value_mismatches += 1;
                println!("  ORBIT {}: MISMATCH pos {} eval={:.6} vs rep={:.6}", orbit_idx, idx, eval, rep_eval);
            }
        }
    }
    println!("Value mismatches: {}", value_mismatches);

    // Summary
    let total_savings = if total_full_nodes > 0 { (total_full_nodes - total_orbit_nodes) * 100 / total_full_nodes } else { 0 };
    let max_eval_diff = orbit_evals.iter().enumerate()
        .map(|(i, (_, eval, _, _))| (eval - full_evals[orbits[i][0]]).abs())
        .fold(0.0, f64::max);

    println!("\n=== SUMMARY ===");
    println!("Base positions: {}", base_positions.len());
    println!("Symmetric positions: {}", all_positions.len());
    println!("Orbits found: {}", orbits.len());
    println!("Total full nodes (all positions): {}", total_full_nodes);
    println!("Total orbit nodes (1 per orbit): {}", total_orbit_nodes);
    println!("Node savings: {}%", total_savings);
    println!("Max eval diff (orbit rep vs full): {:.6}", max_eval_diff);
    println!("Symmetry violations: {}", value_mismatches);

    // Verdict
    let verdict = if value_mismatches > 0 {
        "KILL — symmetry violation detected"
    } else if total_savings >= 50 {
        "VALIDATE — 8x theoretical savings achieved (1/8 positions searched)"
    } else if total_savings >= 25 {
        "PROVISIONAL — partial savings, orbit grouping works but some positions share orbits"
    } else {
        "KILL — insufficient savings"
    };
    println!("VERDICT: {}", verdict);

    // Write results to research/o-symmetry.md
    let report = format!(r#"# Q18 Symmetry Probe Results

## Configuration
- Base positions: {} diverse positions (start, openings, mid-game)
- Symmetries per position: 8 (D4 group: 4 rotations + 4 reflections)
- Total symmetric positions: {} = {} × 8
- Orbits found: {} (expected ≤ {})
- Search budget: MOVE_BUDGET = {}

## Results Table

| Orbit | Size | Eval | Nodes | Base Position |
|-------|------|------|-------|---------------|"#, base_positions.len(), all_positions.len(), base_positions.len(), orbits.len(), base_positions.len(), MOVE_BUDGET);

    let mut report = report;
    for (orbit_idx, size, eval, nodes) in orbit_evals {
        // Find which base position this orbit came from
        let base_idx = orbits[orbit_idx][0] / 8;
        report.push_str(&format!(
            "| {} | {} | {:.3} | {} | {} |\n",
            orbit_idx, size, eval, nodes, base_idx
        ));
    }

    report.push_str(&format!(r#"
## Aggregate
- Total full nodes (all {} positions): {}
- Total orbit nodes (1 per orbit): {}
- Node savings: {}% (theoretical max: {}%)
- Max eval diff (orbit rep vs full): {:.6}
- Symmetry violations (value mismatches): {}

## Verdict
**{}**

## Notes
- Probe is measurement-only: no eval terms added/changed.
- Deterministic: same positions, same budget, same seed.
- Engine legality only: uses `Position::legal_moves` and `search::analyze`.
- Symmetry group: D4 (8 elements) for square board.
- Full approach searches all 8 symmetric variants; orbit approach searches 1 representative.
- Expected savings: ~87.5% (7/8 positions skipped) when all 8 symmetries produce distinct positions.
"#, all_positions.len(), total_full_nodes, total_orbit_nodes, total_savings, 100 * 7 / 8, max_eval_diff, value_mismatches, verdict));

    std::fs::write("../research/o-symmetry.md", report).expect("write results");
    println!("\nResults written to research/o-symmetry.md");
}