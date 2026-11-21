**Status:** built v0.2 (2026-11-18, S2: Workspace parity; v0.1 screens 1).

# Spec: CRM (`crm`)

Family: Screens. Reference: affectUI CRM (`/screens/crm`), layout and behaviour only; no source or assets copied.
Components: `CrmScreen` (Dioxus) · `CrmEl` (GPUI), models in `oxyn_core::kit::workscr`.

## 1. Purpose
A sales table you edit in place: companies with their category, owner, value and stage.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Sales persona: Home · Companies 284 · Contacts · Deals · Reports, then the Pipelines group |
| Top bar | 48 px: a folder glyph, Sales / Head of Sales with a ⌄ (switch the view) and a star |
| View tabs | New view and All views icon buttons, a hairline, then the open views as tabs with their marks (Company CRM, Deals, Contacts, Renewals); on the right the Stage and Owner filters and search |
| Table | In a 12 px radius card: a checkbox column, then Company (official mark + name), Category (toned tag), Domain (dotted underline), Owner, Value, Stage (toned tag: New grey, Qualified blue, Proposal purple, Negotiation amber, Won green, Lost red), Last contact; 40 px rows |
| Foot | + Add row on the left, "Double-click a cell, or press Enter on it, to edit" on the right |

Chrome is monochrome neutral; colour only means something: official brand marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Double-click or Enter on a cell edits it (text field, or a select for Stage and Owner); Enter or Tab saves (Tab moves to the next cell), Escape cancels, and a value that does not parse stays open marked invalid. Company, Owner, Value and Stage headers sort (again to reverse). Checkboxes select rows; a bar offers Delete and Clear. Add row adds Untitled and opens its name. View tabs switch the set (Deals: proposals, Renewals: won).
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States: Data · Loading (row skeletons) · Empty (no companies, pointing at Integrations) · **Editing** (a Value cell open). Frame: Inset · Flat. Oxyn extras: stage and owner filters, search, sorting, row select with Delete, Tab to the next cell, a pipeline strip option (rows and value per stage, press one to filter).

## 5. Keyboard and accessibility
The table has a caption label, sortable headers report aria-sort, cells are focusable with a visible ring, edit fields are labelled and use aria-invalid; tabs are a tablist with ←/→. Escape closes any open menu or popover and clears a search; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/crm/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and top-bar actions keep their icons with hidden labels. Under 760 px the tab row wraps (views on their own scrolling line), Owner and Last contact hide and the table scrolls sideways; the edit hint hides. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; drag, menus, typing and charts' crosshair belong to the host.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
