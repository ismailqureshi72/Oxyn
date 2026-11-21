//! Thinking Selector family, **Context + Effort menu** for GPUI ([`ContextEffortMenuEl`]) over
//! `oxyn_core::kit::effmenufid`. `RenderOnce`: the host keeps the choices and routes [`Act`] ids
//! back (`toggle`, `window-<i>`, `level-<i>`). A static snapshot.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::kit::{
    effmenufid::{ContextOption, context_options},
    reasonfid::context_label,
    thinkfid::{EffortLevel, effort_scale_api},
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, th_pal};

const WIN_ID: [&str; 4] = ["window-0", "window-1", "window-2", "window-3"];

/// `context-effort-menu` (Thinking Selector – Context + Effort menu).
#[derive(IntoElement)]
pub struct ContextEffortMenuEl {
    model: String,
    windows: Vec<ContextOption>,
    levels: Vec<EffortLevel>,
    window: usize,
    effort: usize,
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ContextEffortMenuEl {
    fn default() -> Self {
        Self {
            model: "Atlas 6 Swift".into(),
            windows: context_options(),
            levels: effort_scale_api(),
            window: 0,
            effort: 0,
            open: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ContextEffortMenuEl {
    /// Chosen window and effort.
    pub fn value(mut self, window: usize, effort: usize) -> Self {
        self.window = window;
        self.effort = effort;
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
    /// `toggle`, `window-<i>`, `level-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ContextEffortMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let head = |t: &'static str| {
            div()
                .px(px(10.))
                .py(px(4.))
                .text_size(px(11.5))
                .text_color(p.mute)
                .child(t)
        };
        let row = |on: bool| {
            div()
                .h(px(31.))
                .px(px(10.))
                .flex()
                .items_center()
                .justify_between()
                .rounded(px(7.))
                .when(on, |d| d.bg(p.sel))
        };
        let mut menu = div()
            .w(px(220.))
            .p(px(6.))
            .rounded(px(12.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_size(px(13.5))
            .child(head("Context window"));
        for (i, (w, &id)) in self.windows.iter().zip(WIN_ID.iter()).enumerate() {
            let f = fire(&self.act, id);
            let on = i == self.window;
            let mut l = div().flex().gap(px(7.)).child(context_label(w.tokens));
            if !w.tag.is_empty() {
                l = l.child(
                    div()
                        .px(px(6.))
                        .rounded(px(5.))
                        .bg(p.sel)
                        .text_size(px(11.))
                        .text_color(p.mute)
                        .child(w.tag.clone()),
                );
            }
            menu = menu.child(
                row(on)
                    .id(SharedString::from(format!("oxec-w{i}")))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(l)
                    .when(on, |d| d.child("✓")),
            );
        }
        menu = menu
            .child(div().my(px(5.)).h(px(1.)).bg(p.line))
            .child(head("Effort"));
        let last = self.levels.len().saturating_sub(1);
        for (i, (l, &id)) in self.levels.iter().zip(LEVEL_ID.iter()).enumerate() {
            let f = fire(&self.act, id);
            let on = i == self.effort;
            menu = menu.child(
                row(on)
                    .id(SharedString::from(format!("oxec-e{i}")))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .text_color(if i == last { p.red } else { p.fg })
                            .child(l.label.clone()),
                    )
                    .when(on, |d| d.child("✓")),
            );
        }
        let toggle = fire(&self.act, "toggle");
        let sum = format!(
            "{} · {}",
            self.windows
                .get(self.window)
                .map(|w| context_label(w.tokens))
                .unwrap_or_default(),
            self.levels
                .get(self.effort)
                .map(|l| l.label.clone())
                .unwrap_or_default()
        );
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_color(p.fg)
            .child(
                div()
                    .id("oxec-chip")
                    .h(px(30.))
                    .px(px(11.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .rounded(px(999.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card2)
                    .on_click(move |_, w, cx| toggle(w, cx))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.model.clone()),
                    )
                    .child(div().text_color(p.mute).child(sum)),
            )
            .when(self.open, |d| d.child(menu))
    }
}
