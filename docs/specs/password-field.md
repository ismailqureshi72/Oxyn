**Status:** v1 — implemented (plan #25).

# Spec: password field

> Implemented in `oxyn-core::primitives::password` (`caps_lock_from`, `slash_draw`) on top of the
> text field's reveal, `oxyn-dioxus::PasswordField`, `oxyn-gpui::PasswordFieldEl`. Source: Arc UI
> Password field (behaviour only, no source used).

## 1. Purpose
Sign in, unlock a vault, paste an API secret. The user wants to check what they typed without
leaving it on screen and to know when Caps Lock is fighting them.

## 2. State machine
```
Hidden ─toggle─▶ Revealed (slash undraws, value resolves in) ─toggle─▶ Hidden (slash draws)
Revealed ─reveal_for elapses─▶ Hidden                               Oxyn plus
any ─key with Caps Lock on─▶ CapsWarning (replaces the description) ─blur / Caps off─▶ normal
any ─error─▶ Invalid (message replaces the description)
vault ─click─▶ on_vault (host fills)                                 Oxyn plus
Disabled: no toggle, no input.
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| hidden | dots, eye with slash | Tab to the toggle | toggle "Show password", `aria-pressed=false` |
| revealed | text, eye without slash | Enter / Space toggles | "Hide password", `aria-pressed=true` |
| caps lock | "↑ Caps Lock is on" in warning | — | linked by `aria-describedby` |
| error | danger edge, message | — | `aria-invalid=true` |
| timed reveal (Oxyn plus) | hides again after N s | — | — |
| vault (Oxyn plus) | key button before the eye | Enter | "Use saved password" |
| disabled | 50 % | — | `disabled` |

## 4. Micro-interactions
The eye never swaps icons: a single slash draws across it (with a background-coloured mask so it
reads on any surface) and undraws when revealed. The value resolves through a short fade on each
toggle so the switch between dots and text isn't a jump.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Slash draw / undraw | 180 ms | `ease.out` (cubic) | instant |
| Value resolve | 160 ms | `ease.out` | instant |
| Caps warning in | 180 ms | `ease.out` | swap |
| Timed hide (Oxyn plus) | `reveal_for` s | — | same |

## 6. Variants
Free: label, description, placeholder, `current-password` / `new-password`, required, error,
custom toggle labels, disabled.

**Oxyn plus:** Caps Lock warning, reveal-for-N-seconds, "Use saved password" vault button.

## 7. Edge cases
Very long secrets scroll inside the input; the toggle never wraps. Password managers keep working
(real `type=password` input, `autocomplete`). Caps detection uses the typed letter and Shift, so
it works where `getModifierState` is missing. RTL: the toggle sits at the inline end.

## 8. Accessibility
Native `button` with `aria-pressed`, `aria-controls` and a label that flips between Show and Hide
password. The description or Caps warning is linked through `aria-describedby`. Never announce the
password itself.

## 9. Props sketch
```rust
PasswordField { label, description, value, placeholder, autocomplete, name, required, error,
                show_label, hide_label, caps_warning, reveal_for: Option<u32>, vault, disabled,
                on_input: EventHandler<String>, on_vault }
```

## 10. Related components and references
`password-strength.md` (new passwords), `text-field.md`, `otp-input.md`. Arc
`/components/password-field` — behaviour only.
