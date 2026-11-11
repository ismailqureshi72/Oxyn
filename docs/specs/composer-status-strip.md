**Status:** draft v0.1 (2026-11-11).

# Spec: composer status strip (status tab · budget · jobs · PR pill · stall · approval · offline)

Part of the composer family ([composer.md](composer.md)). Same depth
as the worked example [composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro
variants**, driven by `StripState`, which is a pure function of `AgentState` plus app-supplied
side-channel data (budget, jobs, PR).

## 1. Purpose
While the agent runs, the user wants to know **is it alive, what is it doing, how long has it been,
what is it costing, and does it need me** — without reading the transcript. The status strip is a
thin layer attached to the composer's top edge (above the queue stack when both show) that answers
those questions and holds **Stop**.

It is assembled from patterns we saw separately: a status tab (dot + timer) above the composer that
holds Stop, and on stop "the status line drops into the transcript as **Interrupted**" (firm ease-out,
no overshoot) [R079]; a time-budget chip "2h 06m of 24h" with separate pause and stop [R007]; a
background job pill docked with a percentage, "Image to video · Rendering 4%" [R013]; a composer
footer pill for the active PR, "#128 3 conflicts · +482 −37" [R054]; status taxonomy Approval / Input /
Working (spinner + elapsed) / Done [R003]; Codex CLI's `• Working (12s • esc to interrupt)` row whose
timer **pauses while an approval is open**; and Claude Code's stall-to-red ramp (no tokens for > 3 s
and no tool running → colour ramps toward error over 2 s) (both in research note 08).

Anatomy (left → right, one row, 28 px desktop / 32 px mobile / 1 line terminal):
```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ [banner row, only when offline / quota / rate-limited — full width, above everything]   │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ ● Editing auth.rs · 0:42  ■ │ ⚠ Needs approval │      ⏱ 6m of 30m │ ◔ Render 41% │ #128 · 3 conflicts · +482 −37 │
│ └──── status tab ─────────┘   └ approval badge ┘      └ budget ──┘  └ job pill ┘   └──────── PR pill ───────────┘
└─────────────────────────────────────────────────────────────────────────────────────────┘
                              composer (never moves)
```

Rules:
1. The composer never moves. The strip reserves no height when hidden; it **overlays upward**.
2. The status tab is the **one** place a run's liveness shows near the composer; transcript loaders
   may mirror it but share its clock (shared motion clock, architecture.md).
3. Stop lives in the status tab only in variants where the send button stays ↑ for queueing
   (`send-control/status-tab`); otherwise Stop stays on the send control and the tab is read-only.
4. Elapsed time **pauses** while the run waits on the user (approval, question) and says so.
5. `Stalled` is derived by `oxyn-core` from event timestamps (architecture.md), never set by the app.
6. Side chips (budget, job, PR) appear only when the app supplies data; they never fake progress.

## 2. State machine

```
              send                  first event                       no event for stall_soft
  Hidden ─────────────▶ Submitted ─────────────▶ Working{verb, t} ─────────────────────────────▶ Stalled(soft)
    ▲                    (pulse < 500 ms)         │   ▲    │ tool starts / ends: verb swaps          │ stall_hard
    │                                             │   │    ▼                                         ▼
    │                                             │   └─ event arrives ◀──────────────────────── Stalled(hard)
    │                         approval/question   ▼                                          "No response for 45 s"
    │                       AwaitingYou{request} (timer paused) ── answered ──▶ Working
    │
    │◀── fade 4 s ── Done{elapsed} ("Worked for 42s")  ◀── run completes ─────────┤
    │◀── drop ────── Cancelled → "Interrupted" marker into transcript ◀── Stop ───┤
    │◀── dismiss ─── Failed{kind} ("Failed · Retry") ◀── error ───────────────────┘

  Banner layer (independent, can stack over any state):
    None ⇄ Offline{retry_in} ⇄ Reconnecting ⇄ None
    None ⇄ RateLimited{retry_in} ⇄ None        None ⇄ QuotaExceeded{resets_at} (sticky until resolved)

  Chip layers (independent): Budget{used, limit} · Jobs[{label, pct}] · Pr{number, checks, conflicts, diff}
```

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing (side chips may still show if persistent, e.g. PR pill) | — | — | — |
| `Submitted` | dot pulse, verb "Sending", no timer yet (≤ 500 ms, architecture.md) | stop | — | Esc stop |
| `Working{verb}` | ● dot (accent, breathing) + verb ("Thinking", "Editing auth.rs") + elapsed `0:42`; tokens appear after 30 s in `detailed` | stop ■ | click tab → scroll transcript to the live step | Esc stop (Esc×2 confirm variant) |
| `Stalled(soft)` | dot + verb ramp toward warning tone; timer continues | stop | — | Esc stop |
| `Stalled(hard)` | warning tone + "No response for 45 s" | **Retry** (re-sends last turn) | stop, keep waiting | Enter retry, Esc stop |
| `AwaitingYou{request}` | amber ⚠ badge "Needs approval" / "Question"; verb "Waiting for you"; timer paused with ⏸ glyph | **Review** (scrolls to + focuses the card) | approve-all / bypass only if the permission selector allows | proposed `⌘/Ctrl+Shift+A` jump to card |
| `Done{elapsed}` | ✓ "Worked for 42s" (+ "· 12.4k tokens" if shown) for 4 s, then fades | — | click → summary / scroll to answer | — |
| `Cancelled` | tab text drops into the transcript as "Interrupted · after 42s"; strip hides | — | — | — |
| `Failed{kind}` | danger dot, "Failed · Retry" (kind in tooltip: tool error, model error, timeout) | retry | details, dismiss | Enter retry, Esc dismiss |
| Budget chip | "⏱ 6m of 30m" or "48k / 200k tok" or "$0.46 / $2"; ring/bar fill | open budget menu (extend, set) | — | Enter opens |
| Budget ≥ 80 % / ≥ 95 % / exhausted | warning tone / danger tone / "Budget reached · Extend" (run pauses if app sets `pause_on_exhaust`) | extend | stop | Enter extend |
| Job pill | "◔ Rendering 41%" (+ ETA on hover); n > 1 → "3 jobs · 41%" (mean of known %) | open jobs popover | cancel job | Enter opens |
| Job done / failed | ✓ "Render ready · Open" 6 s / ✕ "Render failed · Retry" (sticky) | open / retry | dismiss | Enter |
| PR pill | "#128 · 3 conflicts · +482 −37" with a 3-segment checks meter | open PR card | copy branch | Enter opens |
| PR states | checking (shimmer) · conflicts (amber) · resolving (36/42 progress) · checks passed (green) · merged (purple-neutral, then hides after 10 s) · checks failed (danger) [R054] | open | — | — |
| Banner `Offline` | "Offline · retrying in 4s" countdown | retry now | — | Enter retry |
| Banner `RateLimited` | "Rate limited · resuming in 20s" countdown; run auto-resumes | — | switch model | — |
| Banner `QuotaExceeded` | "Usage limit reached · resets 14:00" (user's local time) | primary app action (Upgrade / Switch model) | dismiss for session | — |

## 3. Micro-interactions
- **Enter.** On `Submitted`, the tab rises from the composer's top edge (y +6 → 0, opacity 0 → 1); the
  dot pulses once (not a spinner — architecture.md's "submitted pulse").
- **Verb swap.** New verb crossfades with a 4 px vertical slide; a calm brightness shimmer sweeps the
  verb (one 1.0 s sweep every 4.0 s after a 600 ms delay — Codex CLI timings, note 08, MIT). Width
  changes animate; the tab never jumps.
- **Elapsed timer.** Tabular figures; format `0s … 59s`, `1m 00s`, `1h 00m 00s` (Codex format, note 08).
  Ticks once per second; the repaint clock drops to 1 Hz when nothing else animates.
- **Token / cost counter** (detailed variant). Appears after 30 s; counts up smoothly toward the target
  instead of jumping (idea from Claude Code's count-up, reference only; our increments: close 25 % of the
  gap per 50 ms tick, min +1 — proposed).
- **Stall ramp.** After `stall_soft` (default **5 s** with no event *and* no tool running — proposed;
  Claude Code uses 3 s, note 08) dot and verb interpolate toward the warning token over 2 s. At
  `stall_hard` (default **45 s**, proposed) the text changes to "No response for 45 s" and Retry
  appears. The first new event snaps colour back over 200 ms. Running tools use `tool_stall`
  (default 120 s) instead, since tools are legitimately slow.
- **Awaiting you.** A different motion from working: the dot stops breathing and does a slow
  hourglass-style tip every 2 s (the "human-blocked looks different from generating" idea from Zed's
  Sand loader, note 08); the timer shows ⏸. Badge slides in from the tab's trailing edge.
- **Stop → Interrupted drop.** On Stop, the tab's text detaches and translates down into the
  transcript where it becomes an "Interrupted · after 42s" marker row, firm ease-out, no overshoot
  [R079]; the rest of the tab fades.
- **Done.** Dot morphs to ✓ (scale 0.6 → 1), verb rolls to past tense ("Worked for 42s", R021's
  "Thinking → Thought for 14s" pattern); holds 4 s, then fades. Optional arrival ring in Pro.
- **Budget chip.** Ring/bar fills continuously; crossing 80 % / 95 % tints with a 300 ms colour ease;
  exhaustion does a single 2-cycle 3 px nudge.
- **Job pill.** Percentage rolls; the ring is determinate only when % is known, otherwise an
  indeterminate 1.2 s arc. Completion: ring closes, pill flashes success tint, label → "ready · Open".
- **PR pill.** Conflict count rolls down as conflicts resolve; checks meter segments fill one by one;
  amber → green tint on success [R054].
- **Banner.** Slides down from above the strip; countdown numbers roll; on reconnect it shows
  "Back online" for 1.5 s then slides away.
- **Overflow.** When chips don't fit, side chips collapse right-to-left into a "+2" chip with a popover.

## 4. Timing (defaults; every variant may override)
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Tab enter (Submitted) | 160 ms | `ease.out` | y +6 → 0 |
| Submitted pulse | 1 × 400 ms | `ease.out` | scale 1 → 1.4, opacity 1 → 0 ring |
| Dot breathing (Working) | 1600 ms loop | sine | opacity 0.6 ↔ 1 |
| Verb swap | 180 ms, 4 px slide | `ease.out` | width tween 180 ms `spring.snap` |
| Verb shimmer | 600 ms delay, 1.0 s sweep every 4.0 s | cosine band | Codex CLI timings (MIT) |
| Timer repaint | 1 Hz (32 ms frames only while shimmer runs) | — | shared clock |
| Token counter appears / steps | after 30 s; 50 ms ticks | — | proposed increments |
| Stall soft threshold → ramp | 5 s idle → 2 s ramp | linear RGB in OKLab | proposed |
| Stall hard threshold | 45 s | — | proposed |
| Tool stall threshold | 120 s | — | proposed |
| Stall recovery snap-back | 200 ms | `ease.out` | |
| Awaiting-you tip | 2000 ms loop, 300 ms rotate 180° | `ease.in-out` | |
| Approval badge in | 180 ms | `spring.snap` | |
| Stop → "Interrupted" drop | 220 ms | `ease.out` firm (`cubic-bezier(0.2,0,0,1)`), no overshoot | R079 |
| Done ✓ morph | 200 ms | `spring.snap` | hold 4 s, fade 300 ms |
| Budget tint change | 300 ms | `ease.out` | |
| Budget exhausted nudge | 240 ms, 2 cycles, 3 px | linear | |
| Job % roll / ring | 300 ms per change | `ease.out` | indeterminate arc 1.2 s loop |
| Job done hold | 6 s | — | then collapses 160 ms |
| PR conflict count roll | 300 ms | `ease.out` | segment fill 120 ms + 60 ms stagger |
| Banner slide in / out | 200 ms / 160 ms | `ease.out` / `ease.in` | "Back online" hold 1.5 s |
| Chip overflow collapse | 160 ms + 30 ms stagger | `spring.snap` | |
| **Reduced motion** | no breathing, no shimmer (static ● visible, plain verb); stall shows as an instant tone change + text; "Interrupted" marker appears in the transcript with a 80 ms fade, no drop; numbers change without rolling; all slides become 0–80 ms crossfades; indeterminate arcs become a static "…" | — | — |

## 5. Variants

### Free (6)
| Name | Idea | Surfaces |
|---|---|---|
| `status-strip/tab` | Baseline: dot + verb + elapsed tab above the composer; holds Stop with `send-control/status-tab` [R079] | chat, desktop |
| `status-strip/detailed` | Tab + token/cost counter + budget chip + job pill + PR pill | desktop, ide |
| `status-strip/inline-row` | One-line `● Working (42s · esc to interrupt)` row, Codex-style (note 08) | ide, terminal |
| `status-strip/budget` | Pause and Stop as separate buttons with a time-budget chip [R007] | desktop |
| `status-strip/footer-pills` | Status lives as pills inside the composer footer (PR pill, job pill) instead of above it [R054][R013] | chat, desktop |
| `status-strip/minimal` | Text-only "Working · 0:42 · Stop" for 28 px IDE rows | ide |

### Pro (14)
`border-beam` (light travels the composer border while working; stalls dim it), `orbit-dot` (comet
circles the dot), `split-flap-verb` (verb changes with split-flap letters), `pixel-grid` (5×5 twinkle
grid as the dot), `notch` (desktop island mirrors the tab when the window is unfocused),
`run-meter` (tokens / cost / time-to-first-token / cache hit, signoff-ui RunMeter idea [R107]),
`waterfall-peek` (hover the tab to see a mini cost waterfall [R030]), `lanes-peek` (multi-agent
"2 running · 1 needs you" [R061]), `mascot` (pill mascot whose eyes are the pause bars [R023]),
`ember` (stall ramp as cooling embers), `ci-ticks` (PR pill expands into tick-row test results
[R037]), `budget-ring` (budget as a ring around Stop), `heartbeat` (ECG line that flattens on stall),
`ascii` (terminal braille spinner + elapsed, box-drawn banner).

## 6. Edge cases
- **Long verbs / file names.** Verb truncates middle ("Editing src/…/auth.rs"), max 40 % of strip
  width; full text in tooltip. Model-written headers longer than 60 chars are clipped.
- **Overflow.** Priority when space runs out: status tab > approval badge > banner > PR pill > job
  pill > budget chip. Lower-priority chips collapse into "+n".
- **RTL.** Order mirrors (tab on the right); numbers and `+482 −37` stay LTR inside bidi isolates;
  the Interrupted drop target is the same transcript row.
- **Tiny sizes (16–20 px).** Dot only (8 px) + timer; verb hidden; Stop becomes a 16 px ■. Chips
  become icons with counts.
- **Dense IDE panels.** Strip merges into the composer's top border as a 20 px row; banner becomes an
  inline icon with a tooltip.
- **Touch.** Tab is a 44 px hit area even when it looks 28 px; long-press shows details; Stop needs a
  hold of 400 ms if `stop_confirm = Hold` (send-control prop).
- **Very fast agent (< 500 ms).** Skip `Working`: Submitted pulse → Done; don't show "Worked for 0s"
  under 1 s (show nothing). Never flash Stalled.
- **Very slow agent / long runs (hours).** Timer switches to `1h 02m` (drop seconds after 1 h in
  compact variants); budget chip becomes the main signal; stall thresholds unchanged.
- **Slow networks.** Stall is measured on *events received*; when the transport itself reports
  reconnecting, show the Offline banner instead of Stalled (don't blame the agent).
- **Multiple concurrent runs** (subagents). Tab shows the parent run; a count "+2 agents" opens the
  lanes view. Stall applies only when *all* are silent.
- **Background tab / unfocused window.** Clock slows (architecture.md); timer catches up on focus;
  Done fires a system notification only if the app opts in.
- **Approval opened then dismissed elsewhere** (other device): badge clears on the next state event.
- **Quota while queue has items.** Queue moves to `Held`; banner owns the explanation.
- **Clock skew / resumed runs.** Elapsed comes from `oxyn-core`'s run start timestamp, not the
  component's mount time, so remounting doesn't reset the timer.

## 7. Accessibility
- The tab is a `status` region. **Labels:** "Agent working: Editing auth.rs, 42 seconds", "Stop
  generating", "Agent waiting for your approval. Review", "Agent may be stalled: no response for 45
  seconds. Retry".
- **Live announcements (polite):** start ("Agent started"), approval needed (**assertive**), stalled
  hard, done ("Done, worked for 42 seconds"), interrupted, failed (**assertive**), offline/back
  online, quota reached. The timer is **not** announced every second; verb changes are announced at
  most once per 10 s (debounced) to avoid chatter.
- **Contrast:** dot and verb ≥ 4.5:1 in every tone including the stall ramp midpoint; warning/danger
  carry icons (⚠, ✕) and text, never colour alone; amber on light themes uses the darker token.
- **Focus order:** composer → (Shift+Tab) strip: tab (Stop) → approval badge → budget → job → PR →
  banner action. Banner action is reachable first when a banner is showing.
- **Keyboard:** Esc stops (when the composer or strip has focus); proposed `⌘/Ctrl+Shift+A` jumps
  to the pending approval card.
- **Reduced motion:** see timing table; meaning never depends on motion (stall = text + tone).

## 8. Props (sketch)
```rust
pub struct StatusStrip {
    pub state: StripState,              // derived from &AgentState by oxyn-core (incl. Stalled)
    pub variant: StripVariant,          // Tab | Detailed | InlineRow | Budget | FooterPills | Minimal | Pro(..)
    pub holds_stop: bool,               // true with send-control/status-tab
    pub allow_pause: bool,
    pub verb_source: VerbSource,        // VerbPack(id) | ModelHeader | Tool
    pub show_tokens: TokenDisplay,      // Never | After(30 s) | Always
    pub stall: StallThresholds,         // soft 5 s, hard 45 s, tool 120 s
    pub budget: Option<Budget>,         // Time{used, limit} | Tokens{..} | Cost{..}; pause_on_exhaust
    pub jobs: Vec<JobStatus>,           // { id, label, pct: Option<f32>, eta, state }
    pub pr: Option<PrStatus>,           // { number, branch, conflicts, additions, deletions, checks, state }
    pub banner: Option<Banner>,         // Offline{retry_in} | RateLimited{retry_in} | Quota{resets_at, action}
    pub done_hold: Duration,            // 4 s
    pub labels: StripLabels,
    pub motion: MotionLevel,
}

pub enum StripState {
    Hidden, Submitted,
    Working { verb: Verb, elapsed: Duration, tokens: Option<u64> },
    Stalled { level: StallLevel, since: Instant },
    AwaitingYou { request: RequestRef, paused_at: Duration },
    Done { elapsed: Duration }, Cancelled { after: Duration }, Failed { kind: ErrorKind },
}
```
Mapping: `Submitted`→`Submitted`; `Thinking/Streaming/ToolRunning`→`Working` (verb from verb pack or
tool); `Stalled{since}`→`Stalled` (soft/hard by elapsed since); `AwaitingApproval`→`AwaitingYou`;
`Done/Cancelled/Error` → same-named states; `Queued/Steering` keep `Working` (the queue stack shows them).

## 9. Related components
Send control (Stop, `status-tab` variant, [composer-send-control.md](composer-send-control.md)) · queue
stack (sits directly under the strip, [composer-queue-stack.md](composer-queue-stack.md)) · context meter
(token budget vs context window are different numbers, [composer-context-meter.md](composer-context-meter.md)) ·
permission selector (what "approve all" may do, [composer-permission-selector.md](composer-permission-selector.md)) ·
transcript "Interrupted" marker · thinking timeline · approval card · PR / merge-conflict card ·
background jobs tray · loaders (shared clock).

## 10. References (study only unless MIT/Apache)
| Ref | What we take | Licence note |
|---|---|---|
| renzobianchi [R079] | status tab with Stop; "Interrupted" drop, firm ease-out, no overshoot | Check source; study only |
| Cloudy [R007] | time-budget chip; pause ≠ stop | Study only |
| seanaiux studio [R013] | background job pill with % docked | Study only |
| Pitus merge-conflict gen-UI [R054] | PR pill "#128 3 conflicts · +482 −37"; amber → resolved | Study only |
| Pitus composer [R021] | "Thinking 13s" → "Thought for 14s" past-tense roll | Study only |
| T3 Code shell [R003] | status taxonomy Approval / Input / Working + elapsed / Done | Study only |
| Codex CLI status row (note 08) | `Working (12s • esc to interrupt)`, elapsed format, timer pauses on approval, shimmer timings | MIT — may port with attribution |
| Claude Code spinner (note 08) | stall-to-red ramp idea; count-up tokens after 30 s | Proprietary — reference only, no code |
| Zed Sand loader (note 08) | human-blocked state looks different from generating | Mixed licence; study only |
| signoff-ui RunMeter [R107] | Pro `run-meter` | MIT per licensing.md |
| flohoeller cost waterfall [R030] and agent lanes [R061]; zzzzshawn test ticks [R037]; dannyinsf_ pill mascot [R023] | Pro peeks | Study only |
