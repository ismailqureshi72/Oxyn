**Status:** draft v0.1 (2026-11-11).

# Spec: approval card & command-approval footer (`approval-card`)

Part of family G ([approvals.md](approvals.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **5 free + 14 Pro = 19 variants** on one
state machine. Implements the **approval footer slot** that [tool-call.md](tool-call.md) §5 "Approval
hand-off" and [tools.md](tools.md) §2.2 hand to family G. The request model, risk tiers, decision set,
timeout and batching rules are in [approvals.md](approvals.md) §2; this file is the visual component.
Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
The agent is blocked on one action and the user decides, usually in under three seconds, whether it may
proceed. The card must show **exactly what will run** (argument preview), **how risky it is** (risk badge),
**why the agent wants it** (quoted reason), and offer **how far the yes goes** (once / session / always)
or **a no the agent learns from** (deny with reason). It appears in two places:
- **Command-approval footer** (`approval-card/footer`, default): inside the tool-call card whose call is
  `AwaitingApproval`; the call's detail (command, diff) is already expanded above it (tools.md §4 rule 2,
  Warp's expand-while-pending idea, note 08). The footer adds only the decision row.
- **Inline approval card** (`approval-card/inline`): a stand-alone card in the transcript for requests
  that do not belong to a visible tool row (network, MCP with redacted args, spend, credential, questions,
  requests from a collapsed sub-agent).

Anatomy (`footer`, High risk, Command kind, inside a tool-call card):
```
╭─ ✋ Run cargo publish  · crates/oxyn-core          ⏸ 0:07  ──────────────╮  tool-call header (family F)
│  $ cargo publish --allow-dirty                                          │  argument preview (mono, from the row)
│    cwd ~/oxyn · sandbox: off · network: on                              │
├─────────────────────────────────────────────────────────────────────────┤
│ ▲ High risk · leaves the sandbox · publishes to a registry     1 of 3 ‹ › │  risk line + queue pager
│ "Release 0.3.0 now that tests pass."  — Agent                          │  quoted reason
│ [ Allow once  Y ]  [ For this session  S ]  [ Always…  A ]  [ Deny  N ]  ⌄ │  decisions (Always… disabled for High by default)
│  or type why not in the composer · Ask · this session                    │  hint + mini permission chip
╰─────────────────────────────────────────────────────────────────────────╯
```
`⌄` opens the overflow: Deny and tell the agent… (R), Edit and allow (E), Allow and switch to Auto-edit
(⇧Y), Open diff (D), Copy command.

## 2. State machine

```
           request opened (approvals.md §3)                   input guard 400 ms
  (none) ───────────────────────────────▶ Entering ─────────────────────────────▶ Open(tier)
                                         (200 ms rise)                              │ ▲
                                                                                    │ │ blur
                                    Tab / jump ⌘⇧A / click  ┌──────────────── Focused ┘
                                                            ▼
               ┌──────────────────────────────── Focused ───────────────────────────────────┐
               │ Y (Low/Med)   S          A                 E              R / composer text  │ N / Esc
               ▼               ▼          ▼                 ▼              ▼                  ▼
       Submitting ◀────── Submitting  PatternEditing ─✓─▶ EditingArgs ─✓─▶ ReasonEditing ─send─▶ Submitting(deny)
               ▲                          │ Esc back          │ Esc back       │ Esc back
               │  High:  Open ─(600 ms)─▶ Arming ─▶ Armed ──Y──▶ Submitting
               │  Critical: Y/Space/press held ─▶ Holding(p 0→1, 1200 ms) ── p = 1 ──▶ Submitting
               │                                  └─ release early ─▶ Draining (200 ms) ─▶ Focused
               │
     Submitting ── ack ──▶ Resolving (220 ms collapse) ──▶ Resolved(row: "Allowed once · 0:07")
               └─ fail ──▶ SubmitFailed ("Couldn't reach the agent · Retry") ── Retry ──▶ Submitting
  Queue: Open ── ‹ › / [ ] ──▶ Paging(k → k±1, 160 ms) ──▶ Open(page k±1);   answered page ──▶ next open page (or Closed)
  External:  run stopped ─▶ Stale ── 4 s ─▶ StaleRow     expires ─▶ Expired ("Timed out · denied") ── Ask again ─▶ (new request)
             superseded ─▶ Superseded ("Replaced by a newer request" → link)    answered elsewhere ─▶ Resolved(origin)
             rule added on another page matches ─▶ AutoResolved(row: "Allowed by your new rule")
  Overlays:  Hovered(button) · Pressed · Offline (allow disabled with reason) · Disabled(reason) per decision · Countdown (last 60 s)
```

Rules (proposed):
1. **No focus steal.** Entering never moves focus if the composer or another input has focus; the card
   is announced and reachable by `⌘/Ctrl+Shift+A` (status-strip shortcut), the status strip's Review, the
   permission chip's attention badge, or Tab from the transcript.
2. **Input guard** (400 ms after mount) and **focus scope**: single-key shortcuts act only when focus is
   inside the card or the app opts into a global approval scope (`ShortcutScope::Card` default).
3. **Enter maps to the safe default**: Low / Medium → Allow once; High / Critical → Deny.
4. **High arms for 600 ms** after the input guard; **Critical is press-and-hold for 1200 ms** (mouse,
   touch, Y or Space). Critical shows only Allow once + Deny + Deny with reason.
5. **Stale wins**: a run stop / cancel turns every open card of that run `Stale` immediately; actions
   disable before the visual change so a click in flight is rejected with "Run stopped".
6. The composer is part of the card: while a card is open, text typed in the composer and sent with Enter
   becomes **Deny with reason** for the head card (Codex "No, and tell Codex what to do differently",
   note 03), unless the user chooses "Send as a new message" from the send menu (queued, per
   [composer-queue-stack.md](composer-queue-stack.md): nothing is delivered over an open approval).

## 3. States table
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Entering` | footer rising from the card's bottom edge; host row shows ✋ amber and ⏸ timer | — | — | — |
| `Open(Low)` | ○ "Low risk" line, reason, buttons: Allow once · For this session · Always… · Deny | Allow once | session, always, deny, overflow | (after guard) Y S A N, R, E, D |
| `Open(Medium)` | ◇ "Medium risk" + reasons; same buttons | Allow once | as above | as above |
| `Open(High)` | ▲ "High risk", 2 px border, reasons; Allow buttons filling (arming); Always… disabled "Not available for high-risk actions" unless app allows | **Deny** (default) | Allow once, session | N/Esc/Enter deny; Y/S after armed |
| `Arming` | Allow buttons fill left→right over 600 ms, `aria-disabled` | — | deny | early Y → shake + "Available in a moment" |
| `Armed` | Allow buttons enabled | Deny (default) | Allow once (Y) | Y, S, N |
| `Open(Critical)` | ⬣ "Critical" + hatch header, reasons, amount / target in large text; "Hold to allow once" button with ring; Deny; Deny and tell… | **Deny** | hold to allow | hold Y / Space 1.2 s; N / Esc / Enter deny |
| `Holding(p)` | ring / bar fill = p; label "Keep holding…"; percentage not shown | — | release cancels | release Y / Space cancels |
| `Draining` | fill drains 200 ms; label back to "Hold to allow once" | — | — | — |
| `Focused` | card focus ring 2 px; key hints visible on buttons (hidden on touch) | per tier | — | Tab cycles buttons; ←/→ moves between buttons; ‹ › pages |
| `PatternEditing` | "Always" expands: pattern field with the narrowest suggestion highlighted (`cargo test *`), Narrower / Broader steppers, scope radio (This project · All projects), "Would have matched 4 calls this session", broaden-warning for Medium | Allow and add rule | Back | ↑/↓ narrower/broader, Tab scope, Enter confirm, Esc back |
| `EditingArgs` | command / args become an editable field (mono); diff of edits under it ("you changed 1 flag") | Allow edited | Back | Enter confirm (⌘Enter for multi-line), Esc back |
| `ReasonEditing` | one-line field "Tell the agent what to do instead…", 500-char limit, "Sent to the agent" hint | Deny and send | Deny without reason | Enter send, Esc back |
| `Submitting` | pressed button shows a 2 px underline progress after 300 ms; other buttons disabled | — | — | — |
| `SubmitFailed` | "Couldn't reach the agent" + Retry; the gated call stays blocked | Retry | Deny locally (if adapter enforces) | Enter retry |
| `Resolving` | footer collapses into the resolved row | — | — | focus moves (§8) |
| `Resolved` | one line: "✓ Allowed once · you · 0:07" / "✓ Allowed for this session" / "✓ Always allowed `cargo test *` · project · Edit rule" / "⊘ Denied · 'use --dry-run first'" | — | Edit rule / Undo grant (session) | Enter on row → audit entry |
| `Paging` | "1 of 3" with ‹ ›; page body crossfades + slides 12 px | — | — | ←/→ when pager focused, `[` `]` anywhere in card |
| `Batch` (≥ 2 compatible ≤ Medium) | extra row "Allow all 3 once · Deny all" | per page | batch | ⇧Y all once, ⇧N deny all |
| `Countdown` | "Expires in 0:42" right-aligned, digits swap 1 Hz | — | — | — |
| `Expired` | "Timed out · denied after 10:00" + Ask again | Ask again | — | Enter |
| `Stale` | card at 60 % saturation, buttons replaced by "Run stopped · no longer needed"; collapses to one line after 4 s | — | dismiss | — |
| `Superseded` | "Replaced by a newer request" + link to it | jump | — | Enter jump |
| `AutoResolved` | resolved row with provenance ("Allowed by your rule `npm test *`") | — | Edit rule | — |
| `Offline` | Allow buttons disabled "Offline"; Deny enabled only if enforceable locally | — | — | — |

## 4. Micro-interactions
- **Rise.** Footer slides up from the host card's bottom edge: height 0 → auto (200 ms `ease.out`),
  content fades in 160 ms with 40 ms delay. The host row's loader morphs to the ✋ glyph and its timer
  freezes with ⏸ (tool-call.md). Inline cards enter with `dur.row.enter` (height spring + fade, y +4 → 0).
- **Argument preview.** Mono block, syntax-tinted; dangerous tokens flagged by the app classifier get a
  dotted underline in the tier tone plus a superscript glyph (shape, not only colour); the first 4 lines
  show, "Show all N lines" expands with `dur.expand`.
- **Diff preview link.** For `Edit` requests the footer shows "2 files · +12 −3 · Open diff ↗ D"; the
  first hunk is inline above (family F file-edit chip); `D` opens the family H diff viewer, focus returns
  to the footer on close.
- **Buttons.** Hover bg 80 ms; press scale 0.97 (`spring 600 / 30`); key-hint caps (`Y`) sit inside each
  button at 70 % opacity and brighten on hover; on touch the hints are not drawn.
- **Arming (High).** A fill sweeps left→right across Allow once / For this session in 600 ms; an early
  press shakes the button 2 × 3 px and shows "Available in a moment" for 1.2 s. The fill completes into
  the normal button background (no flash).
- **Hold (Critical).** A 2 px ring around the button's leading glyph (card) or a bar under the label
  (footer) fills linearly over 1200 ms; at 50 % a single haptic tick (touch); at 100 % the button pops
  1 → 1.04 → 1 and turns into "Allowed" before the collapse. Releasing early drains in 200 ms. Moving the
  pointer off the button while holding cancels (prevents drag-through).
- **Queue pager "1 of 3".** Numerator rolls (`dur.count.roll`); the page slides 12 px from the
  inline-end (forward) or inline-start (back) with a 160 ms fade; the card's height animates with
  `spring.snap` only if the next page differs by more than 8 px, otherwise it jumps (less motion).
- **Always… → pattern editor.** The button widens into the editor (240 ms `spring.snap`); Narrower /
  Broader moves a highlight over the command tokens (160 ms) and the match count rolls; broadening past
  the first argument tints the field border with the Medium tone and adds "Matches commands you haven't
  seen yet".
- **Resolve.** Footer collapses (220 ms `spring.snap`) into a one-line note with the decision glyph; ✋
  morphs to ✓ (stroke 240 ms) or ⊘; the host row continues: Approved flash 400 ms → Running, or `Denied`
  (tools.md). The resolved note stays for the life of the row (it is the audit's face).
- **Deny with reason.** Field swaps in place; on Enter the typed text lifts 8 px and fades into the
  resolved row quote (220 ms). From the composer: the composer text flies to the card (the composer's
  `dur.travel` 220–260 ms) and the card resolves.
- **Stale.** Desaturate 160 ms, buttons crossfade to the stale line, hold 4 s, collapse 180 ms.
- **Attention link.** When the card is reached by ⌘⇧A or the selector badge, its border flashes once
  (400 ms, the selector's "card flash on jump").

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Footer rise (height / content fade) | 200 ms / 160 ms | 0 / 40 ms | `ease.out` | — | 60 Hz |
| Inline card enter | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 (bursts) | 60 Hz |
| Input guard | 400 ms | from mount | — | — | — |
| Arming fill (High) | 600 ms | after guard | linear | — | 60 Hz |
| Early-press shake | 240 ms, 2 × 3 px | 0 | linear | — | 60 Hz |
| Hold fill / drain (Critical) | 1200 ms / 200 ms | 0 | linear / `ease.in` | — | 60 Hz |
| Hold complete pop | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Button hover / press | 80 ms / ~120 ms | 0 | `ease.out` / spring 600 / 30 | — | 60 Hz |
| Page slide + fade | 160 ms, 12 px | 0 | `ease.out` | — | 60 Hz |
| Pager count roll | 280 ms | 0 | `ease.out` | — | 30 Hz |
| Pattern editor open | 240 ms | 0 | `spring.snap` | — | 60 Hz |
| Pattern highlight move / count roll | 160 ms / 280 ms | 0 | `ease.out` | — | 60 Hz |
| Reason field swap | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Submitting underline | indeterminate, 1200 ms period | 300 ms | linear | — | 30 Hz |
| Resolve collapse | 220 ms | 0 | `spring.snap` | — | 60 Hz |
| ✋ → ✓ / ⊘ | 160 ms + 240 ms stroke | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| Card flash on jump | 400 ms | scroll end | `ease.out` | — | 60 Hz |
| Expiry countdown | 1 s steps | last 60 s | digit swap | — | 1 Hz |
| Stale desaturate / hold / collapse | 160 / 4000 / 180 ms | 0 | `ease.out` / — / `ease.in` | — | 30 Hz |
| **Reduced motion** | footer and pages appear with 80 ms opacity, heights jump; arming = static text "Available in 0.6 s" then enables; hold keeps 1200 ms with a 4-step fill at 4 Hz and the text "Hold to allow · 1.2 s"; no shake (text only), no pop, no stroke draw, no fly-in of reason text; pager digits swap; jump flash becomes a static 2 px border for 1 s | — | linear | none | 1 Hz idle |

## 6. Variants

### Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `approval-card/footer` | Command-approval footer inside the tool-call card: risk line, quoted reason, Allow once · Session · Always… · Deny with key hints, overflow, pager | chat, desktop, ide |
| `approval-card/inline` | Stand-alone transcript card with title (verb + target), argument table, diff link, risk badge, same decision row; resolves into a one-line row | chat, desktop, ide, mobile |
| `approval-card/terminal` | TUI: `Would you like to run …?` with numbered options `1 Yes (y) · 2 Yes, don't ask again for "cargo test" (p) · 3 No, and tell the agent… (esc)` (Codex pattern, note 03), risk glyph `▲`, hold shown as `[hold y ▓▓▓░]` | terminal, ide |
| `approval-card/modal` | Centred sheet for Critical / Spend / Credential when the app wants a hard stop; `alertdialog`, focus on Deny | desktop, mobile |
| `approval-card/paged` | Queue of open requests as one card with "1 of 3", ‹ ›, per-page decisions and batch row (Beautiful UI paging idea [R106]) | chat, desktop, mobile |

### Pro (14)
| Name | One-line idea |
|---|---|
| `approval-card/replace-input` | Card takes the composer's place until answered, then the composer returns (Warp CLI idea, note 03) |
| `approval-card/split-preview` | Left: command or diff preview at full height; right: risk, reason and decisions |
| `approval-card/swipe-stack` | Mobile card stack; tap-only decisions with the next card peeking 8 px behind (no swipe-to-approve) |
| `approval-card/hold-ring` | Every Allow is a hold ring sized by tier (Low 300 ms … Critical 1200 ms) |
| `approval-card/amount` | Spend layout: big tabular amount, payee, budget bar after the spend, "Lower amount" edit |
| `approval-card/secret` | Credential layout: masked key card, destination host, scope chips, "Value never shown to the agent" ([R049] idea) |
| `approval-card/site-grant` | Network / browser: favicon, host incl. subdomains toggle, Allow once / Always for this site / Deny (Claude browser idea, note 03) |
| `approval-card/capability` | Computer-use grant with View-only / Click-only / Full control tier chooser (note 03 tier badge idea) |
| `approval-card/time-grant` | "Allow for 1 hour" option and a resolved row with a time-left ring and Revoke (affectUI Permission Request idea, note 11) |
| `approval-card/reviewer` | Classifier verdict above the decisions ("Reviewer: likely safe · reads only") with Override (Codex auto-review idea, note 03) |
| `approval-card/edit-first` | Opens directly in EditingArgs; Allow sends the edited command (LangGraph "edit" response idea, note 05) |
| `approval-card/inbox` | Cross-agent approval inbox: columns Needs you / Answered with agent swatches (agent inbox idea [R062]) |
| `approval-card/notch` | Desktop island shows "✋ 1 approval · Y / N" while the app is unfocused; expands to the footer |
| `approval-card/voice` | Voice surface: spoken summary, "say allow once or deny"; Critical requires the confirm phrase plus a tap |

## 7. Edge cases
- **Long commands / heredocs / 40-arg MCP calls**: 4 lines + "Show all"; arg tables cap at 6 rows +
  "+12 more"; classifier-flagged tokens hidden below the fold are summarised in the risk line.
- **Long reasons**: quoted reason clamps at 3 lines with "More"; reasons are never truncated in the
  accessible name.
- **Many queued (≥ 10)**: pager shows "1 of 14"; a "Review list" opens the paged queue as a list; batch
  actions only for compatible ≤ Medium groups.
- **Two approvals for the same call** (args changed while open): the old one becomes `Superseded`.
- **User answers the same request on two devices**: first ack wins; the other shows "Answered on your
  phone".
- **Card scrolled off-screen**: the status strip badge and the permission chip badge carry it; ⌘⇧A scrolls
  it into view without moving the user's reading position more than needed (card top 24 px below the
  viewport top).
- **Run cancelled during Holding**: the hold is aborted, card becomes `Stale`; no approval is sent.
- **Rule just added matches the remaining pages**: those pages auto-resolve, pager count rolls down; if all
  resolve, the queue closes with "3 allowed by your rule".
- **RTL**: layout mirrors; command, paths, diff and pattern stay LTR in isolated spans; arming / hold fill
  from inline-start; pager arrows flip.
- **Tiny / dense (IDE 280 px panel)**: buttons collapse to `Y` `S` `A` `N` key caps with tooltips; risk line
  becomes glyph + label; reason in tooltip; footer min height 28 px.
- **Touch**: 44 px buttons in a 2 × 2 grid under 360 px width; hold uses haptics; no single-key shortcuts.
- **Slow network**: Submitting underline after 300 ms, "Still sending…" at 10 s, SubmitFailed with Retry;
  the call stays blocked.
- **Very fast agent**: if the adapter resolves the request (rule, classifier) within 150 ms, no card is
  drawn — only the provenance glyph on the row.
- **Very slow agent after approval**: the resolved row stays; the host row shows Running; nothing in the
  card animates afterwards.
- **No reason given by the agent**: the reason line reads "No reason given" in muted text (not hidden), so
  its absence is visible.

## 8. Accessibility
- Footer / inline card: `role="group"` with `aria-labelledby` (title "Approve: run cargo publish") and
  `aria-describedby` (risk line + reason). Modal variant: `alertdialog`, focus on Deny.
- Arrival announcement (assertive, once): "Approval needed, high risk: run cargo publish. Reason: release
  0.3.0. Press Control Shift A to review." Queue arrivals after the first are polite: "1 more approval
  waiting, 2 total".
- Buttons have full names: "Allow once, Y", "Allow for this session, S", "Always allow, opens rule editor,
  A", "Deny, N". Disabled ones keep focus and explain: "Always allow is not available for high-risk
  actions".
- Arming: `aria-disabled="true"` then polite "Allow is available". Hold: `aria-describedby` "Press and
  hold for 1.2 seconds"; progress exposed as `aria-valuenow` 0–100 on a `progressbar` linked to the button,
  announced at 50 % and on completion; alternative `TypePhrase` (type the target name) when the app or
  user enables it.
- Pager: "Approval 1 of 3" (`aria-live="polite"` on change); ‹ › named "Previous approval", "Next approval".
- Focus order inside: risk / reason (read) → primary → secondary decisions → overflow → pager. After a
  decision: next open page's card; when the queue empties, focus returns to the element that had focus
  before the jump (or the composer).
- Single-key shortcuts can be disabled (WCAG 2.1.4); then `Alt+Y` etc. (proposed). Contrast: text 4.5:1,
  borders / glyphs 3:1, tier carried by shape + label.
- Resolved row: `status`, reads "Allowed once by you after 7 seconds".

## 9. Props sketch
```rust
pub struct ApprovalCard<'a> {
    pub agent: &'a AgentState,              // AwaitingApproval { request } → head of the queue; Cancelled → Stale
    pub queue: &'a ApprovalQueue,           // approvals.md §9; pager + batch actions
    pub request: ApprovalId,                // the page shown (defaults to queue.head())
    pub host: CardHost,                     // ToolCallFooter(CallId) | Inline | Modal | Terminal
    pub variant: ApprovalCardVariant,       // Footer | Inline | Terminal | Modal | Paged | Pro(..)
    pub decisions: DecisionSet,             // from the request after tier rules; app may hide some
    pub enter_default: EnterDefault,        // ByTier (default): Low/Med → AllowOnce, High/Critical → Deny
    pub guard: Duration,                    // 400 ms
    pub arming: Duration,                   // 600 ms (High)
    pub hold: HoldConfig,                   // { duration: 1200 ms, alternative: Option<TypePhrase> } (Critical)
    pub shortcuts: ApprovalShortcuts,       // Y S A E N R D ⇧Y, pager [ ] ; scope: Card (default) | Global
    pub composer_denies: bool,              // true: composer Enter = DenyWithReason for the head card
    pub permission: Option<PermissionChipRef>, // mini read-only chip "Ask · this session"
    pub diff: Option<DiffLink>,             // Edit kind: files, +/− counts, opens family H viewer
    pub redactor: Option<Redactor>,
    pub labels: ApprovalLabels,             // i18n; per-app renames keep decision ids
    pub motion: MotionLevel,
    pub on_decide: Callback<(ApprovalId, Decision)>,
    pub on_open_diff: Callback<DiffRef>,
    pub on_jump: Callback<ApprovalId>,      // Superseded / audit links
}

impl ApprovalCard<'_> {
    pub fn presentation(&self) -> CardState {
        match self.agent {
            AgentState::Cancelled | AgentState::Error { .. } => CardState::Stale,
            AgentState::AwaitingApproval { request } if *request == self.request => CardState::Open(self.tier()),
            _ => self.queue.resolution(self.request).map(CardState::from).unwrap_or(CardState::Open(self.tier())),
        }
    }
}
```

## 10. Related components and references
**Related:** [approvals.md](approvals.md) (model, tiers, decisions, timeout, batching, audit) ·
[approval-questions.md](approval-questions.md) (Beautiful UI's question-style Approval Card, plan #25) ·
[decision-card.md](decision-card.md) ·
[tool-call.md](tool-call.md) (host card, Approved flash, `Denied` row) · [tools.md](tools.md) (lifecycle,
expand while awaiting approval) · [file-edit-chip.md](file-edit-chip.md) (first hunk) · family H diff
viewer · [composer-permission-selector.md](composer-permission-selector.md) (mini chip, attention badge,
card-driven raise) · [composer-status-strip.md](composer-status-strip.md) (`AwaitingYou`, ⌘⇧A) ·
[composer-input-shell.md](composer-input-shell.md) / [composer-send-control.md](composer-send-control.md)
(placeholder, ↑ answers the card) · [permission-rules.md](permission-rules.md) (pattern editor writes a
rule) · [plan-approval.md](plan-approval.md) · [subagent-tree.md](subagent-tree.md) (Review from a child).

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI approval prompt (note 03) | numbered options with keys, command-prefix "don't ask again", "tell Codex what to do differently" | MIT per licensing.md (note 08 reads Apache-2.0) — portable with attribution; re-check |
| Claude Code prompts (notes 03, 05) | "Yes, and don't ask again for `<pattern>`", "switch to auto mode", Esc = no | Proprietary — reference only |
| Warp approval card replacing the input; diffs expanded while pending (notes 03, 08) | `replace-input`, expand-while-pending | AGPL — study only |
| signoff-ui ApprovalCard [R107] | once / session / always, risk level, reason the agent reads | MIT — port with attribution after checking the repo |
| Beautiful UI Approval Card [R106] [R133] | paged "1 / 3", Skip / Continue | MIT — port with attribution |
| affectUI Approval Card / Permission Request (note 11 §17–18) | shortcuts, pausing countdown, resolved-row morph, 1-hour grant + Revoke | Paid — study only |
| Arc UI "Hold to confirm" gesture (note 12) | hold-to-confirm as a gesture family | Paid — study only; our hold timing is from the permission selector |
| Agent inbox board [R062]; Vespin "(recommended)" [R022]; secret card [R049] | inbox columns; recommended option; masked secret | Study only |
| hermes-agent controls [R088] | ↑ answers the pending card | Licence not recorded — study only |


## Implementation (v0.2, plan #21)
`oxyn-core::approval`: `Risk`, `Decision`, `ApprovalRequest` (title, description, tray label, details, command, amount, reasons, expiry), `ApprovalCard` (400 ms input guard, 600 ms arming for High, 1200 ms hold with 200 ms drain for Critical, Enter = safe default by tier, countdown paused while hovered or focused and warning in the last 10 s, spinner after 300 ms, resolved row and live-region text), `ApprovalQueue` (pager, batch for ≥ 2 requests ≤ Medium), `pattern_ladder` / `pattern_matches`. Dioxus `ApprovalCard` (inline with tray or in-card actions, footer, terminal, modal) and `ApprovalPager` (paged). GPUI `ApprovalCardEl` (inline). affectUI Approval Card parity: title, description, risk badge (gray / amber / red), details, tray label, always-allow checkbox, Edit, ⌘↵ / Esc, pausing countdown, morph to resolved row, `actions: tray | card`. Risk tokens now map low → neutral, medium → warning, high and critical → danger. Not built yet: PatternEditing, EditingArgs and ReasonEditing panes, Paging animation, Pro variants.
