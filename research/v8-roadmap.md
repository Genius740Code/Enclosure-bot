# v8 roadmap — everything between here and ship (2026-09-29, master `441ec23`)

External-AI usage is exhausted; all external lanes stopped half-done. Our agents:
replies-Ultra running (`lane-v8-reply`), 1 Nvidia slot free. Cap stays max 2 Nvidia.

**Ship bar** (v8-plan §3): Elo > v7 over 20+ games; beats v7 mirrors both colors;
takes games off ≥2 rivals; no collapses. **Cut rule** (M4): stacked master beats v7
≥7/10 → WASM build → upload "Riposte v8" → 20 rated games.

## Phase 0 — bookkeeping (no engine, do first)

- [x] P dose-2 KILL row on master scoreboard (this commit)
- [ ] W dose KILL rows (done `441ec23`)
- [ ] File the idea backlog: the ideas-AI doc currently lives only in chat.
      Transcribe to `research/idea-backlog.md` on new branch `lane-v8-ideas`
      (ideas 1-5 + E-0–E-8 flags + §4/§5), push, never master.
- [ ] Codify E-6 in v8-plan §2: every gate judges each chair **against control at
      the same color** (fixed 3/5 split bar has ~10% pass rate for an equal bot).
      All running lanes must record dose-vs-control per-chair tables (P already does).
- [ ] Autopsy our-color re-run: b0ac4141, 1c69056c (30c7653b fixed `e1b82d3`).
      No lane may cite those two games until corrected.

## Phase 1 — shared infra (unblocks EVERY h2h gate; build once)

- [ ] `v7base` frozen-baseline harness: copy current opening-ON routed engine into
      `examples/support/v7base.rs` + 10-game alternating-color h2h runner with
      per-chair splits. Infrastructure, not a behavior variable. Consumers: P-dose-3
      gate, R dose-1 gate, all future doses. (Red-split AI's design is the spec.)

## Phase 2 — finish half-done lanes (one variable per dose, full gates)

- [ ] **P (rebuild-denial).** Dose 2 KILLED (byte-identical vs control). Gate dose 3
      (MEMORY 20) once — expect same (diagnostic plateau) — then close lane.
      Branch `origin/lane-v8-p`, worktree `game-v8p`. Successor hypothesis: POP-PRICE.
- [ ] **R (red-split).** Premise REFRAMED by E-1: both colors run 8-move mesh
      (`MESH_R_TXT` exists, `search.rs:94`) — the dose is blue-free vs red-scripted,
      not "make blue like red". Steps: v7base (Phase 1) → apply
      `BLUE_SCRIPTED_OPENING=false` (spec verified at `search.rs:157-172`) → build →
      h2h/league/gauge → push `lane-v8-r`. Worktree `game-v8rt` clean at `1741965`.
- [ ] **S2 (opponent replies, #1 defect).** Ultra phase-1 running: archetype profiles
      + weighted-vs-greedy match % per archetype. If >55% overall and every archetype
      up → wire dose behind toggle (default OFF, verify OFF-identity) → full gates.
      M2 chess-tech port stays BLOCKED until this lands (M7 verdict).
- [ ] **M10 (MCTS pilot).** External session at ~75% (playout 2,240/s native, 1.4k
      sims/s MCTS; variants 22/28; pilot written unrun). Resume: aggregate variants →
      run pilot on M7's 20 positions (reconstructible from `research/games/3a414aad`
      on `origin/lane-v8-autopsy` + `flip_test.rs` on `origin/lane-v8-audit`) →
      write `m10-mcts-pilot.md` → push `lane-v8-mcts`. WASM ratio: use 3x, no build
      needed. Verdict pilot-or-kill at ~2.2k sims (prior ~60% kill).
- [ ] **Rivals.** Build `research/rival-playbooks.md` from local JSONs only
      (`git show origin/lane-v8-autopsy:research/games/<id>-*.json`, 5 games:
      3a414aad, 1c69056c, d2d4b4fd, ad65f054, 30c7653b; b1e7d2d1 is queued-not-played)
      → push `lane-v8-rivals`. No network needed.

## Phase 3 — idea-backlog Step-0s (no engine change; cheapest first)

All five ideas start with a $0 probe; only survivors get doses.
- [ ] **POP-PRICE** (idea #1, +35): extend `flip_test.rs` into decision-node ledger
      at 3a414aad actions 56,57,64,80,84,88,89. Kill-0: re-close beats best
      alternative by more than tail+pair+rebuild explain. Then D1 `DOOM_TAIL_W`,
      D2 `DOOM_PAIR_W`, D3 `REBUILD_K`.
- [ ] **BEAM-SEED** (idea #2, +30): census — screen-argmax outside priority-top-8
      rate on a8e03a5f/ad65f054/d2d4b4fd/30c7653b/3a414aad ranges. Kill-0: <10%.
      Cost gate: median WASM move ≤2.0s, max ≤4.5s (Oct-4 forfeit rule).
- [ ] **TIE-SYM** (idea #4, +15): tie census (<1e-9 / <0.5pt share, direction
      histogram by color). Kill-0: tie share <5%.
- [ ] **PREFIX-ABORT** (idea #5, +10): tabulate foe-area at prefix index 4/6 over
      17 games. Kill-0: fires in ≥50% of wins or <2 of 12 losses.
- [ ] **PREP-BOOK** (idea #3, +20): ghost harness + offline counter-lines vs GB
      scripts. Needs GB site games (b1e7d2d1/f33e29fc — Phase 4 evals). Last.

## Phase 4 — ship path (blocked on site evals, not agents)

- [ ] 16 queued rated evals finish (v3, Stompy, VladNet, AngelBot-WASM, GB b1e7d2d1,
      GB2.0 f33e29fc, capybara-v5, xmybot + user-run v6 mirror) → launch **points
      agent** (full autopsy + rival files + 6j A1-A5 analyses).
- [ ] Merge gated winners to master ONE at a time, re-gating after each merge
      (stacked interactions). Never merge KILLed or ungated doses.
- [ ] M4 bar → `bench_timed` + `site_check` (12/12 legal) → WASM build → upload v2
      (audit fixes ride along, same picks) → 20 rated games → **ship Riposte v8**.

## Killed — do not retry without new evidence

contest-gen 0/32 · Lane E all-KILL · Q1a/Q1b · Q2/Q3b/Q5/Q8/Q11/Q12 · W doses 1-3
(league -27/-30%) · P dose 2 (byte-identical) · GB-book copy (-48pp) · M2 port
(blocked on S2) · symmetry pruning (no Elo while depth is theater) · Cluster-3
engine work (measure first) · red-opener design (premise false, E-1).

## Lane → branch → worktree index

| Lane | Branch (origin) | Worktree | State |
|------|-----------------|----------|-------|
| P rebuild | `lane-v8-p` | `game-v8p` | dose 2 KILL; dose 3 to gate |
| W wall | `lane-v8-w` | `game-v8w` | 3 doses KILL, closed |
| S2 replies | `lane-v8-reply` | `game-v8r` | phase-1 running (Ultra) |
| R split | `lane-v8-r` | `game-v8rt` | spec ready, needs v7base |
| M10 MCTS | `lane-v8-mcts` | — (external container) | 75%, needs resume |
| Rivals | `lane-v8-rivals` | — (external container) | data ready, needs author |
| Ideas | `lane-v8-ideas` | — | doc in chat, needs filing |
| Autopsy/data | `lane-v8-autopsy` | `game-v8c` | done; hosts 12 game JSONs |
| Audit/M7 | `lane-v8-audit` | `game-v8a` | done; hosts flip_test |

Suggested agent order under the 2-Nvidia cap: (1) v7base harness → (2) POP-PRICE
ledger → (3) R dose-1 → (4) BEAM-SEED census → (5) M10 resume → (6) rivals author.
