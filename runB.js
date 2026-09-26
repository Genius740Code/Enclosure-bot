// runB.js — Track B runner: bot.js vs challenger through engine.js.
// Usage: node runB.js [challName...]   (default: all)
// Each challenger plays 2 games: once as blue (bot=red), once as red (bot=blue).
const E = require('./engine.js');
const bot = require('./bot.js');
const fs = require('fs');

const ALL = ['chall-blob', 'chall-turtle', 'chall-neck', 'chall-sprawl', 'chall-aggro', 'chall-sandbag'];
const names = process.argv.slice(2).filter(a => ALL.includes(a));
const CHALLS = names.length ? names : ALL;

function playGame(challMod, botColor, gameId) {
  const chall = require('./' + challMod + '.js');
  let s = E.createInitialState();
  const seq = [];
  let botMs = 0, challMs = 0, turns = 0;
  while (s.moveNumber < s.lineLimit) {
    const isBot = s.turn === botColor;
    const t0 = Date.now();
    let turn = null;
    try {
      turn = (isBot ? bot : chall).bestTurn(s);
    } catch (e) { turn = null; }
    const dt = Date.now() - t0;
    if (isBot) botMs += dt; else challMs += dt;
    if (!turn || !turn.moves || !turn.moves.length) { s = E.applyTimeout(s); seq.push({ turn: s.turn, timeout: true }); continue; }
    // verify by replaying moves on live state (challengers return finalState already)
    try {
      let r = s;
      for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
      s = r;
    } catch (e) {
      // fall back to challenger finalState if replay mismatches (shouldn't happen)
      console.log(`[${gameId}] REPLAY MISMATCH: ${e.message}`);
      s = E.applyTimeout(s); continue;
    }
    seq.push({ who: s.turn === undefined ? '?' : (isBot ? 'bot' : 'chall'), color: isBot ? botColor : (botColor === 'blue' ? 'red' : 'blue'), moves: turn.moves, dt });
    turns++;
    if (turns > 200) break; // safety
  }
  return { scores: { ...s.scores }, areas: { ...s.areas }, seq, botMs, challMs, turns };
}

const summary = [];
for (const c of CHALLS) {
  for (const botColor of ['blue', 'red']) {
    const gameId = `${c}-botIs${botColor}`;
    console.log(`--- ${gameId} starting ---`);
    const t0 = Date.now();
    const g = playGame(c, botColor, gameId);
    const wall = ((Date.now() - t0) / 1000).toFixed(1);
    const challColor = botColor === 'blue' ? 'red' : 'blue';
    const botScore = g.scores[botColor], challScore = g.scores[challColor];
    const winner = botScore === challScore ? 'DRAW' : (challScore > botScore ? 'CHALLENGER' : 'BOT');
    console.log(`${gameId}: bot(${botColor})=${botScore} chall(${challColor})=${challScore} WINNER=${winner} wall=${wall}s botCpu=${(g.botMs/1000).toFixed(1)}s challCpu=${(g.challMs/1000).toFixed(1)}s`);
    fs.writeFileSync(`${gameId}.moves.json`, JSON.stringify({ scores: g.scores, areas: g.areas, seq: g.seq }, null, 1));
    summary.push({ game: gameId, botColor, challColor, botScore, challScore, winner, wall });
  }
}
console.log('\n=== SUMMARY ===');
for (const r of summary) console.log(`${r.game}: bot=${r.botScore} chall=${r.challScore} ${r.winner} (${r.wall}s)`);
