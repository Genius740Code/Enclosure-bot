# v9 census-verify — analysis lane (2026-10-03)

**Lane:** v9-ANGEL (cheap analysis/census, read-only md + probes). **Base:** master
`2a41473` (lane-v9-angel HEAD). **Deliverables:** the four unblocking checks for
the next v9 doses — ANGEL-CIRCLE census verification, MIRROR-ORACLE kill-0,
P1 pair-regret census, CUT-YIELD discrepancy confirmation.

**Not relitigated (per brief):** K-CLASS (pooled 19.5%) and PROT-BLOCK (zero
delta) stay KILLed. No engine doses here: probes + md only, no `src/` changes.

**Laws honoured:** every replay below is deterministic (fixed node budgets,
never `analyze_timed`); read-only probes under `retaliator/examples/`; logs
force-added to `retaliator/gate-logs/` (gitignored dir, cut-D1-gates precedent).

---

## 1. ANGEL-CIRCLE census — first real AngelWASM-vs-us data (n=4)

The 2a41473 census ran with **n=0** AngelWASM games on disk (site API 404s
unauthenticated). Since then the **v8 validation games landed on
`lane-v8-autopsy` (`12073d2`)**: 12 rated games, of which **4 vs AngelBot-WASM
(`a2d70517`)** — the first AngelWASM-vs-Riposte games the repo has ever held.
Copied to `research/games-ref/` + `research/games/` (site format, ids decoded
with `Move::from_index`).

**Replay validation (probe_angel + angel_profile):** final scores
1319-2085 / 479-1810 / 2169-3318 / 212-1077 reproduce the scoreboard's
validation row exactly; the territory-face port matches the engine's `area()`
**8/8** (2 players x 4 games). us = v8 Riposte, 2 Blue + 2 Red chairs.

### 1.1 AngelWASM profile on real games vs us (n=4, area-rising definition)

| metric | AngelWASM | us (v8 Riposte) |
|---|---|---|
| closes (area rises)/game | **22.75** (22/24/24/21) | 28.0 (21/30/33/28) |
| area produced/game | **149.5** | 87.6 |
| area per close | **6.57** | 3.13 |
| first close action | 5/7/9/19 (mean 10) | 5/8/9/8 |
| cuts/game | **42.5** (170/4) | 33.25 (133/4) |
| close levels later undercut (pop-rate upper bound) | 0.77 (70/91) | 0.79 (89/112) |

**AngelWASM is a banker-cutter hybrid, not a pure banker:** 6.57 area per
close (2.1x ours) AND 42.5 cuts/game — which puts it **inside the farm regime
(>= 40 cuts/game)** from the 2a41473 census. It banks big AND cuts hard. The
2a41473 §3 profile (14.5 closes/game at 4.22, from n=2 GB games) under-counted
both axes; the real games vs us show ~23 closes at ~6.6.

### 1.2 The literal "circling us" question (cages)

**5 cages in 2 of 4 games** (1.25/game; **10.9% of gaining closes** — the same
cage rate the corpus showed):

| file | cage circles | shape | our nodes inside | phase |
|---|---|---|---|---|
| `v8_0347cde5` (us=Red) | 3 @97/@108/@109 (4.5/22.6/5.6) | mid loops | 2 each | late |
| `v8_abe514b7` (us=Blue) | 2 @66/@71 (17.0/11.5) | mid loops | 3 each | mid |

- Shape: **mid loops (8-30 area) around 2-4 trapped nodes**, embedded in
  thickets — **avg 18.0 deg>=2 nodes** at circle time (corpus 15.5). No big
  (>=30) cages, no early (0-5) cages.
- **What breaks it:** circle edges cuttable at the first our-turn **5/5**
  (avg 3.8 cuttable edges, max destroyed 5.6) — we cut a circle edge in **5/5**
  cages. BUT the bank fell back (opponent area below the pre-circle level) in
  **4/5** — vs the corpus's **0/4**. *The 2a41473 corpus conclusion "post-close
  cutting never un-banks the score" does NOT hold vs AngelWASM.* Caveat: the
  metric does not attribute the fall to the circle cut itself and n=5 is tiny —
  directional, not a law.
- Outcome: circled games 0/2, uncircled 0/2 — no separation at n=4 (all 4
  validation games were lost).

### 1.3 Kill-0 status — STILL FAILS, no dose

| Gate A requirement | status |
|---|---|
| >= 8 AngelWASM-vs-us games, >= 4 per chair, moves on disk | **4 / 8** (2 + 2 chairs) — FAIL |
| >= 8 AngelWASM games (any opponent) at n>=8 both colors | 6 / 8 (4 v8 + 2 GB) — FAIL |
| replay bit-exact on final scores | instrument ready, verified on the 4 |
| a valid local AngelWASM proxy | none (2a41473 §3 stands — all 7 archetypes wrong on 2 of 3 axes) |

**Gate A fails → no dose. Lane 7 stays a census item.** What unblocks it: 4
more AngelWASM-vs-us games (>= 2 per chair) — the next v8/v9 rated validation
set, or the coordinator pulling them. The instrument needs no changes.

**Gate B note (design flaw found):** the kill-0's Gate B (RECLOSE_SHARE <= 0.50
in the farm stratum) assumed the Angel target populates the farm regime. It
does — 42.5 cuts/game — so once Gate A closes, the farm-stratum measure is
runnable on AngelWASM games directly. But the corpus's farm stratum (aggro/neck)
measures a *popper*, and AngelWASM is a *banker-cutter*: the two strata may
need separate thresholds. Flag for whoever writes the dose.

---

## 2. MIRROR-ORACLE kill-0 — **KILL**

Spec (v9-ideas #4 / roadmap lane 5): offline beam-64 Blue vs the recorded Red
line, 9 openers: Blue @30 area >= 35.0 in >= 8/9; else kill. Eligible
(PROT-BLOCK dead, zero delta). Probes: `oracle_kill0b.rs` (synthetic 9 openers)
+ `oracle_kill0c.rs` (recorded mirrors), both with `ranked()` + helpers frozen
verbatim from `search.rs` @ 2a41473.

**Harnesses (both deterministic; recorded runs used 0 fallbacks):**
- Synthetic: 9 openers = Red = `mesh_prefix` (the recorded mirror mesh, Red's
  identical first 16 own moves in 5/5 mirrors) while legal + the prot_kill0
  continuation, seed `pref = g % min(9)` — **9 genuinely distinct Red lines
  through move 30** (prot_kill0's `min(3)` yielded only 3 distinct games x 3 —
  its "9/9" bar was effectively 3/3).
- Recorded: 5 openers = the 5 recorded v7-vs-v6 mirrors; Red = its recorded
  moves (15/60 used by move 30, 0 fallbacks). **Red @30 = 37.2 — the premise's
  "Red 37.3" reproduces exactly**, which validates the harness.

| Blue mode | synthetic 9 openers | recorded 5 mirrors |
|---|---|---|
| BASELINE: w8 search, per-action re-plan, no prefixes | **40.2** — 9/9 PASS | — |
| BLUE-MESH: w8 + mesh prefix = **current behaviour** | **38.3** — 9/9 PASS | **38.3** — 5/5 PASS |
| BLUE-OPENER: w8 + D10-F7 only | 40.5 — 9/9 PASS | — |
| BEAM64 (w64), per-action re-plan | 37.4 — 9/9 PASS | 37.4 — 5/5 PASS |
| BEAM64-SOLVED: w64 PV played out, **no re-plan** | **33.7 — 0/9 FAIL** | **0/5 FAIL** |

### Verdict: **KILL** — the dose has no case on any reading

1. **The dose as specified (an offline-solved TABLE played without
   re-planning) fails the bar:** the beam-64 solved line reaches **33.7 (0/9
   synthetic) and fails 0/5 recorded** — a fixed table cannot reach Blue @30
   >= 35.0.
2. **The bar is only met by searching every action** (re-plan 37.4) — but the
   **current Blue already reaches 38.3** (9/9 synthetic + 5/5 recorded): the
   control clears the bar, so the beam-64 upgrade adds nothing measurable.
3. **The premise's "Blue @30 = 24-29" is stale.** It was the recorded
   v6/v7-era Blue's area (JS replay of the mirror games). The current search
   reaches 38.3 on the same Red line — the v8-era stack (mesh + D1 tie-break +
   full-horizon valuation) already closed the 6-11 area gap the +20 estimate
   was anchored on. With Blue LEADING at move 30 (38.3 vs 37.2) and v8 still
   0-12 on the site, a mirror-scope @30 gain is not worth Elo.

### Actionable residue (mechanism finding, not a dose)

The solved line's failure has a precise cause: **the reply model ranks the
PV's assumed move 2 WITHOUT the first-action penalties** (`ranked(..., false,
...)` gates REMOTE/FRESH/DENSE/IDLE/REBUILD/PATIENCE/CONTACT/DEADWOOD and
loop_bonus behind `first`), while a fresh re-plan applies them — 33.7 vs 37.4
on the same width-64 search = a **3.7-area gap at @30 from the pair's second
action alone**. The current bot is immune (it re-plans every action), so this
only matters for future TABLE-based doses: **any extracted table must re-rank
its PV move 2 with `first=TRUE` before extraction.** Applies to SCRIPT-LOCK
(reply table — opponent moves, unaffected) and any future Blue-table dose.

---

## 3. P1 pair-regret census — **KILL**

Spec (idea-backlog P1 step-0): on our double turns, does the jointly best pair
beat greedy-then-replan, how often? Kill-0: regret rare/small. D1 only if
regret is big AND frequent. Probe: `p1_pair2.rs` (46-game corpus, our
double-turn starts = `actions_left_in_turn() == 2`, both chairs, WIDTH=8 fixed
budgets, joint = the re-planned mv2 after each of the top-8 root candidates,
pair value = the search's own selection machinery — horizon extension + doom
discount, frozen verbatim).

**Corpus: 1357 our double turns** (46 games, full coverage — 667 us=Blue +
690 us=Red; the local-seq decoder needed the filename fallback for the
R-chall files, which carry no `color` field).

| measure | value |
|---|---|
| re-plan != PV-assumed mv2 | 108 / 1357 = **8.0%** (tracks the PV 92%) |
| regret joint-greedy > 0.5 | 87 / 1357 = **6.4%** |
| regret joint-greedy > 5.0 | 58 / 1357 = **4.3%** |
| regret size | mean 1.016, **p90 0.000**, max 113.25 (selection units) |
| us=Blue (667 turns) | mismatch 4.5%, regret > 0.5 4.5%, > 5: 21, mean 0.519, max 38.0 |
| us=Red (690 turns) | mismatch 11.3%, regret > 0.5 8.3%, > 5: 37, mean 1.497, max 113.25 |

### Verdict: **KILL**

- **The regret is RARE:** 4.3% of double turns exceed 5 selection units; **94%
  of double turns have exactly ZERO regret** (p90 = 0.000) — the greedy pair IS
  the joint pair. The kill-0 ("regret rare/small") fires.
- **The re-plan already handles the second action correctly:** it tracks the
  PV 92% of the time, and where it diverges the fresh re-plan applies the full
  first-action penalty stack — which is why the joint planner finds so little
  (the only mispricing left is the root's mv1 choice, priced on a
  reply-model assumption that is itself rare, 8%).
- **D1 (joint pair search) not justified:** it would fire on ~4% of turns with
  a mean gain of ~1 unit — a 400^2-pair search under the 2s soft budget for
  that is a bad trade. The idea-backlog's own cost threat stands.
- Chair note: Red's regret is ~3x Blue's (8.3% vs 4.5%; mean 1.5 vs 0.5; max
  113 vs 38) — directional at census n, worth remembering if a Blue-chair dose
  ever needs a tie-breaker.
- Note: this census measures the WIDTH=8 search on corpus positions (real
  opposition, old-bot recorded moves). The mirror-scope quantification of the
  same mechanism (solved line vs re-plan = 3.7 area at @30) is in section 2.

---

## 4. CUT-YIELD discrepancy — **CONFIRMED: both numbers real, KILL stands**

The two lanes reported contradictory kill-0 numbers. Both reproduce exactly:

- **61.3% (lane-v9-cut, `cut_census.rs`):** my rerun = Blue 45/29 (64.4%), Red
  17/9 (52.9%), **FLIP 38/62 = 61.3% over 2760 roots — identical to the
  committed log.**
- **0.9% (lane-v9-cutyield, `probe_cutyield.rs`):** my rerun on the full
  corpus reproduces its census (cuts us 566 / opp 342, zero-yield us 214 /
  opp 42) and its flip census; the neck subset rerun reproduces its documented
  zy counts (us 111/14) and both documented flips (the `zy@16` flip: dose
  -247 vs ctrl +84).

**The discrepancy is denominators + flip definitions, not data:**

| | lane-v9-cut (61.3%) | lane-v9-cutyield (0.9%) |
|---|---|---|
| population | 62 our-to-move ROOTS where the current search's pick is a zero-yield cut | 214 recorded zero-yield CUT actions (what the old bot played) |
| "flip" means | the penalty changes the PICK (ranking-level) | the outcome class changes (CONTROL vs DOSE-CF continuation) |
| reading | "when the current search wants a zy cut, the penalty reroutes it 61% of the time" | "skipping zy cuts at the recorded decision points flips the game outcome 0.9% of the time" |

**The roadmap's kill-0 ("flip census >= 10% of 113 zero-yield cuts; 0 flips =
Lane-E disease, kill") intends the recorded-cut denominator** (the 113 was the
12-game estimate; the 46-game corpus holds 214). Under it:

- The pick-level reading passes (the dose is not Lane-E-dead — it changes
  picks) — which is what lane-v9-cut reported as GO, **reinterpreting the
  denominator to 62 picks**.
- The **outcome-level reading fails 10x (0.9% << 10%)** — and it is the
  decision-relevant one, for four reinforcing reasons:
  1. zero-yield cuts correlate with WINS 49.7% vs losses 17.6% (dominance
     artifact — the taxed population lives in dominating positions);
  2. the 2 real flips are 1-for-1 in DIRECTION (one improving, one
     REGRESSING — the zy@16 flip lost 247 vs the control's +84, proving some
     "zero-yield" cuts carry invisible denial value the area-only axes cannot
     see);
  3. where the dose flips a pick inside a won position, the outcome does not
     move (the neck log's dose Win/ctrl Win rows);
  4. the current engine already suppresses the symptom at 88.5% of the
     recorded decision points (24 dose targets remain).

**Consequence for the landed code:** the CUT dose on `lane-v9-cut` is
`ZERO_CUT_PENALTY_ON: bool = false` — a **no-op toggle**; its gates ran with
the toggle OFF (byte-identity), so they prove the toggle does not leak, not
that the dose works. **Leave it OFF.** The KILL stands.

**Actionable residue (narrow, needs its own kill-0, not recommended on this
evidence alone):** on the neck subset alone (losses only) the outcome-flip rate
is 14.3% (2/14) — a zero-yield penalty gated on enemy aggression / loss-leaning
positions has a real signal on 14 zy cuts. n is tiny; a gated variant would
need its own kill-0 at n>=8 both colors.

---

## 5. Dose specs for the coordinator (unblocking the next v9 doses)

| item | verdict | dose spec |
|---|---|---|
| K-CLASS | KILLed (do not relitigate) | — |
| PROT-BLOCK | KILLed (do not relitigate) | — |
| CUT-YIELD | KILL confirmed (both numbers real) | none; toggle stays OFF; gated zy variant needs its own kill-0 |
| MIRROR-ORACLE | **KILL** (this report) | none — the offline table fails 0/9 + 0/5; the control already clears the bar; premise stale |
| P1 pair planning | **KILL** (this report) | none — regret rare (4.3% > 5 units, 94% zero); D1 not justified |
| ANGEL-CIRCLE | census only, Gate A 4/8 | none — needs 4 more AngelWASM-vs-us games (>= 2/chair); instrument ready, no changes |
| REINFORCE-LINES | in progress on master (Lane P H2 `7663361`: patient big-build eval + probe) | its kill-0 (unbreakable-share vs win census) — NOT covered here |
| SCRIPT-LOCK | unstarted (roadmap lane 6) | needs the Phase-4 corpus (>= 8 games/bot) — NOT covered here |

**Where the next Elo actually is, from this census:** nothing in the four
checked items unlocks a dose. The surviving facts: (a) AngelWASM is a
banker-cutter hybrid at 42.5 cuts/game — the farm regime is where the site
games live, and the corpus's RECLOSE_SHARE guardrails (2a41473 §6) are the only
measured lever, blocked on Gate A data (4 more games); (b) vs AngelWASM,
post-close cutting un-banked 4/5 circles (corpus 0/4) — if that holds at n>=8,
a circle-breaker answer becomes measurable where the corpus said only
prevention could matter; (c) the +40/+30/+25/+20/+10 queue above the two
KILLed items is now fully dead — the roadmap needs new candidates from the v8
validation autopsy (rival files), not from the mirror corpus.

## Reproduce

```bash
cd retaliator
cargo run --release --example probe_angel     -- ../research/games-ref/v8_0347cde5.json \
  ../research/games-ref/v8_140faf2d.json ../research/games-ref/v8_82aa8c12.json \
  ../research/games-ref/v8_abe514b7.json                                              # cages + what breaks it
cargo run --release --example angel_profile   -- ../research/games-ref/v8_*.json       # rising-area profile (4 Angel games)
cargo run --release --example oracle_kill0b    # synthetic 9 openers, 5 Blue modes
ORACLE_RED_DIR=../research/games-ref cargo run --release --example oracle_kill0c      # recorded 5 mirrors
cargo run --release --example p1_pair2        -- /home/genius74o/game                  # 46-game corpus
cargo run --release --example cut_census      -- /home/genius74o/game                  # 61.3% rerun
cargo run --release --example probe_cutyield  -- /home/genius74o/game                  # 0.9% rerun
```

Logs: `retaliator/gate-logs/{angel-v8-census,angel-v8-profile,oracle-kill0b,
oracle-kill0c,p1-pair2,cut-census-rerun,cutyield-neck,cutyield-full}.log`.

Corpus inputs: the 4 AngelWASM v8 games + the 5 v7-vs-v6 mirrors (and the other
8 v8 validation games) are copied to `research/games-ref/` — canonical home:
`lane-v8-autopsy` `12073d2`. The 46 local sparring games stay gitignored at the
main checkout.

— Lane ANALYSIS (v9 census), 2026-10-03. Probes read-only; no `src/` changes;
no engine doses.
