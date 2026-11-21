**Status:** built v0.1 (2026-11-19, S13: Arc screens II).

# Spec: Week calendar (`week-calendar`)

Family: Arc screens (pages at `/screens/week-calendar/` and `/full`). Reference: the week calendar screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, customer, workspace and article is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `WeekCalendarScreen` (Dioxus) · `WeekCalendarEl` (GPUI), rules in `oxyn_core::kit::arcscr2`.

## 1. Purpose
Plan the week on a time grid and move meetings by hand or keyboard.

## 2. Parts
The Arc frame (shared with the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and a second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Nav (Calendar, Invites 3, Scheduling on the week screen; Calendar, Upcoming, Invitations 3 on the month screen); a mini month to **jump to a date** (the shown week shaded, today in rose, the picked day filled) on the week screen; **calendars** as checkboxes in their hue (Team meetings blue, Focus time violet, Launch magenta, Holidays green; Personal teal, Travel amber) with counts on the month screen and an **Only** eye button; People / Team calendars with their status. Context card: **Next up** with the time left and a coloured Zoom Join (week), the month's events and busy hours (month) |
| Header | Workspace › Calendar, meta Week 46 or the month's event count, **New event** |
| Second row | Today, ‹ ›, the period as ISO dates (`2026-11-09 – 2026-11-15`, `2026-11`, `Thursday 2026-11-12`), the ISO week, **Filter events**, Day / Week / Month |
| Time grid | 07:00–23:00 at 48 px an hour, a 56 px gutter, day headers (weekday and day, today in a rose disc), an all-day row, weekends shaded, a rose now line on today. Events: a 3 px bar in the calendar's hue over a 15 % tint, title, time, place or a coloured Zoom mark when there is room; overlapping events share the column side by side |
| Foot | The picked event (dot, title, ISO date and time, place, guests, Join, Delete) or the new-event form (title, ISO time, calendar menu, Cancel, Save), else a hint |

## 3. Behaviour
Events, lanes, moves, gaps and focus blocks come from `oxyn_core::kit::arcscr2` (`events`, `day_events`, `lanes`, `move_event`, `nudge_event`, `resize_event`, `free_gaps`, `block_focus`). Drag an event to move it (whole days across columns, 15-minute steps up and down), drag its bottom edge to resize; drag on an empty slot to draw a new event (a click makes 30 minutes) and name it in the foot form (Enter saves to the chosen calendar). Hidden calendars leave the grid, the counts stay. Today returns to 2026-11-12; ‹ › page by a day, week or month. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading (skeleton) · Empty ("No events yet" over the grid with New event) · **Dragging** (Launch dry run lifted one day and 30 minutes later, its new date in the event) · **New event** (a drawn block with the title field and calendar menu) · **Day view**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
The calendar body is one tab stop: T today, D / W / M switch views, N starts a new event in the day's first free slot. A focused event moves with ← → (a day) and ↑ ↓ (15 minutes), Shift + ↑ ↓ resizes, Delete removes it, Escape clears. In the month grid (one tab stop with aria-activedescendant) the arrows move the picked day and Enter opens it in the Day view. Events are buttons named with title, ISO date, time and place (aria-roledescription Draggable event); calendars are checkboxes; Only is a labelled icon button; the busy strip is an image named with its busy time.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 960 px the week keeps a 720 px grid inside the page, which scrolls sideways with a visible scrollbar; the Day view always fits.

## 7. Data
`WeekCalendarScreen` takes `layout` and `state`; events come from `events(true)` (2026-10-26 … 2026-12-06), which a host replaces with its own.

## 8. Known gaps
No backend or recurrence: repeating meetings are separate sample events; no time zones beyond the shown local time; guests cannot be edited.

## 9. GPUI
`WeekCalendarEl` draws one state (`.state()`, `.extra()`). `ScreenView` runs it live in the **Arc screens II** set with the core's `Arc2Ui`: `cal-view-<n>`, `cal-prev|next|today`, `cal-day-<n>` (a day header: opens that day, or moves the picked event there), `cal-pick-<id>`, `cal-earlier|later|up|down|longer|shorter` (also Alt + arrows, Alt + Shift + ↑ ↓), `cal-cal-<n>`, `cal-new`; events drag onto a day column (`cal-grab-<id>`, then `cal-day-<n>`). The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S13 Arc II).
