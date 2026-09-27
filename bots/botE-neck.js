// botE-neck.js — JOB 3 light probe (ENDGAME track). NEW file.
// botE-endgame.js + a "double-connect" reinforcement bonus in the 2-action
// eval (always on): when the two actions of a turn share an endpoint (chained
// or adjacent-reinforcing structure), the pair is preferred. Hypothesis under
// test: reinforced pairs survive break-heavy opponents (chall-aggro) better.
// Mechanical caveat (verified in engine.js): a cut removes ONE enemy edge and
// zeroes the loop it belongs to — redundancy inside the SAME loop does not
// protect banking; only out-of-reach distance does. This probe measures it.
const E = require('../engine/engine.js');
const BOT = require('./bot.js');

const REINFORCE_W = 2.0; // shared-endpoint bonus per (c1,c2) pair
const ENDGAME_MOVES = 12, DENIAL_MID = 0.15, DENIAL_END = 0.6, AREA_W_END = 0.5, BREAK_W_END = 4;

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }
function isEndgame(state) { return (state.lineLimit - state.moveNumber) <= ENDGAME_MOVES; }
function endgameQuickScore(state, cand) {
  const color = state.turn;
  const opp = oppOf(color);
  const areaGain = cand.result.areas[color] - state.areas[color];
  const enemyEdgesRemoved = state.segments[opp].length - cand.result.segments[opp].length;
  return areaGain * AREA_W_END + enemyEdgesRemoved * BREAK_W_END;
}
// Double-connect test: do the two actions of the turn share an endpoint?
function connects(a, b) {
  const eq = (p, q) => p[0] === q[0] && p[1] === q[1];
  return eq(a.to, b.from) || eq(a.to, b.to) || eq(a.from, b.from) || eq(a.from, b.to);
}

function bestTurnBudgeted(state, budgetMs, weights) {
  const color = state.turn;
  const t0 = Date.now();
  const startScore = state.scores[color];
  let budget;
  try { budget = BOT.searchBudget(state); }
  catch (e) { budget = { nodeCap: 7 }; }
  let firstCands;
  try { firstCands = BOT.singleActionCandidates(state, budget); }
  catch (e) { return null; }
  if (!firstCands.length) return null;

  const end = !!weights;
  const denial = end ? DENIAL_END : DENIAL_MID;
  const rank = (cands, st) => {
    try {
      if (end) cands.sort((a, b) => endgameQuickScore(st, b) - endgameQuickScore(st, a));
      else cands.sort((a, b) => BOT.quickScore(st, b) - BOT.quickScore(st, a));
    } catch (e) { /* keep order */ }
    return cands;
  };
  rank(firstCands, state);

  let bestMoves = [{ from: firstCands[0].from, to: firstCands[0].to }];
  let bestFinal = firstCands[0].result;
  let bestVal = -Infinity;
  if (state.actionsRemaining === 1) {
    for (const c of firstCands) {
      if (Date.now() - t0 > budgetMs) break;
      const val = c.result.scores[color] - startScore
        - (c.result.scores[oppOf(color)] - state.scores[oppOf(color)]) * denial;
      if (val > bestVal) { bestVal = val; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
    }
    return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore };
  }

  for (const c of firstCands.slice(0, 25)) {
    if (Date.now() - t0 > budgetMs) break;
    const v0 = c.result.scores[color] - startScore;
    if (v0 > bestVal) { bestVal = v0; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
  }
  const K1 = 10, K2 = 25;
  for (const c1 of firstCands.slice(0, K1)) {
    if (Date.now() - t0 > budgetMs) break;
    let second;
    try { second = rank(BOT.singleActionCandidates(c1.result, budget), c1.result).slice(0, K2); }
    catch (e) { continue; }
    for (const c2 of second) {
      if (Date.now() - t0 > budgetMs) break;
      const ownGain = c2.result.scores[color] - startScore;
      const oppGain = c2.result.scores[oppOf(color)] - state.scores[oppOf(color)];
      const val = ownGain - oppGain * denial + (connects(c1, c2) ? REINFORCE_W : 0);
      if (val > bestVal) {
        bestVal = val;
        bestMoves = [{ from: c1.from, to: c1.to }, { from: c2.from, to: c2.to }];
        bestFinal = c2.result;
      }
    }
    if (c1.result.actionsRemaining === 0) {
      const ownGain = c1.result.scores[color] - startScore;
      if (ownGain > bestVal) { bestVal = ownGain; bestMoves = [{ from: c1.from, to: c1.to }]; bestFinal = c1.result; }
    }
  }
  return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore };
}

function bestTurn(state, budgetMs) {
  try {
    let b = 8000;
    if (typeof budgetMs === 'number') b = budgetMs;
    else if (budgetMs && typeof budgetMs === 'object') b = adaptiveBudget(state, budgetMs);
    else if (budgetMs === undefined) b = adaptiveBudget(state, null);
    const weights = isEndgame(state) ? { endgame: true } : null;
    const r = bestTurnBudgeted(state, b, weights);
    if (r && r.moves && r.moves.length) return r;
  } catch (e) { /* fall through */ }
  try {
    const cands = BOT.singleActionCandidates(state, { nodeCap: 3 });
    if (cands.length) return { moves: [{ from: cands[0].from, to: cands[0].to }], finalState: cands[0].result, scoreGain: 0 };
  } catch (e) { /* give up */ }
  return null;
}

function adaptiveBudget(state, clock) {
  const m = state.moveNumber;
  let b = m <= 6 ? 1500 : m <= 80 ? 8000 : 4000;
  if (clock && typeof clock.clockMs === 'number') {
    if (clock.clockMs < 20000) b = Math.min(b, 1500);
    else if (clock.clockMs < 60000) b = Math.min(b, 4000);
  }
  return b;
}

module.exports = { bestTurn, bestTurnBudgeted, adaptiveBudget, isEndgame, connects };
