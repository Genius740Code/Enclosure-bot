// botE-endgame.js — ENDGAME track (horizon-aware). NEW file; tournament-locked
// files untouched. Same time-boxed 2-ply as bot-tourney.js with an endgame
// mode: when (lineLimit - moveNumber) <= 12:
//   (a) future-area potential weight 2 -> 0.5 in candidate ranking
//       (only banked score matters; area built now banks ~10 more turn-ends max)
//   (b) opponent-denial weight 0.15 -> 0.6 in the eval (denying their banked
//       area / grabbing open area is everything in the last 12 moves)
//   (c) break bonus 1.5 -> 4 in candidate ranking (an endgame break permanently
//       removes enemy banking — no time to rebuild)
// Non-endgame play is byte-identical to bot-tourney.js (verified by verifyE.js:
// identical moves on early/mid test states at full budget). Same fallback chain:
//   time-boxed 2-ply -> best 1-ply so far -> first legal move -> null(timeout)
const E = require('../engine/engine.js');
const BOT = require('./bot.js');

const ENDGAME_MOVES = 12;  // (lineLimit - moveNumber) <= this -> endgame mode
const DENIAL_MID = 0.15;   // = bot-tourney.js (byte-identical outside endgame)
const DENIAL_END = 0.6;    // endgame denial weight
const AREA_W_END = 0.5;    // endgame potential-area weight (was 2 via quickScore)
const BREAK_W_END = 4;     // endgame break weight (was 1.5 via quickScore)

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

function isEndgame(state) {
  return (state.lineLimit - state.moveNumber) <= ENDGAME_MOVES;
}

// Endgame candidate ranking: denial + breaks dominate; potential area ~0.
function endgameQuickScore(state, cand) {
  const color = state.turn;
  const opp = oppOf(color);
  const areaGain = cand.result.areas[color] - state.areas[color];
  const enemyEdgesRemoved = state.segments[opp].length - cand.result.segments[opp].length;
  return areaGain * AREA_W_END + enemyEdgesRemoved * BREAK_W_END;
}

// Yielding 2-ply: iterate first-action shortlist, keep best within budget.
// weights: null -> bot-tourney ranking/eval (byte-identical); else endgame mode.
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

  // Always lock in a 1-ply answer first (fallback if clock dies).
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

  // 2 actions: seed bestVal with best single action, then deepen while clock allows.
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
      const val = ownGain - oppGain * denial;
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
  } catch (e) { /* fall through to emergency legal move */ }
  // Emergency: first legal single action, else null (= caller must timeout).
  try {
    const cands = BOT.singleActionCandidates(state, { nodeCap: 3 });
    if (cands.length) return { moves: [{ from: cands[0].from, to: cands[0].to }], finalState: cands[0].result, scoreGain: 0 };
  } catch (e) { /* give up */ }
  return null;
}

// Adaptive clock — byte-identical to bot-tourney.js.
function adaptiveBudget(state, clock) {
  const m = state.moveNumber;
  let b = m <= 6 ? 1500 : m <= 80 ? 8000 : 4000;
  if (clock && typeof clock.clockMs === 'number') {
    if (clock.clockMs < 20000) b = Math.min(b, 1500); // reserve: never flag
    else if (clock.clockMs < 60000) b = Math.min(b, 4000);
  }
  return b;
}

module.exports = { bestTurn, bestTurnBudgeted, adaptiveBudget, isEndgame, endgameQuickScore };
