**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: Status (`status-board`)

Family: Arc screens (pages at `/screens/status-board/` and `/full`). Reference: the Arc demo app's status page, behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `StatusBoardScreen` (Dioxus) · `StatusBoardEl` (GPUI), rules in `oxyn_core::parity::arcscr4`.

## 1. Purpose
Tell customers and the team what works now, what broke recently and how long things have been reliable.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Header | Status, meta the status domain and the check time, Subscribe (Subscribed when on) |
| Overall | All systems operational (green) / Some systems are degraded (amber) / Major outage (rose) / Planned maintenance (blue), with the average 90-day uptime |
| Open incident | Title, component and timed updates (Investigating → Identified → Monitoring) with ISO date and UTC time |
| Components | API, Dashboard, Webhooks, Sign-in, Background jobs, File storage: what each covers, uptime %, today's state and 90 day bars (press a day for its date, state and incident) |
| Past incidents | Five incidents with state, ISO date; each opens its duration and follow-up |

## 3. Behaviour
`Status` in `oxyn_core::parity::arcscr4`: `day_health(c, d, mode)`, `uptime(c, mode)` (an outage day costs 0.4%, a degraded day 0.06%), `open_incident(mode)`, `read(c, d)`, `subscribe()`.

## 4. Variants
States: Data · Loading · Empty (No components yet) · **Degraded** · **Outage** · **Maintenance**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
The overall state is a status region; day bars are buttons named with date and state, one Tab stop per component (today); states are words as well as colour. Every action says itself back in the frame's polite live region. The focus outline appears only when moving by keyboard.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`StatusBoardScreen` takes `layout` and `state`; sample data comes from `oxyn_core::parity::arcscr4`.

## 8. Known gaps
Subscribe changes demo state only; no email is sent.

## 9. GPUI
`StatusBoardEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `st-day-<c>-<d>`, `st-past-<n>`, `st-sub`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
