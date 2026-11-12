**Status:** draft v0.1 (2026-11-12).

# Category kit: CRM agents

Kit for agent apps that sit on top of a CRM: an agent that enriches records, researches accounts,
drafts outreach, files meeting notes and keeps the pipeline honest, while the user approves anything
that reaches a customer. Follows the 10-section standard in [README.md](README.md). Built only from
families A–L ([../component-taxonomy.md](../component-taxonomy.md), section B row "CRM agents":
records table with calc footer, enrichment chips, outreach drafts with approval, activity timeline).
Tokens from [../tokens.md](../tokens.md), motion from [../motion.md](../motion.md). All values marked
*proposed* are starting points for the Studio. Registry prefix: `shell/crm-<screen>` (family M).

## 1. Who it's for

**Job the user hires the agent for:** "Keep my book of accounts complete and moving. Fill in what I
don't know, tell me what changed, write the first draft, and never contact a customer without me."

| User | What they do here | What they must never lose |
|---|---|---|
| Account executive (primary) | works the inbox, approves drafts, preps for calls, moves deals | control over what reaches a customer |
| SDR / outbound rep | enriches lists, builds sequences, bulk sends | list quality, unsubscribe and bounce hygiene |
| Sales manager / RevOps | reads the dashboard, audits agent moves, sets rules | an audit trail of every agent write |

**Demo company (fictional, no real brands or people):**

| Thing | Fixture |
|---|---|
| Seller (the user's company) | **Tallyfern** — SaaS for fleet-maintenance scheduling, sold to regional logistics firms |
| Signed-in user | **Noor Castellan**, Account Executive, Tallyfern |
| The agent | **Fern** (agent colour `color.agent.green`, monogram "F") |
| Prospect companies | Harrowgate Freight, Bluestem Haulage, Corvid Cold Chain, Pellucid Parcel Co., Marrowbay Logistics, Quillon Tankers, Sablewood Movers (+193 generated) |
| Prospect people | Ines Varga (VP Operations, Harrowgate), Tomasz Okafor-Lind (Fleet Director, Bluestem), Priya Halloran (COO, Corvid), Dov Asante (Head of Procurement, Pellucid) |
| Enrichment sources | fictional: "Registry" (company filings), "Webscan" (company site), "Mailcheck" (email verifier), "Newswire" (news), "Notes" (internal) |
| Domains | `*.example` only (e.g. `harrowgate-freight.example`) |

## 2. Shell

Persistent frame: **left nav · main screen · agent panel**. The agent panel is a normal Oxyn thread
(messages, tool calls, approvals, composer); the main screen is where the agent's work lands.

### 2.1 Desktop (GPUI, ≥ 1280 px)
```text
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ ▣ Tallyfern   ⌘K Search records, ask Fern…            [◷ 2 jobs]  [Ask ▾]  [$ 41 credits]  NC │
├────────────┬───────────────────────────────────────────────────────┬─────────────────────────┤
│ ● Inbox  7 │ Companies › Harrowgate Freight                        │ Fern            ⋯   ✕   │
│   Records  │                                                       │─────────────────────────│
│   ├ Compan.│              (main screen: inbox / record / table /   │ ▸ Read 3 sources  2.1s  │
│   └ People │               pipeline / sequence / brief / dashboard)│ Drafted intro to Ines.  │
│   Pipeline │                                                       │ ┌ approval-card ──────┐ │
│   Sequences│                                                       │ │ ▲ High · Send email │ │
│   Meetings │                                                       │ │ to ines@harrowgate… │ │
│   Dashboard│                                                       │ │ [Allow once] [Deny] │ │
│ ── Agent ──│                                                       │ └─────────────────────┘ │
│   Runs     │                                                       │                         │
│   Rules    │                                                       │ ┌─────────────────────┐ │
│            │                                                       │ │ Ask Fern…        ⏎  │ │
│ Fern ● idle│                                                       │ └─────────────────────┘ │
├────────────┴───────────────────────────────────────────────────────┴─────────────────────────┤
│ Fern · Enriching Q4 freight list · 142/200 · 2 need you                       status bar 24 │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
  top bar 44 · nav 220 (collapses to 56) · main flex ≥ 560 · agent panel 360 (320–480)
```

| Region | Components |
|---|---|
| Left nav | `side-panel/rail` pattern for the nav list ([side-panels.md](../specs/side-panels.md)); count badges; agent presence row = [`acting-indicator/chip`](../specs/acting-indicator.md) + [`icon-state/crossfade`](../specs/icon-states.md) |
| Top bar search | [`command-palette/answer-cards`](../specs/command-palette.md) (Pro) / `command-palette/centered` (free): records, commands, "Ask Fern" |
| Top bar right | [`background-jobs/popover`](../specs/background-jobs.md) pill · [`permission-selector/chip-list`](../specs/composer-permission-selector.md) (Ask / Auto-edit / Full / Bypass) · [`run-meter/chip`](../specs/run-meter.md) showing enrichment credits · account |
| Agent panel | [`message/flat`](../specs/message-bubble.md), [`markdown/chat`](../specs/streaming-markdown.md), [`tool-call/row`](../specs/tool-call.md), [`approval-card/inline`](../specs/approval-card.md), [`composer-input/docked-bar`](../specs/composer-input-shell.md), [`trigger-menu/popover`](../specs/composer-slash-mentions.md) for `@record` and `/enrich` |
| Status bar | [`status-bar/run-strip`](../specs/side-panels.md) with [`status-text/line`](../specs/status-text.md) + [`verb-cycler/fade`](../specs/verb-cycler.md) |
| Splits | [`panel-split/hairline`](../specs/workspace-chrome.md); F6 cycles nav → main → agent panel → status bar |

Rules (proposed):
1. The agent panel is **scoped to the open record** when one is open ("Context: Harrowgate Freight"
   chip in the composer); otherwise to the current view (a table filter, a board).
2. Anything the agent writes to a record is visible **in the main screen**, never only in the thread.
3. The nav's Inbox badge counts items that need the user; it never counts FYI items.

### 2.2 Web (Dioxus)
Same layout. Differences: the top bar adds a "Connected · Mailbox" status chip; the agent panel can
pop out to a [`genui-surface/side-panel`](../specs/genui-renderer.md); tables render with CSS sticky
headers; keyboard shortcuts avoid browser-reserved keys (`⌘K` is fine, `⌘N` is not).

### 2.3 Breakpoints
| Width | Nav | Main | Agent panel |
|---|---|---|---|
| ≥ 1280 | 220 px labels | flex | docked 360 px |
| 960–1279 | 56 px icon rail (labels in tooltips) | flex | docked 320 px, or overlay sheet when main < 560 |
| 600–959 | hidden, ☰ opens a drawer | full | overlay sheet from the right, 85 % width |
| < 600 (mobile) | bottom tab bar: Inbox · Records · Pipeline · Fern | full | full-height sheet; composer docked to bottom |

### 2.4 Narrow / mobile
```text
┌───────────────────────────────┐
│ ☰  Inbox                 ◷ 2  │
├───────────────────────────────┤
│ NEEDS YOU · 7                 │
│ ┌───────────────────────────┐ │
│ │ ▲ Draft to Ines Varga     │ │
│ │ Harrowgate · intro email  │ │
│ │ [Review]                  │ │
│ └───────────────────────────┘ │
│ ┌───────────────────────────┐ │
│ │ ◇ 2 values disagree       │ │
│ │ Bluestem · employee count │ │
│ │ [Pick one]                │ │
│ └───────────────────────────┘ │
├───────────────────────────────┤
│ Ask Fern…                  ⏎  │
├───────────────────────────────┤
│ Inbox  Records  Pipeline  Fern│
└───────────────────────────────┘
```
Tables become card lists (`genui-table/compact-records` → one card per row, 3 key fields); the board
becomes one column at a time with a stage picker; approvals use [`approval-card/modal`](../specs/approval-card.md)
as a bottom sheet (`genui-form/sheet-mobile` for draft edits). Critical holds stay press-and-hold.

## 3. Screens

Seven screens. Each lists purpose, wireframe, components and its empty / loading / error states.

### 3.1 Agent inbox (`shell/crm-inbox`)
**Purpose:** one place for everything that needs the user: drafts to approve, enrichment conflicts,
proposed record updates, stalled runs, agent questions. FYI items ("Fern moved 3 deals") are a
separate, quieter tab.
```text
┌ Inbox ─────────────────────────────────────────────────────────────────────────────┐
│ Needs you 7 · FYI 12 · Done        [All] [Drafts 3] [Conflicts 2] [Updates 1] [Q 1] │
├────────────────────────────────────────┬───────────────────────────────────────────┤
│ ▲ Send intro to Ines Varga        2m   │ Draft · Harrowgate Freight                │
│   Harrowgate · email · High            │ To  Ines Varga <ines@harrowgate-freight…> │
│ ◇ Employee count disagrees       14m   │ Subj Cutting unplanned downtime at …      │
│   Bluestem · Registry 410 / Webscan 380│ ─────────────────────────────────────     │
│ ◇ 5 field updates from call      1h    │ Hi Ines, saw Harrowgate added 40 trailers │
│   Corvid · meeting notes · Medium      │ this quarter [1] …                        │
│ ⬣ Send step 1 to 142 contacts    1h    │ Sources  [1] Newswire · 3 d   [2] Webscan │
│   Q4 freight sequence · Critical       │ ─────────────────────────────────────     │
│ ? Which pipeline for Sablewood?  3h    │ ▲ High risk · emails an external contact │
│   question · 2 options                 │ [Edit] [Allow once ⏎Y] [Deny N] [Why not…]│
│ ⚠ Enrichment stalled · 9 failed  5h    │                                           │
│   Q4 freight list · retry              │                                           │
└────────────────────────────────────────┴───────────────────────────────────────────┘
```
| Part | Component |
|---|---|
| List | [`approval-card/inbox`](../specs/approval-card.md) (Pro) or `approval-card/paged` (free); rows as [`genui-list/rich-rows`](../specs/genui-blocks.md) with [`risk-badge/shape`](../specs/approvals.md) glyphs |
| Filters | chips with counts (`follow-ups/chips` styling, [citations-sources.md](../specs/citations-sources.md)) |
| Draft detail | [`genui-form/draft-review`](../specs/genui-forms.md) (Pro) + [`approval-card/footer`](../specs/approval-card.md); [`citation/number`](../specs/citations-sources.md) + [`source-list/list`](../specs/citations-sources.md) |
| Conflict detail | **new** `genui-question/source-pick` (§6) over [`enrichment-chip`](#6-new-components) values |
| Updates detail | [`diff-viewer/table-cells`](../specs/diff-viewer.md) with per-row keep / undo |
| Question detail | [`genui-question/numbered`](../specs/genui-forms.md) |
| Stalled run | [`fanout-grid/cells`](../specs/fanout-grid.md) summary + retry |

Ordering: newest first within tier; a Critical item shows "1 critical" in the header but does not jump
the list (approvals.md §2.6). Answering an item resolves it in place (220 ms `spring.snap` morph to a
one-line row) and advances focus to the next.

| State | Shows |
|---|---|
| Empty | `empty-state/minimal`: "Nothing needs you. Fern is idle." + last 3 FYI items |
| Loading | 6 [`skeleton/shimmer`](../specs/skeletons.md) rows; detail pane `skeleton/static` |
| Error | inline banner "Couldn't load inbox · Retry"; cached items stay, marked "may be stale" |
| Offline | actions disabled with reason; approvals never queue locally (approvals.md §2.1 rule 5) |

### 3.2 Record view (`shell/crm-record`)
**Purpose:** a company or person page: header, fields with enrichment provenance, deals, and an
activity timeline mixing human and agent events.
```text
┌ Harrowgate Freight ─────────────────────────────────────────── [Ask Fern] [Enrich] ⋯ ┐
│ HF  harrowgate-freight.example · Owner Noor · Stage Evaluation · ARR $48k (est.)       │
├──────────────────────────────┬───────────────────────────────────────────────────────┤
│ Fields                       │ Activity                       [All] [Agent] [Email]  │
│ Industry  Freight ⓡ ● 3d     │ ● Today 10:42  Fern drafted intro to Ines   [Review]  │
│ Employees 410 ⓡ ● 3d    ◇ 2  │ │              ✦ from Newswire, Webscan               │
│ Trucks    1,240 ⓦ ◐ 12d      │ ● Today 09:15  Fern enriched 6 fields       [Undo]    │
│ HQ        Leeds-on-Ouse ⓦ ●  │ │              Employees, Trucks, HQ, +3              │
│ Tech      FleetLedger ⓝ ○    │ ● Mon 16:00    Call with Ines (32 min)      [Notes]   │
│ Email     ines@… ⓜ ● valid   │ │              Stage Discovery → Evaluation (Fern)    │
│ Phone     — ⟳ filling        │ ● Mon 11:02    Noor emailed Ines                      │
│                              │ ● 2 Nov        Created from Q4 freight list           │
│ Deals                        │                                                       │
│ Fleet pilot · $48k · Eval ▸  │                                                       │
│ People (3) Ines · Tomasz · + │                                                       │
└──────────────────────────────┴───────────────────────────────────────────────────────┘
 ⓡ Registry  ⓦ Webscan  ⓝ Newswire  ⓜ Mailcheck · ● Sourced ◐ Inferred ○ Unverified · ◇ conflict
```
| Part | Component |
|---|---|
| Header | monogram (`color.monogram.*`) + [`genui-metric/inline`](../specs/genui-blocks.md) for ARR; agent presence [`acting-indicator/chip`](../specs/acting-indicator.md) when Fern is writing here |
| Fields | **new** `enrichment-chip` per value (source mark, [`confidence-badge/icon`](../specs/citations-sources.md), age); [`citation/quote-peek`](../specs/citations-sources.md) on hover (Pro) |
| Filling field | `enrichment-chip` `Filling` state ([`text-shimmer/calm`](../specs/text-shimmer.md) on the placeholder) |
| Activity | [`genui-timeline/vertical`](../specs/genui-blocks.md) (free) / **new** `genui-timeline/activity` (Pro); agent items carry the agent swatch and an Undo |
| Undo of agent writes | [`rollback-impact/inline`](../specs/rollback-impact.md) ("Undo 6 field changes · 1 was edited since") |
| Deals, people | [`genui-list/rich-rows`](../specs/genui-blocks.md) |

| State | Shows |
|---|---|
| Empty (new record) | header + empty fields with "Enrich with Fern" call to action; timeline "Created just now" |
| Loading | header skeleton, 8 field rows `skeleton/static`, timeline `skeleton/line-by-line` |
| Partial | fields that failed show `Failed` chip "No match in 3 sources · Retry" |
| Error | "Record unavailable" + back; never a blank page |
| Deleted | read-only tombstone "Deleted by Noor · Restore (30 days)" (app policy) |

### 3.3 Records table (`shell/crm-table`)
**Purpose:** lists of companies or people; the agent fills columns cell by cell; the footer
summarises the current filter.
```text
┌ Q4 freight list · 200 companies ──────────── [Filter ▾] [+ Agent column] [Enrich all] ⋯ ┐
│ ☐ Company ▲           │ Employees │ Trucks  │ Fit score (Fern) │ Email found │ Owner    │
├───────────────────────┼───────────┼─────────┼──────────────────┼─────────────┼──────────┤
│ ☐ Harrowgate Freight  │ 410 ●     │ 1,240 ◐ │ 82 ●             │ ✓ ines@…    │ Noor     │
│ ☐ Bluestem Haulage    │ ◇ 410/380 │ 960 ●   │ 74 ●             │ ✓ tomasz@…  │ Noor     │
│ ☐ Corvid Cold Chain   │ 220 ●     │ ░░░░░░  │ ░░░░░░           │ ⟳           │ —        │
│ ☐ Pellucid Parcel Co. │ ░░░░░░    │ ░░░░░░  │ · queued         │ · queued    │ —        │
│ ☐ Quillon Tankers     │ ✕ no match│ ✕       │ —                │ ✕ Retry     │ —        │
│ …                     │           │         │                  │             │          │
├───────────────────────┼───────────┼─────────┼──────────────────┼─────────────┼──────────┤
│ 200 count             │ avg 312   │ sum 61k │ avg 68 · 142/200 │ 71 % found  │ 3 owners │
└───────────────────────┴───────────┴─────────┴──────────────────┴─────────────┴──────────┘
 ░ filling (cell shimmer)  · queued  ⟳ running  ✕ failed  ◇ conflict  ● ◐ confidence
```
| Part | Component |
|---|---|
| Grid | [`genui-table/data-grid`](../specs/genui-blocks.md) (free, virtualised, sticky first column), density `compact` (28 px rows) |
| Calc footer | [`genui-table/calc-footer`](../specs/genui-blocks.md) (Pro); free fallback: a [`genui-metric/strip`](../specs/genui-blocks.md) above the grid |
| Agent columns, cell loading | **new** `genui-table/agent-columns` (§6): per-cell `Queued / Filling / Filled / Failed / Conflict / Locked` |
| Changed cells | [`genui-table/changed-cells`](../specs/genui-blocks.md) highlighter on cells Fern changed since you last looked |
| Selection | [`genui-table/bulk-actions`](../specs/genui-blocks.md) (Pro): floating bar "12 selected · Enrich · Add to sequence · Export"; every bulk intent goes through approvals |
| Run progress | header chip from [`background-jobs/dock-pill`](../specs/background-jobs.md); "Show run" opens [`fanout-grid/cells`](../specs/fanout-grid.md) |
| "+ Agent column" | [`genui-form/inline`](../specs/genui-forms.md): column name + instruction ("Fit score 0–100 from trucks, region, tech stack") + sources |

Rules (proposed):
1. Footer recomputes on filter and as cells fill; numbers roll with [`genui-metric/odometer`](../specs/genui-blocks.md) timing (`stagger.digit` 30 ms), at most once per 500 ms during a run.
2. Footer counts only `Filled` cells and says so: "avg 68 · 142/200" (n shown when < total).
3. A user edit to a cell sets `Locked`; the agent never overwrites a locked cell; its value becomes a conflict.
4. Only cells in view animate; off-screen cells update silently (motion.md §9 budget).

| State | Shows |
|---|---|
| Empty | "No companies yet · Import CSV · Ask Fern to find companies" |
| Loading | header + 12 `skeleton/shimmer` rows; footer `—` |
| Filtering to 0 | "No rows match · Clear filter"; footer "0 count" |
| Error | failed page band in the grid "Couldn't load rows 400–450 · Retry" |

### 3.4 Pipeline board (`shell/crm-pipeline`)
**Purpose:** deals as cards in stage columns. The agent can move cards; every agent move carries a
reason and an Undo, and lands in the FYI tab.
```text
┌ Pipeline · Q4 · Noor ─────────────── 34 deals · $1.21M ── [Board|Table] [Fern moves: 3 ▾] ┐
│ Discovery 9 · $310k │ Evaluation 7 · $402k │ Proposal 6 · $288k │ Stalled 4 · $96k │ Won 8 │
├─────────────────────┼──────────────────────┼────────────────────┼──────────────────┼───────┤
│ ┌─────────────────┐ │ ┌──────────────────┐ │ ┌────────────────┐ │ ┌──────────────┐ │       │
│ │ Bluestem        │ │ │ Harrowgate  $48k │ │ │ Pellucid  $62k │ │ │ Corvid  $30k │ │       │
│ │ $40k · 21 Dec   │ │ │ close 19 Dec     │ │ │ close 12 Dec   │ │ │ ↳ F moved    │ │       │
│ └─────────────────┘ │ │ ↳ F moved 2h ago │ │ └────────────────┘ │ │ from Proposal│ │       │
│ ┌─────────────────┐ │ │ "Call: agreed    │ │                    │ │ "No reply in │ │       │
│ │ Marrowbay       │ │ │ pilot scope"     │ │                    │ │ 21 days"     │ │       │
│ │ ⟳ Fern updating │ │ │ [Undo] [Why]     │ │                    │ │ [Undo] [Why] │ │       │
│ └─────────────────┘ │ └──────────────────┘ │                    │ └──────────────┘ │       │
└─────────────────────┴──────────────────────┴────────────────────┴──────────────────┴───────┘
```
| Part | Component |
|---|---|
| Board | **new** `genui-board` (§6) `genui-board/columns` (free), `genui-board/agent-moves` (Pro: move ribbon with reason, ghost at old position, Undo) |
| Column header totals | [`genui-metric/inline`](../specs/genui-blocks.md), digits roll on change |
| Card "Fern updating" | [`border-beam/hairline`](../specs/border-beam.md) on the card while the agent writes |
| "Fern moves: 3" | popover list of today's agent moves, each linking to its audit entry ([`audit-log/timeline`](../specs/permission-rules.md)) |
| Stage-change approval (when level = Ask) | [`approval-card/inline`](../specs/approval-card.md) anchored to the card, showing from → to and reason |
| Table toggle | `genui-table/compact-records` with stage as a column |

Agent-move motion: card lifts (shadow `elevation.popover`), travels `dur.travel` 240 ms `ease.in-out`
to the new column, lands with `spring.snap`; the old slot leaves a dashed ghost for `dur.beat`
600 ms. Several moves stagger `stagger.base` 40 ms (cap 6). Reduced motion: card crossfades in place
(`dur.instant` 80 ms) and the ribbon carries the move.

| State | Shows |
|---|---|
| Empty | "No deals in this pipeline · New deal · Ask Fern to build deals from Won-ready leads" |
| Loading | column headers + 3 card skeletons per column |
| Error | per-column error "Couldn't load Proposal · Retry"; other columns stay usable |
| Drag conflict | the user drags a card Fern is moving: user wins, Fern's move is cancelled and logged |

### 3.5 Outreach sequence editor (`shell/crm-sequence`)
**Purpose:** a multi-step outreach plan (emails, tasks, waits) with an agent-drafted, per-contact
email at each step, and a clear view of what is approved, scheduled and sent.
```text
┌ Q4 freight intro · 142 enrolled ─────────────────── [Preview as ▾ Ines] [Start step 1 ▸] ┐
│ Steps                              │ Step 1 · Email · Day 0 · send window 09:00–11:00 local │
│ ① Email   Day 0    142 drafted  ⬣ │ ┌ draft for Ines Varga (1 of 142) ‹ › ───────────────┐ │
│ │  wait 3 business days           │ │ Subj  Cutting unplanned downtime at Harrowgate     │ │
│ ② Email   Day 3    template       │ │ Hi Ines, saw Harrowgate added 40 trailers [1] …    │ │
│ │  wait 4 business days           │ │ ✦ personalised: trailers, region, FleetLedger      │ │
│ ③ Task    Day 7    call           │ └────────────────────────────────────────────────────┘ │
│ ④ Email   Day 10   breakup        │ Review: 118 ready · 12 flagged · 12 excluded (unsub.) │
│ [+ Step]                          │ [Review flagged 12] [Regenerate all] [Edit template]  │
│ Stops when: reply · meeting booked│ ⬣ Critical · bulk send to 130 contacts · hold to send │
└───────────────────────────────────┴───────────────────────────────────────────────────────┘
```
| Part | Component |
|---|---|
| Steps rail | [`step-timeline/vertical`](../specs/plan-todo.md) with wait rows; per-step approval state glyph from [`risk-badge/shape`](../specs/approvals.md) |
| Draft pager | [`genui-form/draft-review`](../specs/genui-forms.md) + `approval-card/paged` pager "1 of 142"; personalised spans marked ✦ ([`genui-form/prefill-sparkle`](../specs/genui-forms.md) style) |
| Flagged drafts | [`genui-question/paged`](../specs/genui-forms.md) "Draft 1 of 12 · reason: claim not sourced" with [`confidence-badge/label`](../specs/citations-sources.md) |
| Template edit | [`diff-viewer/redline`](../specs/diff-viewer.md) for "what changed vs template" |
| Start / send | **new** `approval-card/bulk-recipients` (§6), Critical |
| Running send | [`background-jobs/dock-pill`](../specs/background-jobs.md) "Sending 38/130 · Stop remaining" |

| State | Shows |
|---|---|
| Empty | "No steps · Ask Fern to draft a 4-step sequence for this list" |
| Drafting | [`fanout-grid/cells`](../specs/fanout-grid.md) of 142 drafts filling; pager disabled until ≥ 1 ready |
| Partially drafted | "118 / 142 drafted · 24 failed (no verified email)" with "Exclude failed" |
| Error | "Mailbox disconnected · Reconnect" ([`mcp-plugin-list`](../specs/mcp-plugin-list.md) `needs-auth` state); send disabled with reason |
| Sent | step row ✓ "Sent 128 · 2 bounced"; drafts become read-only |

### 3.6 Meeting prep brief (`shell/crm-meeting-prep`)
**Purpose:** a one-page brief before a call: who is coming, what happened so far, what changed,
open questions, and suggested goals. Everything cited.
```text
┌ Prep · Harrowgate Freight · Thu 14:00 (in 2h) ───────────────── [Regenerate] [Share] ┐
│ Attendees  Ines Varga (VP Ops) ● · Tomasz Okafor-Lind (Fleet Dir.) ◐ new to deal      │
├──────────────────────────────────────────────────────┬──────────────────────────────┤
│ Since last call                                       │ Sources                      │
│ • Added 40 trailers this quarter [1] ● Sourced        │ [1] Newswire · 3 d           │
│ • Pilot scope agreed: 120 trucks, Leeds depot [2]     │ [2] Call notes · Mon         │
│ • Tomasz joined in Oct; reports to Ines [3] ◐         │ [3] Webscan · 12 d           │
│ Risks                                                 │ [4] Email thread · 6 d       │
│ • Uses FleetLedger; renewal in March [4] ○ Unverified │                              │
│ Goals for this call                     (checklist)   │ Deal  Fleet pilot · $48k     │
│ ☐ Confirm pilot start date  ☐ Name the budget owner   │ Eval · close 19 Dec          │
│ Questions to ask                                      │ Last touch  Mon (Noor)       │
└──────────────────────────────────────────────────────┴──────────────────────────────┘
```
| Part | Component |
|---|---|
| Body | [`markdown/editorial`](../specs/streaming-markdown.md) streaming, [`citation/number`](../specs/citations-sources.md), [`confidence-badge/icon`](../specs/citations-sources.md) per claim |
| Sources | [`source-list/rail`](../specs/citations-sources.md), `Searching(n)` state while gathering |
| Goals | [`genui-list/checklist`](../specs/genui-blocks.md) (ticks write back to the deal's next steps) |
| Deal sidebar | [`genui-metric/tile`](../specs/genui-blocks.md) + `enrichment-chip` |
| Generation | [`genui-skeleton/progressive-detail`](../specs/genui-skeletons.md) for sections not yet streamed |

| State | Shows |
|---|---|
| Empty | no meeting: "Pick a meeting or ask Fern to prep for one" |
| Generating | section headings first, bodies stream; "Reading 4 sources" in the rail |
| Thin | < 2 sources: banner "Little to go on · Fern found 1 source" (never pads with filler) |
| Error | "Couldn't reach calendar · Prep from records only?" |

### 3.7 Dashboard (`shell/crm-dashboard`)
**Purpose:** pipeline health and what the agent did, at a glance.
```text
┌ Dashboard · Q4 · Noor ────────────────────────────────────────────── [This week ▾] ┐
│ Pipeline $1.21M ▲ 8%  │ Weighted $412k  │ Won $286k / $400k ███████░░ │ Stalled 4  │
├────────────────────────────────────────────┬─────────────────────────────────────────┤
│ Pipeline by stage (this vs last week)      │ Fern this week                          │
│  Disc ████████░  Eval ██████░  Prop █████  │ 412 fields enriched · 9 failed          │
│  Stall ██░  Won ███████                    │ 31 drafts · 27 approved · 4 denied      │
│                                            │ 6 deals moved · 1 undone                │
├────────────────────────────────────────────┤ 2 runs · 1 stalled (provider limit)     │
│ Reply rate by sequence step                │ Credits 41 left · ▁▂▃▅▃▂                 │
│  ① 14%  ② 9%  ③ —  ④ 3%                    │ [Open audit log]                        │
└────────────────────────────────────────────┴─────────────────────────────────────────┘
```
| Part | Component |
|---|---|
| KPI strip | [`genui-metric/strip`](../specs/genui-blocks.md) (free) / `genui-metric/board` (Pro); quota as `genui-metric/target-vs-actual` (Pro) |
| Stage chart | [`genui-chart/card`](../specs/genui-blocks.md) (free) / `genui-chart/compare` (Pro) |
| Reply rate | `genui-chart/card` bar |
| Agent activity | [`genui-list/plain`](../specs/genui-blocks.md) counts + [`run-meter/sparkline-session`](../specs/run-meter.md) for credits; link to [`audit-log/table`](../specs/permission-rules.md) |
| Runs | [`trace-view/cost-rows`](../specs/trace-view.md) on demand (Pro) |

States: Empty ("Connect a pipeline"), Loading (`skeleton/chart-draw` per tile), Error per tile
(other tiles stay), Stale ("Updated 3h ago · Refresh").

## 4. Agent flows

Each flow walks `AgentState` ([../architecture.md](../architecture.md)). "Shows" names the component
the user sees at that step.

### 4.1 Enrich a list of 200 leads (fan-out, partial failures)
| # | AgentState | What happens | Shows |
|---|---|---|---|
| 1 | `Composing` | Noor types `/enrich` on the Q4 list: "Fill employees, trucks, HQ, email; add a fit score" | [`trigger-menu/popover`](../specs/composer-slash-mentions.md), [`token-chip/pill`](../specs/composer-slash-mentions.md) "@Q4 freight list" |
| 2 | `Submitted` → `Thinking{"Planning"}` | plans columns, sources, credit cost | [`status-text/line`](../specs/status-text.md) |
| 3 | `AwaitingApproval{Plan}` | plan: 5 columns × 200 rows, est. 410 credits, writes empty fields only | [`plan-card/document`](../specs/plan-todo.md) + [`plan-approval/footer`](../specs/plan-approval.md); spend shown as [`approval-card/amount`](../specs/approval-card.md) when over the run budget |
| 4 | `ToolRunning{enrich,"Enriching"}` | 200 tasks fan out, 8 at a time | table cells go `Queued → Filling → Filled` (`genui-table/agent-columns`); [`fanout-grid/cells`](../specs/fanout-grid.md) in the agent panel; `background-jobs/dock-pill` 0→200 |
| 5 | `Stalled{since}` | provider rate-limited for 45 s | grid header "No progress for 45 s", [`text-shimmer/stall-ember`](../specs/text-shimmer.md) (Pro) / colour ramp; Retry / Wait |
| 6 | `Steering` | Noor types "skip anyone under 50 trucks" mid-run | [`composer-input/split-steer`](../specs/composer-input-shell.md) (Pro) or queued message; 23 queued cells → `Skipped` |
| 7 | `ToolRunning` | continues; 9 cells fail (no domain), 4 conflicts | cells `Failed` / `Conflict`; inbox gains 2 items (conflicts grouped by field, failures grouped by reason) |
| 8 | `Done{elapsed}` | "Worked for 6m 12s · 164 enriched · 23 skipped · 9 failed · 4 need you" | [`elapsed-timer/summary`](../specs/elapsed-timer.md), [`background-jobs/batch-retry`](../specs/background-jobs.md) "9 failed: no domain · Retry with Webscan" |

User can steer at any time: stop (cells already filled stay), skip a column, raise concurrency (if
the app allows), pin a source. Risk: Medium plan (writes to many records, reversible); overwriting
non-empty values is not in this plan and would need its own approval.

### 4.2 Research + draft outreach → approve → send
| # | AgentState | What happens | Shows |
|---|---|---|---|
| 1 | `Composing` | on Harrowgate's record: "Write Ines an intro about the trailer expansion" | composer with "Context: Harrowgate Freight" chip |
| 2 | `ToolRunning{web.search,"Researching"}` | reads Newswire, Webscan, past emails | [`tool-call/row`](../specs/tool-call.md), [`source-list/stack`](../specs/citations-sources.md) `Searching(3)` |
| 3 | `Streaming` | draft streams into editable fields | `genui-form/draft-review` with [`citation/number`](../specs/citations-sources.md) markers |
| 4 | `AwaitingApproval{SendEmail, High}` | one external recipient | [`approval-card/footer`](../specs/approval-card.md) under the draft (arms 600 ms; `Enter` = Deny); mirrored in the inbox |
| 5a | user edits subject → `AllowEdited` | agent must send the edited version | `genui-form/review-diff` (Pro) shows old → new before allow |
| 5b | user `DenyWithReason("too salesy, mention the pilot")` | agent redrafts | "Sent to agent" line; back to `Thinking` |
| 6 | `ToolRunning{email.send,"Sending"}` | mailbox send | tool row; 5 s "Undo send" if the mail tool supports a delay (app) |
| 7 | `Done` | timeline gets "Email sent to Ines"; next-step task suggested | [`follow-ups/chips`](../specs/citations-sources.md) "Remind me in 3 days" |

### 4.3 Meeting notes → update records → next steps
| # | AgentState | What happens | Shows |
|---|---|---|---|
| 1 | `Composing` | Noor drops the call transcript (or the call app pushes it) | [`composer-attachments`](../specs/composer-attachments.md) chip `uploading → reading → ready` |
| 2 | `ToolRunning{transcript.read,"Reading"}` → `Streaming` | summary + decisions | `markdown/chat` with [`citation/timestamp`](../specs/citations-sources.md) (Pro) to transcript moments |
| 3 | `AwaitingApproval{RecordWrite, Medium}` ×4 (batch) | stage Discovery → Evaluation, amount $40k → $48k, 2 new contacts | [`diff-viewer/table-cells`](../specs/diff-viewer.md) with per-row keep / undo; batch "Allow all 4 once" (approvals.md §2.6) |
| 4 | `AwaitingApproval{RecordWrite, High}` | close date moves out of the committed quarter | separate [`approval-card/inline`](../specs/approval-card.md) (High is never batched) |
| 5 | `ToolRunning{crm.update}` | writes; record header and board card update | `genui-table/changed-cells` highlight; board card moves (§3.4) |
| 6 | `Streaming` | next steps + follow-up email draft | [`genui-list/checklist`](../specs/genui-blocks.md) tasks; draft enters flow 4.2 at step 4 |
| 7 | `Done` | "Updated 5 fields · 2 tasks · 1 draft waiting" | [`memory-receipt/footer-line`](../specs/citations-sources.md)-style receipt line linking to the audit log |

### 4.4 Deal-stage change with reason (background run)
| # | AgentState | What happens | Shows |
|---|---|---|---|
| 1 | `Idle` → `Thinking{"Reviewing pipeline"}` | scheduled hygiene run, Fridays 16:00 | [`background-jobs/scheduled`](../specs/background-jobs.md) (Pro); nav presence chip |
| 2 | `ToolRunning{crm.read}` | finds Corvid: Proposal, no reply 21 days | — |
| 3a | level `Ask` → `AwaitingApproval{StageChange, Medium}` | "Move Corvid to Stalled · No reply in 21 days to 2 emails" | approval anchored to the card; inbox item |
| 3b | level `FullAccess` → auto-resolved | card moves; reason ribbon; FYI item | `genui-board/agent-moves`; audit `by: Level(FullAccess)` |
| 4 | user presses Undo / "Why" | Undo returns the card and sends "rejected: <reason>" to the agent | [`rollback-impact/inline`](../specs/rollback-impact.md) if fields changed after the move |
| 5 | `Done` | "Moved 3 deals · 1 needs you" | status bar + FYI tab |

Moves **into Won or Lost** are High (they change forecast and commission): never auto-run below
Bypass, never batched.

### 4.5 Bulk send (Critical)
| # | AgentState | What happens | Shows |
|---|---|---|---|
| 1 | `Composing` | "Start step 1 of Q4 freight intro" (or presses Start) | sequence editor |
| 2 | `Thinking{"Checking list"}` | removes 12 unsubscribed, 2 bounced-before; flags 12 drafts | review counts in the editor |
| 3 | `AwaitingApproval{BulkSend, Critical}` | 130 recipients, 1 mailbox, window 09:00–11:00 recipient-local, throttle 40/h | **new** `approval-card/bulk-recipients` in [`approval-card/modal`](../specs/approval-card.md); `z.approval`; focus on Deny |
| 4 | user reviews | samples: 5 rendered drafts (random + all flagged), exclusions list, sender, window | pager inside the card; "Review flagged 12" must be visited or excluded first (proposed) |
| 5 | press-and-hold 1200 ms | only "Allow once"; no session / always; **stops even in Bypass** | hold ring (`ease.linear`), `risk.critical` hatch header |
| 6 | `ToolRunning{email.send_bulk,"Sending"}` | sends over 3 h | `background-jobs/dock-pill` "Sending 38/130"; **Stop remaining** always enabled; sent mails are not recalled |
| 7 | `Error{kind: PartialFailure}` or `Done` | "Sent 128 · 2 bounced · 0 complaints" | `background-jobs/batch-retry`; bounced contacts flagged in the table |

If the user changes the list after allowing (adds 10 contacts), the grant does not stretch: the 10
need a new Critical approval (the request is for an exact recipient set, approvals.md §2.1 rule 4).

## 5. Approvals and risk

Tiers per [../specs/approvals.md](../specs/approvals.md) §2.3. CRM actions map onto request kinds
as `Custom { label, icon }` with a declared floor (approvals.md §2.2); the app classifier can only
raise a tier.

| Action | Kind (proposed) | Tier | Always-allow? | Notes |
|---|---|---|---|---|
| Read / search records, read calendar | — | none (no card) | — | |
| Public web research | `Network` (GET) | Low | yes, per source | new hosts follow approvals.md defaults |
| Draft an email, note or brief (not sent) | — | none | — | drafts are inert |
| Fill **empty** fields on ≤ 25 records | `RecordWrite` | Low | yes (rule: field set + "empty only") | |
| Enrich a list (> 25 records, empty fields) | `Plan` + `RecordWrite` | Medium | session | shows credit cost |
| Overwrite a non-empty field | `RecordWrite` | Medium | session only | user-locked cells: never, becomes a conflict |
| Change a deal stage (not Won / Lost) | `StageChange` | Medium | yes, per pipeline | reason required from the agent |
| Create records, add contacts to a sequence (not started) | `RecordWrite` | Medium | yes | |
| Spend enrichment credits above the run budget | `Spend` | High | no | [`approval-card/amount`](../specs/approval-card.md) |
| Send one email to one external contact | `SendEmail` | **High** | no (session = same exact draft only) | 600 ms arming; `Enter` = Deny |
| Book / send a calendar invite to an external contact | `SendInvite` | High | no | |
| Move a deal to Won / Lost; change amount or close date in a committed forecast | `StageChange` / `RecordWrite` | High | no | never batched |
| Reassign owner; merge two records | `RecordWrite` | High | no | merge shows field-by-field winner |
| Send to many contacts (≥ 25, app-set) | `BulkSend` | **Critical** | never | hold 1200 ms; only Allow once; **stops even in Bypass** |
| Delete records (any count) | `RecordDelete` | **Critical** | never | [`rollback-impact/hold-to-confirm`](../specs/rollback-impact.md) shows dependants (deals, activities) |
| Export PII (CSV, sync to another tool, share link) | `Export` | **Critical** | never | lists fields included; emails / phones named |
| Change consent / unsubscribe fields in bulk | `RecordWrite` | **Critical** | never | legal exposure |

Permission levels in this kit ([composer-permission-selector.md](../specs/composer-permission-selector.md)):

| Level | Auto-runs in CRM |
|---|---|
| `ReadOnly` | nothing writes; drafts allowed |
| `Ask` (default) | nothing; every write asks |
| `AutoEdit` | Low `RecordWrite` only (empty fields) |
| `FullAccess` | up to Medium writes and stage changes; High still asks unless a rule exists *(proposed: CRM kit sets FullAccess to stop at Medium, since High here means customer contact)* |
| `Bypass` | up to High; **never Critical** (`bypass_includes_critical = false`) |

Every resolution writes an `AuditEntry`; the kit's Rules screen is [`permission-rules/table`](../specs/permission-rules.md)
+ [`audit-log/table`](../specs/permission-rules.md), filtered by record.

## 6. New components

Kept small: two new components and five new variants of existing ones. Each gets its full 10-point
spec before it is built.

| Proposed name | Family | Tier | Kind |
|---|---|---|---|
| `enrichment-chip` | E (messages & reasoning: provenance, next to `confidence-badge`) | Free 3 + Pro 4 | new component |
| `genui-board` | I (generative UI block) | Free 2 + Pro 4 | new block |
| `genui-table/agent-columns` | I | Pro (free fallback: `data-grid` + per-row loader) | new variant |
| `genui-timeline/activity` | I | Pro (free fallback: `genui-timeline/vertical`) | new variant |
| `genui-question/source-pick` | I | Free (proposed) | new variant |
| `approval-card/bulk-recipients` | G | Free (proposed; safety stays free) | new variant |
| `genui-surface/record-scope` chip | I / A | — | **not new**: reuse `token-chip/pill` as the composer context chip |

**`enrichment-chip`.** One field value with where it came from. Anatomy: value · source mark (letter
monogram or favicon the app supplies) · confidence glyph (reuses `confidence-badge` grammar: Sourced ●
/ Inferred ◐ / Unverified ○, Calibrated only if the provider gives one) · age ("3d"). States:
`Empty`, `Queued`, `Filling` (shimmer on placeholder, `text-shimmer/calm`), `Filled` (value types in
with `dur.base` 180 ms crossfade, highlight `opacity.highlight` fades over `dur.beat`), `Conflict{n}`
(◇ n, opens `source-pick`), `Failed{reason}`, `Stale{age}` (dims to `fg.muted`, "Refresh"),
`Locked` (user-entered, lock glyph, agent cannot write), `Verified` (user confirmed). Hover / focus
opens a peek: each source's value, date, quoted snippet ([`citation/quote-peek`](../specs/citations-sources.md)
pattern). Free: `enrichment-chip/inline` (value + source letter), `/cell` (table-cell fit, glyph
only), `/glyph` (TUI `410 [r●3d]`). Pro: `/stacked-sources` (avatars of all sources that agree),
`/waterfall` (shows the provider order tried, hits and misses — Clay waterfall idea, study only),
`/freshness-ring` (ring empties as data ages), `/diff-peek` (old → new on refresh).

**`genui-board`.** Columns of cards (kanban) as a gen-UI block so agents can emit boards too.
Columns have header totals; cards have title, meta, and an optional agent ribbon. States per card:
`Idle`, `Dragging`, `AgentEditing` (border beam), `AgentMoved{from, reason, at}` (ribbon, Undo,
Why), `AwaitingApproval` (anchored approval), `Locked`. Keyboard drag: Space picks up, arrows move,
Space drops, Esc cancels, with live-region announcements. Free: `genui-board/columns`,
`/compact-list` (mobile, one column + stage picker). Pro: `/agent-moves` (ghost slot + travel +
reason), `/swimlanes` (rows by owner), `/wip-limits` (column caps warn), `/forecast-weights`
(column value × probability shown in headers). Relation: `fanout-grid/inbox-board` and
`todo-list/board` keep their own jobs; all three should share one card-travel motion token
(open question).

**`genui-table/agent-columns`.** Column header gets an agent mark, instruction peek, source list and
"Run column" / "Stop". Each cell is an `enrichment-chip/cell` with states Queued · Filling · Filled ·
Failed · Conflict · Locked · Skipped. Batched announcements ("12 cells filled in Trucks"). Only
cells in view animate; max 8 shimmer at once (loaders.md busy budget).

**`genui-timeline/activity`.** Vertical record timeline with item kinds (email, call, meeting, note,
field change, stage change, agent action), filter chips, agent items with swatch + Undo + "Why",
grouping ("Fern enriched 6 fields" folds 6 rows, `stagger.fold`), and day dividers.

**`genui-question/source-pick`.** A conflict question: one field, 2–4 candidate values, each with
`enrichment-chip` provenance; options "Use Registry (410)", "Use Webscan (380)", "Enter value",
"Keep empty"; "(recommended)" marker only if the agent gives a reason. Answers can apply to "all 4
conflicts like this" (same field, same source pair) — a scoped batch, ≤ Medium only.

**`approval-card/bulk-recipients`.** Critical layout for sending to many: big tabular count
("130 recipients"), sender mailbox, send window and throttle, exclusions with reasons (unsubscribed,
bounced, duplicate, flagged-not-reviewed), a sample pager of rendered messages, and the hold-to-allow
control. "What happens if I stop" line: "Unsent messages are cancelled; sent ones are not recalled."
Never offers session or always. Also used for bulk delete and PII export with the list swapped
(records + dependants, or fields included).

## 7. Data model and fixtures

### 7.1 Rust sketch (proposed, `oxyn-kits::crm`, demo-only types)
```rust
pub struct Company { pub id: CompanyId, pub name: String, pub domain: String,
    pub fields: IndexMap<FieldKey, EnrichedValue>, pub owner: Option<UserId>,
    pub people: Vec<PersonId>, pub deals: Vec<DealId> }

pub struct Person { pub id: PersonId, pub name: String, pub title: Option<String>,
    pub company: CompanyId, pub email: EnrichedValue, pub consent: Consent }

pub struct EnrichedValue {
    pub value: Option<FieldValue>,
    // Empty | Queued | Filling | Filled | Conflict | Failed | Stale | Locked | Verified
    pub state: EnrichState,
    pub sources: Vec<SourceHit>,            // every provider's answer, not just the winner
    pub chosen: Option<SourceId>,
    pub confidence: Provenance,             // Sourced | Inferred | Unverified | Calibrated(Level)
    pub fetched_at: Option<Timestamp>,
    pub written_by: Actor,                  // User(UserId) | Agent(AgentId) | Import
}
pub struct SourceHit { pub source: SourceId, pub value: FieldValue, pub at: Timestamp,
    pub snippet: Option<String>, pub url: Option<String> }

pub struct Deal { pub id: DealId, pub company: CompanyId, pub name: String, pub amount: Money,
    pub stage: StageId, pub close: Date, pub committed: bool, pub moves: Vec<StageMove> }
pub struct StageMove { pub from: StageId, pub to: StageId, pub by: Actor,
    pub reason: Option<String>, pub at: Timestamp, pub audit: AuditId, pub undone: bool }

pub enum Activity { Email(EmailRef), Call(CallRef), Meeting(MeetingRef), Note(NoteRef),
    FieldChange { field: FieldKey, old: Option<FieldValue>, new: FieldValue, by: Actor },
    StageChange(StageMove), AgentAction { summary: String, run: RunId, undo: Option<UndoToken> } }

pub enum InboxItem {                        // each links to an ApprovalRequest or a Question
    Draft { draft: DraftId, request: RequestId },
    Conflict { company: CompanyId, field: FieldKey, hits: Vec<SourceHit> },
    Updates { request_batch: BatchId, changes: Vec<FieldChange> },
    BulkSend { sequence: SequenceId, step: u8, request: RequestId },
    Question { question: QuestionId },
    RunAttention { run: RunId, reason: AttentionReason },   // Stalled, Failed{n}
}

pub struct Sequence { pub id: SequenceId, pub name: String, pub steps: Vec<SequenceStep>,
    pub enrolled: Vec<PersonId>, pub stop_on: Vec<StopRule> }
pub struct SequenceStep { pub kind: StepKind, pub day: u16, pub window: SendWindow,
    pub drafts: IndexMap<PersonId, Draft>,
    pub state: StepState }   // Drafting | Review | Approved | Sending{k,n} | Sent
```
All components still receive `&AgentState` plus their own props; these types are fixture shapes, not
library API.

### 7.2 Fixture plan
| Fixture | Content | Purpose |
|---|---|---|
| `companies.json` | 200 companies: 7 hand-written + 193 generated from a seeded word list (`seed = 1112`) | table, fan-out |
| Enrichment outcome script | 164 filled, 23 skipped (steer), 9 failed (6 no domain, 3 provider timeout), 4 conflicts (employees ×2, HQ, trucks) | flow 4.1 |
| `people.json` | 410 people, 2 per company avg; 12 unsubscribed, 2 prior bounces | sequence exclusions |
| `pipeline.json` | 34 deals over Discovery / Evaluation / Proposal / Stalled / Won / Lost; 3 agent moves today, 1 undone | board, dashboard |
| `activity.json` | 60 events on Harrowgate across 6 weeks, mixed human / agent | record timeline |
| `transcript-harrowgate.md` | 32-min call, 14 speaker turns, invented | flow 4.3 |
| `sequence-q4.json` | 4 steps, 142 enrolled, 12 flagged drafts with reasons | flow 4.5 |
| `sources.json` | Registry, Webscan, Mailcheck, Newswire, Notes with monogram colours | enrichment chips |
| Event scripts | `.oxynrun` replays of all five flows at 1×, 4× and "instant" speed | gallery, tests ([`run-replay/scrubber`](../specs/run-replay.md)) |

Rules: names, domains (`.example`), numbers and quotes are invented; no real company, person or logo;
generated names are checked against a deny-list of real brands before publishing (proposed).

## 8. Variants

### 8.1 Shell
| Variant | Layout | Surfaces | Motion | Tier |
|---|---|---|---|---|
| `shell/crm-frame` | nav + main + docked agent panel | desktop, web | subtle | Free |
| `shell/crm-frame-focus` | nav hidden, agent panel as overlay (focus mode, `panel-split/focus-mode`) | desktop, web | subtle | Pro |
| `shell/crm-frame-mobile` | bottom tabs + sheets | mobile | subtle | Free |
| `shell/crm-frame-tui` | terminal look: box-drawn tables, `approval-card/terminal`, `fanout-grid/ascii-matrix` | terminal | none | Pro |
| `shell/crm-frame-copilot` | agent panel only, embedded beside a host CRM page (web) | web | subtle | Pro |

### 8.2 Screens
| Screen | Free build | Pro build |
|---|---|---|
| Inbox | `approval-card/paged` + `genui-list/rich-rows` | `approval-card/inbox`, `genui-form/draft-review`, swipe-free mobile stack |
| Record | `genui-timeline/vertical`, `enrichment-chip/inline` | `genui-timeline/activity`, `citation/quote-peek`, `enrichment-chip/waterfall` |
| Records table | `genui-table/data-grid` + `genui-metric/strip` | `calc-footer`, `agent-columns`, `changed-cells`, `bulk-actions` |
| Pipeline | `genui-board/columns` | `genui-board/agent-moves`, `/forecast-weights` |
| Sequence editor | step list + `genui-form/stacked` + `approval-card/bulk-recipients` | `draft-review`, `diff-viewer/redline`, `fanout-grid/cells` drafting |
| Meeting prep | `markdown/chat` + `source-list/list` | `markdown/editorial`, `source-list/rail`, `citation/timestamp` |
| Dashboard | `genui-metric/strip`, `genui-chart/card` | `genui-metric/board`, `genui-chart/compare`, `trace-view/cost-rows` |

Motion levels: `none` (TUI, reduced motion), `subtle` (default: cell fills, card travel, digit roll),
`expressive` (Pro: `genui-chart/draw-on`, `border-beam/comet` on agent-editing cards). Every screen
must be complete in the free build; Pro adds depth, never a missing safety step.

## 9. Accessibility and edge cases

### 9.1 Accessibility
| Concern | Rule (proposed) |
|---|---|
| Table semantics | `role=grid`; cells announce "Employees, 410, from Registry, sourced, 3 days ago" |
| Cell loading | one polite live-region message per 2 s max: "Enriching: 142 of 200, 9 failed"; never per cell |
| Agent moves | announced politely: "Fern moved Corvid Cold Chain to Stalled: no reply in 21 days. Undo available" |
| Keyboard board | Space pick up · arrows move · Space drop · Esc cancel; `u` undo last agent move |
| Inbox | ↑/↓ items, Enter open, `Y` / `N` / `R` per approvals.md (input guard 400 ms; never from the composer) |
| Critical hold | hold is also keyboard (hold `Y` 1200 ms) and has a non-timed alternative: type the count ("130") and press Enter (proposed, for motor impairments) |
| Colour | stage, risk and confidence never rely on colour alone: glyphs ● ◐ ○ ◇ ▲ ⬣ and text labels |
| Focus order | nav → main → agent panel (F6); approvals focus Deny for High / Critical |
| Contrast | `enrichment-chip` source letters on monogram backgrounds meet ≥ 4.5:1 ([../tokens.md](../tokens.md) §8) |

### 9.2 Edge cases
| Case | Behaviour |
|---|---|
| 10,000-row table | virtualised; footer computed server-side when the app provides it ("≈" prefix if estimated) |
| Long company names / RTL names | single-line ellipsis + tooltip; RTL board reverses column order; timeline rail mirrors |
| Mixed scripts in one row | per-cell direction (`dir=auto`) |
| User edits a cell the agent is filling | user wins; cell → `Locked`; agent value lands as a conflict |
| Same record open on two devices | approvals resolved elsewhere resolve in place with provenance |
| Contact unsubscribes mid-send | removed from the remaining queue; send total updates; logged |
| Duplicate records in a list | enrichment pauses on the pair and asks to merge (High) or skip |
| Recipient time zones | send window in the recipient's zone; missing zone uses the sender's, shown in the card |
| Very fast agent (< 300 ms per cell) | no shimmer (`delay.shimmer` 600 ms); cells just fill; footer rolls once |
| Very slow agent | `Stalled` after N s per row; grid shows stalled cells in `color.state.stalled` |
| Provider credit runs out mid-run | run pauses (`BudgetReached` banner, fanout-grid); Spend approval to continue |
| Agent cites a source that later 404s | `citation` → `Orphan`; claim downgrades to Unverified |
| PII in screenshots / share | `Export` is Critical; share links of briefs mask emails and phones by default (proposed) |
| Undo after later edits | `rollback-impact` lists fields edited since; user picks keep / revert per field |

## 10. References

Study only unless the licence column says otherwise ([../licensing.md](../licensing.md)). Only pages
opened for this kit are linked; research notes are in the project research folder.

| Source | What we took (idea level) | Licence / use |
|---|---|---|
| Beautiful UI "Records Table (CRM)": tag overflow, connection strength, footer calculation row "60 count · Add calculation · 55% average" (research note 17; [R106], [R133]) | calc footer, records grid density | MIT, port with attribution |
| affectUI catalog: Decision Inbox, Email Draft (recipient + tone, inline-editable body, Send / Discard with undo), Agent Inbox, Agent Lanes, Agent Steering, CRM, Kanban Board, Lead Sourcing, Enrichment Orbit, Outreach Branches (names only for the last three) (research note 11) | inbox, draft review, steering mid-run | Paid, study only |
| Arc UI catalog: Enrich column, Reply draft, Meeting summary, Kanban board, Activity feed, Inbox triage (names, research note 12) | agent column, brief | Paid, study only |
| LangGraph Agent Inbox: accept / edit / respond / ignore (research note 05) | inbox decision set | licence not checked here; study only until checked |
| Flo Hoeller agent inbox, NEEDS YOU / RUNNING / DONE ([R062]); attention inbox dot ([R022]); Draft → Review / Approve and send ([R052]) | inbox columns, draft approval pair | X posts, study only |
| HubSpot prospecting agent docs: "Review emails" vs "Send without review", in-app notification and 9 AM daily digest of drafts awaiting review, "Regenerate all", edits to a play apply only to newly enrolled contacts — https://knowledge.hubspot.com/prospecting/use-the-prospecting-agent | review-before-send default; why a bulk grant must not stretch to new contacts | Proprietary, study only |
| Clay waterfall enrichment: sequential search across many providers until a valid match — https://www.clay.com/waterfall-enrichment | `enrichment-chip/waterfall`, keeping every source hit | Proprietary, study only |
| Attio AI page: "Ask Attio", end-of-call follow-ups and record updates, "Start every day ready" meeting prep, agents that research, enrich, route — https://attio.com/ai | flows 4.2–4.3, meeting prep screen | Proprietary, study only |
| Day AI: agents with names and jobs that report to a person; "nothing went out without your approval"; audit trail of what ran — https://day.ai | named agent ("Fern"), audit link on every agent action | Proprietary, study only |

## Open questions
1. Bulk threshold: is 25 recipients the right default for Critical, or should any send to > 1 be
   Critical? (Proposed 25, app-set.)
2. Should `FullAccess` in this kit stop at Medium (proposed) even though the generic level allows High?
3. `approval-card/bulk-recipients` and `genui-question/source-pick` as **Free** variants: family G
   and I free counts grow by one each; confirm.
4. One shared card-travel motion token for `genui-board`, `fanout-grid/inbox-board` and
   `todo-list/board` (proposed `dur.travel` 240 ms `ease.in-out` + `spring.snap` landing)?
5. Stage colours: reuse neutral columns + status hues for Won / Lost (proposed), or add
   `color.crm.stage.*` tokens?
6. Typed-count alternative to press-and-hold for Critical: does approvals.md accept it as a general
   accessibility rule?
7. Who owns credits UI: `run-meter/credits` or a CRM-specific meter?
