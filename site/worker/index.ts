// The Oxyn site Worker. Static assets (dist/) are served by Cloudflare before this code runs;
// wrangler.jsonc routes only /api/* here (assets.run_worker_first), so the site itself never
// touches the Worker. Plain TypeScript with no dependencies: wrangler bundles it with esbuild,
// so CI needs no extra toolchain next to the Rust + Astro build.
//
//   GET  /api/health    liveness + a D1 round trip
//   POST /api/feedback  multipart/form-data or JSON from the site's Feedback panel
//
// Bindings (wrangler.jsonc): DB (D1 "oxyn"), UPLOADS (R2 "oxyn-uploads"), ASSETS.
// Secrets (optional): GITHUB_TOKEN opens a GitHub issue per feedback; RATE_SALT salts IP hashes.

interface D1Result<T = Record<string, unknown>> { results?: T[]; meta?: Record<string, unknown> }
interface D1Stmt {
  bind(...v: unknown[]): D1Stmt;
  first<T = Record<string, unknown>>(): Promise<T | null>;
  run(): Promise<D1Result>;
  all<T = Record<string, unknown>>(): Promise<D1Result<T>>;
}
interface D1 { prepare(q: string): D1Stmt; batch(s: D1Stmt[]): Promise<D1Result[]> }
interface R2 { put(key: string, value: ArrayBuffer, opts?: { httpMetadata?: { contentType?: string }; customMetadata?: Record<string, string> }): Promise<unknown> }
interface Ctx { waitUntil(p: Promise<unknown>): void }
export interface Env {
  DB: D1;
  UPLOADS: R2;
  ASSETS: { fetch(r: Request): Promise<Response> };
  GITHUB_TOKEN?: string;
  GITHUB_REPO?: string;
  RATE_SALT?: string;
}

const KINDS = ['Bug', 'Idea', 'Question'] as const;
type Kind = (typeof KINDS)[number];

/** Limits, shared with docs/backend.md and the client (public/feedback.js). */
export const LIMITS = {
  body: 2_600_000, // whole request, bytes (2 MB image + form fields + multipart overhead)
  screenshot: 2 * 1024 * 1024, // bytes
  messageMin: 3,
  messageMax: 4000,
  email: 254,
  url: 2048,
  ua: 400,
  // fixed windows per hashed IP
  burst: { window: 600, max: 5 }, // 5 per 10 minutes
  daily: { window: 86_400, max: 30 }, // 30 per day
};
const IMAGE_TYPES: Record<string, string> = { 'image/png': 'png', 'image/jpeg': 'jpg', 'image/webp': 'webp', 'image/gif': 'gif' };
const EMAIL_RE = /^[^\s@<>"']{1,64}@[^\s@<>"']+\.[^\s@<>"']{2,}$/;

class HttpError extends Error {
  constructor(public status: number, public code: string, message: string, public field?: string) { super(message); }
}

const json = (body: unknown, status = 200, extra: Record<string, string> = {}) =>
  new Response(JSON.stringify(body), {
    status,
    headers: {
      'content-type': 'application/json; charset=utf-8',
      'cache-control': 'no-store',
      'x-content-type-options': 'nosniff',
      ...extra,
    },
  });
const fail = (e: HttpError) =>
  json({ ok: false, error: { code: e.code, message: e.message, ...(e.field ? { field: e.field } : {}) } }, e.status,
    e.status === 429 ? { 'retry-after': '600' } : {});

export default {
  async fetch(req: Request, env: Env, ctx: Ctx): Promise<Response> {
    const url = new URL(req.url);
    if (!url.pathname.startsWith('/api/')) return env.ASSETS.fetch(req);
    try {
      const route = url.pathname.replace(/\/+$/, '');
      if (route === '/api/health') {
        if (req.method !== 'GET' && req.method !== 'HEAD') throw new HttpError(405, 'method_not_allowed', 'Use GET.');
        return await health(env);
      }
      if (route === '/api/feedback') {
        if (req.method !== 'POST') throw new HttpError(405, 'method_not_allowed', 'Use POST.');
        return await feedback(req, env, ctx, url);
      }
      throw new HttpError(404, 'not_found', 'No such API route.');
    } catch (err) {
      if (err instanceof HttpError) return fail(err);
      console.error('api error', err);
      return fail(new HttpError(500, 'internal', 'Something went wrong on our side. Please try again.'));
    }
  },
};

async function health(env: Env): Promise<Response> {
  let db = 'ok';
  try { await env.DB.prepare('SELECT 1 AS one').first(); } catch { db = 'error'; }
  return json({ ok: db === 'ok', service: 'oxyn-site', db, time: new Date().toISOString() }, db === 'ok' ? 200 : 503);
}

/** Only the site's own pages may post: the Origin (or Referer) must be this host. */
function sameOrigin(req: Request, url: URL) {
  const origin = req.headers.get('origin');
  if (origin) return origin === url.origin;
  const ref = req.headers.get('referer');
  if (ref) { try { return new URL(ref).origin === url.origin; } catch { return false; } }
  return req.headers.get('sec-fetch-site') === 'same-origin';
}

interface Input { kind: string; message: string; page_url: string; email: string; website: string; file: File | null }

async function readInput(req: Request): Promise<Input> {
  const type = req.headers.get('content-type') || '';
  const str = (v: unknown) => (typeof v === 'string' ? v : '');
  if (type.startsWith('multipart/form-data')) {
    let fd: FormData;
    try { fd = await req.formData(); } catch { throw new HttpError(400, 'bad_body', 'The form could not be read.'); }
    const f = fd.get('screenshot');
    const file = f && typeof f !== 'string' && (f as File).size > 0 ? (f as File) : null;
    return { kind: str(fd.get('kind')), message: str(fd.get('message')), page_url: str(fd.get('page_url')), email: str(fd.get('email')), website: str(fd.get('website')), file };
  }
  if (type.startsWith('application/json')) {
    let b: Record<string, unknown>;
    try { b = await req.json(); } catch { throw new HttpError(400, 'bad_body', 'The JSON body could not be read.'); }
    return { kind: str(b.kind), message: str(b.message), page_url: str(b.page_url), email: str(b.email), website: str(b.website), file: null };
  }
  throw new HttpError(415, 'unsupported_media_type', 'Send multipart/form-data or application/json.');
}

async function sha256Hex(s: string) {
  const d = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return [...new Uint8Array(d)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

/** Fixed-window counters in D1, keyed by a salted hash of the client IP. Throws 429 when over. */
async function rateLimit(env: Env, ip: string) {
  const now = Math.floor(Date.now() / 1000);
  const h = (await sha256Hex(`${env.RATE_SALT ?? 'oxyn-feedback'}|${ip}`)).slice(0, 32);
  const checks = [
    { key: `fb10m:${h}`, ...LIMITS.burst },
    { key: `fb1d:${h}`, ...LIMITS.daily },
  ];
  const stmts = checks.map((c) =>
    env.DB.prepare(
      'INSERT INTO rate_limits (key, window_start, count) VALUES (?1, ?2, 1) ' +
        'ON CONFLICT (key, window_start) DO UPDATE SET count = count + 1 RETURNING count',
    ).bind(c.key, now - (now % c.window)),
  );
  const res = await env.DB.batch(stmts);
  res.forEach((r, i) => {
    const n = Number((r.results?.[0] as { count?: number } | undefined)?.count ?? 0);
    if (n > checks[i].max) throw new HttpError(429, 'rate_limited', 'Too many messages from here. Please try again a little later.');
  });
  // Housekeeping: drop windows older than two days (cheap, indexed by primary key prefix scan).
  if (Math.random() < 0.05) await env.DB.prepare('DELETE FROM rate_limits WHERE window_start < ?1').bind(now - 2 * 86_400).run();
}

function sniffImage(b: Uint8Array): string | null {
  const at = (i: number, ...v: number[]) => v.every((x, k) => b[i + k] === x);
  if (at(0, 0x89, 0x50, 0x4e, 0x47)) return 'image/png';
  if (at(0, 0xff, 0xd8, 0xff)) return 'image/jpeg';
  if (at(0, 0x47, 0x49, 0x46, 0x38)) return 'image/gif';
  if (at(0, 0x52, 0x49, 0x46, 0x46) && at(8, 0x57, 0x45, 0x42, 0x50)) return 'image/webp';
  return null;
}

async function feedback(req: Request, env: Env, ctx: Ctx, url: URL): Promise<Response> {
  if (!sameOrigin(req, url)) throw new HttpError(403, 'forbidden_origin', 'Feedback can only be sent from the Oxyn site.');
  const len = Number(req.headers.get('content-length') || '0');
  if (len > LIMITS.body) throw new HttpError(413, 'too_large', 'That is too large. Screenshots can be up to 2 MB.');

  const input = await readInput(req);
  // Honeypot: a hidden "website" field people never see. Bots fill it; pretend it worked.
  if (input.website.trim()) return json({ ok: true, id: crypto.randomUUID() }, 201);

  const kind = input.kind.trim() as Kind;
  if (!KINDS.includes(kind)) throw new HttpError(400, 'invalid_kind', 'Choose Bug, Idea or Question.', 'kind');
  const message = input.message.replace(/\r\n/g, '\n').trim();
  if (message.length < LIMITS.messageMin) throw new HttpError(400, 'message_too_short', 'Please write a little more.', 'message');
  if (message.length > LIMITS.messageMax) throw new HttpError(400, 'message_too_long', `Please keep it under ${LIMITS.messageMax} characters.`, 'message');
  const email = input.email.trim();
  if (email && (email.length > LIMITS.email || !EMAIL_RE.test(email))) throw new HttpError(400, 'invalid_email', 'That email address does not look right.', 'email');
  let page = input.page_url.trim().slice(0, LIMITS.url);
  try { const p = new URL(page); page = p.origin === url.origin ? p.pathname + p.search + p.hash : ''; } catch { page = page.startsWith('/') ? page : ''; }
  const ua = (req.headers.get('user-agent') || '').slice(0, LIMITS.ua);

  let shot: { bytes: ArrayBuffer; type: string } | null = null;
  if (input.file) {
    if (input.file.size > LIMITS.screenshot) throw new HttpError(413, 'screenshot_too_large', 'Screenshots can be up to 2 MB.', 'screenshot');
    const bytes = await input.file.arrayBuffer();
    const type = sniffImage(new Uint8Array(bytes.slice(0, 16)));
    if (!type || !IMAGE_TYPES[type]) throw new HttpError(415, 'screenshot_not_image', 'Screenshots must be PNG, JPEG, WebP or GIF images.', 'screenshot');
    shot = { bytes, type };
  }

  await rateLimit(env, req.headers.get('cf-connecting-ip') || 'local');

  const id = crypto.randomUUID();
  const created = new Date().toISOString();
  let key: string | null = null;
  if (shot) {
    key = `feedback/${created.slice(0, 10)}/${id}.${IMAGE_TYPES[shot.type]}`;
    await env.UPLOADS.put(key, shot.bytes, { httpMetadata: { contentType: shot.type }, customMetadata: { feedback_id: id } });
  }
  await env.DB.prepare(
    'INSERT INTO feedback (id, kind, message, page_url, email, ua, screenshot_key, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)',
  ).bind(id, kind, message, page || null, email || null, ua || null, key, created, 'new').run();

  if (env.GITHUB_TOKEN) ctx.waitUntil(openIssue(env, { id, kind, message, page, hasShot: !!key, created }).catch((e) => console.error('github issue', e)));
  return json({ ok: true, id }, 201);
}

/** Opens a GitHub issue labelled "feedback". The reporter's email and user agent stay private in D1. */
async function openIssue(env: Env, f: { id: string; kind: Kind; message: string; page: string; hasShot: boolean; created: string }) {
  const repo = env.GITHUB_REPO || 'ismailqureshi72/Oxyn';
  const first = f.message.split('\n')[0].slice(0, 72);
  // Quote the message and break @mentions so a note cannot ping people.
  const quoted = f.message.replace(/@(?=\w)/g, '@\u200b').split('\n').map((l) => `> ${l}`).join('\n');
  const body = [
    `**${f.kind}** from the site feedback button.`,
    '',
    quoted,
    '',
    `- Page: ${f.page ? `https://ui.oxyn.workers.dev${f.page}` : 'unknown'}`,
    `- Received: ${f.created}`,
    `- Screenshot: ${f.hasShot ? 'yes (in the admin inbox)' : 'no'}`,
    `- Feedback id: \`${f.id}\``,
  ].join('\n');
  const r = await fetch(`https://api.github.com/repos/${repo}/issues`, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${env.GITHUB_TOKEN}`,
      accept: 'application/vnd.github+json',
      'x-github-api-version': '2022-11-28',
      'user-agent': 'oxyn-site-worker',
      'content-type': 'application/json',
    },
    body: JSON.stringify({ title: `[${f.kind}] ${first}`, body, labels: ['feedback'] }),
  });
  if (!r.ok) throw new Error(`GitHub ${r.status}: ${(await r.text()).slice(0, 200)}`);
  const issue = (await r.json()) as { html_url?: string };
  if (issue.html_url) await env.DB.prepare('UPDATE feedback SET github_issue_url = ?1 WHERE id = ?2').bind(issue.html_url, f.id).run();
}
