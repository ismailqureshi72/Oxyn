**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Text reveal (`word-reveal`)

Family: Loaders & text ([loaders.md](loaders.md), [text-reveal.md](text-reveal.md)). Closes the parity row(s): affectUI · Text Reveal; Arc UI · Text reveal.

## 1. Purpose
Text that arrives word by word or letter by letter: each piece rises out of a clip and sharpens from a blur (softer for paragraphs) on a stagger capped at 900 ms total, after an optional delay. It plays when `shown` turns true (the host observes the viewport); with `once` it stays revealed, otherwise it plays again on each entry. Pure CSS once rendered, so text is never left hidden by slow scripts.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| hidden | Pieces below their clip |  |
| revealed | Pieces in place | once=false + scroll out → hidden |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| heading | h2 by word |
| paragraph | p, softer |
| chars | By letter |
| replay | Once off |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- The element carries the full text as aria-label.

## 7. Responsive
- Wraps normally.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| text | String | — |  |
| by | word · char | word |  |
| tag | h1 · h2 · h3 · p · span | p |  |
| stagger_ms | u32 | 40 |  |
| delay_ms | u32 | 0 |  |
| duration_ms | u32 | 700 |  |
| shown | bool | true |  |
| once | bool | true |  |

## 9. Edge cases
- Spaces stay inside each word box so wrapping is natural.

## 10. Implementation
- Core: — (`oxyn_core::kit::text`), with tests.
- Dioxus: `WordReveal` (kit/text.rs, text.css).
- GPUI: not yet (plan #28 follow-up; the core models are shared).

## 11. References
- affectUI Text Reveal (https://www.affectui.com/motion/text-reveal) — behaviour-only rebuild, no source copied. Also Arc UI Text reveal (https://uiarc.dev/components/text-reveal) — behaviour-only rebuild, no source copied.
