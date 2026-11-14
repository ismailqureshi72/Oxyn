**Status:** v1 — implemented (plan #27).

# Spec: radio cards

> Implemented in `oxyn-core::primitives::layout (`RadioCard`, `radio_step`)`, `oxyn-dioxus::RadioCards`, `oxyn-gpui::RadioCardsEl`.
> Source: Arc UI Radio cards (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Choices that need a description or price: plans, permission modes, models.

## 2. State machine
```
None ─press / arrow─▶ Selected(v)   (arrows wrap, skip disabled)
Disabled card ─press─▶ no change (reason read as description)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| unselected | card, empty dot | Arrows select | `radio` `aria-checked=false` |
| selected | ring + filled dot | — | `aria-checked=true` |
| disabled | 50 %, reason | skipped | `aria-disabled`, described by reason |

## 4. Micro-interactions
One ring glides to the selected card’s box and size; the dot pops in on a snappy spring.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Ring glide | 260 ms | `ease.out` | jump |
| Dot pop | 220 ms | overshoot | instant |

## 6. Variants
Free: `grid` (auto-fill columns, `min_column`), `list` rows with meta, hidden form input.

**Oxyn plus:** risk tones (low · medium · high) for permission modes, icons, disabled group.

## 7. Edge cases
Grid drops to one column in narrow containers; descriptions wrap; nothing truncates.

## 8. Accessibility
`radiogroup` with one tab stop; each card labelled by its label and described by its description (or the disabled reason); selection shown by ring and dot, not colour.

## 9. Props sketch
```rust
RadioCards { options: Vec<RadioCard>, value: Option<String>, layout: "grid"|"list", min_column: f32, label,
            name: Option<String>, disabled: bool, on_change: EventHandler<String> }
```

## 10. Related components and references
`composer-permission-selector.md`, `toggles.md`. Arc UI Radio cards — behaviour only.
