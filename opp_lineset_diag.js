// opp_lineset_diag.js — Lane O Q7a follow-up: decompose unpredicted pair closes.
// Same seeds/positions as opp_lineset.js. For each closing pair (Y>0):
// Y1 (m1 alone), Y2|1 (m2 given m1), single-ring P for s2-given-s1 context,
// and inScope rejection reasons. Deterministic, engine legality only.
const E = require('./engine/engine.js');
const { genGame, enumSingles, predictRing, inScope, lcg } = require('./opp_lineset.js');

function dshuffle(arr, seed) {
  const rnd = lcg(seed);
  const a = arr.slice();
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(rnd() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}
function scopeReason(ownSegs, newSegs) {
  const epis = s => [s.from, s.to];
  for (const ns of newSegs) {
    for (const os of ownSegs) {
      const r = E.segmentIntersect({ from: ns.from, to: ns.to }, os);
      if (r.kind === 'collinear') return 'own-collinear';
      if (r.kind === 'point' && !epis(ns).some(p => E.pointsEqual(p, r.point)) &&
          !epis(os).some(p => E.pointsEqual(p, r.point))) return 'own-interior-cross';
    }
  }
  for (let i = 0; i < newSegs.length; i++)
    for (let j = i + 1; j < newSegs.length; j++) {
      const a = newSegs[i], b = newSegs[j];
      const r = E.segmentIntersect({ from: a.from, to: a.to }, { from: b.from, to: b.to });
      if (r.kind === 'collinear') return 'sib-collinear';
      if (r.kind === 'point') {
        const onA = [a.from, a.to].some(p => E.pointsEqual(p, r.point));
        const onB = [b.from, b.to].some(p => E.pointsEqual(p, r.point));
        if (!onA || !onB) return 'sib-t-junction';
        // shared endpoint: allowed, keep checking
      }
    }
  return 'in-scope';
}

const positions = [];
for (let g = 0; g < 8; g++) {
  const t = genGame(7000 + g);
  t.slice(0, 30).forEach((s, i) => positions.push({ s, tag: `rnd${g}t${i}` }));
}
let scanned = 0, nPair = 0;
const decomp = { m1closes: 0, s2ringMatch: 0, singleRingMatch: 0, neither: 0 };
const reasons = {};
const k1pos = { n: 0, exact: 0, errs: [] };
for (const { s, tag } of positions) {
  if (nPair >= 60) break;
  if (s.actionsRemaining < 2) continue;
  scanned++;
  const color = s.turn;
  const A0 = s.areas[color];
  const e0 = s.segments[color === 'blue' ? 'red' : 'blue'].length;
  const singles = enumSingles(s, 8);
  // k1 positives only
  for (const m of singles) {
    const Y = m.result.areas[color] - A0;
    if (Y <= 1e-9) continue;
    if (k1pos.n >= 60) break;
    if (s.segments[color === 'blue' ? 'red' : 'blue'].length !== e0 &&
        m.result.segments[color === 'blue' ? 'red' : 'blue'].length !== e0) continue;
    const r = scopeReason(s.segments[color], [{ from: m.from, to: m.to }]);
    reasons['k1:' + r] = (reasons['k1:' + r] || 0) + 1;
    if (r !== 'in-scope') continue;
    const pr = predictRing(s.segments[color], [{ from: m.from, to: m.to }]);
    k1pos.n++;
    const err = Math.abs((pr.ok ? pr.pred : 0) - Y);
    k1pos.errs.push(err);
    if (err <= 1e-6) k1pos.exact++;
  }
  const firsts = dshuffle(singles, scanned * 31 + 7).slice(0, 8);
  for (const m1 of firsts) {
    if (nPair >= 60) break;
    if (m1.result.turn !== color && m1.result.actionsRemaining === 0) continue;
    const Y1 = m1.result.areas[color] - A0;
    const seconds = dshuffle(enumSingles(m1.result, 6, m1.to, 6), scanned * 131 + 13).slice(0, 10);
    for (const m2 of seconds) {
      if (nPair >= 60) break;
      const Y = m2.result.areas[color] - A0;
      if (Y <= 1e-9) continue;
      const r = scopeReason(s.segments[color],
        [{ from: m1.from, to: m1.to }, { from: m2.from, to: m2.to }]);
      reasons['k2:' + r] = (reasons['k2:' + r] || 0) + 1;
      if (r !== 'in-scope') continue;
      nPair++;
      const Y2g1 = m2.result.areas[color] - m1.result.areas[color];
      const single = predictRing(s.segments[color],
        [{ from: m1.from, to: m1.to }, { from: m2.from, to: m2.to }]);
      // sequential single-ring: s2's ring in the post-m1 context
      const seq = predictRing(m1.result.segments[color], [{ from: m2.from, to: m2.to }]);
      const singleErr = Math.abs((single.ok ? single.pred : 0) - Y);
      const seqErr = Math.abs((seq.ok ? seq.pred : 0) - Y2g1);
      if (singleErr <= 1e-6) decomp.singleRingMatch++;
      else if (Y1 > 1e-9 && seqErr <= 1e-6) { decomp.m1closes++; decomp.s2ringMatch++; }
      else if (seqErr <= 1e-6 && Y1 <= 1e-9) { decomp.s2ringMatch++; }
      else decomp.neither++;
      if (nPair <= 12)
        console.log(`${tag} Y=${Y.toFixed(2)} Y1=${Y1.toFixed(2)} Y2|1=${Y2g1.toFixed(2)} ` +
          `singleRing=${single.ok ? single.pred.toFixed(2) : 'ABSTAIN'} ` +
          `seqRing(s2|m1)=${seq.ok ? seq.pred.toFixed(2) : 'ABSTAIN'}`);
    }
  }
}
k1pos.errs.sort((a, b) => a - b);
console.log(JSON.stringify({
  scanned, nPair, decomp, reasons,
  k1: { n: k1pos.n, exact: k1pos.exact, max: k1pos.errs.length ? k1pos.errs[k1pos.errs.length - 1] : 0 },
}));
