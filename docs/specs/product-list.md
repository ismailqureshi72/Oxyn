**Status:** built v0.1 (2026-11-19, S14: Arc screens III).

# Spec: Product list (`product-list`)

Family: Arc screens (pages at `/screens/product-list/` and `/full`). Reference: the product list screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, ticker, product, client and person is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `ProductListScreen` (Dioxus) · `ProductListEl` (GPUI), rules in `oxyn_core::parity::arcscr3`.

## 1. Purpose
Find a product in a shop catalogue by collection, colour and price, and open it.

## 2. Parts
The Arc frame (shared with the S12 and S13 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Fellside Goods: Home, Orders 14, Products, Inventory 6, Customers; **Collections** (Outerwear, Knitwear, Footwear, Bags) with hue dots and counts that filter. Context card: Sales this week |
| Header | Fellside Goods › Products, product count, **Cart** with its count, **Add product** |
| Second row | Search products, six colour swatches (toggle buttons), Any price / Under $150, the sort (Featured, Price low to high, Price high to low, Newest) and Grid / List |
| Grid | Cards with a photo tile tinted by the first colour, name, price (a sale price with the old one struck through), rating, colour dots and stock in words and colour (In stock green, "3 left" amber, Sold out rose) |
| List | A table: product, collection, stock, price |

## 3. Behaviour
`Catalog` in `oxyn_core::parity::arcscr3`: collection, colour, price cap and search narrow the catalogue together; the sort is stable by id. Opening a product shows its page (see Product detail) with All products to come back.

## 4. Variants
States: Data · Loading · Empty ("No products yet" with Add product) · **Filtered** (Outerwear under $200, Clear filters) · **List view** · **No matches** ("linen", Clear filters). Frame: Inset · Flat.

## 5. Keyboard and accessibility
Swatches are toggle buttons named with their colour; Grid / List is a radio group; the result count is a status region; stock is written out, not only coloured.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 640 px the grid is two columns and ratings hide on cards.

## 7. Data
`ProductListScreen` takes `layout` and `state`; products come from `products(true)`.

## 8. Known gaps
Photos are tinted tiles, not product photography; Add product only says so.

## 9. GPUI
`ProductListEl` draws one state. In `ScreenView`: `pl-coll-<n>`, `pl-colour-<n>`, `pl-max`, `pl-sort`, `pl-view`, `pl-clear`, `pl-open-<id>`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S14 Arc III).
