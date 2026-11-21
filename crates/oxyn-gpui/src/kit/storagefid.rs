//! Storage family (variants 2) for GPUI over `oxyn_core::kit::storagefid`: **Storage meter**
//! ([`StorageMeterEl`]), **Storage cleanup** ([`StorageCleanupEl`]), **Storage waffle**
//! ([`StorageWaffleEl`]) and **Storage forecast** ([`StorageForecastEl`]). `RenderOnce` static
//! snapshots: the host keeps the state and routes [`Act`] ids back (`rescan`, `clear`, `cat-<i>`,
//! `trash`).

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_core::kit::storagefid::{
    CleanItem, StoreCat, clean_demo, col_dots, days_until_full, dot_runs, fmt_gb, idle_line,
    store_demo, store_hex, store_history, store_pace, store_used, unused,
};
use oxyn_tokens::Theme;

use super::Act;
use super::ctxvisfid::hex;
use super::rulerfid::{fire, th_pal};

const CAT_ID: [&str; 9] = [
    "cat-0", "cat-1", "cat-2", "cat-3", "cat-4", "cat-5", "cat-6", "cat-7", "cat-8",
];

fn free_dot(theme: &Theme) -> Hsla {
    if th_pal(theme).card.l < 0.5 {
        hex("2A2A2E")
    } else {
        hex("ECECEE")
    }
}

fn card(theme: &Theme, w: f32) -> gpui::Div {
    let p = th_pal(theme);
    div()
        .w(px(w))
        .p(px(18.))
        .rounded(px(20.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .text_color(p.fg)
        .text_size(px(13.))
}

/// `storage-meter` (Storage – Storage meter): figure, dot bar, category grid.
#[derive(IntoElement)]
pub struct StorageMeterEl {
    cats: Vec<StoreCat>,
    cap: f64,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for StorageMeterEl {
    fn default() -> Self {
        Self {
            cats: store_demo(),
            cap: 512.0,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl StorageMeterEl {
    /// Categories.
    pub fn cats(mut self, c: Vec<StoreCat>) -> Self {
        self.cats = c;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `rescan`, `clear`, `cat-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for StorageMeterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let used = store_used(&self.cats);
        let runs = dot_runs(&self.cats, self.cap, 48 * 4);
        let mut bar = div()
            .flex()
            .gap(px(2.))
            .p(px(6.))
            .rounded(px(12.))
            .border_1()
            .border_color(p.line);
        for c in 0..48 {
            let mut col = div().flex().flex_col().gap(px(2.));
            for r in 0..4 {
                let k = runs[c * 4 + r];
                let d = div().size(px(5.5)).rounded(px(2.));
                col = col.child(match k {
                    Some(i) => d.bg(hex(store_hex(self.cats[i].key))),
                    None => d.border_1().border_color(free_dot(self.theme)),
                });
            }
            bar = bar.child(col);
        }
        let mut grid = div().flex().flex_wrap().gap(px(4.)).mt(px(14.));
        for (i, c) in self.cats.iter().enumerate() {
            let f = fire(&self.act, CAT_ID[i.min(8)]);
            grid = grid.child(
                div()
                    .id(SharedString::from(format!("oxsm-{i}")))
                    .w(px(118.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .p(px(7.))
                    .rounded(px(10.))
                    .child(
                        div()
                            .size(px(22.))
                            .rounded(px(7.))
                            .bg(hex(store_hex(c.key)).opacity(0.2)),
                    )
                    .child(
                        div().flex().flex_col().child(c.label).child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.mute)
                                .child(fmt_gb(c.gb)),
                        ),
                    )
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let fr = fire(&self.act, "rescan");
        card(self.theme, 420.)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(30.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(fmt_gb(used)),
                            )
                            .child(
                                div()
                                    .text_color(p.mute)
                                    .pb(px(5.))
                                    .child(format!("of {} used", fmt_gb(self.cap))),
                            ),
                    )
                    .child(
                        div()
                            .id("oxsm-rescan")
                            .h(px(30.))
                            .px(px(13.))
                            .flex()
                            .items_center()
                            .rounded(px(15.))
                            .border_1()
                            .border_color(p.line)
                            .child("Rescan")
                            .on_click(move |_, w, cx| fr(w, cx)),
                    ),
            )
            .child(
                div()
                    .mt(px(2.))
                    .mb(px(14.))
                    .text_color(p.fg2)
                    .child(format!("{} available", fmt_gb((self.cap - used).max(0.0)))),
            )
            .child(bar)
            .child(grid)
    }
}

/// `storage-cleanup` (Storage – Storage cleanup): unused skills with checkboxes.
#[derive(IntoElement)]
pub struct StorageCleanupEl {
    items: Vec<CleanItem>,
    days: u32,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for StorageCleanupEl {
    fn default() -> Self {
        Self {
            items: clean_demo(),
            days: 30,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl StorageCleanupEl {
    /// Window in days.
    pub fn days(mut self, d: u32) -> Self {
        self.days = d;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `trash`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for StorageCleanupEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let idx = unused(&self.items, self.days);
        let mut list = div().flex().flex_col().mt(px(10.));
        for i in &idx {
            let it = &self.items[*i];
            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .child(div().size(px(14.)).rounded(px(4.)).bg(p.fg))
                    .child(div().flex_1().child(it.name))
                    .child(div().text_color(p.mute).child(idle_line(it.idle)))
                    .child(
                        div()
                            .w(px(34.))
                            .text_color(p.mute)
                            .child(it.tokens.to_string()),
                    ),
            );
        }
        let f = fire(&self.act, "trash");
        card(self.theme, 440.)
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Free up context"),
            )
            .child(
                div()
                    .text_color(p.mute)
                    .child("Skills and commands that ride along with each new chat."),
            )
            .child(list)
            .child(
                div()
                    .id("oxsc-trash")
                    .mt(px(12.))
                    .h(px(32.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .rounded(px(10.))
                    .bg(p.fg)
                    .text_color(p.card)
                    .child(format!("Move {} to Trash", idx.len()))
                    .on_click(move |_, w, cx| f(w, cx)),
            )
    }
}

/// `storage-waffle` (Storage – Storage waffle, Oxyn original): 200 dots and a legend.
#[derive(IntoElement)]
pub struct StorageWaffleEl {
    cats: Vec<StoreCat>,
    cap: f64,
    theme: &'static Theme,
}

impl Default for StorageWaffleEl {
    fn default() -> Self {
        Self {
            cats: store_demo(),
            cap: 512.0,
            theme: &Theme::DARK,
        }
    }
}

impl StorageWaffleEl {
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
}

impl RenderOnce for StorageWaffleEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let runs = dot_runs(&self.cats, self.cap, 200);
        let mut grid = div().flex().flex_col().gap(px(2.));
        for r in 0..10 {
            let mut row = div().flex().gap(px(2.));
            for c in 0..20 {
                let d = div().size(px(8.5)).rounded(px(5.));
                row = row.child(match runs[r * 20 + c] {
                    Some(i) => d.bg(hex(store_hex(self.cats[i].key))),
                    None => d.bg(free_dot(self.theme)),
                });
            }
            grid = grid.child(row);
        }
        let mut legend = div().flex().flex_col().gap(px(1.)).flex_1();
        for c in &self.cats {
            legend = legend.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(22.))
                    .child(div().size(px(9.)).rounded(px(5.)).bg(hex(store_hex(c.key))))
                    .child(div().flex_1().child(c.label))
                    .child(div().text_color(p.mute).child(fmt_gb(c.gb))),
            );
        }
        card(self.theme, 440.)
            .child(div().text_color(p.mute).child("Disk"))
            .child(
                div()
                    .mb(px(14.))
                    .text_size(px(20.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!(
                        "{} of {} used",
                        fmt_gb(store_used(&self.cats)),
                        fmt_gb(self.cap)
                    )),
            )
            .child(div().flex().gap(px(16.)).child(grid).child(legend))
    }
}

/// `storage-forecast` (Storage – Storage forecast, Oxyn original): dot columns and days to full.
#[derive(IntoElement)]
pub struct StorageForecastEl {
    history: Vec<f64>,
    cap: f64,
    theme: &'static Theme,
}

impl Default for StorageForecastEl {
    fn default() -> Self {
        Self {
            history: store_history(),
            cap: 512.0,
            theme: &Theme::DARK,
        }
    }
}

impl StorageForecastEl {
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
}

impl RenderOnce for StorageForecastEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let pace = store_pace(&self.history);
        let last = self.history.last().copied().unwrap_or(0.0);
        let mut chart = div().flex().items_end().gap(px(3.));
        for g in &self.history {
            let lit = col_dots(*g, self.cap, 12);
            let mut col = div().flex().flex_col_reverse().gap(px(3.));
            for r in 0..12 {
                let d = div().size(px(11.)).rounded(px(6.));
                col = col.child(if r < lit {
                    d.bg(if r >= 10 { p.red } else { p.fg2 })
                } else {
                    d.bg(free_dot(self.theme))
                });
            }
            chart = chart.child(col);
        }
        let sum = match days_until_full(last, self.cap, pace) {
            Some(d) => format!("Full in about {d} days at {} a day", fmt_gb(pace)),
            None => "Not growing".into(),
        };
        card(self.theme, 460.)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Disk forecast"),
            )
            .child(div().mb(px(14.)).text_color(p.mute).child(sum))
            .child(chart)
    }
}
