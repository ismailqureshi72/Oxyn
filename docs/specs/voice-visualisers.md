**Status:** draft v0.1 (2026-11-12).

# Spec: voice visualisers (`voice-viz`: waveform, bars, radial, particle)

Part of family K ([live-voice.md](live-voice.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **5 free + 14 Pro = 19 variants**.
All variants read the shared audio contract ([live-voice.md](live-voice.md) §2.3) and the voice session
state; none of them talks to an audio API. Numbers are Oxyn **proposals** unless a source is cited.

## 1. Purpose
A visualiser is the "is sound happening, and whose?" strip of a voice UI: in a control bar, under a
caption, in a mini pill, or as the hero of a call screen when the app prefers bars to an orb. The kits
in note 04 treat it as a family, not one widget: LiveKit Agents UI ships five (Aura, Wave, Radial, Grid,
Bar), each reacting to **agent state + audio level**; Pipecat has basic bars with peak lines, a circular
waveform and a WebGL plasma; ElevenLabs UI has a Bar Visualizer with states connecting / initializing /
listening / speaking / thinking, a Live Waveform and a dot-matrix "Matrix"; Hume EVI shows a live
waveform and a spectrogram timeline. Gemini Live's floating pill shows transcript text above a waveform.

The user wants three things from it:
1. **Mic input vs agent output, unmistakably.** "That's me" vs "that's the agent" by position, shape and
   label, with colour as the third cue.
2. **Proof of an open mic** even in silence (live-voice.md rule 3): an idle floor that is visibly alive.
3. **A state when there is no audio** (connecting, thinking, muted, asleep, error) that does not look
   like a broken meter.

Rules (proposed):
1. **Two sources, one selector.** `TurnSource::Auto` shows the mic while Listening / UserSpeaking and the
   agent while AgentSpeaking, crossfading 160 ms; `Mic` or `Agent` pins one; `Both` draws two lanes or a
   mirror (variant-dependent) and **labels them** ("You", agent name) when ≥ 120 px wide.
2. **Non-audio states have their own motion** (§3), drawn from state, never from fake audio.
3. **Never dead while the mic is open**: minimum bar height 8 % of max, floor wave amplitude 1 px.
4. **Never strobe**: bars move in height, not brightness; peak lines fall at a fixed rate.
5. **Data rate ≠ frame rate.** Data arrives at 30–60 Hz; rendering interpolates up to the display rate
   (≤ 60 fps) and drops to data rate in low-power mode.

## 2. State machine
```
                          start
  Off ─────────────────────────────▶ Connecting (scan: a highlight sweeps across bars / wave, 1.2 s)
                                         │ ready
                                         ▼
                 ┌─────────────▶ ListeningFloor (mic floor; bars at 8–15 %, slow shimmer of the floor)
                 │                      │ mic level > gate
                 │                      ▼
                 │               MicActive (bars/wave = mic bands, "You" lane)
                 │                      │ end-of-turn
                 │                      ▼
                 │               Thinking (no audio; travelling ripple left→right at 1 cycle / 1.4 s,
                 │                         amplitude 20 %; Tool: ripple 1.25× faster)
                 │                      │ first audible agent frame
   playout ends  │                      ▼
                 └─────────────── AgentActive (bars/wave = agent bands, agent lane)
                                        │ barge-in
                                        ▼
                                 Interrupted (agent bars drop to floor 80 ms, cut tick on waveform) ─▶ MicActive
  Overlays / other states:
   Muted        mic lane → flat 1 px line + slashed-mic glyph; agent lane unaffected
   Asleep       all bars at floor, 40 % opacity, no shimmer
   OnHold       still frame + ⏸
   Reconnecting dashed floor line, scan paused
   Stale        no frames for 150 ms → decay 300 ms; 2 s → "No audio" flat state with label
   Error        flat line in danger tone + glyph
   Reduced      single level bar (or two, for Both) updated at 10 Hz, no ripple or scan
```

## 3. States table
| State | Bars | Waveform | Radial | Particle | Label (≥ 120 px) | Keyboard |
|---|---|---|---|---|---|---|
| `Off` | not drawn | not drawn | not drawn | not drawn | — | — |
| `Connecting` | floor + sweep highlight | flat line + travelling dot | ring with rotating 60° arc | particles gather to centre | "Connecting" | — |
| `ListeningFloor` | 8–15 % heights, shimmer ±3 % | 1 px line with floor noise | ring at r0, ±2 % | slow drift | "Listening" | — |
| `MicActive` | mic bands | mic envelope history | ring deformed by mic bands | particles pushed inward by mic level | "You" | — |
| `Thinking` | ripple travels across bars | sine ripple 20 % amplitude | arc orbits 1 turn / 1.4 s | swirl | "Thinking" | — |
| `AgentActive` | agent bands (playout-aligned) | agent envelope | ring deformed by agent bands | particles pushed outward | agent name | — |
| `Interrupted` | agent bars fall to floor 80 ms | vertical cut tick at the write head | ring snaps to r0 | burst stops, reverses | "Interrupted" (1.2 s) | — |
| `Muted` | mic lane flat + slashed mic | flat line | ring flat, slashed mic in centre | particles still | "Muted" | — |
| `Asleep` | floor 40 % | dim line | dim ring | dim, still | "Paused" | — |
| `Reconnecting` | dashed floor | dashed line | dashed ring | still | "Reconnecting" | — |
| `Stale` / no audio | floor | flat | r0 | still | "No audio" after 2 s | — |
| `Error` | flat, danger | flat, danger | ring danger | none | error text | — |

Visualisers are never focusable. Any interaction (click to mute, click to open devices) belongs to the
control that hosts them ([voice-control-bar.md](voice-control-bar.md)).

## 4. Micro-interactions and mapping
- **Bars (band mapping).** N bars (5, 7, 9, 16, 32) take the 8 / 16 / 32 contract bands, resampled with
  linear interpolation in log-frequency; **speech-centred ordering** for odd N ≤ 9: the loudest
  band region (≈ 250 Hz–2 kHz) sits in the middle bars so the shape reads as a voice, not as a spectrum
  analyser (proposed). Height = `floor + (1 − floor) · band`, floor 8 %. Bars are 2–6 px wide, gap
  = width ÷ 2, rounded caps, vertically centred (mirrored) by default.
- **Peak hold** (Pro `bars-peaks`): a 1 px line at the bar's recent max; holds 400 ms, then falls at
  1.5 × bar-height per second (Pipecat's "bars with peak lines" is the reference, note 04; timings ours).
- **Waveform (envelope history).** Not raw samples (rule 6, no PCM): each data frame appends one column
  whose height = smoothed RMS level; history scrolls left at a constant rate with the write head at
  85 % of the width (same as composer dictation strip). Columns from the mic and agent use different
  ink and the agent's columns are offset by `playout_offset_ms`.
- **Radial.** Bands mapped around 360° (mirrored left/right so the shape is symmetric); radius
  `r0 · (1 + 0.20 · band)`; a closed Catmull-Rom path through the points; the ring rotates slowly
  (1 turn / 20 s) only in Thinking so speech shapes stay readable.
- **Particle.** 60–240 particles (by size) on a disc; force = level × direction (inward for mic,
  outward for agent); damping 0.9 per frame at 60 fps; respawn at the rim/centre. No colour flashing:
  particle alpha fixed per state.
- **Source crossfade.** Mic ↔ agent: the old source's heights ease to floor while the new source rises
  (160 ms `ease.in-out`); lane labels crossfade 120 ms.
- **Both / duet layouts.** `mirror`: agent above the centre line, mic below; `lanes`: two rows with
  labels at the leading edge. In full-duplex both move at once.
- **Clipping.** Top 2 px of a bar (or waveform column) in `warn` when `clipped`; the host shows "Your
  mic is too loud" after 2 s of repeated clipping (once per session).
- **Too quiet.** If mic speech-band level stays under −45 dBFS for 3 s while the user is speaking
  (VAD on), host hint "We can barely hear you" (thresholds shared with composer-voice-input.md).
- **Enter / exit.** Bars grow from floor in a 30 ms stagger from the centre outwards (cap 9 bars, then
  all together); exit is a 120 ms fade.

## 5. Timing
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Data in | 30–60 Hz | — | live-voice.md §2.3 |
| Render | ≤ 60 fps visible, 30 fps ≤ 24 px tall, data rate in low power, 15 fps unfocused, paused hidden | interpolate between frames | |
| Level smoothing | attack 30 ms, release 150 ms | exponential | shared |
| Bar height follow | per frame, no extra spring | — | smoothing already applied |
| Peak hold / fall | 400 ms / 1.5 heights per s | linear | Pro `bars-peaks` |
| Source crossfade | 160 ms | `ease.in-out` | mic ↔ agent |
| Connecting sweep | 1200 ms loop | `ease.in-out` | |
| Thinking ripple | 1400 ms per cycle (Tool 1120 ms) | sine | `loop.calm`-adjacent |
| Floor shimmer | 2400 ms loop, ±3 % | sine | Listening |
| Interrupted drop | 80 ms | `ease.firm` | |
| Waveform scroll | constant; 1 column per data frame | linear | write head at 85 % |
| Radial idle rotation | 1 turn / 20 s (Thinking only) | integrated | |
| Enter stagger | 30 ms per bar, cap 9 | `ease.out` | |
| Exit | 120 ms | `ease.in` | |
| Stale decay | 300 ms | exponential | after 150 ms gap |
| **Reduced motion** | one horizontal level bar per source (two for `Both`, labelled), updated at 10 Hz, no smoothing animation; Thinking = static "Thinking" label with three still dots; no sweep, ripple, shimmer, scroll or particles | — | level stays (it's information) |

## 6. Variants
### Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `voice-viz/bars` | 5–9 rounded mirrored bars, speech-centred ordering; the default in control bars and pills | all |
| `voice-viz/waveform` | Scrolling envelope history with write head at 85 %, mic and agent in two inks (Gemini Live pill puts text above a waveform, note 04) | chat, desktop, mobile |
| `voice-viz/radial` | Closed band ring around a still core or around the orb (radial visualiser exists in LiveKit, note 04) | voice, desktop |
| `voice-viz/particle` | Disc of particles pushed inward by the user and outward by the agent (T3, CPU) | voice, desktop |
| `voice-viz/glyph` | Terminal: `You ▁▂▅▇▅▂▁` / `Agent ▂▃▆▃▂` with block characters at 15 fps; `⠿` braille ripple for Thinking | terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `voice-viz/bars-peaks` | Bars with falling peak-hold lines (Pipecat-style, note 04) |
| `voice-viz/bars-lanes` | Two labelled lanes, "You" and agent, for full-duplex and debugging |
| `voice-viz/mirror` | Agent above, user below a shared centre line |
| `voice-viz/grid` | Dot grid whose columns light by band (LiveKit has a Grid visualiser, note 04) |
| `voice-viz/matrix` | Dot-matrix display that can also show a short word ("LISTENING") between turns (ElevenLabs "Matrix", note 04) |
| `voice-viz/spectrogram` | Scrolling spectrogram, 32 bands, perceptual colour map (Hume EVI timeline idea, note 04) |
| `voice-viz/oscilloscope` | Smooth single line drawn from bands (no PCM), glowing trail in dark tuning |
| `voice-viz/circular-wave` | Waveform history wrapped around a circle (Pipecat circular waveform, note 04) |
| `voice-viz/plasma` | T4 plasma field whose turbulence follows level (Pipecat plasma, note 04) |
| `voice-viz/aurora-line` | Horizontal aurora ribbon above the control bar, height = level |
| `voice-viz/liquid-line` | Gooey line whose blobs rise with bands |
| `voice-viz/led` | Segmented LED meter with 12 segments per bar, retro hardware look |
| `voice-viz/edge-glow` | The window or panel border glows with level (pairs with [border-beam.md](border-beam.md)) |
| `voice-viz/mic-ring` | Tiny ring of radial ticks around the mic button, the control bar's built-in meter |

## 7. Edge cases
- **No bands from the adapter** (RMS only): bars are synthesised as `level × fixed voice-shaped
  profile` with a 5 % per-bar random walk; labelled `bands: Synthetic` in debug. Radial and spectrogram
  require bands and fall back to `bars`.
- **No level at all** (state only): visualiser shows state motions only; Mic/Agent active show a
  60 %-height still pattern with the speaker label (never fake movement).
- **Stalled audio thread.** Stale rules (§2); never freeze at a high value.
- **Agent playout offset unknown.** Use 0 and accept early motion; the orb and bars stay in sync with
  each other because they share the signal.
- **Long sessions.** Waveform history keeps one screen width; older columns are discarded (not stored).
- **Tiny sizes.** 16–20 px tall: bars only (3–5 bars, 2 px wide, 1 px gap), no labels; < 12 px: a
  single dot whose scale follows level. Waveform needs ≥ 80 px width, else falls back to bars.
- **Dense IDE panels.** 5 bars at 14 px height in the panel header, monochrome ink, 30 fps.
- **RTL.** Waveform scroll direction mirrors (newest on the leading edge in reading order, write head
  at 15 %), lanes' labels move to the right; radial unchanged.
- **Touch.** Not interactive; hosts give it no hit area so it never steals a tap from the mic button.
- **High contrast / forced colours.** Bars in system text colour, agent lane hatched; floor line kept.
- **Slow networks.** Agent frames arrive in bursts: interpolation never runs ahead of the newest frame;
  a burst after a gap fades in over 100 ms.
- **Very fast / very slow agent.** Thinking ripple only if Thinking lasts ≥ 200 ms; after 10 s the
  ripple slows to half speed (calm, not alarming), and the host's "Still thinking…" carries meaning.

## 8. Accessibility
- **Decorative** (`aria-hidden` / not exposed in GPUI): the visualiser never carries information that
  is not also in the status text or caption label.
- **Exception — level meter:** when a variant is used as the *only* mic meter (e.g. `mic-ring` in a
  bar without a status line), the hosting button's label includes the state ("Microphone on, hearing
  you"); no numeric level is announced.
- **Deaf / hard-of-hearing:** agent-active motion must be paired with the caption speaker label and the
  status line; with `Both`, lanes are labelled in text, not only colour. Visualisers are *not* a
  substitute for captions.
- **Photosensitivity:** heights move, brightness does not; peak lines fall at a fixed rate; the
  spectrogram uses a perceptual colour map without full-area luminance jumps; `edge-glow` luminance is
  low-passed at 3 Hz and capped at ±10 % (voice-orb.md rule 2).
- **Contrast:** bars ≥ 3:1 against background; the mic floor line ≥ 3:1 (it is the "mic on" proof);
  agent and user inks differ in lightness by ≥ 20 L* (OKLab) as well as hue.
- **Reduced motion:** level bars at 10 Hz (§5).

## 9. Props sketch
```rust
pub struct VoiceViz {
    pub agent: AgentState,                 // Thinking sub-phase (Tool speeds the ripple), stall
    pub session: VoiceSessionState,        // live-voice.md §8
    pub overlays: VoiceOverlays,           // muted → mic lane flat
    pub mic: LevelSignal,                  // bands + level + stale
    pub agent_level: LevelSignal,
    pub source: TurnSource,                // Auto (default) | Mic | Agent | Both
    pub variant: VizVariant,               // Bars | Waveform | Radial | Particle | Glyph | Pro(VizPro)
    pub bars: BarConfig,                   // count 5|7|9|16|32, width, gap, floor 0.08, ordering SpeechCentred | Linear
    pub peaks: Option<PeakHold>,           // hold 400 ms, fall 1.5/s (Pro)
    pub labels: Option<VizLabels>,         // "You", agent name; shown ≥ 120 px
    pub size: Size<Pixels>,
    pub tier: TierPolicy,
    pub power: PowerPolicy,
    pub motion: MotionLevel,
}
```
No callbacks: visualisers are pure functions of `(AgentState, VoiceSessionState, LevelSignal×2, clock)`.

## 10. Related components and references
**Related:** [live-voice.md](live-voice.md) (audio contract) · [voice-orb.md](voice-orb.md) (same
signal) · [voice-control-bar.md](voice-control-bar.md) (hosts bars / mic-ring) ·
[live-captions.md](live-captions.md) (waveform under captions) ·
[composer-voice-input.md](composer-voice-input.md) (dictation waveform shares bar geometry and
smoothing) · [loader-glyph.md](loader-glyph.md) (block/braille glyphs for `glyph`) ·
[border-beam.md](border-beam.md) (`edge-glow`).

| Reference | What we take | Licence note |
|---|---|---|
| LiveKit Agents UI visualisers (note 04) | five styles (Aura, Wave, Radial, Grid, Bar) driven by state + level | Not in sources.csv; licence not checked — study only |
| Pipecat Voice UI Kit (note 04) | bars with peak lines, circular waveform, plasma | Not in sources.csv; licence not checked — study only |
| ElevenLabs UI (note 04) | Bar Visualizer states, Live Waveform, Matrix | Not in sources.csv; licence not checked — study only |
| Hume EVI (note 04) | live waveform, spectrogram timeline | Product — reference only |
| Gemini Live (note 04) | transcript above a waveform in a floating pill | Product — reference only |
| SPY UI AI Voice Visualizer [R136] | listening / processing visualiser states | Paid — study only |
| Phil Pitus composer [R021] | waveform strip with playhead (shared geometry with dictation) | Study only |
| Paper Shaders [R140] | T4 technique for `plasma` | Apache-2.0 — port with attribution |
