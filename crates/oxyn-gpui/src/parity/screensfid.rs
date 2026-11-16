//! Screens 1 for GPUI over `oxyn_core::parity::screensfid`: static snapshots of the app frames
//! ([`ScreenShellEl`], [`ScreenRailEl`]) and the AI Chat screens ([`ChatStartEl`], [`ChatOverviewEl`],
//! [`ChatAgentsEl`], [`ChatRailEl`], [`ChatProgressEl`], [`ChatRunEl`]). Each takes the data state
//! (Sample, Loading, Empty), the frame (Inset or Flat) and the theme; the host owns any live state.
//! Rows a host can route carry `Act` ids: `nav-<key>`, `thread-<id>`.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_core::parity::screensfid::{
    PERSON, PERSON_INI, ScreenState, ShellLayout, WORKSPACE, ago, projects, split_pinned, threads,
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{ThPal, fire, th_pal};

fn skel(p: &ThPal, n: usize) -> Div {
    let mut d = div().flex().flex_col().gap(px(10.)).p(px(10.));
    for i in 0..n {
        d = d.child(
            div()
                .h(px(12.))
                .w(px([220., 150., 190., 110., 170.][i % 5]))
                .rounded(px(6.))
                .bg(p.sel),
        );
    }
    d
}

fn empty(p: &ThPal, t: &'static str, l: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .py(px(48.))
        .child(div().w(px(40.)).h(px(40.)).rounded(px(12.)).bg(p.sel))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
        .child(div().text_color(p.mute).child(l))
}

fn gh(p: &ThPal, t: &'static str) -> Div {
    div()
        .mt(px(10.))
        .mb(px(4.))
        .px(px(8.))
        .text_size(px(11.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(p.mute)
        .child(t)
}

fn row(p: &ThPal, t: String, a: String, on: bool) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .h(px(30.))
        .px(px(8.))
        .rounded(px(8.))
        .when(on, |d| d.bg(p.sel))
        .text_color(p.fg2)
        .child(div().flex_1().overflow_hidden().child(t))
        .child(div().text_size(px(12.)).text_color(p.mute).child(a))
}

fn sidebar(p: &ThPal, st: ScreenState, act: &Option<Act>, runs: bool) -> Div {
    let mut s = div()
        .w(px(248.))
        .flex_none()
        .flex()
        .flex_col()
        .p(px(8.))
        .bg(p.inset)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(34.))
                .px(px(6.))
                .child(div().w(px(22.)).h(px(22.)).rounded(px(7.)).bg(p.fg))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(WORKSPACE)),
        );
    s = s.child(
        div()
            .h(px(32.))
            .my(px(8.))
            .px(px(8.))
            .flex()
            .items_center()
            .rounded(px(9.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.mute)
            .child("Search  ⌘K"),
    );
    let nav: Vec<(&str, &str)> = if runs {
        vec![
            ("home", "Home"),
            ("chat", "Chat"),
            ("runs", "Runs  3"),
            ("stats", "Numbers"),
            ("made", "Made for you"),
        ]
    } else {
        vec![("new", "New chat"), ("files", "Files"), ("apps", "Apps")]
    };
    for (k, l) in nav {
        let f = fire(act, k);
        s = s.child(
            div()
                .id(SharedString::from(format!("nav-{k}")))
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .text_color(p.fg2)
                .when(runs && k == "runs", |d| d.bg(p.sel))
                .child(l)
                .on_click(move |_, w, cx| f(w, cx)),
        );
    }
    match st {
        ScreenState::Loading => s = s.child(gh(p, "Chats")).child(skel(p, 6)),
        ScreenState::Empty => {
            s = s.child(gh(p, "Chats")).child(
                div()
                    .px(px(8.))
                    .text_color(p.mute)
                    .child("Chats you start land here."),
            )
        }
        ScreenState::Sample if runs => {
            s = s.child(gh(p, "Agents"));
            for a in ["Member update", "Revenue desk", "Inbox sweeper", "Flows"] {
                s = s.child(row(p, a.to_string(), String::new(), false));
            }
        }
        ScreenState::Sample => {
            let (pin, rest) = split_pinned(&threads());
            s = s.child(gh(p, "Pinned"));
            for t in pin {
                s = s.child(row(p, t.title, ago(t.mins), false));
            }
            s = s.child(gh(p, "Projects"));
            for pr in projects() {
                s = s.child(row(
                    p,
                    pr.label.to_string(),
                    pr.chats.len().to_string(),
                    false,
                ));
            }
            s = s.child(gh(p, "Chats"));
            for t in rest {
                s = s.child(row(p, t.title, ago(t.mins), t.id == 2));
            }
        }
    }
    s.child(div().flex_1()).child(
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(36.))
            .px(px(6.))
            .child(
                div()
                    .w(px(26.))
                    .h(px(26.))
                    .rounded(px(13.))
                    .bg(p.sel)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.5))
                    .child(PERSON_INI),
            )
            .child(PERSON),
    )
}

fn page(p: &ThPal, title: &'static str, body: Div) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(46.))
                .px(px(14.))
                .flex()
                .items_center()
                .border_b_1()
                .border_color(p.line)
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(body)
}

fn work_pane(p: &ThPal, w: Div) -> Div {
    div()
        .w(px(340.))
        .flex_none()
        .border_l_1()
        .border_color(p.line)
        .bg(p.card2)
        .p(px(12.))
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(w)
}

fn frame(theme: &Theme, lay: ShellLayout, left: Div, main: Div, work: Option<Div>) -> Div {
    let p = th_pal(theme);
    let mut pg = div().flex_1().flex().bg(p.card).child(main);
    if let Some(w) = work {
        pg = pg.child(work_pane(&p, w));
    }
    let pg = match lay {
        ShellLayout::Inset => pg
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .overflow_hidden(),
        ShellLayout::Flat => pg.border_l_1().border_color(p.line),
    };
    let root = div()
        .w(px(1100.))
        .h(px(660.))
        .flex()
        .rounded(px(16.))
        .border_1()
        .border_color(p.line)
        .bg(p.inset)
        .text_color(p.fg)
        .text_size(px(13.))
        .overflow_hidden()
        .child(left)
        .child(pg);
    match lay {
        ShellLayout::Inset => root.pt(px(8.)).pb(px(8.)).pr(px(8.)),
        ShellLayout::Flat => root,
    }
}

macro_rules! screen_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        pub struct $name {
            state: ScreenState,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, layout: ShellLayout::Inset, act: None, theme: &Theme::DARK }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// Inset or Flat.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// `nav-<key>` from the sidebar.
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
    };
}

screen_el!(
    /// App shell (Inset or Flat): sidebar, a placeholder page and the workspace.
    ScreenShellEl
);
screen_el!(
    /// Three-pane shell: icon rail, chats column, page and work area.
    ScreenRailEl
);

fn workspace(p: &ThPal, st: ScreenState) -> Div {
    let mut w = div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .h(px(30.))
                .w(px(110.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .child("New tab"),
        )
        .child(
            div()
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .text_color(p.mute)
                .child("Search or type an address"),
        );
    if st == ScreenState::Loading {
        return w.child(skel(p, 6));
    }
    w = w.child(gh(p, "Tools"));
    for (t, k) in [("Terminal", "⌃`"), ("Browser", "⌘⇧B"), ("Notes", "⌘⇧N")] {
        w = w.child(row(p, t.to_string(), k.to_string(), false));
    }
    w = w.child(gh(p, "Suggested"));
    if st == ScreenState::Empty {
        return w.child(
            div()
                .px(px(8.))
                .text_color(p.mute)
                .child("Pages you open often show up here."),
        );
    }
    for (t, d) in [
        ("Two papers on dye-free linen", "papers.example.org"),
        ("Launch checklist", "wiki.kestrel.example"),
    ] {
        w = w.child(row(p, t.to_string(), d.to_string(), false));
    }
    w
}

fn placeholder(p: &ThPal, st: ScreenState) -> Div {
    match st {
        ScreenState::Loading => skel(p, 7).p(px(24.)),
        ScreenState::Empty => empty(
            p,
            "Your page goes here",
            "The sidebar and work area stay put.",
        ),
        ScreenState::Sample => {
            let mut g = div().flex().flex_wrap().gap(px(12.)).mt(px(20.));
            for _ in 0..6 {
                g = g.child(
                    div()
                        .w(px(160.))
                        .h(px(110.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.inset),
                );
            }
            div()
                .p(px(24.))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Your page goes here"),
                )
                .child(g)
        }
    }
}

impl RenderOnce for ScreenShellEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Home", placeholder(&p, self.state)),
            Some(workspace(&p, self.state)),
        )
    }
}

fn rail_left(p: &ThPal, st: ScreenState) -> Div {
    let mut rail = div()
        .w(px(52.))
        .flex_none()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .py(px(10.))
        .child(
            div()
                .w(px(22.))
                .h(px(22.))
                .mb(px(8.))
                .rounded(px(7.))
                .bg(p.fg),
        );
    for i in 0..5 {
        rail = rail.child(
            div()
                .w(px(36.))
                .h(px(36.))
                .rounded(px(10.))
                .when(i == 0, |d| d.bg(p.sel)),
        );
    }
    let mut col = div()
        .w(px(230.))
        .flex_none()
        .p(px(8.))
        .flex()
        .flex_col()
        .bg(p.card2)
        .child(
            div()
                .h(px(30.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("All projects"),
        )
        .child(
            div()
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .text_color(p.mute)
                .child("Search"),
        )
        .child(
            div()
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .child("New chat"),
        );
    col = match st {
        ScreenState::Loading => col.child(skel(p, 7)),
        ScreenState::Empty => col.child(div().px(px(8.)).text_color(p.mute).child("No chats yet.")),
        ScreenState::Sample => {
            let mut c = col.child(gh(p, "Chats"));
            for t in threads() {
                c = c.child(row(p, t.title, String::new(), t.id == 3));
            }
            c
        }
    };
    div().flex().flex_none().child(rail).child(col)
}

impl RenderOnce for ScreenRailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        frame(
            self.theme,
            self.layout,
            rail_left(&p, self.state),
            page(&p, "Home", placeholder(&p, self.state)),
            Some(workspace(&p, self.state)),
        )
    }
}
