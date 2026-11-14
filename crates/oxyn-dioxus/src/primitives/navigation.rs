//! Navigation and layout (plan #27): `Tabs`, `Breadcrumb`, `Accordion`, `ResizablePanels`,
//! `Drawer`, `HoverCard`, `TreeView`, `ReorderableList`, `ToggleGroup`, `ChipGroup`,
//! `RadioCards`, `Rating`, `CopyButton`, `Reactions`, `ExpandableText`, `EmptyPreset`, `Dock`,
//! `TocProgress`, `PageHeader`.
//!
//! Logic lives in `oxyn_core::primitives::layout` and `chrome`. Timers (hover-card delays, the
//! copy hold) are CSS animations whose `animationend` advances the model, so no host clock is
//! needed. Behaviour follows the Arc UI docs (rebuilt; no source used).

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

use super::{EmptyState, Sheet, cls, nav_key, use_glide};
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

// ───────────────────────────────────── Breadcrumb ──

/// Breadcrumb: a `nav` landmark with an ordered list; the last crumb is the current page
/// (`aria-current`, never a link). New crumbs slide in from a blur. Oxyn plus: `max` collapses
/// the middle behind a "…" button that reveals it, icons on crumbs, and `slash` separators.
#[component]
pub fn Breadcrumb(
    /// Path from root to the current page.
    items: Vec<Crumb>,
    /// Landmark label.
    #[props(default = "Breadcrumb".to_string())]
    label: String,
    /// Collapse beyond this many crumbs (0 = wrap, never collapse).
    #[props(default)]
    max: usize,
    /// `chevron` · `slash`.
    #[props(default = "chevron".to_string())]
    separator: String,
    /// A crumb without href was pressed (index).
    #[props(default)]
    on_select: EventHandler<usize>,
) -> Element {
    let mut expanded = use_signal(|| false);
    let n = items.len();
    let slots = if expanded() {
        crumb_slots(n, 0)
    } else {
        crumb_slots(n, max)
    };
    rsx! {
        nav { class: "oxyn-crumbs", "aria-label": "{label}",
            ol {
                for (k, s) in slots.into_iter().enumerate() {
                    li { key: "{k}-{n}", class: "cr",
                        if k > 0 { span { class: "sep", "aria-hidden": "true", if separator == "slash" { "/" } else { Icon { name: "chevron-right", size: 12.0 } } } }
                        match s {
                            CrumbSlot::More(hidden) => rsx! {
                                button { class: "more", r#type: "button", "aria-label": "Show {hidden.len()} more", "aria-expanded": "false", onclick: move |_| expanded.set(true), "…" }
                            },
                            CrumbSlot::Item(i) => {
                                let c = items[i].clone();
                                let last = i + 1 == n;
                                rsx! {
                                    if last {
                                        span { class: "cur", "aria-current": "page", if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 13.0 } } "{c.label}" }
                                    } else if let Some(h) = c.href.clone() {
                                        a { class: "lnk", href: "{h}", if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 13.0 } } "{c.label}" }
                                    } else {
                                        button { class: "lnk", r#type: "button", onclick: move |_| on_select.call(i), if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 13.0 } } "{c.label}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ────────────────────────────────────── Accordion ──

/// One accordion row.
#[derive(Clone, Debug, PartialEq)]
pub struct AccordionItem {
    /// Trigger label.
    pub title: String,
    /// Panel text.
    pub content: String,
    /// Oxyn plus: a quiet meta on the right ("2 steps", "12s").
    pub meta: Option<String>,
    /// Disabled.
    pub disabled: bool,
}

impl AccordionItem {
    /// A row.
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            meta: None,
            disabled: false,
        }
    }
    /// With meta.
    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.into());
        self
    }
}

/// Accordion: question-and-answer rows. Single-open by default; the open row can close. Panel
/// height grows on a grid-rows transition while the answer slides down out of a blur; ↑ ↓ Home
/// End move between triggers. Oxyn plus: `multiple`, `variant` `separated` / `card`, a meta
/// column and Expand all.
#[component]
pub fn Accordion(
    /// Rows.
    items: Vec<AccordionItem>,
    /// Open on first render (`None` = all closed).
    #[props(default = Some(0))]
    default_open: Option<usize>,
    /// Several open at once.
    #[props(default)]
    multiple: bool,
    /// `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// `plain` · `separated` · `card`.
    #[props(default = "plain".to_string())]
    variant: String,
    /// Expand all / collapse all control (multiple mode).
    #[props(default)]
    controls: bool,
) -> Element {
    let n = items.len();
    let mut m = use_signal(|| {
        let mut a = AccModel::new(n, default_open);
        a.multiple = multiple;
        a.disabled = items
            .iter()
            .enumerate()
            .filter(|(_, x)| x.disabled)
            .map(|(i, _)| i)
            .collect();
        a
    });
    let mut nodes = use_signal(|| vec![None::<Rc<MountedData>>; n]);
    let a = m.read().clone();
    rsx! {
        div { class: cls(&["oxyn-acc", &size, &variant]),
            if controls && multiple {
                div { class: "acc-ctl",
                    button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| m.write().expand_all(), "Expand all" }
                    button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| m.write().collapse_all(), "Collapse all" }
                }
            }
            for (i, it) in items.iter().cloned().enumerate() {
                div { key: "{i}", class: cls(&["row", if a.is_open(i) { "open" } else { "" }]),
                    h3 { class: "hd",
                        button { class: "trg", r#type: "button", id: "oxyn-acc-t{i}", "aria-expanded": "{a.is_open(i)}", "aria-controls": "oxyn-acc-p{i}", disabled: it.disabled,
                            onmounted: move |e| { if let Some(s) = nodes.write().get_mut(i) { *s = Some(e.data()) } },
                            onclick: move |_| m.write().toggle(i),
                            onkeydown: move |e: KeyboardEvent| if let Some(k) = nav_key(&e.key(), true) {
                                e.prevent_default();
                                let j = m.write().key(k); if let Some(j) = j && let Some(Some(d)) = nodes.peek().get(j).cloned() { spawn(async move { let _ = d.set_focus(true).await; }); }
                            },
                            span { class: "tt", "{it.title}" }
                            if let Some(mt) = it.meta.clone() { span { class: "mt", "{mt}" } }
                            span { class: "acc-chev", "aria-hidden": "true", Icon { name: "chevron", size: 14.0 } }
                        }
                    }
                    div { class: "pnl", id: "oxyn-acc-p{i}", role: "region", "aria-labelledby": "oxyn-acc-t{i}",
                        div { class: "pin", div { class: "ans", "{it.content}" } }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────── ResizablePanels ──

/// One pane's content for [`ResizablePanels`].
#[derive(Clone, Debug, PartialEq)]
pub struct PaneView {
    /// Configuration.
    pub pane: Pane,
    /// Title shown in the pane.
    pub title: String,
    /// Lines of content.
    pub lines: Vec<String>,
}

/// Resizable panels: a split with focusable `separator` dividers. Drag 1:1 (pointer capture),
/// ←/→ move 16 px (Shift 64 px), Home / End to limits, Enter collapses or restores the
/// collapsible neighbour, Escape cancels a drag, double-click resets. Squeezed panes fade; a
/// collapsed pane leaves a restore tab. Needs a definite height. Oxyn plus: `vertical`,
/// a size readout and a `handle` style (`line` · `grip`).
#[component]
pub fn ResizablePanels(
    /// Panes.
    panes: Vec<PaneView>,
    /// Group label.
    #[props(default = "Panels".to_string())]
    label: String,
    /// Height px.
    #[props(default = 320.0)]
    height: f32,
    /// Stack vertically.
    #[props(default)]
    vertical: bool,
    /// `line` · `grip`.
    #[props(default = "line".to_string())]
    handle: String,
    /// Show each pane's share.
    #[props(default)]
    readout: bool,
    /// Layout settled (percent shares).
    #[props(default)]
    on_layout_change: EventHandler<Vec<f32>>,
) -> Element {
    let cfg: Vec<Pane> = panes.iter().map(|p| p.pane.clone()).collect();
    let mut m = use_signal(move || {
        let mut r = Resizable::new(cfg, 800.0);
        r.vertical = vertical;
        r
    });
    let mut last = use_signal(|| 0.0f64);
    let r = m.read().clone();
    let axis = if vertical { "v" } else { "h" };
    rsx! {
        div { class: cls(&["oxyn-split", axis, &handle]), role: "group", "aria-label": "{label}", style: "height:{height}px",
            onmounted: move |e| { let d = e.data(); spawn(async move { if let Ok(b) = d.get_client_rect().await { let w = if vertical { b.size.height } else { b.size.width } as f32; let n = m.peek().panes.len().saturating_sub(1) as f32; m.write().set_total(w - 8.0 * n); } }); },
            onpointermove: move |e| { let dg = m.peek().drag.clone(); if let Some((d, _)) = dg {
                let c = e.client_coordinates();
                let p = if vertical { c.y } else { c.x };
                let dx = (p - last()) as f32;
                last.set(p);
                m.write().move_divider(d, dx);
            } },
            onpointerup: move |_| { let e = m.write().end(); if e { on_layout_change.call(m.peek().shares()) } },
            onpointerleave: move |_| { let e = m.write().end(); if e { on_layout_change.call(m.peek().shares()) } },
            for (i, pv) in panes.iter().cloned().enumerate() {
                div { key: "{pv.pane.id}", class: "pw",
                    div { class: cls(&["pane", if r.collapsed[i] { "collapsed" } else { "" }]), id: "oxyn-pane-{pv.pane.id}",
                        style: "flex-basis:{r.sizes[i]:.1}px;--sq:{r.squeeze(i):.2}", "inert": if r.collapsed[i] { "true" },
                        div { class: "pc",
                            div { class: "ph", "{pv.title}" if readout { span { class: "pr", "{r.shares()[i]}%" } } }
                            for (j, l) in pv.lines.iter().enumerate() { div { key: "{j}", class: "pl", "{l}" } }
                        }
                    }
                    if i + 1 < panes.len() {
                        div { class: cls(&["div", if r.drag.as_ref().is_some_and(|d| d.0 == i) { "drag" } else { "" }]), role: "separator", tabindex: "0",
                            "aria-orientation": if vertical { "horizontal" } else { "vertical" },
                            "aria-controls": "oxyn-pane-{pv.pane.id}", "aria-valuenow": "{r.value_now(i)}", "aria-valuemin": "0", "aria-valuemax": "100",
                            "aria-valuetext": "{r.value_text(i)}", "aria-label": "Resize {pv.pane.label}",
                            onpointerdown: move |e| { e.prevent_default(); let c = e.client_coordinates(); last.set(if vertical { c.y } else { c.x }); m.write().begin(i) },
                            ondoubleclick: move |_| { m.write().reset(); on_layout_change.call(m.peek().shares()) },
                            onkeydown: move |e: KeyboardEvent| {
                                let big = e.modifiers().shift();
                                let st = if big { 64.0 } else { 16.0 };
                                let (dec, inc) = if vertical { (Key::ArrowUp, Key::ArrowDown) } else { (Key::ArrowLeft, Key::ArrowRight) };
                                let k = e.key();
                                let ch = if k == dec { m.write().move_divider(i, -st) }
                                    else if k == inc { m.write().move_divider(i, st) }
                                    else if k == Key::Home { m.write().to_limit(i, false) }
                                    else if k == Key::End { m.write().to_limit(i, true) }
                                    else if k == Key::Enter { m.write().toggle(i) }
                                    else if k == Key::Escape { m.write().cancel() }
                                    else { return };
                                e.prevent_default();
                                if ch { on_layout_change.call(m.peek().shares()) }
                            },
                            span { class: "hb", "aria-hidden": "true" }
                            if r.collapsed[i] || r.collapsed.get(i + 1).copied().unwrap_or(false) {
                                button { class: "restore", r#type: "button", tabindex: "-1", "aria-hidden": "true",
                                    onclick: move |_| { m.write().toggle(i); on_layout_change.call(m.peek().shares()) },
                                    Icon { name: if r.collapsed[i] { "chevron-right" } else { "chevron-left" }, size: 12.0 }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────── Drawer ──

/// Drawer: Arc UI's temporary side surface, built on [`Sheet`]: a titled header that doubles as
/// the drag handle, a close button ("Close drawer"), a scrolling body, any edge. A drag past a
/// third or a flick closes it. Starts closed with a trigger; `contained` keeps it inside the
/// nearest positioned ancestor (previews).
#[component]
pub fn Drawer(
    /// Title.
    title: String,
    /// Description.
    #[props(default)]
    description: Option<String>,
    /// `right` · `left` · `top` · `bottom`.
    #[props(default = "right".to_string())]
    side: String,
    /// Trigger label.
    #[props(default = "Open drawer".to_string())]
    trigger_label: String,
    /// Width / height px.
    #[props(default = 380.0)]
    size: f32,
    /// Floating inset panel.
    #[props(default)]
    inset: bool,
    /// Contained.
    #[props(default)]
    contained: bool,
    /// Footer action.
    #[props(default)]
    footer: Option<String>,
    /// Body.
    children: Element,
) -> Element {
    rsx! {
        Sheet { title, description, side, size, inset, contained, footer, trigger_label: Some(trigger_label), draggable: true, {children} }
    }
}

// ────────────────────────────────────── HoverCard ──

/// A person preview for [`HoverCard`].
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    /// Name.
    pub name: String,
    /// Role and team.
    pub role: Option<String>,
    /// Bio.
    pub bio: Option<String>,
    /// Stats.
    pub stats: Vec<(String, String)>,
    /// Footer line.
    pub meta: Option<String>,
    /// Oxyn plus: an agent (ring in its identity hue).
    pub agent_hue: Option<String>,
}

impl Profile {
    /// A profile.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            role: None,
            bio: None,
            stats: vec![],
            meta: None,
            agent_hue: None,
        }
    }
}

/// Hover card: a read-only preview that opens after `open_delay` of hover (instantly on focus or
/// Enter), stays while the pointer travels into it, closes after `close_delay`, and stays shut
/// after Escape while the pointer rests. Within 300 ms of another card it opens after 80 ms with
/// a fade only. Touch taps toggle. `role=tooltip` + `aria-describedby`; focus never moves in.
/// Oxyn plus: link / source previews (`source`) and an agent profile ring.
#[component]
pub fn HoverCard(
    /// Trigger text.
    trigger: String,
    /// Person preview.
    #[props(default)]
    profile: Option<Profile>,
    /// Oxyn plus: a source preview (title, domain, snippet).
    #[props(default)]
    source: Option<(String, String, String)>,
    /// `top` · `bottom` · `left` · `right`.
    #[props(default = "bottom".to_string())]
    side: String,
    /// `start` · `center` · `end`.
    #[props(default = "start".to_string())]
    align: String,
    /// Open delay ms.
    #[props(default = 500)]
    open_delay: u64,
    /// Close grace ms.
    #[props(default = 140)]
    close_delay: u64,
    /// Start open (no focus moves; nothing covers the page).
    #[props(default)]
    open: bool,
) -> Element {
    let mut h = use_signal(|| {
        let mut x = HoverIntent::with_delays(open_delay, close_delay);
        x.open = open;
        x
    });
    let mut t = use_signal(|| 0u64);
    let mut gn = use_signal(|| 0u32);
    let now = move || ClockMs(*t.peek());
    let hv = h.read().clone();
    let id = format!("oxyn-hc-{}", sl(&trigger));
    let wait = hv.deadline().map(|d| d.0.saturating_sub(t())).unwrap_or(0);
    rsx! {
        span { class: "oxyn-hc",
            onpointerenter: move |e| if e.data().pointer_type() != "touch" { h.write().enter(now()); gn += 1 },
            onpointerleave: move |e| if e.data().pointer_type() != "touch" { h.write().leave(now()); gn += 1 },
            button { class: "hc-trg", r#type: "button", "aria-describedby": if hv.open { "{id}" },
                onfocus: move |_| h.write().focus(),
                onblur: move |_| { h.write().close(now()); },
                onpointerup: move |e| if e.data().pointer_type() == "touch" { h.write().tap(now()) },
                onkeydown: move |e: KeyboardEvent| match e.key() { Key::Escape => { h.write().escape(now()); }, Key::Enter => h.write().focus(), _ => {} },
                if let Some(p) = &profile { span { class: "at", "@" } "{p.name}" } else { "{trigger}" }
            }
            if wait > 0 {
                span { key: "{gn}", class: "oxyn-timer", style: "animation-duration:{wait}ms",
                    onanimationend: move |_| { let d = h.peek().deadline(); if let Some(d) = d { t.set(d.0); h.write().tick(d); } } }
            }
            if hv.open {
                div { class: cls(&["hc-card", &side, &align, if hv.warm { "warm" } else { "" }]), role: "tooltip", id: "{id}",
                    if let Some(p) = profile.clone() {
                        div { class: "hc-top",
                            span { class: cls(&["hc-av", if p.agent_hue.is_some() { "agent" } else { "" }]), style: if let Some(hue) = &p.agent_hue { "--hue:var(--oxyn-color-agent-{hue})" }, "{initials(&p.name)}" }
                            div { class: "hc-id",
                                div { class: "hc-n", "{p.name}" }
                                if let Some(r) = p.role.clone() { div { class: "hc-r", "{r}" } }
                            }
                        }
                        if let Some(b) = p.bio.clone() { p { class: "hc-bio", "{b}" } }
                        if !p.stats.is_empty() {
                            dl { class: "hc-stats",
                                for (k, v) in p.stats.iter().cloned() { div { key: "{k}", dt { "{k}" } dd { "{v}" } } }
                            }
                        }
                        if let Some(mt) = p.meta.clone() { div { class: "hc-meta", Icon { name: "clock", size: 12.0 } "{mt}" } }
                    }
                    if let Some((tt, dom, snip)) = source.clone() {
                        div { class: "hc-src",
                            div { class: "hc-dom", span { class: "fav", "{dom.chars().next().unwrap_or('•')}" } "{dom}" }
                            div { class: "hc-n", "{tt}" }
                            p { class: "hc-bio", "{snip}" }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────── TreeView ──

/// Tree view: a keyboard-navigable file tree (`tree` / `treeitem` with level, position, set
/// size, expanded and selected), roving tabindex, ↑ ↓ ← → Home End Enter and `*`. Folders grow
/// open with children staggering in; the selection highlight glides. Oxyn plus: guide lines,
/// git / agent status badges and `density`.
#[component]
pub fn TreeView(
    /// Roots.
    nodes: Vec<TreeNode>,
    /// Open folders at first.
    #[props(default)]
    expanded: Vec<String>,
    /// Selected id.
    #[props(default)]
    selected: Option<String>,
    /// Accessible name.
    #[props(default = "File tree".to_string())]
    label: String,
    /// Guide lines.
    #[props(default = true)]
    guides: bool,
    /// `comfortable` (36 px) · `compact` (28 px).
    #[props(default = "comfortable".to_string())]
    density: String,
    /// A row was activated.
    #[props(default)]
    on_select: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let ex: Vec<&str> = expanded.iter().map(String::as_str).collect();
        let mut t = Tree::new(nodes.clone(), &ex);
        t.selected = selected.clone();
        t.focus = selected.clone();
        t
    });
    let mut items = use_signal(Vec::<(String, Rc<MountedData>)>::new);
    let focus_id = move |id: Option<String>| {
        if let Some(id) = id
            && let Some((_, d)) = items.peek().iter().find(|(k, _)| *k == id).cloned()
        {
            spawn(async move {
                let _ = d.set_focus(true).await;
            });
        }
    };
    let t = m.read().clone();
    let rows = t.rows();
    let stop = t
        .focus
        .clone()
        .or_else(|| rows.first().map(|r| r.id.clone()));
    rsx! {
        ul { class: cls(&["oxyn-tree", &density, if guides { "guides" } else { "" }]), role: "tree", "aria-label": "{label}",
            onkeydown: move |e: KeyboardEvent| {
                let k = match e.key() {
                    Key::ArrowDown => TreeKey::Down, Key::ArrowUp => TreeKey::Up, Key::ArrowRight => TreeKey::Right,
                    Key::ArrowLeft => TreeKey::Left, Key::Home => TreeKey::Home, Key::End => TreeKey::End,
                    Key::Enter => TreeKey::Activate,
                    Key::Character(c) if c == " " => TreeKey::Activate,
                    Key::Character(c) if c == "*" => TreeKey::ExpandSiblings,
                    _ => return,
                };
                e.prevent_default();
                let r = m.write().key(k);
                if let Some(id) = r { on_select.call(id) }
                focus_id(m.peek().focus.clone());
            },
            for r in rows.iter().cloned() {
                li { key: "{r.id}", class: cls(&["ti", if t.selected.as_ref() == Some(&r.id) { "sel" } else { "" }, if r.folder { "fold" } else { "" }]),
                    role: "treeitem", "aria-level": "{r.level}", "aria-posinset": "{r.pos}", "aria-setsize": "{r.size}",
                    "aria-expanded": if r.folder { "{r.expanded}" }, "aria-selected": "{t.selected.as_ref() == Some(&r.id)}",
                    tabindex: if stop.as_ref() == Some(&r.id) { "0" } else { "-1" },
                    style: "--lvl:{r.level - 1}",
                    onmounted: { let id = r.id.clone(); move |e: MountedEvent| { let mut v = items.write(); v.retain(|(k, _)| *k != id); v.push((id.clone(), e.data())); } },
                    onclick: { let id = r.id.clone(); move |_| { let s = m.write().activate(&id); if let Some(s) = s { on_select.call(s) } } },
                    span { class: "gd", "aria-hidden": "true", for (g, more) in r.guides.iter().enumerate() { i { key: "{g}", class: if *more { "g on" } else { "g" } } } }
                    if r.folder { span { class: "tv-chev", "aria-hidden": "true", Icon { name: "chevron-right", size: 12.0 } } } else { span { class: "tv-chev sp" } }
                    span { class: "ic", "aria-hidden": "true", Icon { name: r.icon.clone().unwrap_or_else(|| if r.folder { "folder".into() } else { "file".into() }), size: 14.0 } }
                    span { class: "lb", "{r.label}" }
                    if let Some(b) = r.badge.clone() { span { class: "bd b-{b}", "aria-label": "status {b}", "{b}" } }
                }
            }
        }
    }
}

// ──────────────────────────────── ReorderableList ──

/// A reorderable row.
#[derive(Clone, Debug, PartialEq)]
pub struct ReorderItem {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Detail line.
    pub detail: Option<String>,
}

impl ReorderItem {
    /// A row.
    pub fn new(id: &str, title: &str, detail: Option<&str>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: detail.map(str::to_string),
        }
    }
}

/// Reorderable list: drag the grip (pointer capture, `touch-action: none`) or press ↑ / ↓ on it.
/// Rows lift on press, neighbours slide, the position number rolls. Focus stays on the moved
/// handle. Oxyn plus: numbered or plain, and a remove button per row (queue editing).
#[component]
pub fn ReorderableList(
    /// Items.
    items: Vec<ReorderItem>,
    /// List label.
    #[props(default = "Reorder items".to_string())]
    label: String,
    /// Show position numbers.
    #[props(default = true)]
    numbered: bool,
    /// Remove buttons.
    #[props(default)]
    removable: bool,
    /// New order.
    #[props(default)]
    on_reorder: EventHandler<Vec<String>>,
) -> Element {
    let mut list = use_signal(|| items.clone());
    let mut drag = use_signal(|| None::<(usize, f64, f64)>);
    let mut handles = use_signal(Vec::<(String, Rc<MountedData>)>::new);
    let mut live = use_signal(String::new);
    let row_h = 58.0f32;
    let v = list.read().clone();
    let n = v.len();
    let (from, dy) = drag()
        .map(|(i, s, c)| (Some(i), (c - s) as f32))
        .unwrap_or((None, 0.0));
    let to = from.map(|f| drag_target(f, dy, row_h, n));
    let emit = move || on_reorder.call(list.peek().iter().map(|x| x.id.clone()).collect());
    rsx! {
        ol { class: "oxyn-reorder", "aria-label": "{label}",
            onpointermove: move |e| if let Some((i, s, _)) = drag() { drag.set(Some((i, s, e.client_coordinates().y))) },
            onpointerup: move |_| if let Some((i, s, c)) = drag() {
                let t = drag_target(i, (c - s) as f32, row_h, list.peek().len());
                drag.set(None);
                if t != i { move_item(&mut list.write(), i, t as isize - i as isize); emit(); }
            },
            for (i, it) in v.iter().cloned().enumerate() {
                li { key: "{it.id}", class: cls(&["rr", if from == Some(i) { "drag" } else { "" }]),
                    style: if from == Some(i) { "transform:translateY({dy}px) scale(1.01)".to_string() } else if let (Some(f), Some(t)) = (from, to) { format!("transform:translateY({}px)", drag_shift(i, f, t, row_h)) } else { String::new() },
                    button { class: "grip", r#type: "button", "aria-label": "{handle_label(&it.title, i + 1, n)}",
                        onmounted: { let id = it.id.clone(); move |e: MountedEvent| { let mut h = handles.write(); h.retain(|(k, _)| *k != id); h.push((id.clone(), e.data())); } },
                        onpointerdown: move |e| { e.prevent_default(); let y = e.client_coordinates().y; drag.set(Some((i, y, y))) },
                        onkeydown: { let id = it.id.clone(); let title = it.title.clone(); move |e: KeyboardEvent| {
                            let d = match e.key() { Key::ArrowUp => -1, Key::ArrowDown => 1, _ => return };
                            e.prevent_default();
                            let len = list.peek().len();
                            let to = move_item(&mut list.write(), i, d);
                            if to != i {
                                live.set(format!("{title} moved to position {} of {len}", to + 1));
                                emit();
                                if let Some((_, h)) = handles.peek().iter().find(|(k, _)| *k == id).cloned() { spawn(async move { let _ = h.set_focus(true).await; }); }
                            }
                        } },
                        Icon { name: "grip", size: 14.0 }
                    }
                    if numbered { span { class: "pos", key: "{i}", "{i + 1}" } }
                    div { class: "tx",
                        div { class: "tt", "{it.title}" }
                        if let Some(d) = it.detail.clone() { div { class: "dt", "{d}" } }
                    }
                    if removable {
                        button { class: "oxyn-x", r#type: "button", "aria-label": "Remove {it.title}", onclick: move |_| { list.write().remove(i); emit() }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                    }
                }
            }
        }
        div { class: "sr", role: "status", "{live}" }
    }
}

// ─────────────────────────────── ToggleGroup / Chips ──

/// Toggle group: a strip of toggles. Single mode is a `radiogroup` whose one fill glides;
/// multiple mode is a `group` of `aria-pressed` buttons whose fills fade in and join with pressed
/// neighbours (a formatting toolbar). Arrows move focus only (toolbar), Space / Enter toggles.
#[component]
pub fn ToggleGroup(
    /// Items.
    items: Vec<ToggleItem>,
    /// Group name.
    label: String,
    /// Multiple mode.
    #[props(default)]
    multiple: bool,
    /// Pressed values.
    #[props(default)]
    value: Vec<String>,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// New value(s).
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut m = use_signal(|| {
        let v: Vec<&str> = value.iter().map(String::as_str).collect();
        ToggleModel::new(items.clone(), multiple, &v)
    });
    let glide = use_glide();
    let mut nodes = use_signal(|| vec![None::<Rc<MountedData>>; items.len()]);
    let g = m.read().clone();
    let joins = g.joins();
    let stop = g
        .focus
        .or_else(|| (0..g.items.len()).find(|&i| g.on(i)))
        .unwrap_or(0);
    let sel = (0..g.items.len()).find(|&i| g.on(i));
    rsx! {
        div { class: cls(&["oxyn-tgl", &size, if multiple { "multi" } else { "single" }]), role: if multiple { "group" } else { "radiogroup" }, "aria-label": "{label}", "aria-disabled": if disabled { "true" },
            onmounted: move |e| { glide.set_root(e.data()); let s = (0..m.peek().items.len()).find(|&i| m.peek().on(i)); if !multiple { glide.go(s, true) } },
            onkeydown: move |e: KeyboardEvent| if let Some(k) = nav_key(&e.key(), false) {
                e.prevent_default();
                if let Some(j) = m.write().key(k) && let Some(Some(d)) = nodes.peek().get(j).cloned() { spawn(async move { let _ = d.set_focus(true).await; }); }
            },
            if !multiple && sel.is_some() { span { class: "{glide.class(\"fill\")}", style: "{glide.style()}", "aria-hidden": "true" } }
            for (i, it) in g.items.iter().cloned().enumerate() {
                button { key: "{it.value}", r#type: "button", class: cls(&["ti", if g.on(i) { "on" } else { "" }, if joins[i].0 { "jl" } else { "" }, if joins[i].1 { "jr" } else { "" }]),
                    role: if !multiple { "radio" }, "aria-checked": if !multiple { "{g.on(i)}" }, "aria-pressed": if multiple { "{g.on(i)}" },
                    "aria-label": if !it.text { "{it.label}" }, title: "{it.label}", disabled: disabled || it.disabled,
                    tabindex: if stop == i { "0" } else { "-1" },
                    onmounted: move |e| { glide.set_item(i, e.data()); if let Some(s) = nodes.write().get_mut(i) { *s = Some(e.data()) } },
                    onclick: move |_| { let ch = m.write().press(i); match (multiple, ch) { (false, c) => { glide.go(Some(i), false); if c { on_change.call(m.peek().value.clone()) } } (true, true) => on_change.call(m.peek().value.clone()), _ => {} } },
                    if multiple { span { class: "mf", "aria-hidden": "true" } }
                    span { class: "ic",
                        if let Some(ic) = it.icon.clone() { Icon { name: ic, size: if size == "sm" { 14.0 } else { 16.0 } } }
                        if it.text { span { "{it.label}" } }
                    }
                }
            }
        }
    }
}

/// Chip group: filter chips with `aria-pressed`. Selecting grows a check in and slides the label;
/// long sets fold behind "+N more" (selected chips always stay visible). Roving focus wraps.
/// Oxyn plus: counts per chip, `variant` `outline` / `soft`, and a Clear chip.
#[component]
pub fn ChipGroup(
    /// (value, label).
    options: Vec<(String, String)>,
    /// Group name.
    label: String,
    /// Selected.
    #[props(default)]
    value: Vec<String>,
    /// Several at once.
    #[props(default = true)]
    multiple: bool,
    /// Visible before folding (0 = all).
    #[props(default)]
    max_visible: usize,
    /// Count per option (same order).
    #[props(default)]
    counts: Vec<u32>,
    /// `outline` · `soft`.
    #[props(default = "outline".to_string())]
    variant: String,
    /// Clear chip when something is selected.
    #[props(default)]
    clearable: bool,
    /// New selection.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut m = use_signal(|| {
        let o: Vec<(&str, &str)> = options
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let v: Vec<&str> = value.iter().map(String::as_str).collect();
        let mut c = ChipModel::new(&o, &v, multiple);
        c.max_visible = max_visible;
        c
    });
    let mut focus = use_signal(|| 0usize);
    let mut nodes = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let c = m.read().clone();
    let (vis, hidden) = c.visible();
    let total = vis.len() + usize::from(hidden > 0);
    rsx! {
        div { class: cls(&["oxyn-chips", &variant]), role: "group", "aria-label": "{label}",
            onkeydown: move |e: KeyboardEvent| {
                let k = match e.key() { Key::ArrowRight | Key::ArrowDown => NavKey::Next, Key::ArrowLeft | Key::ArrowUp => NavKey::Prev, Key::Home => NavKey::First, Key::End => NavKey::Last, _ => return };
                e.prevent_default();
                let f = oxyn_core::primitives::nav::step(Some(focus()), k, total, true, |_| true).unwrap_or(0);
                focus.set(f);
                if let Some(Some(d)) = nodes.peek().get(f).cloned() { spawn(async move { let _ = d.set_focus(true).await; }); }
            },
            for (k, i) in vis.iter().copied().enumerate() {
                button { key: "{c.options[i].0}", r#type: "button", class: cls(&["chip", if c.on(i) { "on" } else { "" }]), "aria-pressed": "{c.on(i)}",
                    tabindex: if focus() == k { "0" } else { "-1" },
                    onmounted: move |e| { let mut n = nodes.write(); if n.len() <= k { n.resize(k + 1, None) } n[k] = Some(e.data()) },
                    onclick: move |_| { focus.set(k); let v = m.write().toggle(i); on_change.call(v) },
                    span { class: "ck", "aria-hidden": "true", Icon { name: "check", size: 12.0, stroke: 2.2 } }
                    span { class: "lb", "{c.options[i].1}" }
                    if let Some(n) = counts.get(i) { span { class: "n", "{n}" } }
                }
            }
            if hidden > 0 {
                button { r#type: "button", class: "chip more", "aria-expanded": "false", tabindex: if focus() == vis.len() { "0" } else { "-1" },
                    onmounted: move |e| { let k = m.peek().visible().0.len(); let mut n = nodes.write(); if n.len() <= k { n.resize(k + 1, None) } n[k] = Some(e.data()) },
                    onclick: move |_| m.write().expanded = true, "+{hidden} more"
                }
            }
            if clearable && !c.value.is_empty() {
                button { r#type: "button", class: "chip clear", tabindex: "-1", onclick: move |_| { m.write().value.clear(); on_change.call(vec![]) }, Icon { name: "x", size: 11.0, stroke: 2.0 } "Clear" }
            }
        }
    }
}

/// Radio cards: option cards in a `radiogroup`; one ring glides to the selected card and its dot
/// pops in. Arrows select and wrap (skipping disabled), Home / End, one tab stop. `grid` uses
/// auto-fill columns (`min_column`), `list` stacks rows with meta at the end. Oxyn plus: risk
/// tones (permission modes) and a hidden form input.
#[component]
pub fn RadioCards(
    /// Cards.
    options: Vec<RadioCard>,
    /// Selected.
    #[props(default)]
    value: Option<String>,
    /// `grid` · `list`.
    #[props(default = "grid".to_string())]
    layout: String,
    /// Narrowest grid column px.
    #[props(default = 180.0)]
    min_column: f32,
    /// Group label.
    #[props(default = "Options".to_string())]
    label: String,
    /// Form name.
    #[props(default)]
    name: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Selected.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let glide = use_glide();
    let mut nodes = use_signal(|| vec![None::<Rc<MountedData>>; options.len()]);
    let cur = v();
    let sel = cur
        .as_ref()
        .and_then(|x| options.iter().position(|o| &o.value == x));
    let stop = sel.or_else(|| options.iter().position(|o| !o.disabled));
    let opts = options.clone();
    rsx! {
        div { class: cls(&["oxyn-rcards", &layout]), role: "radiogroup", "aria-label": "{label}", "aria-disabled": if disabled { "true" },
            style: "--minc:{min_column}px",
            onmounted: move |e| { glide.set_root(e.data()); glide.go(sel, true) },
            onkeydown: move |e: KeyboardEvent| {
                let k = match e.key() { Key::ArrowRight | Key::ArrowDown => NavKey::Next, Key::ArrowLeft | Key::ArrowUp => NavKey::Prev, Key::Home => NavKey::First, Key::End => NavKey::Last, _ => return };
                e.prevent_default();
                if disabled { return }
                let curv = v.peek().clone();
                if let Some(j) = radio_step(&opts, curv.as_deref(), k) {
                    v.set(Some(opts[j].value.clone()));
                    on_change.call(opts[j].value.clone());
                    glide.go(Some(j), true);
                    if let Some(Some(d)) = nodes.peek().get(j).cloned() { spawn(async move { let _ = d.set_focus(true).await; }); }
                }
            },
            if sel.is_some() { span { class: "{glide.class(\"ring\")}", style: "{glide.style()}", "aria-hidden": "true" } }
            for (i, o) in options.iter().cloned().enumerate() {
                div { key: "{o.value}", class: cls(&["rc", if sel == Some(i) { "on" } else { "" }, if o.disabled || disabled { "dis" } else { "" }, o.tone.as_deref().unwrap_or("")]),
                    role: "radio", "aria-checked": "{sel == Some(i)}", "aria-disabled": if o.disabled || disabled { "true" },
                    "aria-labelledby": "oxyn-rc-{o.value}", "aria-describedby": "oxyn-rcd-{o.value}",
                    tabindex: if stop == Some(i) && !disabled { "0" } else { "-1" },
                    onmounted: move |e| { glide.set_item(i, e.data()); if let Some(s) = nodes.write().get_mut(i) { *s = Some(e.data()) } },
                    onclick: { let val = o.value.clone(); let dis = o.disabled || disabled; move |_| if !dis { v.set(Some(val.clone())); on_change.call(val.clone()); glide.go(Some(i), false) } },
                    span { class: "dot", "aria-hidden": "true", i {} }
                    if let Some(ic) = o.icon.clone() { span { class: "ic", "aria-hidden": "true", Icon { name: ic, size: 16.0 } } }
                    div { class: "tx",
                        div { class: "lb", id: "oxyn-rc-{o.value}", "{o.label}" }
                        div { class: "ds", id: "oxyn-rcd-{o.value}", {o.disabled_reason.clone().filter(|_| o.disabled).or(o.description.clone()).unwrap_or_default()} }
                    }
                    if let Some(mt) = o.meta.clone() { span { class: "mt", "{mt}" } }
                }
            }
            if let Some(n) = name { input { r#type: "hidden", name: "{n}", value: "{cur.clone().unwrap_or_default()}" } }
        }
    }
}

// ───────────────────────────────────────── Rating ──

/// Rating: stars as a `radiogroup` with half stars, pointer scrubbing, clearing (same value again
/// or Backspace), digits, Home / End and a value word that rolls. `read_only` renders an `img`.
/// Oxyn plus: `glyph` `star` · `heart` · `dot`, and a count line ("4.6 · 1,204 ratings").
#[component]
pub fn Rating(
    /// Label.
    label: String,
    /// Value (0 = not rated).
    #[props(default)]
    value: f32,
    /// Stars.
    #[props(default = 5)]
    max: u32,
    /// Half steps.
    #[props(default)]
    allow_half: bool,
    /// Same value clears.
    #[props(default = true)]
    allow_clear: bool,
    /// Read only.
    #[props(default)]
    read_only: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Show the value word.
    #[props(default = true)]
    show_value: bool,
    /// Hint under the stars.
    #[props(default)]
    description: Option<String>,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// `star` · `heart` · `dot`.
    #[props(default = "star".to_string())]
    glyph: String,
    /// Words per whole star.
    #[props(default)]
    labels: Vec<String>,
    /// Committed.
    #[props(default)]
    on_change: EventHandler<f32>,
) -> Element {
    let mut m = use_signal(|| {
        let mut r = RatingModel::new(max, value);
        r.allow_half = allow_half;
        r.allow_clear = allow_clear;
        r
    });
    let mut live = use_signal(String::new);
    let r = m.read().clone();
    let shown = r.shown();
    let word = r.word(shown, &labels, "Not rated");
    let did = format!("oxyn-rt-{}", sl(&label));
    let path = match glyph.as_str() {
        "heart" => "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
        "dot" => "M12 19a7 7 0 1 0 0-14 7 7 0 0 0 0 14z",
        _ => "m12 3.5 2.6 5.4 5.9.8-4.3 4.1 1 5.8L12 16.8l-5.2 2.8 1-5.8-4.3-4.1 5.9-.8z",
    };
    if read_only {
        return rsx! {
            div { class: cls(&["oxyn-rating", &size, &glyph, "ro"]),
                span { class: "rl", "{label}" }
                div { class: "stars", role: "img", "aria-label": "{label}: {value} out of {max}",
                    for i in 0..max { span { key: "{i}", class: "st", style: "--f:{r.fill(i)}", svg { view_box: "0 0 24 24", path { class: "bg", d: "{path}" } path { class: "fg", d: "{path}" } } } }
                }
                if show_value { span { class: "rv", "{word}" } }
            }
        };
    }
    let mut commit = move |v: f32| {
        let c = m.write().commit(v);
        let w = m.peek().word(c, &[], "Not rated");
        live.set(if c == 0.0 {
            "Rating cleared".into()
        } else {
            format!("Rated {w}")
        });
        on_change.call(c);
    };
    rsx! {
        div { class: cls(&["oxyn-rating", &size, &glyph, if disabled { "dis" } else { "" }]),
            span { class: "rl", id: "{did}-l", "{label}" }
            div { class: "stars", role: "radiogroup", "aria-labelledby": "{did}-l", "aria-describedby": if description.is_some() { "{did}-d" }, "aria-disabled": if disabled { "true" },
                onpointerleave: move |_| m.write().hover = None,
                onkeydown: move |e: KeyboardEvent| {
                    if disabled { return }
                    let k = match e.key() {
                        Key::ArrowRight | Key::ArrowUp => RatingKey::Up, Key::ArrowLeft | Key::ArrowDown => RatingKey::Down,
                        Key::Home => RatingKey::Home, Key::End => RatingKey::End, Key::Backspace | Key::Delete => RatingKey::Clear,
                        Key::Character(c) if c.len() == 1 && c.chars().all(|x| x.is_ascii_digit() && x != '0') => RatingKey::Digit(c.parse().unwrap_or(1)),
                        _ => return,
                    };
                    e.prevent_default();
                    let v = m.write().key(k);
                    on_change.call(v);
                },
                for i in 0..max {
                    span { key: "{i}", class: cls(&["st", if r.fill(i) > 0.0 { "lit" } else { "" }]), style: "--f:{r.fill(i)};--d:{i * 30}ms",
                        role: "radio", "aria-checked": "{(r.value - (i + 1) as f32).abs() < 0.01}", "aria-label": "{r.radio_label((i + 1) as f32, &labels)}",
                        tabindex: if (r.value.ceil() as u32).max(1) == i + 1 && !disabled { "0" } else { "-1" },
                        onpointermove: move |e| if !disabled { let x = e.element_coordinates().x as f32; let w = if m.peek().allow_half { 0.5 } else { 1.0 }; let f = if x < 14.0 { 0.25 } else { 0.75 }; let v = m.peek().value_at(i, f); let _ = w; m.write().hover = Some(v) },
                        onclick: move |e| if !disabled { let x = e.element_coordinates().x as f32; let f = if x < 14.0 { 0.25 } else { 0.75 }; let v = m.peek().value_at(i, f); commit(v) },
                        svg { view_box: "0 0 24 24", "aria-hidden": "true", path { class: "bg", d: "{path}" } path { class: "fg", d: "{path}" } }
                    }
                }
            }
            if show_value { for _k in std::iter::once(word.clone()) { span { key: "{_k}", class: cls(&["rv", if r.dir < 0 { "dn" } else { "up" }]), "{word}" } } }
            if let Some(d) = description.clone() { span { class: "rd", id: "{did}-d", "{d}" } }
            div { class: "sr", role: "status", "{live}" }
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

// ─────────────────────────────────────── Reactions ──

/// Reactions: pills with counts (yours highlighted) and an add button that grows a picker
/// toolbar. Zero counts leave the bar; counts roll; the picker has roving focus, Escape returns to
/// the add button; a status region announces each change. Oxyn plus: `compact` pills and an
/// agent-message `quick` row (thumbs only).
#[component]
pub fn Reactions(
    /// Picker options.
    options: Vec<ReactionOption>,
    /// Initial pills.
    #[props(default)]
    value: Vec<ReactionCount>,
    /// Bar label.
    #[props(default = "Reactions".to_string())]
    label: String,
    /// Small pills.
    #[props(default)]
    compact: bool,
    /// Picker starts open (rendered inline; nothing covers the page).
    #[props(default)]
    picker_open: bool,
    /// Changed (id, reacted).
    #[props(default)]
    on_change: EventHandler<(String, bool)>,
) -> Element {
    let mut m = use_signal(|| {
        let mut r = ReactionsModel::new(options.clone(), value.clone());
        r.picker = picker_open;
        r
    });
    let mut opts_n = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let mut add = use_signal(|| None::<Rc<MountedData>>);
    let r = m.read().clone();
    let focus_opt = move |i: Option<usize>| {
        if let Some(Some(d)) = i.and_then(|i| opts_n.peek().get(i).cloned()) {
            spawn(async move {
                let _ = d.set_focus(true).await;
            });
        }
    };
    rsx! {
        div { class: cls(&["oxyn-react", if compact { "compact" } else { "" }]), role: "group", "aria-label": "{label}",
            for rc in r.value.iter().cloned() {
                button { key: "{rc.id}", r#type: "button", class: cls(&["pill", if rc.reacted { "mine" } else { "" }]), "aria-pressed": "{rc.reacted}", "aria-label": "{r.pill_label(&rc)}",
                    onclick: { let id = rc.id.clone(); move |_| { let (on, _) = m.write().toggle(&id); on_change.call((id.clone(), on)) } },
                    span { class: "g", "{r.option(&rc.id).map(|o| o.glyph.clone()).unwrap_or_default()}" }
                    span { class: "n", key: "{rc.count}", "{rc.count}" }
                }
            }
            span { class: "addw",
                button { r#type: "button", class: "add", "aria-label": "Add reaction", "aria-expanded": "{r.picker}",
                    onmounted: move |e| add.set(Some(e.data())),
                    onclick: move |_| { let o = !m.peek().picker; m.write().set_picker(o); if o { focus_opt(Some(0)) } },
                    Icon { name: "plus", size: 13.0 } span { class: "sm", "☺" }
                }
                if r.picker {
                    div { class: "tray", role: "toolbar", "aria-label": "Pick a reaction",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape { e.prevent_default(); m.write().set_picker(false); if let Some(a) = add.peek().clone() { spawn(async move { let _ = a.set_focus(true).await; }); } return }
                            let k = match e.key() { Key::ArrowRight | Key::ArrowDown => NavKey::Next, Key::ArrowLeft | Key::ArrowUp => NavKey::Prev, Key::Home => NavKey::First, Key::End => NavKey::Last, _ => return };
                            e.prevent_default(); m.write().key(k); focus_opt(m.peek().focus);
                        },
                        for (i, o) in r.options.iter().cloned().enumerate() {
                            button { key: "{o.id}", r#type: "button", class: cls(&["opt", if r.focus == Some(i) { "is-hl" } else { "" }]), "aria-label": "{o.label}", title: "{o.label}",
                                tabindex: if r.focus.unwrap_or(0) == i { "0" } else { "-1" },
                                onmounted: move |e| { let mut n = opts_n.write(); if n.len() <= i { n.resize(i + 1, None) } n[i] = Some(e.data()) },
                                onpointerenter: move |_| m.write().focus = Some(i),
                                onclick: { let id = o.id.clone(); move |_| { let (on, _) = m.write().toggle(&id); on_change.call((id.clone(), on)) } },
                                "{o.glyph}"
                            }
                        }
                    }
                }
            }
            div { class: "sr", role: "status", "{r.announce()}" }
        }
    }
}

// ───────────────────────────────── ExpandableText ──

/// Expandable text: clamps to `lines` lines with a soft fade; Read more opens to full height on
/// a soft ease (500–600 ms), Show less closes (400–450 ms). The toggle shows only when the text
/// overflows (measured), the full text stays in the accessibility tree, and the two labels
/// crossfade in one cell. Oxyn plus: `fade` off, an inline toggle, and a character `limit`.
#[component]
pub fn ExpandableText(
    /// The text (paragraphs split on blank lines).
    text: String,
    /// Lines while collapsed.
    #[props(default = 3)]
    lines: u32,
    /// Start open.
    #[props(default)]
    expanded: bool,
    /// Toggle label while collapsed.
    #[props(default = "Read more".to_string())]
    more_label: String,
    /// Toggle label while open.
    #[props(default = "Show less".to_string())]
    less_label: String,
    /// Fade over the last line.
    #[props(default = true)]
    fade: bool,
    /// Open state changed.
    #[props(default)]
    on_change: EventHandler<bool>,
) -> Element {
    let mut open = use_signal(|| expanded);
    let mut needs = use_signal(|| true);
    let id = format!("oxyn-xt-{}", sl(&text));
    let o = open();
    rsx! {
        div { class: cls(&["oxyn-xtext", if o { "open" } else { "" }, if fade { "fade" } else { "" }]), style: "--lines:{lines}",
            div { class: "xb", id: "{id}",
                onmounted: move |e| { let d = e.data(); spawn(async move { if let (Ok(sz), Ok(r)) = (d.get_scroll_size().await, d.get_client_rect().await) { needs.set(sz.height > r.size.height + 2.0 || *open.peek()); } }); },
                for (i, p) in text.split("\n\n").enumerate() { p { key: "{i}", "{p}" } }
            }
            if needs() {
                button { class: "xt", r#type: "button", "aria-expanded": "{o}", "aria-controls": "{id}",
                    onclick: move |_| { let n = !open(); open.set(n); on_change.call(n) },
                    span { class: "xl",
                        span { class: if !o { "on" } else { "" }, "aria-hidden": if o { "true" }, "{more_label}" }
                        span { class: if o { "on" } else { "" }, "aria-hidden": if !o { "true" }, "{less_label}" }
                    }
                    Icon { name: "chevron", size: 12.0 }
                }
            }
        }
    }
}

// ─────────────────────────────────── Empty presets ──

/// Empty states block: Arc UI's presets for agent apps (`first-run` · `no-results` · `no-runs` ·
/// `inbox-zero` · `offline` · `error` · `no-access`), drawn by [`EmptyState`]. Copy changes roll
/// in; `layout` `framed` / `bare` / `inline`.
#[component]
pub fn EmptyPreset(
    /// Preset id.
    #[props(default = "first-run".to_string())]
    preset: String,
    /// Query for no-results.
    #[props(default)]
    query: String,
    /// `framed` · `bare` · `inline`.
    #[props(default = "framed".to_string())]
    layout: String,
    /// Starter prompts (first-run).
    #[props(default)]
    prompts: Vec<String>,
    /// Primary action.
    #[props(default)]
    on_action: EventHandler<()>,
) -> Element {
    use oxyn_core::primitives::display::EmptyLayout;
    let mut s = PresetModel::from_id(&preset).state(&query);
    s.layout = match layout.as_str() {
        "bare" => EmptyLayout::Bare,
        "inline" => EmptyLayout::Inline,
        _ => EmptyLayout::Framed,
    };
    s.prompts = prompts;
    rsx! {
        div { class: "oxyn-empties", "data-preset": "{preset}",
            for _k in std::iter::once(preset.clone()) { div { key: "{_k}", class: "eroll", EmptyState { state: s.clone(), on_action } } }
        }
    }
}

// ─────────────────────────────────────────── Dock ──

/// Dock: a floating toolbar for tools or modes. A label glides between items on hover / focus
/// (not touch), an accent tint springs to the chosen item, groups grow a tray of members (↑ opens,
/// ↓ / Escape closes, ← → move inside), Alt + ← → reorders. Names include shortcut and badge.
/// Oxyn plus: `position` `bottom` / `left` (vertical), and `magnify` hover scaling.
#[component]
pub fn Dock(
    /// Items.
    items: Vec<DockItem>,
    /// Selected id.
    #[props(default)]
    value: Option<String>,
    /// Toolbar label.
    #[props(default = "Tools".to_string())]
    label: String,
    /// Magnify on hover.
    #[props(default)]
    magnify: bool,
    /// Allow Alt + arrows to reorder.
    #[props(default)]
    reorder: bool,
    /// `bottom` · `left`.
    #[props(default = "bottom".to_string())]
    position: String,
    /// Chosen.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| DockModel::new(items.clone(), value.as_deref()));
    let mut hover = use_signal(|| None::<usize>);
    let mut nodes = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let vertical = position == "left";
    let d = m.read().clone();
    let focus_i = move |i: Option<usize>| {
        if let Some(Some(n)) = i.and_then(|i| nodes.peek().get(i).cloned()) {
            spawn(async move {
                let _ = n.set_focus(true).await;
            });
        }
    };
    let stop = d.focus.unwrap_or(0);
    rsx! {
        div { class: cls(&["oxyn-dock", &position, if magnify { "mag" } else { "" }]), role: "toolbar", "aria-label": "{label}", "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            onkeydown: move |e: KeyboardEvent| {
                let alt = e.modifiers().alt();
                let (prev, next) = if vertical { (Key::ArrowUp, Key::ArrowDown) } else { (Key::ArrowLeft, Key::ArrowRight) };
                let k = e.key();
                if alt && reorder && (k == prev || k == next) {
                    e.prevent_default();
                    m.write().move_focused(if k == prev { -1 } else { 1 });
                    focus_i(m.peek().focus);
                } else if k == prev || k == next || k == Key::Home || k == Key::End {
                    e.prevent_default();
                    let nk = if k == prev { NavKey::Prev } else if k == next { NavKey::Next } else if k == Key::Home { NavKey::First } else { NavKey::Last };
                    m.write().key(nk);
                    if m.peek().tray_focus.is_none() { focus_i(m.peek().focus) }
                } else if k == Key::ArrowUp && !vertical {
                    e.prevent_default(); m.write().open_tray();
                } else if (k == Key::ArrowDown && !vertical) || k == Key::Escape {
                    if m.write().close_tray() { e.prevent_default(); focus_i(m.peek().focus) }
                } else if k == Key::Enter || k == Key::Character(" ".into()) {
                    e.prevent_default();
                    let (tray, tf) = (m.peek().tray, m.peek().tray_focus);
                    let r = if let (Some(t), Some(j)) = (tray, tf) { m.write().choose(t, j) } else { let f = m.peek().focus.unwrap_or(0); m.write().press(f) };
                    if let Some(v) = r { on_change.call(v) }
                }
            },
            for (i, it) in d.items.iter().cloned().enumerate() {
                div { key: "{it.id}", class: cls(&["dw", if hover() == Some(i) { "hv" } else { "" }]),
                    if hover() == Some(i) || d.focus == Some(i) && d.tray.is_none() {
                        span { class: "dl", "aria-hidden": "true", {d.shown_member(i).map(|m| m.label.clone()).unwrap_or(it.label.clone())} if let Some(k) = d.shown_member(i).and_then(|m| m.shortcut.clone()).or(it.shortcut.clone()) { kbd { "{k}" } } }
                    }
                    button { r#type: "button", class: cls(&["di", if d.is_selected(i) { "on" } else { "" }, if !it.items.is_empty() { "dgrp" } else { "" }]),
                        "aria-label": "{d.aria(i)}", "aria-pressed": if it.items.is_empty() { "{d.is_selected(i)}" },
                        "aria-haspopup": if !it.items.is_empty() { "true" }, "aria-expanded": if !it.items.is_empty() { "{d.tray == Some(i)}" },
                        "aria-controls": if !it.items.is_empty() { "oxyn-dock-tray-{it.id}" },
                        tabindex: if stop == i { "0" } else { "-1" },
                        onmounted: move |e| { let mut n = nodes.write(); if n.len() <= i { n.resize(i + 1, None) } n[i] = Some(e.data()) },
                        onpointerenter: move |e| if e.data().pointer_type() != "touch" { hover.set(Some(i)) },
                        onpointerleave: move |_| hover.set(None),
                        onclick: move |_| { if let Some(v) = m.write().press(i) { on_change.call(v) } },
                        span { class: "tint", "aria-hidden": "true" }
                        for _k in std::iter::once(d.shown_member(i).map(|m| m.id.clone()).unwrap_or_default()) {
                            span { key: "{_k}", class: "dg", Icon { name: d.shown_member(i).map(|m| m.icon.clone()).unwrap_or(it.icon.clone()), size: 18.0 } }
                        }
                        if let Some(b) = it.badge { span { class: "db", "aria-hidden": "true", "{b}" } }
                        if !it.items.is_empty() { span { class: "dc", "aria-hidden": "true" } }
                    }
                    if !it.items.is_empty() {
                        div { class: cls(&["tray", if d.tray == Some(i) { "open" } else { "" }]), id: "oxyn-dock-tray-{it.id}", role: "group", "aria-label": "{it.label}", "inert": if d.tray != Some(i) { "true" },
                            for (j, mb) in it.items.iter().cloned().enumerate() {
                                button { key: "{mb.id}", r#type: "button", class: cls(&["tm", if d.value.as_ref() == Some(&mb.id) { "on" } else { "" }, if d.tray_focus == Some(j) && d.tray == Some(i) { "is-hl" } else { "" }]),
                                    "aria-label": if let Some(k) = &mb.shortcut { "{mb.label}, {k}" } else { "{mb.label}" }, style: "--d:{j * 30}ms",
                                    onclick: move |_| { if let Some(v) = m.write().choose(i, j) { on_change.call(v) } },
                                    Icon { name: mb.icon.clone(), size: 16.0 }
                                    span { "{mb.label}" }
                                    if let Some(k) = mb.shortcut.clone() { kbd { "{k}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────── TocProgress ──

/// A section of the demo article for [`TocProgress`].
#[derive(Clone, Debug, PartialEq)]
pub struct TocSection {
    /// Heading.
    pub item: TocItem,
    /// Body paragraphs.
    pub body: Vec<String>,
}

/// Table of contents that reads along: a line runs down the headings, bends through their
/// indentation, fills as you scroll and carries a marker to the current section; the time left
/// rolls down to Finished. Headings are anchor links (`aria-current="location"`); Enter scrolls
/// and focuses the heading. Renders its own scrolling article (`height`) so it works anywhere.
/// Oxyn plus: `side` `left` / `right` and a compact `bar` mode for narrow screens.
#[component]
pub fn TocProgress(
    /// Sections.
    sections: Vec<TocSection>,
    /// Title.
    #[props(default = "On this page".to_string())]
    title: String,
    /// Reading time minutes (0 = estimate).
    #[props(default)]
    reading_time: f32,
    /// Show time left.
    #[props(default = true)]
    show_time_left: bool,
    /// Reached offset px.
    #[props(default = 48.0)]
    offset: f32,
    /// Height of the article viewport px.
    #[props(default = 360.0)]
    height: f32,
    /// `right` · `left` · `bar`.
    #[props(default = "right".to_string())]
    side: String,
    /// Current heading changed.
    #[props(default)]
    on_active_change: EventHandler<String>,
) -> Element {
    let n = sections.len();
    let mut prog = use_signal(|| (0usize, 0.0f32, 0.0f32));
    let mut heads = use_signal(|| vec![None::<Rc<MountedData>>; n]);
    let mut vp = use_signal(|| None::<Rc<MountedData>>);
    let words: usize = sections
        .iter()
        .map(|s| {
            s.body
                .iter()
                .map(|b| b.split_whitespace().count())
                .sum::<usize>()
        })
        .sum();
    let total = if reading_time > 0.0 {
        reading_time
    } else {
        oxyn_core::primitives::layout::reading_minutes(words * 40)
    };
    let (cur, frac, overall) = prog();
    let ids = use_signal(|| {
        sections
            .iter()
            .map(|s| s.item.id.clone())
            .collect::<Vec<_>>()
    });
    let measure = move |top: f32, view: f32, content: f32| {
        let hs = heads.peek().clone();
        let v = vp.peek().clone();
        spawn(async move {
            let Some(v) = v else { return };
            let Ok(vr) = v.get_client_rect().await else {
                return;
            };
            let mut tops = Vec::new();
            for h in hs.iter().flatten() {
                if let Ok(r) = h.get_client_rect().await {
                    tops.push((r.origin.y - vr.origin.y) as f32 + top);
                }
            }
            let p = toc_progress(&tops, top, view, content, offset);
            if p.0 != prog.peek().0
                && let Some(s) = ids.peek().get(p.0).cloned()
            {
                on_active_change.call(s);
            }
            prog.set(p);
        });
    };
    let secs = sections.clone();
    rsx! {
        div { class: cls(&["oxyn-toc", &side]), style: "height:{height}px",
            div { class: "art", tabindex: "0", role: "region", "aria-label": "Article",
                onmounted: move |e| { vp.set(Some(e.data())); measure(0.0, height, height * 3.0) },
                onscroll: move |e| measure(e.scroll_top() as f32, e.client_height() as f32, e.scroll_height() as f32),
                for (i, s) in secs.iter().cloned().enumerate() {
                    section { key: "{s.item.id}",
                        {
                            let h = rsx! { "{s.item.label}" };
                            match s.item.level { 1 => rsx! { h2 { id: "{s.item.id}", tabindex: "-1", onmounted: move |e| heads.write()[i] = Some(e.data()), {h} } },
                                2 => rsx! { h3 { id: "{s.item.id}", tabindex: "-1", onmounted: move |e| heads.write()[i] = Some(e.data()), {h} } },
                                _ => rsx! { h4 { id: "{s.item.id}", tabindex: "-1", onmounted: move |e| heads.write()[i] = Some(e.data()), {h} } } }
                        }
                        for (j, b) in s.body.iter().enumerate() { p { key: "{j}", "{b}" } }
                    }
                }
            }
            nav { class: "tocn", "aria-label": "{title}",
                if !title.is_empty() {
                    div { class: "tt",
                        span { "{title}" }
                        if show_time_left { for _k in std::iter::once(time_left(total, overall)) { span { key: "{_k}", class: cls(&["tl", if overall >= 0.995 { "done" } else { "" }]), if overall >= 0.995 { Icon { name: "check", size: 11.0, stroke: 2.2 } } "{time_left(total, overall)}" } } }
                    }
                }
                if side == "bar" { div { class: "bar", "aria-hidden": "true", i { style: "width:{overall * 100.0}%" } } }
                ol { class: "ls", style: "--p:{overall}",
                    onkeydown: move |e: KeyboardEvent| { let _ = &e; },
                    for (i, s) in sections.iter().cloned().enumerate() {
                        li { key: "{s.item.id}", class: cls(&["tci", &format!("l{}", s.item.level), if i < cur { "read" } else if i == cur { "cur" } else { "" }]),
                            style: if i == cur { format!("--f:{frac:.3}") } else if i < cur { "--f:1".to_string() } else { "--f:0".to_string() },
                            i { class: "ln", "aria-hidden": "true" }
                            a { href: "#{s.item.id}", "aria-current": if i == cur { "location" },
                                onclick: move |e| { e.prevent_default(); let v = vp.peek().clone(); if let (Some(Some(h)), Some(v)) = (heads.peek().get(i).cloned(), v) { spawn(async move { if let (Ok(hr), Ok(vr), Ok(o)) = (h.get_client_rect().await, v.get_client_rect().await, v.get_scroll_offset().await) { let y = o.y + hr.origin.y - vr.origin.y - 8.0; let _ = v.scroll(PixelsVector2D::new(0.0, y.max(0.0)), ScrollBehavior::Smooth).await; } let _ = h.set_focus(false).await; }); } },
                                "{s.item.label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ────────────────────────────────────── PageHeader ──

/// Page header: a project header (breadcrumb, title, meta, actions, tabs with counts) that folds
/// into a compact bar once its content scrolls past `fold_at` px. Follow / Share / Copy link /
/// Archive report through a polite status region. Renders its own scroll body (`height`).
/// Oxyn plus: an agent-status pill and a `sticky: false` mode.
#[component]
pub fn PageHeader(
    /// Title.
    title: String,
    /// Path above the title.
    #[props(default)]
    crumbs: Vec<Crumb>,
    /// Meta line.
    #[props(default)]
    meta: String,
    /// Tabs (with counts).
    #[props(default)]
    tabs: Vec<Tab>,
    /// Fold threshold px.
    #[props(default = 56.0)]
    fold_at: f32,
    /// Body height px.
    #[props(default = 340.0)]
    height: f32,
    /// Agent status pill text.
    #[props(default)]
    status: Option<String>,
    /// Start folded.
    #[props(default)]
    folded: bool,
) -> Element {
    let mut fold = use_signal(|| folded);
    let mut following = use_signal(|| false);
    let mut live = use_signal(String::new);
    let f = fold();
    rsx! {
        div { class: cls(&["oxyn-phead", if f { "folded" } else { "" }]), style: "height:{height}px",
            header { class: "phd",
                if !crumbs.is_empty() { div { class: "pcr", Breadcrumb { items: crumbs.clone() } } }
                div { class: "prow",
                    div { class: "pti",
                        h2 { class: "pt", "{title}" }
                        if let Some(s) = status.clone() { span { class: "pst", span { class: "d" } "{s}" } }
                    }
                    div { class: "pac",
                        button { class: "oxyn-btn secondary sm", r#type: "button", "aria-pressed": "{following()}", onclick: move |_| { let n = !following(); following.set(n); live.set(if n { "Following".into() } else { "Unfollowed".into() }) }, Icon { name: "bell", size: 13.0 } span { class: "lb", if following() { "Following" } else { "Follow" } } }
                        button { class: "oxyn-btn primary sm", r#type: "button", onclick: move |_| live.set("Share link copied".into()), Icon { name: "share", size: 13.0 } span { class: "lb", "Share" } }
                        super::DropdownMenu { items: vec![oxyn_core::primitives::menu::MenuItem::action("copy", "Copy link").icon("link"), oxyn_core::primitives::menu::MenuItem::action("archive", "Archive").icon("archive"), oxyn_core::primitives::menu::MenuItem::action("delete", "Delete").icon("trash").danger().disabled()], trigger: "More actions", icon: Some("more".to_string()), align: "end", on_action: move |id: String| live.set(if id == "copy" { "Link copied".into() } else { "Project archived".into() }) }
                    }
                }
                if !meta.is_empty() { div { class: "pmeta", "{meta}" } }
                if !tabs.is_empty() { div { class: "ptabs", Tabs { tabs: tabs.clone(), variant: "underline", size: "sm", label: "Project sections" } } }
            }
            div { class: "pbody", tabindex: "0", role: "region", "aria-label": "{title} content",
                onscroll: move |e| { let n = header_folded(e.scroll_top() as f32, fold_at, *fold.peek()); if n != *fold.peek() { fold.set(n) } },
                for i in 0..14 { div { key: "{i}", class: "prow-i", span { class: "pdot" } span { class: "ptx", {["Agent opened PR #482 · fix flaky auth test", "Run 247 finished · 12 files changed", "Maya approved the deploy to staging", "Atlas indexed 3,412 files", "Cost alert: run 251 passed $4"][i % 5]} } span { class: "ptm", "{i + 2}m" } } }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}
