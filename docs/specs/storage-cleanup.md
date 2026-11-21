**Status:** built v0.1 (2026-11-16, variants 2: Storage family).

# Spec: Storage cleanup (`storage-cleanup`)

Family: Storage. Reference: the behaviour of a cleanup panel in a public notch app (unused skills with checkboxes, a not-used-in window, counters, move to Trash with undo). Behaviour only; names, figures and wording are our own.
Components: `StorageCleanup` (Dioxus) · `StorageCleanupEl` (GPUI), models in `oxyn_core::kit::storagefid`.

## 1. Purpose
Give back context an agent wastes every session on skills and commands nobody uses.

## 2. Parts
| Part | Look |
|---|---|
| Head | "Free up context" 16/600 and one muted line |
| Window | "Not used in" segmented 30 days · 90 days · 6 months |
| Counters | unused · never used · selected |
| Dot meter | 40 dots: the share of every-session tokens the picked items cost, in green (status colour), and "1,433 of 1,828 tokens freed every session" |
| List | checkbox rows: monospace name, place tag, "Used 5 months ago" (`idle_line`), tokens |
| Foot | Move N to Trash (ink), then Undo |

## 3. Behaviour
Changing the window re-picks every item in it (`unused`). Rows and Select all toggle; the dots and counters follow. Move removes them from the list and offers Undo. `on_trash(names)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
30 days (default) · 6 months · 90 days.

## 5. Keyboard and accessibility
Window is a radiogroup; rows and Select all are `role="checkbox"` buttons. The dot meter is an image with the token sentence; moves are said in a status line.

## 6. Responsive
440 px card; the list scrolls in a 244 px box; the place tag hides under 480 px.

## 7. Known gaps
Trash is local state only. GPUI: static snapshot (`trash`).
