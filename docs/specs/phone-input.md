**Status:** v1 — implemented (plan #25).

# Spec: phone input

> Implemented in `oxyn-core::primitives::phone` (`PhoneInput`, `Country`, `COUNTRIES`,
> `CountryRow`, `PhoneStatus`, `parse`, `format_national`, `country_from_locale`),
> `oxyn-dioxus::PhoneInput`, `oxyn-gpui::PhoneInputEl`. Source: Arc UI Phone input (behaviour only;
> the country table is Oxyn's own, from public ITU calling codes).

## 1. Purpose
A number for SMS codes, a contact card, a voice agent's callback. The user types it the way they
know it, and the app needs E.164.

## 2. State machine
```
Empty ─type digits─▶ Incomplete ─…─▶ Valid (check) ─more─▶ TooLong
any ─type "+"─▶ Picker open with search "+"
any ─paste "+44 (0)7911…" / "0044…"─▶ country switches, digits set, announced
Country button ─click / ↓ / ↑ / letter─▶ Picker(search seeded, active = current country)
Picker ─type─▶ filtered (names, ISO, +code) ; ─↑ ↓ PgUp PgDn Home End─▶ active'
Picker ─Enter / click─▶ country set (flag rolls), back to the number
Picker ─Esc─▶ closed
blur ─wrong length & validate─▶ message "Too short for a United Kingdom number"
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| empty | flag, +code, example placeholder | `+` opens picker | `type=tel`, `autocomplete=tel` |
| typing | formatted national number | — | — |
| valid | check | — | status "Valid United States number" |
| invalid after blur | danger edge, message | — | `role=alert`, `aria-invalid` |
| picker | search + Suggested / All countries | ↑ ↓ PgUp PgDn Home End Enter Esc | combobox + listbox |
| no match | "No countries match" | — | — |
| masked (Oxyn plus) | `•••• 0132` when not focused | — | — |
| disabled | 50 % | — | `disabled` |

## 4. Micro-interactions
The picker grows out of the flag button. On a country change the flag and code roll — down the
list rolls up, up the list rolls down. Separators appear as you type; Backspace over a separator
removes the digit before it.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Picker grow | 220 ms from top-left | `ease.out` | 80 ms fade |
| Flag / code roll | 200 ms | `ease.out` | swap |
| Valid check | 220 ms pop | `ease.out` | instant |
| Message in | 180 ms | `ease.out` | swap |

## 6. Variants
Free: default country, preferred countries, allowed countries, description, custom error,
validate after blur, required, hidden label, form `name` (E.164 hidden input).

**Oxyn plus:** masked display, country from the user's locale (`en-GB` → United Kingdom),
international format helper (`+44 7400 123456`).

## 7. Edge cases
Trunk prefixes (`0`) are kept as typed but dropped in E.164. `+1` resolves Canada by area code.
Digits past the pattern are kept, never dropped. 15-digit cap (E.164). Pastes with letters keep the
digits. Narrow screens: the list is as wide as the field, up to 340 px. `inputmode=tel` opens the
keypad.

## 8. Accessibility
The country button reads "Country, United Kingdom +44" with `aria-haspopup=listbox` and
`aria-expanded`. The search is a combobox with `aria-activedescendant`; the list is grouped. Picks,
pasted country changes, result counts and validity are announced politely. Errors show after blur
only, as `role=alert`.

## 9. Props sketch
```rust
PhoneInput { label, hide_label, value /* E.164 */, default_country, locale, countries: Vec<String>,
             preferred: Vec<String>, description, error, validate, masked, open, required, name,
             disabled, on_change: EventHandler<String>, on_country_change: EventHandler<String> }
```

## 10. Related components and references
`text-field.md`, `combobox.md` (picker pattern), `smart-form-fill.md` (phone extraction). Arc
`/components/phone-input` — behaviour only. ITU-T E.164.
