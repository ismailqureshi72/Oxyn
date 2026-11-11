**Status:** draft v0.1 (2026-11-12).

# Spec: generative UI data blocks (chart, table, metric, list, timeline, map, media)

Part of family I ([genui.md](genui.md)); follows the 10-point standard in [README.md](README.md) at the
depth of [composer-send-control.md](composer-send-control.md). Seven components share one block lifecycle
and one section per point: `genui-chart`, `genui-table`, `genui-metric`, `genui-list`, `genui-timeline`,
`genui-map`, `genui-media`. Target **21 free + 41 Pro = 62 variants** (≥ 3 free per component; chart and
table are the high-traffic pair at 9 Pro each). Blocks are nodes in a surface ([genui.md](genui.md) §2.4);
their skeleton shapes are in [genui-skeletons.md](genui-skeletons.md), the engine in
[skeletons.md](skeletons.md). Values marked *proposed* are Oxyn's own.

## 1. Purpose
Data blocks are what most generated surfaces are for: the finance agent's spend breakdown, the BI agent's
query result, the CRM agent's pipeline records, the research agent's timeline of events, the travel
agent's map of stops, the media agent's candidate images. The user wants to **read the answer at a
glance** (headline number, shape of the trend), **check it** (hover a point, sort a column, open a row),
**act on it** (select rows and approve, "chart this column") and **trust the numbers** (units, period,
source, what changed since the agent last updated it). Peer coverage we checked: Thesys C1 renders line,
bar, pie and scatter charts and filter / sort / paginate tables (note 05); research note 05 recommends a
chart set of line, bar, pie / donut, scatter + table + KPI tile; Arc lists sparkline, gauge, activity
heatmap and sortable data tables among its free charts and tables (note 12); a finance dashboard with
KPIs with sparkbars and a radial tick gauge drawing in ([R025]); records tables with a calc footer for CRM
agents ([../component-taxonomy.md](../component-taxonomy.md) section B).

## 2. State machine (shared block lifecycle)

```
   Referenced ─▶ Skeleton(shape from props known so far)
                    │ schema known (columns / series / axes)
                    ▼
               Structured ── header / axes / legend real, body still skeleton
                    │ first data (rows, points, items)
                    ▼
               Filling(k of n?) ── rows / points append (batched 30 ms); remaining skeleton parts sweep
                    │ node.done / commit
                    ▼
               Ready ──┬─ hover / focus item ─▶ Inspecting(item)  (tooltip, crosshair, row highlight)
                       ├─ sort / filter / page ─▶ Reordering ─▶ Ready      (client-side; no agent call)
                       ├─ select items ─▶ Selecting(n) ─▶ bulk intent ─▶ (renderer: Submitting / AwaitingApproval)
                       ├─ data.patch ─▶ Updated (changed cells / points washed, numbers roll) ─▶ Ready
                       ├─ "Load more" / next server page ─▶ Fetching (Query intent) ─▶ Ready | FetchError
                       └─ "View as table" (chart) / "Chart this" (table) ─▶ Alternate view ─▶ Ready
   Empty: data final and empty ─▶ Empty("No rows" / "No data for this period")
   Error: data invalid / fetch failed ─▶ Error tile (same footprint) ── Retry ─▶ Skeleton
   Stalled overlay (no patch 5 s) · Truncated (data cap hit, count shown)
```

## 3. States table

### 3.1 Shared
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Skeleton` | block shape ([genui-skeletons.md](genui-skeletons.md)) | — | — | — |
| `Structured` | real header / axes / legend / title; body skeleton | — | — | — |
| `Filling(k/n)` | real data so far; caption "Loading 40 of 120 rows" when n known | — | Stop (composer) | Tab into loaded part |
| `Ready` | full block; block header with title, unit / period subtitle, overflow menu (Copy as CSV / Copy image / View as table / Chart this / Open in panel) | per block | overflow | per block (§3.2) |
| `Inspecting` | tooltip / crosshair / row highlight | open item (if intent) | copy value | arrows move, Enter opens |
| `Reordering` | sort arrow rotates; rows reflow | — | — | — |
| `Selecting(n)` | checkboxes; floating bulk bar "3 selected · Approve · Export" | bulk intent | clear | Space toggles, `⌘/Ctrl+A` all, Esc clears |
| `Updated` | changed values washed 12 % accent, fading 1200 ms; numbers roll | — | "What changed" (lists diffs) | — |
| `Fetching` | footer loader 12 px "Loading more…" | — | cancel | Esc |
| `Empty` | empty-state line with icon at footprint, then height settles | agent-supplied suggestion (e.g. "Widen the date range") as an `Answer` intent | — | Enter |
| `Error` | "Couldn't show this table · Retry" | Retry | Details | Enter |
| `Truncated` | footer "Showing 10,000 of 48,210 rows" | Load more (if Query intent) | Export (if intent) | — |

### 3.2 Per block
| Block | Types / cells | Primary interaction | Keyboard (block focused) |
|---|---|---|---|
| `Chart` | `line`, `area` (stacked opt.), `bar` (vertical / horizontal, grouped / stacked), `pie`, `donut`, `sparkline`; series ≤ 8 visible (rest in "Other"); axes: time, category, linear, log | hover crosshair + tooltip with all series at x; legend toggles series; drag to zoom x (Pro `scrub-insight`) | ←/→ move between points (Home/End first/last), ↑/↓ switch series, Enter pins tooltip, `T` view as table |
| `Table` | cells: `text`, `number`, `currency`, `percent`, `date`, `datetime`, `relative`, `duration`, `bytes`, `boolean`, `status` (badge), `link`, `person` (avatar + name), `progress` (bar), `sparkline`, `code`, `json` (collapsed), `image` (thumb 32 px) | click header: sort asc → desc → none; shift-click adds secondary sort; drag column edge to resize (min 64 px); paginate 25 / page (options 10 / 25 / 50 / 100); virtualise > 200 rows | arrow keys move cell (grid pattern), Enter on header sorts, Enter on row opens (if intent), Space selects row, PgUp / PgDn page |
| `Metric` | `value`, `label`, `delta` (abs or %), `direction_good: up\|down\|none`, `period` ("vs last month"), optional `sparkline`, `target` | hover shows exact value and comparison base | Enter opens drill-down intent |
| `List` | items: leading icon / avatar / thumb, title, subtitle, trailing meta / badge / action (≤ 2) | click item → item intent | ↑/↓ move, Enter open, Space select (if selectable) |
| `Timeline` | events: `time` (ISO 8601), `title`, `body?`, `status` (`done`, `active`, `pending`, `failed`, `skipped`), `actor?`, `duration?` | expand event; "now" marker on live timelines | ↑/↓ (vertical) or ←/→ (horizontal) between events, Enter expand |
| `Map` | `places: [{ id, name, lat, lng, label?, kind? }]`, `route?: [place id]`, `bounds?` | select place ↔ list row highlight | ↑/↓ through places in list order, Enter open |
| `MediaSet` | `image` items (src, alt, aspect), `video` / `audio` (src, poster, captions, chapters?) | open lightbox / play | ←/→ between items, Space play/pause, `C` captions |

### 3.3 Block rules (proposed)
- **Chart.** Y axis starts at 0 for bars and areas, always; lines may use a fitted domain with a visible
  axis break glyph when the domain excludes 0. Max 6 y-ticks, 8 x-ticks, labels never overlap (thinning
  by every-nth). Above 2 000 points per series in view, downsample (largest-triangle three buckets); the
  tooltip shows raw values. Pie / donut: ≤ 6 slices, smaller merged into "Other"; slices sorted
  descending clockwise from 12 o'clock. Every chart gets a generated summary sentence for screen readers
  and copy ("Spend rose from €12.1k in July to €18.2k in September").
- **Table.** Numbers right-aligned in tabular figures; dates left-aligned; header sticky; first column
  sticky when the table is wider than its container. Row height 36 px (`plain`), 28 px (`dense` kit).
  Virtualisation overscan 8 rows. Client-side sort and filter only on data already present; beyond the
  data cap or when the agent declares `server_pages`, paging raises a `Query` intent. Calc footer
  (`sum`, `avg`, `min`, `max`, `count`) per numeric column when declared.
- **Metric.** Delta colour follows `direction_good` (up-good: green up / red down); direction is also an
  arrow glyph and a sign so colour is never the only channel. Values abbreviate past 6 significant digits
  ("€1.24M"), exact on hover.
- **Timeline.** Sorted by `time`; equal times keep emission order; relative labels ("2 h ago") for
  events < 24 h old, absolute beyond, in the user's time zone.
- **Map.** Oxyn ships **no tile engine and no tile provider** (licence and terms vary per provider):
  the free variants show a pin list, an app-provided static image with projected pins, or a host slot
  where the app renders its own map. Coordinates are validated (lat −90..90, lng −180..180).
- **MediaSet.** Generated media inside a surface uses [media-generation-placeholder.md](media-generation-placeholder.md)
  for its in-progress states; finished media never autoplays with sound; video shows captions when the
  track exists.

## 4. Micro-interactions
- **Structured first.** Table headers, chart axes and legends land before data and replace their
  skeleton parts with a 120 ms crossfade ([skeletons.md](skeletons.md) §4 partial fill); the body keeps
  sweeping.
- **Rows stream in.** New rows append below with a 120 ms fade and a 4 px rise, batched per 30 ms;
  when more rows arrive than the skeleton predicted, they append without skeletons; fewer, leftover
  skeleton rows collapse at the end (200 ms).
- **Chart draw.** On first data: bars rise from the baseline (280 ms, 20 ms stagger, cap 12, then all
  at once); lines draw left → right (600 ms); pie slices sweep clockwise (500 ms). Streaming points
  extend the line with no redraw of past segments.
- **Crosshair.** Follows the pointer snapping to the nearest x (no easing; position must be exact);
  tooltip fades in 80 ms after 60 ms hover intent and repositions to stay inside the block.
- **Legend toggle.** Series fades 160 ms; y domain eases to the new extent (240 ms `spring.snap`).
- **Sort.** Header arrow rotates 180° (160 ms); rows reorder with a FLIP move (200 ms `spring.snap`)
  when ≤ 50 rows are visible, instant beyond (moving hundreds of rows reads as noise).
- **Selection.** Checkbox column slides in (160 ms) on first selection or ⌘/Ctrl-click; floating bulk
  bar springs up from the block's bottom edge (`spring.soft`) on the first selection (floating bulk bar,
  [R013]).
- **Updates.** Changed cells and points get a highlighter swipe (left → right accent wash, 400 ms in,
  1200 ms fade) ([R007]); numbers roll digit-wise (400 ms); a "What changed" link appears in the header
  for 10 s.
- **Metric.** Value counts up from 0 on first reveal only when it arrives in one patch (500 ms
  `ease.out`), otherwise rolls between values; delta chip slides in 120 ms after the value.
- **Timeline live edge.** The "now" marker advances once a minute (no continuous motion); the active
  event's status dot breathes on `loop.calm` (2 s).
- **Map.** Selecting a list row pulses its pin once (scale 1 → 1.3 → 1, 300 ms) and vice versa.
- **Media.** Hover lifts a tile 2 px; opening a lightbox is a shared-element expand (260 ms).
- **Empty / error.** Crossfade into the empty or error tile at the same footprint, then height settles.

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Structured part swap | 120 ms | batch 30 ms | `ease.out` | skeletons.md partial fill |
| Row append | 120 ms fade + 4 px rise | batch 30 ms | `ease.out` | |
| Leftover skeleton rows collapse | 200 ms | — | `spring.snap` | |
| Bar rise | 280 ms | 20 ms stagger, cap 12 | `ease.out` | proposed |
| Line draw | 600 ms | — | `ease.in-out` | proposed |
| Pie sweep | 500 ms | — | `ease.out` | |
| Tooltip | 80 ms in / 60 ms out | 60 ms hover intent | `ease.out` | crosshair unanimated |
| Legend series toggle / domain ease | 160 ms / 240 ms | — | `ease.out` / `spring.snap` | |
| Sort arrow / FLIP reorder | 160 ms / 200 ms | — | `ease.out` / `spring.snap` | FLIP only ≤ 50 visible rows |
| Checkbox column in | 160 ms | — | `ease.out` | |
| Bulk bar | spring | — | `spring.soft` | |
| Highlighter swipe (update) | 400 ms in, 1200 ms fade | — | `ease.out` | |
| Number roll | 400 ms | — | `spring.snap` | |
| Metric count-up (first reveal) | 500 ms | — | `ease.out` | |
| Delta chip | 120 ms | 120 ms after value | `ease.out` | |
| Timeline now-marker step | 200 ms per minute | — | `ease.in-out` | |
| Active event breathe | 2 s loop | — | `loop.calm` | |
| Map pin pulse | 300 ms | — | `spring.soft` | |
| Lightbox expand | 260 ms | — | `ease.in-out` | |
| **Reduced motion** | charts render final geometry at once; rows appear without rise; sort reorders instantly; highlighter becomes a static accent cell background for 3 s; numbers change without roll or count-up; no breathe (static dot); lightbox 80 ms crossfade | — | — | |

## 6. Variants

### 6.1 `genui-chart` — Free (3) / Pro (9)
| Name | Tier | Idea |
|---|---|---|
| `genui-chart/plain` | Free | Chart only, axes and legend, no chrome; inline in messages |
| `genui-chart/card` | Free | Card with title, period subtitle, headline value + delta above the plot, overflow menu |
| `genui-chart/spark-inline` | Free | 16–24 px tall sparkline that sits in text or a table cell, last point dotted |
| `genui-chart/scrub-insight` | Pro | Drag across the plot to select a range; agent-supplied annotations appear as pins with notes (finance "scrubbable insight charts", taxonomy section B) |
| `genui-chart/draw-on` | Pro | Expressive first reveal: axes draw, gridlines fade, series draw with a leading dot |
| `genui-chart/radial-gauge` | Pro | Radial tick gauge whose ticks fill one by one to the value ([R025]) |
| `genui-chart/compare` | Pro | Current vs previous period overlaid, previous dashed and muted, delta band between |
| `genui-chart/forecast-band` | Pro | Actuals solid, forecast dashed with an uncertainty band the agent supplies |
| `genui-chart/annotated-story` | Pro | Steps through the agent's annotations one at a time ("1 of 4") with the plot zooming to each |
| `genui-chart/realtime-stream` | Pro | Fixed time window that scrolls as points arrive, for live metrics |
| `genui-chart/braille` | Pro | Terminal: line and bar charts drawn in braille / block glyphs |
| `genui-chart/heatmap-calendar` | Pro | Activity heatmap by day (usage / activity bento idea, [R033], [R044]) |

### 6.2 `genui-table` — Free (3) / Pro (9)
| Name | Tier | Idea |
|---|---|---|
| `genui-table/plain` | Free | Header + rows, sort by click, pagination at 25 |
| `genui-table/data-grid` | Free | Sticky header and first column, resize, virtualised, multi-sort, column menu |
| `genui-table/compact-records` | Free | 28 px rows, person and status cells first, for CRM / ops panels |
| `genui-table/calc-footer` | Pro | Records table with per-column calc footer (sum / avg / count) that recomputes on filter |
| `genui-table/stream-rows` | Pro | Rows land one by one with a highlight trail as the agent emits them |
| `genui-table/changed-cells` | Pro | Highlighter swipe on cells the agent changed, "What changed" diff list ([R007]) |
| `genui-table/tree` | Pro | Grouped / nested rows with expand, group subtotals |
| `genui-table/pivot` | Pro | Rows × columns pivot of a flat result with a measure picker |
| `genui-table/bulk-actions` | Pro | Selection with floating bulk bar; bulk intents go through approvals ([R013]) |
| `genui-table/inline-edit` | Pro | Cells editable; edits collect as a proposed change set submitted as one intent |
| `genui-table/chart-this` | Pro | Column header action makes a `genui-chart` from that column next to the table |
| `genui-table/box-drawn` | Pro | Terminal: box-drawing table with truncation glyphs and keyboard paging |

### 6.3 `genui-metric` — Free (3) / Pro (6)
| Name | Tier | Idea |
|---|---|---|
| `genui-metric/tile` | Free | Label, big value, delta chip, period line |
| `genui-metric/inline` | Free | Value + delta inline with text, for prose answers |
| `genui-metric/strip` | Free | Row of 2–6 KPIs separated by dividers, wraps to a grid on narrow widths |
| `genui-metric/sparkbar` | Pro | Tile with a 12-bar sparkbar under the value ([R025]) |
| `genui-metric/odometer` | Pro | Digits roll on a split-flap / odometer drum when the value changes |
| `genui-metric/goal-ring` | Pro | Ring fills to value / target with the remaining amount in the centre |
| `genui-metric/delta-flash` | Pro | On update, the delta chip flashes its direction once and leaves a "since 2 min ago" trace |
| `genui-metric/board` | Pro | KPI board: tiles arranged in a responsive bento with one hero metric |
| `genui-metric/target-vs-actual` | Pro | Bullet bar: actual bar over target marker with qualitative bands |

### 6.4 `genui-list` — Free (3) / Pro (4)
| Name | Tier | Idea |
|---|---|---|
| `genui-list/plain` | Free | Bulleted or numbered text items |
| `genui-list/rich-rows` | Free | Leading media, title, subtitle, trailing meta and up to 2 actions |
| `genui-list/checklist` | Free | Items with check state the user can tick (ticks raise an `Answer` intent) |
| `genui-list/reorderable` | Pro | Drag handles to rank items; order submitted as an answer |
| `genui-list/grouped-sticky` | Pro | Section headers stick while scrolling long grouped lists |
| `genui-list/faq-grow` | Pro | Accordion where the user's typed question joins the list as a new item and is answered inline ([R057]) |
| `genui-list/card-stack` | Pro | Items as a swipeable stack of cards (mobile, voice) |

### 6.5 `genui-timeline` — Free (3) / Pro (5)
| Name | Tier | Idea |
|---|---|---|
| `genui-timeline/vertical` | Free | Rail with status dots, time on the leading side, expandable bodies |
| `genui-timeline/horizontal` | Free | Time axis with event markers and labels that thin to avoid overlap |
| `genui-timeline/log` | Free | Dense one-line-per-event log with timestamps, for IDE panels |
| `genui-timeline/swimlanes` | Pro | One lane per actor, blocks growing to a live "now" edge, hand-off connectors (agent lanes, research note 15) |
| `genui-timeline/metro` | Pro | Metro-line pipeline: items travel lines that branch to outcomes ([R038]) |
| `genui-timeline/gantt` | Pro | Duration bars with dependencies on a date axis |
| `genui-timeline/itinerary` | Pro | Day-grouped stops with times, places and travel legs (pairs with `genui-map`) |
| `genui-timeline/scrub-replay` | Pro | Playhead scrubber that replays events in order (ties into family F run replay) |

### 6.6 `genui-map` — Free (3) / Pro (4)
| Name | Tier | Idea |
|---|---|---|
| `genui-map/pin-list` | Free | No map: places as a numbered list with coordinates, distance and "Open in maps" link intents |
| `genui-map/static-image` | Free | App-provided base image with places projected as numbered pins |
| `genui-map/host-slot` | Free | Hands places and route to an app-registered map renderer; Oxyn draws only chrome and skeleton |
| `genui-map/route` | Pro | Ordered stops joined by a drawn path with leg durations |
| `genui-map/cluster` | Pro | Pins cluster into counts at low zoom and split on zoom |
| `genui-map/dot-matrix` | Pro | Stylised dot-matrix world or region from a bundled low-res land mask (source to confirm), pins as lit dots ([R011] look) |
| `genui-map/split-list` | Pro | Map and list side by side with linked selection |

### 6.7 `genui-media` — Free (3) / Pro (4)
| Name | Tier | Idea |
|---|---|---|
| `genui-media/image` | Free | Single image at its aspect with caption and alt |
| `genui-media/gallery` | Free | Grid of 2–12 images at a shared aspect, number keys pick |
| `genui-media/player` | Free | Video / audio player with captions and keyboard controls |
| `genui-media/lightbox` | Pro | Shared-element expand into a full-window viewer with filmstrip |
| `genui-media/compare-slider` | Pro | Before / after with a draggable divider |
| `genui-media/filmstrip` | Pro | Horizontal strip of frames or chapters with hover scrub |
| `genui-media/transcript-player` | Pro | Player with a word-synced transcript that highlights as it plays (family K pairing) |

## 7. Edge cases
- **Long text in cells / labels.** Single-line ellipsis; full text in tooltip and in "Copy as CSV";
  wrap toggle per column in `data-grid`.
- **Wide tables.** Horizontal scroll with sticky first column and edge fade masks; < 360 px container →
  rows render as cards (label: value pairs).
- **Mixed units / currencies in one column.** Validation warning; cells show their own currency code.
- **Negative and zero values.** Bars below the baseline; pie refuses negatives (validation error tile:
  "Pie charts can't show negative values · View as table").
- **Single data point.** Line chart becomes a dot with the value label; sparkline shows a flat dash.
- **Huge numbers / precision.** Abbreviate in axes and tiles, exact in tooltips and table cells;
  floats beyond 15 significant digits shown as received (string).
- **Time zones.** Times arrive in ISO 8601 with offset; displayed in the user's zone with the zone
  abbreviation in tooltips when it differs from the data's.
- **Streaming out of order** (row 40 before 39 via `add` at index): placed by index; FLIP not used for
  streaming insertions.
- **Agent patches while the user has sorted / filtered.** User sort and filter stay; updated rows move
  into their sorted position with the wash; pinned tooltip stays on the same item id.
- **Very fast agents.** Whole table in one patch: no row stream, one swap.
- **Very slow agents.** Structured header visible early; "Loading 40 of 120 rows" when n is known.
- **RTL.** Table columns mirror except numeric alignment rules; time axes follow `chart.time_direction`;
  timeline rail on the right.
- **Tiny sizes.** Sparkline min 48 × 16 px; metric tile min 120 px wide (value shrinks to 20 px type);
  chart below 200 × 120 drops axes labels and shows a summary line.
- **Touch.** Chart tooltip on tap (sticky until tap outside); horizontal swipe on a chart scrubs, not
  scrolls, only after a 150 ms hold (so the page still scrolls).
- **Dense IDE panels.** `compact-records`, `log`, `spark-inline` preferred; tooltips become status-bar
  text.
- **Terminal.** `braille`, `box-drawn`, `log`, `pin-list`; images as alt text.
- **Map without network.** Static and host-slot variants degrade to `pin-list`.
- **Private data in tooltips / exports.** Copy and export honour the app's redaction: masked values
  stay masked.

## 8. Accessibility
- **Chart.** Role `img` with the generated summary as its label plus a "View as table" button that
  renders the data as an accessible table; keyboard point navigation announces "September, Spend, €18.2k,
  up 12 %" (polite). Series distinguished by colour **and** dash pattern / marker shape (8 shapes).
- **Table.** Native table semantics with `scope` headers; sort state in `aria-sort`; virtualised rows
  keep `aria-rowcount` / `aria-rowindex`; selection count announced politely ("3 rows selected").
- **Metric.** Read as one phrase: "Total spend, €18,240, up 12 % versus last month, which is good"
  (direction meaning spelled out).
- **List / Timeline.** List semantics; timeline items announce status and time ("Deploy, done, 2 hours
  ago").
- **Map.** The place list is the accessible representation; the map image is decorative unless the
  host renderer provides its own accessibility.
- **Media.** `alt` required for images; captions toggle labelled; players fully keyboard operable; no
  autoplay with sound.
- **Updates.** "What changed" is reachable by keyboard; update announcements coalesced (≤ 1 per 2 s):
  "Table updated, 3 cells changed".
- **Contrast.** Gridlines are decorative (≥ 1.5:1 allowed); axis labels, cell text and delta chips meet
  4.5:1; status badges carry text, not only colour.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct ChartBlock {
    pub kind: ChartKind,                      // Line | Area { stacked: bool } | Bar { horizontal: bool, stacked: bool } | Pie | Donut | Sparkline
    pub series: Vec<Series>,                  // { key, label, points: Bind<Vec<Point>>, style: SeriesStyle }
    pub x: Axis, pub y: Axis,                 // Time | Category | Linear | Log, with format
    pub annotations: Vec<Annotation>,         // Pro scrub-insight / annotated-story
    pub summary: Option<SharedString>,        // agent-provided; else generated
    pub variant: ChartVariant,                // Plain | Card | SparkInline | Pro(..)
    pub max_points: usize,                    // 2_000 visible before downsampling (proposed)
}

pub struct TableBlock {
    pub columns: Vec<Column>,                 // { key, label, cell: CellType, width: Option<Pixels>, align, sortable, calc: Option<Calc> }
    pub rows: Bind<Vec<Row>>,                 // streamed via data.patch
    pub expected_rows: Option<u32>,           // for "40 of 120" and skeleton rows
    pub page_size: u16,                       // 25 (proposed)
    pub virtualise_above: u32,                // 200 (proposed)
    pub server_pages: Option<ActionId>,       // Query intent for more rows
    pub selection: Option<SelectionCfg>,      // { mode: Single | Multi, bulk: Vec<ActionId> }
    pub row_action: Option<ActionId>,
    pub variant: TableVariant,                // Plain | DataGrid | CompactRecords | Pro(..)
}

pub enum CellType { Text, Number, Currency { code }, Percent, Date, DateTime, Relative, Duration, Bytes,
                    Boolean, Status, Link, Person, Progress, Sparkline, Code, Json, Image }

pub struct MetricBlock { pub label: SharedString, pub value: Bind<f64>, pub format: NumberFormat,
                         pub delta: Option<Delta>, pub direction_good: Direction, pub period: Option<SharedString>,
                         pub sparkline: Option<Bind<Vec<f64>>>, pub target: Option<f64>, pub variant: MetricVariant }
pub struct ListBlock { pub items: ItemsSource, pub selectable: bool, pub item_action: Option<ActionId>, pub variant: ListVariant }
pub struct TimelineBlock { pub events: Bind<Vec<TimelineEvent>>, pub live_now: bool, pub variant: TimelineVariant }
pub struct MapBlock { pub places: Vec<Place>, pub route: Option<Vec<PlaceId>>, pub base: MapBase /* None | Image(AssetRef) | Host(RendererId) */, pub variant: MapVariant }
pub struct MediaSetBlock { pub items: Vec<MediaItem>, pub variant: MediaVariant }

pub enum BlockPhase { Skeleton, Structured, Filling { have: u32, expected: Option<u32> }, Ready,
                      Inspecting(ItemId), Selecting(u32), Updated { at: Instant }, Fetching, Empty, Error(BlockError), Truncated { shown: u32, total: u32 } }
// Drive: GenNode life (genui.md §2.8) + data patches; AgentState::Stalled → skeleton stall; intents → renderer.
```

## 10. Related components and references
Related: [genui.md](genui.md) · [genui-renderer.md](genui-renderer.md) · [genui-skeletons.md](genui-skeletons.md)
· [skeletons.md](skeletons.md) · [streaming-markdown.md](streaming-markdown.md) (markdown tables;
`markdown/table-studio` "Chart this" hand-off) · [media-generation-placeholder.md](media-generation-placeholder.md)
· [run-replay.md](run-replay.md) (`scrub-replay`) · [approvals.md](approvals.md) (bulk and edit intents) ·
[elapsed-timer.md](elapsed-timer.md) (relative times).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Thesys C1 chart and table families; note 05 recommended chart set | research note 05 §1 | type coverage | not recorded: concepts only |
| Arc charts (line, bar, donut, sparkline, gauge, activity heatmap…) and tables (sortable data table, tree table, JSON viewer) | research note 12 | coverage check | Paid / unclear: study only |
| affectUI Data Table, KPI Chart, Breakdown Chart, Metrics Dashboard, KPI Board (Pro items) | research note 11 | coverage check only | Paid / unclear: study only |
| Fintech dashboard: KPIs with sparkbars, radial tick gauge drawing in | [R025] | `sparkbar`, `radial-gauge` | Study only |
| Highlighter on changed numbers / cells | [R007] | update wash, `changed-cells` | Study only |
| Floating bulk bar on first selection | [R013] | selection bar | Study only |
| Metro-line triage pipeline | [R038] | `genui-timeline/metro` | Study only |
| Agent lanes swimlane with live now edge | research note 15 | `swimlanes` | Study only |
| FAQ accordion where a typed question joins the list | [R057] | `faq-grow` | Study only |
| Pitus Gen UI #1 Map card | [R056] | map block exists as a gen-UI pattern | Study only |
| Activity / usage heatmap bentos | [R033], [R044] | `heatmap-calendar` | Study only |
| ASCII / dot-matrix globe | [R011] | `dot-matrix` look | Study only |
| Codex table holdback: header held until its delimiter row; table is a mutable tail | research note 08 | `Structured` before `Filling` | MIT: may port with attribution |
| easyui Live Line Chart, Activity Heatmap | research note 17, [R131] | realtime and heatmap behaviour | MIT: may port with attribution |
| Beautiful UI Records Table, Filter Table, Insight Cards | [R106], [R133] | records and insight patterns | MIT: may port with attribution |

**Gaps.** No GPUI charting crate was evaluated for this draft (we assume our own path-based renderer);
map tile licensing and an embeddable Rust map renderer were not researched; the land-mask source for
`dot-matrix` must be a public-domain dataset, to be chosen.
