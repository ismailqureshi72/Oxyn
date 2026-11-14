**Status:** v1 — implemented (plan #27); the draft below is kept as the design record.

# Spec: composer mode switch (Ask / Plan / Agent + custom modes)

Follows the 10-point standard in [README.md](README.md) at the depth of the worked example
[composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro variants** on one state
machine. Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
Agent apps let the user say *what kind of work* the next message asks for: **Ask** (answer, read-only,
no edits), **Plan** (research and produce a plan to approve before anything changes), **Agent** (also
called Act / Build / Code: edit files and run tools). Apps add their own (Debug, Review, Research,
Spec, Mission). The mode switch sits at the left of the composer row. The user wants to change mode in
one key without leaving the keyboard, see the current mode from the corner of their eye (colour,
placeholder, send-button tint), and trust what happens if they switch while the agent is running.

This is table stakes (research 03): Cursor (picker + Shift+Tab; Plan auto-suggested on complex prompts),
Windsurf (Code / Plan / Ask, ⌘.), Claude (Plan as a permission mode), Codex (Plan "collaboration mode",
magenta footer label), Verdent (Agent / Plan, Shift+Tab or Ctrl+.), Factory (Normal / Spec / Mission),
Replit (Plan toggle; off = Build), Lovable (Build / Chat / Plan, ⌥P cycles), Bolt (Plan vs Build),
OpenCode (build / plan agents, Shift+Tab), Kiro (agent personas). **Shift+Tab to cycle is the de facto
keyboard standard.** TUFFUI's composer sends in the brand colour in Act mode [R081].

Mode is **intent**; permissions are **authority** (see
[composer-permission-selector.md](composer-permission-selector.md)). A mode may *suggest* a permission
preset (Plan → read-only), but the two are separate controls in Oxyn, with one exception: apps that model
Plan as a permission mode (Claude-style) can merge them with `merge_with_permissions`.

## 2. State machine

```
                        Shift+Tab (cycle)  /  click segment  /  menu pick  /  "/plan" command
  Idle(mode) ─────────────────────────────────────────────────────────────────────▶ Switching(from, to)
    │  ▲   ▲                                                                              │
    │  │   │                      agent idle (Idle / Composing / Done / Error / Cancelled) │
    │  │   └──────────────────── Applied(to) ◀────────────────────────────────────────────┤
    │  │     (tint + placeholder + send colour follow)                                     │ agent busy
    │  │                                                                                   ▼
    │  │                              to is MORE restrictive ── applies now ──▶ AppliedRestrictNow
    │  │                              (Agent → Plan / Ask)         (pending tool calls need approval)
    │  │                              to is LESS restrictive ── waits ────────▶ PendingNext(to)
    │  │                              (Ask / Plan → Agent)          (badge "→ Agent next")
    │  │                                                                  │ run ends
    │  └──────────────────────────────────────────────────────────────────┘
    │
    ├─ hover / focus ─▶ Hover / Focus ─ click (menu variants) ─▶ Open ─ ↑↓ / Enter ─▶ Switching
    ├─ app suggests (e.g. complex prompt) ─▶ Suggested(mode) ─ accept ─▶ Switching | ignore ─▶ Idle
    ├─ plan approved ("Build") ─▶ Switching(Plan → Agent) (driven by the plan card)
    └─ Locked(reason) · Disabled(reason) · Unavailable(mode, reason) per segment
```

Rules:
1. Every mode has a **restrictiveness rank** (`Ask = 0`, `Plan = 1`, `Agent = 2`; custom modes declare
   one). Switching **down** mid-run applies immediately to anything not yet executed (safety first);
   switching **up** mid-run waits for the next turn (no surprise authority). This is an Oxyn decision.
2. Shift+Tab cycles `cycle_order` (default Ask → Plan → Agent → Ask); custom modes opt in. Shift+Tab only
   cycles while focus is in the composer and no popover/autocomplete is open (otherwise it is reverse-tab).
3. The mode's colour drives four places together: the chip, the composer focus border (1 px), the
   placeholder text, and the send control's accent. Nothing else re-colours.
4. Placeholder follows mode only when the draft is empty; a non-empty draft is never touched.
5. Rapid cycling applies on each press (no confirm), but announcements and `on_change` are debounced
   300 ms so screen readers and the app hear only where the user landed.

## 3. States table

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle(mode)` | chip/segment: glyph + label ("Plan"), tinted `mode.<id>`; placeholder per mode; send tinted | open (menu) or select segment | — | Shift+Tab cycle; `Mod+.` open menu (proposed; Windsurf uses ⌘. per research 03); `/ask` `/plan` `/agent` slash commands (app wires) |
| `Hover` | bg `chip.hover`; tooltip 500 ms: mode description + "Shift+Tab to cycle" | open / select | — | — |
| `Focus` | 2 px focus ring | open | — | ← → move between segments (segmented), Enter/Space select |
| `Open` (menu variants) | list: glyph, label, 1-line description, shortcut hint; custom modes below a divider; "Edit modes…" footer | select | — | ↑↓, 1–9 jump, Enter, Esc |
| `Switching(from, to)` | label morphs letter-by-letter; tint crossfades; segment indicator glides | — | — | further Shift+Tab continues the cycle |
| `Applied(to)` | new mode; placeholder crossfades; send button tint follows | — | — | — |
| `AppliedRestrictNow` | as Applied + note under chip for 2.4 s: "Plan mode: remaining edits will ask first" | — | — | — |
| `PendingNext(to)` | current chip + dashed badge "→ Agent"; tooltip "Applies to your next message" | — | cancel (× on badge) | Esc on badge |
| `Suggested(mode)` | ghost chip next to current with "Plan?" and a 1-line reason ("Big change, plan first?") | accept | dismiss | Tab to it, Enter accept, Esc dismiss |
| `Unavailable(mode, reason)` | segment/menu row dimmed, reason in tooltip ("Plan needs a reasoning model") | — | — | focusable, Enter announces reason; cycle skips it |
| `Locked(reason)` | lock 10 px, no chevron | — | — | Enter announces reason |
| `Disabled(reason)` | muted | — | — | — |

**Per-mode defaults (proposed tokens; apps override):**

| Mode | Glyph | Colour token (dark default) | Placeholder | Send label (a11y) | Suggested permission |
|---|---|---|---|---|---|
| Ask | `?` in circle | `mode.ask` = neutral blue-grey | "Ask anything…" | "Ask" | Read-only |
| Plan | checklist | `mode.plan` = teal | "Describe what to plan…" | "Plan" | Read-only |
| Agent | spark / play | `mode.agent` = brand accent | "Tell the agent what to do…" | "Run" | app default (Ask / Auto-edit) |
| Custom | app glyph | one of 8 `mode.custom.1–8` tones | app string | app string | app choice |

(Teal for Plan is a nod to Claude Code's `planMode` theme token rgb(72,150,140) [research 08 §1.3, LEAK —
colour reference only]; Codex uses magenta for its Plan footer label (research 03). Our values are tokens
tuned in the Studio, not copies.)

## 4. Micro-interactions
- **Segment indicator:** a single pill-shaped indicator glides between segments (shared layout) and its
  colour crossfades from old to new mode tone.
- **Label morph (chip variants):** shared letters glide, new letters blur in (blur 4 px → 0) with a
  20 ms stagger; chip width springs to the new label (idea studied from Arc UI text morph, research 12;
  re-implemented).
- **Mode sheen:** on apply, one 900 ms ease-out sheen sweeps across the chip label (the MonoCode composer
  mode shimmer: single sweep, gradient at 100°, peak `color-mix(currentColor 25 %, white)` [research 08
  §1.18, MIT]).
- **Composer follow:** focus border colour crossfades; placeholder crossfades (old fades out y 0 → −3 px,
  new fades in y +3 px → 0); send control accent crossfades. All three start on the same frame.
- **Shift+Tab feedback:** each press nudges the chip 2 px in the cycle direction and springs back, so
  rapid cycling feels like a detented wheel.
- **PendingNext badge:** slides out of the chip with a dashed outline; merges back at run end.
- **Restrict-now note:** slides down from the chip; any tool row currently "queued for auto-run" in the
  transcript flips to "Awaiting approval" (that row animation belongs to the tool-call spec).
- **Suggested:** ghost chip fades in at 60 % opacity with a 1 px dashed outline; accept morphs it into
  the real chip (the old chip slides away).
- **Plan approved → Agent:** the plan card's "Build" button flies a small spark to the mode chip, then
  the chip morphs Plan → Agent (one choreography shared with the plan card spec).

## 5. Timing table

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Hover bg | 80 ms | 0 | ease-out | — |
| Tooltip | 120 ms | 500 ms | ease-out | — |
| Segment indicator glide | ~220 ms | 0 | spring 420 / 34 | — |
| Tone crossfade (chip, border, send) | 200 ms | 0 | ease-in-out | same frame start |
| Label morph | ~240 ms | 0 | spring 300 / 30; 20 ms letter stagger | — |
| Chip width | ~240 ms | 0 | spring 300 / 30 | — |
| Mode sheen | 900 ms | 60 ms | ease-out | [research 08 §1.18, MIT] |
| Placeholder crossfade | 160 ms out + 160 ms in (overlap 80 ms) | 0 | ease-out | — |
| Shift+Tab nudge | 140 ms | 0 | spring 600 / 22 | — |
| Announcement / `on_change` debounce | 300 ms | — | — | — |
| Menu open / close | ~200 ms / 120 ms | 0 | spring 380 / 32 / ease-in | — |
| PendingNext badge | 180 ms | 0 | ease-out | — |
| Restrict-now note in / hold / out | 160 / 2400 / 160 ms | 0 | ease-out / — / ease-in | — |
| Suggested ghost in | 200 ms | 0 | ease-out | — |
| **Reduced motion** | tone, label, placeholder swap with 80 ms opacity; no glide (indicator jumps), no sheen, no nudge, no letter morph | — | linear | — |

## 6. Variants

### Free (6)
| Name | Idea |
|---|---|
| `mode-switch/segmented` | 2–4 segments with gliding indicator (Ask · Plan · Agent) |
| `mode-switch/chip-menu` | Single chip "Plan ⌄" with a menu of modes + descriptions; best with custom modes |
| `mode-switch/toggle` | Binary chip (e.g. "Plan" on/off; off = Agent), as in Replit-style Plan toggles |
| `mode-switch/glyph` | TUI footer label `⏸ plan mode on (shift+tab to cycle)`-style line, tinted (terminal surface) |
| `mode-switch/persona` | Agent persona picker (avatar + name) for apps where modes are agents (OpenCode / Kiro shape) |
| `mode-switch/minimal` | Text-only "Ask / Plan / Agent" with underline indicator for dense IDE panels |

### Pro (14)
| Name | Idea |
|---|---|
| `mode-switch/dial` | Rotary 3–4 mode dial with detents (Amp "Dial" shape, research 08 §1.14), hold to save custom dials |
| `mode-switch/liquid` | Indicator is a liquid blob that stretches between segments |
| `mode-switch/tint-wash` | Whole composer surface washes to the mode tone for 400 ms on switch |
| `mode-switch/border-beam` | Mode-coloured beam travels the composer border once on switch |
| `mode-switch/split-flap` | Split-flap label flips through modes on Shift+Tab |
| `mode-switch/orbit` | Mode glyphs orbit the chip; active one docks |
| `mode-switch/card-flip` | Chip flips like a card to reveal the next mode |
| `mode-switch/pixel` | Pixel-art glyph per mode morphs cell-by-cell |
| `mode-switch/suggest-coach` | Smart suggestion chip with a reason and a one-key accept |
| `mode-switch/plan-handoff` | Plan card → Build spark choreography into the chip |
| `mode-switch/wheel` | iOS-style picker wheel (mobile) |
| `mode-switch/notch` | Desktop island shows the mode as a coloured sliver |
| `mode-switch/voice` | Voice surface: spoken "plan this" morphs the chip with a confirmation tone |
| `mode-switch/lanes` | Mode decides which lane (chat vs work) the message drops into |

## 7. Edge cases
- **Long custom labels** ("Security Review"): segmented truncates at 96 px per segment and falls back to
  `chip-menu` when segments don't fit; full label in tooltip.
- **More than 4 modes:** segmented shows the cycle-order first 3 + "More ⌄".
- **RTL:** segment order mirrors; Shift+Tab still moves "forward" in the logical cycle order.
- **Tiny sizes (16–20 px):** glyph-only chip; tone shown as a 3 px left bar plus glyph shape (never
  colour alone).
- **Dense IDE panels:** `minimal` or `glyph` variant; placeholder follow can be turned off.
- **Touch:** segments ≥ 44 px tall; no Shift+Tab, so the chip is always tappable; long-press opens menu.
- **Slow networks:** mode is local; switching never waits on the network. If the app must tell the agent
  (`ApplyNow` adapters), the chip applies optimistically and reverts with a note on failure.
- **Very fast agents:** if the run ends < 300 ms after a pending switch, skip the badge.
- **Very slow agents:** `PendingNext` persists; repeated cycling updates the badge, not the run.
- **Shift+Tab conflicts:** inside an autocomplete or mention list Shift+Tab belongs to that list; in a
  multi-field form it is reverse-tab. Apps can rebind (`shortcuts.cycle`).
- **Mode + permission merged** (Claude-style): the cycle includes permission presets; the dangerous ones
  (full access, bypass) are **never** in the cycle (see permission spec).
- **Mode unavailable for current model:** cycle skips it; if the current mode becomes unavailable after a
  model switch, fall back to the next lower-rank mode and show the reason.
- **Queued messages:** each queued chip keeps the mode it was queued in; its chip shows the mode tone.

## 8. Accessibility
- Segmented: `radiogroup` labelled "Mode", segments `radio` with `aria-checked`; chip-menu: `button`
  with `aria-haspopup="menu"`, items `menuitemradio`.
- Labels per state: "Mode: Plan. The agent will plan before changing anything. Shift+Tab to change",
  "Mode: Ask, switching to Agent from your next message", "Mode Plan unavailable: needs a reasoning model".
- Live region (polite, debounced 300 ms): "Plan mode", "Agent mode from your next message", "Plan mode:
  remaining edits will ask first".
- The send button's accessible name follows mode ("Plan", "Ask", "Run") while its role stays "send".
- Colour is never the only signal: glyph + label + placeholder change with the tone. Mode tones meet
  3:1 for the chip border/indicator and 4.5:1 for labels in both themes (Studio contrast check).
- Focus order: model → effort → **mode** (or first in row if the app puts it left) → permission →
  context → send. Shift+Tab cycling does not move focus.

## 9. Props sketch
```rust
pub struct ModeSwitch {
    pub agent: AgentStateRef,               // busy -> restrict-now / pending-next logic
    pub modes: Vec<ModeDef>,
    pub current: ModeId,
    pub pending: Option<ModeId>,
    pub cycle_order: Vec<ModeId>,           // default [ask, plan, agent]
    pub variant: ModeVariant,
    pub follow: FollowTargets,              // placeholder, send_tint, border_tint (all true by default)
    pub mid_run: ModeMidRunPolicy,          // RankBased (default) | AlwaysNextTurn | AlwaysNow
    pub merge_with_permissions: bool,       // Claude-style combined cycle; dangerous presets excluded
    pub suggestion: Option<ModeSuggestion>, // app-provided { mode, reason }
    pub shortcuts: ModeShortcuts,           // cycle: Shift+Tab, open: Mod+. (proposed)
    pub labels: ModeLabels,
    pub motion: MotionLevel,
    pub on_change: Callback<ModeChange>,    // { from, to, applied: Now | NextTurn }
}

pub struct ModeDef {
    pub id: ModeId, pub label: SharedString, pub glyph: IconName, pub tone: ToneToken,
    pub description: SharedString, pub placeholder: SharedString, pub send_label: SharedString,
    pub rank: u8,                           // restrictiveness: lower = more restrictive
    pub suggested_permission: Option<PermissionPresetId>,
    pub available: Availability,            // Yes | No(reason)
}

fn apply_timing(agent: &AgentState, from: &ModeDef, to: &ModeDef) -> Applied {
    let busy = !matches!(agent, AgentState::Idle | AgentState::Composing | AgentState::Done { .. }
        | AgentState::Error { .. } | AgentState::Cancelled);
    if !busy || to.rank < from.rank { Applied::Now } else { Applied::NextTurn }
}
```

## 10. Related components and references
**Related:** permission selector ([composer-permission-selector.md](composer-permission-selector.md)) —
modes may suggest a preset; send control (tint + label follow mode); plan document / plan approval card
("Build" switches Plan → Agent); queued-message chips (per-message mode); slash-command menu (`/plan`);
status bar / footer label (TUI); model picker (mode availability per model).

**References** (study only unless MIT/Apache):
- Research 03 mode-toggle row: Cursor, Windsurf, Claude, Codex, Verdent, Factory, Replit, Lovable, Bolt,
  OpenCode, Kiro, Copilot, Amp — closed products study only, except OpenCode (MIT) and Codex (permissive;
  licensing.md lists MIT, research 08 reads Apache-2.0 — re-check), which may be ported with attribution.
- Claude Code status glyphs `⏸ plan mode on` and Shift+Tab cycling (research 05, docs) and `planMode`
  colour (research 08 §1.3, leaked source) — reference only, never code.
- MonoCode composer mode shimmer 900 ms single sweep (research 08 §1.18) — MIT, port with attribution.
- Amp "The Dial" (modes on a dial, Ctrl+S, hold-to-save custom dials; research 08 §1.14) — study only.
- [R081] TUFFUI Agent Composer Ask / Plan / Act with Act sending in the brand colour — MIT, portable.
- [R007] Cloudy "Work" chip and placeholder that flips while running; [R033] Devin composer — study only.
- Arc UI text morph and chip-group glide (research 12) — paid / unclear, study only.
- [R050] global shape cycle on one key (R) — study only; informs detented cycling feel.


## Plan #27 implementation — mode switch

Implemented in `oxyn-core::composer::parts`, `oxyn-dioxus::{ModeSwitch}` (styles in `PARTS_CSS`, bundled into `PRIMITIVES_CSS`), `oxyn-gpui::{ModeSwitchEl}`.

### 27.1 Purpose
Switch the agent between Ask, Plan and Agent from the composer.

### 27.2 State machine
```
Idle: select(mode) ─▶ Applied::Now
Busy: select(mode) ─▶ Applied::NextTurn (pending shown) ─turn_end─▶ applied
Shift+Tab in the preview ─▶ cycle
```

### 27.3 States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | segments or chip | Tab, Enter | radiogroup / listbox |
| pending | “from the next turn” note | — | polite status |
| menu open | list with descriptions | Escape | listbox |

### 27.4 Micro-interactions
The selected segment slides (160 ms); the placeholder of the preview changes with the mode.

### 27.5 Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Segment move | 160 ms | `--ox-ease` | instant |
| Menu in | 160 ms | `--ox-ease` | instant |

All motion in the parts drops to 1 ms under `prefers-reduced-motion`, except `.oxyn-timer`.

### 27.6 Variants
Free: segmented.

**Oxyn plus:** menu variant, busy → next-turn change, composer preview.

### 27.7 Edge cases
Choosing the current mode is a no-op. A pending change can be replaced before the turn ends.

### 27.8 Accessibility
Segments are radios with `aria-checked`; the menu is a listbox; a queued change is announced.

### 27.9 Props sketch
```rust
ModeSwitch { modes: Vec<Mode>, value, busy: bool, variant: "segmented"|"menu", preview: bool, on_change: EventHandler<(String, bool)> }
```

### 27.10 Related components and references
`composer-permission-selector.md`.
