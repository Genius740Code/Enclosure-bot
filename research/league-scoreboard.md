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

## Experiment V5-1 — patience-off ablation (Blue-chair first-close timing) — 2026-09-27
Hypothesis: PATIENCE_PENALTY (no sub-2.0 closes before action 12) misfires on
forced openings as Blue, delaying closes v1 snatches.
Change: `PATIENCE_PENALTY` 2.0 -> 0.0 in `retaliator/src/search.rs`.
Result: byte-identical lines in every gate — v3all v1 4/10 / v2 7/10,
league -20.0%, gauge 6/6. Term never flips a pick in any measured line.
Verdict: **REJECTED (no effect)**. Reverted to 2.0 with provenance comment.

## Lane D style-mimic tournament — 2026-09-28 (branch `lane-d-opp`)

Round-robin of 4 style-mimic opponents vs shipped `search::best_move` (v3) +
v1base + v2base. 8 games per matchup (4 with each color), games differ only
in Blue's solo (8 verified-legal solos, GB's own book-varying trick);
tiebreak by move index. Margin from OUR perspective. Log:
`logs_lane_d_tournament.txt`; probes `opp_gbstyle/opp_vladstyle/
opp_angelstyle/probe_match_opp/probe_match_dupcheck`.

| variant (matchup) | margins (W-L, avg) | verdict + note |
|---|---|---|
| gb (book 13 own-actions + close floor 8) vs v3 | 8-0, +58.8% (+1255) | we sweep; mimic cut 33-45x/game in own losses — 2-ply can't convert the wall book (disease #4 confirmed) |
| gb vs v1 | 8-0, +64.1% (+1199) | we sweep |
| gb vs v2 | 8-0, +54.3% (+1123) | we sweep |
| vlad (always max-pop cut + max-gain close) vs v3 | 5-3, +8.8% (+79) | **farmer fingerprint: vlad cuts 39-43x in all 3 wins vs our 25-30** |
| vlad vs v1 | 7-1, +32.3% (+363) | farmer fingerprint in the 1 loss (43 cuts vs 25) |
| vlad vs v2 | 6-2, +16.4% (+182) | farmer fingerprint in both losses (39/43 cuts) |
| angel (close floor 3.0, no book) vs v3 | 6-2, +8.8% (+193) | close counts match, area/close decides |
| angel vs v1 | 6-2, +13.1% (+212) | as above |
| angel vs v2 | **3-5, -1.3% (+26)** | **only sub-.500 matchup: floor 3.0 is the danger threshold** |
| scout (scoutbase, floor 0) vs v3 | 5-3, +0.2% (+22) | **most dangerous style**: collapse lines persist (-426, -145) |
| scout vs v1 | 4-4, +6.4% (+104) | near even |
| scout vs v2 | **4-4, -4.1% (-31)** | collapse lines: -855, -661 (scout 28-44 area on 10-26 closes vs our 8-17) |
| v2+avoid (shipped avoid wiring, live cut points) vs gb | 8-0, +56.1% (+1170) | no regression vs plain v2 |
| v2+avoid vs vlad | **8-0, +30.4% (+333)** | **anti-rebuild routing PROVEN: both farmer losses fixed (6-2 → 8-0), n=8 both colors** |
| v2+avoid vs angel | 4-4, +3.8% (+81) | one flip, no regression |
| v2+avoid vs scout | 5-3, -2.7% (-12) | one flip, margin still negative — close-size gap remains |

Style gradient by opponent close-size floor (our avg margin): Scout 0 →
+0.8%, Angel 3 → +6.9%, Vlad greedy → +19.2%, GB 8 → +59.1%. **The more the
opponent pops tiny loops, the worse we do; the GB-style book is harmless at
2-ply.** Details + hypotheses per style: `lane-d-scout.md`, `lane-d-angel.md`,
`lane-d-vlad.md`, `lane-d-gb.md`.
