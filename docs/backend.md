# Backend

The Oxyn site runs as one Cloudflare Worker, `ui` (https://ui.oxyn.workers.dev). It serves the
static site from `site/dist`, a small API under `/api/*` (feedback, flags, site sign-in from
[auth.md](auth.md)) and the maintainers' admin panel under `/admin` ([admin.md](admin.md)). Everything stays on Cloudflare:
Workers, D1 (SQLite) and R2 (object storage). No other services.

## Architecture

```
browser ──► Cloudflare edge ──► static assets (site/dist)        every path except /api/* and /admin*
                           ├──► Cloudflare Access (admins only)   /admin*, /api/admin/*  → Worker
                           └──► Worker  site/worker/index.ts      /api/*, /admin*
                                  ├─ D1  "oxyn"          (binding DB)       feedback, rate limits, admins, settings
                                  ├─ R2  "oxyn-uploads"  (binding UPLOADS)  feedback screenshots
                                  └─ GitHub REST API     (secret GITHUB_TOKEN, optional)
```

- Config: `site/wrangler.jsonc`. `assets.run_worker_first: ["/api/*", "/admin", "/admin/*"]` makes
  assets answer first for everything else, so the site never runs Worker code and a Worker bug
  cannot take pages down.
- The Worker is plain TypeScript with no dependencies. `wrangler` bundles it with esbuild at deploy
  time, so CI needs nothing beyond the existing Node step. Rust (workers-rs) was not used: it would
  add a second wasm build and toolchain to the site job for two small endpoints.
- Feedback UI: `site/src/components/FeedbackButton.astro` (in the top bar and the library bar),
  `site/src/components/Feedback.astro` (the panel, once per page), `site/public/feedback.js`
  (behaviour) and the `.fb-*` rules in `site/src/styles/chrome.css`. It is on every site and docs
  page, never inside demos or the full-screen `/screens/<slug>/full/` views.

## Endpoints

All responses are JSON with `cache-control: no-store`. Errors look like
`{"ok": false, "error": {"code": "invalid_kind", "message": "…", "field": "kind"}}`.
Unknown `/api/*` routes return 404 `not_found`; wrong methods 405 `method_not_allowed`.

### `GET /api/health`

`200 {"ok": true, "service": "oxyn-site", "db": "ok", "time": "<ISO>"}`; `503` with `"db": "error"`
when D1 does not answer.

### `GET /api/flags`

`200 {"ok": true, "flags": {"<name>": "<value>"}}`: the content flags that are switched on in the admin
panel (Settings), `cache-control: public, max-age=60`.

### `/api/auth/*`

Site accounts: sign in with GitHub or an email and a password, sessions in D1. See [auth.md](auth.md).

### `/api/admin/*`

The admin panel's API, behind Cloudflare Access, the Worker's own JWT check and the shared password.
See [admin.md](admin.md).

### `POST /api/feedback`

Body: `multipart/form-data` (what the panel sends) or `application/json`.

| Field | Rules |
|---|---|
| `kind` | `Bug`, `Idea` or `Question` |
| `message` | 3 to 4000 characters after trimming |
| `page_url` | optional; kept only when it is on this site (stored as path + query + hash) |
| `email` | optional; at most 254 characters, must look like an address |
| `screenshot` | optional file (multipart only); PNG, JPEG, WebP or GIF by magic bytes, up to 2 MB |
| `website` | honeypot; hidden from people. When filled the request "succeeds" and nothing is stored |

- Same origin only: the `Origin` header (or `Referer`, or `Sec-Fetch-Site: same-origin`) must be the
  site itself, else `403 forbidden_origin`. No CORS headers are sent.
- Size: requests over 2.6 MB are refused with `413 too_large`.
- Rate limit: fixed windows per salted SHA-256 of `CF-Connecting-IP` in the `rate_limits` table,
  5 per 10 minutes and 30 per day, else `429 rate_limited` with `retry-after`. Raw IPs are never stored.
- Success: `201 {"ok": true, "id": "<uuid>"}`. The row is stored with `status = 'new'`; a screenshot
  goes to R2 at `feedback/<YYYY-MM-DD>/<id>.<ext>` (private; the admin panel serves it).
- GitHub: when the `GITHUB_TOKEN` secret is set, the Worker opens an issue in `GITHUB_REPO`
  (`ismailqureshi72/Oxyn`) labelled `feedback`, after the response (`waitUntil`), and saves its URL
  in `github_issue_url`. The issue holds the kind, message (quoted, @mentions broken), page, time
  and feedback id; the email and user agent stay in D1. Without the secret nothing is sent and the
  row stays `new`.

## D1 schema and migrations

Migrations live in `site/migrations/` (`migrations_dir` in `wrangler.jsonc`).

| Table | Purpose |
|---|---|
| `feedback` | `id`, `kind` (Bug/Idea/Question), `message`, `page_url`, `email`, `ua`, `screenshot_key`, `created_at` (ISO, UTC), `status` (new/triaged/closed), `github_issue_url` |
| `rate_limits` | `key` (`<scope>:<ip hash>`), `window_start`, `count` |
| `admins` | maintainers allowed into `/admin`; `access_synced`, `removed_at` track the Access group sync (0002) |
| `settings` | key/value: `admin_password` (PBKDF2 record), `admin_session_key`, `flag:<name>`, `access_sync_error` |
| `admin_sessions` | admin sessions: id, email, password version, expiry (0002) |
| `audit_log` | admin actions: time, actor, action, target, detail (0002) |
| `users` | site accounts: email, verified flag, GitHub name and avatar, PBKDF2 password record or NULL (0003, [auth.md](auth.md)) |
| `oauth_accounts` | linked GitHub accounts: provider, provider id, user, login (0003) |
| `sessions` | site sessions: SHA-256 of the cookie token, user, provider, expiry (0003) |

Apply new migrations from `site/` (CI does not apply them):

```sh
npx wrangler d1 migrations list oxyn --remote
npx wrangler d1 migrations apply oxyn --remote
```

Add a migration with `npx wrangler d1 migrations create oxyn <name>`; never edit an applied one.

## Secrets and variables

| Name | Kind | Needed for |
|---|---|---|
| `GITHUB_TOKEN` | Worker secret, optional | Opening feedback issues. A fine-grained token limited to `ismailqureshi72/Oxyn` with **Issues: Read and write** only. Set with `npx wrangler secret put GITHUB_TOKEN` from `site/`. |
| `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_OAUTH_CLIENT_SECRET` | Worker secrets, optional | "Continue with GitHub" on `/login/`. Until set, the button reads "coming soon" and the GitHub routes answer 503. Setup in [auth.md](auth.md). |
| `RATE_SALT` | Worker secret, optional | Salt for IP hashes (a fixed default is used when unset). |
| `CF_ADMIN_TOKEN`, `ADMIN_SESSION_KEY` | Worker secrets, optional | Admin panel: Access group sync, session HMAC key ([admin.md](admin.md)). |
| `ACCESS_TEAM_DOMAIN`, `ACCESS_AUD`, `ACCESS_GROUP_ID`, `CF_ACCOUNT_ID` | vars in `wrangler.jsonc` | Admin panel: Access JWT check and group sync. |
| `GITHUB_REPO` | var in `wrangler.jsonc` | Repository for feedback issues. |
| `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` | GitHub Actions secrets | The deploy step on `main`. The token needs Workers Scripts Edit; D1 and R2 Edit are needed for the bindings and for applying migrations from a machine. |

The `feedback` label must exist in the repository for issues to carry it (GitHub creates a missing
label only when the token may manage labels).

## CI and deploy

The `site` job builds the site, runs the 24 MiB asset guard, bundles the Worker with
`wrangler deploy --dry-run` on every PR, and on pushes to `main` runs `wrangler deploy`, which
uploads the assets, the Worker and its D1 / R2 bindings together. Nothing is uploaded as an
Actions artifact.

## Local development

```sh
cd site
npm run build                                       # or build:site when the wasm bundle exists
npx wrangler d1 migrations apply oxyn --local       # local SQLite under .wrangler/
npx wrangler dev                                    # site + API on http://localhost:8787
curl localhost:8787/api/health
curl -X POST localhost:8787/api/feedback -H 'origin: http://localhost:8787' \
  -H 'content-type: application/json' -d '{"kind":"Idea","message":"Hello from local dev"}'
npx wrangler d1 execute oxyn --local --command "SELECT id, kind, message, status FROM feedback"
```

`wrangler dev` uses local D1 and R2 simulations by default, so nothing touches production. Put
local secrets in `site/.dev.vars` (ignored by git), e.g. `GITHUB_TOKEN=...` to test issue creation.
