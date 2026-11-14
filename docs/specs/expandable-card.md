**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Expandable card (`expandable-card`)

Family: Surfaces. Closes the parity row(s): Arc UI · Expandable card.

## 1. Purpose
A dense card that gives itself more room on request: a header button with an icon, title and a one-line description (a changed description rises in), and a body that opens. Width (`width` → `expanded_width`) and height morph together; details fade in after the box has made room and fade out before it shrinks; the card centres and never exceeds its container (0 fills). Escape closes from inside.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| collapsed | Header only |  |
| expanded | Body visible, wider | Esc / header → collapsed |

## 3. Timing and motion
- Width and grid rows move over 320 ms on cubic-bezier(.2,.8,.2,1); details fade in 160 ms after.
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| collapsed | Default |
| open | Started open |
| fill | Fills |
| failed | New description |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space | Toggle |
| Escape | Close from inside |

## 6. Accessibility
- Header button with aria-expanded; the closed body is aria-hidden.

## 7. Responsive
- Width caps are max-width 100 %.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| title / description | String | — |  |
| icon | String | tool |  |
| expanded | bool | — |  |
| width / expanded_width | u32 | 0 | px; 0 fills / same as width |
| on_toggle | EventHandler<bool> | — |  |
| children | Element | — | details |

## 9. Edge cases
- Toggling mid-morph reverses from the current size (CSS transitions).

## 10. Implementation
- Core: — (`oxyn_core::parity::tools`), with tests.
- Dioxus: `ExpandableCard` (parity/tools.rs, tools.css).
- GPUI: `ExpandableCardEl` (`toggle`).

## 11. References
- Arc UI Expandable card (https://uiarc.dev/components/expandable-card) — behaviour-only rebuild, no source copied.
