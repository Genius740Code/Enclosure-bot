# OVERNIGHT BOT IMPROVEMENT SESSION — AI AGENT PROMPT
**Repo:** `/home/genius74o/game` (Enclosure-bot, branch `master`)
**Target:** Riposte WASM bot for Meridian at https://constellation.blueshrimp.uk/#rankings
**Duration:** ~15 hours (overnight), checkpoint every 3–5 hours
**Models available (max 4 concurrent subagents):**
- `nvidia/nemotron-3-ultra-550b-a55b` — fast, huge context, great at code/impl
- `nvidia/z-ai/glm-5.3-flash` — smart, strong reasoning, good at analysis/ideas
- `nvidia/z-ai/glm-5.3` smart
- **Mix freely.** Default: 2×GLM (analysis/ideas) + 1×Nemotron (impl/build) + 1×GLM (autopsy/opponent modeling)

---

## CORE MISSION
Maximize **site Elo**. Local leagues (`probe_v3all`, `probe_league`, `gauge`) are gates — **only site Elo counts**. Current baseline: v3 (in `retaliator/src/search.rs`) loses to v1 4/10, beats v2 7/10, league -20% vs scoutbase, 6/6 vs greedy. **v1 is the gatekeeper** — v1 stays live until dethroned.

**Iron rules (non-negotiable):**
1. All eval terms in **full-horizon points** (area × min(events, 12)) or they silently do nothing — proven twice.
2. Deterministic: tiebreak by move index. Legality via engine only.
3. Every claim needs **n ≥ 8 games, both colors**, vs v1 + scoutbase + greedy.
4. One file per track, never two lanes in one file. Commit + push per step.
5. **Never relitigate rejected ideas without new numbers:** GB 26-move book (-48pp), center-pull bias (-16pp), fixed 3-ply minimax (-58%), room weight 1.5 (noise). This eval is **not minimax-stable** — depth must come from real alpha-beta, not naive deepening.

---

## REPO MAP (read these first)
```
retaliator/src/search.rs       # THE BOT (2-ply WIDTH-8, ~20ms, 1000× time headroom UNUSED)
retaliator/src/lib.rs          # Site ABI + replay (collects cut-avoid points)
retaliator/examples/           # Probes:
  probe_v3all.rs               # v1/v2 round robin (5 openings × 2 colors = 10 games each)
  probe_league.rs              # Scoutbase league (8 games, reports avg margin)
  gauge.rs                     # Greedy baseline (6 games)
  probe_collapse.rs            # Red collapse line tracer
  probe_bluechair.rs           # Blue-chair diagnostic (new)
  probe_balloon.rs             # Megaloop erosion diagnostic (new)
  autopsy.rs                   # Site game replay via GET /api/games/ID
  support/v1base.rs            # v1 baseline (asymmetric break-first)
  support/v2base.rs            # v2 baseline (symmetric horizon)
  support/scoutbase.rs         # Scout baseline (site bot)
retaliator/src/eval_phases.rs  # Lane B draft (legal-cuts-only vulnerability)
research/
  plan-v3.md                   # Lane A spec: alpha-beta + ID + TT + time mgmt
  plan-v4.md                   # Lane B/C symptoms + V4d2 unbreakable-shape rule
  rival-analysis.md            # GB/VladNet/AngelBot stats, steal ideas §7
  league-scoreboard.md         # APPEND HERE: date, variant, margins, verdict
  SUBAGENT-BRIEF.md            # Shared lane context
  OVERNIGHT-AGENT-PROMPT.md    # THIS FILE
vendor/meridian-bot-kit/       # Site engine (exact rules)
```

---

## CURRENT OPEN PROBLEMS (ranked by Elo impact)

| # | Problem | Evidence | Best attack lane |
|---|---------|----------|------------------|
| 1 | **Blue chair 0-5 vs v1** (forced openings 5589/11723/9199) | `probe_v3all`: Blue loses 3/5 forced openings vs v1; empty opening wins | A (search depth/timing), B (eval), C (autopsy divergence) |
| 2 | **Red collapse line** (-111% league at skip=10/20/30) | Single 30+ pop ~t=60 + erosion; `probe_collapse` traces actions 45–85 | A (depth/vulnerability), B (legal-cuts erosion term) |
| 3 | **Search depth** — 2-ply WIDTH-8 ignores `limits.moveTimeMs` (20s budget, ~20ms used) | 1000× headroom free; plan-v3 §2 has full spec | **Lane A owns this** — highest EV structural work |
| 4 | **Legal-cuts-only threat/vulnerability** — fantasy cuts priced in eval | V4d2: dense clusters = cut-proof (2+ touches = illegal); shielded edges untouchable 1 turn | **Lane B owns this** — widen doom discount to total legal cut exposure |
| 5 | **Early expansion / "long sticks"** | User observation: bot doesn't claim space early; GB walls 12 actions before first close (~10 area/loop) | B (mobility/potential-area), A (deeper search sees wall value) |
| 6 | **Opponent modeling** — expect many strat types: GB (big-loop book), VladNet (small-loop popper + red book), AngelBot (middle), Scout (rush), poppers, farmers, transformers | Rival-analysis.md §4-5 has full breakdown | C (autopsy), B (robust eval) |

---

## SUBAGENT ROLES (run ≤4 concurrently, reassign when one finishes)

### Lane A — SEARCH DEEPENING (`retaliator/src/search_deep.rs` + benches)
**Model:** Nemotron (impl speed) or GLM (reasoning)
**Spec:** `research/plan-v3.md` §2 — negamax alpha-beta over actions + iterative deepening + TT (Position hashes free) + time management from `limits.moveTimeMs` (soft 40% / hard 80%, easy-move fast path, panic on fail-low capped).
**Start minimal:** fixed-depth alpha-beta with current priority ordering → measure depth payoff → add ID → add TT → add time mgmt.
**Validation:** `probe_league` + `gauge` + `probe_v3all` + custom `probe_deep.rs` (vs `search::best_move`, n=8 both colors).
**Deliverable:** scoreboard row, `research/lane-a-*.md`, branch `lane-a-search`.

### Lane B — EVAL PHASES (`retaliator/src/eval_phases.rs` + probes)
**Model:** GLM (idea generation + ablation reasoning)
**Current draft:** legal-cuts-only vulnerability term (widens doom discount to total legal cut exposure per position, V4d2-compliant, full-horizon units).
**Next terms (one at a time, smallest-verified-first):**
- Mobility term (legal move count / reachable space)
- Potential-area machinery for empty-space building ("long sticks" expansion)
- Cut+make gap math (if I cut X, what can I close next turn?)
- Shield-expiry threats (enemy edges shielded NOW, cuttable NEXT action — one movegen call, capped)
- Phase gates (opening/midgame/endgame weight tables)
**Validation:** same 3 probes + custom `probe_eval.rs` vs `search::best_move`, n=8 both colors. **Blue-side evidence weighted** (Blue chair is #1 problem).
**Deliverable:** scoreboard row + ablation numbers (with/without), `research/lane-b-*.md`, branch `lane-b-eval`.

### Lane C — AUTOPSY / BLUNDER CATALOG (analysis only, `research/c-*.md`)
**Model:** GLM (pattern recognition)
**Tools:** `probe_bluechair.rs`, `probe_balloon.rs`, `probe_collapse.rs`, `autopsy.rs` (site games via API).
**Targets (action numbers mandatory):**
1. **Blue chair**: per forced opening loss vs v1 — first-close timing/area both sides, divergence action, eval at divergence, hypothesis (first-close timing? placement? patience misfire? forced-opening handling?)
2. **Red collapse**: characterize doomed shape (actions 45–85), erosion steps, single pop trigger
3. **Site losses** (if API works): `GET /api/games/{f61a06ec,f4b7f187,2ae426c6,...}` → replay → blunder catalog
**Deliverable:** `research/c-blunder-catalog.md` (one ranked hypothesis per loss for A/B to test), scoreboard summary rows, branch `lane-c-autopsy`.

### Lane D — OPPONENT MODELING / STRATEGY ENSEMBLE (rotating, when a slot frees)
**Model:** GLM
**Task:** Build probe opponents that mimic GB/VladNet/AngelBot/Scout styles (using rival-analysis §4-5), run `probe_match.rs` tournaments vs current bot + v1/v2, find exploitable patterns. Feed hypotheses to Lanes A/B.
**Deliverable:** `research/lane-d-*.md`, scoreboard rows.

---

## NIGHTLY WORKFLOW (repeat until dawn)

### 1. STARTUP (first 15 min)
- `git pull origin master`
- Read `research/league-scoreboard.md` for latest baselines
- Read `research/SUBAGENT-BRIEF.md` + relevant plan files
- Launch ≤4 subagents with assigned lanes (use `subagent` tool with `background: true`, correct `model`, sessionID if resuming)

### 2. WORK CYCLE (3–5 hours per checkpoint)
**Each subagent:**
- Build → measure → write note → commit to its branch → push
- **Append to `research/league-scoreboard.md`** (atomic append, one line per variant):
  ```
  ## YYYY-MM-DD HH:MM — <variant> — lane <A/B/C/D>
  probe_v3all: v1 <W>/<L> v2 <W>/<L> | probe_league: <avg margin>% | gauge: <W>/<L> | verdict: <MERGE|REJECT>
  ```
- If **MERGE candidate**: open PR / notify main session (do NOT merge to master yourself)

### 3. CHECKPOINT UPLOAD (every 3–5 hours, or when a lane claims MERGE)
- **Rebuild WASM:** `cargo build --release --target wasm32-wasip1` (verify exports, ≤8MB, live execution)
- **Upload to site:** Read `vendor/meridian-bot-kit` or site docs for API — typically POST `/api/bots` with wasm + metadata, then POST `/api/matches` to challenge top bots (Great Barrier 1907, VladNet 1584, AngelBot 1613).
- **Play ≥2 games per opponent** (both colors if possible). Wait for completion.
- **Autopsy results:** `autopsy.rs` on new games → feed to Lane C.
- **Version tag:** `git tag v5`, `v5-1`, `v5-2`... push tags.

### 4. REASSIGN FINISHED SUBAGENTS
- When a subagent finishes (reports MERGE or REJECT with numbers), **immediately spawn a new one** on the highest-impact open problem.
- Keep ≤4 running. Rotate models to strengths.

### 5. SHUTDOWN (dawn)
- Final scoreboard append with summary
- All lanes push final notes
- Main session merges gated winners, builds final v5 WASM, uploads for daytime rated matches

---

## IDEA SOURCES (not just chess programming wiki — any high-EV concept)
- **Transposition tables, alpha-beta, iterative deepening, aspiration windows** (chess/Go standard)
- **Late-move reductions, null-move pruning, probabilistic cutoffs** (Stockfish/AlphaZero)
- **MCTS hybrids** (baby-bot uses MCTS 16/8/6 — could blend with alpha-beta)
- **Potential-area / influence maps** (Go-style territory prediction for "long sticks" expansion)
- **Threat-space search** (only search moves that create/answer threats)
- **Opening book earned by search** (not copied — GB's book is its search earning walls)
- **Endgame tablebases** (Meridian has exact scoring — late-game is solvable)
- **Ensemble eval** (multiple eval functions, weighted by position type)
- **Learned move ordering** (history heuristic, neural net if WASM permits)

---

## API / SITE INTEGRATION (read `vendor/meridian-bot-kit` + site)
- **Upload:** POST wasm to `/api/bots` (check site for exact endpoint/auth)
- **Matchmaking:** POST `/api/matches` with `bot_id` + `opponent_id` + `color`
- **Replay:** GET `/api/games/{id}` → JSON → `autopsy.rs` replay
- **Leaderboard:** GET `/api/bots` for current ratings
- **Rate limits:** respect them; 20s/move, 120 actions, ~40 min/game

---

## SUBAGENT INSTRUCTION FILES (drop these in `research/` for each lane)

### `research/lane-a-instructions.md`
```markdown
# Lane A Instructions — Search Deepening
You are Lane A. File ownership: `retaliator/src/search_deep.rs` + `retaliator/examples/probe_deep.rs` + benches. NEVER touch `search.rs` or `lib.rs`.
Read: plan-v3.md §2 (full spec), SUBAGENT-BRIEF.md, league-scoreboard.md, rival-analysis.md §6 (Scout search).
Goal: Negamax alpha-beta + ID + TT + time mgmt from `limits.moveTimeMs`.
Start: Fixed-depth alpha-beta with current priority ordering. Measure if depth alone pays (v3 fixed-depth-3 was -58% — must be REAL alpha-beta with ordering).
Validate: probe_league (8 games), gauge (6), probe_v3all (10), probe_deep vs search (n=8 both colors).
Append scoreboard rows. Write research/lane-a-*.md. Commit to lane-a-search branch.
When done: report MERGE/REJECT with numbers. Main session merges.
```

### `research/lane-b-instructions.md`
```markdown
# Lane B Instructions — Eval Phases
You are Lane B. File ownership: `retaliator/src/eval_phases.rs` + probes. NEVER touch `search.rs` or `lib.rs`.
Read: plan-v4.md (symptoms 1-3, V4d2), rival-analysis.md §7 (steal ideas), SUBAGENT-BRIEF.md, league-scoreboard.md.
Current draft: legal-cuts-only vulnerability (eval_phases.rs, 522 lines).
Next: One term at a time. Ablation required (with/without, n=8 both colors).
Priority: (1) mobility/potential-area for early expansion ("long sticks"), (2) cut+make gap, (3) shield-expiry threats, (4) phase gates.
Validate: Same 3 probes + custom probe_eval vs search. Blue-side evidence weighted.
Append scoreboard. Write research/lane-b-*.md. Commit to lane-b-eval.
```

### `research/lane-c-instructions.md`
```markdown
# Lane C Instructions — Autopsy / Blunder Catalog
You are Lane C. ANALYSIS ONLY. No src edits. Files: research/c-*.md + read-only probes (probe_bluechair.rs, probe_balloon.rs exist).
Read: SUBAGENT-BRIEF.md, plan-v3.md, plan-v4.md, rival-analysis.md, league-scoreboard.md.
Targets (action numbers mandatory):
1. Blue chair: probe_v3all forced-opening losses vs v1 (5589, 11723, 9199 as Blue). Per-action: first-close timing/area, divergence action, eval at divergence, ONE hypothesis.
2. Red collapse: probe_league skip=10/20/30 (-111%). Probe_collapse actions 45-85: doomed shape, erosion steps, pop trigger.
3. Site losses: If API works, GET /api/games/{f61a06ec,f4b7f187,2ae426c6} → autopsy.rs → blunder catalog.
Deliverable: research/c-blunder-catalog.md (ranked hypotheses for A/B), scoreboard summary, commit lane-c-autopsy.
```

### `research/lane-d-instructions.md`
```markdown
# Lane D Instructions — Opponent Modeling / Strategy Ensemble
You are Lane D (rotating slot). File ownership: research/lane-d-*.md + probe opponents in examples/.
Read: rival-analysis.md §4-5 (GB/VladNet/AngelBot/Scout styles), SUBAGENT-BRIEF.md.
Task: Build probe bots mimicking each style (GB: 12-action diagonal walls → big loops; VladNet: red book + 30+ tiny loops; AngelBot: middleweight; Scout: rush closes). Run probe_match.rs tournaments vs current + v1/v2. Find exploitable patterns → hypotheses for Lanes A/B.
Validate: probe_match (n=8 both colors vs each style).
Append scoreboard. Write research/lane-d-*.md.
```

---

## MODEL ASSIGNMENT HEURISTICS
| Task type | Preferred model | Why |
|-----------|----------------|-----|
| Search implementation (alpha-beta, TT, time mgmt) | Nemotron 3 Ultra | Fast, huge context, strong Rust |
| Eval term design + ablation reasoning | GLM 5.3/5.4 Flash | Smart, good at "why this term works/fails" |
| Autopsy / pattern recognition | GLM 5.3/5.4 Flash | Strong reasoning over game traces |
| Opponent modeling / strategy ensemble | GLM 5.3/5.4 Flash | Creative hypothesis generation |
| WASM build / probe running / scoreboard appends | Nemotron | Fast tool use, reliable execution |

---

## CHECKPOINT TEMPLATE (append to league-scoreboard.md)
```
## 2026-09-27 HH:MM — <variant-name> — lane <A/B/C/D>
probe_v3all: v1 <W>/<L> v2 <W>/<L> | probe_league: <avg margin>% | gauge: <W>/<L>
Site games: vs GB <W>-<L> | vs VladNet <W>-<L> | vs AngelBot <W>-<L>
Verdict: MERGE (beats v1 head-to-head) | REJECT (with numbers)
Notes: <one line>
```

---

## REMINDERS
- **Max 4 subagents at once.** When one finishes → spawn new on highest-impact open problem.
- **Every claim needs numbers.** No "I think" — "probe_league: -15% → -5% (n=8)".
- **Blue chair is #1.** Weight Blue-side evidence in all lanes.
- **Early expansion ("long sticks")** is a user-observed weakness — if a term helps, it's high value.
- **Expect all opponent types.** Don't overfit to one style.
- **Version everything.** v5, v5-1, v5-2... tags on every upload candidate.
- **Have fun.** This is the best kind of optimization problem.

---

## QUICK START COMMANDS (for subagents)
```bash
# Baseline repro (run first)
cd /home/genius74o/game/retaliator
cargo run --release --example probe_v3all
cargo run --release --example probe_league
cargo run --release --example gauge -- 6

# Build WASM
cargo build --release --target wasm32-wasip1
# Check size: ls -lh target/wasm32-wasip1/release/retaliator.wasm

# Custom probe template (copy probe_league.rs, swap opponent fn)
```

---

**END OF PROMPT.** Subagents: read your lane instructions file, then execute. Main session: monitor scoreboard, merge gated winners, manage uploads.