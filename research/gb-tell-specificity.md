# GB-tell specificity (external session, 2026-09-30) — SPECIFIC

Q: is GB's 0.50x close-rate a GB signature or a winners-close-less confound?
Validated: 12/12 replays reproduce scores/areas/counts exactly; codec 120/120.

## Result

| pool | winner closes | loser closes | ratio |
|---|---|---|---|
| GB games (3) | 37/180 = 20.6% | 74/180 = 41.1% | **0.50x** |
| non-GB (9) | 198/540 = 36.7% | 177/540 = 32.8% | **1.12x (opposite sign)** |
| all 12 | 32.6% | 34.9% | no winner effect |

Fisher exact (winner-close in GB vs non-GB games): p = 2.6e-44, OR = 0.45.
Same-bot control: v7 loses all 12 yet closes 41.1% vs GB vs 32.8% vs non-GB —
the gap is GB closing less, not loser spam. GB has the lowest close rate of any
bot in the set (20.6% vs v6 43.0%, v7 34.9%, VladNet 30.8%, Stompy 26.7%).

## Verdict: SPECIFIC — code the tell as ABSOLUTE rate (~20%), not the ratio

The 0.5x ratio needs a known opponent; absolute ~20% does not. Threshold at the
GB band (~≤27%) admits Stompy (26.7%) — expect that false positive. Per-game
ratios overlap the non-GB low tail (Stompy 0.47, VladNet 0.67): population-level
discriminator, not a per-game classifier. n=3 GB games — consistent but thin.

## Routing

C3 close-rate is the only GB number worth coding. S2 redirect target: GB-tell
feature on absolute close rate. PREP-BOOK deny-the-border counter stays the
shape answer; this is the detection answer. Discriminator for the v7-counter
falsifier remains Phase 4 b1e7d2d1/f33e29fc.
