**Status:** v1 — implemented (plan #25).

# Spec: tag input

> Implemented in `oxyn-core::primitives::tags` (`TagInput`, `Tag`, `TagKind`, `TagRule`,
> `Normalize`, `Overflow`, `TagKey`, `TagEvent`), `oxyn-dioxus::TagInput`, `oxyn-gpui::TagInputEl`.
> Sources: affectUI Tag Input, Arc UI Tag input (behaviour only, no source used).

## 1. Purpose
Several short values in one field: invitees, labels, allowed domains, the files and people an
agent should look at. The user types or pastes a list and wants each value checked and kept as a
removable chip without losing what they typed.

## 2. State machine
```
Empty ─type─▶ Drafting(draft) ─Enter / , / Tab / blur─▶ commit(draft)
commit ─valid, new─▶ Added(chip pops in) ─▶ Drafting("")
commit ─invalid─▶ Rejected(draft kept, message) ─type─▶ Drafting (message clears)
commit ─duplicate─▶ Duplicate(existing chip flashes, draft cleared)
commit ─at max─▶ Full("Up to N", counter warns)
paste(list) ─▶ for each part: Added | Rejected (kept in the field, comma-joined) | Duplicate | Full
Drafting("") ─⌫─▶ Picked(last) ─⌫─▶ Removed(last) ─▶ Empty | Drafting("")
Drafting("") ─←─▶ Picked(i) ─← / →─▶ Picked(i±1) ─→ past last─▶ Drafting
Picked(i) ─⌫ / Del─▶ Removed(i), pick moves to the neighbour
Picked(i) ─Enter─▶ Editing(i) (chip back in the field)          Oxyn plus
any ─Esc─▶ clears the pick, then the draft
Disabled: no transitions; chips show without ×.
```

## 3. States table
| State | Shows | Primary / secondary | Keyboard |
|---|---|---|---|
| empty | placeholder, hint | type / paste | Enter, `,`, Tab add |
| drafting | typed text, suggestions (Oxyn plus) | add / pick suggestion | ← at start picks the last chip |
| chip added | chip pops in, counter rolls | remove × | ⌫ marks the last chip |
| picked chip | 2 px focus ring on the chip | remove | ← → walk, ⌫ / Del remove, Enter edit |
| rejected | message in place of the hint, `aria-invalid` | fix the draft | typing clears the message |
| duplicate | the existing chip flashes once | — | — |
| full | counter `8/8` in warning, "Up to 8" | remove a chip | adds are refused |
| folded (Oxyn plus) | first N chips then `+M` | — | — |
| disabled | 50 % chips, no × | — | not focusable |

## 4. Micro-interactions
New chips grow in from 86 % with a short blur; removed chips leave and their neighbours close the
gap. A duplicate flashes the chip it repeats. Pasting a list adds each value in turn. The counter
and `+N` roll. Typed chips (Oxyn plus) carry a person, file, tool or label icon.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Chip in | 160 ms, scale .86 → 1, blur 3 → 0 | `ease.out` | 1 ms |
| Duplicate flash | 600 ms | `ease.out` | none |
| Counter / `+N` roll | 200 ms | `ease.out` | swap |
| Message in | 180 ms rise | `ease.out` | swap |

## 6. Variants
Free: labels (any), people (emails with initials, lowercased), domains, `max:N` length checks,
counter with `max`, wrap.

**Oxyn plus:** typed chips (`@person`, `path/file.rs`, `tool:name`, `#label`) with icons;
suggestions offered while typing; fold past N into `+M`; Enter on a chip moves it back to edit;
hidden inputs per chip for forms (`name`).

## 7. Edge cases
Long values end in … at 14 rem with the full text in `title`. A pasted list keeps rejected parts
in the field so nothing is lost. Space only splits with `split_on_space`. Very long lists scroll
inside the field after ~5 lines. Normalisation (trim / lower / slug) happens before the duplicate
check. RTL: chips flow from the right, ← / → follow visual order. Touch: 24 px × targets, 16 px
input text (no iOS zoom).

## 8. Accessibility
A real labelled `input`; chips are spans with a labelled "Remove {value}" button (`tabindex=-1`;
the keyboard reaches chips with ←). Adds, removals, duplicates and limits go to a polite live
region ("Added sam@acme.dev, 2 of 8"). Messages set `aria-invalid` and are linked with
`aria-describedby`. Chip contrast meets 4.5:1 on `surface.hover`.

## 9. Props sketch
```rust
TagInput { label, value: Vec<String>, placeholder, hint, error, optional,
           validate: "any" | "email" | "domain" | "max:N", normalize: "trim" | "lower" | "slug",
           split_on_space, max: Option<usize>, avatar, typed, suggestions: Vec<String>,
           fold: Option<usize>, name, disabled, on_change: EventHandler<Vec<String>> }
```

## 10. Related components and references
`combobox.md` (pick from a list instead of free entry), `text-field.md` (shared field look),
`badges-chips.md` (chip look). affectUI `/components/tag-input`; Arc `/components/tag-input` —
behaviour only.
