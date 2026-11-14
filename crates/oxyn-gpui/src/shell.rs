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

fn nav_rows(
    t: &'static Theme,
    sections: &[NavSection],
    active: &str,
    open: &[String],
    rail: bool,
    on_select: Option<OnId>,
) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();
    for (si, s) in sections.iter().enumerate() {
        if !s.title.is_empty() && !rail {
            out.push(heading(t, &s.title).into_any_element());
        } else if !s.title.is_empty() && si > 0 {
            out.push(
                div()
                    .h(px(1.))
                    .mx(px(10.))
                    .my(px(6.))
                    .bg(c(t, ColorRole::BorderSubtle))
                    .into_any_element(),
            );
        }
        for it in &s.items {
            let cur = it.id == active || (rail && it.children.iter().any(|ch| ch.id == active));
            let f = on_select.clone();
            let id = it.id.clone();
            out.push(
                row(
                    t,
                    SharedString::from(format!("oxyn-nav-{}", it.id)),
                    &it.icon,
                    &it.label,
                    it.count,
                    cur,
                    rail,
                )
                .when_some(f, move |d, f| {
                    d.on_click(move |_, w, cx| f(id.clone(), w, cx))
                })
                .into_any_element(),
            );
            if !rail && open.contains(&it.id) {
                for ch in &it.children {
                    let f = on_select.clone();
                    let id = ch.id.clone();
                    out.push(
                        row(
                            t,
                            SharedString::from(format!("oxyn-nav-{}", ch.id)),
                            "",
                            &ch.label,
                            0,
                            ch.id == active,
                            false,
                        )
                        .pl(px(34.))
                        .when_some(f, move |d, f| {
                            d.on_click(move |_, w, cx| f(id.clone(), w, cx))
                        })
                        .into_any_element(),
                    );
                }
            }
        }
    }
    out
}

// ─────────────────────────────────────────── Sidebar ──

/// Sidebar: brand, sections with nested groups, settings and the account row; `rail` draws the
/// 64 px icon rail. `on_select` gets a row id (a group id toggles it), `on_fold` the fold button.
#[derive(IntoElement)]
pub struct SidebarEl {
    sections: Vec<NavSection>,
    active: String,
    open: Vec<String>,
    rail: bool,
    width: f32,
    brand: String,
    user: String,
    on_select: Option<OnId>,
    on_fold: Option<OnIndex>,
    theme: &'static Theme,
}

impl SidebarEl {
    /// From sections and the active id.
    pub fn new(sections: Vec<NavSection>, active: impl Into<String>) -> Self {
        Self {
            sections,
            active: active.into(),
            open: Vec::new(),
            rail: false,
            width: 248.,
            brand: "Northline".into(),
            user: "Emma Collins".into(),
            on_select: None,
            on_fold: None,
            theme: &Theme::DARK,
        }
    }
    /// Open groups.
    pub fn open(mut self, ids: Vec<String>) -> Self {
        self.open = ids;
        self
    }
    /// Folded to the icon rail.
    pub fn rail(mut self, on: bool) -> Self {
        self.rail = on;
        self
    }
    /// Open width (px).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    /// Brand and account names.
    pub fn names(mut self, brand: impl Into<String>, user: impl Into<String>) -> Self {
        self.brand = brand.into();
        self.user = user.into();
        self
    }
    on_id!(on_select, "A row was pressed (its id).");
    on_index!(on_fold, "The fold button was pressed (0).");
    themed!();
}

impl RenderOnce for SidebarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.rail;
        let fold = self.on_fold.clone();
        sidebar_frame(t, if r { RAIL_W } else { self.width })
            .id("oxyn-sidebar")
            .role(Role::Navigation)
            .aria_label("Main")
            .child(
                div()
                    .flex()
                    .when(r, |d| d.flex_col())
                    .items_center()
                    .gap(px(9.))
                    .p(px(12.))
                    .child(logo(t, &self.brand, 28.))
                    .when(!r, |d| {
                        d.child(two_line(t, &self.brand, "Product design").flex_1())
                    })
                    .child(
                        div()
                            .id("oxyn-sidebar-fold")
                            .role(Role::Button)
                            .aria_label(if r { "Expand sidebar" } else { "Fold sidebar" })
                            .px(px(6.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(if r { "›" } else { "‹" })
                            .when_some(fold, |d, f| d.on_click(move |_, w, cx| f(0, w, cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(1.))
                    .px(px(if r { 10. } else { 8. }))
                    .children(nav_rows(
                        t,
                        &self.sections,
                        &self.active,
                        &self.open,
                        r,
                        self.on_select.clone(),
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .p(px(8.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(row(
                        t,
                        "oxyn-sidebar-settings",
                        "cog",
                        "Settings",
                        0,
                        false,
                        r,
                    ))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .when(r, |d| d.justify_center())
                            .gap(px(9.))
                            .p(px(4.))
                            .child(avatar(t, &self.user))
                            .when(!r, |d| d.child(two_line(t, &self.user, "Product designer"))),
                    ),
            )
    }
}

// ─────────────────────────────────────────── Rail shell ──

/// Sidebar rail shell: a [`SidebarEl`] sized by the host's [`Rail`], a separator the host drags
/// (`Rail::drag`) or nudges with arrows (`Rail::nudge`), a breadcrumb and the content.
#[derive(IntoElement)]
pub struct RailShellEl {
    rail: Rail,
    sidebar: SidebarEl,
    crumbs: Vec<String>,
    content: Option<AnyElement>,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl RailShellEl {
    /// From the rail model and the sidebar (its rail flag and width follow the model).
    pub fn new(rail: Rail, sidebar: SidebarEl) -> Self {
        Self {
            rail,
            sidebar,
            crumbs: Vec::new(),
            content: None,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    /// Breadcrumb trail (`oxyn_core::shell::trail`).
    pub fn crumbs(mut self, c: Vec<String>) -> Self {
        self.crumbs = c;
        self
    }
    /// Content.
    pub fn content(mut self, e: impl IntoElement) -> Self {
        self.content = Some(e.into_any_element());
        self
    }
    on_index!(
        on_toggle,
        "The fold button or a double-click on the separator (0)."
    );
    themed!();
}

impl RenderOnce for RailShellEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.rail;
        let tog = self.on_toggle.clone();
        let tog2 = self.on_toggle.clone();
        let n = self.crumbs.len();
        div()
            .flex()
            .size_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceBase))
            .overflow_hidden()
            .text_color(c(t, ColorRole::FgDefault))
            .child(self.sidebar.rail(r.collapsed).width(r.width).theme(t))
            .child(
                div()
                    .id("oxyn-rail-sep")
                    .role(Role::Splitter)
                    .aria_label(SharedString::from(if r.collapsed {
                        "Resize sidebar, folded to icon rail".to_string()
                    } else {
                        format!("Resize sidebar, {} pixels", r.value())
                    }))
                    .w(px(5.))
                    .h_full()
                    .cursor_col_resize()
                    .hover(|s| s.bg(c(t, ColorRole::Focus)))
                    .when_some(tog2, |d, f| {
                        d.on_click(move |e, w, cx| {
                            if e.click_count() >= 2 {
                                f(0, w, cx)
                            }
                        })
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .h(px(48.))
                            .px(px(12.))
                            .border_b_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .child(
                                div()
                                    .id("oxyn-rail-fold")
                                    .role(Role::Button)
                                    .aria_label(if r.collapsed {
                                        "Expand sidebar"
                                    } else {
                                        "Fold sidebar"
                                    })
                                    .px(px(6.))
                                    .cursor_pointer()
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child("☰")
                                    .when_some(tog, |d, f| d.on_click(move |_, w, cx| f(0, w, cx))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child("Northline")
                                    .children(self.crumbs.iter().enumerate().map(|(i, s)| {
                                        div()
                                            .flex()
                                            .gap(px(6.))
                                            .child(
                                                div()
                                                    .text_color(c(t, ColorRole::FgSubtle))
                                                    .child("/"),
                                            )
                                            .child(
                                                div()
                                                    .when(i + 1 == n, |d| {
                                                        d.text_color(c(t, ColorRole::FgDefault))
                                                            .font_weight(FontWeight::MEDIUM)
                                                    })
                                                    .child(s.clone()),
                                            )
                                    })),
                            )
                            .child(div().flex_1())
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(if r.collapsed {
                                        format!("rail · {}px", RAIL_W as u32)
                                    } else {
                                        format!("{}px", r.value())
                                    }),
                            ),
                    )
                    .child(div().flex_1().p(px(16.)).children(self.content)),
            )
    }
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

// ─────────────────────────────────────────── Inbox ──

/// Inbox sidebar: compose button, folders with counts, labels, the message list with star /
/// archive / move actions and an undo for the last move. Actions: `on_folder` (folder id),
/// `on_star`, `on_archive`, `on_read` (message id), `on_move` (`"<msg>|<folder>"`), `on_undo`,
/// `on_compose` (0).
#[derive(IntoElement)]
pub struct InboxEl {
    model: Inbox,
    rail: bool,
    menu_for: Option<String>,
    on_folder: Option<OnId>,
    on_star: Option<OnId>,
    on_archive: Option<OnId>,
    on_read: Option<OnId>,
    on_move: Option<OnId>,
    on_undo: Option<OnIndex>,
    on_compose: Option<OnIndex>,
    theme: &'static Theme,
}

impl InboxEl {
    /// From the model.
    pub fn new(model: Inbox) -> Self {
        Self {
            model,
            rail: false,
            menu_for: None,
            on_folder: None,
            on_star: None,
            on_archive: None,
            on_read: None,
            on_move: None,
            on_undo: None,
            on_compose: None,
            theme: &Theme::DARK,
        }
    }
    /// Folded folders.
    pub fn rail(mut self, on: bool) -> Self {
        self.rail = on;
        self
    }
    /// Draw the Move menu for this message.
    pub fn menu_for(mut self, id: Option<String>) -> Self {
        self.menu_for = id;
        self
    }
    on_id!(on_folder, "A folder was opened.");
    on_id!(on_star, "Star toggled on a message.");
    on_id!(on_archive, "A message was archived.");
    on_id!(on_read, "A message was opened.");
    on_id!(on_move, "Move: `message|folder`.");
    on_index!(on_undo, "Undo the last move (0).");
    on_index!(on_compose, "Compose pressed (0).");
    themed!();
}

impl RenderOnce for InboxEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.rail;
        let m = self.model.clone();
        let label = m
            .folders
            .iter()
            .find(|f| f.id == m.open)
            .map(|f| f.label.clone())
            .unwrap_or_default();
        let shown: Vec<_> = m.visible().into_iter().cloned().collect();
        let unread = shown.iter().filter(|x| x.unread).count();
        let comp = self.on_compose.clone();
        let undo = self.on_undo.clone();
        let small_btn = |id: SharedString, txt: &'static str, lbl: &'static str, on: bool| {
            div()
                .id(id)
                .role(Role::Button)
                .aria_label(lbl)
                .px(px(6.))
                .rounded(px(5.))
                .cursor_pointer()
                .text_color(c(
                    t,
                    if on {
                        ColorRole::StatusWarningFg
                    } else {
                        ColorRole::FgMuted
                    },
                ))
                .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
                .child(txt)
        };
        div()
            .flex()
            .size_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceBase))
            .overflow_hidden()
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                sidebar_frame(t, if r { RAIL_W } else { 220. })
                    .id("oxyn-inbox-folders")
                    .role(Role::Navigation)
                    .aria_label("Mail folders")
                    .child(
                        div().p(px(10.)).child(
                            div()
                                .id("oxyn-inbox-compose")
                                .role(Role::Button)
                                .aria_label("Compose")
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap(px(7.))
                                .h(px(34.))
                                .rounded(px(9.))
                                .cursor_pointer()
                                .bg(c(t, ColorRole::FgDefault))
                                .text_color(c(t, ColorRole::BgBase))
                                .child("✎")
                                .when(!r, |d| d.child("Compose"))
                                .when_some(comp, |d, f| d.on_click(move |_, w, cx| f(0, w, cx))),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap(px(1.))
                            .px(px(if r { 10. } else { 8. }))
                            .children(m.folders.iter().map(|f| {
                                let n = m.count(&f.id) as u32;
                                let cb = self.on_folder.clone();
                                let id = f.id.clone();
                                row(
                                    t,
                                    SharedString::from(format!("oxyn-ib-f-{}", f.id)),
                                    &f.icon,
                                    &f.label,
                                    if r && f.id != "inbox" { 0 } else { n },
                                    m.open == f.id,
                                    r,
                                )
                                .when_some(cb, move |d, cb| {
                                    d.on_click(move |_, w, cx| cb(id.clone(), w, cx))
                                })
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(16.))
                            .py(px(12.))
                            .border_b_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(label.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child(format!("{} messages · {unread} unread", shown.len())),
                            )
                            .child(div().flex_1())
                            .when(m.last_move.is_some(), |d| {
                                d.child(
                                    div()
                                        .id("oxyn-ib-undo")
                                        .role(Role::Button)
                                        .px(px(8.))
                                        .rounded(px(6.))
                                        .border_1()
                                        .border_color(c(t, ColorRole::BorderDefault))
                                        .cursor_pointer()
                                        .child("↶ Undo")
                                        .when_some(undo, |d, f| {
                                            d.on_click(move |_, w, cx| f(0, w, cx))
                                        }),
                                )
                            }),
                    )
                    .children(shown.iter().map(|x| {
                        let rd = self.on_read.clone();
                        let st = self.on_star.clone();
                        let ar = self.on_archive.clone();
                        let mv = self.on_move.clone();
                        let (id1, id2, id3) = (x.id.clone(), x.id.clone(), x.id.clone());
                        let menu = self.menu_for.as_deref() == Some(x.id.as_str());
                        div()
                            .id(SharedString::from(format!("oxyn-ib-m-{}", x.id)))
                            .role(Role::ListItem)
                            .aria_label(SharedString::from(format!(
                                "{}, {}, {}{}",
                                x.from,
                                x.subject,
                                x.when,
                                if x.unread { ", unread" } else { "" }
                            )))
                            .flex()
                            .gap(px(10.))
                            .px(px(16.))
                            .py(px(10.))
                            .border_b_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
                            .when_some(rd, move |d, f| {
                                d.on_click(move |_, w, cx| f(id1.clone(), w, cx))
                            })
                            .child(avatar(t, &x.from))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w_0()
                                    .child(
                                        div()
                                            .flex()
                                            .gap(px(8.))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .when(x.unread, |d| {
                                                        d.font_weight(FontWeight::SEMIBOLD)
                                                    })
                                                    .child(x.from.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.5))
                                                    .text_color(c(t, ColorRole::FgSubtle))
                                                    .child(x.when.clone()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .when(x.unread, |d| d.font_weight(FontWeight::SEMIBOLD))
                                            .child(x.subject.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.5))
                                            .text_color(c(t, ColorRole::FgMuted))
                                            .child(x.preview.clone()),
                                    )
                                    .when(menu, |d| {
                                        d.child(
                                            div()
                                                .id("oxyn-sh-1137")
                                                .role(Role::Menu)
                                                .flex()
                                                .gap(px(4.))
                                                .mt(px(6.))
                                                .children(
                                                    m.folders
                                                        .iter()
                                                        .filter(|f| {
                                                            f.id != "starred" && f.id != x.folder
                                                        })
                                                        .map(|f| {
                                                            let mv = mv.clone();
                                                            let key = format!("{}|{}", x.id, f.id);
                                                            div()
                                                                .id(SharedString::from(format!(
                                                                    "oxyn-ib-mv-{key}"
                                                                )))
                                                                .role(Role::MenuItem)
                                                                .px(px(7.))
                                                                .rounded(px(6.))
                                                                .border_1()
                                                                .border_color(c(
                                                                    t,
                                                                    ColorRole::BorderSubtle,
                                                                ))
                                                                .cursor_pointer()
                                                                .text_size(px(12.))
                                                                .child(f.label.clone())
                                                                .when_some(mv, move |d, f| {
                                                                    d.on_click(move |_, w, cx| {
                                                                        f(key.clone(), w, cx)
                                                                    })
                                                                })
                                                        }),
                                                ),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(2.))
                                    .child(
                                        small_btn(
                                            SharedString::from(format!("oxyn-ib-s-{}", x.id)),
                                            "★",
                                            if x.starred { "Remove star" } else { "Star" },
                                            x.starred,
                                        )
                                        .when_some(
                                            st,
                                            move |d, f| {
                                                d.on_click(move |_, w, cx| f(id2.clone(), w, cx))
                                            },
                                        ),
                                    )
                                    .child(
                                        small_btn(
                                            SharedString::from(format!("oxyn-ib-a-{}", x.id)),
                                            "▥",
                                            "Archive",
                                            false,
                                        )
                                        .when_some(
                                            ar,
                                            move |d, f| {
                                                d.on_click(move |_, w, cx| f(id3.clone(), w, cx))
                                            },
                                        ),
                                    ),
                            )
                    }))
                    .when(shown.is_empty(), |d| {
                        d.child(
                            div()
                                .p(px(16.))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(format!("Nothing in {label}")),
                        )
                    }),
            )
    }
}

// ─────────────────────────────────────────── App shell ──

/// App shell: sidebar, header (title, actions) and content. `inset` floats the content on a
/// raised card inside the sidebar's plane; flat keeps one plane split by hairlines.
#[derive(IntoElement)]
pub struct AppShellEl {
    inset: bool,
    title: String,
    sidebar: Option<AnyElement>,
    actions: Vec<AnyElement>,
    content: Option<AnyElement>,
    theme: &'static Theme,
}

impl AppShellEl {
    /// A flat shell titled `title`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            inset: false,
            title: title.into(),
            sidebar: None,
            actions: Vec::new(),
            content: None,
            theme: &Theme::DARK,
        }
    }
    /// Inset variant.
    pub fn inset(mut self, on: bool) -> Self {
        self.inset = on;
        self
    }
    /// Sidebar.
    pub fn sidebar(mut self, e: impl IntoElement) -> Self {
        self.sidebar = Some(e.into_any_element());
        self
    }
    /// Header action.
    pub fn action(mut self, e: impl IntoElement) -> Self {
        self.actions.push(e.into_any_element());
        self
    }
    /// Content.
    pub fn content(mut self, e: impl IntoElement) -> Self {
        self.content = Some(e.into_any_element());
        self
    }
    themed!();
}

impl RenderOnce for AppShellEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let inset = self.inset;
        div()
            .flex()
            .size_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .overflow_hidden()
            .bg(c(
                t,
                if inset {
                    ColorRole::SurfaceSunken
                } else {
                    ColorRole::SurfaceBase
                },
            ))
            .text_color(c(t, ColorRole::FgDefault))
            .children(self.sidebar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .when(inset, |d| {
                        d.m(px(8.))
                            .ml(px(0.))
                            .rounded(px(10.))
                            .bg(c(t, ColorRole::SurfaceBase))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .overflow_hidden()
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .h(px(50.))
                            .px(px(18.))
                            .border_b_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.title.clone()),
                            )
                            .children(self.actions),
                    )
                    .child(div().flex_1().p(px(18.)).children(self.content)),
            )
    }
}

// ─────────────────────────────────────────── AI side panel ──

/// AI side panel beside a page: the page blocks (a just-applied block tinted), and when `open`
/// the panel with the question, the answer, the word-diff card (Apply / Discard / Undo) and
/// suggestions. Actions (index): 0 apply, 1 discard, 2 undo, 3 close, 4 reopen; `on_ask` gets a
/// suggestion.
#[derive(IntoElement)]
pub struct AiPanelEl {
    model: PageEdit,
    open: bool,
    question: Option<String>,
    answer: String,
    tinted: Option<String>,
    on_action: Option<OnIndex>,
    on_ask: Option<OnId>,
    theme: &'static Theme,
}

impl AiPanelEl {
    /// From the page model.
    pub fn new(model: PageEdit) -> Self {
        Self {
            model,
            open: true,
            question: None,
            answer: String::new(),
            tinted: None,
            on_action: None,
            on_ask: None,
            theme: &Theme::DARK,
        }
    }
    /// Panel open.
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        self
    }
    /// The question and the answer so far.
    pub fn thread(mut self, q: impl Into<String>, a: impl Into<String>) -> Self {
        self.question = Some(q.into());
        self.answer = a.into();
        self
    }
    /// A block to tint (just applied).
    pub fn tinted(mut self, id: Option<String>) -> Self {
        self.tinted = id;
        self
    }
    on_index!(on_action, "0 apply, 1 discard, 2 undo, 3 close, 4 reopen.");
    on_id!(on_ask, "A suggestion was pressed.");
    themed!();
}

impl RenderOnce for AiPanelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = self.model.clone();
        let act = self.on_action.clone();
        let b = |id: &'static str, label: &'static str, i: usize, solid: bool| {
            let f = act.clone();
            div()
                .id(id)
                .role(Role::Button)
                .aria_label(label)
                .px(px(10.))
                .py(px(4.))
                .rounded(px(7.))
                .cursor_pointer()
                .text_size(px(12.5))
                .when(solid, |d| {
                    d.bg(c(t, ColorRole::FgDefault))
                        .text_color(c(t, ColorRole::BgBase))
                })
                .when(!solid, |d| {
                    d.border_1().border_color(c(t, ColorRole::BorderDefault))
                })
                .child(label)
                .when_some(f, move |d, f| d.on_click(move |_, w, cx| f(i, w, cx)))
        };
        let page = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .p(px(24.))
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Team workspaces"),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgMuted))
                    .mb(px(10.))
                    .child("Product spec · Maya Chen · Edited Oct 18"),
            )
            .children(p.blocks.iter().map(|bl| {
                let tint = self.tinted.as_deref() == Some(bl.id.as_str());
                div()
                    .p(px(8.))
                    .rounded(px(8.))
                    .when(tint, |d| d.bg(c(t, ColorRole::StatusSuccessBg)))
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(bl.label.to_uppercase()),
                    )
                    .child(div().text_size(px(14.)).child(bl.text.clone()))
            }))
            .when(!self.open, |d| {
                d.child(b("oxyn-ap-reopen", "Assistant", 4, true))
            });
        let diff = p.diff();
        let panel = div()
            .id("oxyn-aipanel")
            .role(Role::Complementary)
            .aria_label("Assistant")
            .flex()
            .flex_col()
            .flex_none()
            .w(px(320.))
            .border_l_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceRaised))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .p(px(12.))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(div().child("✦"))
                    .child(two_line(t, "Assistant", "Claude Sonnet").flex_1())
                    .child(b("oxyn-ap-close", "Close", 3, false)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(10.))
                    .p(px(14.))
                    .when_some(self.question.clone(), |d, q| {
                        d.child(
                            div()
                                .self_end()
                                .px(px(11.))
                                .py(px(7.))
                                .rounded(px(12.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .child(q),
                        )
                        .child(div().child(self.answer.clone()))
                    })
                    .when(self.question.is_none(), |d| {
                        d.child(
                            div()
                                .text_color(c(t, ColorRole::FgMuted))
                                .child("Assistant reads this page and proposes edits. Nothing changes until you apply it."),
                        )
                    })
                    .when(p.phase != EditPhase::None, |d| {
                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .rounded(px(10.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderDefault))
                                .bg(c(t, ColorRole::SurfaceBase))
                                .when(p.phase == EditPhase::Discarded, |d| d.opacity(0.6))
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(4.))
                                        .p(px(10.))
                                        .text_size(px(12.5))
                                        .children(diff.into_iter().map(|(op, w)| {
                                            div()
                                                .px(px(2.))
                                                .rounded(px(3.))
                                                .when(op == DiffOp::Del, |d| {
                                                    d.text_color(c(t, ColorRole::StatusDangerFg))
                                                        .bg(c(t, ColorRole::StatusDangerBg))
                                                        .line_through()
                                                })
                                                .when(op == DiffOp::Ins, |d| {
                                                    d.text_color(c(t, ColorRole::StatusSuccessFg))
                                                        .bg(c(t, ColorRole::StatusSuccessBg))
                                                })
                                                .child(w)
                                        })),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.))
                                        .p(px(8.))
                                        .border_t_1()
                                        .border_color(c(t, ColorRole::BorderSubtle))
                                        .when(p.phase == EditPhase::Proposed, |d| {
                                            d.child(b("oxyn-ap-apply", "Apply", 0, true))
                                                .child(b("oxyn-ap-discard", "Discard", 1, false))
                                        })
                                        .when(p.phase == EditPhase::Applied, |d| {
                                            d.child("✓ Applied to the page")
                                                .child(b("oxyn-ap-undo", "Undo", 2, false))
                                        })
                                        .when(p.phase == EditPhase::Discarded, |d| d.child("Discarded"))
                                        .when(p.phase == EditPhase::Undone, |d| d.child("Undone")),
                                ),
                        )
                    })
                    .when(
                        self.question.is_none()
                            || matches!(p.phase, EditPhase::Applied | EditPhase::Discarded | EditPhase::Undone),
                        |d| {
                            d.children(
                                ["Summarize this page", "Tighten the intro", "Add an annual price column"]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, s)| {
                                        let f = self.on_ask.clone();
                                        div()
                                            .id(("oxyn-ap-sug", i))
                                            .role(Role::Button)
                                            .px(px(10.))
                                            .py(px(5.))
                                            .rounded(px(7.))
                                            .border_1()
                                            .border_color(c(t, ColorRole::BorderSubtle))
                                            .cursor_pointer()
                                            .child(s)
                                            .when_some(f, move |d, f| {
                                                d.on_click(move |_, w, cx| f(s.to_string(), w, cx))
                                            })
                                    }),
                            )
                        },
                    ),
            );
        div()
            .flex()
            .size_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceBase))
            .overflow_hidden()
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(page)
            .when(self.open, |d| d.child(panel))
    }
}

// ─────────────────────────────────────────── Terminal tabs ──

/// Terminal tabs: the strip (activity dots, exit codes, close), a + button and the active tab's
/// output. `on_select` / `on_close` get a tab id (as usize); `on_add` gets 0.
#[derive(IntoElement)]
pub struct TermTabsEl {
    model: TermTabs,
    on_select: Option<OnIndex>,
    on_close: Option<OnIndex>,
    on_add: Option<OnIndex>,
}

impl TermTabsEl {
    /// From the model.
    pub fn new(model: TermTabs) -> Self {
        Self {
            model,
            on_select: None,
            on_close: None,
            on_add: None,
        }
    }
    on_index!(on_select, "A tab was selected (id).");
    on_index!(on_close, "A tab was closed (id).");
    on_index!(on_add, "New terminal (0).");
}

impl RenderOnce for TermTabsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let bg = gpui::rgb(0x0d0f12);
        let strip = gpui::rgb(0x15181d);
        let fg = gpui::rgb(0xd6dae0);
        let dim = gpui::rgb(0x8b929c);
        let tt = self.model;
        let active = tt.tabs.iter().find(|x| x.id == tt.active).cloned();
        let add = self.on_add.clone();
        div()
            .flex()
            .flex_col()
            .size_full()
            .rounded(px(12.))
            .overflow_hidden()
            .bg(bg)
            .text_color(fg)
            .text_size(px(12.5))
            .child(
                div()
                    .id("oxyn-term-strip")
                    .role(Role::TabList)
                    .aria_label("Terminals")
                    .flex()
                    .items_end()
                    .gap(px(2.))
                    .px(px(6.))
                    .pt(px(6.))
                    .bg(strip)
                    .children(tt.tabs.iter().map(|tab| {
                        let on = tab.id == tt.active;
                        let id = tab.id as usize;
                        let s = self.on_select.clone();
                        let x = self.on_close.clone();
                        div()
                            .id(("oxyn-term-tab", id))
                            .role(Role::Tab)
                            .aria_selected(on)
                            .aria_label(SharedString::from(format!(
                                "{}{}",
                                tab.title,
                                if tab.activity { ", new output" } else { "" }
                            )))
                            .flex()
                            .items_center()
                            .gap(px(7.))
                            .h(px(30.))
                            .pl(px(12.))
                            .pr(px(6.))
                            .rounded_t(px(8.))
                            .cursor_pointer()
                            .text_color(if on { fg } else { dim })
                            .when(on, |d| d.bg(bg))
                            .child("›_")
                            .child(tab.title.clone())
                            .when(tab.activity, |d| {
                                d.child(div().size(px(6.)).rounded_full().bg(gpui::rgb(0x58a6ff)))
                            })
                            .when_some(tab.exit, |d, code| {
                                d.child(
                                    div()
                                        .px(px(5.))
                                        .rounded(px(4.))
                                        .text_size(px(10.))
                                        .text_color(if code == 0 {
                                            gpui::rgb(0x56d364)
                                        } else {
                                            gpui::rgb(0xff7b72)
                                        })
                                        .child(code.to_string()),
                                )
                            })
                            .child(
                                div()
                                    .id(("oxyn-term-x", id))
                                    .aria_label(SharedString::from(format!("Close {}", tab.title)))
                                    .px(px(4.))
                                    .child("×")
                                    .when_some(x, move |d, f| {
                                        d.on_click(move |_, w, cx| f(id, w, cx))
                                    }),
                            )
                            .when_some(s, move |d, f| d.on_click(move |_, w, cx| f(id, w, cx)))
                    }))
                    .child(
                        div()
                            .id("oxyn-term-add")
                            .role(Role::Button)
                            .aria_label("New terminal")
                            .px(px(8.))
                            .pb(px(6.))
                            .cursor_pointer()
                            .text_color(dim)
                            .child("+")
                            .when_some(add, |d, f| d.on_click(move |_, w, cx| f(0, w, cx))),
                    ),
            )
            .child(
                div()
                    .id("oxyn-sh-1718")
                    .role(Role::TabPanel)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p(px(12.))
                    .font_family("monospace")
                    .when_some(active, |d, a| {
                        d.children(a.lines.into_iter().map(move |l| {
                            div()
                                .text_color(if l.starts_with('$') { fg } else { dim })
                                .child(l)
                        }))
                        .child(div().child("$ ▍"))
                    }),
            )
    }
}

// ─────────────────────────────────────────── Sidebar Nav ──

/// Sidebar Nav (Beautiful UI, MIT, © 2026 Shane Levine): grouped links, collapsible group
/// headings and badges; the active link is drawn raised (no sliding indicator). `on_select` gets a
/// link id, `on_group` a group title.
#[derive(IntoElement)]
pub struct SidebarNavEl {
    sections: Vec<NavSection>,
    active: String,
    closed: Vec<String>,
    compact: bool,
    on_select: Option<OnId>,
    on_group: Option<OnId>,
    theme: &'static Theme,
}

impl SidebarNavEl {
    /// From sections and the active id.
    pub fn new(sections: Vec<NavSection>, active: impl Into<String>) -> Self {
        Self {
            sections,
            active: active.into(),
            closed: Vec::new(),
            compact: false,
            on_select: None,
            on_group: None,
            theme: &Theme::DARK,
        }
    }
    /// Collapsed group titles.
    pub fn closed(mut self, c: Vec<String>) -> Self {
        self.closed = c;
        self
    }
    /// Compact density.
    pub fn compact(mut self, on: bool) -> Self {
        self.compact = on;
        self
    }
    on_id!(on_select, "A link was pressed (its id).");
    on_id!(on_group, "A group heading was pressed (its title).");
    themed!();
}

impl RenderOnce for SidebarNavEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let rh = if self.compact { 30. } else { 36. };
        div()
            .id("oxyn-snav")
            .role(Role::Navigation)
            .aria_label("Sidebar")
            .flex()
            .flex_col()
            .gap(px(8.))
            .w(px(240.))
            .p(px(8.))
            .rounded(px(12.))
            .bg(c(t, ColorRole::SurfaceSunken))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .children(self.sections.iter().enumerate().map(|(gi, s)| {
                let hidden = self.closed.contains(&s.title);
                let g = self.on_group.clone();
                let title = s.title.clone();
                div()
                    .flex()
                    .flex_col()
                    .when(!s.title.is_empty(), |d| {
                        d.child(
                            div()
                                .id(("oxyn-snav-g", gi))
                                .role(Role::Button)
                                .aria_expanded(!hidden)
                                .flex()
                                .justify_between()
                                .h(px(28.))
                                .px(px(10.))
                                .items_center()
                                .cursor_pointer()
                                .text_size(px(11.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(s.title.to_uppercase())
                                .child(if hidden { "›" } else { "⌄" })
                                .when_some(g, move |d, f| {
                                    d.on_click(move |_, w, cx| f(title.clone(), w, cx))
                                }),
                        )
                    })
                    .when(!hidden, |d| {
                        d.children(s.items.iter().map(|it| {
                            let f = self.on_select.clone();
                            let id = it.id.clone();
                            row(
                                t,
                                SharedString::from(format!("oxyn-snav-{}", it.id)),
                                &it.icon,
                                &it.label,
                                it.count,
                                it.id == self.active,
                                false,
                            )
                            .h(px(rh))
                            .rounded(px(8.))
                            .when_some(f, move |d, f| {
                                d.on_click(move |_, w, cx| f(id.clone(), w, cx))
                            })
                        }))
                    })
            }))
    }
}

// ─────────────────────────────────────────── Chat shell ──

/// The frame of an AI Chat screen: an optional app rail, a thread list, the main column and an
/// optional side panel. `on_thread` gets a thread index.
#[derive(IntoElement)]
pub struct ChatShellEl {
    rail: bool,
    threads: Vec<(String, String)>,
    active: usize,
    title: String,
    main: Option<AnyElement>,
    side: Option<AnyElement>,
    on_thread: Option<OnIndex>,
    theme: &'static Theme,
}

impl ChatShellEl {
    /// An empty frame.
    pub fn new() -> Self {
        Self {
            rail: false,
            threads: Vec::new(),
            active: 0,
            title: String::new(),
            main: None,
            side: None,
            on_thread: None,
            theme: &Theme::DARK,
        }
    }
    /// Show the app rail (Double sidebar).
    pub fn rail(mut self, on: bool) -> Self {
        self.rail = on;
        self
    }
    /// Thread list (title, meta) and the active index.
    pub fn threads(mut self, th: Vec<(String, String)>, active: usize) -> Self {
        self.threads = th;
        self.active = active;
        self
    }
    /// Header title.
    pub fn title(mut self, s: impl Into<String>) -> Self {
        self.title = s.into();
        self
    }
    /// Main column.
    pub fn main(mut self, e: impl IntoElement) -> Self {
        self.main = Some(e.into_any_element());
        self
    }
    /// Side panel.
    pub fn side(mut self, e: impl IntoElement) -> Self {
        self.side = Some(e.into_any_element());
        self
    }
    on_index!(on_thread, "A thread was picked (index).");
    themed!();
}

impl Default for ChatShellEl {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ChatShellEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .size_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceBase))
            .overflow_hidden()
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when(self.rail, |d| {
                d.child(
                    div()
                        .id("oxyn-sh-1934")
                        .role(Role::Navigation)
                        .aria_label("Apps")
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(10.))
                        .w(px(56.))
                        .py(px(12.))
                        .bg(c(t, ColorRole::SurfaceSunken))
                        .border_r_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .child(logo(t, "A", 28.))
                        .children(
                            ["thread", "agents", "folder", "cog"]
                                .into_iter()
                                .enumerate()
                                .map(|(i, ic)| {
                                    div()
                                        .size(px(34.))
                                        .rounded(px(9.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .when(i == 0, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                                        .child(glyph(ic))
                                }),
                        ),
                )
            })
            .when(!self.threads.is_empty(), |d| {
                d.child(
                    div()
                        .id("oxyn-sh-1966")
                        .role(Role::Navigation)
                        .aria_label("Threads")
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .w(px(228.))
                        .p(px(8.))
                        .bg(c(t, ColorRole::SurfaceSunken))
                        .border_r_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .child(
                            div()
                                .h(px(32.))
                                .px(px(10.))
                                .mb(px(6.))
                                .flex()
                                .items_center()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderDefault))
                                .child("+  New chat"),
                        )
                        .children(self.threads.iter().enumerate().map(|(i, (a, b))| {
                            let f = self.on_thread.clone();
                            div()
                                .id(("oxyn-cs-th", i))
                                .role(Role::Button)
                                .aria_label(SharedString::from(a.clone()))
                                .flex()
                                .flex_col()
                                .px(px(10.))
                                .py(px(7.))
                                .rounded(px(8.))
                                .cursor_pointer()
                                .when(i == self.active, |d| {
                                    d.bg(c(t, ColorRole::SurfaceRaised))
                                        .border_1()
                                        .border_color(c(t, ColorRole::BorderSubtle))
                                })
                                .child(a.clone())
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child(b.clone()),
                                )
                                .when_some(f, move |d, f| d.on_click(move |_, w, cx| f(i, w, cx)))
                        })),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .when(!self.title.is_empty(), |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .h(px(48.))
                                .px(px(18.))
                                .border_b_1()
                                .border_color(c(t, ColorRole::BorderSubtle))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(self.title.clone()),
                        )
                    })
                    .child(div().flex().flex_col().flex_1().children(self.main)),
            )
            .when_some(self.side, |d, s| {
                d.child(
                    div()
                        .id("oxyn-sh-2040")
                        .role(Role::Complementary)
                        .aria_label("Details")
                        .w(px(272.))
                        .p(px(14.))
                        .bg(c(t, ColorRole::SurfaceSunken))
                        .border_l_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .child(s),
                )
            })
    }
}

// ─────────────────────────────────────────── Harness ──

/// Harness: a demo frame with a title, viewport switch (0 desktop, 1 tablet, 2 phone), a dark
/// canvas toggle and **Fork this**. Actions (index): 0–2 viewport, 3 dark, 4 fork.
#[derive(IntoElement)]
pub struct HarnessEl {
    title: String,
    viewport: usize,
    dark: bool,
    forked: bool,
    demo: Option<AnyElement>,
    on_action: Option<OnIndex>,
    theme: &'static Theme,
}

impl HarnessEl {
    /// A harness titled `title`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            viewport: 0,
            dark: false,
            forked: false,
            demo: None,
            on_action: None,
            theme: &Theme::DARK,
        }
    }
    /// Viewport, dark canvas, forked.
    pub fn state(mut self, viewport: usize, dark: bool, forked: bool) -> Self {
        self.viewport = viewport;
        self.dark = dark;
        self.forked = forked;
        self
    }
    /// The demo.
    pub fn demo(mut self, e: impl IntoElement) -> Self {
        self.demo = Some(e.into_any_element());
        self
    }
    on_index!(on_action, "0–2 viewport, 3 dark, 4 fork.");
    themed!();
}

impl RenderOnce for HarnessEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let act = self.on_action.clone();
        let w = match self.viewport {
            1 => Some(640.),
            2 => Some(390.),
            _ => None,
        };
        let seg = |i: usize, label: &'static str| {
            let f = act.clone();
            div()
                .id(("oxyn-hn-vp", i))
                .role(Role::RadioButton)
                .aria_label(label)
                .aria_selected(self.viewport == i)
                .px(px(8.))
                .py(px(3.))
                .rounded(px(7.))
                .cursor_pointer()
                .text_size(px(12.))
                .when(self.viewport == i, |d| {
                    d.bg(c(t, ColorRole::SurfaceSelected))
                })
                .child(label)
                .when_some(f, move |d, f| d.on_click(move |_, w, cx| f(i, w, cx)))
        };
        let fork = act.clone();
        let dark = act.clone();
        div()
            .flex()
            .flex_col()
            .size_full()
            .rounded(px(14.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .overflow_hidden()
            .bg(if self.dark {
                gpui::rgb(0x111317).into()
            } else {
                c(t, ColorRole::SurfaceBase)
            })
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(44.))
                    .px(px(14.))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .id("oxyn-sh-2159")
                            .role(Role::RadioGroup)
                            .aria_label("Viewport")
                            .flex()
                            .gap(px(2.))
                            .p(px(2.))
                            .rounded(px(9.))
                            .bg(c(t, ColorRole::SurfaceSunken))
                            .child(seg(0, "Desktop"))
                            .child(seg(1, "Tablet"))
                            .child(seg(2, "Phone")),
                    )
                    .child(
                        div()
                            .id("oxyn-hn-dark")
                            .role(Role::Button)
                            .aria_label("Dark canvas")
                            .px(px(6.))
                            .cursor_pointer()
                            .child(if self.dark { "☀" } else { "☾" })
                            .when_some(dark, |d, f| d.on_click(move |_, w, cx| f(3, w, cx))),
                    )
                    .child(
                        div()
                            .id("oxyn-hn-fork")
                            .role(Role::Button)
                            .aria_label(if self.forked {
                                "Forked, open"
                            } else {
                                "Fork this"
                            })
                            .h(px(28.))
                            .px(px(12.))
                            .flex()
                            .items_center()
                            .rounded(px(8.))
                            .cursor_pointer()
                            .when(!self.forked, |d| {
                                d.bg(c(t, ColorRole::FgDefault))
                                    .text_color(c(t, ColorRole::BgBase))
                            })
                            .when(self.forked, |d| {
                                d.bg(c(t, ColorRole::StatusSuccessBg))
                                    .text_color(c(t, ColorRole::StatusSuccessFg))
                            })
                            .child(if self.forked {
                                "✓ Forked · open"
                            } else {
                                "⑂ Fork this"
                            })
                            .when_some(fork, |d, f| d.on_click(move |_, w, cx| f(4, w, cx))),
                    ),
            )
            .child(
                div().flex().justify_center().flex_1().p(px(20.)).child(
                    div()
                        .when_some(w, |d, w| d.w(px(w)))
                        .when(w.is_none(), |d| d.w_full())
                        .children(self.demo),
                ),
            )
    }
}
