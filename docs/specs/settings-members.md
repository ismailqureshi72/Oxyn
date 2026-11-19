**Status:** built v0.1 (2026-11-18, S7: Settings II).

# Spec: Settings · Members (`settings-members`)

Family: Screens. Reference: affectUI Settings / Teams (`/screens/settings-members`), layout and behaviour only; no source or assets copied.
Components: `MembersSettingsScreen` (Dioxus) · `MembersSettingsEl` (GPUI), models in `oxyn_core::parity::setscr2`.

## 1. Purpose
Invite people, set what they can do and group them into teams.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 816 px (wider for the table): heading, lead, Members / Teams tabs |
| At a glance | Three tiles: members (joined · invited), seats with a meter toned by how full it is, admins |
| Toolbar | Search (name or address), filter by role, Export, and Invite members |
| Invite | Several addresses at once (commas, spaces or lines), a role with what it allows, a seat count; it says why it cannot send (bad address, already here, not enough seats) |
| Table | Avatar (initials on an identity hue, or a mail glyph while invited), name and email, a role menu, team, a usage meter, and a menu: Resend / Revoke invite or Remove (asks first) |
| Teams | A card per team with its hue, count and people; Add puts someone in it |

Chrome is monochrome neutral; colour only means something: official marks (model providers, Google), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Invites, role changes, removals and team moves act at once and are announced. The last owner cannot be demoted or removed, and you cannot remove yourself. Every other settings page opens in place from the sidebar.
prefers-reduced-motion: no fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Inviting (the invite form open with two addresses typed)**. Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every control has a name; a field with a problem is aria-invalid with its message beside it. Segments and filters are radiogroups; destructive actions ask first and focus the safe choice. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-members/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 760 px plan cards stack and meters go two by two; under 560 px tables drop their extra columns and rows put their control under the label. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every settings screen takes `data: Option<Prefs>` (`oxyn_core::parity::setscr::Prefs`: profile, preferences, notifications, privacy and general fields, and `ws: Ws` for members, models and keys, security, billing and the checklist). `None` shows the sample for the state; to show your own, start from `Prefs::default()` (the sample) and replace what you have. The save bar compares edits against this data.

## 8. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, keys, cards, domains and the workspace are fictional; model versions are fictional, marks are the providers' official ones. There is no backend: invites, keys, payments and sign-in changes are simulated.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
