**Status:** v1 — implemented (plan #26).

# Spec: settings command

> Implemented in `oxyn-core::primitives::command` (`SettingsCommand`, `Setting`, `SettingKind`, `SettingValue`, `SavePhase`), `oxyn-dioxus::SettingsCommand`.
> Source: Arc UI Settings command block (behaviour only, no source used).

## 1. Purpose
Workspace settings people search more than they browse: "where do I turn off the digest?" The answer should be one keystroke away, and edits should be safe to undo before they're saved.

## 2. State machine
```
Page ─⌘K / Ctrl+K / / / search field─▶ Palette(query) ─↑ ↓ Enter─▶ Page, scrolled to the setting, row breathes twice, control focused
Page ─edit─▶ Dirty(n changes) — bar "n unsaved changes" + Discard / Save
Dirty ─Save, invalid─▶ jumps to the first error
Dirty ─Save─▶ Saving ─ok─▶ Saved (1.6 s) ─▶ Clean ; ─fail─▶ Failed (Retry)
Dirty ─Discard─▶ Clean
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| page | section nav, sections with rows | Tab | labelled controls |
| palette | search over labels, descriptions, keywords, sections | ↑ ↓ Enter Esc | `dialog` + combobox |
| dirty | edited dots, bar with count | — | polite status |
| saving / saved / failed | spinner, check, Retry | — | announced |

## 4. Micro-interactions
The search field grows into the palette. Jumping scrolls smoothly and the target row breathes twice. The unsaved bar pops in; its label rolls as the count changes.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Palette open | 260 ms | `ease.out` | 120 ms fade |
| Row breathe | 1.6 s, 2 pulses | `ease.out` | one 800 ms pulse |
| Bar in | 280 ms | `ease.out` | fade |
| Saved | 1.6 s visible | — | same |

## 6. Variants
Free: switch, segmented, select and text settings; section nav; suggested settings.

**Oxyn plus:** validation that jumps to the first error, simulated save with failure + Retry for demos, per-section edited dots.

## 7. Edge cases
Text settings validate length and required. Saving with nothing changed is impossible (no bar). Very long setting lists scroll; the palette shows at most eight results.

## 8. Accessibility
Each control is labelled by its setting and described by its hint (or error, `aria-invalid`). Edited rows add hidden "Edited" text, not just a dot. Save results are announced.

## 9. Props sketch
```rust
SettingsCommand { sections: Vec<(id, label, description)>, settings: Vec<Setting>,
                  saved: BTreeMap<String, SettingValue>, suggested, title, save_ms, fail_first,
                  open_query: Option<String>, width, on_save: EventHandler<Vec<String>> }
```

## 10. Related components and references
`command-palette.md`, `toggles.md`, `select.md`. Arc UI blocks `/docs/blocks/settings-command` — behaviour only. GPUI renderer is a known gap.
