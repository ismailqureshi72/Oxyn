//! Screens S16 in GPUI (Agent apps IV, part b): Courier. Threads with their state in the sidebar;
//! the thread's tool lines, its edits card and the closing line; suggested replies and the
//! composer with Model and Effort; and the Pod pane on Ports (the default), Changes or Desktop.
//! `.state()` sets Data / Loading / Empty and `.extra()` the own state (`changes`, `desktop`) or
//! `rail` / `hidden`. Presses fire `row-N`, `view:`, `view:changes`, `view:desktop`, `revert`,
//! `suggest-N` and `send` through `on_act`. Rules from `oxyn_core::parity::courier`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::parity::courier::{
    CR_APP, CR_DONE, CR_POD, CR_PORTS, CR_PROMPT, CR_SUGGEST, CR_TOOLS, ScreenState, cr_files,
    cr_threads, cr_totals,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::{Live, hit};
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;
use super::threadbar::{diff_line, pane_col, pane_head, tabs4};

s11_el!(
    /// Courier: threads with their state, the edits card, suggested replies, the composer with
    /// Model and Effort, and the Pod pane (Ports, Changes or Desktop).
    CourierEl,
    cr_screen
);

const SUGGEST_IDS: [&str; 3] = ["suggest-0", "suggest-1", "suggest-2"];

fn pod(p: &ThPal, st: ScreenState, extra: &'static str, act: &Option<Act>) -> Div {
    let d = pane_col(p)
        .child(pane_head(p, CR_POD.0, CR_POD.1))
        .child(tabs4(
            p,
            &[
                ("Ports", "view:", extra != "changes" && extra != "desktop"),
                ("Changes", "view:changes", extra == "changes"),
                ("Desktop", "view:desktop", extra == "desktop"),
            ],
            act,
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Start a thread to open its pod"));
    }
    match extra {
        "changes" => {
            let files = cr_files(st);
            let (n, a, dl) = cr_totals(&files);
            let mut l = div().p(px(12.)).flex().flex_col().gap(px(2.)).child(
                div()
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .child(format!("{n} files · +{a} −{dl}")),
            );
            for f in &files {
                l = l.child(
                    div()
                        .h(px(36.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(clip(f.path))
                        .child(mono(format!("+{}", f.add)).text_color(tone_of(p, "ok")))
                        .child(mono(format!("−{}", f.del)).text_color(tone_of(p, "bad")))
                        .child(press(p, "Revert", false, act, "revert")),
                );
            }
            d.child(l)
        }
        "desktop" => d.child(
            div().p(px(12.)).child(
                div()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(p.line)
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .h(px(30.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .bg(p.inset)
                            .child(mono("localhost:3000/p/trail-runner").text_color(p.fg2)),
                    )
                    .child(
                        div()
                            .p(px(14.))
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Trail runner"),
                            )
                            .child(div().child("€129.00"))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(tone_of(p, "ok"))
                                    .child("Price from cache · 4 ms"),
                            ),
                    ),
            ),
        ),
        _ => {
            let mut l = div().p(px(12.)).flex().flex_col();
            for (port, what, open) in CR_PORTS {
                l = l.child(
                    div()
                        .h(px(36.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w(px(52.)).child(mono(format!(":{port}"))))
                        .child(clip(what).text_color(p.fg2))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(if open { tone_of(p, "ok") } else { p.mute })
                                .child(if open { "Open" } else { "Not forwarded" }),
                        ),
                );
            }
            d.child(l)
        }
    }
}

pub(super) fn cr_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let threads = cr_threads(st);
    let open = lv.sel.unwrap_or(0);
    let rows: Vec<Stateful<Div>> = threads
        .iter()
        .enumerate()
        .map(|(i, c)| {
            s11_row(
                p,
                c.status,
                c.title.clone(),
                i == open,
                div().text_size(px(11.)).text_color(p.mute).child(c.repo),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        CR_APP,
        "New thread",
        "Threads",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![
        chip(p, Some(brand(p, "claude")), "Opus 5", false),
        chip(p, None, "Medium", false),
    ];
    let comp = composer(
        p,
        chips,
        "Reply, or start something new · @ to mention files",
        false,
        act,
        lv,
    );
    let mut sugs = div().flex().gap(px(6.)).pb(px(8.));
    for (i, s) in CR_SUGGEST.iter().copied().enumerate() {
        sugs = sugs.child(hit(
            div()
                .h(px(28.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(14.))
                .border_1()
                .border_color(p.line)
                .text_size(px(12.))
                .text_color(p.fg2)
                .child(s),
            act,
            SUGGEST_IDS[i],
        ));
    }
    let cur = threads.get(open);
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "Start a thread")
    } else if open != 0 {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, cur.map(|c| c.title.clone()).unwrap_or_default()))
            .child(
                div()
                    .text_color(p.fg2)
                    .child("Done, and the pod is still up if you want to look around."),
            )
    } else {
        let mut tools = div().flex().flex_col().gap(px(2.));
        for (_, v, o) in CR_TOOLS {
            tools = tools.child(
                div()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(v))
                    .child(clip(o).text_color(p.fg2)),
            );
        }
        let files = cr_files(st);
        let (n, a, dl) = cr_totals(&files);
        let mut edits = div()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(38.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .border_b_1()
                    .border_color(p.line)
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("Edited {n} files")),
                    )
                    .child(mono(format!("+{a}")).text_color(tone_of(p, "ok")))
                    .child(mono(format!("−{dl}")).text_color(tone_of(p, "bad")))
                    .child(div().flex_1())
                    .child(press(p, "Review", false, act, "view:changes")),
            );
        for (i, f) in files.iter().enumerate() {
            edits = edits.child(
                div()
                    .h(px(30.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(clip(f.path))
                    .child(mono(format!("+{}", f.add)).text_color(tone_of(p, "ok"))),
            );
            if i == 1 {
                let mut diff = div().py(px(4.)).bg(p.inset).flex().flex_col();
                for (s, l) in f.diff.iter().copied() {
                    diff = diff.child(diff_line(p, s, l));
                }
                edits = edits.child(diff);
            }
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, CR_PROMPT))
            .child(tools)
            .child(edits)
            .child(div().child(CR_DONE))
    };
    let title: SharedString = cur
        .map(|c| c.title.clone().into())
        .unwrap_or_else(|| "New thread".into());
    let pl = cur.map(|c| match c.status {
        "run" => ("Running", "run"),
        "wait" => ("Needs you", "wait"),
        "ok" => ("Done", "ok"),
        _ => ("Idle", ""),
    });
    let foot = div()
        .flex()
        .flex_col()
        .when(cur.is_some(), |d| d.child(sugs))
        .child(comp);
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, foot),
        pod(p, st, extra, act),
    )
}
