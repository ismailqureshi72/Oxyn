**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Number field (`number-field`)

Family: Composer & inputs ([composer-effort-control.md](composer-effort-control.md)). Closes the parity row(s): Arc UI · Number field. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A bounded number with − and + buttons and a typed field. Holding a button repeats after 400 ms, ramping from 150 ms to 40 ms a step (`held_steps`). At a limit a press is refused: the value strains 2–8 px toward the press (further for pushes in a row, capped), the refused button shakes once and a note ("Max 10 seats", built from the prefix and suffix) rises beside the label for 1.5 s. A typed draft past a limit tints the shell with the warning role and springs back to the limit on Enter or blur; an unparseable draft reverts. Digits roll on wheels in the direction of change; the suffix pluralises (`suffix_one`). With `scrub` the label drags sideways (4 px a step).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Value, steppers | press / type |
| held | Value repeats on the clock | release |
| refused | Strain, shake, limit note (1.5 s) | time |
| draft | Typed text; warning tint when past a limit | Enter / blur commits, Esc reverts |
| disabled | Dimmed | — |

## 3. Timing and motion
- Hold repeat: first step on press, then 400 ms, then 150 ms × 0.82 per step down to 40 ms.
- Strain: 260 ms spring (cubic-bezier .3,1.6,.5,1) back from 2 + pushes px (max 8).
- Digits: 220 ms roll up or down per changed place.
- Reduced motion: values change at once; a refused press tints the number with the warning role instead of the strain and shake.

## 4. Variants
| Variant | What differs |
|---|---|
| seats | Plural suffix, limit note |
| price | Prefix, grouping, step 50 |
| ratio | Decimals from the step, scrub label |
| small | sm size, no note |

## 5. Keyboard
| Key | Does |
|---|---|
| ↑ / ↓ | Step (held keys repeat and keep pushing at a limit) |
| Shift + ↑ / ↓, Page Up / Down | Large step (`large_step`, default step × 10) |
| Home / End | Min / max when not typing |
| Enter | Commit the draft, clamped |
| Esc | Revert the draft |

## 6. Accessibility
- role=spinbutton with aria-valuemin/max/now and a valuetext with the affixes.
- Steppers are tabindex −1 (the keys cover them); the note is role=status.

## 7. Responsive
- Widths 164 / 196 / 228 px for sm / md / lg.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | String | — |  |
| value / min / max / step | f64 | 0 / 0 / 1e15 / 1 |  |
| large_step | Option<f64> | step × 10 |  |
| prefix / suffix / suffix_one | String | — | pluralises |
| description | Option<String> | — |  |
| size | sm · md · lg | md |  |
| scrub / limit_hint / grouping | bool | false / true / true |  |
| now | ClockMs | — | hold repeat, note timing |
| on_change | EventHandler<f64> | — | each committed change |

## 9. Gaps against the source
- Locale is fixed to en-US grouping; `formatOptions` beyond decimals and grouping is not offered.
- A function suffix (`n => …`) is covered by `suffix_one` for the singular form only.

## 10. GPUI
NumberFieldEl draws the shell, steppers, affixes and the limit note from `parity::fields::NumberField`; hold-to-repeat and the digit roll are not animated in GPUI yet.
