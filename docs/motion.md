# Oxyn motion language

**Status:** proposed, v1. This is the single source of truth for motion in Oxyn. The machine-readable
copy is [`tokens/oxyn.motion.json`](../tokens/oxyn.motion.json) (DTCG format). The component specs in
[`specs/`](specs/) keep their timing tables, but every number in them must resolve to a token below.
When a spec and this file disagree, this file wins; the [Migration](#14-migration) table lists the spec
edits still to make.

Scope: durations, easings, springs, loops, stagger, delays that gate motion, reduced motion, power and
visibility behaviour, and photosensitivity limits. Behavioural timeouts (undo windows, VAD, reconnect
backoff) are listed in the registry for completeness but are not motion tokens.

Namespaces (one namespace per kind; nothing else is a motion token):

| Namespace | Holds | Example |
|---|---|---|
| `dur.*` | durations: semantic primitives, the step ladder, component tokens | `dur.base`, `dur.step.200`, `dur.row.enter` |
| `ease.*` | cubic-bezier (or exact piecewise) easing curves | `ease.out` |
| `spring.*` | damped harmonic springs (stiffness, damping, mass) | `spring.snap` |
| `loop.*` | repeating motion: period, curve, amplitude | `loop.calm` |
| `stagger.*` | per-item offsets with a cap | `stagger.base` |
| `delay.*` | waits before motion starts (hover intent, show delays, guards) | `delay.loader.show` |
| `motion.*` | policy: frame-rate caps, flash limits, the `motion.reduced` flag | `motion.fps.max` |

Naming rules: lowercase; segments separated by `.`; words inside a segment joined by `-`
(`ease.in-out`, `delay.row-guard`); no underscores; a name is never both a token and a group
(so `dur.reveal` became `dur.reveal.media`, because `dur.reveal.token` exists).

---

## 1. Principles

1. **Motion explains a state change.** Every animation answers "what just changed, and where did it
   go?". Motion that doesn't explain a change isn't allowed. Decoration loops exist only to show that
   an agent is working.
2. **Agent work loops are calm.** Long-running agent states (thinking, working, waiting) use the slowest loops
   (`loop.calm` 2 s, `loop.glow` 3.2 s). Faster loops (`loop.base`, `loop.brisk`) are for small
   spinners inside a row, never for a full surface. A loop slows down when the agent stalls
   (`dur.stall.ramp`); it never speeds up to show urgency.
3. **Nothing loops faster than `loop.brisk`.** No repeating motion has a period below
   **700 ms (≈ 1.43 Hz)** (`motion.loop.min-period`). Anything that repeats a luminance change has a
   period of at least **1 s** (`motion.loop.luminance-min-period`). Frame cycles of a 12–48 px glyph
   (`dur.frame.glyph`, 80–200 ms per frame) are motion inside a glyph, not a loop of the element.
   They are exempt, but they still obey the flash limit (§10).
4. **No motion steals focus.** Animations never move keyboard focus, never scroll the user away from
   where they're reading, and never move an element under a pointer that is about to click. Agent-driven
   scrolls only follow when the user is already at the tail. Agent-driven layout changes animate
   the top edge and keep the reading anchor fixed. Nothing that animates in grabs focus. Approvals
   take focus only through the explicit approval rules, and they arm late (`delay.approval.input-guard`).
5. **Fast in, faster out.** Enters take time to explain where something came from. Exits get out of
   the way (§7.1).
6. **Physics for things that move, curves for things that fade.** Position, size and layout use springs, so
   they can be interrupted and redirected. Opacity, colour and blur use cubic-bezier curves. Fills that
   *represent time* (arming, hold-to-confirm, countdowns) are linear, because their progress is the
   message.
7. **Safety timings are not motion.** `dur.approval.arm`, `dur.approval.hold` and
   `delay.approval.input-guard` keep their length under reduced motion, low power and every theme.
8. **One clock.** All motion in a window runs from the shared `MotionClock` in `oxyn-core`. Loops are
   phase-locked to it, and it slows or pauses with the window (§9).

## 2. Primitive duration scale

Each primitive has **one default** and an **allowed range**. A component may use any value in the
range without a new token. A value outside the range needs a component token in the registry.

| Token | Default | Allowed range | Use |
|---|---|---|---|
| `dur.instant` | **80 ms** | 0–80 ms | press-down, reduced-motion crossfade (every reduced transition uses this) |
| `dur.fast` | **120 ms** | 100–140 ms | hover, colour, small fades, close of small popovers |
| `dur.base` | **180 ms** | 160–200 ms | morphs, chip enter, popover open, crossfades |
| `dur.travel` | **240 ms** | 220–260 ms, **hard cap 260 ms** | an element travelling between two resting places (composer ↔ transcript, panel ↔ fullscreen) |
| `dur.pulse` | **400 ms** | 300–400 ms | a single attention pulse, a completion pass (never repeated) |
| `dur.beat` | **600 ms** | 400–800 ms | the hold of a ✓ or a tint before it fades |

Decisions on the old ranges: `dur.base` 160–200 → **180 ms** (it is the value loaders.md and
container-loaders.md already quote as "`dur.base` 180 ms"). `dur.travel` 220–260 → **240 ms** (most
quoted travel value; 260 stays the cap). `dur.instant` 0–80 → **80 ms** (every reduced-motion rule in the
specs says "80 ms crossfade").

**The step ladder.** Every component token aliases a step `dur.step.N` in the JSON. The ladder is
the full set of durations Oxyn uses: 30, 80, 100, 120, 140, 150, 160, 180, 200, 220, 240, 250, 280,
300, 400, 500, 600, 800, 900, 1200, 1400, 1500, 1600, 2000, 2800 ms. UI transitions sit on a 20 ms grid
from 80 to 280 ms. The off-grid steps each have one job: 30 is the voice level attack, 150 is the
`dur.expand` content fade, and 250 is a delay. Anything longer than 280 ms is a hold, a beat or a highlight.
Holds of whole seconds (4 s stale hold, 5 s toast) are behavioural, not ladder steps. Any other value
off the ladder is a spec bug.

## 3. Easings

| Token | Curve | When |
|---|---|---|
| `ease.out` | `cubic-bezier(0.22, 1, 0.36, 1)` | **Default.** Enters, fades in, rolls, and any motion that *responds to user input* (it starts fast, so it feels immediate). |
| `ease.in` | `cubic-bezier(0.4, 0, 1, 1)` | System-initiated exits: collapse, dismiss, fly-up into the transcript. Never for something the user just pressed. |
| `ease.in-out` | `cubic-bezier(0.65, 0, 0.35, 1)` | Travel between two resting places, crossfades between equals, one-shot passes (scan pass, completion pass). |
| `ease.firm` | `cubic-bezier(0.2, 0, 0, 1)` | Interruptions and coming to rest: "Interrupted", barge-in drop, AwaitingYou rest, failed collapse. Hard deceleration, no overshoot. |
| `ease.linear` | `cubic-bezier(0, 0, 1, 1)` | Fills that represent time (arming, hold, undo hairline), constant-speed sweeps, OKLab colour ramps (the stall ramp). |
| `ease.sine` | `cubic-bezier(0.37, 0, 0.63, 1)` | Breathing loops, run as alternating half-cycles (a raised cosine). Replaces the bare "sine" in the specs. |
| `ease.coast` | piecewise: quadratic ramp on the first and last 20 %, linear between (v = 1.25). CSS exact: `linear(0, 0.0078, 0.0312, 0.0703, 0.125, 0.1875, 0.25, 0.3125, 0.375, 0.4375, 0.5, 0.5625, 0.625, 0.6875, 0.75, 0.8125, 0.875, 0.9297, 0.9688, 0.9922, 1)`; fallback `cubic-bezier(0.25, 0, 0.75, 1)` (max error 0.0095) | Shimmer band travel (OpenCode `coast()`, MIT). |

Rules:
- Colour interpolates in **OKLab** whatever the curve (loaders.md §2.5).
- An easing never overshoots. Overshoot comes only from `spring.soft`.
- Spelling: `ease-out`, `ease-in-out`, a raw `cubic-bezier(0.22, 1, 0.36, 1)`, `sine` and `linear` in spec
  tables are renamed to the tokens above (Migration M-30 to M-34).

## 4. Springs

Springs are damped harmonic oscillators, x″ = −(k/m)(x − target) − (c/m)x′. Numbers below were computed
from the closed-form step response (x₀ = 0, v₀ = 0, target 1):
ω₀ = √(k/m), ζ = c / (2√(km)), t95 = first time x ≥ 0.95, settle = last time |x − 1| > tolerance.

| Token | k | c | m | ζ | ω₀ (rad/s) | Overshoot | t95 | Settle ±2 % | Settle ±0.1 % (CSS duration) | Character |
|---|---|---|---|---|---|---|---|---|---|---|
| `spring.snap` | 500 | 45 | 1 | 1.006 | 22.36 | 0 % | 214 ms | 264 ms | 420 ms | critically damped (ζ ≈ 1.006); no overshoot |
| `spring.soft` | 260 | 26 | 1 | 0.806 | 16.12 | 1.38 % | 212 ms | 236 ms | 528 ms | under-damped; 1.4 % overshoot, one soft return |
| `spring.follow` | 170 | 26 | 1 | 0.997 | 13.04 | 0 % | 362 ms | 444 ms | 702 ms | critically damped, slower; data followers |
| `spring.calm` | 120 | 22 | 1 | 1.004 | 10.95 | 0 % | 436 ms | 536 ms | 854 ms | critically damped, slowest; ambient energy |

`spring.snap` and `spring.soft` are the composer.md springs, unchanged. `spring.follow` and `spring.calm`
give names to springs the specs already used without naming them (composer-context-meter.md and
thinking-orb.md). All four keep **m = 1**.

### 4.1 Nominal duration: one spring, many speeds
A spec row like "160 ms `spring.snap`" or "220 ms `spring.snap`" asks for the spring's *character*
(its ζ) at a *nominal duration* d. The solver scales the spring so that its **t95 equals d**, keeping ζ:

```
ω₀' = ω₀ · t95_ref / d        k' = m · ω₀'²        c' = 2 · ζ · m · ω₀'
```

Examples: `spring.snap` at 160 ms → k 894, c 60.2. At 220 ms → k 473, c 43.8. `spring.soft` at 240 ms →
k 203, c 23.0. At 320 ms → k 114, c 17.2. A row that names a spring with no duration uses the reference
spring (t95 = the `$value` in the JSON). Nominal durations must stay in **120–440 ms**. The time to
*fully* settle is longer than d (×1.96 for snap/follow/calm, ×2.49 for soft). Choreography (§7) sequences
on d, not on full settle.

### 4.2 Spring consolidation
The specs contain 12 unnamed springs. Each one maps onto a named spring:

| Where | Spring as written | ζ | Overshoot | t95 | Maps to | Why |
|---|---|---|---|---|---|---|
| composer-attachments tile grow | k 320, c 28, m 1 | 0.783 | 1.93 % | 184 ms | `spring.soft` | ζ 0.78 vs 0.81 and 2 % vs 1.4 % overshoot are not distinguishable at tile size |
| composer-attachments reorder settle | k 380, c 30, m 1 | 0.769 | 2.27 % | 166 ms | `spring.snap` | reorder is reflow; reflow never overshoots |
| composer-context-meter arc tween | k 170, c 26, m 1 | 0.997 | 0 % | 362 ms | `spring.follow` | identical values; this spring becomes the named follower |
| composer-input-shell paste fold | k 320, c 36, m 1 | 1.006 | 0 % | 268 ms | `spring.snap` | same ζ (1.006); only speed differs, which the nominal duration carries |
| composer-input-shell hero-dock | k 260, c 30, m 1 | 0.93 | 0.03 % | 262 ms | `spring.soft @ 320 ms` | one-time expressive move; ζ 0.93 → 0.81 adds a barely visible settle |
| composer-model-picker press | k 600, c 30, m 1 | 0.612 | 8.77 % | 106 ms | `spring.snap` | 8.8 % overshoot is a bounce; presses in Oxyn do not bounce |
| composer-slash-mentions reorder | k 500, c 40, m 1 | 0.894 | 0.19 % | 178 ms | `spring.snap` | reflow |
| composer-slash-mentions/tools-menu highlight | k 600, c 40, m 1 | 0.816 | 1.18 % | 143 ms | `spring.snap` | highlight glide is a morph; overshoot past a row reads as mis-selection |
| composer-tools-menu plus-to-x | k 400, c 40, m 1 | 1.0 | 0 % | 238 ms | `spring.snap` | ζ 1.0 already |
| icon-states level bars | k 220, c 20, m 0.6 | 0.87 | 0.39 % | 200 ms | `spring.follow` | level-driven value follower |
| thinking-orb/voice-orb energy | k 120, c 22, m 1 | 1.004 | 0 % | 436 ms | `spring.calm` | identical values; this spring becomes the named calm follower |
| voice-orb radius | k 300, c 28, m 1 | 0.808 | 1.34 % | 198 ms | `spring.soft` | k 300 c 28 is ζ 0.81, the same character as spring.soft |

### 4.3 Solver (in `oxyn-core`, frame-rate independent)
- **Closed form, not integration.** Each animated value stores `(x₀, v₀, t₀, target, k, c, m)`, and
  the position at clock time t is evaluated analytically:
  - under-damped (ζ < 1), ω_d = ω₀√(1−ζ²), Δ = x₀ − target:
    `x(t) = target + e^(−ζω₀τ) · (Δ·cos ω_dτ + ((v₀ + ζω₀Δ)/ω_d)·sin ω_dτ)`, τ = t − t₀
  - critically damped (|ζ − 1| < 0.01, treated as 1): `x(t) = target + (Δ + (v₀ + ω₀Δ)τ) · e^(−ω₀τ)`
  - over-damped: the two-root form. Velocity is the analytic derivative.

  The result is the same at 30, 60 or 120 Hz and after a dropped frame. There is no accumulated error and no dt
  clamping.
- **Rest detection:** a spring is at rest when |x − target| < 0.5 px (or 0.001 for unitless values)
  and |v| < 1 px/s. At rest it snaps to target and releases its clock lease, so a settled UI repaints
  nothing.
- **Retargeting** (§7.3): a new target starts a new segment with x₀ = current x(t), v₀ = current
  v(t), t₀ = now. The element never jumps and never loses momentum.
- **API sketch (proposed):** `Spring { stiffness, damping, mass }`,
  `Spring::with_nominal(Duration) -> Spring`, `SpringValue::retarget(target, now)`,
  `SpringValue::sample(now) -> (x, v)`, `SpringValue::is_at_rest()`.

## 5. Loops

| Token | Period | Curve | Shape | Use |
|---|---|---|---|---|
| `loop.brisk` | 800 ms (700–900) | linear | rotation | tool running (= `loop.base` ÷ 1.25) |
| `loop.base` | 1000 ms (1000–1200) | linear | rotation / sweep | default spinners, indeterminate rings |
| `loop.calm` | 2000 ms (1600–2400) | `ease.sine` | breathe | thinking, idle |
| `loop.glow` | 3200 ms | `ease.sine` | intensity 0.7 ↔ 1 | edge glow, conic borders, voice listening |
| `loop.dormant` | 6000 ms | `ease.sine` | breathe | asleep / dormant presence |
| `loop.pulse` | 2000 ms | `ease.sine` | opacity 0.45 ↔ 1 | skeleton pulse |
| `loop.wave` | 2000 ms | `ease.sine` | 150 ms per bar, cap 8 | skeleton wave |
| `loop.sweep` | 2000 ms | linear | 105° band | skeleton shimmer |
| `loop.scan` | 1200 ms pass + 200 ms rest | `ease.in-out` | top → bottom | analysis scan line |
| `loop.beam` | clamp(perimeter ÷ 400 px/s, 1.6 s, 4.0 s) | linear in arc length | travelling head | border beam |
| `loop.noise` | evolves at 30 fps, seed drift 0.2 cells/s | linear | noise | generation placeholder |
| `loop.shimmer.default` | 1400 sweep + 800 rest (2200), after `delay.shimmer` 600 | `ease.coast` | text band | status text, tool args, active plan item |
| `loop.shimmer.calm` | 1000 sweep + 3000 rest (4000), after 600 | linear | text band | status strip verb (Codex cadence) |
| `loop.waiting` | 2000 ms, 300 ms turn | `ease.in-out` | tip / rotate | awaiting-you indicator |

Rules:
- **Phase is computed, not stepped:** `phase = (∫ speed dt) / period mod 1`, with the origin at clock
  zero. Every instance of a variant is in phase, and a speed change never jumps (loaders.md §2.1).
- **Tempo changes** ease over `dur.tempo.change` (400 ms, `ease.in-out`) on the speed, not the phase.
- **Stall** slows a loop to 0.5× over `dur.stall.ramp` (2000 ms, linear, colour in OKLab). Recovery
  takes `dur.stall.recover` (200 ms, `ease.out`).
- **Ending a loop:** finish the current half-cycle at 2× speed, or fade the amplitude to 0 over
  `dur.base`. Never cut mid-breath.
- Breathing loops alternate half-cycles of `ease.sine`. Rotations and sweeps are linear. A breathing loop
  never uses `ease.in-out`, because its sharper acceleration reads as nervous at 2–3 s.
- Loops run at most 60 fps (`motion.fps.max`). Text effects run at 30 fps (`motion.fps.text-effects`).

## 6. Stagger

| Token | Step | Cap | Use |
|---|---|---|---|
| `stagger.base` | 40 ms | 6 items | rows, chips, cards, sections entering together |
| `stagger.tight` | 30 ms | 8 items | bulk cascades, word blur-in, tray rows, voice bars |
| `stagger.fold` | 20 ms | 6 rows | rows folding into one (turn fold, rewind fold, checks collapse) |
| `stagger.digit` | 30 ms | — | digits rolling, right → left |
| `stagger.glyph` | 25 ms | total ≤ 300 ms | per-glyph effects (scramble, split-flap, reveal) |

Rules:
1. **Items beyond the cap arrive together** with the last staggered item. The total spread is never more than
   240 ms (40 × 6), so a burst of 50 rows doesn't play for two seconds.
2. Stagger runs in **reading order** (top → bottom; in RTL, right → left within a row). Digits are the
   exception: they go right → left, from the least significant digit.
3. Exits are **not** staggered, except folds (`stagger.fold`), where the order shows where rows went.
4. Items that arrive inside one batch window (30 ms) stagger as a group. Items that arrive later start
   their own stagger. They never wait for the previous one to finish.
5. Reduced motion: no stagger.
6. Aliases: `stagger.row` → `stagger.base`; `stagger.chips` → `stagger.base` (cap 5 → 6);
   `stagger.bulk` → `stagger.tight`.

## 7. Choreography

### 7.1 Enter / exit asymmetry
- An exit is **shorter than its enter, about ⅔**, and uses `ease.in`. The canonical pairs:
  200 / 160 (`dur.surface.*`), 180 / 120 (`dur.popover.*`), 220 / 160 (`dur.sheet.*`),
  120 / 160 actions (the hide is longer only because it follows a `delay.leave` grace and is never
  the user's own action).
- A **staged collapse** may be longer in total than its expand, because it does two things in sequence. The
  content fades out first (100 ms) so the text never squashes, and then the height closes (180 ms).
  That makes `dur.collapse` 280 ms against `dur.expand` 220 ms. Each segment is still shorter than the
  enter.
- Enters that push content (an in-flow row) animate **height** with `spring.snap` and fade content with
  `ease.out` (`dur.row.enter`). Enters that push nothing (chips appended at the end, cards into
  reserved space) only fade and rise (`dur.chip.enter`).
- An enter that is cancelled before it finishes reverses from where it is, with the exit curve. It
  never completes the enter first.

### 7.2 Sequencing
- A **beat** between two linked steps (loader → ✓ → tense roll) is ≤ 200 ms. Steps that follow each other start on the
  previous step's *nominal* duration, not its full settle.
- At most **two** properties lead an enter (for example, height then opacity). More than that reads as busy.
- Layout changes anchor the **top edge** and keep the user's reading position. The bottom edge of
  the composer and the send control never move (composer.md).
- Shared-element travel (composer → transcript, chip → pane) uses `dur.travel` with `ease.in-out` (or
  `ease.in` for a fly-up that leaves the view), capped at 260 ms. Beyond 2000 px it doesn't travel.
  It crossfades instead.

### 7.3 Interruption and retargeting
- **Springs retarget from their current position and velocity** (§4.3). A row that is expanding when
  the user clicks collapse turns around smoothly, and a progress bar following new data keeps its
  momentum.
- **Tweens** (cubic-bezier) that are interrupted start a new tween from the current *presentation*
  value. The new duration is the token's duration × the remaining distance fraction, with a minimum of
  `dur.instant`. They never jump to the end, and they never replay from the start.
- **Loops** are never restarted by a state update. Speed and amplitude change; the phase is continuous.
- **Never queue animations.** If the next state arrives mid-animation, retarget to it (composer-queue-stack.md rule).
- A user action always wins over a running agent animation on the same element.

### 7.4 Rolls and swaps
Numbers **roll** (`dur.digit.roll`, 280 ms, `ease.out`, `stagger.digit`), and a roll must finish
inside the value's update interval (< 1 s for a 1 Hz timer). Words **swap** (`dur.verb.swap`). Large
first reveals count up (`dur.count.up`, 500 ms).

## 8. Reduced motion

Triggered by `motion.reduced`: the OS setting (`prefers-reduced-motion`, macOS / Windows / GNOME
reduce-motion) or the app's "Pause animations" switch. Every token has a reduced form, listed in the
**Reduced form** column of the [Registry](#13-registry). The general rules:

| Token kind | Reduced form |
|---|---|
| Any transition (`dur.*` with `ease.*`) | opacity crossfade at `dur.instant` (80 ms); no translate, scale or blur |
| Spring-driven size / position | jumps to the target; content crossfades at `dur.instant` |
| Height changes | jump once, with no animation |
| Rolls (`dur.digit.roll`, `dur.count.*`) | the value swaps without rolling |
| Stroke draws (`dur.check.draw`) | the glyph appears whole |
| Stagger | none; everything arrives together |
| Decorative loops (`loop.beam`, `loop.sweep`, `loop.glow`, `loop.noise`, `loop.shimmer.*`, `loop.wave`, `loop.scan`) | **static**: a designed still frame or tone, not frame 0 |
| Liveness loops (`loop.calm`, `loop.base`, `loop.brisk`, `loop.pulse`) | static still frame. Opt-in `SlowFade`: opacity 1 ↔ 0.6 at **0.5 Hz** (2 s period). Liveness otherwise comes from a 1 Hz elapsed timer. |
| Highlights (`dur.highlight.fade`, `dur.flash.target`) | a static accent (a 2 px bar for 4 s, or a tint for the hold), then removed |
| Information-carrying motion (audio level, progress, meters) | **kept** as static bars or rings updated at ≤ 10 Hz |
| Safety timings (`dur.approval.arm`, `dur.approval.hold`, `delay.approval.input-guard`) | **duration kept**. Arming shows "Available in 0.6 s". The hold shows a stepped 4-segment fill at 4 Hz. |
| Holds that carry meaning (`dur.copy.confirm`, `dur.beat`) | kept |
| Delays (`delay.*`) | unchanged; they aren't motion |
| `dur.verb.dwell` | doubled |

Under reduced motion, rotation-based and parallax motion never appears, even when the app opts in to
`SlowFade`.

## 9. Low power, background and hidden windows

All motion runs from the one `MotionClock` per window. Rate policy (from loaders.md §2.1, applied here
to all motion, not just loaders):

| Window / system state | Transitions | Glyph frames | Vector / shader loops | Particle loops |
|---|---|---|---|---|
| Focused, visible | ≤ 60 fps (`motion.fps.max`) | native frame ms | ≤ 60 fps | ≤ 60 fps |
| Visible, unfocused | ≤ 60 fps (they are short) | native | ≤ 20 fps (`motion.fps.unfocused`) | ≤ 15 fps |
| Low-power / battery saver | ≤ 60 fps; springs unchanged (closed form) | native | ≤ 30 fps (`motion.fps.low-power`) | ≤ 30 fps, half particles; shaders fall back to particle tier |
| Hidden, minimised, occluded, or scrolled offscreen | **not run**: they jump to their end state | **paused** | **paused** (GPU resources kept 30 s, then freed) | paused |
| Nothing animating | — | — | — | clock idles at 1 Hz for timers (`motion.fps.idle`) |

- **Pause, don't catch up.** When a paused loop becomes visible, it draws at the phase the clock says
  *now*. There is no catch-up animation and no restart from frame 0. Transitions that would have run
  while hidden are skipped.
- **120 Hz displays** don't raise loop rates above 60 fps. Direct manipulation (drag, scrub)
  follows the pointer at display rate.
- **Repaint only when something changes.** An 80 ms glyph spinner repaints at 12.5 Hz, not 60 Hz.
  A settled spring releases its lease.
- Text shimmer pauses while the user is selecting text and while the window is unfocused, and resumes at its phase.

## 10. Photosensitivity

Required, not proposed (WCAG 2.2 SC 2.3.1, Three Flashes or Below Threshold):

1. **No more than 3 flashes in any 1-second period** (`motion.flash.max-per-second`), in any variant, tier or
   theme. A "flash" is a pair of opposing luminance changes of ≥ 10 % of relative luminance where the
   darker state is below 0.80, over an area larger than the WCAG small-area threshold.
2. **No saturated red flashing** at all.
3. **Repeating luminance changes** have a period ≥ 1 s (`motion.loop.luminance-min-period`). Glows and
   rims ease; they never blink or strobe.
4. **Single completion flashes** change luminance by ≤ 0.25 (`motion.flash.max-luminance-delta`) and
   play once.
5. **Audio-reactive visuals** (voice orb, visualisers, captions) drive **shape and size, not
   brightness**. Brightness modulation is capped at **±10 %** (`motion.audio.luminance-max`) and
   low-passed at **3 Hz** with a one-pole filter (`motion.audio.luminance-lowpass`). Onset rings play
   once per turn.
6. **Area rule:** no loader changes the luminance of an area larger than its own box. Full-window
   tints (take-over, stall) ramp over ≥ 200 ms and never pulse.
7. Studio runs a flash check (luminance sampled per frame, 1 s window) on every variant. A variant that
   fails isn't published.

## 11. Implementation notes

### 11.1 `oxyn-core` (shared by both renderers)
- `motion::tokens` is generated from `tokens/oxyn.motion.json` at build time: `const` durations, `Easing`
  values, `Spring` values and `Loop` descriptors. Tokens are never hand-copied.
- `Easing::CubicBezier(x1, y1, x2, y2)` is evaluated with Newton–Raphson on x (4 iterations) and
  falls back to bisection. `Easing::Coast` uses the exact piecewise form. `Easing::Linear`.
- `Spring` and `SpringValue` (§4.3) are closed form and frame-rate independent, with retargeting.
- `MotionClock` gives monotonic ms, the rate policy (§9), leases, and the `motion.reduced` flag. Every
  preset exposes `fn reduced(&self) -> ReducedForm`.

### 11.2 GPUI (desktop, primary)
- Tweens: GPUI's element animation (`Animation::new(duration).with_easing(f)` with
  `.with_animation(id, animation, |el, t| …)`). The easing argument is a plain `Fn(f32) -> f32`, so
  `oxyn_core::motion::ease_out` and the others plug in directly. Use `.repeat()` only for loops that
  the shared clock doesn't drive. Prefer the clock so instances stay phase-locked.
- Springs: GPUI's `Animation` is duration-based, so springs don't use it. A spring-driven element
  samples its `SpringValue` from the `MotionClock` at paint time and requests the next frame while it
  isn't at rest. The frame-request API is to be confirmed at implementation, as loaders.md says.
  Retargeting is a method call on the stored value. Nothing is rebuilt.
- Reduced motion: `MotionClock::reduced()` makes every preset return its reduced form, so components
  never branch on the flag themselves.
- Rate policy: unfocused, occluded and minimised windows come from the window's activation and
  visibility state. Leases are paused there.

### 11.3 Dioxus (web) / CSS
Tokens become custom properties on `:root`, generated from the same JSON:

```css
:root {
  --oxyn-dur-instant: 80ms;  --oxyn-dur-fast: 120ms;  --oxyn-dur-base: 180ms;  --oxyn-dur-travel: 240ms;
  --oxyn-ease-out: cubic-bezier(0.22, 1, 0.36, 1);
  --oxyn-ease-in: cubic-bezier(0.4, 0, 1, 1);
  --oxyn-ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);
  --oxyn-ease-firm: cubic-bezier(0.2, 0, 0, 1);
  --oxyn-ease-sine: cubic-bezier(0.37, 0, 0.63, 1);
  --oxyn-ease-coast: cubic-bezier(0.25, 0, 0.75, 1);
  /* springs: linear() sampled from the closed form over the ±0.1 % settle time */
  --oxyn-spring-snap: linear(0, 0.0236, 0.081, 0.157, 0.2412, 0.3269, 0.4098, 0.4872, 0.558, 0.6214, 0.6775, 0.7266, 0.7691, 0.8057, 0.8371, 0.8637, 0.8863, 0.9054, 0.9214, 0.9348, 0.946, 0.9554, 0.9632, 0.9696, 0.975, 0.9794, 0.9831, 0.9861, 0.9886, 0.9906, 0.9923, 0.9937, 0.9949, 0.9958, 0.9966, 0.9972, 0.9977, 0.9981, 0.9985, 0.9988, 1);
  --oxyn-spring-snap-duration: 420ms;   /* reference spring; nominal d → d × 1.96 */
  --oxyn-spring-soft: linear(0, 0.0202, 0.072, 0.1441, 0.228, 0.3171, 0.4063, 0.4923, 0.5727, 0.6461, 0.7117, 0.7692, 0.8189, 0.861, 0.8962, 0.9252, 0.9486, 0.9671, 0.9815, 0.9925, 1.0005, 1.0062, 1.01, 1.0123, 1.0135, 1.0138, 1.0135, 1.0128, 1.0118, 1.0106, 1.0094, 1.0081, 1.007, 1.0059, 1.0048, 1.0039, 1.0032, 1.0025, 1.0019, 1.0014, 1);
  --oxyn-spring-soft-duration: 528ms;   /* nominal d → d × 2.49 */
}
@supports (transition-timing-function: linear(0, 1)) {
  :root { --oxyn-ease-coast: linear(0, 0.0078, 0.0312, 0.0703, 0.125, 0.1875, 0.25, 0.3125, 0.375, 0.4375, 0.5, 0.5625, 0.625, 0.6875, 0.75, 0.8125, 0.875, 0.9297, 0.9688, 0.9922, 1); }
}
@media (prefers-reduced-motion: reduce) {
  :root { --oxyn-motion-reduced: 1; }
  /* components switch to opacity-only transitions at var(--oxyn-dur-instant); loops: animation: none */
}
:root[data-oxyn-hidden] *, :root[data-oxyn-reduced] .oxyn-loop { animation-play-state: paused; }
```

- **Springs in CSS** use `linear()` with the token's duration (spring time scales linearly with nominal
  duration, so one `linear()` curve serves every speed: duration = d × the ratio above). Browsers without
  `linear()` get `--oxyn-ease-out` (for `spring.snap`) through an `@supports` fallback.
- **CSS transitions retarget from the current value but lose velocity.** Anything that must keep
  momentum (drag release, data followers using `spring.follow`, rapid re-toggles) runs the `oxyn-core`
  solver through wasm on `requestAnimationFrame` and writes `transform` / `opacity` directly, or uses
  WAAPI with `KeyframeEffect` built from solver samples.
- **Loops** are CSS keyframes (transform/opacity only, so the compositor runs them). They are
  phase-locked to the shared clock with a negative `animation-delay` = −(clock ms mod period)
  (loaders.md §2.2).
- **Hidden tabs:** browsers already throttle `requestAnimationFrame` in background tabs. Oxyn also
  sets `data-oxyn-hidden` on `visibilitychange`, which pauses CSS loops so they don't run, and resumes
  them at the clock phase. An `IntersectionObserver` pauses offscreen loops.
- **Low power:** browsers don't expose battery saver reliably. Apps may set `data-oxyn-low-power`,
  which applies the low-power column of §9.
- **Terminal tier:** no easing. Transitions are frame-quantised to the terminal repaint, and loops use glyph
  frames only.

## 12. Conflicts found and decisions

| # | Conflict | Decision | Reason |
|---|---|---|---|
| C1 | `dur.row.enter`: **180 ms, y +4** (messages.md) vs **200 ms height spring + 160 ms fade, y +4** (tools.md) | `dur.row.enter` = tools.md definition. The messages.md motion becomes the new **`dur.chip.enter`** (180 ms fade + rise, no height). | In-flow rows push content and need the height spring. Chips and cards appended into reserved space don't. Twelve specs already quote the tools.md definition. |
| C2 | `dur.collapse`: **200 ms height spring** (messages.md, reasoning-block, streaming-markdown, message-bubble) vs **100 ms fade + 180 ms `ease.in` = 280 ms** (tools.md, diff-viewer, tool-call) | `dur.collapse` = **280 ms staged** (tools.md). Transcript disclosures use `dur.expand` (220 ms `spring.snap`) to open and `dur.collapse` to close. | Reasoning blocks and tool rows sit in the same turn, so they must open and close alike. The staged fade stops text squashing. |
| C3 | `dur.base` 160–200 ms range | **180 ms** default; range 160–200 allowed | Already the quoted default in loaders.md / container-loaders.md. |
| C4 | `dur.travel` 220–260 ms range | **240 ms** default; 260 hard cap | Most common travel value; the cap is kept. |
| C5 | `dur.instant` 0–80 ms | **80 ms** default | Every reduced-motion rule says "80 ms crossfade". |
| C6 | `loop.calm` 1600–2400, `loop.base` 1000–1200, `loop.brisk` 700–900 | **2000 / 1000 / 800 ms** | 800 = 1000 ÷ 1.25 keeps loaders.md's stated relation exact. The 1.2 s indeterminate rings sit inside `loop.base`'s range. |
| C7 | `dur.approval.in`: card height on **`ease.out`** (approvals.md §5.2) vs **`spring.snap`** (approval-card.md, tools.md footer) | `dur.approval.in` becomes an **alias of `dur.row.enter`** (height `spring.snap` + 160 ms fade) | An approval card is a transcript row, and a burst of approvals must retarget without jumps. |
| C8 | Hold-to-confirm: **1200 ms** (`dur.approval.hold`, permission selector) vs **800 ms** (rollback-impact.md) | **1200 ms everywhere** (`dur.approval.hold`) | One hold length across the library builds one piece of muscle memory. A shorter hold on a destructive sheet teaches the wrong rhythm. |
| C9 | Hold drain on early release: **`ease.in`** (approvals.md) vs **`ease.out`** (rollback-impact.md) | **`ease.out`**; token renamed `dur.approval.drain` | The drain answers the user's release, and responses to input start fast. `ease.in` reads as lag. |
| C10 | `dur.settle` (success fade of edge layers): **`ease.out`** (container-loaders.md) vs done fade **240 ms `ease.in`** (loaders, loader-vector, tools, plugin-icons, file-edit-chip) | `dur.settle` = 240 ms **`ease.in`** | It's an exit of a transient layer, and five specs agree. |
| C11 | Copy confirm hold: **1500 ms** (`dur.copy.confirm`, streaming-markdown, icon-states) vs **1200 ms** (tool-call, git-chips, live-captions) | **1500 ms** | It is the named token. icon-states.md reserves 1200 ms for *generic* one-shots, not copy. |
| C12 | Preview delay: **300 ms** (`delay.preview.open`, citations) vs **200 ms** (`delay.preview`, checkpoints-git, run-replay, checkpoint-timeline, diff-viewer minimap) | **300 ms**; `delay.preview` → `delay.preview.open` | One hover-intent rule. Scrubbing between adjacent markers swaps content with no delay once a preview is open (citations-sources rule), so timelines don't pay the 300 ms twice. |
| C13 | `show_delay` = **300 ms** (family B loaders) vs **150 ms** (family D/I containers, skeletons, jobs, trace); `min_visible` 600 vs `min_display` 300 | Split and renamed: `delay.loader.show` 300 / `dur.loader.min-visible` 600 for free-standing indicators; `delay.container.show` 150 / `dur.container.min-visible` 300 for containers and skeletons | A free-standing loader inserts into the transcript, so a longer delay hides most quick turns. A container loader decorates something already on screen and a skeleton reserves layout, so they can appear sooner. |
| C14 | Text shimmer: **1400 ms sweep / 800 ms rest** (status-text, tools, plan-todo, review, merge) vs **1.0 s sweep every 4.0 s** (composer-status-strip) | Both kept as named cadences: `loop.shimmer.default` and `loop.shimmer.calm` (the strip's value = text-shimmer's `calm`) | They are two documented cadences, not a typo. The strip is a persistent surface and should be calmer. |
| C15 | `dur.context.expand` 180 ms vs `dur.expand` 220 ms (both `spring.snap`) | Alias of **`dur.expand`** | Same disclosure physics. 40 ms doesn't justify a token. |
| C16 | Digit / number rolls: 280 ms (`dur.digit.roll`) vs 300 ms (status strip, pr-card), 320 ms (context meter chip, model picker), 400 ms `spring.snap` (genui number roll), 180 ms (voice latency), 560 ms (context meter popover) | **280 ms `ease.out`** for every roll; first-reveal / large count-ups use new **`dur.count.up` 500 ms** | Numbers next to each other must move together, and a roll has to fit inside a 1 s tick. |
| C17 | Agent-change highlight: **1600 ms** (`dur.highlight.fade`) vs **1200 ms** agent-update wash (genui, genui-renderer) | **1600 ms** | It means the same thing ("the agent changed this"), so it gets one duration. |
| C18 | Breathe curves: "sine", "cosine", `ease.in-out` for `loop.calm/glow/pulse` | New **`ease.sine`** for all breathing loops | Lower peak acceleration, so it reads calmer, and one curve for one kind of motion. |
| C19 | Scan line: **1.2 s + 200 ms rest** (`loop.scan`, media-analysis) vs **1.6 s + 400 ms rest** (genui-skeletons, proposed) | `loop.scan` (1.2 s + 200 ms) | The named token, and composer-attachments.md agrees. |
| C20 | Awaiting-you tip: **300 ms turn every 2000 ms** (status strip, subagent-tree) vs **600 ms every 2000 ms** (loader-vector) | **300 ms** (`loop.waiting`) | Majority, and more rest per cycle is calmer. |
| C21 | Stagger caps: `stagger` 30–40 ms cap 4–6, `stagger.chips` cap 5, `stagger.row` cap 6, `stagger.bulk` 30 / 8 | `stagger.base` 40 / 6, `stagger.tight` 30 / 8, `stagger.fold` 20 / 6 | Three cadences cover every use. A cap of 5 vs 6 isn't a meaningful difference. |
| C22 | 12 unnamed springs (k/c pairs) across composer, icon and voice specs | Mapped onto `spring.snap / soft / follow / calm` (§4.2). Speed is carried by the nominal duration (§4.1). | Four characters and one speed rule are enough. The model-picker press bounce (8.8 % overshoot) is dropped on purpose. |
| C23 | Name clashes in a nested token tree: `dur.reveal` vs `dur.reveal.token`; `dur.approval.hold` vs `.hold.drain`; `delay.preview` vs `.preview.open`; bare `stagger` vs `stagger.*` | Renamed (`dur.reveal.media`, `dur.approval.drain`, `delay.preview.open`, `stagger.base`) | DTCG can't hold a name that is both a token and a group. |
| C24 | Off-namespace motion names: `highlight.flash`, `frame.glyph`, `fps.*`, `voice.*` motion tokens, `delay.*_snake` | Moved into `dur.*` / `motion.*` / kebab-case (Migration) | One namespace per kind. |
| C25 | Toast / banner exit: 160 ms (most) vs 180 ms (takeover-banner) | `dur.surface.exit` 160 ms | The ⅔ rule, and the majority. |

## 13. Registry

Every motion token found in `docs/specs/*.md` (grep for `dur.`, `loop.`, `ease.`, `spring.`, `motion.`,
`delay.`, `stagger`, `frame.`, `fps.`, `voice.`, `highlight.`), plus the names this document adds.
165 rows. **Status:** *canonical*, *alias of X* (kept, resolves to X), *renamed → X* (old name,
deprecated; the JSON keeps it as a `$deprecated` alias where the tree allows), or *behavioural* (a
timing that isn't motion). "Used by" lists the spec files (without `.md`) that name the token or
use its exact value.

#### Primitive durations

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `dur.instant` | 80 ms (range 0–80) | ease.linear / ease.out | composer, container-loaders, icon-states, icons, loaders, status-text | canonical | is the reduced form |
| `dur.fast` | 120 ms (range 100–140) | ease.out | composer, container-loaders, icons, loader-glyph, status-text, subagent-tree | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.base` | 180 ms (range 160–200) | ease.out | animated-marks, composer, container-loaders, genui-renderer, git-chips, loaders, status-text | canonical — was a 160–200 range | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.travel` | 240 ms (range 220–260, hard cap 260) | ease.in-out | approval-card, approvals, background-jobs, checkpoint-timeline, checkpoints-git, citations-sources, command-palette, composer, container-loaders, genui, genui-renderer, live-voice, merge-conflict, permission-rules, status-text, subagent-tree, terminal-output, text-reveal, trace-view, voice-control-bar, workspace-chrome | canonical — was a 220–260 range | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.pulse` | 400 ms (range 300–400) | ease.out | loaders, loader-glyph, loader-vector, composer-status-strip, subagent-tree, trace-view, border-beam, container-loaders, media-analysis-scan, fanout-grid, background-jobs, permission-rules, citations-sources, run-meter | canonical (new name for the recurring single 400 ms pulse / completion pass) | no pulse; state shown by tone/badge |
| `dur.beat` | 600 ms (range 400–800) | — (hold) | loaders, loader-glyph, loader-vector, tools, tool-call, plugin-icons, file-edit-chip, border-beam, plan-todo, diff-viewer, checkpoint-timeline, side-panels, mcp-plugin-list, command-palette | canonical (new name for the recurring 600 ms confirmation hold) | kept (a hold carries meaning) |

#### Easings

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `ease.out` | cubic-bezier(0.22, 1, 0.36, 1) | — | 69 of 79 specs (nearly every timing table) | canonical | linear crossfade |
| `ease.in` | cubic-bezier(0.4, 0, 1, 1) | — | 58 of 79 specs (nearly every timing table) | canonical | linear crossfade |
| `ease.in-out` | cubic-bezier(0.65, 0, 0.35, 1) | — | 58 of 79 specs (nearly every timing table) | canonical | linear crossfade |
| `ease.firm` | cubic-bezier(0.2, 0, 0, 1) | — | border-beam, composer, container-loaders, live-captions, live-voice, loader-mascot, mcp-plugin-list, media-analysis-scan, message-bubble, reasoning-block, status-text, thinking-orb, voice-orb, voice-visualisers | canonical | linear crossfade |
| `ease.linear` | cubic-bezier(0, 0, 1, 1) | — | 71 of 79 specs (nearly every timing table) | canonical | linear crossfade |
| `ease.sine` | cubic-bezier(0.37, 0, 0.63, 1) | — | composer-context-meter, composer-model-picker, composer-status-strip, composer-voice-input, text-shimmer, voice-orb, loaders, container-loaders, genui-blocks | canonical (new; replaces bare “sine” in breathe loops) | static |
| `ease.coast` | quadratic ramps on first/last 20 %, linear between; CSS ≈ cubic-bezier(0.25, 0, 0.75, 1) (max err 0.0095) or exact linear() | — | merge-conflict, plan-todo, reasoning-block, review-card, status-text, text-shimmer, tool-call, tools | canonical | shimmer off |

#### Springs

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `spring.snap` | k 500, c 45, m 1 · ζ 1.006 · ω₀ 22.36 rad/s · overshoot 0 % · t95 214 ms · settle 2 % 264 ms / 0.1 % 420 ms | — | 64 of 79 specs (nearly every timing table) | canonical | crossfade `dur.instant`; value followers jump |
| `spring.soft` | k 260, c 26, m 1 · ζ 0.806 · ω₀ 16.12 rad/s · overshoot 1.38 % · t95 212 ms · settle 2 % 236 ms / 0.1 % 528 ms | — | animated-marks, citations-sources, command-palette, composer, composer-queue-stack, composer-voice-input, container-loaders, desktop-overlays, genui-blocks, genui-renderer, icon-states, live-voice, loader-mascot, loader-vector, message-bubble, side-panels, status-text, terminal-tabs, verb-cycler, voice-control-bar | canonical | crossfade `dur.instant`; value followers jump |
| `spring.follow` | k 170, c 26, m 1 · ζ 0.997 · ω₀ 13.04 rad/s · overshoot 0 % · t95 362 ms · settle 2 % 444 ms / 0.1 % 702 ms | — | composer-context-meter, icon-states | canonical (new name for an unnamed spring in the specs) | crossfade `dur.instant`; value followers jump |
| `spring.calm` | k 120, c 22, m 1 · ζ 1.004 · ω₀ 10.95 rad/s · overshoot 0 % · t95 436 ms · settle 2 % 536 ms / 0.1 % 854 ms | — | thinking-orb, voice-orb | canonical (new name for an unnamed spring in the specs) | crossfade `dur.instant`; value followers jump |

#### Loops

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `loop.brisk` | 800 ms (range 700–900); = loop.base ÷ 1.25 | ease.linear | animated-marks, loaders | canonical — was 700–900 range | static still frame (no loop) |
| `loop.base` | 1000 ms (range 1000–1200) | ease.linear | loaders, message-bubble, plugin-icons | canonical — was 1000–1200 range | static still frame (no loop) |
| `loop.calm` | 2000 ms (range 1600–2400) | ease.sine | animated-marks, genui-blocks, loader-mascot, loader-vector, loaders, pr-card, tools, voice-visualisers | canonical — was 1600–2400 range; curve “sine / ease.in-out” → ease.sine | static; opt-in `SlowFade` 0.5 Hz opacity 1 ↔ 0.6 |
| `loop.glow` | 3200 ms breathe, intensity 0.7 ↔ 1 | ease.sine | acting-indicator, border-beam, container-loaders, voice-orb | canonical — curve ease.in-out → ease.sine | static edge tone |
| `loop.dormant` | 6000 ms breathe | ease.sine | voice-orb, live-voice | canonical (new name for the voice-orb asleep breathe) | static poster |
| `loop.pulse` | 2000 ms, opacity 0.45 ↔ 1 | ease.sine | container-loaders, genui-skeletons, skeletons | canonical — curve ease.in-out → ease.sine | static skeleton tone |
| `loop.wave` | 2000 ms, 150 ms per bar, cap 8 bars | ease.sine | skeletons, genui-skeletons | canonical (new name for skeleton wave) | static skeleton tone |
| `loop.sweep` | 2000 ms per pass, 105° band | ease.linear | container-loaders, fanout-grid, skeletons | canonical | static skeleton tone |
| `loop.scan` | 1200 ms pass + 200 ms rest (period 1400 ms) | ease.in-out | composer-attachments, container-loaders, genui-skeletons, media-analysis-scan | canonical — rest made explicit | static line at last reported position |
| `loop.beam` | lap = clamp(perimeter ÷ 400 px/s, 1.6 s, 4.0 s) | ease.linear (in arc length) | border-beam, container-loaders | canonical | static border tone |
| `loop.noise` | 30 fps evolution, seed drift 0.2 cells/s | ease.linear | container-loaders, media-generation-placeholder | canonical | static noise poster |
| `loop.shimmer.default` | sweep 1400 ms + rest 800 ms (period 2200), delay 600 ms | ease.coast | merge-conflict, plan-todo, reasoning-block, review-card, status-text, text-shimmer, tool-call, tools | canonical (composite of dur.shimmer.*) | shimmer off, plain text at full fg |
| `loop.shimmer.calm` | sweep 1000 ms + rest 3000 ms (period 4000), delay 600 ms | ease.linear | text-shimmer, composer-status-strip | canonical (names text-shimmer `calm` cadence = status strip “1.0 s every 4.0 s”) | shimmer off |
| `loop.waiting` | 2000 ms period, 300 ms turn | ease.in-out | composer-status-strip, subagent-tree, loader-vector | canonical (new name for the “awaiting you” tip) | static waiting glyph |

#### Stagger

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `stagger.base` | 40 ms per item, cap 6 | — | composer, container-loaders, fanout-grid, loaders, media-generation-placeholder, permission-rules, plan-todo, pr-card, review-card, status-text, tool-call | canonical (renamed from bare `stagger`, 30–40 / cap 4–6 → 40 / 6) | none (all at once) |
| `stagger.tight` | 30 ms per item, cap 8 | — | background-jobs, composer-voice-input, desktop-overlays, diff-viewer, genui, genui-forms, genui-renderer, live-captions, mcp-plugin-list, run-meter | canonical | none (all at once) |
| `stagger.fold` | 20 ms per row, cap 6 | — | checkpoint-timeline, checkpoints-git, plan-todo, pr-card, review-card, tools | canonical | none (all at once) |
| `stagger.digit` | 30 ms per digit, right → left | — | approvals, elapsed-timer, fanout-grid, file-edit-chip, git-chips, merge-conflict, review-card, rollback-impact, status-text, streaming-markdown, tools | canonical | none (all at once) |
| `stagger.glyph` | 25 ms per glyph, total cap 300 ms | — | status-text | canonical | none (all at once) |
| `stagger.row` | = stagger.base | — | rollback-impact, tools | alias of stagger.base | none (all at once) |
| `stagger.chips` | = stagger.base (cap 5 → 6) | — | messages | alias of stagger.base (cap aligned 5 → 6) | none (all at once) |
| `stagger.bulk` | = stagger.tight (30 ms, cap 8, rest instant) | — | checkpoint-timeline, checkpoints-git, diff-viewer | alias of stagger.tight | none (all at once) |
| `stagger` | — | — | — | renamed → stagger.base | — |

#### Delays and visibility thresholds

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `delay.intent` | 80 ms | — | messages, message-bubble | canonical (new name; was prose in `dur.actions.reveal`) | unchanged (not motion) |
| `delay.hover` | 150 ms | — | checkpoints-git, checkpoint-timeline, citations-sources | canonical (new name) | unchanged (not motion) |
| `delay.marker.hover` | 150 ms | — | checkpoint-timeline, checkpoints-git | alias of delay.hover | unchanged (not motion) |
| `delay.leave` | 300 ms | — | messages, message-bubble, checkpoint-timeline | canonical (new name for the 300 ms leave grace) | unchanged (not motion) |
| `delay.preview.open` | 300 ms | — | checkpoint-timeline, checkpoints-git, citations-sources, diff-viewer, messages, run-replay | canonical (absorbs checkpoints-git `delay.preview` 200 ms) | unchanged (not motion) |
| `delay.preview.close` | 150 ms | — | citations-sources | canonical | unchanged (not motion) |
| `delay.preview` | 300 ms | — | checkpoints-git | renamed → delay.preview.open (value 200 → 300) | unchanged (not motion) |
| `delay.tooltip` | 400 ms | — | tool-call, review-card, side-panels, workspace-chrome | canonical (new name) | unchanged (not motion) |
| `delay.row-guard` | 150 ms | — | approvals, checkpoints-git, diff-viewer, file-edit-chip, git-chips, merge-conflict, pr-card, review-card, tool-call, tools | canonical (renamed from `delay.row_guard`) | unchanged (not motion) |
| `delay.container.show` | 150 ms | — | container-loaders, border-beam, background-jobs, fanout-grid, skeletons, genui-skeletons, run-replay, trace-view | canonical (renamed from family D/I `show_delay` 150 ms) | unchanged (not motion) |
| `delay.skeleton` | 150 ms | — | checkpoint-timeline, checkpoints-git | alias of delay.container.show | unchanged (not motion) |
| `delay.loader.show` | 300 ms | — | loaders, loader-glyph, loader-mascot, animated-marks, reasoning-block, messages, plugin-icons, live-voice, icon-states, thinking-orb | canonical (renamed from family B `show_delay` 300 ms) | unchanged (not motion) |
| `delay.loader.handoff` | 250 ms | — | loaders, reasoning-block | canonical (renamed from `handoff_grace`) | unchanged (not motion) |
| `delay.auto-collapse` | 1200 ms | — | diff-viewer, review-card, terminal-output, tools | canonical (renamed from `delay.auto_collapse`) | unchanged (not motion) |
| `delay.approval.input-guard` | 400 ms | — | approvals, approval-card, plan-approval | canonical (renamed from `delay.approval.input_guard`); safety — never shortened | unchanged (not motion) |
| `delay.shimmer` | 600 ms | — | merge-conflict, plan-todo, review-card, status-text, text-shimmer, tool-call | canonical (renamed from `dur.shimmer.delay`) | n/a (shimmer off) |
| `dur.loader.min-visible` | 600 ms | — | loaders, elapsed-timer | canonical (renamed from `min_visible`) | unchanged (not motion) |
| `dur.container.min-visible` | 300 ms | — | container-loaders, skeletons, genui-skeletons | canonical (renamed from `min_display`, incl. exit) | unchanged (not motion) |
| `delay.row_guard` | — | — | — | renamed → delay.row-guard | — |
| `delay.auto_collapse` | — | — | — | renamed → delay.auto-collapse | — |
| `delay.approval.input_guard` | — | — | — | renamed → delay.approval.input-guard | — |
| `show_delay (family B, 300 ms)` | — | — | — | renamed → delay.loader.show | — |
| `show_delay (family D / I, 150 ms)` | — | — | — | renamed → delay.container.show | — |
| `min_visible` | — | — | — | renamed → dur.loader.min-visible | — |
| `min_display` | — | — | — | renamed → dur.container.min-visible | — |
| `handoff_grace` | — | — | — | renamed → delay.loader.handoff | — |

#### Component durations

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `dur.row.enter` | 200 ms height (spring.snap) + 160 ms fade (ease.out), y +4 → 0 | spring.snap | approval-card, citations-sources, file-edit-chip, messages, permission-rules, plan-todo, pr-card, reasoning-block, review-card, rollback-impact, streaming-markdown, terminal-output, tool-call, tools | canonical — tools.md definition wins over messages.md 180 ms | fade `dur.instant`, height instant |
| `dur.chip.enter` | 180 ms fade + y +4 → 0, no height animation | ease.out | messages, citations-sources, reasoning-block, command-palette, mcp-plugin-list | canonical (new name; was messages.md `dur.row.enter`) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.row.morph` | 160 ms | spring.snap | file-edit-chip, merge-conflict, plan-todo, pr-card, rollback-impact, terminal-output, tool-call, tools | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.check.draw` | 240 ms | ease.out | merge-conflict, pr-card, review-card, rollback-impact, tool-call, tools | canonical | ✓ appears without stroke draw |
| `dur.expand` | 220 ms height (spring.snap) + content fade 150 ms (ease.out) at +40 ms | spring.snap | approval-card, diff-viewer, merge-conflict, plan-todo, rollback-impact, tool-call, tools | canonical | height instant, content fade `dur.instant` |
| `dur.collapse` | 280 ms total: content fade 100 ms then height 180 ms, both ease.in | ease.in | diff-viewer, message-bubble, messages, pr-card, reasoning-block, review-card, rollback-impact, streaming-markdown, terminal-output, tool-call, tools | canonical — tools.md definition (100 fade + 180 height) wins over messages.md 200 ms | height instant, content fade `dur.instant` |
| `dur.context.expand` | 220 ms | spring.snap | checkpoints-git, diff-viewer | alias of dur.expand (180 → 220) | height instant |
| `dur.height.settle` | 200 ms | spring.snap | skeletons, genui, genui-renderer, genui-skeletons, genui-blocks, message-bubble, streaming-markdown, plan-approval, media-generation-placeholder | canonical (new name for the recurring 200 ms height settle) | height jumps once |
| `dur.version.swap` | 180 ms crossfade + height dur.height.settle | ease.in-out | messages | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.highlight.fade` | 1600 ms | ease.out | diff-viewer, file-edit-chip, genui, genui-renderer, permission-rules, plan-todo, tools | canonical | static 2 px accent bar for 4 s |
| `dur.flash.target` | 600 ms hold (dur.beat) + 900 ms fade | ease.out | citations-sources, messages | canonical (renamed from `highlight.flash`) | static tint for the hold, then removed |
| `highlight.flash` | 600 ms hold + 900 ms fade | ease.out | messages | renamed → dur.flash.target | — |
| `dur.digit.roll` | 280 ms | ease.out | approvals, background-jobs, checkpoint-timeline, checkpoints-git, desktop-overlays, diff-viewer, elapsed-timer, fanout-grid, file-edit-chip, git-chips, mcp-plugin-list, merge-conflict, message-bubble, messages, permission-rules, plan-todo, reasoning-block, review-card, rollback-impact, run-meter, side-panels, status-text, streaming-markdown, text-reveal, tools | canonical | digits swap without rolling |
| `dur.count.roll` | 280 ms | ease.out | approval-card, approvals, genui-forms, plan-approval, plan-todo, review-card, tools | alias of dur.digit.roll | swap |
| `dur.restore.tick` | 280 ms | ease.out | checkpoints-git | alias of dur.digit.roll | swap |
| `dur.count.up` | 500 ms | ease.out | genui-blocks, composer-context-meter | canonical (new name: first-reveal count-ups, large reconciles) | final value shown |
| `dur.tense.roll` | 240 ms | ease.out | elapsed-timer, file-edit-chip, plan-todo, reasoning-block, status-text, tool-call, tools, verb-cycler | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.verb.swap` | 180 ms | ease.out | composer-status-strip, status-text, verb-cycler | canonical | crossfade `dur.instant` (80 ms opacity, no transform); dwell ×2 |
| `dur.verb.dwell` | 2800 ms | — | status-text | canonical | doubled (5600 ms) |
| `dur.reveal.token` | 160 ms | ease.out | status-text, text-reveal | canonical | whole block appears with `dur.instant` fade |
| `dur.reveal.media` | 600 ms | ease.out | container-loaders, media-generation-placeholder | canonical (renamed from `dur.reveal`; clashes with group dur.reveal.*) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.reveal` | 600 ms | ease.out | container-loaders, media-generation-placeholder | renamed → dur.reveal.media | — |
| `dur.shimmer.sweep` | 1400 ms | ease.coast | status-text | canonical | shimmer off |
| `dur.shimmer.rest` | 800 ms | — | status-text | canonical | shimmer off |
| `dur.shimmer.delay` | 600 ms | — | plan-todo, status-text | renamed → delay.shimmer | — |
| `dur.settle` | 240 ms | ease.in | border-beam, container-loaders, file-edit-chip, loader-vector, loaders, plugin-icons, tool-call, tools | canonical — curve ease.out → ease.in (majority; it is an exit) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.stall.ramp` | 2000 ms | ease.linear | loaders, animated-marks, loader-vector, loader-mascot, subagent-tree, skeletons, genui-renderer, container-loaders, text-shimmer, composer-status-strip, tools | canonical (new name; colour ramp in OKLab) | tone swaps with `dur.instant` |
| `dur.stall.recover` | 200 ms | ease.out | loaders, animated-marks, loader-glyph, loader-vector, subagent-tree, composer-status-strip, text-shimmer | canonical (new name) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.tempo.change` | 400 ms | ease.in-out | loaders, loader-glyph | canonical (new name; phase-continuous loop speed change) | n/a (loops static) |
| `dur.error.nudge` | 300 ms | ease.linear | tools, tool-call, loaders, container-loaders | canonical (new name for tools.md “Error nudge”) | no nudge; inline text |
| `dur.actions.reveal` | 120 ms | ease.out | message-bubble, messages | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.actions.hide` | 160 ms | ease.in | messages | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.marker.reveal` | 120 ms | ease.out | checkpoint-timeline, checkpoints-git | alias of dur.actions.reveal (delay delay.hover) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.copy.confirm` | 1500 ms hold, then 160 ms revert | — | git-chips, icon-states, live-captions, message-bubble, messages, streaming-markdown, tool-call | canonical — 1500 ms hold wins over 1200 ms used in some specs | hold kept (carries meaning); revert `dur.instant` |
| `dur.preview.in` | 140 ms | ease.out | checkpoint-timeline, checkpoints-git, diff-viewer, run-replay | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.sheet.enter` | 220 ms | spring.snap | checkpoint-timeline, checkpoints-git, rollback-impact | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.sheet.exit` | 160 ms | ease.in | checkpoint-timeline, checkpoints-git, rollback-impact | canonical (written `.exit` in checkpoints-git) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.surface.enter` | 200 ms | ease.out | background-jobs, command-palette, checkpoint-timeline, checkpoints-git, desktop-overlays, genui-renderer, takeover-banner, composer-status-strip, workspace-chrome | canonical (new name: toasts, banners, overlays, side panels) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.surface.exit` | 160 ms | ease.in | background-jobs, command-palette, checkpoint-timeline, checkpoints-git, desktop-overlays, genui-renderer, takeover-banner, composer-status-strip, workspace-chrome | canonical (new name) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.popover.open` | 180 ms | ease.out | git-chips, voice-control-bar, background-jobs, run-meter, command-palette, desktop-overlays, citations-sources, terminal-tabs, composer-tools-menu, composer-slash-mentions | canonical (new name; = dur.base) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.popover.close` | 120 ms | ease.in | git-chips, voice-control-bar, command-palette, desktop-overlays, citations-sources, terminal-tabs | canonical (new name; = dur.fast) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.rewind.fold` | 280 ms | ease.in-out | checkpoint-timeline, checkpoints-git, tools | canonical (also the tools.md turn fold) | heights instant |
| `dur.hunk.resolve` | 240 ms tint → neutral (ease.out) + chip morph dur.row.morph | ease.out | checkpoints-git, diff-viewer | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.line.enter` | 120 ms | ease.out | checkpoints-git, diff-viewer | canonical | appears without fade |
| `dur.layout.switch` | 160 ms | ease.in-out | checkpoints-git, diff-viewer | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.approval.in` | 200 ms | spring.snap | approval-card, approvals, plan-approval | alias of dur.row.enter (curve conflict resolved) | fade `dur.instant`, height instant |
| `dur.approval.resolve` | 220 ms | spring.snap | approval-card, approvals, plan-approval, tool-call | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.approval.arm` | 600 ms | ease.linear | approval-card, approvals, composer-permission-selector | canonical — safety delay, never shortened | static “Available in 0.6 s” text, delay kept |
| `dur.approval.hold` | 1200 ms | ease.linear | approval-card, approvals, composer-permission-selector, rollback-impact | canonical — safety delay; also adopted by rollback-impact hold-to-confirm (800 → 1200) | duration kept; stepped 4-segment fill at 4 Hz + “Hold to allow · 1.2 s” |
| `dur.approval.drain` | 200 ms | ease.out | approval-card, rollback-impact | canonical (renamed from `dur.approval.hold.drain`; curve ease.in → ease.out) | fill clears with `dur.instant` |
| `dur.approval.hold.drain` | 200 ms | ease.out | approvals | renamed → dur.approval.drain | — |
| `dur.approval.page` | 160 ms fade + 12 px slide | ease.out | approval-card, approvals | canonical | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.approval.stale` | 160 ms desaturate (ease.out), 4000 ms hold, 180 ms collapse (ease.in) | ease.out | approval-card, approvals | canonical | desaturate instant; hold kept; collapse instant |
| `dur.frame.glyph` | 80 / 100 / 120 / 160 / 200 ms per frame | step | loader-glyph, loaders | canonical (renamed from `frame.glyph`) | still glyph per phase |
| `frame.glyph` | 80 ms | step | loaders | renamed → dur.frame.glyph | — |

#### Voice

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `dur.voice.source-crossfade` | 160 ms | ease.in-out | live-voice, voice-visualisers, voice-orb | canonical (renamed from `voice.source.crossfade`) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.voice.speaker-swap` | 120 ms | ease.out | live-voice, live-captions | canonical (renamed from `voice.speaker.swap`) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.voice.barge-drop` | 80 ms | ease.firm | live-voice, voice-orb, voice-visualisers | canonical (renamed from `voice.barge.drop`) | level → 0 instantly |
| `dur.voice.asleep-fade` | 1200 ms | ease.in-out | live-voice, voice-orb | canonical (renamed from `voice.asleep.fade`) | poster swap `dur.instant` |
| `dur.voice.wake` | 240 ms | ease.out | live-voice, voice-orb | canonical (renamed from `voice.asleep.wake` / `wake`) | crossfade `dur.instant` (80 ms opacity, no transform) |
| `dur.voice.level.attack` | 30 ms | filter: one-pole | live-voice, voice-orb, voice-visualisers | canonical (renamed from `voice.level.attack`; a smoothing time constant, not an easing) | level ring updated at 10 Hz, kept |
| `dur.voice.level.release` | 150 ms | filter: one-pole | live-voice, voice-orb, voice-visualisers | canonical (renamed from `voice.level.release`) | kept at 10 Hz |
| `voice.source.crossfade` | 160 ms | — | live-voice | renamed → dur.voice.source-crossfade | — |
| `voice.speaker.swap` | 120 ms | — | live-voice | renamed → dur.voice.speaker-swap | — |
| `voice.barge.drop` | 80 ms | — | live-voice | renamed → dur.voice.barge-drop | — |
| `voice.asleep.fade` | 1200 ms | — | live-voice | renamed → dur.voice.asleep-fade | — |
| `voice.asleep.wake` | 240 ms | — | live-voice | renamed → dur.voice.wake | — |
| `voice.level.attack` | 30 ms | — | live-voice | renamed → dur.voice.level.attack | — |
| `voice.level.release` | 150 ms | — | live-voice | renamed → dur.voice.level.release | — |

#### Policy (motion.*)

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `fps.vector / fps.particle / fps.shader` | — | — | — | renamed → motion.fps.vector / .particle / .shader | — |
| `fps.pixel` | — | — | — | renamed → motion.fps.pixel | — |
| `motion.loop.min-period` | 700 ms (≈ 1.43 Hz) | — | loaders | canonical (policy) | n/a |
| `motion.loop.luminance-min-period` | 1000 ms (1 Hz) | — | container-loaders | canonical (policy) | n/a |
| `motion.flash.max-per-second` | 3 | — | loaders, container-loaders, border-beam, composer-effort-control, media-analysis-scan, text-shimmer, live-voice, voice-orb | canonical (policy, WCAG 2.3.1) | n/a |
| `motion.flash.max-luminance-delta` | 0.25 (single completion flash) | — | container-loaders, media-analysis-scan | canonical (policy) | n/a |
| `motion.audio.luminance-max` | ±10 % | — | voice-orb, voice-visualisers, live-voice, container-loaders | canonical (policy) | n/a |
| `motion.audio.luminance-lowpass` | 3 Hz one-pole | — | voice-orb, voice-visualisers, live-voice | canonical (policy) | n/a |
| `motion.fps.max` | 60 fps (120 Hz displays not raised) | — | loaders | canonical (policy) | n/a |
| `motion.fps.vector` | 60 fps cap | — | loaders | canonical (renamed from `fps.vector`) | n/a |
| `motion.fps.particle` | 60 fps cap | — | loaders | canonical (renamed from `fps.particle`) | n/a |
| `motion.fps.shader` | 60 fps cap | — | loaders | canonical (renamed from `fps.shader`) | n/a |
| `motion.fps.pixel` | 7.5 or 15 fps | — | loaders | canonical (renamed from `fps.pixel`) | n/a |
| `motion.fps.text-effects` | 30 fps (shimmer, rolls) | — | text-shimmer, status-text, tools | canonical (policy) | n/a |
| `motion.fps.unfocused` | 20 fps vector/shader, 15 fps particle | — | loaders | canonical (policy) | n/a |
| `motion.fps.low-power` | 30 fps, half particles | — | loaders | canonical (policy) | n/a |
| `motion.fps.idle` | 1 Hz (timers only) | — | loaders, tools | canonical (policy) | n/a |
| `motion.reduced` | boolean (OS setting or app “Pause animations”) | — | animated-marks, genui, genui-skeletons, loaders, status-text | runtime flag (not a design token value; read by every preset) | is the switch |

#### Behavioural timings found in the specs (not motion tokens)

| Token | Value | Curve | Used by | Status | Reduced form |
|---|---|---|---|---|---|
| `timeout.countdown.show` | last 60 s or last 25 % | — | approvals | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `batch.window` | 50 ms | — | approvals | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `undo.window` | 10 s (paused on hover) | — | checkpoints-git | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.data.rate` | 30–60 Hz | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.vad.on / hangover` | 60 / 300 ms | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.barge.min` | 250 ms | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.barge.ack_max` | 300 ms | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.false_barge.window` | 700 ms | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.filler.after` | 1200 ms / 3000 ms | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.asleep.after` | 60 s | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.reconnect.backoff` | 0.5, 1, 2, 4, 8 s | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |
| `voice.ended.auto_dismiss` | 8 s | — | live-voice | behavioural timing — not a motion token; stays in its spec | unchanged (not motion) |

## 14. Migration

Spec edits to make later (no spec has been changed by this document). IDs are stable so PRs can
reference them.

### 14.1 Renames

| ID | Old | New | Specs to edit |
|---|---|---|---|
| M-01 | `dur.row.enter` (messages.md meaning: 180 ms, y +4) | `dur.chip.enter` | messages, citations-sources (§ Searching, Memory saved), reasoning-block (Appear) |
| M-02 | `dur.reveal` | `dur.reveal.media` | container-loaders, media-generation-placeholder |
| M-03 | `dur.shimmer.delay` | `delay.shimmer` | status-text + its users |
| M-04 | `dur.approval.hold.drain` | `dur.approval.drain` | approvals |
| M-05 | `delay.preview` | `delay.preview.open` | checkpoints-git |
| M-06 | `highlight.flash` | `dur.flash.target` | messages |
| M-07 | `delay.row_guard`, `delay.auto_collapse`, `delay.approval.input_guard` | `delay.row-guard`, `delay.auto-collapse`, `delay.approval.input-guard` | tools, tool-call, terminal-output, approvals, + users |
| M-08 | `stagger` (bare) | `stagger.base` | composer, loaders, container-loaders, status-text, genui |
| M-09 | `stagger.row`, `stagger.chips`, `stagger.bulk` | keep as aliases; prefer `stagger.base` / `stagger.tight` | tools, messages, checkpoints-git |
| M-10 | `frame.glyph` | `dur.frame.glyph` | loaders, loader-glyph |
| M-11 | `fps.vector`, `fps.particle`, `fps.shader`, `fps.pixel` | `motion.fps.*` | loaders |
| M-12 | `show_delay` (300 ms, family B) | `delay.loader.show` | loaders, loader-glyph, loader-mascot, animated-marks, reasoning-block, messages, plugin-icons, live-voice, icon-states, thinking-orb |
| M-13 | `show_delay` (150 ms, family D / I) | `delay.container.show` | container-loaders, border-beam, background-jobs, fanout-grid, skeletons, genui-skeletons, run-replay, trace-view |
| M-14 | `min_visible`, `min_display`, `handoff_grace` | `dur.loader.min-visible`, `dur.container.min-visible`, `delay.loader.handoff` | loaders, elapsed-timer, container-loaders, skeletons, genui-skeletons, reasoning-block |
| M-15 | `voice.source.crossfade`, `voice.speaker.swap`, `voice.barge.drop`, `voice.asleep.fade` / `wake`, `voice.level.attack` / `release` | `dur.voice.source-crossfade`, `dur.voice.speaker-swap`, `dur.voice.barge-drop`, `dur.voice.asleep-fade`, `dur.voice.wake`, `dur.voice.level.attack` / `release` | live-voice, voice-orb, voice-visualisers, live-captions |
| M-16 | `dur.count.roll`, `dur.restore.tick` | keep as aliases of `dur.digit.roll` | tools, checkpoints-git |
| M-17 | `dur.marker.reveal` | keep as alias of `dur.actions.reveal` (+ `delay.hover`) | checkpoints-git |
| M-18 | `delay.marker.hover` | keep as alias of `delay.hover` | checkpoints-git |
| M-19 | `dur.context.expand` | alias of `dur.expand` | checkpoints-git, diff-viewer |
| M-20 | `dur.approval.in` | alias of `dur.row.enter` | approvals, approval-card, plan-approval |

### 14.2 Value and curve changes

| ID | Where | Now | Becomes |
|---|---|---|---|
| M-21 | messages.md `dur.collapse`; reasoning-block (toggle, timing table, "200 ms `ease.out`"); streaming-markdown (Show all / Collapse 200 ms); message-bubble (reason row unfold 200 ms) | 200 ms height | open `dur.expand` (220 ms `spring.snap`), close `dur.collapse` (280 ms staged) |
| M-22 | approvals.md §5.2 Card / footer enter | height on `ease.out` | `dur.row.enter` (height `spring.snap`) |
| M-23 | approvals.md, approval-card.md hold drain | 200 ms `ease.in` | 200 ms `ease.out` (`dur.approval.drain`) |
| M-24 | rollback-impact.md hold-to-confirm (§ Hold to confirm, timing, variant line, keyboard line) | 800 ms | `dur.approval.hold` 1200 ms |
| M-25 | container-loaders.md Success settle fade | `ease.out` | `ease.in` (`dur.settle`) |
| M-26 | tool-call (Copy ✓ revert after 1200), git-chips (Copy hold 1200), live-captions ("Copied" 1200) | 1200 ms | `dur.copy.confirm` 1500 ms |
| M-27 | checkpoints-git `delay.preview`, run-replay snapshot preview, checkpoint-timeline preview popover, diff-viewer minimap tooltip | 200 ms | `delay.preview.open` 300 ms |
| M-28 | checkpoints-git `dur.context.expand`, diff-viewer context expand | 180 ms | 220 ms (`dur.expand`) |
| M-29 | messages.md `stagger.chips`; citations-sources, message-bubble (cap 5) | cap 5 | cap 6 |
| M-30 | composer-*.md spring and curve cells: `ease-out`, `ease-in-out`, raw `cubic-bezier(0.22, 1, 0.36, 1)`; approvals.md "Selector attention pulse `ease-in-out`" | unnamespaced | `ease.out` / `ease.in-out` |
| M-31 | bare `sine` / `cosine` (as a curve) in composer-context-meter, composer-model-picker, composer-status-strip, composer-voice-input, text-shimmer, voice-orb, loaders (`loop.calm`) | sine | `ease.sine` (cosine *band profile* in text-shimmer is a band shape, not a curve: unchanged) |
| M-32 | `loop.glow`, `loop.pulse` (container-loaders, skeletons, genui-skeletons) | `ease.in-out` | `ease.sine` |
| M-33 | "linear", "linear in OKLab", "linear RGB in OKLab" | prose | `ease.linear` (colour in OKLab) |
| M-34 | "exponential" level smoothing (live-voice, voice-orb, voice-visualisers, composer-voice-input) | prose | `dur.voice.level.attack` / `release` (one-pole filter) |
| M-35 | composer.md, loaders.md, container-loaders.md token tables: ranges for `dur.instant/base/travel`, `loop.calm/base/brisk` | ranges | single defaults (80 / 180 / 240; 2000 / 1000 / 800) with the range kept as "allowed" |
| M-36 | composer-status-strip (job % roll 300, PR conflict roll 300), pr-card (conflict count roll 300), composer-model-picker (detail tray roll 320), composer-context-meter chip roll 320, live-voice latency roll 180 | 180–320 ms | `dur.digit.roll` 280 ms |
| M-37 | genui-blocks / genui number roll 400 ms `spring.snap` | spring roll | `dur.digit.roll` 280 ms `ease.out` |
| M-38 | composer-context-meter popover digit roll 560 ms, turn-end reconcile 400 ms; genui-blocks metric count-up 500 ms | 400–560 ms | `dur.count.up` 500 ms |
| M-39 | genui, genui-renderer agent-update wash | 1200 ms | `dur.highlight.fade` 1600 ms |
| M-40 | genui-skeletons Scan-line (proposed) | 1.6 s + 400 ms rest | `loop.scan` 1.2 s + 200 ms |
| M-41 | loader-vector Waiting tip | 600 ms every 2000 | `loop.waiting` (300 ms turn) |
| M-42 | composer-status-strip Verb shimmer "1.0 s sweep every 4.0 s" | prose | `loop.shimmer.calm` |
| M-43 | takeover-banner exit | 180 ms | `dur.surface.exit` 160 ms |
| M-44 | Unnamed springs (§4.2 table: composer-attachments ×2, composer-context-meter, composer-input-shell ×2, composer-model-picker, composer-slash-mentions ×2, composer-tools-menu ×2, icon-states, thinking-orb, voice-orb ×2) | literal k / c | the mapped `spring.*` at the row's nominal duration |
| M-45 | composer-model-picker press spring (k 600, c 30, 8.8 % overshoot) | bounce | `spring.snap` (no bounce) |

## 15. Open questions
- `undo.window` differs across specs (10 s checkpoints-git, 5 s review-card / composer-queue-stack /
  permission-rules, 6 s mcp-plugin-list). It is behaviour, not motion, so it's not decided here. It
  needs a behaviour spec.
- The GPUI frame-request and window-occlusion APIs need to be confirmed at implementation (loaders.md
  already flags this).
