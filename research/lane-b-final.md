# Lane B session summary — eval terms, one at a time (2026-09-27/28)

Branch: `lane-b-eval` (all steps committed + pushed). Rig:
`retaliator/src/eval_phases.rs` — the shipped `search.rs` skeleton with the
tested terms behind flags; `best_move` = the verified recommendation,
`baseline_best_move` = the shipped search (the faithfulness control).

## Faithfulness harness (step 1, verified repeatedly)

`probe_b_h2h`: control (all added terms off) vs shipped `search.rs`,
position for position over 2 full self-play games — **0 mismatches in 239
positions**, re-verified after every restructure (B-1, B-2, the flag
restructure, the final state). The harness never drifted; every number below
measures the terms, not drift.

## Term results (one at a time, ablation required, n=8-10 both colors)

1. **Legal-cuts-only vulnerability (V4d2 erosion gap) — REJECTED.** ON vs
   OFF: v1 h2h 3/10 vs 4/10 (as-blue -24.8% vs -11.6%), league -25.5% vs
   -20.0% (baseline exact), gauge 8/8 equal (blue margin -11.3pp). Collapse
   line NOT fixed (-120.7% vs -111.2%). WORSE on every gate. See
   `research/lane-b-vuln.md`. `VULN_W` left at 0.0.

2. **(a) Mobility / potential-area for early expansion — SKIPPED (premise
   stale).** `probe_b_shape` measured: first close banks **+9.0 at act 4**
   (site-v1 autopsy: 0.25/loop), avg close ≈ +7.2/loop (GB-class), hull grows
   +4.5..+26.5 per opening action, extend length mostly 3. No
   early-expansion deficit exists in the current search; a term would reward
   existing behavior (ROOM_WEIGHT = noise precedent). GB's wall book grows
   hull only +0.5..+4/action and is separately rejected (-48pp). See
   `research/lane-b-mobility.md`.

3. **(b) Cut+make gap math — mechanism verified; the doom discount
   RECOMMENDED OFF (the merge candidate).** Verified in the engine: the doom
   discount prices the enemy's one-action pop at ×12, but the re-make is
   **shield-delayed two turns** (the pop's placed edge crosses the re-place
   path and is fresh/shielded on our next turn), so the popped area misses
   exactly TWO scoring events. Two re-make-aware corrections never fired
   (`SourceNotOwned` / `BreaksShieldedEdge` always block; the correct form
   needs the enemy's block tree — too heavy for an eval term). The ablation
   that verifies the mechanism: **doom OFF** — league **-9.9% vs -20.0%**
   (+10.1pp), v1 h2h **5/10 vs 4/10** (as-red 4/5 +9.2% vs 2/5 +3.6%),
   collapse lines **-94.5% vs -111.2%** and **-55.9% vs -111.4%**, gauge 8/8
   equal. The doom discount fails the very line it was added for (the V4
   note "1098 -> 1031 against" recorded the same regression). Only
   regression: as-Blue chair 2/5 -> 1/5 (n=5 exact). Dose response
   NON-MONOTONE: 0.5 gives v1 **6/10** (the only config crossing the
   plan-v4 ship threshold; as-blue chair 2/5 kept) but league **-27.3%**
   and collapse rows **-137.7%** (the worst of three doses); its wins are
   narrow coin-flips. See `research/lane-b-gap.md`.

4. **(c) Shield-expiry threats — SKIPPED.** The premise is mechanically
   wrong: enemy fresh edges become cuttable **two of our turns later**
   (measured in the engine during B-2), not "next turn/next action" — the
   planning horizon is 3+ actions, which plan-v4 V4b itself assigns to Lane
   A. No measured disease (our cut volume 34-37/game is field-normal
   ~22-26; we do not under-cut). The B-1 evidence shows the cut-denial eval
   axis is over-priced, not under-priced. Revisit only with new numbers
   (a measured failure to cut maturing walls) or Lane A depth.

5. **(d) Phase gates — SKIPPED.** No measured misfire: the ablations (B-1,
   B-2, V5-1) show uniform phase behavior — PATIENCE never flips a pick,
   the doom's harm is global rather than phase-local. The phase-adjacent fix
   that verified is B-2's doom removal. A phase-gated doom (opening-only)
   is untested; the rig supports it if a phase structure is ever measured.

## Final MERGE/REJECT recommendation to the main session

- **MERGE CANDIDATE (the one item with positive numbers): remove the doom
  discount** from `search.rs` (`DOOM_W` 1.0 → 0.0): league **+10.1pp**
  (-9.9% vs -20.0%), v1 h2h **5/10 vs 4/10**, collapse lines **+16.7pp /
  +55.5pp**, gauge 8/8 equal. Cost: the as-Blue chair loses one win (2/5 →
  1/5, n=5 exact).
- **Alternative:** DOOM 0.5 — v1 **6/10** (crosses the plan-v4 ship
  threshold; the as-Blue chair keeps 2/5) but league -27.3% and collapse
  rows -137.7% (the worst of three doses). Weigh the site threshold
  (plan-v4 Gate 2) against the local league.
- **REJECT (do not merge):** the vulnerability term (worse on every gate),
  the re-make-aware correction (never fires), mobility (no disease).
- Not merged to master from this lane; `lane-b-eval` carries the rig, the
  probes, and the evidence.
