**Status:** built v0.1 (2026-11-16, variants 2: Menu Motion family).

# Spec: Liquid tabs (`liquid-tabs`)

Family: Menu Motion. Reference: none (an Oxyn original).
Components: `LiquidTabs` (Dioxus) · `LiquidTabsEl` (GPUI), models in `oxyn_core::parity::gooeyfid`.

## 1. Purpose
A mode switch whose selection feels physical without being loud.

## 2. Parts
| Part | Look |
|---|---|
| Track | 40 px, 13 px radius, grey |
| Pill | ink, springs to the chosen tab (0.5 s) |
| Trail | a slimmer ink droplet that follows slower (0.82 s); the goo joins it to the pill, so it stretches and is pulled back in |
| Tabs | equal width; the chosen label turns card-coloured on the ink |

## 3. Behaviour
Click or arrow keys move the choice (`tab_box`). `on_change(index)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
four · two · five.

## 5. Keyboard and accessibility
`tablist` with `tab`s; roving tab stop; ←/→, Home/End.

## 6. Responsive
Up to 360 px; tabs share the width.

## 7. Known gaps
GPUI: static (no trail).
