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

## Q3 (USER 2026-09-28): Great Barrier's unbreakable-area game — detect, counter, steal
"GB makes moves that get loads of area that isn't breakable. Bot should find
if GB is doing it, counter it, and use it itself sometimes."
- Routed three ways:
  (a) DETECT (Lane C2): census GB's site games (incl. the v6-vs-GB/GB2.0 rated
      games now queued) for unbreakable-share: what fraction of GB's banked
      area sits in 2+ touch walls? Detection spec: a per-game unbreakable-share
      number C2 reports for every GB game, so counters can key off it.
  (b) COUNTER (Lane O): once the shape is characterized, test counters —
      early contest of the corridor root, pre-building to deny the second
      wall, price-denial of GB's bank race. One variable each; collapser work
      first, this second.
  (c) STEAL (Lane E): same as Q2 — adopt the shape for ourselves when the
      board offers it ("sometimes": gate the bonus on build-progress/tempo so
      we don't force corridors into bad ground).
- Falsifiable: if GB's unbreakable-share doesn't predict GB wins, the
  detection spec is wrong and we kill this line.
