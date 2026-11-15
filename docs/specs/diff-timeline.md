**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Diff timeline (`diff-timeline`)

Family: Diff & review. Plan #30 "diff timeline"; Conductor filter-by-commit idea (diff-viewer.md `commit-filter`). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A file was edited several times by the agent and by you. The rail lists each revision (label, who, age, +/− against the one before via `revision_stats`); picking two points shows `revision_diff` between them in an embedded diff viewer.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| latest | The last two revisions compared | pick |
| range | Any base → target, nodes between filled | pick |

## 3. Timing and motion
- Selected dots scale to 1.15 over 160 ms; the rail line fills between base and target.
- The diff below is re-keyed per range, so it swaps without stale hunks.
- Reduced motion: no scale.

## 4. Variants
| Variant | What differs |
|---|---|
| rail | Unified diff below |
| stacked | Before over after below |

## 5. Keyboard
| Key | Does |
|---|---|
| ← / → | Move the target |
| Shift + ← / → | Move the base |
| Home / End | First base / latest |

## 6. Accessibility
- Listbox with aria-activedescendant on the target; author shown by shape (square for you, round for the agent) and text.

## 7. Responsive
- The rail scrolls sideways; nodes are at least 120 px.

## 8. Edge cases
- One revision: compares it with empty.
- Shift-click past the target pushes the target forward.

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| path | String | — |  |
| revisions | Vec<Revision> | — | oldest first |
| now | ClockMs | — | ages |
| layout | String | "unified" |  |
| on_range | EventHandler<(usize, usize)> | — |  |

## 10. Gaps against the source
- No restore button; restoring a revision belongs to checkpoints.

## 11. GPUI
`DiffTimelineEl::new(path, revisions, now).range((a, b))` with an `on_pick` id callback.
