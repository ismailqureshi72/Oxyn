**Status:** v1 — implemented (plan #23).

# Spec: split button, button group, floating button group, expanding button group

> Implemented in `oxyn-core::primitives::button` (`SplitButton`, `Glide`, `hidden_dividers`, `fit`,
> `ExpandingGroup`, `tray_visible`, `GroupItem`) and `oxyn-core::primitives::nav` (roving focus);
> `oxyn-dioxus::{SplitButton, ButtonGroup, FloatingButtonGroup, ExpandingButtonGroup}`;
> `oxyn-gpui::{SplitButtonEl, ButtonGroupEl (joined and floating), ExpandingGroupEl}`. Parity
> source: Arc UI (behaviour only).

## 1. Purpose
Agent UIs keep several related actions in a small space: the answer toolbar (copy, regenerate,
good / bad, share), merge options on a PR card, formatting in an editor, model choice next to Run.
Groups make them one object with one highlight, so the pointer reads as a single soft shape moving
over the row instead of buttons flickering on and off.

## 2. State machine
**Shared highlight (`Glide`).** `target: Option<usize>`, `from`, `at`, `jump`.
`go(Some(i), keyboard=false)` glides from the previous item; `go(Some(i), keyboard=true)` jumps;
`go(None)` hides it; arriving from nothing it lands in place and grows in from 94 %.

**Split button.** `Closed ─(click / Enter on ▾, ↓ / ↑)─▶ Open(highlight) ─(pick)─▶ Closed + action`
`Open ─Esc / outside─▶ Closed (focus back to ▾)`. With `remember`, a picked alternative becomes
the main half (`SplitButton::choose`), and the main label morphs.

**Button group.** Roving tab stop; ← / → (↑ / ↓ when vertical), Home / End move focus *and* the
highlight; Enter / Space activate. Segments past `max_visible` (or `fit()` of the width) go to an
overflow menu at the end of the row.

**Floating group.** Same as the group, plus the tooltip of the highlighted button and, with
`auto_hide`, `Visible ─idle 3 s─▶ Dimmed ─pointer / focus─▶ Visible`.

**Expanding group.** `Collapsed ─hover (after delay) / focus─▶ Open(i)`;
`Open(i) ─activate─▶ Confirmed(i) ─confirm (1.4 s)─▶ Open(i) / Collapsed`. The label slot reserves
`max(label, done_label)` so the confirmation never changes width.

## 3. States table
| Component | State | Shows | Keyboard | ARIA |
|---|---|---|---|---|
| Split | closed | main label + icon, divider, ▾ | Tab between halves; ↓ / ↑ / Enter on ▾ opens | `group`; ▾ `aria-haspopup=menu`, `aria-expanded` |
| Split | open | menu under the button, first / last row highlighted from the keyboard | ↑ ↓ Home End, Enter picks, Esc closes | `menu` / `menuitem` |
| Split | disabled | both halves at 45 % | skipped | `disabled` |
| Group | rest | joined surface, hairline dividers | one tab stop | `group`, `aria-orientation` |
| Group | highlight on i | soft shape under i; dividers each side of i fade out | arrows move it | — |
| Group | selected (toggle groups) | selected surface + inset hairline | Enter / Space | `aria-pressed` |
| Group | overflow | "⋯" opens the rest + attached menu | Enter / ↓ | `aria-haspopup=menu` |
| Floating | hover / focus i | highlight morphs in position and size; tooltip on its side | arrows | `toolbar` |
| Floating | dimmed (auto-hide) | 35 % opacity until pointer / focus returns | focus restores | — |
| Expanding | open i | i grows to show its label; neighbours slide aside | ← → Home End | each button named by its label |
| Expanding | confirmed i | ✓ + done label (success colour), same width | — | polite live region says the done label |

## 4. Micro-interactions
- **One highlight** lives under all segments (Arc): moving the pointer carries it on `spring.snap`;
  keyboard moves jump. Appearing from nothing it grows in from 94 % instead of sliding from where it
  last was. The dividers next to it fade so it reads as one shape inside the joined surface.
- **Pressed segment** answers in place (scale 0.96) without moving the highlight.
- **Split button:** the main half's label and icon morph in place while its width springs; the ▾
  half never scales, so the menu opens from a still anchor; menu rows fade in with a 20 ms stagger.
  The chevron turns half a circle while open.
- **Expanding:** width springs between icon size and icon + measured label (Arc `morph`, 420 ms);
  the label fades in after the width starts; confirmation swaps the icon for a check.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger |
|---|---|---|---|---|
| Highlight glide | 214 ms (`spring.snap`) | 0 | spring | — |
| Highlight appear | 214 ms, scale 0.94 → 1, fade | 0 | `ease.out` | — |
| Divider fade | 120 ms | 0 | `ease.out` | — |
| Segment press | 80 ms, 0.96 | 0 | `ease.out` | — |
| Split label morph | 180 ms crossfade + width `spring.snap` | 0 | `ease.out` | — |
| Split menu open | 180 ms fade (anchor still) | 0 | `ease.out` | rows 20 ms |
| Floating tooltip | instant within the group (neighbour rule) | 0 | — | — |
| Auto-hide | 300 ms fade to 35 % | 3 s idle | `ease.out` | — |
| Expand width | 420 ms (Arc `morph`) | `delay_ms` (Oxyn plus, default 0) | `ease.out` | — |
| Confirm hold | 1400 ms (`confirmDuration`) | — | — | — |
| **Reduced motion** | highlight jumps (80 ms fade); no press scale; widths change in 80 ms; no stagger | — | linear | none |

## 6. Variants
Source (Arc): split `primary` / `secondary`; group `outline` / `solid`, `sm` / `md`, horizontal /
vertical, `collapseLabels`, attached `menu`; floating `muted` / `floating`, `sm` / `md`,
orientation, `iconOnly`, `tooltipSide`; expanding `sm` / `md`, `defaultExpanded`,
`confirmDuration`.

**Oxyn plus:**
1. Split: remember the last choice and promote it to the main half.
2. Group: overflow-to-menu (`max_visible` / `fit()`), per-segment counts, dense IDE size (24 px).
3. Floating: highlight style `pill` · `underline` · `glow`; auto-hide on idle.
4. Expanding: hover delay (`delay_ms`); "labels always on" accessibility toggle.

## 7. Edge cases
- **Disabled segments** are skipped by arrows and never take the highlight.
- **Highlight target removed** (overflow recomputed): the highlight hides instead of pointing at
  nothing; the next hover makes it appear in place.
- **Narrow containers:** `fit()` keeps as many segments as fit beside the overflow button.
- **Vertical groups:** dividers become horizontal; arrows switch axis.
- **Long done labels** in the expanding group widen the reserved slot once, at mount.
- **Touch:** no hover; the floating group shows labels on long-press via the tooltip.
- **Split button with one action** renders the main half only (no menu).

## 8. Accessibility
- Group: `role=group` (toolbars: `role=toolbar`) with `aria-orientation`; one tab stop, arrow keys
  inside. Icon-only segments carry `aria-label` and a tooltip.
- Toggle segments expose `aria-pressed`. The overflow trigger is `aria-haspopup=menu` +
  `aria-expanded`.
- Split: the ▾ is a separate button named "More actions"; Esc returns focus to it.
- Expanding: every button is named by its label even while collapsed; the done label is announced
  once through a polite live region.

## 9. Props sketch
```rust
SplitButton { actions: Vec<(id, label)>, variant: "primary|secondary", icon, disabled, remember, open, on_action }
ButtonGroup { items: Vec<GroupItem>, variant: "outline|solid", size: "ide|sm|md", vertical,
              collapse_labels, selected, max_visible, menu: Vec<MenuItem>, disabled, highlight, on_action }
FloatingButtonGroup { items, variant: "muted|floating", size, vertical, icon_only, tooltip_side,
                      highlight_style: "pill|underline|glow", auto_hide, highlight, on_action }
ExpandingButtonGroup { items, size, default_expanded, labels, delay_ms, confirmed, now, on_action }
GroupItem { id, label, icon, count, done_label, disabled }
```

## 10. Related components and references
`dropdown-menu.md` (split and overflow menus share its rows), `segmented-control.md` (the same
glide for selection), `tooltip-popover.md`. Arc `/components/split-button`, `/button-group`,
`/floating-button-group`, `/expanding-button-group` — behaviour only.
