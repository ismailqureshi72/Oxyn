**Status:** built v0.1 (2026-11-19, S13: Arc screens II).

# Spec: Month calendar (`month-calendar`)

Family: Arc screens (pages at `/screens/month-calendar/` and `/full`). Reference: the month calendar screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, customer, workspace and article is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `MonthCalendarScreen` (Dioxus) · `MonthCalendarEl` (GPUI), rules in `oxyn_core::parity::arcscr2`.

## 1. Purpose
See a month at a glance, find free time on a day and protect it.

## 2. Parts
The Arc frame (shared with the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and a second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Nav (Calendar, Invites 3, Scheduling on the week screen; Calendar, Upcoming, Invitations 3 on the month screen); a mini month to **jump to a date** (the shown week shaded, today in rose, the picked day filled) on the week screen; **calendars** as checkboxes in their hue (Team meetings blue, Focus time violet, Launch magenta, Holidays green; Personal teal, Travel amber) with counts on the month screen and an **Only** eye button; People / Team calendars with their status. Context card: **Next up** with the time left and a coloured Zoom Join (week), the month's events and busy hours (month) |
| Header | Workspace › Calendar, meta Week 46 or the month's event count, **New event** |
| Second row | Today, ‹ ›, the period as ISO dates (`2026-11-09 – 2026-11-15`, `2026-11`, `Thursday 2026-11-12`), the ISO week, **Filter events**, Day / Week / Month |
| Month grid | Weekday names, 6 × 7 days from Monday (days outside the month shaded), today's number in rose, the picked day outlined. Up to three chips per day (time and title with a hue edge; all-day chips filled); a busier day shows two and **+N more**, which opens a popover with the day's events |
| Day panel | The picked day (`Thursday 2026-11-12`, Week 46), a busy strip for 09:00–18:00 in calendar hues with the busy and free time, the schedule, free gaps of 30 minutes or more, **Block focus time** and **New event on 2026-11-12** |

## 3. Behaviour
Events, lanes, moves, gaps and focus blocks come from `oxyn_core::parity::arcscr2` (`events`, `day_events`, `lanes`, `move_event`, `nudge_event`, `resize_event`, `free_gaps`, `block_focus`). Drag an event to move it (whole days across columns, 15-minute steps up and down), drag its bottom edge to resize; drag on an empty slot to draw a new event (a click makes 30 minutes) and name it in the foot form (Enter saves to the chosen calendar). Hidden calendars leave the grid, the counts stay. Today returns to 2026-11-12; ‹ › page by a day, week or month. Every action reports in a polite live region. Block focus time adds up to two hours of Focus time in the day's first gap of an hour or more.

## 4. Variants
States: Data · Loading · Empty · **More events** (the +N more popover on 2026-11-18) · **Calendars off** (Personal and Travel hidden) · **Focus blocked** (09:30–11:30 Focus time on 2026-11-12). Frame: Inset · Flat.

## 5. Keyboard and accessibility
The calendar body is one tab stop: T today, D / W / M switch views, N starts a new event in the day's first free slot. A focused event moves with ← → (a day) and ↑ ↓ (15 minutes), Shift + ↑ ↓ resizes, Delete removes it, Escape clears. In the month grid (one tab stop with aria-activedescendant) the arrows move the picked day and Enter opens it in the Day view. Events are buttons named with title, ISO date, time and place (aria-roledescription Draggable event); calendars are checkboxes; Only is a labelled icon button; the busy strip is an image named with its busy time.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 960 px the day panel moves under the grid; under 640 px chips become coloured dots so day numbers stay readable.

## 7. Data
`MonthCalendarScreen` takes `layout` and `state`; the grid and panel read `month_cells`, `day_events`, `free_gaps` and `busy_minutes`.

## 8. Known gaps
No backend; Upcoming and Invitations are nav only; team calendars are listed but not overlaid.

## 9. GPUI
`MonthCalendarEl` draws one state. In `ScreenView` it shares the calendar's `Arc2Ui` with the week screen: `cal-day-<0..41>` picks a cell, `cal-focus` blocks focus time, `cal-new` adds an event on the picked day, and Day / Week / Month switch in place. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S13 Arc II).
