# Admin panel

`/admin` on https://ui.oxyn.workers.dev is the maintainers' panel: the feedback inbox, the list of
admins, the shared password, content flags and an audit log. It is not linked from anywhere on the
public site, every response carries `X-Robots-Tag: noindex, nofollow, noarchive`, and none of it is
in `site/dist`: the Worker serves the page (`site/worker/admin-ui/`) only after the checks below.

Code: `site/worker/admin.ts` (gates + API), `site/worker/admin-ui/` (page, CSS, JS; bundled into the
Worker as text), `site/migrations/0002_admin.sql`. The rest of the backend is in
[backend.md](backend.md).

## How you get in

Three checks, on every request to `/admin*` and `/api/admin/*`:

1. **Cloudflare Access** (Zero Trust org `oxyn`, team domain `holy-block-5356.cloudflareaccess.com`).
   The self-hosted application **Oxyn admin** covers `ui.oxyn.workers.dev/admin`, `/admin/*`,
   `/api/admin` and `/api/admin/*`. Login method: **email one-time PIN**. Policy **oxyn-admins only**
   allows the Access group **oxyn-admins**. Anyone else is stopped at the edge and sent to the
   Access login page; the Worker never sees them. Access session: 12 hours.
2. **The Worker checks the Access JWT itself.** It reads `Cf-Access-Jwt-Assertion`, verifies the
   RS256 signature against the team's keys (`/cdn-cgi/access/certs`, cached for an hour, refetched
   when a new key id appears), the application audience (`ACCESS_AUD`), the issuer (the team
   domain) and expiry, and takes the email from the token. A header alone is never trusted. The
   email must then be an active row in the D1 `admins` table, so removing someone works at once
   even before the Access group is updated. Failures return `401 access_required` or
   `403 not_an_admin`.
3. **Shared password.** The first admin to open `/admin` sees **Set the shared password** (at least
   12 characters, typed twice) and chooses it; after that everyone signs in with it. It is stored in
   D1 `settings` (`admin_password`) as PBKDF2-SHA256, 100 000 iterations (the most the Workers
   runtime allows), 16-byte random salt; it is compared in constant time and never logged. Wrong
   passwords are limited to 5 per Access email per 15 minutes and 30 per client IP per day
   (`429 too_many_attempts`).

Signing in creates a row in `admin_sessions` and sets `__Host-oxyn_admin` (HttpOnly, Secure,
SameSite=Strict, Path=/, 12 hours). The cookie holds a random session id and its HMAC-SHA256
signature. A session only counts for the Access email that created it and for the current password
version, so changing the password signs everyone out. **Log out** deletes the session server-side.
The HMAC key is the `ADMIN_SESSION_KEY` Worker secret when set, else 32 random bytes the Worker
generates once into `settings` (`admin_session_key`). Every change (`POST`, `PATCH`, `PUT`,
`DELETE`) must also come with `Origin` equal to the site.

Pages are sent with a strict Content-Security-Policy (`script-src 'self'`, no inline script or
style, `frame-ancestors 'none'`), `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` and
`Cache-Control: no-store`. The UI writes every value from the API as text, never as HTML.

## First visit (one-time)

1. Open https://ui.oxyn.workers.dev/admin, enter your email on the Cloudflare Access page and type
   the PIN it mails you.
2. Choose the shared password on the setup screen. Share it with the other maintainers privately
   (not in the repo, issues or chat logs). Nobody else, including automation, knows it.
3. To change it later: **Settings → Shared password** (needs the current one).

If the password is lost, a maintainer with Cloudflare access can reset the panel to the setup
screen:

```sh
npx wrangler d1 execute oxyn --remote --command "DELETE FROM settings WHERE key = 'admin_password'; DELETE FROM admin_sessions"
```

## What is in it

- **Inbox**: feedback from the site's Feedback button. Filter by kind (Bug / Idea / Question) and
  status (New / Triaged / Closed / All), search messages and emails, open one to read the full
  message, page, reply email, browser and the screenshot (served from R2 through the Worker). Change
  the status, **Delete** (removes the D1 row and the R2 screenshot), **Send to GitHub** (opens an
  issue labelled `feedback` the same way as the automatic path; needs the `GITHUB_TOKEN` secret,
  otherwise the button is off with a note). The tab shows the number of New items.
- **Admins**: the D1 `admins` list. **Add admin** takes an email; **Remove** asks first. The last
  active admin cannot be removed; removing yourself signs you out. Each change is synced to the
  Access group (below); until it is, rows show *Pending Access sync*.
- **Settings**: change the shared password; **Content and flags**: key/value flags in `settings`
  (`flag:<name>` = `{"enabled": bool, "value": text}`), switched on/off and edited inline.
  `GET /api/flags` (public, cached 60 s) returns the flags that are on as `{name: value}`.
  `promo_strip` is created by the migration (off); the site does not read it yet.
- **Audit log**: the last 100 admin actions (`audit_log`): sign-ins, wrong passwords, password
  set/change, status changes, deletes, GitHub sends, admin adds/removes, Access syncs, flag edits.
  Passwords are never written to it.

Keyboard: tabs and the segmented filters move with the arrow keys, the inbox list with ↑/↓,
`Esc` closes a message (and the confirm dialog), and every control has a visible focus ring. Light
and dark follow the site's theme choice (`starlight-theme`) or the system, with a toggle in the bar.

## Keeping the Access group in sync: `CF_ADMIN_TOKEN`

Adding or removing an admin updates D1 at once. To also update the Access group **oxyn-admins**,
the Worker needs a Cloudflare API token as the `CF_ADMIN_TOKEN` secret. Without it the Admins page
shows *Access sync is off*: a removed admin is still refused by the Worker, but a new admin cannot
get past Access until the group lists them (add them in the Zero Trust dashboard, or set the token
and press **Sync to Access now**).

Create the token (one-time, by an account owner):

1. Cloudflare dashboard → **My Profile → API Tokens → Create Token → Create Custom Token**.
2. Permissions: **Account → Access: Organizations, Identity Providers, and Groups → Edit**. Nothing
   else.
3. Account resources: **Include → the Oxyn account** (`773b642dbd01a6ff88b33ad8af7251cc`).
4. Create it, then from `site/`:

```sh
npx wrangler secret put CF_ADMIN_TOKEN      # paste the token
```

The sync reads the group, keeps any non-email rules, replaces the email rules with the active admin
list and saves it; then pending rows are marked synced and removed rows are deleted. A failed sync
is shown on the Admins page with the error and can be retried.

## Configuration

| Name | Kind | Value |
|---|---|---|
| `ACCESS_TEAM_DOMAIN` | var (`wrangler.jsonc`) | `https://holy-block-5356.cloudflareaccess.com` |
| `ACCESS_AUD` | var | audience tag of the **Oxyn admin** Access application |
| `ACCESS_GROUP_ID` | var | id of the **oxyn-admins** Access group |
| `CF_ACCOUNT_ID` | var | Cloudflare account id |
| `CF_ADMIN_TOKEN` | secret, optional | Access group sync (above) |
| `GITHUB_TOKEN` | secret, optional | **Send to GitHub** and automatic feedback issues ([backend.md](backend.md)) |
| `ADMIN_SESSION_KEY` | secret, optional | HMAC key for session cookies; generated into D1 when unset |

Missing `ACCESS_TEAM_DOMAIN` or `ACCESS_AUD` closes the panel (`503 admin_not_configured`) rather
than opening it. `assets.run_worker_first` in `wrangler.jsonc` lists `/admin` and `/admin/*` so
these paths reach the Worker.

## API

All under `/api/admin`, JSON, `cache-control: no-store`; errors as in [backend.md](backend.md).

| Route | Needs | Does |
|---|---|---|
| `GET /me` | Access | email, `password_set`, `signed_in`, `github`, `access_sync` |
| `POST /setup` `{password, confirm}` | Access, no password yet | sets the password, signs in (`409 already_set` after the first) |
| `POST /login` `{password}` | Access | signs in (`401 wrong_password`, `429 too_many_attempts`) |
| `POST /logout` | Access | ends the session |
| `GET /feedback?kind=&status=&q=&before=` | session | 50 per page, newest first, `next` cursor, counts per status |
| `GET /feedback/:id` | session | one row |
| `PATCH /feedback/:id` `{status}` | session | `new`, `triaged` or `closed` |
| `DELETE /feedback/:id` | session | row + screenshot |
| `GET /feedback/:id/screenshot` | session | the image from R2 |
| `POST /feedback/:id/github` | session | opens the issue (`409 github_not_configured` / `already_sent`) |
| `GET /admins`, `POST /admins` `{email}`, `DELETE /admins/:email`, `POST /admins/sync` | session | admin list and Access sync |
| `POST /password` `{current, password, confirm}` | session | change password; ends all other sessions |
| `GET /flags`, `PUT /flags/:key` `{enabled, value}`, `DELETE /flags/:key` | session | flags |
| `GET /audit?before=` | session | last 100 actions |

## Local development

```sh
cd site
npx wrangler d1 migrations apply oxyn --local
npx wrangler dev worker/dev.ts --var DEV_ADMIN_EMAIL:m.ismailqureshi62@gmail.com   # http://localhost:8787/admin
```

`worker/dev.ts` replaces only the Access JWT check with a stub that trusts `DEV_ADMIN_EMAIL`, and
only for requests to `localhost` / `127.0.0.1`. It is never deployed: `wrangler.jsonc`'s `main` is
`worker/index.ts`, and CI fails if the stub's marker appears in the deploy bundle. The email must
be in the local `admins` table (the migration adds the two maintainers). Everything after that
(admin list, password, sessions, rate limits) is the production code.

## Applying the migration

```sh
npx wrangler d1 migrations apply oxyn --remote    # from site/; 0002_admin.sql is additive
```
