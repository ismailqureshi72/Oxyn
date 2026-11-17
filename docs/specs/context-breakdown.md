**Status:** built v0.1 (2026-11-16, variants 2: Context Visualizer family).

# Spec: Context breakdown (`context-breakdown`)

Family: Context Visualizer. Reference: the item lists of a public context-map component and a storage-cleanup panel (what each part holds, unload to free room). Behaviour only; wording and look are our own.
Components: `ContextBreakdown` (Dioxus) · `ContextBreakdownEl` (GPUI), models in `oxyn_core::parity::ctxvisfid`.

## 1. Purpose
Answer "what exactly is in there" and let the user unload a file or tool to give room back.

## 2. Parts
| Part | Look |
|---|---|
| Head | "Context", "166.4K of 200K tokens" 12.5 muted; fill pill (grey, amber from 75 %, red at the compaction line) |
| Track | 6 px stacked bar of all kinds; the open kind stays lit |
| Order | segmented "By size" / "Load order" |
| Row | 38 px button: swatch, name, share bar relative to the largest kind, tokens, percent, chevron |
| Items | indented list of names and tokens; files and tools get an Unload button |
| Foot | "About N turns before it compacts at 92 %" (`turns_left`), red when the next turn passes it |

## 3. Behaviour
A row opens and closes its items (one open at a time). Unload removes the item and its tokens (`ctx_drop_item`), the pill, track and foot follow. `on_drop((part, item))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
files open (default) · load order · read only (no Unload, mono).

## 5. Keyboard and accessibility
Rows are buttons with `aria-expanded`; the order switch is a radiogroup. The pill is text, the colour extra. Unloading is said in a status line.

## 6. Responsive
360 px card; the share bar flexes, names never wrap.

## 7. Known gaps
Turn size for the foot is fixed at 6K. GPUI: static snapshot (rows route `kind-<i>`).
