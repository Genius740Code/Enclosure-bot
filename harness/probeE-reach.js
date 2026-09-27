// probeE-reach.js — ENDGAME track: in endgame positions (move >= 108) from
// tourney-vs-tourney play, can the LOSING side actually reach enemy edges that
// destroy banked area? Tests the "denial moves exist in the endgame" premise.
const E = require('../engine/engine.js');
const BOT = require('../bots/bot.js');
const T = require('../bots/bot-tourney.js');

const EG = 12;
let s = E.createInitialState();
const states = [];
let guard = 0;
while (s.moveNumber < s.lineLimit && guard++ < 400) {
  const turn = T.bestTurn(s, 300);
  if (!turn) { s = E.applyTimeout(s); continue; }
  let r = s;
  for (const m of turn.moves) r = E.applyMove(r, m.from, m.to);
  s = r;
  if (s.moveNumber >= s.lineLimit - EG) states.push({ at: s.moveNumber, s: structuredClone(s) });
}
console.log(`endgame states captured: ${states.length}`);
let totCands = 0, totAreaBreaks = 0, totAnyBreaks = 0, totBigDenial = 0;
for (const { at, s: st } of states) {
  // losing side = lower banked score (or lower area as tiebreak)
  const loser = st.scores.blue < st.scores.red ? 'blue' : (st.scores.red < st.scores.blue ? 'red' : (st.areas.blue < st.areas.red ? 'blue' : 'red'));
  const winner = loser === 'blue' ? 'red' : 'blue';
  // What can the loser do if it were their turn? (probe on a turn-swapped clone)
  const probe = structuredClone(st);
  probe.turn = loser;
  probe.actionsRemaining = Math.min(2, probe.lineLimit - probe.moveNumber);
  const cands = BOT.singleActionCandidates(probe, { nodeCap: Infinity, offsets: undefined });
  let areaBreaks = 0, anyBreaks = 0, bigDenial = 0, bestDeny = 0;
  for (const c of cands) {
    const removed = probe.segments[winner].length - c.result.segments[winner].length;
    const dOppA = c.result.areas[winner] - probe.areas[winner];
    if (removed > 0) anyBreaks++;
    if (dOppA < -1e-9) { areaBreaks++; bigDenial += -dOppA; if (-dOppA > bestDeny) bestDeny = -dOppA; }
  }
  totCands += cands.length; totAreaBreaks += areaBreaks; totAnyBreaks += anyBreaks; totBigDenial += bigDenial;
  console.log(`move ${at} (line=${st.lineLimit - st.moveNumber} left): loser=${loser} ` +
    `cands=${cands.length} breakCands(any)=${anyBreaks} breakCands(areaKilling)=${areaBreaks} ` +
    `totalDenialArea=${bigDenial.toFixed(1)} bestSingleDenial=${bestDeny.toFixed(1)} ` +
    `| scores B${st.scores.blue.toFixed(0)}/R${st.scores.red.toFixed(0)} areas B${st.areas.blue.toFixed(1)}/R${st.areas.red.toFixed(1)}`);
}
console.log(`\nAGGREGATE over ${states.length} endgame states:`);
console.log(`avg candidates=${(totCands / states.length).toFixed(0)} avg area-killing break candidates=${(totAreaBreaks / states.length).toFixed(2)} avg any-break candidates=${(totAnyBreaks / states.length).toFixed(2)} avg total denial area=${(totBigDenial / states.length).toFixed(1)}`);
console.log(`states with >=1 area-killing break available to the loser: ${states.length ? 'see above per-state column' : 'n/a'}`);
