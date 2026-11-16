//! affectUI fidelity pass, family B part 8 (plan #32) for GPUI: the exact **Delegate Board**
//! ([`DelegateBoardEl`]) over `oxyn_core::parity::delegatefid`. The element is `RenderOnce`: the
//! host keeps the tray, the selection and the hand-overs and routes [`Act`] ids back (`t-<n>` for
//! the n-th tray task, `a-<n>` to hand the selected task to the n-th agent). A static snapshot:
//! no drag and drop, statuses come from the host.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::delegatefid::{DbAgent, DbStatus, DbTask, db_agents, db_fits, db_tasks};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const T: [&str; 8] = ["t-0", "t-1", "t-2", "t-3", "t-4", "t-5", "t-6", "t-7"];
const A: [&str; 4] = ["a-0", "a-1", "a-2", "a-3"];

/// `delegate-board` (affectUI Delegate Board): the host passes agents, tray and hand-overs.
#[derive(IntoElement)]
pub struct DelegateBoardEl {
    agents: Vec<DbAgent>,
    tray: Vec<DbTask>,
    held: Vec<(usize, DbTask, DbStatus)>,
    selected: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for DelegateBoardEl {
    fn default() -> Self {
        Self {
            agents: db_agents(),
            tray: db_tasks(),
            held: vec![],
            selected: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl DelegateBoardEl {
    /// Agents and the unassigned tray.
    pub fn board(mut self, agents: Vec<DbAgent>, tray: Vec<DbTask>) -> Self {
        self.agents = agents;
        self.tray = tray;
        self
    }
    /// Handed-over tasks: (agent index, task, status).
    pub fn held(mut self, h: Vec<(usize, DbTask, DbStatus)>) -> Self {
        self.held = h;
        self
    }
    /// The selected tray task.
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
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
    /// Tray picks and hand-overs.
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
        let (card, bd, fg, mute, box_bg, blue, ok, band) = if dark {
            (
                h(0x18181B),
                h(0x2C2C31),
                h(0xEDEDEF),
                h(0x8B8B94),
                h(0x1B1B1F),
                h(0x7AA5FF),
                h(0x4ADE80),
                h(0x1D1D21),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE6E6E9),
                h(0x18181B),
                h(0x8A8A93),
                h(0xFAFAFA),
                h(0x2F6FED),
                h(0x16A34A),
                h(0xF4F4F5),
            )
        };
        let picked = self.selected.and_then(|i| self.tray.get(i).cloned());
        let mut tray = div()
            .w(px(210.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(mute)
                    .child("UNASSIGNED"),
            );
        for (i, t) in self.tray.iter().enumerate().take(8) {
            let f = fire(&self.act, T[i]);
            let on = self.selected == Some(i);
            tray = tray.child(
                div()
                    .id(SharedString::from(format!("oxdb-t{i}")))
                    .p(px(9.))
                    .border_1()
                    .border_color(if on { blue } else { bd })
                    .rounded(px(10.))
                    .bg(card)
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(t.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(mute)
                            .child(format!("{} · ~{} min", t.skill, t.est_s)),
                    ),
            );
        }
        let mut cols = div().flex_1().p(px(14.)).flex().gap(px(10.));
        for (ai, a) in self.agents.iter().enumerate().take(4) {
            let fit = picked.as_ref().map(|t| db_fits(a, t));
            let mut col = div()
                .flex_1()
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .border_1()
                .border_color(if fit == Some(true) { blue } else { bd })
                .rounded(px(12.))
                .bg(card)
                .when(fit == Some(false), |d| d.opacity(0.5))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(a.name.clone()),
                )
                .child(div().text_size(px(11.5)).text_color(mute).child(format!(
                    "{} · {}",
                    a.model,
                    a.skills.join(", ")
                )));
            if fit == Some(true) {
                let f = fire(&self.act, A[ai]);
                col = col.child(
                    div()
                        .id(SharedString::from(format!("oxdb-a{ai}")))
                        .px(px(8.))
                        .py(px(5.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(blue)
                        .text_color(blue)
                        .text_size(px(12.5))
                        .on_click(move |_, w, cx| f(w, cx))
                        .child(format!("{}  Hand over", ai + 1)),
                );
            }
            for (_, t, s) in self.held.iter().filter(|(x, _, _)| *x == ai) {
                let tone = match s {
                    DbStatus::Running => blue,
                    DbStatus::Done => ok,
                    DbStatus::Queued => mute,
                };
                col = col.child(
                    div()
                        .px(px(8.))
                        .py(px(6.))
                        .rounded(px(8.))
                        .bg(box_bg)
                        .flex()
                        .justify_between()
                        .text_size(px(12.))
                        .child(t.title.clone())
                        .child(div().text_color(tone).child(s.label())),
                );
            }
            cols = cols.child(col);
        }
        div()
            .w(px(780.))
            .flex()
            .flex_col()
            .border_1()
            .border_color(bd)
            .rounded(px(12.))
            .bg(card)
            .text_color(fg)
            .child(
                div()
                    .h(px(48.))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(bd)
                    .when(self.classic, |d| d.bg(band))
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Delegate"),
            )
            .child(div().flex().bg(box_bg).child(tray).child(cols))
    }
}
