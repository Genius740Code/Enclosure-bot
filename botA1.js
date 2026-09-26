// bot.js — heuristic turn-search bot for "Enclosure", built directly on the
// site's own extracted rules engine (engine.js), so legality + scoring are
// guaranteed exact (no re-implemented/approximate rules).
const E = require('./engine.js');

// Full 7x7 box (48 offsets) vs. a coarser subset used once node counts grow,
// to keep per-candidate cost (which includes a real Dr recompute) bounded.
const OFFSETS_FULL = [];
for (let dx = -3; dx <= 3; dx++)
  for (let dy = -3; dy <= 3; dy++)
    if (dx !== 0 || dy !== 0) OFFSETS_FULL.push([dx, dy]);

const OFFSETS_COARSE = OFFSETS_FULL.filter(([dx, dy]) =>
  Math.max(Math.abs(dx), Math.abs(dy)) === 3 ||           // full reach (perimeter of box) is usually most valuable
  (Math.abs(dx) + Math.abs(dy)) % 2 === 0                  // + a checkerboard sample of the interior
);

function key(p) { return p[0] + ',' + p[1]; }

// Degree = how many of the player's own segments touch a node — used as a
// crude "is this node still a useful frontier to expand from" proxy so we
// don't waste the (expensive) candidate budget on deeply interior nodes.
function frontierNodes(state, color, cap) {
  const nodes = state.nodes[color];
  const segs = state.segments[color];
  const deg = nodes.map(n => segs.reduce((c, s) => c + (E.pointsEqual(s.from, n) || E.pointsEqual(s.to, n) ? 1 : 0), 0));
  const idx = nodes.map((_, i) => i).sort((a, b) => deg[a] - deg[b]);
  return idx.slice(0, cap).map(i => nodes[i]);
}

// All legal single-action (from,to) candidates for the player to move,
// each returned with the resulting state. `cap`/`offsets` control how much
// of the true move space we search (see frontierNodes / OFFSETS_COARSE).
function singleActionCandidates(state, { nodeCap = Infinity, offsets = OFFSETS_FULL } = {}) {
  const color = state.turn;
  const fromNodes = Number.isFinite(nodeCap) ? frontierNodes(state, color, nodeCap) : state.nodes[color];
  const out = [];
  const seenTo = new Set(); // avoid duplicate (from,to) pairs across nodes with same coords
  for (const from of fromNodes) {
    for (const [dx, dy] of offsets) {
      const to = [from[0] + dx, from[1] + dy];
      if (to[0] < 0 || to[0] >= state.boardSize || to[1] < 0 || to[1] >= state.boardSize) continue;
      const k = key(from) + '|' + key(to);
      if (seenTo.has(k)) continue;
      seenTo.add(k);
      try {
        const result = E.applyMove(state, from, to);
        out.push({ from, to, result });
      } catch (e) { /* illegal, skip */ }
    }
  }
  return out;
}

// Cheap heuristic used ONLY to rank/prune candidates before spending a full
// second-ply search on them. Rewards: area already banked this action if the
// turn ended (rare mid-turn), new-own-area potential (approximated by area
// delta even mid-turn since Dr is recomputed every action), and breaking an
// enemy edge (removed edge count).
function quickScore(state, cand) {
  const color = state.turn;
  const opp = color === 'blue' ? 'red' : 'blue';
  const areaGain = cand.result.areas[color] - state.areas[color];
  const enemyEdgesRemoved = state.segments[opp].length - cand.result.segments[opp].length;
  return areaGain * 2 + enemyEdgesRemoved * 1.5;
}

const TOP_K_FIRST = 10;   // how many first actions get a full second-ply search
const TOP_K_SECOND = 25;  // how many second actions get fully scored (all are cheap here)

// --- Track A variant A1: 1-ply opponent-reply lookahead ---
// After scoring all (c1,c2) leaves with the base value, re-rank the top
// LOOKAHEAD_TOP leaves by subtracting W_OPP_REPLY * (opponent's best
// single-action quickScore reply, scanned on a coarse budget). Units roughly
// match: base val is end-of-turn score delta (~ current area), opp reply is
// areaGain*2 + breaks*1.5 for one action.
const LOOKAHEAD_TOP = 5;
const W_OPP_REPLY = 0.2;
const OPP_SCAN_BUDGET = { nodeCap: 7, offsets: OFFSETS_COARSE };

function oppBestReply(resultState) {
  const opp = resultState.turn; // after our turn ends, it's opponent's turn
  const cands = singleActionCandidates(resultState, OPP_SCAN_BUDGET);
  let best = 0;
  for (const c of cands) {
    const v = quickScore(resultState, c);
    if (v > best) best = v;
  }
  return best;
}

// As the board fills up, both candidate generation AND the engine's own
// Dr() recompute get more expensive per call, so we search a narrower slice
// of the true move space (fewer frontier nodes, coarser offset sampling) to
// keep total time roughly bounded across the whole game.
function searchBudget(state) {
  const n = state.nodes.blue.length + state.nodes.red.length;
  if (n <= 12) return { nodeCap: Infinity, offsets: OFFSETS_FULL };
  if (n <= 30) return { nodeCap: 16, offsets: OFFSETS_FULL };
  if (n <= 60) return { nodeCap: 10, offsets: OFFSETS_COARSE };
  return { nodeCap: 7, offsets: OFFSETS_COARSE };
}

// Picks the best full turn (1 or 2 actions) for the side to move.
// Returns { moves: [{from,to}, ...], finalState, scoreGain }
function bestTurn(state) {
  const color = state.turn;
  const startScore = state.scores[color];
  const budget = searchBudget(state);
  const firstCands = singleActionCandidates(state, budget);
  if (firstCands.length === 0) return null; // no legal move (shouldn't happen per game design)

  if (state.actionsRemaining === 1) {
    // Turn ends immediately after this action — real score delta is exact.
    let best = null, bestVal = -Infinity;
    for (const c of firstCands) {
      const val = c.result.scores[color] - startScore
                - (c.result.scores[color === 'blue' ? 'red' : 'blue'] - state.scores[color === 'blue' ? 'red' : 'blue']) * 0.15;
      if (val > bestVal) { bestVal = val; best = c; }
    }
    return { moves: [{ from: best.from, to: best.to }], finalState: best.result, scoreGain: best.result.scores[color] - startScore };
  }

  // actionsRemaining === 2: rank first actions cheaply, deep-check the top K.
  firstCands.sort((a, b) => quickScore(state, b) - quickScore(state, a));
  const shortlist = firstCands.slice(0, TOP_K_FIRST);

  let best = null, bestVal = -Infinity;
  const leaves = [];
  for (const c1 of shortlist) {
    const secondCands = singleActionCandidates(c1.result, budget);
    secondCands.sort((a, b) => quickScore(c1.result, b) - quickScore(c1.result, a));
    const shortlist2 = secondCands.slice(0, TOP_K_SECOND);
    for (const c2 of shortlist2) {
      const opp = color === 'blue' ? 'red' : 'blue';
      const ownGain = c2.result.scores[color] - startScore;
      const oppGain = c2.result.scores[opp] - state.scores[opp];
      const val = ownGain - oppGain * 0.15; // slight preference for denying opponent too
      leaves.push({ c1, c2, val, ownGain });
      if (val > bestVal) {
        bestVal = val;
        best = { moves: [{ from: c1.from, to: c1.to }, { from: c2.from, to: c2.to }], finalState: c2.result, scoreGain: ownGain };
      }
    }
    // also consider stopping after just 1 action if somehow better (rare, but cheap to check)
    if (c1.result.actionsRemaining === 0) {
      const ownGain = c1.result.scores[color] - startScore;
      if (ownGain > bestVal) { bestVal = ownGain; best = { moves: [{ from: c1.from, to: c1.to }], finalState: c1.result, scoreGain: ownGain }; }
    }
  }
  // A1: re-rank top leaves with opponent-reply lookahead (only when turn ended, i.e. opponent to move)
  leaves.sort((a, b) => b.val - a.val);
  const top = leaves.slice(0, LOOKAHEAD_TOP);
  let bestAdj = null, bestAdjVal = -Infinity;
  for (const L of top) {
    let adj = L.val;
    if (L.c2.result.turn !== color) adj -= W_OPP_REPLY * oppBestReply(L.c2.result);
    if (adj > bestAdjVal) {
      bestAdjVal = adj;
      bestAdj = { moves: [{ from: L.c1.from, to: L.c1.to }, { from: L.c2.from, to: L.c2.to }], finalState: L.c2.result, scoreGain: L.ownGain };
    }
  }
  if (bestAdj) best = bestAdj;
  return best;
}

module.exports = { bestTurn, singleActionCandidates, quickScore, searchBudget };

// --- Self-test / demo: play a full bot-vs-bot game and report timing ---
if (require.main === module) {
  let s = E.createInitialState();
  let turns = 0;
  while (s.moveNumber < s.lineLimit) {
    const t0 = Date.now();
    const turn = bestTurn(s);
    const dt = Date.now() - t0;
    if (!turn) { s = E.applyTimeout(s); continue; }
    s = turn.finalState;
    turns++;
    console.log(`turn ${turns}: moveNumber=${s.moveNumber} took ${dt}ms  scores=${JSON.stringify(s.scores)}`);
  }
  console.log('final scores:', s.scores, '  total turns:', turns, ' moveNumber:', s.moveNumber);
}