// chall-neckhunter: breaking dominates everything; else graze enemy frontier.
// Score = breaks*1000 + areaGain*2 - dist(newEndpoint, nearestEnemyNode)*0.1
const { OFF_FULL, cands, assemble } = require('./chall-util.js');
function pick(s) {
  const opp = s.turn === 'blue' ? 'red' : 'blue';
  let best = null, bv = -Infinity;
  for (const c of cands(s, OFF_FULL)) {
    let dmin = Infinity;
    for (const n of s.nodes[opp]) {
      const d = Math.hypot(c.to[0]-n[0], c.to[1]-n[1]);
      if (d < dmin) dmin = d;
    }
    const v = c.breaks * 1000 + c.areaGain * 2 - dmin * 0.1;
    if (v > bv) { bv = v; best = c; }
  }
  return best;
}
function bestTurn(state) { return assemble(state, pick); }
module.exports = { bestTurn };
