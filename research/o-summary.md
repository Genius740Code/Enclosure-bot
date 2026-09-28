# Lane O (v7 opponent-mimic) — Summary

**Branch**: `lane-v7-opp` (pushed to origin)

## COLLAPSER Reproduction (Step 1) ✓

**Probe**: `probe_collapser_solos.rs` / `probe_collapser_detail.rs`
- Baseline: skip=0 ret=Blue, 8 solos → **1-7, avg margin -11.1%**
- 7/8 losses confirms repro gate (≥3/8 required ✓)
- Collapser = scoutbase (floor-0 rush-closer)
- Pattern: closes at own actions 2-3, many small loops compounding, 30+ pop at ~t=60 + erosion

**Defense Ablation Table** (single-term removal vs baseline -11.1%):

| Defense | Term | W-L | Margin | Delta | Verdict |
|---------|------|-----|--------|-------|---------|
| CAPTURE_W | capture exposure | 1-7 | **-31.6%** | **-20.5%** | **CRITICAL** |
| DOOM_W | vulnerability pricing | 2-6 | -14.0% | -2.9% | HELPFUL |
| HORIZON_WEIGHT | horizon extension | 1-7 | -7.4% | **+3.7%** | **HURTS** |
| DENSE_BONUS | thicket density | 2-6 | -4.2% | **+6.9%** | **HURTS** |
| FRESH_PENALTY | fresh wall avoidance | 3-5 | -5.3% | **+5.8%** | **HURTS** |
| REBUILD_PENALTY | anti-rebuild routing | 1-7 | -11.1% | 0.0% | NO EFFECT |
| PATIENCE_PENALTY | patience | 1-7 | -11.1% | 0.0% | NO EFFECT |
| IDLE_PENALTY | do-nothing filter | 1-7 | -11.1% | 0.0% | NO EFFECT |
| DEADWOOD_PENALTY | dead-wood discount | 1-7 | -11.1% | 0.0% | NO EFFECT |
| CONTACT_PENALTY | contact avoidance | 1-7 | -11.3% | -0.2% | NEUTRAL |
| REMOTE_BONUS | two-front play | 2-6 | -11.2% | -0.1% | NEUTRAL |

**Lane B Recommendations** (research/o-collapser-defense.md):
- FRESH_PENALTY: 1.5 → 0.0 (contest fresh walls)
- DENSE_BONUS: 1.0 → 0.0 (wasted vs rush-closer)
- HORIZON_WEIGHT: 0.5 → 0.2-0.3 (reduce future discounting)
- CAPTURE_W: 3.0 → KEEP/INCREASE (critical)
- DOOM_W: 2.5 → 3.0-4.0 (stronger vulnerability pricing)

---

## Q3b Shape-Breaking Counters (Step 2)

**Probe**: `probe_o_shape_break.rs`

| Counter | Concept | Result | Viable? |
|---------|---------|--------|---------|
| 1. Corridor contest | Bonus for cutting enemy corridors | 0-8, -22.7% | ❌ Worse |
| 2. Pre-build deny | Increase REMOTE_BONUS, lower FIGHT_HEAT | 2-6, -8.9% | ⚠️ Marginal |
| 3. Price-denial | Quadratic early-bank bonus (gain²×events) | 1-7, -11.1% | ❌ No effect |

**Root Cause**: 2-ply horizon (12 events) vs collapser's real compounding (60+ events). Search doesn't generate early close candidates; eval caps area value at HORIZON.

**Details**: research/o-q3b-shape-breaking.md

---

## Q11 PREVENT-THE-WALL (Step 3)

**Fast Heuristic**: `enemy_area > 15 AND enemy_frontier_nodes >= 6` → boost DOOM_W

**Result**: 2-6, **-22.3%** (worse than baseline -11.1%)

**Why Failed**:
- Trigger fires too broadly (collapser has many small loops = many frontier nodes)
- Wrong response: boosted DOOM_W = retreat (avoid building loops), should be CONTEST
- Search can't generate corridor-contest moves from eval signal alone

**Details**: research/o-q11-prevent-wall.md

---

## Conclusion

The COLLAPSER (floor-0 rush-closer) exploits a **fundamental horizon mismatch** at 2-ply depth:
- Collapser banks at t=2-3 for 60+ events
- Our search values area at min(events_left, 12)
- No eval term or search modification at 2-ply can bridge this gap

**Only viable paths**:
1. **Lane A**: 3-ply search (search_deep) that sees the 30+ pop in the tree
2. **Lane B**: Phase system with game-history tracking (enemy area/close rate over last N turns) + dedicated contest move generation

The defense ablation gives clear Lane B priorities for the existing search. The shape-breaking counters and closure-progress trigger confirm that 2-ply cannot stop the collapser's compounding tempo advantage.