//! App shells and screens for GPUI (plan #27, chunk 4), drawn from `oxyn_core::shell`:
//! `SidebarEl` (open or icon rail), `RailShellEl` (resizable sidebar with the separator and a
//! breadcrumb), `WorkspaceSidebarEl`, `InboxEl`, `AppShellEl` (flat · inset), `AiPanelEl`,
//! `TermTabsEl`, `SidebarNavEl`, `ChatShellEl` and `HarnessEl`.
//!
//! Elements are `RenderOnce`: the host owns the models (`Rail`, `Inbox`, `TermTabs`, `PageEdit`),
//! routes presses back through the `on_*` callbacks and re-renders. Gaps against Dioxus (listed in
//! the PR): the separator drag takes x from the host's mouse handlers (`Rail::drag`), message
//! drag-and-drop onto folders is a host concern (the row's Move menu is drawn instead), the
//! compose window, workspace search, new-project field, terminal rename field and side-panel
//! prompt are display-only (text entry belongs to the host's input), menus draw in place, and
//! there is no slide / grow / sliding-indicator motion or answer streaming.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    Styled, Window, div, prelude::*, px,
};
use oxyn_core::shell::{DiffOp, EditPhase, Inbox, NavSection, PageEdit, RAIL_W, Rail, TermTabs};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type OnId = Rc<dyn Fn(String, &mut Window, &mut App)>;
type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn glyph(name: &str) -> &'static str {
    match name {
        "home" => "⌂",
        "inbox" => "▤",
        "folder" => "▣",
        "calendar" => "▦",
        "search" => "⌕",
        "star" => "★",
        "text" | "page" => "▢",
        "clock" => "◷",
        "up" => "↑",
        "pencil" => "✎",
        "archive" => "▥",
        "cog" => "⚙",
        "plus" => "+",
        "thread" => "◫",
        "agents" => "◎",
        "spark" => "✦",
        _ => "•",
    }
}

fn initials(s: &str) -> String {
    s.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect()
}

macro_rules! themed {
    () => {
        /// Theme.
        pub fn theme(mut self, t: &'static Theme) -> Self {
            self.theme = t;
            self
        }
    };
}

macro_rules! on_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        }
    };
}

macro_rules! on_index {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        }
    };
}

fn logo(t: &'static Theme, s: &str, size: f32) -> gpui::Div {
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size))
        .rounded(px(size * 0.28))
        .bg(c(t, ColorRole::FgDefault))
        .text_color(c(t, ColorRole::BgBase))
        .text_size(px(size * 0.42))
        .font_weight(FontWeight::SEMIBOLD)
        .child(initials(s))
}

fn avatar(t: &'static Theme, s: &str) -> gpui::Div {
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .size(px(28.))
        .rounded_full()
        .bg(c(t, ColorRole::SurfaceSunken))
        .border_1()
        .border_color(c(t, ColorRole::BorderSubtle))
        .text_size(px(11.))
        .text_color(c(t, ColorRole::FgMuted))
        .child(initials(s))
}

fn two_line(t: &'static Theme, a: &str, b: &str) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(13.))
                .child(a.to_string()),
        )
        .child(
            div()
                .text_size(px(11.5))
                .text_color(c(t, ColorRole::FgMuted))
                .child(b.to_string()),
        )
}

fn heading(t: &'static Theme, s: &str) -> gpui::Div {
    div()
        .px(px(10.))
        .pt(px(8.))
        .pb(px(4.))
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(c(t, ColorRole::FgSubtle))
        .child(s.to_uppercase())
}

/// One sidebar row.
fn row(
    t: &'static Theme,
    id: impl Into<gpui::ElementId>,
    icon: &str,
    label: &str,
    count: u32,
    on: bool,
    rail: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(SharedString::from(if count > 0 {
            format!("{label}, {count}")
        } else {
            label.to_string()
        }))
        .relative()
        .flex()
        .items_center()
        .gap(px(9.))
        .h(px(if rail { 36. } else { 32. }))
        .px(px(if rail { 0. } else { 10. }))
        .when(rail, |d| d.justify_center())
        .rounded(px(7.))
        .cursor_pointer()
        .text_color(c(
            t,
            if on {
                ColorRole::FgDefault
            } else {
                ColorRole::FgMuted
            },
        ))
        .when(on, |d| {
            d.bg(c(t, ColorRole::SurfaceRaised))
                .border_1()
                .border_color(c(t, ColorRole::BorderSubtle))
        })
        .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
        .child(div().w(px(16.)).child(glyph(icon)))
        .when(!rail, |d| {
            d.child(div().flex_1().min_w_0().child(label.to_string()))
                .when(count > 0, |d| {
                    d.child(
                        div()
                            .px(px(5.))
                            .rounded_full()
                            .border_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .text_size(px(11.))
                            .child(count.to_string()),
                    )
                })
        })
        .when(rail && count > 0, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(7.))
                    .right(px(9.))
                    .size(px(7.))
                    .rounded_full()
                    .bg(c(t, ColorRole::StatusDangerFg)),
            )
        })
}

fn sidebar_frame(t: &'static Theme, w: f32) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .flex_none()
        .w(px(w))
        .h_full()
        .bg(c(t, ColorRole::SurfaceSunken))
        .border_r_1()
        .border_color(c(t, ColorRole::BorderSubtle))
        .text_size(px(13.))
        .text_color(c(t, ColorRole::FgDefault))
        .overflow_hidden()
}

// ─────────────────────────────────────────── Workspace sidebar ──

/// Workspace sidebar: switcher (with its menu drawn in place when `menu_open`), search line,
/// favourites, projects, storage meter and account. `on_navigate` gets a name, `on_switch` a
/// workspace index, `on_menu` toggles the switcher (0).
#[derive(IntoElement)]
pub struct WorkspaceSidebarEl {
    workspaces: Vec<(String, String)>,
    current: usize,
    favorites: Vec<String>,
    projects: Vec<String>,
    query: String,
    active: String,
    menu_open: bool,
    rail: bool,
    storage: f32,
    on_navigate: Option<OnId>,
    on_switch: Option<OnIndex>,
    on_menu: Option<OnIndex>,
    theme: &'static Theme,
}

impl WorkspaceSidebarEl {
    /// From workspaces (name, plan), favourites and projects.
    pub fn new(
        workspaces: Vec<(String, String)>,
        favorites: Vec<String>,
        projects: Vec<String>,
    ) -> Self {
        Self {
            workspaces,
            current: 0,
            favorites,
            projects,
            query: String::new(),
            active: String::new(),
            menu_open: false,
            rail: false,
            storage: 0.38,
            on_navigate: None,
            on_switch: None,
            on_menu: None,
            theme: &Theme::DARK,
        }
    }
    /// Current workspace, active row, search text, menu and rail state, storage (0–1).
    pub fn state(
        mut self,
        current: usize,
        active: impl Into<String>,
        query: impl Into<String>,
        menu_open: bool,
        rail: bool,
        storage: f32,
    ) -> Self {
        self.current = current;
        self.active = active.into();
        self.query = query.into();
        self.menu_open = menu_open;
        self.rail = rail;
        self.storage = storage;
        self
    }
    on_id!(
        on_navigate,
        "A favourite or project was pressed (its name)."
    );
    on_index!(on_switch, "A workspace was picked (index).");
    on_index!(on_menu, "The switcher was pressed (0).");
    themed!();
}

impl RenderOnce for WorkspaceSidebarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.rail;
        let (wn, wp) = self
            .workspaces
            .get(self.current)
            .cloned()
            .unwrap_or_default();
        let q = self.query.to_lowercase();
        let fav: Vec<String> = self
            .favorites
            .iter()
            .filter(|f| f.to_lowercase().contains(&q))
            .cloned()
            .collect();
        let pro: Vec<String> = self
            .projects
            .iter()
            .filter(|f| f.to_lowercase().contains(&q))
            .cloned()
            .collect();
        let menu = self.on_menu.clone();
        let list = |icon: &'static str, items: Vec<String>, key: &'static str| {
            items
                .into_iter()
                .enumerate()
                .map(|(i, s)| {
                    let f = self.on_navigate.clone();
                    let s2 = s.clone();
                    row(t, (key, i), icon, &s, 0, s == self.active, r)
                        .when_some(f, move |d, f| {
                            d.on_click(move |_, w, cx| f(s2.clone(), w, cx))
                        })
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        };
        let fav_rows = list("star", fav.clone(), "oxyn-ws-fav");
        let pro_rows = list("folder", pro.clone(), "oxyn-ws-proj");
        let empty = fav.is_empty() && pro.is_empty();
        sidebar_frame(t, if r { RAIL_W } else { 248. })
            .id("oxyn-wsbar")
            .role(Role::Navigation)
            .aria_label("Workspace")
            .relative()
            .child(
                div()
                    .id("oxyn-ws-switch")
                    .role(Role::Button)
                    .aria_label(SharedString::from(format!(
                        "Switch workspace, current {wn}"
                    )))
                    .flex()
                    .items_center()
                    .when(r, |d| d.justify_center())
                    .gap(px(9.))
                    .m(px(8.))
                    .p(px(6.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
                    .child(logo(t, &wn, 28.))
                    .when(!r, |d| d.child(two_line(t, &wn, &wp).flex_1()).child("⌄"))
                    .when_some(menu, |d, f| d.on_click(move |_, w, cx| f(0, w, cx))),
            )
            .when(self.menu_open, |d| {
                d.child(
                    div()
                        .id("oxyn-sh-738")
                        .role(Role::Menu)
                        .mx(px(8.))
                        .p(px(4.))
                        .rounded(px(10.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .children(self.workspaces.iter().enumerate().map(|(i, (n, p))| {
                            let f = self.on_switch.clone();
                            div()
                                .id(("oxyn-ws-mi", i))
                                .role(Role::MenuItemRadio)
                                .aria_label(SharedString::from(n.clone()))
                                .flex()
                                .items_center()
                                .gap(px(9.))
                                .p(px(6.))
                                .rounded(px(7.))
                                .cursor_pointer()
                                .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
                                .child(logo(t, n, 22.))
                                .child(two_line(t, n, p).flex_1())
                                .when(i == self.current, |d| d.child("✓"))
                                .when_some(f, move |d, f| d.on_click(move |_, w, cx| f(i, w, cx)))
                        })),
                )
            })
            .when(!r, |d| {
                d.child(
                    div()
                        .mx(px(10.))
                        .mb(px(8.))
                        .px(px(9.))
                        .h(px(30.))
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .rounded(px(7.))
                        .bg(c(t, ColorRole::SurfaceBase))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .text_size(px(12.5))
                        .text_color(c(
                            t,
                            if self.query.is_empty() {
                                ColorRole::FgSubtle
                            } else {
                                ColorRole::FgDefault
                            },
                        ))
                        .child("⌕")
                        .child(if self.query.is_empty() {
                            "Search favorites and projects".to_string()
                        } else {
                            self.query.clone()
                        }),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(1.))
                    .px(px(if r { 10. } else { 8. }))
                    .when(!r, |d| d.child(heading(t, "Favorites")))
                    .children(fav_rows)
                    .when(!r, |d| d.child(heading(t, "Projects")))
                    .children(pro_rows)
                    .when(empty && !r, |d| {
                        d.child(
                            div()
                                .p(px(10.))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(format!("Nothing matches “{}”", self.query)),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .p(px(8.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .when(!r, |d| {
                        let pct = (self.storage * 100.).round();
                        d.child(
                            div()
                                .id("oxyn-sh-828")
                                .role(Role::Meter)
                                .aria_label(SharedString::from(format!("Storage {pct}%")))
                                .flex()
                                .flex_col()
                                .gap(px(5.))
                                .p(px(6.))
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_size(px(12.))
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child("Storage")
                                        .child(format!("{pct}%")),
                                )
                                .child(
                                    div()
                                        .h(px(5.))
                                        .rounded(px(3.))
                                        .bg(c(t, ColorRole::BorderSubtle))
                                        .child(
                                            div()
                                                .h_full()
                                                .w(gpui::relative(self.storage.clamp(0., 1.)))
                                                .rounded(px(3.))
                                                .bg(c(t, ColorRole::FgDefault)),
                                        ),
                                ),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .when(r, |d| d.justify_center())
                            .gap(px(9.))
                            .p(px(4.))
                            .child(avatar(t, "Emma Collins"))
                            .when(!r, |d| d.child(two_line(t, "Emma Collins", "● Active"))),
                    ),
            )
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
