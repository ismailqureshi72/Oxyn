**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent approvals (`agent-approvals`)

Family: G Approvals ([approvals.md](approvals.md)). Closes the parity row(s): Arc UI · Agent approvals (block).

## 1. Purpose
One approval card for agents' tool calls. The waiting call reads like a permission prompt: tool mark, agent and model, the call in plain words with code in backticks set in mono (“Run `vercel deploy --prod`”), the agent's reason and its arguments as rows, some editable. Approve, Deny, or Always allow for this agent or the whole workspace; a new rule also clears every queued call it covers (logged as Auto-allowed). Rules (with Remove) and a decision log sit behind tabs.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| queue | Top call + “N more waiting” | decide → next |
| menu open | Always allow for <agent> / in <workspace> | pick → rule added |
| rules | Rule rows with scope tag and Remove |  |
| log | Outcome dot, summary, edited tag, outcome · decided by |  |
| empty | All caught up |  |

## 3. Timing and motion
- Rows fade in (240 ms); reduced motion: none.

## 4. Variants
| Variant | What differs |
|---|---|
| queue | Deploy with editable target |
| rules | Rules tab |
| fresh | No rules yet |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Arguments, Deny, Approve, menu |
| Arrow keys | Within the tablist (native buttons) |

## 6. Accessibility
- Tabs use role=tab with aria-selected; the menu is role=menu with menuitems.
- Approve becomes “Approve edited” after an argument changes.

## 7. Responsive
- Max 520 px; 110 px argument key column.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | AgentApprovals | — | queue, rules, log, reviewer, workspace |
| title | String | Approvals |  |
| tab | String | queue |  |
| on_decide | EventHandler<(bool, bool)> | — | approve?, edited? |
| on_always | EventHandler<RuleScope> | — |  |
| on_arg | EventHandler<(String, String)> | — |  |
| on_remove_rule | EventHandler<usize> | — |  |

## 9. Edge cases
- Only editable arguments accept edits.
- A workspace rule covers every agent; an agent rule only that agent.

## 10. Implementation
- Core: `AgentApprovals`, `ToolAsk`, `Arg`, `Rule`, `RuleScope`, `LogEntry`, `Outcome`; `code_spans` for backticks.
- Dioxus: `AgentApprovals`.
- GPUI: `AgentApprovalsEl`.

## 11. References
- Arc UI Agent approvals (https://uiarc.dev/components/blocks/agent-approvals) — behaviour-only rebuild; provider brand marks are not reproduced.
