**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: Overview (`admin-overview`)

Family: Arc screens (pages at `/screens/admin-overview/` and `/full`). Reference: the Arc demo app's overview (uiarc.dev/demo), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `AdminOverviewScreen` (Dioxus) · `AdminOverviewEl` (GPUI), rules in `oxyn_core::kit::arcscr4`.

## 1. Purpose
See at a glance whether the product is healthy and busy, then jump to what needs a look.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Header | Corvel Labs › Overview, meta Updated (ISO date) 09:41 UTC, Range 24h / 7d / 30d, Export |
| KPI tabs | Requests, Error rate, p95 latency, Active members: value and the change against the first half of the range (green when better, rose when worse; signed) |
| Chart | Bars for the open KPI over the range; press a bar for its ISO date (or hour) and value in a status readout |
| Components | Six components with 90-day uptime and today's state |
| Recent activity | The six newest audit events with the actor's avatar and the time |

## 3. Behaviour
`Overview` in `oxyn_core::kit::arcscr4`: `ov_series(kpi, range)` is deterministic; `ov_value`, `ov_delta` (lower is better for errors and latency), `read(i)` says the point back. The incident banner (Webhooks degraded) dismisses.

## 4. Variants
States: Data · Loading · Empty (No traffic yet, Create an API key) · **Last 7 days** · **Incident** (banner up) · **Error rate** (the error chart over 30 days). Frame: Inset · Flat.

## 5. Keyboard and accessibility
KPIs are tabs (aria-selected); the readout is a status region; every bar is a button named with its date and value; changes carry a sign as well as a colour. Every action says itself back in the frame's polite live region. The focus outline appears only when moving by keyboard.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`AdminOverviewScreen` takes `layout` and `state`; sample data comes from `oxyn_core::kit::arcscr4`.

## 8. Known gaps
Export says back a file name; nothing downloads.

## 9. GPUI
`AdminOverviewEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `ov-range-<n>`, `ov-kpi-<n>`, `ov-pt-<n>`, `ov-incident`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
