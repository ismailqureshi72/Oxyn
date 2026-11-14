**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Textarea (`labeled-textarea`)

Family: Composer & inputs ([composer-input-shell.md](composer-input-shell.md)). Closes the parity row(s): Arc UI · Textarea.

## 1. Purpose
The multi-line `Input`: the same helper and error rows, plus an optional count (“· 120 characters”) whose digits roll as you type.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| helper | Description and count |  |
| invalid | Error row | error cleared |

## 3. Timing and motion
- Reduced motion: rows open at full height, words and digits swap without rise or roll, the ghost appears and turns solid without fading, the height tween is off.

## 4. Variants
| Variant | What differs |
|---|---|
| count | Rolling count |
| error | Error |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Focus |

## 6. Accessibility
- role=alert on errors; aria-describedby.

## 7. Responsive
- Fills its container; resizes vertically.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | String | — |  |
| description / error | Option<String> | — |  |
| count | bool | false |  |
| rows | usize | 4 |  |
| max_length | Option<usize> | — |  |

## 9. Edge cases
- The count follows characters, not bytes.

## 10. Implementation
- Core: `digit_changes` (`oxyn_core::parity::input`), with tests.
- Dioxus: `LabeledTextarea` (parity/input.rs, input.css).
- GPUI: `ImeInput` (parity/ime.rs) is the GPUI text entry: IME composition with an underline, selection (shift-arrows, drag, select all), clipboard, word moves, Enter / Shift+Enter events. A wrapped multi-line layout is a follow-up: a line break draws as a space.

## 11. References
- Arc UI Textarea (https://uiarc.dev/components/textarea) — behaviour-only rebuild, no source copied.
