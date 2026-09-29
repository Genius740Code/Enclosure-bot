# Archetype Profile: VladNet

**Date**: 2026-09-29  
**Target Rival**: VladNet (Neural network agent)  
**Primary Source Games**: `3a414aad` (VladNet Red, win 2377.6 vs 1370.7), `1c69056c` (VladNet Blue, win 2254.8 vs 1530.3)  
**Baseline Greedy Match Rate**: **7.5%** overall (Opening 0.0%, Mid 14.6%, Late 3.3% from `research/m7-reply-audit.md`)

---

## 1. Break Policy

VladNet operates as a pure counter-punch sniper with surgical reactivity and zero proactive over-extension.

- **Lag 1–2 Concentration**: **94.1%** of all breaks dealt (32 out of 34 breaks in `3a414aad`).
- **Exact Split**: **16 breaks at Lag 1** (47.1%, immediate reply action) and **16 breaks at Lag 2** (47.1%, second action of reply turn). Only 2 breaks occurred at lag > 2 (Actions 114 and 115).
- **Zero Proactive Breaks**: VladNet never wastes moves hunting speculative cuts into unclosed lines or dead space. It initiates breaks exclusively in response to opponent loop completions (`Connect` or area-gaining `Extend`).
- **Total Damage**: Inflicted **-314.0 area loss** across 34 breaks in `3a414aad`.
- **Destruction Rate on Closes**: Across the 7 audited re-close nodes (Actions 56, 57, 64, 80, 84, 88, 89), Blue gained +105.9 gross area; VladNet broke 7/7 on the immediate next turn, wiping out **-104.4 area (98.6%)** within ≤2 actions.

---

## 2. Reply Habits: Cuts > Growth

Our baseline search model assumes opponents play greedy area-maximization (`ranked().first()`). Against VladNet, this assumption completely collapses:

- **Baseline Greedy Match**: Only **7.5%** across 120 turns in `m7-reply-audit.md` (and 5.0% on 20 sampled midgame positions).
- **Priority Hierarchy**: **Cuts > Structural Anchors > Area Growth**.
  1. If a cut is available on an opponent corridor, VladNet selects the cut with ~94% probability.
  2. If no cut is available, VladNet rejects local 1-ply / 2-ply area-snatching moves (e.g., small 1.2–3.0 area triangles).
  3. Instead, it plays long-range structural anchors (`P16-M13`, `A13-D16`, `G7-I10`, `K13-N14`) that build multi-node interior trusses and establish redundant roots.

---

## 3. Opening Book: Memorized Red Line

As Red (`3a414aad`), VladNet executes a fixed, memorized sequence through Action 15 (8 Red moves):

| Action | Color | Move Index | Move Notation | Move Kind | Area Gain | Resulting Structure |
|---|---|---|---|---|---|---|
| **2** | Red | 17153 | `(6,0)-(9,3)` / `P10-S13` | Extend | +0.0 | Upper boundary reach |
| **3** | Red | 17150 | `(9,3)-(9,0)` / `S13-S10` | Connect | **+4.5** | Immediate turn-1 loop close |
| **6** | Red | 17148 | `(9,3)-(6,1)` / `S13-P11` | Extend | +0.0 | Downward diagonal rib |
| **7** | Red | 17147 | `(6,0)-(9,-3)` / `P10-S7` | Extend | +0.0 | Lower boundary reach |
| **10** | Red | 17149 | `(9,-3)-(6,-1)` / `S7-P9` | Extend | +0.0 | Lower inner rib |
| **11** | Red | 17153 | `(6,0)-(3,3)` / `P10-M13` | Extend | +0.0 | Diagonal spearhead into center |
| **14** | Red | 17148 | `(3,3)-(6,1)` / `M13-P11` | Connect | +0.0 | Closes center-support triangle |
| **15** | Red | 17147 | `(9,0)-(9,-3)` / `S10-S7` | Connect | +4.5 | Closes edge corridor `S13-S10-S7-P10` |

**Architectural Signature**: This 8-move red sequence builds an impenetrable, 6-node home base (`P10, S10, S13, P11, S7, P9`) with an anchored outpost at `M13`. It locks in 9.0 area of permanent, un-cuttable base territory before contesting the center. (Blue in `1c69056c` executes the exact mirror sequence on the left flank).

---

## 4. Bank Policy

- **Inflexible Retention**: Once VladNet establishes a bank, it holds it flat. In `3a414aad`, VladNet reached **49.2 area at move 40** and concluded the game at **49.2 area at move 120**.
- **Depth Over Surface Area**: Banks are built 3+ king steps deep from the edge with internal triangulated trusses. Breaking an outer perimeter segment does not collapse the inner bank (in `3a414aad`, Blue attempted 29 breaks; only 1 achieved ≥10 area loss).
- **Anti-Overextension**: VladNet does not push past the board midline until its rear bases are double-anchored. When ahead, it switches into pure maintenance mode: hold bank, cut opponent closes, refuse speculative commitments.

---

## 5. Pseudo-Spec: Pure Functional Reply Predictor

A zero-training, deterministic decision-list predictor modeling VladNet's reply distribution:

```rust
/// Pure function predicting VladNet's reply to our candidate move.
/// Signature: predict_vladnet_reply(position_before, our_candidate_move) -> Option<Move>
pub fn predict_vladnet_reply(position: &Position, our_mv: Move) -> Option<Move> {
    let mut after_our = position.clone();
    let oc = after_our.apply_unchecked(our_mv);
    let vlad_color = after_our.to_move(); // Enemy to move
    let our_color = vlad_color.opponent();
    let actions_played = after_our.actions_played() as usize;

    // RULE 1: Memorized Red Opener Script (Actions <= 15)
    if vlad_color == Player::Red && actions_played <= 15 {
        let red_script = [
            (2, Move::between(Point::new(6, 0), Point::new(9, 3))),   // P10-S13
            (3, Move::between(Point::new(9, 3), Point::new(9, 0))),   // S13-S10
            (6, Move::between(Point::new(9, 3), Point::new(6, 1))),   // S13-P11
            (7, Move::between(Point::new(6, 0), Point::new(9, -3))),  // P10-S7
            (10, Move::between(Point::new(9, -3), Point::new(6, -1))),// S7-P9
            (11, Move::between(Point::new(6, 0), Point::new(3, 3))),  // P10-M13
            (14, Move::between(Point::new(3, 3), Point::new(6, 1))),  // M13-P11
            (15, Move::between(Point::new(9, 0), Point::new(9, -3))), // S10-S7
        ];
        for (act, mv) in red_script {
            if act == actions_played + 1 {
                if let Some(m) = mv {
                    if after_our.check_move(m).is_ok() {
                        return Some(m);
                    }
                }
            }
        }
    }

    // RULE 2: Immediate Surgical Cut (94% Counter-Punch Policy)
    // If our move closed or expanded a loop, find legal enemy cuts that destroy our area.
    let our_area_before = position.area(our_color).to_f64();
    let our_area_after = after_our.area(our_color).to_f64();
    let legal_replies = after_our.legal_moves();

    let mut best_cut: Option<(Move, f64)> = None;
    for &reply in legal_replies.iter() {
        let mut probe = after_our.clone();
        let outcome = probe.apply_unchecked(reply);
        if outcome.broken.is_some() {
            let area_destroyed = our_area_after - probe.area(our_color).to_f64();
            if area_destroyed > 0.5 {
                // Rank cuts by total area destroyed
                if best_cut.as_ref().map_or(true, |(_, max_d)| area_destroyed > *max_d) {
                    best_cut = Some((reply, area_destroyed));
                }
            }
        }
    }
    if let Some((cut_mv, _)) = best_cut {
        return Some(cut_mv);
    }

    // RULE 3: Anchor Thicket / Multi-Node Truss Extension
    // In quiet positions, VladNet selects moves with maximum node density (shared nodes >= 2)
    // and distance <= 3 from existing home clusters, avoiding exposed single-edge leaves.
    let mut best_truss: Option<(Move, f64)> = None;
    for &reply in legal_replies.iter() {
        let mut probe = after_our.clone();
        let outcome = probe.apply_unchecked(reply);
        let tgt = reply.target().unwrap();
        
        // Count friendly neighbors within distance 1
        let neighbor_count = probe.nodes(vlad_color).iter()
            .filter(|n| (n.x() - tgt.x()).abs() <= 1 && (n.y() - tgt.y()).abs() <= 1)
            .count() as f64;
        
        // Avoid single-edge exposed leaves
        let degree = probe.edges(vlad_color).iter()
            .filter(|e| e.has_endpoint(tgt))
            .count();
        let penalty = if degree <= 1 { -5.0 } else { 0.0 };

        let score = neighbor_count * 2.0 + penalty;
        if best_truss.as_ref().map_or(true, |(_, max_s)| score > *max_s) {
            best_truss = Some((reply, score));
        }
    }
    if let Some((truss_mv, _)) = best_truss {
        return Some(truss_mv);
    }

    // RULE 4: Fallback to Top-Ranked Move
    legal_replies.first().copied()
}
```

---

## 6. Predictor Target Bar

- **Greedy baseline to beat**: **7.5%** reply match (`m7-reply-audit.md`).
- **Tactical Cut Sub-distribution Target**: **≥ 60.0%** match on turns where an opponent loop was closed.
- **Overall Held-out Accuracy Bar**: **≥ 40.0%** match across all VladNet moves on held-out test games (e.g. `9f06cd59` and future evals).
