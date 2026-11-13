**Status:** v1 — implemented (plan #23).

# Spec: number ticker and reaction bar

> Implemented in `oxyn-core::primitives::ticker` (`NumberTicker`, `Column`, `NumberFormat`,
> `Locale`, `format`, `compact`, `TickerMode`, `Reaction`, `CountRule`), `oxyn-dioxus::{NumberTicker,
> ReactionBar}`, `oxyn-gpui::{NumberTickerEl, ReactionBarEl}`. Sources: affectUI Number Ticker and
> Reaction Bar, Arc Reactions (behaviour only).

---

## A. Number ticker

### 1. Purpose
Numbers that change while you watch — tokens used, cost so far, runs completed, a KPI. Rolling
digits show *that* and *which way* it changed without pulling the eye like a flash.

### 2. State machine
`Resting(v) ─set(v')─▶ Rolling(from, to, at) ─duration─▶ Resting(v')`; a new value mid-roll starts
from the current interpolated value. `enter_view()` the first time: `Resting(v) → Rolling(0, v)`
(count-in), never again. Reduced motion: `set` jumps straight to `Resting(v')`.

### 3. States table
| Column | Shows |
|---|---|
| `Digit { pos, up }` | a 0–9 strip at `pos` (fractional while moving); up when the value grows, down when it shrinks |
| `Enter { k }` | a new column (9,999 → 10,000) growing in width and opacity, sliding in |
| `Exit { k }` | a column leaving |
| `Static` | separators, sign, currency, `%`, units (crossfade when they change) |

The accessible name is always the final formatted value.

### 4. Micro-interactions
Each digit rolls through the intermediate digits in its direction (9 → 0 → 1 rolls through the
top). Columns stagger right to left by 30 ms so the change reads as one wave. Count-up mode (Oxyn
plus) shows whole intermediate values instead of a roll.

### 5. Timing table
| Item | Value | Notes |
|---|---|---|
| Roll | 900 ms (`duration`) | affectUI default |
| Delay | 0 (`delay`) | |
| Column stagger | 30 ms (`stagger.digit`) | right → left |
| Easing | `ease.out` (Oxyn plus: any token curve) | |
| Count-in | once, on first view | |
| **Reduced motion** | final value at once | |

### 6. Variants
Source: `value`, `format`, `locale`, `duration`, `delay`. **Oxyn plus:** roll vs count-up mode,
easing choice, compact formatting (`1.2k`, `3.4M`, locale decimal mark), currency and percent
formats, `Plain` / `En` / `De` / `Fr` grouping.

### 7. Edge cases
Negative numbers keep the sign as a static column. Decimal precision is fixed by the format, so the
width never jitters (tabular figures). Rapid updates retarget smoothly from the current value.
Huge jumps (0 → 1,000,000) still finish in `duration`. Values that format identically don't animate.

### 8. Accessibility
Columns are `aria-hidden`; the container is named with the final text, so screen readers never hear
in-between digits. Live regions belong to the host (not every tick should be announced).

### 9. Props
`NumberTicker { ticker: NumberTicker { format, locale, duration, delay, mode, easing }, now, reduced, size }`

---

## B. Reaction bar

### 1. Purpose
Like, comment, repost and share in one pill under a message or post; in agent apps also thumbs up
/ down feedback on an answer.

### 2. State machine
Per reaction: toggles `Off ⇄ On` (count ±1); one-shot actions (comment, share) fire without a
state. Quick taps within the pop window stack the kick (Arc).

### 3. States table
| State | Shows | ARIA |
|---|---|---|
| rest | icon + count (counts hidden at 0, abbreviated ≥ 10,000) | `aria-label` "Like, 1,203" |
| hover | soft surface, full-colour text | — |
| on | icon fills with its tint — monochrome `fg.default` by default (chrome rule); a host may pass a status tint where the reaction *is* a status (thumbs-down → danger) | `aria-pressed=true`, ", on" |
| pressed | icon shrinks, springs past 1, settles | — |
| compact | icons only | names unchanged |

### 4. Micro-interactions
The icon shrinks to 85 %, springs to ~118 % and settles in 420 ms; stacked quick taps kick up to
35 % harder. The count's width glides and changed digits roll in the direction of the change.

### 5. Timing table
| Item | Value | Reduced motion |
|---|---|---|
| Pop | 420 ms | none |
| Digit roll | 280 ms (`dur.digit.roll`) | instant |
| Hover | 120 ms | 80 ms |

### 6. Variants
Source: `reactions`, `actions`, `onReact`. **Oxyn plus:** custom reaction sets (any icon, tint and
toggling flag), compact icon-only bar, count rules (`CountRule { from, hide_zero, locale }`).

### 7. Edge cases
Counts never go below zero. Large counts abbreviate (`12k`) but the accessible name keeps the rule's
text. Many reactions wrap inside the pill's parent.

### 8. Accessibility
Each reaction is a button with `aria-pressed` (toggles only) and a full name including the count.
Tint colour is paired with the filled icon and the "on" word.

### 9. Props
`ReactionBar { reactions: Vec<Reaction>, compact, rule: CountRule, now, on_react }`

## 10. Related components and references
`composer-context-meter.md` and `run-meter.md` (numbers that tick), `message-bubble.md` (feedback).
affectUI `/motion/number-ticker`, `/components/reaction-bar`; Arc `/components/reactions` —
behaviour only.
