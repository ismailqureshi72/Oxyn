**Status:** v1 — implemented (plan #26).

# Spec: card

> Implemented in `oxyn-core::primitives::feedback` (`Card`, `CardEdge`, `StatusWord`), `oxyn-dioxus::Card`, `oxyn-gpui::CardEl`.
> Source: Arc UI Card (behaviour only, no source used).

## 1. Purpose
A unit of work in a grid or list: a run, a PR, a document, a send waiting for approval. It shows enough to choose, and opens into a quick look for the rest.

## 2. State machine
```
Rest ─hover (fine pointer)─▶ Lifted (−2 px, media 1.03)
Rest ─title Enter / click─▶ QuickLook (modal, focus inside) ─Esc / × / scrim─▶ Rest, focus → title
status A ─status B─▶ changed words roll in, staggered 50 ms
edge: none | working (beam) | attention | done | error   Oxyn plus
selectable: off ─toggle─▶ selected   Oxyn plus
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| rest | media, title, description, byline, status, action | Tab: title, action | `article` labelled by the title |
| hover | lifted, shadow, media zoom | — | — |
| quick look | modal with media and details | Esc closes | `dialog`, `aria-modal` |
| working edge | beam around the border | — | — |
| selected | check, outline | Space toggles | `checkbox`, `aria-checked` |

## 4. Micro-interactions
Hover lifts 2 px and zooms the media over 1.2 s. Status changes roll in word by word (only the changed words move). The working edge is a light beam that circles the border.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Lift | 220 ms | `ease.out` | none |
| Media zoom | 1.2 s | `ease.out` | none |
| Status words | 260 ms + 50 ms stagger | `ease.out` | swap |
| Quick look | 240 ms | `ease.out` | 120 ms fade |
| Beam | 2.4 s / turn | linear | 4 s / turn |

## 6. Variants
Free: media image or pattern, byline with avatar, status, action, quick look.

**Oxyn plus:** status edge (working beam, attention, done, error), selectable cards, compact `row` layout.

## 7. Edge cases
A definite width keeps grid cells steady; long titles wrap to two lines. No media collapses the band. Touch skips hover effects.

## 8. Accessibility
Only the title is a button, so the action stays a separate tab stop. The quick look is a modal dialog that returns focus to the title. Status changes are read through a polite region.

## 9. Props sketch
```rust
Card { title, description, media: Option<String>, meta, status, action, details, open,
       edge: "none"|"working"|"attention"|"done"|"error", selectable, layout: "card"|"row", width,
       children, on_open_change, on_action, on_select }
```

## 10. Related components and references
`dialog.md`, `pr-card.md`, `review-card.md`, `border-beam.md`. Arc UI `/docs/components/card` — behaviour only.
