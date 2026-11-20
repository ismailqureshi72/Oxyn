// Privacy-friendly site events (G2), for the admin Activity tab. See docs/admin.md#activity.
//
//   POST /api/events   from the site's own pages (public/site.js batches them): {events: [{type, path, item?, detail?}]}
//
// What is stored: the event type, the page path (no query string), a component / screen slug or a
// plan key, the account id when the visitor is signed in (read from the session cookie, never from
// the body), and `visitor`: a salted hash of IP + user agent + the UTC day, so a visitor can be
// counted once a day but not followed from one day to the next. No IP, user agent, email or
// referrer is stored. Bots and headless browsers are dropped, Do Not Track / Global Privacy Control
// are honoured by the client, each IP is rate limited, and rows older than 90 days are deleted.

import { HttpError, json, sameOrigin, sha256Hex, type Env } from './shared';
import { currentUser } from './auth';

export const EVENTS = {
  maxBatch: 20, // events per request
  maxBody: 8_000, // bytes
  // fixed windows per hashed IP: requests (each up to maxBatch events)
  burst: { window: 60, max: 30 },
  hourly: { window: 3600, max: 300 },
  retentionDays: 90,
};
export const EVENT_TYPES = ['view', 'pricing_view', 'pro_copy', 'checkout_click'] as const;
const HOW = ['copy', 'keys', 'cli', 'menu'];
const BOT = /bot|crawl|spider|slurp|headless|lighthouse|pagespeed|preview|prerender|phantom|puppeteer|playwright|selenium|curl|wget|python|httpclient|okhttp|go-http|java\/|axios|node-fetch|scrapy|facebookexternalhit|embedly|whatsapp|discord|telegram|monitor|uptime|pingdom/i;
const SLUG = /^[a-z0-9][a-z0-9-]{0,79}$/;
const PLAN_KEY = /^(pro|pro-founder|team)-(month|year)$/;

const now = () => Math.floor(Date.now() / 1000);

/** The slug a page path is about: /components/<group>/<slug>/ or /screens/<slug>/. */
export function itemOf(path: string): string | null {
  const m = path.match(/^\/components\/[a-z0-9-]+\/([a-z0-9-]+)\/?$/) || path.match(/^\/screens\/([a-z0-9-]+)\/(?:full\/?)?$/);
  return m && SLUG.test(m[1]) ? m[1] : null;
}

function cleanPath(p: unknown): string | null {
  if (typeof p !== 'string' || !p.startsWith('/') || p.startsWith('//')) return null;
  const path = p.split(/[?#]/)[0].slice(0, 200);
  if (/^\/(admin|api)(\/|$)/.test(path) || /[^\x21-\x7e]/.test(path)) return null;
  return path;
}

async function limit(env: Env, ip: string) {
  const t = now();
  const h = (await sha256Hex(`${env.RATE_SALT ?? 'oxyn-events'}|ev|${ip}`)).slice(0, 32);
  const checks = [{ key: `ev1m:${h}`, ...EVENTS.burst }, { key: `ev1h:${h}`, ...EVENTS.hourly }];
  const res = await env.DB.batch(checks.map((c) => env.DB.prepare(
    'INSERT INTO rate_limits (key, window_start, count) VALUES (?1, ?2, 1) ON CONFLICT (key, window_start) DO UPDATE SET count = count + 1 RETURNING count',
  ).bind(c.key, t - (t % c.window))));
  res.forEach((r, i) => {
    const n = Number((r.results?.[0] as { count?: number } | undefined)?.count ?? 0);
    if (n > checks[i].max) throw new HttpError(429, 'rate_limited', 'Too many events from here.');
  });
}

/** POST /api/events */
export async function postEvents(req: Request, env: Env, ctx: { waitUntil(p: Promise<unknown>): void }, url: URL): Promise<Response> {
  if (req.method !== 'POST') throw new HttpError(405, 'method_not_allowed', 'Use POST.');
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Events can only be sent from the Oxyn site.');
  const ua = req.headers.get('user-agent') || '';
  // Bots, link previews and headless browsers: accept and drop, so they do not retry.
  if (!ua || BOT.test(ua)) return json({ ok: true, stored: 0, dropped: 'bot' }, 202);
  if (Number(req.headers.get('content-length') || '0') > EVENTS.maxBody) throw new HttpError(413, 'too_large', 'Too many events in one request.');
  const raw = await req.text();
  if (raw.length > EVENTS.maxBody) throw new HttpError(413, 'too_large', 'Too many events in one request.');
  let body: { events?: unknown };
  try { body = JSON.parse(raw); } catch { throw new HttpError(400, 'bad_body', 'Send JSON: {"events": [...]}.'); }
  const list = Array.isArray(body?.events) ? body.events.slice(0, EVENTS.maxBatch) : [];
  if (!list.length) throw new HttpError(400, 'no_events', 'Send at least one event.');

  const ip = req.headers.get('cf-connecting-ip') || 'local';
  await limit(env, ip);

  const t = now();
  const day = new Date(t * 1000).toISOString().slice(0, 10);
  const visitor = (await sha256Hex(`${env.RATE_SALT ?? 'oxyn-events'}|v|${day}|${ip}|${ua}`)).slice(0, 16);
  const u = await currentUser(req, env).catch(() => null);
  const rows: [string, string | null, string | null, string | null][] = [];
  for (const e of list as Record<string, unknown>[]) {
    if (!e || typeof e !== 'object') continue;
    const type = String(e.type || '');
    if (!(EVENT_TYPES as readonly string[]).includes(type)) continue;
    const path = cleanPath(e.path);
    if (!path) continue;
    let item: string | null = null;
    let detail: string | null = null;
    if (type === 'view') item = itemOf(path);
    else if (type === 'pro_copy') {
      item = typeof e.item === 'string' && SLUG.test(e.item) ? e.item : itemOf(path);
      detail = typeof e.detail === 'string' && HOW.includes(e.detail) ? e.detail : 'copy';
      if (!item) continue;
    } else if (type === 'pricing_view') item = typeof e.item === 'string' && SLUG.test(e.item) ? e.item : null;
    else if (type === 'checkout_click') {
      item = typeof e.item === 'string' && PLAN_KEY.test(e.item) ? e.item : null;
      if (!item) continue;
    }
    rows.push([type, path, item, detail]);
  }
  if (rows.length) {
    await env.DB.batch(rows.map(([type, path, item, detail]) =>
      env.DB.prepare('INSERT INTO events (at, type, path, item, detail, user_id, visitor) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)')
        .bind(t, type, path, item, detail, u?.id ?? null, visitor)));
  }
  if (Math.random() < 0.01) ctx.waitUntil(prune(env));
  return json({ ok: true, stored: rows.length }, 202);
}

export async function prune(env: Env) {
  await env.DB.prepare('DELETE FROM events WHERE at < ?1').bind(now() - EVENTS.retentionDays * 86_400).run();
}

/** A sign-in (any provider), recorded by the Worker itself. Never fails the sign-in. */
export async function recordSignin(env: Env, userId: string, provider: string) {
  try {
    await env.DB.prepare("INSERT INTO events (at, type, path, item, user_id) VALUES (?1, 'signin', NULL, ?2, ?3)").bind(now(), provider.slice(0, 20), userId).run();
  } catch (e) { console.error('signin event', e); }
}
