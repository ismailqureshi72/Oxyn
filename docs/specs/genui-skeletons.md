**Status:** draft v0.1 (2026-11-12); base skeleton shapes and sweeps implemented in plan #26.

# Spec: generative UI skeletons (block × sweep matrix)

Part of family I ([genui.md](genui.md)); follows the 10-point standard in [README.md](README.md) at the
depth of [composer-send-control.md](composer-send-control.md). This spec delivers the **46 skeletons**
counted in the family-I row of [../component-taxonomy.md](../component-taxonomy.md): a matrix of
**15 generative block shapes × the sweep styles of [skeletons.md](skeletons.md)**, curated to **18 free
+ 28 Pro presets** (§6.1). It also defines the `genui-skeleton` composition strategy (how a surface
assembles node skeletons) with **3 free + 8 Pro** variants (§6.2).

Ownership (as agreed in [skeletons.md](skeletons.md) header): family D owns the skeleton **engine**, the
sweep styles, the swap-in rules, `show_delay` (150 ms) and `min_display` (300 ms incl. exit); family I
owns the **block shapes** for generated nodes and **counts the combos**. Nothing here redefines a
family-D token; per-block tunings are marked *proposed*.

## 1. Purpose
When an agent starts emitting a surface, the user should see **what kind of interface is coming and
where each part goes** — a chart's frame, a table's columns, a form's fields — so the eye is already in
place and nothing jumps when data lands. Generated UI makes this harder than ordinary loading: the shape
is often only partly known (a hint says "table" but not how many rows), parts arrive out of order, and
some parts are never sent. Research note 02 found no library shipping a skeleton-per-generative-block
system with multiple sweep styles (as recorded in [skeletons.md](skeletons.md) §1); generative frameworks
name the states (`building`, `partial`) but leave the visuals to the app (research note 05 §1). The
matrix gives every block a shape that **matches its final layout** and a small set of sweeps that suit
it (a chart's skeleton "draws", a table's rows "stream", a surface "blueprints").

## 2. State machine
Each node skeleton runs the [skeletons.md](skeletons.md) §2 machine (`Hidden → Pending → Showing →
Partial → Swapping → Done`, plus `Stalled`, `Cancelled`, empty and error exits). Family I adds **shape
resolution** in front of it, because a generated node's shape sharpens as props stream:

```
  child id referenced, type unknown
  Unknown ──────────────────────▶ Generic(default block: 3-line text, or the parent's hint)
     │ type known (nodes.upsert with "type")
     ▼
  Typed(block) ── shape from block defaults (table 5 rows, chart bars, form 3 fields)
     │ structural props known (columns, series kind, field list, aspect, expected_rows)
     ▼
  Shaped(block, dims) ── exact footprint; ready for Partial fill
     │ first real part ─▶ skeletons.md Partial(k of n) … ─▶ Swapping ─▶ Done
     │
     ├─ hint contradicted (type differs) ─▶ Reshape: crossfade 120 ms to the new Typed shape (never morph unrelated shapes)
     ├─ commit while Unknown / Generic ─▶ Collapse (footprint → 0, 200 ms)
     └─ validation error at commit ─▶ error tile at current footprint (renderer NodeError)

  Surface level ("generating this UI"): SurfaceWireframe(composition of block shapes from layout_hint
  or default 1 title + 2 cards + 1 table) ── first node Shaped ──▶ per-node skeletons take over
```

## 3. States table

### 3.1 Skeleton states (family-I view)
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Generic` | default 3-line text skeleton (or hinted block) | — | — | — |
| `Typed` | block's default shape (§3.2) | — | — | — |
| `Shaped` | exact shape: real column count and widths, chart kind, field count, media aspect | — | — | — |
| `Partial(k/n)` | real parts in place (header, axes, first rows, arrived fields); rest sweeping | — | Stop (composer) | Tab into real parts |
| `Reshape` | 120 ms crossfade between shapes | — | — | — |
| `Stalled` | sweep 0.5×, "Still generating…" after 5 s (skeletons.md) | — | Stop | — |
| `Collapse` | footprint eases to 0 | — | — | — |
| `SurfaceWireframe` | whole-surface composition with caption "Generating interface…" | — | Stop | — |

### 3.2 Block shapes (15 rows of the matrix; sizes proposed, derived from tokens)
Shared rules from [skeletons.md](skeletons.md) §3: bar height = 0.55 × the line height it stands for,
radius = bar height ÷ 2 (or `radius` token), widths seeded by node id, skeleton tone = surface ± 6 %
lightness, sweep highlight = surface ± 12 %.

| # | Block | Skeleton shape | Known-early parts that swap first |
|---|---|---|---|
| B1 | Text / prose (`Text`, `Card` body) | 3 lines 92 / 76 / 48 %; heading role → one 40 % bar at heading line height | — |
| B2 | Card | 1 px outline real; title 45 % bar, subtitle 30 % bar, 2 body lines, optional media box at its aspect, action pill boxes at real size | title, outline |
| B3 | List | N rows (expected or 4) of leading 20 px circle/square + bar (70 / 55 / 80 / 62 %) + trailing 15 % bar | item count |
| B4 | Table | header bars (or real labels), 5 body rows (max 12), column widths from schema, numeric columns right-aligned 30–50 % bars, real dividers | header, columns |
| B5 | Line / area chart | real axes frame and 4 gridlines when known; one flat, gently waving path at 40 % height; legend bars | axes, legend |
| B6 | Bar chart | N bars (from category count, default 6) of seeded heights 30–85 % rising from the baseline | axes, categories |
| B7 | Pie / donut | ring track (donut) or full disc at 40 % tone, legend bars at 3–6 rows | legend |
| B8 | Sparkline | 48–120 × 16–24 px flat line at mid-height with an end dot box | — |
| B9 | Metric / KPI | label 40 % bar (caption height), value box 60 % at value line height, delta pill box 48 px, optional sparkline box | label |
| B10 | Timeline | rail line real; N dots (default 4) with time 20 % bar + title 55 % bar per event | rail, event count |
| B11 | Map | box at requested aspect (default 16:9) with 3 seeded pin dots and a 1 px graticule at 8 % tone; pin-list rows under it in `pin-list` | aspect, place count |
| B12 | Media (`Image`, `MediaSet`) | box at aspect + caption 60 % bar; gallery → grid of boxes at shared aspect. Generated media switches to [media-generation-placeholder.md](media-generation-placeholder.md) once the job exists | aspect, count |
| B13 | Form | per field: label bar 25 % + field box at real field height (40 px plain, 28 px dense); submit box at real size; section header bars | field count, labels |
| B14 | Question card | header "Question 1 of n" real when n known; question 2 lines (88 / 60 %); 3 option rows of number badge box + 50 % bar | n, option count |
| B15 | Surface ("generating this UI") | composition of the above per `layout_hint`; default 1 title bar + 2 cards + 1 table ([skeletons.md](skeletons.md) §3) | layout |

## 4. Micro-interactions
Engine behaviours (enter fade, shared viewport-pinned sweep, partial fill crossfade, swap without
layout jump, stall slow-down) are [skeletons.md](skeletons.md) §4 verbatim. Per-sweep behaviour on
generated blocks (proposed):
- **Shimmer** (all blocks): one 105° band across every skeleton in the surface, viewport-pinned, so a
  dashboard of 12 skeleton blocks moves as one light.
- **Wave** (text, list, timeline): bars breathe in reading order, 150 ms per bar, cap 8 then repeat; on
  a timeline the wave runs down the rail dots.
- **Line-by-line** (text, form): bars appear one at a time as if being written; on a form, field
  skeletons appear in order and stop at the last field the stream has referenced.
- **Stream-fill** (text, list, table, timeline): each bar fills left → right as tokens / rows for that
  part arrive, then swaps to the real content; a row that is full but not yet final holds at 100 %
  with the sweep.
- **Morph-in** (card, list, table, KPI, question): each bar morphs (position + width, 220 ms
  `spring.snap`) into its real text line / cell / value instead of crossfading; KPI value box morphs
  into the number, then digits roll.
- **Chart-draw** (line, bar, pie, sparkline): axes draw first (stroke 0 → full, 300 ms), then on swap
  bars rise / the line path draws / slices sweep (genui-blocks.md §4 timings) from the skeleton
  geometry.
- **Spectral** (line chart, KPI): the sweep band is tinted through the accent ramp — used for "hero"
  numbers and charts in dashboards.
- **Dither** (bar, map, media): skeleton fill is an ordered dither whose density breathes (8 fps level
  steps).
- **Scan-line** (timeline, map): a 1 px line scans top → bottom (timeline: along the rail) once per
  1.6 s, tying into [media-analysis-scan.md](media-analysis-scan.md).
- **Blueprint** (card, media, form, surface): dotted outline of the block's structure with mono size
  labels ("TABLE 5×4", "IMAGE 16:9", "FORM 6 FIELDS") and dashed corner handles; on swap a scan-line
  reveals real content ([R005]).
- **Aurora** (surface): shader-tier soft aurora across the whole surface wireframe while the agent
  writes ([R034]); drops to shimmer on the CSS / Dioxus path when shaders are unavailable.
- **Glyph** (surface, terminal): skeleton cells drawn with `░▒▓` cycling at 10–12 fps.

## 5. Timing
Family-D values are referenced, not changed. Per-sweep tunings for generated blocks are proposed.

| Transition / sweep | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Show delay / enter fade | 150 ms / 180 ms | — | `ease.out` | skeletons.md |
| Min display | 300 ms incl. exit | content may start at +120 ms | — | skeletons.md |
| Shape resolution (Generic → Typed → Shaped) | 120 ms crossfade per step | — | `ease.out` | proposed; height settles 200 ms `spring.snap` |
| Reshape (hint contradicted) | 120 ms crossfade | — | `ease.out` | never morph |
| Collapse (never defined) | 200 ms | — | `spring.snap` | |
| Shimmer | 2.0 s per pass, 30 fps | — | linear | 105° band; skeletons.md |
| Pulse (engine; available to any cell via override) | 2.0 s | — | `ease.in-out` | skeletons.md |
| Wave | 2.0 s loop | 150 ms per bar, cap 8 | `ease.in-out` | skeletons.md |
| Line-by-line | 90 ms per line reveal | 120 ms between lines; waits for stream | `ease.out` | proposed |
| Stream-fill | follows data; ≤ 250 ms lag | — | `spring.snap` | proposed |
| Morph-in | 220 ms | 30 ms stagger, cap 6 | `spring.snap` | proposed |
| Chart-draw axes / series | 300 ms / blocks timings | — | `ease.out` / per block | proposed |
| Spectral | 2.0 s per pass | — | linear | shimmer with accent ramp |
| Dither breathe | 2.0 s loop, 8 fps steps | — | step | proposed |
| Scan-line | 1.6 s per pass | 400 ms rest between passes | `ease.in-out` | proposed |
| Blueprint draw / reveal scan | 400 ms / 600 ms | 30 ms per block, cap 6 | `ease.out` / `ease.in-out` | proposed |
| Aurora | 3.0 s loop | — | sine | shader tier only; proposed |
| Glyph | 10–12 fps | — | step | skeletons.md |
| Swap crossfade / height settle | 180 ms / 200 ms | — | `ease.in`+`ease.out` / `spring.snap` | skeletons.md |
| Stall | 2 s slow-down ramp; caption after 5 s | — | linear | skeletons.md |
| **Reduced motion** | every cell renders the static skeleton tone (no sweep, wave, fill animation, morph, draw, dither breathe, scan, aurora or glyph cycling); shape resolution and swaps are 80 ms opacity crossfades; heights jump once; blueprint keeps its static dotted outline and labels (information, not motion) | — | — | `motion.reduced` |

## 6. Variants

### 6.1 The 46 skeleton presets (block × sweep)
Preset id: `genui-skel/<block>-<sweep>`, with a stable short id `oxyn-skel-NN` (short-id idea from
research note 16). ● = free, ◆ = Pro, blank = not curated (the engine still lets an app pick any sweep
for any block; only curated cells are designed, snapshot-tested and listed in the gallery).

| Block ↓ / Sweep → | shimmer | wave | line-by-line | stream-fill | morph-in | chart-draw | spectral | dither | scan-line | blueprint | aurora | glyph | Cells |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| B1 Text | ● 01 | ● 16 | ◆ 19 | ◆ 20 | | | | | | | | | 4 |
| B2 Card | ● 02 | | | | ◆ 22 | | | | | ◆ 21 | | | 3 |
| B3 List | ● 03 | ● 17 | | ◆ 23 | ◆ 24 | | | | | | | | 4 |
| B4 Table | ● 04 | | | ◆ 25 | ◆ 26 | | | | | | | | 3 |
| B5 Line / area | ● 05 | | | | | ◆ 27 | ◆ 28 | | | | | | 3 |
| B6 Bar | ● 06 | | | | | ◆ 29 | | ◆ 30 | | | | | 3 |
| B7 Pie / donut | ● 07 | | | | | ◆ 31 | | | | | | | 2 |
| B8 Sparkline | ● 08 | | | | | ◆ 32 | | | | | | | 2 |
| B9 Metric / KPI | ● 09 | | | | ◆ 33 | | ◆ 34 | | | | | | 3 |
| B10 Timeline | ● 10 | ● 18 | | ◆ 36 | | | | | ◆ 35 | | | | 4 |
| B11 Map | ● 11 | | | | | | | ◆ 38 | ◆ 37 | | | | 3 |
| B12 Media | ● 12 | | | | | | | ◆ 40 | | ◆ 39 | | | 3 |
| B13 Form | ● 13 | | ◆ 42 | | | | | | | ◆ 41 | | | 3 |
| B14 Question | ● 14 | | | | ◆ 43 | | | | | | | | 2 |
| B15 Surface | ● 15 | | | | | | | | | ◆ 44 | ◆ 45 | ◆ 46 | 4 |
| **Total** | 15 | 3 | 2 | 4 | 5 | 4 | 2 | 3 | 2 | 4 | 1 | 1 | **46** |

**18 free** (shimmer × 15 + wave × 3: `oxyn-skel-01`…`18`) **+ 28 Pro** (`oxyn-skel-19`…`46`).
Static is not a cell: it is every cell's reduced-motion and battery-saver form (`skeleton/static` in
family D).

Pro cells, one line each:
| Id | Preset | Idea |
|---|---|---|
| 19 | `genui-skel/text-line-by-line` | Prose lines appear one by one, as if written |
| 20 | `genui-skel/text-stream-fill` | Each line fills with tokens, then swaps to text |
| 21 | `genui-skel/card-blueprint` | Dotted card outline with "CARD" label and corner handles, scan reveal |
| 22 | `genui-skel/card-morph-in` | Title and body bars morph into their real lines |
| 23 | `genui-skel/list-stream-fill` | Item bars fill as items stream |
| 24 | `genui-skel/list-morph-in` | Item bars morph into item rows |
| 25 | `genui-skel/table-stream-fill` | Rows fill left → right as cells stream; header real first |
| 26 | `genui-skel/table-morph-in` | Cell bars morph into cell values, column by column |
| 27 | `genui-skel/line-chart-draw` | Axes draw, waving path becomes the real line drawing on |
| 28 | `genui-skel/line-spectral` | Accent-ramp sweep over a hero line chart |
| 29 | `genui-skel/bar-chart-draw` | Skeleton bars resize into real bars from the baseline |
| 30 | `genui-skel/bar-dither` | Dithered bars whose density breathes |
| 31 | `genui-skel/pie-chart-draw` | Ring track splits into slices sweeping clockwise |
| 32 | `genui-skel/sparkline-chart-draw` | Flat line draws into the real sparkline |
| 33 | `genui-skel/kpi-morph-in` | Value box morphs into the number, then digits roll |
| 34 | `genui-skel/kpi-spectral` | Accent sweep across KPI tiles in a board |
| 35 | `genui-skel/timeline-scan-line` | A light runs down the rail lighting each event dot |
| 36 | `genui-skel/timeline-stream-fill` | Event bars fill as events arrive |
| 37 | `genui-skel/map-scan-line` | Scan passes over the map box and graticule |
| 38 | `genui-skel/map-dither` | Dithered map box with pins resolving from the dither |
| 39 | `genui-skel/media-blueprint` | "IMAGE 16:9" dotted frame, scan reveal (hands off to media-generation-placeholder when generated) |
| 40 | `genui-skel/media-dither` | Dithered media box, un-dithers on swap |
| 41 | `genui-skel/form-blueprint` | Dotted field outlines with "FORM 6 FIELDS" label |
| 42 | `genui-skel/form-line-by-line` | Fields appear one at a time as the agent declares them |
| 43 | `genui-skel/question-morph-in` | Option bars morph into numbered options |
| 44 | `genui-skel/surface-blueprint` | Whole-surface blueprint with mono block labels and a build log ([R005]) |
| 45 | `genui-skel/surface-aurora` | Aurora wash over the whole wireframe while the agent writes ([R034]) |
| 46 | `genui-skel/surface-glyph` | Terminal wireframe in `░▒▓` cells |

### 6.2 `genui-skeleton` composition variants
**Free (3)**
| Name | Idea |
|---|---|
| `genui-skeleton/per-node` | Each node shows its own block skeleton as soon as it is referenced (default) |
| `genui-skeleton/surface-wireframe` | One wireframe for the whole surface from the hint until the first node is Shaped, then per-node |
| `genui-skeleton/static` | No motion; matched shapes only (IDE panels, terminals, battery saver) |

**Pro (8)**
| Name | Idea |
|---|---|
| `genui-skeleton/blueprint-labels` | Every skeleton carries a mono size label from its known dims ("TABLE 12×5") |
| `genui-skeleton/build-log` | A small log stream beside the wireframe lists parts as the agent declares them ("+ chart · revenue") ([R005]) |
| `genui-skeleton/tree-cascade` | Containers skeleton first, children cascade in tree order (30 ms) |
| `genui-skeleton/progressive-detail` | Shapes start coarse (one box per block) and gain detail as structure arrives |
| `genui-skeleton/aurora-region` | Only the region currently being written glows; finished regions are still ([R034]) |
| `genui-skeleton/ghost-previous` | On regenerate / edit, the previous version shows at 30 % as the skeleton for the new one |
| `genui-skeleton/glyph` | Terminal composition: all skeletons as glyph cells, one shared cycle |
| `genui-skeleton/count-hint` | Skeletons show expected counts as captions ("12 rows", "6 fields") when the hint has them |

## 7. Edge cases
- **No hint, unknown types.** Generic text skeletons; reshape per node as types arrive (one 120 ms
  crossfade each, never more than two reshapes per node — after the second, the node waits in its
  current shape until Shaped).
- **Hint says table, agent sends chart.** Reshape crossfade; no morph between unrelated shapes.
- **Expected counts wrong.** Extra rows / fields append without skeletons; leftover skeleton parts
  collapse at the end (skeletons.md §7).
- **Dozens of blocks (dashboard).** One shared sweep for all shimmer / spectral cells; dither and scan
  cells phase-lock to the shared clock; container-loaders.md caps apply; offscreen skeletons stop.
- **Mixed sweeps on one surface.** Allowed only within one motion family per surface (proposed): a
  surface uses either the light family (shimmer, spectral, aurora) or the structure family (wave,
  line-by-line, stream-fill, morph-in, chart-draw) plus static; scan-line, dither, blueprint and glyph
  are surface-wide styles. Mixing families looks like several loaders, against
  [loaders.md](loaders.md) §2.3.
- **Very fast nodes (< 150 ms).** No skeleton (show delay).
- **Very slow agents.** Stall behaviour; `build-log` keeps showing the last declared part.
- **Tiny blocks (< 20 px tall: sparkline, badge).** Shimmer is replaced by static (band would be
  narrower than 24 px; skeletons.md §7).
- **RTL.** Bars align to inline-start; shimmer and stream-fill travel right → left; chart skeleton x
  direction follows `chart.time_direction`; numeric table columns keep their alignment.
- **Touch / scroll.** Skeletons never intercept input; the viewport-pinned sweep does not restart on
  scroll.
- **Dense IDE.** `static` or `per-node` with shimmer at 15 fps; no aurora.
- **Terminal.** Only glyph and static cells.
- **Dark / light themes.** Tones from tokens; dither and blueprint lines use the same ± 6 / 12 %
  lightness steps.
- **Shader unavailable** (Dioxus without WebGL, low-power mode): aurora → shimmer; dither → static
  dither frame.

## 8. Accessibility
- Skeleton shapes are `aria-hidden`; the surface container is `aria-busy` with "Generating interface"
  ([skeletons.md](skeletons.md) §8). Blueprint labels and build-log lines are decorative duplicates of
  information announced by the container ("Generating interface: table, chart, form") once, politely,
  when the hint arrives.
- When counts are known, the container's `aria-valuetext` reports progress ("Loaded 3 of 5 parts").
- Nothing in a skeleton is focusable; focus never moves on swap.
- Flashing: no sweep faster than 1.6 s per pass; glyph cycling at ≤ 12 fps with low contrast steps;
  luminance difference of sweep vs base ≤ 12 % lightness.
- Reduced motion: §5 last row; information (shape, labels, counts) is preserved without motion.

## 9. Props sketch
```rust
pub struct GenSkeletonCfg {
    pub composition: SkeletonComposition,     // PerNode | SurfaceWireframe | Static | Pro(..)
    pub presets: SkeletonPresetMap,           // BlockShape -> PresetId (default: shimmer cell of each row)
    pub motion_family: MotionFamily,          // Light | Structure | SurfaceWide(SweepStyle) — one per surface
    pub max_reshapes: u8,                     // 2 (proposed)
    pub show_counts: bool,                    // count-hint
}

pub enum BlockShape {                         // the 15 matrix rows
    Text { lines: u8, heading: bool }, Card { media: Option<Aspect>, actions: u8 }, List { rows: u8 },
    Table { columns: Option<Vec<ColumnHint>>, rows: u8 }, LineArea { series: u8 }, Bar { bars: u8 },
    Pie { donut: bool, slices: u8 }, Sparkline, Metric { sparkline: bool }, Timeline { events: u8 },
    Map { aspect: Aspect, pins: u8 }, Media { aspect: Aspect, count: u8 }, Form { fields: Vec<FieldHint> },
    Question { options: u8, pages: Option<u8> }, Surface(Vec<BlockShape>),
}

pub struct SkeletonPreset {
    pub id: PresetId,                         // "genui-skel/table-stream-fill", short "oxyn-skel-25"
    pub block: BlockShape,
    pub sweep: SweepStyle,                    // from skeletons.md: Shimmer | Wave | LineByLine | StreamFill | MorphIn | ChartDraw
                                              // | Spectral | Dither | ScanLine | Blueprint | Aurora | Glyph
    pub tier: Tier,                           // Free | Pro
}

pub enum ShapeLife { Generic, Typed(BlockShape), Shaped(BlockShape), Reshaping { from: BlockShape, to: BlockShape }, Collapsing }

// Rendering delegates to family D's `Skeleton` (skeletons.md §9) with `block` mapped from BlockShape and
// `seed` = node id; `AgentState::Stalled` → skeleton stall; `Cancelled` → skeleton cancel fade.
```

## 10. Related components and references
Related: [skeletons.md](skeletons.md) (engine, sweeps, swap, a11y — owner) ·
[genui-renderer.md](genui-renderer.md) (surface wireframe, per-node boundaries) ·
[genui-blocks.md](genui-blocks.md) and [genui-forms.md](genui-forms.md) (final layouts each shape must
match) · [media-generation-placeholder.md](media-generation-placeholder.md) (generated media takes over
from B12) · [media-analysis-scan.md](media-analysis-scan.md) (scan-line) ·
[container-loaders.md](container-loaders.md) (shared clock, caps) · [text-shimmer.md](text-shimmer.md)
(same band maths).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Family-D skeleton engine and sweep set (shimmer, pulse, wave, static; Pro comet, spectral, scan-line, dither, blueprint, line-by-line, stream-fill, morph-in, chart-draw, code-gutter, glyph, aurora) | [skeletons.md](skeletons.md) | all sweep definitions | Oxyn spec |
| Generative UI `building` / `partial` states; render with props undefined (Tambo, A2UI, CopilotKit) | research note 05 §1 | shape resolution machine | not recorded: concepts only |
| Skeleton-blueprint with mono size labels, log stream, scan reveal | [R005], research note 15 | blueprint cells, `build-log` | Study only; transitions.dev licence forbids competing kits |
| Aurora glow over the region the agent is writing | [R034] | aurora cell, `aurora-region` | Study only |
| SPY UI "AI Generative Skeleton (text line by line)" | [R136], research note 17 | line-by-line cells | Study only |
| affectUI Shimmer Loader: one viewport-pinned sweep; `aria-hidden` skeletons, `aria-busy` container | research note 11 | shared sweep; a11y | Paid / unclear: study only |
| Arc Skeleton: crossfade into children with height easing, staggered pulse, "use text-shimmer for AI status lines" | research note 12 | swap behaviour cross-check; skeletons not used for status lines | Paid / unclear: study only |
| Zed agent panel generating skeleton (4 bars, staggered opacity, 2 s loop) | research note 08 | wave cells | Mixed licence by crate: study only |
| Stable short ids for gallery items (`oxyn-loader-014` idea) | research note 16 | `oxyn-skel-NN` | idea only |

**Gaps.** Which adapters deliver structural props (columns, field lists, expected counts) before data
is unresearched per protocol, so how often nodes reach `Shaped` early is unknown. The "one motion family
per surface" rule is a proposal to validate in the Studio.

## Plan #26 note
The block shapes `message`, `tool-row`, `diff`, `card`, `table` and `form` and the `shimmer` /
`pulse` / `wave` sweeps ship in `oxyn-core::primitives::skeleton` and `oxyn-dioxus::Skeleton`
(see [skeletons.md](skeletons.md)); the full block × sweep matrix for generative UI is still a draft.
