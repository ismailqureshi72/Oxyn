**Status:** draft v0.1 (2026-11-11).

# Spec: run meter (`run-meter`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
overview in [tools.md](tools.md)). Same depth as [composer-send-control.md](composer-send-control.md).
Target **4 free + 12 Pro = 16 variants**. Number formats, the count-up rule and the `RunClock` come from
[elapsed-timer.md](elapsed-timer.md) (§2, §4); this component is the **structured readout** of those
counters. `elapsed-timer/meter` stays the one-line text form; the status strip's Pro `run-meter` embeds
`run-meter/chip`.

## 1. Purpose
"What did that cost me, how fast was it, and how close am I to the context limit?" Developers, power users
and ops dashboards want the run's **economics and performance** in one place: tokens in / out / cached,
cost (only when the app supplies prices), time to first token, tokens per second, duration, and context
window used — for **this turn** and for **the whole session**, as a small chip that expands into a panel.

What we saw: signoff-ui ships a **RunMeter** with tokens, cost, time to first token and cache hit rate
[R107] (MIT); Claude CLI's custom status line shows model, dir, context and cost; OpenCode's sidebar shows
context % and cost; Codex CLI's footer shows "N% context left" (all research note 03); Warp rests as
`∷ 5s • 0.5 credits` and Claude Code counts tokens up after 30 s (research note 08); a cost total ticks
$0.05 → $0.46 with IN/OUT tokens per step [R030]; AI Elements Context shows used / total, token categories
and a cost estimate (research note 05); affectUI's model-picker tray "rolls the first-token time and price",
and it lists Usage Meter and Context Window (research note 11, Pro, study only); a token-usage chart appears
in Pitus's composer thread [R021]; per-account usage bars in agent-todos [R023].

Anatomy:
```
 chip (status strip / footer):   ⟠ 18.2k tok · $0.07 · 1:12            (segments optional, priority-ordered)

 panel (popover from the chip, or docked):
┌ Run meter ─────────────────────────────── This turn | Session ┐
│ Tokens      in 14.1k   out 4.1k   cached 9.8k (69 %)          │  stacked bar: in · cached · out
│             ▕██████▓▓▓▓▓▓▓▓▓▓░░░░▏                            │
│ Cost        $0.07  (est.)            in $0.04 · out $0.03     │  hidden when no PriceTable
│ Speed       first token 1.2 s · 42 tok/s · 1:12 total         │  TTFT, rate (EMA), duration
│ Context     118k / 200k  ▕███████████▌      ▏ 59 %            │  shares the context meter's thresholds
│ By agent    ◆ Orchestrator 9.4k · ◆ Researcher 8.8k          │  only with a RunGraph
└───────────────────────────────────────────────────────────────┘
```

Rules (proposed):
1. **Oxyn never ships prices.** Cost appears only with an app-supplied `PriceTable` (per model: input,
   output, cached input, optional per-request); without it the cost row and segments do not exist (no
   "$0.00"). Estimated values carry `est.` / `~`.
2. **Measured, not guessed.** Tokens from usage events; estimates (chars ÷ 4) only until the provider's
   count arrives, then corrected with a visible downward/upward roll (elapsed-timer.md §5).
3. **Scope is explicit**: every number is labelled with its scope (This turn / Session / Agent). The chip
   shows the scope the app chooses (default: current turn while running, session when idle).
4. **Context used ≠ tokens spent.** Context is the current window fill (from
   [composer-context-meter.md](composer-context-meter.md), same thresholds); tokens are cumulative spend.
   They are shown on separate rows and never summed.
5. **Quiet by default**: numbers count up smoothly but the meter is not a live region; it never animates
   while idle.

## 2. State machine
```
                    run starts                      first token                     run ends
  Idle{session} ──────────────────▶ Waiting{ttft} ─────────────────▶ Streaming ───────────────────▶ Settling ──▶ Settled{turn}
  (session totals,      (TTFT timer counting,             │  ▲ tool runs: rate hidden, tokens       (final usage arrives,      │
   or hidden if none)    tokens from input usage)         │  │ still count)                         estimates corrected,       │ next run
                                                          ▼  │                                      ≤ 300 ms)                  ▼
                                                     ToolPhase ──┘                                                       Waiting (turn resets,
                                                                                                                          session accumulates)
  Overlays:  AwaitingYou (rate hidden, duration paused per RunClock) · Stalled (rate decays to "—" after 3 s with no tokens)
             Estimated (some values est.) · NoUsage (provider sends none → tokens hidden, only time metrics)
             BudgetWarn{80 %} / BudgetDanger{95 %} / BudgetReached (when a token or cost budget is set)
             ContextWarn / ContextCritical / Compacting / Compacted (from the context meter thresholds)
             Error (run failed: values freeze, "partial" tag) · Cancelled (values freeze, "interrupted" tag)
  View:      Chip ⇄ Panel{scope} (open / close) ; scope toggle This turn ⇄ Session ⇄ By agent
```

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle{session}` | chip: session totals "Session 182k tok · $0.94" (or hidden if the app sets `show_idle = false`) | open panel | — | Enter / Space open |
| `Waiting{ttft}` | chip: "first token…" with TTFT counting in decimals (`0.8s`) after 1 s; input tokens if reported | open panel | — | as above |
| `Streaming` | tokens count up; `42 tok/s` after ≥ 1.5 s and ≥ 20 tokens (elapsed-timer rule); duration ticking | open panel | — | — |
| `ToolPhase` | rate hidden ("—"); tokens and duration continue | open panel | — | — |
| `AwaitingYou` overlay | duration ⏸, rate hidden, "waiting for you" in tooltip | open panel | — | — |
| `Stalled` overlay | rate fades to "—" after 3 s with no tokens; no colour change (status strip owns stall tone) | — | — | — |
| `Settling` | estimates roll to real values (≤ 300 ms); `est.` tags drop | — | — | — |
| `Settled{turn}` | chip: "18.2k tok · $0.07 · 1:12" frozen; TTFT and mean rate in the panel | open panel | copy summary | Enter open, ⌘/Ctrl+C copy (panel) |
| `Estimated` | `~18k`, `$0.07 est.` | — | — | — |
| `NoUsage` | tokens and cost hidden; panel shows TTFT / duration only with note "Provider didn't report usage" | — | — | — |
| `BudgetWarn` / `BudgetDanger` | warning / danger tone + ⚠ on the budgeted segment "48k / 60k tok"; 80 % / 95 % thresholds | open budget (app) | — | — |
| `BudgetReached` | "Budget reached" segment; run handling belongs to the status strip | extend (app) | — | — |
| Context overlays | context row tinted per [composer-context-meter.md](composer-context-meter.md) thresholds; "Compacting…" / "Compacted −62k" | compact now (app) | — | — |
| `Error` / `Cancelled` | values freeze with "partial" / "interrupted" tag | — | — | — |
| `Panel{scope}` | rows per anatomy; scope segmented control | switch scope | copy, export (app) | ←/→ scope, Tab rows, Esc close |

## 4. Micro-interactions
- **Count-up.** Tokens and cost close 25 % of the gap per 50 ms tick, min +1 unit (elapsed-timer.md §5);
  digits roll only in `odometer`-style Pro variants; free variants swap digits in tabular cells.
- **Corrections.** Estimate → real roll once (280 ms) in either direction; the `est.` tag fades 120 ms.
- **Rate.** `tok/s` is an EMA (τ = 1 s) over a 2 s window, displayed at 2 Hz (elapsed-timer.md §4); on
  `ToolPhase` or stall it crossfades to "—" (160 ms) instead of decaying through misleading low numbers.
- **TTFT.** Counts in decimals while waiting; on first token it freezes and does a 1 px lift + success tint
  for 400 ms (once), then settles to secondary text.
- **Stacked token bar.** Segments (in, cached, out) grow with `spring.snap`, ≤ 250 ms behind the numbers;
  cached share is drawn as a hatched sub-segment of input.
- **Scope toggle.** Numbers crossfade with a 4 px vertical slide (180 ms); bars morph widths (260 ms
  `spring.snap`); the session view adds a sparkline of per-turn tokens.
- **Chip ↔ panel.** Panel scales from the chip (0.96 → 1, origin = chip, 180 ms); rows stagger 30 ms.
- **Budget thresholds.** Crossing 80 % / 95 % tints the segment over 300 ms with a single 2-cycle 3 px nudge
  at 95 % (same as the status strip budget chip).
- **Overflow.** When the chip doesn't fit, segments drop right-to-left by priority: tokens > cost > duration >
  rate > TTFT (elapsed-timer.md §8 order adapted), dropped ones move to the tooltip.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Count-up ticks | 50 ms, 25 % of gap, min +1 | — | — | — | 20 Hz while moving |
| Estimate correction roll | 280 ms | — | `ease.out` | — | 30 Hz |
| `est.` tag fade | 120 ms | — | `ease.out` | — | 30 Hz |
| Rate display update | every 500 ms | after 1.5 s and ≥ 20 tokens | swap | — | 2 Hz |
| Rate ↔ "—" crossfade | 160 ms | 3 s without tokens | `ease.out` | — | 30 Hz |
| TTFT decimals | 100 ms resolution | shown after 1 s | — | — | 10 Hz while waiting |
| TTFT freeze lift + tint | 400 ms × 1 | — | `ease.out` | — | 60 Hz |
| Token bar growth | ≤ 250 ms lag | — | `spring.snap` | — | 30 Hz while moving |
| Scope toggle | 180 ms text; 260 ms bars | — | `ease.out`; `spring.snap` | 30 ms rows | 60 Hz |
| Panel open / close | 180 ms / 140 ms | — | `ease.out` / `ease.in` | 30 ms, cap 6 | 60 Hz |
| Budget tint / nudge | 300 ms / 240 ms 2 × 3 px | at 80 % / 95 % | `ease.out` / linear | — | 60 Hz |
| Settle on done | ≤ 300 ms | — | `ease.out` | — | 30 Hz |
| Idle | no repaint | — | — | — | 0 Hz |
| **Reduced motion** | numbers jump to their value at most every 500 ms; no rolls, lifts or nudges (tone + ⚠ text only); bars change width without easing at 2 Hz; scope toggle and panel open use 80 ms crossfades | — | linear | none | ≤ 2 Hz |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `run-meter/chip` | Compact chip `18.2k tok · $0.07 · 1:12` with priority-ordered segments; opens the panel | chat, desktop, ide |
| `run-meter/panel` | Full readout: tokens (in / out / cached bar), cost (if prices), TTFT, tok/s, duration, context, by agent; scope toggle | desktop, ide |
| `run-meter/footer-line` | Status-line text for an IDE/CLI footer: `model · ctx 59% · 18.2k tok · $0.07` (Claude CLI status line idea, note 03) | ide, terminal |
| `run-meter/receipt` | Static per-turn receipt under the answer after completion ("1:12 · 18.2k tokens · first token 1.2s") | chat, mobile |

### Pro (12)
| Name | One-line idea |
|---|---|
| `run-meter/gauge` | Semicircle gauge for tok/s with a needle that eases; TTFT as a tick on the arc |
| `run-meter/odometer` | Rolling per-digit wheels for tokens and cost (OpenCode AnimatedNumber idea, MIT) |
| `run-meter/sparkline-session` | Session view with per-turn token sparkline and cost bars; hover a turn to see its receipt |
| `run-meter/cache-ring` | Ring showing cached-input share; centre shows the saving only when prices exist |
| `run-meter/radial-ticks` | Radial tick gauge whose ticks fill one by one as tokens accrue [R025] |
| `run-meter/per-agent-stack` | Stacked bar of tokens by agent with identity colours (needs RunGraph) |
| `run-meter/budget-burn` | Burn-down of a token / cost budget with projected exhaustion only when the app supplies a rate target |
| `run-meter/model-compare` | Side-by-side TTFT / rate / cost for the models used in a session |
| `run-meter/credits` | Shows app-defined credits instead of currency (`0.5 credits`), as in Warp's rest line idea (note 08) |
| `run-meter/hud` | Floating translucent HUD for demos and screen recordings |
| `run-meter/latency-strip` | Live strip of inter-token gaps; spikes visible (pairs with voice latency meter, family K) |
| `run-meter/ascii` | Terminal box with `▕███▓▓░░▏` token bar and aligned numeric columns |

## 7. Edge cases
- **No prices.** Cost absent everywhere; panel shows "Add prices to see cost" only if the app opts into the
  hint (default off).
- **Mixed currencies / credits.** One `PriceTable` currency per meter; credits are a unit, not a currency
  (formatted `0.5 credits`).
- **Tiny costs.** `$0.004` (3 decimals under $0.01), `< $0.001` below that; never round a non-zero cost to
  `$0.00` (elapsed-timer.md §4).
- **Huge numbers.** `1.24M`, `12.4M`; session totals over 1B as `1.2B`.
- **Cached tokens unreported.** Cached segment hidden; cache-ring disabled with reason.
- **Reasoning tokens** reported separately by a provider: shown as a sub-segment of output labelled
  "reasoning" (when reported; never estimated).
- **Parallel sub-agents.** Session and turn totals sum all agents; "agent time" can exceed wall time and is
  labelled so (elapsed-timer.md §8); By agent row lists each.
- **Very fast turns** (< 1 s): TTFT and duration shown in decimals (`0.6s`); rate hidden (needs ≥ 1.5 s).
- **Very slow turns**: rate hidden during long tool phases; duration compacts after 1 h.
- **Retries / regenerations.** Each attempt is its own turn in the session sparkline; "This turn" shows the
  latest attempt with "+1 retry" tag.
- **Compaction.** Context row shows "Compacted −62k"; tokens spent are unchanged (rule 4).
- **RTL.** Layout mirrors; numbers, currency and units in LTR isolates; bars fill in reading direction
  (right → left) since they are proportions, not time (proposed).
- **Tiny sizes (16–20 px).** Chip shows one segment (tokens) with a 12 px glyph; the rest in tooltip.
- **Dense IDE panels.** `footer-line` only; panel opens as a popover.
- **Touch.** Chip 44 px hit area; panel as a bottom sheet on mobile.
- **Slow networks.** Values stay at last known with "updated 12s ago" in the panel; no fake progress.

## 8. Accessibility
- Chip is a button: "Run meter: 18 thousand 200 tokens, 7 cents, 1 minute 12 seconds. Open details". Panel is
  a `dialog` (popover) or `region` (docked) with a definition-list structure (term: value).
- **Not live.** Values are not announced while counting. Opt-in announcements: budget 80 % / 95 % / reached
  (polite / assertive), context critical (polite). Completion summary is announced by the status strip, not
  here, to avoid duplicates.
- Spelled units in accessible text: "first token after 1.2 seconds", "42 tokens per second", "estimated".
- **Contrast:** numbers ≥ 4.5:1; bar segments ≥ 3:1 and distinguished by pattern (hatched cached) and legend
  text, not tone alone; budget and context warnings add ⚠ + text.
- **Focus order:** chip → (open) scope control → rows (copy buttons) → close; Esc returns focus to the chip.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct RunMeter {
    pub clock: RunClockRef,                 // duration, TTFT (elapsed-timer.md §2)
    pub usage: UsageSource,                 // adapter usage events per agent / turn / session
    pub graph: Option<RunGraphRef>,         // "By agent" (subagent-tree.md §9)
    pub agent_state: AgentState,            // drives Waiting / Streaming / ToolPhase / overlays
    pub price: Option<PriceTable>,          // app-supplied; None → no cost anywhere
    pub context: Option<ContextReading>,    // { used, window, state } from composer-context-meter
    pub budget: Option<Budget>,             // Tokens{limit} | Cost{limit} (cost needs price)
    pub scope: MeterScope,                  // Turn | Session | Agent(AgentId)
    pub variant: MeterVariant,              // Chip | Panel | FooterLine | Receipt | Pro(..)
    pub segments: Vec<MeterSegment>,        // priority order: Tokens, Cost, Duration, Rate, Ttft, Context
    pub show_idle: bool,                    // true: session totals when idle
    pub rate: RateCfg,                      // EMA τ 1 s, window 2 s, min 1.5 s & 20 tokens, 2 Hz
    pub currency: CurrencyFormat,           // locale + currency or Credits{label}
    pub labels: MeterLabels,
    pub motion: MotionLevel,
}

pub struct Usage { pub input: u64, pub output: u64, pub cached_input: Option<u64>,
                   pub reasoning: Option<u64>, pub estimated: bool }
```
Mapping: `Submitted` → `Waiting` (TTFT starts); `Streaming` → `Streaming`; `Thinking` → `Streaming` if
reasoning tokens stream, else `Waiting`; `ToolRunning` → `ToolPhase`; `AwaitingApproval` → `AwaitingYou`
overlay; `Stalled` → `Stalled` overlay; `Done` → `Settling` → `Settled`; `Error` / `Cancelled` → freeze
with tag; `Queued` / `Steering` → unchanged.

## 10. Related components and references
Related: [elapsed-timer.md](elapsed-timer.md) (formats, count-up, `meter` text variant) ·
[composer-status-strip.md](composer-status-strip.md) (Pro `run-meter`, budget chip) ·
[composer-context-meter.md](composer-context-meter.md) (context thresholds) ·
[composer-model-picker.md](composer-model-picker.md) (model metadata) · [trace-view.md](trace-view.md)
(per-span tokens and cost) · [subagent-tree.md](subagent-tree.md) (per-agent usage) ·
[fanout-grid.md](fanout-grid.md) (per-lane scorecard) · [background-jobs.md](background-jobs.md) ·
cost/budget meters for multi-agent ops dashboards (family M).

| Ref | What we take | Licence note |
|---|---|---|
| signoff-ui RunMeter [R107] (note 16) | tokens, cost, time to first token, cache hit rate in one meter | MIT per licensing.md — portable with attribution (verify file headers before porting) |
| Claude CLI status line, OpenCode sidebar, Codex footer (note 03) | model / context / cost in a footer line | Claude proprietary (reference only); OpenCode MIT, Codex MIT per licensing.md (portable with attribution) |
| Claude Code count-up, Warp rest line (note 08) | tokens after 30 s; time + credits at rest | Claude proprietary — reference only; Warp AGPL — study only |
| OpenCode AnimatedNumber (note 08) | odometer for Pro `odometer` | MIT — portable with attribution |
| Cost waterfall [R030] | rolling cost total; IN/OUT tokens per step | Study only |
| AI Elements Context (note 05) | used / total, categories, cost estimate | Licence not checked here — study only until checked |
| affectUI model picker tray, Usage Meter, Context Window (note 11) | TTFT + price roll idea; names | Paid — study only |
| Radial tick gauge [R025]; usage bars [R023]; token chart [R021] | Pro visual ideas | Study only |
