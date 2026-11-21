//! Screens S5 in GPUI: Evals on the shared frame with the Evalkit persona's sidebar
//! (`oxyn_core::kit::workscr::persona("evalkit")`). The element draws one state at a time:
//! `.state()` for Data / Loading / Empty and `.extra()` for the own state (`running`) or an Oxyn
//! option (`fails` failures first, `cost` a cost column). Presses fire `nav-<key>`, `run-eval`,
//! `accept-all`, `reject-all` and `rerun` through `on_act`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::kit::buildscr::{
    COST_1K, DIFF, EVAL_MODELS, HISTORY, Piece, ScreenState, Verdict, avg_tenths, case_cost_tenths,
    case_id, case_view, eur, eval_cases, p50_ms, pass_rate, score_on, tenths, verdict_counts,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_core::kit::workscr::persona;
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::clip;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{frame, skel};
use super::workscr::{btn, none, page, quiet, side, tone};
use crate::brand::brand_mark;

/// Evals: tiles against the version before, pass rate by version, the prompt diff with Accept
/// all / Reject all, and the test cases with their verdicts.
#[derive(IntoElement)]
pub struct EvalsEl {
    state: ScreenState,
    extra: &'static str,
    layout: ShellLayout,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for EvalsEl {
    fn default() -> Self {
        Self {
            state: ScreenState::Sample,
            extra: "",
            layout: ShellLayout::Inset,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl EvalsEl {
    /// Sample, Loading or Empty.
    pub fn state(mut self, s: ScreenState) -> Self {
        self.state = s;
        self
    }
    /// `running`, `fails` or `cost` (see the module docs).
    pub fn extra(mut self, k: &'static str) -> Self {
        self.extra = k;
        self
    }
    /// Inset or Flat.
    pub fn layout(mut self, l: ShellLayout) -> Self {
        self.layout = l;
        self
    }
    /// Light or dark chrome.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Press ids (`nav-<key>`, `run-eval`, `accept-all`, `reject-all`, `rerun`).
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn verdict_tone(p: &ThPal, v: Verdict) -> Hsla {
    match v {
        Verdict::Pass => tone(p, IconTone::Done),
        Verdict::Partial => tone(p, IconTone::Approval),
        Verdict::Fail => tone(p, IconTone::Failed),
    }
}

/// A press target: fires `id` through `on_act`.
fn press(el: Div, key: SharedString, id: &'static str, act: &Option<Act>) -> Div {
    let f = fire(act, id);
    div().child(el.id(key).on_click(move |_, w, cx| f(w, cx)))
}

fn card(p: &ThPal) -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
}

fn delta(p: &ThPal, t: impl Into<SharedString>, good: bool) -> Div {
    div()
        .text_size(px(12.))
        .whitespace_nowrap()
        .text_color(tone(
            p,
            if good {
                IconTone::Done
            } else {
                IconTone::Failed
            },
        ))
        .child(t.into())
}

impl RenderOnce for EvalsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let cases = eval_cases(st == ScreenState::Sample);
        let n = cases.len();
        let (v, m) = (2usize, 0usize);
        let run = tone(&p, IconTone::Running);
        let mut b = div()
            .flex_1()
            .min_h_0()
            .px(px(16.))
            .py(px(14.))
            .flex()
            .flex_col()
            .gap(px(12.));
        if st == ScreenState::Loading {
            b = b.child(skel(&p, 9));
        } else if n == 0 {
            b = b.child(none(
                &p,
                "No eval runs yet",
                "Add test cases or bring them in from a dataset, then press Run eval.",
            ));
        } else {
            if self.extra == "running" {
                b = b.child(
                    card(&p)
                        .flex_row()
                        .items_center()
                        .gap(px(12.))
                        .px(px(14.))
                        .py(px(10.))
                        .child(quiet(&p, format!("Scoring {} · 8 of {n} done", case_id(9))))
                        .child(
                            div().flex_1().h(px(4.)).rounded(px(2.)).bg(p.line).child(
                                div()
                                    .h_full()
                                    .w(px(200. * 8. / n as f32))
                                    .rounded(px(2.))
                                    .bg(run),
                            ),
                        ),
                );
            }
            let tiles = [
                (
                    "Pass rate",
                    format!("{}%", pass_rate(&cases, v, m)),
                    "",
                    "+4 vs v11",
                    true,
                ),
                (
                    "Avg score",
                    tenths(avg_tenths(&cases, v, m)),
                    "/5",
                    "+0.2 vs v11",
                    true,
                ),
                (
                    "Latency p50",
                    tenths(p50_ms(&cases, v) / 100),
                    "s",
                    "−0.2 s vs v11",
                    true,
                ),
                (
                    "Cost per 1k runs",
                    eur(COST_1K[v]),
                    "",
                    "+€0.06 vs v11",
                    false,
                ),
            ];
            let mut tr = div().flex().gap(px(10.));
            for (l, val, unit, d, good) in tiles {
                tr = tr.child(
                    card(&p)
                        .flex_1()
                        .min_w_0()
                        .px(px(16.))
                        .py(px(14.))
                        .gap(px(12.))
                        .child(quiet(&p, l))
                        .child(
                            div()
                                .flex()
                                .items_end()
                                .gap(px(3.))
                                .child(
                                    div()
                                        .text_size(px(22.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(val),
                                )
                                .child(quiet(&p, unit))
                                .child(div().flex_1())
                                .child(delta(&p, d, good)),
                        ),
                );
            }
            let mut bars = div()
                .h(px(130.))
                .flex()
                .items_end()
                .gap(px(6.))
                .border_b_1()
                .border_color(p.line);
            for (i, r) in HISTORY.iter().enumerate() {
                bars = bars.child(
                    div()
                        .flex_1()
                        .h(px(1.2 * *r as f32))
                        .rounded_t(px(2.))
                        .bg(if i == 11 { run } else { run.opacity(0.3) }),
                );
            }
            let chart = card(&p)
                .flex_1()
                .min_w_0()
                .px(px(16.))
                .py(px(14.))
                .gap(px(8.))
                .child(quiet(&p, "Pass rate by version"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(18.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("v12 · 92%"),
                        )
                        .child(delta(&p, "+4%", true)),
                )
                .child(bars)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(quiet(&p, "v1"))
                        .child(quiet(&p, "v7"))
                        .child(quiet(&p, "v12")),
                )
                .child(quiet(
                    &p,
                    format!("Last 12 versions · {n} test cases · {}", EVAL_MODELS[m].0),
                ));
            let mut text = div()
                .flex()
                .flex_wrap()
                .text_size(px(13.))
                .line_height(px(22.));
            for piece in DIFF {
                text = match piece {
                    Piece::Same(t) => text.child(div().child(t)),
                    Piece::Del(t) => text.child(
                        div()
                            .px(px(3.))
                            .rounded(px(4.))
                            .line_through()
                            .text_color(p.mute)
                            .bg(tone(&p, IconTone::Failed).opacity(0.14))
                            .child(t),
                    ),
                    Piece::Add(t) => text.child(
                        div()
                            .px(px(3.))
                            .rounded(px(4.))
                            .bg(tone(&p, IconTone::Done).opacity(0.16))
                            .child(t),
                    ),
                };
            }
            let diff = card(&p)
                .flex_1()
                .min_w_0()
                .px(px(16.))
                .py(px(14.))
                .gap(px(8.))
                .child(quiet(
                    &p,
                    "Changes in v12 · system prompt, edited by Wren Ashby",
                ))
                .child(text)
                .child(div().flex_1())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(quiet(&p, "1/6"))
                        .child(div().flex_1())
                        .child(press(
                            btn(&p, "Reject all", false),
                            "reject-all".into(),
                            "reject-all",
                            &self.act,
                        ))
                        .child(press(
                            btn(&p, "Accept all", true),
                            "accept-all".into(),
                            "accept-all",
                            &self.act,
                        )),
                );
            let counts = verdict_counts(&cases, v, m);
            let cost = self.extra == "cost";
            let mut table = card(&p).overflow_hidden().child(
                div()
                    .h(px(38.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .text_color(p.mute)
                    .child(div().flex_1().child("Input"))
                    .child(div().flex_1().child("Expected"))
                    .child(div().w(px(100.)).child("Score"))
                    .child(div().w(px(56.)).child("Latency"))
                    .when(cost, |d| d.child(div().w(px(60.)).child("Cost"))),
            );
            for c in case_view(&cases, v, m, "all", self.extra == "fails")
                .into_iter()
                .take(8)
            {
                let vd = Verdict::of(score_on(&c, v, m));
                let mut inp = div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(quiet(&p, case_id(c.n)));
                if let Some(mk) = brand_mark(c.channel, 12., p.fg) {
                    inp = inp.child(mk);
                }
                table = table.child(
                    div()
                        .h(px(44.))
                        .px(px(14.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .border_t_1()
                        .border_color(p.line)
                        .child(inp.child(clip(c.input)))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_color(p.fg2)
                                .child(clip(c.expected)),
                        )
                        .child(
                            div().w(px(100.)).child(
                                div()
                                    .px(px(6.))
                                    .h(px(18.))
                                    .rounded(px(5.))
                                    .text_size(px(11.5))
                                    .bg(verdict_tone(&p, vd).opacity(0.22))
                                    .child(format!("{} {}/5", vd.label(), score_on(&c, v, m))),
                            ),
                        )
                        .child(div().w(px(56.)).child(format!("{}s", tenths(c.ms / 100))))
                        .when(cost, |d| {
                            d.child(
                                div()
                                    .w(px(60.))
                                    .child(format!("€0.{:03}", case_cost_tenths(&c))),
                            )
                        })
                        .child(press(
                            quiet(&p, "↻"),
                            format!("rerun-{}", c.n).into(),
                            "rerun",
                            &self.act,
                        )),
                );
            }
            b = b
                .child(tr)
                .child(div().flex().gap(px(10.)).child(chart).child(diff))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().font_weight(FontWeight::MEDIUM).child("Test cases"))
                        .child(quiet(&p, n.to_string()))
                        .child(div().flex_1())
                        .child(quiet(
                            &p,
                            format!(
                                "Pass {} · Partial {} · Fail {}",
                                counts[0], counts[1], counts[2]
                            ),
                        )),
                )
                .child(table);
        }
        let mut model = div().flex().items_center().gap(px(5.)).text_color(p.fg2);
        if let Some(mk) = brand_mark(EVAL_MODELS[0].1, 13., p.fg) {
            model = model.child(mk);
        }
        let model = model.child(EVAL_MODELS[0].0);
        frame(
            self.theme,
            self.layout,
            side(&p, persona("evalkit"), "evals", st, &self.act),
            page(
                &p,
                Some(("Evals", "Support reply prompt".to_string())),
                vec![
                    quiet(&p, "v12 current"),
                    model,
                    press(
                        btn(&p, "Run eval", true),
                        "run-eval".into(),
                        "run-eval",
                        &self.act,
                    ),
                ],
                b,
            ),
            None,
        )
    }
}
