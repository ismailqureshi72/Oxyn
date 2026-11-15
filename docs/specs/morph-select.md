**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Morph select (`morph-select`)

Family: Composer & inputs ([composer-model-picker.md](composer-model-picker.md)). Closes the parity row(s): Arc UI · Morph select. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A select whose trigger surface grows into the list. The open surface starts clipped to the trigger's rectangle (radius 9) and springs to the full panel (radius 12) with a slight bounce, anchored to the start or end edge; it never gets narrower than the trigger. One highlight glides between options. Type-ahead jumps to a label (a repeated letter cycles; it picks while closed). Above eight options (`searchable: auto`) a search field takes the trigger's place when open. PageUp / PageDown move eight options, Home / End go to the ends (and open on them when closed), Alt+↑ picks and closes, Escape closes back to the trigger, Tab closes without picking. The chosen value rolls up when it is later in the list and down when earlier.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| closed | Trigger with value or placeholder | Enter / Space / arrows / click |
| open | Grown surface, highlight, list or search | pick / Esc / Tab / outside click |
| disabled | Dimmed trigger | — |

## 3. Timing and motion
- Grow: 340 ms clip-path morph (cubic-bezier .3,1.25,.5,1); highlight glide 160 ms; value roll 240 ms.
- Reduced motion jumps the size and uses a 120 ms fade; no roll.

## 4. Variants
| Variant | What differs |
|---|---|
| short | Four models, one disabled |
| long | Twelve time zones with search |
| end | Grows from the end edge |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space / ↓ / ↑ | Open on the selection; pick when open |
| Page Up / Down | Eight options |
| Home / End | Ends |
| Alt + ↑ | Pick and close |
| Letters | Type-ahead |
| Esc | Close to the trigger |
| Tab | Close without picking |

## 6. Accessibility
- Trigger: aria-haspopup=listbox, aria-expanded; list: role=listbox with aria-activedescendant; options: role=option, aria-selected, aria-disabled.
- A hidden input submits the value when `name` is set.

## 7. Responsive
- `panel_width` (272) and `max_list_height` (296, then it scrolls).

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| label / hide_label | String / bool | — / false |  |
| options | Vec<Choice> | — | value, label, hint, disabled |
| value | Option<String> | — |  |
| placeholder | String | Select |  |
| searchable | auto · on · off | auto |  |
| panel_width / max_list_height | f64 | 272 / 296 |  |
| align | start · end | start |  |
| name | Option<String> | — |  |
| now | ClockMs | — | type-ahead window |
| on_change | EventHandler<String> | — |  |

## 9. Gaps against the source
- The chosen label does not fly out of the list into the trigger; it rolls in place in the trigger instead.

## 10. GPUI
MorphSelectEl renders the trigger and open list (with highlight) from `parity::fields::MorphSelect`; the clip-path morph is web-only.
