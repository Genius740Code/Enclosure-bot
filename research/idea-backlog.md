# v8 idea backlog (transcribed from founder chat; Phase 0 item — others pending)

## P1 — pair planning ("2 lines as one", 2026-09-30, founder)
On double-action turns the two actions are linked (setup+close, double cuts,
threat-threat) but the bot plans greedily per action: move 1 picked with a PV
that sees the pair, move 2 re-planned from scratch. Proposal: search PAIRS
(first action × follow-up) so linked moves are chosen jointly.
- Cost threat: 400² pairs unaffordable under 2s soft budget; needs beam
  pruning at both levels (coverage problem BEAM-SEED already fights).
- Relation: M8 combo valuation + double-threats (history); M7 reply-model gap
  (opp-modeling may matter more than self-pairing).
- Step-0 (queued): pair-regret census on our double turns — does the jointly
  best pair beat greedy-then-replan, how often? Kill-0: regret rare/small.
  D1 only if regret is big and frequent.

## E-0–E-8, ideas 1-5, §4/§5 — PENDING transcription (source: founder chat)
Placeholders; fill from chat history, one entry each with mechanism + kill-0.
