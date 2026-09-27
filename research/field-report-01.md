# Field report 01 (2026-09-27, from live testing vs old bot, loss)

## Observed
- New bot lost first game vs old bot.
- Recurring pattern: enemy holds a big area; we break it; enemy rebuilds
  immediately; cycle repeats while the enemy "slowly but surely does other
  stuff" (develops elsewhere) — we lose the long game.
- Occasional "zugzwang": positions where every move looks bad, bot drifts.
- Sometimes good (not uniformly broken).

## Working diagnosis (unverified, for lanes to test)
- Our reply search models only ONE enemy action, but an enemy turn is TWO
  actions (reclose + develop). We systematically underprice their recovery
  by one action: cuts look better in-search than they play out.
- Lane A: consider modeling both enemy reply actions on cut lines
  (selective 3-ply), or a rebuild-discount on cut credit.
- Lane B: consider crediting only SURVIVING destruction (post-reply enemy
  area, which analyze() already uses — check whether ranked()'s immediate
  destroyed bonus over-promotes slap lines into the top-8), and a
  meaningful-break threshold (ignore sub-1.0-area cuts for bonus purposes).
- Zugzwang handling: when all candidates fall within epsilon, tiebreak by
  mobility (reply count) instead of pure index; consider risk-aware
  selection (behind = variance, ahead = safety).
- Need: game link / which "old bot" (Scout? previous Riposte build?) for
  Lane C autopsy.
