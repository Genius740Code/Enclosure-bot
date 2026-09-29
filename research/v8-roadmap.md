# Riposte v8 Roadmap

**Goal**: Ship Riposte v8 (Elo > v7). Single source of truth for all gate results, scoreboard rows, and lane progress.

**Master branch**: Currently at `6668733`. All worktrees fork from `master HEAD`; fast-forward pushes only. NEVER merge to master except gated winners one at a time with re-gating.

**Ship bar**: Stacked master beats v7 ≥7/10 → WASM → upload → 20 rated games.

---

## GATE LAW (v8-plan §2 + E-6)
- **gauge**: 6/6
- **league_mesh AVG**: >−9.9% AND no row < −300
- **h2h_base vs v7base**: judged PER CHAIR vs control at same color
- **Fixed 3/5 bar**: has ~10% pass for equal bots — record dose-vs-control tables
- One variable per dose. Scoreboard row per gate run. KILLED/ungated doses never merge.

---

## RUN QUEUE (in order)
*Each item: when a slot frees, launch with full brief (worktree, branch, spec section, gate commands, push target). After completion: record results in roadmap + scoreboard, push, then feed next slot.*

### 1. POP scoreboard row + push on lane-v8-pop
- **Estimated**: 10-min task, any free slot
- **Worktree**: `game-v8pop` (POP)
- **Branch**: `lane-v8-pop` forked from master
- **Goal**: Record a scoreboard row for POP variant vs v7base
- **Gate**: Minimal — just record row; no full gate pass required for first entry
- **Push target**: `lane-v8-pop`

### 2. TIE-SYM D1: implement center-first tiebreak spec (Phase 2)
- **Worktree**: `game-v8t` (TIE)
- **Branch**: `lane-v8-tiesym-d1` forked from master
- **Spec section**: Roadmap Phase 2 — center-first tiebreak implementation
- **Give to**: Nemotron first (fully specified task: scoreboard rows, census probes, spec'd D1 implements)
- **Goal**: Full gates → push lane-v8-tiesym-d1
- **Gate law**: gauge 6/6; league_mesh AVG >−9.9% AND no row<−300; h2h_base vs v7base per chair vs control at same color

### 3. R: rebase game-v8rt onto master, apply blue-free-vs-red-scripted dose, gate, first push of lane-v8-r
- **Worktree**: `game-v8r` (S2)
- **Branch**: `lane-v8-r` forked from master
- **Give to**: A Nvidia worker
- **Goal**: Rebase, apply dose, gate, push first entry

### 4. P dose-3 (MEMORY 20, local cf080b9 in game-v8p): gate once vs control, row, push, close lane
- **Worktree**: `game-v8p` (P)
- **Branch**: lane-v8-p forked from master
- **Give to**: Whichever slot frees first
- **Goal**: Gate once vs control, record row, push, close lane

### 5. BEAM-SEED census → GB predictor validator (held-out GB games) → M10 reconstruct+run
- **Tasks**:
  - BEAM-SEED census
  - GB predictor validator (held-out GB games)
  - M10 reconstruct+run (flip_test.rs in game-v8a + 3a414aad JSON on origin/lane-v8-autopsy)
  - PREFIX-ABORT tabulation
- **Order**: Sequential, one variable per dose

### 6. (Future) Additional v8 lanes as needed
- Additional doses and implementations as discoveries from earlier lanes

---

## SCOREBOARD
*Record per gate run. Format: lane | dose | h2h | league_avg | gauge | verdict | branch*

| Lane | Dose | h2h | league_avg | gauge | verdict | branch |
|------|------|-----|-----------|-------|---------|--------|
| (populated as lanes complete) | | | | | | |

---

## LANE WORKTREES (fixed)
- `game-v8pop` (POP) — lane 1
- `game-v8t` (TIE) — lane 2
- `game-v8r` (S2) — lane 3
- `game-v8rt` (R) — lane 4
- `game-v8p` (P) — lane 5

Each worktree has a corresponding `lane-v8-*` branch. New branches fork from master HEAD. Fast-forward pushes only.

---

## SUBAGENT MANAGEMENT
- Max 2 Nvidia subagents at once. 3rd Nvidia task waits in queue.
- 1 Nemotron subagent via OpenCode at once (small well-specified tasks only: scoreboard rows, census probes, spec'd D1 implements).
- Worktrees NEVER touch /home/genius74o/game.
- Error policy: 429/overload/503/timeout = transient. Sleep 180s and retry same call up to 999 times. Never treat transient error as failure. Re-seat worker ONLY if zero on-disk output for 30+ minutes.

---

## CYCLE LOOP (every cycle, from orchestrator)
(a) Check finished workers via on-disk state (never take their word alone)
(b) Record results in roadmap + scoreboard, push
(c) Feed each freed slot the next queue item with a full brief
(d) Report to founder in ≤15 lines: what finished, gate numbers, verdict, what launched next
Keep cycling until a gated stack passes M4 or the founder stops you.