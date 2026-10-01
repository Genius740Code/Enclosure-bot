# CUT-YIELD kill-0 — GO (proceed to dose + gates)

Dose: ZERO_CUT_PENALTY=1.0 x hz on broken + 0-destroyed + 0-gain first
actions — a first-action cut that breaks an edge but destroys zero area
and gains zero area loses 1.0 * hz priority inside `ranked()`
(`search.rs:310`), alongside the existing first-action terms.

Harness: `retaliator/examples/cut_census.rs` (NEW file, no src changes;
frozen verbatim `ranked()` + a penalty-carrying copy `ranked_zcp` built
by inserting the 6-line penalty after the deadwood block). At every
our-to-move root in the 46 local corpus games, compares the unpenalized
pick vs the penalized pick; a flip is counted when the unpenalized pick
is itself a zero-yield cut and the penalty changes the pick.
Run: `cd retaliator && cargo run --release --example cut_census -- <corpus dir>`
Log: `retaliator/gate-logs/cut-kill0.log` (force-added, gate-logs/ is gitignored).

## Results (46 games, 2760 our-roots, deterministic)

| chair | zero-yield-cut picks | flips | flip fraction | roots |
|-------|---------------------|-------|---------------|-------|
| Blue  | 45                  | 29    | 64.4%         | 1380  |
| Red   | 17                  | 9     | 52.9%         | 1380  |
| pooled| 62                  | 38    | **61.3%**     | 2760  |

Bar: flip census >= 10% of zero-yield cuts. Measured 61.3% — passes with
large margin in BOTH chairs. (Denominator is 62 picks, below the roadmap's
~113 estimate, but the fraction is the bar and it clears by 6x.)

## Verdict: GO

Proceed: implement ZERO_CUT_PENALTY behind a default-OFF toggle in
`search.rs` only, verify OFF-identity (byte-identical picks vs control),
then full gates (gauge_mesh 6/6, h2h_base per-chair E-6 vs v7base,
league_mesh AVG > -9.9% with no row < -300%). No Lane-E disease here —
the penalty flips picks in the majority of zero-yield-cut roots.
