// opp_lineset.js — Lane O Q7a probe: closed-form area-yield math for k-line sets.
// Measures ONLY. No eval/search changes. Deterministic (LCG + deterministic
// bot play). Legality via engine only (E.applyMove / E.applyTimeout /
// E.computeTerritories; every candidate move validated by applyMove try/catch).
//
// Q7a hypothesis: the area yield of a small set of k lines played together
// equals the shoelace area of the newly closed ring(s):
//   P = |shoelace(ring)|, ring = k new segs + existing-segment chains
//         joining their endpoints (BFS on own endpoint-adjacency graph).
// This probe checks P vs engine actual Y = areas_after - areas_before.
const E = require('./engine/engine.js');
const BOT = require('./bots/bot.js');

function lcg(seed) { let s = seed >>> 0; return () => (s = (s * 1664525 + 1013904223) >>> 0) / 4294967296; }
const key = p => p[0] + ',' + p[1];
const ptsEq = (a, b) => a[0] === b[0] && a[1] === b[1];

// ---- position generation: loop-biased deterministic random play ----
const OFFSETS = [];
for (let dx = -3; dx <= 3; dx++) for (let dy = -3; dy <= 3; dy++) {
  if (dx !== 0 || dy !== 0) OFFSETS.push([dx, dy]);
}
function genGame(seed) {
  const rnd = lcg(seed);
  let s = E.createInitialState();
  const turnStarts = [];
  let guard = 0;
  while (s.moveNumber < s.lineLimit && guard++ < 5000) {
    if (s.actionsRemaining === 2 || (s.moveNumber > 0 && s.currentTurnEdges.length === 0))
      turnStarts.push(s);
    const color = s.turn;
    const nodes = s.nodes[color];
    let moved = false;
    for (let t = 0; t < 60 && !moved; t++) {
      const from = nodes[Math.floor(rnd() * nodes.length)];
      let to;
      if (rnd() < 0.5) {
        // connect-biased: aim at an existing own node in range (loop-seeking)
        const cands = nodes.filter(n => !ptsEq(n, from) &&
          Math.abs(n[0] - from[0]) <= 3 && Math.abs(n[1] - from[1]) <= 3);
        to = cands.length ? cands[Math.floor(rnd() * cands.length)]
          : [from[0] + OFFSETS[Math.floor(rnd() * OFFSETS.length)][0],
             from[1] + OFFSETS[Math.floor(rnd() * OFFSETS.length)][1]];
      } else {
        const o = OFFSETS[Math.floor(rnd() * OFFSETS.length)];
        to = [from[0] + o[0], from[1] + o[1]];
      }
      try { s = E.applyMove(s, from, to); moved = true; } catch (e) { /* illegal */ }
    }
    if (!moved) { try { s = E.applyTimeout(s); } catch (e) { break; } }
  }
  return turnStarts;
}

// ---- deterministic single-move enumeration (bounded, sorted) ----
// nearPt: optional [x,y] — restrict from-nodes to within `radius` (closing
// moves attach near the frontier tip; keeps pair search local + cheap).
function enumSingles(state, fromCap, nearPt, radius) {
  const color = state.turn;
  let nodes = [...state.nodes[color]].sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  if (nearPt) {
    const r = radius || 6;
    nodes = nodes
      .map(n => ({ n, d: Math.max(Math.abs(n[0] - nearPt[0]), Math.abs(n[1] - nearPt[1])) }))
      .filter(o => o.d <= r)
      .sort((a, b) => a.d - b.d || a.n[0] - b.n[0] || a.n[1] - b.n[1])
      .map(o => o.n);
  }
  const froms = nodes.slice(0, fromCap);
  const out = [];
  for (const from of froms)
    for (const [dx, dy] of OFFSETS) {
      const to = [from[0] + dx, from[1] + dy];
      if (to[0] < 0 || to[0] >= state.boardSize || to[1] < 0 || to[1] >= state.boardSize) continue;
      try {
        const r = E.applyMove(state, from, to);
        out.push({ from: [...from], to, result: r });
      } catch (e) { /* illegal */ }
    }
  return out;
}

// ---- closed-form predictor: ring = new segs + BFS chains on endpoint graph ----
function buildAdj(segs) {
  const adj = new Map(); // key -> [{k, pt}]
  const ptOf = new Map();
  const add = p => { const k = key(p); if (!adj.has(k)) { adj.set(k, []); ptOf.set(k, p); } return k; };
  for (const s of segs) {
    const a = add(s.from), b = add(s.to);
    adj.get(a).push({ k: b }); adj.get(b).push({ k: a });
  }
  return { adj, ptOf };
}
function bfsChain(adj, fromK, toK) {
  if (fromK === toK) return [fromK];
  const prev = new Map([[fromK, null]]);
  const q = [fromK];
  while (q.length) {
    const cur = q.shift();
    for (const { k } of adj.get(cur) || []) {
      if (prev.has(k)) continue;
      prev.set(k, cur);
      if (k === toK) {
        const path = [k]; let c = k;
        while (prev.get(c) !== null) { c = prev.get(c); path.push(c); }
        return path.reverse();
      }
      q.push(k);
    }
  }
  return null;
}
// newSegs: [{from,to}] in play order. Returns {ok, pred} — pred = shoelace of
// single ring threading all new segs in order + existing chains between them.
function predictRing(ownSegs, newSegs) {
  const { adj, ptOf } = buildAdj(ownSegs);
  const ringKeys = [];
  const n = newSegs.length;
  for (let i = 0; i < n; i++) {
    const cur = newSegs[i], nxt = newSegs[(i + 1) % n];
    ringKeys.push(key(cur.from));
    const chain = bfsChain(adj, key(cur.to), key(nxt.from));
    if (!chain) return { ok: false, pred: 0 };
    ringKeys.push(...chain);
  }
  // dedupe consecutive + build ordered vertex loop
  const verts = [];
  for (const k of ringKeys) {
    const p = ptOf.get(k) || newSegs.flatMap(s => [s.from, s.to]).find(p => key(p) === k);
    if (!p) return { ok: false, pred: 0 };
    if (!verts.length || key(verts[verts.length - 1]) !== key(p)) verts.push(p);
  }
  if (verts.length && key(verts[0]) === key(verts[verts.length - 1])) verts.pop();
  if (verts.length < 3) return { ok: n === 0, pred: 0 };
  return { ok: true, pred: E.polygonArea(verts) };
}

// A new seg is in predictor scope iff it attaches at endpoints only:
// no interior crossing of own segs, no collinear overlap, no interior
// crossing with sibling new segs. (Interior endpoint-touch is fine.)
function inScope(ownSegs, newSegs) {
  const epis = s => [s.from, s.to];
  for (const ns of newSegs) {
    for (const os of ownSegs) {
      const r = E.segmentIntersect({ from: ns.from, to: ns.to }, os);
      if (r.kind === 'collinear') return false;
      if (r.kind === 'point' && !epis(ns).some(p => E.pointsEqual(p, r.point)) &&
          !epis(os).some(p => E.pointsEqual(p, r.point))) return false;
    }
  }
  for (let i = 0; i < newSegs.length; i++)
    for (let j = i + 1; j < newSegs.length; j++) {
      const a = newSegs[i], b = newSegs[j];
      // shared endpoints allowed
      const r = E.segmentIntersect({ from: a.from, to: a.to }, { from: b.from, to: b.to });
      if (r.kind === 'collinear') return false;
      if (r.kind === 'point' && ![a.from, a.to].some(p => E.pointsEqual(p, r.point))) {
        if (![b.from, b.to].some(p => E.pointsEqual(p, r.point))) return false;
        // touches b interiorly at a's endpoint: out of scope (T-junction split)
        return false;
      }
    }
  return true;
}

function enemyCount(st, color) {
  const opp = color === 'blue' ? 'red' : 'blue';
  return st.segments[opp].length;
}

function ns() { return process.hrtime.bigint(); }
const stats = {
  k1: { n: 0, exact: 0, ae: [], max: 0, unpredict: 0, tPred: 0, tDr: 0 },
  k2: { n: 0, exact: 0, ae: [], max: 0, unpredict: 0, tPred: 0, tDr: 0 },
  k3: { n: 0, exact: 0, ae: [], max: 0, unpredict: 0, tPred: 0, tDr: 0 },
};
let skippedCut = 0, skippedX = 0, tApply = 0, nApply = 0;

function record(k, st, color, ownBefore, afterAreas, newSegs, drAfter) {
  const S = stats['k' + k];
  const Y = afterAreas - ownBefore;
  let t0 = ns();
  const scope = inScope(st.segments[color], newSegs);
  let pr;
  if (scope) pr = predictRing(st.segments[color], newSegs);
  else { skippedX++; return 'xs kip'; }
  const t1 = ns();
  S.tPred += Number(t1 - t0) / 1000;
  S.tDr += drAfter;
  S.n++;
  const P = pr.ok ? pr.pred : 0;
  if (!pr.ok) S.unpredict++;
  const err = Math.abs(P - Y);
  S.ae.push(err);
  S.max = Math.max(S.max, err);
  if (err <= 1e-6) S.exact++;
  return { Y, P, err, ok: pr.ok };
}

function timedDr(segments) {
  const t0 = ns();
  const r = E.computeTerritories(JSON.parse(JSON.stringify(segments)));
  const t1 = ns();
  return { r, us: Number(t1 - t0) / 1000 };
}

function main() {
  // deterministic shuffle (LCG on position index) so slices aren't lexically biased
  function dshuffle(arr, seed) {
    const rnd = lcg(seed);
    const a = arr.slice();
    for (let i = a.length - 1; i > 0; i--) {
      const j = Math.floor(rnd() * (i + 1));
      [a[i], a[j]] = [a[j], a[i]];
    }
    return a;
  }
  const QUOTA2 = 50, QUOTA3 = 20, QUOTA1 = 50;
  const K1 = 8, K2 = 10, K3 = 8; // pair/triple enumeration caps
  const positions = [];
  for (let g = 0; g < 8; g++) {
    const t = genGame(7000 + g);
    t.slice(0, 30).forEach((s, i) => positions.push({ s, tag: `rnd${g}t${i}` }));
  }
  console.log(`positions: ${positions.length}`);
  let scanned = 0;
  const samples2 = [], samples3 = [], samples1 = [];
  for (const { s, tag } of positions) {
    if (stats.k2.n >= QUOTA2 + 15 && stats.k3.n >= QUOTA3 + 8 && stats.k1.n >= QUOTA1) break;
    if (s.actionsRemaining < 2) continue;
    scanned++;
    const color = s.turn;
    const ownBefore = s.areas[color];
    const e0 = enemyCount(s, color);
    const singles = enumSingles(s, 8);
    // k=1 calibration
    if (stats.k1.n < QUOTA1) {
      for (const m of singles.slice(0, 24)) {
        if (stats.k1.n >= QUOTA1) break;
        if (enemyCount(m.result, color) !== e0) { skippedCut++; continue; }
        const d = timedDr(m.result.segments);
        record(1, s, color, ownBefore, m.result.areas[color],
          [{ from: m.from, to: m.to }], d.us);
      }
    }
    // k=2 pairs: second moves attach near the first move's tip (local search)
    const firsts = dshuffle(singles, scanned * 31 + 7).slice(0, K1);
    const closingPairs = [];
    if (stats.k2.n < QUOTA2 + 15) {
      for (const m1 of firsts) {
        if (enemyCount(m1.result, color) !== e0) { skippedCut++; continue; }
        if (m1.result.actionsRemaining === 0 && m1.result.turn !== color) continue;
        const m1e0 = enemyCount(m1.result, color);
        const seconds = dshuffle(enumSingles(m1.result, 6, m1.to, 6), scanned * 131 + 13).slice(0, K2);
        for (const m2 of seconds) {
          if (enemyCount(m2.result, color) !== m1e0) { skippedCut++; continue; }
          const tA0 = ns();
          void tA0;
          const Y = m2.result.areas[color] - ownBefore;
          if (Y <= 1e-9) continue;
          const d = timedDr(m2.result.segments);
          const rec = record(2, s, color, ownBefore, m2.result.areas[color],
            [{ from: m1.from, to: m1.to }, { from: m2.from, to: m2.to }], d.us);
          if (rec !== 'xskip' && typeof rec === 'object')
            samples2.push(`${tag} Y=${rec.Y.toFixed(2)} P=${rec.P.toFixed(2)} err=${rec.err.toExponential(1)} ring=${rec.ok}`);
          if (Y > 1e-9) closingPairs.push({ m1, m2, Y });
          if (stats.k2.n >= QUOTA2 + 15) break;
        }
        if (stats.k2.n >= QUOTA2 + 15) break;
      }
    }
    // k=3: extend closing pairs (additivity) + cold triples (pair Y==0, triple Y>0)
    // Triple = turn(m1,m2) + foe applyTimeout (engine-legal pass, no seg change) + m3.
    function passBack(st) {
      let mid = st;
      if (mid.turn !== color) { try { mid = E.applyTimeout(mid); } catch (e) { return null; } }
      return mid.turn === color ? mid : null;
    }
    if (stats.k3.n < QUOTA3 + 8) {
      const exts = closingPairs.slice(0, 6);
      // cold-triple search: non-closing firsts x sampled seconds
      if (stats.k3.n < QUOTA3) {
        for (const m1 of firsts.slice(0, 6)) {
          if (m1.result.turn !== color) continue;
          if (Math.abs(m1.result.areas[color] - ownBefore) > 1e-9) continue; // pair handled above
          const seconds = dshuffle(enumSingles(m1.result, 6, m1.to, 6), scanned * 131 + 13).slice(0, 6);
          for (const m2 of seconds) {
            if (Math.abs(m2.result.areas[color] - ownBefore) > 1e-9) continue;
            // turn(m1,m2) ends our turn -> foe; pass back via engine-legal timeout
            const mid = passBack(m2.result);
            if (!mid) continue;
            const thirds = enumSingles(mid, 4, m2.to, 6).slice(0, K3);
            for (const m3 of thirds) {
              if (enemyCount(m3.result, color) !== e0) { skippedCut++; continue; }
              const Y = m3.result.areas[color] - ownBefore;
              if (Y <= 1e-9) continue;
              const d = timedDr(m3.result.segments);
              const rec = record(3, s, color, ownBefore, m3.result.areas[color],
                [{ from: m1.from, to: m1.to }, { from: m2.from, to: m2.to },
                 { from: m3.from, to: m3.to }], d.us);
              if (typeof rec === 'object')
                samples3.push(`${tag}COLD Y=${rec.Y.toFixed(2)} P=${rec.P.toFixed(2)} err=${rec.err.toExponential(1)} ring=${rec.ok}`);
              if (stats.k3.n >= QUOTA3 + 8) break;
            }
            if (stats.k3.n >= QUOTA3 + 8) break;
          }
          if (stats.k3.n >= QUOTA3 + 8) break;
        }
      }
      for (const { m1, m2 } of exts) {
        if (stats.k3.n >= QUOTA3 + 8) break;
        const mid = passBack(m2.result);
        if (!mid) continue;
        const thirds = enumSingles(mid, 4, m2.to, 6).slice(0, K3);
        for (const m3 of thirds) {
          if (enemyCount(m3.result, color) !== e0) { skippedCut++; continue; }
          const Y = m3.result.areas[color] - ownBefore;
          if (Y <= 1e-9) continue;
          const d = timedDr(m3.result.segments);
          const rec = record(3, s, color, ownBefore, m3.result.areas[color],
            [{ from: m1.from, to: m1.to }, { from: m2.from, to: m2.to },
             { from: m3.from, to: m3.to }], d.us);
          if (typeof rec === 'object')
            samples3.push(`${tag}EXT Y=${rec.Y.toFixed(2)} P=${rec.P.toFixed(2)} err=${rec.err.toExponential(1)} ring=${rec.ok}`);
          if (stats.k3.n >= QUOTA3 + 8) break;
        }
      }
    }
    if (scanned % 10 === 0)
      console.log(`scanned=${scanned} k1=${stats.k1.n} k2=${stats.k2.n} k3=${stats.k3.n} xskip=${skippedX} cutskip=${skippedCut}`);
  }
  console.log(`scanned=${scanned} positions`);
  const summ = k => {
    const S = stats[k];
    const aes = [...S.ae].sort((a, b) => a - b);
    const mean = aes.length ? aes.reduce((a, b) => a + b, 0) / aes.length : 0;
    const p50 = aes.length ? aes[Math.floor(aes.length * 0.5)] : 0;
    const p90 = aes.length ? aes[Math.floor(aes.length * 0.9)] : 0;
    return { n: S.n, exact: S.exact, mean, p50, p90, max: S.max, unpredict: S.unpredict, tPred: S.tPred, tDr: S.tDr };
  };
  const out = {
    k1: summ('k1'), k2: summ('k2'), k3: summ('k3'),
    skippedX, skippedCut, scanned,
    s2: samples2.slice(0, 60), s3: samples3.slice(0, 30),
  };
  console.log(JSON.stringify(out, null, 1));
}

if (require.main === module) main();

module.exports = { predictRing, inScope, enumSingles, genGame, lcg };
