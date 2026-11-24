---
title: Build an AI agent UI in Rust
description: A step-by-step guide to an AI agent interface in Rust with Dioxus or GPUI and Oxyn UI components - chat composer, streaming answers, tool calls, approvals and checkpoints.
---

An AI agent interface has more moving parts than a chat box. The user types into a composer, the model streams an answer and reasons out loud, the agent calls tools, some of those calls need a yes before they run, and a run you don't like needs a way back. This guide builds that interface in Rust with [Oxyn UI](/about/) components, on [Dioxus](/dioxus/) for the web or [GPUI](/gpui/) for the desktop.

## 1. Set up the project

Install the oxyn CLI and set up a Dioxus or GPUI project once:

```sh
cargo install oxyn-cli
oxyn init
```

`oxyn init` detects the framework from `Cargo.toml`, writes `oxyn.toml`, creates `src/oxyn/` and adds the `oxyn-core` dependency. See [the CLI docs](/docs/cli/) for every command.

## 2. The chat UI: composer and messages

Start with the [composer](/components/chat/composer/): send when idle, stop while the agent works, and queue or steer when the user types during a run. Add the [model picker](/components/chat/model-picker/) and the [context meter](/components/chat/context-meter/) so the user can see what the model is and how full its context is.

```sh
oxyn add composer model-picker context-meter
```

Answers arrive as a stream: [streaming markdown](/components/chat/streaming-markdown/) renders partial markdown without flicker, and the [reasoning block](/components/chat/reasoning-block/) shows the model's thinking, collapsed once the answer starts. [Message bubbles](/components/chat/message-bubble/) hold the turns. The guide [Build a composer](/guides/composer/) goes deeper.

## 3. Show what the agent is doing

While a run is going, show it. A [tool call](/components/agents/tool-call/) row names the tool, its arguments, its state and its result; [text shimmer](/components/agents/text-shimmer/) carries the "working" status line. See [Show what your agent is doing](/guides/agent-activity/).

```sh
oxyn add tool-call text-shimmer
```

## 4. Ask before acting

Some actions should wait for the user: sending email, deleting files, spending money. The [approval card](/components/agents/approval-card/) shows what the agent wants to do, how risky it is, and Approve and Deny, with keyboard support. See [Ask before acting](/guides/approvals/).

```sh
oxyn add approval-card
```

## 5. Let the user go back

Agents make mistakes. Checkpoints let the user restore an earlier state or fork a new attempt; the Checkpoints family in the [Agents components](/components/agents/) covers lanes, trees and an undo ladder.

## 6. Let your coding agent do the wiring

Install the [MCP server](/docs/mcp/) and your coding agent (Claude Code, Codex, Cursor and others) can search the library and add components for you:

```sh
cargo install oxyn-mcp
claude mcp add --transport stdio oxyn -- oxyn-mcp
```

## Where to go next

- Browse every [component](/components/), whole [screens](/screens/) of agent apps, and [blocks](/blocks/).
- Theme it: [Theming and tokens](/docs/theming/).
- Check [Accessibility](/docs/accessibility/) for labels, focus and announcements.
