//! Navigation and layout (plan #27): `Tabs`, `Breadcrumb`, `Accordion`, `ResizablePanels`,
//! `Drawer`, `HoverCard`, `TreeView`, `ReorderableList`, `ToggleGroup`, `ChipGroup`,
//! `RadioCards`, `Rating`, `CopyButton`, `Reactions`, `ExpandableText`, `EmptyPreset`, `Dock`,
//! `TocProgress`, `PageHeader`.
//!
//! Logic lives in `oxyn_core::primitives::layout` and `chrome`. Timers (hover-card delays, the
//! copy hold) are CSS animations whose `animationend` advances the model, so no host clock is
//! needed. Behaviour follows the Arc UI docs (rebuilt; no source used).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        chrome::{ReactionCount, ReactionOption, Reactions as ReactionsModel},
        layout::{
            Accordion as AccModel, Activation, ChipGroup as ChipModel, CopyState, Crumb, CrumbSlot,
            Dock as DockModel, DockItem, EmptyPreset as PresetModel, HoverIntent, Pane, RadioCard,
            Rating as RatingModel, RatingKey, Resizable, Tab, Tabs as TabsModel, TocItem,
            ToggleGroup as ToggleModel, ToggleItem, Tree, TreeKey, TreeNode, crumb_slots,
            drag_shift, drag_target, handle_label, header_folded, initials, move_item,
            overflow_edges, radio_step, time_left, toc_progress,
        },
        nav::NavKey,
    },
};

use dioxus::html::geometry::PixelsVector2D;

#[cfg(oxyn_pro)]
use super::{EmptyState, Sheet};
use super::{cls, nav_key, use_glide};
use crate::composer::icons::Icon;

fn sl(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .take(16)
        .collect::<String>()
        .to_lowercase()
}

fn ms_now() -> ClockMs {
    ClockMs(0)
}

// ─────────────────────────────────────────── Tabs ──

/// Tabs: a scrollable tab strip with a gliding selection pill and panels that slide in the
/// direction of travel. `variant`: `pill` · `underline` · `enclosed` (Oxyn plus) · `terminal`
/// (closable tabs with busy dots and a "+"). Arrow keys select (or only focus with
/// `manual: true`), Home / End jump, overflow shows labelled edge buttons. Panels come from
/// `panels` (one string per tab) or from `children` rendered under the strip.
#[component]
pub fn Tabs(
    /// Tabs.
    tabs: Vec<Tab>,
    /// Selected value (initial).
    #[props(default)]
    value: Option<String>,
    /// Panel text per tab (same order).
    #[props(default)]
    panels: Vec<String>,
    /// `pill` · `underline` · `enclosed` · `terminal`.
    #[props(default = "pill".to_string())]
    variant: String,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Manual activation (arrows only move focus).
    #[props(default)]
    manual: bool,
    /// Tab list label.
    #[props(default = "Tabs".to_string())]
    label: String,
    /// Fill the width (equal tabs).
    #[props(default)]
    full: bool,
    /// Terminal variant: show the "+" button.
    #[props(default)]
    addable: bool,
    /// Selection changed.
    #[props(default)]
    on_change: EventHandler<String>,
    /// A tab closed (value).
    #[props(default)]
    on_close: EventHandler<String>,
    /// Extra content under the strip.
    #[props(default)]
    children: Element,
) -> Element {
    let mut m = use_signal(|| {
        let mut t = TabsModel::new(tabs.clone(), value.as_deref());
        if manual {
            t.activation = Activation::Manual;
        }
        t
    });
    let glide = use_glide();
    let mut strip = use_signal(|| None::<Rc<MountedData>>);
    let mut edges = use_signal(|| (false, false));
    let mut added = use_signal(|| 0u32);
    let measure = move || {
        if let Some(s) = strip.peek().clone() {
            spawn(async move {
                if let (Ok(o), Ok(sz), Ok(r)) = (
                    s.get_scroll_offset().await,
                    s.get_scroll_size().await,
                    s.get_client_rect().await,
                ) {
                    edges.set(overflow_edges(
                        o.x as f32,
                        r.size.width as f32,
                        sz.width as f32,
                    ));
                }
            });
        }
    };
    let scroll_by = move |dx: f64| {
        if let Some(s) = strip.peek().clone() {
            spawn(async move {
                if let Ok(o) = s.get_scroll_offset().await {
                    let _ = s
                        .scroll(PixelsVector2D::new(o.x + dx, 0.0), ScrollBehavior::Smooth)
                        .await;
                }
            });
        }
    };
    let t = m.read().clone();
    let sel = t.selected();
    let stop = t.tab_stop();
    let uid = sl(&label);
    let term = variant == "terminal";
    let panel_text = sel.and_then(|i| {
        let v = &t.tabs[i].value;
        tabs.iter()
            .position(|x| &x.value == v)
            .and_then(|j| panels.get(j).cloned())
    });
    let (el, er) = edges();
    rsx! {
        div { class: cls(&["oxyn-tabs", &variant, &size, if full { "full" } else { "" }]),
            div { class: "tbar",
                if el { button { class: "edge l", r#type: "button", "aria-label": "Scroll tabs left", tabindex: "-1", onclick: move |_| scroll_by(-160.0), Icon { name: "chevron-left", size: 14.0 } } }
                div { class: "strip", role: "tablist", "aria-label": "{label}",
                    onmounted: move |e| { strip.set(Some(e.data())); glide.set_root(e.data()); measure(); if let Some(i) = m.peek().selected() { glide.go(Some(i), true); } },
                    onscroll: move |_| measure(),
                    onkeydown: move |e: KeyboardEvent| {
                        if let Some(nk) = nav_key(&e.key(), false) {
                            e.prevent_default();
                            let r = m.write().key(nk);
                            if let Some(v) = r { on_change.call(v) }
                            if let Some(i) = m.peek().selected() { glide.go(Some(i), true) }
                        } else if matches!(e.key(), Key::Enter) || e.key() == Key::Character(" ".into()) {
                            e.prevent_default();
                            if let Some(v) = m.write().activate() { on_change.call(v) }
                            if let Some(i) = m.peek().selected() { glide.go(Some(i), true) }
                        } else if term && (e.key() == Key::Delete) {
                            let ts = m.peek().tab_stop(); if let Some(i) = ts { let v = m.peek().tabs[i].value.clone(); m.write().close(i); on_close.call(v); }
                        }
                    },
                    span { class: "{glide.class(\"pill\")}", style: "{glide.style()}", "aria-hidden": "true" }
                    for (i, tb) in t.tabs.iter().cloned().enumerate() {
                        div { key: "{tb.value}", class: cls(&["tw", if sel == Some(i) { "on" } else { "" }]),
                            onmounted: move |e| glide.set_item(i, e.data()),
                            button { class: "tab", r#type: "button", role: "tab", id: "oxyn-tab-{uid}-{tb.value}",
                                "aria-selected": "{sel == Some(i)}", "aria-controls": "oxyn-tp-{uid}", disabled: tb.disabled,
                                tabindex: if stop == Some(i) { "0" } else { "-1" },
                                onclick: move |_| { if let Some(v) = m.write().select(i) { on_change.call(v) } glide.go(Some(i), false) },
                                if tb.busy { span { class: "busy", "aria-hidden": "true" } }
                                if let Some(ic) = tb.icon.clone() { Icon { name: ic, size: 14.0 } }
                                span { class: "lb", "{tb.label}" }
                                if let Some(n) = tb.count { span { class: "ct", key: "{n}", "{n}" } }
                            }
                            if tb.closable {
                                button { class: "cl", r#type: "button", tabindex: "-1", "aria-label": "Close {tb.label}",
                                    onclick: move |_| { let v = m.peek().tabs.get(i).map(|x| x.value.clone()).unwrap_or_default(); m.write().close(i); on_close.call(v.clone()); let s = m.peek().selected(); glide.go(s, true); },
                                    Icon { name: "x", size: 11.0, stroke: 2.0 }
                                }
                            }
                        }
                    }
                }
                if er { button { class: "edge r", r#type: "button", "aria-label": "Scroll tabs right", tabindex: "-1", onclick: move |_| scroll_by(160.0), Icon { name: "chevron-right", size: 14.0 } } }
                if term && addable {
                    button { class: "add", r#type: "button", "aria-label": "New terminal",
                        onclick: move |_| { added += 1; let n = m.peek().tabs.len() + 1; let v = m.write().push(Tab::new(&format!("t{}", added()), &format!("zsh {n}")).closable().icon("prompt")); on_change.call(v); },
                        Icon { name: "plus", size: 14.0 }
                    }
                }
            }
            if let Some(p) = panel_text {
                for _k in std::iter::once(t.version) {
                    div { key: "{_k}", class: "tpanel {t.travel.id()}", role: "tabpanel", id: "oxyn-tp-{uid}", tabindex: "0",
                        "aria-labelledby": if let Some(i) = sel { "oxyn-tab-{uid}-{t.tabs[i].value}" },
                        if term { pre { class: "term", "{p}" } } else { p { "{p}" } }
                    }
                }
            }
            {children}
        }
    }
}

// ───────────────────────────────────── CopyButton ──

/// Copy button: copies `value` and crossfades its icon and label to Copied (or Failed), resetting
/// after 1.5 s; the label cell reserves its widest state so nothing shifts. A polite status region
/// announces "Copy: Copied". The clipboard write is the host's (`data-copy` is set for the site's
/// handler, and `on_copy` fires with the value). Oxyn plus: `variant` `solid`, a `fail` demo
/// switch, and a `hint` tooltip line.
#[component]
pub fn CopyButton(
    /// Text to copy.
    value: String,
    /// Idle label.
    #[props(default = "Copy".to_string())]
    label: String,
    /// Icon only.
    #[props(default)]
    icon_only: bool,
    /// `outline` · `plain` · `solid`.
    #[props(default = "outline".to_string())]
    variant: String,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Simulate a failed copy.
    #[props(default)]
    fail: bool,
    /// Copy pressed (value).
    #[props(default)]
    on_copy: EventHandler<String>,
) -> Element {
    let mut st = use_signal(CopyState::default);
    let mut gn = use_signal(|| 0u32);
    let s = st();
    let words = [label.as_str(), "Copied", "Failed"];
    rsx! {
        button { class: cls(&["oxyn-copy", &variant, &size, s.id(), if icon_only { "io" } else { "" }]), r#type: "button", disabled,
            "aria-label": "{label}", "data-copy": if !fail { "{value}" },
            onclick: move |_| { st.set(if fail { CopyState::Failed(ms_now()) } else { CopyState::Copied(ms_now()) }); gn += 1; on_copy.call(value.clone()) },
            span { class: "cic", "aria-hidden": "true",
                span { class: "i idle", Icon { name: "copy", size: 14.0 } }
                span { class: "i ok", svg { view_box: "0 0 24 24", width: "14", height: "14", path { d: "m5 12.5 4.5 4.5L19 7.5", "pathLength": "1" } } }
                span { class: "i bad", Icon { name: "x", size: 14.0 } }
            }
            if !icon_only {
                span { class: "clb", "aria-hidden": "true",
                    for (k, w) in words.iter().enumerate() { span { key: "{k}", class: cls(&["w", if k == match s { CopyState::Idle => 0, CopyState::Copied(_) => 1, CopyState::Failed(_) => 2 } { "on" } else { "" }]), "{w}" } }
                }
            }
            span { class: "sr", role: "status", "{s.announce(&label)}" }
            if s != CopyState::Idle { span { key: "{gn}", class: "oxyn-timer", style: "animation-duration:1500ms", onanimationend: move |_| st.set(CopyState::Idle) } }
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
