**Status:** draft v0.1 (2026-11-11).

# Spec: background jobs tray (`background-jobs`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
overview in [tools.md](tools.md)). Same depth as [composer-send-control.md](composer-send-control.md).
Target **4 free + 12 Pro = 16 variants**. The minimized form **is** the job pill of
[composer-status-strip.md](composer-status-strip.md); both read the same `JobStatus` list, and per-job
progress follows the `JobProgress` rules of [container-loaders.md](container-loaders.md) §4.2.

## 1. Purpose
Some agent work outlives the turn: a video render, a long test suite, a 200-file index, a background
sub-agent, a scheduled automation, a deploy. The user moves on and wants to **know what is still running,
how far along it is, be told when it finishes, and cancel or retry without hunting for it**.

What we saw: a background job pill docked bottom-centre with a percentage, "Image to video · Rendering 4%"
[R013]; a time budget chip "2h 06m of 24h" [R007]; Claude Code `/tasks` lists background shells and
subagents and keeps finished ones listed as done (research note 05); Claude Code agent view lists
background sessions with a next-run time for looping jobs (research note 05); Paradev's automations table
shows "⠋ running, ✓ 2h ago" (research note 03); Antigravity's status icons distinguish working, running a
background command and waiting on your answer, with a completion sound and an action-required chime, and a
menu-bar icon counting running agents (research note 08); Zed has notifications + sleep prevention
(research note 03); Hoplite's billing page can retry every thread that stopped for credits (research
note 10); Claude desktop minimizes panes to a floating corner tile (research note 03); Cuelume offers
sound cues for ready / success / error / attention [R110].

Anatomy (`popover` opened from the strip's job pill):
```
 composer status strip:  … │ ◔ 3 jobs · 41% │ …          ← minimized pill (strip job pill)
                              ▼
┌ Background · 3 running · 1 finished ────────────── Clear finished ┐
│ ◔ Render "launch.mp4"      41% · ~2m left      ■ Cancel           │  determinate + app ETA
│ ⠿ Indexing repo            1 284 files                ■          │  unknown total: live count
│ ◑ Test suite (Researcher)  Step 3/7 · 0:48            ■          │  steps
│ ✓ Export report.zip        Done 12s ago        Open ↗             │  finished (kept until cleared)
│ ✕ Deploy preview           Failed · exit 1     Retry · Log        │  failed (sticky)
└───────────────────────────────────────────────────────────────────┘
```

Rules (proposed):
1. **One list, two sizes**: the pill (strip, dock) and the tray (popover, panel) render the same
   `Vec<JobStatus>`; opening the pill opens the tray anchored to it.
2. **Progress is honest**: determinate only when the job reports it; ETA only when the job supplies it
   (container-loaders.md §4.2 rules 1 and 7). The pill's combined % is the mean of jobs with known %,
   labelled "3 jobs · 41%"; when none is known, the pill shows a count only.
3. **Finish notifies once, in the right place**: in-app toast when the app is focused and the tray is
   closed; system notification only if the app opts in and the window is unfocused; a failed job stays
   in the tray until dismissed.
4. **Cancel is safe by default**: one press for jobs the app marks `cheap`; a confirm for jobs marked
   `costly` (long renders, paid runs) — "Cancel render? 41% will be lost."
5. **Finished jobs persist** until cleared or the app's retention (default 24 h, proposed), so the user
   can find the output later.

## 2. State machine
Per job:
```
                  enqueue                    worker starts               progress events
  (absent) ─────────────────▶ Queued{pos} ──────────────────▶ Running ──────────────────────▶ Running{p | k/n | count}
                                  │ cancel                      │  ▲   │ awaiting approval         │ no event for job_stall
                                  ▼                             │  │   ▼                           ▼
                              Cancelled ◀── cancel (confirm) ───┤  └─ NeedsYou{request} ◀──     Stalled ── event ──▶ Running
                                                                │       (paused, amber)
                         pause (user / offline / budget) ──▶ Paused{reason} ── resume ──▶ Running
                                                                │
                                         success ──▶ Succeeded{output} ── seen / cleared / retention ──▶ (removed)
                                         failure ──▶ Failed{kind} ── retry ──▶ Queued (attempt + 1) | dismiss ──▶ (removed)
                                         app restarted mid-job ──▶ Unknown ("Lost track · Check") ── refresh ──▶ any
Tray-level:
  Hidden (no jobs) ⇄ Pill{summary} ── open ──▶ Open{popover | panel} ── close / Esc / outside click ──▶ Pill
  Pill ── job finishes ──▶ PillFlash{done | failed} (6 s) ──▶ Pill or Hidden
```

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| Tray `Hidden` | nothing; no reserved space | — | — | — |
| `Pill{summary}` | "◔ Render 41%" (1 job) or "◔ 3 jobs · 41%" / "3 jobs" (no known %); ring determinate only when known | open tray | — | Enter / Space open |
| `PillFlash{done}` | ✓ "Render ready · Open" for 6 s, success tint | open output | open tray | Enter open |
| `PillFlash{failed}` | ✕ "Render failed · Retry" (sticky until opened) | retry | open tray | Enter retry |
| `Open` | job rows, header counts, Clear finished | per-row action | clear finished | ↑/↓ rows, Enter primary, Delete cancel/dismiss, Esc close |
| `Queued{pos}` | ○ hollow, "Queued · 2nd" | — | cancel | Delete |
| `Running{p}` | ring/bar to p, "41% · ~2m left" (ETA only if supplied) | — | cancel | Delete cancel |
| `Running{k/n}` | ring with n ticks, "Step 3/7" | — | cancel | Delete |
| `Running{count}` | indeterminate arc, live count "1 284 files" | — | cancel | Delete |
| `NeedsYou{request}` | ⚠ amber, "Waiting for approval", progress frozen | **Review** | cancel | Enter review |
| `Stalled` | ring slows to 0.5×, warning tone, "No progress for 2m" | retry (if supported) | cancel | Enter retry |
| `Paused{reason}` | ⏸ frozen at 60 %, "Paused · offline" / "Paused · budget" / "Paused by you" | resume | cancel | Enter resume |
| `Succeeded{output}` | ✓, "Done 12s ago", output name | **Open** / reveal | copy link, dismiss | Enter open, Delete dismiss |
| `Failed{kind}` | ✕ danger, short reason ("exit 1", "out of credits") | **Retry** | log, dismiss | Enter retry |
| `Cancelled` | ■ "Cancelled at 41%" for 4 s, then removed | — | undo if the app supports resume | — |
| `Unknown` | ? "Lost track after restart" | check status | dismiss | Enter |

## 4. Micro-interactions
- **Job joins.** When a job leaves the transcript for the background (e.g. "Build in background"), its
  row's loader detaches and travels to the pill (`dur.travel` 240 ms, `ease.in-out`); the pill grows in
  width with `spring.snap` and its count rolls. Without a visible origin, the pill just rolls its count.
- **Progress.** Displayed fraction follows the reported one with `spring.snap`, never more than 250 ms
  behind, never backwards (container-loaders.md §4.2). Indeterminate → determinate: the arc anchors and
  becomes the fill.
- **Percent text.** Rolls (odometer 280 ms) at most every 500 ms; tabular digits in a reserved width.
- **Open tray.** Popover scales from the pill (0.96 → 1, origin = pill) with fade, 180 ms; rows stagger 30
  ms (cap 6). Panel variant slides from the edge.
- **Finish.** The ring closes, the pill flashes success tint (one 400 ms pulse) and the label rolls to
  "ready · Open"; the row's ✓ morphs with `spring.snap`. Optional sound cue (app opt-in; idea from
  Antigravity's completion sound and Cuelume [R110]).
- **Fail.** Ring segment holds where it stopped in danger tone; pill does a 2-cycle 3 px nudge; label "failed
  · Retry" stays until acted on.
- **Cancel.** Row compresses (height → 0, 160 ms) after the 4 s "Cancelled" note, or immediately on
  Clear; costly-cancel confirm appears inline in the row (no modal).
- **Retry.** Ring resets with a visible fade-out and redraw from 0 (container-loaders.md rule 2); attempt
  badge "2".
- **Hover row.** Reveals Cancel / Retry / Open; tooltip: started at, elapsed, source agent, attempt.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Loader travel to pill | 240 ms | — | `ease.in-out` | 80 ms if several | 60 Hz |
| Pill width change | 200 ms | — | `spring.snap` | — | 60 Hz |
| Count / % roll | 280 ms, max every 500 ms | — | `ease.out` | 30 ms per digit | 30 Hz |
| Progress follow | ≤ 250 ms lag | — | `spring.snap` | — | 30 Hz while moving |
| Indeterminate arc | 1.2 s loop | `show_delay` 150 ms | linear | max 8 animate | 30 Hz |
| Tray open / close | 180 ms / 140 ms | — | `ease.out` / `ease.in` | rows 30 ms, cap 6 | 60 Hz |
| Done pulse + morph | 400 ms pulse; ✓ 200 ms | — | `ease.out`; `spring.snap` | — | 60 Hz |
| Pill done hold | 6 s | — | — | — | — |
| Fail nudge | 240 ms, 2 × 3 px | — | linear | — | 60 Hz |
| Cancelled note hold / collapse | 4 s / 160 ms | — | — / `ease.in` | — | 60 Hz |
| Retry reset | 160 ms fade out + redraw | — | `ease.out` | — | 60 Hz |
| Stall slow-down + tone | 400 ms tempo change; 2 s ramp | after `job_stall` (default 120 s, proposed) | `ease.in-out`; OKLab linear | — | 30 Hz |
| Toast in / out | 200 ms / 160 ms; hold 5 s | — | `ease.out` / `ease.in` | — | 60 Hz |
| **Reduced motion** | no travel (pill count updates with 80 ms crossfade); progress bars still fill (information) but without easing, updated at 2 Hz; no indeterminate arc (static "…" + live count); no pulses or nudges (tone + text only); tray opens with 80 ms fade | — | linear | none | 2 Hz |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `background-jobs/popover` | Baseline: strip job pill opens an anchored popover list (anatomy above) | chat, desktop |
| `background-jobs/panel` | Docked side or bottom panel with filters (running / finished / failed) and output links | desktop, ide |
| `background-jobs/dock-pill` | Pill docked bottom-centre of the window, independent of the composer, with % [R013] | desktop, mobile, media agents |
| `background-jobs/terminal-list` | `/jobs`-style list: `[1] ⠋ render 41%  [2] ✓ export` with numbered cancel/retry | terminal |

### Pro (12)
| Name | One-line idea |
|---|---|
| `background-jobs/stacked-toasts` | Finished jobs stack as toasts in a corner; expand the stack into the tray |
| `background-jobs/corner-tile` | Minimized job shown as a floating corner tile with a live thumbnail (Claude desktop idea, note 03) |
| `background-jobs/menu-bar` | Tray / menu-bar icon with running count and popover (Antigravity idea, note 08) |
| `background-jobs/notch` | Desktop island shows the most important job's ring and label |
| `background-jobs/render-queue` | Media render queue with frame thumbnails and frame k/n (video agents) |
| `background-jobs/timeline` | Jobs as bars on a shared time axis (started, now, ETA when supplied) |
| `background-jobs/ring-cluster` | One ring per job clustered in the pill; hover splits them out |
| `background-jobs/scheduled` | Adds scheduled / looping jobs with "next run in 2h" and last-run status |
| `background-jobs/budget-meter` | Each job shows spent vs time / token / cost budget (cost only with prices) [R007] |
| `background-jobs/batch-retry` | Groups failures by reason ("4 stopped for credits · Retry all") (Hoplite billing idea, note 10) |
| `background-jobs/sound-cues` | Opt-in sound pack for started / ready / failed / needs-you [R110] |
| `background-jobs/ascii-bars` | Terminal progress bars `▕████▍    ▏ 41%` with braille spinner for unknown totals |

## 7. Edge cases
- **Many jobs** (> 20): tray virtualises; running and needs-you first, then failed, then finished by recency.
  Pill caps at "20+ jobs".
- **Mixed known/unknown %**: combined % from known jobs only, labelled "41% of 2" in the tooltip.
- **ETA unstable**: if supplied ETAs jump by > 50 % twice within 30 s, hide ETA and show elapsed (proposed;
  same idea as elapsed-timer `eta`).
- **App restart / reload**: jobs rehydrate from the app's store; if a job's state can't be confirmed it shows
  `Unknown` rather than a fake Running.
- **Job finishes while tray open**: no toast; the row morphs in place.
- **Job owned by a sub-agent**: row shows the sub-agent swatch; Review / Open routes to that child
  ([subagent-tree.md](subagent-tree.md)).
- **Cancel not supported**: Cancel hidden; tooltip "This job can't be stopped".
- **Offline**: jobs on remote workers keep running; the tray shows "last update 40s ago" per row and pauses
  stall detection (don't blame the job).
- **Long labels**: 32 ch, middle truncation for file names; full label in tooltip.
- **RTL**: list mirrors; % and numbers stay LTR; ring progress direction stays clockwise (proposed;
  progress rings are not text).
- **Tiny sizes (16–20 px)**: pill becomes a 16 px ring + count badge.
- **Dense IDE panels**: status-bar item `⟳ 3` that opens the panel variant.
- **Touch**: rows 44 px; swipe-left cancel / dismiss with confirm for costly jobs.
- **Very fast jobs** (< 150 ms): never enter the tray; no toast.
- **Very slow jobs** (days): elapsed `1d 03h`; retention starts at finish, not start.

## 8. Accessibility
- Pill is a button: "3 background jobs, 41 percent, open list"; tray is a `dialog` (popover) or `region`
  (panel) containing a `list`; each row's progress is a `progressbar` with `aria-valuenow` when determinate,
  `aria-valuetext` "1 284 files indexed" when not.
- **Live region (polite):** job finished ("Render ready"), cancelled; failed and needs-you are **assertive**.
  Progress is announced only at 25 / 50 / 75 % if the app opts in, never per change.
- **Contrast:** ring ≥ 3:1 against track; text ≥ 4.5:1; states carry glyphs (✓ ✕ ⚠ ⏸ ■) plus text.
- **Focus order:** pill → (open) tray header (Clear finished) → rows → row actions; Esc closes and returns
  focus to the pill. Toasts are reachable with the app's notification shortcut and never steal focus.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct BackgroundJobs {
    pub jobs: Vec<JobStatus>,               // same type as composer-status-strip.md `jobs`
    pub variant: JobsVariant,               // Popover | Panel | DockPill | TerminalList | Pro(..)
    pub anchor: JobsAnchor,                 // StatusStrip | WindowDock | StatusBar
    pub notify: NotifyPolicy,               // { toast: true, system: false (opt-in), sound: None }
    pub retention: Duration,                // 24 h for finished jobs
    pub job_stall: Duration,                // 120 s
    pub on_open: Callback<JobId>,
    pub on_cancel: Option<Callback<JobId>>, // None hides Cancel everywhere
    pub on_retry: Option<Callback<JobId>>,
    pub on_review: Callback<RequestRef>,
    pub on_clear_finished: Callback<()>,
    pub labels: JobsLabels,
    pub motion: MotionLevel,
}

pub struct JobStatus {
    pub id: JobId,
    pub label: SharedString,
    pub kind: JobKind,                      // Render | Test | Index | Deploy | SubAgent(AgentId) | Automation | Other
    pub owner: Option<AgentId>,
    pub progress: JobProgress,              // container-loaders.md §4.2
    pub eta: Option<Duration>,              // only if the job supplies it
    pub state: JobState,                    // Queued{pos} | Running | NeedsYou{request} | Stalled | Paused{reason}
                                            // | Succeeded{output} | Failed{kind} | Cancelled | Unknown
    pub cost_class: CancelCost,             // Cheap (no confirm) | Costly (confirm)
    pub attempt: u16,
    pub started: Option<SystemTime>,
}
```
Mapping: a background sub-agent's `AgentState` maps to `JobState` (`Thinking/Streaming/ToolRunning` →
`Running`, `AwaitingApproval` → `NeedsYou`, `Stalled` → `Stalled`, `Done` → `Succeeded`, `Error` →
`Failed`, `Cancelled` → `Cancelled`); non-agent jobs (renders, deploys) report `JobState` directly.

## 10. Related components and references
Related: [composer-status-strip.md](composer-status-strip.md) (job pill = minimized tray) ·
[container-loaders.md](container-loaders.md) (`JobProgress` rules, per-job loaders) ·
[terminal-output.md](terminal-output.md) (background shells) · [subagent-tree.md](subagent-tree.md)
(background sub-agents) · [fanout-grid.md](fanout-grid.md) · [elapsed-timer.md](elapsed-timer.md)
(elapsed, budget) · [run-meter.md](run-meter.md) · notification centre and tray popover (family J).

| Ref | What we take | Licence note |
|---|---|---|
| seanaiux studio [R013] | docked job pill with % | Study only |
| Cloudy [R007] | time budget chip | Study only |
| Claude Code `/tasks`, agent view (note 05) | background shells + subagents, finished stay listed; next-run time | Proprietary — reference only |
| Paradev automations (note 03) | "⠋ running, ✓ 2h ago" | Licence not checked; study only |
| Antigravity (note 08) | background-command status icon; completion sound; menu-bar running count | Proprietary; study only |
| Claude desktop (note 03) | minimize to a floating corner tile | Proprietary; study only |
| Hoplite billing (note 10) | retry every thread stopped for credits | Proprietary; study only |
| Cuelume [R110] | sound cues for ready / success / error / attention | MIT per licensing.md — portable with attribution (sounds re-made in-house) |
