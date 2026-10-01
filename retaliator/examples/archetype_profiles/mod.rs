// Archetype Reply Profiles — quantified from site games via API + replay
// 
// Four opponent archetypes discovered in M7 reply-model audit (lane-v8-audit):
// - WallBuilder (GB, GB2.0): corridor roots → shared-node walls → delayed close → late pop/break
// - PopTimer (VladNet): memorized opener → tiny loops from action 2 → 94% Lag 1-2 counter-punch cuts
// - SpaceGrabber (AngelWASM): bigger loops (5.8/loop), middleweight search, remote expansion
// - Mirror (v6/v7): our own search family — greedy when enemy=v6 (77.7%), surprises us when enemy=v7 (32.7%)

use meridian_engine::{Move, MoveKind, Player, Position, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Archetype {
    WallBuilder,    // GB, GB2.0
    PopTimer,       // VladNet
    SpaceGrabber,   // AngelWASM, spacebot
    Mirror,         // v6, v7 (our search family)
    Unknown,
}

impl Archetype {
    pub fn from_bot_name(name: &str) -> Self {
        let n = name.to_lowercase();
        if n.contains("great barrier") || n.contains("gb") {
            Archetype::WallBuilder
        } else if n.contains("vlad") {
            Archetype::PopTimer
        } else if n.contains("angel") || n.contains("space") {
            Archetype::SpaceGrabber
        } else if n.contains("riposte") || n.contains("v6") || n.contains("v7") || n.contains("scout") || n.contains("stompy") {
            Archetype::Mirror
        } else {
            Archetype::Unknown
        }
    }

    /// Baseline greedy match rate from M7 audit (lane-v8-audit, 1320 enemy turns)
    pub fn greedy_baseline(&self) -> f64 {
        match self {
            Archetype::WallBuilder => 0.375,    // GB 37.5%, GB2.0 32.5% → ~35%
            Archetype::PopTimer => 0.075,       // VladNet 7.5%
            Archetype::SpaceGrabber => 0.158,   // AngelWASM 15.8%
            Archetype::Mirror => 0.552,         // Mirror overall 55.2% (but split: v6=77.7%, v7=32.7%)
            Archetype::Unknown => 0.0,
        }
    }

    /// M7 audit match rates by phase
    pub fn phase_baselines(&self) -> (f64, f64, f64) { // (opening, mid, late)
        match self {
            Archetype::WallBuilder => (0.0, 0.208, 0.583),
            Archetype::PopTimer => (0.0, 0.146, 0.033),
            Archetype::SpaceGrabber => (0.083, 0.125, 0.200),
            Archetype::Mirror => (0.0, 0.608, 0.561), // enemy=v6: 75.0% mid, 63.0% late
            Archetype::Unknown => (0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArchetypeProfile {
    pub archetype: Archetype,
    pub bot_name: String,
    pub games_analyzed: usize,
    pub enemy_turns: usize,
    pub overall_match: f64,
    pub opening_match: f64,
    pub mid_match: f64,
    pub late_match: f64,
    pub non_greedy_breakdown: Vec<(String, usize, f64)>, // (category, count, pct)
}

impl ArchetypeProfile {
    /// WallBuilder (GB) profile from archetype-gb.md (2 games, 120 turns)
    pub fn wall_builder() -> Self {
        Self {
            archetype: Archetype::WallBuilder,
            bot_name: "Great Barrier".to_string(),
            games_analyzed: 2,
            enemy_turns: 120,
            overall_match: 0.375,
            opening_match: 0.0,
            mid_match: 0.208,
            late_match: 0.583,
            non_greedy_breakdown: vec![
                ("Other (corridor infra)".to_string(), 52, 0.433),
                ("DelayedClose".to_string(), 8, 0.067),
                ("PopBreak".to_string(), 6, 0.050),
                ("FarExpand".to_string(), 5, 0.042),
                ("WallCompletion".to_string(), 2, 0.017),
                ("ThicketBuild".to_string(), 2, 0.017),
            ],
        }
    }

    /// PopTimer (VladNet) profile from rival-vladnet.md + M7 audit (2 games, 120 turns)
    pub fn pop_timer() -> Self {
        Self {
            archetype: Archetype::PopTimer,
            bot_name: "VladNet".to_string(),
            games_analyzed: 2,
            enemy_turns: 120,
            overall_match: 0.075,
            opening_match: 0.0,
            mid_match: 0.146,
            late_match: 0.033,
            non_greedy_breakdown: vec![
                ("MemorizedOpener".to_string(), 16, 0.133), // 8 moves × 2 colors
                ("SurgicalCut (Lag 1-2)".to_string(), 45, 0.375),
                ("AnchorTrussExtension".to_string(), 35, 0.292),
                ("FarExpand".to_string(), 12, 0.100),
                ("Other".to_string(), 12, 0.100),
            ],
        }
    }

    /// SpaceGrabber (AngelWASM) profile from rival-analysis.md + M7 audit (2 games, 120 turns)
    pub fn space_grabber() -> Self {
        Self {
            archetype: Archetype::SpaceGrabber,
            bot_name: "AngelBot WASM".to_string(),
            games_analyzed: 2,
            enemy_turns: 120,
            overall_match: 0.158,
            opening_match: 0.083,
            mid_match: 0.125,
            late_match: 0.200,
            non_greedy_breakdown: vec![
                ("OpeningScript".to_string(), 10, 0.083),
                ("BiggerLoopClose".to_string(), 35, 0.292),
                ("RemoteExpand".to_string(), 25, 0.208),
                ("SearchMove".to_string(), 30, 0.250), // middleweight search moves
                ("Other".to_string(), 20, 0.167),
            ],
        }
    }

    /// Mirror (v6/v7) profile from M7 audit (10 games, 600 turns)
    pub fn mirror() -> Self {
        Self {
            archetype: Archetype::Mirror,
            bot_name: "Mirror (v6/v7)".to_string(),
            games_analyzed: 10,
            enemy_turns: 600,
            overall_match: 0.552,
            opening_match: 0.0,
            mid_match: 0.608,
            late_match: 0.561,
            non_greedy_breakdown: vec![
                ("v6-enemy (greedy)".to_string(), 300, 0.500), // 77.7% match
                ("v7-enemy (timed)".to_string(), 300, 0.500),  // 32.7% match
                ("MeshPrefix".to_string(), 40, 0.067),
                ("BlueOpener".to_string(), 20, 0.033),
                ("Other".to_string(), 40, 0.067),
            ],
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::wall_builder(),
            Self::pop_timer(),
            Self::space_grabber(),
            Self::mirror(),
        ]
    }
}

/// Predict a reply using archetype-specific logic
pub fn predict_archetype_reply(
    archetype: Archetype,
    position_after_our: &Position,
    our_move_was_close: bool,
    our_last_move_target: Option<Point>,
    action_count: usize,
) -> Option<Move> {
    match archetype {
        Archetype::WallBuilder => predict_wall_builder_reply(position_after_our, our_move_was_close, action_count),
        Archetype::PopTimer => predict_pop_timer_reply(position_after_our, our_move_was_close, our_last_move_target, action_count),
        Archetype::SpaceGrabber => predict_space_grabber_reply(position_after_our, our_move_was_close, action_count),
        Archetype::Mirror => predict_mirror_reply(position_after_our, our_move_was_close, action_count),
        Archetype::Unknown => None,
    }
}

/// WallBuilder (GB) reply prediction — corridor infrastructure + late close/break/expand
fn predict_wall_builder_reply(pos: &Position, we_closed: bool, actions: usize) -> Option<Move> {
    let gb_color = pos.to_move();
    let legal: Vec<Move> = pos.legal_moves().iter().collect();

    // RULE 1: Fixed Opening Script (actions <= 12)
    if actions <= 12 {
        let script = match gb_color {
            Player::Blue => blue_gb_opener(),
            Player::Red => red_gb_opener(),
        };
        if let Some(mv) = script.get(actions).copied().flatten() {
            if pos.check_move(mv).is_ok() {
                return Some(mv);
            }
        }
    }

    // RULE 2: Corridor Root Contest on close turns
    if we_closed {
        if let Some(cut) = find_corridor_cut(pos, gb_color) {
            return Some(cut);
        }
    }

    // RULE 3: Mid-Game Corridor Infrastructure (actions 13-59)
    if actions >= 13 && actions < 60 {
        let mut best_infra: Option<(Move, f64)> = None;
        for mv in &legal {
            let score = corridor_infrastructure_score(pos, gb_color, *mv);
            if best_infra.map_or(true, |(_, s)| score > s) {
                best_infra = Some((*mv, score));
            }
        }
        if let Some((mv, _)) = best_infra {
            return Some(mv);
        }
    }

    // RULE 4: Late-Game Close/Break/Expand (actions >= 60)
    if actions >= 60 {
        return predict_late_wall_builder(pos, gb_color, we_closed, legal);
    }

    legal.first().copied()
}

/// Late-game WallBuilder prediction
fn predict_late_wall_builder(pos: &Position, color: Player, we_closed: bool, legal: Vec<Move>) -> Option<Move> {
    // Priority: Break opponent corridor > Close own corridor > Far expand
    if we_closed {
        // Break opponent's responding corridor
        if let Some(break_mv) = find_opponent_corridor_break(pos, color, legal.iter().copied()) {
            return Some(break_mv);
        }
    }
    // Close own ready corridor
    if let Some(close_mv) = find_own_corridor_close(pos, color, legal.iter().copied()) {
        return Some(close_mv);
    }
    // Far expand (safe, >5 from enemy)
    legal.iter()
        .filter(|mv| is_far_expand(pos, color, **mv))
        .max_by(|a, b| area_gain(pos, color, **a).total_cmp(&area_gain(pos, color, **b)))
        .copied()
}

/// PopTimer (VladNet) reply prediction — memorized opener → surgical cuts → anchor truss
fn predict_pop_timer_reply(pos: &Position, we_closed: bool, our_last_target: Option<Point>, actions: usize) -> Option<Move> {
    let vlad_color = pos.to_move();
    let legal: Vec<Move> = pos.legal_moves().iter().collect();
    let our_area_after = pos.area(vlad_color.opponent()).to_f64();

    // RULE 1: Memorized Opener Script (actions <= 15)
    if actions <= 15 {
        let script = match vlad_color {
            Player::Red => red_vlad_opener(),
            Player::Blue => blue_vlad_opener(),
        };
        for (act, mv_opt) in script {
            if act == actions + 1 {
                if let Some(m) = mv_opt {
                    if pos.check_move(m).is_ok() {
                        return Some(m);
                    }
                }
            }
        }
    }

    // RULE 2: Immediate Surgical Cut (94% Lag 1-2 counter-punch)
    if we_closed {
        let mut best_cut: Option<(Move, f64)> = None;
        for reply in &legal {
            let mut probe = pos.clone();
            let outcome = probe.apply_unchecked(*reply);
            if outcome.broken.is_some() {
                let area_destroyed = our_area_after - probe.area(vlad_color.opponent()).to_f64();
                if area_destroyed > 0.5 {
                    let tgt = reply.target().unwrap();
                    let neighbor_count = probe.nodes(vlad_color).iter()
                        .filter(|n| (n.x() - tgt.x()).abs() <= 1 && (n.y() - tgt.y()).abs() <= 1)
                        .count() as f64;
                    let degree = probe.edges(vlad_color).iter()
                        .filter(|e| e.has_endpoint(tgt))
                        .count();
                    let penalty = if degree <= 1 { -5.0 } else { 0.0 };
                    let struct_score = neighbor_count * 2.0 + penalty;
                    
                    let proximity_bonus = if let Some(our_tgt) = our_last_target {
                        let dx = (tgt.x() - our_tgt.x()).abs() as f64;
                        let dy = (tgt.y() - our_tgt.y()).abs() as f64;
                        let dist = dx.max(dy);
                        if dist <= 3.0 { 10.0 - dist * 2.0 } else { 0.0 }
                    } else { 0.0 };
                    
                    let score = area_destroyed * 2.0 + struct_score + proximity_bonus;
                    if best_cut.map_or(true, |(_, s)| score > s) {
                        best_cut = Some((*reply, score));
                    }
                }
            }
        }
        if let Some((mv, _)) = best_cut {
            return Some(mv);
        }
    }

    // RULE 3: Anchor Thicket / Multi-Node Truss Extension
    let mut best_truss: Option<(Move, f64)> = None;
    for reply in &legal {
        let mut probe = pos.clone();
        let _outcome = probe.apply_unchecked(*reply);
        let tgt = reply.target().unwrap();
        
        let neighbor_count = probe.nodes(vlad_color).iter()
            .filter(|n| (n.x() - tgt.x()).abs() <= 1 && (n.y() - tgt.y()).abs() <= 1)
            .count() as f64;
        let degree = probe.edges(vlad_color).iter()
            .filter(|e| e.has_endpoint(tgt))
            .count();
        let penalty = if degree <= 1 { -5.0 } else { 0.0 };
        let score = neighbor_count * 2.0 + penalty;
        
        if best_truss.map_or(true, |(_, s)| score > s) {
            best_truss = Some((*reply, score));
        }
    }
    if let Some((mv, _)) = best_truss {
        return Some(mv);
    }

    // RULE 4: Fallback to greedy (first legal move)
    legal.first().copied()
}

/// SpaceGrabber (AngelWASM) reply prediction — opening script → bigger loops → remote expand
fn predict_space_grabber_reply(pos: &Position, we_closed: bool, actions: usize) -> Option<Move> {
    let angel_color = pos.to_move();
    let legal: Vec<Move> = pos.legal_moves().iter().collect();

    // RULE 1: Opening Script (actions <= 12)
    if actions <= 12 {
        let script = match angel_color {
            Player::Blue => blue_angel_opener(),
            Player::Red => red_angel_opener(),
        };
        if let Some(mv) = script.get(actions).copied().flatten() {
            if pos.check_move(mv).is_ok() {
                return Some(mv);
            }
        }
    }

    // RULE 2: Bigger Loop Close (mid-game)
    if actions >= 13 && actions < 60 {
        // Look for closes that gain >= 3 area (Angel's loops avg ~5.8)
        let mut best_close: Option<(Move, f64)> = None;
        for mv in &legal {
            let mut probe = pos.clone();
            let outcome = probe.apply_unchecked(*mv);
            if outcome.kind == MoveKind::Connect {
                let gain = probe.area(angel_color).to_f64() - pos.area(angel_color).to_f64();
                if gain >= 3.0 {
                    if best_close.map_or(true, |(_, g)| gain > g) {
                        best_close = Some((*mv, gain));
                    }
                }
            }
        }
        if let Some((mv, _)) = best_close {
            return Some(mv);
        }
    }

    // RULE 3: Remote Expand (safe, >5 from enemy)
    let mut best_expand: Option<(Move, f64)> = None;
    for mv in &legal {
        if is_far_expand(pos, angel_color, *mv) {
            let gain = pos.area(angel_color).to_f64();
            let mut probe = pos.clone();
            probe.apply_unchecked(*mv);
            let new_gain = probe.area(angel_color).to_f64() - gain;
            if best_expand.map_or(true, |(_, g)| new_gain > g) {
                best_expand = Some((*mv, new_gain));
            }
        }
    }
    if let Some((mv, _)) = best_expand {
        return Some(mv);
    }

    // RULE 4: Fallback to search-like move (longest edge)
    legal.iter()
        .max_by_key(|mv| mv.direction.dx().abs().max(mv.direction.dy().abs()))
        .copied()
}

/// Mirror (v6/v7) reply prediction — greedy ranked().first() with mesh prefix
fn predict_mirror_reply(pos: &Position, _we_closed: bool, actions: usize) -> Option<Move> {
    let legal: Vec<Move> = pos.legal_moves().iter().collect();
    
    // Check mesh prefix (same as our mesh_prefix)
    if actions < 16 {
        // Simplified: just return greedy first move for now
        // Real mirror would have mesh prefix logic
    }
    
    // Greedy: longest edge first (simplified ranked)
    legal.iter()
        .max_by_key(|mv| mv.direction.dx().abs().max(mv.direction.dy().abs()))
        .copied()
}

/// Weighted ensemble prediction: combines archetype predictions with confidence weights
pub fn predict_weighted_reply(
    archetype_weights: &[(Archetype, f64)], // (archetype, weight) - weights sum to 1.0
    position_after_our: &Position,
    our_move_was_close: bool,
    our_last_move_target: Option<Point>,
    action_count: usize,
) -> Option<Move> {
    use std::collections::HashMap;
    
    let mut vote_counts: HashMap<Move, f64> = HashMap::new();
    
    for (arch, weight) in archetype_weights {
        if let Some(pred) = predict_archetype_reply(*arch, position_after_our, our_move_was_close, our_last_move_target, action_count) {
            *vote_counts.entry(pred).or_insert(0.0) += weight;
        }
    }
    
    // Return highest-weighted prediction
    vote_counts.into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(mv, _)| mv)
}

// ===== Helper functions =====

fn blue_gb_opener() -> Vec<Option<Move>> {
    vec![
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, 2).unwrap()), // D10-E12
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-3, 3).unwrap()), // D10-F13
        Move::between(Point::new(-4, 2).unwrap(), Point::new(-2, 6).unwrap()), // E12-G15
        Move::between(Point::new(-3, 3).unwrap(), Point::new(-1, 7).unwrap()), // F13-H16
        Move::between(Point::new(-2, 6).unwrap(), Point::new(0, 9).unwrap()),  // G15-I18
        Move::between(Point::new(-1, 7).unwrap(), Point::new(0, 9).unwrap()),  // H16-J19 (H16=(-1,7), J19=(0,9))
        Move::between(Point::new(0, 9).unwrap(), Point::new(0, 9).unwrap()),   // I18-J19 (same point - skip)
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, -3).unwrap()), // D10-F7
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, -1).unwrap()), // D10-E8
        Move::between(Point::new(-4, -1).unwrap(), Point::new(-1, -4).unwrap()), // E8-G5
        Move::between(Point::new(-4, -3).unwrap(), Point::new(-2, -5).unwrap()), // F7-H4
        Move::between(Point::new(-1, -4).unwrap(), Point::new(0, -7).unwrap()),  // G5-I2
        None,
    ]
}

fn red_gb_opener() -> Vec<Option<Move>> {
    vec![
        Move::between(Point::new(6, 0).unwrap(), Point::new(4, 2).unwrap()),   // P10-O12
        Move::between(Point::new(6, 0).unwrap(), Point::new(3, 3).unwrap()),   // P10-N13 (N=12, x=12-9=3)
        Move::between(Point::new(4, 2).unwrap(), Point::new(2, 6).unwrap()),   // O12-M15
        Move::between(Point::new(3, 3).unwrap(), Point::new(1, 7).unwrap()),   // N13-L16
        Move::between(Point::new(2, 6).unwrap(), Point::new(-1, 9).unwrap()),  // M15-K18
        Move::between(Point::new(1, 7).unwrap(), Point::new(0, 9).unwrap()),   // L16-J19
        Move::between(Point::new(0, 9).unwrap(), Point::new(0, 9).unwrap()),   // K18-J19 (same)
        Move::between(Point::new(6, 0).unwrap(), Point::new(4, -3).unwrap()),  // P10-N7
        Move::between(Point::new(6, 0).unwrap(), Point::new(4, -1).unwrap()),  // P10-O8
        Move::between(Point::new(4, -1).unwrap(), Point::new(1, -4).unwrap()), // O8-M5
        Move::between(Point::new(4, -3).unwrap(), Point::new(2, -5).unwrap()), // N7-L4
        Move::between(Point::new(1, -4).unwrap(), Point::new(-1, -7).unwrap()), // M5-K2
        Move::between(Point::new(-1, -7).unwrap(), Point::new(-1, -9).unwrap()), // L4-J1
    ]
}

fn red_vlad_opener() -> Vec<(usize, Option<Move>)> {
    vec![
        (2, Move::between(Point::new(6, 0).unwrap(), Point::new(9, 3).unwrap())),   // P10-S13
        (3, Move::between(Point::new(9, 3).unwrap(), Point::new(9, 0).unwrap())),   // S13-S10
        (6, Move::between(Point::new(9, 3).unwrap(), Point::new(6, 1).unwrap())),   // S13-P11
        (7, Move::between(Point::new(6, 0).unwrap(), Point::new(9, -3).unwrap())),  // P10-S7
        (10, Move::between(Point::new(9, -3).unwrap(), Point::new(6, -1).unwrap())), // S7-P9
        (11, Move::between(Point::new(6, 0).unwrap(), Point::new(3, 3).unwrap())),  // P10-M13
        (14, Move::between(Point::new(3, 3).unwrap(), Point::new(6, 1).unwrap())),  // M13-P11
        (15, Move::between(Point::new(9, 0).unwrap(), Point::new(9, -3).unwrap())), // S10-S7
    ]
}

fn blue_vlad_opener() -> Vec<(usize, Option<Move>)> {
    vec![
        (1, Move::between(Point::new(-6, 0).unwrap(), Point::new(-9, 3).unwrap())),  // D10-A13
        (4, Move::between(Point::new(-9, 3).unwrap(), Point::new(-9, 0).unwrap())),  // A13-A10
        (5, Move::between(Point::new(-9, 3).unwrap(), Point::new(-6, 1).unwrap())),  // A13-D11
        (8, Move::between(Point::new(-6, 0).unwrap(), Point::new(-9, -3).unwrap())), // D10-A7
        (9, Move::between(Point::new(-9, -3).unwrap(), Point::new(-6, -1).unwrap())), // A7-D9
        (12, Move::between(Point::new(-6, 0).unwrap(), Point::new(-3, 3).unwrap())), // D10-J13
        (13, Move::between(Point::new(-3, 3).unwrap(), Point::new(-6, 1).unwrap())), // J13-D11
        (16, Move::between(Point::new(-9, 0).unwrap(), Point::new(-9, -3).unwrap())), // A10-A7
    ]
}

fn blue_angel_opener() -> Vec<Option<Move>> {
    vec![
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, 1).unwrap()),  // D10-C11
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, 3).unwrap()),  // D10-B13
        Move::between(Point::new(-4, 3).unwrap(), Point::new(-1, 7).unwrap()),  // B13-D16
        Move::between(Point::new(-6, 0).unwrap(), Point::new(-3, 4).unwrap()),  // D10-G13
        Move::between(Point::new(-3, 4).unwrap(), Point::new(-2, 1).unwrap()),  // G13-E10
        Move::between(Point::new(-3, 4).unwrap(), Point::new(-2, 7).unwrap()),  // G13-E16
        None, None, None, None, None, None, None,
    ]
}

fn red_angel_opener() -> Vec<Option<Move>> {
    vec![
        Move::between(Point::new(6, 0).unwrap(), Point::new(3, 3).unwrap()),   // S10-S13
        Move::between(Point::new(3, 3).unwrap(), Point::new(0, 7).unwrap()),   // S13-P16
        Move::between(Point::new(6, 0).unwrap(), Point::new(3, 3).unwrap()),   // P10-M13
        Move::between(Point::new(3, 3).unwrap(), Point::new(0, 7).unwrap()),   // M13-P16
        None, None, None, None, None, None, None, None, None, None,
    ]
}

fn corridor_infrastructure_score(pos: &Position, color: Player, mv: Move) -> f64 {
    let mut probe = pos.clone();
    probe.apply_unchecked(mv);
    let tgt = mv.target().unwrap();
    
    // Shared-node density (GB's signature)
    let shared_nodes = probe.edges(color).iter()
        .filter(|e| e.has_endpoint(tgt))
        .flat_map(|e| [e.origin(), e.far()])
        .filter(|n| probe.edges(color).iter().filter(|e2| e2.has_endpoint(*n)).count() >= 2)
        .count() as f64;
    
    // Root proximity (corridor roots: P11/J10 for Blue, D10/G10 for Red)
    let roots = match color {
        Player::Blue => [Point::new(-3, 1).unwrap(), Point::new(0, 4).unwrap()], // P11, J10
        Player::Red => [Point::new(3, 1).unwrap(), Point::new(0, -4).unwrap()],  // D10, G10 mirrored
    };
    let root_proximity = roots.iter()
        .map(|r| (r.x() - tgt.x()).abs().max((r.y() - tgt.y()).abs()) as f64)
        .fold(f64::INFINITY, f64::min);
    let root_bonus = if root_proximity <= 3.0 { 5.0 - root_proximity } else { 0.0 };
    
    shared_nodes * 3.0 + root_bonus
}

fn find_corridor_cut(pos: &Position, color: Player) -> Option<Move> {
    let opp = color.opponent();
    let our_area = pos.area(opp).to_f64();
    let legal: Vec<Move> = pos.legal_moves().iter().collect();
    
    let mut best: Option<(Move, f64)> = None;
    for mv in legal {
        let mut probe = pos.clone();
        let outcome = probe.apply_unchecked(mv);
        if outcome.broken.is_some() {
            let destroyed = our_area - probe.area(opp).to_f64();
            if destroyed > 0.5 {
                // Prefer cuts near corridor roots
                let tgt = mv.target().unwrap();
                let roots = match color {
                    Player::Blue => [Point::new(-3, 1).unwrap(), Point::new(0, 4).unwrap()],
                    Player::Red => [Point::new(3, 1).unwrap(), Point::new(0, -4).unwrap()],
                };
                let root_dist = roots.iter()
                    .map(|r| (r.x() - tgt.x()).abs().max((r.y() - tgt.y()).abs()) as f64)
                    .fold(f64::INFINITY, f64::min);
                let score = destroyed * 2.0 + if root_dist <= 3.0 { 5.0 - root_dist } else { 0.0 };
                if best.map_or(true, |(_, s)| score > s) {
                    best = Some((mv, score));
                }
            }
        }
    }
    best.map(|(mv, _)| mv)
}

fn find_opponent_corridor_break(pos: &Position, color: Player, legal: impl Iterator<Item = Move>) -> Option<Move> {
    let opp = color.opponent();
    let our_area = pos.area(opp).to_f64();
    
    let mut best: Option<(Move, f64)> = None;
    for mv in legal {
        let mut probe = pos.clone();
        let outcome = probe.apply_unchecked(mv);
        if outcome.broken.is_some() {
            let destroyed = our_area - probe.area(opp).to_f64();
            if destroyed > 0.5 {
                if best.map_or(true, |(_, d)| destroyed > d) {
                    best = Some((mv, destroyed));
                }
            }
        }
    }
    best.map(|(mv, _)| mv)
}

fn find_own_corridor_close(pos: &Position, color: Player, legal: impl Iterator<Item = Move>) -> Option<Move> {
    let mut best: Option<(Move, f64)> = None;
    for mv in legal {
        let mut probe = pos.clone();
        let outcome = probe.apply_unchecked(mv);
        if outcome.kind == MoveKind::Connect {
            let gain = probe.area(color).to_f64() - pos.area(color).to_f64();
            if gain >= 5.0 { // GB closes big loops
                if best.map_or(true, |(_, g)| gain > g) {
                    best = Some((mv, gain));
                }
            }
        }
    }
    best.map(|(mv, _)| mv)
}

fn is_far_expand(pos: &Position, color: Player, mv: Move) -> bool {
    let tgt = mv.target().unwrap();
    let opp = color.opponent();
    // >5 Chebyshev from any enemy node
    pos.nodes(opp).iter().all(|n| (n.x() - tgt.x()).abs() > 5 || (n.y() - tgt.y()).abs() > 5)
}

fn area_gain(pos: &Position, color: Player, mv: Move) -> f64 {
    let mut probe = pos.clone();
    probe.apply_unchecked(mv);
    probe.area(color).to_f64() - pos.area(color).to_f64()
}