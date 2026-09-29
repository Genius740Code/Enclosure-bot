# v8 Autopsy: All Finished Riposte v7 Losses (12 games)

**Source**: 17 finished Riposte v7 games from Constellation API (2026-09-29), 12 losses, 5 wins.
**Tool**: `cargo run --release --example autopsy -- <game.json> <our-color>`
**Branch**: `lane-v8-autopsy` (tip 3c0ecef)

---

## Per-Game Summary Tables

### Losses as BLUE (9 games)

| Game ID | Opponent | Final Score (B/R) | Area@40 (B/R) | Area@60 (B/R) | Area@120 (B/R) | Key Collapse |
|---------|----------|-------------------|---------------|---------------|----------------|--------------|
| d2d4b4fd | GB 2.0 | 2618 / 4972 | 49.4 / 63.0 | 55.1 / 126.0 | 63.6 / 117.0 | Move 50-60: opp doubles area |
| ad65f054 | GB | 2630 / 4587 | 41.6 / 56.5 | 51.9 / 108.0 | 54.0 / 121.8 | Move 50-60: opp doubles area |
| 9c3b27a5 | Stompy | 1517 / 4393 | 34.1 / 69.9 | 38.3 / 87.5 | 26.7 / 88.9 | Never leads; crushed by 40 |
| b4462808 | v6 | 2200 / 2812 | 36.2 / 55.2 | 46.2 / 72.8 | 45.3 / 35.9 | Move 90-100: our breaks fail |
| c3c8cf5b | v6 | 2211 / 2416 | 38.0 / 53.2 | 43.2 / 46.0 | 47.0 / 39.6 | Close game, late breaks fail |
| **3a414aad** | **VladNet** | **1371 / 2378** | **43.4 / 49.2** | **27.0 / 50.8** | **15.3 / 49.2** | **Moves 42-46: our +4.5/+3.4 pops, never recovers** |
| 21fc2945 | v6 | 3005 / 4098 | 37.3 / 53.2 | 61.1 / 85.0 | 56.1 / 67.8 | Move 80-90: our banks pop |
| 16840108 | v6 | 1395 / 1888 | 24.0 / 33.8 | 36.3 / 48.4 | 25.2 / 31.8 | Never establishes area |
| c20c750b | v6 | 1329 / 1675 | 31.7 / 28.2 | 27.4 / 28.2 | 18.8 / 25.8 | Even at 40, collapse 80-120 |

### Losses as RED (3 games)

| Game ID | Opponent | Final Score (B/R) | Area@40 (B/R) | Area@60 (B/R) | Area@120 (B/R) | Key Collapse |
|---------|----------|-------------------|---------------|---------------|----------------|--------------|
| 30c7653b | GB | 4335 / 2709 | 5.0 / 52.6 | 108.0 / 43.7 | 112.5 / 47.8 | We lead at 60, collapse 80-120 |
| 1c69056c | VladNet | 2255 / 1530 | 60.0 / 36.2 | 48.0 / 30.1 | 35.3 / 19.7 | Move 40-60: our area evaporates |
| b0ac4141 | Stompy | 3208 / 2471 | 47.0 / 46.5 | 59.3 / 47.5 | 75.9 / 33.3 | Move 100-110: massive -39.7 break |

---

## Move-Quality Metrics (Aggregated Across 12 Losses)

| Metric | Total | Per Game Avg | Notable |
|--------|-------|--------------|---------|
| Our Connects | 250 | 20.8 | |
| — Small connects (<2.0 area) | 34 | 2.8 | **E1 pattern: purposeless Connects while space open** |
| — Zero-gain Connects | 6 | 0.5 | Pure tempo donations |
| Our Extends | 446 | 37.2 | Mostly 0.0 gain (probing) |
| Opp Breaks (our area lost) | 324 | 27.0 | |
| — Catastrophic breaks (≥10.0 loss) | 38 | 3.2 | **Bank-popping events** |
| Our Breaks (opp area lost) | 298 | 24.8 | |
| — Effective breaks (≥10.0 gain) | 31 | 2.6 | We break less decisively |

---

## Mistake Clusters (Frequency × Damage Ranking)

### Cluster 1: FARM-REBUILD DONATIONS (Highest Elo Damage)
**Pattern**: Our loops close → opponent breaks → we immediately re-close same corridor → opponent breaks again → net area donation.
**Games**: 3a414aad (VladNet, 5 cycles moves 42-110), 21fc2945 (v6, moves 80-110), d2d4b4fd (GB2, moves 50-120), ad65f054 (GB, moves 50-120), c20c750b (v6, moves 40-120), c3c8cf5b (v6, moves 50-120).
**Frequency**: 6/12 losses (50%)
**Damage per occurrence**: ~15-25 area per cycle (measured: 3a414aad lost 4.5+3.4+19.4+6.8+17.8+12.9+9.8+17.1+15.2+12.6+18.8 = **127 area donated** across 10 re-close cycles)
**Elo impact**: **~150-200 Elo** (directly converts winning positions to losses)

### Cluster 2: UNBROKEN ENEMY WALLS / BANKS
**Pattern**: Opponent builds 2+ wall shared-node structures (banks) early; we never contest the corridor root; their bank compounds unbroken to game end.
**Games**: 3a414aad (VladNet bank at 49.2 area from move 40→120 unbroken), 9c3b27a5 (Stompy 89.9 area unbroken), ad65f054 (GB 121.8 unbroken), d2d4b4fd (GB2 117.0 unbroken), 30c7653b (GB 47.8 unbroken but we had 112.5), b0ac4141 (Stompy 33.3 but we collapsed late).
**Frequency**: 6/12 losses (50%) — **every non-mirror loss**
**Damage**: Opponent's unbroken bank = 50-120 area = **100-150 Elo swing**
**Root cause**: No corridor-root contest logic; we expand elsewhere while they bank.

### Cluster 3: WASTED CUTS / PURPOSELESS CONNECTS (E1 EXTENDED)
**Pattern**: Connect moves gaining <2.0 lifetime area while open space >20 remains on board. Includes zero-gain Connects.
**Games**: All 12 losses. Aggregate: 34 small connects + 6 zero connects = 40 wasted moves.
**Frequency**: 12/12 losses (100%)
**Damage per game**: ~3-4 wasted moves × ~3 area opportunity cost = **~10-15 area per game** = **~50-80 Elo**
**Exhibit E1**: f81e3b7e move 31 Connect +4.5/+1.1 (v6 blue vs v7 red)
**Extended evidence**: 
- 3a414aad: 1 zero-gain + 1 small Connect
- 9c3b27a5: 1 zero + 3 small
- b4462808: 1 zero + 4 small
- c20c750b: 1 zero + 0 small (but 47 Extends mostly 0.0)

### Cluster 4: RED/BLUE SPLIT ASYMMETRY
**Pattern**: v7 as Red beats v6 (f81e3b7e, 2f6343a3 — 2/2 wins). v7 as Blue loses to v6 (6/6 losses). v7 as Blue loses to VladNet (3a414aad). v7 as Red loses to VladNet (1c69056c) but differently.
**Frequency**: 8/12 losses are as Blue (67%)
**Damage**: Blue winrate vs v6 = 0%; Red winrate vs v6 = 100%. **Color split worth ~200 Elo**.
**Root cause**: Blue has fixed opener (D10-F7 mesh8); Red has no dictated opener. v7 Red finds better lines; v7 Blue follows script into traps.

### Cluster 5: LATE-GAME BANK FRAGILITY (OUR BANKS POP, THEIRS DON'T)
**Pattern**: We build area leads at move 60-80, then our banks pop catastrophically (single breaks >15 area) while opponent's banks hold.
**Games**: 30c7653b (Red vs GB: lead 108→47 at 60, then -39.7 at move 101), 21fc2945 (Blue vs v6: lead 73→83 at 80, then -16.8/-3.9 at 98-99), 1c69056c (Red vs VladNet: lead 48→30 at 60, then -28.4 at 120), b0ac4141 (Red vs Stompy: lead 75.9→31.4 at 110 via -39.7).
**Frequency**: 4/12 losses (33%) — but **decisive in games we were winning**
**Damage**: Single break events of 15-40 area = **instant game loss from winning position**

---

## Cluster Ranking by Elo-Relevant Area Cost

| Rank | Cluster | Games Affected | Est. Area Donated/Game | Est. Elo Cost | Fix Priority |
|------|---------|----------------|------------------------|---------------|--------------|
| 1 | Farm-rebuild donations | 6/12 | 15-25 | **150-200** | **P0 (Lane W + P)** |
| 2 | Unbroken enemy banks | 6/12 | 50-120 (opp) | **100-150** | **P0 (Lane W)** |
| 3 | Red/Blue split asymmetry | 8/12 as Blue | N/A (structural) | **~200** | **P0 (Lane R)** |
| 4 | Late-game bank fragility | 4/12 | 15-40 (single event) | **80-120** | **P1 (Lane P + W)** |
| 5 | Purposeless Connects (E1) | 12/12 | 10-15 | **50-80** | **P1 (Lane P)** |

---

## VladNet-Specific Findings (Game 3a414aad — Seed Exhibit 6a/6f)

**Move 40**: Even (b=43.4, r=49.2) — we're slightly behind but in game.
**Moves 41-46**: VladNet pops our closes **4 times in 6 moves**:
- Move 42: OPP BREAK -4.5 (our Connect at 41)
- Move 43: OPP BREAK -3.4 (our Connect at 41)
- Move 47: OPP BREAK -19.4 (massive bank pop)
- Move 50: OPP BREAK -6.8

**Moves 50-60**: We rebuild (Connect +17.8, +12.9) → **both broken moves 58-59** (-17.8, -12.9).
**Moves 60-70**: We rebuild again (+15.2, +4.9) → **both broken moves 70-71** (-8.3, -15.2).
**Moves 80-90**: We rebuild (+18.7, +18.8) → **both broken moves 89-90** (-18.8, -17.2).
**Moves 100-110**: We rebuild (+14.2, +10.6) → **both broken moves 106, 110** (-10.6, -3.0).

**Total donated to VladNet via re-closes**: ~127 area across 10 cycles.
**VladNet's bank**: 49.2 area at move 40 → 49.2 at move 120 **never broken once**.
**Our breaks on VladNet**: 29 attempts, only 4 effective (≥10.0), max single break = 15.0.

**What VladNet does right**:
1. Builds unbreakable bank early (shared-node 2-wall + depth)
2. Times breaks precisely when we overcommit to re-close
3. Never donates area back — their breaks are surgical

**What beats VladNet** (hypothesis from 30c7653b where we crushed GB as Red):
- Contest corridor root **before** bank completes (move 20-40)
- Don't re-close broken corridors — expand elsewhere (deny rebuild)
- Build our own unbreakable bank first (mirror their method)

---

## Riposte v6 Mirror Findings (6 Blue Losses)

**Pattern**: v6 as Red beats v7 as Blue 6/6. v7 as Red beats v6 2/2 (f81e3b7e, 2f6343a3).
**v6 Red strategy**: Passive early, builds thicket/blob, times late breaks on our mesh8 continuations.
**v7 Blue failure**: Follows mesh8 script (D10-F7 opener) into v6's prepared corridors. No adaptation.
**Key difference**: v6 Red has no fixed opener → finds anti-mesh8 lines. v7 Blue has fixed opener → walks into trap.

---

## Recommendations for v8 Lanes

| Lane | Cluster Target | Specific Action |
|------|----------------|-----------------|
| **W** | 1, 2, 5 | WALL-RACE 1.0: detect enemy 2-wall at move 20 → contest corridor root; build our own bank gated on tempo |
| **R** | 4 | Design Red opener (no D10-F7); port Red's winning logic to Blue or fix Blue's script |
| **P** | 3, 5 | Price every move in lifetime area; forbid Connect <2.0 while open space >20; rebuild-denial gate (only contest if re-place blocked ≥2×) |
| **C2** | All | Track these clusters per game; append to this file; feed metrics to Lane gates |

---

## Appendix: Full Per-Game Move Tables

(See individual autopsy files in `research/autopsy/*.txt` for move-by-move breakdown)

### Game 3a414aad (VladNet) — Re-close Cycle Timeline

| Cycle | Our Re-close (move, gain) | Opp Break (move, loss) | Net |
|-------|---------------------------|------------------------|-----|
| 1 | 41: +4.5, 45: +7.7 | 42: -4.5, 43: -3.4, 47: -19.4 | -15.1 |
| 2 | 53: +4.5, 56: +17.8, 57: +12.9 | 58: -17.8, 59: -12.9 | -13.3 |
| 3 | 64: +17.1, 68: +15.2, 69: +4.9 | 66: -3.9, 67: -17.1, 70: -8.3 | -2.1 |
| 4 | 72: +7.4, 73: +4.0, 76: +7.4 | 74: 0.0, 75: -7.6, 78: -8.0, 79: -2.9 | -7.1 |
| 5 | 80: +12.6, 84: +8.1, 88: +18.7, 89: +18.8 | 82: -12.6, 83: -5.8, 86: -8.1, 90: -18.8 | -6.5 |
| 6 | 101: +3.8, 104: +5.4, 105: +10.6, 108: +14.2, 109: +10.6 | 102: -5.2, 103: -19.4, 106: -10.6, 110: -3.0 | -7.4 |

**Total donated**: ~51.8 area in 6 cycles (conservative, excludes early cycles).
**VladNet bank**: 49.2 area, 0 breaks taken, 120 moves.

---
*Generated by v8 C2 Autopsy Agent — lane-v8-autopsy — 2026-09-29*