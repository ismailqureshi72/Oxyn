**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Split bar (`split-bar`)

Family: Meters ([composer-context-meter.md](composer-context-meter.md)). Closes the parity row(s): Arc UI · Split bar. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A single 100 % bar split into shares. Each segment is a full-width layer cut to its share with an animated clip-path, so reflow never touches layout and a new target retargets mid-flight. Resting on or arrowing onto a segment widens it by 4.5 % of the bar (or to 16 % when smaller, enough to fit its name and share) while the others give way proportionally and dim (`split`). The readout shows the highlighted segment's value and share (one decimal) or the total. The legend switches segments off and on (at least one stays) and the rest reflow. Labels sit inside segments of at least 13 %, with the name opening beside the percentage when the segment is wide or highlighted; smaller ones label below on leader lines.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Shares, total readout | hover / arrows |
| highlight | One segment wide, others dim | leave / Esc |
| hidden | Segment off, legend struck | legend / Enter |

## 3. Timing and motion
- Clip-path edges 450 ms (cubic-bezier .25,1,.4,1); names open 350 ms.
- Reduced motion swaps widths without the glide.

## 4. Variants
| Variant | What differs |
|---|---|
| traffic | Six sources |
| budget | Currency |
| hidden | One switched off |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Bar, then legend items |
| → ↓ / ← ↑ | Next / previous visible segment |
| Home / End | First / last |
| Enter / Space | Switch the highlighted segment off |
| Esc | Clear |

## 6. Accessibility
- The readout is aria-live=polite; legend buttons expose aria-pressed.

## 7. Responsive
- Outside labels below segments narrower than 13 %.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| title / description | String / Option<String> | — |  |
| shares | Vec<Share> | — |  |
| unit / prefix | String | — |  |
| total_label | String | Total |  |
| default_hidden | Vec<String> | — |  |
| legend | bool | true |  |
| on_hidden_change | EventHandler<Vec<String>> | — |  |

## 9. Gaps against the source
- Locale-aware currency formatting is reduced to a prefix and grouping.

## 10. GPUI
SplitBarEl lays out segments with the same `split()` widths.
