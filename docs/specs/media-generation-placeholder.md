**Status:** draft v0.1 (2026-11-11).

# Spec: media generation placeholder (image and video)

Part of family D ([container-loaders.md](container-loaders.md)); follows the 10-point standard in
[README.md](README.md) at the depth of [composer-send-control.md](composer-send-control.md). Target
**3 free + 14 Pro = 17 variants**. One item state machine drives every candidate tile; a grid machine
coordinates N candidates; video adds a frame-progress track.

## 1. Purpose
Image and video agents (media studios, app builders making assets, CRM agents making social posts)
spend seconds to minutes generating. The user has just written a prompt and wants to know: **is it
queued or running, how far along, what shape will it be, can I stop it, and which of the candidates do
I want.** The placeholder holds the **exact final frame** (aspect ratio and size) from the first
frame so the canvas never jumps, shows honest progress, and turns into the result with a reveal that
feels like the image "developing" rather than popping in: noise → blur → sharpen (the progressive
de-noise reveal seen in seanaiux's studio [R013]; "a loader that becomes the image", Libraries.dev
[R100]; "dot noise wakes, then resolves into the image", transitions.dev [R132]; Aceternity's canvas
loader with pixel grids and progress overlays [R122]; research notes 15–17).

It also has to fail well: provider errors, timeouts, quota, and **content-policy blocks**, where the
placeholder must never reveal what was blocked.

## 2. State machine

**Candidate item** (one image or one video)
```
   Requested ──accepted──▶ Queued(pos?) ──slot──▶ Starting ──first progress──▶ Generating(progress)
       │                       │                     │                              │      │
       │ reject (params,       │ cancel              │ cancel                       │      │ provider sends a preview
       │ quota)                ▼                     ▼                              │      ▼
       ▼                   Cancelled ◀───────────────┴───────── cancel ─────────────┤  Previewing(img, progress)
   Failed(kind)                                                                     │      │
       ▲                                                                            ▼      ▼
       ├────────────── error / timeout ─────────────────────────────────────── Finalizing (upscale / encode)
       │                                                                                   │
   Blocked(policy) ◀── policy verdict (any time; replaces whatever is shown) ──────────────┤
                                                                                           ▼
                                                                   Revealing (600 ms) ──▶ Ready ──▶ Selected / Discarded
   Overlays: Stalled (no event for job_stall) · Paused(offline) · Offscreen (clock paused)
```

**Grid** (N candidates for one prompt)
```
  Allocating (N boxes at final ratio) ──▶ Running(k of N ready) ──▶ AllSettled ──▶ Choosing ──▶ Chosen(i)
         Partial: some Failed/Blocked, rest Ready (grid stays; failed tiles offer retry-one)
         CancelAll ──▶ Ready tiles stay, running tiles → Cancelled
```

## 3. States table

| State | Tile shows | Primary | Secondary | Keyboard (tile focused) |
|---|---|---|---|---|
| `Requested` | box at final ratio, flat surface tint, dim "1024 × 1024" size label (optional) | cancel | — | Esc cancel |
| `Queued(pos)` | static low-contrast noise, badge "Queued" or "Queued · 3 ahead" (only if the provider gives a position) | cancel | move up (if app supports) | Esc cancel |
| `Starting` | noise wakes (contrast 0.2 → 0.5 over 400 ms) | cancel | — | Esc |
| `Generating(Indeterminate)` | evolving noise field at 30 fps; slow light sweep across it every 2.4 s; badge "Generating" + elapsed (optional) | cancel | details | Esc |
| `Generating(Steps k/n)` / `(fraction p)` | noise grain shrinks and contrast drops as p rises (abstract, not the image); edge trace fill to p ([border-beam.md](border-beam.md)); badge "Step 12/30" or "41 %" | cancel | details | Esc |
| `Previewing(img, p)` | provider's intermediate image under thinning noise, blurred by lerp(24 px, 4 px, p) | cancel | — | Esc |
| `Finalizing` | noise gone; preview (or noise rest frame) at 4 px blur; badge "Finishing" / "Encoding" (video) | — (cancel if provider allows) | — | — |
| `Revealing` | final media sharpens 8 px → 0 while residual noise dissolves (600 ms) | — | — | — |
| `Ready` | final media; hover lifts tile; actions appear (use, variations, download, more) | select / open | variations, upscale, download, delete | Enter open · Space select · 1–9 pick in grid |
| `Selected` | accent ring 2 px; other tiles dim to 60 % (grid) | use | deselect | Space toggles |
| `Failed(kind)` | noise frozen and desaturated; ⟲ + "Couldn't generate" + kind ("Timed out", "Provider error", "Out of credits") | retry | edit prompt, dismiss | Enter retry · Delete dismiss |
| `Blocked(policy)` | **neutral** abstract surface (never derived from the content), shield icon, "Blocked by content policy" + provider reason if given | edit prompt | learn why, dismiss | Enter edit |
| `Cancelled` | noise fades to flat surface, "Cancelled" for 1.5 s then tile collapses (grid) or stays empty (single) | regenerate | dismiss | Enter regenerate |
| `Stalled` overlay | noise slows to 0.5×, badge tone ramps toward warning: "No progress for 45 s" | retry | cancel | Enter retry |
| `Paused(offline)` | noise frozen, "Paused · offline" | — | cancel | — |
| Video `Generating` | as image + **frame track** along the bottom: K buckets (K = min(frames, 24)) lighting as frames render; timecode "0:02 / 0:05" if frame count known; latest rendered frame shown if the provider streams frames | cancel | — | Esc |
| Video `Ready` | poster frame + play ▶ fading in; never autoplays with sound | play | download, extend, variations | Space play/pause |

## 4. Micro-interactions
- **Allocate.** Boxes appear at the final aspect ratio immediately on send (grow-in scale 0.96 → 1,
  fade, 40 ms stagger, cap 6); the canvas reflows once, at send, never at reveal.
- **Noise wake.** Noise starts nearly flat and gains contrast as the job starts, so "queued" (still) and
  "working" (alive) are distinguishable at a glance.
- **Progress as development.** With a fraction or steps, the abstract noise gets finer and calmer as p
  rises; the edge trace fills along the outline. Without one, the noise evolves at a constant rate and a
  soft sweep passes every 2.4 s. We never fake a percentage (container-loaders.md §4.2).
- **Previews.** If the provider streams intermediate images, each new preview cross-fades in (200 ms)
  under the noise layer; blur decreases with p, so the image visibly "comes into focus".
- **Reveal.** On the final asset: blur 8 px → 0 and noise opacity 1 → 0 over 600 ms (`dur.reveal`)
  with a 1.02 → 1 scale settle; the edge trace runs its completion pass at the same time.
- **Grid reveal order.** Tiles reveal in completion order; if several land within 300 ms they reveal in
  reading order with 80 ms stagger (a ripple, not a flash).
- **Choosing.** Hover lifts a Ready tile 2 px with a stronger shadow; select draws a 2 px ring and dims
  the others to 60 %; number keys 1–9 pick.
- **Cancel one / cancel all.** ✕ on a tile freezes its noise and fades it (160 ms); "Stop all" in the
  grid header cancels running tiles only, never removes Ready ones.
- **Failure.** Noise freezes and desaturates (200 ms), one 4 px shake (300 ms), ⟲ fades in.
- **Blocked.** Whatever was showing (noise or a preview) is replaced **instantly** (no fade through the
  content) by the neutral surface; then the shield and text fade in (160 ms). No shake: it is not the
  user's typing error.
- **Video frame track.** Buckets light left → right (RTL: right → left) as frames complete; the
  "scan-car" variant runs a light along the strip lighting each frame as it is indexed ([R009]).
- **Job pill (docked).** When the grid scrolls out of view, a pill docks at the bottom of the canvas,
  "Image to video · Rendering 4%" ([R013]); clicking it scrolls back.
- **Queue lanes.** Generations queued behind a running one show as the agent's lane in the queue stack
  ("Up next" badge and two lanes as in fal.ai's queue [R084]; see composer-queue-stack.md).

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Box allocate (scale 0.96 → 1, fade) | 180 ms | 40 ms stagger, cap 6 | `ease.out` | at send only |
| Noise wake (contrast 0.2 → 0.5) | 400 ms | — | `ease.out` | |
| Noise evolution (`loop.noise`) | continuous | — | — | 30 fps; 15 fps when > 12 tiles animate |
| Indeterminate sweep | 2.4 s period, 900 ms pass | — | `ease.in-out` | proposed |
| Progress grain / contrast follow | continuous, ≤ 250 ms lag | — | `spring.snap` | |
| Preview cross-fade | 200 ms | — | `ease.out` | |
| Reveal (blur 8 → 0, noise 1 → 0, scale 1.02 → 1) | 600 ms (`dur.reveal`) | grid: 80 ms stagger if within 300 ms | `ease.out` | proposed |
| Hover lift / select ring | 120 ms / 160 ms | — | `ease.out` / `spring.snap` | |
| Others dim on select | 160 ms | — | `ease.out` | |
| Failure desaturate + shake | 200 ms + 300 ms (2 cycles, 4 px) | — | `ease.out` / linear | |
| Blocked swap | 0 ms content swap; text 160 ms | — | — / `ease.out` | never fade through content |
| Cancel fade | 160 ms | — | `ease.in` | |
| Video bucket light | 120 ms | — | `ease.out` | |
| Job pill dock | 200 ms | — | `spring.snap` | |
| **Reduced motion** | noise is a static grain frame; progress = static edge fill + badge text; previews swap with 80 ms fade; reveal = 80 ms crossfade (no blur animation, no scale); no shake; video track buckets fill without animation | — | — | state always in badge text |

## 6. Variants

### Free (3)
| Name | Idea |
|---|---|
| `gen-placeholder/noise-resolve` | Abstract noise that calms with progress, then blur → sharpen reveal (the default) |
| `gen-placeholder/blur-in` | No noise: flat surface + sweep; previews and the final image focus in from blur (cheapest; CSS path) |
| `gen-placeholder/video-strip` | Video: noise main area + frame-bucket track + timecode, poster reveal |

### Pro (14)
| Name | Idea |
|---|---|
| `gen-placeholder/dot-noise` | A dot field wakes and its dots resolve into the image's pixels on reveal |
| `gen-placeholder/dither-resolve` | Ordered-dither surface whose levels increase with progress; reveal un-dithers |
| `gen-placeholder/halftone` | Halftone circles shrink into the image on reveal |
| `gen-placeholder/pixel-scan` | Pixel grid scans across the tile; cells resolve row by row on reveal |
| `gen-placeholder/blueprint` | Dotted wireframe with mono size label ("IMAGE 1024×1024") and corner handles, then a scan reveal ([R005]) |
| `gen-placeholder/latent-preview` | Built around provider previews: step counter, preview history scrubber |
| `gen-placeholder/countdown` | Big ETA countdown with reveal at zero; only when the provider supplies an ETA |
| `gen-placeholder/prompt-echo` | Prompt words drift into the noise and dissolve as generation starts |
| `gen-placeholder/candidate-race` | Grid where each tile's edge fills independently; first ready tile gets a soft spotlight |
| `gen-placeholder/scan-car` | Video: a light runs along the frame strip, lighting frames as they render ([R009]) |
| `gen-placeholder/timeline-render` | Video editor: rendered ranges fill on the clip in the timeline with the playhead preview |
| `gen-placeholder/job-pill` | Docked background-job pill with % that expands back into the grid ([R013]) |
| `gen-placeholder/aurora-noise` | Shader-tier coloured noise tinted by the prompt's style colours (if supplied) |
| `gen-placeholder/ascii-resolve` | Terminal / pixel look: char-density ramp that resolves into a low-res ASCII preview, then the image ([R011]) |

## 7. Edge cases
- **Unknown aspect ratio.** Reserve the requested ratio; if none was requested, 1:1. If the result's
  ratio differs, fit it inside the box (contain) over a blurred, darkened copy of itself; with
  `allow_resize`, animate the box height 220 ms after reveal (top edge fixed).
- **Extreme ratios** (1:4, 21:9). Box capped to the grid cell; badge shows the true size.
- **Large grids (16+).** Only 24 tiles animate (container-loaders.md §4.4); the rest show a static
  grain frame + their own badge.
- **Partial results.** Some tiles Ready, some Failed / Blocked: grid header reads "3 of 4 ready · 1
  failed · Retry"; retry-one regenerates in place.
- **Blocked after a preview was shown.** Replace immediately; previews of a blocked item are not kept
  in history or the preview scrubber.
- **Provider sends no progress at all.** Stays indeterminate with elapsed time; `Stalled` after the job
  stall threshold (120 s, from the status strip, proposed for media too).
- **Very fast** (< 150 ms, cached result). No placeholder animation; the box allocates and the image
  cross-fades in 160 ms.
- **Very slow** (video, minutes). Job pill docks when out of view; elapsed shown; offscreen tiles stop
  ticking.
- **Cancel race.** Cancel pressed as the result arrives: if the asset is already Revealing, keep it and
  show "Finished before cancel" (polite), never throw it away silently.
- **Result URL expired / asset fetch failed.** `Failed("Couldn't load result")` with retry-fetch, not
  regenerate.
- **Tiny tiles (thumbnails 48–64 px).** Noise replaced by a sweep; badge becomes an edge fill only.
- **RTL.** Video bucket fill and sweeps run right → left; grid reading order mirrors.
- **Touch.** Long-press for the actions menu; select by tap; ✕ always visible while running.
- **Offline.** Running remote jobs keep running server-side: tile shows "Paused · offline" locally and
  resyncs state on reconnect (proposed).
- **Dense IDE / chat inline.** Single tile max 320 px wide inline in a message; grid opens in a sheet.

## 8. Accessibility
- **Roles.** Each tile is a `figure`-like group labelled by the prompt summary ("Image 2 of 4: red
  fox in snow"); while busy it has `aria-busy="true"` and a progressbar (determinate with
  `aria-valuenow` only when real progress exists; `aria-valuetext` "Generating image 2 of 4, step 12 of
  30" / "Rendering video, 40 of 96 frames").
- **Labels per state:** Queued "Queued, 3 ahead"; Generating "Generating"; Finalizing "Finishing";
  Ready "Generated image 2 of 4" + alt text if the provider or agent supplies a caption, otherwise the
  prompt; Failed "Generation failed: timed out. Retry available"; Blocked "Blocked by content policy";
  Cancelled "Cancelled".
- **Live region:** polite on start (user-started only), "2 of 4 images ready" at most every 10 s,
  "All 4 images ready"; assertive for failure and block.
- **Keyboard:** grid is a roving-tabindex grid (arrows move, Home/End, 1–9 pick, Space select, Enter
  open, Esc cancel running tile, Shift+Esc cancel all).
- **Flashing:** noise contrast changes are gradual; sweeps ≥ 2.4 s period; reveal is a single blur
  change; no variant flashes > 3 times per second; `dot-noise` and `pixel-scan` keep per-cell luminance
  changes small (≤ 0.3, proposed) and low-frequency.
- **Contrast:** badges and the Blocked / Failed text meet 4.5:1; the shield icon meets 3:1.
- **Focus:** focus never jumps to a newly ready tile; it stays where the user is; the composer keeps
  focus after send.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct GenPlaceholder {
    pub agent: AgentState,               // ToolRunning { tool: ImageGen | VideoGen, .. } drives the grid header
    pub job: JobSnapshot,                // container-loaders.md §8; progress = Steps / Determinate / Indeterminate
    pub kind: MediaKind,                 // Image | Video { frames: Option<u32>, duration: Option<Duration> }
    pub phase: GenPhase,                 // Requested | Queued{position: Option<u32>} | Starting | Generating
                                         // | Previewing | Finalizing | Revealing | Ready | Failed(GenFailure)
                                         // | Blocked{ reason: Option<SharedString> } | Cancelled
    pub aspect: AspectRatio,             // reserved from the request; never re-measured until Ready
    pub size_label: Option<SharedString>,// "1024 × 1024"
    pub preview: Option<ImageSource>,    // provider intermediate (dropped when Blocked)
    pub result: Option<MediaSource>,
    pub allow_resize: bool,              // animate box to the result's ratio after reveal
    pub variant: GenVariant,
    pub labels: GenLabels,
    pub motion: MotionLevel,
    pub quality: RenderQuality,          // noise variants need Gpu/Shader or Canvas; Css falls back to blur-in
}

pub struct GenGrid {
    pub items: Vec<GenPlaceholder>,
    pub layout: GridLayout,              // Auto(N) -> 1, 1×2, 1×3, 2×2, 2×3, 3×3, 4×4
    pub selection: Selection,            // Single | Multi
    pub on_cancel_all: Option<Callback>,
    pub dock_pill_when_offscreen: bool,
}
```

## 10. Related components and references
Related: border beam (`trace-progress` edge) · media analysis scan · composer attachments (generated
media can become an attachment) · queue stack lanes · status strip job pill · family F background jobs
tray · family I generative UI image block · family G content-policy messaging.

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| seanaiux studio: progressive de-noise reveal; "Image to video · Rendering 4%" job pill | [R013] | reveal concept; docked pill | Study only |
| Libraries.dev `img-fx` ("a loader that becomes the image"; needs three) | [R100], research note 16 | loader-into-image idea | Package MIT: may port with attribution |
| transitions.dev image-gen placeholder (dot noise wakes, resolves) | [R132], research note 17 | `dot-noise` behaviour | Custom licence forbids competing kits: study only |
| Aceternity Image Generation Loader (canvas, pixel grids, text masks, progress overlays) | [R122], research note 17 | `pixel-scan` | Paid / unclear: study only |
| BoardUI image generation loader ("animated countdown and reveal") | [R095], research note 16 | `countdown` (ETA only) | Check tier: Pro items study only |
| Jakub Antalik skeleton-blueprint with "IMAGE 1024×1024" labels and scan reveal | [R005] | `blueprint` | Study only |
| twelvelabs scan highway | [R009] | `scan-car` | Study only |
| fal.ai queue: "Up next", two lanes | [R084] | queued generations lane | Check source; study only |
| ASCII 3D renderer | [R011] | `ascii-resolve` | Study only |
| Paper Shaders (noise, dither) | [R140] | shader-tier noise | Apache-2.0: may port with attribution |
| MetalForge "Noise" shader loader | research note 13 | quality bar | Paid: study only |

Gaps: how individual providers report progress (steps vs fraction vs previews vs nothing) is not
researched; the spec supports all four. Queue position availability per provider is unknown.
