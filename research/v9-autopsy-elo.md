# v8 Autopsy: Elo Losses + Rival Fingerprints

**Source:** 12 v8 validation games (GB 0-4, VladNet 0-4, AngelWASM 0-4), game JSONs from `lane-v8-autopsy` branch, rival-analysis.md, league-scoreboard.md.

---

## Per-rival Kill Mechanism

### vs Great Barrier (GB)
**Core problem:** GB deploys a fixed 26-move opening book with mirrored diagonal walls (NE and SE). GB's first close occurs at action 13–15, then closes steadily ~12 actions apart (`[15,28,40,52,…]`), achieving ~10 area per loop and 108–121 total area. Riposte's in-place rebuilt loops (~0.25 area) are farmed — GB continuously cuts the same ground. GB holds ~10 large loops; Riposte holds 30+ tiny loops where each cut costs little. The fixed opening means GB's structure is deterministic and knowable, but Riposte cannot survive the area gap.

**Kill-0 mechanism:** *Anti-rebuild routing* (rival-analysis.md §7 #1). After one of our loops is cut, forbid/penalize re-closing within Chebyshev distance 3 of the cut point for N turns; force the 2-ply to extend walls elsewhere instead. This directly fixes the 41-cuts-farmed failure and the 0.25-area-per-close stat. Cheap: a move filter + eval penalty.

### vs VladNet
**Core problem:** VladNet memorizes the first 13 actions (identical across games as Red). VladNet closes its first loop at action 2 and keeps popping — 30–32 closes as Red, 31 as Blue, with only 44–55 total area. Many tiny banked loops. Against Riposte, VladNet actively demolishes a rebuilder: in one game it recorded 41 cuts (vs 22–26 background rate). When Riposte rebuilds in place, VladNet routes to the same ground and keeps cutting. The memorized opening + high cut rate = Riposte's rebuilds are punished immediately.

**Kill-0 mechanism:** *Avoid-rebuild + route*. After our loop is cut, do NOT re-close in the same bounding box. Instead, extend to unscouted ground the cutter hasn't visited. This defeats VladNet's cut-volume strategy, which depends on re-cutting the same rebuilt ground. Metric: reduce VladNet cut count from ~41 toward 22–26 baseline.

### vs AngelBot WASM
**Core problem:** AngelWASM builds bigger loops than the poppers. As Blue: first close at action 12, 15 closes → 87.7 area (~5.8/loop). As Red: first close at action 6, 14 closes → 36.3 area. GB doubles AngelWASM's area both colors. Riposte's 0.25-area-per-close loops are no match for bigger opponent loops. The smaller the loop we build, the more we lose to opponent area conversion.

**Kill-0 mechanism:** *Big-loop bias + delayed first close*. Delay first close until after action 10 (vs current action 4). Build walls first, prioritize length-3 diagonal extensions toward the far corner/edges (inspired by GB's ~12-action walling before first close at 13–15). Add a small penalty for closing before action 10 while own convex-hull room is growing fast — this stops the action-4 tiny-triangle habit shared with Stompy. Metric: increase our avg area-per-close from 0.25 toward ≥2.0.

---

## Per-Chair Table (v8 Validation 0-12)

| Rival | Chair | Games | Wins | Loss | Avg Margin (ret) |
|-------|-------|-------|------|------|-----------------|
| GB    | Blue  | 4     | 0    | 4    | −2x margins (both chairs) |
| GB    | Red   | 4     | 0    | 4    | −2x margins (both chairs) |
| VladNet| Blue  | 4     | 0    | 4    | 1180/2446 etc. (both chairs) |
| VladNet| Red   | 4     | 0    | 4    | 1893/2631 etc. (both chairs) |
| AngelWASM| Blue | 4   | 0    | 4    | 479/1809 worst as Blue |
| AngelWASM| Red  | 4     | 0    | 4    | 2169/3317 etc. |

**Summary:** v8 loses 0-12 across all three rivals and both chairs. No chair combination achieves a win.

---

## 3+ Ranked Dosable Ideas EACH with Concrete Kill-0 Spec

### Idea 1: Anti-rebuild Routing (Kill-0 vs GB)
- **Metric:** GB average area per game reduced from ~75 → <20; Riposte win % vs GB increased from 0% → >25%
- **Bar:** In 10 consecutive h2h games vs GB, Riposte wins ≥3 (any color), and GB's final area <20 in ≥8/10 games
- **n:** 20 games (10 as Blue, 10 as Red) vs GB baseline
- **Harness:** Run `cargo riposte h2h --rival GB --color both --games 20`; capture final areas and win/loss. Compare against v8 baseline.

### Idea 2: Delayed First Close + Big-Loop Bias (Kill-0 vs AngelWASM)
- **Metric:** Our avg area-per-close increased from 0.25 → ≥2.0; AngelWASM win % vs Riposte reduced from 100% → <50%
- **Bar:** In 10 games vs AngelWASM, Riposte wins ≥5, and our avg area-per-close ≥2.0
- **n:** 15 games (7 as Blue, 8 as Red) vs AngelWASM
- **Harness:** Run `cargo riposte h2h --rival AngelWASM --color both --games 15` with modified search parameters (first-close penalty ≥action 10, loop_bonus triangle planning). Capture area-per-close stats.

### Idea 3: Avoid-Rebuild + Route (Kill-0 vs VladNet)
- **Metric:** VladNet cut count reduced from ~41 → ≤26 per game; Riposte win % vs VladNet increased from 0% → >25%
- **Bar:** In 10 consecutive h2h games vs VladNet, Riposte wins ≥3, and VladNet's cut count ≤26 in ≥8/10 games
- **n:** 20 games (10 as Blue, 10 as Red) vs VladNet baseline
- **Harness:** Run `cargo riposte h2h --rival VladNet --color both --games 20`; count OPP BREAK events and cut counts per game. Compare against v8 baseline of ~41 cuts.

### Idea 4 (bonus): Fixed Opening Book Counter (vs GB)
- **Metric:** GB's opening book effectiveness reduced; Riposte survives opening with >0 area after move 26
- **Bar:** In 10 games vs GB, Riposte's area after move 26 > 10 (vs ~0-5 in v8)
- **n:** 15 games vs GB
- **Harness:** Track area at move 26 (after GB's opening book finishes). Riposte must not auto-lose the opening phase.

---

## Execution Plan

1. **Idea 1 (Anti-rebuild routing):** Implement as a move filter + eval penalty in `retaliator/src/search.rs` (no change to game logic, only evaluation). Harness: run 20-game h2h vs GB, measure area and win%.
2. **Idea 2 (Delayed first close):** Modify search to add penalty for closing before action 10 while convex-hull room is growing. Harness: run 15-game h2h vs AngelWASM, measure area-per-close.
3. **Idea 3 (Avoid-rebuild route):** Add logic to avoid re-closing within Chebyshev distance 3 of last cut point. Harness: run 20-game h2h vs VladNet, measure cut counts.

Commit: append-only to `research/v9-autopsy-elo.md`. Push to `lane-v9-autopsy`. No M files modified; `git status` shows only the new research md.