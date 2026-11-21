//! Menu Motion family (variants 2) for GPUI over `oxyn_core::kit::gooeyfid`: **Gooey attach
//! menu** ([`GooeyAttachMenuEl`]), **Gooey model menu** ([`GooeyModelMenuEl`]), **Gooey
//! composer** ([`GooeyComposerEl`]), **Liquid tabs** ([`LiquidTabsEl`]) and **Liquid drops**
//! ([`LiquidDropsEl`]). `RenderOnce` static snapshots of the settled state (GPUI has no goo
//! filter): the host keeps open/closed and routes [`Act`] ids back (`toggle`, `item-<i>`,
//! `tab-<i>`).

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::kit::gooeyfid::{GooItem, goo_attach_items, goo_models};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{ThPal, fire, th_pal};

const ITEM_ID: [&str; 10] = [
    "item-0", "item-1", "item-2", "item-3", "item-4", "item-5", "item-6", "item-7", "item-8",
    "item-9",
];
const TAB_ID: [&str; 6] = ["tab-0", "tab-1", "tab-2", "tab-3", "tab-4", "tab-5"];

fn sheet(
    p: &ThPal,
    items: &[GooItem],
    chosen: Option<usize>,
    w: f32,
    act: &Option<Act>,
) -> gpui::Stateful<gpui::Div> {
    let mut list = div().flex().flex_col().p(px(6.));
    let mut last = "";
    for (i, it) in items.iter().enumerate().take(10) {
        if !it.group.is_empty() && it.group != last {
            list = list.child(
                div()
                    .h(px(24.))
                    .px(px(10.))
                    .text_size(px(11.5))
                    .text_color(p.mute)
                    .child(it.group),
            );
        }
        last = it.group;
        let f = fire(act, ITEM_ID[i]);
        let badge = it
            .icon
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_uppercase())
            .unwrap_or_else(|| it.label.chars().take(1).collect());
        list = list.child(
            div()
                .id(SharedString::from(format!("oxgm-{i}")))
                .flex()
                .items_center()
                .gap(px(10.))
                .h(px(36.))
                .px(px(10.))
                .rounded(px(10.))
                .bg(if i == 0 { p.sel } else { p.card })
                .child(
                    div()
                        .w(px(26.))
                        .text_size(px(9.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(p.fg2)
                        .child(badge),
                )
                .child(div().text_size(px(13.5)).child(it.label))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.5))
                        .text_color(p.mute)
                        .child(it.hint),
                )
                .child(if chosen == Some(i) {
                    div().child("✓")
                } else {
                    div()
                })
                .on_click(move |_, w, cx| f(w, cx)),
        );
    }
    div()
        .id("oxgm-sheet")
        .w(px(w))
        .rounded(px(16.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .text_color(p.fg)
        .child(list)
}

/// `gooey-attach-menu` (Menu Motion – Gooey attach menu): the drop-up, settled open.
#[derive(IntoElement)]
pub struct GooeyAttachMenuEl {
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for GooeyAttachMenuEl {
    fn default() -> Self {
        Self {
            open: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl GooeyAttachMenuEl {
    /// Open or closed.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `toggle`, `item-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for GooeyAttachMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let f = fire(&self.act, "toggle");
        let mut root = div().flex().flex_col().gap(px(10.)).text_color(p.fg);
        if self.open {
            root = root.child(sheet(&p, &goo_attach_items(), None, 280., &self.act));
        }
        root.child(
            div()
                .id("oxgm-plus")
                .size(px(38.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(13.))
                .bg(p.sel)
                .child(if self.open { "×" } else { "+" })
                .on_click(move |_, w, cx| f(w, cx)),
        )
    }
}

/// `gooey-model-menu` (Menu Motion – Gooey model menu): the model list, settled open.
#[derive(IntoElement)]
pub struct GooeyModelMenuEl {
    open: bool,
    model: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for GooeyModelMenuEl {
    fn default() -> Self {
        Self {
            open: true,
            model: 1,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl GooeyModelMenuEl {
    /// Open or closed.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Chosen model index.
    pub fn model(mut self, m: usize) -> Self {
        self.model = m;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `toggle`, `item-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for GooeyModelMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let models = goo_models();
        let m = self.model.min(models.len() - 1);
        let f = fire(&self.act, "toggle");
        let mut root = div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(10.))
            .text_color(p.fg);
        if self.open {
            root = root.child(sheet(&p, &models, Some(m), 268., &self.act));
        }
        root.child(
            div()
                .id("oxgm-chip")
                .w(px(132.))
                .h(px(38.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(13.))
                .bg(p.sel)
                .child(models[m].label)
                .on_click(move |_, w, cx| f(w, cx)),
        )
    }
}

/// `gooey-composer` (Menu Motion – Gooey composer): the composer with its attach menu open.
#[derive(IntoElement)]
pub struct GooeyComposerEl {
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for GooeyComposerEl {
    fn default() -> Self {
        Self {
            open: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl GooeyComposerEl {
    /// Attach menu open.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `toggle`, `item-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for GooeyComposerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let f = fire(&self.act, "toggle");
        let mut root = div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_color(p.fg);
        if self.open {
            root = root.child(sheet(&p, &goo_attach_items(), None, 560., &self.act));
        }
        root.child(
            div()
                .w_full()
                .h(px(120.))
                .p(px(12.))
                .flex()
                .flex_col()
                .justify_between()
                .rounded(px(22.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .child(div().text_color(p.mute).child("Ask anything…"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .id("oxgm-cplus")
                                .size(px(38.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(13.))
                                .bg(p.sel)
                                .child("+")
                                .on_click(move |_, w, cx| f(w, cx)),
                        )
                        .child(div().flex_1())
                        .child(div().px(px(10.)).child(goo_models()[1].label))
                        .child(div().size(px(34.)).rounded(px(17.)).bg(p.sel)),
                ),
        )
    }
}

/// `liquid-tabs` (Menu Motion – Liquid tabs, Oxyn original): the ink pill on the chosen tab.
#[derive(IntoElement)]
pub struct LiquidTabsEl {
    tabs: Vec<SharedString>,
    value: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for LiquidTabsEl {
    fn default() -> Self {
        Self {
            tabs: ["Chat", "Agent", "Plan", "Code"]
                .into_iter()
                .map(SharedString::from)
                .collect(),
            value: 0,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl LiquidTabsEl {
    /// Chosen tab.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `tab-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for LiquidTabsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut row = div()
            .w(px(360.))
            .h(px(40.))
            .p(px(3.))
            .flex()
            .rounded(px(13.))
            .bg(p.sel);
        for (i, t) in self.tabs.iter().enumerate().take(6) {
            let f = fire(&self.act, TAB_ID[i]);
            let on = i == self.value;
            row = row.child(
                div()
                    .id(SharedString::from(format!("oxlt-{i}")))
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.))
                    .bg(if on { p.fg } else { p.sel })
                    .text_color(if on { p.card } else { p.fg2 })
                    .child(t.clone())
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        row
    }
}

/// `liquid-drops` (Menu Motion – Liquid drops, Oxyn original): the actions, settled open.
#[derive(IntoElement)]
pub struct LiquidDropsEl {
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for LiquidDropsEl {
    fn default() -> Self {
        Self {
            open: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl LiquidDropsEl {
    /// Open or closed.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `toggle`, `item-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for LiquidDropsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut col = div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(14.))
            .text_color(p.fg);
        if self.open {
            for (i, l) in ["Paste a link", "Voice note", "Screenshot", "Upload a file"]
                .into_iter()
                .enumerate()
            {
                let f = fire(&self.act, ITEM_ID[3 - i]);
                col = col.child(
                    div()
                        .id(SharedString::from(format!("oxld-{i}")))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .mr(px(8.))
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(6.))
                                .rounded(px(9.))
                                .bg(p.card)
                                .border_1()
                                .border_color(p.line)
                                .child(l),
                        )
                        .child(div().size(px(40.)).rounded(px(20.)).bg(p.fg))
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
        }
        let f = fire(&self.act, "toggle");
        col.child(
            div()
                .id("oxld-fab")
                .size(px(56.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(28.))
                .bg(p.fg)
                .text_color(p.card)
                .text_size(px(22.))
                .child(if self.open { "×" } else { "+" })
                .on_click(move |_, w, cx| f(w, cx)),
        )
    }
}
