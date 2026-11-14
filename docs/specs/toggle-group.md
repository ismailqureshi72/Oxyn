**Status:** v1 — implemented (plan #27).

# Spec: toggle group

> Implemented in `oxyn-core::primitives::layout (`ToggleGroup`, `ToggleItem`)`, `oxyn-dioxus::ToggleGroup`, `oxyn-gpui::ToggleGroupEl`.
> Source: Arc UI Toggle group (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Formatting toolbars, view toggles, tool sets in the composer.

## 2. State machine
```
single: Off ─press─▶ On (others off); pressing the on item keeps it
multiple: each item toggles; neighbours that are both on join
Arrows move focus only (toolbar)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| off | muted icon | Space / Enter | `aria-checked=false` / `aria-pressed=false` |
| on | filled | same | `true` |
| joined (multiple) | square inner corners | — | — |
| disabled | 40 % opacity | skipped | `disabled` |

## 4. Micro-interactions
Single: one fill glides to the chosen item. Multiple: each fill fades in from 86 % scale and pressed neighbours square their inner corners into one run. The icon dips to 92 % on press; the item box never scales.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Glide | 240 ms | `ease.out` | jump |
| Fill in | 180 ms + 220 ms scale | `ease.out` | instant |
| Press dip | 160 ms | `ease.out` | none |

## 6. Variants
Free: single, multiple, `sm` / `md`, icon or text items.

**Oxyn plus:** joined runs in multiple mode, mixed icon + text, disabled group.

## 7. Edge cases
Icon-only items require `label` (tooltip and name). The strip never wraps; put groups in a wrapping row.

## 8. Accessibility
Single → `radiogroup` / `radio` / `aria-checked`; multiple → `group` / `aria-pressed`; roving tab stop; fills `aria-hidden`.

## 9. Props sketch
```rust
ToggleGroup { items: Vec<ToggleItem>, label: String, multiple: bool, value: Vec<String>, size: "sm"|"md",
             disabled: bool, on_change: EventHandler<Vec<String>> }
```

## 10. Related components and references
`segmented-control.md`, `composer-tools-menu.md`. Arc UI Toggle group — behaviour only.
