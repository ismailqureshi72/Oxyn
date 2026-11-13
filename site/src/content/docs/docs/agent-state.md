---
title: Agent state
description: The 13 states every Oxyn component understands, and how they are derived.
---

Every Oxyn component that shows agent activity reads one value: `oxyn_core::AgentState`.

| State | Meaning |
|---|---|
| `Idle` | Nothing happening. |
| `Composing` | The user is typing a draft. |
| `Submitted` | Sent; waiting for the first event. Shown as a pulse, not a spinner. |
| `Thinking { header }` | Reasoning or planning; `header` is a model-written summary when the protocol has one. |
| `Streaming` | Answer tokens arriving. |
| `ToolRunning { tool, concurrent }` | A tool is running; `concurrent` counts calls in flight. |
| `AwaitingApproval { request }` | Waiting for the user. The run timer pauses. |
| `Queued { count }` | Busy, with messages queued by the user. |
| `Steering` | A user interrupt is being applied mid-run. |
| `Stalled { since, level, was }` | Derived: no event for a while. `was` is the busy state it stalled from. |
| `Done { elapsed, waited }` | Finished. Working time excludes time spent waiting for the user. |
| `Error { kind }` | Failed: tool, model, timeout, offline, rate limited or other. |
| `Cancelled { after }` | Stopped by the user. |

## Deriving the state

Hosts don't set the state directly. They feed `AgentEvent`s (`Submit`, `Thinking`, `TextDelta`, `ToolStarted`, `ToolFinished`, `ApprovalRequested`, `Finished`, `Failed`, `Cancel`…) into an `AgentMachine`, which folds them into the state:

```rust
use oxyn_core::{AgentEvent, AgentMachine, ClockMs};

let mut run = AgentMachine::default();
run.apply(AgentEvent::Submit, ClockMs(0));
run.apply(AgentEvent::Thinking { header: None }, ClockMs(400));
let state = run.tick(ClockMs(9_000)).clone(); // Stalled after a quiet spell
let worked = run.elapsed(ClockMs(9_000));
```

- `Stalled` is derived by `tick` from event timestamps; idle, finished and awaiting-approval states never stall.
- `oxyn-core` never reads a clock. Time is passed in, which keeps it deterministic in tests and safe on `wasm32-unknown-unknown`.

See the [status indicator](/components/loaders/status-indicator/) for all 13 states side by side.
