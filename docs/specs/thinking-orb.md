**Status:** draft v0.1 (2026-11-11).

# Spec: thinking orb (particle and shader orbs)

Part of the loaders family ([loaders.md](loaders.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **6 free + 30 Pro = 36 variants**, render
tiers **T3 particle** (CPU dots) and **T4 shader** (GPU), each with a vector (T2) and still (T0)
fallback. Driven by the shared `LoaderPhase` derived from `AgentState`, through one continuous
**energy** value so the orb never snaps between states.

Voice orbs (listening / speaking with audio level) are family K; they reuse this engine. This spec
covers the orb as a **thinking / working indicator** at 16–48 px and as an empty-state presence at
64–256 px.

## 1. Purpose
The orb is the agent's "presence": an avatar-sized indicator that shows *how hard it is working*
rather than just *that* it is working. References treat the orb as a state machine: thinking-orbs by
Jakub Antalik ships nine verb-named states (working, searching, solving, listening, connecting,
weaving, composing, breathing, shaping) at separately tuned 20 and 64 px [R068] (research note 16);
Orbkit has idle / thinking / speaking [R067]; an AI-assist concept's orb speed tracks the state
[R019]; MetalForge has 22 shader orbs and a thinking-orbs editor with Breathe / Band / Rings /
Twinkle / Vortex motions (research notes 02, 13). thinking-orbs draws with plain 2D canvas arcs and
no shaders, which ports cleanly to GPUI paths (research note 15, creators.md).

Rules:
1. **One energy value.** Each phase sets a target `energy ∈ [0, 1]`; every motion parameter (speed,
   turbulence, dot jitter, glow, flow direction) is a function of a spring-smoothed energy, so
   phase changes are continuous.
2. **Every shader variant has a particle twin** that looks intentional, plus a vector fallback and
   designed still posters per phase.
3. **Orbs pause when not visible** and drop tier under low power ([loaders.md](loaders.md) §2.1).
4. **Size-tuned, not scaled** (20 px inline vs 48 px avatar get different dot counts and motions).
5. In an orb's region, the orb is the lead loader; text loaders next to it go still.

## 2. State machine
```
                 app ready                      send
  Off ─────────────────▶ Idle (breathing, e 0.15) ─────────▶ Submitted ("connecting": contract 300 ms, e 0.35)
                           ▲                                   │ first event
                           │                                   ▼
                           │                          Thinking ("solving", e 0.45, slow inward flow)
                           │                         tool ↓ ↑ tool ends
                           │                          Working{activity} ("working" / "searching", e 0.7)
                           │                                   │ tokens
                           │                                   ▼
                           │                          Streaming ("composing", e 0.55 + token-rate term, outward flow)
                           │                  approval / question │
                           │                                   ▼
                           │                          Waiting (amber, flow stops, slow breathe, e 0.2)
                           │
                           │   any active ── 5 s no event ─▶ Stalled(soft): desaturate, e → 0.1 over 2 s
                           │                                    └─ 45 s ─▶ Stalled(hard): still pose, warn tint
                           │
                           ├── settle 800 ms ── Done (core flash 200 ms, e spike 1.0 → 0.15) ◀── completes
                           ├── 400 ms ──────── Cancelled (flow freezes, e → 0.15, desaturate) ◀── Stop
                           └── on dismiss ──── Failed (collapse inward 400 ms, danger tint, still) ◀── error
  Overlays: Paused (offscreen / hidden) · LowPower (tier −1, 30 fps) · Reduced (still poster per phase)
            · Fallback(tier) · Mirrored (still poster)
```

## 3. States table
| Phase | Motion vocabulary (thinking-orbs name, as reference) | Energy target | Palette | Host actions | Keyboard |
|---|---|---|---|---|---|
| `Off` | not drawn | — | — | — | not focusable |
| `Idle` | breathing: radial scale 0.97 ↔ 1.0, slow axial spin | 0.15 | `accent` at 60 % chroma, dim | — | — |
| `Submitted` | connecting: dots contract to 85 % radius, then re-expand | 0.35 | accent | Stop | — |
| `Thinking` | solving: dots drift inward along latitudes, slow tumble | 0.45 | accent → `spectral[1]` drift ±20° hue over 6 s | Stop | — |
| `Working{Search}` | searching: a bright band sweeps the sphere | 0.7 | accent, brighter core | Stop | — |
| `Working{other}` | working: faster tumble, ring orbits | 0.7 | accent + `spectral` drift ±30° | Stop | — |
| `Streaming` | composing / weaving: dots flow outward, density pulses with token rate | 0.55 + 0.25 × rate | accent, outer glow | Stop | — |
| `Waiting` | flow stops; hold-and-breathe (scale 0.96 ↔ 1.0, 2.4 s) | 0.2 | `waiting` (amber) | Review | — |
| `Stalled(soft)` | everything slows; jitter fades | → 0.1 over 2 s | desaturate 60 %, drift to `warn` | Stop | — |
| `Stalled(hard)` | still pose | 0 | `warn` tint | Retry / Stop | — |
| `Done` | core flash, then settle to idle (shaping: brief circle → settle) | 1.0 spike → 0.15 | `success` flash 200 ms → idle palette | — | — |
| `Failed` | collapse inward to 70 % radius, stop | 0 | `danger` | Retry | — |
| `Cancelled` | flow freezes, eases to idle | 0.15 | desaturated | — | — |
| Reduced / Mirrored | designed still poster per phase | — | phase palette | — | — |

Mapping of thinking-orbs' nine states to our phases (proposed): breathing → `Idle`; connecting →
`Submitted`; solving → `Thinking`; working → `Working`; searching → `Working{Search}`; composing →
`Streaming`; weaving → `Streaming` with several sources/sub-agents; shaping → `Done` transition;
listening → family K voice. The motions are re-implemented from our own spec; the free dotted
variants may port thinking-orbs code (MIT) with attribution.

## 4. Micro-interactions
- **Energy spring.** `energy` follows its target with a critically damped spring (stiffness 120,
  damping 22, mass 1, proposed): about 600 ms to settle, no overshoot except the Done spike.
- **Speed is integrated.** Rotation angle `θ += ω(energy) × dt`, never `θ = ω × t`, so speed changes
  don't jump the drawing.
- **Colour flow.** Phase palettes interpolate in OKLab over 600 ms; within a phase, hue drifts slowly
  (6 s period) so the orb feels alive without changing state meaning.
- **Token-rate coupling (Streaming).** Rate = tokens/s smoothed with a 500 ms exponential window,
  normalised to the session's p90; maps to density pulses. Bursty streams never strobe: coupling is
  capped at 2 pulses/s.
- **Submitted.** One contraction (300 ms `ease.in`) and re-expansion (300 ms `ease.out`), the orb's
  version of the "submitted pulse".
- **Done.** Core brightens to `success` (200 ms), a soft ring may emit (Pro `orb-shockwave`), then
  the orb settles to idle over 800 ms. No confetti.
- **Failed.** Dots pull inward 400 ms `ease.firm`, palette to `danger`, motion stops.
- **Hover** (avatar placements only): tooltip "Thinking · 42 s" after 400 ms; optional Pro drag-to-spin
  with exponential decay (trackball idea from Originkit, research note 16). No other interaction.
- **Mirrored / paused:** eases to the still poster over 200 ms; resumes at the current phase.

## 5. Timing

### 5.1 Phase transitions
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Enter (mount) | 240 ms | `ease.out`, scale 0.85 → 1 + fade | starts in phase |
| Energy change (any phase) | ≈ 600 ms settle | spring k 120, c 22, m 1 | proposed |
| Palette change | 600 ms | OKLab linear | |
| Hue drift within a phase | 6000 ms loop | sine | ±20° thinking, ±30° working |
| Idle breathe | 3200 ms loop | sine | 0.31 Hz |
| Waiting breathe | 2400 ms loop | sine | flow stopped |
| Submitted contract / expand | 300 ms / 300 ms | `ease.in` / `ease.out` | |
| Thinking tumble | 1 turn per 8 s at e 0.45 | integrated | ω scales with energy |
| Working tumble | 1 turn per 4 s at e 0.7 | integrated | |
| Search band sweep | 1600 ms | `ease.in-out` | |
| Streaming density pulse | ≤ 2 per second | follows token rate | capped |
| Stall soft ramp / recovery | 2000 ms / 400 ms | linear / spring | |
| Done flash / settle | 200 ms / 800 ms | `ease.out` / spring | |
| Failed collapse | 400 ms | `ease.firm` | |
| Tier fallback swap | 200 ms crossfade | `ease.in-out` | both phase-locked |
| **Reduced motion** | still poster per phase (pre-rendered); phase changes 0–80 ms crossfade; no tumble, breathe or hue drift | — | opt-in SlowFade 2 s |

### 5.2 Size tuning and budgets (proposed)
| Size | Use | T3 dots | T4 shader res | Frame cap | Notes |
|---|---|---|---|---|---|
| 12 px | — | — | — | — | falls back to `loader/dots-pulse` |
| 16 px | inline | 24 | — | 30 fps | dotted only; no glow |
| 20 px | inline / status | 36 | 40 × 40 | 60 fps | default inline tuning |
| 24 px | composer, buttons | 48 | 48 × 48 | 60 fps | |
| 32 px | avatar | 80 | 64 × 64 | 60 fps | |
| 48 px | avatar / card | 160 | 96 × 96 | 60 fps | default avatar tuning |
| 64–256 px | empty state, hero (and family K) | 240–400 | ≤ 512 × 512 | 60 fps | DPR capped at 2 for shaders |

Budgets per orb: T3 ≤ 0.6 ms CPU at ≤ 400 dots; T4 ≤ 0.3 ms GPU at 48 px and ≤ 1.5 ms at 256 px
([loaders.md](loaders.md) §2.7). Unfocused: 15 fps (T3) / 20 fps (T4). Hidden or offscreen: paused.

### 5.3 GPU fallback and low-power mode
```
  T4 shader ──(no GPU path / context lost / software renderer / budget miss ×60)──▶ T3 particle twin
  T3 particle ─(budget miss ×60 / LowPower at ≥ 32 px with > 2 orbs)─────────────▶ T2 vector twin
  T2 vector ──(Reduced motion / Paused / poster export)───────────────────────────▶ T0 still poster
```
- **GPUI:** T4 depends on custom-shader support in our GPUI stack, which is **not confirmed yet**;
  until it is, every T4 variant renders its T3 twin on GPUI. T3 uses batched circles.
- **Dioxus web:** WebGL2, created with `failIfMajorPerformanceCaveat: true` so software rendering
  counts as "no GPU" (proposed). On `webglcontextlost`, swap to T3 at once; try to restore after 5 s,
  at most 3 times per session.
- **Shader warm-up:** pipelines compile at first mount of the variant (or at app idle if the app opts
  in); the T3 twin shows until the first shader frame is ready, so there is no compile hitch.
- **Low-power mode** (triggered by the platform's low-power or battery-saver flag where it is exposed,
  by the app's `power: Low`, or by more than 3 animating orbs in a window): 30 fps cap, half the dots,
  T4 → T3, glow off, hue drift off, unfocused 10 fps.

## 6. Variants

### Free (6)
| Name | Tier | Idea |
|---|---|---|
| `thinking-orb/dotted-breathe` | T3 | Fibonacci-lattice dot sphere, depth-sorted, that breathes and tumbles; the reference orb (port of thinking-orbs, MIT) |
| `thinking-orb/dotted-rings` | T3 | Three dot rings tumbling at 1 : 2 : 3 turns around a counter-spinning core |
| `thinking-orb/dotted-search` | T3 | Dot sphere with a bright band sweeping for search, dim for thinking |
| `thinking-orb/conic` | T2 | Conic-gradient disc with a soft inner shadow, rotating; no shader, works everywhere |
| `thinking-orb/blob` | T2 | 32-point noise blob whose wobble amplitude follows energy |
| `thinking-orb/pulse-rings` | T2 | Core dot emitting concentric rings; ring rate follows energy |

### Pro (30, in 19 groups)
| Group (count) | Names | Tier | One-line idea |
|---|---|---|---|
| Dotted extras (3) | `thinking-orb/dotted-weave`, `-shaping`, `-gimbal` | T3 | Dots braid between two latitudes; circle → triangle → square shaping on Done; tilted gimbal rings with drag-to-spin |
| Liquid (3) | `thinking-orb/liquid`, `-liquid-glass`, `-iridescent` | T4 | Fluid marbling inside a glass rim; refractive glass shell; thin-film iridescence |
| Plasma (3) | `thinking-orb/nebula`, `-plasma`, `-spectral` | T4 | Drifting gas cloud (calm thinking); plasma filaments; 4–6 stop spectral swirl |
| Fire (2) | `thinking-orb/ember`, `-solar-flare` | T4 | Smouldering surface (good for error tint); flares lift off the rim when working |
| Smoke (1) | `thinking-orb/smoke` | T4 | Billowing smoke ball, slow and soft |
| Ferrofluid (1) | `thinking-orb/ferrofluid` | T4 | Spikes rise with energy and relax when idle |
| Vortex (1) | `thinking-orb/vortex` | T4 | Inward spiral that tightens with energy |
| Aurora (1) | `thinking-orb/aurora-band` | T4 | Aurora curtain band wrapping the sphere |
| Lava (1) | `thinking-orb/lava-lamp` | T4 | Slow rising blobs; calmest shader option |
| Prism (1) | `thinking-orb/prism` | T4 | Faceted refracting core that rotates |
| Gem (1) | `thinking-orb/gem` | T4 | Cut-gem facets catching a moving light |
| Particles (2) | `thinking-orb/swarm`, `-cymatics` | T4 (T3 twin) | GPU particle swarm; sand that settles into a pattern and disperses (idea from [R001]) |
| Metaballs (2) | `thinking-orb/metaball`, `-gooey` | T4 / T3 | Merging spheres; gooey dots that pinch off when streaming |
| Low-res (2) | `thinking-orb/dither`, `-pixel` | T3 | Ordered-dither shading; 12 × 12 pixel sphere at 15 fps |
| Halftone (1) | `thinking-orb/halftone` | T3 | Halftone sphere lit by a rotating light |
| Wireframe (1) | `thinking-orb/wireframe-glass` | T2 / T4 | Wireframe glass sphere with light beams (idea from [R008]) |
| Node orb (1) | `thinking-orb/node` | T3 | Dots connected by short edges, a sphere of a graph; edges brighten on tool calls |
| Voice-ready (2) | `thinking-orb/voice-ripple`, `-voice-swirl` | T3 / T4 | Same orb accepts an audio level (family K): ripples when listening, swirl when thinking |
| Arrival (1) | `thinking-orb/orb-shockwave` | T4 | Done emits a refractive shockwave ring (idea from [R039]) |

Each T4 variant's `meta.toml` names its T3 twin (e.g. `nebula` → `dotted-breathe` with the nebula
palette) and its T2 fallback (`conic`).

## 7. Edge cases
- **Light themes.** Additive glow vanishes on white: each variant has a `dark` tuning flag (as
  Libraries.dev does, research note 16); light tuning uses normal blending, deeper palette steps and a
  1 px rim at 3:1.
- **Transparent / glass backgrounds.** Orbs render premultiplied alpha; glow is clipped to the orb's
  box + 25 % so it never tints neighbouring text.
- **Tiny sizes.** 16 px gets 24 dots and no glow; 12 px falls back to `loader/dots-pulse`.
- **Several orbs.** More than 3 animating orbs in a window → low-power rules; list rows never use T4.
- **Very fast agent.** `show_delay` applies to orbs placed as loaders; avatar orbs stay in `Idle` and
  just pulse once (Submitted) if the answer lands immediately.
- **Very slow agent.** After 10 min in one phase, energy target eases down 30 % and tier drops one
  step (proposed); hue drift continues.
- **Bursty streams.** Token-rate coupling is smoothed and capped at 2 pulses/s.
- **Context loss / GPU reset / sleep-wake.** Swap to T3 immediately, restore later (§5.3); phase is
  preserved because it lives in the clock, not the GPU.
- **HiDPI.** Render at device pixel ratio, capped at 2 for shaders; dots snap to half pixels at 16–20 px
  to stay crisp.
- **Screen recording / screenshots.** No special case; the Studio exports posters and loops from the
  same engine.
- **RTL.** No mirroring (an orb has no reading direction); search band sweep direction stays.
- **Forced colours / high contrast.** Orb becomes the T2 `conic` fallback in system colours with a
  solid outline.
- **Touch.** Not a target unless the host makes the avatar a button; then 44 px hit area.

## 8. Accessibility
- Decorative by default: `aria-hidden="true"` / not exposed in GPUI. When an orb is the only visual
  representation of the agent (avatar with no name), the host labels it ("Agent, thinking").
- `aria-busy="true"` goes on the content being produced, not on the orb.
- No live-region output from the orb; phase announcements come from the composer's single polite live
  region (assertive for waiting and failed) per [loaders.md](loaders.md) §8.
- **Flashing:** the Done flash is a single 200 ms brightening of the orb's own box, never repeated;
  streaming density pulses are capped at 2 per second; no full-area luminance flashes above 3 per
  second; glows ease, they never blink.
- **`prefers-reduced-motion`:** designed still poster per phase; no tumble, breathe, hue drift or
  pulses; phase changes crossfade 0–80 ms.
- **Contrast:** the orb's silhouette (rim or outer dots) ≥ 3:1 against the background in both themes;
  waiting/stalled/failed also change motion and the host's text, never colour alone.
- **Vestibular safety:** no zooming toward the viewer, no parallax with the page, no spin faster than
  1 turn per second even at energy 1.0 (proposed cap).
- Not focusable unless the host makes it a control.

## 9. Props sketch
```rust
pub struct ThinkingOrb {
    pub agent: AgentState,                 // LoaderPhase::derive(&agent, stall, tool) → energy target
    pub variant: OrbVariant,               // DottedBreathe | DottedRings | DottedSearch | Conic | Blob | PulseRings | Pro(OrbPro)
    pub size: OrbSize,                     // Px16..Px48 | Hero(u16) up to 256
    pub palette: OrbPalette,               // Accent (default) | Spectral | Token(..) ; per-phase overrides
    pub dark: bool,                        // light / dark tuning
    pub region: BusyRegion,                // usually Avatar(id) or Composer
    pub priority: BusyPriority,            // Lead by default
    pub tier: TierPolicy,                  // Auto | Max(RenderTier) | Exact(RenderTier)
    pub power: PowerPolicy,                // Auto (default) | Low | Full
    pub energy_curve: EnergyCurve,         // per-phase targets; defaults in §3
    pub token_rate: Option<f32>,           // tokens/s from the stream adapter (Streaming coupling)
    pub audio_level: Option<f32>,          // family K only
    pub interactive: OrbInteraction,       // None (default) | Tooltip | DragSpin (Pro)
    pub labels: LoaderLabels,
    pub motion: MotionLevel,
}

pub struct EnergyCurve {                  // proposed defaults
    pub idle: f32,        // 0.15
    pub submitted: f32,   // 0.35
    pub thinking: f32,    // 0.45
    pub working: f32,     // 0.70
    pub streaming: f32,   // 0.55 (+0.25 × normalised token rate)
    pub waiting: f32,     // 0.20
    pub stalled: f32,     // 0.10
}
```

## 10. Related components and references
**Related:** [loader-vector.md](loader-vector.md) (T2 fallbacks) · [loader-mascot.md](loader-mascot.md)
(`orb-eyes` hybrid; busy glow shares the palette) · status strip ([composer-status-strip.md](composer-status-strip.md)) ·
send control Pro `voice-orb` / `particle` ([composer-send-control.md](composer-send-control.md)) ·
`composer/voice-first` ([composer.md](composer.md)) · family K voice orbs · family L animated marks.

| Reference | What we take | Licence note |
|---|---|---|
| thinking-orbs / Libraries.dev [R068] [R100] (research notes 15, 16) | nine verb-named states, 20 / 64 px separate tuning, `speed` / `dark` / `paused` props, 2D canvas technique | MIT (packages) — **port with attribution** |
| gpui-thinking-orbs (licensing.md) | candidate GPUI port to evaluate first | MIT per licensing.md — port with attribution; repository details not yet reviewed |
| Orbkit [R067] | idle / thinking / speaking orb state API | Study only |
| AI Assist orb [R019] | orb speed tracks the state | Study only |
| MetalForge (research notes 02, 13) | 22 shader orbs and their motion characters; thinking-orbs editor (Breathe, Band, Rings, Twinkle, Vortex); live sliders | Paid — study only |
| Originkit Particle Gimbal (research note 16) | integer turn ratios, back-to-front painting, auto-fit, trackball drag with decay | Paid / unclear — study only |
| pontusab particle field [R001] | settle-and-disperse sand field (`cymatics`) | Study only |
| miu21590 wireframe orb [R008] | wireframe glass orb with beams | Study only |
| Kubai087 shockwave [R039] | layered shockwave recipe for `orb-shockwave` | Study only |
| SPY UI orbs [R136], Rare UI Fluid Orb (research note 17) | voice / orb state sets | SPY UI paid tiers study only; Rare UI licence unclear — do not copy |
| Paper Shaders [R140] | shader technique reference for T4 (mesh gradient, metaballs, dot orbit) | Apache-2.0 — port with attribution |
| ittybitty Edge Glow (research note 02) | offscreen pause + reduced-motion still frame pattern | MIT per note 02 — confirm before porting |
