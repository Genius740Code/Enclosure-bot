// chall-junk.js — deliberately bad play: longest aimless edges, avoids
// closing area. Picks the legal single action with max edge length among
// zero-area-gain moves; only takes area if every legal move gains area.
// Greedy per action across the turn.
const { OFF_FULL, cands } = require('./chall-util.js');

function edgeLen(c) {
  return Math.hypot(c.to[0] - c.from[0], c.to[1] - c.from[1]);
}

function pick(s) {
  let bestNoGain = null, bestNoGainLen = -1;
  let bestGain = null, bestGainVal = Infinity; // least area if forced
  for (const c of cands(s, OFF_FULL)) {
    const L = edgeLen(c);
    if (c.areaGain <= 0) {
      // Prefer longest edge; tiebreak: fewer enemy breaks (avoid contact).
      const v = L - c.breaks * 10;
      if (v > bestNoGainLen) { bestNoGainLen = v; bestNoGain = c; }
    } else if (c.areaGain < bestGainVal) {
      bestGainVal = c.areaGain; bestGain = c;
    }
  }
  return bestNoGain || bestGain;
}

function bestTurn(state) {
  const moves = [];
  let s = state;
  const n = state.actionsRemaining;
  for (let i = 0; i < n; i++) {
    const c = pick(s);
    if (!c) break;
    moves.push({ from: [...c.from], to: [...c.to] });
    s = c.result;
    if (s.turn !== state.turn) break;
  }
  if (!moves.length) return null;
  return { moves, finalState: s };
}

module.exports = { bestTurn };
