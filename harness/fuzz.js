// fuzz.js — generate N random full games on engine.js, print move lists.
// Usage: node fuzz.js <games> ; output: GAME i blocks of "x1 y1 x2 y2" lines.
const E = require('../engine/engine.js');
const N = parseInt(process.argv[2] || '5', 10);
let seed = 12345;
const rnd = () => (seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff;
for (let g = 0; g < N; g++) {
  console.log(`# GAME ${g}`);
  let s = E.createInitialState();
  while (s.moveNumber < s.lineLimit) {
    const B = require('../bots/bot.js');
    const cands = B.singleActionCandidates(s, {});
    if (!cands.length) { console.log('pass'); s = E.applyTimeout(s); continue; }
    const c = cands[Math.floor(rnd() * cands.length)];
    console.log(`${c.from[0]} ${c.from[1]} ${c.to[0]} ${c.to[1]}`);
    s = E.applyMove(s, c.from, c.to);
  }
  console.log(`# END blue=${s.scores.blue.toFixed(6)} red=${s.scores.red.toFixed(6)} bends=${s.segments.blue.length} rends=${s.segments.red.length} bnodes=${s.nodes.blue.length} rnodes=${s.nodes.red.length}`);
}
