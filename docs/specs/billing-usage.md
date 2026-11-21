**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: Billing (`billing-usage`)

Family: Arc screens (pages at `/screens/billing-usage/` and `/full`). Reference: the Arc demo app's billing page, behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `BillingUsageScreen` (Dioxus) · `BillingUsageEl` (GPUI), rules in `oxyn_core::kit::arcscr4`.

## 1. Purpose
Know what you pay, what you use and what a plan change costs today.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Header | Billing, meta the period as ISO dates, Monthly / Yearly · save 17% |
| Past due | A rose banner with the failed date, the deadline and Retry payment |
| Plans | Starter, Team, Scale: price a month (yearly shows the yearly total), seats, API calls, audit log length, Current or Upgrade / Downgrade |
| Change | The prorated charge (or credit) for the days left and the new price from the renewal date; Cancel, Confirm change |
| Usage | Seats, API calls, storage meters against the plan |
| Payment | Visa ending 4242 with the official Visa mark, Update; the billing email |
| Invoices | Number, ISO date, amount, Paid / Open / Failed, download |

## 3. Behaviour
`Billing` in `oxyn_core::kit::arcscr4`: `price(p)`, `prorate(p)` = (new − old) × days left / 30, `pick(p)` refuses a plan whose seats are fewer than used, `confirm()`, `toggle_cycle()`, `retry()`.

## 4. Variants
States: Data · Loading · Empty (Starter, nothing billed) · **Yearly** · **Change plan** (Scale picked) · **Past due**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
Plans are a radio group; the change is a dialog with its sums written out; meters carry values. Every action says itself back in the frame's polite live region. The focus outline appears only when moving by keyboard.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`BillingUsageScreen` takes `layout` and `state`; sample data comes from `oxyn_core::kit::arcscr4`.

## 8. Known gaps
No payment is taken; the card is fictional.

## 9. GPUI
`BillingUsageEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `bl-cycle`, `bl-pick-<n>`, `bl-confirm`, `bl-cancel`, `bl-retry`, `bl-inv-<n>`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
