**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Model pill (`model-pill`)

Family: Composer ([composer-model-picker.md](composer-model-picker.md)). Closes the parity row(s): Arc UI · Model picker (block). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A model picker in one quiet pill. Open it to choose from featured models (the first four when none are given) with a one-line note each; Effort opens a submenu listing every level, with the ones the model does not accept disabled; More models lists every other model grouped by provider. Picking a model whose effort it does not accept snaps the effort to the nearest level it does (`snap`; ties go lower) and reports it. The pill's mark and name crossfade to the choice. The summary under the pill shows description, provider, context, input / output price per 1M and effort, and a short confirmation ("Switched to …") for about two seconds; `show_summary: false` leaves the pill alone for a composer toolbar.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| closed | Pill (+ summary) | click / Enter / ↓ |
| main | Featured, Effort, More | → / pick / Esc |
| effort / more | Submenu page | ← / pick / Esc |

## 3. Timing and motion
- Menu 160 ms; pages slide 180 ms; crossfades 200 ms.
- Reduced motion swaps everything in place.

## 4. Variants
| Variant | What differs |
|---|---|
| default | With summary |
| pill | Pill only |
| three | Three featured |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space / ↓ | Open |
| ↑ / ↓ | Move |
| Letters | Jump to a model |
| → / ← | Open / close a submenu |
| Esc | Close or back out |

## 6. Accessibility
- Menu items are menuitemradio with aria-checked; the confirmation is role=status.

## 7. Responsive
- Up to 360 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| models | Vec<PillModel> | — | id, name, provider, blurb, description, context, price, efforts, default_effort |
| providers | Vec<(String, String)> | — |  |
| featured | Vec<String> | first four |  |
| show_summary | bool | true |  |
| now | ClockMs | — | confirmation timing |
| on_change / on_effort | EventHandler<String> | — |  |

## 9. Gaps against the source
- Phone page navigation is the same submenu page (no separate mobile layout).

## 10. GPUI
ModelPillEl renders the pill and summary in GPUI.
