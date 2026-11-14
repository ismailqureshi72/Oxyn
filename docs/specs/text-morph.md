**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Text morph (`text-morph`)

Family: Loaders & text ([loaders.md](loaders.md), [text-reveal.md](text-reveal.md)). Closes the parity row(s): Arc UI · Text morph.

## 1. Purpose
A label that morphs into its next state letter by letter: letters are matched by character and occurrence (`morph_letters`), so shared letters stay, new letters stagger in 22 ms apart from a blur and removed letters fade out above.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| settled | The label |  |
| morphing | Entering and leaving letters | ~300 ms |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| follow | Follow → Following |
| copy | Copy → Copied → Done |
| manual | By hand |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- The label is the accessible name; letters are aria-hidden.

## 7. Responsive
- Never wraps.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| text | String | — |  |

## 9. Edge cases
- Gap: kept letters snap to their new position instead of gliding on a spring, and the frame width does not spring (no layout measurement in the core).

## 10. Implementation
- Core: `morph_letters`, `MorphLetter`, `LetterMove` (`oxyn_core::parity::text`), with tests.
- Dioxus: `TextMorph` (parity/text.rs, text.css).
- GPUI: not yet (plan #28 follow-up; the core models are shared).

## 11. References
- Arc UI Text morph (https://uiarc.dev/components/text-morph) — behaviour-only rebuild, no source copied.
