//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Fan-out Grid**
//! ([`FanoutGridEl`]) over `oxyn_core::parity::fanfid`. The element is `RenderOnce`: the host
//! passes the tasks and the time and gets [`Act`] ids back (`retry`). A static snapshot: no
//! shine, filter or details card.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::*,
    px, rgb,
};
use oxyn_core::parity::fanfid::{FoStatus, FoTask, fo_demo, fo_secs, fo_view};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

/// `fanout-grid` (affectUI Fan-out Grid): the host passes the tasks and the time.
#[derive(IntoElement)]
pub struct FanoutGridEl {
    tasks: Vec<FoTask>,
    at: u32,
    workers: u32,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for FanoutGridEl {
    fn default() -> Self {
        Self {
            tasks: fo_demo(240, 12),
            at: 8_000,
            workers: 12,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl FanoutGridEl {
    /// The tasks, the ms elapsed and the workers.
    pub fn job(mut self, tasks: Vec<FoTask>, at: u32, workers: u32) -> Self {
        self.tasks = tasks;
        self.at = at;
        self.workers = workers;
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
    /// Retry clicks.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for FanoutGridEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, q, run, done, amb, red) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2F),
                h(0xF2F2F2),
                h(0x8B8B90),
                h(0x26262B),
                h(0x60A5FA),
                h(0x34D399),
                h(0xFBBF24),
                h(0xF87171),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x111111),
                h(0x8A8A8E),
                h(0xEDEDEF),
                h(0x3B82F6),
                h(0x4ADE80),
                h(0xF59E0B),
                h(0xEF4444),
            )
        };
        let v = fo_view(&self.tasks, self.at, None, self.workers);
        let mut grid = div().flex().flex_wrap().gap(px(3.));
        for s in &v.status {
            grid = grid.child(
                div()
                    .size(px(10.))
                    .rounded(px(if self.classic { 2. } else { 3. }))
                    .bg(match s {
                        FoStatus::Queued => q,
                        FoStatus::Running => run,
                        FoStatus::Retrying => amb,
                        FoStatus::Done => done,
                        FoStatus::Failed => red,
                    }),
            );
        }
        let act = self.act.clone();
        let failed = v.counts[3] > 0;
        div()
            .w(px(580.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .border_1()
            .border_color(bd)
            .rounded(px(12.))
            .bg(card)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Fan-out"),
                    )
                    .child(div().flex_1())
                    .child(div().text_size(px(11.5)).text_color(mute).child(format!(
                        "{} / {} · ${:.2} · {}",
                        v.counts[0],
                        self.tasks.len(),
                        v.cost,
                        if v.finished {
                            format!("Finished in {}", fo_secs(v.ended_at))
                        } else {
                            format!("~{} left", fo_secs(v.left))
                        }
                    ))),
            )
            .child(grid)
            .child(
                div()
                    .flex()
                    .gap(px(12.))
                    .text_size(px(11.5))
                    .child(format!("{} done", v.counts[0]))
                    .child(format!("{} running", v.counts[1]))
                    .child(format!("{} retrying", v.counts[2]))
                    .when(failed, |d| {
                        d.child(
                            div()
                                .id("retry")
                                .text_color(red)
                                .on_click(move |_, w, cx| {
                                    if let Some(f) = &act {
                                        f("retry", w, cx)
                                    }
                                })
                                .child(format!("{} failed · Retry", v.counts[3])),
                        )
                    }),
            )
    }
}
