# Bot Health Monitor — findings-health.md

## TOP-LINE SUMMARY (complete: 5/5 games + all targeted probes)

Tournament bot (`bot-tourney.js`, via `T.bestTurn`) is flag-safe on 1min+15s and never throws: 5 full self-play
games completed with ZERO illegal moves, ZERO timeouts, ZERO throws (307 turns total), worst single turn 9864ms
< 15s increment, ~165–187s/player spend vs ~585s allowance (~3x headroom). Wiped-player / near-full-board /
1ms-budget edge states all degrade gracefully to a legal move or clean null.

## 1. Profile: full self-play game (game 0, `bestTurn(s)` adaptive default)

- Turns: 69 total (~35/player, matches expected ~30 turns/player), finalMove=120/120, scores B:2059.5 R:3786.2
- Total game time (both players): 395132ms (~6.6 min) → ~200s per player
- Allowance per player: 60s + 35×15s ≈ 585s → headroom ≈ 3x. FLAG-SAFE.
- Average turn: 5726.6ms. Worst turn: 9848ms @move45 (mid-game).
- By phase: early (m≤6): n=3 avg=513.7ms max=799ms (budget 1500) |
  mid (7–80): n=37 avg=7349.2ms max=9848ms (budget 8000) |
  late (81+): n=29 avg=4195.6ms max=8233ms (budget 4000)
- Overshoot vs budget is real but bounded: mid worst +1848ms, late worst +4233ms over nominal budget.
  Sources: (a) upfront `singleActionCandidates`+rank runs BEFORE any clock check (unbounded),
  (b) one candidate evaluation per loop after the last check. Late-board single evals cost more (Dr recompute over more segments).
- Key bound: worst turn across all 5 games (9864ms) < 15s increment, so even consecutive worst-cases cannot drain the clock.

## 2. Robustness (5/5 games, `health-profile.js default 5`; every turn replayed via `E.applyMove`)

- Game 0: 61 turns, total 373740ms, avg 6126.9ms, worst 9226ms@move77, illegal=0 timeouts=0 throws=0
- Game 1: 61 turns, total 373574ms, avg 6124.2ms, worst 9864ms@move45, illegal=0 timeouts=0 throws=0
- Game 2: 61 turns, total 334278ms, avg 5480.0ms, worst 9238ms@move51, illegal=0 timeouts=0 throws=0
- Game 3: 61 turns, total 326144ms, avg 5346.6ms, worst 8975ms@move71, illegal=0 timeouts=0 throws=0
- Game 4: 63 turns, total 328534ms, avg 5214.8ms, worst 9169ms@move45, illegal=0 timeouts=0 throws=0
- AGG: 307 turns, totIllegal=0, totTimeouts=0, totThrows=0, worstSingleTurn=9864ms.
  Per-player spend ≈ 163–187s vs ~585s allowance (60s + ~31 turns×15s). All 5 games reached move 120/120.
- (Inter-game score variance under self-play is expected: `Date.now()` budget checks make the search timing-nondeterministic.)
- Phase bands stable across games: mid avg 6.1–7.5s (budget 8s), late avg 4.5–4.8s (budget 4s), early avg 0.2–0.8s (budget 1.5s).
- Near-full board (fast-forwarded to move 115, 25B+31R nodes): full-budget turn took 6203ms, returned 2 moves, both legal. OK.
- 1ms budget on move-115 state: took 2588ms, returned 1 legal move (fallback chain works; floor is upfront candidate-gen cost, ~2.5s late-game regardless of budget).
- Zero budget on opening state: 80ms, 1 legal move. 50ms budget: 65ms, 1 legal move. No nulls when moves exist.
- Wiped-player states (nodes+segments emptied): side-to-move wiped → clean `null`, NO throw; opponent wiped → plays normally (5406ms, 1 move). OK.
- Non-object junk budget (`'x'`) falls back to 8000 default, no throw (code read + `budget0` probe).

## 3. adaptiveBudget() — verified by unit probe (engine untouched, read-only calls)

- Phase defaults: m≤6 → 1500; 7–80 → 8000; 81+ → 4000. Confirmed at m = 0,1,6,7,50,80,81,100,119.
- Low-clock throttling: clockMs=19999 → cap 1500; clockMs=20000..59999 → cap 4000; clockMs=60000+/missing/invalid → phase default. Boundary values 19999/20000/59999/60000 all correct; `{}` and `{clockMs:'x'}` safely ignored.
- No path exceeds the intended phase budget except the single candidate-evaluation overshoot documented above (unavoidable by design — granularity is one eval; worst measured +4233ms late-game).
- Note: `play.js` passes hardcoded `{clockMs:60000}` → mid-game capped at 4000ms, i.e. live play uses LESS than the 8000 default profiled here. Our numbers are the conservative case.

## 4. Pre-round checklist (30s)

1. `node -e "require('./bot-tourney.js').bestTurn(require('./engine.js').createInitialState(),0)"` returns a move in <2s (bot loads, fallback chain alive).
2. Confirm harness passes clock object (`{clockMs}` = real remaining ms) — hardcoded 60000 hides low-clock throttling.
3. `git status` clean on `engine.js`, `bot.js`, `bot-tourney.js` (locked files unmodified).
4. Clock math: expect ≤35 turns/player; worst turn ~10s < 15s increment — no special pacing needed.
5. If any probe returns null-with-moves-available or throws, do NOT play the round — redeploy last-known-good `bot-tourney.js`.
