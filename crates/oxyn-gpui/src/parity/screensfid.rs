//! Screens 1 for GPUI over `oxyn_core::parity::screensfid`: static snapshots of the app frames
//! ([`ScreenShellEl`], [`ScreenRailEl`]) and the AI Chat screens ([`ChatStartEl`], [`ChatOverviewEl`],
//! [`ChatAgentsEl`], [`ChatRailEl`], [`ChatProgressEl`], [`ChatRunEl`]) Folders ([`FoldersEl`],
//! [`FolderDetailEl`]) and [`IntegrationsEl`]. Each takes the data state (Sample, Loading, Empty), the frame (Inset or
//! Flat) and the theme; the host owns any live state.
//! Rows a host can route carry `Act` ids: `nav-<key>`, `thread-<id>`, `folder-<id>`,
//! `app-<brand id>`.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_core::parity::screensfid::{
    APP_TABS, PERSON, PERSON_INI, ScreenState, ShellLayout, StepState, WORKSPACE, ago, angles,
    app_view, apps, connected_count, crew, folder_files, folder_view, folders, heat, member_name,
    progress_steps, projects, run_plan, sheet, size_label, split_pinned, starters, step_state,
    synced_words, threads, total_size,
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

fn composer(p: &ThPal) -> Div {
    div()
        .w(px(600.))
        .p(px(10.))
        .rounded(px(16.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .h(px(40.))
                .text_color(p.mute)
                .child("Ask anything, or type / for tools"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .text_size(px(12.5))
                .text_color(p.mute)
                .child("+")
                .child("Ask for big steps")
                .child("Auto")
                .child(div().flex_1())
                .child(div().w(px(32.)).h(px(32.)).rounded(px(16.)).bg(p.sel)),
        )
}

fn msg(p: &ThPal, me: bool, t: &'static str) -> Div {
    if me {
        div().flex().justify_end().child(
            div()
                .max_w(px(460.))
                .px(px(13.))
                .py(px(9.))
                .rounded(px(16.))
                .bg(p.sel)
                .child(t),
        )
    } else {
        div()
            .flex()
            .gap(px(10.))
            .child(div().w(px(20.)).h(px(20.)).rounded(px(10.)).bg(p.sel))
            .child(div().max_w(px(560.)).child(t))
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
screen_el!(
    /// AI Chat / Start.
    ChatStartEl
);
screen_el!(
    /// AI Chat / Overview.
    ChatOverviewEl
);
screen_el!(
    /// AI Chat / Agents.
    ChatAgentsEl
);
screen_el!(
    /// AI Chat / Double sidebar.
    ChatRailEl
);
screen_el!(
    /// AI Chat / Progress.
    ChatProgressEl
);
screen_el!(
    /// AI Chat / Run.
    ChatRunEl
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

impl RenderOnce for ChatStartEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut chips = div()
            .flex()
            .flex_wrap()
            .justify_center()
            .gap(px(8.))
            .w(px(600.));
        for (_, l, _) in starters() {
            chips = chips.child(
                div()
                    .h(px(32.))
                    .px(px(13.))
                    .flex()
                    .items_center()
                    .rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .when(self.state == ScreenState::Loading, |d| {
                        d.bg(p.sel).text_color(p.sel)
                    })
                    .child(l),
            );
        }
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.))
            .pt(px(56.))
            .child(
                div()
                    .text_size(px(26.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Good evening, Rowan. What's next?"),
            )
            .child(composer(&p))
            .child(chips);
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "", body),
            None,
        )
    }
}

impl RenderOnce for ChatOverviewEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut grid = div().flex().gap(px(3.));
        let lv = if self.state == ScreenState::Sample {
            heat(20)
        } else {
            vec![0; 140]
        };
        for w in 0..20 {
            let mut col = div().flex().flex_col().gap(px(3.));
            for d in 0..7 {
                let l = lv[w * 7 + d] as f32;
                col = col.child(div().w(px(12.)).h(px(12.)).rounded(px(3.)).bg(if l == 0. {
                    p.sel
                } else {
                    p.fg.opacity(0.2 + l * 0.2)
                }));
            }
            grid = grid.child(col);
        }
        let card = |t: &'static str, b: Div| {
            div()
                .flex_1()
                .p(px(12.))
                .rounded(px(14.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
                .child(b)
        };
        let ld = self.state == ScreenState::Loading;
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.))
            .p(px(22.))
            .child(composer(&p))
            .child(card("Activity", if ld { skel(&p, 3) } else { grid }).w(px(720.)))
            .child(
                div()
                    .flex()
                    .gap(px(12.))
                    .w(px(720.))
                    .child(card(
                        "Credits",
                        if ld {
                            skel(&p, 2)
                        } else {
                            div()
                                .text_size(px(22.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(if self.state == ScreenState::Empty {
                                    "0 of 2,000"
                                } else {
                                    "1,284 of 2,000"
                                })
                        },
                    ))
                    .child(card(
                        "This month",
                        if ld {
                            skel(&p, 2)
                        } else {
                            div().child("Chats 36  +12").child("Files made 14  −3")
                        },
                    )),
            );
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Overview", body),
            None,
        )
    }
}

impl RenderOnce for ChatAgentsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let body = match self.state {
            ScreenState::Loading => skel(&p, 8).p(px(20.)),
            ScreenState::Empty => empty(
                &p,
                "Start with a goal",
                "The orchestrator brings in the agents it needs.",
            ),
            ScreenState::Sample => {
                let mut crew_row = div().flex().flex_wrap().gap(px(6.));
                for m in crew() {
                    crew_row = crew_row.child(
                        div()
                            .h(px(30.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .rounded(px(15.))
                            .border_1()
                            .border_color(if m.done { p.line } else { p.fg2 })
                            .child(m.name),
                    );
                }
                div()
                    .p(px(20.))
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .child(msg(
                        &p,
                        true,
                        "Make launch ads for the overshirt. Lead with how it feels.",
                    ))
                    .child(msg(
                        &p,
                        false,
                        "I split this across five agents. The brief on the right is filling in.",
                    ))
                    .child(
                        div()
                            .p(px(12.))
                            .rounded(px(14.))
                            .border_1()
                            .border_color(p.line)
                            .child(gh(&p, "Agents working"))
                            .child(crew_row),
                    )
            }
        };
        let mut brief = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("Brief"));
        if self.state == ScreenState::Sample {
            for (i, a) in angles().into_iter().enumerate() {
                brief = brief.child(
                    div()
                        .p(px(8.))
                        .rounded(px(11.))
                        .border_1()
                        .border_color(if i == 0 { p.fg } else { p.line })
                        .child(a.label),
                );
            }
            brief = brief.child(
                div()
                    .mt(px(10.))
                    .h(px(32.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(9.))
                    .bg(p.fg)
                    .text_color(p.card)
                    .child("Make ads"),
            );
        }
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Ads for the overshirt", body),
            Some(brief),
        )
    }
}

impl RenderOnce for ChatRailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let body = match self.state {
            ScreenState::Loading => skel(&p, 8).p(px(20.)),
            ScreenState::Empty => empty(
                &p,
                "Ask with the web",
                "Searches fold into one line you can open.",
            ),
            ScreenState::Sample => div()
                .p(px(20.))
                .flex()
                .flex_col()
                .gap(px(14.))
                .child(msg(&p, true, "Compare five fabrics for the overshirt."))
                .child(
                    div()
                        .h(px(28.))
                        .w(px(180.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .rounded(px(14.))
                        .border_1()
                        .border_color(p.line)
                        .text_color(p.mute)
                        .child("Ran 3 searches · 4.1s"),
                )
                .child(msg(
                    &p,
                    false,
                    "Washed linen and hemp twill are the sturdiest; organic cotton is cheapest.",
                )),
        };
        let mut grid = div()
            .flex()
            .flex_col()
            .border_1()
            .border_color(p.line)
            .rounded(px(9.))
            .overflow_hidden();
        if self.state == ScreenState::Sample {
            for (r, rowv) in sheet().into_iter().enumerate() {
                let mut tr = div().flex();
                for v in rowv {
                    tr = tr.child(
                        div()
                            .w(px(76.))
                            .h(px(26.))
                            .px(px(6.))
                            .flex()
                            .items_center()
                            .border_r_1()
                            .border_b_1()
                            .border_color(p.line)
                            .when(r == 0, |d| d.font_weight(FontWeight::SEMIBOLD))
                            .text_size(px(11.5))
                            .overflow_hidden()
                            .child(v),
                    );
                }
                grid = grid.child(tr);
            }
        }
        frame(
            self.theme,
            self.layout,
            rail_left(&p, self.state),
            page(&p, "Venue shortlist for the offsite", body),
            Some(div().child(gh(&p, "fabric-compare.csv")).child(grid)),
        )
    }
}

impl RenderOnce for ChatProgressEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let body = match self.state {
            ScreenState::Loading => skel(&p, 8).p(px(20.)),
            ScreenState::Empty => empty(
                &p,
                "Ask for something that takes a few steps",
                "Each step and tool shows as it goes.",
            ),
            ScreenState::Sample => div()
                .p(px(20.))
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(msg(
                    &p,
                    true,
                    "Three people start Monday. Plan their first week.",
                ))
                .child(
                    row(
                        &p,
                        "wiki.search · Looked up the first-week pages".to_string(),
                        "1.8s".to_string(),
                        false,
                    )
                    .border_1()
                    .border_color(p.line),
                )
                .child(
                    row(
                        &p,
                        "drive.read · Read the intake form".to_string(),
                        "0.9s".to_string(),
                        false,
                    )
                    .border_1()
                    .border_color(p.line),
                )
                .child(msg(&p, false, "Here is the week at a glance.")),
        };
        let mut steps = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("Progress"));
        if self.state == ScreenState::Sample {
            for (i, s) in progress_steps().into_iter().enumerate() {
                let k = step_state(i, 3);
                steps = steps.child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .w(px(16.))
                                .h(px(16.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(p.fg2)
                                .when(k == StepState::Done, |d| d.bg(p.fg)),
                        )
                        .text_color(if k == StepState::Pending {
                            p.mute
                        } else {
                            p.fg
                        })
                        .child(s),
                );
            }
        }
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Plan the new starters' week", body),
            Some(steps),
        )
    }
}

impl RenderOnce for ChatRunEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let body = match self.state {
            ScreenState::Loading => skel(&p, 10).p(px(20.)),
            ScreenState::Empty => empty(
                &p,
                "No runs yet",
                "Give an agent a goal; its plan shows up here.",
            ),
            ScreenState::Sample => {
                let mut plan = div()
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(p.line)
                    .rounded(px(12.));
                for (i, s) in run_plan().into_iter().enumerate() {
                    let k = step_state(i, 2);
                    plan = plan.child(
                        div()
                            .h(px(46.))
                            .px(px(12.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .border_b_1()
                            .border_color(p.line)
                            .child(
                                div()
                                    .w(px(10.))
                                    .h(px(10.))
                                    .rounded(px(5.))
                                    .border_1()
                                    .border_color(p.fg)
                                    .when(k == StepState::Done, |d| d.bg(p.fg)),
                            )
                            .child(div().flex_1().child(s.title))
                            .child(div().text_size(px(11.5)).text_color(p.fg2).child(s.tool))
                            .child(
                                div()
                                    .w(px(70.))
                                    .text_size(px(12.))
                                    .text_color(p.mute)
                                    .child(k.label()),
                            ),
                    );
                }
                div()
                    .p(px(20.))
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(20.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Write the November member update"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child("Steps 2/5")
                            .child("Tool calls 5")
                            .child("Time 0:17")
                            .child("Cost $0.10"),
                    )
                    .child(plan)
            }
        };
        let side = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("Draft"))
            .child(
                div()
                    .text_color(p.fg2)
                    .child("In October membership grew 18% · we expect · by 12 December"),
            );
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, true),
            page(&p, "Runs / Member update", body),
            Some(side),
        )
    }
}

screen_el!(
    /// Folders / Overview: folder cards with members, agent access and when they last changed.
    /// Rows a host can route carry `folder-<id>`.
    FoldersEl
);
screen_el!(
    /// Folders / Detail: one folder's files as a table, with its details beside it.
    FolderDetailEl
);

fn faces_el(p: &ThPal, members: &[String]) -> Div {
    let mut d = div().flex().items_center().gap(px(2.));
    for m in members.iter().take(4) {
        d = d.child(
            div()
                .w(px(22.))
                .h(px(22.))
                .rounded(px(11.))
                .bg(p.sel)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(9.5))
                .child(SharedString::from(m.clone())),
        );
    }
    d
}

impl RenderOnce for FoldersEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut body = div().flex().flex_col().gap(px(12.)).p(px(22.)).child(
            div()
                .text_size(px(22.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Folders"),
        );
        match self.state {
            ScreenState::Loading => body = body.child(skel(&p, 6)),
            ScreenState::Empty => {
                body = body.child(empty(
                    &p,
                    "No folders yet",
                    "Make one for a project: files, chats and agents can all share it.",
                ))
            }
            ScreenState::Sample => {
                let mut grid = div().flex().flex_wrap().gap(px(10.));
                for f in folder_view(&folders(), "", false) {
                    const IDS: [&str; 6] = [
                        "folder-1", "folder-2", "folder-3", "folder-4", "folder-5", "folder-6",
                    ];
                    let key = IDS
                        .get((f.id as usize).wrapping_sub(1))
                        .copied()
                        .unwrap_or("folder");
                    let fi = fire(&self.act, key);
                    grid = grid.child(
                        div()
                            .id(SharedString::from(key))
                            .w(px(232.))
                            .p(px(12.))
                            .rounded(px(12.))
                            .border_1()
                            .border_color(p.line)
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(if f.starred {
                                format!("★ {}", f.name)
                            } else {
                                f.name.clone()
                            }))
                            .child(div().text_color(p.mute).child(f.note.clone()))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(faces_el(&p, &f.members))
                                    .when(f.agents, |d| {
                                        d.child(div().text_color(p.fg2).child("Agents"))
                                    })
                                    .child(div().flex_1())
                                    .child(div().text_color(p.mute).child(ago(f.mins))),
                            )
                            .on_click(move |_, w, cx| fi(w, cx)),
                    );
                }
                body = body.child(grid);
            }
        }
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Folders", body),
            None,
        )
    }
}

impl RenderOnce for FolderDetailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let f = folders().remove(0);
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(22.))
            .child(div().text_color(p.mute).child("← All folders"))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(f.name.clone()),
            )
            .child(div().text_color(p.mute).child(f.note.clone()));
        let files = if self.state == ScreenState::Sample {
            folder_files()
        } else {
            vec![]
        };
        match self.state {
            ScreenState::Loading => body = body.child(skel(&p, 8)),
            ScreenState::Empty => {
                body = body.child(empty(
                    &p,
                    "Nothing in this folder",
                    "Add a file and everyone it is shared with sees it.",
                ))
            }
            ScreenState::Sample => {
                let mut tb = div()
                    .flex()
                    .flex_col()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .child(row(&p, "Name".into(), "Edited · Size".into(), false));
                for x in &files {
                    tb = tb.child(row(
                        &p,
                        format!("{}  ·  {}", x.name, x.kind),
                        format!("{} · {}", ago(x.mins), size_label(x.kb)),
                        false,
                    ));
                }
                body = body.child(tb).child(div().text_color(p.mute).child(format!(
                    "{} files · {}",
                    files.len(),
                    total_size(&files)
                )));
            }
        }
        let mut info = div().flex().flex_col().gap(px(4.)).child(gh(&p, "About"));
        if self.state == ScreenState::Loading {
            info = info.child(skel(&p, 6));
        } else {
            info = info
                .child(div().px(px(8.)).child(f.note.clone()))
                .child(gh(&p, "Shared with"));
            let members = if self.state == ScreenState::Sample {
                f.members.clone()
            } else {
                vec![PERSON_INI.to_string()]
            };
            for m in &members {
                let r = if m == PERSON_INI { "Owner" } else { "Can edit" };
                info = info.child(row(&p, member_name(m).to_string(), r.to_string(), false));
            }
            info = info.child(row(
                &p,
                "Agents can read this folder".into(),
                if f.agents && self.state == ScreenState::Sample {
                    "On".into()
                } else {
                    "Off".into()
                },
                false,
            ));
        }
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Folders", body),
            Some(info),
        )
    }
}

screen_el!(
    /// Integrations: the app directory with connect states and a connected app's permissions.
    /// Cards a host can route carry `app-<brand id>`.
    IntegrationsEl
);

impl RenderOnce for IntegrationsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let list = apps(self.state == ScreenState::Sample);
        let n = connected_count(&list);
        let mut tabs = div().flex().gap(px(14.)).text_color(p.mute);
        for (k, l) in APP_TABS {
            tabs = tabs.child(
                div()
                    .when(k == "all", |d| {
                        d.text_color(p.fg).font_weight(FontWeight::SEMIBOLD)
                    })
                    .child(if k == "on" {
                        format!("{l} {n}")
                    } else {
                        l.to_string()
                    }),
            );
        }
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(22.))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Integrations"),
            )
            .child(tabs);
        if self.state == ScreenState::Loading {
            body = body.child(skel(&p, 6));
        } else {
            let mut grid = div().flex().flex_wrap().gap(px(10.));
            for a in app_view(&list, "all", "") {
                let key: &'static str = match a.id {
                    "googledrive" => "app-googledrive",
                    "dropbox" => "app-dropbox",
                    "notion" => "app-notion",
                    "slack" => "app-slack",
                    "discord" => "app-discord",
                    "gmail" => "app-gmail",
                    "googlecalendar" => "app-googlecalendar",
                    "zoom" => "app-zoom",
                    "linear" => "app-linear",
                    "figma" => "app-figma",
                    "github" => "app-github",
                    _ => "app-sentry",
                };
                let fi = fire(&self.act, key);
                grid = grid.child(
                    div()
                        .id(SharedString::from(key))
                        .w(px(232.))
                        .p(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(if a.connected { p.fg2 } else { p.line })
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(a.name))
                        .child(div().text_color(p.fg2).child(a.blurb))
                        .child(div().text_color(p.mute).child(if a.connected {
                            "Connected · Manage"
                        } else {
                            "Connect"
                        }))
                        .on_click(move |_, w, cx| fi(w, cx)),
                );
            }
            body = body.child(grid);
        }
        let work = if self.state == ScreenState::Sample {
            let a = list.iter().find(|a| a.id == "slack").cloned();
            a.map(|a| {
                let mut w = div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(row(&p, a.name.to_string(), synced_words(a.synced), false))
                    .child(gh(&p, "It may"));
                for (s, on) in a.scopes {
                    w = w.child(row(
                        &p,
                        s.to_string(),
                        if on { "On" } else { "Off" }.to_string(),
                        false,
                    ));
                }
                w.child(row(&p, "Sync now".into(), "Disconnect".into(), false))
            })
        } else {
            None
        };
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Integrations", body),
            work,
        )
    }
}
