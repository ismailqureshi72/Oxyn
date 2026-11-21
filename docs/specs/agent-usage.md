**Status:** built v0.1 (2026-11-18, S4: Agent ops II).

# Spec: Agent · Usage (`agent-usage`)

Family: Screens. Reference: affectUI Agent / Usage (`/screens/agent-usage`), layout and behaviour only; no source or assets copied.
Components: `UsageScreen` (Dioxus) · `UsageEl` (GPUI), models in `oxyn_core::kit::opsscr`.

## 1. Purpose
What agents cost this month, whether that fits the budget, and where the money went.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Usage · November |
| Spend card | Spent in November, the total of the budget and its share, a plain-language forecast, a running-total line with a dashed budget line and a dashed forecast to month end, then Tasks · Runs · Per task · Tokens |
| Where it went | Projects / Models; rows with a hue square (or the model's official mark), a share bar, amount, share and change on last month |
| Worth a look | Rows with a tinted glyph, what happened, a tag and Open |
| Work area (368) | The project: amount and share, change on last month, day by day bars and What cost the most |

Chrome is monochrome neutral; colour only means something: official app and model marks, identity hues (purple, blue, green, orange, teal, pink) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Projects / Models switches the breakdown; a project row opens it; Open marks an item as seen. The forecast is spend ÷ days so far × days in the month; over the budget it names the day it runs out.
prefers-reduced-motion: no fills, pulses or transitions.

## 4. Variants
States: Data · Loading · Empty · **Over budget**. Frame: Inset · Flat. Oxyn extras: `daily` (day-by-day bars instead of the running total) and `alert` (an alert line at 80 % of the budget).

## 5. Keyboard and accessibility
The chart is role=img with spend, budget and forecast in its label; Projects / Models is a pressed-button group. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-usage/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and the work area becomes a sheet. Under 560 px the totals go two by two and share bars hide. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional; model versions are fictional, model marks are the providers' official ones.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
