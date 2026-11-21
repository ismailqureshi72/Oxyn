**Status:** built v0.1 (2026-11-18, S7: Settings II).

# Spec: Settings · Get started (`settings-onboarding`)

Family: Screens. Reference: Oxyn (onboarding checklist in the settings shell; behaviour of common setup checklists), layout and behaviour only; no source or assets copied.
Components: `OnboardingSettingsScreen` (Dioxus) · `OnboardingSettingsEl` (GPUI), models in `oxyn_core::kit::setscr2`.

## 1. Purpose
A few steps to set the workspace up. Each opens the page that does it.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| Progress | A ring with the percentage, the workspace name, "n of 6 done · next: …", a progress bar and Continue (opens the next step's page) |
| Steps | Six rows: number or check, glyph, title and line; Open <page> and Mark done / Undo; the next step is highlighted, done steps are muted and struck |
| Checklist | Show in the sidebar (a policy that goes through the save bar) |

Chrome is monochrome neutral; colour only means something: official marks (model providers, Google), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Steps act at once. Finishing two-step sign-in on Security or changing plan on Billing ticks their step. Every other settings page opens in place from the sidebar.
prefers-reduced-motion: no fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Complete (every step done)**. Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every control has a name; a field with a problem is aria-invalid with its message beside it. Segments and filters are radiogroups; destructive actions ask first and focus the safe choice. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-onboarding/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 760 px plan cards stack and meters go two by two; under 560 px tables drop their extra columns and rows put their control under the label. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every settings screen takes `data: Option<Prefs>` (`oxyn_core::kit::setscr::Prefs`: profile, preferences, notifications, privacy and general fields, and `ws: Ws` for members, models and keys, security, billing and the checklist). `None` shows the sample for the state; to show your own, start from `Prefs::default()` (the sample) and replace what you have. The save bar compares edits against this data.

## 8. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, keys, cards, domains and the workspace are fictional; model versions are fictional, marks are the providers' official ones. There is no backend: invites, keys, payments and sign-in changes are simulated.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
