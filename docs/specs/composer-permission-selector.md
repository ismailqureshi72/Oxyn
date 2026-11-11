**Status:** draft v0.1 (2026-11-11).

# Spec: composer permission / autonomy selector

Follows the 10-point standard in [README.md](README.md) at the depth of the worked example
[composer-send-control.md](composer-send-control.md). Target **6 free + 16 Pro variants** on one state
machine. Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
Before (and during) a run the user decides **how much the agent may do without asking**: only read,
ask before every change, edit files freely but ask before commands, run everything inside a sandbox, or
skip all checks. The selector is a chip in the composer row; its state is the safety posture of the
session. The user needs to see the posture at a glance (and feel it when it is dangerous), raise or lower
it quickly, understand the scope (this session vs remembered), and see how it connects to the approval
cards that appear in the transcript.

Models seen (research 03, 05, 04):
- **Preset ladder** — Claude: Manual / Accept edits / Plan / Auto (classifier reviews) / Bypass
  permissions, ⌘⇧M, Shift+Tab cycling, status glyphs `⏸ manual mode on`, `⏵⏵ accept edits on`,
  `⏵⏵ bypass permissions on`; Codex presets Default / Full access / Custom via `/permissions`, banner with
  a magenta "YOLO mode" row; Paradev Auto / Semi-auto / Manual + "Full access" chip; Warp Always ask /
  Always allow / agent decides; Claude in Chrome Manually approve / Automatically approve / Skip all
  approvals (research 04).
- **Autonomy ladder** — Factory Autonomy **Off / Low / Medium / High**, Ctrl+L cycles; each command is
  tagged low / medium / high risk and auto-runs at or below the chosen level (research 03: "the cleanest
  mental model"); Replit 4 autonomy levels incl. Max Autonomy; Kiro Autopilot vs Supervised.
- **Two axes** — Codex sandbox `read-only | workspace-write | danger-full-access` × approval
  `on-request | never`, network off by default in workspace-write, `.git` protected (research 05).
- Composer chips: "Request approval" [R044], "Bypass all" shield [R036], "Full access / Supervised" lock
  [R003], "… · Full access" [R053].

Oxyn ships **one component with three models** (`Presets`, `Autonomy`, `TwoAxis`) sharing one state
machine, risk-tier visuals and the danger-confirm flow.

## 2. State machine

```
                         click / Mod+Alt+P / "/permissions"
  Idle(level) ──hover──▶ Hover ───────────────────────────────▶ Open(list | dial | matrix)
   ▲  ▲   ▲                                                         │ ↑↓ / drag / click
   │  │   │                                                         ▼
   │  │   │                                               Highlighted(level)
   │  │   │                          lower or same tier ──────┤──────── raise
   │  │   │                                  ▼                         ▼
   │  │   └─────────────────────────── Applying(level) ◀──── tier ≤ 2: apply   tier ≥ 3 (Full access / Bypass)
   │  │                                      │                                     ▼
   │  │                                      │                          ConfirmDanger(level, scope)
   │  │                                      │                 Arming (600 ms) ─▶ Armed ─ confirm ─▶ Applying
   │  │                                      │                          │ cancel / Esc / click-out
   │  │                                      │                          └─────────────▶ Idle(prev)
   │  │                    agent idle ▼                    agent busy ▼
   │  └──────────── Applied(level, scope)          lower: AppliedNow (pending auto-runs re-gated)
   │                                               raise: PendingNext(level) ── run ends ─▶ Applied
   │
   ├─ AwaitingApproval in transcript ─▶ Attention(n) overlay (badge n, one amber pulse); click → focus card
   ├─ approval card "Yes, and switch to Auto-edit" ─▶ Applying(AutoEdit, session) (card-driven raise)
   ├─ session ends with scope = Session ─▶ Reverted(to persistent default) (shown next session start)
   └─ Locked(policy, max_level) · Disabled(reason) · SandboxUnavailable(reason)
```

Rules:
1. Levels are ordered by **risk tier** 0–4. Any raise to tier ≥ 3 requires `ConfirmDanger`; Bypass
   (tier 4) always requires it, every time, and its scope defaults to **this session**.
2. **Lowering applies immediately, even mid-run**: tool calls not yet started are re-evaluated against
   the new level and become approval cards if needed. **Raising mid-run waits for the next turn** unless
   the raise comes from an approval card the user just answered (that is an explicit in-context consent).
3. Keyboard cycling (`Alt+Shift+P` proposed, or Shift+Tab in merged mode) moves only through tiers 0–2.
   Tiers 3–4 are reachable only through the open list + confirm. No shortcut can enable Bypass.
4. "Always allow `<prefix>`" on an approval card adds a **rule**; it never changes the level. Rules and
   level are shown together in the popover ("Ask · 3 rules").
5. Policy-managed limits (org / workspace) cap the selectable max; capped rows are disabled-with-reason.
6. Scope options: **This session** (default for raises), **This project**, **Always** (user). Persisting
   Bypass requires the app to set `allow_persistent_bypass = true`; default false.

## 3. States table

**Levels (Presets model; labels are i18n tokens):**

| Tier | Level | Meaning (what auto-runs) | Glyph | Tone token |
|---|---|---|---|---|
| 0 | `ReadOnly` | reads / search only; no writes, no commands with side effects | lock | `risk.0` neutral |
| 1 | `Ask` ("Request approval") | asks before every write and command | hand | `risk.1` info |
| 2 | `AutoEdit` ("Accept edits") | file edits inside the workspace auto; commands and network ask | pencil + shield | `risk.2` accent |
| 3 | `FullAccess` | edits + commands auto within the sandbox; network per sandbox setting | open shield | `risk.3` warning (amber) |
| 4 | `Bypass` ("Bypass all") | no approvals, no sandbox restrictions | shield-off + warning | `risk.4` danger (red) |

Autonomy model maps Off → 1, Low → 2 (auto-runs low-risk), Medium → 3 (≤ medium), High → 3 (≤ high);
Bypass stays a separate, explicit row. TwoAxis model shows `sandbox × approval` and derives the tier
from the riskier axis. Glyph meanings follow the inferred icon set in research 03/05 (shield = permission,
hand = awaiting approval, lock = read-only); they are our proposal, no source documents them.

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle(level)` | chip: glyph + short label ("Ask", "Auto-edit", "Full access", "Bypass") tinted by tier; tier ≥ 3 adds 1 px tinted composer border; tier 4 adds a "Bypass on" text tag | open | — | Tab focus; `Mod+Alt+P` open (proposed); `Alt+Shift+P` cycle tiers 0–2 (proposed) |
| `Hover` | bg `chip.hover`; tooltip 500 ms: level description + scope ("This session") + rules count | open | — | — |
| `Focus` | 2 px focus ring (tier 4: ring uses danger tone) | open | — | Enter/Space/↓ open |
| `Open(list)` | levels top→bottom low→high risk, each: glyph, label, one-line "what auto-runs", tier dot; sandbox row; rules row ("3 always-allow rules ›"); scope selector | select | edit rules, sandbox settings | ↑↓, 1–5 jump, Enter, Esc |
| `Open(dial)` (Autonomy) | 180° dial Off · Low · Med · High with detents; risk legend under it | drag / click detent | — | ← → step, Home/End |
| `Open(matrix)` (TwoAxis) | 3 × 2 grid sandbox × approval, cells tinted by derived tier | select cell | network toggle, protected paths | arrows move, Enter |
| `Highlighted(level)` | row bg; footer explains consequences for this level | select | — | — |
| `ConfirmDanger(level)` | inline confirm panel replaces the list: title "Allow full access?" / "Bypass all approvals?", 2–4 consequence lines, scope (session pre-selected), danger button disabled during arming | confirm (after arming) | cancel | Tab cycles confirm panel; Enter only works once armed; Esc cancels |
| `Arming` | danger button fills left→right over 600 ms (prevents click-through) | — | cancel | — |
| `Armed` | danger button enabled "Bypass for this session" | confirm | cancel | Enter confirm |
| `Applying` / `Applied(level, scope)` | chip morphs to new glyph + tone; border tint follows; scope dot (filled = remembered, hollow = session) | — | undo toast 5 s for raises ("Full access on · Undo") | `Mod+Z` while toast visible |
| `AppliedNow` (lowered mid-run) | note 2.4 s: "Ask: remaining actions will need your approval" | — | — | — |
| `PendingNext(level)` | current chip + dashed badge "→ Auto-edit" | — | cancel (× on badge) | Esc on badge |
| `Attention(n)` | chip gets amber badge "n" + one pulse; tooltip "2 actions waiting for approval" | focus first approval card | "Approve all low-risk" (if app enables) | `Mod+Alt+P` then `A` jumps to card (proposed) |
| `Reverted` | at next session start the chip shows the persistent level with a one-time note "Back to Ask (full access was for last session)" | — | "Keep full access" (re-confirm) | — |
| `Locked(policy, max)` | rows above `max` disabled with "Limited by your organisation"; if fully locked, chip shows lock and no chevron | — | — | Enter announces reason |
| `SandboxUnavailable(reason)` | tier 3 row shows "Sandbox not available on this machine"; selecting it is blocked or escalates to the Bypass confirm (app policy) | — | "Learn more" | — |
| `Disabled(reason)` | muted chip, tooltip | — | — | focusable, announces |

## 4. Micro-interactions
- **Hover / press / focus:** shared chip grammar (80 ms bg, 0.97 press spring, 2 px ring).
- **Tier tone:** chip glyph + label + 1 px composer border crossfade to the tier tone. Tier 3 border is
  amber at 60 % alpha; tier 4 border is danger at 80 % alpha plus a **static** 45° hatch on the chip
  background at 8 % alpha (no animated stripes; danger should be calm and constant, not flashy).
- **Glyph morph:** lock → hand → pencil-shield → open shield → shield-off; the shield body is shared
  across glyphs so morphs read as one object changing (path interpolation in GPUI, crossfade in Dioxus).
- **Arming:** the danger button's fill sweeps left→right in 600 ms; hovering away does not cancel; a
  second click before armed does nothing and gives a 2-cycle 3 px shake of the button.
- **Confirm → applied:** confirm panel collapses back into the chip (height to 0, then popover closes);
  chip does a single scale 1 → 1.06 → 1 with the new tone.
- **Lowering:** no celebration; chip tone simply cools; pending auto-run tool rows in the transcript flip to
  "Awaiting approval" (tool-call spec owns that row).
- **Attention badge:** pops in with scale 0 → 1 (spring), the chip pulses amber once (opacity 1 → 0.6 →
  1); repeats at most once per new approval, never loops.
- **Approval-card link:** clicking the chip in `Attention` scrolls the transcript to the first pending card
  and flashes its border once; conversely each approval card header shows a mini read-only copy of this
  chip ("Ask · this session") so the user knows why they are being asked.
- **Auto-approved provenance:** tool rows that ran because of the current level carry a 10 px glyph of
  that level (hover: "Ran automatically · Auto-edit"); in Bypass the glyph is the danger shield.
- **Reverted note:** slides down under the chip once, 4 s, dismissible.

## 5. Timing table

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Hover bg | 80 ms | 0 | ease-out | — |
| Tooltip | 120 ms | 500 ms | ease-out | — |
| Press | ~120 ms | 0 | spring 600 / 30 | — |
| Popover open / close | ~200 ms / 120 ms | 0 | spring 380 / 32 / ease-in | shared |
| Dial detent snap | ~160 ms | 0 | spring 520 / 34 | — |
| Tone crossfade (chip + border) | 240 ms | 0 | ease-in-out | — |
| Glyph morph | 220 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | — |
| Confirm panel in (list → panel) | 200 ms | 0 | ease-out, height spring 320 / 30 | — |
| Arming fill | 600 ms | 0 | linear | proposed safety delay |
| Early-click shake | 240 ms, 2 cycles, 3 px | 0 | linear | — |
| Applied scale pop | 260 ms | 0 | spring 400 / 20 | raises only |
| Undo toast | 5 s visible | 0 | 160 ms in/out | raises only |
| Attention badge in | ~180 ms | 0 | spring 500 / 22 | — |
| Attention pulse | 600 ms | 0 | ease-in-out | once per new approval |
| Card flash on jump | 400 ms | scroll end | ease-out | — |
| Notes (AppliedNow / Reverted) | 160 in / 2400–4000 hold / 160 out | 0 | ease-out / ease-in | — |
| **Reduced motion** | tone and glyph swap with 80 ms opacity; arming shows a static "Available in 0.6 s" text then enables (delay is kept — it is a safety rule, not decoration); no pulse (badge only); no shake (inline text "Wait a moment"); no scale pop | — | linear | — |

## 6. Variants

### Free (6)
| Name | Idea |
|---|---|
| `permission-selector/chip-list` | Chip + popover list of preset levels with tier dots, rules row and scope; inline danger confirm |
| `permission-selector/segmented` | Tiers 0–2 as segments; "More ⌄" holds Full access / Bypass behind confirm |
| `permission-selector/autonomy-dial` | Off · Low · Med · High dial with per-command risk legend |
| `permission-selector/toggle` | Binary "Request approval" / "Bypass all" style chip with confirm on enable |
| `permission-selector/glyph` | TUI status line `⏵⏵ accept edits on (shift+tab to cycle)`-style, tinted per tier |
| `permission-selector/two-axis` | Sandbox × approval matrix with derived tier, network toggle, protected paths tooltip |

### Pro (16)
| Name | Idea |
|---|---|
| `permission-selector/risk-ladder` | Vertical ladder; commands from the current plan shown as dots on their risk rungs |
| `permission-selector/shield-morph` | Single shield glyph morphs through all tiers with fill and crack (bypass) |
| `permission-selector/hold-to-bypass` | Bypass armed by press-and-hold 1.2 s with ring fill instead of a button |
| `permission-selector/sandbox-cube` | Isometric cube shows sandbox walls; walls fade as access widens |
| `permission-selector/policy-badge` | Shows org policy cap, rule count and audit-log link in one badge |
| `permission-selector/rules-drawer` | Always-allow rules as editable chips (pattern, scope, source) |
| `permission-selector/timer-scope` | Elevated level with a countdown ("Full access · 14:59 left"), auto-reverts |
| `permission-selector/classifier` | "Auto" level with a reviewer verdict chip on each auto-approved action |
| `permission-selector/heat-border` | Composer border warms with tier; bypass adds a slow edge glow |
| `permission-selector/lanes` | Splits the transcript margin into "auto-ran" vs "asked" lanes |
| `permission-selector/approval-inbox` | Chip expands into a mini inbox of pending approvals across agents |
| `permission-selector/capability-tiers` | Per-app / per-site capability badges (view / click / full) |
| `permission-selector/scope-slider` | Scope slider session → project → always with consequences text |
| `permission-selector/secret-guard` | Shows when secrets would be exposed at this level (masked key card idea [R049]) |
| `permission-selector/notch` | Desktop island shows tier colour sliver; bypass shows a persistent red dot |
| `permission-selector/voice-confirm` | Voice surface: spoken confirmation phrase required for Bypass |

## 7. Edge cases
- **Long labels / i18n** ("Automatisch bearbeiten"): chip uses `short_label`; popover rows wrap to 2 lines.
- **Custom levels:** apps can add levels between tiers; each declares a tier so visuals and confirm rules
  still apply.
- **RTL:** list order unchanged (risk is vertical); dial mirrors; arming fill runs right→left.
- **Tiny sizes (16–20 px):** glyph only; tier 3–4 add a 4 px corner dot so risk never depends on hue alone.
- **Dense IDE panels:** `glyph` or `segmented`; confirm opens as a small modal sheet if the popover would
  be < 240 px tall.
- **Touch:** rows 44 px; arming is the same 600 ms; `hold-to-bypass` uses haptic ticks.
- **Slow networks:** level is local; if the adapter must acknowledge (remote agent), chip shows
  `Applying` with a 2 px underline progress and reverts with a note on failure. Lowering is applied
  locally first (fail safe): remote tool calls that arrive after a lower are gated by the client.
- **Very fast agents:** an approval card may be answered before the badge animates; skip the pulse if the
  card resolves within 300 ms.
- **Very slow agents / long runs:** `PendingNext` persists; `timer-scope` variant keeps counting while
  the window is hidden (wall clock, not the motion clock).
- **Multiple agents / subagents:** subagents inherit the parent level (Factory: subagents inherit
  autonomy, research 03); the chip shows the parent level and the popover lists overrides.
- **Org policy changes mid-session:** if the current level exceeds a new cap, lower immediately
  (`AppliedNow`) with reason "Lowered by your organisation".
- **Sandbox missing:** Full access without a sandbox is effectively Bypass; app chooses to block or route
  to the Bypass confirm (`SandboxUnavailable`).
- **Mode/permission merged cycles:** dangerous tiers never enter any cycle.

## 8. Accessibility
- Chip: `button`, `aria-haspopup="dialog"` (popover contains list + scope + confirm), label per state:
  "Permissions: Ask before changes, this session. Change permissions", "Permissions: Bypass all approvals
  is on for this session. Dangerous", "Permissions: 2 actions waiting for your approval".
- List: `radiogroup` labelled "Permission level"; each `radio` has `aria-describedby` → its "what
  auto-runs" line and, for capped rows, the policy reason.
- Confirm panel: `alertdialog` with `aria-labelledby` (title) and `aria-describedby` (consequences);
  focus moves to Cancel first, not the danger button; the danger button exposes `aria-disabled="true"`
  while arming and announces "Available" when armed.
- Live region: polite for applied / pending / reverted ("Full access on for this session", "Ask will apply
  from your next message"); **assertive** for "Bypass all approvals is on" and for policy-forced lowering.
- Risk is conveyed by glyph shape, label text and tier dot, not only colour; danger tones meet 4.5:1 for
  text and 3:1 for borders in both themes; daltonized themes add the hatch pattern to tier 3 too.
- Focus order: … mode → **permission** → context → send. Approval cards are reachable from the chip via
  the `Attention` action.

## 9. Props sketch
```rust
pub struct PermissionSelector {
    pub agent: AgentStateRef,                 // AwaitingApproval { request } -> Attention; busy -> lower-now / raise-next
    pub model: PermissionModel,               // Presets(Vec<Level>) | Autonomy(Vec<AutonomyStep>) | TwoAxis { sandbox, approval }
    pub current: LevelId,
    pub scope: Scope,                         // Session (default for raises) | Project | Always
    pub pending: Option<LevelId>,
    pub pending_approvals: u32,               // from the transcript's open approval cards
    pub rules: Vec<AllowRule>,                // shown as count; edited in rules drawer
    pub policy: Option<PolicyCap>,            // { max_tier, reason, source }
    pub sandbox: SandboxStatus,               // Available { network: bool } | Unavailable { reason }
    pub confirm: DangerConfirm,               // { from_tier: 3, arming: 600 ms, style: Button | Hold(1200 ms) | Phrase }
    pub allow_persistent_bypass: bool,        // default false
    pub cycle_max_tier: u8,                   // default 2; shortcuts never pass it
    pub variant: PermissionVariant,
    pub shortcuts: PermissionShortcuts,       // open Mod+Alt+P, cycle Alt+Shift+P (proposed)
    pub labels: PermissionLabels,
    pub motion: MotionLevel,
    pub on_change: Callback<PermissionChange>, // { from, to, scope, applied: Now | NextTurn, source: User | ApprovalCard | Policy }
    pub on_focus_approval: Callback<()>,
}

pub struct Level { pub id: LevelId, pub tier: u8, pub label: SharedString, pub short_label: SharedString,
                   pub glyph: IconName, pub auto_runs: SharedString /* "File edits in this workspace" */ }

fn apply_timing(agent: &AgentState, from_tier: u8, to_tier: u8, source: Source) -> Applied {
    let busy = matches!(agent, AgentState::Submitted | AgentState::Thinking { .. } | AgentState::Streaming
        | AgentState::ToolRunning { .. } | AgentState::AwaitingApproval { .. } | AgentState::Steering);
    if !busy || to_tier <= from_tier || source == Source::ApprovalCard { Applied::Now } else { Applied::NextTurn }
}
```
`Attention(n)` is derived from `AgentState::AwaitingApproval { request }` plus the count of open cards; the
selector never owns the approval itself, it only links to it.

## 10. Related components and references
**Related:** approval card (allow once / session / always, deny with reason; header shows a mini copy of
this chip; "Yes, and switch to Auto-edit" raises the level); risk badge; rules editor / allowlist row;
sandbox indicator; approval inbox; audit log; mode switch
([composer-mode-switch.md](composer-mode-switch.md)) — modes may suggest a level; tool-call row
(auto-ran provenance glyph); send control ([composer-send-control.md](composer-send-control.md)) —
references the "Bypass all" chip; secret redaction card.

**References** (study only unless MIT/Apache):
- Claude Code permission modes, prompt options ("Yes, and don't ask again for `<pattern>`", "Yes, and
  switch to auto mode"), status glyphs, Shift+Tab cycling, ⌘⇧M (research 03, 05) — proprietary, reference
  only.
- Codex sandbox × approval axes, presets, "YOLO mode" banner row, "always allow commands starting with
  `X`" (research 03, 05) — permissive licence (licensing.md lists MIT; research 08 reads Apache-2.0 —
  re-check), portable with attribution.
- Factory Autonomy Off / Low / Medium / High with per-command risk; subagents inherit autonomy (research
  03) — closed, study only. Replit autonomy levels, Kiro Autopilot / Supervised, Warp approval options
  (Warp AGPL → study only), Paradev chips — study only.
- Claude in Chrome permission-mode dropdown and per-site approvals (research 04) — proprietary, study only.
- [R044] affectUI composer "Request approval" chip; affectUI Autonomy Dial, Permission Request
  (once / 1 hour / always + revoke) (research 01, 11) — paid, study only.
- [R036] BoardUI "Bypass all" shield toggle — free tier MIT, Pro study only.
- [R003] T3 Code "Full access / Supervised" lock under the composer; [R053] "Claude · Opus 5.5 · Medium ·
  Full access" — study only (T3 Code licence not checked in our notes).
- [R107] signoff-ui ApprovalCard (once / session / always, risk level, reason the agent reads) and
  `useApprovalPolicy` — MIT, port with attribution.
- [R049] secret redaction card ("Secrets stay out of the chat") — study only.
- Research 05 ApprovalCard / RiskBadge / PermissionModeSwitcher / SandboxIndicator / RuleEditor component
  breakdown — our own synthesis; the inferred icon set (lock / hand / shield) is a proposal.
