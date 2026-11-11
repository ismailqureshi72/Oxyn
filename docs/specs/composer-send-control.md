# Spec: composer send control (send / stop / queue / steer)

**Status:** draft v0.1 (2026-11-11). Hero component of the composer family. Target **12 free + 20 Pro
variants**, all driven by one state machine.

## 1. Why it matters
The send arrow is the most-touched control in any agent app and the place where agent state meets the
user's intent. Mid-run the user may want to **stop**, **pause**, **queue** a follow-up, or **steer**
the current run. Most kits ship one arrow that becomes one square. We ship the full behaviour.

## 2. State machine

```
              type            Enter / click
  Empty ───────────▶ Ready ──────────────▶ Sending ──first event──▶ Working
    ▲                  │                                              │
    │◀── clear ────────┘                                              │ type while working
    │                                                                 ▼
    │                                                        WorkingWithDraft
    │                                                    ┌──── Enter ─┼─ ⌘/Ctrl+Enter ───┐
    │                                                    ▼            │                  ▼
    │                                                 Queued(n)       │               Steering
    │                                                    │            │                  │
    │◀── run ends: Done / Cancelled / Error ─────────────┴────────────┴──────────────────┘
  Special: Disabled(reason) · Answering(card) · Dictating · Error(retry)
```

| State | Button shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Empty` | ↑ muted, no hover lift | none (disabled) | mic | — |
| `Ready` | ↑ accent | send | send-options menu (split ↑▾) | Enter send, Shift+Enter newline |
| `Sending` | instant swap to ■ (no spinner) | stop | — | Esc |
| `Working` | ■ stop (optional progress ring / elapsed) | stop | pause (separate) | Esc stop, Esc×2 confirm (variant) |
| `WorkingWithDraft` | ■ + **Queue** pill or split ■ / ↑ | queue (default) | steer, stop | Enter queue, ⌘↵ steer now |
| `Queued(n)` | as Working + "Queued n" stack above composer | — | per chip: edit, send now, steer, remove, reorder | ↑ edits last queued |
| `Steering` | brief "steer" glyph pulse, then Working | — | — | — |
| `Answering(card)` | ↑ answers the open question/approval card | submit answer | — | Enter |
| `Dictating` | ✓ confirm transcript | accept | cancel ✕ | Enter accept, Esc cancel |
| `Disabled(reason)` | ↑ muted + tooltip reason (offline, quota, auth) | — | — | — |
| `Error` | ⟲ retry in danger tone + 2-cycle shake | retry | edit | Enter retry |

Rules:
1. The composer **never changes position**; it may grow upward.
2. `Sending → Working` is instant. The draft text flies up and becomes the message bubble (≤ 260 ms, ease-in).
3. Stop and pause are different actions; pause is opt-in per app.
4. Enter while busy follows the app's `busy_action`: `queue` (default) · `steer` · `interrupt` · `ask` (shake the three choices).
5. On stop, the run's status drops into the transcript as "Interrupted".
6. Every state has a text label for screen readers ("Stop generating", "Queue message", "Steer now").

## 3. Timing (defaults; every variant may override)
| Transition | Duration | Easing |
|---|---|---|
| Empty → Ready (colour + scale 0.92 → 1) | 120 ms | ease-out |
| Ready → Sending (glyph morph ↑ → ■) | 160 ms | spring (critically damped) |
| Draft → bubble fly-up | ≤ 260 ms | ease-in |
| ■ → Queue pill widen | 180 ms | spring |
| Queued chip unfold upward | 200 ms + 40 ms stagger (cap 4) | ease-out |
| Steer button blur-slide in | 160 ms | ease-out |
| Error shake | 300 ms, 2 cycles, 6 px | linear |
| Reduced motion | all of the above become 0–80 ms opacity crossfades | — |

## 4. Variants

### Free (12)
| Name | Idea |
|---|---|
| `send-control/classic` | Round ↑ → ■, the baseline |
| `send-control/ring-progress` | ■ with a thin progress / elapsed ring around it |
| `send-control/queue-pill` | ■ widens into a labelled "Queue" pill when a draft exists |
| `send-control/split` | ↑ ▾ split button; mid-run becomes ■ │ ↑ (stop │ queue) |
| `send-control/steer-slide` | Steer button blur-slides in beside ■ when there's a draft |
| `send-control/status-tab` | Dot + timer tab above the composer holds Stop; button stays ↑ for queue |
| `send-control/pause-stop` | Separate pause and stop, with a time-budget chip |
| `send-control/glyph` | Terminal look: `⏎` / `■` / `⇥ queue` glyphs (TUI surface) |
| `send-control/voice-confirm` | Mic → waveform strip + live transcript; ↑ becomes ✓ |
| `send-control/answer-card` | ↑ answers the pending question/approval card |
| `send-control/retry` | Error ⟲ with shake + inline "Couldn't send · Retry" |
| `send-control/minimal` | Text-only "Send / Stop / Queue" for dense IDE panels |

### Pro (20, examples)
`liquid` (morphing blob), `particle` (dot field collapses into ■, matches the Oxyn mark),
`orbit` (comet circles while working), `shockwave-done` (arrival ring on completion),
`budget-meter` (tokens/cost fill the ring), `shake-choice` (Enter while busy shakes three choices),
`lanes` (user queue vs agent work queue), `steer-into-plan` (steer note glides into the plan step),
`magnetic`, `pixel`, `ascii-spinner`, `border-beam`, `ember-caret`, `split-flap-label`, `hold-to-stop`,
`swipe-to-steer` (mobile), `long-press-options`, `edge-glow`, `notch` (desktop island), `voice-orb`.

## 5. Props (sketch)
```rust
pub struct SendControl {
    pub state: ComposerState,          // derived from AgentState + draft + queue
    pub variant: SendVariant,
    pub busy_action: BusyAction,       // Queue | Steer | Interrupt | Ask
    pub allow_pause: bool,
    pub stop_confirm: StopConfirm,     // None | DoubleEsc | Hold(ms)
    pub show_elapsed: bool,
    pub budget: Option<Budget>,        // time or tokens
    pub labels: SendLabels,            // i18n + screen reader text
    pub motion: MotionLevel,           // None | Subtle | Expressive
}
```

## 6. Related components
Queued-message stack · steer note · status tab · time-budget chip · voice strip · question card ·
approval chip ("Request approval" / "Bypass all") · attachment tiles with ring progress.

## 7. References (study only unless MIT/Apache)
Phil Pitus composer + voice ✓ and thinking timeline · Cloudy steer-in-place + pause/stop + budget chip ·
BoardUI attachment ring and "Bypass all" · affectUI steering (Enter queues, ⌘↵ interrupts) · T3 Code
"Enter to queue" · Hermes `busyAction` modes · Amp second-Enter steer · Glaze / Replit queued chips ·
fal.ai two lanes · Renzo Bianchi on composer exit animations · coss ui Composer API (MIT).
