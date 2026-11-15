**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent run (`agent-run`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Agent run (block).

## 1. Purpose
One card that follows a coding agent through a task: the model and task in the header, steps streaming in as a dense trail with the live one shimmering, a stop on a proposed change (file + diff) to Approve & write or Request changes with a note, then the result line with Open pull request.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| running | Trail with live step |  |
| review | Diff with Approve / Request changes | approve → done; reject → running |
| done | Result + Open pull request |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| review | Waiting on the diff |
| running | Streaming |
| done | Result |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Request changes, Approve, Open pull request |

## 6. Accessibility
- The change is a labelled region.

## 7. Responsive
- Diff scrolls in its frame.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| task / model | String | — |  |
| steps | Vec<TrailStep> | — |  |
| change | Option<(String, Vec<String>)> | None |  |
| result | Option<String> | None |  |
| now | ClockMs | — |  |
| on_approve / on_pr | EventHandler<()> | — |  |
| on_reject | EventHandler<String> | — |  |

## 9. Edge cases
- Rejecting without a note sends an empty note.

## 10. Implementation
- Core: reuses `Trail` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentRun` (parity/run.rs, run.css).
- GPUI: `AgentRunEl` (`approve`, `reject`, `pr`); the reject note is collected by the host.

## 11. References
- Arc UI Agent run (https://uiarc.dev/components/blocks/agent-run) — behaviour-only rebuild, no source copied.
