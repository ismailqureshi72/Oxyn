**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: Audit log (`audit-log`)

Family: Arc screens (pages at `/screens/audit-log/` and `/full`). Reference: the Arc demo app's audit log, behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `AuditLogScreen` (Dioxus) · `AuditLogEl` (GPUI), rules in `oxyn_core::kit::arcscr4`.

## 1. Purpose
Find who did what, when and from where, and prove it with the before and after.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Bar | Type chips All events · Sign-in · API keys · Members · Billing, Failures only, a search over actor, action, target and IP (Escape clears) |
| Table | Time (UTC, `YYYY-MM-DD HH:MM:SS`), Actor (avatar or System), Action (a button), Target, IP address, Result (OK / Blocked) |
| Right card | The open event: result, When, Actor, Target, Type, IP address, and the change as a red − line and a green + line |
| Foot | Show more (+6 at a time) |

## 3. Behaviour
`Audit` in `oxyn_core::kit::arcscr4`: 16 sample events newest first; filters combine; `toggle(id)` opens and closes; `more()` pages by 6; `export()` names `audit-<date>.csv`.

## 4. Variants
States: Data · Loading · Empty (Nothing logged yet) · **Filtered** (API keys) · **Event open** (a role change) · **No matches** (Clear filters). Frame: Inset · Flat.

## 5. Keyboard and accessibility
Each action is a button with aria-expanded; the details are a labelled region; the count is a status message; results are words, not only colour. Every action says itself back in the frame's polite live region. The focus outline appears only when moving by keyboard.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`AuditLogScreen` takes `layout` and `state`; sample data comes from `oxyn_core::kit::arcscr4`.

## 8. Known gaps
Export produces no file.

## 9. GPUI
`AuditLogEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `au-kind-<n>`, `au-open-<id>`, `au-failed`, `au-clear`, `au-more`, `au-export`, `au-close`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
