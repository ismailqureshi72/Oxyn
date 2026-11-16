//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32) for GPUI: the exact
//! **Agent Lanes** ([`AgentLanesEl`]) over `oxyn_core::parity::lanesfid`. The element is
//! `RenderOnce`: the host keeps the clock and the open block and routes [`Act`] ids back
//! (`task-<n>` opens task n's summary or decision, `approve`, `reject`, `close`). A static
//! snapshot: hand-offs are drawn as a dashed elbow, the running shimmer is not animated.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::lanesfid::{
    LaneScene, LaneStatus, block_span, clipped_left, lane_counts, lane_duration, lane_tag, lanes_at,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const W: f32 = 760.0;
const NAMES: f32 = 175.0;
const LANE_H: f32 = 50.0;
const TASK_ACTS: [&str; 16] = [
    "task-0", "task-1", "task-2", "task-3", "task-4", "task-5", "task-6", "task-7", "task-8",
    "task-9", "task-10", "task-11", "task-12", "task-13", "task-14", "task-15",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    grid: Hsla,
    fg: Hsla,
    mute: Hsla,
    faint: Hsla,
    chip: Hsla,
    blue: Hsla,
    blue_bg: Hsla,
    run: Hsla,
    cap: Hsla,
    done: Hsla,
    done_fg: Hsla,
    fail: Hsla,
    fail_fg: Hsla,
    wait: Hsla,
    wait_fg: Hsla,
    amber: Hsla,
    curve: Hsla,
    tip: Hsla,
    tip_fg: Hsla,
    tip_mute: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x18181B),
            line: h(0x2A2A2E),
            grid: h(0x262629),
            fg: h(0xFAFAFA),
            mute: h(0x8B8B93),
            faint: h(0x6B6B73),
            chip: h(0x26262A),
            blue: h(0x7EB6FF),
            blue_bg: h(0x1E3A5F),
            run: h(0x1F3E63),
            cap: h(0x4C9BFF),
            done: h(0x26262A),
            done_fg: h(0xD4D4D8),
            fail: h(0x3A1F22),
            fail_fg: h(0xFCA5A5),
            wait: h(0x3D2C10),
            wait_fg: h(0xFBBF24),
            amber: h(0xF2A33A),
            curve: h(0x4F6A8F),
            tip: h(0xF4F4F5),
            tip_fg: h(0x18181B),
            tip_mute: h(0x71717A),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xE7E7EA),
            grid: h(0xEFEFF1),
            fg: h(0x18181B),
            mute: h(0x71717A),
            faint: h(0xA1A1AA),
            chip: h(0xF1F1F3),
            blue: h(0x2563EB),
            blue_bg: h(0xE8F1FE),
            run: h(0xDCEAFD),
            cap: h(0x2E7CF6),
            done: h(0xEDEDEF),
            done_fg: h(0x3F3F46),
            fail: h(0xFDECEC),
            fail_fg: h(0xB42318),
            wait: h(0xFBE7C6),
            wait_fg: h(0xB45309),
            amber: h(0xD9822B),
            curve: h(0x9FB6D6),
            tip: h(0x18181B),
            tip_fg: h(0xF4F4F5),
            tip_mute: h(0xA1A1AA),
        }
    }
}

/// `agent-lanes` (affectUI Agent Lanes): the host passes the scene (the demo's at some ms, or
/// its own) and the open task; acts `task-<n>`, `approve`, `reject` and `close`.
#[derive(IntoElement)]
pub struct AgentLanesEl {
    scene: LaneScene,
    title: String,
    open: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AgentLanesEl {
    fn default() -> Self {
        Self {
            scene: lanes_at(9_000),
            title: "Agents".into(),
            open: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AgentLanesEl {
    /// The demo board `ms` into its pass.
    pub fn demo(mut self, ms: u64) -> Self {
        self.scene = lanes_at(ms);
        self
    }
    /// Your own board.
    pub fn scene(mut self, scene: LaneScene) -> Self {
        self.scene = scene;
        self
    }
    /// Heading of the bar.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.into();
        self
    }
    /// The task (index) whose summary or decision is open.
    pub fn open(mut self, i: Option<usize>) -> Self {
        self.open = i;
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
    /// Acts.
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

impl RenderOnce for AgentLanesEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let sc = &self.scene;
        let span = sc.span.max(1);
        let now = sc.now;
        let (running, waiting) = lane_counts(&sc.tasks);
        let tl_w = W - NAMES - 16.0 - 2.0;
        let grid_h = sc.agents.len() as f32 * LANE_H;
        let r = if self.classic { 16. } else { 12. };

        let dot = |col: Hsla| div().size(px(6.)).rounded_full().bg(col);
        let mut bar = div()
            .h(px(48.))
            .pl(px(18.))
            .pr(px(16.))
            .flex()
            .items_center()
            .gap(px(10.))
            .border_b_1()
            .border_color(p.line)
            .when(self.classic, |d| d.bg(p.chip).rounded_t(px(r)))
            .child(div().text_color(p.blue).child("✦"))
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(12.5))
                    .text_color(p.mute)
                    .child(dot(p.cap))
                    .child(format!("{running} running")),
            );
        if waiting > 0 {
            bar = bar.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(12.5))
                    .text_color(p.wait_fg)
                    .child(dot(p.amber))
                    .child(format!("{waiting} needs you")),
            );
        }
        bar = bar.child(div().flex_1()).child(
            div()
                .h(px(24.))
                .px(px(9.))
                .rounded(px(7.))
                .bg(p.chip)
                .text_color(p.mute)
                .text_size(px(12.5))
                .flex()
                .items_center()
                .child(format!("Last {}m", span / 60_000)),
        );

        // Ruler.
        let mut ruler = div().relative().h(px(30.)).ml(px(NAMES)).w(px(tl_w));
        let step = 5 * 60_000u64;
        let mut d = step;
        let mut ticks = vec![];
        while d < span {
            let x = (1.0 - d as f32 / span as f32) * tl_w;
            ticks.push(x);
            ruler = ruler.child(
                div()
                    .absolute()
                    .top(px(12.))
                    .left(px(x - 14.0))
                    .w(px(28.))
                    .flex()
                    .justify_center()
                    .text_size(px(11.5))
                    .text_color(p.faint)
                    .child(format!("−{}m", d / 60_000)),
            );
            d += step;
        }
        ruler = ruler.child(
            div()
                .absolute()
                .right(px(0.))
                .top(px(9.))
                .h(px(17.))
                .px(px(6.))
                .rounded(px(6.))
                .bg(p.blue_bg)
                .text_color(p.blue)
                .text_size(px(11.))
                .child("now"),
        );

        // Names.
        let mut names = div().w(px(NAMES)).flex().flex_col();
        for a in sc.agents.iter() {
            let tag = lane_tag(&a.id, &sc.tasks);
            let mut n1 = div().flex().items_center().gap(px(6.)).child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(a.name.clone()),
            );
            if let Some(tg) = tag {
                let (bg, fg) = if tg == "Idle" {
                    (p.chip, p.mute)
                } else {
                    (alpha(p.wait, 0.6), p.wait_fg)
                };
                n1 = n1.child(
                    div()
                        .h(px(18.))
                        .px(px(6.))
                        .rounded(px(5.))
                        .bg(bg)
                        .text_color(fg)
                        .text_size(px(11.5))
                        .child(tg),
                );
            }
            let mut tile = div()
                .size(px(28.))
                .flex_none()
                .rounded(px(if self.classic { 14. } else { 8. }))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .flex()
                .items_center()
                .justify_center();
            tile = match brand_mark(&a.brand, 15.0, p.fg) {
                Some(m) => tile.child(m),
                None => tile.child(a.name.chars().next().unwrap_or('·').to_string()),
            };
            names = names.child(
                div()
                    .h(px(LANE_H))
                    .pl(px(16.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(tile)
                    .child(
                        div().flex().flex_col().child(n1).child(
                            div()
                                .text_size(px(12.5))
                                .text_color(p.mute)
                                .child(a.model.clone()),
                        ),
                    ),
            );
        }

        // Timeline.
        let mut tl = div()
            .relative()
            .w(px(tl_w))
            .h(px(grid_h))
            .border_t_1()
            .border_b_1()
            .border_color(p.grid);
        for x in ticks.iter() {
            tl = tl.child(
                div()
                    .absolute()
                    .left(px(*x))
                    .top(px(0.))
                    .w(px(1.))
                    .h(px(grid_h))
                    .bg(p.grid),
            );
        }
        let lane_of = |agent: &str| sc.agents.iter().position(|a| a.id == agent);
        // Hand-offs: a dashed elbow from the end of one block to the start of the next.
        for t in sc.tasks.iter().filter(|t| !t.handoff_to.is_empty()) {
            let Some(n) = sc.tasks.iter().find(|x| x.id == t.handoff_to) else {
                continue;
            };
            let (Some(la), Some(lb)) = (lane_of(&t.agent), lane_of(&n.agent)) else {
                continue;
            };
            let (Some((l0, w0)), Some((l1, _))) =
                (block_span(t, now, span), block_span(n, now, span))
            else {
                continue;
            };
            let x0 = (l0 + w0) * tl_w;
            let x1 = l1 * tl_w;
            let y0 = la as f32 * LANE_H + LANE_H / 2.0;
            let y1 = lb as f32 * LANE_H + LANE_H / 2.0;
            let mx = (x0 + x1) / 2.0;
            let (top, bot) = (y0.min(y1), y0.max(y1));
            let mut yy = top;
            while yy < bot {
                tl = tl.child(
                    div()
                        .absolute()
                        .left(px(mx))
                        .top(px(yy))
                        .w(px(1.2))
                        .h(px(3.))
                        .bg(p.curve),
                );
                yy += 6.0;
            }
            for (a, b, y) in [(x0, mx, y0), (mx, x1, y1)] {
                let mut xx = a.min(b);
                while xx < a.max(b) {
                    tl = tl.child(
                        div()
                            .absolute()
                            .left(px(xx))
                            .top(px(y))
                            .w(px(3.))
                            .h(px(1.2))
                            .bg(p.curve),
                    );
                    xx += 6.0;
                }
            }
        }
        let mut pop = None;
        for (i, t) in sc.tasks.iter().enumerate().take(TASK_ACTS.len()) {
            let Some(li) = lane_of(&t.agent) else {
                continue;
            };
            let Some((l, w)) = block_span(t, now, span) else {
                continue;
            };
            let (bg, fg) = match t.status {
                LaneStatus::Done => (p.done, p.done_fg),
                LaneStatus::Running => (p.run, p.blue),
                LaneStatus::Failed => (p.fail, p.fail_fg),
                LaneStatus::Waiting => (p.wait, p.wait_fg),
            };
            let (x, bw) = (l * tl_w, (w * tl_w).max(4.0));
            let y = li as f32 * LANE_H + 11.0;
            let mut b = div()
                .id(SharedString::from(format!("oxln-b-{i}")))
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(bw))
                .h(px(28.))
                .pl(px(7.))
                .pr(px(6.))
                .rounded(px(if self.classic { 9. } else { 7. }))
                .when(clipped_left(t, now, span), |d| d.rounded_l(px(0.)))
                .when(self.classic, |d| d.border_1().border_color(alpha(fg, 0.22)))
                .when(t.status == LaneStatus::Running, |d| {
                    d.border_r_2().border_color(p.cap)
                })
                .bg(bg)
                .text_color(fg)
                .text_size(px(13.))
                .font_weight(FontWeight::MEDIUM)
                .overflow_hidden()
                .flex()
                .items_center()
                .gap(px(5.))
                .cursor_pointer()
                .on_click({
                    let f = fire(&self.act, TASK_ACTS[i]);
                    move |_, w, cx| f(w, cx)
                });
            for tool in t.tools.iter().take(3) {
                let mut chip = div()
                    .size(px(16.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center();
                if let Some(m) = brand_mark(tool, 11.0, fg) {
                    chip = chip.child(m);
                }
                b = b.child(chip);
            }
            b = b.child(div().flex_1().overflow_hidden().child(t.title.clone()));
            if t.status == LaneStatus::Waiting {
                b = b.child(
                    div()
                        .h(px(20.))
                        .px(px(7.))
                        .flex_none()
                        .rounded(px(6.))
                        .border_1()
                        .border_color(alpha(p.amber, 0.5))
                        .bg(p.card)
                        .text_size(px(11.5))
                        .flex()
                        .items_center()
                        .child("▲ Needs you"),
                );
            }
            tl = tl.child(b);
            if self.open == Some(i) {
                let right = l + w > 0.6;
                pop = Some((i, x, bw, y + 36.0, right));
            }
        }
        if let Some((i, x, bw, y, right)) = pop {
            let t = &sc.tasks[i];
            let agent = sc
                .agents
                .iter()
                .find(|a| a.id == t.agent)
                .map(|a| a.name.clone())
                .unwrap_or_default();
            let dur = lane_duration(t.end.unwrap_or(now).min(now).saturating_sub(t.start));
            let (w, card) = if t.status == LaneStatus::Waiting {
                let btn = |label: &'static str, id: &'static str, primary: bool| {
                    div()
                        .id(SharedString::from(format!("oxln-{id}")))
                        .h(px(32.))
                        .px(px(13.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(if primary { p.fg } else { p.line })
                        .bg(if primary { p.fg } else { p.card })
                        .text_color(if primary { p.card } else { p.fg })
                        .text_size(px(13.))
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .on_click({
                            let f = fire(&self.act, id);
                            move |_, w, cx| f(w, cx)
                        })
                        .child(label)
                };
                (
                    280.0,
                    div()
                        .p(px(14.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_size(px(14.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.fg)
                                .child(t.title.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(p.mute)
                                .child(t.ask.clone()),
                        )
                        .child(
                            div()
                                .mt(px(6.))
                                .flex()
                                .justify_end()
                                .gap(px(6.))
                                .child(btn("Reject", "reject", false))
                                .child(btn("Approve", "approve", true)),
                        ),
                )
            } else {
                let mut tip = div()
                    .px(px(12.))
                    .py(px(10.))
                    .rounded(px(10.))
                    .bg(p.tip)
                    .text_color(p.tip_fg)
                    .text_size(px(12.5))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap(px(5.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.tip_mute)
                            .child(format!("{agent} · {} · {dur}", t.status.label())),
                    );
                for s in t.summary.iter() {
                    tip = tip.child(format!("• {s}"));
                }
                (250.0, tip)
            };
            let left = if right { x + bw - w } else { x };
            tl = tl.child(
                div()
                    .absolute()
                    .left(px(left))
                    .top(px(y))
                    .w(px(w))
                    .child(card),
            );
        }

        div()
            .w(px(W))
            .rounded(px(r))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.fg)
            .shadow_sm()
            .flex()
            .flex_col()
            .child(bar)
            .child(
                div()
                    .pb(px(12.))
                    .flex()
                    .flex_col()
                    .child(ruler)
                    .child(div().flex().child(names).child(tl)),
            )
    }
}
