//! Thinking Selector family, **Model effort menu** for GPUI ([`ModelEffortMenuEl`]) over
//! `oxyn_core::parity::effmenufid`. `RenderOnce`: the host keeps the models, the chosen one and the
//! open submenu and routes [`Act`] ids back (`model-<i>` opens a submenu, `level-<i>` picks). A
//! static snapshot.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::parity::effmenufid::{EffortModel, effort_models};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, th_pal};

const MODEL_ID: [&str; 10] = [
    "model-0", "model-1", "model-2", "model-3", "model-4", "model-5", "model-6", "model-7",
    "model-8", "model-9",
];

/// `model-effort-menu` (Thinking Selector – Model effort menu).
#[derive(IntoElement)]
pub struct ModelEffortMenuEl {
    models: Vec<EffortModel>,
    chosen: usize,
    sub: Option<usize>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ModelEffortMenuEl {
    fn default() -> Self {
        Self {
            models: effort_models(),
            chosen: 5,
            sub: Some(5),
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ModelEffortMenuEl {
    /// Your models.
    pub fn models(mut self, m: Vec<EffortModel>) -> Self {
        self.models = m;
        self
    }
    /// The chosen model and the row whose submenu is open.
    pub fn state(mut self, chosen: usize, sub: Option<usize>) -> Self {
        self.chosen = chosen;
        self.sub = sub;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `model-<i>`, `level-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ModelEffortMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut list = div()
            .w(px(330.))
            .p(px(6.))
            .rounded(px(14.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_size(px(15.));
        let mut sub_el = None;
        for (i, (m, &id)) in self.models.iter().zip(MODEL_ID.iter()).enumerate() {
            let f = fire(&self.act, id);
            let lv = m
                .levels
                .get(m.level)
                .map(|l| l.label.clone())
                .unwrap_or_default();
            let top = m.level + 1 == m.levels.len();
            let on = i == self.chosen;
            list = list.child(
                div()
                    .id(SharedString::from(format!("oxem-{i}")))
                    .h(px(36.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded(px(8.))
                    .when(on || self.sub == Some(i), |d| d.bg(p.sel))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .flex()
                            .gap(px(5.))
                            .child(div().font_weight(FontWeight::MEDIUM).child(m.name.clone()))
                            .child(div().text_color(if top { p.red } else { p.mute }).child(lv)),
                    )
                    .child(if on { "✓" } else { "›" }),
            );
            if self.sub == Some(i) {
                let mut s = div()
                    .w(px(160.))
                    .p(px(6.))
                    .rounded(px(12.))
                    .bg(p.card)
                    .border_1()
                    .border_color(p.line);
                let last = m.levels.len().saturating_sub(1);
                for (j, (l, &lid)) in m.levels.iter().zip(LEVEL_ID.iter()).enumerate() {
                    let g = fire(&self.act, lid);
                    s = s.child(
                        div()
                            .id(SharedString::from(format!("oxem-s{j}")))
                            .h(px(34.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .rounded(px(7.))
                            .on_click(move |_, w, cx| g(w, cx))
                            .child(
                                div()
                                    .text_color(if j == last { p.red } else { p.fg })
                                    .child(l.label.clone()),
                            )
                            .when(j == m.level, |d| d.child("✓")),
                    );
                }
                sub_el = Some(div().pt(px(36. * i as f32)).child(s));
            }
        }
        div()
            .flex()
            .items_start()
            .gap(px(8.))
            .text_color(p.fg)
            .child(list)
            .children(sub_el)
    }
}
