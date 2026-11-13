---
title: Show what your agent is doing
description: Loaders, status text, tool calls and plans, driven by one agent state.
---

An agent that works silently feels broken; one that narrates everything is noise. Oxyn splits the job into layers that all read the same `AgentState`, so they never disagree.

## 1. One busy signal

Put one loader where the agent's answer will appear: a [glyph loader](/components/loaders/glyph-loader/) or a [status indicator](/components/loaders/status-indicator/). It settles into a check, a warning or a cross when the run ends, waits or fails.

## 2. Say what it is doing

Next to it, a [verb cycler](/components/status-text/verb-cycler/) rotates through a verb pack while the agent thinks and pins the tool's verb while a tool runs ("Running cargo test"). An [elapsed timer](/components/status-text/elapsed-timer/) counts working time and leaves out the time it waited for you.

## 3. Show the work

Each tool call is a [tool call row](/components/tools/tool-call/) with a live timer; a turn's calls group into a [tool stack](/components/tools/tool-stack/) that folds into "Worked for 31s" when the turn ends. Longer tasks get a [todo list](/components/tools/todo-list/) or a [plan card](/components/tools/plan-card/) to review first; orchestrators get a [sub-agent tree](/components/tools/subagent-tree/).

## 4. Stream the answer

[Text reveal](/components/status-text/text-reveal/) shows streamed text arriving without feeding tokens to screen readers.

## Wiring

Feed your protocol's events into an `AgentMachine` (see [Agent state](/docs/agent-state/)) and pass `machine.state()` and one shared `now` to every component. Nothing else needs to coordinate them.
