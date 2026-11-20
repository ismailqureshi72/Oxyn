//! Screens S16 in GPUI (Agent apps IV, part b): Hangar. Conversations by workspace in the
//! sidebar; the open conversation's task list and artifacts; the composer with Mode and Model; and
//! the Surfaces pane on the Editor, the Plan artifact (with its comments and Proceed) or the
//! Browser the agent drives. `.state()` sets Data / Loading / Empty and `.extra()` the own state
//! (`artifact`, `browser`) or `rail` / `hidden`. Presses fire `row-N`, `view:`, `view:artifact`,
//! `view:browser`, `proceed` and `send` through `on_act`. Rules from `oxyn_core::parity::hangar`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::parity::hangar::{
    HG_APP, HG_ARTIFACTS, HG_BROWSE, HG_CART, HG_CODE, HG_FILE, HG_PLAN, HG_PROMPT, HG_TASKS,
    HG_URL, HangarData, ScreenState, cart_total, hg_convs, price, tasks_done,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::Live;
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;
use super::threadbar::{diff_line, pane_col, pane_head, tabs4};

s11_el!(
    /// Hangar: conversations by workspace, the task list and artifacts, the composer with Mode and
    /// Model, and the Surfaces pane (Editor, Plan or Browser).
    HangarEl,
    hg_screen
);

fn surfaces(p: &ThPal, st: ScreenState, extra: &'static str, act: &Option<Act>) -> Div {
    let d = pane_col(p)
        .child(pane_head(p, "Surfaces", HG_URL))
        .child(tabs4(
            p,
            &[
                ("Editor", "view:", extra != "artifact" && extra != "browser"),
                ("Plan", "view:artifact", extra == "artifact"),
                ("Browser", "view:browser", extra == "browser"),
            ],
            act,
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Open a workspace to see its surfaces"));
    }
    match extra {
        "artifact" => {
            let notes = HangarData::default().notes;
            let mut doc = div().p(px(12.)).flex().flex_col().gap(px(4.)).child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Implementation plan"),
            );
            for (i, l) in HG_PLAN.iter().copied().enumerate() {
                doc = doc.child(
                    div()
                        .py(px(5.))
                        .flex()
                        .gap(px(8.))
                        .child(
                            div()
                                .w(px(16.))
                                .text_color(p.mute)
                                .child(format!("{}", i + 1)),
                        )
                        .child(clip(l)),
                );
                for n in notes.iter().filter(|n| n.line == i) {
                    doc = doc.child(
                        div()
                            .ml(px(24.))
                            .p(px(8.))
                            .rounded(px(8.))
                            .bg(p.inset)
                            .border_l_2()
                            .border_color(tone_of(p, "wait"))
                            .text_size(px(12.))
                            .child(n.text.clone()),
                    );
                }
            }
            d.child(doc.child(div().pt(px(8.)).flex().justify_end().child(press(
                p,
                if notes.len() == 1 {
                    "Send 1 comment"
                } else {
                    "Proceed"
                },
                true,
                act,
                "proceed",
            ))))
        }
        "browser" => {
            let mut items = div().flex().flex_col();
            for (n, c) in HG_CART {
                items = items.child(
                    div()
                        .h(px(28.))
                        .flex()
                        .items_center()
                        .child(clip(n))
                        .child(div().text_color(p.fg2).child(price(c))),
                );
            }
            let phone = div()
                .w(px(220.))
                .rounded(px(18.))
                .border_1()
                .border_color(p.line)
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Your bag"))
                .child(items)
                .child(
                    div()
                        .pt(px(6.))
                        .border_t_1()
                        .border_color(p.line)
                        .flex()
                        .items_center()
                        .child(clip(format!("Total {}", price(cart_total(&HG_CART)))))
                        .child(
                            div()
                                .px(px(12.))
                                .py(px(4.))
                                .rounded(px(12.))
                                .bg(p.fg)
                                .text_color(p.card)
                                .child("Pay"),
                        ),
                );
            let mut log = div().p(px(12.)).flex().flex_col().gap(px(2.));
            for (_, v, o, at) in HG_BROWSE {
                log = log.child(
                    div()
                        .h(px(26.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(v))
                        .child(clip(o))
                        .child(mono(at).text_color(p.mute)),
                );
            }
            d.child(div().p(px(12.)).flex().justify_center().child(phone))
                .child(log)
        }
        _ => {
            let mut code = div().py(px(6.)).bg(p.inset).flex().flex_col();
            for (s, l) in HG_CODE {
                code = code.child(diff_line(p, s, l));
            }
            d.child(
                div()
                    .h(px(36.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .child(clip(HG_FILE))
                    .child(mono("+5").text_color(tone_of(p, "ok"))),
            )
            .child(code)
        }
    }
}

pub(super) fn hg_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let convs = hg_convs(st);
    let open = lv.sel.unwrap_or(0);
    let rows: Vec<Stateful<Div>> = convs
        .iter()
        .enumerate()
        .map(|(i, c)| {
            s11_row(
                p,
                c.status,
                c.title.clone(),
                i == open,
                div()
                    .text_size(px(11.))
                    .text_color(p.mute)
                    .child(c.workspace),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        HG_APP,
        "New conversation",
        "Conversations",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![
        chip(p, None, "Planning", false),
        chip(p, Some(brand(p, "gemini")), "Gemini 3 Pro", false),
    ];
    let comp = composer(
        p,
        chips,
        "Ask the agent to build, fix or check something",
        false,
        act,
        lv,
    );
    let cur = convs.get(open);
    let inner =
        if st == ScreenState::Loading {
            skel(p, 6)
        } else if st == ScreenState::Empty {
            faint(p, "What should the agent build?")
        } else if open != 0 {
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(you(p, cur.map(|c| c.title.clone()).unwrap_or_default()))
                .child(div().text_color(p.fg2).child(
                    "Done. The changes are in the Editor, and I checked them in the Browser.",
                ))
        } else {
            let (done, of) = tasks_done(&HG_TASKS);
            let mut tasks = div()
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .flex()
                        .child(
                            div()
                                .flex_1()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Task list"),
                        )
                        .child(div().text_color(p.mute).child(format!("{done} of {of}"))),
                );
            for (l, s) in HG_TASKS {
                tasks = tasks.child(
                    div()
                        .h(px(26.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .w(px(8.))
                                .h(px(8.))
                                .rounded(px(4.))
                                .bg(tone_of(p, s))
                                .when(s.is_empty(), |d| d.bg(p.line)),
                        )
                        .child(clip(l).text_color(if s == "run" { p.fg } else { p.fg2 })),
                );
            }
            let mut arts = div().flex().gap(px(8.));
            for (_, t, _, l) in HG_ARTIFACTS {
                arts = arts.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .p(px(10.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .flex_col()
                        .child(clip(t).font_weight(FontWeight::SEMIBOLD))
                        .child(clip(l).text_size(px(12.)).text_color(p.mute)),
                );
            }
            div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, HG_PROMPT))
            .child(tasks)
            .child(arts)
            .child(div().child(
                "I wrote a plan for the sticky footer. Read it, comment on any line, then Proceed.",
            ))
        };
    let title: SharedString = cur
        .map(|c| c.title.clone().into())
        .unwrap_or_else(|| "New conversation".into());
    let pl = cur.map(|c| match c.status {
        "run" => ("Working", "run"),
        "wait" => ("Needs review", "wait"),
        "ok" => ("Done", "ok"),
        _ => ("Idle", ""),
    });
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, comp),
        surfaces(p, st, extra, act),
    )
}
