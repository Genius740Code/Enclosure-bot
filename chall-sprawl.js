// chall-sprawler: many small loops. Expand from node farthest from own centroid
// (start new petal), short offsets only (chebyshev<=2), 1-ply area gain.
const { cands, assemble, centroid } = require('./chall-util.js');
const SHORT2 = [];
for (let dx = -2; dx <= 2; dx++) for (let dy = -2; dy <= 2; dy++) {
  if (dx || dy) SHORT2.push([dx, dy]);
}
function pick(s) {
  const color = s.turn;
  const cen = centroid(s.nodes[color]);
  const far = [...s.nodes[color]].sort((a, b) =>
    Math.hypot(b[0]-cen[0],b[1]-cen[1]) - Math.hypot(a[0]-cen[0],a[1]-cen[1]))[0];
  let best = null, bv = -Infinity;
  for (const c of cands(s, SHORT2)) {
    const fromFar = -Math.hypot(c.from[0]-far[0], c.from[1]-far[1]); // prefer far node
    const v = c.areaGain * 10 + fromFar * 0.5;
    if (v > bv) { bv = v; best = c; }
  }
  return best;
}
function bestTurn(state) { return assemble(state, pick); }
module.exports = { bestTurn };
