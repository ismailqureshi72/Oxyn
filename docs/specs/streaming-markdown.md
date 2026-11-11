**Status:** draft v0.1 (2026-11-11).

# Spec: streaming markdown & code blocks (`markdown`, `code-block`)

Part of family E ([messages.md](messages.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **`markdown` 3 free + 8 Pro** and
**`code-block` 4 free + 12 Pro** = **27 variants**.

**Division of labour.** [text-reveal.md](text-reveal.md) already owns *how text appears*: the pacer, the
reveal effects, the caret, scroll follow, and the **incomplete-syntax table** (text-reveal §5: bold /
emphasis repair, lone list markers, headings, links never showing their URL while open, image
placeholders, raw HTML withheld, code fences by line-commit, tail-line highlight every 100 ms, table
header hold-back, math as raw source until closed, citation chips). **This spec does not repeat those
rules.** It owns *what the text becomes*: the block model and renderer, typography, the constructs
text-reveal §5 does not cover, the **code block** (header, language, copy, wrap, run, apply, collapse),
**syntax-highlight timing and budgets**, tables and math as rendered objects, and **long-output
collapse**.

## 1. Purpose
Agent answers are documents: headings, lists, tables, math, and above all **code the user will copy,
run or apply**. While streaming, the renderer must never flash raw syntax, never reflow what the user
is reading, and never hand the user a half-written code block as if it were done (copy disabled until
the fence closes; Run and Apply only on complete blocks). Every product renders markdown with copy
buttons (note 04: "Markdown/code/table rendering w/ copy — All"); the differentiators are the code
block's actions — Conductor's **Run button on bash snippets** with inline output, Claude desktop's
"Fix this one / Apply fixes", ChatGPT code blocks with Preview (notes 03, 04, 08) — and the streaming
discipline Codex and MonoCode document in source (mutable tail, per-block repair; note 08). Arc's code
block explicitly says to use a separate text-stream component for code still arriving (note 12): we
make that one component with two phases instead.

## 2. Block model and state machine

### 2.1 Blocks
The adapter's text stream is parsed (incremental CommonMark + GFM, tail only — text-reveal §5) into
blocks with stable ids: `Paragraph`, `Heading(1..=6)`, `List { ordered, start, task }`, `Blockquote`,
`CodeFence { lang, meta }`, `Table`, `Math { display }`, `Rule`, `Image`, `Html` (sanitised, off by
default), `FootnoteDef`, `Details` (collapsible), `Diagram { kind }` (fenced `mermaid` and similar,
rendered by a pluggable renderer). Inline: emphasis, strong, strike, code, link, autolink, image,
citation marker ([citations-sources.md](citations-sources.md)), footnote ref, inline math.

### 2.2 Block lifecycle
```
               bytes arrive                    block boundary (blank line, fence close, next block start)
  (none) ──────────────────▶ Tail(open) ───────────────────────────────────────────────▶ Settled
                              │  repaired each frame (text-reveal §5)                      │ frozen: parsed once,
                              │  only this block re-renders                                │ height cached,
                              │                                                            │ virtualisable
                              ├── tool / approval interrupts ─▶ ClosedVisually ─▶ Settled (new tail after the tool row)
                              ├── Stop ──────────────────────▶ Truncated (closed visually, marked incomplete)
                              └── Error ─────────────────────▶ Truncated (same) + message Failed row
```

### 2.3 Code block lifecycle (the part with actions)
```
   fence ``` seen         info string ends (newline)        line commits            closing fence
  ───────────────▶ Opening ───────────────────────▶ Streaming(lines n) ─────────────────────▶ Complete
   chrome appears   lang "…" (label pending)          copy disabled, Run/Apply hidden            │ final highlight pass
                                                       height grows, window after 40 lines       ▼
                                                                                   Ready (copy · wrap · run · apply)
                                                                                    │      │          │
                                                       Expanded ◀── Show all ── Collapsed  Running ─▶ Ran(exit, 1.2s) / RunFailed(exit)
                                                                                    │                      └─ output panel
                                                                                  Applying ─▶ Applied(+12 −3) / Conflict / ApplyFailed / Rejected
   Stream ends without closing fence (Stop/Error) ─▶ Truncated: header chip "Incomplete"; copy enabled (copies what exists); Run/Apply disabled with reason
```

### 2.4 States table (code block)
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Opening` | header: language label "…" (dim), copy icon disabled; empty body 1 line tall | — | — | — |
| `Streaming` | lines commit (line-commit, text-reveal §4); header right shows "Writing · 42 lines"; copy disabled with tooltip "Available when the block is complete" | Stop (composer) | select settled lines manually | Esc stops run |
| `Complete` → `Ready` | full block, final highlight; actions fade in 120 ms | Copy | Wrap, Run, Apply, Open in editor, Download (hosts choose) | Tab: copy → wrap → run → apply → body (focusable `pre`) |
| `Collapsed` | first 20 lines + bottom fade mask + "Show all 312 lines ▾" | Show all | copy (copies all) | Enter on toggle |
| `Expanded` | all lines; sticky header while the block is in view; "Collapse ▴" at the bottom | Collapse | — | Enter |
| `Running` | Run ▶ becomes ■ with elapsed; output panel opens under the block, streaming (family F terminal output) | Stop | — | Esc stops the snippet (focus inside block) |
| `Ran(exit 0)` | output panel with ✓ "exit 0 · 1.2 s"; Run ↻ | Rerun | copy output, collapse output | — |
| `RunFailed(exit n)` | ✕ "exit 1 · 0.4 s" in danger tone; output panel open | Rerun | "Ask agent to fix" (host) | — |
| `Applying` | Apply → spinner (loader 12 px) "Applying to auth.rs" | — | cancel | — |
| `Applied` | ✓ "Applied to auth.rs +12 −3" (diff stats in Zed DiffStat typography: thin space + figure dash, note 08); "Undo" for 10 s | View diff | Undo | — |
| `Conflict` | amber "auth.rs changed since this was written · Review" | Review (opens family H diff / merge) | Apply anyway (confirm) | — |
| `ApplyFailed` | danger "Couldn't apply · Details" | Retry | Copy instead | — |
| `Truncated` | "Incomplete" chip in header; last line closed visually | Copy | — (Run/Apply disabled: "Block is incomplete") | — |

Run and Apply are **host capabilities**: the block shows them only when the host passes a handler and
the language is allowed (Run defaults to shell-like languages only — proposed: `sh`, `bash`, `zsh`,
`fish`, `pwsh`, `console`). Run respects the composer's permission level
([composer-permission-selector.md](composer-permission-selector.md)): when the level requires approval,
Run opens a family-G approval card first.

## 3. Rendering rules

### 3.1 Typography (proposed tokens)
| Element | Chat / desktop | IDE / dense | Terminal |
|---|---|---|---|
| Body | 15 / 24 px, `font.ui` | 13 / 20 px | 1 cell |
| H1 / H2 / H3 / H4–6 | 20 / 17 / 15 semibold / 15 medium (answers should not shout) | 15 / 14 / 13 semibold / 13 | bold / bold / bold / normal, `#` hidden |
| Paragraph gap | 12 px | 8 px | 1 line |
| List indent | 20 px per level; bullets • ◦ ▪ by depth | 16 px | 2 cells; `•` `◦` `▪` |
| Inline code | `font.mono` 0.92 em, 2 × 4 px padding, `color.code.inline.bg` | same, 0.9 em | plain, colour only |
| Code block | 13 / 20 px mono; 12 px padding; radius 10 px | 12 / 18 px; 8 px padding; radius 6 | no frame (`code-block/terminal` adds `│`) |
| Blockquote | 3 px leading rule, 12 px indent, fg 85 % (≥ 4.5:1) | same | `▎` |
| Table | 14 px, row 32 px, header semibold, zebra off by default | 12 px, row 24 px | box-drawing |
| Max line length (prose) | 72 ch inside the 760 px column | panel width | terminal width |

### 3.2 Constructs not covered by text-reveal §5 (while open → when closed)
| Construct | While open (tail) | When closed |
|---|---|---|
| Nested list item (indent change) | held one line until the indent is known, then placed (no re-indent jump) | — |
| Task list `- [ ]` / `- [x]` | shown as a list item; checkbox appears when `]` arrives | read-only checkbox (host may make interactive) |
| Ordered list start `7.` | numbering starts at 7 as soon as the marker is complete | — |
| Footnote ref `[^1]` | plain text until `]` | superscript link to the footnote at the end of the message |
| HTML entity `&amp` | held until `;` or 8 chars | decoded |
| Autolink `https://…` | shown as text; becomes a link when a space / newline / end arrives | link |
| Table alignment row `|:--|--:|` | header held (text-reveal); alignment applied on arrival, no animation | — |
| `Details` (`<details>` when HTML allowed, or `:::details` if the host enables directives) | summary shown, body collapsed | collapsible |
| Diagram fence (```` ```mermaid ````) | code lines stream as code (line-commit) under a "Diagram · writing" header | rendered once on close (crossfade 200 ms, height anchored); source toggle "Code / Diagram"; render error → falls back to code with "Couldn't render diagram" |
| Unterminated fence at stream end | — | `Truncated` (§2.3) |
| Link destinations | never shown while open (text-reveal) | `http(s)`, `mailto` only; other schemes render as text (`javascript:` etc. blocked); external links get a trailing ↗ glyph and open via the host |
| Raw HTML | withheld (text-reveal) | **off by default** (rendered as text); when on, a sanitising allow-list (inline formatting, `details`, `kbd`, `sub`, `sup`, `br`) — proposed |

### 3.3 Syntax highlighting — timing and budgets (proposed)
- **When:** (1) settled lines of a streaming block are highlighted as they commit; (2) the tail line is
  re-highlighted at most every 100 ms (text-reveal §5); (3) one **full pass on close** fixes any
  multi-line constructs (block comments, strings) the incremental pass got wrong.
- **Where:** off the UI thread (GPUI background executor; Dioxus web: a Web Worker). Results are applied
  per line on the next frame; no frame waits for the highlighter.
- **Budget:** ≤ 2 ms of UI-thread work per frame for applying highlight spans across all blocks; the
  background highlighter yields every 5 ms.
- **Caps:** blocks > 2 000 lines highlight only the visible window ± 200 lines; blocks > 10 000 lines are
  plain mono with a "Highlighting off for large blocks" note (Codex caps diff highlighting at 10 000
  lines, note 08).
- **Colour arrival:** a line switches from plain to coloured **without animation** (colour fades read as
  flicker in code) — proposed. Under the terminal surface, colours map to the 16-colour palette or none.
- **Language:** from the fence info string (`rust`, `ts`, aliases table). No info string: label "text";
  after 8 lines the host may run detection once and relabel (label crossfade 120 ms, never twice).
- **Engine:** open — candidates are a tree-sitter-based or TextMate-grammar highlighter in
  `oxyn-core`; decision and licence check in the V2.6 tech spike (grammars carry their own licences).

### 3.4 Long-output collapse (proposed)
| Content | While streaming | After complete |
|---|---|---|
| Code block ≤ 30 lines | grows naturally | full |
| Code block > 40 lines | grows to 40 lines, then becomes a **40-line window that follows the tail** (internal scroll; header "Writing · 312 lines") so the transcript does not grow unboundedly; user scroll inside the window detaches it | collapses to **20 lines** + "Show all 312 lines" |
| Table > 20 rows | rows append; after 20 the table freezes column widths (text-reveal §5) | shows 10 rows + "Show all 84 rows"; horizontal scroll inside the table, never the page |
| Diagram | code window (as above) | rendered; max height 480 px with zoom / open full |
| Prose | never collapsed (reading is the point) | — |
| Whole agent message > 6 viewports (rare) | — | optional host policy: sections fold under headings (`markdown/foldable-sections`, Pro) |
Collapse/expand animates height with `dur.collapse` (200 ms) and anchors per [messages.md](messages.md) §4.2:
expanding a block **above** the viewport keeps the reader's line fixed.

### 3.5 Copy semantics
- Code copy = exact source of the fence body (no line numbers, no prompt `$` when the host marks prompt
  lines as decoration — proposed `console` handling strips `$ ` / `> ` prompts and output lines).
- Table copy (`⋯` on table hover): Markdown, TSV (pastes into spreadsheets), CSV.
- Math copy: LaTeX source.

## 4. Micro-interactions
- **Fence opens:** code chrome (header + 1-line body) fades in 120 ms; the block reserves 1 line, then
  grows per committed line (line-commit 80 ms fade, text-reveal §4). No per-token effects in code.
- **Language label:** "…" → "rust" crossfade 120 ms when the info string completes.
- **Streaming window:** at 40 lines the block stops growing; the header line count rolls
  (`dur.digit.roll`); a 24 px top fade mask indicates hidden earlier lines.
- **Complete:** final highlight pass applies; actions fade in 120 ms (stagger 30 ms); copy icon enables
  (opacity 40 % → 100 %). If the block was windowed it collapses to 20 lines (200 ms, anchored).
- **Copy:** copy ✓ morph (message-bubble §4 timings: 160 ms, hold 1500 ms); label "Copy" → "Copied"
  rolls (Arc rolls its toggle labels, note 12).
- **Wrap toggle:** lines re-wrap without animation (animating re-wrap is unreadable); the toggle icon
  swaps; the scroll position inside the block keeps the first visible line.
- **Show all / Collapse:** height spring (`dur.collapse`), chevron rotates 180°, label rolls.
- **Run:** ▶ → ■ morph (160 ms `spring.snap`, same as send control); output panel unfolds (200 ms); exit
  chip enters with `dur.row.enter`; success ✓ holds then rests; failure tone fades 200 ms, no shake.
- **Apply:** button → 12 px loader → ✓ "Applied +12 −3" (diff numbers roll); Undo stays 10 s with a
  shrinking 2 px underline as its timer (proposed).
- **Diagram render:** code view crossfades to the diagram (200 ms); height change anchored.
- **Table:** column widths that grow while streaming animate with `spring.snap` 160 ms (text-reveal);
  "Show all rows" unfolds 200 ms.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Code chrome enter | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Line commit (code) | 80 ms fade | — | linear | — | 30 Hz (text-reveal) |
| Tail-line re-highlight | ≤ 1 per 100 ms | — | none (instant colour) | — | — |
| Language label swap | 120 ms | on info-string end | `ease.out` | — | 60 Hz |
| Line count roll | 280 ms (`dur.digit.roll`) | — | `ease.out` | 30 ms right → left | 30 Hz |
| Actions enter on complete | 120 ms | 0 | `ease.out` | 30 ms | 60 Hz |
| Window → collapsed (on complete) | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Show all / collapse | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Copy ✓ / hold / revert | 160 / 1500 / 160 ms | 0 | `ease.out` | — | 60 Hz |
| Run ▶ → ■ | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Output panel unfold | 200 ms | 0 | `ease.out` | — | 60 Hz |
| Applied ✓ + diff stats roll | 160 ms + 280 ms | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| Undo timer underline | 10 000 ms | 0 | linear | — | 15 Hz |
| Diagram code → render | 200 ms | on close + render time | `ease.in-out` | — | 60 Hz |
| Table column grow | 160 ms | — | `spring.snap` | — | 60 Hz |
| **Reduced motion** | everything above → 0–80 ms opacity crossfades; heights snap; digits swap; Undo timer shown as static "Undo (10 s)" text counting at 1 Hz; code lines appear per line without fade | — | linear | none | on change |

## 6. Variants

### 6.1 `markdown` — Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `markdown/chat` | Comfortable reading typography (15/24), restrained headings, 72 ch measure (default) | chat, desktop, mobile |
| `markdown/compact` | 13/20 body, tighter gaps, headings only one step above body | ide |
| `markdown/terminal` | Cell-based: bold headings with `#` hidden, `•` bullets, box-drawing tables, 16-colour code | terminal |

### 6.2 `markdown` — Pro (8)
| Name | One-line idea |
|---|---|
| `markdown/editorial` | Report typography: serif body, small caps headings, pull-quotes — for research answers |
| `markdown/outline-rail` | Long answers get a live table-of-contents rail; current section highlights while streaming |
| `markdown/foldable-sections` | Every heading section is collapsible; finished sections auto-fold in very long answers |
| `markdown/inline-preview` | Links unfold into compact rich previews on click (title, favicon, snippet) |
| `markdown/glossary` | Terms the agent marks get dotted underlines with hover definitions |
| `markdown/diagram-live` | Diagrams render progressively as nodes and edges arrive instead of on close |
| `markdown/table-studio` | Rendered tables gain sort, filter, column hide and "Chart this" (family I handoff) |
| `markdown/reading-ruler` | A soft focus band follows the paragraph being written; settled text above dims slightly (contrast-floored) |

### 6.3 `code-block` — Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `code-block/header` | Header with language label, line count, Copy, Wrap; Run / Apply when the host wires them (default) | chat, desktop, ide |
| `code-block/minimal` | No header; Copy appears top-right on hover/focus; language as a tiny corner label | chat, mobile |
| `code-block/file` | Filename + path header, line numbers, Apply / Open in editor; diff stats after apply | ide, desktop |
| `code-block/terminal` | `│` left rail, `$` prompt lines dimmed, no background; actions as key hints | terminal |

### 6.4 `code-block` — Pro (12)
| Name | One-line idea |
|---|---|
| `code-block/diff-apply` | Shows the block as a diff against the target file before Apply, with per-hunk keep / undo (family H) |
| `code-block/split-preview` | Code \| live preview side by side for HTML / SVG / UI snippets (ChatGPT Preview idea, note 04) |
| `code-block/tabs` | Several fences for one change become tabs (one per file) with a combined Apply all |
| `code-block/gutter-light` | A light runs down the gutter as lines commit, settling into line numbers |
| `code-block/minimap` | Long blocks get a right-edge minimap with the visible window outlined |
| `code-block/annotated` | Agent annotations pinned to specific lines, expandable inline |
| `code-block/fold-regions` | Functions / impl blocks foldable with syntax-aware ranges |
| `code-block/explain-line` | Hover a line → "Explain" / "Change this" sends a scoped follow-up |
| `code-block/output-dock` | Run output docks under the block as a terminal tab with exit chip, duration and rerun history |
| `code-block/stack-trace` | Error traces render as frames with file:line links and collapsed library frames |
| `code-block/ascii-frame` | Retro box-drawing frame with a title bar for terminal-styled apps on any surface |
| `code-block/glass` | Translucent glass card with a soft language-tinted rim (marketing / consumer chat) |

## 7. Edge cases
- **Huge outputs (> 100 KB per message):** settled blocks virtualise ([messages.md](messages.md) §4.1);
  highlight caps §3.3; the in-flight reveal budget stays at 64 units (text-reveal).
- **Fence inside a fence / tilde fences / 4+ backticks:** follow CommonMark fence-length rules; the
  repair pass never closes a fence early (MonoCode's `remend` skips fences, note 08).
- **Code with very long lines:** no wrap by default; horizontal scroll inside the block; wrap toggle
  persists per session (proposed); a 12 px fade on the overflowing edge.
- **Tabs and mixed indentation:** tab width 4 (token); visible-whitespace off.
- **RTL:** prose mirrors; code blocks, inline code, math and tables of numbers stay LTR (bidi
  isolates); list bullets move to the right; the code header's actions stay trailing in reading
  direction.
- **CJK:** prose line-breaking follows the platform; code font fallback must include CJK glyphs at mono
  width ×2 (cell alignment in terminal surface).
- **Tiny widths (< 320 px):** code header shows icon-only Copy; line numbers hidden; tables become
  "Show table" disclosure that opens a full-width sheet.
- **Dense IDE panels:** `markdown/compact` + `code-block/file`; Apply / Open in editor are primary.
- **Touch:** code blocks scroll horizontally with the finger without scrolling the transcript (axis
  lock after 8 px); Copy 44 px hit area.
- **Slow networks:** a fence open for > `stall_soft` keeps the "Writing" header and loader; the
  composer status handles "Still working".
- **Very fast streams:** whole blocks arrive at once (text-reveal CatchUp) — the code block goes
  `Opening → Complete` in one frame: chrome and body appear together, no window phase.
- **Interrupted mid-fence:** `Truncated` (§2.3); copying works; Run/Apply disabled with the reason.
- **Unknown / unsafe languages for Run:** Run hidden; never infer runnable from content.
- **Math:** rendered on close; render failure → raw LaTeX in mono with a dim "Couldn't render" note.
  The math layout engine for GPUI is an open question (see Gaps in §10).
- **Images in markdown:** placeholder box (text-reveal §5); broken image → alt text in a bordered box;
  remote images can be blocked by host policy ("Image blocked · Load").
- **Malicious markdown:** links with mismatched text/destination show the real domain in the hover
  tooltip; raw HTML off by default.

## 8. Accessibility
- Rendered markdown uses real semantics: headings are headings (levels offset by the host so an answer's
  `#` is not the page's h1 — proposed default offset +2), lists are lists, tables have header cells and
  a caption ("Table, 84 rows, 5 columns"), task-list checkboxes are read-only checkboxes.
- **Code block:** a `region`/`group` labelled "Rust code, 312 lines" (Arc labels its block "hello.ts
  source code", note 12); the `pre` is focusable for keyboard scrolling; Copy "Copy code" → "Copied";
  Wrap is a toggle button (`aria-pressed`); Show all has `aria-expanded` + `aria-controls`; Run "Run
  snippet" / "Stop snippet"; Apply "Apply to auth.rs".
- **Streaming:** `aria-busy="true"` on the message while open; code blocks in `Streaming` expose
  "Code block, writing" and their actions as disabled with the reason as description.
- **Announcements** (polite, composer live region): "Snippet finished, exit 0" / "Snippet failed, exit
  1" (assertive), "Applied to auth.rs, 12 additions, 3 deletions", "Apply conflict" (assertive).
- **Contrast:** syntax themes are checked so every token colour ≥ 4.5:1 on the code background in both
  themes (Codex corrects pale custom theme colours automatically, note 08 — we apply the same contrast
  floor at token load); inline code background ≥ 3:1 is not required, text contrast is.
- **Math:** exposes its LaTeX source as the accessible name until a MathML path exists on that platform.
- **Diagrams:** the source code is the accessible alternative ("Diagram, flowchart. Show source").
- **Reduced motion:** §5 last row.
- **Keyboard:** inside a focused code block, ⌘/Ctrl+C with no selection copies the whole block
  (proposed); `w` toggles wrap when the block is focused (proposed, listed in the ? help).

## 9. Props sketch
```rust
pub struct Markdown {
    pub source: BlockStream,                 // incremental blocks from oxyn-core's tail parser
    pub drive: RevealDrive,                  // shared with TextReveal: FromState(&AgentState)
    pub variant: MarkdownVariant,            // Chat | Compact | Terminal | Pro(..)
    pub heading_offset: u8,                  // 2
    pub html: HtmlPolicy,                    // Off (default) | Sanitised(AllowList)
    pub links: LinkPolicy,                   // { schemes: [Http, Https, Mailto], external_glyph: true }
    pub images: ImagePolicy,                 // Load | BlockRemote | AskFirst
    pub math: MathMode,                      // Render | Source
    pub diagrams: Option<Box<dyn DiagramRenderer>>,
    pub collapse: CollapsePolicy,            // { code_window: 40, code_collapsed: 20, code_threshold: 30, table_rows: 10, table_threshold: 20 }
    pub code: CodeBlockCfg,
    pub motion: MotionLevel,
}

pub struct CodeBlockCfg {
    pub variant: CodeVariant,                // Header | Minimal | File | Terminal | Pro(..)
    pub highlight: HighlightCfg,             // { tail_interval: 100 ms, frame_budget: 2 ms, window_above: 2000, plain_above: 10_000 }
    pub wrap_default: bool,                  // false
    pub line_numbers: LineNumbers,           // Auto (on in File) | On | Off
    pub on_run: Option<Callback<RunRequest>>,     // shown only when set and lang ∈ run_langs
    pub run_langs: Vec<LangId>,              // sh, bash, zsh, fish, pwsh, console
    pub on_apply: Option<Callback<ApplyRequest>>, // target path, content; result drives Applied/Conflict/Failed
    pub on_open: Option<Callback<OpenRequest>>,
    pub labels: CodeLabels,
}

pub enum CodeBlockState {
    Opening, Streaming { lines: u32 }, Ready, Collapsed, Expanded,
    Running { since: Instant }, Ran { exit: i32, took: Duration }, RunFailed { exit: i32, took: Duration },
    Applying, Applied { added: u32, removed: u32 }, Conflict, ApplyFailed { reason: String }, Truncated,
}
// AgentState mapping: Streaming → active block Opening/Streaming; ToolRunning/AwaitingApproval → tail ClosedVisually;
// Done → Ready (+ final highlight); Cancelled/Error → open fence Truncated. Run/Apply states come from host callbacks,
// never from AgentState.
```

## 10. Related components and references
Related: [text-reveal.md](text-reveal.md) (pacer, incomplete-syntax table, caret, follow) ·
[message-bubble.md](message-bubble.md) (copy semantics at message level) ·
[citations-sources.md](citations-sources.md) (inline markers) · [skeletons.md](skeletons.md) ·
family F (terminal output, tool rows) · family G (approval before Run) · family H (diff viewer, apply
conflicts) · family I (gen-UI blocks, charts from tables).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Codex CLI streaming (line commit, table hold-back, mutable tail, 8 KB preview cap) and 10 000-line highlight cap; theme contrast correction | note 08 | streaming discipline, caps, contrast floor | Apache-2.0 per note 08 (MIT in licensing.md — re-check) — portable with attribution |
| MonoCode Streamdown + `remend` per-block repair (skips fences) | note 08 | repair-then-render | MonoCode MIT; Streamdown licence not recorded — check before port |
| Conductor "Run button on bash snippets" with inline output | notes 03, 08 | Run action + output panel | Proprietary docs — idea only |
| Claude desktop code-review card ("Fix this one / Apply fixes") | note 03 | Apply flow idea | Proprietary — idea only |
| ChatGPT code / writing blocks with Preview | note 04 | `split-preview` | Product — study only |
| Arc UI code block (filename + language header, Copy pill, `maxLines` collapse with Show all, focusable `pre`, label "hello.ts source code"; "use text-stream for code still arriving") | note 12 | header anatomy, collapse, a11y labels | Paid / unclear — study only |
| Zed DiffStat (thin space U+2009, figure dash U+2012) | note 08 | diff numbers after Apply | Mixed by crate — idea only (typography convention) |
| Vercel AI Elements Code Block, prompt-kit Code Block / Markdown, shadcn AI Elements | note 01 | API shapes | AI Elements Apache-2.0, prompt-kit MIT, shadcn MIT — may port with attribution |
| Beautiful UI Code Block (line numbers + unified diff, Copy) | [R133], note 17 | `file` / `diff-apply` | MIT — may port with attribution |
| Windsurf / Paradev Mermaid render | note 03 | diagram fence | Products — study only |

**Gaps:** math layout on GPUI (no engine chosen); highlighter engine and grammar licences; whether
GPUI's text system can apply per-line style runs cheaply enough for 2 000-line windows (V2.6 spike).
