# ENCLOSURE BOT - MASTER PLAN (v7 → v8 → v9)

> **Mission**: Build Riposte v7 (stronger than v6), upload to constellation.blueshrimp.uk, run rated games vs top bots, deploy 24/7 sst1 bot for continuous play/analysis, then iterate to v8/v9 with continuous improvement loop.

---

## 1. CURRENT STATE (as of 2026-09-28)

### v6 Status (LIVE ON SITE)
- **Bot**: "Riposte v6" | ID: `853d805d-8aaa-4061-a0bf-816fedd3e4c9` | Version: `93cbc665b8ab5b885983bec02d2969917f421ed916d3a23a0581095616a8cedd`
- **Rating**: 1485 (31-33, 64 games) | Provisional: true
- **v6 Components**: Mesh8 prefix (capybara center-mesh), Doom-ON, Pair-turn routing, XBot ordering
- **Weaknesses**: 0-8 vs GB/Vlad/Angel/Scout; 2-ply horizon mismatch vs collapsers; CAPTURE_W critical

### v7 Subagents (4 lanes, max 3 concurrent)
| Lane | Branch | Status | Work |
|------|--------|--------|------|
| **S (Search)** | `lane-v7-search` | DONE (pushed `667661b`) | Alpha-beta + aspiration + iterative deepening (2s budget); Gauge 6/6 ✅; v1 h2h 5/10 ⚠️ (fails) |
| **E (Eval)** | `lane-v7-eval` | Building probes | Unbreakable shapes (Q2), End-state valuation (Q1), Deny remote (Q4) |
| **O (Opp)** | `lane-v7-opp` | DONE (`9cde34f`) | Collapser repro (7/8); CAPTURE_W critical; 2-ply horizon mismatch root cause |
| **C2 (Autopsy)** | `lane-c2-autopsy` | Not pushed | v6 loss autopsy + 6 questions |

### sst1 24/7 Bot
- **Account**: `Riposte_bot` / passH1 `634f5d70a242a7f3727ddde07a47366126927f43176aff1c991f636dd310e39e`
- **Status**: Auth + WS handshake + syn/syn_ack + lobby_join + lobby_state ✅
- **Missing**: `lobby_quick_play` → `match_found` → edge game WS → retaliator moves → save games

---

## 2. V7 QUEUE — 19 KILL-GATED IDEAS

Each idea: **one variable at a time**, **dose sweep**, **gates** (n≥8 both colors: v1 h2h ≥6/10, league > -9.9%, gauge 6/6, no collapse < -300). Passes all gates or dies.

### SEARCH (Lane S) — 10 Ideas
| # | Idea | Gate |
|---|------|------|
| Q6 | **~2s EVERY MOVE** (amended) | Iterative deepening to max depth in ~2s on EVERY move; selective extensions on captures/breaks/scoring/pre-close | Avg > 1.5s; gauge 6/6; v1 ≥6/10 |
| Q7a | Math Line-Sets (2-6 lines) | Closed-form area yield for small move-sets | Beats single-move eval |
| Q9 | Multi-Turn Horizon (4-5 turns) | C2 judges 5-turn arcs; S extensions reach turn-5 payoff | Best-5 ≠ Best-1 |
| Q13 | Full Transposition Table | Depth-preferred, bound types (EXACT/LOWER/UPPER), aging, PV reconstruction | TT hit > 40% mid/late; value-exact |
| Q14 | Killer Moves + History Heuristic | Dynamic ordering: killers, history, countermoves, capture history | Nodes/pos drops > 20% vs XBot |
| Q15 | LMR + Probcut | Reduce depth on late moves; null-window probes at high depth | Nodes/pos drops > 15%, zero value change |
| Q16 | Quiescence Search | Extend volatile lines (captures/breaks/scoring) until quiet — IS Q6+Q9 | No horizon blunders on bench 596 |
| Q17 | PVS / NegaScout | Null-window probes after first move; re-search on fail-high | Nodes/pos drops > 10% vs alpha-beta |
| Q18 | Symmetry Pruning | In symmetric positions, eval 1 move per orbit, expand to siblings | Nodes/pos drops in symmetric, zero value change |
| Q19 | Confidence-Based Play | High eval variance → widen window, maximin, extend; low → snap | Reduces catastrophic blunders |

### EVAL (Lane E) — 8 Ideas
| # | Idea | Gate |
|---|------|------|
| Q1 | End-State Break-Fix Valuation | Charge only for *missed* scoring events; repaired ground that banks every event costs ~0 |
| Q2 | Unbreakable-Shape Building | Bonus for our moves creating/extending 2+ touch thickets |
| Q4 | Deny Remote Space | Price enemy far-from-fight expansion by unbreakability (shared-node count) |
| Q5 | Connect Timing + Survival | Delayed-close bonus scaled by open space; closes shift later while space remains |
| Q7b | Break Taxonomy Per Type | Per-type pricing: neck cut, loop pop, corridor sever, bank raid |
| Q7c | Keep-Alive Opposite Line | Mid-board opposite-side presence line, maintained as anchor |
| Q8 | Line Strength Ranking | Rank live lines by thickness/touches/bank-rate/repair cost; reinforce strong, abandon weak |
| Q12 | Reinforce vs Prevent-Reinforce | Reinforce our 2+ touch lines; contest opponent's near-thick lines |

### OPP MODELING (Lane O) — 3 Ideas
| # | Idea | Gate |
|---|------|------|
| Q3 | GB Detect/Counter/Steal | C2 census → E prices unbreakable-share; O tests counters |
| Q3b | Shape-Breaking Counters | Early contest of corridor root, pre-build deny 2nd wall, price-denial bank race |
| Q11 | Prevent-The-Wall (Pre-Close) | Closure-progress trigger: enemy 2-4 turns from big close → contest corridor root |

### AUTOPSY (Lane C2) — 4 Ideas
| # | Idea | Gate |
|---|------|------|
| Q3a | GB Unbreakable-Share Census | Per-game unbreakable-share number; predicts GB win |
| Q7b | Break Taxonomy Per Type | Per-type frequency × damage from autopsies; price/avoid per type |
| Q8 | Line Strength Survival Census | Does strength-ranking predict which lines survive? |
| Q10 | Rival Color Asymmetry | Separate responses per color where rival has ≥75% wins in one color |

### ALREADY IN v6 (v7 KEEPS)
- Mesh8 Prefix (capybara center-mesh, 8 own-moves/color, inj 8/8, league +32.6%, gauge 6/6)
- Doom-ON (restored after v5's 4-36)
- Pair-Turn-Aware History Routing (onset scan, permanent truncation)
- XBot Move Ordering (captures > cuts > closes)

---

## 3. GATE PROTOCOL — ONE VARIABLE AT A TIME

**Per dose/variant:**
1. Implement single variable in lane's file(s) only
2. Build: `cargo build --release --examples` (local, no rate limit)
3. Run gates (all n≥8, both colors):
   - `probe_b_h2h` — v1 h2h 10 games (target ≥6/10)
   - `league_b` — scoutbase 8 games (target > -9.9%, no collapse row < -300)
   - `gauge_b` — greedy 6 games (target 6/6)
4. Scoreboard row → `research/league-scoreboard.md`
5. Commit + Push to lane branch (never force-push, never merge to master)
6. Next dose only after current passes or dies

**REJECTED (never retest without NEW numbers):**
GB 26-move book (-48pp), center-pull (-16pp), naive depth-3 (-58%), room weight 1.5 (noise), vulnerability term (worse everywhere), close-size weight 1.0-3.0 (all worse than off)

---

## 4. SUBAGENT MANAGEMENT (MAX 3 CONCURRENT)

### Launch Protocol
```bash
# For each subagent:
git -C /home/genius74o/game fetch origin
rm -rf /tmp/opencode/game-<lane>
git -C /home/genius74o/game worktree add /tmp/opencode/game-<lane> origin/<branch>
cd /tmp/opencode/game-<lane> && git checkout -B <branch> && git log --oneline -3
```

### Subagent Prompt Template
```markdown
You are Lane <X> (<track>) for Enclosure-bot v7.
Work ONLY in /tmp/opencode/game-<lane>; NEVER touch /home/genius74o/game.
NEVER touch retaliator/src/search.rs or lib.rs (main session owns those).

ADOPT, DON'T REDO: <rescue commit hash + description>. Verify builds.

YOUR TASK (one variable at a time):
1. <specific task>
2. Gates: n≥8 both colors, probe_b_h2h (≥6/10), league_b (>-9.9%), gauge_b (6/6)
3. COMMIT + PUSH PER STEP to origin/lane-<branch>
4. On HTTP 429: wait 120s, retry up to 5 times; if still failing, stop quietly

IRON RULES: deterministic (tiebreak move index), legality via engine only,
ALL terms FULL-HORIZON (area × min(events,12)). You own ONLY <lane files>.

REPORT BACK: per-variant tables + which beats control on all gates.
```

### Concurrency Control
- Start **Subagent 1 (S)** → wait for first commit/heartbeat → **Subagent 2 (E)** → **Subagent 3 (O/C2)**
- Max 3 concurrent. If ANY 429 → pause new launches, wait for bucket refill (120s+)
- C2 (autopsy) launches last (no build, just analysis)

**Models:**
- Primary: `nvidia/nvidia/nemotron-3-ultra-550b-a55b` (3 subagents)
- Fallback: `opencode/nemotron-3-ultra-free` (if Nvidia bucket exhausted)

---

## 5. V6 → V7 EVALUATION PIPELINE

### When ALL Gates Pass (v7 candidate ready):
1. **Merge** winner lane(s) to master
2. **Build WASM**: `cargo build --release --target wasm32-wasip1`
3. **Verify exports**: `meridian_abi=1`, `alloc`, `run` + live legal-move replies
4. **Upload**: POST `/api/bots` (new bot "Riposte v7", kind=wasm, private) → POST `/api/bots/:id/versions` (WASM binary)
5. **Rated Evals** (server-side, ~40 min/game):
   - **Self-lineage**: v4, v5 (mirrors, 2 pairs each = 4 games)
   - **Rivals**: GB, GB 2.0, VladNet, AngelBot, Scout, Atlas v3, xmybot, john.fun, Stompy, Scout (1 pair each = 2 games each)
   - **Total**: ~12-14 games = ~8-10 hrs server-side
6. **Autopsy**: Every finished game → `retaliator/examples/autopsy.rs` → Lane C2
7. **Scoreboard row** with site results
8. **Tag v7**, push tag, update scoreboard

### Acceptance Criteria for v7 Ship:
- Site Elo > 1567 (v6) after 20+ rated games
- Beats v4/v5 mirrors (proves not lateral)
- Improves vs at least 2 rivals (GB, Vlad, Angel, Scout)
- No catastrophic collapse (no 0-8 vs any rival)

---

## 6. sst1 24/7 BOT — CONTINUOUS PLAY & ANALYSIS

### Bot: `Riposte_bot` on meaf.us/sst1/
- **Credentials**: `Riposte_bot` / passH1 `634f5d70a242a7f3727ddde07a47366126927f43176aff1c991f636dd310e39e`
- **Working**: Auth → edgeToken → lobby WS → syn/syn_ack → lobby_join → lobby_state ✅
- **Need**: `lobby_quick_play` → `match_found` → edge game WS → retaliator moves → save games

### Bot Loop (daemon):
```python
# 1. Auth → edgeToken + edgeBaseURL
# 2. Connect lobby WS (enclosure-v1, auth.<passH1>)
# 3. Send syn → wait syn_ack → send lobby_join
# 4. Send lobby_quick_play → wait match_found
# 5. On match_found: connect edge game WS (edgeToken + gameID)
# 6. Play game: syn_ack → game_state → retaliator.best_move_routed() → send move
# 7. On game_over: save game JSON to /tmp/opencode/sst1_games/
# 8. Loop back to lobby_quick_play
```

### Analysis Pipeline (every 5 hours):
1. Pull latest games from `/tmp/opencode/sst1_games/`
2. Run `retaliator/examples/autopsy.rs` on each
3. Classify: missed close / farmed re-close / blue-chair fade / mega-blob / wasted cuts
4. Update C2 autopsy metrics (area-lifetime, wasted-cut %, farm-cycle lengths)
5. Update v7 queue priorities based on findings
6. If new bot version (v7/v8) deployed → ensure bot uses latest version

### Target Opponents on sst1 (always latest version):
- **Self-lineage**: v4, v5, v6, v7 (mirrors)
- **Rivals**: GB, GB 2.0, VladNet, AngelBot, Atlas v3, xmybot, john.fun, Stompy, Scout

---

## 7. V8 / V9 CONTINUOUS IMPROVEMENT LOOP

### After v7 Upload → v8 Starts Immediately
**v8 Queue Seeds** (from v7 autopsy + gate failures):
1. **Failed Qs from v7** — retry with new approach
2. **Phase system** (Lane O finding): game-history tracking + dedicated contest move generation
3. **3+ ply search** (Lane O root cause): iterative deepening to 3+ ply within 2s
3. **Neural eval** (if WASM budget allows): tiny net for pattern eval
4. **Adaptive opponent profiles** — per-opponent eval adjustments learned online
5. **Meta-learning** — eval weights that adapt per opponent archetype

### v8 → v9 → ... Continuous Loop:
```
vN gates pass → merge → WASM build → upload "Riposte vN+1" → rated evals → 
sst1 bot auto-updates to latest → continuous play → autopsy every 5h → 
update queue → vN+1 gates → repeat
```

**Subagent Architecture (same for v8/v9):**
- 4 lanes (S/E/O/C2), max 3 subagents
- Nemotron 3 Ultra (primary) / OpenCode free (fallback)
- 120s backoff, max 5 retries on 429
- Sequential launch, max 3 concurrent

---

## 8. AUTOMATION COMMANDS

### Start v7 Subagents (sequential, max 3):
```bash
# 1. Lane S (Search)
launch_subagent("S", "nvidia/nvidia/nemotron-3-ultra-550b-a55b", lane_v7_search_prompt)

# Wait for first commit →
# 2. Lane E (Eval)
launch_subagent("E", "nvidia/nvidia/nemotron-3-ultra-550b-a55b", lane_v7_eval_prompt)

# Wait for first commit →
# 3. Lane O (Opp)
launch_subagent("O", "opencode/nemotron-3-ultra-free", lane_v7_opp_prompt)

# When E heartbeats →
# 4. Lane C2 (Autopsy)
launch_subagent("C2", "opencode/nemotron-3-ultra-free", lane_c2_autopsy_prompt)
```

### sst1 Bot Daemon:
```bash
export SST1_USER="Riposte_bot"
export SST1_PASSH1="634f5d70a242a7f3727ddde07a47366126927f43176aff1c991f636dd310e39e"
export SST1_SAVE_DIR="/tmp/opencode/sst1_games"
nohup python3 /tmp/opencode/sst1bot/sst1bot_complete.py > /tmp/opencode/sst1bot.log 2>&1 &
```

### Analysis Cron (every 5 hours):
```bash
# Pull latest games, run autopsy, update metrics
cd /tmp/opencode/sst1_games && python3 /tmp/opencode/game-v6/retaliator/examples/autopsy.py <latest_game.json> <our_color>
# Update research/c-site-losses-3.md, research/v7-queue.md
```

---

## 9. GIT WORKFLOW

### Branches:
- `master` — only main session merges here (gated winners)
- `lane-v7-search`, `lane-v7-eval`, `lane-v7-opp`, `lane-c2-autopsy` — lane branches
- `v6-mesh` — v6 worktree (v6 merge target)

### Merge Rules:
- Lanes **never** merge to master directly
- Main session merges **only gated winners** to master
- Never force-push lane branches
- Tag v7, v8, v9 on master after successful upload

---

## 10. WHEN USER SAYS "UPDATE PROMPT"

Update this `plan.md` with:
1. **New v7 gate results** (which passed/failed)
2. **v7 upload status** (version hash, Elo after evals)
3. **sst1 bot status** (running? games played? analysis findings?)
4. **v8 queue updates** (new ideas from autopsy)
5. **Subagent status** (which running, which dead, which respawned)
6. **v6/v7 site Elo** current ratings
7. **Git push** this updated plan.md

Then git push:
```bash
cd /home/genius74o/game && git add plan.md && git commit -m "plan: updated with latest status" && git push origin master
```

---

## 11. SUCCESS DEFINITIONS

**v7 Success:**
- [ ] All 19 Qs either passed gates or died with numbers
- [ ] Search (S) + Eval (E) pass (core strength)
- [ ] v7 WASM builds, exports verified, legal moves confirmed
- [ ] Site upload succeeds, rated evals queued
- [ ] Site Elo > 1567 after 20+ games
- [ ] Beats v4/v5 mirrors + improves vs ≥2 rivals

**sst1 Bot Success:**
- [ ] 24/7 daemon running, reconnects on disconnect
- [ ] Plays latest version automatically
- [ ] Saves every game to `/tmp/opencode/sst1_games/`
- [ ] Analysis runs every 5h, updates v7/v8 queue

**v8+ Success:**
- Continuous loop: vN → eval → autopsy → vN+1 queue → gates → ship
- Elo monotonically increasing on site

---

**END OF MASTER PLAN.** This is the complete AI prompt. Any agent with repo access, Nemotron 3 Ultra access, and the `meridian_session` cookie can execute this plan from start to continuous vN+1 loop.

---

*Last updated: 2026-09-28 | v6 live at 1485 | v7 in progress | sst1 bot auth+WS working*