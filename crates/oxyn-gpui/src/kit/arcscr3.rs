//! Screens S14 (Arc screens III) in GPUI: [`TradingDeskEl`], [`ProductListEl`],
//! [`ProductDetailEl`] and [`InvoiceDeskEl`] on the Arc frame of S12, and [`SignInSplitEl`],
//! over `oxyn_core::kit::arcscr3`.
//!
//! Each element draws one state (`.state()`, `.extra()`); with `.arc3(Arc3Ui)` it draws that
//! live state instead of the sample, and its presses (`mk-*`, `pl-*`, `pd-*`, `iv-*`, `si-*`) go
//! back through `on_act` for the host to apply with
//! [`oxyn_core::kit::arcscr3::Arc3Ui::press`].

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::kit::arcscr3::{
    Arc3Ui, CLIENTS, COLLS, COLOURS, DEMO_EMAIL, IV_STATUS, MK_RANGES, SIZES, SORTS, SSO, SYMBOLS,
    ScreenState, Side, TERMS, change, email_error, ids, inv_no, iso, money, money0, pass_error,
    pct, series, stock_word, up,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::hit;
use super::arcscr::{
    Frame, btn, card, chip, frame, glyph, group, hue, icon_for, none, nrow, q, seg, skel, tint,
};
use super::chatscr::{clip, mark};
use super::rulerfid::{ThPal, th_pal};

macro_rules! arc3_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
            arc: Option<Arc3Ui>,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK, arc: None }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state (see `oxyn_core::kit::arcscr3::s14_states`).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`nav-<key>`, `acct`, and the screen's own `mk-*`, `pl-*`, `pd-*`,
            /// `iv-*`, `si-*`).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
            /// Draw this live state instead of the sample.
            pub fn arc3(mut self, a: Arc3Ui) -> Self {
                self.arc = Some(a);
                self
            }
        }
    };
}

arc3_el!(
    /// Trading desk: the watchlist, the live price and chart, positions, orders and the
    /// order ticket.
    TradingDeskEl
);
arc3_el!(
    /// Product list: collections, colour and price filters, a sort, grid or list.
    ProductListEl
);
arc3_el!(
    /// Product detail: colours, sizes with stock, quantity, Add to cart and the cart.
    ProductDetailEl
);
arc3_el!(
    /// Invoice desk: invoices by status, the paper with line items and totals, the details.
    InvoiceDeskEl
);
arc3_el!(
    /// Sign-in split: SSO, email and password with checks, the art panel.
    SignInSplitEl
);

fn live(a: &Option<Arc3Ui>) -> Arc3Ui {
    a.clone().unwrap_or_default()
}

fn delta(p: &ThPal, bp: i64) -> Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .text_size(px(12.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(hue(p, if bp >= 0 { "green" } else { "rose" }))
        .child(pct(bp))
}

fn badge(p: &ThPal, h: &str, t: impl Into<SharedString>, size: f32) -> Div {
    div()
        .flex_none()
        .size(px(size))
        .rounded(px(7.))
        .flex()
        .items_center()
        .justify_center()
        .bg(tint(p, h, 0.18))
        .text_color(hue(p, h))
        .text_size(px(12.))
        .font_weight(FontWeight::BOLD)
        .child(t.into())
}

fn two(p: &ThPal, a: impl Into<SharedString>, b: impl Into<SharedString>) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(div().font_weight(FontWeight::MEDIUM).child(clip(a.into())))
        .child(
            div()
                .text_size(px(11.5))
                .text_color(p.mute)
                .child(clip(b.into())),
        )
}

fn h3(p: &ThPal, t: &'static str) -> Div {
    div()
        .mt(px(12.))
        .mb(px(6.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_size(px(13.5))
        .text_color(p.fg)
        .child(t)
}

fn row(p: &ThPal) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(9.))
        .min_h(px(40.))
        .border_b_1()
        .border_color(p.line)
}

fn kv(p: &ThPal, k: impl Into<SharedString>, v: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .min_h(px(28.))
        .child(q(p, k.into()))
        .child(div().flex_1())
        .child(div().font_weight(FontWeight::MEDIUM).child(v.into()))
}

fn swatch(c: usize, size: f32) -> Div {
    let hex = u32::from_str_radix(COLOURS[c % COLOURS.len()].1.trim_start_matches('#'), 16)
        .unwrap_or(0x888888);
    div()
        .flex_none()
        .size(px(size))
        .rounded_full()
        .border_1()
        .border_color(gpui::hsla(0., 0., 0.5, 0.45))
        .bg(rgb(hex))
}

fn shot(p: &ThPal, colour: usize, h: f32) -> Div {
    let hex = u32::from_str_radix(
        COLOURS[colour % COLOURS.len()].1.trim_start_matches('#'),
        16,
    )
    .unwrap_or(0x888888);
    div()
        .w_full()
        .h(px(h))
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(rgb(hex))
}

fn stepper(p: &ThPal, act: &Option<Act>, down: &'static str, up: &'static str, v: String) -> Div {
    let b = |id: &'static str, t: &'static str| {
        hit(div(), act, id)
            .size(px(26.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(6.))
            .child(t)
    };
    div()
        .flex_none()
        .h(px(32.))
        .px(px(2.))
        .flex()
        .items_center()
        .gap(px(2.))
        .rounded(px(8.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(b(down, "−"))
        .child(div().min_w(px(40.)).flex().justify_center().child(v))
        .child(b(up, "+"))
}

fn err(p: &ThPal, t: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(12.5))
        .text_color(hue(p, "rose"))
        .child(t.into())
}

// ───────────────────────────────────────────────────────────── market ──

impl RenderOnce for TradingDeskEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.mk = oxyn_core::kit::arcscr3::Market::new(st != ScreenState::Empty);
            match self.extra {
                "limit" => {
                    u.mk.ticket.limit = true;
                    u.mk.ticket.qty = 15;
                    u.mk.ticket.price = 18_400;
                }
                "placed" => {
                    let _ = u.mk.place();
                }
                "closed" => u.mk.open = false,
                _ => {}
            }
        }
        let m = &u.mk;
        let s = m.sym;
        let price = m.price();
        let (dc, dbp) = change(s, price);
        let mut ser = series(s, m.range);
        if m.range == 0
            && let Some(l) = ser.last_mut()
        {
            *l = price;
        }
        let h = if up(&ser) { "green" } else { "rose" };
        let lo = ser.iter().copied().min().unwrap_or(0);
        let hi = ser.iter().copied().max().unwrap_or(1);
        let span = (hi - lo).max(1) as f32;
        let mut side = div().flex().flex_col().gap(px(1.));
        side = side
            .child(nrow(
                &p,
                &act,
                "nav-market",
                glyph(&p, icon_for("chart")),
                "Trade",
                "",
                true,
            ))
            .child(nrow(
                &p,
                &act,
                "nav-trade",
                glyph(&p, icon_for("list")),
                "Orders",
                m.orders.len().to_string(),
                false,
            ))
            .child(group(&p, "Watchlist · 7 symbols"));
        for (i, (tk, name, _, hh)) in SYMBOLS.iter().copied().enumerate() {
            let pr = m.price_of(i);
            let (_, bp) = change(i, pr);
            side = side.child(
                hit(div(), &act, ids::at(ids::MK_SYM, i))
                    .flex_none()
                    .h(px(44.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .rounded(px(8.))
                    .when(i == s, |d| d.bg(p.sel))
                    .child(badge(&p, hh, &tk[..1], 26.))
                    .child(two(&p, tk, name))
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .flex_col()
                            .items_end()
                            .text_size(px(12.))
                            .child(money(pr))
                            .child(delta(&p, bp)),
                    ),
            );
        }
        let ctx = div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(q(&p, "Account value"))
            .child(
                div()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(money(m.value())),
            )
            .child(delta(&p, m.day_bp()));
        let actions = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(q(&p, if m.open { "● Live" } else { "Closed" }))
            .child(btn(&p, &act, "mk-tick", "Next tick", false));
        let body = if st == ScreenState::Loading {
            div().p(px(16.)).child(skel(&p, 10))
        } else {
            let mut bars = div().h(px(200.)).flex().items_end().gap(px(1.)).p(px(10.));
            for x in &ser {
                let f = 0.08 + (*x - lo) as f32 / span * 0.9;
                bars = bars.child(
                    div()
                        .flex_1()
                        .h(relative(f))
                        .rounded(px(1.))
                        .bg(tint(&p, h, 0.55)),
                );
            }
            let mut ranges = Vec::new();
            for (i, (l, _)) in MK_RANGES.iter().copied().enumerate() {
                ranges.push(chip(&p, &act, ids::at(ids::MK_RANGE, i), l, i == m.range));
            }
            let mut pos = div().flex().flex_col().child(h3(&p, "Positions"));
            let ps = m.positions();
            if ps.is_empty() {
                pos = pos.child(none(
                    &p,
                    "No positions yet",
                    "Fund the account, then buy your first shares from the ticket.",
                ));
            }
            for (ps_, n, c, pr) in ps {
                let ret = if c > 0 { (pr - c) * 10_000 / c } else { 0 };
                pos = pos.child(
                    row(&p)
                        .child(badge(&p, SYMBOLS[ps_].3, &SYMBOLS[ps_].0[..1], 22.))
                        .child(two(&p, SYMBOLS[ps_].0, format!("{n} shares")))
                        .child(div().child(money0(pr * n as i64)))
                        .child(delta(&p, ret)),
                );
            }
            pos = pos.child(h3(&p, "Open orders"));
            for o in &m.orders {
                let (w, wh) = if o.side == Side::Buy {
                    ("Buy", "green")
                } else {
                    ("Sell", "rose")
                };
                pos = pos.child(
                    row(&p)
                        .child(badge(&p, wh, w, 34.))
                        .child(two(
                            &p,
                            format!(
                                "{} {} · {}",
                                o.qty,
                                SYMBOLS[o.sym].0,
                                o.limit.map(money).unwrap_or_else(|| "Market".into())
                            ),
                            o.at.clone(),
                        ))
                        .child(btn(
                            &p,
                            &act,
                            ids::at(ids::MK_CANCEL, o.id as usize),
                            "Cancel",
                            false,
                        )),
                );
            }
            for o in &m.fills {
                pos = pos.child(row(&p).child(q(&p, "✓")).child(two(
                    &p,
                    format!("Bought {} {}", o.qty, SYMBOLS[o.sym].0),
                    o.at.clone(),
                )));
            }
            let t = &m.ticket;
            let check = m.check();
            let mut tkt = card(&p)
                .w(px(290.))
                .flex_none()
                .p(px(14.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(seg(
                    &p,
                    vec![
                        chip(&p, &act, "mk-buy", "Buy", t.side == Side::Buy),
                        chip(&p, &act, "mk-sell", "Sell", t.side == Side::Sell),
                    ],
                ))
                .child(seg(
                    &p,
                    vec![
                        chip(&p, &act, "mk-market", "Market", !t.limit),
                        chip(&p, &act, "mk-limit", "Limit", t.limit),
                    ],
                ))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(q(&p, "Shares"))
                        .child(div().flex_1())
                        .child(stepper(
                            &p,
                            &act,
                            "mk-qty-down",
                            "mk-qty-up",
                            t.qty.to_string(),
                        )),
                );
            if t.limit {
                tkt = tkt.child(
                    div()
                        .flex()
                        .items_center()
                        .child(q(&p, "Limit price"))
                        .child(div().flex_1())
                        .child(stepper(&p, &act, "mk-px-down", "mk-px-up", money(t.price))),
                );
            }
            tkt = tkt.child(kv(&p, "Estimate", money(m.estimate()))).child(kv(
                &p,
                "Cash available",
                money(m.cash),
            ));
            if let Some(e) = check.clone() {
                tkt = tkt.child(err(&p, e));
            }
            let side_w = if t.side == Side::Buy { "Buy" } else { "Sell" };
            tkt = tkt.child(
                hit(div(), &act, "mk-place")
                    .h(px(38.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(check.is_none(), |d| {
                        d.bg(hue(&p, if t.side == Side::Buy { "green" } else { "rose" }))
                            .text_color(p.card)
                    })
                    .when(check.is_some(), |d| d.bg(p.sel).text_color(p.mute))
                    .child(format!("{side_w} {} {}", t.qty, SYMBOLS[s].0)),
            );
            let main = div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .mb(px(10.))
                        .child(badge(&p, SYMBOLS[s].3, &SYMBOLS[s].0[..1], 30.))
                        .child(two(&p, SYMBOLS[s].0, SYMBOLS[s].1))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_end()
                                .child(
                                    div()
                                        .text_size(px(22.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(money(price)),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(hue(&p, if dc >= 0 { "green" } else { "rose" }))
                                        .child(format!(
                                            "{}{} ({})",
                                            if dc >= 0 { "+" } else { "" },
                                            money(dc),
                                            pct(dbp)
                                        )),
                                ),
                        ),
                )
                .when(!m.open, |d| {
                    d.child(q(
                        &p,
                        "The market is closed. Market orders queue for 2026-11-13 09:30 ET.",
                    ))
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .mb(px(8.))
                        .child(seg(&p, ranges))
                        .child(div().flex_1())
                        .child(q(&p, format!("Low {} · High {}", money(lo), money(hi)))),
                )
                .child(card(&p).child(bars))
                .child(pos);
            div()
                .p(px(16.))
                .flex()
                .items_start()
                .gap(px(16.))
                .child(main)
                .child(tkt)
        };
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "market-terminal",
                side,
                ctx: Some(ctx),
                crumbs: vec!["Individual".into(), "Trade".into()],
                meta: if m.open {
                    "Market open · 2026-11-12 14:05 ET".into()
                } else {
                    "Market closed · opens 2026-11-13 09:30 ET".into()
                },
                actions,
                bar: None,
                body,
                foot: None,
                right: None,
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── products ──

fn product_body(p: &ThPal, act: &Option<Act>, u: &Arc3Ui, st: ScreenState) -> Div {
    let c = &u.cat;
    if st == ScreenState::Loading {
        return div().p(px(16.)).child(skel(p, 10));
    }
    if c.items.is_empty() {
        return none(
            p,
            "No products yet",
            "Add your first product or import a catalogue from a spreadsheet.",
        );
    }
    if let Some(pr) = c.current() {
        let mut sizes = div().flex().flex_wrap().gap(px(6.));
        for (i, sz) in SIZES.iter().copied().enumerate() {
            let out = pr.stock[i] == 0;
            let on = c.pick_size == Some(i);
            sizes = sizes.child(
                hit(div(), act, ids::at(ids::PD_SIZE, i))
                    .min_w(px(48.))
                    .h(px(36.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(if on { p.fg } else { p.line })
                    .bg(if on {
                        p.fg
                    } else if out {
                        p.inset
                    } else {
                        p.card
                    })
                    .text_color(if on {
                        p.card
                    } else if out {
                        p.mute
                    } else {
                        p.fg
                    })
                    .child(sz),
            );
        }
        let mut sw = div().flex().gap(px(6.));
        for k in pr.colours.iter().copied() {
            sw = sw.child(
                hit(div(), act, ids::at(ids::PD_COLOUR, k))
                    .size(px(26.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(k == c.pick_colour, |d| d.border_2().border_color(p.fg))
                    .child(swatch(k, 18.)),
            );
        }
        let stock = c.pick_size.map(|s| stock_word(pr.stock[s]));
        let mut buy = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(btn(p, act, "pd-back", "‹ All products", false))
            .child(q(p, COLLS[pr.coll].0))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(pr.name),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(money(pr.price)),
                    )
                    .children(pr.was.map(|w| q(p, format!("was {}", money(w)))))
                    .child(div().flex_1())
                    .child(q(
                        p,
                        format!(
                            "★ {}.{} · {} reviews",
                            pr.rating / 10,
                            pr.rating % 10,
                            pr.reviews
                        ),
                    )),
            )
            .child(div().text_color(p.fg2).child(pr.blurb))
            .child(q(p, format!("Colour · {}", COLOURS[c.pick_colour].0)))
            .child(sw)
            .child(match &stock {
                Some((w, h)) => div().flex().gap(px(6.)).child(q(p, "Size ·")).child(
                    div()
                        .text_size(px(12.5))
                        .text_color(hue(p, h))
                        .child(w.clone()),
                ),
                None => q(p, "Size"),
            })
            .child(sizes)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(stepper(
                        p,
                        act,
                        "pd-qty-down",
                        "pd-qty-up",
                        c.qty.to_string(),
                    ))
                    .child(btn(
                        p,
                        act,
                        "pd-add",
                        format!("Add to cart · {}", money(pr.price * c.qty as i64)),
                        true,
                    )),
            );
        if let Some(e) = &c.error {
            buy = buy.child(err(p, e.clone()));
        }
        if c.cart_count() > 0 {
            let mut cart = card(p).p(px(12.)).flex().flex_col().gap(px(6.)).child(kv(
                p,
                format!("Cart · {} items", c.cart_count()),
                money(c.cart_total()),
            ));
            for (id, col, sz, n) in &c.cart {
                let nm = c.items.iter().find(|x| x.id == *id).map_or("", |x| x.name);
                cart = cart.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(swatch(*col, 14.))
                        .child(clip(format!("{n} × {nm}")))
                        .child(q(p, format!("{} · {}", COLOURS[*col].0, SIZES[*sz]))),
                );
            }
            buy = buy.child(cart);
        }
        return div()
            .p(px(20.))
            .flex()
            .gap(px(24.))
            .child(
                div()
                    .w(relative(0.45))
                    .flex_none()
                    .child(shot(p, c.pick_colour, 360.)),
            )
            .child(buy);
    }
    let shown = c.shown();
    let mut list = div().p(px(16.)).flex().flex_col().gap(px(12.)).child(q(
        p,
        format!("{} of {} products", shown.len(), c.items.len()),
    ));
    if shown.is_empty() {
        return list.child(none(
            p,
            "No products match",
            "Try another colour or price, or clear the filters.",
        ));
    }
    if c.list {
        for pr in shown {
            let (w, h) = stock_word(pr.total());
            list = list.child(
                hit(row(p), act, ids::at(ids::PL_OPEN, pr.id as usize))
                    .child(div().w(px(40.)).child(shot(p, pr.colours[0], 32.)))
                    .child(two(p, pr.name, COLLS[pr.coll].0))
                    .child(div().text_size(px(12.)).text_color(hue(p, h)).child(w))
                    .child(div().w(px(90.)).flex().justify_end().child(money(pr.price))),
            );
        }
        return list;
    }
    let mut grid = div().flex().flex_wrap().gap(px(14.));
    for pr in shown {
        let (w, h) = stock_word(pr.total());
        let mut sws = div().flex().gap(px(4.));
        for k in pr.colours.iter().copied() {
            sws = sws.child(swatch(k, 12.));
        }
        grid = grid.child(
            hit(div(), act, ids::at(ids::PL_OPEN, pr.id as usize))
                .w(px(200.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(shot(p, pr.colours[0], 150.))
                .child(clip(pr.name))
                .child(
                    div()
                        .flex()
                        .gap(px(6.))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(money(pr.price)),
                        )
                        .children(pr.was.map(|x| q(p, format!("was {}", money(x))))),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(sws)
                        .child(div().flex_1())
                        .child(div().text_size(px(12.)).text_color(hue(p, h)).child(w)),
                ),
        );
    }
    list.child(grid)
}

fn product_render(
    p: &ThPal,
    act: &Option<Act>,
    lay: ShellLayout,
    u: &Arc3Ui,
    st: ScreenState,
    slug: &'static str,
) -> Div {
    let c = &u.cat;
    let mut side = div()
        .flex()
        .flex_col()
        .gap(px(1.))
        .child(nrow(
            p,
            act,
            "nav-products",
            glyph(p, icon_for("tag")),
            "Products",
            c.items.len().to_string(),
            true,
        ))
        .child(nrow(
            p,
            act,
            "nav-product",
            glyph(p, icon_for("box")),
            "Product page",
            "",
            false,
        ))
        .child(group(p, "Collections"));
    for (i, (n, h)) in COLLS.iter().copied().enumerate() {
        side = side.child(nrow(
            p,
            act,
            ids::at(ids::PL_COLL, i),
            div()
                .w(px(16.))
                .flex()
                .justify_center()
                .child(div().size(px(7.)).rounded_full().bg(hue(p, h))),
            n,
            c.items.iter().filter(|x| x.coll == i).count().to_string(),
            c.coll == Some(i),
        ));
    }
    let ctx = div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(q(p, "Sales this week"))
        .child(
            div()
                .text_size(px(17.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("$8,420"),
        )
        .child(delta(p, 640));
    let actions = div().flex().gap(px(8.)).child(btn(
        p,
        act,
        "pl-cart",
        format!("Cart {}", c.cart_count()),
        false,
    ));
    let bar = if c.current().is_some() {
        None
    } else {
        let mut sw = div().flex().gap(px(4.));
        for k in 0..COLOURS.len() {
            sw = sw.child(
                hit(div(), act, ids::at(ids::PL_COLOUR, k))
                    .size(px(24.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(c.colour == Some(k), |d| d.border_2().border_color(p.fg))
                    .child(swatch(k, 16.)),
            );
        }
        Some(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(sw)
                .child(chip(
                    p,
                    act,
                    "pl-max",
                    match c.max {
                        Some(m) => format!("Under {}", money0(m)),
                        None => "Any price".into(),
                    },
                    c.max.is_some(),
                ))
                .child(div().flex_1())
                .child(btn(p, act, "pl-sort", SORTS[c.sort].1, false))
                .child(seg(
                    p,
                    vec![chip(
                        p,
                        act,
                        "pl-view",
                        if c.list { "List" } else { "Grid" },
                        true,
                    )],
                )),
        )
    };
    let mut crumbs = vec!["Fellside Goods".to_string(), "Products".to_string()];
    if let Some(pr) = c.current() {
        crumbs.push(pr.name.to_string());
    }
    frame(
        p,
        lay,
        act,
        Frame {
            slug,
            side,
            ctx: Some(ctx),
            crumbs,
            meta: format!("{} products", c.items.len()),
            actions,
            bar,
            body: product_body(p, act, u, st),
            foot: None,
            right: None,
        },
    )
}

fn product_state(a: &Option<Arc3Ui>, st: ScreenState, extra: &str, detail: bool) -> Arc3Ui {
    let mut u = live(a);
    if a.is_none() {
        let mut c = oxyn_core::kit::arcscr3::Catalog::new(st != ScreenState::Empty);
        match extra {
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
            _ if detail => {
                c.open_id(1);
            }
            _ => {}
        }
        u.cat = c;
    }
    u
}

impl RenderOnce for ProductListEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut u = product_state(&self.arc, self.state, self.extra, false);
        if self.arc.is_some() && !u.cat.items.is_empty() && self.extra.is_empty() {
            // The list page shows the list; the product page keeps the open product.
            u.cat.open = None;
        }
        product_render(&p, &self.act, self.layout, &u, self.state, "product-list")
    }
}

impl RenderOnce for ProductDetailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let u = product_state(&self.arc, self.state, self.extra, true);
        product_render(&p, &self.act, self.layout, &u, self.state, "product-detail")
    }
}

// ───────────────────────────────────────────────────────────── invoice ──

impl RenderOnce for InvoiceDeskEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.iv = oxyn_core::kit::arcscr3::Invoice::new(st != ScreenState::Empty);
            match self.extra {
                "editing" => {
                    u.iv.add_line();
                    u.iv.set_line(4, Some("Rush turnaround"), Some(1), Some(25_000));
                }
                "preview" => u.iv.preview = true,
                "sent" => {
                    let _ = u.iv.send();
                }
                _ => {}
            }
        }
        let v = &u.iv;
        let mut side = div().flex().flex_col().gap(px(1.)).child(nrow(
            &p,
            &act,
            "nav-invoicestudio",
            glyph(&p, icon_for("file")),
            "All invoices",
            v.list.len().to_string(),
            true,
        ));
        for (i, (l, h)) in IV_STATUS.iter().copied().enumerate() {
            side = side.child(
                div()
                    .h(px(32.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .text_color(p.fg2)
                    .child(
                        div()
                            .w(px(16.))
                            .flex()
                            .justify_center()
                            .child(div().size(px(7.)).rounded_full().bg(hue(&p, h))),
                    )
                    .child(clip(l))
                    .child(q(&p, v.count(i).to_string())),
            );
        }
        let ctx = div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(q(&p, "Collected in November"))
            .child(
                div()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("$11,759"),
            )
            .child(q(&p, "2 payments received"));
        let edit = v.editable() && !v.preview;
        let mut actions = div().flex().gap(px(8.));
        if v.current().is_some() {
            actions = actions.child(btn(
                &p,
                &act,
                "iv-preview",
                if v.preview { "Edit" } else { "Preview" },
                false,
            ));
            if v.editable() {
                actions = actions.child(btn(&p, &act, "iv-send", "Send invoice", true));
            }
        }
        actions = actions.child(btn(&p, &act, "iv-new", "New invoice", false));
        let body = if st == ScreenState::Loading {
            div().p(px(16.)).child(skel(&p, 10))
        } else if v.list.is_empty() {
            none(
                &p,
                "No invoices yet",
                "Create an invoice, add line items and send it to a client.",
            )
        } else {
            let mut list = div()
                .w(px(250.))
                .flex_none()
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .border_r_1()
                .border_color(p.line);
            for g in [2usize, 1, 0, 3] {
                let rows: Vec<_> = v.list.iter().filter(|x| x.status == g).collect();
                if rows.is_empty() {
                    continue;
                }
                list = list.child(div().mt(px(6.)).px(px(8.)).child(q(&p, IV_STATUS[g].0)));
                for x in rows {
                    let id = ids::IV_OPEN
                        .get((x.no as usize).saturating_sub(1040))
                        .copied()
                        .unwrap_or("iv-open-1040");
                    list = list.child(
                        hit(div(), &act, id)
                            .min_h(px(46.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .rounded(px(8.))
                            .when(Some(x.no) == v.open, |d| d.bg(p.sel))
                            .child(two(
                                &p,
                                CLIENTS[x.client].0,
                                format!("{} · due {}", inv_no(x.no), iso(x.due())),
                            ))
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(money0(x.total())),
                            ),
                    );
                }
            }
            let mut paper = card(&p).p(px(24.)).flex().flex_col().gap(px(4.));
            if let Some(i) = v.current() {
                let (sl, sh) = IV_STATUS[i.status];
                if v.preview {
                    paper = paper.child(q(
                        &p,
                        format!("Preview: this is what {} sees.", CLIENTS[i.client].0),
                    ));
                }
                if i.status == 1 {
                    paper = paper.child(div().text_color(hue(&p, "green")).child(format!(
                        "Sent to {}. Due {}.",
                        CLIENTS[i.client].1,
                        iso(i.due())
                    )));
                }
                paper = paper
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .mb(px(12.))
                            .child(two(&p, "Northwyn Studio", "14 Ferry Row, Harlow Bay"))
                            .child(badge(&p, sh, sl, 0.).w_auto().h(px(20.)).px(px(7.)))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("Invoice {}", inv_no(i.no))),
                            ),
                    )
                    .child(kv(&p, "Bill to", CLIENTS[i.client].0))
                    .child(kv(&p, "Issued", iso(i.issued)))
                    .child(kv(
                        &p,
                        "Due",
                        format!("{} · {}", iso(i.due()), TERMS[i.terms].0),
                    ));
                for l in &i.lines {
                    let lid = l.id as usize;
                    let mut r =
                        row(&p).child(div().flex_1().min_w_0().child(if l.desc.is_empty() {
                            q(&p, "Describe the work")
                        } else {
                            clip(l.desc.clone())
                        }));
                    if edit {
                        r = r.child(stepper(
                            &p,
                            &act,
                            ids::at(ids::IV_QDOWN, lid),
                            ids::at(ids::IV_QUP, lid),
                            l.qty.to_string(),
                        ));
                    } else {
                        r = r.child(div().child(l.qty.to_string()));
                    }
                    r = r
                        .child(
                            div()
                                .w(px(90.))
                                .flex()
                                .justify_end()
                                .text_color(p.mute)
                                .child(money(l.unit)),
                        )
                        .child(
                            div()
                                .w(px(100.))
                                .flex()
                                .justify_end()
                                .child(money(l.amount())),
                        );
                    if edit {
                        r = r.child(btn(&p, &act, ids::at(ids::IV_RM, lid), "Remove", false));
                    }
                    paper = paper.child(r);
                }
                if edit {
                    paper = paper.child(div().mt(px(8.)).child(btn(
                        &p,
                        &act,
                        "iv-add",
                        "+ Add line",
                        false,
                    )));
                }
                if v.removed.is_some() {
                    paper = paper.child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(q(&p, "Line removed"))
                            .child(btn(&p, &act, "iv-undo", "Undo", false)),
                    );
                }
                paper = paper
                    .child(div().h(px(10.)))
                    .child(kv(&p, "Subtotal", money(i.subtotal())))
                    .child(kv(
                        &p,
                        if i.tax > 0 {
                            "Sales tax 8.25%"
                        } else {
                            "Sales tax"
                        },
                        money(i.tax_amount()),
                    ))
                    .child(kv(&p, "Total due", money(i.total())).font_weight(FontWeight::SEMIBOLD));
                if edit {
                    paper = paper.child(match v.check() {
                        Some(e) => div().text_color(hue(&p, "amber")).child(e),
                        None => div().text_color(hue(&p, "green")).child("Ready to send"),
                    });
                }
            }
            div()
                .flex()
                .size_full()
                .child(list)
                .child(div().flex_1().min_w_0().p(px(16.)).bg(p.inset).child(paper))
        };
        let right = v.current().map(|i| {
            let mut terms = Vec::new();
            for (k, (l, _)) in TERMS.iter().copied().enumerate() {
                terms.push(chip(&p, &act, ids::at(ids::IV_TERMS, k), l, i.terms == k));
            }
            let mut cl = Vec::new();
            for (k, (n, _, _)) in CLIENTS.iter().copied().enumerate() {
                cl.push(chip(&p, &act, ids::at(ids::IV_CLIENT, k), n, i.client == k));
            }
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(h3(&p, "Details"))
                .child(q(&p, "Client"))
                .child(div().flex().flex_wrap().gap(px(4.)).children(cl))
                .child(q(&p, "Payment terms"))
                .child(seg(&p, terms))
                .child(q(&p, format!("Due {}", iso(i.due()))))
                .child(chip(&p, &act, "iv-tax", "Sales tax 8.25%", i.tax > 0))
                .child(chip(&p, &act, "iv-remind", "Automatic reminders", v.remind))
        });
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "invoice-studio",
                side,
                ctx: Some(ctx),
                crumbs: match v.current() {
                    Some(i) => vec!["Invoices".into(), inv_no(i.no)],
                    None => vec!["Invoices".into()],
                },
                meta: format!("{} outstanding", money0(v.outstanding())),
                actions,
                bar: None,
                body,
                foot: None,
                right,
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── sign-in ──

impl RenderOnce for SignInSplitEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            let s = &mut u.si;
            match (st, self.extra) {
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
        }
        let f = &u.si;
        let e_err = if f.touched {
            email_error(&f.email)
        } else {
            None
        };
        let p_err = if f.touched { pass_error(&f.pass) } else { None };
        let field = |label: &'static str, v: String, ph: &'static str, bad: bool| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(p.fg2)
                        .child(label),
                )
                .child(
                    div()
                        .h(px(40.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .rounded(px(9.))
                        .border_1()
                        .border_color(if bad { hue(&p, "rose") } else { p.line })
                        .bg(p.card)
                        .child(if v.is_empty() { q(&p, ph) } else { clip(v) }),
                )
        };
        let mut form = div().w(px(380.)).flex().flex_col().gap(px(8.));
        if st == ScreenState::Loading {
            form = form.child(skel(&p, 9));
        } else if f.done {
            form = form
                .child(
                    div()
                        .text_size(px(24.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("You're in"),
                )
                .child(q(&p, format!("Signed in as {DEMO_EMAIL}.")));
        } else {
            form = form
                .child(
                    div()
                        .text_size(px(24.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Welcome back"),
                )
                .child(q(&p, "Sign in to your Harrowgate workspace."));
            for (i, (id, l)) in SSO.iter().copied().enumerate() {
                let me = f.busy == id;
                form = form.child(
                    hit(div(), &act, ids::at(ids::SI_SSO, i))
                        .h(px(40.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(8.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .font_weight(FontWeight::MEDIUM)
                        .children(if me { None } else { mark(&p, id, 16.) })
                        .child(if me {
                            format!("Continuing with {l}…")
                        } else {
                            format!("Continue with {l}")
                        }),
                );
            }
            let shown = if f.show {
                f.pass.clone()
            } else {
                "•".repeat(f.pass.chars().count())
            };
            form = form
                .child(div().flex().justify_center().child(q(&p, "or")))
                .child(field(
                    "Email",
                    f.email.clone(),
                    "name@company.com",
                    e_err.is_some(),
                ))
                .children(e_err.map(|e| err(&p, e)))
                .child(field(
                    "Password",
                    shown,
                    "At least 8 characters",
                    p_err.is_some(),
                ))
                .children(p_err.map(|e| err(&p, e)))
                .child(div().flex().gap(px(8.)).child(chip(
                    &p,
                    &act,
                    "si-show",
                    if f.show {
                        "Hide password"
                    } else {
                        "Show password"
                    },
                    f.show,
                )))
                .children(f.error.clone().map(|e| err(&p, e)))
                .child(btn(
                    &p,
                    &act,
                    "si-submit",
                    if f.busy == "email" {
                        "Signing in…"
                    } else {
                        "Sign in"
                    },
                    true,
                ))
                .child(
                    card(&p)
                        .p(px(10.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(two(&p, "Demo account", DEMO_EMAIL))
                        .child(btn(
                            &p,
                            &act,
                            if f.busy.is_empty() {
                                "si-fill"
                            } else {
                                "si-finish"
                            },
                            if f.busy.is_empty() {
                                "Fill in"
                            } else {
                                "Finish"
                            },
                            false,
                        )),
                );
        }
        let flat = self.layout == ShellLayout::Flat;
        div()
            .size_full()
            .flex()
            .bg(p.inset)
            .text_color(p.fg)
            .text_size(px(13.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .bg(p.card)
                    .child(
                        div()
                            .h(px(60.))
                            .px(px(24.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(badge(&p, "mute", "H", 22.))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("Harrowgate"))
                            .child(div().flex_1())
                            .child(q(&p, "No account?"))
                            .child(btn(&p, &act, "si-signup", "Sign up", false)),
                    )
                    .child(div().flex_1().flex().items_center().justify_center().child(form)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .when(!flat, |d| d.m(px(8.)).rounded(px(14.)))
                    .bg(p.sel)
                    .flex()
                    .items_end()
                    .p(px(28.))
                    .child(
                        card(&p)
                            .p(px(20.))
                            .flex()
                            .flex_col()
                            .gap(px(10.))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Every client file, invoice and handoff in one place. We stopped chasing threads."),
                            )
                            .child(two(&p, "Mira Okafor", "Studio lead, Larkspur Studio")),
                    ),
            )
    }
}
