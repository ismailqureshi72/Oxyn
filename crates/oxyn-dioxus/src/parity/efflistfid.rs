//! Thinking Selector family, variant "Effort list" ([`EffortList`]): a trigger chip with the current
//! level ("High ^") that opens a plain list of levels above it, the chosen one checked. Models:
//! `oxyn_core::parity::{thinkfid, effmenufid}`. Styles: `efflistfid.css` (classes `oxel-*`).

use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
use oxyn_core::parity::{
    effmenufid::menu_move,
    thinkfid::{EffortLevel, effort_said, effort_scale_short},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A page-unique id prefix for one menu instance (for `aria-activedescendant`).
pub(crate) fn use_uid(p: &'static str) -> String {
    use_hook(|| format!("{p}{}", NEXT.fetch_add(1, Ordering::Relaxed)))
}

pub(crate) fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// The menu key a list understands: ±1, ends, or `None`.
pub(crate) fn menu_key(k: &Key) -> Option<i64> {
    match k {
        Key::ArrowUp => Some(-1),
        Key::ArrowDown => Some(1),
        Key::Home => Some(i64::MIN),
        Key::End => Some(i64::MAX),
        _ => None,
    }
}

pub(crate) fn check_glyph() -> Element {
    rsx! {
        svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "m5 12.5 4.5 4.5L19 7.5" } }
    }
}

pub(crate) fn chevron(up: bool) -> Element {
    let d = if up { "m6 15 6-6 6 6" } else { "m6 9 6 6 6-6" };
    rsx! {
        svg { width: "13", height: "13", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } }
    }
}

/// Thinking Selector – Effort list: a chip showing the level that opens a plain menu of levels
/// above it, the chosen one checked. Up/Down move, Enter picks, Escape closes back to the chip.
#[component]
pub fn EffortList(
    /// The scale (Minimal, Low, Medium, High when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Controlled level.
    #[props(default)]
    value: Option<usize>,
    /// Starting level when uncontrolled (the last when `None`).
    #[props(default)]
    default_value: Option<usize>,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// The menu's heading.
    #[props(default = "Thinking effort".to_string())]
    heading: String,
    /// The level changed.
    #[props(default)]
    on_change: EventHandler<usize>,
) -> Element {
    let levels = levels.unwrap_or_else(effort_scale_short);
    let n = levels.len().max(1);
    let mut own = use_signal(move || default_value.unwrap_or(n - 1).min(n - 1));
    let cur = value.unwrap_or(own()).min(n - 1);
    let mut open = use_signal(move || default_open);
    let mut hi = use_signal(move || cur);
    let mut said = use_signal(String::new);
    let mut chip = use_signal(|| None::<Rc<MountedData>>);
    let uid = use_uid("oxel");
    let all = levels.clone();
    let pick = use_callback(move |i: usize| {
        own.set(i);
        said.set(effort_said(&all[i]));
        on_change.call(i);
        open.set(false);
        focus(chip.peek().clone());
    });
    // The menu takes focus when it mounts after a user opened it (not for a preview).
    let mut want = use_signal(|| false);
    let mut show = move |at: usize| {
        hi.set(at);
        want.set(true);
        open.set(true);
    };
    let active = format!("{uid}-{}", hi());
    let is_open = open();

    rsx! {
        div { class: "oxel", "data-open": "{is_open}",
            if is_open {
                div { class: "oxel-pop",
                    p { class: "oxel-h", id: "{uid}-h", "{heading}" }
                    ul { class: "oxel-menu", role: "menu", tabindex: "-1", "aria-labelledby": "{uid}-h",
                        "aria-activedescendant": "{active}",
                        onmounted: move |e| {
                            let d = e.data();
                            if *want.peek() { want.set(false); focus(Some(d)); }
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            let k = e.key();
                            if let Some(d) = menu_key(&k) {
                                e.prevent_default();
                                hi.set(menu_move(hi(), d, n));
                            } else if k == Key::Enter || k == Key::Character(" ".into()) {
                                e.prevent_default();
                                pick.call(hi());
                            } else if k == Key::Escape || k == Key::Tab {
                                if k == Key::Escape { e.prevent_default(); }
                                open.set(false);
                                focus(chip.peek().clone());
                            }
                        },
                        for (i, l) in levels.iter().enumerate() {
                            li { key: "{i}", "data-k": "{i}", id: "{uid}-{i}", class: "oxel-item", role: "menuitemradio",
                                "aria-checked": "{i == cur}", "data-hi": "{i == hi()}",
                                onpointerenter: move |_| hi.set(i),
                                onclick: move |_| pick.call(i),
                                span { "{l.label}" }
                                if i == cur { span { class: "oxel-check", {check_glyph()} } }
                            }
                        }
                    }
                }
            }
            button { class: "oxel-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{is_open}",
                "aria-label": "Thinking effort: {levels[cur].label}",
                onmounted: move |e| chip.set(Some(e.data())),
                onclick: move |_| if open() { open.set(false) } else { show(cur) },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    Key::ArrowUp | Key::ArrowDown if !open() => { e.prevent_default(); show(cur); }
                    Key::Escape if open() => { e.prevent_default(); open.set(false); }
                    _ => {}
                },
                span { "{levels[cur].label}" } {chevron(true)} }
            div { class: "oxel-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
