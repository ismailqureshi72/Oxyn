# Beautiful UI → Oxyn parity inventory

**Source:** https://www.beautifului.dev/ (Beautiful UI, "Crafted primitives for AI-native interfaces", by Turbo / Shane Levine). Read on 2026-11-12.
**Research IDs:** R106 and R133 in `/workspace/work/Oxyn/docs/research/sources.md`. Earlier notes are in `research/16-niche-sites-a.md` §Beautiful UI and `research/17-niche-sites-b.md` §15 (project workspace).
**Licence:** MIT, © 2026 Shane Levine (https://www.beautifului.dev/license). The repo is also MIT: https://github.com/slev12397/beautiful-ui (362★, 49 forks). **Free/paid:** everything is free. There is no Pro tier and no pricing page. One caveat: `SidebarNav` imports `@central-icons-react`, a **paid** icon set that checks a licence key on `npm install` (stated in the repo README).
**Stack (from the repo README):** Next.js App Router · React 19 · Tailwind v4 · TypeScript. One `.tsx` file per primitive, built on a `foundation.css` token layer with light and dark themes.

## How this was read
- **The index page** is server-rendered, so all 21 sections came through `web_fetch`.
- **No per-item pages exist.** `/components`, `/docs`, `/components/<id>`, `/sitemap.xml`, `/robots.txt` and `/llms.txt` all return 404. Each item is an anchor on the one page, for example `https://www.beautifului.dev/#loading-state`.
- **Source code came from the shadcn registry.** `https://www.beautifului.dev/r/registry.json` lists 27 items (1 style, 6 ui atoms, 20 components). I downloaded every `/r/<id>.json` and read the full source for props, types, state unions, keyboard handling and aria.
- **Agent Screen is not in the registry.** I read it from the repo (`components/primitives/AgentScreen.tsx`), together with the site chrome under `components/site/*`.
- **`/harness`** was fetched and the `IceCreamHarness.tsx` source was read.
- **Not read: none.** Every item has a source.

**Legend.**
- **Oxyn spec** gives the file in `/workspace/work/Oxyn/docs/specs/` (or `docs/site/*`, `docs/categories/*`) that covers the item.
- **Partial** means a spec exists but lacks this item's distinctive behaviour.
- **NEW** means no spec covers it.
- **Oxyn plus** lists extra variants or controls Oxyn should ship beyond parity.
- All items are Free + MIT unless noted, so the Free/Licence column is left out of the tables.

---

## A. Loaders & status

| # | Name · URL · registry id | Purpose | States | Controls / props / variants | Micro-interactions | Oxyn spec | Oxyn plus |
|---|---|---|---|---|---|---|---|
| 01 | **Loading State** · [#loading-state](https://www.beautifului.dev/#loading-state) · `loading-state` | Loader for long-running work: pixel grid + shimmering label + live elapsed time | running (only state); reduced-motion = grid frozen dim, timer still ticks | `variant`: **Drive** (square cells, chevron wavefront, 650 ms), **Dots** (round cells), **Orbit** (comet lapping the 3×3 perimeter, 950 ms), **Surfer** (Drive + a looping meme video below); `label` (default "Churning"); `videoSrc` | 3×3 grid of 4 px cells, `pixel-on` keyframe with per-cell delays so two fronts are always in flight; label `shimmer-text`; timer in mono tabular figures, `0.0s` → `1m 2.3s`; `role=status` | Existing: `loaders.md`, `loader-glyph.md`/`loader-vector.md` (grid), `elapsed-timer.md`, `text-shimmer.md`. Surfer = **NEW** variant | (1) grid size N×M plus cell shape (square / round / diamond / glyph) and a wave-path picker (chevron, spiral, rain, random); (2) `phase` input from `AgentState` (queued → working → stalled turns the grid amber after N s); (3) "companion" slot for any media / mini-game / tip carousel, not only video |
| — | **Shimmer** (atom) · `/r/shimmer.json` | Shimmering text wrapper | on | `children`, `className` | `shimmer-text` gradient sweep | Existing: `text-shimmer.md` | Direction, speed and width tokens; per-glyph wave; "pause on hover" |
| — | **StreamText** (atom) · `/r/stream-text.json` | Reusable streaming text: fast character reveal with a blurred leading edge | streaming → idle | `text`, `charsPerTick` (2), `tickMs` (9), `blurTail` (6), `caret` (bool), `onProgress`, `onDone` | Trailing N chars blurred; caret solid while streaming, blinks (`caret-blink`) once settled | Existing: `text-reveal.md` (5 reveals + 3 carets) | Word/sentence units; pacer that follows real token rate (text-reveal §3); "catch-up" burst when the buffer runs long |

## B. Agent / chat elements

| # | Name · URL · id | Purpose | States | Controls / props / variants | Micro-interactions | Oxyn spec | Oxyn plus |
|---|---|---|---|---|---|---|---|
| 02 | **Thinking** · [#thinking-state](https://www.beautifului.dev/#thinking-state) · `thinking-state` | Expandable agent trace | running (spinner + live label) → settled ("Thought for 4 seconds", "Ran 3 tools", "Searched the web"); collapsed / expanded | `variant`: **Steps** (spinner → muted checks), **Reasoning** (prose expands, then settles), **Search** (query + sources read, favicon/domain rows), **Coding** (Read / Edit / Run rows with file names and +add −del); `rows` override ({primary, secondary, mono, add, del, href}); `active` / `done` labels; `icon`; `onSettled` | Runs once, then auto-settles and stays expandable; header chevron rotates; rows stagger in; `role=status` | Existing: `reasoning-block.md`, `tool-call.md`, `tools.md`, `trace-view.md` | (1) Mixed trace (reasoning + search + code interleaved in one timeline); (2) display level toggle (summary / raw / hidden) per thread; (3) live one-line subtitle while collapsed + token / time counter |
| 03 | **Streaming Text** · [#streaming-text](https://www.beautifului.dev/#streaming-text) · `streaming-text` | Streamed answer with inline sources, actions and follow-ups | streaming → done → (loop restart in demo); sources collapsed / expanded | `content` (tokens; `cite:true` renders an inline source chip), `sources` ({name, domain, href, image}), `followUps`, `labels` {sources, followUps}, `loop`, `fill`, `onDone`, `onFollowUp` | Words resolve out of blur; inline citation chips appear in context; favicon avatar stack "10 sources" toggles a source list; action icons (copy, retry, reply) and follow-up chips become usable only after done | Existing: `text-reveal.md` + `streaming-markdown.md` + `citations-sources.md` + `message-bubble.md` | (1) Hover-card source previews with quote highlight; (2) citation styles (numbered [1], chip, superscript, underline-on-hover); (3) follow-up layouts (chips / list / "ask anything" input) + regenerate-with-model |
| 05 | **Tool Chips** · [#tool-chips](https://www.beautifului.dev/#tool-chips) · `tool-chips` | An agent run as compact rows: tool calls with inline chips, then file-diff chips | collapsed run header ("4 tool calls, 2 messages") / open; each row closed / open; diff preview open | `steps` ({icon, label, chip, mono, detailMono, detail[]}), `diffs` ({file, add, del}), `diffLines` per file, `labels` {header, more}, `onOpenChange`, `onToggleRow` | Hover a row to reveal its chevron; every row expands to show what the tool did; **hovering a file chip opens its diff** (green / red lines) in a body portal | Existing: `tool-call.md`, `tools.md` (ToolStack), `file-edit-chip.md` | (1) Live state per chip (queued / running spinner / failed / needs-approval); (2) density switch (chips / rows / cards); (3) pin a diff preview, open it in the full `diff-viewer` |
| 06 | **Task Rows** · [#task-rows](https://www.beautifului.dev/#task-rows) · `task-rows` | Live agent task status | per row: done (green check + "Completed" pill), running (ring progress + step number), sequence (pending → **Failed** + retry → Completed); row open / closed | `variant`: **Capsules** / **List**; `rows` ({key, label, amount, status, step, details[{label, meta}]}); `labels` {completed, failed}; `onToggleRow` | Rows enter staggered 80 ms; ring sweeps 0 → 66 %; row 1 auto-expands its detail steps then collapses; row 2 flips to Failed then resolves; details stay clickable | Existing: `plan-todo.md` (todo list / step timeline), `background-jobs.md` | (1) Owner lanes (agent vs human) and dependencies between rows; (2) inline Retry / Skip / Cancel per row with a reason; (3) Kanban and timeline (Gantt-lite) views |
| 07 | **Chat** · [#chat-composer](https://www.beautifului.dev/#chat-composer) · `chat-composer` | Tabbed chat panel with reasoning replies and a composer | idle → sent → reply1 → reply2 → done | `messages` ({label, sub, time, body}), `suggestions` (header tabs, e.g. Flavors / Suppliers), `labels` {initialPrompt, placeholder}, `onSend` | Reply sequence starts only after send; each reply shows the tools used and a duration ("Sales History · Flavor Data · for 4s"); fixed-height conversation region so the card never changes shape; Enter sends | Existing: `messages.md`, `message-bubble.md`, `composer.md`. Tabs: Partial (no chat-tab spec) | (1) Thread tabs with unread dots, drag-reorder, close; (2) mini / docked / floating sizes; (3) per-reply tool-duration chip that expands into the trace |
| 08 | **Prompt Bar** · [#prompt-bar](https://www.beautifului.dev/#prompt-bar) · `prompt-bar` (npm `glimm`) | Composer with attach, @ sources, / commands, model picker, dictation, send | idle, typing, @ menu open, / menu open, + menu open, model menu open, listening (dictation), sending; demo-autoplay vs user-controlled | `variant`: **Rounded** / **Pill**; `demo` (self-running walkthrough, yields on first user input); `tall` (hero: multi-line, controls on their own row); `placeholder`; `onSend`. Built-in data: sources (Add photos & files, Scoop Data, Flavor records, Web search, Figma, Slack, Gmail with "Connect" / "Connected"), commands (/compare, /churn-plan, /restock, /draft-email, /summarize), models (Sprinkles 5 *Flagship*, Vanilla 1 *Basic*, Freezer Burn 0.4 *Stale*) | Keyboard ↑ ↓ Enter Tab Esc; menus grow up from the composer top edge; **one gliding highlight** floats to the hovered row; picking the flagship model fires a **rainbow glimm shader sweep** across the input; dictation shows bouncing EQ bars, then the transcript "lands" after a beat; wrapped text moves above the controls, then grows to a max; selected sources become removable chips; outside click closes menus; square send button (round in Pill) | Existing: `composer-input-shell`, `composer-slash-mentions`, `composer-model-picker`, `composer-voice-input`, `composer-attachments`, `composer-tools-menu`, `composer-send-control`. Glimm sweep: **NEW** effect | (1) Send / stop / queue / steer state machine (Beautiful UI has send only); (2) effort, context-meter and permission controls on the same bar; (3) shapes beyond Rounded / Pill: floating, inline-in-thread, terminal (`>` prompt), plus a "model change" effect picker (sweep / beam / none) |
| 18 | **Code Block** · [#code-block](https://www.beautifului.dev/#code-block) · `code-block` | Line-numbered listing and a unified diff | Code / Diff; copy idle → Copied | `variant`: **Code** / **Diff**; `lines`, `code` (clipboard text), `diff` ({old, cur, type ctx/add/del, pieces[{text, change}]}), `filename`, `labels` {copy, copied}, `onCopy` | Header: filename + Copy (turns green "Copied"), or a +N −N stat in Diff; light syntax colouring (keywords, functions, strings/numbers); old/new gutters, accent bar, row tint, **word-level** add/del highlights; lines wrap | Existing: `streaming-markdown.md` (code-block) + `diff-viewer.md` | (1) Split diff, collapsed context ("⋯ 12 lines") and minimap; (2) wrap / no-wrap, line-highlight ranges, "Apply" / "Insert at cursor" actions; (3) streaming code (agent still writing) with a typing caret |

## C. Human-in-the-loop

| # | Name · URL · id | Purpose | States | Controls / props / variants | Micro-interactions | Oxyn spec | Oxyn plus |
|---|---|---|---|---|---|---|---|
| 04 | **Approval Card** · [#approval-card](https://www.beautifului.dev/#approval-card) · `approval-card` (deps button, glide-menu) | Paged human-in-the-loop questions before the agent acts | question k of n; answered / skipped; custom answer typing; submitted ("Answers sent"); dismissed; reset ("Start over") | `questions` ({q, type **radio** / **check**, options[]}), `labels` {skip, continue, send, customPlaceholder, sentMessage}, `onSubmitted(answers)`, `onAnswerChange`, `resettable` | Stack slides vertically between questions and the card height animates to fit; **step counter rolls like an odometer** ("1 / 3"); single-choice auto-advances, multi-select waits; custom free-text answer row; Prev / Next arrows; quiet Skip + dark Continue with ⏎; Enter submits; Dismiss ×; `aria-live=polite`; reduced-motion aware | Existing: `genui-forms.md` → `genui-question/paged` (cites R106). Tool-call approvals are a separate thing, in `approval-card.md` | (1) Review page before send, plus a "recommended" badge on an option; (2) answer kinds: scale, rank, image choice, voice; (3) timeout with a default answer + "always answer this way" rule (links to `permission-rules.md`) |
| 09 | **Recommendation Card** · [#recommendation-card](https://www.beautifului.dev/#recommendation-card) · `recommendation-card` (deps button, entity-chip, value-pill) | Agent suggestion with a confidence meter and actions | default; Alternatives drawer open; alternative promoted; Accepted | `options` ({key, body, short, signal 0–3, tone, label e.g. "High confidence" / "Needs review" / "No signal", cta, ctaVariant}), `labels` {title, alternatives, otherOptions, accepted} | Card keeps its shape; "Alternatives" opens a distinct drawer section; picking one **promotes** it to the recommendation; 3-bar signal meter; inline entity chip with monogram ("Cone King") + value pill ("7 days"); primary action confirms → "Accepted" | Partial: `citations-sources.md` (`confidence-badge`) + `approval-card.md`. Card itself: **NEW** `recommendation-card` | (1) "Why?" expandable rationale with evidence links; (2) confidence styles (bars / ring / % / words) and a calibration tooltip; (3) Accept with edit (inline tweak the parameters), Snooze, "Don't suggest again" |
| 11 | **Diff Table** · [#diff-table](https://www.beautifului.dev/#diff-table) · `diff-table` (dep button) | AI-proposed edits sweeping through tabular data | phase 0 plain → 1 removals marked → 2 completed diff (plays once, then rests); per-row included / excluded; applied | `rows` ({key, id, dept, email, removed}) | Edit "sweeps" through the table; **each changed row is its own toggle** (`role=checkbox`, click / Enter to include or exclude); footer summary follows the row selection ("Click changed rows to toggle"); added row (Pistachio) tinted | Partial: `genui-blocks.md` `genui-table/changed-cells` (Pro) + `diff-viewer.md`. Row-toggle table: **NEW** `diff-table` | (1) Cell-level diffs (old → new inline) and column-change diffs; (2) bulk "accept all adds / reject all removes"; (3) conflict state when the data changed under the proposal |
| 19 | **Fine-tune Card** · [#fine-tune-card](https://www.beautifului.dev/#fine-tune-card) · `fine-tune-card` (dep glide-menu) | The agent adjusts design properties in a compact inspector | "Adjust" (agent working, shimmering accent label) → "Edited ✓" (pop-in) after a user edit; Type menu open / closed | `fields` ({key, label, value, min, max, step, suffix}; default W, H, Radius, Opacity %), `options` (Type: Seasonal / Classic / Limited), `labels` {title, layout, type, placeholder, adjust, edited}, `onChange({segment, values, type})` | Layout segmented control (row / col / grid) with a raised thumb; **number fields scrub**: hover the label for an ↔ cursor and drag, ↑ / ↓ (⇧ = ×10), or type; `role=slider` with aria values | Partial: `docs/studio.md` (inspector) + `categories/app-builder.md` `style-inspector` (Pro, no spec). Agent-driven inspector card: **NEW** | (1) Agent edits animate the values with a "by agent" marker, plus per-field revert; (2) field kinds: colour, shadow, easing curve, spring, font; (3) Before / After toggle + "copy as Rust props" |
| 20 | **Selection Actions** · [#selection-actions](https://www.beautifului.dev/#selection-actions) · `selection-actions` (deps button, shimmer, stream-text; npm iconoir-react) | Highlight a passage and hand it to the agent to rewrite | idle → thinking → streaming → result (Keep / Discard / Try again); more-actions collapsed / expanded; describe-edit input | `text` {lead, original, rewrite}, `actions` {primary[], more[]} ({id, icon, action, busyLabel}); defaults Explain, Improve + Shorten, Tone, Grammar; `labels` {keep, discard, placeholder}; `onAction` | Bar attaches beneath the last selected line, centred; 36 px pill wrapping 28 px controls; busy label shimmers ("Improving", "Changing tone"); rewrite streams in with blur edge + caret; "Describe edits" free-text with send; expand / collapse more actions | **NEW** (no text-selection spec; `app-builder.md` `element-picker` is for UI elements, not text) | (1) Inline diff of the rewrite (strike / insert) vs replace-in-place; (2) custom action presets + per-action model / effort; (3) multi-selection and "apply to all similar" |

## D. Data & knowledge blocks

| # | Name · URL · id | Purpose | States | Controls / props / variants | Micro-interactions | Oxyn spec | Oxyn plus |
|---|---|---|---|---|---|---|---|
| 10 | **Context Cards** · [#context-cards](https://www.beautifului.dev/#context-cards) · `context-cards` | Retrieved knowledge chunks with their sources | enter once, then remain | `chunks` ({title, chars, body, source, badge PDF/CSV, tone}), `labels` {header "All chunks", count "32"} | Cards enter once; char count per chunk; file-type badge; source link ↗ | Partial: `citations-sources.md` (`source-list`, `memory-receipt`). Chunk card: **NEW** `context-chunks` | (1) Relevance score bar + "used in answer" marker; (2) pin / exclude a chunk from context, with a token cost; (3) group by source with expand, plus highlight of the matched span |
| 12 | **Records Table** · [#records-table](https://www.beautifului.dev/#records-table) · `records-table` (+ `records-table.css`, dep glide-menu) | AI spreadsheet / CRM grid where columns are *properties* | idle; sorted asc / desc; column resizing; property popover open; new-property menu; AI column calculating (rows resolve one by one, pulse) → filled; rows selected; table options menu | `rows` ({id, name, tags[], last, strength strong/weak/veryweak/none, website}), `fill`. Built-in: sort by name / last / strength; resize every column; **pin / unpin column**; select all / per-row checkboxes; tag overflow "+3"; strength dot + label; links; **property config popover** (type, tool Model / Web search / User input, grounding + "About grounding", inputs, @-chip prompt, "Allow empty results", "Required value", "Show confidence", More settings); **+ New property** with types Text, File, Collection, Single select, Multi select, URL, Reference, JSON, File splitter; Table options (Compact columns, Reset column widths); footer: "60 count", "Add calculation" (decorative, no menu), "55 % average", "48 links" | Header click opens config, arrow sorts; adding the AI column scrolls it into view and opens its config; staggered row resolution with `records-pulse`; sticky first column; tag colours from one base hue via `color-mix` (light / dark safe); data cells are kept silent (no click sound) | Partial: `genui-blocks.md` `genui-table/data-grid`, `calc-footer` (Pro), `stream-rows` (Pro). AI property column + config popover: **NEW** `ai-column` | (1) Working "Add calculation" (sum / avg / median / % empty / unique) per column; (2) per-cell confidence + source on hover, plus re-run cell / column; (3) views: board, gallery, grouped; saved filters |
| 13 | **Filter Table** · [#filter-table](https://www.beautifului.dev/#filter-table) · `filter-table` | Status chips that reorganize live data | filter All / To do / In Progress / Completed | `rows` ({task, date, status todo/progress/done, owner}), `labels.columns` | Chips with counts and coloured dots (`aria-pressed`); table reorganises live; status pills tinted by `color-mix`; scrollable region | Partial: `genui-table/plain` (no filter-chip variant) | (1) Multi-select chips + search + sort; (2) animated row reflow (FLIP) when the filter changes; (3) agent-applied filter shown as an "AI filter" chip with its natural-language query |
| 16 | **Flowchart** · [#flowchart](https://www.beautifului.dev/#flowchart) · `flowchart` | Workflow trigger and condition steps on a dotted canvas | node idle / selected / dragging; condition dropdown open | `steps` ({id, row, x 0–1, w, kind {label, hue}, title, caption, condition}) | Cards drag anywhere (clamped to the canvas), and the **measured connector follows**; a drag does not also toggle selection; If / Else chips open real dropdowns (same menu as the model picker); outside click closes | **NEW** `workflow-canvas` (no spec; `plan-todo` step timeline is linear) | (1) Add-step "+" on connectors, plus branch / loop / parallel nodes; (2) live run overlay (the active node glows, edges animate, failures turn red); (3) zoom / pan / minimap + auto-layout |
| 17 | **Insight Cards** · [#insight-cards](https://www.beautifului.dev/#insight-cards) · `insight-cards` (npm `liveline`) | Paged agent insights with scrub-ready live charts | page k of n; autoplay → user-controlled; metric Spend / Usage | `pages` ({key, prose, Card, pill}); page types: **return comparison** (2 series, legend, big deltas, line chart), **anomaly** (bars with threshold, Spend / Usage toggle), **allocation** (hero number + segmented bar + legend); `labels.title` | "Insights 3 ‹ ›" pager; **autoplay yields as soon as a person uses it**; blurred crossfade between pages; Catmull-Rom smoothed series; inline @entity mentions and code-styled deltas (`-$2,453.44`); follow-up pill ("Should I rebalance flavors?") | Partial: `genui-blocks.md` (`genui-chart`, `genui-metric`). Carousel: **NEW** `insight-carousel` | (1) Scrub with a tooltip + "explain this point" asking the agent; (2) more page types (funnel, cohort, forecast band, map); (3) pin insight to a dashboard / schedule a recheck |

## E. Navigation & workspace

| # | Name · URL · id | Purpose | States | Controls / props / variants | Micro-interactions | Oxyn spec | Oxyn plus |
|---|---|---|---|---|---|---|---|
| 14 | **Sidebar Nav** · [#sidebar-nav](https://www.beautifului.dev/#sidebar-nav) · `sidebar-nav` (+ css, dep glide-menu, npm **paid** `@central-icons-react`) | Workspace switcher, primary nav, searchable chat history, collapse | expanded / collapsed (icon rail); workspace menu open; chat search open / empty ("No chats found"); active chat; active nav | `activeTitle`, `fill`, `onNewChat`, `onPick(id, label, prompt)`, `activeNav`, `onNavigate`, `footerLabel` ("Upgrade" / "Fork this"), `footerIcon`, `onFooterClick`, `recents` ({id, label, prompt}). Items: workspace "Creamery Ops" (New workspace, Workspace settings, Sign out), New chat, Home, Invite users 3/10 | **Gliding hover highlight**; search field grows right → left from the search control while the "Chats" label fades; Esc closes search; collapse keeps icons aligned | Partial: `workspace-chrome.md` (rails, collapse) + `side-panels.md`. Thread-history sidebar: **NEW** `thread-sidebar` | (1) Chat groups (Today / Pinned / by project) + rename / pin / archive on hover; (2) running-agent badges per chat (spinner, needs-approval dot); (3) resizable width + snap-to-rail with a label cascade |
| 15 | **Search** · [#search](https://www.beautifului.dev/#search) · `search` (dep glide-menu) | Command search with live filtering and an empty state | empty query; filtering; no results ("No results found / Adjust your search to try again"); cleared | `items` (strings), `labels` {placeholder, ariaLabel, emptyTitle, emptyHint} | Live filter; clear × button; gliding row highlight | Existing: `command-palette.md` | (1) Fuzzy match highlight + recent / suggested sections; (2) "Ask AI" row that turns the query into a prompt; (3) scoped tokens (`in:chats`, `type:file`) |
| 21 | **Agent Screen** · [#agent-screen](https://www.beautifului.dev/#agent-screen) · not in registry (repo `AgentScreen.tsx`, dep button) | Watch an agent's screen: open, teach a task, record | `variant` **Working** / **Loading** ("Connecting to agent's screen", ring spinner + skeleton); card → hover "Open" → full viewer (modal portal, Esc closes, body scroll locked); recording (red REC badge, mm:ss timer, End); collapsed while recording (keeps running) | `streamSrc` (image or video URL; a placeholder by default), `variant` | Card `fade-up` 380 ms; hover raises the shadow; animated faux cursor; viewer sized `min(960px, 90vw)` | Partial: `takeover-banner.md`, `acting-indicator.md`, `run-replay.md`. Live viewer: **NEW** `agent-screen` | (1) Take over / hand back control inside the viewer; (2) action overlay (click ripples, typed-text captions, step list beside the stream); (3) PiP / floating HUD mode + record-to-replay |

## F. Atoms & building blocks (registry `ui` items and site-only atoms)

| Name · id | Purpose | Controls / variants | Micro-interactions | Oxyn spec | Oxyn plus |
|---|---|---|---|---|---|
| **Button** · `button` (cva) | Shared pill buttons | `variant`: primary, secondary, ghost, accent, success, quiet; `size`: xs (28 px), sm (27 px), md; disabled | `active:scale-[0.96]`, 150 ms colour / opacity | **NEW** atom (tokens exist in `tokens.md`; no button spec) | Loading-with-spinner + destructive-confirm (hold to confirm) + icon-only sizes |
| **GlideMenu** · `glide-menu` | Menu wrapper with one gliding highlight that follows the hovered row | `className`, `highlightClassName`, `rowSelector` | Single highlight moves between rows instead of per-row hover | Partial: `motion.md` (slash / tools-menu highlight glide spring) | Spring presets; keyboard-follow; vertical + horizontal (tabs) |
| **EntityChip** + **Monogram** · `entity-chip` | Inline entity (company / person) with a coloured monogram | `name`, `color`, `monogram` | — | Partial: `composer-slash-mentions.md` (mention chips) | Hover card; avatar image; status dot |
| **ValuePill** · `value-pill` | Small value badge | `tone`: neutral, green, orange, red, accent | — | Partial: `status-indicator.md` | Delta arrows; number roll on change |
| **StatusPill, Chip, TextRow** (repo only, used in Foundations / harness; not in registry) | Status pill, generic chip, label / value row | — | — | Partial (`status-indicator.md`) | — |

## G. Effects & motion (cross-cutting)

| Effect | Where used | Detail | Oxyn spec | Oxyn plus |
|---|---|---|---|---|
| Pixel wavefront (`pixel-on`) | Loading State | Per-cell delay maps: chevron (Drive / Dots), perimeter comet (Orbit) | `loader-glyph.md` / `loader-vector.md` | Path editor in Studio; GPU-drawn large grids |
| Text shimmer (`shimmer-text`) | Loading label, Fine-tune "Adjust", busy labels | Gradient sweep | `text-shimmer.md` | — (already richer) |
| Blur-resolve streaming + caret | StreamText, Streaming Text, Selection Actions | Blurred trailing chars; caret solid → blink | `text-reveal.md` | — |
| Odometer counter | Approval Card step "1 / 3" | Per-digit roll up / down | `motion.md` C16 `dur.digit.roll`, `composer-context-meter.md` | — |
| Gliding highlight | GlideMenu, PromptBar menus, Sidebar, Search | One shared highlight element | `motion.md` (menu highlight spring) | — |
| **Glimm rainbow shader sweep** | Prompt Bar model upgrade | WebGL shader (npm `glimm`) with pinned hue phase, recreated per sweep | **NEW** (closest: `border-beam.md`) | GPUI-native shader; also on "Pro model" pick |
| Height-animating stack + vertical slide | Approval Card | Card height fits the active question; first paint mounts only the active question to avoid a flash | `genui-forms.md` | — |
| Row pulse while calculating (`records-pulse`) | Records Table AI column | 1.1 s ease-in-out loop | `skeletons.md` | — |
| `pop-in`, `fade-up`, `fade-in`, `spin`, `eq-bounce`, `caret-blink` | Many | Keyframes in `foundation.css` | `motion.md` | — |
| Home reveal sequence | Harness empty state | Blur 17 px + 23 px rise, 800 ms `cubic-bezier(0.16,1,0.3,1)`, staged hello → question → prompt → suggestions | `motion.md` / `text-reveal.md` | Tunable in Studio |
| **Interaction sounds** | Whole site (`InteractionSounds.tsx`, `@web-kits/audio`) | 5 synthesized cues: press, tick, release (close / delete words), page (links), pulse (primary words: send / save / create…); `data-sound` override, `data-sound-silent` regions; master volume 0.32; off via `localStorage bui-sounds=off` (**no visible toggle**) | **NEW** (research mentions Cuelume; no spec) | Agent-event cues (thinking start, tool done, needs approval, error) + visible mute / volume + reduced-sound mode |
| Reduced motion | Approval Card, Prompt Bar, Loading State (+ CSS fallbacks) | Freezes loops, keeps timers | `loaders.md` §2.6, `motion.md` | Oxyn covers every component, not only some |

## H. Site features

| Feature | What Beautiful UI does | Oxyn spec | Oxyn plus |
|---|---|---|---|
| **Navigation** | One long page; left "Components" nav with 21 numbered anchors (01–21); logo; "Built by Turbo" card + "Book a call" (cal.com) | `docs/site/ia.md` §3–4 (multi-page site map) | Already exceeds: family / component / variant pages |
| **Docs layout** | None on site. Each section = number, title, one-line caption, live demo, variant switcher. Docs live in the GitHub README (install, foundation setup, wiring the harness to a real agent, env vars) | `ia.md` §7.3–7.8 (component pages, Starlight docs) | Per-component "wire to your agent" recipes (Beautiful UI has one table in its README) |
| **Live preview** | Every demo is live and interactive; most autoplay once and then rest | `site/gallery.md` §6 (card live policy), `ia.md` §9 | State pinning (gallery §3.4), reduced-motion toggle |
| **Controls** | Inline variant switcher under demos that have variants (Loading, Thinking, Task Rows, Prompt Bar, Code Block, Agent Screen). No props panel | `studio.md`, `ia.md` §7.4 | Full Studio inspector + state scrubber on every demo |
| **Code** | Hover a card → **Copy code** and **View code** buttons. View code opens a modal (Esc, scroll lock) with the file, a "Requires · also copy X · `npm i …`" line (or "Self-contained — needs only the foundation tokens"), and a `$ npx shadcn add …/r/<id>.json` line | `ia.md` §10 (install tabs) | Tabs: CLI / source / prompt / MCP; Rust + Dioxus + GPUI snippets |
| **Install** | shadcn registry: `npx shadcn add https://www.beautifului.dev/r/<id>.json`; index at `/r/registry.json`; `foundation.json` (registry:style) pulled as a dependency; per-component CSS (`records-table.css`, `sidebar-nav.css`) | PR #24 `feat/cli-registry`, `ia.md` §10 | `oxyn add`, plus a Cargo feature per component |
| **Theme** | Light / Dark segmented toggle (sun / moon) at the top; `.dark` token set; theme sync | `ia.md` (dark → light → hc-dark), `tokens.md` | High-contrast theme + live token editor |
| **Foundations** | Repo has a "Foundations" section (Type, Color, Shadow, Spacing & radius, Buttons, Text rows). I did not see it in the live index text | `tokens.md` | Tokens page with copy-per-token |
| **Search** | **None.** No site search and no ⌘K, only the anchor list | `ia.md` §6 (Pagefind ⌘K), `gallery.md` filters | Already exceeds |
| **Pricing** | **None.** Everything is free MIT; the CTA is the studio's consulting ("Book a call") | `ia.md` §7.7, PR #29 | n/a (Oxyn has a Pro tier) |
| **Harness** | `/harness` "Ice Cream Harness": a full agent app built from the primitives. Sidebar, greeting "Hello Shane", model chip, 3 suggestions + "Shuffle suggestions", "Connect your apps…". About 10 scripted scenarios (parking appeal with a browser pane, to-dos with a Tasks pane, workload, off-boarding approval, find-ticket with a Context pane, supplier records as a spreadsheet workspace with the chat docked right, restock ToolChips, flavor DiffTable, rewrite SelectionActions, Surfer loader). Right-hand artifact pane, full-bleed answers | `docs/categories/*.md` shells (PR #27 / #28) | One harness per category kit, plus a "Bring your own agent" adapter (AG-UI / ACP) |
| **Fork this** | Harness footer button opens a "Fork this harness" modal: **Copy instructions** (a ready-to-paste prompt for your coding agent), Copy clone command, repo link | `ia.md` §10 (prompt / MCP install) | Ship it as an MCP tool + `llms.txt` |
| **Newsletter** | Footer "New components, in your inbox" (email, Notify me, Joining state). **Email nudge** modal opens once after more than 10 clicks, stored in localStorage | `ia.md` §5 footer (no nudge) | — (optional) |
| **Licence page** | `/license` "Yes, you can use it for free." + the MIT text | `docs/licensing.md` | — |
| **Sounds** | Site-wide UI sounds, on by default (see G) | **NEW** | Visible toggle |
| **Analytics** | PostHog (optional env) | `ia.md` §13 (cookieless) | — |
| **Demo domain** | All demos share one fictional business ("Creamery Ops") | `docs/categories/*` (one fictional company per kit) | — |

---

## Summary

**Inventory size: 56 rows.**

| Group | Rows |
|---|---|
| Gallery components (tables A–E) | 21 |
| Atoms: Shimmer and StreamText (table A), plus Button, GlideMenu, EntityChip/Monogram, ValuePill and the repo-only StatusPill/Chip/TextRow (table F) | 7 |
| Effects (table G) | 11 |
| Site features (table H) | 17 |

All items are **free + MIT**. The one paid dependency is the Central Icons set used by Sidebar Nav.

**Coverage of the 21 gallery components:**

| Status | Count | Components |
|---|---|---|
| Existing spec covers it | **9** | Loading State (except the Surfer variant), Thinking, Streaming Text, Approval Card (`genui-question/paged`), Tool Chips, Task Rows, Prompt Bar (except the glimm sweep), Code Block, Search |
| Partial (spec exists, distinctive part missing) | **9** | Chat (tabs), Recommendation Card, Diff Table, Fine-tune Card, Context Cards, Records Table (AI column), Filter Table (chips), Insight Cards (carousel), Sidebar Nav (thread history) |
| NEW (no spec) | **3** | Selection Actions, Flowchart, Agent Screen |

So **9 are covered and 12 need new spec work** (9 partial + 3 NEW).

**Atoms and effects:** 1 NEW atom spec (Button). NEW effects: glimm shader sweep and interaction sounds. Everything else maps to `text-reveal`, `text-shimmer`, `motion.md`, `loaders`.

**Site features:**
- Oxyn's site plan already **exceeds** Beautiful UI on search, pricing, docs, theme (hc-dark), install tabs and the Studio controls.
- Gaps to adopt:
  - Copy and View code on hover, with a "Requires" dependency line
  - "Fork this" copy-prompt-for-your-agent modal
  - A harness with a right-hand artifact pane and a spreadsheet workspace mode
  - A visible sound toggle (sounds themselves are NEW)

### NEW and partial work, grouped into PR-sized batches
These slot after PR #20–#23 in `docs/pr-plan.md`; numbers are suggestions. Each batch is a specs-first PR, then a feat PR.

1. **Batch A: HITL extras** (specs + feat)
   - `recommendation-card` (NEW): confidence meter, alternatives drawer, promote, accept
   - `selection-actions` (NEW): text-selection bar, idle → thinking → streaming → result, Keep / Discard / Retry, describe-edit
   - Add the agent-driven `fine-tune-card` inspector (NEW spec; reuse the Studio scrub fields)
2. **Batch B: Data blocks I** (extends `genui-blocks.md`)
   - `diff-table` (row-toggle proposed edits)
   - `genui-table/filter-chips` (chips with counts + live reflow)
   - `context-chunks` (RAG chunk cards: char count, file badge, pin / exclude)
3. **Batch C: Data blocks II**
   - `ai-column` for the records grid: property popover (type / tool / grounding / inputs / @-prompt), + New property with 9 types, row-by-row resolve, pin / resize / compact / reset, working calc footer
   - `insight-carousel`: paged insights, autoplay-yields, compare / anomaly / allocation page types, scrubbable charts
4. **Batch D: Workspace**
   - `thread-sidebar`: workspace switcher, searchable chat history with grow-in search, collapse-to-rail, seat counter, footer CTA
   - Chat thread tabs (extends `messages.md`)
5. **Batch E: Agent screen & workflow**
   - `agent-screen`: card → viewer modal, Working / Loading, Teach-a-task recording with REC badge and timer, stream source
   - `workflow-canvas`: dotted canvas, draggable nodes, measured connectors, trigger + if / else chips with dropdowns
6. **Batch F: Effects & atoms**
   - `button` atom (6 variants × 3 sizes)
   - `glide-menu` primitive
   - `model-change-sweep` effect (glimm-style)
   - `loading-state/surfer` companion slot
   - `oxyn-sound` cue set: press / tick / release / page / pulse + agent-event cues, mute toggle, `data-sound-silent`
7. **Batch G: Site parity** (folds into PR #25 site-gallery)
   - Hover Copy / View code with a Requires line
   - Variant switcher under each demo
   - Harness with artifact pane + spreadsheet-workspace mode
   - "Fork this" copy-prompt modal
   - Sounds toggle

### Notes and judgment calls
- **Licence:** MIT allows porting with attribution. Oxyn is Rust, so these are reimplementations; keep the attribution line in each spec's references.
- **Not ported (paid icons):** Central Icons, used by Sidebar Nav, is commercial and must not be ported.
- **Not ported (dev-only tool):** `DevToolbar` (agentation) only runs in development and is out of scope.
- **Foundations section:** I found it in the repo but not in the live index text, so it is marked "repo-only".
- **"Add calculation" in Records Table** is a decorative button with no menu. Oxyn plus makes it real.
- **Raw material:** source files read for this inventory are in `/workspace/work/bui/` (registry JSON, extracted `.tsx`/`.css` in `src/`, repo files in `gh/`). That folder is scratch.
