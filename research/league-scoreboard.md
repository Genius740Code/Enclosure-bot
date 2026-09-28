# League scoreboard (append-only; date, variant, margins, verdict)

## Baseline — 2026-09-27 (current `search.rs`: v3 + v4 terms, D10-F7 opener)

`probe_v3all` (v1/v2 round robin, 5 openings × 2 colors = 10 games each):
- v3 vs v1: **4/10**. As Blue 2-3 (W: None, 4864; L: 5589, 11723, 9199).
  As Red 2-3 (W: None, 11723; L: 4864, 5589, 9199).
  - v3 vs v1 open=None a=blue 1322-786 A WINS
  - v3 vs v1 open=None a=red 1109-865 A WINS
  - v3 vs v1 open=Some(4864) a=blue 1243-1237 A WINS
  - v3 vs v1 open=Some(4864) a=red 1131-1218 b wins
  - v3 vs v1 open=Some(5589) a=blue 1256-1745 b wins
  - v3 vs v1 open=Some(5589) a=red 1218-1219 b wins
  - v3 vs v1 open=Some(11723) a=blue 1637-1806 b wins
  - v3 vs v1 open=Some(11723) a=red 1343-1259 A WINS
  - v3 vs v1 open=Some(9199) a=blue 1415-2120 b wins
  - v3 vs v1 open=Some(9199) a=red 1541-1578 b wins
- v3 vs v2: **7/10** (matches lineage claim).
`probe_league` (scoutbase, 8 games): AVG margin **-20.0%** (ret perspective).
Genuine games (skip=0) both wins: blue +29.4%, red +38.1%.
Collapse line persists: skip=10/20 red -111.2%, skip=30 red -111.4%
(single 30+ pop ~t=60 + erosion — needs depth or vulnerability pricing).
`gauge` (greedy, 6 games): **6/6** (deterministic repeat of 2 lines).

## Probe — Lane H-verify: genuine probe_league gate with mesh8 wired in — 2026-09-28
Instrument: `probe_mesh_league` (probe-only, bf3c9aa+ba11a79). The genuine 8-game league
structure (skip=0/10/20/30 × both colors, scoutbase) with the mesh8 forced-start prefix
injected on OUR side both colors (`Bot::forced` from `probe_mesh.rs`; our side's play =
mesh8 from our side's first own move in the game — own moves 1-8 — then v3; the
opponent's pre-moves stay bare v3 per the genuine league structure, post-skip scoutbase).
Control reproduced EXACTLY on this box first (AVG -20.0%, genuine blue +29.4/red +38.1,
collapse red -111.2/-111.2/-111.4). **All inj=8/8, 0 stalls** — clean injection on every
row; the opponent's v3 pre-moves never killed a prefix entry.

| skip | ret=blue | ret=red |
|------|----------|---------|
| 0    | +14.3% (1854-1588) | +37.0% (2118-1335) |
| 10   | +14.3% (1854-1588) | +52.8% (2032-958)  |
| 20   | +14.3% (1854-1588) | +52.8% (2032-958)  |
| 30   | +14.3% (1854-1588) | +60.5% (1648-651)  |

**AVG margin: +32.6%** vs -20.0% control (+52.6pp) and vs the -9.9% ACCEPT bar (+42.5pp).
COLLAPSE LINE GONE: red skip rows -111.2/-111.2/-111.4 -> +52.8/+52.8/+60.5 (+498.8pp
across the 4 red rows) — the mesh8 opening as Red avoids the collapse trajectory
entirely. Blue rows identical across all skips (the mesh opening dominates the blue
trajectory; the control shows the same convergence, skip=10 ≡ skip=20). Genuine-blue
caveat: +14.3% vs control +29.4% (-15.1pp) — mesh8 on the EMPTY start as Blue
underperforms the D10-F7 opener, confirming lane-h2's "KEEP blue_opener D10-F7 on the
empty start"; the league flip is carried by the red rows.
Verdict: **league-confirmed** — ACCEPT bar met (AVG +32.6% > -9.9%; worst row +14.3%,
no row near -300). mesh8 stands as adopted (bf3c9aa); keep blue_opener D10-F7 on the
empty start in production.

## Experiment V5-1 — patience-off ablation (Blue-chair first-close timing) — 2026-09-27
Hypothesis: PATIENCE_PENALTY (no sub-2.0 closes before action 12) misfires on
forced openings as Blue, delaying closes v1 snatches.
Change: `PATIENCE_PENALTY` 2.0 -> 0.0 in `retaliator/src/search.rs`.
Result: byte-identical lines in every gate — v3all v1 4/10 / v2 7/10,
league -20.0%, gauge 6/6. Term never flips a pick in any measured line.
Verdict: **REJECTED (no effect)**. Reverted to 2.0 with provenance comment.

## Probe — Lane H blue first-move sweep (forced-action action-1 candidates) — 2026-09-28
Instrument: `probe_bluefirst` (lane-h-bluefirst.md), probe-only, 90 games (45 per matchup).
Fixes catalog §0's no-op: on a forced opening the site's choice plays as engine action 1,
Red's [2,3] via the opponent, then the candidate is forced as Blue's first own move (eng 4).
Baselines reproduce probe_v3all byte-for-byte (all 10 v1-matchup lines); empty-start base ≡
D10-F7-forced (2073-1294 v3-v3, 1322-786 v1) — `blue_opener` is exactly a forced action-1.
Results (side=blue, margin=(ours-theirs)/ours):
- Empty start: D10-F7 best vs both (+40.5% v1, +37.6% v3); A10-C8 2nd (+22.2/+11.0);
  D10-A7 +1.7/−8.4; long diagonals toward center all LOSE (−22.0/−15.8, −28.9/−22.4,
  −41.6/−60.9) despite the best early evals (D10-G10 E9=+121 → still L) — H6 area-first book
  REJECTED.
- Forced openings, no universal fix (per-opening best: 4864→A10-C9 +6.9/+6.9; 5589→D10-G7
  +17.2 v1 only; 11723→base −10.3 best, all 7 candidates L; 9199→D10-F7 +41.2/+32.4 wins both).
  Aggregate 8 games/candidate: D10-F7 3/8 mean −7.7% (worst −49.1), D10-G7 3/8 −12.9%,
  D10-A7 2/8 −13.1%, base 1/8 −25.2% — every candidate's mean vs v1 on forced openings negative.
- Act-5..10 eval is an artifact: identical across candidates that diverge later (9199:
  F7/G7/G13 all −26/−25/−88/−86/+28/+28 → +41.2/−5.8/−65.5 finals) and inverted where it
  differs (9199 base E9=+77 best → −49.8 worst; 4864 winners have the worst evals).
Verdict: **NO ACTION-1 FIXES THE SITE LOSSES — problem is later (close-survival/farming).
Keep `blue_opener` D10-F7 (best empty-start action-1, nothing beats it); reject the
area-first book; do not force D10-F7 at eng 4 (high-variance gamble). Confirm target for B:
9199/v1 base at eng 9 (eval +77, final −49.8).**

## Probe — Lane H2 capy center-mesh forced prefix (lane-g H1) — 2026-09-28
Instrument: `probe_mesh` (research/lane-h2-mesh.md), probe-only, 138 games. Extends Lane H's
forcing pattern to multi-move prefixes (capy mesh 6/8 moves per color, injection-counted,
0 stalls, 90/90 mesh injections complete; b-v1 base rows reproduce probe_v3all 10/10
byte-for-byte). Treatments: base (shipped v3) / mesh6 / mesh8, both colors, 5 openings.
- vs GB 26-move wall book + post-book search (scoutbase or v3, both configs): we win 30/30
  at every treatment (+20 to +76). Lane-g's suppression target holds everywhere with the
  mesh on (GB largest <=47.2, act-38-40 mega window <=15.0 area, <<100). BUT the
  unsuppressed baseline could NOT be reproduced in-harness: book+scoutbase and book+v3
  never build the 51.5+ mega-loop even in base rows (2-3-ply can't see the ~5-move bridge;
  our v3-class play cuts the walls first). Suppression contrast untestable here; mesh is
  free vs GB-class opponents (not the reason to adopt).
- OUR margins, site-forced openings only (n=8 per opponent per treatment, both colors):
  base 11-13 mean -3.6% | mesh6 16-8 +18.3% | **mesh8 22-2 +34.0%** (vs v1 7-1 +32.9,
  vs v2 8-0 +35.4, vs scout 7-1 +33.7).
  As BLUE on forced starts (where blue_opener never fires): base 3-9 mean -17.1% ->
  **mesh8 12-0 mean +43.9%** (+61.0pp swing; the 9199/5589/11723 wipeouts become
  +27.4..+55.5 wins). As RED: base 8-4 +9.9% -> mesh8 10-2 +23.9% (both losses are 5589,
  mild: -12.2 v1 / -10.6 scout). mesh6 REJECTED: leaves the D13 spur dangling, loses
  4864-as-Blue to all three opponents (-25.4/-1.1/-41.3).
- Empty start (never rated): D10-F7 stays best as Blue (base +40.5/+37.4/+29.4 vs
  mesh8 +17.7/+37.2/+14.3; mesh6 loses one, -16.1 vs v2) — Lane H's conclusion stands.
Verdict: **ADOPT mesh8 as the forced-start prefix BOTH COLORS (Blue: ids 16058,4867,14579,
14981,17106,4927,234,14639 on own-moves 2-9; Red: ids 2713,12419,17147,14946,2767,4933,
2406,205 on own-moves 1-8), hand to search after move 8 (or first illegal), KEEP blue_opener
D10-F7 on the empty start. Gate for A/B: rerun probe_v3all + probe_league with the prefix
wired in; watch the 5589-as-Red soft spot.**
