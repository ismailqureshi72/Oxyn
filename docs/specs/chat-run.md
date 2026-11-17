**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: AI Chat · Run (`chat-run`)

Family: Screens. Reference: affectUI AI Chat / Run, behaviour only.
Components: `ChatRunScreen` (Dioxus) · `ChatRunEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
One autonomous run: the plan, live numbers, a gate that waits for you and the draft it wrote.

## 2. Parts
| Part | Look |
|---|---|
| Header | Status tag (Running / Needs you / On hold / Done / Stopped), Stop |
| Stats | Steps, tool calls, time (live clock), cost |
| Plan | Editable steps with tool, state and duration |
| Activity | Finished steps with a fold for their tool result |
| Run details | Approval card (Send it / Hold), draft with five suggestions, Accept / Reject (Y / N), Accept all / Reject all, files |

## 3. Behaviour
The run walks the plan on the clock and stops at the send step until you approve; Hold keeps it waiting; Stop freezes time and cost. Plan: Enter renames inline, Alt+↑/↓ moves, Delete removes. Suggestions: the focused one is outlined; deciding jumps to the next open one.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
