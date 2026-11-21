**Status:** built v0.1 (2026-11-19, S13: Arc screens II).

# Spec: Inbox triage (`inbox-triage`)

Family: Arc screens (pages at `/screens/inbox-triage/` and `/full`). Reference: the inbox triage screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, customer, workspace and article is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `InboxTriageScreen` (Dioxus) · `InboxTriageEl` (GPUI), rules in `oxyn_core::kit::arcscr2`.

## 1. Purpose
Clear a support inbox fast, mostly from the keyboard.

## 2. Parts
The Arc frame (shared with the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and a second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Thistledown support: **Inbox**, Assigned to me, Unassigned, Snoozed, Resolved with live counts; Team (people with open counts or Back at 16:00); Labels (Billing amber, Bug rose, Question blue, Feature request violet). Context card: an **Available** switch |
| Header | Thistledown › folder, meta "12 open · 5 unread", Newest / Oldest menu |
| Second row | Select all; All / Unread / Due soon, or with a selection: N selected, Resolve E, Snooze H, Assign to me; Undo with what it undoes, else the key hints |
| List (340 px) | Rows with a checkbox, the customer's avatar with its channel mark (official coloured Gmail, Slack, X; a chat glyph for the site widget), name, time (`14:18` today, else an ISO date), subject with an unread dot, preview, label, Urgent, the SLA timer (muted, amber under an hour, rose under 15 minutes or late) and the assignee |
| Thread | Subject, via channel, Urgent, "Reply in 14m" or "Reply 12m late", Assign to me / Yours, Snooze menu (Later today 2026-11-12 17:00, Tomorrow, Next week), **Resolve**; messages with avatars and times; a quick reply (Ctrl + Enter sends) |
| Right card | The customer: name, company, email, local time and city, plan, customer since, label, previous conversations |

## 3. Behaviour
`Triage` in `oxyn_core::kit::arcscr2` keeps the conversations, folder, tab, order, the open one and the selection. Resolving or snoozing acts on the selection (else the open conversation), opens the next one and keeps a snapshot for Undo; Assign to me moves rows into Assigned to me. Opening marks read. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty ("Inbox zero" with Connect Gmail) · **Selected** (three rows with the bulk bar) · **Snooze menu** · **Resolved** (the first conversation resolved, Undo offered). Frame: Inset · Flat.

## 5. Keyboard and accessibility
The inbox body is one tab stop: J / K (or ↓ ↑) walk, E resolves, H snoozes until later today, X selects, A assigns to you, U undoes, Escape clears the selection. The list is a multi-select listbox; checkboxes name the customer and subject; the snooze menu is a menu of menuitems with absolute times; SLA words are text, not colour alone.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 640 px the list sits above the thread.

## 7. Data
`InboxTriageScreen` takes `layout` and `state`; conversations come from `convs(true)`, threads from `thread`.

## 8. Known gaps
No backend; replies from the quick reply are kept in the screen only; labels cannot be edited.

## 9. GPUI
`InboxTriageEl` draws one state. `ScreenView` runs it with `Arc2Ui::tri`: `ib-open-<n>`, `ib-sel-<n>`, `ib-all`, `ib-folder-<n>`, `ib-tab-<n>`, `ib-sort`, `ib-snooze` and `ib-snooze-<n>`, `ib-e|h|a|u`, and the J / K / E / H / X / U / A keys on the view. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S13 Arc II).
