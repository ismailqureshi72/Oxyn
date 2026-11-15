**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent orchestrator (`agent-orchestrator`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Agent orchestrator (block).

## 1. Purpose
A lead agent splits a goal into a plan, fans the subtasks out to parallel worker lanes under a 1–4 concurrency limit and merges what comes back. Each lane shows its worker model, the lead's instruction, progress and its result; the header shows the lead and phase (Planning, Working, Merging, Done); a summary appears when merged.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| planning | Lanes queued |  |
| working | Up to N lanes running | lane done → next dispatches |
| merging | All lanes done |  |
| done | Summary |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| working | 2 at a time |
| wide | 4 at a time |
| merged | Summary |
| planning | Lead planning |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Concurrency radios |

## 6. Accessibility
- Concurrency radiogroup; lane state text.

## 7. Responsive
- Lanes wrap at 150 px.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | Orchestration | — | goal, lead, lanes, concurrency, phase, summary |
| on_concurrency | EventHandler<usize> | — |  |

## 9. Edge cases
- Raising concurrency dispatches queued lanes at once.

## 10. Implementation
- Core: `Orchestration`, `Lane`, `OrchPhase` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentOrchestrator` (parity/run.rs, run.css).
- GPUI: `OrchestratorEl`.

## 11. References
- Arc UI Agent orchestrator (https://uiarc.dev/components/blocks/agent-orchestrator) — behaviour-only rebuild, no source copied.
