**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Agent prompt copy (`agent-prompt-copy`)

Page `/components/buttons/agent-prompt-copy/` (family buttons). Reference: an X post the founders saved (R15 ("Give to your agent" button) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AgentPromptCopy` (Dioxus, `oxyn_dioxus::xpost`, `xpost/trust.rs`) · `XPostView` with `XPostVariant::PromptCopy` (GPUI), rules in `oxyn_core::xpost::promptcopy`.

## 1. Purpose
Hand a component to a coding agent in one press: the exact prompt, skill file or install command that agent understands.

## 2. States
Ready · Menu open (agents, formats) · Copied (check and agent name, 1.8 s) · Clipboard blocked (error, preview stays selectable).

## 3. Micro-interactions
The split button shows the chosen agent mark; copying morphs it to a check with the agent name; the chevron opens a menu with radio items for agent and format; the preview updates live.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Copied state | 1800 ms (COPIED_MS) | — |
| Menu | 180 ms | ease-out |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `prompt` | A prompt for the first agent. |
| `skill` | A skill file for Cursor. |
| `button-only` | Just the button (Oxyn extra). |

## 6. Responsive and edge cases
The button drops under the title when narrow; the preview wraps long lines and scrolls past 180 px. Agent marks are the official coloured marks (Lobe Icons / Simple Icons). Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
Menu items are menuitemradio with aria-checked; ArrowDown opens, arrows move, Enter picks, Escape closes; copy results are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::PromptCopy, ..)`; presses by id (ga-copy, ga-menu, ga-close, ga-agent-<n>, ga-format-<n>), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
copy-button, code-block, command-palette.
