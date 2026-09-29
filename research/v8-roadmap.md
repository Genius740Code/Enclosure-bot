# v8 roadmap — everything between here and ship (2026-09-29, master `441ec23`)

External-AI usage is exhausted; all external lanes stopped half-done. Our agents:
POP-PRICE D1 running (Nvidia, `lane-v8-pop`), tie census (OpenCode, `lane-v8-tie`),
S2 handoff sent (backgrounded). Cap: 1 Nvidia + 1 OpenCode at a time.

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

## Phase 1 — shared infra (UNBLOCKED 2026-09-29, merged `8ac25c8`)

- [x] `v7base` frozen-baseline harness (`examples/support/v7base.rs` + `h2h_base.rs`):
      identity 20/20, gauge 6/6, self-h2h Blue 0/5 Red 5/5 (color-decided mirrors).
      Merged to master (examples-only, zero engine delta). All lanes use it.

## Phase 2 — finish half-done lanes (one variable per dose, full gates)

- [ ] **P (rebuild-denial).** Dose 2 KILLED (byte-identical vs control). Gate dose 3
      (MEMORY 20) once — expect same (diagnostic plateau) — then close lane.
      Branch `origin/lane-v8-p`, worktree `game-v8p`. Successor hypothesis: POP-PRICE.
- [ ] **R (red-split).** Premise REFRAMED by E-1: both colors run 8-move mesh
      (`MESH_R_TXT` exists, `search.rs:94`) — the dose is blue-free vs red-scripted,
      not "make blue like red". Steps: apply `BLUE_SCRIPTED_OPENING=false`
      (spec verified at `search.rs:157-172`) → build → h2h (in-tree runner) /
      league / gauge → push `lane-v8-r`. Worktree `game-v8rt` clean at `1741965`
      (needs master merge first).
- [ ] **S2 (opponent replies, #1 defect).** Phase-1 running + VladNet spec handed off
      (`research/archetype-vladnet.md` on `origin/lane-v8-rivals`, commit `e2b10e8`:
      94.1% lag 1-2 predictor pseudo-spec, validate on held-out 1c69056c since
      queued eval 9f06cd59 is unplayed). Then GB archetype. Wire dose behind toggle
      (default OFF, verify OFF-identity) if >55% overall and every archetype up.
      M2 port stays BLOCKED.
- [ ] **M10 (MCTS pilot).** External session at ~75% (playout 2,240/s native, 1.4k
      sims/s MCTS; variants 22/28; pilot written unrun). Resume: aggregate variants →
      run pilot on M7's 20 positions (reconstructible from `research/games/3a414aad`
      on `origin/lane-v8-autopsy` + `flip_test.rs` on `origin/lane-v8-audit`) →
      write `m10-mcts-pilot.md` → push `lane-v8-mcts`. WASM ratio: use 3x, no build
      needed. Verdict pilot-or-kill at ~2.2k sims (prior ~60% kill).
- [x] **Rivals.** `research/rival-playbooks.md` + miner (`bdb5b0e`), habit-2
      replication (`3d6e366`: 32/34 lag 1-2, -314.0/34 confirmed), VladNet archetype
      + predictor spec (`e2b10e8`), all on `origin/lane-v8-rivals`. Remaining: replicate
      one GB habit before lanes cite GB figures; GB playbook figures otherwise provisional.

## Phase 3 — idea-backlog Step-0s (no engine change; cheapest first)

All five ideas start with a $0 probe; only survivors get doses.
- [ ] **POP-PRICE** (idea #1, +35): kill-0 PASSED 6/7 (`poprice-ledger.md` on
      `origin/lane-v8-rivals`) + BEAM-SEED premise confirmed (alternatives rank
      10-106). **D1 (`DOOM_TAIL_W` 0→0.5) RUNNING** on `lane-v8-pop`. Then D2
      (`DOOM_PAIR_W`), D3 (`REBUILD_K`) only if ledger order says so.
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
| POP (`DOOM_TAIL_W`) | `lane-v8-pop` | `game-v8pop` | D1 running (Nvidia) |
| S2 replies | `lane-v8-reply` | `game-v8r` | phase-1 + VladNet spec handoff (bkgd) |
| Tie census | `lane-v8-tie` | `game-v8t` | Step-0 running (OpenCode) |
| R split | `lane-v8-r` | `game-v8rt` | spec ready, needs master merge |
| M10 MCTS | `lane-v8-mcts` | — (external container) | 75%, needs resume |
| Rivals | `lane-v8-rivals` | — (external container) | data ready, needs author |
| Ideas | `lane-v8-ideas` | — | doc in chat, needs filing |
| Autopsy/data | `lane-v8-autopsy` | `game-v8c` | done; hosts 12 game JSONs |
| Audit/M7 | `lane-v8-audit` | `game-v8a` | done; hosts flip_test |

Suggested agent order under the 1+1 cap: POP-D1 (running) → tie verdict →
R dose-1 → BEAM-SEED census → M10 resume → GB-habit replication.
