**Status:** v1 (plan #18, 2026-11-12): 9 free variants in `oxyn-core::composer::input` (rows, paste chips, limits, history, placeholders). Dioxus all 9; GPUI rounded-box.

# Spec: composer input shell (text area + container)

Follows the 10-point standard in [README.md](README.md) at the depth of the worked example
[composer-send-control.md](composer-send-control.md). Target **9 free + 16 Pro variants**, all driven by
one state machine. The send control, tools menu, attachments and trigger popovers are separate specs;
this one owns the box they live in, the text inside it, and the rules that keep it still.

## 1. Purpose
The shell is where the user spends most of their time in an agent app: writing the first prompt,
pasting a stack trace, dropping a screenshot, steering a run that is going the wrong way, answering a
question card. It has to feel like plain text editing (no surprises, no lost drafts) while quietly
telling the user what the agent is doing and what typing will do *right now*. The placeholder is its
loudest teaching surface: it shows the syntax when idle ("@ files, $ skills, / commands" [R003]) and
changes meaning when the agent is busy ("Guide this conversation" [R007]).

## 2. State machine

```
                focus                 type                   Enter (busy_action=Send)
   Idle ───────────────▶ Focused ───────────▶ Composing ────────────────────────▶ Submitting ──▶ Idle
    ▲  ◀──── blur ───────  │  ▲                  │   │  ▲                           (text flies up)
    │                      │  └── clear ─────────┘   │  │
    │          ↑ on empty  ▼                         │  └─ commit chip ── Pasting(large) ◀── paste > threshold
    │               HistoryRecall(i) ── edit ──▶ Composing
    │                                                │
    │      drag enters ▶ DragOver(Valid|Invalid) ── drop ──▶ (hand to attachments) ──▶ previous state
    │                                                │
    │                         rows hit max ▶ Scrolling (internal scroll, still Composing)
    │                         limit ≥ 80 % ▶ NearLimit ── ≥ 100 % ──▶ OverLimit (send blocked + reason)
    │
    │  AgentState busy (Submitted/Thinking/Streaming/ToolRunning/Queued/Stalled)
    └──────────────▶ Busy(EmptyDraft) ── type ──▶ Busy(Draft) ── Enter ▶ queue/steer (send-control)
                         │  AwaitingApproval ─▶ Answering(card)  (placeholder points at the card)
  Overlays (any state):  Composition(IME) · Dictating · TriggerOpen(/ @ $ #) · Disabled(reason) · Offline · SendFailed
```

Rules:
1. **The shell never changes position.** It grows upward from an anchored bottom edge; the transcript
   gives up the space [R079]. The only sanctioned move is `composer-input/hero-dock` on a thread's first
   send (one FLIP, see §5).
2. **A draft is never lost.** Blur, navigation, disable, offline, send failure and app restart keep it
   (persisted per thread, debounced 400 ms, proposed).
3. Busy states do not lock the text area. Typing while busy produces a draft whose fate the send control
   decides (`busy_action`).
4. Enter never submits while an IME composition is active, and never on touch keyboards by default
   (`submit_on_enter = Desktop`, an API shape taken from coss ui [R126]).
5. The placeholder changes with `AgentState`, but holds each value for at least 600 ms (proposed) so a
   fast agent does not make it flicker.

## 3. States table

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Idle` | 1 row (pill) or 2 rows (box); placeholder teaches syntax; send ↑ muted | focus | + menu, mic | Tab focuses; any printable key while the thread has focus jumps here (opt-in) |
| `Focused` | border → accent 1 px + 3 px ring at 24 % alpha; caret | type | paste, drop | Esc blurs (if nothing open) |
| `Composing` | text; height grows per wrapped row; send lights up | send (via send control) | newline | Enter send · Shift+Enter newline · ⌘/Ctrl+Z local undo |
| `Scrolling` | at max rows: internal scroll, 12 px fade mask on the clipped edge | — | — | ⌘/Ctrl+↑/↓ jump to top/bottom of draft |
| `HistoryRecall(i)` | previous sent prompt, dim "History 3/12" hint at right of the hint row | send / edit | ↓ forward, restores stashed draft at the end | ↑ / ↓ when caret is on first / last line |
| `Pasting(large)` | inline "Pasted text · 142 lines" chip replaces the raw text | keep as chip | Expand into raw text | ⌘/Ctrl+Z turns the chip back into raw text |
| `DragOver(Valid)` | dashed 1.5 px accent outline, 6 % accent tint, label "Drop to attach · 3 files" | drop | — | Esc cancels drag highlight |
| `DragOver(Invalid)` | dashed danger outline, label "Can't attach .exe files" | — | — | — |
| `NearLimit` | counter "1.8k / 2k tokens" (tabular digits) in the hint row | send | Convert to attachment | — |
| `OverLimit` | counter in danger tone; send blocked; reason "212 tokens over · attach as file instead" | Convert to attachment | trim | Enter does nothing; reason announced |
| `Busy(EmptyDraft)` | placeholder "Steer the agent…" (see table below); send shows ■ | stop (send control) | — | Esc stops (send control) |
| `Busy(Draft)` | draft text; hint row "Enter to queue · ⌘↵ to steer" | queue or steer (per `busy_action`) | stop | Enter / ⌘↵ |
| `Answering(card)` | placeholder "Reply to the question above…"; card owns Enter | submit answer | type a free-text answer | Enter answers; ↑ moves focus into the card |
| `Composition(IME)` | underlined pre-edit text; no trigger popovers | — | — | Enter commits composition, never sends |
| `Dictating` | text area becomes a waveform strip with live transcript above it [R021] | accept (✓) | cancel | Enter accept · Esc cancel |
| `TriggerOpen` | popover above the caret (see composer-slash-mentions.md) | pick | dismiss | owned by the popover |
| `Disabled(reason)` | text area still readable and focusable; reason line under it; send muted | the reason's action ("Sign in", "Upgrade", "Retry connection") | — | focusable, `aria-disabled`; typing blocked |
| `Offline` | editable; send muted with "You're offline · your draft is saved" | — | — | typing allowed, Enter does nothing |
| `SendFailed` | draft restored in place, 2 px danger underline on the hint row, "Couldn't send · Retry" | retry | edit | Enter retries |

**Placeholder by `AgentState`** (copy is proposed and written in-house; the pattern is from [R003],
[R007], [R021] and Google Antigravity's "Ask anything, @ to mention, / for actions"):

| AgentState | Placeholder (default) |
|---|---|
| `Idle`, `Done`, `Cancelled` | "Ask anything · @ files · $ skills · / commands" (only the triggers the app registered) |
| `Composing` | — (text is present) |
| `Submitted`, `Thinking`, `Streaming`, `ToolRunning` | "Steer the agent…" (`busy_action=Steer`) or "Queue a follow-up…" (`Queue`) |
| `Queued { count }` | "Add to the queue (2 waiting)…" |
| `Steering` | "Steering…" (read-only for ≤ 400 ms while the note is absorbed) |
| `AwaitingApproval` | "Approve above, or type why not…" |
| `Stalled { since }` | "No reply for 40 s · type to nudge, Esc to stop" |
| `Error { kind }` | "Edit and try again" |
| Disabled / offline | the reason, e.g. "Usage limit reached · resets 14:00" |

## 4. Micro-interactions
- **Focus.** Border colour cross-fades to accent and a 3 px outer ring fades in. Pointer focus gets the
  border change only; keyboard focus adds the ring (the shell is always visibly focused either way).
- **Grow.** Each wrapped row animates height upward; the top edge moves, the bottom never does. Shrinking
  (delete, cut) animates down at the same speed. Pasting 40 lines jumps straight to max rows in one
  animation, not 40 steps.
- **Placeholder swap.** Old text fades out with a 4 px upward drift, new text fades in from 4 px below.
  Only the changing words move when both strings share a prefix ("Steer the agent…" → "Steer the
  agent (2 queued)…"), proposed.
- **Large paste → chip.** Raw text appears for one frame, then folds into the chip: the text block scales
  to the chip's box and cross-fades (the chip takes the text's place, so the eye follows it). Format and
  threshold follow Codex CLI (pastes over 1,000 characters collapse to a placeholder) and Antigravity
  (`[Pasted text #X +Y lines]`); research notes 03, 08.
- **Image paste.** No text appears; an attachment tile grows in above the text (see composer-attachments.md).
- **Drag over.** Outline dashes appear and slowly march (1 px per 60 ms) while a valid payload hovers;
  invalid payloads get a static danger outline and one 4 px shake on drop attempt.
- **History recall.** The recalled prompt slides in 6 px from above (↑) or below (↓) with a fade, so the
  direction is felt. The stashed draft returns the same way.
- **Limit.** The counter fades in at 80 %. At 100 % it turns danger tone and does a single 1.06 scale pulse;
  no repeated nagging.
- **Busy tint (optional, off by default).** When the agent is busy, the shell border takes the shared busy
  signal (one flag drives glow, thinking row and controls together, an idea from BoardUI [R095]); the
  default is a 1 px border tint, not a glow.
- **Send.** The text leaves upward and becomes the message bubble (≤ 260 ms, ease-in [R079]); the shell
  collapses to its min rows *after* the text has left, so the height change never races the fly-up.
- **Send failed.** The bubble flies back down into the shell (reverse path, 200 ms) and the hint row
  shows the error; nothing is retyped.

## 5. Timing (defaults; every variant may override)

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Focus border + ring | 120 ms | 0 | ease-out | ring alpha 0 → 24 % |
| Row grow / shrink (per change) | 120 ms | 0 | ease-out (cubic-bezier(0.22, 1, 0.36, 1)) | proposed; cap one animation per frame batch |
| Jump to max rows (large input) | 180 ms | 0 | same | single animation |
| Fade mask on scroll edge | 100 ms | 0 | linear | 12 px mask |
| Placeholder swap | 120 ms out + 160 ms in | 0 / 40 ms overlap | ease-out | hold ≥ 600 ms between swaps |
| Large paste fold into chip | 220 ms | 1 frame | spring, critically damped (stiffness 320, damping 36, proposed) | |
| Drag-over enter / leave | 120 ms / 80 ms | leave delayed 50 ms | ease-out | delay stops flicker from child drag events |
| Marching dashes | 60 ms per 1 px step | — | linear, 30 fps max | paused when window unfocused (shared clock) |
| History slide | 140 ms | 0 | ease-out | 6 px travel |
| Limit counter in / pulse | 160 ms / 200 ms | 0 | ease-out / spring | pulse once |
| Send collapse (after fly-up) | 160 ms | 260 ms | ease-in-out | waits for the send-control fly-up |
| Send-failed fly-back | 200 ms | 0 | ease-out | |
| `hero-dock` first-send move | 320 ms | 0 | spring (stiffness 260, damping 30) | the only allowed move |
| Draft persistence write | — | 400 ms debounce | — | not visual |
| **Reduced motion** | 0–80 ms opacity cross-fades only | — | linear | no height animation (snap), no marching dashes, no fly-up/back, no drift; every state, label and announcement kept |

## 6. Variants

Layout and surface are the variant axes here; colour, radius and density stay tokens
([../variants.md](../variants.md)).

### Free (9)
| Name | Idea |
|---|---|
| `composer-input/rounded-box` | Baseline: 2→8 rows desktop, 12 px radius, toolbar row under the text |
| `composer-input/pill` | 1-row capsule; actions inline; grows into a rounded box when it wraps (radius morphs 999 → 16 px) |
| `composer-input/adaptive` | Actions sit beside a one-line draft and move below it when it wraps (API shape from coss ui [R126]) |
| `composer-input/docked-bar` | Full-width bar flush with the window bottom, top border only; for side panels and mobile |
| `composer-input/floating` | Detached card 16 px above the bottom with shadow; transcript scrolls under it with a fade |
| `composer-input/hero-dock` | Centered greeting composer on an empty thread that docks to the bottom on first send |
| `composer-input/ide-inline` | Dense (18 px line, 28 px min height) for IDE panels and inline edit; mono option; toolbar collapses to icons |
| `composer-input/terminal-prompt` | TUI look: `›` glyph prompt, no border, tinted background row; shell-mode `!` recolours the glyph (pattern from Codex CLI, research notes 03) |
| `composer-input/minimal` | No chrome: underline only, for embedded "ask about this" fields |

### Pro (16)
| Name | Idea |
|---|---|
| `composer-input/node-graph` | The composer is a node on a canvas; context sources (repo, files, skills, images, model) wire into it and output agents branch out [R066] |
| `composer-input/liquid-glass` | Busy flag morphs the controls into refractive glass (idea from BoardUI [R095]) |
| `composer-input/border-light` | Constant-speed light band travels the rim while the agent works; GPU shader + static fallback [R095] |
| `composer-input/ambient` | Background reacts: typing tightens, send bursts, each thinking step morphs (prompt-reactive idea, [R097]) |
| `composer-input/effort-ignition` | Changing reasoning effort "charges" the shell with colour bands for 0.9–1.6 s and recolours the prompt glyph (Codex, MIT) |
| `composer-input/runner` | A 16 px pixel mascot patrols the top edge during a run (MonoCode Composer Runner, MIT) |
| `composer-input/stash-tab` | Esc-stash: the draft shrinks into a tab with a count and can be restored later (exit idea from [R079]) |
| `composer-input/ghost-suggest` | Grey next-prompt suggestion; Tab or → accepts, Enter does not send it (pattern seen in Claude desktop, research notes 03) |
| `composer-input/typewriter-hint` | Idle placeholder types and erases example prompts every 4 s; stops on focus |
| `composer-input/breadcrumb` | Context breadcrumb ("Main · project-sea") rides the top edge of the shell [R036] |
| `composer-input/notch` | Desktop island: a collapsed notch that expands into the shell on hotkey |
| `composer-input/drawer` | Mobile bottom sheet with snap points (peek / half / full) for long drafts |
| `composer-input/spotlight` | On focus the rest of the window dims 12 % and the shell lifts 2 px |
| `composer-input/split-steer` | While busy, the shell splits: left half "steer now", right half "queue" (two drop targets for text) |
| `composer-input/dither-edge` | Dithered pixel edge that densifies with draft length / token use |
| `composer-input/voice-morph` | The whole shell morphs into a voice orb strip while dictating, then back |

## 7. Edge cases
- **Long text.** A 10k-line paste never renders raw: it becomes a chip. A single 2,000-character word wraps
  with break-anywhere; no horizontal scroll ever. Hard cap 100k characters per draft (the limit coss ui
  uses for its structured clipboard [R126]); beyond it the paste is offered as a file attachment.
- **Max rows.** Defaults 2→8 rows on desktop and 1→4 on mobile, capped at 50 % of viewport height (the
  coss ui defaults [R126]); a 6-row cap is another seen default (Arc chat composer, research notes 12).
- **Overflow counting.** Count graphemes, not bytes (emoji, CJK, combining marks). Token estimates are
  approximate: label them "≈" and debounce 150 ms. Chips count by their content, not their label.
- **RTL.** `dir=auto` per paragraph so Arabic/Hebrew and mixed lines align correctly; the toolbar mirrors;
  the placeholder follows the UI locale; keyboard hints swap order; the ↑ send glyph does not mirror.
- **IME (CJK, Vietnamese, etc.).** No trigger detection, history recall or submit during composition.
  GPUI needs explicit IME handling (study GPUI editor code [R028]; Zed crates are per-crate licensed).
- **Mobile keyboard.** Shell docks to the visual viewport above the keyboard with no jump; safe-area inset
  respected; `enterkeyhint=send` but Enter inserts a newline by default; when the keyboard opens the
  transcript scrolls so the last message stays visible.
- **Touch.** 44 px minimum targets for every inline action; long-press on the shell opens paste/select
  menu, not a custom menu.
- **Tiny / dense.** In a 240 px IDE panel, toolbar items collapse into one "…" overflow; counter shortens
  to "92 %". At 16–20 px control sizes only `ide-inline` and `terminal-prompt` are supported.
- **Very fast agent.** If the run ends within 600 ms the busy placeholder never shows (hold rule). Draft
  typed during that window is treated as a normal new prompt.
- **Very slow agent.** `Stalled` swaps the placeholder to the nudge copy; the shell stays editable.
- **Slow network.** Send failure restores the draft (fly-back); attachments keep their state.
- **History.** Recall is per thread (fallback to global, configurable). Editing a recalled prompt does not
  rewrite history. When queued messages exist, ↑ edits the last queued message first (send-control rule).
- **Paste of rich HTML.** Converted to Markdown-lite (headings, lists, code fences); "Paste as plain text"
  is ⌘/Ctrl+Shift+V.
- **Secrets.** Optional detector flags pasted strings that look like keys and offers "Store as secret".

## 8. Accessibility
- Role: multi-line text box labelled "Message the agent" (or the app's agent name). The placeholder is
  never the label.
- `aria-describedby` points at the hint row (keyboard hint, counter, disabled reason). The reason is real
  text, not only a tooltip.
- Polite live region announces: busy changes ("Agent is working. Enter will queue your message."), large
  paste ("Pasted 142 lines as an attachment chip. Press Control Z to expand."), limit at 90 % and 100 %,
  send failure. Assertive only for send failure.
- Contrast: text 7:1 target; placeholder ≥ 4.5:1 because it teaches syntax; focus ring ≥ 3:1 against both
  the shell and the page.
- Focus order: attachment row → text area → tools (+) → mode/model chips → send. Focus returns to the text
  area after any popover, menu or card closes.
- Disabled-with-reason keeps the field focusable (`aria-disabled`, not `disabled`) so screen-reader users
  can hear why.
- Drag-and-drop always has a keyboard equivalent (+ → Upload file, or paste).
- Reduced motion honoured per §5; high-contrast mode replaces the ring alpha with a solid 2 px outline.

## 9. Props sketch

```rust
pub struct ComposerInput {
    pub agent: AgentState,                 // drives placeholder, busy hint, disabled reasons
    pub draft: ComposerDraft,              // structured: text + token chips + paste chips
    pub variant: InputVariant,             // RoundedBox | Pill | Adaptive | DockedBar | Floating | ...
    pub rows: RowRange,                    // { min: 2, max: 8, max_viewport_fraction: 0.5 }
    pub submit_on_enter: SubmitOnEnter,    // Always | Desktop | Never
    pub placeholders: PlaceholderMap,      // per AgentState, i18n; falls back to defaults in §3
    pub placeholder_hold_ms: u32,          // 600
    pub paste: PastePolicy,                // { chip_over_chars: 1000, chip_over_lines: 20, hard_cap: 100_000 }
    pub accept_drop: AcceptRule,           // mime types / extensions; invalid shows reason
    pub limit: Option<InputLimit>,         // Chars(n) | Tokens { max, estimator }
    pub history: HistoryScope,             // None | Thread | Global
    pub disabled: Option<DisabledReason>,  // Offline | Quota { resets_at } | Auth | Custom(String)
    pub persist_key: Option<ThreadId>,     // draft persistence
    pub surface: Surface,                  // Chat | Ide | Desktop | Mobile | Voice | Terminal
    pub labels: InputLabels,               // accessible name, hints, announcements
    pub motion: MotionLevel,               // None | Subtle | Expressive (reduced-motion token wins)
    pub on_submit: Callback<ComposerDraft>,
    pub on_drop: Callback<Vec<DroppedItem>>,  // forwarded to composer-attachments
}
```

## 10. Related components and references
**Related:** send control (composer-send-control.md) · tools menu (composer-tools-menu.md) · attachments
(composer-attachments.md) · slash / mention popovers (composer-slash-mentions.md) · queued-message stack ·
question / approval card · voice strip · context meter · mode and model chips · status tab.

**References** (study only unless marked MIT/Apache; licences per [../licensing.md](../licensing.md)):
- [R003] T3 Code placeholder "Ask anything. @ files, $ skills, / commands" (licence not checked: study only).
- [R007] Cloudy: placeholder becomes "Guide this conversation" while running; stop ≠ pause. Study only.
- [R021] Phil Pitus composer, "type / for skills" placeholder, dictation strip with ✓. Study only.
- [R036] BoardUI context breadcrumb and attachment strip. Study only.
- [R066] node-canvas composer. Study only.
- [R079] Renzo Bianchi "The exit is the confirmation": composer never moves; send fly-up ≤ 260 ms. Study only (check source).
- [R095] BoardUI one `waiting` flag drives the composer glow; rim light band. Free tier MIT, Pro paid: study only unless the item is in the MIT tier.
- [R097] Scrolltide prompt-reactive backgrounds. Paid: study only.
- [R126] coss ui Composer: min/max rows, `submitOnEnter`, adaptive layout, structured clipboard, limits. **MIT**: API shape may be ported with attribution.
- [R028] GPUI text editor (IME, scrolling) as a GPUI precedent. Study only.
- Codex CLI (large-paste placeholder, `›` prompt, shell-mode glyph, effort ignition) and MonoCode (Composer Runner): **MIT**, port with attribution (research notes 03, 08).
- Google Antigravity (placeholder copy, `[Pasted text #X +Y lines]`) and Claude desktop (ghost suggestion): proprietary, reference only (research notes 03, 08).
- Arc UI chat composer (autosize to 6 lines): paid / unclear, study only (research notes 12).
