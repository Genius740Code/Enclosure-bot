# Lane B3: farm-cycle detector + scaled rebuild routing (C2 H1) — 2026-09-28

Branch: `lane-b-eval` (worktree `game-b3`, local branch `work-b3`, push
HEAD:lane-b-eval). Successor to lane-b2 on the same rig:
`retaliator/src/eval_phases.rs` + the gate probes. Ownership respected:
only `eval_phases.rs`, `probe_b_*` / `league_b` / `gauge_b`, this file,
`research/logs/b3_*` and scoreboard appends. `search.rs` / `lib.rs`
untouched.

## The disease (C2 `c-site-losses-2` H1, confirmed in rated play)

- The avoid set (`lib.rs` replay, CUT_MEMORY 6) forgets farm cycles that
  C2 measured at **20-40 actions**: 17112 pops us at [52],[72],[92],[104]
  (re-pops 20 actions apart); we re-close the same triangle with the same
  move id (16013 at [3] and [51]); f9c819ed re-closes 16970 at
  [33],[68],[84],[113]. 61% of our >= +4.5 closes are popped within 3
  actions; 47% of our cuts pop nothing.
- The shipped routing cannot flip these re-closes even when it remembers
  the ground. **The arithmetic, in full-horizon points** (the eval's
  ranking and selection units: area x min(events, 12)):
  - flat cut-ground penalty: REBUILD_PENALTY 3.0 x hz = **36** at hz 12;
  - the farmed re-close's capped-area credit: own_gain x hz = **54-126**
    for the site re-closes (+4.5 x 12 for 16013; +10.5 x 12 for 16970);
    at act 51 of 446956a1 (~35 events left) the total credit including
    the 0.5-weight beyond-horizon extension is 4.5 x 23.5 = **106**;
  - **36 < 54**: the flat penalty never flips the pick; the re-close keeps
    18-90 net credit when the avoid set catches it at all.
- Worse: both C2 confirm re-closes are **close+cut combis**
  (`BREAK(opp+0.0)` each time — the break pops nothing of the enemy's),
  so the shipped exemption (non-breaking first actions only) means the
  deployed bot pays **0**, not 36, on exactly the moves that feed the farm.

## The term (B-3)

One term, two knobs, one at a time from the doom-OFF control:

1. **Memory** (`B3_MEM`, probe-side, caller-owned like the site's): the
   avoid set keeps the endpoints of our edges cut within the last
   `B3_MEM` actions (lib.rs replay semantics, memory generalized).
   Sweep **12/20/40** vs the shipped 6. Lane D proved the shipped mem-6
   wiring itself flips the vlad-style farmer 6-2 -> 8-0 (n=8 both
   colors); the gates here previously ran with NO avoid set at all
   (control = `best_move`, avoid `&[]`).
2. **Scaled penalty** (`Rebuild` gate + `REBUILD_GAIN_W = 1.0`,
   eval-side): on cut ground, in ranking AND selection,
   `priority/adjusted -= REBUILD_GAIN_W x own_gain x hz`, full-horizon
   units, deterministic tiebreak by move index, legality via the engine
   only. At W=1.0 the re-close's capped-area credit cancels exactly, so
   any fresh-ground move with positive net area outranks it. Two forms:
   - `ScaledExempt`: keeps the shipped counter-cut exemption
     (non-breaking first actions only) — measured to show what the
     exemption costs (both C2 targets are breaking combis).
   - `ScaledAll`: prices breaking re-closes too — the farm-cycle form.
     The flat part keeps its non-breaking domain, so pure counter-cuts
     (gain 0) pay nothing (retaliation untouched).
   The beyond-horizon extension credit (0.5 x max(0, events-12)) is left
   in place and measured; a full-factor escalation is the backup dose.

## Rig + faithfulness

- `best_move_with_avoid` restored to the control config (doom OFF, every
  added term off — including CLOSE OFF: the lane-b2 sweep verdict; HEAD
  carried the last swept state T=3.0 for reproducibility only). The B-3
  term is gated per entry point (`farm_best_move_with_avoid`), so the
  control path is code-identical to the published control.
- `probe_b_h2h` after the restructure: **0 mismatches in 239 positions**
  (control vs shipped `search.rs`), re-verified before any claim below.
- Control reproduction after the harness change (gates now track cuts and
  take `B3_MEM`/`B3_SCALE`; defaults = absent = control): league
  **-9.9%** with byte-identical rows (1080/767, 1950/1174, 1098/2136
  x2, 2135/1305, 843/1315; collapse diffs **-1038 / -1038 / -472**),
  v1 **5/10** (blue 1/5 -16.7%, red 4/5 +9.2%, identical scores), gauge
  **8/8** (blue +53.5, red +69.3). Logs `research/logs/b3_ctrl_*.txt`.

## Gates per config

v1 h2h (10 games), league_b (8), gauge_b (8); the gates feed OUR side's
avoid set live (opponents frozen: scoutbase/v1/greedy have no routing —
site-class baselines). Watch the as-Blue chair. Targets: v1 >= 6/10,
league better than -9.9%, no collapse loss worse than -300 diff.

## Confirm probe (`probe_b_farm`)

Replays the two C2 site losses; checks (1) the deployed process
(`search::best_move_with_avoid` + mem-6 reconstruction) re-picks the
site move (validates the probe against `lib.rs` exactly), (2) the two
confirm targets flip off the re-close and off the cut ground per
config, (3) a whole-game scorecard: of the farmed re-closes the CONTROL
picks (Connect, gain > 0, on ground cut within 40 actions), how many
each config flips, and how many flips land off the config's own cut
ground. Fixed site trajectory, per-action comparison.

### Form A run (capped factor hz = min(E,12); `research/logs/b3_farm_confirm_formA.txt`)

- 446956a1: deployed fidelity **60/60** our-actions re-picked — the
  probe's avoid reconstruction is `lib.rs` exactly. Both the deployed
  bot and the doom-OFF control pick the farmed re-close 16013 (+4.5) at
  act 50; it is on cut ground at every memory (6/12/20/40 all Y).
  **NO config flips it** — flat forms because 36 < 54, scaled forms
  because the re-close keeps its beyond-horizon extension credit
  (4.5 x 0.5 x ~23 ~= 52) on top of the penalty-capped remainder
  (~64 net) and no fresh move outscores that. Dose correction: the
  scaled penalty now uses the FULL credit factor (min(E,12) +
  0.5 x max(0, E-12)) — `REBUILD_FULL` in eval_phases.
- f9c819ed (v3-era game, deployed fidelity 53/60): at act 112 (~4
  events left, no extension credit) the arithmetic is exact: the
  ScaledAll forms **flip** the +10.5 farmed re-close 16970 (penalty
  10.5 x 4 = 42 cancels the whole credit); the flat forms and
  ScaledExempt keep it — **the shipped counter-cut exemption misses
  both C2 targets** (both farmed re-closes are close+cut combis).
- Whole-game scorecards (control wants 23 / 18 farmed re-closes):
  flat mem 6/12/20/40 flip 8/9/7/10 and 1/2/3/3; ScaledAll flips
  12/15/15/17 and 14/16/18/18 — the scaled term is 2-9x more active
  against exactly the disease; most flips land on OTHER cut ground
  (the gain-scaled penalty re-ranks cut-ground closes by size), a
  subset land off the ground entirely.

## Results

| config | v1 h2h (blue/red) | league (collapse rows, diff) | gauge | notes |
|---|---|---|---|---|
| control (no avoid) | 5/10 (1/5 -16.7% / 4/5 +9.2%) | -9.9% (-94.5% -1038 / -94.5% -1038 / -55.9% -472) | 8/8 | baseline, reproduced exactly |
| mem 12 flat | 4/10 (1/5 -19.6% / 3/5 +7.8%) | **-6.8%** (blue +37.1 x3, +38.9; red -80.5 -901 / -80.5 -901 / -83.5 -664) | 8/8 (blue +52.4, red +69.3) | league +3.1pp but v1 -1 win (as-red 4864 flips to a loss, as-blue 9199 -804); gauge blue -1.1pp |

## Sweep table

| config | v1 h2h (blue/red) | league margin (collapse rows, diff) | gauge | verdict vs control |
|---|---|---|---|---|
| control (no avoid) | 5/10 (1/5 -16.7% / 4/5 +9.2%) | -9.9% (-94.5% -1038 / -94.5% -1038 / -55.9% -472) | 8/8 | baseline |
| mem 12 flat | | | | |
| mem 20 flat | | | | |
| mem 40 flat | | | | |
| scaled @ chosen mem | | | | |

## MERGE/REJECT vs the doom-OFF control on ALL gates

(to be written after the sweep)
