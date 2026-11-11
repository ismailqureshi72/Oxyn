**Status:** draft v0.1 (2026-11-12).

# Spec: voice control bar, mini voice window, screen-share overlay (`voice-controls`, `voice-mini`, `share-overlay`)

Part of family K ([live-voice.md](live-voice.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Three components that share one state source
(`VoiceSnapshot`) and one set of actions. Target **7 free + 15 Pro = 22 variants**:
`voice-controls` 3 + 6, `voice-mini` 2 + 4, `share-overlay` 2 + 5. Numbers are Oxyn **proposals**
unless a source is cited.

## 1. Purpose
During a call the user's hands need a small, stable set of controls: **mute**, **end**, **push-to-talk**
when the room is noisy, **which mic and speaker**, **captions on/off**, and **share my screen** so the
agent can see what they are talking about. The control set is consistent across products in note 04:
LiveKit's AgentControlBar (mic, camera, screenshare, chat, leave, a device selector, persisted device
choices), ChatGPT voice (End, mute, camera), Gemini Live (mute, end, camera / screen share, captions
toggle, Hold / resume), Pipecat (device controls, bot volume) and the OpenAI Realtime console (VAD vs
push-to-talk toggle).

Two more surfaces carry the call when the user looks elsewhere:
- **Mini voice window** — Gemini Live collapses to a floating pill and then a draggable circle while the
  user uses other apps (note 04); Meta AI shows a mic-in-use icon while multitasking (note 04). Oxyn's
  `voice-mini` keeps the session alive across in-app navigation and, on desktop, as its own small
  always-on-top window.
- **Screen-share overlay** — when the agent can see the screen, the user must always know **what** it
  sees, **that** it is seeing now, and have **Pause** and **Stop** one click away. Gemini Live shares
  after a system confirm; ChatGPT's agent pauses screenshots while the user has control (note 04).
  Viewing is not controlling: control of the surface is family G
  ([takeover-banner.md](takeover-banner.md), [acting-indicator.md](acting-indicator.md)).

Rules (proposed):
1. **End is always visible and always one action**, never behind "more", at the trailing edge, in
   `danger` tone with a text label at ≥ 280 px bar width. No confirm dialog (a call is cheap to restart);
   the Ended summary offers "Start again".
2. **Mute state is visible from two places** (mic button and orb/visualiser badge) and has a hotkey.
3. **Default mute is Hard** (track closed; OS mic indicator off). Soft mute (track open, not sent) is an
   app opt-in and enables the "You're muted" hint (live-voice.md §2.3).
4. **A lost device never auto-switches to another mic.** The session hard-mutes and offers the new
   default with one tap (same principle as composer-voice-input.md: don't record the wrong room).
5. **Screen sharing is never started without an OS picker** and never resumed after Pause without a
   user action. A visible frame marks every shared surface while frames are being sent.
6. **The mini window never hides state**: it always shows the orb (or a dot), the state word or icon,
   mute and end.

## 2. State machines

### 2.1 Control bar (`voice-controls`)
```
  Hidden ──session start──▶ PreConnect (Cancel · device picker · captions) ──ready──▶ Live
                                                                                    │
     Live  ⇄  Live{muted}            M / mic button
     Live  ⇄  Live{ptt_mode}         mode switch (more ⋯ → Hands-free / Push to talk)
     Live{ptt_mode} ── Space or PTT button down ──▶ Talking{held} ── up ──▶ Live{ptt_mode}
                                                       └─ drag off button / Esc while held ──▶ Live (turn discarded)
     Live ── devices ▾ ──▶ DeviceMenu{tab: Input | Output} ── pick ──▶ Live (choice persisted)
     Live ── share ──▶ (share-overlay §2.3) ;  Live ── CC ──▶ captions toggled
     Live ── device lost ──▶ DeviceLost (hard-muted, "Use <new default>?") ── accept ──▶ Live · dismiss ──▶ Live{muted}
     Live ── network drop ──▶ Reconnecting (End only + status) ──▶ Live | Error
     Live ── End / hotkey / agent ends ──▶ Ending (200 ms) ──▶ Ended (summary: Start again · View transcript) ──▶ Hidden
     Any ── window narrower than 240 px or user collapses ──▶ (handed to voice-mini)
  Per-button disabled-with-reason: share unsupported, camera none, PTT unavailable on touch-only web, etc.
```

### 2.2 Mini voice window (`voice-mini`)
```
  (none) ──collapse / navigate away / app loses focus (desktop opt-in)──▶ Pill{expanded}
     Pill ── idle 4 s, not speaking ──▶ Pill{compact}  (orb + state icon + mute + end)
     Pill ── drag ──▶ Dragging ── release ──▶ Snapping (to nearest corner/edge, 16 px margin) ──▶ Pill
     Pill ── drag to edge, ≥ 60 % outside ──▶ Docked (half-hidden tab with orb + level) ── tap ──▶ Pill
     Pill ── tap orb ──▶ Interrupt / Wake per session state (voice-orb.md §3)
     Pill ── double-click / ↗ ──▶ returns to full session (voice-session) ──▶ (none)
     Pill ── shrink (long-press on touch, ⌄ on desktop) ──▶ Bubble (48 px orb only, mute badge, level ring)
     Bubble ── tap ──▶ Pill
     Any ── session Ended ──▶ EndedToast (3 s, "Call ended · 4:12 · View") ──▶ (none)
```

### 2.3 Screen-share overlay (`share-overlay`)
```
  Off ── share button ──▶ Explain (first time per app: "The agent will see the screen or window you pick,
         about 1 frame per second, until you stop." · Continue) ──▶ Picking (OS picker owns focus)
         (Explain is skipped after the first share unless the app requires it every time)
  Picking ── cancelled ──▶ Off ;  denied ──▶ Denied ("Screen recording is blocked · How to allow") ──▶ Off
  Picking ── chosen ──▶ Starting (first frame) ──▶ Sharing{source, fps, last_sent}
  Sharing ── agent reads a frame (if adapter reports) ──▶ Sharing{looking: 1.2 s pulse}
  Sharing ── Pause ──▶ Paused (no frames sent; preview frozen + ⏸; agent told "sharing paused") ── Resume ──▶ Sharing
  Sharing ── user takes control via family G / sensitive field (app signal) ──▶ AutoPaused (reason) ── Resume ──▶ Sharing
  Sharing / Paused ── Stop · window closed · display removed ──▶ Stopped (toast 2 s) ──▶ Off
  Sharing ── call ends ──▶ Stopped (sharing never outlives the session)
```

## 3. States tables

### 3.1 Control bar buttons (default left → right; RTL mirrors)
| Control | States and look | Label (per state) | Primary action | Keyboard |
|---|---|---|---|---|
| Mic / mute | on: mic glyph + `mic-ring` level ticks; muted: slashed mic, `danger`-muted fill; Soft-muted + speech: hint bubble "You're muted"; disabled: no device | "Mute microphone" / "Unmute microphone" / "Microphone unavailable: <reason>" | toggle mute | `M` (focus anywhere in session); Space in VAD mode does nothing |
| Push to talk (PTT mode only; replaces mic as primary) | idle: "Hold to talk"; held: accent fill + level ring, "Release to send"; drag-off: "Release to cancel" | as shown | hold to talk | hold Space (when focus not in a text field) |
| Device picker ▾ | closed; open menu with Input / Output tabs, each device row with a live level meter (input) or Test sound (output); "System default (<name>)" first | "Choose microphone and speaker, current: <name>" | open menu | Enter/↓ open, ↑↓ move, Enter pick, Esc close |
| Captions CC | off; on (accent underline + `aria-pressed`) | "Turn captions on/off" | toggle | `C` |
| Screen share | off; picking (spinner); sharing (accent + small live dot); paused (⏸ badge) | "Share your screen with <agent>" / "Stop sharing" | start / open share menu (Pause · Stop · Change source) | `⇧S` (proposed) |
| Camera (optional) | off; on; no camera (disabled) | "Turn camera on/off" | toggle | `V` (proposed) |
| Chat / keyboard | opens the text input for typed turns (live-voice.md rule 5) | "Type a message" | focus input | `T` |
| Status (centre, ≥ 360 px) | state word + call timer ("Listening · 3:07"), latency chip | — | — | — |
| More ⋯ | Hands-free / Push to talk, speaker volume, hold, report a problem | "More call options" | menu | Enter |
| End | `danger`, "End" text ≥ 280 px wide | "End call" | end | `Esc` when focus is on the bar; app hotkey |

### 3.2 Bar-level states
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `PreConnect` | Cancel, devices, CC; status "Connecting…" after 300 ms | Cancel | devices | Esc cancel |
| `Live` | full set (table 3.1) | mute | all others | as table |
| `Talking{held}` (PTT) | PTT filled, level ring, hint | release | drag off cancel | Space up sends |
| `DeviceLost` | mic slashed + banner "Microphone disconnected · Use <name>" | use new | keep muted | Enter accept |
| `Reconnecting` | all but End disabled-with-reason "Reconnecting" | End | — | — |
| `Ending` / `Ended` | bar collapses into summary row | Start again | View transcript | Enter |

### 3.3 Mini window and share overlay
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Pill{expanded}` | 20 px orb, state word, one caption line (latest final words, 1 line, ellipsis at start), mute, end, ↗ | mute | end, expand | `⌘/Ctrl+Shift+M` mute (app-global, proposed), Enter on ↗ expands |
| `Pill{compact}` | orb, state icon, mute, end | mute | end | as above |
| `Bubble` | 48 px orb + slashed-mic badge when muted + level ring | tap = expand | long-press = end menu | Enter expands |
| `Docked` | 20 × 56 px edge tab with orb and level | tap | drag | Enter |
| `Dragging` / `Snapping` | 1.04 scale, shadow up | — | — | arrow keys move by 16 px when focused (keyboard drag) |
| `EndedToast` | "Call ended · 4:12 · View" | view | dismiss | Enter view |
| Share `Explain` | sheet with what the agent will see, frame rate, Continue / Cancel | Continue | Cancel | Enter / Esc |
| Share `Sharing` | **frame** around the shared screen/window (3 px accent, 8 px radius where the window has one) + **tab** at its top edge "Agent can see this · Pause · Stop"; **preview chip** in the bar: 96 × 60 thumbnail of the last sent frame + "1 fps · sent 0.4 s ago" | Pause | Stop, change source | `⇧S` opens share menu |
| Share `Sharing{looking}` | tab dot pulses once; preview gets "Looking" label 1.2 s | — | — | — |
| Share `Paused` / `AutoPaused` | frame turns dashed and neutral; tab "Sharing paused · Resume · Stop" (+ reason "while you're in control"); preview frozen with ⏸ | Resume | Stop | Enter resume |
| Share `Denied` | inline error with "How to allow" | How to allow | dismiss | Enter |
| Share `Stopped` | toast "Stopped sharing" 2 s; frame removed | — | — | — |

## 4. Micro-interactions
- **Button press.** Scale 0.94 (80 ms down, 140 ms `spring.snap` up); hover tint 100 ms. Icon morphs
  (mic ↔ slashed mic, share ↔ stop) are stroke draws of 140 ms.
- **Mic level ring.** 12 radial ticks around the mic button driven by the mic `LevelSignal` (the
  `voice-viz/mic-ring` geometry); at rest 2 ticks lit (the "mic on" floor); muted: ticks hidden.
- **Mute hint (Soft mute).** Bubble rises 6 px from the mic button (160 ms `ease.out`), stays 3 s, at
  most once per 30 s.
- **PTT.** Tap shorter than 200 ms in PTT mode shows "Hold to talk" (no turn sent); hold ≥ 200 ms opens
  the turn (same threshold as composer dictation); dragging ≥ 48 px off the button arms cancel (button
  turns neutral, hint "Release to cancel"). Pressing PTT while the agent speaks is a barge-in.
- **Device menu.** Opens upward from the bar (180 ms `ease.out`, scale 0.98 → 1). Input rows show a
  live 5-bar meter so users can *see* which mic hears them; output rows have "Test" playing a short
  tone (earcon, 600 ms). The chosen row gets a check; choice persists per app (LiveKit persists device
  choices, note 04). A newly plugged device slides in with a "New" tag for 10 s; it is never selected
  automatically (rule 4).
- **Captions toggle.** Toggling on slides the caption region up 8 px + fade (160 ms); off reverses.
- **End.** Press shows a 120 ms danger fill; the bar collapses into the thread summary row (260 ms).
- **Collapse to mini.** The call screen scales toward the pill's target corner (260 ms `ease.in-out`)
  while the pill grows out of the same point; on desktop, the separate mini window appears only after
  the main window has hidden or lost focus (opt-in).
- **Pill drag.** Follows the pointer 1:1; on release snaps to the nearest of 4 corners / 2 side midpoints
  with `spring.soft` (small overshoot); fling velocity > 1200 px/s projects the target in the fling
  direction (proposed). Docking: dragging ≥ 60 % past an edge leaves a 20 px tab.
- **Pill caption line.** Shows only *final* words of the current turn; new words push from the trailing
  edge (text-reveal fade, 120 ms); interim words are not shown in the pill (too jittery at one line).
- **Share frame.** Frame draws on in 240 ms (stroke from the tab outwards); when the adapter says the
  agent read a frame, the tab's dot does one 600 ms pulse (≤ 1 per 1.2 s). Pause turns the frame
  dashed (120 ms) and desaturates it.
- **Preview chip.** Thumbnail crossfades on each sent frame (100 ms); "sent 0.4 s ago" updates at 2 Hz;
  older than 5 s while Sharing → "No frames sent for 5 s" in `warn` (capture stalled).

## 5. Timing
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Button hover / press | 100 ms / 80 + 140 ms | `ease.out` / `spring.snap` | |
| Icon stroke morph (mute, share) | 140 ms | `ease.out` | |
| PTT hold threshold | 200 ms | — | shared with composer |
| PTT cancel distance | 48 px | — | |
| Mute hint | rise 160 ms, hold 3 s, ≤ 1 / 30 s | `ease.out` | Soft mute only |
| Device menu open / close | 180 ms / 120 ms | `ease.out` / `ease.in` | |
| "New" device tag | 10 s | — | |
| Captions region in/out | 160 ms | `ease.out` / `ease.in` | |
| End → summary | 260 ms | `ease.in-out` | `dur.travel` cap |
| Call → mini pill | 260 ms | `ease.in-out` | |
| Pill compact after idle | 4 s | — | |
| Pill snap | ≈ 320 ms | `spring.soft` | small overshoot |
| Keyboard move step | 16 px | — | |
| Share frame draw / remove | 240 ms / 160 ms | `ease.out` / `ease.in` | |
| Share "looking" pulse | 600 ms, ≤ 1 / 1.2 s | `ease.in-out` | |
| Preview crossfade / age text | 100 ms / 2 Hz | linear | |
| Capture-stalled warning | 5 s | — | |
| Ended toast (mini) | 3 s | — | |
| **Reduced motion** | all morphs 0–80 ms crossfades; mic ring becomes a static 4-step level bar at 10 Hz; pill snaps without overshoot (instant move); frame appears without draw-on; "looking" shown as a static "Looking" label for 1.2 s; no fling projection | — | |

## 6. Variants
### Free (7)
| Name | Idea | Surfaces |
|---|---|---|
| `voice-controls/bar` | Full bar: mic, (PTT), devices, CC, share, chat, status + timer, more, End (LiveKit AgentControlBar-like set, note 04) | desktop, voice, mobile |
| `voice-controls/compact` | Icon-only bar for side panels and in-thread voice rows: mic, CC, share, End; rest in more | ide, chat |
| `voice-controls/glyph` | Terminal: `[m]ute [c]aptions [s]hare [q]uit · ● LISTENING 03:07` | terminal |
| `voice-mini/pill` | Floating pill with orb, state, one caption line, mute, end, expand; collapses to compact | desktop, chat, mobile |
| `voice-mini/bubble` | 48 px draggable orb bubble with level ring and mute badge (Gemini Live's draggable circle, note 04) | mobile, desktop |
| `share-overlay/frame` | Accent frame + top tab "Agent can see this · Pause · Stop" around the shared surface | desktop |
| `share-overlay/preview-chip` | In-bar 96 × 60 thumbnail of the last frame actually sent + fps + age | desktop, chat |

### Pro (15)
| Name | One-line idea |
|---|---|
| `voice-controls/dock` | macOS-dock-like bar with magnified hover and labels on hover |
| `voice-controls/radial` | Controls fan around the hero orb on hover/long-press (mute, CC, share, end) |
| `voice-controls/glass` | Frosted bar floating over a camera or shared-screen view |
| `voice-controls/walkie` | Full-width PTT slab on mobile, mute and end at the corners |
| `voice-controls/keypad` | Adds a DTMF keypad sheet for phone-agent calls (Pipecat DTMF keypad, note 04) |
| `voice-controls/dev` | Developer bar: VAD / PTT switch, VAD sensitivity slider, audio playback toggle, logs toggle (OpenAI Realtime console, note 04), latency chip |
| `voice-mini/notch` | Desktop island/notch status: orb dot + state word + level while the app is in the background (pairs with acting-indicator `notch`) |
| `voice-mini/tray` | Menu-bar / tray item with a popover holding the pill |
| `voice-mini/pip` | Picture-in-picture card with orb and two caption lines, resizable |
| `voice-mini/edge-tab` | Always-docked edge tab that slides out on hover |
| `share-overlay/guide` | Agent highlights: box around the object it means and dims the rest (Gemini Live visual guidance, note 04) |
| `share-overlay/redact` | App-defined redaction regions blurred in sent frames, with a "2 areas hidden" chip |
| `share-overlay/region` | Share a dragged rectangle instead of a window; frame handles to resize |
| `share-overlay/cursor-trail` | Agent's attention point (from adapter) shown as a soft trail on the shared view |
| `share-overlay/what-it-saw` | Filmstrip of the frames the agent actually read this turn, each linked to the caption line |

## 7. Edge cases
- **Narrow bars.** < 360 px: status moves into a tooltip; < 280 px: End loses its text; < 240 px: the
  session becomes `voice-mini/pill`. The order of removal is fixed: chat → camera → share → devices →
  CC (into more); mic and End never move.
- **Tiny sizes.** In-thread voice row at 28 px: 20 px icons, 28 px hit areas on desktop, 44 px on touch.
- **Dense IDE panels.** `compact` uses monochrome icons; End is a text button "End" in danger ink.
- **Touch.** 44 × 44 px targets; PTT hold on touch with haptic tick (proposed); drag-to-cancel 48 px;
  long-press bubble opens end/mute menu.
- **Keyboard shortcut conflicts.** Single-letter keys work only when focus is inside the session region
  and not in a text field; Space never acts while a text field has focus. App-global mute
  (`⌘/Ctrl+Shift+M`) is opt-in because it may clash with other apps.
- **Device churn.** Bluetooth headsets switch profiles and reconnect: a device that disappears and
  comes back within 3 s is treated as the same device (no DeviceLost), proposed. Output loss falls back
  to the system default output (no privacy risk) with a toast.
- **No devices / permission denied.** Mic shows disabled-with-reason; the bar offers "Type instead".
- **Screen share and privacy.** Notifications, passwords and other apps can appear in a shared screen;
  `Explain` says so ("Anything on that screen, including notifications"). Window share is suggested
  over full-screen share (picker default where the OS allows choosing).
- **Overlay in the capture.** The frame and tab must be excluded from the captured frames; whether
  GPUI and the browser can exclude a window from capture is **platform-dependent and unverified** —
  where it cannot, the frame is drawn outside the shared window's bounds (window share) or omitted in
  favour of the preview chip (full-screen share).
- **Multi-monitor.** Frame appears on the shared display only; the preview chip shows which display.
- **Shared window minimised / closed.** Minimised → Sharing continues with a stale frame → capture-stalled
  warning at 5 s; closed → Stopped.
- **Slow networks.** Share frame rate drops; chip shows actual fps ("0.3 fps"); never claims a higher rate.
- **Very fast / very slow agent.** Status word follows session state; the bar itself never animates for
  agent speed. The latency chip (live-voice.md) carries speed.
- **RTL.** Bar mirrors (End at the leading edge visually in LTR terms = trailing in reading order);
  device names keep their own direction; pill snapping is unaffected.
- **Mini window across navigation.** The pill is mounted in the app root, not the page, so route
  changes do not remount it; on the web, a full page reload ends the session (stated in docs).

## 8. Accessibility
- **Toolbar semantics:** the bar is a toolbar with roving tab index (←/→ move, Tab leaves); each toggle
  exposes `aria-pressed` (mute, CC, share, camera); End is a plain button.
- **Labels per state** in table 3.1; the call timer is exposed as text ("3 minutes 7 seconds") and not
  announced on change.
- **Live region** (session-level, polite): "Microphone muted / unmuted", "Captions on", "Sharing your
  screen with <agent>", "Sharing paused", "Microphone disconnected" (assertive), "Call ended".
- **Deaf / hard-of-hearing:** CC is never hidden in "more" until the bar is under 240 px wide (then the
  pill keeps a one-line caption); captions default on when the OS caption preference is on; the output
  device menu's Test plays a sound **and** animates its meter; ended-by-agent shows a text toast.
- **Motor:** PTT has a latch alternative (tap to start, tap to stop) in settings for users who can't
  hold a key; drag-to-cancel has a keyboard equivalent (Esc while held).
- **Pill and bubble** are focusable dialogs (`role="dialog"`, label "Voice call with <agent>"); keyboard
  move with arrows; never trap focus; `⌘/Ctrl+Shift+M` (if enabled) works from anywhere.
- **Share overlay:** the tab is reachable with the share hotkey; its buttons have full labels ("Pause
  sharing your screen"); the frame is decorative; the "looking" pulse is paired with a text label.
- **Contrast:** icons ≥ 3:1, text ≥ 4.5:1; muted and sharing states change glyph, not only colour; the
  share frame ≥ 3:1 against typical content (accent with a 1 px dark/light outer stroke).
- **Reduced motion:** see timing table.

## 9. Props sketch
```rust
pub struct VoiceControls {
    pub agent: AgentState,                 // Thinking{NeedsApproval} disables nothing but adds status
    pub snapshot: VoiceSnapshot,           // live-voice.md §8: state, overlays, levels, turn_mode, latency
    pub variant: ControlsVariant,          // Bar | Compact | Glyph | Pro(..)
    pub controls: ControlSet,              // which buttons exist: mic, ptt, devices, captions, share, camera, chat, more, end
    pub devices: DeviceList,               // inputs/outputs with ids, names, is_default, is_new
    pub selected: DeviceChoice,            // persisted by the app via on_device_change
    pub mute_kind: MuteKind,               // Hard (default) | Soft
    pub ptt: PttConfig,                    // hold_threshold 200 ms, cancel_distance 48 px, latch: bool, key: Option<Keystroke>
    pub hotkeys: ControlHotkeys,           // M, C, T, ⇧S, V, Space(PTT), app-global mute opt-in
    pub labels: VoiceLabels,
    pub motion: MotionLevel,
    pub on_mute: Callback<bool>,
    pub on_ptt: Callback<PttEvent>,        // Down | Up | Cancel
    pub on_device_change: Callback<DeviceChoice>,
    pub on_captions: Callback<bool>,
    pub on_share: Callback<ShareCommand>,  // Start | Pause | Resume | Stop | ChangeSource
    pub on_end: Callback<EndReason>,
}

pub struct VoiceMini {
    pub snapshot: VoiceSnapshot,
    pub variant: MiniVariant,              // Pill | Bubble | Pro(..)
    pub placement: MiniPlacement,          // Corner(..) | Edge(..) | Docked(..); persisted
    pub window: MiniWindowMode,            // InApp (default) | OsWindow { always_on_top: bool } (desktop opt-in)
    pub caption_line: Option<String>,      // latest final words (live-captions.md)
    pub orb: VoiceOrbVariant,
    pub motion: MotionLevel,
    pub on_expand: Callback<()>, pub on_mute: Callback<bool>, pub on_end: Callback<EndReason>,
}

pub enum ShareState {
    Explain, Picking, Denied, Starting,
    Sharing { source: ShareSource, fps: f32, last_sent: Instant, looking_until: Option<Instant> },
    Paused { reason: PauseReason },        // User | UserControl (family G) | Sensitive (app signal)
    Stopped,
}

pub struct ShareOverlay {
    pub state: ShareState,
    pub variant: ShareVariant,             // Frame | PreviewChip | Pro(..)
    pub preview: Option<ImageHandle>,      // last frame actually sent (downscaled by the adapter)
    pub redactions: Vec<Rect>,             // Pro redact; app-supplied
    pub exclude_from_capture: CaptureExclusion, // Supported | DrawOutside | Omit (platform-detected)
    pub labels: ShareLabels,
    pub motion: MotionLevel,
}
```

## 10. Related components and references
**Related:** [live-voice.md](live-voice.md) · [voice-orb.md](voice-orb.md) (pill/bubble orb) ·
[voice-visualisers.md](voice-visualisers.md) (`mic-ring`, bars) · [live-captions.md](live-captions.md)
(caption region, pill line) · [composer-voice-input.md](composer-voice-input.md) (PTT threshold,
device errors) · [takeover-banner.md](takeover-banner.md) and [acting-indicator.md](acting-indicator.md)
(control vs viewing; AutoPaused while the user has control) · [elapsed-timer.md](elapsed-timer.md)
(call timer) · family J chrome (tray, notch, floating HUD).

| Reference | What we take | Licence note |
|---|---|---|
| LiveKit AgentControlBar (note 04) | button set, device selector, persisted device choices, StartAudio | Not in sources.csv; licence not checked — study only |
| Gemini Live (note 04) | floating pill → draggable circle, captions toggle, Hold/resume, screen share with system confirm, visual guidance box + dim | Product — reference only |
| ChatGPT voice / agent (note 04) | End, mute, camera; screenshots paused while user controls | Product — reference only |
| Meta AI (note 04) | mic-in-use icon while multitasking | Product — reference only |
| Pipecat Voice UI Kit (note 04) | device controls, bot volume, DTMF keypad | Not in sources.csv; licence not checked — study only |
| OpenAI Realtime console (note 04) | VAD vs push-to-talk toggle, logs toggle | Not in sources.csv; licence not checked — study only |
| ElevenLabs UI Mic Selector (note 04) | mic picker as its own component | Not in sources.csv; licence not checked — study only |
| Cuelume [R110] | output "Test" tone and optional end earcon | MIT per licensing.md — port with attribution |
| seanaiux dashboard [R013] | background pill docked bottom-centre (pill placement idea) | Study only |
