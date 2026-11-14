**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: In-view title (`in-view-title`)

Family: Loaders & text ([loaders.md](loaders.md), [text-reveal.md](text-reveal.md)). Closes the parity row(s): Arc UI · In-view title.

## 1. Purpose
A section title that comes in as it scrolls into view (host-observed, e.g. 45 % visible): blur (default; opacity leads, blur clears next, the transform settles last), word and line (rise from a clip; line takes explicit `lines`), tracking (letter spacing opens), wipe (uncovers left to right). With `once = false` it hides with a quick fade and plays again.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| hidden | Invisible |  |
| shown | Revealed | once=false + out → hidden |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| blur | Default |
| word | Words |
| line | Lines |
| tracking | Tracking |
| wipe | Wipe |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- A real heading with the full title as its label.

## 7. Responsive
- Lines wrap.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| text | String | — |  |
| variant | blur · word · line · tracking · wipe | blur |  |
| tag | h1 · h2 · h3 | h2 |  |
| lines | Vec<String> | [] |  |
| shown | bool | — |  |
| once | bool | true |  |

## 9. Edge cases
- `line` without lines falls back to one line.

## 10. Implementation
- Core: — (`oxyn_core::parity::text`), with tests.
- Dioxus: `InViewTitle` (parity/text.rs, text.css).
- GPUI: not yet (plan #28 follow-up; the core models are shared).

## 11. References
- Arc UI In-view title (https://uiarc.dev/components/in-view-title) — behaviour-only rebuild, no source copied.
