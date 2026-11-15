**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Approval gates (`approval-gates`)

Family: G Approvals ([approvals.md](approvals.md)). Closes the parity row(s): affectUI · Approval Gates.

## 1. Purpose
All approval requests of an agent run in one block: open gates form a stack with the one to decide now on top (who asks, kind, what exactly, why, its input or diff, how many steps it holds), and decided gates move into a Decided list with their outcome. Approve, Change (edit the input, then approve) or Reject. While the run is live and nothing is open the block reads Running.

Anatomy:
```
Approvals  Launch the spring newsletter          ● 3 waiting · holds 3 steps
╭ W Writer  message                         holds 2 ╮
│ Send “Spring is here” to 4,210 subscribers         │
│ The draft passed review…                           │
│ to: list:spring-2026 …                             │
│                   Reject  ✎ Change  [Approve ⌘↵]   │
╰───────────────────────────────────────────────────╯
  ╰─────────────────────────────────────────────╯   ← peeking gates
DECIDED 1
● web.search “newsletter send times 2026”     Approved
```

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| open stack | Top card + up to 2 peeking edges | decide → next rises |
| editing | Input becomes a textarea; Approve edit | approve edit → Edited |
| running | Spinner + “Running · new requests appear here” | new gate → stack |
| all decided | “Nothing waiting” | — |

## 3. Timing and motion
- Top card rises in (translateY 8 px, scale .98 → 1, 280 ms); decided rows fade in.
- Reduced motion: no rise, rows appear.

## 4. Variants
| Variant | What differs |
|---|---|
| stack | Message send on top |
| file | File change with diff on top |
| decided | All decided |
| idle | Run over, nothing open |

## 5. Keyboard
| Key | Does |
|---|---|
| ⌘/Ctrl + Enter | Approve the top gate |
| Tab | Reject, Change, Approve |

## 6. Accessibility
- section named by the title; status line is text.
- Peeking layers are aria-hidden; only the top gate is focusable.

## 7. Responsive
- Max 520 px; decided rows truncate the action and keep the outcome visible.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| gates | Gates | — | task, live, gates[{id, agent, kind, action, reason, status, input, diff, holds, note}] |
| title | String | Approvals |  |
| decided | bool | true | show the Decided list |
| on_decide | EventHandler<(String, GateDecision)> | — | Approve · Edit(value) · Reject(reason) |

## 9. Edge cases
- Deciding a gate that is not open is ignored (`decide` returns false).
- Diff stat (+a −d) is derived from the diff lines.

## 10. Implementation
- Core: `Gates`, `Gate`, `GateKind`, `GateStatus`, `GateDecision` with tests.
- Dioxus: `ApprovalGates`.
- GPUI: `ApprovalGatesEl` (`approve`, `reject`, `edit`).

## 11. References
- affectUI Approval Gates (https://www.affectui.com/components/approval-gates) — behaviour-only rebuild.
