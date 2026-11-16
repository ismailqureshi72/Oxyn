//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32) for GPUI: the exact
//! **Cost Waterfall** ([`CostWaterfallEl`]) over `oxyn_core::parity::costfid`. The element is
//! `RenderOnce`: the host keeps the mode, the folded sub-agents and the open step and routes
//! [`Act`] ids back (`cost`, `tokens`, `row-<n>` for the n-th visible row). A static snapshot:
//! no details, no live sheen.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::parity::costfid::{
    CostKind, CostMode, CostStatus, CostStep, cost_axis, cost_demo, cost_money, cost_rows,
    cost_sum, cost_time, cost_tok, cost_tokens, cost_top, cost_total,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const W: f32 = 756.0;
const ROW: [&str; 16] = [
    "row-0", "row-1", "row-2", "row-3", "row-4", "row-5", "row-6", "row-7", "row-8", "row-9",
    "row-10", "row-11", "row-12", "row-13", "row-14", "row-15",
];

/// `cost-waterfall` (affectUI Cost Waterfall): the host passes the run.
#[derive(IntoElement)]
pub struct CostWaterfallEl {
    steps: Vec<CostStep>,
    mode: CostMode,
    folded: Vec<String>,
    title: String,
    task: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for CostWaterfallEl {
    fn default() -> Self {
        Self {
            steps: cost_demo(),
            mode: CostMode::Cost,
            folded: vec![],
            title: "Run cost".into(),
            task: "Release notes for v2.4".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl CostWaterfallEl {
    /// The run's steps and what it is about.
    pub fn steps(mut self, s: Vec<CostStep>, task: &str) -> Self {
        self.steps = s;
        self.task = task.into();
        self
    }
    /// Cost or tokens.
    pub fn mode(mut self, m: CostMode) -> Self {
        self.mode = m;
        self
    }
    /// Folded sub-agents.
    pub fn folded(mut self, f: Vec<String>) -> Self {
        self.folded = f;
        self
    }
    /// Classic style.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// The switch and the rows.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

impl RenderOnce for CostWaterfallEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, track, llm, tool, agent, red, band) = if dark {
            (
                h(0x17171B),
                h(0x2A2A30),
                h(0xEDEDEF),
                h(0x8B8B94),
                h(0x232328),
                h(0x2E2545),
                h(0x16313A),
                h(0x26262C),
                h(0xF2726B),
                h(0x1D1D22),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE8E8EB),
                h(0x17171A),
                h(0x8A8A93),
                h(0xF1F1F3),
                h(0xF1EBFE),
                h(0xE9F7F9),
                h(0xF1F1F4),
                h(0xD1483B),
                h(0xF4F4F5),
            )
        };
        let rows = cost_rows(&self.steps, self.mode, &self.folded);
        let total: f64 = self.steps.iter().map(|s| cost_total(s, self.mode)).sum();
        let total_cost: f64 = self
            .steps
            .iter()
            .map(|s| cost_total(s, CostMode::Cost))
            .sum();
        let (axis, _) = cost_axis(total);
        let top = cost_top(&self.steps);
        let mut sw = div().flex().p(px(3.)).rounded(px(8.)).bg(track);
        for (id, label, m) in [
            ("cost", "Cost", CostMode::Cost),
            ("tokens", "Tokens", CostMode::Tokens),
        ] {
            let on = self.mode == m;
            let f = fire(&self.act, id);
            sw = sw.child(
                div()
                    .id(SharedString::from(format!("oxcf-{id}")))
                    .h(px(24.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .rounded(px(6.))
                    .text_size(px(12.5))
                    .when(on, |d| d.bg(card).text_color(fg))
                    .when(!on, |d| d.text_color(mute))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(label),
            );
        }
        let bar = div()
            .h(px(48.))
            .px(px(16.))
            .flex()
            .items_center()
            .gap(px(8.))
            .border_b_1()
            .border_color(bd)
            .when(self.classic, |d| d.bg(band))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(mute)
                    .child(self.task.clone()),
            )
            .child(div().flex_1())
            .child(sw);
        let (tin, tout) = self.steps.iter().fold((0, 0), |(a, b), s| {
            let (x, y) = cost_tokens(s);
            (a + x, b + y)
        });
        let stat = |label: &str, value: String| {
            div()
                .flex_1()
                .px(px(22.))
                .py(px(12.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(9.5))
                        .text_color(mute)
                        .child(label.to_uppercase()),
                )
                .child(
                    div()
                        .text_size(px(16.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(value),
                )
        };
        let stats = div()
            .flex()
            .border_b_1()
            .border_color(bd)
            .child(stat("Total cost", cost_money(Some(total_cost))))
            .child(stat("Total tokens", cost_tok(tin + tout)))
            .child(stat(
                "Most expensive",
                top.as_ref().map_or_else(|| "–".into(), |t| t.name.clone()),
            ));
        let mut list = div().py(px(2.)).flex().flex_col();
        for (i, r) in rows.iter().enumerate() {
            let s = &r.step;
            let kids = !s.children.is_empty();
            let (a, b) = if kids {
                cost_tokens(s)
            } else {
                (s.tokens_in, s.tokens_out)
            };
            let cost = if kids { cost_sum(s) } else { s.cost };
            let fill = match (s.status, s.kind) {
                (CostStatus::Failed, _) => red,
                (_, CostKind::Llm) => llm,
                (_, CostKind::Tool) => tool,
                _ => agent,
            };
            let f = fire(&self.act, ROW[i.min(15)]);
            let num = |t: String| div().w(px(58.)).flex().justify_end().child(t);
            list = list.child(
                div()
                    .id(SharedString::from(format!("oxcf-row-{i}")))
                    .h(px(32.))
                    .pr(px(16.))
                    .flex()
                    .items_center()
                    .text_size(px(12.))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .w(px(250.))
                            .pl(px(16. + r.depth as f32 * 14.))
                            .flex()
                            .gap(px(6.))
                            .child(
                                div()
                                    .when(kids, |d| d.font_weight(FontWeight::SEMIBOLD))
                                    .child(s.name.clone()),
                            )
                            .child(div().text_color(mute).child(s.detail.clone())),
                    )
                    .child(
                        div().flex_1().h(px(14.)).relative().child(
                            div()
                                .absolute()
                                .left(relative((r.start / axis) as f32))
                                .w(relative(((r.len / axis) as f32).max(0.004)))
                                .h(px(14.))
                                .rounded(px(4.))
                                .bg(fill)
                                .border_1()
                                .border_color(bd),
                        ),
                    )
                    .child(num(cost_tok(a)))
                    .child(num(cost_tok(b)))
                    .child(num(cost_money(cost)))
                    .child(num(cost_time(s.duration)).text_color(
                        if s.status == CostStatus::Failed {
                            red
                        } else {
                            mute
                        },
                    )),
            );
        }
        div()
            .w(px(W))
            .flex()
            .flex_col()
            .rounded(px(if self.classic { 14. } else { 12. }))
            .border_1()
            .border_color(bd)
            .bg(card)
            .text_color(fg)
            .overflow_hidden()
            .child(bar)
            .child(stats)
            .child(list)
    }
}
