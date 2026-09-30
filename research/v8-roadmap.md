# v8 roadmap — everything between here and ship (2026-09-30, master `fd2ff63`)

**Strength delta over v7: +0 Elo.** Nothing engine merged. Only docs + examples-only
harness on master. v7 stays the rated bot until a gated stack passes M4.

**Ship bar** (v8-plan §3): Elo > v7 over 20+ games; beats v7 mirrors both colors;
takes games off ≥2 rivals; no collapses. **Cut rule** (M4): stacked master beats v7
≥7/10 → WASM build → upload "Riposte v8" → 20 rated games.

**Resourcing:** max 2 Nvidia subagents + max 2 OpenCode subagents (nemotron=brain,
space-bunny=speed) + 1 OpenRouter (smallest task) + external Claude boxes via
paste prompts (read-only, md paste-back, no push).
429/overload/503 = transient: sleep 180s, retry same call only, x999. Re-seat an
agent only if zero on-disk output for 30+ min. Verify every finish on-disk
(commits pushed, numbers real) before feeding next task. E-6: every gate judges
each chair vs control at same color; record dose-vs-control per-chair tables.

## Phase 0 — bookkeeping (no engine)

- [x] P dose-2 KILL row on scoreboard (`c6adda7`)
- [x] W dose KILL rows (`441ec23`)
- [ ] File idea backlog: ideas doc still only in chat. Transcribe to
  `research/idea-backlog.md` on `lane-v8-ideas` (ideas 1-5 + E-0–E-8 + §4/§5).
  Push, never master.
- [x] Codify E-6 in v8-plan §2 (`3022520` on `lane-v8-ideas`).
- [x] Autopsy our-color re-run: b0ac4141, 1c69056c (`d6a4951`, both CITABLE with
  corrected arcs; two figures later corrected `c686181`). `rival-vladnet.md` is
  CLEAN after 4 rounds (`c784d4e`, `7afdaef`, `c0099c0`, `540b7b1` — 11 lines
  fixed, 1 false alarm caught by verification).
- [x] POP scoreboard row + jsonl (`7418b33`, `5687ecf`) — PRELIMINARY/NEUTRAL,
  figures unbacked; superseded by clean-base re-gate below.

## Phase 1 — shared infra [DONE]

- [x] `v7base` frozen baseline + `h2h_base` runner, merged `8ac25c8`
  (examples-only, zero engine delta). Identity 20/20, gauge 6/6, self-h2h Blue
  0/5 Red 5/5 (color-decided mirrors — the baseline every dose is judged against).

## Phase 2 — lanes (one variable per dose, full gates)

- [ ] **POP-PRICE (idea #1, +35 Elo est).** D1 = `DOOM_TAIL_W` 0→0.5. Dose SECURED
  on clean base: `5b745f0` on `lane-v8-pop-r2` (diff vs old `af35f50` EMPTY =
  identical dose; old branch lineage was docs-only junk, never use
  `origin/lane-v8-pop` for code). Part 1 reported gauge PASS, league
  +37.1%/−11.2% PASS, h2h B0/5 R5/5 (= control, NEUTRAL) — all UNBACKED (no logs
  committed). Part 2 RUNNING: full gates + raw logs + real-figures row on
  `lane-v8-pop-r2`. NEXT: founder stack-or-kill on backed numbers; D2
  (`DOOM_PAIR_W`) only if stacking, else close lane.
- [ ] **TIE-SYM (idea #4, +15).** Census LIVES (`8168ea2`). D1 IMPLEMENTED on
  clean base: `7d0d80e` on `lane-v8-tiesym-d1-r2` — code verified spec-exact (two
  comparators, gap<1e-9, center-then-index, deterministic). Effect NEUTRAL (zero
  pick-flips in gates). Figures UNBACKED (no logs). Merge-blocker nit:
  `mv.target().expect(...)` must become a fallback (panic = forfeit). Part 2
  RUNNING: nit fix + gates + logs + real row on same branch.
- [ ] **S2 replies (#1 defect).** Phase-1 DONE (`origin/lane-v8-reply`):
  VladNet predictor validation (`13b5e83`) — 10% overall vs 7.5% greedy (beats
  baseline, MISSES ≥40% target), 7.7% on close turns (MISSES ≥60%), opener script
  16.7% in opening; finding: VladNet cuts are surgical/corridor-specific, truss
  plays are long-range anchors (max-damage / cluster rules miss). GB profile +
  pseudo-spec (`2454256`, `research/archetype-gb.md`): PREMISE HIT 2026-09-30 —
  GB habit replication (`4e84447` on `lane-v8-ideas`) shows C1 "2+ shared nodes"
  INVERTED (GB 24% vs v7 control 48%), root-anchoring ordinary, only survivor is
  close-rate halving (0.50x, 3/3 games). S2 dose must re-base on the close-rate
  tell or the lane kills; profile root coordinates are wrong for this grid.
  Worktree `game-v8r` has uncommitted `archetype_profile.rs`/`profile_gb.rs`/
  `replay_match.rs` + dirty Cargo files — commit or drop first. NEXT: (1) GB
  predictor validator on held-out GB games for the CLOSE-RATE tell (not
  corridor-infra); (2) wire dose behind toggle default-OFF (verify OFF-identity)
  only if >55% overall and every archetype up; (3) full gates. M2 port BLOCKED.
- [ ] **R red-split.** REFRAMED by E-1 (both colors run 8-move mesh; dose =
  blue-free vs red-scripted, not "make blue like red"). PARKED 2026-09-30 after
  3 failed attempts (stale-base checkout in main repo — corrected; silent death;
  incoherent output). `origin/lane-v8-r` = stale `78e2caa` junk (pre-harness
  base + Lane A contamination) — NEVER use, never cite. Clean worktrees
  `game-v8r2`/`game-v8r3` sit at `fd2ff63`; empty branch `lane-v8-r2` created.
  NEXT: single fresh attempt only when a reliable slot frees; dose-first-push
  discipline. Do NOT cite pre-harness numbers.
- [x] **P rebuild-denial. CLOSED.** Dose 2 KILLED byte-identical (`25d5308`).
  Dose 3 (MEMORY 20) gated `4b45650` on `origin/lane-v8-p`: reported REJECTED
  (league −25.5%; methodology caveat — control comparison unclear). Lane closed
  regardless. Successor hypothesis POP-PRICE.
- [x] **M10 MCTS pilot. KILLED.** External run: 50.0% at 2.2k sims vs fixed beam
  (2–2 informative, rest color noise), ~2.8s WASM fits budget — strength failure,
  not budget. Filed `c3cd6ab`. Effort to M7 reply-model line. Revisit only with
  learned/opponent-conditioned rollouts.
- [x] **Rivals.** Playbooks + miner (`bdb5b0e`), habit-2 replication (`3d6e366`),
  VladNet archetype (`e2b10e8`), all on `origin/lane-v8-rivals`. GB habit
  replicated 2026-09-30 (`4e84447` on `lane-v8-ideas`): corridor-infra C1
  INVERTED (GB 24% vs v7 control 48%) — do not cite; surviving GB number is
  close-rate halving (0.50x, 3/3 games). `rival-vladnet.md` CLEAN after 4 rounds
  (`c784d4e`, `7afdaef`, `c0099c0`, `540b7b1`); our-color rerun filed + corrected
  (`d6a4951`, `c686181`).

## Phase 3 — remaining Step-0s (no engine change; cheapest first)

- [ ] **BEAM-SEED** (idea #2, +30): census 44.0% outside top-8 (strict; 29.4%
  lenient) filed `83ea216` — SURVIVES kill-0 (<10%), provisional GO with
  setup-move caveat. Setup-aware re-run stalled (codec fixed, 3/12 validated,
  external box out of budget — shelved; can run locally if D1 gets sized).
  Cost gate RUNNING (`lane-v8-beam-cost`): median WASM ≤2.0s, max ≤4.5s.
- [ ] **PREFIX-ABORT** (idea #5, +10): losses arm filed `ca83cdb` — foe-area 0 in
  9/12, best rule catches 3/12, lean-KILL. Wins arm BLOCKED (5 win JSONs missing;
  site API pagination 403s without auth — retry with backoff or owner cookie).
- [ ] **PREP-BOOK** (idea #3, +20): LAST. Ghost harness + offline counter-lines vs
  GB scripts. Needs GB site games (b1e7d2d1/f33e29fc — Phase 4 evals).

## Phase 4 — ship path (blocked on site evals, not agents)

- [ ] 16 queued rated evals finish → **points agent** (full autopsy + rival files +
  6j A1-A5 analyses).
- [ ] Merge gated winners to master ONE at a time, re-gating after each merge.
  Never merge KILLed/neutral-without-stacking-decision/ungated doses.
- [ ] M4 bar → `bench_timed` + `site_check` (12/12 legal) → WASM → upload v2 →
  20 rated games → **ship Riposte v8**.

## Killed — do not retry without new evidence

contest-gen 0/32 · Lane E all-KILL · Q1a/Q1b · Q2/Q3b/Q5/Q8/Q11/Q12 · W doses 1-3
(league −27/−30%) · P dose 2 (byte-identical) · P dose 3 (REJECTED) · M10 MCTS
(50.0% at 2.2k sims) · GB-book copy (−48pp) · GB corridor-infra premise (C1
inverted vs control) · M2 port (blocked on S2) · symmetry pruning (no Elo while
depth is theater) · Cluster-3 engine work (measure first) · red-opener design
(premise false, E-1).

## Lane → branch → worktree index

| Lane | Branch (origin) | Worktree | State |
|------|-----------------|----------|-------|
| POP-PRICE D1 | `lane-v8-pop-r2` | `game-v8pop2/3` | dose `5b745f0` clean base; part 2 (gates+logs+row) RUNNING |
| TIE-SYM D1 | `lane-v8-tiesym-d1-r2` | `game-v8t3/4` | dose `7d0d80e` spec-exact, NEUTRAL; part 2 (nit+logs+row) RUNNING |
| S2 replies | `lane-v8-reply` | `game-v8r` | phase-1 done; corridor premise INVALIDATED — re-base on close-rate tell or kill |
| R split | — (parked) | `game-v8r2/3` clean | 3 attempts failed; `origin/lane-v8-r` junk, never use |
| P rebuild | `lane-v8-p` | — | dose 3 REJECTED, CLOSED |
| W wall | `lane-v8-w` | — | 3 doses KILL, closed |
| M10 MCTS | — | — | KILLED (50.0%), filed `c3cd6ab` |
| BEAM-SEED | `lane-v8-beam-cost` | `game-v8cost` | census GO; cost gate RUNNING |
| Rivals | `lane-v8-rivals` + `lane-v8-autopsy` | — | GB C1 dead, close-rate lives; rival-vladnet CLEAN |
| Ideas/docs | `lane-v8-ideas` | `game-ideas-file` | E-6, autopsy rerun, censuses, M10 verdict filed |
| Autopsy/data | `lane-v8-autopsy` | — | hosts 12 game JSONs |
| Audit/M7 | `lane-v8-audit` | — | hosts flip_test |

## Run queue (cheapest unblocked first; R parked, S2 needs redirect)

1. POP-2 + TIE-2 + BEAM-cost finish (running) → stack-or-kill calls on backed numbers.
2. S2 redirect (close-rate tell validator) or kill.
3. PREP-BOOK (last step-0) → idea backlog filing → Phase 4 (16 evals → points
   autopsy → merge winners one-at-a-time → M4 → WASM → 20 games → ship).
