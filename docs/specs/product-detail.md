**Status:** built v0.1 (2026-11-19, S14: Arc screens III).

# Spec: Product detail (`product-detail`)

Family: Arc screens (pages at `/screens/product-detail/` and `/full`). Reference: the product detail screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, ticker, product, client and person is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `ProductDetailScreen` (Dioxus) · `ProductDetailEl` (GPUI), rules in `oxyn_core::parity::arcscr3`.

## 1. Purpose
Choose a colour, size and quantity of one product and add it to the cart.

## 2. Parts
The Arc frame (shared with the S12 and S13 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Sidebar, header | As Product list, with the product name in the breadcrumb |
| Gallery | A large tile in the picked colour and a thumbnail per colour |
| Buy column | All products, the collection tag, name, price (Sale with the old price), rating and reviews, a line about it, **Colour** swatches (radio), **Size** buttons XS–XL (sold-out sizes struck and greyed, aria-disabled), the stock word for the picked size, a quantity stepper and **Add to cart · $289.00** |
| Cart | Items, subtotal, each line's colour and size, Check out |

## 3. Behaviour
`Catalog::size`, `step_qty` and `add`: a sold-out size refuses ("XS is sold out"); the quantity stays between 1 and the size's stock (at most 10); Add to cart needs a size and refuses more than the stock across the cart; the same colour and size merge into one line.

## 4. Variants
States: Data · Loading · Empty · **Pick a size** (Add pressed without a size) · **Sold out** (a sold-out size pressed) · **Added to cart** (Navy, size S). Frame: Inset · Flat.

## 5. Keyboard and accessibility
Colours and sizes are radio groups; refusals are an alert tied to the size group; the stepper buttons are named.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons and the meta line hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 640 px the gallery sits above the buy column.

## 7. Data
`ProductDetailScreen` takes `layout` and `state`.

## 8. Known gaps
No checkout; Check out only says so.

## 9. GPUI
`ProductDetailEl` draws one state. In `ScreenView`: `pd-colour-<n>`, `pd-size-<n>`, `pd-qty-up|down`, `pd-add`, `pd-back`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S14 Arc III).
