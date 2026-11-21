# Renamed items

The naming pass (P4) gave Oxyn its own names to items whose name came from a reference library. Generic UI names (button, tabs, dialog, select, tag input…) stay as they are.

Old slugs keep working:

- **Site**: the old page URL answers with a 301 to the new page (`site/src/lib/renames.json`, written into `dist/_redirects`).
- **CLI and MCP**: the registry index lists the old slug in the item's `aliases`, so `oxyn add <old-slug>` installs the item under its new name (the MCP server's `add_component` and `get_component` do the same). The Pro registry (`/api/r/<old-slug>.json`) serves and counts it as the item itself.
- **Rust**: the old component names stay exported: Dioxus as hidden re-exports, GPUI elements as `#[deprecated]` type aliases. Use the new names in new code.

The internal `parity` module of `oxyn-core`, `oxyn-dioxus` and `oxyn-gpui` is now `kit`; `oxyn add` writes those files under `src/oxyn/kit/`. Spec files in `docs/specs/` keep their original file names.

## Slugs (41)

| Old slug | New slug | Old name | New name |
|---|---|---|---|
| `agent-crew` | `crew-tree` | Agent Crew | Crew tree |
| `agent-cursors` | `presence-cursors` | Agent Cursors | Presence cursors |
| `agent-findings` | `findings-feed` | Agent Findings | Findings feed |
| `agent-graph` | `tool-network` | Agent Graph | Tool network |
| `agent-handoff` | `relay-trail` | Agent Handoff | Relay trail |
| `agent-inbox` | `task-inbox` | Agent Inbox | Task inbox |
| `agent-lanes` | `live-lanes` | Agent Lanes | Live lanes |
| `agent-ledger` | `change-ledger` | Agent Ledger | Change ledger |
| `agent-monitor` | `run-monitor` | Agent Monitor | Run monitor |
| `agent-queue` | `job-queue` | Agent Queue | Job queue |
| `agent-recap` | `overnight-recap` | Agent Recap | Overnight recap |
| `agent-sequence` | `sequence-diagram` | Agent Sequence | Sequence diagram |
| `autonomy-dial` | `autonomy-levels` | Autonomy Dial | Autonomy levels |
| `away-summary` | `return-brief` | Away Summary | Return brief |
| `collaboration-stream` | `crew-chat` | Collaboration Stream | Crew chat |
| `context-map` | `context-treemap` | Context Map | Context treemap |
| `cost-waterfall` | `spend-breakdown` | Cost Waterfall | Spend breakdown |
| `parallel-attempts` | `attempt-race` | Parallel Attempts | Attempt race |
| `judge-bracket` | `model-knockout` | Judge Bracket | Model knockout |
| `running-now` | `activity-pill` | Running Now | Activity pill |
| `tool-access-map` | `tool-reach` | Tool Access Map | Tool reach |
| `rate-limit-planner` | `load-planner` | Rate Limit Planner | Load planner |
| `fanout-grid` | `task-grid` | Fan-out Grid | Task grid |
| `delegate-board` | `assign-board` | Delegate Board | Assign board |
| `thinking-loader` | `thinking-pill` | Thinking Loader | Thinking pill |
| `thinking-state` | `thinking-panel` | Thinking State | Thinking panel |
| `mission-control` | `fleet-overview` | Mission control | Fleet overview |
| `mission-timeline` | `fleet-timeline` | Mission timeline | Fleet timeline |
| `elastic-slider` | `rubber-slider` | Elastic slider | Rubber slider |
| `morph-select` | `unfold-select` | Morph select | Unfold select |
| `confirm-morph` | `inline-confirm` | Confirm morph | Inline confirm |
| `skeleton-morph` | `skeleton-reveal` | Skeleton morph | Skeleton reveal |
| `action-swap` | `action-states` | Action swap | Action states |
| `smart-form-fill` | `form-autofill` | Smart form fill | Form autofill |
| `date-reel` | `date-wheel` | Date reel | Date wheel |
| `in-view-title` | `scroll-title` | In-view title | Scroll title |
| `market-terminal` | `trading-desk` | Market terminal | Trading desk |
| `invoice-studio` | `invoice-desk` | Invoice studio | Invoice desk |
| `release-readiness` | `launch-checklist` | Release readiness | Launch checklist |
| `model-arena` | `blind-compare` | Model arena | Blind compare |
| `prompt-studio` | `prompt-bench` | Prompt studio | Prompt bench |

## Names only (11)

Sentence case, slug unchanged.

| Slug | Old name | New name |
|---|---|---|
| `ai-answer` | AI Answer | AI answer |
| `cited-answer` | Cited Answer | Cited answer |
| `message-actions` | Message Actions | Message actions |
| `writing-assistant` | Writing Assistant | Writing assistant |
| `decision-tree` | Decision Tree | Decision tree |
| `memory-panel` | Memory Panel | Memory panel |
| `plan-vs-actual` | Plan vs Actual | Plan vs actual |
| `root-cause-tree` | Root Cause Tree | Root cause tree |
| `scheduled-runs` | Scheduled Runs | Scheduled runs |
| `trace-waterfall` | Trace Waterfall | Trace waterfall |
| `fallback-chain` | Fallback Chain | Fallback chain |
