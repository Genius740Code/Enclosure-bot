// Shared tiny helper for Track B challengers: enumerate legal single actions.
const E = require('../engine/engine.js');
const OFF_FULL = [];
for (let dx = -3; dx <= 3; dx++) for (let dy = -3; dy <= 3; dy++) {
  if (dx || dy) OFF_FULL.push([dx, dy]);
}
function centroid(pts) {
  if (!pts.length) return [9, 9];
  return [pts.reduce((a, p) => a + p[0], 0) / pts.length, pts.reduce((a, p) => a + p[1], 0) / pts.length];
}
function* cands(state, offsets) {
  const color = state.turn, opp = color === 'blue' ? 'red' : 'blue';
  for (const from of state.nodes[color]) {
    for (const o of offsets) {
      const to = [from[0] + o[0], from[1] + o[1]];
      if (to[0] < 0 || to[0] >= state.boardSize || to[1] < 0 || to[1] >= state.boardSize) continue;
      try {
        const r = E.applyMove(state, from, to);
        yield { from: [...from], to, result: r,
          areaGain: r.areas[color] - state.areas[color],
          breaks: state.segments[opp].length - r.segments[opp].length };
      } catch (e) { /* illegal */ }
    }
  }
}
// Greedy full-turn assembler: apply single-action rule N=actionsRemaining times.
function assemble(state, pick) {
  const moves = [];
  let s = state;
  const n = state.actionsRemaining;
  for (let i = 0; i < n; i++) {
    const c = pick(s);
    if (!c) break;
    moves.push({ from: c.from, to: c.to });
    s = c.result;
    if (s.turn !== state.turn) break; // turn ended
  }
  if (!moves.length) return null;
  return { moves, finalState: s };
}
module.exports = { E, OFF_FULL, centroid, cands, assemble };
