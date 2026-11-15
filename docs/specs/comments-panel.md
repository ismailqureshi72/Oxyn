**Status:** built v0.1 (2026-11-15, plan #32 affectUI fidelity pass).

# Spec: Comments panel (`comments-panel`)

Family: Chat / review. Closes the parity row: affectUI · Comments Panel. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A side panel of every comment on a document or run, where some comments are questions and some
are `/edit` instructions the agent can run. Pairs with [comment-thread.md](comment-thread.md).

## 2. Parts
| Part | Behaviour |
|---|---|
| Header | Title, tabs All / Questions / Edits (Edits carries a count badge of open edits), a sort menu (Newest first / Oldest first), optional ✕ |
| Comment | Portrait, name, time ("just now", "4min ago", "2h ago"), text with a leading `/Edit` greyed as a command; Resolve toggle (resolved comments dim) |
| Edit run | `/edit` comments carry **Run edit** → spinner "Running…" (`run_ms`) → "✓ Edit applied" with Undo |
| Composer | "Add a comment…"; the new comment lands on top as "just now" |

## 3. States
| State | Looks | Leaves on |
|---|---|---|
| list | Filtered and sorted (`panel_view`) | tab, sort |
| empty tab | "No comments here" | another tab |
| running | Spinner on that comment | `run_ms` |
| applied | Check + Undo | Undo |
| resolved | Dimmed row, Reopen | Reopen |

## 4. Timing and motion
- Tab underline slides (200 ms); new comments rise in (240 ms); the Edits badge rolls.
- Reduced motion: no travel.

## 5. Variants
| Variant | What differs |
|---|---|
| all | All tab |
| edits | Starts on Edits |
| questions | Starts on Questions |

## 6. Keyboard and accessibility
- Tabs are a `tablist` (←/→); the sort menu uses `menuitemradio`; Enter in the composer posts.
- Run results are announced in a polite live region. The list never scrolls inside a preview; the panel grows.

## 7. API
Dioxus `CommentsPanel` / GPUI `CommentsPanelEl`; core `PanelComment`, `PanelFilter`, `panel_view`, `open_edits`, `EditRun`, `is_edit`, `is_question`, `ago`.
