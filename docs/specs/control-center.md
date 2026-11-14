**Status:** v1 — implemented (plan #26).

# Spec: control center

> Implemented in `oxyn-core::primitives::control_center` (`ControlCenter`, `FocusMode`, `Channel`, `Meter`, `Detail`, `StepKey`), `oxyn-dioxus::ControlCenter`, `oxyn-gpui::ControlCenterEl`.
> Source: Arc UI Control center (behaviour only, no source used).

## 1. Purpose
Quick workspace settings in one panel: start a focus session, silence channels, present, set alert volume and text size. Oxyn adds the agent's effort, permissions and voice.

## 2. State machine
```
Grid ─tile─▶ toggle (Focus starts the first mode at its default / ends; Notifications all off / all on; Presenting)
Grid ─chevron─▶ Detail(focus | notifications | agent) — grid inert
Focus detail ─pick mode─▶ dial seeded ─turn─▶ minutes count across 12 o'clock, resist past 5 / 120 ─release─▶ snap to 5
Focus detail ─Start / Update session─▶ Grid ; ─End session─▶ Grid
Meter ─grab─▶ follows 1:1 from the grab point, rubber-banded ─release─▶ step ; tap without drag jumps
Detail ─Esc / scrim / back─▶ Grid
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| tile off / on | icon, name, live line | Enter toggles | `button` `aria-pressed` |
| detail | modal panel over the grid | Esc closes | `dialog`, grid `inert` |
| dial | ring, knob, minutes, until time | arrows ±5, Page ±15, Home / End | `slider` with valuetext |
| meter held | track swells | arrows, Page, Home / End | `slider` with % |

## 4. Micro-interactions
Tiles flip to the inverted surface when on; the line under the name rises when it changes. The dial's knob springs to snapped minutes on release. Meters swell 6 % while held.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Detail open | 280 ms | `ease.out` | fade |
| Knob snap | 240 ms | spring (1.4) | instant |
| Meter swell | 160 ms | `ease.out` | none |
| Tile line | 200 ms | `ease.out` | swap |

## 6. Variants
Free: focus modes, session dial, channels, alert volume, text size, people who can still reach you.

**Oxyn plus:** agent tile and detail (effort, permissions, voice), "silence alerts while agents run", tile order.

## 7. Edge cases
The dial keeps counting across twelve so 50 → 65 crosses the top cleanly. Values past the ends resist and settle back. The panel is a fixed height; details scroll.

## 8. Accessibility
Tiles are toggle buttons; chevrons open a modal detail named by its title; the dial and meters are sliders with formatted valuetext ("50 minutes, until 3:40 PM", "64%"); channels are switches.

## 9. Props sketch
```rust
ControlCenter { modes: Vec<FocusMode>, channels: Vec<Channel>, people: Vec<String>,
                default_volume, default_text_size, now_min, agent_tiles, open_detail: Option<String>,
                label, on_state_change: EventHandler<(String, String)> }
```

## 10. Related components and references
`toggles.md`, `slider.md`, `time-dial.md`. Arc UI `/docs/components/control-center` — behaviour only.
