**Status:** v1 (plan #17, 2026-11-12): 4 free variants: `oxyn-core::text::timer_view`, Dioxus `ElapsedTimer`, GPUI `TimerText`.

# Spec: elapsed timer and run counters (`elapsed-timer`)

Part of family C ([status-text.md](status-text.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **4 free + 14 Pro = 18 variants**. Covers
the elapsed time of a run or step, token count, tokens per second, cost, and the completion summary
("Worked for 1m 12s").

## 1. Purpose
"How long has it been, and what is it costing?" is the second question after "is it alive?". A
ticking number reassures during long runs, lets the user decide when to stop, and becomes the receipt
when the run ends. The studied apps: Codex shows `Working (12s • esc to interrupt)` with the format
`0s … 59s`, `1m 00s`, `1h 00m 00s` and **pauses the timer while an approval is open**; Claude Code
counts tokens up smoothly after 30 s and ends with "{verb} for 1m 23s"; Warp rests as
`∷ 5s • 0.5 credits`; Roo ticks seconds on its reasoning block; OpenCode ships a 560 ms odometer
(AnimatedNumber); Zed spaces numbers with a thin space and a figure dash (all note 08). Elsewhere: a
cost total ticking $0.05 → $0.46 [R030], a test timer 2.9 → 10.6 s with counters [R037], a time
budget "2h 06m of 24h" [R007], "Thinking 13s" → "Thought for 14s" [R021], and counters that roll on a
widget bento [R018]. The user wants these numbers to be **true, stable (no jitter), and quiet**.

## 2. Time source (`RunClock`, in `oxyn-core`)
- `elapsed = now − run_start − Σ paused_intervals`, computed from a monotonic clock locally; a run
  resumed after an app restart uses the adapter's server-side start timestamp, so remounting or
  reopening never resets the timer (composer-status-strip.md edge case).
- **Paused** while the run waits on the user: `AwaitingApproval`, a question card, or an explicit
  user pause (Codex pauses on approval/elicitation, note 08). Waiting time is accumulated separately
  as `waited`.
- **Not paused** while `Stalled`, offline or rate-limited: the user's wall time is still being spent
  (proposed; the banner explains why).
- Step timers (tool rows, plan steps) are child clocks of the same `RunClock`.

## 3. State machine
```
              run starts (Submitted)            first second elapsed
  Hidden ─────────────────────────▶ Armed ──────────────────────────▶ Running ──AwaitingApproval──▶ Paused
    ▲                         (nothing shown < 1 s)                   │  ▲  ▲                          │
    │                                                                 │  │  └────── answered ──────────┘
    │                                                                 │  └── event ── Stalled (keeps counting; tone via status-text)
    │                                                                 │
    │◀── host hides after done_hold ── Summary("Worked for 1m 12s") ◀── Done ── Freezing (digits stop, tense roll)
    │◀── stays in transcript ───────── Frozen("Interrupted after 42s") ◀── Cancelled
    │◀── stays ─────────────────────── Frozen("Stopped after 42s") ◀───── Error
    │
  Done before 1 s: Armed → Hidden (no "Worked for 0s").
  Counters (tokens, tok/s, cost) run beside the timer: Hidden → Counting → Settled; tok/s only while Streaming.
```

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing | — | — | — |
| `Armed` | nothing yet (first second) | — | — | — |
| `Running` | `42s` / `0:42`, tabular; counters if enabled | host's (click → run details) | tooltip: start time, exact seconds, waited time | focusable only in actionable hosts |
| `Paused` | digits frozen + ⏸ glyph; optional "waiting 0:12" chip | host → focus approval card | — | host's |
| `Stalled` | keeps counting; tone handled by status-text | Retry (status strip) | — | — |
| `Freezing` | final value stops, then rolls into summary | — | — | — |
| `Summary` | "Worked for 1m 12s" (+ "· 18.2k tokens · $0.07") | expand run summary | copy summary | Enter when focused |
| `Frozen` | "Interrupted after 42s" / "Stopped after 42s" | retry (failed) | details | Enter |

## 4. Formats
| Format | 0–59 s | 1–59 min | 1–23 h | ≥ 24 h | Use |
|---|---|---|---|---|---|
| `Units` (default) | `42s` | `1m 05s` | `1h 02m 33s` (compact: `1h 02m`) | `1d 03h` | status lines, Codex-compatible |
| `Clock` | `0:42` | `12:05` | `1:02:33` | `27:02:33` | timer-first UIs, media agents |
| `Decimal` (steps < 10 s) | `0.8s`, `2.9s` (one decimal under 10 s, integer above) | — | — | — | tool rows, test runners [R037] |
| `Summary` | `Worked for 42s` | `Worked for 1m 05s` | `Worked for 1h 02m` | `Worked for 1d 3h` | completion (verb from the pack) |
| Under 1 s | hidden | — | — | — | never "0s" |

Rules: units are localised with the locale's **narrow** unit forms (e.g. ja `1時間2分`), falling back to
`h/m/s`; numbers use the locale's numbering system unless the app forces Latin digits; minutes and
seconds after the leading unit are zero-padded (`1m 05s`); the **leading** unit is never padded.

### Counters
| Counter | Format (proposed) | Source | Notes |
|---|---|---|---|
| Tokens | `842` · `18.2k` · `182k` · `1.24M`; `~` prefix when estimated | adapter usage events; estimate (chars ÷ 4) only when the provider reports usage at the end | estimate replaced by the real count when it arrives |
| Tokens per second | `7.4 tok/s` (< 10, one decimal) · `42 tok/s` | output tokens, EMA over a 2 s window (τ = 1 s) | shown only while `Streaming`, after ≥ 1.5 s and ≥ 20 tokens; display updates at 2 Hz |
| Cost | `$0.004` (< $0.01, 3 decimals) · `$0.46` · `$12.30`; locale currency format; `est.` when based on estimates | tokens × the app's `PriceTable` per model (input / output / cached) | **Oxyn never ships prices**; no table → no cost counter |
| Time to first token | `first token 1.2s` | `Submitted` → first `Streaming` event | detailed variants only |
| Waited | `waited 1m 04s for you` | sum of paused intervals | shown in summary when ≥ 5 s |

## 5. Micro-interactions
- **Appear.** At 1 s the timer fades in (120 ms) in its reserved slot — the slot exists from `Armed`
  so nothing shifts (`Width::ReserveMagnitude`, status-text.md §2.2).
- **Tick.** Once per second, aligned to the run's whole seconds. Default variants swap digits; rolling
  variants roll only the digits that change, right to left, 30 ms stagger, so `0:59 → 1:00` rolls
  three wheels in a short cascade.
- **Magnitude change** (`59s → 1m 00s`, `9:59 → 10:00`, `59:59 → 1:00:00`): the slot widens with
  `spring.snap` 200 ms *before* the new digits roll in.
- **Count-up counters.** Tokens and cost move toward their target, closing 25 % of the gap per 50 ms
  tick (min +1 unit) — the status strip's rule; a correction downward (estimate → real) rolls *down*
  once (280 ms), never silently.
- **Interrupted roll.** If a new value arrives mid-roll, the wheel continues from its current position
  to the new target (no queue of rolls).
- **Pause.** Digits stop; ⏸ fades in 120 ms beside them; digits shift to the secondary text tone
  (still ≥ 4.5:1). Resume: ⏸ fades out, counting continues from the paused value (no catch-up jump).
- **Done.** Final value freezes at the `Done` event time; after the verb's past-tense roll
  (`dur.tense.roll` 240 ms), digits re-flow into the summary phrase ("1:12" → "Worked for 1m 12s");
  counters settle to their final values in ≤ 300 ms.
- **Hover.** Tooltip: "Started 14:02 · 1 min 12 s working · waited 1 min 04 s" (local time).

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Timer appear | 120 ms fade | at 1.0 s | `ease.out` | — | 30 Hz |
| Tick (swap) | instant | — | — | — | 1 Hz |
| Digit roll (rolling variants) | 280 ms (`dur.digit.roll`) | — | `ease.out` | 30 ms right → left | 30 Hz during roll, else 1 Hz |
| Odometer mask | 18 % top/bottom fade | — | — | — | — |
| Slot widen at magnitude change | 200 ms | before the roll | `spring.snap` | — | 60 Hz |
| Counter count-up | 50 ms ticks, 25 % of gap, min +1 | — | — | — | 20 Hz while moving |
| tok/s display update | every 500 ms | after 1.5 s streaming | swap | — | 2 Hz |
| Pause glyph in / out | 120 ms | — | `ease.out` / `ease.in` | — | 30 Hz |
| Tone to secondary on pause | 200 ms | — | `ease.out` | — | 30 Hz |
| Done: freeze → summary | 240 ms | after the verb roll | `ease.out` | — | 60 Hz |
| Counter settle on done | ≤ 300 ms | — | `ease.out` | — | 30 Hz |
| Budget countdown tint (80 % / 95 %) | 300 ms | — | `ease.out` | — | 30 Hz |
| Decimal step timer | 100 ms resolution | — | — | — | 10 Hz while the step row is visible |
| **Reduced motion** | digits swap without rolling; counters jump to the value at most every 500 ms; pause glyph appears without fade; summary replaces the timer with an 80 ms crossfade | — | linear | none | 1 Hz |

## 7. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `elapsed-timer/units` | `42s · 1m 05s · 1h 02m 33s`, tabular, swap on tick; Codex-compatible format | all |
| `elapsed-timer/clock` | `0:42 · 12:05 · 1:02:33` in a reserved slot | desktop, ide, media agents |
| `elapsed-timer/summary` | Only the completion phrase "Worked for 1m 12s" (+ waited), no live ticking — for calm UIs | chat, mobile |
| `elapsed-timer/meter` | Text-only run meter: `1:12 · 18.2k tokens · 42 tok/s · $0.07`, each segment optional | desktop, ide, terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `elapsed-timer/odometer` | Every digit on its own rolling wheel with masked edges |
| `elapsed-timer/split-flap` | Departure-board flaps per digit |
| `elapsed-timer/flip-card` | Minutes and seconds on two flip cards (desk-clock look) |
| `elapsed-timer/seven-segment` | Segment display; segments cross-fade on change |
| `elapsed-timer/dot-matrix` | 5×7 dot glyphs; dots light in sequence on change |
| `elapsed-timer/nixie` | Warm glowing digits with a soft bloom (dark themes; light themes fall back to `odometer`) |
| `elapsed-timer/ring-seconds` | A thin ring around the loader fills once per minute; digits beside |
| `elapsed-timer/cost-ticker` | Rolling cost total with a per-step breakdown on hover [R030] |
| `elapsed-timer/rate-sparkline` | tok/s with a 30-sample sparkline; flattens visibly on stall |
| `elapsed-timer/budget-countdown` | Counts *down* a time / token / cost budget; warning at 80 %, danger at 95 % [R007] |
| `elapsed-timer/laps` | Per-step split times in a column, newest on top, decimals under 10 s [R037] |
| `elapsed-timer/breakdown` | Final summary with a stacked bar: thinking · tools · streaming · waiting |
| `elapsed-timer/eta` | "~2m left" **only** when the app supplies progress; hidden when the estimate is unstable |
| `elapsed-timer/waited-chip` | Separate amber chip counting time spent waiting on you while paused |

## 8. Edge cases
- **0 s / under 1 s.** Nothing shown, no summary; counters may still show in `meter` if non-zero.
- **Exactly at boundaries.** 59.6 s shows `59s` (floor, never round up to a minute early).
- **Very long runs (hours, days).** Compact hosts drop seconds from 1 h; `Clock` grows to `27:02:33`;
  `Units` switches to `1d 03h`.
- **RTL.** Digit groups and durations sit in an LTR bidi isolate; the unit labels follow the locale;
  the slot mirrors with the line.
- **CJK.** Narrow unit forms (`1時間2分`); full-width digits are never used (tabular Latin or the
  locale's numbering system).
- **Locale digits.** Arabic-Indic or Devanagari digits keep tabular cells measured from that digit set.
- **Fonts without `tnum`.** Fixed digit cells (status-text.md §2.3).
- **Screen width.** Priority when space runs out: elapsed > tokens > cost > tok/s > TTFT; dropped
  segments move into the tooltip.
- **Clock skew / sleep.** Local monotonic clock for live counting; resumed runs use server start time;
  negative skew clamps to 0; after device sleep the timer jumps to the true value without rolling.
- **Background tab.** Repaint at 0.25 Hz; value always computed from timestamps, so it is correct on
  focus.
- **Estimated vs real tokens.** `~18k` → `17.6k` correction rolls once; cost follows with `est.` removed.
- **No usage events at all.** Tokens and cost hidden (never show 0); tok/s hidden.
- **Parallel sub-agents.** Parent timer is wall time of the parent run; child totals summed in the
  `breakdown` / `meter` tooltip, labelled "agent time" so it can exceed wall time without confusion.
- **Paused across a reload.** Pause intervals live in `RunClock`, so a reload during an approval still
  shows the paused value.

## 9. Accessibility
- The timer uses `role="timer"` (implicitly not live). It is **never announced per second**.
- Accessible label, computed when read: "Elapsed 1 minute 12 seconds" (spelled units, not "1m 12s");
  paused: "Elapsed 1 minute 12 seconds, paused while waiting for you".
- Opt-in milestone announcements (polite) at 30 s, 1 min, then every 2 min (family rule,
  status-text.md §11). Done summary announced once ("Done, worked for 1 minute 12 seconds").
- Counters (tokens, cost, tok/s) are not live; their accessible text is the formatted value with
  units spelled ("18 thousand 200 tokens, estimated").
- **Contrast:** digits ≥ 4.5:1 in every state, including paused (secondary tone token is contrast-checked)
  and budget warning tones (which also add an icon and text, never colour alone).
- **Reduced motion:** §6 last row. Rolling, flipping and segment animations are decorative; the value
  is always readable in its final form on the same frame.
- Focus: not in tab order unless the host is actionable (summary that expands, budget chip).

## 10. Props sketch
```rust
pub struct ElapsedTimer {
    pub clock: RunClockRef,              // from oxyn-core; drives elapsed / paused / waited
    pub drive: TimerDrive,               // FromState(&AgentState): Submitted → Armed,
                                         // Thinking | Streaming | ToolRunning | Queued | Steering → Running,
                                         // AwaitingApproval → Paused, Stalled → Running (+tone),
                                         // Done{elapsed} → Summary, Cancelled → Frozen(Interrupted),
                                         // Error → Frozen(Stopped)
    pub variant: TimerVariant,           // Units | Clock | Summary | Meter | Pro(ProTimer)
    pub format: TimeFormat,              // Units { compact_after: 1 h } | Clock | Decimal { under: 10 s }
    pub min_visible: Duration,           // 1 s
    pub metrics: Metrics,                // tokens, rate, cost, ttft, waited: each Off | On | Tooltip
    pub usage: Option<UsageStream>,      // adapter usage events (input, output, cached)
    pub price: Option<PriceTable>,       // app-supplied; None → no cost
    pub budget: Option<Budget>,          // Time | Tokens | Cost limit (budget-countdown, strip chip)
    pub summary: SummaryCfg,             // verb from the pack, include waited ≥ 5 s, details
    pub width: WidthPolicy,              // ReserveMagnitude
    pub locale: Locale,
    pub motion: MotionLevel,
}

pub struct RunClock {
    pub start: RunStart,                 // Local(Instant) | Server(SystemTime)
    pub paused: Vec<(Instant, Option<Instant>)>,
    pub waited: Duration,
    pub first_token_at: Option<Instant>,
}
```

## 11. Related components and references
Related: [status-text.md](status-text.md) (summary phrasing, tense rules) ·
[verb-cycler.md](verb-cycler.md) (past-tense roll precedes the summary) ·
[composer-status-strip.md](composer-status-strip.md) (budget chip, token count after 30 s, stall) ·
[composer-context-meter.md](composer-context-meter.md) (context window ≠ run tokens) · run meter and
cost waterfall (family F) · test runner and tool rows (family F).

| Ref | What we take | Licence note |
|---|---|---|
| Codex CLI `timer.rs` / status row (note 08) | `0s … 1h 00m 00s` format, pause on approval, 1 Hz repaint | Apache-2.0 per note 08 (MIT in licensing.md) — portable with attribution |
| OpenCode AnimatedNumber (note 08) | per-digit rolling strip, 18 % mask, animated width | MIT — portable with attribution |
| Claude Code (note 08) | tokens after 30 s; smooth count-up idea; "{verb} for 1m 23s" | Proprietary — reference only, no code |
| Warp (note 08) | rest line with time and credits | AGPL — study only |
| Zed DiffStat (note 08) | thin space and figure dash | mixed — study only |
| Roo Code (note 08) | live seconds on the reasoning block | Apache-2.0 (archived) — portable |
| Cost waterfall [R030] | rolling cost total, per-step rows | Study only |
| Test runner [R037] | decimal timer and counters | Study only |
| Cloudy [R007] | time budget "2h 06m of 24h" | Study only |
| Widget bento [R018] | rolling counters | Study only |
| Pitus [R021] | "Thinking 13s" → "Thought for 14s" | Study only |
| Beautiful UI [R106]/[R133] (note 17) | elapsed counter beside a loader ("Churning 0.0s") | MIT per sources.csv — portable with attribution (verify) |
| Arc UI (note 12) | tabular numerals in a reserved width; timer halts while waiting | Paid/unclear — study only |
| affectUI Number Ticker (note 11) | per-digit flip | Paid — study only |

## Plan #28 parity (2026-11-14)

Related new components: [countdown](countdown.md) and [animated-counter](animated-counter.md).
