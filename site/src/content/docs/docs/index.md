---
title: Introduction
description: What Oxyn is, how it is built and where to start.
---

Oxyn is a library of interface components for **agent products**, written in Rust. It covers the parts an agent app needs that generic UI kits don't: a composer that can queue and steer, loaders and status text driven by the agent's state, tool calls with live timers, plans and sub-agent trees, approval cards with risk tiers, and checkpoints you can rewind.

## How it is built

| Crate | What it holds |
|---|---|
| `oxyn-core` | The agent state machine, component models, timing and copy. No UI, no clock. |
| `oxyn-tokens` | Colour, type, spacing and motion tokens for the dark, light and high-contrast themes. |
| `oxyn-dioxus` | Web components for [Dioxus](https://dioxuslabs.com). Every preview on this site is one. |
| `oxyn-gpui` | Desktop components for [GPUI](https://www.gpui.rs) on the same core models. |

Components take the current `AgentState` plus their own props. Time is always passed in (`now: ClockMs`), so the same component renders a live run, a paused poster, or a test at any instant.

## Where to start

- [Installation](/docs/installation/): add the crates and the stylesheets.
- [Agent state](/docs/agent-state/): the 13 states every component understands.
- [Components](/components/): every component with a live preview, controls, props, keyboard and accessibility notes.
- [How the live demos work](/docs/live-demos/): what you are looking at on each component page.

Oxyn is in early development. The free core is MIT licensed.
