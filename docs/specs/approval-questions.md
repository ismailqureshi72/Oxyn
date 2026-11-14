**Status:** v1 — implemented (plan #25).

# Spec: approval questions

> Implemented in `oxyn-core::primitives::questions` (`QuestionDeck` in `DeckMode::Paged`,
> `AUTO_ADVANCE_MS`, `ROLL_MS`), `oxyn-dioxus::ApprovalQuestions`, `oxyn-gpui::ApprovalQuestionsEl`.
> Ported from Beautiful UI's **Approval Card** (MIT, © 2026 Shane Levine) — behaviour, labels and
> timings; drawn with Oxyn tokens and Oxyn's own icons (Beautiful UI's paid Central Icons are not
> used). Complements `approval-card.md`, which covers tool-call approvals.

## 1. Purpose
Before acting, the agent needs a few answers: how many flavours, which mix-ins, which market
first. One question at a time, quick to answer, with room for "something else".

## 2. State machine
```
Asking(index i of n, answers[])
single choice ─click─▶ answers[i] = [k] ─480 ms─▶ Asking(i+1) (counter rolls up)
multi ─click─▶ toggle k (waits for Continue)
custom row ─type─▶ replaces a single choice ─Enter─▶ answers[i] = Text, advance
Skip ─▶ answers[i] = Skipped, advance
Prev / Next ─▶ Asking(i ∓ 1) (counter rolls down / up)
Continue / ⏎ on the last ─▶ Review (Oxyn plus) ─Send─▶ Sent
Continue / ⏎ on the last ─▶ Sent ("Answers sent") + on_submitted(answers)
Sent ─Start over─▶ Asking(0, cleared)
Dismiss / Esc ─▶ Dismissed ("Open approval") ─▶ Asking
timeout (Oxyn plus) ─▶ answers Recommended
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| question | "1 / 3", ‹ › ×, question, options, "Something else…" | Enter continues, Esc dismisses | `radiogroup` or checkbox `group` |
| picked (single) | ring + filled dot | auto-advances | `aria-checked` |
| picked (multi) | filled box, drawn tick | — | `aria-checked` |
| custom | pencil row with input | Enter sends | labelled input |
| footer | quiet Skip, dark Continue / Send with ⏎ | — | Continue disabled until answered |
| review (Oxyn plus) | question → answer rows with Edit | — | — |
| sent | pill "Answers sent", Start over | — | `status` |
| dismissed | "Open approval" button | — | — |

## 4. Micro-interactions
The step counter rolls like an odometer (up when moving forward). Pages slide vertically with a
360 ms ease and the card's height fits the active page. Single choice auto-advances after 480 ms so
the pick is seen first. The dark button's label crossfades to Send on the last page.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Counter roll | 350 ms (400 ms total) | `cubic-bezier(.4,0,.2,1)` | swap |
| Page slide + height | 360 ms | `cubic-bezier(.22,1,.36,1)` | 1 ms |
| Auto-advance | 480 ms after a single pick | — | same delay |
| Dot / tick | `ox-snap` / 220 ms draw | `ease.out` | instant |

## 6. Variants
Free (from the source): radio and check questions, custom answer row, labels (`skip`, `continue`,
`send`, placeholder, sent message), resettable.

**Oxyn plus:** review page before Send; Recommended tags; timeout that answers with Recommended.

## 7. Edge cases
Questions without a custom row (`allow_text = false`). Going back keeps answers and focuses the
picked option. A custom answer on a multi question is kept alongside picks. Long questions wrap;
the card is up to 460 px and fills smaller containers.

## 8. Accessibility
Single choice is a `radiogroup` of `radio`s, multi a `group` of `checkbox`es, both named by the
question. The counter reads "Question 1 of 3". The card is `aria-live=polite`, so each new question
is read. Prev, Next and Dismiss are labelled icon buttons.

## 9. Props sketch
```rust
ApprovalQuestions { questions: Vec<Question>, skip_label, continue_label, send_label,
                    custom_placeholder, sent_message, resettable, review, timeout_s,
                    on_submitted: EventHandler<Vec<String>>,
                    on_answer_change: EventHandler<(usize, String)> }
```

## 10. Related components and references
`decision-card.md` (same model, queue mode), `approval-card.md`, `genui-forms.md`. Beautiful UI
Approval Card — MIT, © 2026 Shane Levine (attribution kept in the source files).
