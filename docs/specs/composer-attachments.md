**Status:** v1 (plan #18, 2026-11-12): 10 free variants in `oxyn-core::composer::attachments`. Dioxus `AttachmentRow` + `DropZone` (all 10); GPUI tiles with upload ring.

# Spec: composer attachments (clip, drop zone, chips and tiles)

Follows the 10-point standard in [README.md](README.md) at the depth of
[composer-send-control.md](composer-send-control.md). Target **10 free + 16 Pro variants**. One item
state machine drives every tile and chip; the drop zone and the attachment row have their own small
machines.

## 1. Purpose
Agents work on material: screenshots of a bug, a CSV to analyse, a PDF contract, a folder, a URL, the
repo itself, a 300-line log pasted from the terminal. The attachment row is where that material waits
before it is sent, and where the user learns whether the agent can actually use it (uploaded? read?
too big? wrong type?). Most kits show a filename and a spinner. We show every step from queued to ready,
make failure recoverable in place, and keep context chips (files, URLs, repo) visually distinct from
uploaded files so the user knows what will be *sent* versus *referenced*.

## 2. State machine

**Item** (file, image, pasted text, context reference)
```
              add (pick / drop / paste)
                  │
                  ▼          slot free            bytes 100 %            parsed
   Validating ─▶ Queued ───────────▶ Uploading(p%) ──────────▶ Reading ─────────▶ Ready ──send──▶ Sent (locked into the message)
       │            │                  │      │                   │                 │
       │            │ offline          │ net  │ cancel (×)        │ parse fail      │ × / Backspace
       │            ▼                  ▼ err  ▼                   ▼                 ▼
       │         Paused ◀── back online ─ Error(retry) ◀───────────┘             Removing ─▶ (gone, Undo 6 s)
       ├─▶ TooLarge(limit)      ← rejected before upload, tile stays with reason until dismissed
       ├─▶ Unsupported(type)
       └─▶ Duplicate            ← same hash already attached: existing tile pulses, new one is not added
  Context refs (file path, URL, repo, symbol, thread) skip Uploading: Validating ─▶ Resolving ─▶ Ready | Error
```

**Drop zone**
```
  Hidden ── drag enters window ──▶ Armed (full-window hint) ── over composer ──▶ Hot(Valid | Invalid)
     ▲                                  │                                         │ drop
     └──────── drag leaves / Esc ───────┴─────────────────────────────────────────┴──▶ Hidden (items → Validating)
```

**Row**: `Empty` → `Some(n ≤ visible)` → `Overflow(+k more)` ↔ `Expanded` · `Reordering` while a tile is lifted.

## 3. States table

| State | Tile / chip shows | Primary | Secondary | Keyboard (tile focused) |
|---|---|---|---|---|
| `Validating` | tile at 60 % opacity, no ring, ≤ 1 frame normally | — | — | — |
| `Queued` | type icon or thumbnail at 60 %, label "Waiting…", dotted ring track | cancel (×) | — | Delete/Backspace removes |
| `Uploading(p)` | accent ring draws clockwise around the tile; % counts up in a 9 px corner label (pattern from [R036]); indeterminate 90° arc if size unknown | cancel (×, on hover/focus) | — | Delete cancels |
| `Paused` | ring frozen at p %, label "Paused · offline" | — | remove | Delete removes |
| `Reading` | ring gone; shimmer label "Reading…" for docs, "Analyzing image" with a 1-line scan bar for images, "Indexing 214 files" for folders | — | remove | Delete removes |
| `Ready` | full-opacity thumbnail or icon + name + meta ("PDF · 12 pages", "CSV · 2.4 MB"); × appears on hover/focus in the % slot | open preview | remove, reorder | Enter preview · Delete remove · Alt+←/→ reorder |
| `Error(reason)` | danger ring segment + ⟲ icon; label "Upload failed" / "Couldn't read file" | retry | remove, details | Enter retry · Delete remove |
| `TooLarge(limit)` | danger tint, label "32 MB · max 20 MB" | remove | "Compress" / "Attach as link" when the app supports it | Delete |
| `Unsupported(type)` | muted tile with struck type icon, "Can't read .exe" | remove | "Attach as plain text" for text-like files | Delete |
| `Duplicate` | no new tile; existing tile pulses once; polite announcement | — | — | — |
| `Removing` | tile shrinks and fades; siblings close the gap; Undo bar "Removed report.pdf · Undo" | undo | — | ⌘/Ctrl+Z undo while bar shows |
| `Sent` | tile moves up with the message bubble and becomes read-only there | open | — | — |
| Pasted-text chip | "Pasted text · 142 lines" with first line preview on hover | expand into the draft | remove, open as file | Enter expand |
| Context chip (file / folder / URL / repo / symbol / thread / selection) | icon + short name + dim tail ("auth.rs · src/middleware", "github.com", "main · 3 changes", "lines 12–40") | open / preview | remove | Enter preview · Delete remove |
| URL chip `Resolving` | favicon placeholder + shimmer domain | — | remove | — |
| `Overflow(+k)` | last visible slot becomes "+3 more" chip | expand row | — | Enter expands; Esc collapses |
| Drop zone `Armed` | 1 px dashed outline on the composer + top-of-window hint "Drop files to attach" | — | — | Esc |
| Drop zone `Hot(Valid)` | composer outline dashed accent 1.5 px, 6 % tint, "Drop to attach · 3 files" | drop | — | Esc |
| Drop zone `Hot(Invalid)` | danger outline, "2 of 3 files can't be attached" | drop (valid ones only) | — | Esc |

**Send gating** (`send_while_pending`, shared with the send control): `Block` (default: send shows
"Waiting for 2 uploads", Enter queues the send until ready), `SendWithout` (pending items are dropped
with a confirmation line), `QueueUntilReady` (send is accepted; the message waits in the queue). Errors
and rejected tiles never block: they are excluded and named in the hint row ("1 file won't be sent").

**File-type icons** (16 px in chips, 24 px in tiles; our own set, outlined, 1.5 px stroke): image, PDF,
document, spreadsheet/CSV, slides, code (with language badge text, e.g. "RS", "TS"), archive, audio,
video, text/log, folder, repo, URL/favicon, symbol, thread, selection, unknown. Type is always also
written in the meta text, never icon-only.

## 4. Micro-interactions
- **Add.** Tiles grow in above the text from scale 0.85 to 1 with a fade (a thumbnail grow-in seen in
  Claude's GUI, reported at medium-low confidence in research notes 08); several files land one by one
  (40 ms stagger), "file tiles landing one by one above the prompt, each with its upload ring" [R095].
- **Upload ring.** The ring draws clockwise; the % label counts up with tabular digits. At 100 % the %
  blurs out and the × blurs in **at the same spot** (blur-cross-fade in the same slot [R036]). The ring
  then fades as the tile moves to `Reading`.
- **Reading.** A thin shimmer sweeps the label; images get a 1 px scan line travelling top to bottom. On
  ready, the thumbnail goes from 60 % to 100 % opacity and de-saturated to full colour.
- **Error.** The ring's unfinished arc turns danger colour where it stopped (so you can see how far it
  got), the tile does a 2-cycle 4 px shake, ⟲ fades in. Retry restarts the ring from the stopped point if
  the transport supports resume.
- **Too large / unsupported.** No ring ever draws. The tile lands, tints, and shows the reason; one shake.
- **Remove.** Tile shrinks to 0.9 and fades; neighbours glide into the gap on a spring. Undo re-opens the
  gap and the tile grows back in place.
- **Reorder.** Press-and-drag (or Alt+←/→) lifts the tile (scale 1.03, ≈3° tilt, shadow) and the gap
  opens where it will land, then it settles on a spring (lift pattern from [R040]).
- **Overflow.** Beyond `max_visible`, the last slot becomes "+3 more"; expanding staggers the hidden
  tiles in and morphs the row height (stagger capped at 300 ms total, the cap Arc's chip group uses,
  research notes 12). Selected or erroring items always stay visible.
- **Drop zone.** On window drag-enter, the composer outline appears; over the composer it heats up
  (tint + dashes march). Leaving the window clears it after a 50 ms delay (avoids flicker from child
  drag events).
- **Image preview.** Click opens a lightbox that grows from the tile's rect (shared-element), closes back
  into it.
- **Pasted-text chip.** Hover shows the first 3 lines in a popover after 300 ms; Expand unfolds the text
  back into the draft where the chip was.
- **Send.** Ready tiles travel up with the message bubble and dock in it, in the same order.

## 5. Timing (defaults)

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Tile grow-in (scale 0.85→1, fade) | 180 ms | 40 ms stagger, cap 6 | spring (stiffness 320, damping 28, proposed) | |
| Upload ring progress | follows real progress, smoothed | updates ≤ 10 Hz | linear interpolation between updates | never goes backward |
| Indeterminate arc | 1,000 ms per turn | — | linear | 90° arc |
| % → × blur swap | 160 ms | at 100 % | ease-out, blur 4 px → 0 | same slot [R036] |
| Ring fade after 100 % | 160 ms | 120 ms | ease-out | |
| Reading shimmer | 1,800 ms per sweep | — | linear | loop until ready |
| Image scan line | 1,200 ms top→bottom | — | ease-in-out | loop |
| Ready (opacity + saturation) | 200 ms | 0 | ease-out | |
| Error shake | 300 ms, 2 cycles, 4 px | 0 | linear | |
| Remove (scale 0.9, fade) | 140 ms | 0 | ease-in | gap closes 180 ms spring |
| Undo window | 6 s | pauses on hover/focus | — | Arc MCP uses about 6 s, research notes 12 |
| Reorder lift / settle | 120 ms / 220 ms | 0 | ease-out / spring (stiffness 380, damping 30) | 1.03 scale, ≈3° tilt |
| Overflow expand | 160 ms row height | 30 ms stagger, cap 300 ms total | spring | |
| Drop zone armed / hot | 120 ms / 100 ms | leave delay 50 ms | ease-out | dashes 60 ms per px, ≤ 30 fps |
| Hover preview popover | 140 ms | 300 ms | ease-out | |
| Lightbox open / close | 240 ms / 200 ms | 0 | spring / ease-in | shared element |
| Minimum `Uploading` display | 300 ms | — | — | if the upload finishes faster, skip the ring entirely and go straight to `Ready` (no flash) |
| **Reduced motion** | 0–80 ms opacity cross-fades | no stagger | linear | ring is replaced by a static % text label (still updates); no shimmer, scan, shake, lift, tilt or marching dashes; every state label and announcement kept |

Sizes (proposed): image/file tile 56 × 56 px (the BoardUI strip uses 56 px tiles [R036]), radius 10 px,
gap 8 px; ring 2 px stroke inset 1 px; % label 9 px mono [R036]; chip height 28 px (24 px dense,
36 px touch), max chip width 220 px; × hit area 24 px (44 px on touch). Default `max_visible` = 6 tiles
or 1 row of chips.

## 6. Variants

### Free (10)
| Name | Idea |
|---|---|
| `attachment/tile-ring` | 56 px tiles above the text with clockwise upload ring and % → × swap |
| `attachment/chip` | Compact one-line chips (icon · name · meta) for text-first composers |
| `attachment/chip-bar` | Linear progress bar along the chip's bottom edge instead of a ring |
| `attachment/thumb-strip` | Image-first: larger 72 px thumbnails in a horizontally scrolling strip with edge fades |
| `attachment/context-chip` | References (file, folder, URL, repo, symbol, thread, selection) with path tail; no upload states |
| `attachment/paste-chip` | Pasted-text chip with hover preview and Expand |
| `attachment/list` | Vertical list rows with filename, size and status text for dense IDE panels |
| `attachment/drop-overlay` | Full-window drop overlay with a large target and per-file validity |
| `attachment/drop-inline` | Drop target confined to the composer outline (default) |
| `attachment/terminal` | `[file] report.pdf 64 %` text rows with a block progress bar `▓▓▓░░` |

### Pro (16)
| Name | Idea |
|---|---|
| `attachment/pile` | Attached items stack into an animated pile; hover fans them out, click expands (pattern seen in Dia, research notes 04) |
| `attachment/shape-trace` | The progress line traces the tile's actual outline (rounded rect, image silhouette) |
| `attachment/fill-over` | Upload fills the thumbnail from bottom to top like liquid, then clears |
| `attachment/scan-highlight` | While reading, detected regions (text blocks, faces, tables) flash briefly on the image |
| `attachment/denoise-reveal` | Thumbnail goes from noise to sharp as the upload progresses (progressive de-noise idea, [R013]) |
| `attachment/page-flip` | PDFs show a mini page count that flips through the first pages while being read |
| `attachment/token-meter` | Each tile shows its context cost; the row totals it against the model's window |
| `attachment/magnetic-drop` | Tiles are pulled toward the drop point and snap into order |
| `attachment/orbit-upload` | A comet orbits the tile during upload, leaves a ring on completion |
| `attachment/pixel-assemble` | Thumbnail assembles from a pixel grid as bytes arrive |
| `attachment/split-confirm` | On ready, a ✓ bubble pinches off the tile like a droplet (split-off confirmation, [R006]) |
| `attachment/folder-tree` | Folder attachments grow a mini file tree as indexing proceeds |
| `attachment/url-card` | URL chips unfold into a preview card (title, favicon, image) once resolved |
| `attachment/cover-flow` | Many images browse as a cover-flow carousel |
| `attachment/ascii` | ASCII thumbnails of images in terminal surfaces |
| `attachment/blueprint` | Dotted wireframe with mono size labels ("IMAGE 1024×1024") draws in before the real thumbnail (skeleton-blueprint idea, [R005]) |

## 7. Edge cases
- **Long filenames** truncate in the middle so the extension stays visible ("quarterly-rep…final.pdf").
  Full name in the accessible label and tooltip.
- **Many files.** 50 files dropped at once: validate all, upload with a concurrency of 3 (proposed),
  `Queued` for the rest; the row shows 6 + "+44 more". Above `max_files` (default 10, proposed) the extra
  files are rejected with one summary line, not 40 error tiles.
- **Folders.** Dropping a folder becomes one folder chip with a count; never 300 tiles.
- **Duplicates** (same content hash): no second tile; the existing one pulses.
- **Huge images** are downscaled client-side for the thumbnail only; the original is uploaded unless the
  app sets a resize policy.
- **HEIC / RAW / SVG.** Thumbnails fall back to the type icon when the platform cannot decode; SVG
  thumbnails render sanitised (no scripts).
- **RTL.** Row order follows reading direction (first attachment on the right); the ring still draws
  clockwise (a clock is not mirrored); Alt+←/→ reorder follows visual direction.
- **Tiny sizes / dense IDE.** Below 40 px the ring becomes a 2 px bottom bar and % hides (shown in the
  tooltip); `list` is the default in IDE surfaces.
- **Touch.** No hover: × is always visible on touch; long-press starts reorder; drag-from-other-apps is
  supported where the platform allows, otherwise the "+" picker is the path.
- **Slow networks.** Ring smoothing hides jitter; if no progress event arrives for 10 s the label adds
  "Slow connection…" (proposed). Offline pauses uploads (`Paused`) and resumes automatically.
- **Very fast uploads** (< 300 ms) skip the ring entirely. **Very slow reading** (> 20 s) shows elapsed
  time "Reading · 24 s".
- **Agent busy.** Attachments added while busy belong to the draft (and therefore to the queued or steer
  message); they never change the running turn silently.
- **Model cannot read a type** (e.g. images on a text-only model): tile lands as `Unsupported("This model
  can't read images")` with an action to switch model if the app provides one.
- **Privacy.** Context chips that point to secrets-like files (`.env`, keys) show a warning line before send.

## 8. Accessibility
- The row is a list labelled "Attachments, 3 items"; each tile is a list item containing a button
  (preview) and a remove button.
- Accessible names per state: "report.pdf, PDF, 2.4 megabytes, uploading 64 percent"; "…, reading";
  "…, ready"; "…, upload failed, press Enter to retry"; "…, too large, maximum 20 megabytes".
- Progress is announced politely at 25 % steps at most (not every tick) and on completion; errors are
  announced assertively once.
- Remove: "Removed report.pdf. Press Control Z to undo." Undo: "Restored report.pdf."
- Drop zone: drag-and-drop always has an alternative (+ → Upload file, paste). Drop results are announced
  ("2 files attached, 1 file can't be attached: setup.exe").
- Reorder by keyboard: Alt+←/→ with announcement "Moved to position 2 of 4".
- Focus order: tiles in visual order, then the text area. Removing a focused tile moves focus to the next
  tile, or to the text area if none remain.
- Contrast: meta text 4.5:1; ring ≥ 3:1 against tile and background; error state uses icon + text.
- Image thumbnails take alt text from the filename unless the app provides a description.

## 9. Props sketch

```rust
pub struct Attachments {
    pub agent: AgentState,                    // busy → items go to the draft, model limits apply
    pub items: Vec<AttachmentItem>,
    pub variant: AttachmentVariant,           // TileRing | Chip | ChipBar | ThumbStrip | List | Terminal | ...
    pub drop_zone: DropZoneMode,              // Inline | Window | None
    pub accept: AcceptRule,                   // mime types, extensions, per-type max size
    pub max_files: u16,                       // 10
    pub max_visible: u16,                     // 6, rest behind "+k more"
    pub concurrency: u8,                      // 3
    pub send_while_pending: PendingPolicy,    // Block | SendWithout | QueueUntilReady
    pub undo_ms: u32,                         // 6000
    pub surface: Surface,
    pub labels: AttachmentLabels,
    pub motion: MotionLevel,
    pub on_add: Callback<Vec<NewAttachment>>,
    pub on_remove: Callback<AttachmentId>,    // fires after the undo window closes
    pub on_retry: Callback<AttachmentId>,
    pub on_reorder: Callback<(AttachmentId, usize)>,
}

pub struct AttachmentItem {
    pub id: AttachmentId,
    pub kind: AttachmentKind,   // File { mime, bytes } | Image { thumb } | PastedText { lines } | Context(ContextRef)
    pub name: String,
    pub state: AttachmentState,
}

pub enum AttachmentState {
    Validating, Queued, Uploading { progress: Option<f32> }, Paused { progress: f32 },
    Reading { label: ReadLabel, since: Instant }, Ready { meta: String },
    Error { reason: String, retryable: bool }, TooLarge { limit: u64 }, Unsupported { reason: String },
    Removing, Sent,
}

pub enum ContextRef { File(PathBuf), Folder(PathBuf), Url(String), Repo { branch: String }, Symbol(String), Thread(ThreadId), Selection { path: PathBuf, lines: Range<u32> } }
```

## 10. Related components and references
**Related:** composer input shell (drop and paste entry) · tools menu ("Upload file", drop on "+") ·
mention tokens (an `@file` can become a context chip) · send control (send gating) · context meter ·
message bubble attachments · lightbox / media viewer · image-generation placeholder · undo bar / toast.

**References** (study only unless marked MIT/Apache; see [../licensing.md](../licensing.md)):
- [R036] BoardUI "Composer Attachments": 56 px tiles, clockwise accent ring, 9 px % that blurs into × at 100 %. Study only unless in the MIT tier.
- [R095] BoardUI attachment tray "file tiles landing one by one above the prompt, each with its upload ring". Free tier MIT, Pro paid.
- [R021] Phil Pitus file chips with "+3 more". Study only.
- [R040] Kanban lift (scale 1.03, ≈3° tilt, shadow, spring settle). Study only.
- [R005] skeleton-blueprint reveal; [R006] split-off ✓ confirmation; [R013] progressive de-noise reveal. Study only.
- [R126] coss ui composer (`allowEmpty`: an attachment can be sent alone; structured parts). **MIT**: API shape portable with attribution.
- Arc UI chat (attachment preview, tray springs open), chip group "+N more" (stagger cap 0.3 s), MCP undo window (≈6 s): paid / unclear, study only (research notes 12).
- Claude GUI `file-thumbnail-grow-in` scale .85→1 (reverse-engineered, medium-low confidence; proprietary, reference only); Dia attachment pile and per-tab failure reasons (proprietary): research notes 04, 08.
