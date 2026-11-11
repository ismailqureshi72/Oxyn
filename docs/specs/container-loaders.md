**Status:** draft v0.1 (2026-11-11).

# Spec: container & media loaders (family overview + window edge glow)

Overview of family D in [../component-taxonomy.md](../component-taxonomy.md) ("Container & media
loaders", target **90 variants**). It follows the 10-point standard in [README.md](README.md) at the depth
of [composer-send-control.md](composer-send-control.md). This file owns four things the part specs
share: the **wrapper model** (any container can host a loader layer without reflow), the **mapping from
`AgentState` and job progress** to what a container shows, the **busy-signal rule shared with family B**,
and the **performance budget**. It also holds the full spec of one part, **window edge glow** (§11),
and the family's **variant catalog** (§5), which adds up to 90.

Part specs:

| Part | Spec | What it owns |
|---|---|---|
| Border beam / shape trace / composer border light | [border-beam.md](border-beam.md) | light travelling a container's outline; determinate stroke fill |
| Media generation placeholder | [media-generation-placeholder.md](media-generation-placeholder.md) | image / video generation: noise → blur → sharpen, candidate grids, blocked state |
| Media analysis scan | [media-analysis-scan.md](media-analysis-scan.md) | scan line, region boxes, OCR highlight, page counter, fill-over-media |
| Skeletons | [skeletons.md](skeletons.md) | shape-matched placeholders, sweep styles, min display, swap-in |
| Window edge glow | §11 of this file | desktop / app window border while the agent acts |

## 1. Purpose
In an agent app, a lot of waiting happens **inside a box**: an image tile while it is generated, a PDF
while the agent reads it, a card while a tool fills it, a preview pane while an app builder rebuilds, the
composer while a run is live, the whole window while a computer-use agent drives the screen. The user
wants to know three things at a glance: **is something happening in this box, how far along is it, and
is it the agent's work or mine to do next.** A spinner in the middle of the box answers only the first,
covers the content, and makes the layout jump when it is swapped out.

Family D answers all three by **decorating the container itself**: its edge (beam, trace, glow), its
surface (sweep, scan, fill) or its content slot (noise placeholder, skeleton). Typical moments:
- coding agent: file cards while edits apply, preview frame while a dev server restarts;
- app builder: the whole preview while "generating this UI" (with family I skeletons);
- video / media agent: candidate grids, render queue tiles, the timeline clip being rendered [R013];
- research agent: PDFs and screenshots being read ("Reading page 3/12");
- browser / computer-use: the window or screen edge while the agent has control;
- every category: the composer border light while a run is live [R095].

Family rules (apply to every part):
1. **No reflow, ever.** A loader is a layer painted within the host's existing bounds. Starting,
   stopping or swapping a loader never changes the host's size or position (§2).
2. **One state source.** Loaders read `AgentState` (global) and a `JobSnapshot` (per container); they
   keep no private timer (shared motion clock, [../architecture.md](../architecture.md)).
3. **Determinate only when the data is real.** No fake percentages; indeterminate until the job
   reports a fraction or a step count (§3).
4. **One looping attention signal per region** (§4, shared with family B).
5. **Content beats decoration.** As soon as real content streams in, the decoration steps down.
6. **Every variant has a reduced-motion fallback and a text equivalent.**

## 2. State machine

### 2.1 Wrapper model (layers)
```
 ┌──────────────── host container (tile, card, composer, pane, window) ─────────────────┐
 │  Edge layer      beam / trace / conic border / edge glow     painted on the outline  │
 │  ┌──────────────────────────────────────────────────────────────────────────────┐    │
 │  │  Surface layer   sweep / scan line / fill-over / region boxes  over content  │    │
 │  │  ┌──────────────────────────────────────────────────────────────────────┐    │    │
 │  │  │  Content slot  real content │ skeleton │ generation placeholder     │    │    │
 │  │  └──────────────────────────────────────────────────────────────────────┘    │    │
 │  └──────────────────────────────────────────────────────────────────────────────┘    │
 │  Badge slot (corner)  "Reading 3/12" · "41%" · ⟲ · ✕   ← overlays, never pushes      │
 └──────────────────────────────────────────────────────────────────────────────────────┘
```
- The host passes its **shape**: rect, per-corner radii (or pill / circle / squircle / custom path) and
  a clip. The edge layer draws **inside** the outline (inset by half the stroke) so nothing paints
  outside the host's bounds unless the variant declares an `outset` glow (then it is clipped by the
  host's parent, and the host must reserve the outset via `overflow: visible` / no clip; proposed).
- The surface layer is clipped to the host's shape. The content slot keeps the host's final size from
  the first frame (aspect ratio, row heights; see skeletons.md and media-generation-placeholder.md).
- Hit testing: edge and surface layers are **pointer-transparent** except the badge slot actions.

### 2.2 Container loader lifecycle (all parts)
```
                job created / agent busy           show_delay passed (150 ms)
   Hidden ───────────────────────────▶ Pending ────────────────────────────▶ Active(Indeterminate)
     ▲          (nothing painted yet;      │ done before delay: skip straight      │ first fraction / steps
     │           avoids a flash)           │ to Hidden (no flash)                  ▼
     │                                     │                               Active(Determinate p)
     │                                     │                                       │
     │   ┌──────── user's turn (approval / question) ── AwaitingYou ◀──────────────┤
     │   │                                                    │ answered           │ no event for job_stall
     │   │                                                    ▼                    ▼
     │   │                                                 Active ◀── event ── Stalled
     │   ▼                                                                         │
     │  Settling(Success) ── 240–600 ms ──▶ Hidden          ◀── Done ─────────────┤
     │  Settling(Failure) ── holds until dismissed/retried   ◀── Error ────────────┤
     └──Settling(Cancelled) ── 160 ms fade ──▶ Hidden        ◀── Cancelled ────────┘
   Special: Blocked(reason) (media only) · Paused (offline / user paused) · Offscreen (clock paused)
```
`min_display` (300 ms, proposed): once `Active` has painted, it stays at least 300 ms before
`Settling(Success)` so the eye registers it; the success settle then runs normally.

## 3. States table (family level; each part refines it)

| State | Edge layer | Surface layer | Content slot | Primary | Secondary | Keyboard (host focused) |
|---|---|---|---|---|---|---|
| `Hidden` | host's normal border | none | content | host's own | — | host's own |
| `Pending` (≤ 150 ms) | unchanged | none | unchanged / reserved box | — | — | — |
| `Active(Indeterminate)` | beam laps / glow breathes | sweep or scan loops | skeleton / noise / dimmed content | cancel (if job is cancellable) | open details | Esc cancel (when focused and cancellable) |
| `Active(Determinate p)` | stroke fills to p / beam anchored | fill-over to p | progressive reveal | cancel | details | Esc cancel |
| `Active(Steps k/n)` | as determinate with n ticks | — | "Reading 3/12" badge | cancel | details | Esc cancel |
| `AwaitingYou` | loop stops; steady attention-tone edge (no motion) | frozen | content as is + "Needs you" badge | open the pending card | — | Enter opens card |
| `Stalled` | loop slows to 0.5×, tone ramps toward warning over 2 s | slowed | unchanged | retry (if job supports) | cancel | Enter retry, Esc cancel |
| `Paused` (offline / user) | frozen at current position, 60 % opacity | frozen | badge "Paused · offline" | resume | cancel | Enter resume |
| `Settling(Success)` | one completion pass then fade | sweep fades | real content cross-fades in | — | — | — |
| `Settling(Failure)` | danger segment held where it stopped | none | content or placeholder + "Failed · Retry" | retry | dismiss, details | Enter retry, Delete dismiss |
| `Settling(Cancelled)` | fades 160 ms | fades | previous content or empty placeholder | — | — | — |
| `Blocked(reason)` | none | none | abstract blocked placeholder (media) | edit request | learn more | Enter edit |
| `Offscreen` | not painted; clock not ticking for this host | — | — | — | — | — |

## 4. Shared rules

### 4.1 Mapping from `AgentState` (global containers: composer border light, window edge glow, preview pane)

| `AgentState` | Global container loader | Notes |
|---|---|---|
| `Idle`, `Composing` | `Hidden` | |
| `Submitted` | no loop; a single 400 ms "submitted" rim pulse (opacity 0 → 0.6 → 0) | architecture.md: "submitted" pulse, not a spinner, < 500 ms |
| `Thinking { verb }` | `Active(Indeterminate)` at base intensity | |
| `Streaming` | `Active(Indeterminate)` at **50 % intensity** (proposed) | the text is already moving; rule 5 |
| `ToolRunning { tool, .. }` | `Active` at base intensity; per-tool tone optional | the tool's own tile gets a job loader |
| `AwaitingApproval { .. }` | `AwaitingYou` (static, attention tone) | motion stops so "waiting on you" never looks like "working" |
| `Queued { count }` | no change (it is a modifier of a busy run) | |
| `Steering` | one accelerated pass (beam 2× speed for one lap, or glow swell 300 ms), then back | |
| `Done { elapsed }` | `Settling(Success)` | |
| `Error { kind }` | `Settling(Failure)` (global containers fade after 1.2 s; the error is shown by the status strip) | |
| `Cancelled` | `Settling(Cancelled)` | |
| `Stalled { since }` | `Stalled` | derived by `oxyn-core`; thresholds from [composer-status-strip.md](composer-status-strip.md) (5 s soft, 45 s hard, 120 s tool) |

### 4.2 Mapping from job progress (per-container loaders: tiles, cards, media)

```rust
pub enum JobProgress {
    Indeterminate,                                        // nothing measurable yet
    Determinate { fraction: f32 },                        // 0.0..=1.0, from the job
    Steps { done: u32, total: u32, label: Option<SharedString> }, // "page 3 of 12", "frame 40/96"
}
```
Rules (proposed):
1. **Indeterminate until real.** A job becomes determinate only when the source reports a fraction, a
   byte count with known total, or steps with a known total. Never interpolate from elapsed time.
2. **Monotonic display.** The displayed fraction never goes backwards; a lower report is held (logged in
   debug builds). A job restart (retry) resets to 0 with a visible reset (stroke fades out, redraws).
3. **Smoothing.** Displayed fraction eases toward the reported one with a critically damped follow
   (`spring.snap`), capped so it never runs more than 250 ms behind.
4. **Indeterminate → determinate morph.** The moving element anchors and becomes the progress mark
   (beam tail anchors at the start point and its head becomes the fill end; sweep becomes a fill edge).
   Determinate → indeterminate happens only via `Stalled` or `Paused`.
5. **Steps beat fractions for text.** When both exist, the badge shows steps ("Reading 3/12") and the
   edge shows the fraction.
6. **Unknown total with a counter** ("214 files indexed") stays indeterminate with a live count.
7. **ETA** is shown only when the job supplies one; we do not compute ETAs (proposed).

### 4.3 Busy-signal rule (shared with family B)
`oxyn-core` derives one `BusySignal` from `AgentState` (the "one `waiting` flag drives composer glow,
thinking row and controls together" idea seen in BoardUI [R095]; research note 16 idea 1):

```rust
pub struct BusySignal {
    pub level: BusyLevel,          // Off | Submitted | Working | Streaming | AwaitingYou | Stalled
    pub tone: Tone,                // Accent | Attention | Warning | Danger | Tool(ToolKind)
    pub intensity: f32,            // 0..1, eased; Streaming = 0.5 (proposed)
    pub since: Instant,
}
```
1. **Global loaders** (family B thinking indicators, status tab, composer border light, window edge
   glow) all subscribe to the same `BusySignal` and the same motion clock, so they start, slow, stall
   and stop together; phases stay locked (one clock, as Warp does with one `AnimationClock`, research
   note 08).
2. **One looping attention signal per region.** A region is the composer block, a transcript message,
   a tile, or the window frame. If a family B indicator is already looping in the composer block (the
   status tab dot), the composer border light runs in `subtle` (no bloom, 50 % intensity) unless the app
   opts into `expressive`. Window edge glow counts as its own region but is off by default except on the
   `Desktop` surface in computer-use mode.
3. **Per-job loaders don't read `BusySignal`** for their motion; they read their `JobSnapshot`. They
   *do* read it for two things: they freeze when the global level is `AwaitingYou` and the job is
   blocked on that approval, and they inherit `Stalled` tone only if their own job has stalled.
4. **Fade, don't switch.** Turning a loader on or off is a fade (`dur.base` in, `dur.fast` out);
   "lighting up rather than switching" (BoardUI composer loader description, research note 16).
5. **Fast agents.** `show_delay` 150 ms (proposed): a run or job that finishes inside it shows nothing.
   Family B's submitted pulse is the only feedback in that window.

### 4.4 Performance budget (proposed)
Many tiles can load at once (a 4 × 4 candidate grid, a 200-file index, a dashboard of 30 cards).

| Budget item | Value (proposed) |
|---|---|
| Frame cost, all family D layers in a window | ≤ 1.5 ms CPU + ≤ 1.5 ms GPU per frame on a 2020-class laptop iGPU |
| Animating hosts at once | 24 max at full rate; beyond that, extra hosts render the **static** state of their variant, nearest-to-viewport-centre first get motion |
| Frame rate | beams, scans, reveals 60 fps; ambient glows, skeleton sweeps, noise fields 30 fps; terminal surface 10–12 fps |
| Offscreen | hosts outside the viewport (plus 1 viewport of margin) do not tick; on return they jump to the clock's current phase (no catch-up animation) |
| Unfocused window | shared clock slows to 0.25× (architecture.md: clock slowed when unfocused); edge glow drops to 15 fps |
| Shared sweeps | skeleton and shimmer sweeps across many blocks use **one sweep pinned to the viewport** (one gradient, many masks), as affectUI's Shimmer Loader does [research note 11] |
| Noise fields | one noise texture per window, sampled with per-tile offsets; never one generator per tile |
| Memory | ≤ 4 MB of textures for the whole family per window (noise atlas + glow ramp) |

**Render paths** (same props on every path; research note 02 "same props across tiers"):

| Path | Where | How (proposed) |
|---|---|---|
| **GPU (GPUI, primary)** | desktop | edge layers as stroked paths / quads with gradient fills through GPUI's element paint pass; the arc-length parameterisation is computed once per shape change on the CPU and cached. Shader-tier variants (bloom, noise, glimm-style sweeps) need a render-to-texture + shader pass (research note 15: wgpu), so they are Pro and opt-in (`quality = Shader`) |
| **CSS (Dioxus web, default)** | web | `@property --angle` + `conic-gradient` with `mask-composite: exclude` for borders; `offset-path` for beams; `background-position` sweeps; all `transform`/`opacity`/`background-position` only (no layout-affecting properties) (techniques in research note 02) |
| **Canvas / WebGL (Dioxus web, opt-in)** | web | noise placeholders, edge glow haze; pauses offscreen; still frame under reduced motion (ittybitty Edge Glow pattern, research note 02) |
| **CPU fallback** | GPUI without shader support, terminal | no bloom, no noise; beam = 2-segment gradient stroke; terminal = glyph frames (`░▒▓█` / braille) on the container's border cells |

### 4.5 Micro-interactions (family level; parts refine them)
- **Enter.** After `show_delay`, the active layer fades in over `dur.base`; edge layers also grow from
  0 to full stroke width so the outline "lights up" rather than pops.
- **Hover on a busy host.** The badge slot reveals ✕ cancel (and the step label if hidden); the loop
  never pauses on hover (pausing reads as "stalled").
- **Press on ✕.** 0.94 scale for 80 ms; the loader goes to `Settling(Cancelled)` immediately, the job
  cancel is sent in parallel (optimistic, rolled back with an error badge if the job refuses).
- **Morphs.** Indeterminate → determinate anchors the moving element (§4.2 rule 4); success runs one
  completion pass, then real content cross-fades in under the fading layer; failure freezes the layer
  where it stopped and tints it danger so the user sees how far it got.
- **Number changes.** Step and % badges use tabular digits and roll (family C odometer), never re-flow
  the badge width (badge reserves the widest string, e.g. "12/12").
- **Grid choreography.** Settling tiles in a grid stagger 40 ms in reading order (cap 6), so a batch
  that finishes together lands as a ripple, not a flash.

## 5. Variants (family catalog, 90)

Naming per [../variants.md](../variants.md). Counts per part (proposed):

| Part | Free | Pro | Total |
|---|---|---|---|
| Container wrapper presets (this file) | 1 | 3 | 4 |
| Window edge glow (§11) | 3 | 9 | 12 |
| Border beam / shape trace (border-beam.md) | 4 | 20 | 24 |
| Media generation placeholder | 3 | 14 | 17 |
| Media analysis scan (incl. fill-over-media) | 3 | 14 | 17 |
| Skeletons (container sweeps; block coverage shared with family I) | 4 | 12 | 16 |
| **Family D** | **18** | **72** | **90** |

### Wrapper presets (this file)
| Name | Tier | Idea |
|---|---|---|
| `container-loader/auto` | Free | Picks edge vs surface vs content layer from the host kind (tile → beam, media → placeholder/scan, card → skeleton) and job progress |
| `container-loader/grid-sync` | Pro | Many tiles share one phase: a single beam "hands off" from tile to tile across a grid in reading order |
| `container-loader/focus-follow` | Pro | Only the tile the agent is currently touching animates; finished tiles settle, queued tiles show a static dashed edge |
| `container-loader/writing-here` | Pro | A soft glow sweeps over the region being changed and content settles after it ("agent is writing here", [R034]); shader tier |

Window edge glow variants are in §11.5. Part variants are listed in each part spec.

## 6. Edge cases (family level)
- **Tiny hosts (16–20 px chips, icons).** Edge layers switch to a 1 px stroke with no bloom; beam length
  min 8 px; noise and scans are replaced by a 2-frame opacity pulse; text badges move to a tooltip.
- **Huge hosts (full-screen preview, 4K window).** Beam length is capped (240 px) and lap time capped
  (4 s) so it never crawls; glow haze width caps at 48 px.
- **Dense IDE panels.** `motion = Subtle` by default: no bloom, no surface sweeps on rows < 28 px.
- **Long jobs (hours).** Indeterminate loops never accelerate or "give up"; after `job_stall` they go
  `Stalled`. Elapsed is the status strip's job, not the tile's.
- **Very fast jobs.** `show_delay` hides jobs < 150 ms; jobs between 150 ms and 450 ms get
  `min_display` 300 ms so they never flash for one frame.
- **Many concurrent jobs.** §4.4 cap; the rest render static. A "+18 working" count belongs to the
  jobs tray (family F), not to each tile.
- **Host resizes mid-animation.** Arc-length cache is rebuilt; the beam keeps its normalised position
  `s / P` so it does not jump.
- **Host scrolls or is virtualised.** A recycled host must take the new job's state on the same frame;
  loaders are keyed by job id, not by host instance.
- **RTL.** Beams and determinate strokes run counter-clockwise from the top-trailing (visually
  top-right in LTR becomes top-left) start point; sweeps run right → left; badges mirror corners.
- **Touch.** No hover-dependent information; the badge slot is ≥ 44 px when it holds an action.
- **Slow networks / offline.** Jobs that depend on the network go `Paused` with the reason; the edge
  freezes rather than looping on, so motion never lies about progress.
- **Transparent / image hosts.** Edge strokes get a 1 px contrast halo (inner, 30 % of the surface's
  opposite luminance) so beams read on busy media (proposed).
- **Terminal surface.** Glyph equivalents only: border cells cycle `─━` / `░▒▓█`, scans become a moving
  inverted row, 10–12 fps.

## 7. Accessibility (family level)
- **Busy state.** The host gets `aria-busy="true"` (web) / the GPUI accessibility "busy" property while
  `Pending`, `Active`, `Stalled`; removed on `Settling`.
- **Progress semantics.** A determinate or stepped job exposes `role="progressbar"` with
  `aria-valuemin=0`, `aria-valuemax=100` (or `total`), `aria-valuenow`, and `aria-valuetext`
  ("Reading page 3 of 12", "Generating 2 of 4 images"). Indeterminate jobs expose `progressbar`
  **without** `aria-valuenow`. The progressbar is labelled by the host ("report.pdf").
- **Decorative layers are hidden.** Beams, glows, sweeps, noise are `aria-hidden`; skeleton shapes
  are `aria-hidden` (affectUI does the same, research note 11).
- **Announcements (polite, through the composer/app's single live region):** start only for jobs the
  user started explicitly; milestones at most every 10 s **or** at 25 / 50 / 75 % (whichever is rarer);
  completion ("4 images ready"); failure is **assertive** ("Image generation failed. Retry available").
  Global busy changes are announced by family B, never by edge glow or border light.
- **Flashing limits.** No layer may produce more than **3 flashes in any 1 s** (WCAG 2.3.1); completion
  flashes are single and ≤ 0.25 luminance change (proposed); no saturated red flashing at all; edge
  glows never strobe; looping luminance changes keep a period ≥ 1 s.
- **Motion.** Reduced motion (global token) stops every loop; state is carried by a static edge tone,
  a static fill amount and the text badge. Apps can expose "Pause animations" which maps to the same
  token.
- **Contrast.** Static edge tones used as state (attention, warning, danger) meet 3:1 non-text contrast
  against the surface; beams are decorative and exempt, but their static fallback is not.
- **Focus.** Loaders never take focus. The badge-slot actions (✕ cancel, ⟲ retry) are in the host's tab
  order after its primary content; focus is never stolen on completion.

## 8. Props sketch (shared)
```rust
pub struct LoaderLayer {
    pub agent: AgentState,                 // global state (architecture.md)
    pub busy: BusySignal,                  // derived by oxyn-core from `agent`
    pub job: Option<JobSnapshot>,          // per-container job; None = follow `busy`
    pub shape: ContainerShape,             // outline the edge layer follows
    pub layers: LayerSet,                  // { edge: Option<EdgeVariant>, surface: Option<SurfaceVariant>, content: Option<ContentVariant> }
    pub motion: MotionLevel,               // None | Subtle | Expressive
    pub quality: RenderQuality,            // Auto | Css | Gpu | Shader | Cpu | Glyph
    pub show_delay: Duration,              // 150 ms (proposed)
    pub min_display: Duration,             // 300 ms (proposed)
    pub labels: LoaderLabels,              // i18n + aria-valuetext templates
}

pub struct JobSnapshot {
    pub id: JobId,
    pub phase: JobPhase,                   // Pending | Queued{position} | Running | Paused(reason) | AwaitingYou
                                           // | Succeeded | Failed(kind) | Cancelled | Blocked(reason)
    pub progress: JobProgress,             // §4.2
    pub started_at: Instant,
    pub last_event_at: Instant,            // oxyn-core derives job Stalled from this
    pub eta: Option<Duration>,             // only if the source supplies it
    pub cancellable: bool,
    pub retryable: bool,
}

pub enum ContainerShape {
    RoundedRect { bounds: Bounds<Pixels>, radii: Corners<Pixels> },
    Pill { bounds: Bounds<Pixels> },
    Circle { center: Point<Pixels>, radius: Pixels },
    Squircle { bounds: Bounds<Pixels>, smoothing: f32 },
    Path(PathSpec),                        // custom outline (triangle, hex, logo)
    Window,                                // the app window frame (§11)
}
```

## 9. Timing (family tokens)
Family D reuses the composer motion tokens ([composer.md](composer.md) §4: `dur.instant`, `dur.fast`,
`dur.base`, `dur.travel`, `ease.out`, `ease.in`, `ease.in-out`, `ease.firm`, `spring.snap`,
`spring.soft`, `stagger`) and adds these loop tokens (all **proposed**):

| Token | Value | Used for |
|---|---|---|
| `loop.beam` | lap = clamp(perimeter ÷ 400 px/s, 1.6 s, 4.0 s), linear in arc length | border beam, composer border light |
| `loop.sweep` | 2.0 s per pass, linear, 105° band | skeleton shimmer (2 s linear at 105° matches the Claude desktop skeleton sweep in research note 08) |
| `loop.pulse` | 2.0 s, `ease.in-out`, opacity 0.45 ↔ 1 | skeleton pulse |
| `loop.scan` | 1.2 s top → bottom, `ease.in-out` | analysis scan line (same value as composer-attachments.md) |
| `loop.glow` | 3.2 s breathe, `ease.in-out`, intensity 0.7 ↔ 1 | edge glow, conic borders |
| `loop.noise` | 30 fps noise evolution, seed drift 0.2 cells/s | generation placeholder |
| `dur.reveal` | 600 ms blur 24 px → 0 + noise dissolve | media reveal |
| `dur.settle` | 240 ms | success fade-out of edge layers |
| `show_delay` / `min_display` | 150 ms / 300 ms | every loader |

| Transition | Duration | Easing | Notes |
|---|---|---|---|
| Hidden → Active (fade in) | `dur.base` 180 ms | `ease.out` | after `show_delay` |
| Indeterminate → determinate morph | 220 ms | `spring.snap` | moving element anchors |
| Determinate follow | continuous, ≤ 250 ms lag | `spring.snap` | |
| Active → AwaitingYou | 200 ms motion decelerates to rest | `ease.firm` | then static |
| Active → Stalled | speed 1× → 0.5× over 2 s; tone ramp 2 s | linear (OKLab) | matches status strip ramp |
| Success settle | completion pass 400 ms + fade `dur.settle` | `ease.out` | single pass, no repeat |
| Failure hold | danger tone in 160 ms; one 4 px shake 300 ms (tiles only) | `ease.out` / linear | no shake on windows |
| Cancel | 160 ms fade | `ease.in` | |
| **Reduced motion** | all loops stop; transitions become 0–80 ms opacity crossfades; determinate fill jumps to the value with an 80 ms fade; state carried by static tone + badge text | — | per-variant static frame defined in each part spec |

## 10. Related components and references
Related: family B loaders and the busy signal · family C status text · composer status strip (stall
thresholds) · composer attachments (tile ring) · family F jobs tray, tool cards, fan-out grid · family I
generative UI skeletons · family K voice orb · family G take-over banner.

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| BoardUI Composer Loader, one `waiting` flag | [R095], research note 16 | busy signal fan-out; fade "lighting up rather than switching" | Free tier MIT; check the item is in the MIT tier before porting, else study only |
| Libraries.dev `border-beam`, `img-fx` | [R100], research note 16 | beam + image-gen loader as packages | Packages MIT: may port with attribution |
| Magic UI Border Beam / Shine Border, UploadKit UploadBeam | research note 02 (no source ID yet) | beam props; beam bound to an upload lifecycle | Licence not recorded in our notes: study only until checked |
| ittybitty Edge Glow; CodeShack / aicssanimations Siri edge glow; edge-aura; intelligence-glow; Wensity glow | research note 02 (no IDs) | rim + haze + wake parts; pause offscreen; still frame under reduced motion | ittybitty MIT per note 02 (re-check); others not recorded or paid: study only |
| affectUI Shimmer Loader | research note 11 | one sweep shared across blocks; `aria-busy` on container, skeletons `aria-hidden` | Paid / unclear: study only |
| Warp one `AnimationClock`; OpenCode tab pulse | research note 08 | phase-locked loops | Warp AGPL: study only; OpenCode MIT: may port |
| glimm "agent is writing here" sweep | [R034] | `writing-here` | Study only |
| twelvelabs scan highway | [R009] | video scan (media-analysis-scan.md) | Study only |
| seanaiux job pill and de-noise reveal | [R013] | generation placeholder | Study only |
| Paper Shaders | [R140] | noise / dither fields for the shader tier | Apache-2.0: may port with attribution |
| MetalForge shader loaders | research note 13 | quality bar for shader-tier loops | Paid: study only |

## 11. Window edge glow (part spec)

### 11.1 Purpose
On the `Desktop` surface, when the agent is **acting on the user's machine or in the app window**
(computer-use, browser automation, an app builder rewriting the open document, a voice agent
listening), the user needs a peripheral, unmissable but quiet signal that "the agent has control
right now", visible even when they are looking at another part of the screen. The edge glow is that
signal: a thin rim of light around the window frame with a soft haze inward. It also marks hand-off:
when the user takes over, the glow dims; when the agent needs them, it turns attention-tone and stops
moving. (Seen as "Siri edge glow" / "Apple Intelligence glow" patterns, research note 02.)

### 11.2 State machine
```
            agent acquires control (computer-use / voice listening / app-wide edit)
   Off ─────────────────────────────────────────────▶ Arming ── 320 ms ──▶ Acting
    ▲                                                                       │  ▲
    │                                    user moves mouse / types           │  │ user hands back
    │                                    (take-over detected)               ▼  │
    │                                                              UserHasControl
    │      approval / question                                                 
    │   Acting ───────────────▶ AwaitingYou ── answered ──▶ Acting             
    │   Acting ── no event for stall_soft ──▶ Stalled ── event ──▶ Acting      
    │   Voice: Listening ⇄ Acting (speaking / thinking map to Acting)          
    │◀── Done: completion wake 600 ms ──────────────────────────────── Acting  
    │◀── Error: danger rim 1.2 s fade ─────────────────────────────── Acting  
    └◀── Cancelled / window closed: 160 ms fade ───────────────────────────────
  Overlays: Unfocused (window not key: intensity 40 %) · Fullscreen video (suppressed) · ReducedMotion (static rim)
```

### 11.3 States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Off` | nothing | — | — | — |
| `Arming` | rim draws in from the top centre around both sides, meeting at the bottom (320 ms) | — | — | — |
| `Acting` | 2 px rim (accent gradient, 3–4 stops) + 24–48 px inward haze at ≤ 0.35 alpha; slow drift of the gradient along the frame; optional "wake" brightening near the window edge closest to the agent's cursor | stop (owned by the take-over banner, family G) | — | app's stop shortcut (Esc×2 in computer-use, proposed) |
| `Listening` (voice) | rim swells with audio level (0.6–1.0 intensity), haze narrower (16 px) | — | — | push-to-talk key |
| `UserHasControl` | rim at 30 % intensity, no drift; banner "You have control · Hand back" | hand back | stop agent | ⌘/Ctrl+Enter hand back (proposed) |
| `AwaitingYou` | drift stops; steady attention-tone rim, no haze motion | open the pending card | — | Enter opens card |
| `Stalled` | drift slows to 0.5×, tone ramps toward warning (2 s) | — | stop | — |
| Done (transient) | one "wake" pass around the frame (600 ms) then fade 240 ms | — | — | — |
| Error (transient) | rim turns danger, haze off, fades over 1.2 s; no flashing | — | — | — |
| `Unfocused` overlay | 40 % intensity, 15 fps | — | — | — |
| Fullscreen video / screen share of this window (overlay) | suppressed unless `force` | — | — | — |

### 11.4 Micro-interactions
- **Arming.** The rim draws from the top centre outward along both sides and meets at the bottom
  centre, so the window "is taken" in one gesture; haze fades in behind it.
- **Drift.** Gradient stops travel along the frame at `loop.glow` pace; at 1 lap per ~8 s (proposed),
  slow enough to read as ambient, never as a progress beam.
- **Wake.** In computer-use, the haze brightens on the edge nearest the agent's virtual cursor (rim +
  haze + wake parts as in ittybitty's Edge Glow, research note 02), so the user's eye is led to where the
  agent is acting.
- **Take-over.** The first user input event during `Acting` dims the glow (200 ms) and slides in the
  take-over banner (family G); handing back re-arms with a short 200 ms swell.
- **Approval.** Motion decelerates to rest (200 ms `ease.firm`) and the tone cross-fades to attention.
- **Done.** A single wake pass travels once around the frame; no confetti, no repeat.
- **Error.** Tone shifts to danger and fades; the window never shakes.

### 11.5 Variants (12: 3 free + 9 Pro)
| Name | Tier | Idea |
|---|---|---|
| `edge-glow/rim` | Free | 2 px gradient rim, no haze; CSS / quads only (cheapest) |
| `edge-glow/haze` | Free | rim + inward haze, slow drift (the default) |
| `edge-glow/mono` | Free | single brand colour, no gradient; for dense or professional apps |
| `edge-glow/wake` | Pro | haze brightens near the agent's cursor (computer-use) |
| `edge-glow/listening-swell` | Pro | rim thickness and haze follow voice level |
| `edge-glow/aurora` | Pro | multi-stop aurora haze, shader tier |
| `edge-glow/ember` | Pro | warm low-intensity glow for long background runs |
| `edge-glow/notch` | Pro | glow concentrates at the top centre around a notch / island status (desktop) |
| `edge-glow/corners` | Pro | only the four corners glow (bracket marks), for minimal distraction |
| `edge-glow/screen` | Pro | whole-screen overlay for OS-level computer-use (needs a transparent click-through window; platform support not established) |
| `edge-glow/dither` | Pro | dithered rim and haze (pixel look, matches `pixel` loaders) |
| `edge-glow/terminal` | Pro | TUI frame: border characters cycle intensity in the terminal's own box-drawing border |

### 11.6 Timing
| Transition | Duration | Easing | Notes |
|---|---|---|---|
| Arming draw-in | 320 ms | `ease.out` | haze fades in 240 ms, 80 ms delay |
| Drift | ~8 s per lap (proposed) | linear | 30 fps; 15 fps unfocused |
| Listening swell | follows level, 60 ms attack / 180 ms release | — | |
| Take-over dim | 200 ms | `ease.out` | |
| Approval rest | 200 ms | `ease.firm` | |
| Stall ramp | 2 s | linear (OKLab) | |
| Done wake pass + fade | 600 ms + 240 ms | `ease.in-out` / `ease.out` | single |
| Error fade | 1.2 s | `ease.out` | |
| Cancel | 160 ms | `ease.in` | |
| **Reduced motion** | static rim at 70 % intensity, no haze motion, no drift, no wake; state by tone; transitions 0–80 ms opacity | — | ittybitty-style still frame |

### 11.7 Edge cases
- **Multiple windows.** Only windows where the agent acts glow; a second agent window uses the same
  glow at 60 % so the active one is clear (proposed).
- **Multiple agents in one window.** One glow; tone = the agent that currently holds control (family F
  "who's acting" names it).
- **Fullscreen video / presentations / screen sharing.** Suppressed by default (no glow on recordings),
  `force` overrides.
- **Tiny windows (< 320 px).** Haze capped at 12 px; rim 1 px.
- **Rounded window corners.** Rim follows the platform window radius (from the host); with unknown
  radius, 10 px (proposed).
- **Transparent / vibrancy windows.** Haze alpha capped at 0.25 to avoid tinting content.
- **Long runs.** After 10 min in `Acting` without user interaction, intensity eases to 70 % (proposed)
  to avoid fatigue; any state change restores it.
- **Web (Dioxus).** Glow is drawn inside the page viewport (fixed overlay, pointer-events none).

### 11.8 Accessibility
- Decorative: the glow is `aria-hidden`. The **take-over banner** and the status strip carry the
  text: "Agent has control of this window", "You have control", "Agent needs your approval".
- The control change is announced politely once; approval is assertive (owned by family G).
- Never more than 3 flashes per second; drift luminance change per second ≤ 10 % of range (proposed).
- Static tones meet 3:1 against the window background in both themes.
- Reduced motion: §11.6.

### 11.9 Props
```rust
pub struct WindowEdgeGlow {
    pub agent: AgentState,                 // Acting = ToolRunning / Streaming / Thinking while `control` is Agent
    pub busy: BusySignal,
    pub control: ControlHolder,            // Agent | User | None
    pub cursor_hint: Option<Point<Pixels>>,// agent's virtual cursor for `wake`
    pub voice_level: Option<f32>,          // 0..1 for `listening-swell`
    pub variant: EdgeGlowVariant,
    pub rim_width: Pixels,                 // 2 px (proposed)
    pub haze_width: Pixels,                // 24–48 px (proposed)
    pub max_alpha: f32,                    // 0.35 (proposed)
    pub suppress_when: SuppressPolicy,     // Fullscreen | ScreenShare | Never
    pub motion: MotionLevel,
    pub quality: RenderQuality,
}
```

### 11.10 References
ittybitty Edge Glow (rim / haze / wake, presets Listening / Calm / Aurora / Ember / Mono, pauses
offscreen, still frame under reduced motion; MIT per research note 02, re-check before porting) ·
CodeShack and aicssanimations Siri edge glow (conic ring + mask + blurred haze; licence not recorded:
study only) · edge-aura, intelligence-glow (licence not recorded: study only) · Svelte Motion AI glow
border (feTurbulence displacement; study only) · Wensity Apple-Intelligence glow ([R119]; Pro: study
only) · OpenCode tab pulse edge flash 800 ms (research note 08; MIT).

## Gaps
- Magic UI, UploadKit, ittybitty and the other glow sources in research note 02 have no row in
  `sources.csv` yet; licences need checking before any port.
- GPUI support for a click-through transparent overlay window (for `edge-glow/screen`) is not
  established.
- All numbers marked proposed need a test on real hardware (frame budget, 24-host cap).
