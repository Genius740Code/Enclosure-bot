// run-match.js — self-play harness for IDEAS track. Usage:
//   node run-match.js <botA-path> <botB-path> [games=6] [budgetMs=400]
// A is blue in even games, red in odd games (alternating). Reports W-L-D from A's perspective + avg margin.
const E = require('./engine.js');
function load(p) { const m = require(p.startsWith('.') ? p : './' + p); return m; }
async function main() {
  const aPath = process.argv[2], bPath = process.argv[3];
  const games = parseInt(process.argv[4] || '6', 10);
  const budget = parseInt(process.argv[5] || '400', 10);
  if (!aPath || !bPath) { console.error('use: node run-match.js <botA> <botB> [games] [budgetMs]'); process.exit(1); }
  const A = load(aPath), B = load(bPath);
  let w = 0, l = 0, d = 0, marginSum = 0;
  for (let g = 0; g < games; g++) {
    const aIsBlue = g % 2 === 0;
    const botFor = (color) => ((color === 'blue') === aIsBlue ? A : B);
    let s = E.createInitialState();
    let guard = 0;
    while (s.moveNumber < s.lineLimit && guard++ < 400) {
      const bot = botFor(s.turn);
      let turn = null;
      try { turn = bot.bestTurn(s, budget); } catch (e) { turn = null; }
      if (!turn || !turn.moves || !turn.moves.length) { s = E.applyTimeout(s); continue; }
      try {
        let r = s;
        for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
        s = r;
      } catch (e) { s = E.applyTimeout(s); }
    }
    const aScore = aIsBlue ? s.scores.blue : s.scores.red;
    const bScore = aIsBlue ? s.scores.red : s.scores.blue;
    const m = aScore - bScore;
    marginSum += m;
    if (m > 0) w++; else if (m < 0) l++; else d++;
    console.log(`game ${g + 1}: A=${aIsBlue ? 'blue' : 'red'} B:${s.scores.blue} R:${s.scores.red} margin(A)=${m.toFixed(1)}`);
  }
  console.log(`RESULT A(${aPath}) vs B(${bPath}): ${w}W-${l}L-${d}D / ${games}, avgMargin(A)=${(marginSum / games).toFixed(1)}  budget=${budget}ms`);
}
main();
