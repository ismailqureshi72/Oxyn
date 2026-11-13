//! Buttons: `Button`, `IconButton`, `ActionButton`, `SplitButton`, `ButtonGroup`,
//! `FloatingButtonGroup`, `ExpandingButtonGroup`.

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

/// A square icon-only button: ghost, secondary or primary; five sizes (16–40 px); a pressed state
/// for toggles; its label as the tooltip. Oxyn plus: icon morph when pressed (`pressed_icon`),
/// and a dot / count badge.
#[component]
pub fn IconButton(
    /// Icon.
    icon: String,
    /// Accessible name and tooltip.
    label: String,
    /// `ghost` · `secondary` · `primary`.
    #[props(default = "ghost".to_string())]
    variant: String,
    /// `xs` (16) · `sm` (28) · `md` (32) · `lg` (36) · `xl` (40).
    #[props(default = "md".to_string())]
    size: String,
    /// Toggle state (`aria-pressed`); `None` = a plain action.
    #[props(default)]
    pressed: Option<bool>,
    /// Icon shown while pressed (copy → check, mic → stop).
    #[props(default)]
    pressed_icon: Option<String>,
    /// Badge: `Some(0)` = dot, `Some(n)` = count.
    #[props(default)]
    badge: Option<u64>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Review sheets: force a state.
    #[props(default)]
    state: Option<String>,
    /// Pressed.
    #[props(default)]
    onclick: EventHandler<()>,
) -> Element {
    let px: f32 = match size.as_str() {
        "xs" => 16.0,
        "sm" => 28.0,
        "lg" => 36.0,
        "xl" => 40.0,
        _ => 32.0,
    };
    let on = pressed.unwrap_or(false);
    let shown = if on {
        pressed_icon.clone().unwrap_or(icon.clone())
    } else {
        icon.clone()
    };
    let class = cls(&[
        "oxyn-ibtn",
        &variant,
        if on { "on" } else { "" },
        state.as_deref().unwrap_or(""),
    ]);
    rsx! {
        button { class: "{class}", r#type: "button", style: "--s:{px}px", title: "{label}", "aria-label": "{label}",
            "aria-pressed": if let Some(p) = pressed { "{p}" },
            disabled,
            onclick: move |_| onclick.call(()),
            span { class: "ic", key: "{shown}", Icon { name: shown.clone(), size: (px * 0.5).max(10.0) } }
            match badge {
                Some(0) => rsx! { span { class: "bdg dot", "aria-hidden": "true" } },
                Some(n) => rsx! { span { class: "bdg", "aria-hidden": "true", "{oxyn_core::primitives::display::badge_count(n, 99)}" } },
                None => rsx! {},
            }
        }
    }
}

/// Action button (Arc): pending → success with a drawn check → back after `reset_after`; error
/// with retry back-off (Oxyn plus). Drive it with the core model and the clock.
#[component]
pub fn ActionButton(
    /// Model.
    model: ActionModel,
    /// Clock.
    now: ClockMs,
    /// Icon at rest.
    #[props(default = "up".to_string())]
    icon: String,
    /// Shortcut hint chip (Oxyn plus).
    #[props(default)]
    shortcut: Option<String>,
    /// Pressed (ignored while pending or backing off).
    #[props(default)]
    on_press: EventHandler<()>,
) -> Element {
    let phase = model.phase(now);
    let text = model.text(now);
    let widest = model.widest().to_string();
    let (c, ic) = match &phase {
        ActionPhase::Idle => ("idle", Some(icon.clone())),
        ActionPhase::Pending => ("pending", None),
        ActionPhase::Success => ("success", Some("check".to_string())),
        ActionPhase::Error { .. } => ("error", Some("retry".to_string())),
    };
    let blocked = matches!(
        phase,
        ActionPhase::Pending | ActionPhase::Error { retry_in: Some(_) }
    );
    rsx! {
        button { class: "oxyn-btn secondary sm action {c}", r#type: "button", "aria-busy": "{phase == ActionPhase::Pending}",
            "aria-disabled": "{blocked}",
            onclick: move |_| if !blocked { on_press.call(()) },
            if phase == ActionPhase::Pending { Spin { size: 13.0 } } else if let Some(i) = ic { span { class: "aic", key: "{c}", Icon { name: i, size: 13.0, stroke: 2.0 } } }
            span { class: "slot", span { class: "ghost", "aria-hidden": "true", "{widest}" } span { class: "lb", key: "{text}", "{text}" } }
            if let Some(s) = shortcut { Keys { keys: s } }
            span { class: "sr", role: "status", "aria-live": "polite", if phase == ActionPhase::Success { "{text}" } }
        }
    }
}

/// Split button: the main action and a menu of alternatives. The main half morphs when the
/// remembered choice changes (Oxyn plus); the menu half never scales.
#[component]
pub fn SplitButton(
    /// Actions `(id, label)`; the first is the default main action.
    actions: Vec<(String, String)>,
    /// `primary` · `secondary`.
    #[props(default = "primary".to_string())]
    variant: String,
    /// Main icon.
    #[props(default)]
    icon: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Promote the last chosen alternative to the main half.
    #[props(default)]
    remember: bool,
    /// Start open (review sheets).
    #[props(default)]
    open: bool,
    /// An action ran.
    #[props(default)]
    on_action: EventHandler<String>,
) -> Element {
    let pairs: Vec<(&str, &str)> = actions
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let mut model = use_signal(|| {
        let mut m = SplitModel::new(&pairs);
        m.remember = remember;
        m
    });
    let mut menu = use_signal(|| {
        let mut m = Menu::new(vec![]);
        m.open = open;
        m
    });
    let glide = use_glide();
    let (main_id, main_label) = {
        let m = model.read();
        let (a, b) = m.main();
        (a.to_string(), b.to_string())
    };
    let items: Vec<MenuItem> = model
        .read()
        .menu()
        .iter()
        .map(|(a, b)| MenuItem::action(a, b))
        .collect();
    let is_open = menu.read().open;
    let class = cls(&[
        "oxyn-split",
        &variant,
        if disabled { "disabled" } else { "" },
    ]);
    let main_c = main_id.clone();
    rsx! {
        div { class: "{class}", role: "group", "aria-label": "{main_label}",
            button { class: "main", r#type: "button", disabled,
                onclick: move |_| on_action.call(main_c.clone()),
                if let Some(i) = icon.clone() { Icon { name: i, size: 14.0 } }
                span { class: "lb", key: "{main_id}", "{main_label}" }
            }
            span { class: "div", "aria-hidden": "true" }
            button { class: "more", r#type: "button", disabled, "aria-label": "More actions", "aria-haspopup": "menu", "aria-expanded": "{is_open}",
                onclick: move |_| { let o = !menu.peek().open; if o { menu.write().open_menu(ClockMs(0), false, false) } else { menu.write().close(true); } },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    Key::ArrowDown | Key::ArrowUp => { e.prevent_default(); let last = e.key() == Key::ArrowUp; menu.write().open_menu(ClockMs(0), true, last); let h = menu.peek().highlighted().map(|x| x.1); glide.go(h, true); }
                    Key::Escape => { menu.write().close(true); }
                    _ => {}
                },
                Icon { name: "chevron", size: 12.0, stroke: 2.0 }
            }
            if is_open {
                MenuList { items, glide, level: 0, labelled: "More actions".to_string(),
                    on_pick: move |id: String| { model.write().choose(&id); menu.write().close(true); glide.go(None, false); on_action.call(id) },
                    on_close: move |_| { menu.write().close(true); },
                }
            }
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

/// Floating button group (Arc): separate soft buttons in a quiet tray, one highlight that morphs
/// between them. Oxyn plus: auto-hide on idle (`auto_hide`), highlight style (pill / underline /
/// glow).
#[component]
pub fn FloatingButtonGroup(
    /// Buttons.
    items: Vec<GroupItem>,
    /// `muted` · `floating`.
    #[props(default = "floating".to_string())]
    variant: String,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Vertical tray.
    #[props(default)]
    vertical: bool,
    /// Icons only with label tooltips.
    #[props(default = true)]
    icon_only: bool,
    /// Tooltip side.
    #[props(default = "top".to_string())]
    tooltip_side: String,
    /// `pill` · `underline` · `glow`.
    #[props(default = "pill".to_string())]
    highlight_style: String,
    /// Fade the tray out when idle.
    #[props(default)]
    auto_hide: bool,
    /// Review sheets: force the highlight.
    #[props(default)]
    highlight: Option<usize>,
    /// Pressed.
    #[props(default)]
    on_action: EventHandler<String>,
) -> Element {
    let glide = use_glide();
    let mut tip = use_signal(|| None::<usize>);
    let n = items.len();
    let en: Vec<bool> = items.iter().map(|g| !g.disabled).collect();
    let hl = highlight.or(*glide.at.read());
    let class = cls(&[
        "oxyn-tray",
        &variant,
        &size,
        &highlight_style,
        if vertical { "vertical" } else { "" },
        if auto_hide { "autohide" } else { "" },
    ]);
    rsx! {
        div { class: "{class}", role: "toolbar", "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            onmounted: move |e| glide.set_root(e.data()),
            onpointerleave: move |_| { glide.go(None, false); tip.set(None); },
            onkeydown: move |e: KeyboardEvent| if let Some(k) = nav_key(&e.key(), vertical) {
                e.prevent_default();
                let to = step(*glide.at.peek(), k, n, true, |i| en[i]);
                glide.go(to, true);
                tip.set(to);
            },
            span { class: "{glide.class(\"hl\")}", style: "{glide.style()}", "aria-hidden": "true" }
            for (i, it) in items.iter().enumerate() {
                {
                    let id = it.id.clone();
                    let on = hl == Some(i);
                    rsx! {
                        div { class: "slot",
                            button { class: cls(&["tb", if on && !glide.measured() { "hl-self" } else { "" }]), r#type: "button",
                                "aria-label": "{it.label}", disabled: it.disabled,
                                onmounted: move |e| glide.set_item(i, e.data()),
                                onpointerenter: move |_| { glide.go(Some(i), false); tip.set(Some(i)); },
                                onfocus: move |_| { glide.go(Some(i), true); tip.set(Some(i)); },
                                onclick: move |_| on_action.call(id.clone()),
                                if let Some(ic) = it.icon.clone() { Icon { name: ic, size: 16.0 } }
                                if !icon_only { span { class: "lb", "{it.label}" } }
                            }
                            if icon_only && (*tip.read() == Some(i) || on && highlight.is_some()) {
                                span { class: "oxyn-tip {tooltip_side} instant", role: "tooltip", "{it.label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Expanding button group (Arc): the pointed-at or focused icon button grows to reveal its label
/// while its neighbours slide aside; an action confirms in place without changing width. Oxyn
/// plus: hover delay (`delay_ms`), always-expanded labels toggle (`labels`).
#[component]
pub fn ExpandingButtonGroup(
    /// Buttons (`done_label` = confirmation).
    items: Vec<GroupItem>,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Expanded on load.
    #[props(default)]
    default_expanded: Option<usize>,
    /// Labels always shown.
    #[props(default)]
    labels: bool,
    /// Hover delay before expanding.
    #[props(default)]
    delay_ms: u64,
    /// Review sheets: show item `i` as confirmed.
    #[props(default)]
    confirmed: Option<usize>,
    /// Clock (confirmation hold).
    #[props(default)]
    now: ClockMs,
    /// Pressed.
    #[props(default)]
    on_action: EventHandler<String>,
) -> Element {
    let triples: Vec<(String, String, String)> = items
        .iter()
        .map(|g| {
            (
                g.id.clone(),
                g.label.clone(),
                g.done_label.clone().unwrap_or(g.label.clone()),
            )
        })
        .collect();
    let mut model = use_signal(|| {
        let t: Vec<(&str, &str, &str)> = triples
            .iter()
            .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
            .collect();
        let mut g = ExpandingGroup::new(&t);
        g.expanded = default_expanded;
        g.always = labels;
        g.delay = std::time::Duration::from_millis(delay_ms);
        g
    });
    let mut open = use_signal(|| default_expanded);
    let mut done = use_signal(|| confirmed);
    let n = items.len();
    rsx! {
        div { class: "oxyn-expand {size}", role: "toolbar",
            style: "--delay:{delay_ms}ms",
            onpointerleave: move |_| { model.write().point(None, now); open.set(None); },
            onkeydown: move |e: KeyboardEvent| if let Some(k) = nav_key(&e.key(), false) {
                e.prevent_default();
                let to = step(*open.peek(), k, n, true, |_| true);
                open.set(to);
            },
            for (i, it) in items.iter().enumerate() {
                {
                    let id = it.id.clone();
                    let is_done = *done.read() == Some(i);
                    let shows = labels || *open.read() == Some(i) || is_done;
                    let slot = model.read().slot(i);
                    let text = if is_done { it.done_label.clone().unwrap_or(it.label.clone()) } else { it.label.clone() };
                    rsx! {
                        button { class: cls(&["xb", if shows { "open" } else { "" }, if is_done { "done" } else { "" }]), r#type: "button",
                            "aria-label": "{it.label}", style: "--slot:{slot}ch",
                            onpointerenter: move |_| { model.write().point(Some(i), now); open.set(Some(i)); },
                            onfocus: move |_| open.set(Some(i)),
                            onclick: move |_| { model.write().act(i, now); done.set(Some(i)); on_action.call(id.clone()) },
                            span { class: "ic", onanimationend: move |_| done.set(None),
                                if is_done { Icon { name: "check", size: 14.0, stroke: 2.0 } } else if let Some(ic) = it.icon.clone() { Icon { name: ic, size: 15.0 } } }
                            span { class: "lbw", span { class: "lb", "aria-hidden": "true", "{text}" } }
                        }
                    }
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", if let Some(i) = *done.read() { "{items.get(i).and_then(|g| g.done_label.clone()).unwrap_or_default()}" } }
        }
    }
}
