**Status:** draft v0.1 (2026-11-11).

# Spec: diff viewer (`diff-viewer`)

Part of family H ([checkpoints-git.md](checkpoints-git.md) — shared `ChangeSet` model §2.6, diff colour
and motion tokens §5.2 live there). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **4 free + 16 Pro = 20 variants** (hero
component). Opened from [file-edit-chip.md](file-edit-chip.md) ("View diff"), the expanded Edit tool
call ([tool-call.md](tool-call.md)), [checkpoint-timeline.md](checkpoint-timeline.md) ("View changes"),
[rollback-impact.md](rollback-impact.md) file rows, [pr-card.md](pr-card.md) and
[review-card.md](review-card.md). Three-way conflicts hand off to [merge-conflict.md](merge-conflict.md).
Syntax highlighting reuses the code-block highlighter ([streaming-markdown.md](streaming-markdown.md)
§3.3).

## 1. Purpose
The agent changed code. The user wants to **read the change quickly, decide hunk by hunk what stays,
tell the agent what to fix, and move on** — while the agent may still be writing. The diff viewer is the
place where trust in an agent's edits is built or lost, so it must be fast on 10,000-line files, never
jump under the reader, and make "keep", "undo" and "comment" one key each.

What we saw (notes 03 §1F, 05 §3, 08, 10): Cursor reviews agent edits with **Keep / Undo** per change,
per file and for all, a floating review bar with next / previous file and Cmd+Y / Cmd+N shortcuts;
Codex's review pane stages or reverts **hunks or files** with inline comments; Claude desktop lets you
hover a line, press "+" to comment and submit all comments with ⌘Enter; Conductor filters the diff by
commit, has "Viewed" checkboxes, lets you edit files in the diff and shows a loading bar on diff refresh;
Hoplite's diff has a resizable changed-files tree with addition / deletion counts, patches in one
continuous scroll, collapse or mark-as-viewed per file, `[` / `]` to step between files, range comments
sent to the agent together, and an "unavailable patch" notice; Paradev puts green/red markers in the
diff scrollbar; Zed and Windsurf review per hunk inline in the editor, Windsurf's diff bar count
updating as you accept or reject; Amp detects moved blocks so they don't show as delete + add; Warp's `E`
toggles all diffs; Codex CLI separates hunks with a dim `⋮` and caps highlighting at 10,000 lines.
Research note 03 calls **batched line comments sent back to the agent** "the defining 2026 review loop".

Anatomy (`unified` with file list and review bar, desktop):
```
┌ Changes · 4 files +58 −21 ──────────────── [Unified|Split|Stacked]  Wrap  ⋯ ┐
│ ▾ src/                     │ auth.rs  M  +12 −3          ☐ Viewed   ⋯         ▐│ file header (sticky)
│   M auth.rs     +12 −3  ◐  │ @@ fn login (L40–52) ─────── Keep  Undo  💬     ▐│ hunk header + actions
│   M session.rs  +30 −15 ○  │  40   40   let user = find(id)?;                 ▐│ context
│   A rate.rs     +8      ●  │  41      − let ok = check(pw);                    █│ removed (word: check)
│   D legacy.rs   −2      ○  │       41 + let ok = verify(pw, &user.salt)?;      █│ added (word: verify… salt)?)
│   tests/                   │  42   42   if !ok { return Err(Denied) }          ▐│
│                            │ ⋯ Show 24 hidden lines ⋯                          ▐│ collapsed context
│ 2 of 4 viewed              │ @@ fn logout (L80–84) ── Kept ✓                    ▐│ resolved hunk
├────────────────────────────┴───────────────────────────────────────────────┤ ▐│ ← minimap (12 px)
│ Hunk 1 of 6 · ‹ j k › · file ‹ n p ›   3 comments · Send to agent ⌘↩   [Undo all] [Keep all] │ review bar
└──────────────────────────────────────────────────────────────────────────────┘
```

Rules (proposed):
1. **Two review modes, two vocabularies.** `Proposed` (edit awaiting approval, nothing on disk yet):
   hunks are **Accept / Reject**. `Applied` (auto-applied edits): hunks are **Keep / Undo** (Cursor's
   words, note 05 §3). The viewer never mixes them in one file.
2. **Nothing moves under the reader.** Streaming lines append below the write head; settled hunks are
   frozen; re-alignment of earlier hunks happens once, on file completion, with the scroll anchored to
   the line under the pointer / focus.
3. **Every decision is reversible in the viewer.** A kept hunk can be undone, an undone hunk restored,
   until the viewer closes; after that, checkpoints are the safety net.
4. **Stable file order.** Files keep first-touch order (file-edit-chip.md §5); other sorts are explicit.
5. **The viewer never diffs text itself.** It renders a `ChangeSet` (checkpoints-git.md §9) computed in
   `oxyn-core` off the UI thread or supplied by the host.

## 2. Rendering model

### 2.1 Layouts
| Layout | What it is | Best for | Minimum width (proposed) |
|---|---|---|---|
| `unified` | one column; removed lines (`−`) then added lines (`+`) interleaved per hunk, two line-number gutters | default everywhere; chat, IDE panels | 360 px |
| `split` | old on the left, new on the right, paired rows; unpaired rows show a hatched filler on the other side | wide review; refactors | 960 px — below it falls back to `unified` with a one-line note "Split needs more room" |
| `stacked` | per hunk, a labelled **Before** block above an **After** block (whole hunk each), no interleaving | prose, config, mobile, narrow panels, explaining a change | 280 px |

(@pierre/diffs calls its single-column layout "stacked" (note 05 §3); in Oxyn that is `unified`, and
`stacked` means before-block over after-block.)

### 2.2 Line and word styling
- **Rows:** added / removed rows use `color.diff.add.bg` / `color.diff.del.bg`; gutters use
  `color.diff.gutter.*`; the sign column always shows `+` / `−` (never colour alone). Change style is a
  token: `background` (default), `markers` (signs only, no tint), `bars` (3 px stripe — Pro
  `bar-style` adds stripe-only motion and density, Cline's 4 px stripe idea, note 08 §1.7).
- **Word-level highlights** (default on; setting `inline = None | Word | Char`): removed and added lines
  in a hunk are **paired** by alignment; each pair is tokenised (words = runs of letters, digits and `_`;
  each punctuation character is its own token; whitespace runs are tokens) and diffed; changed tokens get
  `color.diff.*.word` backgrounds with 2 px radius. Skip rules (proposed): line > 500 chars, or pair
  similarity < 0.4 (changed tokens / total tokens > 0.6) → whole-line tint only, no word marks (avoids
  confetti). `Char` mode diffs characters inside changed words (for typos and numbers).
- **Whitespace:** "Hide whitespace changes" setting; whitespace-only files collapse to "Only whitespace
  changed · Show". Trailing-whitespace changes show a dim `·` marker.
- **Hunk header:** `@@` row with the enclosing symbol when the host supplies it ("fn login, L40–52"),
  the hunk's `+N −N`, its actions, and its state chip. In `terminal`, hunks are separated by a dim `⋮`
  (Codex CLI, note 08 §1.1).
- **Context:** 3 lines around each hunk (proposed); gaps become a "⋯ Show 24 hidden lines ⋯" row with ↑
  20 / ↓ 20 / all (react-diff-view "source expansion" idea, note 05 §3; Verdent "hide unchanged regions",
  note 08 §1.16).

### 2.3 Syntax highlighting (reused)
- Same highlighter service, budgets and caps as code blocks ([streaming-markdown.md](streaming-markdown.md)
  §3.3): off the UI thread, ≤ 2 ms of UI-thread span application per frame, visible window ± 200 lines
  above 2,000 lines, plain mono above 10,000 lines with a "Highlighting off for large files" note (Codex
  caps diff highlighting at 10,000 lines, note 08 §1.1).
- **Whole-file context:** when the host provides full old and new texts, both are highlighted in full and
  lines are mapped onto rows, so a hunk inside a block comment or string highlights correctly. Without full
  texts, each hunk is highlighted as one block (Codex's approach, note 08 §1.1), with a final pass when
  expanded context arrives.
- **Contrast:** syntax foregrounds are checked against the diff tints; any token colour below 4.5:1 on
  `add.bg` / `del.bg` is nudged in OKLab lightness until it passes (Codex corrects pale theme colours
  automatically, note 08 §8). Word backgrounds sit under syntax colours.
- **Colour arrival is not animated** (streaming-markdown.md rule): a line switches from plain to coloured
  instantly.

### 2.4 File list
| Element | Spec (proposed) |
|---|---|
| Layout | tree (default) or flat list; resizable divider 200–480 px, hide / show button; layout remembered per workspace (Hoplite, note 10); under 720 px the tree becomes a file dropdown in the header |
| Status | letter badge `A` added · `M` modified · `D` deleted · `R` renamed (with similarity %, e.g. `R 92%`) · `B` binary · `T` mode / type change · `!` conflict (→ merge-conflict.md); letters are words in the accessible name |
| Counts | `+N −N` in file-edit-chip format (tabular, `+`/`−` signs, zero side omitted) |
| Review progress | ○ none decided · ◐ some · ● all decided · ✓ viewed (checkbox, Conductor / Hoplite) |
| Extras | comment count 💬2; "agent writing" dot while streaming; "generated" tag for `low_signal` files (sorted last, collapsed) |
| Header | "4 files +58 −21" and "2 of 4 viewed" counter; filters: All · Unviewed · With comments · Undecided |
| Sort | first-touch (default, never reshuffles live), path, size; a new order is offered with a dot instead of reshuffling (Amp idea, note 08 §1.14) |

### 2.5 Large files, binary and odd content (proposed thresholds)
| Case | Behaviour |
|---|---|
| > 2,000 lines in a file | highlight visible window ± 200 lines (§2.3) |
| > 1,500 changed lines or patch > 400 KB | file starts collapsed: "Large diff · 2,340 lines · Load diff"; loading shows a top progress bar (Conductor shows a loading bar on diff refresh, note 08 §1.15) |
| > 10,000 lines | plain mono, no word highlights |
| > 50,000 lines or > 5 MB | not rendered inline: "Too large to show here · Open in editor" (Rune's DiffView virtualises up to 50k lines, note 07 — our cap matches) |
| Line > 1,000 chars | cut at 1,000 with "… 3,412 more characters" expander; word diff skipped |
| Minified file (average line > 300 chars) | collapsed "Minified file · diff hidden · Show" |
| Binary (non-image) | "Binary file changed · 24 KB → 31 KB"; hash short form in tooltip |
| Image (png, jpg, gif, webp, svg) | before / after side by side with pixel size and byte size (Jules renders images in its diff, note 03); SVG toggles Image / Source; Pro `image-compare` adds swipe and onion skin |
| Renamed, no content change | header only: `src/{util → helpers}.rs` "Renamed, no changes" |
| Mode change only | `mode 100644 → 100755` row |
| Line endings changed (CRLF ↔ LF) | banner "Line endings changed (CRLF → LF) on 412 lines"; those lines are not tinted individually |
| Non-UTF-8 text | decoded with the detected encoding and a label; undecodable → treated as binary |
| Patch unavailable | "This file's changes can't be shown · <reason>" (Hoplite's unavailable-patch notice, note 10); actions still work at file level |

### 2.6 Streaming diff (agent still writing)
- The file enters `Streaming`; arriving lines append inside the current hunk with `dur.line.enter`
  (120 ms opacity only, **no height animation** — rows take their full height at once so text below never
  slides). Batches are applied at ≤ 30 Hz.
- A 2 px accent **write head** bar sits at the last written line; the hunk header counts roll
  (`dur.digit.roll`).
- The diff for the tail region re-runs at most every 100 ms (same cadence as tail-line re-highlight);
  hunks above the write head that are complete are **frozen** until the file finishes; a final pass on
  completion fixes alignment once, without animation, scroll-anchored.
- **Follow:** while `follow` is on, the viewer keeps the write head in the lower third (scroll `ease.out`
  200 ms, at most once per 250 ms). Any user scroll, j/k or click detaches; a "Follow agent ↓" pill
  appears and re-attaches on press ([text-reveal.md](text-reveal.md) follow convention; Zed / Devin follow-the-agent
  idea, note 03).
- Actions on a streaming hunk are disabled with the reason "Agent is still writing"; bulk actions apply
  to settled files only and say so ("Keep 3 settled files").

### 2.7 Minimap (overview ruler)
12 px column at the end edge (proposed) spanning the scroll height: 2 px marks for added (add hue),
removed (del hue), comment (accent dot), conflict (warn), current hunk (outlined), write head (accent
line). A translucent viewport box shows the visible region; click jumps (centres the target), drag
scrolls. In `split`, the ruler has two 5 px lanes (old / new). Paradev shows green/red markers in the
diff scrollbar (note 03). Hidden under 480 px width.

## 3. State machine
Viewer:
```
   open (ChangeSet ref)            change set ready
  Closed ──────────────▶ Loading ───────────────────▶ Reviewing ◀─────────────────────────────┐
             (skeleton after 150 ms)      │  ▲    │ any file Streaming ⇄ (sub-state, follow on/off)│
             load fails ──▶ LoadFailed    │  │    │ all hunks decided                              │
             (Retry)                      │  │    ▼                                                │
             empty set ──▶ Empty          │  │  AllDecided ("All changes reviewed · Send / Close") ┘ (any hunk reopened)
             ("No changes")               │  │
                                          │  └── bulk done ◀── BulkApplying{n of m} ◀── Keep all / Undo all / Accept all / Reject all
                                          │                         │ partial failure ──▶ Reviewing + failed hunks marked
                                          │
                                          ├── base changed on disk / new agent turn ──▶ Stale ("Refresh to see 2 new changes")
                                          ├── host offline (cannot write) ──▶ ReadOnly{"Offline · decisions disabled"}
                                          └── comments drafted ──▶ (CommentsPending n) ── send ⌘↩ ──▶ Sending ──▶ Sent | SendFailed
```
Hunk (one per hunk; `Proposed` mode uses Accept / Reject labels for the same states):
```
  Streaming ──settled──▶ Pending ──y / Keep──▶ Applying ──▶ Kept ──u / Undo──▶ Applying ──▶ Undone ──r / Restore──▶ … Kept
                            │  └─u / Undo──▶ Applying ──▶ Undone                                  │
                            │                       Applying ── write fails ──▶ Failed{reason} ──retry──▶ Applying
                            ├── user edited overlapping lines ──▶ Edited (needs re-review; shows user's version)
                            ├── file changed on disk overlapping ──▶ Conflict ──resolve (merge-conflict.md)──▶ Pending
                            └── comment added ──▶ (flag: Commented{n}, orthogonal to the states above)
  Presentation: Rest · Hovered (actions reveal) · Focused (current hunk outline) · Pressed
```
File: `Streaming → Ready → (Viewed flag) → Decided{AllKept | AllUndone | Mixed}`; `Collapsed`;
`Large{not loaded}`; `Binary`; `Unavailable{reason}`; `Conflict`.

## 4. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Loading` | header + file list skeleton after 150 ms; top 2 px progress line | — | Close | Esc |
| `Empty` | "No changes" with the compared range ("3:42 PM → now") | Close | — | Esc |
| `LoadFailed` | danger notice + reason | Retry | Close | R |
| `Reviewing` | full viewer; review bar with "Hunk 3 of 12" | per-hunk decide | comment, bulk | j/k hunks, n/p (or ]/[) files, y keep, u undo, c comment, v viewed, s layout, w wrap, e expand / collapse all, x expand context, o open in editor, ? shortcuts |
| `Reviewing·Streaming` | write head, rolling counts, "Writing session.rs…" in the file header, Follow pill when detached | Follow agent | — | f toggle follow |
| `AllDecided` | review bar "All 12 hunks reviewed" + (if comments) "Send 3 comments" | Send to agent / Close | — | ⌘↩ send, Esc close |
| `BulkApplying` | review bar progress "Keeping 8 of 12"; hunks resolve in cascade | — | — | — |
| `Stale` | notice under header "2 new changes since you opened · Refresh" | Refresh | — | ⌘R / Ctrl+R (viewer focus) |
| `ReadOnly` | decisions disabled with reason (offline, viewing a checkpoint preview, replay) | — | Comment (if host queues) | — |
| `CommentsPending` | review bar "3 comments · Send to agent ⌘↩" | Send | Discard all | ⌘↩ |
| `Sending` → `Sent` / `SendFailed` | button label swap "Sending…" → "Sent ✓" (600 ms) / "Couldn't send · Retry" | — / Retry | — | — |
| Hunk `Pending` | tinted rows; actions Keep · Undo · 💬 on hover/focus (always visible on touch and in IDE density) | Keep (Accept) | Undo (Reject), Comment | y, u, c |
| Hunk `Kept` | tint fades to neutral, sign column keeps `+`/`−`, chip "Kept ✓" | — | Undo | u |
| Hunk `Undone` | removed lines restored as plain context, added lines struck and dimmed (collapsed to header after 1.2 s, `delay.auto_collapse`), chip "Undone · Restore" | — | Restore | r |
| Hunk `Applying` | chip shows a 10 px loader (lease) | — | — | — |
| Hunk `Edited` | info chip "You edited this · Review again" with the user's lines | Keep yours | View agent's | — |
| Hunk `Conflict` | warn chip "Changed on disk · Resolve" | Resolve (merge-conflict.md) | — | Enter |
| Hunk `Failed` | danger chip "Couldn't write · permission denied" | Retry | — | r |
| Hunk `Commented{n}` | 💬n in header; threads inline under the anchored line (expanded until sent) | Reply / Edit | Delete | c, Enter on thread |
| File `Large` | collapsed row "Large diff · 2,340 lines" | Load diff | Open in editor | Enter |
| File `Binary` / `Unavailable` | §2.5 rows | file-level Keep / Undo | Open | Y / U |

## 5. Micro-interactions
- **Hunk focus.** j/k move a 2 px focus outline (`spring.snap`, 160 ms) and scroll the hunk to the upper
  third (`ease.out` 200 ms); the review bar counter rolls "Hunk 3 → 4 of 12".
- **Action reveal.** On hover or focus, hunk actions fade in (120 ms); they reserve their space in the
  header, so nothing shifts. On touch and in IDE density they are always visible.
- **Keep.** Rows' tint fades to neutral (`dur.hunk.resolve`, 240 ms `ease.out`); the action group morphs
  into a "Kept ✓" chip (160 ms `spring.snap`, ✓ stroke 240 ms); the file's progress glyph advances
  (○ → ◐ → ●) and the header counts update (`+12 −3` → "+8 −2 of +12 −3" in partially-kept files,
  file-edit-chip.md format). Focus jumps to the next pending hunk after 120 ms (proposed `auto_advance`,
  on by default; Windsurf's count updates as you accept one by one, note 08 §1.13).
- **Undo.** Added lines get a strike drawn start → end (200 ms) and dim to 50 %; removed lines lose their
  tint and become context; after `delay.auto_collapse` (1200 ms) the hunk collapses to its header
  (`dur.collapse`) with "Undone · Restore". Restore plays it back.
- **Bulk (Keep all / Undo all).** Hunks resolve in document order with `stagger.bulk` (30 ms, cap 8 —
  the rest resolve together); off-screen hunks resolve instantly; the review bar shows progress. Undo all
  on applied changes spanning > 1 file asks an inline confirm in the review bar ("Undo 12 hunks in 4
  files? Undo · Cancel") — no modal.
- **Comment.** Hover a line shows a "+" in the gutter (Claude desktop pattern, note 03); click or `c`
  opens an inline comment composer under the line (`dur.expand`); Shift+click or Shift+↑/↓ extends to a
  range (Hoplite range comments, note 10), the range gets a 2 px accent bar in the gutter. Drafts batch;
  ⌘↩ sends all; on send, threads collapse to a 💬 chip with "Sent" and the agent's replies appear in them.
- **Layout switch.** 160 ms crossfade between layouts (`dur.layout.switch`), the focused hunk (or the line
  under the pointer) keeps its screen position (scroll anchoring).
- **Context expand.** Gap row opens to the requested lines with `dur.context.expand` (180 ms
  `spring.snap`), content above fixed.
- **Streaming.** Lines fade in (120 ms); the write head bar slides to the next line (80 ms `ease.out`);
  hunk counts roll; a file finishing writing gets a single 600 ms `dur.highlight.fade`-style header wash.
- **Viewed.** Ticking Viewed draws the check, collapses the file (`dur.collapse`) and the "2 of 4 viewed"
  counter rolls; the file list row dims to 70 %.
- **Minimap.** Viewport box follows scroll at 60 Hz; hover over a mark shows a tooltip ("Hunk 7 · +4 −1
  · 2 comments") after 200 ms.
- **Failure.** Chip morphs to ✕ with the tools.md error nudge on the glyph only; rows never shake.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Loading skeleton | — | 150 ms | — | — | 30 Hz shimmer |
| Hunk focus outline + scroll | 160 ms + 200 ms | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| Action reveal | 120 ms | 0 (hover) | `ease.out` | — | 60 Hz |
| Keep: tint fade + chip morph + ✓ | 240 + 160 + 240 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Auto-advance to next hunk | as focus | 120 ms after decision | `ease.out` | — | 60 Hz |
| Undo strike + dim | 200 ms | 0 | `ease.out` | — | 60 Hz |
| Undone collapse | 280 ms (`dur.collapse`) | 1200 ms | `ease.in` | — | 60 Hz |
| Bulk cascade | 240 ms each | 0 | `ease.out` | 30 ms, cap 8 | 60 Hz |
| Comment composer open / close | 220 ms / 280 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Layout switch | 160 ms crossfade | 0 | `ease.in-out` | — | 60 Hz |
| Context expand | 180 ms | 0 | `spring.snap` | — | 60 Hz |
| Streaming line enter | 120 ms opacity | 0 | `ease.out` | batched ≤ 30 Hz | 30 Hz |
| Write head move | 80 ms | 0 | `ease.out` | — | 60 Hz |
| Follow scroll | 200 ms, ≤ 1 per 250 ms | 0 | `ease.out` | — | 60 Hz |
| Count roll (hunk / file / review bar) | 280 ms | 0 | `ease.out` | 30 ms digits | 30 Hz |
| File done wash | 600 ms | 0 | `ease.out` | — | 30 Hz |
| Viewed collapse | 280 ms | after ✓ 240 ms | `ease.in` | — | 60 Hz |
| Minimap tooltip | 140 ms | 200 ms | `ease.out` | — | 60 Hz |
| Send label swap / "Sent ✓" hold | 120 ms / 600 ms | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | no scroll animation (jump), no outline spring (outline appears), keep / undo swap tint and chip with an 80 ms crossfade, no strike draw (strike appears), no cascade (bulk resolves at once), layout switch instant with anchoring kept, streaming lines appear without fade and follow jumps at most once per 1 s, no washes (2 px static bar for 2 s), counts swap | — | linear | none | event-driven |

## 7. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `diff-viewer/unified` | One column, interleaved −/+ with two gutters, file list, review bar, minimap (anatomy above) | all GUI |
| `diff-viewer/split` | Side-by-side paired rows with hatched fillers; synced scroll; falls back to unified under 960 px | desktop, ide |
| `diff-viewer/stacked` | Before block above After block per hunk with labels; best for prose, config and phones | chat, mobile, desktop |
| `diff-viewer/terminal` | TUI diff: `└ path (+a -r)` headers, `⋮` between hunks, signs + ANSI colours (Codex CLI layout, note 08 §1.1), `y / u / j / k` keys | terminal |

### Pro (16)
| Name | One-line idea |
|---|---|
| `diff-viewer/review-pane` | Full-screen review workspace: resizable tree, continuous diff, and a comments column listing every draft and agent reply, with an "n of N reviewed" ring |
| `diff-viewer/inline-editor` | Diff zones inside the editor buffer: hunks tinted in place with Keep / Undo floating at the hunk's end (Zed / Windsurf idea, note 08) |
| `diff-viewer/minimap-heat` | The minimap becomes a heat strip of change density with hunk ticks and a draggable lens |
| `diff-viewer/follow-agent` | While streaming, a soft glow sweeps the region being written and the viewer glides with it ("agent is writing here" idea [R034]) |
| `diff-viewer/redline` | Prose / document mode: word-level only, deletions struck in place, insertions underlined, no line gutters |
| `diff-viewer/image-compare` | Images with side-by-side, swipe handle and onion-skin opacity slider |
| `diff-viewer/table-cells` | CSV / table diffs as a grid with changed cells tinted and row add/remove bands (Beautiful UI Diff Table idea [R106]) |
| `diff-viewer/json-tree` | Structured diff of JSON / TOML / YAML as a collapsible tree with changed keys and values |
| `diff-viewer/moved-blocks` | Moved code shown once with "moved from L120" links instead of a delete + add pair (Amp idea, note 08 §1.14) |
| `diff-viewer/semantic-fold` | Unchanged functions fold to their signature using syntax-aware ranges; changed ones stay open |
| `diff-viewer/commit-filter` | Range picker over checkpoints and commits ("since 3:42 PM", "commit a1b2c3d") to filter the diff (Conductor filter-by-commit idea, note 03) |
| `diff-viewer/walkthrough` | Stepper that visits hunks in a host-supplied order with the agent's one-line explanation per hunk (Claude "walk through in diff" idea, note 03) |
| `diff-viewer/bar-style` | Stripe-only change style with tight rows for dense IDE panels; the stripe grows into a tint on focus |
| `diff-viewer/hunk-cards` | Each hunk is a swipeable card (keep right, undo left) for phones, with a progress stack |
| `diff-viewer/focus-dim` | The focused hunk is at full strength, others dim to 55 %, so long reviews read one decision at a time |
| `diff-viewer/origin-lanes` | A gutter lane colours each hunk by origin — which agent, turn or the user — with a legend (multi-agent workspaces) |

## 8. Edge cases
- **Huge change sets** (500+ files): file list virtualised; diff body is one virtualised scroll; files load
  lazily as they approach the viewport (± 2 screens); first paint budget < 100 ms for the visible region
  (proposed).
- **Very fast agents.** A file that streams and settles inside `delay.row_guard` (150 ms) renders settled
  with no streaming state.
- **Very slow agents.** A streaming file with no new lines for `tool_stall` (120 s, tools.md) shows "No
  changes for 2 min" in its header; the write head stops (no looping animation).
- **Edits arrive while reviewing.** New hunks in already-decided files mark the file ◐ again and enter
  with a 1600 ms `dur.highlight.fade` tint; decisions already made are kept unless the new edit overlaps a
  decided hunk, which then becomes `Edited` (agent version) and asks for review again.
- **User edits the file outside the viewer.** Overlapping hunks → `Conflict` (Resolve opens
  merge-conflict.md); non-overlapping hunks re-anchor silently.
- **Proposed-mode timeouts.** If the approval behind a proposed edit is cancelled (run stopped), the diff
  becomes `ReadOnly` with "Edit was cancelled".
- **Checkpoint preview / replay.** Viewer renders `ReadOnly` with the snapshot-banner context; all decision
  keys are inert and announce "Viewing a past state".
- **Long lines with wrap off.** Horizontal scroll per file (split: both panes scroll together); wrap on:
  continuation rows get a `↪` in the gutter and no line number.
- **Tabs vs spaces.** Tabs render at the host's tab width (default 4, proposed); whitespace-only diffs can be
  hidden (§2.2).
- **RTL UI.** Chrome mirrors (file list moves to the end edge, review bar order mirrors); code, gutters,
  line numbers and the split old/new order stay LTR (old left, new right) in a bidi isolate. RTL text inside
  code lines renders per Unicode bidi but word-diff marks follow logical order.
- **Tiny sizes.** Under 360 px only `stacked` is offered; under 240 px the viewer opens as a full-screen
  sheet; file list becomes a dropdown; minimap hidden under 480 px.
- **Dense IDE panels.** 18 px rows, 12 px mono, actions as icons with tooltips, always visible; hunk
  headers 20 px.
- **Touch.** 44 px targets for actions (header grows, rows don't); long-press a line to comment; two-finger
  horizontal scroll inside code; `hunk-cards` for swipe review.
- **Slow networks / remote host.** Skeleton after 150 ms; files stream in as they load; decisions queue
  locally and show `Applying` until the host confirms; offline → `ReadOnly` with queued comments kept as
  drafts.
- **Secrets.** Lines the adapter marks redacted render as `••••` with "Redacted" in the gutter; copy
  excludes them.
- **Copy semantics.** Copying from `unified` / `split` copies the **new** side's text without signs or line
  numbers by default; Alt-copy copies the patch.

## 9. Accessibility
- **Structure.** Viewer is a `region` "Changes, 4 files, 58 lines added, 21 removed". File list is a `tree`
  (or `listbox` when flat). Each file is a `region` labelled by its header; each hunk is a `group`
  "Hunk 2 of 5 in auth.rs, lines 40 to 52, 1 added 1 removed, pending". Lines are rows in a `grid`
  (`split`: two cells per row) so screen readers can move by line and by column.
- **Line names.** "Line 41, added: let ok = verify(pw, &user.salt)?" and for paired lines a summary of
  the word change on request (`i` reads "changed check to verify, added user.salt") — never only colour.
- **Keyboard.** All single-letter keys work only when focus is inside the viewer and not in a text field;
  every key has a menu equivalent; `?` opens the shortcuts sheet. j/k hunks, n/p or ]/[ files, ↑/↓ lines,
  Shift+↑/↓ extend selection for comments, y/u decide, r restore, c comment, ⌘↩ send comments, v viewed,
  s layout, w wrap, e expand/collapse all, x expand context, f follow, o open in editor, Esc closes the
  comment composer then the viewer.
- **Live region (polite):** decisions ("Hunk kept. 3 of 12 remaining."), bulk results ("Kept 12 hunks in 4
  files."), streaming start / finish per file once ("Agent is editing session.rs" / "Finished editing
  session.rs, 30 added, 15 removed"), comments sent. Rolling counts are never announced per tick.
  Conflicts and write failures assertive once.
- **Focus.** Auto-advance moves focus only if focus was on the decided hunk; it never steals focus from a
  comment composer or the file list. After bulk actions focus stays on the button.
- **Contrast.** Code text on tints ≥ 4.5:1 (auto-corrected, §2.3); word highlights ≥ 3:1 against the row
  tint plus an underline in high-contrast mode; `+`/`−` signs always present; minimap marks are decorative
  (their information is in the file list and review bar).
- **Colour-blind.** `daltonized` token preset (checkpoints-git.md §5.2).
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct DiffViewer<'a> {
    pub agent: &'a AgentState,                 // Streaming / ToolRunning(Edit) drives the streaming sub-state
    pub changes: &'a ChangeSet,                // checkpoints-git.md §9 (files → hunks → lines with word spans)
    pub mode: ReviewMode,                      // Proposed (Accept / Reject) | Applied (Keep / Undo) | ReadOnly { reason }
    pub variant: DiffVariant,                  // Unified | Split | Stacked | Terminal | Pro(..)
    pub layout_fallback: u16,                  // 960 px: Split → Unified below this
    pub change_style: ChangeStyle,             // Background (default) | Markers | Bars — token preset
    pub inline: InlineDiff,                    // None | Word (default) | Char
    pub word_rules: WordDiffRules,             // { max_line: 500, min_similarity: 0.4 }
    pub context_lines: u8,                     // 3
    pub whitespace: WhitespaceMode,            // Show | HideChanges
    pub wrap: bool,
    pub highlight: HighlightCfg,               // shared with code-block: tail 100 ms, 2 ms/frame, window 2000, plain 10_000
    pub limits: DiffLimits,                    // { collapse_changed: 1_500, collapse_bytes: 400 KB, max_lines: 50_000,
                                               //   max_bytes: 5 MB, long_line: 1_000, minified_avg: 300 }
    pub file_list: Option<FileListCfg>,        // { tree: true, width: 280, sort: FirstTouch, filters, remember: WorkspaceKey }
    pub minimap: bool,                         // hidden < 480 px regardless
    pub follow: FollowPolicy,                  // { on: true, zone: LowerThird, max_rate: 250 ms }
    pub auto_advance: bool,                    // true
    pub comments: Option<CommentCfg>,          // { batch: true, ranges: true, send_label }
    pub keymap: DiffKeymap,                    // defaults in §9; remappable
    pub labels: DiffLabels,                    // i18n; plural rules; Proposed vs Applied vocabularies
    pub on_decide: Callback<HunkDecision>,     // { hunk, decision: Keep | Undo | Restore | Accept | Reject } → host writes
    pub on_bulk: Callback<BulkDecision>,       // { scope: File(path) | All, decision }
    pub on_comment: Option<Callback<CommentDraft>>,      // { anchor: LineRange, side: Old | New, body }
    pub on_send_comments: Option<Callback<Vec<CommentDraft>>>,
    pub on_resolve_conflict: Option<Callback<FilePath>>,  // → merge-conflict.md
    pub on_open_in_editor: Option<Callback<(FilePath, u32)>>,
    pub motion: MotionLevel,
}

pub enum HunkState {
    Streaming, Pending, Applying { to: Decision }, Kept, Undone,
    Edited { by: Origin }, Conflict, Failed { reason: SharedString },
}

pub struct DiffLine {
    pub kind: LineKind,                        // Context | Added | Removed | Filler (split only)
    pub old_no: Option<u32>, pub new_no: Option<u32>,
    pub text: SharedString,
    pub words: SmallVec<[Range<u32>; 4]>,      // changed token ranges for word highlights
    pub syntax: Option<HighlightSpansRef>,     // from the shared highlighter
    pub redacted: bool,
}
```

## 11. Related components and references
**Related:** [checkpoints-git.md](checkpoints-git.md) (ChangeSet, tokens) ·
[file-edit-chip.md](file-edit-chip.md) (entry point; partial-apply count format) ·
[tool-call.md](tool-call.md) / [tools.md](tools.md) (expanded while awaiting approval, collapsed after) ·
[streaming-markdown.md](streaming-markdown.md) §3.3 (highlighter, caps, follow) ·
[checkpoint-timeline.md](checkpoint-timeline.md) (View changes, compare-pair) ·
[rollback-impact.md](rollback-impact.md) (file-diffs variant) · [merge-conflict.md](merge-conflict.md)
(Conflict hunks) · [review-card.md](review-card.md) (findings that jump into the diff, sent comments) ·
[pr-card.md](pr-card.md) (PR file changes) · [git-chips.md](git-chips.md) (base branch shown in header) ·
[run-replay.md](run-replay.md) (`diff-scrub` variant renders this read-only) · family G approvals
(Proposed mode).

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI `diff_render` (note 08 §1.1) | `⋮` hunk separators, per-hunk highlight, 10,000-line cap, line colours, contrast correction | MIT per licensing.md (note 08 says Apache-2.0) — portable with attribution; re-check |
| Rune `DiffView` (note 07) | unified / split, word-level, annotations, 50k-line virtualisation in Rust | MIT — port with attribution (pre-release; pin a revision) |
| gpui-component editor (note 07) | large-buffer editor base for `inline-editor` | Apache-2.0 — port with attribution |
| Cline DiffEditRow (note 08 §1.7) | 10 % tint + 4 px stripe | Apache-2.0 — port with attribution |
| OpenCode desktop diff with line comments (note 03) | comment UX reference | MIT — port with attribution |
| signoff-ui DiffReview [R107] | per-hunk review, unified or split | MIT — may port with attribution |
| Beautiful UI Code Block (Code / Diff) and Diff Table [R106] [R133] | unified diff block; table-cells idea | MIT — may port with attribution |
| Cursor agent review (note 05 §3) | Keep / Undo per change, file, all; review bar; Cmd+Y / Cmd+N | Proprietary — reference only |
| Codex app, Claude desktop, Conductor (note 03) | hunk / file staging, "+" line comments, batched ⌘Enter, filter by commit, Viewed, loading bar | Proprietary — reference only |
| Hoplite diff (note 10) | resizable tree with counts, continuous scroll, `[` `]` stepping, range comments, unavailable-patch notice, remembered layout | Proprietary — reference only |
| Paradev, Zed, Windsurf, Verdent, Jules, Amp (notes 03, 08) | scrollbar markers, inline keep/reject, count updates, hide unchanged, image diffs, moved blocks | Proprietary (Zed mixed by crate) — reference only |
| Warp (note 08 §1.10) | `E` expand / collapse all; expanded while pending | AGPL — study only |
| Claude Code themes (note 08 §1.3) | separate word-level colours, daltonized swap | Proprietary — reference only |
| @pierre/diffs, react-diff-view (note 05 §3) | change styles (markers / background / bars), char / word highlights, widgets for comments, worker tokenisation, source expansion | Licence not recorded in licensing.md — study only until checked |
| VS Code / JetBrains / GitHub (note 05 §3) | conflict hand-off conventions | Not in licensing.md — study only |
| affectUI Inline Diff, HextaUI Diff Review (notes 11, 16) | exist as paid components; nothing taken | Paid — study only |
| Wensity UI diff renderer [R119] | per-step diff renderer idea | Unclear — study only |
