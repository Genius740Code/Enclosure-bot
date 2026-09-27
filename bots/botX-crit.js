// botX-crit.js — IDEAS idea 4: variance-based time control ("criticality").
// Prototype: after a cheap 1-ply sweep, measure the gap between best and
// runner-up (and top-K spread). Clear-best -> move fast, narrow/shallow search.
// Critical (small gap or high variance) -> spend full budget, widen K1/K2.
// API: bestTurn(state, budget) + criticality(state) helper. Never throws.
const E = require('../engine/engine.js');
const BOT = require('./bot.js');
const TQ = require('./bot-tourney.js');

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

// Returns {gap, spread, crit} in [0,1]. crit~1 = critical (spend), ~0 = clear (save).
function criticality(state, topVals) {
  if (!topVals || topVals.length < 2) return { gap: 0, spread: 0, crit: 0.5 };
  const vals = topVals.slice(0, 8);
  const gap = vals[0] - vals[1];
  const spread = vals[0] - vals[vals.length - 1];
  // Normalize: gaps are in score units; typical decisive gap ~5+, noise ~<1.
  // crit = 1 when gap tiny, 0 when gap huge.
  const crit = Math.max(0, Math.min(1, 1 - gap / 5));
  return { gap, spread, crit };
}

function bestTurnBudgeted(state, budgetMs) {
  const color = state.turn;
  const opp = oppOf(color);
  const t0 = Date.now();
  const tEnd = t0 + budgetMs;
  const startScore = state.scores[color];
  const startOpp = state.scores[opp];
  let budget;
  try { budget = BOT.searchBudget(state); } catch (e) { budget = { nodeCap: 7 }; }
  let firstCands;
  try { firstCands = BOT.singleActionCandidates(state, budget); } catch (e) { return null; }
  if (!firstCands.length) return null;
  try { firstCands.sort((a, b) => BOT.quickScore(state, b) - BOT.quickScore(state, a)); } catch (e) {}

  const val1 = (c) => (c.result.scores[color] - startScore) - (c.result.scores[opp] - startOpp) * 0.15;

  // Cheap 1-ply sweep (already have results from candidate gen) -> criticality.
  const topVals = firstCands.slice(0, 8).map(val1).sort((a, b) => b - a);
  const { gap, spread, crit } = criticality(state, topVals);

  // Budget multiplier: critical -> use all of it (and widen); clear-best ->
  // cap depth (effective early exit). Multiplier on K widths + early-stop.
  let K1 = 10, K2 = 25, tEff = tEnd;
  if (crit < 0.2) { K1 = 5; K2 = 12; tEff = t0 + budgetMs * 0.35; }       // clear-best: move fast
  else if (crit > 0.6) { K1 = 14; K2 = 32; tEff = tEnd; }                // critical: spend more
  // else default widths, full budget.

  let bestMoves = [{ from: firstCands[0].from, to: firstCands[0].to }];
  let bestFinal = firstCands[0].result;
  let bestVal = -Infinity;

  if (state.actionsRemaining === 1) {
    for (const c of firstCands) {
      if (Date.now() > tEff) break;
      const v = val1(c);
      if (v > bestVal) { bestVal = v; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
    }
    return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore, _crit: crit, _gap: gap };
  }

  for (const c of firstCands.slice(0, 25)) {
    if (Date.now() > tEff) break;
    const v = val1(c);
    if (v > bestVal) { bestVal = v; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
  }
  for (const c1 of firstCands.slice(0, K1)) {
    if (Date.now() > tEff) break;
    let second;
    try {
      second = BOT.singleActionCandidates(c1.result, budget);
      try { second.sort((a, b) => BOT.quickScore(c1.result, b) - BOT.quickScore(c1.result, a)); } catch (e) {}
      second = second.slice(0, K2);
    } catch (e) { continue; }
    for (const c2 of second) {
      if (Date.now() > tEff) break;
      const ownGain = c2.result.scores[color] - startScore;
      const oppGain = c2.result.scores[opp] - startOpp;
      const v = ownGain - oppGain * 0.15;
      if (v > bestVal) {
        bestVal = v;
        bestMoves = [{ from: c1.from, to: c1.to }, { from: c2.from, to: c2.to }];
        bestFinal = c2.result;
      }
    }
    if (c1.result.actionsRemaining === 0) {
      const v = val1(c1);
      if (v > bestVal) { bestVal = v; bestMoves = [{ from: c1.from, to: c1.to }]; bestFinal = c1.result; }
    }
  }
  return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore, _crit: crit, _gap: gap, _spread: spread };
}

function bestTurn(state, budgetMs) {
  try {
    let b = 8000;
    if (typeof budgetMs === 'number') b = budgetMs;
    else if (budgetMs && typeof budgetMs === 'object' && typeof budgetMs.clockMs === 'number')
      b = TQ.adaptiveBudget(state, budgetMs);
    const r = bestTurnBudgeted(state, b);
    if (r && r.moves && r.moves.length) return r;
  } catch (e) {}
  try {
    const cands = BOT.singleActionCandidates(state, { nodeCap: 3 });
    if (cands.length) return { moves: [{ from: cands[0].from, to: cands[0].to }], finalState: cands[0].result, scoreGain: 0 };
  } catch (e) {}
  return null;
}

module.exports = { bestTurn, bestTurnBudgeted, criticality };
