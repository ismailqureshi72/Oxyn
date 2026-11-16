//! affectUI fidelity pass, family B part 8 (plan #32) for GPUI: the exact **Delegate Board**
//! ([`DelegateBoardEl`]) over `oxyn_core::parity::delegatefid`. The element is `RenderOnce`: the
//! host keeps the pile, the hand-overs and the Auto switch and routes [`Act`] ids back (`t-<n>`
//! for the n-th pile task, `a-<n>` to hand the held task to the n-th agent, `auto` for the switch).
//! A static snapshot: no drag and drop.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::delegatefid::{
    DB_START, DbAgent, DbTask, db_agents, db_clock, db_cost, db_load_min, db_tasks, db_usd,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const T: [&str; 8] = ["t-0", "t-1", "t-2", "t-3", "t-4", "t-5", "t-6", "t-7"];
const A: [&str; 6] = ["a-0", "a-1", "a-2", "a-3", "a-4", "a-5"];

/// `delegate-board` (affectUI Delegate Board): the host passes agents, tasks and hand-overs.
#[derive(IntoElement)]
pub struct DelegateBoardEl {
    agents: Vec<DbAgent>,
    tasks: Vec<DbTask>,
    given: Vec<(String, String)>,
    held: Option<usize>,
    auto: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for DelegateBoardEl {
    fn default() -> Self {
        Self {
            agents: db_agents(),
            tasks: db_tasks(),
            given: vec![],
            held: None,
            auto: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl DelegateBoardEl {
    /// Agents and every task.
    pub fn board(mut self, agents: Vec<DbAgent>, tasks: Vec<DbTask>) -> Self {
        self.agents = agents;
        self.tasks = tasks;
        self
    }
    /// Hand-overs: (task id, agent id).
    pub fn given(mut self, g: Vec<(String, String)>) -> Self {
        self.given = g;
        self
    }
    /// The pile task picked up (index into the pile).
    pub fn held(mut self, i: Option<usize>) -> Self {
        self.held = i;
        self
    }
    /// Auto switch state.
    pub fn auto(mut self, on: bool) -> Self {
        self.auto = on;
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
    /// Pile picks, hand-overs and the switch.
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

impl RenderOnce for DelegateBoardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, pile_bg, blue, blue_bg, ok, chip) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2E),
                h(0xEDEDEF),
                h(0x8A8A93),
                h(0x101012),
                h(0x4C8DFF),
                h(0x1A2740),
                h(0x34D399),
                h(0x232327),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE6E6E6),
                h(0x111827),
                h(0x6B7280),
                h(0xF5F5F5),
                h(0x2563EB),
                h(0xEFF6FF),
                h(0x16A34A),
                h(0xF1F1F1),
            )
        };
        let pile: Vec<&DbTask> = self
            .tasks
            .iter()
            .filter(|t| !self.given.iter().any(|(ti, _)| *ti == t.id))
            .collect();
        let af = fire(&self.act, "auto");
        let bar = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Delegate"),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(mute)
                    .child(if self.auto {
                        "Each task goes to the agent that frees up first"
                    } else {
                        "Drop a task on an agent to hand it over"
                    }),
            )
            .child(div().flex_1())
            .child(div().text_size(px(12.)).child("Auto"))
            .child(
                div()
                    .id("oxdb-auto")
                    .w(px(32.))
                    .h(px(18.))
                    .rounded_full()
                    .bg(if self.auto { blue } else { bd })
                    .flex()
                    .items_center()
                    .px(px(2.))
                    .when(self.auto, |d| d.justify_end())
                    .on_click(move |_, w, cx| af(w, cx))
                    .child(div().size(px(14.)).rounded_full().bg(h(0xFFFFFF))),
            );
        let mut left = div()
            .w(px(170.))
            .p(px(10.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .rounded(px(12.))
            .bg(pile_bg)
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(mute)
                    .child(format!("Unassigned {}", pile.len())),
            );
        if pile.is_empty() {
            left = left.child(
                div()
                    .text_size(px(12.))
                    .text_color(mute)
                    .child("Nothing left to hand out"),
            );
        }
        for (i, t) in pile.iter().enumerate().take(8) {
            let f = fire(&self.act, T[i]);
            left = left.child(
                div()
                    .id(SharedString::from(format!("oxdb-t{i}")))
                    .p(px(8.))
                    .border_1()
                    .border_color(if self.held == Some(i) { blue } else { bd })
                    .rounded(px(10.))
                    .bg(card)
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(div().text_size(px(12.)).child(t.title.clone()))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child(format!("~{} min", t.est)),
                    ),
            );
        }
        let cost = db_cost(&self.agents, &self.tasks, &self.given);
        let mut right = div().flex_1().flex().flex_col().gap(px(14.)).child(
            div()
                .flex()
                .text_size(px(12.))
                .text_color(mute)
                .child("Agents")
                .child(div().flex_1())
                .child(format!("{} in queues", db_usd(cost))),
        );
        for (ai, a) in self.agents.iter().enumerate().take(6) {
            let load = db_load_min(a, &self.tasks, &self.given);
            let f = fire(&self.act, A[ai]);
            let mut chips: Vec<String> = a.current.iter().map(|c| c.0.clone()).collect();
            chips.extend(
                self.given
                    .iter()
                    .filter(|(_, x)| *x == a.id)
                    .filter_map(|(ti, _)| self.tasks.iter().find(|t| t.id == *ti))
                    .map(|t| t.title.clone()),
            );
            let mut q = div().flex().flex_wrap().gap(px(5.)).pl(px(30.));
            for (ci, ch) in chips.iter().enumerate().take(3) {
                q = q.child(
                    div()
                        .h(px(22.))
                        .px(px(9.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .text_size(px(11.))
                        .when(ci == 0, |d| d.bg(blue_bg).text_color(blue))
                        .when(ci > 0, |d| d.border_1().border_color(bd))
                        .child(ch.clone()),
                );
            }
            if chips.len() > 3 {
                q = q.child(
                    div()
                        .px(px(8.))
                        .text_size(px(11.))
                        .text_color(mute)
                        .child(format!("+{}", chips.len() - 3)),
                );
            }
            let (pill_t, pill_fg, pill_bg) = if load == 0 {
                ("Free".to_string(), ok, chip)
            } else {
                (
                    format!("Busy until {}", db_clock(DB_START + load)),
                    mute,
                    chip,
                )
            };
            right = right.child(
                div()
                    .id(SharedString::from(format!("oxdb-a{ai}")))
                    .flex()
                    .flex_col()
                    .gap(px(7.))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .size(px(22.))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(bd)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(10.))
                                    .child(a.name.chars().next().unwrap_or('?').to_string()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .child(
                                        div()
                                            .text_size(px(12.5))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(a.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(mute)
                                            .child(format!("{} · ${:.2}/min", a.model, a.rate)),
                                    ),
                            )
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .rounded(px(8.))
                                    .bg(pill_bg)
                                    .text_color(pill_fg)
                                    .text_size(px(11.))
                                    .child(pill_t),
                            ),
                    )
                    .child(q),
            );
        }
        div()
            .w(px(720.))
            .p(px(18.))
            .flex()
            .flex_col()
            .gap(px(14.))
            .border_1()
            .border_color(bd)
            .rounded(px(if self.classic { 16. } else { 14. }))
            .bg(card)
            .text_color(fg)
            .child(bar)
            .child(div().flex().gap(px(16.)).child(left).child(right))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(mute)
                    .child("Drop a card on an agent, or use Enter, the arrows and the digits."),
            )
    }
}
