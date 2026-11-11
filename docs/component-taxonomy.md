# Component taxonomy

Targets are variant counts for the full library (free + Pro), not V1 scope. V1 scope is in
[roadmap.md](roadmap.md).

## A. Component families

| # | Family | What's in it | Variant target |
|---|---|---|---|
| A | **Composer** | Input shells, `+` tools menu, attachment clip & chips (uploading → reading → ready → error morph), drop zones, model picker, thinking-effort slider/dial, context meter (ring, bar, grid), mode switch (Plan / Agent / Ask), permission & autonomy selector, slash commands, @mentions, voice / push-to-talk, queued messages with steer + reorder, **send / stop / queue / steer control** ([spec](specs/composer-send-control.md)) | 60+ |
| B | **Loaders & thinking indicators** | Dots, arcs, glyph frames (braille, block ░▒▓█), SVG discs, particle orbs (thinking-orb style), mini mascots, composer border light, shared busy signal | 180 |
| C | **Status-text effects** | Shimmer sweep, verb cycler + past tense ("Worked for 12s"), scramble, split-flap, odometer, typewriter / token reveal, blur-in stream, carets, elapsed timers | 100 + 12 verb packs |
| D | **Container & media loaders** | Border beam, shape-trace around tiles, fill-over-media, image-analysis scan, image-generation placeholder, window edge glow | 90 |
| E | **Messages & reasoning** | Bubbles, streaming markdown, collapsible thinking (normal / thinking / verbose), citations & source cards, confidence badges, branch arrows, feedback, follow-ups, memory receipts, empty states | 40+ |
| F | **Tool calls, plans & orchestration** | Tool rows/cards (all states), file-edit chips (+/−), terminal output, todo lists, plan card, step timeline, sub-agent tree, fan-out grid, "who's acting" indicator, trace views, background jobs tray, run replay scrubber, run meter (tokens, cost, TTFT) | 50+ |
| G | **Approvals & safety** | Allow once / session / always + deny, risk tiers, reason field the agent reads, rules by tool & argument, paged multi-question approvals, plan approval, audit log, sandbox badge, take-over / hand-back banner | 30+ |
| H | **Checkpoints, diff & git** | Checkpoint timeline (restore files / rewind chat / fork), rollback impact, diff viewer (split / stacked, per-hunk keep/undo), **3-way merge-conflict editor**, branch/worktree chips, PR card with CI, review card | 30+ |
| I | **Generative UI** | A2UI-style 18 primitives + charts, tables, forms, slides; skeletons per block × sweep styles; inline / fullscreen / PiP containers; "generating this UI" wireframe | 46 skeletons + blocks |
| J | **Workspace chrome** | Terminal tabs, side panels, file tree, status bar, MCP / plugin list (9 states), notification centre, ⌘K palette with AI answers, tray popover, hotkey launcher, floating HUD, notch status | 40+ |
| K | **Live & voice** | Orb states (idle / listening / thinking / speaking / asleep), visualisers, control bar, barge-in markers, captions, word-synced transcript, mini voice window, screen-share overlay | 28+ |
| L | **Icons & animated marks** | ~60 agent icons in 11 groups, plugin/MCP icon variations, 20 animated mark styles × 10 states (idle, thinking, working, tool, waiting, streaming, success, error, stalled, reduced-motion) | 200+ |
| M | **Shells, dashboards & screens** | Full app layouts assembled from A–L (see section B) | per category |

## B. Agent-app categories

Every family above is tagged with the categories it suits, and section M ships a **shell**, a
**dashboard** and a **full screen set** per category, built only from Oxyn parts (a forkable "harness"
with one made-up company per category, so demos read like a real product).

| Category | Shell / screens | Category-specific components |
|---|---|---|
| **Coding agents** | Desktop IDE agent window, CLI/TUI look, review queue | Diff/merge, checkpoints, terminal tabs, PR card, sub-agent grid |
| **App builders** | Prompt → preview split, version history, deploy panel | Web preview frame, element picker, "generating UI" wireframe, publish flow |
| **CRM agents** | Inbox + record view, pipeline board | Records table with calc footer, enrichment chips, outreach drafts with approval, activity timeline |
| **Finance agents** | Portfolio / ledger dashboard, reconciliation view | Transaction table, anomaly flags, scrubbable insight charts, approval with amount & risk |
| **Video / media agents** | Timeline editor with agent panel, render queue | Frame scrubber, generation placeholders, render progress, shot list, asset bin |
| **Support agents** | Ticket queue, conversation + suggested replies | Handoff banner, sentiment chip, macro picker, SLA timers |
| **Research agents** | Report canvas + sources rail | Source cards, citation hover, claim confidence, progress by sub-question |
| **Data / BI agents** | Notebook / SQL canvas | Query plan card, result table, chart generator, cost meter |
| **Browser / computer-use** | Live view + action log | Cursor trail overlay, step approvals, take-over control |
| **Voice / live agents** | Call screen, mini window | Orb sets, captions, barge-in, latency meter |
| **Multi-agent ops** | Orchestrator dashboard | Agent roster, run history strip, trace waterfall, cost/budget meters |

Categories are a filter on the gallery as well as a section: a user browsing *Finance* sees only the
composers, tables, approvals and shells tagged for it.

## C. Gallery filters
- **Family** (A–M) · **Category** (section B) · **State** (idle … stalled) · **Surface** (chat, IDE,
  desktop, mobile, voice, terminal) · **Motion** (none, subtle, expressive) · **Gesture tags** (hover,
  press, drag, type, select, loading, scroll, morph, reveal) · **Tier** (free, Pro).
