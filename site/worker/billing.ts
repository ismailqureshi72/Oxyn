// Oxyn Pro and Team, sold through Polar (merchant of record: Polar charges the card, adds the tax
// and sends the receipts). See docs/pricing.md.
//
//   GET    /api/billing/plans            public: plans, prices, founder spots left (cached 60 s)
//   GET    /api/billing/checkout?plan=   signed in: redirect to the plan's Polar checkout link with the
//                                        account's email and id (reference_id) filled in
//   GET    /api/billing/me               signed in: plan, status, renewal, Team seats
//   GET    /api/billing/portal           signed in: Polar customer portal (manage, cancel, invoices)
//   POST   /api/billing/seats            Team owner: {email} takes one of the 3 teammate seats (4 people with the owner)
//   DELETE /api/billing/seats/<email>    Team owner: frees a seat
//   GET    /api/pro/source/<slug>        Pro: the usage code of a Pro item this account's pool already
//                                        opened this month (else 403 unlock_required + usage)
//   POST   /api/pro/source/<slug>        Pro, from the site: opens it (one of the month's caps, pro.ts)
//   POST   /api/polar/webhook            Polar events, signed (Standard Webhooks)
//
// The Worker never needs the Polar API for a sale: checkout links are public URLs, and the signed
// webhook writes subscriptions, licences and the founder counter to D1. Two extras use the Polar API
// when the Worker secret POLAR_ACCESS_TOKEN exists, and degrade without it: a pre-signed customer
// portal session (else Polar's portal asks for the email), and archiving the founder products once
// 100 founder subscriptions exist (else the admin panel shows it as pending; the site already stops
// offering the founder rate either way).

import { EMAIL_RE, HttpError, fail, json, sameOrigin, type Ctx, type Env } from './shared';
import { currentUser } from './auth';
import PRO_SOURCE from './pro-src.json';
import { isAdminRow } from './entitle';
import { QuotaError, bucketOf, failQuota, quotaFor, registryIndex, unlock, usageFor } from './pro';
export { OWNERS, isAdminRow } from './entitle';

export const POLAR = {
  api: 'https://api.polar.sh/v1',
  portal: 'https://polar.sh/oxyn/portal',
  founderCap: 100,
  people: 4, // a Team is up to 4 people in total: the owner plus 3 teammates
  seats: 3, // teammate emails the owner can add (people - 1)
  tolerance: 300, // webhook timestamp, seconds
  maxBody: 1_000_000,
};

export type PlanKey = 'pro-month' | 'pro-year' | 'pro-founder-month' | 'pro-founder-year' | 'team-month' | 'team-year';
export interface Plan { key: PlanKey; plan: 'pro' | 'team'; interval: 'month' | 'year'; founder: boolean; cents: number; product: string; link: string }

/** The six Polar products (production, organization "oxyn") and their checkout links. */
export const PLANS: Plan[] = [
  { key: 'pro-month', plan: 'pro', interval: 'month', founder: false, cents: 1999, product: '677990ee-37fd-4144-a7b4-fc0072db0236', link: 'https://buy.polar.sh/polar_cl_nOTwWhBDBia5V47drawKkcEPM1g1ClF73jekN2cDk0k' },
  { key: 'pro-year', plan: 'pro', interval: 'year', founder: false, cents: 19999, product: '6c6f941c-eb68-42a1-9588-6999793d7723', link: 'https://buy.polar.sh/polar_cl_TIpeARSu8pHt9MTbFwisPzixrXIdJhlEBJOHG2B2fYk' },
  { key: 'pro-founder-month', plan: 'pro', interval: 'month', founder: true, cents: 1299, product: 'dd6c77e0-537e-4575-8272-179818c4636c', link: 'https://buy.polar.sh/polar_cl_1DDghPtFc5Mtf4xW8kEUbXO68TclXcBADP5xN1UD4x6' },
  { key: 'pro-founder-year', plan: 'pro', interval: 'year', founder: true, cents: 12445, product: 'b2c7ff7d-5ed5-4677-b4c2-023860cf6440', link: 'https://buy.polar.sh/polar_cl_coirJZKmfkFV75qqxxVDvjQO9bmbnqmAnQd9T2Xrezv' },
  { key: 'team-month', plan: 'team', interval: 'month', founder: false, cents: 4499, product: '47ac4829-9dea-4c5f-a47c-948eb3005865', link: 'https://buy.polar.sh/polar_cl_NCvpX9ojhDrafHw1bLtki9PTVUOePRTSPrUgL2ipK4Y' },
  { key: 'team-year', plan: 'team', interval: 'year', founder: false, cents: 44900, product: 'c9ad78fa-8fd7-4f81-a19a-af4c6472f000', link: 'https://buy.polar.sh/polar_cl_tGlTFd4OLxE5lYiu8NWR2eDjPRkrJGfUUcEwd1eo3vC' },
];
const byProduct = new Map(PLANS.map((p) => [p.product, p]));
const byKey = new Map(PLANS.map((p) => [p.key, p]));

/** Polar statuses that keep Pro open. past_due: Polar is retrying the card; it revokes on failure. */
export const ACTIVE = ['active', 'trialing', 'past_due'];
const ACTIVE_SQL = ACTIVE.map((s) => `'${s}'`).join(', ');

const iso = () => new Date().toISOString();
const SEC = { 'cache-control': 'no-store', 'referrer-policy': 'no-referrer' };
const redirect = (to: string, extra: Record<string, string> = {}) => new Response(null, { status: 302, headers: { location: to, ...SEC, ...extra } });
const isPrefetch = (req: Request) => /prefetch|prerender/i.test(`${req.headers.get('sec-purpose') || ''} ${req.headers.get('purpose') || ''}`);

// ── Entitlement ──────────────────────────────────────────────────────────────────────────────
interface SubRow {
  id: string; user_id: string | null; customer_id: string; customer_email: string; product_id: string; plan: 'pro' | 'team';
  interval: 'month' | 'year'; founder: number; status: string; amount: number; currency: string; cancel_at_period_end: number;
  current_period_start?: string | null; current_period_end: string | null; started_at: string | null; ended_at: string | null; created_at: string;
}
export interface Who { id: string; email: string; email_verified: number }
const live = (s: SubRow) => ACTIVE.includes(s.status) && !s.ended_at;

interface GrantRow { id: number; parent_id: number | null; email: string; plan: 'pro' | 'team'; seats: number; expires_at: string | null; reason: string | null; created_at: string }
export type Tier = 'free' | 'pro' | 'team';
export interface PlanFor {
  tier: Tier;
  pro: boolean;
  /** A maintainer: everything open, no upgrade prompts (see OWNERS and the D1 `admins` list). */
  admin: boolean;
  via: 'subscription' | 'seat' | 'grant' | 'admin' | null;
  mine: SubRow | null; // the account's own live subscription (Pro, or Team owner)
  latest: SubRow | null; // its newest subscription, live or not
  seat: (SubRow & { owner_email: string }) | null; // a live Team it holds a seat on
  grant: GrantRow | null; // a live manual grant
  customer: string | null; // Polar customer id, for the portal
}
const rank = (t: Tier) => (t === 'team' ? 2 : t === 'pro' ? 1 : 0);


/** The one licence check: what this account may use right now. Reads, in one place,
 *  1. its own Polar subscriptions (licences: Pro for one developer, or the Team owner),
 *  2. a seat on someone's live Team subscription,
 *  3. a manual grant (D1 `grants`, written by the admin panel from G2): unrevoked and not expired
 *     counts exactly like an active subscription.
 *  An email only counts when Google or GitHub verified it, so nobody can claim a licence, a seat or
 *  a grant by signing up with an address that is not theirs. The highest tier wins.
 *  4. Admins (OWNERS and the D1 `admins` list, provider-verified) have all access: tier `team`,
 *     `pro` true, `admin` true, whatever they pay for. Every Pro check (the Pro source endpoint and
 *     the Pro registry of P3) goes through this function, so admins are entitled everywhere. */
export async function plan_for(env: Env, u: Who): Promise<PlanFor> {
  const v = u.email_verified ? 1 : 0;
  const now = iso();
  const [ownR, seatR, grantR, adminR] = await env.DB.batch([
    env.DB.prepare(
      'SELECT s.* FROM licences l JOIN subscriptions s ON s.id = l.subscription_id ' +
        'WHERE l.user_id = ?1 OR (?3 = 1 AND l.email = ?2) ORDER BY s.created_at DESC LIMIT 20',
    ).bind(u.id, u.email, v),
    env.DB.prepare(
      `SELECT s.*, (SELECT email FROM licences WHERE subscription_id = s.id) AS owner_email FROM team_seats t JOIN subscriptions s ON s.id = t.subscription_id ` +
        `WHERE ?2 = 1 AND t.email = ?1 AND t.removed_at IS NULL AND s.plan = 'team' AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL ORDER BY s.created_at DESC LIMIT 1`,
    ).bind(u.email, v),
    env.DB.prepare(
      "SELECT id, parent_id, email, plan, seats, expires_at, reason, created_at FROM grants WHERE ?2 = 1 AND email = ?1 AND revoked_at IS NULL " +
        "AND (expires_at IS NULL OR expires_at > ?3) ORDER BY (plan = 'team') DESC, created_at DESC LIMIT 1",
    ).bind(u.email, v, now),
    env.DB.prepare(
      'SELECT (SELECT 1 FROM admins WHERE email = ?1 AND removed_at IS NULL) AS listed, (SELECT COUNT(*) FROM oauth_accounts WHERE user_id = ?2) AS linked',
    ).bind(u.email, u.id),
  ]);
  const adminRow = ((adminR.results ?? [])[0] as { listed: number | null; linked: number } | undefined) ?? { listed: null, linked: 0 };
  const admin = isAdminRow(u, !!adminRow.listed, Number(adminRow.linked ?? 0));
  const own = (ownR.results ?? []) as SubRow[];
  const seat = ((seatR.results ?? [])[0] as (SubRow & { owner_email: string }) | undefined) ?? null;
  const grant = ((grantR.results ?? [])[0] as GrantRow | undefined) ?? null;
  const mine = own.find(live) ?? null;
  const options: [Tier, PlanFor['via']][] = [];
  if (mine) options.push([mine.plan, 'subscription']);
  if (seat) options.push(['team', 'seat']);
  if (grant) options.push([grant.plan, 'grant']);
  options.sort((a, b) => rank(b[0]) - rank(a[0]));
  const [tier, via] = admin ? (['team', 'admin'] as [Tier, PlanFor['via']]) : (options[0] ?? ['free', null]);
  return { tier, pro: admin || tier !== 'free', admin, via, mine, latest: mine ?? own[0] ?? null, seat, grant, customer: own[0]?.customer_id ?? null };
}

const pub = (s: SubRow | null) => s && {
  plan: s.plan, interval: s.interval, founder: !!s.founder, status: s.status, active: live(s), amount: s.amount, currency: s.currency,
  cancel_at_period_end: !!s.cancel_at_period_end, current_period_end: s.current_period_end, started_at: s.started_at, ended_at: s.ended_at,
};

async function founderState(env: Env) {
  const r = await env.DB.prepare('SELECT COUNT(*) AS n FROM founder_claims').first<{ n: number }>();
  const claimed = Number(r?.n ?? 0);
  return { cap: POLAR.founderCap, claimed, remaining: Math.max(0, POLAR.founderCap - claimed), open: claimed < POLAR.founderCap };
}

// ── Routes ───────────────────────────────────────────────────────────────────────────────────
export const isBillingPath = (p: string) => p.startsWith('/api/billing/') || p.startsWith('/api/pro/') || p.replace(/\/+$/, '') === '/api/polar/webhook';

export async function handleBilling(req: Request, env: Env, ctx: Ctx, url: URL): Promise<Response> {
  const route = url.pathname.replace(/\/+$/, '');
  const m = req.method;
  try {
    if (route === '/api/polar/webhook') {
      if (m !== 'POST') throw new HttpError(405, 'method_not_allowed', 'Use POST.');
      return await webhook(req, env, ctx);
    }
    if (route === '/api/billing/plans' && (m === 'GET' || m === 'HEAD')) {
      const founder = await founderState(env);
      return json({ ok: true, founder, plans: PLANS.filter((p) => founder.open || !p.founder).map(({ key, plan, interval, founder: f, cents }) => ({ key, plan, interval, founder: f, cents })) },
        200, { 'cache-control': 'public, max-age=60' });
    }
    if (route === '/api/billing/checkout' && (m === 'GET' || m === 'HEAD')) {
      if (isPrefetch(req)) return new Response(null, { status: 204, headers: SEC });
      return await checkout(req, env, url);
    }
    const u = await currentUser(req, env);
    if (route === '/api/billing/portal' && m === 'GET') {
      if (!u) return redirect(`${url.origin}/login/?next=${encodeURIComponent('/account/')}`);
      return await portal(env, url, u);
    }
    if (!u) throw new HttpError(401, 'signed_out', 'Sign in to Oxyn first.');
    if (route === '/api/billing/me' && m === 'GET') return await billingMe(env, u);
    const src = route.match(/^\/api\/pro\/source\/([a-z0-9-]{1,80})$/);
    if (src && (m === 'GET' || m === 'POST')) return await proSource(req, env, url, u, src[1]);
    if (route === '/api/billing/seats' && m === 'POST') return await addSeat(req, env, url, u);
    const sm = route.match(/^\/api\/billing\/seats\/([^/]+)$/);
    if (sm && m === 'DELETE') return await removeSeat(req, env, url, u, sm[1]);
    throw new HttpError(404, 'not_found', 'No such API route.');
  } catch (err) {
    if (err instanceof QuotaError) return failQuota(err);
    if (err instanceof HttpError) return fail(err);
    console.error('billing error', err);
    return fail(new HttpError(500, 'internal', 'Something went wrong on our side. Please try again.'));
  }
}

/** The site's Pro code: shown once the account's pool opened the item this month (an admin's
 *  always); opening it (POST, the Unlock / Copy buttons) counts like a CLI download, see pro.ts. */
async function proSource(req: Request, env: Env, url: URL, u: Who, slug: string) {
  const item = (PRO_SOURCE as Record<string, unknown>)[slug];
  if (!item) throw new HttpError(404, 'not_found', 'There is no Pro item with that name.');
  const e = await plan_for(env, u);
  let usage;
  if (req.method === 'POST') {
    if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
    let kind: string | undefined;
    try { kind = (await registryIndex(env, url.origin)).get(slug)?.kind; } catch { kind = undefined; }
    ({ usage } = await unlock(env, u, slug, bucketOf(kind ?? 'screen'), 'site', e));
  } else {
    usage = await usageFor(env, u, e);
    if (!e.pro) throw new QuotaError('pro_required', 'This code is part of Oxyn Pro.', usage);
    const q = quotaFor(e, u.id);
    const open = q.unlimited || !!(await env.DB.prepare('SELECT 1 AS one FROM pro_unlocks WHERE pool = ?1 AND period_start = ?2 AND slug = ?3 AND reset_at IS NULL').bind(q.pool, q.period.start, slug).first());
    if (!open) throw new QuotaError('unlock_required', 'Open this item to see its code. It counts toward this month\'s Pro items.', usage);
  }
  return json({ ok: true, slug, ...(item as object), usage });
}

/** Sends a signed-in buyer to Polar with their email and account id prefilled. */
async function checkout(req: Request, env: Env, url: URL) {
  const plan = byKey.get((url.searchParams.get('plan') || '') as PlanKey);
  if (!plan) return redirect(`${url.origin}/pricing/`);
  // ?from=<slug>: the Pro item the buyer came from, so pricing can name it and lead back to it.
  const fromRaw = url.searchParams.get('from') || '';
  const from = /^[a-z0-9-]{1,80}$/.test(fromRaw) ? `&from=${fromRaw}` : '';
  const u = await currentUser(req, env);
  if (!u) return redirect(`${url.origin}/login/?next=${encodeURIComponent(`/pricing/?buy=${plan.key}${from}`)}`);
  if (plan.founder && !(await founderState(env)).open) return redirect(`${url.origin}/pricing/?founder=closed${from}`);
  const ent = await plan_for(env, u);
  if (ent.mine || ent.admin) return redirect(`${url.origin}/account/?billing=active#plan`);
  const to = new URL(plan.link);
  to.searchParams.set('customer_email', u.email);
  if (u.name) to.searchParams.set('customer_name', u.name.slice(0, 100));
  to.searchParams.set('reference_id', u.id);
  to.searchParams.set('utm_source', 'oxyn-site');
  return redirect(to.toString());
}

async function billingMe(env: Env, u: Who) {
  const e = await plan_for(env, u);
  let seats: { email: string; created_at: string }[] | null = null;
  if (e.mine && e.mine.plan === 'team') {
    seats = (await env.DB.prepare('SELECT email, created_at FROM team_seats WHERE subscription_id = ?1 AND removed_at IS NULL ORDER BY created_at')
      .bind(e.mine.id).all<{ email: string; created_at: string }>()).results ?? [];
  }
  return json({
    ok: true,
    tier: e.tier,
    pro: e.pro,
    admin: e.admin,
    via: e.via,
    subscription: pub(e.latest),
    seat: e.seat ? { owner_email: e.seat.owner_email, plan: 'team' } : null,
    grant: e.grant ? { plan: e.grant.plan, expires_at: e.grant.expires_at } : null,
    seats,
    seat_limit: POLAR.seats,
    people_limit: POLAR.people,
    has_billing: !!e.customer,
    email_verified: !!u.email_verified,
    usage: await usageFor(env, u, e),
  });
}

/** Polar's customer portal: a pre-signed session when POLAR_ACCESS_TOKEN is set, else the portal's
 *  own sign-in (Polar emails a code to the address the subscription was bought with). */
async function portal(env: Env, url: URL, u: Who) {
  const e = await plan_for(env, u);
  if (env.POLAR_ACCESS_TOKEN && e.customer) {
    try {
      const r = await fetch(`${POLAR.api}/customer-sessions/`, {
        method: 'POST',
        headers: { authorization: `Bearer ${env.POLAR_ACCESS_TOKEN}`, 'content-type': 'application/json', accept: 'application/json' },
        body: JSON.stringify({ customer_id: e.customer, return_url: `${url.origin}/account/` }),
      });
      if (r.ok) {
        const s = (await r.json()) as { customer_portal_url?: string };
        if (s.customer_portal_url?.startsWith('https://')) return redirect(s.customer_portal_url);
      } else console.error('polar customer session', r.status, (await r.text()).slice(0, 200));
    } catch (err) { console.error('polar customer session', err); }
  }
  return redirect(POLAR.portal);
}

async function body(req: Request): Promise<Record<string, unknown>> {
  if (!(req.headers.get('content-type') || '').startsWith('application/json')) throw new HttpError(415, 'unsupported_media_type', 'Send application/json.');
  if (Number(req.headers.get('content-length') || '0') > 4096) throw new HttpError(413, 'too_large', 'That request is too large.');
  try { return (await req.json()) as Record<string, unknown>; } catch { throw new HttpError(400, 'bad_body', 'The request could not be read.'); }
}

async function teamOf(env: Env, u: Who) {
  const e = await plan_for(env, u);
  if (!e.mine || e.mine.plan !== 'team') throw new HttpError(403, 'not_team_owner', 'Only the owner of an active Team plan can change its seats.');
  return e.mine;
}

async function addSeat(req: Request, env: Env, url: URL, u: Who) {
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
  const b = await body(req);
  const email = String(b.email || '').trim().toLowerCase();
  if (!email || email.length > 254 || !EMAIL_RE.test(email)) throw new HttpError(400, 'invalid_email', 'Enter a valid email address.', 'email');
  const sub = await teamOf(env, u);
  const n = await env.DB.prepare('SELECT COUNT(*) AS n FROM team_seats WHERE subscription_id = ?1 AND removed_at IS NULL').bind(sub.id).first<{ n: number }>();
  const ex = await env.DB.prepare('SELECT removed_at FROM team_seats WHERE subscription_id = ?1 AND email = ?2').bind(sub.id, email).first<{ removed_at: string | null }>();
  if (ex && !ex.removed_at) throw new HttpError(409, 'already_seated', 'That email already has a seat.', 'email');
  if (Number(n?.n ?? 0) >= POLAR.seats) throw new HttpError(409, 'seats_full', `A Team is up to ${POLAR.people} people: you and ${POLAR.seats} teammates. Remove one first.`, 'email');
  await env.DB.prepare(
    'INSERT INTO team_seats (subscription_id, email, invited_by, created_at) VALUES (?1, ?2, ?3, ?4) ' +
      'ON CONFLICT (subscription_id, email) DO UPDATE SET removed_at = NULL, invited_by = ?3, created_at = ?4',
  ).bind(sub.id, email, u.id, iso()).run();
  return json({ ok: true }, 201);
}

async function removeSeat(req: Request, env: Env, url: URL, u: Who, raw: string) {
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
  let email = '';
  try { email = decodeURIComponent(raw).toLowerCase(); } catch { throw new HttpError(400, 'invalid_email', 'Bad email.'); }
  const sub = await teamOf(env, u);
  const r = await env.DB.prepare('UPDATE team_seats SET removed_at = ?3 WHERE subscription_id = ?1 AND email = ?2 AND removed_at IS NULL').bind(sub.id, email, iso()).run();
  if (!Number(r.meta?.changes ?? 0)) throw new HttpError(404, 'not_found', 'That email has no seat.');
  return json({ ok: true });
}

// ── Webhook ──────────────────────────────────────────────────────────────────────────────────
const enc = new TextEncoder();
function b64(bytes: Uint8Array) { let s = ''; for (const b of bytes) s += String.fromCharCode(b); return btoa(s); }
function unb64(s: string) { const b = atob(s); return Uint8Array.from(b, (c) => c.charCodeAt(0)); }
function sameText(a: string, b: string) {
  const x = enc.encode(a), y = enc.encode(b);
  if (x.length !== y.length) return false;
  let d = 0;
  for (let i = 0; i < x.length; i++) d |= x[i] ^ y[i];
  return d === 0;
}
async function hmac(key: Uint8Array, msg: string) {
  const k = await crypto.subtle.importKey('raw', key, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
  return b64(new Uint8Array(await crypto.subtle.sign('HMAC', k, enc.encode(msg))));
}

/** Standard Webhooks: HMAC-SHA256 over "<id>.<timestamp>.<body>", base64, in "v1,<sig>" entries.
 *  The key is the base64 part of the `whsec_` secret; secrets Polar made before 8 Sep 2026 use the
 *  UTF-8 bytes of the whole secret instead, so both are tried. Exported for the tests. */
export async function verifySignature(secret: string, id: string, ts: string, sig: string, raw: string, now = Date.now() / 1000) {
  if (!id || !ts || !sig || !/^\d{1,12}$/.test(ts)) return false;
  if (Math.abs(now - Number(ts)) > POLAR.tolerance) return false;
  const msg = `${id}.${ts}.${raw}`;
  const keys: Uint8Array[] = [];
  const s = secret.trim();
  try { keys.push(unb64(s.replace(/^whsec_/, ''))); } catch { /* not base64 */ }
  keys.push(enc.encode(s));
  const given = sig.split(' ').map((p) => p.split(',')).filter(([v, x]) => v === 'v1' && x).map(([, x]) => x);
  for (const k of keys) {
    const want = await hmac(k, msg);
    if (given.some((g) => sameText(g, want))) return true;
  }
  return false;
}

async function webhook(req: Request, env: Env, ctx: Ctx) {
  if (!env.POLAR_WEBHOOK_SECRET) throw new HttpError(503, 'billing_not_configured', 'Polar webhooks are not set up on this Worker.');
  if (Number(req.headers.get('content-length') || '0') > POLAR.maxBody) throw new HttpError(413, 'too_large', 'Payload too large.');
  const raw = await req.text();
  if (raw.length > POLAR.maxBody) throw new HttpError(413, 'too_large', 'Payload too large.');
  const id = req.headers.get('webhook-id') || '';
  const ok = await verifySignature(env.POLAR_WEBHOOK_SECRET, id, req.headers.get('webhook-timestamp') || '', req.headers.get('webhook-signature') || '', raw);
  if (!ok) throw new HttpError(403, 'bad_signature', 'Invalid webhook signature.');
  let ev: { type?: string; data?: Record<string, any> };
  try { ev = JSON.parse(raw); } catch { throw new HttpError(400, 'bad_body', 'Not JSON.'); }
  const type = String(ev.type || '');
  const ins = await env.DB.prepare('INSERT OR IGNORE INTO webhook_events (id, type, received_at) VALUES (?1, ?2, ?3)').bind(id.slice(0, 200), type.slice(0, 80), iso()).run();
  if (!Number(ins.meta?.changes ?? 1)) return json({ ok: true, duplicate: true });
  try {
    await handleEvent(env, ctx, type, ev.data || {});
  } catch (err) {
    // Let Polar retry this delivery.
    await env.DB.prepare('DELETE FROM webhook_events WHERE id = ?1').bind(id.slice(0, 200)).run();
    throw err;
  }
  if (Math.random() < 0.02) ctx.waitUntil(env.DB.prepare("DELETE FROM webhook_events WHERE received_at < ?1").bind(new Date(Date.now() - 30 * 86_400_000).toISOString()).run());
  return json({ ok: true });
}

/** Exported for the tests. */
export async function handleEvent(env: Env, ctx: Ctx, type: string, d: Record<string, any>) {
  if (type.startsWith('subscription.')) return upsertSubscription(env, ctx, d, type);
  if (type === 'order.paid' && d.subscription && typeof d.subscription === 'object') {
    return upsertSubscription(env, ctx, { ...d.subscription, customer: d.subscription.customer ?? d.customer, product: d.subscription.product ?? d.product }, type);
  }
  if (type === 'checkout.updated' && d.status === 'succeeded') {
    const ref = typeof d.metadata?.reference_id === 'string' ? d.metadata.reference_id : '';
    const user = ref && (await env.DB.prepare('SELECT id FROM users WHERE id = ?1').bind(ref).first<{ id: string }>());
    if (!user || !d.id) return;
    await env.DB.batch([
      env.DB.prepare('INSERT OR IGNORE INTO checkout_refs (checkout_id, user_id, email, created_at) VALUES (?1, ?2, ?3, ?4)').bind(String(d.id), user.id, d.customer_email ?? null, iso()),
      env.DB.prepare('UPDATE subscriptions SET user_id = ?2 WHERE checkout_id = ?1 AND user_id IS NULL').bind(String(d.id), user.id),
      env.DB.prepare('UPDATE licences SET user_id = ?2 WHERE user_id IS NULL AND subscription_id IN (SELECT id FROM subscriptions WHERE checkout_id = ?1)').bind(String(d.id), user.id),
    ]);
  }
}

async function userFor(env: Env, d: Record<string, any>): Promise<string | null> {
  const tries: [string, unknown][] = [
    ['SELECT id FROM users WHERE id = ?1', d.metadata?.reference_id],
    ['SELECT user_id AS id FROM checkout_refs WHERE checkout_id = ?1', d.checkout_id],
    ['SELECT id FROM users WHERE id = ?1', d.customer?.external_id],
  ];
  for (const [q, v] of tries) {
    if (typeof v !== 'string' || !v) continue;
    const r = await env.DB.prepare(q).bind(v).first<{ id: string }>();
    if (r?.id) return r.id;
  }
  return null;
}

async function upsertSubscription(env: Env, ctx: Ctx, d: Record<string, any>, type: string) {
  const plan = byProduct.get(String(d.product_id || d.product?.id || ''));
  if (!d.id || !plan) { console.log('polar webhook: not an Oxyn subscription', type, d.product_id); return; }
  const prev = await env.DB.prepare('SELECT polar_modified_at FROM subscriptions WHERE id = ?1').bind(String(d.id)).first<{ polar_modified_at: string | null }>();
  const mod = String(d.modified_at || d.created_at || '');
  // Deliveries can arrive out of order: never let an older state overwrite a newer one.
  if (prev?.polar_modified_at && mod && mod < prev.polar_modified_at) return;
  const email = String(d.customer?.email || '').toLowerCase();
  if (!email) throw new Error('subscription without customer email');
  const userId = await userFor(env, d);
  const now = iso();
  const status = String(d.status || 'incomplete');
  const ended = d.ended_at ?? (type === 'subscription.revoked' ? now : null);
  await env.DB.batch([
    env.DB.prepare(
      'INSERT INTO subscriptions (id, user_id, customer_id, customer_email, product_id, plan, interval, founder, status, amount, currency, ' +
        'cancel_at_period_end, current_period_end, started_at, ended_at, checkout_id, polar_modified_at, created_at, updated_at, current_period_start) ' +
        'VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?18, ?19) ' +
        'ON CONFLICT (id) DO UPDATE SET user_id = COALESCE(excluded.user_id, subscriptions.user_id), customer_id = excluded.customer_id, ' +
        'customer_email = excluded.customer_email, product_id = excluded.product_id, plan = excluded.plan, interval = excluded.interval, ' +
        'founder = excluded.founder, status = excluded.status, amount = excluded.amount, currency = excluded.currency, ' +
        'cancel_at_period_end = excluded.cancel_at_period_end, current_period_end = excluded.current_period_end, started_at = excluded.started_at, ' +
        'ended_at = excluded.ended_at, checkout_id = COALESCE(excluded.checkout_id, subscriptions.checkout_id), ' +
        'polar_modified_at = excluded.polar_modified_at, updated_at = excluded.updated_at, current_period_start = excluded.current_period_start',
    ).bind(
      String(d.id), userId, String(d.customer_id || d.customer?.id || ''), email, plan.product, plan.plan, plan.interval, plan.founder ? 1 : 0, status,
      Number.isFinite(d.amount) ? d.amount : plan.cents, String(d.currency || 'usd'), d.cancel_at_period_end ? 1 : 0,
      d.current_period_end ?? null, d.started_at ?? null, ended, d.checkout_id ?? null, mod || null, now, d.current_period_start ?? null,
    ),
    env.DB.prepare(
      'INSERT INTO licences (id, subscription_id, user_id, email, plan, created_at, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ' +
        'ON CONFLICT (subscription_id) DO UPDATE SET user_id = COALESCE(excluded.user_id, licences.user_id), email = excluded.email, ' +
        'plan = excluded.plan, revoked_at = excluded.revoked_at',
    ).bind(crypto.randomUUID(), String(d.id), userId, email, plan.plan, now, ended ? now : null),
  ]);
  if (plan.founder && ACTIVE.includes(status) && !ended) {
    await env.DB.prepare('INSERT OR IGNORE INTO founder_claims (subscription_id, email, claimed_at) VALUES (?1, ?2, ?3)').bind(String(d.id), email, now).run();
  }
  const f = await founderState(env);
  if (!f.open) ctx.waitUntil(archiveFounder(env).catch((e) => console.error('founder archive', e)));
}

/** At 100 founder subscriptions: archive both founder products in Polar so no new founder checkout
 *  can start (existing founder subscriptions keep renewing at their rate). Runs once; retried on
 *  later webhooks while it has not succeeded. Needs POLAR_ACCESS_TOKEN. */
export async function archiveFounder(env: Env) {
  const st = await env.DB.prepare("SELECT value FROM settings WHERE key = 'founder_archive'").first<{ value: string }>();
  if (st?.value === 'done') return;
  const save = (v: string) => env.DB.prepare("INSERT INTO settings (key, value, updated_at) VALUES ('founder_archive', ?1, ?2) ON CONFLICT (key) DO UPDATE SET value = ?1, updated_at = ?2").bind(v, iso()).run();
  if (!env.POLAR_ACCESS_TOKEN) { await save('pending: set the POLAR_ACCESS_TOKEN Worker secret'); return; }
  const errs: string[] = [];
  for (const p of PLANS.filter((x) => x.founder)) {
    const r = await fetch(`${POLAR.api}/products/${p.product}`, {
      method: 'PATCH',
      headers: { authorization: `Bearer ${env.POLAR_ACCESS_TOKEN}`, 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ is_archived: true }),
    });
    if (!r.ok) errs.push(`${p.key} ${r.status}`);
  }
  await save(errs.length ? `error: ${errs.join(', ')}` : 'done');
}

// ── Admin ────────────────────────────────────────────────────────────────────────────────────
/** GET /api/admin/billing: subscribers and the recurring revenue they add up to. */
export async function adminBilling(env: Env) {
  return json(await billingSummary(env));
}

/** Subscribers, counts and MRR (also used by the admin Activity tab). */
export async function billingSummary(env: Env) {
  const rows = (await env.DB.prepare(
    'SELECT s.id, s.customer_email, s.plan, s.interval, s.founder, s.status, s.amount, s.currency, s.cancel_at_period_end, ' +
      's.current_period_end, s.started_at, s.ended_at, s.created_at, s.user_id IS NOT NULL AS linked, ' +
      "(SELECT COUNT(*) FROM team_seats t WHERE t.subscription_id = s.id AND t.removed_at IS NULL) AS seats " +
      'FROM subscriptions s ORDER BY s.created_at DESC LIMIT 500',
  ).all<SubRow & { linked: number; seats: number }>()).results ?? [];
  let mrr = 0;
  const counts = { active: 0, pro: 0, team: 0, founder: 0, canceling: 0, ended: 0 };
  for (const s of rows) {
    if (live(s)) {
      counts.active++;
      counts[s.plan]++;
      if (s.founder) counts.founder++;
      if (s.cancel_at_period_end) counts.canceling++;
      mrr += s.interval === 'year' ? s.amount / 12 : s.amount;
    } else counts.ended++;
  }
  const founder = await founderState(env);
  const st = await env.DB.prepare("SELECT value FROM settings WHERE key = 'founder_archive'").first<{ value: string }>();
  return {
    ok: true,
    mrr_cents: Math.round(mrr),
    arr_cents: Math.round(mrr * 12),
    counts,
    founder: { ...founder, archive: st?.value ?? null },
    webhook_ready: !!env.POLAR_WEBHOOK_SECRET,
    api_token: !!env.POLAR_ACCESS_TOKEN,
    items: rows.map((s) => ({ ...s, active: live(s) })),
  };
}
