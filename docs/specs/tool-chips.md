**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Tool chips (`tool-chips`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Beautiful UI · Tool Chips.

## 1. Purpose
Code edits and tool calls as compact chips: a header “4 tool calls, 2 messages” that folds; each call as a row with its kind icon, a verb and an inline chip (mono for files and commands), a live state per chip (spinner, failed, needs approval) and details that open per row; then file-diff chips (+18 −4) whose diff previews on hover or focus and pins on click. Density rows, chips (chips only, wrapped) or cards.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| open | Rows + diff chips | header → folded |
| folded | Header only |  |
| peek / pinned | Diff preview | leave / click again |

## 3. Timing and motion
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| rows | Default |
| chips | Chips only |
| cards | Cards |
| live | A run in progress |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Header, calls, diff chips |
| Enter / Space | Open a call; pin a diff |

## 6. Accessibility
- Diff chips are toggle buttons (aria-pressed, aria-expanded) and the preview is a labelled region.

## 7. Responsive
- Chips wrap; the preview scrolls sideways.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| steps | Vec<ChipStep> | — | tool, label, chip, mono, status, detail |
| diffs | Vec<DiffChip> | — | file, add, del, lines |
| messages | usize | 0 |  |
| density | rows · chips · cards | rows |  |
| open | bool | true |  |

## 9. Edge cases
- Hover and focus both preview; a pinned diff wins over a peek.

## 10. Implementation
- Core: kind icons from `ToolKindMap` (`oxyn_core::parity::tools`), with tests.
- Dioxus: `ToolChips` (parity/tools.rs, tools.css).
- GPUI: `ToolChipsEl`.

## 11. References
- Beautiful UI Tool Chips (https://www.beautifului.dev/#tool-chips), MIT, © 2026 Shane Levine — behaviour rebuilt in Rust; no source or icons copied.
