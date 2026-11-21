//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Scheduled Runs**
//! ([`ScheduledRunsEl`]) over `oxyn_core::kit::schedfid`. The element is `RenderOnce`: the
//! host passes the tasks, the mode and an agent filter and gets [`Act`] ids back (`mode-auto`,
//! `mode-manual`, `cell-<n>`). A static snapshot: no dragging, fly-in or detail popover.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::kit::schedfid::{SR_DAYS, SR_LIMIT, SR_SLOTS, SrMode, SrTask, sr_demo, sr_plan};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};
use crate::brand::brand_icon;

const CELL: [&str; 9] = [
    "cell-0", "cell-1", "cell-2", "cell-3", "cell-4", "cell-5", "cell-6", "cell-7", "cell-8",
];

/// `scheduled-runs` (affectUI Scheduled Runs): the host passes the tasks and the mode.
#[derive(IntoElement)]
pub struct ScheduledRunsEl {
    tasks: Vec<SrTask>,
    mode: SrMode,
    agent: Option<String>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ScheduledRunsEl {
    fn default() -> Self {
        Self {
            tasks: sr_demo(),
            mode: SrMode::Auto,
            agent: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ScheduledRunsEl {
    /// The tasks and the booking mode.
    pub fn tasks(mut self, tasks: Vec<SrTask>, mode: SrMode) -> Self {
        self.tasks = tasks;
        self.mode = mode;
        self
    }
    /// Show one agent's runs (others dim).
    pub fn agent(mut self, a: Option<String>) -> Self {
        self.agent = a;
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
    /// Mode and cell clicks.
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

impl RenderOnce for ScheduledRunsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (col, card, bd, fg, mute, vi, red, track) = if dark {
            (
                h(0x1A1A1A),
                h(0x202020),
                h(0x2E2E2E),
                h(0xF2F2F2),
                h(0x85858C),
                h(0x9B87F5),
                h(0xF2685C),
                h(0x2C2C2F),
            )
        } else {
            (
                h(0xF6F6F7),
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x111111),
                h(0x8A8A8E),
                h(0x6E56CF),
                h(0xC0362C),
                h(0xE9E9EC),
            )
        };
        let t = &self.tasks;
        let nslots = SR_SLOTS.len();
        let cells = (SR_DAYS.len() * nslots).min(CELL.len());
        let plan = sr_plan(t, cells, self.mode, &[], &[]);
        let radius = if self.classic { 4. } else { 8. };
        let tile = |i: usize, share: u32, part: Option<(u32, u32)>| {
            let x = &t[i];
            let dim = self.agent.as_ref().is_some_and(|a| *a != x.agent);
            let mut top = div().flex().items_center().gap(px(4.));
            if let Some(ic) = brand_icon(&x.agent, 11., fg, true) {
                top = top.child(ic);
            }
            top = top.child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(x.title.clone()),
            );
            if let Some((k, of)) = part {
                top = top.child(
                    div()
                        .text_size(px(9.))
                        .text_color(vi)
                        .child(format!("{k}/{of}")),
                );
            }
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .p(px(6.))
                .border_1()
                .border_color(bd)
                .rounded(px(radius))
                .bg(card)
                .opacity(if dim { 0.35 } else { 1. })
                .child(top)
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(mute)
                        .child(format!("{share}%")),
                )
        };
        let mut back = div()
            .w(px(136.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .p(px(8.))
            .rounded(px(10.))
            .bg(col)
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("To book · {}", plan.backlog.len())),
            );
        if plan.backlog.is_empty() {
            back = back.child(
                div()
                    .text_size(px(11.))
                    .text_color(mute)
                    .child("Nothing left to book"),
            );
        }
        for &i in &plan.backlog {
            back = back.child(tile(i, t[i].need, None));
        }
        let mut grid = div().flex_1().flex().gap(px(6.));
        for (d, (dn, dd)) in SR_DAYS.iter().enumerate() {
            let mut colm = div().flex_1().flex().flex_col().child(
                div()
                    .h(px(24.))
                    .text_size(px(10.))
                    .text_color(mute)
                    .child(format!("{} {dd}", dn.to_uppercase())),
            );
            for s in 0..nslots {
                let cell = d * nslots + s;
                if cell >= cells {
                    break;
                }
                let load = plan.load[cell];
                let f = fire(&self.act, CELL[cell]);
                let mut stack = div().flex().flex_col().gap(px(5.));
                for p in plan.parts.iter().filter(|p| p.cell == cell) {
                    stack = stack.child(tile(p.task, p.share, (p.of > 1).then_some((p.k, p.of))));
                }
                colm = colm.child(
                    div()
                        .id(SharedString::from(format!("oxsr-{cell}")))
                        .min_h(px(92.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .py(px(6.))
                        .border_t_1()
                        .border_color(if load > SR_LIMIT { red } else { bd })
                        .on_click(move |_, w, cx| f(w, cx))
                        .child(stack)
                        .child(
                            div().h(px(3.)).rounded(px(2.)).bg(track).child(
                                div()
                                    .h_full()
                                    .rounded(px(2.))
                                    .w(relative(load.min(SR_LIMIT) as f32 / SR_LIMIT as f32))
                                    .bg(if load > SR_LIMIT { red } else { vi }),
                            ),
                        ),
                );
            }
            grid = grid.child(colm);
        }
        let seg = |m: SrMode, label: &'static str, id: &'static str| {
            let f = fire(&self.act, id);
            div()
                .id(id)
                .px(px(10.))
                .h(px(22.))
                .flex()
                .items_center()
                .rounded(px(6.))
                .text_size(px(11.5))
                .when(self.mode == m, |d| {
                    d.bg(card).font_weight(FontWeight::SEMIBOLD).text_color(fg)
                })
                .when(self.mode != m, |d| d.text_color(mute))
                .on_click(move |_, w, cx| f(w, cx))
                .child(label)
        };
        div()
            .w(px(600.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("This week"),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child("Sessions of 5 h"),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .p(px(2.))
                            .rounded(px(8.))
                            .bg(col)
                            .child(seg(SrMode::Auto, "Auto", "mode-auto"))
                            .child(seg(SrMode::Manual, "Manual", "mode-manual")),
                    ),
            )
            .child(div().flex().gap(px(10.)).child(back).child(grid))
    }
}
