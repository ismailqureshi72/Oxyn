**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Context window (cells) (`context-field`)

Family: Meters ([composer-context-meter.md](composer-context-meter.md)). Closes the parity row(s): affectUI · Context Window. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
What fills the model's context as a field of cells (216 in 36 columns by default). Parts (`Part { id, label, tokens, items, locked, compact }`) fill the field in order with cumulative rounding so every non-empty part gets at least one cell. Pointing at a part lights its cells and quiets the rest. Parts with items open to list them with × to remove (locked parts cannot). Compact shrinks a part to its compact share: the freed cells shimmer for 900 ms in a left-to-right wave and clear. From `warn_at` (0.8) the percentage turns amber and the tray says how much is left; from 95 % the window counts as full. An optional tray action ("New chat").

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| ok | Grey field, tokens of limit | — |
| warn | Amber percent, "38K left" | — |
| full | Red, "Full · compact or start a new chat" | — |
| compacting | Freed cells shimmer and clear | 900 ms |

## 3. Timing and motion
- Cell clear 900 ms with 8 ms per column; part rows fade 180 ms.
- Reduced motion clears cells with a 200 ms fade.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Full demo |
| warn | Past 80 % |
| light | Early chat |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Part toggles, Compact, item removes, tray action |

## 6. Accessibility
- The field is role=img with tokens in use; the tray line is role=status.

## 7. Responsive
- Cells keep their aspect ratio.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| title | String | Context window |  |
| model | Option<String> | — | tag |
| parts | Vec<Part> | — |  |
| limit | u64 | — |  |
| warn_at | f64 | 0.8 |  |
| cells / columns | usize | 216 / 36 |  |
| action | Option<String> | — |  |
| on_remove / on_compact / on_action | EventHandler | — |  |

## 9. Gaps against the source
- A model logo next to the tag is not offered (text tag only).

## 10. GPUI
ContextFieldEl draws the field from `CellField::cells` with part highlight.
