**Status:** built v0.1 (2026-11-18, S4: Agent ops II).

# Spec: Agent · Permissions (`agent-permissions`)

Family: Screens. Reference: affectUI Agent / Permissions (`/screens/agent-permissions`), layout and behaviour only; no source or assets copied.
Components: `PermissionsScreen` (Dioxus) · `PermissionsEl` (GPUI), models in `oxyn_core::kit::opsscr`.

## 1. Purpose
What the agent – and every agent it starts – may do without asking first.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Lead | One line on what this page decides |
| Levels | Careful · Balanced · Hands-off as a three-stop radio group with what each means, then Does N things on its own · asks before N · never does N and a three-part bar (blue, amber, grey) |
| Apps | Rows with the official mark on a white tile, name, account, a mini bar and counts |
| Red lines | Actions that always ask or never happen whatever the level |
| Work area (388) | One app's actions by kind (reading, changing, risky), each with a select of three settings; a refund limit while refunds need no approval; recent events |

Chrome is monochrome neutral; colour only means something: official app and model marks, identity hues (purple, blue, green, orange, teal, pink) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
A level recomputes every rule (red lines win); a rule changed by hand overrides the level until the level changes. Counts and bars follow at once.
prefers-reduced-motion: no fills, pulses or transitions.

## 4. Variants
States: Data · Loading · Empty · **Hands-off**. Frame: Inset · Flat. Oxyn extras: `compact` (app rows without the account) and `lock_red` (red lines locked, with a lock glyph).

## 5. Keyboard and accessibility
Levels are a radio group with arrow keys and roving tabindex; each rule is a native select named by its action; changes announce in a polite live region. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-permissions/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and the work area becomes a sheet. Under 560 px the levels stack and per-app counts hide. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional; model versions are fictional, model marks are the providers' official ones.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
