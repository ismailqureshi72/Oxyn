//! Screens S14 (Arc screens III): **Market terminal**, **Product list**, **Product detail** and
//! **Invoice studio** on the Arc frame shared with S12 (`arcscr.rs`), and **Sign-in split**, a
//! split page of its own. Rules in `oxyn_core::parity::arcscr3`; styles in `arcscr3.css`
//! (`oxs14-`). Chrome stays monochrome; colour is for gains and losses, stock, statuses,
//! collections and the official SSO marks.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::parity::arcscr3::{
    CLIENTS, COLLS, COLOURS, Catalog, DEMO_EMAIL, IV_STATUS, Invoice, MK_RANGES, Market, SIZES,
    SORTS, SSO, SYMBOLS, ScreenState, Side, SignIn, TERMS, TODAY, arc3_split, change, email_error,
    inv_no, iso, money, money0, pass_error, pct, s14_states as core_states, series, stock_word, up,
};

use super::arcscr::{ArcFrame, btn, find_field, group, none, nrow, seg, skl};
use super::screensfid::nico;
use crate::brand::BrandIcon;

/// The state bar entries of an S14 screen (for a host's `ScreenStage { states }`).
pub fn s14_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    core_states(slug).to_vec()
}

// ───────────────────────────────────────────────────────────── shared parts ──

fn tk_badge(s: usize) -> Element {
    let (t, _, _, hue) = SYMBOLS[s % SYMBOLS.len()];
    let ini = &t[..1];
    rsx! { span { class: "oxs14-tkb", "data-hue": "{hue}", "aria-hidden": "true", "{ini}" } }
}

fn delta(bp: i64) -> Element {
    let hue = if bp >= 0 { "green" } else { "rose" };
    let txt = pct(bp);
    rsx! { span { class: "oxs14-dl", "data-hue": "{hue}", "{txt}" } }
}

/// A polyline path over `v` in a `w` × `h` box.
fn path_of(v: &[i64], w: f64, h: f64) -> (String, String) {
    let lo = v.iter().copied().min().unwrap_or(0) as f64;
    let hi = v.iter().copied().max().unwrap_or(1) as f64;
    let span = (hi - lo).max(1.0);
    let n = v.len().max(2) - 1;
    let mut d = String::new();
    for (i, x) in v.iter().enumerate() {
        let px = i as f64 / n as f64 * w;
        let py = h - 4.0 - (*x as f64 - lo) / span * (h - 8.0);
        d.push_str(&format!(
            "{}{px:.1} {py:.1} ",
            if i == 0 { "M" } else { "L" }
        ));
    }
    let area = format!("{d}L{w:.1} {h:.1} L0 {h:.1} Z");
    (d, area)
}

fn spark(v: &[i64]) -> Element {
    let (d, _) = path_of(v, 56.0, 20.0);
    let hue = if up(v) { "green" } else { "rose" };
    rsx! { svg { class: "oxs14-spk", "data-hue": "{hue}", view_box: "0 0 56 20", width: "56", height: "20", "aria-hidden": "true", path { d: "{d}" } } }
}

// ───────────────────────────────────────────────────────────── market ──

/// Screen – Market terminal: a watchlist of fictional symbols with sparklines, the open symbol's
/// live price (ticking with the host clock, Pause to hold it) and chart over 1D–1Y, positions,
/// open orders with Cancel, and an order ticket (Buy / Sell, Market / Limit, a shares stepper,
/// an estimate, and checks for cash and shares). States: `sample`, `loading`, `empty`, `limit`,
/// `placed`, `closed`.
#[component]
pub fn MarketTerminalScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// The host clock (the live price ticks every 1.5 s).
    #[props(default)]
    now: ClockMs,
) -> Element {
    let (st, own) = arc3_split(&state);
    let mut said = use_signal(String::new);
    let mut mk = use_signal(move || {
        let mut m = Market::new(st != ScreenState::Empty);
        match own {
            "limit" => {
                m.ticket.limit = true;
                m.ticket.qty = 15;
                m.ticket.price = 18_400;
            }
            "placed" => {
                let _ = m.place();
            }
            "closed" => m.open = false,
            _ => {}
        }
        m
    });
    let base = use_signal(move || now.0);
    let mut held_at = use_signal(|| None::<u64>);
    let loading = st == ScreenState::Loading;
    let live_tick = if !mk.read().open {
        0
    } else {
        held_at().unwrap_or(now.0.saturating_sub(base()) / 1500)
    };
    let mut m = mk();
    m.tick = live_tick;
    let s = m.sym;
    let price = m.price();
    let (dc, dbp) = change(s, price);
    let mut ser = series(s, m.range);
    if m.range == 0
        && let Some(l) = ser.last_mut()
    {
        *l = price;
    }
    let dtxt = format!(
        "{}{} ({})",
        if dc >= 0 { "+" } else { "" },
        money(dc),
        pct(dbp)
    );
    let (line, area) = path_of(&ser, 600.0, 220.0);
    let hue = if up(&ser) { "green" } else { "rose" };
    let lo = ser.iter().copied().min().unwrap_or(0);
    let hi = ser.iter().copied().max().unwrap_or(0);
    let check = m.check();
    let est = m.estimate();
    let paused = held_at().is_some();
    let open = m.open;
    let value = m.value();
    let day = m.day_bp();
    let positions = m.positions();
    let orders = m.orders.clone();
    let fills = m.fills.clone();
    let t = m.ticket.clone();
    let side_word = if t.side == Side::Buy { "Buy" } else { "Sell" };
    let ranges: Vec<(&'static str, String)> =
        MK_RANGES.iter().map(|r| (r.0, r.0.to_string())).collect();
    rsx! {
        ArcFrame { slug: "market-terminal", layout, said,
            crumbs: vec!["Individual".to_string(), "Trade".to_string()],
            meta: if open { "Market open · 2026-11-12 14:05 ET".to_string() } else { "Market closed · opens 2026-11-13 09:30 ET".to_string() },
            side: rsx! {
                {nrow(nico("chart"), "Trade".to_string(), String::new(), true, EventHandler::new(move |_| said.set("Trade".to_string())))}
                {nrow(nico("wallet"), "Positions".to_string(), positions.len().to_string(), false, EventHandler::new(move |_| said.set("Positions".to_string())))}
                {nrow(nico("list"), "Orders".to_string(), orders.len().to_string(), false, EventHandler::new(move |_| said.set("Orders".to_string())))}
                {nrow(nico("clock"), "Activity".to_string(), String::new(), false, EventHandler::new(move |_| said.set("Activity".to_string())))}
                {group("Watchlist · 7 symbols", true)}
                for (i, (tk, name, _, _)) in SYMBOLS.iter().copied().enumerate() {
                    { let p = m.price_of(i); let (_, bp) = change(i, p); let on = i == s;
                      rsx! { button { key: "{i}", r#type: "button", class: "oxs14-wl", "aria-pressed": "{on}", "aria-label": "{tk} {name} {money(p)} {pct(bp)}",
                        onclick: move |_| { let mut w = mk.write(); w.tick = live_tick; let r = w.open_sym(i); said.set(r); },
                        {tk_badge(i)}
                        span { class: "oxs12-an", b { "{tk}" } span { class: "oxs12-ell", "{name}" } }
                        span { class: "oxs14-wlr", span { class: "oxs14-num", "{money(p)}" } {delta(bp)} }
                      } } }
                }
            },
            card: rsx! {
                div { class: "oxs14-acv",
                    span { class: "oxs12-q", "Account value" }
                    b { class: "oxs14-num", "{money(value)}" }
                    span { {delta(day)} span { class: "oxs12-q", " today" } }
                }
            },
            actions: rsx! {
                span { class: "oxs14-live", "data-on": "{open && !paused}", span { class: "oxs12-dot" } if !open { "Closed" } else if paused { "Paused" } else { "Live" } }
                if open { button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-pressed": "{paused}", onclick: move |_| { if held_at().is_some() { held_at.set(None); said.set("Live prices".to_string()); } else { held_at.set(Some(live_tick)); said.set("Prices paused".to_string()); } }, {nico(if paused { "flash" } else { "pause" })} span { class: "oxs12-btl", if paused { "Resume" } else { "Pause" } } } }
            },
            if loading {
                div { class: "oxs14-mk", div { class: "oxs14-mkm", {skl(10, "")} } div { class: "oxs14-tkt", {skl(6, "")} } }
            } else {
                div { class: "oxs14-mk",
                    div { class: "oxs14-mkm",
                        div { class: "oxs14-qh",
                            {tk_badge(s)}
                            div { class: "oxs12-an", b { "{SYMBOLS[s].0}" } span { class: "oxs12-ell", "{SYMBOLS[s].1}" } }
                            span { class: "oxs12-grow" }
                            div { class: "oxs14-px",
                                b { class: "oxs14-num", role: "status", "aria-label": "{SYMBOLS[s].0} {money(price)}", "{money(price)}" }
                                span { class: "oxs14-pxd", "data-hue": if dc >= 0 { "green" } else { "rose" }, "{dtxt}" }
                            }
                        }
                        if !open {
                            p { class: "oxs14-ban", role: "note", {nico("clock")} span { "The market is closed. Market orders queue for the open on 2026-11-13 09:30 ET." } }
                        }
                        div { class: "oxs14-chh",
                            {seg("Range", ranges, MK_RANGES[m.range].0, EventHandler::new(move |k: &'static str| { if let Some(i) = MK_RANGES.iter().position(|r| r.0 == k) { mk.write().range = i; said.set(k.to_string()); } }))}
                            span { class: "oxs12-grow" }
                            span { class: "oxs12-q", "Low {money(lo)} · High {money(hi)}" }
                        }
                        div { class: "oxs14-chart", "data-hue": "{hue}",
                            svg { view_box: "0 0 600 220", preserve_aspect_ratio: "none", role: "img", "aria-label": "{SYMBOLS[s].0} over {MK_RANGES[m.range].0}, low {money(lo)}, high {money(hi)}",
                                for g in 1..4 { { let y = g * 55; rsx! { line { key: "{g}", class: "oxs14-gl", x1: "0", x2: "600", y1: "{y}", y2: "{y}" } } } }
                                path { class: "oxs14-area", d: "{area}" }
                                path { class: "oxs14-line", d: "{line}" }
                            }
                        }
                        div { class: "oxs14-tbls",
                            section { class: "oxs14-tb",
                                h3 { class: "oxs12-h2", "Positions" }
                                if positions.is_empty() {
                                    {none("wallet", "No positions yet", "Fund the account, then buy your first shares from the ticket.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| said.set("Add funds".to_string()), {nico("plus")} span { class: "oxs12-btl", "Add funds" } } })}
                                } else {
                                    table { class: "oxs14-t",
                                        thead { tr { th { "Symbol" } th { class: "oxs14-r", "Shares" } th { class: "oxs14-r oxs14-hide", "Avg cost" } th { class: "oxs14-r", "Value" } th { class: "oxs14-r", "Return" } } }
                                        tbody {
                                            for (ps, n, c, p) in positions.iter().copied() {
                                                { let ret = if c > 0 { (p - c) * 10_000 / c } else { 0 };
                                                  rsx! { tr { key: "{ps}", td { button { r#type: "button", class: "oxs14-sb", onclick: move |_| { let r = mk.write().open_sym(ps); said.set(r); }, {tk_badge(ps)} "{SYMBOLS[ps].0}" } } td { class: "oxs14-r oxs14-num", "{n}" } td { class: "oxs14-r oxs14-num oxs14-hide", "{money(c)}" } td { class: "oxs14-r oxs14-num", "{money0(p * n as i64)}" } td { class: "oxs14-r", {delta(ret)} } } } }
                                            }
                                        }
                                    }
                                }
                            }
                            section { class: "oxs14-tb",
                                h3 { class: "oxs12-h2", "Open orders" }
                                if orders.is_empty() {
                                    p { class: "oxs12-q oxs14-pad", "No open orders" }
                                }
                                ul { class: "oxs14-ol",
                                    for o in orders.iter().cloned() {
                                        { let id = o.id; let w = if o.side == Side::Buy { ("Buy", "green") } else { ("Sell", "rose") };
                                          let lim = o.limit.map(money).unwrap_or_else(|| "Market".to_string());
                                          rsx! { li { key: "{id}",
                                            span { class: "oxs14-side", "data-hue": "{w.1}", "{w.0}" }
                                            span { class: "oxs12-an", b { class: "oxs12-ell", "{o.qty} {SYMBOLS[o.sym].0} · {lim}" } span { class: "oxs12-ell", "{o.at}" } }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-label": "Cancel {o.qty} {SYMBOLS[o.sym].0}", onclick: move |_| { let r = mk.write().cancel(id); if let Some(r) = r { said.set(r); } }, "Cancel" }
                                          } } }
                                    }
                                }
                                if !fills.is_empty() {
                                    h3 { class: "oxs12-h2", "Filled today" }
                                    ul { class: "oxs14-ol",
                                        for o in fills.iter().cloned() {
                                            li { key: "f{o.id}", span { class: "oxs12-hi", "data-hue": "green", {nico("check")} } span { class: "oxs12-ell", if o.side == Side::Buy { "Bought {o.qty} {SYMBOLS[o.sym].0}" } else { "Sold {o.qty} {SYMBOLS[o.sym].0}" } } span { class: "oxs12-grow" } span { class: "oxs12-q", "{o.at}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    form { class: "oxs14-tkt", "aria-label": "Order ticket", onsubmit: move |e: FormEvent| { e.prevent_default(); let mut w = mk.write(); w.tick = live_tick; let r = w.place(); said.set(match r { Ok(s) | Err(s) => s }); },
                        div { class: "oxs12-seg oxs14-bs", role: "radiogroup", "aria-label": "Side",
                            button { r#type: "button", role: "radio", class: "oxs12-sego", "data-side": "buy", "aria-checked": "{t.side == Side::Buy}", onclick: move |_| mk.write().ticket.side = Side::Buy, "Buy" }
                            button { r#type: "button", role: "radio", class: "oxs12-sego", "data-side": "sell", "aria-checked": "{t.side == Side::Sell}", onclick: move |_| mk.write().ticket.side = Side::Sell, "Sell" }
                        }
                        div { class: "oxs14-fr", span { class: "oxs12-q", "Order type" }
                            div { class: "oxs12-seg", role: "radiogroup", "aria-label": "Order type",
                                button { r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{!t.limit}", onclick: move |_| mk.write().ticket.limit = false, "Market" }
                                button { r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{t.limit}", onclick: move |_| { let mut w = mk.write(); w.tick = live_tick; w.ticket.limit = true; w.ticket.price = w.price(); }, "Limit" }
                            }
                        }
                        div { class: "oxs14-fr", span { class: "oxs12-q", "Shares" }
                            div { class: "oxs14-step",
                                button { r#type: "button", class: "oxs12-ib", "aria-label": "Fewer shares", onclick: move |_| { let mut w = mk.write(); w.ticket.qty = w.ticket.qty.saturating_sub(1); }, "−" }
                                input { r#type: "number", inputmode: "numeric", min: "0", "aria-label": "Shares", value: "{t.qty}", oninput: move |e| { let q = e.value().trim().parse::<u32>().unwrap_or(0).min(9_999); mk.write().ticket.qty = q; } }
                                button { r#type: "button", class: "oxs12-ib", "aria-label": "More shares", onclick: move |_| { let mut w = mk.write(); w.ticket.qty = (w.ticket.qty + 1).min(9_999); }, "+" }
                            }
                        }
                        if t.limit {
                            div { class: "oxs14-fr", span { class: "oxs12-q", "Limit price" }
                                div { class: "oxs14-step",
                                    button { r#type: "button", class: "oxs12-ib", "aria-label": "Lower limit", onclick: move |_| { let mut w = mk.write(); w.ticket.price = (w.ticket.price - 5).max(1); }, "−" }
                                    span { class: "oxs14-num oxs14-stv", "{money(t.price)}" }
                                    button { r#type: "button", class: "oxs12-ib", "aria-label": "Raise limit", onclick: move |_| mk.write().ticket.price += 5, "+" }
                                }
                            }
                        }
                        div { class: "oxs14-fr", span { class: "oxs12-q", if t.side == Side::Buy { "Estimated cost" } else { "Estimated proceeds" } } b { class: "oxs14-num", "{money(est)}" } }
                        div { class: "oxs14-fr", span { class: "oxs12-q", if t.side == Side::Buy { "Cash available" } else { "Shares held" } } span { class: "oxs14-num", if t.side == Side::Buy { "{money(m.cash)}" } else { "{m.held[s]}" } } }
                        if let Some(e) = check.clone() { p { class: "oxs14-err", role: "alert", {nico("help")} span { "{e}" } } }
                        button { r#type: "submit", class: "oxs12-btn oxs14-go", "data-side": if t.side == Side::Buy { "buy" } else { "sell" }, disabled: check.is_some(), "{side_word} {t.qty} {SYMBOLS[s].0}" }
                        p { class: "oxs12-q oxs14-fine", if t.limit { "Good for the day. Fills at {money(t.price)} or better." } else if open { "Fills at the best price now." } else { "Queues for the open." } }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── products ──

fn swatch(c: usize) -> Element {
    let (n, hex) = COLOURS[c % COLOURS.len()];
    rsx! { span { class: "oxs14-sw", style: "background:{hex}", title: "{n}", "aria-hidden": "true" } }
}

fn shot(coll: usize, colour: usize, big: bool) -> Element {
    let (_, hex) = COLOURS[colour % COLOURS.len()];
    let ic = match coll {
        0 => "shirt",
        1 => "shirt",
        2 => "box",
        _ => "bag",
    };
    rsx! { span { class: "oxs14-shot", "data-big": "{big}", style: "--sw:{hex}", "aria-hidden": "true", {nico(ic)} } }
}

fn stars(r: u32) -> Element {
    let txt = format!("{}.{}", r / 10, r % 10);
    rsx! { span { class: "oxs14-stars", "aria-label": "Rated {txt} of 5", {nico("star")} b { "{txt}" } } }
}

/// Screen – Product list: a shop's catalogue with collections, a search, colour swatches, an
/// Under $150 filter, a sort and Grid / List; open a product for its detail. States: `sample`,
/// `loading`, `empty`, `filtered`, `listview`, `nomatch`.
#[component]
pub fn ProductListScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    rsx! { ProductScreen { slug: "product-list", layout, state } }
}

/// Screen – Product detail: the product page with a gallery, colour swatches, sizes with stock
/// (sold-out sizes refuse), a quantity stepper and Add to cart, and the cart beside it. States:
/// `sample`, `loading`, `empty`, `nosize`, `soldout`, `added`.
#[component]
pub fn ProductDetailScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    rsx! { ProductScreen { slug: "product-detail", layout, state } }
}

#[component]
fn ProductScreen(slug: &'static str, layout: String, state: String) -> Element {
    let (st, own) = arc3_split(&state);
    let mut said = use_signal(String::new);
    let detail_first = slug == "product-detail";
    let mut cat = use_signal(move || {
        let mut c = Catalog::new(st != ScreenState::Empty);
        match own {
            "filtered" => {
                c.coll = Some(0);
                c.max = Some(20_000);
            }
            "listview" => c.list = true,
            "nomatch" => c.query = "linen".into(),
            "nosize" => {
                c.open_id(1);
                let _ = c.add();
            }
            "soldout" => {
                c.open_id(3);
                c.size(0);
            }
            "added" => {
                c.open_id(1);
                c.pick_colour = 4;
                c.size(1);
                let _ = c.add();
            }
            _ if detail_first => {
                c.open_id(1);
            }
            _ => {}
        }
        c
    });
    let mut q = use_signal(move || {
        if own == "nomatch" {
            "linen".to_string()
        } else {
            String::new()
        }
    });
    let loading = st == ScreenState::Loading;
    let mut cv = cat();
    cv.query = q();
    let shown = cv.shown();
    let cur = cv.current().cloned();
    let detail = cur.is_some();
    let count = cv.cart_count();
    let total = cv.cart_total();
    let crumbs = match &cur {
        Some(p) => vec![
            "Fellside Goods".to_string(),
            "Products".to_string(),
            p.name.to_string(),
        ],
        None => vec!["Fellside Goods".to_string(), "Products".to_string()],
    };
    let sort_l = SORTS[cv.sort].1;
    let cart_lines = cv.cart.clone();
    let items = cv.items.clone();
    rsx! {
        ArcFrame { slug, layout, said,
            crumbs,
            meta: format!("{} products", cv.items.len()),
            side: rsx! {
                {nrow(nico("home"), "Home".to_string(), String::new(), false, EventHandler::new(move |_| said.set("Home".to_string())))}
                {nrow(nico("box"), "Orders".to_string(), "14".to_string(), false, EventHandler::new(move |_| said.set("Orders".to_string())))}
                {nrow(nico("tag"), "Products".to_string(), cv.items.len().to_string(), true, EventHandler::new(move |_| { cat.write().open = None; said.set("Products".to_string()); }))}
                {nrow(nico("layers"), "Inventory".to_string(), "6".to_string(), false, EventHandler::new(move |_| said.set("Inventory".to_string())))}
                {nrow(nico("people"), "Customers".to_string(), String::new(), false, EventHandler::new(move |_| said.set("Customers".to_string())))}
                {group("Collections", true)}
                for (i, (name, hue)) in COLLS.iter().copied().enumerate() {
                    { let on = cv.coll == Some(i); let n = cv.items.iter().filter(|p| p.coll == i).count();
                      rsx! { button { key: "{i}", r#type: "button", class: "oxs12-nr", "aria-pressed": "{on}", onclick: move |_| { let mut w = cat.write(); w.open = None; w.coll = if on { None } else { Some(i) }; said.set(if on { "All products".to_string() } else { name.to_string() }); },
                        span { class: "oxs12-nri", span { class: "oxs12-dot", "data-hue": "{hue}" } } span { class: "oxs12-ell oxs12-nrl", "{name}" } span { class: "oxs12-nrc", "{n}" } } } }
                }
            },
            card: rsx! {
                div { class: "oxs14-acv",
                    span { class: "oxs12-q", "Sales this week" }
                    b { class: "oxs14-num", "$8,420" }
                    span { {delta(640)} span { class: "oxs12-q", " vs last week" } }
                }
            },
            actions: rsx! {
                button { r#type: "button", class: "oxs12-btn", "aria-label": "Cart, {count} items", onclick: move |_| said.set(format!("Cart: {count} items, {}", money(total))), {nico("bag")} span { class: "oxs12-btl", "Cart" } span { class: "oxs14-badge", "{count}" } }
                if !detail { {btn("Add product", "plus", true, EventHandler::new(move |_| said.set("New product draft".to_string())))} }
            },
            bar: if detail { None } else { Some(rsx! {
                {find_field("Search products", "Search products", q, None)}
                div { class: "oxs14-sws", role: "group", "aria-label": "Colour",
                    for (c, (n, _)) in COLOURS.iter().copied().enumerate() {
                        { let on = cv.colour == Some(c);
                          rsx! { button { key: "{c}", r#type: "button", class: "oxs14-swb", "aria-pressed": "{on}", "aria-label": "{n}", title: "{n}", onclick: move |_| { let mut w = cat.write(); w.colour = if on { None } else { Some(c) }; said.set(if on { "Any colour".to_string() } else { n.to_string() }); }, {swatch(c)} } } }
                    }
                }
                button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-pressed": "{cv.max.is_some()}", onclick: move |_| { let mut w = cat.write(); w.max = if w.max.is_some() { None } else { Some(15_000) }; }, {nico("filter")} span { class: "oxs12-btl", if let Some(mx) = cv.max { "Under {money0(mx)}" } else { "Any price" } } }
                span { class: "oxs12-grow" }
                button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-label": "Sort: {sort_l}", onclick: move |_| { let mut w = cat.write(); w.sort = (w.sort + 1) % SORTS.len(); said.set(format!("Sorted: {}", SORTS[w.sort].1)); }, {nico("sort")} span { class: "oxs12-btl oxs12-ell", "{sort_l}" } }
                div { class: "oxs12-seg", role: "radiogroup", "aria-label": "Layout",
                    button { r#type: "button", role: "radio", class: "oxs12-sego oxs14-icn", "aria-checked": "{!cv.list}", "aria-label": "Grid", onclick: move |_| cat.write().list = false, {nico("apps")} }
                    button { r#type: "button", role: "radio", class: "oxs12-sego oxs14-icn", "aria-checked": "{cv.list}", "aria-label": "List", onclick: move |_| cat.write().list = true, {nico("list")} }
                }
            }) },
            if loading {
                div { class: "oxs14-pg", for i in 0..6 { div { key: "{i}", class: "oxs14-pc", span { class: "oxs14-shot oxs12-sb" } {skl(2, "")} } } }
            } else if cv.items.is_empty() {
                {none("tag", "No products yet", "Add your first product or import a catalogue from a spreadsheet.", rsx! { button { r#type: "button", class: "oxs12-btn", "data-pri": "true", onclick: move |_| said.set("New product draft".to_string()), {nico("plus")} span { class: "oxs12-btl", "Add product" } } })}
            } else if let Some(p) = cur {
                { let size = cv.pick_size; let colour = cv.pick_colour; let err = cv.error.clone(); let qty = cv.qty;
                  let left = size.map(|s| p.stock[s]);
                  rsx! {
                    div { class: "oxs14-pd",
                        div { class: "oxs14-gal",
                            {shot(p.coll, colour, true)}
                            div { class: "oxs14-thumbs", for c in p.colours.iter().copied() { button { key: "{c}", r#type: "button", class: "oxs14-th", "aria-pressed": "{c == colour}", "aria-label": "{COLOURS[c].0} photo", onclick: move |_| cat.write().pick_colour = c, {shot(p.coll, c, false)} } } }
                        }
                        div { class: "oxs14-buy",
                            button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost oxs14-back", onclick: move |_| { cat.write().open = None; said.set("Products".to_string()); }, {nico("back")} span { class: "oxs12-btl", "All products" } }
                            span { class: "oxs13-tag", span { class: "oxs12-dot", "data-hue": "{COLLS[p.coll].1}" } "{COLLS[p.coll].0}" }
                            h1 { class: "oxs14-pn", "{p.name}" }
                            div { class: "oxs14-prc",
                                b { class: "oxs14-num", "{money(p.price)}" }
                                if let Some(w) = p.was { del { class: "oxs12-q", "{money(w)}" } span { class: "oxs13-pill", "data-hue": "rose", "Sale" } }
                                span { class: "oxs12-grow" }
                                {stars(p.rating)} span { class: "oxs12-q", "{p.reviews} reviews" }
                            }
                            p { class: "oxs14-blurb", "{p.blurb}" }
                            fieldset { class: "oxs14-fs",
                                legend { "Colour " b { "{COLOURS[colour].0}" } }
                                div { class: "oxs14-sws", role: "radiogroup", "aria-label": "Colour",
                                    for c in p.colours.iter().copied() {
                                        button { key: "{c}", r#type: "button", role: "radio", class: "oxs14-swb", "aria-checked": "{c == colour}", "aria-label": "{COLOURS[c].0}", onclick: move |_| { cat.write().pick_colour = c; said.set(COLOURS[c].0.to_string()); }, {swatch(c)} }
                                    }
                                }
                            }
                            fieldset { class: "oxs14-fs",
                                legend { "Size" if let Some(l) = left { span { class: "oxs12-q", " · " } { let (w, h) = stock_word(l); rsx! { span { class: "oxs14-stk", "data-hue": "{h}", "{w}" } } } } }
                                div { class: "oxs14-sizes", role: "radiogroup", "aria-label": "Size", "aria-describedby": "oxs14-perr",
                                    for (i, sz) in SIZES.iter().copied().enumerate() {
                                        { let out = p.stock[i] == 0; let lab = if out { format!("{sz}, sold out") } else { sz.to_string() };
                                          rsx! { button { key: "{i}", r#type: "button", role: "radio", class: "oxs14-sz", "aria-checked": "{size == Some(i)}", "aria-disabled": "{out}", "aria-label": "{lab}", onclick: move |_| { let r = cat.write().size(i); said.set(r); }, "{sz}" } } }
                                    }
                                }
                            }
                            div { class: "oxs14-addr",
                                div { class: "oxs14-step",
                                    button { r#type: "button", class: "oxs12-ib", "aria-label": "Fewer", onclick: move |_| { cat.write().step_qty(-1); }, "−" }
                                    span { class: "oxs14-num oxs14-stv", "aria-label": "Quantity {qty}", "{qty}" }
                                    button { r#type: "button", class: "oxs12-ib", "aria-label": "More", onclick: move |_| { cat.write().step_qty(1); }, "+" }
                                }
                                button { r#type: "button", class: "oxs12-btn oxs14-go", "data-pri": "true", onclick: move |_| { let r = cat.write().add(); said.set(match r { Ok(s) | Err(s) => s }); }, {nico("bag")} span { class: "oxs12-btl", "Add to cart · {money(p.price * qty as i64)}" } }
                            }
                            p { id: "oxs14-perr", class: "oxs14-perr", role: "alert", if let Some(e) = err { {nico("help")} span { "{e}" } } }
                            if count > 0 {
                                div { class: "oxs14-cart", "aria-label": "Cart",
                                    div { class: "oxs12-cr1", b { "Cart · {count} items" } span { class: "oxs14-num", "{money(total)}" } }
                                    ul {
                                        for (k, (lid, lc, ls, lq)) in cart_lines.iter().copied().enumerate() {
                                            { let nm = items.iter().find(|x| x.id == lid).map(|x| x.name).unwrap_or("");
                                              rsx! { li { key: "{k}", {swatch(lc)} span { class: "oxs12-ell", "{lq} × {nm}" } span { class: "oxs12-q", "{COLOURS[lc].0} · {SIZES[ls]}" } } } }
                                        }
                                    }
                                    button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| said.set(format!("Checkout · {}", money(total))), span { class: "oxs12-btl", "Check out" } }
                                }
                            }
                            p { class: "oxs12-q oxs14-fine", "Free delivery over $150 · returns within 30 days · ships by {iso(TODAY + 2)}" }
                        }
                    }
                  } }
            } else {
                div { class: "oxs14-plw",
                    p { class: "oxs12-q oxs14-cnt", role: "status", "{shown.len()} of {cv.items.len()} products"
                        if cv.filtered() { button { r#type: "button", class: "oxs14-lnk", onclick: move |_| { cat.write().clear(); q.set(String::new()); said.set("Filters cleared".to_string()); }, "Clear filters" } }
                    }
                    if shown.is_empty() {
                        {none("search", "No products match", "Try another colour or price, or clear the filters.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { cat.write().clear(); q.set(String::new()); said.set("Filters cleared".to_string()); }, span { class: "oxs12-btl", "Clear filters" } } })}
                    } else if cv.list {
                        table { class: "oxs14-t oxs14-pt",
                            thead { tr { th { "Product" } th { class: "oxs14-hide", "Collection" } th { "Stock" } th { class: "oxs14-r", "Price" } } }
                            tbody {
                                for p in shown.iter().cloned() {
                                    { let id = p.id; let (w, h) = stock_word(p.total());
                                      rsx! { tr { key: "{id}",
                                        td { button { r#type: "button", class: "oxs14-sb", onclick: move |_| { let r = cat.write().open_id(id); if let Some(r) = r { said.set(r); } }, {shot(p.coll, p.colours[0], false)} span { class: "oxs12-ell", "{p.name}" } } }
                                        td { class: "oxs14-hide oxs12-q", "{COLLS[p.coll].0}" }
                                        td { span { class: "oxs14-stk", "data-hue": "{h}", "{w}" } }
                                        td { class: "oxs14-r oxs14-num", "{money(p.price)}" }
                                      } } }
                                }
                            }
                        }
                    } else {
                        ul { class: "oxs14-pg", "aria-label": "Products",
                            for p in shown.iter().cloned() {
                                { let id = p.id; let (w, h) = stock_word(p.total());
                                  rsx! { li { key: "{id}", class: "oxs14-pc",
                                    button { r#type: "button", class: "oxs14-pcb", "aria-label": "{p.name}, {money(p.price)}", onclick: move |_| { let r = cat.write().open_id(id); if let Some(r) = r { said.set(r); } },
                                        {shot(p.coll, p.colours[0], false)}
                                        span { class: "oxs14-pcn oxs12-ell", "{p.name}" }
                                        span { class: "oxs14-pcp", b { class: "oxs14-num", "{money(p.price)}" } if let Some(was) = p.was { del { class: "oxs12-q", "{money(was)}" } } span { class: "oxs12-grow" } {stars(p.rating)} }
                                        span { class: "oxs14-pcp", span { class: "oxs14-sws", for c in p.colours.iter().copied() { {swatch(c)} } } span { class: "oxs12-grow" } span { class: "oxs14-stk", "data-hue": "{h}", "{w}" } }
                                    }
                                  } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── invoice ──

/// Screen – Invoice studio: invoices grouped by status, the open invoice as paper edited in
/// place (line items to add, edit and remove with Undo, quantity steppers, live subtotal, tax and
/// total), payment terms with an absolute due date, Preview and Send with checks. States:
/// `sample`, `loading`, `empty`, `editing`, `preview`, `sent`.
#[component]
pub fn InvoiceStudioScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc3_split(&state);
    let mut said = use_signal(String::new);
    let mut iv = use_signal(move || {
        let mut v = Invoice::new(st != ScreenState::Empty);
        match own {
            "editing" => {
                v.add_line();
                v.set_line(4, Some("Rush turnaround"), Some(1), Some(25_000));
            }
            "preview" => v.preview = true,
            "sent" => {
                let _ = v.send();
            }
            _ => {}
        }
        v
    });
    let loading = st == ScreenState::Loading;
    let v = iv();
    let cur = v.current().cloned();
    let edit = v.editable() && !v.preview;
    let check = v.check();
    let groups = [2usize, 1, 0, 3];
    rsx! {
        ArcFrame { slug: "invoice-studio", layout, said,
            crumbs: match &cur { Some(i) => vec!["Invoices".to_string(), inv_no(i.no)], None => vec!["Invoices".to_string()] },
            meta: format!("{} outstanding", money0(v.outstanding())),
            side: rsx! {
                {nrow(nico("file"), "All invoices".to_string(), v.list.len().to_string(), true, EventHandler::new(move |_| said.set("All invoices".to_string())))}
                for (i, (l, hue)) in IV_STATUS.iter().copied().enumerate() {
                    { let n = v.count(i); rsx! { button { key: "{i}", r#type: "button", class: "oxs12-nr", onclick: move |_| said.set(format!("{l}: {n}")), span { class: "oxs12-nri", span { class: "oxs12-dot", "data-hue": "{hue}" } } span { class: "oxs12-ell oxs12-nrl", "{l}" } span { class: "oxs12-nrc", "{n}" } } } }
                }
                {group("Clients", true)}
                for (i, (n, _, hue)) in CLIENTS.iter().copied().enumerate() {
                    { let owed: i64 = v.list.iter().filter(|x| x.client == i && (x.status == 1 || x.status == 2)).map(|x| x.total()).sum();
                      let ini = &n[..1];
                      rsx! { button { key: "c{i}", r#type: "button", class: "oxs12-nr", onclick: move |_| said.set(n.to_string()), span { class: "oxs12-nri", span { class: "oxs14-cav", "data-hue": "{hue}", "{ini}" } } span { class: "oxs12-ell oxs12-nrl", "{n}" } span { class: "oxs12-nrc", if owed > 0 { "{money0(owed)}" } else { "" } } } } }
                }
            },
            card: rsx! {
                div { class: "oxs14-acv",
                    span { class: "oxs12-q", "Collected in November" }
                    b { class: "oxs14-num", "$11,759" }
                    span { class: "oxs12-q", "2 payments received" }
                }
            },
            actions: rsx! {
                if cur.is_some() {
                    button { r#type: "button", class: "oxs12-btn", "aria-pressed": "{v.preview}", onclick: move |_| { let mut w = iv.write(); w.preview = !w.preview; w.edit = None; said.set(if w.preview { "Preview".to_string() } else { "Editing".to_string() }); }, {nico(if v.preview { "pen" } else { "eye" })} span { class: "oxs12-btl", if v.preview { "Edit" } else { "Preview" } } }
                    if v.editable() { button { r#type: "button", class: "oxs12-btn", "data-pri": "true", "aria-describedby": "oxs14-ivchk", onclick: move |_| { let r = iv.write().send(); said.set(match r { Ok(s) | Err(s) => s }); }, {nico("send")} span { class: "oxs12-btl", "Send invoice" } } }
                }
                {btn("New invoice", "plus", false, EventHandler::new(move |_| { let r = iv.write().new_draft(); said.set(r); }))}
            },
            right: cur.clone().map(|i| rsx! {
                div { class: "oxs14-det",
                    h3 { class: "oxs12-h2", "Details" }
                    label { class: "oxs14-lbl", "Client"
                        select { class: "oxs14-sel", disabled: !edit, value: "{i.client}", onchange: move |e| { if let Ok(c) = e.value().parse::<usize>() { let r = iv.write().client(c); if let Some(r) = r { said.set(r); } } },
                            for (k, (n, _, _)) in CLIENTS.iter().copied().enumerate() { option { key: "{k}", value: "{k}", selected: k == i.client, "{n}" } }
                        }
                    }
                    span { class: "oxs12-q", if CLIENTS[i.client].1.is_empty() { "No billing email on file" } else { "{CLIENTS[i.client].1}" } }
                    span { class: "oxs14-lbl", "Payment terms" }
                    div { class: "oxs12-seg oxs14-terms", role: "radiogroup", "aria-label": "Payment terms",
                        for (k, (l, _)) in TERMS.iter().copied().enumerate() {
                            button { key: "{k}", r#type: "button", role: "radio", class: "oxs12-sego", disabled: !edit, "aria-checked": "{i.terms == k}", onclick: move |_| { let r = iv.write().terms(k); if let Some(r) = r { said.set(r); } }, "{l}" }
                        }
                    }
                    span { class: "oxs12-q", "Due {iso(i.due())}" }
                    div { class: "oxs14-swr", span { "Sales tax 8.25%" } span { class: "oxs12-grow" } button { r#type: "button", role: "switch", class: "oxs14-swt", disabled: !edit, "aria-checked": "{i.tax > 0}", "aria-label": "Sales tax", onclick: move |_| { let r = iv.write().toggle_tax(); if let Some(r) = r { said.set(r); } } } }
                    div { class: "oxs14-swr", span { "Automatic reminders" } span { class: "oxs12-grow" } button { r#type: "button", role: "switch", class: "oxs14-swt", "aria-checked": "{v.remind}", "aria-label": "Automatic reminders", onclick: move |_| { let mut w = iv.write(); w.remind = !w.remind; said.set(if w.remind { "Reminders on".to_string() } else { "Reminders off".to_string() }); } } }
                    h3 { class: "oxs12-h2", "Activity" }
                    ul { class: "oxs14-act",
                        if i.status != 0 { li { span { class: "oxs12-dot", "data-hue": "blue" } span { class: "oxs12-ell", "Sent to {CLIENTS[i.client].0}" } span { class: "oxs12-q", "{iso(i.issued)}" } } }
                        li { span { class: "oxs12-dot" } span { class: "oxs12-ell", "Created by Rowan Vale" } span { class: "oxs12-q", "{iso(i.issued)}" } }
                    }
                }
            }),
            if loading {
                div { class: "oxs14-iv", div { class: "oxs14-ivl", {skl(8, "")} } div { class: "oxs14-paper", {skl(10, "")} } }
            } else if v.list.is_empty() {
                {none("file", "No invoices yet", "Create an invoice, add line items and send it to a client.", rsx! { button { r#type: "button", class: "oxs12-btn", "data-pri": "true", onclick: move |_| { let r = iv.write().new_draft(); said.set(r); }, {nico("plus")} span { class: "oxs12-btl", "New invoice" } } })}
            } else {
                div { class: "oxs14-iv",
                    nav { class: "oxs14-ivl", "aria-label": "Invoices",
                        for g in groups {
                            { let rows: Vec<_> = v.list.iter().filter(|x| x.status == g).cloned().collect();
                              let (gl, gh) = IV_STATUS[g];
                              rsx! { if !rows.is_empty() {
                                div { key: "g{g}", class: "oxs14-ivg", span { class: "oxs12-dot", "data-hue": "{gh}" } "{gl}" span { class: "oxs12-q", "{rows.len()}" } }
                                for x in rows {
                                    { let no = x.no; let on = Some(no) == v.open;
                                      rsx! { button { key: "{no}", r#type: "button", class: "oxs14-ivr", "aria-current": "{on}", onclick: move |_| { let r = iv.write().open_no(no); if let Some(r) = r { said.set(r); } },
                                        span { class: "oxs12-an", b { class: "oxs12-ell", "{CLIENTS[x.client].0}" } span { class: "oxs12-ell", "{inv_no(no)} · due {iso(x.due())}" } }
                                        b { class: "oxs14-num", "{money0(x.total())}" }
                                      } } }
                                }
                              } } }
                        }
                    }
                    if let Some(i) = cur {
                        { let (sl, sh) = IV_STATUS[i.status];
                          rsx! {
                          div { class: "oxs14-pw",
                            if v.preview { p { class: "oxs14-ban", role: "note", {nico("eye")} span { "Preview: this is what {CLIENTS[i.client].0} sees." } } }
                            if i.status == 1 { p { class: "oxs14-ban", "data-hue": "green", role: "status", {nico("check")} span { "Sent to {CLIENTS[i.client].1}. Due {iso(i.due())}." } } }
                            if let Some((_, l)) = v.removed.clone() {
                                p { class: "oxs14-ban", role: "status", {nico("trash")} span { class: "oxs12-ell", if l.desc.is_empty() { "Removed a line" } else { "Removed {l.desc}" } } button { r#type: "button", class: "oxs14-lnk", onclick: move |_| { let r = iv.write().undo(); if let Some(r) = r { said.set(r); } }, "Undo" } }
                            }
                            article { class: "oxs14-paper", "aria-label": "Invoice {inv_no(i.no)}",
                                div { class: "oxs14-ph",
                                    div { class: "oxs12-an", b { "Northwyn Studio" } span { "14 Ferry Row, Harlow Bay" } }
                                    span { class: "oxs12-grow" }
                                    div { class: "oxs14-pno", span { class: "oxs13-pill", "data-hue": "{sh}", "{sl}" } b { "Invoice {inv_no(i.no)}" } }
                                }
                                div { class: "oxs14-meta",
                                    div { span { class: "oxs12-q", "Bill to" } b { "{CLIENTS[i.client].0}" } span { class: "oxs12-q oxs12-ell", "{CLIENTS[i.client].1}" } }
                                    div { span { class: "oxs12-q", "Issued" } b { "{iso(i.issued)}" } }
                                    div { span { class: "oxs12-q", "Due" } b { "{iso(i.due())}" } span { class: "oxs12-q", "{TERMS[i.terms].0}" } }
                                }
                                table { class: "oxs14-t oxs14-li",
                                    thead { tr { th { "Item" } th { class: "oxs14-r", "Qty" } th { class: "oxs14-r oxs14-hide", "Rate" } th { class: "oxs14-r", "Amount" } if edit { th { span { class: "oxs12-live", "Actions" } } } } }
                                    tbody {
                                        for l in i.lines.iter().cloned() {
                                            { let lid = l.id; let on = edit && v.edit == Some(lid);
                                              rsx! { tr { key: "{lid}", "data-edit": "{on}",
                                                td { if on { input { class: "oxs14-in", "aria-label": "Item", placeholder: "Describe the work", value: "{l.desc}", oninput: move |e| { iv.write().set_line(lid, Some(&e.value()), None, None); } } } else if edit { button { r#type: "button", class: "oxs14-cellb", onclick: move |_| iv.write().edit = Some(lid), if l.desc.is_empty() { span { class: "oxs12-q", "Describe the work" } } else { "{l.desc}" } } } else { "{l.desc}" } }
                                                td { class: "oxs14-r", if edit { div { class: "oxs14-step oxs14-sm", button { r#type: "button", class: "oxs12-ib", "aria-label": "Less", onclick: move |_| { let mut w = iv.write(); let q = w.current().and_then(|x| x.lines.iter().find(|y| y.id == lid)).map_or(0, |y| y.qty); w.set_line(lid, None, Some(q.saturating_sub(1)), None); }, "−" } span { class: "oxs14-num oxs14-stv", "{l.qty}" } button { r#type: "button", class: "oxs12-ib", "aria-label": "More", onclick: move |_| { let mut w = iv.write(); let q = w.current().and_then(|x| x.lines.iter().find(|y| y.id == lid)).map_or(0, |y| y.qty); w.set_line(lid, None, Some(q + 1), None); }, "+" } } } else { span { class: "oxs14-num", "{l.qty}" } } }
                                                td { class: "oxs14-r oxs14-hide", if on { input { class: "oxs14-in oxs14-num oxs14-r", "aria-label": "Rate in dollars", inputmode: "decimal", value: "{l.unit / 100}", oninput: move |e| { let d = e.value().trim().trim_start_matches('$').replace(',', "").parse::<f64>().unwrap_or(0.0); iv.write().set_line(lid, None, None, Some((d * 100.0).round() as i64)); } } } else { span { class: "oxs14-num", "{money(l.unit)}" } } }
                                                td { class: "oxs14-r oxs14-num", "{money(l.amount())}" }
                                                if edit { td { class: "oxs14-r", button { r#type: "button", class: "oxs12-ib", "aria-label": "Remove {l.desc}", onclick: move |_| { let r = iv.write().remove_line(lid); if let Some(r) = r { said.set(r); } }, {nico("trash")} } } }
                                              } } }
                                        }
                                    }
                                }
                                if edit { button { r#type: "button", class: "oxs12-btn oxs12-sm oxs14-addl", onclick: move |_| { let r = iv.write().add_line(); if let Some(r) = r { said.set(r); } }, {nico("plus")} span { class: "oxs12-btl", "Add line" } } }
                                dl { class: "oxs14-tot",
                                    div { dt { "Subtotal" } dd { class: "oxs14-num", "{money(i.subtotal())}" } }
                                    div { dt { if i.tax > 0 { "Sales tax 8.25%" } else { "Sales tax" } } dd { class: "oxs14-num", "{money(i.tax_amount())}" } }
                                    div { class: "oxs14-grand", dt { "Total due" } dd { class: "oxs14-num", "{money(i.total())}" } }
                                }
                                if edit { p { id: "oxs14-ivchk", class: "oxs14-chk", "data-ok": "{check.is_none()}", role: "status", if let Some(e) = check { {nico("help")} span { "{e}" } } else { {nico("check")} span { "Ready to send" } } } }
                                p { class: "oxs12-q oxs14-fine", "Pay by bank transfer or card. Thank you for working with Northwyn Studio." }
                            }
                          }
                        } }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── sign-in ──

const EMAIL: &str = "email";

/// Screen – Sign-in split: the form on the left (Google and GitHub with their official marks,
/// email and password with checks, show password, a wrong-password count and a lock after three
/// tries, a demo account to fill), an art panel on the right that steps aside on narrow widths.
/// States: `sample`, `loading`, `empty`, `invalid`, `wrong`, `signing`.
#[component]
pub fn SignInSplitScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc3_split(&state);
    let mut said = use_signal(String::new);
    let mut f = use_signal(move || {
        let mut s = SignIn::default();
        match (st, own) {
            (ScreenState::Sample, "invalid") => {
                s.email = "rowan@harrowgate".into();
                s.pass = "fern".into();
                let _ = s.submit();
            }
            (ScreenState::Sample, "wrong") => {
                s.email = DEMO_EMAIL.into();
                s.pass = "fern-lantern-24".into();
                let _ = s.submit();
            }
            (ScreenState::Sample, "signing") => {
                s.sso(1);
            }
            (ScreenState::Sample, _) => s.email = DEMO_EMAIL.into(),
            _ => {}
        }
        s
    });
    let fv = f();
    let loading = st == ScreenState::Loading;
    let e_err = if fv.touched {
        email_error(&fv.email)
    } else {
        None
    };
    let p_err = if fv.touched {
        pass_error(&fv.pass)
    } else {
        None
    };
    let busy = !fv.busy.is_empty();
    let locked = fv.locked();
    rsx! {
        div { class: "oxs12 oxs14-si", "data-layout": "{layout}",
            div { class: "oxs14-sif",
                header { class: "oxs14-sih",
                    span { class: "oxs12-wsm", "aria-hidden": "true", svg { view_box: "0 0 20 20", width: "14", height: "14", path { d: "M4 16 V4 M16 16 V4 M4 10 H16", fill: "none", stroke: "currentColor", stroke_width: "2.4", stroke_linecap: "round" } } }
                    b { "Harrowgate" }
                    span { class: "oxs12-grow" }
                    span { class: "oxs12-q oxs14-hide2", "No account?" }
                    button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| said.set("Sign up".to_string()), "Sign up" }
                }
                if loading {
                    div { class: "oxs14-sibox", {skl(9, "")} }
                } else if fv.done {
                    div { class: "oxs14-sibox", role: "status",
                        span { class: "oxs12-none-g", {nico("check")} }
                        h1 { class: "oxs14-sit", "You're in" }
                        p { class: "oxs12-q", "Signed in as {DEMO_EMAIL}. Opening your workspace." }
                    }
                } else {
                    form { class: "oxs14-sibox", "aria-label": "Sign in",
                        onsubmit: move |e: FormEvent| { e.prevent_default(); let r = f.write().submit(); said.set(match r { Ok(s) | Err(s) => s }); },
                        h1 { class: "oxs14-sit", "Welcome back" }
                        p { class: "oxs12-q", "Sign in to your Harrowgate workspace." }
                        div { class: "oxs14-sso",
                            for (i, (id, l)) in SSO.iter().copied().enumerate() {
                                { let me = fv.busy == id;
                                  rsx! { button { key: "{id}", r#type: "button", class: "oxs12-btn oxs14-ssob", disabled: busy && !me, "aria-busy": "{me}", onclick: move |_| { if me { let r = f.write().finish(); if let Some(r) = r { said.set(r); } } else { let r = f.write().sso(i); said.set(r); } },
                                    if me { span { class: "oxs14-spin", "aria-hidden": "true" } } else { BrandIcon { name: id.to_string(), size: 16.0, coloured: id != "github" } }
                                    span { class: "oxs12-btl", if me { "Continuing with {l}…" } else { "Continue with {l}" } }
                                  } } }
                            }
                        }
                        div { class: "oxs14-or", span { "or" } }
                        label { class: "oxs14-lbl", r#for: "oxs14-em", "Email" }
                        input { id: "oxs14-em", class: "oxs14-fld", r#type: "text", inputmode: "email", autocomplete: "email", placeholder: "name@company.com", value: "{fv.email}", disabled: busy, "aria-invalid": "{e_err.is_some()}", "aria-describedby": "oxs14-eme",
                            oninput: move |e| { let mut w = f.write(); w.email = e.value(); w.error = None; } }
                        p { id: "oxs14-eme", class: "oxs14-fe", if let Some(e) = e_err { "{e}" } }
                        div { class: "oxs14-lblr", label { class: "oxs14-lbl", r#for: "oxs14-pw", "Password" } span { class: "oxs12-grow" } button { r#type: "button", class: "oxs14-lnk", onclick: move |_| said.set("Reset link sent".to_string()), "Forgot password?" } }
                        div { class: "oxs14-pwr",
                            input { id: "oxs14-pw", class: "oxs14-fld", r#type: if fv.show { "text" } else { "password" }, autocomplete: "current-password", placeholder: "At least 8 characters", value: "{fv.pass}", disabled: busy, "aria-invalid": "{p_err.is_some()}", "aria-describedby": "oxs14-pwe",
                                oninput: move |e| { let mut w = f.write(); w.pass = e.value(); w.error = None; } }
                            button { r#type: "button", class: "oxs12-ib oxs14-eye", "aria-pressed": "{fv.show}", "aria-label": if fv.show { "Hide password" } else { "Show password" }, onclick: move |_| { let mut w = f.write(); w.show = !w.show; }, {nico(if fv.show { "eyeoff" } else { "eye" })} }
                        }
                        p { id: "oxs14-pwe", class: "oxs14-fe", if let Some(e) = p_err { "{e}" } }
                        if let Some(e) = fv.error.clone() { p { class: "oxs14-err", role: "alert", {nico(if locked { "lock" } else { "help" })} span { "{e}" } } }
                        button { r#type: "submit", class: "oxs12-btn oxs14-go", "data-pri": "true", disabled: busy || locked, "aria-busy": "{fv.busy == EMAIL}",
                            if fv.busy == "email" { span { class: "oxs14-spin", "aria-hidden": "true" } "Signing in…" } else { "Sign in" } }
                        if st == ScreenState::Sample {
                            div { class: "oxs14-demo",
                                span { class: "oxs12-an", b { "Demo account" } span { class: "oxs12-ell", "{DEMO_EMAIL}" } }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { let r = f.write().fill(); said.set(r); }, "Fill in" }
                            }
                        }
                        p { class: "oxs12-q oxs14-fine", "By continuing you agree to the Terms (2026-09-01) and the Privacy notice." }
                    }
                }
            }
            aside { class: "oxs14-art", "aria-hidden": "true",
                svg { class: "oxs14-artg", view_box: "0 0 400 520", preserve_aspect_ratio: "xMidYMid slice",
                    for r in 0..9 { { let rr = 40 + r * 34; rsx! { circle { key: "{r}", cx: "300", cy: "140", r: "{rr}" } } } }
                }
                figure { class: "oxs14-quote",
                    blockquote { "Every client file, invoice and handoff in one place. We stopped chasing threads." }
                    figcaption { b { "Mira Okafor" } span { "Studio lead, Larkspur Studio" } }
                }
            }
            div { class: "oxs12-live", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
