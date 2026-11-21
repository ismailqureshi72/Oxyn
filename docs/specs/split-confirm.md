**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Split confirm (`split-confirm`)

Page `/components/buttons/split-confirm/` (family buttons). Reference: an X post the founders saved (R06 (stretch pill) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `SplitConfirm` (Dioxus, `oxyn_dioxus::xpost`, `xpost/trust.rs`) · `XPostView` with `XPostVariant::SplitConfirm` (GPUI), rules in `oxyn_core::xpost::split`.

## 1. Purpose
Confirm an important action with a moment of weight: the button stretches and a check pinches off, which doubles as Undo.

## 2. States
Idle · Stretch · Split (bubble pinches off) · Done (past tense, bubble is Undo for 4 s) · Failed (cross bubble, one shake, Try again).

## 3. Micro-interactions
The pill stretches with a small overshoot from its left edge; the bubble grows out of its right end and settles 6 px away with a springy pop; the label crossfades to the past tense.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Stretch | 260 ms (STRETCH_MS) | sine overshoot |
| Split | 420 ms (SPLIT_MS) | ease-out cubic |
| Bubble pop | 320 ms | spring |
| Undo window | 4000 ms (UNDO_MS) | — |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `approve` | Approve → Approved. |
| `send-large` | Send at the large size (Oxyn extra). |
| `fails` | Merge that fails and offers Try again. |

## 6. Responsive and edge cases
Fixed size (md 40 px, lg 52 px); never wraps. Label pairs: Approve/Approved, Send/Sent, Merge/Merged. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The pill is a real button; the bubble is a named Undo button while it can undo; results are announced; Backspace or Cmd/Ctrl+Z undo. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::SplitConfirm, ..)`; presses by id (sp-go, sp-undo, sp-retry, sp-reset), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
confirm-morph, hold-to-confirm, action-swap.
