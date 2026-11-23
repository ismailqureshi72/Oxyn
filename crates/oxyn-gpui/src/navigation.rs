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
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

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

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
