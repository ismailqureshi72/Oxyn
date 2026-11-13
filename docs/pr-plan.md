# PR plan

One PR per row, merged in order with **Create a merge commit**. Authors alternate; the other maintainer
is `Co-authored-by` on every commit. Agreed 2026-11-11.

| PR | Branch | Delivers | Phase | Author | Status |
|---|---|---|---|---|---|
| #1 | docs/baseline | README, docs, spec standard, roadmap, brand | V1 | Both | ✅ merged |
| #2 | research/sources | Reference catalog (141 refs, IDs, CSV) + this plan | V2.1 | Khubaib | ✅ merged |
| #3 | specs/composer | All composer specs: input, attach, model, effort, context, send | V2.2 | Ismail | ✅ merged |
| #4 | specs/thinking-loaders | Thinking, loaders, status-text specs | V2.2 | Khubaib | ✅ merged |
| #5 | specs/messages-tools | Messages, reasoning, tool calls, plans | V2.2 | Ismail | ✅ merged |
| #6 | specs/approvals-diff | Approvals, checkpoints, diff, merge conflict | V2.2 | Khubaib | ✅ merged |
| #7 | specs/genui-chrome-voice | Gen UI, workspace chrome, voice, icons | V2.2 | Ismail | ✅ merged |
| #8 | design/tokens-motion | Tokens, motion language, SVG logo | V2.3 | Khubaib | ✅ merged |
| #9 | docs/category-kits | Coding, app builder, CRM, finance, video shells | V2.4 | Ismail | ✅ merged |
| #10 | docs/site-studio-ia | Site map, gallery filters, Studio wireframes | V2.5 | Khubaib | ✅ merged |
| #11 | spike/rust-web | GPUI + Dioxus + WASM size proof ([report](spike/rust-web.md)) | V2.6 | Ismail | merged (+ GitHub #12 follow-up) |
| #12 | chore/workspace-ci | Cargo workspace, fmt/clippy/test CI (no artifacts) | M0 | Khubaib | merged (GitHub #13) |
| #13 | feat/tokens | Tokens JSON → Rust, CSS, GPUI theme | M0 | Ismail | merged (GitHub #14) |
| #14 | feat/core-state | AgentState machine, clock, verb packs | M1 | Khubaib | merged (GitHub #15) |
| #15 | feat/status-indicator | First component, all states | M1 | Ismail | merged (GitHub #16) |
| #16 | feat/loaders | Loader engine + first variant set | M2 | Khubaib | merged (GitHub #17) |
| #17 | feat/status-text | Shimmer, verb cycler, timers, reveal | M2 | Ismail | merged (GitHub #18) |
| #18 | feat/composer-shell | Input, attach, model, effort, context | M3 | Khubaib | merged (GitHub #19) |
| #19 | feat/send-control | Send / stop / queue / steer + variants | M3 | Ismail | merged (GitHub #20) |
| #20 | feat/tool-calls | Tool rows, plans, sub-agent tree | M4 | Khubaib | merged (GitHub #21) |
| #21 | feat/approvals-checkpoints | Approval cards, checkpoint timeline | M4 | Ismail | merged (GitHub #22) |
| #22 | docs/parity-roadmap | Parity inventories (affectUI, Arc, Beautiful UI, X posts), `status.csv`, this plan v2 | P0 | Khubaib | in progress (GitHub #23) |

### Plan v2 (from #23): parity build

Agreed 2026-11-12. Every row follows the definition of done in [parity/README.md](parity/README.md):
real components in Dioxus and GPUI, exact parity with the source, at least two extra variants or
controls, a full spec, and (from #24) a page on the site. "Items" counts rows in
[parity/status.csv](parity/status.csv).

| PR | Branch | Delivers | Items | Author | Status |
|---|---|---|---|---|---|
| #23 | feat/primitives-core | Buttons (button, icon, split, group, floating, expanding), badge, chip, value pill, entity chip + monogram, avatar with agent ring, kbd, separator, spinner, empty state, reaction bar, number ticker; switch, checkbox, radio, segmented, select, slider, text field; tooltip, popover, dropdown menu with gliding highlight | 20 | Ismail | in review (GitHub #24) |
| #24 | feat/site-v1 | Oxyn site in the affectUI layout: top nav (Components, Screens, Blocks, Illustrations, Motion, Docs, Guides, Compare, Pricing), category pages, component page (live preview, controls panel, variants, code, props, keyboard, a11y, responsive, install), theme toggle, ⌘K search, Cloudflare Pages deploy; pages for every component built so far | 7 | Khubaib | |
| #25 | feat/primitives-forms | Combobox, tag input, OTP, multi-select, inline edit, password field + strength, phone input, filter menu, calendar, date / range / time pickers, date reel, time dial, genui forms | 26 | Ismail | |
| #26 | feat/primitives-surfaces | Dialog, sheet, sheet stack, bottom sheet, toaster, notification stack, scroll area (stick to bottom), card, pagination, alert, context menu, command palette, skeletons, desktop overlays | 29 | Khubaib | |
| #27 | feat/app-shell-chat | affectUI-style app shell (resizable sidebar that snaps to icons, threads, folders, tabs, side pane), AI chat screens, messages, streaming markdown, reasoning, citations, terminal tabs, remaining composer parts (voice, queue, slash, mode, permission, tools); Beautiful UI harness and "Fork this" | 68 | Ismail | |
| #28 | feat/parity-built | Close every gap on plans #15–#21 (approval panes, plan editing, tool-kind map, send menus, GPUI IME, GPUI detail panes) against affectUI / Arc / Beautiful UI | 74 | Khubaib | |
| #29 | feat/diff-review | Diff viewer (split, stacked, per-hunk actions), inline diff, diff table, change summary, diff timeline, code block, review card | 5 + specs | Ismail | |
| #30 | feat/merge-conflict | 3-way merge conflict editor, conflict radar | specs | Khubaib | |
| #31 | feat/hitl-extras | Rollback impact sheet, checkpoint preview, decision inbox, autonomy dial, decision tree, recommendation card, selection actions, fine-tune card, away summary, agent recap, permission rules, swipe and review gestures | 24 | Ismail | |
| #32 | feat/multi-agent | Agent graph, handoff, collaboration stream, agent cursors, delegate board, findings, agent lanes, plan vs actual, fallback chain, root cause tree, monitor, trace view, fan-out | 28 | Khubaib | |
| #33 | feat/agent-ops | Parallel attempts, judge bracket, experiments, agent queue, scheduled runs, rate-limit planner, context map, tool access, integrations, memory, prompt library, writing tools, agent studio, provider keys, routing, run meter | 61 | Ismail | |
| #34 | feat/data-viz | Data table, KPI chart, records table with AI column, filter table, insight cards, context cards, cost waterfall, dashboards and charts | 42 | Khubaib | |
| #35 | feat/motion-effects | Number ticker variants, blur crossfade, gradient text and floor, grid background, tilt card, glass and dock morphs, genie menu, stretch pill, shockwave, theme wipe, shader sweep, ASCII and dither effects, interaction sounds with mute | 42 | Ismail | |
| #36 | feat/agent-x-kit | Agent mascots and orbs, voice orb and captions, tests visualised, GPUI rich editor, media placeholders, animated marks | 20 | Khubaib | |
| #37 | feat/screens-agent | Agent inbox, inbox board, tasks, activity, memory, orchestration, schedules, skills, usage, evals, mission control, run inspector, live session, Agent Screen | 18 | Ismail | |
| #38 | feat/screens-settings-auth | Settings (9 screens), onboarding (5), login family, 2FA, magic link, team, API keys, webhooks, env vars, import wizard | 43 | Khubaib | |
| #39 | feat/screens-business | Analytics, CRM, kanban (drag with tilt), files, workflow builder and flowchart, calendar boards, commerce, invoices, collaboration cards | 61 | Ismail | |
| #40 | feat/illustrations | Agent, integration and card illustrations, maps (licensed data), explainer illustrations, showpieces | 50 | Khubaib | |
| #41 | feat/marketing-kit | Pricing, logo marquee, testimonials, hero, bento, stats, theme switchers, changelog, FAQ, CTA, newsletter | 32 | Ismail | |
| #42 | feat/cli-registry | `oxyn` CLI + free registry (copy-paste install) | — | Khubaib | |
| #43 | feat/studio-v1 | Studio: click to edit, every prop as a control, state timeline, colour presets, radius modes, frame inspector | 2 | Ismail | |
| #44 | feat/kits | Coding, app builder, CRM, finance, video shells | — | Khubaib | |
| #45 | feat/pro-registry | Polar licences + Pro registry | — | Ismail | |
| #46 | release/v0.1 | Changelog, release, launch | — | Khubaib | |

> **Plan #23 scope note (2026-11-13):** `status.csv` had five affectUI motion rows (blur crossfade,
> gradient floor, gradient text, grid background, tilt card) under #23; the Delivers column puts them
> in #35, so they moved there. Arc's Context menu shipped in #23 as the dropdown menu's `context`
> mode. #23 counts 20 rows; the other primitives in this row (icon button, chip, value pill, entity
> chip + monogram, kbd, separator, spinner, radio, slider, popover) have their own rows under later
> plans or no row yet, and are built here too.

The old rows #22 to #30 (diff viewer, merge conflict, CLI, site, Studio, shells, Pro, release) are
folded into the table above.

> **Numbering note (2026-11-12):** the #11 follow-up (GPUI on the web) landed as GitHub PR #12, so from
> plan #12 on, the GitHub number is the plan number + 1 (plan #12 `chore/workspace-ci` is GitHub #13).
> This table keeps the plan numbers.
