// chall-sandbag: turtle while moveNumber<85, then blob greedy. Tests whether
// late building (after invincibility/break dynamics settle) beats early area.
const turtle = require('./chall-turtle.js');
const blob = require('./chall-blob.js');
function bestTurn(state) {
  if (state.moveNumber < 85) return turtle.bestTurn(state);
  return blob.bestTurn(state);
}
module.exports = { bestTurn };
