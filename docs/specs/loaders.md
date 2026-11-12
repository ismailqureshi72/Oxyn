**Status:** draft v0.1 (2026-11-11).

# Spec: loaders & thinking indicators (family overview)

Overview of family **B** in [../component-taxonomy.md](../component-taxonomy.md) (target **180
variants**). The style families each have their own spec at the depth of the worked example
[composer-send-control.md](composer-send-control.md):

| Spec | Style family | Variants (free + Pro) |
|---|---|---|
| [loader-glyph.md](loader-glyph.md) | terminal / glyph frame loaders | 14 + 30 = **44** |
| [loader-vector.md](loader-vector.md) | dots, arcs, rings, bars, discs, morphs, orbits | 16 + 44 = **60** |
| [thinking-orb.md](thinking-orb.md) | particle / shader orbs | 6 + 30 = **36** |
| [loader-mascot.md](loader-mascot.md) | mini mascots, animated marks, composer border light / busy glow | 8 + 32 = **40** |
| **Total** | | **44 free + 136 Pro = 180** |

This file defines what all of them share: **one loader engine** (frame clock, render tiers, busy
signal, sizes, colour roles, still frames, budgets) and **one mapping from `AgentState` to loader
behaviour**. A variant only supplies its drawing and its per-state motion; it never owns a timer,
decides when to show, or picks its own colours.

All values marked *proposed* are Oxyn's own starting points, to be tuned in the Studio. Numbers
attributed to other products come from the research notes named next to them.

## 1. Purpose
Every agent app has the same three seconds of doubt after the user sends: *is it doing anything?*
Loaders answer that, and in an agent app they carry more than "busy": **what kind** of work
(thinking vs running a tool vs writing), **whether it is stuck** (stalled), and **whether it is
waiting on me** (approval). The best references already encode this in motion: Claude Code's
glyph drifts toward red when no tokens arrive for 3 s; Zed uses a different spinner (Sand) for
"awaiting confirmation" than for generating; Claude Desktop changed its mark to 3 nodes for search
and 4 for tools (all research note 08). Circle Loaders pairs each loader with an agent verb
(research note 02). We make that the default rather than the exception.

Where loaders appear (each is a **busy region**, see §3.3):
- **Composer region:** the status-strip dot ([composer-status-strip.md](composer-status-strip.md)),
  the composer border light, or a mascot on the composer edge. One of them leads.
- **Transcript tail:** the thinking row under the last message ("Thinking · 12s").
- **Tool / step rows:** one loader per running row (family F).
- **Lists:** sidebar thread rows, sub-agent rows, background jobs (status dots).
- **Avatars / heroes:** orb or mascot at 32–48 px (and larger in family K voice screens).

Family rules (apply to every loader):
1. **No private timers.** Every loader samples the shared motion clock
   ([../architecture.md](../architecture.md)); same-variant loaders on screen are phase-locked.
2. **One animating loader per busy region.** Others in that region show their still frame.
3. **State drives motion, not mount/unmount.** A loader that changes from thinking to tool
   morphs its motion in place; it never restarts.
4. **Every variant ships a still frame per state** (reduced motion, paused, offscreen poster).
5. **No flashing above 3 per second**, and meaning never depends on motion or colour alone.

## 2. The loader engine (`oxyn-core::loader`, drawn by `oxyn-gpui` / `oxyn-dioxus`)

> Implemented in plan #16: `LoaderPhase` (§3.2), `TempoClock` (phase integral, §2.1), show/hide
> hysteresis (§3.3), `BusyArbiter` (§2.3) and the T1 glyph engine. T2–T4 tiers, the steer blip and
> per-size tuning land with the vector, orb and mascot PRs. Deviations: `Queued` and `Steering`
> map to `Thinking`; a run that never showed a loader shows no `✓` either (the status strip says
> "Worked for…"); GPUI mixes colours in sRGB, the web in OKLab (`color-mix`).

### 2.1 Frame clock
```
 platform vsync / rAF ──▶ MotionClock (one per window) ──▶ t (monotonic ms, never resets)
                              │  rate policy: Focused · Unfocused · Hidden · LowPower · Reduced
                              ├─▶ leases: each animating loader holds a lease {next_due_ms}
                              │     glyph:  next_due = next frame boundary (e.g. every 80 ms)
                              │     vector/particle/shader: every display frame, capped by tier fps
                              └─▶ requests a repaint only at min(next_due) over live leases
```
- **Phase is computed, not stepped.** `phase = (∫ speed dt) / period mod 1` with the integral kept
  per loader, so a speed change (e.g. stall slow-down) never jumps the drawing. Frame loaders use
  `frame = floor(phase × n)`. Phase origin is clock zero, not mount time, which keeps all instances of
  a variant in phase (the idea behind OpenCode's `syncSpinPhase` and Warp's single `AnimationClock`,
  research note 08).
- **Repaint only when something changes.** A region whose only animation is an 80 ms braille
  spinner repaints at 12.5 Hz, not 60 Hz (Codex repaints at 32 ms only while effects run and 1 s
  otherwise; research note 08). With no leases the clock idles at 1 Hz for timers.
- **Rate policy (proposed):**

| Window / system state | Glyph loaders | Vector loaders | Particle (CPU) | Shader (GPU) |
|---|---|---|---|---|
| Focused, visible | native frame ms | ≤ 60 fps | ≤ 60 fps | ≤ 60 fps |
| Visible, unfocused | native frame ms | ≤ 20 fps | ≤ 15 fps | ≤ 20 fps |
| Low-power / battery saver (where the platform reports it) | native | ≤ 30 fps | ≤ 30 fps, half particles | falls back to particle tier |
| Window hidden / minimised / occluded, or loader scrolled offscreen | **paused** (no repaint) | paused | paused | paused, GPU resources kept 30 s then freed |
| Reduced motion | still frame | still frame | still frame | still frame |

  High-refresh displays (120 Hz) do not raise loader rates above 60 fps (proposed). MonoCode's GPUI
  pulse clock (33 ms lease, 30 / 15 / 7.5 fps rates, 8× slower in background windows; research
  note 08, MIT) is the closest precedent and may be ported with attribution.
- **Resume:** when a paused loader comes back, it draws at the phase the clock says *now* (no
  catch-up animation, no restart from frame 0).

### 2.2 Render tiers
Every variant declares its tier; tiers share one props contract so an app can swap them.

| Tier | Technique | GPUI (primary) | Dioxus web | Examples |
|---|---|---|---|---|
| **T0 still** | one static frame per state | text or path | text / inline SVG | reduced motion, posters |
| **T1 glyph** | text frames from a frame table | text run in the mono font, fixed cell box | text node, `font-variant-numeric`, fixed `ch` width | braille, blocks, star, scanner |
| **T2 vector** | paths, quads, opacity/transform only | canvas element + path API (exact API to confirm at implementation) | inline SVG; CSS keyframes phase-locked with a negative `animation-delay` from the clock, so the compositor runs them | dots, arcs, rings, morphs |
| **T3 particle** | CPU-computed dots/paths, ≤ 400 primitives | batched quads (circles) | Canvas 2D | dotted orbs, swarms |
| **T4 shader** | GPU fragment shader | **open:** depends on custom-shader support in our GPUI stack; until confirmed, T4 variants fall back to T3 on GPUI | WebGL2 (WebGPU later) | liquid, nebula, aurora rim |

Fallback chain: **T4 → T3 → T2 → T0**, chosen per loader at mount and again on GPU context loss.
Each T4/T3 variant names its fallback variant in its `meta.toml` (proposed field `fallback`).

### 2.3 Busy signal (one loader animates per region)
BoardUI drives the composer glow, the thinking row and the controls from one `waiting` flag
[R095]; we generalise it:

```rust
pub struct BusySignal {             // one per BusyRegion, owned by oxyn-core
    pub region: BusyRegion,         // Composer | TranscriptTail | ToolRow(id) | ListRow(id) | Avatar(id) | Window
    pub phase: LoaderPhase,         // derived from AgentState (§4)
    pub lead: Option<LoaderId>,     // the one loader allowed to animate
    pub since: Instant,
}
```
Rules (proposed):
- Loaders **claim** a region with a `BusyPriority` (`Lead`, `Normal`, `Mirror`). The highest
  claimant animates; ties go to the most recently mounted. Everyone else renders the **still frame
  of the same phase** (so colour and state still read) or hides if the variant says
  `hide_when_mirrored`.
- **Composer region default lead:** status-strip dot. If the app turns on a border light or a
  composer-edge mascot, that becomes the lead and the strip dot goes still.
- **Lists** (sidebar, sub-agents, jobs): every row is its own region, but at most **8 rows animate**
  at once; the rest show the still "working" dot. Visible rows nearest the viewport centre win.
- **Tool rows:** only rows in `Running` animate; finished rows morph to ✓/✕ and go still.
- **Window region:** a window-edge glow (Pro) can only lead when no composer loader is visible.

### 2.4 Sizes
Six tuned sizes: **12, 16, 20, 24, 32, 48 px** (box, square). Each variant ships **per-size tuning**
(stroke, dot count, frame set), not one asset scaled; Libraries.dev tunes its orbs separately at 20
and 64 px (research note 16), and Claude Desktop's node mark lost its connecting lines at 20 px
(research note 08).

| Size | Typical place | Min stroke / dot | Glyph tier | Particle / shader |
|---|---|---|---|---|
| 12 px | dense IDE rows, inline in 11–12 px text | 1.5 px stroke, 2 px dot | yes (font size 12) | no (falls to T2 pulse dot) |
| 16 px | tool rows, list rows | 1.5 px / 2.5 px | yes | dotted only, ≤ 24 dots |
| 20 px | status line, thinking row (default) | 2 px / 3 px | yes | ≤ 36 dots |
| 24 px | composer status, buttons | 2 px / 3.5 px | yes | ≤ 48 dots |
| 32 px | avatars, cards | 2.5 px / 4 px | yes (larger mono) | ≤ 80 dots; shader ok |
| 48 px | empty states, mini heroes | 3 px / 6 px | rare | ≤ 160 dots; shader ok |

No hairlines: any stroke under 1 px at its rendered size is a spec bug (legibility test at 16 and
20 px is part of every variant's review, an idea from research note 09). Hero sizes (64–256 px)
belong to orbs and family K and are tuned there.

### 2.5 Colour roles
Loaders read tokens, never literal colours (`oxyn-tokens`):

| Token | Default | Used for |
|---|---|---|
| `color.loader.fg` | `currentColor` | the moving part |
| `color.loader.track` | `fg` at 20 % alpha | rings' tracks, unlit dots (OpenCode uses 0.2, research note 08) |
| `color.loader.accent` | brand accent | thinking / working emphasis, orb core |
| `color.loader.waiting` | amber (darker step on light themes) | awaiting approval / input |
| `color.loader.warn` | warning | stalled ramp target |
| `color.loader.danger` | danger | error |
| `color.loader.success` | success | done moment (brief), then back to `fg` |
| `color.loader.spectral[0..n]` | 3–6 stops | orbs, rims, glows |

Colour changes between phases interpolate in OKLab over the timing in §5; the stall ramp is the
same mechanism as the status strip's ([composer-status-strip.md](composer-status-strip.md)).

### 2.6 Reduced motion and still frames
- Global token `motion.reduced` (from the OS setting or the app) makes every loader draw its
  **still frame for the current phase**. Loops stop. Phase changes become 0–80 ms opacity crossfades
  (`dur.instant`, as in [composer.md](composer.md)).
- Still frames are designed, not frame 0: e.g. the glyph `◌` for working (Codex's reduced-motion
  glyph, research note 08), an arc frozen a fifth of the way round (OpenCode's radar does this,
  research note 08), an hourglass glyph for waiting, `✓` / `✕` for done / error.
- Liveness under reduced motion comes from text that changes slowly (the 1 Hz elapsed timer in the
  status strip), not from the loader. Opt-in `ReducedLiveness::SlowFade` gives a 2 s opacity
  breathe 1 ↔ 0.6 (0.5 Hz) for apps that want it (proposed).
- The same still frames are used for **paused**, **offscreen poster** and **static export**.

### 2.7 Performance budgets (proposed, per loader, measured on the reference machines in the Studio)
| Tier | CPU per drawn frame | GPU per frame | Max repaint rate | Memory |
|---|---|---|---|---|
| T1 glyph | ≤ 0.05 ms | text only | frame ms (80–600 ms) | frame tables are static data |
| T2 vector | ≤ 0.2 ms | negligible | 60 fps | ≤ 64 path segments per size |
| T3 particle | ≤ 0.6 ms at ≤ 400 dots | quads only | 60 fps | dot buffer reused, no per-frame allocation |
| T4 shader | ≤ 0.1 ms to encode | ≤ 0.3 ms at 48 px, ≤ 1.5 ms at 256 px | 60 fps | one pipeline per variant, shared by instances |
| **All loaders in a window** | ≤ 1.0 ms CPU per frame | ≤ 2 ms GPU | — | — |

A loader that exceeds its budget for 60 consecutive frames drops one tier and logs a dev warning.

## 3. State mapping: `AgentState` → loader behaviour

### 3.1 State machine (loader phase)
```
               send                first event (Thinking)
  Hidden ─────────────▶ Submitted ───────────────────────▶ Thinking ◀────────────────┐
    ▲   (pulse, no loop, < 500 ms)          │   ▲  tool starts │  ▲ tool ends        │
    │                                        │   │             ▼  │                    │
    │                                        │   └──────── Working{activity} ──────────┤
    │                         first token    ▼                    │ tokens             │
    │                                     Streaming ◀─────────────┘                    │
    │                                        │  approval / question                    │
    │                                        ▼                                         │
    │                                     Waiting (own motion, amber) ── answered ─────┘
    │
    │  any active phase ── no event for stall_soft ──▶ Stalled(soft) ── stall_hard ──▶ Stalled(hard)
    │                      (event arrives: back to the phase it left, colour snaps back)
    │
    │◀── fade ── Done (morph to ✓, hold) ◀────── run completes ─────────────────────────┤
    │◀── fade ── Cancelled (freeze, desaturate) ◀── Stop ───────────────────────────────┤
    │◀── dismiss ─ Failed (morph to ✕ / !, static) ◀── error ───────────────────────────┘
  Overlays: Paused(offscreen / hidden) · Reduced (still frames) · Mirrored (not the region lead)
```

### 3.2 Mapping table
| `AgentState` | Loader phase | What every loader does (variants choose the drawing) |
|---|---|---|
| `Idle` | `Hidden` (or `Rest` for marks/orbs that stay on screen) | nothing animates; marks/orbs show their idle still or a ≤ 0.25 Hz breathe (opt-in) |
| `Composing` | `Hidden` / `Rest` | unchanged; marks may "attend" (Pro mascots) |
| `Submitted` | `Submitted` | **one** pulse (400 ms ring or brightness bump), no loop (architecture.md: "submitted pulse, not a spinner, < 500 ms") |
| `Thinking { verb }` | `Thinking` | the variant's calm loop at tempo **1.0×**; verb shown by family C next to it |
| `ToolRunning { tool, verb }` | `Working { activity }` | brisker loop at **1.25×**; activity (`Search`, `Read`, `Edit`, `Run`, `Browse`, `Generate`) may select a semantic motion (§3.4) |
| `Streaming` | `Streaming` | loop slows to **0.6×** or hands off to the caret/stream effect; the transcript is now the liveness cue |
| `AwaitingApproval { request }` | `Waiting` | **different motion**, not a slower spin: hourglass/sand, tip, or hold-and-breathe in `color.loader.waiting`; elapsed timers pause |
| `Queued { count }` | unchanged | queued messages do not change the loader (the queue stack shows them) |
| `Steering` | unchanged + blip | one 240 ms brightness blip on the lead loader, then continue |
| `Stalled { since }` < `stall_hard` | `Stalled(soft)` | tempo eases to 0.5× over 2 s and colour ramps toward `warn` over 2 s |
| `Stalled { since }` ≥ `stall_hard` | `Stalled(hard)` | loop stops on the still frame in `warn`; status text explains (status strip owns Retry) |
| `Done { elapsed }` | `Done` | morph to ✓ / settle pose in `success`, hold 600 ms, fade 240 ms (status strip keeps "Worked for 42s") |
| `Error { kind }` | `Failed` | morph to ✕ or `!` in `danger`, one 2-cycle 3 px nudge, then static until dismissed |
| `Cancelled` | `Cancelled` | freeze current frame, desaturate to `fg` 40 % over 160 ms, fade |

Thresholds match the status strip: `stall_soft` 5 s with no event and no tool running, `stall_hard`
45 s, `tool_stall` 120 s while a tool runs (proposed in
[composer-status-strip.md](composer-status-strip.md)). `Stalled` is derived by `oxyn-core`.

### 3.3 Show / hide hysteresis (proposed)
- `show_delay` **300 ms**: a `Thinking` phase shorter than this never shows a loader (Cline waits
  500 ms before its tail "Thinking…" to avoid flicker on reasoning → tool handoffs; research note 08).
- `min_visible` **600 ms**: once shown, a loader stays at least this long, then exits with the
  Done/Failed morph.
- `handoff_grace` **250 ms**: phase changes inside this window (e.g. tool → thinking → tool) don't
  restart or re-morph; the loader keeps its phase and only eases tempo.
- If `Submitted` and `Done` land in one frame, nothing is drawn (same rule as the send control).

### 3.4 Semantic activity (verb ↔ motion pairing)
`ToolRunning.tool` maps to an `Activity`; variants that support it switch motion family, others just
change tempo. Defaults (proposed):

| Activity | Motion idea | Example variants |
|---|---|---|
| `Think` | breathe / slow orbit | `loader/glyph-spark`, `thinking-orb/dotted-breathe` |
| `Search` | sweep / radar / 3-node row | `loader/dots-radar`, `loader/nodes-3` |
| `Read` | scanline top→bottom | `loader/disc-scanline`, `loader/glyph-block-wave` |
| `Edit` / `Write` | fill / caret / draw-on | `loader/glyph-write`, `loader-mascot/mark-draw-on` |
| `Run` | brisk spin / 4-node square | `loader/ring-comet`, `loader/nodes-4` |
| `Browse` | orbit / handoff | `loader/orbit-3` |
| `Generate` (image, media) | fill / bloom | handed to family D container loaders |
| `Wait` (approval) | hourglass / tip | `loader/glyph-sand`, `loader/hourglass` |

## 4. Micro-interactions (shared)
- **Enter:** fade + scale 0.8 → 1 over `dur.base` (160–200 ms) `ease.out`, already in phase.
- **Phase morph:** tempo changes ease over 400 ms (phase integral, §2.1); colour lerps in OKLab;
  motion-family swaps (e.g. spin → hourglass) crossfade over 200 ms with both drawings phase-locked.
- **Submitted pulse:** 1 × 400 ms ring, scale 1 → 1.4, opacity 1 → 0 (same as the status strip).
- **Steer blip:** brightness +25 % for 240 ms, `ease.out`.
- **Stall ramp:** 2 s to `warn`; on the next event colour snaps back over 200 ms `ease.out`.
- **Done:** loop finishes its current beat (≤ 200 ms) then morphs to ✓ with `spring.snap`; hold
  600 ms; fade 240 ms `ease.in`. No celebration by default; Pro variants may add an arrival ring
  (idea from the shockwave post [R039]).
- **Failed:** morph to ✕ / `!` over 160 ms `spring.snap`, then a 2-cycle 3 px horizontal nudge
  (300 ms), then static.
- **Hover** (only where a loader is interactive, e.g. a mascot or a tool row): tooltip with the
  elapsed time and verb after 400 ms; no motion change.

## 5. Timing (shared defaults; each variant spec may override)
Reuses the composer motion tokens (`dur.*`, `ease.*`, `spring.*`, `stagger`) from
[composer.md](composer.md) and adds loader tokens (all proposed):

| Token / transition | Value | Easing / spring | Notes |
|---|---|---|---|
| `loop.calm` | 1600–2400 ms | sine / `ease.in-out` | thinking, idle breathe |
| `loop.base` | 1000–1200 ms | linear (rotation) | default spinners |
| `loop.brisk` | 700–900 ms | linear | tool running (= base ÷ 1.25) |
| `frame.glyph` | 80 / 100 / 120 / 160 / 200 ms | step | per frame table |
| `fps.vector` / `fps.particle` / `fps.shader` | 60 / 60 / 60 (caps) | — | §2.1 rate policy |
| `fps.pixel` | 7.5 or 15 | step | pixel mascots |
| Enter | `dur.base` 180 ms | `ease.out` | scale 0.8 → 1 + fade |
| Exit | 240 ms | `ease.in` | after Done hold |
| Submitted pulse | 400 ms × 1 | `ease.out` | |
| Tempo change | 400 ms | `ease.in-out` on speed | phase-continuous |
| Motion-family crossfade | 200 ms | `ease.in-out` | |
| Steer blip | 240 ms | `ease.out` | |
| Stall ramp / snap-back | 2000 ms / 200 ms | linear in OKLab / `ease.out` | |
| Done morph / hold / fade | 160 ms / 600 ms / 240 ms | `spring.snap` / — / `ease.in` | |
| Failed morph + nudge | 160 ms + 300 ms (2 × 3 px) | `spring.snap` / linear | |
| `show_delay` / `min_visible` / `handoff_grace` | 300 / 600 / 250 ms | — | §3.3 |
| **Reduced motion** | still frame per phase; phase changes 0–80 ms opacity crossfade; no loops (opt-in 2 s SlowFade) | — | §2.6 |

## 6. Variant catalog (180)
Names follow [../variants.md](../variants.md). Counts per style family; each spec lists every name
with a one-line idea. **F** = free, **P** = Pro.

| Style family | Sub-family | F | P | Owning spec |
|---|---|---|---|---|
| **Glyph** | braille (classic, sand, packs) | 2 | 6 | loader-glyph |
| | block / bar (░▒▓█, ▁…█) | 2 | 4 | loader-glyph |
| | dots / line / arcs / quadrants / bloom / half-rotation | 6 | 0 | loader-glyph |
| | bullet blink, scanner, spark (own star) | 3 | 0 | loader-glyph |
| | semantic, matrix / ASCII, braille raster, pixel-text, choreography, misc | 1 | 20 | loader-glyph |
| | *glyph subtotal* | *14* | *30* | **44** |
| **Vector** | dots (typing, pulse, wave, grid, radar, gooey, matrix, chase) | 5 | 9 | loader-vector |
| | arcs & rings (arc, dash, comet, dual, traces, clock, radar) | 4 | 5 | loader-vector |
| | bars | 2 | 3 | loader-vector |
| | discs (SVG motifs) | 2 | 8 | loader-vector |
| | morphs | 1 | 4 | loader-vector |
| | orbits & node graphs | 2 | 6 | loader-vector |
| | math curves, pixel grid, completion, hourglass, ripple, conic | 0 | 9 | loader-vector |
| | *vector subtotal* | *16* | *44* | **60** |
| **Orb** | dotted canvas (thinking-orbs port and extensions) | 3 | 3 | thinking-orb |
| | vector orbs (conic, blob, pulse rings) | 3 | 0 | thinking-orb |
| | shader fluids (liquid, nebula, ember, smoke, ferro, vortex, aurora, lava, prism, gem) | 0 | 15 | thinking-orb |
| | particle / metaball / dither / halftone / wireframe / node / voice-ready / arrival | 0 | 12 | thinking-orb |
| | *orb subtotal* | *6* | *30* | **36** |
| **Mascot & busy glow** | mascots & marks (incl. Oxyn mark) | 4 | 18 | loader-mascot |
| | composer border light / busy glow / window edge | 4 | 14 | loader-mascot |
| | *mascot subtotal* | *8* | *32* | **40** |
| **Family B total** | | **44** | **136** | **180** |

Counting rule: a variant differs in structure or motion (variants.md). Colour, size and speed
presets are tokens and are **not** counted. Frame packs count one variant per distinct frame
sequence because the motion differs.

## 7. Edge cases (family level)
- **Many loaders at once** (20 tool rows, 40 sidebar threads): busy-region rules in §2.3; at most 8
  list rows animate; total budget §2.7.
- **Very fast agent:** `show_delay` hides sub-300 ms thinking; Submitted→Done in one frame draws
  nothing.
- **Very slow agent (hours):** phase is computed from the clock, so no drift or float blow-up (phase
  integral is wrapped mod 1 every loop). After **10 min** in one phase the lead loader drops to
  `loop.calm` tempo and particle/shader loaders drop one tier to save power (proposed); the elapsed
  timer remains the liveness cue.
- **Offline / slow network:** shows as `Stalled` via oxyn-core; the banner explains (status strip).
- **Window hidden / laptop lid / tab hidden:** paused; resume at the current phase.
- **GPU loss, WebGL2 unavailable, software rendering:** fallback chain §2.2; a T4 variant must look
  intentional at T3.
- **RTL:** rotations stay clockwise (they are not reading-direction); directional motion (scanner,
  fill, handoff, border-light travel start point) mirrors.
- **Light / dark:** each variant is checked in both; orbs and glows can carry a `dark` tuning flag
  (Libraries.dev does separate light/dark tuning, research note 16).
- **Terminal surface:** only T1 glyph variants and T0 stills; non-truecolor terminals get the
  monochrome frame set with no colour ramp (Codex shows static dim text there, research note 08).
- **Embedding inside text:** glyph and 12–16 px vector loaders align to the text baseline and never
  change line height.
- **Touch:** loaders are not targets; when a loader sits on a target (tool row), the row's 44 px hit
  area applies.

## 8. Accessibility (family level)
- **Graphics are decorative.** The drawn loader is `aria-hidden="true"` (GPUI: not exposed as an
  accessible node); its meaning is carried by the region's label and text.
- **`aria-busy="true"`** goes on the content being produced (the message or tool card), not on the
  loader, and is removed on Done/Failed/Cancelled (affectUI's Shimmer Loader documents the same
  pattern, research note 11).
- **Live regions:** loaders never own one. The composer's single polite live region
  ([composer.md](composer.md) §9) announces phase changes it is given: "Agent thinking", "Running
  search", "Waiting for your approval" (assertive), "Agent may be stalled" (at `stall_hard` only),
  "Done", "Failed" (assertive). Tool-row loaders announce nothing; their row label changes ("Running
  search" → "Searched 12 sources"). Verb changes are debounced to once per 10 s (status strip rule).
- **Labels per phase** (default English, i18n via `LoaderLabels`): Submitted "Sending", Thinking
  "Thinking", Working "Running <tool>", Streaming "Writing", Waiting "Waiting for you", Stalled
  "No response for <n> seconds", Done "Done", Failed "Failed", Cancelled "Stopped".
- **No flashing:** no loader changes the luminance of an area larger than its own box, and no
  whole-loader blink or flash runs faster than **3 per second** (WCAG 2.3.1 general threshold).
  Frame cycles of a 12–48 px glyph or dot are motion, not flashes; glows and rims ease, never blink.
- **`prefers-reduced-motion` / OS reduce motion:** §2.6. Studio and gallery show the still frames
  next to every variant.
- **Contrast:** moving parts ≥ 3:1 against the background (non-text contrast), tracks are
  decorative; `waiting`, `warn`, `danger` states also change shape or glyph, never colour alone.
- **Focus:** loaders are never focusable.

## 9. Props sketch (shared)
```rust
pub struct Loader {
    pub agent: AgentState,              // same value as ComposerState.agent; phase derived from it
    pub variant: LoaderVariant,         // Glyph(..) | Vector(..) | Orb(..) | Mascot(..) | Glow(..)
    pub size: LoaderSize,               // Px12 | Px16 | Px20 (default) | Px24 | Px32 | Px48
    pub tone: LoaderTone,               // Current (default) | Accent | Spectral | Token(TokenId)
    pub region: BusyRegion,             // Composer | TranscriptTail | ToolRow(id) | ListRow(id) | Avatar(id) | Window
    pub priority: BusyPriority,         // Lead | Normal (default) | Mirror
    pub speed: f32,                     // multiplier on the variant's base tempo, clamp 0.5..=2.0
    pub tier: TierPolicy,               // Auto (default) | Max(RenderTier) | Exact(RenderTier)
    pub activity: Option<Activity>,     // override of the activity derived from ToolRunning.tool
    pub timing: LoaderTiming,           // show_delay 300 ms, min_visible 600 ms, handoff_grace 250 ms
    pub stall: StallThresholds,         // shared with the status strip: soft 5 s, hard 45 s, tool 120 s
    pub reduced_liveness: ReducedLiveness, // Static (default) | SlowFade
    pub labels: LoaderLabels,
    pub motion: MotionLevel,            // None | Subtle | Expressive
}

pub enum LoaderPhase {
    Hidden, Rest, Submitted, Thinking, Working { activity: Activity }, Streaming,
    Waiting, Stalled { level: StallLevel }, Done, Failed { kind: ErrorKind }, Cancelled,
}

impl LoaderPhase {
    /// Pure mapping of §3.2. `stall` comes from oxyn-core, never from the app.
    pub fn derive(agent: &AgentState, stall: Option<StallLevel>, tool: Option<&ToolRef>) -> Self { /* … */ }
}

pub enum RenderTier { Still, Glyph, Vector, Particle, Shader }
pub enum Activity { Think, Search, Read, Edit, Run, Browse, Generate, Wait, Listen }
```
Each style-family spec adds its own variant enum and props (frame table, geometry, palette).

## 10. Related components and references
**Related:** status strip (owns elapsed, Stop, Retry; [composer-status-strip.md](composer-status-strip.md)) ·
input shell busy tint and `composer-input/border-light` ([composer-input-shell.md](composer-input-shell.md)) ·
send control `particle` / `orbit` / `border-beam` Pro variants ([composer-send-control.md](composer-send-control.md)) ·
family C status-text effects (verb, shimmer, "Worked for 12s") · family D container loaders ·
family F tool rows · family K voice orbs · family L animated marks.

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI motion module (research note 08) | braille at 100 ms, `◌` reduced-motion still, repaint only while effects run, `•`/`◦` blink 600 ms | licensing.md lists MIT, note 08 lists Apache-2.0: either way **portable with attribution**; re-check before porting |
| OpenCode / Kilo (research note 08) | braille 80 ms, one-cell frame gallery, Soft Radar 5×5 at 1.8 s, phase sync, offscreen pause, radar frozen at 1/5 under reduced motion | MIT — port with attribution |
| MonoCode GPUI pulse clock (research note 08) | leased 33 ms clock, 30/15/7.5 fps, 8× slower in background | MIT — port with attribution |
| Cline thinking grace (research note 08) | 500 ms grace before "Thinking…" | Apache-2.0 — port with attribution |
| Claude Code spinner (research notes 08, 09) | stall-to-red idea, mirrored breathing glyph idea | Proprietary (leaked source) — **reference only, no code, no frame copy** |
| Claude Desktop node mark (research note 08) [RE] | 3-node vs 4-node semantic mark; 20 px legibility failure | Proprietary — reference only |
| Zed SpinnerLabel (research note 08) | Dots vs Sand for generating vs awaiting | Mixed licence — study only |
| Warp warping indicator (research note 08) | choreographed turn + flourish; one clock for spinner, shimmer, counter | AGPL — study only |
| BoardUI [R095] | one `waiting` flag drives glow, thinking row, controls | Free tier MIT; Pro study only |
| loading.dev [R090] | one small prop contract (`size`, `color`, `duration`) for every loader | MIT — port with attribution |
| Loading UI [R091] | agent-verb loaders as a category; block-glyph loaders | Open source (licence to confirm per item) |
| Circle Loaders (research note 02) | verb ↔ loader pairing; reduced-motion-safe SVG | Licence not established — study only |
| thinking-orbs / Libraries.dev [R068] [R100] | 9 verb-named orb states, per-size tuning, light/dark flag | MIT (packages) — port with attribution |
| MetalForge (research notes 02, 13) | shader orb/loader palette, live sliders | Paid — study only |
| ittybitty Edge Glow (research note 02) | pauses offscreen, still frame under reduced motion | MIT per note 02 — confirm before porting |
