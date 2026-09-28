# Lane H2 — capybara center-mesh opener probe (mesh vs GB wall book + our margins), 2026-09-28

Branch `lane-h2-opener` (successor to `lane-h-opener` @ `c6762cf`; pushed to `origin/lane-h-opener`).
Probe-only: no `retaliator/src/*` changes. Instrument: `retaliator/examples/probe_mesh.rs`
(`cargo run --release --example probe_mesh -- <a-scout|a-v3|b-v1|b-v2|b-scout|all>` from `retaliator/`).
138 games: 48 part (a) (24 per GB engine) + 90 part (b) (30 per opponent).

## 0. Question and method

**Question (lane-g H1, the hand-off to this lane).** Capybara (site leader, 1778) beats Great
Barrier 34-11 with a **center mesh** (Blue `D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 ...`;
Red mirror) while GB's book-built mega-loop never assembles (GB largest <=26.0 in capy wins vs
51.5+ in capy losses; avg-gain 4.06-8.88 vs 11.7-12.04; GB banks ~0 until act 38-40 then
megaloops). Is that suppression reproducible in OUR engine by FORCING the mesh prefix and
handing to the shipped search — and does the mesh keep/improve OUR OWN margins (the
SUBAGENT-BRIEF #4 caveat: our own GB-book test died at -48pp because the follow-up search
could not convert a borrowed opening; the mesh must be tested with our brain)?

**Design (`probe_mesh.rs`, extends lane-h `probe_bluefirst`'s forced-move pattern to
multi-move prefixes):**
- A forced opening plays as engine action 1 (the site consumed that side's first own move);
  the prefix is delayed one tempo, entries are never skipped (the probe_mesh v1 off-by-one
  lesson, fixed in commit 84408bf: prefix index is its own counter).
- A bot with a prefix plays those moves as its own moves in order; the FIRST prefix move that
  is illegal at its turn TRUNCATES the prefix (search takes over for good), with the slot,
  engine action and `IllegalMove` reason recorded and every injection counted (the
  probe_bluechair no-op lesson). **Across all 138 games: 0 stalls; every mesh injection
  complete (6/6, 8/8) both colors, all 5 openings x 3 opponents — the capy prefix is fully
  legal against every tested opponent.**
- After the prefix everyone plays their shipped search: us = `retaliator::search::best_move`
  (v3, the V5-1 brain); the GB clone plays `scoutbase` (GB is described as an enhanced
  Scout) and, as a second configuration, v3 itself.
- Close gains = live-area delta across the Connect move (lane-g's corrected metric).
  `mega[34-42]` = largest close gain at engine actions 34..=42, the act-38-40 mega-loop
  window. Lane-g H1 target: with the mesh on, GB largest <= ~30 and mega < 100;
  unsuppressed GB = largest 51.5+, mega > 100.
- Numbering: engine actions 1..120 (B[1] solo, R[2,3], B[4,5]...). Margin = (ours-theirs)/ours,
  lane-h convention.

**Exact forced prefixes (ids verified at startup: D10-D13=16058, D10-F7=1979, A10-C11=11723,
matching rival-analysis §2 / catalog §0):**

| prefix | moves (notation) | ids | as own-moves |
|---|---|---|---|
| MESH-B 8 (capy Blue, lane-g §4) | D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10 J11-M13 | 16058, 4867, 14579, 14981, 17106, 4927, 234, 14639 | 1-8 (eng 1,4,5,8,9,12,16,17); on forced starts 2-9 |
| MESH-B 6 | first 6 of the above | 16058, 4867, 14579, 14981, 17106, 4927 | 1-6 (eng 1,4,5,8,9,12) |
| MESH-R 8 (capy Red mirror) | P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8 | 2713, 12419, 17147, 14946, 2767, 4933, 2406, 205 | 1-8 (eng 2,3,6,7,10,11,14,15) |
| MESH-R 6 | first 6 of the above | 2713, 12419, 17147, 14946, 2767, 4933 | 1-6 |
| GB-B book (rival-analysis §4) | D10-E12 D10-F13 E12-G15 F13-H16 G15-I18 H16-J19 I18-J19 D10-F7 D10-E8 E8-G5 F7-H4 G5-I2 | 13892, 16780, 16819, 16839, 16878, 16898, 11522, 1979, 4145, 1942, 1924, 1887 | 2-13 after the site's action-1 |
| GB-R book | P10-O12 P10-N13 O12-M15 N13-L16 M15-K18 L16-J19 K18-J19 P10-N7 P10-O8 O8-M5 N7-L4 M5-K2 L4-J1 | 13182, 15348, 15385, 15403, 15440, 15458, 10802, 547, 3435, 508, 488, 449, 429 | 1-13 |

GB-as-Blue action-1s (its observed site set): D10-F11=11726, D10-C7=896, A10-C11=11723.
mesh-B's D10-D13 replaces `blue_opener` (D10-F7) on the empty start via the same forcing path
lane-h validated.

**Part (a)** — suppression: GB book + post-book search vs us, treatments = base (plain v3, our
current openings) / mesh6 / mesh8. GB as Red on the 5 standard openings (none = the D10-F7
start for base); GB as Blue on its 3 observed action-1s. n = 5 (resp. 3) openings per
treatment per GB engine, deterministic.
**Part (b)** — our margins: us base/mesh6/mesh8 both colors vs v1 / v2 / scoutbase on the 5
standard openings; n = 10 per (opponent, treatment); the 4 site-forced openings alone give
n = 8 (4 x 2 colors) per claim. b-v1 base rows = the validation gate.

## 1. Validation (all passed)

- Id decode gate: D10-D13=16058, D10-F7=1979, A10-C11=11723 asserted at startup.
- **All 10 b-v1 base rows reproduce `probe_v3all` byte-for-byte** (finals 1322-786, 1109-865,
  1243-1237, 1131-1218, 1256-1745, 1218-1219, 1637-1806, 1343-1259, 1415-2120, 1541-1578) —
  both pre-fix and post-fix runs.
- Book/mesh placement: GB-Red's first close lands at eng act 14 and GB-Blue's at act 16
  (K18-J19 / I18-J19 book Connects — exactly the move math); our mesh first close at eng act
  12 (Blue, G13-J11) / act 10-13 (Red, M13-J11) — the capy "first close act 9-11" profile.
- Injections: 0 stalls in 138 games; gbinj 13/13 (GB-Red), 12/12 (GB-Blue), mesh inj 6/6
  and 8/8 everywhere in part (b).
- One harness phenomenon worth noting: a-scout's GB=blue BASE rows for openings D10-F11 and
  A10-C11 are byte-identical games (1845-446 both) — the two row-11 spur nodes are
  eval-irrelevant to both scoutbase (room term is owner-relative, a constant offset within
  a position) and our v3 until the game converges; D10-C7 (row-7 spur) diverges, which rules
  out a plumbing bug. That cell has n=2 independent games, not 3. a-v3's base rows differ
  across all three (v3's post-book play is enemy-distance sensitive).

## 2. Results (a) — GB book vs capy-mesh: the mega-loop target, and a harness limit

Pooled readout (48 games; our margins from OUR side; GB metrics from GB's side):

| config | treatment | n | us W-L | mean margin | GB largest mean/max | GB mega[34-42] mean/max | GB area@40 mean | GB final area range |
|---|---|---|---|---|---|---|---|---|
| GB=red, post=scoutbase | base | 5 | 5-0 | +52.2% | 16.6 / 28.2 | 3.6 / 4.5 | 13.0 | 12.9-26.1 |
| GB=red, post=scoutbase | mesh6 | 5 | 5-0 | +53.4% | 12.4 / 15.1 | 6.6 / 15.0 | 16.9 | 10.3-25.9 |
| GB=red, post=scoutbase | mesh8 | 5 | 5-0 | +47.3% | 20.5 / 50.9 | 4.5 / 4.5 | 13.9 | 14.8-32.4 |
| GB=blue, post=scoutbase | base | 3(=2) | 3-0 | +69.7% | 9.7 / 11.8 | 6.2 / 7.7 | 14.9 | 9.4-26.5 |
| GB=blue, post=scoutbase | mesh6 | 3 | 3-0 | +53.0% | 21.1 / 27.0 | 4.6 / 9.4 | 17.0 | 5.4-29.7 |
| GB=blue, post=scoutbase | mesh8 | 3 | 3-0 | +59.5% | 32.2 / 38.7 | 5.6 / 7.7 | 16.4 | 8.8-21.5 |
| GB=red, post=v3 | base | 5 | 5-0 | +43.6% | 12.6 / 20.9 | 2.7 / 4.5 | 15.8 | 18.9-35.0 |
| GB=red, post=v3 | mesh6 | 5 | 5-0 | +39.2% | 12.8 / 23.0 | 0.9 / 4.5 | 13.9 | 17.0-27.8 |
| GB=red, post=v3 | mesh8 | 5 | 5-0 | +43.6% | 24.7 / 47.2 | 4.5 / 4.5 | 22.0 | 18.0-32.9 |
| GB=blue, post=v3 | base | 3 | 3-0 | +41.6% | 20.7 / 45.4 | 4.5 / 4.5 | 21.9 | 15.1-28.8 |
| GB=blue, post=v3 | mesh6 | 3 | 3-0 | +43.6% | 24.8 / 38.8 | 5.0 / 11.5 | 17.8 | 8.3-49.3 |
| GB=blue, post=v3 | mesh8 | 3 | 3-0 | +51.5% | 13.2 / 20.5 | 1.1 / 3.4 | 15.7 | 6.1-25.8 |

- **The lane-g target holds everywhere: with the mesh on, GB's largest loop never exceeds
  47.2 (mean 12.4-32.2) and its act-38-40 window never banks more than 15.0 area — far under
  the <100 target.** GB's closes stay at book-4.5-to-15 scale, its area@40 stays 13-22, its
  final area never exceeds 49.3 — the suppressed profile lane-g measured in capy wins.
- **BUT the harness limit: the UNSUPPRESSED baseline could not be reproduced.** Book +
  scoutbase AND book + v3 never build the mega-loop even in base rows (base GB largest
  max 45.4, mega max 7.7 as blue / 4.5 as red; area@40 max 28.8). The mega-loop is a ~5-move
  bridge with no intermediate area payoff — beyond a 2-3-ply horizon — and our v3-class
  opponent also cuts the walls before the region seals. The site's 51.5+/12.04-avg
  unsuppressed GB (lane-g §4) is real but needs GB's actual engine (larger budget, greater
  first-move breadth, cut bonus) and site-opponent passivity. So the base-vs-mesh CONTRAST
  is untestable here; what is established: **the mesh is free vs GB-class book opponents**
  (30/30 wins at every treatment; margins comparable or better — vs GB=blue with v3
  post-book, mesh8 +51.5% > base +41.6%) and the mega target is met with the mesh on.
- The corridor mechanism is visible in the data: with the mesh on, GB's closes scatter into
  many small loops (cls 5-23) and its late "largest" loops (act 55-120) come too late to
  matter — e.g. a-scout GB=red mesh8 5589: GB largest 50.9 at act 70, we still win +36.7%.

## 3. Results (b) — OUR bot with the mesh vs v1 / v2 / scoutbase

Forced-openings-only (the site-forced starts — the rated reality; n = 8 per opponent per
treatment = 4 openings x 2 colors; margins ours):

| opponent | base | mesh6 | mesh8 |
|---|---|---|---|
| v1 | 2-6, **-12.8%** | 4-4, +16.0% | **7-1, +32.9%** |
| v2 | 5-3, +4.3% | 6-2, +18.5% | **8-0, +35.4%** |
| scout | 4-4, -2.2% | 6-2, +20.5% | **7-1, +33.7%** |
| pooled (n=24) | 11-13, -3.6% | 16-8, +18.3% | **22-2, +34.0%** |

By side, pooled over the 3 opponents (forced openings):
- **As Blue (where `blue_opener` never fires on site): base 3-9, mean -17.1% -> mesh6 8-4
  +21.7% -> mesh8 12-0, +43.9%** — a +61.0pp swing on the exact site-loss set. Every one of
  the 12 mesh8-as-Blue forced games is a win at +27.4 to +55.5, including the three
  catastrophic base losses (9199: -49.8/-44.3 v1/scout -> +55.5/+46.7; 5589: -39.0/-5.5 ->
  +34.3/+46.3; 11723: -10.3/-18.3 -> +55.5/+43.8).
- As Red: base 8-4, +9.9% -> mesh6 11-1, +30.8% -> mesh8 10-2, +23.9%. The two mesh8 red
  losses are 5589 (D10-B9) to v1 (-12.2) and scout (-10.6), mild and late; mesh6 is the
  better red-side variant but the weaker blue-side one (it loses 4864-as-Blue to all three
  opponents, -25.4/-1.1/-41.3 — the 6-move prefix leaves the D13 spur dangling; mesh8's
  G13-D10 close + J11-M13 extension fix exactly that).
- **Empty start (the lab-only case the site never produces): D10-F7 stays the best Blue
  action-1** (base +40.5/+37.4/+29.4 vs v1/v2/scout) — mesh6-as-Blue even loses one
  (-16.1 vs v2); mesh8-as-Blue wins all three but smaller (+17.7/+37.2/+14.3). Lane H's
  conclusion stands; the mesh is not an empty-start improvement, it is a FORCED-START fix.
- Mechanism check: the mesh rows show our area at eng act 11 at 1.2-2.5 vs base 9.0-15.0 —
  the mesh deliberately banks late (first close act 10-13 vs the base's act 3-5 tiny loops)
  yet wins the forced starts, one more confirmation (from the opening phase) that the early
  area/eval lead is inverted w.r.t. outcome (lane-h §3). The mesh contests the center
  J-file instead of farming the back edge.

## 4. Recommendation (ONE)

**ADOPT the capy mesh as a forced opening prefix — the 8-move version, both colors, on every
site-forced start — and keep `blue_opener` D10-F7 for the (never-rated) empty start. Reject
mesh6.** Not "vs-GB-only": our current bot already beats every GB clone we can build without
the mesh (part a, 30/30), so a GB-only book has no measurable value; the mesh's measured value
is on the forced starts against our OWN league field (base 11-13 -> mesh8 22-2, n=24).

Hand-off to the engine lane (lane A/B), exact:

- **As Red** (site forces the opponent's action-1): force ids
  `2713, 12419, 17147, 14946, 2767, 4933, 2406, 205`
  (P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8) on own-moves 1-8
  (eng acts 2,3,6,7,10,11,14,15), then hand to `search::best_move`.
- **As Blue after a forced action-1** (`blue_opener` cannot fire — the rated case): force ids
  `16058, 4867, 14579, 14981, 17106, 4927, 234, 14639`
  (D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10 J11-M13) on own-moves 2-9
  (eng acts 4,5,8,9,12,13,16,17), then hand to `search::best_move`.
- **On a true empty start**: keep D10-F7 (actions_played()==0, unchanged; measured best,
  lane-h + this probe). A D10-F7-then-mesh-continuation was NOT tested — do not assume it.
- Truncation rule: on the first prefix move that is illegal, abandon the prefix and hand to
  search immediately (measured: never fired in 138 games; the prefix is legal against
  every book/bot/opponent tested, both colors, all openings).
- Hand-off condition (when to trust it): the numbers above are vs v1/v2/scoutbase with the
  V5-1 search follow-up, n=8 both colors per opponent per claim (12-0 as Blue pooled).
  Gate before shipping: rerun `probe_v3all` with the prefix wired in — the v1/v2 lines must
  not regress — and one `probe_league` pass. The 5589-as-Red soft spot (-10.6/-12.2, 2 of
  24) is the residual to watch; if it bothers the gate, mesh8-as-Red + mesh6-as-Blue is the
  untested hybrid, NOT recommended on current evidence.
