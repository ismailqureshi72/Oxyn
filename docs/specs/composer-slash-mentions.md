**Status:** v1 — implemented (plan #27); the draft below is kept as the design record.

# Spec: slash commands, @mentions, $skills and #context (trigger popovers + inline tokens)

Follows the 10-point standard in [README.md](README.md) at the depth of
[composer-send-control.md](composer-send-control.md). Target **9 free + 15 Pro variants** across two
parts that share one state machine: the **trigger popover** (`trigger-menu/*`) and the **inline token**
it inserts (`token-chip/*`).

## 1. Purpose
Power users drive agents from the keyboard. `/` runs commands (`/plan`, `/review`, `/model`), `@` pulls
context in (files, folders, symbols, previous threads, URLs, people) or calls an agent or app, `$` picks a
skill ("@ files, $ skills, / commands" [R003]) and `#` adds context or steering rules (Kiro uses `#` for
steering, research notes 03). The popover must be fast, forgiving (fuzzy), honest about async search
(loading, empty, error), and once something is picked, the text area must show it as a solid token the
user can trust, hover, and delete as one unit. Research found two meanings of a mention, **attach
context** versus **force a tool** (research notes 04: Gemini `@Gmail`, Raycast `@extension` versus Dia
and Notion page mentions); the library must make that difference visible.

## 2. State machine

```
            trigger char typed at a valid boundary            query changes
  Inactive ──────────────────────────────────────▶ Open(Idle) ───────────────▶ Querying(q)
     ▲                                                │   ▲                         │ results / timeout
     │  Esc · caret leaves token · 2 spaces ·         │   │                         ▼
     │  boundary char with no match · blur            │   └──── query edited ── Results(n) | Empty | Error(retry)
     │◀───────────────────────────────────────────────┴──────────────────────────────┤
     │                                                                               │ Enter / Tab / click
     │                                                                               ▼
     │                                     Accepting ──▶ Token inserted ──▶ (ArgHint) ──▶ Inactive
     │                                                         │ Tab on folder / group → drill: Open(Idle) with new scope
  Dismissed(at offset)  ← Esc: the same trigger is not re-opened until the user types a new trigger char
  Token states:  Inserted ─hover─▶ Previewing · ─Backspace─▶ Selected ─Backspace─▶ Removed
                 Inserted ─target gone─▶ Broken("file deleted") · Inserted ─needs auth─▶ NeedsAuth
```

**Trigger detection rules**
1. The trigger char is at the start of the draft or follows whitespace or an opening bracket/quote.
   `/` as a *command* trigger additionally requires start of line by default (`slash_anywhere = false`),
   so paths like `src/lib` and fractions never open the menu.
2. Never inside a code span or fenced block, a URL, or an email address (`a@b.com` does not open).
3. Never during IME composition; the check runs on composition end.
4. The query is the text from the trigger to the caret. It ends at whitespace for `/`, `$` and `#`. For
   `@` it may contain one space while results still match ("@auth mid"), and closes on two spaces or after
   2 further characters with zero matches (proposed).
5. Esc dismisses for this occurrence; the text stays literal. Typing the trigger twice (`//`, `@@`)
   inserts one literal char (proposed).
6. Async sources are keyed by query; a stale response (older key) is dropped (the `queryKey` idea from
   coss ui [R126]).

## 3. States table

**Popover**

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Open(Idle)` | header with trigger scope ("Commands", "Mention files, symbols, threads…"); recent + suggested items | pick | switch tab | ↑/↓, Enter/Tab accept, Esc dismiss |
| `Querying(q)`, cached results present | previous results stay visible at 60 % opacity; thin indeterminate bar on the header | pick (from stale list, still valid) | — | as above |
| `Querying(q)`, nothing cached | 3 skeleton rows after a 120 ms delay | — | — | Esc |
| `Results(n)` | grouped rows; matched characters in medium weight; type badge per row; first row highlighted | accept | preview pane (variant) | ↑/↓ · PageUp/PageDown · Enter accept · Tab accept-and-continue · Ctrl+Tab next tab |
| `Empty` | "No files match 'zqx'" + one hint ("Try a path like src/" or "Type / to see all commands") | — | create (e.g. "Add URL 'zqx'?") when the source allows | Esc |
| `Error` | "Couldn't search files · Retry"; other groups still listed | retry | — | Enter retries when the row is highlighted |
| `Drilled(scope)` | breadcrumb "src › middleware ›"; children of the folder / group | accept | back | Backspace at empty query goes up one level |
| `ArgHint` | after a command token: ghost argument text `<branch> [--strict]`, fields fill inline | fill args | skip | Tab next arg · Shift+Tab previous · Enter send/confirm |
| `Disabled item` | muted row + reason ("Can't /clear while the agent is working") | — | — | skipped by ↑/↓ (still read by screen readers) |
| `NeedsAuth item` | amber dot + "Sign in" for an app/agent mention | sign in | — | Enter |

**Inline token**

| Token state | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Inserted` | chip inline with text baseline; icon + label; kind styling (see below) | hover/focus preview | remove | caret skips over it as one character |
| `Previewing` | hover card after 300 ms: file path + first lines, thread title + date, skill description, command help | open | remove | Enter opens when focused |
| `Selected` | 2 px ring on the chip after one Backspace | — | — | second Backspace removes; any other key deselects |
| `Broken` | strike-through label + warning icon ("auth.rs was moved") | fix (re-pick) | remove | — |
| `NeedsAuth` | amber dot on an app/agent token | sign in | remove | — |
| `Running` (command tokens that act immediately, e.g. `/compact`) | the token shimmers once and the action runs; the token is removed when done | — | undo when available | — |

**Kinds and how they differ** (shape + icon + prefix, never colour alone):

| Kind | Trigger | Means | Token style (default) |
|---|---|---|---|
| Context mention | `@` | add a file, folder, symbol, thread, URL, person as context for the agent to read | neutral fill, type icon, no prefix glyph shown |
| Tool / agent / app mention | `@` | **call** a tool, sub-agent or app ("will run") | accent outline, lightning/robot icon, label "Gmail · runs" in the preview card |
| Skill | `$` | load a skill / instruction module | dotted outline, book icon, `$` prefix kept |
| Context / steering rule | `#` | attach a rule file or steering doc | neutral fill, hash icon, `#` prefix kept |
| Command | `/` | change mode, run an action, or insert a template | mono label `/plan`; mode commands also tint the composer's mode chip |

Rule: a command that **acts** (e.g. `/clear`, `/compact`) runs on accept and asks first if it is
destructive; a command that **modifies the message** (e.g. `/plan`, `/review <branch>`) becomes a token
sent with the message. Queued slash commands are parsed when the queued message runs, not when it is
queued (Codex CLI behaviour, research notes 03).

## 4. Micro-interactions
- **Open.** Popover appears above the caret (below if no room), anchored to the trigger char's x
  position, scaling from 0.98 with a 4 px rise and fade. The trigger char gets a faint underline while
  the popover is open, so the user sees what is being completed.
- **Loading.** Stale results dim instead of disappearing (no layout flash); skeleton only when nothing
  is cached and the wait exceeds 120 ms.
- **Filtering.** Rows re-order with a short glide when the move is small and snap when it is a long jump
  (a rule from Arc's command palette, research notes 12); the highlight glides on a snappy spring; the
  panel height springs to fit.
- **Match highlight.** Matched characters render in medium weight (not colour) as the user types.
- **Accept.** The highlighted row's label flies from the popover into the caret position and lands as
  the token chip (shared-element, 200 ms); the popover fades out at the same time. Reduced motion: the
  token appears in place.
- **Mode command shimmer.** A mode token (`/plan`) gets a single 900 ms ease-out light sweep as it lands
  (MonoCode's composer mode shimmer, MIT, research notes 08).
- **Arg hint.** Ghost text fades in after the token; each filled argument becomes a sub-chip inside the
  command token (inline argument fields, a pattern seen in Paradev, research notes 03).
- **Backspace.** First press rings the token (selected), second press shrinks it to 0.9 and fades it out;
  the following text slides left to close the gap.
- **Broken token.** When its target disappears, the label strikes through over 200 ms and a warning icon
  fades in; no popup.
- **Hover card.** Appears after 300 ms hover or immediately on keyboard focus; stays while the pointer
  moves into it.
- **Tabs.** Switching tabs (All · Files · Symbols · Threads · Skills…) slides the list 12 px in the
  direction of travel and cross-fades (tabbed `@` popup idea from Codex CLI, research notes 03).

## 5. Timing (defaults)

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Popover open (scale 0.98→1, y 4→0, fade) | 140 ms | 0 | ease-out cubic-bezier(0.22, 1, 0.36, 1) | proposed |
| Popover close | 100 ms | 0 | ease-in | |
| Local source debounce | — | 0–30 ms | — | in-memory lists filter every keystroke |
| Remote source debounce | — | 120 ms | — | cancel in-flight on new key |
| Skeleton appears | — | 120 ms | — | never flashes for fast sources |
| Stale-results dim | 100 ms | 0 | linear | 100 % → 60 % |
| Row reorder glide | 140 ms | 0 | spring (stiffness 500, damping 40, proposed) | snap when moving > 6 rows |
| Highlight glide | 120 ms | 0 | spring (stiffness 600, damping 40, proposed) | |
| Panel height | 160 ms | 0 | spring, no overshoot | |
| Accept fly-in to caret | 200 ms | 0 | ease-in-out | shared element |
| Mode token shimmer | 900 ms, one sweep | on insert | ease-out | MonoCode, MIT |
| Arg ghost text | 120 ms | 60 ms | ease-out | |
| Token selected ring | 80 ms | 0 | ease-out | |
| Token remove | 120 ms + 140 ms text close | 0 | ease-in / spring | |
| Hover card | 140 ms | 300 ms (0 on focus) | ease-out | |
| Tab switch | 160 ms | 0 | ease-out, 12 px | |
| Broken strike-through | 200 ms | 0 | ease-out | |
| Request timeout → `Error` | — | 8 s (proposed) | — | |
| **Reduced motion** | 0–80 ms opacity cross-fades | — | linear | no fly-in, glide, slide, shimmer or scale; highlight and list change instantly; every announcement kept |

Sizes (proposed): popover width 360 px (min 280, max 480; 560 with preview pane), list max height
320 px (about 9 rows), row 32 px (28 dense, 44 touch), group header 24 px, token height = line height
− 2 px (18 px on a 20 px line), token max width 240 px, 6 px horizontal padding, 4 px radius.

**Fuzzy matching** (shared with composer-tools-menu.md): every query word must match the label as a
prefix, a word start in the description or keywords, or as letters in order across the label
(subsequence: "shpr" finds "Share project"); label-prefix matches rank first and groups follow their best
match (the matching rules Arc's command palette documents, research notes 12, re-implemented by us).
For paths, match the filename first, then directories; prefer recently opened and git-modified files;
show the dimmed directory tail. Ties break by recency, then shortness.

## 6. Variants

### Free (9)
| Name | Idea |
|---|---|
| `trigger-menu/popover` | Baseline caret-anchored list with groups and match highlight |
| `trigger-menu/tabbed` | Tabs across the top (All · Files · Symbols · Threads · Skills), Ctrl+Tab cycles (Codex CLI idea, MIT) |
| `trigger-menu/preview` | List on the left, live preview on the right (file head, command help, skill description) |
| `trigger-menu/inline-ghost` | No popover: best match shown as ghost text after the caret, Tab accepts, ↓ opens the full list |
| `trigger-menu/sheet` | Mobile: results in a bottom sheet above the keyboard with 44 px rows |
| `trigger-menu/terminal` | TUI list under the prompt with `›` highlight and dim type column |
| `token-chip/pill` | Rounded inline chip with icon and label (default) |
| `token-chip/text` | No chip: coloured mono text with a prefix (`@auth.rs`), for terminal and minimal surfaces |
| `token-chip/args` | Command token with inline argument sub-fields and ghost hints |

### Pro (15)
| Name | Idea |
|---|---|
| `trigger-menu/spotlight` | Full-width overlay that searches all trigger scopes at once with sections and keyboard hints |
| `trigger-menu/graph` | Mentions show as nodes linked to the composer (works with `composer-input/node-graph`) |
| `trigger-menu/morph-caret` | The popover grows out of the caret itself and shrinks back into the inserted token |
| `trigger-menu/radial` | Command groups in a radial menu around the caret (pen and touch) |
| `trigger-menu/stacked-preview` | Hovering rows stacks preview cards behind the list like a deck |
| `trigger-menu/cost` | Each context row shows its token cost and the running total for the message |
| `trigger-menu/smart-rank` | Rows carry a "why" badge (recent, modified, mentioned in thread) with a small glow on the top pick |
| `trigger-menu/pixel` | Pixel-grid type icons and a pixel highlight bar |
| `token-chip/glass` | Refractive glass tokens that catch the composer's busy light |
| `token-chip/avatar` | Agent and person mentions as avatar chips with live status dot |
| `token-chip/magnetic` | The token snaps toward the caret with a small overshoot on insert |
| `token-chip/split-flap` | Command labels flip into place like a split-flap display |
| `token-chip/pile` | Many context mentions collapse into one "5 files" chip that fans out on hover |
| `token-chip/live` | Tokens show live status (thread still running, PR checks, file dirty) with a tiny pulse |
| `token-chip/arg-drawer` | Argument fields slide out of the command token like a drawer with per-arg validation ticks |

## 7. Edge cases
- **Long labels** ellipsize in the list (path tail kept: "…/middleware/auth.rs"); tokens truncate at
  240 px with full name in the preview and accessible name.
- **Thousands of results** (monorepo): sources return the top 50 per group; the list virtualises;
  "Show 1,200 more in Files" row at the end of a group.
- **Paths with spaces** are inserted quoted in the serialised text; the token still shows the bare name.
- **Ambiguity.** Two files named `auth.rs`: both rows show their directory tail; the token keeps the full
  path.
- **RTL.** Popover anchors to the trigger's visual position and aligns to its start edge; tokens follow
  bidi rules as single neutral units; the type column mirrors; arrow keys stay ↑/↓ for the list.
- **IME.** Detection waits for composition end; full-width `／` and `＠` are treated as triggers in CJK
  locales (proposed, configurable).
- **Tiny / dense IDE.** In narrow panels (< 300 px) the popover takes the panel width and drops
  descriptions; `token-chip/text` is the default in `terminal` surfaces.
- **Touch.** Tapping a row accepts; no hover cards (long-press shows them); the sheet keeps the text area
  visible above it.
- **Slow sources.** Stale results stay usable; after 8 s the group shows Error with Retry while other
  groups still work. **Very fast sources** never show a skeleton.
- **Agent busy.** Commands that would break the running turn are disabled with a reason; mentions are
  always allowed (they join the draft / queued message). Mode commands either queue for the next turn or
  are disabled, per app policy.
- **Pasting text that contains triggers** does not open the popover; pasted `@path` strings that resolve
  to real files can be offered as tokens via one "Convert 3 mentions" hint (proposed).
- **Copy/paste of tokens** keeps them as tokens inside Oxyn (structured clipboard, as coss ui does with
  its own MIME type [R126]) and as plain text (`@src/auth.rs`, `/plan`) elsewhere.
- **Undo.** ⌘/Ctrl+Z after accept restores the raw query text and reopens nothing.
- **Deleted targets** turn tokens `Broken` before send; send warns "1 mention can't be resolved".
- **Security.** Tool/app mentions that would act on the user's behalf show the "runs" label and respect
  the approval policy; a mention is never an implicit approval.

## 8. Accessibility
- The text area is the combobox: `aria-expanded`, `aria-controls` → listbox, `aria-activedescendant` →
  highlighted option. Focus never leaves the text area while the popover is open.
- Options: `role=option` with `aria-selected`, grouped in labelled `group`s; disabled options use
  `aria-disabled` and keep their reason in the accessible description.
- Polite live region announces result counts ("12 results, Files"), "No results", "Loading files",
  errors, and acceptance ("Inserted mention auth.rs", "Command plan added. Argument: branch").
- Tokens are exposed inline as text with a role description, e.g. "mention, file auth.rs" or "tool,
  Gmail, will run"; the kind is spoken, not only styled.
- Hover cards are reachable by keyboard focus on the token and dismissible with Esc.
- Contrast: token label 4.5:1 against its fill; matched-character emphasis uses weight, not colour;
  kind distinctions use icon + outline style + text.
- Focus order is unchanged by the popover; Esc always returns to plain typing.
- Reduced motion per §5.

## 9. Props sketch

```rust
pub struct TriggerMenu {
    pub agent: AgentState,                    // busy → per-command availability
    pub triggers: Vec<TriggerDef>,            // registry: '/', '@', '$', '#' and app-defined chars
    pub variant: TriggerVariant,              // Popover | Tabbed | Preview | InlineGhost | Sheet | Terminal
    pub matcher: MatcherConfig,               // subsequence + word-start + prefix weights, recency boost
    pub max_rows: u16,                        // 9 visible
    pub remote_debounce_ms: u32,              // 120
    pub skeleton_delay_ms: u32,               // 120
    pub timeout_ms: u32,                      // 8000
    pub surface: Surface,
    pub labels: TriggerLabels,
    pub motion: MotionLevel,
    pub on_accept: Callback<AcceptedItem>,
}

pub struct TriggerDef {
    pub ch: char,                             // '/'
    pub boundary: Boundary,                   // LineStart | WordStart | Anywhere
    pub allow_space_in_query: bool,           // true for '@'
    pub source: ItemSource,                   // Static(Vec<Item>) | Async(fn(query, key) -> Future<Result<Vec<Item>>>)
    pub tabs: Vec<TabDef>,                    // optional grouping into tabs
}

pub enum TokenKind {
    Context(ContextRef),                      // file, folder, symbol, thread, URL, person
    Invoke { target: ToolOrAgentId },         // "will run" — mention that calls a tool or agent
    Skill(SkillId),
    Rule(RuleId),                             // '#' steering / context rule
    Command { name: String, args: Vec<ArgSpec>, acts_immediately: bool, destructive: bool },
}

pub struct TokenChip {
    pub kind: TokenKind,
    pub label: String,
    pub state: TokenState,                    // Inserted | Previewing | Selected | Broken(String) | NeedsAuth | Running
    pub variant: TokenVariant,                // Pill | Text | Args | ...
}
```

## 10. Related components and references
**Related:** composer input shell (owns caret, IME, clipboard) · tools menu (same row component and
matcher; skills and connectors appear in both) · attachments (context chips; an `@file` can be shown as a
chip in the attachment row instead of inline, app choice) · mode switch (mode commands tint it) · model
picker (`/model`) · command palette in workspace chrome · approval card (tool mentions that act).

**References** (study only unless marked MIT/Apache; see [../licensing.md](../licensing.md)):
- [R003] T3 Code placeholder "@ files, $ skills, / commands" (licence not checked: study only).
- [R021] Phil Pitus composer "type / for skills". Study only.
- [R126] coss ui Composer: structured draft of text / mention / action parts, `@` `/` `:` triggers, async `onQueryChange` / `itemsFor` / `queryKey`, loading / error / `showEmpty`, `renderToken`, structured clipboard, suggestion-count announcements. **MIT**: API shape portable with attribution.
- Codex CLI: tabbed `@` popup (All Results / Filesystem Only / Plugins with type badges), queued slash commands parsed at run time. **MIT**, port with attribution (research notes 03).
- MonoCode: composer mode shimmer, single 900 ms ease-out sweep. **MIT** (research notes 08).
- OpenCode (`/undo`, `/redo`, `/share`, `/thinking`): **MIT**. Cline: **Apache-2.0** (research notes 03, licensing.md).
- Zed typed `@` (symbols, diagnostics, threads, branch diffs): mixed licence, study only by default. Paradev inline command chips with argument fields; Kiro `#` steering; Claude, Cursor slash menus; Gemini / Raycast tool-forcing mentions; Dia / Notion context mentions: proprietary or unchecked, reference only (research notes 03, 04).
- Arc UI command palette (matching rules, glide vs snap, live-region counts): paid / unclear, study only (research notes 12).
- [R106] / [R133] Beautiful UI Prompt Bar (@ sources, / commands, model picker, dictation; Rounded / Pill). **MIT**, port with attribution.


## Plan #27 implementation — mention input

Implemented in `oxyn-core::composer::parts`, `oxyn-dioxus::{MentionInput}` (styles in `PARTS_CSS`, bundled into `PRIMITIVES_CSS`), `oxyn-gpui::{MentionListEl}`.

### 27.1 Purpose
Mention people, files and agents with `@` and run commands with `/` from the message field (Arc UI Mention input, rebuilt from its behaviour).

### 27.2 State machine
```
Typing ─@ or / at a word start─▶ Open (query = token after trigger, ranked by `rank`)
Open ─↑/↓─▶ move   Open ─Enter/Tab─▶ accept (`accept` replaces the token, adds a space) ─▶ Typing
Open ─Escape | caret leaves token─▶ Typing
```

### 27.3 States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| typing | field with pills preview | Enter sends | combobox |
| open | grouped list (People, Files, Agents, Commands) | ↑ ↓ Enter Tab Escape | listbox, `aria-activedescendant` |
| no match | “No one matches “@q”” or “No command matches” | Escape | count announced |

### 27.4 Micro-interactions
The list pops in with a 4 px rise; the active row highlights on hover and keys alike; accepted mentions render as pills.

### 27.5 Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| List in | 160 ms | `--ox-ease` | instant |
| Row highlight | `--ox-fast` | — | instant |

All motion in the parts drops to 1 ms under `prefers-reduced-motion`, except `.oxyn-timer`.

### 27.6 Variants
Free: `@` mentions with grouping.

**Oxyn plus:** `/` commands only at a line start, upward opening for bottom composers, row limit, pill preview.

### 27.7 Edge cases
`/` mid-sentence is plain text. Prefix matches rank above substring matches. Unknown `@names` stay plain text.

### 27.8 Accessibility
Combobox pattern: `aria-expanded`, `aria-activedescendant`, options with `aria-selected`; result count in a polite region.

### 27.9 Props sketch
```rust
MentionInput { suggestions: Vec<Suggestion>, commands: Vec<Suggestion>, value, limit: usize, placeholder, up: bool, on_submit: EventHandler<String> }
```

### 27.10 Related components and references
Arc UI Mention input — behaviour only.
