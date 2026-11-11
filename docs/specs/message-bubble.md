**Status:** draft v0.1 (2026-11-11).

# Spec: message bubble & message actions (`message`, `message-actions`)

Part of family E ([messages.md](messages.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **`message` 5 free + 15 Pro** and
**`message-actions` 4 free + 12 Pro** = **36 variants**. Anatomy, roles and turn grouping are in
[messages.md](messages.md) §2; the body renderer is [streaming-markdown.md](streaming-markdown.md); the
reveal effect is [text-reveal.md](text-reveal.md). This spec owns the **message shell** (bubble / flat /
card), its **lifecycle states** (pending, streaming, interrupted, partial, failed, edited, versioned),
and the **action bar** (copy, edit-and-resend, retry / regenerate-with, branch arrows ‹ 2/3 ›, branch
to new thread, feedback thumbs with reason, timestamps).

## 1. Purpose
After the agent answers, the user does one of five things with a message: **copy it**, **fix what they
asked** (edit and resend), **ask again** (retry, maybe with a different model or effort), **compare
attempts** (version arrows), or **grade it** (thumbs with a reason). Every product we studied has this
bar (note 04 §2g: "Action bar — All"; ChatGPT version arrows `< 2/2 >` and "Branch in new chat"; Zed
thumbs with text on down, Copy as Markdown; affectUI Message Actions: quiet until hover, copy blurs into
a check, regenerate spins, thumbs-down unfolds a reason row, "‹ 2 / 3 ›" when versions ≥ 2, note 11).
Agent runs add states chat kits ignore: a message **interrupted** mid-tool, a **partial** answer after a
model error, a message **sent mid-run that the agent has not read yet**, and edits that must **stop the
current run** before they branch. This spec gives every one of those a defined look, action and label.

## 2. State machine

### 2.1 User message
```
          composer send                  adapter ack                 agent picks it up
  (Draft) ─────────────▶ Pending ─────────────────▶ Sent ──────────────────────────▶ Read
                          │  ▲ retry                   │ (mid-run send: Unread 60 % fade until Read)
                          ▼  │                         │
                        SendFailed ── edit ──┐         │ Edit (pencil / ↑ on last / E)
                                             ▼         ▼
                                           Editing ── Esc ──▶ (back to previous state)
                                             │ Save & send (run active → StopConfirm first)
                                             ▼
                                        Resending ──▶ Sent (new version k+1 of n+1; ‹ k+1/n+1 ›)
  Overlays: Hover / Focus (actions visible) · Selected (text selection) · Highlighted (deep link)
```

### 2.2 Agent message
```
             Submitted                 first token / block                tool / approval
  Reserved ─────────────▶ Thinking ───────────────────────▶ Streaming ◀──────────────────▶ Paused
  (status row slot)       (reasoning-block)                   │  ▲   (block closed visually; tool row below)
                                                             │  └── Stalled (status row handles words)
           ┌────────────────────────── Done ─────────────────┤
           ▼                                                 ├── Stop ──────▶ Interrupted(partial)
       Complete ── Retry / Regenerate-with ──▶ Regenerating  ├── Error ─────▶ Failed(partial?)
           │  ▲        (old text 40 %, collapses on          └── length/limit ▶ Incomplete ── Continue ──▶ Streaming
           │  │         first new block)                          Withheld (guardrail / policy notice)
           │  └── ‹ › VersionSwitch (crossfade + height spring)
           ▼
   Feedback overlay: None ─ 👍 ─▶ Up ; None ─ 👎 ─▶ ReasonOpen ─ send/skip ─▶ Thanks (1.2 s) ─▶ Down
```

### 2.3 States table
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| **User** `Pending` | bubble at 100 % with a 12 px clock glyph in the footer after 600 ms without ack | — | cancel (if the adapter allows) | — |
| `SendFailed` | danger outline + "Not sent · Retry" row under the bubble | retry | edit, delete | Enter retry (focused) |
| `Sent` | bubble | — | copy, edit | ↑ in empty composer edits last user message (proposed) |
| `Unread` (sent mid-run) | bubble fill at 60 % + "Queued for the agent" tooltip (Conductor read-receipt, note 08) | — | edit, remove (until read) | — |
| `Read` | bubble at 100 % (fade 200 ms) | — | copy, edit, branch | — |
| `Editing` | bubble replaced in place by an editor (same width, min 3 rows), attachments editable, "Cancel · Save & send" | Save & send | Cancel | Enter / ⌘↵ save & send (follows composer Enter rule), Shift+Enter newline, Esc cancel |
| `StopConfirm` (edit while run active) | inline notice in the editor: "Sending this will stop the current response" | Stop & send | Cancel | Enter / Esc |
| `Resending` | editor collapses into the new bubble; branch arrows appear "‹ 2/2 ›" | — | — | — |
| **Agent** `Reserved` | status row / thinking row slot (owned by status-text, loaders) | Stop (composer) | — | Esc |
| `Thinking` | reasoning block live ([reasoning-block.md](reasoning-block.md)) | Stop | expand / collapse reasoning | Esc |
| `Streaming` | body revealing ([text-reveal.md](text-reveal.md)); actions hidden | Stop | select / copy settled text | Esc |
| `Paused` | body block closed; tool row or approval card below (families F, G) | host's | — | host's |
| `Complete` | full body; footer actions (quiet until hover) + "Worked for 1m 12s" | copy | retry, feedback, branch, more | Tab into actions; `c` copy, `r` retry when message focused (proposed) |
| `Interrupted` | partial body at full contrast + marker row "Interrupted after 42s" | Continue (if adapter supports resume) | Retry | Enter |
| `Incomplete` | body + row "Response cut off (length limit) · Continue" | Continue | Retry | Enter |
| `Failed` | partial body (if any) + danger row "Stopped · model error" + Details disclosure (code, request id) | Retry | Copy error, Details | Enter |
| `Failed` (no tokens) | danger card in place of the body: "Couldn't get a response · Retry" | Retry | Details | Enter |
| `Withheld` | system-style notice "Response withheld by a safety or policy check" + Learn more (host link) (assistant-ui guardrail notice, note 01) | — | Retry, feedback | — |
| `Regenerating` | previous version at 40 % opacity until the new first block lands, then collapses (200 ms) | Stop | — | Esc |
| `VersionSwitch` | "‹ 2 / 3 ›" in the footer; body crossfades to the chosen version | ‹ / › | — | ←/→ when the switcher is focused; Alt+←/→ when the message is focused (proposed) |
| Feedback `ReasonOpen` | row unfolds under the bar: reason chips + optional text (≤ 500 chars) + Send / Skip | Send | Skip | Tab through chips (toggle buttons), Enter send, Esc skip |
| Feedback `Thanks` | "Thanks — this helps" for 1.2 s, then the row folds; thumb stays pressed | — | change vote | — |

## 3. Behaviour rules

### 3.1 Bubble vs flat vs card (structure axis, chosen by variant)
| Look | User message | Agent message | When |
|---|---|---|---|
| **Bubble** | trailing-aligned, fill `color.bubble.user`, radius 18 px (corner nearest the avatar 6 px when grouped), max-width 80 % (chat) / 640 px | flat, full column width | consumer chat (default `message/bubble`) |
| **Flat** | leading-aligned, no fill, "You" label or 2 px accent rule | flat | IDE, docs, research (`message/flat`) |
| **Card** | bordered card per turn; user prompt is the card header | card body | review / report apps (`message/card`) |
Agent output is **never** a narrow bubble by default: code blocks, tables and gen-UI need the full
column (all coding agents in note 03 render agent output flat). Padding: bubble 10 × 14 px; flat 0;
card 16 px (proposed).

### 3.2 Copy
- **Copy** = the message's Markdown source (code fences kept) by default; secondary items in `⋯`:
  "Copy as plain text", "Copy as Markdown", "Copy code blocks only" (Zed offers Copy as Markdown /
  Copy this response, note 03).
- Agent turn copy excludes reasoning and tool rows unless the display level is `verbose`
  ([reasoning-block.md](reasoning-block.md)) — proposed.
- Copy ✓ holds `dur.copy.confirm` (1500 ms); on clipboard failure show ✕ + tooltip "Couldn't copy".

### 3.3 Edit-and-resend
- **In place** by default (the bubble becomes an editor); variant option `EditTarget::Composer` loads
  the text into the composer instead (Codex CLI/Claude Esc-Esc rewind style, note 03).
- Only **user** messages are editable. Editing message *k* creates version *n+1* of message *k*;
  everything after *k* in the old version is kept in that version's branch, not deleted.
- If a run is active in this thread: `StopConfirm` first (app may set `edit_while_busy = StopAndSend |
  Disallow`). Never edit silently under a running agent.
- Attachments: removable / addable in the editor; mentions and slash-command chips stay tokens.
- Edited messages carry "(edited)" in the header tooltip only when the app keeps a single version
  (no branching); with branching the arrows already say it.

### 3.4 Retry and regenerate-with
- **Retry** on an agent message reruns the same user message with the same settings → new agent
  version under the same user message ("‹ 2/2 ›" on the agent footer).
- **Regenerate with…** (menu, ▾ beside ↻): model ▸, effort ▸ (ChatGPT retry menu with model/effort,
  note 04 §2g); uses the composer's pickers' data ([composer-model-picker.md](composer-model-picker.md),
  [composer-effort-control.md](composer-effort-control.md)). The chosen model shows as a chip in the new
  version's header.
- **Continue** is offered only for `Incomplete` and `Interrupted` and only when the adapter declares
  resume support; otherwise only Retry.
- Retry on a **user** message that failed to send = resend, not a new version.

### 3.5 Branch arrows ‹ 2/3 › and branch to new thread
- Versions are a **tree** per message; the arrows show siblings at this node only. Switching a user
  message's version swaps **everything after it** (the old downstream turns belong to the old version).
- Arrows appear only when siblings ≥ 2. Format "‹ 2 / 3 ›" with tabular digits; ‹ disabled at 1, ›
  disabled at n (shown, not hidden, so the control doesn't jump).
- **Branch in new thread** (`⋯` → "Branch from here"): copies turns 1…k into a new thread and opens it;
  the original shows a system row "Branched to ‘<title>’ · Open" (ChatGPT / Raycast branch, note 04;
  Pi's `/tree`, note 03 — tree view is Pro `message-actions/branch-tree`).
- Checkpoint restore / rewind of files is **family H**, not this control; when the app pairs them, the
  edit editor offers "Also restore files to this point" as a checkbox owned by family H.

### 3.6 Feedback
- 👍 / 👎 are toggle buttons (`aria-pressed`); one vote per message version; clicking the pressed thumb
  clears the vote.
- 👎 opens the reason row (always); 👍 opens it only if `ask_reason_on_up` (default off).
- Reason chips (in-house copy, proposed): **Not accurate · Didn't follow instructions · Too long ·
  Too short · Unsafe or harmful · Other**; category presets add one: coding "Code doesn't work",
  research "Bad sources", finance "Wrong numbers". Text field optional, ≤ 500 chars, placeholder "What
  should it have done?".
- Feedback is sent on **Send**, or on **Skip / blur after 10 s** as vote-only (proposed); the reason
  row never blocks reading.

### 3.7 Timestamps
| Age | Shown (header, on hover / always in `card`) | Tooltip |
|---|---|---|
| < 1 min | "now" | full date-time with seconds and zone |
| < 60 min | "12m" | same |
| today | "14:02" (locale 12/24 h) | same |
| yesterday | "Yesterday 14:02" | same |
| < 7 days | "Tue 14:02" | same |
| this year | "12 Oct" | same |
| older | "12 Oct 2025" | same |
Relative labels update on the shared 1 Hz clock only while visible and only at boundaries (once a
minute below 1 h). Agent messages may show the **run duration** ("Worked for 1m 12s",
[status-text.md](status-text.md) §5.4) in the footer instead of a clock time.

## 4. Micro-interactions
- **User bubble enter:** receives the composer fly-up (≤ 260 ms, [composer.md](composer.md) §4); for
  messages not typed here (sync, queue dispatch): scale 0.96 → 1 + y 6 → 0, 200 ms `spring.snap`
  (Claude desktop user bubble enters from scale .92 + 6 px, note 08 — we use a smaller scale).
- **Actions reveal:** after `dur.actions.reveal` hover intent (80 ms) the bar fades in 120 ms; hides
  160 ms after a 300 ms leave delay; focus-within keeps it visible. Last agent turn: always visible.
- **Copy:** icon crossfades + blur 4 → 0 px into ✓ (160 ms), holds 1500 ms, reverts 160 ms (affectUI
  "copy blurs into a green check", note 11 — our timings).
- **Retry:** ↻ rotates 360° once (400 ms `ease.in-out`); keeps rotating at 1 turn / 1000 ms while
  `Regenerating` ([loaders.md](loaders.md) `loop.base`).
- **Thumb vote:** scale 1 → 1.18 → 1 (`spring.soft`, ~240 ms), fill swaps to `color.feedback.on`; the
  opposite thumb dims to 50 % (still ≥ 3:1).
- **Reason row:** unfolds below the bar (`dur.collapse` 200 ms); chips stagger in (40 ms, cap 5); on Send
  the row crossfades to "Thanks" then folds after 1.2 s.
- **Version switch:** number rolls (`dur.digit.roll` 280 ms); body crossfades 180 ms while height springs
  (`spring.snap`) — affectUI's message-body crossfade with height easing, note 11.
- **Edit open:** bubble fill fades to the editor surface (120 ms); width locks to the bubble's width then
  springs to editor width (200 ms); caret at end. **Cancel:** reverse. **Save & send:** editor text
  morphs into the new bubble (same fly as composer send, 260 ms).
- **Interrupted / failed marker:** "Interrupted" drops in from the status tab (owned by
  [composer-status-strip.md](composer-status-strip.md), 220 ms `ease.firm`); failed row tone fades 200 ms;
  no shake on text.
- **Regenerating:** old body to 40 % opacity (160 ms); when the new first block lands, old collapses
  (200 ms) with the anchor rule ([messages.md](messages.md) §4.2).
- **Unread → Read:** fill 60 → 100 % over 200 ms.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| User bubble enter (non-composer) | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Actions reveal / hide | 120 / 160 ms | 80 / 300 ms | `ease.out` / `ease.in` | — | 60 Hz |
| Copy → ✓ / hold / revert | 160 / 1500 / 160 ms | 0 | `ease.out` | — | 60 Hz |
| Retry spin (press / loop) | 400 ms / 1000 ms per turn | 0 | `ease.in-out` / linear | — | 60 Hz |
| Thumb pop | ~240 ms | 0 | `spring.soft` | — | 60 Hz |
| Reason row unfold / fold | 200 / 200 ms | 0 / 1200 ms after Thanks | `ease.out` / `ease.in` | chips 40 ms, cap 5 | 60 Hz |
| Version switch | 180 ms crossfade + height spring; digits 280 ms | 0 | `ease.in-out` + `spring.snap` | — | 60 Hz |
| Edit open / cancel | 120 ms fill + 200 ms width | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Save & send fly | ≤ 260 ms | 0 | `ease.in` | — | 60 Hz |
| Regenerating dim / collapse | 160 / 200 ms | collapse on first new block | `ease.out` / `spring.snap` | — | 60 Hz |
| Unread → Read | 200 ms | 0 | `ease.out` | — | 30 Hz |
| Pending clock glyph | appears after 600 ms | 600 ms | fade 120 ms | — | — |
| **Reduced motion** | all transitions → 0–80 ms opacity crossfade; no pop, no spin (↻ shows a static "…" badge while regenerating), no width spring; copy ✓ still holds 1500 ms; version digits swap without rolling | — | linear | none | on change |

## 6. Variants

### 6.1 `message` — Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `message/bubble` | User bubble trailing, agent flat full-width; grouped corners (default) | chat, mobile, desktop |
| `message/flat` | Both roles flat; user marked by "You" + 2 px accent rule; avatars optional | ide, desktop |
| `message/card` | Each turn a bordered card; user prompt as card header, agent answer as body, footer bar inside | desktop, research |
| `message/terminal` | `›` prefix for user, no chrome, footer actions as key hints | terminal |
| `message/dense` | 13 px text, 4 px gaps, avatar → 16 px glyph, header inline with first line | ide (≤ 320 px) |

### 6.2 `message` — Pro (15)
| Name | One-line idea |
|---|---|
| `message/liquid-glass` | Refractive glass user bubble that stretches slightly on send (study [R006] split-off ✓ idea) |
| `message/speech-tail` | Bubble tail that morphs from the composer's edge into the bubble during the fly-up |
| `message/persona` | Named-agent identity: avatar ring, name colour token and a signature loader per agent |
| `message/quote-reply` | Select text in an agent message → "Reply to this" quotes it into the next user message |
| `message/annotated` | Margin notes beside agent paragraphs (agent's own comments / suggestions with Apply) |
| `message/version-diff` | Switching versions highlights what changed between them (word-level diff) |
| `message/version-stack` | Versions fan out as stacked cards behind the message; drag to flip |
| `message/read-receipt` | Mid-run messages show "Queued → Read" receipts with a tiny agent avatar travelling to the read message (Arc chat receipt idea, note 12) |
| `message/audit-stamp` | Footer stamp with run id, model, tokens, cost and time — finance / regulated apps |
| `message/voice` | Voice turn: waveform + transcript; tap a word to seek (family K handoff) |
| `message/handoff` | "Agent changed" header card when a different agent continues the turn (note 04 §2c) |
| `message/draft-envelope` | Agent output framed as an outgoing draft (email, reply) with Approve / Edit (CRM, support) |
| `message/run-header` | Header shows model chip, effort glyph and elapsed; collapses to a dot when scrolled past |
| `message/paste-fold` | Long user pastes fold to a "Pasted 1,240 lines" chip that expands in place (Codex large-paste placeholder, note 03) |
| `message/pinned` | Pin any message to a pinned strip at the top of the transcript; pinned messages glow on return |

### 6.3 `message-actions` — Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `message-actions/hover-bar` | Icon row under the message, quiet until hover/focus; always visible on the last turn (default) | chat, desktop |
| `message-actions/always` | Persistent labelled row (Copy · Retry · 👍 👎) for touch | mobile |
| `message-actions/overflow` | Single `⋯` menu with all actions + shortcuts listed | ide (dense) |
| `message-actions/key-hints` | `c copy · r retry · e edit · ← → versions` hint line, actions by key | terminal |

### 6.4 `message-actions` — Pro (12)
| Name | One-line idea |
|---|---|
| `message-actions/floating-toolbar` | A pill toolbar floats at the message's top-trailing corner on hover |
| `message-actions/radial` | Long-press opens a radial menu around the finger (touch) |
| `message-actions/feedback-sheet` | 👎 opens a sheet with tags, screenshot of the message and free text |
| `message-actions/rating-scale` | 5-step scale instead of thumbs, with per-step labels (evaluation tools) |
| `message-actions/reactions` | Emoji reactions on agent messages for team threads; counts roll |
| `message-actions/copy-morph` | Expressive copy: icon morphs into ✓ with a droplet split-off ([R006]) |
| `message-actions/regenerate-with` | Retry opens an inline model/effort strip instead of a menu |
| `message-actions/version-scrubber` | Drag across the ‹ 2/3 › control to scrub versions live |
| `message-actions/compare-versions` | Side-by-side view of two versions with "Keep this one" |
| `message-actions/read-aloud` | Speaker swaps to equalizer bars while reading (affectUI read aloud, note 11) |
| `message-actions/share-snapshot` | Share a read-only snapshot of the thread up to this message |
| `message-actions/branch-tree` | Popover tree of all branches of the thread; click a node to switch (Pi `/tree`, note 03) |

## 7. Edge cases
- **Long text:** user messages > 24 lines fold to 12 lines with "Show more" (coss ui long-message
  collapse, note 01); agent long output is handled by streaming-markdown (code collapse) — prose is
  never folded by default.
- **Huge pastes / binary-looking text:** > 20 000 chars renders as plain mono with no markdown parse
  (proposed).
- **Overflow:** long URLs / tokens break after 40 chars in prose (text-reveal rule); code never wraps
  unless the code block's wrap toggle is on.
- **RTL:** bubbles mirror (user on the left); grouped-corner radii mirror; ‹ › arrows mirror (‹ =
  previous in reading order); version numbers stay LTR in a bidi isolate; mixed-direction bubbles use
  the first strong character for alignment.
- **Tiny sizes (16–20 px controls):** action icons 16 px with 24 px hit areas in IDE; on < 280 px width
  everything goes into `⋯`.
- **Dense IDE panels:** `message/dense` + `message-actions/overflow`; timestamps tooltip-only.
- **Touch:** 44 px hit areas; long-press = actions sheet; no hover state; feedback chips wrap to two rows.
- **Slow networks:** `Pending` clock after 600 ms; `SendFailed` after the adapter's timeout; feedback
  sends queue locally and retry silently (vote shown immediately).
- **Very fast agent:** Reserved → Complete in one frame: no status row, actions fade in directly.
- **Very slow agent:** footer stays hidden until Done; partial text is copyable by selection at any time.
- **Edit while the agent is mid-approval:** `StopConfirm` copy changes to "This will cancel the pending
  approval and stop the run".
- **Many versions (> 9):** arrows show "‹ 12 / 37 ›" with reserved width for the widest count
  (status-text §2.2 `ReserveMagnitude`).
- **Deleted / redacted message** (app policy): row "Message deleted" with no actions except Undo for 5 s.
- **Feedback on an old version:** stored against that version id; switching versions shows that
  version's vote.

## 8. Accessibility
- Each message: `role="article"`, `aria-labelledby` header ("You, 14:02" / "Agent, 14:02"), and
  `aria-describedby` the status row when not `Complete` ("Interrupted after 42 seconds").
- **Action labels:** "Copy message" → "Copied" (announced politely once); "Edit message"; "Retry
  response"; "Retry with a different model"; "Good response" / "Bad response" (`aria-pressed`);
  "Previous version" / "Next version" and the switcher `aria-label` "Version 2 of 3"; "Branch from here";
  "More actions".
- **Hover-revealed actions are always in the DOM / accessibility tree** and appear on focus-within; they
  are never removed when hidden (opacity only), so keyboard users can reach them.
- **Live region:** copy result, "Feedback sent", "Version 2 of 3 shown", "Response stopped",
  "Response failed" (assertive), "Editing message" when the editor opens.
- **Editor:** labelled "Edit your message"; focus moves into it on open and back to the message's Edit
  button on cancel, to the new bubble on Save.
- **Contrast:** user bubble text ≥ 4.5:1 on its fill in both themes; unread 60 % state dims the fill
  only; muted thumbs ≥ 3:1; danger rows carry words.
- **Reduced motion:** §5 last row.
- **Focus order inside a message:** header (if interactive) → body links / code buttons / citations →
  status row actions (Continue / Retry) → footer actions (copy → retry → ▾ → 👍 → 👎 → ‹ › → ⋯).

## 9. Props sketch
```rust
pub struct Message {
    pub id: MessageId,
    pub role: Role,                          // User | Agent { id } | System(kind)  (messages.md)
    pub content: MessageContent,             // blocks for the markdown renderer + attachments + mentions
    pub status: MessageStatus,               // derived, see below
    pub versions: Option<VersionNav>,        // { index: usize, count: usize }
    pub meta: MessageMeta,                   // sent_at, edited_at, model, effort, run_elapsed, tokens, cost
    pub variant: MessageVariant,             // Bubble | Flat | Card | Terminal | Dense | Pro(..)
    pub actions: Option<MessageActions>,
    pub edit: EditPolicy,                    // { target: InPlace | Composer, while_busy: StopAndSend | Disallow }
    pub fold_user_after_lines: Option<u16>,  // 24
    pub labels: MessageLabels,
    pub motion: MotionLevel,
}

pub enum MessageStatus {
    // user
    Pending { since: Instant }, SendFailed { reason: String }, Sent, Unread, Read, Editing, Resending,
    // agent
    Reserved, Thinking, Streaming, Paused, Complete,
    Interrupted { after: Duration, resumable: bool },
    Incomplete { reason: IncompleteReason, resumable: bool },   // LengthLimit | ContextFull
    Failed { kind: ErrorKind, partial: bool, details: Option<ErrorDetails> },
    Withheld { notice: String },
    Regenerating,
}

impl MessageStatus {
    /// Pure: the active turn's agent message maps AgentState; settled messages keep their stored status.
    /// Submitted→Reserved, Thinking→Thinking, Streaming→Streaming, ToolRunning|AwaitingApproval→Paused,
    /// Done→Complete, Cancelled→Interrupted, Error→Failed, Stalled→(unchanged; status row shows it).
    pub fn derive(agent: &AgentState, record: &MessageRecord, is_active_turn: bool) -> Self { /* … */ }
}

pub struct MessageActions {
    pub variant: ActionsVariant,             // HoverBar | Always | Overflow | KeyHints | Pro(..)
    pub items: Vec<ActionKind>,              // Copy | Edit | Retry | RegenerateWith | Feedback | Versions | Branch | More
    pub copy_format: CopyFormat,             // Markdown (default) | Plain | CodeOnly
    pub feedback: FeedbackCfg,               // { reasons: Vec<ReasonChip>, ask_reason_on_up: false, max_chars: 500, auto_send_after: 10 s }
    pub visibility: ActionsVisibility,       // QuietUntilHover (default) | Always | LastTurnAlways
    pub on_action: Callback<MessageAction>,
}
```

## 10. Related components and references
Related: [messages.md](messages.md) · [streaming-markdown.md](streaming-markdown.md) ·
[reasoning-block.md](reasoning-block.md) · [citations-sources.md](citations-sources.md) ·
[composer-send-control.md](composer-send-control.md) (fly-up, Stop) ·
[composer-queue-stack.md](composer-queue-stack.md) (dispatch, unread) ·
[composer-model-picker.md](composer-model-picker.md) (regenerate-with) · [status-text.md](status-text.md)
(Worked for, Interrupted) · family H (checkpoint restore on edit).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| affectUI Message Actions (quiet until hover, copy → check, spin while regenerating, thumbs pop, reason row, ‹ 2 / 3 › switcher, read aloud) | note 11 | action set and morphs (our timings) | Paid / unclear — study only (component is in their free tier, licence not recorded) |
| ChatGPT action bar, retry with model/effort, edit → version arrows, Branch in new chat; Claude / assistant-ui version arrows | note 04 §2g | behaviour | Products — study only; assistant-ui MIT (port with attribution) |
| assistant-ui messages (branches, edit-as-branch, feedback, stopped run, regenerate-with, guardrail notice) | note 01 | state vocabulary | MIT — may port with attribution |
| Zed agent panel (Copy as Markdown, thumbs + text on down) | note 03 | copy formats, feedback | Mixed licence by crate — study only |
| Conductor read-receipt fade | note 08 | `Unread` state | Proprietary docs — idea only |
| Claude desktop user bubble enter (scale .92 + 6 px), message-fade-up 10 px | note 08 (reverse-engineered) | entry motion idea | Proprietary — reference only |
| Arc UI chat thread (receipts travel, reactions roll) | note 12 | `read-receipt`, `reactions` | Paid / unclear — study only |
| coss ui long-message collapse, delivery states | [R126], note 01 | fold rule, pending/failed | MIT — may port with attribution |
| Codex CLI large-paste placeholder | note 03 | `paste-fold` | Apache-2.0 per note 08 (MIT in licensing.md — re-check) |
| Pi `/tree` session tree | note 03 | `branch-tree` | Licence not recorded — study only |
| kushsolitary split-off ✓ | [R006] | `liquid-glass`, `copy-morph` | Study only |
| Renzo Bianchi send exit | [R079] | Save & send fly | Check source; study only |
