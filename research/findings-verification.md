# Track C — Engine Verification Findings

## TOP-LINE SUMMARY
- **DIVERGENCE (strategy-critical): cross-color nested enclosures DOUBLE-count.** A red loop inside a blue loop scores for BOTH (blue banks 9 AND red banks 1, every turn). The "no nested double-count" rule holds only same-color. Overlapping territory is not zero-sum — verifyC.js T8.
- **DIVERGENCE (tactics-critical): mere TOUCH breaks the whole enemy edge.** Endpoint-to-endpoint touch, T-junction landing on an enemy interior, even moving away from a coincident node all cut the entire edge; threading an enemy shared corner hits TWO edges and throws. Breaks are far cheaper/denser than "crossing" implies — T2/T2b/T2c/T3b.
- **Everything else matches assumptions.** Same-color nesting (9 not 10), at-most-one, degree-0-only removal, 1-action blue opener, Chebyshev ≤3 box, 0–18 board, per-turn both-player cumulative banking, 120-lock with scoring resolved, timeout ≡ played turn-end. Two minor flags: `applyTimeout` burns remaining actions from the 120 budget without placing (T11b); a node-wiped player can only timeout, game continues (T14).

## Method
- `verifyC.js` (this dir, kept): 22 tests, all passing (`node verifyC.js` → "22 passed, 0 failed").
- States built via `createInitialState` + field override (segments as `{from,to,invincible:false}`), so each test isolates one rule. No engine logic modified.

## Per-area results

### 1. Intersection / break edge cases
| Case | Verdict | Actual |
|---|---|---|
| Collinear-overlap with enemy | PASS (cuts) | `applyMove(s,[3,9],[6,9])` over red `(5,9)-(8,9)` → red seg removed, both endpoints pruned. Overlap kind `"collinear"` counts as intersection. |
| Collinear-overlap with own | PASS (illegal) | `(2,9)-(5,9)` over own `(0,9)-(3,9)` throws `"A segment cannot overlap one of your own lines."` Control: touching own edge exactly at endpoint is LEGAL (kind `"point"`, not collinear). |
| Touch enemy exactly at ITS endpoint | PASS (cuts whole edge) | `(5,9)-(8,9)` vs red `(8,9)-(11,9)` → red seg gone. Expected "maybe touch is safe" — actual: touch = break. |
| Endpoint lands on enemy interior (T-junction) | PASS (cuts whole 4-long edge) | `(5,9)-(8,9)` vs red `(8,8)-(8,12)` → red seg gone. |
| from-node coincident w/ enemy line, move away | PASS (breaks it) | `(6,9)-(6,12)` with red `(4,9)-(8,9)` through `(6,9)` → red seg gone. |
| Cross two enemy edges at once | PASS (throws) | Throws `"A segment can cut at most one enemy line."` (T3). |
| Pass through shared enemy corner (2 edges, 1 point) | PASS (throws) | `(3,10)-(6,10)` through red corner `(6,10)` → throws at-most-one (T3b). **A single geometric point can count as two breaks.** |
| Cut leaves 3-degree node | PASS | Red star at `(5,5)`; cut `(5,5)-(8,5)` → `(5,5)` survives (still on 2 segs), only isolated `(8,5)` removed (T4). Confirms only-degree-0 removal. |

### 2. Boundary / radius box
- PASS: `to` at x/y = 0 or 18 legal (`(2,0)-(0,0)`, `(16,17)-(18,18)`); `to` at -1 or 19 throws `"That point is outside the board."` Board edge simply clips the legal set; no wrap/pad behavior (T5).
- PASS: box is Chebyshev `|dx|<=3 && |dy|<=3` — diagonal `(9,9)-(12,12)` legal; `(4,0)`/`(0,4)` offsets throw `"Stay within the 3-unit placement box."` (T6).

### 3. Nested enclosure scoring
- PASS same-color: blue outer 3×3 (area 9) + blue inner 1×1 (area 1) → `areas.blue === 9`, control outer-alone === 9. Inner group's faces dropped; outer shoelace keeps the hole counted once (T7).
- **FLAG cross-color: blue outer + RED inner → `{blue: 9, red: 1}` (T8).** Enemy segments are invisible to your territory computation, so both players bank overlapping area every turn. If strategy-notes' "no nested double-count" was read as universal, that reading is WRONG in engine.

### 4. Move-120 truncation
- PASS: `moveNumber=119`, blue 1 action → move ends at 120, turn flips to red with `actionsRemaining = min(2, 0) = 0`, scores banked at game-end (`J` block runs even when the limit triggers it). Further `applyMove`/`applyTimeout` both throw `"This practice game is complete."` (T10).
- PASS: initial state is blue/`actionsRemaining: 1`; after opener red gets 2 (T10b).

### 5. `applyTimeout`
- PASS: banks BOTH areas into scores, flips turn with `min(2, remaining)` actions, clears all invincibility then marks the timing-out player's `currentTurnEdges` — identical end-of-turn block to a played turn (T11).
- FLAG: timeout does `moveNumber += actionsRemaining` — a 2-action timeout at move 10 jumps to 12, burning 2 of the 120 budget with zero placements (T11b). Timeouts are not free passes.
- PASS: timeout never removes segments (T13) — breaking strictly piggybacks on `applyMove` expansion; no standalone break exists.

### 6. Invincibility lifecycle
- PASS (T12, live 3-action sequence + crafted-flag probes): only edges played on the just-ended turn become invincible (stale edges stay vincible); protection covers the ENTIRE opponent turn (verified still-invincible mid-turn with 1 action left, cleared exactly at opponent turn-end); cutting an invincible edge throws `"That segment would cross an invincible line."`, same geometry succeeds once expired. Matches "exactly one opponent turn, then gone forever, recalculated each turn end".
- NOTE (T12b): a player's own just-played edge reads `invincible:true` mid-turn (before turn end clears/re-marks). No gameplay effect (enemy can't move mid-turn); observable only via state inspection.

### 7. Node-wipe edge case (bonus)
- If all of a player's segments are cut, their nodes prune to `[]`; `applyMove` then always throws `"Start from one of your existing nodes."` but `applyTimeout` still works (flips turn, burns budget). Engine does not end the game or skip the player (T14).

## Assumption checklist (strategy-notes list vs engine)
| Assumed | Engine |
|---|---|
| 19×19 (0–18) | ✔ exact (`>= boardSize` illegal) |
| Blue (0,9)→(3,9), Red (18,9)→(15,9) | ✔ exact |
| Blue first turn 1 action, else up to 2 | ✔ exact |
| 120 total placements | ✔ with timeout-burn nuance (§5) |
| \|dx\|≤3 && \|dy\|≤3 | ✔ Chebyshev box, diagonals included |
| At-most-one break | ✔ incl. point-touches and shared corners |
| Only degree-0 removed | ✔ |
| Invincibility exactly one opp turn, recalculated each turn end | ✔ per-edge granularity |
| Breaking piggybacks on expansion | ✔ no other break path |
| Per-turn cumulative scoring for both | ✔ standing loops re-bank EVERY turn (T9: two timeouts → 9/4 then 18/8) |
| No nested double-count | ⚠ SAME-COLOR ONLY; cross-color double-counts |
