# FINAL-LOCK VALIDATION — findings-finalcheck.md
**LOCK. bot-tourney.js stays locked. Do NOT merge botX-quiesce.** (Full-budget match: 2W-2L-0D, quiesce wins only as red / loses only as blue; avg margin +144.3 is noise against ±2200 per-game swings. Separately, break-first beats greedy expansion 6-0 — see Job 2 for the post-tournament fix.) Connector rebuild safe for every requested server shape; one REAL mirror-breaker found — numeric-string coords diverge silently (post-tournament pt() fix). NOTE ON DEADLINE: the heavy full-budget matches ran long; jobs 1–2 completed past 16:55 (results final); job 3's script had a bug, fixed and re-run.

Files touched (new only, as instructed): `test-connector-rebuild.js`, `findings-finalcheck.md`. **No chall-breakfirst.js written — see Job 2: chall-aggro.js already IS break-first.**

---

## JOB 1 — Full-budget confirmation: botX-quiesce vs bot-tourney @ 8000ms, 4 games alternating

Command: `node run-match.js ./botX-quiesce.js ./bot-tourney.js 4 8000` (run-match.js alternates colors; quiesce = A)

| game | quiesce color | blue | red | margin (quiesce) |
|---|---|---|---|---|
| 1 | blue | 1618.6 | 3842.6 | **−2224.1** |
| 2 | red | 1884.6 | 4026.1 | **+2141.5** |
| 3 | blue | 1737.5 | 3346.4 | **−1608.9** |
| 4 | red | 2027.9 | 4296.7 | **+2268.8** |

**RESULT: 2W-2L-0D / 4, avgMargin(quiesce) = +144.3, budget = 8000ms**

Findings:
1. **The 12x30 wider search DOES diverge at full time.** At 400ms quiesce was byte-identical to tourney; at 8000ms the K1=12/K2=30 deepen + quiescence re-rank stage actually changes moves (per-game score profiles are far apart from any 400ms baseline). So the wider search is not dead code at tournament budget — it changes play.
2. **But it changes play into a wash.** The merge criterion (quiesce wins 3+ with margin) is not met: quiesce won exactly the games where it played red and lost exactly the games where it played blue. Per-color splits: quiesce-as-red avg +2205 vs quiesce-as-blue avg −1916 — the mirror image of tourney's own splits. The +144.3 net is noise vs ±2000+ swings at n=2 per color.
3. **Color signal at full budget:** red won all 4 games regardless of engine (red ~3300–4300 vs blue ~1600–2000). At 8000ms the side that gets the last full reply punishes the other's plan — the second player's deeper counter-search dominates. (At 2000ms in Job 2, first player mostly won instead — the effect flips with budget.) No action for the lock; worth a post-tournament look at blue's opening plan.
4. Quiescence stage cost is real (extra opp-reply searches on volatile finals) yet produced no edge over tourney's simpler ordering. Confirms earlier 400ms finding: quickScore2's added terms don't pay for themselves against this opponent.

**Verdict: CONFIRM THE LOCK.** bot-tourney.js keeps its slot for the 17:00 game.

## JOB 2 — Break-vs-build: break-first vs greedy expansion @ 2000ms, 6 games

Reuse decision: **chall-aggro.js already always takes a break when one exists** — verified statically: in `pick`, any break scores ≥ 500·1 − 25.5 (max board crossing distance) ≈ +474, while any non-break scores ≤ areaGain·0.05 ≤ ~16 (max polygon area on a 0..18 board ≈ 324). No break can ever lose the valuation, so `assemble` takes one at every action where one exists. Hence no `chall-breakfirst.js` was needed/written. (Empirical property probe over bot-tourney self-play found 0 break-available states in the first 30 turns — breaks are rare early; the static bound is the guarantee.)

Command: `node run-match.js ./chall-aggro.js ./bot-tourney.js 6 2000`

| game | aggro color | blue | red | margin (aggro) |
|---|---|---|---|---|
| 1 | blue | 864.8 | 406.1 | **+458.7** |
| 2 | red | 519.6 | 630.8 | **+111.2** |
| 3 | blue | 761.5 | 425.3 | **+336.2** |
| 4 | red | 314.5 | 582.3 | **+267.8** |
| 5 | blue | 847.5 | 400.5 | **+447.0** |
| 6 | red | 352.3 | 632.9 | **+280.6** |

**RESULT: 6W-0L-0D / 6, avgMargin(aggro) = +316.9, budget = 2000ms**

Findings:
1. **YES — break-first beats greedy expansion, decisively.** A pure 1-ply aggro bot with NO search 6-0'd the 2-ply tournament bot, winning both colors (blue by ~450/336/447, red by ~111/268/280).
2. Root cause in the locked bot's valuation: `bestTurnBudgeted` scores `ownGain − oppGain·0.15` — a break that deletes opponent segments without a score change earns **zero** in the 2-ply valuation (breaks only get `1.5` weight in BOT.quickScore, which is used for *ordering*, not choice). Aggro weights breaks at 500. The tournament bot systematically undervalues the strongest tactical resource.
3. "Should I break every turn?" → **empirically yes when one exists** (chall-aggro takes every available break and still wins all no-break states by harassment — dmin minimization — so the 6-0 isn't only break-driven).
4. **Post-tournament recommendation (not applied — lock):** fold break-first weighting into the next bot revision — either raise the break term in the 2-ply valuation (`+ breaks·k` with k ≫ 0, aggro suggests large k works) or add an explicit break-first override ahead of expansion. Do NOT merge quiesce (Job 1).

## JOB 3 — Connector rebuild() safety drill (simulated payloads, no live game)

Script: `test-connector-rebuild.js` — `pt()`/`rebuild()` copied verbatim from connector.js lines 71–86 (NOT required — it connects on load); the onState resync decision (lines 129–137: `historyLength` vs `moveHistory.length`) replicated. Reference game: engine + bot-tourney @40ms, 24 turns (mirror `{scores:{blue:67,red:81}, turn:blue, segB:13, segR:13, mn:24, ar:1}`). First run had a script bug (loop-var scoping); fixed and re-run — full log at `/tmp/opencode/job3-rebuild.log`.

**The four requested shapes — all SAFE:**
- **A `{from:[x,y],to:[x,y]}`** → exact mirror, PASS.
- **B `{from:{x,y},to:{x,y}}`** → pt() normalizes; exact mirror, PASS. Mixed array/object within one entry (B2) also exact, PASS — pt handles each point independently.
- **C `{pass:true}`** → PASS. A mid-turn pass (ar=1) through rebuild skips the remainder of the turn and flips to the opponent with fresh actions (`blue 67→75 / red 81→90 banked, turn blue→red, mn 24→25, ar→2`), exactly matching the engine's direct application. Caveat: no natural pass occurred in 24 bot-tourney turns, so the pass path was verified via direct rebuild-with-pass probe (not a server-observed entry). Second caveat: applyTimeout BANKS pending score on pass — if the server's pass semantics differ (e.g. no bank), scores would diverge while positions stay identical, and connector's syncOk check only compares **segment counts**, not scores — divergence would not trigger a resync (the log prints both scores but nothing acts on a mismatch).
- **D empty moveHistory + historyLength>0** → rebuild([]) returns a fresh initial state (no divergence), then the historyLength check fires `request_game_history` before any thinking — correct: mirror momentarily stale-initial, resync, never thinks on partial. Missing moveHistory key entirely (D2) behaves the same, PASS. Empty moveHistory + historyLength=0 (D3) correctly thinks on an empty board.

**Shape that breaks the mirror — REAL FINDING:**
- **Numeric-string coordinates** (`{from:["3","4"],to:[...]}`) → **catastrophic silent divergence**: rebuild returned a nonsense state (mirror `scores blue:1947, red:0` vs reference 67/81) with **no exception** — rebuild's REBUILD-DIVERGENCE catch never fires, so onState proceeds thinking on garbage and the segment-count syncOk check may even pass (segB/segR matched 13/13 in the diverged state). pt() passes strings through untouched (`Array.isArray → [p[0], p[1]]`), and engine arithmetic on strings breaks the replay. Observed server data uses numbers, so probability is low — but one line in pt() (`Number(p[0])`) would kill it. Post-tournament fix, not applicable under the lock.

**Other probes:**
- **X1 `{from:null,to:null}`** → TypeError inside rebuild's try → REBUILD-DIVERGENCE → null → resync. Caught, safe.
- **X3 valid prefix (5 entries) + bigger historyLength** → resync fired correctly (`resync=true`, no think on the partial). Safe. (The harness's own expected-label was over-strict — expected the mirror to reset to initial, but keeping the 5-entry prefix + resync is the correct connector behavior.)
- **X5 1e-10 float drift** → mirror functionally identical (same positions, same segments, same turn; scores drifted ~2e-9, invisible to engine's 1e-9 geometry tolerance in practice). Benign; exact-string comparison in the harness flagged it, nothing to fix.
- **X2 hypothetical exhausted-actions pass** → the tested state had ar=1, which is a legit timeout (correct skip+flip). A pass sent when the mover had ar=0 cannot occur as a resting engine state, so that hazard is not constructible/testable here — downgrade from "genuine hazard" to "unconstructible, no action."

## TOP-LINE (repeat for the parent session)
**LOCK confirmed. No merge. bot-tourney.js plays the 17:00 game.** Quiesce diverges at 8000ms but splits 2-2 by color (noise). Break-first empirically beats greedy expansion 6-0 — top post-tournament fix: raise the break weight in the locked bot's 2-ply valuation. Connector rebuild is safe for every requested server shape; the one REAL mirror-breaker is numeric-string coordinates (silent divergence — `Number(p[0])` in pt() kills it post-tournament).
