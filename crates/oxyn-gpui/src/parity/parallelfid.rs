//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Parallel Attempts**
//! ([`ParallelAttemptsEl`]) over `oxyn_core::parity::parallelfid`. The element is `RenderOnce`:
//! the host passes the attempts and the one in use and gets [`Act`] ids back (`use-<n>` for the
//! n-th attempt). A static snapshot: no shine or count-up.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::parity::parallelfid::{
    PaAttempt, PaStatus, pa_at, pa_best, pa_cost, pa_demo, pa_dur, pa_finished, pa_len, pa_usd,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const U: [&str; 6] = ["use-0", "use-1", "use-2", "use-3", "use-4", "use-5"];

/// `parallel-attempts` (affectUI Parallel Attempts): the host passes the race.
#[derive(IntoElement)]
pub struct ParallelAttemptsEl {
    attempts: Vec<PaAttempt>,
    task: String,
    picked: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ParallelAttemptsEl {
    fn default() -> Self {
        let d = pa_demo();
        Self {
            attempts: pa_at(&d, pa_len(&d)),
            task: "Steady the flaky payment test".into(),
            picked: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ParallelAttemptsEl {
    /// The attempts and the task.
    pub fn attempts(mut self, a: Vec<PaAttempt>, task: &str) -> Self {
        self.attempts = a;
        self.task = task.into();
        self
    }
    /// The attempt in use (the best when `None`).
    pub fn picked(mut self, i: Option<usize>) -> Self {
        self.picked = i;
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
    /// Use buttons.
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

impl RenderOnce for ParallelAttemptsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, track, blue, green, wash, red) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2E),
                h(0xEDEDEF),
                h(0x8A8A93),
                h(0x26262B),
                h(0x4C8DFF),
                h(0x34D399),
                h(0x15241E),
                h(0xF87171),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE5E7EB),
                h(0x111827),
                h(0x6B7280),
                h(0xEEF0F2),
                h(0x2563EB),
                h(0x16A34A),
                h(0xF0FDF4),
                h(0xDC2626),
            )
        };
        let a = &self.attempts;
        let best = pa_best(a);
        let used = self.picked.or(best);
        let mut list = div().flex().flex_col().gap(px(8.));
        for (i, x) in a.iter().enumerate().take(6) {
            let f = fire(&self.act, U[i]);
            let in_use = used == Some(i) && x.status == PaStatus::Done;
            let fill = match x.status {
                PaStatus::Running => blue,
                PaStatus::Failed => red,
                PaStatus::Done if in_use => green,
                PaStatus::Done => mute,
            };
            let right = match x.status {
                PaStatus::Running => format!("{:.0}%", x.progress * 100.0),
                PaStatus::Failed => "Failed".into(),
                PaStatus::Done => format!(
                    "{}{}",
                    if best == Some(i) { "Best " } else { "" },
                    x.score.unwrap_or(0)
                ),
            };
            list = list.child(
                div()
                    .id(SharedString::from(format!("oxpt-{i}")))
                    .h(px(48.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .border_1()
                    .border_color(if in_use { green } else { bd })
                    .rounded(px(12.))
                    .bg(if in_use { wash } else { card })
                    .when(x.status == PaStatus::Failed, |d| d.opacity(0.58))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .w(px(220.))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("{} · {}", x.model, x.approach)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(if x.status == PaStatus::Running {
                                        blue
                                    } else {
                                        mute
                                    })
                                    .child(if x.status == PaStatus::Running {
                                        x.step.clone()
                                    } else {
                                        x.note.clone()
                                    }),
                            ),
                    )
                    .child(
                        div().flex_1().h(px(6.)).rounded_full().bg(track).child(
                            div()
                                .h_full()
                                .w(relative(x.progress as f32))
                                .rounded_full()
                                .bg(fill),
                        ),
                    )
                    .child(
                        div()
                            .w(px(84.))
                            .flex()
                            .flex_col()
                            .items_end()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(match x.status {
                                        PaStatus::Failed => red,
                                        _ if in_use => green,
                                        _ => fg,
                                    })
                                    .child(right),
                            )
                            .child(div().text_size(px(11.)).text_color(mute).child(format!(
                                "{} · {}",
                                pa_dur(x.took_s),
                                pa_usd(x.cost)
                            ))),
                    )
                    .child(
                        div()
                            .w(px(84.))
                            .text_size(px(11.5))
                            .text_color(if in_use { green } else { mute })
                            .child(match x.status {
                                PaStatus::Done if in_use => "✓ In use",
                                PaStatus::Done => "Use this one",
                                _ => "",
                            }),
                    ),
            );
        }
        div()
            .w(px(720.))
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .border_1()
            .border_color(bd)
            .rounded(px(16.))
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
                            .child(format!("Best of {}", a.len())),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(mute)
                            .child(self.task.clone()),
                    )
                    .child(div().flex_1())
                    .child(div().text_size(px(12.)).text_color(mute).child(format!(
                        "{} of {} done",
                        pa_finished(a),
                        a.len()
                    )))
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(pa_usd(pa_cost(a))),
                    ),
            )
            .when(self.classic, |d| d.rounded(px(14.)))
            .child(list)
    }
}
