**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Model menu (`model-menu`)

Family: Composer ([composer-model-picker.md](composer-model-picker.md)). Closes the parity row(s): affectUI · Model Picker. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
The model menu of an AI chat. Pointing at a model (or arrowing onto it) fills its speed, quality and cost bars (1–5, staggered); a router with no strengths says Varies. Extended thinking is a switch; turning it on unfolds the effort control (Low, Medium, High) on a height spring. Web search is a switch. The tray rolls the pointed model's first-token time and price per 1M. Opens below or above (`side: top` for a composer at the bottom) and lines up with the start or end edge of the trigger; the trigger shows the effort while thinking is on.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| closed | Chip | click / ↓ |
| open | List, switches, tray | pick / Esc / outside |

## 3. Timing and motion
- Bars fill 300 ms with 40 ms stagger; unfold 280 ms; tray roll 220 ms.
- Reduced motion drops the fill and roll.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Closed |
| thinking | Effort unfolded |
| top | Opens upward |

## 5. Keyboard
| Key | Does |
|---|---|
| ↓ | Open, move |
| ↑ | Move |
| Enter | Pick |
| Esc | Close |

## 6. Accessibility
- role=listbox with aria-activedescendant; switches are role=switch.

## 7. Responsive
- 300 px menu.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| models | Vec<MenuModel> | — | id, name, description, speed, quality, cost, latency_ms, price_per_m |
| value | String | first |  |
| thinking / web_search | bool | false |  |
| effort | low · medium · high | medium |  |
| align | start · end | start |  |
| side | bottom · top | bottom |  |
| on_change / on_option | EventHandler | — |  |

## 9. Gaps against the source
- Model logos are left to the host (text names).

## 10. GPUI
ModelMenuEl renders rows, bars and the tray in GPUI.
