**Status:** v1 — implemented (plan #23).

# Spec: button, icon button, action button

> Implemented in `oxyn-core::primitives::button` (variants, sizes, press scale, agent binding,
> `ActionButton`, `HoldConfirm`), `oxyn-dioxus::{Button, IconButton, ActionButton}` with
> `PRIMITIVES_CSS`, and `oxyn-gpui::{ButtonEl, IconButtonEl, ActionButtonEl}`. Parity sources:
> affectUI Button and Icon Button, Arc UI Button and Action button, Beautiful UI `button` (MIT,
> © 2026 Shane Levine). affectUI and Arc are behaviour only; nothing was copied.

## 1. Purpose
The one control every surface uses to act: send, approve, run, retry, open. Agent apps press
buttons while something is *already happening*, so a button has to say "busy", "not now, and why",
"done" and "failed, try again in 4 s" without changing size or moving its neighbours. The icon
button is the same control at toolbar density, named by its tooltip. The action button is a
self-contained press → pending → done / failed loop for frequent toolbar actions (deploy, sync,
run evals).

## 2. State machine

```
            pointer enter            pointer down / Space down
  Idle ───────────────────▶ Hover ───────────────────────────▶ Pressed
   ▲  ◀─────────────────────  │  ◀───────────────────────────── │  release outside: cancel
   │        pointer leave      │          release inside        ▼
   │                            └──────────────────────────────▶ (activate) ── onclick
   │ loading=false                                               │
   ├──────────────── Loading(progress?) ◀── loading=true ────────┘  (presses ignored, width kept)
   └──────────────── Disabled(reason?)                               (presses ignored, reason shown)

  Hold-to-confirm:  Idle ─down─▶ Filling(p) ─p=1─▶ Confirmed (onclick)  ;  up early ─▶ Draining ─▶ Idle
  Action button:    Idle ─press─▶ Pending ─ok─▶ Success ─reset_after─▶ Idle
                                         └fail─▶ Error{retry_in} ─backoff over─▶ Error{None} ─press─▶ Pending
```

`agent_bound(&AgentState)` (Oxyn plus) maps the agent onto a button: Submitted / Thinking /
Streaming / ToolRunning / Steering / Stalled → `Loading`; AwaitingApproval → `Disabled("Waiting
for your approval")`; everything else → `Idle`.

## 3. States table
| State | Shows | Primary action | Keyboard | ARIA |
|---|---|---|---|---|
| Idle | label, optional left / right icon, shortcut chip | `onclick` | Enter / Space activate; Tab focus | `button` |
| Hover (fine pointer only) | surface darkens one step (ghost gets a surface) | — | — | — |
| Focus (keyboard) | 2 px `color.focus` ring, 2 px offset | — | — | — |
| Pressed | scale 0.97 (icon-only 0.94) | — | Space held | — |
| Loading · shimmer | spinner replaces the left icon; a band sweeps the label | none (ignored) | focus kept | `aria-busy=true`, `aria-disabled=true` |
| Loading · spinner | spinner only | none | focus kept | same |
| Loading · progress | fill grows behind the label to `progress` | none | focus kept | same |
| Disabled | 45 % opacity, `not-allowed` | none | stays focusable when it has a reason | `aria-disabled=true`; reason as `title` + hidden text |
| Hold (destructive) | "Hold to delete" → "Keep holding…" while the fill runs | `onclick` when full | Space held 1.2 s | `aria-describedby` hint |
| Action · pending | spinner + `pending_label` | ignored | — | `aria-busy=true` |
| Action · success | drawn check + `success_label` (success colour) | — | — | polite live region says it |
| Action · error | ↻ + "Retry in 4s" → "Retry" | retry when the countdown ends | — | `aria-disabled` during back-off |
| Icon button · pressed (toggle) | selected surface; optional morph to `pressed_icon` | toggles | Enter / Space | `aria-pressed` |

## 4. Micro-interactions
- **Press.** Scales to 97 % in 80 ms on press, back on release (icon-only buttons 94 %, Arc). Never
  bounces (motion.md: presses do not overshoot).
- **Label / icon change.** The width glides to the new size on `spring.snap` and the new content
  fades in out of a 3 px blur (affectUI, Arc). Action button labels rise 5 px as they fade in.
- **Loading.** The left icon becomes a 750 ms arc spinner; a highlight band sweeps the label every
  1.4 s (shimmer). The button keeps its width so the layout never jumps.
- **Hold.** A fill runs left → right for 1.2 s; releasing drains it in 200 ms; completing fires
  once.
- **Success check** draws itself in 240 ms. **Icon morph** (copy → check, mic → stop) scales from
  60 % with a −20° turn.
- **Badge** on an icon button: a dot (unread) or a count, ringed in the page colour so it reads on
  any surface.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Frame rate |
|---|---|---|---|---|
| Hover colour / shadow | 120 ms (`dur.fast`) | 0 | `ease.out` | 60 Hz |
| Press scale | 80 ms (`dur.instant`) | 0 | `ease.out` | 60 Hz |
| Width glide on label change | 214 ms (`spring.snap`) | 0 | spring | 60 Hz |
| Label crossfade + blur | 180 ms (`dur.base`) | 0 | `ease.out` | 60 Hz |
| Loading shimmer band | 1400 ms loop (`dur.shimmer.sweep`) | 0 | linear | 30 Hz |
| Spinner | 750 ms / turn | 0 | linear | 60 Hz |
| Hold fill / drain | 1200 ms / 200 ms (`dur.approval.hold/drain`) | 0 | linear / `ease.out` | 60 Hz |
| Success hold (`reset_after`) | 1600 ms (Oxyn plus: user-set) | — | — | — |
| Retry back-off | `backoff` × 2ⁿ, max 30 s | after failure | 1 Hz countdown | 1 Hz |
| Check draw | 240 ms (`dur.check.draw`) | 0 | `ease.out` | 60 Hz |
| **Reduced motion** | no press scale; label swaps with an 80 ms fade; shimmer off (static label); spinner slows to 2.4 s per turn instead of freezing; hold keeps its 1.2 s but fills in steps | — | linear | — |

## 6. Variants
Source: `primary`, `secondary`, `ghost`, `destructive` (Arc `danger`); sizes `sm`, `md`, `lg`;
`loading`; `icon`, `iconRight`. Icon button: `ghost`, `secondary`, `primary`; `sm`, `md`; `pressed`.
Beautiful UI adds `quiet`, `success`, `accent` (folded into primary; chrome is monochrome).

**Oxyn plus (≥ 2 each):**
1. `outline`, `link`, `quiet`, `success` variants → 8 variants.
2. Sizes `xs` (24) and `xl` (48) → 5 sizes; icon button 16 / 28 / 32 / 36 / 40 px.
3. Loading style: `shimmer` (source) · `spinner` · `progress` fill with a value.
4. Hold-to-confirm for destructive actions.
5. Disabled-with-reason (tooltip + screen-reader text) and the `AgentState` binding.
6. Shortcut chip (`Mod+Enter`, platform-aware) with `aria-keyshortcuts`.
7. Icon button: badge (dot or count) and icon morph when pressed.
8. Action button: retry with exponential back-off and a countdown; adjustable success hold.

## 7. Edge cases
- **Long labels** never wrap; the button grows. In a constrained toolbar the host truncates with an
  ellipsis and the full label goes to `title`.
- **Label change while loading** keeps the loading width until loading ends, then glides once.
- **Double press** during `Pending` is ignored (core returns `false`), so a slow action can't run
  twice. Back-off presses are ignored until the countdown ends.
- **Tiny sizes:** `xs` drops the gap to 4 px and the icon to 12 px; icon buttons under 20 px keep a
  24 px hit area through padding in the host.
- **Touch:** no hover state; press scale still applies. Hold uses pointer capture semantics: leaving
  the button cancels and drains.
- **RTL:** icon order follows the inline direction (flex). The hold fill runs from the inline start.
- **Very fast actions** (< 300 ms) still show the success state for the full hold so it is seen.

## 8. Accessibility
- Native `<button type=button>`; GPUI `Role::Button` with the label as the accessible name.
- Icon buttons always carry `aria-label` (= tooltip text).
- Loading: `aria-busy=true` + `aria-disabled=true`; focus stays on the button so keyboard users
  don't lose their place. Disabled-with-reason stays focusable and exposes the reason.
- Toggle icon buttons expose `aria-pressed`. Shortcuts expose `aria-keyshortcuts`.
- Action results are announced through a polite live region inside the button.
- Contrast: primary is white-on-black (dark: black-on-white) ≥ 15:1; secondary text ≥ 7:1.
  Destructive and success use the status tokens, which meet 4.5:1 on their fills.

## 9. Props sketch
```rust
Button { label, variant: "primary|secondary|ghost|destructive|outline|link|quiet|success",
         size: "xs|sm|md|lg|xl", loading, loading_style: "shimmer|spinner|progress", progress,
         icon, icon_right, disabled, reason, hold, shortcut, full, onclick }
IconButton { icon, label, variant: "ghost|secondary|primary", size, pressed: Option<bool>,
             pressed_icon, badge: Option<u64>, disabled, onclick }
ActionButton { model: ActionButton, now: ClockMs, icon, shortcut, on_press }
// core
oxyn_core::primitives::button::{ButtonVariant, ButtonSize, LoadingStyle, ButtonState,
    agent_bound(&AgentState) -> ButtonState, press_scale, ActionButton, ActionPhase, HoldConfirm}
```

## 10. Related components and references
`composer-send-control.md` (the send button's own state machine), `button-groups.md`,
`tooltip-popover.md` (labels and reasons), `spinner` in `display-atoms.md`. Sources: affectUI
`/components/button`, `/components/icon-button`; Arc `/components/button`, `/components/action-button`
(behaviour only); Beautiful UI `button` (MIT, © 2026 Shane Levine — variant names ported).
