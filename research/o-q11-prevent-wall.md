# Lane O — Q11 PREVENT-THE-WALL: Closure-Progress Trigger

**Baseline (skip=0, ret=Blue, 8 solos): 1-7, avg margin -11.1%**

## Concept

When enemy's live-area growth rate + frontier-pair count signals a big close is 2-4 turns out:
- STOP banking small stuff
- CONTEST the corridor root (enemy's expansion path)

## Fast Heuristic Tested

**Trigger**: `enemy_area > 15.0` AND `enemy_frontier_nodes (degree < 3) >= 6`

**Action**: Boost DOOM_W by `1.0 + threat × 2.0` in both static eval and selection.

## Result

| Metric | Baseline | Q11 Trigger | Delta |
|--------|----------|-------------|-------|
| W-L | 1-7 | 2-6 | +1 win |
| Avg Margin | -11.1% | **-22.3%** | **-11.2%** |
| Wins flipped | Games 1, 3 | — | +2 wins |
| Losses worsened | Games 2,4,5,6,7,8 | — | -6 games |

Games 1 (+10.8%) and 3 (+11.9%) flipped to wins, but Games 4,5,6,7 got **much worse** (-29.7%, -48.4%, -49.7%, -38.6%).

## Why It Failed

1. **Trigger fires too broadly**: The collapser has many small loops → many degree-2 nodes (frontier) → trigger activates early and often, not just before the 30+ pop.

2. **Wrong response**: Boosting DOOM_W makes us **avoid building loops** (fear of being popped). But the collapser's threat is that THEY pop a big loop, not that our loops are vulnerable. We should CONTEST their corridor, not retreat.

3. **Search can't act on signal**: Even with boosted threat, the 2-ply search doesn't generate "contest corridor root" moves. The top 8 candidates are still long edges and small closes.

4. **Signal timing mismatch**: The collapser's 30+ pop at t=60 corresponds to enemy_area ~30+ at game action ~30. But our trigger fires at enemy_area > 15 (much earlier), causing premature defensive play.

## What Would Be Needed

A working closure-progress trigger needs:
- **Better signal**: Track enemy's *area per close* over recent turns (requires game history, not in static eval)
- **Targeted response**: Specific "contest this corridor" move generation, not just eval penalty
- **Search integration**: Lane A (search_deep) to see the 30+ pop coming, or Lane B phase system with dedicated contest moves

## Conclusion for Q11

**The fast heuristic closure-progress trigger fails at 2-ply**. The signal is noisy, the response is wrong (retreat vs contest), and the search can't generate corridor-contest moves.

**Path forward**: This requires either:
1. Lane A: 3-ply search that sees the 30+ pop in the tree
2. Lane B: Phase system with game-history tracking (enemy area/close rate over last N turns)
3. Dedicated "corridor contest" move generator in search (not just eval penalty)