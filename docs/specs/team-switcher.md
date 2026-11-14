**Status:** v1 — implemented (plan #27).

# Spec: team switcher

> Implemented in `oxyn-core::primitives::chrome (`TeamSwitcher`, `Workspace`, `SwitcherRow`, `SwitcherView`)`, `oxyn-dioxus::TeamSwitcher, WorkspaceTile`, `oxyn-gpui::TeamSwitcherEl`.
> Source: Arc UI Team switcher (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Switch between workspaces or orgs at the top of the sidebar, like Linear, Vercel or Slack.

## 2. State machine
```
Closed ─open─▶ List(query = "") ─type─▶ filtered (+ Create “query”)
List ─Enter on workspace / ⌘n─▶ Switching(id, shimmer) ─delay─▶ Active(id)
List ─Create / Join─▶ Form ─Enter─▶ create + switch / “Simulated, nothing was sent”
Escape: Form → List → clear query → Closed
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| closed | tile, name, plan | Enter / ↓ | `aria-haspopup=dialog` |
| list | search + rows + actions | ↑ ↓ Enter, ⌘1–9 | combobox + listbox |
| switching | shimmer, spinner | — | `aria-busy` |
| form | create / join | Enter, Escape | labelled field |

## 4. Micro-interactions
The panel fades and scales in from 98 %; switching crossfades the trigger and shimmers the plan line; views shift 6 px in the direction of travel.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Panel in | 180 ms | `ease.out` | fade |
| Switch | 650 ms (demo) | — | same |
| Shimmer | 1.2 s loop | linear | static |

## 6. Variants
Free: search, roles, shortcuts, create and join.

**Oxyn plus:** `compact` tile-only trigger for the icon rail, identity-hued tiles, member counts.

## 7. Edge cases
No match: Enter opens Create with the name filled in. Home / End only move when the search is empty.

## 8. Accessibility
Search is a combobox with `aria-activedescendant`; the current workspace is `aria-selected` with a check; a polite region announces switches.

## 9. Props sketch
```rust
TeamSwitcher { workspaces: Vec<Workspace>, value: Option<String>, compact: bool, open: bool, delay: u32, on_change: EventHandler<String> }
```

## 10. Related components and references
`user-menu.md`, `sidebar.md`, `command-palette.md`. Arc UI Team switcher — behaviour only.
