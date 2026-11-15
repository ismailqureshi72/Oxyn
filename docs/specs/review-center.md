**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Review center (`review-center`)

Family: Approvals ([review-card.md](review-card.md), [approvals.md](approvals.md)). Closes affectUI **Review Center** (Pro). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Human in the loop at volume. `Center` holds the queue: sorted by risk then oldest (or oldest, or as given), filtered All / High risk / Unsure (confidence below `low_confidence`), one current item. Items carry reasoning, labelled fields and a body made of spans with flagged parts (the agent's note and source), files (a unified diff with line numbers) or plan steps (flagged numbers in amber). Approve, Edit (approve with your text), Reject with a reason (1–4) and Skip; undo puts the item back. Waiting times turn amber at 75 % of the service level and red past it.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| item | Agent line, title, reasoning, fields, body / files / steps, actions | decide |
| editing | Body becomes a textarea; ⌘↵ approves with edits | ⌘↵ / Esc |
| rejecting | Reason group 1–4 | pick / Esc |
| flag open | Note tooltip under the flagged span | F / Esc |
| empty | "All caught up" + Undo last | undo |

## 3. Timing and motion
- The next item rises 8 px over 300 ms; queue rows fade in at their sorted place.
- The pressed decision stamps in the footer (200 ms pop); the progress bar glides (300 ms).
- Filter counts roll; notes fade in 140 ms.
- Reduced motion: no travel.

## 4. Variants
| Variant | What differs |
|---|---|
| queue | Risk order |
| unsure | Unsure filter |
| tight-sla | 20-minute service level |
| batch | Approve all low risk (Oxyn plus) |

## 5. Keyboard
| Key | Does |
|---|---|
| A | Approve |
| E | Edit; ⌘↵ approves with edits, Esc cancels |
| R | Reject; 1–4 reason; Esc cancels |
| S | Skip |
| J / K or ↓ / ↑ | Next / previous |
| F | Next flagged span's note |
| Z or ⌘Z | Undo |
| Esc | Close picker, editor or note |

## 6. Accessibility
- Shortcuts work while focus is anywhere inside; buttons carry aria-keyshortcuts.
- Queue rows are labelled with title, type, risk, confidence and waiting time; aria-current on the current one.
- Flagged spans are focusable with aria-describedby → role=tooltip; decisions, skips and undos are announced with the new count.
- When the current item leaves, the next one becomes current (focus is not lost).

## 7. Responsive
- Container query: side by side when wide (480 px workspace); under 640 px the queue is on top, three rows high; buttons share the width.
- Under 420 px the flag counter hides and button icons step aside; diffs scroll sideways.

## 8. Edge cases
- Skipped items stay in the queue marked "skipped".
- Undo of the last decision only (a stack).
- Filter that empties the queue shows "Nothing waiting".

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| items | Vec<ReviewItem> | — |  |
| now | ClockMs | — |  |
| reviewed | usize | 0 | counted before |
| sla | u64 | 1 800 000 | ms |
| low_confidence | f32 | 0.7 |  |
| reasons | Vec<String> | 4 defaults |  |
| order | Order | Risk |  |
| default_filter | Filter | All |  |
| title | String | "Review center" |  |
| batch_low | bool | false | Oxyn plus |
| on_decide | EventHandler<(String, Verdict)> | — |  |
| on_undo | EventHandler<String> | — |  |
| on_skip | EventHandler<String> | — |  |
| on_filter_change | EventHandler<Filter> | — |  |

## 10. Gaps against the source
- No imperative handle (`ref`); hosts drive it through props and events.
- Fields are read-only in Edit; only the body is editable.
- No custom key bindings yet.

## 11. GPUI
`ReviewCenterEl::new(center, now).sla(ms)` with `on_item(id, "select")` and approve / edit / reject / skip acts.
