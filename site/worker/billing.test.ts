// Founder intro price, first-time buyers, and cancel / resume (billing.ts), against an in-memory D1
// with every migration applied (testing/d1.ts) and a mocked Polar API. `npm test` from site/.
import { test, beforeEach, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import worker from './index.ts';
import { memoryD1 } from './testing/d1.ts';
import { PLANS, handleEvent, renewsAs, scheduleRegularRenewal } from './billing.ts';
import { sha256Hex, type Env } from './shared.ts';

const ORIGIN = 'https://ui.test';
const ctx = { waitUntil: (_p: Promise<unknown>) => {} };
const plan = (k: string) => PLANS.find((p) => p.key === k)!;
const FOUNDER_M = plan('pro-founder-month'), FOUNDER_Y = plan('pro-founder-year'), PRO_M = plan('pro-month'), PRO_Y = plan('pro-year');

// ── Polar mock ────────────────────────────────────────────────────────────────────────────────
type Call = { method: string; url: string; body: any };
let calls: Call[] = [];
let remote: Map<string, Record<string, any>>; // subscriptions as Polar has them
let patchStatus = 200;
const realFetch = globalThis.fetch;
beforeEach(() => {
  calls = []; remote = new Map(); patchStatus = 200;
  globalThis.fetch = (async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const url = String(input);
    const method = init.method || 'GET';
    const body = init.body ? JSON.parse(String(init.body)) : null;
    calls.push({ method, url, body });
    const m = url.match(/^https:\/\/api\.polar\.sh\/v1\/subscriptions\/([^/]+)$/);
    if (!m) return new Response('{}', { status: 404 });
    const s = remote.get(decodeURIComponent(m[1]));
    if (!s) return new Response('{}', { status: 404 });
    if (method === 'GET') return Response.json(s);
    if (patchStatus !== 200) return new Response('{"detail":"nope"}', { status: patchStatus });
    if ('product_id' in body) s.pending_update = { id: 'pu1', product_id: body.product_id, applies_at: s.current_period_end };
    if ('cancel_at_period_end' in body) s.cancel_at_period_end = body.cancel_at_period_end;
    s.modified_at = new Date().toISOString();
    return Response.json(s);
  }) as typeof fetch;
});
afterEach(() => { globalThis.fetch = realFetch; });

let env: Env & { DB: ReturnType<typeof memoryD1> };
beforeEach(() => {
  env = {
    DB: memoryD1(),
    POLAR_ACCESS_TOKEN: 'polar_test',
    UPLOADS: { put: async () => ({}), get: async () => null, delete: async () => {} },
    ASSETS: { fetch: async () => new Response('nope', { status: 404 }) },
  } as unknown as Env & { DB: ReturnType<typeof memoryD1> };
});

const now = () => new Date().toISOString();
const later = (days: number) => new Date(Date.now() + days * 86_400_000).toISOString();
function user(email: string) {
  const id = crypto.randomUUID();
  env.DB.raw.prepare('INSERT INTO users (id, email, email_verified, created_at) VALUES (?, ?, 1, ?)').run(id, email, now());
  env.DB.raw.prepare("INSERT INTO oauth_accounts (provider, provider_id, user_id, created_at) VALUES ('github', ?, ?, ?)").run(id, id, now());
  return { id, email };
}
async function session(u: { id: string }) {
  const t = crypto.randomUUID().replace(/-/g, '') + 'x'.repeat(16);
  env.DB.raw.prepare("INSERT INTO sessions (token_hash, user_id, provider, created_at, expires_at) VALUES (?, ?, 'github', ?, ?)").run(await sha256Hex(t), u.id, now(), Math.floor(Date.now() / 1000) + 3600);
  return `oxyn_session=${t}`;
}
/** A subscription as Polar sends it (webhook data, and what GET returns). */
function polarSub(id: string, p: typeof PRO_M, u: { id: string; email: string }, extra: Record<string, any> = {}) {
  const s = {
    id, product_id: p.product, status: 'active', amount: p.cents, currency: 'usd', cancel_at_period_end: false,
    current_period_start: now(), current_period_end: later(30), started_at: now(), ended_at: null, created_at: now(), modified_at: now(),
    customer_id: 'cus_' + u.email, customer: { id: 'cus_' + u.email, email: u.email }, metadata: { reference_id: u.id }, pending_update: null, ...extra,
  };
  remote.set(id, structuredClone(s));
  return s;
}
const row = (id: string) => env.DB.raw.prepare('SELECT * FROM subscriptions WHERE id = ?').get(id) as any;
const claims = () => (env.DB.raw.prepare('SELECT subscription_id FROM founder_claims').all() as any[]).map((r) => r.subscription_id);
const patches = () => calls.filter((c) => c.method === 'PATCH');
const post = async (path: string, cookie: string, origin = ORIGIN) => {
  const r = await worker.fetch(new Request(ORIGIN + path, { method: 'POST', headers: { cookie, origin, 'content-type': 'application/json' }, body: '{}' }), env, ctx);
  return { status: r.status, body: (await r.json()) as any };
};

// ── Founder rate = intro price ─────────────────────────────────────────────────────────────────
test('founder plans renew as the regular Pro product of the same interval', () => {
  assert.equal(renewsAs(FOUNDER_M)?.product, '677990ee-37fd-4144-a7b4-fc0072db0236');
  assert.equal(renewsAs(FOUNDER_Y)?.product, '6c6f941c-eb68-42a1-9588-6999793d7723');
  assert.equal(renewsAs(PRO_M), undefined);
});

test('a new founder subscription takes a spot and is scheduled to renew as Pro (next_period, no org id)', async () => {
  const u = user('new@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('s1', FOUNDER_M, u));
  assert.deepEqual(claims(), ['s1']);
  assert.equal(row('s1').founder, 1);
  const p = patches();
  assert.equal(p.length, 1);
  assert.equal(p[0].url, 'https://api.polar.sh/v1/subscriptions/s1');
  assert.deepEqual(p[0].body, { product_id: PRO_M.product, proration_behavior: 'next_period' });
  assert.ok(!('organization_id' in p[0].body));
  const st = env.DB.raw.prepare("SELECT value FROM settings WHERE key = 'founder_renewal:s1'").get() as any;
  assert.equal(st.value, 'scheduled');
});

test('idempotent: a webhook that already shows the pending change makes no Polar call', async () => {
  const u = user('idem@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('s2', FOUNDER_Y, u));
  assert.deepEqual(patches()[0].body.product_id, PRO_Y.product);
  calls = [];
  // The PATCH fires subscription.updated carrying the pending update.
  await handleEvent(env, ctx, 'subscription.updated', { ...remote.get('s2')!, modified_at: later(0.001) });
  assert.equal(calls.length, 0);
  // A stale payload without it: one GET, still no second PATCH.
  await handleEvent(env, ctx, 'subscription.active', { ...remote.get('s2')!, pending_update: null, modified_at: later(0.002) });
  assert.equal(patches().length, 0);
});

test('the friend case: Polar already has the pending change, nothing is sent', async () => {
  const u = user('friend@example.com');
  const s = polarSub('s3', FOUNDER_M, u, { pending_update: { id: '85b10903', product_id: PRO_M.product } });
  await handleEvent(env, ctx, 'subscription.updated', s);
  assert.equal(calls.length, 0);
  assert.deepEqual(claims(), ['s3']);
});

test('at renewal the row flips to Pro: plan stays pro, founder 0, the spot stays counted', async () => {
  const u = user('renew@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('s4', FOUNDER_M, u));
  calls = [];
  await handleEvent(env, ctx, 'subscription.updated', polarSub('s4', PRO_M, u, { modified_at: later(31), current_period_end: later(61) }));
  const r = row('s4');
  assert.equal(r.product_id, PRO_M.product);
  assert.equal(r.plan, 'pro');
  assert.equal(r.founder, 0);
  assert.equal(r.amount, 1999);
  assert.deepEqual(claims(), ['s4']);
  assert.equal(calls.length, 0);
  const me = await worker.fetch(new Request(ORIGIN + '/api/billing/me', { headers: { cookie: await session(u) } }), env, ctx);
  const b = (await me.json()) as any;
  assert.equal(b.tier, 'pro');
  assert.equal(b.subscription.founder, false);
  assert.equal(b.subscription.renews_at_amount, null);
});

test('a canceling founder subscription waits; resuming schedules it', async () => {
  const u = user('cancel1@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('s5', FOUNDER_M, u, { cancel_at_period_end: true }));
  assert.equal(patches().length, 0);
  assert.match((env.DB.raw.prepare("SELECT value FROM settings WHERE key = 'founder_renewal:s5'").get() as any).value, /^waiting/);
  remote.get('s5')!.cancel_at_period_end = false;
  await handleEvent(env, ctx, 'subscription.uncanceled', { ...remote.get('s5')!, modified_at: later(0.01) });
  assert.equal(patches().length, 1);
});

test('without the token the renewal is pending, then the admin retry schedules it', async () => {
  const u = user('notoken@example.com');
  const token = env.POLAR_ACCESS_TOKEN;
  env.POLAR_ACCESS_TOKEN = undefined;
  await handleEvent(env, ctx, 'subscription.created', polarSub('s6', FOUNDER_M, u));
  assert.equal(calls.length, 0);
  env.POLAR_ACCESS_TOKEN = token;
  patchStatus = 500;
  assert.match(await scheduleRegularRenewal(env, { id: 's6' }, FOUNDER_M), /^error: 500/);
  patchStatus = 200;
  assert.equal(await scheduleRegularRenewal(env, { id: 's6' }, FOUNDER_M), 'scheduled');
});

// ── First-time buyers only ──────────────────────────────────────────────────────────────────────
test('checkout: a returning customer is sent to the regular price, a new one to Polar', async () => {
  const fresh = user('fresh@example.com');
  let r = await worker.fetch(new Request(ORIGIN + '/api/billing/checkout?plan=pro-founder-month', { headers: { cookie: await session(fresh) } }), env, ctx);
  assert.equal(r.status, 302);
  assert.match(r.headers.get('location')!, /^https:\/\/buy\.polar\.sh\//);
  const back = user('back@example.com');
  const old = polarSub('old1', PRO_M, back, { status: 'canceled', ended_at: now() });
  await handleEvent(env, ctx, 'subscription.revoked', old);
  r = await worker.fetch(new Request(ORIGIN + '/api/billing/checkout?plan=pro-founder-year&from=x-y', { headers: { cookie: await session(back) } }), env, ctx);
  assert.equal(r.headers.get('location'), `${ORIGIN}/pricing/?founder=used&interval=year&from=x-y`);
  // Regular Pro is still fine for them.
  r = await worker.fetch(new Request(ORIGIN + '/api/billing/checkout?plan=pro-year', { headers: { cookie: await session(back) } }), env, ctx);
  assert.match(r.headers.get('location')!, /^https:\/\/buy\.polar\.sh\//);
});

test('webhook: a founder subscription from a returning customer takes no spot but still renews as Pro', async () => {
  const u = user('again@example.com');
  await handleEvent(env, ctx, 'subscription.revoked', polarSub('old2', PRO_M, u, { status: 'canceled', ended_at: now() }));
  await handleEvent(env, ctx, 'subscription.created', polarSub('s7', FOUNDER_M, u));
  assert.deepEqual(claims(), []);
  assert.equal(patches().length, 1);
});

// ── Cancel / resume ───────────────────────────────────────────────────────────────────────────
test('cancel and resume: owner only, same origin, D1 follows Polar', async () => {
  const u = user('owner@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('c1', PRO_M, u));
  const cookie = await session(u);
  assert.equal((await post('/api/billing/cancel', cookie, 'https://evil.test')).status, 403);
  calls = [];
  let r = await post('/api/billing/cancel', cookie);
  assert.equal(r.status, 200);
  assert.equal(r.body.subscription.cancel_at_period_end, true);
  assert.equal(r.body.subscription.active, true, 'Pro stays open until the period end');
  assert.deepEqual(patches()[0].body, { cancel_at_period_end: true });
  assert.equal(row('c1').cancel_at_period_end, 1);
  // Already cancelling: no second call.
  calls = [];
  assert.equal((await post('/api/billing/cancel', cookie)).status, 200);
  assert.equal(calls.length, 0);
  r = await post('/api/billing/resume', cookie);
  assert.equal(r.body.subscription.cancel_at_period_end, false);
  assert.deepEqual(patches()[0].body, { cancel_at_period_end: false });
  assert.equal(row('c1').cancel_at_period_end, 0);
});

test('cancel: a Team member, an account without a subscription and a Polar failure', async () => {
  const owner = user('teamowner@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('t1', plan('team-month'), owner));
  const mate = user('mate@example.com');
  env.DB.raw.prepare('INSERT INTO team_seats (subscription_id, email, invited_by, created_at) VALUES (?, ?, ?, ?)').run('t1', mate.email, owner.id, now());
  let r = await post('/api/billing/cancel', await session(mate));
  assert.equal(r.status, 404);
  assert.match(r.body.error.message, /Team owner/);
  assert.equal(patches().length, 0);
  const nobody = user('nobody@example.com');
  assert.equal((await post('/api/billing/cancel', await session(nobody))).status, 404);
  patchStatus = 422;
  r = await post('/api/billing/cancel', await session(owner));
  assert.equal(r.status, 502);
  assert.equal(row('t1').cancel_at_period_end, 0);
  patchStatus = 200;
  assert.equal((await post('/api/billing/cancel', await session(owner))).body.subscription.cancel_at_period_end, true);
});
