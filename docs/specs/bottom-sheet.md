**Status:** v1 — implemented (plan #26).

# Spec: bottom sheet

> Implemented in `oxyn-core::primitives::modal` (`BottomSheet`, `DetentKey`, `SheetRelease`, `rubber`), `oxyn-dioxus::BottomSheet`, `oxyn-gpui::BottomSheetEl`.
> Source: Arc UI Bottom sheet (behaviour only, no source used).

## 1. Purpose
Filters, share targets and quick actions on a phone: a sheet that rests at a peek height and can be pulled to full height or flicked away with one thumb.

## 2. State machine
```
Closed ─trigger─▶ Open(detent = initial)
Open ─drag─▶ Dragging(height follows 1:1; past the tallest detent rubber-bands)
Dragging ─release─▶ project(height − velocity × 0.2) → nearest detent, or Dismiss when below half the smallest
Open ─grabber ↑ ↓ Home End Enter─▶ Open(detent')
Open ─Esc / scrim / ×─▶ Closed
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| resting | sheet at a detent, scrim dim ∝ height | ↑ ↓ on the grabber | `dialog`, grabber `button` with `aria-expanded` |
| dragging | height follows the finger | — | — |
| over-drag | rubber-banded past the top | — | — |
| expanded | tallest detent | Enter collapses | label "Collapse sheet" |
| closed | trigger only | — | — |

## 4. Micro-interactions
The sheet follows the finger with no lag; past the tallest detent the pull resists (rubber band, 0.55 coefficient). On release the height springs to the projected detent. The backdrop dim tracks the live height, so it fades as the sheet is dragged away.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Open | 360 ms | `ease.out` | 120 ms fade |
| Settle on a detent | 380 ms | cubic-bezier(.32,.72,0,1) | instant |
| Dismiss | 220 ms | `ease.out` | instant |

## 6. Variants
Free: two detents (45 % / 92 %), single detent, opens at full height.

**Oxyn plus:** three detents, detent change events and spoken announcements, `contained` for previews.

## 7. Edge cases
Detents are fractions of the container (or viewport) height and are sorted. A fast flick down from the smallest detent dismisses. Content scrolls inside the sheet; dragging starts from the grabber or header so scrolling still works.

## 8. Accessibility
A modal dialog named by its title. The grabber is a real button: ↑ ↓ step detents, Home / End jump, Enter / Space toggles; ↓ or Page Down on the sheet expands first. Each settle is announced ("Sheet at 45 percent").

## 9. Props sketch
```rust
BottomSheet { title, description, open, trigger_label, detents: Vec<f32>, initial_detent: usize,
              close_label, contained, children, on_open_change: EventHandler<bool>,
              on_detent_change: EventHandler<usize> }
```

## 10. Related components and references
`sheet.md`, `sheet-stack.md`. Arc UI `/docs/components/bottom-sheet` — behaviour only.
