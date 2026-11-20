// Site accounts: one sign-in for everyone, with Google, GitHub, or an email and a password. The
// maintainers' admin panel (admin.ts) uses these same sessions: an admin is a signed-in account
// whose email Google or GitHub verified and that is on the D1 `admins` list. Pages are static
// (/login/, /signup/, /account/); they and the navbar menu call these routes from the browser
// (public/account.js), so nothing here touches the static cache. See docs/auth.md.
//
//   GET  /api/auth/me                 who is signed in ({signed_in:false} when nobody), providers,
//                                     is_admin
//   POST /api/auth/signup             {email, password}           -> session cookie
//   POST /api/auth/login              {email, password}           -> session cookie
//   POST /api/auth/logout             ends this session
//   POST /api/auth/logout-all         ends every session of this account
//   GET  /api/auth/github/start       redirect to GitHub (503 JSON until the OAuth app is set up)
//   GET  /api/auth/github/callback    GitHub redirects back here
//   GET  /api/auth/google/start       redirect to Google (503 JSON until the OAuth client is set up)
//   GET  /api/auth/google/callback    Google redirects back here (OpenID Connect, code + PKCE)
//
// Sessions: a random 256-bit token in an HttpOnly, Secure, SameSite=Lax cookie; D1 keeps only its
// SHA-256, so signing out (or "sign out everywhere") is a row delete. Passwords: PBKDF2-SHA256,
// 100 000 iterations, 16-byte salt. Every failure on sign-in reads the same, and a sign-in for an
// unknown email still runs PBKDF2, so responses do not tell which emails have accounts.

import { EMAIL_RE, HttpError, fail, json, sameOrigin, sha256Hex, type Ctx, type Env } from './shared';
import { recordSignin } from './events';

export const AUTH = {
  cookie: 'oxyn_session',
  hint: 'oxyn_signed_in', // readable by the page: "a session may exist, ask /api/auth/me"
  stateCookie: 'oxyn_gh_state',
  googleCookie: 'oxyn_gg_state',
  sessionDays: 30,
  pbkdf2: 100_000, // the Workers runtime allows at most 100 000
  passwordMin: 10,
  passwordMax: 256,
  email: 254,
  // fixed windows per hashed IP, and per hashed email for sign-in
  signup: [{ scope: 'su10m', window: 600, max: 5 }, { scope: 'su1d', window: 86_400, max: 20 }],
  login: [{ scope: 'li10m', window: 600, max: 10 }, { scope: 'li1d', window: 86_400, max: 100 }],
  loginEmail: { scope: 'lie10m', window: 600, max: 8 },
  github: [{ scope: 'gh10m', window: 600, max: 20 }],
  google: [{ scope: 'gg10m', window: 600, max: 20 }],
};

/** Where GitHub lives. Production uses github.com; worker/dev.ts swaps in a local mock (an
 *  authorize path on this origin, and a fetch that answers the token and user calls itself). */
export interface GitHubEndpoints { authorize: string; token: string; api: string }
export const GITHUB: GitHubEndpoints = {
  authorize: 'https://github.com/login/oauth/authorize',
  token: 'https://github.com/login/oauth/access_token',
  api: 'https://api.github.com',
};

/** Where Google lives. Production uses Google's OpenID Connect endpoints; worker/dev.ts swaps in a
 *  local mock. `issuers` are the accepted `iss` values of the ID token. */
export interface GoogleEndpoints { authorize: string; token: string; issuers: string[] }
export const GOOGLE: GoogleEndpoints = {
  authorize: 'https://accounts.google.com/o/oauth2/v2/auth',
  token: 'https://oauth2.googleapis.com/token',
  issuers: ['https://accounts.google.com', 'accounts.google.com'],
};

const enc = new TextEncoder();
function b64u(bytes: ArrayBuffer | Uint8Array) {
  const u = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  let s = '';
  for (const b of u) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
function unb64u(s: string) {
  const t = s.replace(/-/g, '+').replace(/_/g, '/');
  const b = atob(t + '='.repeat((4 - (t.length % 4)) % 4));
  return Uint8Array.from(b, (c) => c.charCodeAt(0));
}
const randomB64u = (n: number) => b64u(crypto.getRandomValues(new Uint8Array(n)));
function sameBytes(a: Uint8Array, b: Uint8Array) {
  if (a.length !== b.length) return false;
  let d = 0;
  for (let i = 0; i < a.length; i++) d |= a[i] ^ b[i];
  return d === 0;
}
const sameText = (a: string, b: string) => sameBytes(enc.encode(a), enc.encode(b));

interface PwRecord { alg: 'pbkdf2-sha256'; iter: number; salt: string; hash: string }
async function pbkdf2(password: string, salt: Uint8Array, iter: number) {
  const k = await crypto.subtle.importKey('raw', enc.encode(password), 'PBKDF2', false, ['deriveBits']);
  return new Uint8Array(await crypto.subtle.deriveBits({ name: 'PBKDF2', hash: 'SHA-256', salt, iterations: iter }, k, 256));
}
async function hashPassword(password: string): Promise<string> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const hash = await pbkdf2(password, salt, AUTH.pbkdf2);
  return JSON.stringify({ alg: 'pbkdf2-sha256', iter: AUTH.pbkdf2, salt: b64u(salt), hash: b64u(hash) } satisfies PwRecord);
}
// Used when there is no stored password, so every sign-in costs one PBKDF2 run.
const DUMMY: PwRecord = { alg: 'pbkdf2-sha256', iter: 100_000, salt: 'b3h5bi1kdW1teS1zYWx0', hash: 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA' };
async function checkPassword(stored: string | null, password: string) {
  let rec = DUMMY;
  let real = false;
  if (stored) { try { rec = JSON.parse(stored) as PwRecord; real = true; } catch { rec = DUMMY; } }
  const got = await pbkdf2(password, unb64u(rec.salt), rec.iter);
  return sameBytes(got, unb64u(rec.hash)) && real;
}

interface UserRow { id: string; email: string; email_verified: number; name: string | null; avatar_url: string | null; password: string | null; created_at: string }

function cookies(req: Request) {
  const out: Record<string, string> = {};
  for (const part of (req.headers.get('cookie') || '').split(';')) {
    const i = part.indexOf('=');
    if (i > 0) out[part.slice(0, i).trim()] = part.slice(i + 1).trim();
  }
  return out;
}
const maxAge = () => AUTH.sessionDays * 86_400;
function sessionCookies(token: string) {
  return [
    `${AUTH.cookie}=${token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=${maxAge()}`,
    `${AUTH.hint}=1; Path=/; Secure; SameSite=Lax; Max-Age=${maxAge()}`,
  ];
}
const clearCookies = () => [
  `${AUTH.cookie}=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0`,
  `${AUTH.hint}=; Path=/; Secure; SameSite=Lax; Max-Age=0`,
];
function withCookies(res: Response, list: string[]) {
  const r = new Response(res.body, res);
  for (const c of list) r.headers.append('set-cookie', c);
  return r;
}

async function startSession(env: Env, userId: string, provider: string) {
  const token = randomB64u(32); // 256 bits
  const now = Math.floor(Date.now() / 1000);
  await env.DB.batch([
    env.DB.prepare('INSERT INTO sessions (token_hash, user_id, provider, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)')
      .bind(await sha256Hex(token), userId, provider, new Date().toISOString(), now + maxAge()),
    env.DB.prepare('UPDATE users SET last_login_at = ?1 WHERE id = ?2').bind(new Date().toISOString(), userId),
  ]);
  // Sign-ins by provider for the admin Activity tab (never fails the sign-in).
  await recordSignin(env, userId, provider);
  // Housekeeping: expired sessions go now and then.
  if (Math.random() < 0.05) await env.DB.prepare('DELETE FROM sessions WHERE expires_at < ?1').bind(now).run();
  return token;
}

/** The signed-in user for this request, or null. */
export async function currentUser(req: Request, env: Env): Promise<(UserRow & { token_hash: string; provider: string }) | null> {
  const token = cookies(req)[AUTH.cookie];
  if (!token || token.length < 40 || token.length > 64) return null;
  const row = await env.DB.prepare(
    'SELECT u.*, s.token_hash AS token_hash, s.provider AS provider FROM sessions s JOIN users u ON u.id = s.user_id WHERE s.token_hash = ?1 AND s.expires_at > ?2',
  ).bind(await sha256Hex(token), Math.floor(Date.now() / 1000)).first<UserRow & { token_hash: string; provider: string }>();
  return row ?? null;
}

async function rateLimit(env: Env, req: Request, rules: { scope: string; window: number; max: number }[], subject?: string) {
  const now = Math.floor(Date.now() / 1000);
  const who = subject ?? (req.headers.get('cf-connecting-ip') || 'local');
  const h = (await sha256Hex(`${env.RATE_SALT ?? 'oxyn-auth'}|${who}`)).slice(0, 32);
  const res = await env.DB.batch(rules.map((c) =>
    env.DB.prepare(
      'INSERT INTO rate_limits (key, window_start, count) VALUES (?1, ?2, 1) ' +
        'ON CONFLICT (key, window_start) DO UPDATE SET count = count + 1 RETURNING count',
    ).bind(`${c.scope}:${h}`, now - (now % c.window)),
  ));
  res.forEach((r, i) => {
    const n = Number((r.results?.[0] as { count?: number } | undefined)?.count ?? 0);
    if (n > rules[i].max) throw new HttpError(429, 'rate_limited', 'Too many attempts from here. Please wait a few minutes and try again.');
  });
}

async function readBody(req: Request): Promise<{ email: string; password: string }> {
  const type = req.headers.get('content-type') || '';
  if (!type.startsWith('application/json')) throw new HttpError(415, 'unsupported_media_type', 'Send application/json.');
  const len = Number(req.headers.get('content-length') || '0');
  if (len > 4096) throw new HttpError(413, 'too_large', 'That request is too large.');
  let b: Record<string, unknown>;
  try { b = await req.json(); } catch { throw new HttpError(400, 'bad_body', 'The request could not be read.'); }
  const str = (v: unknown) => (typeof v === 'string' ? v : '');
  return { email: str(b.email).trim().toLowerCase(), password: str(b.password) };
}

function validEmail(email: string) {
  if (!email || email.length > AUTH.email || !EMAIL_RE.test(email)) throw new HttpError(400, 'invalid_email', 'Enter a valid email address.', 'email');
}

const githubReady = (env: Env) => !!(env.GITHUB_OAUTH_CLIENT_ID && env.GITHUB_OAUTH_CLIENT_SECRET);
const googleReady = (env: Env) => !!(env.GOOGLE_OAUTH_CLIENT_ID && env.GOOGLE_OAUTH_CLIENT_SECRET);

/** Admin standing of a signed-in account. `admin`: the email is on the active admin list and a
 *  provider (Google or GitHub) verified it, so the admin panel opens. `verify`: on the list, but
 *  only an email + password account so far (nobody has checked that this person owns the email). */
export async function adminStanding(env: Env, u: { id: string; email: string; email_verified: number }): Promise<'admin' | 'verify' | null> {
  const row = await env.DB.prepare(
    'SELECT (SELECT 1 FROM admins WHERE email = ?1 AND removed_at IS NULL) AS listed, ' +
      '(SELECT COUNT(*) FROM oauth_accounts WHERE user_id = ?2) AS linked',
  ).bind(u.email, u.id).first<{ listed: number | null; linked: number }>();
  if (!row?.listed) return null;
  return u.email_verified && row.linked > 0 ? 'admin' : 'verify';
}

function publicUser(u: UserRow, providers: string[], session: string, standing: 'admin' | 'verify' | null) {
  return {
    admin_needs_verification: standing === 'verify',
    email: u.email,
    name: u.name,
    avatar_url: u.avatar_url,
    providers,
    has_password: !!u.password,
    session_provider: session,
    created_at: u.created_at,
  };
}

/** A relative path on this site to land on after signing in; anything else falls back to /account/. */
function safeNext(v: string | null) {
  if (!v || !v.startsWith('/') || v.startsWith('//') || v.startsWith('/\\') || v.startsWith('/api/') || v.length > 512) return '/account/';
  return v;
}

export interface Providers {
  github: GitHubEndpoints;
  google: GoogleEndpoints;
  fetch: typeof fetch; // used for the token and API calls of both providers
}
const realFetch: typeof fetch = (i, init) => fetch(i, init);

/** A browser prefetch (link rel=prefetch, speculation rules) of a sign-in start link must not start
 *  a sign-in: it would replace the state cookie of the real click. */
const isPrefetch = (req: Request) => /prefetch|prerender/i.test(`${req.headers.get('sec-purpose') || ''} ${req.headers.get('purpose') || ''}`);
const PREFETCH_SKIP = () => new Response(null, { status: 204, headers: { 'cache-control': 'no-store' } });

export function makeAuthHandler(p: Providers = { github: GITHUB, google: GOOGLE, fetch: realFetch }) {
  return async function handleAuth(req: Request, env: Env, _ctx: Ctx, url: URL): Promise<Response> {
    const route = url.pathname.replace(/\/+$/, '');
    try {
      if ((route === '/api/auth/github/start' || route === '/api/auth/google/start') && isPrefetch(req)) return PREFETCH_SKIP();
      if (route === '/api/auth/github/start') return await githubStart(req, env, url, p.github);
      if (route === '/api/auth/github/callback') return await githubCallback(req, env, url, p.github, p.fetch);
      if (route === '/api/auth/google/start') return await googleStart(req, env, url, p.google);
      if (route === '/api/auth/google/callback') return await googleCallback(req, env, url, p.google, p.fetch);
      if (route === '/api/auth/me') {
        if (req.method !== 'GET' && req.method !== 'HEAD') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
        return await me(req, env);
      }
      const posts: Record<string, (r: Request, e: Env) => Promise<Response>> = {
        '/api/auth/signup': signup,
        '/api/auth/login': login,
        '/api/auth/logout': logout,
        '/api/auth/logout-all': logoutAll,
      };
      const fn = posts[route];
      if (!fn) throw new HttpError(404, 'not_found', 'No such API route.');
      if (req.method !== 'POST') throw new HttpError(405, 'method_not_allowed', 'Use POST.');
      if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Requests must come from the Oxyn site.');
      return await fn(req, env);
    } catch (err) {
      if (err instanceof HttpError) return fail(err);
      console.error('auth error', err);
      return fail(new HttpError(500, 'internal', 'Something went wrong on our side. Please try again.'));
    }
  };
}
export const handleAuth = makeAuthHandler();

async function providersOf(env: Env, userId: string) {
  const r = await env.DB.prepare('SELECT provider FROM oauth_accounts WHERE user_id = ?1').bind(userId).all<{ provider: string }>();
  return (r.results ?? []).map((x) => x.provider);
}

async function me(req: Request, env: Env) {
  const u = await currentUser(req, env);
  const github = githubReady(env);
  const google = googleReady(env);
  if (!u) {
    // A stale hint cookie (expired or revoked session) is cleared so the page stops asking.
    const res = json({ ok: true, signed_in: false, github, google, is_admin: false });
    return cookies(req)[AUTH.hint] || cookies(req)[AUTH.cookie] ? withCookies(res, clearCookies()) : res;
  }
  const providers = await providersOf(env, u.id);
  if (u.password) providers.unshift('password');
  const standing = await adminStanding(env, u);
  return json({ ok: true, signed_in: true, github, google, is_admin: standing === 'admin', user: publicUser(u, providers, u.provider, standing) });
}

const SIGNUP_FAIL = 'We could not create an account with that email and password. If you already have one, sign in instead.';
const LOGIN_FAIL = 'That email and password do not match an account.';

async function signup(req: Request, env: Env) {
  const { email, password } = await readBody(req);
  validEmail(email);
  if (password.length < AUTH.passwordMin) throw new HttpError(400, 'weak_password', `Use at least ${AUTH.passwordMin} characters.`, 'password');
  if (password.length > AUTH.passwordMax) throw new HttpError(400, 'long_password', `Use at most ${AUTH.passwordMax} characters.`, 'password');
  await rateLimit(env, req, AUTH.signup);
  const existing = await env.DB.prepare('SELECT * FROM users WHERE email = ?1').bind(email).first<UserRow>();
  if (existing) {
    // The same email and password simply signs in; anything else gets the same answer a bad
    // address would, after the same PBKDF2 cost.
    if (await checkPassword(existing.password, password)) {
      const token = await startSession(env, existing.id, 'password');
      return withCookies(json({ ok: true, created: false }), sessionCookies(token));
    }
    throw new HttpError(400, 'signup_failed', SIGNUP_FAIL);
  }
  const id = crypto.randomUUID();
  const pw = await hashPassword(password);
  try {
    await env.DB.prepare('INSERT INTO users (id, email, email_verified, password, created_at) VALUES (?1, ?2, 0, ?3, ?4)')
      .bind(id, email, pw, new Date().toISOString()).run();
  } catch {
    throw new HttpError(400, 'signup_failed', SIGNUP_FAIL); // lost a race with the same email
  }
  const token = await startSession(env, id, 'password');
  return withCookies(json({ ok: true, created: true }, 201), sessionCookies(token));
}

async function login(req: Request, env: Env) {
  const { email, password } = await readBody(req);
  if (!email || !password || email.length > AUTH.email || password.length > AUTH.passwordMax) throw new HttpError(400, 'login_failed', LOGIN_FAIL);
  await rateLimit(env, req, AUTH.login);
  await rateLimit(env, req, [AUTH.loginEmail], `email:${email}`);
  const u = await env.DB.prepare('SELECT * FROM users WHERE email = ?1').bind(email).first<UserRow>();
  const ok = await checkPassword(u?.password ?? null, password);
  if (!u || !ok) throw new HttpError(401, 'login_failed', LOGIN_FAIL);
  const token = await startSession(env, u.id, 'password');
  return withCookies(json({ ok: true }), sessionCookies(token));
}

async function logout(req: Request, env: Env) {
  const token = cookies(req)[AUTH.cookie];
  if (token) await env.DB.prepare('DELETE FROM sessions WHERE token_hash = ?1').bind(await sha256Hex(token)).run();
  return withCookies(json({ ok: true }), clearCookies());
}

async function logoutAll(req: Request, env: Env) {
  const u = await currentUser(req, env);
  if (!u) throw new HttpError(401, 'signed_out', 'You are not signed in.');
  const r = await env.DB.prepare('DELETE FROM sessions WHERE user_id = ?1').bind(u.id).run();
  return withCookies(json({ ok: true, ended: Number(r.meta?.changes ?? 0) }), clearCookies());
}

// ── GitHub OAuth (authorization code flow with a CSRF state cookie) ───────────────────────────

function callbackUrl(url: URL) { return `${url.origin}/api/auth/github/callback`; }
const toLogin = (url: URL, code: string, extra: string[] = []) =>
  new Response(null, { status: 302, headers: [['location', `${url.origin}/login/?error=${code}`], ['cache-control', 'no-store'], ...extra.map((c) => ['set-cookie', c] as [string, string])] });
const clearState = `${AUTH.stateCookie}=; Path=/api/auth/github; HttpOnly; Secure; SameSite=Lax; Max-Age=0`;

async function githubStart(req: Request, env: Env, url: URL, gh: GitHubEndpoints) {
  if (req.method !== 'GET' && req.method !== 'HEAD') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
  if (!githubReady(env)) throw new HttpError(503, 'github_unavailable', 'GitHub sign-in is coming soon. Use your email for now.');
  await rateLimit(env, req, AUTH.github);
  const state = randomB64u(32);
  const next = safeNext(url.searchParams.get('next'));
  const q = new URLSearchParams({
    client_id: env.GITHUB_OAUTH_CLIENT_ID!,
    redirect_uri: callbackUrl(url),
    scope: 'read:user user:email',
    state,
    allow_signup: 'true',
  });
  return new Response(null, {
    status: 302,
    headers: [
      ['location', `${gh.authorize.startsWith('/') ? url.origin : ''}${gh.authorize}?${q}`],
      ['cache-control', 'no-store'],
      // The state is bound to this browser; `next` rides along in the same cookie.
      ['set-cookie', `${AUTH.stateCookie}=${state}.${b64u(enc.encode(next))}; Path=/api/auth/github; HttpOnly; Secure; SameSite=Lax; Max-Age=600`],
    ],
  });
}

interface GhUser { id: number; login: string; name: string | null; avatar_url: string | null }
interface GhEmail { email: string; primary: boolean; verified: boolean }

async function githubCallback(req: Request, env: Env, url: URL, gh: GitHubEndpoints, fetch: typeof globalThis.fetch) {
  if (req.method !== 'GET') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
  if (!githubReady(env)) throw new HttpError(503, 'github_unavailable', 'GitHub sign-in is coming soon. Use your email for now.');
  const raw = cookies(req)[AUTH.stateCookie] || '';
  const [want, nextEnc] = raw.split('.');
  const got = url.searchParams.get('state') || '';
  if (!want || !got || !sameText(want, got)) return toLogin(url, 'github_state', [clearState]);
  let next = '/account/';
  try { next = safeNext(new TextDecoder().decode(unb64u(nextEnc || ''))); } catch { /* default */ }
  if (url.searchParams.get('error')) return toLogin(url, 'github_denied', [clearState]);
  const code = url.searchParams.get('code');
  if (!code || code.length > 200) return toLogin(url, 'github_failed', [clearState]);

  const ua = { 'user-agent': 'oxyn-site-worker', accept: 'application/json' };
  const tr = await fetch(gh.token, {
    method: 'POST',
    headers: { ...ua, 'content-type': 'application/json' },
    body: JSON.stringify({ client_id: env.GITHUB_OAUTH_CLIENT_ID, client_secret: env.GITHUB_OAUTH_CLIENT_SECRET, code, redirect_uri: callbackUrl(url) }),
  });
  const tj = (await tr.json().catch(() => ({}))) as { access_token?: string };
  if (!tr.ok || !tj.access_token) return toLogin(url, 'github_failed', [clearState]);
  const api = { ...ua, accept: 'application/vnd.github+json', authorization: `Bearer ${tj.access_token}`, 'x-github-api-version': '2022-11-28' };
  const [ur, er] = await Promise.all([fetch(`${gh.api}/user`, { headers: api }), fetch(`${gh.api}/user/emails`, { headers: api })]);
  if (!ur.ok || !er.ok) return toLogin(url, 'github_failed', [clearState]);
  const g = (await ur.json()) as GhUser;
  const emails = (await er.json()) as GhEmail[];
  const primary = Array.isArray(emails) ? emails.find((e) => e.primary && e.verified) ?? emails.find((e) => e.verified) : undefined;
  if (!g?.id || !primary) return toLogin(url, 'github_email', [clearState]);
  const email = primary.email.trim().toLowerCase();
  const pid = String(g.id);
  const avatar = typeof g.avatar_url === 'string' && g.avatar_url.startsWith('https://') ? g.avatar_url.slice(0, 500) : null;
  const name = (g.name || g.login || '').slice(0, 100) || null;

  const userId = await linkProvider(env, { provider: 'github', pid, email, login: g.login, name, avatar });
  const token = await startSession(env, userId, 'github');
  return signedInRedirect(url, next, token, clearState);
}

// ── Account linking (shared by GitHub and Google) ─────────────────────────────────────────────

interface ProviderUser { provider: 'github' | 'google'; pid: string; email: string; login: string | null; name: string | null; avatar: string | null }

/** Signs a provider identity in: the account it is linked to, else the account with the same
 *  (provider-verified) email, else a new account. Returns the user id. */
async function linkProvider(env: Env, p: ProviderUser): Promise<string> {
  const now = new Date().toISOString();
  const linked = await env.DB.prepare('SELECT user_id FROM oauth_accounts WHERE provider = ?1 AND provider_id = ?2').bind(p.provider, p.pid).first<{ user_id: string }>();
  if (linked) {
    await env.DB.batch([
      env.DB.prepare('UPDATE oauth_accounts SET login = ?1 WHERE provider = ?2 AND provider_id = ?3').bind(p.login, p.provider, p.pid),
      env.DB.prepare('UPDATE users SET name = COALESCE(?1, name), avatar_url = COALESCE(?2, avatar_url), email_verified = 1 WHERE id = ?3').bind(p.name, p.avatar, linked.user_id),
    ]);
    return linked.user_id;
  }
  const byEmail = await env.DB.prepare('SELECT * FROM users WHERE email = ?1').bind(p.email).first<UserRow>();
  if (byEmail) {
    // The provider verified this email; ours never did. If the account was only ever a password,
    // the password may have been set by someone else, so it is dropped and its sessions end.
    const stmts = [
      env.DB.prepare('INSERT INTO oauth_accounts (provider, provider_id, user_id, login, created_at) VALUES (?1, ?2, ?3, ?4, ?5)').bind(p.provider, p.pid, byEmail.id, p.login, now),
      env.DB.prepare('UPDATE users SET name = COALESCE(name, ?1), avatar_url = COALESCE(?2, avatar_url), email_verified = 1 WHERE id = ?3').bind(p.name, p.avatar, byEmail.id),
    ];
    if (!byEmail.email_verified) {
      stmts.push(env.DB.prepare('UPDATE users SET password = NULL WHERE id = ?1').bind(byEmail.id));
      stmts.push(env.DB.prepare('DELETE FROM sessions WHERE user_id = ?1').bind(byEmail.id));
    }
    await env.DB.batch(stmts);
    return byEmail.id;
  }
  const id = crypto.randomUUID();
  await env.DB.batch([
    env.DB.prepare('INSERT INTO users (id, email, email_verified, name, avatar_url, created_at) VALUES (?1, ?2, 1, ?3, ?4, ?5)').bind(id, p.email, p.name, p.avatar, now),
    env.DB.prepare('INSERT INTO oauth_accounts (provider, provider_id, user_id, login, created_at) VALUES (?1, ?2, ?3, ?4, ?5)').bind(p.provider, p.pid, id, p.login, now),
  ]);
  return id;
}

function signedInRedirect(url: URL, next: string, token: string, clear: string) {
  return new Response(null, {
    status: 302,
    headers: [['location', `${url.origin}${next}`], ['cache-control', 'no-store'], ['set-cookie', clear], ...sessionCookies(token).map((c) => ['set-cookie', c] as [string, string])],
  });
}

// ── Google (OpenID Connect: authorization code flow + PKCE S256 + nonce) ─────────────────────
//
// The browser gets a 10-minute HttpOnly cookie scoped to /api/auth/google holding the `state`,
// the `nonce`, the PKCE verifier and `next`. The callback compares `state` in constant time,
// exchanges the code with the verifier and the client secret, and reads the ID token that the
// token endpoint returns over TLS (OpenID Connect Core 3.1.3.7: a token received directly from
// the token endpoint may rely on TLS instead of its signature). Its iss, aud, exp, iat and nonce
// are checked, and only email_verified = true is accepted.

const googleCallbackUrl = (url: URL) => `${url.origin}/api/auth/google/callback`;
const clearGoogle = `${AUTH.googleCookie}=; Path=/api/auth/google; HttpOnly; Secure; SameSite=Lax; Max-Age=0`;
const GOOGLE_OFF = 'Google sign-in is coming soon. Use your email for now.';

async function googleStart(req: Request, env: Env, url: URL, g: GoogleEndpoints) {
  if (req.method !== 'GET' && req.method !== 'HEAD') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
  if (!googleReady(env)) throw new HttpError(503, 'google_unavailable', GOOGLE_OFF);
  await rateLimit(env, req, AUTH.google);
  const state = randomB64u(32);
  const nonce = randomB64u(32);
  const verifier = randomB64u(48); // 64 characters, inside RFC 7636's 43..128
  const challenge = b64u(await crypto.subtle.digest('SHA-256', enc.encode(verifier)));
  const next = safeNext(url.searchParams.get('next'));
  const q = new URLSearchParams({
    client_id: env.GOOGLE_OAUTH_CLIENT_ID!,
    redirect_uri: googleCallbackUrl(url),
    response_type: 'code',
    scope: 'openid email profile',
    state,
    nonce,
    code_challenge: challenge,
    code_challenge_method: 'S256',
    prompt: 'select_account',
  });
  return new Response(null, {
    status: 302,
    headers: [
      ['location', `${g.authorize.startsWith('/') ? url.origin : ''}${g.authorize}?${q}`],
      ['cache-control', 'no-store'],
      ['set-cookie', `${AUTH.googleCookie}=${state}.${nonce}.${verifier}.${b64u(enc.encode(next))}; Path=/api/auth/google; HttpOnly; Secure; SameSite=Lax; Max-Age=600`],
    ],
  });
}

interface IdClaims { iss?: string; aud?: string | string[]; azp?: string; sub?: string; exp?: number; iat?: number; nonce?: string; email?: string; email_verified?: boolean | string; name?: string; picture?: string }

function readIdToken(tok: unknown): IdClaims | null {
  if (typeof tok !== 'string' || tok.length > 8192) return null;
  const parts = tok.split('.');
  if (parts.length !== 3) return null;
  try { return JSON.parse(new TextDecoder().decode(unb64u(parts[1]))) as IdClaims; } catch { return null; }
}

async function googleCallback(req: Request, env: Env, url: URL, g: GoogleEndpoints, fetch: typeof globalThis.fetch) {
  if (req.method !== 'GET') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
  if (!googleReady(env)) throw new HttpError(503, 'google_unavailable', GOOGLE_OFF);
  const back = (code: string) => toLogin(url, code, [clearGoogle]);
  const [want, nonce, verifier, nextEnc] = (cookies(req)[AUTH.googleCookie] || '').split('.');
  const got = url.searchParams.get('state') || '';
  if (!want || !nonce || !verifier || !got || !sameText(want, got)) return back('google_state');
  let next = '/account/';
  try { next = safeNext(new TextDecoder().decode(unb64u(nextEnc || ''))); } catch { /* default */ }
  if (url.searchParams.get('error')) return back('google_denied');
  const code = url.searchParams.get('code');
  if (!code || code.length > 2048) return back('google_failed');

  const tr = await fetch(g.token, {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded', accept: 'application/json', 'user-agent': 'oxyn-site-worker' },
    body: new URLSearchParams({
      grant_type: 'authorization_code',
      code,
      client_id: env.GOOGLE_OAUTH_CLIENT_ID!,
      client_secret: env.GOOGLE_OAUTH_CLIENT_SECRET!,
      redirect_uri: googleCallbackUrl(url),
      code_verifier: verifier,
    }).toString(),
  });
  const tj = (await tr.json().catch(() => ({}))) as { id_token?: string };
  const c = tr.ok ? readIdToken(tj.id_token) : null;
  if (!c) return back('google_failed');
  const t = Math.floor(Date.now() / 1000);
  const auds = Array.isArray(c.aud) ? c.aud : [c.aud];
  const valid =
    g.issuers.includes(c.iss || '') &&
    auds.includes(env.GOOGLE_OAUTH_CLIENT_ID) && (auds.length === 1 || c.azp === env.GOOGLE_OAUTH_CLIENT_ID) &&
    typeof c.exp === 'number' && c.exp > t - 60 &&
    (typeof c.iat !== 'number' || c.iat < t + 300) &&
    typeof c.nonce === 'string' && sameText(c.nonce, nonce) &&
    typeof c.sub === 'string' && c.sub.length > 0 && c.sub.length <= 255;
  if (!valid) return back('google_failed');
  const email = (c.email || '').trim().toLowerCase();
  if (!email || email.length > AUTH.email || !EMAIL_RE.test(email) || !(c.email_verified === true || c.email_verified === 'true')) return back('google_email');
  const avatar = typeof c.picture === 'string' && c.picture.startsWith('https://') ? c.picture.slice(0, 500) : null;
  const name = (typeof c.name === 'string' ? c.name : '').slice(0, 100) || null;
  const userId = await linkProvider(env, { provider: 'google', pid: c.sub!, email, login: email, name, avatar });
  const token = await startSession(env, userId, 'google');
  return signedInRedirect(url, next, token, clearGoogle);
}
