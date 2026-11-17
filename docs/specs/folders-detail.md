**Status:** built v0.2 (2026-11-18, S2: Workspace parity; v0.1 screens 1).

# Spec: Folders · Detail (`folders-detail`)

Family: Screens. Reference: affectUI Folders / Detail (`/screens/folder`), layout and behaviour only; no source or assets copied.
Components: `FoldersScreen { detail: true }` (Dioxus) · `FolderDetailEl` (GPUI), models in `oxyn_core::parity::workscr`.

## 1. Purpose
One folder open: what is in it, who it is shared with, and a way to add more.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Folders persona with the open project current |
| Top bar | 48 px: the crumb Files / Autumn launch |
| Back | ← Files, quiet, above the title |
| Title row | The folder glyph (30 px, folder tone) and the name (24 px), then More (Rename, Duplicate, Move to…, Delete folder), sort, Grid | List, Share and the black Upload files |
| File cards | 236 px cards with a 148 px page preview and the file type badge in its tone, the name and Kind · when ("Document · 2 hours ago") |
| Empty | A dashed drop area: an upload glyph, This folder is empty, one line and Upload files |

Chrome is monochrome neutral; colour only means something: official brand marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Upload files adds a file in place and announces it; Share opens a dialog with an invite field (Enter invites), the people on the folder with Remove, and Copy link; More and sort are menus; the view switch is shared with the Overview.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States: Data · Loading (card skeletons) · Empty (the drop area) · **Uploading** (two files with progress bars). Frame: Inset · Flat. Oxyn extras: List view, sort, the Share dialog with people, the Uploading state.

## 5. Keyboard and accessibility
The Share popover is a labelled dialog that Escape closes; upload progress is a progressbar with its value; the back link and crumb return to the Overview; file cards are buttons named by the file. Escape closes any open menu or popover and clears a search; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/folders-detail/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and top-bar actions keep their icons with hidden labels. Under 620 px the title row wraps under the name, sort steps aside and the cards keep two columns. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; drag, menus, typing and charts' crosshair belong to the host.
