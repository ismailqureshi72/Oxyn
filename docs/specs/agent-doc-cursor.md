**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Agent doc cursor (`agent-doc-cursor`)

Family: AI Components (page `/components/ai/agent-doc-cursor/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AgentDocCursor` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::DocCursor` (GPUI), rules in `oxyn_core::aikit::doccursor`.

## 1. Purpose
Show an agent editing a shared document as a visible collaborator, so people see exactly what changed and approve it.

## 2. States
Idle (Ask) · Reading · Editing: rewrite (select, then stream), insert (a new section grows in), typo fix · Review (tinted changes, Undo and Accept) · Accepted · Undone · Stopped (finished edits stay for review) · Error (Try again). A looping demo accepts on its own after a pause unless hovered or focused.

## 3. Micro-interactions
The agent cursor (its mark colour, name tag) travels to each block; a selection fills as it drags; rewritten words stream in tinted; the typo swaps in place. A teammate cursor reads along one paragraph at a time. The review bar rises in when changes are ready.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Reading | 1000 ms | — |
| Cursor travel | 480 ms | ease-in-out |
| Drag select | 640 ms | linear |
| Word stream | 34 ms per word | linear |
| Loop accept | 3.6 s after review | — |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `editing` | A looping run with a teammate reading along. |
| `review` | No loop: the changes wait in the review bar. |
| `ask` | Idle with an Ask button, no teammate (Oxyn extra). |
| `error` | Couldn't reach the agent, with Try again. |

## 6. Responsive and edge cases
Cursor tags sit inline with the text they act on, so they wrap with it and never cover a line. Under 480 px the document padding tightens and the review bar drops its by-line. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
Headings and paragraphs stay real elements; pending text uses mark; cursors are decorative. Every step is announced; status words pair with icons. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::DocCursor, ..)`; presses by id (dc-ask, dc-stop, dc-accept, dc-undo, dc-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-cursors, diff-view, approval-card.
