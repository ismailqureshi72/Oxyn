// Types and helpers shared by the public API (index.ts) and the admin panel (admin.ts).

export interface D1Result<T = Record<string, unknown>> { results?: T[]; meta?: Record<string, unknown> }
export interface D1Stmt {
  bind(...v: unknown[]): D1Stmt;
  first<T = Record<string, unknown>>(): Promise<T | null>;
  run(): Promise<D1Result>;
  all<T = Record<string, unknown>>(): Promise<D1Result<T>>;
}
export interface D1 { prepare(q: string): D1Stmt; batch(s: D1Stmt[]): Promise<D1Result[]> }
export interface R2Object { body: ReadableStream; httpMetadata?: { contentType?: string }; size: number }
export interface R2 {
  put(key: string, value: ArrayBuffer, opts?: { httpMetadata?: { contentType?: string }; customMetadata?: Record<string, string> }): Promise<unknown>;
  get(key: string): Promise<R2Object | null>;
  delete(key: string): Promise<void>;
}
export interface Ctx { waitUntil(p: Promise<unknown>): void }
export interface Env {
  DB: D1;
  UPLOADS: R2;
  ASSETS: { fetch(r: Request): Promise<Response> };
  GITHUB_TOKEN?: string;
  GITHUB_REPO?: string;
  RATE_SALT?: string;
  // Admin panel (docs/admin.md)
  ACCESS_TEAM_DOMAIN?: string; // https://<team>.cloudflareaccess.com
  ACCESS_AUD?: string; // Access application audience tag
  ACCESS_GROUP_ID?: string; // Access group "oxyn-admins"
  CF_ACCOUNT_ID?: string;
  CF_ADMIN_TOKEN?: string; // secret: Access Groups edit, keeps the group in sync with the admin list
  ADMIN_SESSION_KEY?: string; // secret, optional: HMAC key for session cookies (else generated into D1)
  // Site accounts (docs/auth.md): the GitHub OAuth app. Unset = GitHub sign-in shows "coming soon".
  GITHUB_OAUTH_CLIENT_ID?: string; // secret
  GITHUB_OAUTH_CLIENT_SECRET?: string; // secret
}

export const KINDS = ['Bug', 'Idea', 'Question'] as const;
export type Kind = (typeof KINDS)[number];
export const EMAIL_RE = /^[^\s@<>"']{1,64}@[^\s@<>"']+\.[^\s@<>"']{2,}$/;

export class HttpError extends Error {
  constructor(public status: number, public code: string, message: string, public field?: string) { super(message); }
}

export const json = (body: unknown, status = 200, extra: Record<string, string> = {}) =>
  new Response(JSON.stringify(body), {
    status,
    headers: {
      'content-type': 'application/json; charset=utf-8',
      'cache-control': 'no-store',
      'x-content-type-options': 'nosniff',
      ...extra,
    },
  });

export const fail = (e: HttpError, extra: Record<string, string> = {}) =>
  json({ ok: false, error: { code: e.code, message: e.message, ...(e.field ? { field: e.field } : {}) } }, e.status,
    { ...(e.status === 429 ? { 'retry-after': '600' } : {}), ...extra });

/** Only the site's own pages may post: the Origin (or Referer) must be this host. */
export function sameOrigin(req: Request, url: URL) {
  const origin = req.headers.get('origin');
  if (origin) return origin === url.origin;
  const ref = req.headers.get('referer');
  if (ref) { try { return new URL(ref).origin === url.origin; } catch { return false; } }
  return req.headers.get('sec-fetch-site') === 'same-origin';
}

export async function sha256Hex(s: string) {
  const d = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return [...new Uint8Array(d)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

/** Opens a GitHub issue labelled "feedback" and stores its URL on the row. The reporter's email and
 *  user agent stay private in D1. Returns the issue URL. */
export async function openIssue(env: Env, f: { id: string; kind: Kind; message: string; page: string; hasShot: boolean; created: string }) {
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
  return issue.html_url || '';
}
