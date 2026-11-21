**Status:** built v0.2 (2026-11-18, S2: Workspace parity; v0.1 screens 1).

# Spec: Folders · Overview (`folders-overview`)

Family: Screens. Reference: affectUI Folders / Overview (`/screens/files`), layout and behaviour only; no source or assets copied.
Components: `FoldersScreen` (Dioxus) · `FoldersEl` (GPUI), models in `oxyn_core::kit::workscr`.

## 1. Purpose
The files your chats made or used and the folders you keep them in: find, sort and open a folder, make a new one, and copy a template into a file.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Folders persona: the workspace switcher and the ⌘K search, three nav rows (chat, folders, integrations), then the Projects group; the card slot and the account row at the bottom |
| Header | No top bar: the page opens with Files (24 px / 600) and one quiet lead line with a How folders work link, 44 px from the top, 32 px side padding |
| Folders | Caption FOLDERS n (11.5 px uppercase), then All · Shared · Private tabs; on the right Sort (icon menu: Last changed, Name, Most files), the Grid | List switch, Search folders… and the black Create folder button |
| Folder tiles | Four columns, 56 px tall, 10 px radius hairline cards: the folder glyph in the folder tone (#00A1FF light), the name, and Shared/Private · n files with a people or lock glyph |
| File templates | Caption FILE TEMPLATES n, All · Mine · Shared with me tabs; Type ⌄ (filter by kind), A–Z / Z–A, Search templates…, Add template; 236 px cards with a 148 px page preview whose type badge carries the file tone (DOC blue, XLS green, PDF red) |

Chrome is monochrome neutral; colour only means something: official brand marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Tabs, the type filter, both searches and the sort narrow the lists live; Create folder opens a named field in the grid (Enter creates, Escape cancels) and announces it; a tile opens Folders · Detail, as do the sidebar projects, and the crumb or ← comes back; the star keeps a folder first; a template press copies it into a new file.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States: Data · Loading (tile and card skeletons) · Empty (no folders: a glyph, a line and Create folder; no templates: one quiet line) · **Creating** (the new-folder field open and focused). Frame: Inset · Flat. Oxyn extras beyond affectUI: List view with when each folder changed, star to keep a folder first, Most files sort, the template type filter and sort direction.

## 5. Keyboard and accessibility
Folder tiles are buttons named with the folder and who sees it; the star is a toggle (aria-pressed); tab rows are tablists (←/→); Sort, Type and the view switch report their state; the new-folder field takes focus; results and actions are announced in a polite status. Escape closes any open menu or popover and clears a search; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/folders-overview/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and top-bar actions keep their icons with hidden labels. Tiles go 4 → 2 columns under 900 px of page width and 1 under 620 px; the tool rows wrap with the tabs on their own line; templates keep two columns on a phone. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; drag, menus, typing and charts' crosshair belong to the host.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
