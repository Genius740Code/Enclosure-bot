# Lane X — XBot one_move_potential sweep (Steal 1) — 2026-09-28

**Base**: lane-b-eval tip (doom-OFF control + mem variants). Branch: `lane-x-eval`.
**Term**: `POT_W * (pot_me - pot_opp) * hz` in full-horizon points (area × min(events, 12)).
**Potential definition**: unordered pairs of own nodes with Chebyshev distance ≤ 3
(one legal action could join them). Global, both sides, every node (not triangle-
specific like `loop_bonus`).

**Gates per variant** (n≥8, both colors):
- vs v1: 10 games (5 openings × 2 colors)
- probe_league: scoutbase, 8 games (skip 0/10/20/30 × 2 colors)
- gauge: greedy max-area 1-ply, 6 games (alternating colors)

**Targets**: missed-closes/game FALLS; v1 ≥6/10; league better than -9.9%; no collapse loss worse than -300.

---

## Sweep Results

### POT_W = 0.0 (control, doom-OFF baseline)
```
v1: 5/10 | as-blue 1/5 avg -16.7% | as-red 4/5 avg +9.2%
league: -9.9% (skip=0 blue +29.0%, red +39.8%; skip=10 red -94.5%; skip=20 red -94.5%; skip=30 red -55.9%)
gauge: 6/6 (blue +53.5%, red +69.3%)
missed-closes: laneB=11.56, v1=19.13
```

### POT_W = 0.25
```
v1: 3/10 | as-blue 1/5 avg -20.4% | as-red 2/5 avg -12.3%
league: -21.9% (skip=0 red +15.1%; skip=10 red -73.3%; skip=20 red -92.2%; skip=30 red -29.7%)
gauge: 6/6 (blue +70.2%, red +76.4%)
missed-closes: laneB=23.81, v1=18.96
```

### POT_W = 0.5
```
v1: 3/10 | as-blue 1/5 avg -48.5% | as-red 2/5 avg -24.8%
league: -34.7% (skip=0 red -59.2%; skip=10 red -60.4%; skip=20 red -98.7%; skip=30 red -43.1%)
gauge: 6/6 (blue +62.0%, red +73.3%)
missed-closes: laneB=72.15, v1=16.32
```

### POT_W = 1.0
```
v1: 2/10 | as-blue 0/5 avg -169.7% | as-red 2/5 avg -23.6%
league: -70.0% (skip=0 red -24.9%; skip=10 red -127.8%; skip=20 red -69.3%; skip=30 red -108.6%)
gauge: 6/6 (blue +46.7%, red +57.2%)
missed-closes: laneB=192.75, v1=16.49
```

### POT_W = 2.0
```
v1: 0/10 | as-blue 0/5 avg -218.0% | as-red 0/5 avg -213.0%
league: -135.1% (skip=0 red -181.6%; skip=10 red -96.8%; skip=20 red -114.8%; skip=30 red -128.2%)
gauge: 3/6 (blue -75.1%; red +4.9%)
missed-closes: laneB=349.39, v1=18.54
```

---

## Per-Game Details (v1 gate)

### POT_W=0.0
| opening | as-Blue | as-Red |
|---|---|---|
| None | 1020-826 W (+19.1%) | 1239-893 W (+28.0%) |
| 4864 | 1037-1286 L (-24.1%) | 1257-1150 W (+8.5%) |
| 5589 | 1160-1604 L (-38.3%) | 1298-1320 L (-1.7%) |
| 11723 | 1857-2013 L (-8.4%) | 1343-1259 W (+6.3%) |
| 9199 | 1649-2173 L (-31.8%) | 1448-1374 W (+5.2%) |

### POT_W=0.25
| opening | as-Blue | as-Red |
|---|---|---|
| None | 1520-1500 W (+1.3%) | 895-1471 L (-64.4%) |
| 4864 | 1389-1503 L (-8.2%) | 1807-1405 W (+22.2%) |
| 5589 | 1098-1474 L (-34.3%) | 2218-2547 L (-14.9%) |
| 11723 | 1562-2174 L (-39.2%) | 1405-1919 L (-36.6%) |
| 9199 | 1305-1587 L (-21.6%) | 1490-1009 W (+32.3%) |

### POT_W=0.5
| opening | as-Blue | as-Red |
|---|---|---|
| None | 1493-2288 L (-53.3%) | 973-1627 L (-67.2%) |
| 4864 | 1547-1401 W (+9.4%) | 1810-1552 W (+14.3%) |
| 5589 | 2080-3655 L (-75.7%) | 1937-2354 L (-21.5%) |
| 11723 | 1381-2373 L (-71.8%) | 1477-2290 L (-55.0%) |
| 9199 | 1458-2206 L (-51.3%) | 1146-1086 W (+5.2%) |

### POT_W=1.0
| opening | as-Blue | as-Red |
|---|---|---|
| None | 1802-3833 L (-112.8%) | 1840-1802 W (+2.0%) |
| 4864 | 1310-4581 L (-249.6%) | 1128-1484 L (-31.6%) |
| 5589 | 1331-3809 L (-186.3%) | 1486-2602 L (-75.0%) |
| 11723 | 1369-4263 L (-211.4%) | 1895-2247 L (-18.5%) |
| 9199 | 1389-2619 L (-88.6%) | 1926-1825 W (+5.2%) |

### POT_W=2.0
| opening | as-Blue | as-Red |
|---|---|---|
| None | 1190-3864 L (-224.7%) | 1485-4182 L (-181.6%) |
| 4864 | 1282-4475 L (-249.0%) | 1485-4836 L (-225.7%) |
| 5589 | 1348-3743 L (-177.7%) | 1518-4381 L (-188.7%) |
| 11723 | 1314-4323 L (-229.0%) | 1394-4332 L (-210.8%) |
| 9199 | 1436-4446 L (-209.6%) | 1246-4464 L (-258.3%) |

---

## Analysis

**Monotonic degradation** on every gate as POT_W increases:
- v1 W/L: 5 → 3 → 3 → 2 → 0
- League margin: -9.9% → -21.9% → -34.7% → -70.0% → -135.1%
- Gauge: 6/6 throughout but margins degrade; collapses to 3/6 at 2.0
- Missed-closes/game: 11.6 → 23.8 → 72.2 → 192.8 → 349.4 (rises at every dose)

**Mechanism**: The potential term rewards node density (pairs within distance 3).
The bot builds tight clusters to maximize potential, but these clusters:
1. Are easily cut (dense = many edges in small area = many cut targets)
2. Don't convert potential to actual closes (potential ≠ area)
3. Get farmed by opponents who cut once and bank the area

The "missed-closes" metric (avg_potential - actual_closes) explodes because
potential grows faster than closes — the bot chases the proxy without scoring.

**As-Blue chair** (forced openings) collapses first and hardest:
- At POT_W=0.25: as-Blue already 1/5 (same as control) but margins worse
- At POT_W=0.5: as-Blue margins -48.5% (vs -16.7% control)
- At POT_W=1.0: as-Blue 0/5, margins -169.7%
- At POT_W=2.0: as-Blue 0/5, margins -218.0%

**No dose dominates** the control on any gate. The control (POT_W=0.0, doom-OFF)
remains the best configuration across all three gates.

---

## Verdict

**REJECTED**. The XBot one_move_potential proxy is harmful in our engine.
It creates a perverse incentive to build dense, cuttable clusters that
opponents farm, without converting to durable area. The term fundamentally
misunderstands the game: proximity ≠ scoring threat in a cut-heavy environment.

`POT_W` remains at 0.0 with provenance comment in `eval_phases.rs`.
No further doses need testing — trend is uniformly and catastrophically negative.