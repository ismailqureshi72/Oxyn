**Status:** built v0.1 (2026-11-16, variants 2: Storage family).

# Spec: Storage waffle (`storage-waffle`)

Family: Storage. Reference: none (an Oxyn original).
Components: `StorageWaffle` (Dioxus) · `StorageWaffleEl` (GPUI), models in `oxyn_core::parity::storagefid`.

## 1. Purpose
A whole disk at a glance where one dot is a fixed amount, and a lens that answers "how much could I get back".

## 2. Parts
| Part | Look |
|---|---|
| Head | "Disk" muted and the headline 20/600 ("288.2 GB of 512 GB used" or "35.9 GB you can win back"); lens By kind · Win back |
| Waffle | 20 × 10 round dots, half a percent each, category colours in load order |
| Legend | category rows with swatch, label and size |
| Foot | "One dot is 2.6 GB." |

## 3. Behaviour
Pointing at or focusing a legend row lights only its dots. The Win back lens keeps caches and Trash lit, dims the rest and changes the headline (it rolls in).
prefers-reduced-motion: no transitions or animation.

## 4. Variants
by kind (default) · win back · mono 1 TB.

## 5. Keyboard and accessibility
The waffle is an image labelled with the headline; legend rows are focusable and say size and dot count; the lens is a radiogroup.

## 6. Responsive
440 px card; the legend moves under the waffle under 480 px.

## 7. Known gaps
GPUI: static snapshot, kind lens only.
