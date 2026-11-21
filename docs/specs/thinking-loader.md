**Status:** built v0.2 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family A; v0.1 plan #28).

# Spec: Thinking pill (`thinking-loader`)

Family: Chat. Closes the parity row: affectUI · Thinking Loader. Behaviour-only rebuild; nothing is copied.
Components: `ThinkingPill` (Dioxus) · `ThinkingPillEl` (GPUI), models in `oxyn_core::kit::answers`.
Oxyn extras on the same page: the plan #28 `ThinkingLoader` (variant `oxyn-thinking-loader`).

## 1. Purpose
A small working pill for an AI reply while it thinks, searches and writes.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Pill | 32 px (sm 26 px), 10 px continuous radius, white with hairline shadows (#1F2022 / #343538 dark); plain: no chip |
| Spinner | 14 px, eight thin accent-blue rays fading one after another (0.8 s), no rotation |
| Label | 13 px grey (sm 12 px) with a darker band sweeping through; done: solid |
| Done | The rays shrink away, a blue check grows and draws (0.35 s); "Thought for 11s" |

## 3. Behaviour
- Phases advance every 2.2 s; a new label rolls up and blurs out while the next rolls in, and the pill glides to its new width.
- Loop starts over; otherwise it holds the last phase or settles on done.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Looping phases |
| settle | Settles on done |
| done | Done |
| small | 26 px |
| plain | No chip |
| classic | Capsule radius, grey rays |
| oxyn-thinking-loader | Oxyn extra |

## 5. Keyboard and accessibility
role=status, aria-live=polite: each phase and the done label read once; spinner, check and the outgoing label are aria-hidden.

## 6. Responsive
Sizes to its label on one line.

## Appendix: the plan #28 loader (variant `oxyn-thinking-loader`)

### 1. Purpose
A small working pill for AI replies: an eight-ray spinner and a shimmering label that walks through `phases` every `interval_ms` (default Thinking → Searching the web → Reading sources → Writing response), holding the last or looping, each new label rising in; once `done` the spinner becomes a check and the label “Thought for Ns” (seconds since start). Pill or plain, sm (28 px) or md (34 px).

### 2. States
| State | Looks | Leaves on |
|---|---|---|
| working | Spinner + shimmering phase | done → done |
| done | Check + Thought for Ns | start over → working |

### 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

### 4. Variants
| Variant | What differs |
|---|---|
| phases | Default |
| done | Settled |
| plain-sm | Plain, small, looping |

### 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

### 6. Accessibility
- role=status, aria-live=polite.

### 7. Responsive
- Label truncates in the pill.

### 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| phases | Phases | — | list, interval_ms (2200), looping, started, done |
| now | ClockMs | — |  |
| size | sm · md | md |  |
| variant | pill · plain | pill |  |

### 9. Edge cases
- A controlled phase: set `started` so `index(now)` lands on it.
- Done label rounds to whole seconds, at least 1.

### 10. Implementation
- Core: `Phases` (`oxyn_core::kit::text`), with tests.
- Dioxus: `ThinkingLoader` (kit/text.rs, text.css).
- GPUI: `ThinkingLoaderEl` (ray dots, shimmer label, check on done; `reduced(true)` stills it). The host re-renders on its frame tick.

### 11. References
- affectUI Thinking Loader (https://www.affectui.com/components/thinking-loader) — behaviour-only rebuild, no source copied.
