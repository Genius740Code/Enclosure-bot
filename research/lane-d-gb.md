# Lane D — GB-style (Great Barrier mimic) vs our bots — 2026-09-27/28

Probe: `retaliator/examples/opp_gbstyle.rs` (book of 13 own-actions of long
diagonal walls, mirrored by color — the observed GB book from
`rival-analysis.md` §4 — then Scout's search refusing any close worth < 8.0
area). Tournament: `retaliator/examples/probe_match_opp.rs` (round-robin vs
shipped `search::best_move` = v3, v1base, v2base; 8 games per matchup, 4 with
each color; games differ only in Blue's solo action — the same single-move
variation GB's own book uses; tiebreak by move index; all 8 solos verified
legal via `probe_match_dupcheck`).

## Results (n=8 both colors per baseline; margin from our perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v3 vs gb | **8-0** | +58.8% | +1255 |
| v1 vs gb | **8-0** | +64.1% | +1199 |
| v2 vs gb | **8-0** | +54.3% | +1123 |
| **combined** | **24-0** | **+59.1%** | **+1192** |

The GB-style mimic is the LEAST dangerous style to us: every baseline sweeps
it 8-0. In its losses the mimic gets cut 33-45x/game (its long walls are
cut-able targets) while our bots out-bank it.

## Action-numbered pattern

- The mimic walls own actions 1-13 (book: Blue solo + `D10-E12…G5-I2` NE/SE
  diagonals; Red mirror `P10-O12…L4-J1`), first close at own action 7-8 (the
  book's `I18-J19`/`K18-J19` reinforcement, ~2 area), then closes only 5-11
  times total at 2-4 area/close — far from real GB's 10-13 closes at ~10.
- Root cause (probe trace, gb-as-blue vs scoutbase): the wall phase builds
  area 4→17 by own action 19, then the 2-ply search NEVER finds the big
  cycle-closing moves — Scout's `loop_bonus` plans only triangles a second
  action could complete, and the big region between the two diagonal walls
  needs a multi-action cycle. The mimic banks ~0 instead and its walls get cut.
- This CONFIRMS disease #4 locally: a Scout-class 2-ply cannot convert the
  GB wall book into big banked loops. The book alone is not the threat.

## Hypotheses for Lane A (search)

- **H-A-GB1 (do not re-add the wall book at 2-ply).** The book + 2-ply goes
  0-24 vs our bots (mimic cut 33-45x/game in its own losses). Re-test the
  book ONLY behind real depth: if `search_deep` (3-ply negamax) lands, wire
  the book in front of it and re-run this tournament — the hypothesis is
  that depth finds the big cycle closures the 2-ply cannot (the mimic's wall
  phase reaches area 17 by own action 19 and then stalls).
- **H-A-GB2 (what beats the walls).** Our bots beat the wall style by cutting
  it 33-45x/game while out-banking it. A search-side cut-threat term
  (rival-analysis steal #3) should rank cuts of LONG enemy walls (many
  exposed edges, big area behind them) above stray-edge cuts — testable by
  re-running this tournament with the term added and checking the mimic's
  cut count rises further while our margin holds.

## Hypotheses for Lane B (eval/phases)

- **H-B-GB1 (close-size floor as a phase knob).** The three-way gradient
  (Scout floor 0 → +0.8% avg margin for us; Angel floor 3 → +6.9%; GB floor
  8 → +59.1%) says a close-size floor HURTS a 2-ply bot at 8 but the flat
  patience penalty (sub-2.0 before action 12) does nothing (V5-1: term never
  fires). Test a CONTINUOUS close-size weight instead: scale close priority
  by the loop's gained area (e.g. `priority += gain·hz` for closes, which the
  eval already does implicitly — verify it is not being canceled by the
  dead-wood discount for mid-size closes 2-8).
