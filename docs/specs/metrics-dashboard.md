**Status:** built v0.1 (2026-11-19, S12: Arc screens I).

# Spec: Metrics dashboard (`metrics-dashboard`)

Family: Arc screens (pages at `/screens/metrics-dashboard/` and `/full`). Reference: the metrics dashboard screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, workspace and project is fictional; dates are absolute ISO dates.
Components: `MetricsDashboardScreen` (Dioxus) · `MetricsDashboardEl` (GPUI), rules in `oxyn_core::parity::arcscr`.

## 1. Purpose
Read a site's traffic at a glance and dig into one metric over a chosen period.

## 2. Parts
The Arc frame (shared by the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Lumenfield: Overview, Pages, Sources, Locations, Devices, Goals 6; **Saved views** (Launch week, Pricing page, Organic search with the coloured Google mark, DACH visitors; each sets its range). Context card **Online now 142** with a 24-bar sparkline (the newest bar solid) and "30 min ago · Now" |
| Header | Lumenfield › Overview; **date range** menu (Last 7 days, 30 days, 90 days, 12 months); **Export** (turns to Exported with a green check; the file name joins the date line) |
| Second row | **Filter** (the saved views as a menu, with a chip to clear), the period as absolute ISO dates (`2026-10-13 to 2026-11-11`) |
| KPI tabs | Visitors, Page views, Signups, Bounce rate, Visit duration: value and change (↗ green when good, ↘ or rose when bad; bounce rate in points, where down is good). The open tab is grey-filled |
| Chart | Legend, "Drag across the chart to zoom" or **Reset zoom**, a **Compare** switch. Y axis 0…top on four steps (`6K`, `42%`, `2:30`), the area and line in blue, the previous period dashed grey, six ISO dates under it. Hover shows a dashed line and `2026-10-17 · 4,021`; dragging paints the window and zooms to it (at least 3 points) |
| Tables | Three cards, each with two tabs: Top pages / Entry pages, Referrers / Campaigns (official marks: Google, X, GitHub, Slack, Discord, Notion; Direct is a link glyph), Countries / Cities (two-letter codes). Rows carry an inline bar by share, the count right-aligned; a row press selects it; "View all … ›" |

## 3. Behaviour
All numbers come from `oxyn_core::parity::arcscr`: one series a day (52 weeks for 12 months) ending 2026-11-11, the previous period for Compare and the KPI changes, tables scaled to the range. Changing the range resets the zoom. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading (skeleton tiles, chart and tables) · Empty (zeros, "No visits yet" with Copy snippet, "No pages yet" in each table) · **Compare** · **Zoomed** · **Exported**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
The KPIs are a tablist (Tab, then Enter); the chart is an image named for its metric and dates; Compare is a switch; menus close on Escape. Table tabs are tablists; rows are toggle buttons. The sidebar is a nav landmark; the open row has aria-current=page; the account menu and every menu close on Escape.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it). Under 960 px the five KPI tabs stay in one row with tighter padding; under 640 px they wrap into two columns (Visit duration spans both), so none is cut off or hidden behind a sideways scroll. Under 640 px header actions become icon buttons, the meta line hides and grids drop to one column. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`MetricsDashboardScreen` takes `layout` and `state`; its numbers come from the functions in `oxyn_core::parity::arcscr` (`series`, `kpi_value`, `table`), which a host replaces with its own source.

## 8. Known gaps
No backend: the series are generated fictional data. In GPUI the chart draws columns (with the previous period as a line over each), zooms to a preset window (Zoom in, Reset zoom) instead of a dragged one, shows no hover readout, and its tables show their first tab; Filter is said back.

## 9. GPUI
`MetricsDashboardEl` draws one state (`.state()`, `.extra()`). `ScreenView` (oxyn-gpui, `parity::screenview`) runs it live as part of the **Arc screens I** set, driven by `oxyn_core::parity::screenui::ScreenUi` and its `ArcUi`: `ArcUi` keeps the range, KPI, Compare, zoom (`mx-zoom` zooms to the middle half, `mx-unzoom`) and Export (`mx-export`); presses `mx-range-<7d|30d|90d|12m>`, `mx-kpi-<n>`, `mx-compare`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S12 Arc I).
