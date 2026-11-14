**Status:** v1 — implemented (plan #26).

# Spec: toaster

> Implemented in `oxyn-core::primitives::toast` (`Toaster`, `Toast`, `ToastKind`, `ToastPosition`, `Slot`), `oxyn-dioxus::Toaster` + `use_toasts`, `oxyn-gpui::ToasterEl`.
> Source: affectUI Toaster (behaviour only, no source used).

## 1. Purpose
Short-lived confirmations and background results: a draft saved, a file moved with Undo, an upload that resolves, a sync that failed. They must not cover the work or demand attention.

## 2. State machine
```
add ─▶ Shown (if fewer than `visible` live) | Waiting (timer paused)
Shown ─duration elapses (not hovered / focused)─▶ Closing(200 ms) ─▶ removed; next waiting one shows
Shown ─hover / focus / F8─▶ Expanded (timers paused) ─leave─▶ Collapsed (timers resume)
Loading (promise) ─resolve(ok)─▶ Success | Error (morph; timer starts)
Shown ─swipe past 45 % or fling─▶ Closing ; ─Esc on focused─▶ Closing, focus → next
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| collapsed | newest in front, older ones scaled 0.96ⁿ and peeking 10 px | F8 focuses | region "Notifications (F8)" |
| expanded | fanned list with `gap` | Tab through toasts | each toast focusable |
| loading | spinner arc | — | polite announcement |
| success / error | drawn check / cross | — | error announced assertively |
| waiting | hidden, timer paused | — | `aria-hidden` |

## 4. Micro-interactions
New toasts slide 16 px in from their edge; the stack fans out on hover with a spring. The hairline runs down with the timer and pauses with it. Promise toasts morph: the icon pops into a check or cross that draws in, and the copy rises out of a blur. Swiped toasts follow the finger and fade.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Enter | 320 ms, 16 px | `ease.out` | 120 ms fade |
| Fan out / in | 360 ms | `ease.out` | instant |
| Exit | 200 ms | `ease.out` | instant |
| Icon morph | 300 ms pop + 260 ms draw | `ease.out` | swap |
| Timer | `duration` (4 s) | linear | same |

## 6. Variants
Free: six positions, duration (0 = until dismissed), visible count, always-expanded, close button, action button.

**Oxyn plus:** promise toasts that morph, timer hairline, inline (contained) toaster, F8 hotkey, swipe velocity.

## 7. Edge cases
Long copy wraps to two lines. More than `visible` toasts queue with paused timers. Under 600 px the stack spans the width. A toast resolved while hovered starts its timer when the pointer leaves.

## 8. Accessibility
One labelled region; every toast is focusable with its title as name. Statuses are announced politely, errors assertively. Timers pause while the pointer or focus is inside, so nothing disappears mid-read. Close and swipe are never the only way out: Escape works too.

## 9. Props sketch
```rust
Toaster { position, inline, duration: u32, visible: usize, expand, offset: Option<f32>, gap,
          hotkey, hairline, close_button, label, seed: Vec<SeedToast>, children,
          on_action: EventHandler<(u64, String)> }
let t = use_toasts(); t.success(title, desc); let id = t.promise(title, desc); t.resolve(id, ok, title, desc);
```

## 10. Related components and references
`notification-stack.md`, `alert.md`, `checkpoint` restore toast. affectUI `/components/toaster` — behaviour only.
