//! Thinking Selector family, **Effort list** for GPUI ([`EffortListEl`]) over
//! `oxyn_core::parity::thinkfid`. `RenderOnce`: the host keeps the level and whether the menu is
//! open and routes [`Act`] ids back (`toggle`, `level-<i>`). A static snapshot.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::parity::thinkfid::{EffortLevel, effort_scale_short};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, th_pal};

/// `effort-list` (Thinking Selector – Effort list).
#[derive(IntoElement)]
pub struct EffortListEl {
    levels: Vec<EffortLevel>,
    value: usize,
    open: bool,
    heading: String,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for EffortListEl {
    fn default() -> Self {
        Self {
            levels: effort_scale_short(),
            value: 3,
            open: true,
            heading: "Thinking effort".into(),
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl EffortListEl {
    /// Your own scale.
    pub fn levels(mut self, l: Vec<EffortLevel>) -> Self {
        self.levels = l;
        self
    }
    /// Level index.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
        self
    }
    /// Menu open.
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `toggle`, `level-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for EffortListEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let n = self.levels.len().max(1);
        let cur = self.value.min(n - 1);
        let mut menu = div()
            .w(px(300.))
            .p(px(6.))
            .rounded(px(14.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .child(
                div()
                    .px(px(12.))
                    .py(px(4.))
                    .text_size(px(13.))
                    .text_color(p.mute)
                    .child(self.heading.clone()),
            );
        for (i, (l, &id)) in self.levels.iter().zip(LEVEL_ID.iter()).enumerate() {
            let f = fire(&self.act, id);
            menu = menu.child(
                div()
                    .id(SharedString::from(format!("oxel-{i}")))
                    .h(px(36.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded(px(9.))
                    .text_size(px(17.))
                    .when(i == cur, |d| d.bg(p.sel))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(l.label.clone())
                    .when(i == cur, |d| d.child("✓")),
            );
        }
        let toggle = fire(&self.act, "toggle");
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_color(p.fg)
            .when(self.open, |d| d.child(menu))
            .child(
                div()
                    .id("oxel-chip")
                    .h(px(30.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card2)
                    .font_weight(FontWeight::MEDIUM)
                    .on_click(move |_, w, cx| toggle(w, cx))
                    .child(self.levels[cur].label.clone())
                    .child(div().text_color(p.mute).child("⌃")),
            )
    }
}
