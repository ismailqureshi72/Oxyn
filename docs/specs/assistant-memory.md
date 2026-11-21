**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: Assistant memory (`assistant-memory`)

Section: Blocks · AI (page `/blocks/ai/assistant-memory/`, Pro). Reference: uiarc "Blocks · AI" ("Agent memory", https://uiarc.dev/components/blocks/agent-memory), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AssistantMemory` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Memory` (GPUI), rules in `oxyn_core::blocks::memory`.

## 1. Purpose
What an assistant remembers about you, one sentence each: filter, search, edit or forget in place, file what the chat just learned, add your own, and switch memory or chat history off. Arc "Agent memory" ships as `assistant-memory` because `agent-memory` is the Agent / Memory screen.

## 2. States
All or one group · Search open (match count announced) · Memory updated pill (files into its group with a highlight) · Editing (Enter saves, Escape cancels) · Forgotten (Undo bar with a 6 s countdown) · Added (sorted by a keyword guess) · Add refused (memory full or saved memories off) · Near capacity (meter warns in amber with an icon) · Saved memories off (note, list dimmed).

## 3. Micro-interactions
A filed or added memory glows for 1.6 s; row actions fade in on hover or focus in reserved space so text never shifts; the undo countdown ticks each second.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Highlight | 1,600 ms (GLOW_MS) | ease-out |
| Undo window | 6,000 ms (UNDO_MS) | — |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `memories` | All memories with a new one to file. |
| `full` | Near capacity, Preferences only (Oxyn extra). |
| `paused` | Saved memories off (Oxyn extra). |

## 6. Responsive and edge cases
Up to 640 px. The list scrolls inside a 380 px area. Below 520 px of block width the switches stack and sources hide; below 440 px the meter takes the full width. Row actions stay visible on touch screens. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
A level two heading; each group is a labelled group with a level three heading and its count; switches are named by their titles; the meter is a meter with a text value; filter pills are toggle buttons with roving arrow keys; Edit and Forget name their memory. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Memory, ..)`; presses by id (mm-saved, mm-history, mm-f-<n>, mm-search, mm-file, mm-edit-<id>, mm-save, mm-cancel, mm-forget-<id>, mm-undo), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
agent-memory, memory-receipts, settings-switches.
