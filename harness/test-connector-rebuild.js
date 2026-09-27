// test-connector-rebuild.js — FINAL-LOCK job 3: unit-test connector.js's
// rebuild()/pt() logic paths against simulated server payloads WITHOUT
// requiring connector.js (it connects on load). pt() and rebuild() are copied
// verbatim from connector.js lines 71-86; the onState resync decision
// (historyLength vs moveHistory.length) is replicated from lines 129-137.
const E = require('../engine/engine.js');
const T = require('../bots/bot-tourney.js');

// ---- copied verbatim from connector.js ----
function pt(p) {
  if (Array.isArray(p)) return [p[0], p[1]];
  return [p.x, p.y];
}
function say(...a) { console.log(...a); }
function rebuild(history) {
  let s = E.createInitialState();
  for (const h of history) {
    try {
      if (h.pass) s = E.applyTimeout(s);
      else s = E.applyMove(s, pt(h.from), pt(h.to));
    } catch (e) { say('REBUILD-DIVERGENCE at', JSON.stringify(h), '::', e.message); return null; }
  }
  return s;
}
// replicated onState head (connector.js lines 129-137)
function onStateMirrorDecision(F, mirrorIn) {
  let mirror = mirrorIn, resync = false;
  if (Array.isArray(F.moveHistory)) {
    const m = rebuild(F.moveHistory);
    if (m) { mirror = m; }
    else { resync = true; return { mirror, resync, thought: false }; }
  }
  if (Number(F.historyLength || 0) > (Array.isArray(F.moveHistory) ? F.moveHistory.length : 0)) {
    resync = true; return { mirror, resync, thought: false };
  }
  return { mirror, resync, thought: true };
}

// ---- build a real legal reference game (bot-tourney @40ms, 24 turns + 1 pass) ----
let s = E.createInitialState();
const refMoves = [];
let guard = 0, passInserted = false;
while (s.moveNumber < s.lineLimit && guard++ < 24) {
  const t = T.bestTurn(s, 40);
  if (!t || !t.moves || !t.moves.length) {
    if (!passInserted) { refMoves.push({ __pass: true }); passInserted = true; } // mark; payload shapes below expand it
    s = E.applyTimeout(s);
    continue;
  }
  let r = s;
  let last = null;
  for (const m of t.moves) { r = E.applyMove(r, m.from, m.to); last = m; }
  refMoves.push({ from: [...last.from], to: [...last.to] });
  s = r;
}
// Reference mirror = replay of the exact sequence actually applied.
const refSeq = [];
let sref = E.createInitialState();
for (const mm of refMoves) {
  if (mm.__pass) { sref = E.applyTimeout(sref); refSeq.push({ pass: true }); continue; }
  sref = E.applyMove(sref, mm.from, mm.to);
  refSeq.push({ from: [...mm.from], to: [...mm.to] });
}
const snap = (st) => JSON.stringify({ scores: st.scores, turn: st.turn,
  segB: st.segments.blue.length, segR: st.segments.red.length, mn: st.moveNumber, ar: st.actionsRemaining });
const REF = snap(sref);
say('reference game: ' + refSeq.length + ' entries, mirror=' + REF);
say('initial state turn=' + E.createInitialState().turn + ' actionsRemaining=' + E.createInitialState().actionsRemaining + ' lineLimit=' + E.createInitialState().lineLimit);
let passIdx = refSeq.findIndex(e => e.pass);
say('pass entry at index: ' + passIdx);

let failures = 0, oddities = [];
function check(name, F, expect) {
  const { mirror, resync, thought } = onStateMirrorDecision(F, E.createInitialState());
  const got = mirror ? snap(mirror) : 'null';
  const ok = expect === 'MIRROR' ? got === REF
    : expect === 'RESYNC' ? resync === true
    : expect === 'MIRROR-STALE-INITIAL' ? resync === true && got === snap(E.createInitialState())
    : false;
  if (!ok) failures++;
  say(`${ok ? 'PASS' : 'FAIL'}  ${name}  -> mirror=${got === 'null' ? 'null' : got.slice(0, 80)} resync=${resync} (expected ${expect})`);
  return { mirror, resync, got };
}

// Shape A: [x,y] array points (the common server shape)
check('A: {from:[x,y],to:[x,y]}', { type: 'game_history', moveHistory: refSeq.map(e => e.pass ? { pass: true } : { from: e.from, to: e.to }), historyLength: refSeq.length }, 'MIRROR');

// Shape B: {x,y} object points
check('B: {from:{x,y},to:{x,y}}', { type: 'game_history', moveHistory: refSeq.map(e => e.pass ? { pass: true } : { from: { x: e.from[0], y: e.from[1] }, to: { x: e.to[0], y: e.to[1] } }), historyLength: refSeq.length }, 'MIRROR');

// Shape B2: MIXED arrays and objects within one entry
{
  const hist = refSeq.map((e, i) => e.pass ? { pass: true }
    : (i % 2 === 0 ? { from: e.from, to: { x: e.to[0], y: e.to[1] } } : { from: { x: e.from[0], y: e.from[1] }, to: e.to }));
  check('B2: mixed array/object points', { type: 'game_history', moveHistory: hist, historyLength: refSeq.length }, 'MIRROR');
}

// Shape C: {pass:true} entries already inside refSeq (mid-history pass)
check('C: {pass:true} entries', { type: 'game_history', moveHistory: refSeq.map(e => e.pass ? { pass: true } : { from: e.from, to: e.to }), historyLength: refSeq.length }, 'MIRROR');

// Shape C2: pass with extra fields / pass:1 truthy
{
  const hist = refSeq.map(e => e.pass ? { pass: 1, by: 'someone', t: 123 } : { from: e.from, to: e.to, actionID: 7 });
  check('C2: pass:1 + extra fields', { type: 'game_history', moveHistory: hist, historyLength: refSeq.length }, 'MIRROR');
}

// Shape D: empty moveHistory + historyLength>0
check('D: empty moveHistory, historyLength>0', { type: 'game_history', moveHistory: [], historyLength: refSeq.length }, 'MIRROR-STALE-INITIAL');

// Shape D2: moveHistory missing entirely + historyLength>0
check('D2: no moveHistory key, historyLength>0', { type: 'game_history', historyLength: refSeq.length }, 'MIRROR-STALE-INITIAL');

// Shape D3: empty moveHistory + historyLength 0 (empty game)
{
  const { mirror, resync, thought } = onStateMirrorDecision({ type: 'game_history', moveHistory: [], historyLength: 0 }, E.createInitialState());
  const ok = resync === false && thought === true && snap(mirror) === snap(E.createInitialState());
  if (!ok) failures++;
  say(`${ok ? 'PASS' : 'FAIL'}  D3: empty moveHistory, historyLength=0 -> thinks on empty board (expected: think, no resync)`);
}

// Adversarial extras — shapes that may break the mirror:
// X1: null points (malformed entry)
check('X1: {from:null,to:null} malformed', { type: 'game_history', moveHistory: [{ from: null, to: null }, ...refSeq.slice(1)], historyLength: refSeq.length }, 'RESYNC');

// X2: pass when actions were already exhausted (redundant server pass) —
// applyTimeout would skip the OPPONENT's next turn: silent divergence probe.
{
  const before = snap(sref);
  let probe;
  try { probe = E.applyTimeout(sref); } catch (e) { probe = null; }
  if (probe === null) { say('NOTE  X2: applyTimeout throws on exhausted-turn state (caught by rebuild -> resync; safe)'); }
  else {
    const after = snap(probe);
    if (after !== before) oddities.push('X2: redundant pass with actionsRemaining exhausted SILENTLY flips turn (mirror divergence if server ever sends one)');
    say(`NOTE  X2: applyTimeout on post-turn state -> ${after} (was ${before})`);
  }
}

// X3: historyLength > moveHistory.length (server ahead) with valid prefix
check('X3: valid prefix but historyLength bigger', { type: 'game_history', moveHistory: refSeq.slice(0, 5).map(e => e.pass ? { pass: true } : { from: e.from, to: e.to }), historyLength: refSeq.length }, 'MIRROR-STALE-INITIAL');

// X4: numeric-string points "3"
{
  const hist = refSeq.map(e => e.pass ? { pass: true } : { from: [String(e.from[0]), String(e.from[1])], to: [String(e.to[0]), String(e.to[1])] });
  const r = check('X4: numeric-string points', { type: 'game_history', moveHistory: hist, historyLength: refSeq.length }, 'MIRROR');
}

// X5: float drift points (3.0000000001)
{
  const hist = refSeq.map(e => e.pass ? { pass: true } : { from: [e.from[0] + 1e-10, e.from[1] + 1e-10], to: [e.to[0] + 1e-10, e.to[1] + 1e-10] });
  check('X5: 1e-10 float drift', { type: 'game_history', moveHistory: hist, historyLength: refSeq.length }, 'MIRROR');
}

say('');
say(oddities.length ? 'ODDITIES:\n' + oddities.join('\n') : 'no silent-diversion oddities found');
say(failures === 0 ? `ALL CHECKS PASSED (failures=${failures})` : `FAILURES=${failures}`);
