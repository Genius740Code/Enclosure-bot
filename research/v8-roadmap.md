# v8 roadmap — everything between here and ship (2026-09-29, master `eaf8af5`)

**Strength delta over v7: +0 Elo.** Nothing engine merged. Only docs + examples-only
harness on master. v7 stays the rated bot until a gated stack passes M4.

**Ship bar** (v8-plan §3): Elo > v7 over 20+ games; beats v7 mirrors both colors;
takes games off ≥2 rivals; no collapses. **Cut rule** (M4): stacked master beats v7
≥7/10 → WASM build → upload "Riposte v8" → 20 rated games.

**Resourcing:** max 2 Nvidia subagents + 1 Nemotron-via-OpenCode subagent, 24/7.
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
- [ ] Codify E-6 in v8-plan §2 (per-chair dose-vs-control; fixed 3/5 bar has ~10%
  pass for equal bot). All lanes already record per-chair tables.
- [ ] Autopsy our-color re-run: b0ac4141, 1c69056c (`30c7653b` fixed `e1b82d3`).
  No lane cites those two games until corrected.

## Phase 1 — shared infra [DONE]

- [x] `v7base` frozen baseline + `h2h_base` runner, merged `8ac25c8`
  (examples-only, zero engine delta). Identity 20/20, gauge 6/6, self-h2h Blue
  0/5 Red 5/5 (color-decided mirrors — the baseline every dose is judged against).

## Phase 2 — lanes (one variable per dose, full gates)

- [ ] **POP-PRICE (idea #1, +35 Elo est).** Kill-0 PASSED 6/7 (`poprice-ledger.md`,
  `origin/lane-v8-rivals` `3d6e366`) + BEAM-SEED premise confirmed (alternatives
  rank 10–106). **D1 IMPLEMENTED** (`DOOM_TAIL_W` 0→0.5, `af35f50`,
  `origin/lane-v8-pop`, worktree `game-v8pop`). Gates run 2026-09-29: gauge 6/6
  PASS; league_mesh AVG +29.3% / worst −42.7% PASS per bar (>−9.9%, no row<−300);
  h2h vs v7base Blue 0/5 Red 5/5 FAIL — identical to v7base self-h2h, i.e.
  dose == control per chair (NEUTRAL, not a winner). NEXT: (1) scoreboard row on
  `lane-v8-pop` + push; (2) founder decision stack-or-kill (neutral single-variable
  fix is stackable); (3) D2 (`DOOM_PAIR_W`) only if stacking, else close lane.
- [ ] **TIE-SYM (idea #4, +15).** Census DONE, LIVES (`8168ea2`,
  `origin/lane-v8-tie`, worktree `game-v8t`): exact-tie share 13.4% beam /
  19.3% aw (≫5% kill-0), dx color gap 1.131 (≫0.5), replication 0/648 mismatches,
  `tie-census-beam-full.csv` (648 rows) committed. D1 SPEC ready: on exact ties
  (gap <1e-9) break by distance-to-center nearest-first, then `mv.index()` (two
  sort comparators in `search.rs`). NEXT: implement D1 on fresh branch off master
  in `game-v8t` (or new worktree) → build → gauge + h2h + league → scoreboard row
  → push `lane-v8-tiesym-d1`. Owner: Nemotron slot (small, well-specified).
- [ ] **S2 replies (#1 defect).** Phase-1 DONE (`origin/lane-v8-reply`):
  VladNet predictor validation (`13b5e83`) — 10% overall vs 7.5% greedy (beats
  baseline, MISSES ≥40% target), 7.7% on close turns (MISSES ≥60%), opener script
  16.7% in opening; finding: VladNet cuts are surgical/corridor-specific, truss
  plays are long-range anchors (max-damage / cluster rules miss). GB profile +
  pseudo-spec (`2454256`, `research/archetype-gb.md`): 37.5% greedy baseline,
  43.3% corridor-infra "Other", targets ≥45%/≥80%/≥40%/≥50%. Worktree `game-v8r`
  has uncommitted `archetype_profile.rs`/`profile_gb.rs`/`replay_match.rs` +
  dirty Cargo files — commit or drop first. NEXT: (1) GB predictor validator on
  held-out GB games; (2) wire dose behind toggle default-OFF (verify OFF-identity)
  only if >55% overall and every archetype up; (3) full gates. M2 port BLOCKED.
- [ ] **R red-split.** REFRAMED by E-1 (both colors run 8-move mesh; dose =
  blue-free vs red-scripted, not "make blue like red"). STALE: worktree `game-v8rt`
  sits at pre-harness `1741965`; `lane-v8-r` was never pushed (no origin ref).
  NEXT: rebase worktree onto master `eaf8af5` (has harness), apply dose per spec,
  build, gate (h2h/league/gauge), push `lane-v8-r`. Do NOT cite pre-harness numbers.
- [ ] **P rebuild-denial.** Dose 2 KILLED byte-identical (`25d5308`). Dose 3
  (MEMORY 20, `cf080b9`) exists LOCALLY in `game-v8p`, ungated, unpushed, plus
  untracked diagnose scripts. NEXT: gate dose 3 once vs control (expect plateau)
  → scoreboard row → push → close lane regardless. Successor hypothesis POP-PRICE.
- [ ] **M10 MCTS pilot.** External session reached ~75% (playout 2,240/s native,
  1.4k sims/s MCTS, variants 22/28, pilot written unrun) then died with quota.
  Code NOT in any worktree (nothing named m10/pilot on disk). NEXT: reconstruct
  from `flip_test.rs` (`game-v8a`, M7 audit: 39.5% match, flip 45%) + M7's 20
  positions (3a414aad JSON on `origin/lane-v8-autopsy`) → run pilot-vs-beam →
  write `m10-mcts-pilot.md` → push `lane-v8-mcts`. Verdict pilot-or-kill at ~2.2k
  sims (prior ~60% kill). WASM ratio: use 3x, no build needed.
- [x] **Rivals.** Playbooks + miner (`bdb5b0e`), habit-2 replication (`3d6e366`:
  32/34 lag 1-2 confirmed+strengthened, −314.0/34 exact), VladNet archetype
  (`e2b10e8`), all on `origin/lane-v8-rivals`. Remaining: replicate one GB habit
  before lanes cite GB figures (GB numbers provisional).

## Phase 3 — remaining Step-0s (no engine change; cheapest first)

- [ ] **BEAM-SEED** (idea #2, +30): census screen-argmax-outside-priority-top-8 rate
  on a8e03a5f/ad65f054/d2d4b4fd/30c7653b/3a414aad. Kill-0: <10%. Cost gate: median
  WASM move ≤2.0s, max ≤4.5s (Oct-4 forfeit rule). Premise already half-confirmed
  by POP ledger (ranks 10–106); this is the confirmation sweep.
- [ ] **PREFIX-ABORT** (idea #5, +10): tabulate foe-area at prefix index 4/6 over 17
  games. Kill-0: fires in ≥50% of wins or <2 of 12 losses.
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
(league −27/−30%) · P dose 2 (byte-identical) · GB-book copy (−48pp) · M2 port
(blocked on S2) · symmetry pruning (no Elo while depth is theater) · Cluster-3
engine work (measure first) · red-opener design (premise false, E-1).

## Lane → branch → worktree index

| Lane | Branch (origin) | Worktree | State |
|------|-----------------|----------|-------|
| POP-PRICE D1 | `lane-v8-pop` | `game-v8pop` | implemented `af35f50`, gated: gauge PASS, league PASS, h2h neutral-FAIL; needs scoreboard row + stack decision |
| TIE-SYM | `lane-v8-tie` | `game-v8t` | census LIVES `8168ea2`; D1 spec ready, not implemented |
| S2 replies | `lane-v8-reply` | `game-v8r` | phase-1 done (`13b5e83`+`2454256`); dirty files to resolve; dose not wired |
| R split | `lane-v8-r` (local only) | `game-v8rt` | stale `1741965`, needs rebase + dose + gates + first push |
| P rebuild | `lane-v8-p` | `game-v8p` | dose 2 KILL; dose 3 local ungated; close after gating |
| W wall | `lane-v8-w` | `game-v8w` | 3 doses KILL, closed |
| M10 MCTS | `lane-v8-mcts` | — | 75% external, code missing, needs reconstruct+run |
| Rivals | `lane-v8-rivals` | — | done, GB figures provisional |
| Ideas | `lane-v8-ideas` | — | doc in chat, needs filing |
| Autopsy/data | `lane-v8-autopsy` | `game-v8c` | done; hosts 12 game JSONs |
| Audit/M7 | `lane-v8-audit` | `game-v8a` | done; hosts flip_test |

## Run queue (2 Nvidia + 1 Nemotron, cheapest unblocked first)

1. POP scoreboard row + push (either slot, 10 min) → founder stack-or-kill call.
2. TIE-SYM D1 implement+gate (Nemotron slot — small, fully specified).
3. R rebase+dose+gate (Nvidia slot — needs harness-era build/gate cycle).
4. P dose-3 gate + close (whichever slot frees first).
5. BEAM-SEED census → GB predictor validator → M10 reconstruct → PREFIX-ABORT.
