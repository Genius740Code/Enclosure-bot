# Track A findings — strengthening bot.js (search / eval / budget)

TOP-LINE SUMMARY (PARTIAL — A1/C1 rejected, B1+E1 running): A1 lookahead: 4/8, -28, DO NOT MERGE. C1 prune-weight tweak: 4/8, behaviorally identical to base in half the pairs, DO NOT MERGE. Key lesson: only final-leaf-value changes have leverage. Now testing B1 (neck penalty in final rerank) and E1 (denial 0.15→0.3 + 1.0/break in final scoring), 8 games each, same seeds.

## Method
- Harness: `/tmp/opencode/run_game.js` (one game: botBlue vs botRed, seeded mulberry32 random legal prefix of P edges from the FULL unpruned move space, then full-strength play to move 120) + `/tmp/opencode/batch.sh` (max 2 parallel; box has 2 cores) + `/tmp/opencode/agg.py` (variant-perspective margins).
- Determinism verified: same seed+prefix twice => identical scores (only wall-clock differs).
- Metric per matchup: wins/12 + avg variant-perspective margin (variantScore - baselineScore), split by color.
- Rule: nothing is called "better" without a multi-game result below.

## Baseline profile (bot.js vs bot.js)
- Full game: 212 s wall, 61 turns, p50 turn ~4.0 s, p90 ~5.3 s, worst 7.3 s (early game sub-second; cost grows with node count as expected).
- Per-turn applyMove calls: ~1.5k early → ~3.6k late; each call embeds a full Dr (territory) recompute. Candidate generation itself is a small fraction; Dr-inside-applyMove dominates.
- Score scale (prefix-10, seed 42, self-play): blue 2640 vs red 2214 (margin ~426). No-prefix deterministic line: blue 2073 vs red 4188 — color balance is position-dependent, hence swapped-colors methodology.
- Calibration: 6 self-play games, seeds 101-106, prefix 10 — DONE.
  seed101 red+17 | seed102 red+18 | seed103 BLUE+655 | seed104 red+185 | seed105 red+108 | seed106 red+1175.
  Red 5/6. Score scale typically ~1600–3000 per side; game margins swing ±1000 by position, so small effects need many games — screening matchups use 4 seeds x 2 colors (8 games) and only large/consistent effects advance.
- TIMING CAVEAT: calibration ran 2-parallel on a 2-core box, so its timings are contention-inflated (worst 22–39 s, p50 6.5–19 s vs solo-probe worst 7.3 s, p50 4 s). Scores are deterministic and unaffected. All timing comparisons must be done solo; a solo worst-case re-probe is queued with the D1 (budget) experiment.

## Idea 1 — 1-ply opponent-reply lookahead (botA1.js)
- Hypothesis: second-action scoring that ignores the opponent's immediate reply overvalues greedy grabs the opponent instantly punishes (recapture/break or bigger adjacent close).
- Experiment: base 2-ply search unchanged; top-5 leaves re-ranked by `val - 0.2 * oppBestReply`, where oppBestReply = max opponent single-action quickScore on a coarse scan (nodeCap 7, coarse offsets). +~0.1–1 s/turn.
- Result: DONE — 4/8 wins, avg variant margin -28 (8 games vs bot.js, seeds 201-204 swapped).
  Per-seed pair nets (A1-blue margin + A1-red margin): 201: -565+561 = -4; 202: -454+194 = -260; 203: +67-27 = +40; 204: +282-285 = -3.
  Margins mirror almost exactly across colors, i.e. A1 plays nearly identically to base — the w=0.2 rerank rarely flips the pick or the flips wash out. No measurable gain.
- Merge recommendation: DO NOT MERGE as-configured. Possible follow-up (only if cheap): flip-rate probe with larger w before any more games.
- Merge recommendation: PENDING.

## Idea 2 — explicit neck detection (botB1.js)
- Hypothesis: enclosures held by a single edge are routinely collapsed by one enemy break; current eval values area with no fragility discount.
- Experiment: top-5 leaves re-ranked by `val - 0.3 * maxDrop`, where maxDrop = worst area loss from removing each of our 6 most-recent edges (geometry-only Dr calls, ~15–30 ms total). Bridge-finding on the coarse graph was rejected as a proxy: loop-closing edges sit in cycles by construction, so graph bridges almost never flag the real threat.
- Result: PENDING (8 games vs bot.js).
- Merge recommendation: PENDING.

## Idea 3 — quickScore weightings (botC1.js: break weight 1.5 -> 4.0)
- Hypothesis: base may undervalue breaks (each break is a free action that also denies opponent area); also to test: area weight and end-of-turn denial weight 0.15.
- Experiment: C1 changes ONLY the prune-rank break weight 1.5 -> 4.0 (same speed as base). More weightings queued based on outcome.
- Result: DONE — 4/8, avg +79.5, but pairs 205/206 mirror EXACTLY (identical scores both colors) and the average rides on seed 208 alone (+523 pair net; 207: +113). Prune weights only reorder the shortlist; the exact 2-ply final scoring usually still picks the same leaf, so C1 is behaviorally near-identical to base. No evidence of gain.
- Merge recommendation: DO NOT MERGE. Lesson: prune-level tweaks lack leverage — future ideas must change the final leaf value or the search itself.
- Merge recommendation: PENDING.

## Idea 4 — searchBudget tuning (botD1.js: TOP_K 10/25 -> 8/16, nodeCaps 16/10/7 -> 12/8/5)
- Hypothesis: late-game full-width search spends most time on moves that don't change the pick; a cheaper budget may be strength-neutral and buy headroom for lookahead ideas.
- Experiment: D1 vs bot.js, 12 games, compare win rate/margin AND worst-turn time.
- Result: PENDING.
- Merge recommendation: PENDING.

## Idea 5 — final-scoring break bonus + stronger denial (botE1.js: denial 0.15 -> 0.3, +1.0/break across the turn)
- Hypothesis (from C1 lesson): only FINAL leaf-value changes have leverage. Breaks are free actions that also deny opponent area; base's 0.15 denial may underweight both.
- Experiment: E1 vs bot.js, QUEUED (8 games, seeds 209-212 swapped — same seeds as B1 for comparability). Zero added search cost.
- Result: PENDING.
- Merge recommendation: PENDING.

## Timing / merge log
- (append negatively- and positively-decided merges here with game counts)
