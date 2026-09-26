// bridge.js — stdio bridge so the Python UI can use the JS bot/engine.
// Protocol: one JSON object per line on stdin, one per line on stdout.
//   {cmd:"new", color:"blue"|"red"}  -> {state, bot}
//   {cmd:"legal", from:[x,y]}        -> {targets:[[x,y]..]}
//   {cmd:"human", from, to} | {cmd:"human", pass:true} -> {state,bot} | {error,state}
const E = require('./engine.js');
const T = require('./bot-tourney.js');
const readline = require('readline');

let S = E.createInitialState();
let human = 'blue';
const BUDGET = 4000;

function pub() {
  return {
    turn: S.turn, moveNumber: S.moveNumber, lineLimit: S.lineLimit,
    actionsRemaining: S.actionsRemaining, scores: S.scores, areas: S.areas,
    nodes: S.nodes,
    segments: { blue: S.segments.blue, red: S.segments.red },
    human, over: S.moveNumber >= S.lineLimit,
    winner: S.moveNumber >= S.lineLimit
      ? (S.scores.blue === S.scores.red ? 'draw' : S.scores.blue > S.scores.red ? 'blue' : 'red') : null,
  };
}
function botRespond() {
  const t0 = Date.now();
  const moves = [];
  while (S.turn !== human && S.moveNumber < S.lineLimit) {
    const turn = T.bestTurn(S, BUDGET);
    if (!turn) { S = E.applyTimeout(S); break; }
    for (const m of turn.moves) S = E.applyMove(S, m.from, m.to);
    moves.push(...turn.moves.map((m) => ({ from: m.from, to: m.to })));
    if (S.turn === human || S.moveNumber >= S.lineLimit) break;
  }
  return { moves, ms: Date.now() - t0 };
}
const rl = readline.createInterface({ input: process.stdin });
rl.on('line', (line) => {
  let out;
  try {
    const b = JSON.parse(line);
    if (b.cmd === 'new') {
      human = b.color === 'red' ? 'red' : 'blue';
      S = E.createInitialState();
      const info = S.turn !== human ? botRespond() : { moves: [], ms: 0 };
      out = { state: pub(), bot: info };
    } else if (b.cmd === 'legal') {
      const f = b.from, targets = [];
      if (S.turn === human && S.nodes[human].some((n) => n[0] === f[0] && n[1] === f[1])) {
        for (let dx = -3; dx <= 3; dx++) for (let dy = -3; dy <= 3; dy++) {
          if (!dx && !dy) continue;
          const to = [f[0] + dx, f[1] + dy];
          if (to[0] < 0 || to[0] > 18 || to[1] < 0 || to[1] > 18) continue;
          try { E.applyMove(E.cloneState(S), f, to); targets.push(to); } catch (e) {}
        }
      }
      out = { targets };
    } else if (b.cmd === 'human') {
      if (S.turn !== human) throw new Error('not your turn');
      S = b.pass ? E.applyTimeout(S) : E.applyMove(S, b.from, b.to);
      const info = (S.turn !== human && S.moveNumber < S.lineLimit) ? botRespond() : { moves: [], ms: 0 };
      out = { state: pub(), bot: info };
    } else throw new Error('bad cmd');
  } catch (e) { out = { error: e.message, state: pub() }; }
  process.stdout.write(JSON.stringify(out) + '\n');
});
