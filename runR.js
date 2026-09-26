// runR.js — Robustness runner: bot-tourney.js vs challengers through engine.js.
// Usage: node runR.js [--budget 1000] [--seeds 10] [challName...]
// Deterministic challengers: 2 games (bot blue / bot red).
// chall-random: --seeds games per color (default 10), seed = game index.
// Legality verified by replaying returned moves via E.applyMove on live state.
const E = require('./engine.js');
const bot = require('./bot-tourney.js');
const fs = require('fs');

const args = process.argv.slice(2);
const budgetIx = args.indexOf('--budget');
const BUDGET = budgetIx >= 0 ? parseInt(args[budgetIx + 1], 10) : 1000;
const seedsIx = args.indexOf('--seeds');
const NSEEDS = seedsIx >= 0 ? parseInt(args[seedsIx + 1], 10) : 10;
const ALL = ['chall-random', 'chall-junk', 'chall-blob', 'chall-turtle', 'chall-neck', 'chall-sprawl', 'chall-aggro', 'chall-sandbag'];
const names = args.filter(a => !a.startsWith('--') && ALL.includes(a) && a !== String(BUDGET) && a !== String(NSEEDS));
const CHALLS = names.length ? names : ALL;
// remove numeric budget/seeds values that coincide with names (none do) — also drop bare numbers
const numArgs = new Set([String(BUDGET), String(NSEEDS)]);

function playGame(challMod, botColor, gameId, seed) {
  delete require.cache[require.resolve('./' + challMod + '.js')];
  const chall = require('./' + challMod + '.js');
  if (challMod === 'chall-random' && chall.setSeed) chall.setSeed(seed);
  let s = E.createInitialState();
  const seq = [];
  let botMs = 0, challMs = 0, turns = 0, mismatches = 0;
  while (s.moveNumber < s.lineLimit) {
    const isBot = s.turn === botColor;
    const t0 = Date.now();
    let turn = null;
    try {
      turn = isBot ? bot.bestTurn(s, BUDGET) : chall.bestTurn(s);
    } catch (e) { turn = null; }
    const dt = Date.now() - t0;
    if (isBot) botMs += dt; else challMs += dt;
    if (!turn || !turn.moves || !turn.moves.length) { s = E.applyTimeout(s); seq.push({ timeout: true }); continue; }
    try {
      let r = s;
      for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
      s = r;
    } catch (e) {
      mismatches++;
      console.log(`[${gameId}] REPLAY MISMATCH (illegal returned move): ${e.message}`);
      s = E.applyTimeout(s); continue;
    }
    seq.push({ who: isBot ? 'bot' : 'chall', moves: turn.moves });
    turns++;
    if (turns > 200) break;
  }
  return { scores: { ...s.scores }, seq, botMs, challMs, turns, mismatches };
}

const summary = [];
for (const c of CHALLS) {
  const games = c === 'chall-random'
    ? ['blue', 'red'].flatMap(bc => Array.from({ length: NSEEDS }, (_, i) => ({ botColor: bc, seed: bc === 'blue' ? 1000 + i : 2000 + i })))
    : [{ botColor: 'blue' }, { botColor: 'red' }];
  for (const g of games) {
    const seedSuffix = g.seed !== undefined ? `-s${g.seed}` : '';
    const gameId = `R-${c}-botIs${g.botColor}${seedSuffix}`;
    const t0 = Date.now();
    const r = playGame(c, g.botColor, gameId, g.seed);
    const wall = ((Date.now() - t0) / 1000).toFixed(1);
    const challColor = g.botColor === 'blue' ? 'red' : 'blue';
    const botScore = r.scores[g.botColor], challScore = r.scores[challColor];
    const winner = botScore === challScore ? 'DRAW' : (challScore > botScore ? 'CHALLENGER' : 'BOT');
    const margin = botScore > 0 ? ((botScore - challScore) / botScore) : 0;
    const flag = (winner !== 'BOT' || margin < 0.10) ? '  <== P0-CANDIDATE' : '';
    console.log(`${gameId}: bot(${g.botColor})=${botScore.toFixed(1)} chall(${challColor})=${challScore.toFixed(1)} WINNER=${winner}${flag} wall=${wall}s mism=${r.mismatches}`);
    fs.writeFileSync(`${gameId}.moves.json`, JSON.stringify({ scores: r.scores, seq: r.seq }, null, 1));
    summary.push({ game: gameId, botScore, challScore, winner, margin });
  }
}
console.log('\n=== SUMMARY ===');
for (const r of summary) console.log(`${r.game}: bot=${r.botScore.toFixed(1)} chall=${r.challScore.toFixed(1)} ${r.winner} margin=${(r.margin * 100).toFixed(1)}%`);
