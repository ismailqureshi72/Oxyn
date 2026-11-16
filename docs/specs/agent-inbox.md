**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B).

# Spec: Agent Inbox (`agent-inbox`)

Family: Agents. Closes the parity row: affectUI · Agent Inbox. Behaviour-only rebuild; nothing is copied.
Components: `AgentInbox` (Dioxus) · `AgentInboxEl` (GPUI), models in `oxyn_core::parity::agentsfid`.

## 1. Purpose
Collect every background agent task in one calm board – what waits for you, what runs and what finished – with the last steps and the decision one click away.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Bar | "Agents / Agent inbox" and a Board | List segmented switch |
| Columns | Needs you (amber sparkle), Running (blue), Done (green): 28 px grey header bars, 11 px uppercase tracked label, round count pill that rolls |
| Card | 14 px radius, soft shadow: coloured sparkle + title 15/600, description 13.5 grey clamped to two lines, status tag ("Waiting for you · 6m 14s" amber; running blue pill with a spinning ring and a live clock; done green with a dot; Failed red with !), tool and source counts |
| Unfolded | Hairline, model name, dotted timeline of the last steps with times, Reject + Approve (waiting), Retry (failed) or Open |
| Empty | "Nothing here" in a grey box |
| Status | "2 need you, 2 running" (polite) |

## 3. Behaviour
- Clocks tick every second while running; a task with a finish time moves to Done by itself.
- Approve: running again with "Approved by you"; Reject: failed; Retry: running from 0 with "Retrying with a slower pace".
- A card unfolds in place (Esc folds); the board can switch to a list.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Board, the waiting migration open |
| list | List layout |
| closed | All folded |
| classic | Hairline cards, title-case bold headers, dashed empty box |

## 5. Keyboard and accessibility
Columns are labelled sections, cards buttons with aria-expanded, the layout a radiogroup; status tags carry words; a status line summarises.

## 6. Responsive
Up to 740 px; below 620 px the board stacks into one column.
