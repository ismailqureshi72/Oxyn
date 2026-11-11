**Status:** draft v0.1 (2026-11-12).

# Spec: voice orb (`voice-orb`)

Part of family K ([live-voice.md](live-voice.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **4 free + 16 Pro = 20 variants**.

The voice orb is the [thinking-orb.md](thinking-orb.md) engine (one spring-smoothed **energy**, render
tiers T0–T4, per-size tuning, phase-locked clock) with three additions: it reads the **voice session
state** instead of only `LoaderPhase`, it reads **audio levels** (mic and agent) through the
[live-voice.md](live-voice.md) §2.3 contract, and it has a **direction of flow** that says whose turn it
is (inward = the agent is taking sound in; outward = the agent is giving sound out). Thinking-orb
already reserves this: its `listening` state maps to family K, and its Pro `voice-ripple` / `voice-swirl`
variants accept an audio level. Numbers here are Oxyn **proposals** unless a source is cited.

## 1. Purpose
In a live call the orb is the agent's face: the one element a user glances at to know whether to talk,
wait or interrupt. References treat it the same way: ChatGPT's legacy voice mode is a blue orb with
mute, camera, interrupt and exit and no transcript (note 04); ElevenLabs UI ships an Orb with idle /
listening / talking states; LiveKit's Aura visualiser reacts to agent state and audio level; Orbkit has
idle / thinking / speaking [R067]; SPY UI has an interactive voice orb with chat states [R136];
thinking-orbs names a `listening` state among its nine [R068].

What the user needs from it:
1. **Whose turn** — readable in under 300 ms from motion direction and shape, confirmed by a caption.
2. **That it hears them** — the orb moves with *their* voice while they talk (mic level), which is the
   fastest proof the mic works.
3. **That it is speaking, and when it stops** — the orb moves with the agent's voice, aligned to what the
   user hears; when interrupted it visibly stops.
4. **That nothing is happening** (asleep, on hold, muted) without anything looking broken.

Rules (proposed):
1. **Same engine, same energy.** Session state sets an energy target; audio adds a bounded term on top.
   The orb never snaps.
2. **Audio drives shape and size, not brightness.** Luminance modulation ≤ ±10 %, low-passed at 3 Hz
   (§7). Speech syllable rate (roughly 3–6 Hz) must never become a flashing light.
3. **One source at a time** by default (`TurnSource::Auto`); `Both` only on variants that draw two
   layers (e.g. `voice-orb/duet`).
4. **Muted is a badge, not a new orb.** The orb keeps its state motion; a slashed-mic badge appears.
5. **Thinking inside a call looks like thinking-orb Thinking**, so users who know the chat orb read it
   the same way.

## 2. State machine
```
                         start
  Idle (e 0.12, still-ish) ───▶ Connecting (contract 300 ms → slow orbit, e 0.30)
     ▲                              │ ready
     │                              ▼
     │              ┌──────▶ Listening (inward drift, breathe 3.2 s, e 0.20 + 0.15·mic floor)
     │              │               │ user speech
     │              │               ▼
     │              │        UserSpeaking (inward flow, radius 1 + 0.12·mic, e 0.30 + 0.40·mic)
     │              │               │ end-of-turn
     │              │               ▼
     │              │        Thinking{..} (thinking-orb "solving", e 0.45; Tool e 0.70; NeedsApproval amber hold e 0.20)
     │              │               │ first audible frame
     │   playout end│               ▼
     │              └──────── AgentSpeaking (outward flow, radius 1 + 0.18·agent, e 0.45 + 0.45·agent)
     │                              │ barge-in
     │                              ▼
     │                       Interrupted (flow reverses 160 ms, agent term → 0 in 80 ms) ──▶ UserSpeaking
     │
     ├── Ended: shrink into the summary glyph (260 ms) ──▶ Idle
  Overlays: Muted (badge) · Asleep (40 % brightness, breathe 6 s, e 0.05) · OnHold (still poster + ⏸ badge)
            · Reconnecting (desaturate 60 %, orbit continues at e 0.15) · Error (collapse to 70 % radius, danger, still)
            · Stalled (thinking-orb stall rules) · Reduced (still poster per state + static level ring)
```

### Amplitude mapping (audio-reactive)
```
level L ∈ [0,1]  (live-voice.md §2.3: dBFS floor −60, ceil −6, curve L^0.6, attack 30 / release 150 ms)
   │  low-pass for luminance only: 3 Hz one-pole
   ▼
radius   r = r0 · (1 + g_r · L)              g_r: 0.12 user, 0.18 agent   (clamped to r0 · 1.25)
energy   e = e_state + g_e · L               g_e: 0.40 user, 0.45 agent   (clamped to 1.0)
jitter   dot displacement = 0.04 · r0 · L    (T3 only)
bands    b[0..8] → per-latitude ripple amplitude (low bands at the poles, high at the equator) on variants
         that declare `uses_bands`
light    luminance = base · (1 + 0.10 · lowpass_3Hz(L))
spring   r follows its target with stiffness 300, damping 28, mass 1 (ζ ≈ 0.81) → quick but no wobble
```
All gains are proposed and exposed in `VoiceOrbTuning`. The **mic floor** term in Listening (0.15 × the
noise-floor level) is what makes an open mic visibly "alive" even in silence (live-voice.md rule 3).

## 3. States table
| State | Motion (shape language) | Energy | Audio input | Palette | Host action on orb | Keyboard |
|---|---|---|---|---|---|---|
| `Idle` | barely-moving still, 1 turn / 20 s | 0.12 | none | accent at 50 % chroma | tap starts session (if host allows) | Enter (if focusable) |
| `Connecting` | one contraction to 85 % then slow orbit | 0.30 | none | accent | — | — |
| `Listening` | slow inward drift, breathe 0.97 ↔ 1.0 | 0.20 + floor | mic floor only | accent | — | — |
| `UserSpeaking` | inward flow, swells with the user | 0.30 + 0.40·L | mic | accent, `voice.user` tint 20 % | — | — |
| `Thinking{Reasoning}` | thinking-orb solving (tumble, inward latitudes) | 0.45 | none | accent → spectral drift | tap = stop answer | `S` |
| `Thinking{Tool}` | thinking-orb working / searching band | 0.70 | none | accent, brighter core | tap = stop | `S` |
| `Thinking{NeedsApproval}` | flow stops, hold-and-breathe 2.4 s | 0.20 | none | `waiting` amber | tap focuses the card | — |
| `AgentSpeaking` | outward flow, swells with the agent | 0.45 + 0.45·L | agent (playout-aligned) | accent + `voice.agent` glow | tap = interrupt | Space/`S` |
| `Interrupted` | outward flow reverses inward, one 160 ms snap | → 0.30 | agent → 0, mic takes over | accent | — | — |
| `Muted` (overlay) | unchanged + 12 px slashed-mic badge at 4 o'clock | unchanged | mic ignored | unchanged | — | `M` toggles (control bar) |
| `Asleep` | 40 % brightness, breathe 6 s, no tumble | 0.05 | none | desaturated 40 % | tap wakes | Space |
| `OnHold` | still poster + ⏸ badge | 0 | none | desaturated | tap resumes (if host allows) | Enter |
| `Reconnecting` | desaturated orbit | 0.15 | none | 60 % desaturated | — | — |
| `Error` | collapse to 70 % radius, still | 0 | none | `danger` | tap = retry (host) | Enter |
| `Ended` | shrinks into the summary glyph | — | none | — | — | — |

## 4. Micro-interactions
- **Direction of flow** is the main turn cue: inward (dots or noise field drift toward the centre) while
  the agent is taking input; outward while it is giving output. The swap takes one 160 ms spring on the
  flow speed through zero, never a jump.
- **Swell with voice.** Radius follows the amplitude map (§2) with the spring above; the orb never
  grows more than 25 % so layout around it is stable (the orb's box is sized for r0 · 1.25).
- **Onset accent.** When UserSpeaking begins, one 120 ms ring (1 px, 30 % opacity) leaves the rim — a
  "heard you" tick. When AgentSpeaking begins, nothing extra: the swell itself is the cue.
- **Barge-in.** Agent term drops to 0 over 80 ms (`ease.firm`); flow reverses; if the agent is still
  audible after 300 ms the orb holds a 2 px `warn` rim until playout stops ("Stopping…" in captions).
- **Tap.** Tap behaviour depends on state (table §3); press scales 0.96 (80 ms), release springs back
  (`spring.snap`). Hover shows the state caption tooltip after 400 ms. Long-press (500 ms) on touch
  opens the control sheet (variant-level option).
- **Mute badge.** Badge scales 0.6 → 1 (140 ms `spring.snap`), slash stroke draws 140 ms.
- **Sleep.** Brightness eases to 40 % over 1200 ms; wake reverses in 240 ms with one contraction.
- **Handoff to another agent.** Palette crossfades to the new agent's palette over 600 ms (OKLab), so
  a multi-agent call shows *who* speaks by colour **and** by the caption label.
- **Thinking.** Exactly thinking-orb's Thinking/Working motions and timings ([thinking-orb.md](thinking-orb.md) §4–5).

## 5. Timing
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Enter (mount, in call) | 320 ms | `ease.out`, scale from source button | from composer voice-mode button |
| Energy change | ≈ 600 ms settle | spring k 120, c 22, m 1 | thinking-orb default |
| Radius follow (audio) | ≈ 180 ms settle | spring k 300, c 28, m 1 | proposed |
| Level smoothing | attack 30 ms, release 150 ms | exponential | live-voice.md §2.3 |
| Luminance low-pass | 3 Hz cut-off | one-pole | photosensitivity rule |
| Flow direction swap | 160 ms | spring through zero | turn change |
| Onset ring (UserSpeaking) | 120 ms | `ease.out` | once per turn |
| Barge-in drop | 80 ms | `ease.firm` | agent term → 0 |
| Stopping rim | after 300 ms | — | until playout stops |
| Listening breathe | 3200 ms loop | sine | |
| Asleep breathe / fade / wake | 6000 ms loop / 1200 ms / 240 ms | sine / `ease.in-out` / `ease.out` | |
| Connecting contract | 300 ms in, 300 ms out | `ease.in` / `ease.out` | then orbit 1 turn / 6 s |
| Palette change (handoff, state) | 600 ms | OKLab linear | |
| Mute badge | 140 ms | `spring.snap` | |
| Error collapse | 400 ms | `ease.firm` | |
| End shrink | 260 ms | `ease.in-out` | into summary glyph |
| Frame rate | ≤ 60 fps visible; 30 fps at ≤ 24 px; 15 fps unfocused; paused hidden | — | [loaders.md](loaders.md) §2.1 |
| **Reduced motion** | still poster per state; a **static level ring** (2 px arc, length = level, 10 Hz updates, no easing) shows mic/agent level so "it hears me" survives; state changes 0–80 ms crossfade; no flow, swell, breathe, onset ring or collapse | — | level ring is information |

### Sizes (proposed)
| Size | Use | T3 dots | Audio mapping | Notes |
|---|---|---|---|---|
| 16–20 px | in-thread voice row, mini pill | 24–36 | radius only (g_r halved) | no bands, no glow |
| 24–32 px | side-panel header, control bar | 48–80 | radius + energy | |
| 48 px | mini bubble ([voice-control-bar.md](voice-control-bar.md)) | 160 | + jitter | |
| 96–160 px | side-panel hero | 240–320 | + bands | |
| 200–320 px | call screen hero | 400 (T3) / shader ≤ 512² (T4) | full | DPR ≤ 2 |

## 6. Variants
### Free (4)
| Name | Tier | Idea |
|---|---|---|
| `voice-orb/dotted` | T3 | Fibonacci dot sphere; inward drift when listening, outward when speaking, dots swell with level; the voice tuning of `thinking-orb/dotted-breathe` (port of thinking-orbs, MIT) |
| `voice-orb/conic` | T2 | Conic-gradient disc; rotation direction flips with turn, radius follows level; works on every renderer |
| `voice-orb/blob` | T2 | 32-point noise blob; wobble amplitude = level, low bands push the bottom, high bands the top |
| `voice-orb/rings` | T2 | Core dot + concentric rings; rings travel inward while listening, outward while speaking, ring spacing tightens with level |

### Pro (16)
| Name | Tier | One-line idea |
|---|---|---|
| `voice-orb/liquid` | T4 | Marbled fluid in a glass rim; flow speed follows level, direction follows turn |
| `voice-orb/glass` | T4 | Refractive glass shell whose inner light pulses (≤ ±10 % luminance) with the agent |
| `voice-orb/aura` | T4 | Soft radial aura glow, state-coloured, edge breathes with level (aura-style visualiser idea, LiveKit, note 04) |
| `voice-orb/plasma` | T4 | Plasma filaments that lengthen with level |
| `voice-orb/ferrofluid` | T4 | Spikes rise per band, relax in silence |
| `voice-orb/gooey` | T4 / T3 | Metaball drops pinch off outward when speaking, merge inward when listening (gooey idea, SPY UI interactive orb [R136]) |
| `voice-orb/halftone` | T3 | Halftone sphere; dot size follows bands |
| `voice-orb/dither` | T3 | Ordered-dither sphere at 30 fps; level raises the light |
| `voice-orb/pixel` | T3 | 16 × 16 pixel orb at 15 fps for retro and terminal-adjacent apps |
| `voice-orb/cymatics` | T4 (T3 twin) | Sand that forms a pattern from the bands while speaking and scatters when the user talks (idea from [R001]) |
| `voice-orb/face` | T2 | Orb with two eye dots that look toward the user while listening and blink on turn change (mascot states from [R031]) |
| `voice-orb/ring-meter` | T2 | Thin ring of 48 radial ticks around a still core; ticks are the band meter (voice-ring idea, Libraries.dev [R100]) |
| `voice-orb/duet` | T3 | Two layers: inner sphere = agent, outer shell = user; both visible in full-duplex |
| `voice-orb/nebula` | T4 | Drifting gas cloud; calmest option for long calls |
| `voice-orb/iridescent` | T4 | Thin-film colours shift with turn (cool listening, warm speaking) plus labels |
| `voice-orb/wireframe` | T2 / T4 | Wireframe sphere; vertex displacement from bands (idea from [R008]) |

Each Pro variant names its T3 twin and T2 fallback in `meta.toml` (thinking-orb rule). Voice tunings of
thinking-orb variants count once, in family K, because the audio-reactive motion and turn flow are new
motion (variants.md: variants differ in structure or motion).

## 7. Edge cases
- **Silence while UserSpeaking** (long pause mid-sentence): the orb stays in UserSpeaking until the
  adapter's end-of-turn; the mic floor term keeps it alive.
- **Very loud input / clipping.** Radius clamps at 1.25 · r0; clipping adds a 1 px `warn` rim for
  400 ms (once per 5 s) — the visualiser and control bar show the hint, not the orb.
- **No level available** (adapter gives state only): the orb runs state motion with a synthetic
  0.3 level while speaking and **no** swell while listening; `level_source: None` disables the
  reduced-motion level ring (nothing to show).
- **Stale level.** `LevelSignal::Stale` → decay to 0 over 300 ms; never freeze mid-swell.
- **Agent speaks but the user hears nothing** (output device muted): the orb still moves (it shows
  what the agent is sending); the control bar shows "Speaker volume is 0" if the platform exposes it.
- **Light themes.** Additive glow vanishes on white: `dark` tuning flag as thinking-orb; light tuning
  uses normal blending and a 1 px rim at 3:1.
- **Tiny sizes.** At 16–20 px only radius follows level (no bands); 12 px falls back to a 6 px dot that
  scales with level.
- **Many orbs** (multi-agent call grid): only the speaking agent's orb is audio-reactive; others hold
  their state motion at half energy; > 3 animating orbs → low-power rules.
- **GPU loss / sleep-wake.** Swap to T3 twin at once (thinking-orb §5.3); session state is in the clock.
- **RTL.** No mirroring; badge stays at 4 o'clock (physical, not logical) so it never covers the
  caption anchor (proposed).
- **Touch.** Hero orb is a 44 px+ target; in-thread 16 px orb has a 44 px invisible hit area.
- **Very fast agent.** Thinking shorter than 200 ms is skipped (live-voice.md); energy still springs, so
  no hard jump.
- **Very slow agent.** Thinking-orb stall rules (soft at 5 s, hard at 45 s) apply; status text carries
  "Still thinking…".

## 8. Accessibility
- Decorative by default (`aria-hidden`); state is announced by the session's live region and shown by
  the status line / caption header. When the orb is the only element (hero layout with captions off),
  the host must keep the status line visible or label the orb ("Agent, listening").
- When the orb is a control (tap to interrupt / wake / retry), it is a button with a per-state label:
  "Interrupt <agent>", "Wake — start talking", "Retry connection"; `aria-pressed` is not used.
- **Deaf / hard-of-hearing:** the orb's turn cue (flow direction) is backed by the caption speaker
  label and the status line; the orb must never be the *only* sign the agent is speaking.
- **Photosensitivity:** luminance from audio is low-passed at 3 Hz and capped at ±10 %; onset rings
  ≤ 1 per turn; no full-area flashes; total change of any area larger than the orb never above 3/s.
- **Vestibular:** no zoom toward the viewer beyond the 1.25 clamp; no parallax; rotation ≤ 1 turn/s.
- **Reduced motion:** still poster per state + static level ring (§5).
- **Contrast:** orb silhouette ≥ 3:1 in both themes; amber (approval) and danger (error) also change
  motion and the status text, never colour alone.

## 9. Props sketch
```rust
pub struct VoiceOrb {
    pub agent: AgentState,                 // for Thinking sub-phases and stall
    pub session: VoiceSessionState,        // live-voice.md §8; selects motion + energy target
    pub overlays: VoiceOverlays,           // muted badge, hold, sharing (no visual effect)
    pub mic: LevelSignal,                  // live-voice.md §2.3
    pub agent_level: LevelSignal,          // playout-aligned
    pub source: TurnSource,                // Auto (default) | Mic | Agent | Both (duet only)
    pub variant: VoiceOrbVariant,          // Dotted | Conic | Blob | Rings | Pro(VoiceOrbPro)
    pub size: OrbSize,                     // Px16..Px48 | Hero(u16) up to 320
    pub palette: OrbPalette,               // per-agent palettes for handoff
    pub dark: bool,
    pub tuning: VoiceOrbTuning,            // gains, clamps, springs (§2 defaults)
    pub tier: TierPolicy,                  // thinking-orb
    pub power: PowerPolicy,
    pub interactive: VoiceOrbInteraction,  // None | TapAction (per-state, §3) | TapAndLongPress
    pub labels: VoiceLabels,
    pub motion: MotionLevel,
}

pub struct VoiceOrbTuning {               // proposed defaults
    pub radius_gain_user: f32,   // 0.12
    pub radius_gain_agent: f32,  // 0.18
    pub radius_max: f32,         // 1.25
    pub energy_gain_user: f32,   // 0.40
    pub energy_gain_agent: f32,  // 0.45
    pub luminance_gain: f32,     // 0.10 (hard cap 0.10)
    pub luminance_lowpass_hz: f32, // 3.0 (hard floor: cannot exceed 3.0)
    pub radius_spring: Spring,   // k 300, c 28, m 1
}
```

## 10. Related components and references
**Related:** [thinking-orb.md](thinking-orb.md) (engine, tiers, size tuning, stall) ·
[live-voice.md](live-voice.md) (states, audio contract) · [voice-visualisers.md](voice-visualisers.md)
(same level signal) · [voice-control-bar.md](voice-control-bar.md) (mini bubble hosts the orb) ·
[live-captions.md](live-captions.md) (speaker label backs the turn cue) ·
[composer-voice-input.md](composer-voice-input.md) (Pro `orb` dictation variant) · family L marks.

| Reference | What we take | Licence note |
|---|---|---|
| thinking-orbs / Libraries.dev [R068] [R100] | `listening` as an orb state; dotted canvas technique; size tuning; `dark` flag | thinking-orbs MIT — **port with attribution** (free `dotted`) |
| gpui-thinking-orbs (licensing.md) | GPUI port candidate | MIT per licensing.md — repository not yet reviewed |
| Orbkit [R067] | idle / thinking / speaking orb API | Study only |
| SPY UI [R136] | interactive voice orb with gooey physics and chat states | Paid — study only |
| ElevenLabs UI Orb (note 04) | idle / listening / talking orb with custom colours | Not in sources.csv; licence not checked — study only |
| LiveKit Aura (note 04) | visualiser reacting to state + level | Not in sources.csv; licence not checked — study only |
| ChatGPT legacy voice (note 04) | orb as the whole call UI | Product — reference only |
| eliakuratli mascot [R031] | listening / thinking states for `face` | Study only |
| pontusab particle field [R001], miu21590 wireframe orb [R008] | `cymatics`, `wireframe` ideas | Study only |
| Paper Shaders [R140] | T4 techniques (metaballs, mesh gradient) | Apache-2.0 — port with attribution |
| affectUI Voice Orb (note 11), Rare UI Fluid Orb (note 17) | that a voice orb is a sellable illustration | Paid / unclear — study only, do not copy |
