// probeE-mined.js — ENDGAME track: replay mined-data/game-*.json, compute
// endgame-window (last-12-moves, line >= 108) behavior of winners vs losers.
// Supports/refutes: "human winners close with bank-and-deny endgames".
const fs = require('fs');
const path = require('path');
const E = require('../engine/engine.js');

const DATA = path.join(__dirname, '..', 'mined-data');
const opp = (c) => (c === 'blue' ? 'red' : 'blue');
const EG_MOVES = 12;
function pt(p) { return Array.isArray(p) ? [p[0], p[1]] : [p.x, p.y]; }

function analyze(fp) {
  const d = JSON.parse(fs.readFileSync(fp, 'utf8'));
  const mh = Array.isArray(d.moveHistory) ? d.moveHistory : [];
  const winnerColor = d.winner === d.blue ? 'blue' : (d.winner === d.red ? 'red' : null);
  if (!winnerColor) return { gameID: d.gameID, winner: d.winner, replayOk: false, failAt: -1, winReason: d.winReason };
  let s = E.createInitialState();
  const out = {
    gameID: d.gameID, winner: winnerColor, winReason: d.winReason, moveNumber: d.moveNumber, mhLen: mh.length,
    replayOk: true, failAt: -1,
    egStart: null, egBreaks: { blue: 0, red: 0 }, egBreakDmg: { blue: 0, red: 0 },
    egSelfBreaks: { blue: 0, red: 0 }, egGain: { blue: 0, red: 0 },
    egTurns: { blue: 0, red: 0 },
  };
  const eps = 1e-9;
  for (let i = 0; i < mh.length; i++) {
    const h = mh[i];
    const color = h.color || s.turn;
    const prevA = { ...s.areas };
    const prevS = { ...s.scores };
    const line = h.lineNumber != null ? h.lineNumber : s.moveNumber;
    try {
      if (h.pass) s = E.applyTimeout(s);
      else s = E.applyMove(s, pt(h.from), pt(h.to));
    } catch (e) { out.replayOk = false; out.failAt = i; break; }
    const inEG = (120 - line) <= EG_MOVES;
    if (inEG && !out.egStart) {
      out.egStart = { line, blue: prevA.blue, red: prevA.red, sB: prevS.blue, sR: prevS.red };
    }
    if (inEG && !h.pass) {
      out.egTurns[color]++;
      const o = opp(color);
      const dOppA = s.areas[o] - prevA[o];
      const gOwn = s.areas[color] - prevA[color];
      if (dOppA < -eps) { out.egBreaks[color]++; out.egBreakDmg[color] += -dOppA; }
      if (gOwn < -eps) out.egSelfBreaks[color]++;
      if (gOwn > 0) out.egGain[color] += gOwn;
    }
  }
  if (out.egStart) {
    out.egFinal = { areas: { ...s.areas }, scores: { ...s.scores } };
    out.egDelta = { blue: s.scores.blue - out.egStart.sB, red: s.scores.red - out.egStart.sR };
    out.egSwing = out.egDelta[out.winner] - out.egDelta[opp(out.winner)];
    out.leaderAtEg = out.egStart.blue > out.egStart.red ? 'blue' : (out.egStart.red > out.egStart.blue ? 'red' : 'even');
  }
  return out;
}

function main() {
  const files = fs.readdirSync(DATA).filter((f) => f.startsWith('game-') && f.endsWith('.json'));
  const res = [];
  for (const f of files) { try { const r = analyze(path.join(DATA, f)); r.file = f; res.push(r); } catch (e) { /* skip */ } }
  const ok = res.filter(r => r.replayOk && r.egStart);
  console.log(`games=${res.length} okWithEndgame=${ok.length} (skipped/failed=${res.length - ok.length})`);
  let flips = 0;
  let wB = 0, wBD = 0, lB = 0, lBD = 0, wG = 0, lG = 0;
  for (const r of ok) {
    const w = r.winner, l = opp(w);
    if (r.leaderAtEg !== 'even' && r.leaderAtEg !== w) flips++;
    wB += r.egBreaks[w]; wBD += r.egBreakDmg[w]; wG += r.egGain[w];
    lB += r.egBreaks[l]; lBD += r.egBreakDmg[l]; lG += r.egGain[l];
  }
  console.log(`\nENDGAME WINDOW (line >= 108, last <=12 moves):`);
  console.log(`leader-flipped-by-endgame: ${flips}/${ok.length}`);
  if (ok.length) {
    console.log(`WINNERS:  egBreaks/game=${(wB / ok.length).toFixed(2)} egBreakDmg/game=${(wBD / ok.length).toFixed(1)} egGain/game=${(wG / ok.length).toFixed(1)}`);
    console.log(`LOSERS:   egBreaks/game=${(lB / ok.length).toFixed(2)} egBreakDmg/game=${(lBD / ok.length).toFixed(1)} egGain/game=${(lG / ok.length).toFixed(1)}`);
    const avgSwing = ok.reduce((a, r) => a + r.egSwing, 0) / ok.length;
    console.log(`egSwing (winner endgame banked delta - loser's) avg=${avgSwing.toFixed(1)}`);
    const dist = {};
    for (const r of ok) dist[r.egBreaks[r.winner]] = (dist[r.egBreaks[r.winner]] || 0) + 1;
    console.log(`winner egBreaks distribution: ${JSON.stringify(dist)}`);
  }
  require('fs').writeFileSync(path.join(DATA, 'endgame-probe.json'), JSON.stringify(res.filter(r => r.egStart), null, 1));
}
main();
