// mined-fetch.js — polite read-only archive fetcher (NEW file, game-data miner).
// Auth via POST /sst1/api/access, then GET list + full games. Max ~1 req/2s.
const fs = require('fs');
const path = require('path');

const BASE = 'https://meaf.us/sst1';
const USER = process.env.ENC_USER;
const PASSH1 = process.env.ENC_PASS;
if (!USER || !PASSH1) { console.error('set ENC_USER and ENC_PASS'); process.exit(1); }
const DATA = path.join(__dirname, '..', 'mined-data');
fs.mkdirSync(DATA, { recursive: true });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let cookie = '';

async function req(url, opts = {}, timeoutMs = 10000) {
  const ctl = new AbortController();
  const t = setTimeout(() => ctl.abort(), timeoutMs);
  try {
    const r = await fetch(url, {
      ...opts,
      signal: ctl.signal,
      headers: { ...(opts.headers || {}), ...(cookie ? { Cookie: cookie } : {}) },
    });
    return r;
  } finally { clearTimeout(t); }
}

async function login() {
  const r = await req(BASE + '/api/access', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username: USER, passH1: PASSH1 }),
  });
  if (!r.ok) throw new Error('login HTTP ' + r.status);
  const sc = r.headers.get('set-cookie') || '';
  const m = sc.match(/sst1_access=[^;]+/);
  if (!m) throw new Error('no session cookie');
  cookie = m[0];
  console.log('login ok');
}

async function getJSON(url) {
  const r = await req(url);
  if (!r.ok) throw new Error(`GET ${url} -> HTTP ${r.status}`);
  return r.json();
}

async function main() {
  const maxPages = parseInt(process.argv[2] || '3', 10);
  const maxGames = parseInt(process.argv[3] || '50', 10);
  const minMoves = parseInt(process.argv[4] || '10', 10);
  await login();
  let listed = [];
  for (let p = 1; p <= maxPages; p++) {
    await sleep(2100);
    let d;
    try { d = await getJSON(`${BASE}/api/games?search=&page=${p}`); }
    catch (e) { console.log(`page ${p} ERROR: ${e.message}`); continue; }
    console.log(`page ${p}/${d.pages} total=${d.total} got=${d.games.length}`);
    fs.writeFileSync(path.join(DATA, `list-p${p}.json`), JSON.stringify(d, null, 1));
    listed = listed.concat(d.games);
    if (p === 1) console.log('winReasons p1:', JSON.stringify(countBy(d.games, (g) => g.winReason)));
  }
  // Prefer games with real play: higher moveNumber first, but keep a mix of winReasons.
  listed.sort((a, b) => b.moveNumber - a.moveNumber);
  const scored = listed.filter((g) => g.winReason === 'score');
  const others = listed.filter((g) => g.winReason !== 'score' && g.moveNumber >= minMoves);
  const want = [];
  for (const g of scored) { if (want.length < Math.min(20, maxGames)) want.push(g); }
  for (const g of others) { if (want.length < maxGames) want.push(g); }
  console.log(`fetching ${want.length} full games (${scored.length} score-games in batch)`);
  let ok = 0, fail = 0;
  for (const g of want) {
    const fp = path.join(DATA, `game-${g.gameID}.json`);
    if (fs.existsSync(fp)) { ok++; continue; }
    await sleep(2100);
    try {
      const d = await getJSON(`${BASE}/api/game?id=${g.gameID}`);
      fs.writeFileSync(fp, JSON.stringify(d));
      ok++;
      const mh = Array.isArray(d.moveHistory) ? d.moveHistory.length : -1;
      console.log(`ok ${g.gameID} ${g.winReason} mv=${g.moveNumber} mh=${mh} ${g.blue}-vs-${g.red} w=${g.winner}`);
    } catch (e) { fail++; console.log(`FAIL ${g.gameID}: ${e.message}`); }
    if (fail >= 5) { console.log('too many errors, stopping'); break; }
  }
  console.log(`done: ${ok} cached, ${fail} failed`);
}

function countBy(arr, f) {
  const o = {};
  for (const x of arr) { const k = f(x); o[k] = (o[k] || 0) + 1; }
  return o;
}

main().catch((e) => { console.error('FATAL', e.message); process.exit(1); });
