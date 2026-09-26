// server.js — local web UI to play the bot. Run: node server.js [port]
// Open http://localhost:PORT in your browser.
const http = require('http');
const fs = require('fs');
const E = require('./engine.js');
const T = require('./bot-tourney.js');

const PORT = parseInt(process.argv[2] || '8901', 10);
let S = E.createInitialState();
let human = 'blue';
const BOT_BUDGET = 4000;

function pub() {
  return {
    turn: S.turn, moveNumber: S.moveNumber, lineLimit: S.lineLimit,
    actionsRemaining: S.actionsRemaining, scores: S.scores, areas: S.areas,
    nodes: S.nodes, segments: S.segments, human,
    over: S.moveNumber >= S.lineLimit,
    winner: S.moveNumber >= S.lineLimit
      ? (S.scores.blue === S.scores.red ? 'draw' : S.scores.blue > S.scores.red ? 'blue' : 'red') : null,
  };
}

function botRespond() {
  const t0 = Date.now();
  let moves = [];
  while (S.turn !== human && S.moveNumber < S.lineLimit) {
    const turn = T.bestTurn(S, BOT_BUDGET);
    if (!turn) { S = E.applyTimeout(S); break; }
    for (const m of turn.moves) S = E.applyMove(S, m.from, m.to);
    moves.push(...turn.moves.map((m) => ({ from: m.from, to: m.to })));
    if (S.turn === human || S.moveNumber >= S.lineLimit) break;
  }
  return { moves, ms: Date.now() - t0 };
}

const server = http.createServer((req, res) => {
  const url = new URL(req.url, 'http://x');
  if (req.method === 'GET' && url.pathname === '/') {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(fs.readFileSync(__dirname + '/play.html'));
    return;
  }
  if (req.method === 'GET' && url.pathname === '/state') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(pub()));
    return;
  }
  if (req.method === 'GET' && url.pathname === '/legal') {
    const f = (url.searchParams.get('from') || '').split(',').map(Number);
    const out = [];
    if (f.length === 2 && S.turn === human && S.nodes[human].some((n) => n[0] === f[0] && n[1] === f[1])) {
      for (let dx = -3; dx <= 3; dx++) for (let dy = -3; dy <= 3; dy++) {
        if (!dx && !dy) continue;
        const to = [f[0] + dx, f[1] + dy];
        if (to[0] < 0 || to[0] > 18 || to[1] < 0 || to[1] > 18) continue;
        try { E.applyMove(E.cloneState(S), f, to); out.push(to); } catch (e) { /* illegal */ }
      }
    }
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ targets: out }));
    return;
  }
  if (req.method === 'POST' && url.pathname === '/demo-step') {
    let body = '';
    req.on('data', (c) => (body += c));
    req.on('end', () => {
      try {
        const b = body ? JSON.parse(body) : {};
        if (b.restart || S.moveNumber >= S.lineLimit) S = E.createInitialState();
        const t0 = Date.now();
        const turn = T.bestTurn(S, 4000);
        let moves = [];
        if (!turn) S = E.applyTimeout(S);
        else { for (const m of turn.moves) S = E.applyMove(S, m.from, m.to); moves = turn.moves; }
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ...pub(), bot: { moves, ms: Date.now() - t0 } }));
      } catch (e) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: e.message, ...pub() }));
      }
    });
    return;
  }
  if (req.method === 'POST' && (url.pathname === '/new' || url.pathname === '/human')) {
    let body = '';
    req.on('data', (c) => (body += c));
    req.on('end', () => {
      try {
        const b = body ? JSON.parse(body) : {};
        if (url.pathname === '/new') {
          human = b.color === 'red' ? 'red' : 'blue';
          S = E.createInitialState();
          let info = { moves: [], ms: 0 };
          if (S.turn !== human) info = botRespond();
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ ...pub(), bot: info }));
          return;
        }
        // /human
        if (S.turn !== human || S.moveNumber >= S.lineLimit) throw new Error('not your turn');
        if (b.pass) S = E.applyTimeout(S);
        else S = E.applyMove(S, b.from, b.to);
        let info = { moves: [], ms: 0 };
        if (S.turn !== human && S.moveNumber < S.lineLimit) info = botRespond();
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ...pub(), bot: info }));
      } catch (e) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: e.message, ...pub() }));
      }
    });
    return;
  }
  res.writeHead(404); res.end('nope');
});
server.listen(PORT, () => console.log(`play at http://localhost:${PORT}`));
