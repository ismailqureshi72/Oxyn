//! Screens S16 in GPUI (Agent apps IV): Threadbar. Threads by project in the sidebar (agent and
//! terminal threads), the open thread's tool calls and the edit's diff with Keep / Reject, the
//! composer with Profile, Agent and Model, and the Project pane on Files (git letters) or Git
//! (staged files and Commit). `.state()` sets Data / Loading / Empty and `.extra()` the own state
//! (`terminal`, `git`) or `rail` / `hidden`. Presses fire `row-N`, `view:git`, `view:`, `keep`,
//! `reject`, `commit` and `send` through `on_act`. Rules from `oxyn_core::parity::threadbar`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_core::parity::threadbar::{
    ScreenState, TB_APP, TB_CALLS, TB_DIFF, TB_DONE, TB_PROMPT, TB_TERM, TB_TREE, staged_sum,
    tb_changes, tb_threads,
};
use oxyn_tokens::Theme;

use super::Act;
use super::apps::{Live, hit};
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;

s11_el!(
    /// Threadbar: threads by project, tool calls and an edit to keep or reject, the composer with
    /// Profile, Agent and Model, and the Project pane (Files or Git).
    ThreadbarEl,
    tb_screen
);

/// A row of pressable tabs: (label, press id, on).
pub(super) fn tabs4(
    p: &ThPal,
    items: &[(&'static str, &'static str, bool)],
    act: &Option<Act>,
) -> Div {
    let mut d = div()
        .flex()
        .gap(px(4.))
        .p(px(6.))
        .border_b_1()
        .border_color(p.line);
    for (l, id, on) in items.iter().copied() {
        d = d.child(hit(
            div()
                .h(px(28.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .text_color(if on { p.fg } else { p.fg2 })
                .when(on, |d| d.bg(p.sel))
                .child(l),
            act,
            id,
        ));
    }
    d
}

/// A diff line: sign tint, the sign and the text in mono.
pub(super) fn diff_line(p: &ThPal, s: char, l: &'static str) -> Div {
    let tint = match s {
        '+' => Some(tone_of(p, "ok").opacity(0.14)),
        '-' => Some(tone_of(p, "bad").opacity(0.14)),
        _ => None,
    };
    let mut d = div()
        .flex()
        .gap(px(6.))
        .px(px(8.))
        .child(
            div()
                .w(px(12.))
                .text_color(p.mute)
                .child(mono(s.to_string())),
        )
        .child(mono(l).text_color(if s == ' ' { p.fg2 } else { p.fg }));
    if let Some(t) = tint {
        d = d.bg(t);
    }
    d
}

/// A pane header: a bold title and a muted part on the right.
pub(super) fn pane_head(p: &ThPal, t: &'static str, right: impl Into<SharedString>) -> Div {
    div()
        .h(px(44.))
        .flex_none()
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(p.line)
        .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(12.))
                .text_color(p.mute)
                .child(right.into()),
        )
}

/// The 440 px side pane column.
pub(super) fn pane_col(p: &ThPal) -> Div {
    div()
        .w(px(440.))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(p.line)
}

fn project(p: &ThPal, st: ScreenState, git: bool, act: &Option<Act>) -> Div {
    let d = pane_col(p)
        .child(pane_head(p, "Project", "webhook-retries"))
        .child(tabs4(
            p,
            &[("Files", "view:", !git), ("Git", "view:git", git)],
            act,
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Open a folder to see its files"));
    }
    if git {
        let ch = tb_changes(st);
        let (n, a, dl) = staged_sum(&ch);
        let mut list = div().p(px(12.)).flex().flex_col().gap(px(4.));
        for c in &ch {
            list = list.child(
                div()
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .w(px(14.))
                            .h(px(14.))
                            .rounded(px(3.))
                            .border_1()
                            .border_color(p.fg2)
                            .when(c.staged, |d| d.bg(p.fg)),
                    )
                    .child(clip(c.path))
                    .child(mono(format!("+{}", c.add)).text_color(tone_of(p, "ok")))
                    .child(mono(format!("−{}", c.del)).text_color(tone_of(p, "bad"))),
            );
        }
        return d.child(
            list.child(
                div()
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .child(format!("{n} staged · +{a} −{dl}")),
            )
            .child(
                div()
                    .p(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(p.line)
                    .child(mono("webhooks: retry failed deliveries with backoff")),
            )
            .child(press(p, "Commit", true, act, "commit")),
        );
    }
    let mut tree = div().p(px(6.)).flex().flex_col();
    for (name, depth, g) in TB_TREE {
        let gc = match g {
            'M' => tone_of(p, "wait"),
            'A' => tone_of(p, "ok"),
            _ => p.mute,
        };
        tree = tree.child(
            div()
                .h(px(28.))
                .pl(px(8. + depth as f32 * 16.))
                .pr(px(8.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .text_color(if g == ' ' { p.fg2 } else { p.fg })
                .child(div().flex_1().min_w_0().overflow_hidden().child(mono(name)))
                .when(g != ' ', |d| d.child(mono(g.to_string()).text_color(gc))),
        );
    }
    d.child(tree)
}

pub(super) fn tb_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let threads = tb_threads(st);
    let term_row = threads.iter().position(|t| t.terminal).unwrap_or(0);
    let open = if extra == "terminal" {
        term_row
    } else {
        lv.sel.unwrap_or(0)
    };
    let rows: Vec<Stateful<Div>> = threads
        .iter()
        .enumerate()
        .map(|(i, c)| {
            s11_row(
                p,
                if c.running { "run" } else { "" },
                c.title.clone(),
                i == open,
                div()
                    .text_size(px(11.))
                    .text_color(p.mute)
                    .child(if c.terminal { "Terminal" } else { c.project }),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        TB_APP,
        "New thread",
        "Threads",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let cur = threads.get(open);
    let terminal = cur.is_some_and(|t| t.terminal);
    let chips = vec![
        chip(p, None, "Write", false),
        chip(p, None, "Threadbar agent", false),
        chip(p, Some(brand(p, "claude")), "Sonnet 5.5", false),
    ];
    let comp = composer(
        p,
        chips,
        "Message the agent · @ to add files",
        false,
        act,
        lv,
    );
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "Start a thread")
    } else if terminal {
        let mut out = div()
            .p(px(12.))
            .rounded(px(10.))
            .bg(p.inset)
            .flex()
            .flex_col()
            .gap(px(2.));
        for (cmd, l) in TB_TERM {
            out = out.child(
                mono(if cmd {
                    format!("ledger-api $ {l}")
                } else {
                    l.to_string()
                })
                .text_color(if cmd { p.fg } else { p.fg2 }),
            );
        }
        out
    } else {
        let mut calls = div().flex().flex_col().gap(px(2.));
        for (_, v, o, x) in TB_CALLS {
            calls = calls.child(
                div()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(v))
                    .child(div().flex_1().min_w_0().overflow_hidden().child(mono(o)))
                    .child(div().text_size(px(12.)).text_color(p.mute).child(x)),
            );
        }
        let mut diff = div().py(px(6.)).bg(p.inset).flex().flex_col();
        for (s, l) in TB_DIFF {
            diff = diff.child(diff_line(p, s, l));
        }
        let edit = div()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(36.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .child(mono("src/webhooks/deliver.rs")),
                    )
                    .child(mono("+18").text_color(tone_of(p, "ok")))
                    .child(mono("−4").text_color(tone_of(p, "bad"))),
            )
            .child(diff)
            .child(
                div()
                    .p(px(8.))
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(press(p, "Reject", false, act, "reject"))
                    .child(press(p, "Keep", true, act, "keep")),
            );
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, TB_PROMPT))
            .child(calls)
            .child(edit)
            .child(div().child(TB_DONE))
    };
    let title: SharedString = cur
        .map(|t| t.title.clone().into())
        .unwrap_or_else(|| "New thread".into());
    let pl = cur.map(|t| {
        if t.running {
            ("Working", "run")
        } else if t.terminal {
            ("Terminal", "")
        } else {
            ("Write", "")
        }
    });
    let body = if terminal {
        s11_col(p, inner, div())
    } else {
        s11_col(p, inner, comp)
    };
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        body,
        project(p, st, extra == "git", act),
    )
}
