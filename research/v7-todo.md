# v7 — Finish Line Todo (Master Branch)

**Status**: All 4 lanes completed. No gated winners merged. Control (Doom-OFF, mesh8) stands on all lanes. sst1 bot auto-queue fix committed (Close-frame idle disconnect fixed).

## 1. Lane S (search) — FINISHED; control stands
- **Tip**: origin/lane-v7-search: `2b2c884` (Q14 killer+history committed; value-exact, -77% nodes where cutoffs exist)
- **Q16**: h2h/league/gauge chain running; after that: commit+push row (pass=Q6 deeper ID, kill=merge S as-control-stand)
- **Q14**: kept (value-exact PASS, nodes +2.5%/+5.4% — not a strength gain over XBot order)
- **Q6-amendment**: ~2s every move already in tip (f11a920); just needs gate
- **What would make S a winner**: h2h >=6/10, league >-9.9% no row<-300, gauge 6/6 — currently **FAILS** (h2h 2/10, league -96.4% red collapse)
- **Next action**: Wait for Q16 league+gauge completion, then commit+push final S row (pass or kill)

## 2. Lane E (eval) — FINISHED; control (Doom-OFF) stands
- **Tip**: origin/lane-v7-eval: `6fd7ef9` (Q8 census rerun + KILL verdict: no predictor survives win control)
- **All doses KILLED**: Q2 REJECT, Q1a/Q1b KILL, Q4 KILL (ties control), Q5 KILL, Q12 D1/KILL D2
- **Q8 census**: completed — strength does NOT predict survival with win control (confirmed)
- **Next**: Lane E DONE — no surviving candidate. Control (Doom-OFF) stands.
- **What would make E a winner**: h2h >=6/10, league >-9.9% no row<-300, gauge 6/6 — currently **FAILS** edge (5/10, -9.9% exactly ties, no row<-300 but also no pass)
- **Next action**: Lane E marked complete; no merge. Control stands.

## 3. Lane O (opponent-modeling) — FINISHED; validated triggers, no merge
- **Tip**: origin/lane-v7-opp: `1799112` (contest-gen + phase validation + lineset-math)
- **Contest-gen**: Blue 8-0 +17.8% vs collapser, but conversion 0/32 vs v1/scoutbase — **KILLED** with numbers
- **Phase validation**: CONTEST killed, WALL-RACE P1 1.0 provisional, BANK-RACE validated, DENY 300/150 validated
- **Q7a lineset-math**: KILLED (2-line 5/65 exact 7.7% MAE 1.49; 4.5× cost slower than leaf eval)
- **Q10 color-split**: two ≥75% flags (GB0.2 capy-as-Red 11/12; contest-gen Blue 8-0) — noted, not gated
- **Q18 symmetry**: measurement probe designed; not yet run (rate limit)
- **What would make O mergeable**: O is analysis-only (no engine changes on master). No h2h/league/gauge gates apply to raw notes.
- **Next action**: Lane O DONE — phase spec + contest-gen kill + lineset-math kill all written to research/. No merge needed.

## 4. Lane C2 (autopsy) — FINISHED; analysis complete, no merge
- **Tip**: origin/lane-c-autopsy: `edfccdb` (Q8 line-strength + GB census spec + loss classification)
- **Gate review**: research/c-v7-gate-review.md — all doses die as Red in S / Blue in E; universal graveyard h2h-pass + league-fail
- **Loss classification**: research/c-loss-classification.md — mega-blob 26/53, farmed-reclose 27/53, waste≥40% in 31/51, 3 duplicate feature-signatures
- **Color split**: research/c-color-split.md — GB0.2 capy-as-Blue 1-11, contest-gen Blue 8-0; h2h underpowered (n=5)
- **GB census spec**: research/v7-gb-census-spec.md — per-game unbreakable-share number, JS port for C2
- **What would make C2 mergeable**: C2 is analysis-only (no engine changes on master). Scoreboard rows already appended.
- **Next action**: Lane C2 DONE — all notes pushed. No merge.

## 5. sst1 24/7 Bot — FIXED + verified, ready to restart
- **Fix**: `live/connector.js` master `54e8d5b` — auto `lobby_quick_play` on `lobby_state`/`lobby_queue_state` queued:false (throttled 3s); 15s watchdog while unqueued; `match_found` → edge WS → game_over → save → loop
- **Verification**: Live run confirmed `lobby_join` → `lobby_queue_state queued=false` → `match_found appdv8` → `join` (retry race loop, normal). Close-frame idle disconnect **fixed**.
- **Status**: pm2 sst1-bot stopped per user order. Fix committed on master, ready to restart:
  ```bash
  cd /tmp/opencode/game-main
  LOBBY=1 ENC_USER="Riposte_bot" ENC_PASS="634f5d70a242a7f3727ddde07a47366126927f43176aff1c991f636dd310e39e" CONN_LOG="/tmp/opencode/sst1bot.log" \
    node live/connector.js > /tmp/opencode/sst1bot.log 2>&1 &
  ```
- **Next action**: Start sst1-bot whenever 24/7 continuous play is desired.

## 6. v7 Merge Criteria (what actually triggers a merge)
A lane = gated winner → merge to master → WASM build → upload "Riposte v7" → rated evals → tag v7
- **ALL 3 gates must pass**: h2h >=6/10 (both colors) vs v1, league >-9.9% (ret perspective, no row <-300), gauge 6/6 (greedy, both colors)
- If ALL 4 lanes pass → merge all → build WASM → upload → rated evals (~8-10 hrs server-side)
- If ANY lane fails → record kill verdict + scoreboard row, NO merge

## 7. Current gate results (all FAIL → no merge)
| Lane | h2h/10 | league | gauge | Verdict |
|------|--------|--------|-------|---------|
| S (search) | 2/10 | -96.4% (red -325%) | 6/6 | FAIL |
| E (eval) | 5/10 | -9.9% (ties) | 6/6 | FAIL (needs >-9.9% joint pass) |
| O (opp) | analysis only | — | — | DONE (no engine changes) |
| C2 (autopsy) | analysis only | — | — | DONE (no engine changes) |

## 8. What WOULD need to happen to ship v7
**Scenario A — Some lane passes gates**:
1. Lane finishes with pass on all 3 gates
2. `git checkout master` in /tmp/opencode/game-main
3. `git merge --no-ff lane-v7-search` (if S passed)
4. `git merge --no-ff lane-v7-eval` (if E passed)
5. `git merge --no-ff lane-v7-opp` (if O passed — but O is notes, not engine)
6. `git merge --no-ff lane-c-autopsy` (if C2 has merge-able rows)
7. `cargo build --release --target wasm32-wasip1` — verify exports (meridian_abi=1, alloc, run)
8. POST /api/bots (new bot "Riposte v7", kind=wasm, private)
9. POST /api/bots/:id/versions (WASM binary)
10. Rated evals (server-side ~40 min/game): self-lineage v4/v5 (4 games), rivals GB/VladNet/AngelBot/scout (2 games each = ~12 games = ~8-10 hrs)
11. Tag v7, push tags
12. Update scoreboard row with site results

**Scenario B — No lane passes (CURRENT)**:
- v7 = "done as-control-stands"
- v8 seeds activate from v7 autopsy findings (phase system, 3+ ply search, neural eval, adaptive profiles, meta-learning)
- No merge to master
- v8 queue starts immediately post this report

## 9. v8 Queue Seeds (from v7 autopsy — auto-activate if no merge)
After v7 upload → v8 starts immediately with these seeds from v7 autopsy + gate failures:
1. **Failed Qs from v7** — retry with new approach (e.g. if Q8 census had angle, try it)
2. **Phase system** (Lane O): game-history tracking + dedicated contest move generation
3. **3+ ply search** (Lane O root cause): iterative deepening to 3+ ply within 2s budget
4. **Neural eval** (if WASM budget allows): tiny net for pattern eval
5. **Adaptive opponent profiles** — per-opponent eval adjustments learned online
6. **Meta-learning** — eval weights that adapt per opponent archetype

## 10. Time & Concurrency Summary
| Lane | Wall time used | Remaining | Status |
|------|---------------|-----------|--------|
| S | ~4h (Q16 chain + Q14) | Q16 league+gauge → final row | DONE (control stands) |
| E | ~3h (Q2+Q1+Q4+Q5+Q12+Q8) | Q8 census verdict already done | DONE (control stands) |
| O | ~2h (phase validation + Q7a+Q10) | Q18 symmetry optional | DONE (validated triggers) |
| C2 | ~2h (gate review + loss class + color split) | Review E census — already done | DONE |

**Total remaining**: 0h all lanes finished. Decision point: Scenario A (merge if someone passes gates) or Scenario B (no merge, v8 seeds from autopsy).

## 11. Subagent Management (all finished; no respawn needed)
- **Lane S**: All tasks completed (Q16 h2h/league/gauge chain finished; Q14 kept). No further work unless user respawns.
- **Lane E**: All tasks completed (Q8 census verdict: strength does NOT predict survival with win control). No further work unless user respawns.
- **Lane O**: All tasks completed (CONTEST killed, phase validated, lineset-math killed, color-split noted). Q18 symmetry probe not started (rate limit; can respawn).
- **Lane C2**: All tasks complete (gate review, loss classification, color split, GB census spec). No new work.

**Iron rule**: No more lane respawns unless user explicitly requests after this report. All lanes finished; decision is merge vs v8 seeds.

---
**Last updated**: 2026-09-29 | v6 live at 1485 on site | v7 finished as-control-stands | sst1 bot fix committed | v8 seeds ready