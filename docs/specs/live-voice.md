**Status:** draft v0.1 (2026-11-12).

# Spec: live & voice — family K overview (`voice-session`, `voice-latency`)

Family K in [../component-taxonomy.md](../component-taxonomy.md): orb states, visualisers, control bar,
barge-in markers, captions, word-synced transcript, mini voice window and screen-share overlay. This
overview owns the **shared contracts** every family-K part reads, plus two components of its own:

- **`voice-session`** — the container of a live spoken conversation (full call screen, in-thread voice,
  side panel). It is the component that [composer-voice-input.md](composer-voice-input.md) hands over to
  at `VoiceModeEntry`.
- **`voice-latency`** — a user-facing display of the *response gap* (user stops talking → agent is
  audible), split into measured segments.

The shared contracts are: the **voice session state machine** (§2), the **audio-level data contract**
(§2.3), the **latency model** (§2.4) and the family **motion and timing tokens** (§5). The parts:

| Spec | Components | Free | Pro |
|---|---|---|---|
| live-voice.md (this file) | `voice-session`, `voice-latency` | 6 | 10 |
| [voice-orb.md](voice-orb.md) | `voice-orb` | 4 | 16 |
| [voice-visualisers.md](voice-visualisers.md) | `voice-viz` | 5 | 14 |
| [voice-control-bar.md](voice-control-bar.md) | `voice-controls`, `voice-mini`, `share-overlay` | 7 | 15 |
| [live-captions.md](live-captions.md) | `captions`, `voice-transcript` | 5 | 14 |
| **Family K total** | 10 components | **27** | **69** = **96 variants** (target was 28+) |

**How much is ours.** Research on live voice UI is thin. Notes 04, 15, 16 and 17 give us *which* parts
exist and *which* states they name; they give almost no timings, thresholds or sizes. Every number in
family K is an Oxyn **proposal** unless a reference sits next to it, and the ones that matter most
(VAD thresholds, barge-in minimum, latency bands, sleep timeout) are flagged for tuning in the Studio
against real sessions before v1.

## 1. Purpose
A user is *talking* with an agent: hands busy, eyes elsewhere, or simply faster by voice. Unlike a chat
turn, the conversation runs in real time, so the UI has four jobs that text UIs do not:

1. **Say whose turn it is** — the agent is listening, hearing you, thinking, or speaking — at a glance
   and without reading. Research note 04 lists the states the kits name: LiveKit and ElevenLabs use
   connecting, initializing, listening, thinking, speaking (+ idle); Gemini Live adds Hold / resume; the
   realtime demos show interruption. Note 04 also records the gap: no kit has **one** state model
   that covers text *and* voice; Oxyn's `VoiceSessionState` is derived from the same `AgentState`.
2. **Prove the mic is honest** — whether audio is being captured, whether it is being *sent*, and
   whether it is muted (rule 4 of composer-voice-input.md, extended to sessions).
3. **Make interruption feel safe** — when the user talks over the agent (barge-in), the agent must
   *visibly* stop, and the transcript must show where it was cut (note 04 gap 11: "visual
   confirmation that agent stopped" exists only partly in products and not in kits).
4. **Keep voice and text continuous** — ChatGPT voice moved into the chat thread (note 04, Nov 2025),
   and Gemini Live has a floating pill with transcript text above a waveform and a captions toggle
   (note 04). A voice turn is a message with audio, transcript and an interrupted marker.

Rules (family-wide, proposed):
1. **One state machine** (§2), derived in `oxyn-core`; parts never keep private session state or timers
   (they read the shared motion clock, [../architecture.md](../architecture.md)).
2. **State is never carried by motion, colour or sound alone.** Every state has a text label available
   (orb caption, control-bar status, captions header) and a still form.
3. **Mic capture is always visible.** While audio is captured, at least one visible element shows a
   live level and a "mic on" mark. While muted, a slashed-mic mark is visible.
4. **Barge-in is acknowledged within one frame** of the stop event and resolved (agent audibly stopped)
   within `barge_in_ack_max` (300 ms, proposed) or the UI says "Stopping…".
5. **Typing is always possible.** A voice session never removes the text input; deaf, hard-of-hearing
   and speech-impaired users, and anyone in a quiet room, can type into the same turn stream.
6. **No audio samples in UI data.** Components receive levels, bands and words, never PCM.
7. **Consequential actions are never approved by voice in v1** (same rule as composer-voice-input.md);
   an approval during a call is shown as a card and answered by touch, click or keyboard.

## 2. State machine

### 2.1 Session states
```
                  start (voice-mode button · hotkey · composer VoiceModeEntry)
  Idle ─────────────────────────────────────────────────────────────▶ Connecting{Mic → Transport → Agent}
   ▲                                                                       │  mic denied / no device / transport fail
   │                                                                       ├──────────────────────────────▶ Error{kind}
   │                                                                       │  agent ready, audio output unlocked
   │                                                                       ▼
   │                    ┌──────────────────────────────────────────▶ Listening ◀───────────── no end-of-turn
   │                    │                                              │    ▲       (speech dropped < hangover)
   │                    │              vad ≥ 0.5 for ≥ 60 ms (or PTT down)│    │
   │                    │                                              ▼    │
   │                    │                                         UserSpeaking ───────┘
   │                    │                                              │ end-of-turn (VAD hangover / PTT up / server)
   │                    │                                              ▼
   │                    │                    Thinking{Reasoning | Tool(name) | Handoff(to) | NeedsApproval}
   │                    │                                              │ first audible agent frame
   │      playout ends  │                                              ▼
   │      (+ 200 ms)    └─────────────────────────────────────── AgentSpeaking{utterance}
   │                                                                   │ user speech ≥ barge_in_min (250 ms)
   │                                                                   │ or PTT down or "Stop" button
   │                                                                   ▼
   │                                                       Interrupted{at_word, cause} ──▶ UserSpeaking
   │                                                           │ false barge-in (no words in 700 ms) and app allows resume
   │                                                           └──────────────────────────▶ AgentSpeaking (resumes)
   │
   ├── End (button · hotkey · agent ends) from any state ──▶ Ended{reason, duration} ── dismiss / 8 s ──▶ Idle
   │
  Overlays (orthogonal, combine with the live states above):
   Muted{Soft | Hard}  user mutes; Listening/UserSpeaking show slashed mic; agent may still speak
   Asleep              Listening with no speech either side for asleep_after (60 s) → mic closes (or wake-word mode); tap/hotkey wakes → Listening
   Reconnecting{n}     transport drops → capture paused, playout stopped; backoff 0.5·1·2·4·8 s; ok → previous live state (AgentSpeaking resumes as Listening); 5 fails → Error{Transport}
   OnHold              app/user holds the session (Gemini Live "Hold", note 04): mic closed, agent silent, timer paused
   Sharing / Captions / Mini   presentation overlays owned by voice-control-bar.md and live-captions.md
```

Derivation from `AgentState` (pure function in `oxyn-core`, proposed):

| `AgentState` + audio events | `VoiceSessionState` |
|---|---|
| no session | `Idle` |
| transport/agent not ready | `Connecting{step}` |
| `Idle` / `Done` and no speech | `Listening` (or `Asleep` after timeout) |
| local VAD ≥ threshold, or PTT held | `UserSpeaking` |
| `Submitted`, `Thinking`, `Streaming` before first audible frame | `Thinking{Reasoning}` |
| `ToolRunning { tool }` | `Thinking{Tool(tool)}` |
| handoff event from adapter (e.g. realtime-agents handoff rows, note 04) | `Thinking{Handoff(to)}` |
| `AwaitingApproval` | `Thinking{NeedsApproval}` — agent silent, card shown, amber |
| agent audio playing | `AgentSpeaking` (even if `AgentState::Streaming` continues) |
| `Steering` / `Cancelled` caused by user speech or Stop | `Interrupted` |
| `Stalled { since }` while `Thinking` | `Thinking` + `stalled = true` ("Still thinking…") |
| `Error { kind }` | `Error{Agent(kind)}` |

### 2.2 States table
| State | Shows (all parts) | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle` | voice-mode button; nothing else | Start voice | device picker | app hotkey (no default, as composer-voice-input.md) |
| `Connecting{Mic}` | orb contracts; status "Allow microphone"; OS dialog owns focus | — | Cancel | Esc cancels |
| `Connecting{Transport/Agent}` | orb pre-connect motion; status "Connecting…" after 300 ms; optional pre-connect prompt "Start talking — I'll hear you when connected" (idea from LiveKit's pre-connect shimmer prompt, note 04) | — | Cancel | Esc cancels |
| `Listening` | orb idle-listening; viz shows mic floor; status "Listening" | Mute | End, captions, share, PTT | `M` mute, `Esc` end (confirm-free), `C` captions, Space PTT (PTT mode) |
| `UserSpeaking` | orb leans inward with mic level; viz mic; interim captions under "You" | Mute | End | as Listening |
| `Thinking{Reasoning}` | orb thinking motion (thinking-orb energy 0.45); viz thinking ripple; status "Thinking"; after `filler_after` "Still thinking…" | Stop | End | `S` / Esc-Esc stop answer |
| `Thinking{Tool(t)}` | as above + tool chip "Searching the web" (verb pack) | Stop | open tool row | `S` |
| `Thinking{Handoff(to)}` | status "Handing to Billing agent"; avatar/label swaps on speak | Stop | — | `S` |
| `Thinking{NeedsApproval}` | amber orb hold; approval card ([approval-card.md](approval-card.md)) slides above control bar; status "Needs your approval" | Review | End | focus moves to card |
| `AgentSpeaking` | orb outward with agent level; viz agent; captions word-synced | Interrupt (tap orb / Stop) | Mute, End | Space (PTT) or `S` interrupts |
| `Interrupted` | flow reverses (160 ms); captions show `↯` cut marker; status "You interrupted" 1.2 s | — | Resume answer (if app allows) | `R` resume |
| `Muted{Hard}` (overlay) | slashed mic on control bar + orb badge; viz flat line; status "Muted" | Unmute | — | `M` |
| `Muted{Soft}` (overlay) | as Hard + "You're muted" hint if speech detected locally for 1 s (§2.3) | Unmute | dismiss hint | `M` |
| `Asleep` | orb dim, 6 s breathe; viz off; status "Paused — tap to talk" | Wake | End | Space / Enter wakes |
| `OnHold` | orb still poster; status "On hold · 0:42"; captions paused | Resume | End | Enter resumes |
| `Reconnecting{n}` | orb desaturates; status "Reconnecting… (2)" with countdown; captions separator | — | End | — |
| `Error{kind}` | slashed or danger orb still; message per kind (table §7) | Retry / Fix | End | Enter retry |
| `Ended{reason}` | session collapses into a voice-turn summary row: duration, turns, "Interrupted 2×" | Start again | View transcript | Enter start again |

### 2.3 Audio-level data contract (`oxyn-core::audio`)
Every visual in family K reads the same frames. The audio engine (an adapter: WebRTC, LiveKit, a
native capture crate, or a server stream) publishes **analysis frames only**.

```rust
/// One analysis frame. Published at 30–60 Hz per source. No PCM, ever (rule 6).
pub struct AudioFrame {
    pub t_ms: u64,              // MotionClock time base (monotonic, ms)
    pub source: AudioSource,    // Mic | Agent | Remote(ParticipantId)
    pub rms_dbfs: f32,          // −120.0 ..= 0.0, over the frame window
    pub peak_dbfs: f32,         // sample peak in the window
    pub bands: Bands,           // normalised 0.0..=1.0, log-spaced, len 8 | 16 | 32
    pub vad: Option<f32>,       // speech probability 0..=1 if the engine provides one
    pub clipped: bool,          // any sample ≥ −1 dBFS
    pub playout_offset_ms: i32, // Agent only: how far ahead of the speaker this frame is analysed
}
```

| Field / rule | Value (all proposed unless noted) | Why |
|---|---|---|
| Publish rate | 30 Hz minimum, 60 Hz preferred; consumers interpolate | brief: 30–60 Hz; 30 Hz is enough for bars, 60 Hz for orbs |
| Analysis window | 1024 samples at 48 kHz (≈ 21 ms), Hann, 50 % overlap → ≈ 94 analyses/s, decimated to publish rate by taking the max RMS | covers one publish period without gaps |
| Bands | log-spaced 80 Hz – 8 kHz; 8 bands default, 16 / 32 for wide visualisers | speech energy band; below 80 Hz is mostly rumble |
| Band normalisation | per band: `(dB − floor) / (ceil − floor)`, floor −60 dBFS, ceil −10 dBFS, clamped; then slow AGC (2 s window, max +12 dB gain) | quiet voices still move the bars |
| Level → 0..1 | `level = clamp((rms_dbfs + 60) / 54, 0, 1)` (floor −60, ceil −6), then `level^0.6` perceptual curve | same floor as composer-voice-input.md |
| Smoothing (consumer side) | attack 30 ms, release 150 ms, exponential (same as composer-voice-input.md) | no flicker, fast onsets |
| Noise gate | below −55 dBFS after AGC → treated as 0 for **motion**, but the "mic on" floor stays visible | rule 3 |
| Transport | lock-free single-producer ring (capacity 128 frames ≈ 2 s at 60 Hz) per source; UI samples the latest frame each motion-clock tick | audio thread never blocks on UI |
| Staleness | no frame for 150 ms → `LevelSignal::Stale`; level decays to 0 over 300 ms; a flat-line + "no audio" state after 2 s | device stalls look different from silence |
| Agent sync | agent frames are analysed before playout; consumers delay them by `playout_offset_ms` so the orb moves when the user **hears** the sound, not when it is decoded | lip-sync style alignment |
| VAD | engine `vad` if given; else local energy VAD: speech when level ≥ 0.35 for ≥ 60 ms; hangover 300 ms | display only; **end-of-turn is the adapter's decision**, the UI never decides turns |
| Soft-mute detection | in `Muted{Soft}` the local track stays open but is not sent; speech ≥ 1 s → "You're muted" hint once per 30 s | Hard mute closes the track, so no hint is possible (and the OS mic indicator turns off) |
| Privacy | frames are not logged or persisted by Oxyn; Studio recordings store frames only when the developer opts in | — |

Consumers receive a `LevelSignal` (smoothed scalar + bands + stale flag) per source and a `TurnSource`
selector: `Mic`, `Agent`, `Auto` (mic in Listening / UserSpeaking, agent in AgentSpeaking), `Both`.

### 2.4 Latency model and budget display (`voice-latency`)
Note 04 found latency meters only in developer consoles (Pipecat Console's metrics tab) and calls
user-facing latency feedback a gap. Oxyn's model splits one **response gap** into segments, each
optional because adapters expose different events:

```
user stops speaking                                                       agent audible
      │◀─ endpoint ─▶│◀─ stt ─▶│◀────── llm (to first token) ──────▶│◀─ tts ─▶│◀ playout ▶│
      t0 (VAD end)    t1 end-of-turn  t2 final transcript          t3 first token  t4 first audio  t5 heard
```

| Segment | From → to | Budget (proposed) | Source of timestamps |
|---|---|---|---|
| `endpoint` | last speech frame → end-of-turn decision | 300 ms | local VAD + adapter end-of-turn event |
| `stt` | end-of-turn → final transcript | 150 ms | adapter |
| `llm` | final transcript → first model token | 400 ms | adapter (`AgentState::Streaming` start) |
| `tts` | first token → first audio byte | 150 ms | adapter |
| `playout` | first audio byte → audible (jitter buffer + device output latency) | 100 ms | local audio engine |
| **Total response gap** | t0 → t5 | **≤ 1100 ms good · ≤ 1800 ms ok · > 1800 ms slow · > 3000 ms very slow** | client clock only |

These bands are **our own targets for display**, not measured industry figures. Rules: all times are on
the client's monotonic clock; server timestamps are converted with a clock-offset estimate, and if no
offset is known only the client-measured total is shown. Unmeasured segments are drawn hatched and
labelled "not reported", never estimated. Speech-to-speech models (no separate STT/TTS) report only
`endpoint` and `model` (first audio) segments.

**Filler indicator.** When `Thinking` passes `filler_after` (1200 ms, proposed) the status text becomes
"Still thinking…" with a text shimmer ([status-text.md](status-text.md)); at 3000 ms an optional app
earcon may play (off by default). Note 04 gap 11 names a "still thinking" filler indicator as missing.

## 3. Micro-interactions (session level)
- **Enter voice mode.** From the composer, the input row morphs into the session (240 ms `spring.soft`,
  owned jointly with [composer-voice-input.md](composer-voice-input.md)). In `call` layout the orb grows
  from the voice-mode button's position to the hero slot (320 ms `ease.out`, scale from button size).
- **Connecting.** Orb contracts once (300 ms) and holds a slow pre-connect rotation; the status text
  appears only after 300 ms so fast connects show nothing but the orb.
- **Turn handover.** Listening → UserSpeaking → Thinking → AgentSpeaking are *one continuous motion*:
  energy springs (thinking-orb engine), visualiser source crossfades over 160 ms, the captions'
  speaker label swaps with a 120 ms crossfade. No component remounts on a state change.
- **Barge-in.** The agent level drops to 0 over 80 ms (`ease.firm`, same token as composer
  "Interrupted"), orb flow reverses, captions place `↯` at the last *heard* word and dim the unspoken
  remainder. If playout has not stopped within 300 ms, status shows "Stopping…".
- **Mute.** Mic glyph slashes with a 140 ms stroke draw; the viz mic floor flattens over 120 ms; the orb
  shows a 12 px slashed-mic badge at 4 o'clock.
- **Sleep / wake.** Asleep fades the orb to 40 % brightness over 1200 ms; wake reverses in 240 ms.
- **Reconnecting.** Orb desaturates 400 ms; status counts down to the next attempt at 1 Hz; on success
  one 200 ms re-saturation.
- **End.** Session collapses into the voice-turn summary row in the thread (260 ms `ease.in-out`, the
  `dur.travel` cap); the orb shrinks into the row's leading glyph.
- **Latency chip.** After each agent turn starts, the chip's number rolls to the new total (180 ms digit
  roll, tabular figures); the segment bar re-lays with a 160 ms width tween. A turn in the slow band
  tints the chip `warn` *and* adds a "slow" word.

## 4. Variants of this file's components
### Free (6)
| Name | Idea | Surfaces |
|---|---|---|
| `voice-session/call` | Full-window call screen: hero orb, status line, captions below, control bar at the bottom (the classic separate voice screen, e.g. ChatGPT's legacy blue-orb mode, note 04) | desktop, mobile, voice |
| `voice-session/in-thread` | Voice inside the chat thread: each spoken turn becomes a message with transcript; a slim live row with orb (24 px), viz and controls sits where the composer was (ChatGPT in-chat voice, note 04) | chat, mobile |
| `voice-session/side-panel` | 320–400 px panel in an IDE or browser sidecar: 64 px orb, captions, compact controls | ide, desktop |
| `voice-latency/chip` | `0.9 s` chip after each answer; tooltip shows segments | all |
| `voice-latency/segments` | Stacked horizontal bar per turn, one colour + label per segment, hatched when not reported, budget tick at 1.1 s | desktop, ide |
| `voice-latency/glyph` | Terminal: `⏱ 0.9s ▁▂▅▂` (last 4 turns as block sparkline) | terminal |

### Pro (10)
| Name | Idea |
|---|---|
| `voice-session/hero-orb` | 240 px orb centred, captions as a single floating line, controls appear on pointer move and hide after 3 s |
| `voice-session/split-share` | Left: shared screen with share-overlay; right: orb + transcript (computer-use voice) |
| `voice-session/phone` | Telephony card: dialing → ringing → connected, live transcript, DTMF keypad (Pipecat ships a DTMF keypad; Genspark's Call For Me returns transcript + summary, note 04) |
| `voice-session/console` | Developer layout: transcript left, event log right (▲ client / ▼ server), VAD vs push-to-talk toggle, metrics tab (OpenAI Realtime console and Pipecat Console, note 04) |
| `voice-session/kiosk` | Large-target layout for cars, kitchens and walk-up screens: 88 px controls, captions at 28 px |
| `voice-session/persona` | Persona header with voice picker and per-persona orb palette (voice/persona pickers in ElevenLabs, Sesame, Grok, note 04) |
| `voice-latency/sparkline` | Last 20 turns as a sparkline with the good/ok bands shaded |
| `voice-latency/waterfall` | Per-turn waterfall (one row per segment, start offset + width) for debugging |
| `voice-latency/budget-ring` | Ring that fills during Thinking against the 1.1 s budget, then freezes as the turn's result |
| `voice-latency/turn-table` | Table: turn, total, each segment, interrupted?, exportable CSV |

## 5. Timing (family tokens; parts may override)
Reuses composer motion tokens (`dur.*`, `ease.*`, `spring.*`, [composer.md](composer.md)) and loader
tokens ([loaders.md](loaders.md) §5). Family K adds (all proposed):

| Token / transition | Value | Easing / spring | Notes |
|---|---|---|---|
| `voice.level.attack` / `release` | 30 ms / 150 ms | exponential | consumer smoothing of `LevelSignal` |
| `voice.data.rate` | 30–60 Hz | — | §2.3; rendering at ≤ 60 fps with interpolation |
| `voice.source.crossfade` | 160 ms | `ease.in-out` | mic ↔ agent source in viz and orb |
| `voice.speaker.swap` | 120 ms | `ease.out` | caption label / avatar |
| `voice.vad.on` / `hangover` | 60 ms / 300 ms | — | display-side VAD only |
| `voice.barge.min` | 250 ms of speech | — | below this, a cough is not a barge-in |
| `voice.barge.drop` | 80 ms | `ease.firm` | agent level → 0 |
| `voice.barge.ack_max` | 300 ms | — | then "Stopping…" |
| `voice.false_barge.window` | 700 ms | — | no words → resume allowed |
| `voice.filler.after` | 1200 ms / earcon 3000 ms | — | "Still thinking…" |
| `voice.asleep.after` | 60 s of silence both sides | — | app-configurable; 0 disables |
| `voice.asleep.fade` / `wake` | 1200 ms / 240 ms | `ease.in-out` / `ease.out` | |
| `voice.reconnect.backoff` | 0.5, 1, 2, 4, 8 s (5 tries) | — | then `Error{Transport}` |
| `voice.ended.auto_dismiss` | 8 s | — | summary row stays in thread |
| Enter voice mode | 240 ms (in-thread) / 320 ms (call) | `spring.soft` / `ease.out` | |
| End → summary row | 260 ms | `ease.in-out` | `dur.travel` cap |
| Latency number roll | 180 ms | `ease.out` | tabular figures |
| Mute stroke | 140 ms | `ease.out` | |
| **Reduced motion** | every morph is a 0–80 ms crossfade; orb shows a still poster per state **plus a static level ring** (level is information, it stays, updated at 10 Hz); visualisers become a single level bar at 10 Hz; no reversal, breathe or desaturation animation; latency number swaps without roll | — | composer reduced-motion rule |

## 6. Edge cases
- **Very fast agent.** First audio < 300 ms after end-of-turn: skip the Thinking visual (no flash of a
  thinking state shorter than 200 ms — `show_delay`-style rule from [loaders.md](loaders.md)).
- **Very slow agent.** Filler at 1.2 s; at 10 s "Still working — tap Stop to cancel"; `Stalled` adds the
  loaders' warn drift. The session never auto-ends while the agent works.
- **Full-duplex / backchannel.** Agents that speak while listening ("mm-hm") produce agent frames during
  `UserSpeaking`; these do **not** switch state, they show on `Source::Both` visualisers only (Meta AI
  exposes a full-duplex demo toggle, note 04).
- **Echo.** Without echo cancellation the agent's voice re-enters the mic and triggers false barge-ins;
  the adapter reports `aec: false`, and the session switches to push-to-talk by default with a one-line
  hint "Using headphones? Turn on hands-free".
- **Autoplay blocked (web).** Browsers can refuse audio output until a user gesture; the session enters
  `Error{AutoplayBlocked}` with one **Start audio** button (LiveKit ships a StartAudio button for this,
  note 04).
- **Slow / lossy network.** Reconnecting overlay; captions mark the gap; agent audio that was cut is
  marked interrupted with cause `Network`, not `User`.
- **Long sessions.** After 60 min (proposed) captions and transcript virtualise older turns; the
  latency sparkline keeps 20 turns.
- **Tiny sizes.** `voice-session/in-thread` row at 28 px height: orb 16 px, viz 5 bars, controls icon-only.
  Below 240 px width the session falls back to `voice-mini/pill` ([voice-control-bar.md](voice-control-bar.md)).
- **Dense IDE panels.** Side-panel variant hides the hero orb under 360 px height and keeps a 24 px orb
  in the header.
- **RTL.** Control order mirrors (End stays at the trailing edge in reading order); captions follow the
  language of each segment; latency bars run right-to-left with segment order preserved.
- **Touch.** 44 × 44 px minimum targets; tap orb = interrupt in `AgentSpeaking`, wake in `Asleep`,
  nothing otherwise (no accidental mutes).
- **Multiple sessions.** One live session per window; starting a second ends or holds the first (app).
- **Agent ends the call.** `Ended{AgentEnded}` with the agent's last line kept in captions.

## 7. Accessibility
- **Labels per state** (default English, `VoiceLabels`): Connecting "Connecting to <agent>"; Listening
  "Listening"; UserSpeaking "Hearing you"; Thinking "Thinking" / "Running <tool>" / "Waiting for your
  approval"; AgentSpeaking "<agent> is speaking"; Interrupted "You interrupted <agent>"; Muted
  "Microphone muted"; Asleep "Paused, press Space to talk"; OnHold "On hold"; Reconnecting
  "Reconnecting, attempt <n> of 5"; Ended "Call ended, <m> minutes".
- **One live region per session** (polite) announces state changes **except** UserSpeaking and
  AgentSpeaking (announcing them would talk over the conversation); assertive only for Error, mic
  permission and NeedsApproval. Screen-reader users get captions as the content channel
  ([live-captions.md](live-captions.md)), not the live region.
- **Deaf and hard-of-hearing users.** Captions are on by default when the OS caption preference is on,
  and one keystroke (`C`) otherwise; every agent state is visible as text; barge-in and agent-ended are
  shown as caption markers; the text input stays available (rule 5); earcons are always paired with a
  visual; mobile may add haptics for "agent started speaking" (opt-in, proposed).
- **Speech-impaired users.** Typing in the session sends a text turn that the agent answers by voice
  or text (app setting). VAD thresholds are adjustable (Studio + user setting "needs a louder voice to
  start").
- **Vestibular / photosensitive.** Audio-reactive motion modulates size and shape, not luminance
  (≤ ±10 % luminance, low-passed at 3 Hz); no full-area flashes above 3/s ([voice-orb.md](voice-orb.md) §7).
- **Focus order (call layout):** status line → captions region → control bar (mic, PTT, captions,
  share, devices, more, End). End is last but has its own hotkey.
- **Contrast:** status text ≥ 4.5:1; orb silhouette and viz bars ≥ 3:1; latency bands use label + colour.

## 8. Props sketch
```rust
pub enum VoiceSessionState {
    Idle,
    Connecting { step: ConnectStep, since: Instant },          // Mic | Transport | Agent
    Listening { since: Instant },
    UserSpeaking { since: Instant },
    Thinking { activity: VoiceActivity, since: Instant, stalled: bool }, // Reasoning | Tool(ToolId) | Handoff(AgentId) | NeedsApproval
    AgentSpeaking { utterance: UtteranceId, since: Instant },
    Interrupted { utterance: UtteranceId, at_word: Option<u32>, cause: InterruptCause }, // User | Button | Network
    Asleep { since: Instant, wake: WakeMode },                  // Tap | Hotkey | WakeWord
    OnHold { since: Instant },
    Reconnecting { attempt: u8, next_in: Duration },
    Error { kind: VoiceError },  // MicDenied | NoDevice | DeviceLost | Transport | AutoplayBlocked | Agent(ErrorKind) | Quota
    Ended { reason: EndReason, duration: Duration, turns: u32, interruptions: u32 },
}

pub struct VoiceOverlays {
    pub muted: Option<MuteKind>,             // Soft | Hard (default Hard)
    pub captions: bool,
    pub sharing: Option<ShareState>,         // voice-control-bar.md
    pub presentation: Presentation,          // Full | InThread | Panel | Mini
}

pub struct VoiceSnapshot {                   // what every family-K part receives
    pub state: VoiceSessionState,            // VoiceSessionState::derive(&AgentState, &AudioEvents, &Timers)
    pub overlays: VoiceOverlays,
    pub mic: LevelSignal,                    // §2.3
    pub agent: LevelSignal,
    pub turn_mode: TurnMode,                 // Vad | PushToTalk { key: Option<Keystroke> }
    pub latency: Option<TurnLatency>,        // §2.4, last completed turn
}

pub struct VoiceSession {
    pub agent: AgentState,
    pub snapshot: VoiceSnapshot,
    pub variant: VoiceSessionVariant,        // Call | InThread | SidePanel | Pro(..)
    pub timings: VoiceTimings,               // §5 tokens, overridable
    pub preconnect_buffer: Option<Duration>, // capture while connecting, max 2 s (proposed), None = off
    pub allow_resume_after_false_barge: bool,
    pub labels: VoiceLabels,
    pub motion: MotionLevel,
    pub on_end: Callback<EndReason>,
}

pub struct TurnLatency {
    pub endpoint: Option<Duration>, pub stt: Option<Duration>, pub llm: Option<Duration>,
    pub tts: Option<Duration>, pub playout: Option<Duration>, pub total: Duration,
    pub band: LatencyBand,                   // Good | Ok | Slow | VerySlow (§2.4 thresholds)
}

pub struct VoiceLatency {
    pub history: Vec<TurnLatency>,           // capped at 20
    pub variant: LatencyVariant,             // Chip | Segments | Glyph | Pro(..)
    pub budget: LatencyBudget,               // per-segment and total, proposed defaults
    pub motion: MotionLevel,
}
```

## 9. Related components
[composer-voice-input.md](composer-voice-input.md) (dictation and `VoiceModeEntry`) ·
[thinking-orb.md](thinking-orb.md) (orb engine) · [voice-orb.md](voice-orb.md) ·
[voice-visualisers.md](voice-visualisers.md) · [voice-control-bar.md](voice-control-bar.md) ·
[live-captions.md](live-captions.md) · [text-reveal.md](text-reveal.md) (word reveal in captions) ·
[approval-card.md](approval-card.md) (approvals during a call) · [acting-indicator.md](acting-indicator.md)
and [takeover-banner.md](takeover-banner.md) (screen share is viewing, control is family G) ·
[status-text.md](status-text.md) ("Still thinking…") · [elapsed-timer.md](elapsed-timer.md) (call timer).

## 10. References (study only unless MIT/Apache, per [../licensing.md](../licensing.md))
| Reference | What we take | Licence note |
|---|---|---|
| LiveKit Agents UI (note 04, verified page) | state-reactive visualisers, control bar with device selector and persisted devices, transcript with "scroll to end", pre-connect prompt, StartAudio button | Not in sources.csv; note 01 says free/open source, licence type not checked — study only until checked |
| ElevenLabs UI (notes 01, 04) | state names (connecting, initializing, listening, speaking, thinking), orb, bar visualiser, transcript viewer, mic selector | Not in sources.csv; licence type not checked — study only |
| Pipecat Voice UI Kit (note 04) | metrics tab (latency in dev tools), DTMF keypad, transcript overlay, bot volume | Not in sources.csv; licence not checked — study only |
| OpenAI Realtime console / realtime-agents (note 04) | VAD vs push-to-talk toggle, event log, handoff rows, tool approval event | Not in sources.csv; licence not checked — study only |
| ChatGPT voice, Gemini Live, Meta AI, Hume EVI, Sesame (note 04) | in-thread voice; Hold/resume; floating pill; captions toggle; full-duplex toggle; post-call summary | Products — reference only |
| Scrim UI (note 01) | "full-duplex voice with barge-in" as a pattern | Has a Pro tier; licence not checked — study only |
| DopeOblivion voice hero [R020] | "Listening" pill with breathing dot | Study only |
| SPY UI [R136] | idle / listening / thinking / speaking orb set as one component with a state prop | Paid tiers — study only |
| Libraries.dev Voice, Voice ring [R100] | voice and ring visuals | Five core packages are MIT (note 16); whether Voice / Voice ring are among the MIT packages is **not confirmed** — study only until checked |
| Cuelume [R110] | optional earcons (filler, ended) | MIT per licensing.md — port with attribution |
