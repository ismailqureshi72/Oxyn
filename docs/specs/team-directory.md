**Status:** built v0.1 (2026-11-19, S12: Arc screens I).

# Spec: Team directory (`team-directory`)

Family: Arc screens (pages at `/screens/team-directory/` and `/full`). Reference: the team directory screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, workspace and project is fictional; dates are absolute ISO dates.
Components: `TeamDirectoryScreen` (Dioxus) · `TeamDirectoryEl` (GPUI), rules in `oxyn_core::kit::arcscr`.

## 1. Purpose
Find a colleague, see where and when they are, and invite someone new.

## 2. Parts
The Arc frame (shared by the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Saltwick: **Find a person** (focuses the search), People 16, Org chart, Time off 2; **Offices** (Zürich, London, New York, Lisbon, Singapore: a sun when inside working hours, "n of m working now", the local time; a press filters by office); **Favorites** |
| Header | Saltwick › People, "16 people in 5 offices"; **Invite** |
| Second row | Search people, roles, skills (`/` focuses, Escape clears), **Team** and **Status** menus, an office chip, Grid / List |
| Groups | Engineering, Design, Go-to-market, Operations with a toned glyph and count |
| Card | Avatar, name (Lead tag), role, a star to favourite; office and local time; availability with a toned glyph: Available (green), Busy until 15:00 (amber), Out until 2026-11-16 (blue), Starts 2026-11-17 (violet), Online at 09:00 (grey) |
| List | One row per person: name and role, team, office time and availability |
| Invite | Email field with validation (an alert under a wrong address), Send invite, Cancel; sent invites show as Pending |

## 3. Behaviour
Local times come from one clock (2026-11-12 13:32 UTC) and each office's offset; the office card counts people available or busy while the office is open. Search matches every word against name, role, team and skills, then the team, status and office filters apply.

## 4. Variants
States: Data · Loading · Empty ("No one here yet" with Invite) · **List** · **No results** (with Clear search) · **Invite**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
Teams are labelled sections; favourites are toggle buttons named for the person; the invite field is aria-invalid with an alert; every action reports in a polite live region. The sidebar is a nav landmark; the open row has aria-current=page; the account menu and every menu close on Escape.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it). Under 640 px header actions become icon buttons, the meta line hides and grids drop to one column. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`TeamDirectoryScreen` takes `data: Option<Vec<Member>>`: person, role, team, office, availability, lead, skills. `None` shows the sample.

## 8. Known gaps
No backend: invites are not sent anywhere; Org chart and Time off only report. In GPUI Invite is said back (the Invite state shows the bar with its address) and the team and status filters are chips.

## 9. GPUI
`TeamDirectoryEl` draws one state (`.state()`, `.extra()`). `ScreenView` (oxyn-gpui, `kit::screenview`) runs it live as part of the **Arc screens I** set, driven by `oxyn_core::kit::screenui::ScreenUi` and its `ArcUi`: `ArcUi` keeps the search text, team / status / office filters, view and favourites: `td-team-<n>`, `td-status-<n>`, `td-office-<n>`, `td-view`, `td-fav-<n>`, `td-clear`; the search is a real `ImeInput`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S12 Arc I).
