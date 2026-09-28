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

## Q9 (USER 2026-09-28, DIRECTIVE): multi-turn shape investments + big-picture horizon
"C2 should aim for bigger picture: even 4-5 turns of setup can build a big
shape or help a lot. Bot may be too short-term — partly the 33ms calculation
time, which will be fixed."
- Two coupled fixes: (a) ANALYSIS horizon (C2: judge plans over 4-5 turn
  arcs, not single moves — a setup that pays on turn 5 is good, not slow);
  (b) THINK horizon (S: Q6 ~2s budget must BUY multi-turn shape lines, not
  just deeper 1-ply ranking — extensions must reach the shape payoff).
- Test: positions where the best 1-ply move and the best 5-turn plan differ;
  v7 must pick the plan. If longer thought just ranks the same move higher,
  the budget is wasted and we cut it back.

## Q10 (USER 2026-09-28): color asymmetry — some bots only exist as one color
"Stompy/Scout win all as red, lose all as blue."
- If a rival's strength is color-conditional, our prep should be too:
  separate responses per color (different prefix handling, different risk
  posture), and exploit their weak color deliberately rather than playing
  both colors the same way.
- Routed to: Lane C2 first — color-split W/L for every tracked rival
  (Stompy, Scout, GB family, VladNet, AngelBot, xmybot, Atlas, john.fun).
  Any rival with >=75% of wins in one color gets a color-specific prep note;
  lanes E/S implement only where C2's split clears significance (n>=8 per
  color, no small-sample theater).
- Falsifiable: if splits regress to 50/50 with sample size, delete the note.

## Q11 (USER 2026-09-28): prevent-the-wall — contest enemy big-close BEFORE it shuts
Exhibit A: game adfb6cf5 (v6 blue vs xmybot red, LOSS). Moves 10-60 we lead
everything (area 73-8, score 1018-131). Moves 60-70 xmybot closes ONE
114-area wall (8->122) that never breaks again; banks 131->5484 and wins.
During construction our moves were +3/+9 pottering connects and zero-contest
Extends — we never touched their building site while our eval peaked.
Full log: /tmp/opencode/aut-wall.log (also: our 6 breaks vs their 15).
- Mechanism needed: closure-progress trigger — when the enemy's live-area
  growth / frontier-pair count says a big close is 2-4 turns out, STOP
  banking small stuff and contest the corridor root. After it shuts,
  Q2-shapes say it is unbreakable: the ONLY winning window is before.
- Composes with Q4 (deny early) + Q1 (their bank scores EVERY event after).
- Falsifiable: prevention attempts must convert (wall never closes) at a
  rate beating the tempo cost; if contests just donate necks, kill it.

## Q12 (USER 2026-09-28): reinforce vs prevent-reinforce on thick lines
"Sometimes bot should reinforce its lines OR deny opponent reinforcement,
because the line is already impossible to break (2+ touches = no legal cut)."

- Tactical corollary of Q2/Q4: when a line already has 2+ touches (unbreakable
  by single cut), the decision is:
  - FOR US: reinforce (add thickness, extend ends) — we WANT this line to
    bank every event.
  - VS OPP: contest their reinforcement (early contest of the corridor
    root, pre-build to deny the second wall) — we want to STOP them from
    reaching unbreakable thickness.
- Already partially in shipped code:
  - DENSE_BONUS (bonus for 2+ own-node adjacency on first action)
  - REBUILD_PENALTY / CUT_RADIUS (avoid re-closing near recent cuts)
  - But no explicit "line is thick → reinforce; opponent thickens → contest"
    evaluation term.
- Proposed: eval term that detects `thick_lines_us` (shared-node count ≥ 2
  per wall segment) and `thick_lines_opp`, then:
  - +bonus for our moves that extend/anchor thick lines
  - +bonus for our moves that contest opponent's near-thick lines
    (frontier pair 1 move from 2+ touches)
- Gate: area lifetime rises (our thick lines bank longer) AND opponent's
  thick-line completion rate drops.
- Composes with Q2 (build), Q4 (deny), Q11 (prevent close).

## Q13–Q17 (USER 2026-09-28): Chess programming techniques adapted for Enclosure

### Q13: Transposition Table Expansion (beyond Lane A's depth-1)
Lane A has depth-1 exact entries + best-move ordering. Full TT would store:
- **Depth-preferred** (replace if new depth ≥ stored depth, or always-replace for cut-nodes)
- **Bound types** (EXACT, LOWER, UPPER) for correct cutoff logic
- **Aging/bucket eviction** instead of Lane A's simple overwrite
- **PV-node storage** (principal variation reconstruction)
Gate: TT hit rate > 40% on mid/late game, value-exact vs no-TT.

### Q14: Killer Moves + History Heuristic (move ordering beyond XBot)
XBot orders captures > cuts > closes by static priority. Chess adds:
- **Killer moves** (2 per ply: moves that caused beta-cutoffs at same depth)
- **History heuristic** (move -> score table, updated on cutoffs)
- **Countermoves** (response to opponent's last move)
- **Capture history** (victim/attacker pair tables)
Combined with XBot static order: dynamic > static. Gate: nodes/position drops > 20% vs XBot alone.

### Q15: Late Move Reductions (LMR) + Probcut
After ordering, reduce depth for late moves (assumed bad):
- **LMR**: reduce depth by 1–2 for moves 4+ after first few full-width
- **Probcut**: at high depths, probe with reduced window before full search
Risky for Enclosure (no "pass"), but captures/breaks likely safe to keep full.
Gate: nodes/position drops > 15% with zero value change on bench 596.

### Q16: Quiescence Search (critical for Enclosure)
v6 stops at fixed 2-ply. Chess quiescence: extend volatile lines until "quiet":
- Extend on: captures, breaks, scoring events (area gain/loss), pending banks
- Stop on: quiet extends, shuffles, remote builds
This IS Q6+Q9 combined — the "selective depth" IS quiescence.
Gate: no horizon blunders (missed close / missed break) on 596 bench.

### Q17: Principal Variation Search (PVS) / NegaScout
Instead of full alpha-beta on all moves:
- First move: full window
- Subsequent moves: null-window (beta = alpha+1) probe
- If probe fails high, re-search full window
Typically 10% faster than standard alpha-beta with same result.
Gate: nodes/position drops > 10% vs standard alpha-beta, value-exact.

---

**Already in v6/v7 queue:**
- Iterative Deepening (v6, Lane A) → Q6
- Aspiration Windows (v7 Q6) → Q6
- Move Ordering (XBot captures>cuts>closes) → Q14
- Selective Depth / Quiescence (Q6+Q9) → Q16
- Mesh8 Opening Book (v6) → opening book
- Time Management (Q6) → time management

**Not applicable to Enclosure:**
- Null Move Pruning (no pass move)
- SMP/Parallel (single-threaded WASM)
- Tablebases / Syzygy (no endgame DB)
- Pondering (no opponent clock)
- Syzygy/EGTB (no endgame DB)

---

**Priority for v7 (beyond Q1–Q12):**
1. **Q16 (Quiescence)** — already covered by Q6+Q9, highest impact
2. **Q14 (Killer/History)** — low risk, high reward for move ordering
3. **Q13 (Full TT)** — Lane A has skeleton; expand if LMR needs it
4. **Q15 (LMR/Probcut)** — only after Q14 stable
5. **Q17 (PVS)** — marginal gain, last

Each would be a separate dose-swept gate (nodes/position, value-exact, gauge/h2h).

## Q18 (USER 2026-09-28): Symmetry pruning in perfectly symmetric positions
"In a perfectly symmetric position, a move left or right is the same — no need to calculate twice."

- Enclosure board (19×19) has rotational and reflection symmetries. At game start and many early/mid positions, the position is perfectly symmetric under 180° rotation (Blue ↔ Red swap + board flip).
- In symmetric positions: moves related by symmetry are **value-equivalent**. Evaluating one suffices; the other gets the same score by symmetry.
- Implementation: detect symmetry class of position (full symmetry, rotational only, reflection only, none). For each symmetry orbit of moves, evaluate **one representative**, assign its score to all symmetric siblings.
- Savings: up to 2–4× fewer evaluations in symmetric phases (early game, mirrored mid-game). Zero cost in asymmetric positions.
- Must respect deterministic tiebreak: if symmetric moves tie, pick by canonical ordering (lowest move id) — already the rule.
- Composes with move ordering: only order/evaluate representatives; expand scores to siblings after.
- Gate: nodes/position drops in symmetric positions with zero value change on bench 596.
- Composes with Q14 (move ordering): representatives ordered by XBot priority, then symmetry-expanded.

## Q19 (USER 2026-09-28): Confidence-based play / adaptive risk
"When eval variance across legal moves is high, play safer (wider margin); when low, play normally."

- Metric: variance/range of top-N move evaluations (after ordering). High variance = sharp tactical fork or trap nearby; low variance = many similar moves (quiet position).
- Behavior:
  - High variance: widen aspiration window, prefer moves with higher *minimum* score (maximin), extend search on top moves.
  - Low variance: narrow aspiration, trust ordering, snap-reply.
- Composes with Q6 (budget) + Q16 (quiescence): spend extra time only when variance is high AND it matters.
- Gate: reduces catastrophic blunders (large swing losses) without lowering average margin.

## Q6-amendment (USER 2026-09-28): No snap-reply — spend the budget on EVERY move
"v7 should NOT snap-reply at 30ms on quiet moves. It should spend the full ~2s on EVERY move, calculating more lines, deeper variations, more strategic alternatives. Not scared of using time."

- Revised Q6: **uniform base depth increase + selective extensions**.
  - Base search: iterative deepening to max depth within ~2s budget (not 2-ply base).
  - Quiet positions: still search deeper (strategic maneuvering, prophylaxis, space control).
  - Critical lines: selective extensions on top (Q16 quiescence).
- Rationale: "quiet" in Enclosure often means "space race" or "prophylaxis" — deeper search finds better space control, better prophylactic moves, better long-term shape building.
- Time allocation: iterative deepening with hard ~2s cap; if depth N completes in <2s, start depth N+1.
- Gate: avg move time > 1.5s (not 33ms); gauge/h2h must improve vs 33ms baseline.
- Composes with Q15 (LMR) to keep deeper search affordable, Q13 (TT) to avoid re-search.
