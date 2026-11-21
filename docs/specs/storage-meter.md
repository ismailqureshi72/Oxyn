**Status:** built v0.1 (2026-11-16, variants 2: Storage family).

# Spec: Storage meter (`storage-meter`)

Family: Storage. Reference: the behaviour of a public storage-scanner experiment and the notch app that ships it (dot-matrix capacity bar, category grid with a hover/click detail hint, Rescan, clear caches). Behaviour only; categories, figures, icons and wording are our own.
Components: `StorageMeter` (Dioxus) · `StorageMeterEl` (GPUI), models in `oxyn_core::kit::storagefid`.

## 1. Purpose
Show what fills a disk and offer the one quick win, in a card small enough for a menu-bar panel.

## 2. Parts
| Part | Look |
|---|---|
| Head | used 30/600 tabular, "of 512 GB used" muted, Rescan pill |
| Sub | "223.8 GB available", "Calculating…" while scanning |
| Dot bar | outlined track, `cols` × `rows` square dots filled left to right, one colour block per category (`dot_runs`, largest remainder); free dots are hollow |
| Grid | three columns of category buttons: tinted glyph tile (our own drawings), label, size |
| Hint | inset row: the focused category's detail (`store_detail`) or the biggest clearable one, with its action (Clear caches · Empty Trash) |

## 3. Behaviour
Pointing at a category dims the other dots and the other tiles and fills the hint; a click keeps it. Rescan replays the dots column by column and sweeps a light across the bar; the state ends on the sweep's animation end. Clear dims that category's dots, runs a thin progress line, then shrinks it to its floor (`store_clear`) and says what was freed. `on_clear((key, gb))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · caches picked · small disk (finer bar) · read only.

## 5. Keyboard and accessibility
The bar is `role="img"` labelled "288.2 GB of 512 GB used". Categories are buttons with `aria-pressed`. The hint is a polite live region.

## 6. Responsive
420 px card; two grid columns under 480 px.

## 7. Known gaps
Rescan does not change the figures (demo data). Reduced motion: timed states end after 1 ms. GPUI: static snapshot.
