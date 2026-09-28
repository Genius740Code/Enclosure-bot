# Q3a GB Unbreakable-Share Census — Spec for C2

## Per-game number

`share(P)` = fraction of player P's **banked** area enclosed in **2+ touch
walls**, banked-point-weighted over the game's scoring events:

```
on every scoring event e (engine: MoveOutcome.scored.is_some()):
    gain[P] = scored area credited to P at e        // Area -> f64
    T[P]    = thick_P_edges / all_P_edges           // snapshot AFTER the move
        thick = edge whose BOTH endpoints touch 2+ P-edges
share(P) = sum_e gain[P](e) * T[P](e) / sum_e gain[P](e)
         = NaN if P banked nothing
```

Degree counts come from the player's own edge-graph only
(`position.edges(P)`: both endpoints of each edge, count occurrences —
same construction as `one_edge_nodes` in `retaliator/src/search.rs`).

## Why this number

Lane D finding (`research/lane-d-gb.md`): the GB mimic's long diagonal
walls get cut 33-45x/game and it banks ~0 — thin walls don't survive.
DENSE_BONUS rationale: walls sharing nodes are near-unbreakable (any cut
touches 2+). If real GB wins by holding thick-walled loops while mimics
lose with thin ones, `share` separates them. Hypothesis H-Q3a: **the
higher-share player wins** (bank quality, not bank volume).

## Reference implementation (Rust, local)

`retaliator/examples/opp_census.rs` — `thickness()` + `census_game()`.
Legality via engine only; deterministic (index tiebreaks inherited).

## Running it on site games (C2)

Port to `tools/mined-analyze.js` (replay path already exists, bit-exact):

```js
// per game, per color P:
let acc = {blue:0, red:0}, tot = {blue:0, red:0};
for (const line of game.moveHistory) {
  const before = snapshot(); applyMove(line); const after = snapshot();
  // scoring event <=> cumulative score increased for either color
  for (const P of ['blue','red']) {
    const gain = after.score[P] - before.score[P];
    if (gain > 0) { acc[P] += gain * thickness(after, P); tot[P] += gain; }
  }
}
share[P] = tot[P] > 0 ? acc[P]/tot[P] : NaN;
// thickness(state,P): degree map over P edges; thick = both ends deg>=2.
```

`thickness` needs edge-endpoint degrees from engine.js state: reuse the
replay's edge list per color (`applyMove` already tracks edges for break
attribution — see `research/findings-gamedata.md` Method). Cost: O(edges)
per scoring event, negligible next to replay.

Prediction test: over score-decided games (highest signal), fraction where
`share(winner) > share(loser)`. Baseline to beat: 50%. Report n, ties
(exact-equal shares), and NaN games (a side banked nothing) separately.

## Local validation (Lane O, available data)

GB-mimic vs scoutbase, 8 solos x both colors (n=16) — see `opp_census.rs`
output (committed with results). Site-game test BLOCKED: no cached
`tools/mined-data`, fetch needs site credentials (not attempted; HTTP-429
policy applies). C2 runs the js port when data is available.
