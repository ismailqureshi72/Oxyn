**Status:** v1 — implemented (plan #26).

# Spec: dialog

> Implemented in `oxyn-core::primitives::modal` (`Dialog`, `DialogSize`, `CloseReason`, `Phase`), `oxyn-dioxus::Dialog`, `oxyn-gpui::DialogEl`.
> Source: Arc UI Dialog and affectUI Dialog (behaviour only, no source used).

## 1. Purpose
A focused task or a confirmation the user must answer before going back to the page: editing an agent's permissions, renaming a project, confirming that a run, a worktree or a whole project should be deleted. The rest of the app dims and waits.

## 2. State machine
```
Closed ─trigger / open=true─▶ Opening(240 ms) ─▶ Open
Open ─Esc (close_on_escape)─▶ Closing(160 ms) ─▶ Closed, focus → trigger
Open ─backdrop press (close_on_backdrop)─▶ Closing
Open ─× / Cancel─▶ Closing
Open ─Confirm, busy_ms = 0─▶ on_confirm, Closing
Open ─Confirm, busy_ms > 0─▶ Busy (spinner, aria-busy; Esc / backdrop / × refused) ─timer─▶ on_confirm, Closing   Oxyn plus
Open ─open nested─▶ Open(stack + 1); Esc closes the top one only                                           Oxyn plus (core)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| closed | trigger only | Enter / Space opens | `aria-haspopup=dialog`, `aria-expanded=false` |
| opening / open | scrim + panel, header, scrolling body, footer | Tab cycles inside, Esc closes | `dialog` / `alertdialog`, `aria-modal`, `aria-labelledby`, `aria-describedby` |
| busy | confirm shows a spinner, Cancel and × disabled | Esc ignored | `aria-busy=true` |
| closing | panel scales to 0.98 and fades, scrim fades | — | focus returns to trigger |
| floating (modal=false) | panel without a scrim, page stays live | Esc closes | `aria-modal=false` |

## 4. Micro-interactions
The scrim fades in with a 3 px backdrop blur. The panel rises 8 px from 0.96 scale; the title rises out of a 3 px blur. Exit is quicker than enter. The close button tints on hover; the confirm spinner sits before its label so the width barely moves.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Scrim in | 200 ms | `ease.out` | 120 ms fade |
| Panel in | 240 ms, 8 px, 0.96 → 1 | `ease.out` | 120 ms fade |
| Panel out | 160 ms, to 0.98 | `ease.out` | instant |
| Busy confirm | busy_ms (default 0) | linear timer | same |

## 6. Variants
Free: form dialog, size `sm` · `md` · `lg` · `full`, closed-with-trigger, sticky (no backdrop / Escape close).

**Oxyn plus:** `danger` confirm (alertdialog with a red confirm), a confirm that runs for `busy_ms` and can't be dismissed meanwhile, non-modal floating dialog, nested dialogs in the core model, `contained` rendering inside a preview pane.

## 7. Edge cases
Long titles wrap; long bodies scroll under the fixed header and footer. On a phone the panel is `viewport − 32 px` wide and `full` fills the viewport with a 16 px margin. Dialogs opened from a menu restore focus to the menu's trigger. A run finishing while the confirm is busy is the host's to reconcile.

## 8. Accessibility
Focus moves into the panel on open and wraps at both ends with sentinel guards; on close it returns to the trigger. The title names the dialog and the quiet line describes it. Danger dialogs are `alertdialog`. The close button is labelled (`close_label`, default "Close").

## 9. Props sketch
```rust
Dialog { title, description: Option<String>, open: bool, trigger_label: Option<String>,
         size: "sm"|"md"|"lg"|"full", width: Option<f32>, close_on_backdrop, close_on_escape,
         close_button, close_label, contained, modal, tone: "default"|"danger",
         confirm_label: Option<String>, cancel_label, busy_ms: u32, children,
         on_open_change: EventHandler<bool>, on_confirm: EventHandler<()> }
```

## 10. Related components and references
`sheet.md` (docked), `sheet-stack.md` (nested sheets), `bottom-sheet.md`, `approval-card.md` (approvals inline, not modal). Arc UI `/docs/components/dialog`, affectUI Dialog — behaviour only.
