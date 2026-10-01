# Lane D2 — long-farm (20-40-action farm cycles) vs our bots — 2026-09-28

Probe: `retaliator/examples/opp_longfarm.rs` — the C2 catalog §4 H1 signature
(`research/c-blunder-catalog.md` on `origin/lane-c-autopsy`: pop ids reused
across 20-40-action spans — 17112 x4, 12114 x3, 2707 x3, 452 x3 — far beyond
CUT_MEMORY 6; §3.2: VladNet farms us WITH anti-rebuild routing live) and
capybara `b4dd04bf`'s fingerprint (`research/g-capybara.md` §4: 41 cuts
re-cutting exactly the edges the rebuilder re-closes). Tournament:
`retaliator/examples/probe_match_d2.rs` (round-robin vs shipped
`search::best_move` = v3, v1base, v2base, the deployed search (master's tip,
v8 stack, probe path), scoutbase, and 1-ply greedy; plus the v2+avoid arm;
8 games per matchup, 4 with each color, the same verified solo jitter and
move-index tiebreak as the blob run). Log: `logs_lane_d2_longfarm.txt`.

## The mimic

- **Build phase**: capybara's center mesh (Lane G §4: length-3 edges heading
  INWARD to the J files — Blue `D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11
  G13-D10 J11-M13`, Red the mirror), 8 book edges closing small triangles —
  the durable web the farmer banks from (VladNet area 48-51.8 from act 40 on
  site, alive all game).
- **THE FARM (the one variable vs the Vlad farmer)**: a **no-decay farm
  ledger** — every edge the mimic has ever cut is remembered forever, and the
  moment the opponent re-closes any of them (it is back in their edges), the
  mimic re-cuts that same ground on priority, the fattest farm-ground cut.
  The rebuilder rebuilds, the farmer re-cuts the same ground: capy
  `b4dd04bf`'s "identical move ids on both sides of the cycle". The cycle
  length is whatever the rebuilder's re-close cadence makes it; the mimic's
  half is always prompt.
- **Else Vlad's rules**: max-pop cut, else fattest close (banks the web),
  else Scout's walls.
- `avoid` (the shipped wiring's own-cut list) is used only to refuse
  re-closes on freshly popped ground unless they bank ≥ 10 — the site farmer
  rebuilds rarely (0-2 repeated closes/game, capy §8). The farm re-cuts
  themselves are untouched by it.

## Results (n=8 both colors per baseline; margins clearly labeled both ways)

| Matchup | W-L (ours) | our avg margin | longfarm persp W-L | its margin |
|---|---|---|---|---|
| v3 vs longfarm | **1-7** | -18.4% (-247) | 7-1 | +13.7% |
| v1 vs longfarm | **3-5** | -8.3% (-22) | 5-3 | -1.5% |
| v2 vs longfarm | 5-3 | -13.2% (+22) | 3-5 | -5.3% |
| v8 (deployed search) vs longfarm | **1-7** | **-62.9% (-484)** | 7-1 | +33.9% |
| scout vs longfarm | **3-5** | -17.9% (-122) | 5-3 | +9.6% |
| greedy vs longfarm | **0-8** | -169.9% (-1751) | 8-0 | +62.3% |
| v2+avoid vs longfarm | 5-3 | -7.9% (+42) | 3-5 | -7.0% |
| **combined (7 arms)** | **18-38** | | **38-18** | |

**The long farm is the most dangerous opponent measured in this repo** —
our baselines go 18-38 across 7 arms (blob 0-32, sac 7-32), and BOTH the
branch snapshot (v3, 1-7) and the deployed search (v8 stack, 1-7, -484)
lose to it. greedy 0-8 at -169.9% is the worst local matchup on record.

## Action-numbered pattern (probe traces + tournament lines)

- **The farm cycle forms against rebuilders and is constant**: the mimic's
  farm re-cuts fire from the moment a re-closed edge is back in our edges;
  smoke-trace cycle spans run **7-12 own actions = 14-24 engine actions per
  farm edge** — 2-4x beyond CUT_MEMORY 6, so the avoid set expires mid-cycle
  every time. The mimic landed 39-50 cuts/game in its wins (farm + ambient),
  26 in the greedy blowouts (it didn't need the farm — greedy never rebuilds,
  so it just out-banked 62.3%).
- **The danger is as Red (the second mover)**: v1 loses 0-4 as Blue (our
  area 4.0-12.5 vs its 21.2-29.2, its cuts 44-50/game) and WINS 3-1 as Red
  (its area collapses to 1.1-9.7 — our Red-side cut volume farms it back,
  capy's red-edge asymmetry in reverse). v2+avoid: 1-3 as Blue, **4-0 as
  Red**. But v3 and v8 lose BOTH colors (1-7) — the newer evals cannot
  convert the Red-side farm race that v1/v2+avoid win.
- **v2+avoid does NOT sweep (5-3, +42)**: the shipped anti-rebuild routing
  fixed the Vlad farmer outright (Lane D: 6-2 → 8-0) but only contains the
  long farm. In its 3 Blue losses our area is 4.5-23.8 vs its 19.8-31.9 with
  cuts 44-50/game: the avoid set (6 actions) forgets the cycle, our
  re-closes on farm ground still outrank fresh ground (a +20-area re-close
  is worth ~240 priority vs the -36 penalty), and the farmer re-cuts 1-2
  actions later. Lane C's H1 prediction CONFIRMED locally.
- **The deployed search (v8 stack) is WORSE than v2+avoid** (1-7 -484 vs
  5-3 +42): the TIE-SYM center-first tie break and the doom tail weight do
  not price the farm cycle, and the v8 eval's own-area × horizon dominates
  into farm ground. The deployed entry (`best_move_routed`, mesh8 prefix +
  timed think) is not run here — the timed think costs 2-4.8s/move, ~1h per
  matchup; it is the next test.

## What this probe proves

1. **CUT_MEMORY 6 is numerically incapable of defending a 20-40-action farm
   cycle** (C2 §4 H1 confirmed): v2+avoid 5-3 vs the Vlad farmer's 8-0, and
   the deployed search 1-7. The avoid set must decay only while the ENEMY
   keeps cutting near those points, and re-closes on known farm ground must
   be priced below any fresh-ground move of the same gain.
2. **The always-on farm is the scarier threat**: capybara's farm switch is
   DETECTOR-GATED (≥2x re-closes, c-blunder-catalog §2.4) — the fingerprint
   probe (`probe_capyfingerprint.rs`, see `d2-capyfingerprint.md`) shows it
   fires rarely vs our bots (0-2 farmed re-closes/game, v2a 7-1) because our
   avoid wiring stops the ≥2x rebuild pattern. A farmer with a no-decay
   ledger (this mimic) needs no detector and farms every re-close. The
   threat model for Lane B/X: farm memory must be modeled on BOTH sides —
   ours (decay) and theirs (always-on).
3. **Red-side conversion is the cheap win**: v1/v2+avoid win as Red 3-1/4-0
   by farming the long farm back (its area collapses to 1.1-17.4). The v3/v8
   evals lose that race — an eval that prices the enemy's farm-cycle tempo
   (its cuts cost it nothing when its own web is durable) converts Red into
   the sweep.

## Hypotheses for Lane B (the eval terms this style demands)

- **H-B-D2-FARM1 (always-on farm threat + enemy-conditioned memory — the
  C2 §4 H1 local form, CONFIRMED).** Two-part: (a) OUR memory must decay
  only while the enemy keeps cutting near those points — per-edge enemy-cut
  counters with no fixed clock (the same mechanism H-B-D2-SAC2 needs), so a
  20-40-action cycle stays remembered to its end; (b) a re-close on ground
  the enemy has cut must be priced below *any* fresh-ground move of the same
  gain — the rebuild penalty must scale with the candidate's own gain
  (catalog §1.5), not the flat -36. **Confirm target:** v2+avoid vs this
  mimic must sweep (8-0, the Vlad-farmer bar) and the deployed search must
  recover to ≥ 5-3; in the 3 Blue losses the re-close on farm ground at own
  action ~9+ (memory expired) must not outrank fresh ground.
- **H-B-D2-FARM2 (farm-race tempo pricing — the Red-side conversion).** The
  long farm's cuts cost it nothing (its web is durable and banks anyway);
  ours cost us the re-close. An eval term pricing the enemy's cut tempo
  against its own web durability — or simply the C2 §2.4 farm-cycle detector
  on OUR side (when the enemy re-cuts our ground ≥2x, route to fresh ground
  permanently) — converts Red into the sweep (v1/v2+avoid already win there:
  its area collapses to 1.1-17.4). **Confirm target:** v3/v8 as Red vs this
  mimic must go to ≥ 3-1 (v1's bar) with the enemy's final area < 10.
- **For Lane X (the site entry):** the v8 deployed entry (mesh8 + timed
  think + beam) is untested vs the long farm — the timed think's cost is the
  blocker locally. A runtime-shimmed probe (no sleep, same selection) should
  run this tournament against `best_move_routed` before any v9 ship: the
  mesh8 prefix opens the center exactly where the long farm builds its web.

## Caveats

- The mimic's mesh book is capy's 8-edge center mesh — a stronger build
  phase than VladNet's actual early game (its web came from search, not a
  book), so the mimic is a *stronger* farmer than the site's. The 0-8 vs
  greedy and the 1-7 vs the deployed search are upper bounds on the danger.
- The farm ledger is fed by the harness (CUT_MEMORY 6 own-cut endpoints for
  the avoid rule) — the farm re-cuts themselves need no history: they fire
  whenever a re-closed edge is back in our edges, which is positional.
- The `avoid` refusal (BIG_RECLOSE 10) fires rarely vs our bots — the
  mimic's mesh re-closes were byte-identical with and without it vs
  scoutbase; the site farmer's rare-rebuild habit is honored but not
  load-bearing here.
- The "v8" arm follows the deployed master tip, which moved (v6 candidate →
  v8 stack) between the first and re-run tournaments; the first run's v5 row
  (-247, the v6-candidate eval) and the re-run's (-484, the v8-stack eval)
  both lose — the row is the re-run (shipped) one.
