# V4 plan: win the fights that matter, stop playing dead moves (2026-09-27)

## Where V3 stands
Beats v2 7/10, loses v1 7/10, arena: "barely won". Eval tweaks are yielding
single points. V4 must fix STRUCTURAL blindness, not add weight #12.
Lanes A/B/C silent — this plan is executable by me Openerlane by lane,
smallest-verified-first. Every item ships only on league evidence.

## Symptom 1: fighting unbreakable new lines (area yes/no, nothing sticks)
Mechanism: fresh enemy edges are SHIELDED (untouchable for a turn), so
contesting a fresh wall burns actions for zero — and our break bonus only
rewards cuts that exist NOW, never the threat that matures next turn.
- **V4a. Fresh-wall avoidance (small, high confidence).** `Position` exposes
  `fresh_edges()`/`shielded_edges()`. Penalize non-breaking first actions
  landing within 2 of enemy fresh/shielded edges (same currency:
  × min(events,12)). Rationale: if you can't cut it this turn, build
  elsewhere; come back when the shield drops. Mirror of the existing
  contact rule, which only prices adjacency, not freshness.
- **V4b. Shield-expiry threats (medium, needs depth).** A cut that becomes
  legal next turn is worth planning this turn. Requires seeing 3+ actions
  (Lane A), OR a cheap eval proxy: bonus for moves after which an enemy
  edge becomes our cuttable target next action (compute: enemy edges that
  are shielded NOW but cuttable from our new node LATER — one movegen
  call, capped). If proxy works, depth not required.

## Symptom 2: useless moves (does nothing on any axis)
Mechanism: deadwood penalty covers only no-area CONNECTS far from enemy.
No-area non-breaking EXTENDS in dead zones are never penalized, and tiny
room gains (+0.1 hull) outscore doing nothing — so shuffling beats passing
and the bot shuffles.
- **V4c. Do-nothing filter (small).** Generalize: a first action with no
  area gain AND no break AND room gain < 0.5 AND not near enemy gets the
  deadwood penalty (both kinds). Everything a move can do is already
  measured in `ranked()` — this just prices "none of the above" below
  every real option. Watch: must exempt the only-legal-move case
  (fallback chain already handles it — verify in test).

## Symptom 3: clustering in one area
Mechanism: unknown — could be optimal (GB builds connected walls!) or
could be all-eggs-one-basket fragility. DO NOT "fix" blindly.
- **V4d. Measure first (tiny).** Probe: mean pairwise node distance +
  loop count per game, us vs GB/VladNet site games. If GB spreads wider
  per loop held, add a spread term (distance-to-own-centroid bonus, small,
  opening/midgame only). If GB clusters tighter, clustering is correct and
  the user's read is wrong — report and move on. One probe, one decision.

## V4d2. Unbreakable shapes (field rule)
Dense enemy clusters are cut-PROOF: any edge through them touches 2+
opponent edges, which is illegal (one cut per move max). Consequences:
(a) never spend actions "fighting" a cluster with no legal cut — build
elsewhere; (b) every threat/vulnerability term (ours and Lane B's) must
count only LEGAL cuts (via `check_move`/legal set), never geometric
touches, or the eval prices fantasy cuts; (c) our own mild crowding is
protective for the same reason — a second, defensive reading of the
clustering question in V4d.

## V4e. Structural (the actual Elo)
Lane A owns alpha-beta + ID + TT + time management per plan-v3 §2. If no
output by next session, I build it in `search_deep.rs` myself, starting
with: fixed-depth-3 alpha-beta with current priority as ordering (no ID,
no TT) to measure whether depth alone pays before investing in the rest.

## Outcomes (measured during build)
- REJECTED: fixed-depth-3 minimax (-58% league at 256-budget replies,
  -44% at full coverage). This eval is not minimax-stable; depth must come
  via Lane A's alpha-beta, not naive deepening.
- Doom discount (V4, selection-level): collapse line 1098 -> 1031 against;
  genuine games unchanged (both wins). Kept at 1.0 (0.5 tested worse).
- Catastrophic synthetic red line (-111%) persists across all variants:
  single 30+ area pop ~t=60 plus erosion. Needs Lane A depth or Lane B
  vulnerability, not more weights.
- Center-pull opening bias: REJECTED earlier (-16pp). Corner-first stands.

## Validation (unchanged, tightened)
- Gate 1 (local): `probe_v3all` (v1+v2 round robin) + `probe_league` +
  `gauge`. Ship threshold: beats v1 head-to-head (currently 3/10).
- Gate 2 (arena): versioned upload, rated match vs v1 first, then GB.
  v1 stays live until dethroned. Site Elo is the only judge that counts.

## Stop list (carried over)
No copied books, no fixed budgets, no invented-unit weights, no new eval
term without an ablation number (with/without, 8+ games).
