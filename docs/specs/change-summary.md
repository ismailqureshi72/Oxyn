**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Change summary (`change-summary`)

Family: Diff & review. Oxyn plus for plan #30 (header of the diff viewer as its own part, used in chat replies and PR cards). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
After an agent edits files, the reply needs a one-line receipt: "4 files changed +58 −21", a five-block bar (GitHub style: green, red, neutral, `stat_blocks`) and, unfolded, a row per file with its status letter (A M D R), path, counts and blocks. A row opens that file in the viewer.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| open | Headline and rows | toggle |
| folded | Headline only, chevron turned | toggle |

## 3. Timing and motion
- Rows fade down over 200 ms when unfolded; chevron turns in 200 ms.
- Reduced motion: instant.

## 4. Variants
| Variant | What differs |
|---|---|
| expanded | Rows shown |
| compact | Starts folded for chat messages |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space | Fold / unfold; on a row, open the file |

## 6. Accessibility
- Headline is a button with aria-expanded; letters carry the word as a title; the bar is aria-hidden; counts are text.

## 7. Responsive
- Max 560 px; paths ellipsize from the end.

## 8. Edge cases
- A pure rename shows R and 0.
- Zero changes give a neutral bar.
- One-sided changes never round the other colour away (at least one block each).

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| files | Vec<FileDiff> | — |  |
| compact | bool | false |  |
| blocks | usize | 5 | bar length |
| on_open | EventHandler<String> | — | path |

## 10. Gaps against the source
- No sort control; files keep first-touch order.

## 11. GPUI
`ChangeSummaryEl` with `open(bool)` and a `toggle` act.
