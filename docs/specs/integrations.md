**Status:** built v0.2 (2026-11-18, S2: Workspace parity; v0.1 screens 1).

# Spec: Integrations (`integrations`)

Family: Screens. Reference: affectUI Integrations (`/screens/integrations`), layout and behaviour only; no source or assets copied.
Components: `IntegrationsScreen` (Dioxus) · `IntegrationsEl` (GPUI), models in `oxyn_core::parity::workscr`.

## 1. Purpose
A directory of the apps a workspace can link, with what each one does and whether it is linked.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Folders persona with Integrations current |
| Column | 958 px centred, 40 px from the top: Integrations (24 px) and one lead line |
| Sticky header | All · Connected n · Available tabs, Search integrations and the black Request; it stays on top while the page scrolls |
| Categories | Popular, Google, Communication, Engineering, Sales and payments: a 15 px heading, then two columns of apps |
| App row | A 30 px white tile with the official mark in its own colours, the name with a verified glyph, one line that ellipsises, and on the right + (link) or a filled ✓ (linked) |
| More row | Popular shows six; the rest fold into a row of overlapping marks and "Stripe, Zapier and 2 more ›" |

Chrome is monochrome neutral; colour only means something: official brand marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
+ opens a small confirm (Link {app}? with what agents may do; Link is focused) and links it; ✓ opens Settings / Unlink; the tabs and search filter every category live and empty categories drop out; Request opens a field (Enter sends) and thanks you; the more row unfolds Popular.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States: Data · Loading (heading and row skeletons) · Empty (nothing allowed yet, with Request) · **Link failed** (one app shows the error line and Try again). Frame: Inset · Flat. Oxyn extras: the confirm before linking, the Unlink menu, the Request dialog, a category menu option that narrows the page to one group.

## 5. Keyboard and accessibility
Each action button names the app and its state (Link Slack; Slack is linked. Options); the confirm and Request are labelled dialogs that Escape closes; errors are role=alert; the Connected tab carries its count. Escape closes any open menu or popover and clears a search; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/integrations/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and top-bar actions keep their icons with hidden labels. Two columns become one under 900 px; under 620 px the sticky header wraps with the tabs on their own line and popovers open to the left. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; drag, menus, typing and charts' crosshair belong to the host.
