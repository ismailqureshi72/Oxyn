**Status:** draft v0.1 (2026-11-11).

# Spec: skeletons (messages, cards, tables, charts, code blocks)

Part of family D ([container-loaders.md](container-loaders.md)); follows the 10-point standard in
[README.md](README.md) at the depth of [composer-send-control.md](composer-send-control.md). Target
**4 free + 12 Pro = 16 variants** of sweep style and swap behaviour. **Block shapes** (message, card,
table, chart, code, plus image, list, form, map, timeline) are shared with family I's generative-UI
skeletons ([../component-taxonomy.md](../component-taxonomy.md): 46 block × sweep combos); family I
counts the combos, family D owns the skeleton engine, the sweep styles, timing and swap-in.

## 1. Purpose
Skeletons hold the place of content the agent is about to produce or fetch: the reply before the first
token, a source card while its page loads, a generated table while rows stream, a chart while the
query runs, a code block while a file is read, a whole "generating this UI" surface in an app builder.
The user wants to see **what kind of thing is coming and where**, so their eye is already in the right
place when it arrives, and they want **no jump** when it does. A skeleton that does not match the
shape it is replaced by is worse than none: it moves everything when the real content lands. Research
note 02 found no library shipping a skeleton-per-generative-block system with multiple sweep styles;
that is the gap this spec fills.

## 2. State machine

```
             content requested                 show_delay (150 ms)
   Hidden ─────────────────────▶ Pending ─────────────────────────▶ Showing(sweep)
     ▲                             │ content < 150 ms: render directly,       │
     │                             │ no skeleton (no flash)                   │ part of the content known
     │                             │                                          ▼  (header, first rows, first lines)
     │                             │                                     Partial(filled k of n)
     │                             │                                          │
     │   no event for stall_soft ──┼──────────────▶ Stalled ("Still loading…")│
     │                             │                    │ event               │
     │                             │                    └──────▶ Showing ◀────┘
     │                                                                        │ content complete (or first token
     │                                                                        ▼  for streaming text)
     │◀───────────── Done ◀── Swapping (crossfade + height settle) ◀──────────┤  (waits for min_display)
     │◀── Empty state block ◀── Swapping ◀── content arrives empty ───────────┤
     │◀── Error block (same footprint, Retry) ◀── Swapping ◀── error ─────────┤
     └◀── fade 160 ms ── Cancelled ◀── cancel ────────────────────────────────┘
```
`min_display` = **300 ms** measured from the first painted frame and **including** the exit crossfade
(decision, proposed): content may start fading in once the skeleton has been fully visible for 120 ms,
so streaming text is never held back more than 120 ms by the rule, while the eye still registers the
skeleton.

## 3. States table

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing / real content | — | — | — |
| `Pending` (≤ 150 ms) | reserved footprint, empty | — | — | — |
| `Showing` | block shapes in skeleton tone with the chosen sweep | — | cancel (only if the host block is cancellable, e.g. a tool card) | Esc cancel (focused host) |
| `Partial(k/n)` | real parts in place (table header, first rows, chart axes); remaining parts still skeleton | — | — | — |
| `Stalled` | sweep slows to 0.5×; after 5 s a small "Still loading…" caption appears under the block (status strip thresholds) | retry (if host supports) | cancel | Enter retry |
| `Swapping` | skeleton fades out as content fades in; container height eases from predicted to actual (top edge fixed) | — | — | — |
| `Done` | real content | host's | — | host's |
| Empty result | empty-state block ("No rows", "No results") at the skeleton's footprint, then height settles | host's | — | — |
| Error | error block in the same footprint: "Couldn't load · Retry" | retry | details | Enter retry |
| `Cancelled` | skeleton fades (160 ms); host decides what remains | — | — | — |

### Block shapes (shape matching; all sizes proposed, all derive from tokens)
Bar height = 0.55 × the text line height it stands for; bar radius = bar height ÷ 2 (or the `radius`
token); gaps = the real line gap. Widths are pseudo-random but **seeded by the block id**, so they never
change on re-render.

| Block | Skeleton shape |
|---|---|
| **Message (assistant)** | avatar circle at the real avatar size (if the thread shows avatars); 3 lines at 92 % / 76 % / 48 % width; when the expected length is known (e.g. "short reply"), 1–2 lines; no bubble chrome change |
| **Message (tool / status row)** | icon square + one 40 % bar + trailing 48 px time bar, row height = real row height |
| **Source / link card** | favicon 16 px circle + domain 30 % bar; title 2 lines (88 %, 60 %); optional thumbnail box at the real ratio on the trailing side |
| **Media card** | media box at the requested aspect ratio + title 60 % + 2 meta bars (40 %, 25 %) |
| **Table** | header row with column labels if known (real text), else header bars; N body rows (N = expected rows, default 5, max 12); column widths from the schema when known; numeric columns right-aligned short bars (30–50 % of cell); row dividers real |
| **Chart** | real axes frame and gridlines if known; bars: N bars of seeded heights rising from the baseline; line / area: one flat, gently waving path at 40 % height; pie / donut: ring track; legend bars below |
| **Code block** | real header (language chip, filename if known); gutter with line numbers in the muted tone; bars in monospace line height with seeded indentation (0–3 levels × 2 ch) and widths 20–80 ch; ≥ 4 lines, default 8 |
| **List** | N rows of bullet dot + bar (70 %, 55 %, 80 % …) |
| **Form** | label bar 25 % + field box at the real field height, × N fields; primary button box at real size |
| **Whole surface ("generating this UI")** | composition of the blocks above in the layout the agent declared; unknown layout → 1 title bar + 2 cards + 1 table |

## 4. Micro-interactions
- **Enter.** After `show_delay`, the skeleton fades in (180 ms) at its final footprint. No grow-in; a
  skeleton that animates its size defeats its purpose.
- **Shimmer (shared sweep).** One sweep band (105°, 2 s linear) travels across **all** skeleton blocks in
  the viewport together, pinned to the viewport rather than to each block, so a page of skeletons moves
  as one light and costs one gradient (affectUI Shimmer Loader behaviour, research note 11; the 2 s
  105° sweep matches the Claude desktop skeleton sweep, research note 08).
- **Pulse.** All bars breathe opacity 0.45 ↔ 1 on a 2 s `ease.in-out` loop, in phase.
- **Wave.** Bars breathe in sequence, 150 ms stagger per bar in reading order (the staggered generating
  skeleton in Zed's agent panel, research note 08, uses 4 bars with a 0.15 stagger on a 2 s loop; our
  values are our own).
- **Partial fill.** As real parts arrive (table header, rows, chart axes, first code lines), each replaces
  its skeleton part with a 120 ms crossfade in place; remaining skeleton parts keep sweeping. Rows
  arriving faster than 30 ms apart batch into one crossfade.
- **Swap-in without layout jump.** Content is laid out invisibly in the same frame; the skeleton
  crossfades out (180 ms) while content crossfades in; if the measured height differs, the container
  eases from predicted to actual height (200 ms, `spring.snap`) with the **top edge fixed**, so text
  above never moves and content below shifts once, smoothly. In a chat transcript pinned to the bottom,
  the scroll anchor compensates so the viewport does not jump.
- **Streaming text.** The message skeleton is replaced by the streaming text at the first token
  (subject to `min_display`); remaining skeleton lines collapse as text overtakes them, never below the
  text.
- **Stall.** Sweep slows to half speed (2 s ramp) and a muted "Still loading…" caption fades in after the
  status strip's soft threshold (5 s).
- **Error.** The skeleton crossfades into an error block of the same footprint, then height settles; one
  4 px shake on the Retry button only (not the block).

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Show delay | — | 150 ms | — | proposed |
| Skeleton enter (fade) | 180 ms | 0 | `ease.out` | no size animation |
| Shimmer sweep (`loop.sweep`) | 2.0 s per pass | — | linear | 105° band, ~30 % of viewport width; 30 fps |
| Pulse (`loop.pulse`) | 2.0 s | — | `ease.in-out` | opacity 0.45 ↔ 1 |
| Wave | 2.0 s loop | 150 ms per bar, cap 8 bars then repeats | `ease.in-out` | |
| Partial part crossfade | 120 ms | batch window 30 ms | `ease.out` | |
| Min display | 300 ms total incl. exit | content may start at +120 ms | — | decision, proposed |
| Swap crossfade (out / in) | 180 ms / 180 ms | simultaneous | `ease.in` / `ease.out` | |
| Height settle | 200 ms | 0 | `spring.snap` | top edge fixed |
| Stall: slow-down / caption | 2 s ramp / 160 ms fade | caption after 5 s | linear / `ease.out` | thresholds: status strip |
| Cancel | 160 ms | 0 | `ease.in` | |
| **Reduced motion** | static skeleton tone (no sweep, no pulse, no wave); swap = 80 ms opacity crossfade; height change instant (one jump, no animation); caption still appears on stall | — | — | |

## 6. Variants

### Free (4)
| Name | Idea |
|---|---|
| `skeleton/shimmer` | One viewport-pinned 105° sweep across all blocks (default) |
| `skeleton/pulse` | All bars breathe in phase |
| `skeleton/wave` | Bars breathe in reading-order sequence |
| `skeleton/static` | No motion at all; for dense IDE panels, terminals and battery saver |

### Pro (12)
| Name | Idea |
|---|---|
| `skeleton/comet` | Asymmetric sweep with a short bright head and long tail on coast easing (OpenCode-style shimmer shape, research note 08) |
| `skeleton/spectral` | Sweep tinted through the accent ramp (brand colour sweep) |
| `skeleton/scan-line` | A 1 px line scans the blocks top to bottom (ties into media-analysis-scan) |
| `skeleton/dither` | Bars rendered as ordered dither whose density breathes |
| `skeleton/blueprint` | Dotted wireframe built from the block's structure with mono size labels, then a scan reveal ([R005]) |
| `skeleton/line-by-line` | Text bars appear one line at a time as if being written (generative skeleton) |
| `skeleton/stream-fill` | Bars fill left → right as tokens for that block arrive, then swap to text |
| `skeleton/morph-in` | Each bar morphs into its real text line / cell (width and position) instead of a crossfade |
| `skeleton/chart-draw` | Axes draw first, bars rise or the line path draws on swap |
| `skeleton/code-gutter` | Line numbers count in with the bars; real lines replace bars top-down |
| `skeleton/glyph` | Terminal: bars drawn with `░▒▓` cells cycling at 10–12 fps |
| `skeleton/aurora` | Shader-tier soft aurora sweep across the whole surface ("generating this UI") |

## 7. Edge cases
- **Unknown content size.** Use the block's default shape (message 3 lines, table 5 rows, code 8 lines);
  the height settle on swap handles the difference once.
- **Predicted much larger than actual** (5-row skeleton, 1-row result). Height settles in 200 ms; in a
  bottom-pinned transcript the scroll anchor keeps the composer and latest message still.
- **Very fast content (< 150 ms).** No skeleton.
- **Content between 150 ms and 300 ms.** Skeleton shown; content starts fading in at +120 ms after the
  skeleton's first frame; total skeleton visibility 300 ms including its fade.
- **Very slow (minutes).** Sweep continues; `Stalled` caption; offscreen skeletons stop ticking.
- **Many skeletons (dashboards of 30 cards).** One shared sweep for all; per-block pulses are
  phase-locked to the shared clock; container-loaders.md §4.4 caps apply.
- **Streaming tables.** Header first (real), rows replace skeleton rows; when actual rows exceed the
  skeleton rows, new rows append without skeletons; when fewer, leftover skeleton rows collapse at the
  end.
- **Charts with unknown type.** Bars placeholder; switching to the real type is a crossfade, never a
  morph between unrelated shapes.
- **Code language unknown.** No chip; gutter still shown.
- **RTL.** Bars align to the inline-start (right); sweep travels right → left; table numeric columns
  stay aligned as the real content will be; code blocks stay LTR.
- **Tiny sizes (rows < 20 px).** `static` or `pulse` only; no sweep band narrower than 24 px.
- **Dark / light themes.** Skeleton tone = surface ± 6 % lightness; sweep highlight = surface ± 12 %
  (proposed); both from tokens.
- **Touch / scroll.** Skeletons never intercept input; scrolling a skeleton list does not restart the
  sweep (it is pinned to the viewport).
- **Virtualised lists.** Recycled rows take the new item's skeleton seed immediately.
- **Partial errors** (some cards fail). Each card swaps to its own error block; the rest keep loading.

## 8. Accessibility
- **Skeletons are hidden.** Every skeleton shape is `aria-hidden="true"` (affectUI does the same,
  research note 11). The **container** gets `aria-busy="true"` while `Showing` / `Partial` / `Stalled`
  and a visually hidden label ("Loading results", "Generating reply").
- **Progress.** Skeletons are indeterminate by nature; when a table or list knows its expected count,
  the container's `aria-valuetext` says "Loaded 3 of 12 rows".
- **Live region:** the skeleton itself announces nothing; family B announces the run, and the host
  announces completion only if the user requested it ("Results loaded", "Table ready, 12 rows").
  Errors are assertive ("Couldn't load results. Retry available").
- **Focus.** Nothing focusable inside a skeleton; focus waits on the host; when content swaps in,
  focus is not moved.
- **Flashing.** Sweep period 2 s; pulse 2 s; no skeleton flashes; luminance difference of sweep vs
  base ≤ 12 % lightness.
- **Contrast.** Skeleton shapes are decorative; the "Still loading…" caption and error text meet 4.5:1.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct Skeleton {
    pub agent: AgentState,                // Submitted / Thinking → message skeleton; ToolRunning → tool card skeleton
    pub job: Option<JobSnapshot>,         // when the block is fed by a tool or fetch
    pub block: SkeletonBlock,             // Message{lines, avatar} | ToolRow | SourceCard | MediaCard{aspect}
                                          // | Table{columns: Option<Vec<ColumnHint>>, rows: u8}
                                          // | Chart{kind: Option<ChartKind>, series: u8}
                                          // | Code{lines: u16, language: Option<SharedString>}
                                          // | List{rows} | Form{fields} | Surface(Vec<SkeletonBlock>)
    pub seed: u64,                        // usually the block / message id: stable widths
    pub sweep: SweepStyle,                // Shimmer | Pulse | Wave | Static | Comet | Spectral | ScanLine | Dither ...
    pub shared_sweep: bool,               // true = viewport-pinned single band (default)
    pub filled: Option<PartialFill>,      // real parts already known (header, k rows, axes)
    pub show_delay: Duration,             // 150 ms (proposed)
    pub min_display: Duration,            // 300 ms incl. exit (proposed)
    pub swap: SwapStyle,                  // Crossfade (default) | MorphIn | StreamFill
    pub motion: MotionLevel,
    pub labels: SkeletonLabels,           // container busy label, stall caption
}
```

## 10. Related components and references
Related: family I generative UI blocks (owns block × sweep combos) · family E messages (first-token
skeleton) and source cards · family F tool cards · family C shimmer (same `shimmer()` primitive with a
2D band) · media-analysis-scan (`scan-line`) · container-loaders.md (shared clock and caps).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| affectUI Shimmer Loader: one viewport-pinned sweep across all blocks; Skeleton `width` / `height` 12 / `radius` 6 / `circle`; `aria-hidden` skeletons, `aria-busy` container; static under reduced motion | research note 11 | shared sweep; a11y pattern | Paid / unclear: study only |
| Claude desktop skeleton sweep (2 s linear at 105°) | research note 08 | sweep angle and period | Proprietary: reference only |
| Zed agent panel generating skeleton (4 bars, staggered opacity, 2 s loop) | research note 08 | `wave` idea | Mixed licence by crate: study only |
| OpenCode comet shimmer (4-col head, 18-col tail, coast easing) | research note 08 | `comet` | MIT: may port with attribution |
| Jakub Antalik skeleton-blueprint; transitions.dev `transitions-ui-generation` skill (modes `skeleton`, `skeleton-texts`) | [R005], [R132], research note 17 | `blueprint` | Study only; transitions.dev licence forbids competing kits |
| SPY UI "AI Generative Skeleton (text line by line)" | [R136], research note 17 | `line-by-line` | Study only |
| Animata skeletons (Code, List, Report, Wide Card…); Loading UI skeleton | research note 02, [R091] | block coverage check | Animata licence not recorded: study only; Loading UI open source (confirm before porting) |
| Generative UI `building` / `partial` states (CopilotKit, A2UI, Tambo) | research note 05 | `Partial` state: render gracefully with missing props | Framework docs: concepts only |

Gaps: the "expected size" hint (rows, lines) depends on what each adapter can tell us before content
arrives; not researched per protocol.
