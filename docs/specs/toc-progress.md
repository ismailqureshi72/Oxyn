**Status:** v1 — implemented (plan #27).

# Spec: table of contents progress

> Implemented in `oxyn-core::primitives::layout (`toc_progress`, `time_left`, `reading_minutes`)`, `oxyn-dioxus::TocProgress, TocSection`, `oxyn-gpui::TocProgressEl`.
> Source: Arc UI Table of contents progress (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Long answers, research reports and docs: where you are and how much is left.

## 2. State machine
```
Scroll ─▶ (section, fraction, overall) — a heading is reached at `offset`; near the end the reading line eases to the bottom edge
Heading ─Enter / click─▶ article scrolls to it
overall ≥ 99.5 % ─▶ “Finished” ✓
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| upcoming | subtle text | Tab | link |
| current | strong text, marker | — | `aria-current=location` |
| read | muted, filled line | — | — |
| finished | check | — | — |

## 4. Micro-interactions
Each section fills the line from its heading to the next; the marker rides at the fill tip; heading colours change over 240 ms; the time left rolls.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Fill | follows scroll | linear | same |
| Colour | 240 ms | linear | instant |
| Time roll | 240 ms | `ease.out` | swap |

## 6. Variants
Free: right or left contents, title, time left.

**Oxyn plus:** `bar` mode (a single progress bar for narrow screens), its own scrolling article viewport.

## 7. Edge cases
Short last sections are still reached at the bottom. Below 560 px of container width the contents move above.

## 8. Accessibility
`nav` landmark named by the title with an ordered list of anchors; contrast marks state, not only the line; the line is decorative.

## 9. Props sketch
```rust
TocProgress { sections: Vec<TocSection>, title, reading_time: f32, show_time_left: bool, offset: f32, height: f32,
             side: "right"|"left"|"bar", on_active_change: EventHandler<String> }
```

## 10. Related components and references
`streaming-markdown.md`, `scroll-area.md`. Arc UI Table of contents progress — behaviour only.
