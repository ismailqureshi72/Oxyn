# Status indicator

> Spec for the first built component (plan #15). Implemented in `oxyn-core::indicator` (model),
> `oxyn-gpui::StatusIndicator` and `oxyn-dioxus::StatusIndicator`. Both renderers draw what the model
> returns, so colour, words and motion can't drift between desktop and web.

## 1. Purpose
The smallest honest answer to "what is the agent doing?": a dot and a few words. It sits in thread
headers, sidebar rows, tab strips, sub-agent rows and the corner of a chat window. Larger surfaces
(status strip, verb cycler, loaders) build on the same `AgentState` and the same words.

## 2. State machine
The indicator has no state of its own. It renders `AgentState` (architecture.md) at clock time `now`.
Transitions come from `AgentMachine`; `Stalled` is derived from silence, never set by the app.

## 3. States table
Labels use the `neutral` pack; other packs swap the verbs. Colours are the `color.state.*` tokens.

| State | Dot | Label | Motion |
|---|---|---|---|
| `Idle` | hollow ring, `state.idle` | Idle | still |
| `Composing` | hollow ring, `state.composing` | Drafting | still |
| `Submitted` | `state.submitted` | Sending | pulse, `loop.base` (1000 ms) |
| `Thinking { header }` | `state.thinking` | header, else "Thinking" | pulse, `loop.pulse` (2000 ms, 0.45 ↔ 1) |
| `Streaming` | `state.streaming` | Writing | pulse |
| `ToolRunning` (1) | `state.tool-running` | activity verb + target: "Running cargo test" | pulse |
| `ToolRunning` (n) | `state.tool-running` | "Running 3 tools" (count, never a list) | pulse |
| `AwaitingApproval` | `state.awaiting-approval` | Waiting for you | breathe, `loop.calm` (2000 ms, 0.7 ↔ 1) |
| `Queued { n }` | `state.queued` | "2 queued" | pulse |
| `Steering` | `state.steering` | Steering | pulse |
| `Stalled` soft | ramps from the stalled-from colour to `state.stalled` | stalled-from label | ramp `dur.stall.ramp` (2000 ms, linear), then still |
| `Stalled` hard | `state.stalled` | "No response for 45s" | still |
| `Done` | `state.done` | pack `turn_done`: "Worked for 1m 12s" ("Done" under 1 s) | still |
| `Error { kind }` | `state.error` | "Stopped · timeout" | still |
| `Cancelled` | `state.cancelled` | "Interrupted after 42s" | still |

No actions: the indicator is read-only and never takes focus.

## 4. Micro-interactions
- **Pulse** is a cosine on opacity, phase-locked to the window's `MotionClock`, so every indicator in a
  window breathes together (GPUI reads `clock.phase`; CSS uses a negative `animation-delay`).
- **Stall ramp** runs once on entering soft stall; recovery snaps back over `dur.stall.recover` (200 ms).
- **Label changes** swap instantly here. Animated verb swaps are the verb cycler's job (plan #17).

## 5. Timing table
| Transition | Duration | Easing | Rate |
|---|---|---|---|
| Busy pulse | `loop.pulse` 2000 ms | cosine | clock rate (≤ 60 fps, 20 unfocused) |
| Submitted pulse | `loop.base` 1000 ms | cosine | same |
| Waiting breathe | `loop.calm` 2000 ms | cosine | same |
| Stall ramp | `dur.stall.ramp` 2000 ms | `ease.linear` | once |
| Stall recover | `dur.stall.recover` 200 ms | `ease.out` | once |

Photosensitivity: every luminance loop has a period ≥ `motion.loop.luminance-min-period` (1000 ms);
tested in `oxyn-core`.

## 6. Variants
Free (3 × 2 sizes): `status-indicator/pill` (default: surface, hairline border),
`status-indicator/inline` (no chrome, for rows), `status-indicator/dot` (dot only; label becomes the
accessible name). Sizes `sm` (6 px dot, 12 px text) and `md` (8 px dot, 13 px text). Pro variants
(glyph loaders in the dot, mark morphs) come with the loaders (plan #16).

## 7. Edge cases
- **Long targets** ("Reading a_very_long_file_name.rs"): the label never wraps; hosts clip with an
  ellipsis. The adapter shortens targets (status-text.md §5.3).
- **Tiny rows (16–20 px):** use `sm` + `inline` or `dot`, with the `terse` pack.
- **Hidden windows:** loops stop (clock rule); the label stays correct when shown again.
- **RTL:** the dot sits at the inline start (flex follows direction).

## 8. Accessibility
- Dot is decorative; meaning is always in words (never colour alone).
- Web: a visually hidden `role="status"` `aria-live="polite"` span carries the label. During a hard
  stall it says "No response" without the ticking number, so screen readers hear each state once.
- GPUI: `Role::Status` with the label as the accessible name.
- Reduced motion: no pulse, breathe or ramp; the dot is static at full opacity.
- Contrast: the label is `fg.default` on `surface.raised` (checked in docs/tokens.md §8); the dot is a
  non-text indicator (WCAG 1.4.11, 3:1), with its meaning repeated in the label.

## 9. Props sketch
```rust
// GPUI
StatusIndicator::new("thread-42", &state, &clock)      // neutral pack, dark theme
    .theme(&Theme::LIGHT).variant(IndicatorVariant::Inline).size(IndicatorSize::Small)

// Dioxus
rsx! { StatusIndicator { state, now, pack: "coder", variant: IndicatorVariant::Pill } }
```
