**Status:** draft v0.1 (2026-11-12).

# Spec: animated marks (20 styles × 10 states)

Part of family L ([icons.md](icons.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). An **animated mark** is a product logo (the
app's, a user's, or Oxyn's) that shows the agent's state. This spec defines the **mark engine**: one
state model, **20 styles** (how a mark moves) × **10 states** = **200 combinations**, an import
pipeline so any user SVG logo plugs in, and the **Oxyn mark** as one preset.

| Tier | Styles | Combos (× 10 states) |
|---|---|---|
| Free | 5 | 50 |
| Pro | 15 | 150 |
| **Total** | **20** | **200** |

Relation to [loader-mascot.md](loader-mascot.md): that spec places marks and mascots in the loader
slot of the composer region. This spec is the engine underneath. `loader-mascot/oxyn-mark` =
`mark/particle-core` + the Oxyn preset; its Pro mark-engine variants map to styles here
(`mark-draw-on` → `mark/draw-on`, `mark-breathe` → `mark/breathe`, `mark-hue-settle` →
`mark/hue-settle`, `mark-nodes` → `mark/node-graph`, `mark-particle` → the Pro `dissolve` option of
`mark/particle-core`, §6.3). Tiers are kept consistent with that spec.

## 1. Purpose
Agent products put their logo to work as the "it's thinking" signal: a breathing star glyph in a
terminal, a spark that collapses into a rotating node graph, a mark that rotates from the cursor, a
gradient spark that morphs through shapes, a pulsing heart (research note 09 §2.1). Two lessons from
the same research shape this spec:
1. **Legibility at 16–20 px decides success.** A desktop app's node mark drew complaints because at
   20 px its connecting lines vanished and it read as "loose orange dots" (note 09 §2.1). Every style
   here passes a 16 and 20 px test or swaps to a simpler form.
2. **One engine, many brands.** The recommended architecture is a logo-agnostic engine (frames, morph
   targets, gradient stops) plus brand presets, so a user drops in an SVG and gets the states for free
   (note 09 §2.3).

Where marks appear: the app header or sidebar avatar (idle / success), the composer loader slot and
status row (thinking … streaming), agent avatars in multi-agent rosters, the tray / menu-bar icon,
empty states and splash screens. Users want **brand presence without losing the information a
spinner carries**: each state reads differently, and waiting-on-you never looks like working.

Rules:
1. **State = motion family**, not just speed: waiting holds and breathes, stalled slows and dims,
   error stops; only thinking / working / tool / streaming loop.
2. **One animating indicator per region** (loader-mascot rule 2): a mark that is not its region's
   lead shows its still for the current state.
3. **Calm by default:** idle gestures ≤ 1 per 4 s; still after 30 s idle.
4. **No product's mark or frames are copied.** Styles are re-implemented behaviours with our own
   geometry and timings (note 09 caveat: "do not copy brand marks").
5. **The Oxyn mark is a preset, not a special case**; user marks get every state.

## 2. State machine
The 10 states (taxonomy row L) map from `AgentState` through `LoaderPhase`
([loaders.md](loaders.md) §3; [../architecture.md](../architecture.md)).
```
                   app ready                       Submitted / Thinking
   Off ─────────────────────▶ IDLE ─────────────────────────────────────▶ THINKING (1.0×, loop.calm)
                               ▲  └─ 30 s ─▶ (asleep: idle still)           │   ▲ tool ends
                               │                            tool starts     ▼   │
                               │                        ┌──────────────▶ TOOL (activity tell, 1.25×)
                               │                        │                   │
                               │     model-side Working │ (Edit, Generate)  │ first token
                               │                        └── WORKING (1.25×) ▼
                               │                                         STREAMING (0.8×, token-coupled)
                               │                                            │ approval / question
                               │                                            ▼
                               │                                         WAITING (hold + breathe, amber)
                               │   any active ── 5 s no event ──▶ STALLED soft (0.5×, dim)
                               │                                    └─ 45 s ─▶ STALLED hard (still, warn)
                               │                                    (event → previous state, 200 ms)
                               ├── 1.2 s ── SUCCESS (settle + hold) ◀── Done
                               ├── 200 ms ─ (fade to idle)           ◀── Cancelled
                               └── dismiss ─ ERROR (static)          ◀── Failed
   REDUCED-MOTION: overlay on every state above → the designed still for that state (10th column)
   Other overlays: NotLead (still) · Offscreen (paused) · Steering (one 240 ms brightness blip)
```
Mapping: `idle` = Idle / Rest / Composing (Composing may lean the mark 1 px toward the caret in
Full styles); `thinking` = Submitted + Thinking; `working` = `Working { activity }` with no tool
(Edit, Generate in-model); `tool` = `Working` with a tool ref (`ToolRunning`); `waiting` =
`AwaitingApproval`; `streaming` = Streaming; `success` = Done; `error` = Failed; `stalled` =
Stalled soft / hard (derived by `oxyn-core`); `reduced-motion` = the global `motion.reduced` token.

## 3. States table
Marks are not controls; actions belong to the host (status strip, avatar menu).

| State | Engine rule (all styles) | Colour | Host actions | Keyboard |
|---|---|---|---|---|
| `idle` | rest pose; optional gesture ≤ 1 per 4 s; still after 30 s | `mark.fg` (brand or `currentColor`) | — | not focusable (or host avatar) |
| `thinking` | style's thinking loop at 1.0× (`loop.calm` 1600–2400 ms) | `mark.fg`, accent optional | Stop | — |
| `working` | same family at 1.25× (`loop.brisk`) | accent | Stop | — |
| `tool` | 1.25× plus the style's **tool tell** (a satellite, a tick, a raster scan) | accent | Stop, open tool | — |
| `waiting` | **different motion**: loop eases to a hold pose (400 ms), then a 2.4 s breathe | `color.loader.waiting` (amber) | Review | — |
| `streaming` | 0.8×; Full styles may couple to the smoothed token rate (≤ 2 pulses / s) | `mark.fg` | Stop | — |
| `success` | finish current beat (≤ 200 ms), settle to rest pose (≤ 600 ms), hold 600 ms, idle | `success` briefly, then `mark.fg` | — | — |
| `error` | morph to the style's error pose (160 ms `spring.snap`), static until dismissed | `color.loader.danger` | Retry | — |
| `stalled` | soft: tempo → 0.5× and opacity → 60 % over 2 s; hard (45 s): freeze on current pose, `warn` tint | `color.loader.warn` | Retry / Stop | — |
| `reduced-motion` | the designed still for the current state (§5.3 column) | per state | same | same |

## 4. Micro-interactions
- **Enter:** fade + scale 0.85 → 1 over 180 ms `ease.out`, already in phase with the shared clock.
- **State change:** crossfade between loops over 200 ms on the shared clock; tempo changes ease over
  400 ms (phase-continuous, never a jump in rotation).
- **Submitted:** a single 240 ms nod (scale 1 → 0.94 → 1) before the thinking loop, so a sub-300 ms
  answer shows a nod, not a loop (`show_delay` 300 ms from loaders.md).
- **Waiting:** the loop decelerates to its hold pose in 400 ms; the amber breathe begins; the host
  shows the reason ("Needs your approval").
- **Success:** the style's settle (ring closes, dots reform, hue settles) then a 600 ms hold; no
  confetti by default.
- **Error:** the style's error pose (slump 2 px, scatter, ✕ cut); the host nudges, not the mark.
- **Stall recovery:** colour and tempo snap back in 200 ms on the next event.
- **Steer:** brightness +25 % for 240 ms.
- **Hover (when the mark is an avatar):** tooltip "Agent · thinking · 12 s"; no motion change.
- **Click reaction (opt-in, Pro):** one short spin or bounce ≤ 1 per 2 s; ignored in waiting, error
  and stalled-hard (loader-mascot rule).

## 5. Timing

### 5.1 Engine timing table (all styles)
| Transition | Duration | Delay | Easing / spring | Frame rate |
|---|---|---|---|---|
| Enter | 180 ms (`dur.base`) | 0 | `ease.out`, scale 0.85 → 1 | 60 Hz |
| Idle gesture | 400–800 ms, ≤ 1 per 4 s | seeded 4–7 s | `spring.soft` | 60 Hz, then idle (no repaint) |
| Submitted nod | 240 ms | 0 | `spring.snap` | 60 Hz |
| Thinking loop | 1600–2400 ms (`loop.calm`) | 0 | per style | ≤ 60 fps (T2), ≤ 30 fps (T3 at ≤ 24 px) |
| Working / tool loop | 1000–1200 ÷ 1.25 → 800–960 ms (`loop.brisk`) | 0 | per style | same |
| Streaming | loop × 1/0.8; token pulses ≤ 2 / s | 0 | per style | same |
| State crossfade | 200 ms | 0 | `ease.in-out` | 60 Hz |
| Tempo change | 400 ms | 0 | `ease.in-out` on speed | — |
| Waiting stop + breathe | 400 ms + 2400 ms loop | 0 | `ease.out` / sine | ≤ 30 fps |
| Success settle / hold / return | ≤ 600 ms / 600 ms / 240 ms | finish beat ≤ 200 ms | `spring.soft` / — / `ease.in` | 60 Hz |
| Error pose | 160 ms | 0 | `spring.snap` | 60 Hz, then static |
| Stall soft ramp / hard freeze | 2000 ms / 400 ms | 5 s / 45 s no event | linear / `ease.out` | halves with tempo |
| Recovery snap | 200 ms | 0 | `ease.out` | 60 Hz |
| Cancelled → idle | 200 ms | 0 | `ease.in` | 60 Hz |
| Steer blip | 240 ms | 0 | `ease.out` | 60 Hz |
| **Reduced motion** | designed still per state (§5.3); state changes are 0–80 ms crossfades; no gestures, nods, loops, breathe, token pulses; opt-in SlowFade (2 s opacity 1 ↔ 0.6) on busy states only | 0 | linear | repaint on state change only |

Values are proposed and reuse the loader tokens (`loop.*`, `dur.*`, `ease.*`, `spring.*`) from
[loaders.md](loaders.md) and [composer.md](composer.md).

### 5.2 Style × state matrix (the 200 combinations), part A
Mark use: **Full** animates the mark's own geometry (needs the import pipeline, §6.2); **Frame**
keeps the mark still and animates around it; **Swap** shows the mark at idle / success and a style
glyph while busy. Tier column: render tier from [loaders.md](loaders.md) §2.2.

| # | Style (tier) | idle | thinking | working | tool | waiting |
|---|---|---|---|---|---|---|
| 1 | `pacing-dot` (T2, Swap) | mark at rest | mark shrinks to one dot that pulses 1 → 1.15 | dot pulses faster | dot + short label chip ("Searching") | dot holds, amber breathe |
| 2 | `ring-step` (T2, Frame) | thin full ring at 20 % | arc (90°) travels round the mark | arc 1.25× | arc gains a tick per tool call | arc stops at top, ring breathes amber |
| 3 | `star-flip` (T1/T2, Swap) | mark at rest | star flipbook `· ⋆ ✧ ✦` mirrored | flipbook 1.25× | star + one orbiting spark | hourglass frames (sand) |
| 4 | `orbit-merge` (T2, Frame) | three dots parked under the mark | dots orbit, merge, breathe, split | orbit 1.25× | one dot detaches to a satellite | dots merge into one, breathe |
| 5 | `particle-core` (T3, Full) | dot field holds the rest shape | dots morph between shape targets | morph 1.25× | orbit / satellite dots run (preset-defined) | morph pauses on a target, breathe |
| 6 | `breathe` (T2, Full) | still | scale 1 → 1.06, opacity 0.85 → 1 | 1.25× | breathe + tick on each tool | slow amber breathe 2.4 s |
| 7 | `draw-on` (T2, Full) | filled mark | outline segment chases the contour | chase 1.25× | segment leaves a dash per tool | segment parks at the start point |
| 8 | `hue-settle` (T2, Full) | neutral | hue sweeps across the mark | sweep 1.25× | sweep pauses on accent per tool | hue holds amber |
| 9 | `node-graph` (T2, Swap) | mark | mark collapses to a dot, 3-node graph rotates | 4-node graph | node count = active tools (≤ 4) | graph stops, one node amber |
| 10 | `shape-shift` (T2, Full) | mark | mark → 3 primitive shapes, +45° and scale dip per step | 1.25× | shape per activity (search = circle …) | holds on a rounded square |
| 11 | `gradient-flow` (T2/T4, Full) | static 2-stop gradient | stops rotate inside the fill | 1.25× | a third stop flashes in per tool | gradient freezes, amber stop |
| 12 | `twinkle` (T2, Frame) | mark | two sparks twinkle out of phase | + mark turns 90° per cycle | third spark per tool | sparks dim, mark still |
| 13 | `revolve` (T2, Full) | mark | rolodex turn (scaleX) at a calm fixed speed | 1.25× | half-turn per tool call | stops face-on, breathe |
| 14 | `iso-cube` (T2/T3, Full) | isometric slab, still | slab spins on Y | 1.25× | spins on X for tool calls | stops on a face, pause bars on it |
| 15 | `heartbeat` (T2, Full) | still | lub-dub 1 → 1.12 → 1 → 1.06 → 1 per 1.6 s | per 1.2 s | extra beat per tool | single slow beat per 2.4 s, amber |
| 16 | `braille-raster` (T1, Full) | mark rastered in braille | scanline lights rows | 1.25× | raster flickers column by column | hourglass sand in the raster |
| 17 | `pixel-assemble` (T2, Full) | assembled pixels | edge pixels shimmer in a seeded order | 1.25× | pixels drift toward a corner tool tick | pixels freeze, amber outline pixels |
| 18 | `spotlight` (T2/T4, Frame) | mark in a tile | light pulse travels through the tile | 1.25× | pulse starts from the tool's side | pulse stops, tile rim amber |
| 19 | `conic-halo` (T2, Frame) | slow conic ring (8 s/turn) | ring 3 s/turn | 2.4 s/turn | ring segment per tool | ring stops, amber |
| 20 | `ascii-bloom` (T1, Full) | mark glyph | characters bloom out from the centre on a density ramp | 1.25× | bloom leaves the tool's glyph | bloom holds, amber `?` centre |

### 5.3 Style × state matrix, part B
| # | Style | streaming | success | error | stalled | reduced-motion still |
|---|---|---|---|---|---|---|
| 1 | `pacing-dot` | dot pulses with token rate | dot grows back into the mark | dot → ✕ | pulse 0.5×, dims | mark (idle), dot (busy), amber dot (waiting), ✓ / ✕ |
| 2 | `ring-step` | arc 0.8×, brightens on bursts | ring closes, ✓ cut in centre | ring breaks at bottom, danger | arc crawls, warn | full ring (idle), 3/4 arc (busy), amber ring, closed ring, broken ring |
| 3 | `star-flip` | flipbook 0.8× | star → ✓ | star → ✕ | mid frame held, warn | `✦` busy, `⧗` waiting, ✓, ✕ |
| 4 | `orbit-merge` | dots trail the stream direction | dots merge into the mark | dots drop below the mark | orbit 0.5×, dims | three parked dots; one merged dot for waiting |
| 5 | `particle-core` | dots stream off the edge with tokens | dots reform the rest shape, hold | dots sag 2 px, danger tint | morph 0.5×, then freeze | rest shape (idle / success), mid shape (busy), danger tint |
| 6 | `breathe` | 0.8× | one 1.1 pulse, settle | slump 2 px | slow, dim | mark at 100 % / 85 % (busy) |
| 7 | `draw-on` | stroke writes continuously | outline closes then fills | stroke stops, gap visible | segment crawls | filled (idle), outline (busy), outline + gap (error) |
| 8 | `hue-settle` | sweep 0.8× | hue settles to neutral | danger hue, static | sweep slows, desaturates | neutral / accent / amber / danger fills |
| 9 | `node-graph` | edges pulse along with tokens | graph collapses to dot → mark | graph splits, danger node | rotation 0.5× | 3-node graph still; ✓ / ✕ |
| 10 | `shape-shift` | 0.8× | collapses + fades back to mark | stops on a cut shape | morph 0.5× | mark / primitive / ✓ / ✕ |
| 11 | `gradient-flow` | stops drift at token rate | gradient settles to brand stops | danger stop, frozen | slow, desaturated | static gradient per state |
| 12 | `twinkle` | sparks trail rightward | sparks merge into the mark | sparks drop out | twinkle 0.5× | mark + 2 static sparks (busy) |
| 13 | `revolve` | 0.8× | stops face-on (or hands off to a host shape) | stops edge-on, danger | 0.5× | face-on mark; edge-on for error |
| 14 | `iso-cube` | 0.8× | settles flat, ✓ face | tips 15°, danger | 0.5×, then freeze | slab per state face (✓, ✕, ‖) |
| 15 | `heartbeat` | beat follows token bursts (≤ 2 / s) | one strong beat, settle | flatline | beat every 3 s | mark; amber for waiting |
| 16 | `braille-raster` | rows fill bottom-up with tokens | raster fills solid, then mark | raster rows break | scan 0.5× | still raster; `⧗` waiting |
| 17 | `pixel-assemble` | pixels stream in from the trailing edge | last pixel snaps in | pixels scatter 2 px | shimmer 0.5× | assembled; scattered (error) |
| 18 | `spotlight` | pulse 0.8× | one bright pass, fade | tile rim danger | pulse dims | lit tile (busy), amber rim, plain tile |
| 19 | `conic-halo` | 0.8× | ring snaps full, fades | ring stops, danger segment | 0.5× | static ring segment per state |
| 20 | `ascii-bloom` | bloom pulses with tokens | collapses to the mark glyph | collapses to `x` | bloom 0.5× | mark glyph; `?` waiting; `x` error |

Each still in the last column is a designed frame shipped with the style (not a paused frame), so
reduced motion and NotLead show a deliberate pose (loader-mascot rule "pose = phase").

## 6. Variants, user marks and presets

### 6.1 Style catalogue
**Free (5 styles = 50 combos)**
| Name | One-line idea |
|---|---|
| `mark/pacing-dot` | The mark collapses to a single pacing dot while busy and grows back on success (idea: a pacing dot for pure waiting, note 09) |
| `mark/ring-step` | The mark sits in a thin ring whose arc tracks the state and closes on success (idea: a loading ring round an avatar, note 09) |
| `mark/star-flip` | Our own mirrored star flipbook stands in for the mark while busy (same frames as `loader/glyph-spark`) |
| `mark/orbit-merge` | Three dots orbit the mark, merge into one, breathe and split (idea: community "Orbit" style, note 09) |
| `mark/particle-core` | The mark sampled into a depth-shaded dot field that morphs between shape targets; the Oxyn preset's default |

**Pro (15 styles = 150 combos)**
| Name | One-line idea |
|---|---|
| `mark/breathe` | Whole mark breathes in scale and opacity |
| `mark/draw-on` | A stroke segment chases the mark's outline; success closes and fills it |
| `mark/hue-settle` | Hue sweeps across the mark, then settles to neutral on success |
| `mark/node-graph` | Mark collapses to a dot, then a 3–4-node graph rotates, edges ≥ 2 px at 20 px |
| `mark/shape-shift` | Mark morphs through three primitive shapes with a +45° turn and scale dip per step |
| `mark/gradient-flow` | Gradient stops rotate inside the mark's fill; edges soften and sharpen |
| `mark/twinkle` | Two small sparks twinkle out of phase beside a slowly turning mark |
| `mark/revolve` | Rolodex-style rotation at a fixed calm speed; success can hand off to a host shape |
| `mark/iso-cube` | Mark extruded into an isometric slab that spins in 3D |
| `mark/heartbeat` | Lub-dub double beat; flatline on error |
| `mark/braille-raster` | Mark rasterised into braille cells with a scanline (shares data with `loader/glyph-mark-raster`) |
| `mark/pixel-assemble` | Mark built from pixels that assemble in, shimmer while busy, scatter on error (idea [R027]) |
| `mark/spotlight` | Mark in a tile while a light pulse travels through the tile |
| `mark/conic-halo` | Conic ring slow at rest, faster while busy, snaps full on success |
| `mark/ascii-bloom` | Density-ramp characters bloom from the centre and collapse back into the mark glyph |

### 6.2 Plugging in your own logo
1. **Supply** an SVG (and optionally 1–3 extra SVGs as morph targets, a draw-on start point, and a
   brand palette of ≤ 4 colours) through the Studio upload or a proposed `oxyn mark import <logo.svg>`
   CLI command (not yet in the CLI plan in [../architecture.md](../architecture.md)).
2. **Sanitise** with the same allow-list and caps as user plugin logos
   ([plugin-icons.md](plugin-icons.md) §5.4); flatten transforms; outline strokes.
3. **Normalise** into a 24-unit box with 2 units of padding; keep aspect; centre optically (area
   centroid, not bounding box).
4. **Derive** (all at import, nothing at runtime): simplified masters for 16 / 20 / 24 px
   (Douglas–Peucker tolerance 0.25 / 0.2 / 0.15 units, proposed); contours resampled to 64 points per
   sub-path for morph styles; dot samples (Poisson disc) capped per size at the loaders.md budgets
   (≤ 24 / 36 / 48 / 80 / 160 dots at 16 / 20 / 24 / 32 / 48 px); braille, pixel and ASCII rasters at
   16 and 24 px; arc-length parameterisation for `draw-on`; the 10 designed stills per style.
5. **Legibility test** at 16 and 20 px: fail if any stroke or gap renders under 1.5 px or a counter
   closes. On fail the importer offers (a) the simplified master, (b) a user-drawn small version, or
   (c) restricting the mark to Frame and Swap styles, which only need the still.
6. **Output** `marks/<name>/mark.toml` + data, loaded as `MarkSource`. The logo stays in the user's
   project; Oxyn never uploads it or adds it to shared presets.

Style requirements: **Full** styles need contours / dots / rasters; **Frame** and **Swap** styles work
with any still image, including a raster logo.

### 6.3 Presets
A preset is a `MarkSource` plus per-style overrides (shape targets, palette, satellites). Oxyn ships
one brand preset; users add theirs.

**`oxyn` preset** (logo v3, `assets/brand/`, unchanged). From `render_logo.py`: a matte dot core laid
on a rounded cube's faces (an 11 × 11 grid per face) that morphs cube → tube → sphere → cube with
sine easing; two thin muted orbits locked in an X, each spinning on its long axis; a token-stream
caret (a rounded bar with trailing token squares) travelling each orbit in a muted ember accent;
depth carried by dot size and ink weight, no glow; the rendered loop is 5 s at 30 fps. Ink, ring and
ember colours in the script become the preset's palette tokens.

| State | Oxyn preset on `particle-core` (proposed) |
|---|---|
| idle | holds the cube; orbits drift at 0.25×; carets parked |
| thinking | the logo's own morph loop (5 s); carets travel |
| working | loop at 1.25× (4 s) |
| tool | orbits 1.5×; carets emit token squares |
| waiting | morph pauses on the sphere; orbits stop; carets park at the top and breathe amber 2.4 s |
| streaming | 0.8×; token squares leave the carets at the smoothed token rate (≤ 2 / s) |
| success | settles to the cube (≤ 600 ms); carets merge into the core; hold 600 ms |
| error | dots sag 2 px; orbits stop; carets turn `danger` |
| stalled | soft 0.5× and 60 % opacity; hard: freeze on the current shape, carets `warn` |
| reduced-motion | still cube with both orbits and parked carets; sphere + amber carets for waiting; danger carets for error |

Small sizes (proposed; to confirm in the V2.3 "logo legible at 16–20 px" review,
[../roadmap.md](../roadmap.md)): 16 px = 3 × 3 front-face dots, no orbits, caret as one 2 px dot;
20 px = 4 × 4 face with shaded side dots (≤ 36), one orbit; 24 px = ≤ 48 dots, both orbits;
32 / 48 px = loaders.md dot caps; ≥ 64 px = the full logo dot count.

`particle-core` options: `Subtle` / `Expressive` motion levels are free; the **`dissolve`** option
(mark dissolves into a free dot field and re-forms, = `loader-mascot/mark-particle`) is Pro.

## 7. Edge cases
- **Tiny sizes (12 px):** marks are not drawn below 16 px; the slot shows `loader/dots-pulse` (as
  loader-mascot does); Swap styles show their busy glyph.
- **Complex or thin logos:** import legibility test (§6.2 step 5); Full styles disabled for marks that
  fail at 20 px.
- **Wordmarks / wide logos:** aspect > 2:1 is rejected for Full styles; use the logo's symbol or a
  Frame style around a monogram ([plugin-icons.md](plugin-icons.md) §5.3).
- **Multi-colour logos:** Full styles that tint (hue-settle, waiting amber) tint a duplicate overlay at
  60 % instead of recolouring brand colours; error and waiting also change pose, so meaning never
  rests on tint.
- **Light / dark:** preset palettes declare both; additive glows (spotlight, halo) use normal blending
  on light themes.
- **RTL:** directional motion (draw-on direction, revolve, streaming trails) mirrors; the mark itself
  never mirrors.
- **Very fast agent:** `show_delay` 300 ms → just the nod.
- **Very slow agent (> 10 min):** tempo eases to the `loop.calm` floor; idle gestures off.
- **Many marks on screen** (agent roster of 12): only region leads animate; others show stills; T3
  dots drop to ≤ 30 fps; T4 falls back per tier chain (loaders.md §2.2).
- **Offscreen / minimised:** paused on the shared clock; resumes in phase.
- **Terminal surface:** only T1 styles (`star-flip`, `braille-raster`, `ascii-bloom`) and stills.
- **Tray / menu-bar icons:** platform template images are monochrome; use `pacing-dot`, `ring-step` or
  stills; frame rate ≤ 10 fps (proposed) to respect battery.
- **Touch:** marks are never controls unless the host makes them one.

## 8. Accessibility
- **Decorative** by default (`aria-hidden`, not exposed in GPUI); as an avatar with no visible name the
  host labels it with state ("Agent, waiting for your approval").
- **No live regions** from marks; state announcements come from the composer's single polite region
  (assertive for waiting and error), per [loaders.md](loaders.md) §8.
- **Not colour alone:** each state has a distinct motion family and still (§5.3), plus host text.
- **Flashing:** nothing changes the luminance of a large area faster than 3 per second; token pulses
  ≤ 2 / s; spotlight and halo stay ≤ 30 % opacity over their tile (proposed); success passes once.
- **Reduced motion:** designed stills (§5.3), 0–80 ms swaps.
- **Contrast:** mark ≥ 3:1 against its background in every state, including stalled dim (dim via
  weight / desaturation; opacity floor 0.6).
- **Attention:** idle gestures ≤ 1 per 4 s and stop after 30 s.

## 9. Props sketch
```rust
pub struct AnimatedMark {
    pub agent: AgentState,                  // → MarkState::derive(&agent, stall, tool)
    pub source: MarkSource,                 // Preset(MarkPreset::Oxyn) | User(MarkData) | Still(ImageSource)
    pub style: MarkStyle,                   // PacingDot | RingStep | StarFlip | OrbitMerge | ParticleCore (free)
                                            // | Pro(MarkStylePro)
    pub size: MarkSize,                     // Px16 | Px20 | Px24 | Px32 | Px48 | Hero(u16)
    pub tone: MarkTone,                     // Brand (preset palette) | Current | Accent
    pub region: BusyRegion,                 // shared busy signal; NotLead → still
    pub priority: BusyPriority,
    pub policy: MascotPolicy,               // Off | Calm (default) | Lively — shared with loader-mascot
    pub token_coupling: bool,               // streaming pulses follow token rate (Full styles)
    pub options: MarkOptions,               // e.g. ParticleCore { dissolve: bool /* Pro */ }
    pub labels: LoaderLabels,
    pub motion: MotionLevel,
}

pub enum MarkState { Idle, Thinking, Working, Tool, Waiting, Streaming, Success, Error, Stalled(StallLevel) }
// reduced-motion is not a state: MotionLevel::None selects the designed still for MarkState.

pub struct MarkData {                       // produced by the importer (§6.2), static at runtime
    pub masters: [PathSet; 3],              // 16 / 20 / 24
    pub contours: Vec<[Point; 64]>,         // morph-ready
    pub morph_targets: Vec<Vec<[Point; 64]>>,
    pub dots: DotSets,                      // per size cap
    pub rasters: Rasters,                   // braille / pixel / ascii at 16 and 24
    pub stills: StillSet,                   // 10 designed stills per enabled style
    pub palette: Vec<TokenColor>,           // ≤ 4
    pub legibility: LegibilityReport,       // gates Full styles
}
```

## 10. Related components and references
**Related:** [loader-mascot.md](loader-mascot.md) (mark slot, Oxyn mark behaviour, mascots) ·
[loader-glyph.md](loader-glyph.md) (`glyph-spark`, `glyph-mark-raster`) · [thinking-orb.md](thinking-orb.md)
(dot-field rendering) · [loaders.md](loaders.md) (phases, tiers, size caps, tokens) ·
[icons.md](icons.md) · [icon-states.md](icon-states.md) · [plugin-icons.md](plugin-icons.md)
(sanitiser, monogram) · [composer-send-control.md](composer-send-control.md) (`particle` variant
collapses into ■ "matching the Oxyn mark").

| Reference | What we take | Licence note |
|---|---|---|
| Research note 09 §2.1–2.3 (animated agent-state logos) | the 20-style shortlist, the 16–20 px legibility rule, engine + presets architecture | Our analysis; product marks studied are proprietary — **no geometry or frames copied** |
| Claude Code breathing star, Claude Desktop node mark (note 09) | mirrored-breath and spark → node-graph ideas (`star-flip`, `node-graph`) | Proprietary — reference only; our glyphs and timings |
| Gemini, Perplexity, Cursor, Lovable, Manus marks (note 09) | shape-shift, revolve, iso-cube, heartbeat, ring ideas | Proprietary — reference only |
| thinking-orbs (Jakub Antalik), gpui-thinking-orbs | dot-field depth language behind the Oxyn mark and `particle-core` | MIT — port with attribution ([../licensing.md](../licensing.md)) |
| Codex CLI braille logo raster (research note 08 via loader-glyph) | `braille-raster` idea | MIT per licensing.md — portable with attribution |
| cli-spinners (note 09 §2.2) | flipbook data model (frames + interval + mirror) | MIT — portable with attribution |
| Paper Shaders logo animations (note 17 §S1) | shader-material ideas for Pro T4 fallbacks (`gradient-flow`, `spotlight`) | Apache-2.0 — portable with attribution |
| Rive / dotLottie state machines (note 09 §2.2) | data-bound state inputs as an export target idea | Runtimes' licences not checked — study only |
| SavannahFeder pixel-grid logo [R027] | `pixel-assemble` | Study only |
| Orchids "ASCII flower" (note 09), praveenisomer ASCII forms [R011] | `ascii-bloom` | Study only |
| eliakuratli mascot [R031], dannyinsf_ pill mascot [R023] | state-per-pose discipline | Study only |
