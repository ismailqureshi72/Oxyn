# Backend

The Oxyn site runs as one Cloudflare Worker, `ui`, at https://oxynui.com (see [Domains](#domains)). It serves the
static site from `site/dist`, a small API under `/api/*` (feedback, flags, site sign-in from
[auth.md](auth.md)) and the maintainers' admin panel under `/admin` ([admin.md](admin.md)). Everything stays on Cloudflare:
Workers, D1 (SQLite) and R2 (object storage). No other services.

## Architecture

```
browser ──► Cloudflare edge ──► static assets (site/dist)        /_astro/*, /wasm/gallery-dioxus.*, /fonts/*
                           └──► Worker  site/worker/index.ts      every other path: host redirects,
                                  │                               /api/*, /admin*, then ASSETS for pages
                                  ├─ D1  "oxyn"          (binding DB)       feedback, rate limits, admins, settings
                                  ├─ R2  "oxyn-uploads"  (binding UPLOADS)  feedback screenshots
                                  └─ GitHub REST API     (secret GITHUB_TOKEN, optional)
```

- Config: `site/wrangler.jsonc`. `assets.run_worker_first: ["/*", "!/_astro/*",
  "!/wasm/gallery-dioxus.*", "!/fonts/*"]` runs the Worker first for pages too, so it can redirect
  the non-canonical hosts (below). On https://oxynui.com it hands a page straight to the `ASSETS`
  binding, which still applies `dist/_redirects` and `dist/_headers`. The fingerprinted bundles,
  the gallery wasm and the fonts stay asset-first: no Worker invocation, no billing.
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

### Billing: `/api/billing/*`, `/api/pro/source/<slug>`, `/api/polar/webhook`

Plans, checkout redirects, the signed-in plan, Team seats, Pro code and the Polar webhook. See
[pricing.md](pricing.md) and `site/worker/billing.ts`.

### `GET /api/flags`

`200 {"ok": true, "flags": {"<name>": "<value>"}}`: the content flags that are switched on in the admin
panel (Settings), `cache-control: public, max-age=60`.

### `POST /api/events`

`202 {"ok": true, "stored": n}`: privacy-friendly page events from the site's own pages (page views,
Pro copy attempts, pricing views, checkout clicks) for the admin Activity tab. Same-origin, up to 20
events per request, rate limited per IP, bots dropped, no IP / user agent / email stored, kept 90
days. See [admin.md](admin.md#site-events-post-apievents) and `site/worker/events.ts`.

### `/api/auth/*`

Site accounts: one sign-in for everyone with Google, GitHub or an email and a password, sessions in D1. See [auth.md](auth.md).

### `/api/admin/*`

The admin panel's API: the normal site session of an account whose email Google or GitHub verified,
and that email on the D1 `admins` list. See [admin.md](admin.md).

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
| `admins` | maintainers allowed into `/admin` (with a verified sign-in); `removed_at` set = removed (0002). `access_synced` is unused since A1 |
| `settings` | key/value: `flag:<name>` (the old admin password keys were deleted by 0004) |
| `admin_sessions` | unused since A1 (emptied by 0004; the admin panel uses `sessions`) |
| `audit_log` | admin actions: time, actor, action, target, detail (0002) |
| `users` | site accounts: email, `email_verified` (1 after a Google or GitHub sign-in), name and avatar, PBKDF2 password record or NULL (0003, [auth.md](auth.md)) |
| `oauth_accounts` | linked Google and GitHub accounts: provider, provider id, user, login (0003) |
| `sessions` | site sessions: SHA-256 of the cookie token, user, provider, expiry (0003) |
| `subscriptions` | Polar subscriptions, latest state per id: account, customer, plan, interval, founder, status, amount, period end (0005, [pricing.md](pricing.md)) |
| `licences` | the licence each subscription grants its buyer (account id or email) (0005) |
| `team_seats` | emails a Team owner gave a seat (3 per Team: up to 4 people with the owner) (0005) |
| `founder_claims` | one row per new subscriber whose founder subscription became active (kept after it renews as regular Pro); 100 closes the founder rate (0005) |
| `checkout_refs` | finished Polar checkouts and the account that started them (0005) |
| `webhook_events` | Polar webhook ids already handled (replays are no-ops) (0005) |
| `grants` | manual Pro/Team grants (email, plan, seats, expiry, reason); read by `plan_for`, admin UI in G2 (0005) |

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
| `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET` | Worker secrets, optional | "Continue with Google" on `/login/`. Until set, the button reads "coming soon" and the Google routes answer 503. Setup in [auth.md](auth.md). Google or GitHub sign-in is also what opens the admin panel ([admin.md](admin.md)). |
| `POLAR_WEBHOOK_SECRET` | Worker secret | Verifying Polar webhooks (set). Unset: the webhook answers 503. [pricing.md](pricing.md) |
| `POLAR_ACCESS_TOKEN` | Worker secret | The account page's Billing section (plan switch, payments, card), Cancel / Resume, scheduling founder subscriptions to renew as regular Pro, and archiving the founder products at 100. [pricing.md](pricing.md) |
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

## Domains

| Host | What it does |
| --- | --- |
| `https://oxynui.com` | Canonical. Worker Custom Domain on the `oxynui.com` zone. |
| `https://www.oxynui.com` | Worker Custom Domain; every path answers `301` to `https://oxynui.com` + the same path and query. |
| `https://ui.oxyn.workers.dev` | The original address (`workers_dev: true`). Pages answer `301` to `https://oxynui.com`; `/api/*` is served as before. |

- Both Custom Domains are declared in `site/wrangler.jsonc` (`routes` with `custom_domain: true`),
  so a CI deploy keeps them attached.
- `/api/*` is never redirected on any host: GitHub and Google send sign-ins back to the callback
  registered for them, Polar posts its webhook to the URL saved on the endpoint, and an `oxyn` or
  `oxyn-mcp` built before the move still calls `https://ui.oxyn.workers.dev/api/r/...`. Free
  registry files (`/r/*`) redirect; the CLI and MCP server follow redirects.
- The redirect lives in `canonicalRedirect` (`site/worker/shared.ts`); `SITE_URL` in
  `wrangler.jsonc` (default `https://oxynui.com`) is the target. Other hosts (local dev, version
  preview URLs) are left alone.
- The site's own origin for canonical links, Open Graph, the sitemap and the registry index is
  `SITE` in `site/src/lib/site.mjs`. The Rust defaults are `DEFAULT_REGISTRY` (oxyn-cli) and the
  `--homepage` default (oxyn-registry); both are `https://oxynui.com`.
- Sign-in cookies are per host: someone signed in on the old address signs in again on
  https://oxynui.com.
