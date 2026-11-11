**Status:** draft v0.1 (2026-11-11).

# Spec: text reveal and carets (`text-reveal`, `caret`)

Part of family C ([status-text.md](status-text.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **`text-reveal` 5 free + 13 Pro** and
**`caret` 3 free + 5 Pro** = **26 variants**.

## 1. Purpose
While the agent answers, tokens arrive in bursts: nothing for 400 ms, then 300 characters at once,
then a trickle. The user is *reading as it arrives*. Raw appends feel jerky; heavy animation lags
behind the model and makes a fast model look slow; half-written markdown flashes raw `**` and
`[link](`; and auto-scroll fights the user who scrolled up to re-read. The apps and kits we studied
each solve one piece (notes 02, 08): Codex commits whole lines with a two-gear catch-up and a table
hold-back; MonoCode repairs unterminated markdown per block (Streamdown + `remend`); Claude desktop
fades graphemes in from a 6 px blur (reverse-engineered, note 08); Streamdown animates per-word spans
for 150 ms and removes them when done; LibreChat staggers 25 ms and uses `Intl.Segmenter` for CJK;
a trailing fade keeps the last 3–5 words lighter [R019]; a word blur-in uses 8 px → 0, y +4 → 0,
~60 ms stagger [R051]. Smoothers (smooth-stream-text, use-smooth-text) prove that **pacing and visual
effect must be two separate layers** or they fight (note 02).

Oxyn's `text-reveal` is therefore two layers:
1. **Pacer** — decides *how many* units become visible each frame (never more than ~600 ms behind
   the stream).
2. **Effect** — decides *how* a newly visible unit appears (fade, blur, typewriter…), within a fixed
   budget of in-flight animations.
Plus a **markdown-aware live tail**, a **caret**, and **scroll anchoring**.

## 2. State machine
```
               first token                     buffer ≥ 600 graphemes or oldest ≥ 600 ms
  Waiting ───────────────▶ Revealing(Smooth) ─────────────────────────────────────────▶ Revealing(CatchUp)
  (caret blinks at          │   ▲   │  ▲                ◀── buffer ≤ 60 and age ≤ 100 ms, held 250 ms ──┘
   insertion point)         │   │   │  └── token arrives
                            │   │   ▼ no token for 500 ms
                            │   │  Idle (caret starts blinking) ── no token ≥ stall_soft ──▶ Stalled (caret dims)
                            │   │                                                         │ token → Revealing
                            │   └── tool call / approval ends ◀── Paused (block boundary; caret moves to next block)
                            │
                            │ stream ends ──▶ Draining (flush ≤ 300 ms) ──▶ Settled (spans unwrapped, caret fades)
                            │ Stop ─────────▶ Interrupted (received text shown at once, no animation)
                            │ Error ────────▶ Failed (partial kept; error row appended)
                            │ live tail off-screen ──▶ Offscreen (instant reveal, no effect cost) ──▶ back when visible
  Reduced motion: Pacer unchanged (line / sentence chunks); effect = 80 ms fade per chunk; caret static.
```

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Waiting` | empty message shell + caret at insertion point (or the status line's "Thinking"; host picks one) | Stop (send control) | — | Esc stops |
| `Revealing(Smooth)` | text appears at stream pace with the variant's effect; caret solid | Stop | select / copy settled text | Esc stops |
| `Revealing(CatchUp)` | whole blocks appear with a 120 ms block fade; no per-token effects | Stop | — | Esc |
| `Idle` | caret blinks (500 / 500 ms), stops blinking after 5 s (stays solid) | Stop | — | Esc |
| `Stalled` | caret dims (non-text contrast ≥ 3:1); status line handles words | Stop / Retry (status strip) | — | Esc |
| `Paused` | block ends cleanly; tool row or approval card inserts below; caret hidden | host's | — | host's |
| `Draining` | remaining buffer flushes, accelerating | — | — | — |
| `Settled` | plain text, caret gone, animation spans removed | message actions (copy, retry…) | — | message actions |
| `Interrupted` | everything received, "Interrupted" marker row (status strip owns it) | retry | continue | Enter |
| `Failed` | partial text + error row "Stopped · model error" | retry | — | Enter |
| `Offscreen` | (not visible) text revealed instantly; "↓ New output" pill visible | jump to latest | — | End |

## 3. Pacer (proposed numbers; idea from Codex's two-gear policy, Apache-2.0/MIT, note 08)
| Parameter | Value | Notes |
|---|---|---|
| Unit | grapheme (typewriter, glyph effects) · word (fade-token, word-blur) · line (line-commit) | CJK / Thai: word → grapheme |
| Target lag `L` | 250 ms | how far behind the stream we aim to be |
| Smooth rate | `max(40 graphemes/s, buffer / L)` | never crawls; matches the stream on average |
| Enter CatchUp | buffer ≥ 600 graphemes **or** oldest unrevealed ≥ 600 ms | — |
| CatchUp behaviour | reveal whole blocks per frame; block fade 120 ms; no per-unit effects | effect budget protected |
| Exit CatchUp | buffer ≤ 60 **and** age ≤ 100 ms, held 250 ms; no re-entry for 250 ms unless buffer ≥ 4000 or age ≥ 1500 ms | hysteresis avoids gear flapping |
| Max lag guarantee | 600 ms | the pacer never hides received text longer |
| Draining (stream ended) | flush rest within ≤ 300 ms, linear acceleration | end feels prompt |
| Burst at start (first chunk ≥ 400 graphemes) | reveal first 400 at once with a 160 ms block fade, pace the rest | cached or instant answers |
| `Pacing::Raw` | show as received (effects still apply per arrival) | for apps with their own smoother |
| In-flight effect budget | ≤ 64 animated units; older units are committed static | bounds per-frame cost |

## 4. Effects (per newly visible unit)
All effects modulate opacity, blur, colour and small transforms on glyphs **already laid out at their
final positions** — so revealing never reflows text. The only exceptions are `typewriter` and
`line-commit`, which append text and let the line grow (append-only, so earlier lines never move).

| Effect | Per-unit animation | Duration | Easing | Stagger | Frame rate |
|---|---|---|---|---|---|
| `fade-token` | opacity 0 → 1 | 160 ms (`dur.reveal.token`) | `ease.out` | 25 ms per word, cap 300 ms | 60 Hz |
| `word-blur` | blur 6 → 0 px, y +3 → 0, opacity 0 → 1 | 180 ms | `ease.out` | 25 ms, cap 300 ms | 60 Hz |
| `typewriter` | grapheme appears whole (no fade) | — | — | pace = pacer | 60 Hz |
| `line-commit` | full line appears; 80 ms fade | 80 ms | linear | — | 30 Hz |
| `instant` | none (pacing only) | — | — | — | on change |
| `trailing-fade` | last 4 words held at the contrast-floored dim level, step to full as each new word lands [R019] | 200 ms per step | `ease.out` | — | 60 Hz |
| `gradient-edge` | the last 8 graphemes carry a fg → base gradient that travels with the edge | continuous | — | — | 60 Hz |
| `drop-in` | y −6 → 0, opacity | 200 ms | `spring.snap` | 20 ms | 60 Hz |
| `pull-up` | y +8 → 0 clipped at the line box | 220 ms | `ease.out` | 20 ms | 60 Hz |
| `slide-in` | x −6 → 0 (mirrors in RTL), opacity | 180 ms | `ease.out` | 20 ms | 60 Hz |
| `gradual-spacing` | per-glyph x offsets spread ±0.1 em converge to 0 (layout unchanged) | 260 ms | `ease.out` | — | 60 Hz |
| `highlight-trail` | new text carries an accent background tint that fades | 1200 ms | `ease.out` | — | 30 Hz |
| `ink-settle` | blur 3 → 0 and colour from 70 % → 100 % fg | 240 ms | `ease.out` | 15 ms | 60 Hz |
| `cross-blur` | each new word crossfades from a blurred ghost of itself | 220 ms | `ease.in-out` | 25 ms | 60 Hz |
| `scramble-in` | last 3 graphemes show noise for 2 ticks (40 ms) before settling; prose only | 80 ms | linear | — | 30 Hz |
| `dither-in` | ordered-dither alpha mask 0 → 100 % | 200 ms | stepped (4 steps) | — | 30 Hz |
| `glyph-rise` | each grapheme rises 2 px | 120 ms | `ease.out` | 8 ms | 60 Hz |
| `focus-pull` | when a sentence completes, it sharpens from blur 2 px → 0 | 200 ms | `ease.out` | — | 60 Hz |
| CatchUp block fade (all) | whole block opacity 0 → 1 | 120 ms | `ease.out` | — | 60 Hz |
| **Reduced motion** | pacer releases whole lines (prose) or sentences; each chunk fades in 80 ms; no blur, no transforms, no stagger; caret static, no blink | 80 ms | linear | none | on change |

## 5. Markdown and code while streaming
Rule: **stable blocks + one live tail.** Closed blocks are parsed once and frozen; only the last
(open) block is re-parsed and re-rendered each frame (Codex's mutable tail, MonoCode's per-block repair
— both portable, note 08). Parsing uses an incremental CommonMark/GFM pass on the tail only.

| Construct | While open | When closed |
|---|---|---|
| Paragraph text | revealed with the effect | spans unwrapped |
| `**bold**`, `_em_`, `` `code` `` | **repaired**: rendered as if closed (auto-close for display only) | real parse replaces repair; no visible change if it matched |
| Lone `*` / `-` / `1.` at line start | held one grapheme until the next char decides list vs emphasis | — |
| Heading `#` | held until a space follows, then shown as a heading | — |
| Link `[text](url` | link text shown as plain text; **URL never shown** while open | upgrades to a link (colour 120 ms crossfade) |
| Image `![alt](…` | placeholder box with alt text, fixed 16:9 at 40 % column width (proposed) | image loads into the box |
| Raw HTML | withheld until the tag closes, then sanitised | — |
| Code fence ```` ```lang ```` | block chrome (language label, disabled copy) appears at once; lines reveal by **line-commit** (80 ms fade) — never per-token effects in code | copy enabled; final highlight pass |
| Syntax highlight | settled lines highlighted; the tail line re-highlighted at most every 100 ms | full pass once on close |
| Table | header row held until its delimiter row arrives; rows appear whole; column widths may grow (160 ms `spring.snap`) until 20 rows, then freeze and wrap | final widths |
| Math `$…$` / `$$…$$` | raw source in mono, dim (contrast-floored) | rendered |
| Citation `[1]` | chip appears when `[`, digits and `]` are complete | hover card ready |
| Blockquote, rule | immediate | — |

Interleaved tool calls: when a tool or approval interrupts, the open block is closed **visually**
(repair applied), the caret leaves, the tool row inserts below; text resumes in a new block.

## 6. Carets
| Caret | Shape | Default colour | Notes |
|---|---|---|---|
| `caret/bar` (free default) | 2 px × 1.1 em, rounded 1 px | `text.fg` | chat, desktop |
| `caret/block` | 0.55 em × 1.1 em at 70 % opacity (terminal: full cell `█`) | `text.fg` | terminal, IDE |
| `caret/underscore` | 0.55 em × 2 px on the baseline | `text.fg` | mono / retro |
| `caret/glow` (Pro) | bar + 6 px soft glow in accent | accent | |
| `caret/pulse-dot` (Pro) | 6 px dot breathing 0.6 ↔ 1 on a 1.2 s sine | accent | voice, mobile |
| `caret/comet-trail` (Pro) | bar leaving a 3-grapheme highlight trail that fades in 300 ms | accent | |
| `caret/ghost-trail` (Pro) | the last 4 caret positions fade out over 400 ms | fg 30 % | |
| `caret/gradient` (Pro) | bar with a vertical accent → fg gradient | accent | |

Behaviour (all carets):
- **Position:** after the last *revealed* grapheme (not the last received). In code, at the end of the
  current code line in a mono cell; **hidden inside tables** and math; jumps to a new block without
  animation (Pro `comet-trail` glides 80 ms).
- **Solid while text is arriving.** After 500 ms without a reveal it blinks 500 ms on / 500 ms off
  (step); **after 5 s of idle it stops blinking and stays solid** (WCAG 2.2.2 — no blinking over 5 s).
- **Stalled:** dims to the lowest level that keeps 3:1 non-text contrast.
- **Done:** fades out over 200 ms (`ease.out`). **Stop / error:** removed instantly.
- Cline swaps the caret for a shimmer when typing completes (note 08); we offer that as host
  composition (`text-shimmer` on the last line), not caret behaviour.

## 7. Scroll anchoring
| Rule | Value (proposed) |
|---|---|
| Follow mode on | when the user is within 48 px of the bottom at stream start, or returns there |
| Follow motion | keep the caret line ≥ 24 px above the viewport bottom; scroll each frame with `spring.snap`, never ahead of content growth (Cursor "glides while following a stream", note 08) |
| Detach | user scrolls up > 48 px, uses PageUp/Home, or drags the scrollbar |
| While detached | content keeps arriving; a "↓ New output" pill (bottom centre) shows a rolling line count; tail reveals instantly (`Offscreen`) |
| Re-attach | click the pill (smooth scroll `dur.travel` 260 ms `ease.in-out`; instant if > 3 viewports away) or reach the bottom by hand |
| Selection | an active text selection in the streaming message pauses follow mode until cleared |
| Above-viewport stability | settled blocks never change height; when a table above the fold re-widens, the scroller anchors to the first visible block so the view does not jump |
| Resize mid-stream | stable blocks re-wrap without animation; anchoring keeps the first visible line in place |
| Reduced motion | follow scroll is instant per chunk; pill appears without slide |

## 8. Micro-interactions
- **First token.** Caret (or the host's "Thinking" line) is replaced by text with the effect; the
  message container height grows downward only, with the composer never moving (composer.md rule).
- **Burst.** CatchUp blocks fade in as wholes — the user sees paragraphs land, not a sprint of words.
- **End.** Draining flush ≤ 300 ms → caret fade 200 ms → animation spans unwrapped (one relayout,
  invisible because positions are identical) → message actions fade in 120 ms.
- **Stop.** Everything received appears instantly; no further effects; the status strip drops
  "Interrupted" into the transcript.
- **New output pill.** Slides up 8 px + fade (160 ms `ease.out`); count rolls with `dur.digit.roll`.
- **Retry / regenerate.** Old text crossfades to 40 % and collapses when the new stream's first block
  lands (200 ms), so the user does not read stale text as new.

## 9. Variants

### Free — `text-reveal` (5)
| Name | Idea | Surfaces |
|---|---|---|
| `text-reveal/fade-token` | Default: words fade in 160 ms with a 25 ms stagger | chat, desktop, mobile |
| `text-reveal/word-blur` | Words sharpen from a 6 px blur with a 3 px lift | chat, desktop |
| `text-reveal/typewriter` | Graphemes appear at stream pace, no fade; pairs with `caret/block` | terminal-look chat, voice captions |
| `text-reveal/line-commit` | Whole lines land; best for terminals and code (Codex approach) | terminal, ide |
| `text-reveal/instant` | Pacing only, no effect — for performance-critical or accessibility-first apps | all |

### Pro — `text-reveal` (13)
| Name | One-line idea |
|---|---|
| `text-reveal/trailing-fade` | The last four words stay lighter and darken as new words arrive |
| `text-reveal/gradient-edge` | A soft gradient rides the leading edge of the text |
| `text-reveal/drop-in` | Words drop 6 px into place on a crisp spring |
| `text-reveal/pull-up` | Words rise from below the line box, clipped |
| `text-reveal/slide-in` | Words slide 6 px in reading direction |
| `text-reveal/gradual-spacing` | Letters converge from a slight spread (visual offsets only, no reflow) |
| `text-reveal/highlight-trail` | Fresh text carries an accent tint that fades over 1.2 s — easy to find "where it was" |
| `text-reveal/ink-settle` | Text settles like ink: slight blur and lighter colour resolving |
| `text-reveal/cross-blur` | Each word crossfades from a blurred ghost of itself |
| `text-reveal/scramble-in` | The newest graphemes flicker through noise for two ticks (prose only) |
| `text-reveal/dither-in` | Words resolve through a 4-step dither mask (pixel aesthetic) |
| `text-reveal/glyph-rise` | Each grapheme rises 2 px with a tight 8 ms stagger |
| `text-reveal/focus-pull` | A sentence snaps into focus when its full stop arrives |

### Free — `caret` (3): `caret/bar`, `caret/block`, `caret/underscore`.
### Pro — `caret` (5): `caret/glow`, `caret/pulse-dot`, `caret/comet-trail`, `caret/ghost-trail`, `caret/gradient` (ideas in §6).

## 10. Edge cases
- **RTL.** Units and stagger follow logical order, so the reveal visually moves right → left; `slide-in`
  mirrors; the caret sits at the logical end (visual left). Mixed-direction lines are shaped once per
  frame on the tail only.
- **CJK / Thai.** No word spaces: word effects use per-grapheme units with the same caps; line-breaking
  follows the platform's rules; the pacer's `40 graphemes/s` minimum becomes 20 (denser script —
  proposed).
- **Emoji / ZWJ sequences.** Revealed only when the full cluster has arrived (a half sequence is never
  shown as two emoji).
- **Very long words / URLs.** Break anywhere after 40 characters in prose; never inside inline code.
- **Screen width.** Narrow (< 360 px) disables blur effects (cost and legibility) → `fade-token`.
- **0 s / whole answer in one chunk.** Burst rule: one 160 ms block fade, no per-word animation.
- **Very fast streams** (> 2000 graphemes/s): CatchUp nearly always; effects effectively off; that is
  intended. **Very slow streams** (< 2 tokens/s): each word animates; caret blinks between; stall at
  `stall_soft` hands wording to the status line.
- **Huge outputs (> 100 KB).** Transcript virtualises settled blocks; reveal cost bounded by the
  64-unit budget.
- **Tool interleaving.** Reveal pauses at the block boundary; the pacer's buffer is flushed before the
  tool row appears so text never trails behind a later tool row.
- **Background tab.** Reveal does nothing while hidden; on focus the received text appears with one
  block fade (no replay of the "typing").
- **Terminal surface.** Only `line-commit`, `typewriter`, `instant`; carets `block` / `underscore`.

## 11. Accessibility
- Streaming message: real text nodes; container `aria-busy="true"` while streaming; **`aria-live`
  off on the message itself** — screen readers are **never** fed tokens.
- Announcements (polite, via the family live region): "Response started" (optional), **"Response
  complete"**, "Interrupted", "Failed". Opt-in `announce = Paragraphs`: each *completed* paragraph is
  announced, throttled to one per 5 s and never mid-sentence (proposed).
- Users can read partial content at any time; the caret and effect spans are `aria-hidden` and do not
  split words for assistive tech (span unwrapping on settle; during streaming the accessible text is
  the concatenated text).
- **Contrast:** settled text is full fg; transient dim levels (trailing-fade, ink-settle, math raw)
  respect the contrast floor (≥ 4.5:1 for anything held longer than 300 ms; ≥ 3:1 for carets).
- **Reduced motion:** §4 last row; caret static; follow scroll instant.
- **Blinking:** caret blink stops after 5 s idle (WCAG 2.2.2).
- **Focus:** streaming never steals focus from the composer; "↓ New output" pill is reachable after
  the message in tab order and via End.

## 12. Props sketch
```rust
pub struct TextReveal {
    pub source: StreamHandle,           // incremental text + block events from the adapter
    pub drive: RevealDrive,             // FromState(&AgentState): Streaming → Revealing,
                                        // AwaitingApproval / ToolRunning → Paused, Done → Draining,
                                        // Cancelled → Interrupted, Error → Failed, Stalled → Stalled
    pub pacing: Pacing,                 // Smooth { target_lag: 250 ms, min_rate: 40, catchup: CatchUpCfg } | Raw
    pub effect: RevealEffect,           // FadeToken | WordBlur | Typewriter | LineCommit | Instant | Pro(..)
    pub unit: Option<RevealUnit>,       // override: Grapheme | Word | Line
    pub budget: usize,                  // 64 in-flight animated units
    pub caret: Option<CaretVariant>,    // Bar | Block | Underscore | Pro(..)
    pub markdown: MarkdownMode,         // Gfm { repair_tail: true, table_holdback: true } | Plain
    pub code_reveal: CodeReveal,        // LineCommit (default) | Instant
    pub anchor: ScrollAnchor,           // Follow { threshold: 48 px, margin: 24 px } | None
    pub announce: Announce,             // CompletionOnly | Paragraphs { min_gap: 5 s } | Off
    pub motion: MotionLevel,
}

pub struct CatchUpCfg {
    pub enter_buffer: usize,            // 600 graphemes
    pub enter_age: Duration,            // 600 ms
    pub exit_buffer: usize,             // 60
    pub exit_age: Duration,             // 100 ms
    pub hold: Duration,                 // 250 ms
    pub severe_buffer: usize,           // 4000
    pub severe_age: Duration,           // 1500 ms
    pub max_lag: Duration,              // 600 ms
    pub drain: Duration,                // 300 ms
}
```

## 13. Related components and references
Related: [status-text.md](status-text.md) · [text-shimmer.md](text-shimmer.md) (optional shimmer on
the last line after typing) · streaming markdown, message bubble, citations (family E) · tool rows
(family F) · voice captions (family K) · [composer-send-control.md](composer-send-control.md) (Stop).

| Ref | What we take | Licence note |
|---|---|---|
| Codex CLI streaming (note 08) | line commit, two-gear catch-up with hysteresis, table hold-back, link/math withholding | Apache-2.0 per note 08 (MIT in licensing.md) — portable with attribution |
| MonoCode (note 08) | Streamdown + `remend` per-block repair of the live tail | MIT — portable with attribution (Streamdown's own licence not recorded — check) |
| Cline TypewriterText (note 08) | 30 ms/char labels, then shimmer instead of caret | Apache-2.0 — portable |
| OpenCode Typewriter (note 08) | humanised per-char delays (labels only) | MIT — portable |
| Claude desktop grapheme blur-in (note 08, reverse-engineered) | opacity + 6 px blur reveal | proprietary — idea only |
| Streamdown, FlowToken, Vivid Layer, smooth-stream-text, use-smooth-text, LibreChat (note 02) | per-word spans removed after streaming; effect menu; pacing as its own layer; 25 ms stagger and `Intl.Segmenter` | licences not recorded in our notes — study only until checked |
| @david_visuals_ [R019] | trailing-fade streaming | Study only |
| @AdiKodez [R051] | word blur-in numbers | Study only |
| transitions.dev [R132] | "streaming text resolves through a soft cross-blur" (note 17) | custom licence forbids competing kits — study only, no code |
| Arc UI (note 12) | recommends text-stream for answers, shimmer for status | Paid/unclear — study only |
