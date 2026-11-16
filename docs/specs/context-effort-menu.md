**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Context + Effort menu (`context-effort-menu`)

Family: Thinking Selector. Reference: a user-supplied screenshot of a composer dropdown with "Context window" and "Effort" groups (layout idea only; names, wording and colours are our own).
Components: `ContextEffortMenu` (Dioxus) · `ContextEffortMenuEl` (GPUI), models in `oxyn_core::parity::{effmenufid, reasonfid, thinkfid}`.

## 1. Purpose
Set the context window and the effort of the current model in one dropdown, from one chip in the composer.

## 2. Parts
| Part | Look |
|---|---|
| Chip | 30 px round, hairline; model name 13.5/500 + muted summary "512K · high" + caret |
| Menu | 220 px, 12 px radius, soft shadow |
| Group heading | "Context window" / "Effort", 11.5 muted |
| Window rows | 31 px; size (`context_label`) + optional tag pill ("Higher usage"); chosen row filled + check |
| Effort rows | default, low, medium, high, xhigh, max (`effort_scale_api`); max in red |
| Separator | full-bleed hairline between groups |

## 3. Behaviour
Two radio groups, one keyboard list (`menu_move` over both). A pick sets that group's value and keeps the menu
open so both can be changed; Escape or Tab closes it and focus returns to the chip. Uncontrolled
(`default_window`, `default_effort`); `on_window_change`, `on_effort_change`.
prefers-reduced-motion: no transitions or pop-in.

## 4. Variants
open (default) · big (1M + max) · closed.

## 5. Keyboard and accessibility
Menu button pattern; `role="group"` per section with `aria-labelledby`; rows `menuitemradio` + `aria-checked`; highlight via `aria-activedescendant`. ↑/↓ wrap, Home/End, Enter/Space pick, Escape close.

## 6. Responsive
220 px menu, capped at the viewport minus 32 px; the chip ellipsises its summary.

## 7. Known gaps
No click-outside close; values are uncontrolled only. GPUI: static snapshot.
