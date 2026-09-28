# Lane D — VladNet-style (small-loop popper + max-pop cutter) vs our bots — 2026-09-27/28

Probe: `retaliator/examples/opp_vladstyle.rs` (always takes the cut that pops
the most enemy area whenever a cut is available — the 41-cut farming face of
`f61a06ec` — else pops the fattest tiny loop it can, else Scout's search for
walls). Tournament: `retaliator/examples/probe_match_opp.rs` (round-robin vs
shipped `search::best_move` = v3, v1base, v2base; 8 games per matchup, 4 with
each color; solo jitter; tiebreak by move index).

## Results (n=8 both colors per baseline; margin from our perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v3 vs vlad | **5-3** | +8.8% | +79 |
| v1 vs vlad | **7-1** | +32.3% | +363 |
| v2 vs vlad | **6-2** | +16.4% | +182 |
| **combined** | **18-6** | **+19.2%** | **+208** |

## Action-numbered pattern of how it beats us

- **The farming fingerprint reproduces locally.** In ALL 6 of its wins the
  mimic cut **39-43x/game** vs our 25-30 (site: VladNet cut us 41x in
  `f61a06ec` vs the ~22-26 background rate). The mimic's max-pop cut targets
  our biggest exposed loop every action; our 2-ply keeps re-closing where we
  were cut.
- Our closes in those losses: 30-39/game at ~0.2-0.9 area/close (tiny) while
  the mimic pops only 10-15 closes at ~2-4 area/close (its greedy max-gain
  close). It converts the actions we burn on re-closing into bigger loops.
- Loss margins -129 to -628; the two -6xx blowups (v3 g4, v2 g4) have the
  mimic holding 21-40 area vs our 7-20 — the farmed-rebuild collapse.

## Anti-rebuild arm (rival-analysis steal #1, tested live)

v2 with the shipped avoid wiring (endpoints of our edges cut in the last 6
actions, fed from the game, CUT_RADIUS 3 / REBUILD_PENALTY 3.0) vs every
style — same solos/colors, so the difference vs the plain `v2` rows isolates
the routing alone (n=8 both colors each):

| Matchup | plain v2 | v2+avoid | effect |
|---|---|---|---|
| vs vlad | 6-2 (+16.4%) | **8-0 (+30.4%)** | **both farmer losses fixed** (g3 -129→+146, g4 -616→+506) |
| vs angel | 3-5 (-1.3%) | 4-4 (+3.8%) | one flip (g1 -67→+19), no regression |
| vs scout | 4-4 (-4.1%) | 5-3 (-2.7%) | one flip (g6 -117→+49), no regression |
| vs gb | 8-0 (+54.3%) | 8-0 (+56.1%) | no regression |

**The anti-rebuild routing is now PROVEN against the farmer**: v2 goes 6-2 →
8-0 vs the vlad-style mimic with the shipped avoid wiring, n=8 both colors,
and it does not hurt any other matchup. The mimic's cut count in its own
losses stayed 39-46 (it still farms — we just stop re-closing where it cuts).

## Hypotheses for Lane A (search)

- **H-A-VLAD1 (KEEP the anti-rebuild wiring — now proven).** The mimic cuts
  our biggest exposed loop every action (39-43x/game) and plain v2 loses
  6/24 to it; with the shipped avoid routing fed live, v2 goes **8-0** vs the
  mimic. Lane A should NOT drop the `avoid` wiring when adding depth — and
  should verify `search_deep` preserves it (the deep search's doom-discount
  drop must not reintroduce re-closing into cut-visited ground). Test:
  `search_deep` + this tournament, target 8-0 vs the mimic retained.
- **H-A-VLAD2 (cut the farmer's loops back).** The mimic holds 10-15 loops at
  ~2-4 area/close — bigger targets than our ~0.3. The shipped cut selection
  treats all cuts equal; ranking cuts by the enemy area destroyed (GB's cut
  bonus, rival-analysis steal #3) should farm the farmer back. Testable with
  the term added vs this mimic, n=8 both colors.

## Hypotheses for Lane B (eval/phases)

- **H-B-VLAD1 (weight closes by gained area, all game).** Our closes bank
  ~0.2-0.9 area in the losses while the mimic's greedy max-gain close banks
  ~2-4. The eval's flat close treatment (any Connect scores the same room
  bonus) is what makes tiny closes competitive. Test: scale the close
  priority by the loop's area (`gain × hz`, continuous, no opening window) so
  a 0.5-area close never outranks a wall that leads to a 3+ area close.
- **H-B-VLAD2 (exposure pricing).** The mimic's max-pop cut works because our
  biggest loop sits exposed. A per-candidate exposure term (own area
  unbanked-if-cut along the move line, weighted by events left) would make
  the search avoid leaving big loops cut-able — test as an eval_phases weight
  table entry.
