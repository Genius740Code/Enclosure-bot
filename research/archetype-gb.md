# Archetype Profile: Great Barrier (GB)

**Date**: 2026-09-29  
**Target Rival**: Great Barrier (Wall-builder / Corridor banker)  
**Primary Source Games**: `30c7653b` (GB Blue, win), `ad65f054` (GB Red, win)  
**Baseline Greedy Match Rate**: **37.5%** overall (Opening 0%, Mid 20.8%, Late 58.3% from `m7-reply-audit.md`)

---

## 1. Strategic Identity

GB is a **methodical wall-builder** that constructs shared-node 2-wall corridors, banks area behind them, and delays closes until corridors are unbreakable. It plays the "long game" — early moves invest in corridor roots, mid-game extends walls, late-game closes and defends.

- **Wall-Race Specialist**: Builds corridors with 2+ shared nodes (near-unbreakable, any cut touches 2+ walls)
- **Banking Over Snatching**: Delays first close until ~action 13+ (vs our patience window of 12), banking ~10+ area per loop
- **Corridor Root Control**: Secures corridor roots early; contesting roots is the only way to prevent the bank
- **Late-Game Pop Defense**: Once banks are established, switches to breaking opponent corridors

---

## 2. Reply Distribution (120 enemy turns across 2 games)

| Category | Count | % | Phase Concentration |
|----------|-------|-----|---------------------|
| **GreedyMatch** | 45 | 37.5% | Increases late (58.3%) |
| **Other** | 52 | 43.3% | Opening 100%, Mid 82%, Late 28% |
| **DelayedClose** | 8 | 6.7% | Mid 3, Late 5 |
| **PopBreak** | 6 | 5.0% | Late only |
| **FarExpand** | 5 | 4.2% | Late only |
| **WallCompletion** | 2 | 1.7% | Mid only |
| **ThicketBuild** | 2 | 1.7% | Late only |

### Phase Breakdown

**Opening (actions 0-11, 12 turns)**: 100% Other — GB executes a fixed opening script not captured by greedy model.

**Mid (actions 12-59, 38 turns)**: 82% Other, 8% DelayedClose, 5% WallCompletion — GB extends corridors, places corridor roots, builds shared-node walls.

**Late (actions 60+, 25 turns)**: 28% Other, 24% PopBreak, 20% DelayedClose, 20% FarExpand, 8% ThicketBuild — GB closes corridors, breaks opponent banks, expands when safe.

---

## 3. Opening Script (Inferred)

GB has a memorized opening sequence (both colors). From the two games:

**As Blue (`30c7653b`)**:
- Action 0: Extend from D10 toward center
- Action 3: Extend corridor root
- Action 4: Extend shared-node wall
- Action 7: Extend corridor
- Action 8: Connect to form first shared node
- Actions 11, 12: Continue corridor extension

**As Red (`ad65f054`)**:
- Mirror of Blue on right flank

The opening builds a **corridor root at P11/J10** (Blue) or **D10/G10** (Red) with shared-node density from the start.

---

## 4. Mid-Game: Corridor Extension & Root Control

GB's mid-game moves fall into "Other" because they don't fit greedy categories:
- **Corridor Root Placement**: Moves at corridor roots (P11, J10, D10, G10) that don't immediately gain area but secure future banks
- **Shared-Node Wall Extension**: Extends walls that share nodes with existing walls (2+ shared nodes = near-unbreakable)
- **DelayedClose**: Could close a small loop but waits — the Connect gains <2 area while corridor extends

**Key Insight**: GB's mid-game "Other" moves are **corridor infrastructure** — they pay off in late-game banks. Greedy model sees 0-2 area and rejects; GB sees corridor progress and invests.

---

## 5. Late-Game: Close, Break, Expand

| Behavior | Trigger | Frequency |
|----------|---------|-----------|
| **PopBreak** | Opponent corridor near completion | 6/25 late turns |
| **DelayedClose** | Own corridor ready, safe to close | 5/25 late turns |
| **FarExpand** | Safe remote expansion, no local fight | 5/25 late turns |
| **ThicketBuild** | Reinforce existing dense cluster | 2/25 late turns |

**PopBreak Timing**: GB breaks opponent corridors at Lag 1-2 after opponent's close (similar to VladNet but corridor-focused vs loop-focused).

---

## 6. Pseudo-Spec: GB Reply Predictor

```rust
/// Predicts GB's reply. GB = methodical wall-builder with fixed opener, corridor infrastructure mid-game, late-game close/break/expand.
pub fn predict_gb_reply(position_after_our: &Position, our_move_was_close: bool) -> Option<Move> {
    let gb_color = position_after_our.to_move();
    let actions_played = position_after_our.actions_played() as usize;
    let legal = position_after_our.legal_moves();

    // RULE 1: Fixed Opening Script (actions <= 12)
    if actions_played <= 12 {
        let script = match gb_color {
            Player::Blue => blue_gb_opener(),
            Player::Red => red_gb_opener(),
        };
        if let Some(mv) = script.get(actions_played).copied().flatten() {
            if position_after_our.check_move(mv).is_ok() {
                return Some(mv);
            }
        }
    }

    // RULE 2: Corridor Root Contest (if opponent threatens corridor root)
    // Detect if our last move advanced a corridor toward GB's root
    if our_move_was_close {
        // Prefer cuts on corridors we're building
        if let Some(cut) = find_corridor_cut(position_after_our, gb_color) {
            return Some(cut);
        }
    }

    // RULE 3: Mid-Game Corridor Infrastructure
    // Score moves by: corridor progress (shared nodes >= 2) + root proximity
    let mut best_infra: Option<(Move, f64)> = None;
    for mv in legal.iter() {
        let score = corridor_infrastructure_score(position_after_our, gb_color, mv);
        if best_infra.map_or(true, |(_, s)| score > s) {
            best_infra = Some((mv, score));
        }
    }
    if let Some((mv, _)) = best_infra {
        return Some(mv);
    }

    // RULE 4: Late-Game Close/Break/Expand
    if actions_played >= 60 {
        return predict_late_gb(position_after_our, gb_color, our_move_was_close, legal);
    }

    legal.first().copied()
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

fn predict_late_gb(pos: &Position, color: Player, we_closed: bool, legal: Vec<Move>) -> Option<Move> {
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
        .filter(|mv| is_far_expand(pos, color, *mv))
        .max_by_key(|mv| area_gain(pos, color, *mv))
        .copied()
}
```

---

## 7. Predictor Target Bar

- **Greedy baseline to beat**: **37.5%** overall (M7 audit)
- **Opening Script Target**: **≥ 80%** match on actions 0-11
- **Mid-Game Infrastructure Target**: **≥ 40%** match on actions 12-59 (corridor moves)
- **Late-Game Target**: **≥ 50%** match on actions 60+ (close/break/expand)
- **Overall Held-out Target**: **≥ 45%** (vs 37.5% greedy)

---

## 8. Key Modeling Gaps (vs VladNet)

| Aspect | VladNet | GB |
|--------|---------|-----|
| **Opening** | Fixed 8-move script (both colors) | Fixed script, corridor-root focused |
| **Mid-Game** | Surgical cuts (94% Lag 1-2) | Corridor infrastructure (shared nodes, root proximity) |
| **Late-Game** | Pure counter-punch sniper | Close own corridors, break opponent corridors, far expand |
| **Greedy Match** | 7.5% (catastrophic failure) | 37.5% (moderate — late game becomes predictable) |
| **Predictor Difficulty** | High (specific cut selection) | Medium (corridor progress is measurable) |

---

## 9. Next Steps

1. **Extract exact opening scripts** from the two games (action-by-action)
2. **Implement corridor detection** (shared-node census, root identification)
3. **Validate on held-out GB games** (queued evals: `b1e7d2d1`, `f33e29fc`)
4. **Build predictor** and measure hit rate vs 37.5% baseline