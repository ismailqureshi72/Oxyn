//! Screens S16 in GPUI (Agent apps IV): Tabstack. Vertical tabs with the agent's mark, branch and
//! pull request; command blocks with output, exit status and time; the command the agent asks to
//! run (Run / Edit / Deny); the input with Terminal / Agent, Profile and Model; and the pane on
//! Agents (local and cloud runs) or Review. `.state()` sets Data / Loading / Empty and `.extra()`
//! the own state (`approval`, `review`) or `rail` / `hidden`. Presses fire `row-N`, `view:`,
//! `view:review`, `run`, `deny` and `send` through `on_act`. Rules from
//! `oxyn_core::parity::tabstack`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, px,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_core::parity::tabstack::{
    Block, ScreenState, TS_APP, TS_DIFF, TS_PENDING, TS_RUNS, TsState, took, ts_blocks, ts_tabs,
};
use oxyn_tokens::Theme;

use super::Act;
use super::apps::Live;
use super::cmdstudio::{brand, chip, composer, dot, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;
use super::threadbar::{diff_line, pane_col, pane_head, tabs4};

s11_el!(
    /// Tabstack: vertical tabs, command blocks with the agent's turns and an approval card, the
    /// Terminal / Agent input, and the pane (Agents or Review).
    TabstackEl,
    ts_screen
);

fn block(p: &ThPal, cmd: String, out: Vec<String>, code: i32, ms: u32) -> Div {
    let mut o = div().px(px(12.)).pb(px(8.)).flex().flex_col();
    for l in out {
        o = o.child(mono(l).text_color(p.fg2));
    }
    div()
        .rounded(px(10.))
        .border_1()
        .border_color(if code == 0 { p.line } else { tone_of(p, "bad") })
        .bg(p.card)
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(
            div()
                .px(px(12.))
                .pt(px(8.))
                .pb(px(4.))
                .flex()
                .gap(px(8.))
                .child(mono("❯").text_color(tone_of(p, "run")))
                .child(mono(cmd).font_weight(FontWeight::SEMIBOLD)),
        )
        .child(o)
        .child(
            div()
                .h(px(30.))
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(10.))
                .border_t_1()
                .border_color(p.line)
                .bg(p.inset)
                .child(dot(tone_of(p, if code == 0 { "ok" } else { "bad" })))
                .child(mono(format!("exit {code}")))
                .child(div().text_size(px(12.)).text_color(p.mute).child(took(ms))),
        )
}

fn side_pane(p: &ThPal, st: ScreenState, review: bool, act: &Option<Act>) -> Div {
    let d = pane_col(p)
        .child(pane_head(p, "Agents", "2 local · 2 cloud"))
        .child(tabs4(
            p,
            &[
                ("Agents", "view:", !review),
                ("Review", "view:review", review),
            ],
            act,
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Agent runs show up here"));
    }
    if review {
        let mut diff = div().py(px(6.)).bg(p.inset).flex().flex_col();
        for (s, l) in TS_DIFF {
            diff = diff.child(diff_line(p, s, l));
        }
        return d.child(
            div()
                .m(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(div().p(px(10.)).child(mono("tests/upload.rs")))
                .child(diff)
                .child(
                    div()
                        .p(px(8.))
                        .flex()
                        .justify_end()
                        .gap(px(8.))
                        .child(press(p, "Request changes", false, act, "changes"))
                        .child(press(p, "Approve", true, act, "approve")),
                ),
        );
    }
    let mut l = div().p(px(8.)).flex().flex_col().gap(px(2.));
    for (t, wh, s, at) in TS_RUNS {
        l = l.child(
            div()
                .h(px(46.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(8.))
                .child(dot(tone_of(p, s.tone())))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(div().child(t))
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(p.mute)
                                .child(format!("{wh} · started {at}")),
                        ),
                ),
        );
    }
    d.child(l)
}

pub(super) fn ts_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let tabs = ts_tabs(st);
    let open = lv.sel.unwrap_or(0);
    let rows: Vec<Stateful<Div>> = tabs
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let tone = if i == 0 && extra == "approval" {
                "wait"
            } else {
                c.state.tone()
            };
            let tail = if c.agent.is_empty() {
                div()
                    .text_size(px(11.))
                    .text_color(p.mute)
                    .child(if c.pr > 0 {
                        format!("#{}", c.pr)
                    } else {
                        String::new()
                    })
            } else {
                div().flex().gap(px(4.)).child(brand(p, c.agent))
            };
            s11_row(p, tone, c.title.clone(), i == open, tail, i, act)
        })
        .collect();
    let side = s11_side(
        p,
        TS_APP,
        "New tab",
        "Tabs",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let cur = tabs.get(open);
    let chips = vec![
        chip(p, None, "Agent", true),
        chip(p, None, "Default", false),
        chip(p, Some(brand(p, "claude")), "Sonnet 5.5", false),
    ];
    let comp = composer(
        p,
        chips,
        "Ask the agent · it asks before it writes",
        false,
        act,
        lv,
    );
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "What should the agent do?")
    } else if open != 0 {
        let wt = cur.map(|t| t.worktree).unwrap_or("");
        div().text_color(p.mute).child(format!(
            "This session's blocks live in its own worktree, {wt}."
        ))
    } else {
        let mut col = div().flex().flex_col().gap(px(10.));
        for b in ts_blocks() {
            col = col.child(match b {
                Block::Cmd(c, o, code, ms) => block(p, c, o, code, ms),
                Block::Ask(t) => you(p, t),
                Block::Say(t) => div().text_color(p.fg).child(t),
            });
        }
        if extra == "approval" {
            col = col.child(
                div()
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(tone_of(p, "wait"))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("The agent wants to run"),
                    )
                    .child(
                        div()
                            .p(px(8.))
                            .rounded(px(8.))
                            .bg(p.inset)
                            .child(mono(TS_PENDING)),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(press(p, "Deny", false, act, "deny"))
                            .child(press(p, "Run", true, act, "run")),
                    ),
            );
        } else if extra == "review" {
            col = col
                .child(block(
                    p,
                    TS_PENDING.to_string(),
                    vec![
                        "running 9 tests".to_string(),
                        "test result: ok. 9 passed; 0 failed".to_string(),
                    ],
                    0,
                    5_870,
                ))
                .child(div().child("Passes now, 50 runs in a row. The change is in Review."));
        }
        col
    };
    let title: SharedString = cur
        .map(|t| t.title.clone().into())
        .unwrap_or_else(|| "New tab".into());
    let pl = cur.map(|t| match (open, extra, t.state) {
        (0, "approval", _) => ("Needs you", "wait"),
        (0, "review", _) => ("Done", "ok"),
        (_, _, TsState::Working) => ("Working", "run"),
        (_, _, TsState::Waiting) => ("Needs you", "wait"),
        (_, _, TsState::Done) => ("Done", "ok"),
        _ => ("Shell", ""),
    });
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, comp),
        side_pane(p, st, extra == "review", act),
    )
}
