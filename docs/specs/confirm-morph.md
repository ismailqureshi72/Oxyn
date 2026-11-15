**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Confirm morph (`confirm-morph`)

Family: G Approvals ([approvals.md](approvals.md)). Closes the parity row(s): Arc UI · Confirm morph.

## 1. Purpose
A destructive button that morphs in place: rest → inline question (Cancel / Confirm) → spinner with the pending label → result with Undo, or error with Retry. Confirmation can be a press, a hold on Confirm, or a countdown that Cancel or Escape can still stop. An unanswered question returns to rest after 6 s.

Anatomy:
```
[ ⌫ Delete 3 files ]  →  ( Delete 3 files?  Cancel [Delete] )  →  ( ◌ Deleting )  →  ( ✓ Deleted  ↶ Undo )
```

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Resting button | press → confirming |
| confirming | Question + Cancel + Confirm (or Hold to delete) | confirm → pending / counting; cancel, Esc, 6 s → idle |
| counting | Ring with seconds left, Cancel | 0 → pending; cancel → idle |
| pending | Spinner + pending label | resolve ok → done; fail → error |
| done | Drawn check + done label + Undo | undo → undoing; open → confirming |
| error | Alert + error label + Cancel + Retry | retry → pending; cancel → idle |
| undoing | Spinner + undoing label | undone → idle |

## 3. Timing and motion
- Width springs between faces (240 ms); the question fades in from a 3 px blur (200 ms).
- Countdown: one tick per second; hold fill as in hold-to-confirm (1000 ms default).
- Reduced motion: no width spring or blur; faces swap with a plain fade.

## 4. Variants
| Variant | What differs |
|---|---|
| press | Confirm on press (default) |
| hold | Hold Confirm while it fills |
| countdown | 3 s countdown after confirm |
| error | Failure face with Retry |
| done | Result with Undo |
| neutral | Foreground confirm for reversible actions |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter | Confirm |
| Escape | Cancel / stop countdown |
| Tab | Through the face's buttons |

## 6. Accessibility
- role=group named by the resting label; each face keeps a real button so focus is never lost.
- Face changes are announced in a polite live region (“pending: Deleting”).

## 7. Responsive
- Inline; the question never wraps (white-space: nowrap).

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | ConfirmMorph | — | labels, tone, mode, countdown, confirm_timeout_ms, undoable |
| now | ClockMs | — |  |
| icon | String | trash |  |
| on_open / on_confirm / on_cancel / on_undo | EventHandler<()> | — |  |
| on_hold | EventHandler<bool> | — | hold mode |

## 9. Edge cases
- Retry from error runs the action again (confirm returns true).
- Countdown cancelled at 1 s never starts the action.
- Opening again from done starts a fresh question.

## 10. Implementation
- Core: `ConfirmMorph`, `MorphFace`, `ConfirmMode`, `MorphLabels` with tests.
- Dioxus: `ConfirmMorph`.
- GPUI: `ConfirmMorphEl` (`open`, `confirm`, `cancel`, `undo`, `down`, `up`). GPUI reports button presses through one `Act` callback with an action id; the host routes them to the model.

## 11. References
- Arc UI Confirm morph (https://uiarc.dev/components/confirm-morph) — behaviour-only rebuild.
