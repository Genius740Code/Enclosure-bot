// bot-tourney.js — tournament-safe wrapper. NEVER throws, NEVER returns an
// illegal move, respects a millisecond budget (checks clock between
// candidates, so effective granularity is one candidate evaluation).
// Strategy: same greedy 2-ply as bot.js (early area first, break bonus,
// slight opponent denial), but time-boxed with a fallback chain:
//   time-boxed 2-ply -> best 1-ply so far -> first legal move -> null(timeout)
const E = require('./engine.js');
const BOT = require('./bot.js');

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

// Yielding 2-ply: iterate first-action shortlist, keep best within budget.
function bestTurnBudgeted(state, budgetMs) {
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

  const rank = (cands, st) => {
    try { cands.sort((a, b) => BOT.quickScore(st, b) - BOT.quickScore(st, a)); }
    catch (e) { /* keep order */ }
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
        - (c.result.scores[oppOf(color)] - state.scores[oppOf(color)]) * 0.15;
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
      const val = ownGain - oppGain * 0.15;
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
    const r = bestTurnBudgeted(state, b);
    if (r && r.moves && r.moves.length) return r;
  } catch (e) { /* fall through to emergency legal move */ }
  // Emergency: first legal single action, else null (= caller must timeout).
  try {
    const cands = BOT.singleActionCandidates(state, { nodeCap: 3 });
    if (cands.length) return { moves: [{ from: cands[0].from, to: cands[0].to }], finalState: cands[0].result, scoreGain: 0 };
  } catch (e) { /* give up */ }
  return null;
}

// Adaptive clock for 1min + 15s/move. Early moves are near-forced (cheap),
// mid-game area decisions compound for dozens of turns (spend here),
// late game is mostly decided (save time). Never spend into the reserve.
// Pass { clockMs } (your remaining ms) or nothing for phase defaults.
function adaptiveBudget(state, clock) {
  const m = state.moveNumber;
  let b = m <= 6 ? 1500 : m <= 80 ? 8000 : 4000;
  if (clock && typeof clock.clockMs === 'number') {
    if (clock.clockMs < 20000) b = Math.min(b, 1500); // reserve: never flag
    else if (clock.clockMs < 60000) b = Math.min(b, 4000);
  }
  return b;
}

module.exports = { bestTurn, bestTurnBudgeted, adaptiveBudget };

// Self-test: full game vs itself with per-turn timing, budget 8s.
if (require.main === module) {
  const budget = parseInt(process.argv[2] || '8000', 10);
  let s = E.createInitialState();
  let turns = 0, worst = 0;
  while (s.moveNumber < s.lineLimit) {
    const t0 = Date.now();
    const turn = bestTurn(s, budget);
    const dt = Date.now() - t0;
    worst = Math.max(worst, dt);
    if (!turn) { s = E.applyTimeout(s); continue; }
    let r = s;
    for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
    s = r;
    turns++;
    if (turns % 10 === 0 || s.moveNumber >= s.lineLimit)
      console.log(`turn ${turns}: moveNumber=${s.moveNumber} took ${dt}ms scores=${JSON.stringify(s.scores)}`);
    if (turns > 200) break;
  }
  console.log('final:', s.scores, 'turns:', turns, 'worst turn ms:', worst);
}
