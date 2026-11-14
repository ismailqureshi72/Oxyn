**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Ghost text (`ghost-text`)

Family: Composer & inputs ([composer-input-shell.md](composer-input-shell.md)). Closes the parity row(s): Arc UI · Ghost text.

## 1. Purpose
Inline AI autocomplete. After `delay_ms` (450) of pause and at least `min_len` (3) trimmed characters, the model is asked for a continuation of the text; it shows muted right after the text, fading in (140 ms). Typing that matches the ghost consumes it in place; any other edit clears it and a new ask follows the next pause. Tab accepts all of it, ⌘/Ctrl/Alt+→ accepts one word (the rest stays), Escape dismisses it (with nothing showing Escape is left to the page); moving the caret away (←, ↑, Home, a click) or leaving the field clears it. Accepted text eases from muted to solid in place (200 ms). A chip names the model and shows Thinking, Tab to accept or No suggestion; key hints sit beside it on wide fields. Multi-line (growing, label row, footer) or single line (chip inline at the end); field or plain (flush in a card).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Model chip |  |
| loading | Thinking (pulsing dot) | answer → ready; fail → error |
| ready | Ghost after the text, Tab to accept | accept / dismiss / edit |
| error | No suggestion | next edit |

## 3. Timing and motion
- Reduced motion: rows open at full height, words and digits swap without rise or roll, the ghost appears and turns solid without fading, the height tween is off.

## 4. Variants
| Variant | What differs |
|---|---|
| field | Multi-line with a suggestion waiting |
| single | Single line, chip inline |
| plain | Flush in a card |
| offline | Error chip |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Accept all (moves focus when nothing shows) |
| ⌘/Ctrl/Alt + → | Accept a word |
| Esc | Dismiss |
| ← ↑ Home, click | Clear |

## 6. Accessibility
- The ghost is drawn in an aria-hidden mirror under a transparent-text field, so the field's own value is what assistive tech reads.
- aria-describedby explains the keys; the chip is a polite live region.

## 7. Responsive
- Key hints hide below 420 px of container width; the chip stays.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| value | String | "" | start |
| placeholder / label | Option<String> | — |  |
| model | String | Autocomplete | chip |
| multiline | bool | true |  |
| variant | field · plain | field |  |
| rows | usize | 3 |  |
| hint | bool | true |  |
| delay_ms / latency_ms | u64 | 450 / 380 |  |
| offline | bool | false | demo error |
| now | ClockMs | — |  |
| on_accept | EventHandler<(String, &str)> | — | text, all · word |
| on_input | EventHandler<String> | — |  |

## 9. Edge cases
- A stale answer (the text moved on) is dropped.
- A dismissed text is not asked again until it changes.
- Past ten lines the mirror does not follow the field's own scroll (follow-up).

## 10. Implementation
- Core: `Ghost` (`settle`, `edit`, `accept_all`, `accept_word`, `dismiss`), `next_word_end`, `demo_suggestion` (`oxyn_core::parity::input`), with tests.
- Dioxus: `GhostText` (parity/input.rs, input.css).
- GPUI: `ImeInput::set_ghost` draws a muted continuation after the text; Tab accepts it all, Alt/Cmd+→ one word, Escape dismisses (the host owns the `Ghost` model).

## 11. References
- Arc UI Ghost text (https://uiarc.dev/components/ghost-text) — behaviour-only rebuild, no source copied.
