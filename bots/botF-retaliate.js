// botF-retaliate.js — Track F RETALIATION PATCH (see findings-retaliate.md).
// Byte-parallel copy of bot-tourney.js with ONLY the valuation changed.
// Widths (K1=10/K2=25/shortlist 25), search budgets (BOT.searchBudget) and the
// fallback chain (time-boxed 2-ply -> 1-ply -> first legal -> null) are
// IDENTICAL to bot-tourney.js. NEVER throws, NEVER returns an illegal move.
// Valuation changes vs bot-tourney.js:
//   (a) quickScore break term: enemyEdgesRemoved * max(1.5, enemyAreaDelta *
//       turnsLeft/40), where enemyAreaDelta is the area the break takes away
//       from the enemy (state.areas[opp] - result.areas[opp], i.e. the banked
//       loop that collapses) and turnsLeft = lineLimit - moveNumber is the
//       remaining payout horizon. bot-tourney valued breaks flat 1.5, so it
//       never retaliated against banking loops (P0-1..P0-4).
//   (b) opponent-denial weight 0.15 -> 0.35 at both valuation sites (the
//       1-action-terminal branch and the 2-ply branch share this knob).
//   (c) contact-avoidance: a FIRST action whose new endpoint lands within 1.5
//       of an enemy node is penalized -1.0 unless it breaks something (graze
//       only when it pays). NOTE: an edge that geometrically touches an enemy
//       node always breaks one of that node's segments (or is illegal), so the
//       only non-breaking contact is adjacency — exactly what neck/aggro
//       graze. Second-ply ranking is not penalized (spec: first actions).
const E = require('../engine/engine.js');
const BOT = require('./bot.js');

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

// (c) helper: does the candidate's new endpoint land within 1.5 of an enemy
// node (orthogonal or diagonal adjacency on the integer lattice)?
function touchesEnemyNode(state, opp, cand) {
  for (const n of state.nodes[opp]) {
    if (Math.hypot(cand.to[0] - n[0], cand.to[1] - n[1]) <= 1.5) return true;
  }
  return false;
}

// Track F quickScore: bot.js quickScore shape + scaled break term + contact
// penalty on FIRST actions only (isFirst=false for second-ply ranking).
function quickScoreF(state, cand, isFirst) {
  const color = state.turn;
  const opp = oppOf(color);
  const areaGain = cand.result.areas[color] - state.areas[color];
  const enemyEdgesRemoved = state.segments[opp].length - cand.result.segments[opp].length;
  const enemyAreaDelta = state.areas[opp] - cand.result.areas[opp]; // >0 when the break pops enemy banked area
  const turnsLeft = state.lineLimit - state.moveNumber;
  let v = areaGain * 2 + enemyEdgesRemoved * Math.max(1.5, enemyAreaDelta * turnsLeft / 40);
  if (isFirst && enemyEdgesRemoved === 0 && touchesEnemyNode(state, opp, cand)) v -= 1.0;
  return v;
}

// Yielding 2-ply: identical structure to bot-tourney.js bestTurnBudgeted.
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

  const rank = (cands, st, isFirst) => {
    try { cands.sort((a, b) => quickScoreF(st, b, isFirst) - quickScoreF(st, a, isFirst)); }
    catch (e) { /* keep order */ }
    return cands;
  };
  rank(firstCands, state, true);

  // Always lock in a 1-ply answer first (fallback if clock dies).
  let bestMoves = [{ from: firstCands[0].from, to: firstCands[0].to }];
  let bestFinal = firstCands[0].result;
  let bestVal = -Infinity;
  if (state.actionsRemaining === 1) {
    for (const c of firstCands) {
      if (Date.now() - t0 > budgetMs) break;
      const val = c.result.scores[color] - startScore
        - (c.result.scores[oppOf(color)] - state.scores[oppOf(color)]) * 0.35;
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
    try { second = rank(BOT.singleActionCandidates(c1.result, budget), c1.result, false).slice(0, K2); }
    catch (e) { continue; }
    for (const c2 of second) {
      if (Date.now() - t0 > budgetMs) break;
      const ownGain = c2.result.scores[color] - startScore;
      const oppGain = c2.result.scores[oppOf(color)] - state.scores[oppOf(color)];
      const val = ownGain - oppGain * 0.35;
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

// Adaptive clock — identical to bot-tourney.js.
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
