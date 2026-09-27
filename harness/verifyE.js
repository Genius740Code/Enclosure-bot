// verifyE.js — byte-identity verification for botE-endgame.js.
// Builds deterministic test states (cheap tourney-vs-tourney play), then on
// each state compares botE-endgame.bestTurn vs bot-tourney.bestTurn at FULL
// budget (search completes -> result is a pure function of state):
//   - non-endgame states (lineLimit - moveNumber > 12): moves MUST be identical
//   - endgame states (lineLimit - moveNumber <= 12): report divergence (expected)
// Usage: node verifyE.js [fullBudgetMs=30000]
const E = require('../engine/engine.js');
const T = require('../bots/bot-tourney.js');
const G = require('../bots/botE-endgame.js');
const E_STATE = require('../bots/botE-endgame.js'); // isEndgame lives here too

const BIG = parseInt(process.argv[2] || '30000', 10);
const snapAt = new Set([2, 6, 20, 40, 60, 80, 90, 100, 104, 108, 110, 112, 116]);

function movesKey(t) {
  if (!t || !t.moves || !t.moves.length) return 'null';
  return t.moves.map(m => `${m.from[0]},${m.from[1]}>${m.to[0]},${m.to[1]}`).join(' ');
}

// Build test states with a CHEAP bot (states just need to be legal + realistic).
function buildStates() {
  const states = [{ at: 'move0', s: E.createInitialState() }];
  let s = E.createInitialState();
  let last = -1, guard = 0;
  while (s.moveNumber < s.lineLimit && guard++ < 400) {
    const turn = T.bestTurn(s, 300);
    if (!turn) { s = E.applyTimeout(s); continue; }
    let r = s;
    for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
    s = r;
    for (const t of snapAt) {
      if (last < t && s.moveNumber >= t) states.push({ at: 'move' + s.moveNumber, s });
    }
    last = s.moveNumber;
  }
  // dedupe by moveNumber
  const seen = new Set();
  return states.filter(x => (seen.has(x.at) ? false : (seen.add(x.at), true)));
}

const states = buildStates();
console.log(`test states: ${states.length} (full budget ${BIG}ms per bestTurn)`);

let egIdent = 0, egDiverge = 0, nonIdent = 0, nonDiverge = 0;
for (const { at, s } of states) {
  const isEG = (s.lineLimit - s.moveNumber) <= 12;
  let t1, g1, msT = 0, msG = 0;
  let t0 = Date.now(); try { t1 = T.bestTurn(s, BIG); } catch (e) { t1 = { error: e.message }; } msT = Date.now() - t0;
  t0 = Date.now(); try { g1 = G.bestTurn(s, BIG); } catch (e) { g1 = { error: e.message }; } msG = Date.now() - t0;
  const same = movesKey(t1) === movesKey(g1);
  const tag = `${isEG ? 'ENDGAME ' : 'mid     '} ${at} line=${s.lineLimit - s.moveNumber} left`;
  if (isEG) { same ? egIdent++ : egDiverge++; }
  else { same ? nonDiverge++ : nonIdent++; }
  const verdict = same ? 'IDENTICAL' : 'DIVERGE  ';
  const flag = (!isEG && !same) ? '  <<< BYTE-IDENTITY VIOLATION' : '';
  console.log(`${tag} ${verdict} (T ${msT}ms / G ${msG}ms) T=[${movesKey(t1).slice(0, 60)}] G=[${movesKey(g1).slice(0, 60)}]${flag}`);
}
console.log(`\nRESULT: non-endgame identical=${nonDiverge}/${nonDiverge + nonIdent} (violations=${nonIdent}) ; endgame diverge=${egDiverge}/${egDiverge + egIdent}`);
console.log(nonDiverge + nonIdent > 0 && nonIdent === 0 ? 'BYTE-IDENTITY: PASS' : 'BYTE-IDENTITY: FAIL');
