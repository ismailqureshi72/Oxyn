**Status:** built v0.1 (2026-11-16, variants 2: Menu Motion family).

# Spec: Gooey composer (`gooey-composer`)

Family: Menu Motion. Reference: the attach (plus) and model menus of a public chat-composer component, which grow out of the corner beside their buttons, cascade rows and glide a hover fill. Behaviour only, rebuilt from scratch; items, icons and wording are our own.
Components: `GooeyComposer` (Dioxus) · `GooeyComposerEl` (GPUI), models in `oxyn_core::kit::gooeyfid`.

## 1. Purpose
Put the gooey menu motion on a real prompt box: both menus pour out of the box's top edge above their buttons.

## 2. Parts
| Part | Look |
|---|---|
| Box | up to 560 px, 22 px radius, drawn in the goo layer so menus melt out of its top edge |
| Field | chips of picked files (file-type icons), a plain textarea "Ask anything…" |
| Foot | + button (38 px, turns 45° when open), the model chip (maker's mark + name + caret), send (ink when there is something to send) |
| + menu | spans the box; groups Add (PDF, Image, Excel, Doc, Slides, Code – file-type icons) and From (Drive, GitHub – vetted brand marks) |
| Model menu | 268 px, lined up with the chip; makers' marks, names and hints; the chosen one checked |

## 3. Behaviour
The shared gooey menu motion: the button, a stem and the menu sheet are drawn in one layer under an SVG goo filter (blur `goo` px, then a hard alpha curve), so shapes near each other join with a liquid neck; rows sit above it, unfiltered. Open: the stem stretches out of the button (0.24 s) and pinches off while the sheet swells from the corner nearest the button with a slight overshoot (0.46 s); rows arrive top to bottom (`stagger_ms`: 70 ms, then 24 ms apart, a 3 px blur clearing); a grey fill glides under the pointer (0.17 s). Close: the sheet runs back to the corner fast (0.18 s) and the menu leaves the page when that animation ends. Timings were tuned by eye (the reference publishes none). Picking a file type adds a chip; picking a model blurs the new name into the chip. Opening one menu closes the other. `on_send(text)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
attach open · model open · closed.

## 5. Keyboard and accessibility
Triggers carry `aria-haspopup` and `aria-expanded`; the + menu is a `menu` of `menuitem`s, the model menu a `listbox` of `option`s with `aria-selected`, both followed with `aria-activedescendant`. ↑/↓ on a trigger opens its menu and moves focus in; Escape returns to the trigger; Tab closes. A press outside closes a menu the user opened.

## 6. Responsive
The + menu takes the box's width; the + menu shows all eight rows (scrolls past 360 px), the model menu scrolls past 300 px.

## 7. Known gaps
Menus always open upward here (the solo menus take `side`); no measured flip to the roomier side yet. GPUI: static settled snapshot, no goo.
