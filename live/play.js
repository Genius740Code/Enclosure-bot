// play.js — play against the bot in your terminal. Usage:
//   node live/play.js [blue|red]   (default: you are blue)
// Moves: type "x1,y1 x2,y2" per action, e.g.  3,9 6,9
// Type "pass" to end your turn early. Ctrl+C to quit.
const E = require('../engine/engine.js');
const T = require('../bots/bot-tourney.js');
const readline = require('readline');

const human = (process.argv[2] || 'blue').toLowerCase();
if (human !== 'blue' && human !== 'red') { console.error('use: node live/play.js [blue|red]'); process.exit(1); }
const botColor = human === 'blue' ? 'red' : 'blue';

function draw(s) {
  const mark = {};
  for (const n of s.nodes.blue) mark[n] = (mark[n] || '') + 'B';
  for (const n of s.nodes.red) mark[n] = (mark[n] || '') + 'R';
  const lines = [];
  for (let y = s.boardSize - 1; y >= 0; y--) {
    let row = String(y).padStart(2) + ' ';
    for (let x = 0; x < s.boardSize; x++) row += (mark[[x, y]] || '.') + ' ';
    lines.push(row);
  }
  lines.push('   ' + Array.from({ length: s.boardSize }, (_, x) => (x % 10)).join(' '));
  return lines.join('\n');
}

function status(s) {
  console.log(`\n--- move ${s.moveNumber}/${s.lineLimit} | turn: ${s.turn} (${s.actionsRemaining} action(s)) | scores B:${s.scores.blue} R:${s.scores.red} | areas B:${s.areas.blue} R:${s.areas.red} ---`);
}

const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
const ask = (q) => new Promise((res) => rl.question(q, res));

(async () => {
  let s = E.createInitialState();
  console.log(`You are ${human.toUpperCase()}. Bot is ${botColor.toUpperCase()}. Board: B=your... (see legend)`);
  while (s.moveNumber < s.lineLimit) {
    status(s);
    if (s.turn === human) {
      console.log(draw(s));
      console.log(`Your nodes (${s.nodes[human].length}): ` + s.nodes[human].map((n) => n.join(',')).join(' '));
      let acted = false;
      while (s.turn === human && s.moveNumber < s.lineLimit) {
        const ans = (await ask(`action (${s.actionsRemaining} left) "x1,y1 x2,y2" or pass: `)).trim();
        if (ans === '' ) continue;
        if (ans.toLowerCase() === 'pass') { s = E.applyTimeout(s); acted = true; break; }
        const m = ans.match(/(\d+)\s*,\s*(\d+)\s+(\d+)\s*,\s*(\d+)/);
        if (!m) { console.log('format: x1,y1 x2,y2'); continue; }
        try {
          s = E.applyMove(s, [+m[1], +m[2]], [+m[3], +m[4]]);
          acted = true;
        } catch (e) { console.log('illegal: ' + e.message); }
      }
      if (!acted) s = E.applyTimeout(s);
    } else {
      const t0 = Date.now();
      const turn = T.bestTurn(s, { clockMs: 60000 });
      const dt = Date.now() - t0;
      if (!turn) { console.log(`bot timeouts (${dt}ms)`); s = E.applyTimeout(s); continue; }
      let r = s;
      for (const mv of turn.moves) r = E.applyMove(r, mv.from, mv.to);
      s = r;
      console.log(`bot (${botColor}) played in ${dt}ms: ` + turn.moves.map((m) => m.from.join(',') + '->' + m.to.join(',')).join(' | '));
    }
  }
  status(s);
  console.log('GAME OVER. Winner:', s.scores.blue === s.scores.red ? 'DRAW' : s.scores.blue > s.scores.red ? 'BLUE' : 'RED');
  rl.close();
})().catch((e) => { console.error(e); process.exit(1); });
