**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Thinking loader (`thinking-loader`)

Family: Loaders & text ([loaders.md](loaders.md), [text-reveal.md](text-reveal.md)). Closes the parity row(s): affectUI · Thinking Loader.

## 1. Purpose
A small working pill for AI replies: an eight-ray spinner and a shimmering label that walks through `phases` every `interval_ms` (default Thinking → Searching the web → Reading sources → Writing response), holding the last or looping, each new label rising in; once `done` the spinner becomes a check and the label “Thought for Ns” (seconds since start). Pill or plain, sm (28 px) or md (34 px).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| working | Spinner + shimmering phase | done → done |
| done | Check + Thought for Ns | start over → working |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| phases | Default |
| done | Settled |
| plain-sm | Plain, small, looping |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- role=status, aria-live=polite.

## 7. Responsive
- Label truncates in the pill.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| phases | Phases | — | list, interval_ms (2200), looping, started, done |
| now | ClockMs | — |  |
| size | sm · md | md |  |
| variant | pill · plain | pill |  |

## 9. Edge cases
- A controlled phase: set `started` so `index(now)` lands on it.
- Done label rounds to whole seconds, at least 1.

## 10. Implementation
- Core: `Phases` (`oxyn_core::parity::text`), with tests.
- Dioxus: `ThinkingLoader` (parity/text.rs, text.css).
- GPUI: `ThinkingLoaderEl` (ray dots, shimmer label, check on done; `reduced(true)` stills it). The host re-renders on its frame tick.

## 11. References
- affectUI Thinking Loader (https://www.affectui.com/components/thinking-loader) — behaviour-only rebuild, no source copied.
