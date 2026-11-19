**Status:** built v0.1 (2026-11-19, S13: Arc screens II).

# Spec: Support conversation (`support-conversation`)

Family: Arc screens (pages at `/screens/support-conversation/` and `/full`). Reference: the support conversation screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, customer, workspace and article is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `SupportConversationScreen` (Dioxus) · `SupportConversationEl` (GPUI), rules in `oxyn_core::parity::arcscr2`.

## 1. Purpose
Answer one customer well, with the team's context beside the thread.

## 2. Parts
The Arc frame (shared with the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and a second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Pinecrest: Your inbox, Mentions, Unassigned, All open, Snoozed, Closed with counts; **Waiting on you** (customers with "12m late" or "55m left"); Teams. Context card: Available switch |
| Header | Pinecrest › view › #4812, the customer; Assign to me (when not yours), Snooze menu, **Close** / Reopen |
| Ticket list (300 px) | Customer, ticket number, subject with unread dot, last message, status (Open blue, Waiting on customer amber, Snoozed violet, Closed green) and the SLA |
| Thread | Subject, channel, status, Urgent, SLA, assignee; customer messages with a screenshot file, replies on grey with a help article card, **internal notes** in amber with a lock and Internal note, system lines centred |
| Composer | Reply / Note (notes tint the box amber), the text, attached files and article as removable chips, Saved replies (Fix date, Need more detail, Refund on its way, Closing the loop; filled with the customer's first name), Article, Attach, Ctrl ↵, Send / Add note |
| Right card | Profile / Activity tabs; email, local time, plan, customer since; Apps with official coloured marks (Google Calendar: the next call, Stripe: plan and last invoice); the linked issue with the Linear mark; previous conversations |

## 3. Behaviour
`Desk` in `oxyn_core::parity::arcscr2` keeps tickets, the open one, the view, the composer (mode, text, files, article). Send needs text, a file or an article; a reply sets Waiting on customer and stops the SLA, a note stays internal. Close adds a system line (Reopen undoes it); Snooze moves the ticket to Snoozed until an absolute time. The list puts late SLAs first.

## 4. Variants
States: Data · Loading · Empty ("No tickets yet" with Copy support address) · **Internal note** (Note mode with a draft) · **Reply sent** (a saved reply sent) · **Closed**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
Ctrl + Enter sends from the composer; Reply / Note is a radio group; the thread is an ordered list named Conversation; menus close on Escape; the customer card's tabs are a tablist.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 640 px the ticket list sits above the thread and the composer's tool labels become icons.

## 7. Data
`SupportConversationScreen` takes `layout` and `state`; tickets come from `tickets(true)`, saved replies from `SAVED`.

## 8. Known gaps
No backend or real uploads: Attach adds demo file names; @ mentions are text.

## 9. GPUI
`SupportConversationEl` draws one state. In `ScreenView` the composer is a real text field (Enter sends) over `Arc2Ui::desk`: `sp-open-<n>`, `sp-view-<n>`, `sp-reply` / `sp-note`, `sp-saved-<n>` (fills the field), `sp-article`, `sp-attach`, `sp-send`, `sp-close`, `sp-snooze-<n>`, `sp-take`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S13 Arc II).
