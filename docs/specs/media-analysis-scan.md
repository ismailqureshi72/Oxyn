**Status:** draft v0.1 (2026-11-11).

# Spec: media analysis scan (images, documents, video)

Part of family D ([container-loaders.md](container-loaders.md)); follows the 10-point standard in
[README.md](README.md) at the depth of [composer-send-control.md](composer-send-control.md). Target
**3 free + 14 Pro = 17 variants**, including **fill-over-media**. One analysis state machine drives a
surface layer over the media (scan line, fill, region boxes, OCR highlight) and a badge ("Reading page
3/12"); documents add a page strip, video adds a frame-index strip.

## 1. Purpose
Agents read material the user hands them: a screenshot of a bug, a photo of a receipt, a 40-page
contract, a scanned form, a 4-minute screen recording. While the agent reads, the user wants to know
**that it is reading, where it is in the material, and what it has found so far**, and afterwards
**what the agent looked at** so they can trust (or correct) the answer. A spinner over the image says
none of this. The scan layer puts the progress **on the media itself**: a scan line passes, regions the
agent detects get boxes, text it extracts lights up in reading order, pages tick off, frames get
indexed. Typical places: attachment tiles in the `Reading` state (composer-attachments.md uses a 1-line
scan bar), source cards in research agents, the live view of a computer-use agent, a video agent's
index ("the agent layer for video understanding", the scan highway in [R009]), and finance / support
agents reading receipts and screenshots. Loading UI ships an "Analyzing image" loader as a first-class
agent-verb loader ([R091], research note 16).

## 2. State machine

```
             analysis requested              bytes ready
   Hidden ───────────────────▶ Loading ───────────────────▶ Scanning (indeterminate)
                                  │                            │
                                  │ unreadable                 ├── doc with pages ──▶ Reading(page i/n) ─┐
                                  ▼                            ├── regions stream ─▶ Detecting(k)  ──────┤
                         Unreadable(reason)                    ├── text stream ────▶ Extracting(words) ──┤ (may run
                         (encrypted, corrupt, too small,       └── video ─────────▶ Indexing(frame k/n)──┤  together)
                          unsupported)                                                                   │
                                  ▲          needs password / permission                                 │
                                  └── fail ── AwaitingYou ◀──────────────────────────────────────────────┤
                                                    │ provided → resume where it stopped                  │
     Stalled (no event for stall threshold) ◀───────┼──────────────────────────────────────────────────── ┤
     Paused (offline) ◀─────────────────────────────┘                                                     │
                                                                                                          ▼
                                                     Summarizing (agent post-processing; layer calms) ──▶ Done(summary)
                                                                                                          │
   Partial(failed pages / frames) ◀── some units failed ──────────────────────────────────────────────────┤
   Failed(kind) ◀── error ─── any active state            Cancelled ◀── cancel ─── any active state       ▼
                                                                                         Reviewing (detections toggle)
```
Sub-states run **in parallel** when the source supports it: a page can be `Reading` while its words
are `Extracting` and its regions are `Detecting`. The badge shows the coarsest unit (pages > frames >
regions > words).

## 3. States table

| State | Surface over the media | Badge | Primary | Secondary | Keyboard (media focused) |
|---|---|---|---|---|---|
| `Loading` | media at 60 % opacity, edge beam ([border-beam.md](border-beam.md)) | "Opening…" | cancel | — | Esc |
| `Scanning` | 1 px accent scan line travels top → bottom with a 24 px soft trail; unread area slightly dimmed | "Analyzing image" | cancel | — | Esc |
| `Reading(i/n)` | page strip with page i enlarged; scan line runs over page i; pages < i get ✓ | "Reading page 3/12" | cancel | jump to page | ←/→ pages, Esc |
| `Detecting(k)` | corner-bracket boxes draw on detected regions with label chips; low-confidence boxes dashed | "Found 7 regions" | cancel | toggle boxes | Tab cycles boxes |
| `Extracting(words)` | highlighter band moves along text in reading order; extracted lines keep a faint underline | "Reading text · 340 words" | cancel | — | Esc |
| `Indexing(k/n)` (video) | keyframe strip; a light travels it and lights frames as they are indexed; main view shows the current frame | "Indexing 1:12 / 4:30" | cancel | scrub | ←/→ frame, Esc |
| Fill-over (any determinate unit) | unread part dimmed (60 %) and desaturated; read part restored; the boundary is a 1 px accent line | "41 %" or unit | cancel | — | Esc |
| `AwaitingYou` | motion stops; attention-tone outline; reason ("Password required", "Allow reading this file?") | "Needs you" | provide / allow | skip | Enter |
| `Stalled` | scan slows to 0.5×; badge tone ramps toward warning | "No progress for 45 s" | retry | cancel | Enter retry |
| `Paused` | frozen | "Paused · offline" | — | cancel | — |
| `Summarizing` | layer calms: scan stops, boxes stay, dim lifts | "Summarizing" | — | — | — |
| `Done(summary)` | dim lifts fully; boxes fade to 0 unless `keep_regions`; one completion edge pass | "12 regions · 340 words · 12 pages" (fades to a small ✓ chip after 3 s) | show detections | open extracted text | D toggles detections |
| `Partial` | as Done + failed pages / frames marked ⚠ in the strip | "11 of 12 pages read · 1 failed" | retry failed | open | Enter retry |
| `Unreadable(reason)` | media (or file icon) muted, struck icon | "Can't read: encrypted PDF" / "Image too small to read" | remove | details | Delete |
| `Failed(kind)` | scan frozen where it stopped, danger tint on the boundary | "Couldn't analyze · Retry" | retry | dismiss | Enter retry |
| `Cancelled` | layer fades | — | analyze again | — | — |
| `Reviewing` | detections toggled on after Done; hovering a box shows label + confidence | — | open citation | hide | Tab / Enter |

## 4. Micro-interactions
- **Scan line.** 1 px line with a 24 px trailing gradient travels top → bottom in 1.2 s (same value as
  the attachment tile scan in composer-attachments.md), loops with a 200 ms rest at the bottom; on
  documents it follows the current page; when the source reports the current text line, the scan line
  snaps to it (`spring.snap`) instead of looping (proposed).
- **Fill-over-media.** For any determinate unit (pages, frames, %), the unread part stays dimmed and
  desaturated, the read part is restored, and the boundary moves in reading direction (top → bottom for
  images and pages, left → right for frame strips; mirrored in RTL). It reads as "the agent's attention
  has passed here".
- **Region boxes.** Each detection draws its four corner brackets (160 ms), then a 6 % tint fills the
  box, then the label chip fades in (80 ms delay). Boxes arrive with 60 ms stagger, at most 12 per
  second; beyond 50 visible boxes new ones aggregate into a "+38 regions" chip. Confidence below the
  app's threshold uses a dashed outline. Hover: box lifts to 12 % tint and shows label + confidence.
- **OCR highlight.** A highlighter band (30 % accent) slides along extracted words in reading order;
  it trails by ≤ 600 ms and leaves a faint underline under extracted lines. In RTL scripts the band
  moves right → left; in vertical scripts top → bottom (proposed).
- **Page strip.** Thumbnails in a row (or column in narrow panels); the current page scales 1.08;
  finished pages get a ✓ that pops in (scale 0.6 → 1, 120 ms); failed pages a ⚠. Badge digits roll
  (family C odometer), badge width reserved for "12/12".
- **Video index.** A light runs along the keyframe strip and lights each frame as indexed (scan-car
  pattern [R009]); the main view shows the current frame with the scan line over it.
- **Done.** Dim lifts (240 ms), scan line fades, an edge completion pass runs once, the badge collapses
  to a ✓ chip after 3 s. Boxes fade unless `keep_regions` is on; "D" or the chip toggles them back.
- **Citation link.** When the agent's answer cites a region, hovering the citation highlights its box on
  the media (and scrolls the page strip) — shared with family E citations.
- **Failure.** The scan freezes where it stopped and the boundary tints danger, so the user sees how far
  it got; one 4 px shake on tiles only.

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Show delay / min display | — | 150 ms / 300 ms | — | container-loaders.md |
| Scan line pass (`loop.scan`) | 1.2 s + 200 ms rest | — | `ease.in-out` | 60 fps; 30 fps when > 12 hosts |
| Scan snap to reported line | 160 ms | — | `spring.snap` | |
| Fill-over boundary follow | continuous, ≤ 250 ms lag | — | `spring.snap` | |
| Region bracket draw / tint / label | 160 ms / 120 ms / 120 ms | label +80 ms; boxes 60 ms stagger, ≤ 12 / s | `ease.out` | |
| OCR band | trails ≤ 600 ms behind extraction | — | linear | |
| Page ✓ pop / current page scale | 120 ms / 160 ms | — | `spring.snap` | |
| Frame light (video) | 120 ms per frame | — | `ease.out` | |
| Done: dim lift / completion pass / badge collapse | 240 ms / 400 ms / 200 ms after 3 s | — | `ease.out` | |
| Boxes fade after Done | 320 ms | 600 ms | `ease.out` | unless `keep_regions` |
| Failure tint / shake | 160 ms / 300 ms (2 cycles, 4 px) | — | `ease.out` / linear | tiles only |
| AwaitingYou rest | 200 ms | — | `ease.firm` | |
| **Reduced motion** | no scan line: static 1 px line at the current position (or none if indeterminate) + badge text; fill-over updates with an 80 ms fade; boxes appear with an 80 ms fade, no draw; OCR shows static underlines per extracted line; no shake | — | — | state always in the badge |

## 6. Variants

### Free (3)
| Name | Idea |
|---|---|
| `analysis-scan/line` | Scan line with trail over any media; snaps to the current line when reported |
| `analysis-scan/fill-over` | Fill-over-media: unread part dimmed and desaturated, boundary moves with progress |
| `analysis-scan/pages` | Document page strip + "Reading page 3/12" badge + per-page ✓ / ⚠ |

### Pro (14)
| Name | Idea |
|---|---|
| `analysis-scan/grid-scan` | A grid of cells lights cell by cell across the image (tile-based analysis) |
| `analysis-scan/region-hunt` | Bracket boxes draw on detections with label chips and confidence |
| `analysis-scan/ocr-highlight` | Highlighter band follows extracted words in reading order |
| `analysis-scan/table-detect` | Row and column rules draw over detected tables, then cells tint as read |
| `analysis-scan/pixelate-resolve` | Media starts pixelated and resolves as the agent reads (input, not output) |
| `analysis-scan/halftone-reveal` | Halftone screen lifts as progress advances |
| `analysis-scan/radar` | Radial sweep from the centre with a fading wedge (UI screenshots, maps) |
| `analysis-scan/liquid-tide` | Fill-over boundary as a gentle liquid edge rising |
| `analysis-scan/clock-wipe` | Fill-over as a radial clock wipe (round thumbnails, avatars) |
| `analysis-scan/dither-fill` | Unread area ordered-dithered; dither lifts as it is read |
| `analysis-scan/frame-index` | Video keyframe strip lit by a travelling light ([R009]) with timecode badge |
| `analysis-scan/heatmap` | Attention heat accumulates where the agent looked; kept as a review layer |
| `analysis-scan/redact` | Sensitive regions (faces, card numbers, keys) blur as soon as detected, before display ([R049] idea) |
| `analysis-scan/terminal` | TUI: inverted row moves down an ASCII preview; "page 3/12" in the status cell |

## 7. Edge cases
- **Huge documents (500+ pages).** Page strip virtualises around the current page (± 6); badge shows
  "Reading page 214/530"; fill-over uses page fraction.
- **Non-linear reading.** Agents may jump pages (search, then read 47, then 3). The strip marks each
  page as read independently; the badge says "Read 5 of 530 pages" instead of "page i/n" when reading
  is not sequential (proposed).
- **Many regions.** Cap 50 visible, aggregate chip; dense small boxes (< 12 px) render as dots.
- **Tiny media (thumbnails < 64 px).** Only fill-over or edge beam; boxes and OCR deferred to the opened
  preview.
- **Unknown structure.** No regions, no text, no pages reported → stays `Scanning` (indeterminate) with
  the scan loop; never fakes boxes.
- **Password-protected / permission needed.** `AwaitingYou` with the reason; resume where it stopped.
- **Low resolution / blurry input.** `Unreadable("Image too small to read")` only when the agent reports
  it; otherwise analysis continues.
- **Media changes while scanning** (user replaces the file). Analysis cancels; new one starts; no
  stale boxes.
- **Video longer than the strip.** Strip buckets frames (≤ 48 visible buckets); timecode badge carries
  precision.
- **Privacy.** With `redact`, detected sensitive regions are blurred **before** the media is shown in
  the review layer and in exports.
- **Very fast analysis.** Under `show_delay`, nothing animates; detections (if kept) fade in once.
- **Very slow (long video).** Offscreen hosts stop ticking; badge keeps the elapsed / timecode.
- **RTL / vertical scripts.** Fill-over and OCR direction follow the content's script direction, not the
  UI direction, for documents (proposed); UI chrome (strip, badge) follows UI direction.
- **Touch.** Tap a box to show its label; long-press for "Open citation".
- **Dense IDE panels.** Line variant only, 1 px, no trail.

## 8. Accessibility
- **Roles.** The media keeps its own role and alt text; during analysis it gets `aria-busy="true"` and a
  progressbar labelled "Analysis of <file name>" with `aria-valuetext` ("Reading page 3 of 12",
  "Indexing 1 minute 12 of 4 minutes 30"); indeterminate while `Scanning`.
- **Labels per state:** Loading "Opening <file>"; Scanning "Analyzing image"; Reading "Reading page i
  of n"; Detecting "Found k regions"; Extracting "Reading text"; Indexing "Indexing video";
  AwaitingYou "<reason>"; Done "Analysis complete: 12 regions, 340 words"; Partial "11 of 12 pages read,
  1 failed"; Unreadable "Can't read <file>: <reason>"; Failed "Analysis failed. Retry available".
- **Detections are navigable.** In `Reviewing`, boxes are a list of focusable items (Tab / arrows),
  each labelled "<label>, <confidence> percent confidence"; Enter opens the citation. Hidden from the
  accessibility tree while boxes are hidden.
- **Extracted text** is available as real text (Open extracted text), not only as highlights.
- **Live region:** polite at start (user-started), page milestones at most every 10 s, completion;
  assertive for failure and `AwaitingYou` that blocks the run.
- **Flashing:** scan period ≥ 1.2 s; region boxes ≤ 12 / s with low tint (≤ 12 %); `grid-scan` cells
  change ≤ 0.3 luminance; never > 3 flashes per second.
- **Contrast:** box outlines and label chips meet 3:1 against the media via a 1 px contrast halo; badge
  text 4.5:1.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct AnalysisScan {
    pub agent: AgentState,               // ToolRunning { tool: Read | Vision | Ocr | VideoIndex, verb } -> badge verb
    pub job: JobSnapshot,                // container-loaders.md §8
    pub media: MediaKind,                // Image | Document { pages: Option<u32> } | Video { duration: Option<Duration> }
    pub phase: AnalysisPhase,            // Loading | Scanning | Active(Units) | AwaitingYou(reason) | Summarizing
                                         // | Done(Summary) | Partial(Summary, failed: Vec<UnitId>)
                                         // | Unreadable(reason) | Failed(kind) | Cancelled
    pub units: Units,                    // { page: Option<(u32, u32)>, frame: Option<(u32, u32)>, read_pages: BitSet }
    pub current_line: Option<Rect01>,    // normalised rect of the line being read (scan snaps to it)
    pub regions: Vec<Region>,            // { id, rect: Rect01, label, confidence: f32, kind: RegionKind }
    pub ocr: Option<OcrStream>,          // words with normalised boxes, in reading order
    pub keep_regions: bool,
    pub redact: RedactPolicy,            // Off | Kinds(Vec<RegionKind>)
    pub confidence_floor: f32,           // below = dashed (default 0.5, proposed)
    pub variant: ScanVariant,
    pub motion: MotionLevel,
    pub quality: RenderQuality,
}
```

## 10. Related components and references
Related: composer attachments (`Reading` tile state) · border beam (edge while `Loading`) · media
generation placeholder · source cards and citations (family E) · computer-use live view and cursor
trail (family F / M) · approval card for permission to read (family G).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| twelvelabs "agent layer for video understanding" scan highway | [R009] | `frame-index` | Study only |
| Loading UI "analyzing-image" loader; agent-verb loaders | [R091], research note 16 | "Analyzing image" state as a named loader | Open source (MIT per research note 16; confirm before porting) |
| Jakub Antalik skeleton-blueprint with scan-line reveal | [R005] | scan-line reveal | Study only |
| glimm "agent is writing here" sweep | [R034] | attention passing over a region | Study only |
| Hairline UI attachment chip ("Reading" under a 1.4 s light sweep); Eased UI blur → sharp thumbnail | research note 02 | reading sweep on chips | Licences not recorded: study only |
| Secret redaction card | [R049] | `redact` idea | Study only |
| Aceternity Image Generation Loader (pixel grids, progress overlays) | [R122] | `grid-scan` feel | Paid / unclear: study only |
| Paper Shaders (dither, halftone) | [R140] | `dither-fill`, `halftone-reveal` on the shader tier | Apache-2.0: may port with attribution |

Gaps: how vision / OCR tools report regions, lines and confidence varies by provider and is not
researched; the props accept normalised rects and treat every field as optional.
