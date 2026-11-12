**Status:** v1 — free set implemented (plan #16, 2026-11-12): `oxyn-core::loader::FREE_GLYPH_TABLES`,
`oxyn_gpui::GlyphLoaderView`, `oxyn_dioxus::GlyphLoader`. Pro tables are not built yet.

# Spec: glyph loaders (terminal / text frame loaders)

Part of the loaders family ([loaders.md](loaders.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **14 free + 30 Pro = 44 variants**, all
drawn by one renderer from **frame tables** (render tier T1), all driven by the shared
`LoaderPhase` derived from `AgentState`.

## 1. Purpose
Glyph loaders are the spinner that sits **left of a status line** in a terminal or an IDE panel:
`⠹ Thinking · 12s`. They are the cheapest loader we ship (text only, a frame every 80–600 ms), they
work on every surface including `Terminal`, and they fit inside running text at 12–16 px. Coding
agents use them everywhere: braille at 80 ms in OpenCode, Kilo and MonoCode, braille at 100 ms in
Codex and Zed, a mirrored star glyph at 120 ms in Claude Code (research note 08).

What the user needs from them: **alive vs stuck vs waiting on me**, at a glance, without the line
jumping. So the hard parts are not the frames; they are **width stability**, **monospace
alignment**, **font coverage**, and a clear **per-phase change of motion**.

Rules:
1. A glyph loader **never changes the width of its line**. Every frame of a set has the same cell
   count; the box is reserved even when the frame is blank.
2. **One renderer, many tables.** A variant is a frame table plus per-phase overrides; no variant
   has code of its own (cheap to add, cheap to review).
3. Frame index comes from the shared clock (`floor(phase × n)`), so every glyph loader of the same
   variant on screen shows the same frame.
4. Each table names a **fallback table** for fonts or terminals that can't draw it, ending at ASCII
   `|/-\`.
5. Frames copied from proprietary products (Claude Code) are **not** shipped; our star is our own.

## 2. State machine
```
               send                  first event
  Hidden ──────────────▶ Submitted ─────────────▶ Thinking  (base table, 1.0×)
    ▲        (one bright `·`, 400 ms)                │  ▲
    │                                    tool starts │  │ tool ends
    │                                                ▼  │
    │                                   Working{activity} (activity table or 1.25× frame rate)
    │                                                │ first token
    │                                                ▼
    │                                   Streaming (0.6×, or hand off to caret)
    │                              approval/question │
    │                                                ▼
    │                                   Waiting (waiting table: sand / hourglass, amber)
    │
    │   any active ── 5 s no event ──▶ Stalled(soft): frame ms × 2, colour → warn over 2 s
    │                                       └── 45 s ──▶ Stalled(hard): hold still glyph `◌` in warn
    │                                  (event arrives → previous phase, colour snaps back 200 ms)
    │
    │◀── clear ── Done: `✓` 600 ms ◀───── completes
    │◀── clear ── Cancelled: freeze, dim 40 %, clear after 160 ms ◀── Stop
    │◀── dismiss ─ Failed: `✕` (ASCII `x`) in danger, static ◀── error
  Overlays: Reduced (still glyph per phase) · Paused (offscreen) · Mirrored (still glyph) · Fallback(table)
```

## 3. States table
Glyph loaders are not interactive; actions belong to the host row (status strip, tool row).

| Phase | Shows | Host primary / secondary | Keyboard |
|---|---|---|---|
| `Hidden` | nothing; box reserved only if `reserve_when_hidden` (default false) | — | not focusable |
| `Submitted` | first frame (or `·`) at full brightness, dims to 60 % over 400 ms; no cycling | host: Stop | — |
| `Thinking` | base table at its frame ms | host: Stop; click row → live step | — |
| `Working{activity}` | activity table if the variant has one (e.g. `glyph-search`), else base table at frame ms ÷ 1.25 | same | — |
| `Streaming` | base table at frame ms ÷ 0.6 (slower), or hidden if `hand_off_to_caret` | same | — |
| `Waiting` | waiting table (`glyph-sand` by default) in `color.loader.waiting` | host: Review | — |
| `Stalled(soft)` | frame ms × 2, colour ramps to `warn` over 2 s | host: Stop | — |
| `Stalled(hard)` | still `◌` in `warn` (no cycling) | host: Retry / Stop | — |
| `Done` | `✓` in `success` for 600 ms, then cleared (or replaced by host's past-tense line) | — | — |
| `Failed` | `✕` in `danger`, static | host: Retry / details | — |
| `Cancelled` | frozen frame at 40 %, cleared after 160 ms | — | — |
| Reduced motion | still glyph per phase: working `◌`, waiting `⧗`, done `✓`, failed `✕` | — | — |
| Mirrored (not region lead) | same still glyphs as reduced motion | — | — |

## 4. Micro-interactions
- **Enter.** A glyph can't scale cleanly, so enter is an opacity fade 0 → 1 over 120 ms
  (`dur.fast`), starting on the frame the clock dictates (never frame 0 by default).
- **Submitted pulse.** One bright frame dimming over 400 ms; the loop starts only when `Thinking`
  arrives, so a quick answer shows no spinner at all.
- **Phase change.** Tables swap on the next frame boundary (never mid-frame); colour lerps in
  OKLab over 400 ms. Thinking → Waiting changes **both** table and colour, so human-blocked reads
  differently from generating (Zed swaps Dots for Sand for this; research note 08).
- **Multi-cell trails.** Per-cell brightness levels `0.3 + 0.7 × level` (OpenCode's ramp, research
  note 08) for scanners, waves and rain; implemented as per-cell alpha on one colour.
- **Holds.** Tables may hold a frame for several ticks (`holds`) to give a beat: fill/drain holds at
  the top and bottom, double-beat rests, the scanner pauses at each end.
- **Stall.** Frame ms doubles over the 2 s ramp (eased via the phase integral, no stutter); colour
  ramps toward `warn`. At `stall_hard` the glyph stops on `◌`. The first event snaps back in 200 ms.
- **Done.** The current frame is replaced by `✓` with a 120 ms fade; the host line rolls to past
  tense ("Worked for 42s", family C). Failed: `✕` + the host shakes, not the glyph.
- **Hover** (only when the host row is hoverable): tooltip "Thinking · 42 s"; the glyph doesn't change.

## 5. Timing

### 5.1 Free frame tables
| Variant | Frames (one cell unless noted) | Frame ms | Loop | Notes |
|---|---|---|---|---|
| `loader/glyph-braille` | `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` | 80 | 800 ms | the common braille set (cli-spinners, OpenCode at 80 ms; Codex, Zed at 100 ms) |
| `loader/glyph-sand` | `⠛ ⠫ ⡋ ⡍ ⣉ ⣅ ⣤ ⣤ ⣄ ⣀ ⡀ ⠀` (own sequence) | 150, holds `[1,1,1,1,1,1,2,1,1,1,1,1]` | 1950 ms | grains fall top → bottom, then drain; **default waiting table**; authored for Oxyn, not Zed's 35-frame Sand |
| `loader/glyph-block` | 3 cells: `░░░ ▒░░ ▓▒░ █▓▒ ▓█▓ ▒▓█ ░▒▓ ░░▒` | 100 | 800 ms | shade wave; width-unsafe on ambiguous-wide terminals (§7) |
| `loader/glyph-bar` | `▁ ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃ ▂` | 80, holds `[2,2,1,1,1,1,1,2,1,1,1,1,1,2]` | 1440 ms | fill / drain (OpenCode table, MIT) |
| `loader/glyph-dots` | 3 cells: `·   ` `··  ` `··· ` ` ·· ` `  · ` `    ` (padded) | 180 | 1080 ms | ellipsis that travels, not a typing bounce |
| `loader/glyph-line` | `\| / - \` (ASCII) | 100 | 400 ms | universal fallback; terminal minimum |
| `loader/glyph-arc` | `◜ ◠ ◝ ◞ ◡ ◟` | 100 | 600 ms | arc travelling round a circle |
| `loader/glyph-quadrant` | `◰ ◳ ◲ ◱` | 160 | 640 ms | corner orbit (OpenCode, MIT) |
| `loader/glyph-bloom` | `▫ ▪ ▣ ■ ▣ ▪` | 120 | 720 ms | inset bloom (OpenCode, MIT) |
| `loader/glyph-half` | `◐ ◓ ◑ ◒` | 150 | 600 ms | half-disc rotation (proposed timing) |
| `loader/glyph-bullet` | `•` ↔ `◦` (dim) | 600 | 1200 ms | Codex's exec blink; **calmest** option; ≤ 0.84 Hz |
| `loader/glyph-scanner` | 8 cells, head `■`, trail `▪ ▪ ·` at levels 1/0.6/0.35/0.15, inactive `·` at 0.1 | 50, holds 6 at each end | ≈ 1.3 s there and back | own values; idea from OpenCode's 8-wide scanner (holds 30 / 9 frames there) |
| `loader/glyph-spark` | `· ⋆ ✧ ✦ ✧ ⋆` (own set, ping-pong) | 110 | 660 ms | Oxyn's breathing star; the mirrored-breath *idea* is Claude Code's, the glyphs and timing are ours |
| `loader/glyph-pulse` | `●` brightness `0.3 + 0.7 × (cos + 1) / 2` | brightness at ≤ 20 fps | 1600 ms | seed breathe (idea from OpenCode's one-cell gallery) |

### 5.2 Transition timing
| Transition | Duration | Easing | Notes |
|---|---|---|---|
| Enter fade | 120 ms (`dur.fast`) | `ease.out` | starts on the clock's frame |
| Submitted dim | 400 ms × 1 | `ease.out` | no cycling |
| Table swap (phase change) | next frame boundary | step | never mid-frame |
| Colour lerp (phase change) | 400 ms | OKLab linear | |
| Thinking → Working tempo | 400 ms | `ease.in-out` on speed | frame ms ÷ 1.25 |
| Streaming tempo | 400 ms | `ease.in-out` | frame ms ÷ 0.6 |
| Stall soft (frame ms × 2 + colour) | 2000 ms | linear | |
| Stall hard → still `◌` | next frame boundary | step | |
| Recovery snap | 200 ms | `ease.out` | |
| Done `✓` fade-in / hold / clear | 120 / 600 / 160 ms | `ease.out` / — / `ease.in` | |
| Cancelled dim → clear | 160 ms | `ease.in` | |
| **Reduced motion** | still glyph per phase; swaps are 0–80 ms crossfades; no cycling, no brightness breathe | — | opt-in SlowFade: 2 s opacity 1 ↔ 0.6 |

Frame rate rule: glyph loaders repaint only at frame boundaries (12.5 Hz for 80 ms tables, 1.7 Hz
for `glyph-bullet`). The `glyph-pulse` brightness loop is capped at 20 fps (proposed).

## 6. Variants

### Free (14)
| Name | Idea |
|---|---|
| `loader/glyph-braille` | Classic braille spinner, the default for terminal and IDE status lines |
| `loader/glyph-sand` | Hourglass of braille grains; default `Waiting` table for every glyph loader |
| `loader/glyph-block` | Three-cell `░▒▓█` shade wave |
| `loader/glyph-bar` | One-cell fill and drain `▁…█…▁` with holds |
| `loader/glyph-dots` | Travelling three-dot ellipsis |
| `loader/glyph-line` | ASCII `\|/-\`; the fallback every table ends in |
| `loader/glyph-arc` | Arc segment circling `◜◠◝◞◡◟` |
| `loader/glyph-quadrant` | Corner orbit `◰◳◲◱` |
| `loader/glyph-bloom` | Square blooming and shrinking `▫▪▣■` |
| `loader/glyph-half` | Half-disc rotation `◐◓◑◒` |
| `loader/glyph-bullet` | Slow `•`/`◦` blink for calm, low-attention rows |
| `loader/glyph-scanner` | Eight-cell scanner with a fading trail |
| `loader/glyph-spark` | Oxyn's own mirrored star breath `·⋆✧✦` |
| `loader/glyph-pulse` | Single `●` breathing in brightness |

### Pro (30, in 14 groups)
| Group (count) | Names | One-line idea |
|---|---|---|
| Braille pack (6) | `glyph-braille-dots8`, `-orbit`, `-trio`, `-bounce`, `-wave`, `-rain` | Denser cells: `⣾⣽⣻⢿⡿⣟⣯⣷` full-cell rotation; one dot circling `⠁⠈⠐⠠⢀⡀⠄⠂`; three-dot comet `⠇⠋⠙⠸⢰⣠⣄⡆`; rising/falling pair `⠉⠒⠤⣀⠤⠒`; 4-cell wave; 3-cell seeded rain |
| Block pack (4) | `glyph-block-wave`, `-conveyor`, `-snake`, `-infinity` | 6-cell travelling shade wave; `▰▱` belt; quadrant snake `▖▘▝▗` round a 2×2; 8-cell ∞ track (ideas from Loading UI's block loaders) |
| Choreography (2) | `glyph-turn`, `glyph-double-beat` | Slow half-turn there and back, a rest, then a quick flourish (`⋮⋰⋯⋱`, idea from Warp; our own timing: 220 ms steps then 70 ms flourish); `▪■▪■▪` with a long rest (OpenCode, MIT) |
| Semantic (3) | `glyph-search`, `glyph-write`, `glyph-run` | Activity tables: braille magnifier sweeping 2 cells; pen fill `▏▎▍▌▋▊▉█` then clear; chevron march `▸▹▹ ▹▸▹ ▹▹▸` |
| Matrix / ASCII (3) | `glyph-rain`, `glyph-density-wave`, `glyph-ascii-orb` | 5-cell falling density ramp; ` .:-=+*#%@` wave across 6 cells; 5×3-cell rotating sphere shaded with a density ramp (idea from [R011]) |
| Braille raster (2) | `glyph-mark-raster`, `glyph-blossom` | The Oxyn mark lit and rasterised into braille dots (idea from Codex's braille logo, research note 08); dots bloom out from the centre and collapse to `·` on Done |
| Heartbeat (1) | `glyph-heartbeat` | `●` with two Gaussian beats then rest (lub-dub) |
| Ember (1) | `glyph-ember` | `·∙•●` with 10 % attack and cubic decay, like a coal |
| Seasonal (1) | `glyph-moon` | Opt-in moon-phase cycle (idea: seasonal moon spinner, research note 08) |
| Effort sparks (1) | `glyph-effort-sparks` | `· ✦ ✧` trail whose length follows the effort level (idea from Codex's effort sparks) |
| Handoff (1) | `glyph-handoff` | `●··` → `·●·` → `··●` with eased holds, for sub-agent handoffs |
| Pixel text (3) | `glyph-pixel-radar`, `-road`, `-wave` | 2-cell half-block (`▀▄█`) 4×4 pixel grids: sweeping radar, scrolling road, wave |
| Determinate (1) | `glyph-progress` | `○◔◑◕●` when a percentage is known (jobs, uploads) |
| Hourglass (1) | `glyph-hourglass` | `⧗` / `⧖` flip every 1 s; alternative waiting table |

## 7. Edge cases
- **Width stability (the main rule).** At build, `xtask build-registry` measures every frame with
  `unicode-width` and rejects tables whose frames differ in cell count, contain emoji presentation
  selectors, ZWJ sequences or combining marks. Blank cells are real spaces (or `⠀` U+2800 in braille
  tables) and are never trimmed.
- **East Asian Ambiguous width.** Several block and geometric glyphs (e.g. `░▒▓█`) are ambiguous width
  and render two cells wide in terminals set to ambiguous = wide (common in CJK locales). Tables carry
  `width_safe: bool`; on a terminal with ambiguous-wide (configurable on our `Terminal` surface,
  since it can't always be detected), width-unsafe tables fall back to `glyph-braille` (braille
  patterns are not ambiguous width).
- **Monospace alignment in the GUI.** The loader box is `cells × advance("0")` of the mono font at the
  loader size, glyph centred horizontally, aligned to the label's **baseline** (braille sits high in
  most fonts: optical offset +0.05 em, proposed, per font in tokens). If a glyph's advance exceeds its
  cell (fallback font), it is scaled down to fit rather than pushing the label.
- **Missing glyphs / font fallback.** At mount the renderer checks the font chain covers every frame;
  if not, it uses the table's `fallback` (chain ends at `glyph-line`). In a terminal we can't check
  coverage, so `Terminal` surface defaults to braille or ASCII tables unless the app opts in.
- **Long labels / overflow.** The label truncates with an ellipsis; the loader box never shrinks.
  Multi-cell tables (scanner, waves) collapse to their one-cell fallback when the row is narrower
  than 24 cells (proposed).
- **RTL.** The glyph sits at the inline start (right). Rotating tables are unchanged; directional
  tables (scanner, dots, waves, handoff, chevrons) play mirrored frames generated at build.
- **Tiny sizes (12 px).** One-cell tables only; brightness-only tables (`glyph-pulse`) keep a floor of
  0.3 so they never vanish.
- **Dense IDE panels.** Twenty running tool rows each want a spinner: only rows that lead their
  region animate (all are their own region, but list cap 8 applies in sidebars); the rest show `◌`.
- **Non-truecolor terminals.** No colour ramps: stalled uses the terminal's yellow/red palette index
  and the text label; brightness trails become two levels (bold / dim).
- **Slow terminals / SSH.** Repaint writes only the loader cells (cursor-addressed), never the whole
  line; with high latency (configurable) frame ms is floored at 120 ms (proposed).
- **Very fast agent.** `show_delay` 300 ms (loaders.md) hides the loader on quick turns.
- **Very slow agent.** After 10 min in one phase, frame ms rises to the table's calm rate (×1.5,
  proposed); the elapsed timer remains the liveness cue.
- **Copy / select.** Loader glyphs are excluded from text selection and from copied transcript text.
- **Screen readers in terminals.** Terminal screen readers read cell changes; the `Terminal`
  surface can disable cycling (`a11y_static_terminal`) so only phase changes write to the screen.

## 8. Accessibility
- The glyph is decorative: `aria-hidden="true"` in Dioxus, not exposed as an accessible node in GPUI,
  excluded from selection. The host row carries the label ("Thinking, 12 seconds").
- `aria-busy="true"` is set on the content being produced (message, tool card) by the host, removed
  on Done / Failed / Cancelled.
- No live-region output from the loader. Phase announcements come from the composer's single polite
  live region (loaders.md §8); waiting and failed are assertive there.
- **Flashing:** the fastest table (`glyph-scanner`, 50 ms) moves a small glyph; it does not flash an
  area. Whole-glyph on/off blinks are limited to ≤ 3 per second (`glyph-bullet` is 0.84 Hz;
  `glyph-heartbeat` beats twice per 1.6 s cycle).
- **Reduced motion / `prefers-reduced-motion`:** still glyph per phase (§3); cycling stops.
- **Contrast:** glyph colour ≥ 3:1 against the row background in every phase including the stall
  ramp midpoint; `◌` dim state still ≥ 3:1 (dim via weight, not alpha below 0.6).
- **Not colour alone:** waiting changes the glyph (sand), stalled hard changes to `◌` plus text,
  done/failed change to `✓`/`✕`.

## 9. Props sketch
```rust
pub struct GlyphLoader {
    pub agent: AgentState,                 // LoaderPhase::derive(&agent, stall, tool)
    pub table: GlyphTableId,               // e.g. GlyphTableId::Braille, ::Sand, ::Spark …
    pub waiting_table: GlyphTableId,       // default ::Sand
    pub activity_tables: ActivityTables,   // optional per-Activity tables (Search → ::Search …)
    pub size: LoaderSize,                  // Px12..Px48 (font size of the cell)
    pub tone: LoaderTone,
    pub region: BusyRegion,
    pub priority: BusyPriority,
    pub speed: f32,                        // multiplies the table's tempo; clamp 0.5..=2.0
    pub hand_off_to_caret: bool,           // hide in Streaming when the stream shows a caret
    pub reserve_when_hidden: bool,         // keep the box so the label never shifts
    pub surface: Surface,                  // Terminal limits tables to width-safe ones
    pub labels: LoaderLabels,
    pub motion: MotionLevel,
}

/// Static data in oxyn-core (frames JSON, validated at build).
pub struct GlyphTable {
    pub id: &'static str,
    pub frames: &'static [&'static str],   // equal cell counts, enforced
    pub frame_ms: u16,
    pub holds: Option<&'static [u8]>,      // ticks per frame
    pub mirror: bool,                      // ping-pong
    pub cells: u8,
    pub levels: Option<&'static [f32]>,    // per-cell brightness for trails
    pub still: StillGlyphs,                // working '◌', waiting '⧗', done '✓', failed '✕'
    pub width_safe: bool,                  // false if any glyph is East Asian Ambiguous
    pub fallback: Option<&'static str>,    // ends at "line"
    pub rtl_mirror: bool,                  // generate mirrored frames for directional tables
}
```

## 10. Related components and references
**Related:** status strip dot and verb ([composer-status-strip.md](composer-status-strip.md)) ·
`composer/terminal` layout ([composer.md](composer.md)) · send control `glyph` and Pro
`ascii-spinner` ([composer-send-control.md](composer-send-control.md)) · family C shimmer and verb
cycler (sits right of the glyph) · family F tool rows · [loader-vector.md](loader-vector.md) (SVG
versions of the same motions for GUI).

| Reference | What we take | Licence note |
|---|---|---|
| cli-spinners (research note 02) | the common braille / arc frame sets and the frames + interval data model | MIT — port tables with attribution |
| OpenCode one-cell gallery, scanner, brightness ramp (research note 08) | bloom, quadrant, fill/drain tables with holds; `0.3 + 0.7 × level` trails; double-beat | MIT — port with attribution |
| Kilo, MonoCode (research note 08) | braille at 80 ms | MIT — port with attribution |
| Codex CLI motion (research note 08) | braille 100 ms, `◌` reduced-motion still, `•`/`◦` 600 ms, effort sparks `· ✦ ✧`, braille logo raster idea | MIT per licensing.md (Apache-2.0 per note 08) — portable with attribution |
| Zed SpinnerLabel (research note 08) | Dots for generating vs Sand for awaiting (idea only; our sand is our own sequence) | Mixed licence — study only |
| Warp warping indicator (research note 08) | turn-and-flourish choreography (idea; our own timings) | AGPL — study only |
| Claude Code spinner (research notes 08, 09) | mirrored breathing-star idea; stall-to-red idea | Proprietary — reference only; **its frame set is not used** |
| Loading UI [R091] | block-glyph loader ideas (conveyor, square snake, infinity track) | Open source; confirm per-item licence before porting |
| unicode-animations (research note 02) | braille frame data source to check against | Licence not checked — study only |
| praveenisomer ASCII globe [R011] | density-ramp sphere idea for `glyph-ascii-orb` | Study only |
