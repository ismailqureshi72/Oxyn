//! Overlays: `Tooltip`, `Popover`, `DropdownMenu` (also a context menu).
//!
//! Timers run in CSS: an invisible element plays an animation of the wanted length and its
//! `animationend` advances the state (hover delay, the 300 ms neighbour window, hover-intent open
//! and close), so the timings hold without a host clock. Placement uses CSS anchor positioning
//! with flip fallbacks where the browser has it, and absolute side / align classes otherwise.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        display::{Platform, parse_shortcut},
        menu::{ItemKind, Menu, MenuItem, MenuKey, MenuOutcome},
    },
};

use super::{buttons::Keys, cls, use_glide};
use crate::composer::icons::Icon;

fn menu_key(k: &Key) -> Option<MenuKey> {
    Some(match k {
        Key::ArrowDown => MenuKey::Down,
        Key::ArrowUp => MenuKey::Up,
        Key::Home => MenuKey::Home,
        Key::End => MenuKey::End,
        Key::Enter => MenuKey::Enter,
        Key::ArrowRight => MenuKey::Right,
        Key::ArrowLeft => MenuKey::Left,
        Key::Escape => MenuKey::Escape,
        Key::Tab => MenuKey::Tab,
        Key::Character(c) if c == " " => MenuKey::Space,
        Key::Character(c) => MenuKey::Char(c.chars().next()?),
        _ => return None,
    })
}

/// Dropdown menu: actions and choices behind any trigger. Icons, shortcuts, checks, red items,
/// dividers and submenus; one soft highlight that glides between rows (red-tinted on a danger
/// row); type-ahead; the full menu keyboard. Oxyn plus: a filter field, compact density, and a
/// context-menu mode (`context`) that wraps content and opens at the pointer or on Shift+F10.
#[component]
pub fn DropdownMenu(
    /// Rows.
    items: Vec<MenuItem>,
    /// Trigger label (ignored in context mode).
    #[props(default = "Options".to_string())]
    trigger: String,
    /// Trigger icon.
    #[props(default)]
    icon: Option<String>,
    /// `start` · `end`.
    #[props(default = "start".to_string())]
    align: String,
    /// `bottom` · `top`.
    #[props(default = "bottom".to_string())]
    side: String,
    /// Heading row.
    #[props(default)]
    title: Option<String>,
    /// Width px.
    #[props(default)]
    width: Option<f32>,
    /// Empty text.
    #[props(default = "Nothing here yet".to_string())]
    empty: String,
    /// Start open.
    #[props(default)]
    open: bool,
    /// Review sheets: highlight row `i` (and open its submenu when it has one).
    #[props(default)]
    highlight: Option<usize>,
    /// Filter field.
    #[props(default)]
    searchable: bool,
    /// `compact` · `comfortable`.
    #[props(default = "comfortable".to_string())]
    density: String,
    /// Context-menu mode: wraps `children` and opens on right-click / Shift+F10.
    #[props(default)]
    context: bool,
    /// Wrapped content in context mode.
    #[props(default)]
    children: Element,
    /// Clock (type-ahead window).
    #[props(default)]
    now: ClockMs,
    /// An action ran.
    #[props(default)]
    on_action: EventHandler<String>,
    /// A checkbox / radio row changed.
    #[props(default)]
    on_toggle: EventHandler<(String, bool)>,
    /// Open state changed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
) -> Element {
    let mut m = use_signal(|| {
        let mut x = Menu::new(items.clone());
        x.empty = empty.clone();
        x.searchable = searchable;
        if open {
            x.open_menu(ClockMs(0), false, false);
            if let Some(h) = highlight {
                x.hover(0, Some(h), ClockMs(0));
                if matches!(
                    x.rows(0).get(h).map(|r| &r.kind),
                    Some(ItemKind::Submenu(_))
                ) {
                    x.key(MenuKey::Right, ClockMs(0));
                }
            }
        }
        x
    });
    let g0 = use_glide();
    let g1 = use_glide();
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut point = use_signal(|| None::<(f64, f64)>);
    // Only a menu the user opened gets the click-outside layer and takes focus. A menu that
    // starts open (`open: true`, previews, review sheets) must not cover the page: it may be
    // static HTML that never hydrates, or one of several on the same page.
    let mut armed = use_signal(|| false);
    let st = m.read().clone();
    let glides = [g0, g1];
    let sync = move || {
        let s = m.peek();
        for (l, g) in glides.iter().enumerate() {
            let t = s.levels.get(l).and_then(|x| x.target);
            let kb = s.levels.get(l).map(|x| x.jump).unwrap_or(true);
            g.go(t, kb);
        }
    };
    let outcome = move |o: MenuOutcome| match o {
        MenuOutcome::Activated(id) => {
            on_action.call(id);
            on_open_change.call(false);
        }
        MenuOutcome::Toggled(id, v) => on_toggle.call((id, v)),
        MenuOutcome::Closed { restore } => {
            on_open_change.call(false);
            if restore && let Some(t) = trig.peek().clone() {
                spawn(async move {
                    let _ = t.set_focus(true).await;
                });
            }
        }
        MenuOutcome::None => {}
    };
    let w = width
        .map(|w| format!("min-width:{w}px"))
        .unwrap_or_default();
    let pos = match (*point.read(), context) {
        (Some((x, y)), true) => format!("position:fixed;left:{x:.0}px;top:{y:.0}px"),
        _ => String::new(),
    };
    let levels = st.depth() + 1;
    let panel = rsx! {
        if st.open {
            if *armed.read() {
                div { class: "oxyn-backdrop", onpointerdown: move |_| { let o = m.write().close(false); outcome(o) },
                    oncontextmenu: move |e| { e.prevent_default(); let o = m.write().close(false); outcome(o) } }
            }
            div { class: cls(&["oxyn-menu", &side, &align, &density, if context { "ctx" } else { "" }]), style: "{w};{pos}",
                role: "menu", "aria-label": if let Some(t) = &title { "{t}" } else { "{trigger}" }, tabindex: "-1",
                onmounted: move |e| { g0.set_root(e.data()); if *armed.peek() { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); } },
                onkeydown: move |e: KeyboardEvent| if let Some(k) = menu_key(&e.key()) {
                    if k != MenuKey::Tab { e.prevent_default(); }
                    let o = m.write().key(k, now);
                    sync();
                    outcome(o);
                },
                if let Some(t) = title.clone() { div { class: "mtitle", role: "presentation", "{t}" } }
                if searchable { div { class: "filter", Icon { name: "search", size: 13.0 } span { class: if st.query.is_empty() { "q ph" } else { "q" }, if st.query.is_empty() { "Type to filter" } else { "{st.query}" } } } }
                for l in 0..levels {
                    {
                        let rows = st.rows(l);
                        let g = glides[l.min(1)];
                        let hl = st.levels.get(l).and_then(|x| x.target);
                        let danger_hl = hl.and_then(|i| rows.get(i)).is_some_and(|r| r.danger);
                        let sub_at = if l > 0 { st.path.get(l - 1).copied() } else { None };
                        let body = rsx! {
                            span { class: cls(&[&g.class("hl"), if danger_hl { "danger" } else { "" }]), style: "{g.style()}", "aria-hidden": "true" }
                            if rows.is_empty() { div { class: "empty", role: "presentation", "{st.empty}" } }
                            for (i, it) in rows.iter().enumerate() {
                                {
                                    let on = hl == Some(i);
                                    let has_sub = matches!(it.kind, ItemKind::Submenu(_));
                                    let open_sub = has_sub && st.path.get(l) == Some(&i);
                                    match it.kind {
                                        ItemKind::Divider => rsx! { div { class: "div", role: "separator" } },
                                        ItemKind::Label => rsx! { div { class: "lbl", role: "presentation", "{it.label}" } },
                                        _ => {
                                            let keys = it.shortcut.clone();
                                            let checked = it.checked();
                                            let radio = matches!(it.kind, ItemKind::Radio { .. });
                                            rsx! {
                                                div { class: cls(&["row", if it.danger { "danger" } else { "" }, if it.disabled { "disabled" } else { "" }, if on && !g.measured() { "hl-self" } else { "" }, if open_sub { "subopen" } else { "" }]),
                                                    role: "{it.role()}", tabindex: "-1", "aria-disabled": "{it.disabled}",
                                                    "aria-checked": if let Some(c) = checked { "{c}" },
                                                    "aria-haspopup": if has_sub { "menu" }, "aria-expanded": if has_sub { "{open_sub}" },
                                                    onmounted: move |e| g.set_item(i, e.data()),
                                                    onpointerenter: move |_| {
                                                        m.write().hover(l, Some(i), now);
                                                        if has_sub { m.write().key(MenuKey::Right, now); }
                                                        sync();
                                                    },
                                                    onclick: move |_| { let o = m.write().activate(l, i, now); sync(); outcome(o) },
                                                    span { class: "lead",
                                                        if let Some(c) = checked {
                                                            if c { svg { class: if radio { "chk radio" } else { "chk" }, view_box: "0 0 16 16", width: "14", height: "14",
                                                                if radio { circle { cx: "8", cy: "8", r: "3" } } else { path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } } }
                                                        } else if let Some(lg) = it.logo.clone() { if crate::brand::has_brand(&lg) { span { class: "logo brand", "aria-hidden": "true", crate::brand::BrandIcon { name: lg.clone(), size: 13.0, coloured: true } } } else { span { class: "logo", "aria-hidden": "true", "{lg}" } } }
                                                        else if let Some(ic) = it.icon.clone() { Icon { name: ic, size: 14.0 } }
                                                    }
                                                    span { class: "txt", span { class: "lb", "{it.label}" } if let Some(h) = it.hint.clone() { span { class: "ds", "{h}" } } }
                                                    if let Some(k) = keys { Keys { keys: k } }
                                                    if has_sub { span { class: "sub", Icon { name: "chevron-right", size: 12.0, stroke: 2.0 } } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        };
                        if l == 0 {
                            rsx! { {body} }
                        } else {
                            rsx! {
                                div { class: "oxyn-menu sub", role: "menu", style: "--row:{sub_at.unwrap_or(0)}",
                                    onmounted: move |e| g.set_root(e.data()),
                                    {body}
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    if context {
        return rsx! {
            div { class: "oxyn-ctx", tabindex: "0", "aria-haspopup": "menu",
                onmounted: move |e| trig.set(Some(e.data())),
                oncontextmenu: move |e: MouseEvent| {
                    e.prevent_default();
                    let c = e.client_coordinates();
                    point.set(Some((c.x, c.y)));
                    m.write().open_at(c.x as f32, c.y as f32, now, false);
                    armed.set(true); on_open_change.call(true);
                },
                onkeydown: move |e: KeyboardEvent| if (e.key() == Key::F10 && e.modifiers().shift()) || e.key() == Key::ContextMenu {
                    e.prevent_default();
                    point.set(None);
                    m.write().open_menu(now, true, false);
                    sync();
                    armed.set(true); on_open_change.call(true);
                },
                {children}
                {panel}
            }
        };
    }
    let o = st.open;
    rsx! {
        span { class: "oxyn-ddwrap",
            button { class: "oxyn-btn secondary sm oxyn-ddtrig", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{o}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { let was = m.peek().open; if was { let x = m.write().close(false); outcome(x) } else { m.write().open_menu(now, false, false); armed.set(true); on_open_change.call(true) } },
                onkeydown: move |e: KeyboardEvent| if !m.peek().open && matches!(e.key(), Key::ArrowDown | Key::ArrowUp | Key::Enter) || (!m.peek().open && e.key() == Key::Character(" ".into())) {
                    e.prevent_default();
                    if let Some(k) = menu_key(&e.key()) { m.write().key(k, now); sync(); armed.set(true); on_open_change.call(true); }
                },
                if let Some(i) = icon.clone() { Icon { name: i, size: 14.0 } }
                span { class: "lb", "{trigger}" }
                span { class: "chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
            }
            {panel}
        }
    }
}

/// Spoken shortcut for docs ("Command Shift K").
pub fn spoken_shortcut(s: &str) -> String {
    parse_shortcut(s, Platform::Mac)
        .iter()
        .map(|c| {
            c.iter()
                .map(|k| k.name.clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(", then ")
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
