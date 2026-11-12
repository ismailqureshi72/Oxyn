**Status:** draft v0.1 (2026-11-12).

# Spec: desktop overlays — tray popover, hotkey launcher, floating HUD, notch / island (`tray-popover`, `hotkey-launcher`, `floating-hud`, `island`)

Part of family J ([workspace-chrome.md](workspace-chrome.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Four OS-level surfaces that live **outside the main
window**: a menu-bar / system-tray **popover**, a global-hotkey **launcher**, an always-on-top **floating
HUD**, and a **notch / island** status at the top of the screen. Target **8 free + 17 Pro = 25 variants**.
These are the hosts for the many `…/notch` Pro variants other specs already declare
(`approval-card/notch`, `acting-indicator/notch`, `composer-status-strip` `notch`,
`composer-context-meter` `context-meter/notch`, `model-picker/notch`, `mode-switch/notch`,
`permission-selector/notch`, `composer-input/notch`, `background-jobs/notch`, voice `notch`): this spec
owns the **island host** they render into. Numbers are Oxyn **proposals**; platform facts in §2.2 are
from general platform knowledge and are marked **verify** — they must be checked against current OS
documentation and GPUI's window API before implementation.

## 1. Purpose
Agents run for minutes to hours while the user works in other apps. The user wants to **see whether the
agent is working, done or waiting on them without switching windows, answer a quick approval, and start a
new request from anywhere**. Research found these surfaces **missing entirely** from component libraries
— "Menubar/tray agent popover", "Global-hotkey 'spotlight' agent launcher", "Always-on-top floating agent
HUD / mini-window", "Notch / Dynamic-Island live activity for agent status (SmoothUI and Skiper have
generic islands only)" (research note 01 §6.A) — and named "desktop HUD/notch status" as a signature free
piece (note 01). What apps do: Antigravity has a **menu-bar icon showing the count of running agents**
(note 08 §1.11); Claude desktop minimizes panes to a floating corner tile (note 03); Codex pets are a
floating overlay whose bubbles mirror agent activity, clicking a pet opens a reply path (note 08 §1.2);
Zed keeps the system awake while generating and sends OS notifications (note 03); Aceternity lists a
generic **Notch** component and Skiper UI a **Dynamic island** (notes 16, 17).

## 2. Anatomy
```
 Tray popover (anchored under the menu-bar / tray icon; 360 × ≤ 480 px)
   menu bar:  … ◆3  🔋 …          icon: agent mark + running count; amber dot when someone needs you
            ┌──────────────────────────────────────────┐
            │ ◆ 3 running · 1 needs you      ⚙  ⤢ Open │
            │ ✋ Researcher: run `rm -rf dist`? [Allow ▾][Deny] │   needs-you first (family G footer, compact)
            │ ⠋ Fix auth refresh   Editing tokens.rs 4:12 │
            │ ⠋ Render launch.mp4  41%                  │
            │ ✓ Docs update        Done 3m ago   [Open] │
            │ ┌ Ask the agent…                     ↵ ┐ │   optional mini composer
            └──────────────────────────────────────────┘

 Hotkey launcher (centred on the active screen, top at 25 %; 680 px wide)
            ┌ ✦ Ask or run…                    [this window ▾] ⌥Space ┐
            │ ▸ New thread in "api"                                  │  reuses command-palette rows + ask row
            │   Ask the agent: "summarise this PDF"            Tab   │  scope chip: what the agent may see
            └────────────────────────────────────────────────────────┘

 Floating HUD (always on top, default bottom-right, 280 × 64 px pill / 280 × 160 card)
            ╭──────────────────────────────────────╮
            │ ◆⠋ Editing tokens.rs · 4:12   ■  ⤢  │   click-through body; handle + buttons interactive
            ╰──────────────────────────────────────╯

 Notch / island (top-centre; wraps the camera housing on notched Macs, a pill elsewhere)
   collapsed:      ( ◆⠋        ▬▬▬▬camera▬▬▬▬        4:12 )      ← glyph left of the housing, timer right
   expanded:       ╭ ◆ Researcher needs you ─ run `rm -rf dist`? ─ [Allow] [Deny] [Open] ╮
```

### 2.1 Shared rules (proposed)
1. **One source of truth.** All four read the same `AgentSummary` (runs, needs-you, jobs) the main
   window uses — [composer-status-strip.md](composer-status-strip.md) state, [background-jobs.md](background-jobs.md)
   jobs, family G requests. No overlay invents state.
2. **Only one overlay speaks at a time** for a given event: priority island > HUD > tray badge > OS
   notification (coordinated with [command-palette.md](command-palette.md) §2.2 "never duplicate").
3. **Overlays never take keyboard focus unasked.** They show; the user summons focus (hotkey, click).
   The launcher is the exception because the user summoned it.
4. **Mirror only when the main window is not visible** (unfocused, minimised, or on another space /
   desktop) unless the user pins an overlay.
5. **Approvals from overlays are allowed** only for requests the permission level marks low / medium risk
   ([permission-rules.md](permission-rules.md)); high-risk requests show "Review in app" only.
6. **Privacy.** Overlays hide prompt text and file names on the lock screen and when screen sharing is
   detected (option "Hide details while sharing", default on; detection support is platform-dependent —
   verify).

### 2.2 Platform matrix (verify every cell before implementing)
| Concern | macOS | Windows | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| Tray / menu-bar icon | menu-bar status item; template (monochrome) icon recommended | notification-area icon; Windows 11 hides new icons in the overflow by default — tell the user how to pin it | StatusNotifierItem / AppIndicator where the desktop supports it; legacy XEmbed tray elsewhere | StatusNotifierItem where supported; **GNOME shows tray icons only with an extension** — always offer the HUD as a fallback |
| Popover anchoring | below the status item | above / near the icon (taskbar position varies: bottom, top, sides) | near the pointer if the icon rect is unknown | compositor-dependent; fall back to a normal window near the top-right |
| Global hotkey | system-wide shortcut registration; conflicts with system shortcuts must be detected | global hotkey registration; Win+ combos largely reserved by the OS | X11 key grab | **no general global-hotkey API for apps**; use the desktop portal's GlobalShortcuts where available, else ask the user to bind a desktop shortcut that runs `app --launcher` |
| Always on top | floating window level; can join all spaces | topmost window | `_NET_WM_STATE_ABOVE` hint (WM-dependent) | not guaranteed for regular windows; layer-shell overlay layer on compositors that support it (not GNOME) |
| Click-through | window ignores mouse events (toggle per state) | layered + transparent extended style | input shape / empty input region | empty `wl_surface` input region |
| Precise positioning | yes | yes | yes (WM may adjust) | **apps cannot place regular windows**; layer-shell anchors where available; otherwise the HUD lives inside the main window |
| Notch | notched MacBook displays expose the camera housing via screen safe-area / auxiliary-area APIs; non-notched displays have none | no notch; island = top-centre pill below the screen edge | no notch; top-centre pill | no notch; layer-shell top anchor where available, else no island |
| Exclude from screen capture | window sharing type "none" | display affinity "exclude from capture" (newer Windows 10/11 builds) | not available generally | not available generally |
| Full-screen apps | overlays can appear over full-screen spaces only if configured to; default **hide** | topmost windows may cover full-screen apps; default hide when a full-screen app is foreground | WM-dependent; default hide | default hide |
| Web (Dioxus) | — in-page equivalents only: HUD as a fixed in-page pill, island as a top in-page pill, launcher = ⌘K palette, no tray | | | |

GPUI support for each window option (window level, click-through, capture exclusion, layer-shell) is
**not yet established** in our notes; see Gaps in the summary and roadmap. Components declare
`capabilities_required` and degrade by the fallback ladder: **island → HUD → tray badge → in-window
status strip → OS notification**.

### 2.3 Click-through rules (floating HUD and island)
| Situation | Pointer behaviour |
|---|---|
| Resting, nothing needs the user | **click-through** body; the 24 px handle, ■ and ⤢ buttons are hit-testable (on platforms that support per-region input; otherwise the whole HUD is interactive and smaller: 200 × 40 px) |
| Holding ⌥ / Alt for 300 ms over the HUD | whole HUD becomes interactive while held (drag anywhere, hover shows tooltips) |
| Needs you (approval / question) | whole overlay **interactive**; it is now asking for input |
| Pointer hovers 600 ms over the island (macOS) | island expands to its detail state; leaving for 400 ms collapses it |
| User dragging something across the screen (drag session active) | overlays become click-through and fade to 40 % so they never steal a drop (platform support — verify) |
| Secure input / password fields focused elsewhere | overlays never request focus; HUD shows no typed content |
| Screen recording / sharing detected and "Hide details" on | text replaced by "Agent working"; approvals show "Review in app" |

## 3. State machines
Shared overlay lifecycle:
```
 Off (user disabled) ⇄ Armed (enabled, main window visible → overlay hidden)
 Armed ─main window hidden / unfocused (debounce 800 ms)─▶ Mirroring{summary}
 Mirroring ─main window focused─▶ Armed (overlay fades out)      Pinned: Mirroring persists regardless
 Mirroring{summary} by AgentSummary:
   Idle ──run starts──▶ Working{verb, t} ──needs you──▶ NeedsYou{request} ──answered──▶ Working
     ▲                     │ stall_soft ▶ Stalled ──event──▶ Working
     │◀── 6 s ── Done{elapsed} ◀── run ends ok ─┤
     │◀── sticky ─ Failed{kind} ◀── error ──────┘           Offline / Lost connection ─▶ Disconnected (grey)
 Capability gate:  any ─capability missing─▶ Fallback{next host}
 Privacy gate:     any ─sharing detected / screen locked─▶ Redacted(same state, no details)
 Full-screen gate: any ─foreground app is full screen─▶ Suppressed (unless Pinned or NeedsYou + allowed)
```
Per surface:
```
 Tray:     IconIdle ⇄ IconWorking{count} ⇄ IconNeedsYou ; icon click ─▶ PopoverOpen ─Esc / blur─▶ closed
 Launcher: Hidden ─hotkey─▶ Summoned{query ""} ─▶ (command-palette.md states) ─run / ask / Esc / blur─▶ Hidden
           HotkeyConflict{combo} (registration failed) ─▶ settings prompt ; Unsupported (Wayland w/o portal) ─▶ instructions
 HUD:      Pill ⇄ Card (⤢) ; Resting(click-through) ⇄ Interactive(Alt held | NeedsYou | hover handle)
           Dragging ─release─▶ Snapped{corner | edge} ; Docked-to-edge (Pro) ⇄ Peeking
 Island:   Collapsed ─event─▶ Bloom (brief expand 2.5 s) ─▶ Collapsed
           Collapsed ─hover 600 ms / click─▶ Expanded ─leave 400 ms / Esc─▶ Collapsed
           Expanded{NeedsYou} stays until answered or dismissed
```

## 4. States table
| State | Tray | Launcher | HUD | Island | Actions / keyboard |
|---|---|---|---|---|---|
| `Off` / `Armed` | icon only (if enabled) | hotkey registered | hidden | hidden | — |
| `Idle` | mark, no count | — | hidden unless pinned ("Idle") | hidden unless pinned (thin sliver) | — |
| `Working{verb,t}` | mark + count "3"; mark's loader runs at reduced tempo | — | glyph loader + verb + elapsed | loader left of housing, elapsed right | HUD ■ stop (confirm if costly), ⤢ open app |
| `NeedsYou` | amber dot on icon | — | amber edge, request line, Allow / Deny (low/medium risk) or Review | blooms to expanded with request + actions | no default Enter (approvals need an explicit button); Esc collapses |
| `Stalled` | icon mark tone ramps to warning | — | warning tone "No response 45s" + Retry | warning tone | Retry |
| `Done{elapsed}` | ✓ flash on icon 2 s | — | "✓ Worked for 4m 12s · Open" 6 s | ✓ bloom 2.5 s | Open |
| `Failed{kind}` | danger dot until opened | — | "✕ Failed · Retry · Open" sticky | ✕ sticky sliver | Retry, Open |
| `Disconnected` | grey mark | ask row disabled with reason | grey "Offline" | hidden | — |
| `Redacted` | count only | — | "Agent working" | glyph only | — |
| `Summoned` (launcher) | — | input focused, recents | — | — | see [command-palette.md](command-palette.md) §4 |
| `HotkeyConflict` / `Unsupported` | — | settings card "⌥Space is taken by another app — choose another" / "Bind a desktop shortcut to `oxyn --launcher`" (example) | — | — | choose |
| `Fallback{host}` | — | — | — | — | the next host renders the state |

## 5. Micro-interactions
- **Tray icon.** Count changes roll (one digit, tabular); working state uses the mark's slow loop at
  0.5× tempo so the menu bar is calm; needs-you adds a 6 px amber dot that pops once (`spring.soft`);
  never blinks continuously.
- **Tray popover open.** Scale 0.96 → 1 from the icon, 160 ms; rows stagger 30 ms (cap 6); needs-you row
  is first with an amber edge.
- **Launcher summon.** Appears on key down, at full size, input focused in the first frame; 120 ms fade +
  8 px rise. Dismiss on blur with no animation delay so the previous app regains focus instantly.
- **HUD appear.** When the main window loses focus and a run is live, the HUD fades + rises 8 px from its
  corner after the 800 ms debounce; it **detaches from the status strip's position** only conceptually
  (no cross-window flight).
- **HUD drag.** Follows 1:1; release snaps to the nearest corner or edge with `spring.snap`; 16 px screen
  margin; remembers position per display.
- **Pill ⇄ card.** Height morph with `spring.snap`; card shows last 3 steps and the job pill.
- **Alt-hold interactive.** HUD outline brightens 120 ms when it becomes interactive, dims when released.
- **Island bloom.** On an event the island widens from the housing outward (both sides), shows the message
  2.5 s, then retracts; widening and corner radius morph together on `spring.soft` (small overshoot is
  allowed only here — it reads as physical). NeedsYou bloom stays expanded.
- **Island loader.** Uses a T1/T2 loader ([loaders.md](loaders.md)) at 16–18 px in the safe area beside
  the housing; elapsed text 1 Hz.
- **Done / fail.** Same morphs as loaders.md §4 (✓ morph, hold, fade / ✕ morph + 2 × 3 px nudge — nudge
  disabled on the island to avoid jitter at the screen edge).

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Mirror debounce (window hidden → overlay) | — | 800 ms | — | — | — |
| Overlay fade in / out | 200 ms (8 px rise) / 160 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Tray count roll | 280 ms | max 2 Hz | `ease.out` | — | 30 Hz |
| Tray working loop | mark loop at 0.5× tempo | 0 | per mark | — | ≤ 15 Hz (menu bar) |
| Tray needs-you dot pop | ~240 ms | 0 | `spring.soft` | — | 60 Hz |
| Tray popover open / close | 160 ms / 120 ms | 0 | `ease.out` / `ease.in` | rows 30 ms, cap 6 | 60 Hz |
| Launcher summon / dismiss | 120 ms (8 px) / 0 ms | 0 (key down) | `ease.out` / — | — | 60 Hz |
| HUD drag snap | ~220 ms | release | `spring.snap` | — | 60 Hz |
| HUD pill ⇄ card | ~220 ms | 0 | `spring.snap` | — | 60 Hz |
| Alt-hold interactive | 120 ms outline | 300 ms hold | `ease.out` | — | 60 Hz |
| Island bloom / hold / retract | ~320 ms / 2.5 s / ~260 ms | 0 | `spring.soft` / — / `spring.snap` | — | 60 Hz |
| Island hover expand / collapse | ~260 ms / ~220 ms | 600 ms / 400 ms | `spring.soft` / `spring.snap` | — | 60 Hz |
| Done hold (HUD) | 6 s | — | — | — | — |
| Stall ramp | 2 s | `stall_soft` | OKLab linear | — | 15 Hz |
| **Reduced motion** | no rises, blooms, pops, rolls or overshoot: overlays appear / disappear with an 80 ms opacity fade; island switches between collapsed and expanded instantly; loaders show still frames ([loaders.md](loaders.md) §2.6); elapsed text still ticks at 1 Hz; tray icon swaps static glyphs (working / needs-you / done) | — | linear | none | 1 Hz |

## 7. Variants
### Tray popover — free (2) / Pro (4)
| Name | Tier | Idea |
|---|---|---|
| `tray-popover/list` | Free | Baseline: needs-you first, running runs and jobs, recent done; Open app (desktop) |
| `tray-popover/mini-thread` | Free | List + a one-line composer to start a new request in the last workspace |
| `tray-popover/count-mark` | Pro | Animated menu-bar mark with running count and a radial ring for the longest job (Antigravity count idea, note 08) |
| `tray-popover/approvals-only` | Pro | A minimal popover that only shows pending low-risk approvals with Allow / Deny |
| `tray-popover/usage` | Pro | Adds today's runs, tokens and budget meters ([run-meter.md](run-meter.md)) |
| `tray-popover/pet` | Pro | Tray mark is a tiny mascot that mirrors agent activity (Codex pets idea, note 08; [loader-mascot.md](loader-mascot.md)) |

### Hotkey launcher — free (2) / Pro (4)
| Name | Tier | Idea |
|---|---|---|
| `hotkey-launcher/spotlight` | Free | Centred box reusing `command-palette/centered` (commands + ask row + inline answer) |
| `hotkey-launcher/bar` | Free | Thin full-width bar at the top of the active screen; answer drops below |
| `hotkey-launcher/screen-context` | Pro | Scope chip "This window" attaches a capture of the frontmost window (user-visible, removable) |
| `hotkey-launcher/selection-ask` | Pro | Summoned with selected text (where the OS exposes it) prefilled as context |
| `hotkey-launcher/voice-hold` | Pro | Hold the hotkey to dictate; release to send ([composer-voice-input.md](composer-voice-input.md)) |
| `hotkey-launcher/handoff` | Pro | Answer has "Continue in app" that opens the thread in the main window at the same scroll |

### Floating HUD — free (2) / Pro (4)
| Name | Tier | Idea |
|---|---|---|
| `floating-hud/pill` | Free | 280 × 40–64 px pill: loader, verb, elapsed, Stop, Open; click-through body |
| `floating-hud/card` | Free | 280 × 160 px card with last 3 steps, job pill and needs-you actions (Claude desktop corner-tile idea, note 03) |
| `floating-hud/edge-dock` | Pro | Docks into a screen edge as a 6 px glowing strip; hover peeks it out |
| `floating-hud/ghost` | Pro | Fully click-through translucent caption of the agent's current step, Alt to interact |
| `floating-hud/live-thumb` | Pro | Live thumbnail of what the agent is driving (browser / computer use) with take-over ([acting-indicator.md](acting-indicator.md)) |
| `floating-hud/multi-agent` | Pro | One mini row per running agent with tone and status; collapses to stacked dots |

### Notch / island — free (2) / Pro (5)
| Name | Tier | Idea |
|---|---|---|
| `island/notch` | Free | Wraps the camera housing on notched Macs: loader left, elapsed right; blooms on events; falls back to `island/top-pill` |
| `island/top-pill` | Free | Top-centre pill below the screen edge for non-notched displays, Windows and Linux |
| `island/live-activity` | Pro | Expanded island with a progress ring, step text and Open / Stop like a phone live activity |
| `island/approvals` | Pro | Host for `approval-card/notch`: "✋ 1 approval · Allow / Deny" for low-risk requests |
| `island/agent-dots` | Pro | One coloured dot per running agent around the housing; dot pulses when that agent needs you |
| `island/voice` | Pro | Listening / speaking waveform in the island for voice agents (family K host) |
| `island/meters` | Pro | Context % and budget mini rings beside the housing (`context-meter/notch` host) |

## 8. Edge cases
- **No tray support** (GNOME without extension, some Wayland setups): tray variants report
  `Fallback(HUD)`; settings explain why.
- **Hotkey taken** or cannot be registered: never silently fail — `HotkeyConflict` card with alternatives;
  no default hotkey is enabled without the user confirming it in onboarding (proposed).
- **Multiple displays**: launcher opens on the display with the pointer (or the frontmost window); HUD
  position is per display; island on the display with the notch, else the primary.
- **Display changes** (notch display disconnected): island moves to `top-pill` on the new primary.
- **Full-screen video / presentations**: overlays suppressed; NeedsYou falls back to an OS notification.
- **Long verbs / names**: HUD 28 ch, island 24 ch collapsed / 60 ch expanded; ellipsis with tooltip.
- **Many runs** (10+): tray lists needs-you, then running (max 5) + "and 6 more · Open app"; island shows
  the most urgent one plus a count.
- **Agent finishes while the user is answering in the overlay**: the action is applied if still valid,
  else "This request has ended".
- **Main window crashed / closed but agent process alive**: overlays keep mirroring from the background
  process if the app architecture supports it; else they show `Disconnected`.
- **Sleep / lid close**: on wake, the overlays show the state as of the last event with "updated 3m ago"
  until a fresh event arrives (never fake Working).
- **RTL**: tray popover and HUD mirror; island keeps the loader on the inline-start side of the housing.
- **Tiny sizes**: menu-bar mark is 16–18 px; count drops above 9 to a dot + "9+" in the popover only.
- **Touch** (Windows tablets): HUD and island are fully interactive (no hover); 44 px targets.
- **Slow networks**: remote runs show "last update 40s ago" in HUD / popover; stall detection pauses while
  offline.
- **Very fast runs** (< 2 s): no HUD / island appearance at all; only the final done state if the window
  is unfocused (and only if the user enabled done notifications).

## 9. Accessibility
- **Tray icon:** accessible label updates with state ("Oxyn, 3 agents running, 1 needs you"); popover is a
  `dialog` with the needs-you list first; Esc closes and returns focus to the previously active app.
- **Launcher:** same roles as [command-palette.md](command-palette.md) §9; on dismissal focus returns to the
  previous app.
- **HUD and island:** exposed as accessible windows with role `status` and a name; they do **not** take
  focus when they appear; an OS-level shortcut (proposed ⌃⌥⌘H / Ctrl+Alt+Shift+H) focuses the HUD for
  keyboard users; inside, Tab cycles its buttons, Esc returns focus.
- **Announcements:** needs-you announced once (assertive) via the OS notification or the main window's
  live region — not duplicated by every overlay; done / failed polite; working never announced repeatedly.
- **Click-through** never applies when assistive technology requests keyboard focus on the overlay.
- **Contrast:** overlays sit on arbitrary wallpapers and apps, so they always draw an opaque or ≥ 85 %
  opaque background with text ≥ 4.5:1; menu-bar marks use template icons that follow the OS light / dark
  menu bar.
- **Reduced motion and reduced transparency** OS settings are both honoured (reduced transparency
  removes blur and uses opaque fills).

## 10. Props sketch
```rust
pub struct DesktopOverlays<'a> {
    pub summary: &'a AgentSummary,           // runs: Vec<(RunId, AgentState)>, needs_you: Vec<RequestRef>, jobs: &[JobStatus]
    pub tray: Option<TrayConfig>,            // { variant: TrayVariant, show_count: bool, popover_composer: bool }
    pub launcher: Option<LauncherConfig>,    // { hotkey: Option<Hotkey>, variant, scope_default: Scope, palette: PaletteSources }
    pub hud: Option<HudConfig>,              // { variant, corner: Corner, pinned: bool, click_through: ClickThrough /* Body | Off */ }
    pub island: Option<IslandConfig>,        // { variant, bloom_hold: Duration /* 2.5 s */, hover_expand: Duration /* 600 ms */ }
    pub mirror_when: MirrorPolicy,           // WindowHidden (default) | Always | Never
    pub mirror_debounce: Duration,           // 800 ms
    pub overlay_approval_max_risk: RiskTier, // Medium
    pub privacy: OverlayPrivacy,             // { redact_when_sharing: true, redact_on_lock: true, exclude_from_capture: false }
    pub fullscreen: FullscreenPolicy,        // Hide (default) | Show
    pub capabilities: &'a PlatformCaps,      // probed at start: tray, global_hotkey, always_on_top, click_through,
                                             // per_region_input, notch_safe_area, capture_exclusion, layer_shell
    pub on_open_app: Callback<Option<RunId>>,
    pub on_stop: Callback<RunId>,
    pub on_decide: Callback<(RequestRef, Decision)>,  // only for requests ≤ overlay_approval_max_risk
    pub on_ask: Callback<AskRequest>,
    pub motion: MotionLevel,
}
```
Hosts: other specs' `…/notch` variants implement `IslandContent` (collapsed glyph, collapsed trailing
text, expanded view, priority) and register with the island host; the host picks the highest-priority
content (needs-you > failed > working > done > meters).

## 11. Related components and references
**Related:** [command-palette.md](command-palette.md) (launcher reuses it; notification policy) ·
[composer-status-strip.md](composer-status-strip.md) (summary state, stall) ·
[background-jobs.md](background-jobs.md) (`menu-bar`, `corner-tile`, `notch` ideas) ·
[approval-card.md](approval-card.md) (`approval-card/notch`) · [acting-indicator.md](acting-indicator.md)
(`acting-indicator/notch`, take over) · [permission-rules.md](permission-rules.md) (risk tiers) ·
[loaders.md](loaders.md) · [loader-mascot.md](loader-mascot.md) · [run-meter.md](run-meter.md) ·
[composer-voice-input.md](composer-voice-input.md) · [workspace-chrome.md](workspace-chrome.md).

| Reference | What we take | Licence note |
|---|---|---|
| Research note 01 §6.A gap list | these four surfaces are missing from libraries | Our research |
| Antigravity (note 08) | menu-bar icon with running-agent count; completion / action-required sounds | Proprietary — reference only |
| Claude desktop (note 03) | minimize to a floating corner tile | Proprietary — reference only |
| Codex pets (note 08) | floating overlay mirroring agent activity, click to reply | App proprietary; CLI pet code is MIT / Apache-2.0 — portable with attribution (re-check) |
| Zed (note 03) | OS notifications, keep system awake while generating | Mixed GPL / Apache — study only by default |
| Aceternity Notch [R122] (note 17) | generic notch component exists | Licence not recorded in licensing.md — study only |
| Skiper UI Dynamic island [R104] (note 16) | generic island exists | Paid — study only |
| SmoothUI (note 01) | generic island only | MIT per note 01, not yet in licensing.md — re-check before porting |
| Platform window APIs (§2.2) | capability matrix | Platform facts — verify against current OS docs |
