**Status:** draft v0.1 (2026-11-11).

# Spec: vector loaders (dots, arcs, rings, bars, discs, morphs, orbits)

Part of the loaders family ([loaders.md](loaders.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **16 free + 44 Pro = 60 variants**, render
tier **T2** (paths, quads, opacity and transform only), all driven by the shared `LoaderPhase`
derived from `AgentState`.

## 1. Purpose
Vector loaders are the GUI counterpart of glyph loaders: the 12–48 px mark beside "Thinking", on a
running tool row, in a sidebar thread, on a card. This is where most kits stop at one spinner
(prompt-kit ships 12 loaders, loading.dev 36, LDRS 44; research note 02). Our difference is not the
count; it is that **every vector loader has a designed pose for each agent phase**: a calm loop for
thinking, a brisker one for tools, a **different** motion for waiting on the user, a stall slowdown,
and a morph into ✓ or ✕ at the end. OpenCode's Soft Radar shows the system idea: one AI activity
spinner used on tabs, chips, sub-agents and tool cards, phase-synced, paused offscreen (research
note 08).

Rules:
1. Only **opacity and transform** animate per frame where the platform composites them (Dioxus web:
   compositor CSS/WAAPI; GPUI: redraw of small paths within the T2 budget).
2. Every variant is **tuned per size** (12/16/20/24/32/48): stroke, dot count, gaps. No stroke below
   1.5 px at 12–16 px; detail that can't survive 16 px is removed at that size, not shrunk.
3. **Morphs need matching point counts.** Every morphable shape is resampled to 32 points
   (proposed) so shape → shape → ✓ tweens cleanly (the same-point-count rule from research note 09).
4. Phase-locked to the shared clock; tempo changes are phase-continuous ([loaders.md](loaders.md) §2.1).

## 2. State machine
```
               send                      first event
  Hidden ──────────────▶ Submitted ────────────────▶ Thinking (calm loop, 1.0×)
    ▲      (one 400 ms ring pulse, no loop)              │  ▲
    │                                        tool starts │  │ tool ends
    │                                                    ▼  │
    │                                       Working{activity} (1.25×; semantic variants swap motion)
    │                                                    │ first token
    │                                                    ▼
    │                                       Streaming (0.6×)
    │                                  approval/question │
    │                                                    ▼
    │                                       Waiting (variant's waiting pose: tip / breathe, amber)
    │
    │  any active ── 5 s no event ─▶ Stalled(soft): tempo → 0.5× over 2 s, colour → warn
    │                                     └─ 45 s ─▶ Stalled(hard): settle on still pose, warn
    │                                (event → back to previous phase; colour snaps back 200 ms)
    │
    │◀── fade ── Done: finish beat → morph to ✓ (hold 600 ms) ◀── completes
    │◀── fade ── Cancelled: freeze → desaturate 160 ms ◀── Stop
    │◀── dismiss ─ Failed: morph to ✕ / ! + nudge, static ◀── error
  Overlays: Reduced (still pose per phase) · Paused (offscreen / hidden) · Mirrored (still pose)
```

## 3. States table
Vector loaders are not interactive; actions belong to the host.

| Phase | Shows (default; per-variant poses in §5.1) | Host actions | Keyboard |
|---|---|---|---|
| `Hidden` | nothing | — | not focusable |
| `Submitted` | track appears with one ring pulse (scale 1 → 1.4, opacity 1 → 0, 400 ms) | Stop | — |
| `Thinking` | variant loop at base tempo | Stop; click row → live step | — |
| `Working{activity}` | loop at 1.25× or the activity motion (§5.3) | Stop | — |
| `Streaming` | loop at 0.6× | Stop | — |
| `Waiting` | waiting pose in `color.loader.waiting`: rotation stops, shape **tips** 180° every 2 s (rings/arcs) or all dots breathe together (dots/bars) | Review | — |
| `Stalled(soft)` | tempo eases to 0.5×; colour ramps to `warn` over 2 s | Stop | — |
| `Stalled(hard)` | still pose in `warn` | Retry / Stop | — |
| `Done` | morph to ✓ in `success`, hold 600 ms, fade | — | — |
| `Failed` | morph to ✕ (or `!` for `kind = Timeout`) in `danger`, 2-cycle 3 px nudge, static | Retry | — |
| `Cancelled` | freeze, desaturate to `fg` 40 %, fade | — | — |
| Reduced / Mirrored | designed still pose per phase (§5.1 last column) | — | — |

## 4. Micro-interactions
- **Enter:** scale 0.8 → 1 + fade over 180 ms `ease.out`, already at the clock's phase.
- **Submitted:** a single ring pulse; the loop starts only when `Thinking` arrives.
- **Tempo:** speed eases over 400 ms; the drawing never jumps (phase integral).
- **Waiting tip:** ring/arc variants stop rotating at their current angle, then tip 180° over 600 ms
  `ease.in-out` every 2 s, like an hourglass being turned (the status strip uses the same 2 s tip for
  "awaiting you", after Zed's idea that human-blocked should look different; research note 08).
  Dot/bar variants stop their wave and breathe together (opacity 0.5 ↔ 1, 2 s).
- **Stall:** 2 s tempo + colour ramp; at `stall_hard` the loop eases to its still pose over 400 ms.
- **Done morph:** the loop finishes its current beat (≤ 200 ms), then: arcs/rings sweep closed into a
  full circle (160 ms) and a ✓ path draws inside (160 ms, `spring.snap`); dots converge to the centre
  (160 ms) and the ✓ draws from that point; bars collapse to a baseline that bends into ✓. Optional
  Pro `ring-arrival`: one outward ring 400 ms.
- **Failed morph:** same convergence into ✕ (two strokes, 40 ms stagger), then a 2-cycle 3 px nudge.
- **Mirrored:** when another loader leads the region, this one eases to its still pose over 200 ms.

## 5. Timing

### 5.1 Free variants: geometry, loop and poses (all values proposed unless a source is named)
| Variant | Geometry at 20 px (scales per §5.2) | Loop | Easing / stagger | Waiting pose | Still pose (reduced) |
|---|---|---|---|---|---|
| `loader/dots-typing` | 3 dots ⌀ 3 px, gap 2.5 px, rise 3 px | 1400 ms | `ease.in-out`, stagger 160 ms | dots level, breathe together | three dots, middle raised 1.5 px |
| `loader/dots-pulse` | 1 dot ⌀ 8 px | 1600 ms | sine; scale 0.7 ↔ 1, opacity 0.5 ↔ 1 | slower 2400 ms, amber | dot at scale 0.85 |
| `loader/dots-wave` | 4 dots ⌀ 3 px in a row | 1200 ms | opacity 0.2 → 1 → 0.2, stagger 120 ms | breathe together | dots at 0.2/0.5/0.8/1 |
| `loader/dots-grid` | 3 × 3 dots ⌀ 3 px, pitch 6 px | 1200 ms | diagonal opacity wave, stagger 100 ms per diagonal | centre dot only breathes | diagonal frozen mid-wave |
| `loader/dots-radar` | 5 × 5 dots, 2 px at 3 px pitch in a 16-unit box (3 × 3 at 12 px); centre steady | 1800 ms | head sweeps each square ring clockwise; trail fades over 48 % of the loop; unlit 0.2 | sweep stops; centre breathes amber | sweep frozen 1/5 of the way round |
| `loader/arc` | 90° arc, stroke 2 px, round cap, track at 20 % | 900 ms | linear rotation | tip 180° every 2 s | arc at 12 o'clock to 3 o'clock |
| `loader/ring-dash` | arc length 30° ↔ 270° while rotating | dash 1400 ms, rotation 2000 ms | `ease.in-out` dash, linear spin | arc fixed at 270°, tips | 270° arc |
| `loader/ring-comet` | 300° conic tail fading to transparent, stroke 2 px | 1000 ms | linear | comet stops; head breathes | comet frozen at 45° |
| `loader/ring-dual` | outer 120° arc, inner 90° arc at 60 % radius (single arc at 12–16 px) | outer 1200 ms, inner 900 ms counter | linear | both stop, outer tips | both arcs at opposite angles |
| `loader/bars-eq` | 3 vertical bars 3 px wide, gap 2 px, height 30 % ↔ 100 % | 1000 ms | `ease.in-out`, stagger 150 ms | bars at 50 %, breathe together | bars at 40/100/70 % |
| `loader/bars-skeleton` | 4 rounded bars 3 px tall, widths 80/40/60/30 % (for ≥ 24 px rows) | 1800 ms | opacity 0.15 → 1 → 0.15, stagger 120 ms | all bars at 0.5, breathe | bars at 0.3/0.6/0.9/0.5 |
| `loader/disc-latitude` | disc outline + 5 latitude ellipses whose `ry` oscillate (fake rotation) | 2400 ms | sine, phase offset 1/5 per line | lines stop; disc tips | lines at evenly spaced latitudes |
| `loader/disc-halftone` | dot grid clipped to a circle; dot radius modulated by a travelling sine | 2000 ms | sine across x | wave stops; disc breathes | static halftone gradient |
| `loader/morph-squircle` | square (corner 20 %) → circle → square, +45° per morph, 32-point paths | 1600 ms | `spring.soft` per morph | holds circle, breathes | circle |
| `loader/orbit-3` | 3 dots ⌀ 3 px on a 7 px radius at 120° | 1200 ms | `ease.in-out` per revolution (dots bunch and spread) | dots stop at 120°, breathe | dots at 120° |
| `loader/orbit-merge` | 3 dots orbit, converge into one centre dot that breathes, then split | 2400 ms (orbit 1000, merge 300, breathe 800, split 300) | `ease.in-out` | stays merged, amber breathe | merged dot |

### 5.2 Per-size tuning (proposed)
| Size | Dot ⌀ | Stroke | Typing rise | Radar grid | Ring-dual | Notes |
|---|---|---|---|---|---|---|
| 12 px | 2 px | 1.5 px | 1.5 px | 3 × 3 | single arc | discs become `dots-pulse` |
| 16 px | 2.5 px | 1.5 px | 2 px | 3 × 3 | single arc | halftone ≤ 12 dots |
| 20 px | 3 px | 2 px | 3 px | 5 × 5 | dual | default |
| 24 px | 3.5 px | 2 px | 3.5 px | 5 × 5 | dual | |
| 32 px | 5 px | 2.5 px | 5 px | 5 × 5 | dual | |
| 48 px | 7 px | 3.5 px | 7 px | 7 × 7 | dual + track | discs gain detail lines |

### 5.3 Transition timing
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Enter | 180 ms | `ease.out` | scale 0.8 → 1 + fade |
| Submitted ring pulse | 400 ms × 1 | `ease.out` | |
| Tempo change (any phase) | 400 ms | `ease.in-out` on speed | Working 1.25×, Streaming 0.6×, Stall 0.5× |
| Activity motion swap | 200 ms crossfade | `ease.in-out` | both drawings phase-locked |
| Waiting tip | 600 ms every 2000 ms | `ease.in-out` | rings / arcs / discs |
| Waiting breathe | 2000 ms loop | sine | dots / bars |
| Stall ramp / snap-back | 2000 ms / 200 ms | OKLab linear / `ease.out` | |
| Stall hard settle to still | 400 ms | `ease.out` | |
| Done: close ring / draw ✓ / hold / fade | 160 / 160 / 600 / 240 ms | `ease.out` / `spring.snap` / — / `ease.in` | |
| Failed: ✕ draw + nudge | 160 ms (40 ms stagger) + 300 ms 2 × 3 px | `spring.snap` / linear | |
| Mirrored settle | 200 ms | `ease.out` | |
| **Reduced motion** | still pose per phase (§5.1); phase changes 0–80 ms opacity crossfade; no loops | — | opt-in 2 s SlowFade on the still pose |

Frame rate: up to 60 fps focused, 20 fps unfocused, paused offscreen (loaders.md §2.1). In Dioxus,
loops run as Web Animations on SVG elements with `currentTime` set from the shared clock and
`updatePlaybackRate` eased for tempo changes, so the compositor runs steady loops and phase stays
locked across instances (proposed approach).

## 6. Variants

### Free (16)
| Name | Idea |
|---|---|
| `loader/dots-typing` | Three dots rising in turn, the chat "typing" archetype |
| `loader/dots-pulse` | One breathing dot for pure pacing (the pacing-dot idea, research note 09) |
| `loader/dots-wave` | Four dots with a travelling opacity wave |
| `loader/dots-grid` | 3 × 3 grid with a diagonal wave |
| `loader/dots-radar` | 5 × 5 radar sweep, phase-synced (port of OpenCode's Soft Radar) |
| `loader/arc` | 90° arc on a faint track, linear spin |
| `loader/ring-dash` | Arc that grows and shrinks while spinning |
| `loader/ring-comet` | Conic comet tail circling |
| `loader/ring-dual` | Two counter-rotating arcs (single arc at small sizes) |
| `loader/bars-eq` | Three equaliser bars |
| `loader/bars-skeleton` | Four horizontal bars pulsing in turn, a mini "generating" skeleton |
| `loader/disc-latitude` | Disc with latitude lines that fake a slow rotation |
| `loader/disc-halftone` | Halftone disc with a travelling dot-size wave |
| `loader/morph-squircle` | Square ↔ circle morph with a quarter turn |
| `loader/orbit-3` | Three dots orbiting, bunching and spreading |
| `loader/orbit-merge` | Three dots orbit, merge, breathe and split |

### Pro (44, in 16 groups)
| Group (count) | Names | One-line idea |
|---|---|---|
| Gooey dots (3) | `loader/dots-gooey-merge`, `-split`, `-drip` | Metaball-style dots that fuse and separate (gooey SVG filter on web, thresholded field on GPUI) |
| Dot matrix (4) | `loader/matrix-ripple`, `-spiral`, `-checker`, `-rain` | 4 × 4 dot matrix pulsing in four patterns (idea from transitions.dev's matrix loader, study only) |
| Chase (2) | `loader/dots-leapfrog`, `loader/dots-chase` | Dots hop over each other; dots chase round a square |
| Shape traces (3) | `loader/trace-squircle`, `-reuleaux`, `-hex` | A dash travels the outline of a squircle, Reuleaux triangle, hexagon (loading.dev's Trace idea) |
| Clock & radar (2) | `loader/ring-clock`, `loader/ring-radar` | Two-hand clock sweep; radar wedge with afterglow |
| Discs (8) | `loader/disc-scanline`, `-moire`, `-lissajous`, `-helix`, `-constellation`, `-iris`, `-gyroscope`, `-rotor-text` | Agent-verb disc motifs, each with a default verb (reading, resolving, tracing, unzipping, traversing, focusing, orienting; rotor = ring of text "THINKING · WORKING"); idea from Circle Loaders, drawings our own |
| Morphs (4) | `loader/morph-shapeshift`, `-star`, `-blob`, `-square-breathe` | Sparkle → rounded triangle → gear → hexagon with +45° per step (Gemini-like idea, research note 09); star ↔ pentagon; noise blob; square breathing |
| Orbits (4) | `loader/orbit-gimbal`, `-atom`, `-moon`, `-comet-pair` | Three tilted rings at 1 : 2 : 3 turns per loop so it closes seamlessly (Originkit craft rule, research note 16); atom; planet and moon; two comets |
| Node graphs (2) | `loader/nodes-3`, `loader/nodes-4` | Semantic marks: 3 nodes in a row for search, 4 in a square for tools; **edges ≥ 1.5 px and dots ≥ 3 px at 20 px** so they don't dissolve (the Claude Desktop lesson, research note 08) |
| Bars (3) | `loader/bars-wave`, `-stack`, `-signal` | Five-bar sine wave; bars stacking and unstacking; signal-strength bars climbing |
| Math curves (3) | `loader/curve-cardioid`, `-rose`, `-lemniscate` | A dot and fading trail drawing a parametric curve (idea from Math Curve Loaders [R113]) |
| Pixel grid (2) | `loader/pixel-drive`, `loader/pixel-surfer` | 6 × 6 pixel grid at 15 fps: a road scrolling; a wave being ridden |
| Completion (1) | `loader/ring-check` | Spinner whose arc closes and draws ✓ as part of the loop vocabulary, with a determinate mode |
| Hourglass (1) | `loader/hourglass` | Vector hourglass that drains and turns; default waiting pose for any vector loader that opts in |
| Ripple (1) | `loader/ripple` | Concentric rings emitted from the centre every 800 ms, fading out |
| Conic sweep (1) | `loader/conic-sweep` | Filled disc with a conic gradient sweeping round (no stroke), legible at 12 px |

## 7. Edge cases
- **Tiny sizes (12–16 px).** Per-size tuning (§5.2). Variants with detail that can't survive (discs,
  gimbal, constellation) declare `min_size` and fall back to a named small variant (`dots-pulse` or
  `arc`).
- **Hairline check.** `xtask render-posters` renders every variant at 16 and 20 px in light and dark;
  any stroke rendering under 1 px or any dot under 2 px fails the build.
- **Dense IDE panels.** Many running rows: region rules ([loaders.md](loaders.md) §2.3); non-lead rows
  show the still pose. At 12 px in 22 px rows, the loader is vertically centred on the text's x-height.
- **Inline in text.** 12–16 px loaders align to the text baseline (`vertical-align: -0.125em`
  equivalent) and never change line height.
- **RTL.** Rotation stays clockwise; directional motion (dots-wave, typing order, pixel-drive,
  leapfrog) mirrors.
- **Light / dark.** Track alpha 20 % in dark, 16 % in light (proposed) so tracks don't look heavier on
  white; checked in the poster run.
- **Long labels / overflow.** The loader box is fixed; the label truncates.
- **Very fast agent.** `show_delay` hides it; a Done arriving during the enter animation skips to the ✓
  morph directly.
- **Very slow agent.** After 10 min the loop drops to `loop.calm` tempo (loaders.md §7).
- **Slow / janky frames.** Phase is time-based, so dropped frames skip ahead rather than slowing down;
  morphs complete at their scheduled time.
- **Many instances.** All instances of a variant share one path cache per size; only transforms differ.
- **Touch.** Not a target; host rows keep 44 px hit areas.
- **High-contrast / forced colours.** Under forced colours (Windows), tracks are dropped and the moving
  part uses the system text colour; the still pose must still read.
- **Printing / static export.** Uses the still pose.

## 8. Accessibility
- Decorative: `aria-hidden="true"` / not exposed in GPUI. Labels live on the host ("Running search,
  8 seconds").
- `aria-busy="true"` on the content being produced, set by the host, removed on Done / Failed /
  Cancelled.
- No live-region output from the loader; phase announcements come from the composer's single polite
  live region (assertive for waiting and failed) per [loaders.md](loaders.md) §8.
- **Flashing:** no variant blinks the whole loader faster than 3 per second; ripple emits every
  800 ms (1.25 Hz); opacity waves move across dots rather than flashing the whole box.
- **`prefers-reduced-motion`:** designed still pose per phase; loops stop; phase changes crossfade.
- **Contrast:** moving part ≥ 3:1 against background; tracks are decorative and may be lower;
  waiting/stalled/failed change shape (tip, still pose, ✕), not only colour.
- **Not focusable.** Never in the tab order.

## 9. Props sketch
```rust
pub struct VectorLoader {
    pub agent: AgentState,                // LoaderPhase::derive(&agent, stall, tool)
    pub variant: VectorVariant,           // DotsTyping | DotsRadar | Arc | RingComet | … | Pro(VectorPro)
    pub size: LoaderSize,                 // Px12..Px48; per-size geometry table, not scaling
    pub tone: LoaderTone,                 // Current | Accent | Spectral | Token(..)
    pub track: TrackStyle,                // Faint (20 %) | None
    pub region: BusyRegion,
    pub priority: BusyPriority,
    pub speed: f32,                       // clamp 0.5..=2.0
    pub activity_motion: bool,            // allow semantic swaps (e.g. nodes-3 for Search)
    pub waiting: WaitingPose,             // Variant (default) | Hourglass | Breathe
    pub completion: CompletionMorph,      // Check (default) | Fade | Arrival (Pro)
    pub determinate: Option<f32>,         // 0.0..=1.0 for ring-check / progress-capable variants
    pub labels: LoaderLabels,
    pub motion: MotionLevel,
}

/// Static per-variant data in oxyn-core.
pub struct VectorSpec {
    pub id: &'static str,
    pub sizes: [SizeTuning; 6],           // stroke, dot, counts, radii per size
    pub loop_ms: u16,
    pub stagger_ms: Option<u16>,
    pub min_size: LoaderSize,
    pub fallback: Option<&'static str>,
    pub morph_points: u8,                 // 32 for morphable shapes
    pub still: StillPoses,                // per phase
}
```

## 10. Related components and references
**Related:** [loader-glyph.md](loader-glyph.md) (text versions of the same motions) ·
[thinking-orb.md](thinking-orb.md) (vector orbs are the T2 fallback of particle orbs) · status strip
dot ([composer-status-strip.md](composer-status-strip.md)) · send control `ring-progress` / `orbit`
([composer-send-control.md](composer-send-control.md)) · context meter rings · family F tool rows ·
family D container loaders (shape traces scale up into border traces).

| Reference | What we take | Licence note |
|---|---|---|
| OpenCode Soft Radar, Loader-v2, phase sync (research note 08) | 5 × 5 radar 1.8 s, 48 % trail, unlit 0.2, freeze at 1/5 for reduced motion; 900 ms ring spin with 0.2 track | MIT — port with attribution |
| loading.dev [R090] (research note 02) | `size` / `color` / `duration` contract; Trace (dash round a rounded square) | MIT — port with attribution |
| Uiverse [R093] | community loader ideas | All elements MIT — port with author credit |
| prompt-kit, GradientDeck, SpinKit, LDRS (research note 02) | market baseline of shapes | Licences not checked here — study only until checked |
| Circle Loaders (research note 02) | verb-paired disc motifs, reduced-motion-safe SVG | Licence not established — study only; our drawings are new |
| Claude Desktop node mark (research note 08) [RE] | 3 vs 4 nodes = search vs tool; 20 px failure | Proprietary — reference only |
| Claude desktop quiet dots (research note 08) [RE] | typing-dot rhythm as a reference point | Proprietary — reference only; our timings differ |
| Zed generating skeleton (research note 08) | staggered pulsing bars idea | Mixed licence — study only; our widths and timings |
| transitions.dev matrix loader (research note 17) [R132] | 16-dot matrix, four patterns (idea) | Forbids competing kits — study only |
| Originkit Particle Gimbal (research note 16) | integer turn ratios, back-to-front painting | Paid / unclear — study only |
| Math Curve Loaders [R113] | parametric curve loaders | See notes — study only |
| Gemini shape morph, community Claude "Orbit" (research note 09) | shapeshift sequence; orbit-merge-breathe | Proprietary / unknown — study only |
