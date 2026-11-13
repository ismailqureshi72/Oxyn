**Status:** v1 — implemented (plan #25).

# Spec: inline edit

> Implemented in `oxyn-core::primitives::inline_edit` (`InlineEdit`, `EditPhase`, `EditKey`,
> `EditAction`, `EditVariant`), `oxyn-dioxus::InlineEdit`, `oxyn-gpui::InlineEditEl`. Source: Arc
> UI Inline edit (behaviour only, no source used).

## 1. Purpose
Rename a project, a thread or an agent; fix a task description where it is. The user wants the
text to become editable without a jump and trusts the save to land or tell them it didn't.

## 2. State machine
```
Idle ─click / Enter / Space─▶ Editing(draft = value)
Editing ─Enter (Shift+Enter in multiline = newline) / Tab / blur─▶ validate
validate ─invalid─▶ Editing + message
validate ─unchanged─▶ Idle
validate ─changed─▶ Saving (optimistic: new text shows) ─host ok─▶ Saved (check) ─1.4 s─▶ Idle
Saving ─host error─▶ Failed (old text rolls back, Retry) ─Retry─▶ Saving
Editing ─Esc─▶ Idle (draft discarded)
Editing ─outside change─▶ Conflict("theirs") ─Keep mine─▶ Saving | ─Use theirs─▶ Idle   Oxyn plus
Idle ─outside change─▶ Idle(value', rolls)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | text, pencil on hover | Enter / Space edits | button "Project name: Atlas" |
| editing | field with the same metrics, Save / Cancel | Enter, Esc, Tab | `aria-label`, `aria-invalid` |
| invalid | message under | typing clears | linked by `aria-describedby` |
| saving | new text, small spinner | — | status "Saving…" |
| saved | drawn check, fades | — | status "Saved" |
| failed | old text rolls back, "Retry" | Enter on Retry | status "Couldn't save: …" |
| conflict (Oxyn plus) | warning row, Keep mine / Use theirs | — | `role=alert` |
| suggestions (Oxyn plus) | chips under the field | click applies | — |

## 4. Micro-interactions
The field uses the text's own font and padding so nothing moves when editing starts. The value
rolls up on a change and down on a rollback. The check draws, then fades. The pencil fades in on
hover only.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Enter edit | 0 ms layout shift | — | — |
| Value roll | 200 ms | `ease.out` | swap |
| Check draw | 240 ms, visible 1.4 s | `ease.out` | drawn |
| Pencil | 120 ms fade | `ease.out` | instant |

## 6. Variants
Free: title, body, multiline (grows), placeholder (empty allowed), max length.

**Oxyn plus:** agent rename suggestions, conflict state when the agent renames it mid-edit,
simulated latency and failure for demos and tests.

## 7. Edge cases
Long titles wrap; the end keeps room for the buttons. Whitespace-only is rejected when required.
Multiline trims only trailing space. A save that resolves after another edit started is ignored by
phase. Touch: tap edits; no hover pencil. RTL: the buttons sit at the inline end.

## 8. Accessibility
At rest the text is a real button whose name includes the label and value. The field has an
explicit label and `aria-invalid`. Saving, saved and failed go to a status region; conflicts use
`role=alert`. Save and Cancel are labelled icon buttons that keep focus in the field
(`mousedown` prevented).

## 9. Props sketch
```rust
InlineEdit { value, label, placeholder, multiline, variant: "title" | "body", max: Option<usize>,
             suggestions: Vec<String>, latency_ms, fail, conflict: Option<String>,
             on_save: EventHandler<String> }
```

## 10. Related components and references
`text-field.md`, `button.md` (spinner), `side-panels.md` (renaming threads). Arc
`/components/inline-edit` — behaviour only.
