**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Agent gallery (`agent-gallery`)

Section: Blocks · Agents (page `/blocks/agents/agent-gallery/`, Pro). Reference: uiarc "Blocks · Agents" ("Agent gallery"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AgentGallery` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Gallery` (GPUI), rules in `oxyn_core::blocks::gallery`.

## 1. Purpose
A marketplace of agent templates: find one by team or search, preview a sample run and install it inline.

## 2. States
Browsing (team, search, result count) · Empty (with Show in <team>) · Preview (three steps) · Choose workspace · Connecting apps one by one · Could not connect (Retry) · Installed (check, workspace).

## 3. Micro-interactions
Preview steps stagger in; apps connect in turn; the card returns to its summary with a check once installed.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| App connect | 900 ms each | — |
| Preview step | 220 ms stagger | ease-out |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `engineering` | Engineering agents. |
| `support` | Support agents (Oxyn extra). |
| `failing` | Ops, where the second app fails to connect (Oxyn extra). |

## 6. Responsive and edge cases
Two columns from 600 px of card width, one below; the search goes under the title below 560 px; team chips scroll sideways; actions stay visible on touch screens. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Team chips and workspaces are radio groups with a roving tab stop; each card is an article whose app marks are replaced by a sentence and whose rating is read in words; results, progress and installs are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Gallery, ..)`; presses by id (ag-team-<n>, ag-preview-<n>, ag-install-<n>, ag-ws-<n>, ag-grant, ag-back, ag-retry, ag-clear), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-agents, connector-setup.
