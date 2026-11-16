**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Model effort menu (`model-effort-menu`)

Family: Thinking Selector. Reference: a user-supplied screenshot of a model list with an effort suffix per model and a cascading effort submenu (layout idea only; names, wording and colours are our own).
Components: `ModelEffortMenu` (Dioxus) · `ModelEffortMenuEl` (GPUI), model in `oxyn_core::parity::effmenufid`.

## 1. Purpose
Pick a model and its effort in one gesture: every model shows its effort as a suffix, and its submenu changes it.

## 2. Parts
| Part | Look |
|---|---|
| List | 330 px, 14 px radius, soft shadow; a provider-neutral bolt mark on top |
| Row | 36 px: name 15/500 + effort suffix (muted; red at Max); a check on the chosen model, a › on the others |
| Submenu | 160 px, cascades to the right of its row (8 px gap); Low … Max, the model's level checked, Max in red |
| Chip | 32 px, 10 px radius; the chosen model's name + caret |

## 3. Behaviour
Hovering a row, clicking it, → or Enter opens its submenu; picking a level sets that model's level, makes it the
chosen model and closes everything (focus back to the chip). ← or Escape inside a submenu returns to the list. A press outside closes it when the user opened it; a start-open preview draws no catcher.
The demo's seven models are fictional (`effort_models`). `on_change((model, level))`.
prefers-reduced-motion: no transitions or pop-in.

## 4. Variants
submenu (default) · list · closed.

## 5. Keyboard and accessibility
Rows are `menuitem` with `aria-haspopup` / `aria-expanded` and a name that includes the effort; levels are `menuitemradio`. One focus owner (the list) with `aria-activedescendant` pointing into the submenu when it is open.

## 6. Responsive
Under a 640 px viewport the submenu opens in place under its row instead of to the right.

## 7. Known gaps
No hover-intent delay for diagonal pointer moves; no type-ahead. GPUI: static snapshot.
