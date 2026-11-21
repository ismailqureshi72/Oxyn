# Admin panel

`/admin` on https://ui.oxyn.workers.dev is the maintainers' panel: site activity, every account and
its plan, grants (Pro or Team without payment), the feedback inbox, billing, the list of admins,
content flags and an audit log. Every response carries
`X-Robots-Tag: noindex, nofollow, noarchive`, and none of it is in `site/dist`: the Worker serves the
page (`site/worker/admin-ui/`) only after the checks below. The only link to it is the **Admin** item
in the account menu, which only admins see.

Code: `site/worker/admin.ts` (checks + API), `site/worker/access.ts` (Users, Grants, Activity),
`site/worker/emailcheck.ts` (grant email checks), `site/worker/events.ts` (site events),
`site/worker/admin-ui/` (page, CSS, JS; bundled into the Worker as text), `site/worker/auth.ts`
(the site sign-in it relies on), migrations `0002_admin.sql`, `0004_unified_admin.sql` and
`0006_admin_access.sql`. The rest of the backend is in [backend.md](backend.md).

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

The panel opens on **Activity**. Times are shown in the admin's own time zone (Pakistan time for us),
and the day buckets of the charts follow it too.

- **Activity**: accounts active in the last day / 7 / 30 days (an account counts when it signs in or
  browses while signed in), visitors and page views today, MRR and ARR (same numbers as Billing),
  active subscriptions by plan, churn (subscriptions canceled or ended in the last 30 days, and how
  many are set to cancel), grants in effect; signups and page views per day (7, 30 or 90 days);
  sign-ins by provider; pricing views → checkout clicks → paid subscriptions; the most viewed
  components and screens; and **Pro copy attempts** per item (Copy button, Ctrl/⌘+C or the CLI
  command on a Pro item without a plan, which leads to pricing): the clearest sign of what people
  want to buy. Charts are plain SVG and CSS.
- **Users**: total signups with today / 7 / 30 days and the change against the period before; one
  row per account with its email, name, sign-in methods, verified (Google/GitHub) or not, signed
  up, last active and **current plan**: Free, Pro, Founder, Team owner, Team member (a seat on a
  paid Team) or Granted (with what was granted and until when), plus where it comes from (Polar,
  a seat, a grant) and a note when a grant is waiting for the email to be verified. Search by email
  or name, filter by plan, sort, 50 per page, **Export CSV** (the filtered list; cells that would
  start a spreadsheet formula are prefixed with `'`). Click a row for the account: its grants
  (Revoke / Restore), its Polar subscriptions and **Change plan**: Free (removes the grant), Pro
  (optionally showing the founder rate; display only, nothing is charged) or Team (the account as
  owner plus up to 3 teammate emails, 4 people in all). A plan paid through Polar cannot be removed
  here: the panel says so and links **Manage in Polar**. **Pro items this month** shows the
  account's count against its plan's caps (20 components + 8 screens for Pro, 75 + 22 shared for a
  Team; see [pricing.md](pricing.md#monthly-caps-decided-2026-11-21)), what was opened and from where
  (site, CLI, MCP), and **Reset this month** frees the count (for a Team member: the whole Team's).
- **Grants**: give Pro or Team without payment, to one email, a Team (owner + up to 3) or a bulk
  paste (up to 200, one per line or separated by commas). The grant can be given **before or after
  someone signs up**: it turns on when the person signs in with Google or GitHub on that exact
  email; an email + password account does not get it until Google or GitHub verified the email.
  Safeguards (the browser shows them at once, the Worker checks everything again and decides):
  - format check and normalising (trim, lower case, `Name <email>` unwrapped); Gmail dots and
    `+tags` are kept as typed, with a note that the grant only matches that exact address;
  - **typo check** on common domains (`gmial.com`, `gamil.com`, `gmail.co`, `hotmial.com`,
    `yahooo.com`, `outlok.com`, a wrong `.con`/`.cmo`, or one letter off a big provider): "Did you
    mean …?" with **Use** and **Keep as typed**; a typo is never granted unless kept;
  - duplicates in the list, an email that already has an active grant (refused: revoke it or change
    the plan instead), an email that already pays through Polar (refused unless "grant anyway");
  - a **reason** is required, the **end date** is optional and must be in the future (end of that
    day, admin's time);
  - **Check and preview** shows a line-by-line report and writes nothing; in a bulk paste the lines
    that do not pass are left out and listed;
  - the confirm dialog asks to **type the email** (or, for a bulk grant, the number of emails);
    more than 20 also needs "I checked all N emails"; at most 300 grant emails per admin per hour;
  - **Revoke** is soft (a Team owner's grant takes its teammates along) with **Undo** for 30
    seconds, and **History** keeps every revoked grant with **Restore**. Nothing is ever deleted.
  Every add, revoke, restore and plan change is in the audit log with the admin's email and the
  state before and after.
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
  email; **Remove** asks first. The last active admin cannot be removed, and removing yourself asks
  you to type your own email (the Worker checks it too) because it closes the panel for you at once.
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
| `GET /users?q=&plan=&sort=&dir=&page=&size=&tz=&format=csv` | accounts with plan and source, summary counts, CSV |
| `GET /users/:id`, `POST /users/:id/plan` | one account (with `usage`: this month's Pro items of its pool); change its plan (`409 paid_subscription` for a Polar plan) |
| `POST /users/:id/usage/reset` | frees this month's Pro items of the account's pool (a Team's for a member), audited `usage.reset` |
| `GET /grants?status=active\|pending\|revoked\|expired\|all&q=` | grants, Team members nested under the owner |
| `POST /grants/check` | per-line report, writes nothing |
| `POST /grants` | adds (`409 duplicate` / `typo_suspected` / `paid_subscription`, `400 confirm_mismatch` / `reason_required` / `expiry_in_past` / `bulk_ack_required`) |
| `POST /grants/:id/revoke`, `POST /grants/:id/restore` | soft revoke and its undo |
| `GET /activity?days=7\|30\|90&tz=` | the Activity numbers |

`DELETE /admins/:email` for your own email needs `?confirm=<your email>` (`400 confirm_self`).

## Site events (`POST /api/events`)

The Activity tab reads a small event log the site writes itself, built to stay privacy-friendly:

- **What**: page views, `pricing_view` (with the Pro item that led there), `pro_copy` (item and how:
  copy, Ctrl/⌘+C, CLI), `checkout_click` (plan key), and sign-ins per provider (written by the
  Worker). Paths only, never query strings; `/admin` and `/api` are never recorded.
- **Who**: no IP, user agent, email or referrer is stored. `visitor` is a salted hash of IP + user
  agent + the UTC day, so a visitor is counted once a day but cannot be followed across days. The
  account id is stored only for a signed-in visitor and is read from the session, never from the
  request.
- **Limits**: same-origin only, up to 20 events per request (the site batches them and sends with
  `sendBeacon` when a page is left), 30 requests a minute and 300 an hour per IP (`429`), bots and
  headless browsers are accepted and dropped (`202`, `stored: 0`), nothing is sent when Do Not
  Track or Global Privacy Control is on, no cookie is set. Rows older than **90 days** are deleted.

## Local development

See [auth.md](auth.md#local-development): run `worker/dev.ts` with the fake Google (and GitHub),
then sign in with the fake provider on an email that is in the local `admins` table (the 0002
migration adds the two maintainers), for example
`/api/auth/google/start?next=/admin` after choosing the test user with
`/__dev/google/authorize?as=…`. Everything after the fake provider is the production code.

## Migrations

```sh
npx wrangler d1 migrations apply oxyn --remote    # from site/; 0002, 0004, 0006 and 0007 are additive
```

`0004_unified_admin.sql` deletes the old shared-password hash, the admin-session HMAC key and all
rows of `admin_sessions` (the table stays, unused), and marks every account with a linked provider
as verified.

## History: Cloudflare Access (until A1)

Until A1 the panel sat behind a Cloudflare Access application (**Oxyn admin**, email one-time PIN)
plus a shared admin password. The Access application and its policy were deleted when A1 shipped.
The Zero Trust org still has the one-time-PIN login method and the **oxyn-admins** group; they are
unused and harmless, and can be deleted in the Zero Trust dashboard.
