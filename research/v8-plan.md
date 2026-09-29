# v8 PLAN — ELO MAX (2026-09-29)

Founder brief: v7 mirror (v6) shows purposeless moves (f81e3b7e move 31:
Connect +1.1 area), passive play, impenetrable-wall losses to top bots,
unguarded remote lines, flat 2s clock with recompute waste, red weakness.
Objective: maximize site Elo. Kill-gated, one variable at a time.

## 0. Exhibits (site evidence, not theory)

- **E1 — move 31, f81e3b7e** (v6 blue vs v7 red, RED won): moves 30-31 are
  Connects gaining +4.5 / +1.1 area (autopsy `me=red`). Small-loop snatches
  while areas run b=39.8 r=42.1 at move 30. Angel lesson restated: space first,
  close late. Metric for Lane P: every played move must carry lifetime area.
- **E2 — mirror color split (7 finished v6-vs-v7, 2026-09-29)**: RED won 6/7
  regardless of version. v7 as red 3-1, v7 as blue 0-3. Founder watched red
  lose — contradicts this sample; EITHER the sample is noise (n=7) OR the
  watched games are other matchups (send IDs). Queued evals (16 games) decide.
  Lane R gates red/blue splits separately until resolved.
- **E3 — top-3 method**: Yshan/Capy/GB build impenetrable (2+ touch shared-node)
  walls and bank uncontested. Unbreakable-share does NOT predict wins (52.5%,
  coin-flip — Lane O), but shared-node counts still target WHERE walls form.
  Steal the method + break the method (Lane W).

## 1. Lanes (max 3 concurrent, Nemotron Ultra primary)

| Lane | Track | First task | Gate addition |
|------|-------|-----------|---------------|
| **R** | Red play | Red has no dictated opener (blue has D10-F7) and pair-turns start 1-2 Red. Mine v7-red wins (3-1) vs v7-blue losses (0-3): what does red do right? Port it to blue or fix blue. | h2h split: red ≥3/5 AND blue ≥3/5 separately (no hiding behind 6/10) |
| **W** | Walls: detect/prevent/steal | WALL-RACE 1.0 trigger (validated) → full implementation: enemy 2-wall progress → contest corridor root (prevent) + own 2-wall bonus gated on tempo (steal). Detect via shared-node census per game. | enemy wall completions DOWN + own wall bank UP, no tempo collapse (league still >-9.9%) |
| **P** | Purpose per move + rebuild-aware contest | Every move priced in lifetime area; punish +0.x Connects while open space remains (E1). Contest remote lines ONLY when rebuild is denied (shield/cut-memory: re-place path blocked ≥2 events); else expand (don't donate). Q4 retry with this condition. | mean lifetime-area/move UP, sub-2.0 closes before act 30 DOWN, wasted-cut % DOWN (C2 measures) |
| **T** | Adaptive clock + cross-move memory | 2s base; spend to HARD on top-2 gap < threshold; SAVE (snap) forced/quiet/decided; persistent TT: site reuses the WASM instance between requests — cache search state across moves so move N+1 never recomputes move N (the "calcute then calcute same thing" fix). | median ~2s, 0 over-limit, same pick on quiet (cache-hit rate reported) |
| **C2** | Autopsy all | Every finished v7 eval → autopsy → move-level purpose table; points analysis when evals land (founder: subagent thinks of points). E1 is exhibit #1. | append-only notes, no engine |

## 2. Gate protocol (unchanged + splits)

Per dose: h2h ≥6/10 (WITH red ≥3/5 and blue ≥3/5), league >-9.9% no row<-300,
gauge 6/6, plus lane metric above. One variable, dose sweep, scoreboard row,
commit+push to `lane-v8-*`, never force-push, never merge to master.
Main session merges ONLY full-gate winners. 429: wait 120s x5, stop quiet.

## 3. Ship bar (v8)

- Site Elo > v7 final after 20+ rated games (ELO MAX: no cap, keep iterating)
- Beats v7 mirrors both colors (no 0-3 repeats)
- Takes games off ≥2 of: GB, GB2.0, AngelBot-WASM, capybara, xmybot
- No 0-8 vs any rival; median move ~2s, 0 over-limit (Oct-4 rule)

## 4. Launch order

1. C2 first (autopsy backlog grows with every finished eval; feeds all lanes)
2. R + W (highest Elo leverage: color split + wall game)
3. P, then T (clock/memory needs stable eval to price against)
Max 3 concurrent. Worktrees /tmp/opencode/game-v8-<lane> on origin/lane-v8-<lane>.

## 5. Open questions for founder

- Red: which games did you watch red lose? IDs decide E2.
- Passivity: sometimes correct (doom discipline) — Lane P prices it per move
  instead of judging style.
- v7 evals still running (16 games); v8 doses must not move the shipped
  version mid-eval. v8 ships as new bot "Riposte v8".

---

## 6. FULL PLAN (founder brief 2026-09-29 late — ELO MAX, all points)

### 6a. VladNet loss seed (3a414aad, v7 blue, RED won, 120 moves)
v7 even at move 40 (b=43.4 r=49.2). Moves 42-46: VladNet pops our +4.5/+3.4
closes; by move 60 areas b=27.0 r=50.8 — never recovers (final b=25.4 r=49.2,
scores 1187-1861). Repeating-mistake cluster #1: OUR loops pop mid-game,
THEIR banks never pop. v8 must invert exactly this: unbreakable own banks +
break enemy banks. First C2 cluster file: `research/c-vlad-loss.md` (pending
full autopsy when evals land).

### 6b. Move quality: area-first with priced exceptions (M1)
Rule: the area move plays UNLESS another move outbids it in full-horizon
points with a NAMED reason (break value, deny value, wall-progress,
rebuild-denial). No reason = area move. Covers "shitty moves without reason".
- FAR-SMALL (small gain far from enemy): two-faced. Deferrable = option value
  (good slow move, do it later); played now = wasted tempo. C2 measures FIRST:
  do played-now far-small moves correlate with losses? Then price: FAR-SMALL
  penalty scaled by open space (penalize while space remains, allow when the
  board fills — the "can do future" judgment, mechanized).
- Lane P owns M1. Metric: exception ledger per game (% moves with named
  reason, area/move lifetime UP, sub-2.0 early closes DOWN).

### 6c. Chess programming port (M2, Lane S2) — chessprogramming.org mapping
"More search = stronger" holds only if depth is real. Timed beam is real but
shallow per second vs alpha-beta. Port from `lane-v7-search` (all measured):
- alpha-beta + aspiration windows (mover-perspective fix kept; red-collapse
  lesson: gate red split separately)
- TT with Zobrist keys (exact entries + best-move ordering; was value-exact)
- killer + history + countermove ordering (-77% nodes where cutoffs exist)
- quiescence: extend captures/breaks/scoring lines until quiet (this game's
  captures/breaks ARE the tactics)
- dose 2: LMR + null-move + PVS (Q15/Q17, ungated — fresh)
Composition: alpha-beta makes the 2s budget reach 2-4x deeper. Lane S2 owns
the port INTO live `search.rs` (not a side module — the v7 mistake was a
parallel engine that never shipped). Gate: nodes/pos DOWN at equal depth +
full gate chain + v7-mirror (M4).

### 6d. v8-beats-v7 bar (M4)
Founder: v8 must beat v7 most of the time. Mirror gate: candidate vs v7
(shipped timed version) ≥7/10 with red ≥3/5 and blue ≥3/5. v1 gates stay
(≥6/10). Both must pass. No lateral ships.

### 6e. Shape library (M5, Lane W second half)
Strong start shapes = unbreakable area (mesh8 prefix works: keep + extend).
Build a shape book: red opener (Lane R designs, red has none), mesh
continuations past move 8, corridor/blob/thicket starters keyed by STRUCTURAL
properties (never named patterns — Q2-amendment). Validate each shape across
GB + capybara + Angel + blob-mimic or it dies.

### 6f. Rival strat files (M6, C2+R when games land)
VladNet (mid-game pop timing + unbreakable banks — 3a414aad seed above),
Stompy, GB/GB2.0, AngelBot-WASM, capybara, xmybot: one file each —
what they do, when they do it, what beats it. Mined from OUR finished games
vs them (16 evals queued) + site games. This is the "analyse the bots"
deliverable.

### 6g. Phasing (games still running: 8 finished, 4 waiting, rest queued)
- PHASE 0 (now): mirror autopsy done (E1/E2); S2 port prep (no behavior
  change until gated); shape lab; warning cleanup done.
- PHASE 1 (when 16 evals land): points subagent (founder call) — full
  autopsy + rival mining + M6 files; full gates on R/W/P/T/S2 doses.
- v8 ships as NEW bot "Riposte v8". Never touch v7 mid-eval.

### 6h. M7 — reply-model audit (DIAGNOSTIC GATE for M2, Phase 0, no engine change)
Why depth never converts: our deep search assumes the enemy always plays
`ranked().first()` (greedy reply). Measure reply-match rate: across finished
site games + local lines, how often does the enemy's ACTUAL reply equal our
predicted reply? Also: does deeper search ever FLIP a pick vs 2-ply on real
positions? Verdicts: match <50% or flips ~never → depth is theater; v8's Elo
comes from opponent-aware replies (M6 files feed the model), NOT the M2 port.
Match high + flips frequent → M2 port proceeds. M2 ports NOTHING until M7
reports. Owner: Lane-recipe below (audit agent). Trigger for POINTS agent:
all 16 queued evals + user mirrors finished (eval IDs: f94f561b v3, 2adafd6a
Stompy, 9f06cd59 VladNet, 5164143f AngelWASM, b1e7d2d1 GB, f33e29fc GB2.0,
90a4e520 capybara, 323bf662 xmybot). Founder says "launch another subagent to
analyse" at that point — full autopsy + rival files + gate doses.
Objective over everything: higher Elo, wins vs other bots. Style is irrelevant.
