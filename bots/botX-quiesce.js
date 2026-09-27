// botX-quiesce.js — IDEAS idea 1: iterative deepening + time-boxed 2-ply with
// improved move ordering + opponent-reply quiescence (extend only on volatile
// finals: a break happened or area just closed).
// API: bestTurn(state, budget) like bot-tourney.js. Never throws, never illegal.
const E = require('../engine/engine.js');
const BOT = require('./bot.js');

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

// Improved quickScore: tourney's areaGain*2 + breaks*1.5 misses banked score
// (score only updates at turn end) and ignores "closing" moves that create
// score without area delta mid-turn. Add score delta + small centrality bonus
// (moves landing nearer own centroid = more likely to close loops).
function quickScore2(state, cand) {
  const color = state.turn;
  const opp = oppOf(color);
  let v = 0;
  try {
    v += (cand.result.areas[color] - state.areas[color]) * 2;
    v += (state.segments[opp].length - cand.result.segments[opp].length) * 1.5;
    v += (cand.result.scores[color] - state.scores[color]) * 1.0;
    v -= (cand.result.scores[opp] - state.scores[opp]) * 0.3;
  } catch (e) { /* ignore */ }
  return v;
}

// Volatile final = worth extending: a break occurred on either side or fresh
// area/score appeared (tactics still hanging). Quiet finals stand pat.
function isVolatile(before, after, color) {
  const opp = oppOf(color);
  if (before.segments[opp].length !== after.segments[opp].length) return true;
  if (before.segments[color].length + 2 < after.segments[color].length) return true;
  if (Math.abs(after.areas[color] - before.areas[color]) > 1e-9) return true;
  if (Math.abs(after.areas[opp] - before.areas[opp]) > 1e-9) return true;
  return false;
}

// Quiescence: opponent's best immediate reply value from `state` (opponent to
// move), bounded 1-ply over a narrow slice. Returns max (oppScoreGain + breakBonus).
function oppReplyValue(state, tEnd, oppBudget) {
  const opp = state.turn;
  let cands;
  try { cands = BOT.singleActionCandidates(state, oppBudget); }
  catch (e) { return 0; }
  if (!cands.length) return 0;
  let best = -Infinity;
  const N = Math.min(cands.length, 40);
  // rank cheaply first with base quickScore, only fully evaluate top N
  try { cands.sort((a, b) => BOT.quickScore(state, b) - BOT.quickScore(state, a)); } catch (e) {}
  for (let i = 0; i < N; i++) {
    if (Date.now() > tEnd) break;
    const c = cands[i];
    const gain = (c.result.scores[opp] - state.scores[opp])
      + (state.segments[oppOf(opp)].length - c.result.segments[oppOf(opp)].length) * 1.0;
    if (gain > best) best = gain;
  }
  return best === -Infinity ? 0 : best;
}

function bestTurnBudgeted(state, budgetMs) {
  const color = state.turn;
  const opp = oppOf(color);
  const t0 = Date.now();
  const tEnd = t0 + budgetMs;
  const startScore = state.scores[color];
  let budget;
  try { budget = BOT.searchBudget(state); } catch (e) { budget = { nodeCap: 7 }; }
  let firstCands;
  try { firstCands = BOT.singleActionCandidates(state, budget); } catch (e) { return null; }
  if (!firstCands.length) return null;

  const rank = (cands, st) => {
    try { cands.sort((a, b) => quickScore2(st, b) - quickScore2(st, a)); } catch (e) {}
    return cands;
  };
  rank(firstCands, state);

  const baseVal = (finalSt) =>
    (finalSt.scores[color] - startScore) - (finalSt.scores[opp] - state.scores[opp]) * 0.15;

  // ID depth 1: full 1-ply sweep -> fallback answer locked in.
  let bestMoves = [{ from: firstCands[0].from, to: firstCands[0].to }];
  let bestFinal = firstCands[0].result;
  let bestVal = -Infinity;
  if (state.actionsRemaining === 1) {
    for (const c of firstCands) {
      if (Date.now() > tEnd) break;
      const v = baseVal(c.result);
      if (v > bestVal) { bestVal = v; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
    }
    // quiescence extension on volatile best only (bounded: 1 opp reply search)
    if (Date.now() < tEnd && isVolatile(state, bestFinal, color)) {
      const q = oppReplyValue(bestFinal, tEnd, { nodeCap: 7 });
      bestVal -= q * 0.5; // recorded for transparency; move stands (1 action forced)
    }
    return { moves: bestMoves, finalState: bestFinal, scoreGain: bestFinal.scores[color] - startScore };
  }

  // ID depth 1 (2-action turns): seed with best single actions.
  for (const c of firstCands.slice(0, 25)) {
    if (Date.now() > tEnd) break;
    const v0 = baseVal(c.result);
    if (v0 > bestVal) { bestVal = v0; bestMoves = [{ from: c.from, to: c.to }]; bestFinal = c.result; }
  }

  // ID depth 2: deepen shortlist while clock allows. Wider than tourney (12x30)
  // because ordering is better; time-box keeps it safe.
  const K1 = 12, K2 = 30;
  const scored = []; // {moves, final, val, volatile} for quiescence stage
  for (const c1 of firstCands.slice(0, K1)) {
    if (Date.now() > tEnd) break;
    let second;
    try { second = rank(BOT.singleActionCandidates(c1.result, budget), c1.result).slice(0, K2); }
    catch (e) { continue; }
    for (const c2 of second) {
      if (Date.now() > tEnd) break;
      const v = baseVal(c2.result);
      if (v > bestVal) {
        bestVal = v;
        bestMoves = [{ from: c1.from, to: c1.to }, { from: c2.from, to: c2.to }];
        bestFinal = c2.result;
      }
      scored.push({ m: [{ from: c1.from, to: c1.to }, { from: c2.from, to: c2.to }], f: c2.result, v });
    }
    if (c1.result.actionsRemaining === 0) {
      const v = baseVal(c1.result);
      if (v > bestVal) { bestVal = v; bestMoves = [{ from: c1.from, to: c1.to }]; bestFinal = c1.result; }
    }
  }

  // Quiescence stage: re-rank only the top-4 by base val; extend volatile ones
  // with one bounded opponent-reply search, penalize by 0.5*reply. Quiet ones
  // stand pat (no extra cost). This is the horizon-effect fix.
  try {
    scored.sort((a, b) => b.v - a.v);
    let qBest = { moves: bestMoves, final: bestFinal, v: bestVal };
    const QN = Math.min(4, scored.length);
    for (let i = 0; i < QN; i++) {
      if (Date.now() > tEnd) break;
      const s = scored[i];
      let v = s.v;
      if (isVolatile(state, s.f, color)) {
        const q = oppReplyValue(s.f, tEnd, { nodeCap: 7 });
        v -= q * 0.5;
      }
      if (v > qBest.v) qBest = { moves: s.m, final: s.f, v };
    }
    bestMoves = qBest.moves; bestFinal = qBest.final; bestVal = qBest.v;
  } catch (e) { /* keep deepen answer */ }

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

module.exports = { bestTurn, bestTurnBudgeted, quickScore2, isVolatile, oppReplyValue };
