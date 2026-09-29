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
