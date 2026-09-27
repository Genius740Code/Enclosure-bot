// matchE.js — ENDGAME track match harness. NEW file (run-match.js untouched).
// Usage:
//   node harness/matchE.js <botA-path> <botB-path> [games=6] [budgetMs=2000]
// A is blue in even games, red in odd (alternating). Per game reports:
//   W-L + margin + endgame-window (last-12-moves) score delta per color
//   + shadow divergence: on the variant's endgame turns, what the opponent bot
//     would have played on the same state (computed on a non-mutated state).
// Run sequentially in one process (no parallel-search noise), niced by caller.
const E = require('../engine/engine.js');

function load(p) { const path = require('path'); const q = p.startsWith('.') || path.isAbsolute(p) ? p : path.join(process.cwd(), p); return require(q); }
function movesKey(t) {
  if (!t || !t.moves || !t.moves.length) return 'null';
  return t.moves.map(m => `${m.from[0]},${m.from[1]}>${m.to[0]},${m.to[1]}`).join(' ');
}

function main() {
  const aPath = process.argv[2], bPath = process.argv[3];
  const games = parseInt(process.argv[4] || '6', 10);
  const budget = parseInt(process.argv[5] || '2000', 10);
  const logFile = process.argv[6] || 'matchE-log.json';
  if (!aPath || !bPath) { console.error('use: node harness/matchE.js <botA> <botB> [games] [budgetMs]'); process.exit(1); }
  const A = load(aPath), B = load(bPath);
  let w = 0, l = 0, d = 0, marginSum = 0;
  const rows = [];
  for (let g = 0; g < games; g++) {
    const aIsBlue = g % 2 === 0;
    const botFor = (color) => ((color === 'blue') === aIsBlue ? A : B);
    const myColor = aIsBlue ? 'blue' : 'red';
    let s = E.createInitialState();
    let egStart = null; // scores/margin when the endgame window opened
    let guard = 0;
    let divCount = 0, firstDivAt = -1;
    let egSamples = 0;
    while (s.moveNumber < s.lineLimit && guard++ < 400) {
      if (!egStart && (s.lineLimit - s.moveNumber) <= 12) {
        egStart = {
          move: s.moveNumber, blue: s.scores.blue, red: s.scores.red,
          margin: (aIsBlue ? s.scores.blue - s.scores.red : s.scores.red - s.scores.blue),
        };
      }
      // Shadow divergence probe: variant's endgame turns only, cap samples/game.
      const inEG = (s.lineLimit - s.moveNumber) <= 12;
      if (inEG && s.turn === myColor && egSamples < 16) {
        egSamples++;
        let shadow = null;
        try { shadow = B.bestTurn(s, budget); } catch (e) { shadow = null; }
        const mine = (() => { try { return botFor(myColor).bestTurn(s, budget); } catch (e) { return null; } })();
        if (movesKey(shadow) !== movesKey(mine)) {
          divCount++;
          if (firstDivAt < 0) {
            firstDivAt = s.moveNumber;
            console.log(`  game ${g + 1} FIRST DIVERGENCE at move ${s.moveNumber} (${s.lineLimit - s.moveNumber} left): ` +
              `botE=[${movesKey(mine).slice(0, 70)}] tourney=[${movesKey(shadow).slice(0, 70)}]`);
          }
        }
        // apply the ACTUAL plan computed above (state untouched by bestTurn)
        if (mine && mine.moves && mine.moves.length) {
          let r = s;
          try { for (const m of mine.moves) r = E.applyMove(r, m.from, m.to); s = r; continue; }
          catch (e) { s = E.applyTimeout(s); continue; }
        } else { s = E.applyTimeout(s); continue; }
      }
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
    const egDelta = egStart ? {
      blue: +(s.scores.blue - egStart.blue).toFixed(1),
      red: +(s.scores.red - egStart.red).toFixed(1),
      marginSwing: +((aIsBlue ? s.scores.blue - s.scores.red : s.scores.red - s.scores.blue) - egStart.margin).toFixed(1),
    } : null;
    const line = `game ${g + 1}: A=${myColor} B:${s.scores.blue.toFixed(1)} R:${s.scores.red.toFixed(1)} margin(A)=${m.toFixed(1)}` +
      (egStart ? ` | endgame from move ${egStart.move}: start(${egStart.blue.toFixed(1)}/${egStart.red.toFixed(1)} m=${egStart.margin.toFixed(1)})` +
        ` delta B${egDelta.blue >= 0 ? '+' : ''}${egDelta.blue}/R${egDelta.red >= 0 ? '+' : ''}${egDelta.red} swing=${egDelta.marginSwing >= 0 ? '+' : ''}${egDelta.marginSwing}` +
        ` | shadow divergences=${divCount}/${egSamples}${firstDivAt >= 0 ? ` (first at ${firstDivAt})` : ''}` : ' | (no endgame window reached)');
    console.log(line);
    rows.push({ game: g + 1, aColor: myColor, aScore, bScore, margin: m, egStart, egDelta, divCount, egSamples, firstDivAt });
  }
  console.log(`RESULT A(${aPath}) vs B(${bPath}): ${w}W-${l}L-${d}D / ${games}, avgMargin(A)=${(marginSum / games).toFixed(1)}  budget=${budget}ms`);
  // Aggregate endgame-window deltas from A's perspective
  const withEg = rows.filter(r => r.egStart);
  if (withEg.length) {
    const asBlue = withEg.filter(r => r.aColor === 'blue');
    const asRed = withEg.filter(r => r.aColor === 'red');
    const avg = (arr, k) => arr.length ? (arr.reduce((a, b) => a + b[k], 0) / arr.length).toFixed(1) : 'n/a';
    console.log(`endgame-window avg (A): marginAtStart=${avg(withEg, 'margin')} (blue games ${avg(asBlue, 'margin')}, red games ${avg(asRed, 'margin')}) ` +
      `swing=${avg(withEg, 'marginSwing')} | shadow divergences=${withEg.reduce((a, b) => a + b.divCount, 0)}/${withEg.reduce((a, b) => a + b.egSamples, 0)} endgame turns`);
  }
  require('fs').writeFileSync(logFile, JSON.stringify(rows, null, 1));
}
main();
