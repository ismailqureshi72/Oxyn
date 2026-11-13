**Status:** v1 — implemented (plan #23).

# Spec: kbd, separator, spinner, empty state

> Implemented in `oxyn-core::primitives::display` (`parse_shortcut`, `aria_keyshortcuts`,
> `Platform`, `Separator`, `Spinner`, `SpinnerStyle`, `EmptyState`, `EmptyLayout`),
> `oxyn-dioxus::{Kbd, Separator, Spinner, EmptyState}`, `oxyn-gpui::{KbdEl, SeparatorEl, SpinnerEl,
> EmptyStateEl}`. Sources: affectUI Spinner and Empty State, Arc Empty state (behaviour only).

---

## A. Kbd

### 1. Purpose
Shows a shortcut so people learn it: in tooltips, menu rows, buttons and docs.

### 2. State machine
Stateless, plus `pressed` (the host sets it while the keys are held, e.g. a cheat sheet that lights
up).

### 3. States table
| State | Shows | ARIA |
|---|---|---|
| keycap (default) | each key as a cap with a 1.5 px bottom edge | `aria-label` "Command Shift K", `aria-keyshortcuts` "Meta+Shift+K" |
| flat | caps without the edge, hover fill | same |
| inline | glyphs only, muted (inside prose and menu rows) | same |
| sequence ("G I") | caps with "then" between chords | "G, then I" |
| pressed | caps sink 1 px and fill | — |

### 4–5. Micro-interactions and timing
Pressed: 80 ms translate + fill (`dur.instant`). Reduced motion: fill only.

### 6. Variants
Oxyn plus (no source item): platform-aware glyphs (`Mod` = ⌘ on Mac, Ctrl elsewhere; ⌥ / Alt;
⇧ / Shift), three styles, sequences, pressed state, `aria-keyshortcuts` generation.

### 7. Edge cases
Unknown keys are upper-cased as-is; arrows, Enter, Esc, Tab, Backspace, Delete, Space have glyphs.
Mixed-case input ("mod+k") works. Very long chords wrap between caps, never inside one.

### 8. Accessibility
Caps are `aria-hidden`; the container speaks the key names. Buttons with shortcuts carry the same
`aria-keyshortcuts` value.

### 9. Props
`Kbd { keys: "Mod+Shift+K", other: bool, style: "keycap|flat|inline", pressed }`

---

## B. Separator

### 1. Purpose
Groups content: menu sections, thread days ("Today"), toolbar groups, settings blocks.

### 2–3. States
| Variant | Shows | ARIA |
|---|---|---|
| horizontal / vertical | 1 px `border.subtle` | decorative: `role=none`; semantic: `role=separator` + `aria-orientation` |
| labelled | centred label with lines either side | `role=separator` |
| dashed | 4 px dashes | — |
| fade | line fades to transparent at both ends | — |
| inset | starts after an icon column (`inset` px) | — |

### 4–5. Motion
None (static by design).

### 6. Variants
Oxyn plus: label, dashed, fade, inset; semantic vs decorative.

### 7. Edge cases
Vertical separators need a stretch parent; they keep a 16 px minimum height. Labels never wrap.

### 8. Accessibility
Decorative by default so screen readers skip it; set `decorative=false` where it separates real
groups.

### 9. Props
`Separator { vertical, decorative, label, dashed, fade, inset }`

---

## C. Spinner

### 1. Purpose
A small turning mark for work running right now — in a button, a status row, a tool row. The big
loaders (`loaders.md`) are for waits; the spinner is for "this one thing is busy".

### 2. State machine
Running only. It appears when work starts and is removed when it ends (hosts use
`dur.loader.show` 300 ms before showing it for short work — `loaders.md`).

### 3. States table
| Style | Shows |
|---|---|
| arc (source) | 2 px arc on a ring at 20 % of the colour, 25 % arc length |
| dots | 8 dots fading round |
| ray | 12 rays fading round (system style) |
| orbit | one dot orbiting a faint ring |

### 4. Micro-interactions
One turn every 750 ms, linear (affectUI). Colour is `currentColor`, so it takes the text colour of a
button or row.

### 5. Timing table
| Item | Value | Reduced motion |
|---|---|---|
| Turn | 750 ms linear (`speed_ms`) | 2400 ms per turn (slow, still readable as busy) |
| Segment fade | one period | same period |
| Stroke | 2 px up to 24 px, then size / 12 | — |

### 6. Variants
Source: `size` (14), `label`. **Oxyn plus:** arc length, speed, four styles, sizes 12–48.

### 7. Edge cases
Inside buttons it replaces the leading icon at the icon's size. Many spinners on one screen share
the motion clock (and its caps when the window is unfocused).

### 8. Accessibility
Announced only when it has a label (`role=status` + `aria-label`); otherwise `aria-hidden` so a row
of tool spinners doesn't chatter.

### 9. Props
`Spinner { size, label, style: "arc|dots|ray|orbit", speed_ms, arc }`

---

## D. Empty state

### 1. Purpose
What a list, table or panel shows while it has nothing yet — and the first step to change that.

### 2. State machine
Stateless; the host swaps it for content. Arc: changing title or description rolls the copy and the
block height springs; a new icon pops in with a short blur.

### 3. States table
| Layout | Shows | ARIA |
|---|---|---|
| framed (default) | dashed frame, icon on a soft tile, title, one line, primary + secondary actions | `role=status` |
| bare | same without the frame | `role=status` |
| tall | 64 px padding for page-level empties | — |
| inline (Oxyn plus) | one line inside a table row or list: icon, title, text, ghost action | `role=status` |
| with prompts (Oxyn plus) | starter prompt chips under the actions | chips are buttons |

### 4–5. Motion
Copy change: 180 ms fade + height `spring.snap` (Arc). Icon change: 180 ms pop from a 3 px blur.
Reduced motion: 80 ms fade, height jumps.

### 6. Variants
Source: `title`, `text`, `action`, `icon` (file), `bare`, `tall`. **Oxyn plus:** inline row layout,
secondary action / docs link, starter prompts for agent apps ("Summarise this repo").

### 7. Edge cases
Text wraps at 340 px; titles stay one line where possible. Without an action the block shrinks.
In a table, use `inline` inside a full-width cell.

### 8. Accessibility
`role=status` so a list emptying (e.g. after a filter) is announced; actions are real buttons.

### 9. Props
`EmptyState { state: EmptyState { title, text, action, secondary, icon, layout, tall, prompts }, on_action, on_secondary, on_prompt }`

## 10. Related components and references
`loaders.md` / `loader-vector.md` (bigger waits), `tooltip-popover.md` and `dropdown-menu.md`
(kbd in context), `messages.md` (thread empties). affectUI `/components/spinner`,
`/components/empty-state`; Arc `/components/empty-state` — behaviour only.
