**Status:** v1 — implemented (plan #26).

# Spec: skeleton morph

> Implemented in `oxyn-core::primitives::skeleton` (`Morph`, `line_widths`, `timing`), `oxyn-dioxus::SkeletonMorph` + `MorphBlock`, `oxyn-gpui::SkeletonMorphEl`.
> Source: Arc UI Skeleton morph (behaviour only, no source used).

## 1. Purpose
A card or panel whose content arrives a moment later: the skeleton should already be the shape of the content, then become it block by block instead of swapping all at once.

## 2. State machine
```
Loading ─loading=false─▶ Resolving(block i starts at i × stagger)
Resolving(i) : placeholder stretches from skeleton box to content box while fading; content sharpens from blur(4px)
all done ─▶ Ready
Ready ─loading=true─▶ Loading (immediate)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| loading | size-matched bones with a shared sheen | — | `aria-busy`, hidden status |
| resolving | blocks morph in reading order | — | — |
| ready | content | — | `aria-busy=false` |

## 4. Micro-interactions
Each placeholder stretches to the real content's box and fades while the content sharpens out of a 4 px blur; blocks start `stagger` apart in reading order (avatar, title, meta, body, chart, buttons).

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Block morph | 300 ms | `ease.out` | 120 ms fade, no blur |
| Stagger | 40 ms (Arc) | — | 0 |
| Back to loading | instant | — | instant |

## 6. Variants
Free: single blocks, circle avatars, multi-line text blocks.

**Oxyn plus:** chart / media blocks, configurable stagger, GPUI renderer from the same timings.

## 7. Edge cases
Blocks that load faster than the stagger still wait their turn. Text blocks set their line count; the last of several lines is shorter. Very long content grows the card after the morph.

## 8. Accessibility
While loading the region is `aria-busy` with a hidden status; content is not rendered until ready, so nothing half-loaded is announced. Reduced motion fades without stretching or blur.

## 9. Props sketch
```rust
SkeletonMorph { loading, stagger: u32, loading_label, width, children }
MorphBlock { index, width: String, height: f32, radius: String, lines: Option<usize>, line_height, class, children }
```

## 10. Related components and references
`skeletons.md`, `genui-skeletons.md`. Arc UI `/docs/components/skeleton-morph` — behaviour only.
