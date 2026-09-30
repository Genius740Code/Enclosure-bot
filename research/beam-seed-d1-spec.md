# BEAM-SEED D1 spec (external session, 2026-09-30) — widen 8→12 + HARD guard

Premise filed (44%→59% outside top-8; setup-credit diffuse, bad ranking term).
Code facts: analyze_timed is WALL-CLOCK-boxed (SOFT 2000 floor, HARD 4800 cap)
— widening buys breadth with per-line depth, not ms. Root already fully ranked
(~400 moves in memory); per-line reply passes are the cost (~1.16ms/line;
~218 full 8-line rounds ⇒ depth in surplus, lines scarce). Filed v1 KILL is a
depth kill dressed as clock kill; do not re-litigate, measure hard_frac.

Recommendation: (a) BEAM 8→12 + round-aware HARD guard (never START an
unfinishable round; backstop kept). One const + ~6 lines. Do NOT touch WIDTH
(probe determinism) or v7base.rs (frozen control). Sweep 8→10→12, ship smallest
passing rung. (b) seeded alternates deferred (screen hit rate unpredicted;
73/285 eviction lesson); (c) hybrid rejected for D1.

Gates: G0 reachability (probe_beamrank: cov(N), beam_incl@8; KILL if ≥85% or
cov(12)−cov(8) <10pp — cheapest kill in program) → G1 mechanism+clock (control
= guard-only @8; hard_frac ≤ control+5pp binding; min-line-depth; tighten
bench_timed assert 6800→4500; clock precondition: dose-0 WASM already over bar,
Lane T owns clock — gate BEAM on delta-vs-control natively) → G2 quality
(h2h ≥6/10 both chairs ≥3/5, league bars, gauge 6/6; PRIMARY missed-area-
closes/game ≥10% below control via lane-x-eval MissedCloses port ~40 lines).
G2 beam_incl-up-but-missed-flat (±3%) = KILL. Full detail in session paste.
