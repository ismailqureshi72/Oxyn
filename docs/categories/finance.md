**Status:** draft v0.1 (2026-11-12).

# Category kit: finance agents

Family M kit for the **Finance agents** row of [../component-taxonomy.md](../component-taxonomy.md)
section B ("Portfolio / ledger dashboard, reconciliation view · Transaction table, anomaly flags,
scrubbable insight charts, approval with amount & risk"). Follows the 10-section standard in
[README.md](README.md). Registry names are `shell/finance-<screen>`. Values marked *proposed* are Oxyn's
own starting points, to be tuned in the Studio. Every screen is built from families A–L; the few gaps are
listed in §6.

**Hard rules for this kit**
1. **Money never moves without a Critical stop.** Any payment, transfer, payroll run or tax filing is a
   `Critical` request: press-and-hold 1200 ms, "Allow once" only, never auto-approved at any permission
   level, Bypass included ([../specs/approvals.md](../specs/approvals.md) §2.3).
2. **A money approval always shows amount, payee, account and risk** without a click: amount and
   currency in large tabular figures, payee name, the account debited and the payee account (masked),
   the risk tier glyph + label + reasons.
3. **Gains and losses are never hue only.** Sign, glyph and words carry direction; colour is the
   redundant fourth channel ([../tokens.md](../tokens.md) §9).
4. **Every number the agent writes can be traced** to the transactions behind it (citations, §3.6).

---

## 1. Who it's for

| | |
|---|---|
| **User** | Owner-operator or part-time bookkeeper of a small business; an outside accountant reviews at month end |
| **Job** | "Keep the books right and current, tell me what looks wrong, pay the bills I approve, and hand me a close pack I can sign" |
| **What the agent does** | Categorises transactions, reconciles the bank statement, flags anomalies and explains them, prepares bill payment runs, runs the month-end checklist, drafts the close memo with citations |
| **What the agent never does alone** | Move money, change who gets paid, lock or reopen a period, file anything with a tax authority |
| **Peers studied** | Ramp Accounting Agent, Digits, Rillet, Puzzle AI Close, Brex agents (§10) |

### Demo company (invented)
**Kettle & Kiln Ceramics Ltd**, a 9-person pottery studio and online shop in an invented market town
("Wexmoor"). Books in **GBP**. Fiscal year = calendar year. The demo period is **October 2026**, closed in
the first week of November.

| Fixture | Value |
|---|---|
| Bank | "Harbourline Bank" current account `•• 4417`, savings `•• 9020` |
| Card | "Harbourline Business Card" `•• 7781` |
| Foreign supplier | **Brightwick Kilns GmbH**, invoices in **EUR** (multi-currency edge case) |
| Marketplace | "Makersquare" payouts in **USD**, converted on receipt (second currency edge case) |
| Accountant | Fenwick & Ash Accountants (outside reviewer, read-only role) |
| People | Owner "Rosa Adeyemi-Clarke" (user), bookkeeper "Tom Hale" (second user, for audit "by whom") |

All names, figures and logos are invented; any match with a real business is accidental.

---

## 2. Shell

One persistent frame for every screen: **nav rail · top bar · canvas · agent panel · status bar**.
The agent panel is a chat thread plus composer that stays docked while the canvas changes, so the user can
ask "why?" about whatever is on screen.

### 2.1 Desktop (GPUI) and wide web (Dioxus), ≥ 1440 px

```text
+--------------------------------------------------------------------------------------------------+
| (K) Kettle & Kiln v | [Oct 2026 · Open v] |  ⌘K Search or ask...          | ⏸ 2 need you | (RA) |
+----+------------------------------------------------------------+--------------------------------+
| ⌂  |  CANVAS (current screen)                                   |  AGENT                     ⋯  |
| ⇄  |                                                            |  ┌──────────────────────────┐ |
| ≡  |                                                            |  │ thread: messages, tool    │ |
| ✓  |                                                            |  │ rows, plan card, approval │ |
| ⧉  |                                                            |  │ cards, citations          │ |
| ✎  |                                                            |  └──────────────────────────┘ |
| ⌸  |                                                            |  [Ask ▾] [Ask first ▾]        |
|    |                                                            |  ┌──────────────────────────┐ |
| ⚙  |                                                            |  │ Ask about October...   ⏎ │ |
+----+------------------------------------------------------------+--------------------------------+
| ● Bank feed synced 09:12 · Categoriser idle · 1 job running · Oct: 23 to review · £0.41 run cost |
+--------------------------------------------------------------------------------------------------+
  rail: ⌂ Overview  ⇄ Transactions  ≡ Reconcile  ✓ Close  ⧉ Bills  ✎ Reports  ⌸ Audit  ⚙ Rules
```

| Region | Size | Components |
|---|---|---|
| Nav rail | 56 px, icons + tooltip; 200 px expanded | [side-panels.md](../specs/side-panels.md) `side-panel/rail`; counts as badges ("23") |
| Top bar | 48 px | company switcher, **period chip** (new, §6), [command-palette.md](../specs/command-palette.md) `command-palette/answer-cards`, approvals badge → [desktop-overlays.md](../specs/desktop-overlays.md) `tray-popover/approvals-only` |
| Canvas | fluid, min 720 px | the screen (§3) |
| Agent panel | 400 px (resizable 320–560) | [message-bubble.md](../specs/message-bubble.md) `message/flat`, [tool-call.md](../specs/tool-call.md) `tool-call/row`, [plan-todo.md](../specs/plan-todo.md), [approval-card.md](../specs/approval-card.md), composer below |
| Composer | in panel, bottom | [composer-input-shell.md](../specs/composer-input-shell.md) `composer-input/rounded-box`, [composer-mode-switch.md](../specs/composer-mode-switch.md) `mode-switch/chip-menu` (Ask / Plan / Agent), [composer-permission-selector.md](../specs/composer-permission-selector.md) `permission-selector/risk-ladder`, [composer-slash-mentions.md](../specs/composer-slash-mentions.md) |
| Status bar | 24 px | [side-panels.md](../specs/side-panels.md) `status-bar/run-strip`, [background-jobs.md](../specs/background-jobs.md) `background-jobs/dock-pill`, [run-meter.md](../specs/run-meter.md) |

Slash commands (in-house copy): `/categorise`, `/reconcile`, `/explain`, `/pay`, `/close`, `/memo`.
Mentions: `@payee`, `@account`, `@txn` (a transaction chip), `@period`.

### 2.2 Breakpoints (proposed; no breakpoint tokens exist yet)

| Width | Layout |
|---|---|
| ≥ 1440 | three columns as above |
| 1024–1439 | rail icons only; agent panel collapses to a 48 px tab on the right edge, opens as an overlay sheet (`panel-split/peek`); approvals still pop the panel open |
| 640–1023 | rail becomes a top tab strip; agent panel is a bottom sheet (`panel-split/mobile-sheets`); tables switch to `genui-table/compact-records` |
| < 640 | mobile layout (§2.3) |

### 2.3 Narrow / mobile

```text
+------------------------------+      +------------------------------+
| Kettle & Kiln   Oct 2026 ▾   |      | ← Pay 5 bills       ⬣ Crit. |
+------------------------------+      +------------------------------+
| Cash        £62,480.12       |      |  £13,642.40  + €3,200.00     |
| ▲ +£4,102.55 vs Sep          |      |  from Harbourline •• 4417    |
|------------------------------|      |------------------------------|
| ⏸ Needs you (2)              |      | Moorfield Clay     £4,812.00 |
|  ⬣ Pay 5 bills · £13.6k      |      | Tallow Lane ▲68%   £1,936.40 |
|  ◇ 8 unmatched lines         |      | Brightwick (EUR)   €3,200.00 |
|------------------------------|      | Fenwick & Ash      £1,450.00 |
| ⚑ 2 anomalies                |      | Ridgeway ⚠ new a/c £5,444.00 |
|------------------------------|      | ☐ Quayside (dup?)  £1,284.00 |
| [ Ask about October...  ⏎ ]  |      |------------------------------|
+------------------------------+      |   ( Hold to pay · 1.2 s )    |
| ⌂ Home  ⇄ Activity  ⏸  ✦ Agent |    |   [ Deny ]  [ Tell agent ]   |
+------------------------------+      +------------------------------+
```

Mobile is **approve-first**: the "Needs you" list is the home screen, because most mobile sessions are
"approve the payment run" or "answer the agent's question". Approvals open as
`approval-card/modal` (full sheet, focus on Deny); Critical hold uses haptics (approvals.md §4). Swipe is
never an approval gesture.

### 2.4 Terminal look (Pro)
A TUI variant (`shell/finance-terminal`) for bookkeepers who live in a terminal: `genui-table/box-drawn`,
`approval-card/terminal`, `risk-badge/glyph` (`○ ◇ ▲ ⬣`), `genui-chart/braille` for sparklines. Hold-to-allow
becomes "hold Y for 1.2 s" (approvals.md §8).

---

## 3. Screens

Every screen shares: the period chip scopes all data; the agent panel can be asked about anything
selected ("Explain this row" sends an `@txn` chip); agent-changed values get the "agent changed this"
wash (`dur.highlight.fade`, 1600 ms, [../motion.md](../motion.md)).

| # | Screen | Registry | Main job |
|---|---|---|---|
| 3.1 | Overview (dashboard) | `shell/finance-overview` | Read the month at a glance; see what needs you |
| 3.2 | Transactions | `shell/finance-transactions` | Review agent categorisation; act on flags |
| 3.3 | Reconcile | `shell/finance-reconcile` | Match bank statement to ledger; clear unmatched |
| 3.4 | Close | `shell/finance-close` | Run and sign off month-end tasks |
| 3.5 | Bills & payments | `shell/finance-bills` | Approve payment runs |
| 3.6 | Reports | `shell/finance-reports` | Generate the close memo / P&L with citations |
| 3.7 | Audit | `shell/finance-audit` | See who (person or agent) did what, when, and why |
| 3.8 | Rules | `shell/finance-rules` | Permission rules and categorisation rules |

### 3.1 Overview

```text
+------------------------------------------------------------------------------------------+
| October 2026 · Open                                         [ Ask: "Why is margin down?" ]|
+----------------------+----------------------+----------------------+----------------------+
| Cash                 | Revenue              | Expenses             | Net income           |
| £62,480.12           | £48,210.00           | £41,377.80           | £6,832.20            |
| ▲ +£4,102.55 vs Sep  | ▲ +6.1% vs Sep       | ▲ +11.4% vs Sep  ⚑   | ▼ −18.9% vs Sep      |
| ▁▂▃▃▄▅▆▆▇            | ▃▄▄▅▅▆▆▇▇            | ▃▃▄▄▅▅▆▇█            | ▇▇▆▆▅▅▄▄▃            |
+----------------------+----------------------+----------------------+----------------------+
| Revenue vs expenses, Jan–Oct 2026                         [line|bar]  [T table] [⋯]       |
|  £50k ┤                                         ╭──● Revenue                              |
|       │                         ①               ╭─╯                                       |
|  £40k ┤      ╭───╮       ╭────────╮      ╭─────╯ ╭──● Expenses   ② Utilities spike        |
|       │ ╭────╯   ╰───────╯        ╰──────╯  ╭────╯                                        |
|  £30k ┼─┴────┬────┬────┬────┬────┬────┬────┬┴───┬────┬                                    |
|       Jan  Feb  Mar  Apr  May  Jun  Jul  Aug  Sep  Oct     drag to select a range          |
|  ① Kiln repair (Jun) · ② Utilities +68% (Oct)  [1 of 2 ‹ ›]                                |
+----------------------------------------------+-------------------------------------------+
| Needs you                                    | Close progress  Oct 2026                  |
|  ⬣ Pay 5 bills · £13,642.40 + €3,200.00      |  ███████████░░░░  7 / 11 tasks · BD+4     |
|  ⚑ Possible duplicate · Quayside £1,284.00   |  ✓ Categorise  ✓ Bank rec  ◐ Accruals     |
|  ◇ 8 unmatched statement lines               |  ○ Flux review  ○ Memo  ○ Lock period     |
|  ? Agent asks: "Studio lease — capex?"       |                                [Open →]   |
+----------------------------------------------+-------------------------------------------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| KPI tiles | [genui-blocks.md](../specs/genui-blocks.md) `genui-metric/board` (Pro) or `genui-metric/tile` (free) with `genui-metric/sparkbar` | `direction_good`: up for cash/revenue/net, **down** for expenses; delta = sign + ▲/▼ + % (§3.0 number rules) |
| Insight chart | `genui-chart/scrub-insight` (Pro), `genui-chart/annotated-story` (Pro), free fallback `genui-chart/plain` | Agent annotations are numbered pins; "1 of 2 ‹ ›" steps through them; dragging a range sends "Explain Aug–Oct" to the agent |
| Needs-you list | [genui-blocks.md](../specs/genui-blocks.md) `genui-list/rich-rows`; risk glyphs via [approvals.md](../specs/approvals.md) `risk-badge/shape` | Click opens the card in the agent panel ([approval-card.md](../specs/approval-card.md) `approval-card/paged`) |
| Agent question | [genui-forms.md](../specs/genui-forms.md) `genui-question/chips` | "(recommended)" marker on the agent's preferred answer |
| Close progress | [plan-todo.md](../specs/plan-todo.md) `todo-list/capsules` | links to 3.4 |

| State | Shows |
|---|---|
| Empty (no bank connected) | [citations-sources.md](../specs/citations-sources.md) `empty-state/starters-grid`: "Connect a bank feed", "Import a CSV", "Try with demo data" |
| Loading | [genui-skeletons.md](../specs/genui-skeletons.md) `genui-skel/kpi-morph-in`, `genui-skel/line-chart-draw`, `genui-skel/list-stream-fill`; first reveal counts up (`dur.count.up` 500 ms) |
| Error | per-tile error at the same footprint ("Couldn't load cash · Retry"); other tiles stay live |
| Stale feed | top-bar chip "Bank feed 3 days old" in warning glyph ◇ + text; KPIs show "as of 9 Nov" |

#### 3.0 Number rules (apply to every screen)
| Rule | Detail |
|---|---|
| Tabular figures | `tnum` everywhere a number sits in a column, tile or approval ([../tokens.md](../tokens.md) "Tabular figures") |
| Minus | U+2212 `−`, never hyphen; optional accounting style `(1,284.00)` per app setting |
| Alignment | numbers right-aligned on the decimal; fixed minor units per currency (GBP/EUR/USD 2, JPY 0) |
| Currency | symbol when the whole view is one currency; **ISO code** (`EUR 3,200.00`) whenever a view mixes currencies; never add mixed currencies without a stated rate and date |
| Direction | sign + glyph (▲ / ▼) + word in the accessible name ("up 6.1 percent"); colour from proposed `color.data.gain` / `color.data.loss` (aliases of `color.status.success` / `color.status.danger`), with a daltonized pair (blue / orange) in CVD themes |
| Good vs bad | `direction_good` decides tone, not the arrow: expenses ▲ is drawn in the loss tone |
| Compact | `£13.6k` only in tight tiles and mobile lists; full value in tooltip and accessible name |
| Changes | values roll on update (`dur.digit.roll` 280 ms, `stagger.digit` 30 ms); reduced motion swaps |

### 3.2 Transactions

```text
+------------------------------------------------------------------------------------------+
| Transactions · Oct 2026   [All 312] [To review 23] [Flagged 2] [Agent-posted 289]   ⌕    |
| Agent categorised 289 of 312 · 23 below 90% confidence                [Accept all ≥ 90%] |
+---+------------+-------------------------+----------------------+-----------+------------+
| ☐ | Date       | Payee / description      | Category (agent)     | Conf.     |     Amount |
+---+------------+-------------------------+----------------------+-----------+------------+
| ☐ | 28 Oct     | Moorfield Clay Supply   | COGS · Clay ✦        | ●●●● 98%  | −£2,406.00 |
| ☐ | 27 Oct     | Makersquare payout      | Sales · Marketplace ✦| ●●●● 99%  | +£3,118.42 |
|   |            |   USD 3,987.10 @ 0.7821 |                      |           |            |
| ☐ | 24 Oct     | Quayside Packaging  ⚑   | COGS · Packaging ✦   | ●●●○ 91%  | −£1,284.00 |
|   |            |   ⚑ Possible duplicate of 09 Oct · same invoice QP-2291  [Explain] [Dismiss]|
| ☐ | 22 Oct     | "TAXPAY 0193"           | ? Uncategorised      | ●○○○ 34%  |   −£612.00 |
|   |            |   Agent: "Looks like a VAT payment; confirm?"  [VAT liability] [Other ▾]    |
| ☐ | 19 Oct     | Ridgeway Studio Lease   | Rent ✦               | ●●●○ 88%  | −£5,444.00 |
+---+------------+-------------------------+----------------------+-----------+------------+
| 3 selected · [Accept 3] [Recategorise ▾] [Create rule…] [Ask agent]       Σ −£9,134.00   |
+------------------------------------------------------------------------------------------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| Table | [genui-blocks.md](../specs/genui-blocks.md) `genui-table/data-grid` (free) · `genui-table/bulk-actions` + `genui-table/calc-footer` + `genui-table/changed-cells` (Pro) | cells: `date`, `text`, `status`, `currency { code }`; virtualised > 200 rows; Σ footer recomputes on filter |
| Filter chips with counts | `follow-ups/chips` styling, counts roll | |
| Category cell | ✦ = set by agent; hover shows why ([reasoning-block.md](../specs/reasoning-block.md) `reasoning-block/inline-summary`) and "similar past transactions" | |
| Confidence | [citations-sources.md](../specs/citations-sources.md) `confidence-badge/meter` (Pro) / `confidence-badge/label` (free: "Sure" / "Likely" / "Unsure") | 4 dots + % text; never colour only |
| Anomaly flag | **`anomaly-flag`** (new, §6) chip ⚑ on the row + expandable explanation line | |
| Inline question | [genui-forms.md](../specs/genui-forms.md) `genui-question/chips` | answer is learned (memory receipt `memory-receipt/footer-line`: "I'll file TAXPAY as VAT liability from now on · Undo") |
| Bulk bar | `genui-table/bulk-actions` | "Accept all ≥ 90%" is one **Medium** `Edit`-like request (posts to ledger), batched per approvals.md §2.6 |
| Multi-currency row | second line shows original amount, rate and rate date | |

States: **Empty** "No transactions in Oct 2026 · Change period" · **Loading** `genui-skel/table-stream-fill`
· **Streaming** while categorising, rows fill with `Filling(k of n)` caption "Categorising 140 of 312"
· **Error** "Couldn't load transactions · Retry" · **Locked period** read-only banner "Oct 2026 is locked ·
Reopen (High risk)".

### 3.3 Reconcile

```text
+------------------------------------------------------------------------------------------+
| Reconcile · Harbourline •• 4417 · Statement 1–31 Oct 2026                                |
| Statement £58,377.57  Ledger £59,004.26  Difference −£626.69   239 / 247 matched  [Finish]|
+-------------------------------------------+---+------------------------------------------+
| BANK STATEMENT                            |   | LEDGER                                   |
+-------------------------------------------+---+------------------------------------------+
| 24 Oct  QUAYSIDE PKG       −£1,284.00     |═══| 24 Oct Quayside Packaging    −£1,284.00  |
| 22 Oct  TAXPAY 0193          −£612.00     |   |   (no ledger match)                      |
|          ✦ Suggest: create "VAT payment"  ┆   ┆  [Create entry] [Find match] [Ignore]     |
| 21 Oct  CARD FEES OCT         −£41.18     |┄┄┄| 31 Oct Bank charges           −£41.18 ✦  |
|          ✦ 87% · date differs by 10 days  ┆   ┆  [Accept] [Reject]                       |
| 14 Oct  BRIGHTWICK KILNS   −£2,781.44     |┄┄┄| 14 Oct Brightwick (EUR 3,200) −£2,784.00 |
|          ✦ 79% · FX difference £2.56      ┆   ┆  [Accept + post FX £2.56] [Reject]       |
| 03 Oct  MAKERSQ USD        +£3,101.17     |┄┄┄| 03 Oct Makersquare payout +£3,118.42 ✦   |
|          ✦ 74% · £17.25 fee withheld?     ┆   ┆  [Split: payout + fee] [Reject]          |
+-------------------------------------------+---+------------------------------------------+
| ═══ matched   ┄┄┄ agent suggestion (dashed, with %)   unmatched: no line                 |
+------------------------------------------------------------------------------------------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| Pair view | **`reconcile-pairs`** (new, §6) | two columns, link lines between rows; solid = matched, dashed = suggestion with confidence text |
| Header totals | `genui-metric/strip` | difference rolls toward £0.00 as items clear; "Finish" disabled with reason until difference = 0 or every unmatched line has a reason |
| Suggestions | `confidence-badge/label` + reason line ("date differs by 10 days") | |
| Actions | Accept (Low), Split (Medium), Create entry (Medium), Post FX difference (Medium) — via `approval-card/footer` when the level asks | |
| Statement upload | [composer-attachments.md](../specs/composer-attachments.md) `attachment/scan-highlight` while the PDF is read | |

States: **Empty** "No statement for Oct · Upload PDF or wait for feed" · **Loading** two-column
`genui-skel/table-morph-in` + link lines drawn last · **Done** difference £0.00 → ✓ draws
(`dur.check.draw` 240 ms), task ticks in Close · **Error** "Statement couldn't be read (page 3) · Retry ·
Enter manually".

### 3.4 Close

```text
+------------------------------------------------------------------------------------------+
| Close · October 2026        due BD+5 (7 Nov)        7 / 11 done        [Run remaining ▶] |
+----+------------------------------------+-------------+-----------+----------+-----------+
|    | Task                               | Owner       | Status    | Evidence | Sign-off  |
+----+------------------------------------+-------------+-----------+----------+-----------+
| ✓  | Categorise all transactions        | ✦ Categoriser| Done 0:42 | 312 txns | Tom H.    |
| ✓  | Reconcile Harbourline •• 4417      | ✦ Reconciler | Done 3:10 | rec pack | Tom H.    |
| ✓  | Reconcile card •• 7781             | ✦ Reconciler | Done 1:05 | rec pack | Tom H.    |
| ◐  | Accruals and prepaids              | ✦ Accruals   | Running ⠼ | —        | —         |
| ⏸  | Review anomalies (2)               | Rosa A-C     | Needs you | 2 flags  | —         |
| ○  | FX revaluation (EUR, USD)          | ✦ Accruals   | Waiting   | —        | —         |
| ○  | Flux review vs Sep (> £500 / 10%)  | ✦ Reporter   | Waiting   | —        | —         |
| ○  | Draft close memo                   | ✦ Reporter   | Waiting   | —        | —         |
| ○  | Accountant review                  | Fenwick & Ash| Waiting   | —        | —         |
| ○  | Lock October 2026                  | Rosa A-C     | Waiting   | —        | ▲ High    |
+----+------------------------------------+-------------+-----------+----------+-----------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| Checklist | **`todo-list/close-checklist`** (new variant of [plan-todo.md](../specs/plan-todo.md) `todo-list`, §6), built on `todo-list/owner-lanes` | owner = agent (identity colour + name) or person; evidence links; sign-off column |
| Running task | [loaders.md](../specs/loaders.md) glyph loader + [elapsed-timer.md](../specs/elapsed-timer.md) `elapsed-timer/units` | |
| Agent tree | [subagent-tree.md](../specs/subagent-tree.md) `subagent-tree/indented` in the agent panel: Close agent → Categoriser, Reconciler, Accruals, Reporter | |
| Run remaining | [plan-approval.md](../specs/plan-approval.md) — the run is a plan; approving it does not pre-approve its gated steps | |
| Lock period | `approval-card/inline` High (§5) | |
| Scheduled | [background-jobs.md](../specs/background-jobs.md) `background-jobs/scheduled`: "Close prep runs on BD+1 at 07:00" | |

States: **Empty** (new period) "Start October close · Use last month's checklist" · **Blocked** task shows
⊘ + reason ("Waiting on bank rec") · **Overdue** BD count turns to warning glyph ◇ + "1 day late".

### 3.5 Bills & payments

```text
+------------------------------------------------------------------------------------------+
| Bills · Due in next 30 days                      [Upcoming 6] [Scheduled 0] [Paid 14]    |
+---+---------------------------+------------+-------------+----------+-------------------+
| ☑ | Payee                     | Due        |      Amount | Risk     | Agent note        |
+---+---------------------------+------------+-------------+----------+-------------------+
| ☑ | Moorfield Clay Supply     | 14 Nov     |   £4,812.00 | ○        | usual amount      |
| ☐ | Quayside Packaging    ⚑   | 15 Nov     |   £1,284.00 | ▲        | possible duplicate|
| ☑ | Tallow Lane Utilities     | 18 Nov     |   £1,936.40 | ◇        | +68% vs 3-mo avg  |
| ☑ | Brightwick Kilns GmbH     | 19 Nov     | EUR 3,200.00| ◇        | FX ≈ £2,784.00    |
| ☑ | Fenwick & Ash Accountants | 20 Nov     |   £1,450.00 | ○        | usual amount      |
| ☑ | Ridgeway Studio Lease     | 01 Dec     |   £5,444.00 | ▲        | bank details new  |
+---+---------------------------+------------+-------------+----------+-------------------+
| 5 selected · GBP £13,642.40 + EUR 3,200.00 · from Harbourline •• 4417  [Prepare run ▶]   |
+------------------------------------------------------------------------------------------+
```

"Prepare run" asks the agent to build a payment run; the run is one **Critical** request rendered with
the new variant **`approval-card/payment-run`** (§6) in the agent panel or as `approval-card/modal`:

```text
+-------------------------------------------------------------------------------+
| ⬣ CRITICAL · Pay 5 bills                                     1 of 1 · ⏱ —     |
|///////////////////////////////////////////////////////////////////////////////|
|   £13,642.40   +   EUR 3,200.00  (≈ £2,784.00 @ 0.8700, rate 12 Nov 09:00)    |
|   From  Harbourline current •• 4417   balance after ≈ £46,053.72              |
+-------------------------------------------------------------------------------+
| ☑ Moorfield Clay Supply      → •• 2210               £4,812.00   ○           |
| ☑ Tallow Lane Utilities      → •• 5530                £1,936.40   ◇ +68%      |
| ☑ Brightwick Kilns GmbH      → IBAN DE•• •• 8841   EUR 3,200.00   ◇ FX        |
| ☑ Fenwick & Ash Accountants  → •• 0917                £1,450.00   ○           |
| ☑ Ridgeway Studio Lease      → •• 6604 (NEW 3 Nov)   £5,444.00   ▲ new a/c   |
| ☐ Quayside Packaging         → •• 3381                £1,284.00   ▲ excluded  |
+-------------------------------------------------------------------------------+
| Why: "These five are due before 1 Dec; Quayside left out (possible duplicate)."|
| Risk: ⬣ moves money · ▲ 1 payee with bank details changed in the last 30 days |
+-------------------------------------------------------------------------------+
|   (  ◔  Hold to pay 5 bills · 1.2 s  )     [ Deny ]   [ Deny and tell… ]      |
+-------------------------------------------------------------------------------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| Bills table | `genui-table/data-grid` + `genui-table/bulk-actions` | Risk column uses `risk-badge/glyph`; per-row reasons in tooltip |
| Payment-run card | **`approval-card/payment-run`** (new variant, §6), built on `approval-card/amount` + `approval-card/hold-ring` | Unticking a line is an `AllowEdited` before the hold; totals roll; hold length never changes |
| Result | [tool-call.md](../specs/tool-call.md) `tool-call/row` per payment: Submitted → Sent → Settled / Failed | each with bank reference; failure keeps the others' status |
| Undo | none for sent payments; [rollback-impact.md](../specs/rollback-impact.md) `rollback-impact/compensations` lists the compensating action ("Request refund from payee") | |

States: **Empty** "No bills due · Upload a bill" · **Loading** `genui-skel/table-stream-fill` ·
**Partial failure** "4 of 5 sent · Brightwick failed: IBAN rejected · Retry (new Critical request)" ·
**Offline** hold disabled with reason "Offline · payment can't reach the bank" (fail safe, approvals.md §2.1).

### 3.6 Reports

```text
+-------------------------------------------------------------------------------+------------+
| Close memo · October 2026                       [Regenerate] [Export PDF] [⋯] | SOURCES    |
+-------------------------------------------------------------------------------+------------+
| Net income was £6,832.20, down 18.9% on September [1]. Revenue grew 6.1% to   | [1] P&L    |
| £48,210.00, led by marketplace sales [2]. Expenses rose 11.4%, mostly utilities| Oct vs Sep |
| (+£783.78, +68% vs the 3-month average) [3] and one-off packaging [4].        | [2] 41 txns|
|                                                                               |  Makersq.  |
| ┌ Expenses by category, Oct vs Sep ────────────────────────────┐              | [3] 2 txns |
| │ Clay & glazes   ██████████████▏ £14,210  ▲ +4.2%              │              |  Tallow L. |
| │ Wages           ████████████████████ £18,900  ━ 0.0%          │              | [4] 1 txn  |
| │ Utilities       ████▏ £1,936  ▲ +68% vs avg ⚑                  │              |  Quayside  |
| └───────────────────────────────────────────────────────────────┘              |            |
| Open items: 1 possible duplicate (£1,284.00) under review [4].                | Confidence |
|                                                                               | Sourced 9  |
|                                                                               | Inferred 1 |
+-------------------------------------------------------------------------------+------------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| Memo body | [streaming-markdown.md](../specs/streaming-markdown.md) `markdown/editorial` | streams in; numbers in tabular figures |
| Citations | [citations-sources.md](../specs/citations-sources.md) `citation/number` + `citation/quote-peek` (Pro) | **new source kind `Record { ledger, ids }`** (§6): preview shows the transactions as a mini table, "Open in Transactions" filters 3.2 to them |
| Sources rail | `source-list/rail` | grouped by report, transaction set, statement |
| Claim confidence | `confidence-badge/label` ("Sourced" / "Inferred") per sentence; inferred claims underlined (`confidence-badge/underline`, Pro) | |
| Embedded charts | `genui-chart/plain` bar (horizontal) with `genui-chart/compare` (Pro) | "View as table" (`T`) |
| Report picker | [genui-forms.md](../specs/genui-forms.md) `genui-form/inline`: report type (Close memo, P&L, Cash flow, Flux), period, compare-to | |

States: **Empty** "Pick a report" · **Generating** `genui-skel/text-line-by-line` with source cards
arriving first (`source-list/loading-first`) · **Stale** "Ledger changed since this memo · Regenerate"
banner (the memo keeps its old numbers until regenerated; changed numbers listed) · **Error** per block.

### 3.7 Audit

```text
+------------------------------------------------------------------------------------------+
| Audit · Oct 2026     [All] [Money moved] [Agent actions] [Approvals] [Edits]   Export ▾  |
+-------------+------------------------------------------+---------------+-----------------+
| When        | What                                     | By            | Decision        |
+-------------+------------------------------------------+---------------+-----------------+
| 12 Nov 10:04| Paid 5 bills · £13,642.40 + EUR 3,200.00  | Rosa A-C (hold)| ⬣ Allowed once |
| 12 Nov 10:01| Excluded Quayside £1,284.00 from run      | Rosa A-C      | Edited          |
| 06 Nov 16:20| Reconciled •• 4417 · 247 lines · £0.00    | ✦ Reconciler  | ◇ Allowed (rule)|
| 06 Nov 15:58| Posted FX difference £2.56                | ✦ Reconciler  | ◇ Allowed once  |
| 05 Nov 09:12| Categorised 289 transactions             | ✦ Categoriser | ○ Auto (≥ 90%)  |
| 05 Nov 09:13| Flagged duplicate Quayside QP-2291        | ✦ Categoriser | —               |
+-------------+------------------------------------------+---------------+-----------------+
| ▸ detail: request snapshot · risk reasons · waited 0:41 · origin: desktop · run replay ▶ |
+------------------------------------------------------------------------------------------+
```

| Part | Component (spec) | Notes |
|---|---|---|
| Log | [permission-rules.md](../specs/permission-rules.md) `audit-log/table` (dense) or `audit-log/ledger` (grouped by run) | append-only; "By" distinguishes person, agent, rule, level |
| Detail | `audit-log/timeline` detail sheet | request snapshot with masked accounts |
| Replay | [run-replay.md](../specs/run-replay.md) `run-replay/step-list` | "what the agent saw and did" for one run |
| Export | CSV / JSON callback; read-only role for the accountant | |

### 3.8 Rules
[permission-rules.md](../specs/permission-rules.md) `permission-rules/table` + `permission-rules/simulator`
("would this rule have auto-posted last month's 312 transactions?"). Categorisation rules the agent learned
appear as `permission-rules/suggestions` ("Always file TAXPAY as VAT liability?"). Money rules are
**not offered**: no "always allow" exists for Spend (approvals.md §2.4).

---

## 4. Agent flows

Each flow names the `AgentState` ([../architecture.md](../architecture.md)), the component showing it,
where the run stops, and what the user can steer. Verbs come from a **finance verb pack** (in-house copy,
§6): "Categorising", "Matching", "Totting up", "Checking the statement", "Drafting".

### 4.1 Categorise the month's transactions

| # | AgentState | What the user sees | Component |
|---|---|---|---|
| 1 | `Composing` → `Submitted` | "/categorise October" | composer, send-control "submitted" pulse |
| 2 | `ToolRunning { tool: ledger.fetch, verb: "Fetching" }` | tool row "Fetched 312 transactions" | `tool-call/row` |
| 3 | `ToolRunning { tool: categorise, verb: "Categorising" }` | table fills; caption "Categorising 140 of 312"; status strip shows rate | `genui-table/stream-rows`, `status-text/step-label` |
| 4 | `AwaitingApproval` (Medium, batch) — only at `Ask` level | "Post 289 at ≥ 90%?" Allow once / for session | `approval-card/footer` |
| 5 | `Streaming` | summary: "289 posted · 23 need you · 2 flags" + follow-ups | `message/flat`, `follow-ups/chips` |
| 6 | `AwaitingApproval { Question }` | inline questions for the lowest-confidence items | `genui-question/paged` ("1 / 5") |
| 7 | `Done { elapsed }` | "Worked for 42s" | `status-text/line` |

**Steer:** while running, the user types "treat all Makersquare fees as Selling fees" → `Steering`
(`composer-queue-stack`) and the remaining rows use it. **Stop** keeps what is posted (each post is
audited). **Error** `ledger.fetch` fails → `Error { kind: Source }`, Retry; partial results stay.
**Fast agent:** 312 rows in < 1 s → rows appear in one batch with a single wash, no per-row animation.

### 4.2 Reconcile the bank statement (unmatched items)

| # | AgentState | What the user sees | Component |
|---|---|---|---|
| 1 | `Submitted` | user drops `harbourline-oct.pdf` | `attachment/scan-highlight` |
| 2 | `ToolRunning { statement.read }` | "Read 247 lines (4 pages)" | `tool-call/row` |
| 3 | `ToolRunning { match, verb: "Matching" }` | pairs link up top-down; difference rolls | `reconcile-pairs`, `genui-metric/strip` |
| 4 | `Streaming` | "239 matched. 8 left: 4 suggestions, 3 need a new entry, 1 unknown" | message + `genui-list/checklist` |
| 5 | `AwaitingApproval` (Medium each) | "Post FX difference £2.56", "Split Makersquare payout + £17.25 fee" | `approval-card/paged` "1 of 4"; batch "Allow all 4 once" allowed (all ≤ Medium, one agent) |
| 6 | `AwaitingApproval { Question }` | "What is TAXPAY 0193 (−£612.00)?" options + (recommended) | `genui-question/chips` |
| 7 | `Done` | Difference £0.00 ✓, Close task ticks | `reconcile-pairs` done state, `todo-list/close-checklist` |

**Steer:** reject a suggestion with a reason ("that fee is October's, not November's") → `DenyWithReason`,
the agent re-matches. **Stalled:** bank PDF parse hangs > N s → `Stalled { since }`, strip ramps to warning
(`status-text/stall-ramp`).

### 4.3 Detect anomaly → explain → user decides

| # | AgentState | What the user sees | Component |
|---|---|---|---|
| 1 | (background) `ToolRunning { anomaly.scan }` | nothing intrusive; status bar "Checking for anomalies" | `background-jobs/dock-pill` |
| 2 | — | ⚑ on the Quayside row and Overview "Needs you"; polite announcement | `anomaly-flag` chip |
| 3 | user clicks **Explain** → `Thinking { verb: "Checking" }` | reasoning collapsed | `reasoning-block/inline-summary` |
| 4 | `Streaming` | "Same payee, same amount, same invoice no. QP-2291, 15 days apart [1][2]. The 24 Oct payment is likely a duplicate." | `anomaly-flag` card + `citation/number` (Record sources) + `confidence-badge/label` "Likely" |
| 5 | `AwaitingApproval { Question }` | **Mark duplicate and request refund** (recommended) · Not a duplicate · Ask Quayside (draft email) | `genui-question/chips` |
| 6 | → choice "Request refund" | drafts an email to the payee (outbound message = approval, per approval prefs) | `genui-form/draft-review`, `approval-card/inline` (Medium) |
| 7 | `Done` | flag resolves to "Refund requested · 12 Nov"; Quayside line stays excluded from future runs until resolved | `anomaly-flag` resolved row |

**Never automatic:** the agent may flag and explain; it never reverses, voids or blocks a payee on its own.
"Not a duplicate" teaches the scanner (memory receipt with Undo).

### 4.4 Pay a batch of bills (Critical)

| # | AgentState | What the user sees | Component |
|---|---|---|---|
| 1 | `Submitted` | "Pay the 5 selected bills" (from 3.5) | bulk bar |
| 2 | `ToolRunning { payments.prepare, verb: "Preparing" }` | checks: balance, due dates, duplicates, payee account history, FX quote | `tool-call/row` ×4, `plan-todo` checklist |
| 3 | `AwaitingApproval { Spend, Critical }` | payment-run card (§3.5): totals per currency, from-account, each payee + masked account + amount + risk; FX rate and its time | `approval-card/payment-run` |
| 4 | user unticks a line | `AllowEdited` draft: totals roll; hold length unchanged | — |
| 5 | user holds 1200 ms | ring fills (linear); early release drains (`dur.approval.drain`) | `approval-card/hold-ring` |
| 6 | `ToolRunning { payments.send }` | per-payment rows: Submitted → Sent | `tool-call/row` per payee |
| 7 | `Done` or partial `Error` | "5 sent · refs HB-…"; a failure shows its own Retry, which is a **new** Critical request | `tool-call/row`, `approval-card` |

Rules: the FX quote **expires** (`expires_at` = quote time + 10 min, *proposed*); expiry → `Expired`, never
approval; "Ask again" fetches a new quote. A payee whose bank details changed in the last 30 days adds
reason "▲ bank details changed 3 Nov" and the card shows old vs new last-4. Bypass does not skip this card.

### 4.5 Month-end close report

| # | AgentState | What the user sees | Component |
|---|---|---|---|
| 1 | `Submitted` | "/close October" | composer |
| 2 | `Thinking { "Planning" }` → plan card | 11 steps with owners | `plan-card/document` |
| 3 | `AwaitingApproval { Plan }` (Low) | "Run 6 agent tasks" | `plan-approval` |
| 4 | `ToolRunning` (parallel sub-agents) | checklist ticks; agent tree shows who is working | `todo-list/close-checklist`, `subagent-tree/indented`, `acting-indicator/chip` |
| 5 | `AwaitingApproval` (various) | accrual entries (Medium), FX revaluation entry (Medium); anomalies route to 4.3 | `approval-card/paged` |
| 6 | `Streaming` | close memo with citations (3.6); flux lines over threshold explained | `markdown/editorial`, `citation/quote-peek` |
| 7 | `AwaitingApproval` (High) | "Lock October 2026" — arms 600 ms, Enter = Deny | `approval-card/inline` |
| 8 | `Done { elapsed }` | "Closed in 3 h 12 m (agent 14 m)"; period chip → "Oct 2026 · Locked 🔒" | `elapsed-timer/summary`, period chip |

**Steer:** "skip FX revaluation this month" mid-run → `Steering`, the plan card marks the step skipped (⤼)
with the user's reason. **Cancelled:** open approvals go `Stale` ("Run stopped · no longer needed").

---

## 5. Approvals and risk

Tiers per [../specs/approvals.md](../specs/approvals.md) §2.3. The finance kit sets the `Spend` threshold to
**0**, so every money movement is Critical (approvals.md allows "Critical above an app threshold").

| Action | Kind | Tier | Why | "Always allow"? |
|---|---|---|---|---|
| Read ledger, reports, statements | — | none | read-only | — |
| Suggest a category / match (not posted) | — | none | nothing changes | — |
| Post a high-confidence categorisation | `Custom{Post}` | Low | reversible, audited | yes (rule by confidence ≥ x) |
| Accept a reconciliation match | `Custom{Match}` | Low | reversible | yes |
| Bulk recategorise, create category rule | `Custom{Post}` | Medium | many rows | yes, pattern editor warns |
| Post journal / accrual / FX entry ≤ materiality (£500, *proposed*) | `Custom{Journal}` | Medium | changes balances | session only |
| Post journal entry > materiality | `Custom{Journal}` | High | material | session, exact args only |
| Send an email to a payee (refund request) | `Custom{Message}` | Medium | outbound | no |
| Change payee bank details | `Custom{PayeeChange}` | High | top fraud route; next payment to that payee carries a reason | no |
| Lock period · reopen locked period | `Custom{Period}` | High | affects filed numbers | no |
| Export ledger to accountant | `Network` | Medium | data leaves the app | per destination |
| **Pay a bill / payment run** | `Spend` | **Critical** | moves money | **never** |
| **Transfer between own accounts** | `Spend` | **Critical** | moves money | **never** |
| **Run payroll** | `Spend` | **Critical** | moves money, personal data | **never** |
| **File a tax return / submit VAT** | `Custom{Filing}` | **Critical** | legal filing, can't be recalled | **never** |
| Use a bank credential | `Credential` | High | approvals.md floor | session |

**Every money approval shows, without a click:** amount + currency (large, tabular), FX rate and its time
when currencies differ, payee name, debited account (masked `•• 4417`), payee account (masked, with "NEW"
when changed ≤ 30 days), what it pays (bill no.), risk tier glyph + label + 1–3 reasons, balance after.
Hold-to-allow is 1200 ms on every surface and under reduced motion (stepped 4-segment fill, approvals.md
§5.2). A run of several payments is **one request** with an itemised preview, not a batch of requests, so
the approvals rule "Critical is never batched" holds (open question Q1).

---

## 6. New components

Few, by rule. Each needs a full 10-point spec before it is built.

| Proposed name | Family | Tier | Brief |
|---|---|---|---|
| `money-figure` | I (Generative UI, shared primitive) | free (3) + Pro | One amount renderer used by table cells, metric tiles, approvals and memos: minor-unit integer + ISO currency in, formatted string out (tabular, U+2212, accounting parentheses option, symbol vs code rule, compact form with full value in the accessible name). Optional delta mode: sign + ▲/▼ + `direction_good` tone, never hue only. Rolls with `dur.digit.roll`. Free: `money-figure/plain`, `/delta`, `/fx` (original + converted + rate date). |
| `reconcile-pairs` | I (block) | free (1) + Pro | Two aligned columns (statement vs ledger) with link lines: solid = matched, dashed + % = suggestion, none = unmatched. Per-pair actions (accept, reject, split, create entry), header difference that rolls to zero, keyboard pairing (`M` to match focused rows). Lines draw in 240 ms (`dur.check.draw` alias, *proposed*); reduced motion shows them whole. Free `reconcile-pairs/columns`; Pro `/ribbon` (curved links), `/sweep` (agent matching animates top-down), `/split-sheet`. |
| `anomaly-flag` | E (Messages & reasoning — it is a claim with confidence) | free (2) + Pro | ⚑ chip on a row and an expandable card: what looks wrong, evidence as Record citations, confidence label, and decisions the agent proposes (one marked recommended). Resolved state folds to a line with outcome and date. Severity uses the risk glyphs, not new colours. Idea from the fintech alert card with inline actions ([R025], study only). Free `anomaly-flag/chip`, `/card`; Pro `/inbox` (all flags), `/timeline`. |
| `period-chip` | J (Workspace chrome) | free (3) | Top-bar chip "Oct 2026 · Open / Closing BD+4 / Locked 🔒"; menu to switch period; lock state uses glyph + word. Shared by future BI kits. |
| `approval-card/payment-run` | G (variant of `approval-card`) | Pro | Spend layout for N payments in one request: per-currency totals, from-account + balance after, itemised lines (payee, masked account, amount, risk), include / exclude per line as `AllowEdited`, one hold ring. Built on `approval-card/amount` + `approval-card/hold-ring`. A free fallback is `approval-card/inline` with the itemised list in the preview slot. |
| `todo-list/close-checklist` | F (variant of `todo-list`) | free | Checklist with owner (agent identity or person), due business day (BD+n), evidence link and sign-off columns; built on `todo-list/owner-lanes`. |
| `citation` source kind `Record` | E (extension of [citations-sources.md](../specs/citations-sources.md)) | free | `Source::Record { system, ids, label }`; preview renders the cited transactions as a mini table with totals, and "Open" filters the host screen. |
| Finance verb pack | C (verb pack for [verb-cycler.md](../specs/verb-cycler.md)) | free | In-house copy: Categorising / Categorised, Matching / Matched, Totting up / Totted up, Checking the statement / Checked, Drafting / Drafted, Preparing payments / Prepared. |
| Tokens `color.data.gain` / `color.data.loss` | tokens | — | Aliases of status success / danger; daltonized themes swap to blue / orange; added to tokens.md §9 table with cue "sign + ▲/▼ + word". |

---

## 7. Data model and fixtures

### 7.1 Struct sketch (`oxyn-kit-finance`, proposed)

```rust
pub struct Money { pub minor: i64, pub currency: Currency }      // never f64; Currency = ISO 4217 + minor units
pub struct FxRate { pub from: Currency, pub to: Currency, pub rate: Decimal, pub at: SystemTime, pub source: SharedString }

pub struct Account { pub id: AccountId, pub name: SharedString, pub kind: AccountKind /* Bank | Card | Ledger(GlCode) */,
                     pub mask: SharedString /* "•• 4417" */, pub currency: Currency }

pub struct Transaction {
    pub id: TxnId, pub date: NaiveDate, pub account: AccountId,
    pub payee: Option<PayeeId>, pub description: SharedString,
    pub amount: Money, pub original: Option<(Money, FxRate)>,   // multi-currency
    pub category: Option<Categorisation>, pub flags: Vec<AnomalyFlag>,
    pub status: TxnStatus,                                      // Pending | Posted | Reconciled | Locked
}
pub struct Categorisation { pub gl: GlCode, pub confidence: f32, pub by: Actor /* Agent(AgentRef) | User(UserId) | Rule(RuleId) */, pub reason: Option<SharedString> }

pub struct AnomalyFlag { pub id: FlagId, pub kind: AnomalyKind /* Duplicate | Spike{pct} | NewPayeeAccount | RoundNumber | OutOfPeriod | Custom */,
                         pub tier: RiskTier, pub evidence: Vec<TxnId>, pub explanation: Option<SharedString>, pub state: FlagState /* Open | Explained | Resolved{outcome, at} | Dismissed */ }

pub struct StatementLine { pub id: LineId, pub date: NaiveDate, pub text: SharedString, pub amount: Money }
pub struct MatchSuggestion { pub line: LineId, pub txns: Vec<TxnId>, pub confidence: f32, pub reason: SharedString, pub fix: Option<ProposedEntry> }
pub struct Reconciliation { pub account: AccountId, pub period: Period, pub statement_total: Money, pub ledger_total: Money,
                            pub matched: Vec<(LineId, Vec<TxnId>)>, pub suggestions: Vec<MatchSuggestion>, pub unmatched: Vec<LineId> }

pub struct CloseTask { pub id: TaskId, pub title: SharedString, pub owner: Actor, pub due_bd: u8, pub status: TodoStatus, pub evidence: Vec<EvidenceRef>, pub sign_off: Option<(UserId, SystemTime)> }

pub struct Bill { pub id: BillId, pub payee: PayeeId, pub number: SharedString, pub due: NaiveDate, pub amount: Money, pub risk: Risk }
pub struct PaymentRun { pub from: AccountId, pub lines: Vec<PaymentLine>, pub fx: Vec<FxRate>, pub balance_after: Money }
pub struct PaymentLine { pub bill: BillId, pub payee_account_mask: SharedString, pub account_changed_at: Option<NaiveDate>, pub amount: Money, pub included: bool, pub risk: Risk }

pub enum Period { Month { year: i32, month: u8 } }              // + lock state
pub struct Report { pub kind: ReportKind, pub period: Period, pub body: MarkdownRef, pub sources: SourceSet /* Record sources */, pub generated_at: SystemTime }
// Audit uses approvals' AuditEntry unchanged; ledger edits add an EditEntry { before, after, by: Actor }.
```

`PaymentRun` maps to `ApprovalKind::Spend` with `amount` = per-currency totals and the lines in the
preview (open question Q1 on whether `Spend` gets a `lines` field).

### 7.2 Fixture plan (`fixtures/finance/kettle-and-kiln/`, invented)

| File | Contents |
|---|---|
| `company.json` | Kettle & Kiln Ceramics Ltd, GBP, chart of accounts (≈ 40 GL codes) |
| `accounts.json` | Harbourline current •• 4417, savings •• 9020, card •• 7781 |
| `payees.json` | 24 payees incl. Moorfield Clay Supply, Quayside Packaging, Tallow Lane Utilities, Brightwick Kilns GmbH (EUR), Fenwick & Ash, Ridgeway Studio Lease (bank details changed 3 Nov), Makersquare (USD) |
| `transactions-2026-01..10.json` | ≈ 300/month; October = 312 (289 ≥ 90% confidence, 23 below) |
| `statement-2026-10.json` | 247 lines; 239 auto-match; 8 unmatched incl. TAXPAY 0193 −£612.00, card fees −£41.18, FX diff £2.56, Makersquare fee £17.25 |
| `anomalies.json` | duplicate Quayside QP-2291 £1,284.00 (09 Oct + 24 Oct); utilities spike +68%; new payee account Ridgeway |
| `bills.json` | the 6 bills of §3.5 (GBP subtotal £14,926.40; default selection £13,642.40 + EUR 3,200.00) |
| `fx.json` | EUR→GBP 0.8700 (12 Nov 09:00), USD→GBP 0.7821 (27 Oct) |
| `close-2026-10.json` | 11 tasks, states as in §3.4 |
| `kpis-2026-10.json` | cash £62,480.12; revenue £48,210.00; expenses £41,377.80; net £6,832.20 |
| `runs/*.jsonl` | recorded agent event streams for flows 4.1–4.5 (for `run-replay` and offline demos), incl. a slow run (stall), a fast run (< 1 s) and a failure (IBAN rejected) |

Edge fixtures: a 120-character payee name; a JPY refund (0 minor units); a negative balance; a period
with 10,000 transactions (virtualisation); an RTL locale (Arabic UI, `ar-EG` digits off by default).

---

## 8. Variants

### 8.1 Shells
| Variant | Layout × surface × motion | Tier |
|---|---|---|
| `shell/finance-ledger` | three-pane desktop / web, subtle motion | free |
| `shell/finance-compact` | rail + canvas, agent as overlay sheet; dense tables | free |
| `shell/finance-mobile` | approve-first tabs, sheets | free |
| `shell/finance-terminal` | TUI look, box-drawn tables, glyph risks | Pro |
| `shell/finance-cockpit` | wide KPI wall + live agent tree + approvals inbox (multi-entity) | Pro |
| `shell/finance-accountant` | read-only reviewer view: close pack, audit, comments | Pro |

### 8.2 Screens
| Screen | Free | Pro |
|---|---|---|
| Overview | KPI tiles + plain chart + needs-you list | `genui-metric/board`, `genui-chart/scrub-insight`, `genui-chart/annotated-story`, `genui-chart/forecast-band` (cash runway) |
| Transactions | `genui-table/data-grid`, confidence label, flag chip | bulk actions, calc footer, changed cells, confidence meter, anomaly inbox |
| Reconcile | `reconcile-pairs/columns` | `/ribbon`, `/sweep`, `/split-sheet` |
| Close | `todo-list/close-checklist` | `plan-card/plan-vs-actual` (days late), `subagent-tree/org-chart` |
| Bills | data grid + `approval-card/inline` with itemised preview | `approval-card/payment-run`, `approval-card/split-preview` |
| Reports | memo + `citation/number` | `citation/quote-peek`, `confidence-badge/underline`, `genui-chart/compare` |
| Audit | `audit-log/table` | `audit-log/ledger`, `run-replay/step-list` |

Motion axis: **none** (static, every number swaps), **subtle** (default: rolls, washes, link draws),
**expressive** (Pro: chart draw-on `genui-chart/draw-on`, matching sweep). Hold-to-allow and arming keep
their durations on every motion setting.

---

## 9. Accessibility and edge cases

| Case | Handling |
|---|---|
| **Screen readers: numbers** | accessible name spells currency and sign: "minus one thousand two hundred eighty-four pounds"; deltas "down 18.9 percent versus September, worse"; masked accounts "account ending 4417" |
| **Screen readers: tables** | grid role, header cells, row label = date + payee; flags announced as "flagged: possible duplicate"; agent changes coalesced (≤ 1 per 2 s, genui-blocks.md) "Agent categorised 40 more rows" |
| **Charts** | `T` toggles table view; annotation pins are focusable buttons announcing their note; scrub range is keyboard selectable (Shift+←/→) |
| **Approvals** | Critical announced assertively once with amount, payee, account, risk; keyboard hold = hold Y or Space 1.2 s; `hold_alternative = TypePhrase` ("type PAY 5") for users who can't hold |
| **Colour-blind** | gain/loss = sign + ▲/▼ + word; risk = ○ ◇ ▲ ⬣ + label; match lines solid vs dashed; confidence = dots + % |
| **Keyboard-only** | `J/K` rows, `Space` select, `A` accept, `E` explain, `M` match pair, `⌘/Ctrl+Shift+A` jump to pending approval; single-key shortcuts can be turned off (WCAG 2.1.4) |
| **Dense data** | virtualise > 200 rows; sticky header + first column; compact density token; 10,000-row month loads in pages, Σ footer computed server-side with "of 10,000" |
| **Long names** | payee truncates in the middle ("Ridgeway Studio … Ltd"), full name in tooltip and accessible name; approval card wraps (never truncates payee or amount) |
| **Mixed currencies** | columns with more than one currency show ISO codes on every cell; totals split per currency; converted total only with rate + time shown |
| **RTL** | layout mirrors; amounts, IBAN masks and account numbers stay LTR in isolated spans; minus stays leading; hold fill runs from inline-start |
| **Locale** | thousands / decimal separators from locale; GBP `£1,284.00`, de-DE `1.284,00 €`; dates in locale format, tooltips show ISO |
| **Slow agent / slow bank** | `Stalled` after N s ramps the strip; bank feed "last synced" age shown; payment `Submitting` shows "Still sending…" after 10 s and never re-sends on its own |
| **Very fast agent** | categorising < 1 s: rows land in one batch, one wash; no per-row stagger; approvals auto-resolved by rule within 150 ms show no card (approvals.md §7) |
| **Offline** | read cached data with "Offline · as of 09:12"; every write and every hold disabled with reason |
| **Period locked mid-run** | pending entries for that period go `Stale` with "Oct 2026 was locked"; agent proposes posting to Nov instead (new request) |
| **Two users** | Tom and Rosa on the same run: an approval answered by Tom resolves Rosa's card "Answered by Tom H." (origin) |
| **Duplicate request** | agent retries a failed payment: card shows "Asked again · first attempt failed 10:06 (IBAN rejected)" |

---

## 10. References

Products were studied from their public pages only; no code, copy or assets are taken. Ideas are
re-implemented from our own specs ([../licensing.md](../licensing.md)).

| Reference | What we take (idea only) | Licence / use |
|---|---|---|
| Ramp Accounting Agent — https://ramp.com/blog/accounting-agent-launch | agent codes every line item, routes only what needs judgment, low-risk spend syncs automatically with an audit trail; user chooses what the agent may do (code / review / sync); corrections in plain language that the agent learns from; accrues stragglers at month end | Proprietary — study only |
| Digits "How AI Works" — https://help.digits.com/business-agentic-general-ledger/digits-ai | every categorisation gets a confidence score: high posts, lower routes to a review checklist; statement-aware reconciliation surfacing missing / duplicate / unmatched; per-transaction activity log; AI does not override human decisions or auto-post when unsure | Proprietary — study only |
| Rillet January 2026 update — https://www.rillet.com/blog/january-2026-product-update | flux analysis with materiality thresholds ($ and/or %), AI explanations with source links; accrual suggestions with plain-language rationale and approve / post workflow; audit log with plain-English change summaries | Proprietary — study only |
| Puzzle AI Close — https://puzzle.io/ai-close | close tasks assigned to agents with due dates; review panel listing every proposed entry, reasoning step and source transaction; nothing posts without sign-off; audit rows "Matched 249 of 249 … ✓ Sarah K. via AI" ("by person via agent") | Proprietary — study only |
| Brex agents — https://www.brex.com/platform/brex-ai | audit agent sorts potential violations by risk level; review agent auto-approves low risk and escalates exceptions | Proprietary — study only |
| Fintech "Authorizations" dashboard [R025] (research note 15 #25) | KPIs with sparkbars, radial tick gauge, alert card with two inline actions → `anomaly-flag` | Study only |
| Beautiful UI harness / "Creamery Ops" (research note 17 §4 ideas 1, 6, 8) | one invented company per kit; data-heavy blocks: records grid with calc footer, insight cards with scrubbable charts | MIT per [../licensing.md](../licensing.md) — may port with attribution |
| affectUI KPI Board, Metrics Dashboard, Agent Ledger, Decision Inbox, Invoice & Credit Note (research note 11) | coverage check for dashboard and ledger blocks | Paid — study only |
| Arc UI Finance (Market terminal, Budget variance), Analytics, Money input, Scrub input (research note 12) | coverage check: money input, scrub input, budget variance | Paid — study only |
| Codex / Claude Code approvals (via [../specs/approvals.md](../specs/approvals.md)) | decision set, risk tiers, hold-to-allow | see approvals.md §10 |

**Open questions**
1. **Q1 Payment runs:** one `Spend` request with itemised lines (this kit) vs one Critical request per
   payment. One request keeps "Critical is never batched" and one hold; per-payment gives finer audit.
   Proposal: one request, one audit entry per line. Needs a `lines` field on `ApprovalKind::Spend`.
2. **Q2 FX quote expiry** (10 min proposed) — app-set via `expires_at`; should the card offer "re-quote"
   inside the card, or always a new request?
3. **Q3 Filing kind:** add `ApprovalKind::Filing` to approvals.md (Critical floor, no "always") instead of
   `Custom`?
4. **Q4 Materiality threshold** (£500 / 10% proposed) for Medium vs High journal entries: app setting or
   kit default?
5. **Q5 Gain/loss tokens:** add `color.data.gain` / `.loss` to tokens.md §9, and settle the daltonized pair.
6. **Q6 Breakpoint tokens** do not exist yet; this kit's 640 / 1024 / 1440 are proposals.
7. **Q7 `money-figure` home:** family I primitive (proposed) or a `format` helper in `oxyn-core` only?
