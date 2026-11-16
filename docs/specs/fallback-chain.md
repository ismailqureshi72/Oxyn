**Status:** built v0.2 (2026-11-16, plan #32 affectUI fidelity pass, family B part 8 – exact pass against the ref-1 capture notes).

# Spec: Fallback Chain (`fallback-chain`)

Family: Agents. Closes the parity row: affectUI · Fallback Chain (Pro). Behaviour-only rebuild; sizes, colours, states,
motion and responsive rules follow the ref-1 capture notes (approximate, read off screenshots); models, timings,
reasons and copy are our own. Related Oxyn component: trace-view.
Components: `FallbackChain` (Dioxus) · `FallbackChainEl` (GPUI), model in `oxyn_core::parity::fallbackfid`.

## 1. Purpose
Which model actually served a request: the fallback order as a small flow with a running clock and cost, each failed
try broken red with its reason, the one that answered carried on to the response.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | 680 px (max 100 %), container-sized card: white / #17171A, 1 px #E6E6E6 / #2A2A2E, 14 px radius, soft shadow, 18–20 px padding |
| Header | the request as the title (13/600); right: dot + status (blue "Attempt k of n", green "Answered", red "All failed", 12 px), clock + "7.5s", cost "$0.0049" (11 px muted, tabular) and – when every model failed – a Retry button |
| Request / Response | 108 × 44 px nodes, 10 px radius: glyph, "Request" 12/600 over the route (10 px muted); the response turns green-tinted with "412 tokens" once answered |
| Model card | 46 px, 10 px radius, 10 px gap: order chip, 22 px brand tile (vetted coloured mark), model 12/600 + provider 11 px muted, time over cost on the right (tabular); status line 11 px: red "Rate limited · 429" with a slash glyph, blue "Trying…", blue arrow "Up next", grey "Waiting" / "Not needed", green "Answered · 412 tokens" |
| Card states | trying: blue #2563EB / #4C8DFF border on #EFF6FF; failed: red-tinted border; answered: green-tinted border; waiting / skipped: 55 % opacity (full on hover / focus) |
| Links | 1.5 px dashed #D4D4D8 curves request → every model; failed ones red and broken; the answering one solid green and drawn on to the response; a blue dotted light grows along the branch being tried in step with the try |
| Details card | 220 px popover, 12 px radius, shadow: provider + state tag, then Model, Time ("2.4s of 5.0s"), Cost, Code, Price ("$3 / $15 per 1M"), Why (red) |

## 3. Behaviour
- `fb_states(attempts, elapsed, gap)` walks the chain; `fb_progress` gives the trying attempt's fraction; `fb_meter`
  the clock (stops at the answer) and the money spent (finished tries in full, the trying one pro rata);
  `fb_status` the header words; `fb_usd` / `fb_s` the formats.
- `live` plays the chain and loops after a 4.2 s rest. Every node opens its details card on hover, focus or click
  (click pins it; Escape closes). Retry restarts the chain and calls `on_retry`.
- prefers-reduced-motion: no light, drawing, pulse or slide-ins.

## 4. Variants
live (default) · settled · first-try · second-try · all-failed (Retry) · details · medium (560 px) · narrow (380 px) · classic.

## 5. Keyboard and accessibility
Tab: request, each model, response – each shows its details card on focus; Enter / Space pins it (aria-expanded);
Escape closes. Model buttons are named with place, model, provider and outcome; the header status is a status region;
the links are an aria-hidden SVG and every state is also in words.

## 6. Responsive
Under 600 px the provider leaves the row (it stays in the details card); under 520 px the diagram becomes a vertical
tree on a dashed rail with request, models and response stacked.

## 7. Known gaps
- The capture gives approximate sizes only (screenshot reading); provider/region strings and prices are illustrative.
- GPUI: a static snapshot – straight layout, no curves, light or details card.
