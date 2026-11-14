**Status:** v1 — implemented (plan #26).

# Spec: scroll area

> Implemented in `oxyn-core::primitives::scroll` (`Metrics`, `Scrollbars`, `StickToBottom`, `fades`, `thumb`, `page_toward`), `oxyn-dioxus::ScrollArea`, `oxyn-gpui::ScrollAreaEl`.
> Source: Arc UI Scroll area; stick-to-bottom from pr-plan #26 (behaviour only, no source used).

## 1. Purpose
Long transcripts, file lists and strips inside panels. Scrolling must stay native (momentum, keyboard, screen readers), with quieter bars and fades that say there is more.

## 2. State machine
```
Idle(bars hidden) ─scroll / hover─▶ Active(bars shown) ─hide_delay─▶ Idle
Track press ─▶ page 90 % of the viewport toward the press
Following (at bottom) ─content grows─▶ scroll to bottom
Following ─user scrolls up─▶ Detached(pill "Jump to latest", count of new items)
Detached ─pill / reach bottom─▶ Following
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | content, edge fades | native keys | region when labelled |
| active | overlay thumb(s) | — | bars `aria-hidden` |
| following | sticks to the newest line | — | — |
| detached | "↓ 3 new messages" pill | Enter on pill | pill is a `button` |

## 4. Micro-interactions
Thumbs fade in while scrolling or on hover and fade out after `hide_delay`; under a fine pointer they thicken on hover. Edge fades grow with the distance left to each edge (capped at `fade`). New transcript lines rise in.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Bars in | 120 ms | linear | same |
| Bars out | 300 ms after `hide_delay` | linear | same |
| Thumb thicken | 120 ms | linear | none |
| Jump to latest | smooth scroll | native | instant |

## 6. Variants
Free: vertical, horizontal, both; fade size; scrollbars auto · always · hidden; hide delay.

**Oxyn plus:** stick-to-bottom with the jump pill and a new-item count, streaming demo, scroll snap.

## 7. Edge cases
Short content draws no bar and no fades. Very long content keeps a minimum thumb of 24 px. Touch keeps native momentum and overscroll containment.

## 8. Accessibility
Native scrolling is untouched: the viewport is focusable and keyboard-scrollable; with a label it is a named region. Bars are decorative; the pill is a real button.

## 9. Props sketch
```rust
ScrollArea { orientation, fade: f32, scrollbars: "auto"|"always"|"hidden", hide_delay: u32,
             height, width, label: Option<String>, snap: Option<String>, stick_to_bottom,
             lines: Vec<Line>, stream, stream_ms, children, on_edge_change }
```

## 10. Related components and references
`messages.md` (transcripts), `diff-viewer.md`. Arc UI `/docs/components/scroll-area` — behaviour only.
