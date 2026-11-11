**Status:** draft v0.1 (2026-11-12).

# Spec: live captions and word-synced transcript (`captions`, `voice-transcript`)

Part of family K ([live-voice.md](live-voice.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **5 free + 14 Pro = 19 variants**
(`captions` 2 + 6, `voice-transcript` 3 + 8). Numbers are Oxyn **proposals** unless a source is cited.

## 1. Purpose
Speech disappears; text stays. During and after a call the user needs to **read what was said**: to
follow in a noisy or silent room, because they are deaf or hard of hearing, to check that the agent
heard them right, to see exactly where they cut the agent off, and to copy what the agent said.

Two components share one word stream:
- **`captions`** — the live overlay: 1–2 lines of the *current* speech, under the orb or over a camera /
  shared-screen view. Gemini Live has a captions toggle and shows transcript text above a waveform in its
  floating pill (note 04); Pipecat has a Transcript Overlay (note 04).
- **`voice-transcript`** — the scrollable record: every turn with speaker labels and timestamps,
  word-level sync with the agent's audio, interim vs final words, barge-in markers, scroll lock and copy.
  ChatGPT streams the live transcript into the chat thread (note 04); LiveKit's AgentChatTranscript
  shows voice + text, timestamps, a thinking indicator and "scroll to end"; ElevenLabs' Transcript Viewer
  highlights words synced to audio; the OpenAI realtime-agents demo shows tool calls and agent handoffs
  as expandable rows; Notion meeting notes link summaries to transcript timestamps (note 04). Note 04
  gap 10 names the missing pieces: a voice-turn message type with an interrupted marker.

Rules (proposed):
1. **Agent words follow the audio by default** (`AgentText::Synced`): a word appears when it is
   *heard*, not when it is generated, so the text never claims the agent said something it did not.
   `AsGenerated` (text ahead of audio, dimmed) is a user setting, useful for deaf users who want lower
   latency.
2. **Interim words are visibly provisional** (60 % ink, same as composer dictation) and may change in
   place; **final words never move backwards** once settled.
3. **Barge-in leaves a marker at the last heard word**; the unspoken remainder is kept but collapsed
   and never presented as said.
4. **Scroll lock:** follow live only while the user is at the bottom; reading history is never yanked.
5. **Copy never includes interim words**, and includes unspoken text only when explicitly asked.
6. **Captions never cover the controls or the orb's centre**; they move above the control bar.

### Word model
```rust
pub struct Word {
    pub id: WordId,
    pub text: String,              // includes trailing punctuation; whitespace handled by the renderer
    pub speaker: SpeakerId,        // User | Agent(AgentId) | Remote(ParticipantId)
    pub start_ms: Option<u64>,     // session clock; from STT (user) or TTS alignment (agent)
    pub end_ms: Option<u64>,
    pub timing: Timing,            // Measured | Estimated
    pub state: WordState,          // see below
    pub confidence: Option<f32>,   // STT only
}
// User:  Interim ──▶ Final            (Interim may be Replaced in place any number of times)
// Agent: Pending ──▶ Speaking ──▶ Spoken
//                 └─ barge-in / network ─▶ Unspoken
```
**Estimated timing** (adapter gives utterance audio but no word times, proposed): distribute the
utterance's duration over its words by character count, minimum 120 ms per word; such turns are marked
`timing: Estimated` and the karaoke highlight moves by phrase (comma / clause) instead of by word.

## 2. State machine

### 2.1 Turn and word flow
```
  Off ── captions on / transcript mounted ──▶ Waiting ("Captions will appear when someone speaks")
                                                  │ first word
                                                  ▼
  ┌──────────── UserTurn{interim} ── words finalised ──▶ UserTurn{final} ── end-of-turn ──┐
  │                 ▲  (interim replaced in place)                                       │
  │                 │                                                                    ▼
  │                 │                                          AgentPending ("…" thinking row; tool / handoff rows)
  │                 │                                                                    │ first audible word
  │                 │                                                                    ▼
  │                 │                              AgentTurn{speaking, current_word} ─ playout ends ─▶ AgentTurn{done}
  │                 │                                     │ barge-in                                     │
  │                 │                                     ▼                                              │
  │                 └─────────────────── AgentTurn{interrupted at word n, k unspoken} ◀──────────────────┘ (next turn)
  │
  Gaps: Reconnecting → GapMarker("Connection lost 0:04") · OnHold → HoldMarker · Asleep → nothing
  Errors: CaptionsUnavailable (STT down / not supported) → banner, transcript keeps typed turns
  Ended → Archived (static transcript; copy/export; Pro: audio scrub)
```

### 2.2 Scroll and selection (transcript)
```
  Following ── user scrolls up > 48 px ──▶ Locked{new: n} ("Jump to live ↓ · n new") ── Jump / End / scroll to bottom ──▶ Following
  Following/Locked ── text selection starts ──▶ Selecting (updates above/inside selection frozen; new words append below)
  Selecting ── selection cleared ──▶ previous state (frozen updates applied in one frame)
  Following ── window hidden ──▶ Paused-render (data kept) ── visible ──▶ Following (jump, no animation)
```

### 2.3 Captions line flow
```
  Empty ── words ──▶ Line1 filling ── line full (≤ 42 chars) or phrase break ──▶ Line1 + Line2 filling
  Line2 full ──▶ RollUp (line 1 scrolls out, line 2 becomes line 1)
  Speaker change ──▶ new line with label (never mix speakers on a line)
  Silence ≥ 4 s ──▶ fade lines out (keep the last final line in the mini pill)
  Barge-in ──▶ "↯" appended to the agent line, user line starts on the next line
```

## 3. States tables

### 3.1 `captions`
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Off` | nothing (CC button off) | — | — | `C` toggles (control bar) |
| `Waiting` | one muted line "Captions will appear when someone speaks" (only for 3 s after turning on) | — | — | — |
| `User{interim}` | label "You" + words, interim at 60 % ink | — | — | — |
| `User{final}` | same, 100 % ink | — | — | — |
| `AgentPending` | label + "…" (three still dots, or Pro shimmer) | — | — | — |
| `Agent{speaking}` | label + spoken words; current word underlined 2 px (Synced) | — | — | — |
| `Agent{interrupted}` | "… so the next step is ↯" (marker + "interrupted" label for 1.2 s) | — | — | — |
| `Gap` | "Reconnecting…" line in muted ink | — | — | — |
| `Unavailable` | "Captions unavailable · <reason>" | — | Retry (if adapter can) | — |
| `Idle` (silence ≥ 4 s) | faded out | — | — | — |

### 3.2 `voice-transcript`
| Element / state | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| Turn row (user) | "You" label · timestamp `03:07` · words (interim at 60 %); low-confidence words dotted-underlined (if enabled) | copy turn (hover) | edit & resend as text (app) | ↑/↓ move between turns, `c` copy turn |
| Turn row (agent) | agent label + avatar/orb glyph · timestamp · words; current word highlighted while speaking | copy turn | play from here (Pro, if audio kept) | as above, `Enter` play (Pro) |
| Interrupted agent turn | spoken words · `↯ You interrupted` chip · "18 words not spoken ▸" collapsed | expand unspoken | copy (spoken only) | `Enter` on chip expands |
| Unspoken (expanded) | remainder in 50 % ink italic with a left rule, label "Not spoken" | collapse | copy including unspoken | `Enter` |
| Tool / handoff row | "Searched the web · 3 results ▸" / "Handed to Billing agent" (one line, expandable) | expand | — | `Enter` |
| Thinking row | "Thinking…" with text shimmer ([text-shimmer.md](text-shimmer.md)); after 1.2 s "Still thinking…" | — | — | — |
| Gap marker | `⌁ Connection lost · 0:04` (network) or `⏸ On hold · 1:12` | — | — | — |
| Typed turn | same row with a ⌨ glyph ("typed") | copy | — | — |
| Header | title, call timer, `Copy transcript ▾` (Plain · With timestamps · Markdown), search (Pro) | copy | export | `⌘/Ctrl+Shift+C` copy all (proposed) |
| `Following` | newest at bottom, auto-scroll | — | — | `End` stays |
| `Locked{n}` | pill "Jump to live ↓ · n new" bottom-centre | jump | — | `End` jumps |
| `Selecting` | updates frozen inside selection | copy | — | standard selection keys |
| `Archived` | static; Pro: scrub bar and play-from-word | copy / export | play | — |

## 4. Micro-interactions
- **Word arrival (user).** Interim words blur-in 4 px → 0 at 60 % opacity (140 ms, 30 ms stagger, cap 6
  words — composer-voice-input.md values). A replaced interim word crossfades in place (120 ms); if
  the replacement is wider, the line reflows with `spring.snap` (no jump). Final: opacity 0.6 → 1
  (120 ms), no movement.
- **Word arrival (agent, Synced).** Each word appears at its `start_ms` (playout-aligned) with a 100 ms
  fade (no blur: it must keep pace with speech at ~3 words/s). The current word carries a 2 px underline
  in accent that slides to the next word (80 ms `ease.out`); in `AsGenerated` mode, pending words show at
  45 % ink ahead of the underline. With `timing: Estimated`, the underline spans the current phrase.
- **Barge-in.** At the cut: the underline turns into a `↯` glyph (120 ms morph) after the last *heard*
  word; any words already shown ahead (AsGenerated) dim to 30 % and collapse into the "n words not
  spoken" chip (200 ms height collapse, `ease.in-out`). The "You interrupted" label shows 1.2 s in
  captions and stays as a chip in the transcript.
- **False barge-in resume** (if the app resumes): the chip changes to "Paused briefly" and the agent's
  words continue on the same row.
- **Speaker change.** New row (transcript) or new line (captions) with the label fading in 120 ms; labels
  use a 3 px speaker colour bar **and** the name.
- **Caption roll-up.** When line 2 fills, line 1 slides up and fades (160 ms `ease.out`); minimum
  display per line 1000 ms (a line is never rolled out before it has been visible 1 s; if speech is
  faster, the region grows to a 3rd line temporarily, proposed).
- **Silence fade.** After 4 s with no words, caption lines fade out over 400 ms.
- **Jump to live.** Pill rises 8 px + fade (160 ms) when Locked; counter updates with a digit roll; click
  scrolls to the bottom with a 260 ms `ease.in-out` scroll (instant if > 3 screens away).
- **Copy.** Turn copy button appears on hover / focus (100 ms); on copy: ✓ morph 160 ms and "Copied" for
  1.2 s. Copy-all formats: Plain (`You: …` / `Agent: …`), With timestamps (`[03:07] You: …`), Markdown
  (bold labels, interruption as `*(interrupted)*`).
- **Thinking row.** Appears only if AgentPending lasts ≥ 300 ms; replaced in place by the agent turn's
  first word (no remount).
- **Low-confidence words** (Pro `confidence-ink`): dotted underline at confidence < 0.6 (proposed); click
  to fix → becomes an edit of the user turn (app decides whether to resend).

## 5. Timing
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Interim word blur-in | 140 ms, 30 ms stagger, cap 6 | `ease.out` | blur 4 → 0, opacity 0 → 0.6 |
| Interim replace | 120 ms | crossfade; reflow `spring.snap` | |
| Interim → final | 120 ms | `ease.out` | opacity only |
| Agent word fade (Synced) | 100 ms at `start_ms` | `ease.out` | playout-aligned |
| Current-word underline move | 80 ms | `ease.out` | |
| Barge-in ↯ morph | 120 ms | `ease.firm` | |
| Unspoken collapse | 200 ms | `ease.in-out` | |
| "Interrupted" caption label | 1200 ms | — | |
| Speaker label in | 120 ms | `ease.out` | |
| Caption roll-up | 160 ms | `ease.out` | min line display 1000 ms |
| Caption silence fade | after 4 s, 400 ms | `ease.in` | |
| Thinking row show delay / filler | 300 ms / 1200 ms | — | live-voice.md |
| Jump pill in / scroll | 160 ms / 260 ms | `ease.out` / `ease.in-out` | instant > 3 screens |
| Follow threshold | 48 px from bottom | — | |
| Copy ✓ / "Copied" | 160 ms / 1200 ms | `spring.snap` | |
| Frozen-selection flush | 1 frame | — | |
| **Reduced motion** | words appear with no blur, fade or stagger (instant at 0–80 ms opacity); current word shown by a **static** bold weight, no sliding underline; roll-up is an instant swap; barge-in marker appears instantly; jump scrolls instantly | — | text is never delayed for motion |

## 6. Variants
### Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `captions/overlay` | Two roll-up lines on a 70 % scrim under the orb or over a camera/shared view; speaker label + colour bar | voice, mobile, desktop |
| `captions/strip` | One full-width line above the control bar, newest words on the trailing edge; used in in-thread voice and the side panel | chat, ide |
| `voice-transcript/thread` | Each turn becomes a chat message (voice-turn message type) with timestamp, ↯ chip and unspoken disclosure (ChatGPT in-thread transcript, note 04) | chat |
| `voice-transcript/panel` | Dense list for side panels: label column, timestamp column, words; tool and handoff rows inline (realtime-agents style, note 04) | ide, desktop |
| `voice-transcript/glyph` | Terminal: `03:07 you  │ book a table for…` / `03:09 agent│ sure, which… ↯` with interim in dim | terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `captions/karaoke` | Large single line, current word highlighted with a sweeping fill (ElevenLabs word-level highlight idea, note 04) |
| `captions/cinema` | Broadcast-style captions with user-tunable size, font and background (follows OS caption settings) |
| `captions/bubble` | Speech-bubble captions that pop from the orb (agent) or the bottom (user) |
| `captions/translate` | Two lines per utterance: original and translated, language tags on each |
| `captions/focus` | Only the agent's words, extra large, for low-vision and kiosk use |
| `captions/ticker` | Single scrolling ticker for the mini pill and notch |
| `voice-transcript/scrub` | Archived transcript with a scrub bar; click a word to play from it (ElevenLabs Transcript Viewer + Scrub Bar, note 04) |
| `voice-transcript/confidence-ink` | Low-confidence user words dotted-underlined, click to fix |
| `voice-transcript/expressions` | Per-utterance top-3 expression scores under user turns (Hume EVI, note 04), labelled as estimates |
| `voice-transcript/summary-links` | Post-call summary whose bullets link to transcript timestamps (Notion meeting notes, note 04) |
| `voice-transcript/search` | In-transcript search with hit markers on the scroll track |
| `voice-transcript/two-column` | User left, agent right, aligned on a shared timeline (overlaps visible in full-duplex) |
| `voice-transcript/events` | Developer view: transcript interleaved with ▲/▼ event rows, latency per turn (OpenAI Realtime console, note 04) |
| `voice-transcript/what-it-saw` | Frames the agent read during screen share inline beside the words spoken at that moment |

## 7. Edge cases
- **Long utterances.** Captions never exceed 3 lines; transcript turns longer than 12 lines collapse to
  8 lines + "Show all" while a newer turn exists.
- **Fast speech.** If words arrive faster than lines can be shown for 1 s each, the region grows to 3
  lines; beyond that, oldest line rolls anyway (readability beats completeness; the transcript has all).
- **Overlap / full-duplex.** Simultaneous speakers get separate lines (captions) or interleaved rows by
  `start_ms` (transcript); the `two-column` Pro variant shows overlap directly.
- **Interim churn.** Recognisers can rewrite the last several words; replacements are limited to the
  current interim span, so final text above it never moves.
- **No word timings.** Estimated timing (§1) and phrase-level highlight; marked in the archived
  transcript so "play from word" is approximate.
- **Agent text that is never spoken** (tool output, code blocks, links): shown in the transcript as an
  attached card under the agent turn ("Shown, not spoken"), never inside captions.
- **Numbers and spelling.** STT may write "two" or "2"; the component shows what the adapter gives; no
  normalisation (proposed).
- **RTL and mixed scripts.** Each turn gets its own direction from its language (bidi isolation per
  turn); labels and timestamps sit at the start edge of the turn's direction; captions align per line.
- **CJK.** Word segmentation via `Intl.Segmenter` on web and an ICU segmenter on GPUI (as in
  text-reveal.md); highlight moves per segment; 42-character line limit becomes 24 full-width characters
  (proposed).
- **Tiny sizes.** In a 28 px voice row the strip shows one line, 12 px text, no labels (speaker by colour
  bar + `You:` prefix). Below 160 px wide, captions move to the mini pill's ticker.
- **Dense IDE panels.** `panel` variant: 12 px text, timestamp column 40 px, tool rows collapsed.
- **Touch.** Long-press a turn for copy / play; jump pill 44 px tall.
- **Slow networks.** User words may finalise late; the turn stays at 60 % until final; network gaps get
  a marker, never silent missing text.
- **Very fast agent.** Thinking row skipped (< 300 ms). **Very slow agent:** thinking row shows "Still
  thinking…" at 1.2 s and an elapsed counter after 10 s ([elapsed-timer.md](elapsed-timer.md)).
- **Privacy.** Transcripts are app data; Oxyn keeps nothing. Archived audio for `scrub` exists only if
  the app records and passes it.
- **Profanity / sensitive words.** Masking is the adapter's job; captions render what they receive.

## 8. Accessibility
- **Deaf and hard-of-hearing users (primary audience):**
  - Captions on by default when the OS caption preference is on; caption styling follows OS caption
    settings where the platform exposes them (size, colour, background, opacity); user setting
    `AgentText::AsGenerated` for lower latency.
  - Every spoken event has a text form: speaker changes (labels), interruptions (↯ chip), connection
    loss and hold (markers), agent ended the call (final row "Agent ended the call").
  - Non-speech sounds the adapter reports (e.g. `[music]`, `[background noise]`, `[agent paused]`) are
    shown in brackets in muted ink.
  - Minimum caption size 16 px (20 px in `call` layout), line length ≤ 42 characters, contrast ≥ 7:1 on
    the scrim (AAA for captions, proposed), speaker identified by name, not only colour.
  - Typed turns appear in the same transcript (live-voice.md rule 5).
- **Screen readers.** The transcript is a `log` region (`role="log"`, polite): only **final** user
  turns and **completed** agent turns are announced (one announcement per turn, not per word);
  interim and per-word updates are suppressed. Captions overlay is `aria-hidden` when the transcript is
  present (avoid double reading); if captions are the only text surface, they announce per completed
  line. Unspoken text is announced as "Not spoken: …" only when expanded.
- **Keyboard.** Transcript rows are focusable list items (↑/↓, Home/End); `c` copies the focused turn;
  `End` jumps to live; the jump pill is a button "Jump to latest, 3 new turns".
- **Cognitive load.** Interim text is visually distinct (60 % ink) and never jumps lines; the
  "interrupted" chip uses words, not only a glyph.
- **Contrast.** Text ≥ 4.5:1 (≥ 7:1 captions on scrim); interim 60 % ink must still meet 4.5:1 (darken
  base ink if needed, composer rule); current-word highlight ≥ 3:1 and also bold in reduced motion.
- **Reduced motion.** See timing table; text is never delayed for an animation.

## 9. Props sketch
```rust
pub struct Captions {
    pub agent: AgentState,                 // AgentPending row, stall
    pub session: VoiceSessionState,        // Interrupted, Reconnecting, OnHold
    pub words: WordStream,                 // shared ring of Word (live tail only)
    pub variant: CaptionsVariant,          // Overlay | Strip | Pro(..)
    pub lines: u8,                         // 1..=3, default 2
    pub max_chars: u16,                    // 42 (24 for full-width scripts)
    pub min_line_ms: u32,                  // 1000
    pub silence_fade_after: Duration,      // 4 s
    pub agent_text: AgentText,             // Synced (default) | AsGenerated
    pub style: CaptionStyle,               // FromOs (default) | Custom { size, bg, opacity, font }
    pub labels: SpeakerLabels,             // "You", agent names
    pub motion: MotionLevel,
}

pub struct VoiceTranscript {
    pub agent: AgentState,
    pub session: VoiceSessionState,
    pub turns: Vec<Turn>,                  // Turn { speaker, words, started_at, kind: Spoken | Typed, interrupted: Option<Interruption>, attachments }
    pub events: Vec<TranscriptEvent>,      // Tool, Handoff, Gap, Hold, ThinkingRow
    pub variant: TranscriptVariant,        // Thread | Panel | Glyph | Pro(..)
    pub agent_text: AgentText,
    pub follow: FollowPolicy,              // threshold 48 px; Following | Locked
    pub show_confidence: Option<f32>,      // Pro: dotted underline below this
    pub timestamps: TimestampStyle,        // Relative (03:07, default) | WallClock | Hidden
    pub copy_formats: Vec<CopyFormat>,     // Plain | WithTimestamps | Markdown
    pub audio: Option<AudioSourceHandle>,  // Pro scrub/play, only if the app kept audio
    pub labels: TranscriptLabels,
    pub motion: MotionLevel,
    pub on_copy: Callback<CopyEvent>,
    pub on_play_from: Option<Callback<WordId>>,
    pub on_fix_word: Option<Callback<(WordId, String)>>,
}

pub struct Interruption {
    pub at_word: Option<WordId>,           // last heard word
    pub unspoken: Vec<WordId>,
    pub cause: InterruptCause,             // User | Button | Network
    pub resumed: bool,                     // false barge-in resume
}
```

## 10. Related components and references
**Related:** [live-voice.md](live-voice.md) (states, latency, barge-in) ·
[voice-control-bar.md](voice-control-bar.md) (CC toggle, mini pill caption line) ·
[voice-orb.md](voice-orb.md) · [text-reveal.md](text-reveal.md) (pacer/effect split, segmenters) ·
[text-shimmer.md](text-shimmer.md) (thinking row) · [message-bubble.md](message-bubble.md) (thread
variant hosts voice turns) · [tool-call.md](tool-call.md) (tool rows) ·
[composer-voice-input.md](composer-voice-input.md) (interim/final ink values) ·
[elapsed-timer.md](elapsed-timer.md).

| Reference | What we take | Licence note |
|---|---|---|
| ElevenLabs UI Transcript Viewer, Scrub Bar (note 04) | word-level highlight synced to audio; scrub | Not in sources.csv; licence not checked — study only |
| LiveKit AgentChatTranscript (note 04) | voice + text, timestamps, thinking indicator, scroll to end | Not in sources.csv; licence not checked — study only |
| OpenAI realtime-agents demo (note 04) | tool-call and agent-handoff rows in the transcript | Not in sources.csv; licence not checked — study only |
| Pipecat Transcript Overlay (note 04) | caption overlay as its own component | Not in sources.csv; licence not checked — study only |
| ChatGPT in-thread voice, Gemini Live (note 04) | live transcript in the thread; captions toggle; text above waveform | Products — reference only |
| Hume EVI (note 04) | per-utterance expression scores | Product — reference only |
| Notion AI Meeting Notes (note 04) | summaries linked to transcript timestamps | Product — reference only |
| Arc UI speech player (note 01) | karaoke word highlight | Paid — study only |
| DopeOblivion voice hero [R020] | conversation lines appear with a short blur-in | Study only |
| David Visuals AI Assist [R019] | trailing-fade streaming (last words lighter) — basis for `AsGenerated` dimming | Study only |
