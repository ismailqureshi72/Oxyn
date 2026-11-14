**Status:** v1 — implemented (plan #25).

# Spec: OTP input

> Implemented in `oxyn-core::primitives::otp` (`Otp`, `OtpStatus`, `OtpKey`, `timing`),
> `oxyn-dioxus::OtpInput`, `oxyn-gpui::OtpEl`. Source: affectUI OTP Input (behaviour only).

## 1. Purpose
The verification code after sign-in, a new device, or an agent asking to confirm a sensitive
action. The user types or pastes six digits and wants to know at once whether it worked.

## 2. State machine
```
Idle(empty) ─type digit─▶ Idle(cell i filled, ring → i+1)
Idle ─paste / autofill─▶ Idle(cells fill one after another)
Idle(full) ─▶ on_complete(code) ─host─▶ Verifying (locked, light passes)
Verifying ─wrong─▶ Error (shake, red edges, message, a try spent) ─420 ms─▶ cleared → Idle(empty)
Verifying ─right─▶ Success (boxes slide together → "Verified" pill)
Error ─type─▶ Idle (message clears)
attempts = 0 ─▶ Locked ("No tries left")                          Oxyn plus
Resend: Countdown(30…1) ─0─▶ Resend button ─click─▶ on_resend, Countdown(30)
Disabled: no input.
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | boxes, ring on the active box, blinking caret | digits, ⌫, ← →, Home, End, paste | input "Verification code, 6 digits" |
| filled box | digit pops in | typing replaces | — |
| verifying | sheen across the boxes, muted digits | locked | status "Verifying code" |
| error | shake, danger edges, message, tries left | typing resets | `aria-invalid=true`, message linked |
| success | "Verified" pill with drawn check | — | status "Verified" |
| resend countdown | "Resend in 0:24" | — | — |
| resend | "Resend" link | Enter | button |
| masked (Oxyn plus) | dots | — | — |
| locked (Oxyn plus) | "No tries left" | — | announced |

## 4. Micro-interactions
One focus ring glides from box to box rather than each box drawing its own. Pasted digits land 40
ms apart. The error shake decays over three swings; digits fade then clear. Success: boxes slide
together and the frame morphs into a pill; the check draws.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Ring glide | 214 ms `spring.snap` | spring | jump |
| Digit in | 140 ms, stagger 40 ms on paste | `ease.out` | instant |
| Verifying sheen | 1.1 s loop | linear | static muted |
| Error shake | 420 ms, 6 → 0 px | damped sine | none |
| Clear after error | 420 ms | — | 420 ms |
| Success morph | 360 ms + 260 ms check | `ease.out` | crossfade |

## 6. Variants
Free: 4–8 boxes, separator (middle or after N), digits only, start / centre alignment, statuses,
resend countdown, disabled.

**Oxyn plus:** letters and digits (shown in capitals), masked display, attempt counter that locks
after the last try, local `expect` for demos.

## 7. Edge cases
Paste with spaces or dashes is cleaned. A paste longer than the field fills from the first box.
Clicking a box past the next empty one focuses the next empty one. Narrow phones: boxes shrink to
their container (six + separator fit 320 px). 16 px input text so iOS doesn't zoom. Slow verify:
the sheen continues; nothing can be typed.

## 8. Accessibility
One real `input` holds the whole code (`autocomplete=one-time-code`, `inputmode=numeric`), so SMS
autofill, paste and screen readers work; the boxes are `aria-hidden`. The name includes the length.
Verifying, errors (with tries left), success and resend are announced in a polite region. Colour is
never the only signal: errors also shake and show text.

## 9. Props sketch
```rust
OtpInput { length: usize, separator: Option<usize>, numeric, value, status: "idle" | "verifying" |
           "error" | "success", error_message, clear_on_error, success_label, resend_in: Option<u32>,
           align, disabled, auto_focus, label, masked, attempts: Option<u32>, expect: Option<String>,
           on_change, on_complete: EventHandler<String>, on_resend }
```

## 10. Related components and references
`text-field.md`, `password-field.md`, `approval-card.md` (step-up confirmation). affectUI
`/components/otp-input` — behaviour only.
