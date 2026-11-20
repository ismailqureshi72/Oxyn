// The maintainers' admin panel: /admin (the UI) and /api/admin/* (its JSON API). See docs/admin.md.
//
// There is one sign-in for the whole site (auth.ts). The panel opens for a signed-in account when,
// on every request:
//   1. its email was verified by a provider: the account has signed in with Google or GitHub at
//      least once (users.email_verified = 1 and a linked oauth_accounts row). An email + password
//      account is never enough, because nothing has checked that its owner controls the address;
//   2. that email is an active row in D1 `admins`. Removing someone takes effect on their next
//      request.
// Signed out: /admin redirects to /login/?next=/admin. Signed in but not an admin: 403 page. The UI
// files (admin-ui/) are served only after both checks; the stylesheet and the theme script carry
// nothing private and are served to the refusal pages too.

import { EMAIL_RE, HttpError, KINDS, fail, json, openIssue, sha256Hex, type Ctx, type Env, type Kind } from './shared';
import { adminStanding, currentUser } from './auth';
import { adminBilling } from './billing';
import { activity, addGrants, changePlan, checkGrants, getUser, listGrants, listUsers, restoreGrant, revokeGrant } from './access';
import PAGE from './admin-ui/index.html';
import CSS from './admin-ui/app.css';
import JS from './admin-ui/app.js';
import THEME from './admin-ui/theme.js';
// The current Oxyn lockup (same SVG as the site's top bar), drawn in currentColor.
import { lockup } from '../src/components/v2/logo';
const BRAND = `${lockup}<span class="sr">Oxyn</span><span class="tag">Admin</span>`;
const ICONS = '<link rel="icon" href="/favicon.svg" type="image/svg+xml">\n<link rel="icon" href="/favicon-32.png" type="image/png" sizes="32x32">\n<link rel="apple-touch-icon" href="/apple-touch-icon.png" sizes="180x180">';

export const ADMIN = {
  flagValueMax: 500,
  pageSize: 50,
  // changes made through the API: per admin email, fixed window
  writes: { window: 600, max: 120 },
};
const STATUSES = ['new', 'triaged', 'closed'] as const;
const FLAG_KEY = /^[a-z0-9][a-z0-9_.-]{0,47}$/;

const now = () => Math.floor(Date.now() / 1000);
const iso = () => new Date().toISOString();

/** Fixed-window limit on admin changes, per admin email. */
async function writeLimit(env: Env, email: string) {
  const t = now();
  const w = ADMIN.writes;
  const key = `admw:${(await sha256Hex(`${env.RATE_SALT ?? 'oxyn-admin'}|${email}`)).slice(0, 32)}`;
  const r = await env.DB.prepare(
    'INSERT INTO rate_limits (key, window_start, count) VALUES (?1, ?2, 1) ON CONFLICT (key, window_start) DO UPDATE SET count = count + 1 RETURNING count',
  ).bind(key, t - (t % w.window)).first<{ count: number }>();
  if (r && r.count > w.max) throw new HttpError(429, 'rate_limited', 'Too many changes in a short time. Wait a few minutes and try again.');
}

export async function audit(env: Env, actor: string, action: string, target: string | null = null, detail: unknown = null) {
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
const esc = (t: string) => t.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);

/** A refusal page in the panel's own look (no data, no app script). */
function deniedPage(title: string, msg: string, status: number, actions: [string, string, string?][] = [['/', 'Back to Oxyn']], who = '') {
  const btns = actions.map(([href, label, cls]) => `<a class="btn ${cls || ''}" href="${esc(href)}">${esc(label)}</a>`).join('');
  return page(`<!doctype html>
<html lang="en" data-theme="light">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex, nofollow, noarchive">
<meta name="referrer" content="no-referrer">
<title>${esc(title)} · Oxyn admin</title>
${ICONS}
<link rel="stylesheet" href="/admin/app.css">
<script src="/admin/theme.js"></script>
</head>
<body>
<header class="bar"><a class="brand" href="/" aria-label="Oxyn admin, back to the site">${BRAND}</a></header>
<main id="main"><div class="auth"><div class="card">
<h1>${esc(title)}</h1>
<p>${esc(msg)}</p>
${who ? `<p class="small">Signed in as ${esc(who)}.</p>` : ''}
<div class="row">${btns}</div>
</div></div></main>
</body>
</html>`, 'text/html; charset=utf-8', status);
}

const isAdminPath = (p: string) => p === '/admin' || p.startsWith('/admin/') || p === '/api/admin' || p.startsWith('/api/admin/');

export async function handleAdmin(req: Request, env: Env, ctx: Ctx, url: URL): Promise<Response> {
  const isApi = url.pathname.startsWith('/api/');
  const path = url.pathname.replace(/\/+$/, '') || '/admin';
  // Styles and the theme switch carry nothing private: the refusal pages use them too.
  if (!isApi && (path === '/admin/app.css' || path === '/admin/theme.js') && (req.method === 'GET' || req.method === 'HEAD'))
    return path === '/admin/app.css' ? page(CSS, 'text/css; charset=utf-8') : page(THEME, 'text/javascript; charset=utf-8');
  try {
    const u = await currentUser(req, env);
    if (!u) {
      if (!isApi && (req.method === 'GET' || req.method === 'HEAD'))
        return new Response(null, { status: 302, headers: { location: `${url.origin}/login/?next=${encodeURIComponent('/admin')}`, ...SEC } });
      throw new HttpError(401, 'signed_out', 'Sign in to Oxyn first.');
    }
    const standing = await adminStanding(env, u);
    if (standing === 'verify') {
      if (!isApi) {
        // Straight to the provider: the sign-in page would only send a signed-in visitor back here.
        const go: [string, string, string?][] = [];
        if (env.GOOGLE_OAUTH_CLIENT_ID && env.GOOGLE_OAUTH_CLIENT_SECRET) go.push(['/api/auth/google/start?next=%2Fadmin', 'Continue with Google', 'primary']);
        if (env.GITHUB_OAUTH_CLIENT_ID && env.GITHUB_OAUTH_CLIENT_SECRET) go.push(['/api/auth/github/start?next=%2Fadmin', 'Continue with GitHub', go.length ? '' : 'primary']);
        return deniedPage('Verify this email first',
          'This email is on the admin list, but it has only signed in with a password, so nobody has checked that it is yours yet. ' +
            (go.length ? 'Sign in with Google or GitHub on the same email once to verify it.' : 'Google and GitHub sign-in are not turned on for this site yet; once one is, sign in with it on the same email to verify it.'),
          403, [...go, ['/', 'Back to Oxyn']], u.email);
      }
      throw new HttpError(403, 'verify_email', 'Sign in with Google or GitHub to verify this email.');
    }
    if (standing !== 'admin') {
      if (!isApi) return deniedPage('No admin access', 'This account is not on the Oxyn admin list.', 403, [['/', 'Back to Oxyn', 'primary'], ['/account/', 'Your account']], u.email);
      throw new HttpError(403, 'not_an_admin', 'This account is not on the admin list.');
    }
    if (!isApi) return ui(req, path);
    return await api(req, env, ctx, url, u.email, u.name);
  } catch (err) {
    const e = err instanceof HttpError ? err : new HttpError(500, 'internal', 'Something went wrong on our side.');
    if (!(err instanceof HttpError)) console.error('admin error', err);
    if (!isApi) return deniedPage('Admin unavailable', e.message, e.status);
    return fail(e, { 'x-robots-tag': SEC['x-robots-tag'] });
  }
}
export { isAdminPath };

function ui(req: Request, p: string) {
  if (req.method !== 'GET' && req.method !== 'HEAD') return deniedPage('Not allowed', 'Method not allowed.', 405);
  if (p === '/admin') return page(PAGE.replace('<!--BRAND-->', BRAND).replace('<!--ICONS-->', ICONS), 'text/html; charset=utf-8');
  if (p === '/admin/app.js') return page(JS, 'text/javascript; charset=utf-8');
  return deniedPage('Not found', 'There is no admin page here.', 404, [['/admin', 'Open the admin panel', 'primary']]);
}

async function body(req: Request): Promise<Record<string, unknown>> {
  if (!(req.headers.get('content-type') || '').startsWith('application/json')) throw new HttpError(415, 'unsupported_media_type', 'Send application/json.');
  if (Number(req.headers.get('content-length') || '0') > 64_000) throw new HttpError(413, 'too_large', 'That request is too large.');
  try { return (await req.json()) as Record<string, unknown>; } catch { throw new HttpError(400, 'bad_body', 'The JSON body could not be read.'); }
}

async function api(req: Request, env: Env, ctx: Ctx, url: URL, email: string, name: string | null): Promise<Response> {
  const m = req.method;
  const route = url.pathname.replace(/\/+$/, '').slice('/api/admin'.length) || '/';
  // Changes must come from the panel itself (same Origin + JSON body), and are rate limited.
  if (m !== 'GET' && m !== 'HEAD') {
    const origin = req.headers.get('origin');
    if (!origin || origin !== url.origin) throw new HttpError(403, 'forbidden_origin', 'Admin changes can only be made from the admin panel.');
    await writeLimit(env, email);
  }

  if (route === '/me' && m === 'GET') return json({ ok: true, email, name, github: !!env.GITHUB_TOKEN });

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
    if (ex) await env.DB.prepare('UPDATE admins SET removed_at = NULL, added_by = ?2, created_at = ?3 WHERE email = ?1').bind(add, email, iso()).run();
    else await env.DB.prepare('INSERT INTO admins (email, added_by, created_at) VALUES (?1, ?2, ?3)').bind(add, email, iso()).run();
    await audit(env, email, 'admin.add', add);
    return json({ ok: true }, 201);
  }
  const am = route.match(/^\/admins\/([^/]+)$/);
  if (am && m === 'DELETE') {
    let target = '';
    try { target = decodeURIComponent(am[1]).toLowerCase(); } catch { throw new HttpError(400, 'invalid_email', 'Bad email.'); }
    const row = await env.DB.prepare('SELECT email FROM admins WHERE email = ?1 AND removed_at IS NULL').bind(target).first();
    if (!row) throw new HttpError(404, 'not_found', 'That email is not an admin.');
    const n = await env.DB.prepare('SELECT COUNT(*) AS n FROM admins WHERE removed_at IS NULL').first<{ n: number }>();
    if (!n || n.n <= 1) throw new HttpError(409, 'last_admin', 'Keep at least one admin. Add someone else first.');
    // Removing yourself locks you out at once: the panel asks you to type your own email.
    if (target === email.toLowerCase() && (url.searchParams.get('confirm') || '').trim().toLowerCase() !== target)
      throw new HttpError(400, 'confirm_self', 'You are removing yourself. Type your own email to confirm.', 'confirm');
    await env.DB.prepare('UPDATE admins SET removed_at = ?2 WHERE email = ?1').bind(target, iso()).run();
    await audit(env, email, 'admin.remove', target);
    return json({ ok: true, self: target === email });
  }

  // Users, grants and activity (access.ts)
  if (route === '/users' && m === 'GET') return listUsers(env, url);
  const um = route.match(/^\/users\/([0-9a-f-]{36})(\/plan)?$/);
  if (um && !um[2] && m === 'GET') return getUser(env, um[1]);
  if (um && um[2] && m === 'POST') return changePlan(env, email, um[1], await body(req));
  if (route === '/grants' && m === 'GET') return listGrants(env, url);
  if (route === '/grants/check' && m === 'POST') return checkGrants(env, await body(req));
  if (route === '/grants' && m === 'POST') return addGrants(env, email, await body(req));
  const gm = route.match(/^\/grants\/(\d{1,12})\/(revoke|restore)$/);
  if (gm && m === 'POST') {
    const b = await body(req);
    return gm[2] === 'revoke' ? revokeGrant(env, email, Number(gm[1]), typeof b.reason === 'string' ? b.reason : '') : restoreGrant(env, email, Number(gm[1]));
  }
  if (route === '/activity' && m === 'GET') return activity(env, ctx, url);

  // Billing: subscribers and recurring revenue (Polar webhook copy in D1, see billing.ts)
  if (route === '/billing' && m === 'GET') return adminBilling(env);

  // Settings: content flags
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
  const r = await env.DB.prepare(
    'SELECT a.email, a.added_by, a.created_at, ' +
      '(SELECT u.email_verified FROM users u WHERE u.email = a.email) AS verified ' +
      'FROM admins a WHERE a.removed_at IS NULL ORDER BY a.created_at',
  ).all();
  return json({ ok: true, me, items: r.results || [] });
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
