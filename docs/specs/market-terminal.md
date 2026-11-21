**Status:** built v0.1 (2026-11-19, S14: Arc screens III).

# Spec: Trading desk (`market-terminal`)

Family: Arc screens (pages at `/screens/trading-desk/` and `/full`). Reference: the market terminal screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, ticker, product, client and person is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `MarketTerminalScreen` (Dioxus) · `MarketTerminalEl` (GPUI), rules in `oxyn_core::kit::arcscr3`.

## 1. Purpose
Watch a few symbols, read one price and chart, and place an order that the account can cover.

## 2. Parts
The Arc frame (shared with the S12 and S13 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Individual: Trade, Positions, Orders, Activity; **Watchlist · 7 symbols** (fictional tickers KSTR, NVLA, BRWK, TMBR, QLMR, HRBR, CDRX) with a lettered badge, name, live price and signed day change in green or rose. Context card: **Account value** and its day change |
| Header | Individual › Trade, meta "Market open · 2026-11-12 14:05 ET" (or closed, opening 2026-11-13 09:30 ET), a **Live / Paused / Closed** mark and Pause / Resume |
| Quote | Badge, ticker and name, the live price (a status region) and the change since the previous close, signed and coloured |
| Chart | 1D / 1W / 1M / 3M / 1Y, low and high, a line with a light area in the trend's hue over grid lines |
| Tables | Positions (shares, average cost, value, return) and Open orders (side tag, quantity, limit or Market, ISO time, Cancel), then Filled today |
| Order ticket (300 px) | Buy / Sell, Market / Limit, a shares stepper (typed or stepped), the limit price stepper, the estimate, cash or shares held, the refusal line and **Buy 10 KSTR** in green (Sell in rose) |

## 3. Behaviour
`Market`, `series` and `live_price` in `oxyn_core::kit::arcscr3`: each symbol has a fixed series per range ending at its close; the live price is a small deterministic walk driven by a tick (the host clock every 1.5 s in Dioxus, a press in GPUI) and stops while paused or closed. A market order fills at the live price and moves cash, shares and the average cost; a limit order joins Open orders; a closed market queues market orders. The ticket refuses zero shares, a buy beyond cash and a sell beyond the shares held.

## 4. Variants
States: Data · Loading · Empty ("No positions yet" with Add funds) · **Limit order** (15 shares at $184.00) · **Order placed** (a market buy filled) · **Market closed** (prices still, a note). Frame: Inset · Flat.

## 5. Keyboard and accessibility
Tab walks the watchlist, the range group, the tables and the ticket; Enter places the order from the ticket. Buy / Sell, Market / Limit and the range are radio groups; the price is a status region; a refusal is an alert; gains and losses carry a sign as well as a colour.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 1180 px positions and orders stack; under 960 px the ticket drops below the chart.

## 7. Data
`MarketTerminalScreen` takes `layout`, `state` and `now` (`ClockMs`, the host clock).

## 8. Known gaps
No real market data or broker; orders fill in the demo state only.

## 9. GPUI
`MarketTerminalEl` draws one state; the chart is a bar series. In `ScreenView` presses act on `Arc3Ui::mk`: `mk-sym-<n>`, `mk-range-<n>`, `mk-buy|sell`, `mk-market|limit`, `mk-qty-up|down`, `mk-px-up|down`, `mk-place`, `mk-cancel-<id>`, `mk-tick` (Next tick). The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S14 Arc III).
