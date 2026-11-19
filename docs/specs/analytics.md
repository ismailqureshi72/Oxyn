**Status:** built v0.2 (2026-11-18, S2: Workspace parity; v0.1 screens 1).

# Spec: Analytics (`analytics`)

Family: Screens. Reference: affectUI Analytics (`/screens/analytics`), layout and behaviour only; no source or assets copied.
Components: `AnalyticsScreen` (Dioxus) · `AnalyticsEl` (GPUI), models in `oxyn_core::parity::workscr`.

## 1. Purpose
Revenue at a glance for a period: the headline numbers, where sales came from and how this week compares.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Revenue persona: Home · Revenue · Invoices 12 · Payouts · Customers, then the Channels group |
| Top bar | 48 px: Analytics / Revenue; on the right Last 7 days ⌄, Compare, Export ⌄ and the black Save as report |
| KPI tiles | Four 88 px cards (revenue, customers, conversion and refunds), each with its value, a green change and seven small bars, the last one darker |
| Revenue by channel | The headline ($96.4k) with a change chip, a line chart of four channels ($6k / $3k / $0 axis, three day labels), and a footer "Top 4 channels" with a legend |
| Revenue breakdown | 342 px card: six rows from gross sales down to net revenue, each with its value and a trend sparkline (green up, red down, grey flat), alternate rows tinted |
| Bottom row | Top products as blue bars, then two charts of this week against last (sessions; average order value), each with a footer line |

Chrome is monochrome neutral; colour only means something: official brand marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
The range menu (7, 30, 90 days) rescales every number and axis; Compare draws the previous period dashed under each channel; legend buttons hide or show a channel; charts take focus and ← / → move a crosshair with a readout (also on hover); Export offers CSV, PDF or a sheet and announces it; Save as report toggles to Saved.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States: Data · Loading (value and chart skeletons) · Empty (dashes and "No sales in this range") · **Compared** (previous period dashed). Frame: Inset · Flat. Oxyn extras: Compare, legend toggles, the crosshair readout, the range menu rescaling, Export choices.

## 5. Keyboard and accessibility
Charts are role=img with a label and a polite live readout; legend toggles report aria-pressed; product bars are meters with values; the range and Export are menus. Escape closes any open menu or popover and clears a search; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/analytics/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and top-bar actions keep their icons with hidden labels. Under 980 px KPI tiles go to two columns and both chart rows stack; under 520 px the KPI bars hide and values wrap under their change. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; drag, menus, typing and charts' crosshair belong to the host.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
