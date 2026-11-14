**Status:** v1 — palette, command menu and AI answers implemented (plan #26); notification centre stays a draft.

# Spec: ⌘K command palette with AI answers & notification centre (`command-palette`, `notification-center`)

Part of family J ([workspace-chrome.md](workspace-chrome.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Two components that share the "jump anywhere,
know what happened" job: the **command palette** (fuzzy commands + an **ask-the-agent** row whose answer
streams inline with **action chips**) and the **notification centre** (the durable list of agent events
behind toasts and OS notifications). Target **3 + 3 free, 10 + 8 Pro = 24 variants**. Numbers are Oxyn
**proposals** unless a source is cited.

## 1. Purpose
- **Palette.** ⌘K / Ctrl+K "jump to anything" is table-stakes: Cursor (⌘K across transcripts), pi-gui
  (⌘K), OpenCode (Ctrl+P), Zed (thread switcher Ctrl+Tab), Conductor (⌘K in workspace) (note 03);
  Hoplite "Jump to anything…" where opening a thread puts the cursor into its message box (note 10);
  Codex app has a command-palette theme switcher (note 08). In an agent app half the queries are not
  commands but **questions** ("how do I rerun the failed tests?", "where is the auth middleware?"). The
  user wants **one box**: type, get the command if it exists, otherwise ask the agent and get a short
  answer **without leaving what they were doing** — with a button that does the thing.
- **Notification centre.** Agents finish, fail and wait while the user is elsewhere. Two triggers matter:
  **done** and **needs approval** (note 03 §1H: Zed OS notification + sound when done / waiting, Codex
  CLI `tui.notifications` for turn-complete / approval-requested, Paradev notification hooks, pi-harness
  sound alerts). Codex's pet overlay gives lifetimes per state: Running→"Thinking" 3 min, Waiting→"Needs
  input" 24 h, Review→"Ready" 7 d, Failed→"Blocked" 1 h (note 08 §1.1). Hoplite's iOS notification can
  "Answer" a single pending question without opening the app (note 10). The user wants **a place where
  nothing is lost**: what finished, what failed, what is still waiting on them, in one list they can act
  from.

## 2. Anatomy
```
 Command palette (centred, 640 px wide, top at 20 % of window height; max list height 360 px)
┌──────────────────────────────────────────────────────────────── Esc  ⌘K ┐
│ ⌕ how do I rerun failed tests                                           │  input (scope chip: [Commands ▾])
├──────────────────────────────────────────────────────────────────────────┤
│ COMMANDS                                                                │
│ ▸ ⟲ Rerun last command                       Terminal          ⌘⇧R      │  fuzzy rows: icon · label (matched
│   ▶ Run tests                                 Tasks                      │  chars medium weight) · group · keys
│ ASK                                                                     │
│   ✦ Ask the agent: "how do I rerun failed tests"                  Tab   │  ask row (always last in COMMANDS,
├──────────────────────────────────────────────────────────────────────────┤   first when the query reads as a question)
│ ✦ Answer · Opus · 1.2s                                       ■ Stop      │  answer pane (after Tab / Enter on ask row)
│ Run `cargo test -- --failed` from the workspace root. The last run      │  streaming markdown, ≤ 12 lines then scroll
│ failed in auth::refresh (exit 101).▌                                     │
│ [▶ Run in terminal] [⧉ Copy] [↗ Open auth/refresh.rs] [→ Continue in thread] │  action chips
└──────────────────────────────────────────────────────────────────────────┘
 footer hints: ↑↓ select · ↵ run · Tab ask · ⌘↵ ask in new thread · Esc close

 Notification centre (popover from the 🔔 status-bar item or panel; 380 px wide)
┌ Notifications ── [All | Needs you 2 | Runs] ─────────── ⋯ (Do not disturb, Clear) ┐
│ NEEDS YOU                                                                          │
│ ✋ Researcher wants to run `rm -rf dist`        2m   [Review]                       │  sticky until answered
│ ? Planner asks: Which database?                 5m   [Answer]                       │
│ TODAY                                                                              │
│ ✓ Fix auth refresh — Worked for 4m 12s          12m  [Open] [Diff +42 −7]           │  grouped by thread
│ ✕ Deploy preview failed · exit 1                1h   [Retry] [Log]                  │
│ ↻ Linear MCP added 2 tools                      3h                                  │  system events
└────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Matching and the ask row
- **Matcher** shared with [composer-slash-mentions.md](composer-slash-mentions.md) §"Fuzzy matching":
  every query word matches the label anywhere, a word start in the description / keywords, or as a
  subsequence; label-prefix first; groups follow their best match; recency boost.
- **Sources** (app-registered, each a group): Commands, Threads, Files, Symbols, Settings, Agents, MCP
  servers. Prefix scopes (proposed): `>` commands, `#` threads, `@` files, `?` ask.
- **Ask row rules (proposed):** shown whenever the query is ≥ 2 characters and asking is enabled. It is
  placed **first** when the query "reads as a question" — ends with `?`, starts with a question word
  (how, what, why, where, which, can, should, does, is) in the UI language, or has ≥ 5 words with no
  command match above score 0.6; otherwise it is **last**. It never auto-runs: the user must press Tab,
  Enter on the row, or ⌘/Ctrl+Enter.
- **Answer budget:** the palette asks for a short answer (app prompt; we suggest ≤ 120 words) and shows
  it in ≤ 12 lines before scrolling. Long answers offer "Continue in thread".
- **Action chips** come from the agent's structured output (tool suggestions) or from detected entities:
  a fenced shell command → "Run in terminal" (opens a terminal tab, never runs without the press), a
  path → "Open", a setting → "Open setting", a known command → that command's row as a chip. Max 4 chips,
  overflow in ⋯. Chips that would **act** (run, change setting) go through the app's permission rules;
  the palette never bypasses family G.

### 2.2 Notification rules (proposed)
| Kind | Lifetime in centre | Toast in app | OS notification | Sound |
|---|---|---|---|---|
| Needs you (approval, question) | until answered or the run ends (then "Expired") | yes, sticky | if window unfocused and app opted in | opt-in "attention" cue |
| Run done | 7 d (Codex "Ready" 7 d, note 08) | yes, 5 s, only if the user is not looking at that thread | if unfocused + opted in | opt-in "ready" cue |
| Run failed | until dismissed; auto-archive after 7 d | yes, sticky | if unfocused + opted in | opt-in "error" cue |
| Job finished / failed | as [background-jobs.md](background-jobs.md) retention (24 h finished) | per jobs rules | per jobs rules | — |
| System (MCP, update, offline) | 24 h | only if it blocks work | no | — |
| Progress ("Thinking") | not stored (Codex keeps "Thinking" 3 min as a transient, note 08) | no | no | — |
Rules: one notification per event — **toast, centre and OS never duplicate**: if an OS notification
was shown, the in-app toast is suppressed and the centre entry is marked delivered. Repeated events from
the same thread **coalesce** ("3 updates from Fix auth refresh"). Do Not Disturb holds toasts and OS
notifications but still fills the centre; "Needs you" may break through DND only if the user allows it.

## 3. State machines
Palette:
```
 Closed ─⌘K / Ctrl+K─▶ Open{query: ""} (recents + suggested) ─type─▶ Filtering{query}
   ▲                       │                                   │ async sources pending > 150 ms ─▶ Filtering + "Searching…"
   │                       │                                   │ no results ─▶ NoMatches (ask row becomes primary)
   │ Esc (empty query)     │ Enter on command ─▶ Executing ─ok─▶ Closed (focus returns) ; ─needs args─▶ ArgStep{cmd} ─▶ Executing
   │ / click outside       │                                 └ fail ─▶ Open + inline error row
   │                       └ Tab / Enter on ask row / ⌘↵ ─▶ Asking
 Asking: Submitted (pulse < 500 ms) ─first token─▶ Streaming ─done─▶ Answered{chips}
           │ Stop / Esc ─▶ Cancelled(partial kept, "Stopped")      │ error ─▶ AnswerError{retry}
           │ no token for stall_soft ─▶ Stalled ("Still thinking…") │ offline ─▶ Offline("Can't reach the agent")
           │ needs a tool / approval ─▶ Escalated ("Continue in thread" primary — palette answers don't run tools inline)
 Answered ─chip─▶ ChipRunning ─▶ Closed | ChipNeedsApproval (family G inline) | ChipFailed
 Answered ─edit query─▶ Filtering (answer collapses to a "Previous answer" row for 30 s)
 Answered ─⌘↵ / "Continue in thread"─▶ Closed + thread opened with Q & A as the first turn
 Disabled ask: AskUnavailable{reason} (no model, offline, policy) — ask row shows reason, not hidden
```
Notification centre:
```
 Entry: New(unread) ─seen─▶ Read ─archive / expire─▶ Archived ─(retention)─▶ removed
        New ─acted (Approve, Open, Retry)─▶ Resolved ("Approved by you") ─▶ Read
        NeedsYou ─run ended / timed out─▶ Expired ("No longer needed")
        Group: Collapsed{n} ⇄ Expanded
 Centre: Closed ⇄ Open{filter} ; Empty ; DND{until}
 Toast:  Hidden ─event─▶ Entering ─▶ Shown{hold} ─timeout / dismiss / acted─▶ Leaving ─▶ Hidden ; Shown ─hover / focus─▶ Paused
         Stack: ≤ 3 visible, ≤ 12 kept (Arc toast stack numbers, note 12 — study only)
```

## 4. States tables
### Palette
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Open{""}` | recents (5), suggested commands, ask row hint "Ask the agent anything" | run highlighted | — | ↑↓, Enter, Esc closes |
| `Filtering` | grouped rows, matched characters bold, counts per group | run highlighted | ask (Tab) | ↑↓ / ⌃N ⌃P, PgUp/PgDn, Enter |
| `Searching` | rows so far + "Searching files…" shimmer row | — | — | — |
| `NoMatches` | "No commands match" + ask row highlighted | ask | clear | Enter asks |
| `ArgStep{cmd}` | breadcrumb chip `Rerun ›` + argument list | pick | Backspace on empty = back | Enter, Backspace |
| `Executing` | row shows a loader (> 300 ms) | — | — | Esc closes (command continues) |
| `Submitted` | answer pane opens with a pulse | Stop | — | Esc stop |
| `Streaming` | streaming markdown + caret, elapsed after 1 s, Stop | Stop | — | Esc stop, ⌘↵ continue in thread |
| `Stalled` | "Still thinking… 12s" in warning tone after `stall_soft` | Stop | keep waiting | Esc |
| `Answered{chips}` | answer + chips + "Answered in 1.8s" | first chip | Copy answer, Continue in thread, 👍/👎 | Tab cycles chips, Enter runs, ⌘C copies |
| `Escalated` | "This needs tools — continue in a thread" | **Continue in thread** | Copy question | Enter |
| `AnswerError` / `Offline` | reason + Retry | retry | continue in thread | Enter |
| `AskUnavailable{reason}` | ask row dimmed with reason ("No model selected") | fix (opens picker) | — | — |
| `ChipNeedsApproval` | inline approval footer (family G) inside the pane | Allow / Deny | — | per family G |

### Notification centre
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `New` | bold title, unread dot `●` (Codex unread marker idea, note 08) | entry action | mark read | Enter, M mark read |
| `NeedsYou` | ✋ / ? amber, sticky at top | **Review** / **Answer** (inline single-question answer field, Hoplite idea) | open thread | Enter, A answer |
| `Resolved` | "Approved by you · 1m" muted | open | — | — |
| `Expired` | "No longer needed" muted, strike-free | dismiss | — | Delete |
| `Group Collapsed{n}` | "3 updates from Fix auth refresh" | expand | open thread | → expand |
| `Empty` | "You're all caught up" | — | notification settings | — |
| `DND{until}` | header chip "Do not disturb until 14:00" | end DND | — | — |
| Toast `Shown` | icon, one-line title, one action, close | action | close | F6 reaches toasts; Esc closes the focused toast |

## 5. Micro-interactions
- **Open.** Scrim fades; palette rises 8 px + scale 0.98 → 1 with `spring.snap`; input is focused on the
  first frame (Hoplite "the view is there on press", note 10). Opens on **key down**.
- **Typing.** List frame springs to the new height; rows glide when the shift is ≤ 8 rows and snap on long
  jumps; one gliding highlight follows ↑↓ on `spring.snap` (behaviour as studied in Arc's palette, note
  12, re-implemented).
- **Ask row promotion.** When the query becomes a question the ask row moves to the top with a 160 ms
  glide and its ✦ glyph does one soft shimmer ([text-shimmer.md](text-shimmer.md) `calm`).
- **Asking.** The ask row's text flies into the answer pane header (`dur.travel` ≤ 260 ms); the pane opens
  with height `spring.snap`; tokens render through [streaming-markdown.md](streaming-markdown.md) with a
  trailing caret; the palette grows to at most 12 answer lines, then the answer scrolls inside (follow on,
  same rule as terminal-output follow).
- **Chips.** Arrive after the answer completes (never while streaming, so they don't jump): 4 px rise +
  fade, 40 ms stagger, cap 4. Running a chip morphs it into a loader pill, then ✓ and the palette closes
  after 600 ms.
- **Stop.** Caret freezes, answer keeps what arrived, label "Stopped"; no shake.
- **Close.** 120 ms fade + 4 px drop; focus returns to the element that had it.
- **Toast in.** Rises from its own height at the screen edge, older toasts tuck behind as a fanned stack
  (peek 12 px, scale step 0.05) — idea from Arc's toast stack (note 12), our numbers.
- **Centre entry arrival** (centre open): new row slides in at the top of its group with a 600 ms accent
  tint; the 🔔 count rolls.
- **Resolve.** Action button morphs to "Approved ✓" (crossfade 160 ms), row dims after 1.2 s.
- **Swipe** (touch): right to dismiss with velocity throw; left rubber-bands.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Palette open (scrim / panel) | 120 ms / ~180 ms | 0 (on key down) | `ease.out` / `spring.snap` | — | 60 Hz |
| Palette close | 120 ms | 0 | `ease.in` | — | 60 Hz |
| Results reflow | ≤ 180 ms glide; snap > 8 rows | 0 | `spring.snap` | — | 60 Hz |
| Highlight glide | ~120 ms | 0 | `spring.snap` | — | 60 Hz |
| Async "Searching…" row | 120 ms fade | 150 ms | `ease.out` | — | 30 Hz shimmer |
| Ask row promotion | 160 ms | 0 | `ease.in-out` | — | 60 Hz |
| Question fly to pane | ≤ 260 ms | 0 | `ease.in` | — | 60 Hz |
| Answer pane open | ~200 ms | 0 | `spring.snap` | — | 60 Hz |
| Streaming paint | per streaming-markdown | — | — | — | ≤ 30 Hz |
| Stall soft | ramp 2 s | `stall_soft` (default 8 s, proposed) | OKLab linear | — | 30 Hz |
| Chips in | 160 ms, 4 px rise | after done | `ease.out` | 40 ms, cap 4 | 60 Hz |
| Chip run → close | loader, ✓ 160 ms, close after 600 ms | — | `spring.snap` | — | 60 Hz |
| Toast in / out | 200 ms / 160 ms; hold 5 s (done), sticky (needs you, failed) | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Toast stack fan | ~220 ms | 0 | `spring.soft` | — | 60 Hz |
| Centre row arrival tint | 200 ms slide + 600 ms tint | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| Resolve morph / dim | 160 ms / after 1.2 s | — | `ease.out` | — | 60 Hz |
| **Reduced motion** | palette and toasts appear / disappear with an 80 ms opacity fade; no scale, rise, glide, fly, fan or shimmer; highlight jumps; answer still streams (information) but without caret blink; chips appear at once | — | linear | none | streaming ≤ 30 Hz |

## 7. Variants
### Command palette — free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `command-palette/centered` | Baseline modal (anatomy above): fuzzy groups, ask row, inline streaming answer, action chips | desktop, ide, chat |
| `command-palette/sheet` | Full-width top sheet on mobile / narrow windows; answer pane below input | mobile, desktop |
| `command-palette/tui` | Ctrl+P overlay in a box-drawn frame; `?` prefix asks; answer streams in the same box | terminal |
### Command palette — Pro (10)
| Name | One-line idea |
|---|---|
| `command-palette/split-preview` | Right pane previews the highlighted thread / file / command before you run it |
| `command-palette/answer-cards` | Answers render as cards with sources ([citations-sources.md](citations-sources.md)) and a confidence badge |
| `command-palette/agent-scope` | Scope chips (This thread / Workspace / Web) decide what the agent may read before answering |
| `command-palette/morph-from-composer` | ⌘K morphs the composer into the palette and back (same shell, different job) |
| `command-palette/orb-ask` | Ask row shows a mini [thinking-orb.md](thinking-orb.md) that becomes the answer's header loader |
| `command-palette/voice-ask` | Hold Space on the ask row to dictate the question ([composer-voice-input.md](composer-voice-input.md)) |
| `command-palette/recipes` | Multi-step command recipes ("Review → fix → test") shown as a step row with progress |
| `command-palette/history` | Previous palette questions and answers searchable as a group |
| `command-palette/theme-switcher` | Theme rows preview live on highlight (Codex palette theme switcher idea, note 08) |
| `command-palette/glass` | Frosted translucent panel with a slow spectral rim while the agent answers |

### Notification centre — free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `notification-center/popover` | Baseline popover from the bell: Needs-you section, day groups, filters | desktop, ide, chat |
| `notification-center/toasts` | Toast stack only (≤ 3 shown, ≤ 12 kept), each with one action | desktop, chat, mobile |
| `notification-center/tui` | `[!] 2 need you · [✓] 3 done` line; `/notifications` list with numbered actions | terminal |
### Notification centre — Pro (8)
| Name | One-line idea |
|---|---|
| `notification-center/inbox` | Full-panel inbox with keyboard triage (J/K, A approve, D dismiss), risk-sorted needs-you first |
| `notification-center/digest` | "While you were away" summary at the top when the user returns after ≥ 30 min |
| `notification-center/inline-answer` | Answer a single pending question in the toast or row without opening the thread (Hoplite iOS idea, note 10) |
| `notification-center/sound-pack` | Opt-in cues for ready / needs-you / error ([R110] Cuelume idea, sounds made in-house) |
| `notification-center/thread-stacks` | Toasts from one thread stack into one card with a count and the latest line |
| `notification-center/timeline` | Events on a day timeline with run bars; click to open run replay |
| `notification-center/focus-schedules` | Quiet hours and per-thread mute with a visible "held 4" counter |
| `notification-center/island-relay` | When the app is unfocused, needs-you events go to the desktop island first ([desktop-overlays.md](desktop-overlays.md)) |

## 8. Edge cases
- **Command vs question ambiguity** ("test"): command wins; the ask row stays last; pressing Tab always
  asks, so the user is never trapped.
- **Destructive commands**: rows marked destructive ask for confirmation in the row ("Press ↵ again to
  delete thread"), 3 s window.
- **Very long answers / code**: 12 lines, then inner scroll; code blocks have their own copy; "Continue in
  thread" carries the full answer.
- **Answer refers to stale state**: chips that target files or commands that no longer exist are disabled
  with "No longer available".
- **Closing while streaming**: the answer keeps streaming into a "Palette answer" notification entry only
  if the app enables it; otherwise Esc stops it. Default: Esc once stops, Esc twice closes.
- **Secrets in query** (pasted token): the app's redactor runs before sending; the ask row warns "Looks
  like a secret — remove before asking".
- **Large catalogues** (10k files): async sources stream results in; the list never reorders items the
  pointer is over (pointer-stable ranking for 300 ms).
- **IME composition**: no filtering until composition ends; Enter during composition never runs a command.
- **Notification floods** (50 events / min): coalesce per thread; toast rate limit 1 per 2 s; the rest go
  straight to the centre with a count.
- **Stale needs-you** (run ended): entry becomes `Expired`; acting on it is rejected with "This request
  has ended" (Hoplite clears stale approvals, note 10).
- **Multiple windows**: one centre per app; toasts show only in the focused window.
- **RTL**: palette mirrors; shortcut keycaps keep LTR order; toast stack anchors bottom-left.
- **Tiny sizes**: palette min width 320 px; group headers hide under 400 px; bell at 16 px shows a dot,
  not a number.
- **Touch**: rows 48 px; chips 44 px; swipe on toasts.
- **Slow networks**: answer Submitted pulse ≤ 500 ms then shimmer "Asking…"; Offline state offers to
  queue the question as a thread message.
- **Very fast agents**: answers that complete in < 300 ms render at once without the streaming caret.

## 9. Accessibility
- **Palette:** `role="dialog"` `aria-modal="true"` labelled "Command palette"; input `role="combobox"`
  with `aria-expanded`, `aria-controls` the `listbox`, `aria-activedescendant` the highlighted option;
  groups are `group` with labels; matched characters are not announced separately. Result count announced
  politely ("8 results") debounced 500 ms.
- **Ask row:** option named "Ask the agent: how do I rerun failed tests". The answer pane is a `region`
  "Agent answer" with `aria-busy` while streaming; the answer is **not** streamed into a live region —
  one polite announcement "Answer ready" (or "Answer stopped") with the first sentence; the user reads the
  rest with arrow keys after Tab moves focus into the pane.
- **Chips:** `button`s in a `toolbar` "Answer actions"; acting chips name their effect ("Run cargo test in
  a new terminal").
- **Notification centre:** popover `dialog`; list of `article`s with headings; unread state in the name
  ("Unread. Researcher wants to run rm -rf dist. 2 minutes ago."). Toasts: polite live region with a
  hidden type prefix ("Needs you:", "Error:"); assertive only for needs-you; toasts never steal focus;
  F6 reaches the toast region ([workspace-chrome.md](workspace-chrome.md) §8); when a focused toast closes,
  focus moves to the next toast or back to where it came from.
- **Contrast:** text ≥ 4.5:1, unread dot ≥ 3:1 plus bold title; states carry glyph + word.
- **Reduced motion:** §6 last row.

## 10. Props sketch
```rust
pub struct CommandPalette<'a> {
    pub sources: &'a [PaletteSource],        // Commands | Threads | Files | Symbols | Settings | Agents | Mcp | Custom(id)
    pub matcher: MatcherConfig,              // shared with composer-slash-mentions
    pub ask: Option<AskConfig>,              // None hides the ask row
    pub answer: Option<&'a PaletteAnswer>,   // driven by the app's agent call
    pub variant: PaletteVariant,             // Centered | Sheet | Tui | Pro(..)
    pub max_list_px: f32,                    // 360
    pub answer_max_lines: u8,                // 12
    pub labels: PaletteLabels,
    pub on_run: Callback<(CommandId, Vec<Arg>)>,
    pub on_ask: Callback<AskRequest>,        // { question, scope, new_thread: bool }
    pub on_stop: Callback<()>,
    pub on_chip: Callback<ChipAction>,       // RunInTerminal(cmd) | Open(path) | OpenSetting(id) | Command(id) | ContinueInThread
    pub on_close: Callback<()>,
    pub motion: MotionLevel,
}
pub struct AskConfig { pub question_heuristic: bool, pub min_chars: u8 /* 2 */, pub stall_soft: Duration /* 8 s */,
                       pub unavailable: Option<SharedString> }
pub struct PaletteAnswer { pub state: AgentState /* Submitted | Streaming | Done | Error | Cancelled | Stalled */,
                           pub markdown: StreamRef, pub chips: Vec<ChipAction>, pub model: Option<SharedString>, pub elapsed: Duration }

pub struct NotificationCenter<'a> {
    pub entries: &'a [Notice],               // { id, kind: NeedsYou|Done|Failed|Job|System, thread, title, at, state, actions }
    pub variant: NotificationVariant,        // Popover | Toasts | Tui | Pro(..)
    pub policy: NotifyPolicy,                // lifetimes per kind, toast holds, os: OptIn, sound: None, dnd
    pub max_toasts_shown: u8,                // 3
    pub max_toasts_kept: u8,                 // 12
    pub toast_rate: Duration,                // 2 s
    pub on_action: Callback<(NoticeId, ActionId)>,
    pub on_answer: Option<Callback<(NoticeId, SharedString)>>,
    pub on_read: Callback<Vec<NoticeId>>,
    pub motion: MotionLevel,
}
```

## 11. Related components and references
**Related:** [composer-slash-mentions.md](composer-slash-mentions.md) (matcher, row) ·
[composer-model-picker.md](composer-model-picker.md) (`model-picker/command` shares the sheet) ·
[streaming-markdown.md](streaming-markdown.md) · [text-shimmer.md](text-shimmer.md) ·
[approval-card.md](approval-card.md) (chip approvals, needs-you) · [background-jobs.md](background-jobs.md)
(job notifications, retention) · [side-panels.md](side-panels.md) (bell item) ·
[desktop-overlays.md](desktop-overlays.md) (hotkey launcher reuses this palette outside the window) ·
[workspace-chrome.md](workspace-chrome.md) (F6 to toasts).

| Reference | What we take | Licence note |
|---|---|---|
| Cursor, pi-gui, OpenCode, Zed, Conductor palettes (note 03) | ⌘K / Ctrl+P conventions, thread switching | Cursor, Conductor proprietary; OpenCode MIT; Zed study only; pi-gui not checked |
| Hoplite (note 10) | "Jump to anything…", open on press, cursor into message box; iOS "Answer"; stale approvals cleared | Proprietary — reference only |
| Codex (note 08) | unread ● / read ○ markers, state lifetimes 3 min / 24 h / 7 d / 1 h, palette theme switcher | MIT / Apache-2.0 for CLI source — portable with attribution; app is reference only |
| Zed, Codex CLI, Paradev, pi-harness notifications (note 03) | done + needs-approval triggers, sound | Zed study only; Codex portable; others reference only |
| Arc UI command palette, toast stack (note 12) | gliding highlight, glide-vs-snap, 3 shown / 12 kept, fan stack | Paid / unclear — study only |
| affectUI Command Menu (Pro), Notification Stack (Free) (note 11) | catalogue presence | Study only (licensing.md) |
| Cuelume [R110] | sound cue categories | MIT — portable with attribution (sounds re-made) |
| Astryx Command Palette, Power Search (note 16 [R103]) | presence of primitives | Licence not recorded — study only until checked |

## Plan #26 implementation
Implemented in `oxyn-core::primitives::command` (`Command`, `Palette`, `PaletteKey`, `PaletteOutcome`,
`search`, `score`, `highlight`) with `oxyn-dioxus::CommandPalette` and `oxyn-gpui::CommandPaletteEl`.
The AI answer half is split into [ai-command.md](ai-command.md), [ai-search-bar.md](ai-search-bar.md)
and [settings-command.md](settings-command.md); toasts and notices into [toaster.md](toaster.md)
and [notification-stack.md](notification-stack.md).

- **Arc UI Command palette (block):** every word must match (label anywhere, a word start in the
  description or keywords, or letters in order); matched letters in medium weight; groups follow
  their best match; ↑ ↓ wrap, Home / End, Tab / Shift+Tab jump sections, Enter runs, Escape clears
  then closes; one gliding highlight (spring for the pointer, a jump for the keyboard).
- **affectUI Command Menu:** `variant: "menu"` — tool logos, ⌘1–⌘5 on the first five suggestions,
  an "esc" cap, a tray confirming what ran ("✓ Ran Summarize this thread") and an "Ask the
  assistant" row when nothing matches.
- **Oxyn plus:** scoped modes by prefix (`>` commands, `@` files, `#` threads) with a scope chip and
  Backspace to leave it, recents, `elevated`, `max_height`, announced result counts.
