// Track C verification script — engine zero-divergence checks. Run: node verifyC.js
const E = require('./engine.js');
let pass = 0, fail = 0;
function t(name, fn) {
  try { fn(); pass++; console.log('PASS ' + name); }
  catch (e) { fail++; console.log('FAIL ' + name + ' :: ' + (e && e.message)); }
}
function assert(c, m) { if (!c) throw new Error(m); }
function seg(f, tt, inv) { return { from: [...f], to: [...tt], invincible: !!inv }; }
// Build a state from segment lists; nodes derived from endpoints unless overridden.
function S(o) {
  const s = E.createInitialState();
  s.segments.blue = (o.bt || []).map(([f, tt]) => seg(f, tt, o.inv));
  s.segments.red = (o.rt || []).map(([f, tt]) => seg(f, tt, o.inv));
  const eps = (pair) => pair.map(p => p.join(','));
  const un = (pair) => [...new Set(eps(pair))].map(k => k.split(',').map(Number));
  s.nodes.blue = o.bn || un((o.bt || []).flat());
  s.nodes.red = o.rn || un((o.rt || []).flat());
  s.turn = o.turn || 'blue';
  s.actionsRemaining = o.actions !== undefined ? o.actions : 2;
  s.moveNumber = o.move || 0;
  if (o.scores) s.scores = { ...o.scores };
  s.currentTurnEdges = (o.cte || []).map(([f, tt]) => seg(f, tt, true));
  return s;
}
const SQ_B = [[[0,0],[3,0]], [[3,0],[3,3]], [[3,3],[0,3]], [[0,3],[0,0]]]; // area 9
const SQ_R = [[[10,10],[12,10]], [[12,10],[12,12]], [[12,12],[10,12]], [[10,12],[10,10]]]; // area 4
const INNER = [[[1,1],[2,1]], [[2,1],[2,2]], [[2,2],[1,2]], [[1,2],[1,1]]]; // area 1

t('init: board/starts', () => {
  const s = E.createInitialState();
  assert(s.boardSize === 19, 'boardSize=' + s.boardSize);
  assert(s.placeRadius === 3, 'radius=' + s.placeRadius);
  assert(s.lineLimit === 120, 'limit=' + s.lineLimit);
  assert(s.turn === 'blue' && s.actionsRemaining === 1, 'blue first, 1 action');
  assert(JSON.stringify(s.nodes.blue) === '[[0,9],[3,9]]', 'blue nodes');
  assert(JSON.stringify(s.nodes.red) === '[[18,9],[15,9]]', 'red nodes');
});

t('T1 collinear-overlap with enemy CUTS (not illegal)', () => {
  // red (5,9)-(8,9); blue node (3,9), own seg (1,9)-(3,9) disjoint from red
  let s = S({ bt: [[[1,9],[3,9]]], rt: [[[5,9],[8,9]]], turn: 'blue', actions: 2 });
  s = E.applyMove(s, [3,9], [6,9]); // overlaps red over [5,6]
  assert(s.segments.red.length === 0, 'red seg removed, got ' + s.segments.red.length);
  assert(s.nodes.red.length === 0, 'both red endpoints isolated+removed, got ' + JSON.stringify(s.nodes.red));
  assert(s.segments.blue.length === 2, 'blue has 2 segs');
});

t('T1b collinear-overlap with OWN throws', () => {
  // own (0,9)-(3,9); from own node (2,9) play (2,9)-(5,9): overlaps own over [2,3]
  let s = S({ bt: [[[0,9],[3,9]]], bn: [[0,9],[3,9],[2,9]], rt: [], turn: 'blue', actions: 2 });
  let err = null;
  try { E.applyMove(s, [2,9], [5,9]); } catch (e) { err = e.message; }
  assert(/overlap/i.test(err || ''), 'expected overlap throw, got: ' + err);
  // control: touching own edge exactly at endpoint is LEGAL (point, not collinear)
  const s2 = S({ bt: [[[0,9],[3,9]]], rt: [], turn: 'blue', actions: 2 });
  E.applyMove(s2, [3,9], [5,9]);
});

t('T2 touch enemy exactly at ITS endpoint still CUTS whole edge', () => {
  // red (8,9)-(11,9); blue (3,9)-(5,9); play (5,9)-(8,9) touches red at (8,9)
  let s = S({ bt: [[[3,9],[5,9]]], rt: [[[8,9],[11,9]]], turn: 'blue', actions: 2 });
  s = E.applyMove(s, [5,9], [8,9]);
  assert(s.segments.red.length === 0, 'touch at endpoint cut whole edge, red segs=' + s.segments.red.length);
});

t('T2b endpoint landing on enemy INTERIOR (T-junction) cuts whole edge', () => {
  // red vertical (8,8)-(8,12); blue node (5,9); play (5,9)-(8,9), to lands mid-edge
  let s = S({ bt: [[[3,9],[5,9]]], rt: [[[8,8],[8,12]]], turn: 'blue', actions: 2 });
  s = E.applyMove(s, [5,9], [8,9]);
  assert(s.segments.red.length === 0, 'T-junction endpoint cut whole 4-long edge');
});

t('T2c from-node sitting on enemy line, moving away, breaks it', () => {
  // red (4,9)-(8,9); blue node (6,9) coincides with enemy interior; play (6,9)-(6,12)
  let s = S({ bt: [], rt: [[[4,9],[8,9]]], bn: [[6,9]], turn: 'blue', actions: 2 });
  s = E.applyMove(s, [6,9], [6,12]);
  assert(s.segments.red.length === 0, 'moving off enemy line broke it');
});

t('T3 crossing TWO enemy edges throws at-most-one', () => {
  let s = S({ bt: [[[3,10],[5,10]]], rt: [[[6,8],[6,12]], [[8,8],[8,12]]], turn: 'blue', actions: 2 });
  let err = null;
  try { E.applyMove(s, [5,10], [8,10]); } catch (e) { err = e.message; }
  assert(/at most one/i.test(err || ''), 'expected at-most-one throw, got: ' + err);
});

t('T3b passing through shared enemy corner (2 edges, 1 point) throws', () => {
  let s = S({ bt: [[[1,10],[3,10]]], rt: [[[6,8],[6,10]], [[6,10],[8,10]]], turn: 'blue', actions: 2 });
  let err = null;
  try { E.applyMove(s, [3,10], [6,10]); } catch (e) { err = e.message; }
  assert(/at most one/i.test(err || ''), 'expected at-most-one throw, got: ' + err);
});

t('T4 cut leaves 3-degree node alive, only isolated node removed', () => {
  // red star at (5,5): segs to (8,5),(5,8),(2,5). blue node (6,4), seg (4,4)-(6,4); play (6,4)-(6,7) crosses (5,5)-(8,5) at (6,5)
  let s = S({
    bt: [[[4,4],[6,4]]],
    rt: [[[5,5],[8,5]], [[5,5],[5,8]], [[5,5],[2,5]]],
    turn: 'blue', actions: 2
  });
  s = E.applyMove(s, [6,4], [6,7]);
  assert(s.segments.red.length === 2, 'one red seg cut, got ' + s.segments.red.length);
  const rk = s.nodes.red.map(p => p.join(','));
  assert(rk.includes('5,5'), 'degree-3 node (5,5) survives: ' + JSON.stringify(s.nodes.red));
  assert(!rk.includes('8,5'), 'isolated (8,5) removed');
  assert(rk.includes('5,8') && rk.includes('2,5'), 'other branch endpoints kept');
});

t('T5 boundary: 0 and 18 legal, -1/19 illegal', () => {
  let s = S({ bt: [], bn: [[2,0]], rt: [], turn: 'blue', actions: 2 });
  s = E.applyMove(s, [2,0], [0,0]);
  assert(s.nodes.blue.some(p => p[0] === 0 && p[1] === 0), 'x=0 placement ok');
  let s2 = S({ bt: [], bn: [[16,17]], rt: [], turn: 'blue', actions: 2 });
  s2 = E.applyMove(s2, [16,17], [18,18]);
  assert(true, 'x=18,y=18 ok');
  for (const bad of [[[2,0],[2,-1]], [[16,17],[19,17]], [[0,0],[0,19]]]) {
    const sb = S({ bt: [], bn: [bad[0]], rt: [], turn: 'blue', actions: 2 });
    let err = null;
    try { E.applyMove(sb, bad[0], bad[1]); } catch (e) { err = e.message; }
    assert(/outside/i.test(err || ''), `to=${bad[1]} should throw outside, got: ${err}`);
  }
});

t('T6 radius box is Chebyshev: (3,3) ok, (4,0)/(0,4) throw', () => {
  const sb = S({ bt: [], bn: [[9,9]], rt: [], turn: 'blue', actions: 2 });
  E.applyMove(sb, [9,9], [12,12]);
  for (const bad of [[13,9],[9,13],[12,13]]) {
    const s2 = S({ bt: [], bn: [[9,9]], rt: [], turn: 'blue', actions: 2 });
    let err = null;
    try { E.applyMove(s2, [9,9], bad); } catch (e) { err = e.message; }
    assert(/placement box/i.test(err || ''), `to=${bad} should throw box, got: ${err}`);
  }
});

t('T7 nested same-color: only outer counts (9, not 10)', () => {
  const r = E.computeTerritories({
    blue: [...SQ_B, ...INNER].map(([f, tt]) => seg(f, tt)),
    red: []
  });
  assert(r.areas.blue === 9, 'nested same-color areas.blue=' + r.areas.blue + ' (want 9)');
  const ctrl = E.computeTerritories({ blue: SQ_B.map(([f, tt]) => seg(f, tt)), red: [] });
  assert(ctrl.areas.blue === 9, 'control outer-alone=' + ctrl.areas.blue);
});

t('T8 cross-color nesting DOUBLE counts (blue 9 AND red 1)', () => {
  const r = E.computeTerritories({
    blue: SQ_B.map(([f, tt]) => seg(f, tt)),
    red: INNER.map(([f, tt]) => seg(f, tt))
  });
  assert(r.areas.blue === 9 && r.areas.red === 1,
    `cross-color areas=${JSON.stringify(r.areas)} (blue 9 incl. red interior + red 1)`);
});

t('T9 per-turn cumulative scoring banks BOTH, every turn end', () => {
  let s = S({ bt: SQ_B, rt: SQ_R, turn: 'blue', actions: 1, scores: { blue: 0, red: 0 } });
  s = E.applyTimeout(s);
  assert(s.scores.blue === 9 && s.scores.red === 4, 'both banked: ' + JSON.stringify(s.scores));
  s = E.applyTimeout(s); // red passes; standing loops pay again
  assert(s.scores.blue === 18 && s.scores.red === 8, 'banks AGAIN each turn: ' + JSON.stringify(s.scores));
});

t('T10 truncation: 1 move left -> 1-action turn, scoring resolves, game locks', () => {
  let s = E.createInitialState();
  s.moveNumber = 119; s.actionsRemaining = 1; s.turn = 'blue';
  s = E.applyMove(s, [3,9], [4,9]);
  assert(s.moveNumber === 120, 'moveNumber=120');
  assert(s.turn === 'red' && s.actionsRemaining === 0, 'flips w/ 0 actions: ' + s.turn + '/' + s.actionsRemaining);
  let e1 = null; try { E.applyMove(s, [15,9], [14,9]); } catch (e) { e1 = e.message; }
  assert(/complete/i.test(e1 || ''), 'move after end throws, got: ' + e1);
  let e2 = null; try { E.applyTimeout(s); } catch (e) { e2 = e.message; }
  assert(/complete/i.test(e2 || ''), 'timeout after end throws, got: ' + e2);
});

t('T10b blue opener gets 1 action, then red gets 2', () => {
  let s = E.createInitialState();
  s = E.applyMove(s, [3,9], [4,9]);
  assert(s.turn === 'red' && s.actionsRemaining === 2, 'red gets 2: ' + s.actionsRemaining);
});

t('T11 timeout banks area, flips turn/invincibility like a played turn-end', () => {
  // blue owns 9-area square (vincible) + just-played edge E (3,3)-(5,3); mid-turn timeout
  let s = S({ bt: [...SQ_B, [[3,3],[5,3]]], rt: SQ_R, turn: 'blue', actions: 1, move: 10,
    scores: { blue: 100, red: 50 }, cte: [[[3,3],[5,3]]] });
  s = E.applyTimeout(s);
  assert(s.moveNumber === 11, 'timeout consumed 1 remaining action, move=' + s.moveNumber);
  assert(s.scores.blue === 109 && s.scores.red === 54, 'banked both: ' + JSON.stringify(s.scores));
  assert(s.turn === 'red' && s.actionsRemaining === 2, 'turn flips w/ 2 actions');
  const e5 = s.segments.blue.find(g => g.from.join() === '3,3' && g.to.join() === '5,3');
  assert(e5 && e5.invincible === true, 'current-turn edge invincible');
  assert(s.segments.blue.filter(g => !(g.from.join() === '3,3' && g.to.join() === '5,3')).every(g => !g.invincible), 'older edges vincible');
});

t('T11b timeout with 2 remaining BURNS 2 of the 120 budget without placing', () => {
  let s = S({ bt: SQ_B, rt: [], turn: 'blue', actions: 2, move: 10 });
  const n = s.segments.blue.length;
  s = E.applyTimeout(s);
  assert(s.moveNumber === 12, 'move 10->12 on 2-action timeout, got ' + s.moveNumber);
  assert(s.segments.blue.length === n, 'no segments placed');
});

t('T12 invincibility lifecycle: 1 opp turn, then gone; cleared at each turn end', () => {
  let s = E.createInitialState();
  s = E.applyMove(s, [3,9], [6,9]); // blue turn ends
  const fresh = s.segments.blue.find(g => g.from.join() === '3,9');
  const stale = s.segments.blue.find(g => g.from.join() === '0,9');
  assert(fresh && fresh.invincible === true, 'just-played edge invincible after own turn');
  assert(stale && stale.invincible === false, 'stale edge (never played) stays vincible');
  s = E.applyMove(s, [15,9], [12,9]); // red action 1 of 2: turn NOT over
  assert(s.turn === 'red' && s.actionsRemaining === 1, 'red mid-turn');
  assert(fresh && s.segments.blue.find(g => g.from.join() === '3,9').invincible === true,
    'blue edge stays invincible through ALL of opp turn (incl. 2nd action window)');
  s = E.applyMove(s, [12,9], [12,12]); // red action 2 of 2: turn ends
  assert(s.segments.blue.every(g => !g.invincible), 'blue edges cleared after opp turn');
  assert(s.segments.red.filter(g => g.from.join() !== '18,9').every(g => g.invincible), 'red just-played edges invincible after own turn');
  assert(s.segments.red.find(g => g.from.join() === '18,9').invincible === false, 'red stale edge stays vincible');
  // crafted: red tries to cut invincible blue edge -> throws; after expiry -> cuts
  let c = S({ bt: [[[9,9],[12,9]]], rt: [], bn: [[9,9],[12,9]], rn: [[10,10]], rt2: null,
    turn: 'red', actions: 2 });
  c.segments.red = [seg([10,11], [10,10])];
  c.nodes.red = [[10,11],[10,10]];
  c.segments.blue[0].invincible = true;
  let err = null;
  try { E.applyMove(c, [10,10], [10,8]); } catch (e) { err = e.message; }
  assert(/invincible/i.test(err || ''), 'cutting invincible throws, got: ' + err);
  c.segments.blue[0].invincible = false; // simulate a full round passing
  c = E.applyMove(c, [10,10], [10,8]); // red's turn: cuts BLUE's expired edge
  assert(c.segments.blue.length === 0 && c.segments.red.length === 2, 'expired edge cut ok');
});

t('T12b mid-turn own edge shows invincible:true (observable, harmless)', () => {
  let s = S({ bt: [], bn: [[9,9]], rt: [], turn: 'blue', actions: 2 });
  s = E.applyMove(s, [9,9], [10,10]);
  assert(s.turn === 'blue' && s.actionsRemaining === 1, 'still mid-turn');
  assert(s.segments.blue[0].invincible === true, 'new edge flagged invincible mid-turn');
});

t('T13 break only piggybacks expansion (no standalone break API)', () => {
  assert(typeof E.applyMove === 'function' && typeof E.applyTimeout === 'function', 'only two mutators');
  // timeout never removes enemy segs:
  let s = S({ bt: SQ_B, rt: SQ_R, turn: 'blue', actions: 2, cte: [] });
  const n = s.segments.red.length;
  s = E.applyTimeout(s);
  assert(s.segments.red.length === n, 'timeout removes nothing');
});

t('T14 wiped player (no nodes) can only timeout; timeout still works', () => {
  const s = S({ bt: [[[0,9],[2,9]]], rt: [], rn: [], turn: 'red', actions: 2, move: 5 });
  let err = null;
  try { E.applyMove(s, [0,0], [1,1]); } catch (e) { err = e.message; }
  assert(/existing nodes/i.test(err || ''), 'got: ' + err);
  const s2 = E.applyTimeout(s);
  assert(s2.turn === 'blue' && s2.moveNumber === 7, 'timeout flips + burns 2');
});

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
