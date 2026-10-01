# ANGEL-CIRCLE census — v9 roadmap lane 7

**Lane:** v9-ANGEL (cheap analysis/census, read-only md). **Base:** master `34baf47`.
**Date:** 2026-10-01. **Deliverable:** census instrument + census numbers + kill-0.
**Laws honoured:** no Rust touched, no `cargo build`, no gates, no scoreboard edits.
Every number below is replayed through the real `engine/engine.js` and verified
bit-exact against the recorded `scores`/`areas` of each corpus file.

---

## 0. DATA GATE — read this before the tables

**There are zero AngelWASM games on disk. The literal lane-7 question ("how often
does AngelWASM circle *us*") cannot be answered from this repo, and no amount of
analysis changes that.** n = 0, so the v9 law "every claim n>=8 both colors" is
unsatisfiable for the AngelWASM-vs-us census specifically.

What I verified, exhaustively:

| Where | What it holds | AngelWASM content |
|---|---|---|
| `mined-data/` | 114 finished games + `analysis.json` | **0** — all 114 are human-vs-human (`analysis.json` player roster is 200+ distinct humans, e.g. `01smaw` bigbadmouse vs syn) |
| `R-chall-*.moves.json` (34) | Riposte (`bot-tourney.js`) vs 8 sparring archetypes | **0** — our own `sparring/chall-*.js` |
| `chall-*.moves.json` (12) | Riposte (`bot.js`) vs 6 sparring archetypes | **0** — our own |
| `research/rival-analysis.md` §4 | 2 AngelWASM games, **aggregate stats only** | `3d3f8cb8`, `65095072` — **both vs Great Barrier, neither vs us**, and no move files on disk |
| `research/league-scoreboard.md`:183 | v8 validation vs AngelWASM `a2d70517` (12 games) | a match ID only, no moves on disk |

Live-site re-fetch was attempted and failed (the API surface the repo's own
`tools/mined-fetch.js` targets no longer answers unauthenticated):

```
GET https://meaf.us/sst1/api/games/65095072        -> HTTP 404 {"error":"Not found"}
GET https://meaf.us/sst1/api/games?search=&page=1  -> HTTP 404 {"error":"Not found"}
GET https://meaf.us/sst1/games/65095072            -> HTTP 200 (SPA HTML shell, no data)
```

**What this document therefore delivers, in order of value:**

1. **§1 a validated circle-census instrument** — drop AngelWASM `.moves.json` files
   in and it produces the exact tables lane 7 asked for, immediately.
2. **§2–§5 the census on the 46-game corpus that does exist** (n=23 per chair),
   which is where every real mechanism was found.
3. **§6 the AngelWASM target profile** and the proof that **no corpus archetype
   is a valid AngelWASM proxy** — so a dose cannot be calibrated on sparring data.
4. **§7 a kill-0** with a hard data precondition that currently **fails**.

---

## 1. The instrument

A **circle** = one applied move that increases the mover's banked territory
area. Detected engine-natively as `computeTerritories(state.segments).areas[c]`
rising — the same quantity the game scores on, so there is no proxy drift.
(A face-key based detector was tried first and is **unusable**: segment
intersections mutate a face's vertex set, so exact-key tracking reported 8917
"circles" across 5520 actions, which is impossible.)

Per circle the instrument records: action index, owner, banked area, vertex
count, bounding box `w x h` in node units, aspect, centroid, ring distance from
board centre `max(|cx-9|,|cy-9|)`, timing bucket, and whether it **encloses ≥1
enemy node** (a literal cage). Circle death is attributed to the action that
reduced the owner's area, and cross-checked against whether a boundary segment
was removed on that same action.

**Verification (this is what makes the numbers trustworthy):**

```
replay bit-exact on final scores : 46/46 games
replay bit-exact on final areas  : 12/12 games that carry an `areas` field
total replayed actions            : 46 x 120 = 5520, zero illegal moves
```

---

## 2. Census on the available corpus (n=46, 23 per chair)

46 games x 120 actions, 2837 circles total: 1607 ours, 1230 the opponent's.
"us" = Riposte; "them" = the sparring archetype. Chair is *our* chair.

### 2.1 Frequency, loop size, timing — per chair

| Metric (opponent circles) | us = Blue (n=23) | us = Red (n=23) |
|---|---|---|
| circles/game | 26.65 | 26.83 |
| area per close (mean) | 1.66 | 1.48 |
| area per close (median) | 0.75 | 0.58 |
| pop rate | **0.13** | **0.23** |
| cage rate (encloses our node) | 0.02 | 0.02 |
| bbox w x h (node units) | 1.76 x 1.69 | 1.72 x 1.60 |
| aspect (w/h) | 1.17 | 1.27 |
| ring distance from centre | 6.46 | 6.42 |
| timing early(0-5) / mid(6-15) / late(16+) | 0.01 / 0.05 / 0.94 | 0.01 / 0.05 / 0.94 |

### 2.2 Per archetype x chair — the table that explains everything

| archetype | cuts/game | closes/game | area/close | pop rate | first close (act) |
|---|---|---|---|---|---|
| neck | **67.75** | 39.50 | 1.56 | **0.40** | 12 / 11 |
| aggro | **62.50** | 34.75 | 0.96 | **0.32** | 12 / 11 |
| random | 13.60 | 19.85 | 1.93 | 0.19 | 10 / 14 |
| blob | 10.75 | 36.25 | **3.71** | 0.22 | 2 / 3 |
| sprawl | 4.50 | 25.50 | 0.89 | 0.03 | 2 / 4 |
| turtle | 1.75 | 33.75 | 0.41 | 0.01 | 9 / 8 |
| sandbag | 0.75 | 38.50 | 0.66 | 0.01 | 9 / 8 |
| junk | 17 / 27 (2 g) | **0** | 0 | 0 | never closes |

### 2.3 Shape distribution (opponent circles, n=1230)

| shape | n | share | mean area | pop rate | expected banked |
|---|---|---|---|---|---|
| triangle | 761 | 61.9% | 0.80 | 0.14 | 0.69 |
| quad | 266 | 21.6% | 2.15 | 0.19 | 1.74 |
| pentagon/hexagon | 121 | 9.8% | 2.61 | 0.28 | 1.88 |
| poly 7-10 sides | 71 | 5.8% | 4.52 | 0.38 | 2.80 |
| poly 11+ sides | 11 | 0.9% | 10.54 | 0.09 | 9.58 |

Census is triangle-dominated: 62% of all circle events bank under 1.0 area.
"Angel-shaped" circles (area >= 2.5) are only **227 = 18.5%** of the opponent's
circles, but they carry 5.2 mean area vs 0.80 for triangles — i.e. **18.5% of
events carry the area.**

### 2.4 Our own signature (for contrast)

| Metric (our circles) | us = Blue (n=23) | us = Red (n=23) |
|---|---|---|
| first close | **act 3 in 23/23 games** | **act 2 in 23/23 games** |
| first-close area | 4.50 (a fixed 3x3 box) | 4.50 |
| closes/game (mean / median) | 34.93 / 34 | 34.93 / 34 |
| area/close (mean / median) | 2.36 / 2.19 | 2.36 / 2.19 |
| pop rate (all circles) | 0.12 | 0.16 |
| final area, us vs opponent | 45.04 vs 36.27 = **1.24x** | 85.72 vs 25.12 = **3.41x** |

Two hard facts: **we open with the same act-2/3 three-by-three box in 46/46
games** (zero variation — it is a fixed opener, not a decision), and **we are a
popper, not a banker**: 35 closes/game against Great Barrier's 10-13.

---

## 3. AngelWASM target profile and the proxy gap

From `rival-analysis.md` §4, n = 2, **both vs Great Barrier**:

| game | Angel chair | opponent | closes | area | area/close | first close |
|---|---|---|---|---|---|---|
| `65095072` | Blue | GB | 15 | 87.7 | 5.85 | act 12 |
| `3d3f8cb8` | Red | GB | 14 | 36.3 | 2.59 | act 6 |
| **mean** | | | **14.5** | | **4.22** | **act 9** |

AngelWASM's circle signature is **low close count (~15/game) x high area per
close (2.6-5.8) x first close in the act 6-15 mid band** — i.e. a *mini Great
Barrier*. Distance to each corpus archetype on those three axes:

| archetype | closes/g | area/close | first close | d(closes) | d(area) | d(first) | total |
|---|---|---|---|---|---|---|---|
| random | 19.85 | 1.93 | 3 | 0.37 | 0.54 | 0.67 | **1.58** |
| aggro | 34.75 | 0.96 | 10 | 1.40 | 0.77 | 0.11 | 2.28 |
| sprawl | 25.50 | 0.89 | 2 | 0.76 | 0.79 | 0.78 | 2.33 |
| turtle | 33.75 | 0.41 | 8 | 1.33 | 0.90 | 0.11 | 2.34 |
| blob | 36.25 | 3.71 | 2 | 1.50 | **0.12** | 0.78 | 2.40 |
| neck | 39.50 | 1.56 | 10 | 1.72 | 0.63 | 0.11 | 2.46 |
| sandbag | 38.50 | 0.66 | 8 | 1.66 | 0.84 | 0.11 | 2.61 |

**Conclusion: the corpus has no AngelWASM analogue.** The nearest on loop *size*
is `blob` (d=0.12) but blob closes 36 times a game and closes at act 2 — wrong
on two of three axes. The nearest on first-close *timing* is aggro/turtle/neck
(d=0.11) but they close 0.4-1.6 area a game — wrong on size. Every archetype is
a **high-count popper**; AngelWASM is a **low-count banker**. AngelWASM's
closest relative in all recorded data is Great Barrier, which is not in this
corpus at all.

**Therefore no AngelWASM dose may be calibrated on sparring data.** Tuning a
"deny the Angel circle" rule against a popper would optimise against the wrong
target — we already out-close poppers 1.24x (Blue) and 3.41x (Red).

---

## 4. What breaks the circle

### 4.1 Law 1 — a circle only ever dies to a cut

Bidirectional sweep of every action in all 46 games:

```
area-loss events (a territory LOST area)            : 446
...of which a boundary segment was removed that act : 446  (100.0%)
...area lost with NO segment removed                : 0
```

**100.0% of circle deaths, 446/446.** No circle in 5520 actions ever died by
attrition, resegmentation, self-collapse, or a failed re-close. Cuts are the
only cause. (This is *not* tautological — an added segment can in principle
re-partition a face and change counted area without removing anything. It never
did in 5520 actions.)

### 4.2 Law 2 — survival is a function of the opponent's cut volume, not of your loop

| | pearson r |
|---|---|
| opponent cuts/game vs circle pop rate | **0.909** (n=7 archetypes) |
| opponent cuts vs big-circle pop rate | **0.720** (n=28 games) |
| closes/game vs pop rate | 0.254 |
| mean circle area vs pop rate | 0.415 |

**Size does not protect a circle.** Controlling for archetype (removing the
cut-volume confound), *bigger circles are popped MORE*, not less:

| archetype | tiny (<0.5) | 0.5-1.5 | 1.5-4 | 4+ |
|---|---|---|---|---|
| neck (67.8 cuts/g) | 0.25 | 0.35 | 0.53 | **0.73** |
| aggro (62.5 cuts/g) | 0.24 | 0.36 | 0.48 | 0.25 |
| random (13.6 cuts/g) | 0.10 | 0.20 | 0.24 | 0.26 |
| blob (10.8 cuts/g) | 0.00 | 0.00 | 0.41 | 0.18 |

Circle lifetime once closed (opponent circles, n=220 popped): mean 23.2 actions,
**median 12**, p90 67, max 102. That median is short: half of all *popped*
circles die within 12 actions of being closed.

### 4.3 Law 3 — the farm phase transition (this is the disease, quantified)

`prompt.md` disease #2 says "re-closes re-popped by identical move ids" and
`rival-analysis.md` §7 idea 1 proposes a Chebyshev-3 same-ground re-close ban.
Census of that re-close waste (a close is a *re-close* if its centroid is within
Chebyshev 3 of ground we had already held and lost):

| opponent cuts/game | n games | re-close share of our banked area | games with **zero** re-popped area | our area lost |
|---|---|---|---|---|
| 0-5 | 17 | 1% | **17/17** | 1.0 |
| 6-15 | 8 | 2% | **8/8** | 0.3 |
| 16-30 | 13 | 5% | **13/13** | 4.1 |
| **31-44** | **0** | — | — | — |
| 45-60 | 3 | **80%** | 0/3 | 34.6 |
| 61-100 | 5 | **92%** | 0/5 | 102.4 |

```
our area banked, corpus total            : 3729.51
  of which re-close on already-lost ground: 753.91  (20.2%)
  re-close that then died AGAIN          : 554.03  (14.9% of all banked)
our area lost to cuts, corpus total      : 688.79
pearson(opponent cuts, our area lost)    = 0.892   (n=46 games)
area lost per opponent cut               = 0.759   (688.79 / 908 cuts)
```

This is bimodal, not a gradient. Below ~30 cuts a game the farm cycle **never
engages** — 38/38 games have literally zero re-popped area. Above ~45 it
engages completely — 8/8 games spend 73-94% of their area budget re-closing
ground they already lost, and every one of them has re-popped area.

*Caveat, stated because it matters:* the `31-44` bucket has **n=0**. The
transition is therefore bracketed to (30, 45) but **not localised** — the
threshold number itself is unsampled. Do not quote "the farm threshold is 37".

### 4.4 Farm timing — it starts in the act 6-15 mid band

| regime | n | first area-loss act (mean / median / range) |
|---|---|---|
| farm (lost > 5) | 13 | 31.3 / **11** / 7-107 |
| no farm (lost <= 5) | 19 | 71.5 / **78** / 25-118 |

In every farm game the first loop is popped at **act 7-13** — squarely the
`mid(6-15)` band, which is exactly where AngelWASM's own first close sits
(act 6 Red / act 12 Blue). The trap is early and never recovers.

### 4.5 Wall collisions

`cut_this_act` was true on 446/446 pops, so the census has **no separate
"wall collision" class**: a cut either crosses a live boundary (pop) or hits
empty ground (no area event). 908 cuts total produced 446 pops — **49% of all
cuts are wasted**, cutting ground that banks nothing.

### 4.6 Citable individual circles

Largest opponent circles closed, and largest that were then popped:

| file | closed at act | area | verts | outcome |
|---|---|---|---|---|
| `R-chall-random-botIsblue-s1005.moves.json` | 62 | 21.14 | 10 | survived |
| `chall-blob-botIsred.moves.json` | 95 | 21.00 | 8 | **popped act 97** (cage: held 3 of our nodes) |
| `R-chall-random-botIsblue-s1002.moves.json` | 85 | 17.95 | 16 | survived |
| `R-chall-random-botIsred-s2003.moves.json` | 35 | 13.00 | 8 | **popped act 57** |
| `R-chall-random-botIsblue-s1009.moves.json` | 94 | 12.68 | 6 | **popped act 100** |
| `R-chall-neck-botIsred.moves.json` | 47 | 12.17 | 10 | **popped act 82** |

Our own largest popped circles — all of them the same act 25-37 / pop 2-actions-later
signature:

| file | our chair | closed act | area | popped act |
|---|---|---|---|---|
| `chall-blob-botIsred.moves.json` | red | 54 | 20.00 | 95 |
| `R-chall-aggro-botIsred.moves.json` | red | 25 | 12.00 | **27** |
| `R-chall-neck-botIsred.moves.json` | red | 25 | 12.00 | **27** |
| `R-chall-random-botIsred-s2007.moves.json` | red | 58 | 10.10 | 64 |
| `chall-aggro-botIsred.moves.json` | red | 37 | 9.00 | 39 |

Note the pop-latency: aggro/neck pop us **2 actions** after we close. That is
inside the opponent's own current turn, i.e. the circle is never even scorable
before it dies.

### 4.7 Literal "circling us" (cages) is rare

Only **5 of 46 games** (10.9%) contain any circle enclosing one of our nodes:

| file | opp chair | cage circles | / circles | our nodes held |
|---|---|---|---|---|
| `chall-neck-botIsblue.moves.json` | red | 10 | 25.0% | 13 |
| `R-chall-neck-botIsred.moves.json` | blue | 7 | 21.9% | 7 |
| `chall-neck-botIsred.moves.json` | blue | 4 | 8.7% | 5 |
| `R-chall-neck-botIsblue.moves.json` | red | 1 | 2.5% | 1 |
| `chall-blob-botIsred.moves.json` | blue | 1 | 2.5% | 3 |

Pooled cage rate is **0.02** in both chairs. **Any kill-0 built on cage-rate is
unmeasurable at n>=8** — only 23 opponent cage events, in 5 games, 4 of them
against one archetype. The metric for a real AngelWASM dose must be circle
*survival*, not encirclement.

---

## 5. Chair split — the farm is chair-symmetric

The farm cycle hits both chairs almost identically (re-close share):

| archetype | us = Blue | us = Red |
|---|---|---|
| aggro | 92% | 94% / 88% |
| neck | 92% / 80% | 92% / 91% |

**The farm mechanism is NOT a Blue-chair disease.** It is a cut-volume disease
that triggers in both chairs at the same rate. `prompt.md` lists the Blue chair
problem as "close-survival/farming after act 12"; the mechanism is real and
quantified here, but it is chair-symmetric, so the Blue handicap has a
*different* cause. The chair gap is visible elsewhere and is large:

```
final area, us vs opponent:  us=Blue 45.04 vs 36.27 = 1.24x
                            us=Red  85.72 vs 25.12 = 3.41x
our big-circle (area>=2.5) count/game: Blue 7.96, Red 13.22
```

As Red we bank 1.66x more big circles per game than as Blue (13.22 vs 7.96) and
finish 3.4x ahead; as Blue we finish only 1.24x ahead. **Any Blue fix should be justified on the Blue
numbers, not by importing a Red-side mechanism.**

---

## 6. Kill-0 proposal for a future dose

Two gates. **Gate A currently FAILS and must be closed before any Rust work.**

### Gate A — data kill-0 (blocking; status: FAIL)

| requirement | status |
|---|---|
| >= 8 finished AngelWASM-vs-Riposte games, >= 4 per chair, moves on disk | **0 / 8** |
| >= 8 finished AngelWASM games (any opponent) to re-derive the §4 profile at n>=8 both colors | **2 / 8**, and 0 move files |
| every game replays bit-exact on final `scores` | instrument is ready; no input to run it on |
| a valid local AngelWASM proxy exists | **none** (§3 — all 7 archetypes are wrong on 2 of 3 signature axes) |
| >= 8 farm-regime games per chair (opponent cut volume >= 40/game) | **4 per chair** (aggro x2, neck x2) |

**If Gate A fails: no dose. Lane 7 stays a census item.** This is the same
kill-before-gate discipline the v9 laws require, applied to the census itself.

### Gate B — metric kill-0 (only after Gate A passes)

**Primary metric — `RECLOSE_SHARE`:** share of our banked area that is re-close
on ground we had already lost (Chebyshev-3 rule, `rival-analysis.md` §7 idea 1),
measured **only inside the farm regime** (opponent cut volume >= 40/game), per
chair. This is the direct measure of `prompt.md` disease #2 and it is the one
number the census says is actually movable: Law 2 says loop size and close count
are *not* where the loss is, Law 1 says every death is a cut, and Law 3 says the
whole loss is concentrated in one bimodal regime.

| | baseline (corpus, farm regime) | kill threshold |
|---|---|---|
| `RECLOSE_SHARE`, us Blue | 0.73-0.92 | **<= 0.50** in >= 8/9 games |
| `RECLOSE_SHARE`, us Red | 0.88-0.94 | **<= 0.50** in >= 8/9 games |
| re-popped area share of banked | 0.149 corpus-wide (0.37-0.84 in the farm regime) | **<= 0.08** |
| our area lost per opponent cut | 0.759 | **<= 0.55** (i.e. -27%) |

**Required n — the law is not sufficient here.** Measured dispersion makes this
explicit; a **20% relative effect at 2 sigma** needs (dispersion figures are
the pooled within-archetype variance from `metrics3.js`):

| metric | pooled n needed (Blue / Red) | n needed with opponent held fixed |
|---|---|---|
| `RECLOSE_SHARE` | 289 / 206 | not reducible — bimodal, must **stratify** |
| big-circle popRATE | 578 / 346 | 49 / 49 |
| big-circle area banked/game | 49 / 21 | 38 |
| big-circle area surviving/game | 55 / 44 | 40 |

**So: stratify, do not pool.** n>=8 per chair inside the farm stratum is
achievable (the regime is 8/46 games today — it needs 4 more per chair, a
harness job, not Rust), and stratifying is what collapses the dispersion: the
pooled `RECLOSE_SHARE` cv is 1.8-2.1 only because it is averaging a 1% regime
with a 92% regime. **Kill the dose if the farm-regime corpus cannot be grown to
n>=8 per chair — a pooled n>=8 reading of this metric is guaranteed noise.**

**Guardrails (a dose that wins the metric by losing the circles is not a win):**
- our big-circle count/game must not fall below 0.75x control (Blue >= 5.97,
  Red >= 9.91) — blocks "just stop closing";
- opponent final area must not rise > 5% — blocks pure turtling;
- expected banked per close (meanArea x (1 - popRate)) must not fall below
  control (Blue 3.56, Red 4.67 in the big band).

---

## 7. Data acquisition recipe (unblocks Gate A)

1. **AngelWASM moves.** `ENC_USER`/`ENC_PASS` are not in this lane's
   environment and the site API now 404s unauthenticated. Either restore
   credentials for `tools/mined-fetch.js`, or have the coordinator pull
   `65095072`, `3d3f8cb8`, `feec4693` and the 12-game `a2d70517` validation set
   and drop them in as `chall-angel-botIs{blue,red}*.moves.json` — the
   instrument needs no changes, only files in the corpus dir.
2. **Farm-regime games.** Re-run `harness/runR.js` for `aggro` and `neck` with
   more seeds to get from 4 to >= 8 per chair at cut volume >= 40/game. Pure
   harness work, no engine or Rust change.
3. **Localise the 31-44 cut bucket.** Add one sparring bot tuned to ~35-40
   cuts/game to find where the transition actually is. Until then the farm
   threshold is bracketed, not known.
4. **A Great Barrier or AngelWASM banker-style sparring bot.** The entire
   corpus is poppers; the strongest recorded style (GB, 10-13 closes at ~10
   area) and AngelWASM (14.5 closes at 4.22) have no local representative, and
   that is precisely the style lane 7 is about.

---

## Appendix — reproduce

Scripts (analysis only, outside the repo, no engine or bot file touched):
`/tmp/opencode/angel-census/circle.js` (census) -> `census.json`,
`break.js` -> `breaks.json`, `final.js` (Law 1 + Angel distance),
`size.js` (Law 2), `farm.js` (Law 3), `verify.js` (Law 1 bidirectional +
noise floor), `metrics3.js` (n requirements).

```bash
node /tmp/opencode/angel-census/circle.js   /home/genius74o/game  census.json
node /tmp/opencode/angel-census/break.js   /home/genius74o/game
node /tmp/opencode/angel-census/verify.js
node /tmp/opencode/angel-census/farm.js
```

Corpus: `46` files matched `/^(R-)?chall-.*\.moves\.json$/` in
`/home/genius74o/game` (gitignored — see `.gitignore` `*.moves.json`).
Action index = flat move index, matching the `act N` convention in
`rival-analysis.md` §3.
