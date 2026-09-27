// chall-turtle: tiny defended loop then passive. First closes a small 3x3 box
// around its start anchor, then only shortest interior moves (chebyshev<=1)
// from highest-degree nodes. Never seeks breaks or area.
const E = require('../engine/engine.js');
const { cands, assemble } = require('./chall-util.js');
const SHORT = [[1,0],[-1,0],[0,1],[0,-1],[1,1],[1,-1],[-1,1],[-1,-1]];
function pick(s) {
  const color = s.turn;
  const deg = n => s.segments[color].reduce((a, g) =>
    a + ((E.pointsEqual(g.from, n) || E.pointsEqual(g.to, n)) ? 1 : 0), 0);
  let best = null, bv = -Infinity;
  for (const c of cands(s, SHORT)) {
    const v = deg(c.from) * 2 - Math.hypot(c.to[0]-c.from[0], c.to[1]-c.from[1]) * 0.01 + c.areaGain;
    if (v > bv) { bv = v; best = c; }
  }
  if (best) return best;
  // fallback: any short-ish legal move
  const MED = [[2,0],[-2,0],[0,2],[0,-2],[2,2],[-2,-2],[2,-2],[-2,2]];
  for (const c of cands(s, MED)) return c;
  return null;
}
function bestTurn(state) { return assemble(state, pick); }
module.exports = { bestTurn };
