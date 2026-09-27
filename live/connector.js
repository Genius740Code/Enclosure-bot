// connector.js — live tournament connector (Enclosure @ meaf.us/sst1).
// Protocol reverse-engineered from the site's own JS bundle (index-BCoKy2Nw.js).
//
//   WS: wss://meaf.us/sst1/ws
//   -> {action:"syn", username, passH1}            <- {type:"syn_ack"}
//   -> {action:"tournament_subscribe", tournamentID}
//      <- {type:"tournament_user_state", assignment:{gameID,...}}
//   -> {action:"join", gameID}                     (or spectate:true)
//      <- {type:"game_state"|"game_history", gameID, blue, red, turn,
//          phase, revision, historyLength, moveHistory:[{from,to}|{pass}], ...}
//   -> {action:"move", from:[x,y], to:[x,y], expectedRevision, actionID}
//      <- {type:"action_ack", actionID}  / {type:"error", message}
//   -> {action:"request_game_history"}             (resync when behind)
//
// State sync: EVERY server moveHistory entry is replayed through engine.js,
// so the local mirror is bit-exact (scores, invincibility, turn).
// Points are [x,y] 0..18 — identical to engine.js, no coordinate mapping.
//
// Env: ENC_USER, ENC_PASS (required; never hardcoded),
//   ENC_TOURNAMENT (default mt5x4j77), ENC_GAME (join directly, skip tournament),
//   DRY_RUN=1 (connect + sync + compute, but never send moves),
//   BUDGET_MS (cap per think, default 8000).
const WebSocket = require('ws');
const fs = require('fs');
const E = require('../engine/engine.js');
const T = require('../bots/bot-tourney.js');

const USER = process.env.ENC_USER;
const PASS = process.env.ENC_PASS;
const TOURNAMENT = process.env.ENC_TOURNAMENT || 'mt5x4j77';
const DIRECT_GAME = process.env.ENC_GAME || null;
const SPECTATE = process.env.ENC_SPECTATE === '1';
const LOBBY = process.env.LOBBY === '1';
const log = fs.createWriteStream(process.env.CONN_LOG || 'connector.log', { flags: 'a' });
const DRY_RUN = process.env.DRY_RUN === '1';
const BUDGET = parseInt(process.env.BUDGET_MS || '8000', 10);
const WS_URL = 'wss://meaf.us/sst1/ws';
if (!USER || !PASS) { console.error('set ENC_USER and ENC_PASS'); process.exit(1); }

function say(...a) { const l = `[${new Date().toISOString()}] ${a.join(' ')}`; console.log(l); log.write(l + '\n'); }
function mask(o) { const s = JSON.stringify(o); return s.replace(/"passH1":"[^"]*"/, '"passH1":"***"'); }

let ws = null, actionID = 0, connected = false;
let sessionCookie = '';
let joinRetries = 0;
async function login() {
  const ctl = new AbortController();
  const to = setTimeout(() => ctl.abort(), 15000);
  let r;
  try {
    r = await fetch('https://meaf.us/sst1/api/access', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: USER, passH1: PASS }),
      signal: ctl.signal,
    });
  } finally { clearTimeout(to); }
  const setCookie = r.headers.get('set-cookie') || '';
  const m = setCookie.match(/sst1_access=[^;]+/);
  if (!r.ok || !m) throw new Error('login failed: HTTP ' + r.status);
  sessionCookie = m[0];
  say('login ok (cookie captured, masked)');
}
let gameID = null, myColor = null, serverRev = -1, serverState = null;
let mirror = E.createInitialState(); // local engine mirror
let thinking = false, closed = false;
// Local clock: 60s base + 15s increment per completed own turn.
let clockMs = 60000;
let turnStart = 0;

function send(o) {
  if (!ws || ws.readyState !== WebSocket.OPEN) { say('SEND-FAIL (offline):', mask(o)); return false; }
  ws.send(JSON.stringify(o));
  if (o.action !== 'syn') say('>>', mask(o));
  if (o.action === 'join') lastJoinAt = Date.now();
  return true;
}

// Server points may be [x,y] arrays or {x,y} objects — normalize.
function pt(p) {
  if (Array.isArray(p)) return [Number(p[0]), Number(p[1])];
  return [Number(p.x), Number(p.y)];
}

// Rebuild local mirror from server moveHistory.
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

function isOurTurn(F) {
  if (!F || !myColor || F.phase === 'done') return false;
  return F.turn === myColor || F.currentPlayer === USER;
}

function maybeThink() {
  if (!serverState || serverState.phase === 'done' || thinking) return;
  if (!isOurTurn(serverState)) return;
  // It's our turn on the server. Mirror should agree; if not, resync first.
  if (mirror.turn !== myColor) { say('mirror/server turn mismatch — resyncing'); send({ action: 'request_game_history' }); return; }
  thinking = true;
  turnStart = Date.now();
  setImmediate(() => {
    let plan = null;
    try { plan = T.bestTurn(mirror, { clockMs }); } catch (e) { say('THINK-ERROR', e.message); }
    const dt = Date.now() - turnStart;
    if (!plan || !plan.moves.length) {
      say(`no legal move (dt=${dt}ms) — standing by for server timeout`);
      thinking = false;
      return;
    }
    // Send actions ONE at a time; server bumps revision per action and we
    // re-verify against fresh state before sending the next.
    pendingPlan = plan.moves.slice(1); // rest after the first
    const m0 = plan.moves[0];
    clockMs -= dt;
    if (DRY_RUN) { say(`DRY-RUN would play ${m0.from}->${m0.to} (dt=${dt}ms, rev=${serverRev})`); thinking = false; pendingPlan = []; return; }
    actionID++;
    say(`playing ${m0.from}->${m0.to} (dt=${dt}ms, rev=${serverRev}, clock~${Math.round(clockMs / 1000)}s)`);
    send({ action: 'move', from: m0.from, to: m0.to, expectedRevision: serverRev, actionID });
    // thinking stays true until server state confirms turn end / next action.
  });
}
let pendingPlan = [];
let histRequestedFor = null; // gameID we already asked full history for (ask once)
let lastStateAt = Date.now();
let lastJoinAt = 0;

// Build mirror directly from a full server snapshot (no history needed).
// Server flags are authoritative; my own sent actions are applied on top
// via engine (only I can move on my turn, so nothing else changes).
function snapshot(F) {
  try {
    const normSeg = (arr) => (arr || []).map((g) => ({ from: pt(g.from), to: pt(g.to), invincible: !!g.invincible }));
    const s = E.createInitialState();
    s.boardSize = F.boardSize || 19;
    s.turn = F.turn; s.moveNumber = F.moveNumber;
    s.lineLimit = F.lineLimit || 120;
    s.actionsRemaining = F.actionsRemaining;
    s.revision = F.revision;
    s.nodes = { blue: (F.nodes.blue || []).map(pt), red: (F.nodes.red || []).map(pt) };
    s.segments = { blue: normSeg(F.segments.blue), red: normSeg(F.segments.red) };
    if (F.scores) s.scores = { blue: F.scores.blue, red: F.scores.red };
    const t = E.computeTerritories(s.segments);
    s.territories = t.territories; s.areas = t.areas;
    s.currentTurnEdges = [];
    return s;
  } catch (e) { say('SNAPSHOT-FAIL ::', e.message); return null; }
}

function onState(F) {
  serverState = F;
  serverRev = F.revision;
  lastStateAt = Date.now();
  if (F.blue === USER) myColor = 'blue'; else if (F.red === USER) myColor = 'red';
  // REAL server clocks (seconds, keyed by username) — authoritative for budgeting.
  if (F.clocks && typeof F.clocks[USER] === 'number') clockMs = F.clocks[USER] * 1000;
  if (Array.isArray(F.moveHistory) && F.moveHistory.length) {
    const m = rebuild(F.moveHistory);
    if (m) { mirror = m; }
  }
  // Prefer/refresh from full snapshot whenever nodes+segments are present
  // (game_state ships moveHistory:[] — history is NOT the sync path).
  if (F.nodes && F.segments) {
    const snap = snapshot(F);
    if (snap) mirror = snap;
  }
  if ((!Array.isArray(F.moveHistory) || !F.moveHistory.length) && !(F.nodes && F.segments)) {
    if (histRequestedFor !== F.gameID) {
      histRequestedFor = F.gameID;
      send({ action: 'request_game_history' }); // nothing to build from — ask once
    }
    return;
  }
  const over = F.phase === 'done';
  const nB = F.segments && F.segments.blue ? F.segments.blue.length : '?';
  const nR = F.segments && F.segments.red ? F.segments.red.length : '?';
  const mB = mirror.segments.blue.length, mR = mirror.segments.red.length;
  const syncOk = (nB === mB && nR === mR);
  say(`state game=${F.gameID} rev=${F.revision} turn=${F.turn} phase=${F.phase} ` +
    `blue=${F.blue} red=${F.red} me=${myColor} clock=${Math.round(clockMs / 1000)}s segs B:${mB}/${nB} R:${mR}/${nR}${syncOk ? '' : ' MIRROR-MISMATCH'} ` +
    `scores=${JSON.stringify(F.scores || mirror.scores)}${over ? ' GAME-OVER winner=' + F.winner : ''}`);
  if (!syncOk && Array.isArray(F.moveHistory) && F.moveHistory.length) {
    say('segment-count mismatch — resyncing'); send({ action: 'request_game_history' }); return;
  }
  if (over) {
    thinking = false; pendingPlan = []; clockMs = 60000;
    say(`GAME-OVER logged. winner=${F.winner} reason=${F.winReason} final=${JSON.stringify(F.scores)}`);
    if (LOBBY) { gameID = null; say('re-queueing lobby…'); send({ action: 'lobby_join' }); send({ action: 'lobby_quick_play' }); }
    return;
  }
  // If we just moved: turn advanced or actions decreased -> update clock, maybe send action 2.
  if (thinking && isOurTurn(F) && pendingPlan.length) {
    // Still our turn: verify next planned action is still legal on fresh mirror, else recompute.
    let nxt = pendingPlan[0];
    let ok = false;
    try { E.applyMove(mirror, nxt.from, nxt.to); ok = true; } catch (e) { ok = false; }
    if (!ok) {
      say('planned action 2 stale — recomputing');
      try {
        const p = T.bestTurn(mirror, { clockMs });
        pendingPlan = p && p.moves.length ? p.moves : [];
        nxt = pendingPlan[0];
        ok = !!nxt;
      } catch (e) { ok = false; }
    }
    if (ok && nxt) {
      pendingPlan = pendingPlan.slice(1);
      if (DRY_RUN) { say(`DRY-RUN would play action2 ${nxt.from}->${nxt.to}`); thinking = false; pendingPlan = []; return; }
      actionID++;
      send({ action: 'move', from: nxt.from, to: nxt.to, expectedRevision: serverRev, actionID });
      return; // keep thinking until turn flips
    }
    thinking = false; pendingPlan = [];
    return;
  }
  if (thinking && (!isOurTurn(F) || !pendingPlan.length)) {
    // Our turn ended (or single-action turn): increment clock, release.
    clockMs += 15000;
    thinking = false; pendingPlan = [];
  }
  // Site-updated flow: the server may auto-seat us (states arrive with
  // me=<color>) while join keeps 404ing. If it's OUR turn, TRY the move
  // directly — if the server accepts it, join is dead weight.
  if (!thinking && isOurTurn(F) && !pendingPlan.length && myColor) {
    if (mirror.turn !== myColor) { say('mirror/server turn mismatch — resyncing'); send({ action: 'request_game_history' }); return; }
    thinking = true;
    turnStart = Date.now();
    setImmediate(() => {
      let plan = null;
      try { plan = T.bestTurn(mirror, { clockMs }); } catch (e) { say('THINK-ERROR', e.message); }
      const dt = Date.now() - turnStart;
      if (!plan || !plan.moves.length) { say(`no legal move (dt=${dt}ms)`); thinking = false; return; }
      pendingPlan = plan.moves.slice(1);
      const m0 = plan.moves[0];
      clockMs -= dt;
      if (DRY_RUN) { say(`DRY-RUN would play ${m0.from}->${m0.to} (dt=${dt}ms, rev=${serverRev})`); thinking = false; pendingPlan = []; return; }
      actionID++;
      say(`playing ${m0.from}->${m0.to} (dt=${dt}ms, rev=${serverRev}, clock~${Math.round(clockMs / 1000)}s)`);
      send({ action: 'move', from: m0.from, to: m0.to, expectedRevision: serverRev, actionID });
    });
    return;
  }
  maybeThink();
}

function connect() {
  if (closed) return;
  say('connecting', WS_URL);
  ws = new WebSocket(WS_URL, {
    headers: {
      ...(sessionCookie ? { Cookie: sessionCookie } : {}),
      Origin: 'https://meaf.us',
      'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36',
    },
  });
  ws.on('open', () => {
    connected = true;
    ws.send(JSON.stringify({ action: 'syn', username: USER, passH1: PASS }));
    say('>> syn (masked)');
  });
  ws.on('message', (data) => {
    let F;
    try { F = JSON.parse(data); } catch (e) { return; }
    if (F.type === 'syn_ack') {
      say('<< syn_ack');
      if (LOBBY) { send({ action: 'lobby_join' }); send({ action: 'lobby_quick_play' }); }
      else if (DIRECT_GAME) { gameID = DIRECT_GAME; send({ action: 'join', gameID, ...(SPECTATE ? { spectate: true } : {}) }); }
      else send({ action: 'tournament_subscribe', tournamentID: TOURNAMENT });
    } else if (F.type === 'tournament_user_state') {
      const a = F.assignment || null;
      say(`<< tournament_user_state joined=${F.joined} assignment=${a ? a.gameID : 'none'}`);
      if (a && a.gameID && a.gameID !== gameID) {
        gameID = a.gameID; mirror = E.createInitialState(); serverRev = -1;
        clockMs = 60000; thinking = false; pendingPlan = []; joinRetries = 0;
        say('\x07============ ASSIGNMENT: game ' + a.gameID + ' status=' + (a.status || '?') + ' starts=' + (a.scheduledStart || 'now') + ' ============');
        send({ action: 'join', gameID });
      }
    } else if (F.type === 'tournament_state') {
      say('<< tournament_state', JSON.stringify(F).slice(0, 500));
    } else if (F.type === 'game_state' || F.type === 'game_history') {
      if (DIRECT_GAME || F.gameID === gameID) onState({ ...F });
    } else if (F.type === 'action_ack') {
      say(`<< action_ack ${F.actionID}`);
    } else if (F.type === 'error') {
      // "Game not found" right after join = site race (game not live yet).
      // Retry the join a few times instead of skipping the game (a missed
      // join = forfeit loss by abandonment).
      if (/not found/i.test(F.message || '') && gameID && joinRetries < 12) {
        joinRetries++;
        say(`<< join race for ${gameID} — retry ${joinRetries}/12 in 6s`);
        setTimeout(() => send({ action: 'join', gameID }), 6000);
        return;
      }
      if (/stale|Unknown action/i.test(F.message || '')) {
        say(`<< ERROR (transient, ignored): ${F.message}`);
        return;
      }
      say(`<< ERROR: ${F.message} (resyncing)`);
      thinking = false; // release; fresh state will retrigger think
      send({ action: 'request_game_history' });
    } else if (F.type === 'match_found') {
      say(`<< match_found ${F.gameID}`);
      gameID = F.gameID; mirror = E.createInitialState(); serverRev = -1;
      clockMs = 60000; thinking = false; pendingPlan = []; joinRetries = 0;
      send({ action: 'join', gameID: F.gameID });
    } else {
      say('<<', JSON.stringify(F).slice(0, 300));
    }
  });
  ws.on('close', (code) => {
    connected = false;
    if (code === 1008) {
      // Likely stale session cookie — re-login once, then reconnect.
      say('ws closed 1008 — refreshing session cookie');
      login().then(() => setTimeout(connect, 1000)).catch(() => setTimeout(connect, 5000));
      return;
    }
    say(`ws closed code=${code} — reconnecting`);
    setTimeout(connect, 2000);
  });
  ws.on('error', (e) => say('ws error:', e.message));
}

process.on('SIGINT', () => { say('shutting down'); closed = true; try { ws.close(); } catch (e) {} process.exit(0); });
// Stream-theft watchdog: if a live game goes quiet >30s, the updates are
// probably going to another same-account connection (browser tab). Say so LOUD.
setInterval(() => {
  if (closed) return;
  if (gameID && (!serverState || serverState.gameID !== gameID) && Date.now() - lastJoinAt > 30000 && lastJoinAt > 0) {
    say('\x07WATCHDOG: joined ' + gameID + ' 30s+ ago, no state — re-joining');
    send({ action: 'join', gameID });
  }
  if (gameID && serverState && serverState.phase !== 'done' && Date.now() - lastStateAt > 30000) {
    say('\x07WATCHDOG: no server state for 30s+ in live game ' + gameID + ' — CLOSE ALL SITE TABS, stream may be stolen by same-account browser session');
    lastStateAt = Date.now(); // re-arm (warn every 30s, don't spam)
  }
}, 10000);
say(`connector up. user=${USER} tournament=${TOURNAMENT} game=${DIRECT_GAME || '(via assignment)'} DRY_RUN=${DRY_RUN} BUDGET=${BUDGET}`);
function boot() {
  login().then(() => { connect(); })
    .catch((e) => { say('login failed, retry in 5s:', e.message); setTimeout(boot, 5000); });
}
boot();
