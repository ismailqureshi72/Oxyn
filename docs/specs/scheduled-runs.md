**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-2 capture notes).

# Spec: Scheduled runs (`scheduled-runs`)

Family: Agents. Closes the parity row: affectUI · Scheduled Runs (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-2 capture notes (approximate); the tasks, days and copy are our
own. Related Oxyn components: background-jobs.
Components: `ScheduledRuns` (Dioxus) · `ScheduledRunsEl` (GPUI), model in `oxyn_core::kit::schedfid`.

## 1. Purpose
Plan a week of agent work against session usage limits: see what is booked where, what still waits, and which
session is full, with automatic booking that packs small tasks into the gaps and cuts big ones across sessions.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | no outer card; sits on the #FAFAFA / #141414 preview surface, 600 px wide |
| Header | title 13.5/650, muted "Sessions of 5 h"; right: one round toggle per agent (coloured brand mark), a white pill with the picked agent (or "All agents"), the task count, and an Auto / Manual segmented control (active segment white and bold on a grey track) |
| List column | 136 px, #F6F6F7 / #1A1A1A, 10 px radius: "To book" + count badge, the waiting cards, or "Nothing left to book" |
| Week | 3 day columns (uppercase day, bold date) × 3 sessions (08:00 / 13:00 / 18:00 labels at the left), hairline separators |
| Card | white / #202020, 1 px border, 8 px radius: brand mark + title 11/600 (ellipsis), a "k/n" violet chip on split parts; agent · description 10 px muted; metric glyphs with numbers and the share of a session (bold, violet over 100 %) |
| Session bar | 3 px bar of the booked share (violet; red past 100 %, with a red top rule) and the percent |
| Details | 230 px popover under the card: title, description, Agent / Session / metrics / Needs, then session buttons that still fit, or "Take back to the list" |
| Status | muted line: all booked / n waiting / n could not be fitted / which session is over its limit (red) |

## 3. Behaviour
- Auto books everything not booked by hand: biggest first, each into the fullest session that still has room (so
  small tasks fill gaps); a task over 100 % is cut into ⌈need/100⌉ equal parts in distinct sessions, numbered in time
  order. A task that cannot fit stays in the list.
- Manual keeps only hand bookings (each task's own pin plus drops); the rest returns to the list.
- Drag a card onto a session (or open it and press a session) to book it by hand; a hand booking may overfill a
  session, which turns red. Drag back onto the list, or "Take back to the list", to unbook (switches to Manual so it
  stays out). Agent toggles dim other agents' cards to 35 % and update pill and count.
- Motion: cards fly in one after another (60 ms stagger) whenever the mode changes; bars grow; the popover scales in.
  prefers-reduced-motion turns all of it off.

## 4. Variants
auto · manual · filtered · open · booked-open · overbooked · narrow · classic.
Controls: style, state (6), width.
Extras beyond the reference: booking from the details (no drag needed), "Take back to the list", the over-limit
status line naming the session.

## 5. Keyboard and accessibility
Tab through agent toggles, the radio group and the cards; Enter / Space opens a card; inside, the session buttons book
it; Escape closes. Cards and sessions carry full names (title, agent, share; session and booked percent). The status
line is a polite status region; over-limit is in text as well as red.

## 6. Responsive
Under 560 px the list sits above the week (cards wrap in rows) and cards inside sessions keep only title and share.

## 7. Known gaps
- Parts of a split task cannot be dragged one at a time; a drag moves the whole task into one session.
- GPUI: a static snapshot – no drag, fly-in or details.
