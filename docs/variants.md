# Variants

A **variant** is a named, documented configuration of a component that differs in structure or motion,
not just colour. Colour, radius, font and density are **tokens**, tuned in the Studio, not variants.

## Axes
| Axis | Values |
|---|---|
| Layout | `compact`, `inline`, `card`, `panel`, `floating` |
| Surface | `chat`, `ide`, `desktop`, `mobile`, `voice`, `terminal` |
| Motion | `none`, `subtle`, `expressive` (each with a reduced-motion fallback) |
| Style family | free-form name: `pill`, `liquid`, `ring`, `glyph`, `particle`, `pixel`, `ascii`, ... |

## Naming
`<component>/<style-family>[-<detail>]`, e.g. `send-control/ring-progress`, `loader/particle-orb`,
`text-shimmer/comet`. Names are stable once released.

## Every variant documents
1. Supported `AgentState`s and what each looks like.
2. Timing table (durations, delays, frame rate, easing or spring constants).
3. Keyboard behaviour and accessible labels per state.
4. Reduced-motion fallback.
5. Surfaces and layouts it supports.
6. Tokens it reads.
7. Category tags.
8. Credits / licence of any reference it is based on.

## Per-component minimum
The same micro-level depth applies to **every** component (see `specs/README.md`).
- Free: at least 3 variants per component at launch.
- Pro: deeper sets (target 8–20 for hero components like the send control, loaders and thinking).
