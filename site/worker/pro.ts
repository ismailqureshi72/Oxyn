// The Pro registry (P3): CLI tokens, `oxyn login`, and the monthly Pro unlock caps. See docs/cli.md
// ("Pro items") and docs/pricing.md ("Monthly caps").
//
//   GET    /api/r/<slug>.json            Bearer token: a Pro item's registry JSON (counts one unlock);
//                                        a free item is served too (no token needed).
//                                        ?for=<slug>: a dependency of an item this pool already
//                                        unlocked this month, served without counting.
//   GET    /api/cli/whoami               Bearer token: account, plan and this month's usage
//   GET    /api/cli/tokens               signed in: the account's CLI tokens (never the tokens)
//   POST   /api/cli/tokens               signed in: {name} makes a token, shown once
//   DELETE /api/cli/tokens/<id>          signed in: revokes one
//   POST   /api/cli/auth/start           the CLI: {client} starts `oxyn login`
//   GET    /api/cli/auth/request?code=   signed in: what /cli/auth/ shows before approving
//   POST   /api/cli/auth/approve         signed in, from the site: {code, approve}
//   POST   /api/cli/auth/poll            the CLI: {device_code} returns the token once approved
//
// Pro item JSON is never a public asset: the build writes it to dist/_pro/r/ (site/public/_pro/r),
// and wrangler.jsonc runs this Worker first for /_pro/*, which answers 404 to every outside request.
// Only this file reads it, through the ASSETS binding.
//
// Caps (decided 2026-11-21): Pro and Founder open 20 Pro components and 8 Pro screens per billing
// month (blocks and agent apps count as screens); a Team shares 75 components and 22 screens across
// its people; Free opens none; admins are unlimited. Each distinct item counts once per month: a
// re-download or an update of an item the pool already opened is free. The site's Copy / unlock of
// Pro code goes through the same unlock() as the CLI and MCP, so they share one count.

import { HttpError, fail, json, sameOrigin, sha256Hex, type Ctx, type Env } from './shared';
import { currentUser } from './auth';
import { plan_for, type PlanFor, type Who } from './billing';

export const CAPS = {
  pro: { component: 20, screen: 8 },
  team: { component: 75, screen: 22 },
} as const;
export type Bucket = 'component' | 'screen';
export type Via = 'cli' | 'mcp' | 'site';

const TOKEN = { prefix: 'oxyn_', maxPerUser: 20, nameMax: 60 };
const DEVICE = { ttl: 600, interval: 3 };
const iso = () => new Date().toISOString();
const SLUG = /^[a-z0-9][a-z0-9-]{0,79}$/;

// ── Billing month ─────────────────────────────────────────────────────────────────────────────
const daysIn = (y: number, m: number) => new Date(Date.UTC(y, m + 1, 0)).getUTCDate();
function addMonths(a: Date, k: number): Date {
  const y = a.getUTCFullYear(), m = a.getUTCMonth() + k;
  const ty = y + Math.floor(m / 12), tm = ((m % 12) + 12) % 12;
  return new Date(Date.UTC(ty, tm, Math.min(a.getUTCDate(), daysIn(ty, tm)), a.getUTCHours(), a.getUTCMinutes(), a.getUTCSeconds()));
}

/** The billing month that holds `now`: monthly steps from `anchor` (a Polar period start or end,
 *  so yearly plans also get monthly caps on their billing day), else the calendar month (UTC). */
export function billingMonth(anchor: string | null | undefined, now = new Date()): { start: string; end: string } {
  const a = anchor ? new Date(anchor) : null;
  if (!a || Number.isNaN(a.getTime())) {
    const s = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
    return { start: s.toISOString(), end: new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth() + 1, 1)).toISOString() };
  }
  let k = (now.getUTCFullYear() - a.getUTCFullYear()) * 12 + (now.getUTCMonth() - a.getUTCMonth());
  while (addMonths(a, k) > now) k--;
  while (addMonths(a, k + 1) <= now) k++;
  return { start: addMonths(a, k).toISOString(), end: addMonths(a, k + 1).toISOString() };
}

// ── Quota ─────────────────────────────────────────────────────────────────────────────────────
export interface Quota {
  pool: string;
  plan: 'free' | 'pro' | 'team' | 'admin';
  unlimited: boolean;
  pooled: boolean; // shared by a Team
  limits: Record<Bucket, number>;
  period: { start: string; end: string };
}

/** Who shares the count and which month it is in, from plan_for(). */
export function quotaFor(e: PlanFor, userId: string, now = new Date()): Quota {
  const own = (anchor?: string | null) => billingMonth(anchor, now);
  if (e.admin) return { pool: `user:${userId}`, plan: 'admin', unlimited: true, pooled: false, limits: { ...CAPS.team }, period: own(null) };
  if (!e.pro || e.tier === 'free') return { pool: `user:${userId}`, plan: 'free', unlimited: false, pooled: false, limits: { component: 0, screen: 0 }, period: own(null) };
  const anchorOf = (s: { current_period_start?: string | null; current_period_end: string | null } | null) => s?.current_period_start || s?.current_period_end || null;
  if (e.tier === 'team') {
    if (e.via === 'subscription' && e.mine) return { pool: `team:${e.mine.id}`, plan: 'team', unlimited: false, pooled: true, limits: { ...CAPS.team }, period: own(anchorOf(e.mine)) };
    if (e.via === 'seat' && e.seat) return { pool: `team:${e.seat.id}`, plan: 'team', unlimited: false, pooled: true, limits: { ...CAPS.team }, period: own(anchorOf(e.seat)) };
    if (e.grant) return { pool: `grant:${e.grant.parent_id ?? e.grant.id}`, plan: 'team', unlimited: false, pooled: true, limits: { ...CAPS.team }, period: own(null) };
  }
  const anchor = e.via === 'subscription' ? anchorOf(e.mine) : null;
  return { pool: `user:${userId}`, plan: 'pro', unlimited: false, pooled: false, limits: { ...CAPS.pro }, period: own(anchor) };
}

export const bucketOf = (kind: string | undefined): Bucket => (kind === 'component' || kind === 'style' || !kind ? 'component' : 'screen');

export interface Usage {
  plan: Quota['plan'];
  unlimited: boolean;
  pooled: boolean;
  components: { used: number; limit: number | null };
  screens: { used: number; limit: number | null };
  period_start: string;
  resets_at: string;
  summary: string;
}

const day = (s: string) => s.slice(0, 10);
const plural = (n: number, w: string) => `${n} ${w}${n === 1 ? '' : 's'}`;

export function usageView(q: Quota, used: Record<Bucket, number>): Usage {
  const lim = (b: Bucket) => (q.unlimited ? null : q.limits[b]);
  const part = (b: Bucket, w: string) => (q.unlimited ? plural(used[b], w) : `${used[b]} of ${plural(q.limits[b], w)}`);
  const summary = q.unlimited
    ? `Unlimited (admin): ${part('component', 'component')}, ${part('screen', 'screen')} this month`
    : q.plan === 'free'
      ? 'Free plan: Pro items need Oxyn Pro'
      : `${part('component', 'component')}, ${part('screen', 'screen')}${q.pooled ? ' (shared by your Team)' : ''}, resets ${day(q.period.end)}`;
  return {
    plan: q.plan, unlimited: q.unlimited, pooled: q.pooled,
    components: { used: used.component, limit: lim('component') },
    screens: { used: used.screen, limit: lim('screen') },
    period_start: q.period.start, resets_at: q.period.end, summary,
  };
}

export async function countUsed(env: Env, q: Quota): Promise<Record<Bucket, number>> {
  const r = await env.DB.prepare('SELECT kind, COUNT(*) AS n FROM pro_unlocks WHERE pool = ?1 AND period_start = ?2 AND reset_at IS NULL GROUP BY kind')
    .bind(q.pool, q.period.start).all<{ kind: Bucket; n: number }>();
  const used = { component: 0, screen: 0 };
  for (const row of r.results ?? []) used[row.kind] = Number(row.n);
  return used;
}

export async function usageFor(env: Env, u: Who, e?: PlanFor): Promise<Usage> {
  const q = quotaFor(e ?? (await plan_for(env, u)), u.id);
  return usageView(q, await countUsed(env, q));
}

/** A 403 with this month's usage in the body. */
export class QuotaError extends HttpError {
  constructor(code: string, message: string, public usage: Usage) { super(403, code, message); }
}
export const failQuota = (e: QuotaError) =>
  json({ ok: false, error: { code: e.code, message: e.message }, usage: e.usage }, 403);

const already = (env: Env, q: Quota, slug: string) =>
  env.DB.prepare('SELECT 1 AS one FROM pro_unlocks WHERE pool = ?1 AND period_start = ?2 AND slug = ?3 AND reset_at IS NULL').bind(q.pool, q.period.start, slug).first();

/** Opens one Pro item for this account: free when its pool already opened it this month, else it
 *  takes one of the month's components or screens (refused at the cap). The single counting path for
 *  the CLI, the MCP server and the site. `counted` is false for a re-download. */
export async function unlock(env: Env, u: Who, slug: string, bucket: Bucket, via: Via, e?: PlanFor): Promise<{ counted: boolean; usage: Usage }> {
  const ent = e ?? (await plan_for(env, u));
  const q = quotaFor(ent, u.id);
  if (!ent.pro) {
    const used = await countUsed(env, q);
    throw new QuotaError('pro_required', 'This item is part of Oxyn Pro. See the plans at /pricing/.', usageView(q, used));
  }
  if (await already(env, q, slug)) return { counted: false, usage: usageView(q, await countUsed(env, q)) };
  const now = iso();
  const cap = q.unlimited ? 1_000_000_000 : q.limits[bucket];
  const r = await env.DB.prepare(
    'INSERT OR IGNORE INTO pro_unlocks (pool, period_start, slug, kind, user_id, via, created_at) ' +
      'SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7 WHERE (SELECT COUNT(*) FROM pro_unlocks WHERE pool = ?1 AND period_start = ?2 AND kind = ?4 AND reset_at IS NULL) < ?8',
  ).bind(q.pool, q.period.start, slug, bucket, u.id, via, now, cap).run();
  const used = await countUsed(env, q);
  if (Number(r.meta?.changes ?? 0) > 0) return { counted: true, usage: usageView(q, used) };
  if (await already(env, q, slug)) return { counted: false, usage: usageView(q, used) }; // a parallel request opened it
  const usage = usageView(q, used);
  const what = bucket === 'component' ? 'components' : 'screens';
  throw new QuotaError(
    'quota_exceeded',
    `You have opened ${used[bucket]} of ${q.limits[bucket]} Pro ${what} this billing month${q.pooled ? ' (shared by your Team)' : ''}. ` +
      `Items you already opened this month stay free to download again. The count resets on ${day(q.period.end)}.`,
    usage,
  );
}

// ── Registry index (kinds and dependencies) ───────────────────────────────────────────────────
interface IdxEntry { slug: string; kind: string; tier: 'free' | 'pro'; installable: boolean; oxyn_deps?: Record<string, string[]> }
let IDX: { at: number; items: Map<string, IdxEntry> } | null = null;

/** The public index (dist/r/index.json), cached per isolate for 5 minutes. Kinds come from here. */
export async function registryIndex(env: Env, origin: string): Promise<Map<string, IdxEntry>> {
  if (IDX && Date.now() - IDX.at < 300_000) return IDX.items;
  const r = await env.ASSETS.fetch(new Request(`${origin}/r/index.json`));
  if (!r.ok) throw new HttpError(503, 'registry_unavailable', 'The registry index is not available right now.');
  const d = (await r.json()) as { items: IdxEntry[] };
  IDX = { at: Date.now(), items: new Map(d.items.map((i) => [i.slug, i])) };
  return IDX.items;
}
export const resetIndexCache = () => { IDX = null; };

/** Every item `slug` needs, in any framework (transitively). */
export function depClosure(idx: Map<string, IdxEntry>, slug: string): Set<string> {
  const out = new Set<string>();
  const todo = [slug];
  while (todo.length) {
    const s = todo.pop()!;
    for (const list of Object.values(idx.get(s)?.oxyn_deps ?? {})) for (const d of list) if (!out.has(d)) { out.add(d); todo.push(d); }
  }
  return out;
}

// ── Tokens ────────────────────────────────────────────────────────────────────────────────────
function randomToken() {
  const b = crypto.getRandomValues(new Uint8Array(24));
  return TOKEN.prefix + [...b].map((x) => x.toString(16).padStart(2, '0')).join('');
}

/** The account behind `Authorization: Bearer oxyn_…`, or null. Touches last_used_at. */
export async function tokenUser(req: Request, env: Env, ctx?: Ctx): Promise<(Who & { token_id: string }) | null> {
  const m = /^Bearer\s+(\S+)$/i.exec(req.headers.get('authorization') || '');
  if (!m || !m[1].startsWith(TOKEN.prefix) || m[1].length > 100) return null;
  const row = await env.DB.prepare(
    'SELECT u.id, u.email, u.email_verified, u.name, t.id AS token_id, t.last_used_at FROM cli_tokens t JOIN users u ON u.id = t.user_id WHERE t.token_hash = ?1 AND t.revoked_at IS NULL',
  ).bind(await sha256Hex(m[1])).first<Who & { token_id: string; name: string | null; last_used_at: string | null }>();
  if (!row) return null;
  // Write at most once a minute per token.
  if (!row.last_used_at || Date.now() - Date.parse(row.last_used_at) > 60_000) {
    const p = env.DB.prepare('UPDATE cli_tokens SET last_used_at = ?2 WHERE id = ?1').bind(row.token_id, iso()).run();
    if (ctx) ctx.waitUntil(p); else await p;
  }
  return row;
}

export async function createToken(env: Env, userId: string, name: string) {
  const n = await env.DB.prepare('SELECT COUNT(*) AS n FROM cli_tokens WHERE user_id = ?1 AND revoked_at IS NULL').bind(userId).first<{ n: number }>();
  if (Number(n?.n ?? 0) >= TOKEN.maxPerUser) throw new HttpError(409, 'too_many_tokens', `You have ${TOKEN.maxPerUser} tokens. Revoke one you no longer use first.`);
  const token = randomToken();
  const id = crypto.randomUUID();
  const created = iso();
  const clean = name.replace(/[\u0000-\u001f]/g, '').trim().slice(0, TOKEN.nameMax) || 'CLI token';
  await env.DB.prepare('INSERT INTO cli_tokens (id, user_id, token_hash, name, hint, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)')
    .bind(id, userId, await sha256Hex(token), clean, token.slice(-4), created).run();
  return { id, name: clean, hint: token.slice(-4), created_at: created, token };
}

// ── Routes ────────────────────────────────────────────────────────────────────────────────────
export const isProPath = (p: string) => p.startsWith('/api/r/') || p.startsWith('/api/cli/');

async function readJson(req: Request): Promise<Record<string, unknown>> {
  if (!(req.headers.get('content-type') || '').startsWith('application/json')) throw new HttpError(415, 'unsupported_media_type', 'Send application/json.');
  if (Number(req.headers.get('content-length') || '0') > 4096) throw new HttpError(413, 'too_large', 'That request is too large.');
  try { return (await req.json()) as Record<string, unknown>; } catch { throw new HttpError(400, 'bad_body', 'The request could not be read.'); }
}

const viaOf = (req: Request, url: URL): Via => {
  const v = url.searchParams.get('via') || req.headers.get('x-oxyn-client') || '';
  return /mcp/i.test(v) ? 'mcp' : 'cli';
};

const SIGNED_OUT = () => new HttpError(401, 'unauthorized', 'Sign in first: run `oxyn login`, or set OXYN_TOKEN to a token from your account page.');

export async function handlePro(req: Request, env: Env, ctx: Ctx, url: URL): Promise<Response> {
  const route = url.pathname.replace(/\/+$/, '');
  const m = req.method;
  try {
    const rm = route.match(/^\/api\/r\/([a-z0-9-]{1,80})\.json$/);
    if (rm) {
      if (m !== 'GET' && m !== 'HEAD') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
      return await registryItem(req, env, ctx, url, rm[1]);
    }
    if (route === '/api/cli/whoami' && m === 'GET') {
      const u = await tokenUser(req, env, ctx);
      if (!u) throw SIGNED_OUT();
      const e = await plan_for(env, u);
      return json({ ok: true, email: u.email, tier: e.tier, pro: e.pro, admin: e.admin, via: e.via, usage: await usageFor(env, u, e) });
    }
    if (route === '/api/cli/auth/start' && m === 'POST') return await authStart(req, env, url);
    if (route === '/api/cli/auth/poll' && m === 'POST') return await authPoll(req, env);
    const u = await currentUser(req, env);
    if (!u) throw new HttpError(401, 'signed_out', 'Sign in to Oxyn first.');
    if (route === '/api/cli/auth/request' && m === 'GET') return await authRequest(env, url);
    if (route === '/api/cli/auth/approve' && m === 'POST') {
      if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
      return await authApprove(req, env, u);
    }
    if (route === '/api/cli/tokens' && m === 'GET') {
      const r = await env.DB.prepare('SELECT id, name, hint, created_at, last_used_at FROM cli_tokens WHERE user_id = ?1 AND revoked_at IS NULL ORDER BY created_at DESC').bind(u.id).all();
      return json({ ok: true, tokens: r.results ?? [] });
    }
    if (route === '/api/cli/tokens' && m === 'POST') {
      if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
      const b = await readJson(req);
      return json({ ok: true, ...(await createToken(env, u.id, String(b.name || ''))) }, 201);
    }
    const tm = route.match(/^\/api\/cli\/tokens\/([0-9a-f-]{36})$/);
    if (tm && m === 'DELETE') {
      if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
      const r = await env.DB.prepare('UPDATE cli_tokens SET revoked_at = ?3 WHERE id = ?1 AND user_id = ?2 AND revoked_at IS NULL').bind(tm[1], u.id, iso()).run();
      if (!Number(r.meta?.changes ?? 0)) throw new HttpError(404, 'not_found', 'No such token.');
      return json({ ok: true });
    }
    throw new HttpError(404, 'not_found', 'No such API route.');
  } catch (err) {
    if (err instanceof QuotaError) return failQuota(err);
    if (err instanceof HttpError) return fail(err);
    console.error('pro error', err);
    return fail(new HttpError(500, 'internal', 'Something went wrong on our side. Please try again.'));
  }
}

async function registryItem(req: Request, env: Env, ctx: Ctx, url: URL, slug: string): Promise<Response> {
  const idx = await registryIndex(env, url.origin);
  const entry = idx.get(slug);
  if (!entry) throw new HttpError(404, 'not_found', `There is no item called \`${slug}\`.`);
  if (entry.tier !== 'pro') {
    const r = await env.ASSETS.fetch(new Request(`${url.origin}/r/${slug}.json`));
    if (!r.ok) throw new HttpError(404, 'not_found', `\`${slug}\` has no installable build.`);
    return new Response(r.body, { status: 200, headers: { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'public, max-age=300' } });
  }
  const u = await tokenUser(req, env, ctx);
  if (!u) throw SIGNED_OUT();
  if (!entry.installable) throw new HttpError(404, 'not_installable', `\`${slug}\` is listed but has no installable build yet. Use the code on its page.`);
  const e = await plan_for(env, u);
  const via = viaOf(req, url);
  // A dependency of an item this pool already opened this month comes with it.
  const parent = url.searchParams.get('for') || '';
  let usage: Usage;
  let counted = false;
  if (parent && parent !== slug && SLUG.test(parent) && e.pro && depClosure(idx, parent).has(slug) &&
      (await already(env, quotaFor(e, u.id), parent))) {
    usage = await usageFor(env, u, e);
  } else {
    ({ usage, counted } = await unlock(env, u, slug, bucketOf(entry.kind), via, e));
  }
  const r = await env.ASSETS.fetch(new Request(`${url.origin}/_pro/r/${slug}.json`));
  if (!r.ok) throw new HttpError(503, 'registry_unavailable', 'That Pro item is not available right now. Please try again in a minute.');
  return new Response(r.body, {
    status: 200,
    headers: {
      'content-type': 'application/json; charset=utf-8', 'cache-control': 'private, no-store', 'x-content-type-options': 'nosniff',
      'x-oxyn-counted': counted ? '1' : '0', 'x-oxyn-usage': usage.summary.replace(/[^\x20-\x7e]/g, ''),
    },
  });
}

// ── `oxyn login` (device flow) ────────────────────────────────────────────────────────────────
const CODE_ALPHABET = 'BCDFGHJKLMNPQRSTVWXZ'; // no vowels: no words, no 0/O 1/I mix-ups
function userCode() {
  const b = crypto.getRandomValues(new Uint8Array(8));
  const c = [...b].map((x) => CODE_ALPHABET[x % CODE_ALPHABET.length]).join('');
  return `${c.slice(0, 4)}-${c.slice(4)}`;
}

async function authStart(req: Request, env: Env, url: URL) {
  const b = await readJson(req);
  const client = String(b.client || '').replace(/[^\w .@-]/g, '').slice(0, 60) || null;
  const now = Math.floor(Date.now() / 1000);
  await env.DB.prepare('DELETE FROM cli_auth_requests WHERE expires_at < ?1').bind(now).run();
  const device = randomToken().replace(TOKEN.prefix, 'dev_');
  const code = userCode();
  await env.DB.prepare('INSERT INTO cli_auth_requests (device_hash, user_code, client_name, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)')
    .bind(await sha256Hex(device), code, client, now, now + DEVICE.ttl).run();
  return json({ ok: true, device_code: device, user_code: code, verification_url: `${url.origin}/cli/auth/?code=${code}`, interval: DEVICE.interval, expires_in: DEVICE.ttl });
}

const CODE_RE = /^[A-Z]{4}-[A-Z]{4}$/;

async function authRequest(env: Env, url: URL) {
  const code = (url.searchParams.get('code') || '').toUpperCase();
  if (!CODE_RE.test(code)) throw new HttpError(400, 'bad_code', 'That code does not look right.');
  const r = await env.DB.prepare('SELECT client_name, user_id, expires_at, denied FROM cli_auth_requests WHERE user_code = ?1').bind(code).first<{ client_name: string | null; user_id: string | null; expires_at: number; denied: number }>();
  if (!r || r.expires_at < Date.now() / 1000) throw new HttpError(404, 'expired', 'This sign-in request has expired. Run `oxyn login` again.');
  return json({ ok: true, code, client: r.client_name, approved: !!r.user_id, denied: !!r.denied });
}

async function authApprove(req: Request, env: Env, u: Who) {
  const b = await readJson(req);
  const code = String(b.code || '').toUpperCase();
  if (!CODE_RE.test(code)) throw new HttpError(400, 'bad_code', 'That code does not look right.');
  const now = Math.floor(Date.now() / 1000);
  const r = b.approve === false
    ? await env.DB.prepare('UPDATE cli_auth_requests SET denied = 1 WHERE user_code = ?1 AND user_id IS NULL AND expires_at >= ?2').bind(code, now).run()
    : await env.DB.prepare('UPDATE cli_auth_requests SET user_id = ?2 WHERE user_code = ?1 AND user_id IS NULL AND denied = 0 AND expires_at >= ?3').bind(code, u.id, now).run();
  if (!Number(r.meta?.changes ?? 0)) throw new HttpError(404, 'expired', 'This sign-in request has expired or was already answered. Run `oxyn login` again.');
  return json({ ok: true });
}

async function authPoll(req: Request, env: Env) {
  const b = await readJson(req);
  const device = String(b.device_code || '');
  if (!/^dev_[0-9a-f]{48}$/.test(device)) throw new HttpError(400, 'bad_code', 'Bad device code.');
  const h = await sha256Hex(device);
  const r = await env.DB.prepare('SELECT user_id, client_name, expires_at, denied FROM cli_auth_requests WHERE device_hash = ?1').bind(h).first<{ user_id: string | null; client_name: string | null; expires_at: number; denied: number }>();
  if (!r || r.expires_at < Date.now() / 1000) throw new HttpError(410, 'expired', 'The sign-in request expired. Run `oxyn login` again.');
  if (r.denied) {
    await env.DB.prepare('DELETE FROM cli_auth_requests WHERE device_hash = ?1').bind(h).run();
    throw new HttpError(403, 'denied', 'The sign-in request was declined in the browser.');
  }
  if (!r.user_id) return json({ ok: true, status: 'pending' });
  // One token, once: the row goes as the token is made.
  const del = await env.DB.prepare('DELETE FROM cli_auth_requests WHERE device_hash = ?1 AND user_id IS NOT NULL').bind(h).run();
  if (!Number(del.meta?.changes ?? 0)) throw new HttpError(410, 'expired', 'The sign-in request was already used. Run `oxyn login` again.');
  const t = await createToken(env, r.user_id, `oxyn login${r.client_name ? ` on ${r.client_name}` : ''}`);
  const user = await env.DB.prepare('SELECT email FROM users WHERE id = ?1').bind(r.user_id).first<{ email: string }>();
  return json({ ok: true, status: 'approved', token: t.token, email: user?.email ?? null });
}

// ── Admin ─────────────────────────────────────────────────────────────────────────────────────
/** This month's usage of one account (for the admin user page). */
export async function adminUsage(env: Env, u: Who) {
  const e = await plan_for(env, u);
  const q = quotaFor(e, u.id);
  const items = (await env.DB.prepare('SELECT slug, kind, via, user_id, created_at FROM pro_unlocks WHERE pool = ?1 AND period_start = ?2 AND reset_at IS NULL ORDER BY created_at DESC LIMIT 200')
    .bind(q.pool, q.period.start).all()).results ?? [];
  return { pool: q.pool, ...usageView(q, await countUsed(env, q)), items };
}

/** Frees this month's count of the account's pool (its Team's, for a Team). */
export async function adminResetUsage(env: Env, admin: string, u: Who) {
  const q = quotaFor(await plan_for(env, u), u.id);
  const r = await env.DB.prepare('UPDATE pro_unlocks SET reset_at = ?3, reset_by = ?4 WHERE pool = ?1 AND period_start = ?2 AND reset_at IS NULL')
    .bind(q.pool, q.period.start, iso(), admin).run();
  return { pool: q.pool, cleared: Number(r.meta?.changes ?? 0) };
}
