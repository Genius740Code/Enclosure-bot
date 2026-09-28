# v7 Phase System Spec (Lane O → Lane E/S)

**Status**: spec only, unimplemented. Lane O owns the triggers/rationale;
Lane E (eval) implements phase-gated weights, Lane S (search) implements
phase-gated move generation. No eval-code changes were made from Lane O.

## Why phases at all (measured root cause)

The COLLAPSER (floor-0 rush-closer) banks at its own actions 2-3 and pops
30+ at ~t=60 (`research/o-summary.md`). Our 2-ply static eval prices area at
`area × min(events_left, 12)` — a position-only snapshot. Three things the
snapshot cannot express, all confirmed by failed probes:

1. **Rates.** Q11's static trigger (`enemy_area > 15 AND enemy_frontier ≥ 6`)
   fires on the collapser's many small loops and misfires everywhere else
   (2-6, -22.3%, `research/o-q11-prevent-wall.md`). "Enemy area grew 12 in
   the last 4 turns" needs ≥2 samples — a static eval has one.
2. **Opponent tempo.** "Closure 2-4 turns out banking 60 events" = enemy
   close-rate × banked-per-close × events_left. Static eval sees current
   area, not closes-per-turn.
3. **Correct response direction.** Boosting DOOM_W on the Q11 trigger =
   retreat (avoid loops); the right response is CONTEST (occupy the corridor
   root). Response selection is a phase decision, not a weight.

## Phase tracker (game history, not position)

Lane E/S keeps a `PhaseTracker` across the game — updated once per
**turn end** (both players have moved), never inside search:

```rust
struct TurnSnapshot {
    turn: u8,               // position.turn_index()
    score_us: f64, score_foe: f64,
    area_us: f64,  area_foe: f64,
    edges_foe: usize,       // foe construction volume
    fresh_foe: usize,       // foe edges placed last turn (shielded_edges count w/ foe owner)
    breaks_by_foe: u32,     // cumulative cuts foe dealt us
    closes_by_foe: u32,     // cumulative foe Connect moves with area gain > 0
}
struct PhaseTracker { hist: Vec<TurnSnapshot>, /* + running counters */ }
```

Required engine hooks (all public today): `turn_index()`,
`score()`, `area()`, `edges()`, `shielded_edges()`, `MoveOutcome.kind ==
MoveKind::Connect` + area delta for close counting, `broken.is_some()` +
mover attribution for break counting. Close/break counters increment in the
game loop (harness-side), snapshots push on turn end.

Derived rates over window W (default W = 4 turns):

- `foe_area_rate = (area_foe[t] - area_foe[t-W]) / W`
- `foe_close_rate = (closes_by_foe[t] - closes_by_foe[t-W]) / W`
- `foe_build_rate = (edges_foe[t] - edges_foe[t-W]) / W`
- `bank_lead = score_us - score_foe` (cumulative, survives zeroing — cf.
  `spn59l` in `research/findings-gamedata.md`: winner ended 0 area, won on bank)

## Phases and triggers

| Phase | Entry trigger (ALL conditions) | Exit trigger |
|---|---|---|
| BUILD (default) | game start; none of below fire | any below fires |
| CONTEST | `foe_close_rate ≥ 0.75 closes/turn` over last 4 turns AND `turn < 40` (early rush-close signature: collapser closes own-actions 2-3) | `foe_close_rate < 0.4` for 4 straight turns, or `turn ≥ 60` |
| WALL-RACE | `foe_build_rate ≥ 3 edges/turn` over last 4 turns AND `foe_area_rate < 1.0` (GB-style: walls without closing — long diagonals, `research/lane-d-gb.md`) | foe closes ≥ 2 in a turn (they started banking → BANK-RACE), or build_rate halves |
| BANK-RACE | `area_us + area_foe ≥ 25` (both sides holding bankable loops) OR `turn ≥ 60` with `\|bank_lead\| < 200` | game end; if `bank_lead > 300` → DENY (sub-phase) |
| DENY (ahead) | BANK-RACE + `bank_lead ≥ 300` (bank-and-deny endgame `spn59l`: ahead on bank, correct play is pure denial) | `bank_lead < 150` |

Threshold rationale (calibration status: PROPOSED, not tuned — Lane B must
sweep; values anchored to measurements):
- `0.75 closes/turn`: collapser closes ~every turn early (own actions 2-3
  then compounding); GB-style closes ~0.1/turn during wall phase
  (`research/lane-d-gb.md`: first close own-action 7-8, 5-11 total).
- `turn < 40`: collapser's 30+ pop lands ~t=60; contest must start ≥20 turns
  earlier (2-4 turns of construction + travel). Late contest donates necks.
- `bank_lead ≥ 300`: order of one unanswered mid-size bank; below this,
  denial trades away real comebacks.

## Phase → action mapping (who implements what)

**Lane E (eval weights, phase-gated):**
- CONTEST: FRESH_PENALTY 1.5 → 0.0 **only near foe construction**
  (`near_fresh_enemy` already exists in search); CONTACT_PENALTY stays
  (contests that don't cut still donate necks — Q3b counter-1 lesson, 0-8).
- WALL-RACE: REMOTE_BONUS 1.0 → 2.0 (two-front banking while foe walls —
  H-B-D2-BLOB1 logic), PATIENCE stays (don't snatch small).
- DENY: DOOM_W 2.5 → 4.0 + weight foe-area-destruction ×2 (ahead: breaks
  bank; rebuilding loses — P0-3).
- BUILD: current shipped weights (untouched).

**Lane S (move generation, phase-gated):**
- CONTEST: generate the explicit contest set (Lane O prototype
  `retaliator/examples/opp_contest.rs`): CUT_SITE (cut foe edge adjacent to
  foe fresh wall) + OCCUPY (non-breaking land within 2 of foe fresh wall);
  force-pick on full-horizon merit (BONUS=0 test first).
- Other phases: default generation (search already enumerates all legal
  1-ply moves; no new generation needed).

## Anti-thrash rules

- Minimum phase dwell: 4 turns (no flip-flop on one snapshot).
- CONTEST re-entry cooldown: 8 turns after exit (failed contests donate
  necks; don't re-chase immediately).
- All thresholds are turn-indexed, never wall-clock; deterministic
  (no RNG, engine legality only).

## Acceptance (Lane B/E/S gates)

Phase system as a whole is a playable counter-proposal: n≥8 both colors,
`probe_b_h2h ≥ 6/10`, `league_b > -9.9%`, no collapse `< -300`, gauge 6/6.
Trigger thresholds may be swept one variable at a time; each sweep logs
exact config + n≥8.
