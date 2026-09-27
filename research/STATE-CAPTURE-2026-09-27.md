# SUBAGENT STATE CAPTURE — 2026-09-27 (for continuation)

**Session:** `ses_f1ce7c415ffenGiwZX6uG7802M` (main) + 3 subagent sessions  
**Repo:** `/home/genius74o/game` (Enclosure-bot, branch `master`, pushed to `origin/master`)  
**Target:** Riposte WASM bot for Meridian at https://constellation.blueshrimp.uk/#rankings

---

## WHAT HAPPENED THIS SESSION

### 1. Baseline Reproduction (committed `eb100e9`)
- `probe_v3all`: v3 vs v1 **4/10** (Blue 2-3 — forced openings 5589/11723/9199 all lost as Blue), vs v2 **7/10**
- `probe_league`: **-20.0%** avg — genuine games both won (+29/+38%), red collapse line (-111%) persists
- `gauge`: **6/6** vs greedy
- Scoreboard created: `research/league-scoreboard.md`

### 2. My V5-1 Experiment (committed)
- Hypothesis: PATIENCE_PENALTY delays closes v1 snatches on forced openings
- Test: `PATIENCE_PENALTY` 2.0 → 0.0
- Result: **Byte-identical lines** across all three gates — term never flips a pick
- **REJECTED**, reverted to 2.0 with provenance comment in `search.rs`

### 3. Three Subagents Launched (all made progress, files on disk)

| Lane | Branch | Files Created | Status |
|------|--------|---------------|--------|
| A (Search) | `lane-a-search` | `retaliator/src/search_deep.rs` (589 lines) | **Complete negamax alpha-beta** over actions, PV table, iterative-deepening skeleton, time management hooks, depth parameterized. Eval + root priority verbatim from `search.rs` for clean isolation. |
| B (Eval) | `lane-b-eval` | `retaliator/src/eval_phases.rs` (547 lines) + 3 probes | **Legal-cuts-only vulnerability term** (V4d2): widens doom discount from single worst pop to total legally-cuttable area (erosion gap). One pass over enemy legal moves, no fantasy cuts, full-horizon units. Ablation control `baseline_best_move` (term OFF). Probes: `probe_b_h2h` (faithfulness + h2h vs shipped), `league_b` (vs scoutbase), `gauge_b` (vs greedy). |
| C (Autopsy) | `lane-c-autopsy` | `probe_bluechair.rs` (206 lines), `probe_balloon.rs` (198 lines) | **Blue chair diagnostic:** full per-action logs for v3 vs v1 at all 5 site openings, eval at every our-move, analyze() top-5 at big events, opener-response test (forced D10-F7 as action 1). **Red collapse diagnostic:** per-turn doomed-shape read t=40-90 (hull, components, independent loops, maxEnemyPop, minDist), every big break/area drop with exact action/cut edge, skip=10/20/30 like `probe_league`. |

---

## FILE INVENTORY (all untracked, on current branch `lane-a-search`)

### Lane A — Search Deepening
```
retaliator/src/search_deep.rs          # 589 lines — complete negamax alpha-beta
```
**Key features implemented:**
- Negamax alpha-beta over actions (depth parameterized, default 3)
- PV table (`MAX_PLY=121`), proper window handling (same player moves twice → no negation mid-turn)
- Extension factor applied at leaf with correct `scoring_events_left`
- Root priority = verbatim `search.rs` priority (all V3/V4 terms)
- `best_move` / `best_move_with_avoid` / `analyze` / `analyze_with_avoid` API
- Time management hooks (stubs for `limits.moveTimeMs` — soft 40% / hard 80%)
- **Deliberate deltas from `search.rs`**: fixed-array `one_edge_nodes` (no HashMap), no doom discount (depth sees pops inside tree), beyond-horizon accumulated along line

### Lane B — Eval Phases
```
retaliator/src/eval_phases.rs          # 547 lines — skeleton + vulnerability term
retaliator/examples/probe_b_h2h.rs     # Faithfulness + h2h vs shipped search
retaliator/examples/league_b.rs        # vs scoutbase (8 games)
retaliator/examples/gauge_b.rs         # vs greedy
```
**Legal-cuts-only vulnerability term (`VULN_W=1.0`):**
- `pop_and_cuttable(pos, victim)` → `(worst_pop, total_cuttable_area)` in one pass over enemy legal moves
- Legal cuts only: dense cluster (2+ touches = illegal) prices zero; shielded edges excluded by movegen
- `cuttable - worst` = erosion gap (multi-edge vulnerability the max never prices)
- Gated by `vuln: bool`; `baseline_best_move` = identical search with term OFF (ablation control)
- Faithfulness check in `probe_b_h2h`: control must match shipped search position-for-position

### Lane C — Autopsy
```
retaliator/examples/probe_bluechair.rs # Blue chair: v3 vs v1 at all 5 site openings
retaliator/examples/probe_balloon.rs   # Red collapse: ret vs scoutbase skip=10/20/30
```
**probe_bluechair:** Per-action log (eval before our move, close/break annotations, analyze() top-5 at big events), first-gain/first-close timing for both sides, opener-response test (forced D10-F7 at action 1 after site opening).

**probe_balloon:** Per-turn doomed-shape read (hull, components, independent cycles, maxEnemyPop, minDist), every big enemy break/our big drop with exact action/cut edge, t=40-90 focus.

---

## HOW TO CONTINUE (copy-paste into fresh session)

```bash
# 1. Clone / pull
cd /home/genius74o/game
git pull origin master

# 2. Check current state
git branch -a
# Should show: master, lane-a-search, lane-b-eval, lane-c-autopsy
# Current branch: lane-a-search (working tree has all 7 new files)

# 3. Run baselines to confirm nothing regressed
cd retaliator
cargo run --release --example probe_v3all
cargo run --release --example probe_league
cargo run --release --example gauge -- 6

# 4. Test Lane A (search_deep) — needs probe_deep.rs first
#    Compare search_deep::best_move vs search::best_move at depth 3
#    Then run probe_v3all, probe_league, gauge with search_deep

# 5. Test Lane B (eval_phases) — run the three probes
cargo run --release --example probe_b_h2h    # Faithfulness check first!
cargo run --release --example league_b
cargo run --release --example gauge_b -- 6

# 6. Test Lane C (autopsy) — run the diagnostic probes
cargo run --release --example probe_bluechair
cargo run --release --example probe_balloon

# 7. When a lane claims MERGE with numbers → append to scoreboard, merge to master, rebuild WASM
cargo build --release --target wasm32-wasip1
# Upload to site, play ≥2 games vs top bots, autopsy, iterate
```

---

## SUBAGENT SESSIONS (for resuming their exact context)

| Lane | Session ID | Model | Resume with |
|------|------------|-------|-------------|
| A (Search) | `ses_f1ce3bc27ffeHA0K5QYrWhn43n` | `nvidia/z-ai/glm-5.3-flash` | `tools.opencode.session_move({sessionID: "ses_f1ce3bc27ffeHA0K5QYrWhn43n", directory: "/home/genius74o/game"})` then continue |
| B (Eval) | `ses_f1ce3bc14ffeExrz3HJ6UIYgY5` | `nvidia/z-ai/glm-5.3-flash` | Same |
| C (Autopsy) | `ses_f1ce3b944ffet6CmJAgMpBBR4g` | `nvidia/z-ai/glm-5.3-flash` | Same |

**To resume a subagent:** use the `subagent` tool with the `sessionID` above, same prompt style as launch (they'll pick up where they left off).

---

## NEXT HIGH-IMPACT ACTIONS (priority order)

1. **Lane A:** Build `probe_deep.rs` (h2h vs `search::best_move` at depth 3, then depth 4/5), run full gate suite. If depth alone pays → add ID + TT + time mgmt. This is the **highest-EV structural work** (plan-v3 §2).

2. **Lane B:** Run `probe_b_h2h` → if faithfulness passes (0 mismatches), run `league_b` + `gauge_b`. If legal-cuts vulnerability moves league avg from -20% toward positive **and** doesn't hurt Blue chair → MERGE candidate.

3. **Lane C:** Run `probe_bluechair` + `probe_balloon`. Extract **action numbers** for every Blue loss divergence and every red collapse step. Write `research/c-blunder-catalog.md` with one ranked hypothesis per loss for Lanes A/B to test.

4. **WASM rebuild gate:** Only when a lane beats v1 head-to-head in `probe_v3all` (currently 4/10 → need ≥6/10) AND league avg improves from -20%.

---

## IRON RULES (carried forward)

1. All eval terms in **full-horizon points** (area × min(events,12)) or they silently do nothing.
2. Deterministic: tiebreak by move index. Legality via engine only.
3. Every claim needs **n ≥ 8 games, both colors**, vs v1 + scoutbase + greedy.
4. One file per track, never two lanes in one file. Commit + push per step.
5. **Never relitigate rejected ideas without new numbers:** GB book (-48pp), center-pull (-16pp), fixed 3-ply minimax (-58%), room weight 1.5 (noise). Eval is **not minimax-stable** — depth from real alpha-beta only.

---

## OPEN PROBLEMS (unchanged, ranked)

1. **Blue chair 0-5 vs v1** — first-close timing/placement as Blue (forced openings 5589/11723/9199)
2. **Red collapse line** — single 30+ pop ~t=60 + erosion (needs depth or vulnerability pricing)
3. **Search depth** — 2-ply ignores 20s budget (1000× headroom) → Lane A owns this
4. **Legal-cuts-only threat/vulnerability** — Lane B's term is the first cut at this
5. **Early expansion / "long sticks"** — bot doesn't claim space early; GB walls 12 actions before first close (~10 area/loop)
6. **Opponent modeling** — expect GB (big-loop book), VladNet (small-loop popper + red book), AngelBot (middle), Scout (rush), poppers, farmers, transformers

---

## OVERNIGHT PROMPT (for autonomous 15h run)

See `research/OVERNIGHT-AGENT-PROMPT.md` — full spec for running ≤4 subagents overnight with checkpoint uploads every 3-5h to constellation.blueshrimp.uk. Includes lane instructions, model assignments, API integration notes, and checkpoint template.

---

## QUICK COMMANDS REFERENCE

```bash
# Build WASM
cargo build --release --target wasm32-wasip1
ls -lh target/wasm32-wasip1/release/retaliator.wasm  # verify ≤8MB

# Scoreboard append template
cat >> research/league-scoreboard.md <<'EOF'
## 2026-09-27 HH:MM — <variant> — lane <A/B/C>
probe_v3all: v1 <W>/<L> v2 <W>/<L> | probe_league: <avg margin>% | gauge: <W>/<L>
Verdict: MERGE | REJECT
EOF

# Git workflow
git add -A
git commit -m "<lane>: <one-line summary with numbers>"
git push origin <lane-branch>  # never push to master directly
```

---

**END STATE CAPTURE.** All work preserved in working tree on `lane-a-search` branch. Subagent sessions active and resumable. Ready for continuation.