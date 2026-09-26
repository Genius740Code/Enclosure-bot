// chall-blob: big single blob. 1-ply greedy own-area gain only, tiebreak longest edge.
// No opponent modeling, no 2nd ply, no break-seeking — genuinely different from bot.js.
const { OFF_FULL, cands, assemble } = require('./chall-util.js');
function pick(s) {
  let best = null, bv = -Infinity;
  for (const c of cands(s, OFF_FULL)) {
    const len = Math.hypot(c.to[0] - c.from[0], c.to[1] - c.from[1]);
    const v = c.areaGain * 10 + len * 0.1;
    if (v > bv) { bv = v; best = c; }
  }
  return best;
}
function bestTurn(state) { return assemble(state, pick); }
module.exports = { bestTurn };
