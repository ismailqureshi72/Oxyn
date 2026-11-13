**Status:** v1 — implemented (plan #23).

# Spec: text field

> Implemented in `oxyn-core::primitives::text_field` (`TextField`, `FieldType`, `Validate`,
> `CountLevel`, validation, nudge, clear, reveal timer, copy, counter), `oxyn-dioxus::TextField`,
> `oxyn-gpui::TextFieldEl` (display; editing with the shared GPUI input handler). Sources: affectUI
> Text Field, Arc Input (behaviour only).

## 1. Purpose
One input for every form: repository, email, budget, API key, search, commit message.

## 2. State machine
```
Idle ─focus─▶ Focused ─type─▶ Focused(value)  ─blur─▶ Idle (validate: OnBlur)
Validation timing (Oxyn plus):
  OnBlur  : quiet while typing the first time; validates on blur; once an error shows it is live,
            so it clears the moment the input is fixed.
  OnType  : validates on every change after the first character.
  OnSubmit: only `submit()` validates.
Error(e) ─new e─▶ nudge once; message replaces the hint in the same slot.
Password: Hidden ⇄ Revealed (Oxyn plus: Revealed ─auto_hide─▶ Hidden).
Copy: Idle ─copy─▶ Copied (check, 1.5 s) ─▶ Idle.  Clear: ×, or Esc in the field.
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | grey edge, label above, hint below | Tab | `label for` |
| hover | edge darkens (no focus ring) | — | — |
| focus | edge darkens a little more | — | — |
| filled + clearable | × button | Esc clears (focus stays) | × named "Clear {label}" |
| error | danger edge, one nudge, message in the hint slot | — | `aria-invalid=true`, described by the message (polite live) |
| prefix / suffix | "$", "/ month", or an icon | — | part of the visual only |
| password hidden / shown | ••• / text out of a short blur; eye ⇄ eye-off; a ring counts down the auto-hide | — | eye named "Show / Hide password", `aria-pressed` |
| copy | copy icon morphs into a check that draws itself | — | named "Copy {label}" |
| counter | "49 / 50"; warning past the soft limit, danger past the hard limit | — | polite |
| disabled / read-only | 50 % / normal text, no edits | — | `disabled` / `readonly` |

## 4. Micro-interactions
No focus ring: the grey edge darkens on hover and a little more on focus (affectUI). An error nudges
the field once (4 → 1 px over 300 ms) and its message slides in out of a short blur in place of the
hint — nothing below moves. Showing a password brings the characters out of a short blur. Arc: the
helper / error row opens its height on a smooth spring; changed words rise in; numbers roll.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Edge colour | 120 ms | `ease.out` | 80 ms |
| Error nudge | 300 ms (`dur.error.nudge`) | damped sine | none |
| Message in | 180 ms, 5 px rise, blur 3 → 0 | `ease.out` | 80 ms fade |
| Reveal | 180 ms blur → sharp | `ease.out` | instant |
| Copy confirm | 1500 ms (`dur.copy.confirm`) | — | same |
| Check draw | 240 ms | `ease.out` | drawn |
| Auto-hide (Oxyn plus) | 10 s default | linear ring | ring hidden, timer kept |

## 6. Variants
Source: `label`, `hint`, `error`, `prefix`, `suffix`, `size` sm / md / lg, `optional`, `clearable`,
`copyable`, `counter`, `type`, `onClear`; Arc `description`, `error`.

**Oxyn plus:** validation timing (blur / type / submit), soft limit on the counter, secret field whose
reveal hides itself again (with a countdown ring), built-in checks for email / URL / number / required.

## 7. Edge cases
Pasting over the hard limit: the counter goes red and the error explains; the value is kept (no
silent truncation) unless `maxlength` is set without a soft limit. Clearing a field that showed an
error re-validates. Long prefixes shrink the input, never overflow. Disabled fields ignore input in
the model as well as the DOM.

## 8. Accessibility
Native `<input>` with a `<label for>`; `aria-describedby` always points at the message slot, which is
`aria-live=polite`; `aria-invalid` and `aria-required` reflect the model. Icon buttons inside the field
are named and reachable after the input in tab order.

## 9. Props sketch
```rust
TextField { label, value, kind: "text|password|email|search|number|url", placeholder, hint, error,
            prefix: "$" | "icon:search", suffix, size, optional, required, clearable, copyable,
            counter, max, soft, validate: "blur|type|submit", auto_hide_s, disabled, now,
            on_input, on_clear, on_copy }
```

## 10. Related components and references
`select.md` (shared field chrome), `composer-input-shell.md` (multi-line composer), `genui-forms.md`.
affectUI `/components/text-field`; Arc `/components/input` — behaviour only.
