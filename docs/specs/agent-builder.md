**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Agent builder (`agent-builder`)

Section: Blocks · Agents (page `/blocks/agents/agent-builder/`, Pro). Reference: uiarc "Blocks · Agents" ("Agent builder"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AgentBuilder` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Builder` (GPUI), rules in `oxyn_core::blocks::builder`.

## 1. Purpose
Configure one agent in a card (name, model, instructions, tools, knowledge), try it, and publish it.

## 2. States
Configure · Test (prompts, streaming reply, Stop) · Published · Unsaved changes (measured against the live config, so reverting clears it) · Publishing · Publish failed (Try again) · Upload progress / upload failed (Retry).

## 3. Micro-interactions
Tool switches slide; uploads fill; the reply streams two words a tick and names the tools it used; the publish state word changes with an icon.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Upload | 2600 ms | linear |
| Publish | 1400 ms | — |
| Reply stream | 2 words per 34 ms | linear |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `configure` | The billing assistant, published. |
| `test` | The Test view (Oxyn extra). |
| `unsaved` | Unsaved changes whose first publish fails (Oxyn extra). |

## 6. Responsive and edge cases
Up to 640 px; names and descriptions truncate with an ellipsis; the Test thread scrolls with its composer pinned; the view switch takes its own row under 340 px. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Every field and tool switch is labelled (the tool text is in the switch name); the model picker is a menu of radio items; uploads are progressbars; publishing and replies are announced, including that nothing was deployed. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Builder, ..)`; presses by id (ab-view-<n>, ab-menu, ab-model-<n>, ab-tool-<n>, ab-retry-<n>, ab-publish, ab-prompt-<n>, ab-stop), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-agents, agent-skills.
