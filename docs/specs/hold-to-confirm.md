**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Hold to confirm (`hold-to-confirm`)

Family: G Approvals ([approvals.md](approvals.md)). Closes the parity row(s): Arc UI · Hold to confirm.

## 1. Purpose
Confirm a destructive action by holding, not tapping. A tap does nothing; holding fills the button over `duration`; letting go early rewinds the fill on a smooth curve, and pressing again during the rewind resumes from where the fill is instead of from zero. When the fill completes the label changes, the width springs and a check draws itself in.

Anatomy:
```
[ ⌫ Hold to delete ▓▓▓▓▓░░░ ]
      Keep holding…            ← hint (polite live region)
```

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Outline button, icon, label | press → holding |
| holding | Fill grows linearly; button scaled 0.97 | release <100 % → rewinding; fill = 100 % → done |
| rewinding | Fill shrinks over 240 ms (smoothstep) | press → holding (resumes); 0 % → rest |
| done | Check drawn, confirmed label, danger outline | host sets confirmed = false → rest |

## 3. Timing and motion
- Fill: linear, `duration` (1200 ms default, `motion.md` hold-to-confirm token).
- Rewind: 240 ms smoothstep from the release fill; re-press keeps the current fill as its base.
- Press scale 0.97; check stroke draws in 320 ms.
- Reduced motion: no press scale, no rewind animation (the fill still tracks the hold), check appears without drawing.

## 4. Variants
| Variant | What differs |
|---|---|
| danger | Red fill (default) |
| neutral | Foreground fill for reversible actions |
| long | 2 s hold, large size |
| bare | Small, no hint line |

## 5. Keyboard
| Key | Does |
|---|---|
| Space / Enter (held) | Fills; key-up early rewinds |
| Tab away (blur) | Cancels an active hold |

## 6. Accessibility
- Native button named “<label>, hold to confirm”, described by the hint.
- The hint is a polite live region: “Hold to confirm” → “Keep holding…” → “Done”.
- Auto-repeat keydown events are ignored so holding a key does not restart the hold.

## 7. Responsive
- Inline size; `touch-action: none` so the hold never scrolls the page; three sizes (sm 28, md 34, lg 42 px).

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | HoldConfirm | — | duration_ms, tone, confirmed |
| now | ClockMs | — | shared clock |
| label | String | Hold to delete |  |
| confirmed_label | String | Done |  |
| icon | String | trash | Icon name |
| size | sm·md·lg | md |  |
| hint | bool | true | hint line |
| on_hold | EventHandler<bool> | — | down / up |

## 9. Edge cases
- Pointer leaves the button while held → treated as release.
- `confirmed` reset by the host returns to rest with fill 0.
- Hold longer than duration completes on the frame the fill reaches 1 (`tick`).

## 10. Implementation
- Core: `oxyn_core::parity::confirm::HoldConfirm` (press, release, tick, fill, hint, reset) with tests.
- Dioxus: `HoldToConfirm` (parity/approvals.rs, approvals.css).
- GPUI: `HoldConfirmEl` reports `down` / `up`. GPUI reports button presses through one `Act` callback with an action id; the host routes them to the model.

## 11. References
- Arc UI Hold to confirm (https://uiarc.dev/components/hold-to-confirm) — behaviour-only rebuild, no source copied.
