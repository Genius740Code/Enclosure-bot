# v7 work queue (founder-fed ideas + lane routing)

## Q1 (USER 2026-09-28): break-fix cycles must be valued by END-STATE, not transients
"They break a lot of ur area and u fix, they break and so on — what matters
at the end is if ur area is THERE or broken."
- Mechanism: enemy breaks our loop, we rebuild, they re-break. Current evals
  (doom discount, rebuild penalty) charge EVERY break as near-full loss, even
  when the area scores at every scoring event anyway.
- Suspected mispricing: N break-fix cycles on area that never misses a payout
  should cost ~0, not N x pop-value. V5 doom term charged x12 for a re-make
  that only misses 2 events (~6x overcharge, documented in search.rs).
- Routed to: Lane E, after unbreakable-shape building. Falsifiable metric:
  banked-score-per-built-area across a game must rise while break-count stays
  flat (i.e. we stop panic-pricing repaired ground).
- Ablation: end-state-weighted area value vs current per-event charging,
  one variable, full-horizon points, standard gates.

## Q2 (USER 2026-09-28): build unbreakable shapes (screenshot: double-wall corridor)
Long shared-node walls (2+ touches = no legal cut). V4 avoids cutting there;
v7 must BUILD there. Routed to: Lane E job one (own-shape bonus, metric: our
area lifetime 30.7 -> parity with rivals ~49.8); Lane O second half
(what cracks them when rivals build them); Lane C2 census (who builds them,
do they win).
