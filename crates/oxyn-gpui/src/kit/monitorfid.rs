//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32) for GPUI: the exact
//! **Run monitor** ([`RunMonitorEl`]) over `oxyn_core::kit::monitorfid`. The element is
//! `RenderOnce`: the host keeps the scene, the fold and the open step, and routes [`Act`] ids
//! back (`fold`, `unfold`, `allow`, `decline`, `pause`, `cancel`, `retry`, `log`, `step-<n>`
//! for the n-th finished row from the top, `close`). A static snapshot: no shimmer or roll.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::monitorfid::{
    MonStepState, MonitorScene, MonitorState, MonitorStep, monitor_at, monitor_clock, monitor_cost,
    step_count, step_share, step_took, time_strip,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};
use crate::brand_mark;

const W: f32 = 340.0;
const STEP_ACTS: [&str; 3] = ["step-0", "step-1", "step-2"];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    fg: Hsla,
    fg2: Hsla,
    mute: Hsla,
    faint: Hsla,
    well: Hsla,
    chip: Hsla,
    blue: Hsla,
    blue_bg: Hsla,
    seg: Hsla,
    amber: Hsla,
    amber_bg: Hsla,
    amber_dot: Hsla,
    green: Hsla,
    green_bg: Hsla,
    green_dot: Hsla,
    red: Hsla,
    red_bg: Hsla,
    red_dot: Hsla,
    ink: Hsla,
    ink_fg: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x1C1C1E),
            line: h(0x323235),
            fg: h(0xFAFAFA),
            fg2: h(0xD4D4D8),
            mute: h(0x8E8E95),
            faint: h(0x626268),
            well: h(0x262628),
            chip: h(0x2A2A2D),
            blue: h(0x6AAEFF),
            blue_bg: h(0x1E2B3D),
            seg: h(0x3A3A3E),
            amber: h(0xF5B454),
            amber_bg: h(0x332818),
            amber_dot: h(0xF5A524),
            green: h(0x4ADE80),
            green_bg: h(0x1B3022),
            green_dot: h(0x22C55E),
            red: h(0xF87171),
            red_bg: h(0x3A1F20),
            red_dot: h(0xEF4444),
            ink: h(0xFAFAFA),
            ink_fg: h(0x18181B),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xE9E9EC),
            fg: h(0x18181B),
            fg2: h(0x3F3F46),
            mute: h(0x71717A),
            faint: h(0xA1A1AA),
            well: h(0xF4F4F5),
            chip: h(0xF1F1F3),
            blue: h(0x2563EB),
            blue_bg: h(0xEAF2FE),
            seg: h(0xE4E4E7),
            amber: h(0xB45309),
            amber_bg: h(0xFEF4E2),
            amber_dot: h(0xF59E0B),
            green: h(0x15803D),
            green_bg: h(0xE8F6EE),
            green_dot: h(0x22C55E),
            red: h(0xB91C1C),
            red_bg: h(0xFDECEC),
            red_dot: h(0xEF4444),
            ink: h(0x18181B),
            ink_fg: h(0xFFFFFF),
        }
    }
}

/// `agent-monitor` (affectUI Agent Monitor): the host passes the scene, whether it is folded and
/// the open step (index among the finished rows, newest first).
#[derive(IntoElement)]
pub struct RunMonitorEl {
    scene: MonitorScene,
    folded: bool,
    open: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for RunMonitorEl {
    fn default() -> Self {
        Self {
            scene: monitor_at(5_200, None, None),
            folded: false,
            open: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl RunMonitorEl {
    /// The demo `ms` into its pass.
    pub fn demo(mut self, ms: u64) -> Self {
        self.scene = monitor_at(ms, None, None);
        self
    }
    /// Your own run.
    pub fn scene(mut self, s: MonitorScene) -> Self {
        self.scene = s;
        self
    }
    /// Folded into the small bar.
    pub fn folded(mut self, on: bool) -> Self {
        self.folded = on;
        self
    }
    /// The finished row (newest first) whose details are open.
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

fn tile(s: &MonitorStep, p: Pal, size: f32) -> gpui::Div {
    let t = div()
        .size(px(size))
        .flex_none()
        .rounded(px(5.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.))
        .text_color(p.fg2);
    match brand_mark(&s.brand, size * 0.62, p.fg) {
        Some(m) => t.child(m),
        None => t.child(">_"),
    }
}

impl RenderOnce for RunMonitorEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let sc = &self.scene;
        let (pfg, pbg, pdot) = match sc.state {
            MonitorState::Running | MonitorState::Paused => (p.blue, p.blue_bg, p.blue),
            MonitorState::Waiting => (p.amber, p.amber_bg, p.amber_dot),
            MonitorState::Finished => (p.green, p.green_bg, p.green_dot),
            MonitorState::Failed => (p.red, p.red_bg, p.red_dot),
            _ => (p.mute, p.chip, p.mute),
        };
        let pill = |txt: String| {
            div()
                .h(px(20.))
                .px(px(7.))
                .rounded(px(6.))
                .bg(pbg)
                .text_color(pfg)
                .text_size(px(11.5))
                .font_weight(FontWeight::MEDIUM)
                .flex()
                .items_center()
                .gap(px(5.))
                .child(div().size(px(6.)).rounded_full().bg(pdot))
                .child(txt)
        };
        let logo = || match brand_mark(&sc.brand, 16.0, p.fg) {
            Some(m) => div()
                .size(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .child(m),
            None => div()
                .size(px(20.))
                .child(sc.agent.chars().next().unwrap_or('·').to_string()),
        };
        let r = if self.classic { 16. } else { 12. };
        if self.folded {
            let used: Vec<&MonitorStep> = sc
                .steps
                .iter()
                .filter(|s| s.state != MonStepState::Queued)
                .collect();
            let mut stack = div().flex().items_center().gap(px(2.));
            for s in used.iter().rev().take(4).rev() {
                stack = stack.child(tile(s, p, 22.));
            }
            let total: u64 = sc.steps.iter().map(|s| s.took).sum();
            let txt = match sc.state {
                MonitorState::Finished => step_took(total),
                s if s.busy() => monitor_clock(sc.clock),
                s => s.label().to_string(),
            };
            return div()
                .id("oxmn-fold")
                .w(px(W))
                .h(px(44.))
                .px(px(10.))
                .rounded(px(r))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .shadow_md()
                .text_color(p.fg)
                .flex()
                .items_center()
                .gap(px(8.))
                .cursor_pointer()
                .on_click({
                    let f = fire(&self.act, "unfold");
                    move |_, w, cx| f(w, cx)
                })
                .child(logo())
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(sc.agent.clone()),
                )
                .child(stack)
                .child(div().flex_1())
                .child(pill(txt))
                .into_any_element();
        }
        let (ca, cb) = monitor_cost(sc.cost);
        let head = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(logo())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(sc.agent.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(p.mute)
                            .child(sc.role.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .text_size(px(11.5))
                    .child(div().text_color(p.fg).child(ca))
                    .child(div().text_color(p.faint).child(cb)),
            )
            .child(pill(sc.state.label().to_string()))
            .child(
                div()
                    .id("oxmn-foldaway")
                    .size(px(22.))
                    .rounded(px(6.))
                    .text_color(p.faint)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click({
                        let f = fire(&self.act, "fold");
                        move |_, w, cx| f(w, cx)
                    })
                    .child("⌃"),
            );
        let running = sc
            .steps
            .iter()
            .rev()
            .find(|s| s.state == MonStepState::Running);
        let clock = monitor_clock(sc.clock);
        let mut well = div()
            .h(px(36.))
            .pl(px(8.))
            .pr(px(10.))
            .rounded(px(8.))
            .bg(if sc.state == MonitorState::Waiting {
                p.amber_bg
            } else {
                p.well
            })
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(12.5));
        if let (MonitorState::Waiting, Some(a)) = (sc.state, sc.ask.as_ref()) {
            let mut t = div()
                .size(px(20.))
                .rounded(px(5.))
                .bg(p.card)
                .flex()
                .items_center()
                .justify_center();
            if let Some(m) = brand_mark(&a.brand, 13.0, p.fg) {
                t = t.child(m);
            }
            well = well
                .child(t)
                .child(div().font_weight(FontWeight::MEDIUM).child(a.title.clone()))
                .child(
                    div().flex_1().overflow_hidden().child(
                        div()
                            .px(px(6.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(p.line)
                            .bg(p.card)
                            .text_size(px(11.5))
                            .child(a.command.clone()),
                    ),
                )
                .child(div().text_color(p.mute).child(clock.clone()));
        } else if let Some(s) = running {
            well = well
                .child(tile(s, p, 20.))
                .child(div().flex_1().overflow_hidden().child(s.label.clone()))
                .child(div().text_color(p.mute).child(clock.clone()));
        } else {
            let (txt, col) = match sc.state {
                MonitorState::Finished => (format!("Finished in {}", step_took(sc.clock)), p.green),
                MonitorState::Failed => (sc.error.clone(), p.red),
                MonitorState::Cancelled => (format!("Cancelled at {clock}"), p.mute),
                _ => ("Waiting to start".into(), p.mute),
            };
            well = well.child(div().text_color(col).child(txt));
        }
        let mut strip = div().h(px(4.)).flex().gap(px(3.));
        let fin = sc.state == MonitorState::Finished;
        for (i, share) in time_strip(&sc.steps, sc.clock) {
            let col = match sc.steps[i].state {
                _ if fin => p.green_dot,
                MonStepState::Running if sc.state == MonitorState::Waiting => p.amber_dot,
                MonStepState::Running => p.blue,
                MonStepState::Failed => p.red_dot,
                _ => p.seg,
            };
            strip = strip.child(
                div()
                    .h(px(4.))
                    .w(px((W - 24.0 - 15.0) * share))
                    .min_w(px(4.))
                    .rounded(px(2.))
                    .bg(col),
            );
        }
        let done: Vec<(usize, &MonitorStep)> = sc
            .steps
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.state, MonStepState::Done | MonStepState::Failed))
            .collect();
        let mut rows = div().flex().flex_col();
        for (k, (i, s)) in done.iter().rev().take(3).enumerate() {
            let open = self.open == Some(k);
            rows = rows.child(
                div()
                    .id(SharedString::from(format!("oxmn-row-{k}")))
                    .h(px(26.))
                    .px(px(2.))
                    .rounded(px(6.))
                    .when(open, |d| d.bg(p.well))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(12.))
                    .cursor_pointer()
                    .on_click({
                        let f = fire(&self.act, STEP_ACTS[k]);
                        move |_, w, cx| f(w, cx)
                    })
                    .child(tile(s, p, 18.))
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_color(p.fg2)
                            .child(s.label.clone()),
                    )
                    .child(if s.state == MonStepState::Failed {
                        div().text_color(p.red).child("Failed")
                    } else {
                        div().text_color(p.faint).child(step_took(s.took))
                    }),
            );
            if open {
                let cell = |k: &str, v: String| {
                    div()
                        .flex_1()
                        .px(px(8.))
                        .py(px(6.))
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(p.mute)
                                .child(k.to_string()),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(v),
                        )
                };
                rows = rows.child(
                    div()
                        .my(px(4.))
                        .p(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(tile(s, p, 20.))
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_size(px(12.5))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(s.label.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.))
                                                .text_color(p.mute)
                                                .child(s.tool.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("oxmn-close")
                                        .text_color(p.faint)
                                        .cursor_pointer()
                                        .on_click({
                                            let f = fire(&self.act, "close");
                                            move |_, w, cx| f(w, cx)
                                        })
                                        .child("✕"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(p.line)
                                .child(cell("Started", format!("+{}", monitor_clock(s.since))))
                                .child(cell("Took", step_took(s.took)))
                                .child(cell("Share", step_share(&sc.steps, *i)))
                                .child(cell(
                                    "State",
                                    if s.state == MonStepState::Failed {
                                        "Failed".into()
                                    } else {
                                        "Done".into()
                                    },
                                )),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.mute)
                                .child(s.detail.clone()),
                        )
                        .child(
                            div()
                                .px(px(6.))
                                .rounded(px(6.))
                                .bg(p.well)
                                .text_size(px(11.5))
                                .child(s.output.clone()),
                        ),
                );
            }
        }
        let btn = |txt: &'static str, id: &'static str, primary: bool| {
            div()
                .id(SharedString::from(format!("oxmn-{id}")))
                .h(px(24.))
                .px(px(8.))
                .rounded(px(6.))
                .bg(if primary { p.ink } else { p.chip })
                .text_color(if primary { p.ink_fg } else { p.fg2 })
                .text_size(px(11.5))
                .flex()
                .items_center()
                .cursor_pointer()
                .on_click({
                    let f = fire(&self.act, id);
                    move |_, w, cx| f(w, cx)
                })
                .child(txt)
        };
        let mut foot = div().flex().items_center().gap(px(6.));
        foot =
            match sc.state {
                MonitorState::Waiting => foot
                    .child(btn("Decline  Esc", "decline", false))
                    .child(btn("Allow  ↵", "allow", true)),
                MonitorState::Running | MonitorState::Paused => foot
                    .child(btn("⏸ Pause", "pause", false))
                    .child(btn("✕ Cancel", "cancel", false)),
                MonitorState::Failed => foot.child(btn("↻ Try again", "retry", true)),
                _ => foot,
            };
        foot = foot
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(p.faint)
                    .child(step_count(&sc.steps)),
            )
            .child(btn("Log ↗", "log", false));
        div()
            .w(px(W))
            .p(px(12.))
            .rounded(px(r))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .shadow_md()
            .text_color(p.fg)
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(head)
            .child(well)
            .child(strip)
            .child(rows)
            .child(foot)
            .into_any_element()
    }
}
