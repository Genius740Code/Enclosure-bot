# findings-endgame.md — ENDGAME track (horizon-aware endgame mode)

## TOP-LINE SUMMARY
- HEADLINE: `botE-endgame.js` — endgame mode firing at `(lineLimit − moveNumber) ≤ 12` (opponent-denial 0.15 → 0.6 in the eval, future-area potential weight 2 → 0.5 and break weight 1.5 → 4 in candidate ranking) — went **3W-3L-0D vs `bot-tourney.js`** over 6 alternating-color games @ 2000ms, avgMargin(A) **−177.4**. Baseline control (`bot-tourney` vs `bot-tourney`, same harness): **3W-3L-0D, avgMargin +123.3**. Every win as Red / every loss as Blue in BOTH runs — color decided everything, botE's margins sit inside the control's variance band. **NOT merge-worthy** (bar: 4+ wins or consistent positive margin both colors). Keep `bot-tourney.js` as-is.
- WHY neutral (mechanism, engine-verified): in builder-vs-builder endgames the enemy loop is **out of reach** — `probeE-reach.js` on 12 endgame states: ~277 legal candidates each, ~15 candidates break *some* enemy edge, but **0 candidates destroy banked area** (a cut only helps if it zeroes a loop; frontier-edge cuts remove 0 area). The 0.6 denial weight has nothing to act on. Denial moves only exist in frontier brawls (vs neck/aggro archetypes) — and there areas are tiny (final 5–30), so the endgame delta is small too.
- Mined data (114 real tournament games): only **8/114 reach line 108** — 67 end by resignation, 40 by abandonment long before the window. In those 8: winners dealt **2.38 area-killing breaks/game** (7/8 dealt 2–4, distribution 0:1/2:3/3:3/4:1), denying 49.9 area; endgame banked-swing avg just **+18.9**; 3/8 games flipped leader in the window. So the thesis is half-right: winners DO close with bank-and-deny endgames *when denial is reachable* — but the last-12 window decides almost nothing; mid-game compounding decides ~93% of real games. The bigger lever is Track B's mid-game denial underpricing (0.15), applied from earlier — not a line-108 switch.
- Byte-identity: **PASS** — 10/10 non-endgame test states return moves identical to `bot-tourney.js` at full budget (0 violations; verifyE.js). Endgame states diverge where it matters: 1/4 in the probe set, and **13/20 endgame turns** in the match vs the control's **2/18** timing-noise floor — the mode genuinely fires and changes moves; the changed moves just don't shift results.
- Job 3 (double-connect/neck probe): **RAN** — `botE-neck.js` (shared-endpoint reinforcement bonus 2.0, always on) vs `chall-aggro`: **0W-2L** but avgMargin **−141.6 vs −299.1** for botE-endgame against the same break-heavy opponent — better BOTH colors (blue −122.1 vs −204.6, red −161.1 vs −393.5). First positive signal of the session; still loses (aggro beats the whole bot family, cf. Track B), 2-game sample with 2000ms timing noise → not merge-worthy, but worth a follow-up sweep. Mechanical caveat confirmed first: in this engine redundancy inside the *same* loop does not protect banking (one cut zeroes the loop regardless); only out-of-reach distance does.

## Method
- `botE-endgame.js` (NEW): copy of `bot-tourney.js`'s time-boxed 2-ply, same fallback chain (2-ply → 1-ply → first-legal → timeout), same `adaptiveBudget`/`searchBudget`, plus an endgame mode when `(lineLimit − moveNumber) ≤ 12`:
  - (a) candidate ranking `areaGain×2 + breaks×1.5` (via `BOT.quickScore`) → `areaGain×0.5 + breaks×4` (potential ~0, breaks up),
  - (b) eval `ownGain − oppGain×0.15` → `ownGain − oppGain×0.6`,
  - (c) break bonus 1.5 → 4 (in ranking; the eval credits breaks via the 0.6 oppGain reduction — an endgame break permanently removes enemy banking, no rebuild time).
  Non-endgame path is byte-identical by construction (weights parameterized, mid values = tourney's).
- `verifyE.js` (NEW): 14 test states from cheap tourney-vs-tourney play (moves 0–116); both bots at full 30s budget (search completes → result is a pure function of state); compares moves exactly.
- `matchE.js` (NEW): 6 games alternating colors, per-turn budget arg; per game logs W-L + margin + endgame-window (last-12-moves) start margin, per-color banked delta and swing + shadow divergence (what the opponent bot would play on the variant's endgame turns, computed on non-mutated states). Runs sequentially in one process (no parallel-search noise), niced.
- `probeE-mined.js` (NEW): replays `mined-data/game-*.json` through engine.js; endgame-window (line ≥ 108) behavior of winners vs losers. `probeE-reach.js` (NEW): enumerates the losing side's full candidate set in endgame states and counts area-killing vs any-edge break candidates.
- Locked files untouched: engine.js, bot.js, bot-tourney.js, connector.js, server.js, others' files.

## Match results (6 games alternating colors, budget 2000ms)
### botE-endgame vs bot-tourney.js (A = botE-endgame)
| game | A color | B blue | R red | margin(A) | endgame start (move, m) | endgame Δ B/R | swing(A) | shadow divergences |
|---|---|---|---|---|---|---|---|---|
| 1 | blue | 2093.2 | 3670.0 | −1576.9 | 109, −1482.1 | +333.0 / +427.7 | −94.7 | 1/3 (first at 111) |
| 2 | red | 1882.3 | 3299.5 | +1417.3 | 109, +1302.7 | +246.8 / +361.4 | +114.5 | 2/3 (first at 109) |
| 3 | blue | 2147.3 | 3806.3 | −1659.0 | 109, −1468.9 | +325.3 / +515.4 | −190.1 | 3/3 (first at 111) |
| 4 | red | 1918.9 | 3378.5 | +1459.6 | 109, +1309.8 | +287.0 / +436.7 | +149.7 | 2/3 (first at 109) |
| 5 | blue | 1930.8 | 4013.2 | −2082.5 | 109, −1765.8 | +313.2 / +629.9 | −316.7 | 2/5 (first at 111) |
| 6 | red | 1932.3 | 3309.6 | +1377.3 | 109, +1300.8 | +291.4 / +367.9 | +76.5 | 3/3 (first at 109) |
| **TOTAL** | | | | **3W-3L-0D, avg −177.4** | | | mixed signs | 13/20 endgame turns |

### Control: bot-tourney vs bot-tourney (same harness, same budget)
| game | A color | margin(A) | endgame start m | endgame Δ B/R | swing(A) | shadow divergences |
|---|---|---|---|---|---|---|
| 1 | blue | −1003.6 | 109 | +321.8 / +318.0 | +3.9 | 0/3 |
| 2 | red | +1524.6 | 109 | +174.2 / +377.3 | +203.1 | 1/3 (first at 109) |
| 3 | blue | −1333.3 | 109 | +286.1 / +384.8 | −98.8 | 0/3 |
| 4 | red | +1440.1 | 109 | +317.1 / +408.0 | +90.9 | 0/3 |
| 5 | blue | −1323.3 | 109 | +311.8 / +490.0 | −178.2 | 1/3 (first at 111) |
| 6 | red | +1435.6 | 109 | +314.3 / +436.9 | +122.6 | 0/3 |
| **TOTAL** | | **3W-3L-0D, avg +123.3** (blue −1220.1, red +1466.8) | | | mixed signs | 2/18 endgame turns |

### Interpretation
- W-L and margins: botE (3W-3L, −177.4; blue −1772.8 / red +1418.0) vs control (3W-3L, +123.3; blue −1220.1 / red +1466.8). The ~300 avg-margin difference is far inside the ±900–2700 per-game variance these 2000ms self-play matches show (cf. findings-ideas.md). The color edge (Red ~2x structural scoring edge + 2-action reply) decides every game in both runs.
- Endgame-window swings: mixed signs in BOTH runs; the blue-game negative swings in the botE run (−94.7/−190.1/−316.7) also appear in the control (−98.8/−178.2, and red games swing +3.9 to +203.1). Structural (Red's 2-action reply + last-move advantage compound in the endgame), not botE's doing.
- Divergence: botE's endgame mode genuinely fires — 13/20 endgame turns diverged from what bot-tourney would have played, vs 2/18 in the control (pure clock-timing noise floor). The mode changes moves; the changed moves are result-neutral.

## Byte-identity verification (verifyE.js, full 30s budget)
- Non-endgame states (line left > 12): **10/10 IDENTICAL** to bot-tourney.js (moves 0, 3, 7, 21, 40, 60, 80, 90, 100, 104) — 0 violations. PASS.
- Endgame states: 1/4 diverged (move 110, 10 left: tourney `[4,4>5,1]` vs botE `[0,0>1,3]`); 3/4 identical because the evals happen to agree on those positions.

## Why the endgame mode is neutral (mechanism — engine-verified)
1. **Denial is out of reach in builder endgames.** `probeE-reach.js` (12 endgame states from tourney-vs-tourney play): the losing side has ~277 candidates, ~15 that break *some* enemy edge, but **0 that reduce enemy area** — the enemy loop is simply not reachable (placeRadius 3 + frontier geometry). With no denial move available, denial weight 0.6 ≡ 0.15 ≡ 0: the bot builds, exactly like baseline. Denial moves exist only when both sides fight on the same frontier (vs neck/aggro) — and those games end with tiny areas (final 5–30), so the last-12 delta is small either way.
2. **Real games rarely reach the window.** 8/114 mined games reach line 108 (67 resigned, 40 abandoned). The last-12 banked swing avg is +18.9 — real endgames barely move the needle. Mid-game compounding decides ~93% of games. A line-108 endgame mode optimizes the smallest phase of the game.
3. **The eval was already banked-score-based.** `scores` bank at every turn end (both colors' current areas — engine-verified), so the eval's `ownGain − oppGain×w` on turn-ending candidates is already a bank-and-deny formula; remaining-turns multiplies both own and opp terms equally, so relative comparisons are horizon-invariant. The endgame weights mainly change candidate *ranking* (pruning order), and pruning changes were result-neutral here.
4. **What would help instead** (unchanged from Track B, now with endgame evidence): the mid-game denial underpricing (0.15 opp term, 1.5 break weight) applied from *earlier* — i.e., break-awareness/retaliation from the mid-game, not a line-108 switch. Real winners deny when denial is reachable (2–4 area-killing breaks in the last 12 of the 8 games that got there) — but they spent the mid-game making denial reachable.

## Job 3 — double-connect/neck probe (light)
- Design: `botE-neck.js` = botE-endgame.js + shared-endpoint reinforcement bonus (+2.0 per (c1,c2) pair whose two actions share a node — chained or adjacent-reinforcing) in the 2-action eval, always on. Hypothesis: reinforced pairs survive break-heavy opponents better.
- Mechanical caveat verified in engine.js BEFORE running: a cut removes ONE enemy edge and zeroes the loop it belongs to — redundancy inside the same loop does not protect banking (one cut zeroes the whole loop regardless of how reinforced it looks); only out-of-reach distance protects. Petals (separate small loops) were already rejected by Track B (sprawler lost).
- Results (2 games each vs `chall-aggro`, budget 2000ms; A = variant; full logs `matchE-log-eg-vs-aggro.json` / `matchE-log-neck-vs-aggro.json`):

  | variant | A color | margin(A) | endgame start m | endgame Δ B/R | swing(A) | shadow divergences |
  |---|---|---|---|---|---|---|
  | botE-endgame | blue | −204.6 | 109, −157.7 | +43.3 / +90.2 | −47.0 | 3/4 (first at 111) |
  | botE-endgame | red | −393.5 | 109, −295.9 | +171.0 / +73.3 | −97.7 | 5/5 (first at 109) |
  | **botE-endgame total** | | **0W-2L, avg −299.1** | | | | 8/9 |
  | botE-neck | blue | −122.1 | 109, −92.8 | +93.6 / +122.8 | −29.3 | 3/3 (first at 111) |
  | botE-neck | red | −161.1 | 109, −86.4 | +131.8 / +57.0 | −74.8 | 4/4 (first at 109) |
  | **botE-neck total** | | **0W-2L, avg −141.6** | | | | 7/7 |

- Reading: the reinforcement bonus survived aggro ~2x better on margin (−141.6 vs −299.1), better in BOTH colors — the only variant signal all session. But: 0W-2L (aggro beats the entire bot family, cf. Track B 2/2), 2-game sample, 2000ms timing noise (shadow divergences 7–9 per 9 show near-total endgame divergence — vs aggro, denial/reach is contested so every endgame turn differs). In these low-area games the endgame deltas are small (43–172) — the margin gap is built mid-game.
- Verdict: **NEUTRAL-TO-POSITIVE, do not merge on this sample.** Follow-up worth time post-tournament: reinforcement-weight sweep (2.0 vs 4 vs 8), always-on vs endgame-only, 6+ games, vs both aggro AND neck-hunter. If it holds up as Blue it addresses the same underpriced-denial surface as Track B's patch, from the build side rather than the break side.

## Recommendation
- **Do NOT merge** `botE-endgame.js` — fails the brief's bar (needs 4+ wins or consistent positive margin both colors; got 3W-3L color-decided with margins inside control variance). Keep `bot-tourney.js` as-is for the tournament.
- The endgame mode is safe (byte-identical outside the window, never times out more, CPU-light) but result-neutral: worth keeping on a branch only if a future opponent archetype makes endgame denial reachable (frontier brawls) — otherwise it optimizes a phase that decides ~7% of real games.
- Bigger lever for a post-tournament patch (Track B finding, reinforced here): mid-game denial/retaliation weights, applied from earlier in the game.
