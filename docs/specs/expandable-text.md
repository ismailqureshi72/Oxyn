**Status:** v1 — implemented (plan #27).

# Spec: expandable text

> Implemented in `oxyn-core::primitives::layout (`needs_clamp`, `expand_duration`)`, `oxyn-dioxus::ExpandableText`, `oxyn-gpui::ExpandableTextEl`.
> Source: Arc UI Expandable text (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Long run summaries, descriptions and message bodies that should not push everything else down.

## 2. State machine
```
Clamped(lines) ─Read more─▶ Open ─Show less─▶ Clamped
text fits ─▶ no toggle
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| clamped | n lines + fade | Enter on toggle | `aria-expanded=false` |
| open | full text | same | `aria-expanded=true` |
| fits | text only | — | — |

## 4. Micro-interactions
Height and the fade move together on a soft ease; the two labels crossfade in one cell with a short drift and blur; the chevron flips.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Open | 560 ms | cubic-bezier(.32,.5,.2,1) | instant |
| Close | 420 ms | same | instant |
| Label crossfade | 160–200 ms | `ease.out` | instant |

## 6. Variants
Free: line count, custom labels.

**Oxyn plus:** fade off, paragraphs split on blank lines.

## 7. Edge cases
The toggle is measured: short text never shows it. The clamp is plain CSS so resizes need no script.

## 8. Accessibility
The whole text stays in the accessibility tree; the toggle has `aria-expanded` and `aria-controls`; the hidden label is `aria-hidden`.

## 9. Props sketch
```rust
ExpandableText { text: String, lines: u32, expanded: bool, more_label, less_label, fade: bool, on_change: EventHandler<bool> }
```

## 10. Related components and references
`accordion.md`, `message-bubble.md`. Arc UI Expandable text — behaviour only.
