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

### Form B run (full credit factor; `research/logs/b3_farm_confirm_full.txt`)

- **Both C2 confirm targets flip under ScaledAll** (and only under it):
  act 51 -> 16127 (+0.0, **off** cut ground) at every memory 6/12/20/40;
  act 112 -> 15669 (+0.0, still on own ground, but the farmed +10.5
  re-close is no longer picked). The flat forms and ScaledExempt keep
  both — the exemption misses the disease (both farmed re-closes are
  close+cut combis).
- Whole-game scorecards (control wants 23 / 18 farmed re-closes):
  ScaledAll flips **17/23, 21/23, 21/23, 23/23** (mem 6/12/20/40) in
  446956a1 and **14/18, 16/18, 18/18, 18/18** in f9c819ed — vs flat
  8-10 and 1-3. The flips route AWAY (+0.0 moves), i.e. deny the farm
  rather than re-rank it: routing, not re-sizing.
- Deployed fidelity re-verified: 60/60 (446956a1), 53/60 (f9c819ed —
  a v3-era game, deployed diverges there per C2 §0).

## Results — full ablation matrix (every cell: v1 h2h 10, league 8, gauge 8; collapse diffs are the score diffs of the red skip=10/20/30 league rows)

cols: v1 = W/L (as-blue W/5 avg / as-red W/5 avg); lg = league AVG margin (blue rows; red rows incl. collapse diffs); gg = gauge (blue margin, red +69.3 in every config — the greedy never cuts us as red there); worst = worst collapse diff.

| config | v1 | lg | gg | collapse worst |
|---|---|---|---|---|
| **control (no avoid; doom OFF)** | **5/10** (1/5 -16.7% / 4/5 +9.2%) | **-9.9%** (blue +29.0 x3, +38.9; red +39.8, -94.5%, -94.5%, -55.9%) | **8/8** (blue +53.5) | **-1038** (-1038/-1038/-472) |
| mem 12 flat | 4/10 (1/5 -19.6% / 3/5 +7.8%) | -6.8% (blue +37.1 x3, +38.9; red +39.8, -80.5%, -80.5%, -83.5%) | 8/8 (blue +52.4) | -901 (-901/-901/-664) |
| mem 20 flat | 4/10 (1/5 -19.7% / 3/5 +7.8%) | -9.3% (blue +30.4 x3, +38.9; red rows identical to mem 12) | 8/8 (blue +52.1) | -901 |
| mem 40 flat | 4/10 (1/5 -19.7% / 3/5 +7.3%) | -11.7% (blue +24.5 x3, +38.9; red +38.4, then identical) | 8/8 (blue +52.1) | -901 |
| mem 6 ScaledAll, form B (full factor) | 3/10 (1/5 -45.1% / 2/5 -11.0%) | -25.4% (blue +14.5 x3, +33.7; red +11.1, -92.6%, -92.6%, -106.1%) | 8/8 (blue +73.1) | -764 (-561/-561/-764) |
| mem 12 ScaledAll, form B | 2/10 (1/5 -83.8% / 1/5 -28.6%) | -31.8% (blue +1.6 x3, +33.1 — the skip0/10/20 blue wins collapse to near-draws) | 8/8 (blue +44.5) | -614 (-539/-539/-614) |
| mem 20 ScaledAll, form B | 3/10 (1/5 -94.0% / 2/5 -21.5%) | -47.6% (blue -19.5 x3 — genuine wins flip to LOSSES; +47.2; red -135.1% x2, -123.3%) | 8/8 (blue +44.3) | -763 (-662/-662/-763) |
| mem 40 ScaledAll, form B | 2/10 (1/5 -75.3% / 1/5 -28.4%) | -70.7% (blue -40.0 x3 losses; +47.0; red -204.4% x2, -101.7%) | 8/8 (blue +46.9) | -804 (-804/-804/-612) |
| **mem 6 ScaledAll, form A (capped hz — the task-literal own_gain x hz)** | **4/10** (1/5 -30.4% / 3/5 -6.0%) | **+1.2%** (blue +16.7 x3, +43.7; red +11.1, **-31.7%, -31.7%, -32.1%**) | **8/8 (blue +76.0)** | **-306** (-270/-270/-306) |

Dose-response reading:

- **Form B (full factor)** is monotone-catastrophic in memory: league
  -25.4 / -31.8 / -47.6 / -70.7% at mem 6/12/20/40, v1 3/2/3/2 of 10,
  as-blue averages -45..-94%. At E>12 the full factor is up to 3x the
  capped one (36 vs 12 at 60 events left), and ~30 cuts/game of NORMAL
  play means the penalty starves ordinary consolidation everywhere, not
  just on farm cycles. It flips both C2 confirm targets (below) — and
  still loses: the mechanism is right, the discrimination is missing.
- **Form A (capped hz)** at the shipped memory 6 is the session's one
  big positive: league **+1.2%** (+11.1pp), the collapse line FIXED
  (-270/-270/-306 vs -1038/-1038/-472; two of three rows inside the -300
  target), gauge 8/8 with blue +76.0 (+22.5pp). The cost: v1 4/10 vs
  5/10 — the as-red 11723 win flips to a -36 loss, as-blue margins
  worsen (1/5 kept). It does NOT flip the mid-game act-51 confirm
  target (the farmed re-close keeps its beyond-horizon credit there),
  but flips act-112 exactly (E<=12, no extension credit) and flips
  12-14 of the 23/18 whole-game farmed re-closes (form-A confirm run).
- **Flat memory extension alone** (12/20/40) is a nothing-burger: league
  -6.8 / -9.3 / -11.7 (non-monotone), v1 4/10 at every depth, gauge
  equal. The collapse rows are byte-identical across 12/20/40 — the
  collapse-line farm is already caught within 12 actions; what the
  flat penalty cannot do is make the re-close lose (36 < 54-198), which
  is exactly the scaled term's job.
- Nobody reaches v1 >= 6/10 (the control itself is 5/10); nobody gets
  all three collapse rows inside -300 (form-A m6s misses by 6 points on
  skip=30).

## Confirm targets (C2 H1) — the mechanism, measured on the site losses

- Deployed fidelity on the probe's avoid reconstruction: **60/60**
  our-actions re-picked on 446956a1 (v4 == master), 53/60 on f9c819ed
  (v3-era game — deployed diverges there per C2 §0). The probe matches
  `lib.rs` replay exactly.
- act 51 (446956a1, re-close 16013 +4.5, on cut ground at every memory,
  picked by the deployed bot AND the doom-OFF control): flipped ONLY by
  form B (-> 16127, +0.0, off ground, at every memory); form A and the
  flat/ScaledExempt forms keep it. Arithmetic: form B penalty 4.5 x
  (12 + 0.5x23) = 106 cancels the whole credit; form A penalty 54 leaves
  the ~52-point extension credit on top.
- act 112 (f9c819ed, re-close 16970 +10.5, ~4 events left): flipped by
  every ScaledAll form (A and B — identical at E<=12), to a +0.0 move;
  flat and ScaledExempt keep it. **The shipped counter-cut exemption
  misses both targets** — the farmed re-closes are close+cut combis
  (`BREAK(opp+0.0)`); ScaledAll is required.
- Whole-game scorecards (control wants 23 / 18 farmed re-closes):
  flat mem 6/12/20/40 flip 8/9/7/10 and 1/2/3/3; form-B ScaledAll flips
  17/21/21/23 and 14/16/18/18 — routing AWAY (+0.0 moves), i.e. denying
  the farm rather than re-ranking it; most flips land on other cut
  ground, a subset off the ground entirely.

## Faithfulness

Re-verified after every restructure, including the final state:
`probe_b_h2h` control (all terms off, doom ON = shipped) vs shipped
`search.rs` — **0 mismatches in 239 positions** (logs
`research/logs/b3_h2h_final.txt`). The control gates reproduce the
published doom-OFF control byte-for-byte (league rows, v1 scores, gauge
margins — `research/logs/b3_ctrl_*.txt`).

## MERGE/REJECT vs the doom-OFF control on ALL gates

- **ScaledAll form B (any memory): REJECT.** Worse on league (by 15-61pp)
  and v1 (by 2-3 wins, as-blue -28..-77pp); gauge 8/8-equal is the only
  non-loss. The dose that flips both C2 targets kills normal play.
- **ScaledAll form A at mem 6: REJECT as a merge, KEEP as the
  documented near-miss.** League +11.1pp (best of the session), collapse
  line fixed (-270/-270/-306, two rows inside -300), gauge 8/8 with
  blue +22.5pp — but v1 4/10 vs 5/10 (the as-red 11723 razor-thin flip
  and worse as-blue margins) fails the "no regression" bar that got
  doom-off its recommendation, and it does not flip the mid-game act-51
  target. The main session may still weigh league+collapse vs v1 (the
  same trade lane-b's DOOM 0.5 alternative poses) — the numbers are all
  above and in `research/logs/b3_*`.
- **Flat memory extension (12/20/40): REJECT.** v1 -1 at every depth,
  league non-monotone (-6.8 best), gauge equal; the collapse rows it
  improves are byte-identical to what form-A m6s achieves while also
  fixing the margins.
- **Root cause (for the next lane): the avoid slice cannot carry
  COUNTS.** A single recent cut and a 4x-re-popped farm cycle are priced
  identically; ~30 cuts/game is field-normal, so any "recently cut"
  penalty big enough to flip the farm also starves normal consolidation.
  C2 H1's original form — per-edge cut COUNTERS (decay >= 30 or none) —
  discriminates them; it needs an API change (counts into the avoid set,
  or repeat-cut detection), which is outside this lane's term form.
  The confirm probe (`probe_b_farm`) is the harness to gate it: fidelity
  60/60, both targets, whole-game scorecard.

All 9 treatment configs + control measured on all 3 gates (26 gate
runs); every config committed + pushed to `lane-b-eval` as it landed.
`search.rs` / `lib.rs` untouched; nothing merged to master from here.
