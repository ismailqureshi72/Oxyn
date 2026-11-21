**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-1 capture notes).

# Spec: Memory panel (`memory-panel`)

Family: Agents. Closes the parity row: affectUI · Memory Panel (Pro). Behaviour-only rebuild; layout, sizes, colours,
states, motion and keyboard follow the ref-1 capture notes (approximate); the memories and copy are our own.
Components: `MemoryPanel` (Dioxus) · `MemoryPanelEl` (GPUI), model in `oxyn_core::kit::memoryfid`.

## 1. Purpose
Show what the assistant remembers about you, grouped by category, and let you switch memory off, filter, search, edit,
pause, forget (with undo) and add memories.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | 320 px card: white / #17171A, 1 px #E5E7EB / #2A2A2E, 14 px radius, soft shadow; the list scrolls past 360 px |
| Header | "Memory" 13/600 + grey count pill; right: "Use memory" 12 px + 30 × 18 switch (blue #2563EB / #4C8DFF on, grey off) |
| Filter | chips 22 px (All / each category, count muted), the picked one a white pill with a hairline and shadow; a search button at the end, which swaps the chips for a field with clear (×) and Close search |
| Groups | uppercase 10 px muted heading per category |
| Row | 44 px min, hairline dividers #F0F0F0 / #222226; text 12 px over "source · date" 11 muted; a paused row is grey with an amber Paused tag (#B45309 on #FEF3C7); on hover or focus a light fill (#F7F7F8 / #1E1E22) and a three-button tray (edit, pause/resume, forget) floats at the right over a fade, so the text keeps the full width |
| Last row | "+ New memory" (opens a field, category chips, Cancel and Save – Save disabled until there is text); after a forget, a "Forgot "…" · Undo" line for a few seconds |
| Status | 11 px muted: "n memories · n paused", the pointed row's category, or "Memory is off" |
| Off note | when switched off, a soft note unfolds under the header and the chips and list dim; nothing is deleted |

## 3. Behaviour
- `mp_counts` per category (All first), `mp_groups` the filtered and searched groups in category order, `mp_status`
  the status line, `mp_quote` the shortened text in the undo line.
- Filtering folds the other groups away; searching filters as you type (case-insensitive, text and category).
- Forget removes the row and drops the counts; Undo restores it at its old place. Pause greys the text and shows the
  tag; the button becomes Resume. Edit turns the text into a field in place.
- `on_change` gets the whole list after every change; `on_enabled_change` the switch.
- prefers-reduced-motion: changes happen without folding or fading.

## 4. Variants
default · filtered (Projects) · search · tray · undo · off · new memory · classic.
Controls: style, state (7).

## 5. Keyboard and accessibility
Tab: switch, chips, search, each memory's actions, New memory. ← / → move between chips (radiogroup, roving focus).
Enter saves an edit or a new memory (Shift+Enter adds a line); Escape cancels it or closes search. Use memory is a
role=switch; Pause is a toggle (aria-pressed); a row's actions also appear on keyboard focus; the undo line is a polite
status region.

## 6. Responsive
Fixed 320 px card, shrinking to its container; the tray overlays the row so nothing reflows on hover.

## 7. Known gaps
- Counts change in place rather than rolling.
- Undo does not scroll the restored row into view.
- GPUI: a static snapshot – no editing, search or undo timer.
