# V3 plan: much stronger search + the v1-vs-v2 mystery (2026-09-27)

## 0. What the linked bot is
`75b1cb0f` = **Riposte v2 (ours)**, rating 1437, 0-8 rated (0-4 vs v1 Riposte,
0-4 vs Great Barrier), plus a running VladNet match already 0-2 down
(`0e1f52df`: 1576-2312 as blue, 1860-2416 as red). V3 must beat v1 first —
losing to yourself 0-4 means the "improvements" broke something real.

## 1. The v1-vs-v2 mystery (solve FIRST, before any new search code)
Site v1-vs-v2 (`42525822`, forced openings [4864] and [5589]) was CLOSE:
1201-1109, 1109-1409, 1803-1577, 1183-1889 — v1 edged all four, none a
blowout. Local empty-opening mirror (`probe_v1v2`) splits 1-1 with BIGGER
margins (v2 blue loses 541, v2 red wins 251). So: v2 is fine in open water,
worse from forced openings. Hypotheses: (a) patience knob mishandles the
forced first action; (b) avoid-points from the forced line misroute early
development; (c) symmetric horizon overvalues answering the opening.
Jobs: Lane C autopsies site games `a3f328d1 e066f371 782c8615 acd3dcff`
(action-by-action: where does v2's line diverge from v1's, and what does
the eval say at the divergence?). I reproduce locally with forced openings
(extend `probe_v1v2` to take an opening move id — site used 4864, 5589).

## 2. Search core rewrite (Lane A executes, this is the spec)
Chess-programming techniques mapped to Meridian (branching ~437, 120
actions, 2 actions/turn, exact engine, 20s/move, hard kill 25s server /
~23s browser, wasm32-wasip1, deterministic tiebreaks):

1. **Negamax alpha-beta over actions** (not turns). Our eval is already a
   Blue-lead number — negate per side-to-move. Move ordering = current
   priority function (it already ranks well) + hash move + killers.
2. **Iterative deepening 1..N actions** with aspiration windows (fail-soft,
   re-search on fail-high/low). Never start an iteration you can't finish:
   predict cost by effective branching factor measured live.
3. **Transposition table** keyed by `Position` hash (it derives Hash + Eq;
   canonical form free). Store value/bound/depth/best move. Size for 256MiB
   cap: fixed pool (e.g. 2^20 entries), always-replace.
4. **Time management** (`limits.moveTimeMs` is currently IGNORED — the
   single biggest free gain): soft bound ~40% / hard bound ~80% of the
   limit, checked per completed iteration (soft) and per 1024 nodes (hard).
   Spend MORE on: first moves out of book/opening (Cray-Blitz factor
   2 - nMoves/10), fail-low iterations (panic extension, capped), positions
   where best move changed between iterations. Spend LESS (easy-move fast
   path): one legal move, or top-1 beating top-2 by >X in two consecutive
   iterations. Never exceed hard bound: browser kills ~23s, no increment.
5. **Quiescence**: at depth 0, extend volatile lines only (a cut is
   threatened/executed, a loop close is pending) with captures+closes move
   set, standing pat allowed. (Old JS quiesce was a wash WITHOUT alpha-beta;
   retest inside this framework, keep only on league evidence.)
6. **Selectivity**: extensions (single legal reply to a cut = forced,
   imminent big-loop close), reductions (deadwood/back moves searched
   shallower = late-move reductions), **null move via timeout**:
   `applyTimeout` is a legal game action with exact semantics (banks, flips
   turn) — use as R=1 null with verification search; if it fails, the
   position is too good to be true somewhere. Test, don't assume.
7. **Pondering**: not available (wasm is called per request). Skip.

## 3. Eval (Lane B, unchanged ownership)
Phase gates, vulnerability/mobility/potential terms, cut+make gap math.
Constraint: keep eval CHEAP (it runs at every node now, not 4k positions
per move — millions of calls; no enemy-movegen inside eval without budget).

## 4. Validation protocol (mandatory, all lanes)
- Local: `probe_league` (scoutbase) + `gauge` (greedy) + `probe_v1v2`
  (v1 mirror) — n=8 minimum, both colors, report avg margin per color.
- NEW: opening-seeded games. The site forces openings ([4864], [5589],
  [11723], [9199] seen); empty-opening results do NOT predict them (proof:
  v1-vs-v2). Extend league with the site's exact forced openings.
- Site Elo is the final judge. Version every upload (v3, v4…), never
  overwrite the best-so-far bot until the new file beats it in a rated
  match (v1 stays live as the baseline).

## 5. Stop-doing list
- No more copied opening books (rejected at -48pp; our search must EARN
  walls itself or not play them).
- No fixed 4096 budget (use the clock).
- No penalty in invented units (all terms must be in full-horizon points
  or they silently do nothing — proven twice now).
