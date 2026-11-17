**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: AI Chat · Start (`chat-start`)

Family: Screens. Reference: affectUI AI Chat / Start, behaviour only.
Components: `ChatStartScreen` (Dioxus) · `ChatStartEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
Where a new chat begins: greeting, composer, folder and apps to work with, starters.

## 2. Parts
| Part | Look |
|---|---|
| Greeting | By hour of day (morning / afternoon / evening / late) |
| Composer | Field, Add menu (adds file chips), ask-before-acting menu, model menu, dictation toggle, Send disabled until text |
| Context | Work-in-a-folder menu, Connect apps menu (brand marks), connected app badges |
| Starters | Seven chips that fill the composer |

## 3. Behaviour
Enter sends (Shift+Enter breaks a line). The first message adds a chat to the sidebar (title from the first words), opens it, and the conversation takes over the page; the reply appears after a short Thinking state. New chat returns to the start.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
