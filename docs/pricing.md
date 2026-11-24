# Pricing, Pro and billing

74 of Oxyn's components are free; everything else is Oxyn Pro. Pro and Team are subscriptions sold through
[Polar](https://polar.sh), which is the merchant of record: Polar takes the payment, adds sales tax
or VAT at checkout and sends receipts and invoices. Everything after the checkout happens on the
site's own account page (see Billing on the site); the site never sees card details.

## Plans

| Plan | Monthly | Yearly | Who |
| --- | --- | --- | --- |
| Free | $0 | $0 | Everyone. The 74 free components (the Free list below), docs, CLI. MIT. |
| Pro | $19.99 | $199.99 | One developer. |
| Pro, founder rate | $12.99 first month, then $19.99 | $124.45 first year, then $199.99 | Intro price for the first 100 new subscribers; same Pro. |
| Team | $44.99 | $449 | A team of up to 4 people: the owner plus 3 teammates. |

Prices are US dollars before tax. There is no lifetime plan.

**Pro includes** every component outside the Free list, every screen, agent-app template, block
and kit, and the AI components, with their code, weekly drops, Studio extras and the Pro CLI and
MCP. Every Pro preview stays open to everyone. New components ship as Pro.

**Admins** (the owners `m.ismailqureshi62@gmail.com`, `khubaibnazeer8@gmail.com` and
everyone on the D1 `admins` list) have all access: `plan_for()` in `site/worker/billing.ts`
returns tier `team`, `pro: true`, `admin: true` for them once Google or GitHub verified their
email, so the Pro source endpoint and the Pro registry (P3) open, no upgrade prompt or Get Pro
button shows, and the account menu says Admin.

## The Free list

`site/src/lib/free-tier.json` is the single source of truth for what is free. Everything reads it:
the site (`isFree` / `isPro` in `site/src/lib/taxonomy.mjs` and `data.ts`: the bolt marker, the
locked code panel, the copy and CLI gates), the Worker (only non-free items go into
`worker/pro-src.json`, served by `/api/pro/source/<slug>` behind `plan_for()`), the CLI registry
(`site/scripts/gen.mjs` marks each item's tier from it, and only free items get a public
`r/<slug>.json`), and `docs/parity/status.csv` (the `oxyn_tier` column, written by
`site/scripts/sync-tiers.mjs`; CI fails when it is stale).

**How it was chosen (November 2026).** By the amount of work in each component, scored by
`site/scripts/score-free.mjs` over the site catalogue: 3 points per variant (up to 12), 2 per
playground control (up to 12), 8 for a GPUI build next to the Dioxus one, 1 per spec section (up to
12) plus 1 per 1,500 characters of spec (up to 8), half a point per accessibility, keyboard and
responsive note (up to 12) and per prop (up to 16), and 2 per built parity row in
`docs/parity/status.csv` (up to 3). Only components are candidates (screens, agent apps, blocks and
the AI components are Pro by definition). Components are taken in score order together with the
components they render (from the registry's dependencies), so every free component installs and
compiles on its own, up to 74. Re-running the script is a pricing decision, never part of a build.

The 74 free items: `crew-tree`, `presence-cursors`, `agent-trail`, `approval-card`, `attachments`, `autonomy-levels`, `avatar`, `badge`, `button`, `button-group`, `calendar`, `card`, `chat-composer`, `chat-thread`, `checkbox`, `chip`, `code-block`, `code-suggestion`, `crew-chat`, `combobox`, `command-palette`, `composer`, `confirm-popover`, `context-treemap`, `context-meter`, `spend-breakdown`, `date-wheel`, `decision-tree`, `dialog`, `diff-timeline`, `diff-viewer`, `dropdown-menu`, `effort-control`, `elapsed-timer`, `fallback-chain`, `task-grid`, `file-dropzone`, `glyph-loader`, `inline-diff`, `inline-edit`, `model-knockout`, `memory-panel`, `message-bubble`, `model-picker`, `otp-input`, `password-strength`, `phone-input`, `queue-stack`, `rating`, `reasoning-block`, `response-feedback`, `run-controls`, `scheduled-runs`, `segmented-control`, `select`, `send-control`, `skeleton`, `slider`, `spinner`, `stepper`, `streaming-markdown`, `subagent-tree`, `switch`, `tabs`, `tag-input`, `text-field`, `text-reveal`, `text-shimmer`, `tool-reach`, `tool-call`, `trace-waterfall`, `verb-cycler`, `voice-recorder`, `workspace-sidebar`.

**Founder rate.** An intro price, not a locked-in one: $12.99 for the first month (or $124.45 for
the first year), then $19.99/mo ($199.99/yr). First 100 new subscribers.

- *Renewal at the regular price.* When a founder subscription becomes active, the webhook schedules
  a product change in Polar to the matching regular product (Founder Monthly → Pro Monthly, Founder
  Yearly → Pro Yearly): `PATCH /v1/subscriptions/{id} {"product_id": …, "proration_behavior":
  "next_period"}`. Nothing is charged now; the first renewal bills the regular price. It is
  idempotent (skipped when `pending_update` already targets the Pro product; the PATCH's own
  `subscription.updated` webhook lands on that check) and waits while the subscription is set to
  cancel (un-cancelling sends a webhook that schedules it). The outcome is kept in `settings`
  (`founder_renewal:<subscription id>`); the admin Billing tab lists founder subscriptions not yet
  scheduled and has **Schedule now** (`POST /api/admin/billing/founder-renewals`).
- *In D1.* At renewal Polar switches the product, the next webhook writes the row with the Pro
  product and `founder = 0`: `founder` means "in the founder intro period". The plan stays `pro`, the
  monthly caps do not change, and the `founder_claims` row stays, so the spot stays counted.
- *New subscribers only.* `/api/billing/checkout` sends anyone whose account or email had an Oxyn
  subscription before to the regular price (`/pricing/?founder=used`), and the pricing page hides
  the founder rate for a signed-in account with a past subscription. A founder subscription that
  still arrives from a returning customer (a direct Polar link) takes no founder spot and renews at
  the regular price like any other.
- *Not in the customer portal.* Both founder products are `visibility: private` in Polar, so they
  can only be bought through their checkout links and a subscriber cannot switch to them from the
  portal.

The first 100 founder subscriptions that become active are counted in D1
(`founder_claims`). From the 100th on, `/api/billing/plans` reports the founder rate as closed, the
pricing page stops showing it, `/api/billing/checkout` refuses founder plans, and the Worker archives
both founder products in Polar (`PATCH /v1/products/{id} {"is_archived": true}`) so a direct link
cannot start a checkout either. Existing founder subscriptions are unaffected and renew at the
regular price as above. Archiving needs the `POLAR_ACCESS_TOKEN` Worker secret; without it the admin panel shows the
archive as pending.

**Team.** A Team is up to 4 people in total. The owner adds up to three teammate emails on the Account page (`team_seats`). A seat opens Pro for
the account with that email once Google or GitHub has verified the email (an email and password
account alone is not enough, so nobody can take a seat by signing up with someone else's address).

**Cancelling.** Code already installed keeps working. When the subscription ends (Polar's
`subscription.revoked`), new Pro access stops: the Pro code on the site, the Pro registry and the
Pro CLI. A cancellation at the end of the period keeps Pro open until that date.

## Polar setup (production, organization `oxyn`)

| Plan key | Product id | Checkout link |
| --- | --- | --- |
| `pro-month` | `677990ee-37fd-4144-a7b4-fc0072db0236` | https://buy.polar.sh/polar_cl_nOTwWhBDBia5V47drawKkcEPM1g1ClF73jekN2cDk0k |
| `pro-year` | `6c6f941c-eb68-42a1-9588-6999793d7723` | https://buy.polar.sh/polar_cl_TIpeARSu8pHt9MTbFwisPzixrXIdJhlEBJOHG2B2fYk |
| `pro-founder-month` | `dd6c77e0-537e-4575-8272-179818c4636c` | https://buy.polar.sh/polar_cl_1DDghPtFc5Mtf4xW8kEUbXO68TclXcBADP5xN1UD4x6 |
| `pro-founder-year` | `b2c7ff7d-5ed5-4677-b4c2-023860cf6440` | https://buy.polar.sh/polar_cl_coirJZKmfkFV75qqxxVDvjQO9bmbnqmAnQd9T2Xrezv |
| `team-month` | `47ac4829-9dea-4c5f-a47c-948eb3005865` | https://buy.polar.sh/polar_cl_NCvpX9ojhDrafHw1bLtki9PTVUOePRTSPrUgL2ipK4Y |
| `team-year` | `c9ad78fa-8fd7-4f81-a19a-af4c6472f000` | https://buy.polar.sh/polar_cl_tGlTFd4OLxE5lYiu8NWR2eDjPRkrJGfUUcEwd1eo3vC |

Every checkout link has the success URL
`https://oxynui.com/pricing/success?checkout_id={CHECKOUT_ID}` and the return URL
`https://oxynui.com/pricing/` (set on Polar; moved from `ui.oxyn.workers.dev` with the domain). The ids and links live in `PLANS` in
`site/worker/billing.ts`; prices shown on the site are `PRICES` in `site/src/lib/data.ts`. Change
both together, and never create a second product for an existing plan.

Webhook endpoint `3e81f336-ecd1-4446-aaa0-7598e1a055da`: `https://oxynui.com/api/polar/webhook`
(`https://ui.oxyn.workers.dev/api/polar/webhook` still answers: `/api/*` is not redirected),
format raw, API version 2026-10, events `subscription.created`, `subscription.updated`,
`subscription.active`, `subscription.canceled`, `subscription.revoked`, `order.paid`,
`checkout.updated`.

### Worker secrets (Worker `ui`)

| Secret | Needed for | Without it |
| --- | --- | --- |
| `POLAR_WEBHOOK_SECRET` | Verifying webhooks (Standard Webhooks). | The webhook answers 503; no subscription is recorded. |
| `POLAR_ACCESS_TOKEN` | The Billing section of the account page (plan switch, payments and receipts, the card on file, Update card), Cancel / Resume, scheduling founder subscriptions to renew as regular Pro, archiving the founder products at 100. Scopes: `subscriptions:read/write`, `orders:read`, `customers:read`, `customer_sessions:write`, `products:write`; checked daily (D1 settings `polar_check`, shown in the admin panel). | The Billing section and Cancel plan answer 503; founder renewals and the founder archive stay pending in the admin panel (the site still stops selling it). |

Set them from `site/` with `npx wrangler secret put <NAME>`. An organization access token from Polar
(Settings, Developers) with `products:write`, `subscriptions:read`, `subscriptions:write` and `customer_sessions:write` is enough for
`POLAR_ACCESS_TOKEN`.

## How a sale flows

1. The pricing page links to `/api/billing/checkout?plan=<key>`. Signed out, that goes to
   `/login/?next=/pricing/?buy=<key>`, and the pricing page carries on to checkout after sign-in.
2. Signed in, the Worker redirects to the plan's checkout link with `customer_email`,
   `customer_name` and `reference_id=<account id>` (stored by Polar in the checkout metadata). An
   account that already has a live subscription goes to its Account page instead.
3. Polar sends `checkout.updated` (status `succeeded`) and the `subscription.*` events. The Worker
   checks the signature (HMAC-SHA256 over `id.timestamp.body`; both Standard Webhooks and Polar's
   older key derivation), ignores replays (`webhook_events`) and older states (`polar_modified_at`),
   and upserts `subscriptions` and `licences`. The account is found by `reference_id`, then the
   checkout, then Polar's external customer id; otherwise the licence matches the buyer's email
   once Google or GitHub verified it.
4. `/pricing/success/` polls `/api/billing/me` until the plan is on.

## Cancelling

**Cancel plan** on the Account page (plan section) cancels at the period end: no refund, no
renewal, and Pro stays open until `current_period_end`. After a confirm step the page shows
"Cancels on <date>" with **Resume plan**. Both call the Worker as the signed-in subscription owner
(a Pro subscriber, or a Team owner; a teammate's seat and a grant have nothing to cancel), from the
site's own origin:

- `POST /api/billing/cancel` → Polar `PATCH /v1/subscriptions/{id} {"cancel_at_period_end": true}`
- `POST /api/billing/resume` → the same with `false`

The Worker writes Polar's answer to D1 straight away (the `subscription.updated` webhook that
follows agrees with it) and needs `POLAR_ACCESS_TOKEN`; without it the routes answer 503. Resuming a founder subscription in its
first period schedules its regular-price renewal again.

## Billing on the site

Subscribers manage billing on `/account/#billing`, never in Polar's customer portal. The Worker
reads Polar live with the organization token (no `organization_id` in any call):

- **Summary**: plan, price, next payment (a founder subscription shows the switch to the regular
  price at its renewal), the card on file (`GET /v1/customers/{id}/payment-methods`).
- **Change plan** (subscription owner only; a teammate sees nothing to change; founder products are
  never offered): a confirm step shows what is charged and when (`GET /api/billing/preview`), then
  `POST /api/billing/change` sends `PATCH /v1/subscriptions/{id}` with this rule (`changeRule`):

  | Switch | Takes effect | `proration_behavior` | Charged now |
  |---|---|---|---|
  | Pro → Team, same interval | at once | `invoice` | the price difference for the rest of the period (estimate shown; Polar adds tax) |
  | monthly ↔ yearly, Team → Pro, any interval change | next renewal | `next_period` | nothing; the renewal bills the new plan |

  One change is scheduled at a time: a pending one (an earlier switch, or a founder subscription's
  renewal as Pro) is cleared first and put back if Polar refuses. **Keep my current plan**
  (`POST /api/billing/keep`) drops a scheduled switch; a founder subscription keeps its renewal as
  Pro. Cancelling subscriptions cannot switch until resumed; past-due ones until the card works.
- **Payments and invoices**: `GET /v1/orders/?customer_id=` for the account's own Polar customers
  only (never a Team owner's); **Receipt** and **Invoice** PDFs through
  `/v1/orders/{id}/receipt|invoice` after checking the order belongs to the account. An invoice
  needs a billing name and address on the order; without them only the receipt is offered.
- **Update card** is the one Polar page: Polar's API adds a card only from a Stripe confirmation
  token made with Polar's own Stripe account, so `/api/billing/card` opens Polar's hosted page with a
  pre-signed customer session and a way back to the account page. Plan changes are switched off in
  Polar's customer portal (organization `customer_portal_settings.subscription.update_plan: false`),
  so the portal cannot route around the rule above. The old `/api/billing/portal` link now lands on
  `/account/#billing`.

## Who has Pro: `plan_for(user)`

One helper in `site/worker/billing.ts` answers "what may this account use", and every check
(`/api/billing/me`, `/api/pro/source/*`, checkout, seats, billing) goes through it. It reads:

1. the account's own Polar subscriptions (via `licences`): live means status `active`, `trialing` or
   `past_due` (Polar is retrying the card) and not ended;
2. a seat on a live Team subscription;
3. a manual grant (`grants`, D1 migration 0005): unrevoked and with `expires_at` empty or in the
   future, it counts exactly like an active subscription. Grants are given, revoked and restored in
   the admin panel (**Grants**, or **Users → Change plan**; see [admin.md](admin.md)). A Team grant
   is the owner's row plus one row per teammate (`parent_id`, migration 0006).

Emails (licence fallback, seats, grants) only count once Google or GitHub verified them.

## Pro on the site

Pro items (every screen and agent-app template; blocks and kits as they ship) carry a Pro badge.
Their previews are open to everyone. Their code is not in the static pages at all: `gen.mjs`
writes it to `site/worker/pro-src.json`, which is bundled into the Worker, and
`GET /api/pro/source/<slug>` hands it out only when `plan_for` says Pro. The live WASM playground
leaves the generated code out for Pro items (`hide_code`). Without a plan, the Copy buttons,
Ctrl/Cmd+C inside the locked code, and copying the Pro CLI command (`oxyn add pro/<slug>`) all go to
`/pricing/?from=<slug>`, which names the item and offers the way back after checkout
(`public/pro.js`, `public/pricing.js`).

A Pro account opens an item's code with **Open code** (or a Copy button) on its page; that is one
of the month's Pro items (below), and the code then shows on every visit that month.

The repository itself is public and MIT licensed, so the gate is about the site, the Pro registry
and the Pro CLI, not about hiding source on GitHub.

## Monthly caps (decided 2026-11-21)

| Plan | Pro components | Screens (incl. blocks and agent apps) |
|---|---|---|
| Free | 0 | 0 |
| Pro, founder rate | 20 | 8 |
| Team | 75, shared by its people | 22, shared |
| Admins | no limit | no limit |

- **What counts:** each distinct Pro item once per billing month per pool, the first time its code
  is handed out: `GET /api/r/<slug>.json` (CLI, MCP) or `POST /api/pro/source/<slug>` (the site's
  Open code / Copy). One function does it, `unlock()` in `site/worker/pro.ts`, so the site, the
  CLI and the MCP server share one count. Re-downloads, updates and diffs of an item already opened
  that month are free; so are Pro dependencies fetched with `?for=<item>` for an item the pool
  already opened (the Worker checks the dependency closure from the registry index).
- **Kind** comes from the registry index (`site/public/r/index.json`): `component` counts as a
  component; `screen`, `block` and `app` count as screens. Tiers still come only from
  `site/src/lib/free-tier.json`.
- **Pool:** one account for Pro and Founder (and admins, uncapped); one Team for everyone on it
  (`team:<subscription>` for a paid Team and its seats, `grant:<owner grant>` for a granted Team).
- **Billing month:** monthly steps from the Polar subscription's `current_period_start` (stored by
  the webhook from 0007; `current_period_end` is used as the anchor for older rows), so yearly
  plans also get monthly caps on their billing day. Grants use the calendar month (UTC).
- **At the limit** the API answers `403 quota_exceeded` with the count and the reset date; the
  CLI exits 3. Admins can reset an account's month on its admin page (audited `usage.reset`).
- **Tables (0007):** `pro_unlocks` (one row per pool, month and item; a reset stamps `reset_at`),
  `cli_tokens` (SHA-256 only, shown once), `cli_auth_requests` (`oxyn login`, 10 minutes).

## Pro registry

`gen.mjs` runs `oxyn-registry --pro-out site/public/_pro/r`, so each installable Pro item's JSON
(same schema as the free ones) ends up in `dist/_pro/r/` as a Worker static asset. `wrangler.jsonc`
runs the Worker first for `/_pro` and `/_pro/*`, and the Worker answers 404 to every request there;
only `pro.ts` reads those files, through the `ASSETS` binding, after checking the token and the
cap. This was chosen over R2 because it ships atomically with the `index.json` that names the
items (one `wrangler deploy`), needs no upload step or R2 write permission in CI, and costs nothing
per request. Cloudflare redirects non-canonical paths (`/%5Fpro/…`, `//_pro/…`) to the canonical
one before the asset lookup, so the route cannot be bypassed by encoding.

| Route | Who | Does |
|---|---|---|
| `GET /api/r/<slug>.json[?for=<item>]` | Bearer token | Pro item JSON (free items too, no token) |
| `GET /api/cli/whoami` | Bearer token | email, tier, admin, `usage` |
| `GET/POST /api/cli/tokens`, `DELETE /api/cli/tokens/<id>` | site session | list, create (shown once), revoke |
| `POST /api/cli/auth/start`, `POST /api/cli/auth/poll` | the CLI | `oxyn login` device flow |
| `GET /api/cli/auth/request?code=`, `POST /api/cli/auth/approve` | site session | `/cli/auth/` page |

## Admin

The admin panel's Billing tab lists subscribers (email, plan, billing period, amount, status,
renewal) and sums monthly recurring revenue: monthly plans at their amount, yearly plans at a
twelfth, live subscriptions only, before tax and Polar's fee.
