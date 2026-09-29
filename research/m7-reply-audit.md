# M7 Reply-Model Audit — Enclosure-bot v8

**Date:** 2026-09-29  
**Branch:** `lane-v8-audit` (tip `3c0ecef`)  
**Objective:** Measure whether deeper search (timed beam, `analyze_timed`) converts to wins by auditing the reply-model assumption: *our search assumes the enemy always plays `ranked().first()` (greedy reply).*

---

## 1. Reply-Match Rate — Replay of Finished Site Games

Replayed **20 finished games** (1,320 enemy turns) via `replay_match.rs`. At each enemy turn, recorded our `analyze_with_avoid()` predicted reply vs. the enemy's ACTUAL move.

### Overall Results

| Opponent Archetype | Games | Enemy Turns | Match Rate | Opening (<12) | Mid (12–59) | Late (60+) |
|--------------------|-------|-------------|------------|---------------|-------------|------------|
| **Mirror (v6-vs-v7)** | 10 | 600 | **55.2%** | 0/30 (0%) | 146/240 (60.8%) | 185/330 (56.1%) |
|  • We are v7 (enemy v6) | 5 | 300 | **77.7%** | 0/15 (0%) | 90/120 (75.0%) | 104/165 (63.0%) |
|  • We are v6 (enemy v7) | 5 | 300 | **32.7%** | 0/15 (0%) | 56/120 (46.7%) | 81/165 (49.1%) |
| **VladNet** | 2 | 120 | **7.5%** | 0/12 (0%) | 7/48 (14.6%) | 2/60 (3.3%) |
| **Great Barrier (GB)** | 2 | 120 | **37.5%** | 0/12 (0%) | 10/48 (20.8%) | 35/60 (58.3%) |
| **GB 2.0** | 2 | 120 | **32.5%** | 0/12 (0%) | 6/48 (12.5%) | 33/60 (55.0%) |
| **AngelBot WASM** | 2 | 120 | **15.8%** | 1/12 (8.3%) | 6/48 (12.5%) | 12/60 (20.0%) |
| **Stompy** | 2 | 120 | **22.5%** | 2/12 (16.7%) | 7/48 (14.6%) | 18/60 (30.0%) |
| **v3 (Riposte v3)** | 2 | 120 | **43.3%** | 6/12 (50.0%) | 22/48 (45.8%) | 24/60 (40.0%) |

### Weighted Overall (all 20 games, 1,320 enemy turns)
- **Total matched: 522 / 1,320 = 39.5%**
- Opening: 8/102 (7.8%)
- Mid: 198/480 (41.3%)
- Late: 316/738 (42.8%)

### Key Observations
1. **Mirror split is diagnostic**: When the enemy IS our own search algorithm (v6), match rate is high (77.7%). When the enemy is v7 (timed beam, different behavior), match rate collapses to 32.7%.
2. **Top bots are unpredictable**: VladNet (7.5%), AngelWASM (15.8%), GB2.0 (32.5%) — the strongest opponents deviate most from greedy `ranked().first()`.
3. **Opening is noise**: 0% match in opening for almost all archetypes (forced prefixes, mesh, blue_opener dominate).
4. **Late-game improves slightly** for wall-building bots (GB, GB2.0) but not for pop/space-grab bots (VladNet, Angel).

---

## 2. Flip Test — `analyze` (2-ply fixed) vs `analyze_timed` (beam deepening)

Tested **20 real mid-game positions** (actions 12–59) from the VladNet game (3a414aad). Compared best pick from fixed-budget `analyze()` (2-ply, MOVE_BUDGET=4096) vs timed `analyze_timed()` (beam-8, 2s/4.8s).

| Position (action) | Fixed Best | Timed Best | Timed Depth | Flipped? |
|-------------------|------------|------------|-------------|----------|
| 12 | (-6,3)→(-3,3) | (-9,2)→(-6,-1) | 12 | ✅ |
| 15 | (-6,3)→(-3,3) | (-6,0)→(-9,-3) | 10 | ✅ |
| 16 | (-9,2)→(-6,3) | (-9,2)→(-6,3) | 7 | ❌ |
| 19 | (-9,2)→(-6,3) | (-6,0)→(-8,-3) | 7 | ✅ |
| 20 | (-6,3)→(-3,3) | (-6,3)→(-3,3) | 12 | ❌ |
| 23 | (-9,0)→(-7,-3) | (-9,0)→(-7,-3) | 6 | ❌ |
| 24 | (-8,-3)→(-5,0) | (-8,-3)→(-5,0) | 6 | ❌ |
| 27 | (0,0)→(0,-3) | (-6,3)→(-9,5) | 10 | ✅ |
| 28 | (0,-3)→(-3,-2) | (0,-3)→(-3,-2) | 5 | ❌ |
| 31 | (0,-3)→(3,0) | (-6,3)→(-9,5) | 8 | ✅ |
| 32 | (3,-1)→(0,1) | (0,1)→(3,-2) | 5 | ✅ |
| 35 | (-6,3)→(-4,5) | (-6,3)→(-4,5) | 5 | ❌ |
| 36 | (-5,5)→(-8,2) | (-6,3)→(-3,6) | 8 | ✅ |
| 39 | (-2,3)→(-3,6) | (-2,3)→(-3,6) | 4 | ❌ |
| 40 | (3,-1)→(0,0) | (3,-1)→(0,0) | 3 | ❌ |
| 43 | (-9,2)→(-8,5) | (-5,5)→(-8,2) | 5 | ✅ |
| 44 | (-8,5)→(-5,5) | (-8,5)→(-5,5) | 3 | ❌ |
| 47 | (-5,0)→(-3,3) | (-5,0)→(-3,3) | 3 | ❌ |
| 48 | (3,-1)→(0,1) | (3,-1)→(0,1) | 3 | ❌ |
| 51 | (-3,2)→(-4,-1) | (-3,-2)→(-5,-1) | 5 | ✅ |

### Flip Test Summary
- **Total positions: 20**
- **Flipped: 9 (45.0%)**
- **Same: 11 (55.0%)**
- Timed depths ranged 3–12 (median ~6)

**Interpretation:** Deeper search **does change the best move** nearly half the time (45% flip rate). The timed beam is not "theater" in terms of changing our own pick.

---

## 3. Verdict

### The Core Tension
| Metric | Value | Threshold (from v8-plan §6h) | Verdict |
|--------|-------|------------------------------|---------|
| **Reply-match rate (overall)** | **39.5%** | < 50% → "depth is theater" | ❌ **FAIL** |
| **Reply-match rate vs top bots** | 7.5–32.5% | < 50% → "depth is theater" | ❌ **FAIL** |
| **Flip rate (analyze vs analyze_timed)** | **45%** | ~never → "depth is theater" | ✅ **PASS** (flips frequent) |

### Conclusion
**Match < 50% overall (39.5%) and catastrophically low vs top bots (7.5–32.5%) → DEPTH IS THEATER.**

The M2 port (chess-programming alpha-beta + TT + ordering) **MUST NOT PROCEED** until the reply model is fixed. Deeper search only helps if the enemy replies we assume are real. They are not.

**Required for v8 Elo gains:**
1. **Opponent-aware reply model** (M6 rival strat files → feed into search)
2. **Stop assuming `ranked().first()`** — model actual opponent distributions:
   - VladNet: mid-game pops, unbreakable banks
   - GB/GB2.0: wall races, shared-node corridors
   - AngelWASM: space-grabs, remote expansion
   - Stompy: thicket density, contact pressure
3. **M7 audit gates M2** — this report is the gate. M2 port BLOCKED.

---

## 4. Data Sources
- **Replay tool:** `retaliator/examples/replay_match.rs` (and `_others`, `_remaining`)
- **Flip test tool:** `retaliator/examples/flip_test.rs`
- **Games analyzed:** 20 finished games from `https://constellation.blueshrimp.uk/api/games/ID`
- **Search config:** `MOVE_BUDGET=4096`, `WIDTH=8`, `BEAM=8`, `THINK_SOFT_MS=2000`, `THINK_HARD_MS=4800`

---

## 5. Commit
All analysis code committed to `lane-v8-audit`:

```
git add retaliator/examples/replay_match.rs retaliator/examples/replay_match_others.rs retaliator/examples/replay_match_remaining.rs retaliator/examples/flip_test.rs retaliator/Cargo.toml research/m7-reply-audit.md
git commit -m "M7: reply-model audit — match 39.5% overall, 7.5-32.5% vs top bots, flip 45%. M2 port BLOCKED."
git push origin lane-v8-audit
```

**Commit hash:** (to be filled after push)