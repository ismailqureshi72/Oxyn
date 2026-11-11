**Status:** draft v0.1 (2026-11-11).

# Spec: approvals & safety (family G overview)

Overview of family **G** in [../component-taxonomy.md](../component-taxonomy.md) (target **30+
variants**). Same depth as the worked example [composer-send-control.md](composer-send-control.md).
Motion tokens are the shared ones in [composer.md](composer.md) §"Motion tokens" and the family-F tokens
in [tools.md](tools.md) §5; family-G tokens are added in §5 below. Waiting motion and colour roles are
the ones in [loaders.md](loaders.md) §2.5 and §3.2 (`color.loader.waiting`, "different motion, not a
slower spin"); nothing here overrides them. All values marked *proposed* are Oxyn's own starting points,
to be tuned in the Studio. Numbers attributed to products come from the research notes named beside them
(note numbers refer to the project research folder; `[Rxxx]` IDs are in
[../research/sources.csv](../research/sources.csv)).

Specs in this family:

| Spec | Component(s) | Variants (free + Pro) |
|---|---|---|
| this file | approval request model, risk tiers, decision set, timeout, batching, audit model; `risk-badge` (shared primitive) | 3 + 4 |
| [approval-card.md](approval-card.md) | `approval-card` (inline card, command-approval footer inside a tool-call card, paged queue, stale state) | 5 + 14 |
| [plan-approval.md](plan-approval.md) | `plan-approval` (the decision footer rendered in the plan card's `PlanApprovalSlot`) | 3 + 9 |
| [permission-rules.md](permission-rules.md) | `permission-rules` (rules list / editor / tester), `sandbox-badge`, `audit-log` | 5 + 12 |
| [takeover-banner.md](takeover-banner.md) | `takeover-banner` (take-over / hand-back banner with resume note) | 3 + 9 |
| **Total** | | **19 free + 48 Pro = 67** |

Hand-offs this family implements (owned elsewhere, rendered here):
- **Command / tool approval footer** inside a tool-call card: [tool-call.md](tool-call.md) §5 "Approval
  hand-off" (footer slides in 200 ms `ease.out`, resolves 220 ms `spring.snap`, collapses into a one-line
  "Allowed once" note; deny morphs the row to `Denied`). Lifecycle in [tools.md](tools.md) §2.2.
- **Plan approval footer** in the plan card's `approval: Option<PlanApprovalSlot>`:
  [plan-todo.md](plan-todo.md) §3.2, §5 "Approval hand-off", §10.
- **Take-over / hand-back banner** attached to the acting indicator's `HandoffRequested` /
  `ControlRequested` / `UserControl{from_agent}` states and its `on_open_request` route:
  [acting-indicator.md](acting-indicator.md) §1 rule 6, §2, §9.
- **Raise from a card**: "Allow and switch to Auto-edit" uses the permission selector's card-driven raise
  (`Source::ApprovalCard`, applied now): [composer-permission-selector.md](composer-permission-selector.md)
  §2 rule 2, §9. "Always allow" **adds a rule and never changes the level** (selector §2 rule 4).
- **Jump to the pending card**: status strip `AwaitingYou` → Review, proposed `⌘/Ctrl+Shift+A`
  ([composer-status-strip.md](composer-status-strip.md)); the composer's ↑ answers the open card
  ([composer-send-control.md](composer-send-control.md), [R088]); the input shell's placeholder reads
  "Approve above, or type why not…" ([composer-input-shell.md](composer-input-shell.md)).

## 1. Purpose
An agent stops and asks before it does something the user has not already allowed: run a command, write
a file, reach a host, call an MCP tool, start executing a plan, spend money, use a credential, or take
control of a screen. The user is trying to answer, in a second or two and without scrolling:
1. **What exactly will happen?** (the command, the files and diff, the host, the amount, the secret's
   name — never its value)
2. **How risky is it?** (a tier they can read without colour: icon shape + label + reason line)
3. **Why does the agent want it?** (the agent's stated reason, quoted, not paraphrased)
4. **How far does my yes go?** (once, for this session, always via a rule — and what that rule matches)
5. **If I say no, what does the agent hear?** (deny silently, or deny with a reason the agent reads)

What we saw (research note 03 §1D, note 05 §4): the standard triad is **Allow once / Always allow
(scoped) / Deny-with-feedback**. Codex CLI asks "Would you like to run…", shows an italic Reason and
`$ cmd`, and numbers its options **1 Yes (y) / 2 Yes, don't ask again for commands starting with `X` (p)
/ 3 No, and tell Codex what to do differently (esc)**; Claude Code offers "Yes", "Yes, and don't ask again
for: `<pattern>`", "Yes, and switch to auto mode", "No" (Esc = no without comment); Claude desktop's
browser offers Allow once / Always allow / Deny per site, computer use adds a capability tier badge and
an extra warning for high-reach apps; the Codex app's auto-review item carries a **risk level** and the
statuses approved / denied / stopped / timed out (note 03). LangGraph Agent Inbox allows accept / edit /
respond / ignore (note 05). Kits: signoff-ui's ApprovalCard (once / session / always, risk level, a reason
the agent reads) and `useApprovalPolicy` (rules by tool and argument pattern with an audit trail) [R107];
Beautiful UI's paged question approval "1 / 3, Skip, Continue" [R106]; affectUI's Approval Card (risk
badge, change preview, Approve / Deny with shortcuts, pausing countdown, morph into a resolved row) and
Permission Request (Allow once / Allow 1 hour / Allow always, folds into a line with time left and Revoke)
(note 11 §17–18). Family G turns these into **one request model, one decision set, one risk grammar**
shared by every card, footer, banner, rules list and audit row.

## 2. The approval model (`oxyn-core::approval`, proposed)

### 2.1 Where it sits relative to `AgentState`
[../architecture.md](../architecture.md) has one aggregate `AgentState::AwaitingApproval { request }`.
Several requests can be open at once (parallel tool calls, sub-agents, a plan plus a command), so
`oxyn-core` keeps an **`ApprovalQueue`** beside the `RunLog` ([tools.md](tools.md) §2.1). Rules (proposed):
1. `AwaitingApproval { request }` is **derived**: `request` is the **oldest open** request of the run (or
   of the focused sub-agent thread). The queue holds the rest; components read the queue for "1 of 3".
2. A request is always linked to what it gates: a `CallId` (tool call), a `PlanId`, a `SurfaceId`
   (control), or nothing (free-standing question). The gated object shows `AwaitingApproval` and its
   child clock **pauses** ([elapsed-timer.md](elapsed-timer.md)).
3. **Approval wins over running** in the aggregate state (tools.md §2.1 rule 1).
4. Resolutions are **append-only**: an answered request is never reopened; a retry is a new request
   (with `retry_of`). Every resolution writes one `AuditEntry` (§2.7).
5. The client **fails safe**: if a resolution cannot reach a remote agent, the card shows `Submitting`
   and the gated call stays blocked; nothing is treated as approved until the adapter acknowledges.

### 2.2 Request kinds
| Kind | Gated object | Preview the card must show (never hidden behind a click) | Default risk floor (proposed) | Rule matcher |
|---|---|---|---|---|
| `Command` | tool call (shell) | `$ cmd` in mono, cwd, env changes, sandbox (in / out), network (on / off) | Medium; High if outside sandbox or destructive verbs (`rm -rf`, `git push --force`, `DROP`) are detected by the app's classifier | tool + command prefix / glob |
| `Edit` | tool call (write) | file list with `+N −N`, first hunk inline, "Open diff" link to family H | Low inside workspace; High outside workspace or protected paths (`.git`, per Codex note 05) | tool + path glob |
| `Network` | tool call (fetch / browse) | method + host + path, first-time-host flag | Medium; High for non-GET to a new host | host / domain (incl. subdomains) |
| `McpTool` | tool call (MCP) | server identity badge, tool name, arguments as a key/value table (redacted values masked) | Medium; app may set per tool | server + tool + argument pattern |
| `Plan` | plan card | plan title, step count, files / commands the plan names, chosen run level | Low (the plan itself does nothing; its steps are gated by their own rules) | — (no "always" for plans) |
| `Spend` | tool call / payment step | amount + currency in large tabular figures, payee, what it buys, budget left | **High**, Critical above an app threshold | payee + max amount (session only) |
| `Credential` | tool call / sign-in | secret **name** and scope ("STRIPE_KEY · read-only"), where it will be sent; value never shown ([R049] idea) | **High** | secret + destination |
| `Control` | surface (acting indicator) | surface name, what the agent will drive, capability tier (view / click / full, note 03) | Medium; High for full desktop | surface + capability |
| `Question` | nothing (free-standing) | the question and its options, "(recommended)" marker if the agent gives one ([R022]) | none (no risk badge) | — |

Kinds are open (`Custom { label, icon }`); a custom kind declares a risk floor and whether "always" is
allowed. The **risk tier of a request is `max(kind floor, app classifier, policy)`** — a classifier can
raise a tier, never lower it below the floor (proposed).

### 2.3 Risk tiers
Risk is carried by **icon shape, label text and border weight**; colour is a fourth, redundant channel.
Every tier stays distinguishable in greyscale, daltonized themes and the terminal surface.

| Tier | Label | Glyph (shape) | TUI glyph | Border | Tone token | What changes in the card |
|---|---|---|---|---|---|---|
| `Low` | "Low risk" | circle with check-dot (round = calm) | `○` | 1 px | `risk.low` (neutral / info) | all decisions available; batching allowed |
| `Medium` | "Medium risk" | diamond | `◇` | 1 px | `risk.medium` (accent-amber) | all decisions; "always" pattern editor warns when broadening |
| `High` | "High risk" | triangle with `!` | `▲` | 2 px | `risk.high` (warning) | Allow buttons **arm for 600 ms** before they accept input; never batched; "always" limited to session unless app allows |
| `Critical` | "Critical" | octagon with `!` | `⬣` | 2 px + static 45° hatch at 8 % alpha on the header | `risk.critical` (danger) | **press-and-hold 1200 ms** to allow; only "Allow once"; no "always"; never auto-approved by a level below Bypass, and not by Bypass unless `bypass_includes_critical` (default false) |

Each request also carries `risk_reasons` (1–3 short lines, written by the app or classifier, e.g.
"Writes outside the workspace", "Sends data to a new host", "Spends $240.00", "Deletes 14 files").
Glyph shapes follow research note 05's recommended metaphors (warning triangle = high, octagon-alert =
critical); the circle / diamond pair for low / medium is our proposal. Permission *levels* (tiers 0–4 in
the permission selector) are a different scale: a level says **which request risks auto-run**:
`ReadOnly` none · `Ask` none · `AutoEdit` Low `Edit` only · `FullAccess` up to High inside the sandbox ·
`Bypass` up to High; **Critical never auto-runs at any level** (decided 2026-11-12;
`bypass_includes_critical = false`, matching the permission selector).

### 2.4 Decision set
| Decision | Label (i18n token, default copy) | Key | Effect | Agent receives | Allowed for |
|---|---|---|---|---|---|
| `AllowOnce` | "Allow once" | `Y` | this call only | `approved` | all tiers |
| `AllowSession` | "Allow for this session" | `S` | auto-approves identical requests (same kind + matcher) until the session ends | `approved` | Low, Medium, High (High: same exact args only) |
| `AllowAlways` | "Always allow…" (opens pattern editor) | `A` | **adds a rule** (permission-rules.md) with the edited pattern and chosen scope, then approves this call | `approved` | Low, Medium; High only if `allow_always_high` (default false); never Critical, Plan, Spend |
| `AllowEdited` | "Edit and allow" | `E` | the user edits the args (command text, amount down, host) and approves the edited version | `approved` + `edited_args` (agent must use them) | kinds whose adapter supports it |
| `AllowAndRaise(level)` | "Allow and switch to Auto-edit" | `⇧Y` (proposed) | approve, then the permission selector raises with `Source::ApprovalCard` (tier ≥ 3 still runs the selector's danger confirm) | `approved` | when the app enables it |
| `Deny` | "Deny" | `N` / `Esc` | blocks the call; row → `Denied` | `denied` (no reason) | all |
| `DenyWithReason(text)` | "Deny and tell the agent…" | `R`, or type in the composer and press Enter | blocks the call and sends `text` to the agent as the denial reason | `denied` + `reason` (agent reads it; AI SDK `output-denied` with reason, note 05) | all |
| `Skip` | "Skip" | `K` | Question kind only: leaves the question unanswered, agent decides | `skipped` | Question |

Rules (proposed):
1. **Default focus target is never the most dangerous option.** Low / Medium: focus lands on the card
   (group), `Y` / `Enter` on "Allow once"; High / Critical: `Enter` maps to **Deny**; allowing needs `Y`
   (High, after arming) or a hold (Critical).
2. **Input guard**: single-key shortcuts are ignored for **400 ms** after a card mounts and only work
   while focus is inside the card or the app's approval scope — a user typing "yes" in the composer must
   never approve anything.
3. "Always" never silently broadens: the pattern editor opens pre-filled with the **narrowest** useful
   pattern (Codex's command prefix idea, note 03) and shows how many past calls this session it would
   have matched.
4. The deny-reason text is the user's words, sent verbatim; the card shows "Sent to agent" under it.
5. Labels can be renamed per app (Approve / Reject, Run / Don't run) but the decision IDs and keys stay.

### 2.5 Timeout
- Default **no timeout** for interactive sessions (proposed). Apps set `expires_at` (e.g. headless or
  background runs).
- **A timeout never approves.** Expiry resolves as `Expired` → the agent receives `denied` with reason
  "timed out" (Codex app lists "timed out" as its own review status, note 03); the gated call becomes
  `Cancelled { by: ApprovalTimeout }` ([tools.md](tools.md) §2.2).
- The countdown appears only in the last **60 s** (or the last 25 % if the window is shorter than 4 min),
  as text "Expires in 0:42", with a 1 Hz digit swap; no ring that pulses.
- Hover or focus does not pause a server-side timeout (the agent is waiting on wall time); the card says
  so in the tooltip. Opt-in **auto-approve** countdowns exist only for plans and Low requests in Pro
  variants and **do** pause on hover / focus (affectUI's pausing countdown, note 11 §17; Jules auto-approve
  timer, note 03).
- After expiry the card enters `Expired` with "Ask again" (re-issues a new request through the adapter if
  supported).

### 2.6 Batching and queues
- Requests created within **50 ms** of each other by parallel calls (the tools grouping window,
  [tools.md](tools.md)) form a **batch**. Requests across turns or agents form the **queue**.
- The queue is paged: one card visible, header "1 of 3", `←/→` or `[` / `]` to page (Beautiful UI
  paging idea [R106]). Order: creation time; the page of a newly arrived Critical request does not jump
  ahead (no surprise reflow), but the header shows "1 critical" in its tier glyph.
- **Batch actions** appear only when every request in the batch is ≤ Medium, of one kind, and from one
  agent: "Allow all 3 once" / "Deny all". Any High or Critical in a batch disables batch actions with a
  reason ("Review the high-risk request on its own").
- Answering a page advances to the next open page after the resolve morph (220 ms); answering the last
  closes the queue. A request resolved elsewhere (other device, rule added on page 1 that matches page 3)
  resolves in place with its provenance ("Allowed by rule you just added").

### 2.7 Audit trail
Every resolution appends an `AuditEntry` (never edited, never deleted by the UI; retention is the app's):

| Field | Example |
|---|---|
| `at` | wall time (UTC stored, local shown) |
| `request` | snapshot: kind, tool, target, args (redacted), risk tier + reasons, agent / sub-agent, run, turn |
| `decision` | AllowOnce · AllowSession · AllowAlways{rule} · AllowEdited{diff of args} · AllowAndRaise · Deny · DenyWithReason{text} · Skip · Expired · Stale · Superseded |
| `by` | `User` · `Rule(id)` · `Session(grant id)` · `Level(PermissionLevel)` · `Classifier(verdict)` · `Policy(source)` · `Timeout` |
| `latency` | time from request to decision (paused-clock interval) |
| `origin` | device / surface the answer came from (app-provided) |

The audit log view is in [permission-rules.md](permission-rules.md) §audit-log.

## 3. Request lifecycle (state machine shared by every family-G component)

```
                 adapter: approval-requested
   (none) ───────────────────────────────▶ Requested ──(input guard 400 ms)──▶ Open
                                               │ matches rule / session grant / level
                                               └────────────────────────────────▶ AutoResolved{by} ──▶ (audit, no card;
                                                                                                         provenance glyph on the row)
   Open ──focus──▶ Focused ──Y/S/A/E/N/R──▶ Deciding
     │                                         │ A: PatternEditing ──confirm──┐   E: EditingArgs ──confirm──┐
     │                                         │ R: ReasonEditing ──send────┐ │                             │
     │                                         │ High: Arming(600 ms) ──▶ Armed ──Y──┐                     │
     │                                         │ Critical: Holding(≤1200 ms) ─full─▶─┤ release early ─▶ Open│
     │                                         ▼                                      ▼                     ▼
     │                                     Submitting (adapter ack pending) ─────────────────────────────────┘
     │                                         │ ack ok                     │ ack failed / offline
     │                                         ▼                            ▼
     │                          Resolved{Allowed | Denied | Skipped}     SubmitFailed ("Couldn't send · Retry") ──▶ Submitting
     │                                         │ (220 ms morph to resolved row; audit entry)
     │
     ├─ expires_at reached ────────────▶ Expired (deny "timed out") ──"Ask again"──▶ new Requested
     ├─ run stopped / cancelled ───────▶ Stale ("Run stopped · no longer needed"), actions disabled
     ├─ agent withdraws / retries ─────▶ Superseded{by} ("Replaced by a newer request")
     ├─ answered on another device ────▶ Resolved{.., origin: Remote}
     └─ policy blocks while open ──────▶ Resolved{Denied, by: Policy} with reason
   Overlays: Queued(page k of n) · Batched(batch id) · Offline (actions disabled with reason; answers are not queued locally — fail safe)
```

## 4. Micro-interactions (family level)
- **Arrival.** The gated row switches to the amber hand glyph and pauses its timer; the card / footer
  rises from the bottom edge of its host (200 ms `ease.out`, the tool-call hand-off). The waiting loader
  motion (hourglass / hold-and-breathe in `color.loader.waiting`, loaders.md §3.2) replaces any spinner.
  The permission selector chip gets its attention badge and pulses once (selector §4).
- **Risk entrance.** Low / Medium: no extra motion. High: the 2 px border draws in 160 ms. Critical: the
  hatch appears static (no animated stripes — danger is calm and constant, the selector's rule).
- **Arming (High).** Allow buttons show a left-to-right fill over 600 ms (linear); early press does a
  2-cycle 3 px shake and announces "Available in a moment".
- **Hold (Critical).** A ring (or bar on the footer layout) fills over 1200 ms while held; releasing
  early drains it in 200 ms `ease.in`; completion gives a 1-step scale 1 → 1.04 → 1 and, on touch, one
  haptic tick at 50 % and one at 100 % (proposed).
- **Resolve.** The card height collapses (220 ms `spring.snap`) into a one-line resolved row: "Allowed
  once · 0:04", "Denied · 'use the staging DB'", "Always allowed `npm test *` · project" (affectUI
  resolved-row morph, note 11 §17). The row's glyph morphs hand → ✓ / ⊘.
- **Pattern editing.** The "Always" chip expands in place into an editor; narrowing / broadening steps
  slide the highlighted part of the command (160 ms); the match count rolls (280 ms `dur.count.roll`).
- **Deny with reason.** The footer swaps to a one-line field (height `spring.snap`); on send the text flies
  up 8 px and fades into the resolved row (220 ms `dur.travel`).
- **Paging.** Next page slides in from the inline-end 12 px with a 160 ms fade; the "1 of 3" numerator
  rolls.
- **Stale.** Card desaturates to 60 % over 160 ms; buttons crossfade to "Run stopped"; collapses to one
  line after 4 s.

## 5. Tokens and timing

### 5.1 Family-G tokens (proposed)
| Token | Value | Used for |
|---|---|---|
| `dur.approval.in` | 200 ms `ease.out` (= tool-call footer in) | card / footer / banner enter |
| `dur.approval.resolve` | 220 ms `spring.snap` | collapse into resolved row |
| `delay.approval.input_guard` | 400 ms | shortcut lockout after mount |
| `dur.approval.arm` | 600 ms linear (= selector arming) | High allow buttons |
| `dur.approval.hold` | 1200 ms linear (= selector `hold-to-bypass`) | Critical allow |
| `dur.approval.hold.drain` | 200 ms `ease.in` | early release |
| `dur.approval.page` | 160 ms fade + 12 px slide | queue paging |
| `dur.approval.stale` | 160 ms desaturate, 4000 ms hold, 180 ms collapse | stale / expired |
| `timeout.countdown.show` | last 60 s or last 25 % | expiry text |
| `batch.window` | 50 ms | parallel request coalescing |
| `risk.low/medium/high/critical` | colour tokens; text ≥ 4.5:1, borders ≥ 3:1 in both themes | badge, border, glyph |

### 5.2 Timing table (family defaults)
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Card / footer enter | 200 ms height + 160 ms fade | 0 | `ease.out` | — | 60 Hz |
| High border draw | 160 ms | 0 | `ease.out` | — | 60 Hz |
| Input guard | 400 ms (no visual) | from mount | — | — | — |
| Arming fill | 600 ms | after input guard | linear | — | 60 Hz |
| Early-press shake | 240 ms, 2 × 3 px | 0 | linear | — | 60 Hz |
| Hold fill / drain | 1200 ms / 200 ms | 0 | linear / `ease.in` | — | 60 Hz |
| Hold complete pop | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Resolve collapse | 220 ms | 0 | `spring.snap` | — | 60 Hz |
| Glyph morph hand → ✓ / ⊘ | 160 ms + 240 ms stroke | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| Page change | 160 ms | 0 | `ease.out` | — | 60 Hz |
| Count roll ("1 of 3", match count) | 280 ms | 0 | `ease.out` | 30 ms | 30 Hz |
| Expiry countdown | per second | at threshold | digit swap | — | 1 Hz |
| Stale desaturate / hold / collapse | 160 / 4000 / 180 ms | 0 | `ease.out` / — / `ease.in` | — | 30 Hz |
| Selector attention pulse | 600 ms once | 0 | `ease-in-out` | — | 60 Hz |
| **Reduced motion** | every change is a 0–80 ms opacity crossfade; heights jump; no shake (inline text "Available in a moment"); arming shows static "Available in 0.6 s" then enables (delay kept — safety, not decoration); hold keeps its 1200 ms duration and shows a stepped 4-segment fill updated at 4 Hz plus "Hold to allow · 1.2 s"; no glyph stroke draw; countdown digits swap | — | linear | none | 1 Hz idle |

## 6. Variants

### 6.1 Family catalog
| Component | Free | Pro | Spec |
|---|---|---|---|
| `risk-badge/*` | 3 | 4 | this file §6.2 |
| `approval-card/*` | 5 | 14 | [approval-card.md](approval-card.md) §6 |
| `plan-approval/*` | 3 | 9 | [plan-approval.md](plan-approval.md) §6 |
| `permission-rules/*`, `sandbox-badge/*`, `audit-log/*` | 5 | 12 | [permission-rules.md](permission-rules.md) §6 |
| `takeover-banner/*` | 3 | 9 | [takeover-banner.md](takeover-banner.md) §6 |
| **Total** | **19** | **48** | 67 ≥ 30 target |

### 6.2 `risk-badge/*` (shared primitive, owned here)
**Free (3)**
| Name | Idea | Surfaces |
|---|---|---|
| `risk-badge/shape` | Tier glyph (circle / diamond / triangle / octagon) + label "High risk"; reasons in tooltip | all |
| `risk-badge/pill` | Pill with glyph, label and the first reason inline ("▲ High · writes outside workspace") | chat, desktop, ide |
| `risk-badge/glyph` | TUI: `○ ◇ ▲ ⬣` + bracketed label `[high]`, reasons on the next dim line | terminal, ide |

**Pro (4)**
| Name | One-line idea |
|---|---|
| `risk-badge/meter` | Four-step meter whose filled steps equal the tier, with the reason under it |
| `risk-badge/reasons-stack` | Reasons as stacked chips with their own icons (network, delete, money, key) |
| `risk-badge/classifier` | Adds a reviewer verdict chip ("Reviewed · raised to High") beside the tier (Codex auto-review risk level idea, note 03) |
| `risk-badge/blast-radius` | Small concentric rings showing reach: this file → workspace → machine → internet |

## 7. Edge cases (family level)
- **Many parallel requests** (fan-out of 20 sub-agents): the queue shows "1 of 20 · 3 agents"; the
  approval inbox variant (Pro) groups by agent; batch actions only per the §2.6 rule.
- **Request arrives while the user types in the composer**: no focus steal; announcement + attention
  badge; input guard prevents stray keys; composer placeholder changes to "Approve above, or type why
  not…".
- **Same request twice** (agent retries after a deny): the new card shows "Asked again · you denied this
  0:30 ago" with the previous reason.
- **Rule added on page 1 matches page 3**: page 3 resolves in place as `AutoResolved{Rule}`, the queue
  count rolls down.
- **Very fast agents**: requests resolved by a rule within 150 ms (`delay.row_guard`) never show a card.
- **Very slow agents / remote**: `Submitting` shows a 2 px underline progress after 300 ms; after 10 s
  "Still sending…"; failure → `SubmitFailed` with Retry; the gated call stays blocked (fail safe).
- **Offline**: actions disabled with "Offline · answers can't reach the agent"; Deny stays enabled only if
  the adapter can enforce it locally (client-side gate).
- **Secrets in arguments**: the redactor masks values (`sk-…•••• last 4`); the "Edit and allow" editor
  never reveals them; audit snapshots store masked values.
- **Long commands** (> 8 lines, heredocs): first 4 lines + "Show all 23 lines"; dangerous tokens found by
  the classifier are highlighted even if they sit in the hidden part ("contains `rm -rf` on line 17").
- **RTL**: layout mirrors; commands, paths and diffs stay LTR in isolated spans; arming and hold fills run
  from the inline-start.
- **Tiny sizes (16–20 px)**: the risk glyph alone; Critical adds a 4 px corner dot so risk never depends on
  hue; approval actions move to the card / overflow.
- **Dense IDE panels**: `approval-card/footer` compact (28 px buttons with key hints); pattern editor opens
  as a popover.
- **Touch**: 44 px targets; no single-key shortcuts; Critical hold uses haptics; swipe is never an
  approval gesture (too easy to trigger) — proposed.
- **Multiple devices**: a request answered on the phone resolves the desktop card with "Answered on your
  phone" (origin).
- **Policy-managed**: rows / options disabled with reason "Your organisation requires approval for this".

## 8. Accessibility (family level)
- Each open request is announced **assertively** once: "Approval needed. High risk. Run cargo publish.
  Reason: release the crate. Press Y to allow once, N to deny" (keys only if the user's focus can reach
  the card). Further queue arrivals are polite: "2 more approvals waiting".
- Risk is in text (label + reasons) and shape, never colour only; contrast ≥ 4.5:1 for text, ≥ 3:1 for
  borders and glyphs in light, dark and daltonized themes.
- Arming and hold are exposed: `aria-disabled="true"` during arming, then "Allow is available"; the hold
  button is `button` with `aria-describedby` "Press and hold for 1.2 seconds", and **keyboard users hold
  Y or Space** for the same duration; an app setting `hold_alternative = TypePhrase` lets users who cannot
  hold type the target name instead (proposed).
- Focus never moves into a newly arrived card while the user is typing; the jump shortcut (`⌘/Ctrl+Shift+A`)
  and the status strip's Review move it there. After a decision, focus moves to the next page's card, or
  back to where it was before the jump.
- Shortcuts are listed with `aria-keyshortcuts`; single-key shortcuts can be turned off (WCAG 2.1.4
  character key shortcuts) — then Alt+key is used (proposed).
- Resolved rows read as "Allowed once by you, 4 seconds after asking" / "Denied, reason sent to agent".

## 9. Props sketch (shared model)
```rust
pub struct ApprovalRequest {
    pub id: ApprovalId,
    pub kind: ApprovalKind,                 // Command { cmd, cwd, env } | Edit { files: Vec<FileEditRef>, diff: DiffRef }
                                            // | Network { method, url, first_time } | McpTool { server: McpServerRef, tool, args }
                                            // | Plan { plan: PlanId } | Spend { amount: Money, payee, item, budget_left: Option<Money> }
                                            // | Credential { secret: SecretName, scope, destination } | Control { surface: SurfaceId, capability: CapabilityTier }
                                            // | Question { prompt, options: Vec<QuestionOption> } | Custom { label, icon, risk_floor, allow_always: bool }
    pub gated: Gated,                       // Call(CallId) | Plan(PlanId) | Surface(SurfaceId) | None
    pub requester: AgentRef,                // root agent or sub-agent (identity colour from subagent-tree)
    pub reason: Option<SharedString>,       // the agent's stated reason, shown quoted
    pub risk: Risk,                         // { tier: RiskTier, reasons: SmallVec<[SharedString; 3]>, source: Floor | Classifier | Policy }
    pub decisions: DecisionSet,             // allowed decisions after tier rules (§2.4)
    pub suggested_rule: Option<RulePattern>,// narrowest pattern for "Always allow…"
    pub sandbox: SandboxStatus,             // shared with sandbox-badge
    pub created_at: Instant,
    pub expires_at: Option<Instant>,        // None = no timeout (default)
    pub batch: Option<BatchId>,
    pub retry_of: Option<ApprovalId>,
}

pub enum RiskTier { Low, Medium, High, Critical }

pub enum Decision {
    AllowOnce, AllowSession, AllowAlways { rule: RuleDraft }, AllowEdited { args: Value },
    AllowAndRaise { level: LevelId }, Deny, DenyWithReason { text: SharedString }, Skip,
}

pub enum Resolution {
    Resolved { decision: Decision, by: DecidedBy, at: Instant, origin: Origin },
    Expired, Stale { cause: StaleCause /* RunCancelled | RunFailed | TurnEnded */ }, Superseded { by: ApprovalId },
}

pub enum DecidedBy { User, Rule(RuleId), Session(GrantId), Level(LevelId), Classifier(Verdict), Policy(PolicySource), Timeout }

pub struct ApprovalQueue {                  // oxyn-core, beside RunLog
    pub open: Vec<ApprovalRequest>,         // creation order
    pub resolved: Vec<(ApprovalId, Resolution)>,
}

impl ApprovalQueue {
    /// AgentState::AwaitingApproval { request } is derived from this: the oldest open request.
    pub fn head(&self, scope: ThreadScope) -> Option<&ApprovalRequest> { /* … */ }
    pub fn page_of(&self, id: ApprovalId) -> (usize, usize) { /* "1 of 3" */ }
    pub fn batch_actions(&self, batch: BatchId) -> Option<BatchActions> { /* §2.6 rule */ }
}

pub struct AuditEntry { pub at: SystemTime, pub request: RequestSnapshot, pub resolution: Resolution, pub latency: Duration, pub origin: Origin }
```

## 10. Related components and references
**Related:** [approval-card.md](approval-card.md) · [plan-approval.md](plan-approval.md) ·
[permission-rules.md](permission-rules.md) · [takeover-banner.md](takeover-banner.md) ·
[composer-permission-selector.md](composer-permission-selector.md) (levels, danger confirm, attention
badge) · [composer-mode-switch.md](composer-mode-switch.md) (Plan → Agent) ·
[tools.md](tools.md) / [tool-call.md](tool-call.md) (lifecycle, footer slot, `Denied`, provenance) ·
[plan-todo.md](plan-todo.md) (`PlanApprovalSlot`) · [acting-indicator.md](acting-indicator.md)
(control requests) · [composer-status-strip.md](composer-status-strip.md) (`AwaitingYou`) ·
[subagent-tree.md](subagent-tree.md) / [background-jobs.md](background-jobs.md) (`NeedsYou` roll-ups) ·
[trace-view.md](trace-view.md) (hatched waiting segments) · family H diff viewer (diff preview link).

| Reference | What we take | Licence note |
|---|---|---|
| signoff-ui ApprovalCard, `useApprovalPolicy` [R107] (note 16) | once / session / always, risk level, reason the agent reads, rules by tool and argument pattern, audit trail | MIT — may port with attribution (we have not read its source yet; verify the repo licence file first) |
| Beautiful UI Approval Card [R106] [R133] (notes 16, 17) | paged multi-question "1 / 3", Skip / Continue | MIT (© 2026 Shane Levine) — may port with attribution |
| Codex CLI / app approvals (note 03, 05) | numbered options with keys, "don't ask again for commands starting with `X`", "tell Codex what to do differently", risk level, timed-out status, sandbox × approval axes | licensing.md lists MIT, note 08 reads Apache-2.0 — portable either way with attribution; re-check |
| Claude Code permission prompts (notes 03, 05) | "Yes, and don't ask again for `<pattern>`", "Yes, and switch to auto mode", Esc = no without comment | Proprietary — reference only |
| Claude desktop browser / computer use (note 03) | Allow once / Always allow / Deny per site; capability tier badge; high-reach warning | Proprietary — reference only |
| LangGraph Agent Inbox, AI SDK Confirmation (note 05) | accept / edit / respond / ignore; `approval-requested` → `output-denied` with reason | Protocol mapping only; Agent Inbox licence not recorded in our notes — study only until checked |
| affectUI Approval Card, Permission Request, Approval Gates, Decision Inbox (note 11) | risk badge + preview + shortcuts, pausing countdown, resolved-row morph, once / 1 hour / always + Revoke | Paid — study only |
| Vespin agent questions with "(recommended)" [R022] | recommended option marker | Study only |
| Secret redaction card [R049] | secret name visible, value never in chat | Study only |
| hermes-agent composer controls [R088] | ↑ answers the pending card | Licence not recorded in our notes — study only |
| Wensity UI AI Agent UI [R119] | approval status inside step rows | Study only |

**Open questions / gaps** (for the next revision):
- `AgentState::AwaitingApproval { request }` carries one request; the queue is new `oxyn-core` state
  (proposed here). Architecture.md should list `ApprovalQueue` beside `RunLog`.
- ~~Bypass vs Critical~~ resolved 2026-11-12: Bypass still stops for Critical requests; the permission
  selector spec now says so.
- Risk floors per kind and the destructive-verb list are proposals; the classifier is the app's.
- No source documents a shield / hand glyph set (note 03 §1D); our glyphs are proposals.
