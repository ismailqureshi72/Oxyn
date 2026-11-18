**Status:** built v0.1 (2026-11-18, S7: Settings II).

# Spec: Settings · Billing (`settings-billing`)

Family: Screens. Reference: affectUI Settings / Billing (`/screens/settings-subscription`), layout and behaviour only; no source or assets copied.
Components: `BillingSettingsScreen` (Dioxus) · `BillingSettingsEl` (GPUI), models in `oxyn_core::parity::setscr2`.

## 1. Purpose
Your plan, what you have used, how you pay and past invoices.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 816 px |
| Plan | The plan with its price per seat and interval; Upcoming bill with the renewal date and seats, and the amount |
| Usage | Usage meters for seats, credits, storage and agent runs, each with used / limit as text and a bar toned run → warn (75 %) → bad (90 %) |
| Plans | Monthly / Yearly (−20 %) and three plan cards (Free, Pro, Team) with features; Upgrade or Switch asks first and shows the new bill |
| Extra usage | Monthly cap and Email invoices |
| Payment method | The card on file (its network's official coloured mark on a white chip, "Visa ending 4242", expiry) and Update / Add card. The number field shows the Visa, Mastercard and American Express marks; as you type, the network is read from the first digits (Visa 4, Mastercard 51–55 and 2221–2720, Amex 34 and 37) and its mark lights while the others fade. Checks: Luhn and the network's length (Amex 15, others 16), expiry (MM/YY, not past), security code (Amex 4 digits, others 3) |
| Invoices | Number, date, amount, status (Paid, Due, Refunded) and Download |
| Danger zone | Cancel plan: why (radio list), what changes (with a yearly offer when price is the reason), then cancelled at period end with Resume plan |

Chrome is monochrome neutral; colour only means something: official marks (model providers, Google), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Interval, cap and invoice emails go through the save bar; plan changes, the card and cancelling act at once and are announced. Every other settings page opens in place from the sidebar.
prefers-reduced-motion: no fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Cancelling (the cancel flow open at its first step)**. Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every control has a name; a field with a problem is aria-invalid with its message beside it. Segments and filters are radiogroups; destructive actions ask first and focus the safe choice. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-billing/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 760 px plan cards stack and meters go two by two; under 560 px tables drop their extra columns and rows put their control under the label. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, keys, cards, domains and the workspace are fictional; model versions are fictional, marks are the providers' official ones. There is no backend: invites, keys, payments and sign-in changes are simulated.
