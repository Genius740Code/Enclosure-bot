# Lane G — capybara net v5 fingerprint (site leader recon), 2026-09-28

Target: `medium capybara net v5` — site leader, rating ~1778, 100+ rated games, wasm,
bot id `8a3d9816-6a2c-45f8-aec8-941cc402f378`. Previously uncharacterized.

Method: `GET /api/games?limit=100` paged 8x (800 games scanned; cursor) → 145 capybara games,
132 finished → **130 replayed legally** in the vendored `meridian-engine` (0 illegal moves),
plus 2 GB-win deep dives (`e42677d0`, `2ab9c5a0`) fetched separately = **8 games deeply**.
Action numbers are 0-based engine indices (action 0 = Blue solo, then Red 1–2, Blue 3–4, …).
Move ID = `direction*361 + source` (rival-analysis §2). Raw JSONs + probe outside the repo at
`/tmp/opencode/sitegames-g/` (`capy_probe/` = standalone crate, path-dep on the engine).

## 1. Headline (aggregate, 130 finished games, replayed)

| metric | capybara | opponents |
|---|---|---|
| W-L | **86-44** (66%) | — |
| as Blue / as Red | 37-30 (n=67) / **49-14** (n=63) | — |
| closes/game | 17.6 | 18.1 |
| cuts/game | 23.9 | 25.8 |
| final area | 49.4 | **78.6** |
| final score | 2090 | 2544 |
| first-close action | **9.1** | 11.9 |
| area at first close (self/opp) | 2.6 / 3.3 | 4.9 / 6.4 |
| largest loop (median / max) | 15.6 / 60.6 | 26.2 / 132.7 |
| avg loop gain per close | 5.72 | 8.12 |
| final-area / closes | 2.92 | 5.19 |
| first 15 actions mix | 75% Extend, 23% Connect | — |
| close cadence (gap between closes) | mean 6.3, median 4.0 engine actions | — |

**It holds LESS area than its opponents on average (49.4 vs 78.6) and posts a LOWER average
score (2090 vs 2544) — and still wins 86-44.** The wins are narrow grinds (+363 avg margin,
58 of 86 in the +100..500 band, max capy score in a win 3709); the losses are concentrated
blowouts (−2054 avg, 31 of 44 under −1500, all vs the mega-loop builders).

## 2. Fingerprint (the requested dimensions, with action numbers)

1. **First-close timing + area both sides.** First close at act 9.1 avg (range 6–11 in the 8
   deep games), always tiny: self area 2.6 at that point (first loop gains are +1.25 ×many and
   +13.75). Opponents first-close at 11.9 avg (GB 13–15 by book; VladNet/riposte v4 at 2–4).
2. **Avg area/close.** 5.72 per close (loop-gain basis), final-area/closes 2.92 — between
   VladNet/Stompy (1.6–1.8) and AngelBot (5.8); nowhere near GB (~10–12). Largest loop median
   15.6, max 60.6 — it does build occasional big loops but not as a system.
3. **Cuts/game.** 23.9 baseline (background rate, ~like everyone) — but **adaptive by
   opponent**: 41.5 vs riposte v4, 37.9 vs AngelBot WASM, 37.8 vs Flux, 37.0 vs yeezus (the
   farmer mode) vs 23.7 vs GB, 20.5 vs GB 0.2, **7.6 vs xmybot** (quiet-builder mode).
4. **First 15 actions (walls vs closes).** 75% Extend / 23% Connect ≈ 5 extends + 2–3 closes
   per game as both colors. It builds a **center mesh**: as Blue
   `D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10 J11-M13…`; as Red the mirror
   `P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8…` — length-3 edges heading
   INWARD to the J/K files, unlike GB's diagonals to the far corners (J19/K2).
5. **Largest loop banked.** Median 15.6 / max 60.6 (vs opponents 26.2 / 132.7). In the 8 deep
   games capy's largest was 13.8–42.7; opponents' largest was 8.2–51.5 in the games capy
   WON and 51.5–164.0 in the games capy LOST (see §4).
6. **Color dependence.** As Red: 49-14, cuts 27.4/game, area 43.5 — as Blue: 37-30, cuts
   20.5, area 55.0. Largest-loop and cadence are color-independent (15.5 vs 15.8; 6.3 gap
   both). The red-edge edge: more cut volume as Red (the second mover harasses more), and the
   first action as B varies (`D10-D13` / `A10-C12` / `D10-D9`) consistent with the site
   forcing Blue's action 1 (lane-c-autopsy §0).
7. **No opening book.** 67 distinct first-13-action lines in 67 games as Blue; 59 distinct in
   63 as Red. Pure search (or a net), not a memorized line. No repeated-byte book like GB's.
8. **Rebuild behavior: rare.** 0–2 repeated closes of the same edge per game (0 in 3 of the 6
   deep dives). It does NOT rebuild-in-place as a habit — but see §4: it re-CUTS rebuilders.

## 3. Per-opponent (n≥3)

| opponent | n | W-L | capy closes/cuts/area/largest | opp closes/cuts/area/largest |
|---|---|---|---|---|
| Great Barrier (1907) | 45 | **34-11** | 19.3 / 23.7 / 48.3 / 18.0 | 16.6 / 26.0 / 71.1 / 21.0 |
| Great Barrier 0.2 | 24 | 9-15 | 18.5 / 20.5 / 51.3 / 15.0 | 19.4 / 23.7 / **113.6** / 54.0 |
| xmybot | 10 | **7-3** | 20.8 / **7.6** / 81.5 / 18.0 | 12.5 / 11.5 / 118.8 / 84.6 |
| AngelBot WASM (1613) | 10 | **9-1** | 11.5 / 33.9 / 42.7 / 14.9 | 17.4 / 37.9 / 30.8 / 25.2 |
| VladNet (1584) | 7 | **5-2** | 17.1 / 25.3 / 47.7 / 16.3 | 23.4 / 30.7 / 52.4 / 16.2 |
| Flux | 4 | **4-0** | 10.5 / 37.8 / 28.1 / 13.8 | 24.2 / 30.5 / 22.0 / 13.0 |
| Atlas v2 | 4 | 3-1 | 14.8 / 30.0 / 49.1 / 14.4 | 22.8 / 26.5 / 65.1 / 51.2 |
| riposte v4 (ours) | 4 | **4-0** | 11.2 / **41.5** / 29.2 / 15.2 | 26.5 / 32.8 / 18.7 / 21.2 |
| yeezus | 3 | 3-0 | 16.0 / 37.0 / 28.3 / 16.2 | 11.3 / 37.3 / 21.2 / 8.5 |

Two regimes vs GB-proper: **wins (n=34) are low-scoring grinds — capy 1755 vs GB 1327;
losses (n=11) are mega-loop blowouts — capy 2863 vs GB 4804.** Same split vs GB 0.2
(wins 1381 vs 1064; losses 2783 vs 5555). Capy suppresses the GB mega-loop ~75% of the time.

## 4. Deep dives (8 games; capy side marked)

### Losses to the builders
- `a1234a15` capy-B L vs GB, 2487-4779. Capy leads 180-30 at act 30 (GB banks ~0 until its
  mega-loop; first close act 13 but only 2.5 area), 1046-1032 at act 60; GB's second
  mega-loop (largest 51.5, avg-gain 12.04) catches it ~act 70-90 (GB 2646 at 90). Capy: 20
  closes avg 6.25, largest 18.0, repeated closes I5-K3 2x, O11-N8 2x. Capy's cuts → GB area
  drop within 2 actions: mean 0.52, **1 of 26 cuts lands** (max 13.5).
- `7346ea17` capy-R L vs GB, 2370-4920. GB banked early too (first close act 15, 180 by act
  30). Capy's 8 big closes are each answered by a GB cut with the FULL loop dropped:
  act 53 +13.0 → −13.0; 73 +42.7 → −42.7; 82 +32.8 → −32.8; 98 +16.8 → −16.8; 102 +18.0 →
  −18.0; 110 +16.8 → −16.8; 118 +7.0 → −7.0 (GB re-cuts the J5-M8 / G8-J11 corridor
  repeatedly; those edges closed 2x each). Pure area-lifetime asymmetry, GB side.
- `b713a140` capy-R L vs AngelBot WASM, 3149-3243 (closest loss). Capy OUT-AREAS AngelBot
  65.5 vs 40.5 with bigger loops (largest 16.0, avg-gain 8.01 vs 5.96) and still loses: the
  deficit is the score integral — AngelBot banks from act 11 and capy trails 743-982 at act
  59, closing to 94 at the end. Mid-weight vs mid-weight; whoever banks first, wins.

### Wins
- `b6dddfb0` capy-B W vs VladNet, 2788-2703. Capy 84.3 area vs VladNet 68.8; 16 closes avg
  6.32 vs VladNet's 27-close popper (avg 5.36); only 3 of capy's big closes answered (drops
  0–5.4); 2 double-duty cut-closes.
- `e7876498` capy-R W vs VladNet, 2206-2125. Close gains `[1.25, 13.75, 1.25, 1.75, 1.25,
  13.75, …]` — alternating tiny re-closes with 13.75 loops; only 1 big close answered. Win by
  steady banking, not by structure.
- `b4dd04bf` capy-R W vs riposte v4, 1187-945. **The farm, from the winning side:** capy cuts
  41; its repeated cut-edges are F4-G7 ×3 and A5-D7 / A8-D10 / F4-I4 / F6-I7 / C5-A8 / I3-F6
  / D9-E12 / I7-L7 / L4-L7 / A10-D10 ×2 each — and riposte v4 re-CLOSES exactly those edges
  (A5-D7, F4-I4, F4-G7, A8-D10 ×2 each). Identical move ids on both sides of the cycle: the
  rebuilder rebuilds, capy re-cuts the same ground. This is the exact fingerprint VladNet
  farmed us with (rival-analysis §5, c-blunder-catalog §3.2) — in reverse.
- `e42677d0` capy-B W vs GB, 2016-1471. Capy leads from act 29 and never trails. **GB's
  mega-loop never forms: largest 26.0, avg-gain 8.88** (vs 51.5 / 12.04 in capy's GB losses);
  GB's closes stay at book-4.5 scale, area 5.0 at act 30, 20.7 at 50, never above ~54. Capy
  cuts → GB area drop: mean 0.52, 1/26 lands — the suppression is NOT cut damage. Capy runs
  10 double-duty cut-closes (J4-M7 ×3 cutting O4-L6 / N3-L6, M2-P5 ×2, E15-H13 ×2, E16-H13
  ×2, E11-H13 ×2) and re-closes in place (J4-M7 ×3) — and wins while GB is suppressed.
- `2ab9c5a0` capy-R W vs GB, 1914-1465. **GB largest loop 8.2, avg-gain 4.06 — completely
  suppressed.** GB banks early (156 by act 30) and capy comes from behind (38-156 at 30,
  957-1418 at 100) with a late burst: capy gains ~950 score in the last 19 actions while GB's
  area is cut at the end (GB 41.4→55.0 flat). Capy cuts → GB drop: mean 0.39, 3/17 land.

## 5. Comparison to known styles; does it exploit area-lifetime asymmetry?

- **Not GB** (no book; first close 9.1 vs 13–15; 17.6 closes at 5.7 avg vs 10–13 at ~10–12;
  center mesh vs far-corner diagonals). **Not VladNet** (17.6 closes vs 30+; avg gain 5.7 vs
  1.6–5.4; no memorized red opening — 59 distinct lines in 63 games). **Not Stompy/Scout**
  (closes later than the act-4 rushers, loops 8–10x bigger per close). **Not AngelBot**
  (similar mid-weight, but capy's cut volume adapts to the opponent; AngelBot's doesn't).
- Closest description: **an adaptive mid-weight steady banker** — center-mesh development,
  first close by act ~9-11, then closes its best available loop every 4–6 own-actions at 5.7
  avg gain, background-rate cuts vs builders, farm volume (30–41 cuts) vs rebuilders, and no
  structural attachment to any loop (0–2 repeated closes/game).
- **Area-lifetime asymmetry: NO in the classic sense.** It does not hold more durable area
  (49.4 vs 78.6 — it loses the area battle to every big builder, including VladNet and
  xmybot). What it exploits is the **temporal side of the score integral**: score accrues
  from the first turn-end, so early+steady banking (even +1.25–13.75 loops) converts time
  into score before late mega-bankers come online — GB banks ~0 until act 38-40 in half its
  games, and capy's act-9 head start is worth ~300+ score by then (`a1234a15`: 180-30 at act
  30). Its wins are exactly those games where the opponent's mega-loop is suppressed or
  absent (opponent avg score 1401 in wins vs 4779 in losses).
- **The farm side confirms the shared thesis from the winning side:** vs rebuild-in-place
  bots (riposte v4, Stompy, AngelBot, Flux, yeezus) it plays 30–41 cuts/game re-cutting the
  exact edges the opponent re-closes, and wins those matchups nearly perfectly (4-0, 9-1,
  4-0, 3-0). Lane C's "the best reply to a rebuilder is to keep cutting the same ground" is
  now observed from BOTH sides of the table.
- **New lesson for us (not in any prior file): a mega-loop builder can be suppressed WITHOUT
  cut damage.** In both capy-vs-GB wins, capy's cuts landed on GB's area 1–3 times out of
  17–26 (mean area damage 0.39–0.52 per cut) yet GB's largest loop stayed ≤26 (vs 51.5+ when
  it loses). The suppression is structural: the center mesh occupies the corridor GB's book
  expands through, and steady contest + double-duty cut-closes keep every GB region small.
  GB's 26-move book assumes uncontested diagonals; against the mesh it hands over a position
  where its closes stay at book-4.5 scale and the mega-loop never assembles.

## 6. Testable hypotheses for Lanes A/B

- **H1 (Lane A — center-mesh suppresses the GB book).** The capy center mesh (Blue:
  `D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10…`; Red: `P10-M8 M8-J10 J10-M13
  M8-J11 M13-J11 M13-P11 M13-P10…`) correlates with GB's mega-loop never assembling (GB
  largest loop ≤26.0, avg-gain 4.06–8.88 in capy wins vs 51.5+ / 11.7–12.04 in capy losses)
  and with 34-11 vs GB. TEST: in the engine probe, replay GB's 26-move book as Red against
  the capy center-mesh line as Blue (and mirrored), with both sides then handing to search;
  measure GB's largest loop, area trajectory and final score vs the same book against our
  current openings. Confirm target: GB's largest loop ≤~30 and its act-38-40 mega-loop fails
  to bank >100 area. Note the −48pp caveat on our own GB book (SUBAGENT-BRIEF #4): the mesh
  CONTESTS the center instead of walling the far corners — the test must verify our follow-up
  search can convert the mesh, else it dies the same death.
- **H2 (Lane B — steady mid-size banking + adaptive farm volume).** Capy's edge is cadence
  and adaptivity, not structure: first close by act ~9-11 (NOT act 4 like our old bot, NOT
  act 13+ like GB), then close the best available loop every 4–6 own-actions at 5.7 avg gain,
  gated by lane C's per-close survival pricing (c-blunder-catalog §1.4) so doomed closes don't
  get full credit; AND switch cut volume to farm mode (30–40 cuts targeting the opponent's
  re-closed ground) only when the opponent re-closes the same edge ≥2x (farm-cycle detector,
  c-blunder-catalog §2.4) — capy's 7.6 cuts/game vs xmybot vs 41.5 vs riposte v4 is the
  adaptive fingerprint. TEST: in `probe_league` (deduped per lane C §2.3) + a riposte-v4-style
  spawner + a GB-clone gauge, measure avg margin both colors and the cuts/game split vs
  baseline; expect the narrow-grind win profile (mode +100..500) and a flip of the
  rebuilder matchups.

## 7. Caveats

- Action numbers are 0-based engine indices throughout (rival-analysis uses the same; lane C
  uses 1-based probe-n — offset by 1 there).
- Engine `MoveOutcome::scored` is the (areaB, areaR) pair banked at each turn-end scoring
  event (61 events/game) — NOT the mover's loop gain. Loop gains here are computed as the
  live-area delta across the move (area after the move minus area after the previous action).
  An earlier draft of this analysis mislabeled that; all numbers above use the corrected metric.
- The 130-game aggregate pools all opponents (including humans' bots like bowot, nitram,
  NormallyNormal, PurpleDuck, Elias, ercraftcito, Conbomb, Areax, john.fun/v2 — the W-L and
  averages are dominated by the rivals tabled in §3).
- One 109-ply game (`2c0e154c`, vs bowot) included as-is; all others 120 actions.
- Site first-action forcing (lane C §0) means capy's "no book" conclusion rests on variation
  from action 1 onward as Blue; its first action varies there consistent with forcing.
- No repo src files touched; artifacts: this file only. Probe + JSONs at
  `/tmp/opencode/sitegames-g/`.
