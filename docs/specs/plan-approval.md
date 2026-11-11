**Status:** draft v0.1 (2026-11-11).

# Spec: plan approval footer (`plan-approval`)

Part of family G ([approvals.md](approvals.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **3 free + 9 Pro = 12 variants**.
This is the component rendered in the plan card's **`approval: Option<PlanApprovalSlot>`**
([plan-todo.md](plan-todo.md) §3.2, §5 "Approval hand-off", §10): the plan card owns the plan, its steps,
versions, inline editing and the fold to the "Plan · 0/7" pill; this footer owns the **decision**. The
Plan → Agent mode choreography is [composer-mode-switch.md](composer-mode-switch.md) §4 "Plan approved →
Agent"; level raises go through [composer-permission-selector.md](composer-permission-selector.md).
Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
In Plan mode (or whenever the app gates execution on a plan) the agent has proposed what it will do and
waits. The user wants to: **approve and let it build**; **approve and choose how freely it runs** (switch
to Agent mode at Ask / Auto-edit / Full access); **edit the plan themselves** and approve the edited
version; **send it back with feedback** so the agent revises; or **reject** it outright. The footer must
make the run level of the approval explicit (a plan approved "with Full access" is a much bigger yes than
one approved "with Ask"), and must never approve a version of the plan the user did not see.

What we saw (note 03, note 05): Cursor's editable plan with a Build button and a minimised "Plan" pill;
Jules' Approve plan with an auto-approve timer; Factory combines "proceed" with an autonomy level in one
approval and spec approval sets autonomy; Lovable keeps plan versions ("Viewing version 2 of 3"); Claude
Code's plan-approval prompt offers "Yes, and use auto mode" / auto-accept edits / manual (note 05). The
plan card spec already lists the footer's actions: **Approve, Approve with autonomy, Request changes,
Reject** (plan-todo.md §4.2).

Anatomy (`footer`, plan v2, current level Ask):
```
│ 7 steps · touches 4 files · runs cargo test, cargo fmt            v2 · 3 changes since v1 ›  │  scope line + version chip
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│ [ Approve & build  ⏎ ]  [ Run with: Ask ⌄  M ]   [ Edit plan  E ]  [ Request changes  R ]  ⌄ │
│   switches to Agent mode · edits and commands will ask                         Reject  N     │  consequence line
```

## 2. State machine

```
   plan card: Drafting / Streaming                    plan card enters AwaitingApproval { request }
   ─────────────────────────────▶ Hidden ─────────────────────────────────────────────▶ Entering (200 ms rise)
                                                                                           │ input guard 400 ms
                                                                                           ▼
       ┌────────────────────────────────────────── Ready(version v, level L) ◀───────────────┐
       │ Enter / Y           │ M                     │ E                     │ R                 │ N
       ▼                     ▼                       ▼                       ▼                   ▼
   Submitting(approve, L)  LevelMenu ─pick L'─┐    EditingPlan(edits n)   FeedbackEditing      RejectConfirm (note optional)
       ▲                     │ Esc            │      │ Approve edited ──▶ Submitting(approve-edited, L)        │ confirm
       │                     ▼                │      │ Discard edits ──▶ Ready                                  ▼
       │                   Ready              │      └ Esc (keeps edits) ──▶ Ready(edited)              Submitting(reject)
       │       L' tier ≤ 2 ───────────────────┤                              │ send
       │       L' tier ≥ 3 ──▶ ConfirmLevel: Arming(600 ms) ─▶ Armed ─confirm─┘  ▼
       │                                                            Submitting(request-changes, text)
       └──────────────────────────────────────────────────────────────────────────┘
   Submitting ── ack ──▶ Approved ── (≤ 600 ms, plan card folds to pill; mode chip Plan → Agent) ──▶ Done
              ├─ ack ──▶ ChangesRequested ──▶ Revising ("Revising plan…") ── v+1 streams ──▶ Hidden (new footer for v+1)
              ├─ ack ──▶ Rejected (footer collapses to "Rejected" + note)
              └─ fail ─▶ SubmitFailed ── Retry ─▶ Submitting
   Version guard:  agent changes the plan while Ready ─▶ PlanChanged("Plan updated · review changes")
                   approve disabled for 600 ms after the last change, then Ready(v+1)
   External:  run stopped ─▶ Stale       expires ─▶ Expired (not approved; "Ask again")       answered elsewhere ─▶ Approved/Rejected(origin)
   Pro auto-approve (opt-in): Ready ── countdown ends ─▶ Submitting(approve, L);  hover / focus / any edit ─▶ CountdownPaused
```

Rules (proposed):
1. **Approval is bound to a version.** The decision carries `plan.version` and a content hash; if the
   plan changes after the footer was shown, Approve is disabled until 600 ms after the last change and
   the footer says what changed (plan-todo.md coalesces rewrites in 300 ms).
2. **The run level is part of the approval.** Default is the current permission level (shown, never
   hidden). Choosing a level of tier ≥ 3 (Full access) runs the selector's danger confirm inline (600 ms
   arming). **Bypass is never offered from the plan footer.**
3. **Approve switches mode** when the app is in Plan mode: Plan → Agent via the shared spark choreography.
   Apps that keep Plan mode for step-by-step execution set `mode_after = Keep`.
4. **User edits ride with the approval**: "Approve edited plan" sends the edited steps; the agent must
   treat them as the plan (plan-todo.md: edited steps carry a "you edited" tag).
5. **Request changes** keeps the plan alive (agent revises → v+1, old version `Superseded`); **Reject**
   ends the planning turn. Both carry text the agent reads; Request changes requires text, Reject does
   not.
6. No "always" for plans (approvals.md §2.4). Auto-approve countdowns are off by default, only in Pro,
   only when the chosen level is tier ≤ 2, and pause on hover, focus or any edit.

## 3. States table
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing (plan still drafting / streaming); plan card shows "Planning" | — | — | — |
| `Entering` | footer rises from the plan card's bottom edge | — | — | — |
| `Ready(v, L)` | scope line (steps, files, commands, each with a risk glyph if any step is ≥ High), version chip, Approve & build, Run with: L, Edit plan, Request changes, Reject; consequence line ("switches to Agent mode · edits and commands will ask") | **Approve & build** | level, edit, request changes, reject | Enter/Y approve, M level menu, E edit, R request changes, N reject (all after the 400 ms guard) |
| `LevelMenu` | list of levels tier 0–3 with "what auto-runs" lines (from the selector); current level marked; Full access marked ▲ | pick | Esc | ↑/↓, 1–4, Enter, Esc |
| `ConfirmLevel` / `Arming` / `Armed` | inline panel "Run this plan with Full access?" + consequences; confirm fills over 600 ms | confirm (armed) | cancel | Enter after armed; Esc cancel |
| `EditingPlan(n)` | footer shows "Editing · 3 changes" + Approve edited plan + Discard edits; plan card shows its inline edit affordances (plan-todo.md) | Approve edited plan | Discard edits, Done editing | ⌘/Ctrl+Enter approve edited, Esc done editing |
| `FeedbackEditing` | multi-line field "What should change?", 2 000-char limit, suggestion chips ("Smaller steps", "Add tests first", "Don't touch the API") (copy in-house), "Sent to the agent" | Send feedback | Back | Enter send (⇧Enter newline), Esc back |
| `RejectConfirm` | "Reject this plan?" + optional note field | Reject | Back | Enter reject, Esc back |
| `PlanChanged` | note "Plan updated · 2 added, 1 reworded · Review" (links to plan card highlights); Approve disabled with reason "Plan just changed" | Review changes | — | Enter review |
| `Submitting` | 2 px underline progress on the pressed button after 300 ms | — | — | — |
| `SubmitFailed` | "Couldn't reach the agent · Retry"; plan stays unapproved | Retry | — | Enter |
| `Approved` | ✓ "Approved · runs with Ask" for ≤ 600 ms, then the plan card folds to its pill (plan-todo.md) and the mode chip morphs | — | — | focus → composer |
| `Revising` | footer collapses to "Revising plan…" (shimmer, `text-shimmer/calm`); your note quoted under it | — | Stop (composer) | — |
| `Rejected` | "Rejected" + your note; plan card dims | — | — | — |
| `Stale` | "Run stopped · plan not approved"; actions removed | — | — | — |
| `Expired` | "Timed out · not approved" + Ask again | Ask again | — | Enter |
| `CountdownRunning` (Pro) | ring on Approve & build with seconds "Builds in 0:10"; "Pause" | — | pause | any key or hover pauses |
| `CountdownPaused` (Pro) | ring frozen + "Paused" | Approve | resume | — |

## 4. Micro-interactions
- **Rise.** As plan-todo.md: footer rises from the card's bottom edge (200 ms `ease.out`); the scope line
  fills in after 80 ms; numbers roll once (`dur.count.roll`) as the plan's file / command counts settle.
- **Level chip.** "Run with: Ask ⌄" carries the selector's glyph and tier tone; changing it crossfades
  glyph + tone (240 ms) and rewrites the consequence line (80 ms out / 160 ms in, y 3 px). Picking Full
  access morphs the footer into the inline confirm panel (200 ms height spring) with the 600 ms arming fill.
- **Approve.** Press scale 0.97 → the button turns into "Approved ✓" (160 ms), footer collapses (220 ms
  `spring.snap`), the plan card folds to the "Plan · 0/7" pill (280 ms `ease.in-out`), the todo dock
  unfolds, and in Plan mode the Approve button **flies a spark to the mode chip** which morphs Plan →
  Agent (composer-mode-switch.md); if the level changed, the permission chip morphs on the same frame as the
  mode chip (one choreography, not two).
- **Edit plan.** Footer slides its button row 8 px down and swaps to the editing row; the change counter
  rolls with each edit; "Discard edits" restores steps with a 200 ms crossfade per changed step.
- **Request changes.** The footer height follows the field (`spring.snap`); on send the note lifts and
  docks into the Revising line as a quote; the old version dims to 60 % (plan-todo.md) and v+1 streams.
- **Plan changed while open.** The version chip pulses once (opacity 1 → 0.6 → 1, 600 ms); Approve shows a
  600 ms re-arm fill so a click aimed at the old version cannot land on the new one.
- **Reject.** Footer collapses to "Rejected" (220 ms); no shake, no red flash.
- **Countdown (Pro).** Ring drains 1 Hz with digit swaps; hover / focus freezes it immediately and shows
  "Paused"; any edit cancels it for this version.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Footer rise | 200 ms | 0 | `ease.out` | — | 60 Hz |
| Scope line fill / count roll | 160 ms / 280 ms | 80 ms | `ease.out` | 30 ms | 30 Hz |
| Input guard | 400 ms | from mount | — | — | — |
| Level glyph + tone crossfade | 240 ms | 0 | `ease-in-out` | — | 60 Hz |
| Consequence line swap | 80 ms out / 160 ms in, 3 px | 0 | `ease.in` / `ease.out` | — | 60 Hz |
| Confirm panel in | 200 ms | 0 | height `spring.snap` | — | 60 Hz |
| Arming (Full access) | 600 ms | 0 | linear | — | 60 Hz |
| Approve → "Approved ✓" | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Footer collapse | 220 ms | 0 | `spring.snap` | — | 60 Hz |
| Card fold to pill | 280 ms | after collapse | `ease.in-out` | — | 60 Hz |
| Spark to mode chip + chip morph | per composer-mode-switch.md | after fold starts | — | — | 60 Hz |
| Editing row swap | 200 ms, 8 px | 0 | `spring.snap` | — | 60 Hz |
| Feedback field grow | per line | 0 | `spring.snap` | — | 60 Hz |
| Plan-changed re-arm | 600 ms after last change | coalesce 300 ms | linear | — | 60 Hz |
| Version chip pulse | 600 ms once | 0 | `ease-in-out` | — | 60 Hz |
| Countdown (Pro) | 1 s steps | — | digit swap | — | 1 Hz |
| **Reduced motion** | 80 ms opacity crossfades; heights jump; no spark (mode and permission chips swap with 80 ms fade); arming and re-arm show static text "Available in 0.6 s" (delays kept); no pulse (version chip shows a static "updated" dot for 4 s); countdown digits swap, no ring drain | — | linear | none | 1 Hz idle |

## 6. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `plan-approval/footer` | Default slot footer: scope line, version chip, Approve & build, Run with: level, Edit plan, Request changes, Reject | chat, desktop, ide |
| `plan-approval/bar` | Sticky bar above the composer while the plan card scrolls; same actions; collapses to "Plan ready · Review ⏎" when the card is in view | chat, desktop, mobile |
| `plan-approval/terminal` | TUI prompt under the plan: `Approve plan v2? 1 Yes, build (y) · 2 Yes, run with auto-edit (a) · 3 Edit (e) · 4 No, revise: … (esc)` | terminal, ide |

### Pro (9)
| Name | One-line idea |
|---|---|
| `plan-approval/autonomy` | One-tap "Approve and run with: Ask · Auto-edit · Full" segmented row (Factory idea, note 03; used by `plan-card/autonomy`) |
| `plan-approval/countdown` | Opt-in auto-approve ring on Approve, pauses on hover / focus / edit, tier ≤ 2 only (Jules idea, note 03; used by `plan-card/countdown`) |
| `plan-approval/scope-preview` | Before approving, lists files, commands and hosts the plan names with risk glyphs, and which will still ask at the chosen level |
| `plan-approval/step-gates` | Approve now, but mark individual steps "ask me before this step" (checkpoint gates in the step timeline) |
| `plan-approval/partial` | "Approve steps 1–3" range handle; the rest stay proposed and re-ask when reached |
| `plan-approval/version-diff` | Footer shows a word-level diff summary v1 → v2 with "Approve v2" bound to the shown version (Lovable versions idea, note 03) |
| `plan-approval/comments` | Inline comments on steps collected into one "Request changes" bundle the agent reads |
| `plan-approval/review-bar` | Composer merged with Approve / Request changes, typing goes straight to feedback (`composer/review-bar` idea, composer.md) |
| `plan-approval/voice` | Voice: agent reads the plan summary; "approve", "approve with auto-edit", "change step three…" |

## 7. Edge cases
- **Very long plans (> 30 steps)**: scope line summarises ("32 steps · 18 files"); Approve still binds to
  the whole version; `partial` (Pro) for ranges.
- **Plan streams slowly**: footer stays `Hidden` until the plan card reaches `Proposed` /
  `AwaitingApproval`; no "approve while streaming".
- **Agent revises while the user is editing**: user edits are kept as a draft; the footer shows "The agent
  updated the plan · Keep my edits / Use v3"; nothing merges silently.
- **Level capped by policy**: options above the cap disabled with "Limited by your organisation"
  (selector `Locked`).
- **Sandbox unavailable** and Full access chosen: routes through the selector's `SandboxUnavailable` rule
  (blocked or Bypass-style confirm, app policy).
- **Not in Plan mode** (plan gate from an Agent-mode run): Approve does not switch modes; label "Approve &
  continue".
- **Two plans pending** (sub-agents): each plan card has its own footer; the queue pager from
  approval-card.md appears in the status strip's Review, not inside plan cards.
- **Run stopped while editing**: edits kept locally in the card (plan-todo.md draft); footer `Stale`.
- **RTL**: mirrors; version chip moves inline-start; arming fills from inline-start.
- **Tiny / dense IDE (≤ 320 px)**: buttons become Approve ⏎ + overflow (level, edit, request changes,
  reject); scope line truncates with tooltip.
- **Touch**: 44 px buttons; level menu as a bottom sheet; feedback field opens the keyboard with the
  plan still visible above (sheet ≤ 50 % height).
- **Slow network**: Submitting underline after 300 ms; the mode chip does **not** switch until the
  adapter acknowledges the approval.
- **Very fast agent**: if execution starts in < 600 ms after Approve, the Approved hold is cut short; the
  fold still runs to completion (no skipped frames mid-morph).

## 8. Accessibility
- Footer: `role="group"` labelled "Plan approval, version 2, 7 steps"; consequence line is
  `aria-describedby` of Approve ("Approve and build. Switches to Agent mode. Edits and commands will ask").
- Arrival: assertive "Plan ready for approval: 7 steps. Press Enter to approve, R to request changes"
  (owned here; plan-todo.md defers it to family G).
- Level menu: `radiogroup` "Run with"; Full access row has "High risk" in its name, not only its tone.
- Confirm panel: `alertdialog`, focus on Cancel; armed state announced.
- Plan changed: polite "Plan updated, 2 steps added. Approve is available again" after the re-arm.
- Countdown (Pro): announced at start and when paused; never announces every second; pauses on focus so a
  keyboard user is never raced.
- Focus after Approve: composer; after Request changes: the plan card heading (so the user hears v+1
  arrive); after Reject: composer.
- Keys are listed via `aria-keyshortcuts` and can be remapped; contrast and shape rules as approvals.md §8.

## 9. Props sketch
```rust
/// Rendered by PlanCard in its `approval: Option<PlanApprovalSlot>` (plan-todo.md §10).
pub struct PlanApprovalSlot {
    pub request: ApprovalId,                // ApprovalKind::Plan { plan } (approvals.md §2.2)
    pub variant: PlanApprovalVariant,       // Footer | Bar | Terminal | Pro(..)
    pub mode_after: ModeAfter,              // SwitchTo(ModeId::Agent) (default in Plan mode) | Keep
    pub levels: Vec<LevelId>,               // offered run levels; default: selector tiers 0–3, never Bypass
    pub default_level: LevelId,             // current permission level
    pub allow_edit: bool,                   // true
    pub feedback_required_for_changes: bool,// true
    pub auto_approve: Option<AutoApprove>,  // Pro only: { after: Duration, max_tier: 2, pause_on: HoverFocusEdit }
    pub rearm_after_change: Duration,       // 600 ms
    pub guard: Duration,                    // 400 ms
    pub labels: PlanApprovalLabels,
    pub motion: MotionLevel,
    pub on_decide: Callback<PlanDecision>,
}

pub enum PlanDecision {
    Approve { version: u16, hash: PlanHash, level: LevelId, mode_after: ModeAfter },
    ApproveEdited { version: u16, edits: Vec<PlanEdit>, level: LevelId, mode_after: ModeAfter },
    RequestChanges { version: u16, feedback: SharedString },      // → plan Revising, agent writes v+1
    Reject { version: u16, note: Option<SharedString> },
}

impl PlanApprovalSlot {
    pub fn visible(agent: &AgentState, plan: &Plan) -> bool {
        matches!(plan.state, PlanState::AwaitingApproval { .. })
            && matches!(agent, AgentState::AwaitingApproval { request } if Some(*request) == plan.request_id())
    }
}
```
On `Approve`, the app forwards `level` to the permission selector as
`PermissionChange { source: ApprovalCard, applied: Now }` and `mode_after` to the mode switch; both chips
morph on the same frame (§4).

## 10. Related components and references
**Related:** [plan-todo.md](plan-todo.md) (plan card, versions, edits, fold, todo dock) ·
[composer-mode-switch.md](composer-mode-switch.md) (Plan → Agent spark) ·
[composer-permission-selector.md](composer-permission-selector.md) (levels, danger confirm, policy cap) ·
[approvals.md](approvals.md) (model, audit) · [approval-card.md](approval-card.md) (step-level approvals
during execution) · [composer-queue-stack.md](composer-queue-stack.md) (edits during execution become
steering notes) · [text-shimmer.md](text-shimmer.md) (Revising line).

| Reference | What we take | Licence note |
|---|---|---|
| Cursor plan mode (notes 03, 08) | editable plan, Build, minimise to pill | Proprietary — reference only |
| Jules Approve plan + auto-approve timer (note 03) | countdown idea (Pro, opt-in, pausable) | Proprietary — reference only |
| Factory spec approval with autonomy (note 03) | approve + level in one action | Proprietary — reference only |
| Lovable plan versions (note 03) | "version 2 of 3", bind approval to a version | Proprietary — reference only |
| Claude Code plan-approval prompt (note 05) | approve with auto mode / accept edits / manual | Proprietary — reference only |
| Codex Plan collaboration mode (note 03) | mode label | MIT per licensing.md (note 08 reads Apache-2.0) — portable with attribution |
| affectUI Agent Plan, Approval Gates (note 11) | names only | Paid — study only |
