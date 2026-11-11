**Status:** draft v0.1 (2026-11-11).

# Spec: verb cycler (`verb-cycler`)

Part of family C ([status-text.md](status-text.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **5 free + 17 Pro = 22 variants**.
Content comes from verb packs (status-text.md §5); this spec is the **motion and timing** of
changing one status word into another, including the final roll into past tense.

## 1. Purpose
During a long turn the status word is the only thing that changes. If it never changes the app looks
frozen; if it changes too often it is noise; if it changes to a cute word while a real tool is
running it hides information. Claude Code picks a random verb from a large list and ends with a
past-tense line ("Cooked for 1m 23s"); Codex shows a model-written header instead of a fixed verb;
Cursor shows "Planning next moves" and "Taking longer than expected"; Zed types its label in; Arc
crossfades a step verb to past tense; affectUI rolls phase words and locks to a check (notes 08, 11,
12); Pitus rolls "Thinking 13s" to "Thought for 14s" [R021]. The cycler unifies these:

1. **Cycle** flavour verbs from a pack while nothing more specific is known.
2. **Pin** the real activity the moment the agent reports one (tool, model header), without flicker.
3. **Settle** into a past-tense summary when the work ends.

## 2. State machine
```
                 activity starts                       dwell elapsed (and not pinned)
  Idle ─────────────────────────▶ Showing{activity verb} ─────────────────────▶ Swapping{out, in}
    ▲                               │  ▲      ▲                                   │ transition done
    │                               │  │      └───────────────────────────────────┘ Showing{pack verb}
    │                               │  │
    │     tool / header event ──────┼──┼──▶ PendingPin (pin_show 300 ms debounce) ──▶ Pinned{verb, target}
    │     (event ends < 300 ms: dropped, never shown)                                   │ event ends:
    │                               │  │                                                │ hold ≥ pin_min 900 ms,
    │                               │  └──────────── unpin: Swapping → Showing ◀───────┘ then fresh dwell
    │                               │
    │                AwaitingApproval ▼                       Stalled(hard)
    │                          Paused("Waiting for you")      Frozen (no swaps; text escalates via status-text)
    │                               │ answered → Showing
    │
    │◀─ host hides ─ Settled{past} ◀── Completing (past-tense roll) ◀── Done (from any state)
    │◀─ host keeps ─ Settled("Interrupted after 42s" | "Stopped · {kind}") ◀── Cancelled / Error
  Reduced motion: same states; every Swapping is an 80 ms crossfade; dwell × 2.
```

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle` | nothing | — | — | — |
| `Showing{verb}` | one verb (activity verb first, then pack verbs) | host's (click → live step) | tooltip: full verb + target | host's |
| `Swapping` | transition between two verbs; accessible name is already the *incoming* verb | — | — | — |
| `PendingPin` | still the previous verb (debouncing) | — | — | — |
| `Pinned{verb, target}` | "Reading auth.rs", or model header | click → expands that tool row | copy target | Enter when host focusable |
| `Paused` | "Waiting for you" (static) | host → focus approval card | — | host's |
| `Frozen` | last verb, no swaps | — | — | — |
| `Completing` | past-tense roll | — | — | — |
| `Settled{past}` | "Worked for 1m 12s", "Thought for 12s", "Read auth.rs" | host → expand summary | — | — |

## 3. Selection and dwell rules
| Rule | Value (proposed) | Why |
|---|---|---|
| First verb of a turn | the activity verb ("Thinking", "Working"), never a flavour verb | first impression must inform |
| Dwell per verb | `max(2800 ms, 1200 ms + 70 ms × graphemes)`; CJK counts graphemes × 1.6 | long verbs need reading time |
| Min gap after a state change | 1200 ms before the next flavour swap | avoids double changes |
| Order | `shuffle`: a shuffle bag of the pack's 12 verbs; no repeat until the bag empties; first of a new bag ≠ last of the old one. `sequential`: pack order, wraps | random without repeats feels alive; sequential suits narrative packs |
| Seed | hash of run id | snapshot tests and multi-window consistency |
| Long runs | after 5 min dwell × 1.5 | calmer for background runs |
| Pin debounce `pin_show` | 300 ms | quick tool bursts never flash (Cline uses a 500 ms grace; Antigravity fixed flicker between quick tool names — note 08) |
| Pin hold `pin_min` | 900 ms once shown | a shown verb never disappears unread |
| Concurrent tools | "Running 3 tools" (count) | a list would flicker |
| Model-written header | pins over everything except tools; min 1500 ms between header swaps | Codex pattern (note 08) |
| Cycling off | `cycle = false` → only pinned/activity verbs ever change the text | user and app setting |
| Unfocused window | no swaps (the clock pauses); on focus, resume dwell where it was | nothing changes unseen |

## 4. Micro-interactions
- **Swap.** Outgoing and incoming verbs are measured before the swap; the slot width follows with
  `spring.snap` (status-text.md §2.2). Any shimmer drops out for the swap and restarts after its delay.
- **Pin.** Pinning uses the same transition as cycling, but the *target* (file name, query) slides in
  80 ms after the verb so the eye reads verb then object.
- **Re-pin to the same verb** with a new target ("Reading a.rs" → "Reading b.rs") only swaps the
  target (fade 120 ms); the verb stays put.
- **Unpin.** After `pin_min`, back to the flavour cycle with a fresh dwell; if the run is still in a
  tool chain, the next pin usually arrives first — no flavour verb is squeezed in under 1200 ms.
- **Past-tense completion.** The present verb rolls to past using the variant's transition, capped at
  `dur.tense.roll` 240 ms (scramble / split-flap capped at 400 ms). Shared text (the target, "for")
  stays still; only the changed part moves (morph of the common suffix/prefix). Duration digits are
  frozen at the completion event, then appear with the elapsed-timer's summary style.
- **Failure.** No playful transition: 120 ms crossfade to "Ran cargo test · 3 failed" in
  `tone.danger`.
- **Interrupt.** The cycler stops mid-swap if needed (incoming text wins, 80 ms) and settles on
  "Interrupted after 42s"; the status strip owns the drop into the transcript.
- **Hover.** Tooltip after 500 ms with full verb + target; hover never pauses cycling (it would make
  the tooltip lie a moment later — the tooltip updates live instead).

## 5. Transition catalogue and timing
| Transition | Out | In | Easing / spring | Stagger | Frame rate | Notes |
|---|---|---|---|---|---|---|
| `fade` | 80 ms, y 0 → −4 px | 140 ms, y +4 → 0 | `ease.in` / `ease.out` | — | 30 Hz | total ≈ `dur.verb.swap` 180 ms (overlap 40 ms) |
| `slide` | 160 ms, up 0.6 em | 200 ms, from +0.6 em | `ease.in` / `ease.out` | — | 60 Hz | clipped to the line box |
| `blur-swap` | 120 ms, blur 0 → 6 px | 180 ms, blur 6 → 0 px | `ease.out` | — | 60 Hz | terminal falls back to `fade` |
| `ellipsis` | as `fade` | as `fade` | — | dots 300 ms each | 30 Hz | three dot cells fade in 0.3 → 1, hold 300 ms, reset (1200 ms loop); grouped with the verb |
| `static` | 80 ms crossfade | — | linear | — | 30 Hz | no cycling; pinned changes only |
| `scramble` | — | positions resolve L→R: random glyphs every 40 ms, position *i* settles at 120 ms + 25 ms·i | linear | 25 ms, cap 300 ms | 30 Hz | glyph set matches case and script (§7) |
| `scramble-braille` | — | as scramble with braille noise `⠁…⣿` | — | 25 ms | 30 Hz | terminal-friendly |
| `scramble-kana` | — | as scramble with katakana noise | — | 25 ms | 30 Hz | ja locale or opt-in |
| `decrypt-hex` | — | noise in `0-9a-f`, monospace only | — | 20 ms | 30 Hz | coder pack |
| `split-flap` | — | each position flips through ≤ 3 intermediate characters; each flip = 2 × 60 ms halves | `ease.in` then `ease.out` | 30 ms, cap 600 ms total | 60 Hz | uses fixed monospace cells |
| `odometer` | — | each position rolls vertically old → new char | `ease.out`, 320 ms | 20 ms | 60 Hz | 18 % top/bottom mask fade (OpenCode AnimatedNumber, MIT) |
| `flip-3d` | rotateX 0 → 90°, 120 ms | −90° → 0, 160 ms | `ease.in` / `spring.snap` | — | 60 Hz | perspective 400 px; desktop/web only |
| `morph` | non-shared glyphs fade 140 ms | shared glyphs glide to new x | `spring.snap` | — | 60 Hz | shared = longest common subsequence |
| `typewriter-replace` | backspace 18 ms/char (cap 250 ms) | type 28 ms/char (cap 450 ms) | linear | — | 60 Hz | caret visible during |
| `kinetic-push` | pushed out −100 % x | pushes in from +100 % x | `spring.soft` | — | 60 Hz | 220 ms settle |
| `glitch-rgb` | 3 frames channel split ±1.5 px | settle | 30 ms frames | — | 30 Hz | once per swap max (flash safety) |
| `drum` | cylinder roll up | 260 ms | `ease.out` | — | 60 Hz | 3D drum with mask fade |
| `stack-peek` | current slides up | next verb waits below at 35 % opacity | `ease.out` 200 ms | — | 60 Hz | peek is decorative (`aria-hidden`) |
| `ticker-tape` | — | continuous horizontal marquee at 24 px/s, " · " separators | linear | — | 30 Hz | wide status bars; pauses on hover |
| `letter-shuffle` | letters of the old word travel to their slot in the new word; leftovers drop 4 px | 260 ms | `spring.snap` | 15 ms | 60 Hz | |
| `wave-in` | fade 100 ms | glyphs rise 3 px, 160 ms each | `ease.out` | 30 ms, cap 300 ms | 60 Hz | |
| `cascade-drop` | glyphs drop 4 px + fade | new glyphs fall from −4 px | `ease.in` / `ease.out` | 20 ms | 60 Hz | |
| Past-tense roll (any) | — | variant transition capped at 240 ms (400 ms for scramble / split-flap) | — | — | 60 Hz | — |
| Width follow | — | ≤ 200 ms | `spring.snap` | — | 60 Hz | starts after the outgoing fade |
| **Reduced motion** | — | every transition → 80 ms opacity crossfade; ellipsis static "…"; ticker-tape becomes a static list in the tooltip; dwell × 2 | linear | none | 30 Hz | — |

## 6. Variants

### Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `verb-cycler/fade` | Default 180 ms fade with a 4 px lift; same as the status strip's verb swap | all |
| `verb-cycler/slide` | Old verb slides up and out, new slides up from below; clipped | chat, desktop, mobile |
| `verb-cycler/blur-swap` | Blur out, blur in — soft, good on glass surfaces | chat, desktop, mobile |
| `verb-cycler/ellipsis` | Any swap + an animated three-dot suffix grouped with the verb | all (terminal uses static "…") |
| `verb-cycler/static` | No flavour cycling; only activity, pins and past tense change the text | ide, terminal, enterprise apps |

### Pro (17)
| Name | One-line idea |
|---|---|
| `verb-cycler/scramble` | New verb resolves out of random letters, left to right |
| `verb-cycler/scramble-braille` | Same, with braille noise; reads as "machine" in terminals |
| `verb-cycler/scramble-kana` | Katakana noise; Japanese locale default for scramble |
| `verb-cycler/decrypt-hex` | Monospace hex noise for the `coder` pack |
| `verb-cycler/split-flap` | Departure-board flaps per character |
| `verb-cycler/odometer` | Each character rolls vertically like a counter wheel |
| `verb-cycler/flip-3d` | The word flips on its X axis |
| `verb-cycler/morph` | Shared letters glide into place, others fade (Thinking → Thought keeps "Th") |
| `verb-cycler/typewriter-replace` | Backspaces the old verb and types the new one with a caret |
| `verb-cycler/kinetic-push` | New word pushes the old one sideways on a soft spring |
| `verb-cycler/glitch-rgb` | One brief channel-split glitch per swap |
| `verb-cycler/drum` | Words sit on a rotating drum |
| `verb-cycler/stack-peek` | The next verb is faintly visible below, then slides up |
| `verb-cycler/ticker-tape` | Verbs scroll as a slow marquee for wide status bars |
| `verb-cycler/letter-shuffle` | Letters fly from the old word into the new word |
| `verb-cycler/wave-in` | New glyphs rise in a short wave |
| `verb-cycler/cascade-drop` | Old glyphs fall away, new ones drop in |

## 7. Edge cases
- **RTL.** Vertical transitions unchanged; horizontal ones (`kinetic-push`, `ticker-tape`, scramble
  resolve order) follow reading direction. **Arabic and other joining scripts:** per-glyph transitions
  (scramble, split-flap, odometer, letter-shuffle, wave-in, cascade-drop) would break joining forms, so
  they fall back to `blur-swap` (proposed rule: per-glyph transitions require a non-joining script).
- **CJK.** Scramble uses a script-matched noise set (kana for ja; for zh/ko fall back to `fade`, since
  random Hanzi/Hangul reads as real words — proposed). Dwell weighting ×1.6 per character.
- **Very long verbs / targets.** Pack schema caps verbs at 24 chars; targets middle-truncate; with
  `ReserveLongest` the slot is the longest pack verb, and pinned text longer than the slot truncates
  rather than growing (tooltip has it all).
- **Screen width.** Under 320 px or in 16–20 px rows: `terse` pack, `fade` only, no targets.
- **0 s / very fast turns.** A turn under 1 s shows only the activity verb and no past-tense line.
- **Tool storms** (dozens of 50 ms tools): debounce means none are shown; after 3 s of continuous
  short tools the line pins "Running tools" (count shown when known) — proposed.
- **Very slow turns.** Dwell grows; after 15 s of stall the cycler freezes (status-text decides text).
- **Header and tool at once.** Tool pin wins; the header returns when the tool ends (if still current).
- **Pack swap mid-run.** Takes effect at the next swap; never interrupts a pin.
- **Font fallback.** Noise glyphs not present in the UI font fall back to `fade` (no tofu boxes).
- **Copy.** Copying the line yields the current settled text, never noise characters.

## 8. Accessibility
- Accessible name = the **incoming** verb from the first frame of a swap; noise glyphs, peeks, flaps
  and dots are `aria-hidden`.
- **Flavour verbs are never announced.** Only activity changes (pinned tool verbs lasting ≥ 2 s,
  "Waiting for you", completion, interruption, failure) reach the live region, at most one polite
  message per 10 s (family throttle, status-text.md §11). Waiting and failure are assertive.
- **Reduced motion:** 80 ms crossfades, dwell × 2, no marquee, no glitch.
- **Flashing:** `glitch-rgb` runs at most once per swap; swaps are ≥ 2.8 s apart, well under 3/s.
- **Contrast:** every frame of a transition keeps ≥ 4.5:1 for the *settled* glyphs; in-flight glyphs
  may dip during ≤ 200 ms fades (they are not the readable state). `stack-peek` text is decorative.
- Users can turn flavour cycling off (`cycle = false`), which apps should expose as "Show playful
  status words".

## 9. Props sketch
```rust
pub struct VerbCycler {
    pub drive: CyclerDrive,              // FromState(&AgentState) | Manual(CyclerInput)
    pub pack: VerbPackId,                // "neutral"
    pub pack_override: Option<VerbPackOverride>,
    pub variant: CyclerVariant,          // Fade | Slide | BlurSwap | Ellipsis | Static | Pro(ProCycler)
    pub cycle: bool,                     // flavour cycling on/off
    pub order: Option<Order>,            // overrides the pack: Shuffle | Sequential
    pub timing: CyclerTiming,            // dwell_min 2800 ms, dwell_per_grapheme 70 ms,
                                         // pin_show 300 ms, pin_min 900 ms, header_min 1500 ms
    pub width: WidthPolicy,              // ReserveLongest in hosts with neighbours
    pub show_target: bool,
    pub motion: MotionLevel,
}

pub enum CyclerInput {
    Activity(ActivityKind),
    Pin { verb: VerbPair, target: Option<SharedString> },   // from ToolRunning { tool, verb }
    Header(SharedString),                                    // model-written summary header
    Unpin,
    Pause,                                                   // AwaitingApproval
    Freeze,                                                  // Stalled(hard)
    Complete { elapsed: Duration, kind: Completion },        // Done → past tense
    Interrupt { after: Duration },
    Fail { kind: ErrorKind },
}
// Mapping: Thinking{verb} → Activity(Think) then flavour; ToolRunning{tool, verb} → Pin;
// Streaming → Activity(Write) pinned; AwaitingApproval → Pause; Stalled → Freeze (hard level);
// Done{elapsed} → Complete; Cancelled → Interrupt; Error{kind} → Fail.
```

## 10. Related components and references
Related: [status-text.md](status-text.md) (packs, tense rules, activity table) ·
[text-shimmer.md](text-shimmer.md) (band yields to swaps) · [elapsed-timer.md](elapsed-timer.md)
(duration in the settled line) · [composer-status-strip.md](composer-status-strip.md) · tool rows and
plan steps (family F) · loaders (family B, `semantic-verb` pairing).

| Ref | What we take | Licence note |
|---|---|---|
| Claude Code spinner verbs (note 08) | idea of flavour verbs + past-tense completion; user-overridable lists | Proprietary — reference only; **no words copied** (CI deny-list) |
| Codex CLI (note 08) | model-written header replaces the fixed verb | Apache-2.0 per note 08 (MIT in licensing.md) — portable |
| OpenCode (note 08) | scramble wordmark settling after 6–13 ticks; AnimatedNumber roll + mask | MIT — portable with attribution |
| Cline (note 08) | 500 ms grace before "Thinking…" | Apache-2.0 — portable |
| Zed LoadingLabel (note 08) | typed-in label then dot cycle | mixed — study only |
| Antigravity changelog (note 08) | count of tools instead of flickering names | product docs — idea only |
| Cursor (note 08) | "Taking longer than expected" escalation | closed — idea only |
| Pitus [R021] | Thinking → Thought for Ns roll | Study only |
| Arc UI (note 12) | past-tense crossfade with measured slot | Paid/unclear — study only |
| affectUI Thinking Loader (note 11) | phase words that lock to a check | Paid — study only |
| Motion Primitives, Magic UI, React Bits (note 02) | Text Scramble / Morph / Roll, Word Rotate, Split-Flap Text | licences not recorded in notes — study only |
| Fancy Components [R138] | Scramble In, Text Rotate | MIT — portable with attribution |
| pi-spinner (note 02) | verb packs as a product concept | licence not recorded — study only |
