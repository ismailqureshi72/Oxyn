//! Overlays: `Tooltip`, `Popover`, `DropdownMenu` (also a context menu).
//!
//! Timers run in CSS: an invisible element plays an animation of the wanted length and its
//! `animationend` advances the state (hover delay, the 300 ms neighbour window, hover-intent open
//! and close), so the timings hold without a host clock. Placement uses CSS anchor positioning
//! with flip fallbacks where the browser has it, and absolute side / align classes otherwise.

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

/// Shared tooltip group state: one open at a time, and a warm window after one closes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TipGroup {
    open: Option<u64>,
    warm: bool,
    next: u64,
}

/// Provides one tooltip delay group for everything inside (put it near the root).
#[component]
pub fn TooltipProvider(children: Element) -> Element {
    use_context_provider(|| Signal::new(TipGroup::default()));
    rsx! { {children} }
}

fn tip_group() -> Signal<TipGroup> {
    match try_consume_context::<Signal<TipGroup>>() {
        Some(s) => s,
        None => use_context_provider(|| Signal::new(TipGroup::default())),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TipPhase {
    Closed,
    Waiting,
    Open { instant: bool },
}

/// Tooltip: a small dark label for one control. Shows on hover after `delay` (500 ms) and at once
/// on keyboard focus; once one is open a neighbour within 300 ms shows instantly without the fade;
/// it hides at once; Esc hides it. Optional shortcut inside. Oxyn plus: configurable delay, a
/// rich title + body, side preference with flip.
#[component]
pub fn Tooltip(
    /// Text (becomes the trigger's description).
    content: String,
    /// Rich: a title line above the text.
    #[props(default)]
    title: Option<String>,
    /// Shortcut shown inside.
    #[props(default)]
    shortcut: Option<String>,
    /// `top` · `bottom` · `left` · `right`.
    #[props(default = "top".to_string())]
    side: String,
    /// `start` · `center` · `end`.
    #[props(default = "center".to_string())]
    align: String,
    /// Gap px (6).
    #[props(default = 6.0)]
    offset: f32,
    /// Hover delay ms (500).
    #[props(default = 500)]
    delay: u32,
    /// Never shows.
    #[props(default)]
    disabled: bool,
    /// Force open (review sheets) / controlled.
    #[props(default)]
    open: Option<bool>,
    /// Open state changed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
    /// The trigger.
    children: Element,
) -> Element {
    let mut group = tip_group();
    let mut me = use_signal(|| {
        let mut g = group.write();
        g.next += 1;
        g.next
    });
    let _ = &mut me;
    let id = *me.read();
    let mut phase = use_signal(|| TipPhase::Closed);
    let p = match open {
        Some(true) => TipPhase::Open { instant: true },
        Some(false) => TipPhase::Closed,
        None => *phase.read(),
    };
    let tid = format!("oxyn-tip-{id}");
    let mut show = move |instant: bool| {
        group.write().open = Some(id);
        phase.set(TipPhase::Open { instant });
        on_open_change.call(true);
    };
    let mut hide = move || {
        if matches!(*phase.peek(), TipPhase::Open { .. }) {
            let mut g = group.write();
            if g.open == Some(id) {
                g.open = None;
                g.warm = true;
            }
            on_open_change.call(false);
        }
        phase.set(TipPhase::Closed);
    };
    // Another tooltip in the group took over.
    if matches!(p, TipPhase::Open { .. }) && open.is_none() && group.read().open != Some(id) {
        phase.set(TipPhase::Closed);
    }
    let visible = matches!(p, TipPhase::Open { .. });
    let instant = matches!(p, TipPhase::Open { instant: true });
    let warm = group.read().warm;
    rsx! {
        span { class: "oxyn-tt", style: "--off:{offset}px;anchor-name:--{tid}",
            "aria-describedby": if visible { "{tid}" },
            onpointerenter: move |_| if !disabled {
                let g = *group.peek();
                if g.open.is_some() || g.warm { show(true) } else { phase.set(TipPhase::Waiting) }
            },
            onpointerleave: move |_| hide(),
            onpointerdown: move |_| hide(),
            onfocusin: move |_| if !disabled { let w = { let g = group.peek(); g.open.is_some() || g.warm }; show(w) },
            onfocusout: move |_| hide(),
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { hide() },
            {children}
            if p == TipPhase::Waiting { span { class: "oxyn-timer", style: "animation-duration:{delay}ms", onanimationend: move |_| show(false) } }
            if warm && !visible { span { class: "oxyn-timer", style: "animation-duration:300ms", onanimationend: move |_| group.write().warm = false } }
            if visible {
                span { id: "{tid}", class: cls(&["oxyn-tip", &side, &align, if instant { "instant" } else { "" }, if title.is_some() { "rich" } else { "" }]),
                    role: "tooltip", style: "position-anchor:--{tid}",
                    if let Some(t) = title.clone() { span { class: "tt", "{t}" } }
                    span { class: "tx", "{content}" }
                    if let Some(s) = shortcut.clone() { Keys { keys: s } }
                }
            }
        }
    }
}

/// Popover: a small floating card attached to its trigger. Title and quiet line, any content;
/// flips to stay on screen; closes on a press outside or Escape and hands focus back. Oxyn plus:
/// arrow on / off, hover-intent trigger, nested popovers (inner closes first).
#[component]
pub fn Popover(
    /// Trigger content (rendered inside a button).
    trigger: Element,
    /// Trigger's accessible name.
    trigger_label: String,
    /// Body.
    children: Element,
    /// Start open.
    #[props(default)]
    open: bool,
    /// `bottom` · `top` · `right` · `left`.
    #[props(default = "bottom".to_string())]
    side: String,
    /// `start` · `center` · `end`.
    #[props(default = "start".to_string())]
    align: String,
    /// Gap px (6).
    #[props(default = 6.0)]
    offset: f32,
    /// Title.
    #[props(default)]
    title: Option<String>,
    /// Quiet line.
    #[props(default)]
    description: Option<String>,
    /// Width px.
    #[props(default)]
    width: Option<f32>,
    /// Move focus into the card on open.
    #[props(default = true)]
    focus_on_open: bool,
    /// No card chrome (content draws its own).
    #[props(default)]
    unstyled: bool,
    /// `dialog` or another role.
    #[props(default = "dialog".to_string())]
    role: String,
    /// Arrow.
    #[props(default)]
    arrow: bool,
    /// `press` · `hover`.
    #[props(default = "press".to_string())]
    mode: String,
    /// Closed.
    #[props(default)]
    on_close: EventHandler<()>,
) -> Element {
    let mut is_open = use_signal(|| open);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut intent = use_signal(|| 0i8); // 1 = opening, -1 = closing
    let hover = mode == "hover";
    let id = format!(
        "oxyn-pop-{}",
        trigger_label
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    );
    let mut close = move |restore: bool| {
        if *is_open.peek() {
            is_open.set(false);
            on_close.call(());
            if restore && let Some(t) = trig.peek().clone() {
                spawn(async move {
                    let _ = t.set_focus(true).await;
                });
            }
        }
    };
    let w = width.map(|w| format!("width:{w}px")).unwrap_or_default();
    let o = *is_open.read();
    rsx! {
        span { class: "oxyn-popwrap", style: "--off:{offset}px",
            onpointerenter: move |_| if hover { intent.set(if *is_open.peek() { 0 } else { 1 }) },
            onpointerleave: move |_| if hover { intent.set(if *is_open.peek() { -1 } else { 0 }) },
            button { class: "oxyn-poptrig", r#type: "button", "aria-haspopup": "{role}", "aria-expanded": "{o}", "aria-controls": "{id}",
                "aria-label": "{trigger_label}", style: "anchor-name:--{id}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| if !hover { if *is_open.peek() { close(false) } else { is_open.set(true) } },
                {trigger}
            }
            if *intent.read() == 1 { span { class: "oxyn-timer", style: "animation-duration:150ms", onanimationend: move |_| { is_open.set(true); intent.set(0) } } }
            if *intent.read() == -1 { span { class: "oxyn-timer", style: "animation-duration:300ms", onanimationend: move |_| { close(false); intent.set(0) } } }
            if o {
                if !hover { div { class: "oxyn-backdrop", onpointerdown: move |_| close(false) } }
                div { id: "{id}", class: cls(&["oxyn-popover", &side, &align, if unstyled { "unstyled" } else { "" }, if arrow { "arrow" } else { "" }]),
                    role: "{role}", "aria-label": if let Some(t) = &title { "{t}" } else { "{trigger_label}" },
                    style: "{w};position-anchor:--{id}", tabindex: "-1",
                    onmounted: move |e| if focus_on_open { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); close(true) },
                    if arrow { span { class: "arw", "aria-hidden": "true" } }
                    if title.is_some() || description.is_some() {
                        div { class: "ph",
                            if let Some(t) = title.clone() { div { class: "pt", "{t}" } }
                            if let Some(d) = description.clone() { div { class: "pd", "{d}" } }
                        }
                    }
                    {children}
                }
            }
        }
    }
}

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
            div { class: "oxyn-backdrop", onpointerdown: move |_| { let o = m.write().close(false); outcome(o) },
                oncontextmenu: move |e| { e.prevent_default(); let o = m.write().close(false); outcome(o) } }
            div { class: cls(&["oxyn-menu", &side, &align, &density, if context { "ctx" } else { "" }]), style: "{w};{pos}",
                role: "menu", "aria-label": if let Some(t) = &title { "{t}" } else { "{trigger}" }, tabindex: "-1",
                onmounted: move |e| { g0.set_root(e.data()); let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
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
                                                        } else if let Some(ic) = it.icon.clone() { Icon { name: ic, size: 14.0 } }
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
                    on_open_change.call(true);
                },
                onkeydown: move |e: KeyboardEvent| if (e.key() == Key::F10 && e.modifiers().shift()) || e.key() == Key::ContextMenu {
                    e.prevent_default();
                    point.set(None);
                    m.write().open_menu(now, true, false);
                    sync();
                    on_open_change.call(true);
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
                onclick: move |_| { let was = m.peek().open; if was { let x = m.write().close(false); outcome(x) } else { m.write().open_menu(now, false, false); on_open_change.call(true) } },
                onkeydown: move |e: KeyboardEvent| if !m.peek().open && matches!(e.key(), Key::ArrowDown | Key::ArrowUp | Key::Enter) || (!m.peek().open && e.key() == Key::Character(" ".into())) {
                    e.prevent_default();
                    if let Some(k) = menu_key(&e.key()) { m.write().key(k, now); sync(); on_open_change.call(true); }
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
