//! Screens S11 in GPUI (Agent apps III): Command Board. A static snapshot: sessions in the
//! sidebar, the agent's updates with its question (or the pull request and its checks), the reply
//! composer, and the Workspace pane on Shell or Planner with the timeline at Live. `.state()` sets
//! Data / Loading / Empty and `.extra()` the own state (`pr`, `planner`) or `rail` / `hidden`.
//! Presses fire `new`, `send`, `answer`, `sleep` and `open-pr` through `on_act`. Rules from
//! `oxyn_core::parity::cmdboard`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view; the elements
//! below stay single-state snapshots.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, Stateful, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::parity::cmdboard::{
    CB_APP, CB_ASK, CB_PLAN, CB_REPOS, SStatus, ScreenState, TOOLS, cb_events, cb_msgs,
    cb_sessions, checks_passed, plan_state, sample_pr, shell_upto,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::{Live, hit};
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, dot, mono, pill, press, stat, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;

s11_el!(
    /// Command Board: sessions, the agent's updates and question or pull request, the reply
    /// composer, and the Workspace pane with its timeline.
    CommandBoardEl,
    cb_screen
);

fn board_pane(p: &ThPal, st: ScreenState, extra: &str, act: &Option<Act>) -> Div {
    let on = if extra == "planner" { 3 } else { 0 };
    let mut tabs = div()
        .flex()
        .gap(px(4.))
        .p(px(6.))
        .border_b_1()
        .border_color(p.line);
    for (i, t) in TOOLS.iter().enumerate() {
        let id = match i {
            0 => "view:",
            3 => "view:planner",
            _ => "tab-other",
        };
        tabs = tabs.child(hit(
            div()
                .h(px(28.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .text_color(if i == on { p.fg } else { p.fg2 })
                .when(i == on, |d| d.bg(p.sel))
                .child(t.label()),
            act,
            id,
        ));
    }
    let d = div()
        .w(px(440.))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(p.line)
        .child(tabs);
    let ev = cb_events(st);
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if ev.is_empty() {
        return d.child(faint(p, "The agent's workspace shows up here"));
    }
    let at = ev.len() - 1;
    let mut body = div()
        .flex_1()
        .min_h_0()
        .p(px(12.))
        .flex()
        .flex_col()
        .gap(px(4.));
    if on == 3 {
        for (i, (s, _)) in CB_PLAN.iter().copied().enumerate() {
            let t = match plan_state(i, at) {
                "done" => "ok",
                "run" => "run",
                _ => "",
            };
            body = body.child(
                div()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(dot(tone_of(p, t)))
                    .child(clip(s)),
            );
        }
    } else {
        body = body.bg(p.inset);
        for (cmd, l) in shell_upto(&ev, at) {
            body = body.child(
                mono(if cmd { format!("$ {l}") } else { l.to_string() }).text_color(if cmd {
                    p.fg
                } else {
                    p.fg2
                }),
            );
        }
    }
    d.child(body).child(
        div()
            .p(px(12.))
            .border_t_1()
            .border_color(p.line)
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .child(clip(format!("{} · {}", ev[at].at, ev[at].label)).flex_1())
                    .child(pill(p, "Live", "bad")),
            )
            .child(div().h(px(4.)).rounded(px(2.)).bg(p.fg2)),
    )
}

pub(super) fn cb_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let is_pr = extra == "pr";
    let rows: Vec<Stateful<Div>> = cb_sessions(st)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let status = if s.id == 1 && is_pr {
                SStatus::PrOpen
            } else {
                s.status
            };
            let mut tail = div().flex().items_center().gap(px(6.));
            if !s.from.is_empty() {
                tail = tail.child(brand(p, s.from));
            }
            s11_row(
                p,
                status.tone(),
                s.title.clone(),
                lv.on(i, s.id == 1),
                tail.child(div().text_color(p.mute).child(s.ago.clone())),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        CB_APP,
        "New session",
        "Sessions",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![chip(p, Some(brand(p, "github")), CB_REPOS[0], false)];
    let comp = composer(
        p,
        chips,
        "Reply, or add something for the agent to do",
        false,
        act,
        lv,
    );
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "What should the agent take on?")
    } else {
        let mut c = div().flex().flex_col().gap(px(12.));
        for (agent, t) in cb_msgs(st) {
            c = c.child(if agent { div().child(t) } else { you(p, t) });
        }
        if is_pr {
            let pr = sample_pr(true);
            let (ok, n) = checks_passed(&pr);
            let mut card = div()
                .p(px(12.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(brand(p, "github"))
                        .child(
                            clip(format!("#{} {}", pr.number, pr.title))
                                .flex_1()
                                .font_weight(FontWeight::SEMIBOLD),
                        )
                        .child(pill(
                            p,
                            if ok == n {
                                "All checks passed"
                            } else {
                                "Checks running"
                            },
                            if ok == n { "ok" } else { "run" },
                        )),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(10.))
                        .child(mono(pr.branch))
                        .child(stat(p, pr.stat.0, pr.stat.1)),
                );
            for (name, s) in &pr.checks {
                card = card.child(
                    div()
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(dot(tone_of(p, s)))
                        .child(*name),
                );
            }
            c = c.child(card.child(press(p, "Open PR", true, act, "open-pr")));
        } else {
            c = c.child(
                div()
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(tone_of(p, "wait"))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(CB_ASK.0)
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(press(p, CB_ASK.1[0], false, act, "answer"))
                            .child(press(p, CB_ASK.1[1], false, act, "answer")),
                    ),
            );
        }
        c
    };
    let pane = board_pane(p, st, extra, act);
    let samp = st == ScreenState::Sample;
    let title = if samp {
        "CSV export on the reports page"
    } else {
        "New session"
    };
    let pl = if !samp {
        None
    } else if is_pr {
        Some(("PR open", "ok"))
    } else {
        Some(("Needs you", "wait"))
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
