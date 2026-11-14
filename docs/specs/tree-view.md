**Status:** v1 — implemented (plan #27).

# Spec: tree view

> Implemented in `oxyn-core::primitives::layout (`Tree`, `TreeNode`, `TreeKey`, `TreeRow`)`, `oxyn-dioxus::TreeView`, `oxyn-gpui::TreeViewEl`.
> Source: Arc UI Tree view (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Files and folders an agent touched, project navigation, nested settings.

## 2. State machine
```
Row ─click / Enter─▶ Selected (+ folder toggles)
Folder closed ─→─▶ Open ─→─▶ focus first child     Folder open ─←─▶ Closed ─←─▶ focus parent
↑ ↓ Home End move focus over visible rows; * expands sibling folders
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| row | icon, label, badge | arrows | `treeitem` level/pos/size |
| selected | filled row | Enter | `aria-selected` |
| folder open | rotated chevron, children | ← closes | `aria-expanded` |

## 4. Micro-interactions
Opening a folder staggers its children in; chevrons rotate on a snappy spring; the selection fill transitions between rows.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Children in | 200 ms | `ease.out` | instant |
| Chevron | 200 ms | overshoot | instant |

## 6. Variants
Free: comfortable rows, icons.

**Oxyn plus:** guide lines, git / agent badges (M · A · D · ●), `compact` density.

## 7. Edge cases
Labels ellipsize; 18 px per level so very deep trees lose label room. Empty folders still toggle.

## 8. Accessibility
`tree` / `treeitem` with level, position, set size, expanded and selected; roving tab index; icons and guides `aria-hidden`.

## 9. Props sketch
```rust
TreeView { nodes: Vec<TreeNode>, expanded: Vec<String>, selected: Option<String>, label, guides: bool,
           density: "comfortable"|"compact", on_select: EventHandler<String> }
```

## 10. Related components and references
`workspace-chrome.md` (file tree), `sidebar.md`. Arc UI Tree view — behaviour only.
