// health-profile.js — BOT HEALTH MONITOR harness (NEW file, tournament bot untouched).
// Profiles bestTurn self-play, robustness over N games, weird states, adaptiveBudget.
const E = require('../engine/engine.js');
const T = require('../bots/bot-tourney.js');

function playGame(budgetArg, gameIdx) {
  // budgetArg: undefined => adaptiveBudget(state,null) [tournament-realistic default]
  let s = E.createInitialState();
  const perTurn = []; // {moveNumber, turn, dt, moves}
  let illegal = 0, timeouts = 0, throws = 0, turns = 0;
  while (s.moveNumber < s.lineLimit) {
    const t0 = Date.now();
    let turn = null;
    try {
      turn = budgetArg === 'CLOCKSIM' ? T.bestTurn(s, { clockMs: 60000 }) : T.bestTurn(s, budgetArg);
    } catch (e) { throws++; }
    const dt = Date.now() - t0;
    if (!turn) { timeouts++; s = E.applyTimeout(s); continue; }
    // verify legality by replaying each move on a clone
    let r = s, ok = true;
    try {
      for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
    } catch (e) { ok = false; illegal++; }
    if (!ok) { s = E.applyTimeout(s); continue; }
    s = r; turns++;
    perTurn.push({ moveNumber: s.moveNumber, turn: s.turn === 'blue' ? 'red' : 'blue', dt, nMoves: turn.moves.length });
    if (turns > 300) break; // safety
  }
  const dts = perTurn.map(t => t.dt);
  const sum = dts.reduce((a, b) => a + b, 0);
  const worst = dts.length ? perTurn.reduce((a, b) => b.dt > a.dt ? b : a) : null;
  const phase = (mn) => mn <= 6 ? 'early' : mn <= 80 ? 'mid' : 'late';
  const byPhase = { early: [], mid: [], late: [] };
  for (const t of perTurn) byPhase[phase(t.moveNumber)].push(t.dt);
  const avg = (a) => a.length ? (a.reduce((x, y) => x + y, 0) / a.length).toFixed(1) : 'n/a';
  const max = (a) => a.length ? Math.max(...a) : 'n/a';
  return {
    gameIdx, turns, totalMs: sum, avgMs: dts.length ? (sum / dts.length).toFixed(1) : 'n/a',
    worst, illegal, timeouts, throws,
    early: { n: byPhase.early.length, avg: avg(byPhase.early), max: max(byPhase.early) },
    mid: { n: byPhase.mid.length, avg: avg(byPhase.mid), max: max(byPhase.mid) },
    late: { n: byPhase.late.length, avg: avg(byPhase.late), max: max(byPhase.late) },
    scores: s.scores, finalMove: s.moveNumber,
  };
}

const mode = process.argv[2] || 'default';
const games = parseInt(process.argv[3] || '5', 10);
const budgetArg = mode === 'clocksim' ? 'CLOCKSIM' : undefined;
const results = [];
for (let g = 0; g < games; g++) {
  const r = playGame(budgetArg, g);
  results.push(r);
  console.log(`game ${g}: turns=${r.turns} finalMove=${r.finalMove} scores=${JSON.stringify(r.scores)} ` +
    `total=${r.totalMs}ms avg=${r.avgMs}ms worst=${r.worst ? r.worst.dt + 'ms@move' + r.worst.moveNumber : 'n/a'} ` +
    `illegal=${r.illegal} timeouts=${r.timeouts} throws=${r.throws}`);
  console.log(`  early(m<=6): n=${r.early.n} avg=${r.early.avg} max=${r.early.max} | ` +
    `mid(7-80): n=${r.mid.n} avg=${r.mid.avg} max=${r.mid.max} | ` +
    `late(81+): n=${r.late.n} avg=${r.late.avg} max=${r.late.max}`);
}
const allWorst = Math.max(...results.map(r => r.worst ? r.worst.dt : 0));
const allTotal = results.map(r => r.totalMs);
console.log(`AGG games=${games} mode=${mode} worstSingleTurn=${allWorst}ms totals=${allTotal.join(',')} ` +
  `totIllegal=${results.reduce((a, r) => a + r.illegal, 0)} totTimeouts=${results.reduce((a, r) => a + r.timeouts, 0)} totThrows=${results.reduce((a, r) => a + r.throws, 0)}`);
