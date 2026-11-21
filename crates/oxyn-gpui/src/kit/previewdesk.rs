//! Screens S16 in GPUI (Agent apps IV): Preview Desk. Sessions with their machine; the session's
//! steps; the composer with Machine, folder and Model; and the preview pane on the live site (with
//! Design mode and a picked element's comment) or the diff with a line comment. `.state()` sets
//! Data / Loading / Empty and `.extra()` the own state (`design`, `diff`) or `rail` / `hidden`.
//! Presses fire `row-N`, `view:`, `view:design`, `view:diff`, `approve` and `send` through
//! `on_act`. Rules from `oxyn_core::kit::previewdesk`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px, rgb,
};
use oxyn_core::kit::previewdesk::{
    PD_APP, PD_DIFF, PD_DONE, PD_MACHINES, PD_PLANS, PD_PROMPT, PD_STEPS, ScreenState, pd_notes,
    pd_sessions, price,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::Live;
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;
use super::threadbar::{pane_col, pane_head, tabs4};

s11_el!(
    /// Preview Desk: sessions on machines, the session's steps, and the preview pane (live site
    /// with Design mode, or the diff with a line comment).
    PreviewDeskEl,
    pd_screen
);

fn note(p: &ThPal, on: String, text: String) -> Div {
    div()
        .p(px(10.))
        .rounded(px(8.))
        .border_l_2()
        .border_color(tone_of(p, "wait"))
        .bg(p.inset)
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(div().text_size(px(11.5)).text_color(p.mute).child(on))
        .child(div().text_size(px(12.5)).child(text))
}

fn previews(p: &ThPal, st: ScreenState, own: &str, act: &Option<Act>) -> Div {
    let diff = own == "diff";
    let d = pane_col(p)
        .child(pane_head(p, "Preview", "localhost:3000"))
        .child(tabs4(
            p,
            &[
                (
                    "Site",
                    if own == "design" {
                        "view:design"
                    } else {
                        "view:"
                    },
                    !diff,
                ),
                ("Diff", "view:diff", diff),
            ],
            act,
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Previews show up here"));
    }
    if diff {
        let mut col = div()
            .m(px(10.))
            .py(px(6.))
            .rounded(px(10.))
            .bg(p.inset)
            .flex()
            .flex_col();
        for (s, n, l) in PD_DIFF {
            let tint = match s {
                '+' => Some(tone_of(p, "ok").opacity(0.14)),
                '-' => Some(tone_of(p, "bad").opacity(0.14)),
                _ => None,
            };
            let mut row = div()
                .flex()
                .gap(px(6.))
                .px(px(8.))
                .child(
                    div()
                        .w(px(22.))
                        .text_color(p.mute)
                        .child(mono(n.to_string())),
                )
                .child(
                    div()
                        .w(px(10.))
                        .text_color(p.mute)
                        .child(mono(s.to_string())),
                )
                .child(div().flex_1().min_w_0().child(mono(l)));
            if let Some(t) = tint {
                row = row.bg(t);
            }
            col = col.child(row);
            if s == '+' && n == 9 {
                for x in pd_notes("diff") {
                    col = col.child(div().mx(px(8.)).my(px(4.)).child(note(p, x.on, x.text)));
                }
            }
        }
        return d
            .child(col)
            .child(div().px(px(10.)).flex().justify_end().child(press(
                p,
                "Approve changes",
                true,
                act,
                "approve",
            )));
    }
    let design = own == "design";
    let ink = rgb(0x18181B);
    let mut plans = div().flex().gap(px(8.));
    for (i, (n, _, _)) in PD_PLANS.iter().copied().enumerate() {
        let feat = i == 1;
        plans = plans.child(
            div()
                .flex_1()
                .p(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(if feat { ink } else { rgb(0xE4E4E7) })
                .when(feat && design, |d| {
                    d.border_2().border_color(tone_of(p, "run"))
                })
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(n))
                .child(
                    div()
                        .text_size(px(17.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("${}", price(i, false))),
                )
                .child(
                    div()
                        .py(px(4.))
                        .rounded(px(6.))
                        .bg(ink)
                        .text_color(gpui::white())
                        .text_size(px(11.5))
                        .flex()
                        .justify_center()
                        .child("Start trial"),
                ),
        );
    }
    let site = div()
        .m(px(10.))
        .p(px(16.))
        .rounded(px(10.))
        .border_1()
        .border_color(p.line)
        .bg(gpui::white())
        .text_color(ink)
        .flex()
        .flex_col()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .text_size(px(17.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Plans that grow with you"),
        )
        .child(plans);
    let mut d = d
        .child(
            div()
                .px(px(10.))
                .pt(px(8.))
                .flex()
                .justify_end()
                .child(press(
                    p,
                    if design {
                        "Design mode on"
                    } else {
                        "Design mode"
                    },
                    design,
                    act,
                    if design { "view:" } else { "view:design" },
                )),
        )
        .child(site);
    if design {
        for x in pd_notes("design") {
            d = d.child(div().mx(px(10.)).child(note(p, x.on, x.text)));
        }
    }
    d
}

pub(super) fn pd_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let done = extra == "design" || extra == "diff";
    let sessions = pd_sessions(st);
    let open = lv.sel.unwrap_or(0);
    let rows: Vec<Stateful<Div>> = sessions
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let run = c.running && !(done && i == 0);
            s11_row(
                p,
                if run { "run" } else { "" },
                c.title.clone(),
                i == open,
                div()
                    .text_size(px(11.))
                    .text_color(p.mute)
                    .child(PD_MACHINES[c.machine].0),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        PD_APP,
        "New session",
        "Sessions",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![
        chip(p, None, "build-box-2", true),
        chip(p, None, "~/src/storefront", false),
        chip(p, Some(brand(p, "claude")), "Sonnet 5.5", false),
    ];
    let comp = composer(
        p,
        chips,
        "Describe a change, or reply to the agent",
        false,
        act,
        lv,
    );
    let cur = sessions.get(open);
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "What should we build?")
    } else if open != 0 {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, cur.map(|c| c.title.clone()).unwrap_or_default()))
            .child(div().text_color(p.fg2).child(
                "Here's a first pass. Open the preview, or point at what to change in Design mode.",
            ))
    } else {
        let mut steps = div().flex().flex_col().gap(px(2.));
        for (_, s) in PD_STEPS.iter().copied().take(if done { 5 } else { 3 }) {
            steps = steps.child(div().h(px(26.)).flex().items_center().child(clip(s)));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, PD_PROMPT))
            .child(steps)
            .child(if done {
                div().child(PD_DONE)
            } else {
                div().text_color(p.mute).child("Starting the dev server")
            })
    };
    let title: SharedString = cur
        .map(|t| t.title.clone().into())
        .unwrap_or_else(|| "New session".into());
    let pl = cur.map(|t| {
        if t.running && !(done && open == 0) {
            ("Working", "run")
        } else {
            ("Ready to review", "ok")
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
        previews(p, st, extra, act),
    )
}
