**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 6).

# Spec: Autonomy levels (`autonomy-dial`)

Family: Agents. Closes the parity row: affectUI · Autonomy Dial (Pro). Behaviour-only rebuild; nothing is copied (the
demo agent, its actions and wording are our own). Related Oxyn components: composer-permission-selector and
permission-rules (per-tool rules; this one is the level view).
Components: `AutonomyDial` (Dioxus) · `AutonomyDialEl` (GPUI), model in `oxyn_core::kit::autonomyfid`.

## 1. Purpose
How much an agent may do on its own: three levels, and the agent's real actions split between "Runs on its own" and
"Asks you first", with a lock per action that always asks.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 432 px (max 100 %), container-sized, 12 px radius (continuous where supported), hairline, faint shadow |
| Header | blue two-tone sparkle; title 14.5/500; subtitle 13 muted |
| Levels | radio group "Autonomy": three 26 px grey pills 9 px apart; the chosen one white with a hairline and a soft lift |
| Groups | a full-width hairline, then "Runs on its own" and "Asks you first" (12.5 muted) each with a bordered count chip |
| Row | 32.6 px: 16 px coloured brand mark, label 14; Risky (amber) or Destructive (red) tag with its glyph; a lock button at the end, shown when locked, on hover or focus |
| Empty | a grey panel "Every action waits for you" |

## 3. Behaviour
- `asks_first(action, level, pinned)`: a lock always asks; Always ask asks for everything; If risky runs only safe
  actions; Full auto runs everything not locked. The running agent uses the same rule.
- Picking a level or a lock moves the affected rows to the other group; they glide in a beat after each other and the
  count chips roll. Uncontrolled (`default_level`, `default_pinned`) or controlled (`level`, `pinned`);
  `on_level_change`, `on_pinned_change`.
- prefers-reduced-motion: everything switches at once.

## 4. Variants
if-risky (default) · full-auto · always-ask (empty group) · locked (a safe action locked) · pointed (row tint + lock) ·
details (a line under each action) · narrow (320 px: details hidden, tighter levels) · classic (levels in one track in a
white card, each action its own bordered card on a grey panel).

## 5. Keyboard and accessibility
Arrows pick the previous / next level (roving focus); Tab goes on to the locks; Space or Enter sets or removes a lock.
The levels are a radio group named Autonomy and described by what the level means; both groups are labelled sections;
every lock is a toggle button with aria-pressed naming its action; changes are announced politely
("If risky: 2 on its own, 4 ask first").

## 6. Responsive
Up to 432 px. Under 340 px the detail lines go and the levels tighten; without hover the locks always show and rows
grow to 44 px.

## 7. Known gaps
- The glide is a fade-and-slide into the new group, not a true FLIP from the old position; the card's height jumps.
- GPUI: a static snapshot – the host keeps the level and locks; the sparkle is a square and the lock a glyph.
