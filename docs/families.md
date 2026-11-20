# Oxyn component families (sidebar categories)

Generated 2026-11-16 for PR #39 (variants 1); PR #40 (variants 2) added the Storage and Menu Motion families and built variants in Context Visualizer and Attachments. Every existing component sits in exactly one family; each family will hold many variants. The site sidebar groups components by these families (`site/src/lib/families.mjs`).

38 families, 268 existing components before variants (PR #39 added 8, PR #40 added 17).

## Composer (`composer`)
The prompt box itself, the field and what sits in it.

**Existing:** Composer (`composer`), Chat composer (`chat-composer`), AI composer (`ai-composer`), Rich text editor (`rich-text-editor`), Mention input (`mention-input`), Mode switch (`mode-switch`), Run mode menu (`run-mode-menu`), Tools menu (`tools-menu`), Queue stack (`queue-stack`), Agent steering (`agent-steering`), Starter prompts (`starter-prompts`), Writing Assistant (`writing-assistant`), Ghost text (`ghost-text`)

**New variants to build:**
- Split-intent composer: one field, a hairline divides "what" from "constraints"; Tab jumps across
- Draft deck: earlier unsent drafts stack behind the field like cards, swipe up to restore
- Prompt lint: quiet underline hints (vague verb, missing file) that fix in one click
- Margin notes: inline @-chips grow a tiny gutter label of what each mention resolves to
- Focus composer: field expands to a centred sheet with a word budget ring

## Send & Run Controls (`send-run`)
Send, stop, queue and steer: the button that changes with the run.

**Existing:** Send control (`send-control`), Run controls (`run-controls`), Action swap (`action-swap`)

**New variants to build:**
- Breath button: send arrow that slowly inhales while the agent is busy, stop on press
- Fuse stop: hold to stop draws a burning fuse around the button edge
- Queue ticket: sending while busy prints a numbered ticket that tears off into the queue
- Steer dial: press-and-drag the send button sideways to pick Send / Queue / Steer

## Model Picker (`model-picker`)
Choosing the model: pills, menus and full pickers.

**Existing:** Model picker (`model-picker`), Model menu (`model-menu`), Model pill (`model-pill`)

**New variants to build:**
- Constellation picker: models as dots on speed × quality plane; drag a lasso
- Model deck: horizontal card carousel with spec rows that align across cards
- Recent-first pill with a ghost of the last two models behind it
- Compare-on-hover: hovering two rows shows a diff of their specs

## Thinking Selector (`thinking-selector`)
How hard the model thinks: rulers, sliders, menus and pickers.

**Existing:** Effort control (`effort-control`), Ruler card (`effort-ruler`), Reasoning panel (`reasoning-panel`)

**New variants to build:**
- Ruler card (built)
- Reasoning panel with notched slider, Fast mode, Favorite (built)
- Effort list menu with check + trigger chip
- Effort pill + popover with step dots
- Context window + Effort grouped dropdown
- Model list with effort suffix + cascading submenu
- Nozzle line (user design): even thin lines in a steel-blue → coral → periwinkle gradient, a solid nozzle in the colour under it slides to select (tone red keeps the all-red look)
- Effort stairs: ascending bars, budget + latency estimate per step
- Thought dial: a knob whose ring fills with tick marks; scroll to turn
- Budget dial in tokens: pick a token cap, effort label follows

## Context Visualizer (`context-visualizer`)
How full the context window is and what fills it.

**Existing:** Context meter (`context-meter`), Context window (cells) (`context-field`), Context window (grid) (`context-grid`), Context Map (`context-map`), Context cards (`context-cards`)

**Built in #40 (variants 2):** Context bar (`context-bar`), Context breakdown (`context-breakdown`), Context stack (`context-stack`), Context tide (`context-tide`)

**New variants to build:**
- Context meter ring with hover legend
- Shelf view: each file a book on a shelf, width = tokens; drag off to drop
- Heat strip of the transcript showing which turns weigh most
- Pressure bar that blinks before auto-compact with a countdown

## Attachments (`attachments`)
Files, images and links going into a prompt.

**Existing:** Attachments (`attachments`), Attachment strip (`attachment-strip`), File dropzone (`file-dropzone`), Avatar upload (`avatar-upload`), Link unfurl (`link-unfurl`), Document extraction (`document-extraction`)

**Built in #40 (variants 2):** Upload dropzone (`upload-dropzone`), Attachment chips (`attachment-chips`), Polaroid stack (`polaroid-stack`), Drop lanes (`drop-lanes`)

**New variants to build:**
- Paper clip tray: files dangle from a clip on the composer edge
- Inline chips that expand into a mini preview sheet
- Storage scanner style: sunburst of what you attach most

## Storage (`storage`)
Disk and context usage drawn as dot matrices: capacity, cleanup, waffle and forecast.

**Existing:** _none yet_

**Built in #40 (variants 2):** Storage meter (`storage-meter`), Storage cleanup (`storage-cleanup`), Storage waffle (`storage-waffle`), Storage forecast (`storage-forecast`)

**New variants to build:**
- Setup map: where agent skills, rules and servers live, folder by folder, with counts
- Leftovers sweep: old sessions, logs and build output an agent left behind, grouped by age
- Notch tabs: Now / Skills / Storage / Cleanup in one compact panel

## Menu Motion (`menu-motion`)
Menus and controls that move like liquid: the gooey menu motion on attach and model menus, a composer, tabs and drops.

**Existing:** _none yet_

**Built in #40 (variants 2):** Gooey composer (`gooey-composer`), Gooey attach menu (`gooey-attach-menu`), Gooey model menu (`gooey-model-menu`), Liquid tabs (`liquid-tabs`), Liquid drops (`liquid-drops`)

The gooey motion is a shared Oxyn menu motion (`gooeyfid.css`, `oxgm-*`): any menu can take it (see the spec).

**New variants to build:**
- Gooey context menu at the pointer
- Melt switch: the knob stretches before it lands
- Approval menu whose Full access row bleeds red into the button

## Voice (`voice`)
Speaking to the agent: record, dictate, listen.

**Existing:** Voice recorder (`voice-recorder`), Dictation button (`dictation-button`)

**New variants to build:**
- Waveform pill that collapses to a dot on pause
- Push-to-talk orb with breath ring
- Live captions ribbon under the composer
- Voice memo cards with scrub and transcribe

## Approval & Permissions (`approval`)
The agent asks before it acts; how much it may do alone.

**Existing:** Approval card (`approval-card`), Approval pager (`approval-pager`), Decision card (`decision-card`), Approval questions (`approval-questions`), Hold to confirm (`hold-to-confirm`), Confirm morph (`confirm-morph`), Confirm popover (`confirm-popover`), Approval gates (`approval-gates`), Permission request (`permission-request`), Agent approvals (`agent-approvals`), Permission selector (`permission-selector`), Autonomy Dial (`autonomy-dial`), Tool Access Map (`tool-access-map`)

**New variants to build:**
- Gate turnstile: approvals stack like turnstile arms that rotate away
- Risk thermometer card
- Batch approve with swipe lanes (allow left, deny right)
- Signature approval: hold-and-draw a check to approve destructive actions
- Timeout approval: auto-deny ring with countdown

## Messages & Answers (`messages`)
The conversation: bubbles, answers, streaming text and actions.

**Existing:** Message bubble (`message-bubble`), Chat thread (`chat-thread`), AI chat (`ai-chat`), AI Answer (`ai-answer`), Cited Answer (`cited-answer`), Message Actions (`message-actions`), Response feedback (`response-feedback`), Streaming markdown (`streaming-markdown`), Streaming text (`streaming-text`), Support conversation (`support-conversation`), Recommendation card (`recommendation-card`)

**New variants to build:**
- Margin-annotated answer: claims get side notes
- Collapsible answer outline (headings as chips)
- Diffable regenerate: compare two answers inline
- Message receipts: delivered, read by agent, acted on
- Quote-reply bubble with tail anchored to quoted span

## Reasoning (`reasoning`)
What the model thought: blocks, traces and live thinking states.

**Existing:** Reasoning block (`reasoning-block`), Thinking (`thinking-traces`), Thinking State (`thinking-state`), Thinking Loader (`thinking-loader`)

**New variants to build:**
- Thought river: steps flow as stones across a stream
- Timeline scrubber over reasoning steps
- Collapsed thinking chip that counts seconds and steps
- Confidence spark line through the reasoning

## Sources & Research (`sources`)
Citations, source cards and research runs.

**Existing:** Source cards (`source-cards`), Research run (`research-run`), Experiment grid (`experiment-grid`)

**New variants to build:**
- Source wall: favicons tile, hover lights the cited sentences
- Footnote drawer
- Trust meter per source
- Research map: query tree with found/used/ignored leaves

## Loaders (`loaders`)
One busy signal per region: glyphs, spinners and skeletons.

**Existing:** Glyph loader (`glyph-loader`), Spinner (`spinner`), Loading state (`loading-state`), Status indicator (`status-indicator`), Skeleton (`skeleton`), Skeleton morph (`skeleton-morph`)

**New variants to build:**
- Moth flap loader (brand)
- Dot matrix loader with token rain
- Orbit loader with N agents as moons
- Skeleton that types itself in

## Text Motion (`text-motion`)
Shimmering, cycling, revealing and counting text.

**Existing:** Text shimmer (`text-shimmer`), Text shimmer (`shimmer-label`), Verb cycler (`verb-cycler`), Text reveal (`text-reveal`), Text reveal (`word-reveal`), Text morph (`text-morph`), Slot text (`slot-text`), In-view title (`in-view-title`), Elapsed timer (`elapsed-timer`), Number ticker (`number-ticker`), Animated counter (`animated-counter`), Countdown (`countdown`)

**New variants to build:**
- Ink bleed reveal
- Typewriter with backspace corrections
- Scramble-to-word cycler
- Odometer elapsed timer

## Tool Calls (`tool-calls`)
What the agent called and what came back.

**Existing:** Tool call (`tool-call`), Tool stack (`tool-stack`), Tool call card (`tool-call-card`), Tool call (`tool-call-transcript`), Tool chips (`tool-chips`), Tool-kind map (`tool-kind-map`), JSON viewer (`json-viewer`)

**New variants to build:**
- Receipt roll: tool calls print as thermal receipt lines
- Tool call as terminal tab
- Call graph mini map
- Payload peek with JSON folding

## Plans & Tasks (`plans`)
Plans, to-dos and checklists, and how the run kept to them.

**Existing:** Todo list (`todo-list`), Plan card (`plan-card`), Agent plan (`agent-plan`), Task rows (`task-rows`), Task breakdown (`task-breakdown`), Stepper (`stepper`), Step timeline (`step-timeline`), Onboarding checklist (`onboarding-checklist`), Release readiness (`release-readiness`), Plan vs Actual (`plan-vs-actual`), Decision Tree (`decision-tree`)

**New variants to build:**
- Kanban plan with live progress
- Plan as metro line with stations
- Checklist with time budget per item
- Plan diff when the agent re-plans

## Run & Trace (`run-trace`)
One run end to end: status, trail, waterfall and root cause.

**Existing:** Agent run (`agent-run`), Agent trail (`agent-trail`), Trace Waterfall (`trace-waterfall`), Activity feed & timeline (`activity-feed`), Agent Sequence (`agent-sequence`), Root Cause Tree (`root-cause-tree`), Running Now (`running-now`), Agent status (`agent-status`), Agent Monitor (`agent-monitor`)

**New variants to build:**
- Flame graph of a run
- Replay scrubber with frame thumbnails
- Black box recorder: last N events before a failure
- Heartbeat strip of agent activity

## Multi-agent (`multi-agent`)
Many agents at once: trees, crews, lanes, fan-outs and judges.

**Existing:** Sub-agent tree (`subagent-tree`), Agent tree (`agent-tree`), Agent fleet (`agent-fleet`), Agent orchestrator (`agent-orchestrator`), Agent Crew (`agent-crew`), Agent Cursors (`agent-cursors`), Agent Graph (`agent-graph`), Agent Handoff (`agent-handoff`), Agent Lanes (`agent-lanes`), Delegate Board (`delegate-board`), Fan-out Grid (`fanout-grid`), Parallel Attempts (`parallel-attempts`), Judge Bracket (`judge-bracket`), Fallback Chain (`fallback-chain`), Collaboration Stream (`collaboration-stream`)

**New variants to build:**
- Air traffic board
- Swim-lane radar
- Tournament judge with vote animation
- Relay baton handoff animation
- Hive view: hex cells per agent

## Inbox & Recap (`inbox`)
What happened while you were away and what needs you now.

**Existing:** Agent Inbox (`agent-inbox`), Agent Queue (`agent-queue`), Agent Recap (`agent-recap`), Away Summary (`away-summary`), Agent Findings (`agent-findings`), Notification center (`notification-center`), Notification stack (`notification-stack`), Thread list (`thread-list`), Activity home (`activity-home`)

**New variants to build:**
- Morning brief card stack
- Triage swipe deck
- Digest timeline with "since you left" marker
- Priority radar

## Usage & Cost (`usage-cost`)
Tokens, money and limits: meters, charts and planners.

**Existing:** Usage meter (`usage-meter`), Token usage chart (`token-usage-chart`), Polling cost (`polling-cost`), Cost Waterfall (`cost-waterfall`), Agent Ledger (`agent-ledger`), Rate Limit Planner (`rate-limit-planner`), Gauge (`gauge`), Activity heat map (`activity-heatmap`), Split bar (`split-bar`)

**New variants to build:**
- Fuel gauge for monthly budget
- Cost receipt per run
- Token burn-down chart
- Limit planner with countdown to reset
- Per-model cost split donut

## Memory (`memory`)
What the agent remembers about you and the work.

**Existing:** Memory Panel (`memory-panel`)

**New variants to build:**
- Memory garden: facts as plants that wither when stale
- Memory ledger with source and expiry
- Forget slider
- Memory diff after a session
- Pinned facts board

## Diff & Review (`diff`)
Changes the agent made, line by line, and the review around them.

**Existing:** Diff viewer (`diff-viewer`), Inline diff (`inline-diff`), Diff table (`diff-table`), Diff timeline (`diff-timeline`), Change summary (`change-summary`), Code suggestion (`code-suggestion`), AI inline edit (`ai-inline-edit`), Review center (`review-center`), Review card (`review-card`), Image compare (`image-compare`)

**New variants to build:**
- Split/unified toggle with minimap
- Word-level prose diff
- Image diff with onion skin and swipe
- Review checklist per hunk
- Diff heat map per file

## Merge Conflict (`merge`)
Resolving two versions of the same change.

**Existing:** 3-way merge conflict (`merge-conflict`), Conflict Radar (`conflict-radar`)

**New variants to build:**
- Three-column braid view
- Conflict cards with pick-left / pick-right / both
- Auto-merge confidence badges

## Checkpoints (`checkpoints`)
Rewind a run to any point; branch new attempts from it.

**Existing:** Checkpoint lanes (`checkpoint-lanes`), Checkpoint tree (`checkpoint-tree`), Checkpoint film strip (`checkpoint-film-strip`), Checkpoint list (`checkpoint-list`), Checkpoint marker (`checkpoint-marker`), Undo ladder (`undo-ladder`), Restore preview sheet (`restore-preview`), Restore toast (`restore-toast`)

**Built in C1a (2026-11-20):** the tree, film strip, undo ladder and restore preview sheet, plus keyboard / accessibility / ISO-time parity for the four existing members.

## Comments & Reactions (`comments`)
Talking about the work: threads, panels, reactions.

**Existing:** Comment thread (`comment-thread`), Comments panel (`comments-panel`), Threaded comments (`threaded-comments`), Reactions (`reactions`), Reaction bar (`reaction-bar`)

**New variants to build:**
- Sticky notes on the canvas
- Threaded margin comments
- Reaction burst
- Resolve-by-agent suggestion

## Schedule & Time (`schedule`)
Dates, times and when agents run.

**Existing:** Scheduled Runs (`scheduled-runs`), Calendar (`calendar`), Date picker (`date-picker`), Date range picker (`date-range-picker`), Time picker (`time-picker`), Time dial (`time-dial`), Date reel (`date-reel`)

**New variants to build:**
- Week board of scheduled runs
- Cron builder with natural language
- Time dial for run windows
- Next-run countdown chip

## Theme Selector (`theme-selector`)
Background pictures and texture, theme swatches with font size, and originals: split compare, accent and density, schedule, type scale, light playground, app icon and display comfort.

**Existing:** _none yet_

**Built in #41 (variants 3):** Background picker (`background-picker`), Theme grid (`theme-grid`), Theme split (`theme-split`), Accent and density (`accent-density`), Theme schedule (`theme-schedule`), Type scale (`type-scale`), Light playground (`light-playground`), App icon (`app-icon-picker`), Display comfort (`display-comfort`)

**New variants to build:**
- Wallpaper from a prompt: describe a picture and add the result as a tile
- Per-workspace themes: a different theme for each workspace or project

## Buttons (`buttons`)
Buttons, icon buttons, split buttons and the groups they sit in.

**Existing:** Button (`button`), Icon button (`icon-button`), Action button (`action-button`), Split button (`split-button`), Button group (`button-group`), Floating button group (`floating-button-group`), Expanding button group (`expanding-button-group`), Copy button (`copy-button`)

**New variants to build:**
- Magnetic button
- Hold-to-confirm ring button
- Morphing icon button
- Split button with recent actions

## Fields (`fields`)
Typing things in: text, numbers, codes, passwords, tags.

**Existing:** Text field (`text-field`), Text area (`text-area`), Input (`labeled-input`), Textarea (`labeled-textarea`), Number field (`number-field`), Scrub input (`scrub-input`), Password field (`password-field`), Password strength (`password-strength`), OTP input (`otp-input`), Phone input (`phone-input`), Inline edit (`inline-edit`), Tag input (`tag-input`), Smart form fill (`smart-form-fill`), Stepper form (`stepper-form`), Multi-step form (`multi-step-form`)

**New variants to build:**
- Ghost-completion field
- Secret field with peek and copy
- Scrub number field with units
- OTP field with paste-fill animation

## Choice (`choice`)
Picking one or many: switches, radios, selects, chips and ratings.

**Existing:** Switch (`switch`), Checkbox (`checkbox`), Radio group (`radio-group`), Radio cards (`radio-cards`), Segmented control (`segmented-control`), Toggle group (`toggle-group`), Chip group (`chip-group`), Rating (`rating`), Select (`select`), Morph select (`morph-select`), Combobox (`combobox`), Multi-select (`multi-select`), Filter menu (`filter-menu`), Reorderable list (`reorderable-list`)

**New variants to build:**
- Segmented control with sliding pill and counts
- Chip group with overflow +N
- Rating as signal bars
- Reorder list with drop shadow lanes

## Sliders (`sliders`)
Dragging a value: plain, elastic and scrubbed.

**Existing:** Slider (`slider`), Elastic slider (`elastic-slider`)

**New variants to build:**
- Range slider with histogram
- Snap slider with haptic ticks
- Elastic slider with overshoot
- Dual-thumb budget slider

## Data Display (`display`)
Badges, chips, avatars, keys, code and empty states.

**Existing:** Badge (`badge`), Chip (`chip`), Value pill (`value-pill`), Entity chip (`entity-chip`), Avatar (`avatar`), Kbd (`kbd`), Separator (`separator`), Empty state (`empty-state`), Empty states (`empty-states`), Code block (`code-block`), Tree table (`tree-table`)

**New variants to build:**
- Entity chip with hover card
- Kbd combo renderer
- Value pill with trend arrow
- Avatar stack with live presence

## Overlays (`overlays`)
Tooltips, popovers, menus, dialogs, sheets and drawers.

**Existing:** Tooltip (`tooltip`), Popover (`popover`), Dropdown menu (`dropdown-menu`), Hover card (`hover-card`), Dialog (`dialog`), Sheet (`sheet`), Sheet stack (`sheet-stack`), Bottom sheet (`bottom-sheet`), Drawer (`drawer`), Control center (`control-center`)

**New variants to build:**
- Peek sheet
- Command sheet
- Popover with arrow that follows the pointer
- Stacked sheets

## Toasts & Alerts (`feedback`)
Telling you something happened.

**Existing:** Toaster (`toaster`), Alert (`alert`)

**New variants to build:**
- Toast with undo countdown
- Inline alert that becomes a toast
- Progress toast for long runs

## Search & Commands (`search`)
Palettes, AI search, settings search and shortcuts.

**Existing:** Command palette (`command-palette`), AI command (`ai-command`), AI search bar (`ai-search-bar`), Settings command (`settings-command`), Search field (`search-field`), Expanding search (`expanding-search`), Shortcut recorder (`shortcut-recorder`), Pagination (`pagination`)

**New variants to build:**
- AI search with answer preview
- Command palette with scopes
- Settings search that jumps and highlights
- Shortcut recorder with conflicts

## Navigation & Layout (`navigation`)
Tabs, breadcrumbs, accordions, panes, trees and docks.

**Existing:** Tabs (`tabs`), Breadcrumb (`breadcrumb`), Accordion (`accordion`), Resizable panels (`resizable-panels`), Tree view (`tree-view`), Table of contents progress (`toc-progress`), Page header (`page-header`), Dock (`dock`), Expandable text (`expandable-text`), Sidebar Nav (`sidebar-nav`), Terminal tabs (`terminal-tabs`), Scroll area (`scroll-area`), Expandable card (`expandable-card`), Card (`card`)

**New variants to build:**
- Sticky ToC progress
- Dock with magnification
- Resizable panes with snap
- Tabs with overflow menu

## Screens (`screens`)
Whole app frames built from the components (affectUI's screen set by behaviour, in Oxyn's own words and greys). Every screen has Sample / Loading / Empty data, an Inset or Flat frame, and light / dark. Pages live at `/screens/<slug>/` with a full-viewport view.

**Built in #42 (screens 1):** App shell · Inset (`shell-inset`), App shell · Flat (`shell-flat`), Three-pane shell (`shell-three-pane`), AI Chat · Start (`chat-start`), AI Chat · Overview (`chat-overview`), AI Chat · Agents (`chat-agents`), AI Chat · Double sidebar (`chat-double-sidebar`), AI Chat · Progress (`chat-progress`), AI Chat · Run (`chat-run`), Folders · Overview (`folders-overview`), Folders · Detail (`folders-detail`), Integrations (`integrations`), Kanban (`kanban`), CRM (`crm`), Workflow builder (`workflow`), Settings shell (`settings`)

**Built in screens 2 (agent screens):** Agent · Orchestration (`agent-orchestration`), Agent · Thread (`agent-thread`)

**New variants to build:**
- Agent screens still to build (Screens 2): Inbox, Inbox board, Agents, Run detail, Tasks, Memory, Permissions, Usage, Activity, Schedules, Skills, Analytics, Evals; Settings pages and Onboarding, coding-agent apps (later screens PRs)

## App Shell (`app-shell`)
Sidebars, rails, side panels and account chrome.

**Existing:** Sidebar (`sidebar`), Workspace sidebar (`workspace-sidebar`), Sidebar rail (`sidebar-rail`), Inbox sidebar (`inbox-sidebar`), App shell (`app-shell`), AI side panel (`ai-side-panel`), Harness (`harness`), AI Chat screens (`ai-chat-screens`), Document grid (`doc-grid`), User menu (`user-menu`), Team switcher (`team-switcher`)

**New variants to build:**
- Claude-Code-like terminal agent screen
- Codex-like task board screen
- DeepSeek-like chat screen
- Qwen/Z.ai code screens
- Agent workspace with three panes
- Storage scanner screen (sunburst)
