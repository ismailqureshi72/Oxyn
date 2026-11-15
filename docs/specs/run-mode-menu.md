**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Run mode menu (`run-mode-menu`)

Family: Composer ([composer-mode-switch.md](composer-modes.md)). Closes the parity row(s): X #7 @naklecha Cloudy (mode menu and time budget). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
The composer chip that sets how much the agent may do: Ask (answer, change nothing), Plan (propose, run nothing), Work (talk and run code) and Autoresearch (works on its own toward a goal). Each mode has its own shape, so the chip reads without colour. Autoresearch opens a budget row with 1h, 5h, 10h and 24h presets and a minutes field (`parse_budget`, clamped 5 … 10 080); the chip then shows the budget (`hm`).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| closed | Shape, mode, budget for Autoresearch | click / Enter |
| open | Menu with the active row | pick / Esc / outside |

## 3. Timing and motion
- Menu pops 180 ms from its corner; the label swaps with a 220 ms rise.
- Reduced motion: no pop or rise.

## 4. Variants
| Variant | What differs |
|---|---|
| work | Work mode |
| autoresearch | Autoresearch with 10h |

## 5. Keyboard
| Key | Does |
|---|---|
| ↑ / ↓ | Move |
| Enter | Pick; set the typed minutes |
| Esc | Close |

## 6. Accessibility
- The chip has aria-haspopup=menu and aria-expanded; modes are menuitemradio with aria-checked; the budget is a labelled group.

## 7. Responsive
- Opens upwards by default (bottom composers).

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| mode | ask · plan · work · auto | work |  |
| budget | u32 | 1440 | minutes |
| up | bool | true |  |
| on_change | EventHandler<(String, u32)> | — |  |

## 9. Gaps against the source
- The "starts on a CPU, adds GPUs" compute line and the long-run summary panes are product content, not built.

## 10. GPUI
RunModeMenuEl renders the open menu with the checked mode and the budget.
