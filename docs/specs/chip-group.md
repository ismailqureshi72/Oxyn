**Status:** v1 — implemented (plan #27).

# Spec: chip group

> Implemented in `oxyn-core::primitives::layout (`ChipGroup`)`, `oxyn-dioxus::ChipGroup`, `oxyn-gpui::ChipGroupEl`.
> Source: Arc UI Chip group (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Filter sources, topics or statuses; the mode chips under a composer.

## 2. State machine
```
Chip off ─press─▶ on (multiple adds; single replaces)   on ─press─▶ off (single can clear)
n > max_visible ─▶ “+N more” (selected always visible) ─press─▶ expanded
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| off | outline | Arrows, Enter | `aria-pressed=false` |
| on | check + strong border | same | `aria-pressed=true` |
| folded | +N more | Enter reveals | `aria-expanded` |

## 4. Micro-interactions
Selecting grows a check in and slides the label over; the chip edge springs. Revealed chips stagger in.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Check grow | 200 ms | `ease.out` | instant |
| Colour | 120 ms | linear | instant |

## 6. Variants
Free: multiple, single, folding.

**Oxyn plus:** counts, `soft` style, Clear chip.

## 7. Edge cases
Chips wrap; selected chips beyond `max_visible` stay in view so a filter is never hidden.

## 8. Accessibility
`group` + `aria-label`; chips are buttons with `aria-pressed`; roving focus wraps; checks are decorative.

## 9. Props sketch
```rust
ChipGroup { options: Vec<(String, String)>, label, value: Vec<String>, multiple: bool, max_visible: usize,
           counts: Vec<u32>, variant: "outline"|"soft", clearable: bool, on_change }
```

## 10. Related components and references
`composer-mode-switch.md`, `filter-menu.md`, `badges-chips.md`. Arc UI Chip group — behaviour only.
