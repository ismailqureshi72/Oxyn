**Status:** built v0.1 (2026-11-19, S13: Arc screens II).

# Spec: Help center (`help-center`)

Family: Arc screens (pages at `/screens/help-center/` and `/full`). Reference: the help center screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, customer, workspace and article is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `HelpCenterScreen` (Dioxus) · `HelpCenterEl` (GPUI), rules in `oxyn_core::kit::arcscr2`.

## 1. Purpose
Keep a help site's articles current and learn what readers can't find.

## 2. Parts
The Arc frame (shared with the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and a second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Quillmoor Help: Articles (count), Help site, Search insights 5, Feedback 6; **Collections** in hue (Getting started, Account & billing, Integrations, Troubleshooting, Developers) with counts that filter. Context card: **Quarterly review** percentage with a bar, articles to check and Review (opens In review) |
| Header | Quillmoor Help › Articles, meta article count, **New article** |
| Second row | All / Published / Drafts / In review with counts, **Search articles** (`/`) |
| List (340 px) | Articles with collection dot, title, status (Published green, Draft grey, In review amber), collection and ISO update date; while searching, a result count, snippets with the match marked; with no match: suggestions that do match, Write "query", Clear search |
| Reader | Collection tag, Publish (drafts and reviews), Copy link, title, owner, updated date and views, sections, Related articles, then **Was this helpful?** Yes / No with the helpful share; No asks for a reason (Out of date, Hard to follow, Missing steps, Didn't answer my question) and a comment, then Send feedback |
| Insights (250 px) | Views in 30 days, helpful share with a bar, votes, Top searches with counts; those with no results flagged rose |

## 3. Behaviour
`Help` and `search` in `oxyn_core::kit::arcscr2`: search needs every word in the title or body, ranks title matches first, then views, and returns the matching sentence; tabs and collections narrow the pool. One vote per reader per article; a No needs a reason before Send; Publish turns a draft or review into Published with today's date.

## 4. Variants
States: Data · Loading · Empty ("No articles yet" with Write an article) · **Search** ("cancel") · **No results** ("dark mode") · **Feedback** (a No with Missing steps picked). Frame: Inset · Flat.

## 5. Keyboard and accessibility
`/` focuses the search and Escape clears it; the result count is a status region; matches use `mark`; reasons are a radio group; the thank-you is a status message; tabs are a radio group.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 1180 px the insights column hides; under 640 px the list sits above the reader.

## 7. Data
`HelpCenterScreen` takes `layout` and `state`; articles come from `articles(true)`, top searches from `SEARCHES`.

## 8. Known gaps
No backend or editor: New article and Write only say so; the comment is not stored.

## 9. GPUI
`HelpCenterEl` draws one state. In `ScreenView` the search is a real text field over `Arc2Ui::help`: `hc-open-<n>`, `hc-art-<id>`, `hc-tab-<n>`, `hc-coll-<n>`, `hc-yes|no`, `hc-reason-<n>`, `hc-send`, `hc-publish`, `hc-q-<n>` (a top search fills the field), `hc-clear`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S13 Arc II).
