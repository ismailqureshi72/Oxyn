**Status:** v1 — implemented (plan #25).

# Spec: decision card

> Implemented in `oxyn-core::primitives::questions` (`QuestionDeck` in `DeckMode::Decisions`,
> `Question`, `QuestionOption`, `QuestionKind`, `Answer`, `DeckKey`, `DeckEvent`),
> `oxyn-dioxus::DecisionCard`, `oxyn-gpui::DecisionCardEl`. Source: affectUI Decision Card
> (behaviour only, no source used). Shares the question-card state machine in `genui-forms.md`.

## 1. Purpose
The agent has paused for the user: which cache, which hero image, may it deploy. The question sits
in its own card just above the composer so the user can answer in one key and keep working.

## 2. State machine
```
Asking(queue = [d1, d2, …], focus = Recommended or first)
1–5 ─▶ answer with that row at once ─▶ on_decide(id, answer); the decision leaves; next fades in
↑ ↓ (← → on tiles), Home, End ─▶ focus'; ↓ from the last row ─▶ own-words field
Enter / Space ─▶ answer with focus; in the field, Enter sends the words
approval: 1 Approve / 2 Decline
Esc / Later ─▶ Folded ("2 decisions waiting · Show"), on_later(id)
Folded ─Show / reveal─▶ Asking (card pulses grey once, focus on the selected row)
queue empty ─▶ Done (card folds away)
timeout (Oxyn plus) ─▶ answers with Recommended ; "Always answer this way" stores a rule
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| decision | numbered rows, one "Recommended", own-words field | 1–5, ↑ ↓, Enter, Esc | `group` named by the question; `listbox` |
| pick | three tiles | ← → | `listbox` |
| approval | facts list, Approve (dark) / Decline | 1, 2 | `aria-keyshortcuts` |
| several | pager "1 of 3" ‹ › | — | labelled pager buttons |
| folded | pill "2 decisions waiting · Show" | — | status |
| answered | next decision fades in | — | polite: "Redis. 1 decision waiting." |
| timeout (Oxyn plus) | "Picks Postgres table in 12s" | — | — |

## 4. Micro-interactions
The card rises in above the composer. Answered decisions leave at once and the next fades in. The
grey head carries "Needs you", the task and Later; the white card holds the rows. `reveal` pulses
the card grey once to draw the eye without colour.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Card rise | 200 ms | `ease.out` | 1 ms |
| Next decision | 220 ms fade | `ease.out` | 1 ms |
| Reveal pulse | 700 ms, once | ease-out | none |
| Row hover | 120 ms | `ease.out` | 80 ms |

## 6. Variants
Free: decision rows with summaries, Recommended tag, pick tiles, approval with facts, own words,
queue with pager, default folded.

**Oxyn plus:** timeout that answers with Recommended; "Always answer this way" rule; task line.

## 7. Edge cases
Long titles wrap to two lines then end in …. More than five options: numbers stop at 5, the rest
are reached by arrows. Narrow (< 420 px): tiles become rows; touch rows are 44 px. An agent that
withdraws a decision removes it from the queue; an empty queue folds the card away.

## 8. Accessibility
The card is a `group` named by the question; rows are a `listbox` with one tab stop and
`aria-keyshortcuts` per row; Later carries `aria-keyshortcuts=Escape`. Answers and folds are
announced politely with how many still wait. Recommended is text, not colour.

## 9. Props sketch
```rust
DecisionCard { decisions: Vec<Question { id, prompt, task, kind: Single | Pick | Approval,
               options: Vec<QuestionOption { id, title, summary, recommended }>, facts, allow_text }>,
               default_folded, needs_you, timeout_s: Option<u32>, always,
               on_decide: EventHandler<(String, String)>, on_later: EventHandler<String> }
```

## 10. Related components and references
`approval-questions.md` (paged variant, same model), `approval-card.md` (tool-call approvals),
`genui-forms.md`, `composer.md`. affectUI `/components/decision-card` — behaviour only.
