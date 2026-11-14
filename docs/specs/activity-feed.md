**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Activity feed and timeline (`activity-feed`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Activity feed (block); Arc UI · Timeline.

## 1. Purpose
What happened, newest first, grouped by day (Today, Yesterday, n days ago) on a quiet avatar line. The feed adds a type filter, unread dots and a count, new events held behind a “n new updates” button so nothing jumps, expandable details (expanding marks read), Mark all read, and a range (today, week, month). The timeline variant keeps only the line, markers and details.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| default | Groups and rows |  |
| new waiting | Button above the list | press → events slide in |
| expanded | Detail below the row |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| feed | Filter, unread, new |
| timeline | Quiet line |
| today | Range 1 day |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Filter, new updates, events |
| Enter / Space | Expand (marks read) |

## 6. Accessibility
- Day headings; events are buttons with aria-expanded; filter is a radiogroup.

## 7. Responsive
- Targets truncate; times stay right.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| feed | Feed | — | events, incoming, filter, range, expanded |
| variant | feed · timeline | feed |  |
| title | String | Activity |  |
| on_filter | EventHandler<Option<EventKind>> | — |  |
| on_toggle | EventHandler<String> | — |  |
| on_show_new/on_read_all | EventHandler<()> | — |  |

## 9. Edge cases
- Events without detail are not expandable.

## 10. Implementation
- Core: `Feed`, `FeedEvent`, `EventKind` (`oxyn_core::parity`), with tests.
- Dioxus: `ActivityFeed` (parity/run.rs, run.css).
- GPUI: `ActivityFeedEl` (`show-new`, `read-all`).

## 11. References
- Arc UI Activity feed (https://uiarc.dev/components/blocks/activity-feed) — behaviour-only rebuild, no source copied. Also Arc UI Timeline (https://uiarc.dev/components/timeline).
