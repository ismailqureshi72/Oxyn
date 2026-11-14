**Status:** v1 — implemented (plan #27).

# Spec: rating

> Implemented in `oxyn-core::primitives::layout (`Rating`, `RatingKey`, `RATING_WORDS`)`, `oxyn-dioxus::Rating`, `oxyn-gpui::RatingEl`.
> Source: Arc UI Rating (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Rate an answer, a run or a tool; show an average read-only.

## 2. State machine
```
Value v ─hover─▶ Preview(h) ─leave─▶ v
v ─press star k─▶ k (same value → 0 when allow_clear)
keys: ±step, Home End, digits, Backspace clears
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| not rated | empty stars, “Not rated” | arrows | radiogroup |
| preview | fill follows pointer | — | — |
| rated | filled, word | arrows | `aria-checked` on the value |
| read only | static | — | `img` “Quality: 4 out of 5” |

## 4. Micro-interactions
Fill sweeps star by star with a 30 ms stagger; pressed stars dip to 0.86; the value word rolls in the direction of change.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Fill sweep | 160 ms + 30 ms stagger | `ease.out` | instant |
| Word roll | 200 ms | `ease.out` | swap |

## 6. Variants
Free: 5 stars, half steps, sizes, read only, hint.

**Oxyn plus:** `heart` and `dot` glyphs, any max (3–10), custom words.

## 7. Edge cases
0 means not rated; `allow_clear: false` keeps at least one step. Half stars come from the left half of a star.

## 8. Accessibility
One radio per whole star labelled “Great, 4 of 5 stars”; commits announced politely; read-only is an image with a label.

## 9. Props sketch
```rust
Rating { label, value: f32, max: u32, allow_half: bool, allow_clear: bool, read_only: bool, disabled: bool,
        show_value: bool, description, size, glyph: "star"|"heart"|"dot", labels: Vec<String>, on_change: EventHandler<f32> }
```

## 10. Related components and references
`response-feedback` (message-bubble.md), `toggles.md`. Arc UI Rating — behaviour only.
