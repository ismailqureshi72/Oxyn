# Admin panel

`/admin` on https://ui.oxyn.workers.dev is the maintainers' panel: the feedback inbox, the list of
admins, content flags and an audit log. Every response carries
`X-Robots-Tag: noindex, nofollow, noarchive`, and none of it is in `site/dist`: the Worker serves the
page (`site/worker/admin-ui/`) only after the checks below. The only link to it is the **Admin** item
in the account menu, which only admins see.

Code: `site/worker/admin.ts` (checks + API), `site/worker/admin-ui/` (page, CSS, JS; bundled into the
Worker as text), `site/worker/auth.ts` (the site sign-in it relies on), migrations `0002_admin.sql`
and `0004_unified_admin.sql`. The rest of the backend is in [backend.md](backend.md).

## How you get in

There is **one sign-in for everyone** ([auth.md](auth.md)): Google, GitHub, or email + password.
An admin is simply a signed-in account that passes two checks, on every request to `/admin*` and
`/api/admin/*`:

1. **The email is verified.** The account has signed in with **Google or GitHub** at least once on
   this email (`users.email_verified = 1` and a linked `oauth_accounts` row). An email + password
   account alone is not enough, because nothing has checked that its owner controls the address
   (there is no email-sending service yet). Such an account on an admin email gets a page that says
   *Verify this email first* with a button to sign in with Google or GitHub; when it does, the
   provider links to the same account, the unverified password is removed and its other sessions
   end (the normal account-linking rule).
2. **The email is on the admin list**: an active row in D1 `admins`. Removing someone works on
   their very next request.

What a visitor sees:

| Who | `/admin` | `/api/admin/*` |
|---|---|---|
| Signed out | `302` to `/login/?next=/admin` (back to the panel after signing in) | `401 signed_out` |
| Signed in, email on the list but only a password account | `403` *Verify this email first* | `403 verify_email` |
| Signed in, not on the list (or removed) | `403` *No admin access* | `403 not_an_admin` |
| Signed in with Google/GitHub, on the list | the panel | the API |

No Cloudflare Access, no shared admin password and no separate admin session any more: the panel
uses the normal `oxyn_session` cookie (HttpOnly, Secure, SameSite=Lax, 30 days; see
[auth.md](auth.md)). **Sign out** in the panel signs out of the site.

Every change (`POST`, `PATCH`, `PUT`, `DELETE`) must come with `Origin` equal to the site and a JSON
body, and changes are limited to 120 per admin per 10 minutes (`429 rate_limited`). Pages are sent
with a strict Content-Security-Policy (`script-src 'self'`, no inline script or style,
`frame-ancestors 'none'`), `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` and
`Cache-Control: no-store`. The UI writes every value from the API as text, never as HTML. Only
`/admin/app.css` and `/admin/theme.js` (styles and the light/dark switch, nothing private) are
served before the checks, for the refusal pages.

## First visit

1. Make sure Google or GitHub sign-in is turned on ([auth.md](auth.md): the
   `GOOGLE_OAUTH_CLIENT_*` or `GITHUB_OAUTH_CLIENT_*` Worker secrets). Until one of them is, nobody
   can verify an email and the panel stays closed to everyone.
2. Open https://ui.oxyn.workers.dev/admin (or **Admin** in the account menu), sign in with Google or
   GitHub on your admin email. That is all.

## What is in it

- **Inbox**: feedback from the site's Feedback button. Filter by kind (Bug / Idea / Question) and
  status (New / Triaged / Closed / All), search messages and emails, open one to read the full
  message, page, reply email, browser and the screenshot (served from R2 through the Worker). Change
  the status, **Delete** (removes the D1 row and the R2 screenshot), **Send to GitHub** (opens an
  issue labelled `feedback` the same way as the automatic path; needs the `GITHUB_TOKEN` secret,
  otherwise the button is off with a note). The tab shows the number of New items.
- **Billing**: subscribers from Polar (email, plan, monthly or yearly, amount, status, renewal),
  monthly recurring revenue (yearly plans count a twelfth, live subscriptions only, before tax),
  founder spots used, and warnings when a Polar secret is missing or the founder products still need
  archiving. Read-only; refunds and plan changes happen in Polar. See [pricing.md](pricing.md).
- **Admins**: the D1 `admins` list, with whether each email is verified yet. **Add admin** takes an
  email; **Remove** asks first. The last active admin cannot be removed; removing yourself closes
  the panel for you at once.
- **Settings → Content and flags**: key/value flags in `settings`
  (`flag:<name>` = `{"enabled": bool, "value": text}`), switched on/off and edited inline.
  `GET /api/flags` (public, cached 60 s) returns the flags that are on as `{name: value}`.
- **Audit log**: the last 100 admin actions (`audit_log`): status changes, deletes, GitHub sends,
  admin adds/removes, flag edits (and, from before A1, the old shared-password sign-ins).

Keyboard: tabs and the segmented filters move with the arrow keys, the inbox list with ↑/↓,
`Esc` closes a message (and the confirm dialog), and every control has a visible focus ring. Light
and dark follow the site's theme choice (`starlight-theme`) or the system, with a toggle in the bar.

## Configuration

| Name | Kind | Value |
|---|---|---|
| `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET` | secrets | "Continue with Google" ([auth.md](auth.md)) |
| `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_OAUTH_CLIENT_SECRET` | secrets | "Continue with GitHub" ([auth.md](auth.md)) |
| `GITHUB_TOKEN` | secret, optional | **Send to GitHub** and automatic feedback issues ([backend.md](backend.md)) |

`assets.run_worker_first` in `wrangler.jsonc` lists `/admin` and `/admin/*` so these paths reach the
Worker.

## API

All under `/api/admin`, JSON, `cache-control: no-store`; errors as in [backend.md](backend.md).
Every route needs a verified admin (above).

| Route | Does |
|---|---|
| `GET /me` | `email`, `name`, `github` (whether `GITHUB_TOKEN` is set) |
| `GET /feedback?kind=&status=&q=&before=` | 50 per page, newest first, `next` cursor, counts per status |
| `GET /feedback/:id` | one row |
| `PATCH /feedback/:id` `{status}` | `new`, `triaged` or `closed` |
| `DELETE /feedback/:id` | row + screenshot |
| `GET /feedback/:id/screenshot` | the image from R2 |
| `POST /feedback/:id/github` | opens the issue (`409 github_not_configured` / `already_sent`) |
| `GET /admins`, `POST /admins` `{email}`, `DELETE /admins/:email` | admin list (`409 already_admin` / `last_admin`) |
| `GET /flags`, `PUT /flags/:key` `{enabled, value}`, `DELETE /flags/:key` | flags |
| `GET /audit?before=` | last 100 actions |

## Local development

See [auth.md](auth.md#local-development): run `worker/dev.ts` with the fake Google (and GitHub),
then sign in with the fake provider on an email that is in the local `admins` table (the 0002
migration adds the two maintainers), for example
`/api/auth/google/start?next=/admin` after choosing the test user with
`/__dev/google/authorize?as=…`. Everything after the fake provider is the production code.

## Migrations

```sh
npx wrangler d1 migrations apply oxyn --remote    # from site/; 0002 and 0004 are additive
```

`0004_unified_admin.sql` deletes the old shared-password hash, the admin-session HMAC key and all
rows of `admin_sessions` (the table stays, unused), and marks every account with a linked provider
as verified.

## History: Cloudflare Access (until A1)

Until A1 the panel sat behind a Cloudflare Access application (**Oxyn admin**, email one-time PIN)
plus a shared admin password. The Access application and its policy were deleted when A1 shipped.
The Zero Trust org still has the one-time-PIN login method and the **oxyn-admins** group; they are
unused and harmless, and can be deleted in the Zero Trust dashboard.
