**Status:** draft v0.1 (2026-11-11).

# Spec: "who's acting" indicator (`acting-indicator`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
overview in [tools.md](tools.md)). Same depth as [composer-send-control.md](composer-send-control.md).
Target **4 free + 12 Pro = 16 variants**. Reads the shared `RunGraph` from
[subagent-tree.md](subagent-tree.md) §9. The take-over / hand-back **banner and its approval semantics**
belong to the approvals family G (see [../component-taxonomy.md](../component-taxonomy.md)); this
component is the **always-on signal** that banner is attached to.

## 1. Purpose
Agents act on shared surfaces: an editor buffer, a terminal, a browser view, a spreadsheet, the user's
desktop in computer use. At any moment the user must be able to answer **"who is driving this right
now — me, the agent, one of its sub-agents, or a teammate?"** and **"how do I take the wheel, and how do
I give it back?"** Getting this wrong means typing into a terminal the agent is also typing into, or an
agent clicking where the user's hand is.

What we saw: Zed has a crosshair to **follow the agent** into each file; Devin has "Follow Devin" in its
Progress tab and an **IDE take-over**; Replit shows an **agent cursor** in its App Testing browser with
video replay; Claude computer use shows a capability tier badge View-only / Click-only / Full control
(all research note 03); terminal tabs are marked `agent-owned` vs `user-owned` with bot vs user glyphs
(research note 05 TerminalPanel recommendation); Hoplite shows multiplayer threads with teammates
alongside the agent and counts "You stepped in 0" on a long run (research note 10); a presence chip
appears in an X footer demo [R051]; affectUI lists Agent Cursors, Agent Handoff and Collaboration Stream
(names only, research note 11). The taxonomy lists the take-over / hand-back banner under family G and
take-over control under browser / computer-use agents.

Anatomy (`frame` variant on an editor or browser surface):
```
╭─ ◆ Agent is driving · Researcher ──────── Take over ⌃⌥T ─╮   actor tab (top-left of the surface)
│                                                          │   2 px edge in the actor's tone
│        ↖ Researcher                                      │   agent cursor flag (name + colour)
│                                ↖ Ines (viewing)          │   teammate presence cursor (hollow)
│                                                          │
╰──────────────────────────────────────────── 👁 2 viewing ╯   presence stack (bottom-right)
```

Actors: `User` (you), `Agent` (the root run), `SubAgent(id)`, `Teammate(person)`, `Automation` (scheduled
or rule-driven run), `Nobody`.

Rules (proposed):
1. **Exactly one controller per surface** at a time; everyone else is a viewer. The indicator always
   shows the controller, even when it is the user (subtle) — never an empty state that could mean either.
2. **Control changes are always visible and always explained**: who took it, from whom, why (user
   take-over, agent request approved, timeout, policy).
3. **Explicit take-over by default.** User input on an agent-controlled surface does not silently steal
   control; it raises a "Take over to interact" chip at the input point (`TakeoverPolicy::Explicit`).
   Apps may choose `OnInput` (first key/click takes over) or `Never` (view-only surfaces).
4. **Hand-back resumes at a safe point**: the agent re-reads the surface before acting (the app supplies
   the "resyncing" state); the indicator shows "Agent resuming…" until the app confirms.
5. **Cursor presence is labelled and identity-coloured**; the agent cursor is solid, teammate cursors are
   hollow when viewing and solid when they control.
6. Approval-gated hand-offs (agent asks for control, or asks the user to take over for a login or a
   payment) are rendered by family G; this component only shows the resulting state and the link.

## 2. State machine
Per surface:
```
                       agent starts acting                          user presses Take over
  UserControl ───────────────────────────────▶ AgentControl{actor} ───────────────────────────▶ TakingOver
  (subtle "You")   ◀── run ends / agent releases ──┘   │  ▲   │                               (agent pauses at next
        ▲                                              │  │   │ agent asks user to act          safe point, ≤ app timeout)
        │                                              │  │   ▼                                        │
        │                       sub-agent takes the    │  │  HandoffRequested{to: User, reason}  ──────┤ (family G banner)
        │                       surface                │  │   │ accept                                 ▼
        │                                              ▼  │   └──────────────────────────────────▶ UserControl{from_agent}
        │                              AgentControl{SubAgent} │                                        │ Hand back
        │                                                      │                                        ▼
        │                                                      └──────── resumed ◀──────────── Resuming (agent resyncs)
        │
        │  agent asks for control while user drives:
  UserControl ── ControlRequested{by: Agent, reason} (family G approval) ── allow ──▶ AgentControl
                                                                     └── deny ──▶ UserControl
  Teammate:  any ── teammate takes control (app permits) ──▶ TeammateControl{person} ── releases ──▶ previous
  Conflict:  AgentControl + user input with policy Explicit ──▶ InputBlocked (chip "Take over to interact", 3 s)
             two controllers reported by the app ──▶ Conflict (both shown, warning) ── app resolves ──▶ one controller
  Overlays:  Paused (run paused / awaiting approval: agent holds control but is idle)
             Stalled (controller silent > stall_soft)  ·  Offline (presence unknown, last known at 60 %)
  Nobody:    no run and no user focus on a shared surface (e.g. headless sandbox) → "Idle"
```

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `UserControl` | subtle "You" tab or nothing on private surfaces; no edge tint | — | start agent here (app action) | — |
| `UserControl{from_agent}` | "You have control · Hand back" tab in user tone; agent's cursor parked as a hollow flag at its last position | **Hand back** | keep control (dismiss hint) | proposed `⌃⌥T` / `⌘⌥T` toggles; Esc does **not** hand back |
| `AgentControl{Agent}` | "◆ Agent is driving" tab + 2 px edge in accent; agent cursor flag; verb ("Clicking Sign in") | **Take over** | follow / unfollow agent (scrolls the view with it), pause | `⌃⌥T` take over; `f` follow when the surface has focus |
| `AgentControl{SubAgent}` | tab names the sub-agent with its identity swatch ("Researcher is driving"); breadcrumb to parent on hover | take over | focus sub-agent (subagent-tree) | as above |
| `AgentControl{Automation}` | "⟳ Automation is driving · Nightly sync" | take over | open automation | as above |
| `TakingOver` | tab text "Taking over…" with a 2-segment progress; agent cursor fades | cancel | — | Esc cancel |
| `HandoffRequested{to: User}` | tab amber "Agent needs you to act: sign in" + link to the family G banner | **Take over** | decline (agent continues/aborts per app) | Enter take over |
| `ControlRequested{by: Agent}` | tab amber "Agent asks to take control" + link to the family G approval | **Allow** | Deny | Enter allow, Esc deny |
| `Resuming` | "Agent resuming…" with indeterminate edge shimmer (one pass) | take over again | — | `⌃⌥T` |
| `TeammateControl{person}` | "Ines is driving" with avatar; solid teammate cursor | request control (app) | follow | — |
| `InputBlocked` | chip at the pointer / caret: "Agent is driving · Take over to interact" (3 s, or until pointer leaves) | take over | dismiss | Enter take over, Esc dismiss |
| `Conflict` | both actor tabs, warning tone, "Two controllers — Ines and Agent" | take over (resolves to you) | — | — |
| `Paused` overlay | controller tab + ⏸ "waiting for approval" (agent idle but holding control) | review | take over | — |
| `Stalled` overlay | controller tab ramps to warning; "Agent idle for 45 s" | take over | retry (status strip) | — |
| `Offline` overlay | last known tab at 60 % + "presence unknown" tooltip | — | — | — |
| `Nobody` | "Idle" only on shared surfaces that show presence | — | — | — |
| Presence stack | up to 3 avatars + "+2"; "👁 2 viewing" | open presence list | follow a person | Enter opens list |

## 4. Micro-interactions
- **Agent takes control.** The edge draws around the surface from the actor tab's corner clockwise (one
  pass, 320 ms) and settles at 2 px; the tab slides out of the corner (x −8 → 0). No looping glow by
  default (the busy loader elsewhere already loops; one looping signal per region, container-loaders.md
  §4.3); `expressive` variants may breathe the edge at `loop.glow`.
- **Agent cursor.** Moves on eased paths, not teleports: each move tweens over
  `clamp(distance ÷ 1600 px/s, 80 ms, 400 ms)` with `ease.in-out` (proposed); clicks show a 24 px ripple
  (200 ms); typing shows a caret flag with the name. The cursor's label hides after 1.5 s of stillness
  and re-appears on the next move. Path trails are a Pro variant.
- **Take over.** Pressing Take over: tab text morphs to "Taking over…", the agent cursor shrinks and
  fades (160 ms), the edge recolours from accent to the user tone (200 ms OKLab), then the tab becomes
  "You have control · Hand back". If the app reports the agent paused immediately, the whole morph runs
  in 240 ms; otherwise it waits on the app with the 2-segment progress (timeout per app).
- **Hand back.** The user tone retracts to the tab, a single shimmer pass ("Resuming…") runs along the edge,
  then the agent tone returns when the app confirms resume.
- **Input blocked.** The chip pops at the pointer (scale 0.9 → 1, `spring.snap`) with a 1-cycle 2 px nudge
  so it reads as "not yet" without being an error.
- **Sub-agent hand-over.** When control moves from parent to a sub-agent, the tab's swatch and name
  crossfade (180 ms); the cursor flag's label rolls to the new name.
- **Teammate presence.** Avatars enter the stack with a 160 ms scale-in, 30 ms stagger; a viewing teammate's
  hollow cursor follows at 15 Hz (smoothed) to save bandwidth; leaving fades 240 ms.
- **Follow mode.** While following, the view scrolls to keep the agent cursor within the middle 60 %
  (`ease.out` 240 ms per scroll); any manual scroll pauses follow and shows "Follow paused · Resume".

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Edge draw on agent control | 320 ms one pass | — | `ease.out` along perimeter | — | 60 Hz |
| Actor tab slide in | 180 ms, x −8 → 0 | — | `ease.out` | — | 60 Hz |
| Agent cursor move | 80–400 ms (distance ÷ 1600 px/s) | — | `ease.in-out` | — | 60 Hz while moving |
| Click ripple | 200 ms, 0 → 24 px | — | `ease.out` | — | 60 Hz |
| Cursor label auto-hide | after 1.5 s still; 160 ms fade | 1.5 s | `ease.in` | — | 30 Hz |
| Take-over tone change | 200 ms | — | OKLab `ease.out` | — | 60 Hz |
| Take-over morph (immediate pause) | 240 ms total | — | `spring.snap` | — | 60 Hz |
| Hand-back shimmer pass | 600 ms × 1 | — | linear | — | 60 Hz |
| Input-blocked chip | 160 ms in, 2 px × 1 nudge 160 ms; hold 3 s; 120 ms out | — | `spring.snap` / linear / `ease.in` | — | 60 Hz |
| Sub-agent name roll | 180 ms | — | `ease.out` | — | 60 Hz |
| Presence avatar in / out | 160 ms / 240 ms | — | `spring.snap` / `ease.in` | 30 ms | 60 Hz |
| Teammate cursor updates | 15 Hz smoothed (critically damped, 120 ms) | — | `spring.snap` | — | 60 Hz render |
| Follow-mode scroll | 240 ms | — | `ease.out` | — | 60 Hz |
| Expressive edge breathe | `loop.glow` 3.2 s | — | `ease.in-out` | — | 30 Hz |
| **Reduced motion** | edge appears at full 2 px with an 80 ms fade (no draw); agent cursor jumps to its target with an 80 ms fade-in at the new spot plus a static 12 px target ring for 600 ms; no ripple (static ring); tone changes are instant; follow mode jumps instead of scrolling; presence avatars appear without scale | — | linear | none | event-driven |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `acting-indicator/chip` | A pill in the surface's header: actor glyph + name + Take over / Hand back | desktop, ide, chat, mobile |
| `acting-indicator/frame` | Actor tab in the corner + 2 px edge around the surface in the actor's tone (anatomy above) | desktop (computer use, browser view), ide |
| `acting-indicator/cursor-flag` | Labelled agent cursor and teammate presence cursors, no frame; for canvases and docs | desktop, ide, web apps |
| `acting-indicator/terminal-owner` | Terminal tab marker `🤖 agent` / `you` and a one-line footer "agent typing · ^⌥T take over" | terminal, ide |

### Pro (12)
| Name | One-line idea |
|---|---|
| `acting-indicator/steering-wheel` | A small wheel glyph that rotates to whoever holds it; tap to take |
| `acting-indicator/baton` | A baton chip that visibly passes between avatars on every control change |
| `acting-indicator/cursor-trail` | Agent cursor leaves a fading trail of its last 8 positions and click marks (computer-use review) |
| `acting-indicator/ghost-hand` | Agent pointer drawn as a translucent hand with press states for touch-like surfaces |
| `acting-indicator/tier-badge` | Shows the agent's capability tier (view-only / click-only / full control) beside the actor (Claude computer-use idea, note 03) |
| `acting-indicator/spotlight` | Dims everything except the region the agent is about to act on, 300 ms before the action |
| `acting-indicator/avatar-ring` | Avatar stack where the controller's avatar has a ring and the others are dimmed |
| `acting-indicator/split-cursor` | When user and agent work in the same doc on different regions, both carets and a soft region lock |
| `acting-indicator/timeline-strip` | A thin strip under the surface showing control segments over time (who drove when); click jumps to run replay |
| `acting-indicator/notch` | Desktop island shows "Agent is driving your screen · Take over" while the app is unfocused |
| `acting-indicator/countdown-handback` | After user take-over, an optional auto hand-back countdown (only if the app enables it) |
| `acting-indicator/ascii` | Terminal status line with a block-character edge and `[AGENT]`/`[YOU]` tag |

## 7. Edge cases
- **Agent and user both type** (policy `OnInput` race): the first event the app acknowledges wins; the
  other side's keystrokes are buffered and shown as "3 keystrokes not sent · Send / Discard" (app-supplied
  buffer). Never drop input silently.
- **Rapid control churn** (sub-agents handing back and forth): the tab debounces changes shorter than
  400 ms (shows the latest), but the timeline strip and run replay keep every segment.
- **Multiple surfaces.** Each surface has its own indicator; a window-level summary "Agent is driving 2
  surfaces" appears in the status bar when more than one is agent-controlled.
- **Off-screen agent cursor.** An edge arrow points toward it with the name; click scrolls there (follow).
- **Many teammates.** Max 3 avatars + "+n"; cursors for more than 5 viewers are hidden by default (list
  only).
- **Same name / colour.** Agent identity colours are the 8 agent tokens; teammates use their avatar and a
  hashed colour from a separate palette so an agent and a person never share a tone.
- **Long names.** Actor name truncates at 18 ch; full name + role in tooltip.
- **RTL.** Tab moves to the top-right; the edge draw runs counter-clockwise; cursor flags flip their label
  side.
- **Tiny sizes (16–20 px).** Glyph-only chip (◆ agent, ● you, avatar for teammate) with tooltip; Take over
  lives in the overflow menu and the shortcut.
- **Dense IDE panels.** No frame; 1 px top border in actor tone + chip in the panel header.
- **Touch.** No hover cursors; take over is a 44 px chip; agent "cursor" shows as tap ripples with labels.
- **Slow networks.** Presence older than 5 s fades to 60 % and says "last seen"; never show a stale
  "Agent is driving" as live after the run ended (run end clears control).
- **Accessibility tech driving the surface.** Screen-reader focus on an agent-controlled surface announces
  the controller once; the agent's focus changes never move the user's screen-reader focus.
- **Fast agents.** Control held for less than 300 ms (one click) does not draw the edge; the cursor ripple
  alone shows it, and the timeline strip records it.

## 8. Accessibility
- The actor tab is a `status` element with a label like "Agent Researcher is controlling this browser
  view. Take over, Control Option T". When the user controls: "You have control. Hand back to agent".
- **Live region:** control changes are **assertive** when they affect the user's input ("Agent took control
  of the terminal", "You have control"), polite otherwise (teammate started viewing). Cursor movements are
  never announced; agent actions can be announced as a polite log only if the app opts in (one per 2 s
  max).
- The agent cursor and presence cursors are `aria-hidden`; their information is in the tab and the
  presence list.
- **Contrast:** edge ≥ 3:1 against the surface background; tab text ≥ 4.5:1; actor carried by text and
  glyph, not tone alone; user vs agent tones differ in hue *and* the tab label.
- **Focus order:** actor tab (Take over / Hand back) is the first focusable element of the surface's
  chrome; the input-blocked chip receives focus only when invoked by keyboard input.
- **Keyboard:** proposed `⌃⌥T` (Windows/Linux) / `⌘⌥T` (macOS) toggles take over / hand back, configurable;
  `f` follow / unfollow when the surface has focus.
- **Reduced motion:** §5 last row; control changes stay readable from the tab text alone.

## 9. Props sketch
```rust
pub struct ActingIndicator {
    pub surface: SurfaceId,
    pub control: ControlState,                 // derived by the app/adapter, see below
    pub graph: Option<RunGraphRef>,            // to name sub-agents and colours (subagent-tree.md §9)
    pub agent_state: Option<AgentState>,       // controller's AgentState → Paused / Stalled overlays
    pub presence: Vec<Presence>,               // { actor, role: Viewing | Controlling, cursor: Option<Point>, last_seen }
    pub policy: TakeoverPolicy,                // Explicit (default) | OnInput | Never
    pub variant: ActingVariant,                // Chip | Frame | CursorFlag | TerminalOwner | Pro(..)
    pub capability: Option<CapabilityTier>,    // ViewOnly | ClickOnly | Full (tier-badge)
    pub follow: bool,
    pub on_take_over: Callback<SurfaceId>,
    pub on_hand_back: Callback<SurfaceId>,
    pub on_open_request: Callback<RequestRef>, // routes to the family G banner / approval
    pub debounce: Duration,                    // 400 ms
    pub labels: ActingLabels,
    pub motion: MotionLevel,
}

pub enum Actor { User, Agent(AgentId), SubAgent(AgentId), Teammate(PersonRef), Automation(AutomationRef), Nobody }

pub enum ControlState {
    Controlled { by: Actor, since: Instant, reason: ControlReason },   // TakeOver | Released | Approved | Policy | Timeout
    TakingOver { from: Actor },
    Resuming { agent: AgentId },
    HandoffRequested { to: Actor, request: RequestRef },
    ControlRequested { by: Actor, request: RequestRef },
    Conflict { actors: SmallVec<[Actor; 2]> },
}
```
Mapping: controller's `AgentState` `AwaitingApproval` → `Paused` overlay; `Stalled` → `Stalled`
overlay; `Done/Cancelled/Error` → control returns to `UserControl` (app confirms); `Steering` → one
steer blip on the tab. `ControlState` itself is app/adapter data (architecture.md's `AgentState` has no
surface-ownership concept; gap noted).

## 10. Related components and references
Related: take-over / hand-back banner, plan and step approvals (family G, in
[../component-taxonomy.md](../component-taxonomy.md)) · [subagent-tree.md](subagent-tree.md) (sub-agent
identity) · [terminal-output.md](terminal-output.md) (agent-owned vs user-owned terminals) ·
[run-replay.md](run-replay.md) (control segments on the timeline) · [trace-view.md](trace-view.md) ·
[container-loaders.md](container-loaders.md) (window edge glow; one looping signal per region) ·
[composer-status-strip.md](composer-status-strip.md) · live and voice screen-share overlay (family K).

| Ref | What we take | Licence note |
|---|---|---|
| Zed follow-agent crosshair (note 03) | follow the agent into each file | Mixed licence; study only |
| Devin "Follow Devin", IDE take-over (note 03) | follow mode; explicit take-over | Proprietary; study only |
| Replit agent cursor + video replay (note 03) | visible agent cursor on a live surface | Proprietary; study only |
| Claude computer use tier badge (note 03) | view-only / click-only / full control | Proprietary — reference only |
| TerminalPanel agent-owned vs user-owned (note 05 recommendation) | owner glyph on terminals | Our own recommendation |
| Hoplite multiplayer threads, "You stepped in" (note 10) | teammates + agent in one surface; intervention count | Proprietary; study only |
| Presence chip [R051] | presence chip styling idea | Study only |
| affectUI Agent Cursors, Agent Handoff, Collaboration Stream (note 11) | names only | Paid — study only |
