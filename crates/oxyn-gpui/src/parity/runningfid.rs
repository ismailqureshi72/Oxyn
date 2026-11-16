//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Running Now**
//! ([`RunningNowEl`]) over `oxyn_core::parity::runningfid`. The element is `RenderOnce`: the host
//! keeps the tasks and whether the list is open and gets [`Act`] ids back (`pill`, `open-<n>`,
//! `pause-<n>` for the n-th task, `all`). A static snapshot: the list draws in flow under the pill,
//! rings are drawn as a step bar.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::parity::runningfid::{RnState, RnTask, rn_counts, rn_demo};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const OPEN: [&str; 8] = [
    "open-0", "open-1", "open-2", "open-3", "open-4", "open-5", "open-6", "open-7",
];
const PAUSE: [&str; 8] = [
    "pause-0", "pause-1", "pause-2", "pause-3", "pause-4", "pause-5", "pause-6", "pause-7",
];

/// `running-now` (affectUI Running Now): the host passes the tasks.
#[derive(IntoElement)]
pub struct RunningNowEl {
    tasks: Vec<RnTask>,
    open: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for RunningNowEl {
    fn default() -> Self {
        Self {
            tasks: rn_demo(),
            open: true,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl RunningNowEl {
    /// The tasks.
    pub fn tasks(mut self, t: Vec<RnTask>) -> Self {
        self.tasks = t;
        self
    }
    /// List open.
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
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
    /// Pill, rows, pause / resume and Open tasks.
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

impl RenderOnce for RunningNowEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, run, amb, done, track) = if dark {
            (
                h(0x1B1B1B),
                h(0x2E2E2E),
                h(0xF2F2F2),
                h(0x85858C),
                h(0x9B87F5),
                h(0xF0A54A),
                h(0x3DD68C),
                h(0x333336),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x111111),
                h(0x8A8A8E),
                h(0x6E56CF),
                h(0xB45F06),
                h(0x1A7F4B),
                h(0xE8E8EA),
            )
        };
        let (r, n) = rn_counts(&self.tasks);
        let pf = fire(&self.act, "pill");
        let pill = div()
            .id("oxrn-pill")
            .h(px(24.))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(10.))
            .border_1()
            .border_color(bd)
            .rounded(px(if self.classic { 8. } else { 999. }))
            .bg(card)
            .text_size(px(12.))
            .on_click(move |_, w, cx| pf(w, cx))
            .when(r > 0, |d| {
                d.child(div().text_color(run).child(format!("● {r} running")))
            })
            .when(n > 0, |d| {
                d.child(div().text_color(amb).child(format!(
                    "● {n} {}",
                    if n == 1 { "needs you" } else { "need you" }
                )))
            });
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_color(fg)
            .child(pill);
        if self.open {
            let mut list = div()
                .w(px(284.))
                .p(px(8.))
                .flex()
                .flex_col()
                .border_1()
                .border_color(bd)
                .rounded(px(12.))
                .bg(card)
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(mute)
                        .child(format!("BACKGROUND TASKS {}", self.tasks.len())),
                );
            for (i, t) in self.tasks.iter().enumerate().take(8) {
                let of = fire(&self.act, OPEN[i]);
                let pz = fire(&self.act, PAUSE[i]);
                let tone = match t.state {
                    RnState::Running => run,
                    RnState::NeedsYou => amb,
                    RnState::Done => done,
                    RnState::Paused => mute,
                };
                let f = if t.state == RnState::Done {
                    1.0
                } else {
                    t.done as f32 / t.total().max(1) as f32
                };
                list = list.child(
                    div()
                        .h(px(40.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .id(SharedString::from(format!("oxrn-o{i}")))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .on_click(move |_, w, cx| of(w, cx))
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(t.title.clone()),
                                )
                                .child(div().text_size(px(11.)).text_color(mute).child(t.line()))
                                .child(
                                    div()
                                        .h(px(2.))
                                        .rounded_full()
                                        .bg(track)
                                        .child(div().h_full().w(relative(f)).bg(tone)),
                                ),
                        )
                        .child(div().text_size(px(11.)).text_color(mute).child(format!(
                            "{}/{}",
                            t.done.min(t.total()),
                            t.total()
                        )))
                        .child(
                            div()
                                .id(SharedString::from(format!("oxrn-p{i}")))
                                .text_size(px(11.))
                                .text_color(tone)
                                .on_click(move |_, w, cx| pz(w, cx))
                                .child(match t.state {
                                    RnState::Running => "❚❚",
                                    RnState::Paused => "▶",
                                    RnState::NeedsYou => "Open",
                                    RnState::Done => "✓",
                                }),
                        ),
                );
            }
            let af = fire(&self.act, "all");
            list = list.child(
                div()
                    .id("oxrn-all")
                    .h(px(32.))
                    .border_t_1()
                    .border_color(bd)
                    .flex()
                    .items_center()
                    .text_size(px(12.))
                    .on_click(move |_, w, cx| af(w, cx))
                    .child("Open tasks"),
            );
            root = root.child(list);
        }
        root
    }
}
