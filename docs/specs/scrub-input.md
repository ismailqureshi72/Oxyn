**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Scrub input (`scrub-input`)

Family: Composer & inputs ([composer-effort-control.md](composer-effort-control.md)). Closes the parity row(s): Arc UI · Scrub input. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A Figma-style number field. Drag the label sideways to scrub (`px_per_step`, 2 px a step); Shift ten times coarser and snapped to the coarse grid, Alt ten times finer; changing gear mid-drag re-anchors so the value never jumps and a ×10 / ×0.1 badge shows. Typed text accepts units from `units` and arithmetic (`12*2`, `(1+2)*3`) through `parse_value` / `eval`. Escape restores typing or cancels a scrub back to where it started; double-click the label resets to `reset_value`. Past a limit the field strains with rubber-band resistance up to 12 px and springs back on release; invalid input and steps against a limit kick it (short shake). With both limits a quiet track fill shows where the value sits; `wrap` turns it into a rotation (360 → 0). More than one unit makes the unit a button that cycles them.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Label, value, unit | drag / type / keys |
| scrubbing | ew-resize cursor everywhere, gear badge | release / Esc |
| typing | Draft text | Enter / blur / Esc |
| kick | Shake 320 ms | time |

## 3. Timing and motion
- Strain: `rubber(over, 12)`; release springs back on a 300 ms overshooting curve.
- Track fill glides 200 ms to typed or stepped values and follows a drag one to one.
- Reduced motion removes the strain, shake and glide; units swap with a fade.

## 4. Variants
| Variant | What differs |
|---|---|
| size | W with px / % / rem and X without limits |
| rotation | Wraps at 360 |
| opacity | Percent with a track; double-click resets |

## 5. Keyboard
| Key | Does |
|---|---|
| ↑ / ↓ | Step (Shift ×10, Alt ×0.1) |
| Enter | Commit text, units and arithmetic |
| Esc | Restore while typing, cancel a scrub |

## 6. Accessibility
- The input is role=spinbutton; the label remains a <label> for the input.
- The unit button names the current unit.

## 7. Responsive
- 150 px default; sm is 26 px tall.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | String | — | drag handle |
| value / min / max / step | f64 | 0 / −∞ / ∞ / 1 |  |
| px_per_step | f32 | 2 |  |
| wrap | bool | false | needs both limits |
| units / unit | Vec<String> / String | — |  |
| reset_value | Option<f64> | — |  |
| track | bool | true | needs both limits |
| size | md · sm | md |  |
| now | ClockMs | — | kick timing |
| on_change / on_commit | EventHandler<(f64, String)> | — | every change / settled |

## 9. Gaps against the source
- Pointer lock is not used (Dioxus has no pointer-lock API without JS); a full-window drag layer keeps the cursor and events during a scrub instead.

## 10. GPUI
ScrubInputEl renders label, value, unit and track from `kit::fields::Scrub`; drag wiring is left to the host view.
