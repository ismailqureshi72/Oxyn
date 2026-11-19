// Site accounts (L2): sign in with GitHub or with an email and a password. Separate from the
// maintainers' admin panel (admin.ts). Pages are static (/login/, /signup/, /account/); they and
// the navbar menu call these routes from the browser (public/account.js), so nothing here touches
// the static cache. See docs/auth.md.
//
//   GET  /api/auth/me                 who is signed in ({signed_in:false} when nobody), providers
//   POST /api/auth/signup             {email, password}           -> session cookie
//   POST /api/auth/login              {email, password}           -> session cookie
//   POST /api/auth/logout             ends this session
//   POST /api/auth/logout-all         ends every session of this account
//   GET  /api/auth/github/start       redirect to GitHub (503 JSON until the OAuth app is set up)
//   GET  /api/auth/github/callback    GitHub redirects back here
//
// Sessions: a random 256-bit token in an HttpOnly, Secure, SameSite=Lax cookie; D1 keeps only its
// SHA-256, so signing out (or "sign out everywhere") is a row delete. Passwords: PBKDF2-SHA256,
// 100 000 iterations, 16-byte salt. Every failure on sign-in reads the same, and a sign-in for an
// unknown email still runs PBKDF2, so responses do not tell which emails have accounts.

import { EMAIL_RE, HttpError, fail, json, sameOrigin, sha256Hex, type Ctx, type Env } from './shared';

export const AUTH = {
  cookie: 'oxyn_session',
  hint: 'oxyn_signed_in', // readable by the page: "a session may exist, ask /api/auth/me"
  stateCookie: 'oxyn_gh_state',
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
};

/** Where GitHub lives. Production uses github.com; worker/dev.ts swaps in a local mock (an
 *  authorize path on this origin, and a fetch that answers the token and user calls itself). */
export interface GitHubEndpoints { authorize: string; token: string; api: string }
export const GITHUB: GitHubEndpoints = {
  authorize: 'https://github.com/login/oauth/authorize',
  token: 'https://github.com/login/oauth/access_token',
  api: 'https://api.github.com',
};

const enc = new TextEncoder();
function b64u(bytes: ArrayBuffer | Uint8Array) {
  const u = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  let s = '';
  for (const b of u) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
function unb64u(s: string) {
  const b = atob(s.replace(/-/g, '+').replace(/_/g, '/'));
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

function publicUser(u: UserRow, providers: string[], session: string) {
  return {
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
  if (!v || !v.startsWith('/') || v.startsWith('//') || v.startsWith('/\\') || v.startsWith('/api/') || v.startsWith('/admin') || v.length > 512) return '/account/';
  return v;
}

export function makeAuthHandler(gh: GitHubEndpoints, fetchGh: typeof fetch = (i, init) => fetch(i, init)) {
  return async function handleAuth(req: Request, env: Env, _ctx: Ctx, url: URL): Promise<Response> {
    const route = url.pathname.replace(/\/+$/, '');
    try {
      if (route === '/api/auth/github/start') return await githubStart(req, env, url, gh);
      if (route === '/api/auth/github/callback') return await githubCallback(req, env, url, gh, fetchGh);
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
export const handleAuth = makeAuthHandler(GITHUB);

async function providersOf(env: Env, userId: string) {
  const r = await env.DB.prepare('SELECT provider FROM oauth_accounts WHERE user_id = ?1').bind(userId).all<{ provider: string }>();
  return (r.results ?? []).map((x) => x.provider);
}

async function me(req: Request, env: Env) {
  const u = await currentUser(req, env);
  const github = githubReady(env);
  if (!u) {
    // A stale hint cookie (expired or revoked session) is cleared so the page stops asking.
    const res = json({ ok: true, signed_in: false, github });
    return cookies(req)[AUTH.hint] || cookies(req)[AUTH.cookie] ? withCookies(res, clearCookies()) : res;
  }
  const providers = await providersOf(env, u.id);
  if (u.password) providers.unshift('password');
  return json({ ok: true, signed_in: true, github, user: publicUser(u, providers, u.provider) });
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
  const now = new Date().toISOString();
  const avatar = typeof g.avatar_url === 'string' && g.avatar_url.startsWith('https://') ? g.avatar_url.slice(0, 500) : null;
  const name = (g.name || g.login || '').slice(0, 100) || null;

  let userId: string;
  const linked = await env.DB.prepare('SELECT user_id FROM oauth_accounts WHERE provider = ?1 AND provider_id = ?2').bind('github', pid).first<{ user_id: string }>();
  if (linked) {
    userId = linked.user_id;
    await env.DB.batch([
      env.DB.prepare('UPDATE oauth_accounts SET login = ?1 WHERE provider = ?2 AND provider_id = ?3').bind(g.login, 'github', pid),
      env.DB.prepare('UPDATE users SET name = ?1, avatar_url = ?2 WHERE id = ?3').bind(name, avatar, userId),
    ]);
  } else {
    const byEmail = await env.DB.prepare('SELECT * FROM users WHERE email = ?1').bind(email).first<UserRow>();
    if (byEmail) {
      userId = byEmail.id;
      // GitHub verified this email; ours never did. If the account was only ever a password, the
      // password may have been set by someone else, so it is dropped and its sessions end.
      const stmts = [
        env.DB.prepare('INSERT INTO oauth_accounts (provider, provider_id, user_id, login, created_at) VALUES (?1, ?2, ?3, ?4, ?5)').bind('github', pid, userId, g.login, now),
        env.DB.prepare('UPDATE users SET name = COALESCE(name, ?1), avatar_url = ?2, email_verified = 1 WHERE id = ?3').bind(name, avatar, userId),
      ];
      if (!byEmail.email_verified) {
        stmts.push(env.DB.prepare('UPDATE users SET password = NULL WHERE id = ?1').bind(userId));
        stmts.push(env.DB.prepare('DELETE FROM sessions WHERE user_id = ?1').bind(userId));
      }
      await env.DB.batch(stmts);
    } else {
      userId = crypto.randomUUID();
      await env.DB.batch([
        env.DB.prepare('INSERT INTO users (id, email, email_verified, name, avatar_url, created_at) VALUES (?1, ?2, 1, ?3, ?4, ?5)').bind(userId, email, name, avatar, now),
        env.DB.prepare('INSERT INTO oauth_accounts (provider, provider_id, user_id, login, created_at) VALUES (?1, ?2, ?3, ?4, ?5)').bind('github', pid, userId, g.login, now),
      ]);
    }
  }
  const token = await startSession(env, userId, 'github');
  return new Response(null, {
    status: 302,
    headers: [['location', `${url.origin}${next}`], ['cache-control', 'no-store'], ['set-cookie', clearState], ...sessionCookies(token).map((c) => ['set-cookie', c] as [string, string])],
  });
}
