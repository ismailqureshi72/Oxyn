**Status:** draft v0.1 (2026-11-11).

# Spec: reasoning block (`reasoning-block`) — collapsible thinking

Part of family E ([messages.md](messages.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **5 free + 15 Pro = 20 variants**.
The header line reuses family C: verb + timer + shimmer from [status-text.md](status-text.md)
(`status-text/thought-header` is the stand-alone header line; this spec owns the container, body,
display levels and collapse policy). Body text reveal follows [text-reveal.md](text-reveal.md).

## 1. Purpose
Reasoning models think before they answer — seconds to minutes. Users want two opposite things from
that time: **proof it is working and roughly on what** ("Thinking 8s · Checking the auth flow"), and
**not to read it** once the answer arrives. The converged pattern (note 04 §2c: ChatGPT, Claude,
Gemini, Grok, Raycast, AI Elements Reasoning) is a block that **auto-opens while streaming and
collapses to "Thought for Ns" when the answer starts**; Warp streams a "Thinking…" header that
collapses to "Thought for…"; Roo ticks a live seconds counter on its reasoning block; Cline expands its
body into a 150 px scrolling area; Claude desktop offers a three-level transcript density switch
**Normal / Thinking / Verbose** (all note 03 / 08). Agent runs add complications chat kits skip:
reasoning **resumes between tool calls**, providers may send **summaries instead of raw thought**, or
**redacted / encrypted** reasoning that must never be shown, and auto-collapse must not yank text out
from under someone reading it. This spec defines all of it.

## 2. State machine
```
                 Thinking (AgentState)                show_delay 300 ms passes
  Hidden ──────────────────────────────▶ Pending ─────────────────────────────▶ Live{expanded | collapsed}
    ▲          (< 300 ms of thinking never shows)       │                          │   ▲  user toggles → Pinned flag set
    │                                                   │ Thinking ends < 300 ms   │   │
    │◀──────────────────────────────────────────────────┘                          │   └── Stalled (header tone ramps; status-text)
    │                                                                              │
    │            first answer token / tool start (segment ends)                    ▼
    │                                       ┌─────────────────────────── SegmentEnd ───────────────────────────┐
    │                                       ▼ (display level Thinking, not Pinned, reader not inside)          │
    │                              AutoCollapsing (wait collapse_delay 400 ms, then 200 ms) ──▶ Done{collapsed}│
    │                              (reader inside: defer until pointer leaves + 1500 ms)                       │
    │                                                                                                          ▼
    │                                                                          Done{expanded} (Verbose, or Pinned open)
    │   Done{collapsed} ◀── toggle ──▶ Done{expanded}            ("Thought for 12s ›")
    │
    │   Variants of Done by content:  Summary (chip "Summary") · Raw · Redacted (🔒 no body) · Encrypted (🔒 no body)
    │   Terminal outcomes:  Stop ──▶ Interrupted("Stopped thinking after 8s")   Error ──▶ Failed("Thinking failed")
  Next segment (after a tool row): a new block starts at Pending (or joins the timeline in `reasoning-block/timeline`)
```

### 2.1 States table
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing (status row covers the first 300 ms) | — | — | — |
| `Pending` | nothing yet; the transcript-tail loader / status row runs ([loaders.md](loaders.md) §3.3) | — | — | — |
| `Live{expanded}` | header "Thinking 8s" (shimmer, ticking timer, ▾) + body window (max 160 px, follows tail, 24 px top fade) | collapse | Stop (composer) | Enter/Space on header toggles; Esc stops run (composer) |
| `Live{collapsed}` | header "Thinking 8s" + one-line live subtitle "· Checking the auth flow" (latest summary heading or first sentence ≤ 60 chars) | expand | — | Enter/Space |
| `Stalled` (overlay) | header verb freezes, tone ramps to warning after `stall_soft`; text "Still thinking…" after 15 s (status-text rules) | — | — | — |
| `AutoCollapsing` | body folds; header rolls "Thinking 12s" → "Thought for 12s" (240 ms, status-text §5.4) | — | — | — |
| `Done{collapsed}` | "Thought for 12s ›" (+ "Summary" chip if summarised; 🔒 if redacted) | expand | copy reasoning (`⋯`) | Enter/Space |
| `Done{expanded}` | header + full body (no window cap; long bodies > 40 lines get "Show all" like code, streaming-markdown §3.4) | collapse | copy reasoning | Enter/Space; Esc collapses when focus is inside |
| `Redacted` / `Encrypted` | "Thought for 12s 🔒" ; expanded body: "The model's reasoning isn't available for this response." (no blob, no length hint beyond duration) | — (expand shows the notice) | learn more (host) | Enter/Space |
| `Interrupted` | "Stopped thinking after 8s" + partial body (collapsed by default) | expand | — | Enter/Space |
| `Failed` | "Thinking failed" in danger tone; message-level Failed row owns Retry | expand (partial, if any) | — | Enter/Space |
| `Briefly` (thinking 300 ms – 1 s) | "Thought briefly ›" (no seconds; status-text "no 0 s lines" rule) | expand | — | Enter/Space |

`Pinned` is a flag, not a state: once the user toggles a block during this turn, auto-collapse and
display-level changes no longer move **that** block (proposed; the user's hand beats the policy).

## 3. Behaviour rules

### 3.1 Display levels (per thread, persisted per app; default `Thinking`)
| Level | Live | After the answer starts | Tool rows (family F) | Copy message includes reasoning |
|---|---|---|---|---|
| `Normal` | `Live{collapsed}` — header + live subtitle only | `Done{collapsed}` | rolled up ("Explored 6 files") | no |
| `Thinking` (default) | `Live{expanded}` with the 160 px window | auto-collapse → `Done{collapsed}` | rolled up | no |
| `Verbose` | `Live{expanded}`, no window cap | stays `Done{expanded}`; raw shown if available, plus per-segment duration and reasoning token count when the adapter provides it | expanded with inputs / outputs | yes (as a quoted "Reasoning" section) |
- Switching level re-applies to all **unpinned** blocks in the thread with one `dur.collapse` animation,
  anchored to the first visible block ([messages.md](messages.md) §4.2).
- Shortcut: Ctrl+O cycles Normal → Thinking → Verbose (proposed; Claude desktop uses Ctrl+O for its
  density switch, note 03 — remappable, listed in the ? help). A transient toast names the new level
  ("Showing: Verbose") for 1.5 s.

### 3.2 Summaries vs raw vs redacted
| Content kind (from the adapter) | Header | Body | Copy |
|---|---|---|---|
| `Raw` | "Thought for 12s" | full text, markdown-lite (bold, lists, inline code; no tables / images) | allowed |
| `Summary` | "Thought for 12s" + "Summary" chip (tooltip "A summary of the model's reasoning") | summary text; bold-led lines become step titles and feed the live subtitle | allowed |
| `Redacted` (provider withheld some or all) | "Thought for 12s 🔒" | notice; if partly redacted, visible parts + a "[part of the reasoning is hidden]" row at each gap | visible parts only |
| `Encrypted` (opaque token for round-tripping) | "Thought for 12s 🔒" | notice only; the opaque payload is **never rendered, logged to the UI or copied** | disabled |
| `None` (model doesn't expose reasoning, but we know it thought) | "Thought for 12s" (no chevron) | — | — |
The adapter decides the kind; Oxyn never guesses it from text. Duration comes from event timestamps
(first reasoning event → first non-reasoning event), frozen at the event, not at the end of the roll
(status-text §5.4).

### 3.3 Auto-collapse (proposed)
1. Trigger: the segment ends — first answer token, or a tool call starts (the latter collapses only in
   `Normal`/`Thinking` levels; in `Verbose` nothing collapses).
2. Wait `collapse_delay` **400 ms** so the eye registers the switch (and catches a quick tool→thinking
   bounce: if thinking resumes within 400 ms, cancel and keep the block live — `handoff_grace` from
   loaders.md is 250 ms; we use 400 ms here because collapsing is costlier than a tempo change).
3. **Reader guard:** if the pointer is over the block, the body has a text selection, focus is inside, or
   the user scrolled the body window in the last 2 s, defer until those clear + **1500 ms**.
4. Collapse with `dur.collapse` (200 ms). If the block is above the viewport's first visible line,
   the height loss is compensated (no visible jump); if following, the view keeps following.
5. Never auto-collapse a `Pinned` block, a block in `Verbose`, or a block whose segment ended in error.
6. Never auto-**expand** a done block (e.g., on level change only unpinned blocks move, as one batch).

### 3.4 Live subtitle (collapsed live view)
Source priority: (1) the adapter's explicit step title if any; (2) the last bold-led line of a summary;
(3) the first sentence of the latest raw paragraph, truncated at 60 chars with an ellipsis; updated at
most once per **2 s** with a `verb-cycler/fade` swap (180 ms) so it reads, not flickers (proposed;
Codex pins a model-written header over generic verbs, note 08).

### 3.5 Multiple segments
Reasoning that resumes after a tool row starts a **new block** (each with its own duration) placed in
arrival order inside the agent message. When a turn has ≥ 3 segments, the footer summary adds total
thinking time: "Worked for 1m 12s · thought for 38s" (proposed). `reasoning-block/timeline` (Pro)
merges segments and tool steps into one timeline instead.

## 4. Micro-interactions
- **Appear (Pending → Live):** header fades + rises 4 px (180 ms, `dur.row.enter`); in `Thinking`
  level the body window opens 0 → first line height, then grows with content up to 160 px.
- **Header while live:** verb shimmer (`text-shimmer/calm`, status-text family timings), timer ticks at
  1 Hz with digit roll; chevron ▾ static (no idle motion).
- **Body window:** text reveals with the message's reveal effect (word-level `fade-token` by default;
  terminal `line-commit`); auto-follows the tail inside the window; 24 px top fade mask; user scroll
  inside detaches the window's follow (re-attach at bottom).
- **Toggle:** chevron rotates 90° (› ↔ ▾) 160 ms `spring.snap`; body height animates `dur.collapse`
  (grid 0fr ↔ 1fr style; Claude desktop expands in .2 s, Cline 250 ms max-height, note 08); body
  opacity 0 ↔ 1 over 120 ms, starting 40 ms after height on open, ending before height on close.
- **Auto-collapse:** 400 ms wait → header verb rolls to past tense (240 ms) while the body folds (200 ms);
  shimmer finishes its current sweep ≤ 400 ms or fades.
- **Hover (done):** header text to full fg (from 85 %), chevron nudges 2 px in the open direction
  (120 ms); tooltip after 600 ms: "Thought for 12 s · 1,240 tokens" when known.
- **Redacted 🔒:** lock fades in with the past-tense roll; no special animation.
- **Interrupted:** header swaps to "Stopped thinking after 8s" with the status strip's drop timing
  (220 ms `ease.firm`); body collapses immediately (no 400 ms wait).
- **Level switch:** all unpinned blocks animate together (one batch, no stagger) plus the 1.5 s toast.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Appear (header) | 180 ms, y +4 → 0 | `show_delay` 300 ms | `ease.out` | — | 60 Hz |
| Body window grow | continuous with content, cap 160 px | — | `spring.snap` per line | — | 60 Hz |
| Header shimmer | per `text-shimmer/calm` | 600 ms after text change | `ease.coast` | — | 30 Hz |
| Timer tick | 1 Hz, digit roll 280 ms | — | `ease.out` | 30 ms | 30 Hz |
| Live subtitle swap | 180 ms | ≥ 2 s since last swap | `ease.out` | — | 30 Hz |
| Toggle chevron | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Expand / collapse height | 200 ms (`dur.collapse`) | 0 | `cubic-bezier(0.22, 1, 0.36, 1)` (`ease.out`) | — | 60 Hz |
| Body opacity (open / close) | 120 ms | 40 ms / 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Auto-collapse | 400 ms wait + 200 ms | reader guard + 1500 ms | `ease.in-out` | — | 60 Hz |
| Present → past roll | 240 ms (`dur.tense.roll`) | after shimmer ends | `ease.out` | — | 60 Hz |
| Interrupted swap | 220 ms | 0 | `ease.firm` | — | 60 Hz |
| Level-switch batch | 200 ms + toast 1.5 s | 0 | `ease.in-out` | none | 60 Hz |
| **Reduced motion** | no shimmer (plain "Thinking"), timer digits swap without roll, height snaps with an 80 ms opacity crossfade, no chevron rotation (glyph swap), subtitle swaps by 80 ms crossfade every ≥ 4 s, auto-collapse still waits 400 ms then snaps | — | linear | none | 1 Hz timer |

## 6. Variants

### 6.1 Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `reasoning-block/disclosure` | Header row "Thought for 12s ›", body indented under a 2 px leading rule; live window 160 px (default) | chat, desktop, ide |
| `reasoning-block/inline-summary` | Never expands while live: header + live subtitle only; expandable after done (good for consumer chat) | chat, mobile |
| `reasoning-block/card` | Bordered card, timer right-aligned in a reserved slot, "Summary" / 🔒 chips in the header | desktop, research |
| `reasoning-block/dense` | 20 px single row, `terse` verb pack, body opens as a popover instead of inline (narrow panels) | ide (≤ 320 px) |
| `reasoning-block/terminal` | `◇ Thinking (8s)` dim line → `◇ thought for 12s`; body as dim indented lines; key hint "ctrl+o" | terminal |

### 6.2 Pro (15)
| Name | One-line idea |
|---|---|
| `reasoning-block/timeline` | Merges segments and tool steps into one thinking timeline: present → past verbs, active ring, dimmed dead ends, ✓ summary row [R021] |
| `reasoning-block/stream-window` | Body scrolls by two lines at a time in a fixed two-line window, like a ticker (transitions.dev "reasoning stream", note 17 — idea only) |
| `reasoning-block/search-trace` | Sources found while thinking stream in as favicons under the header; steps appear one by one (affectUI Thinking State, note 11) |
| `reasoning-block/orb-header` | A 16 px thinking orb replaces the chevron and morphs to a still mark on done (family B handoff) |
| `reasoning-block/budget-bar` | Thin bar under the header: reasoning tokens used vs the effort budget, fills while thinking |
| `reasoning-block/decision-tree` | Reasoning rendered as branches; abandoned branches fold and dim, risky ones marked ([R060]) |
| `reasoning-block/margin` | Reasoning lives in a right margin, each segment aligned to the answer paragraph it preceded |
| `reasoning-block/ghost-overlay` | While live, reasoning shows faint where the answer will appear, then dissolves as the answer arrives |
| `reasoning-block/step-cards` | Summary steps become small cards that stack and compress into one "Thought for 12s" chip |
| `reasoning-block/inspector` | Verbose dev view: raw text, per-segment timings, token counts, provider kind, copy JSON |
| `reasoning-block/critic` | Two voices (planner / critic) in alternating rows with distinct markers (Jules critic, note 03) |
| `reasoning-block/redaction-veil` | Redacted reasoning shown as a dithered veil with a lock; the veil's size is fixed, not proportional (leaks nothing) |
| `reasoning-block/blur-peek` | Collapsed done block shows the first line blurred; hovering sharpens it for a peek without expanding |
| `reasoning-block/summary-chips` | Summary step titles become a row of chips; click one to expand only that step |
| `reasoning-block/effort-tint` | Header tint and glyph follow the effort level the turn used (Codex effort ignition colour idea, note 08) |

## 7. Edge cases
- **Very short thinking (< 300 ms):** never shown. **300 ms – 1 s:** "Thought briefly".
- **Very long thinking (> 10 min):** timer switches format (`10m 04s`, `1h 02m` per elapsed-timer);
  live window keeps the tail only (older lines virtualised); subtitle swaps slow to every 4 s (proposed).
- **Huge reasoning bodies (> 50 KB):** expanded `Done` view virtualises like any message body; "Show
  all" past 40 lines.
- **Thinking ↔ tool bouncing every few hundred ms:** the 400 ms collapse delay absorbs it; if a turn
  produces > 6 segments, segments after the 6th auto-merge into the previous block with a thin divider
  (proposed) to avoid a wall of headers.
- **Reasoning after the answer started** (some models interleave): the new segment appears below the
  answer text so far, collapsed in `Normal`/`Thinking` levels ("Thought for 3s ›"), never above it.
- **Redacted then raw in the same turn:** each segment keeps its own kind.
- **Timestamps missing:** header "Thought" without a duration; never show "Thought for 0s".
- **Offline / stalled mid-thinking:** the status row and header stall ramp handle it; the block keeps
  its partial body.
- **RTL:** header mirrors (chevron points to the inline-start when collapsed: ‹ in RTL), duration in a
  bidi isolate; body follows text direction; leading rule moves to the right.
- **CJK:** subtitle truncation counts graphemes ×1.6 weight (status-text §5.6).
- **Tiny sizes:** `reasoning-block/dense` 20 px row; at < 240 px the header shows only the timer and
  a 12 px glyph.
- **Dense IDE:** default level `Normal` for IDE presets (proposed) — side panels are short.
- **Touch:** whole header is the toggle (≥ 44 px tall on mobile); live window scrolls inside with axis
  lock.
- **Background tab:** body reveal pauses; on return, received text appears with one block fade
  (text-reveal); the collapse that would have happened applies instantly.
- **Copy:** message copy excludes reasoning except in `Verbose`; the block's own `⋯ → Copy reasoning`
  copies visible (non-redacted) text only.

## 8. Accessibility
- Header is a **disclosure button** (`aria-expanded`, `aria-controls` → body region). Accessible name by
  state: "Thinking, 8 seconds" (live) / "Thought for 12 seconds, summary, collapsed" / "Thought for 12
  seconds, reasoning hidden by the provider".
- **Body:** `role="region"` labelled "Reasoning"; while live `aria-busy="true"`; **never** `aria-live`
  — reasoning is not read aloud as it streams.
- **Announcements** (composer polite live region): "Thinking" when the block appears (only if no other
  status announcement in the last 10 s), "Thought for 12 seconds" once at the roll (throttled with the
  status-text rules); level switch: "Reasoning display: Verbose".
- **Auto-collapse never moves focus.** If focus is inside the body when a collapse would happen, the
  reader guard defers it; if the user collapses manually, focus returns to the header.
- **Contrast:** body text uses `text.secondary` ≥ 4.5:1; the live fade mask covers only already-read
  lines and never sits over the newest line; the 🔒 and "Summary" chips carry text labels.
- **Reduced motion:** §5 last row; the live timer is the liveness cue.
- **Keyboard:** Tab reaches the header; Enter/Space toggles; inside the body, arrow keys scroll the live
  window; Ctrl+O level cycle is announced and remappable.

## 9. Props sketch
```rust
pub struct ReasoningBlock {
    pub segment: ReasoningSegment,           // { id, kind: ReasoningKind, text: StreamHandle, started: Instant, ended: Option<Instant>, tokens: Option<u32> }
    pub agent: AgentState,                   // Thinking → Live; Streaming/ToolRunning → SegmentEnd; Cancelled → Interrupted;
                                             // Error → Failed; Stalled → Stalled overlay; Done → Done
    pub level: DisplayLevel,                 // Normal | Thinking (default) | Verbose  — thread-level, passed down
    pub variant: ReasoningVariant,           // Disclosure | InlineSummary | Card | Dense | Terminal | Pro(..)
    pub header: HeaderCfg,                   // verb pack id, timer variant, shimmer variant (status-text family)
    pub live_window: Option<Px>,             // Some(160 px); None = uncapped (Verbose)
    pub collapse: CollapseCfg,               // { delay: 400 ms, reader_guard: 1500 ms, scroll_guard: 2 s, on_tool_start: true }
    pub show_delay: Duration,                // 300 ms (loaders.md)
    pub subtitle: SubtitleCfg,               // { min_interval: 2 s, max_chars: 60 }
    pub merge_after_segments: Option<u8>,    // 6
    pub pinned: bool,                        // set by user toggle; host persists per turn
    pub labels: ReasoningLabels,             // thinking, thought_for, briefly, summary, hidden_notice, stopped_after, failed
    pub motion: MotionLevel,
    pub on_toggle: Callback<bool>,
}

pub enum ReasoningKind { Raw, Summary, Redacted { partial: bool }, Encrypted, None }

pub enum ReasoningPhase {
    Hidden, Pending, Live { expanded: bool }, AutoCollapsing, Done { expanded: bool }, Briefly,
    Interrupted { after: Duration }, Failed,
}

impl ReasoningPhase {
    /// Pure: AgentState + segment timestamps + level + pinned + ReaderGuard → phase. The shared clock supplies `now`.
    pub fn derive(agent: &AgentState, seg: &ReasoningSegment, level: DisplayLevel,
                  pinned: bool, guard: &ReaderGuard, now: Instant) -> Self { /* … */ }
}
```

## 10. Related components and references
Related: [status-text.md](status-text.md) (`status-text/thought-header`, tense roll) ·
[verb-cycler.md](verb-cycler.md) · [elapsed-timer.md](elapsed-timer.md) ·
[text-shimmer.md](text-shimmer.md) · [text-reveal.md](text-reveal.md) · [loaders.md](loaders.md)
(`show_delay`, transcript-tail loader) · [composer-effort-control.md](composer-effort-control.md)
(`budget-bar`, `effort-tint`) · [message-bubble.md](message-bubble.md) · family F (tool rows between
segments).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Pitus thinking timeline ("Thinking 1s…13s" → "Thought for 14s", present → past steps, dead ends dimmed, auto-collapse) | [R021], [R059] | `timeline`, tense roll | Study only |
| Claude desktop Normal / Thinking / Verbose density switch (Ctrl+O), `showThinkingSummaries` setting; thinking expand .2 s | notes 03, 08 (RE) | display levels; expand timing reference | Proprietary — idea / numbers reference only |
| Warp "Thinking…" header → "Thought for…" | notes 03, 08 | collapse-on-answer | AGPL — study only, re-implement |
| Cline ThinkingRow (5 s shimmer, 250 ms max-height expand into a 150 px scrolling area, 500 ms grace before "Thinking…") | note 08 | live window, grace | Apache-2.0 — may port with attribution |
| Roo reasoning block (live seconds counter + chevron) | note 08 | header timer | Apache-2.0 (archived) — may port with attribution |
| Codex reasoning header shimmer (600 ms delay, 1 s sweep every 4 s); model-written headers; effort ignition | note 08 | subtitle priority, `effort-tint` | Apache-2.0 per note 08 (MIT in licensing.md — re-check) |
| Vercel AI Elements Reasoning / Chain of Thought; prompt-kit Reasoning / Thinking Bar; assistant-ui reasoning group | notes 01, 04 | API shape (auto-open while streaming, collapse after) | Apache-2.0 / MIT / MIT — may port with attribution |
| affectUI Thinking State (live timer, searched sources, steps one by one) | note 11 | `search-trace` | Paid — study only |
| Beautiful UI Thinking (Steps / Reasoning / Search / Coding traces) | [R133], note 17 | trace modes | MIT — may port with attribution |
| transitions.dev reasoning stream (two lines at a time) | [R132], note 17 | `stream-window` idea | Custom licence forbids competing kits — study only, no code |
| Jules critic | note 03 | `critic` | Proprietary — idea only |
| flohoeller decision tree | [R060] | `decision-tree` | Study only |
