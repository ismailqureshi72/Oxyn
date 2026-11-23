//! Buttons: `Button`, `IconButton`, `ActionButton`, `SplitButton`, `ButtonGroup`,
//! `FloatingButtonGroup`, `ExpandingButtonGroup`.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        button::{
            ActionButton as ActionModel, ActionPhase, ButtonSize, ButtonVariant, ExpandingGroup,
            GroupItem, LoadingStyle, SplitButton as SplitModel, hidden_dividers,
        },
        display::{Platform, parse_shortcut},
        menu::{Menu, MenuItem},
        nav::{NavKey, step},
    },
};

use super::{Glide, cls, nav_key, use_glide};
use crate::composer::icons::Icon;

/// The arc spinner used inside buttons and rows.
#[component]
pub(crate) fn Spin(#[props(default = 14.0)] size: f32) -> Element {
    rsx! { span { class: "oxyn-spn", style: "--s:{size}px", "aria-hidden": "true" } }
}

/// Shortcut keycaps inside a button or tooltip.
#[component]
pub(crate) fn Keys(keys: String, #[props(default)] other: bool) -> Element {
    let p = if other {
        Platform::Other
    } else {
        Platform::Mac
    };
    let chords = parse_shortcut(&keys, p);
    rsx! {
        span { class: "oxyn-keys", "aria-hidden": "true",
            for (n, chord) in chords.iter().enumerate() {
                if n > 0 { span { class: "then", "then" } }
                for k in chord.iter() { kbd { title: "{k.name}", "{k.glyph}" } }
            }
        }
    }
}

/// A button: primary, secondary, ghost, destructive (+ outline, link, quiet, success); five sizes;
/// loading with spinner + shimmer band, spinner only, or progress fill; disabled with a reason;
/// hold-to-confirm; a shortcut chip.
#[component]
pub fn Button(
    /// Label.
    label: String,
    /// `primary` · `secondary` · `ghost` · `destructive` · `outline` · `link` · `quiet` · `success`.
    #[props(default = "primary".to_string())]
    variant: String,
    /// `xs` · `sm` · `md` · `lg` · `xl`.
    #[props(default = "md".to_string())]
    size: String,
    /// Busy: keeps its width, ignores presses, `aria-busy`.
    #[props(default)]
    loading: bool,
    /// `shimmer` (spinner + band) · `spinner` · `progress`.
    #[props(default = "shimmer".to_string())]
    loading_style: String,
    /// Progress 0..1 for the `progress` loading style.
    #[props(default)]
    progress: Option<f32>,
    /// Leading icon.
    #[props(default)]
    icon: Option<String>,
    /// Trailing icon.
    #[props(default)]
    icon_right: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Why it is disabled (tooltip and `aria-describedby` text).
    #[props(default)]
    reason: Option<String>,
    /// Press and hold to confirm (destructive actions).
    #[props(default)]
    hold: bool,
    /// Shortcut chip ("Mod+Enter").
    #[props(default)]
    shortcut: Option<String>,
    /// Stretch to the container.
    #[props(default)]
    full: bool,
    /// Review sheets: force `hover`, `pressed` or `focus`.
    #[props(default)]
    state: Option<String>,
    /// Pressed (or the hold completed).
    #[props(default)]
    onclick: EventHandler<()>,
) -> Element {
    let v = ButtonVariant::from_id(&variant).unwrap_or_default();
    let sz = ButtonSize::from_id(&size).unwrap_or_default();
    let ls = match loading_style.as_str() {
        "spinner" => LoadingStyle::Spinner,
        "progress" => LoadingStyle::Progress,
        _ => LoadingStyle::SpinnerShimmer,
    };
    let mut holding = use_signal(|| false);
    let inert = loading || disabled;
    let forced = state.clone().unwrap_or_default();
    let class = cls(&[
        "oxyn-btn",
        v.id(),
        sz.id(),
        if loading { "loading" } else { "" },
        if loading { ls.id() } else { "" },
        if hold { "hold" } else { "" },
        if *holding.read() { "holding" } else { "" },
        if full { "full" } else { "" },
        &forced,
    ]);
    let icon_size = sz.icon();
    let fill = progress.unwrap_or(0.0).clamp(0.0, 1.0) * 100.0;
    let label_c = label.clone();
    rsx! {
        button {
            class: "{class}",
            r#type: "button",
            disabled: disabled && reason.is_none(),
            "aria-disabled": "{inert}",
            "aria-busy": "{loading}",
            title: if let Some(r) = &reason { "{r}" },
            "aria-keyshortcuts": if let Some(s) = &shortcut { "{oxyn_core::primitives::display::aria_keyshortcuts(s, Platform::Mac)}" },
            onclick: move |_| if !inert && !hold { onclick.call(()) },
            onpointerdown: move |_| if hold && !inert { holding.set(true) },
            onpointerup: move |_| holding.set(false),
            onpointerleave: move |_| holding.set(false),
            onkeydown: move |e: KeyboardEvent| if hold && !inert && e.key() == Key::Character(" ".into()) { e.prevent_default(); holding.set(true) },
            onkeyup: move |_| holding.set(false),
            if hold { span { class: "hold-fill", onanimationend: move |_| { holding.set(false); onclick.call(()) } } }
            if loading && ls == LoadingStyle::Progress { span { class: "prog", style: "width:{fill:.0}%" } }
            if loading && ls != LoadingStyle::Progress {
                Spin { size: icon_size }
            } else if let Some(i) = icon.clone() {
                Icon { name: i, size: icon_size }
            }
            span { class: "lb", "data-text": "{label_c}", if hold && *holding.read() { "Keep holding…" } else { "{label}" } }
            if let Some(i) = icon_right.clone() { Icon { name: i, size: icon_size } }
            if let Some(s) = shortcut.clone() { Keys { keys: s } }
            if let Some(r) = reason.clone() { span { class: "sr", "{r}" } }
        }
    }
}

/// A plain list of menu rows with the shared highlight; used by split buttons and group menus.
#[component]
pub(crate) fn MenuList(
    items: Vec<MenuItem>,
    glide: Glide,
    level: usize,
    labelled: String,
    on_pick: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let n = items.len();
    let items_k = items.clone();
    rsx! {
        div { class: "oxyn-menu plain", role: "menu", "aria-label": "{labelled}", tabindex: "-1",
            onmounted: move |e| glide.set_root(e.data()),
            onkeydown: move |e: KeyboardEvent| {
                let cur = *glide.at.peek();
                let k = match e.key() { Key::ArrowDown => Some(NavKey::Next), Key::ArrowUp => Some(NavKey::Prev), Key::Home => Some(NavKey::First), Key::End => Some(NavKey::Last), _ => None };
                if let Some(k) = k { e.prevent_default(); glide.go(step(cur, k, n, true, |i| items_k[i].focusable()), true); }
                match e.key() {
                    Key::Enter => if let Some(i) = cur { on_pick.call(items_k[i].id.clone()) },
                    Key::Escape | Key::Tab => on_close.call(()),
                    _ => {}
                }
            },
            onpointerleave: move |_| glide.go(None, false),
            span { class: "{glide.class(\"hl\")}", style: "{glide.style()}", "aria-hidden": "true" }
            for (i, it) in items.iter().enumerate() {
                {
                    let id = it.id.clone();
                    let on = *glide.at.read() == Some(i);
                    rsx! {
                        div { class: cls(&["row", if on && !glide.measured() { "hl-self" } else { "" }, if on { "on" } else { "" }]),
                            role: "menuitem", tabindex: "-1", "data-level": "{level}",
                            onmounted: move |e| glide.set_item(i, e.data()),
                            onpointerenter: move |_| glide.go(Some(i), false),
                            onclick: move |_| on_pick.call(id.clone()),
                            "{it.label}"
                        }
                    }
                }
            }
        }
    }
}

/// Joined button group (Arc): hairline dividers, one hover highlight under all segments, a
/// pressed / selected segment, horizontal or vertical, an attached menu. Oxyn plus: overflow to a
/// menu (`max_visible`), per-segment counts, a dense IDE size.
#[component]
pub fn ButtonGroup(
    /// Segments.
    items: Vec<GroupItem>,
    /// `outline` · `solid`.
    #[props(default = "outline".to_string())]
    variant: String,
    /// `ide` (24) · `sm` (28) · `md` (32).
    #[props(default = "md".to_string())]
    size: String,
    /// Vertical.
    #[props(default)]
    vertical: bool,
    /// Icons only (labels become tooltips).
    #[props(default)]
    collapse_labels: bool,
    /// Selected segment (toggle groups).
    #[props(default)]
    selected: Option<String>,
    /// Segments that fit before the rest go to an overflow menu.
    #[props(default)]
    max_visible: Option<usize>,
    /// Attached menu at the end of the row.
    #[props(default)]
    menu: Vec<MenuItem>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Review sheets: force the highlight on a segment.
    #[props(default)]
    highlight: Option<usize>,
    /// A segment or menu item ran.
    #[props(default)]
    on_action: EventHandler<String>,
) -> Element {
    let glide = use_glide();
    let mut menu_open = use_signal(|| false);
    let menu_glide = use_glide();
    let shown = max_visible.unwrap_or(items.len()).min(items.len());
    let (vis, rest) = items.split_at(shown);
    let mut overflow: Vec<MenuItem> = rest
        .iter()
        .map(|g| MenuItem::action(&g.id, &g.label))
        .collect();
    overflow.extend(menu.iter().cloned());
    let hl = highlight.or(*glide.at.read());
    let hidden = hidden_dividers(hl, vis.len() + usize::from(!overflow.is_empty()));
    let n = vis.len();
    let en: Vec<bool> = vis.iter().map(|g| !g.disabled && !disabled).collect();
    let en_k = en.clone();
    let class = cls(&[
        "oxyn-bgroup",
        &variant,
        &size,
        if vertical { "vertical" } else { "" },
        if collapse_labels { "icons" } else { "" },
    ]);
    rsx! {
        div { class: "{class}", role: "group", "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            onmounted: move |e| glide.set_root(e.data()),
            onpointerleave: move |_| glide.go(None, false),
            onkeydown: move |e: KeyboardEvent| {
                if let Some(k) = nav_key(&e.key(), vertical) {
                    e.prevent_default();
                    let to = step(*glide.at.peek(), k, n, true, |i| en_k[i]);
                    glide.go(to, true);
                }
            },
            span { class: "{glide.class(\"hl\")}", style: "{glide.style()}", "aria-hidden": "true" }
            for (i, it) in vis.iter().enumerate() {
                {
                    let id = it.id.clone();
                    let sel = selected.as_deref() == Some(it.id.as_str());
                    let on = hl == Some(i);
                    let tab = if glide.at.read().unwrap_or(0) == i { "0" } else { "-1" };
                    rsx! {
                        if i > 0 { span { class: if hidden[i - 1] { "hair off" } else { "hair" }, "aria-hidden": "true" } }
                        button { class: cls(&["seg", if sel { "sel" } else { "" }, if on && !glide.measured() { "hl-self" } else { "" }]), r#type: "button",
                            tabindex: "{tab}", disabled: !en[i], title: if collapse_labels { "{it.label}" },
                            "aria-label": if collapse_labels { "{it.label}" },
                            "aria-pressed": if selected.is_some() { "{sel}" },
                            onmounted: move |e| glide.set_item(i, e.data()),
                            onpointerenter: move |_| glide.go(Some(i), false),
                            onfocus: move |_| glide.go(Some(i), true),
                            onclick: move |_| on_action.call(id.clone()),
                            if let Some(ic) = it.icon.clone() { Icon { name: ic, size: 14.0 } }
                            if !collapse_labels { span { class: "lb", "{it.label}" } }
                            if let Some(c) = it.count { span { class: "cnt", "{c}" } }
                        }
                    }
                }
            }
            if !overflow.is_empty() {
                span { class: if hidden.last().copied().unwrap_or(false) { "hair off" } else { "hair" }, "aria-hidden": "true" }
                div { class: "menu-wrap",
                    button { class: "seg more", r#type: "button", "aria-label": "More", "aria-haspopup": "menu", "aria-expanded": "{menu_open}",
                        onclick: move |_| { let o = !*menu_open.peek(); menu_open.set(o); },
                        Icon { name: "more", size: 14.0, stroke: 2.4 }
                    }
                    if *menu_open.read() {
                        MenuList { items: overflow, glide: menu_glide, level: 0, labelled: "More".to_string(),
                            on_pick: move |id: String| { menu_open.set(false); on_action.call(id) },
                            on_close: move |_| menu_open.set(false),
                        }
                    }
                }
            }
        }
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
