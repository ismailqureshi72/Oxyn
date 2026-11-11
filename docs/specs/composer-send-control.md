**Status:** draft v0.2 (2026-11-11).

# Spec: composer send control (send / stop / queue / steer)

v0.2 adds micro-interactions, edge cases, accessibility and source IDs
so the spec meets all 10 points of [README.md](README.md); v0.1 content is unchanged. This is the **worked example** of the depth every Oxyn component gets
(see [README.md](README.md)); nothing here is special to the send button. Target **12 free + 20 Pro
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

## 3. Micro-interactions
- **Hover / press.** `Ready`: hover lifts the button 1 px with a slightly stronger shadow; press scales
  0.94 for 80 ms. `Empty` has no hover lift [R044]. 32 px hit area on desktop, 44 px on touch.
- **Focus ring.** 2 px accent ring, 2 px offset, follows the morphing shape (circle → pill) rather than
  a fixed square.
- **First keystroke.** Muted ↑ lights up: colour + scale 0.92 → 1 [R044].
- **Send.** The draft text flies up and becomes the bubble in one commit (≤ 260 ms, ease-in) [R079];
  the button swaps to ■ instantly, no spinner [R086].
- **Queue pill.** Typing while busy widens ■ into a labelled "Queue" pill [R087]; the chip it creates
  unfolds upward above the composer [R079] (see [composer-queue-stack.md](composer-queue-stack.md)).
- **Steer slide.** A steer-send button blur-slides in beside ■ once there is a draft [R086].
- **Busy-choice shake.** With `busy_action = Ask`, Enter shakes Steer / Queue / Stop to force a choice
  [R085].
- **Stop.** ■ press → the status line drops into the transcript as "Interrupted" [R079] (owned by
  [composer-status-strip.md](composer-status-strip.md)); the button morphs ■ → ↑ (or ↑ muted if empty).
- **Done.** ■ → ↑ morph; no celebration on the button by default (Pro `shockwave-done` adds one).
- **Error.** ⟲ in danger tone + 2-cycle shake + inline "Couldn't send · Retry".
- **Dictation.** ↑ → ✓ while the voice strip is open [R021] (see
  [composer-voice-input.md](composer-voice-input.md)).

## 4. Timing (defaults; every variant may override)
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

## 5. Variants

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

## 6. Edge cases
- **Long text / huge pastes.** The button never grows with the draft; it stays bottom-trailing as the
  input grows upward. A paste over the app's size limit puts the button in `Disabled("Message too
  long")` with a count, rather than failing on send.
- **Draft that is only whitespace or only attachments.** Whitespace = `Empty`. Attachments without
  text = `Ready` (sending files alone is valid) unless an attachment is still uploading, in which case
  `Disabled("Waiting for upload")` with the attachment ring as the progress cue [R036].
- **Very fast agent.** If the first event and `Done` both arrive within one frame of `Sending`, skip
  the ■ entirely (no flash of stop).
- **Very slow agent.** ■ stays; the optional ring shows elapsed; stall colour belongs to the status
  strip, not the button.
- **Double-submit.** Enter pressed twice within 300 ms while `Sending` is ignored, not queued.
- **IME composition.** Enter that commits an IME candidate never sends or queues (check composition
  state first).
- **RTL.** The button sits bottom-left; ↑ is direction-neutral; split ↑ ▾ mirrors to ▾ ↑; the queue
  pill's label grows toward the inline-start.
- **Tiny sizes (16–20 px).** Glyph only; queue pill collapses to ■ + a 6 px badge dot; split button
  becomes a long-press menu.
- **Dense IDE panels.** `send-control/minimal` text labels; hit area stays ≥ 24 px.
- **Touch.** 44 px hit area; `long-press-options` replaces the ▾; Stop with `StopConfirm::Hold`
  uses 400 ms.
- **Slow networks / offline.** `Sending` may last; the status strip's stall thresholds apply
  ([composer-status-strip.md](composer-status-strip.md)), and the button stays ■ so the user can cancel. Offline → `Disabled("Offline")`, but
  Enter still queues locally if the queue stack is enabled (proposed).
- **Pending card.** While an approval/question card waits, ↑ answers the card [R088]; a normal
  message is still possible via the queue.

## 7. Accessibility
- **Labels per state:** Empty "Send message (type a message first)"; Ready "Send message";
  Sending/Working "Stop generating"; WorkingWithDraft "Queue message" (and "Steer now" on the steer
  button); Answering "Submit answer"; Dictating "Insert dictated text"; Disabled "Send unavailable:
  <reason>"; Error "Retry sending".
- **Live region (polite):** "Message sent", "Message queued, 2 in queue", "Steering the agent",
  "Generation stopped". Errors are **assertive**: "Message failed to send. Retry available".
- **Keyboard:** all actions reachable without a pointer (Enter, Shift+Enter, ⌘/Ctrl+Enter, Esc, ↑);
  the footer hint ("Enter to queue", [R089]) is also in the button's description.
- **Contrast:** the muted `Empty` ↑ still meets 3:1 (non-text contrast); accent and danger glyphs meet
  3:1 against the button fill and the fill meets 3:1 against the composer.
- **Focus order:** input → tools / pickers → mic → send control (→ steer button when present). Focus
  stays on the input after send; after Stop focus moves back to the input.
- **Reduced motion:** see the timing table; state is always also carried by the glyph and label.

## 8. Props (sketch)
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

## 9. Related components
Queued-message stack · steer note · status tab · time-budget chip · voice strip · question card ·
approval chip ("Request approval" / "Bypass all") · attachment tiles with ring progress.

## 10. References (study only unless MIT/Apache)
Phil Pitus composer + voice ✓ and thinking timeline · Cloudy steer-in-place + pause/stop + budget chip ·
BoardUI attachment ring and "Bypass all" · affectUI steering (Enter queues, ⌘↵ interrupts) · T3 Code
"Enter to queue" · Hermes `busyAction` modes · Amp second-Enter steer · Glaze / Replit queued chips ·
fal.ai two lanes · Renzo Bianchi on composer exit animations · coss ui Composer API (MIT).

### Source IDs (from [../research/sources.csv](../research/sources.csv))
| Reference | IDs | Licence note |
|---|---|---|
| Phil Pitus composer + voice ✓ and thinking timeline | [R021] | Study only |
| Cloudy steer-in-place + pause/stop + budget chip | [R007] | Study only |
| BoardUI attachment ring and "Bypass all" | [R036], [R095] | Study only (free tier MIT; Pro paid) |
| affectUI steering (Enter queues, ⌘↵ interrupts); affectUI composer (muted ↑ lights on first key, "Request approval") | [R080], [R044] | Paid — study only |
| T3 Code "Enter to queue" | [R089], shell [R003] | Check source; study only |
| Hermes `busyAction` modes | [R088], [R076], [R075] | Check source; study only |
| Amp second-Enter steer | [R077] | Check source; study only |
| Glaze / Replit queued chips | [R083], [R078] | Check source; study only |
| fal.ai two lanes, "Up next", Esc×2 | [R084] | Check source; study only |
| Renzo Bianchi on composer exit animations | [R079] | Check source; study only |
| coss ui Composer API | [R126] | MIT — may port with attribution |
| openwork busy shake, "Esc again to stop" | [R085] | Check source; study only |
| os-clovy instant Send→Stop, steer blur-slide | [R086] | Check source; study only |
| gaia Queue pill widen | [R087] | Check source; study only |
| AI Patterns "Queue ⌘↵"; TUFFUI Ask/Plan/Act | [R082], [R081] | Check source; study only |
| Devin concept split ↑ ⌄ | [R033] | Study only |
| Aceternity stateful send button | [R122] | Study only (paid tier) |
