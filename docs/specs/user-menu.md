**Status:** v1 — implemented (plan #27).

# Spec: user menu

> Implemented in `oxyn-core::primitives::chrome (`UserMenu`, `UserRow`, `ThemePref`, `Presence`, `account_label`)`, `oxyn-dioxus::UserMenu, PresenceDot`, `oxyn-gpui::UserMenuEl`.
> Source: Arc UI User menu (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
The account menu in the app shell: identity, settings, theme, status, usage and sign out.

## 2. State machine
```
Closed ─Enter / ↓─▶ Open(first)   ─↑─▶ Open(last)   ─click─▶ Open(no highlight)
Open ─↑ ↓ Home End / A–Z─▶ highlight ─← →─▶ theme / status cycle (stays open)
Open ─row─▶ select + close   ─Sign out─▶ Signing out (spinner, aria-busy) ─▶ Closed
Open ─Escape / Tab / outside─▶ Closed (focus → avatar)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| closed | avatar (+ name) | Enter, ↑ ↓ | `aria-haspopup=menu` |
| open | panel | arrows, typeahead | `menu` |
| switch row | segmented | ← → | `menuitemradio` |
| signing out | spinner | — | `aria-busy` |

## 4. Micro-interactions
The panel scales and fades out of the avatar without overshoot; rows stagger in; the highlight follows pointer and keys.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Open | 200 ms | `ease.out` | fade |
| Rows | 200 ms stagger | `ease.out` | instant |

## 6. Variants
Free: identity, links with shortcuts, theme switch, sign out.

**Oxyn plus:** status switch with shaped presence dots, usage meter, opens upward from a sidebar footer.

## 7. Edge cases
Below 640 px it is a bottom sheet. A menu that starts open draws no backdrop and takes no focus.

## 8. Accessibility
Trigger “Account menu, name, status”; rows are menuitems; switches are labelled groups of menuitemradio; shortcut hints `aria-hidden`.

## 9. Props sketch
```rust
UserMenu { name, email, plan: Option<String>, items: Vec<(label, icon, keys)>, show_name, show_theme, show_status,
           theme, status, usage: Option<(f32, f32)>, align, side: "down"|"up", open, on_theme, on_select, on_sign_out }
```

## 10. Related components and references
`team-switcher.md`, `sidebar.md`, `dropdown-menu.md`. Arc UI User menu — behaviour only.
