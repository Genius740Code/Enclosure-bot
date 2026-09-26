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

const TOP_K_FIRST = 8;   // D1: 10 -> 8 (fewer full second-ply expansions)
const TOP_K_SECOND = 16;  // D1: 25 -> 16

// As the board fills up, both candidate generation AND the engine's own
// Dr() recompute get more expensive per call, so we search a narrower slice
// of the true move space (fewer frontier nodes, coarser offset sampling) to
// keep total time roughly bounded across the whole game.
function searchBudget(state) {
  const n = state.nodes.blue.length + state.nodes.red.length;
  if (n <= 12) return { nodeCap: Infinity, offsets: OFFSETS_FULL };
  if (n <= 30) return { nodeCap: 12, offsets: OFFSETS_FULL }; // D1: 16 -> 12
  if (n <= 60) return { nodeCap: 8, offsets: OFFSETS_COARSE }; // D1: 10 -> 8
  return { nodeCap: 5, offsets: OFFSETS_COARSE }; // D1: 7 -> 5
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
  for (const c1 of shortlist) {
    const secondCands = singleActionCandidates(c1.result, budget);
    secondCands.sort((a, b) => quickScore(c1.result, b) - quickScore(c1.result, a));
    const shortlist2 = secondCands.slice(0, TOP_K_SECOND);
    for (const c2 of shortlist2) {
      const opp = color === 'blue' ? 'red' : 'blue';
      const ownGain = c2.result.scores[color] - startScore;
      const oppGain = c2.result.scores[opp] - state.scores[opp];
      const val = ownGain - oppGain * 0.15; // slight preference for denying opponent too
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