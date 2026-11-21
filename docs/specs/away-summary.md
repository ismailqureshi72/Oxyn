**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 6).

# Spec: Return brief (`away-summary`)

Family: Agents. Closes the parity row: affectUI · Away Summary (Pro). Behaviour-only rebuild; nothing is copied (the
thread, tasks, details and counts in the demo are our own). The full overnight view is Overnight recap.
Components: `AwaySummary` (Dioxus) · `AwaySummaryEl` (GPUI), model in `oxyn_core::kit::awayfid`.

## 1. Purpose
The "While you were away" card for a chat thread, right where the person left off: how long they were gone, what needs
them, what failed and what's done as a short list with one action per row; Got it folds it into a one-line divider.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 600 px (max 100 %), container-sized, 12 px radius (continuous where supported), hairline, faint shadow |
| Header | 40 px: "While you were away" 13.5/600 + " · 2h 14m" muted; right: dot + "2 need you" · "1 failed" · "5 done" (amber, red, blue dots) |
| Row | 48 px: 13 px mark (amber ring with a breathing centre, red disc with !, blue disc with a drawn tick); title 13.5/500 and a 12 px muted detail; one action – Decide raised white, Retry / Open quiet |
| Footer | 38 px: "Show 3 more" + chevron (Show less when open); "✓ Got it" |
| Divider | folded: hairline · amber dot · "You were away 2h 14m · 8 updates" · chevron · hairline |

## 3. Behaviour
- Rows are ordered once (needs you, failed, done) and stay put: an action resolves its row in place (`resolved` line,
  kind done, Open), the mark pops, the tick draws, the row flashes and the counts roll. `work_ms` keeps the button
  working (spinner, aria-disabled) before it settles.
- Show N more grows the rest in and focuses the first new action; Got it (`on_dismiss`) or Escape folds the card
  (`on_collapsed_change`) and focuses the divider; the divider opens it again and returns focus to the last action.
- `since_clock` makes the head read "since 9:40"; `default_collapsed` starts folded.
- prefers-reduced-motion: everything simply appears.

## 4. Variants
default · resolved · expanded · folded (since a time, folded at first) · slow-retry · narrow (380 px) · classic (grey
header band, stronger border and shadow, 16 px radius).

## 5. Keyboard and accessibility
Tab through the row actions, then Show more and Got it; Enter / Space runs them; Escape folds. A region named by its
title; rows are a list and say their state in words; actions name their task ("Decide: Export job"); Got it and the
divider carry aria-expanded; a polite live region announces each change ("Export job: done. 1 needs you").

## 6. Responsive
Up to 600 px; rows keep one height. Under 460 px the counts drop under the title; titles, details and long action
labels end in an ellipsis (full text in a tooltip); touch buttons are 36 px tall.

## 7. Known gaps
- Folding swaps the card for the divider with a short fade, not a height slide; the folded card is not kept in the DOM.
- The duration is a prop (`away`), not ticked from a clock every half minute.
- GPUI: a static snapshot – the host keeps resolved / expanded / folded; no breathing mark or rolling counts.
