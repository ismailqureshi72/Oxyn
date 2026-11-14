**Status:** v1 — implemented (plan #27).

# Spec: reactions

> Implemented in `oxyn-core::primitives::chrome (`Reactions`, `ReactionOption`, `ReactionCount`)`, `oxyn-dioxus::Reactions`, `oxyn-gpui::ReactionsEl`.
> Source: Arc UI Reactions (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Acknowledge a message or an agent reply in one tap; teammates’ reactions on shared runs.

## 2. State machine
```
Pill(count, mine) ─press─▶ mine toggles, count ±1 ─count 0─▶ pill leaves
Add ─press─▶ Picker open ─arrows / pick─▶ toggle + close  ─Escape─▶ close, focus add
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| pill | glyph + count | Enter | `aria-pressed`, “Love, 4 reactions, including yours” |
| mine | strong border, fill | — | same |
| picker | toolbar tray | arrows, Escape | `toolbar` |

## 4. Micro-interactions
Pills pop in; counts roll; the picker grows out of the add button and options lift on hover.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Pill in | 260 ms | pop | fade |
| Count roll | 200 ms | `ease.out` | swap |
| Tray | 200 ms | `ease.out` | fade |

## 6. Variants
Free: bar + picker.

**Oxyn plus:** `compact` pills, picker that starts open inline.

## 7. Edge cases
A pill that leaves while focused hands focus back to the bar; zero-count pills never render.

## 8. Accessibility
`group` bar, labelled pills, `toolbar` picker with roving focus, polite status for each change.

## 9. Props sketch
```rust
Reactions { options: Vec<ReactionOption>, value: Vec<ReactionCount>, label, compact: bool, picker_open: bool,
           on_change: EventHandler<(String, bool)> }
```

## 10. Related components and references
`message-bubble.md` (feedback), `display-atoms.md` (ReactionBar). Arc UI Reactions — behaviour only.
