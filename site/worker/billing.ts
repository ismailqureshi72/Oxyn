// Oxyn Pro and Team, sold through Polar (merchant of record: Polar charges the card, adds the tax
// and sends the receipts). See docs/pricing.md.
//
//   GET    /api/billing/plans            public: plans, prices, founder spots left (cached 60 s)
//   GET    /api/billing/checkout?plan=   signed in: redirect to the plan's Polar checkout link with the
//                                        account's email and id (reference_id) filled in; founder plans:
//                                        a checkout session made for this buyer after the checks
//   GET    /api/billing/me               signed in: plan, status, renewal, Team seats
//   GET    /api/billing/subscription     signed in: the billing section of the account page, read live
//                                        from Polar: plan, price, next payment, scheduled change, card,
//                                        and (subscription owner) the plans it can switch to
//   GET    /api/billing/preview?plan=    subscription owner: what switching to that plan charges and when
//   POST   /api/billing/change           subscription owner: {plan} switches plan (rule: changeRule)
//   POST   /api/billing/keep             subscription owner: drops a switch scheduled for the renewal
//   GET    /api/billing/invoices         signed in: the account's own Polar orders (newest first)
//   GET    /api/billing/invoices/<id>/receipt|invoice   signed in, own order only: {url} of the PDF
//   GET    /api/billing/card             subscription owner: Polar's hosted card page (customer session);
//                                        the only Polar page a customer is sent to (see updateCard)
//   GET    /api/billing/portal           old links: 301 to the billing section of the account page
//   POST   /api/billing/cancel           subscription owner: cancel at the period end (no refund; Pro
//                                        stays open until current_period_end, then it does not renew)
//   POST   /api/billing/resume           subscription owner: undo that cancel before the period ends
//   POST   /api/billing/seats            Team owner: {email} takes one of the 3 teammate seats (4 people with the owner)
//   DELETE /api/billing/seats/<email>    Team owner: frees a seat
//   GET    /api/pro/source/<slug>        Pro: the usage code of a Pro item this account's pool already
//                                        opened this month (else 403 unlock_required + usage)
//   POST   /api/pro/source/<slug>        Pro, from the site: opens it (one of the month's caps, pro.ts)
//   POST   /api/polar/webhook            Polar events, signed (Standard Webhooks)
//
// Regular plans sell through public Polar checkout links; the founder intro price has no link and
// each founder checkout is a session the Worker creates after its checks (founderCheckout). The signed
// webhook writes subscriptions, licences and the founder counter to D1. Billing after the sale
// happens on the account page, through the Polar API with the Worker secret POLAR_ACCESS_TOKEN
// (never with an organization_id: the token is an organization token): cancel / resume, switching
// plan, invoices and receipts, the card on file. The one exception is changing the card: Polar's
// API only adds a card from a Stripe confirmation token made with Polar's own Stripe account, so
// "Update card" opens Polar's hosted page through a pre-signed customer session. Plan changes are
// switched off in Polar's customer portal (organization customer_portal_settings), so plans change
// only here, under changeRule. The token also schedules every founder subscription to renew as
// regular Pro (below) and archives the founder products once 100 founder subscriptions exist.
//
// The founder rate is an intro price, not a locked-in one: $12.99 for the first month (or $124.45
// for the first year), then regular Pro at $19.99 a month ($199.99 a year), for the first 100 new
// subscribers. Polar charges what the subscription's product costs, so when a founder subscription
// starts, the webhook schedules a product change to the matching regular Pro product with
// proration_behavior "next_period": nothing is charged now, and the first renewal bills the regular
// price. Only someone who never had an Oxyn subscription can buy (or claim) the founder rate.

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

/** The six Polar products (production, organization "oxyn") and their checkout links. The founder
 *  products have none on purpose: their checkouts are created per buyer by founderCheckout. */
export const PLANS: Plan[] = [
  { key: 'pro-month', plan: 'pro', interval: 'month', founder: false, cents: 1999, product: '677990ee-37fd-4144-a7b4-fc0072db0236', link: 'https://buy.polar.sh/polar_cl_nOTwWhBDBia5V47drawKkcEPM1g1ClF73jekN2cDk0k' },
  { key: 'pro-year', plan: 'pro', interval: 'year', founder: false, cents: 19999, product: '6c6f941c-eb68-42a1-9588-6999793d7723', link: 'https://buy.polar.sh/polar_cl_TIpeARSu8pHt9MTbFwisPzixrXIdJhlEBJOHG2B2fYk' },
  { key: 'pro-founder-month', plan: 'pro', interval: 'month', founder: true, cents: 1299, product: 'dd6c77e0-537e-4575-8272-179818c4636c', link: '' },
  { key: 'pro-founder-year', plan: 'pro', interval: 'year', founder: true, cents: 12445, product: 'b2c7ff7d-5ed5-4677-b4c2-023860cf6440', link: '' },
  { key: 'team-month', plan: 'team', interval: 'month', founder: false, cents: 4499, product: '47ac4829-9dea-4c5f-a47c-948eb3005865', link: 'https://buy.polar.sh/polar_cl_NCvpX9ojhDrafHw1bLtki9PTVUOePRTSPrUgL2ipK4Y' },
  { key: 'team-year', plan: 'team', interval: 'year', founder: false, cents: 44900, product: 'c9ad78fa-8fd7-4f81-a19a-af4c6472f000', link: 'https://buy.polar.sh/polar_cl_tGlTFd4OLxE5lYiu8NWR2eDjPRkrJGfUUcEwd1eo3vC' },
];
const byProduct = new Map(PLANS.map((p) => [p.product, p]));
/** The regular plan a founder plan renews as after its first period (same plan and interval). */
export const renewsAs = (p: Plan): Plan | undefined => (p.founder ? PLANS.find((x) => !x.founder && x.plan === p.plan && x.interval === p.interval) : undefined);
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
  customer: string | null; // Polar customer id of its newest own subscription
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
  // A founder subscription renews at the regular price: what it will cost from the next period.
  renews_at_amount: s.founder ? (renewsAs(byProduct.get(s.product_id) ?? PLANS[0])?.cents ?? null) : null,
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
      ctx.waitUntil(polarCheck(env).catch((e) => console.error('polar check', e)));
      return json({ ok: true, founder, plans: PLANS.filter((p) => founder.open || !p.founder).map(({ key, plan, interval, founder: f, cents }) => ({ key, plan, interval, founder: f, cents })) },
        200, { 'cache-control': 'public, max-age=60' });
    }
    if (route === '/api/billing/checkout' && (m === 'GET' || m === 'HEAD')) {
      if (isPrefetch(req)) return new Response(null, { status: 204, headers: SEC });
      return await checkout(req, env, url);
    }
    const u = await currentUser(req, env);
    // Billing lives on the account page now; old "Manage billing" links land there.
    if (route === '/api/billing/portal' && m === 'GET') return redirect(`${url.origin}/account/#billing`, { 'cache-control': 'no-store' });
    if (route === '/api/billing/card' && m === 'GET') {
      if (isPrefetch(req)) return new Response(null, { status: 204, headers: SEC });
      if (!u) return redirect(`${url.origin}/login/?next=${encodeURIComponent('/account/#billing')}`);
      return await updateCard(env, url, u);
    }
    if (!u) throw new HttpError(401, 'signed_out', 'Sign in to Oxyn first.');
    if (route === '/api/billing/me' && m === 'GET') return await billingMe(env, u);
    if (route === '/api/billing/subscription' && m === 'GET') return await billingDetails(env, u);
    if (route === '/api/billing/preview' && m === 'GET') return await previewChange(env, u, url.searchParams.get('plan') || '');
    if (route === '/api/billing/change' && m === 'POST') return await changePlan(req, env, ctx, url, u);
    if (route === '/api/billing/keep' && m === 'POST') return await keepPlan(req, env, ctx, url, u);
    if (route === '/api/billing/invoices' && m === 'GET') return await listInvoices(env, u);
    const inv = route.match(/^\/api\/billing\/invoices\/([0-9a-f-]{36})\/(receipt|invoice)$/);
    if (inv && m === 'GET') return await invoiceFile(env, u, inv[1], inv[2] as 'receipt' | 'invoice');
    if ((route === '/api/billing/cancel' || route === '/api/billing/resume') && m === 'POST') return await setCancel(req, env, ctx, url, u, route.endsWith('/cancel'));
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
  // The founder rate is for new subscribers only: anyone who had an Oxyn subscription before (by
  // account, or by email in Polar's records) is sent to the regular price.
  if (plan.founder && (ent.latest || (await hadSubscription(env, u.id, u.email)))) {
    const regular = renewsAs(plan)!;
    return redirect(`${url.origin}/pricing/?founder=used&interval=${regular.interval}${from}`);
  }
  if (plan.founder) return founderCheckout(env, url, u, plan, from);
  const to = new URL(plan.link);
  to.searchParams.set('customer_email', u.email);
  if (u.name) to.searchParams.set('customer_name', u.name.slice(0, 100));
  to.searchParams.set('reference_id', u.id);
  to.searchParams.set('utm_source', 'oxyn-site');
  return redirect(to.toString());
}

/** The founder intro price has no public checkout link: a founder checkout is created here, one
 *  per buyer, after the checks above (signed in, spots left, never subscribed), through Polar's
 *  checkout session API. Opening a Polar link directly can no longer start a founder subscription.
 *  If Polar does not answer, the buyer goes back to pricing (never to a link). */
async function founderCheckout(env: Env, url: URL, u: Who & { name?: string | null }, plan: Plan, from: string) {
  const back = `${url.origin}/pricing/?checkout=unavailable${from}`;
  if (!env.POLAR_ACCESS_TOKEN) return redirect(back);
  let r: { status: number; data: Polar };
  try {
    r = await polar(env, '/checkouts/', {
      method: 'POST',
      body: {
        products: [plan.product],
        customer_email: u.email,
        ...(u.name ? { customer_name: u.name.slice(0, 100) } : {}),
        metadata: { reference_id: u.id, oxyn_plan: plan.plan, oxyn_interval: plan.interval, oxyn_founder: true, utm_source: 'oxyn-site' },
        success_url: `${url.origin}/pricing/success?checkout_id={CHECKOUT_ID}`,
        return_url: `${url.origin}/pricing/`,
      },
    });
  } catch (err) {
    console.error('founder checkout', err);
    return redirect(back);
  }
  const to = typeof r.data.url === 'string' ? r.data.url : '';
  if ((r.status !== 201 && r.status !== 200) || !to.startsWith('https://')) {
    console.error('founder checkout', r.status, JSON.stringify(r.data).slice(0, 300));
    return redirect(back);
  }
  return redirect(to);
}

/** Whether this account or email ever had an Oxyn subscription (any plan, any state). `except`
 *  leaves one subscription out (the one a webhook is handling). */
async function hadSubscription(env: Env, userId: string | null, email: string, except = '') {
  const r = await env.DB.prepare(
    'SELECT 1 AS one FROM subscriptions WHERE id != ?3 AND ((?1 IS NOT NULL AND user_id = ?1) OR lower(customer_email) = lower(?2)) LIMIT 1',
  ).bind(userId, email, except).first();
  return !!r;
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

/** Cancel at the period end, or resume, the account's own live subscription (Pro, or a Team it
 *  owns; a teammate's seat is not theirs to cancel). Polar keeps access until current_period_end and
 *  issues no refund. D1 is updated from Polar's answer at once; the webhook that follows agrees. */
async function setCancel(req: Request, env: Env, ctx: Ctx, url: URL, u: Who, cancel: boolean) {
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
  const e = await plan_for(env, u);
  const sub = e.mine;
  if (!sub) throw new HttpError(404, 'no_subscription', e.seat ? 'Only the Team owner can cancel this plan.' : 'This account has no active subscription to cancel.');
  if (!!sub.cancel_at_period_end === cancel) return json({ ok: true, subscription: pub(sub) });
  if (!env.POLAR_ACCESS_TOKEN) throw new HttpError(503, 'billing_api_unavailable', 'Billing changes are not available right now. Please try again later.');
  let r: Response;
  try {
    r = await fetch(`${POLAR.api}/subscriptions/${encodeURIComponent(sub.id)}`, {
      method: 'PATCH', headers: polarHeaders(env), body: JSON.stringify({ cancel_at_period_end: cancel }),
    });
  } catch (err) {
    console.error('polar cancel', sub.id, err);
    throw new HttpError(502, 'billing_unreachable', 'Could not reach our billing provider. Please try again.');
  }
  if (!r.ok) {
    console.error('polar cancel', sub.id, r.status, (await r.text()).slice(0, 300));
    throw new HttpError(502, 'billing_failed', 'Our billing provider did not accept the change. Please try again.');
  }
  const row = await syncFromPolar(env, ctx, (await r.json()) as Record<string, any>, sub, u, cancel ? 'api.cancel' : 'api.resume', { cancel_at_period_end: cancel });
  return json({ ok: true, subscription: pub(row ?? sub) });
}

// ── In-site billing (account page) ───────────────────────────────────────────────────────────
// Everything a subscriber manages happens on oxynui.com/account/#billing through the Polar API.
// Only "Update card" opens a Polar page (see updateCard).

/** When a plan switch takes effect, and how Polar bills it (proration_behavior):
 *  - Pro -> Team on the same interval: right away, `invoice`: Polar charges the prorated difference
 *    for the rest of the current period now, and the next renewal bills the full Team price.
 *  - Everything else (monthly <-> yearly, Team -> Pro): at the next renewal, `next_period`: nothing
 *    is charged or refunded now; the renewal bills the new plan.
 *  Founder products are never a target (founder: false only). Exported for the tests. */
export function changeRule(from: Plan, to: Plan): { when: 'now' | 'renewal'; proration: 'invoice' | 'next_period' } {
  return from.interval === to.interval && from.plan === 'pro' && to.plan === 'team'
    ? { when: 'now', proration: 'invoice' }
    : { when: 'renewal', proration: 'next_period' };
}
/** The plans an account can switch to: regular plans other than the current one. */
export const switchTargets = (current: Plan) => PLANS.filter((p) => !p.founder && p.product !== current.product);
/** Estimated charge now for an immediate switch: the price difference for the share of the period
 *  that is left (Polar's own proration; tax is added by Polar on top). */
export function prorated(currentCents: number, to: Plan, start: string | null, end: string | null, nowMs = Date.now()) {
  const s = Date.parse(start || ''), e = Date.parse(end || '');
  if (!Number.isFinite(s) || !Number.isFinite(e) || e <= s) return Math.max(0, to.cents - currentCents);
  const left = Math.min(1, Math.max(0, (e - nowMs) / (e - s)));
  return Math.max(0, Math.round((to.cents - currentCents) * left));
}

type Polar = Record<string, any>;
async function polar(env: Env, path: string, init: { method?: string; body?: unknown } = {}): Promise<{ status: number; data: Polar }> {
  if (!env.POLAR_ACCESS_TOKEN) throw new HttpError(503, 'billing_api_unavailable', 'Billing is not available right now. Please try again later.');
  let r: Response;
  try {
    r = await fetch(`${POLAR.api}${path}`, { method: init.method || 'GET', headers: polarHeaders(env), body: init.body === undefined ? undefined : JSON.stringify(init.body) });
  } catch (err) {
    console.error('polar', path, err);
    throw new HttpError(502, 'billing_unreachable', 'Could not reach our billing provider. Please try again.');
  }
  let data: Polar = {};
  try { data = (await r.json()) as Polar; } catch { data = {}; }
  if (r.status >= 500 || r.status === 401 || r.status === 403) {
    console.error('polar', init.method || 'GET', path, r.status, JSON.stringify(data).slice(0, 300));
    throw new HttpError(502, 'billing_failed', 'Our billing provider did not answer as expected. Please try again.');
  }
  return { status: r.status, data };
}

/** Writes Polar's answer for one subscription to D1, like a webhook would (fields Polar leaves out
 *  come from the D1 row), and returns the row. */
async function syncFromPolar(env: Env, ctx: Ctx, d: Polar, sub: SubRow, u: Who, type: string, fallback: Polar = {}) {
  await upsertSubscription(env, ctx, {
    ...fallback,
    ...d,
    id: d.id ?? sub.id,
    modified_at: d.modified_at ?? iso(),
    cancel_at_period_end: d.cancel_at_period_end ?? fallback.cancel_at_period_end ?? !!sub.cancel_at_period_end,
    customer: d.customer ?? { id: sub.customer_id, email: sub.customer_email },
    customer_id: d.customer_id ?? sub.customer_id,
    product_id: d.product_id ?? d.product?.id ?? sub.product_id,
    current_period_end: d.current_period_end ?? sub.current_period_end,
    current_period_start: d.current_period_start ?? sub.current_period_start ?? null,
    started_at: d.started_at ?? sub.started_at,
    status: d.status ?? sub.status,
    amount: d.amount ?? sub.amount,
    currency: d.currency ?? sub.currency,
    metadata: { reference_id: u.id },
  }, type);
  return env.DB.prepare('SELECT * FROM subscriptions WHERE id = ?1').bind(sub.id).first<SubRow>();
}

/** The account's own live subscription (Pro, or a Team it owns), else a 404 naming why. */
async function ownSub(env: Env, u: Who, what = 'change this plan') {
  const e = await plan_for(env, u);
  if (!e.mine) throw new HttpError(404, 'no_subscription', e.seat ? `Only the Team owner can ${what}.` : 'This account has no active subscription.');
  return { e, sub: e.mine };
}
const planOf = (product: string | null | undefined) => (product ? byProduct.get(product) : undefined);
const planOut = (p: Plan) => ({ key: p.key, plan: p.plan, interval: p.interval, founder: p.founder, cents: p.cents });

/** What the account page shows about one live Polar subscription. */
function describe(s: Polar, row: SubRow, owner: boolean) {
  const cur = planOf(s.product_id) ?? planOf(row.product_id)!;
  const pendingPlan = planOf(s.pending_update?.product_id);
  const founderRenewal = !!(cur.founder && pendingPlan && pendingPlan.product === renewsAs(cur)?.product);
  const ends = !!s.cancel_at_period_end || !!s.ended_at;
  const nextPlan = ends ? null : pendingPlan ?? renewsAs(cur) ?? cur;
  // A founder (intro) price renews at the regular price; otherwise the next charge is what Polar
  // bills today (which includes any discount), unless a switch is scheduled.
  const nextCents = !nextPlan ? null : nextPlan.product === cur.product ? Number(s.amount ?? row.amount) : nextPlan.cents;
  const canSwitch = owner && ['active', 'trialing'].includes(String(s.status)) && !s.cancel_at_period_end && !s.ended_at;
  return {
    plan: planOut(cur),
    status: String(s.status ?? row.status),
    amount: Number(s.amount ?? row.amount),
    currency: String(s.currency ?? row.currency ?? 'usd'),
    cancel_at_period_end: !!s.cancel_at_period_end,
    current_period_start: s.current_period_start ?? row.current_period_start ?? null,
    current_period_end: s.current_period_end ?? row.current_period_end,
    next: nextPlan ? { date: s.current_period_end ?? row.current_period_end, cents: nextCents, plan: planOut(nextPlan), founder_ends: cur.founder } : null,
    pending: pendingPlan ? { plan: planOut(pendingPlan), applies_at: s.pending_update.applies_at ?? s.current_period_end, founder_renewal: founderRenewal } : null,
    options: canSwitch ? switchTargets(cur).map((p) => ({ ...planOut(p), ...changeRule(cur, p), scheduled: pendingPlan?.product === p.product })) : [],
    blocked: !owner ? 'not_owner' : s.cancel_at_period_end ? 'cancelling' : String(s.status) === 'past_due' ? 'past_due' : canSwitch ? null : 'inactive',
  };
}

async function readSub(env: Env, sub: SubRow) {
  const r = await polar(env, `/subscriptions/${encodeURIComponent(sub.id)}`);
  if (r.status !== 200) throw new HttpError(502, 'billing_failed', 'Could not read your subscription from our billing provider.');
  return r.data;
}

/** GET /api/billing/subscription: the billing section. A teammate on a Team seat sees no billing
 *  (it is the owner's); an account with no own subscription gets subscription: null. */
async function billingDetails(env: Env, u: Who) {
  const e = await plan_for(env, u);
  const row = e.mine;
  const customers = await customerIds(env, u);
  if (!row) return json({ ok: true, subscription: null, has_history: customers.length > 0, seat: !!e.seat });
  const s = await readSub(env, row);
  let card: { brand: string; last4: string; exp_month: number; exp_year: number; wallet: string | null } | null = null;
  try {
    const pm = await polar(env, `/customers/${encodeURIComponent(row.customer_id)}/payment-methods`);
    const c = (pm.data.items ?? []).find((x: Polar) => x.type === 'card' && x.method_metadata);
    if (c) card = { brand: String(c.method_metadata.brand), last4: String(c.method_metadata.last4), exp_month: Number(c.method_metadata.exp_month), exp_year: Number(c.method_metadata.exp_year), wallet: c.method_metadata.wallet ?? null };
  } catch (err) { console.error('polar payment methods', err); }
  return json({ ok: true, subscription: describe(s, row, true), card, has_history: true, seat: false });
}

async function targetFor(env: Env, u: Who, key: string) {
  const to = byKey.get(key as PlanKey);
  if (!to || to.founder) throw new HttpError(400, 'invalid_plan', 'That plan cannot be chosen here.');
  const { sub } = await ownSub(env, u);
  const s = await readSub(env, sub);
  const d = describe(s, sub, true);
  const cur = planOf(s.product_id) ?? planOf(sub.product_id)!;
  if (d.blocked === 'cancelling') throw new HttpError(409, 'cancelling', 'This plan is set to cancel. Resume it first, then switch.');
  if (d.blocked === 'past_due') throw new HttpError(409, 'past_due', 'The last payment did not go through. Update your card first, then switch.');
  if (d.blocked) throw new HttpError(409, 'inactive', 'This subscription cannot be changed right now.');
  if (to.product === cur.product) throw new HttpError(409, 'same_plan', 'You are already on that plan.');
  const rule = changeRule(cur, to);
  const charge_now = rule.when === 'now' ? prorated(Number(s.amount ?? sub.amount), to, s.current_period_start ?? sub.current_period_start ?? null, s.current_period_end ?? sub.current_period_end) : 0;
  const preview = {
    from: planOut(cur), to: planOut(to), ...rule, currency: String(s.currency ?? 'usd'),
    charge_now_cents: charge_now, // an estimate before tax; Polar's invoice is exact
    next_charge: { date: s.current_period_end ?? sub.current_period_end, cents: to.cents },
    seats_end: cur.plan === 'team' && to.plan === 'pro',
  };
  return { sub, s, cur, to, rule, preview };
}

/** GET /api/billing/preview?plan=: what a switch charges and when, for the confirm step. */
async function previewChange(env: Env, u: Who, key: string) {
  return json({ ok: true, preview: (await targetFor(env, u, key)).preview });
}

/** POST /api/billing/change {plan}: switches the account's own subscription under changeRule. A
 *  pending change (a founder renewal or an earlier switch) is cleared first, so exactly one change
 *  is ever scheduled; if Polar then refuses the switch, the earlier one is put back. */
async function changePlan(req: Request, env: Env, ctx: Ctx, url: URL, u: Who) {
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
  const b = await body(req);
  const { sub, s, cur, to, rule, preview } = await targetFor(env, u, String(b.plan || ''));
  const path = `/subscriptions/${encodeURIComponent(sub.id)}`;
  const pending = s.pending_update?.product_id as string | undefined;
  if (rule.when === 'renewal' && pending === to.product) return json({ ok: true, preview, subscription: describe(s, sub, true) });
  if (pending) {
    const c = await polar(env, path, { method: 'PATCH', body: { pending_update: null } });
    if (c.status >= 300) throw new HttpError(502, 'billing_failed', 'Our billing provider did not accept the change. Please try again.');
  }
  const r = await polar(env, path, { method: 'PATCH', body: { product_id: to.product, proration_behavior: rule.proration } });
  if (r.status >= 300) {
    console.error('polar change', sub.id, r.status, JSON.stringify(r.data).slice(0, 300));
    if (pending) {
      if (cur.founder) await scheduleRegularRenewal(env, { id: sub.id }, cur);
      else await polar(env, path, { method: 'PATCH', body: { product_id: pending, proration_behavior: 'next_period' } }).catch(() => undefined);
    }
    throw new HttpError(502, 'billing_failed', r.status === 402 || /payment|card/i.test(JSON.stringify(r.data)) ? 'The payment for this switch did not go through. Check your card and try again.' : 'Our billing provider did not accept the change. Please try again.');
  }
  const row = await syncFromPolar(env, ctx, r.data, sub, u, rule.when === 'now' ? 'api.change' : 'api.schedule');
  return json({ ok: true, preview, subscription: describe(r.data.id ? r.data : s, row ?? sub, true) });
}

/** POST /api/billing/keep: drops a switch scheduled for the renewal. A founder subscription keeps
 *  (or gets back) its scheduled renewal as regular Pro; that one cannot be dropped. */
async function keepPlan(req: Request, env: Env, ctx: Ctx, url: URL, u: Who) {
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
  const { sub } = await ownSub(env, u);
  const s = await readSub(env, sub);
  const cur = planOf(s.product_id) ?? planOf(sub.product_id)!;
  const pending = s.pending_update?.product_id as string | undefined;
  const regular = renewsAs(cur);
  if (!pending || (cur.founder && pending === regular?.product)) return json({ ok: true, subscription: describe(s, sub, true) });
  const path = `/subscriptions/${encodeURIComponent(sub.id)}`;
  const r = await polar(env, path, { method: 'PATCH', body: { pending_update: null } });
  if (r.status >= 300) throw new HttpError(502, 'billing_failed', 'Our billing provider did not accept the change. Please try again.');
  if (cur.founder) await scheduleRegularRenewal(env, { id: sub.id }, cur);
  const fresh = await readSub(env, sub).catch(() => r.data);
  const row = await syncFromPolar(env, ctx, fresh, sub, u, 'api.keep');
  return json({ ok: true, subscription: describe(fresh, row ?? sub, true) });
}

/** The Polar customer ids of this account's own subscriptions (never a Team owner's). */
async function customerIds(env: Env, u: Who): Promise<string[]> {
  const r = await env.DB.prepare(
    'SELECT DISTINCT s.customer_id AS c FROM licences l JOIN subscriptions s ON s.id = l.subscription_id ' +
      "WHERE (l.user_id = ?1 OR (?3 = 1 AND l.email = ?2)) AND s.customer_id != '' LIMIT 5",
  ).bind(u.id, u.email, u.email_verified ? 1 : 0).all<{ c: string }>();
  return (r.results ?? []).map((x) => x.c);
}

/** GET /api/billing/invoices: the account's own orders (payments), newest first. */
async function listInvoices(env: Env, u: Who) {
  const ids = await customerIds(env, u);
  const out: Polar[] = [];
  for (const id of ids) {
    const r = await polar(env, `/orders/?customer_id=${encodeURIComponent(id)}&limit=50&sorting=-created_at`);
    for (const o of r.data.items ?? []) {
      if (o.customer_id !== id) continue; // only ever this customer's orders
      out.push({
        id: o.id, created_at: o.created_at, status: o.status, total_cents: o.total_amount, tax_cents: o.tax_amount, refunded_cents: o.refunded_amount ?? 0,
        currency: o.currency, description: o.description || o.product?.name || 'Oxyn', billing_reason: o.billing_reason,
        invoice_number: o.invoice_number ?? null, invoice_ready: !!o.is_invoice_generated, receipt: o.status !== 'draft' && o.status !== 'pending',
      });
    }
  }
  out.sort((a, b) => String(b.created_at).localeCompare(String(a.created_at)));
  return json({ ok: true, items: out.slice(0, 50) });
}

/** GET /api/billing/invoices/<id>/receipt|invoice: a short-lived download URL for one of the
 *  account's own orders. 202 {pending} while Polar is still making the PDF. */
async function invoiceFile(env: Env, u: Who, id: string, kind: 'receipt' | 'invoice') {
  const ids = await customerIds(env, u);
  const o = await polar(env, `/orders/${id}`);
  if (o.status !== 200 || !ids.includes(String(o.data.customer_id))) throw new HttpError(404, 'not_found', 'There is no such payment on this account.');
  const pdf = async (path: string) => {
    const r = await polar(env, path);
    if (r.status === 202) return json({ ok: false, pending: true, message: 'Polar is preparing this PDF. Try again in a few seconds.' }, 202);
    if (r.status === 200 && typeof r.data.url === 'string' && r.data.url.startsWith('https://')) return json({ ok: true, url: r.data.url });
    throw new HttpError(502, 'billing_failed', 'This PDF is not available right now. Please try again.');
  };
  if (kind === 'receipt') return pdf(`/orders/${id}/receipt`);
  if (o.data.is_invoice_generated) return pdf(`/orders/${id}/invoice`);
  const g = await polar(env, `/orders/${id}/invoice`, { method: 'POST' });
  if (g.status === 422) throw new HttpError(422, 'invoice_details', 'An invoice needs the billing name and address from the payment. This payment has none, so use the receipt.');
  if (g.status === 409) throw new HttpError(409, 'invoice_not_eligible', 'This payment cannot have an invoice yet. Use the receipt.');
  if (g.status >= 300) throw new HttpError(502, 'billing_failed', 'This invoice is not available right now. Please try again.');
  return json({ ok: false, pending: true, message: 'Your invoice is being made. Try again in a few seconds.' }, 202);
}

/** GET /api/billing/card: Polar's API cannot take a new card without Polar's own Stripe page, so
 *  this is the one step that leaves the site: a pre-signed customer session on Polar's page (plan
 *  changes are switched off there), with a way back to the account page. */
async function updateCard(env: Env, url: URL, u: Who) {
  const back = `${url.origin}/account/#billing`;
  const e = await plan_for(env, u);
  const customer = e.mine?.customer_id ?? (e.seat ? null : e.customer);
  if (!customer || !env.POLAR_ACCESS_TOKEN) return redirect(`${url.origin}/account/?billing=card_unavailable#billing`);
  try {
    const r = await polar(env, '/customer-sessions/', { method: 'POST', body: { customer_id: customer, return_url: back } });
    const to = r.data.customer_portal_url;
    if (r.status < 300 && typeof to === 'string' && to.startsWith('https://')) return redirect(to);
    console.error('polar customer session', r.status, JSON.stringify(r.data).slice(0, 200));
  } catch (err) { console.error('polar customer session', err); }
  return redirect(`${url.origin}/account/?billing=card_unavailable#billing`);
}

/** Which Polar API scopes the Worker's token has, by harmless calls (reads, and writes aimed at an
 *  id that does not exist: 404/422 means the scope is there, 401/403 that it is not). Kept in D1
 *  settings `polar_check` for the admin panel; rerun at most once a day (from /api/billing/plans). */
export async function polarCheck(env: Env, force = false) {
  if (!env.POLAR_ACCESS_TOKEN) return null;
  const prev = await env.DB.prepare("SELECT updated_at FROM settings WHERE key = 'polar_check'").first<{ updated_at: string }>();
  if (!force && prev && Date.now() - Date.parse(prev.updated_at) < 86_400_000) return null;
  // Claim the run first so concurrent requests do not repeat it.
  await env.DB.prepare("INSERT INTO settings (key, value, updated_at) VALUES ('polar_check', '{\"running\":true}', ?1) ON CONFLICT (key) DO UPDATE SET value = '{\"running\":true}', updated_at = ?1").bind(iso()).run();
  const zero = '00000000-0000-4000-8000-000000000000';
  const probes: [string, string, string, unknown?][] = [
    ['subscriptions_read', 'GET', '/subscriptions/?limit=1'],
    ['subscriptions_write', 'PATCH', `/subscriptions/${zero}`, { cancel_at_period_end: false }],
    ['orders_read', 'GET', '/orders/?limit=1'],
    ['customers_read', 'GET', `/customers/${zero}/payment-methods`],
    ['customer_sessions_write', 'POST', '/customer-sessions/', { customer_id: zero }],
    ['checkouts_write', 'POST', '/checkouts/', { products: [zero] }],
  ];
  const out: Record<string, string> = {};
  for (const [name, method, path, b] of probes) {
    try {
      const r = await fetch(`${POLAR.api}${path}`, { method, headers: polarHeaders(env), body: b === undefined ? undefined : JSON.stringify(b) });
      out[name] = r.status === 401 ? 'invalid_token' : r.status === 403 ? 'missing_scope' : [200, 404, 422].includes(r.status) ? 'ok' : `http_${r.status}`;
    } catch { out[name] = 'unreachable'; }
  }
  await env.DB.prepare("UPDATE settings SET value = ?1, updated_at = ?2 WHERE key = 'polar_check'").bind(JSON.stringify(out), iso()).run();
  return out;
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
    // One founder spot per new subscriber. Someone who subscribed before (a direct Polar link gets
    // past the checkout check) takes no spot; their subscription still renews at the regular price.
    const customerId = String(d.customer_id || d.customer?.id || '');
    const before = (await hadSubscription(env, userId, email, String(d.id))) ||
      (!!customerId && !!(await env.DB.prepare('SELECT 1 AS one FROM subscriptions WHERE customer_id = ?1 AND id != ?2 LIMIT 1').bind(customerId, String(d.id)).first()));
    const claimed = await env.DB.prepare('SELECT 1 AS one FROM founder_claims WHERE subscription_id = ?1').bind(String(d.id)).first();
    if (!before || claimed) {
      await env.DB.prepare('INSERT OR IGNORE INTO founder_claims (subscription_id, email, claimed_at) VALUES (?1, ?2, ?3)').bind(String(d.id), email, now).run();
    } else await flagFounderRepeat(env, String(d.id), email, customerId, plan);
    await scheduleRegularRenewal(env, d, plan);
  }
  const f = await founderState(env);
  if (!f.open) ctx.waitUntil(archiveFounder(env).catch((e) => console.error('founder archive', e)));
}

/** Backstop for the founder checkout gate: a founder subscription for a customer who already had an
 *  Oxyn subscription (say a checkout made outside the site) is written once to the audit log as
 *  `billing.founder_repeat`, which the admin Activity tab lists under "Needs a look". Nothing is
 *  changed on the subscription: an admin decides what to do with it. */
async function flagFounderRepeat(env: Env, id: string, email: string, customerId: string, plan: Plan) {
  console.log('polar webhook: founder subscription from a returning customer, no founder spot taken', id);
  const seen = await env.DB.prepare("SELECT 1 AS one FROM audit_log WHERE action = 'billing.founder_repeat' AND target = ?1 LIMIT 1").bind(id).first();
  if (seen) return;
  await env.DB.prepare('INSERT INTO audit_log (at, actor, action, target, detail) VALUES (?1, ?2, ?3, ?4, ?5)')
    .bind(iso(), 'polar-webhook', 'billing.founder_repeat', id, JSON.stringify({ email, customer_id: customerId || null, plan: plan.key })).run();
}

const polarHeaders = (env: Env) => ({ authorization: `Bearer ${env.POLAR_ACCESS_TOKEN}`, 'content-type': 'application/json', accept: 'application/json' });
const renewalKey = (id: string) => `founder_renewal:${id}`;

/** Founder rate = intro price: makes a founder subscription renew as the matching regular Pro
 *  product (Founder Monthly -> Pro Monthly, Founder Yearly -> Pro Yearly) with
 *  proration_behavior "next_period", so nothing is charged now and the next renewal bills the
 *  regular price. Idempotent: skipped when Polar already has that change pending (the PATCH itself
 *  fires a subscription.updated webhook that lands here again). Never throws: the outcome is kept
 *  in D1 settings (`founder_renewal:<id>`) for the admin panel, and later webhooks for the same
 *  subscription (or Retry in the admin panel) try again. Exported for the tests. */
export async function scheduleRegularRenewal(env: Env, d: Record<string, any>, plan: Plan): Promise<string> {
  const target = renewsAs(plan);
  const id = String(d.id || '');
  if (!target || !id) return 'not_founder';
  const save = async (v: string) => {
    await env.DB.prepare('INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT (key) DO UPDATE SET value = ?2, updated_at = ?3')
      .bind(renewalKey(id), v, iso()).run();
    return v;
  };
  try {
    if (d.pending_update?.product_id === target.product) return await save('scheduled');
    if (!env.POLAR_ACCESS_TOKEN) return await save('pending: set the POLAR_ACCESS_TOKEN Worker secret');
    // The webhook payload can be stale: read the subscription as Polar has it now.
    const g = await fetch(`${POLAR.api}/subscriptions/${encodeURIComponent(id)}`, { headers: polarHeaders(env) });
    if (!g.ok) return await save(`error: read ${g.status}`);
    const s = (await g.json()) as Record<string, any>;
    const pending = s.pending_update?.product_id as string | undefined;
    if (s.product_id !== plan.product) return await save('done: no longer on the founder product');
    if (pending === target.product) return await save('scheduled');
    if (!ACTIVE.includes(String(s.status)) || s.ended_at) return await save(`skipped: ${s.status}`);
    if (s.cancel_at_period_end) return await save('waiting: cancels at the period end');
    // A change the customer scheduled to another regular plan (say Team) already ends the founder price.
    if (pending && !byProduct.get(pending)?.founder) return await save('done: another plan change is scheduled');
    const r = await fetch(`${POLAR.api}/subscriptions/${encodeURIComponent(id)}`, {
      method: 'PATCH',
      headers: polarHeaders(env),
      body: JSON.stringify({ product_id: target.product, proration_behavior: 'next_period' }),
    });
    if (!r.ok) {
      console.error('founder renewal', id, r.status, (await r.text()).slice(0, 300));
      return await save(`error: ${r.status}`);
    }
    return await save('scheduled');
  } catch (err) {
    console.error('founder renewal', id, err);
    try { return await save('error: request failed'); } catch { return 'error'; }
  }
}

/** POST /api/admin/billing/founder-renewals: schedules the regular-price renewal on every live
 *  founder subscription that does not have it yet (after the token was missing or Polar failed). */
export async function adminFounderRenewals(env: Env) {
  const subs = (await env.DB.prepare(
    `SELECT s.id, s.product_id, st.value AS renewal FROM subscriptions s LEFT JOIN settings st ON st.key = 'founder_renewal:' || s.id ` +
      `WHERE s.founder = 1 AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL`,
  ).all<{ id: string; product_id: string; renewal: string | null }>()).results ?? [];
  const out: { id: string; result: string }[] = [];
  for (const s of subs) {
    const plan = byProduct.get(s.product_id);
    if (!plan?.founder || s.renewal === 'scheduled') continue;
    out.push({ id: s.id, result: await scheduleRegularRenewal(env, { id: s.id }, plan) });
  }
  return json({ ok: true, checked: subs.length, results: out });
}

/** At 100 founder subscriptions: archive both founder products in Polar so no new founder checkout
 *  can start (existing founder subscriptions are unaffected: each renews as regular Pro after its
 *  first period, see scheduleRegularRenewal). Runs once; retried on
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
      headers: polarHeaders(env),
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
      "(SELECT COUNT(*) FROM team_seats t WHERE t.subscription_id = s.id AND t.removed_at IS NULL) AS seats, " +
      "(SELECT value FROM settings WHERE key = 'founder_renewal:' || s.id) AS renewal " +
      'FROM subscriptions s ORDER BY s.created_at DESC LIMIT 500',
  ).all<SubRow & { linked: number; seats: number; renewal: string | null }>()).results ?? [];
  let mrr = 0;
  const counts = { active: 0, pro: 0, team: 0, founder: 0, canceling: 0, ended: 0, renewal_unscheduled: 0 };
  for (const s of rows) {
    if (live(s)) {
      counts.active++;
      counts[s.plan]++;
      if (s.founder) counts.founder++;
      if (s.founder && !s.cancel_at_period_end && s.renewal !== 'scheduled') counts.renewal_unscheduled++;
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
    polar_check: await env.DB.prepare("SELECT value, updated_at FROM settings WHERE key = 'polar_check'").first<{ value: string; updated_at: string }>(),
    items: rows.map((s) => ({ ...s, active: live(s) })),
  };
}
