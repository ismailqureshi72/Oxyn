**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: API keys and webhooks (`api-keys-webhooks`)

Family: Arc screens (pages at `/screens/api-keys-webhooks/` and `/full`). Reference: the Arc demo app's developer settings, behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `ApiKeysWebhooksScreen` (Dioxus) · `ApiKeysWebhooksEl` (GPUI), rules in `oxyn_core::parity::arcscr4`.

## 1. Purpose
Give servers a key with the least scope they need, and see whether the events we send arrive.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Header | API keys (n) / Webhooks (n) segmented, New key or Add endpoint |
| New key form | Name (checked: present, under 32 characters, unique) and Scope (Read only, Read and write, Admin), Create key |
| Secret | Shown once after Create or Roll, with Copy (Copied) and Done; an alert says it will not show again |
| Keys table | Name with who made it, `ox_live_…` and the last four, scope (Admin in amber), created and last used ISO dates, Roll, Revoke (asks again as Revoke?) |
| Webhooks | Endpoints with events and delivery rate (On / Off / Failing); the open endpoint's events, Send test, an on/off switch, and deliveries with status code, event, duration and time, Retry on failures |

## 3. Behaviour
`Keys` in `oxyn_core::parity::arcscr4`: `create()`, `roll(id)`, `revoke(id)` (two presses), `test()`, `toggle()`, `retry(id)`; an endpoint is failing when its last two deliveries failed.

## 4. Variants
States: Data · Loading · Empty (No API keys) · **Key created** · **Webhooks** · **Failing endpoint**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
The secret is an alert with Copy; Revoke and Remove ask again before acting; the endpoint switch is role switch; codes are words as well as colour. Every action says itself back in the frame's polite live region. The focus outline appears only when moving by keyboard.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`ApiKeysWebhooksScreen` takes `layout` and `state`; sample data comes from `oxyn_core::parity::arcscr4`.

## 8. Known gaps
Secrets are fictional; Copy does not touch the clipboard.

## 9. GPUI
`ApiKeysWebhooksEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `ak-tab-<n>`, `ak-new`, `ak-name`, `ak-scope-<n>`, `ak-create`, `ak-cancel`, `ak-copy`, `ak-done`, `ak-roll-<id>`, `ak-revoke-<id>`, `wh-open-<n>`, `wh-test`, `wh-toggle`, `wh-retry-<id>`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
