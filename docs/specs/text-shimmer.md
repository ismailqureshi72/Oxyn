**Status:** v1 (plan #17, 2026-11-12): 4 free variants: `oxyn-core::text::Shimmer`, Dioxus `TextShimmer`, GPUI `ShimmerText`.

# Spec: text shimmer (`text-shimmer`)

Part of family C ([status-text.md](status-text.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **4 free + 18 Pro = 22 variants**.
Runs on the family-C engine (status-text.md §2): per-glyph sampling, no advance-width changes, shared
clock.

## 1. Purpose
A band of light crossing a status word ("Thinking", "Reading auth.rs") is the most common way agent
apps say *the model is alive and this text is live, not a finished label*. Every app we studied ships
one, each with different maths (note 08 §3): Codex's cosine band (1 s sweep every 4 s after 600 ms,
brightness only), Warp's 3 s cosine with radius 6 glyphs, OpenCode's asymmetric comet (head 4, tail
18, 1.2 s, coast easing), Claude Code's hard ±1-column glimmer at 50 or 200 ms/column, Cline's 5 s
background-position gradient, MonoCode / AI Elements' 2 s gradient; Arc's 1.8 s bell band [note 12],
affectUI's 2 s pass [note 11]. Kits ship one shimmer each (note 02). We ship **one primitive with
named knobs** and make each look a preset — and we add what none of them document: a **contrast
floor**, a **graceful stop**, and per-theme colour rules.

The user is not trying to *do* anything with a shimmer; they glance at it. So it must be calm enough
to sit in peripheral vision for minutes, and it must stop the instant it would lie (stalled, waiting
on the user, done).

## 2. State machine
```
               text set + active            dur.shimmer.delay (600 ms)
  Inactive ─────────────────────▶ Delay ───────────────────────────▶ Sweeping ──band exits──▶ Resting
     ▲  ▲                            ▲                                  ▲  │                     │
     │  │                            │ text changes (any state):        │  │ rest elapsed        │
     │  │                            └── band fades 100 ms, restart ────┼──┼─────────────────────┘
     │  │                                                               │  │
     │  │       active=false / Done                                     │  ▼ off-screen, unfocused,
     │  └───── Fading (finish if ≥70 % across, else amplitude→0 200 ms)  │  Paused (keeps phase) ──▶ resumes
     │                                                                  │
     │       AwaitingApproval / Error / Cancelled: Fading (same rule)   │
     │                                                                  │
     │       Stalled(soft): speed ×1 → ×0.5 over 2 s ── Stalled(hard) ─▶ Frozen (amplitude 0, base→warning tone)
     │                                                                  │ event arrives: 200 ms back to Sweeping
     └──── reduced motion at any time: Static (full fg, no band) ───────┘
```

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Inactive` | text at full `text.status.fg` | host's | — | host's |
| `Delay` | text at `text.status.base` (dimmed per contrast floor), no band | host's | — | — |
| `Sweeping` | band crossing in reading direction | host's (click → live step) | hover tooltip | — |
| `Resting` | text at base, no band | host's | — | — |
| `Paused` | frozen frame, band where it stopped | — | — | — |
| `Fading` | band finishing or amplitude dropping; base → fg | — | — | — |
| `Frozen` (stalled) | no band; base ramps toward `tone.warning` | host's (Retry lives in status strip) | — | — |
| `Static` (reduced motion / high-contrast) | full fg, no band, no dimming | host's | — | — |
| `Disabled` (no colour terminal) | plain text | — | — | — |

The shimmer owns no actions; the host (status line, tab, thought header) owns click and keys. It does
own the `aria-busy` flag on its host while not `Inactive`/`Static`.

## 3. Parameters (the primitive)
`shimmer(t, i, n, cfg) → mix` — one function, every variant is a preset of these knobs (the knob list
follows note 08's recommendation, our own values):

| Knob | Default (proposed) | Range | Notes |
|---|---|---|---|
| `profile` | `Cosine` | Cosine · Comet{head, tail} · Hard · Gaussian · Plateau | cosine `0.5·(1+cos πd)`, d = distance / half-width, clamped |
| `half_width` | `max(0.12 × line_w, 1.2 em)`, cap `0.5 × line_w` | px or % | the minimum keeps 3–5 letter words from flashing letter by letter (Codex uses ≥ 3 columns for the same reason) |
| `padding` | 0.5 em each side | — | band starts and ends fully off-text |
| `sweep` | 1400 ms for lines ≤ 200 px; above that constant **140 px/s** | 600–4000 ms | long headers don't streak |
| `rest` | 800 ms | 0–4000 ms | gap between sweeps |
| `delay` | 600 ms | 0–2000 ms | after text set/change |
| `direction` | reading direction | Forward · Reverse · PingPong | RTL → right-to-left automatically |
| `easing` | `ease.coast` (OpenCode, MIT) | linear · coast · `ease.in-out` | coast = quadratic ramps on first/last 20 % |
| `unit` | Grapheme | Grapheme · Word · Group | Group keeps "…" and "12s" as one unit |
| `colour_space` | BrightnessOnly (OKLab L lerp) | BrightnessOnly · HueLerp · Spectral | hue variants are Pro |
| `amplitude` | 1.0 | 0–1 | scaled down by the contrast floor if needed |

Per-glyph colour: `c_i = oklab_mix(base, crest, amplitude × intensity(d_i))`, sampled at the glyph's
**horizontal centre**, so a wide CJK glyph gets one colour, never a split.

### Per-theme colours
| Theme | `text.status.crest` | `text.status.base` (before floor) | Notes |
|---|---|---|---|
| Dark | `text.fg` | mix(fg → bg, **40 %**) | Codex rests at 50 % — we stay brighter so the floor holds on more themes |
| Light | `text.fg` | mix(fg → bg, **35 %**) | light backgrounds lose contrast faster |
| High-contrast (light/dark) | — | — | shimmer off: amplitude 0, text = fg; motion is carried by the loader |
| Colour-blind palettes | same as base theme | same | brightness-only, so no hue cue is lost |
| Accent-tinted (Pro `aurora`, `spectral`, `rainbow`) | mix(fg, accent, 50 %) or hue cycle | as above | each hue sample must also pass the floor |
| Terminal truecolor | fg | mix 40 % | per-cell colour |
| Terminal 256 / 16 colour | — | — | tiers: DIM < 0.2, normal < 0.6, BOLD ≥ 0.6 (Codex legacy fallback, note 08) |
| No colour / `NO_COLOR` | — | — | static text |

### Contrast floor (required)
The dimmest frame of any shimmer must still be readable:
1. At theme load, for each `(fg, bg)` pair, find the largest dim factor `k ≤ k_default` such that
   `contrast(mix(fg, bg, k), bg) ≥ 4.5:1` (3:1 when the status text is ≥ 18.66 px bold or ≥ 24 px —
   WCAG "large text"). Binary search, 8 iterations, cached.
2. If even `k = 0` fails (the theme's fg is below 4.5:1), set `amplitude = 0` and log a dev-build
   warning naming the token pair.
3. Hue variants: every crest sample is checked the same way; failing hues are pulled toward fg in
   OKLab L until they pass.
4. The **crest is never dimmer than base** (no "dark band" on light themes unless the variant is
   explicitly `ink-bleed`, which then uses the same floor).
5. Flash safety: luminance swings repeat at most once per 1.0 s by schema (min `sweep + rest` = 1 s),
   far below the WCAG 2.3.1 three-flashes-per-second limit.

## 4. Micro-interactions
- **Start.** New text arrives at base colour; nothing moves for 600 ms; the first sweep eases in with
  `ease.coast`. A turn that ends inside 600 ms never shimmers.
- **Text change mid-sweep.** The band's amplitude drops to 0 in 100 ms while the verb cycler swaps the
  text; the clock restarts at `Delay` (Codex resets the shimmer clock per header, note 08). Band and swap
  never overlap.
- **Graceful stop (done, waiting, error).** If the band centre is ≥ 70 % across, it finishes the sweep
  (≤ 400 ms worst case); otherwise its amplitude fades to 0 over 200 ms while it keeps moving. Base then
  brightens to full fg over 200 ms. It never snaps off (OpenCode's inactive/active/fading machine, MIT,
  solves the same problem).
- **Stall.** At `stall_soft` the sweep slows from 1× to 0.5× over 2 s; at `stall_hard` it freezes and
  base ramps toward `tone.warning` (tone ramp owned by status-text). First new event: 200 ms back to
  normal speed.
- **Hover.** No change by default (status text is peripheral); `pointer-follow` (Pro) moves the band to
  the pointer and returns to auto-sweep 400 ms after the pointer leaves.
- **Selection.** When the user selects the text, the shimmer pauses and selection colours win.
- **Off-screen / unfocused.** Paused at the current phase; resumes there (Arc does the same, note 12).

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| First sweep after text set | 1400 ms (≤ 200 px) / 140 px/s | 600 ms | `ease.coast` | — | 30 Hz |
| Rest between sweeps | 800 ms | — | — | — | 1 Hz (idle) |
| `calm` cadence | 1000 ms sweep, 3000 ms rest | 600 ms | cosine band, linear travel | — | 30 Hz |
| Band drop on text change | 100 ms | 0 | `ease.in` | — | 30 Hz |
| Graceful stop (amplitude → 0) | 200 ms | 0 | `ease.out` | — | 30 Hz |
| Base → fg after stop | 200 ms | after band gone | `ease.out` | — | 30 Hz |
| Stall slow-down | 2000 ms to 0.5× | stall_soft | linear | — | 30 Hz |
| Recover from stall | 200 ms | 0 | `ease.out` | — | 30 Hz |
| `pulse` breath | 2000 ms loop, amplitude 0.6 | 600 ms | sine | — | 30 Hz |
| `wave` (Pro) per-glyph y bob | 1 px, 1200 ms | — | sine | 45 ms/glyph | 60 Hz |
| Terminal tiers | sweep 2000 ms | 600 ms | linear | — | 30 Hz (frame-quantised) |
| **Reduced motion** | no band, no pulse: text static at full fg (not base — dimmed static text looks disabled); stall shown as instant tone change + text | — | — | — | 0 Hz |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `text-shimmer/sweep` | Default: cosine band, 1.4 s sweep + 0.8 s rest, brightness only | all except terminal |
| `text-shimmer/calm` | One 1 s sweep every 4 s after 600 ms — the quietest motion for long runs and dense IDEs (Codex cadence, portable) | ide, desktop, terminal |
| `text-shimmer/pulse` | Whole text breathes base ↔ fg on a 2 s sine; for very short labels and screens where a moving band is too busy | mobile, voice, 16–20 px rows |
| `text-shimmer/ascii` | Terminal tiers (DIM / normal / BOLD) driven by the same band; static under no-colour | terminal |

### Pro (18)
| Name | One-line idea |
|---|---|
| `text-shimmer/comet` | Asymmetric band: short bright head, long fading tail (head 0.3 em, tail 1.4 em), coast easing |
| `text-shimmer/dual-band` | Two narrow bands 40 % apart — reads "busy" without being faster |
| `text-shimmer/spectral` | Crest hue rotates through the accent ramp as it travels; each hue checked against the floor |
| `text-shimmer/chrome` | Metallic: three-stop crest (bright core, dim shoulders) for the `oxyn` house look |
| `text-shimmer/aurora` | Crest tinted 50 % toward accent, slow 2.4 s sweep |
| `text-shimmer/glow-halo` | Crest adds a 4 px blurred glow behind glyphs (desktop/web only; terminal falls back to `sweep`) |
| `text-shimmer/yoyo` | Band ping-pongs instead of wrapping; rest only at the far end |
| `text-shimmer/wave` | Brightness plus a 1 px per-glyph vertical bob in a travelling wave |
| `text-shimmer/hairline` | Text stays solid; a 1 px underline carries the band — for users who find text shimmer distracting |
| `text-shimmer/mask-wipe` | One-shot: new text is revealed by the band (left of band visible, right hidden), then rests solid |
| `text-shimmer/rainbow` | Full hue sweep for brand moments and wordmarks [R029]; never on body status text by default |
| `text-shimmer/ink-bleed` | Inverse: a darker band on light themes, like ink soaking through; floor still applies |
| `text-shimmer/stall-ember` | Normal sweep that cools to embers (warm, slow, then frozen) as the stall ramp proceeds |
| `text-shimmer/pointer-follow` | Band follows the pointer on hover; auto-sweep resumes 400 ms after leave |
| `text-shimmer/word-pop` | Unit = word: each word brightens in turn (good for long model-written headers) |
| `text-shimmer/dither` | Band rendered as an ordered-dither pattern of brightness on glyph pixels (pixel/terminal aesthetic) |
| `text-shimmer/weight-wave` | Variable-font weight +100 inside the band instead of colour; **only with fonts that keep advance widths constant across weights**, else falls back to `sweep` |
| `text-shimmer/outline` | Glyph outlines light up inside the band (stroke only), fill stays base |

Every variant documents states, timing, reduced-motion fallback (always `Static`), surfaces, tokens
read and credits per [../variants.md](../variants.md).

## 7. Edge cases
- **Very short text (1–3 graphemes, e.g. "…", "Go").** `half_width` minimum makes the whole word glow
  together; below 3 graphemes `sweep` auto-switches to `pulse` (proposed).
- **Very long text (> 400 px).** Speed cap 140 px/s; the band stays a readable width; a model-written
  header longer than 60 chars is middle-truncated by the host before shimmering.
- **Wrapped / multi-line text.** Status hosts are single-line; if a host wraps (narrow mobile), each
  line sweeps in order with a 120 ms offset, never simultaneously.
- **RTL.** Direction = reading direction of the paragraph; mixed-direction runs are sampled by visual x,
  so the band moves smoothly across the screen even where the logical order jumps.
- **CJK / complex scripts.** Sampling at each grapheme's centre; full-width glyphs take one colour;
  conjuncts never split.
- **Emoji.** Colour emoji get opacity modulation only (0.85 ↔ 1), never a colour mix.
- **Tiny sizes (16–20 px rows, 11–12 px type).** `calm` cadence and 1.0 em minimum half-width; on a
  20 px row the shimmer is the only motion (loader becomes static).
- **Screen width / zoom.** Half-width and speed are in em/px of the text, re-measured on zoom; no
  restart of the clock on re-measure.
- **0 s / very fast turns.** Never visible (600 ms delay). **Very slow turns.** After 5 min, rest grows
  to 2.4 s (proposed) so the line calms down.
- **Many shimmers on screen.** All phase-locked to the shared clock; capped at 8 animated (status-text
  §10); extra ones render `Static`.
- **Selection, copy.** Copy gives plain text; shimmer never inserts characters.
- **Transparent / image backgrounds.** Contrast floor uses the host's declared `bg` token; on unknown
  backgrounds (glass), `amplitude` is capped at 0.5 and base dim at 20 % (proposed).

## 8. Accessibility
- The shimmer is purely visual; the text node is real text and the accessible name is the text.
- Host gets `aria-busy="true"` while the shimmer is in any active state; cleared on `Fading` end.
- Nothing is announced by the shimmer; announcements belong to the host's live region with the
  family throttling (status-text.md §11).
- **Contrast floor** (§3) guarantees ≥ 4.5:1 (3:1 for large text) on every frame; high-contrast themes
  disable the effect.
- **Reduced motion:** `Static` at full fg. **Flashing:** ≤ 1 luminance cycle per second by schema.
- Focus ring and selection colours are never altered by the shimmer.

## 9. Props sketch
```rust
pub struct TextShimmer {
    pub text: SharedString,
    pub active: ShimmerDrive,          // FromState(&AgentState) | Manual(bool)
    pub variant: ShimmerVariant,       // Sweep | Calm | Pulse | Ascii | Pro(ProShimmer)
    pub cfg: Option<ShimmerCfg>,       // overrides of the preset (Studio exposes these)
    pub tone: ToneToken,               // follows status tone (stall → warning)
    pub motion: MotionLevel,
}

pub struct ShimmerCfg {
    pub profile: BandProfile,          // Cosine | Comet { head: Em, tail: Em } | Hard | Gaussian | Plateau
    pub half_width: Length,            // Max(Percent(0.12), Em(1.2))
    pub padding: Em,                   // 0.5
    pub sweep: SweepSpeed,             // Duration(1400 ms) | PxPerSec(140) | Auto
    pub rest: Duration,                // 800 ms
    pub delay: Duration,               // 600 ms
    pub direction: Direction,          // ReadingOrder | Forward | Reverse | PingPong
    pub easing: Easing,                // Coast
    pub unit: Unit,                    // Grapheme | Word | Group
    pub colour: ShimmerColour,         // BrightnessOnly | HueLerp(ToneToken) | Spectral
    pub contrast_floor: f32,           // 4.5 (3.0 for large text)
}

impl ShimmerDrive {
    // Thinking / Streaming / ToolRunning → active; Stalled → slowing/frozen;
    // AwaitingApproval / Done / Error / Cancelled → fading; Idle / Composing → inactive.
    pub fn from_state(s: &AgentState) -> ShimmerPhase { /* … */ }
}
```

## 10. Related components and references
Related: [status-text.md](status-text.md) (engine, tokens, stall ramp) · [verb-cycler.md](verb-cycler.md)
(text swaps interrupt the band) · [composer-status-strip.md](composer-status-strip.md) (uses the Codex
cadence) · family D border beams (same band maths along a path) · skeleton sweeps (family I).

| Ref | What we take | Licence note |
|---|---|---|
| Codex CLI shimmer (note 08) | cosine band, min half-width, 600 ms / 1 s / 4 s cadence, brightness-only, tier fallback | Apache-2.0 per note 08 (MIT in licensing.md) — portable with attribution |
| OpenCode / Kilo TextShimmer (note 08) | comet head/tail, `coast()` easing, compositor mask technique, fading state | MIT — portable with attribution |
| Warp shimmer_math (note 08) | grouped "...", padding concept | AGPL — study only |
| Claude Code glimmer (note 08) | direction by mode, whole-word flash idea | Proprietary — reference only |
| Cline `animate-shimmer` (note 08) | slow 5 s gradient as a calm extreme | Apache-2.0 — portable |
| MonoCode Shimmer (note 08) | one-shot 900 ms sweep (basis of `mask-wipe`) | MIT — portable |
| Arc UI Text shimmer (note 12) | pause off-view and resume at phase; `aria-busy` | Paid/unclear — study only |
| affectUI Shimmer Loader (note 11) | shared sweep across blocks | Paid — study only |
| prompt-kit, Motion Primitives, Magic UI, React Bits, GradientDeck (note 02) | market scan of 1–3 shimmers each; React Bits' many-knob shiny text | licences not recorded in our notes — study only |
| Cue wordmark [R029] | rainbow sweep for brand moments | Study only |
