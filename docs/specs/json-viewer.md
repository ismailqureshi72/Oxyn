**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: JSON viewer (`json-viewer`)

Family: Data display ([display-atoms.md](display-atoms.md)). Closes the parity row(s): Arc UI · JSON viewer.

## 1. Purpose
A collapsible JSON tree (a small built-in parser, no dependency): branches open to `with_depth(n)` on first render (1 opens the root), search filters to matches and opens their branches, long arrays and objects page (`page`, default 50) with Show n more, each row copies its value or path (Copy value text is the real value via `value_text`), the current row's path shows under the tree with a Copied status, and the keyboard moves through it.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| browsing | Rows, current row highlighted |  |
| searching | Matches highlighted, branches forced open | Esc clears |
| no matches | No matches |  |

## 3. Timing and motion
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| tree | Two levels |
| search | Query |
| paged | Show more |
| bare | No search / copy |

## 5. Keyboard
| Key | Does |
|---|---|
| ↓ ↑ | Next / previous row |
| → ← | Open / step in; close / parent |
| Home End | First / last |
| Enter / Space | Toggle, or load more on Show more |
| ⌘/Ctrl C (+⇧) | Copy value (path) |
| / | Focus search |
| Enter / ⇧Enter (search) | Next / previous match |
| Esc (search) | Clear |
| ↓ (search) | Into the tree |

## 6. Accessibility
- role=tree with treeitem rows (aria-level, aria-expanded, aria-selected) and aria-activedescendant; the tree has an accessible name (`label`).

## 7. Responsive
- Values truncate; the tree scrolls inside `max_height`.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| tree | JsonTree | — | root, root_name, open, query, page, pages, current |
| searchable | bool | true |  |
| copyable | bool | true |  |
| show_path | bool | true |  |
| max_height | u32 | 320 | px |
| label | String | JSON |  |
| on_toggle / on_query / on_more / on_key | EventHandler<String> | — | key ids: up, down, left, right, home, end, toggle, next, prev, #index |

## 9. Edge cases
- Match navigation wraps.
- Paths read root.users[0].name.

## 10. Implementation
- Core: `Json`, `JsonTree` (`with_depth`, `value_text`, `rows`, `key`), `JRow` (`oxyn_core::parity::tools`), with tests.
- Dioxus: `JsonViewer` (parity/tools.rs, tools.css).
- GPUI: `JsonViewerEl` (rows and the path line; keyboard via the host).

## 11. References
- Arc UI JSON viewer (https://uiarc.dev/components/json-viewer) — behaviour-only rebuild, no source copied.
