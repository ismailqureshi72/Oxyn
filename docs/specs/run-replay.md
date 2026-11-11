**Status:** draft v0.1 (2026-11-11).

# Spec: run replay scrubber (`run-replay`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
overview in [tools.md](tools.md)). Same depth as [composer-send-control.md](composer-send-control.md).
Target **4 free + 12 Pro = 16 variants**. This file defines the shared **`RunLog`** (§9) that
[trace-view.md](trace-view.md) derives spans from.

## 1. Purpose
A run took 20 minutes and 300 steps. The user wants to **see how it unfolded**: scrub back to the moment it
went wrong, watch the sequence at 4×, jump from tool call to tool call, and preview what the screen, files or
transcript looked like at that point — without re-running anything. Reviewers use it to audit an agent's
work; developers use it to debug; the Oxyn Studio uses the same control to play component state scripts.

What we saw: Replit's App Testing records a **video replay** of the agent's browser session with its cursor;
Devin's Shell tab has **time-travel command history** (both research note 03); checkpoint timelines exist as
a vertical list in a side panel or a horizontal scrubber, with Cursor previewing files when you click a
checkpoint (research note 05); Hoplite's long-run timeline lists milestones of a 61 h run ("Picked up
ENG-482, opened #212, CI failed overnight & fixed…") with stats Steps / PRs / Fixed CI / You stepped in
(research note 10); our own Studio research proposes a play/pause, time scrubber and 0.25×–2× speed under
the canvas with "Simulate" scripts (research note 17); a sidebar demo exposes a Scrub control in its
tweak panel [R046]; AG-UI's event stream (RUN_*, STEP_*, TEXT_MESSAGE_*, TOOL_CALL_*, STATE_SNAPSHOT /
DELTA, SUBAGENT_*) is a natural replay log (research note 05).

**Core decision (proposed): replay is a fold.** Every Oxyn component is already a pure function of
`AgentState` + props (architecture.md). Replaying to time *t* = folding the `RunLog` events up to *t* through
the same adapters and rendering the result. No video is required; video/screenshots are optional snapshot
attachments.

Anatomy (`scrubber`, docked under the transcript or a live view):
```
  ┌ Replay · run 4f2a · 2m 14s ──────────────────────────────────────────────────── ● Live ┐
  │ ⏮  ◀◀  ▶  ▶▶  ⏭   1× ▾   Skip idle ☑                         0:48 / 2:14            │ transport
  │ ▏·  · ⚒ ·  ·· ⚒   ◆─────────⚒⚒ ·  ⚠ ·· ⚒   ✕ ·  · ✓                                 ▕ │ event ticks by type
  │ ▏━━━━━━━━━━━━━━━━━━━━━━━━━━━●───────────────────────────────────────────────────────▕ │ track + playhead
  │ ▏ T1 plan     │ T2 research                 │ ⏸ approval │ T3 write          │ end    ▕ │ step markers (turns, phases)
  └──────────────────────────────────────────────────────────────────────────────────────┘
        ┌──────────────────────┐
        │ 0:31 · ⚒ fetch_pages │   hover snapshot preview (transcript tail / file / screenshot)
        │ [ thumbnail ]        │
        └──────────────────────┘
```

Rules (proposed):
1. **Replay never acts.** In replay mode every interactive component renders read-only (send, approve,
   stop disabled with reason "Viewing replay"). Leaving replay restores the live view.
2. **Live runs can be scrubbed**; scrubbing detaches from live; "● Live" returns to the head.
3. **Time is wall time**, with optional *Skip idle*: gaps > 2 s between events compress to 0.5 s of playback
   (proposed), shown as `≈` breaks on the track.
4. **Markers come from data**: turns, tool calls, approvals, errors, checkpoints, control changes
   ([acting-indicator.md](acting-indicator.md)) and sub-agent starts/ends; no invented chapters (a model may
   label phases only if the app supplies labels).
5. **Snapshots are optional and lazy**: transcript and state come from the fold; screenshots / video frames /
   file contents appear only if the app recorded them.

## 2. State machine
```
                    open replay (finished run)                 play
  Closed ───────────────────────────────────▶ Paused{t} ─────────────────▶ Playing{t, speed}
    ▲   open replay (live run) ──▶ Following{head} (plays at 1× with the run; same as live view + track)
    │                                  │ scrub / step / jump                 │ pause / Space
    │                                  ▼                                     ▼
    │                              Paused{t} ◀──────────────────────────── Paused{t}
    │                                  │ press-and-drag on track
    │                                  ▼
    │                            Scrubbing{t} (preview follows pointer, view updates at ≤ 30 Hz) ── release ──▶ Paused{t}
    │                                  │ reaches end of log
    │                Playing ── end ──▶ Ended (finished run) | Following{head} (live run, auto re-attach)
    │                Paused ── "● Live" ──▶ Following{head}
    │
    │◀── close / Esc (twice when playing) ── any
  Loading (log or snapshots fetching, show_delay 150 ms) ⇄ any · Seeking (long fold, > 100 ms) ⇄ any
  LoadFailed ("Couldn't load replay · Retry") · Partial (log truncated: "Replay starts at 12:04, earlier events not kept")
  SnapshotMissing at t (preview shows reconstructed state only, "No screenshot at this point")
```

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Closed` | "Replay" button on finished runs (transcript footer / trace header) | open replay | — | — |
| `Loading` | track skeleton; transport disabled | — | cancel | Esc |
| `Following{head}` | playhead pinned at the right end, "● Live" lit, ticks appended live | pause (detach) | — | Space pause |
| `Paused{t}` | playhead at t, time "0:48 / 2:14", views show state at t (read-only banner "Viewing replay at 0:48 · Back to live") | play | step ±, jump ± tool call | Space play, ←/→ step event, Shift+←/→ ±10 s, `[`/`]` prev/next tool call, `,`/`.` prev/next marker |
| `Playing{t, speed}` | playhead moves; views update; speed chip | pause | change speed | Space pause, `<`/`>` speed down/up |
| `Scrubbing{t}` | playhead follows pointer, snapshot preview pinned above it, views update live (throttled) | release to land | — | — |
| `Seeking` | thin indeterminate line under the track (> 100 ms folds) | — | cancel seek | Esc |
| `Ended` | playhead at end, ▶ becomes ↺ replay | replay from start | close | Space restart, Home start |
| `Partial` | hatched start region with note | — | — | — |
| `SnapshotMissing` | preview card with reconstructed transcript tail only | — | — | — |
| `LoadFailed` | danger note + Retry | retry | close | Enter retry |
| Marker hover | tooltip: type, label ("⚒ fetch_pages · 18.4 s"), time | jump | open in trace | Enter jump |

Speeds: 0.5×, 1×, 2×, 4×, 8×, 16× (proposed; the Studio variant uses 0.25×–2× per research note 17).

## 4. Micro-interactions
- **Open.** The track rises from the bottom edge of its host (y +8 → 0, 180 ms); ticks draw in left → right
  in a 300 ms sweep (one pass), then sit still.
- **Playhead.** 12 px knob on a 2 px track; grows to 16 px on hover / drag; time label floats above while
  dragging. Track fill left of the playhead in accent, right in track tone.
- **Scrub feel.** Fold results are cached every 50 events (keyframes, proposed) so scrubbing re-renders at
  ≤ 30 Hz; while dragging fast, views show the nearest keyframe and settle to the exact event 120 ms after the
  pointer stops (snaps without animation).
- **Magnetic markers.** Dragging within 6 px of a marker snaps to it with a 1 px haptic-like nudge (and a
  platform haptic on touch); hold Alt/Option to scrub without snapping.
- **Step.** ←/→ moves to the previous/next event; the playhead tweens 120 ms; the transcript scrolls to the
  affected message with `ease.out` 200 ms and the changed element gets a 600 ms highlight wash.
- **Jump to tool call.** `[`/`]` or clicking a ⚒ tick: playhead travels (240 ms `ease.in-out`); the tool row
  in the transcript expands; the trace view (if open) selects the matching span.
- **Play.** Views animate at the chosen speed; component micro-animations are time-scaled up to 2×, and above
  2× run with motion disabled (state changes only) to avoid strobing (proposed).
- **Snapshot preview.** Hover over the track shows a 160 × 100 px card after 200 ms (transcript tail,
  focused file excerpt, or screenshot thumbnail if recorded); the card follows the pointer horizontally,
  clamped to the track.
- **Back to live.** "● Live" click: playhead glides to the head (240 ms), the read-only banner slides away,
  live interactivity returns.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Track open | 180 ms, y +8 → 0 | — | `ease.out` | — | 60 Hz |
| Tick draw-in | 300 ms one sweep | — | `ease.out` | by position | 60 Hz |
| Playhead hover grow | 120 ms, 12 → 16 px | — | `spring.snap` | — | 60 Hz |
| Scrub re-render | throttled | — | — | — | ≤ 30 Hz |
| Settle to exact event | — | 120 ms after pointer stops | instant | — | — |
| Marker snap nudge | 80 ms, 1 px | — | `spring.snap` | — | 60 Hz |
| Step playhead tween | 120 ms | — | `ease.out` | — | 60 Hz |
| Transcript scroll on step | 200 ms | — | `ease.out` | — | 60 Hz |
| Changed-element highlight | 600 ms wash | — | `ease.out` fade | — | 30 Hz |
| Jump to tool call | 240 ms | — | `ease.in-out` | — | 60 Hz |
| Snapshot preview | 140 ms fade in | 200 ms hover | `ease.out` | — | 60 Hz |
| Back to live glide | 240 ms | — | `ease.in-out` | — | 60 Hz |
| Playback component motion | × speed up to 2×; off above 2× | — | as component | — | 60 Hz |
| Playhead during play | continuous | — | linear | — | 30 Hz (60 Hz above 4×) |
| **Reduced motion** | track appears with 80 ms fade, ticks present immediately; playhead jumps on step/jump (no tween), transcript jumps instead of scrolling, highlight is a static outline for 1 s; playback renders state changes only (no component motion at any speed); preview appears without fade | — | linear | none | event-driven, ≤ 10 Hz playhead |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `run-replay/scrubber` | Baseline: transport + typed event ticks + step markers + snapshot preview (anatomy above) | desktop, ide, chat |
| `run-replay/step-list` | Vertical list of events/steps with a playhead row; click a step to jump; for narrow side panels | ide, mobile |
| `run-replay/mini` | Collapsed 24 px track under a finished message ("Replay 2m 14s ▶"), expands to `scrubber` | chat |
| `run-replay/terminal` | Bottom line `⏮ ◀ ▶ ▶▶ ⏭ 1× [━━━━●────] 0:48/2:14` with key bindings; replays terminal output | terminal |

### Pro (12)
| Name | One-line idea |
|---|---|
| `run-replay/filmstrip` | Track shows screenshot thumbnails at markers (computer-use / browser agents, needs recorded frames) |
| `run-replay/video-sync` | Recorded video synced to the event log; ticks overlay the video timeline (Replit-style idea, note 03) |
| `run-replay/chapters` | App-supplied phase labels as chapters with a chapter list (only with labels from data) |
| `run-replay/milestones` | Long-run milestone timeline with stats (steps, PRs, fixes, "you stepped in") (Hoplite idea, note 10) |
| `run-replay/diff-scrub` | Scrubbing shows the cumulative file diff at t; a file tree highlights touched files |
| `run-replay/cursor-trail` | Replays the agent cursor path with clicks over a snapshot (pairs with acting-indicator) |
| `run-replay/lanes` | Multi-agent replay: one track per agent, shared playhead (pairs with fanout swimlanes) |
| `run-replay/density-heat` | Track background shows event density / token rate as a heat strip |
| `run-replay/compare` | Two runs on stacked tracks aligned by step; jump both together |
| `run-replay/share-clip` | Select a range on the track and export it as a shareable replay link (app supplies hosting) |
| `run-replay/studio-sim` | Studio control: plays Simulate scripts through components at 0.25×–2× (note 17) |
| `run-replay/vinyl` | Expressive jog-wheel scrubber (drag the wheel to scrub, flick to spin) for touch |

## 7. Edge cases
- **Huge logs** (> 100 000 events): ticks aggregate into density bins below 2 px per event; keyframes every 50
  events keep seek under 100 ms (proposed budget); older events may be paged in on demand (`Seeking`).
- **Live run growing while paused**: the head keeps moving; the time label shows "0:48 / 2:14 · live +3s".
- **Events with identical timestamps**: ordered by sequence number; stepping visits each; ticks offset by 1 px.
- **Clock skew across agents**: log order is by receive sequence; display time uses the adapter timestamp with
  skew clamped (as trace-view.md §7).
- **Redacted content**: payloads marked secret by the adapter render as masked in replay too; snapshots never
  un-redact.
- **Snapshots missing / expired**: preview shows reconstructed state; the filmstrip shows placeholders.
- **State not reconstructible** (adapter lacks deltas before a STATE_SNAPSHOT): replay starts at the first
  snapshot (`Partial`).
- **Very short runs** (< 2 s): track still shows; Skip idle has no effect; speeds above 2× disabled.
- **Very long runs** (days): track zoom (pinch / Ctrl+scroll) with a minimap; times show `1d 03h 12m`.
- **Approval waits**: drawn as hatched ⏸ segments; Skip idle compresses them too (labelled "waited 4m").
- **Long labels** on markers: 24 ch in tooltips, full in the step list.
- **RTL**: transport buttons and the time axis do **not** mirror; they keep left → right, following the
  common media-player convention (proposed). Surrounding labels mirror.
- **Tiny sizes**: under 320 px, transport shrinks to ▶ + time; ticks become a density strip.
- **Dense IDE panels**: 20 px track, markers as 1 px ticks, preview disabled (step list instead).
- **Touch**: 44 px playhead hit area; drag on the track to scrub; long-press a marker for details.
- **Slow networks**: snapshots load lazily with a skeleton thumbnail; scrubbing never waits for snapshots.

## 8. Accessibility
- The track is a `slider` with `aria-valuemin` 0, `aria-valuemax` run duration (s), `aria-valuenow`, and
  `aria-valuetext` "48 seconds of 2 minutes 14 seconds, during tool call fetch_pages". Transport controls are
  buttons with state labels ("Play", "Pause", "Speed 2 times").
- Markers are also exposed as a list (the step list) so screen-reader users can navigate by event type.
- **Live region (polite):** on step/jump, announce the event ("Tool call fetch_pages, 18 seconds"); during
  playback, announce only markers of types the user filters for (default: errors and approvals), max one per
  2 s; "Back to live" announced.
- **Contrast:** playhead and fill ≥ 3:1; tick glyphs distinguish types by shape (⚒ ✦ ⚠ ✕ ◆), not tone alone.
- **Focus order:** transport → speed → Skip idle → slider → Live; Esc leaves replay (returns focus to the
  Replay button).
- **Keyboard:** Space play/pause, ←/→ step, Shift+←/→ ±10 s, Home/End, `[`/`]` tool calls, `,`/`.` markers,
  `<`/`>` speed.
- **Reduced motion:** §5 last row; playback never flashes (component motion off at any speed).

## 9. Props sketch
Shared log (proposed, `oxyn-core`; adapters append; trace-view and replay read):
```rust
pub struct RunLog {
    pub run: RunId,
    pub events: Vec<RunEvent>,              // append-only, ordered by `seq`
    pub keyframes: Vec<(u64, FoldState)>,   // every 50 events (seek cache)
    pub starts_at: Option<u64>,             // Partial when earlier events were dropped
}

pub struct RunEvent {
    pub seq: u64,
    pub at: SystemTime,                     // adapter timestamp
    pub agent: AgentId,
    pub kind: RunEventKind,                 // StateChange(AgentState) | TextDelta | ToolStart | ToolEnd | Approval
                                            // | SubagentStarted | SubagentFinished | Checkpoint | ControlChange
                                            // | Usage(Usage) | Snapshot(SnapshotRef) | Custom
}

pub struct RunReplay {
    pub log: RunLogRef,
    pub mode: ReplayMode,                   // Following | Paused(t) | Playing { t, speed } | Scrubbing(t)
    pub variant: ReplayVariant,             // Scrubber | StepList | Mini | Terminal | Pro(..)
    pub speeds: SmallVec<[f32; 6]>,         // 0.5, 1, 2, 4, 8, 16
    pub skip_idle: Option<SkipIdle>,        // { gap: 2 s, compress_to: 500 ms }
    pub markers: MarkerFilter,              // Turns | ToolCalls | Approvals | Errors | Checkpoints | Control | Subagents
    pub snapshots: Option<SnapshotProvider>,// lazy screenshots / files / video; None → reconstructed only
    pub on_seek: Callback<Instant>,         // host re-renders views from the fold at t
    pub on_exit: Callback<()>,
    pub motion_scale_cap: f32,              // 2.0 — above this, playback disables component motion
    pub labels: ReplayLabels,
    pub motion: MotionLevel,
}
```
Fold: `state_at(t) = keyframe_before(t).apply(events in (k, t])`, producing per-agent `AgentState`,
transcript, `RunGraph`, open spans and control state; every component renders from that, read-only.

## 10. Related components and references
Related: [trace-view.md](trace-view.md) (same log; "Replay from here") · [subagent-tree.md](subagent-tree.md)
(graph at time t) · [acting-indicator.md](acting-indicator.md) (control segments as markers) ·
[tool-call.md](tool-call.md) (jump target) · [terminal-output.md](terminal-output.md) (terminal replay) ·
[plan-todo.md](plan-todo.md) (plan state at t) · [elapsed-timer.md](elapsed-timer.md) (time formats) ·
checkpoint timeline (family H, [../component-taxonomy.md](../component-taxonomy.md)) · Studio state scrubber
([../studio.md](../studio.md)).

| Ref | What we take | Licence note |
|---|---|---|
| Replit App Testing video replay (note 03) | replay with agent cursor | Proprietary; study only |
| Devin shell time-travel history (note 03) | scrub through command history | Proprietary; study only |
| Checkpoint timelines (note 05: Cursor preview, horizontal scrubber orientation) | marker + preview pattern | Proprietary; study only |
| Hoplite long-run timeline (note 10) | milestones + "you stepped in" stat | Proprietary; study only |
| AG-UI events (note 05) | event types for the log | Protocol; implement from spec |
| Studio scrubber idea (note 17) | play / pause, scrubber, 0.25×–2×, Simulate scripts | Our own research recommendation |
| Sidebar tweak panel [R046] | Scrub control in a tweak panel | Study only |
