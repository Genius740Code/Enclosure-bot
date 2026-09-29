# Rival Playbooks: VladNet & Great Barrier (GB / GB 2.0)

**Date**: 2026-09-29  
**Source**: Mined site games vs Riposte v7:
- **VladNet**: `3a414aad` (VladNet Red, win 2377.6 vs 1370.7), `1c69056c` (VladNet Blue, win 2254.8 vs 1530.3)
- **Great Barrier (GB)**: `30c7653b` (GB Blue, win 4334.9 vs 2708.9), `ad65f054` (GB Red, win 4586.9 vs 2629.7)
- **Great Barrier 2.0 (GB2)**: `d2d4b4fd` (GB2 Red, win 4972.2 vs 2618.1)

---

## 1. Rival Playbook: VladNet

VladNet is a neural-net agent characterized by early thicket expansion, dense multi-node interior walls, and highly reactive, surgical break punishment.

### 1.1 Opener (First 6 Moves)

#### Game 3a414aad (VladNet as Red)
- **First 6 game actions**:
  1. Action 1 [Blue / Us]: `(-9,0)-(-9,2)` / `A10-A12` (Extend, +0.0)
  2. Action 2 [Red / VladNet]: `(6,0)-(9,3)` / `P10-S13` (Extend, +0.0)
  3. Action 3 [Red / VladNet]: `(9,3)-(9,0)` / `S13-S10` (Connect, +4.5 area, Red total = 4.5)
  4. Action 4 [Blue / Us]: `(-6,0)-(-6,3)` / `D10-D13` (Extend, +0.0)
  5. Action 5 [Blue / Us]: `(-6,0)-(-3,-2)` / `D10-G8` (Extend, +0.0)
  6. Action 6 [Red / VladNet]: `(9,3)-(6,1)` / `S13-P11` (Extend, +0.0)
- **VladNet's first 6 individual moves (Actions 2, 3, 6, 7, 10, 11)**:
  - Action 2: `(6,0)-(9,3)` / `P10-S13` (Extend, +0.0)
  - Action 3: `(9,3)-(9,0)` / `S13-S10` (Connect, +4.5, total 4.5) — *immediate turn-1 loop close*
  - Action 6: `(9,3)-(6,1)` / `S13-P11` (Extend, +0.0)
  - Action 7: `(6,0)-(9,-3)` / `P10-S7` (Extend, +0.0)
  - Action 10: `(9,-3)-(6,-1)` / `S7-P9` (Extend, +0.0)
  - Action 11: `(6,0)-(3,3)` / `P10-M13` (Extend, +0.0)

#### Game 1c69056c (VladNet as Blue)
- **First 6 game actions**:
  1. Action 1 [Blue / VladNet]: `(-9,0)-(-9,2)` / `A10-A12` (Extend, +0.0)
  2. Action 2 [Red / Us]: `(6,0)-(3,-2)` / `P10-M8` (Extend, +0.0)
  3. Action 3 [Red / Us]: `(3,-2)-(0,0)` / `M8-J10` (Extend, +0.0)
  4. Action 4 [Blue / VladNet]: `(-6,0)-(-9,3)` / `D10-A13` (Extend, +0.0)
  5. Action 5 [Blue / VladNet]: `(-9,3)-(-9,2)` / `A13-A12` (Connect, +4.5 area, Blue total = 4.5)
  6. Action 6 [Red / Us]: `(0,0)-(3,3)` / `J10-M13` (Extend, +0.0)
- **VladNet's first 6 individual moves (Actions 1, 4, 5, 8, 9, 12)**:
  - Action 1: `(-9,0)-(-9,2)` / `A10-A12` (Extend, +0.0)
  - Action 4: `(-6,0)-(-9,3)` / `D10-A13` (Extend, +0.0)
  - Action 5: `(-9,3)-(-9,2)` / `A13-A12` (Connect, +4.5, total 4.5) — *immediate turn-2 loop close*
  - Action 8: `(-9,3)-(-6,1)` / `A13-D11` (Extend, +0.0)
  - Action 9: `(-6,0)-(-9,-3)` / `D10-A7` (Extend, +0.0)
  - Action 12: `(-6,0)-(-3,3)` / `D10-G13` (Extend, +0.0)

**Key Opener Signature**: VladNet ALWAYS closes a tight 4.5 area triangle on its very first turn/pair-turn (Action 3 as Red, Action 5 as Blue) anchored against its home edge, followed by radial thicket tentacles into the board.

### 1.2 First-Wall Move + Size

- **Game 3a414aad**:
  - First wall close: Action 3, Move `(9,3)-(9,0)` / `S13-S10`, gain **+4.5 area** (total 4.5).
  - First major bank (gain ≥ 10.0): Action 27, Move `(9,-3)-(6,-6)` / `S7-P4`, gain **+15.0 area** (total 31.8).
- **Game 1c69056c**:
  - First wall close: Action 5, Move `(-9,3)-(-9,2)` / `A13-A12`, gain **+4.5 area** (total 4.5).
  - First major bank (gain ≥ 10.0): Action 29, Move `(-6,6)-(-3,3)` / `D16-G13`, gain **+15.0 area** (total 33.0).

Both games show first major bank completion around **Action 27–29** for exactly **+15.0 area**, taking total area to **31.8–33.0**.

### 1.3 Break Timing vs Us

VladNet operates as a pure counter-puncher: it rarely initiates breaks into dead space; instead, it waits for us to play a Connect move, then punishes within 1–2 actions.

- **Game 3a414aad (34 breaks dealt, -314.0 total area loss to us, 12 catastrophic breaks ≥10.0)**:
  - Us closes Action 41 (+4.5) → VladNet breaks Action 42 (-4.5, lag 1) and Action 43 (-3.4, lag 2).
  - Us closes Action 45 (+7.7) → VladNet breaks Action 47 (-19.4, lag 2) and Action 50 (-6.8, lag 5).
  - Us closes Action 53 (+4.5) → VladNet breaks Action 54 (-1.2, lag 1) and Action 55 (-7.9, lag 2).
  - Us closes Action 57 (+12.9) → VladNet breaks Action 58 (-17.8, lag 1) and Action 59 (-12.9, lag 2).
  - Us closes Action 64 (+17.1) → VladNet breaks Action 66 (-3.9, lag 2) and Action 67 (-17.1, lag 3).
  - Us closes Action 69 (+4.9) → VladNet breaks Action 70 (-8.3, lag 1) and Action 71 (-15.2, lag 2).
  - Us closes Action 81 (+12.6) → VladNet breaks Action 82 (-12.6, lag 1) and Action 83 (-5.8, lag 2).
  - Us closes Action 89 (+18.8) → VladNet breaks Action 90 (-18.8, lag 1) and Action 91 (-17.2, lag 2).
  - Us closes Action 101 (+3.8) → VladNet breaks Action 102 (-5.2, lag 1) and Action 103 (-19.4, lag 2).
  - Us closes Action 105 (+10.6) → VladNet breaks Action 106 (-10.6, lag 1).
  - Us closes Action 109 (+10.6) → VladNet breaks Action 110 (-3.0, lag 1) and Action 111 (-9.3, lag 2).
  - Us closes Action 117 (+10.6) → VladNet breaks Action 118 (-10.6, lag 1).
  - **Lag summary**: 12/21 break sequences occur at **lag 1** (immediate reply), 9/21 at **lag 2** (same pair-turn). Over 85% of break damage lands within ≤2 actions of our close.
- **Game 1c69056c (30 breaks dealt, -179.3 total area loss to us, 4 catastrophic breaks ≥10.0)**:
  - Punishes our closes at Action 40 (lag 1, -2.4), Action 41 (lag 2, -2.1), Action 60 (lag 1, -2.4), Action 61 (lag 2, -4.5), Action 64 (lag 1, -6.0), Action 72 (lag 1, -3.0), Action 84 (lag 1, -9.0), Action 88 (lag 1, -4.7), Action 108 (lag 1, -15.2), Action 120 (lag 10, -28.4).

### 1.4 Bank Sizes at 40 / 60 / 80

| Game ID | Metric | Move 40 | Move 60 | Move 80 | Final (120) |
|---------|--------|---------|---------|---------|-------------|
| **3a414aad** (VladNet Red) | Rival Area | **49.2** | **50.8** | **49.6** | **49.2** |
| | Our Area | 43.4 | 27.0 | 24.1 | 15.3 |
| | Delta (Rival - Us) | +5.8 | +23.8 | +25.5 | +33.9 |
| **1c69056c** (VladNet Blue) | Rival Area | **60.0** | **48.0** | **33.0** | **35.3** |
| | Our Area | 36.2 | 30.1 | 20.9 | 19.7 |
| | Delta (Rival - Us) | +23.8 | +17.9 | +12.1 | +15.6 |

In `3a414aad`, VladNet locked in a **49.2 area bank by move 40 and held it flat through move 120 (49.2)** without giving up an inch.

### 1.5 Bank Fragility

- **3a414aad (VladNet Red)**:
  - Rival peak area: 53.2 | Final area: 49.2
  - Breaks suffered by rival: 7 breaks, total loss **31.3 area**
  - Catastrophic breaks suffered (≥10.0): **1** (Action 113: -15.0 area, quickly neutralized)
  - Contrast with Us: 34 breaks suffered, **314.0 area lost**, 12 catastrophic breaks
  - **Fragility**: **EXTREMELY LOW** as Red (retains 92.5% of peak area). Redundant nodes prevent cascading collapse.
- **1c69056c (VladNet Blue)**:
  - Rival peak area: 64.8 | Final area: 35.3
  - Breaks suffered by rival: 9 breaks, total loss **111.9 area**
  - Catastrophic breaks suffered (≥10.0): 6 (Actions 43, 67, 78, 86, 103, 119)
  - Contrast with Us: 30 breaks suffered, **179.3 area lost**, 4 catastrophic breaks
  - **Fragility**: **MODERATE** as Blue (retains 54.5% of peak area). When we play Red and contest aggressively, VladNet Blue bleeds area, but we bled even more (19.7 final).

### 1.6 Reply Habits (Greedy-First % on 20 Sampled Positions)

Evaluated `analyze_with_avoid(budget=4096).candidates.first().mv` vs VladNet's actual move on 20 midgame positions (Actions 12–50):

| Game ID | Action | Player | Predicted Greedy Move | Actual Played Move | Matched? | Candidates |
|---------|--------|--------|-----------------------|--------------------|----------|------------|
| 3a414aad | 14 | Red | `(6,1)-(3,3)` / `P11-M13` | `(3,3)-(6,1)` / `M13-P11` | NO | 8 |
| 3a414aad | 18 | Red | `(3,3)-(3,0)` / `M13-M10` | `(3,3)-(5,0)` / `M13-O10` | NO | 8 |
| 3a414aad | 22 | Red | `(3,3)-(2,0)` / `M13-L10` | `(6,0)-(3,-3)` / `P10-M7` | NO | 8 |
| 3a414aad | 26 | Red | `(3,3)-(2,0)` / `M13-L10` | `(3,-3)-(6,-6)` / `M7-P4` | NO | 8 |
| 3a414aad | 30 | Red | `(3,-3)-(3,-6)` / `M7-M4` | `(6,-6)-(3,-4)` / `P4-M6` | NO | 8 |
| 3a414aad | 34 | Red | `(3,-4)-(3,-7)` / `M6-M3` | `(5,0)-(3,-3)` / `O10-M7` | NO | 8 |
| 3a414aad | 38 | Red | `(3,3)-(1,0)` / `M13-K10` | `(6,6)-(3,3)` / `P16-M13` | NO | 8 |
| 3a414aad | 42 | Red | `(3,-4)-(1,-2)` / `M6-K8` | `(3,-4)-(1,-1)` / `M6-K9` | NO | 8 |
| 3a414aad | 46 | Red | `(1,0)-(0,-2)` / `K10-J8` | `(-1,-1)-(-4,-3)` / `I9-F7` | NO | 8 |
| 3a414aad | 50 | Red | `(-4,0)-(-7,-1)` / `F10-C9` | `(-4,0)-(-7,-1)` / `F10-C9` | **YES** | 8 |
| 1c69056c | 12 | Blue | `(-9,-3)-(-9,0)` / `A7-A10` | `(-6,0)-(-3,3)` / `D10-G13` | NO | 8 |
| 1c69056c | 16 | Blue | `(-9,-3)-(-9,0)` / `A7-A10` | `(-3,3)-(-5,0)` / `G13-E10` | NO | 8 |
| 1c69056c | 20 | Blue | `(-5,0)-(-2,0)` / `E10-H10` | `(-6,0)-(-3,-3)` / `D10-G7` | NO | 8 |
| 1c69056c | 24 | Blue | `(-3,3)-(-2,0)` / `G13-H10` | `(-3,-3)-(-6,-1)` / `G7-D9` | NO | 8 |
| 1c69056c | 28 | Blue | `(-3,3)-(-2,0)` / `G13-H10` | `(-9,3)-(-6,6)` / `A13-D16` | NO | 8 |
| 1c69056c | 32 | Blue | `(-3,3)-(-2,0)` / `G13-H10` | `(-3,-3)-(-6,-6)` / `G7-D4` | NO | 8 |
| 1c69056c | 36 | Blue | `(-9,-3)-(-9,-6)` / `A7-A4` | `(-3,-3)-(-1,0)` / `G7-I10` | NO | 8 |
| 1c69056c | 40 | Blue | `(-1,0)-(0,-2)` / `I10-J8` | `(-1,0)-(1,3)` / `I10-K13` | NO | 8 |
| 1c69056c | 44 | Blue | `(-9,-3)-(-9,-6)` / `A7-A4` | `(1,3)-(4,4)` / `K13-N14` | NO | 8 |
| 1c69056c | 48 | Blue | `(7,2)-(8,1)` / `Q12-R11` | `(4,4)-(7,1)` / `N14-Q11` | NO | 8 |

- **Greedy-first match rate**: **1 / 20 = 5.0%** (3a414aad: 1/10 = 10%, 1c69056c: 0/10 = 0%).
- **Behavioral signature**: Our 2-ply search repeatedly expects VladNet to play local short connects or edge consolidations (`cand_count=8`). Instead, VladNet completely ignores greedy local connects to extend long-range structural anchors across the board (`P16-M13`, `A13-D16`, `G7-I10`, `K13-N14`).

---

## 2. Rival Playbook: Great Barrier (GB / GB 2.0)

Great Barrier is a macroeconomic wall builder. It avoids early small triangles, drives corridors diagonally straight to the board edges, and closes monolithic 50+ area banks by move ~40.

### 2.1 Opener (First 6 Moves)

#### Game 30c7653b (GB as Blue)
- **First 6 game actions**:
  1. Action 1 [Blue / GB]: `(-6,0)-(-7,-3)` / `D10-C7` (Extend, +0.0)
  2. Action 2 [Red / Us]: `(6,0)-(3,-2)` / `P10-M8` (Extend, +0.0)
  3. Action 3 [Red / Us]: `(3,-2)-(0,0)` / `M8-J10` (Extend, +0.0)
  4. Action 4 [Blue / GB]: `(-6,0)-(-5,2)` / `D10-E12` (Extend, +0.0)
  5. Action 5 [Blue / GB]: `(-6,0)-(-4,3)` / `D10-F13` (Extend, +0.0)
  6. Action 6 [Red / Us]: `(0,0)-(3,3)` / `J10-M13` (Extend, +0.0)
- **GB's first 6 individual moves (Actions 1, 4, 5, 8, 9, 12)**:
  - Action 1: `(-6,0)-(-7,-3)` / `D10-C7` (Extend, +0.0)
  - Action 4: `(-6,0)-(-5,2)` / `D10-E12` (Extend, +0.0)
  - Action 5: `(-6,0)-(-4,3)` / `D10-F13` (Extend, +0.0)
  - Action 8: `(-5,2)-(-3,5)` / `E12-G15` (Extend, +0.0)
  - Action 9: `(-4,3)-(-2,6)` / `F13-H16` (Extend, +0.0)
  - Action 12: `(-3,5)-(-1,8)` / `G15-I18` (Extend, +0.0)

#### Game ad65f054 (GB as Red)
- **First 6 game actions**:
  1. Action 1 [Blue / Us]: `(-6,0)-(-7,-3)` / `D10-C7` (Extend, +0.0)
  2. Action 2 [Red / GB]: `(6,0)-(5,2)` / `P10-O12` (Extend, +0.0)
  3. Action 3 [Red / GB]: `(6,0)-(4,3)` / `P10-N13` (Extend, +0.0)
  4. Action 4 [Blue / Us]: `(-6,0)-(-6,3)` / `D10-D13` (Extend, +0.0)
  5. Action 5 [Blue / Us]: `(-6,0)-(-3,-2)` / `D10-G8` (Extend, +0.0)
  6. Action 6 [Red / GB]: `(5,2)-(3,5)` / `O12-M15` (Extend, +0.0)
- **GB's first 6 individual moves (Actions 2, 3, 6, 7, 10, 11)**:
  - Action 2: `(6,0)-(5,2)` / `P10-O12` (Extend, +0.0)
  - Action 3: `(6,0)-(4,3)` / `P10-N13` (Extend, +0.0)
  - Action 6: `(5,2)-(3,5)` / `O12-M15` (Extend, +0.0)
  - Action 7: `(4,3)-(2,6)` / `N13-L16` (Extend, +0.0)
  - Action 10: `(3,5)-(1,8)` / `M15-K18` (Extend, +0.0)
  - Action 11: `(2,6)-(0,9)` / `L16-J19` (Extend, +0.0)

#### Game d2d4b4fd (GB 2.0 as Red)
- **First 6 game actions**:
  1. Action 1 [Blue / Us]: `(-6,0)-(-4,2)` / `D10-F12` (Extend, +0.0)
  2. Action 2 [Red / GB2]: `(6,0)-(3,0)` / `P10-M10` (Extend, +0.0)
  3. Action 3 [Red / GB2]: `(3,0)-(3,3)` / `M10-M13` (Extend, +0.0)
  4. Action 4 [Blue / Us]: `(-6,0)-(-6,3)` / `D10-D13` (Extend, +0.0)
  5. Action 5 [Blue / Us]: `(-6,0)-(-3,-2)` / `D10-G8` (Extend, +0.0)
  6. Action 6 [Red / GB2]: `(3,0)-(2,3)` / `M10-L13` (Extend, +0.0)
- **GB 2.0's first 6 individual moves (Actions 2, 3, 6, 7, 10, 11)**:
  - Action 2: `(6,0)-(3,0)` / `P10-M10` (Extend, +0.0)
  - Action 3: `(3,0)-(3,3)` / `M10-M13` (Extend, +0.0)
  - Action 6: `(3,0)-(2,3)` / `M10-L13` (Extend, +0.0)
  - Action 7: `(2,3)-(3,6)` / `L13-M16` (Extend, +0.0)
  - Action 10: `(3,3)-(3,6)` / `M13-M16` (Connect, +3.0 area)
  - Action 11: `(3,0)-(3,-3)` / `M10-M7` (Extend, +0.0)

**Key Opener Signature**: GB NEVER closes a 0-gain or small 4.5 triangle on turn 1. Instead, GB plays parallel dual rails (`D10-E12`, `D10-F13` or `P10-O12`, `P10-N13`) running straight to the board boundary at `(0,9)` / `J19`.

### 2.2 First-Wall Move + Size

- **Game 30c7653b (GB Blue)**:
  - First wall close: Action 16, Move `(-1,8)-(0,9)` / `I18-J19`, gain **+2.5 area** (total 2.5).
  - First major bank (gain ≥ 10.0): Action 41, Move `(-3,-9)-(0,-9)` / `G1-J1`, gain **+51.5 area** (total 56.5).
- **Game ad65f054 (GB Red)**:
  - First wall close: Action 14, Move `(1,8)-(0,9)` / `K18-J19`, gain **+2.5 area** (total 2.5).
  - First major bank (gain ≥ 10.0): Action 39, Move `(3,-9)-(0,-9)` / `M1-J1`, gain **+51.5 area** (total 56.5).
- **Game d2d4b4fd (GB 2.0 Red)**:
  - First wall close: Action 10, Move `(3,3)-(3,6)` / `M13-M16`, gain **+3.0 area** (total 3.0).
  - First major bank (gain ≥ 10.0): Action 39, Move `(9,3)-(9,0)` / `S13-S10`, gain **+54.0 area** (total 63.0).

GB/GB2 closes its first tiny contact at Action 10–16 (+2.5 to +3.0 area), but completes an immense **monolithic bank of 51.5–54.0 area exactly at Action 39–41**.

### 2.3 Break Timing vs Us

GB lets us play into corridors for the first 50 actions while it builds its boundary wall. Once its wall is secured (Action 50+), it launches brutal multi-action break barrages that pop our banks in late game.

- **Game 30c7653b (21 breaks dealt, -188.8 area lost by us, 8 catastrophic breaks ≥10.0)**:
  - Action 56 & 57: breaks us for -4.5 and -17.0 (lag 5 and 6 from our close at 51).
  - Action 60: breaks us for -2.7 (lag 1 from close at 59).
  - Action 68: breaks us for -14.7 (lag 1 from close at 67).
  - Action 84 & 85: breaks us for -3.0 and -20.2 (lag 1 and 2 from close at 83).
  - Action 100, 101, 104: breaks us for -1.8, -3.4, -10.2 (lags 1, 2, 2 from close at 99, 102).
  - Action 108 & 109: breaks us for -3.0 and -21.2 (lags 2 and 3 from close at 106).
  - Action 112 & 113: breaks us for -4.5 and -6.0 (lags 1 and 2 from close at 111).
  - Action 120: breaks us for -22.8 (lag 6 from close at 114).
- **Game ad65f054 (21 breaks dealt, -152.5 area lost by us, 4 catastrophic breaks ≥10.0)**:
  - Punishes our closes at Action 54 (lag 1, -6.2), Action 62 (lag 1, -5.8), Action 66 (lag 1, -8.6), Action 74 (lag 1, -3.2), Action 82 (lag 1, -7.0), Action 86 (lag 1, -4.5), Action 114 (lag 2, -22.5), Action 115 (lag 3, -12.6), Action 118 (lag 2, -13.5), Action 119 (lag 3, -13.1).
- **Game d2d4b4fd (21 breaks dealt, -233.5 area lost by us, 10 catastrophic breaks ≥10.0)**:
  - Punishes our closes at Action 54 (lag 1, -18.0), Action 70 (lag 1, -3.9), Action 91 (lag 7, -26.1), Action 94 (lag 1, -7.6), Action 95 (lag 2, -16.5), Action 103 (lag 2, -26.1), Action 107 (lag 2, -8.7), Action 110 (lag 5, -13.1), Action 111 (lag 6, -16.7), Action 119 (lag 2, -14.8).

**Key Break Signature**: GB holds back on breaking until move 50+. Then, across all 3 games, GB deals exactly **21 breaks** per game, targeting freshly closed corridors immediately on lag 1–2, and dealing massive catastrophic pops (-15 to -26 area) in moves 80–120.

### 2.4 Bank Sizes at 40 / 60 / 80

| Game ID | Metric | Move 40 | Move 60 | Move 80 | Final (120) |
|---------|--------|---------|---------|---------|-------------|
| **30c7653b** (GB Blue) | Rival Area | **5.0** | **108.0** | **113.5** | **112.5** |
| | Our Area | 52.6 | 43.7 | 56.9 | 47.8 |
| | Delta (Rival - Us) | -47.6 | **+64.3** | **+56.6** | **+64.7** |
| **ad65f054** (GB Red) | Rival Area | **56.5** | **108.0** | **115.2** | **121.8** |
| | Our Area | 41.6 | 51.9 | 55.8 | 54.0 |
| | Delta (Rival - Us) | +14.9 | **+56.1** | **+59.4** | **+67.8** |
| **d2d4b4fd** (GB2 Red) | Rival Area | **63.0** | **126.0** | **126.0** | **117.0** |
| | Our Area | 49.4 | 55.1 | 55.7 | 63.6 |
| | Delta (Rival - Us) | +13.6 | **+70.9** | **+70.3** | **+53.4** |

**Decisive Metric**: Between Move 40 and Move 60, GB explodes from 5.0–63.0 area to **108.0–126.0 area**, establishing a +55 to +70 area lead that never shrinks.

### 2.5 Bank Fragility

- **30c7653b (GB Blue)**:
  - Rival peak area: 133.5 | Final area: 112.5
  - Breaks suffered: 11 breaks, total loss **97.5 area**
  - Catastrophic breaks suffered (≥10.0): 2 (Action 86: -10.1, Action 118: -21.0)
  - Retains **84.3%** of peak bank area despite 11 breaks dealt by us.
- **ad65f054 (GB Red)**:
  - Rival peak area: 126.2 | Final area: 121.8
  - Breaks suffered: 11 breaks, total loss **55.4 area**
  - Catastrophic breaks suffered (≥10.0): **0**! ZERO catastrophic breaks in 120 moves!
  - Retains **96.5%** of peak bank area.
- **d2d4b4fd (GB 2.0 Red)**:
  - Rival peak area: 144.0 | Final area: 117.0
  - Breaks suffered: 10 breaks, total loss **88.2 area**
  - Catastrophic breaks suffered (≥10.0): 3 (Action 100: -22.5, Action 104: -10.2, Action 108: -13.8)
  - Retains **81.3%** of peak bank area.

**Fragility Verdict**: **NEAR-ZERO FRAGILITY**. GB builds deep walls backed by outer edges and redundant interior trusses. Even when we score 10–11 cuts against GB, its bank remains above 112 area. Meanwhile, our bank suffers 21 breaks per game and loses 150–233 area.

### 2.6 Reply Habits (Greedy-First % on 20 Sampled Positions)

Evaluated `analyze_with_avoid(budget=4096).candidates.first().mv` vs GB's actual move on 20 sampled positions across `30c7653b`, `ad65f054`, and `d2d4b4fd`:

| Game ID | Action | Player | Predicted Greedy Move | Actual Played Move | Matched? | Candidates |
|---------|--------|--------|-----------------------|--------------------|----------|------------|
| 30c7653b | 12 | Blue | `(-4,3)-(-7,3)` / `F13-C13` | `(-3,5)-(-1,8)` / `G15-I18` | NO | 8 |
| 30c7653b | 16 | Blue | `(-7,-3)-(-9,0)` / `C7-A10` | `(-1,8)-(0,9)` / `I18-J19` | NO | 8 |
| 30c7653b | 24 | Blue | `(-7,-3)-(-4,-3)` / `C7-F7` | `(-4,-3)-(-2,-6)` / `F7-H4` | NO | 8 |
| 30c7653b | 32 | Blue | `(-9,0)-(-6,-3)` / `A10-D7` | `(-9,0)-(-9,-3)` / `A10-A7` | NO | 8 |
| 30c7653b | 40 | Blue | `(-1,8)-(2,5)` / `I18-L15` | `(-6,-9)-(-3,-9)` / `D1-G1` | NO | 8 |
| 30c7653b | 48 | Blue | `(-1,8)-(2,5)` / `I18-L15` | `(-9,6)-(-9,9)` / `A16-A19` | NO | 8 |
| 30c7653b | 56 | Blue | `(-1,8)-(2,5)` / `I18-L15` | `(-1,8)-(2,5)` / `I18-L15` | **YES** | 8 |
| ad65f054 | 14 | Red | `(2,6)-(3,9)` / `L16-M19` | `(1,8)-(0,9)` / `K18-J19` | NO | 8 |
| ad65f054 | 22 | Red | `(4,-3)-(7,-3)` / `N7-Q7` | `(4,-3)-(2,-6)` / `N7-L4` | NO | 8 |
| ad65f054 | 30 | Red | `(9,0)-(8,-3)` / `S10-R7` | `(9,0)-(9,-3)` / `S10-S7` | NO | 8 |
| ad65f054 | 38 | Red | `(9,-3)-(6,0)` / `S7-P10` | `(6,-9)-(3,-9)` / `P1-M1` | NO | 8 |
| ad65f054 | 46 | Red | `(4,3)-(2,1)` / `N13-L11` | `(9,6)-(9,9)` / `S16-S19` | NO | 8 |
| ad65f054 | 54 | Red | `(4,3)-(2,1)` / `N13-L11` | `(4,3)-(2,1)` / `N13-L11` | **YES** | 8 |
| ad65f054 | 62 | Red | `(-6,-4)-(-8,-3)` / `D6-B7` | `(-6,-4)-(-8,-3)` / `D6-B7` | **YES** | 8 |
| d2d4b4fd | 14 | Red | `(6,0)-(3,3)` / `P10-M13` | `(3,0)-(2,-3)` / `M10-L7` | NO | 8 |
| d2d4b4fd | 22 | Red | `(6,0)-(3,3)` / `P10-M13` | `(3,6)-(2,9)` / `M16-L19` | NO | 8 |
| d2d4b4fd | 30 | Red | `(3,9)-(0,6)` / `M19-J16` | `(2,9)-(3,9)` / `L19-M19` | NO | 8 |
| d2d4b4fd | 38 | Red | `(2,3)-(-1,2)` / `L13-I12` | `(9,6)-(9,3)` / `S16-S13` | NO | 8 |
| d2d4b4fd | 46 | Red | `(3,-9)-(6,-9)` / `M1-P1` | `(9,-6)-(9,-9)` / `S4-S1` | NO | 8 |
| d2d4b4fd | 54 | Red | `(-1,2)-(-3,4)` / `I12-G14` | `(-1,2)-(-3,4)` / `I12-G14` | **YES** | 8 |

- **Greedy-first match rate**: **4 / 20 = 20.0%** (30c7653b: 1/7 = 14.3%, ad65f054: 2/7 = 28.6%, d2d4b4fd: 1/6 = 16.7%).
- **Phase breakdown**:
  - Opening / Mid (Actions 12–48): **0 / 16 = 0.0%** match. GB relentlessly marches along board edges (`A16-A19`, `S16-S19`, `D1-G1`, `P1-M1`) while 2-ply expects local interior cuts.
  - Action 54+ (Late Tactical Breaks): **4 / 4 = 100.0%** match. When tactical breaks open up, GB switches directly to decisive greedy cuts.

---

## 3. Cross-Rival Comparison Summary

| Attribute | VladNet | Great Barrier (GB / GB2) | Riposte v7 (Us) |
|-----------|---------|--------------------------|-----------------|
| **Opener philosophy** | Turn-1 tight 4.5 triangle + central thicket | Dual diagonal rails straight to board edges | Fixed mesh8 (Blue) or unscripted (Red) |
| **First wall timing** | Action 3 (Red) / Action 5 (Blue) | Action 10–16 (small), Action 39–41 (massive) | Action 10–14 |
| **First major wall size** | +15.0 area at Action 27–29 | **+51.5 to +54.0 area** at Action 39–41 | +12 to +15 area |
| **Bank size at Move 60** | 48.0–50.8 area | **108.0–126.0 area** | 27.0–55.1 area |
| **Bank fragility (final/peak)** | 92.5% (Red), 54.5% (Blue) | **81.3% to 96.5%** | 24% to 73% (frequent late collapse) |
| **Break policy** | Reactive sniper: lag 1–2 on our closes | Phase gated: quiet moves 1–50, 21 breaks moves 50–120 | Random / opportunistic shallow cuts |
| **Greedy-first reply match** | **5.0%** (neural net pathing) | **20.0%** (0% early edge race, 100% late cuts) | Assumed 100% in beam search |

---

## 4. Habits-to-Exploit Table

| Habit | Evidence | Killing Lane |
|-------|----------|--------------|
| **1. GB Uncontested Boundary Wall Sprint (Moves 1–40)** | GB builds dual rails (`D10-E12`, `D10-F13` in `30c7653b`; `P10-O12`, `P10-N13` in `ad65f054`) unhindered to edge `J19`, yielding +51.5 bank on Action 39–41. Bank hits 108.0–126.0 at move 60 and never breaks (121.8 final in `ad65f054`). 0% greedy match moves 12–48. | **Lane W (WALL-RACE 1.0)**: Shared-node census at move 16–20. Detect corridor root towards boundary, drop single blocking stone at root (`E12` or `O12`). Denies 50+ area bank for 1 move cost. |
| **2. VladNet Counter-Punch Sniper (Lag 1–2 Break)** | VladNet breaks our closes on lag 1–2 in 12/21 sequences (`3a414aad`), dealing -314.0 area across 34 breaks, farming 127 donated area across 10 re-close cycles. VladNet never hunts proactive breaks if we do not close. | **Lane P (Rebuild-Denial Gate & Price-per-Move)**: Forbid re-closing broken corridors (`re_place_blocked_count >= 2`). Ban Connects <2.0 area when open space >20 remains. Starves VladNet of break targets. |
| **3. GB Turn 1–30 Zero-Area Passivity** | In `30c7653b`, GB area is only 5.0 at Action 40 while we had 52.6. GB completely ignores center area and local cuts (0% greedy match in 16 sampled positions) to anchor edges. | **Lane W + Lane R (Steal Wall & Cut Boundary Root)**: Use pair-turn tempo (moves 1–2 Red) to intercept GB's boundary anchor before Action 14, turning GB's open rails into dead endpoints. |
| **4. VladNet Blue Fragility Under Contact** | As Blue (`1c69056c`), VladNet suffered 6 catastrophic breaks (Actions 43, 67, 78, 86, 103, 119) and area fell from 64.8 peak to 35.3. Unlike its Red form, its Blue banks bleed heavily when pressured. | **Lane R (Red Opener Design)**: Exploit VladNet Blue with high-tempo Red invasion lines that sever thicket roots before Action 29 (+15 wall). |
| **5. Search Assumption Collapse (Depth is Theater)** | VladNet greedy reply match is only 5.0% (`3a414aad`, `1c69056c`). GB opening/mid reply match is 0.0% (`30c7653b`, `ad65f054`, `d2d4b4fd`). Our search deepened beam assuming `ranked().first()` replies that rivals play ≤20% of the time. | **Lane S2 / M7 (Opponent-Aware Reply Distribution)**: Gate M2 port on rival-aware reply models. Replace `ranked().first()` assumption with rival profile distributions (macro edge-wall vs contact sniper). |

---

*Authored by Game Analyst Agent for Riposte v8 Strategy Core*  
*Target Branch: `lane-v8-rivals`*
