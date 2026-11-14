**Status:** v1 — implemented (plan #26).

# Spec: shortcut recorder

> Implemented in `oxyn-core::primitives::search` (`Recorder`, `RecState`, `Binding`, `Mods`, `canonical`, `normalize`, `filter_cheatsheet`, `dimmed`), `oxyn-dioxus::ShortcutRecorder` + `ShortcutList`, `oxyn-gpui::ShortcutRecorderEl`.
> Source: Arc UI Shortcut recorder (behaviour only, no source used).

## 1. Purpose
Let people rebind keys: pick a combination, see it as caps, and be warned before taking one the browser or another command already uses.

## 2. State machine
```
Idle ─Enter / Space / click─▶ Recording(held = ∅)
Recording ─modifier down─▶ Recording(held caps shown)
Recording ─key without ⌘ / Ctrl / ⌥ (not F1–F24)─▶ Invalid("Use ⌘, Ctrl or ⌥ with another key")
Recording ─reserved combo─▶ Conflict(reserved) ; ─bound combo─▶ Conflict(binding)
Conflict ─Use anyway─▶ Idle(value, binding replaced) ; ─Cancel─▶ Idle(previous)
Recording ─Esc─▶ Idle ; ─Backspace─▶ cleared ; ─Tab─▶ Idle, focus moves
sequences (Oxyn plus): bare key ─▶ waiting for second key ─▶ "G then I"
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle, set | caps, Clear, Reset | Enter records, Backspace clears | `button`, `aria-pressed=false`, spoken value |
| idle, empty | placeholder | — | "<label>, not set" |
| recording | red dot, held caps | any combination | `aria-pressed=true` |
| conflict | warning line with Use anyway / Cancel | Tab to buttons | announced |
| invalid | reason line | — | announced |

## 4. Micro-interactions
Caps pop in one after another; held modifiers light as they're pressed; the message line opens with a height transition; the recording dot pulses.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Caps in | 180 ms + 40 ms stagger | `ease.out` | none |
| Message | 260 ms | `ease.out` | instant |
| Dot pulse | 1 s loop | ease-in-out | static |

## 6. Variants
Free: mac and other glyphs, require-modifier, reserved warnings, conflicts with Use anyway, clear, sizes.

**Oxyn plus:** two-key sequences, Reset to default, a searchable cheatsheet that lights held keys and fades rows that don't use them, JSON keymap export.

## 7. Edge cases
`mod` is ⌘ on Mac and Ctrl elsewhere; shortcuts written in any order normalise before comparing. Function keys are allowed alone. Recording blurs back to idle.

## 8. Accessibility
The recorder is a toggle button named by its label and the spoken shortcut ("Command Shift P"). Results, conflicts and cancels are announced. Conflict choices are real buttons.

## 9. Props sketch
```rust
ShortcutRecorder { label, hide_label, value: Option<String>, reset_value, bindings: Vec<Binding>,
                   warn_reserved, require_modifier, platform: "mac"|"other", placeholder, description,
                   disabled, size, sequences, recording, on_change: EventHandler<(Option<String>, Option<String>)> }
ShortcutList { groups: Vec<ShortcutGroup>, label, searchable, highlight_pressed, platform, width }
```

## 10. Related components and references
`display-atoms.md` (Kbd), `command-palette.md`. Arc UI `/docs/components/shortcut-recorder` — behaviour only.
