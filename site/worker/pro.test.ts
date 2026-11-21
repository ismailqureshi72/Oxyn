// Pro registry, CLI tokens, `oxyn login` and the monthly caps (pro.ts), against an in-memory D1
// with every migration applied (testing/d1.ts). `npm test` from site/.
import { test, beforeEach } from 'node:test';
import assert from 'node:assert/strict';
import worker from './index.ts';
import { memoryD1 } from './testing/d1.ts';
import { CAPS, billingMonth, createToken, resetIndexCache } from './pro.ts';
import { adminResetUsage, adminUsage } from './pro.ts';
import { sha256Hex, type Env } from './shared.ts';
import PRO_SOURCE from './pro-src.json' with { type: 'json' };

const ORIGIN = 'https://ui.test';
const ctx = { waitUntil: (_p: Promise<unknown>) => {} };

// ── Fixture registry ──────────────────────────────────────────────────────────────────────────
type Item = { slug: string; kind: string; tier: 'free' | 'pro'; installable: boolean; oxyn_deps?: Record<string, string[]> };
const items: Item[] = [
  { slug: 'free-btn', kind: 'component', tier: 'free', installable: true },
  { slug: 'parent-x', kind: 'screen', tier: 'pro', installable: true, oxyn_deps: { dioxus: ['dep-y', 'free-btn'], gpui: [] } },
  { slug: 'dep-y', kind: 'component', tier: 'pro', installable: true, oxyn_deps: { dioxus: ['dep-z'] } },
  { slug: 'dep-z', kind: 'component', tier: 'pro', installable: true },
  { slug: 'not-built', kind: 'screen', tier: 'pro', installable: false },
  { slug: 'blk-1', kind: 'block', tier: 'pro', installable: true },
  { slug: 'app-1', kind: 'app', tier: 'pro', installable: true },
];
for (let i = 0; i < 80; i++) items.push({ slug: `comp-${i}`, kind: 'component', tier: 'pro', installable: true });
for (let i = 0; i < 25; i++) items.push({ slug: `scr-${i}`, kind: 'screen', tier: 'pro', installable: true });
const assets = new Map<string, string>([['/r/index.json', JSON.stringify({ schema: 1, items })], ['/r/free-btn.json', '{"slug":"free-btn"}']]);
for (const it of items) if (it.tier === 'pro' && it.installable) assets.set(`/_pro/r/${it.slug}.json`, JSON.stringify({ slug: it.slug, tier: 'pro' }));

let env: Env & { DB: ReturnType<typeof memoryD1> };
beforeEach(() => {
  resetIndexCache();
  env = {
    DB: memoryD1(),
    UPLOADS: { put: async () => ({}), get: async () => null, delete: async () => {} },
    ASSETS: { fetch: async (r: Request) => { const b = assets.get(new URL(r.url).pathname); return b ? new Response(b, { status: 200 }) : new Response('nope', { status: 404 }); } },
  } as unknown as Env & { DB: ReturnType<typeof memoryD1> };
});

// ── Helpers ───────────────────────────────────────────────────────────────────────────────────
const now = () => new Date().toISOString();
function user(email: string, { verified = 1, linked = true } = {}) {
  const id = crypto.randomUUID();
  env.DB.raw.prepare('INSERT INTO users (id, email, email_verified, created_at) VALUES (?, ?, ?, ?)').run(id, email, verified, now());
  if (linked) env.DB.raw.prepare("INSERT INTO oauth_accounts (provider, provider_id, user_id, created_at) VALUES ('github', ?, ?, ?)").run(id, id, now());
  return { id, email, email_verified: verified };
}
function grant(email: string, plan: 'pro' | 'team', parent: number | null = null): number {
  const r = env.DB.raw.prepare("INSERT INTO grants (email, plan, seats, granted_by, created_at, parent_id) VALUES (?, ?, ?, 'test', ?, ?)").run(email, plan, plan === 'team' && !parent ? 4 : 1, now(), parent);
  return Number(r.lastInsertRowid);
}
async function tokenFor(u: { id: string }) { return (await createToken(env, u.id, 'test')).token; }
async function session(u: { id: string }) {
  const t = 'S'.repeat(48);
  env.DB.raw.prepare("INSERT OR REPLACE INTO sessions (token_hash, user_id, provider, created_at, expires_at) VALUES (?, ?, 'github', ?, ?)").run(await sha256Hex(t), u.id, now(), Math.floor(Date.now() / 1000) + 3600);
  return `oxyn_session=${t}`;
}
const call = (path: string, init: RequestInit & { token?: string } = {}) => {
  const headers = new Headers(init.headers);
  if (init.token) headers.set('authorization', `Bearer ${init.token}`);
  return worker.fetch(new Request(ORIGIN + path, { ...init, headers }), env, ctx);
};
const get = async (path: string, token?: string) => { const r = await call(path, { token }); return { status: r.status, body: await r.json() as any, headers: r.headers }; };

// ── Billing month ─────────────────────────────────────────────────────────────────────────────
test('billing month: calendar month (UTC) without an anchor', () => {
  assert.deepEqual(billingMonth(null, new Date('2026-11-21T10:00:00Z')), { start: '2026-11-01T00:00:00.000Z', end: '2026-12-01T00:00:00.000Z' });
});
test('billing month: monthly steps from the Polar period, also for yearly plans and short months', () => {
  assert.deepEqual(billingMonth('2026-01-31T08:00:00Z', new Date('2026-03-01T00:00:00Z')), { start: '2026-02-28T08:00:00.000Z', end: '2026-03-31T08:00:00.000Z' });
  assert.deepEqual(billingMonth('2026-11-05T12:00:00Z', new Date('2026-11-21T00:00:00Z')).end, '2026-12-05T12:00:00.000Z');
  // an anchor in the future (a period end) steps back
  assert.deepEqual(billingMonth('2027-05-05T12:00:00Z', new Date('2026-11-21T00:00:00Z')), { start: '2026-11-05T12:00:00.000Z', end: '2026-12-05T12:00:00.000Z' });
});

// ── Auth ──────────────────────────────────────────────────────────────────────────────────────
test('auth: a Pro item needs a valid, unrevoked token; free items and the private path', async () => {
  assert.equal((await get('/api/r/comp-1.json')).status, 401);
  assert.equal((await get('/api/r/comp-1.json', 'oxyn_' + '0'.repeat(48))).status, 401);
  assert.equal((await get('/api/r/comp-1.json', 'not-a-token')).status, 401);
  const free = await get('/api/r/free-btn.json');
  assert.equal(free.status, 200);
  assert.equal(free.body.slug, 'free-btn');
  // the private assets never answer an outside request, whatever the token
  const u = user('pro@example.com'); grant(u.email, 'pro');
  const t = await tokenFor(u);
  for (const p of ['/_pro/r/comp-1.json', '/_pro/', '/_pro']) assert.equal((await call(p, { token: t })).status, 404);
  assert.equal((await get('/api/r/comp-1.json', t)).status, 200);
  // revoke it from the account page
  const c = await session(u);
  const list = await (await call('/api/cli/tokens', { headers: { cookie: c } })).json() as any;
  assert.equal(list.tokens.length, 1);
  assert.equal('token' in list.tokens[0] || 'token_hash' in list.tokens[0], false);
  assert.equal((await call(`/api/cli/tokens/${list.tokens[0].id}`, { method: 'DELETE', headers: { cookie: c } })).status, 403, 'needs the site origin');
  assert.equal((await call(`/api/cli/tokens/${list.tokens[0].id}`, { method: 'DELETE', headers: { cookie: c, origin: ORIGIN } })).status, 200);
  assert.equal((await get('/api/r/comp-1.json', t)).status, 401);
});

test('auth: only the token hash is stored; tokens are made from the site only', async () => {
  const u = user('h@example.com');
  const c = await session(u);
  const r = await call('/api/cli/tokens', { method: 'POST', headers: { cookie: c, origin: ORIGIN, 'content-type': 'application/json' }, body: JSON.stringify({ name: 'laptop' }) });
  assert.equal(r.status, 201);
  const b = await r.json() as any;
  assert.match(b.token, /^oxyn_[0-9a-f]{48}$/);
  const row = env.DB.raw.prepare('SELECT * FROM cli_tokens').get() as any;
  assert.equal(row.token_hash, await sha256Hex(b.token));
  assert.equal(JSON.stringify(row).includes(b.token), false);
  assert.equal((await call('/api/cli/tokens', { method: 'POST', headers: { cookie: c, 'content-type': 'application/json' }, body: '{}' })).status, 403);
});

test('free plan: no Pro items, with the usage in the answer', async () => {
  const u = user('free@example.com');
  const r = await get('/api/r/comp-1.json', await tokenFor(u));
  assert.equal(r.status, 403);
  assert.equal(r.body.error.code, 'pro_required');
  assert.equal(r.body.usage.plan, 'free');
});

// ── Caps ──────────────────────────────────────────────────────────────────────────────────────
test('Pro: 20 components and 8 screens a month; blocks and apps count as screens; re-downloads are free', async () => {
  const u = user('p@example.com'); grant(u.email, 'pro');
  const t = await tokenFor(u);
  for (let i = 0; i < CAPS.pro.component; i++) {
    const r = await get(`/api/r/comp-${i}.json`, t);
    assert.equal(r.status, 200, `comp-${i}`);
    assert.equal(r.headers.get('x-oxyn-counted'), '1');
  }
  const over = await get('/api/r/comp-20.json', t);
  assert.equal(over.status, 403);
  assert.equal(over.body.error.code, 'quota_exceeded');
  assert.match(over.body.error.message, /20 of 20 Pro components.*resets on \d{4}-\d{2}-\d{2}/);
  assert.equal(over.body.usage.components.used, 20);
  // the same item again, any number of times: not counted, still served
  const again = await get('/api/r/comp-3.json', t);
  assert.equal(again.status, 200);
  assert.equal(again.headers.get('x-oxyn-counted'), '0');
  // screens are their own bucket
  assert.equal((await get('/api/r/blk-1.json', t)).status, 200);
  assert.equal((await get('/api/r/app-1.json', t)).status, 200);
  for (let i = 0; i < 6; i++) assert.equal((await get(`/api/r/scr-${i}.json`, t)).status, 200);
  const s9 = await get('/api/r/scr-6.json', t);
  assert.equal(s9.body.error.code, 'quota_exceeded');
  const who = await get('/api/cli/whoami', t);
  assert.equal(who.status, 200);
  assert.equal(who.body.usage.components.used, 20);
  assert.equal(who.body.usage.screens.used, 8);
  assert.match(who.body.usage.summary, /^20 of 20 components, 8 of 8 screens, resets \d{4}-\d{2}-\d{2}$/);
});

test('a dependency of an item already opened this month comes with it (not counted)', async () => {
  const u = user('d@example.com'); grant(u.email, 'pro');
  const t = await tokenFor(u);
  // ?for= an item not opened yet: counts normally
  assert.equal((await get('/api/r/dep-z.json?for=parent-x', t)).headers.get('x-oxyn-counted'), '1');
  assert.equal((await get('/api/r/parent-x.json', t)).headers.get('x-oxyn-counted'), '1');
  const dep = await get('/api/r/dep-y.json?for=parent-x', t);
  assert.equal(dep.status, 200);
  assert.equal(dep.headers.get('x-oxyn-counted'), '0');
  // not a dependency of the parent: counts
  assert.equal((await get('/api/r/comp-1.json?for=parent-x', t)).headers.get('x-oxyn-counted'), '1');
  const w = await get('/api/cli/whoami', t);
  assert.deepEqual([w.body.usage.components.used, w.body.usage.screens.used], [2, 1]);
  assert.equal((await get('/api/r/not-built.json', t)).status, 404);
});

test('admins are unlimited', async () => {
  const a = user('m.ismailqureshi62@gmail.com');
  const t = await tokenFor(a);
  for (let i = 0; i < 30; i++) assert.equal((await get(`/api/r/comp-${i}.json`, t)).status, 200);
  for (let i = 0; i < 12; i++) assert.equal((await get(`/api/r/scr-${i}.json`, t)).status, 200);
  const w = await get('/api/cli/whoami', t);
  assert.equal(w.body.admin, true);
  assert.equal(w.body.usage.unlimited, true);
  assert.equal(w.body.usage.components.limit, null);
  // an unverified owner email is not an admin (and not Pro)
  const fake = user('khubaibnazeer8@gmail.com', { verified: 0, linked: false });
  assert.equal((await get('/api/r/comp-1.json', await tokenFor(fake))).body.error.code, 'pro_required');
});

test('Team: one pool of 75 components and 22 screens shared by its people', async () => {
  const owner = user('owner@example.com');
  const g = grant(owner.email, 'team');
  const mate = user('mate@example.com'); grant(mate.email, 'team', g);
  const [to, tm] = [await tokenFor(owner), await tokenFor(mate)];
  for (let i = 0; i < 40; i++) assert.equal((await get(`/api/r/comp-${i}.json`, to)).status, 200);
  for (let i = 40; i < 75; i++) assert.equal((await get(`/api/r/comp-${i}.json`, tm)).status, 200);
  // the teammate re-downloading what the owner opened: shared, free
  assert.equal((await get('/api/r/comp-1.json', tm)).headers.get('x-oxyn-counted'), '0');
  const over = await get('/api/r/comp-75.json', to);
  assert.equal(over.body.error.code, 'quota_exceeded');
  assert.match(over.body.error.message, /75 of 75 .*shared by your Team/);
  const w = await get('/api/cli/whoami', tm);
  assert.equal(w.body.usage.pooled, true);
  assert.equal(w.body.usage.components.limit, CAPS.team.component);
  assert.equal(w.body.usage.screens.limit, CAPS.team.screen);
  // someone with their own Pro grant keeps their own pool
  const solo = user('solo@example.com'); grant(solo.email, 'pro');
  assert.equal((await get('/api/r/comp-75.json', await tokenFor(solo))).status, 200);
});

test('admin reset frees the month, and admin usage lists it', async () => {
  const u = user('r@example.com'); grant(u.email, 'pro');
  const t = await tokenFor(u);
  for (let i = 0; i < 20; i++) await get(`/api/r/comp-${i}.json`, t);
  assert.equal((await get('/api/r/comp-20.json', t)).status, 403);
  const before = await adminUsage(env, u);
  assert.equal(before.components.used, 20);
  assert.equal(before.items.length, 20);
  assert.deepEqual(await adminResetUsage(env, 'admin@example.com', u), { pool: `user:${u.id}`, cleared: 20 });
  assert.equal((await get('/api/r/comp-20.json', t)).status, 200);
  assert.equal((await get('/api/r/comp-1.json', t)).headers.get('x-oxyn-counted'), '1', 'opened again after a reset counts again');
});

test('the Polar period start sets the billing month', async () => {
  const u = user('polar@example.com');
  env.DB.raw.prepare("INSERT INTO subscriptions (id, user_id, customer_id, customer_email, product_id, plan, interval, status, current_period_start, current_period_end, created_at, updated_at) VALUES ('sub1', ?, 'c', ?, 'p', 'pro', 'year', 'active', '2026-03-09T10:00:00Z', '2027-03-09T10:00:00Z', ?, ?)").run(u.id, u.email, now(), now());
  env.DB.raw.prepare("INSERT INTO licences (id, subscription_id, user_id, email, plan, created_at) VALUES ('l1', 'sub1', ?, ?, 'pro', ?)").run(u.id, u.email, now());
  const w = await get('/api/cli/whoami', await tokenFor(u));
  assert.equal(w.body.tier, 'pro');
  assert.equal(w.body.usage.resets_at.slice(8, 19), '09T10:00:00');
});

// ── Site copy shares the count ────────────────────────────────────────────────────────────────
test('site: Pro code opens through the same count (POST), GET shows it only once opened', async () => {
  const slug = Object.keys(PRO_SOURCE as Record<string, unknown>)[0];
  assert.ok(slug, 'pro-src.json has items (run npm run gen first)');
  const u = user('site@example.com'); grant(u.email, 'pro');
  const c = await session(u);
  const g1 = await call(`/api/pro/source/${slug}`, { headers: { cookie: c } });
  assert.equal(g1.status, 403);
  assert.equal(((await g1.json()) as any).error.code, 'unlock_required');
  assert.equal((await call(`/api/pro/source/${slug}`, { method: 'POST', headers: { cookie: c } })).status, 403, 'needs the site origin');
  const p = await call(`/api/pro/source/${slug}`, { method: 'POST', headers: { cookie: c, origin: ORIGIN } });
  assert.equal(p.status, 200);
  const pb = await p.json() as any;
  assert.equal(typeof pb.code, 'string');
  assert.equal((await call(`/api/pro/source/${slug}`, { headers: { cookie: c } })).status, 200);
  const me = await (await call('/api/billing/me', { headers: { cookie: c } })).json() as any;
  assert.equal(me.usage.components.used + me.usage.screens.used, 1);
  // the CLI sees the same month
  const w = await get('/api/cli/whoami', await tokenFor(u));
  assert.equal(w.body.usage.components.used + w.body.usage.screens.used, 1);
});

// ── oxyn login ────────────────────────────────────────────────────────────────────────────────
test('oxyn login: start, approve in the signed-in browser, poll returns the token once', async () => {
  const u = user('login@example.com'); grant(u.email, 'pro');
  const s = await (await call('/api/cli/auth/start', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ client: 'my-laptop' }) })).json() as any;
  assert.match(s.user_code, /^[A-Z]{4}-[A-Z]{4}$/);
  assert.equal(s.verification_url, `${ORIGIN}/cli/auth/?code=${s.user_code}`);
  const poll = () => call('/api/cli/auth/poll', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ device_code: s.device_code }) });
  assert.equal(((await (await poll()).json()) as any).status, 'pending');
  const c = await session(u);
  const req = await (await call(`/api/cli/auth/request?code=${s.user_code}`, { headers: { cookie: c } })).json() as any;
  assert.equal(req.client, 'my-laptop');
  assert.equal((await call('/api/cli/auth/approve', { method: 'POST', headers: { cookie: c, 'content-type': 'application/json' }, body: JSON.stringify({ code: s.user_code }) })).status, 403);
  assert.equal((await call('/api/cli/auth/approve', { method: 'POST', headers: { cookie: c, origin: ORIGIN, 'content-type': 'application/json' }, body: JSON.stringify({ code: s.user_code }) })).status, 200);
  const done = await (await poll()).json() as any;
  assert.equal(done.status, 'approved');
  assert.equal(done.email, 'login@example.com');
  assert.equal((await poll()).status, 410, 'the token is handed out once');
  const w = await get('/api/cli/whoami', done.token);
  assert.equal(w.body.email, 'login@example.com');
  assert.equal(w.body.tier, 'pro');
  const tok = env.DB.raw.prepare('SELECT name FROM cli_tokens').get() as any;
  assert.equal(tok.name, 'oxyn login on my-laptop');
});

test('oxyn login: declined in the browser', async () => {
  const u = user('deny@example.com');
  const s = await (await call('/api/cli/auth/start', { method: 'POST', headers: { 'content-type': 'application/json' }, body: '{}' })).json() as any;
  const c = await session(u);
  assert.equal((await call('/api/cli/auth/approve', { method: 'POST', headers: { cookie: c, origin: ORIGIN, 'content-type': 'application/json' }, body: JSON.stringify({ code: s.user_code, approve: false }) })).status, 200);
  const r = await call('/api/cli/auth/poll', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ device_code: s.device_code }) });
  assert.equal(r.status, 403);
});
