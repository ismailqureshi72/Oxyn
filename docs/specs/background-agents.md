**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Background agents (`background-agents`)

Section: Blocks · Agents (page `/blocks/agents/background-agents/`, Pro). Reference: uiarc "Blocks · Agents" ("Agent tasks"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `BackgroundAgents` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::BackgroundAgents` (GPUI), rules in `oxyn_core::blocks::bgagents`.

## 1. Purpose
Hand coding jobs to agents that run off-screen, follow each through its steps to a reviewable diff, then raise and merge a pull request. Arc "Agent tasks" ships as `background-agents` because `agent-tasks` is the Agent / Tasks screen.

## 2. States
Queued · Setting up environment · Running (live step) · Ready for review (diff stat, checks) · Creating pull request · PR open (#n) · Merged · Failed at a step (Retry) · Cancelled (Retry) · Archived (Undo for 6 s).

## 3. Micro-interactions
A new card slides in on top; the live step shimmers; Create pull request turns into the PR number with the GitHub mark and Merge. Chip menus open under their chips.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Queue | 1400 ms | — |
| Set up | 2000 ms | — |
| Step | 1700 ms × 5 | — |
| Pull request | 1200 ms | — |
| Undo window | 6000 ms | — |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `all` | Running, ready, merged and failed tasks. |
| `review` | Ready for review, details open (Oxyn extra). |
| `failing` | The next task you start fails its tests (Oxyn extra). |

## 6. Responsive and edge cases
Up to 760 px and down to 320 px; chips wrap under the field, the shortcut hint hides below 520 px; titles clamp to two lines; details are two columns from 600 px. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Card titles are buttons with aria-expanded and aria-controls; chip menus use menuitemradio with aria-checked; status pairs icon and word; elapsed time and diff stat have spoken labels; every action is announced including that the run and the PR are simulated. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::BackgroundAgents, ..)`; presses by id (ba-repo-<n>, ba-branch-<n>, ba-model-<n>, ba-f-<n>, ba-open-<id>, ba-cancel-<id>, ba-retry-<id>, ba-pr-<id>, ba-merge-<id>, ba-archive-<id>, ba-undo-<id>), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-tasks, pr-card, checks-list.
