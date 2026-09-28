# Lane B: cut+make gap math — the doom discount's over-discount, measured (2026-09-28)

Branch: `lane-b-eval`. Harness: `retaliator/src/eval_phases.rs`; probes
`probe_b_gap` (trajectories) / `probe_b_v1` / `league_b` / `gauge_b` /
`probe_b_shape`. Faithfulness control first (0 mismatches / 239 positions,
re-verified after every restructure).

## The mechanism (cut+make gap, verified in the engine)

The doom discount prices the enemy's worst one-action pop of our area at the
full horizon: `worst × min(events_left, 12)`. But a popped loop is re-made
only after a **two-turn shield delay**:

1. The pop *places* an edge (every move places one) whose segment crosses our
   cut edge — that crossing is how the cut happened.
2. That placed edge is **fresh** → shielded for us on our next turn, so the
   re-place of the cut edge is **illegal** on our turn right after the pop.
3. The shield drops after our turn ends; the re-place becomes legal on our
   second turn after the pop.

So the popped area misses exactly **TWO scoring events**, not twelve — the
true doom is `worst × 2`, not `worst × 12`. Two implementations of a
re-make-aware correction never fired:

- Checking the re-place from the post-pop position: always fails
  `SourceNotOwned` (it is the enemy's move there — `check_move` checks for
  the player to move).
- Checking it after every legal reply: always fails `BreaksShieldedEdge`
  (the pop's placed edge always crosses the re-place path).

The correct form needs the enemy's block tree over their remaining actions
(too heavy for an eval term), so the term itself was abandoned and the
mechanism was verified by the ablation below instead.

## Ablation: doom discount ON (shipped) vs OFF

n=8-10 both colors per gate; all bots deterministic (one run each is exact);
the only change is `DOOM_W` 1.0 → 0.0 in the rig copy.

| Gate | DOOM ON (shipped) | DOOM OFF | Delta |
|---|---|---|---|
| league vs scoutbase (8 g) | **-20.0%** (red -111.2/-111.2/-111.4) | **-9.9%** (red **-94.5/-94.5/-55.9**; blue +29.0-38.9) | **+10.1pp** |
| v1 h2h (10 g) | **4/10** — blue 2/5 avg -11.6%, red 2/5 avg +3.6% | **5/10** — blue 1/5 avg -16.7%, red **4/5 avg +9.2%** | **+1 win** (as-red +5.6pp; as-blue -1 win) |
| collapse line (skip=10 ret=red) | 1031/2177 = **-111.2%** | 1098/2136 = **-94.5%** | **+16.7pp** |
| collapse line (skip=30 ret=red) | 735/1553 = -111.4% | 843/1315 = **-55.9%** | **+55.5pp** |
| gauge vs greedy (8 g) | **8/8** (blue +66.0, red +69.3) | **8/8** (blue +53.5, red +69.3) | equal W/L |

**The doom discount fails the very line it was added for.** plan-v4 V4 added
it for the catastrophic synthetic red line; the V4 note "collapse line
1098 -> 1031 against" recorded the regression (our score DROPPED with the
term on) and kept it anyway on "genuine games unchanged". On the current
search the regression is larger and the term also costs +10.1pp league and 1
v1 win.

Chair"5589" trajectory (doom off vs on, same openings): the doom-off bot's
midgame diverges at act ~80 (close to +11.5 gap, no cut) vs the doom-on bot
(close to +5.2, cut); final -38.3% vs -39.0%.

## Dose response: 0.5 tested — NON-MONOTONE

| Gate | DOOM 1.0 (shipped) | DOOM 0.5 | DOOM 0.0 (off) |
|---|---|---|---|
| league (8 g) | -20.0% | **-27.3%** (worst) | **-9.9%** (best) |
| collapse rows (red) | -111.2/-111.2/-111.4 | **-137.7/-137.7/-104.7** (worst) | **-94.5/-94.5/-55.9** (best) |
| v1 h2h (10 g) | 4/10 | **6/10** (best) | 5/10 |
| gauge (8 g) | 8/8 | 8/8 | 8/8 |

The 0.5 dose crosses the ship threshold on v1 (6/10 > 5/10 — plan-v4 Gate 1:
"beats v1 head-to-head") and keeps the as-Blue chair at 2/5 — but its wins
are narrow coin-flips (4864-blue +0.5%, 5589-red +3.6%) and it loses 9199-red
(-8.0%) that 0.0 wins (+5.2%). Meanwhile its league (-27.3%) and collapse
rows (-137.7%) are the WORST of the three doses. No configuration dominates.

## Why the over-discount costs so much

The doom discount nets every poppable big loop to ~0 (area × 12 credit minus
worst × 12 doom), so the bot never builds one — it builds split/dense ground
and cuts. v1 and scoutbase (no doom discount) build big loops (+31..+57 per
close in the chair games) and out-bank us; the re-make delay (2 turns) means
the "doomed" balloon is actually only 2 events of risk.

## Regression flag

As-Blue v1 chair: 2/5 -11.6% (doom ON) vs 1/5 -16.7% (doom OFF) — n=5,
exact for this opening set (deterministic bots). The doom discount helps
the Blue chair by one win. If the main session wants that win back, the
rig supports a dose test (flip `DOOM_W` + the `doom` flag); 0.5 was tested
worse on the collapse line in the V4 era but not on the current gates.

## Verdict

**RECOMMEND DOOM OFF** (`DOOM_W` gated off in `best_move`;
`baseline_best_move` keeps it ON = the shipped search, which the
faithfulness probe checks position for position, 0/239). This is a REMOVAL
with ablation numbers, not a new term. The main session decides the
`search.rs` merge; do not merge to master from this lane. The
vulnerability (VULN) and re-make (REM) terms: REJECTED / never-fire — see
`research/lane-b-vuln.md` and the const docs.

Dose alternative: DOOM 0.5 gives v1 6/10 (the only configuration crossing
the plan-v4 ship threshold) and keeps the as-Blue chair at 2/5, but its
league (-27.3%) and collapse rows (-137.7%) are the worst of the three
doses — the main session weighs the site threshold (Gate 2) against the
local league. The dose response is non-monotone; 0.25 untested.
