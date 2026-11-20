# Pricing, Pro and billing

Oxyn's components are free. Pro and Team are subscriptions sold through
[Polar](https://polar.sh), which is the merchant of record: Polar takes the payment, adds sales tax
or VAT at checkout, sends receipts and invoices, and runs the customer portal. The site never sees
card details.

## Plans

| Plan | Monthly | Yearly | Who |
| --- | --- | --- | --- |
| Free | $0 | $0 | Everyone. Every component and primitive, docs, CLI. MIT. |
| Pro | $19.99 | $199.99 | One developer. |
| Pro, founder rate | $12.99 | $124.45 | The first 100 subscribers; same Pro. |
| Team | $44.99 | $449 | A team of up to 4 people: the owner plus 3 teammates. |

Prices are US dollars before tax. There is no lifetime plan.

**Pro includes** every screen, agent-app template, block and kit (with their code), weekly drops,
Studio extras and the Pro CLI and MCP. **Free stays fully usable**: components and primitives never
move to Pro, and every Pro preview stays open to everyone.

**Founder rate.** The first 100 founder subscriptions that become active are counted in D1
(`founder_claims`). From the 100th on, `/api/billing/plans` reports the founder rate as closed, the
pricing page stops showing it, `/api/billing/checkout` refuses founder plans, and the Worker archives
both founder products in Polar (`PATCH /v1/products/{id} {"is_archived": true}`) so a direct link
cannot start a checkout either. Existing founder subscriptions keep their rate while they stay
active. Archiving needs the `POLAR_ACCESS_TOKEN` Worker secret; without it the admin panel shows the
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
`https://ui.oxyn.workers.dev/pricing/success?checkout_id={CHECKOUT_ID}` and the return URL
`https://ui.oxyn.workers.dev/pricing/`. The ids and links live in `PLANS` in
`site/worker/billing.ts`; prices shown on the site are `PRICES` in `site/src/lib/data.ts`. Change
both together, and never create a second product for an existing plan.

Webhook endpoint `3e81f336-ecd1-4446-aaa0-7598e1a055da`: `https://ui.oxyn.workers.dev/api/polar/webhook`,
format raw, API version 2026-10, events `subscription.created`, `subscription.updated`,
`subscription.active`, `subscription.canceled`, `subscription.revoked`, `order.paid`,
`checkout.updated`.

### Worker secrets (Worker `ui`)

| Secret | Needed for | Without it |
| --- | --- | --- |
| `POLAR_WEBHOOK_SECRET` | Verifying webhooks (Standard Webhooks). | The webhook answers 503; no subscription is recorded. |
| `POLAR_ACCESS_TOKEN` (optional) | Pre-signed customer portal sessions; archiving the founder products at 100. | Manage billing opens `https://polar.sh/oxyn/portal`, where Polar emails a sign-in code; the founder archive stays pending (the site still stops selling it). |

Set them from `site/` with `npx wrangler secret put <NAME>`. An organization access token from Polar
(Settings, Developers) with `products:write` and `customer_sessions:write` is enough for
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

## Who has Pro: `plan_for(user)`

One helper in `site/worker/billing.ts` answers "what may this account use", and every check
(`/api/billing/me`, `/api/pro/source/*`, checkout, seats, the portal) goes through it. It reads:

1. the account's own Polar subscriptions (via `licences`): live means status `active`, `trialing` or
   `past_due` (Polar is retrying the card) and not ended;
2. a seat on a live Team subscription;
3. a manual grant (`grants`, D1 migration 0005): unrevoked and with `expires_at` empty or in the
   future, it counts exactly like an active subscription. The admin UI for grants comes in G2; until
   then a grant is a row added with `wrangler d1 execute`.

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

The repository itself is public and MIT licensed, so the gate is about the site, the Pro registry
and the Pro CLI, not about hiding source on GitHub.

## Admin

The admin panel's Billing tab lists subscribers (email, plan, billing period, amount, status,
renewal) and sums monthly recurring revenue: monthly plans at their amount, yearly plans at a
twelfth, live subscriptions only, before tax and Polar's fee.
