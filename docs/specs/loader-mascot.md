**Status:** draft v0.1 (2026-11-11).

# Spec: mini mascots, animated marks and the busy glow (composer border light)

Part of the loaders family ([loaders.md](loaders.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **8 free + 32 Pro = 40 variants** across
two components that share one state machine:

- **`loader-mascot`**: small characters and animated marks (including the **Oxyn mark**) used as busy
  indicators at 16–48 px (4 free + 18 Pro).
- **`busy-glow`**: the composer border light and other rim / edge glows that show the **shared busy
  signal** on a container (4 free + 14 Pro).

Both read the composer region's `BusySignal` ([loaders.md](loaders.md) §2.3), so the glow, the
mascot, the status-strip dot and the thinking row never disagree; only one of them animates.

## 1. Purpose
**Mascots and marks** give an agent a face. A blob with two eye-dots that changes pose for Idle /
Listening / Thinking / Writing / Done / Error [R031]; a pill mascot whose pause-bars double as eyes
[R023]; Claude Code's block mascot that reacts on click and MonoCode's 16 px runner that patrols the
composer edge while a turn runs (research note 08); a 20-style animated-mark set where every mark
passes a 16–20 px legibility test (research note 09). Users want delight **without** losing the
information a plain spinner carries, so every mascot pose maps one-to-one to a loader phase.

**The busy glow** answers "is it working?" from the edge of the thing being worked on. BoardUI drives
the composer glow, the thinking row and the glass controls from one `waiting` flag [R095]; the input
shell spec already reserves a default **1 px border tint** and a Pro `composer-input/border-light`
that this spec draws ([composer-input-shell.md](composer-input-shell.md)).

Rules:
1. **Pose = phase.** Every mascot / mark has a designed pose (and still) for every `LoaderPhase`;
   fidgets and easter eggs never replace the phase pose.
2. **One animating indicator per region.** If the border light leads the composer region, the
   status-strip dot and any mascot there go still (and vice versa).
3. **Calm by default.** Idle mascots fidget at most once every 4 s and fall asleep (still) after 30 s
   idle (proposed); glows ease, never blink.
4. **Pixel art scales by integers only**; vector marks are tuned per size.
5. **The Oxyn mark is a preset of the mark engine**, not a special case: user SVG marks get the same
   states.

## 2. State machine
```
                      app ready                     user types
  Off ───────────────────────▶ Idle (rest pose; blink every 4–7 s) ──────────▶ Attend (looks at composer)
                                 ▲   └── 30 s idle ──▶ Asleep (still)            │ send
                                 │                                                ▼
                                 │                         Submitted (nod / hop · glow: band appears at send control)
                                 │                                                │ first event
                                 │                                                ▼
                                 │                         Thinking (thinking pose · glow: band travels, 1.0×)
                                 │                        tool ↓ ↑ tool ends
                                 │                         Working{activity} (busy pose · glow 1.25×)
                                 │                                                │ tokens
                                 │                                                ▼
                                 │                         Streaming (writing pose · glow 0.8×)
                                 │                                approval/question │
                                 │                                                ▼
                                 │                         Waiting (looks at user + "?" · glow: band stops, rim breathes amber)
                                 │
                                 │  active ── 5 s no event ─▶ Stalled(soft) (slows, droops · glow dims to 40 %, 0.5×)
                                 │                               └─ 45 s ─▶ Stalled(hard) (still, warn · rim static 1 px warn)
                                 │
                                 ├── 1.2 s ── Done (happy pose + hop · glow: one fast success lap, fade) ◀── completes
                                 ├── 400 ms ─ Cancelled (stops, shrug · glow fades) ◀── Stop
                                 └── dismiss ─ Failed (slump, ✕ eyes · rim danger 1 px static) ◀── error
  Overlays: Reduced (still pose per phase; glow = static tint) · Paused (hidden / offscreen)
            · Mirrored (still pose / static tint) · Reacting (tap reaction, ≤ 1 per 2 s, never during Waiting or Failed)
```

## 3. States table
| Phase | Mascot / mark shows | Busy glow shows | Host actions | Keyboard |
|---|---|---|---|---|
| `Off` | nothing | nothing | — | not focusable |
| `Idle` | rest pose, blink every 4–7 s (seeded random) | no glow (or `border-tint` off) | — | — |
| `Asleep` (30 s idle) | still rest pose, eyes closed | — | — | — |
| `Attend` (Composing) | looks toward the caret / composer | — | — | — |
| `Submitted` | single nod or 3 px hop | band fades in at the send control | Stop | — |
| `Thinking` | thinking loop (eyes up, dot circling the head [R031]) | band travels the rim clockwise at base speed; `border-tint` tints to accent | Stop | — |
| `Working{activity}` | busy loop (typing / running / searching prop) | band 1.25× | Stop | — |
| `Streaming` | writing loop (pen dash line) | band 0.8×, slightly brighter on token bursts (Pro `token-pulse`) | Stop | — |
| `Waiting` | faces the user, holds "?" or hourglass, amber accent | band stops; whole rim breathes `waiting` amber, 2.4 s | Review | — |
| `Stalled(soft)` | slows to 0.5×, droops 1 px | band dims to 40 % and slows to 0.5× | Stop | — |
| `Stalled(hard)` | still pose, warn accent | rim static 1 px `warn` | Retry / Stop | — |
| `Done` | happy pose + 3 px hop, then rest | one fast lap in `success` (600 ms), fades 400 ms | — | — |
| `Failed` | slump, ✕ eyes, `danger` accent, still | rim static 1 px `danger` until dismissed | Retry | — |
| `Cancelled` | stops, small shrug, back to rest | glow fades 200 ms | — | — |
| Reduced / Mirrored | designed still pose per phase | static 1 px tint in the phase colour | — | — |

## 4. Micro-interactions
**Mascots and marks**
- **Blink:** 120 ms close / open, every 4–7 s (seeded so instances don't blink in sync); never during
  Waiting (eyes on the user).
- **Attend:** eyes ease toward the caret's horizontal position over 300 ms (`ease.out`), clamped to
  ±3 px at 32 px; stops following when typing pauses for 2 s. (Cursor-tracking is a Pro variant only,
  after the Cline CLI robot whose head follows the mouse; research note 08.)
- **Pose changes** crossfade between pose loops over 200 ms on the shared clock; pixel mascots change
  on the next sprite frame.
- **Done:** happy pose + 3 px hop (`spring.soft`), hold 800 ms, back to rest. Failed: slump 2 px with
  `ease.firm`, no bounce.
- **Tap / click reaction** (opt-in): one short reaction (spin, wave) at most once per 2 s; ignored in
  Waiting, Stalled(hard) and Failed so the state stays readable (click reactions are a Claude Code
  idea, reference only).
- **Runner (Pro):** patrols the composer's top edge, turns at the ends, pauses while the user types or
  any popover is open, exits in a 560 ms arc on Done (MonoCode's runner: 160 px/s, jumps obstacles,
  coins, stun stars; research note 08, MIT).

**Busy glow**
- **Band start:** the band appears at the **send control** (trailing bottom corner) on Submitted and
  travels clockwise, so the light visibly comes from the action that started it.
- **Constant speed, not constant period.** Lap time = `clamp(perimeter / 400 px·s⁻¹, 2.4 s, 6 s)`
  (proposed), so a tall composer doesn't race and a small one doesn't crawl. Band length =
  `clamp(18 % of perimeter, 48 px, 160 px)` with a soft head and a long tail.
- **Composer growth:** when the composer grows upward, the band keeps its position as a fraction of
  the perimeter (no jump).
- **Focus ring wins:** the focus ring draws outside the border; the light draws on the border and is
  dimmed to 60 % while the shell has keyboard focus (proposed).
- **Waiting:** band slows to a stop over 400 ms, then the full rim breathes amber (2.4 s).
- **Done lap:** one 600 ms lap in `success` from wherever the band is, then a 400 ms fade.
- **Error:** no shake on the rim (the send control owns the shake); static danger tint.

## 5. Timing
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Mascot enter | 240 ms | `ease.out`, scale 0.85 → 1 + fade | |
| Blink | 120 ms | step (pixel) / `ease.in-out` (vector) | every 4–7 s |
| Fidget (Idle) | ≤ 1 per 4 s, 400–800 ms each | `spring.soft` | none after 30 s idle |
| Attend eye follow | 300 ms | `ease.out` | ±3 px at 32 px |
| Pose crossfade | 200 ms | `ease.in-out` | |
| Thinking pose loop | 1600–2400 ms (`loop.calm`) | per pose | |
| Working pose loop | 800–1200 ms | per pose | |
| Pixel sprite rate | 7.5 or 15 fps | step | MonoCode's Half / Low rates (research note 08) |
| Submitted nod / hop | 240 ms | `spring.snap` | 3 px |
| Done happy + hop / hold | 320 ms / 800 ms | `spring.soft` | |
| Failed slump | 240 ms | `ease.firm` | |
| Tap reaction | 400–700 ms | per reaction | ≤ 1 per 2 s |
| Glow band fade-in (Submitted) | 200 ms | `ease.out` | at the send control |
| Glow lap | `clamp(perimeter / 400, 2.4, 6)` s | linear | Thinking 1.0×, Working 1.25×, Streaming 0.8× |
| Glow tempo change | 400 ms | `ease.in-out` on speed | phase-continuous |
| `border-tint` colour change | 300 ms | OKLab linear | 1 px border |
| `breathing-rim` | 2400 ms loop | sine | blur 0 ↔ 6 px, opacity 0.3 ↔ 0.7 |
| Waiting: band stop + amber breathe | 400 ms + 2400 ms loop | `ease.out` / sine | |
| Stall soft dim / hard settle | 2000 ms / 400 ms | linear / `ease.out` | |
| Done lap + fade | 600 ms + 400 ms | `ease.in-out` / `ease.in` | |
| Cancelled fade | 200 ms | `ease.in` | |
| **Reduced motion** | mascots: still pose per phase, no blink, fidget, hop or runner; glow: static 1 px tint in the phase colour, no band, no breathe; phase changes 0–80 ms crossfade | — | opt-in SlowFade on the tint only |

Budgets: vector mascot ≤ 0.2 ms CPU per frame; pixel mascot redraws only on sprite frames;
`border-light` ≤ 0.2 ms (T2); blurred glows (`breathing-rim`, aurora) count as T4 and fall back to
`border-light` or `border-tint`.

## 6. Variants

### Free (8)
| Name | Idea |
|---|---|
| `loader-mascot/oxyn-mark` | The Oxyn mark with a pose per phase (breathe, spin-up, settle, ✓, ✕); legible at 16–20 px |
| `loader-mascot/pill-eyes` | Rounded pill whose two pause-bars are eyes that blink and look up while thinking (idea from [R023]) |
| `loader-mascot/blob-eyes` | Soft blob with two eye-dots; a dot circles the head while thinking, ^^ eyes on Done (idea from [R031]) |
| `loader-mascot/pixel-bot` | 12 × 12 pixel robot at 7.5 fps with typing, waiting and sleeping sprites |
| `busy-glow/border-tint` | Default: the 1 px composer border tints to accent while busy, amber while waiting |
| `busy-glow/border-light` | A light band travels the composer rim at constant speed from the send control |
| `busy-glow/breathing-rim` | Soft outer glow that breathes while busy (falls back to `border-tint`) |
| `busy-glow/conic-rim` | Rotating conic-gradient border, brand duotone |

### Pro (32, in 17 groups)
| Group (count) | Names | One-line idea |
|---|---|---|
| Runner (1) | `loader-mascot/runner` | 16 px pixel mascot patrolling the composer edge during a run, hops obstacles, exits on Done (idea from MonoCode's Composer Runner, MIT) |
| Critters (4) | `loader-mascot/fox`, `-cat`, `-owl`, `-bug` | Pixel critters with phase sprites: owl blinks slowly while thinking, fox digs while searching, etc. |
| Faces (4) | `loader-mascot/face-round`, `-square`, `-visor`, `-pixel` | Minimal agent faces (eyes + mouth) for avatars; expression per phase (idea: Libraries.dev Bot avatars, licence not established) |
| Eyes (2) | `loader-mascot/wandering-eyes`, `-cursor-eyes` | Two eyes that wander while thinking (idea from Loading UI); eyes that follow the pointer (idea from the Cline CLI robot) |
| Mark engine (3) | `loader-mascot/mark-draw-on`, `-breathe`, `-hue-settle` | Any user SVG mark: stroke draws on then fills; breathes; hue sweeps then settles to grey on Done (research note 09 techniques) |
| Mark morphs (2) | `loader-mascot/mark-nodes`, `-particle` | Mark collapses to a dot then a 3- or 4-node graph by activity; mark dissolves into a dot field and re-forms (pairs with send control `particle`) |
| Orb-eyes (1) | `loader-mascot/orb-eyes` | A thinking orb with two eye-dots: orb motion + character |
| Plant (1) | `loader-mascot/sprout` | A sprout grows a leaf per tool call and blooms on Done (growth-as-reveal idea from [R004]) |
| Border beam (3) | `busy-glow/beam-single`, `-twin`, `-reverse` | Comet beam on the rim; two beams opposite; reverse direction for steer / undo moments (Libraries.dev `border-beam` package, MIT) |
| Shader rims (2) | `busy-glow/aurora-rim`, `-ember-rim` | Aurora light flowing along the rim; warm ember rim for long runs (T4, fall back to `border-light`) |
| Edges (2) | `busy-glow/window-edge`, `-notch-glow` | Whole-window edge glow with rim, haze and wake (idea from ittybitty Edge Glow, MIT per note 02); desktop notch / island glow |
| Region sweep (1) | `busy-glow/region-sweep` | A glow passes over the region the agent is changing, content settles after it (idea from [R034]) |
| Token pulse (1) | `busy-glow/token-pulse` | Rim brightness follows the smoothed token rate while streaming, capped at 2 pulses/s |
| Dither rim (1) | `busy-glow/dither-rim` | Ordered-dither band crawling the rim, pixel aesthetic |
| Arrival (1) | `busy-glow/shockwave-done` | On Done, a single shockwave ring leaves the composer (idea from [R039]) |
| Multi-agent (2) | `busy-glow/lanes-rim`, `-handoff-rim` | Rim split into coloured segments, one per active agent lane (idea from [R061]); a light travels from one agent's card to another on handoff |
| Dotted rim (1) | `busy-glow/dotted-rim` | Marching dots around the rim (1 px per 60 ms, the input shell's drag-over rhythm), calm alternative to a beam |

### The Oxyn mark
- The vector logo and mark states are a **V2.3 design-language deliverable** ("vector logo + mark
  states", "logo legible at 16–20 px"; [../roadmap.md](../roadmap.md)). Until then
  `loader-mascot/oxyn-mark` uses a placeholder and this spec fixes only its **behaviour**.
- Family L requires 10 mark states (idle, thinking, working, tool, waiting, streaming, success, error,
  stalled, reduced-motion) ([../component-taxonomy.md](../component-taxonomy.md)); they map to the
  phases in §3 (tool = `Working{activity}`, success = `Done`).
- The send control's Pro `particle` variant describes a dot field that collapses into ■ "matching the
  Oxyn mark", so we **infer** the mark is dot-based; `mark-particle` assumes that. To confirm in V2.3.

## 7. Edge cases
- **Tiny sizes.** Mascots need ≥ 16 px; at 12 px `oxyn-mark` and marks use a single-shape still +
  `loader/dots-pulse`. Pixel mascots pick grid and integer scale per size: 16 px = 16 × 16 at 1× (or
  8 × 8 at 2×), 24 px = 12 × 12 at 2×, 32 px = 16 × 16 at 2×, 48 px = 16 × 16 at 3× or 12 × 12 at 4×;
  no fractional scaling, no subpixel positions.
- **HiDPI.** Pixel art uses nearest-neighbour sampling at device scale; vector marks snap to half
  pixels at 16–20 px.
- **Composer resize / grow.** Band position kept as a fraction of the perimeter; perimeter recomputed
  per layout, including large corner radii (pill shells): the path is a rounded rect parameterised by
  arc length so speed stays constant on corners.
- **Composer hidden / collapsed / scrolled away.** The glow pauses; the region lead passes to the
  next visible claimant (e.g. the transcript-tail loader).
- **Popovers / IME window over the rim.** The band passes under popovers (drawn on the border layer)
  and the runner hides while any popover is open.
- **Dense IDE panel.** `composer/ide-panel` defaults to `border-tint` only; beams are off below 320 px
  wide (proposed).
- **Light / dark.** Glows use normal blending on light themes (additive glows vanish on white) with a
  deeper palette step; mascots have light and dark outline tokens.
- **RTL.** Band starts at the send control's mirrored position (bottom-left) and travels
  counter-clockwise; mascots face inline-start; text props (e.g. "?") stay unmirrored.
- **Very fast agent.** `show_delay` 300 ms: the glow only appears if the run lasts; a mascot just nods.
- **Very slow agent.** After 10 min the band slows to the 6 s lap cap and mascots switch to their calm
  thinking loop (proposed).
- **Multiple composers / windows.** Each composer has its own region and band; windows in the
  background run at the unfocused rate or pause when hidden.
- **Touch.** Mascot tap reaction only if the host enables it; never a hidden control. The glow is never
  interactive.
- **Distraction.** Users can turn mascots off per app (`MascotPolicy::Off`) and the runner is always
  opt-in.

## 8. Accessibility
- Mascots, marks and glows are decorative: `aria-hidden="true"` / not exposed in GPUI. When a mascot
  is the agent's avatar with no visible name, the host labels it ("Agent, waiting for your approval").
- `aria-busy="true"` goes on the content being produced, set by the host.
- No live-region output; phase announcements come from the composer's single polite live region
  (assertive for waiting and failed) per [loaders.md](loaders.md) §8.
- **Flashing:** glows ease and never blink; the Done lap runs once; token-pulse is capped at 2 per
  second; tap reactions are rate-limited; nothing changes the luminance of a large area faster than 3
  per second. Window-edge glows stay ≤ 30 % opacity (proposed) so they never read as a screen flash.
- **`prefers-reduced-motion`:** still pose per phase, static tint for glows (§5); the runner never
  appears.
- **Contrast:** mascot outline ≥ 3:1 against the background; `border-tint` meets 3:1 against the
  composer fill in busy, waiting, warn and danger tones; waiting/failed also change pose and host text,
  never colour alone.
- **Attention:** idle fidgets ≤ 1 per 4 s and stop after 30 s, so a mascot never competes with reading.
- Not focusable.

## 9. Props sketch
```rust
pub struct LoaderMascot {
    pub agent: AgentState,                 // LoaderPhase::derive(&agent, stall, tool)
    pub variant: MascotVariant,            // OxynMark | PillEyes | BlobEyes | PixelBot | Pro(MascotPro)
    pub mark: Option<MarkSource>,          // user SVG + optional morph targets for mark-engine variants
    pub size: LoaderSize,                  // Px16..Px48 (Px12 → still + dots-pulse)
    pub tone: LoaderTone,
    pub region: BusyRegion,
    pub priority: BusyPriority,
    pub policy: MascotPolicy,              // Off | Calm (default: sleeps after 30 s) | Lively
    pub attend_caret: bool,                // Composing pose follows the caret (default true)
    pub reactions: bool,                   // tap reactions (default false)
    pub sprite_fps: SpriteRate,            // Low 7.5 | Half 15
    pub labels: LoaderLabels,
    pub motion: MotionLevel,
}

pub struct BusyGlow {
    pub signal: BusySignal,                // composer region's signal (phase + lead) from oxyn-core
    pub variant: GlowVariant,              // BorderTint (default) | BorderLight | BreathingRim | ConicRim | Pro(GlowPro)
    pub target: GlowTarget,                // ComposerShell (default) | Container(id) | Window | Notch
    pub start_at: GlowAnchor,              // SendControl (default) | TopLeading | Custom(f32 along perimeter)
    pub speed_px_s: f32,                   // 400 (proposed); lap clamped 2.4–6 s
    pub band: BandShape,                   // length 18 % (48–160 px), head / tail falloff
    pub dim_when_focused: f32,             // 0.6
    pub min_width_px: f32,                 // beams off below this (320 in ide-panel)
    pub lanes: Vec<LaneColor>,             // lanes-rim (Pro)
    pub tier: TierPolicy,                  // blurred / shader rims fall back to BorderLight → BorderTint
    pub motion: MotionLevel,
}
```

## 10. Related components and references
**Related:** input shell busy tint and `composer-input/border-light` (geometry owner;
[composer-input-shell.md](composer-input-shell.md)) · status strip dot (goes still when the glow
leads; [composer-status-strip.md](composer-status-strip.md)) · send control `particle` / `border-beam`
/ `edge-glow` ([composer-send-control.md](composer-send-control.md)) · effort control
`border-beam`, mode switch `border-beam`, permission selector `heat-border` (they reuse `busy-glow`
drawing; [composer-effort-control.md](composer-effort-control.md),
[composer-mode-switch.md](composer-mode-switch.md),
[composer-permission-selector.md](composer-permission-selector.md)) · [thinking-orb.md](thinking-orb.md)
(`orb-eyes`) · family D container loaders (border beams on tiles) · family L animated marks.

| Reference | What we take | Licence note |
|---|---|---|
| BoardUI [R095] | one `waiting` flag drives composer glow, thinking row, controls | Free tier MIT; Pro study only |
| MonoCode Composer Runner + pulse clock (research note 08) | runner on the composer edge; 30 / 15 / 7.5 fps leased clock | MIT — port with attribution |
| Cline CLI robot (research note 08) | head follows the cursor, 12 ms interpolation | Apache-2.0 — port with attribution |
| Claude Code Clawd (research note 08) | click reactions on a block mascot | Proprietary — reference only |
| eliakuratli mascot [R031] | 6-state blob mascot with captions | Study only |
| dannyinsf_ pill mascot [R023] | pause-bars as eyes | Study only |
| Animated-mark set and techniques (research note 09) | 16–20 px legibility test; draw-on, hue-settle, node-graph marks; logo-agnostic engine + brand presets | Techniques are ideas; product marks are proprietary — study only |
| Libraries.dev `border-beam` [R100] (research note 16) | beam on a border | MIT — port with attribution |
| Libraries.dev Bot avatars (research note 16) | animated faces for agents | Not in the MIT package list — study only |
| Loading UI wandering-eyes [R091] | wandering eyes | Open source; confirm per-item licence |
| Magic UI Border Beam, UploadKit UploadBeam (research note 02) | beam along `offset-path`; beam bound to a lifecycle (spin → green / red → fade) | Licences not checked — study only |
| ittybitty Edge Glow (research note 02) | rim / haze / wake, offscreen pause, reduced-motion still | MIT per note 02 — confirm before porting |
| jbedln glimm sweep [R034] | "agent is writing here" region sweep | Study only |
| Kubai087 shockwave [R039] | done shockwave | Study only |
| FinnoTaylor pixel tree [R004] | growth-as-reveal for `sprout` | Study only |
| flohoeller lanes [R061] | per-agent lanes for `lanes-rim` | Study only |
