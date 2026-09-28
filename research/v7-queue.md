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

## Q4 (USER 2026-09-28): never concede remote space — one far line becomes an unbankable bank
"Bad to give them space: even 1 line far from their main lines is risky —
if they place it, it's never break and causes problems in the long run."
- Mechanism: a remote foothold faces no contest, thickens unchallenged into
  2+ touch walls, then banks every event forever. Early contest costs a tempo;
  late contest is impossible (nothing to cut).
- Mirror image of shipped REMOTE_BONUS (we go far when heat is high). The
  missing half: DENY_THEIRS — price enemy far-from-fight expansion as a
  threat proportional to its unbreakability (shared-node count), not just its
  current area (which reads ~0 while it is being built — exactly when it is
  cheapest to kill).
- Routed to: Lane E, with Q1 (end-state valuation: a remote bank scores at
  EVERY event, so its full-horizon value is maximal — the two ideas compose).
- Falsifiable: contest-remote dose sweep must raise enemy-area-lifetime gap
  (theirs falls) without tanking our own build; if early contests just donate
  targets (neck/aggro lesson), CONTACT_PENALTY interplay decides and we kill it.

## Q2-amendment (USER 2026-09-28): MANY different shapes — never pattern-match one
Founder correction: unbreakable comes in many shape families (corridors,
blobs, thickets, nested loops...), not just the double-wall corridor.
- Consequence for Q2/Q3: detection AND building bonuses must key off
  STRUCTURAL properties (cut-touch counts, shared-node ratios, wall thickness
  per unit area), never off named patterns. A term that only fires on
  corridors is a bug, not a feature.
- Required coverage: validate any shape term across GB + capybara + AngelBot
  + blob-mimic lines. If it helps vs one family and hurts vs others, it fails
  generality and dies (no family-specific carve-outs without their own gates).

## Q5 (USER 2026-09-28): connect timing — space first, close late (Angel lesson)
"Angel wastes 3 turns then connects big; early you have time; Angel took
loads of space and waited to connect. Stop early connects that could wait
50 moves — space-grabbing matters more."
- Tension with V5 ablation (patience term "never flips a pick", kept at 2.0/12):
  user OBSERVES early connects. Either they happen after act 12, or the gain
  threshold misfires. Re-open with user data, not the old verdict.
- Routed to: Lane E. Dose: PATIENCE window/gain + delayed-close bonus scaled
  by open-space available (close late ONLY while space remains). Metric:
  mean close-action per game must shift later AND banked area must not fall.
- Falsifiable: if later closes lose area to snipers, revert to 2.0/12.

## Q6 (USER 2026-09-28, DIRECTIVE): think budget raised — up to ~2s, position-dependent
"Avg move 33ms — it can spend more thinking. Doesn't have to be instant;
longer depending on position."
- Site envelope: 5s requested, 20-25s lease (hard forfeit after Oct 4).
  v7 budget: soft cap ~2s, POSITION-DEPENDENT (spend on volatile/open lines,
  snap-reply quiet shuffles). No more 33ms guilt.
- Routed to: Lane S (owns the budget/deepening machinery).

## Q7 (USER 2026-09-28): math-ify it — line-set area eval, break taxonomy, keep-alive line
(a) "Group-caught moves: sets of moves evaluated together for optimal area
    by 2-6 lines" — evaluate small move-SETS (not single moves) for area
    yield; a lot of this can be closed-form math, not search.
(b) "Some types of moves break area" — build the BREAK TAXONOMY (neck cut,
    loop pop, corridor sever, bank raid...) with per-type frequency x damage
    from autopsies; price/avoid per type, not one flat break term.
(c) "Keep one line alive at the opposite side in the middle — when we move
    they can re-break the line" — a mid-board opposite-side presence line,
    maintained (repaired) as a standing threat/anchor, not abandoned.
- Routed to: (a) Lane S (math inside search), (b) Lane C2 census then Lane E
  pricing, (c) Lane E as presence-line bonus dose.

## Q8 (USER 2026-09-28): some lines are STRONGER than others — rank them
Not all our lines deserve equal defense. Rank live lines by strength
(structural: thickness, touches, bank-rate, repair cost) and: reinforce the
strong, abandon the weak early (don't good-money-after-bad a doomed line).
- Gated by: does strength-ranking predict which lines survive? (C2 first,
  then E prices it.) Kills the sunk-cost rebuilds that farmers exploit.
