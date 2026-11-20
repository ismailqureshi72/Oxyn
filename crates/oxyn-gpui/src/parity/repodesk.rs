//! Screens S16 in GPUI (Agent apps IV): Repo Desk. Home, Inbox and Projects in the sidebar with
//! the sessions; the session's chat, the Inbox (issues and pull requests that need you) or the
//! Project (goal, due date, milestones); and the canvases pane (Plan, Pull request, Terminal,
//! Browser) on Plan. `.state()` sets Data / Loading / Empty and `.extra()` the own state
//! (`inbox`, `project`) or `rail` / `hidden`. Presses fire `row-N`, `view:`, `view:inbox`,
//! `view:project`, `start-N`, `ready` and `send` through `on_act`. Rules from
//! `oxyn_core::parity::repodesk`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::parity::repodesk::{
    RD_APP, RD_CHECKS, RD_GOAL, RD_MILESTONES, RD_PLAN, RD_PR, RD_PROMPT, RD_REPLY, ScreenState,
    goal_progress, rd_inbox, rd_sessions,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::{Live, hit};
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, dot, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;
use super::threadbar::{pane_col, pane_head, tabs4};

s11_el!(
    /// Repo Desk: the nav and sessions, the session chat, Inbox or Project, and the canvases pane.
    RepoDeskEl,
    rd_screen
);

fn check(p: &ThPal, done: bool, t: &'static str) -> Div {
    div()
        .h(px(30.))
        .flex()
        .items_center()
        .gap(px(9.))
        .child(
            div()
                .w(px(14.))
                .h(px(14.))
                .rounded(px(3.))
                .border_1()
                .border_color(p.fg2)
                .when(done, |d| d.bg(p.fg)),
        )
        .child(clip(t).text_color(if done { p.mute } else { p.fg }))
}

fn canvases(p: &ThPal, st: ScreenState, act: &Option<Act>) -> Div {
    let d = pane_col(p)
        .child(pane_head(p, "Canvases", "shared with the agent"))
        .child(tabs4(
            p,
            &[
                ("Plan", "canvas-plan", true),
                ("Pull request", "canvas-pr", false),
                ("Terminal", "canvas-term", false),
                ("Browser", "canvas-web", false),
            ],
            act,
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "The agent's canvases show up here"));
    }
    let mut plan = div().p(px(12.)).flex().flex_col();
    for (t, done) in RD_PLAN {
        plan = plan.child(check(p, done, t));
    }
    let (num, title, files, add, del) = RD_PR;
    let mut checks = div().flex().flex_col().gap(px(2.));
    for (n, t) in RD_CHECKS {
        checks = checks.child(
            div()
                .h(px(26.))
                .flex()
                .items_center()
                .gap(px(8.))
                .child(dot(tone_of(p, t)))
                .child(n),
        );
    }
    d.child(plan).child(
        div()
            .m(px(12.))
            .p(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(p.line)
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .child(brand(p, "github"))
                    .child(div().text_color(p.mute).child(format!("#{num}")))
                    .child(clip(title).font_weight(FontWeight::SEMIBOLD)),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(p.fg2)
                    .child(format!("Draft · {files} files · +{add} −{del}")),
            )
            .child(checks)
            .child(
                div()
                    .flex()
                    .justify_end()
                    .child(press(p, "Mark ready", true, act, "ready")),
            ),
    )
}

pub(super) fn rd_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let sessions = rd_sessions(st);
    let open = lv.sel.unwrap_or(0);
    let view = if extra == "rail" || extra == "hidden" {
        ""
    } else {
        extra
    };
    let n = rd_inbox(st).len();
    let mut rows: Vec<Stateful<Div>> = Vec::new();
    for (l, id, on) in [
        (
            "Home",
            "view:",
            view.is_empty() && st != ScreenState::Sample,
        ),
        ("Inbox", "view:inbox", view == "inbox"),
        ("Projects", "view:project", view == "project"),
    ] {
        rows.push(
            hit(div(), act, id)
                .h(px(30.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .text_size(px(12.5))
                .text_color(if on { p.fg } else { p.fg2 })
                .when(on, |d| d.bg(p.sel))
                .child(div().flex_1().child(l))
                .when(l == "Inbox" && n > 0, |d| {
                    d.child(
                        div()
                            .text_size(px(11.))
                            .text_color(p.mute)
                            .child(n.to_string()),
                    )
                }),
        );
    }
    rows.push(
        div()
            .id("rd-sessions")
            .pt(px(10.))
            .pb(px(4.))
            .px(px(8.))
            .text_size(px(11.5))
            .text_color(p.mute)
            .child("Sessions"),
    );
    for (i, c) in sessions.iter().enumerate() {
        rows.push(s11_row(
            p,
            if c.running { "run" } else { "" },
            c.title.clone(),
            view.is_empty() && i == open,
            div().text_size(px(11.)).text_color(p.mute).child(c.repo),
            i,
            act,
        ));
    }
    let side = s11_side(
        p,
        RD_APP,
        "New session",
        "Go to",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![
        chip(p, Some(brand(p, "github")), "acorn/insights", true),
        chip(p, Some(brand(p, "openai")), "GPT-6", false),
    ];
    let comp = composer(
        p,
        chips,
        "Ask the agent to work on something · # for issues",
        false,
        act,
        lv,
    );
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "What should we work on?")
    } else if view == "inbox" {
        let mut l = div().flex().flex_col().gap(px(6.));
        for it in rd_inbox(st) {
            let start: &'static str = match it.id {
                1 => "start-1",
                2 => "start-2",
                3 => "start-3",
                4 => "start-4",
                _ => "start-5",
            };
            l = l.child(
                div()
                    .p(px(10.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .text_size(px(11.5))
                                    .text_color(p.mute)
                                    .child(brand(p, "github"))
                                    .child(mono(it.refs))
                                    .child(div().flex_1())
                                    .child(it.at),
                            )
                            .child(clip(it.title).font_weight(FontWeight::SEMIBOLD))
                            .child(div().text_size(px(12.)).text_color(p.fg2).child(format!(
                                "{} · {}",
                                it.kind.label(),
                                it.why
                            ))),
                    )
                    .child(press(p, "Start session", true, act, start)),
            );
        }
        l
    } else if view == "project" {
        let (dn, tn, pc) = goal_progress(&RD_MILESTONES);
        let mut ms = div().flex().flex_col();
        for (t, done) in RD_MILESTONES {
            ms = ms.child(check(p, done, t));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(div().text_color(p.mute).child("Goal"))
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(RD_GOAL.1),
            )
            .child(div().text_color(p.fg2).child(format!(
                "Due {} · {dn} of {tn} milestones · {pc}%",
                RD_GOAL.2
            )))
            .child(
                div().h(px(6.)).rounded(px(3.)).bg(p.inset).child(
                    div()
                        .h_full()
                        .w(px(6. * pc as f32))
                        .rounded(px(3.))
                        .bg(tone_of(p, "ok")),
                ),
            )
            .child(ms)
    } else {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, RD_PROMPT))
            .child(div().child(RD_REPLY))
    };
    let title: SharedString = match view {
        "inbox" => "Inbox".into(),
        "project" => RD_GOAL.0.into(),
        _ => sessions
            .get(open)
            .map(|s| s.title.clone().into())
            .unwrap_or_else(|| "Home".into()),
    };
    let pl = (view.is_empty() && !sessions.is_empty()).then(|| {
        if sessions.get(open).is_some_and(|s| s.running) {
            ("Working", "run")
        } else {
            ("Idle", "")
        }
    });
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, comp),
        canvases(p, st, act),
    )
}
