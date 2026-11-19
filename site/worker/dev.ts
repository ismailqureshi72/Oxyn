// LOCAL DEVELOPMENT ONLY. Never deployed: wrangler.jsonc's `main` is worker/index.ts, and this
// file is only an entry point when named on the command line:
//
//   npx wrangler dev worker/dev.ts --var DEV_ADMIN_EMAIL:you@example.com      (from site/)
//
// Two stubs, both answering only on localhost / 127.0.0.1:
// - Admin: swaps the Cloudflare Access JWT check for a stub that trusts DEV_ADMIN_EMAIL. Everything
//   after the Access check (admin list, shared password, sessions) is the production code.
// - GitHub sign-in: with --var GITHUB_OAUTH_CLIENT_ID:dev --var GITHUB_OAUTH_CLIENT_SECRET:dev,
//   "Continue with GitHub" goes to a fake GitHub at /__dev/github/authorize that signs in a test
//   user (pick one with ?as=<base64url JSON {id,login,email,verified,name}>, or ?deny=1). The
//   callback, account linking and sessions are the production code.
// CI fails if either stub ever shows up in the deploy bundle (see .github/workflows/ci.yml,
// "dev stubs are not deployed").
import worker from './index';
import { HttpError, fail, json, type Ctx, type Env } from './shared';
import { makeAdminHandler } from './admin';
import { makeAuthHandler } from './auth';

const local = (req: Request) => ['localhost', '127.0.0.1'].includes(new URL(req.url).hostname);

const devAdmin = makeAdminHandler(async (req, env) => {
  const email = ((env as Env & { DEV_ADMIN_EMAIL?: string }).DEV_ADMIN_EMAIL || '').trim().toLowerCase();
  if (!local(req) || !email) throw new HttpError(401, 'access_required', 'OXYN_DEV_ACCESS_STUB: set DEV_ADMIN_EMAIL and use localhost.');
  return { email };
});

// ── Fake GitHub (OXYN_DEV_GITHUB_STUB) ─────────────────────────────────────────────────────────
const FAKE = 'https://fake-github.oxyn.invalid';
interface FakeUser { id: number; login: string; email: string; verified?: boolean; name?: string }
const DEFAULT_USER: FakeUser = { id: 1001, login: 'octo-dev', email: 'octo@example.com', verified: true, name: 'Octo Dev' };
const enc64 = (o: unknown) => btoa(JSON.stringify(o)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
const dec64 = (s: string): FakeUser => JSON.parse(atob(s.replace(/-/g, '+').replace(/_/g, '/')));

function fakeAuthorize(url: URL) {
  const back = new URL(url.searchParams.get('redirect_uri') || '/api/auth/github/callback', url.origin);
  if (back.origin !== url.origin) return json({ error: 'OXYN_DEV_GITHUB_STUB: redirect_uri must be this origin' }, 400);
  const state = url.searchParams.get('state') || '';
  if (url.searchParams.get('deny')) { back.search = new URLSearchParams({ error: 'access_denied', state }).toString(); }
  else {
    let user = DEFAULT_USER;
    try { if (url.searchParams.get('as')) user = { ...DEFAULT_USER, ...dec64(url.searchParams.get('as')!) }; } catch { /* default */ }
    back.search = new URLSearchParams({ code: 'mock.' + enc64(user), state }).toString();
  }
  return new Response(null, { status: 302, headers: { location: back.toString() } });
}

const fakeFetch: typeof fetch = async (input, init) => {
  const u = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url);
  if (u.origin !== FAKE) throw new Error('OXYN_DEV_GITHUB_STUB: unexpected fetch ' + u.href);
  if (u.pathname === '/login/oauth/access_token') {
    const b = JSON.parse(String(init?.body || '{}')) as { code?: string };
    if (!b.code?.startsWith('mock.')) return json({ error: 'bad_verification_code' }, 200);
    return json({ access_token: b.code.slice(5), token_type: 'bearer', scope: 'read:user,user:email' });
  }
  const auth = new Headers(init?.headers).get('authorization') || '';
  let user: FakeUser;
  try { user = dec64(auth.replace(/^Bearer /, '')); } catch { return json({ message: 'Bad credentials' }, 401); }
  if (u.pathname === '/user') return json({ id: user.id, login: user.login, name: user.name ?? null, avatar_url: `https://avatars.githubusercontent.com/u/${user.id}?v=4` });
  if (u.pathname === '/user/emails') return json([{ email: user.email, primary: true, verified: user.verified !== false }]);
  return json({ message: 'Not Found' }, 404);
};

const devAuth = makeAuthHandler({ authorize: '/__dev/github/authorize', token: `${FAKE}/login/oauth/access_token`, api: FAKE }, fakeFetch);

export default {
  fetch(req: Request, env: Env, ctx: Ctx) {
    const url = new URL(req.url);
    const p = url.pathname;
    if (p === '/admin' || p.startsWith('/admin/') || p === '/api/admin' || p.startsWith('/api/admin/')) return devAdmin(req, env, ctx, url);
    if (p.startsWith('/api/auth/') || p === '/__dev/github/authorize') {
      if (!local(req)) return fail(new HttpError(403, 'dev_only', 'OXYN_DEV_GITHUB_STUB: localhost only.'));
      if (p === '/__dev/github/authorize') return Promise.resolve(fakeAuthorize(url));
      return devAuth(req, env, ctx, url);
    }
    return worker.fetch(req, env, ctx);
  },
};
