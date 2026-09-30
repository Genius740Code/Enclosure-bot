# GB-line gate design retry (external session, 2026-09-30) — method (c) + Stompy controls

Confirms both tracker defects (alternation assumption; unreachable in h2h —
gate must call init_opp_close_tracker per our turn in the harness loop).
Corpus: GB won 3/3 both chairs (+1626/+1957/+2354), partially discharging the
prep-book falsifier (only 30c7653b is GB-as-Blue).

Recommendation: (c) scripted low-close GB-mimic (support/gbmimic.rs,
run-then-connect, one ~100-area border rectangle bank ≥ply 45, rate 16.7–23.3%)
as h2h_gbmimic.rs opponent (10 games, v7base control arm, dose arm, per-chair
Δ), (a) counterfactual replay demoted to 30-min sanity pre-check, (b) Stompy =
leak control (never fires <25%; proves gate isn't "any opponent").
Fidelity precondition first (rate ≤0.22, tell fires ≥9/10, ≥50-area close in
≥3/10) — kills the MIMIC not the dose. Dose kill bars: mechanism null /
payoff null / harness void / control leak. Keep bar = blocked ≥2/3 AND margin
better (counterfactual-only, no Elo claim). GB identity stays with Phase 4
b1e7d2d1/f33e29fc. If tracker is site-write-only: park lane OFF.
