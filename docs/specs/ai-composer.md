**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: AI composer (`ai-composer`)

Family: Composer ([composer.md](composer.md)). Closes the parity row(s): Arc UI · AI composer. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
The prompt box of an AI app. `/` at a line start opens commands (/summarize, /translate, /explain, /fix); `@` after a space or at the start opens files and docs (`trigger`, `filter`). Arrow keys move through the open list while the field keeps focus; Enter or Tab inserts the choice as one atomic token, and Backspace on an empty field removes the last token whole (text before it becomes editable again). Starters under the composer fill it (a known command at the start becomes a token). The toolbar holds the model chip with an effort control and a Web search toggle. A ring counter appears at 80 % of `max_length`, turns amber at 95 % and red over it, where sending stops. Sending lifts the message into the thread; a Thinking shimmer (or Searching the web…) covers the moment before the first token; the reply streams in whole-word chunks every 34 ms (`stream_upto`); the send arrow becomes a stop square ringed by the reply's progress and turns back when the stream ends.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| new chat | Greeting, starters | send |
| list open | Commands or mentions | insert / Esc |
| thinking | Shimmer | first token |
| streaming | Reply, stop ring | done / Stop |
| over limit | Red ring, send disabled | delete text |

## 3. Timing and motion
- Arrow lift and stop pop 220 ms; reply chunks every 34 ms; ring 200 ms.
- Reduced motion stops the shimmer and caret blink.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Full |
| limit | Low limit, ring counter |
| bare | No starters |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter | Send or insert |
| Shift + Enter | New line |
| / and @ | Open lists |
| ↑ / ↓ | Move in the list |
| Tab | Insert |
| Esc | Close the list |
| Backspace | Remove a token |

## 6. Accessibility
- The field references the list with aria-activedescendant; Send becomes Stop while streaming.

## 7. Responsive
- Up to 560 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| greeting | String | What should we work on? |  |
| suggestions | Vec<(String, String)> | — | label, prompt |
| models | Vec<(String, String, String)> | — | id, short, note |
| mentions | Vec<String> | — |  |
| placeholder | String | … |  |
| max_length | usize | 4000 |  |
| text / reply | String | — | demo |
| now | ClockMs | — |  |
| on_send | EventHandler<(String, String)> | — | text with /cmd and @name, model |

## 9. Gaps against the source
- Files upload in place is represented by the Attach button only; the uploading chip lives in the main Composer.

## 10. GPUI
AiComposerEl: tokens and the ring counter render in GPUI; streaming text uses `StreamingText`.
