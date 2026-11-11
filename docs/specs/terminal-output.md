**Status:** draft v0.1 (2026-11-11).

# Spec: terminal output (inline command block, live tail)

Part of family F ([tools.md](tools.md) — event model, collapse and density rules live there). Same
depth as the worked example [composer-send-control.md](composer-send-control.md). Target **3 free +
11 Pro variants**. Terminal *tabs* and the full in-app terminal are family J; this spec owns the
**inline block** a `Shell` tool call renders in the transcript (and that a jobs tray or side panel can
host).

## 1. Purpose
When an agent runs `cargo test`, `npm install` or a migration, the user wants to know **what command
ran, whether it is still going, whether it passed, and — if not — the lines that explain why**. They
also want to **stop** a runaway command, **copy** the command to run it themselves, and **watch** a
long build without the transcript jumping. "All" agents ship a command block (research note 03,
"Terminal / command output block", table-stakes): Codex shows a 5-line preview with an ellipsis line
and a transcript hint, `✓` / `✗ (code)` plus duration, a magenta `$` and a `│` continuation rail, and
treats search exit 1 as not-a-failure; Devin shows a read-only live shell preview in its worklog;
Conductor puts a Run button on bash snippets with inline output; Antigravity added a status icon for
"running a background command" (notes 03, 08). The test-runner post [R037] shows the richest live form:
timer, passed / failed counters, per-file tick rows.

## 2. Anatomy
```
 header (32 px chat / 24 px ide)
 ┌──────────────────────────────────────────────────────────────────────────────────────────┐
 │ [loader|✓|✕] $ cargo test --workspace          ~/api   Running 1m 02s  [⧉] [■ Stop] [⇣] [›] │
 ├──────────────────────────────────────────────────────────────────────────────────────────┤
 │    Compiling api v0.3.1                                                    (mono, 12/13 px)│
 │ test auth::rejects_expired ... ok                                                          │
 │ test auth::refresh ... FAILED                                    ← ANSI colours rendered    │
 │ …                                                                                          │
 │ ⋯ 1,204 earlier lines · Show all                       ↓ 42 new lines (when follow is off)  │
 └──────────────────────────────────────────────────────────────────────────────────────────┘
 settled header:  ✕ $ cargo test --workspace   exit 101 · 1m 14s   [⧉] [⟲ Rerun] [›]
```
- **Command line.** `$` prompt in `color.term.prompt` (accent), command in mono; multi-line commands
  show the first line + "+3 lines"; a cwd chip appears only when cwd ≠ workspace root; an owner glyph
  marks user-run commands ("You ran", Codex's wording, note 08 §1.1).
- **Timer.** "Running 12s", "Running 1m 02s" (elapsed-timer Units; minutes and seconds after the
  leading unit are zero-padded, [elapsed-timer.md](elapsed-timer.md) §4); appears at 1 s; paused ⏸
  while awaiting approval or input.
- **Exit badge** (proposed): exit 0 → `✓` + duration (tooltip "exit 0"); non-zero → `✕ exit 101` in
  `danger`; codes listed in `ok_exit_codes` for the command (e.g. `grep`, `rg` exit 1) → neutral
  "No matches"; signal → `■ Stopped (SIGINT)`; timeout → `⏱ Timed out after 10m`.
- **Actions.** Copy command ⧉ · Stop ■ (becomes "Force kill" after the grace period) · Follow ⇣
  toggle · Expand › · overflow ⋯ (Copy output, Open in terminal, Rerun, Move to background).

### 2.1 Output rendering
| Topic | Rule (proposed) |
|---|---|
| ANSI SGR | bold (weight 600), dim (secondary tone), italic, underline, strikethrough, inverse, hidden (spaces); 16 colours → theme `color.ansi.0…15`; 256-colour cube computed; truecolour passed through |
| Contrast correction | any foreground under 3:1 against the block background is moved in OKLab lightness until it reaches 4.5:1 (Codex corrects pale custom theme colours, note 08 §8) |
| Bold vs bright | bold never brightens colour unless the app sets `bold_is_bright` |
| Carriage return / erase | `\r` overwrites the current line and `ESC[K` erases it, so progress bars update in place; cursor-up (`ESC[nA`) is honoured inside the last 10 lines (multi-line progress) |
| Clear screen | `ESC[2J` folds earlier output under "Screen cleared · 120 earlier lines" |
| Alternate screen / full TUI apps | not rendered inline; the block shows "Interactive program · Open in terminal" (Pro `pty-live` renders it) |
| Hyperlinks | OSC 8 links are clickable; non-http(s)/file schemes ask first; plain paths `src/x.rs:12` become "open at line" links |
| Ignored | OSC 52 clipboard writes, title changes, bells, unknown sequences (stripped, kept in the raw log) |
| Encoding | UTF-8; invalid bytes → U+FFFD; tabs expand to 8 columns |
| stdout vs stderr | interleaved in arrival order; stderr lines get a 2 px neutral inline-start marker (not red: many tools print progress to stderr); the failure tail uses stderr |
| Wrap | soft wrap in chat and mobile; no-wrap + horizontal scroll in IDE (toggle in ⋯) |
| Very long lines | first 4,096 chars, then "… +12,345 chars" |
| Memory | last 2,000 lines kept per block for display; the full log is spooled to app storage and opened with "Show all / Open full log" |

## 3. State machine
```
               shell call created                   approval needed (tools.md §2.2)
  (none) ──────────────────────▶ Queued ─────────────────────────────────▶ AwaitingApproval ──deny──▶ Denied
                                   │ ready / approved                                    │ approve
                                   ▼                                                     │
                                Starting ◀──────────────────────────────────────────────┘
                                   │ process spawned
                                   ▼
                                Running ──first byte──▶ Streaming ◀──▶ AwaitingInput (prompt detected / stdin read)
                                   │                       │
            ┌──────────────────────┼───────────────┬───────┼─────────────────┬──────────────────┐
            ▼                      ▼               ▼       ▼                 ▼                  ▼
     Exited{0}=Succeeded    Exited{n}=Failed    TimedOut  Stopping ──grace 3 s──▶ Killed   Backgrounded (→ jobs tray,
                                                           (interrupt sent)  └─exits─▶ Stopped    still running)
  Lost (sandbox / connection gone) from any live state · Stalled{soft,hard} overlay (no output for tool_stall)
  Presentation: Collapsed (header) · Preview(tail N) · Expanded (scroll, max 320 px) · Panel (host-provided)
                Follow: On (pinned to bottom) ◀── user scrolls up ≥ 24 px / End or "↓ new lines" ──▶ Off
  Rerun (settled) ──▶ new block below, linked "Rerun of #12" (the old block stays as history)
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard (block focused) |
|---|---|---|---|---|
| `Queued` | ○, command, "Queued" | — | Copy command | C copy |
| `AwaitingApproval` | amber, command expanded with cwd and env diff (family G footer below), timer ⏸ | family G (Allow / Deny) | Copy command | Tab into approval |
| `Starting` | loader, "Starting…" (only if > 300 ms) | Stop | Copy | Esc stop |
| `Running` | loader, "Running 12s", empty output area hidden | Stop | Copy, Move to background (after 10 s) | Esc stop, C copy |
| `Streaming` | live tail (Preview: last 5 lines), timer | Stop | Follow toggle, Expand | Esc stop, F follow, Enter expand |
| `AwaitingInput` | "Waiting for input" badge, timer ⏸; agent-run blocks are read-only; user-run blocks show an input line (Pro `pty-live`) | Stop | Open in terminal | Esc stop |
| `Stopping` | Stop label → "Stopping…" for the grace period, then "Force kill" in `danger` | Force kill (after 3 s) | — | Esc (again) force kill |
| `Succeeded` | ✓ + duration; collapses to header after `delay.auto_collapse` if the system opened it | Expand | Copy output, Rerun | Enter, C, R rerun |
| `Failed` | ✕ exit N + duration; **last 5 stderr lines stay visible** | Rerun | Copy output, Expand, Open full log | R, Enter, C |
| `Neutral exit` | "No matches" (ok exit code) | Expand | — | Enter |
| `TimedOut` | ⏱ "Timed out after 10m" | Rerun | Expand | R |
| `Killed` / `Stopped` | ■ "Stopped (SIGINT) after 42s" / "Killed after 45s" | Rerun | Expand | R |
| `Backgrounded` | header only, "Running in background" + jobs tray link; output keeps spooling | Bring back | Stop | Enter bring back |
| `Lost` | ! "Connection to the sandbox was lost", last output kept | Retry connection (adapter) | Copy output | — |
| `Stalled` (overlay) | timer keeps counting, `warn` tone, "No output for 2m" | Stop | keep waiting | Esc |
| `Follow Off` | "↓ 42 new lines" pill at the bottom edge | resume follow | — | End resumes |

## 5. Micro-interactions
- **Enter.** Header row enters with `dur.row.enter`; the output area opens only when the first byte
  arrives (so instant no-output commands are one line).
- **Output lines.** New lines appear **instantly** — no per-line fade or slide; output is data, not
  prose. Incoming chunks are batched and painted at most every 33 ms (30 Hz), so a firehose never costs
  more than one layout per frame. Overwritten lines (`\r`) change in place with no motion.
- **Live tail (Preview).** The preview window is a fixed N-line height (5 by default); lines scroll up
  inside it without changing the block's height. A 12 px fade mask at the top edge hints at more
  above.
- **Timer.** Fades in at 1 s; ticks at 1 Hz; magnitude change widens the slot with `spring.snap`
  (elapsed-timer.md §5).
- **Follow toggle.** Scrolling up ≥ 24 px turns follow off: the ⇣ button un-fills (120 ms) and the
  "↓ 42 new lines" pill rises from the bottom edge (160 ms `ease.out`); the count swaps as lines
  arrive (no roll, ≤ 2 Hz). Clicking the pill or pressing End scrolls to the bottom with a 200 ms
  `ease.in-out` glide (instant if > 2,000 px away) and turns follow back on.
- **Stop / kill.** Press: scale 0.94 for 80 ms; label crossfades to "Stopping…" in the same cell; after
  the 3 s grace the label crossfades to "Force kill" and the button fill turns `danger` over 200 ms.
  On exit the loader freezes and desaturates (loaders.md Cancelled).
- **Exit.** Loader morphs to ✓ / ✕ (`dur.row.morph`); exit badge slides in from the inline-end (8 px,
  160 ms); "Running 1m 02s" rolls to the frozen duration. Failure: glyph nudge 2 × 3 px; the preview
  switches from tail to "stderr tail" with a 120 ms crossfade.
- **Auto-collapse after success.** Preview height closes with `dur.collapse` after 1200 ms, only when
  the system opened it and the pointer is not over it (tools.md §4.1).
- **Copy.** ⧉ → ✓ (scale 0.9 → 1, blur 4 → 0 px) for 1200 ms; copies the command without the `$`.
- **Move to background.** The block's header shrinks into a pill that flies toward the jobs tray
  location (≤ 260 ms, `dur.travel`); the transcript keeps a one-line "Moved to background" row.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Header enter | 200 ms + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| Output area open (first byte) | 200 ms height | 0 | `spring.snap` | — | 60 Hz |
| Output paint batching | ≤ 33 ms per paint | — | none (instant lines) | — | ≤ 30 Hz while streaming |
| "Starting…" label | shown only after 300 ms | 300 ms | 120 ms fade | — | — |
| Timer appear / tick | 120 ms / 1 Hz | at 1.0 s | `ease.out` | — | 1 Hz |
| Follow pill in / out | 160 ms / 120 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Scroll-to-bottom glide | 200 ms (instant > 2,000 px) | 0 | `ease.in-out` | — | 60 Hz |
| Stop press / label swap | 80 ms / 120 ms | 0 | `ease.out` | — | 60 Hz |
| Kill grace → "Force kill" | 200 ms tone | 3000 ms | `ease.out` | — | 60 Hz |
| Exit morph + badge | 160 ms + 160 ms | ≤ 200 ms beat | `spring.snap` / `ease.out` | — | 60 Hz |
| Failure nudge | 300 ms, 2 × 3 px | after morph | linear | — | 60 Hz |
| Tail → stderr tail | 120 ms crossfade | after badge | `ease.out` | — | 60 Hz |
| Auto-collapse | 100 ms fade + 180 ms height | 1200 ms | `ease.in` | — | 60 Hz |
| Move to background fly | ≤ 260 ms | 0 | `ease.in` | — | 60 Hz |
| Stall tone ramp | 2000 ms | `tool_stall` 120 s | linear in OKLab | — | 30 Hz |
| **Reduced motion** | output and timer unchanged (they are information); every other change is an 80 ms opacity crossfade; instant heights and scrolls; no fly, no nudge | — | linear | none | output ≤ 30 Hz, else 1 Hz |

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `terminal-output/block` | Bordered block: header with command, timer, actions; 5-line live tail; expand to 320 px scroll | chat, desktop, ide |
| `terminal-output/tail` | Borderless: command line + dim tail lines hanging off a `│` rail, ellipsis line "… 1,204 lines" (Codex-like rhythm) | chat, ide, mobile |
| `terminal-output/tui` | Terminal surface: `$` line, `│` rail, `✓` / `✗ (exit N)` + duration, 16-colour safe, no borders | terminal |

### Pro (11)
| Name | One-line idea |
|---|---|
| `terminal-output/follow-hud` | Floating mini HUD while the block is off-screen: last line + timer + Stop, click to jump back |
| `terminal-output/split-streams` | Tabs for All / stdout / stderr with live counts per stream |
| `terminal-output/test-ticks` | Parses common test output into a tick per test that fills green / red; failures stay lit, others dim ([R037]) |
| `terminal-output/progress-native` | Detects `\r` percentage bars and draws a native progress bar with the percentage, raw line on hover |
| `terminal-output/rate-sparkline` | Lines-per-second sparkline in the header; flat line hints a stall before the threshold |
| `terminal-output/pty-live` | Full PTY grid (cursor, alternate screen, colours) for interactive programs, with "Take over" for user input |
| `terminal-output/runnable-snippet` | A bash code block in an answer gets a Run button and grows its own output block in place (Conductor idea, note 08) |
| `terminal-output/window-chrome` | Block drawn as a small terminal window with a title bar showing cwd and process name |
| `terminal-output/section-fold` | Folds runs of repetitive lines ("Compiling 212 crates") and known phases into collapsible sections |
| `terminal-output/hold-to-kill` | Stop becomes press-and-hold 400 ms with a ring fill (send-control's hold idea), Force kill on a second hold |
| `terminal-output/dock-to-jobs` | Long commands offer one-tap "Run in background" that docks a progress pill to the jobs tray (part 2) |

## 8. Edge cases
- **Instant commands** (< 150 ms, no output): one settled header line, no output area, no timer.
- **Silent long commands** (builds that print nothing for minutes): timer keeps counting; stall
  overlay text is "No output for 2m" (not "stalled"), and it never stops the command.
- **Firehose output** (100k lines/s): paint batching (≤ 30 Hz), 2,000-line display ring, full log
  spooled; the tail shows the last N lines of the current batch.
- **Progress bars with `\r`**: overwrite in place; a `\r` line never counts toward "N new lines".
- **Binary / garbage output**: U+FFFD substitution; more than 20 % invalid bytes in a chunk → the block
  shows "Binary output hidden · Open full log".
- **Secrets in output**: the `Redactor` runs on output before paint and before copy (tools.md §7).
- **Interactive prompts** (`[y/N]`, password reads): `AwaitingInput`; agent-run blocks never accept
  typed input inline by default; `pty-live` offers "Take over".
- **Command not found / spawn failure**: `Failed` with the spawn error as the error line and exit code
  "—" (no fake 127 unless the shell reports it).
- **Killed but still printing**: lines that arrive during `Stopping` are shown; the block settles when
  the process exits or the force kill completes.
- **Restored threads**: output renders from the stored log, settled, no animation, follow off.
- **RTL**: the block chrome mirrors; terminal content is always LTR (bidi isolate around the whole
  output area); the follow pill sits bottom-left.
- **Tiny sizes**: at 16–20 px rows the header shows glyph + command head + exit badge only; output
  hidden behind expand.
- **Dense IDE**: 24 px header; output 12 px mono, 18 px line height; preview 3 lines (proposed).
- **Touch**: Stop, Copy, Follow are 44 px targets in an overflow bar under the header; horizontal
  scroll on no-wrap output uses a two-finger pan; long-press a line to copy it.
- **Slow networks**: output arrives in bursts; the timer is computed from the child clock, not from
  arrival, so it never jumps backwards; `Lost` after the adapter reports disconnection.

## 9. Accessibility
- **Roles.** Block = `role="group"` labelled "Command: cargo test --workspace"; output area =
  `role="log"` with `aria-live="off"` (output is never streamed to screen readers; the user reads it on
  demand); header buttons have names: "Copy command", "Stop command", "Force kill command", "Follow
  output, on", "Expand output".
- **Announcements** (composer polite region, once each): "Command started: cargo test" (only if it
  runs ≥ 2 s), "Command finished in 1 minute 14 seconds", "Command failed, exit code 101", "Waiting
  for input". Timer never announced per second.
- **Reading output.** "Copy output" gives ANSI-stripped text; Expand moves focus into the log, which
  is scrollable with arrow keys / Page Up / Page Down; Home / End jump.
- **Colour.** ANSI colours are contrast-corrected (§2.1); failure is stated in words and glyph, never
  only red text; exit badge text ≥ 4.5:1.
- **Keyboard.** C copy command, Esc stop (Esc again during Stopping = force kill), F follow, Enter
  expand, R rerun, End resume follow.
- **Focus.** Arriving output never moves focus; the follow pill is focusable and reached after the
  output region.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct TerminalBlock {
    pub call: CallId,                      // Shell tool call (tools.md §9)
    pub command: SharedString,
    pub cwd: Option<PathBuf>,              // shown only when ≠ workspace root
    pub owner: CommandOwner,               // Agent | User
    pub state: TermState,                  // Queued | AwaitingApproval | Starting | Running | Streaming
                                           // | AwaitingInput | Stopping { since } | Exited { code } | TimedOut { after }
                                           // | Killed { signal } | Backgrounded | Lost | Denied
    pub output: TermBufferRef,             // ring buffer (2,000 lines) + spooled full log
    pub clock: ChildClockRef,
}

pub struct TerminalOutput<'a> {
    pub block: &'a TerminalBlock,
    pub agent: &'a AgentState,
    pub variant: TerminalVariant,          // Block | Tail | Tui | Pro(..)
    pub preview_lines: u8,                 // 5 (Normal), 20 (Verbose), 0 (Compact)
    pub expanded_max_px: u16,              // 320
    pub follow: FollowPolicy,              // AutoWhileRunning (default) | Always | Off
    pub ok_exit_codes: OkExitCodes,        // e.g. grep / rg → [0, 1]
    pub kill: KillPolicy,                  // Interrupt then Force after 3 s | HoldToKill(400 ms)
    pub ansi: AnsiPolicy,                  // palette tokens, contrast_min 4.5, bold_is_bright false
    pub wrap: WrapMode,                    // Soft | None
    pub background_after: Option<Duration>,// offer "Move to background" after 10 s
    pub redactor: Option<Redactor>,
    pub labels: TerminalLabels,
    pub on_stop: Option<Callback<CallId>>,
    pub on_force_kill: Option<Callback<CallId>>,
    pub on_rerun: Option<Callback<CallId>>,
    pub on_background: Option<Callback<CallId>>,
    pub motion: MotionLevel,
}
```

## 11. Related components and references
**Related:** [tool-call.md](tool-call.md) (shell rows host this block) · [tools.md](tools.md) ·
[elapsed-timer.md](elapsed-timer.md) (Units format, child clocks) ·
[loader-glyph.md](loader-glyph.md) (TUI spinners) · background jobs tray (part 2) · family J
terminal tabs and panels · family G command approval footer.

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI exec cell (note 08 §1.1, note 03) | 5-line preview + ellipsis + hint, `✓` / `✗ (code)` + duration, `$`, `│` rail, search exit 1 neutral, "You ran", contrast correction | MIT per licensing.md / Apache-2.0 per note 08 — portable with attribution; re-check |
| Codex streaming commit tick (note 08) | batching so the display never lags; we use a simpler ≤ 30 Hz paint batch | same as above |
| Devin worklog shell preview (note 03) | read-only live preview | Proprietary — reference only |
| Conductor runnable snippets (note 08 §1.15) | Run button on bash snippets | Proprietary — reference only |
| Antigravity background command status (note 08 §1.11) | background-command state | Proprietary — reference only |
| Warp blocks (note 03) | command / output block model | AGPL — study only |
| Terminal tab states (note 05 §6, a design proposal there) | running / exited-0 / exited-nonzero / awaiting-input / killed / background | Our own proposal |
| Test runner post [R037] | tick rows, counters, timer | Study only |
| AI Elements Terminal (note 05 §6) | command with streaming output | Licence not recorded in licensing.md — study only until checked |
