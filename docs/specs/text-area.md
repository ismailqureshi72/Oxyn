**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Text area (`text-area`)

Family: Composer & inputs ([composer-input-shell.md](composer-input-shell.md)). Closes the parity row(s): affectUI · Text Area.

## 1. Purpose
The multi-line twin of the text field. It grows with its text from `min_rows` (3) to `max_rows` (10) on a 180 ms height tween, then scrolls inside; a counter shows “120/2,000” (red, and the field invalid, when a prefilled value is already over the limit); an optional corner button gives the field `expanded_rows` (16) of room and back; ⌘/Ctrl+Enter submits while Enter alone makes a new line. The label focuses the field; the error replaces the hint with a red edge and a faint red fill.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| empty | min_rows tall, placeholder |  |
| growing | Height follows the text | past max_rows → scrolling |
| expanded | expanded_rows tall | button → back |
| invalid | Red edge and fill, error row | error cleared |
| disabled | Dimmed |  |

## 3. Timing and motion
- Reduced motion: rows open at full height, words and digits swap without rise or roll, the ghost appears and turns solid without fading, the height tween is off.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Counter and more-room button |
| error | Error row |
| over | Prefilled over the limit |
| prompt | Monospace, optional, empty |

## 5. Keyboard
| Key | Does |
|---|---|
| ⌘/Ctrl + Enter | Submit |
| Enter | New line |
| Tab | To the more-room button |

## 6. Accessibility
- <label for> focuses the field.
- aria-invalid; aria-describedby → the hint or error row.
- More-room button: aria-expanded, aria-controls.

## 7. Responsive
- Fills its container.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | Option<String> | — |  |
| value | String | "" | initial |
| hint / error | Option<String> | — | error wins |
| optional | bool | false |  |
| counter_on | bool | false | needs max_length |
| max_length | Option<usize> | — |  |
| min_rows / max_rows | usize | 3 / 10 |  |
| expandable | bool | false |  |
| expanded_rows | usize | 16 |  |
| mono | bool | false |  |
| on_input / on_submit | EventHandler<String> | — |  |
| on_expanded | EventHandler<bool> | — |  |

## 9. Edge cases
- Wrapped lines are estimated at 56 characters per row for the height; CSS scrolls past the maximum.
- maxlength is only set while the text is within it, so a prefilled overflow stays visible and red.

## 10. Implementation
- Core: `grow_rows`, `counter` (`oxyn_core::kit::input`), with tests.
- Dioxus: `TextArea` (kit/input.rs, input.css).
- GPUI: `ImeInput` (kit/ime.rs) is the GPUI text entry: IME composition with an underline, selection (shift-arrows, drag, select all), clipboard, word moves, Enter / Shift+Enter events. A wrapped multi-line layout is a follow-up: a line break draws as a space.

## 11. References
- affectUI Text Area (https://www.affectui.com/components/text-area) — behaviour-only rebuild, no source copied.
