**Status:** draft v0.1 (2026-11-11).

# Spec: messages & reasoning (family E overview — transcript, turns, jump-to-latest)

Overview of family **E. Messages & reasoning** in [../component-taxonomy.md](../component-taxonomy.md)
(target "40+"; this draft lands **163**). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). This file owns what every family-E part shares:
**message anatomy, roles, turn grouping, the transcript scroller (virtualisation and scroll anchoring),
the jump-to-latest pill, family motion tokens and the free/Pro split**. It also specifies two
components of its own: `transcript` and `jump-pill`.

| Spec | Component(s) | Variants (free + Pro) |
|---|---|---|
| this file | `transcript` (scroller, turn layout), `jump-pill` | 4 + 10, 3 + 8 = **25** |
| [message-bubble.md](message-bubble.md) | `message` (user / agent / system shells), `message-actions` (copy, edit, retry, branch ‹ 2/3 ›, feedback) | 5 + 15, 4 + 12 = **36** |
| [streaming-markdown.md](streaming-markdown.md) | `markdown` (block renderer), `code-block` | 3 + 8, 4 + 12 = **27** |
| [reasoning-block.md](reasoning-block.md) | `reasoning-block` (collapsible thinking, display levels) | 5 + 15 = **20** |
| [citations-sources.md](citations-sources.md) | `citation`, `source-list`, `confidence-badge`, `memory-receipt`, `follow-ups`, `empty-state` | 18 + 37 = **55** |
| **Family E total** | | **46 free + 117 Pro = 163** (table in §7) |

All values marked *proposed* are Oxyn's own starting points, tuned later in the Studio. Numbers
attributed to products come from the research notes or source IDs named next to them.

## 1. Purpose
The transcript is where the user **reads what the agent did and decides what to do next**. In an
agent app it is not a chat log: one user message can produce minutes of interleaved reasoning, tool
rows, approvals, partial answers and a final answer, and the user will scroll back through hours of it,
copy code out of it, branch it, and grade it. Every product we studied converges on the same pieces —
message action bars, "Thought for Ns" reasoning, inline citations, follow-up chips, edit-to-branch
arrows, auto-scroll with a "jump to latest" button (note 04 §2c, 2e, 2g; note 03 §1C) — and the free
agent layer is commoditised across 6–12 React kits (note 01). What none of them gives a Rust app is the
**agent-shaped transcript**: turns that hold a whole run, a scroller that never fights the reader while
tokens stream, and every part driven by the same `AgentState`.

Family rules (apply to every part):
1. **The transcript never moves under the reader.** Content above the viewport never changes height
   without the scroller compensating (anchor to the first visible block). Collapsing a reasoning block,
   loading a source card or re-widening a table above the fold must not shift what the user is reading.
2. **One state.** Parts read `AgentState` plus their own message record; none keeps a private copy of
   agent state or its own timer (shared motion clock, [../architecture.md](../architecture.md)).
3. **Streaming is a property of one message at a time.** Only the live tail of the active turn
   re-renders per frame; everything else is settled and frozen ([text-reveal.md](text-reveal.md) §5).
4. **Actions appear once per turn, at the end.** Copy / retry / feedback belong to the agent turn, not
   to each fragment between tool rows.
5. **Screen readers get messages, not tokens.** Completion is announced once through the composer's
   single live region ([composer.md](composer.md) §9).
6. **Every part has a reduced-motion fallback and a text label per state.**

## 2. Anatomy

### 2.1 Message anatomy (every role uses the same slots; variants fill or hide them)
```
 ┌ gutter ┐┌ header ─────────────────────────────────────────────┐
 │ avatar ││ name / role · model chip · timestamp (on hover)  ⋯ │  ← header row (optional by variant)
 │ (32px) │├──────────────────────────────────────────────────────┤
 │        ││ attachments (user) / reasoning block (agent)        │  ← pre-body slot
 │        ││ body: markdown blocks · code blocks · tool rows     │  ← body (streaming-markdown.md)
 │        ││       · inline citations [1] · gen-UI blocks        │
 │        ││ partial / error marker row                          │  ← status slot ("Interrupted", "Failed")
 │        ││ sources strip · memory receipt                      │  ← post-body slot (citations-sources.md)
 │        │├──────────────────────────────────────────────────────┤
 │        ││ ⧉ ↻ 👍 👎 ‹ 2/3 › · Worked for 1m 12s               │  ← footer: actions + turn summary
 └────────┘└──────────────────────────────────────────────────────┘
            follow-up chips (last agent turn only)                    ← trailing slot
```

### 2.2 Roles
| Role | What it is | Default look (proposed) | Owns actions |
|---|---|---|---|
| `User` | what the user sent (text, attachments, mentions, slash commands as chips) | trailing-aligned bubble in chat; flat with a "You" label in IDE / terminal | copy, edit-and-resend, branch arrows |
| `Agent` | everything the agent produced for one turn: reasoning, prose, code, tool rows, gen-UI | flat, full width, leading-aligned (no bubble) | copy, retry / regenerate-with, feedback, branch arrows, branch-to-new-thread |
| `System` | app or harness notices: model switched, context compacted, handoff to another agent, permission changed, memory toggled, "Interrupted" | centred thin row with an icon, muted text, no avatar | optional single action ("Undo", "View summary") |
| `Tool` | a tool call and its result | **rendered by family F** (tool row / card); family E reserves the slot and its turn position | owned by family F |
| `Event` (sub-type of System) | run boundary markers: "Worked for 1m 12s", "Stopped · model error", compaction ("Context compacted · View"), "Agent changed" (note 04 §2c) | same as System, inline with the turn | expand / view |

A host may add **named agents** (multi-agent threads, sub-agents): `Agent { id, name, colour_token }`
selects avatar and name; the role stays `Agent`.

### 2.3 Turns and grouping
A **turn** = one user message (or one queued message dispatched from the queue stack) + everything the
agent produced until the next user message is dispatched.

```
 Turn 7 ──────────────────────────────────────────────────────────────
   User      "Add refresh-token rotation"                ‹ 1/2 ›
   Agent ─┬─ reasoning-block  "Thought for 12s"
          ├─ prose block
          ├─ tool rows (family F) — "Explored 6 files", "Edited auth.rs +24 −3"
          ├─ approval card (family G)               ← run paused here, AwaitingApproval
          ├─ prose block (answer)
          └─ footer: ⧉ ↻ 👍 👎 · Worked for 1m 12s   ← once, after Done
   follow-ups (if last turn)
 ──────────────────────────────────────────────────────────────────────
```
Rules (proposed):
- **Agent fragments of one turn render as one message** with one avatar and one footer. Reasoning and
  tool rows sit between prose blocks in arrival order (Antigravity's "Worked for 31s ›" groups between
  paragraphs, note 08).
- **Consecutive tool rows** collapse into a rollup ("Explored 6 files") owned by family F; family E only
  guarantees the rollup sits inside the agent message.
- **Chat grouping:** consecutive user messages sent within **5 min** share one avatar and header; the
  gap between grouped messages is 4 px, between turns 24 px (desktop) / 16 px (IDE, mobile).
- **Day separators** ("Today", "Yesterday", weekday, then date) are sticky at the top of the viewport
  while their day is in view (pattern in Arc's chat thread and coss ui, notes 12, 01).
- **Queued user messages** stay in the composer queue stack until dispatched; on dispatch the chip flies
  into the transcript as a user message (owned by [composer-queue-stack.md](composer-queue-stack.md)).
  A message sent mid-run that the agent has not read yet renders at 60 % opacity until the agent picks
  it up — the "read receipt" fade (Conductor, note 08).
- **Steer notes** render as a small user sub-row inside the current agent message ("↳ also check the
  refresh-token path · steered"), not as a new turn.

## 3. State machine (transcript level)

```
                    open thread
  Empty ──────────────────────────────▶ Loading(history) ── first page ──▶ Resting
  (empty-state, starter prompts)          │  error ──▶ LoadFailed(retry)       │
    │ first send                          ▼                                   │ send / queue dispatch
    └──────────────────────────────▶ Following ◀──── user returns ≤ 48 px ────┤
                                       │  ▲   (pill click / End)               │
              user scrolls up > 48 px  │  │                                    ▼
                                       ▼  │                               Following (turn active)
                                   Detached(new = n) ── run ends ──▶ Detached(idle)
                                       │                                  │
                                       │ scroll to top edge               │
                                       ▼                                  │
                                 Prepending(older page) ── loaded ──▶ (back, anchored to first visible)
  Overlays: Searching(match k/n) · Selecting(text) pauses follow · Offline (cached pages only)
```

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Empty` | empty state + starter prompts ([citations-sources.md](citations-sources.md) §empty-state) | pick a starter | — | Tab into starters; typing goes to the composer |
| `Loading(history)` | message skeletons (2 user + 2 agent shapes, [skeletons.md](skeletons.md)) after 150 ms | — | — | — |
| `LoadFailed` | inline row "Couldn't load this thread · Retry" | retry | open offline copy (if cached) | Enter |
| `Resting` | transcript at bottom, no run | read / act on messages | — | PageUp/PageDown, Home/End, `[`/`]` previous/next turn (proposed) |
| `Following` | viewport follows the live tail (text-reveal §7 rules) | Stop (send control) | — | Esc stops the run (owned by composer) |
| `Detached(new = n)` | user reading above; `jump-pill` visible with "↓ n new" | jump to latest | — | End or the pill's shortcut (⌘/Ctrl+↓, proposed) |
| `Detached(idle)` | pill shows "↓ Jump to latest" without count | jump | — | End |
| `Prepending` | "Loading earlier messages" row at top (skeleton, 1 row) | — | — | — |
| `Searching` | match highlights, "3 / 12" chip, matched message expanded | next / previous match | close | Enter / Shift+Enter, Esc |
| `Selecting` | user selection held; follow paused | copy | quote-reply (Pro) | ⌘/Ctrl+C |

## 4. Virtualisation and scroll anchoring

### 4.1 Virtualisation (proposed)
- **Unit = a block, not a message.** An agent turn can be 50 viewports tall; we virtualise its
  settled blocks (paragraphs, code blocks, tool rows) individually so a long turn does not mount whole.
- **Height cache:** measured heights keyed by `(block_id, width_bucket, font_scale)`; width buckets are
  16 px wide so a 1 px resize does not invalidate everything. Unmeasured blocks use an estimate:
  `lines ≈ ceil(chars × avg_advance / width)` × line height; code blocks `lines × code_line_height + 40 px`
  chrome; images their known aspect ratio or 16:9.
- **Overscan:** 1 viewport above and 2 below (more below because the reader usually continues down).
- **Never virtualised:** the live tail of the active turn, any block containing focus or an active text
  selection, a reasoning block the user has expanded in the viewport, and the jump-pill target.
- **Find in thread** (⌘/Ctrl+F) searches the message store, not the DOM / view tree, then mounts and
  scrolls to the match.
- **Thresholds:** below **200 blocks** the GPUI list renders everything (no virtualisation cost); above
  it switches to the virtual list. GPUI: a uniform/variable-height list primitive with our height cache
  (exact GPUI list API to confirm at implementation); Dioxus web: absolutely positioned blocks in a
  spacer of the summed height.

### 4.2 Scroll anchoring (rules shared with [text-reveal.md](text-reveal.md) §7 — not repeated here)
Follow mode (48 px threshold, caret line kept ≥ 24 px above the bottom, detach on scroll-up > 48 px,
selection pauses follow) is specified in text-reveal §7. The transcript adds:

| Situation | Rule (proposed) |
|---|---|
| Height change **above** the first visible block (estimate → measurement, reasoning auto-collapse, source cards loading, table re-widen) | adjust `scroll_top` by the delta in the same frame; never animate the compensation |
| Height change **inside** the first visible block | anchor to the first visible *line* of that block (offset within the block is preserved) |
| Prepending older history | anchor to the first visible message; the "Loading earlier" row is inserted above it and compensated |
| User sends a message while detached | jump to the new user message, positioned **≤ 33 % from the top** of the viewport so the agent's answer has room below (proposed; avoids the "answer starts at the very bottom" feel) |
| Thread opened | land on the last turn's **user message top** if the last agent message is taller than the viewport, otherwise at the bottom (proposed) |
| Deep link to a message / citation target | land with the target 25 % from the top; flash its background (`highlight.flash` §6) |
| Window resize mid-stream | first visible line stays in place (text-reveal §7) |
| Reduced motion | all programmatic scrolls instant |

## 5. Jump-to-latest pill (`jump-pill`)
Owned by this spec; text-reveal's "↓ New output" pill is this component.

| State | Shows | Action | Keyboard |
|---|---|---|---|
| `Hidden` | nothing (following, or within 48 px of bottom) | — | — |
| `Idle` | "↓" + "Jump to latest" (label hidden below 360 px width) | smooth scroll to bottom | End, ⌘/Ctrl+↓ |
| `NewContent(n)` | "↓ n new" — n = new **messages** when ≥ 1 message arrived, else rolling **line** count of the live tail | jump to the first unread block, not the very bottom, when n ≥ 2 messages (proposed) | same |
| `Streaming` | pill carries a 12 px loader still frame or the lead loader if the composer region is hidden ([loaders.md](loaders.md) §2.3: mirror by default) | jump | same |
| `AwaitingYou` | amber dot + "Needs your approval ↓" | jump to the approval card | same |
| `Error` | danger dot + "Run failed ↓" | jump to the error row | same |

- Position: bottom-centre of the transcript, 16 px above the composer's top edge; it rides the composer's
  top edge when layers grow (composer rule "grows upward", [composer.md](composer.md) §1).
- **Hysteresis:** shows after the user is > 48 px from the bottom for 300 ms (proposed, avoids flashing
  during fling scrolls); hides immediately on reaching the bottom.
- Unread marker: when the user returns after ≥ 2 new messages, a thin "New" divider stays above the
  first unread message for 10 s or until scrolled past (proposed; coss ui's unread marker, note 01).

## 6. Micro-interactions (family-wide)
- **New user message:** arrives via the composer's fly-up (≤ 260 ms, [composer.md](composer.md) §4); the
  transcript reserves its final height in the same frame so nothing jumps.
- **Agent message appears:** the status line / thinking row occupies the agent slot first ([loaders.md](loaders.md)
  §3.3 `show_delay` 300 ms); first content replaces it in place (no slide).
- **System / event row:** fades in with a 4 px rise, 180 ms.
- **Hover:** message footer actions fade from 0 → 100 % (desktop); timestamp appears in the header.
  Touch surfaces show actions persistently on the last agent turn and on long-press for others (pattern
  "hover on desktop, persistent on mobile", note 04 §2g).
- **Jump pill:** slides up 8 px + fade (160 ms `ease.out`); count rolls with `dur.digit.roll`; on click
  the pill shrinks to 0.9 and fades as the scroll begins.
- **Deep-link flash:** target block background tints with `color.highlight` at 24 % alpha, holds 600 ms,
  fades 900 ms.
- **Day separator:** sticky header crossfades between days (120 ms) rather than pushing.

### Family-E motion tokens (proposed; shared tokens keep their [composer.md](composer.md) values)
| Token | Value | Used for |
|---|---|---|
| `dur.actions.reveal` | 120 ms fade, 80 ms hover intent delay | footer actions on hover / focus |
| `dur.actions.hide` | 160 ms, 300 ms after pointer leaves | footer actions hide |
| `dur.copy.confirm` | 1500 ms hold, then 160 ms revert | copy → ✓ |
| `dur.collapse` | 200 ms (grid-rows 0fr ↔ 1fr style height spring) | reasoning, long output, sources expand / collapse (Claude desktop expand .2 s, note 08) |
| `delay.preview.open` / `.close` | 300 ms / 150 ms grace | citation hover cards, timestamp tooltips |
| `dur.version.swap` | 180 ms crossfade + height `spring.snap` | branch ‹ 2/3 › switch |
| `dur.row.enter` | 180 ms, y +4 → 0 | system rows, source cards, chips |
| `stagger.chips` | 40 ms, cap 5 | follow-ups, starter prompts, source cards |
| `highlight.flash` | 600 ms hold + 900 ms fade | deep links, search hits, citation targets |
| **Reduced motion** | all of the above → 0–80 ms opacity crossfades; heights snap; no stagger; scrolls instant; the copy ✓ still holds 1500 ms (it carries meaning) | — |

## 7. Variants

### 7.1 `transcript` — Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `transcript/chat` | Centred column max 760 px; user bubbles trailing, agent flat; day separators; 24 px turn gap | chat, desktop, mobile |
| `transcript/ide-panel` | Full-width flat rows, 12 px turn gap, user rows with a 2 px accent left rule, no avatars; tool rollups collapsed | ide |
| `transcript/terminal` | Scrollback look: `›` user prompt lines, agent text unprefixed, `•` tool rows, `└` details (Codex-style, note 08) | terminal |
| `transcript/document` | No bubbles; each turn is a section with the user prompt as a heading and the answer as the body (research / report agents) | desktop, chat |

### 7.2 `transcript` — Pro (10)
| Name | One-line idea |
|---|---|
| `transcript/split-canvas` | Answer prose left, the artifact / gen-UI it produced pinned right; scrolling syncs the two |
| `transcript/timeline-rail` | A vertical time rail in the gutter; turn dots sized by duration; scrub the rail to jump |
| `transcript/minimap` | Right-edge minimap of turns coloured by role / state (approval amber, error red), click to jump |
| `transcript/turn-cards` | Each turn is a collapsible card with its summary line ("Worked for 1m 12s · 3 files"); older turns auto-fold |
| `transcript/lanes` | Multi-agent: one column per agent, user messages span all lanes [R061] |
| `transcript/threaded` | Side questions ("btw") branch into an indented sub-thread instead of the main line (note 03 §1B) |
| `transcript/focus-mode` | Only the active turn at full contrast; earlier turns dim to 60 % until hovered |
| `transcript/replay` | Read-only run replay with a scrubber and "Jump to live" (Manus pattern, note 04 §2d) |
| `transcript/compact-dense` | 18 px rows, single-line previews of each agent turn that expand in place (IDE side panels ≤ 300 px) |
| `transcript/voice-captions` | Voice-first: big current utterance, earlier turns shrink into a caption history (family K handoff) |

### 7.3 `jump-pill` — Free (3) / Pro (8)
| Name | Tier | Idea |
|---|---|---|
| `jump-pill/arrow` | Free | Round 32 px ↓ button; count badge when new content |
| `jump-pill/labelled` | Free | "↓ 3 new" text pill (default) |
| `jump-pill/edge-bar` | Free | Full-width 24 px bar on the composer's top edge: "New output below" (dense IDE) |
| `jump-pill/live-preview` | Pro | Pill shows the last streamed line, fading, so the reader can decide whether to jump |
| `jump-pill/loader` | Pro | Lead loader inside the pill while the agent works off-screen |
| `jump-pill/stacked-avatars` | Pro | Multi-agent: avatars of agents that posted while detached |
| `jump-pill/progress` | Pro | Ring around ↓ shows how far below the latest content is |
| `jump-pill/magnetic` | Pro | Pill pulls toward the pointer within 40 px (desktop) |
| `jump-pill/attention` | Pro | Distinct amber / red forms for "Needs your approval" / "Run failed" with one 400 ms pulse |
| `jump-pill/edge-glow` | Pro | No pill; the composer's top edge glows and a ↓ appears on hover |
| `jump-pill/notch` | Pro | Desktop island variant: the pill lives in the window's notch/status HUD |

### 7.4 Family-E catalog (all 163)
| Component | Spec | Free | Pro | Total |
|---|---|---|---|---|
| `transcript` | this file | chat, ide-panel, terminal, document | split-canvas, timeline-rail, minimap, turn-cards, lanes, threaded, focus-mode, replay, compact-dense, voice-captions | 14 |
| `jump-pill` | this file | arrow, labelled, edge-bar | live-preview, loader, stacked-avatars, progress, magnetic, attention, edge-glow, notch | 11 |
| `message` | message-bubble | 5 | 15 | 20 |
| `message-actions` | message-bubble | 4 | 12 | 16 |
| `markdown` | streaming-markdown | 3 | 8 | 11 |
| `code-block` | streaming-markdown | 4 | 12 | 16 |
| `reasoning-block` | reasoning-block | 5 | 15 | 20 |
| `citation` | citations-sources | 3 | 7 | 10 |
| `source-list` | citations-sources | 3 | 7 | 10 |
| `confidence-badge` | citations-sources | 3 | 5 | 8 |
| `memory-receipt` | citations-sources | 3 | 6 | 9 |
| `follow-ups` | citations-sources | 3 | 6 | 9 |
| `empty-state` | citations-sources | 3 | 6 | 9 |
| **Total** | | **46** | **117** | **163** |

Free/Pro rule (proposed, same as families B–D): **the free tier covers every surface** (chat, IDE,
terminal, mobile) with complete states and accessibility for every component; Pro adds structural
alternatives, expressive motion and vertical-specific layouts. No state, keyboard path or a11y label is
Pro-only.

## 8. Edge cases (family level)
- **Very long threads (10 000+ messages, 100 MB text):** block virtualisation (§4.1); history paged in
  50-turn pages (proposed); search runs on the store.
- **Very long single turns (hour-long runs):** older blocks inside the turn virtualise; the turn header
  ("Turn 7 · Worked for 1h 02m") becomes sticky while the turn is in view (proposed).
- **Very fast agent:** answer completes before the status row's 300 ms `show_delay` — the row never
  shows; footer actions fade in after `Done`.
- **Very slow agent:** status row and loaders handle liveness; transcript only follows.
- **Out-of-order / late events** (a tool result arriving after the next prose block): blocks are
  ordered by the adapter's sequence id, not arrival time; a late block inserts in place and the
  anchor rule compensates if above the viewport.
- **Edits of earlier messages while a run is active:** see message-bubble §edit (stop-then-branch).
- **RTL:** chat layout mirrors (user bubbles on the left); code, numbers, diff stats stay LTR in bidi
  isolates; the jump pill stays centred.
- **Tiny / dense:** `transcript/ide-panel` at 280 px: avatars and headers off, actions in a `⋯` menu.
- **Touch:** long-press a message → action sheet; swipe-right on a user message = quote/reply (Pro).
- **Slow networks / offline:** cached pages render; uncached older pages show "Offline · Earlier
  messages unavailable"; sends queue in the composer (composer edge cases).
- **Mixed content heights** (images arriving late): images reserve aspect-ratio boxes from metadata;
  unknown ratio = 16:9 at 40 % column width (same rule as text-reveal §5) and the anchor compensates.

## 9. Accessibility (family level)
- **Transcript:** `role="feed"` (WAI-ARIA feed pattern) with `aria-busy="true"` while a page loads;
  each message is `role="article"` with `aria-labelledby` = author + time ("Agent, 14:02") and
  `aria-posinset` / `aria-setsize` (set size = -1 when unknown). **No `aria-live` on the transcript**
  (a `role="log"` would read tokens aloud — rejected).
- **Keyboard:** PageDown / PageUp move focus between articles (feed pattern); `[` / `]` previous / next
  turn (proposed); within an article, Tab reaches links, code-block buttons, citations, then actions.
  Esc returns focus to the composer.
- **Announcements** (via the composer's single polite live region): "Response complete" once per turn;
  "3 new messages" when detached (throttled 1 per 10 s); errors and "Needs your approval" assertive.
- **Focus never jumps** because content arrived; streaming never steals focus from the composer.
- **Contrast:** body text ≥ 4.5:1 on every bubble fill; muted system rows ≥ 4.5:1 (muted is a hue/
  weight choice, not a contrast cut); the 60 % "unread" fade on mid-run messages keeps ≥ 4.5:1 by
  dimming the fill, not the text (proposed).
- **Reduced motion:** §6 table; scrolls instant.
- **Zoom 200 %:** bubbles reflow, max-width becomes 100 %; no horizontal page scroll (code blocks scroll
  internally).

## 10. Props sketch (shared types)
```rust
pub struct Transcript {
    pub agent: AgentState,                 // same value as ComposerState.agent
    pub turns: TurnStore,                  // paged; Turn { id, user: MessageId, agent: Option<MessageId>, events: Vec<EventRow> }
    pub variant: TranscriptVariant,        // Chat | IdePanel | Terminal | Document | Pro(..)
    pub grouping: Grouping,                // { same_author_window: 5 min, day_separators: true }
    pub virtualize: Virtualize,            // Auto { min_blocks: 200, overscan_above: 1.0, overscan_below: 2.0 } | Off
    pub anchor: ScrollAnchor,              // shared with TextReveal: Follow { threshold: 48 px, margin: 24 px }
    pub land_on_open: LandPolicy,          // LastUserIfTall (default) | Bottom
    pub send_position: f32,                // 0.33 — new user message lands ≤ 33 % from top
    pub jump_pill: Option<JumpPillVariant>,
    pub unread_marker: Option<Duration>,   // 10 s
    pub labels: TranscriptLabels,          // i18n: today, yesterday, jump_to_latest, n_new, loading_earlier
    pub motion: MotionLevel,
}

pub enum Role { User, Agent { id: AgentId }, System(SystemKind), Tool /* rendered by family F */ }

pub enum SystemKind { ModelSwitched, Compacted, Handoff, PermissionChanged, MemoryToggled, Interrupted, RunSummary, Custom(String) }

pub enum TranscriptPhase { Empty, LoadingHistory, LoadFailed, Resting, Following, Detached { new: u32 }, Prepending }

impl TranscriptPhase {
    /// Pure: scroll position + AgentState + store paging → phase. No timers except the shared clock.
    pub fn derive(agent: &AgentState, scroll: &ScrollSnapshot, store: &TurnStore) -> Self { /* … */ }
}
```
`AgentState → transcript`: `Submitted` reserves the agent slot; `Thinking`/`ToolRunning`/`Streaming`
keep `Following` if attached; `AwaitingApproval` changes the pill to `AwaitingYou` when detached;
`Done` fades in footer actions and follow-ups; `Error`/`Cancelled` insert the marker row; `Stalled` is
shown by the status row, not the transcript.

## 11. Related components and references
Related: [composer.md](composer.md) (fly-up, queue dispatch) · [text-reveal.md](text-reveal.md)
(follow mode, live tail) · [status-text.md](status-text.md) (status row, "Worked for") ·
[loaders.md](loaders.md) (busy regions) · [skeletons.md](skeletons.md) (history loading) · family F
(tool rows, rollups) · family G (approval cards) · family H (checkpoints / fork) · family K (captions).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Arc UI chat thread: day separators, own-side rise-in, "Jump to latest" pill springs in, reduced-motion instant scroll | note 12 | pill + separators behaviour | Paid / unclear — study only |
| coss ui chat: sticky dates, unread marker, long-message collapse, loading / offline / error / access-lost states | [R126], note 01 | failure-state vocabulary | MIT — may port with attribution |
| Vercel AI Elements Conversation; shadcn AI Elements MessageScroller | note 01 | stick-to-bottom only when at bottom | AI Elements Apache-2.0 (deprecated end Nov 2026), shadcn MIT — may port with attribution |
| ElevenLabs UI Conversation (auto-scroll, empty state, scroll button); LiveKit transcript "Scroll to end" | note 04 | jump button + empty state | licence not recorded ("open source") — study only until checked |
| assistant-ui messages group (branches, queue, edit-as-branch, stopped run) | note 01 | role / turn model | MIT — may port with attribution |
| Codex CLI transcript rhythm (`›`, `•`, `└`) | note 08 | `transcript/terminal` | Apache-2.0 per note 08 (MIT in licensing.md — re-check) |
| Antigravity "Worked for 31s ›" groups between paragraphs | note 08 | fragments-in-one-turn layout | Proprietary — idea only |
| Conductor read-receipt fade for mid-run messages | note 08 | 60 % unread state | Proprietary docs — idea only |
| Renzo Bianchi "the exit is the confirmation" | [R079] | send → bubble handoff | Check source; study only |
| Manus replay + "jump to live" | note 04 | `transcript/replay` | Proprietary — idea only |
| flohoeller agent lanes | [R061] | `transcript/lanes` | Study only |
