**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Streaming text (`streaming-text`)

Family: Chat ([streaming-markdown.md](streaming-markdown.md)). Closes the parity row(s): Beautiful UI · Streaming Text.

## 1. Purpose
A streamed answer: words fade in at a pace with a caret, `[n]` markers become citation chips (the source's host, linking out) that pop in place; once done a “10 sources” button with stacked favicons lists the sources, actions appear (copy, good, bad, retry) and follow-up questions can be asked.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| streaming | Words + caret, aria-busy |  |
| done | Sources, actions, follow-ups |  |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| streaming | Default |
| done | Finished |
| bare | No follow-ups |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Citations, sources, actions, follow-ups |
| Enter | Open sources / ask |

## 6. Accessibility
- aria-busy while streaming; named action buttons; votes use aria-pressed.

## 7. Responsive
- Fills its container.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| stream | Stream | — | text, started, wps |
| now | ClockMs | — |  |
| sources | Vec<Source> | — | name, host, url |
| source_count | Option<usize> | sources.len() |  |
| followups | Vec<String> | [] |  |
| on_follow | EventHandler<String> | — |  |
| on_retry | EventHandler<()> | — |  |

## 9. Edge cases
- An unknown citation index shows the number.

## 10. Implementation
- Core: `Stream`, `Piece` (`oxyn_core::kit::text`), with tests.
- Dioxus: `StreamingText` (kit/text.rs, text.css).
- GPUI: not yet (plan #28 follow-up; the core models are shared).

## 11. References
- Beautiful UI Streaming Text (https://www.beautifului.dev/#streaming-text), MIT, © 2026 Shane Levine — behaviour rebuilt in Rust; no source or icons copied.
