**Status:** built v0.1 (2026-11-18, S7: Settings II).

# Spec: Settings · Models (`settings-models`)

Family: Screens. Reference: affectUI Settings / Models (`/screens/settings-models`), layout and behaviour only; no source or assets copied.
Components: `ModelsSettingsScreen` (Dioxus) · `ModelsSettingsEl` (GPUI), models in `oxyn_core::parity::setscr2`.

## 1. Purpose
Which models people here can use, the defaults, and your own keys.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| Defaults | Chat model (Auto or any chat model that is on, with its official mark) and Image model (Off or an image model) |
| Toolbar | All / Chat / Image segment and a model search |
| Providers | A card per provider (Anthropic, OpenAI, Google, Mistral, DeepSeek) with its official coloured mark, where it is hosted, n/m on, and a chevron to fold it; each model: mark, name, line, an access menu (Everyone, Admins, Engineering) and a switch |
| API keys | Provider mark, name, the key masked (first 8 and last 4), added and last used; Reveal / Hide, Copy, Rotate (asks first, then shows the new key once with Copy and hide) and Delete; Add key checks the name and the key; the empty state reads "No API keys – Usage runs on your plan until you add a provider's key." |

Chrome is monochrome neutral; colour only means something: official marks (model providers, Google), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Switches, access and defaults go through the save bar (or save at once with `autosave`). Turning off a model that is a default moves chat to Auto or pictures to Off and says so. Keys act at once. Every other settings page opens in place from the sidebar.
prefers-reduced-motion: no fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **New key (a just-rotated key shown once)**. Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every control has a name; a field with a problem is aria-invalid with its message beside it. Segments and filters are radiogroups; destructive actions ask first and focus the safe choice. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-models/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 760 px plan cards stack and meters go two by two; under 560 px tables drop their extra columns and rows put their control under the label. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, keys, cards, domains and the workspace are fictional; model versions are fictional, marks are the providers' official ones. There is no backend: invites, keys, payments and sign-in changes are simulated.
