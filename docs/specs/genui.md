**Status:** draft v0.1 (2026-11-12).

# Spec: generative UI (family I overview)

Overview of family **I** in [../component-taxonomy.md](../component-taxonomy.md) (target **46 skeletons +
blocks**). Follows the 10-point standard in [README.md](README.md) at the depth of
[composer-send-control.md](composer-send-control.md). Motion tokens are the shared ones in
[composer.md](composer.md) §"Motion tokens" and the loader tokens in [loaders.md](loaders.md) §5; skeleton
timing is owned by [skeletons.md](skeletons.md) (family D) and reused, never redefined, here. Any action
with a side effect is resolved by family G ([approvals.md](approvals.md)). All values marked *proposed*
are Oxyn's own starting points, to be tuned in the Studio. Facts about other products come from the
research notes named beside them (note numbers refer to the project research folder; `[Rxxx]` IDs are in
[../research/sources.csv](../research/sources.csv)).

Specs in this family:

| Spec | Component(s) | Variants (free + Pro) |
|---|---|---|
| this file | schema (`oxyn.genui/1`), the 18 primitives, validation, fallback, versioning, streaming model, security model; `genui-kit` (primitive presentation sets) | 3 + 8 |
| [genui-renderer.md](genui-renderer.md) | `genui-surface` (renderer + inline / side-panel / fullscreen / PiP containers, progressive streaming, per-node error boundary, "generating this UI" wireframe, regenerate, edit-in-place) | 4 + 12 |
| [genui-blocks.md](genui-blocks.md) | `genui-chart`, `genui-table`, `genui-metric`, `genui-list`, `genui-timeline`, `genui-map`, `genui-media` | 21 + 41 |
| [genui-forms.md](genui-forms.md) | `genui-form`, `genui-question` (human-in-the-loop question cards) | 6 + 16 |
| [genui-skeletons.md](genui-skeletons.md) | `genui-skeleton` (composition strategy) + the **46-cell block × sweep matrix** | 3 + 8 (matrix: 18 free + 28 Pro cells) |
| **Total** | | **37 free + 85 Pro = 122 variants**, plus **46 skeleton presets** |

The catalogue of every named variant is in §6.2.

## 1. Purpose
Some answers are better as interface than as prose: a budget as a table with a total, a comparison as a
chart, a set of choices as a form, a trip as a timeline, five candidate images as a grid you can pick
from. In an agent app the agent decides **mid-run** that it needs one of these, emits a description of
it, and the client draws it with the app's own components, theme and accessibility. The user wants:
1. **To see the right thing appear in the right place without the transcript jumping**, starting with
   a shape of what is coming (skeleton or "generating this UI" wireframe) and filling in as data
   streams.
2. **To trust it.** It looks and behaves like the rest of the app; it cannot run code, phish for a
   password, or press a button on their behalf.
3. **To act on it.** Sort, filter, pick, fill in, submit, and know what each button really does
   before it does it.
4. **To change it.** Ask for a different chart, edit one block in place, go back to the previous
   version.

What we saw (research note 05 §1): the field ranges from **controlled** (the app pre-registers a
component per tool and the agent fills its props: Vercel AI SDK UI `tool-${name}` parts, Tambo's
registered components with Zod schemas, CopilotKit's components-as-tools) through **declarative** (the
agent emits a tree drawn from a client-supplied catalogue: Google A2UI, whose basic catalogue has 18
components — Text, Image, Icon, Video, AudioPlayer, Row, Column, List, Card, Tabs, Modal, Divider,
Button, TextField, CheckBox, ChoicePicker, Slider, DateTimeInput — plus functions such as required,
regex, email, formatCurrency / formatDate, pluralize, openUrl; Thesys C1's UI DSL with charts, forms,
tables, slides and reports) to **open-ended** (MCP Apps / OpenAI Apps SDK ship sandboxed HTML in an
iframe with display modes inline / fullscreen / PiP). CopilotKit frames it as Controlled (AG-UI) →
Declarative (A2UI) → Open-ended. AG-UI carries A2UI surfaces as `ACTIVITY_SNAPSHOT` events and state
updates as JSON Patch (`STATE_DELTA`). Tambo distinguishes **generative** components (new instance per
message) from **interactable** ones (pre-placed, updated in place by id) and calls out that components
must render gracefully with props still undefined while streaming.

**Oxyn's position (decision, proposed):** Oxyn is a Rust UI library, and GPUI cannot host a web view, so
family I is **declarative-first**: one schema, `oxyn.genui/1`, with **our own 18 primitives** and **8
composite blocks**, rendered natively by `oxyn-gpui` and `oxyn-dioxus`. Adapters map A2UI's basic
catalogue, AG-UI activity events and tool-result props onto it. Registered app components ("controlled"
genUI) plug in as **custom catalogue entries** with a Rust props schema. Open-ended HTML (MCP Apps) is a
**gap**: Dioxus web could host a sandboxed iframe; GPUI has no path today (§7, §10).

## 2. The model (`oxyn-core::genui`, proposed)

### 2.1 Surface, nodes, data, actions
A **surface** is one generated UI: a tree of nodes, a data model the nodes bind to, and the actions its
buttons and forms may raise. Nodes are sent as a **flat map keyed by id** with a `root` id, not as a
nested tree (decision, proposed): a flat map lets a stream add a child before or after its parent, lets
patches address any node in O(1), and lets a node referenced before it is defined render as a skeleton.

```json
{
  "schema": "oxyn.genui/1.0",
  "surface": "q3-spend",
  "catalog": ["oxyn.basic@1", "oxyn.blocks@1", "acme.crm@2"],
  "root": "root",
  "nodes": {
    "root":  { "type": "Stack", "props": { "axis": "vertical", "gap": "md" }, "children": ["title", "kpis", "tbl", "cta"] },
    "title": { "type": "Text", "props": { "text": "Q3 team spend", "role": "heading", "level": 2 } },
    "kpis":  { "type": "Metric", "props": { "label": "Total", "value": { "$bind": "/total" }, "format": { "currency": "EUR" } } },
    "tbl":   { "type": "Table", "props": { "rows": { "$bind": "/rows" }, "columns": [ { "key": "who", "label": "Person" },
               { "key": "amount", "label": "Amount", "cell": "currency", "currency": "EUR" } ] } },
    "cta":   { "type": "Button", "props": { "label": "Approve all", "action": "approve_selected", "style": "primary" } }
  },
  "data": { "total": 18240.5, "rows": [] },
  "actions": { "approve_selected": { "intent": "expenses.approve", "args": { "ids": { "$bind": "/selection" } } } }
}
```

Wire messages (one surface, any transport; proposed names):

| Message | Payload | Effect |
|---|---|---|
| `surface.begin` | `schema`, `surface`, `catalog`, optional `layout_hint` (block shapes for the wireframe), `container` request | creates the surface in `Wireframe` (renderer §2) |
| `nodes.upsert` | one or more `{id: node}` (partial props allowed) | adds or merges nodes; merges are shallow per prop |
| `nodes.remove` | ids | removes nodes (children re-parented to nothing; dangling refs render nothing) |
| `node.done` | id | marks a node's props final (its skeleton may swap) |
| `data.patch` | JSON Patch operations (RFC 6902) on `data` | updates bound values (AG-UI `STATE_DELTA` maps here) |
| `surface.commit` | — | everything sent so far is final; remaining skeletons resolve to empty or error |
| `surface.end` | `reason`: `done` / `cancelled` / `error` | closes the stream; the surface stays interactive |

### 2.2 Values and binding
- A prop is a literal or `{ "$bind": "/json/pointer" }` into `data`. Inside a `List` or `Table` item
  template, `{ "$item": "/field" }` points into the current item.
- Formatting is **declarative**: `format: { number | currency | percent | date | time | relative |
  bytes | duration, locale?: "auto" }`, resolved by the client with the user's locale (A2UI-style
  `formatCurrency` / `formatDate` functions, note 05). There is no expression language (§2.6).
- Theme values are **tokens only**: `gap: "xs|sm|md|lg|xl"`, `tone: "neutral|info|success|warning|
  danger|accent"`. A colour string, pixel size or CSS string is a validation warning and is ignored.

### 2.3 The 18 primitives (`oxyn.basic@1`, proposed)
Our own set; the A2UI basic catalogue is the coverage check, not the source (licence §10). All
primitives accept `id`, `visible: bool | $bind`, `a11y_label?`, and `fallback_text?` (§2.5).

| # | Primitive | Group | Props (beyond the common ones) | A2UI basic catalogue item it absorbs |
|---|---|---|---|---|
| 1 | `Stack` | layout | `axis: vertical\|horizontal`, `gap`, `align: start\|center\|end\|stretch`, `justify: start\|center\|end\|between`, `wrap: bool`, `children` | Row, Column |
| 2 | `Grid` | layout | `columns: 1..=6` or `min_column: 160..=480` px (auto-fit), `gap`, `children` | — |
| 3 | `Card` | layout | `title?`, `subtitle?`, `tone`, `leading_icon?`, `media?: node id`, `actions?: [action id] (≤ 3)`, `children` | Card |
| 4 | `Tabs` | layout | `tabs: [{ id, label, icon?, child }]` (2–8), `selected?` | Tabs |
| 5 | `Disclosure` | layout | `summary`, `open: bool = false`, `child` | (replaces Modal, §2.6 rule 6) |
| 6 | `Divider` | layout | `axis`, `label?` | Divider |
| 7 | `Text` | content | `text`, `role: body\|heading\|caption\|label\|code\|quote`, `level: 1..=4`, `tone?`, `max_lines?`, `inline_markdown: bool = true` (bold, italic, code, link only) | Text |
| 8 | `Icon` | content | `name` (Oxyn icon map; covers every name in A2UI's icon enum, note 05), `size: 12\|16\|20\|24`, `tone?`, `label?` (absent = decorative) | Icon |
| 9 | `Image` | content | `src` (https URL or asset ref), `alt` (**required**), `aspect?: "16:9"…`, `fit: cover\|contain`, `caption?` | Image |
| 10 | `Media` | content | `kind: video\|audio`, `src`, `poster?`, `captions?`, `duration?`; autoplay is not a prop (never autoplays with sound) | Video, AudioPlayer |
| 11 | `Badge` | content | `text`, `tone`, `icon?` | — |
| 12 | `List` | content | `items: [node id]` or `items_bind` + `item` template, `ordered: bool`, `dense: bool`, `dividers: bool` | List |
| 13 | `Button` | input | `label`, `action: action id`, `style: primary\|secondary\|ghost\|danger`, `icon?`, `disabled_reason?` | Button |
| 14 | `TextField` | input | `name`, `label`, `kind: text\|multiline\|number\|email\|url\|phone`, `placeholder?`, `default?`, `required`, `min_len?`, `max_len?` (cap 10 000), `pattern?` (§2.4), `help?` | TextField |
| 15 | `Toggle` | input | `name`, `label`, `style: checkbox\|switch`, `default: bool` | CheckBox |
| 16 | `Choice` | input | `name`, `label`, `options: [{ value, label, description?, recommended?: bool }]` (1–50), `multiple: bool`, `display: radio\|chips\|select\|segmented`, `min?`, `max?`, `allow_other: bool` | ChoicePicker |
| 17 | `Slider` | input | `name`, `label`, `min`, `max`, `step`, `default`, `range: bool`, `format?` | Slider |
| 18 | `DateTime` | input | `name`, `label`, `mode: date\|time\|datetime\|date_range`, `min?`, `max?`, `default?`, `tz?` (display only) | DateTimeInput |

### 2.4 Blocks (`oxyn.blocks@1`, proposed)
Composites that a primitive tree would express badly or inaccessibly. Specified in
[genui-blocks.md](genui-blocks.md) and [genui-forms.md](genui-forms.md).

| Block | Spec | One-line role |
|---|---|---|
| `Chart` | blocks | line, area, bar (grouped / stacked), pie / donut, sparkline; "view as table" built in |
| `Table` | blocks | typed cells, sort, paginate, virtualise, selection → intents |
| `Metric` | blocks | KPI value + delta + period + optional sparkline |
| `Timeline` | blocks | ordered events with time, status and actor |
| `Map` | blocks | places and routes; a placeholder + host slot (no tile engine shipped) |
| `MediaSet` | blocks | image gallery, video / audio player with captions |
| `Form` | forms | fields + validation + submit through an action |
| `QuestionCard` | forms | human-in-the-loop questions, paged "1 of 3", recommended option |

Custom catalogue entries (`acme.crm@2`) are app components registered with a Rust props schema; they
obey every rule here (validation, binding, intents) and get the generic skeleton unless they declare a
`SkeletonBlock` ([genui-skeletons.md](genui-skeletons.md)).

### 2.5 Validation and the unknown-component fallback
Validation runs on every `nodes.upsert` and `data.patch`, **per node**, so one bad node never blanks a
surface (per-node error boundaries: [genui-renderer.md](genui-renderer.md) §2).

| Check | Limit (proposed) | On failure |
|---|---|---|
| Schema major version | must equal client's (`1`) | whole surface → **degrade to text** (§2.7) |
| Node count / depth | ≤ 500 nodes, depth ≤ 12 | nodes past the limit are dropped; one "Some content was too large to show" row |
| Required prop missing | e.g. `Image.alt`, `Button.action`, `TextField.name` | while streaming: node stays skeleton; after `node.done` / commit: node error tile |
| Prop type mismatch | — | prop ignored, default used, dev warning; node error tile if the prop is required |
| Unknown prop | — | ignored (forward compatibility), dev warning |
| String length | `Text` ≤ 10 000 chars, labels ≤ 120 chars (ellipsis + full text in tooltip) | truncated, "Show more" |
| `pattern` regex | linear-time subset (no backreferences or lookaround), ≤ 256 chars | pattern dropped, field still validates other rules |
| URL scheme | `https`, `mailto`, app asset refs; `http` only if the host's `LinkPolicy` allows | link rendered as plain text |
| Action id | must exist in `actions` **and** its `intent` in the app's `IntentRegistry` | button rendered disabled with reason "This action isn't available" |
| Secret-shaped field | `TextField` named or labelled like a password, API key, token, card number, CVV or SSN | converted to a **Secure input** hand-off ([genui-forms.md](genui-forms.md) §2.4) |
| Data size | table rows ≤ 10 000 per surface, chart points ≤ 5 000 per series | rows beyond the cap dropped with a count; chart downsampled (largest-triangle) |

**Unknown component fallback ladder** (decision, proposed): (1) if the node has `fallback_text`, render
it as `Text`; (2) else if it has `children`, render them in a vertical `Stack`; (3) else if the catalogue
id is known but the version is newer, render the closest known version and ignore unknown props;
(4) else render a quiet **"Unsupported: `Foo`"** chip (with a dev-only "View data" disclosure showing
the redacted props as JSON). The agent is told (via the adapter's error channel) which types failed, so
it can resend in supported types.

### 2.6 Security model
Generated UI is **untrusted content from a model** that may be prompt-injected. Rules (all decisions,
proposed):
1. **No code.** No scripts, no HTML, no CSS strings, no expressions, no event handlers. The only logic is
   declared validation (§2.5) and formatting (§2.2).
2. **Actions are intents, not calls.** A button or form names an action; the action names an `intent`
   from the app's `IntentRegistry` with typed args. The agent cannot invent intents. The client never
   performs an effect itself; it raises the intent to the app.
3. **The label is the agent's, the consequence is the app's.** Every intent in the registry declares a
   `kind` (`Answer`, `Navigate`, `Query`, `Mutate`, `Spend`, `Send`, `Credential`, `Custom`) and a risk
   floor. `Answer` / `Query` / `Navigate` (inside the app) run directly; **any other kind becomes an
   `ApprovalRequest`** of the matching family-G kind ([approvals.md](approvals.md) §2.2: `Spend`,
   `Network`, `McpTool`, `Credential`, `Custom`), and the approval card shows the **registry's**
   description of the effect, not the button's label. A button saying "View invoice" whose intent is
   `payments.send` gets a `Spend` card.
4. **Never auto-submit.** A surface cannot press its own buttons, submit its own forms or move focus
   into a field. Agent-supplied defaults are marked as suggestions ([genui-forms.md](genui-forms.md)).
5. **No secrets typed into generated UI.** Secret fields are refused and replaced by the host's secure
   flow (§2.5); typed values that look like secrets are blocked before submit.
6. **No focus theft.** There is no Modal primitive: an agent cannot open a blocking overlay. Fullscreen
   and PiP are **requests** the user grants ([genui-renderer.md](genui-renderer.md) §2).
7. **External links** show the destination host on hover / focus and an external glyph (shared
   `LinkPolicy` with [streaming-markdown.md](streaming-markdown.md) §9); remote images follow its
   `ImagePolicy` (Load / BlockRemote / AskFirst).
8. **Provenance.** Every surface carries a "Generated by <agent>" affordance in its container chrome
   (an `Icon` + tooltip), so a generated form is never mistaken for app chrome.

### 2.7 Versioning
- `schema: "oxyn.genui/<major>.<minor>"`. Minor versions only add optional props, primitives and
  messages; majors may remove or rename.
- The client advertises `{ schema: "1.x", catalogs: [{ id, version, types }] }` to the agent (via the
  adapter's capabilities message or the system prompt the app builds). The agent should emit only
  advertised types; anything else hits the fallback ladder.
- **Major mismatch → degrade to text**: the surface renders the agent's `fallback_text` for the root
  (or the concatenated `fallback_text` of its nodes) as markdown, with a chip "Shown as text (UI format
  not supported)".
- Released variant names and catalogue ids are stable (variants.md naming rule).

### 2.8 Streaming partial trees
Per-node lifecycle (the renderer's machine is in [genui-renderer.md](genui-renderer.md) §2):

```
  child id seen in a parent, node not yet sent
  (none) ───────────────────────────────────────▶ Referenced ── skeleton from parent hint / generic
                         nodes.upsert (partial)     │
  (none) ──────────────────────────────────────────▶ Partial ── required props missing → skeleton
                                                    │           optional props missing → render without them
                                                    │ required props present
                                                    ▼
                                                 Renderable ── renders; more props / data may still merge
                                                    │ node.done | surface.commit | surface.end(done)
                                                    ▼
                                                  Final ── invalid now → NodeError tile (same footprint)
  Referenced at commit → empty (nothing rendered, footprint collapses)
```
- Patches are **coalesced per frame** (one layout pass per frame at most; proposed 16 ms at 60 Hz).
- Structural arrivals (a new node becoming renderable) are staggered **30 ms, cap 6** so a burst reads as
  a cascade, not a flash (composer `stagger` token).
- Tables and lists may stream rows through `data.patch` `add` ops at `/rows/-`; rows arriving < 30 ms
  apart batch into one crossfade ([skeletons.md](skeletons.md) §4 partial fill).
- A node's footprint is reserved from its skeleton; when it becomes renderable, the swap follows
  [skeletons.md](skeletons.md) (crossfade 180 ms, height settle 200 ms `spring.snap`, top edge fixed).

## 3. States table (surface level)
The full renderer machine is in [genui-renderer.md](genui-renderer.md) §2; this table is the summary the
other family-I specs reference.

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Wireframe` | "generating this UI" wireframe from `layout_hint` or default composition | Stop (composer) | — | Esc stops run (composer focus) |
| `Streaming` | real nodes in place, unarrived nodes as skeletons | — | Stop | Tab enters rendered interactive nodes only |
| `Ready` | complete surface | the surface's own | Regenerate, Edit, container controls | Tab order = tree order |
| `Submitting` | the submitting node (form / button) busy; rest interactive | — | Cancel (if intent cancellable) | — |
| `AwaitingApproval` | the gated button / form shows the amber hand glyph; family-G card attached | answer card | — | family G keys |
| `Degraded` | text fallback + chip | — | "Try again as UI" (regenerate) | — |
| `PartialError` | rest of the surface + per-node error tiles | Retry node | Report | Enter on tile retries |
| `Failed` | surface-level error card in the wireframe footprint | Retry | Show as text | Enter |
| `Stale` | surface dimmed 60 %, actions disabled: "Superseded by a newer version" | Open latest | — | — |
| `Cancelled` | nodes so far stay; skeletons fade; chip "Stopped" | Regenerate | — | — |

## 4. Micro-interactions (family level)
- **One light per surface.** All skeletons in a surface share one viewport-pinned sweep
  ([skeletons.md](skeletons.md) §4). The renderer adds no second loader; family B's busy signal lives in
  the status strip, not inside the surface ([loaders.md](loaders.md) §2.3 "one loader animates per
  region").
- **Arrive, don't pop.** Node swap = skeleton crossfade + height settle with the top edge fixed; in a
  bottom-pinned transcript the scroll anchor compensates.
- **Agent updates in place** (interactable surfaces and `data.patch` after `Ready`): changed nodes get a
  brief accent wash (12 % alpha, fades over 1200 ms) and changed numbers roll; a "Updated by agent ·
  just now" tick appears in the container chrome (Tambo's "updated by agent" highlight idea, note 05;
  highlighter swipe on changed numbers and cells, [R007]).
- **Intent press.** Buttons press-down 0.97 scale (80 ms); intents that route to approvals morph the
  button label to "Waiting for approval" with the amber hand glyph ([approvals.md](approvals.md) §4),
  never a spinner that suggests the effect is already happening.
- **Errors stay local.** A failing node shakes nothing; its error tile crossfades in at its footprint.

## 5. Timing (family defaults; each spec may override)

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Patch coalescing | 1 frame (16 ms @ 60 Hz) | — | — | proposed |
| Node arrival cascade | 180 ms fade (skeletons.md swap) | 30 ms stagger, cap 6 | `ease.out` | composer `stagger` |
| Skeleton show delay / min display | 150 ms / 300 ms incl. exit | — | — | owned by skeletons.md |
| Height settle on swap | 200 ms | — | `spring.snap` | top edge fixed |
| Agent-update wash | 1200 ms fade-out from 12 % alpha | — | `ease.out` | proposed |
| Number roll on update | 400 ms | — | `spring.snap` | Metric, Table cells |
| Intent press | 80 ms down / 120 ms up | — | `ease.out` | |
| Container change (inline ↔ panel ↔ fullscreen) | 260 ms | — | `ease.in-out` | `dur.travel`; renderer §5 |
| Error tile in | 160 ms | — | `ease.out` | |
| Degrade-to-text swap | 180 ms crossfade | — | `ease.in-out` | |
| **Reduced motion** | no sweep or cascade; swaps are 80 ms opacity crossfades; height changes jump once; agent-update wash replaced by a static 2 px accent bar for 3 s; numbers change without rolling | — | — | `motion.reduced` token |

## 6. Variants

### 6.1 `genui-kit` (primitive presentation sets)
A kit changes the **structure** primitives render with (density, label placement, chrome), per
[../variants.md](../variants.md); colour and radius stay tokens.

**Free (3)**
| Name | Idea |
|---|---|
| `genui-kit/plain` | Chat default: borderless primitives on the message surface, 14 px body, labels above fields, cards with 1 px outline |
| `genui-kit/dense` | IDE panels: 28 px rows, 12 px type, inline labels left of fields, cards become sections with a header rule |
| `genui-kit/terminal` | TUI look: box-drawing frames, mono type, `[ ]` / `(•)` controls, images render as `alt` text in a frame |

**Pro (8)**
| Name | Idea |
|---|---|
| `genui-kit/editorial` | Report layout: wide measure, numbered figures and tables with captions, pull-out metrics in the margin |
| `genui-kit/bento` | `Grid` auto-spans cards by content weight (metric 1×1, chart 2×1, table 2×2) into a bento board |
| `genui-kit/mobile-sheet` | Touch: 44 px targets, stacked fields, sticky bottom action bar, swipeable `Tabs` |
| `genui-kit/blueprint` | Technical: mono size labels on blocks, dashed outlines on containers, for app builders ([R005] look) |
| `genui-kit/voice-cards` | Voice surface: one card at a time at large type, "Next / Back" by voice or swipe |
| `genui-kit/print` | Paginated A4 / Letter layout for export, tables split with repeated headers |
| `genui-kit/canvas` | Nodes placed on an infinite canvas with connectors (app-builder and planning boards) |
| `genui-kit/chips` | Every block collapses to a chip in a sidebar and expands on click; for narrow side panels |

### 6.2 Family catalogue (every named variant in family I)

| Component | Spec | Free | Pro |
|---|---|---|---|
| `genui-kit` | this file | `plain`, `dense`, `terminal` | `editorial`, `bento`, `mobile-sheet`, `blueprint`, `voice-cards`, `print`, `canvas`, `chips` |
| `genui-surface` | renderer | `inline`, `side-panel`, `fullscreen`, `pip` | `carousel`, `split-preview`, `blueprint-build`, `aurora-build`, `cascade`, `versions-rail`, `inspector-edit`, `pinned`, `diff-regenerate`, `device-frames`, `docked-pill`, `stage` |
| `genui-chart` | blocks | `plain`, `card`, `spark-inline` | `scrub-insight`, `draw-on`, `radial-gauge`, `compare`, `forecast-band`, `annotated-story`, `realtime-stream`, `braille`, `heatmap-calendar` |
| `genui-table` | blocks | `plain`, `data-grid`, `compact-records` | `calc-footer`, `stream-rows`, `changed-cells`, `tree`, `pivot`, `bulk-actions`, `inline-edit`, `chart-this`, `box-drawn` |
| `genui-metric` | blocks | `tile`, `inline`, `strip` | `sparkbar`, `odometer`, `goal-ring`, `delta-flash`, `board`, `target-vs-actual` |
| `genui-list` | blocks | `plain`, `rich-rows`, `checklist` | `reorderable`, `grouped-sticky`, `faq-grow`, `card-stack` |
| `genui-timeline` | blocks | `vertical`, `horizontal`, `log` | `swimlanes`, `metro`, `gantt`, `itinerary`, `scrub-replay` |
| `genui-map` | blocks | `pin-list`, `static-image`, `host-slot` | `route`, `cluster`, `dot-matrix`, `split-list` |
| `genui-media` | blocks | `image`, `gallery`, `player` | `lightbox`, `compare-slider`, `filmstrip`, `transcript-player` |
| `genui-form` | forms | `stacked`, `inline`, `side-panel` | `stepper`, `conversational`, `review-diff`, `prefill-sparkle`, `two-column`, `scheduler`, `sheet-mobile`, `box-drawn`, `repeat-rows`, `draft-review` |
| `genui-question` | forms | `numbered`, `paged`, `chips` | `inbox`, `scale`, `rank`, `image-choice`, `voice`, `countdown-default` |
| `genui-skeleton` | skeletons | `per-node`, `surface-wireframe`, `static` | `blueprint-labels`, `build-log`, `tree-cascade`, `progressive-detail`, `aurora-region`, `ghost-previous`, `glyph`, `count-hint` |
| **Totals** | | **37** | **85** |

Skeleton presets (block × sweep): **46** = 18 free + 28 Pro cells, listed in
[genui-skeletons.md](genui-skeletons.md) §6.1. Not yet specified: a `Slides` block (named in the
taxonomy row) — see §10 gaps.

## 7. Edge cases (family level)
- **Agent sends prose and a surface for the same content.** The app chooses; the default renders the
  surface and collapses the duplicate prose under "Text version" (useful for copy and screen readers).
- **Very fast agents** (whole surface in < 150 ms): no wireframe, no skeletons; one 180 ms fade-in.
- **Very slow agents** (minutes): wireframe and skeletons persist; `Stalled` from `oxyn-core` slows the
  sweep and shows "Still generating…" after 5 s without a patch (status strip thresholds).
- **Stop mid-stream.** Rendered nodes stay usable; skeletons fade (160 ms); `Referenced` nodes vanish;
  forms with missing required fields cannot submit ("Incomplete form — ask the agent to finish").
- **Out-of-order patches** (child before parent; data before the table node): held in the flat map
  and placed when their parent arrives; data patches against missing paths create them.
- **Duplicate ids** in one upsert: last wins, dev warning.
- **Huge data.** Tables virtualise beyond 200 rows; charts downsample above 2 000 visible points; caps
  in §2.5.
- **Locale.** Formatting uses the user's locale, never the agent's assumption; ambiguous dates
  (`03/04`) must arrive as ISO 8601 or they fail validation for `DateTime`.
- **RTL.** Stacks mirror; charts keep time running left → right only if the app's locale convention says
  so (token `chart.time_direction`, default follows reading direction); numbers stay LTR.
- **Tiny containers** (PiP 320 × 200, side panel 280 px). Grids collapse to one column; tables become
  card lists below 360 px; charts drop legend into a toggle.
- **Terminal surface.** No images or charts with fills: charts render as braille or block glyphs
  (`genui-chart/braille`), images as alt text.
- **Offline.** Intents that need the network are disabled with reason "Offline"; nothing is queued for
  later (fail safe, matching [approvals.md](approvals.md) §3).
- **Embedded HTML apps (MCP Apps).** Not rendered by family I; the adapter shows a link card "This app
  needs a web view" unless the Dioxus host opts into a sandboxed iframe (gap).

## 8. Accessibility (family level)
- **Native semantics.** Each primitive maps to its platform role (button, checkbox, radio group,
  slider, tablist, table with headers, list). Generated UI is never a picture of UI.
- **Container label.** The surface is a `region` labelled "<surface title>, generated by <agent>";
  while streaming it is `aria-busy="true"` with "Generating interface" and skeletons are `aria-hidden`
  ([skeletons.md](skeletons.md) §8).
- **Announcements.** Surface ready: polite "Interface ready: Q3 team spend, 1 table, 1 button" (only if
  the user started this run from this view). Agent updates after Ready: polite "Updated: Total" (at most
  one announcement per 2 s, coalesced). Errors: assertive for submit failures only.
- **Required alt text.** `Image.alt` is required (§2.5); charts expose "View as table" and a generated
  summary sentence; `Icon` without `label` is decorative.
- **Focus.** Never moved by the agent (§2.6 rule 6). Tab order is tree order; a new node arriving
  before the focused one does not steal or shift focus.
- **Contrast.** Tones resolve through tokens checked at 4.5:1 for text and 3:1 for controls; an agent
  cannot set colours.
- **Keyboard.** Every interactive primitive is operable by keyboard; family-G keys apply inside
  approval cards attached to intents.
- **Reduced motion:** §5 last row.

## 9. Props sketch (shared model)
```rust
pub struct GenSurface {
    pub id: SurfaceId,
    pub schema: SchemaVersion,                 // 1.0
    pub catalogs: SmallVec<[CatalogRef; 4]>,   // oxyn.basic@1, oxyn.blocks@1, app catalogs
    pub root: Option<NodeId>,
    pub nodes: IndexMap<NodeId, GenNode>,      // flat map, insertion order kept for the cascade
    pub data: serde_json::Value,               // bound data model
    pub actions: HashMap<ActionId, ActionDecl>,
    pub phase: SurfacePhase,                   // Wireframe | Streaming | Ready | Submitting | AwaitingApproval
                                               // | Degraded | PartialError | Failed | Stale | Cancelled (renderer §2)
    pub layout_hint: Option<Vec<SkeletonBlock>>,
    pub provenance: AgentRef,
    pub version: u32,                          // regenerate / edit bumps it (renderer §2)
}

pub struct GenNode {
    pub ty: NodeType,                          // Primitive(PrimitiveKind) | Block(BlockKind) | Custom { catalog, name }
    pub props: PropMap,                        // Literal(Value) | Bind(JsonPointer) | Item(JsonPointer)
    pub children: SmallVec<[NodeId; 8]>,
    pub life: NodeLife,                        // Referenced | Partial | Renderable | Final | Error(NodeError)
    pub fallback_text: Option<SharedString>,
}

pub enum PrimitiveKind {
    Stack, Grid, Card, Tabs, Disclosure, Divider, Text, Icon, Image, Media, Badge, List,
    Button, TextField, Toggle, Choice, Slider, DateTime,
}
pub enum BlockKind { Chart, Table, Metric, Timeline, Map, MediaSet, Form, QuestionCard }

pub struct ActionDecl { pub intent: IntentId, pub args: PropMap }

pub struct IntentSpec {                        // app-owned registry entry; the agent cannot add these
    pub id: IntentId,                          // "expenses.approve"
    pub kind: IntentKind,                      // Answer | Query | Navigate | Mutate | Spend | Send | Credential | Custom
    pub describe: fn(&serde_json::Value) -> SharedString, // the consequence shown on approval cards
    pub risk_floor: Option<RiskTier>,          // family G tiers
    pub to_approval: Option<fn(&serde_json::Value) -> ApprovalKind>, // None = runs directly (Answer/Query/Navigate)
    pub cancellable: bool,
}

pub enum GenEvent {                            // adapters (A2UI, AG-UI ACTIVITY_*, tool parts) produce these
    Begin { surface: SurfaceId, schema: SchemaVersion, catalogs: Vec<CatalogRef>, hint: Option<Vec<SkeletonBlock>>, container: Option<ContainerRequest> },
    Upsert(Vec<(NodeId, GenNodePatch)>), Remove(Vec<NodeId>), NodeDone(NodeId),
    DataPatch(json_patch::Patch), Commit, End(EndReason),
}
// AgentState mapping: Streaming / ToolRunning while a surface is open → SurfacePhase::Streaming;
// AwaitingApproval { request } whose gated object is a surface intent → SurfacePhase::AwaitingApproval;
// Stalled → skeleton stall behaviour; Cancelled → SurfacePhase::Cancelled; Error → Failed or PartialError.
```

## 10. Related components and references
Related: [genui-renderer.md](genui-renderer.md) · [genui-blocks.md](genui-blocks.md) ·
[genui-forms.md](genui-forms.md) · [genui-skeletons.md](genui-skeletons.md) · [skeletons.md](skeletons.md)
(engine, sweeps, swap) · [media-generation-placeholder.md](media-generation-placeholder.md) (generated
images inside surfaces) · [streaming-markdown.md](streaming-markdown.md) (link / image policy; its
`markdown/table-studio` "Chart this" hands off to `genui-chart`) · [approvals.md](approvals.md) and
[approval-card.md](approval-card.md) (intents with side effects) · [tool-call.md](tool-call.md) (a
surface produced by a tool renders under its tool row) · [container-loaders.md](container-loaders.md).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Google A2UI: declarative JSON UI, client-supplied catalogue, 18-component basic catalogue, functions (required, regex, email, formatCurrency / formatDate, pluralize, openUrl), icon enum | research note 05 §1 | catalogue-negotiation idea; coverage check for our 18; icon enum coverage | licence not recorded in [../licensing.md](../licensing.md): concepts only until checked |
| AG-UI: `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`, `STATE_SNAPSHOT` / `STATE_DELTA` as JSON Patch | research note 05 §1 | adapter target; JSON Patch for `data.patch` | not recorded: concepts only |
| Tambo: generative vs interactable components, props streamed, render with props undefined | research note 05 §1 | `Partial` node state; `genui-surface/pinned` | not recorded: concepts only |
| CopilotKit: Controlled → Declarative → Open-ended framing; HITL hooks | research note 05 §1 | positioning (§1) | not recorded: concepts only |
| Thesys C1 / Crayon (OpenUI): UI DSL with charts, forms, tables, slides, reports | research note 05 §1 | block coverage check | not recorded: concepts only |
| Vercel AI SDK UI / AI Elements: tool parts mapped to components | research note 05 §1 | "controlled" genUI as custom catalogue entries | not recorded: concepts only |
| MCP Apps / MCP-UI / OpenAI Apps SDK: sandboxed HTML, display modes inline / fullscreen / PiP | research note 05 §1 | container names (renderer) | not recorded: concepts only |
| pitus_ Generative UI series (Map, Merge conflicts, FAQ) | [R056], [R054], [R057] | block ideas | Study only |
| Composer chips opening gen-UI popovers that resolve into a result card | [R047] | intents resolving into result cards | Study only |
| Beautiful UI (21 sections incl. Fine-tune Card, Questionnaire-style approval) | [R106], [R133] | edit-in-place inspector idea | MIT: port with attribution |

**Gaps.** (1) A `Slides` block is in the taxonomy row but not specified (V2.2). (2) No research on how
each adapter advertises client capabilities to the agent; §2.7 assumes the app can put the catalogue in
the system prompt. (3) Open-ended HTML apps: GPUI has no web view; Dioxus iframe sandboxing not yet
designed. (4) A2UI's own wire format was not read in detail for this draft; the adapter mapping in §2.3
is by component name only.
