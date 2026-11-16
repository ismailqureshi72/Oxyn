//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Trace Waterfall**
//! ([`TraceWaterfallEl`]) over `oxyn_core::parity::tracefid`. The element is `RenderOnce`: the
//! host passes the trace, the time and the folded branches and gets [`Act`] ids back
//! (`span-<n>`, the row index). A static snapshot: no sheen, details window or keyboard tree.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::parity::tracefid::{
    TwKind, TwSpan, TwStatus, tw_axis, tw_demo, tw_dur, tw_find, tw_rows, tw_stats,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const SPAN: [&str; 16] = [
    "span-0", "span-1", "span-2", "span-3", "span-4", "span-5", "span-6", "span-7", "span-8",
    "span-9", "span-10", "span-11", "span-12", "span-13", "span-14", "span-15",
];

/// `trace-waterfall` (affectUI Trace Waterfall): the host passes the trace and the time.
#[derive(IntoElement)]
pub struct TraceWaterfallEl {
    trace: TwSpan,
    at: u32,
    folded: Vec<String>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for TraceWaterfallEl {
    fn default() -> Self {
        let t = tw_demo();
        let at = t.end;
        Self {
            trace: t,
            at,
            folded: vec![],
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl TraceWaterfallEl {
    /// The trace and the ms elapsed.
    pub fn trace(mut self, t: TwSpan, at: u32) -> Self {
        self.trace = t;
        self.at = at;
        self
    }
    /// Folded branches (ids).
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
    /// Row clicks.
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

impl RenderOnce for TraceWaterfallEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, agent, llm, tool, red) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2F),
                h(0xF2F2F2),
                h(0x8B8B90),
                h(0x94A3B8),
                h(0xA78BFA),
                h(0x2DD4BF),
                h(0xF2685C),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x111111),
                h(0x8A8A8E),
                h(0x64748B),
                h(0x6E56CF),
                h(0x0F9D8A),
                h(0xC0362C),
            )
        };
        let tr = &self.trace;
        let st = tw_stats(tr, self.at);
        let (axis, _) = tw_axis(st.duration.max(1));
        let rows = tw_rows(tr, self.at, &self.folded);
        let mut list = div().flex().flex_col();
        for (i, r) in rows.iter().enumerate().take(SPAN.len()) {
            let Some(s) = tw_find(tr, &r.id) else {
                continue;
            };
            let end = s.shown_end(self.at);
            let tint = match (s.status(self.at), s.kind) {
                (TwStatus::Failed, _) => red,
                (_, TwKind::Llm) => llm,
                (_, TwKind::Tool) => tool,
                _ => agent,
            };
            let f = fire(&self.act, SPAN[i]);
            list = list.child(
                div()
                    .id(SharedString::from(format!("oxtw-{i}")))
                    .h(px(if self.classic { 28. } else { 26. }))
                    .flex()
                    .items_center()
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .w(px(220.))
                            .pl(px(r.depth as f32 * 14.))
                            .flex()
                            .gap(px(5.))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(s.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(mute)
                                    .child(s.detail.clone()),
                            ),
                    )
                    .child(
                        div().flex_1().relative().h(px(8.)).child(
                            div()
                                .absolute()
                                .left(relative(s.start as f32 / axis as f32))
                                .w(relative((end - s.start) as f32 / axis as f32))
                                .h_full()
                                .rounded(px(4.))
                                .bg(tint),
                        ),
                    )
                    .child(
                        div()
                            .w(px(80.))
                            .pl(px(6.))
                            .text_size(px(10.5))
                            .text_color(mute)
                            .child(tw_dur(end - s.start)),
                    ),
            );
        }
        let stat = |k: &'static str, v: String, bad: bool| {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .child(div().text_size(px(9.5)).text_color(mute).child(k))
                .child(
                    div()
                        .text_size(px(13.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if bad { red } else { fg })
                        .child(v),
                )
        };
        div()
            .w(px(600.))
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
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Trace"),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.))
                            .child(if st.done { "Done" } else { "Live" }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(stat("DURATION", tw_dur(st.duration), false))
                    .child(stat("COST", format!("${:.2}", st.cost), false))
                    .child(stat("MODEL CALLS", st.llm.to_string(), false))
                    .child(stat("TOOL CALLS", st.tools.to_string(), false))
                    .child(stat("ERRORS", st.errors.to_string(), st.errors > 0)),
            )
            .child(list)
    }
}
