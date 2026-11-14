**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Input (`labeled-input`)

Family: Composer & inputs ([composer-input-shell.md](composer-input-shell.md)). Closes the parity row(s): Arc UI · Input.

## 1. Purpose
A single-line field with a label, a helper row linked through aria-describedby and an error row (role=alert, aria-invalid) that replaces it. Rows open their height on a smooth spring (a grid row from 0fr to 1fr with a slight overshoot); words that changed rise in and unblur while unchanged words stay put, and numbers roll only the digits that changed (keys by place from the end).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| helper | Description row |  |
| invalid | Error row, red edge | error cleared |
| bare | No row | description set → opens |

## 3. Timing and motion
- Row height: 420 ms spring-like ease with overshoot; words: 320 ms rise with a 3 px blur; digits: 300 ms roll.
- Reduced motion: rows open at full height, words and digits swap without rise or roll, the ghost appears and turns solid without fading, the height tween is off.

## 4. Variants
| Variant | What differs |
|---|---|
| helper | Default |
| error | Error |
| bare | No helper |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Focus |

## 6. Accessibility
- role=alert on the error row; aria-invalid; aria-describedby.

## 7. Responsive
- Fills its container.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | String | — |  |
| value | String | "" |  |
| description / error | Option<String> | — |  |
| kind | text · email · password · url | text |  |
| on_input | EventHandler<String> | — |  |

## 9. Edge cases
- `word_changes` / `digit_changes` describe the same diff for hosts without keyed rendering (GPUI).

## 10. Implementation
- Core: `word_changes`, `digit_changes` (`oxyn_core::parity::input`), with tests.
- Dioxus: `LabeledInput` (parity/input.rs, input.css).
- GPUI: `ImeInput` (parity/ime.rs) is the GPUI text entry: IME composition with an underline, selection (shift-arrows, drag, select all), clipboard, word moves, Enter / Shift+Enter events. A wrapped multi-line layout is a follow-up: a line break draws as a space.

## 11. References
- Arc UI Input (https://uiarc.dev/components/input) — behaviour-only rebuild, no source copied.
