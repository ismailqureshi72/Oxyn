**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: Agent / Tool calls (`agent-tool-calls`)

Family: App screens (pages at `/screens/agent-tool-calls/` and `/full`). Reference: affectUI Agent / Tool calls (https://www.affectui.com/screens/agent-tool-calls), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `AgentToolCallsScreen` (Dioxus) · `AgentToolCallsEl` (GPUI), rules in `oxyn_core::kit::arcscr4`.

## 1. Purpose
See what one agent can do and what it did: which tool it calls now, what failed and why, and decide per tool whether it may run on its own, ask first or not at all.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Header | Agents › Harbor › Tool calls, meta Live with the ISO date (Paused), Range Today / 7 days / 30 days, Pause agent / Resume agent, Connect tool |
| Org chart | The agent card on top (its model's official Claude mark, name, model, Working / Idle / Paused); a line down to the tools in two rows like an org chart; each tool card shows its official coloured logo (Gmail, Postgres, Stripe, Notion, Linear, Slack, GitHub), name, function, source tag (MCP · Notion, Built-in), mode (Allowed, Ask first in amber, Paused faded) and calls, failures and average time in the range; the tool called right now shows Calling and a light running down its line |
| Summary | Calls, failed, waiting and tokens in the range |
| Calls table | Time, tool with logo and function, status (Running, Needs you, Done, Failed, Rejected), duration (a running call counts up), tokens, input and run |
| Right card | Agent: task, model, tools, calls, tokens, Pause, Connect tool. Tool: description and source, function, **May run** Allowed / Ask first / Paused, parameters with type and required, Disconnect (press twice; its calls stay). Call: status, ISO time and run, Approve / Reject for a waiting call, input and output as formatted JSON, the error, **Run again** for a failed call (the Slack fix posts in a channel the bot is in) |
| Connect picker | Unconnected tools with their logos and Connect (joins as Ask first) |

## 3. Behaviour
`ToolCalls` in `oxyn_core::kit::arcscr4`: `step()` once a second: a running call counts up and finishes with its output and tokens, then (unless paused) every third second the agent calls the next tool it may — Allowed tools run, Ask-first tools wait for you, Paused tools are skipped. `select`, `set_mode`, `disconnect` (two presses), `connect`, `pause`, `approve`, `reject`, `solve`, ranges filter by days ago.

## 4. Variants
States: Data · Loading (grey cards and rows) · Empty (the agent alone, a dashed slot Connect the first tool) · **Needs approval** (a Stripe refund waiting) · **Failed call** (a Slack post with Run again) · **Agent paused**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
The chart is a labelled section of buttons that read name, source, mode and numbers; calls are buttons naming tool, time and status; the panel is a region; Approve and Reject sit in an alert; modes are a radio group. Every action says itself back in the frame's polite live region. The focus outline appears only when moving by keyboard.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`AgentToolCallsScreen` takes `layout` and `state` and the host clock `now` (and `live`); sample data comes from `oxyn_core::kit::arcscr4`.

## 8. Known gaps
No canvas layout (pan and zoom) and no compact layout; arrow-key navigation between cards is Tab order only; the table's top edge does not resize; the GPUI view ticks only when the host sends `tc-tick`.

## 9. GPUI
`AgentToolCallsEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `tc-agent`, `tc-tool-<n>`, `tc-call-<id>`, `tc-close`, `tc-mode-auto`, `tc-mode-ask`, `tc-mode-off`, `tc-disc`, `tc-add`, `tc-connect-<n>`, `tc-pause`, `tc-approve`, `tc-reject`, `tc-solve`, `tc-range-<n>`, `tc-tick`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
