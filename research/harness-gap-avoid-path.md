# Harness gap — probes never exercise the deployed anti-rebuild path

Lane E verification of Lane C's blunder-catalog §3.6 ("Instrument gap: the league gate never
exercises the deployed anti-rebuild path"). Verdict: **CONFIRMED, by reading the sources.**
Patch plan below is a proposal only — **NOT applied** (gates are shared; main session decides).
No `retaliator/src/*` files were touched.

## Verdict (one line)

Every local probe gate measures `best_move` (≡ `best_move_with_avoid(&[])`, empty avoid), while the
deployed module always runs `replay()`'s cut-history collection and calls
`best_move_with_avoid(position, &avoid)` — so local gates validate a different bot than the one on
site.

## Evidence

### Deployed path (retaliator/src/lib.rs, identical on master and lane-c-autopsy)

- `reply()`, `"type": "move"` branch (lib.rs:30-34): calls
  `search::best_move_with_avoid(game.position(), &avoid)`.
- `replay()` (lib.rs:46-78): replays `start` + `moves`; collects `(action_index, mover, broken)`
  for every cut; then keeps the recent cuts of the **final mover's opponent** within
  `CUT_MEMORY = 6` actions and pushes `cut.origin()` + `cut.far()` into `avoid`.
- `search::best_move` (search.rs:149-151) is literally
  `best_move_with_avoid(position, &[])` — same search core, empty avoid.

### Local probes call best_move (no avoid)

Master `retaliator/examples/`: **16 files, 19 direct `search::best_move(` call sites**, and
**zero** examples call `answer()`/`replay()`:

- `probe_league.rs:8,25` · `gauge.rs:31` · `probe_match.rs:12` · `probe_openings.rs:8` ·
  `probe_collapse.rs (×2)` · `probe_traj.rs (×2)` · `probe_one.rs` · `probe_blueopen.rs` ·
  `probe_blueopen2.rs` · `probe_branch.rs` · `probe_breaks.rs` · `probe_breaks2.rs` ·
  `probe_budget.rs` · `probe_open.rs` · `probe_top5.rs` · `probe_v1v2.rs`
- `probe_v3all.rs:17` additionally passes `best_move` as a fn pointer (`wra(...)`) — still no avoid.
- Lane C's branch adds `probe_bluechair.rs:44,193` and `probe_balloon.rs:127,193` — also
  `search::best_move` only.

Partial coverage that does exist: `probe_one.rs` calls `analyze_with_avoid` directly with one
hand-built avoid point at a fixed position (dodge mechanics only — not the `replay()` collection,
not `best_move_with_avoid`).

### The one answer()-path test with cut history is environment-dependent and broken

`tests/protocol.rs::cut_history_still_answers_legally` (protocol.rs:81-105) is the only test that
exercises the deployed route via `answer()` with a cut-containing line — but it reads
`/tmp/opencode/game-f4b7f187-37e3-403d-a578-70f6cfd0cda1.json` (absolute tmp path, absent in a
fresh checkout). Measured 2026-09-28 on a clean worktree of master:

```
cargo test --release --test protocol
  cut_history_still_answers_legally ... FAILED
  panicked at tests/protocol.rs:85: called `Result::unwrap()` on an `Err` value:
  Os { code: 2, kind: NotFound, message: "No such file or directory" }
  test result: FAILED. 5 passed; 1 failed
```

### Live proof the deployed path itself is sound (2026-09-28)

The actual `.wasm` binary (master worktree build) was live-executed on a cut-history position
(harness kept outside the repo at `/tmp/wasm-live/exec.mjs`; Node host providing the 5
`wasi_snapshot_preview1` imports): line `blue D10-D13 | red P10-M10, M10-J10 | blue A10-A7, A7-A4 |
red <build>, <cut of D10-D13>` → `replay()` collected avoid = {D10, D13} and the module returned
`{"move": 2280}`, legal per `engine/engine.js` replay. Move IDs used: `[16058, 7767, 7764, 1254,
1197, 2707, 14940]` from start `[B:A10-D10 R:P10-S10]` (also usable as a test fixture). So the gap
is purely that **local gates don't measure the deployed machinery** — not that the machinery is
broken.

## Consequence (Lane C §3.6, restated)

League/self-play margins measured with `avoid = []` understate (or misstate) deployed behaviour:
the bot's only anti-farming machinery — cut-memory routing away from broken ground — never runs in
any local gate. Any rebuild-penalty or routing experiment must be gated with a harness that
reconstructs `avoid` the way `lib.rs::replay` does.

## Proposed minimal fix (PATCH PLAN — do NOT apply)

### Option 1 — route probes through the deployed `answer()` path (RECOMMENDED)

No src changes; zero divergence by construction (the probes exercise exactly the deployed route,
so the copy can never drift from `lib.rs`).

1. Add one shared helper, `retaliator/examples/support/answered.rs` (a `#[path]` module like
   `scoutbase.rs`), exposing:

   ```rust
   /// The deployed route: replay() cut collection + best_move_with_avoid, via answer().
   pub fn best_move_answered(start: &str, ids: &[usize], seed: u64) -> Option<Move> {
       let reply = retaliator::answer(&json!({
           "type": "move", "start": start, "moves": ids, "seed": seed,
       }));
       Move::from_index(reply["move"].as_u64()? as usize)
   }
   ```

2. In each gate probe's play loop, keep the played move-ID list alongside the game: probes already
   do `game.play(mv)` — add `ids.push(mv.index())` next to it (1 line), with
   `const START: &str = "[B:A10-D10 R:P10-S10]"` (all current gate probes start from
   `Game::new()`, i.e. the standard start; a probe starting mid-game would need that position's
   notation instead).
3. Swap `search::best_move(game.position())` → `answered::best_move_answered(START, &ids, seed)`
   (1 line per call site). For `probe_v3all.rs`, give `wra(...)` a closure over the helper instead
   of the fn pointer.
4. Files touched: `probe_league.rs`, `gauge.rs`, `probe_v3all.rs`, `probe_match.rs`,
   `probe_openings.rs` (master) + `probe_bluechair.rs`, `probe_balloon.rs` (lane-c-autopsy) + the
   new support module. Every other probe can migrate the same way opportunistically.

Cost: `answer()` re-replays the whole move list per request (O(n²) per game) — negligible at probe
lengths (≤ 120 moves); the move budget is unchanged (`MOVE_BUDGET`), so margins stay comparable.

### Option 2 — smaller diff, some drift risk

`retaliator/examples/support/avoidline.rs`: reconstruct `avoid` exactly like `lib.rs::replay`
(collect cuts with action index; keep recent cuts of the final mover's opponent within
`CUT_MEMORY = 6`; push origin + far) and expose `best_move_avoid(&Game)`. Probes swap
`best_move` → `avoidline::best_move_avoid`. Downside: duplicates `replay()`'s logic outside
`lib.rs` — it has already changed once (the avoid logic was added in v5), and a silent copy drift
re-opens the same gap.

### Option 3 — share one implementation (src change; needs main session sign-off)

Make the reconstruction reusable from the crate itself (e.g. `pub fn avoid_from_history(...)` or
a `pub replay()` in `lib.rs`) and have both `reply()` and the probes call it. Cleanest long-term;
touches `retaliator/src/*`, so it conflicts with the current "no src changes" discipline and needs
a dedicated lane.

### Independent one-line fix (either way)

`tests/protocol.rs::cut_history_still_answers_legally` must not depend on an absolute tmp path.
Either (a) replace the file read with a committed fixture line — today's live-proof line
`[16058, 7767, 7764, 1254, 1197, 2707, 14940]` from `[B:A10-D10 R:P10-S10]` (a real cut of
D10-D13, avoid non-empty) — or (b) make the test skip cleanly when the file is absent
(`match std::fs::read_to_string(...) { Ok(d) => { ... } Err(_) => return }`). Note the file-based
version also cannot run on other machines at all.

## Gate discipline (per §3.6)

Until Option 1/2/3 lands, treat every local league/self-play number as "avoid = []" behaviour and
say so in scoreboard rows; gate any rebuild-penalty experiment with the avoid-reconstructing
harness only.
