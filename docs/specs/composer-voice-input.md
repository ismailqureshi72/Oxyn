**Status:** draft v0.1 (2026-11-11).

# Spec: composer voice input (mic · push-to-talk · dictation strip · voice-mode entry)

Part of the composer family ([composer.md](composer.md)). Same depth
as the worked example [composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro
variants**, all driven by one `VoiceState` machine that is derived from `ComposerState`.

## 1. Purpose
The user wants to talk instead of type: dictate a long prompt, think out loud, or answer an agent's
question with their hands busy. Two different intents share the composer and must look different
(research note 04 found most chat apps ship them side by side: a **mic for dictation** and a
**waveform button for live voice**):

- **Dictation** — speech becomes *draft text*. The user reviews it, then sends with the normal send
  control. This is the part this spec owns in full.
- **Voice mode** — a live spoken conversation with the agent (listening / thinking / speaking /
  interrupted). This spec owns only the **entry** into it; the session itself is the future
  `voice-session` component.

The reference behaviour is the Pitus composer [R021]: pressing the mic turns the composer into a
**waveform strip with a moving playhead and a live transcript above it**, and the send ↑ becomes a
**✓ confirm**. Mic placement beside ↑ is near-universal in the composers we studied
[R007][R021][R033][R036][R044]. A "Listening" pill with a breathing dot [R020] and the record-button
states `idle · recording · processing · success · error` (ElevenLabs UI, note 04) complete the picture.

Rules:
1. The composer **never moves** (same rule as the send control). The strip replaces the input row in
   place, at the same height; the transcript grows **upward**.
2. Dictated text is **never sent without a confirm** unless the app opts into `InsertAndSend`.
3. Cancel is always one action away (✕, Esc, or releasing outside the button in push-to-talk) and it
   **discards audio and transcript** — nothing is kept or uploaded after cancel.
4. Audio level and capture are always visible while the mic is open (a privacy rule, not decoration):
   there is no recording state without a live level indicator and a red-or-accent "mic on" mark.
5. Voice confirm puts text into the draft; from there the send control's `busy_action` decides
   queue / steer / interrupt if the agent is working. Voice never bypasses the queue.

## 2. State machine

```
 Idle (mic) ──────────────── voice-mode button ───────────────▶ VoiceModeEntry ──▶ voice-session
   │ click · hotkey · hold (PTT)
   ▼
 Requesting ──── denied / dismissed ────▶ PermissionDenied ── retry / settings ──┐
   │ granted (or already granted)                                              │
   ▼                                                                           │
 Arming (open device, warm-up) ◀────────────────────────────────────────────────┘
   │ stream open            └── no device / device busy ──▶ DeviceError
   ▼
 Listening (no speech yet) ── 8 s silence ──▶ NoInput ── retry ──▶ Listening
   │ level > speech threshold                    └── ✕ ──▶ Cancelled
   ▼
 Capturing (waveform + live transcript) ⇄ Switching(lang)   (language chip; segment restarts)
   │ ✓ / Enter / PTT release / max duration
   ▼
 Finalizing (last pass on the tail) ── fails / 15 s timeout ──▶ TranscriptionError ── retry ──┐
   │ final text                                     └── keep text so far / discard          │
   ▼                                                                                        │
 Confirmed ──▶ text lands in draft (InsertIntoDraft) or is sent (InsertAndSend) ──▶ Idle   ◀─┘
 ✕ / Esc from Arming · Listening · Capturing · Finalizing ──▶ Cancelled ──▶ Idle
 Device unplugged / OS revokes permission mid-capture ──▶ DeviceError (transcript so far kept as draft)
 Special: Disabled(reason) — offline with server STT, quota, app policy, agent speaking in voice mode
```

`PushToTalk` is not a separate state: it is `Capturing` with `held = true`. Releasing the key goes to
`Finalizing` (the confirm is the release); dragging/moving the pointer off the button before release
goes to `Cancelled` (proposed, mirrors iOS voice-memo "slide to cancel").

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle` | mic glyph, muted; optional voice-mode waveform glyph beside it | start dictation | voice mode (separate button) | dictation hotkey (app-set, proposed `⌘/Ctrl+Shift+Space`), Tab reaches mic |
| `Requesting` | mic in accent, hint "Allow microphone access" under the composer | — (OS dialog owns focus) | cancel ✕ | Esc cancels |
| `PermissionDenied` | slashed mic, danger-muted; strip text "Microphone blocked" + **How to allow** | retry | open settings (desktop deep-link, proposed) / dismiss | Enter retry, Esc dismiss |
| `Arming` | strip appears, flat waveform, dot pulsing; "Starting mic…" only if > 300 ms | — | cancel ✕ | Esc cancel |
| `Listening` | flat waveform with live level, breathing dot, placeholder "Listening…", timer 0:00 | confirm ✓ (disabled until text exists) | cancel ✕, language chip | Esc cancel |
| `Capturing` | waveform + playhead + timer; live transcript above (partial words at 60 % opacity) | confirm ✓ (send control shows ✓) | cancel ✕, language chip, pause (variant) | Enter confirm, Esc cancel, Space toggles pause (variant only) |
| `Capturing{held}` (PTT) | as Capturing + "Release to insert · move away to cancel" hint | release = confirm | move off = cancel | key-up confirms |
| `NoInput` | strip dimmed 50 %, "Didn't catch anything" + **Try again** | retry | cancel ✕, check mic (opens device menu) | Enter retry, Esc cancel |
| `Switching(lang)` | language chip shows new code with a 1-frame flash; transcript keeps prior text | — | — | — |
| `Finalizing` | waveform frozen, playhead stops, "Transcribing…" shimmer on the tail words | — | cancel ✕ | Esc cancel |
| `TranscriptionError` | tail in danger tone, "Couldn't transcribe · Retry"; kept text stays editable | retry (re-sends kept audio) | keep text / discard | Enter retry, Esc discard |
| `DeviceError` | slashed mic, "No microphone found" or "Mic in use by another app" | choose device | dismiss | Enter opens device menu |
| `Confirmed` | transcript flies down into the input; strip collapses; ✓ morphs back to ↑ | — | — | focus returns to input, caret at end |
| `Cancelled` | strip shrinks into the mic; nothing inserted | — | — | focus returns to mic |
| `VoiceModeEntry` | composer morphs into the voice-session bar (owned by that component) | — | exit | Esc exits voice mode |
| `Disabled(reason)` | mic muted + tooltip reason | — | — | Tab still reaches it; tooltip on focus |

## 3. Micro-interactions
- **Mic hover / press.** Hover tints the 32 px hit area (icon 18 px). Press scales the glyph 0.94.
  A press longer than **200 ms** (`hold_threshold`, proposed) is push-to-talk; shorter is a toggle.
- **Composer → strip morph.** The input text crossfades out while the strip fades and scales in from
  the mic's x-position (transform-origin at the mic). The send ↑ morphs to ✓ in the same frame,
  reusing the send control's glyph morph. Draft text that existed before dictation is **kept**: the
  transcript is inserted at the caret on confirm, never replacing the draft.
- **Waveform.** Bars 2 px wide, 2 px gap, 2–24 px tall, rounded caps. New bars append at the
  **playhead** (a 1 px accent line at the write head). When the strip is full, history scrolls left
  at a constant rate so the playhead stays at 85 % of the strip width. Bar height = smoothed RMS
  level mapped from −60…0 dBFS (attack 30 ms, release 150 ms, proposed).
- **Level / noise meter.** The breathing dot doubles as a level meter (scale 1.0–1.35 with level).
  Two hints, each shown once per session (proposed thresholds): **Too quiet** — speech-band level
  stays under −45 dBFS for 3 s while Capturing; **Noisy room** — noise floor above −35 dBFS for 2 s
  in Listening. **Clipping** (> −1 dBFS) paints the top 2 px of the bar in warning tone.
- **Live transcript.** Region above the strip, 14 px text, 1–3 lines visible (grows upward, then
  scrolls with a top fade mask 16 px). Partial words blur-in (4 px → 0) at 60 % opacity; when the
  recogniser finalises them they settle to 100 % without moving. Words never reflow backwards once
  final; partials may be replaced in place.
- **Timer.** `0:07` at the strip's leading edge, tabular figures; at **4:30** of a 5:00 cap it turns
  warning tone and shows "30 s left"; at the cap it auto-confirms (goes to `Finalizing`).
- **Confirm.** Final transcript translates down into the input baseline and becomes ordinary draft
  text; the strip collapses; ✓ morphs back to ↑ (or to the busy-state control). Optional Cuelume-style
  success tick sound [R110] (off by default).
- **Cancel.** Strip shrinks back into the mic; transcript fades; no shake (cancel is not an error).
- **No-input.** Waveform dims, hint slides up 8 px. The mic stays open for **2 s** more in case speech
  starts (then closes), so "Try again" rarely needs a second permission round-trip.
- **Language switch.** A 2-letter chip (`EN`) at the strip's trailing edge opens the language menu
  (recent 3 on top, then Auto-detect, then all). Switching mid-capture **closes the current segment
  and opens a new one**; text so far stays. RTL languages flip the transcript region's direction
  per segment.
- **Permission denied.** Mic does one 2-cycle 4 px shake, then shows the slash. "How to allow" opens
  platform-specific steps (browser site settings vs. macOS Privacy & Security, proposed copy).
- **Voice-mode entry.** Waveform-bars button beside the mic. Press → the whole composer row morphs
  into the voice-session bar (height grows by 8 px max, upward). Exit returns the draft untouched.

## 4. Timing (defaults; every variant may override)
Easing names refer to the composer motion tokens in [composer.md](composer.md#motion-tokens).

| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Mic hover tint | 100 ms | `ease.out` | |
| Mic press scale 1 → 0.94 → 1 | 80 ms down, 140 ms up | `ease.out` / `spring.snap` | |
| Tap vs hold decision | 200 ms | — | `hold_threshold`, proposed |
| Input fade-out → strip in | 120 ms out, 180 ms in, 40 ms overlap | `ease.out` | strip scales 0.96 → 1 from mic origin |
| ↑ → ✓ glyph morph | 160 ms | `spring.snap` (critically damped) | same as send control |
| "Starting mic…" text appears | delay 300 ms, fade 120 ms | `ease.out` | only if Arming is slow |
| Waveform frame rate | 30 fps (60 fps in `expressive`) | — | shared motion clock |
| Level smoothing | attack 30 ms, release 150 ms | exponential | proposed |
| Breathing dot (Listening) | 1600 ms loop | sine | scales with level while Capturing |
| Partial word blur-in | 140 ms, 30 ms stagger (cap 6 words) | `ease.out` | blur 4 → 0 px, opacity 0 → 0.6 |
| Partial → final settle | 120 ms | `ease.out` | opacity 0.6 → 1, no movement |
| No-input timeout | 8 s silence in Listening | — | proposed; mic closes 2 s after hint |
| No-input hint slide | 160 ms | `ease.out` | y +8 → 0 |
| Max dictation / warning | 5:00 cap, warn at 4:30 | — | proposed, app-configurable |
| Finalizing timeout | 15 s | — | then `TranscriptionError` |
| Confirm: transcript → input | 200 ms | `ease.in-out` | strip collapse 160 ms, starts at 80 ms |
| Cancel: strip → mic | 140 ms | `ease.in` | transcript fade 100 ms |
| Permission-denied shake | 240 ms, 2 cycles, 4 px | linear | |
| Language chip flash | 1 frame accent + 200 ms fade | `ease.out` | |
| Voice-mode morph | 240 ms | `spring.soft` | owned jointly with voice-session |
| **Reduced motion** | all morphs become 80 ms opacity crossfades; waveform replaced by a **single horizontal level bar** updated at 10 fps (level is information, so it stays); no blur-in, no shake (slash + text only); breathing dot static | — | — |

## 5. Variants

### Free (6)
| Name | Idea | Surfaces |
|---|---|---|
| `voice-input/strip` | The baseline: waveform strip + playhead + transcript above, ✓ / ✕ [R021] | chat, desktop, ide |
| `voice-input/pill` | Compact "● Listening 0:07" pill with breathing dot replaces only the mic slot; transcript streams straight into the input [R020] | chat, mobile |
| `voice-input/push-to-talk` | Hold mic or PTT key; release inserts, slide away cancels; level ring around the mic | desktop, mobile |
| `voice-input/inline-caret` | No strip: dictated words stream into the draft at the caret in 60 % ink, settling on final (Arc / assistant-ui style "dictation streams into the composer", note 01) | ide, chat |
| `voice-input/minimal` | Text-only `Dictate` / `Listening… Esc to cancel` for dense IDE panels | ide |
| `voice-input/glyph` | Terminal look: `● REC 0:07 ▁▂▅▇▅▂` braille/block meter, Enter accept, Esc cancel | terminal |

### Pro (14)
`orb` (state orb listens and swells with level; thinking-orbs family), `voice-ring` (mic ring of
radial bars), `scrubbable` (after stop, scrub the recording and re-transcribe a span), `karaoke`
(word highlight synced to audio on review), `dot-matrix` (level as a dot-matrix display), `liquid`
(goo waveform), `aurora-edge` (composer border glows with level), `notch` (desktop island shows the
level while the app is in the background), `two-button` (dictation mic + voice-mode bars with a
shared morph), `walkie` (full-width hold bar on mobile), `particle` (dots gather into the transcript,
matches the Oxyn mark), `confidence-ink` (low-confidence words underlined dotted, tap to fix),
`multilang-tabs` (one transcript lane per detected language), `spectrogram` (scrolling spectrogram
for audio/video agent apps).

## 6. Edge cases
- **Long dictation.** Transcript region caps at 3 visible lines then scrolls; on confirm the draft may
  exceed the input's max rows, so the input-shell's own overflow applies. 5-min cap prevents runaway.
- **Existing draft.** Insert at caret with a single leading space if the char before the caret isn't
  whitespace; never overwrite a selection silently (selection is replaced only when the user started
  dictation with text selected — proposed "dictate to replace").
- **Agent busy.** Dictation works during `Thinking/Streaming/ToolRunning`. Confirm inserts into the
  draft; the send control is then in `WorkingWithDraft` and its `busy_action` applies. With
  `InsertAndSend`, the text goes to the queue as a normal queued message.
- **Awaiting approval / question card.** Confirmed text answers the open card (send control
  `Answering`). Saying "approve" is **not** treated as approval (proposed: no voice commands for
  consequential actions in v1).
- **Voice mode while agent speaks.** Mic is `Disabled("Agent is speaking — tap to interrupt")` unless
  the session supports barge-in; then the interrupt belongs to voice-session.
- **RTL / mixed scripts.** Strip layout mirrors (timer trailing, chip leading). Transcript direction
  is per segment; bidi isolation around each segment.
- **Tiny sizes.** At 16–20 px the mic is glyph-only; the strip needs ≥ 160 px width, below that the
  variant falls back to `voice-input/pill`. In a 28 px IDE row the waveform drops to 1 px bars,
  max 14 px tall.
- **Touch.** 44 × 44 px minimum hit area. PTT on touch: hold ≥ 200 ms; slide up 48 px or off-target
  to cancel; haptic tick on start and on cancel (mobile, proposed).
- **Slow network (server STT).** Partials may lag: show the waveform immediately and keep a
  "Transcribing…" shimmer at the tail; never block capture on the network. Offline with server-only
  STT → `Disabled("Offline — dictation needs a connection")`; on-device STT keeps working.
- **Very fast recogniser / very short utterance.** Under 400 ms of speech with no text → treat as
  NoInput, not error.
- **Device changes.** Unplugging the active mic mid-capture → `DeviceError`, transcript so far kept as
  draft. A new default device is not auto-adopted mid-capture (avoids recording the wrong room).
- **Window loses focus.** Capture continues (it's a dictation, not a hover); the `notch` Pro variant
  or OS mic indicator is the reminder. Toggle mode keeps recording; PTT with a key ends on key-up
  even if focus moved (proposed).
- **Hotkey collisions.** Every global candidate collides with some OS or IME shortcut (e.g. input
  switching), so Oxyn ships **no global PTT default**; apps set one. The in-composer hotkey is
  configurable and shown in the mic tooltip.

## 7. Accessibility
- **Labels per state:** Idle "Start dictation"; Recording "Stop dictation and insert text" (✓) and
  "Cancel dictation" (✕); PTT "Hold to dictate"; Denied "Microphone blocked. Activate to see how to
  allow it"; Disabled "Dictation unavailable: <reason>"; voice-mode "Start voice conversation".
- **Live region (polite):** "Listening", "Didn't catch anything", "Transcribing", "Inserted 24
  words", "Dictation cancelled", "Language: Spanish". **Assertive** only for permission denied and
  device errors. Partial transcript text is **not** announced word by word (it would talk over the
  user); the final text is readable in the input after confirm.
- **Recording indicator** has a text equivalent ("Recording, 0:07") exposed on the strip, and the
  `aria-pressed`/toggled state on the mic.
- **Contrast:** waveform bars ≥ 3:1 against the strip background; partial words at 60 % opacity must
  still meet 4.5:1 (darken the base ink token if needed); danger and warning tones carry an icon,
  never colour alone.
- **Focus order:** mic → (strip appears) language chip → ✕ → ✓. Focus moves to ✓ on start only for
  pointer starts; keyboard starts keep focus in the input so Enter/Esc work. On end, focus returns to
  where dictation began.
- **Reduced motion:** see timing table; level bar always present.
- **Screen-reader users and dictation** often already use OS dictation; the component must not
  steal OS dictation hotkeys.

## 8. Props (sketch)
```rust
pub struct VoiceInput {
    pub state: VoiceState,              // derived from ComposerState + VoiceSession events
    pub variant: VoiceVariant,          // Strip | Pill | PushToTalk | InlineCaret | Minimal | Glyph | Pro(..)
    pub trigger: VoiceTrigger,          // Toggle | PushToTalk { hold_threshold: Duration } | Both
    pub hotkey: Option<Keystroke>,      // in-composer toggle; None = no hotkey
    pub ptt_key: Option<Keystroke>,     // no default (see edge cases)
    pub on_confirm: ConfirmAction,      // InsertIntoDraft (default) | InsertAndSend
    pub no_input_timeout: Duration,     // 8 s
    pub max_duration: Duration,         // 5 min, warning at max - 30 s
    pub finalize_timeout: Duration,     // 15 s
    pub language: LanguageSetting,      // Auto | Fixed(LangTag), plus recent: Vec<LangTag>
    pub level: LevelSignal,             // shared 0.0–1.0 smoothed level from the audio adapter
    pub show_voice_mode: bool,          // second button for live voice
    pub sounds: Option<SoundSet>,       // start / stop / error cues, off by default
    pub labels: VoiceLabels,            // i18n + screen-reader text per state
    pub motion: MotionLevel,            // None | Subtle | Expressive
}

pub enum VoiceState {
    Idle, Requesting, PermissionDenied { can_retry: bool },
    Arming, Listening { since: Instant }, NoInput,
    Capturing { elapsed: Duration, held: bool, partial: String, finals: Vec<Segment> },
    Switching { to: LangTag }, Finalizing,
    TranscriptionError { kind: SttError, audio_kept: bool },
    DeviceError { kind: DeviceErrorKind }, Confirmed, Cancelled,
    VoiceModeEntry, Disabled(DisabledReason),
}
```
`AgentState` drives it through `ComposerState`: `AwaitingApproval` routes confirm to the open card;
any busy state routes confirm through the send control's `busy_action`; `Error { kind: Offline }`
disables server STT. The STT engine is an adapter (`trait Transcriber`), not part of the component.

## 9. Related components
Send control (✓ confirm, [composer-send-control.md](composer-send-control.md)) · input shell
([composer-input-shell.md](composer-input-shell.md)) · queue stack (InsertAndSend while busy,
[composer-queue-stack.md](composer-queue-stack.md)) · status strip (offline banner,
[composer-status-strip.md](composer-status-strip.md)) · voice-session bar and voice orb (future) ·
device picker (future) · question / approval card.

## 10. References (study only unless MIT/Apache)
| Ref | What we take | Licence note |
|---|---|---|
| Phil Pitus composer [R021] | waveform strip + playhead + transcript above, ↑ → ✓ confirm | Study only |
| DopeOblivion voice hero [R020] | "Listening" pill with breathing dot; blur-in lines | Study only |
| Cloudy [R007], Devin concept [R033], BoardUI [R036], affectUI composer [R044] | mic beside send; Devin's separate voice-mode bars icon | Study only (affectUI paid) |
| Cuelume [R110] | optional start/stop/error sound cues | MIT per licensing.md |
| Libraries.dev Voice / Voice ring [R100] | Pro `voice-ring`, `orb` ideas | Packages MIT |
| Astryx Chat Dictation Button [R103] | dictation as its own composer part | Open source (Meta), licence type to check |
| SPY UI voice visualiser [R136] | orb states idle / listening / thinking / speaking | Study only |
| ElevenLabs UI, LiveKit Agents UI, OpenAI Realtime console, Gemini Live (research note 04) | voice-button states; visualisers driven by state + level; VAD vs push-to-talk toggle; mic + voice as two affordances | Not in sources.csv; licences not checked — study only |
| Thinking orbs [R068] | Pro `orb` | Post is study only; the thinking-orbs package is MIT per licensing.md |
