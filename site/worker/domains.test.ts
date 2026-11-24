// Host redirects (shared.ts canonicalRedirect, run first in index.ts): www.oxynui.com and the old
// ui.oxyn.workers.dev send pages to https://oxynui.com; /api/* is served on every host. `npm test`.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import worker from './index.ts';
import { canonicalRedirect, siteOrigin, type Env } from './shared.ts';

const ctx = { waitUntil: (_p: Promise<unknown>) => {} };
const served: string[] = [];
const env = {
  SITE_URL: 'https://oxynui.com',
  ASSETS: { fetch: async (r: Request) => { served.push(r.url); return new Response('asset', { status: 200 }); } },
} as unknown as Env;
const go = (u: string, init?: RequestInit) => worker.fetch(new Request(u, init), env, ctx);

test('www and the workers.dev address redirect pages with 301, keeping path and query', async () => {
  for (const host of ['www.oxynui.com', 'ui.oxyn.workers.dev', 'WWW.OXYNUI.COM']) {
    const r = await go(`https://${host}/components/chat/composer/?tab=code`);
    assert.equal(r.status, 301, host);
    assert.equal(r.headers.get('location'), 'https://oxynui.com/components/chat/composer/?tab=code');
  }
  const root = await go('https://www.oxynui.com/');
  assert.equal(root.headers.get('location'), 'https://oxynui.com/');
  const reg = await go('https://ui.oxyn.workers.dev/r/index.json');
  assert.equal(reg.status, 301);
  assert.equal(reg.headers.get('location'), 'https://oxynui.com/r/index.json');
  const admin = await go('https://ui.oxyn.workers.dev/admin');
  assert.equal(admin.headers.get('location'), 'https://oxynui.com/admin');
});

test('a non-GET page request keeps its method (308)', async () => {
  const r = await go('https://ui.oxyn.workers.dev/pricing/', { method: 'POST', body: 'x' });
  assert.equal(r.status, 308);
});

test('/api/* is never redirected, on any host', () => {
  for (const host of ['ui.oxyn.workers.dev', 'www.oxynui.com']) {
    for (const p of ['/api/polar/webhook', '/api/auth/github/callback?code=1', '/api/r/index', '/api']) {
      assert.equal(canonicalRedirect(env, new URL(`https://${host}${p}`)), null, host + p);
    }
  }
});

test('the canonical host and other hosts (local dev, previews) are served, not redirected', async () => {
  served.length = 0;
  for (const u of ['https://oxynui.com/', 'https://oxynui.com/docs/cli/', 'http://localhost:8787/', 'https://abc123-ui.oxyn.workers.dev/']) {
    const r = await go(u);
    assert.equal(r.status, 200, u);
  }
  assert.equal(served.length, 4);
});

test('SITE_URL defaults to https://oxynui.com', () => {
  assert.equal(siteOrigin({}), 'https://oxynui.com');
  assert.equal(siteOrigin({ SITE_URL: 'https://example.test/' }), 'https://example.test');
  const r = canonicalRedirect({ SITE_URL: 'https://example.test' }, new URL('https://www.example.test/a?b=1'));
  assert.equal(r?.headers.get('location'), 'https://example.test/a?b=1');
});

test('plain http on the canonical host goes to https (pages only)', async () => {
  const r = await go('http://oxynui.com/pricing/?x=1');
  assert.equal(r.status, 301);
  assert.equal(r.headers.get('location'), 'https://oxynui.com/pricing/?x=1');
  assert.equal((await go('http://oxynui.com/')).headers.get('location'), 'https://oxynui.com/');
  assert.equal(canonicalRedirect(env, new URL('http://oxynui.com/api/billing/plans')), null);
  assert.equal(canonicalRedirect(env, new URL('https://oxynui.com/')), null);
  assert.equal(canonicalRedirect(env, new URL('http://localhost:4321/')), null);
});
