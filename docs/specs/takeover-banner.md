**Status:** draft v0.1 (2026-11-11).

# Spec: take-over / hand-back banner (`takeover-banner`)

Part of family G ([approvals.md](approvals.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **3 free + 9 Pro = 12 variants**.
Implements the banner and approval semantics that [acting-indicator.md](acting-indicator.md) hands to
family G (§1 rule 6, §2, §3 rows `HandoffRequested`, `ControlRequested`, `UserControl{from_agent}`,
`Resuming`, §9 `on_open_request`). The acting indicator is the **always-on signal** (actor tab, edge,
cursors); this banner is the **conversation about control**: the agent asking the user to act, the agent
asking for control, the user holding control while the agent is paused, and the hand-back with a note the
agent reads. Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
On a shared surface — a browser view, the user's desktop in computer use, a terminal, an editor — control
moves between agent and human. The user wants to:
1. **Take over now** (the agent is about to click the wrong thing) and know the agent has **actually
   stopped** before they touch anything.
2. **Do the part only they can do** when the agent asks (sign in, enter a one-time code, confirm a
   payment, pick a file) without the agent watching values it should not see.
3. **Hand back** with a short **context note** ("I signed in as the admin account", "I closed the cookie
   dialog") so the agent does not redo or undo their work, and see the agent **resume at a safe point**.
4. **Answer the agent's request for control** of a surface the user is driving (allow / deny), at a
   capability tier (view / click / full).

What we saw (note 03, acting-indicator.md §1): Devin's IDE take-over and "Follow Devin"; Replit's agent
cursor in its testing browser; Claude computer use's capability tier badge View-only / Click-only / Full
control and its "Allow for this session / Deny" grant; Hoplite counting "You stepped in 0" (note 10);
affectUI's Agent Handoff (name only, note 11); the taxonomy lists the take-over / hand-back banner under
family G and take-over control for browser / computer-use agents. No source we studied documents a
hand-back note; that part is our proposal.

Anatomy (`strip` variant over a browser surface, user in control):
```
╭──────────────────────────────────────────────────────────────────────────────────────────╮
│ ● You have control · Agent paused at step 4 "Open billing"   ⏸ 1:12   [ Hand back ⌃⌥T ] ⌄ │  banner (top of the surface)
╰──────────────────────────────────────────────────────────────────────────────────────────╯
 ┌ Hand back to agent ────────────────────────────────────────────────┐
 │ What should the agent know?  [ I signed in ✓ ] [ Changed nothing ] │  quick chips (in-house copy)
 │ ┌──────────────────────────────────────────────────────────────┐   │
 │ │ Logged in with the finance account; don't change the plan.   │   │  note (agent reads it)
 │ └──────────────────────────────────────────────────────────────┘   │
 │ You: 3 pages · typed in 2 fields (1 hidden from the agent)          │  activity summary (app-supplied)
 │                                   [ Keep control ]  [ Hand back ⏎ ] │
 └─────────────────────────────────────────────────────────────────────┘
```

## 2. State machine
Per surface; control states mirror acting-indicator.md §2, the banner adds the request, pause-confirm and
hand-back steps.

```
                         ┌────────────────── agent asks the user to act (login, code, payment, file) ─────────────────┐
                         │                                                                                           ▼
  AgentDriving ──Take over (banner / tab / ⌃⌥T)──▶ Pausing ──agent at safe point (ack)──▶ UserControl ◀── accept ── HandoffRequested{reason, sensitive}
  (banner hidden;          │                         │  ack timeout (app, default 5 s)        │   │                       │ decline
   tab only)               │                         ▼                                         │   │                       ▼
                           │                  PauseSlow ("Agent is finishing a step…")         │   │                 HandoffDeclined → agent continues / aborts (app)
                           │                     │ Stop now ─▶ ForceStopping ─▶ UserControl{forced}│
                           │                     └ ack ─▶ UserControl                           │   │ user idle ≥ idle_hint (Pro: auto hand-back countdown, opt-in)
                           │                                                                    │   ▼
                           │                                                        IdleHint ("Still working? Hand back when done")
                           │                                                                    │
                           │                                            Hand back ─▶ NoteComposing ── send / ⏎ ──▶ HandingBack(note)
                           │                                                │ Esc / Keep control            │ (⌃⌥T twice = hand back without note)
                           │                                                ▼                               ▼
                           │                                           UserControl              Resuming ("Agent re-reading the page…")
                           │                                                                               │ app confirms resync      │ fail
                           │                                                                               ▼                          ▼
                           └────────────────────────────────────────────────────────────── AgentDriving (toast "Resumed · used your note")   ResumeFailed (Retry / keep control)
  User driving, agent asks for control:
  UserDriving ── ControlRequested{capability, reason} ──▶ ControlAsk ── Allow (tier) ──▶ AgentDriving{capability}
                                                                    └─ Deny (+ reason) ──▶ UserDriving
  External:  run stopped / ended while UserControl ─▶ RunEnded ("Run ended · you keep control") ── dismiss ─▶ UserDriving
             teammate takes control (app) ─▶ TeammateControl (banner names them; take over only if app allows)
             two controllers ─▶ Conflict (warning; "Take over" resolves to you)        offline ─▶ Offline overlay (hand back disabled)
```

Rules (proposed):
1. **The user's input is never sent to the surface until the agent has paused** (`Pausing` acks), unless
   the user chooses **Stop now**, which stops the run's current action (the app decides how) and gives
   control immediately. The banner says which happened ("Agent paused" vs "Agent stopped").
2. **The agent is paused, not cancelled**: its run clock pauses (elapsed-timer.md), its `AgentState` reads
   `AwaitingApproval` with a `Control` request, and the acting indicator shows `UserControl{from_agent}`.
3. **Hand-back always offers a note**; the note is optional; the default focus is the note field, and
   `⏎` sends with whatever is typed (empty = no note). Pressing the take-over shortcut twice within
   600 ms hands back without opening the note (proposed).
4. **Sensitive hand-offs** (`sensitive = true`: login, code, payment) show "The agent can't see what you
   type in password and code fields" only if the app reports that it masks them; otherwise the banner
   says plainly "The agent can see this screen" — never imply privacy the app does not provide.
5. **The note and an activity summary are sent as context**, not as an instruction to undo; the summary is
   app-supplied, counts only (pages, fields, files), never field values.
6. **Control requests are approvals**: `ControlRequested` is `ApprovalKind::Control` with a capability
   tier; it obeys the decision set (once = until the agent releases; session; no "always" above Click-only
   by default) and is audited.

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `AgentDriving` | no banner (acting-indicator tab "◆ Agent is driving · Take over") | — | — | `⌃⌥T` / `⌘⌥T` take over (acting-indicator.md) |
| `Pausing` | banner slides in: "Taking over… waiting for the agent to pause" + 2-segment progress; surface input blocked | — | Stop now | Esc cancels take-over (agent continues) |
| `PauseSlow` | after 5 s (app timeout): "Agent is finishing a step: clicking Submit" + **Stop now** | Stop now | keep waiting | Enter stop now |
| `ForceStopping` | "Stopping…" ≤ 1 s | — | — | — |
| `UserControl` | "● You have control · Agent paused at step 4 'Open billing'" + paused timer + Hand back | **Hand back** | overflow: Stop run, Show agent's last step | `⌃⌥T` opens hand-back; Esc does **not** hand back |
| `UserControl{forced}` | as above but "Agent stopped mid-step · check the page before handing back" | Hand back | — | as above |
| `HandoffRequested` | amber banner: "✋ Agent needs you to sign in to billing.example" + reason quote + privacy line (rule 4) | **Take over** | Decline (+ reason) | Enter take over, N decline |
| `HandoffDeclined` | one line "You declined · agent will try another way" (app copy) | — | — | — |
| `ControlAsk` | amber banner: "Agent asks to control your browser" + capability chooser (View · Click · Full) + reason + risk glyph (Full desktop = High) | **Allow** (tier) | Deny, Deny and tell… | Y allow, N deny, R reason, ←/→ tier |
| `IdleHint` | after `idle_hint` (proposed 120 s of no input): banner line "Done? Hand back when you're ready" (no countdown by default) | Hand back | dismiss | — |
| `NoteComposing` | popover under the banner: quick chips, note field (500 chars), activity summary, Keep control / Hand back | **Hand back** | Keep control | Enter send (⇧Enter newline), Esc keep control, Tab chips |
| `HandingBack` | banner "Handing back…" | — | — | — |
| `Resuming` | banner "Agent re-reading the page…" with one shimmer pass; tab switches back on confirm | take over again | — | `⌃⌥T` |
| `Resumed` | toast 2.4 s "Resumed · read your note" (if a note was sent) | — | view note | — |
| `ResumeFailed` | "Agent couldn't resume · Retry / Keep control" | Retry | keep control | Enter retry |
| `RunEnded` | "Run ended · you keep control"; Hand back removed | dismiss | start new run (app) | Esc dismiss |
| `TeammateControl` | "Ines has control" + avatar; Take over only if app allows | request control (app) | — | — |
| `Conflict` | warning banner "Two controllers — you and the agent · Take over" | take over | — | Enter |
| `Offline` | hand back disabled "Offline · the agent can't resume yet"; user keeps control | — | — | — |

## 4. Micro-interactions
- **Take over.** The banner drops from the surface's top edge (height 0 → 36 px, 200 ms `ease.out`)
  while the acting-indicator tab morphs to "Taking over…" (acting-indicator.md §4); the 2-segment progress
  fills segment 1 on request, segment 2 on ack. On ack the edge recolours to the user tone (200 ms OKLab,
  owned by the acting indicator) and the banner text rolls to "You have control" (240 ms).
- **Input blocked during Pausing.** Clicks on the surface show the acting indicator's "Take over to
  interact" chip variant reading "Waiting for the agent to pause" (no nudge — the user did the right
  thing).
- **Paused timer.** The banner's ⏸ timer counts the user's time at 1 Hz (the agent's run clock is paused;
  this is the hand-off interval shown in the trace as a hatched segment, trace-view.md).
- **Handoff request.** Banner slides in amber with the hand glyph; one 600 ms attention pulse of the
  border; the acting-indicator tab links to it. No looping motion.
- **Hand back.** Pressing Hand back opens the note popover anchored to the button (200 ms `spring.snap`,
  scale 0.98 → 1); quick chips toggle with a 120 ms fill and insert their text at the caret; on send the
  note lifts 6 px and fades into the banner, which rolls to "Agent re-reading the page…".
- **Resume.** One shimmer pass along the surface edge (600 ms, acting-indicator.md); then the banner
  collapses (180 ms `ease.in`) and the toast appears. If resume fails, the banner stays and its text swaps.
- **Control ask.** Capability chooser is a 3-segment control; moving to Full adds the ▲ High glyph and a
  consequence line; Allow on Full desktop arms for 600 ms (approvals.md High rule).
- **Run ended.** Banner text crossfades (160 ms), Hand back button collapses (180 ms).

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Banner enter / exit | 200 ms / 180 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Pausing progress segments | 160 ms per segment fill | on request / on ack | `ease.out` | — | 60 Hz |
| Pause ack timeout → PauseSlow | 5 s (app-configurable) | — | — | — | — |
| Banner text roll | 240 ms | 0 | `ease.out` | — | 60 Hz |
| Edge recolour (acting indicator) | 200 ms | on ack | OKLab `ease.out` | — | 60 Hz |
| Paused / user timer | 1 s steps | — | digit swap | — | 1 Hz |
| Handoff attention pulse | 600 ms once | 0 | `ease-in-out` | — | 60 Hz |
| Note popover open / close | 200 / 140 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Quick chip toggle | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Note send lift | 220 ms, 6 px | 0 | `ease.in-out` | — | 60 Hz |
| Double-shortcut window | 600 ms | — | — | — | — |
| Resume shimmer pass | 600 ms × 1 | after send | linear | — | 60 Hz |
| Resumed toast | 160 in / 2400 hold / 160 out | on confirm | `ease.out` / `ease.in` | — | 60 Hz |
| Idle hint | after 120 s no input | — | 160 ms fade | — | — |
| Full-capability arming | 600 ms | after 400 ms guard | linear | — | 60 Hz |
| **Reduced motion** | banner appears / disappears with 80 ms opacity, height jumps; progress shows "1/2", "2/2" text; text swaps instead of rolling; no pulse (amber banner + hand glyph stay static); no shimmer pass (text "Resuming…" is enough); popover fades 80 ms; arming shows static "Available in 0.6 s" | — | linear | none | 1 Hz idle |

## 6. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `takeover-banner/strip` | Full-width strip at the top of the surface with state text, paused timer, Take over / Hand back, note popover | desktop, ide, chat (embedded browser) |
| `takeover-banner/card` | Transcript card for hand-off requests ("Agent needs you to sign in") with Take over / Decline, linking to the surface | chat, mobile |
| `takeover-banner/terminal` | One footer line: `■ you have control · agent paused · ^⌥T hand back (note: …)`; note entered inline after `^⌥T` | terminal, ide |

### Pro (9)
| Name | One-line idea |
|---|---|
| `takeover-banner/capability` | Control request with View-only / Click-only / Full chooser and per-app warnings (Claude computer use tier idea, note 03) |
| `takeover-banner/secure-input` | Sensitive hand-off shows which fields the app hides from the agent, as masked chips |
| `takeover-banner/activity-recap` | Hand-back popover lists the user's actions as a mini step list (counts and targets, no values) |
| `takeover-banner/countdown-handback` | Opt-in auto hand-back countdown after idle, pauses on any input (pairs with `acting-indicator/countdown-handback`) |
| `takeover-banner/notch` | Desktop island: "You have control · Hand back" while the agent app is unfocused |
| `takeover-banner/steer-note` | Note becomes a steering message with "Continue from here" / "Redo step 4" quick intents |
| `takeover-banner/split-view` | Side-by-side: user's live view and the agent's last screenshot with "agent paused here" marker |
| `takeover-banner/stepped-in` | Counter of interventions for the run ("You stepped in 2×") with links to each segment (Hoplite idea, note 10) |
| `takeover-banner/voice` | Voice surface: "Say 'take over' / 'hand back'"; note dictated |

## 7. Edge cases
- **Agent never acks the pause**: `PauseSlow` after 5 s; Stop now is always available; if the adapter
  cannot pause at all (`can_pause = false`), the Take over button reads "Stop and take over" from the
  start.
- **User takes over during an open approval card**: the card stays open but its decisions disable with
  "You have control · hand back to decide" (avoids approving an action on a page the user is changing).
- **User changes the page completely**: the activity summary says "navigated to a different page"; the
  resume step is the agent's re-read (rule 4 in acting-indicator.md).
- **Long notes**: 500-char cap with counter at 80 %; pasted text over the cap is truncated with a notice.
- **Multiple surfaces**: one banner per surface; the composer status strip shows "You control 2 surfaces";
  hand-back is per surface.
- **Sub-agent was driving**: banner names it ("Researcher paused"); hand-back returns control to the same
  sub-agent.
- **Run ends while the note is open**: popover closes with "Run ended · note not sent"; note text is kept
  as a composer draft (proposed).
- **Teammate presence**: teammates see "Ines has control" banner (read-only); only the controller
  sees Hand back.
- **RTL**: banner mirrors; timer and buttons move to inline-start side; URLs stay LTR.
- **Tiny sizes / narrow panels (< 320 px)**: banner becomes a 24 px bar with glyph + "You · Hand back";
  note opens as a sheet.
- **Touch**: 44 px buttons; banner stays above the on-screen keyboard; no shortcut hints.
- **Slow network / remote browser**: Pausing progress shows segment 1 until ack; Hand back shows
  "Sending…" after 300 ms; failure keeps control with the user (fail safe).
- **Very fast agents**: hand-off requests answered by the user in < 300 ms skip the pulse.
- **Very long user control (hours)**: timer switches to `1h 05m` format; idle hint repeats at most every
  15 min.

## 8. Accessibility
- Banner: `region` labelled "Control of browser view" containing a `status` line; state changes that
  affect the user's input are **assertive**: "You have control. Agent paused at step 4", "Agent asks to
  control your browser", "Agent needs you to sign in"; resume is polite: "Agent resumed".
- Pausing is announced ("Waiting for the agent to pause"), and the surface's input blocking is announced
  once when the user tries to type.
- Note popover: `dialog` labelled "Hand back to agent"; focus on the note field; chips are toggle
  `button`s with `aria-pressed`; the activity summary is plain text read after the field.
- Control ask: `group` with `radiogroup` for capability (each option names its reach); Full has "High risk"
  in its name.
- Keyboard: proposed `⌃⌥T` / `⌘⌥T` (shared with acting-indicator.md) toggles take over / opens hand-back;
  double press hands back without a note; Esc never hands back.
- Focus never jumps into the surface after take-over; the user moves there. After hand-back focus returns
  to where it was before the take-over (or the composer).
- Contrast: banner text 4.5:1, borders 3:1; actor and state carried by text and glyph (● you, ◆ agent,
  ✋ request), not tone alone.
- Privacy copy is part of the accessible description (rule 4), so screen-reader users hear the same
  guarantee or warning.

## 9. Props sketch
```rust
pub struct TakeoverBanner<'a> {
    pub surface: SurfaceId,
    pub control: &'a ControlState,          // from acting-indicator.md §9 (Controlled / TakingOver / Resuming / HandoffRequested / ControlRequested / Conflict)
    pub agent: &'a AgentState,              // AwaitingApproval { request } with ApprovalKind::Control → UserControl / ControlAsk;
                                            // Cancelled / Done / Error while user controls → RunEnded
    pub request: Option<&'a ApprovalRequest>, // HandoffRequested / ControlRequested payload (reason, capability, sensitive)
    pub paused_at: Option<StepRef>,         // "step 4 'Open billing'" from the plan / run log
    pub can_pause: bool,                    // false → "Stop and take over"
    pub pause_timeout: Duration,            // 5 s → PauseSlow
    pub privacy: PrivacyNotice,             // MaskedFields { count } | AgentCanSee | None (app must choose; no default claim)
    pub activity: Option<ActivitySummary>,  // counts only: pages, fields, files, hidden fields
    pub quick_notes: Vec<SharedString>,     // in-house copy, i18n
    pub note_limit: usize,                  // 500
    pub idle_hint: Option<Duration>,        // 120 s; None disables
    pub auto_handback: Option<Duration>,    // Pro, opt-in; None by default
    pub variant: TakeoverVariant,           // Strip | Card | Terminal | Pro(..)
    pub shortcuts: TakeoverShortcuts,       // toggle ⌃⌥T / ⌘⌥T, double-press window 600 ms
    pub labels: TakeoverLabels,
    pub motion: MotionLevel,
    pub on_take_over: Callback<TakeOver>,   // { surface, mode: Pause | StopNow }
    pub on_hand_back: Callback<HandBack>,   // { surface, note: Option<SharedString>, activity: Option<ActivitySummary> }
    pub on_answer: Callback<(ApprovalId, Decision)>, // HandoffRequested accept/decline, ControlRequested allow (tier)/deny
}

pub enum BannerState {
    Hidden, Pausing, PauseSlow, ForceStopping, UserControl { forced: bool }, HandoffRequested, HandoffDeclined,
    ControlAsk, IdleHint, NoteComposing, HandingBack, Resuming, Resumed, ResumeFailed, RunEnded,
    TeammateControl, Conflict, Offline,
}
```
The hand-back note reaches the agent as context on resume (adapter-specific: a user message, an AG-UI
custom event, or a tool result for the paused control request). `ControlState` remains app / adapter data
(acting-indicator.md notes the `AgentState` gap); this banner only reads it.

## 10. Related components and references
**Related:** [acting-indicator.md](acting-indicator.md) (actor tab, edge, cursors, shortcut) ·
[approvals.md](approvals.md) (`ApprovalKind::Control`, audit) · [approval-card.md](approval-card.md)
(cards disabled while the user controls) · [plan-todo.md](plan-todo.md) / [tool-call.md](tool-call.md)
("paused at step") · [elapsed-timer.md](elapsed-timer.md) (paused run clock) ·
[trace-view.md](trace-view.md) / [run-replay.md](run-replay.md) (control segments) ·
[composer-status-strip.md](composer-status-strip.md) (surfaces count) · family K screen-share overlay.

| Reference | What we take | Licence note |
|---|---|---|
| Devin IDE take-over, "Follow Devin" (note 03) | explicit take-over | Proprietary — study only |
| Claude computer use tier badge and session grant (note 03) | View-only / Click-only / Full, Allow for this session / Deny | Proprietary — reference only |
| Replit agent cursor in the testing browser (note 03) | agent-visible surface | Proprietary — study only |
| Hoplite "You stepped in" (note 10) | intervention counter | Proprietary — study only |
| affectUI Agent Handoff (note 11) | name only | Paid — study only |
| Hand-back note, activity summary, privacy notice | — | Oxyn proposal (no source documents them) |
