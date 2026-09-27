# Next-tournament playbook (different game, 1 week, 1h clock)

## Lessons from this run (keep)

1. **Engine extraction + verification FIRST.** Everything stood on engine.js
   being bit-exact (22 checks, 114/114 archive replays). Day 1 job, always.
2. **One file per track, locked core files.** Zero merge conflicts across
   6+ agents. Keep: `bot-tourney.js`-style lock + green-light merge rule
   (both colors, ≥4 games, beats live bot).
3. **Connector rehearsal with CASUAL games days before**, never the day of.
   Proved: protocol, revision flow, clocks, resync — and found the
   same-account stream-theft bug in time.
4. **Secrets in env only.** One hardcoded credential nearly shipped; scans
   before every commit from now on.
5. **Never open the game page on the bot's account.** Stream goes to the
   browser, bot starves. Watch logged-out or via logs.

## What changes with a 1-hour clock

A 1h clock flips the bottleneck from *search width* to *search depth +
offline prep*. Plan:

- **Days 1–2: rules + baseline.** Extract engine, verify (replay archive if
  one exists), ship a greedy 1-ply baseline + random/junk challengers.
  Regression harness from day 2: every candidate must beat baseline AND all
  prior P0 archetypes (no-regression rule — this run's missing piece).
- **Days 3–4: sparring + archive mining.** The miner was our highest-ROI
  agent (104 games → winners out-build, breaks symmetric, tilt/resign
  noise quantified). With a week: mine 500+, build the opening book from
  real winners (first-N-moves table by win rate), and grow the challenger
  zoo from observed human styles, not invented ones.
- **Days 5–6: depth.** With 1h/move: iterative-deepening alpha-beta or MCTS
  with the greedy as rollout policy, transposition tables (worth it at
  depth — they were dead weight at 2-ply), pondering with a real protocol
  (predict → pre-search → validate), endgame tables if the game is
  solvable late. Time control: spend by position criticality (variance
  between top candidates), not flat budgets.
- **Day 7: FREEZE.** Lock the bot 24h before. Only connector/health checks
  after. This run's rule held (no unproven swaps) — formalize it: code
  freeze + rehearsal games + pre-round checklist (health agent's 5-liner).

## Agent workflow (max 3–4, merge gates)

- Lanes: (1) strength/ideas, (2) red-team archetypes, (3) verification,
  (4) game-data miner (spin up day 2–3). Each owns its files; a daily
  merge review promotes only green-light variants.
- Every variant ships with: hypothesis, matchup table (both colors),
  and the exact sequences of any loss (P0 format). No numbers = no merge.
- Keep one `findings-*.md` per lane + a single `STATE.md` (what's live,
  what's green-lit, what's next) so lanes never block on each other.

## Connector (stable-site assumptions)

- Login with timeout+retry, cookie refresh, Origin/UA headers (all learned
  the hard way). Snapshot mirror from full state (never depend on history
  arrays). Dual turn detection. Watchdogs for silence + stream theft.
- Rehearse: casual games, both colors, plus a chaos test (kill/restart
  mid-game, resubscribe, confirm it recovers and replays correctly).
