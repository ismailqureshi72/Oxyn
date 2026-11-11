**Status:** draft v0.1 (2026-11-11).

# Spec: composer (family overview)

Overview of the composer family (category A in
[../component-taxonomy.md](../component-taxonomy.md)). Each part has its own full spec at the depth of the
worked example [composer-send-control.md](composer-send-control.md); this file says **where every part
sits, how they share one state, and which parts each layout and category preset turns on**. Target
**6 free + 12 Pro composer-level variants** (layouts), plus the parts' own variant sets.

## 1. Purpose
The composer is where the user hands work to the agent and where they keep steering it while it runs.
In an agent app it is not a text box with a button: it is a **control surface** that carries model,
effort, mode, permissions, context, attachments, voice, a queue of follow-ups and the live status of
the run. The richest references pack most of these into one composer (Pitus [R021]: mic, ✓ confirm,
effort "Medium" with signal bars; Cloudy [R007]: steer-in-place, pause/stop, budget; BoardUI [R036]:
attachment ring, "Bypass all"; affectUI [R044]: "Request approval" chip, model pill, folder tray;
Devin concept [R033]: split send; T3 Code [R003]: "Full access / Supervised" and branch under the
composer). coss ui's Composer API (MIT) [R126] is our reference for the draft / trigger / submit model
underneath (research note 17).

Family rules (apply to every part):
1. **The composer never changes position** [R079]. Parts grow it **upward**; layers (queue stack,
   status strip, banners) stack upward above it.
2. **One state.** Every part reads the same `ComposerState`, derived from `AgentState`; no part keeps a
   private copy of agent state or its own timer (shared motion clock, [../architecture.md](../architecture.md)).
3. **One busy policy.** What Enter does while the agent works is decided once (`busy_action`) and every
   part (send, voice, queue, slash commands) obeys it.
4. **Mid-run changes apply to the next turn** unless a part's spec says otherwise (model, effort, mode,
   permission show "applies next turn" rather than silently changing a live run).
5. **Every part has a reduced-motion fallback and a text label per state.**

## 2. Anatomy

```
                          ┌───────────────────────────────────────────────────────────────┐
 banner layer             │ ⚠ Offline · retrying in 4s                         Retry now  │  status-strip (banner)
                          ├───────────────────────────────────────────────────────────────┤
 status strip             │ ● Editing auth.rs · 0:42 ■ │ ⚠ Needs approval │ ⏱ 6m/30m │ #128 · 3 conflicts · +482 −37 │
                          ├───────────────────────────────────────────────────────────────┤
 queue stack              │ Queued 2                                            ▾  Clear  │  queue-stack
 (grows upward)           │  ⤷ also check the refresh-token path        Steer · ✕         │
                          │  ✉ then write the migration          [Up next]  Send now · ✕  │
                          ╞═══════════════════════════════════════════════════════════════╡
 input shell ─────────────│ ┌──────┐┌──────┐┌──────┐                                      │  attachments (tiles, ring progress)
                          │ │ img ◔││ pdf ✓││ csv ✕│                                      │
                          │ └──────┘└──────┘└──────┘                                      │
                          │ Ask anything, @ to mention, / for commands▌                   │  input shell + slash/mentions popover
                          │   ┌ /review   Review the open diff ┐                           │  (popover opens upward)
                          │                                                               │
 footer row ──────────────│ [+] [Agent ▾] [🔒 Ask first ▾] [Opus 5 ▾] [▮▮▯ Med] (◔ 64%)  🎙 ≋ [↑] │
                          │  │      │           │               │          │       │     │  │  │
                          └──┼──────┼───────────┼───────────────┼──────────┼───────┼─────┼──┼──┼──┘
                             │      │           │               │          │       │     │  │  └ send control (send/stop/queue/steer)
                             │      │           │               │          │       │     │  └ voice-mode entry
                             │      │           │               │          │       │     └ voice input (mic / PTT / dictation strip)
                             │      │           │               │          │       └ context meter
                             │      │           │               │          └ effort control
                             │      │           │               └ model picker
                             │      │           └ permission selector
                             │      └ mode switch (Ask / Plan / Agent)
                             └ tools menu (+ → ×)
```

Placement rules (proposed):
- **Leading → trailing in the footer:** create (tools) → *how* the agent behaves (mode, permission) →
  *which brain* (model, effort) → *budget* (context meter) → *input method* (voice) → *commit* (send).
  Send is always the trailing-most control; voice is always immediately before it.
- **Above the text:** attachments, then the text. Popovers (slash, mentions, model, effort, tools)
  open **upward** and never cover the send control.
- **Above the shell:** queue stack, then status strip, then banner — closest to the composer is what
  the user acts on most (their queue).
- When the footer overflows, controls collapse into the tools menu in this order: context meter (→
  inline % in tools menu), effort (→ model picker submenu), permission, mode. Model, voice and send
  never collapse.

## 3. Shared state: `ComposerState`

All parts render from one derived value. `oxyn-core` builds it each time any input changes:

```
 AgentState ─┐
 Draft ──────┤                         ┌─▶ input shell   (placeholder, disabled reason, busy hint)
 Queue ──────┤                         ├─▶ send control  (Empty/Ready/Working/WorkingWithDraft/…)
 Voice ──────┼─▶ ComposerState::derive ├─▶ voice input   (Idle/Capturing/…; confirm routing)
 Pending ────┤   (pure fn, no timers)  ├─▶ queue stack   (Hidden/Open/Draining/Paused)
 Env ────────┘                         ├─▶ status strip  (Working/Stalled/AwaitingYou/…)
 (offline, quota, surface)             └─▶ pickers       (model / effort / mode / permission: "applies next turn")
```

**Composer-level phases** (what the whole composer is doing; each part maps it to its own states):
```
             type                 Enter                first event
  Empty ───────────▶ Composing ─────────▶ Submitted ─────────────▶ Busy ──── type ───▶ BusyWithDraft
    ▲                    │                                         │  ▲                  │ Enter: busy_action
    │◀── clear ──────────┘                                         │  └── queue/steer ───┘
    │                                       approval / question    ▼
    │                                     AwaitingYou ── answered ──▶ Busy
    │◀── Done / Cancelled / Error (queue policy decides what happens next) ───┘
  Overlays: Dictating · Disabled(reason) · Offline · Editing(queued item)
```

| Phase | Input shell | Send control | Voice | Queue stack | Status strip | Pickers |
|---|---|---|---|---|---|---|
| `Empty` | placeholder | `Empty` (muted ↑) | mic idle | hidden / paused | hidden | free switch |
| `Composing` | text, triggers live | `Ready` | mic idle | as is | hidden | free switch |
| `Submitted` | clears; draft → bubble | `Sending` (■) | mic idle | as is | `Submitted` pulse | locked for this turn |
| `Busy` | placeholder "Steer or queue…" | `Working` (■) | mic idle | as is | `Working` / `Stalled` | "applies next turn" |
| `BusyWithDraft` | text | `WorkingWithDraft` (Queue pill / steer) | mic idle | receives chips | `Working` | "applies next turn" |
| `AwaitingYou` | placeholder "Answer or queue…" | `Answering(card)` | confirm answers card | holds | `AwaitingYou` (timer paused) | locked |
| `Dictating` | strip replaces text row | `Dictating` (✓) | `Listening`/`Capturing` | as is | unchanged | unchanged |
| `Editing(item)` | queued text loaded | "Save" | available | ghost chip | unchanged | per-item overrides (Pro) |
| `Disabled(reason)` | read-only + reason | `Disabled(reason)` | disabled if server STT | held | banner explains | unchanged |

```rust
pub struct ComposerState {
    pub agent: AgentState,                 // the single source of truth (architecture.md)
    pub phase: ComposerPhase,              // derived, table above
    pub draft: DraftSummary,               // { is_empty, chars, attachments, uploading, ime_composing }
    pub queue: QueueSnapshot,              // { items, up_next, paused: Option<PauseReason> }
    pub voice: VoiceSnapshot,              // { state, level }
    pub pending: Option<PendingCard>,      // approval / question awaiting the user
    pub disabled: Option<DisabledReason>,  // Offline | Quota{resets_at} | Auth | Policy(String) | TooLong
    pub busy_action: BusyAction,           // Queue (default) | Steer | Interrupt | Ask
    pub next_turn: TurnSettings,           // model, effort, mode, permission chosen for the next send
    pub surface: Surface,                  // Chat | Ide | Desktop | Mobile | Voice | Terminal
}

impl ComposerState {
    pub fn derive(agent: &AgentState, draft: &Draft, queue: &Queue,
                  voice: &VoiceSession, env: &Env) -> Self { /* pure, no I/O */ }
}
```
Parts written in parallel take `agent: AgentState` (or `&AgentState`) in their own props; that is the
same value as `ComposerState.agent`. A host app may pass either the whole `ComposerState` or each
part's slice.

## 4. Choreography across parts
- **Send:** draft → bubble fly-up (≤ 260 ms, ease-in) [R079]; status tab rises (160 ms); send ↑ → ■
  instantly. Attachments ride with the bubble.
- **Queue:** draft → chip unfolds upward (200 ms) [R079]; status strip and banner shift up with it as
  one block (`spring.snap`), never independently.
- **Stop:** status tab text drops into the transcript as "Interrupted" (220 ms, firm, no overshoot)
  [R079]; queue goes `Paused("Stopped")`; ■ → ↑.
- **Dictation:** text row ↔ strip in place; send ↑ ↔ ✓ (160 ms spring) [R021].
- **Popovers:** only one open at a time; opening one closes another with no exit animation (instant
  swap feels faster than two overlapping animations — proposed).
- **Height changes** of any layer animate the composer's *top* edge only, 160–200 ms; the bottom edge
  and the send control never move.

### Motion tokens
Shared by all composer parts (proposed values; tokens live in `oxyn-core` motion presets):

| Token | Value | Used for |
|---|---|---|
| `dur.instant` | 0–80 ms | reduced-motion crossfades, press-down |
| `dur.fast` | 120 ms | hover, colour, small fades |
| `dur.base` | 160–200 ms | morphs, chip enter, popovers |
| `dur.travel` | 220–260 ms | text flying between composer and transcript (cap 260 ms) |
| `ease.out` | `cubic-bezier(0.22, 1, 0.36, 1)` | enters (same curve as the transitions.dev token noted in research note 15; curve values only) |
| `ease.in` | `cubic-bezier(0.4, 0, 1, 1)` | exits, fly-up |
| `ease.in-out` | `cubic-bezier(0.65, 0, 0.35, 1)` | travel between two resting places |
| `ease.firm` | `cubic-bezier(0.2, 0, 0, 1)` | "Interrupted" drop, no overshoot |
| `spring.snap` | stiffness 500, damping 45, mass 1 (ζ ≈ 1.0, critically damped) | glyph morphs, reflow |
| `spring.soft` | stiffness 260, damping 26, mass 1 (ζ ≈ 0.8, small overshoot) | expressive fans, voice-mode morph |
| `stagger` | 30–40 ms, cap 4–6 items | lists of chips/words |
| **Reduced motion** | every token above resolves to a 0–80 ms opacity crossfade; loops stop; information-carrying meters (audio level, progress) stay as static bars | — |

## 5. Composer-level layouts (variants)

### Free (6)
| Name | Layout | Notes |
|---|---|---|
| `composer/chat` | centred card, max-width 760 px, 2–12 rows, footer row of pills | consumer and general chat |
| `composer/ide-panel` | docked to a side panel 280–480 px wide; 28 px rows; footer pills become icons with tooltips; status strip `inline-row` | coding agents in an editor |
| `composer/desktop` | wide floating glass composer with footer tray (repo / branch / env pills under it, as in T3 Code [R003] and affectUI's folder tray [R044]) | standalone agent apps |
| `composer/mobile` | bottom-docked, safe-area aware; footer collapses to `+`, model pill and send; voice gets the largest target; queue as a sheet | phone apps |
| `composer/terminal` | single-line prompt `› ▌`, glyph send (`⏎ / ■ / ⇥ queue`), `↳` queue rows and `● Working (12s · esc to interrupt)` status (Codex-style, research note 08) | TUI surface |
| `composer/node` | the composer is a node on a canvas; context sources (repo, files, skills, model) are wired-in nodes and output agents branch out [R066] | workflow / builder apps |

### Pro (12)
`composer/notch` (desktop island that expands into a composer), `composer/spotlight` (⌘K-style
floating launcher composer that escalates to a full thread), `composer/split-pane` (prompt left, live
plan right), `composer/canvas-dock` (docks to the selected canvas object in design / video tools),
`composer/inline-edit` (composer opens at the cursor inside a document or code), `composer/sidecar`
(browser-sidecar composer that reads the current page), `composer/multi-agent` (one composer, a target
chip per agent lane [R061]), `composer/glass-hero` (empty-state hero composer that docks to the bottom
after the first send), `composer/voice-first` (orb above a slim text fallback), `composer/review-bar`
(composer merged with an Approve / Request changes bar for PR review), `composer/ticket` (support-desk
composer with customer context and macros), `composer/timeline` (video editor: composer bound to the
playhead range).

## 6. Category presets
A preset is a set of enabled parts + default variants + copy. ✓ = on by default, ○ = available, off
by default, — = not offered.

| Part | Coding | App builder | CRM | Finance | Video | Support | Voice |
|---|---|---|---|---|---|---|---|
| Input shell | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ (fallback) |
| Tools menu (+) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ |
| Attachments | ✓ files, images | ✓ images, designs | ○ | ✓ statements, CSV | ✓ clips, images | ✓ screenshots | — |
| Slash / mentions | ✓ `/` commands, `@` files | ✓ `@` pages/components | ✓ `@` contacts, deals | ○ `@` accounts | ○ `@` clips | ✓ `/` macros, `@` customer | — |
| Model picker | ✓ | ✓ | ○ | ○ | ✓ | — | ○ |
| Effort control | ✓ | ○ | — | ○ | — | — | — |
| Context meter | ✓ | ✓ | ○ | ○ | — | — | — |
| Mode switch | ✓ Ask / Plan / Agent | ✓ Plan / Build | ○ Draft / Send | ○ Analyse / Act | ○ Edit / Generate | ○ Reply / Note | — |
| Permission selector | ✓ | ✓ | ✓ (send emails) | ✓ (move money: always ask) | ○ | ✓ (refunds) | ○ |
| Voice input | ○ | ○ | ✓ (call notes) | ○ | ○ | ○ | ✓ dictation + voice mode |
| Send control | ✓ `split` / `queue-pill` | ✓ `queue-pill` | ✓ `classic` | ✓ `answer-card` | ✓ `ring-progress` | ✓ `classic` | ✓ `voice-confirm` |
| Queue stack | ✓ `chips` + steer | ✓ `lanes` | ○ | ○ | ✓ `lanes` (render jobs) | ○ | ○ |
| Status strip | ✓ `detailed` + PR pill | ✓ job pill + preview | ○ `tab` | ✓ `tab` + approval badge | ✓ job pill (render %) | ○ `tab` | ✓ `tab` |

Preset notes (proposed):
- **Finance:** permission selector never offers a "bypass" level for money movement; approval badge is
  always on; budget chip shows cost, not tokens.
- **Video:** job pill is the main status ("Image to video · Rendering 4%" pattern [R013]); queue lanes
  show render jobs in the agent lane.
- **CRM:** quick-action chips that open gen-UI popovers [R047] live in the tools menu.
- **Support:** mode switch "Reply / Internal note" changes the send control's colour; send never
  auto-sends a reply to a customer without the permission selector allowing it.
- **Voice:** text shell is a fallback; voice-mode entry is primary; the status strip shows
  listening / thinking / speaking via the voice session.

## 7. Part specs
| Part | Spec | What it owns |
|---|---|---|
| Input shell | [composer-input-shell.md](composer-input-shell.md) | text area, container, growth, placeholder by state |
| Tools menu | [composer-tools-menu.md](composer-tools-menu.md) | `+` → × menu, submenus, overflowed controls |
| Attachments | [composer-attachments.md](composer-attachments.md) | clip, drop zone, chips / tiles with ring progress |
| Slash commands & mentions | [composer-slash-mentions.md](composer-slash-mentions.md) | `/` and `@` triggers, popovers, tokens in the draft |
| Model picker | [composer-model-picker.md](composer-model-picker.md) | model pill + popover, mid-run policy |
| Effort control | [composer-effort-control.md](composer-effort-control.md) | thinking-effort levels / slider / dial |
| Context meter | [composer-context-meter.md](composer-context-meter.md) | context-window ring / bar / grid |
| Mode switch | [composer-mode-switch.md](composer-mode-switch.md) | Ask / Plan / Agent (and category modes) |
| Permission selector | [composer-permission-selector.md](composer-permission-selector.md) | autonomy / approval level |
| Voice input | [composer-voice-input.md](composer-voice-input.md) | mic, push-to-talk, dictation strip, voice-mode entry |
| Send control | [composer-send-control.md](composer-send-control.md) | send / stop / queue / steer (worked example) |
| Queue stack | [composer-queue-stack.md](composer-queue-stack.md) | queued messages, steer notes, lanes, stash |
| Status strip | [composer-status-strip.md](composer-status-strip.md) | status tab, budget, jobs, PR pill, stall, approval, banners |

## 8. Edge cases (composer level)
- **Overflow:** footer collapse order in §2; layers above the shell share one max height
  (min(50 % viewport, 360 px)) — queue stack scrolls first, banner never collapses.
- **RTL:** the whole footer mirrors (send bottom-left); numbers, diff stats and code tokens stay LTR in
  bidi isolates.
- **Tiny / dense:** `composer/ide-panel` at 280 px shows tools, model (icon), voice, send only.
- **Touch:** every control ≥ 44 px hit area; popovers become sheets on mobile.
- **Slow networks / offline:** typing, attachments (local), queue and on-device voice keep working;
  send is disabled with a reason or queues locally (send-control edge cases).
- **Fast / slow agents:** parts must tolerate `Submitted → Done` in one frame and runs of hours
  (see each part's edge cases).
- **IME:** no part acts on Enter during IME composition.

## 9. Accessibility (composer level)
- The composer is a `form` labelled "Message the agent"; footer controls are a `toolbar` with roving
  tabindex (←/→ between pills, Tab leaves the toolbar).
- **Focus order:** banner action → status strip → queue stack → attachments → input → footer
  toolbar (tools → mode → permission → model → effort → context → voice) → send.
- **One live region per composer** (polite) owned by `oxyn-core`, so parts don't talk over each other;
  assertive only for errors, approvals and permission-denied.
- Shortcuts are listed in a `?` help popover and in each control's description; all are remappable.

## 10. References (study only unless MIT/Apache)
| Ref | What we take | Licence note |
|---|---|---|
| Pitus composer [R021], merge-conflict pill [R054] | the richest single composer; PR pill | Study only |
| Cloudy [R007] | steer-in-place, pause ≠ stop, budget chip | Study only |
| BoardUI [R036][R095] | attachment ring; one `waiting` flag drives composer glow + thinking row together (our `ComposerState` idea in their words) | Free tier MIT; Pro study only |
| affectUI composer [R044], steering [R080] | approval chip, model pill, folder tray; steer into plan | Paid — study only |
| T3 Code [R003] | branch / access mode under the composer | Study only |
| Devin concept [R033] | split send | Study only |
| zzzzshawn node composer [R066] | `composer/node` | Study only |
| renzobianchi [R079] | "the composer never moves"; exit choreography | Check source; study only |
| coss ui Composer API [R126] | draft / trigger registry / submit policy model | MIT — may port with attribution |
| Codex CLI (research note 08) | terminal layout | MIT — may port with attribution |
| desdivj CRM composer [R047], seanaiux studio [R013], flohoeller lanes [R061] | preset details | Study only |
