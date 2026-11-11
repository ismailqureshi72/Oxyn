**Status:** draft v0.1 (2026-11-12).

# Spec: animated icon state transitions

Part of family L ([icons.md](icons.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). This spec owns **one icon turning into another
in the same box**: copy → check, play → stop, mic → waveform, plus → ×, spinner → ✓. It defines a
**transition engine**, a catalogue of **20 transition pairs** (content) and **16 transition styles**
(variants: 4 free + 12 Pro). Loops that are really loaders (spinners, orbs) stay in
[loaders.md](loaders.md); this spec only covers their entry and exit.

## 1. Purpose
Icon buttons answer the user's last action **in place**: "did it copy?", "is it recording?", "did the
server connect?". A toast or a separate label costs attention and space; a morph in the button the
user just pressed is read in the same glance. Reference products do exactly this: affectUI's message
actions blur the copy icon into a green check and revert "after a moment", spin regenerate once per
press and keep it spinning while regenerating, pop the thumb on vote, and swap the speaker for four
bouncing bars while reading aloud (research note 11); the composer send ↑ morphs to ■ in 160 ms
([composer-send-control.md](composer-send-control.md)); lucide-animated drives static and animated
states from the same geometry and only accepts motion that **conveys meaning**, rejecting plain
stroke-dash "drawing" for its own sake (research note 05 §7).

Rules:
1. **Same box, same optical centre.** Both glyphs share the icon grid ([icons.md](icons.md) §5.2); the
   transition never changes the button's size.
2. **Meaningful motion only.** A pair's motion says something (the check *writes* itself, the lock
   shackle *lifts*, the hand *rises*); decorative wiggles are not allowed.
3. **Interruptible.** A new request mid-transition reverses or retargets from the current progress;
   nothing jumps.
4. **Labels change with glyphs.** "Copy" → "Copied"; the accessible name switches at the transition's
   midpoint.
5. **Reduced motion keeps the information**, never the motion (a level meter stays a level meter).

## 2. State machine
```
                          request(B)                      progress = 1
  Rest(A) ───────────────────────────▶ Morphing(A→B, p) ─────────────────▶ Rest(B)
    ▲  ▲                                  │  ▲                                │ │ │
    │  │ hover (opt-in hint, ≤ 1 per 2 s)  │  │ request(A) mid-morph:          │ │ └── B is a loop ──▶ Looping(B)
    │  └── HintPlaying(A) ◀───────────────┘  │ reverse from current p         │ │          (spin / level-driven)
    │                                        └─────────────────────────────── │ │              │ request(C)
    │                                                                         │ │              ▼
    │      auto_revert after hold                                             │ │      Settling (finish beat ≤ 200 ms)
    ├──── Reverting(B→A) ◀── Holding(B, t) ◀──────────────────────────────────┘ │              │
    │                                                                           │              ▼
    │                                                       failure ───────────┘      Morphing(B→C)
    │                                                          ▼
    │                                     Failed(✕ glyph; host nudges 2 × 3 px) ── hold ──▶ Reverting
  Overlays: Disabled (frozen at current rest glyph, 40 %) · Reduced (crossfade only; loops → stills)
            · Offscreen (loops paused on the shared clock; state still advances) · Terminal (frame swap)
```

## 3. States table
| State | Shows | Primary / secondary (host) | Keyboard | Accessible name |
|---|---|---|---|---|
| `Rest(A)` | glyph A | press → request(B) | Space / Enter on host | A's label ("Copy") |
| `HintPlaying(A)` | small meaning hint on hover (copy sheets nudge 1 px) — opt-in | — | none (hover only) | unchanged |
| `Morphing(A→B)` | engine draws A→B at progress p | press again → reverse (toggles) or ignored (one-shots, §4) | live | switches at p = 0.5 |
| `Rest(B)` | glyph B (e.g. stop ■) | press → request(A) | live | B's label ("Stop") |
| `Holding(B)` | glyph B, auto-revert timer running | press → restart action; timer resets | live | B's label ("Copied") |
| `Reverting(B→A)` | B→A at the revert duration | press → request(B) from current p | live | switches at midpoint |
| `Looping(B)` | B animating (spin, level bars) | press → stop the loop (host decides) | live | B's label + host status |
| `Settling` | loop finishes its current beat | — | live | unchanged |
| `Failed` | ✕ or `!` glyph in `danger` | host retry | live | "Copy failed" |
| `Disabled` | rest glyph at 40 %, frozen | host reason tooltip | `aria-disabled` | label + reason |
| Reduced motion | crossfades; loops as stills (§5.3) | same | same | same |

## 4. Micro-interactions
- **Press → morph starts on pointer down** for toggles (play/stop, mic) and on **success** for
  one-shots (copy → check starts when the clipboard write resolves, never optimistically).
- **One-shots ignore re-press during Morphing**, then a re-press in `Holding` re-runs the action and
  restarts the hold (no second morph: the check pulses 1 → 1.08 → 1 in 160 ms to acknowledge).
- **Toggles reverse:** pressing play → stop mid-morph plays stop → play from the current progress
  with the same spring, so rapid presses never queue animations.
- **Midpoint label switch:** the accessible name and the tooltip text swap at p = 0.5; a tooltip that
  is open crossfades its text in 120 ms.
- **Loops settle before leaving:** a spinner finishes its current beat (≤ 200 ms) before morphing to
  ✓ (same rule as loaders.md "Done" morph).
- **Failure:** B is replaced by ✕ (or `!` for partial failure) with `spring.snap`; the **host** nudges
  2 × 3 px, the icon itself never shakes (send-control and loaders keep the same split).
- **Hover hint (opt-in, default off in IDE surfaces):** a ≤ 400 ms "what will happen" hint, e.g.
  the lock shackle lifts 1 px; at most once per 2 s per icon.

### 4.1 Transition-pair catalogue (20 pairs)
Timings are proposed unless a source is named. Engine durations come from §5.

| # | Pair | Kind | Motion (meaning) | Morph | Hold / revert | Reduced motion |
|---|---|---|---|---|---|---|
| 1 | `copy` → `check` | one-shot | sheets blur out, check writes left → right | 160 ms | hold 1500 ms, revert 200 ms | crossfade; "Copied" text 1500 ms |
| 2 | `play` ↔ `stop` | toggle | triangle's corners round into a square (point-matched) | 160 ms `spring.snap` | — | crossfade |
| 3 | `mic` → `audio-lines` | toggle + loop | capsule splits into 4 bars, bars follow input level | 200 ms, then level loop | — | 4 static bars stepped at 4 Hz (level stays visible) |
| 4 | `send` ↑ → `stop` ■ | toggle | owned by [composer-send-control.md](composer-send-control.md); listed for parity | 160 ms | — | crossfade |
| 5 | `plus` → `x` | toggle | rotate 45° while arms lengthen 1 px (menu open) | 180 ms `spring.snap` | — | crossfade |
| 6 | spinner → `check` | loop exit | arc finishes beat, closes into ring, check writes | 160 ms + 120 ms | hold 600 ms, fade 240 ms (loaders.md) | `◌` → ✓ swap |
| 7 | spinner → `x` | loop exit | arc stops, two strokes cross | 160 ms | stays until dismissed | swap |
| 8 | `plug-pending` → `plug-connected` | one-shot | plug slides 2 px into socket, clock badge exits | 240 ms `spring.soft` | — | swap |
| 9 | `lock` → `lock-open` | toggle | shackle lifts 2 px and pivots 20° (unlocking) | 200 ms `spring.soft` | — | swap |
| 10 | (none) → `hand` (approval needed) | enter | hand rises 3 px from the baseline and settles | 240 ms `spring.soft` | — | appear |
| 11 | `shield` → `shield-check` | one-shot | check writes inside the shield | 200 ms | — | swap |
| 12 | `context-ring` level n → n+1 | value | fill arc sweeps to the new level | 400 ms `ease.out` | — | jump to level |
| 13 | `refresh` spin | press + loop | one 360° turn per press; loops while busy | 800 ms / turn linear | — | static icon + host "Refreshing" |
| 14 | `rotate-ccw` (rewind / restore) | one-shot | one −360° turn, faster at the start | 600 ms `ease.out` | — | none |
| 15 | `chevron-down` expand | toggle | rotate 180° (or −90° → 0° for tree rows) | 160 ms `ease.in-out` | — | instant rotate |
| 16 | `eye` ↔ `eye-off` | toggle | slash draws across, lid closes 1 px | 160 ms | — | crossfade |
| 17 | `thumbs-up` / `-down` select | toggle | pop 1 → 1.2 → 1 and fill (filled form) | 240 ms `spring.soft` | — | fill only |
| 18 | `volume` → bars (read aloud) | toggle + loop | waves become 4 bars bouncing at 4 phases (affectUI idea) | 200 ms, loop 900 ms | — | static bars |
| 19 | `summarize-from-here` compaction | one-shot | three lines squeeze into one | 320 ms `ease.in-out` | — | swap |
| 20 | `bell` ring | one-shot | 3 damped swings ±12° → ±4°, ≤ 1 per 10 s | 600 ms | — | badge dot only |

## 5. Timing

### 5.1 Engine timing (per style)
| Transition | Duration | Delay | Easing / spring | Frame rate |
|---|---|---|---|---|
| `crossfade` out / in (blur 0 ↔ 2 px) | 160 ms overlapped (out 0–100, in 60–160) | 0 | `ease.out` | 60 Hz |
| `scale-swap` out (1 → 0.6, fade) / in (0.6 → 1) | 100 ms / spring | in starts at 60 ms | `ease.in` / `spring.snap` | 60 Hz |
| `draw` (new glyph's stroke written, old fades) | 120–240 ms by stroke length (≈ 1 px per 4 ms, proposed) | old fade 0–80 ms | `ease.out` | 60 Hz |
| `glyph` (terminal): A → mid frame → B | 2 frames × 80 ms | 0 | step | frame boundary |
| Reverse from mid-progress | remaining fraction of the forward duration | 0 | same curve, mirrored | 60 Hz |
| Auto-revert | hold 1500 ms (copy), 1200 ms (generic one-shots) then 200 ms | — | `ease.in-out` | 60 Hz |
| Loop settle (finish beat) | ≤ 200 ms | 0 | loop's own | — |
| Level-driven bars | input smoothed by a spring (stiffness 220, damping 20, mass 0.6, a value seen for voice orbs in research note 09; ours proposed) | 0 | spring | ≤ 30 fps |
| Hover hint | ≤ 400 ms | 120 ms hover intent | `spring.soft` | 60 Hz |
| Failure swap | 160 ms | 0 | `spring.snap` | 60 Hz |
| Batch (many icons change at once, e.g. "accept all") | stagger 30 ms, cap 8 icons then all at once | — | per style | 60 Hz |
| **Reduced motion** | every swap is a 0–80 ms (`dur.instant`) opacity crossfade; no blur, scale, rotation, drawing; loops become stills (§5.3); auto-revert hold kept | 0 | linear | repaint on change; level meters 4 Hz stepped |

### 5.2 Point matching (path morphs)
- Pro `path-morph` requires both glyphs resampled to the **same point count** at build (the
  clip-path / SVG morph rule; research note 09 §2.2); `xtask` resamples each pair to 32 points per
  sub-path (proposed) and stores the pair as one asset, so runtime does no matching.
- Free styles never morph paths; they swap whole glyphs, so any two icons form a pair.

### 5.3 Reduced-motion stills for loops
| Loop | Still |
|---|---|
| spinner | `◌` still (loader-glyph reduced glyph) or a static 3/4 arc |
| `audio-lines` / bars (mic, read aloud) | 4 bars at the current level, **stepped at 4 Hz** — the level never disappears while the mic is open ([composer-voice-input.md](composer-voice-input.md) privacy rule) |
| `refresh` | static icon; the host shows "Refreshing" |
| bell | no swing; a 4 px dot badge |

## 6. Variants (transition styles)
A style is **how** a pair transitions; the 20 pairs in §4.1 are content and work with every style
(path-based styles fall back to `crossfade` when a pair has no matched asset).

**Free (4)**
| Name | Idea | Surfaces |
|---|---|---|
| `icon-state/crossfade` | Default: blur-crossfade (old blurs out, new sharpens in), the affectUI copy → check feel | all GUI |
| `icon-state/scale-swap` | Old shrinks and fades, new springs in from 0.6 | all GUI |
| `icon-state/draw` | New glyph writes its stroke (check, cross, slash only; others crossfade) | all GUI |
| `icon-state/glyph` | Terminal: A → intermediate frame → B (e.g. `▶ ▪ ■`) | terminal, ide |

**Pro (12)**
| Name | One-line idea |
|---|---|
| `icon-state/path-morph` | True point-matched morph (triangle → square, plus → ×, capsule → bars) |
| `icon-state/spring-pop` | New glyph overshoots to 1.12 and settles with `spring.soft`; for celebratory one-shots |
| `icon-state/flip` | 3D flip on the Y axis, A on the front face, B on the back (180 ms) |
| `icon-state/slot-roll` | A rolls up and out, B rolls in from below, odometer style |
| `icon-state/mask-wipe` | A diagonal mask wipes A away revealing B underneath |
| `icon-state/stroke-retrace` | A un-writes its stroke, then B writes (meaning-bearing pairs only) |
| `icon-state/pixel-dissolve` | Glyphs as 16 × 16 pixels; A's pixels dissolve in a seeded order into B (pairs with `icon/pixel`) |
| `icon-state/dot-reform` | Glyph sampled to dots that drift into B's shape (pairs with `icon/dot-matrix` and the Oxyn mark) |
| `icon-state/particle-burst` | On success only: B appears with a 6-dot burst that fades in 300 ms |
| `icon-state/liquid` | Gooey metaball bridge between A and B (T3/T4; falls back to `crossfade`) |
| `icon-state/ink-fill` | B's filled form floods from the press point (Active toggles) |
| `icon-state/scramble` | Terminal: cycles 3 random width-safe glyphs before resolving to B |

## 7. Edge cases
- **Rapid presses / toggle spam:** reversals from current progress; no queue; a toggle pressed 5×
  in 300 ms ends in the state implied by an odd/even count, with one animation in flight.
- **Action fails after optimistic toggle** (mic permission denied): Morphing reverses to A and the
  host shows the reason; for one-shots, B never appears (they start on success).
- **Very fast success (< 50 ms):** one-shot still plays the full 160 ms morph so the user sees it.
- **Very slow action (> 400 ms):** one-shots show the host's spinner in the icon slot after 300 ms
  (`show_delay` from loaders.md), then pair #6 or #7.
- **Many icons at once** (accept-all over 40 hunks): stagger cap §5.1; offscreen icons switch with no
  animation.
- **Tiny sizes (12–16 px):** `draw` falls back to `crossfade` below 16 px (stroke writing is unreadable);
  `path-morph` uses the 16 master's matched asset.
- **RTL:** check writes right-to-left start is **not** mirrored (a check is not directional);
  rotating pairs (`rotate-ccw`, `refresh`) keep their direction; directional glyphs flip per
  [icons.md](icons.md) §7.
- **Offscreen / background window:** loops pause on the shared clock; state still advances, so a
  window returning to focus shows the current glyph without replaying.
- **Touch:** morph starts on touch-up for one-shots (avoids firing on scroll); toggles still on down.
- **Long-held state** (mic open for an hour): level loop runs at ≤ 30 fps and drops to 15 fps when the
  window is unfocused (shared-clock rule, [../architecture.md](../architecture.md)).
- **Dense IDE panels:** hover hints default off; `scale-swap` limited to 0.8 → 1 so neighbours aren't
  overlapped.

## 8. Accessibility
- The **host control's** accessible name follows the glyph ("Copy" → "Copied", "Start recording" →
  "Stop recording"); toggles expose `aria-pressed` (or `aria-checked` for switches).
- **Announcements:** one-shots announce once in the app's polite live region ("Copied to clipboard");
  failures announce assertively ("Copy failed"). Toggles do not announce (pressed state is enough).
- **Flashing:** no transition changes luminance of more than a 48 px area; bell swings ≤ 1 per 10 s;
  `particle-burst` is one burst per success; level bars move, never blink.
- **Reduced motion:** §5.1 last row and §5.3; information (levels, pressed state) always kept.
- **Contrast:** both glyphs ≥ 3:1; success green is a redundant channel (the check shape carries it).
- **Focus:** the focus ring is drawn by the host around the unchanged button box, never around the
  morphing glyph.

## 9. Props sketch
```rust
pub struct AnimatedIcon {
    pub from: IconName,                     // A
    pub to: IconName,                       // B (a TransitionPair is looked up for matched assets)
    pub shown: IconSide,                    // A | B — the controlled state; engine animates changes
    pub kind: PairKind,                     // OneShot { hold_ms: u32 } | Toggle | ToggleLoop(LoopDriver) | Value(f32)
    pub style: IconStateStyle,              // Crossfade (default) | ScaleSwap | Draw | Glyph | Pro(IconStatePro)
    pub failed: bool,                       // swap to Cross / Warn glyph
    pub hover_hint: bool,                   // default false (true in chat surfaces, proposed)
    pub size: IconSize,
    pub tone_from: IconTone,
    pub tone_to: IconTone,                  // e.g. Status(Success) for the check
    pub label_from: SharedString,           // "Copy"
    pub label_to: SharedString,             // "Copied"
    pub announce: Option<SharedString>,     // polite live-region text for one-shots
    pub motion: MotionLevel,
}

pub enum LoopDriver { Spin { period_ms: u32 }, Level(Signal<f32>), Bars { phases: u8, period_ms: u32 } }

/// Imperative handle, like lucide-animated's start/stop, for hosts that are not state-driven.
pub trait IconAnimHandle { fn start(&self); fn stop(&self); fn set_level(&self, level: f32); }
```

## 10. Related components and references
**Related:** [icons.md](icons.md) (glyphs and grid) · [plugin-icons.md](plugin-icons.md) (status badge
changes use pair #8 and the badge engine) · [animated-marks.md](animated-marks.md) ·
[composer-send-control.md](composer-send-control.md) (↑ → ■) · [composer-voice-input.md](composer-voice-input.md)
(mic → waveform, level privacy rule) · [message-bubble.md](message-bubble.md) (copy, regenerate,
thumbs, read-aloud actions) · [loaders.md](loaders.md) (loop exit, Done / Failed morph) ·
[approvals.md](approvals.md) (hand → ✓ / ⊘ glyph morph, 160 ms + 240 ms stroke).

| Reference | What we take | Licence note |
|---|---|---|
| lucide-animated (research note 05 §7) | same geometry for static + animated; meaning-only motion rule; start/stop handle | MIT — portable with attribution (we write Rust; behaviour + any ported path data credited) |
| itshover (note 05 §7) | start/stop handle shape | MIT code; its brand icons are third-party trademarks — not used (see [plugin-icons.md](plugin-icons.md)) |
| affectUI Message Actions (note 11) | copy blurs into check and reverts; regenerate spin per press; thumb pop; speaker → 4 bars | Paid / unclear — study only; our timings |
| Composer send control (this repo) | 160 ms `spring.snap` glyph morph, reduced = 0–80 ms crossfade | Ours |
| Clip-path / SVG morph point-count rule (note 09 §2.2) | equal point counts for morphs | Technique; GSAP MorphSVG free but not Rust — idea only |
| Voice-orb level smoothing (note 09 §2.2, amplitude-reactive orb row) | spring smoothing of audio level | Study only; constants re-chosen by us (proposed) |
| Stagger [R109] | state-machine icons (idle / loading / success / error) | Paid generator — study only |
| transitions.dev "Spinner to check morph" (note 17 §14, a Pro item) | spinner → check as a named effect | Terms forbid competing kits ([../licensing.md](../licensing.md)) — study only |
