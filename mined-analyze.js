// mined-analyze.js — replay mined-data/game-*.json through engine.js, emit metrics JSON.
const fs = require('fs');
const path = require('path');
const E = require('./engine.js');

const DATA = path.join(__dirname, 'mined-data');
const EPS = 1e-9;
const opp = (c) => (c === 'blue' ? 'red' : 'blue');
function pt(p) { return Array.isArray(p) ? [p[0], p[1]] : [p.x, p.y]; }

function analyzeFile(fp) {
  const d = JSON.parse(fs.readFileSync(fp, 'utf8'));
  const mh = Array.isArray(d.moveHistory) ? d.moveHistory : [];
  const out = {
    gameID: d.gameID, blue: d.blue, red: d.red, winner: d.winner,
    winReason: d.winReason, moveNumber: d.moveNumber,
    serverScores: d.scores, tournamentID: d.tournamentID || null,
    mhLen: mh.length, realMoves: 0, passes: 0,
    replayOk: true, failAt: -1,
    firstClose: { blue: -1, red: -1 },
    breaksDealt: { blue: 0, red: 0 }, breakDmg: { blue: 0, red: 0 },
    selfBreaks: { blue: 0, red: 0 },
    maxSingleGain: { blue: 0, red: 0 },
    curve: [], // {line, aB, aR, sB, sR}
    final: null,
  };
  let s = E.createInitialState();
  let lastLine = 0;
  const seenClose = { blue: false, red: false };
  for (let i = 0; i < mh.length; i++) {
    const h = mh[i];
    const color = h.color || s.turn;
    const prevA = { ...s.areas };
    const prevS = { ...s.scores };
    try {
      if (h.pass) { s = E.applyTimeout(s); out.passes++; }
      else { s = E.applyMove(s, pt(h.from), pt(h.to)); out.realMoves++; }
    } catch (e) { out.replayOk = false; out.failAt = i; break; }
    lastLine = h.lineNumber || lastLine;
    for (const c of ['blue', 'red']) {
      if (!seenClose[c] && s.areas[c] > 1 + EPS) { seenClose[c] = true; out.firstClose[c] = h.lineNumber || lastLine; }
    }
    if (!h.pass) {
      const o = opp(color);
      const dOppA = s.areas[o] - prevA[o];
      const dOwnA = s.areas[color] - prevA[color];
      if (dOppA < -EPS) { out.breaksDealt[color]++; out.breakDmg[color] += -dOppA; }
      if (dOwnA < -EPS) out.selfBreaks[color]++;
      const g = s.areas[color] - prevA[color];
      if (g > out.maxSingleGain[color]) out.maxSingleGain[color] = g;
    }
    out.curve.push({ line: h.lineNumber || lastLine, aB: s.areas.blue, aR: s.areas.red, sB: s.scores.blue, sR: s.scores.red, c: color });
  }
  // Final per-color territory stats from replay state
  out.final = {
    areas: { ...s.areas }, scores: { ...s.scores },
    terrCount: { blue: s.territories.blue.length, red: s.territories.red.length },
    maxTerr: { blue: 0, red: 0 },
  };
  for (const c of ['blue', 'red']) {
    for (const t of s.territories[c]) {
      const ring = Array.isArray(t) ? t : t.ring;
      const a = Math.abs(E.polygonArea(ring));
      if (a > out.final.maxTerr[c]) out.final.maxTerr[c] = a;
    }
  }
  // Area at / gained-after line markers (use curve: last sample with line<=X)
  const atLine = (L, k) => {
    let v = 0;
    for (const p of out.curve) { if (p.line <= L) v = p[k]; else break; }
    return v;
  };
  out.areaAt40 = { blue: atLine(40, 'aB'), red: atLine(40, 'aR') };
  out.areaAt80 = { blue: atLine(80, 'aB'), red: atLine(80, 'aR') };
  out.lateGain = {
    blue: out.final.areas.blue - out.areaAt80.blue,
    red: out.final.areas.red - out.areaAt80.red,
  };
  return out;
}

function main() {
  const files = fs.readdirSync(DATA).filter((f) => f.startsWith('game-') && f.endsWith('.json'));
  const res = files.map((f) => {
    try { return analyzeFile(path.join(DATA, f)); }
    catch (e) { return { file: f, replayOk: false, error: e.message }; }
  });
  fs.writeFileSync(path.join(DATA, 'analysis.json'), JSON.stringify(res, null, 1));
  const ok = res.filter((r) => r.replayOk);
  console.log(`games=${res.length} replayOk=${ok.length} failed=${res.length - ok.length}`);
  for (const r of res.filter((r) => !r.replayOk)) console.log('FAIL', r.gameID || r.file, r.failAt, r.error || '');
  // Quick aggregates
  const q = (arr) => arr.length ? (arr.reduce((a, b) => a + b, 0) / arr.length).toFixed(1) : 'n/a';
  console.log('avg realMoves', q(ok.map((r) => r.realMoves)), 'avg passes', q(ok.map((r) => r.passes)));
  console.log('avg breaksDealt B', q(ok.map((r) => r.breaksDealt.blue)), 'R', q(ok.map((r) => r.breaksDealt.red)));
}
main();
