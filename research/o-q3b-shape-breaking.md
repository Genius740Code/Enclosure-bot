# Lane O — Q3b Shape-Breaking Counters vs COLLAPSER

**Baseline (skip=0, ret=Blue, 8 solos): 1-7, avg margin -11.1%**

The COLLAPSER (scoutbase, floor-0 rush-closer) signature:
- Closes at own actions 2-3, many small loops compounding
- 30+ pop at ~t=60 (game action ~30) followed by steady erosion
- 16-30 closes at ~0.7-2.9 area/close, cuts 28-41x/game

## Three Counters Tested

### Counter 1: Early Contest of Corridor Root
**Concept**: Bonus for cutting enemy corridors (open edge paths) at their root.
**Implementation**: `corridor_cut_bonus` tracing enemy edge paths from cut endpoints, bonus = corridor_len × events_left × 5.0.
**Result**: 0-8, **-22.7%** (worse).
**Why it failed**: The bonus incentivized cutting ANY enemy corridor, including short/irrelevant ones. Our bot wasted tempo cutting corridors the collapser wasn't actually going to close, while the collapser kept banking. The search cannot predict which corridor will become the 30+ pop.

### Counter 2: Pre-Build Deny Second Wall
**Concept**: Increase REMOTE_BONUS (2.5→1.0) and lower FIGHT_HEAT threshold (5→10) to establish remote/thicket structure earlier, denying opponent expansion corridors.
**Implementation**: Modified constants in search.rs.
**Result**: 2-6, **-8.9%** (slight improvement on avg margin, but still losing).
**Why it partially worked**: Game 1 (+18.6% vs +3.9%) and Game 6 (+16.5% vs -8.2%) flipped to wins. But Games 3,4,5,7 got worse. The remote bias helps when the collapser expands toward our remote structure, but hurts when the collapser rushes the center.

### Counter 3: Price-Denial Bank Race
**Concept**: Quadratic bonus for first-action closes in early window (actions < 20): `own_gain² × events_left × bonus`.
**Implementation**: `EARLY_BANK_BONUS = 3.0` (also tested 50.0).
**Result**: No change (1-7, -11.1%) even at 50x weight.
**Why it failed**: The search's move ordering (longest edges first) and candidate selection (top 8 by priority) don't surface early closes. Short edges (length 1-2) that close small loops are at the bottom of the 2048-move first_actions list. Even with massive priority bonuses, they don't crack the top 8 searched. The 2-ply search fundamentally doesn't "see" the value of early small closes because the static eval caps area value at HORIZON=12 events.

## Root Cause Analysis

The collapser exploits a **horizon mismatch**:
- Collapser: banks area at t=2-3, compounds for 60+ events (real game length)
- Our search: values area at min(events_left, 12) — early banks look identical to late banks
- Horizon_extension adds beyond-horizon value symmetrically for BOTH players, but the collapser's early banks are ALREADY SCORED by t=60 while our eval treats them as future potential

The PATIENCE_PENALTY (sub-2.0 closes before action 12) was meant to prevent tiny closes, but V5-1 confirmed it never fires. The real issue is that the search **doesn't generate early close candidates** in its top 8.

## Conclusion for Q3b

| Counter | Result | Viable? |
|---------|--------|---------|
| Corridor contest | -22.7% | ❌ Worse |
| Pre-build deny | -8.9% | ⚠️ Marginal, inconsistent |
| Price-denial (early bank bonus) | -11.1% | ❌ No effect |

**None of the three shape-breaking counters work at 2-ply depth**. The search cannot:
1. Predict which enemy corridor becomes the 30+ pop (Counter 1)
2. Consistently deny expansion without hurting center control (Counter 2)
3. Generate early close candidates to price (Counter 3)

## Path Forward: Q11 PREVENT-THE-WALL

The only viable approach is **Q11: closure-progress trigger**. Instead of trying to out-bank or out-cut the collapser, detect when the enemy's live-area growth / frontier-pair count signals a big close is 2-4 turns out, then STOP banking small stuff and contest the corridor root.

This requires a **phase detection** term in the eval (Lane B), not a search modification (Lane A). The trigger:
- Enemy live area growing rapidly (area/close > 2.0 for 3+ consecutive closes)
- Enemy frontier-pair count (closeable node pairs) exceeding threshold
- When triggered: suppress REMOTE_BONUS, increase CORRIDOR_CUT_BONUS, force local contest