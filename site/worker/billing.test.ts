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
let orders: Record<string, any>[] = []; // orders as Polar has them (every customer)
let receiptStatus = 200;
const realFetch = globalThis.fetch;
const P = 'https://api.polar.sh/v1';
beforeEach(() => {
  calls = []; remote = new Map(); patchStatus = 200; orders = []; receiptStatus = 200;
  globalThis.fetch = (async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const url = String(input);
    const method = init.method || 'GET';
    const body = init.body ? JSON.parse(String(init.body)) : null;
    calls.push({ method, url, body });
    if (url.startsWith(P + '/orders/?')) {
      const q = new URL(url).searchParams;
      return Response.json({ items: orders.filter((o) => !q.get('customer_id') || o.customer_id === q.get('customer_id')) });
    }
    let om = url.match(/^https:\/\/api\.polar\.sh\/v1\/orders\/([^/]+)(\/receipt|\/invoice)?$/);
    if (om) {
      const o = orders.find((x) => x.id === om![1]);
      if (!o) return Response.json({ error: 'ResourceNotFound' }, { status: 404 });
      if (!om[2]) return Response.json(o);
      if (om[2] === '/receipt') return receiptStatus === 202 ? new Response(null, { status: 202 }) : Response.json({ url: `https://files.test/r/${o.id}.pdf` });
      if (method === 'POST') { if (!o.billing_name) return Response.json({ error: 'MissingInvoiceBillingDetails' }, { status: 422 }); o.is_invoice_generated = true; return Response.json({}, { status: 202 }); }
      return Response.json({ url: `https://files.test/i/${o.id}.pdf` });
    }
    const pm = url.match(/^https:\/\/api\.polar\.sh\/v1\/customers\/([^/]+)\/payment-methods$/);
    if (pm) return Response.json({ items: [{ id: 'pm1', type: 'card', customer_id: decodeURIComponent(pm[1]), method_metadata: { brand: 'visa', last4: '4242', exp_month: 12, exp_year: 2030, wallet: null } }] });
    if (url === P + '/customer-sessions/') return Response.json({ token: 't', customer_portal_url: `https://polar.sh/oxyn/portal?customer_session_token=t&c=${body.customer_id}` }, { status: 201 });
    const m = url.match(/^https:\/\/api\.polar\.sh\/v1\/subscriptions\/([^/]+)$/);
    if (!m) return new Response('{}', { status: 404 });
    const s = remote.get(decodeURIComponent(m[1]));
    if (!s) return new Response('{}', { status: 404 });
    if (method === 'GET') return Response.json(s);
    if (patchStatus !== 200) return new Response('{"detail":"nope"}', { status: patchStatus });
    if ('pending_update' in body && body.pending_update === null) s.pending_update = null;
    if ('product_id' in body) {
      if (body.proration_behavior === 'next_period') s.pending_update = { id: 'pu1', product_id: body.product_id, applies_at: s.current_period_end };
      else { s.product_id = body.product_id; s.amount = PLANS.find((p) => p.product === body.product_id)!.cents; }
    }
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

// ── In-site billing ─────────────────────────────────────────────────────────────────────────────
const TEAM_M = plan('team-month'), TEAM_Y = plan('team-year');
const get = async (path: string, cookie: string) => {
  const r = await worker.fetch(new Request(ORIGIN + path, { headers: { cookie } }), env, ctx);
  const t = await r.text();
  let body: any = null; try { body = JSON.parse(t); } catch { /* redirect */ }
  return { status: r.status, body, location: r.headers.get('location') };
};
const change = async (cookie: string, planKey: string, origin = ORIGIN) => {
  const r = await worker.fetch(new Request(ORIGIN + '/api/billing/change', { method: 'POST', headers: { cookie, origin, 'content-type': 'application/json' }, body: JSON.stringify({ plan: planKey }) }), env, ctx);
  return { status: r.status, body: (await r.json()) as any };
};

test('change rule: Pro -> Team on the same interval now (invoice), everything else at renewal (next_period)', async () => {
  const { changeRule, switchTargets, prorated } = await import('./billing.ts');
  assert.deepEqual(changeRule(PRO_M, TEAM_M), { when: 'now', proration: 'invoice' });
  assert.deepEqual(changeRule(FOUNDER_Y, TEAM_Y), { when: 'now', proration: 'invoice' });
  assert.deepEqual(changeRule(PRO_M, PRO_Y), { when: 'renewal', proration: 'next_period' });
  assert.deepEqual(changeRule(PRO_M, TEAM_Y), { when: 'renewal', proration: 'next_period' });
  assert.deepEqual(changeRule(TEAM_M, PRO_M), { when: 'renewal', proration: 'next_period' });
  assert.ok(switchTargets(FOUNDER_M).every((p) => !p.founder));
  assert.equal(switchTargets(PRO_M).length, 3);
  const start = new Date(0).toISOString(), end = new Date(30 * 86_400_000).toISOString();
  assert.equal(prorated(1999, TEAM_M, start, end, 15 * 86_400_000), Math.round((4499 - 1999) / 2));
});

test('billing details: live plan, founder renewal shown as the next charge, card, no founder options', async () => {
  const u = user('details@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b1', FOUNDER_M, u));
  const r = await get('/api/billing/subscription', await session(u));
  assert.equal(r.status, 200);
  const s = r.body.subscription;
  assert.equal(s.plan.key, 'pro-founder-month');
  assert.equal(s.next.cents, PRO_M.cents, 'the founder intro price ends at the renewal');
  assert.equal(s.next.founder_ends, true);
  assert.equal(s.pending.founder_renewal, true);
  assert.ok(s.options.length > 0 && s.options.every((o: any) => !o.founder));
  assert.equal(r.body.card.last4, '4242');
  for (const c of calls) assert.ok(!c.url.includes('organization_id'));
});

test('switch: monthly -> yearly is scheduled at renewal; keep drops it; founder renewal cannot be dropped', async () => {
  const u = user('switch1@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b2', PRO_M, u));
  const cookie = await session(u);
  const pv = await get('/api/billing/preview?plan=pro-year', cookie);
  assert.equal(pv.body.preview.when, 'renewal');
  assert.equal(pv.body.preview.charge_now_cents, 0);
  assert.equal(pv.body.preview.next_charge.cents, PRO_Y.cents);
  assert.equal((await change(cookie, 'pro-year', 'https://evil.test')).status, 403);
  calls = [];
  const r = await change(cookie, 'pro-year');
  assert.equal(r.status, 200);
  assert.deepEqual(patches().map((p) => p.body), [{ product_id: PRO_Y.product, proration_behavior: 'next_period' }]);
  assert.equal(remote.get('b2')!.product_id, PRO_M.product);
  assert.equal(r.body.subscription.pending.plan.key, 'pro-year');
  assert.equal(row('b2').plan, 'pro');
  calls = [];
  const k = await post('/api/billing/keep', cookie);
  assert.equal(k.status, 200);
  assert.deepEqual(patches().map((p) => p.body), [{ pending_update: null }]);
  assert.equal(remote.get('b2')!.pending_update, null);
  // Founder: its renewal as Pro stays.
  const f = user('switch2@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b3', FOUNDER_M, f));
  calls = [];
  await post('/api/billing/keep', await session(f));
  assert.equal(patches().length, 0);
  assert.equal(remote.get('b3')!.pending_update.product_id, PRO_M.product);
});

test('switch: Pro -> Team charges the prorated difference now; founder pending renewal is cleared first', async () => {
  const u = user('switch3@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b4', FOUNDER_M, u));
  const cookie = await session(u);
  const pv = await get('/api/billing/preview?plan=team-month', cookie);
  assert.equal(pv.body.preview.when, 'now');
  assert.ok(pv.body.preview.charge_now_cents > 0 && pv.body.preview.charge_now_cents <= TEAM_M.cents - FOUNDER_M.cents);
  calls = [];
  const r = await change(cookie, 'team-month');
  assert.equal(r.status, 200);
  assert.deepEqual(patches().map((p) => p.body), [{ pending_update: null }, { product_id: TEAM_M.product, proration_behavior: 'invoice' }]);
  assert.equal(row('b4').plan, 'team');
  assert.equal(row('b4').founder, 0);
  assert.equal((await get('/api/billing/me', cookie)).body.tier, 'team');
});

test('switch: founder plans, unknown plans, a cancelling plan, a teammate and a Polar refusal', async () => {
  const u = user('switch4@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b5', PRO_M, u));
  const cookie = await session(u);
  assert.equal((await change(cookie, 'pro-founder-year')).status, 400);
  assert.equal((await change(cookie, 'nope')).status, 400);
  assert.equal((await change(cookie, 'pro-month')).status, 409);
  // A teammate cannot switch the owner's Team.
  const owner = user('switch5@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b6', TEAM_M, owner));
  const mate = user('switch6@example.com');
  env.DB.raw.prepare('INSERT INTO team_seats (subscription_id, email, invited_by, created_at) VALUES (?, ?, ?, ?)').run('b6', mate.email, owner.id, now());
  const mc = await session(mate);
  const m = await change(mc, 'team-year');
  assert.equal(m.status, 404);
  assert.match(m.body.error.message, /Team owner/);
  assert.equal((await get('/api/billing/subscription', mc)).body.subscription, null);
  // Polar refuses: 502, D1 unchanged.
  patchStatus = 422;
  assert.equal((await change(cookie, 'team-month')).status, 502);
  assert.equal(row('b5').plan, 'pro');
  patchStatus = 200;
  // Cancelling: resume first.
  await post('/api/billing/cancel', cookie);
  const c = await change(cookie, 'team-month');
  assert.equal(c.status, 409);
  assert.equal(c.body.error.code, 'cancelling');
});

test('invoices: only the account\'s own orders; receipt and invoice downloads; another customer\'s order is 404', async () => {
  const u = user('inv@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b7', PRO_M, u));
  const other = user('inv2@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b8', PRO_M, other));
  const mk = (id: string, cust: string, extra = {}) => ({ id, customer_id: cust, created_at: now(), status: 'paid', total_amount: 1999, tax_amount: 0, currency: 'usd', description: 'Oxyn Pro Monthly', billing_reason: 'subscription_create', invoice_number: 'OXYN-1', is_invoice_generated: false, billing_name: null, ...extra });
  const A = '11111111-1111-4111-8111-111111111111', B = '22222222-2222-4222-8222-222222222222', C = '33333333-3333-4333-8333-333333333333';
  orders = [mk(A, 'cus_inv@example.com'), mk(B, 'cus_inv2@example.com'), mk(C, 'cus_inv@example.com', { billing_name: 'Ada', is_invoice_generated: false })];
  const cookie = await session(u);
  const l = await get('/api/billing/invoices', cookie);
  assert.deepEqual(l.body.items.map((o: any) => o.id).sort(), [A, C].sort());
  assert.ok(calls.some((c) => c.url.includes('customer_id=cus_inv%40example.com')));
  assert.ok(!calls.some((c) => c.url.includes('organization_id')));
  assert.equal((await get(`/api/billing/invoices/${A}/receipt`, cookie)).body.url, `https://files.test/r/${A}.pdf`);
  assert.equal((await get(`/api/billing/invoices/${B}/receipt`, cookie)).status, 404);
  assert.equal((await get(`/api/billing/invoices/${A}/invoice`, cookie)).status, 422, 'no billing address: use the receipt');
  assert.equal((await get(`/api/billing/invoices/${C}/invoice`, cookie)).status, 202, 'generation starts');
  assert.equal((await get(`/api/billing/invoices/${C}/invoice`, cookie)).body.url, `https://files.test/i/${C}.pdf`);
  receiptStatus = 202;
  assert.equal((await get(`/api/billing/invoices/${A}/receipt`, cookie)).status, 202);
});

test('card: a customer session on Polar\'s page for the owner; the old portal link lands on the account page', async () => {
  const u = user('card@example.com');
  await handleEvent(env, ctx, 'subscription.created', polarSub('b9', PRO_M, u));
  const r = await get('/api/billing/card', await session(u));
  assert.equal(r.status, 302);
  assert.match(r.location!, /^https:\/\/polar\.sh\/oxyn\/portal\?customer_session_token=t&c=cus_card@example\.com$/);
  const s = calls.find((c) => c.url.endsWith('/customer-sessions/'))!;
  assert.deepEqual(s.body, { customer_id: 'cus_card@example.com', return_url: `${ORIGIN}/account/#billing` });
  const p = await get('/api/billing/portal', await session(u));
  assert.equal(p.location, `${ORIGIN}/account/#billing`);
  const nobody = user('card2@example.com');
  assert.equal((await get('/api/billing/card', await session(nobody))).location, `${ORIGIN}/account/?billing=card_unavailable#billing`);
});

test('polar check: scopes by harmless probes, once a day', async () => {
  const { polarCheck } = await import('./billing.ts');
  const out = await polarCheck(env);
  assert.ok(out);
  assert.equal(out!.subscriptions_write, 'ok');
  assert.ok(patches().every((p) => p.url.endsWith('/00000000-0000-4000-8000-000000000000')));
  assert.equal(await polarCheck(env), null);
});
