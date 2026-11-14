//! Navigation, layout and app chrome for GPUI (plan #27), drawn from the same
//! `oxyn_core::primitives::{layout, chrome}` models as the Dioxus components. Elements are
//! `RenderOnce`: the host owns the model, routes pointer and key events to its methods (each
//! element reports presses through `on_*` callbacks with an index or id) and re-renders.
//!
//! Gaps against Dioxus (listed in the PR): no gliding highlight (the selected item is drawn in
//! place), no height springs (rows appear at full height), drags take positions from the host's
//! mouse handlers, the hover-card timers run on the host clock (`HoverIntent::tick`), menus and
//! panels draw in place (no window-level portal or focus trap), text entry in the team switcher
//! is display-only, and the TOC reads its progress from the host's scroll values.

use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, Orientation, ParentElement, RenderOnce, Role,
    SharedString, Styled, Toggled, Window, div, prelude::*, px,
};
use oxyn_core::primitives::{
    chrome::{
        NotificationCenter, Reactions, TeamSwitcher, Tok, UserMenu, UserRow, highlight,
        language_label,
    },
    layout::{
        Accordion, ChipGroup, CopyState, Crumb, CrumbSlot, Dock, HoverIntent, RadioCard, Rating,
        Resizable, Tabs, TocItem, ToggleGroup, Tree, crumb_slots, initials, time_left,
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnId = Rc<dyn Fn(String, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
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

macro_rules! on_index {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
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

fn frame(t: &'static Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .bg(c(t, ColorRole::SurfaceRaised))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .rounded(px(12.))
        .text_color(c(t, ColorRole::FgDefault))
        .text_size(px(13.))
}

// ─────────────────────────────────────────── Tabs ──

/// Tabs (`pill` or `underline`, closable terminal tabs with busy dots).
#[derive(IntoElement)]
pub struct TabsEl {
    model: Tabs,
    underline: bool,
    panel: Option<SharedString>,
    on_select: Option<OnIndex>,
    on_close: Option<OnIndex>,
    theme: &'static Theme,
}

impl TabsEl {
    /// From the model.
    pub fn new(model: Tabs) -> Self {
        Self {
            model,
            underline: false,
            panel: None,
            on_select: None,
            on_close: None,
            theme: &Theme::DARK,
        }
    }
    /// Underline variant.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    /// Panel text under the strip.
    pub fn panel(mut self, s: impl Into<SharedString>) -> Self {
        self.panel = Some(s.into());
        self
    }
    on_index!(on_select, "A tab was pressed.");
    on_index!(on_close, "A close button was pressed.");
    themed!();
}

impl RenderOnce for TabsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let sel = self.model.selected();
        let under = self.underline;
        let strip = div()
            .id("oxyn-tabs")
            .role(Role::TabList)
            .flex()
            .gap(px(2.))
            .p(px(if under { 0. } else { 3. }))
            .rounded(px(10.))
            .when(!under, |d| d.bg(c(t, ColorRole::SurfaceSunken)))
            .when(under, |d| {
                d.border_b_1().border_color(c(t, ColorRole::BorderSubtle))
            })
            .children(self.model.tabs.iter().enumerate().map(|(i, tb)| {
                let on = sel == Some(i);
                let f = self.on_select.clone();
                let fc = self.on_close.clone();
                div()
                    .id(("oxyn-tab", i))
                    .role(Role::Tab)
                    .aria_label(SharedString::from(tb.label.clone()))
                    .aria_selected(on)
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(30.))
                    .px(px(12.))
                    .rounded(px(7.))
                    .text_color(c(
                        t,
                        if on {
                            ColorRole::FgDefault
                        } else {
                            ColorRole::FgMuted
                        },
                    ))
                    .when(on && !under, |d| {
                        d.bg(c(t, ColorRole::SurfaceRaised))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                    })
                    .when(on && under, |d| {
                        d.border_b_2().border_color(c(t, ColorRole::FgDefault))
                    })
                    .when(tb.disabled, |d| d.opacity(0.45))
                    .when(tb.busy, |d| {
                        d.child(
                            div()
                                .size(px(6.))
                                .rounded_full()
                                .bg(c(t, ColorRole::StatusSuccessFg)),
                        )
                    })
                    .child(tb.label.clone())
                    .when_some(tb.count, |d, n| {
                        d.child(
                            div()
                                .px(px(5.))
                                .rounded_full()
                                .bg(c(t, ColorRole::SurfaceSelected))
                                .text_size(px(11.))
                                .child(n.to_string()),
                        )
                    })
                    .when(tb.closable, |d| {
                        d.child(
                            div()
                                .id(("oxyn-tab-x", i))
                                .aria_label(SharedString::from(format!("Close {}", tb.label)))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child("×")
                                .when_some(fc, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                                }),
                        )
                    })
                    .when_some(f.filter(|_| !tb.disabled), |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
            }));
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_size(px(13.))
            .child(strip)
            .when_some(self.panel, |d, p| {
                d.child(
                    div()
                        .id("oxyn-tabpanel")
                        .role(Role::TabPanel)
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(p),
                )
            })
    }
}

// ───────────────────────────────────── Breadcrumb ──

/// Breadcrumb with an optional collapsed middle.
#[derive(IntoElement)]
pub struct BreadcrumbEl {
    items: Vec<Crumb>,
    max: usize,
    on_select: Option<OnIndex>,
    theme: &'static Theme,
}

impl BreadcrumbEl {
    /// Crumbs.
    pub fn new(items: Vec<Crumb>) -> Self {
        Self {
            items,
            max: 0,
            on_select: None,
            theme: &Theme::DARK,
        }
    }
    /// Collapse beyond `n` crumbs.
    pub fn max(mut self, n: usize) -> Self {
        self.max = n;
        self
    }
    on_index!(on_select, "A crumb was pressed.");
    themed!();
}

impl RenderOnce for BreadcrumbEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let n = self.items.len();
        let slots = crumb_slots(n, self.max);
        div()
            .id("oxyn-crumbs")
            .role(Role::Navigation)
            .aria_label("Breadcrumb")
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(4.))
            .text_size(px(13.))
            .children(slots.into_iter().enumerate().map(|(k, s)| {
                let sep = (k > 0).then(|| div().text_color(c(t, ColorRole::FgSubtle)).child("›"));
                let body: AnyElement = match s {
                    CrumbSlot::More(h) => div()
                        .id(("crumb-more", k))
                        .text_color(c(t, ColorRole::FgMuted))
                        .aria_label(SharedString::from(format!("Show {} more", h.len())))
                        .child("…")
                        .into_any_element(),
                    CrumbSlot::Item(i) => {
                        let last = i + 1 == n;
                        let f = self.on_select.clone();
                        div()
                            .id(("crumb", i))
                            .when(!last, |d| d.role(Role::Link))
                            .text_color(c(
                                t,
                                if last {
                                    ColorRole::FgDefault
                                } else {
                                    ColorRole::FgMuted
                                },
                            ))
                            .when(last, |d| d.font_weight(FontWeight::MEDIUM))
                            .child(self.items[i].label.clone())
                            .when_some(f.filter(|_| !last), |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            })
                            .into_any_element()
                    }
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .children(sep)
                    .child(body)
            }))
    }
}

// ────────────────────────────────────── Accordion ──

/// Accordion rows (title, answer).
#[derive(IntoElement)]
pub struct AccordionEl {
    model: Accordion,
    rows: Vec<(SharedString, SharedString)>,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl AccordionEl {
    /// From the model and (title, answer) rows.
    pub fn new(model: Accordion, rows: Vec<(SharedString, SharedString)>) -> Self {
        Self {
            model,
            rows,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_toggle, "A trigger was pressed.");
    themed!();
}

impl RenderOnce for AccordionEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .text_size(px(14.))
            .children(self.rows.into_iter().enumerate().map(|(i, (q, a))| {
                let open = self.model.is_open(i);
                let f = self.on_toggle.clone();
                div()
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(
                        div()
                            .id(("acc", i))
                            .role(Role::Button)
                            .aria_expanded(open)
                            .aria_label(q.clone())
                            .flex()
                            .items_center()
                            .justify_between()
                            .min_h(px(52.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(q)
                            .child(div().text_color(c(t, ColorRole::FgMuted)).child(if open {
                                "⌃"
                            } else {
                                "⌄"
                            }))
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            }),
                    )
                    .when(open, |d| {
                        d.child(
                            div()
                                .id(("acc-p", i))
                                .role(Role::Region)
                                .pb(px(14.))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(a),
                        )
                    })
            }))
    }
}

// ─────────────────────────────── Resizable panels ──

/// A split view; the host maps drags to [`Resizable::move_divider`].
#[derive(IntoElement)]
pub struct ResizableEl {
    model: Resizable,
    titles: Vec<SharedString>,
    theme: &'static Theme,
}

impl ResizableEl {
    /// From the model and pane titles.
    pub fn new(model: Resizable, titles: Vec<SharedString>) -> Self {
        Self {
            model,
            titles,
            theme: &Theme::DARK,
        }
    }
    themed!();
}

impl RenderOnce for ResizableEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.model;
        let n = r.panes.len();
        let mut row = div()
            .flex()
            .size_full()
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .rounded(px(12.))
            .overflow_hidden()
            .text_size(px(13.));
        if r.vertical {
            row = row.flex_col();
        }
        for i in 0..n {
            let s = r.sizes[i];
            row = row.child(
                div()
                    .flex_none()
                    .when(!r.vertical, |d| d.w(px(s)))
                    .when(r.vertical, |d| d.h(px(s)))
                    .overflow_hidden()
                    .opacity(r.squeeze(i).max(0.15))
                    .p(px(12.))
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgMuted))
                            .text_size(px(11.))
                            .child(self.titles.get(i).cloned().unwrap_or_default()),
                    ),
            );
            if i + 1 < n {
                row = row.child(
                    div()
                        .id(("divider", i))
                        .role(Role::Splitter)
                        .aria_orientation(if r.vertical {
                            Orientation::Horizontal
                        } else {
                            Orientation::Vertical
                        })
                        .aria_numeric_value(f64::from(r.value_now(i)))
                        .aria_min_numeric_value(0.)
                        .aria_max_numeric_value(100.)
                        .aria_value(SharedString::from(r.value_text(i)))
                        .flex_none()
                        .when(!r.vertical, |d| d.w(px(8.)).h_full())
                        .when(r.vertical, |d| d.h(px(8.)).w_full())
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .when(!r.vertical, |d| d.w(px(1.)).h_full())
                                .when(r.vertical, |d| d.h(px(1.)).w_full())
                                .bg(c(t, ColorRole::BorderDefault)),
                        ),
                );
            }
        }
        row
    }
}

// ────────────────────────────────────── Hover card ──

/// A hover card around a trigger; the host drives [`HoverIntent`] from hover / focus events.
#[derive(IntoElement)]
pub struct HoverCardEl {
    intent: HoverIntent,
    trigger: SharedString,
    name: SharedString,
    role: Option<SharedString>,
    bio: Option<SharedString>,
    theme: &'static Theme,
}

impl HoverCardEl {
    /// A person preview.
    pub fn new(intent: HoverIntent, trigger: impl Into<SharedString>, name: &str) -> Self {
        Self {
            intent,
            trigger: trigger.into(),
            name: SharedString::from(name.to_string()),
            role: None,
            bio: None,
            theme: &Theme::DARK,
        }
    }
    /// Role line.
    pub fn role_line(mut self, r: impl Into<SharedString>) -> Self {
        self.role = Some(r.into());
        self
    }
    /// Bio.
    pub fn bio(mut self, b: impl Into<SharedString>) -> Self {
        self.bio = Some(b.into());
        self
    }
    themed!();
}

impl RenderOnce for HoverCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .id("oxyn-hc-trg")
                    .role(Role::Button)
                    .font_weight(FontWeight::MEDIUM)
                    .underline()
                    .child(self.trigger),
            )
            .when(self.intent.open, |d| {
                d.child(
                    frame(t)
                        .id("oxyn-hc")
                        .role(Role::Tooltip)
                        .w(px(296.))
                        .p(px(14.))
                        .gap(px(8.))
                        .child(
                            div()
                                .flex()
                                .gap(px(10.))
                                .items_center()
                                .child(
                                    div()
                                        .size(px(40.))
                                        .rounded_full()
                                        .bg(c(t, ColorRole::SurfaceSelected))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(initials(&self.name)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(self.name.clone()),
                                        )
                                        .children(self.role.map(|r| {
                                            div().text_color(c(t, ColorRole::FgMuted)).child(r)
                                        })),
                                ),
                        )
                        .children(
                            self.bio
                                .map(|b| div().text_color(c(t, ColorRole::FgMuted)).child(b)),
                        ),
                )
            })
    }
}

// ─────────────────────────────────────── Tree view ──

/// Tree view.
#[derive(IntoElement)]
pub struct TreeViewEl {
    model: Tree,
    on_activate: Option<OnId>,
    theme: &'static Theme,
}

impl TreeViewEl {
    /// From the model.
    pub fn new(model: Tree) -> Self {
        Self {
            model,
            on_activate: None,
            theme: &Theme::DARK,
        }
    }
    on_id!(on_activate, "A row was pressed (id).");
    themed!();
}

impl RenderOnce for TreeViewEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let sel = self.model.selected.clone();
        div()
            .id("oxyn-tree")
            .role(Role::Tree)
            .flex()
            .flex_col()
            .text_size(px(13.))
            .children(self.model.rows().into_iter().map(|r| {
                let on = sel.as_ref() == Some(&r.id);
                let f = self.on_activate.clone();
                let id = r.id.clone();
                div()
                    .id(SharedString::from(format!("tree-{}", r.id)))
                    .role(Role::TreeItem)
                    .aria_level(r.level)
                    .aria_position_in_set(r.pos)
                    .aria_size_of_set(r.size)
                    .aria_selected(on)
                    .when(r.folder, |d| d.aria_expanded(r.expanded))
                    .aria_label(SharedString::from(r.label.clone()))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(32.))
                    .pl(px(6. + (r.level as f32 - 1.) * 18.))
                    .rounded(px(7.))
                    .text_color(c(
                        t,
                        if on {
                            ColorRole::FgDefault
                        } else {
                            ColorRole::FgMuted
                        },
                    ))
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .child(if r.folder {
                        if r.expanded { "▾" } else { "▸" }
                    } else {
                        " "
                    })
                    .child(r.label.clone())
                    .children(r.badge.clone().map(|b| {
                        div()
                            .text_size(px(10.5))
                            .text_color(c(
                                t,
                                match b.as_str() {
                                    "A" => ColorRole::StatusSuccessFg,
                                    "D" => ColorRole::StatusDangerFg,
                                    _ => ColorRole::StatusWarningFg,
                                },
                            ))
                            .child(b)
                    }))
                    .when_some(f, |d, f| {
                        d.cursor_pointer()
                            .on_click(move |_, w, cx| f(id.clone(), w, cx))
                    })
            }))
    }
}

// ───────────────────────────────── Reorderable list ──

/// Reorderable list rows (title, detail); the host moves items with `move_item`.
#[derive(IntoElement)]
pub struct ReorderListEl {
    rows: Vec<(SharedString, Option<SharedString>)>,
    dragging: Option<usize>,
    theme: &'static Theme,
}

impl ReorderListEl {
    /// Rows.
    pub fn new(rows: Vec<(SharedString, Option<SharedString>)>) -> Self {
        Self {
            rows,
            dragging: None,
            theme: &Theme::DARK,
        }
    }
    /// The row being dragged (lifted).
    pub fn dragging(mut self, i: Option<usize>) -> Self {
        self.dragging = i;
        self
    }
    themed!();
}

impl RenderOnce for ReorderListEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let n = self.rows.len();
        div()
            .id("oxyn-reorder")
            .role(Role::List)
            .flex()
            .flex_col()
            .gap(px(6.))
            .children(self.rows.into_iter().enumerate().map(|(i, (title, det))| {
                frame(t)
                    .id(("reorder", i))
                    .flex_row()
                    .items_center()
                    .gap(px(10.))
                    .min_h(px(52.))
                    .px(px(8.))
                    .rounded(px(10.))
                    .when(self.dragging == Some(i), |d| d.shadow_lg())
                    .child(
                        div()
                            .id(("grip", i))
                            .role(Role::Button)
                            .aria_label(SharedString::from(
                                oxyn_core::primitives::layout::handle_label(&title, i + 1, n),
                            ))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child("⋮⋮"),
                    )
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgSubtle))
                            .text_size(px(12.))
                            .child((i + 1).to_string()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().font_weight(FontWeight::MEDIUM).child(title))
                            .children(det.map(|d| {
                                div()
                                    .text_size(px(12.))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child(d)
                            })),
                    )
            }))
    }
}

// ───────────────────────────── Toggle / chips / cards ──

/// Toggle group (single radio or multiple pressed with joined neighbours).
#[derive(IntoElement)]
pub struct ToggleGroupEl {
    model: ToggleGroup,
    on_press: Option<OnIndex>,
    theme: &'static Theme,
}

impl ToggleGroupEl {
    /// From the model.
    pub fn new(model: ToggleGroup) -> Self {
        Self {
            model,
            on_press: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_press, "An item was pressed.");
    themed!();
}

impl RenderOnce for ToggleGroupEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let g = self.model;
        let joins = g.joins();
        div()
            .id("oxyn-tgl")
            .role(if g.multiple {
                Role::Group
            } else {
                Role::RadioGroup
            })
            .flex()
            .gap(px(2.))
            .p(px(3.))
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .children(g.items.iter().enumerate().map(|(i, it)| {
                let on = g.on(i);
                let f = self.on_press.clone();
                let (jl, jr) = joins[i];
                div()
                    .id(("tgl", i))
                    .role(if g.multiple {
                        Role::Button
                    } else {
                        Role::RadioButton
                    })
                    .aria_label(SharedString::from(it.label.clone()))
                    .aria_toggled(if on { Toggled::True } else { Toggled::False })
                    .min_w(px(36.))
                    .h(px(36.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(7.))
                    .when(jl, |d| d.rounded_l_none())
                    .when(jr, |d| d.rounded_r_none())
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .text_color(c(
                        t,
                        if on {
                            ColorRole::FgDefault
                        } else {
                            ColorRole::FgMuted
                        },
                    ))
                    .child(if it.text {
                        it.label.clone()
                    } else {
                        it.label.chars().take(1).collect()
                    })
                    .when_some(f.filter(|_| !it.disabled), |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
            }))
    }
}

/// Chip group with "+N more".
#[derive(IntoElement)]
pub struct ChipGroupEl {
    model: ChipGroup,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl ChipGroupEl {
    /// From the model.
    pub fn new(model: ChipGroup) -> Self {
        Self {
            model,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_toggle, "A chip was pressed (option index).");
    themed!();
}

impl RenderOnce for ChipGroupEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let cg = self.model;
        let (vis, hidden) = cg.visible();
        div()
            .id("oxyn-chips")
            .role(Role::Group)
            .flex()
            .flex_wrap()
            .gap(px(6.))
            .children(vis.into_iter().map(|i| {
                let on = cg.on(i);
                let f = self.on_toggle.clone();
                div()
                    .id(("chip", i))
                    .role(Role::Button)
                    .aria_toggled(if on { Toggled::True } else { Toggled::False })
                    .h(px(30.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .rounded_full()
                    .border_1()
                    .border_color(c(
                        t,
                        if on {
                            ColorRole::BorderStrong
                        } else {
                            ColorRole::BorderDefault
                        },
                    ))
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)).child("✓"))
                    .child(cg.options[i].1.clone())
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
            }))
            .when(hidden > 0, |d| {
                d.child(
                    div()
                        .h(px(30.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(format!("+{hidden} more")),
                )
            })
    }
}

/// Radio cards.
#[derive(IntoElement)]
pub struct RadioCardsEl {
    cards: Vec<RadioCard>,
    value: Option<String>,
    on_select: Option<OnIndex>,
    theme: &'static Theme,
}

impl RadioCardsEl {
    /// Cards and the selected value.
    pub fn new(cards: Vec<RadioCard>, value: Option<&str>) -> Self {
        Self {
            cards,
            value: value.map(str::to_string),
            on_select: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_select, "A card was pressed.");
    themed!();
}

impl RenderOnce for RadioCardsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .id("oxyn-rcards")
            .role(Role::RadioGroup)
            .flex()
            .flex_wrap()
            .gap(px(10.))
            .children(self.cards.iter().enumerate().map(|(i, rc)| {
                let on = self.value.as_deref() == Some(rc.value.as_str());
                let f = self.on_select.clone();
                frame(t)
                    .id(("rcard", i))
                    .role(Role::RadioButton)
                    .aria_selected(on)
                    .aria_label(SharedString::from(rc.label.clone()))
                    .w(px(200.))
                    .p(px(12.))
                    .gap(px(4.))
                    .when(on, |d| {
                        d.border_2().border_color(c(t, ColorRole::FgDefault))
                    })
                    .when(rc.disabled, |d| d.opacity(0.5))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(rc.label.clone()),
                    )
                    .children(rc.description.clone().map(|ds| {
                        div()
                            .text_size(px(12.5))
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(ds)
                    }))
                    .children(
                        rc.meta
                            .clone()
                            .map(|m| div().font_weight(FontWeight::SEMIBOLD).child(m)),
                    )
                    .when_some(f.filter(|_| !rc.disabled), |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
            }))
    }
}

/// Star rating.
#[derive(IntoElement)]
pub struct RatingEl {
    model: Rating,
    label: SharedString,
    on_commit: Option<OnIndex>,
    theme: &'static Theme,
}

impl RatingEl {
    /// From the model.
    pub fn new(model: Rating, label: impl Into<SharedString>) -> Self {
        Self {
            model,
            label: label.into(),
            on_commit: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_commit, "A star was pressed (0-based).");
    themed!();
}

impl RenderOnce for RatingEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.model;
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .id("oxyn-rating")
                    .role(Role::RadioGroup)
                    .aria_label(self.label.clone())
                    .flex()
                    .children((0..r.max).map(|i| {
                        let f = r.fill(i);
                        let cb = self.on_commit.clone();
                        div()
                            .id(("star", i as usize))
                            .role(Role::RadioButton)
                            .aria_label(SharedString::from(r.radio_label((i + 1) as f32, &[])))
                            .size(px(32.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(20.))
                            .text_color(if f >= 1.0 {
                                c(t, ColorRole::FgDefault)
                            } else if f > 0.0 {
                                c(t, ColorRole::FgMuted)
                            } else {
                                c(t, ColorRole::BorderStrong)
                            })
                            .child(if f > 0.0 && f < 1.0 { "⯪" } else { "★" })
                            .when_some(cb, |d, cb| {
                                d.cursor_pointer()
                                    .on_click(move |_, w, cx| cb(i as usize, w, cx))
                            })
                    })),
            )
            .child(div().text_color(c(t, ColorRole::FgMuted)).child(r.word(
                r.shown(),
                &[],
                "Not rated",
            )))
    }
}

/// Copy button.
#[derive(IntoElement)]
pub struct CopyButtonEl {
    state: CopyState,
    label: SharedString,
    on_copy: Option<OnIndex>,
    theme: &'static Theme,
}

impl CopyButtonEl {
    /// A copy button in `state` (from [`CopyState::at`]).
    pub fn new(state: CopyState, label: impl Into<SharedString>) -> Self {
        Self {
            state,
            label: label.into(),
            on_copy: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(
        on_copy,
        "Pressed (index is always 0); the host writes the clipboard."
    );
    themed!();
}

impl RenderOnce for CopyButtonEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = self.state;
        let f = self.on_copy.clone();
        div()
            .id("oxyn-copy")
            .role(Role::Button)
            .aria_label(self.label.clone())
            .flex()
            .items_center()
            .gap(px(6.))
            .h(px(30.))
            .px(px(10.))
            .rounded(px(8.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .text_size(px(12.5))
            .child(
                div()
                    .text_color(c(
                        t,
                        match s {
                            CopyState::Copied(_) => ColorRole::StatusSuccessFg,
                            CopyState::Failed(_) => ColorRole::StatusDangerFg,
                            CopyState::Idle => ColorRole::FgMuted,
                        },
                    ))
                    .child(match s {
                        CopyState::Copied(_) => "✓",
                        CopyState::Failed(_) => "×",
                        CopyState::Idle => "⧉",
                    }),
            )
            .child(s.label(&self.label).to_string())
            .when_some(f, |d, f| {
                d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
            })
    }
}

/// Reactions bar.
#[derive(IntoElement)]
pub struct ReactionsEl {
    model: Reactions,
    on_toggle: Option<OnId>,
    theme: &'static Theme,
}

impl ReactionsEl {
    /// From the model.
    pub fn new(model: Reactions) -> Self {
        Self {
            model,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    on_id!(on_toggle, "A pill or picker option was pressed (id).");
    themed!();
}

impl RenderOnce for ReactionsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.model;
        let pill = |t: &'static Theme| {
            div()
                .flex()
                .items_center()
                .gap(px(5.))
                .h(px(28.))
                .px(px(9.))
                .rounded_full()
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
        };
        div()
            .id("oxyn-react")
            .role(Role::Group)
            .flex()
            .flex_wrap()
            .gap(px(6.))
            .children(r.value.iter().enumerate().map(|(i, rc)| {
                let f = self.on_toggle.clone();
                let id = rc.id.clone();
                pill(t)
                    .id(("react", i))
                    .role(Role::Button)
                    .aria_label(SharedString::from(r.pill_label(rc)))
                    .aria_toggled(if rc.reacted {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .when(rc.reacted, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .child(
                        r.option(&rc.id)
                            .map(|o| o.glyph.clone())
                            .unwrap_or_default(),
                    )
                    .child(rc.count.to_string())
                    .when_some(f, |d, f| {
                        d.cursor_pointer()
                            .on_click(move |_, w, cx| f(id.clone(), w, cx))
                    })
            }))
            .child(
                pill(t)
                    .id("react-add")
                    .aria_label("Add reaction")
                    .child("+"),
            )
            .when(r.picker, |d| {
                d.child(
                    div()
                        .id("oxyn-react-tray")
                        .role(Role::Toolbar)
                        .flex()
                        .gap(px(2.))
                        .p(px(4.))
                        .rounded_full()
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .children(r.options.iter().enumerate().map(|(i, o)| {
                            let f = self.on_toggle.clone();
                            let id = o.id.clone();
                            div()
                                .id(("react-opt", i))
                                .aria_label(SharedString::from(o.label.clone()))
                                .size(px(30.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(o.glyph.clone())
                                .when_some(f, |d, f| {
                                    d.cursor_pointer()
                                        .on_click(move |_, w, cx| f(id.clone(), w, cx))
                                })
                        })),
                )
            })
    }
}

/// Expandable text (clamped by a line count).
#[derive(IntoElement)]
pub struct ExpandableTextEl {
    text: SharedString,
    lines: u32,
    open: bool,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl ExpandableTextEl {
    /// Text clamped to `lines`.
    pub fn new(text: impl Into<SharedString>, lines: u32, open: bool) -> Self {
        Self {
            text: text.into(),
            lines,
            open,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_toggle, "The toggle was pressed.");
    themed!();
}

impl RenderOnce for ExpandableTextEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let f = self.on_toggle.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .max_w(px(560.))
            .child(
                div()
                    .when(!self.open, |d| {
                        d.max_h(px(20. * self.lines as f32)).overflow_hidden()
                    })
                    .child(self.text),
            )
            .child(
                div()
                    .id("oxyn-xt")
                    .role(Role::Button)
                    .aria_expanded(self.open)
                    .text_color(c(t, ColorRole::FgMuted))
                    .font_weight(FontWeight::MEDIUM)
                    .child(if self.open { "Show less" } else { "Read more" })
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                    }),
            )
    }
}

// ─────────────────────────────────────────── Dock ──

/// Dock toolbar with group trays.
#[derive(IntoElement)]
pub struct DockEl {
    model: Dock,
    on_press: Option<OnIndex>,
    theme: &'static Theme,
}

impl DockEl {
    /// From the model.
    pub fn new(model: Dock) -> Self {
        Self {
            model,
            on_press: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_press, "A dock item was pressed.");
    themed!();
}

impl RenderOnce for DockEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let d = self.model;
        div()
            .id("oxyn-dock")
            .role(Role::Toolbar)
            .aria_label("Tools")
            .flex()
            .items_end()
            .gap(px(4.))
            .p(px(6.))
            .rounded(px(18.))
            .bg(c(t, ColorRole::SurfaceOverlay))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .shadow_lg()
            .children(d.items.iter().enumerate().map(|(i, it)| {
                let on = d.is_selected(i);
                let f = self.on_press.clone();
                let glyph = d
                    .shown_member(i)
                    .map(|m| m.label.clone())
                    .unwrap_or(it.label.clone());
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .items_center()
                    .when(d.tray == Some(i), |x| {
                        x.child(
                            div()
                                .id(("dock-tray", i))
                                .role(Role::Group)
                                .flex()
                                .flex_col()
                                .p(px(5.))
                                .mb(px(6.))
                                .rounded(px(12.))
                                .bg(c(t, ColorRole::SurfaceOverlay))
                                .children(
                                    it.items.iter().map(|m| {
                                        div().h(px(28.)).px(px(8.)).child(m.label.clone())
                                    }),
                                ),
                        )
                    })
                    .child(
                        div()
                            .id(("dock", i))
                            .role(Role::Button)
                            .aria_label(SharedString::from(d.aria(i)))
                            .size(px(42.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(12.))
                            .when(on, |x| x.bg(c(t, ColorRole::SurfaceSelected)))
                            .text_color(c(
                                t,
                                if on {
                                    ColorRole::FgDefault
                                } else {
                                    ColorRole::FgMuted
                                },
                            ))
                            .child(glyph.chars().take(2).collect::<String>())
                            .when_some(f, |x, f| {
                                x.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            }),
                    )
            }))
    }
}

// ─────────────────────────────── TOC / page header ──

/// Table of contents with reading progress (section, fraction, overall from `toc_progress`).
#[derive(IntoElement)]
pub struct TocProgressEl {
    items: Vec<TocItem>,
    progress: (usize, f32, f32),
    minutes: f32,
    theme: &'static Theme,
}

impl TocProgressEl {
    /// Headings and the current progress.
    pub fn new(items: Vec<TocItem>, progress: (usize, f32, f32), minutes: f32) -> Self {
        Self {
            items,
            progress,
            minutes,
            theme: &Theme::DARK,
        }
    }
    themed!();
}

impl RenderOnce for TocProgressEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let (cur, frac, overall) = self.progress;
        div()
            .id("oxyn-toc")
            .role(Role::Navigation)
            .aria_label("On this page")
            .flex()
            .flex_col()
            .w(px(200.))
            .text_size(px(12.5))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .mb(px(8.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("On this page")
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(time_left(self.minutes, overall)),
                    ),
            )
            .children(self.items.iter().enumerate().map(|(i, it)| {
                let f = if i < cur {
                    1.0
                } else if i == cur {
                    frac
                } else {
                    0.0
                };
                div()
                    .flex()
                    .gap(px(8.))
                    .pl(px((it.level as f32 - 1.) * 12.))
                    .child(
                        div()
                            .w(px(2.))
                            .h(px(22.))
                            .bg(c(t, ColorRole::BorderSubtle))
                            .child(div().w_full().h(px(22. * f)).bg(c(t, ColorRole::FgDefault))),
                    )
                    .child(
                        div()
                            .text_color(c(
                                t,
                                if i == cur {
                                    ColorRole::FgDefault
                                } else if i < cur {
                                    ColorRole::FgMuted
                                } else {
                                    ColorRole::FgSubtle
                                },
                            ))
                            .child(it.label.clone()),
                    )
            }))
    }
}

/// Page header (folds into a compact bar when `folded`).
#[derive(IntoElement)]
pub struct PageHeaderEl {
    title: SharedString,
    meta: SharedString,
    folded: bool,
    tabs: Option<Tabs>,
    theme: &'static Theme,
}

impl PageHeaderEl {
    /// A header.
    pub fn new(
        title: impl Into<SharedString>,
        meta: impl Into<SharedString>,
        folded: bool,
    ) -> Self {
        Self {
            title: title.into(),
            meta: meta.into(),
            folded,
            tabs: None,
            theme: &Theme::DARK,
        }
    }
    /// Tabs under the title.
    pub fn tabs(mut self, t: Tabs) -> Self {
        self.tabs = Some(t);
        self
    }
    themed!();
}

impl RenderOnce for PageHeaderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .id("oxyn-phead")
            .role(Role::Banner)
            .flex()
            .flex_col()
            .gap(px(if self.folded { 4. } else { 8. }))
            .px(px(16.))
            .pt(px(if self.folded { 8. } else { 14. }))
            .border_b_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .child(
                div()
                    .text_size(px(if self.folded { 15. } else { 20. }))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title),
            )
            .when(!self.folded, |d| {
                d.child(div().text_color(c(t, ColorRole::FgMuted)).child(self.meta))
            })
            .children(self.tabs.map(|tb| TabsEl::new(tb).underline().theme(t)))
    }
}

// ───────────────────────────── User menu / switcher ──

/// User menu panel (draw when `model.open`).
#[derive(IntoElement)]
pub struct UserMenuEl {
    model: UserMenu,
    name: SharedString,
    email: SharedString,
    on_row: Option<OnIndex>,
    theme: &'static Theme,
}

impl UserMenuEl {
    /// From the model.
    pub fn new(model: UserMenu, name: &str, email: &str) -> Self {
        Self {
            model,
            name: SharedString::from(name.to_string()),
            email: SharedString::from(email.to_string()),
            on_row: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_row, "A row was pressed (index into `rows()`).");
    themed!();
}

impl RenderOnce for UserMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let trig = div()
            .id("oxyn-um-trg")
            .role(Role::Button)
            .aria_expanded(m.open)
            .aria_label(SharedString::from(
                oxyn_core::primitives::chrome::account_label(&self.name, None),
            ))
            .size(px(30.))
            .rounded_full()
            .bg(c(t, ColorRole::SurfaceSelected))
            .flex()
            .items_center()
            .justify_center()
            .child(initials(&self.name));
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(trig)
            .when(m.open, |d| {
                d.child(
                    frame(t)
                        .id("oxyn-um")
                        .role(Role::Menu)
                        .w(px(280.))
                        .p(px(6.))
                        .child(
                            div()
                                .p(px(8.))
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(self.name.clone()),
                                )
                                .child(
                                    div().text_color(c(t, ColorRole::FgMuted)).child(self.email),
                                ),
                        )
                        .children(m.rows().into_iter().enumerate().map(|(ri, r)| {
                            let f = self.on_row.clone();
                            let label = match &r {
                                UserRow::Item(i) => m.items[*i].clone(),
                                UserRow::Theme => format!("Theme · {}", m.theme.id()),
                                UserRow::Status => format!("Status · {}", m.status.label()),
                                UserRow::SignOut => "Sign out".into(),
                            };
                            div()
                                .id(("um-row", ri))
                                .role(Role::MenuItem)
                                .h(px(34.))
                                .px(px(8.))
                                .flex()
                                .items_center()
                                .rounded(px(8.))
                                .when(m.highlight == Some(ri), |d| {
                                    d.bg(c(t, ColorRole::SurfaceHover))
                                })
                                .child(label)
                                .when_some(f, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(ri, w, cx))
                                })
                        })),
                )
            })
    }
}

/// Team switcher trigger and (when open) its list.
#[derive(IntoElement)]
pub struct TeamSwitcherEl {
    model: TeamSwitcher,
    on_pick: Option<OnIndex>,
    theme: &'static Theme,
}

impl TeamSwitcherEl {
    /// From the model.
    pub fn new(model: TeamSwitcher) -> Self {
        Self {
            model,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_pick, "A workspace row was pressed (workspace index).");
    themed!();
}

impl RenderOnce for TeamSwitcherEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = self.model;
        let tile = |t: &'static Theme, l: String| {
            div()
                .size(px(24.))
                .rounded(px(7.))
                .bg(c(t, ColorRole::SurfaceSelected))
                .flex()
                .items_center()
                .justify_center()
                .font_weight(FontWeight::BOLD)
                .child(l)
        };
        let cur = s.current().cloned();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .child(
                div()
                    .id("oxyn-team-trg")
                    .role(Role::Button)
                    .aria_expanded(s.open)
                    .aria_label(SharedString::from(s.trigger_label()))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .children(cur.map(|w| {
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(tile(t, w.letter()))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(w.name))
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(c(t, ColorRole::FgMuted))
                                            .child(if s.switching.is_some() {
                                                "Switching…".to_string()
                                            } else {
                                                w.plan.unwrap_or_default()
                                            }),
                                    ),
                            )
                    })),
            )
            .when(s.open, |d| {
                d.child(
                    frame(t)
                        .id("oxyn-team")
                        .role(Role::ListBox)
                        .w(px(300.))
                        .p(px(6.))
                        .children(s.workspaces.iter().enumerate().map(|(i, w)| {
                            let f = self.on_pick.clone();
                            div()
                                .id(("team", i))
                                .role(Role::ListBoxOption)
                                .aria_selected(w.id == s.active)
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .h(px(40.))
                                .px(px(8.))
                                .rounded(px(8.))
                                .child(tile(t, w.letter()))
                                .child(w.name.clone())
                                .when(w.id == s.active, |d| d.child("✓"))
                                .child(
                                    div()
                                        .text_color(c(t, ColorRole::FgSubtle))
                                        .child(format!("⌘{}", i + 1)),
                                )
                                .when_some(f, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                                })
                        })),
                )
            })
    }
}

/// Notification center panel.
#[derive(IntoElement)]
pub struct NotificationCenterEl {
    model: NotificationCenter,
    on_open: Option<OnId>,
    theme: &'static Theme,
}

impl NotificationCenterEl {
    /// From the model.
    pub fn new(model: NotificationCenter) -> Self {
        Self {
            model,
            on_open: None,
            theme: &Theme::DARK,
        }
    }
    on_id!(on_open, "A row was pressed (id).");
    themed!();
}

impl RenderOnce for NotificationCenterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        frame(t)
            .id("oxyn-nc")
            .role(Role::Region)
            .aria_label(SharedString::from(m.trigger_label()))
            .w(px(360.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .p(px(12.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Notifications"),
                    )
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(m.summary()),
                    ),
            )
            .children(m.groups().into_iter().map(|(g, notes)| {
                div()
                    .flex()
                    .flex_col()
                    .px(px(6.))
                    .child(
                        div()
                            .px(px(6.))
                            .py(px(4.))
                            .text_size(px(11.))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(g),
                    )
                    .children(notes.into_iter().map(|n| {
                        let f = self.on_open.clone();
                        let id = n.id.clone();
                        let open = m.expanded.as_deref() == Some(n.id.as_str());
                        div()
                            .id(SharedString::from(format!("nc-{}", n.id)))
                            .role(Role::Button)
                            .aria_expanded(open)
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .rounded(px(9.))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(n.actor.clone()),
                                    )
                                    .child(n.title.clone())
                                    .when(n.unread, |d| {
                                        d.child(
                                            div()
                                                .size(px(7.))
                                                .rounded_full()
                                                .bg(c(t, ColorRole::FgDefault)),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(n.time.clone()),
                            )
                            .when(open, |d| {
                                d.child(
                                    div()
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child(n.body.clone()),
                                )
                            })
                            .when_some(f, |d, f| {
                                d.cursor_pointer()
                                    .on_click(move |_, w, cx| f(id.clone(), w, cx))
                            })
                    }))
            }))
    }
}

// ─────────────────────────────────────── Code block ──

/// Code block with the shared highlighter.
#[derive(IntoElement)]
pub struct CodeBlockEl {
    code: String,
    lang: String,
    filename: Option<SharedString>,
    line_numbers: bool,
    theme: &'static Theme,
}

impl CodeBlockEl {
    /// Source and language.
    pub fn new(code: &str, lang: &str) -> Self {
        Self {
            code: code.into(),
            lang: lang.into(),
            filename: None,
            line_numbers: false,
            theme: &Theme::DARK,
        }
    }
    /// File name.
    pub fn filename(mut self, f: impl Into<SharedString>) -> Self {
        self.filename = Some(f.into());
        self
    }
    /// Line numbers.
    pub fn line_numbers(mut self) -> Self {
        self.line_numbers = true;
        self
    }
    themed!();
}

impl RenderOnce for CodeBlockEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let color = |k: Tok| match k {
            Tok::Keyword | Tok::Type => c(t, ColorRole::FgDefault),
            Tok::Str | Tok::Add => c(t, ColorRole::StatusSuccessFg),
            Tok::Num => c(t, ColorRole::StatusWarningFg),
            Tok::Comment => c(t, ColorRole::FgSubtle),
            Tok::Func => c(t, ColorRole::StatusInfoFg),
            Tok::Del => c(t, ColorRole::StatusDangerFg),
            Tok::Punct | Tok::Plain => c(t, ColorRole::FgMuted),
        };
        let lines = highlight(&self.code, &self.lang);
        let ln = self.line_numbers;
        div()
            .id("oxyn-code")
            .role(Role::Region)
            .aria_label(SharedString::from(format!(
                "{} source code",
                self.filename.clone().unwrap_or("Source".into())
            )))
            .flex()
            .flex_col()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceSunken))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .h(px(36.))
                    .px(px(12.))
                    .items_center()
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .text_size(px(12.5))
                    .child(self.filename.unwrap_or("Source code".into()))
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(language_label(&self.lang)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(px(12.))
                    .font_family("monospace")
                    .text_size(px(12.5))
                    .children(lines.into_iter().enumerate().map(move |(i, l)| {
                        div()
                            .flex()
                            .when(ln, |d| {
                                d.child(
                                    div()
                                        .w(px(28.))
                                        .text_color(c(t, ColorRole::FgSubtle))
                                        .child((i + 1).to_string()),
                                )
                            })
                            .children(
                                l.into_iter()
                                    .map(|(k, s)| div().text_color(color(k)).child(s)),
                            )
                    })),
            )
    }
}
