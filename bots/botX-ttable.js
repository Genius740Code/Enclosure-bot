// botX-ttable.js — IDEAS idea 2: transposition / state cache.
// Within one turn, different action orders (A-then-B vs B-then-A) and duplicate
// (from,to) pairs across frontier nodes can converge on identical positions.
// Hash states canonically and reuse evals instead of re-scoring.
// API: bestTurn(state, budget). Exposes stats() for hit-rate reporting.
const E = require('../engine/engine.js');
const BOT = require('./bot.js');

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

function segKey(s) { return `${Math.round(s.from[0] * 1e9)},${Math.round(s.from[1] * 1e9)}>${Math.round(s.to[0] * 1e9)},${Math.round(s.to[1] * 1e9)}`; }
function stateHash(st) {
  const b = st.segments.blue.map(segKey).sort().join(';');
  const r = st.segments.red.map(segKey).sort().join(';');
  return `${st.turn}|${st.actionsRemaining}|B:${b}|R:${r}`;
}

let _hits = 0, _misses = 0;
function stats() { return { hits: _hits, misses: _misses, hitRate: (_hits + _misses) ? _hits / (_hits + _misses) : 0 }; }
function resetStats() { _hits = 0; _misses = 0; }

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

  // eval cache: finalStateHash -> {own, oppGain, val}
  const cache = new Map();
  const evalFinal = (finalSt) => {
    const h = stateHash(finalSt);
    const hit = cache.get(h);
    if (hit) { _hits++; return hit; }
    _misses++;
    const e = {
      own: finalSt.scores[color] - startScore,
      og: finalSt.scores[opp] - startOpp,
    };
    e.val = e.own - e.og * 0.15;
    cache.set(h, e);
    return e;
  };

  try { firstCands.sort((a, b) => BOT.quickScore(state, b) - BOT.quickScore(state, a)); } catch (e) {}
  let bestMoves = [{ from: firstCands[0].from, to: firstCands[0].to }];
  let bestFinal = firstCands[0].result;
  let bestVal = -Infinity;

  if (state.actionsRemaining === 1) {
    for (const c of firstCands) {
      if (Date.now() > tEnd) break;
      const v = evalFinal(c.result).val;
      if (v > bestVal) { bestVal = v; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
    }
    return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore };
  }

  for (const c of firstCands.slice(0, 25)) {
    if (Date.now() > tEnd) break;
    const v = evalFinal(c.result).val;
    if (v > bestVal) { bestVal = v; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
  }
  const K1 = 10, K2 = 25;
  // second-ply dedupe: skip generating/evaluating a second action whose
  // resulting state hash was already scored this turn (transposition hit).
  const seenSecond = new Set();
  for (const c1 of firstCands.slice(0, K1)) {
    if (Date.now() > tEnd) break;
    let second;
    try {
      second = BOT.singleActionCandidates(c1.result, budget);
      try { second.sort((a, b) => BOT.quickScore(c1.result, b) - BOT.quickScore(c1.result, a)); } catch (e) {}
      second = second.slice(0, K2);
    } catch (e) { continue; }
    for (const c2 of second) {
      if (Date.now() > tEnd) break;
      const h = stateHash(c2.result);
      if (seenSecond.has(h)) { _hits++; continue; }
      seenSecond.add(h);
      const v = evalFinal(c2.result).val;
      if (v > bestVal) {
        bestVal = v;
        bestMoves = [{ from: c1.from, to: c1.to }, { from: c2.from, to: c2.to }];
        bestFinal = c2.result;
      }
    }
    if (c1.result.actionsRemaining === 0) {
      const v = evalFinal(c1.result).val;
      if (v > bestVal) { bestVal = v; bestMoves = [{ from: c1.from, to: c1.to }]; bestFinal = c1.result; }
    }
  }
  return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore };
}

function bestTurn(state, budgetMs) {
  try {
    let b = 8000;
    if (typeof budgetMs === 'number') b = budgetMs;
    else if (budgetMs && typeof budgetMs === 'object' && typeof budgetMs.clockMs === 'number')
      b = require('./bot-tourney.js').adaptiveBudget(state, budgetMs);
    const r = bestTurnBudgeted(state, b);
    if (r && r.moves && r.moves.length) return r;
  } catch (e) {}
  try {
    const cands = BOT.singleActionCandidates(state, { nodeCap: 3 });
    if (cands.length) return { moves: [{ from: cands[0].from, to: cands[0].to }], finalState: cands[0].result, scoreGain: 0 };
  } catch (e) {}
  return null;
}

module.exports = { bestTurn, bestTurnBudgeted, stateHash, stats, resetStats };
