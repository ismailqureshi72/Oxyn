**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Theme split (`theme-split`)

Family: Theme Selector. Reference: none (an Oxyn original).
Components: `ThemeSplit` (Dioxus) · `ThemeSplitEl` (GPUI), models in `oxyn_core::kit::themefid`.

## 1. Purpose
Compare the theme in use with a candidate on the same window, side by side, before switching.

## 2. Parts
| Part | Look |
|---|---|
| Head | "Compare themes" and candidate dots (pane / line halves); the theme in use is disabled |
| Stage | 220 px sample window drawn twice, the candidate clipped from the divider (`clip-path`) |
| Divider | 2 px white rule with a pill grip; tags name both sides |
| Foot | Centre (ghost) and "Use <theme>" (ink) |

## 3. Behaviour
Dragging anywhere on the stage moves the divider (a transparent native range over it, 8–92 %, `split_clamp`); arrow keys step it. A dot picks the candidate. Use swaps the two, so the old theme becomes the candidate to compare back. `on_apply(theme)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (Light vs Midnight) · dark vs contrast · moss vs plum.

## 5. Keyboard and accessibility
The divider is a native range labelled with both themes; candidates are a radio group; Use and Centre report in a polite note.

## 6. Responsive
The sample's sidebar narrows under 480 px.

## 7. Known gaps
No vertical split. GPUI: static snapshot (host routes `cand-<i>` and `apply`).
