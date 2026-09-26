// botX-ponder.js — IDEAS idea 3: pondering prototype (prediction + validation
// logic ONLY; no harness/worker code, per mission constraints).
//
// Design (fits "bestTurn called only on our turn", single-threaded Node):
//   During OUR turn we already compute opponent-reply quiescence. Cache the
//   predicted opponent reply: { oppStateHash -> predictedReply {from,to} } plus
//   our pre-searched response. On our NEXT turn, validate: if actual state hash
//   matches a predicted state hash, reuse the pre-searched response instantly
//   (saving a full 2-ply); else discard and search normally.
// Worker-thread protocol sketch (if harness ever supports it):
//   main thread: after our move, post {predictedOppState} to worker; worker runs
//   preSearchResponse() and posts back {hash, response}. On our turn, main
//   thread validates hash and either reuses or searches. No shared memory needed
//   beyond the state JSON + hash.
//
// This file implements + tests the prediction/validation core:
//   predictOpponentReply(state) - opponent's quickScore-best 1-ply (their most
//     likely reply under the assumption they play like bot-tourney).
//   preSearchResponse(predState) - our budgeted 1-ply best response from predState.
//   validate(actualState, prediction) - hash equality check.
const E = require('./engine.js');
const BOT = require('./bot.js');
const TT = require('./botX-ttable.js');

function oppOf(c) { return c === 'blue' ? 'red' : 'blue'; }

// Most likely opponent reply: their quickScore-best single action.
function predictOpponentReply(state) {
  const opp = state.turn;
  let cands;
  try { cands = BOT.singleActionCandidates(state, { nodeCap: 10 }); } catch (e) { return null; }
  if (!cands.length) return null;
  try { cands.sort((a, b) => BOT.quickScore(state, b) - BOT.quickScore(state, a)); } catch (e) {}
  const c = cands[0];
  return { from: c.from, to: c.to, result: c.result, hash: TT.stateHash(c.result) };
}

// Pre-search our response from the predicted state (bounded 1-ply sweep;
// full 2-ply would also be possible with more ponder time).
function preSearchResponse(predResultState, budgetMs) {
  const tEnd = Date.now() + (budgetMs || 500);
  const color = predResultState.turn; // us to move in predicted state
  const opp = oppOf(color);
  const s0 = predResultState.scores[color], o0 = predResultState.scores[opp];
  let cands;
  try { cands = BOT.singleActionCandidates(predResultState, BOT.searchBudget(predResultState)); }
  catch (e) { return null; }
  if (!cands.length) return null;
  let best = cands[0], bestVal = -Infinity;
  const K = Math.min(cands.length, 25);
  try { cands.sort((a, b) => BOT.quickScore(predResultState, b) - BOT.quickScore(predResultState, a)); } catch (e) {}
  for (let i = 0; i < K; i++) {
    if (Date.now() > tEnd) break;
    const c = cands[i];
    const v = (c.result.scores[color] - s0) - (c.result.scores[opp] - o0) * 0.15;
    if (v > bestVal) { bestVal = v; best = c; }
  }
  return { from: best.from, to: best.to, val: bestVal };
}

function validate(actualState, prediction) {
  if (!prediction || !actualState) return false;
  try { return TT.stateHash(actualState) === prediction.hash; } catch (e) { return false; }
}

// Full ponder cycle helper used by the hit-rate probe below.
function ponderCycle(oppStateBeforeReply, actualStateAfterReply) {
  const pred = predictOpponentReply(oppStateBeforeReply);
  if (!pred) return { hit: false };
  const t0 = Date.now();
  const resp = preSearchResponse(pred.result, 300);
  const ponderMs = Date.now() - t0;
  return { hit: validate(actualStateAfterReply, pred), ponderMs };
}

// bestTurn: normal tourney-equivalent search (pondering needs harness support
// to run between turns; here we just play the base game so self-play tests the
// underlying move quality, while hit-rate is measured separately).
function bestTurn(state, budgetMs) {
  try {
    const T = require('./bot-tourney.js');
    let b = 8000;
    if (typeof budgetMs === 'number') b = budgetMs;
    else if (budgetMs && typeof budgetMs === 'object' && typeof budgetMs.clockMs === 'number')
      b = T.adaptiveBudget(state, budgetMs);
    const r = T.bestTurnBudgeted(state, b);
    if (r && r.moves && r.moves.length) return r;
  } catch (e) {}
  return null;
}

module.exports = { bestTurn, predictOpponentReply, preSearchResponse, validate, ponderCycle };

// Probe: measure prediction hit-rate during a tourney-vs-tourney game.
// `node botX-ponder.js [games=3] [budgetMs=300]`
if (require.main === module) {
  const T = require('./bot-tourney.js');
  const games = parseInt(process.argv[2] || '3', 10);
  const budget = parseInt(process.argv[3] || '300', 10);
  let hits = 0, total = 0, savedMs = 0;
  for (let g = 0; g < games; g++) {
    let s = E.createInitialState();
    let guard = 0;
    while (s.moveNumber < s.lineLimit && guard++ < 400) {
      const before = s;
      const turn = T.bestTurn(s, budget);
      let after;
      if (!turn) { after = E.applyTimeout(before); }
      else { let r = before; for (const m of turn.moves) r = E.applyMove(r, m.from, m.to); after = r; }
      // ponder: predict reply to `before`... actually predict from opponent's
      // perspective: prediction happens on opponent's turn. Simulate: from `after`
      // (opponent to move), predict their reply, then check against next actual.
      if (after.moveNumber < after.lineLimit) {
        const pred = predictOpponentReply(after);
        if (pred) {
          const t0 = Date.now();
          const resp = preSearchResponse(pred.result, 200);
          const ms = Date.now() - t0;
          const turn2 = T.bestTurn(after, budget);
          let actual;
          if (!turn2) actual = E.applyTimeout(after);
          else { let r = after; for (const m of turn2.moves) r = E.applyMove(r, m.from, m.to); actual = r; }
          total++;
          // hit = predicted single reply matches actual first action's resulting
          // state after 1 action? Only comparable when actual turn was 1 action.
          // Approximate: compare predicted result hash vs actual-after-first-action.
          if (turn2 && turn2.moves.length === 1) {
            try {
              const oneStep = E.applyMove(after, turn2.moves[0].from, turn2.moves[0].to);
              if (TT.stateHash(oneStep) === pred.hash) { hits++; savedMs += ms; }
            } catch (e) {}
          }
          s = actual;
          continue;
        }
      }
      s = after;
    }
  }
  console.log(`ponder probe: ${hits}/${total} exact-reply hits (${(100 * hits / Math.max(1, total)).toFixed(1)}%), ponderCost~${savedMs}ms saved-on-hit`);
}
