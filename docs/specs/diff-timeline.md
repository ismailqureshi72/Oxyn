**Status:** built v0.2 (2026-11-16: run view added for plan #32, affectUI fidelity pass family B part 9, against the ref-3 capture notes; v0.1 2026-11-15, plan #30 – the revision rail below).

# Spec: Diff timeline (`diff-timeline`)

Family: Diff & review. Plan #30 "diff timeline"; Conductor filter-by-commit idea (diff-viewer.md `commit-filter`). Behaviour-only rebuild; nothing is copied.

## 0. Built: the run view (`DiffTimelineRun` · `DiffTimelineRunEl`, model `oxyn_core::kit::difftlfid`)
Closes the parity row affectUI · Diff Timeline (Pro). Behaviour-only rebuild; the run, file names, labels and copy are our own. The revision rail (sections 1–11) stays as the `view = revisions` form of the same page.

| Part | Look (Clean, light / dark) |
|---|---|
| Root | card 440 px max: white / #17171A, 1 px hairline, 12 px radius, soft shadow, 14 × 16 px padding |
| Header | the agent's 5 × 5 pixel mark (from its id), name 13/600 + muted "· context"; status line under it ("Editing app/billing/page.tsx" with a blue breathing dot during the run; at the end the count of edits and the run time); right: "+n" green chip, "−n" red chip, muted "across n files" |
| Chart | 150 px plot, y ticks on a 1-2-5 ladder, x ticks per minute up to the run rounded up; added line green over a soft green area, removed red over a fainter one, net dashed grey; 1.5 px non-scaling strokes; dots at the line ends that breathe out a ring while live; one tick per edit on the baseline |
| Files | one 22 px row per file in first-touch order: file glyph, muted folder + bold name, a bar of this file's edits in time order (green / red pieces; hatched for binary), "+n −n" or "binary"; the row being edited has a grey wash; seven rows then the list scrolls |
| Popover | an edit opened from the chart: file + m:ss, Added / Removed running totals with this edit's (+n) / (−n), "Changed lines" in mono (green / red), or "Binary file changed"; other rows fade to 38 % |
| Legend | "Added / Removed / Net +n" under the list |
| Classic | a grey tray with the chart and the files as two white panels; legend on the tray |

| State | Shown |
|---|---|
| live (default demo) | replays and loops; a row fades up from a blur when its file is first edited, its bar widens piece by piece and the header counts rise |
| early / mid / done | fixed moments of the same run |
| open | an edit popover open, other rows dimmed |
| narrow | under about 410 px: totals under the name, folders hidden, counts shortened (4.8k) |

Keyboard: Tab focuses the chart (one stop); ← / → step edits and open them; Home jumps to the first edit and End to the last; Escape shuts the popover. Hovering a file row lifts its ticks on the baseline.
Accessibility: the chart is one group labelled with its edit count; the popover is a polite live region; the status line is a status region; each row's accessible name gives path, counts and edit count. Reduced motion stops every animation.
Props: `edits: Option<Vec<DtEdit>>`, `at: Option<u32>` (ms; `None` replays the demo), `agent_id`, `agent`, `context`, `max_rows` (7), `open: Option<usize>`, `now`, `style`, `on_open: EventHandler<Option<usize>>`.
GPUI: `DiffTimelineRunEl::default().edits(e, at).agent(name, ctx).classic(b).theme(t).on_act(..)` – a static snapshot (cumulative columns per edit, file rows with bars; no popover or motion).
Gaps: the source's per-agent icon set and its exact easing curves are not copied; the GPUI chart is columns, not lines; there is no "Reset preview" inside the component (the gallery's own reset reloads it).

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
