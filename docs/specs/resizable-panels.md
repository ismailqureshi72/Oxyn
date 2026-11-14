**Status:** v1 — implemented (plan #27).

# Spec: resizable panels

> Implemented in `oxyn-core::primitives::layout (`Resizable`, `Pane`, `sidebar_snap`)`, `oxyn-dioxus::ResizablePanels, PaneView`, `oxyn-gpui::ResizableEl`.
> Source: Arc UI Resizable panels (behaviour only, rebuilt from the public docs; no source used). The sidebar snap rule is Oxyn’s (affectUI app shell behaviour).

## 1. Purpose
Trade space between files, editor and agent panes; the same model drives the app shell’s resizable sidebar that snaps to icons.

## 2. State machine
```
Rest ─pointerdown─▶ Dragging(start sizes) ─move─▶ clamp [min, max] (rubber band)
Dragging ─past min − 48 px on a collapsible pane─▶ Collapsed ─drag out / Enter─▶ Restored
Dragging ─Escape─▶ Rest(start sizes)    Dragging ─up─▶ Rest (on_layout_change)
Divider ─← → (Shift)─▶ ±16 (64) px   Home / End ─▶ limits   double-click ─▶ reset
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| rest | 1 px line | arrows, Home End, Enter | `separator` value now/min/max |
| hover / drag | 2 px line | Escape cancels | — |
| squeezed | content fades | — | — |
| collapsed | restore tab | Enter restores | `inert` pane, valuetext “hidden” |

## 4. Micro-interactions
Dividers follow the pointer 1:1 (pointer capture) and clamp; a pane pushed well past its minimum snaps shut and leaves a small restore tab that pops in. Content fades as a pane is squeezed below its minimum.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Line hover | 120 ms | linear | instant |
| Restore tab | 260 ms | pop | fade |
| Squeeze fade | 120 ms | linear | instant |

## 6. Variants
Free: two or more panes, collapsible panes, `on_layout_change`.

**Oxyn plus:** `vertical`, `grip` handle, size readout, `sidebar_snap` (hidden · rail · open) for app shells.

## 7. Edge cases
Needs a definite height. Container resizes scale the panes. Keyboard limits never collapse (Home / End clamp). Panes do not stack on mobile; switch to tabs.

## 8. Accessibility
Each divider is a focusable `separator` with `aria-orientation`, `aria-controls`, `aria-valuenow/min/max` in percent and `aria-valuetext` (“Files 25 %”, or “Files hidden, press Enter to show”). Collapsed panes are `inert`.

## 9. Props sketch
```rust
ResizablePanels { panes: Vec<PaneView>, label: String, height: f32, vertical: bool, handle: "line"|"grip",
                  readout: bool, on_layout_change: EventHandler<Vec<f32>> }
Pane::new(id, label, default_size).min(px).max(px).collapsible()
```

## 10. Related components and references
`side-panels.md`, `app-shell.md`, `sidebar.md`. Arc UI Resizable panels — behaviour only.
