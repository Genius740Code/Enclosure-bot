// chall-random.js — uniform-random legal move chooser (seedable mulberry32).
// Rejection sampling over (own-node, dx/dy-in-box) pairs: each legal move has
// exactly one (from,to), so accepted samples are uniform over legal moves.
// Compatible signature: bestTurn(state). Seed via setSeed(n).
const E = require('./engine.js');

function mulberry32(seed) {
  let a = (seed >>> 0) || 1;
  return function () {
    a |= 0; a = (a + 0x6D2B79F5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

let defaultRng = mulberry32(1);
function setSeed(n) { defaultRng = mulberry32(n); }

function randomLegalAction(state, rng, maxAttempts) {
  const nodes = state.nodes[state.turn];
  const R = 3, size = state.boardSize;
  for (let a = 0; a < maxAttempts; a++) {
    const from = nodes[Math.floor(rng() * nodes.length)];
    const dx = Math.floor(rng() * (2 * R + 1)) - R;
    const dy = Math.floor(rng() * (2 * R + 1)) - R;
    if (dx === 0 && dy === 0) continue;
    const to = [from[0] + dx, from[1] + dy];
    if (to[0] < 0 || to[0] >= size || to[1] < 0 || to[1] >= size) continue;
    try {
      const result = E.applyMove(state, from, to);
      return { from: [...from], to, result };
    } catch (e) { /* illegal, retry */ }
  }
  return null;
}

// Deterministic fallback: first legal move in scan order (only used when
// rejection sampling fails, e.g. extremely constrained positions).
function firstLegalAction(state) {
  const R = 3, size = state.boardSize;
  for (const from of state.nodes[state.turn]) {
    for (let dx = -R; dx <= R; dx++) for (let dy = -R; dy <= R; dy++) {
      if (dx === 0 && dy === 0) continue;
      const to = [from[0] + dx, from[1] + dy];
      if (to[0] < 0 || to[0] >= size || to[1] < 0 || to[1] >= size) continue;
      try {
        const result = E.applyMove(state, from, to);
        return { from: [...from], to, result };
      } catch (e) { /* illegal */ }
    }
  }
  return null;
}

function bestTurn(state) {
  const rng = defaultRng;
  const moves = [];
  let s = state;
  const n = state.actionsRemaining;
  for (let i = 0; i < n; i++) {
    const c = randomLegalAction(s, rng, 500) || firstLegalAction(s);
    if (!c) break;
    moves.push({ from: c.from, to: c.to });
    s = c.result;
    if (s.turn !== state.turn) break;
  }
  if (!moves.length) return null;
  return { moves, finalState: s };
}

module.exports = { bestTurn, setSeed, mulberry32 };
