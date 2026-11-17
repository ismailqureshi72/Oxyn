**Status:** built v0.1 (2026-11-18, S4: Agent ops II).

# Spec: Agent · Skills (`agent-skills`)

Family: Screens. Reference: affectUI Agent / Skills (`/screens/agent-skills`), layout and behaviour only; no source or assets copied.
Components: `SkillsScreen` (Dioxus) · `SkillsEl` (GPUI), models in `oxyn_core::parity::opsscr`.

## 1. Purpose
The skills agents can use: which are on, who made them, and new ones to add.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Skills, a search and New skill |
| Categories | Tabs: All, then five subjects (for example Data or Engineering) |
| Installed | Cards with a glyph in the skill's hue, a switch, a two-line description and who made it (Yours, Built-in or a handle) |
| Discover | Cards with + that fills a ring as it installs, and how many use it |
| Work area (368) | The skill with Turn off / Remove (or Install), runs over two weeks in its hue, Steps in when you say, What it does, What it needs, Ran lately |

Chrome is monochrome neutral; colour only means something: official app and model marks, identity hues (purple, blue, green, orange, teal, pink) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Switches turn a skill on or off; + fills its ring in quarters and moves the skill to Installed; Remove moves it back; New skill adds one of yours; search and categories narrow both lists.
prefers-reduced-motion: no fills, pulses or transitions.

## 4. Variants
States: Data · Loading · Empty · **Installing**. Frame: Inset · Flat. Oxyn extras: `compact` (cards without the description) and `by_use` (Discover sorted by how many use each skill).

## 5. Keyboard and accessibility
Switches are role=switch named by the skill; Install reports aria-busy while its ring fills; New skill is a dialog. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-skills/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and the work area becomes a sheet. Under 640 px search hides; under 560 px cards are one column. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional; model versions are fictional, model marks are the providers' official ones.
