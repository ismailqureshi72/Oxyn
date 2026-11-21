**Status:** built v0.1 (2026-11-19, S14: Arc screens III).

# Spec: Invoice desk (`invoice-studio`)

Family: Arc screens (pages at `/screens/invoice-desk/` and `/full`). Reference: the invoice studio screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, ticker, product, client and person is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `InvoiceStudioScreen` (Dioxus) · `InvoiceStudioEl` (GPUI), rules in `oxyn_core::kit::arcscr3`.

## 1. Purpose
Write an invoice line by line with live totals and send it when it is complete.

## 2. Parts
The Arc frame (shared with the S12 and S13 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Northwyn Studio: All invoices, Draft / Open / Overdue / Paid with counts and hue dots; **Clients** with what each owes. Context card: Collected in November |
| Header | Invoices › NS-1046, meta outstanding total, **Preview** (Edit), **Send invoice**, New invoice |
| List (264 px) | Invoices grouped Overdue, Open, Drafts, Paid: client, number, ISO due date, total |
| Paper | Studio name and address, status pill and number, Bill to with the billing email, Issued and Due dates with the terms; line items (description edited in place, a quantity stepper, rate, amount, Remove), Add line, Subtotal, Sales tax, **Total due**, the send check (amber, or Ready to send in green) |
| Details card (312 px) | Client select, Payment terms (On receipt, Net 15, Net 30, Net 45) with the due date, Sales tax 8.25% and Automatic reminders switches, Activity |

## 3. Behaviour
`Invoice` in `oxyn_core::kit::arcscr3`: only drafts edit; removing a line offers Undo that puts it back in place; tax rounds half up; the due date is the issue date plus the terms. Send checks in order: the client has a billing email, there is a line, every line is described, the total is above zero; a sent invoice turns Open and reports its email, total and due date.

## 4. Variants
States: Data · Loading · Empty ("No invoices yet" with New invoice) · **Editing** (a new line "Rush turnaround") · **Preview** (read-only, as the client sees it) · **Sent** (Open, due 2026-12-12). Frame: Inset · Flat.

## 5. Keyboard and accessibility
Lines are a table with labelled inputs and steppers; totals are a description list; the send check is a status message tied to Send invoice by aria-describedby; switches use role switch.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 960 px the list narrows to 220 px; under 640 px the list sits above the paper and the rate column hides.

## 7. Data
`InvoiceStudioScreen` takes `layout` and `state`; invoices come from `invoices(true)`.

## 8. Known gaps
No PDF or email is produced; Send changes the demo state only.

## 9. GPUI
`InvoiceStudioEl` draws one state. In `ScreenView`: `iv-open-<no>`, `iv-new`, `iv-add`, `iv-qup-<id>`, `iv-qdown-<id>`, `iv-rm-<id>`, `iv-undo`, `iv-terms-<n>`, `iv-tax`, `iv-client-<n>`, `iv-preview`, `iv-send`, `iv-remind`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S14 Arc III).
