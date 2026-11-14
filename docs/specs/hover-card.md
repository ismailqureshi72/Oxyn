**Status:** v1 — implemented (plan #27).

# Spec: hover card

> Implemented in `oxyn-core::primitives::layout (`HoverIntent`, `initials`)`, `oxyn-dioxus::HoverCard, Profile`, `oxyn-gpui::HoverCardEl`.
> Source: Arc UI Hover card (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Preview a teammate, an agent or a cited source without leaving the transcript.

## 2. State machine
```
Closed ─enter─▶ Opening(500 ms; 80 ms if within 300 ms of a close = warm) ─▶ Open
Closed ─focus / Enter─▶ Open (instant)
Open ─leave─▶ Closing(140 ms grace) ─enter card─▶ Open  ─timeout─▶ Closed
Open ─Escape─▶ Closed + suppressed until the pointer leaves
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| closed | trigger | Tab / Enter opens | — |
| open | card | Escape closes | `tooltip`, trigger `aria-describedby` |
| warm open | fade only | — | — |

## 4. Micro-interactions
Grows from the trigger edge (0.96 → 1, 4 px) with a separate fade; warm opens fade only. Bio and stats settle in with a small stagger.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Open delay | 500 ms (80 warm) | — | same |
| Close grace | 140 ms | — | same |
| Grow | 220 ms | `ease.out` | fade 120 ms |

## 6. Variants
Free: person profile (name, role, bio, stats, meta), side and align.

**Oxyn plus:** agent profile with identity ring, citation source preview.

## 7. Edge cases
Touch taps toggle. Long names ellipsize; stats wrap. Card width caps to the viewport minus 24 px.

## 8. Accessibility
Focus never moves into the card (read-only content). `aria-describedby` only while open.

## 9. Props sketch
```rust
HoverCard { trigger: String, profile: Option<Profile>, source: Option<(title, domain, snippet)>, side, align,
           open_delay: u64, close_delay: u64, open: bool }
```

## 10. Related components and references
`tooltip-popover.md`, `citations-sources.md`. Arc UI Hover card — behaviour only.
