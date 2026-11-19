// The maintainers' admin panel: /admin (the UI) and /api/admin/* (its JSON API). See docs/admin.md.
//
// Three checks, all on every request:
//   1. Cloudflare Access (application "Oxyn admin") stops anyone who is not in the Access group
//      "oxyn-admins" at the edge, with an email one-time PIN.
//   2. This Worker verifies the Cf-Access-Jwt-Assertion JWT itself (RS256 against the team's
//      certs, audience, issuer, expiry) and requires the email to be an active row in D1 `admins`.
//      A header is never trusted on its own.
//   3. A shared password, set by the first admin who opens /admin, stored as PBKDF2-SHA256 in D1
//      `settings`. Signing in gives a 12 h HttpOnly session cookie bound to the Access email.
//
// The UI files (admin-ui/) are served only after checks 1 and 2, so none of it is public.

import { EMAIL_RE, HttpError, KINDS, fail, json, openIssue, sha256Hex, type Ctx, type Env, type Kind } from './shared';
import PAGE from './admin-ui/index.html';
import CSS from './admin-ui/app.css';
import JS from './admin-ui/app.js';

export const ADMIN = {
  session: 12 * 3600, // seconds
  pbkdf2: 100_000, // iterations (the Workers runtime allows up to 100 000)
  passwordMin: 12,
  passwordMax: 256,
  // failed password attempts: per Access email and per client IP
  failEmail: { window: 900, max: 5 }, // 5 per 15 minutes
  failIp: { window: 86_400, max: 30 }, // 30 per day
  flagValueMax: 500,
  pageSize: 50,
};
const COOKIE = '__Host-oxyn_admin';
const STATUSES = ['new', 'triaged', 'closed'] as const;
const FLAG_KEY = /^[a-z0-9][a-z0-9_.-]{0,47}$/;

/** The verified identity behind a request. */
export interface AccessIdentity { email: string }
export type AccessVerifier = (req: Request, env: Env) => Promise<AccessIdentity>;

// ── Base64url + bytes ─────────────────────────────────────────────────────────────────────
const enc = new TextEncoder();
function b64u(bytes: ArrayBuffer | Uint8Array) {
  const a = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  let s = '';
  for (const b of a) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
function unb64u(s: string) {
  const t = s.replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(t + '='.repeat((4 - (t.length % 4)) % 4));
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}
function randomB64u(n: number) { return b64u(crypto.getRandomValues(new Uint8Array(n))); }
/** Constant-time comparison of two byte arrays. */
function sameBytes(a: Uint8Array, b: Uint8Array) {
  if (a.length !== b.length) return false;
  let d = 0;
  for (let i = 0; i < a.length; i++) d |= a[i] ^ b[i];
  return d === 0;
}
const now = () => Math.floor(Date.now() / 1000);
const iso = () => new Date().toISOString();

// ── Gate 2: Cloudflare Access JWT ─────────────────────────────────────────────────────────
interface Jwk { kid: string; kty: string; alg?: string; n: string; e: string }
let certs: { team: string; at: number; keys: Map<string, CryptoKey> } | null = null;

async function accessKey(team: string, kid: string): Promise<CryptoKey | undefined> {
  const fresh = certs && certs.team === team && now() - certs.at < 3600;
  if (fresh && certs!.keys.has(kid)) return certs!.keys.get(kid);
  // Unknown kid or stale cache: refetch (keys rotate), at most once a minute.
  if (!fresh || now() - certs!.at > 60) {
    const r = await fetch(`${team}/cdn-cgi/access/certs`, { headers: { accept: 'application/json' } });
    if (!r.ok) throw new HttpError(503, 'access_unavailable', 'Could not reach Cloudflare Access to check your sign-in.');
    const { keys = [] } = (await r.json()) as { keys?: Jwk[] };
    const map = new Map<string, CryptoKey>();
    for (const k of keys) {
      if (k.kty !== 'RSA' || !k.kid) continue;
      map.set(k.kid, await crypto.subtle.importKey('jwk', { kty: 'RSA', n: k.n, e: k.e, alg: 'RS256', ext: true },
        { name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256' }, false, ['verify']));
    }
    certs = { team, at: now(), keys: map };
  }
  return certs!.keys.get(kid);
}

/** Verifies the Access JWT on the request: RS256 signature by the team's current keys, this
 *  application's audience, the team as issuer, not expired. Returns the signed-in email. */
export const verifyAccessJwt: AccessVerifier = async (req, env) => {
  const team = (env.ACCESS_TEAM_DOMAIN || '').replace(/\/+$/, '');
  const aud = env.ACCESS_AUD || '';
  if (!team || !aud) throw new HttpError(503, 'admin_not_configured', 'The admin panel is not configured (ACCESS_TEAM_DOMAIN / ACCESS_AUD).');
  const token = req.headers.get('cf-access-jwt-assertion') || '';
  const deny = (why: string) => new HttpError(401, 'access_required', `Cloudflare Access sign-in required (${why}).`);
  const parts = token.split('.');
  if (parts.length !== 3) throw deny('no token');
  let head: { alg?: string; kid?: string }, claims: { aud?: string | string[]; iss?: string; exp?: number; nbf?: number; email?: string; type?: string };
  try {
    head = JSON.parse(new TextDecoder().decode(unb64u(parts[0])));
    claims = JSON.parse(new TextDecoder().decode(unb64u(parts[1])));
  } catch { throw deny('malformed token'); }
  if (head.alg !== 'RS256' || !head.kid) throw deny('bad algorithm');
  const key = await accessKey(team, head.kid);
  if (!key) throw deny('unknown key');
  const ok = await crypto.subtle.verify('RSASSA-PKCS1-v1_5', key, unb64u(parts[2]), enc.encode(`${parts[0]}.${parts[1]}`));
  if (!ok) throw deny('bad signature');
  const auds = Array.isArray(claims.aud) ? claims.aud : [claims.aud];
  if (!auds.includes(aud)) throw deny('wrong audience');
  if (claims.iss !== team) throw deny('wrong issuer');
  const t = now();
  if (typeof claims.exp !== 'number' || claims.exp < t - 30) throw deny('expired');
  if (typeof claims.nbf === 'number' && claims.nbf > t + 30) throw deny('not yet valid');
  if (claims.type && claims.type !== 'app') throw deny('not an app token');
  const email = (claims.email || '').trim().toLowerCase();
  if (!email || !EMAIL_RE.test(email)) throw deny('no email (service tokens are not admins)');
  return { email };
};

// ── Gate 3: shared password, sessions ─────────────────────────────────────────────────────
interface PwRecord { alg: 'pbkdf2-sha256'; iter: number; salt: string; hash: string; version: number; set_by: string }

async function pbkdf2(password: string, salt: Uint8Array, iter: number) {
  const k = await crypto.subtle.importKey('raw', enc.encode(password), 'PBKDF2', false, ['deriveBits']);
  return new Uint8Array(await crypto.subtle.deriveBits({ name: 'PBKDF2', hash: 'SHA-256', salt, iterations: iter }, k, 256));
}
async function getSetting(env: Env, key: string) {
  const r = await env.DB.prepare('SELECT value FROM settings WHERE key = ?1').bind(key).first<{ value: string }>();
  return r ? r.value : null;
}
async function passwordRecord(env: Env): Promise<PwRecord | null> {
  const v = await getSetting(env, 'admin_password');
  return v ? (JSON.parse(v) as PwRecord) : null;
}
async function newPwRecord(password: string, version: number, by: string): Promise<PwRecord> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const hash = await pbkdf2(password, salt, ADMIN.pbkdf2);
  return { alg: 'pbkdf2-sha256', iter: ADMIN.pbkdf2, salt: b64u(salt), hash: b64u(hash), version, set_by: by };
}
async function checkPassword(rec: PwRecord, password: string) {
  const got = await pbkdf2(password, unb64u(rec.salt), rec.iter);
  return sameBytes(got, unb64u(rec.hash));
}
function validateNew(password: unknown, confirm: unknown) {
  if (typeof password !== 'string' || password.length < ADMIN.passwordMin)
    throw new HttpError(400, 'password_too_short', `Use at least ${ADMIN.passwordMin} characters.`, 'password');
  if (password.length > ADMIN.passwordMax) throw new HttpError(400, 'password_too_long', `Use at most ${ADMIN.passwordMax} characters.`, 'password');
  if (password !== confirm) throw new HttpError(400, 'password_mismatch', 'The two passwords do not match.', 'confirm');
  return password;
}

/** HMAC key for session cookies: the ADMIN_SESSION_KEY secret, else 32 random bytes kept in D1. */
async function sessionKey(env: Env) {
  let raw = env.ADMIN_SESSION_KEY;
  if (!raw) {
    await env.DB.prepare('INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)')
      .bind('admin_session_key', randomB64u(32), iso()).run();
    raw = (await getSetting(env, 'admin_session_key'))!;
  }
  return crypto.subtle.importKey('raw', enc.encode(raw), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign', 'verify']);
}
async function sign(env: Env, sid: string) { return b64u(await crypto.subtle.sign('HMAC', await sessionKey(env), enc.encode(sid))); }

function readCookie(req: Request) {
  for (const part of (req.headers.get('cookie') || '').split(';')) {
    const i = part.indexOf('=');
    if (i > 0 && part.slice(0, i).trim() === COOKIE) return part.slice(i + 1).trim();
  }
  return '';
}
const cookieHeader = (value: string, maxAge: number) =>
  `${COOKIE}=${value}; Path=/; Max-Age=${maxAge}; HttpOnly; Secure; SameSite=Strict`;

async function startSession(env: Env, email: string, version: number) {
  const sid = randomB64u(32);
  await env.DB.prepare('INSERT INTO admin_sessions (id, email, pw_version, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)')
    .bind(sid, email, version, iso(), now() + ADMIN.session).run();
  if (Math.random() < 0.1) await env.DB.prepare('DELETE FROM admin_sessions WHERE expires_at < ?1').bind(now()).run();
  return cookieHeader(`${sid}.${await sign(env, sid)}`, ADMIN.session);
}

/** The session id when the cookie is genuine, unexpired, for this Access email and the current password. */
async function currentSession(req: Request, env: Env, email: string, rec: PwRecord | null) {
  if (!rec) return null;
  const [sid, sig] = readCookie(req).split('.');
  if (!sid || !sig) return null;
  const key = await sessionKey(env);
  let ok = false;
  try { ok = await crypto.subtle.verify('HMAC', key, unb64u(sig), enc.encode(sid)); } catch { ok = false; }
  if (!ok) return null;
  const row = await env.DB.prepare('SELECT email, pw_version, expires_at FROM admin_sessions WHERE id = ?1')
    .bind(sid).first<{ email: string; pw_version: number; expires_at: number }>();
  if (!row || row.expires_at <= now() || row.email.toLowerCase() !== email || row.pw_version !== rec.version) return null;
  return sid;
}

async function ipHash(req: Request, env: Env) {
  return (await sha256Hex(`${env.RATE_SALT ?? 'oxyn-feedback'}|admin|${req.headers.get('cf-connecting-ip') || 'local'}`)).slice(0, 32);
}
/** Failed-password windows. `check` throws 429 when either is full; `record` counts one failure. */
async function failWindows(req: Request, env: Env, email: string) {
  const t = now();
  const ih = await ipHash(req, env);
  const w = [
    { key: `admpw:${await sha256Hex(email)}`.slice(0, 40), ...ADMIN.failEmail },
    { key: `admip:${ih}`, ...ADMIN.failIp },
  ].map((x) => ({ ...x, start: t - (t % x.window) }));
  return {
    async check() {
      for (const x of w) {
        const r = await env.DB.prepare('SELECT count FROM rate_limits WHERE key = ?1 AND window_start = ?2').bind(x.key, x.start).first<{ count: number }>();
        if (r && r.count >= x.max) throw new HttpError(429, 'too_many_attempts', 'Too many wrong passwords. Wait 15 minutes and try again.');
      }
    },
    async record() {
      await env.DB.batch(w.map((x) => env.DB.prepare(
        'INSERT INTO rate_limits (key, window_start, count) VALUES (?1, ?2, 1) ON CONFLICT (key, window_start) DO UPDATE SET count = count + 1',
      ).bind(x.key, x.start)));
    },
  };
}

async function audit(env: Env, actor: string, action: string, target: string | null = null, detail: unknown = null) {
  await env.DB.prepare('INSERT INTO audit_log (at, actor, action, target, detail) VALUES (?1, ?2, ?3, ?4, ?5)')
    .bind(iso(), actor, action, target, detail == null ? null : typeof detail === 'string' ? detail : JSON.stringify(detail)).run();
}

// ── Routing ───────────────────────────────────────────────────────────────────────────────
const SEC = {
  'x-robots-tag': 'noindex, nofollow, noarchive',
  'x-frame-options': 'DENY',
  'referrer-policy': 'no-referrer',
  'x-content-type-options': 'nosniff',
  'cache-control': 'no-store',
};
const CSP = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' blob:; connect-src 'self'; " +
  "font-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

function page(body: string, type: string, status = 200) {
  return new Response(body, { status, headers: { 'content-type': type, 'content-security-policy': CSP, ...SEC } });
}
const deniedPage = (msg: string, status: number) =>
  page(`<!doctype html><meta charset="utf-8"><meta name="robots" content="noindex"><title>Oxyn admin</title><p>${msg}</p>`, 'text/html; charset=utf-8', status);

export function makeAdminHandler(verify: AccessVerifier) {
  return async function handleAdmin(req: Request, env: Env, ctx: Ctx, url: URL): Promise<Response> {
    const isApi = url.pathname.startsWith('/api/');
    try {
      const who = await verify(req, env);
      const admin = await env.DB.prepare('SELECT email FROM admins WHERE email = ?1 AND removed_at IS NULL').bind(who.email).first();
      if (!admin) throw new HttpError(403, 'not_an_admin', 'This account is not on the admin list.');
      if (!isApi) return ui(req, url);
      return await api(req, env, ctx, url, who.email);
    } catch (err) {
      const e = err instanceof HttpError ? err : new HttpError(500, 'internal', 'Something went wrong on our side.');
      if (!(err instanceof HttpError)) console.error('admin error', err);
      if (!isApi) return deniedPage(e.message, e.status);
      return fail(e, { 'x-robots-tag': SEC['x-robots-tag'] });
    }
  };
}
export const handleAdmin = makeAdminHandler(verifyAccessJwt);

function ui(req: Request, url: URL) {
  if (req.method !== 'GET' && req.method !== 'HEAD') return deniedPage('Method not allowed.', 405);
  const p = url.pathname.replace(/\/+$/, '') || '/admin';
  if (p === '/admin') return page(PAGE, 'text/html; charset=utf-8');
  if (p === '/admin/app.css') return page(CSS, 'text/css; charset=utf-8');
  if (p === '/admin/app.js') return page(JS, 'text/javascript; charset=utf-8');
  return deniedPage('Not found.', 404);
}

async function body(req: Request): Promise<Record<string, unknown>> {
  if (!(req.headers.get('content-type') || '').startsWith('application/json')) throw new HttpError(415, 'unsupported_media_type', 'Send application/json.');
  try { return (await req.json()) as Record<string, unknown>; } catch { throw new HttpError(400, 'bad_body', 'The JSON body could not be read.'); }
}

async function api(req: Request, env: Env, ctx: Ctx, url: URL, email: string): Promise<Response> {
  const m = req.method;
  const route = url.pathname.replace(/\/+$/, '').slice('/api/admin'.length) || '/';
  // Changes must come from the panel itself (SameSite=Strict cookie + same Origin + JSON).
  if (m !== 'GET' && m !== 'HEAD') {
    const origin = req.headers.get('origin');
    if (!origin || origin !== url.origin) throw new HttpError(403, 'forbidden_origin', 'Admin changes can only be made from the admin panel.');
  }
  const rec = await passwordRecord(env);
  const sid = await currentSession(req, env, email, rec);

  if (route === '/me' && m === 'GET') {
    return json({ ok: true, email, password_set: !!rec, signed_in: !!sid, github: !!env.GITHUB_TOKEN, access_sync: !!env.CF_ADMIN_TOKEN });
  }
  if (route === '/setup' && m === 'POST') {
    if (rec) throw new HttpError(409, 'already_set', 'The shared password is already set. Sign in with it.');
    const b = await body(req);
    const pw = validateNew(b.password, b.confirm);
    const r = await newPwRecord(pw, 1, email);
    const ins = await env.DB.prepare('INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)')
      .bind('admin_password', JSON.stringify(r), iso()).run();
    if (!(ins.meta && Number((ins.meta as { changes?: number }).changes) > 0)) throw new HttpError(409, 'already_set', 'Another admin just set the password. Sign in with it.');
    await audit(env, email, 'password.setup');
    return json({ ok: true }, 201, { 'set-cookie': await startSession(env, email, r.version) });
  }
  if (route === '/login' && m === 'POST') {
    if (!rec) throw new HttpError(409, 'not_set', 'No shared password yet. Set one first.');
    const b = await body(req);
    const fw = await failWindows(req, env, email);
    await fw.check();
    if (typeof b.password !== 'string' || !b.password || b.password.length > ADMIN.passwordMax || !(await checkPassword(rec, b.password))) {
      await fw.record();
      await audit(env, email, 'login.fail');
      throw new HttpError(401, 'wrong_password', 'That password is not right.', 'password');
    }
    await audit(env, email, 'login');
    return json({ ok: true }, 200, { 'set-cookie': await startSession(env, email, rec.version) });
  }
  if (route === '/logout' && m === 'POST') {
    if (sid) await env.DB.prepare('DELETE FROM admin_sessions WHERE id = ?1').bind(sid).run();
    if (sid) await audit(env, email, 'logout');
    return json({ ok: true }, 200, { 'set-cookie': cookieHeader('', 0) });
  }

  if (!sid) throw new HttpError(401, 'session_required', rec ? 'Sign in with the shared password.' : 'Set the shared password first.');

  // Feedback inbox
  if (route === '/feedback' && m === 'GET') return listFeedback(env, url);
  const fm = route.match(/^\/feedback\/([0-9a-f-]{36})(\/screenshot|\/github)?$/);
  if (fm) {
    const id = fm[1];
    const row = await env.DB.prepare('SELECT * FROM feedback WHERE id = ?1').bind(id).first<Record<string, string | null>>();
    if (!row) throw new HttpError(404, 'not_found', 'That feedback no longer exists.');
    if (!fm[2] && m === 'GET') return json({ ok: true, item: row });
    if (!fm[2] && m === 'PATCH') {
      const b = await body(req);
      const st = String(b.status || '');
      if (!(STATUSES as readonly string[]).includes(st)) throw new HttpError(400, 'invalid_status', 'Status is new, triaged or closed.', 'status');
      await env.DB.prepare('UPDATE feedback SET status = ?1 WHERE id = ?2').bind(st, id).run();
      await audit(env, email, 'feedback.status', id, { from: row.status, to: st });
      return json({ ok: true });
    }
    if (!fm[2] && m === 'DELETE') {
      if (row.screenshot_key) await env.UPLOADS.delete(row.screenshot_key);
      await env.DB.prepare('DELETE FROM feedback WHERE id = ?1').bind(id).run();
      await audit(env, email, 'feedback.delete', id, { kind: row.kind });
      return json({ ok: true });
    }
    if (fm[2] === '/screenshot' && m === 'GET') {
      if (!row.screenshot_key) throw new HttpError(404, 'no_screenshot', 'This feedback has no screenshot.');
      const obj = await env.UPLOADS.get(row.screenshot_key);
      if (!obj) throw new HttpError(404, 'no_screenshot', 'The screenshot file is missing.');
      const type = obj.httpMetadata?.contentType || 'application/octet-stream';
      return new Response(obj.body, { headers: { 'content-type': type.startsWith('image/') ? type : 'application/octet-stream', 'content-disposition': 'inline', ...SEC } });
    }
    if (fm[2] === '/github' && m === 'POST') {
      if (!env.GITHUB_TOKEN) throw new HttpError(409, 'github_not_configured', 'Set the GITHUB_TOKEN Worker secret to send feedback to GitHub.');
      if (row.github_issue_url) throw new HttpError(409, 'already_sent', 'This feedback already has an issue.');
      let issue = '';
      try {
        issue = await openIssue(env, { id, kind: row.kind as Kind, message: row.message || '', page: row.page_url || '', hasShot: !!row.screenshot_key, created: row.created_at || '' });
      } catch (e) {
        console.error('github issue', e);
        throw new HttpError(502, 'github_failed', 'GitHub did not accept the issue. Check the token and try again.');
      }
      await audit(env, email, 'feedback.github', id, issue);
      return json({ ok: true, url: issue });
    }
    throw new HttpError(405, 'method_not_allowed', 'Method not allowed.');
  }

  // Admins
  if (route === '/admins' && m === 'GET') return listAdmins(env, email);
  if (route === '/admins' && m === 'POST') {
    const b = await body(req);
    const add = String(b.email || '').trim().toLowerCase();
    if (!add || add.length > 254 || !EMAIL_RE.test(add)) throw new HttpError(400, 'invalid_email', 'That email address does not look right.', 'email');
    const ex = await env.DB.prepare('SELECT removed_at FROM admins WHERE email = ?1').bind(add).first<{ removed_at: string | null }>();
    if (ex && !ex.removed_at) throw new HttpError(409, 'already_admin', 'That email is already an admin.', 'email');
    if (ex) await env.DB.prepare('UPDATE admins SET removed_at = NULL, access_synced = 0, added_by = ?2, created_at = ?3 WHERE email = ?1').bind(add, email, iso()).run();
    else await env.DB.prepare('INSERT INTO admins (email, added_by, created_at, access_synced) VALUES (?1, ?2, ?3, 0)').bind(add, email, iso()).run();
    await audit(env, email, 'admin.add', add);
    const sync = await syncAccess(env, email);
    return json({ ok: true, sync }, 201);
  }
  const am = route.match(/^\/admins\/([^/]+)$/);
  if (am && m === 'DELETE') {
    let target = '';
    try { target = decodeURIComponent(am[1]).toLowerCase(); } catch { throw new HttpError(400, 'invalid_email', 'Bad email.'); }
    const row = await env.DB.prepare('SELECT email FROM admins WHERE email = ?1 AND removed_at IS NULL').bind(target).first();
    if (!row) throw new HttpError(404, 'not_found', 'That email is not an admin.');
    const n = await env.DB.prepare('SELECT COUNT(*) AS n FROM admins WHERE removed_at IS NULL').first<{ n: number }>();
    if (!n || n.n <= 1) throw new HttpError(409, 'last_admin', 'Keep at least one admin. Add someone else first.');
    await env.DB.batch([
      env.DB.prepare('UPDATE admins SET removed_at = ?2, access_synced = 0 WHERE email = ?1').bind(target, iso()),
      env.DB.prepare('DELETE FROM admin_sessions WHERE email = ?1').bind(target),
    ]);
    await audit(env, email, 'admin.remove', target);
    const sync = await syncAccess(env, email);
    return json({ ok: true, sync, self: target === email });
  }
  if (route === '/admins/sync' && m === 'POST') return json({ ok: true, sync: await syncAccess(env, email) });

  // Settings: password + flags
  if (route === '/password' && m === 'POST') {
    const b = await body(req);
    const fw = await failWindows(req, env, email);
    await fw.check();
    if (typeof b.current !== 'string' || b.current.length > ADMIN.passwordMax || !(await checkPassword(rec!, b.current))) {
      await fw.record();
      await audit(env, email, 'password.change_fail');
      throw new HttpError(401, 'wrong_password', 'The current password is not right.', 'current');
    }
    const pw = validateNew(b.password, b.confirm);
    const r = await newPwRecord(pw, rec!.version + 1, email);
    await env.DB.batch([
      env.DB.prepare('UPDATE settings SET value = ?2, updated_at = ?3 WHERE key = ?1').bind('admin_password', JSON.stringify(r), iso()),
      env.DB.prepare('DELETE FROM admin_sessions'), // everyone signs in again with the new password
    ]);
    await audit(env, email, 'password.change');
    return json({ ok: true }, 200, { 'set-cookie': await startSession(env, email, r.version) });
  }
  if (route === '/flags' && m === 'GET') return json({ ok: true, flags: await readFlags(env) });
  const flm = route.match(/^\/flags\/([^/]+)$/);
  if (flm && (m === 'PUT' || m === 'DELETE')) {
    const key = decodeURIComponent(flm[1]);
    if (!FLAG_KEY.test(key)) throw new HttpError(400, 'invalid_key', 'Flag names use a-z, 0-9, _ . - (up to 48).', 'key');
    if (m === 'DELETE') {
      await env.DB.prepare('DELETE FROM settings WHERE key = ?1').bind(`flag:${key}`).run();
      await audit(env, email, 'flag.delete', key);
      return json({ ok: true });
    }
    const b = await body(req);
    const value = typeof b.value === 'string' ? b.value.trim() : '';
    if (value.length > ADMIN.flagValueMax) throw new HttpError(400, 'value_too_long', `Keep the value under ${ADMIN.flagValueMax} characters.`, 'value');
    const v = JSON.stringify({ enabled: b.enabled === true, value });
    await env.DB.prepare('INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT (key) DO UPDATE SET value = ?2, updated_at = ?3')
      .bind(`flag:${key}`, v, iso()).run();
    await audit(env, email, 'flag.set', key, { enabled: b.enabled === true, value: value.slice(0, 80) });
    return json({ ok: true });
  }
  if (route === '/audit' && m === 'GET') {
    const before = Number(url.searchParams.get('before') || 0);
    const r = await env.DB.prepare('SELECT id, at, actor, action, target, detail FROM audit_log WHERE (?1 = 0 OR id < ?1) ORDER BY id DESC LIMIT 100').bind(before).all();
    return json({ ok: true, items: r.results || [] });
  }
  throw new HttpError(404, 'not_found', 'No such admin route.');
}

async function listFeedback(env: Env, url: URL) {
  const q = url.searchParams;
  const kind = q.get('kind') || '';
  const status = q.get('status') || '';
  const before = q.get('before') || '';
  const text = (q.get('q') || '').trim().slice(0, 100);
  if (kind && !(KINDS as readonly string[]).includes(kind)) throw new HttpError(400, 'invalid_kind', 'Unknown kind.');
  if (status && !(STATUSES as readonly string[]).includes(status)) throw new HttpError(400, 'invalid_status', 'Unknown status.');
  const like = text ? `%${text.replace(/[\\%_]/g, (c) => '\\' + c)}%` : '';
  const r = await env.DB.prepare(
    "SELECT id, kind, substr(message, 1, 240) AS message, page_url, email, created_at, status, github_issue_url, screenshot_key IS NOT NULL AS has_screenshot FROM feedback " +
    "WHERE (?1 = '' OR kind = ?1) AND (?2 = '' OR status = ?2) AND (?3 = '' OR created_at < ?3) " +
    "AND (?4 = '' OR message LIKE ?4 ESCAPE '\\' OR email LIKE ?4 ESCAPE '\\') ORDER BY created_at DESC LIMIT ?5",
  ).bind(kind, status, before, like, ADMIN.pageSize + 1).all<Record<string, unknown>>();
  const items = r.results || [];
  const more = items.length > ADMIN.pageSize;
  const counts = await env.DB.prepare('SELECT status, COUNT(*) AS n FROM feedback GROUP BY status').all<{ status: string; n: number }>();
  return json({
    ok: true,
    items: items.slice(0, ADMIN.pageSize),
    next: more ? items[ADMIN.pageSize - 1].created_at : null,
    counts: Object.fromEntries((counts.results || []).map((c) => [c.status, c.n])),
  });
}

async function listAdmins(env: Env, me: string) {
  const r = await env.DB.prepare('SELECT email, added_by, created_at, access_synced, removed_at FROM admins ORDER BY removed_at IS NOT NULL, created_at').all();
  const err = await getSetting(env, 'access_sync_error');
  return json({ ok: true, me, items: r.results || [], access_sync: !!env.CF_ADMIN_TOKEN, sync_error: err || null });
}

/** Makes the Access group's email rules equal the active admin list. Without CF_ADMIN_TOKEN the
 *  change stays in D1 as "pending sync" (the Worker already enforces it). */
async function syncAccess(env: Env, actor: string): Promise<{ ok: boolean; reason?: string }> {
  const pending = await env.DB.prepare('SELECT COUNT(*) AS n FROM admins WHERE access_synced = 0').first<{ n: number }>();
  if (!pending || pending.n === 0) return { ok: true };
  if (!env.CF_ADMIN_TOKEN || !env.CF_ACCOUNT_ID || !env.ACCESS_GROUP_ID) return { ok: false, reason: 'no_token' };
  const base = `https://api.cloudflare.com/client/v4/accounts/${env.CF_ACCOUNT_ID}/access/groups/${env.ACCESS_GROUP_ID}`;
  const headers = { authorization: `Bearer ${env.CF_ADMIN_TOKEN}`, 'content-type': 'application/json' };
  const setErr = (msg: string) => env.DB.prepare('INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT (key) DO UPDATE SET value = ?2, updated_at = ?3')
    .bind('access_sync_error', msg, iso()).run();
  try {
    const g = await fetch(base, { headers });
    const gj = (await g.json()) as { success?: boolean; result?: { name: string; include?: Record<string, unknown>[]; exclude?: unknown[]; require?: unknown[] } };
    if (!g.ok || !gj.success || !gj.result) throw new Error(`read group ${g.status}`);
    const active = await env.DB.prepare('SELECT email FROM admins WHERE removed_at IS NULL ORDER BY email').all<{ email: string }>();
    const keep = (gj.result.include || []).filter((r) => !('email' in r)); // other rule types stay as they are
    const include = [...keep, ...(active.results || []).map((a) => ({ email: { email: a.email } }))];
    const u = await fetch(base, { method: 'PUT', headers, body: JSON.stringify({ name: gj.result.name, include, exclude: gj.result.exclude || [], require: gj.result.require || [] }) });
    const uj = (await u.json()) as { success?: boolean; errors?: { message?: string }[] };
    if (!u.ok || !uj.success) throw new Error(`update group ${u.status} ${(uj.errors || []).map((e) => e.message).join('; ')}`.slice(0, 200));
    await env.DB.batch([
      env.DB.prepare('UPDATE admins SET access_synced = 1 WHERE removed_at IS NULL'),
      env.DB.prepare('DELETE FROM admins WHERE removed_at IS NOT NULL'),
      env.DB.prepare("DELETE FROM settings WHERE key = 'access_sync_error'"),
    ]);
    await audit(env, actor, 'access.sync', env.ACCESS_GROUP_ID, { emails: (active.results || []).length });
    return { ok: true };
  } catch (e) {
    console.error('access sync', e);
    await setErr(String((e as Error).message || e));
    return { ok: false, reason: 'error' };
  }
}

async function readFlags(env: Env) {
  const r = await env.DB.prepare("SELECT key, value, updated_at FROM settings WHERE key LIKE 'flag:%' ORDER BY key").all<{ key: string; value: string; updated_at: string }>();
  return (r.results || []).map((x) => {
    let v: { enabled?: boolean; value?: string } = {};
    try { v = JSON.parse(x.value); } catch { /* keep defaults */ }
    return { key: x.key.slice(5), enabled: v.enabled === true, value: v.value || '', updated_at: x.updated_at };
  });
}

/** GET /api/flags (public): the enabled flags as { key: value }. */
export async function publicFlags(env: Env) {
  const flags = Object.fromEntries((await readFlags(env)).filter((f) => f.enabled).map((f) => [f.key, f.value]));
  return json({ ok: true, flags }, 200, { 'cache-control': 'public, max-age=60' });
}
