**Status:** draft v0.1 (2026-11-11).

# Spec: border beam, shape trace and composer border light

Part of family D ([container-loaders.md](container-loaders.md)); follows the 10-point standard in
[README.md](README.md) at the depth of [composer-send-control.md](composer-send-control.md). Target
**4 free + 20 Pro = 24 variants**. One primitive, `EdgeTrace`, draws light along any container outline;
indeterminate (a beam that laps) and determinate (a stroke that fills to the job's fraction) are two
modes of the same primitive, so a tile can morph from one to the other without swapping components.

Consumers already named in other specs use this primitive: `send-control` Pro `border-beam`,
`status-strip` `border-beam`, `mode-switch/border-beam` (one pass on switch), `effort-control/border-beam`
(speed follows effort), and the composer busy tint in [composer-input-shell.md](composer-input-shell.md).

## 1. Purpose
Agents work on things that live in boxes: a file card being edited, an attachment uploading, a tool
card running, a candidate image rendering, the composer while the run is live. A border beam says
"this box is being worked on" **without covering the content** and **without moving the layout**,
which a centred spinner cannot do. A shape trace says "and it is this far along" by filling the outline
itself (an accent ring drawing around a tile's own outline is the pattern in composer-attachments.md
and in research note 02's Sundae UI / UploadKit examples).

The **composer border light** is the same primitive tuned for one place: a constant-speed band of light
around the composer pill while the agent works ("an iridescent light band travelling the pill's rim at
constant speed … with a soft bloom bleeding inward", BoardUI's description, research note 16). It is the
composer's region signal and obeys the one-loop-per-region rule in container-loaders.md §4.3.

## 2. State machine

```
                    job / busy starts            show_delay (150 ms)
   Hidden ──────────────────────────▶ Pending ─────────────────────▶ Arming ── 180 ms ──▶ Running (beam laps)
     ▲                                  │ ends < 150 ms → Hidden                          │      │
     │                                  │                                 fraction known  │      │ steps known
     │                                  │                                                 ▼      ▼
     │                                  │                                     Filling(p)   Stepping(k/n)
     │                                  │                                          │  ▲  ▲      │
     │       ┌───────────── approval / question ─────────────────────────────────┤  │  └──────┘
     │       ▼                                                                    │  │
     │   AwaitingYou (static, attention tone) ── answered ──▶ previous mode       │  │ event
     │   Stalled (0.5× speed, warning ramp) ◀── no event for stall threshold ─────┘  │
     │   Paused (frozen, 60 % opacity) ◀── offline / user pause ── resume ───────────┘
     │
     │◀─ fade 240 ms ── Completing (closing pass 400 ms) ◀── Done / p reaches 1.0
     │◀─ dismiss / retry ── Failed (done part kept, rest danger-dashed) ◀── Error
     └◀─ fade 160 ms ── Cancelled ◀── Cancelled
   Composer border light only: Submitted → one rim pulse (400 ms), no laps; Streaming → 50 % intensity
```

## 3. States table

| State | Shows | Primary | Secondary | Keyboard (host focused) |
|---|---|---|---|---|
| `Hidden` | host's normal border | host's | — | host's |
| `Pending` | nothing yet | — | — | — |
| `Arming` | stroke grows from 0 to full width at the start point; beam head appears | — | — | — |
| `Running` | beam of length L laps the outline at constant arc-length speed; optional twin beam offset by P/2 | cancel ✕ (badge, if cancellable) | details | Esc cancel |
| `Filling(p)` | 1 px track at 12 % opacity around the whole outline; accent stroke from the start point to p·P; small head glow; optional % in the badge | cancel | details | Esc cancel |
| `Stepping(k/n)` | n tick gaps along the outline (n ≤ 24); k segments lit; badge "3/12" | cancel | details | Esc cancel |
| `AwaitingYou` | beam stops where it is and widens into a steady attention-tone outline; no motion | open pending card | — | Enter |
| `Stalled` | speed eases to 0.5×; tone ramps toward warning over 2 s | retry (if supported) | cancel | Enter retry · Esc cancel |
| `Paused` | frozen at last position, 60 % opacity, badge "Paused · offline" | resume | cancel | Enter resume |
| `Completing` | stroke closes the loop; one bright pass travels the full outline (400 ms); success tone for 600 ms (tiles) then fade | — | — | — |
| `Failed` | lit part stays, turns danger; unfinished part becomes a 1 px danger dashed line; ⟲ in badge | retry | dismiss | Enter retry · Delete dismiss |
| `Cancelled` | fades out (160 ms) from wherever it was | — | — | — |
| Composer light `Submitted` | one rim pulse 0 → 0.6 → 0 opacity, 400 ms, no beam | — | — | — |
| Composer light `Streaming` | beam at 50 % intensity, no bloom | — | — | — |

## 4. Micro-interactions

### Geometry (all modes)
- **Path.** The outline is converted once (per shape change) into an arc-length table: straight edges
  plus one arc per corner with its own radius (radii clamped to ≤ min(w, h) ÷ 2). Perimeter of a
  uniform rounded rect is P = 2(w + h) − 8r + 2πr. Pill, circle, squircle, triangle, hexagon and custom
  paths use the same table (sampled at ≤ 2 px spacing).
- **Constant speed through corners.** The beam advances by arc length, so it does not speed up on
  straight edges or bunch up in corners (the bug when a beam is driven by angle around the centre, as
  conic-gradient borders are; that look is a separate variant family, `conic-*`).
- **Inset.** The stroke centre sits `stroke ÷ 2` inside the outline, so nothing paints outside the
  host's bounds; pixel-snapped to half pixels on 1× displays.
- **Start point.** Top centre by default (`StartPoint::TopCenter`), clockwise in LTR, counter-clockwise
  in RTL. Determinate fills always start here so users learn where "0" is. Options: top-leading corner,
  any normalised `s`.
- **Beam shape.** Length L = clamp(0.18 · P, 48 px, 240 px) (proposed). Gradient along the beam: tail
  alpha 0 → head alpha 1 in the accent ramp (2–4 stops); head gets a 1 px brighter core and a soft dot
  glow of radius 2 × stroke. Stroke 1.5 px default, 1 px dense, 2 px expressive.
- **Speed.** Lap time = clamp(P ÷ 400 px/s, 1.6 s, 4.0 s) (`loop.beam`, proposed), so a 56 px chip
  and a 760 px composer move at similar visual speeds without the chip spinning frantically.

### Modes and morphs
- **Twin beams.** Second beam offset by P ÷ 2, same direction (`twin`) or opposite (`twin-reverse`).
  Where opposite beams cross, brightness adds but is clamped to the head colour (no flash).
- **Indeterminate → determinate.** When the first fraction arrives, the beam's head keeps travelling to
  the point p·P (shortest way forward, never backwards) while its tail anchors at the start point and
  stretches; 220 ms `spring.snap`. From then on the head follows p.
- **Determinate follow.** The lit length eases toward p·P (`spring.snap`, ≤ 250 ms lag); never shrinks
  (monotonic rule, container-loaders.md §4.2).
- **Steps.** n tick gaps (2 px) are cut into the track; each completed step lights its segment with a
  120 ms fill; above 24 steps the track is continuous and the badge carries "k/n".
- **Completion.** Stroke closes, one bright pass runs the whole outline in 400 ms, then the layer fades
  over 240 ms (tiles may hold the success tone for 600 ms first). Single pass, never repeated.
- **Failure.** Lit part freezes and cross-fades to danger (160 ms); the remainder becomes a dashed
  danger line so "how far it got" stays visible (same idea as the attachment ring error in
  composer-attachments.md); tiles do one 4 px shake (300 ms); composers never shake.
- **Approval.** Beam decelerates to rest (200 ms `ease.firm`), then widens along the whole outline into
  a steady attention tone. Motion off = "waiting on you".
- **Stall.** Speed 1× → 0.5× over 2 s; tone ramps toward warning (OKLab). First new event snaps back
  in 200 ms.
- **Steer.** One lap at 2× speed, then back to normal (composer light only).
- **Hover.** No change to the beam; the badge slot reveals ✕.

### Composer border light (specifics)
- Band = beam with `arc` (fraction of perimeter, default 0.22, proposed), a crisp 1 px leading line
  and an **inward bloom** (8–16 px, strength 0.25, proposed) clipped to the composer's interior so text
  stays readable; 4 colour stops; `reverse` option. Bloom requires the shader tier; without it the band
  is a plain gradient stroke.
- Turning on and off is a fade of the whole effect (180 ms in / 120 ms out), so the composer "lights up"
  (BoardUI behaviour as described in research note 16).
- Follows `BusySignal`: subtle (no bloom, 50 % intensity) when the status tab is already looping in the
  composer block (container-loaders.md §4.3); stops for `AwaitingYou`.
- Composer height changes (queue stack, attachments) rebuild the arc-length table; the band keeps its
  normalised position so it does not jump.

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Show delay | — | 150 ms | — | proposed |
| Arming (stroke 0 → width, head fade in) | 180 ms | 0 | `ease.out` | |
| Lap (`loop.beam`) | clamp(P ÷ 400 px/s, 1.6 s, 4.0 s) | twin offset P ÷ 2 | linear in arc length | 60 fps; 30 fps when > 12 hosts animate |
| Indeterminate → determinate | 220 ms | 0 | `spring.snap` | head forward only |
| Determinate follow | continuous, ≤ 250 ms lag | — | `spring.snap` | |
| Step segment light | 120 ms | 0 | `ease.out` | |
| Completion pass | 400 ms | 0 | `ease.in-out` | single |
| Success hold (tiles) → fade | 600 ms → 240 ms | — | — / `ease.out` | |
| Failure tint / shake | 160 ms / 300 ms, 2 cycles, 4 px | 0 | `ease.out` / linear | no shake on composer |
| Approval rest | 200 ms | 0 | `ease.firm` | then static |
| Stall slow-down + tone ramp | 2 s | after stall threshold | linear (OKLab) | thresholds: status strip |
| Steer pass | 1 lap at 2× | 0 | linear | composer only |
| Composer light on / off | 180 ms / 120 ms | 0 | `ease.out` / `ease.in` | |
| Submitted rim pulse | 400 ms | 0 | `ease.in-out` | < 500 ms per architecture.md |
| Cancel | 160 ms | 0 | `ease.in` | |
| **Reduced motion** | no laps: `Running` = static accent outline at 70 % + badge "Working"; `Filling(p)` = static stroke to p, updated with an 80 ms fade; completion = 80 ms tone change; no bloom motion, no shake | — | — | state always also in badge / label |

## 6. Variants

### Free (4)
| Name | Idea |
|---|---|
| `border-beam/single` | One gradient beam laps any outline at constant arc-length speed (the baseline) |
| `border-beam/twin` | Two beams offset by half the perimeter, same direction |
| `border-beam/trace-progress` | Determinate: stroke draws from top centre to the job's fraction, ticks for steps, closes with one pass |
| `border-beam/composer-light` | Composer rim band with leading line; bloom on the shader tier, plain gradient otherwise; obeys the busy signal |

### Pro (20)
| Name | Idea |
|---|---|
| `border-beam/twin-reverse` | Two beams in opposite directions that cross twice per lap |
| `border-beam/comet` | Short bright head with a long exponential tail (30 % of perimeter) |
| `border-beam/spring` | Beam surges and coasts each lap (speed eased per lap, not per corner) |
| `border-beam/multi-color` | 4-stop spectral beam whose hue rotates slowly along the lap |
| `border-beam/dashed` | Beam made of dashes; dashes march when indeterminate, fill when determinate |
| `border-beam/hairline` | 0.5–1 px beam with no glow for dense IDE panels |
| `border-beam/conic-rotate` | Rotating conic-gradient border (angle-driven, the `@property --angle` look) |
| `border-beam/conic-aurora` | Conic border with blurred aurora haze outside the outline (`outset`) |
| `border-beam/conic-pulse` | Conic border whose intensity breathes with `loop.glow` |
| `border-beam/bloom` | Composer-light band with refraction line and inward bloom, shader tier |
| `border-beam/squircle-trace` | Determinate trace tuned for squircle app-icon tiles |
| `border-beam/circle-trace` | Avatar / orb ring trace (agent avatar busy ring) |
| `border-beam/triangle-trace` | Triangle outline trace (warning / pyramid marks; research note 02 C3) |
| `border-beam/hex-trace` | Hexagon outline trace (node / plugin tiles) |
| `border-beam/path-trace` | Any custom path (logo, mascot outline) traced or filled by fraction |
| `border-beam/segmented` | Outline split into n lanes for parallel sub-jobs, each lane fills on its own |
| `border-beam/effort-speed` | Lap speed and beam length scale with the selected thinking effort |
| `border-beam/mode-flash` | One coloured pass on mode or permission change, then off |
| `border-beam/pixel` | Beam quantised to 2–4 px cells, stepped at 12 fps (pixel look) |
| `border-beam/terminal` | TUI box border: `─` cells brighten to `━` in a moving window of 6 cells, 10–12 fps |

## 7. Edge cases
- **Tiny hosts (16–20 px).** Stroke 1 px, L min 8 px, lap min 1.6 s; twin collapses to single; ticks
  hidden (badge in tooltip).
- **Radius larger than half the side.** Clamped; a 20 × 20 host with r = 16 becomes a circle.
- **Non-uniform radii** (cards with square bottom corners docked to a toolbar). Handled by the per-corner
  arc table; the start point stays top centre.
- **Very elongated hosts** (760 × 56 composer, 1 px divider rows). L uses perimeter, so it stays a beam,
  not a line; on hosts < 8 px tall the beam runs along the top edge only (`edge = Top`).
- **Host resize / composer growth.** Table rebuilt; normalised position kept; determinate fill keeps p.
- **Hosts in a grid.** Neighbouring beams can look like one long beam jumping; default staggers phase by
  `index × 0.13` laps (proposed) so they don't align; `container-loader/grid-sync` is the deliberate
  alternative.
- **Busy media under the stroke.** 1 px inner contrast halo (container-loaders.md §6).
- **RTL.** Counter-clockwise from top centre; badges mirror.
- **Touch.** No hover; ✕ always visible on cancellable hosts at ≥ 44 px hit area.
- **Very fast jobs.** Hidden by `show_delay`; if p jumps 0 → 1 in one update, skip straight to
  `Completing` (no fill animation).
- **Very slow jobs (hours).** Laps continue at fixed speed; `Stalled` covers silence. Determinate fills
  with tiny increments still move (sub-pixel accumulation).
- **Fraction regression / restart.** Held; retry resets with a 160 ms fade-out and redraw.
- **High contrast / forced colours.** Beam becomes a solid 2 px system-highlight segment; track uses
  system border colour.
- **Terminal.** `border-beam/terminal` only; determinate fill lights border cells proportionally.

## 8. Accessibility
- The beam is decorative (`aria-hidden`). The host carries `aria-busy` and, for `Filling` / `Stepping`,
  `role="progressbar"` with `aria-valuenow` / `aria-valuetext` ("Uploading report.pdf, 41 %",
  "Step 3 of 12"); `Running` exposes an indeterminate progressbar (no `aria-valuenow`).
- **Labels per state:** Running "Working on <host label>"; Filling "<label>, <p> percent"; Stepping
  "<label>, step k of n"; AwaitingYou "<label> needs your input"; Stalled "<label>, no progress for
  <n> seconds"; Paused "<label> paused, offline"; Failed "<label> failed. Retry available";
  Cancelled "<label> cancelled".
- **Live region:** milestones per container-loaders.md §7 (≤ every 10 s or 25 % steps); failure
  assertive. The composer light announces nothing (family B owns global busy announcements).
- **Flashing:** completion pass is a single travelling highlight (not a full-area flash); laps are
  ≥ 1.6 s; no more than 3 flashes per second in any variant, including `pixel`.
- **Contrast:** the static fallbacks (attention, warning, danger outlines; determinate stroke) meet
  3:1 against the host surface and the host's background.
- **Focus:** the focus ring and the beam never share a path: focus ring sits 2 px outside the outline
  (offset), the beam inside; when focused, the beam continues.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct EdgeTrace {
    pub agent: AgentState,               // global; composer light reads only this + busy
    pub busy: BusySignal,
    pub job: Option<JobSnapshot>,        // per-host job; drives Filling / Stepping
    pub shape: ContainerShape,           // container-loaders.md §8
    pub mode: TraceMode,                 // Auto (from job.progress) | Beam | Fill | Steps
    pub variant: BeamVariant,            // Single | Twin | TwinReverse | Comet | Conic.. | ComposerLight | ...
    pub start: StartPoint,               // TopCenter (default) | TopLeading | Normalized(f32)
    pub direction: TraceDirection,       // Clockwise | CounterClockwise | Auto (RTL-aware)
    pub stroke: Pixels,                  // 1.5 px default (proposed)
    pub beam_len: BeamLen,               // Auto = clamp(0.18·P, 48, 240) | Px | Fraction
    pub speed: BeamSpeed,                // Auto = clamp(P/400 px/s, 1.6 s, 4.0 s) | LapSecs(f32)
    pub colors: GradientStops,           // 2–4 stops; default accent ramp from tokens
    pub bloom: Option<Bloom>,            // { inward: Pixels, strength: f32 } (shader tier)
    pub phase_offset: f32,               // 0..1 laps (grid stagger)
    pub motion: MotionLevel,
    pub quality: RenderQuality,
}

impl EdgeTrace {
    /// Composer border light preset bound to the composer's `ComposerState`.
    pub fn composer_light(state: &ComposerState, shape: ContainerShape) -> Self { /* ... */ }
}
```

## 10. Related components and references
Related: composer input shell (busy tint) · send control `border-beam` · status strip `border-beam` ·
mode switch / effort control beams · attachment tile ring · tool card (family F) · avatar busy ring
(family K/L) · `container-loader/grid-sync`.

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Libraries.dev `border-beam` package | [R100], research note 16 | beam as a standalone primitive; SwiftUI / RN parity idea | Packages MIT: may port with attribution |
| BoardUI Composer Loader (`colors` 4 stops, `speed` s/lap, `arc`, `intensity`, `bloom`, `bloomStrength`, `reverse`, `radius`; stops under reduced motion) | [R095], research note 16 | composer-light parameter set and fade-on behaviour | Free tier MIT; confirm the item is in the free tier before porting, otherwise study only |
| Magic UI Border Beam (Motion along `offset-path`; size, duration, delay, colorFrom/To, reverse, borderWidth; 2-beam demos) and Shine Border | research note 02 | twin beams, reverse | Licence not recorded in our notes: study only until checked |
| UploadKit `UploadBeam` (beam bound to upload lifecycle; success / error colours then fade; `@property` + conic + mask-composite) | research note 02 | lifecycle-bound beam | Licence not recorded: study only |
| `@property --angle` conic border technique (theosoti, gradients.sh, webberui guides) | research note 02 | `conic-*` variants on the CSS path | Technique, not code; write our own |
| Sundae UI composer attachments (ring draws around the tile's own outline, counting %) | research note 02 | `trace-progress` | Search excerpt only; study only |
| Codepen trend: `@property` conic borders, Apple-Intelligence glow | research note 02 | — | Study only |
| OpenCode tab pulse (2.8 s running pulse, 1.2 s completion pulse) | research note 08 | single completion pass | MIT: may port with attribution |
| Aceternity effects | [R122] | — | Paid tier: study only |

Gaps: none of the research-note-02 beam sources has a `sources.csv` ID yet; Magic UI and UploadKit
licences need a check before anything beyond behaviour is reused.
