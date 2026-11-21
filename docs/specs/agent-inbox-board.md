**Status:** built v0.1 (2026-11-18, S3: Agent ops I).

# Spec: Agent · Inbox Board (`agent-inbox-board`)

Family: Screens. Reference: affectUI Agent / Inbox Board (`/screens/agent-inbox-board`), layout and behaviour only; no source or assets copied.
Components: `InboxBoardScreen` (Dioxus) · `InboxBoardEl` (GPUI), models in `oxyn_core::kit::agentscr`.

## 1. Purpose
Every task at a glance by where it stands, so the ones that need you are first.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | A tab per lane plus All tasks, each counted, then Agents (a filter menu), board / list and the black New task |
| Lanes | Three equal lanes on a grey ground, each with a 36 px uppercase head (amber hand, blue spinner, green check) and a count |
| Cards | The agent's pixel mark in its hue, the title, two lines of what it is doing, then a status dot with elapsed time (or Failed), sources read and tool calls |
| Work area (348) | The card: title, blurb, Approve / Reject (Needs you), Retry / Open log (failed) or Open; Status, Agent, Sources, Tools; What it did as a timeline |

Chrome is monochrome neutral; colour only means something: official app marks, file-type glyphs, project hues (purple, blue, green, orange) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Tabs narrow the board to one lane; Agents filters cards; list view turns lanes into rows. Approve moves a card to Running, Reject to Done, Retry clears the failure. New task opens a field and starts a card in Running. Every change is announced.
prefers-reduced-motion: no glide, spin, waveform or pulse.

## 4. Variants
States: Data · Loading · Empty · **Run failed** (a card stopped on a rate limit, Retry ready). Frame: Inset · Flat. Oxyn extras: `compact` (one-line cards) and `wip` (a limit on Running, shown as 3 / 4).

## 5. Keyboard and accessibility
Lanes are labelled sections; cards are buttons with aria-current when open; the view switch is a pair of toggle buttons; New task is a dialog with focus in its field, Enter starts and Escape cancels.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-inbox-board/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button. Under 760 px the lanes stack in one column. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
