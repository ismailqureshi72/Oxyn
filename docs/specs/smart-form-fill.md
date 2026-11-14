**Status:** v1 — implemented (plan #25).

# Spec: smart form fill

> Implemented in `oxyn-core::primitives::smart_fill` (`SmartFill`, `FillField`, `FillKind`,
> `Extracted`, `FieldMark`, `FillPhase`, `extract`, `contact_fields`), `oxyn-dioxus::SmartFormFill`,
> `oxyn-gpui::SmartFillEl`. Source: Arc UI Smart form fill (behaviour only, no source used). The
> built-in `extract` is a deterministic heuristic standing in for the model; hosts pass real model
> results to `SmartFill::fill`.

## 1. Purpose
Paste an email signature or messy notes and have the agent fill a contact (or lead, or ticket)
form. The user wants to see where each value came from and to check only the doubtful ones.

## 2. State machine
```
Idle ─paste / ⌘↵─▶ Reading (model shimmer; Esc stops: nothing changes)
Reading ─result─▶ Filling (fields fill in form order, 140 ms apart; each source span lights up)
Filling ─▶ Done
field confidence < threshold ─▶ Review (aria-invalid, blocks Save)
Review ─check─▶ Checked ; any filled field ─type─▶ Edited
Save ─any Review─▶ refused, first one announced ; ─else─▶ Saving ─▶ Saved | Error
Undo ─▶ values from before the fill ; Undo field (Oxyn plus)
threshold slider (Oxyn plus) ─▶ re-marks filled fields
```

## 3. States table
| Field mark | Shows | ARIA |
|---|---|---|
| plain | empty field | — |
| filled | value fades in, tint breathes, dot + "91%" | `aria-description` "Confidence 91%" |
| review | warning edge, dot + "Review", note ("Check the street and unit"), ✓ button | `aria-invalid=true` |
| checked | "Checked" | — |
| edited | "Edited" | — |

Footer: "Filled 6 fields" / "1 to review" / "Saved"; Undo; Save contact. Keyboard: Tab through the
text area, Try sample, every field, the ✓ buttons, Undo, Save.

## 4. Micro-interactions
While reading, the model name shimmers. Each value fades in with a short tint on the field; its
source in the pasted text highlights as it lands, and again on hover or focus of the field. The
company field shows a logo slot. Status is a dot plus a word or percentage, never colour alone.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Read (demo) | 900 ms | — | same |
| Field stagger | 140 ms apart | — | 0 |
| Value fade | 200 ms | `ease.out` | 1 ms |
| Field tint | 900 ms | ease-out | none |
| Source highlight | 160 ms | `ease.out` | instant |

## 6. Variants
Free: default contact fields (name, title, company, email, phone, address), custom fields, model
name, review threshold, submit label, sample texts.

**Oxyn plus:** undo per field; a confidence gate the user can move; provenance notes per field
("Guessed from the email address", "From the email domain").

## 7. Edge cases
Missing name: guessed from the email local part at low confidence. Personal email domains don't
become companies. Phones without a country code are flagged. A second fill keeps Undo pointing at
the values before the latest fill. Narrow: the paste well stacks over the form below 720 px;
fields take the full row below 440 px. Very slow models: Stop stays available.

## 8. Accessibility
A polite region announces each step ("Reading the pasted text…", "Filled 6 fields. 1 need
review.", "Check Address before saving", "Saved"). Each filled input has an `aria-description`
with confidence and reason; review fields are `aria-invalid`. The pasted text is a focusable
labelled region.

## 9. Props sketch
```rust
SmartFormFill { fields: Vec<FillField { id, label, kind }>, title, description, model,
                review_threshold: u8, gate, submit_label, prefill,
                on_values_change: EventHandler<(String, String)>,
                on_submit: EventHandler<Vec<(String, String)>> }
```

## 10. Related components and references
`phone-input.md`, `text-field.md`, `genui-forms.md`, `streaming-markdown.md`. Arc
`/components/smart-form-fill` — behaviour only.
