# v9 roadmap — Elo hunt (opened 2026-09-30, v8 stack-2 shipped to site same day)

**Base:** master past v8 stack (POP+TIE) + whatever layer 3 lands. v8 validation
(12 rated games) runs in parallel — autopsy them when they land, feed rival
files. **Goal:** beat VladNet + AngelWASM, fix Blue, measurable Elo over v8.

**Laws (same as v8):** one variable per dose; full-horizon points;
deterministic (tiebreak by move index); every claim n>=8 both colors; per-chair
E-6 tables; kill-0 before gates; scoreboard row per gate run; NEVER merge
KILLed/ungated/neutral-without-decision doses; verify every finish on disk.

**Resourcing:** 2 fast lanes (implement+gate, Rust) + 1 cheap lane
(analysis/census, read-only md) + coordinator (merges, verification, roadmap).
24/7, no user prompts. Quota/429/403 = sleep 180s, retry same call x999.
Zero-on-disk-output 30+ min = re-seat. Garbled output + zero commits = failure:
salvage disk work, retry once, then coordinator takes the task or kills it.
NEVER checkout branches inside the main repo or another lane's worktree
(fresh `git worktree add` per lane, push append-only, no force-push).

## Lane queue (Elo order)

1. **K-CLASS reply hedge** (+40 est, opp model, VladNet priority). Dose:
   REPLY_CLASSES=3 root reply pass (top-priority / max-our-loss /
   max-their-gain, min of selection_adjusted). Kill-0: actual reply in
   3-class union <40% pooled or VladNet <20% (extend replay_match.rs).
   Full idea: research/v9-ideas.md #1.
2. **PROT-BLOCK** (+30, Blue chair, mirror scope). Dose: MESH_B_SEIZE table
   (D10-G10, G10-J11, D10-G13, G13-J11) behind Red-mesh-entry trigger; rule 6
   kills Red's entry-4. Kill-0: Rust 9/9 block + Red @16 ≤10.0 + our max_pop
   @9 ≤4.5. Idea #2. Conflicts MIRROR-ORACLE — never stack blind.
3. **CUT-YIELD** (+25). Dose: ZERO_CUT_PENALTY=1.0 x hz on broken +
   0-destroyed + 0-gain first actions. Kill-0: Rust replay flip census ≥10%
   of 113 zero-yield cuts (0 flips = Lane-E disease, kill). Idea #3.
4. **REINFORCE-LINES** (founder, unbreakable-first). Extend DENSE_BONUS toward
   enforced building behind 2+ shared-node walls, then expand far / bank
   behind (GB-like, AngelWASM-like). Needs kill-0 first: unbreakable-share vs
   win census on corpus games. Q2 lineage.
5. **MIRROR-ORACLE** (+20, Blue; ONLY if PROT-BLOCK dies). Offline-solved
   table vs recorded Red line; kill-0: beam-64, Blue @30 ≥35 in ≥8/9. Idea #4.
6. **SCRIPT-LOCK** (+10, mirror-only). OPP_LINES table; needs Phase-4 corpus
   (≥8 games/bot) for top-bot claims. Idea #5.
7. **ANGEL-CIRCLE census** (new, unscored). How often AngelWASM circles us,
   what shape, what breaks it. Analysis lane first, dose only after kill-0.
8. **P1 pair planning** (idea-backlog.md): pair-regret census running as v8
   item; D1 only if regret big+frequent.

## Killed — do not build (v9-ideas.md dead ends + v8 list)

Pre-emptive Blue cuts (0.0 at 0/55) · LEAD-RISK DOOM raise · RATE-GATED DOOM ·
junk-action-1 theory · all v8 kills (v8-roadmap.md: GB-book, M10@2.2k,
corridor-infra, P/W rebuild, naive depth-3, symmetry pruning, Cluster-3
unmeasured work).

## Ship path (same as v8)

Gated winners stack one at a time with re-gate → M4 (≥7/10 + league/gauge) →
bench_timed + site_check → WASM → upload new bot version → rated validation →
tag v9. Site games: ~40min each, serial — queue early, autopsy on landing.
