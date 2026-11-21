//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the **Diff Timeline** run view
//! ([`DiffTimelineRunEl`]) over `oxyn_core::kit::difftlfid`. The element is `RenderOnce`: the
//! host passes the edits and the time and gets [`Act`] ids back (`file-<n>`, the row index). A
//! static snapshot: the chart is drawn as cumulative columns per edit, with no popover or motion.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::kit::difftlfid::{
    DtEdit, dt_demo, dt_end, dt_files, dt_landed, dt_split, dt_status, dt_totals, dt_y_axis,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const FILE: [&str; 12] = [
    "file-0", "file-1", "file-2", "file-3", "file-4", "file-5", "file-6", "file-7", "file-8",
    "file-9", "file-10", "file-11",
];

/// `diff-timeline` run view (affectUI Diff Timeline): the host passes the edits and the time.
#[derive(IntoElement)]
pub struct DiffTimelineRunEl {
    edits: Vec<DtEdit>,
    at: u32,
    agent: String,
    context: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for DiffTimelineRunEl {
    fn default() -> Self {
        let e = dt_demo();
        let at = dt_end(&e);
        Self {
            edits: e,
            at,
            agent: "UI agent".into(),
            context: "PR #57".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl DiffTimelineRunEl {
    /// The edits and the ms elapsed.
    pub fn edits(mut self, e: Vec<DtEdit>, at: u32) -> Self {
        self.edits = e;
        self.at = at;
        self
    }
    /// The agent's name and the context after it.
    pub fn agent(mut self, name: impl Into<String>, context: impl Into<String>) -> Self {
        self.agent = name.into();
        self.context = context.into();
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
    /// File row clicks.
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

impl RenderOnce for DiffTimelineRunEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, wash, add, rem) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2F),
                h(0xF2F2F2),
                h(0x8B8B90),
                h(0x222226),
                h(0x4ADE80),
                h(0xF87171),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE7E7EA),
                h(0x111111),
                h(0x8A8A8E),
                h(0xF3F3F5),
                h(0x16A34A),
                h(0xDC2626),
            )
        };
        let e = &self.edits;
        let tot = dt_totals(e, self.at);
        let (ymax, _) = dt_y_axis(dt_totals(e, dt_end(e)).added.max(1));
        // Chart: one column pair per landed edit, cumulative.
        let mut chart = div().h(px(120.)).flex().items_end().gap(px(3.));
        let (mut a, mut r) = (0u32, 0u32);
        for i in dt_landed(e, self.at) {
            a += e[i].added;
            r += e[i].removed;
            chart = chart.child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_end()
                    .gap(px(1.))
                    .child(
                        div()
                            .flex_1()
                            .h(relative(a as f32 / ymax as f32))
                            .rounded_t(px(2.))
                            .bg(add),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h(relative(r as f32 / ymax as f32))
                            .rounded_t(px(2.))
                            .bg(rem),
                    ),
            );
        }
        let files = dt_files(e, self.at);
        let big = files
            .iter()
            .map(|f| f.added + f.removed)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut list = div().flex().flex_col();
        for (i, f) in files.iter().enumerate().take(FILE.len()) {
            let (fd, fnm) = dt_split(&f.file);
            let n = (f.added + f.removed).max(1) as f32;
            let mut bar = div()
                .h(px(6.))
                .w(relative((f.added + f.removed).max(1) as f32 / big as f32))
                .flex()
                .rounded(px(3.))
                .bg(wash);
            if !f.binary {
                for (_, pa, pr) in &f.pieces {
                    bar = bar
                        .child(div().h_full().w(relative(*pa as f32 / n)).bg(add))
                        .child(div().h_full().w(relative(*pr as f32 / n)).bg(rem));
                }
            }
            let cb = fire(&self.act, FILE[i]);
            list = list.child(
                div()
                    .id(SharedString::from(format!("oxdl-{i}")))
                    .h(px(22.))
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .on_click(move |_, w, cx| cb(w, cx))
                    .child(
                        div()
                            .w(px(190.))
                            .flex()
                            .text_size(px(11.5))
                            .child(div().text_color(mute).child(fd))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(fnm)),
                    )
                    .child(div().flex_1().child(bar))
                    .child(
                        div()
                            .w(px(70.))
                            .flex()
                            .justify_end()
                            .gap(px(4.))
                            .text_size(px(11.))
                            .when(f.binary, |d| d.text_color(mute).child("binary"))
                            .when(!f.binary, |d| {
                                d.child(div().text_color(add).child(format!("+{}", f.added)))
                                    .child(div().text_color(rem).child(format!("−{}", f.removed)))
                            }),
                    ),
            );
        }
        div()
            .w(px(440.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .border_1()
            .border_color(bd)
            .rounded(px(if self.classic { 16. } else { 12. }))
            .bg(card)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .flex()
                                    .gap(px(5.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(self.agent.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(mute)
                                            .child(format!("· {}", self.context)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(mute)
                                    .child(dt_status(e, self.at)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(add)
                            .child(format!("+{}", tot.added)),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rem)
                            .child(format!("−{}", tot.removed)),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child(format!("across {} files", tot.files)),
                    ),
            )
            .child(chart)
            .child(list)
    }
}
