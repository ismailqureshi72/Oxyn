**Status:** draft v0.1 (2026-11-11).

# Spec: composer thinking-effort control

Follows the 10-point standard in [README.md](README.md) at the depth of the worked example
[composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro variants** on one state
machine. Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
Reasoning models expose a dial between "answer fast" and "think hard". The effort control lives in the
composer row next to the model pill (or inside the model popover in the combined layout). The user is
deciding how much time and how many tokens the **next** turn may spend thinking. They need to see the
current level at a glance, change it in one gesture or one key, know which levels this model allows,
and understand when the control does nothing (non-reasoning model). Raising effort to the top tiers is
a deliberate act, so it gets a moment of feedback: the Codex CLI "effort ignition" shows how a settings
change can feel physical [research 08 §1.1, §7 #2].

Observed shapes are mostly **discrete steppers, not continuous sliders** (research 03): Claude desktop
Effort menu (⌘⇧E) and a thinking toggle (⌥T in the CLI); Codex CLI levels minimal → low → medium → high
→ xhigh → max → ultra with Alt+. / Alt+, stepping; Verdent "Thinking depth"; ChatGPT Light / Standard /
Extended / Heavy, later a slider (research 04); Claude Code status-row effort glyphs `○ ◐ ● ◉` for low /
medium / high / max [research 08 §1.3, LEAK — numbers/behaviour reference only]. pitus shows effort
"Medium" with signal bars in the composer [R021]; BoardUI shows "Effort Medium / Faster–Smarter" [R095].

## 2. State machine

```
                 model has no adjustable reasoning
   ┌──────────────────────────────────────────────────────────▶ Unsupported ─▶ (Hidden | Disabled(reason))
   │
 Idle(level) ──hover──▶ Hover ──click / Mod+Shift+E──▶ Open(menu | slider | dial)
   │  ▲                                                     │  drag / ↑↓ / 1–n
   │  │                                                     ▼
   │  │                                          Adjusting(preview level)
   │  │                                                     │ release / Enter
   │  │            ┌────────────────────────────────────────┤
   │  │   lower    ▼                       raise into ignite tier ▼
   │  └──── Committed ◀────── Discharge          Igniting(style, tier) ── 0.9–1.6 s ──▶ Committed
   │            │
   │            │ agent busy (turn in flight)
   │            ▼
   │      PendingNext(level) ── run ends ──▶ Idle(level)
   │
   ├─ Alt+. / Alt+, (no popover) ─▶ Stepping ─▶ Committed (or Igniting)
   ├─ model switched, level not allowed ─▶ Clamped(from, to) ─ 2.4 s ─▶ Idle(to)
   ├─ Locked(policy)              ├─ ToggleOnly(on/off) for models with binary thinking
   └─ Auto (model/router chooses effort) — shows "Auto" and the resolved level once known
```

Rules:
1. Effort is a **request parameter for the next turn**. A change while `Thinking`/`Streaming`/
   `ToolRunning` never alters the running turn; it enters `PendingNext`.
2. Levels are an ordered ladder owned by Oxyn (`EffortLevel`), mapped to provider labels per model
   (`allowed` + `labels`). The control only ever offers `allowed` levels.
3. **Ignition** fires only on a raise *into* an ignite tier (default: the top two allowed levels) and
   only on commit, never while dragging. Lowering plays a short "discharge" instead.
4. Ignition does not replay if the same tier was ignited < 10 s ago (prevents spam when flicking).
5. On model switch, the level clamps to the nearest allowed level **at or below** the old one (never
   silently raises cost), and the control shows `Clamped` with a reason.

## 3. States table

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle(level)` | chip: glyph (bars / chevrons / ring) + label "High"; tinted per tier | open | — | Tab focus; `Alt+.` raise, `Alt+,` lower (as Codex CLI, research 03); `Mod+Shift+E` open (as Claude desktop, research 03) |
| `Hover` | bg `chip.hover`; tooltip after 500 ms: "Thinking effort: High · Alt+. / Alt+," | open | — | — |
| `Focus` | 2 px focus ring | open | — | Enter/Space open; ← → step without opening (slider variants) |
| `Open(menu)` | list of allowed levels top→bottom high→low, each with glyph + one-line hint ("Faster", "Deeper reasoning, slower") | choose | "Auto" row if supported | ↑↓ move, 1–n jump, Enter commit, Esc cancel |
| `Open(slider)` | horizontal track with one detent per allowed level, end labels "Faster ← → Smarter" | drag / click a detent | — | ← → step, Home/End min/max, Esc revert |
| `Open(dial)` | 240° arc, detents, knob | rotate (drag vertical) | — | ↑↓ step, Home/End |
| `Adjusting(preview)` | thumb/knob follows pointer, label previews level, no ignition yet | release commits | Esc reverts to start level | — |
| `Stepping` | chip label rolls one step; glyph fills/empties one unit | — | — | repeated `Alt+.` steps; held key auto-repeats at 180 ms |
| `Igniting(style, tier)` | composer background washed with travelling colour bands; prompt glyph recolours (`›` amber at tier A, `»` violet at tier B); sparks `· ✦ ✧` | — (non-blocking; typing continues) | — | all keys pass through |
| `Discharge` | tier tint fades back; glyph returns to neutral `›` | — | — | — |
| `Committed` | new level; 1 accent flash on the glyph | — | — | — |
| `PendingNext(level)` | current level + small dashed "→ Max" badge; tooltip "Applies to your next message" | — | cancel (× on badge) | Esc on badge |
| `Clamped(from, to)` | label shows "High"; inline note for 2.4 s: "Ultra isn't available on Sonnet 5" | — | "Change model" link | — |
| `ToggleOnly(on)` | switch-chip "Thinking" on/off (single binary) | toggle | — | Enter/Space; `Alt+T` (proposed; Claude CLI uses ⌥T for its thinking toggle, research 03) |
| `Auto(resolved?)` | "Auto" or "Auto · High" once reported | open | — | — |
| `Unsupported → Hidden` | not rendered; row reflows over 160 ms | — | — | — |
| `Unsupported → Disabled(reason)` | chip muted 50 %, no chevron; tooltip "Sonnet 5 doesn't use adjustable reasoning" | — | — | focusable, Enter announces reason |
| `Locked(policy)` | lock 10 px + level; tooltip "Set by your workspace" | — | — | Enter announces |

## 4. Micro-interactions
- **Hover / press / focus:** same chip grammar as the model pill (bg fade, 0.97 press spring, 2 px ring).
- **Signal bars glyph:** 4 bars in a 14 × 14 px box, bar width 2 px, gap 1.5 px, heights 4 / 7 / 10 / 13 px.
  Filled bars = level position scaled to 4 (a 7-level ladder maps 1,1,2,2,3,4,4). Raising fills the next
  bar bottom-up (scaleY 0 → 1); lowering drains top-down.
- **Chevrons glyph:** `›` (neutral) → `›` amber (tier A) → `»` violet (tier B); the glyph is also the
  composer prompt glyph in TUI and `glyph` variants, the Codex pattern [research 08 §1.1].
- **Ring glyph:** quarter-filled ring `○ ◔ ◑ ◕ ●` style (our own drawing, 12 px, 1.5 px stroke); arc
  length tweens on change.
- **Slider detents:** the thumb snaps to the nearest detent with a spring; passing a detent while dragging
  gives a 1 px thumb "tick" (scale 1.08 → 1) and an optional haptic on touch.
- **Label roll:** level label rolls up on raise, down on lower (tabular width fixed to the longest label).
- **Ignition (re-implemented from Codex CLI numbers; Codex licence is permissive, see §10):**
  - Three styles **Wave / Aurora / Pulse**, picked at random and never repeating the previous style.
  - Durations for tier A / tier B: Wave 1000 / 1300 ms, Aurora 1300 / 1600 ms, Pulse 900 / 1250 ms.
  - Accent "charge" on the prompt glyph ramps in over 150 ms; glyph accent alpha 0.86.
  - Bands painted across the composer background, alpha capped at 0.6; colours (dark bg) tier A
    (255,178,66), (255,214,120), (255,120,60); tier B (186,130,255), (255,120,220), (120,170,255);
    light-bg sets are darker (Oxyn tokens `effort.ignite.a.*`, `effort.ignite.b.*`).
  - Band profile: cosine crest; Wave half-width 9 columns (GUI: 9 × 8 px = 72 px proposed), Pulse 4.5
    columns (36 px proposed); easing cubic in-out; envelope linear fade-in/out.
  - Sparks `· ✦ ✧` start at 900 ms, 100 ms per frame.
  - **Protected regions:** bands and sparks never paint over the draft text or attachment rows; in GUI the
    draft text layer sits above the wash, and the wash is clipped to the composer's padding box.
  - All values in [research 08 §1.1, SRC].
- **Discharge (ours):** the tier tint and glyph colour fade to neutral over 200 ms; no bands.
- **Clamped:** label cross-fades to the clamped level, and a 1-line note slides down from the chip
  (y −4 → 0) and auto-hides after 2.4 s.
- **PendingNext:** dashed badge slides out of the chip; merges back when the run ends.
- **Unsupported → Hidden:** chip shrinks width → 0 and fades; neighbours glide left.

## 5. Timing table

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Hover bg | 80 ms | 0 | ease-out | — |
| Press | ~120 ms | 0 | spring 600 / 30 | — |
| Menu open / close | ~200 ms / 120 ms | 0 | spring 380 / 32 / ease-in | same as model popover |
| Signal-bar fill / drain | 140 ms per bar | 30 ms stagger | ease-out | — |
| Ring arc tween | 220 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | — |
| Slider thumb snap | ~160 ms | 0 | spring 520 / 34 | — |
| Detent tick | 90 ms | 0 | ease-out | — |
| Label roll | 220 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | — |
| Key auto-repeat (`Alt+.` held) | 180 ms interval | 400 ms initial | — | — |
| Ignition charge ramp | 150 ms | 0 | linear | [research 08 §1.1] |
| Ignition Wave A / B | 1000 / 1300 ms | 0 | cubic in-out, linear envelope | 33 ms tick in source; GPUI may render at display rate with the same envelope |
| Ignition Aurora A / B | 1300 / 1600 ms | 0 | cubic in-out | — |
| Ignition Pulse A / B | 900 / 1250 ms | 0 | cubic in-out | — |
| Sparks | 100 ms / frame | 900 ms | step | — |
| Ignition cool-down (no replay) | 10 s | — | — | proposed |
| Discharge | 200 ms | 0 | ease-out | — |
| Clamped note in / hold / out | 160 / 2400 / 160 ms | 0 | ease-out / — / ease-in | — |
| Hide (unsupported) | 160 ms | 0 | ease-in-out | neighbours glide |
| **Reduced motion** | no bands, no sparks, no rolls; glyph and tint change instantly with an 80 ms opacity crossfade; bars set instantly | — | linear | Codex also disables ignition without animation support [research 08 §1.1] |

## 6. Variants

### Free (6)
| Name | Idea |
|---|---|
| `effort-control/segmented` | Inline segmented control of allowed levels (2–4 visible; more collapses into a menu) |
| `effort-control/signal-bars` | Chip with 4 signal bars + label, menu on click (pitus-style [R021]) |
| `effort-control/slider` | Detented slider with "Faster ← → Smarter" end labels |
| `effort-control/chevrons` | Prompt glyph `›` / `›` / `»` doubles as the effort indicator (TUI and minimal chats) |
| `effort-control/toggle` | Binary "Thinking" switch for models with on/off reasoning only |
| `effort-control/menu` | Plain text chip "Effort: High ⌄" for dense IDE panels |

### Pro (14)
| Name | Idea |
|---|---|
| `effort-control/ignition` | Full Wave / Aurora / Pulse ignition with sparks and recoloured prompt glyph |
| `effort-control/dial` | Rotary knob with detents and an arc that glows brighter per tier |
| `effort-control/thermo` | Vertical thermometer; liquid rises with a wobble spring |
| `effort-control/ember` | Glyph smoulders at low, glows at high, flares on ignite |
| `effort-control/stack` | Stacked translucent layers that thicken with effort |
| `effort-control/orbit-count` | Dots orbit the chip; orbit count = level |
| `effort-control/tachometer` | Needle gauge with redline zone for ultra |
| `effort-control/pixel-meter` | Pixel column that fills cell by cell, ignition as pixel burst |
| `effort-control/ascii-flame` | TUI flame of glyphs that grows per level |
| `effort-control/charge-ring` | Ring around the send button charges to the effort level (pairs with send control) |
| `effort-control/hold-to-boost` | Press and hold send to temporarily boost to max for this message |
| `effort-control/cost-hint` | Shows app-supplied relative time/cost hint per level, digits rolling |
| `effort-control/border-beam` | Composer border beam speeds up with effort |
| `effort-control/haptic-wheel` | Mobile scroll wheel with haptic detents |

## 7. Edge cases
- **Long labels / i18n** ("Erweitert"): chip width fixed to the longest label in the active ladder;
  slider end labels drop below 200 px width.
- **Ladders of different lengths** (2 to 8 levels): segmented shows ≤ 4, otherwise menu; slider spacing
  = track width / (n − 1), min 16 px per detent, else falls back to menu.
- **RTL:** slider and bars mirror (low on the right); chevron glyphs mirror to `‹` / `«`.
- **Tiny sizes (16–20 px):** signal bars scale to 12 px box (heights 3/5/8/11), label hidden, tooltip only.
- **Dense IDE panels:** `menu` variant; ignition reduced to the glyph recolour (no bands) when the
  composer is < 280 px wide.
- **Touch:** slider thumb hit area 44 px; dial becomes a horizontal wheel.
- **Slow networks:** effort is local state; nothing waits on the network.
- **Very fast agents:** if the turn ends within 300 ms of a mid-run change, skip `PendingNext`.
- **Very slow agents:** `PendingNext` persists; repeated changes just update the badge.
- **Model switch while menu open:** menu re-renders with the new ladder; highlighted level clamps.
- **Per-message override:** a queued message keeps the effort it was queued with; its chip shows it.
- **Ignition during typing:** must never steal focus, move the caret or repaint draft glyphs.
- **Unfocused window:** ignition does not play if the window is unfocused at commit time (shared clock
  slows in background, architecture.md); the glyph simply recolours.

## 8. Accessibility
- Segmented / menu: `radiogroup` with `radio` items; slider and dial: `slider` with `aria-valuemin`,
  `aria-valuemax`, `aria-valuenow` (index) and `aria-valuetext` ("High, deeper reasoning, slower").
- Labels per state: "Thinking effort: High. Change effort", "Thinking effort: Max, applies to your next
  message", "Thinking effort unavailable: Sonnet 5 doesn't use adjustable reasoning", "Thinking effort set
  by workspace: Medium".
- Live region (polite): on commit "Effort set to Max"; on clamp "Effort lowered to High, Ultra isn't
  available on Sonnet 5"; ignition itself is decorative (`aria-hidden`).
- Tier colours never carry meaning alone: label + glyph shape always change too.
- Ignition bands keep draft text contrast ≥ 4.5:1 (alpha cap 0.6 and text layer above the wash; verify
  per theme in the Studio contrast check).
- No flashing above 3 Hz: Pulse style must stay under 3 flashes per second (it does at ≥ 900 ms).
- Focus order: model pill → effort → mode → permission → context → send.

## 9. Props sketch
```rust
pub struct EffortControl {
    pub agent: AgentStateRef,                  // busy states -> PendingNext; Idle/Composing -> immediate
    pub level: EffortSetting,                  // Level(EffortLevel) | Auto { resolved: Option<EffortLevel> } | Toggle(bool)
    pub support: ReasoningSupport,             // from ModelInfo.caps: None | Toggle | Levels { allowed, default, labels }
    pub variant: EffortVariant,
    pub unsupported: UnsupportedDisplay,       // Hide | Disable (default Disable in chat, Hide in IDE)
    pub ignite: IgnitePolicy,                  // Off | TopTiers(u8 = 2) | From(EffortLevel)
    pub ignite_styles: &'static [IgniteStyle], // Wave, Aurora, Pulse
    pub ignite_cooldown: Duration,             // default 10 s
    pub prompt_glyph: bool,                    // recolour composer prompt glyph (› / »)
    pub hint: Option<Box<dyn Fn(EffortLevel) -> Option<SharedString>>>, // app-supplied "slower, more tokens"
    pub locked: Option<SharedString>,          // policy reason
    pub shortcuts: EffortShortcuts,            // raise Alt+., lower Alt+,, open Mod+Shift+E, toggle Alt+T (proposed defaults)
    pub labels: EffortLabels,
    pub motion: MotionLevel,
    pub on_change: Callback<EffortSetting>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffortLevel { Minimal, Low, Medium, High, XHigh, Max, Ultra }   // ladder seen in Codex CLI (research 03)

fn clamp_on_model_switch(cur: EffortLevel, allowed: &[EffortLevel]) -> EffortLevel {
    allowed.iter().copied().filter(|l| *l <= cur).max().unwrap_or(allowed[0]) // never raise silently
}
```

## 10. Related components and references
**Related:** model picker ([composer-model-picker.md](composer-model-picker.md)) — supplies
`ReasoningSupport`; combined "model + effort" layout embeds this control in the popover; send control
(`charge-ring` variant); thinking row / reasoning block (shows "thought for Ns" with the effort glyph);
context meter (higher effort → more output reserve); queued-message chips (per-message effort).

**References** (study only unless MIT/Apache):
- Codex CLI effort ignition (`bottom_pane/effort_ignition.rs`, `effort_ignition_styles.rs`) and effort
  stepping — research 08 §1.1 [SRC] and research 03. Licence: permissive (licensing.md lists Codex as MIT;
  research note 08 reads Apache-2.0 — re-check before porting). Port with attribution and keep the notice.
- Claude desktop Effort menu ⌘⇧E, CLI thinking toggle ⌥T (research 03, docs) — proprietary, reference
  only. Claude Code effort glyphs `○ ◐ ● ◉` (research 08 §1.3, leaked source) — numbers/behaviour only,
  never code.
- ChatGPT Light / Standard / Extended / Heavy → slider (research 04) — closed, study only.
- Verdent "Thinking depth", Factory `--reasoning-effort`, Paradev thinking toggle, OpenCode Ctrl+T model
  variants (research 03) — study only (OpenCode MIT if porting).
- [R021] pitus composer effort "Medium" with signal bars — study only.
- [R095] BoardUI model picker "Effort Medium / Faster–Smarter" — free tier MIT, Pro study only.
- affectUI Model Picker "switch on extended thinking and its effort metric unfolds" (research 11) — Pro,
  study only.
- assistant-ui "reasoning effort" element (research 01) — MIT, portable with attribution.
- Arc UI morph select signal-bars priority icon (research 12) — study only.
