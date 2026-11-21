//! Screens S11 in GPUI (Agent apps III): Workspace Coder. A static snapshot: chats in the sidebar,
//! the agent's progress list and tool log, the composer with mode, Thinking and model, and the
//! Workspace pane on Files (a file and its lines) or Preview (the built page's bars).
//! `.state()` sets Data / Loading / Empty and `.extra()` the own state (`preview`, `files`) or
//! `rail` / `hidden`. Presses fire `new`, `send` and `publish` through `on_act`. Rules from
//! `oxyn_core::kit::wscoder`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view; the elements
//! below stay single-state snapshots.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, Stateful, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_core::kit::wscoder::{
    ScreenState, WC_APP, WC_AT, WC_DONE, WC_LOG, WC_MODES, WC_PROMPT, WC_TODOS, WS_BROWSER,
    WS_TABS, WS_TERM, cheapest, preview_rows, wc_chats, ws_files,
};
use oxyn_tokens::Theme;

use super::Act;
use super::apps::{Live, hit};
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, dot, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;

s11_el!(
    /// Workspace Coder: chats, the agent's progress and tool log, the composer, and the Workspace
    /// pane (Files or Preview).
    WorkspaceCoderEl,
    wc_screen
);

fn ws_pane(p: &ThPal, st: ScreenState, extra: &str, act: &Option<Act>, lv: &Live) -> Div {
    let live = lv.data.is_some();
    let on = match &lv.data {
        Some(d) => d.ws_tab,
        None if extra == "preview" => 3,
        None => 0,
    };
    let mut tabs = div()
        .flex()
        .gap(px(4.))
        .p(px(6.))
        .border_b_1()
        .border_color(p.line);
    for (i, (_, l)) in WS_TABS.iter().copied().enumerate() {
        let id = match i {
            _ if live => oxyn_core::kit::appui::ids::at(&oxyn_core::kit::appui::ids::WSTAB, i),
            0 => "view:files",
            3 => "view:preview",
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
                .child(l),
            act,
            id,
        ));
    }
    let mut d = div()
        .w(px(440.))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(p.line)
        .child(
            div()
                .h(px(44.))
                .px(px(12.))
                .flex()
                .items_center()
                .border_b_1()
                .border_color(p.line)
                .font_weight(FontWeight::SEMIBOLD)
                .child("Workspace"),
        )
        .child(tabs);
    let files = ws_files(st);
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if files.is_empty() {
        return d.child(faint(p, "The agent's computer shows up here"));
    }
    if on == 1 {
        let mut out = div()
            .flex_1()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(2.))
            .bg(p.inset);
        for (cmd, l) in WS_TERM {
            out = out.child(
                mono(if cmd { format!("$ {l}") } else { l.to_string() }).text_color(if cmd {
                    p.fg
                } else {
                    p.fg2
                }),
            );
        }
        return d.child(out);
    }
    if on == 2 {
        let (addr, hits) = WS_BROWSER;
        let mut page = div().flex_1().p(px(12.)).flex().flex_col().gap(px(10.));
        for (t, site, line) in hits {
            page = page.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(div().text_size(px(11.5)).text_color(p.mute).child(site))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
                    .child(div().text_size(px(12.5)).text_color(p.fg2).child(line)),
            );
        }
        return d
            .child(
                div()
                    .m(px(8.))
                    .h(px(28.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .bg(p.inset)
                    .text_size(px(12.))
                    .child(clip(addr)),
            )
            .child(page);
    }
    if on == 3 {
        let rows = preview_rows(&files);
        let low = cheapest(&rows);
        let max = rows.iter().map(|r| r.1).max().unwrap_or(1).max(1) as f32;
        let mut page = div()
            .m(px(10.))
            .p(px(16.))
            .rounded(px(10.))
            .border_1()
            .border_color(p.line)
            .bg(gpui::white())
            .text_color(gpui::black())
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("A month in three cities"),
            );
        for (i, (c, v)) in rows.iter().enumerate() {
            page = page.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().w(px(64.)).child(c.clone()))
                    .child(
                        div().flex_1().h(px(12.)).rounded(px(4.)).bg(p.sel).child(
                            div()
                                .h_full()
                                .w(px(220. * *v as f32 / max))
                                .rounded(px(4.))
                                .bg(if Some(i) == low {
                                    tone_of(p, "ok")
                                } else {
                                    p.mute
                                }),
                        ),
                    )
                    .child(format!("€{v}")),
            );
        }
        d = d
            .child(
                div()
                    .p(px(10.))
                    .flex()
                    .justify_end()
                    .child(press(p, "Publish", true, act, "publish")),
            )
            .child(page);
    } else {
        let f = &files[if extra == "files" { 1 } else { 0 }];
        let mut code = div().flex_1().p(px(8.)).bg(p.inset).flex().flex_col();
        for (n, l) in f.lines.iter().enumerate() {
            code = code.child(
                div()
                    .flex()
                    .gap(px(10.))
                    .child(
                        div()
                            .w(px(18.))
                            .text_color(p.mute)
                            .child(mono((n + 1).to_string())),
                    )
                    .child(mono(*l)),
            );
        }
        let mut tree = div()
            .w(px(148.))
            .p(px(6.))
            .border_r_1()
            .border_color(p.line)
            .flex()
            .flex_col();
        for x in &files {
            tree = tree.child(
                div()
                    .h(px(28.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .when(x.path == f.path, |d| d.bg(p.sel))
                    .child(mono(x.path)),
            );
        }
        d = d.child(div().flex_1().min_h_0().flex().child(tree).child(code));
    }
    d
}

pub(super) fn wc_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let done = !extra.is_empty() && extra != "rail" && extra != "hidden";
    let rows: Vec<Stateful<Div>> = wc_chats(st)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let run = c.running && !done;
            s11_row(
                p,
                if run { "run" } else { "" },
                c.title.clone(),
                lv.on(i, c.id == 1),
                div().text_color(p.mute).child(WC_MODES[c.mode].0),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        WC_APP,
        "New chat",
        "Chats",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let chips = vec![
        chip(p, None, "Workspace", false),
        chip(p, None, "Thinking", false),
        chip(p, Some(brand(p, "kimi")), "Kimi K3", false),
    ];
    let comp = composer(
        p,
        chips,
        "Ask anything, or describe what to build",
        false,
        act,
        lv,
    );
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "What can I build for you?")
    } else {
        let mut todo = div()
            .p(px(12.))
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("Progress"));
        for (i, t) in WC_TODOS.iter().copied().enumerate() {
            let tn = if done || i < WC_AT {
                "ok"
            } else if i == WC_AT {
                "run"
            } else {
                ""
            };
            todo = todo.child(
                div()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(dot(tone_of(p, tn)))
                    .child(clip(t)),
            );
        }
        let mut log = div().flex().flex_col().gap(px(2.));
        for (_, v, o) in WC_LOG.iter().copied().take(if done { 6 } else { 5 }) {
            log = log.child(
                div()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(v))
                    .child(mono(o)),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, WC_PROMPT))
            .child(todo)
            .child(log)
            .child(if done {
                div().child(WC_DONE)
            } else {
                div().text_color(p.mute).child("Building the page")
            })
    };
    let pane = ws_pane(p, st, extra, act, lv);
    let samp = st == ScreenState::Sample;
    let title = if samp {
        "Moving budget for three cities"
    } else {
        "New chat"
    };
    let pl = if samp {
        Some(if done {
            ("Workspace", "")
        } else {
            ("Working", "run")
        })
    } else {
        None
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
