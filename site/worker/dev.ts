// LOCAL DEVELOPMENT ONLY. Never deployed: wrangler.jsonc's `main` is worker/index.ts, and this
// file is only an entry point when named on the command line:
//
//   npx wrangler dev worker/dev.ts --var GOOGLE_OAUTH_CLIENT_ID:dev --var GOOGLE_OAUTH_CLIENT_SECRET:dev \
//     --var GITHUB_OAUTH_CLIENT_ID:dev --var GITHUB_OAUTH_CLIENT_SECRET:dev          (from site/)
//
// Two fake sign-in providers, both answering only on localhost / 127.0.0.1. Everything after the
// provider (state / nonce / PKCE checks, account linking, sessions, the admin panel) is production
// code, so the admin panel is tested by signing in with the fake Google or GitHub on an email in
// the local `admins` table.
// - Google: "Continue with Google" goes to /__dev/google/authorize, which signs in a test user
//   (pick one with ?as=<base64url JSON {sub,email,email_verified,name}> or the same value in a
//   `dev_google_as` cookie, so a browser test can choose before clicking; or ?deny=1). Its token
//   endpoint checks the PKCE verifier and returns an unsigned ID token; ?as can also override the
//   token's iss / aud / nonce / exp to test that the callback refuses them.
// - GitHub: /__dev/github/authorize (?as=<base64url JSON {id,login,email,verified,name}>, ?deny=1).
// CI fails if either stub ever shows up in the deploy bundle (see .github/workflows/ci.yml,
// "dev stubs are not deployed").
import worker from './index';
import { HttpError, fail, json, type Ctx, type Env } from './shared';
import { makeAuthHandler } from './auth';

const local = (req: Request) => ['localhost', '127.0.0.1'].includes(new URL(req.url).hostname);
const enc64 = (o: unknown) => btoa(JSON.stringify(o)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
const dec64 = <T>(s: string): T => JSON.parse(atob(s.replace(/-/g, '+').replace(/_/g, '/')));
const b64u = (b: ArrayBuffer) => btoa(String.fromCharCode(...new Uint8Array(b))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');

// ── Fake GitHub (OXYN_DEV_GITHUB_STUB) ─────────────────────────────────────────────────────────
const FAKE_GH = 'https://fake-github.oxyn.invalid';
interface FakeGhUser { id: number; login: string; email: string; verified?: boolean; name?: string }
const GH_USER: FakeGhUser = { id: 1001, login: 'octo-dev', email: 'octo@example.com', verified: true, name: 'Octo Dev' };

function fakeGithubAuthorize(url: URL) {
  const back = new URL(url.searchParams.get('redirect_uri') || '/api/auth/github/callback', url.origin);
  if (back.origin !== url.origin) return json({ error: 'OXYN_DEV_GITHUB_STUB: redirect_uri must be this origin' }, 400);
  const state = url.searchParams.get('state') || '';
  if (url.searchParams.get('deny')) back.search = new URLSearchParams({ error: 'access_denied', state }).toString();
  else {
    let user = GH_USER;
    try { if (url.searchParams.get('as')) user = { ...GH_USER, ...dec64<FakeGhUser>(url.searchParams.get('as')!) }; } catch { /* default */ }
    back.search = new URLSearchParams({ code: 'mock.' + enc64(user), state }).toString();
  }
  return new Response(null, { status: 302, headers: { location: back.toString() } });
}

async function fakeGithub(u: URL, init?: RequestInit) {
  if (u.pathname === '/login/oauth/access_token') {
    const b = JSON.parse(String(init?.body || '{}')) as { code?: string };
    if (!b.code?.startsWith('mock.')) return json({ error: 'bad_verification_code' }, 200);
    return json({ access_token: b.code.slice(5), token_type: 'bearer', scope: 'read:user,user:email' });
  }
  const auth = new Headers(init?.headers).get('authorization') || '';
  let user: FakeGhUser;
  try { user = dec64<FakeGhUser>(auth.replace(/^Bearer /, '')); } catch { return json({ message: 'Bad credentials' }, 401); }
  if (u.pathname === '/user') return json({ id: user.id, login: user.login, name: user.name ?? null, avatar_url: `https://avatars.githubusercontent.com/u/${user.id}?v=4` });
  if (u.pathname === '/user/emails') return json([{ email: user.email, primary: true, verified: user.verified !== false }]);
  return json({ message: 'Not Found' }, 404);
}

// ── Fake Google (OXYN_DEV_GOOGLE_STUB) ─────────────────────────────────────────────────────────
const FAKE_GG = 'https://fake-google.oxyn.invalid';
interface FakeGgUser {
  sub: string; email: string; email_verified?: boolean; name?: string;
  // test overrides for the ID token
  iss?: string; aud?: string; nonce?: string; exp?: number;
}
const GG_USER: FakeGgUser = { sub: '2001', email: 'gina@example.com', email_verified: true, name: 'Gina Dev' };
interface FakeCode { user: FakeGgUser; client_id: string; challenge: string; method: string; nonce: string; redirect_uri: string }

function fakeGoogleAuthorize(req: Request, url: URL) {
  const q = url.searchParams;
  const back = new URL(q.get('redirect_uri') || '/api/auth/google/callback', url.origin);
  if (back.origin !== url.origin) return json({ error: 'OXYN_DEV_GOOGLE_STUB: redirect_uri must be this origin' }, 400);
  if (q.get('response_type') !== 'code' || !q.get('code_challenge') || q.get('code_challenge_method') !== 'S256' || !q.get('nonce'))
    return json({ error: 'OXYN_DEV_GOOGLE_STUB: expected response_type=code, PKCE S256 and a nonce' }, 400);
  const state = q.get('state') || '';
  if (q.get('deny')) back.search = new URLSearchParams({ error: 'access_denied', state }).toString();
  else {
    let user = GG_USER;
    const as = q.get('as') || /(?:^|;\s*)dev_google_as=([^;]+)/.exec(req.headers.get('cookie') || '')?.[1];
    try { if (as) user = { ...GG_USER, ...dec64<FakeGgUser>(as) }; } catch { /* default */ }
    const code: FakeCode = { user, client_id: q.get('client_id') || '', challenge: q.get('code_challenge')!, method: 'S256', nonce: q.get('nonce')!, redirect_uri: back.origin + back.pathname };
    back.search = new URLSearchParams({ code: 'gmock.' + enc64(code), state }).toString();
  }
  return new Response(null, { status: 302, headers: { location: back.toString() } });
}

async function fakeGoogle(u: URL, init?: RequestInit) {
  if (u.pathname !== '/token') return json({ error: 'not_found' }, 404);
  const f = new URLSearchParams(String(init?.body || ''));
  const raw = f.get('code') || '';
  if (!raw.startsWith('gmock.')) return json({ error: 'invalid_grant' }, 400);
  let c: FakeCode;
  try { c = dec64<FakeCode>(raw.slice(6)); } catch { return json({ error: 'invalid_grant' }, 400); }
  if (f.get('grant_type') !== 'authorization_code' || f.get('client_id') !== c.client_id || !f.get('client_secret') || f.get('redirect_uri') !== c.redirect_uri)
    return json({ error: 'invalid_client' }, 401);
  const verifier = f.get('code_verifier') || '';
  const want = b64u(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier)));
  if (!verifier || want !== c.challenge) return json({ error: 'invalid_grant', error_description: 'PKCE verification failed' }, 400);
  const t = Math.floor(Date.now() / 1000);
  const u2 = c.user;
  const claims = {
    iss: u2.iss ?? 'https://accounts.google.com', aud: u2.aud ?? c.client_id, sub: u2.sub, email: u2.email,
    email_verified: u2.email_verified !== false, name: u2.name, nonce: u2.nonce ?? c.nonce, iat: t, exp: u2.exp ?? t + 3600,
  };
  return json({ access_token: 'fake', token_type: 'Bearer', expires_in: 3600, id_token: `${enc64({ alg: 'none', typ: 'JWT' })}.${enc64(claims)}.sig` });
}

const fakeFetch: typeof fetch = async (input, init) => {
  const u = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url);
  if (u.origin === FAKE_GH) return fakeGithub(u, init);
  if (u.origin === FAKE_GG) return fakeGoogle(u, init);
  throw new Error('OXYN_DEV_GITHUB_STUB / OXYN_DEV_GOOGLE_STUB: unexpected fetch ' + u.href);
};

const devAuth = makeAuthHandler({
  github: { authorize: '/__dev/github/authorize', token: `${FAKE_GH}/login/oauth/access_token`, api: FAKE_GH },
  google: { authorize: '/__dev/google/authorize', token: `${FAKE_GG}/token`, issuers: ['https://accounts.google.com', 'accounts.google.com'] },
  fetch: fakeFetch,
});

export default {
  fetch(req: Request, env: Env, ctx: Ctx) {
    const url = new URL(req.url);
    const p = url.pathname;
    if (p.startsWith('/api/auth/') || p.startsWith('/__dev/')) {
      if (!local(req)) return fail(new HttpError(403, 'dev_only', 'OXYN_DEV_GOOGLE_STUB: localhost only.'));
      if (p === '/__dev/github/authorize') return Promise.resolve(fakeGithubAuthorize(url));
      if (p === '/__dev/google/authorize') return Promise.resolve(fakeGoogleAuthorize(req, url));
      if (p.startsWith('/__dev/')) return Promise.resolve(json({ error: 'not found' }, 404));
      return devAuth(req, env, ctx, url);
    }
    return worker.fetch(req, env, ctx);
  },
};
