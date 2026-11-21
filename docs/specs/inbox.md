**Status:** built v0.1 (2026-11-18, S3: Agent ops I).

# Spec: Agent · Inbox (`inbox`)

Family: Screens. Reference: affectUI Agent / Inbox (`/screens/inbox`), layout and behaviour only; no source or assets copied.
Components: `InboxScreen` (Dioxus) · `InboxEl` (GPUI), models in `oxyn_core::kit::agentscr`.

## 1. Purpose
Every question agents are waiting on, answered one at a time with a single keystroke.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Inbox with its count, Open / Done tabs (Done counts answers) |
| List (320) | Question rows: a project-tinted glyph (choose, variants, approve), the question, task · age, a blue unread dot; the open one is grey |
| Detail (466) | The task and age, the question (23 px), its context, then the answers in one card: a number key, label, a Suggested pill and, on the picked answer, its + and − lines; "Or tell it something else", Later and a black Send; a key hint line |
| Work area (368) | The task: brief, Status (ring, Waiting on you, n/m), Project, Started, Steps (done blue, current orange with Needs you), Artifacts as file chips |

Chrome is monochrome neutral; colour only means something: official app marks, file-type glyphs, project hues (purple, blue, green, orange) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
1–9 (or Y / N) answers with that choice at once; ↑ / ↓ moves the pick and Enter sends; L or Later moves the question to the end. After an answer the question moves to Done and the next one glides in from the right; a toast says what was sent and Undo puts it back. Typing another answer and Enter sends that text. When nothing is left the detail says you are caught up.
prefers-reduced-motion: no glide, spin, waveform or pulse.

## 4. Variants
States: Data · Loading · Empty · **Just answered** (the next question in, with Undo). Frame: Inset · Flat. Oxyn extras: `peek_next` (the next question previewed under the answers) and `hold` (stay on the answered question).

## 5. Keyboard and accessibility
The question region is focusable, names its shortcuts (aria-keyshortcuts) and takes focus on open; answers are a radiogroup of radios; list rows report aria-current; the toast is a status with an Undo button.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/inbox/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button. Under 760 px the list stacks above the question (max 42 % of the height); under 520 px the question title is 19 px. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
