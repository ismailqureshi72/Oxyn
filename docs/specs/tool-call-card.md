**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Tool call card (`tool-call-card`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Tool Call Card.

## 1. Purpose
An agent's tool call in one line: a tile with the kind icon, the tool name in mono, a summary that truncates, a live timer while running (from `started`) or the final duration, and the status (spinner, check, Failed). The header folds the card open to arguments and result, pretty-printed and lightly tinted (keys, strings, numbers, literals) with copy buttons; JSON strings are parsed, other text stays plain; while running without a result it reads “Waiting for result…”. Code is capped at `max_code` px with a fade and Show all. A failed call opens itself (unless toggled by hand) and shows the error with Retry; a retry closes the auto-opened card. `ToolCallGroup` sums its cards: “Used 4 tools · 2.4s · 1 failed”, or “Using tools… · 2 of 4 done” shimmering while any runs, with the first four icons stacked.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| running | Spinner, live timer, Waiting for result… | → success / error |
| success | Check + duration, result |  |
| error | Failed, error + Retry, opened unless touched | retry → running (auto-open closes) |
| approval / denied | Needs approval / Denied tag |  |

## 3. Timing and motion
- The body rises 5 px in over 200 ms; the chevron turns on the same curve.
- The group label shimmers while any call runs and settles to solid.
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| result | Open on a capped result |
| running | Live timer |
| error | Opened on error |
| group | Group header |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Header, copy, Retry, Show all |
| Enter / Space | Open / close card or group |

## 6. Accessibility
- Header button with aria-expanded; statuses are text.
- Copy buttons named Copy arguments / Copy result; the clipboard text is in data-copy for the host.

## 7. Responsive
- Summary truncates first; name, status and time keep their width.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| card | CallCard | — | name, summary, status, args, result, error, started, duration_ms, open, touched |
| now | ClockMs | — | live timer |
| icon | String | tool |  |
| max_code | u32 | 220 | px |
| on_toggle | EventHandler<()> | — | call CallCard::toggle |
| on_retry | Option<EventHandler<()>> | None | shows Retry |
| ToolCallGroup · cards / open / on_open / on_toggle / on_retry | — | open = true |  |

## 9. Edge cases
- `fail` opens the card only when it was never toggled by hand (openOnError).
- Durations format as 640ms / 1.2s.
- A controlled `open` is the card's `open` field.

## 10. Implementation
- Core: `CallCard`, `CallStatus`, `group_label`, `ms_label` (`oxyn_core::parity::tools`), with tests.
- Dioxus: `ToolCallCard`, `ToolCallGroup` (parity/tools.rs, tools.css).
- GPUI: `ToolCallCardEl` (`toggle`, `retry`).

## 11. References
- affectUI Tool Call Card (https://www.affectui.com/components/tool-call-card) — behaviour-only rebuild, no source copied.
