//! Screens S16 in GPUI (Agent apps IV, part b): Halo Chat. Helpers and chats in the sidebar; the
//! chat's tool cards (a web search, code it ran, a table) and the answer; the composer with
//! Tools, Mode and Model; and the Sources pane. Own states: `research` (the plan to start) and
//! `mini` (the mini chat over the desktop). `.state()` sets Data / Loading / Empty and `.extra()`
//! the own state or `rail` / `hidden`. Presses fire `row-N`, `view:research`, `start`, `full`
//! and `send` through `on_act`. Rules from `oxyn_core::kit::halochat`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::kit::halochat::{
    HC_ANSWER, HC_APP, HC_BIKES, HC_PLAN, HC_PROMPT, HC_QUERY, HC_SHORTCUT, HC_SOURCES,
    ScreenState, dollars, hc_chats, plan_minutes, usd_per_km,
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
use super::threadbar::{pane_col, pane_head};

s11_el!(
    /// Halo Chat: helpers and chats, tool cards and the answer, the composer with Tools, Mode and
    /// Model, and the Sources pane; the research plan and the mini chat as own states.
    HaloChatEl,
    hc_screen
);

/// A tool card: a bold title, a muted part and a body.
fn card(p: &ThPal, t: &'static str, meta: impl Into<SharedString>, body: Div) -> Div {
    div()
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
                .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.mute)
                        .child(meta.into()),
                ),
        )
        .child(body.p(px(10.)))
}

fn sources(p: &ThPal, st: ScreenState) -> Div {
    let d = pane_col(p).child(pane_head(p, "Sources", format!("{}", HC_SOURCES.len())));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Sources from your chats show here"));
    }
    let mut l = div().p(px(8.)).flex().flex_col().gap(px(4.));
    for (i, (t, site, at)) in HC_SOURCES.iter().copied().enumerate() {
        l = l.child(
            div()
                .p(px(8.))
                .flex()
                .gap(px(10.))
                .child(
                    div()
                        .w(px(16.))
                        .text_color(p.mute)
                        .child(format!("{}", i + 1)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(t).font_weight(FontWeight::MEDIUM))
                        .child(
                            clip(format!("{site} · {at}"))
                                .text_size(px(12.))
                                .text_color(p.mute),
                        ),
                ),
        );
    }
    d.child(l)
}

pub(super) fn hc_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let chats = hc_chats(st);
    let open = lv.sel.unwrap_or(0);
    let rows: Vec<Stateful<Div>> = chats
        .iter()
        .enumerate()
        .map(|(i, c)| {
            s11_row(
                p,
                "",
                c.title.clone(),
                i == open,
                div()
                    .text_size(px(11.))
                    .text_color(p.mute)
                    .child(if c.pinned { "Pinned" } else { c.at }),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        HC_APP,
        "New chat",
        "Chats",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![
        chip(
            p,
            None,
            if extra == "research" {
                "Deep research"
            } else {
                "Tools"
            },
            extra == "research",
        ),
        chip(p, None, "Thinking", false),
        chip(p, Some(brand(p, "gemini")), "Gemini 3 Pro", false),
    ];
    let comp = composer(p, chips, "Ask anything", false, act, lv);
    let cur = chats.get(open);
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "Hello, Rowan")
    } else if extra == "mini" {
        div().py(px(40.)).flex().justify_center().child(
            div()
                .w(px(440.))
                .rounded(px(16.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(40.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .border_b_1()
                        .border_color(p.line)
                        .child(
                            div()
                                .flex_1()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(HC_APP),
                        )
                        .child(mono(HC_SHORTCUT).text_color(p.mute))
                        .child(press(p, "Open full chat", false, act, "full")),
                )
                .child(
                    div()
                        .p(px(12.))
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(you(p, "How do I say “see you on Thursday” in Portuguese?"))
                        .child(div().child("“Até quinta-feira.” Informal and friendly.")),
                ),
        )
    } else if open != 0 {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, cur.map(|c| c.title.clone()).unwrap_or_default()))
            .child(div().text_color(p.fg2).child(
                "Here's a start. Ask a follow-up, or turn on a tool like Deep research for more.",
            ))
    } else if extra == "research" {
        let on: Vec<bool> = HC_PLAN.iter().map(|s| s.1).collect();
        let mut steps = div().flex().flex_col().gap(px(2.));
        for (l, s) in HC_PLAN {
            steps = steps.child(
                div()
                    .h(px(28.))
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
                            .when(s, |d| d.bg(p.fg)),
                    )
                    .child(clip(l).text_color(if s { p.fg } else { p.mute })),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, HC_PROMPT))
            .child(card(
                p,
                "Research plan",
                format!("About {} min", plan_minutes(&on)),
                steps.child(div().pt(px(8.)).flex().justify_end().child(press(
                    p,
                    "Start research",
                    true,
                    act,
                    "start",
                ))),
            ))
    } else {
        let mut src = div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(mono(HC_QUERY).text_color(p.fg2));
        for (t, site, _) in HC_SOURCES.iter().copied().take(3) {
            src = src.child(
                div()
                    .h(px(24.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(clip(t))
                    .child(div().text_size(px(12.)).text_color(p.mute).child(site)),
            );
        }
        let mut out = div().flex().flex_col();
        let mut table = div().flex().flex_col();
        for (n, usd, km, nm) in HC_BIKES {
            out = out.child(mono(format!(
                "{n}: {} per km of range",
                usd_per_km(usd, km)
            )));
            table = table.child(
                div()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .border_b_1()
                    .border_color(p.line)
                    .child(clip(n))
                    .child(div().w(px(70.)).child(dollars(usd)))
                    .child(div().w(px(60.)).child(format!("{km} km")))
                    .child(
                        div()
                            .w(px(70.))
                            .text_color(tone_of(p, "ok"))
                            .child(format!("{nm} N·m")),
                    ),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, HC_PROMPT))
            .child(card(
                p,
                "Searched the web",
                format!("{} sources", HC_SOURCES.len()),
                src,
            ))
            .child(card(p, "Ran code", "Python · 0.4 s", out))
            .child(card(p, "Made a table", "3 bikes", table))
            .child(div().child(HC_ANSWER))
    };
    let title: SharedString = if extra == "mini" {
        "Mini chat".into()
    } else {
        cur.map(|c| c.title.clone().into())
            .unwrap_or_else(|| "New chat".into())
    };
    let pl = if extra == "research" {
        Some(("Deep research", "wait"))
    } else {
        cur.filter(|c| c.pinned).map(|_| ("Pinned", ""))
    };
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, comp),
        sources(p, st),
    )
}
