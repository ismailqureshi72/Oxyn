**Status:** v1 — implemented (plan #26).

# Spec: alert

> Implemented in `oxyn-core::primitives::feedback` (`Alert`, `AlertTone`), `oxyn-dioxus::Alert`, `oxyn-gpui::AlertEl`.
> Source: Arc UI Alert (behaviour only, no source used).

## 1. Purpose
A message that stays until it is dealt with: a failed deploy with Retry and logs, a context window nearly full, an archived workspace. It sits in the flow of the page, not over it.

## 2. State machine
```
Open(tone, copy) ─tone change─▶ icon morphs, copy rises (version + 1)
Open ─copy change─▶ copy rises
Open ─dismiss─▶ Collapsing(320 ms, height → 0) ─▶ Closed
Closed ─open=true─▶ Expanding
Open ─Show logs / Details─▶ Open(expanded) ─again─▶ Open   Oxyn plus
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| info / success / warning | tone icon, title, body | Tab to buttons | `status` |
| danger | red tone | — | `alert` |
| agent (Oxyn plus) | agent violet | — | `status` |
| details open | monospace log block | Enter toggles | `aria-expanded` |
| dismissed | height collapses | — | — |

## 4. Micro-interactions
The icon pops in when the tone changes; the copy rises out of a 3 px blur when it changes. Dismissing collapses the height so content below closes the gap. Details open with a height transition.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Icon morph | 260 ms | `ease.out` | swap |
| Copy change | 220 ms | `ease.out` | swap |
| Collapse | 320 ms | `ease.out` | instant |
| Details | 260 ms | `ease.out` | instant |

## 6. Variants
Free: info, success, warning, danger; dismissible; title-only.

**Oxyn plus:** `agent` tone, actions (Retry, Show logs), collapsible details, `soft` · `outline` · `solid` styles.

## 7. Edge cases
Long titles wrap; details scroll inside 140 px. Changing tone while collapsing reopens with the new tone. Inside narrow panels the actions wrap below.

## 8. Accessibility
Danger is `role=alert`, the rest `role=status`, so tone changes are announced with the right urgency. The icon is decorative; the dismiss button reads "Dismiss: <title>". Colour is never the only signal: each tone has its own icon.

## 9. Props sketch
```rust
Alert { title, tone: "info"|"success"|"warning"|"danger"|"agent", body, open, dismissible,
        actions: Vec<String>, details: Option<String>, variant: "soft"|"outline"|"solid",
        on_dismiss: EventHandler<()>, on_action: EventHandler<String> }
```

## 10. Related components and references
`toaster.md` (transient), `notification-stack.md`, `badges-chips.md`. Arc UI `/docs/components/alert` — behaviour only.
