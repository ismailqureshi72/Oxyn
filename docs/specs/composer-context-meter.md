**Status:** draft v0.1 (2026-11-11).

# Spec: composer context-window meter

Follows the 10-point standard in [README.md](README.md) at the depth of the worked example
[composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro variants**, one state
machine. Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
Long agent sessions fill the model's context window with system prompt, tool schemas, memory files,
attachments and history. When it fills, quality drops, requests fail or the app compacts. The meter sits
in the composer row (or status bar) and answers: *how full is it*, *what is filling it*, and *what should
I do now* (compact, hand off to a new thread, remove a tool server, drop an attachment). It must update
live while the agent streams without becoming a distraction, and only raise its voice at thresholds.

Forms seen (research 03, 05): a **ring/donut** next to the model picker that turns clay above 80 % and
breathes (Claude desktop, r = 10 [research 08 §1.4, RE]); a **% text in the footer** ("N% context left",
Codex CLI); a **card with per-category breakdown** (Claude desktop); a **sidebar stat block** (OpenCode:
tokens, %, cost); a **coloured grid** — Claude Code `/context` with categories System prompt, System
tools, MCP tools, Memory files, Skills, Messages, Free space, an autocompact buffer, a 10×10 grid for
200K and 20×10 for 1M (research 05); Cursor's **context ring → breakdown tray** (research 05); Arc UI
"Context window (200-cell grid)" and affectUI "Context Window (cells)" / "Context Map (treemap with
compaction)" (research 01). Thresholds seen in third-party tools: orange ≥ 60 % / red ≥ 85 %, and
yellow ≥ 50 % / red ≥ 75 % (research 05).

## 2. State machine

```
           first usage report                                    ratio ≥ warn_at
 Unknown ───────────────────▶ Normal(ratio) ◀────────────────────────────▶ Warning(ratio)
    │  (no window size)            │   ▲          ratio < warn_at − 3 %        │  ratio ≥ critical_at
    │                              │   │ (hysteresis)                          ▼
    ▼                              │   │                                Critical(ratio)
 Estimating(≈) ── exact usage ─────┘   │                                      │ ratio ≥ compact_prompt_at
   (chars/4 while streaming,           │                                      ▼
    reconciled at turn end)            │                              CompactSuggested ── dismiss ──▶ Critical (snoozed)
                                       │                                      │ "Compact now" / auto-compact fires
                                       │                                      ▼
                                       │                                 Compacting ── fail ──▶ CompactFailed(retry)
                                       │                                      │ done
                                       └────────── Compacted(Δ) ◀─────────────┘
                                                   (−84K float, drains to new ratio)
 Overlays: Live (agent streaming → digits update at ≤ 4 Hz)  ·  Hover / Focus / Open(popover)
 Specials: Overflow(> 100 %: next request will fail) · Stale(offline / no report) · Disabled(reason)
```

Rules:
1. `ratio = used / window`, where `used` includes the **reserved output / autocompact buffer** as its own
   category (shown hatched) so the visual "full" matches when the app actually compacts or fails.
2. Thresholds (proposed defaults, all props): `warn_at = 0.70`, `critical_at = 0.85`,
   `compact_prompt_at = 0.90`. Hysteresis of 3 percentage points stops flapping when a turn hovers on a
   threshold.
3. While `Streaming`/`Thinking` the count is an **estimate** (prefix `≈`); exact usage from the provider
   replaces it at turn end and the digits roll to the corrected value (up or down).
4. The meter never blocks sending by itself; `Overflow` tells the send control to go `Disabled(reason)`.
5. Auto-compact (if the app has it) is announced **before** it fires: at `compact_prompt_at` the prompt
   says "Auto-compacts at 95 %"; the meter never compacts silently.

## 3. States table

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Unknown` | ring outline only, no number; tooltip "Context size not reported" | open (shows what is known) | — | Tab focus, Enter open |
| `Estimating(≈)` | ring/bar fill + "≈ 41%" label (if labelled variant) | open | — | — |
| `Normal(ratio)` | neutral fill (`context.fill`), label "41%" or "82K / 200K" per variant | open breakdown | — | Enter/Space open; `/context` slash command opens it (app wires) |
| `Hover` | tooltip after 400 ms: "82.4K of 200K tokens · 41% · 118K free" | open | — | — |
| `Focus` | 2 px focus ring | open | — | — |
| `Open(popover)` | header total + stacked bar/grid + category rows + actions | close | per-row actions (remove attachment, disable MCP server, view memory file) | ↑↓ rows, Enter row action, Esc close |
| `Live` (overlay) | digits roll, arc grows smoothly, `≈` prefix | — | — | — |
| `Warning(ratio)` | fill → `context.warn` (amber); label weight 500 | open | — | — |
| `Critical(ratio)` | fill → `context.critical` (red); ring breathes (opacity 1 ↔ 0.75, 2.4 s) | open | — | — |
| `CompactSuggested` | prompt chip above the composer: "Context 91% full · Compact now · New thread with summary · ✕" | compact now | hand off, dismiss (snooze until +5 %) | `Mod+Shift+K` compact (proposed), Esc dismiss when chip focused |
| `Compacting` | ring becomes an indeterminate arc sweeping, icon "arrows-in"; label "Compacting…"; transcript marker appears | — | cancel (if adapter supports) | Esc cancel |
| `Compacted(Δ)` | fill drains to new ratio; "−84K" floats up and fades; popover shows "Compacted 2 min ago" | open | "View summary" → compaction marker | — |
| `CompactFailed` | danger dot; chip "Couldn't compact · Retry" | retry | open details | Enter retry |
| `Overflow` | ring full + red notch; label "Over by 12K" | open | "Compact now" | — |
| `Stale` | fill at 50 % opacity, tooltip "Last updated 3 min ago" | open | — | — |
| `Disabled(reason)` | muted outline, tooltip reason | — | — | focusable, Enter announces |

**Popover content (all variants):**
- Header: `112.4K / 200K tokens · 56%` (tabular digits) + model name; `≈` while estimating.
- Visual: stacked horizontal bar 8 px tall (popover width − 32 px) or a cell grid (see `grid` variant).
- Category rows, fixed order and colour tokens: **System** (`ctx.system`), **Tools** (`ctx.tools`,
  expandable per MCP server with tool count and tokens), **Memory & rules** (`ctx.memory`),
  **Skills** (`ctx.skills`), **Files & attachments** (`ctx.files`, per file, removable), **History**
  (`ctx.history`, incl. summarised turns), **Reasoning carried** (`ctx.reasoning`, only if the adapter
  reports it), **Reserved** (hatched, output + autocompact buffer), **Free**. Each row: 8 px swatch,
  label, tokens, % of window; rows < 0.5 % fold into "Other".
- Footer actions: "Compact now", "New thread with summary", "Open full view" (`/context`), and the
  auto-compact note "Auto-compacts at 95 %" when enabled.

## 4. Micro-interactions
- **Ring:** 16 px diameter in the composer row (20 px in the status bar), stroke 2 px, track at 18 %
  opacity, round cap; starts at 12 o'clock, clockwise (counter-clockwise in RTL). `ring-segmented`
  draws each category as its own arc with 1 px gaps.
- **Bar:** 4 px tall, 48–96 px wide, radius 2 px; `bar-segmented` shows categories.
- **Grid:** cells of `window / cells` tokens: 10 × 10 for windows ≤ 200K, 20 × 10 for 1M (the Claude Code
  `/context` layout, research 05); cell 6 px, gap 2 px in the popover; filled cells coloured in category
  order (row-major), reserved cells hatched, free cells hollow 1 px outline. On change, cells fill one by
  one with a 12 ms stagger (cap 160 ms total).
- **Rolling digits:** odometer per digit, tabular figures, top/bottom 18 % mask fade, digits roll **up**
  when the value grows and **down** when it shrinks — after OpenCode AnimatedNumber (560 ms,
  cubic-bezier(0.22, 1, 0.36, 1)) [research 08 §1.5, MIT]. In the 16 px chip we use 320 ms.
- **Live updates while streaming:** values are sampled at ≤ 4 Hz (every 250 ms); between samples the arc
  eases toward the target (critically damped spring) so it never jumps; a count-up adapts its step to the
  gap — the same idea as Claude Code's adaptive token count-up (+3 / 15 % / +50 per 50 ms tick)
  [research 08 §1.3, LEAK — behaviour reference only].
- **Threshold crossing:** colour cross-fades over 240 ms; on entering `Warning` the ring does one scale
  pulse 1 → 1.12 → 1; on `Critical` it starts a slow breathe (the Claude desktop ring "breathes" above
  80 % [research 08 §1.4, RE]; our timing is our own).
- **Compact prompt chip:** unfolds upward above the composer (same choreography as queued chips in the
  send control); composer does not move.
- **Compacting:** the filled arc sweeps as a 25 % indeterminate segment (1.2 s per turn); grid cells
  "squeeze" toward the start (history cells collapse into a single summary-coloured block).
- **Compacted:** fill drains from old to new ratio; the delta label "−84K" rises 8 px and fades; the
  numbers roll down. A compaction marker is inserted into the transcript (related component).
- **Hover in popover:** hovering a category row highlights its segment / cells (others dim to 40 %).
- **Overflow:** a 2 px red notch appears at 12 o'clock; the send control shakes once if the user presses
  Enter (that shake belongs to the send control spec).

## 5. Timing table

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Tooltip | 120 ms fade | 400 ms | ease-out | — |
| Popover open / close | ~200 ms / 120 ms | 0 | spring 380 / 32 / ease-in | shared composer popover |
| Live sample rate | 250 ms (≤ 4 Hz) | — | — | numbers only; arc interpolates continuously |
| Arc / bar tween to sample | ~300 ms | 0 | critically damped spring (stiffness 170, damping 26) | — |
| Digit roll (popover) | 560 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | [research 08 §1.5, MIT] |
| Digit roll (chip) | 320 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | — |
| Turn-end reconcile roll | 400 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | `≈` fades out 120 ms |
| Grid cell fill | 120 ms each | 12 ms stagger, cap 160 ms | ease-out | — |
| Threshold colour | 240 ms | 0 | ease-in-out | — |
| Warning pulse | 360 ms | 0 | spring 400 / 18 | once |
| Critical breathe | 2.4 s loop | 0 | sine, opacity 1 ↔ 0.75 | shared clock; pauses when unfocused |
| Compact chip unfold | 200 ms | 0 | ease-out | same as queued chip |
| Compacting sweep | 1.2 s per turn | 0 | linear | — |
| Compacted drain | 600 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | — |
| Delta float "−84K" | 900 ms (rise 8 px + fade) | 100 ms | ease-out | — |
| Category hover highlight | 120 ms | 0 | ease-out | — |
| **Reduced motion** | values and colours swap instantly (≤ 80 ms opacity); no roll (digits replace), no breathe (static critical colour + "!" glyph), no sweep (static "Compacting…" text), no float (delta shown as text for 2 s) | — | linear | — |

## 6. Variants

### Free (6)
| Name | Idea |
|---|---|
| `context-meter/ring` | 16 px ring in the composer row, popover breakdown; breathes when critical |
| `context-meter/ring-segmented` | Ring split into category arcs |
| `context-meter/bar` | Thin bar (segmented optional) with "56%" label; fits status bars |
| `context-meter/grid` | Cell grid (10×10 / 20×10) like a `/context` map, in chip (mini 5×4) and popover (full) |
| `context-meter/text` | Footer text "44% context left" for TUI / IDE status lines |
| `context-meter/card` | Session-rail card: total, stacked bar, category list, actions always visible |

### Pro (14)
| Name | Idea |
|---|---|
| `context-meter/treemap` | Categories as a treemap; compaction squeezes history tiles into one |
| `context-meter/liquid` | Liquid level in a capsule; sloshes on big jumps, drains on compaction |
| `context-meter/radial-ticks` | Radial tick gauge whose ticks light in sequence (study [R025]) |
| `context-meter/sediment` | Layers settle like sediment per category; compaction compresses layers |
| `context-meter/pixel-fill` | Pixel cells fill with dither; critical cells flicker red |
| `context-meter/heat-ring` | Ring hue shifts continuously from cool to hot |
| `context-meter/timeline` | Per-turn token growth sparkline; click a turn to see what it added |
| `context-meter/mcp-budget` | Per-server tool cost bars with on/off switches (budget against limit) |
| `context-meter/compaction-film` | Compaction plays as a short film strip of turns folding into a summary |
| `context-meter/split-flap` | Split-flap percentage digits |
| `context-meter/orb` | Orb that grows denser as context fills |
| `context-meter/handoff-card` | At critical, morphs into a "Start new thread with summary" card |
| `context-meter/cost-overlay` | Adds running cost and cache-hit ratio to the popover, digits rolling |
| `context-meter/notch` | Desktop island ring with breakdown on hover |

## 7. Edge cases
- **Huge windows (1M+):** grid switches to 20 × 10; labels use `1.2M`; categories < 0.5 % fold to "Other".
- **Tiny windows (8K):** cells = 8K / 100 = 80 tokens each; fine.
- **Unknown window size:** show absolute tokens only ("82K tokens"), no ring fill, no thresholds.
- **Window changes on model switch:** ratio recomputes; the arc tweens; crossing into `Overflow` because
  the new model is smaller sets the model picker's `Incompatible` reason (shared data).
- **Estimates wrong by a lot** (e.g. images): the turn-end reconcile may jump > 10 %; roll anyway, no
  extra emphasis.
- **Long category names / many files:** rows ellipsis at 1 line; files list caps at 8, then "+12 more".
- **RTL:** ring direction counter-clockwise, bars fill right-to-left, digits stay LTR.
- **Tiny sizes (16–20 px):** ring only (no label); thresholds still change colour **and** add a 4 px dot
  (amber / red) so colour isn't the only signal.
- **Dense IDE panels:** `text` or `bar` variant; popover becomes a side card if vertical space < 240 px.
- **Touch:** tap opens popover as bottom sheet; rows 44 px; remove actions behind swipe or "…" menu.
- **Slow networks:** `Stale` after 30 s without a usage report while the agent is busy; keep last value.
- **Very fast agents:** many reports in a burst coalesce to the 250 ms sample; at most one threshold
  announcement per 5 s.
- **Very slow agents:** `Live` overlay may last minutes; breathe pauses when the window is unfocused.
- **Compaction while the popover is open:** popover rows animate to new values; actions disabled during
  `Compacting`.
- **Subagents with their own windows:** the meter shows the parent thread only; subagent windows belong to
  the subagent tree component.

## 8. Accessibility
- Chip: `meter` role (or `progressbar` where `meter` is unsupported) inside a `button` that opens the
  popover; `aria-valuemin=0`, `aria-valuemax=window`, `aria-valuenow=used`, `aria-valuetext` "112
  thousand of 200 thousand tokens used, 56 percent, 88 thousand free" (prefix "about" while estimating).
- Labels per state: "Context: 56 percent used. Show breakdown", "Context warning: 74 percent used",
  "Context critical: 91 percent used. Compact suggested", "Compacting context", "Context compacted, now 28
  percent", "Context over limit by 12 thousand tokens, sending is blocked".
- Live region: **polite** on crossing warn / critical and on compaction done; **assertive** only for
  `Overflow` when it blocks sending. Never announce routine token changes; at most one announcement per 5 s.
- Popover: category list is a `list`; each row's swatch is decorative; token values are text.
- Contrast: warn/critical fills ≥ 3:1 against the track; labels ≥ 4.5:1; daltonized themes swap amber/red
  for patterns (hatched critical) in addition to hue.
- Focus order: … permission → **context** → send; compact chip receives focus only when the user tabs to it.

## 9. Props sketch
```rust
pub struct ContextMeter {
    pub agent: AgentStateRef,                // Streaming/Thinking/ToolRunning -> Live + Estimating; Done -> reconcile
    pub usage: ContextUsage,                 // window: Option<u32>, categories: Vec<CategoryUsage>, exact: bool, at: Instant
    pub variant: ContextVariant,
    pub thresholds: Thresholds,              // warn_at 0.70, critical_at 0.85, compact_prompt_at 0.90, hysteresis 0.03 (proposed)
    pub compaction: CompactionState,         // Idle | Suggested | Running | Done { delta: i64, at } | Failed { reason }
    pub auto_compact_at: Option<f32>,        // shown as "Auto-compacts at 95 %" when Some
    pub sample_interval: Duration,           // default 250 ms
    pub grid: GridSpec,                      // Auto (10x10 ≤ 200K, 20x10 above) | Fixed(cols, rows)
    pub show_label: LabelMode,               // None | Percent | Tokens | Remaining
    pub labels: ContextLabels,
    pub motion: MotionLevel,
    pub on_compact: Callback<()>,
    pub on_handoff: Option<Callback<()>>,    // "New thread with summary"
    pub on_category_action: Callback<CategoryAction>, // remove file, disable MCP server, open memory file
}

pub enum ContextCategory { System, Tools, Memory, Skills, Files, History, Reasoning, Reserved, Other }

fn level(u: &ContextUsage, t: &Thresholds, prev: Level) -> Level {
    let Some(w) = u.window else { return Level::Unknown };
    let r = u.used() as f32 / w as f32;
    if r > 1.0 { return Level::Overflow }
    // hysteresis: only drop a level when r < threshold - t.hysteresis
    Level::from_ratio_with_hysteresis(r, t, prev)
}
```
`Compacting` is driven by the adapter (compaction events), not by the meter. `Stale` is derived by
`oxyn-core` from the last usage timestamp, like `Stalled` in architecture.md.

## 10. Related components and references
**Related:** compaction marker in the transcript; handoff / "new thread from summary" card; model picker
([composer-model-picker.md](composer-model-picker.md)) incompatibility for small windows; effort control
(reserve grows with effort); MCP servers / tool access panel (per-server token cost); attachment tiles
(remove from context); plan-usage / rate-limit meter (different thing: account quota, not window); send
control (`Overflow` → disabled-with-reason).

**References** (study only unless MIT/Apache):
- Claude Code `/context` grid, categories, autocompact buffer, `/autocompact` (research 05) and Claude
  desktop context ring r = 10, clay above 80 %, breathing (research 08 §1.4, community RE) — proprietary,
  reference only.
- Cursor context ring → breakdown tray (research 05) — closed, study only.
- Codex CLI footer "N% context left" (research 03) — permissive licence, portable with attribution.
- OpenCode sidebar context stats and AnimatedNumber odometer (research 03, 08 §1.5) — MIT, port with
  attribution.
- Claude Code adaptive token count-up (research 08 §1.3) — leaked proprietary source; behaviour only.
- Third-party thresholds 60/85 and 50/75 (research 05) — informed our 70/85/90 proposal.
- Arc UI context window 200-cell grid; Arc MCP servers card "18 tools · 6.4k tokens of context" with
  rolling tabular digits and per-server segments (research 01, 12) — paid / unclear, study only.
- affectUI Context Window (cells), Context Map (treemap with compaction) (research 01, 11) — Pro, study only.
- Vercel AI Elements Context (used/total, categories, cost; research 05, 01) — Apache-2.0, portable.
- [R095] BoardUI Agent Limits block "context window usage with a token breakdown and plan limits" — free
  tier MIT, Pro study only.
- [R081] TUFFUI Agent Composer context tray with token counts — MIT (research 01), portable with attribution.
- [R107] signoff-ui RunMeter (tokens, cost, time to first token, cache hit) — MIT, portable.
- [R025] radial tick gauge — study only. [R018] rolling live counters — study only.
