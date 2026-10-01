# REINFORCE-LINES kill-0 — GO (proceed to dose design)

Question: do buildings behind 2+ shared-node walls survive (unbreakable-first
premise)? Census: every wall/extend action by our bot landing next to 2+
own nodes (Chebyshev <= 1, mirroring `near_own_count`/DENSE in
`search.rs:436`), credited survived iff no later cut touches its nodes.
Split by game outcome (winner = higher final score) and chair.

Harness: `retaliator/examples/reinf_census.rs` (NEW file, no src changes).
Run: `cd retaliator && cargo run --release --example reinf_census -- <corpus dir>`
Log: `retaliator/gate-logs/reinf-census.log` (force-added, gate-logs/ is gitignored).

## Results (46 games, deterministic replay)

| chair | losses: survived/builds | wins: survived/builds | gap |
|-------|------------------------|----------------------|-----|
| Blue  | 345/436 = 79.1%        | 513/542 = 94.6%      | +15.5pp |
| Red   | 44/154 = 28.6%          | 465/486 = 95.7%      | +67.1pp |
| pooled| —                      | 1367/1618 = 84.5%    | — |

Notable lines: neck holds survival to 0–30% in losses (chall-neck-blue
0/30, R-neck-red 3/35); junk/random/turtle/sandbag/sprawl sit at ~100%
either way (opponents that never cut); blob splits by chair.

## Verdict: GO

Proposed kill-0 bar (met): survival share in wins >= 90% AND win-loss
gap >= 10pp in BOTH chairs. Measured 94.6%/95.7% and +15.5/+67.1pp.
Unbreakable building clearly associates with winning from both chairs —
the premise the dose needs. Next: design the DENSE_BONUS extension dose
(build behind 2+ shared-node walls, then expand far / bank behind) with
its own gated D1.
