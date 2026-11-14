**Status:** v1 — implemented (plan #27).

# Spec: empty states

> Implemented in `oxyn-core::primitives::layout (`EmptyPreset`) + display (`EmptyState`)`, `oxyn-dioxus::EmptyPreset`, `oxyn-gpui::EmptyStateEl`.
> Source: Arc UI Empty state and Empty states (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
A next step when there is nothing to show: first run, no results, no runs, inbox zero, offline, error, no access.

## 2. State machine
```
Preset p ─preset changes─▶ copy rolls in place
first-run + prompts ─pick─▶ on_prompt
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| framed | dashed frame, icon, title, text, actions | Tab | `status` |
| bare | no frame | Tab | same |
| inline | one line | Tab | same |

## 4. Micro-interactions
Changing preset rolls the block in place (rise + fade); the icon tile pops.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Roll | 280 ms | `ease.out` | instant |

## 6. Variants
Free: Arc’s empty state (icon, title, description, action).

**Oxyn plus:** seven agent-app presets, starter prompts, secondary action, inline layout.

## 7. Edge cases
No-results reflects the query. Long text wraps up to 520 px.

## 8. Accessibility
`role=status` with the title; the icon is decorative.

## 9. Props sketch
```rust
EmptyPreset { preset: "first-run"|"no-results"|"no-runs"|"inbox-zero"|"offline"|"error"|"no-access", query, layout, prompts, on_action }
```

## 10. Related components and references
`display-atoms.md` (EmptyState), `messages.md` (empty transcript), `starter-prompts` (messages.md). Arc UI Empty state(s) — behaviour only.
