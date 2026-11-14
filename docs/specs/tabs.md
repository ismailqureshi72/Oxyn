**Status:** v1 — implemented (plan #27).

# Spec: tabs

> Implemented in `oxyn-core::primitives::layout (`Tabs`, `Tab`, `Activation`, `Travel`, `overflow_edges`)`, `oxyn-dioxus::Tabs`, `oxyn-gpui::TabsEl`.
> Source: Arc UI Tabs (behaviour only, rebuilt from the public docs; no source used). Terminal tabs follow `terminal-tabs.md`.

## 1. Purpose
Peer views of one object (Overview · Runs · Approvals · Settings), and the terminal strip of a coding agent where each tab is a shell that can be closed, added and keeps a busy dot while a command runs.

## 2. State machine
```
Idle(value) ─click / arrow (auto)─▶ Selected(new, travel = fwd|back) ─▶ panel slides in from travel side
Idle ─arrow (manual)─▶ Focused(i) ─Enter / Space─▶ Selected
Selected ─close(i)─▶ selects right neighbour (else left) ─last closed─▶ Empty
Any ─"+" (terminal)─▶ push + select
Strip overflow ─scroll─▶ edge buttons shown per side
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| selected | pill / underline under the tab, panel | ← → Home End | `tab` `aria-selected=true` |
| focused (manual) | focus ring only | Enter / Space selects | roving `tabindex` |
| disabled | 45 % opacity | skipped by arrows | `disabled` |
| busy (terminal) | pulsing green dot | — | label only |
| overflow | edge chevrons | — | “Scroll tabs left / right” |

## 4. Micro-interactions
One pill per instance glides between triggers (transform + width, measured with `onmounted`); keyboard moves jump. Panels re-key on every change and slide 8 px from the side the selection came from. Counts roll up when they change; the close button fades in on hover or selection.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Pill glide | 240 ms | `ease.out` | jump |
| Panel slide | 240 ms | `ease.out` | fade |
| Count roll | 220 ms | `ease.out` | swap |
| Close fade | 120 ms | linear | instant |

## 6. Variants
Free: `pill`, `underline`, `sm` / `md`, manual activation, full-width.

**Oxyn plus:** `enclosed` folder tabs, `terminal` (closable, busy dot, “+”, Delete closes), counts, icons, reorder (`Tabs::reorder`).

## 7. Edge cases
Many tabs scroll; focusing a tab keeps it in view. Closing the last terminal leaves an empty strip with only “+”. Disabled first tab: the tab stop moves to the first enabled tab. Labels never wrap; terminal labels ellipsize at 16ch.

## 8. Accessibility
`tablist` / `tab` / `tabpanel` with `aria-controls` and `aria-labelledby`; one tab stop. The outgoing panel is gone (re-keyed), so it can never take focus. Overflow buttons are `tabindex=-1` because the arrows already reach every tab.

## 9. Props sketch
```rust
Tabs { tabs: Vec<Tab>, value: Option<String>, panels: Vec<String>, variant: "pill"|"underline"|"enclosed"|"terminal",
       size: "sm"|"md", manual: bool, label: String, full: bool, addable: bool,
       on_change: EventHandler<String>, on_close: EventHandler<String>, children }
```

## 10. Related components and references
`segmented-control.md` (value toggle), `terminal-tabs.md`, `page-header.md`. Arc UI Tabs — behaviour only.
