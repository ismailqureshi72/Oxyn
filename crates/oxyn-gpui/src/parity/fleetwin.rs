//! Screens S11 in GPUI (Agent apps III): Fleet Window, plus the frame the S11 snapshots share.
//! A static snapshot of one state, like the other screen sets: agents in the sidebar (status,
//! model mark, lines changed), the open agent's steps or the Compare view of a parallel run, the
//! composer with mode and models, and the Changes pane. `.state()` sets Data / Loading / Empty and
//! `.extra()` the own state (`compare`, `plan`) or `rail` / `hidden`. Presses fire `new`, `send`,
//! `apply`, `accept-all` and `revert-all` through `on_act`. Rules from
//! `oxyn_core::parity::fleetwin`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view; the elements
//! below stay single-state snapshots.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::parity::appui::{ids, row_id};
use oxyn_core::parity::fleetwin::{
    COMPARE_PROMPT, FW_APP, FW_MODELS, FW_PROMPT, FW_STEPS, PERSON, ScreenState, agent_diff,
    best_attempt, fw_agents, mmss, sample_attempts, sample_plan,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::{Live, hit};
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, diff_pane, dot, pill, press, stat, tone_of};
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{gh, skel};

/// Declares an S11 snapshot element: state, extra, layout, theme and on_act, rendered by `$f`.
macro_rules! s11_el {
    ($(#[$m:meta])* $name:ident, $f:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The own state, `rail` or `hidden` (see the module docs).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat: the main area as a card or edge to edge.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`new`, `send`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
        impl RenderOnce for $name {
            fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
                let p = th_pal(self.theme);
                $f(&p, self.layout, self.state, self.extra, &self.act, &Live::default())
            }
        }
    };
}
pub(super) use s11_el;

s11_el!(
    /// Fleet Window: agents on their own worktrees, the open agent's steps or the Compare view, the
    /// composer with mode and models, and the Changes pane.
    FleetWindowEl,
    fw_screen
);

/// One sidebar row: a status dot, a label, an optional brand mark and a trailing part. Pressing
/// it fires `row-<i>`.
pub(super) fn s11_row(
    p: &ThPal,
    tone: &str,
    l: impl Into<SharedString>,
    on: bool,
    tail: Div,
    i: usize,
    act: &Option<Act>,
) -> Stateful<Div> {
    let d = div()
        .h(px(32.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(8.))
        .text_size(px(12.5))
        .text_color(p.fg2)
        .when(on, |d| d.bg(p.sel).text_color(p.fg))
        .child(dot(tone_of(p, tone)))
        .child(clip(l).flex_1())
        .child(tail);
    hit(d, act, row_id(i))
}

/// The S11 sidebar: the app name, New, a list and the account row.
#[allow(clippy::too_many_arguments)]
pub(super) fn s11_side(
    p: &ThPal,
    app: &'static str,
    new: &'static str,
    head: &'static str,
    rows: Vec<Stateful<Div>>,
    st: ScreenState,
    rail: bool,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let rail = rail || lv.rail;
    let f = fire(act, "new");
    let s = div()
        .w(px(if rail { 56. } else { 272. }))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .p(px(8.))
        .bg(p.inset)
        .child(
            div()
                .h(px(34.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(hit(
                    div()
                        .w(px(16.))
                        .h(px(16.))
                        .rounded(px(5.))
                        .border_2()
                        .border_color(p.fg)
                        .flex_none(),
                    act,
                    "rail",
                ))
                .when(!rail, |d| d.child(app)),
        )
        .child(
            div()
                .id("new")
                .on_click(move |_, w, cx| f(w, cx))
                .h(px(34.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .text_color(p.fg2)
                .child(if rail { "+" } else { new }),
        );
    if rail {
        return s.child(div().flex_1()).child(hit(
            div()
                .flex()
                .justify_center()
                .child(super::setscr::av(p, "RV", 28.)),
            act,
            "acct",
        ));
    }
    let mut body = div()
        .flex_1()
        .min_h_0()
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(gh(p, head));
    if st == ScreenState::Loading {
        body = body.child(skel(p, 7));
    } else if rows.is_empty() {
        body = body.child(
            div()
                .py(px(14.))
                .flex()
                .justify_center()
                .text_color(p.mute)
                .child("Nothing yet"),
        );
    }
    for r in rows {
        body = body.child(r);
    }
    s.child(body).child(hit(
        div()
            .mt(px(6.))
            .pt(px(6.))
            .border_t_1()
            .border_color(p.line)
            .flex()
            .items_center()
            .gap(px(9.))
            .child(super::setscr::av(p, "RV", 28.))
            .child(clip(PERSON).flex_1())
            .child(div().text_color(p.mute).child("⌄")),
        act,
        "acct",
    ))
}

/// The S11 frame: sidebar, top bar, body and pane, laid out Inset or Flat.
#[allow(clippy::too_many_arguments)]
pub(super) fn s11_frame(
    p: &ThPal,
    lay: ShellLayout,
    extra: &'static str,
    side: Div,
    title: &'static str,
    pl: Option<(&'static str, &'static str)>,
    body: Div,
    pane: Div,
) -> Div {
    let mut bar = div()
        .h(px(48.))
        .flex_none()
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(p.line)
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title));
    if let Some((l, t)) = pl {
        bar = bar.child(pill(p, l, t));
    }
    let main = div()
        .flex_1()
        .min_w_0()
        .h_full()
        .bg(p.card)
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(bar.child(div().flex_1()))
        .child(div().flex_1().min_h_0().flex().child(body).child(pane));
    let mut main = match lay {
        ShellLayout::Inset => main
            .m(px(8.))
            .ml(px(0.))
            .rounded(px(12.))
            .border_1()
            .border_color(p.line),
        ShellLayout::Flat => main.border_l_1().border_color(p.line),
    };
    let mut root = div()
        .size_full()
        .flex()
        .bg(p.inset)
        .text_color(p.fg)
        .text_size(px(13.));
    if extra != "hidden" {
        root = root.child(side);
    } else {
        main = main.ml(px(8.));
    }
    root.child(main)
}

/// The conversation column with a composer at the foot.
pub(super) fn s11_col(p: &ThPal, inner: Div, comp: Div) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .h_full()
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p(px(20.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(inner),
        )
        .child(div().p(px(12.)).child(comp))
        .text_color(p.fg)
}

/// Your prompt as a bubble.
pub(super) fn you(p: &ThPal, t: impl Into<SharedString>) -> Div {
    div().flex().justify_end().child(
        div()
            .max_w(px(520.))
            .px(px(14.))
            .py(px(10.))
            .rounded(px(14.))
            .bg(p.sel)
            .child(t.into()),
    )
}

/// A muted empty/loading message.
pub(super) fn faint(p: &ThPal, t: &'static str) -> Div {
    div()
        .py(px(40.))
        .flex()
        .justify_center()
        .text_color(p.mute)
        .child(t)
}

pub(super) fn fw_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let agents = fw_agents(st);
    let rows: Vec<Stateful<Div>> = agents
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let tail = div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(brand(p, FW_MODELS[a.model].1))
                .child(if a.stat.0 + a.stat.1 > 0 {
                    stat(p, a.stat.0, a.stat.1)
                } else {
                    div().text_color(p.mute).child(a.ago.clone())
                });
            let l = if a.group > 0 {
                format!("{} · {}", a.title, FW_MODELS[a.model].0)
            } else {
                a.title.clone()
            };
            let on = lv.on(i, a.id == 1 && extra != "compare");
            s11_row(p, a.state.tone(), l, on, tail, i, act)
        })
        .collect();
    let side = s11_side(
        p,
        FW_APP,
        "New agent",
        "Agents",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let mut chips = vec![chip(
        p,
        None,
        if extra == "plan" { "Plan" } else { "Agent" },
        false,
    )];
    match &lv.data {
        // Live: every model is a toggle; two or more run the prompt in parallel.
        Some(d) => {
            for (m, (name, mk)) in FW_MODELS.iter().copied().enumerate() {
                let on = d.models.contains(&m);
                chips.push(
                    div().child(hit(
                        div()
                            .flex_none()
                            .h(px(28.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .rounded(px(14.))
                            .border_1()
                            .border_color(if on { p.fg2 } else { p.line })
                            .text_size(px(12.5))
                            .text_color(if on { p.fg } else { p.mute })
                            .child(brand(p, mk))
                            .child(name),
                        act,
                        ids::at(&ids::MODEL, m),
                    )),
                );
            }
            if d.models.len() > 1 {
                chips.push(
                    div()
                        .text_color(p.mute)
                        .text_size(px(12.))
                        .child(format!("{} in parallel", d.models.len())),
                );
            }
        }
        None => chips.push(chip(p, Some(brand(p, "claude")), "Sonnet 5.5", false)),
    }
    let comp = composer(
        p,
        chips,
        "Plan, build, fix anything",
        extra == "plan",
        act,
        lv,
    );
    let samp = st == ScreenState::Sample;
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if !samp {
        faint(p, "What should the agents work on?")
    } else if extra == "compare" {
        let a = sample_attempts();
        let best = best_attempt(&a);
        let mut grid = div().flex().gap(px(10.));
        for (i, x) in a.iter().enumerate() {
            let ok = x.tests.0 == x.tests.1;
            grid = grid.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(brand(p, FW_MODELS[x.model].1))
                            .child(
                                clip(FW_MODELS[x.model].0)
                                    .flex_1()
                                    .font_weight(FontWeight::SEMIBOLD),
                            )
                            .when(Some(i) == best, |d| d.child(pill(p, "Suggested", ""))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.))
                            .child(
                                div()
                                    .text_color(tone_of(p, if ok { "ok" } else { "bad" }))
                                    .child(format!("{}/{} tests", x.tests.0, x.tests.1)),
                            )
                            .child(stat(p, x.stat.0, x.stat.1))
                            .child(div().text_color(p.mute).child(mmss(x.secs))),
                    )
                    .child(div().text_color(p.fg2).text_size(px(12.5)).child(x.summary))
                    .child(press(p, "Apply", true, act, "apply")),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, COMPARE_PROMPT))
            .child(grid)
    } else if extra == "plan" {
        let mut l = div()
            .p(px(12.))
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("Plan"));
        for (t, _) in sample_plan() {
            l = l.child(div().flex().gap(px(8.)).child("☐").child(clip(t)));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, FW_PROMPT))
            .child(l.child(press(p, "Build", true, act, "send")))
    } else {
        let mut log = div().flex().flex_col().gap(px(4.));
        for (i, (_, l, d)) in FW_STEPS.iter().copied().enumerate() {
            log = log.child(
                div()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(dot(tone_of(p, if i == 4 { "run" } else { "ok" })))
                    .child(clip(l).flex_1())
                    .child(div().text_color(p.mute).child(d)),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, FW_PROMPT))
            .child(log)
            .child(div().text_color(p.mute).child(match &lv.data {
                Some(d) => format!("Running tests · 4 of 5 steps · {}", d.clock(194)),
                None => "Running tests · 4 of 5 steps".to_string(),
            }))
    };
    let files = if samp {
        agent_diff(if extra == "compare" { 3 } else { 2 })
    } else {
        Vec::new()
    };
    let pane = diff_pane(p, st, &files, "Changes", act);
    let (title, pl) = match (samp, extra) {
        (false, _) => ("New agent", None),
        (_, "compare") => ("Compare · Paginate the invoices API", None),
        (_, "plan") => ("Plan", None),
        _ => ("Retry uploads on 503", Some(("Running", "run"))),
    };
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, comp),
        pane,
    )
}
