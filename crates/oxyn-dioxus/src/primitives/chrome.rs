//! App chrome (plan #27): `UserMenu`, `PresenceDot`, `TeamSwitcher`, `WorkspaceTile`,
//! `NotificationCenter` and `CodeBlock`.
//!
//! Logic lives in `oxyn_core::primitives::chrome`. Menus and panels render below their trigger
//! inside a positioned wrapper (no portal), so they work in contained previews. A menu that
//! starts open draws no click-outside layer and takes no focus; only a user-opened menu does.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        chrome::{
            Note, NoteView, NotificationCenter as CenterModel, Presence, SWITCH_DELAY, SwitcherRow,
            SwitcherView, TeamSwitcher as SwitcherModel, ThemePref, UserMenu as MenuModel, UserRow,
            Workspace, account_label, code_collapses, highlight, language_label,
        },
        layout::initials,
        nav::NavKey,
    },
};

use super::{CopyButton, cls, nav_key};
use crate::composer::icons::Icon;

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

// ───────────────────────────────────────── User menu ──

/// The presence mark on its own: each status has its own shape as well as colour (available a
/// dot, busy a bar, away a half moon, offline a ring).
#[component]
pub fn PresenceDot(
    /// `available` · `busy` · `away` · `offline`.
    status: String,
) -> Element {
    let p = Presence::from_id(&status);
    rsx! { span { class: "oxyn-presence {p.id()}", role: "img", "aria-label": "{p.label()}" } }
}

/// User menu: the account menu behind an avatar. Identity header (name, email, plan badge),
/// account links with shortcut hints, an inline theme switch (light · dark · system) and an
/// optional status switch that keep the menu open, and Sign out (async: spinner, `aria-busy`).
/// Enter / Space / ↓ open on the first row, ↑ on the last; ↑ ↓ Home End loop; ← → change the
/// switch; A–Z typeahead; Escape / Tab close and return focus. Below 640 px the panel becomes a
/// bottom sheet. Oxyn plus: an agent-usage meter row.
#[component]
pub fn UserMenu(
    /// Name.
    name: String,
    /// Email.
    email: String,
    /// Plan badge.
    #[props(default)]
    plan: Option<String>,
    /// Links: (label, icon, shortcut).
    #[props(default)]
    items: Vec<(String, String, String)>,
    /// Name + chevron beside the avatar.
    #[props(default)]
    show_name: bool,
    /// Theme switch.
    #[props(default = true)]
    show_theme: bool,
    /// Status switch + presence dot.
    #[props(default)]
    show_status: bool,
    /// Theme.
    #[props(default = "system".to_string())]
    theme: String,
    /// Status.
    #[props(default = "available".to_string())]
    status: String,
    /// Oxyn plus: usage (used, limit) shown as a meter.
    #[props(default)]
    usage: Option<(f32, f32)>,
    /// `start` · `end`.
    #[props(default = "end".to_string())]
    align: String,
    /// `down` · `up` (from a sidebar footer).
    #[props(default = "down".to_string())]
    side: String,
    /// Start open (no focus, no backdrop).
    #[props(default)]
    open: bool,
    /// Theme chosen.
    #[props(default)]
    on_theme: EventHandler<String>,
    /// Link chosen.
    #[props(default)]
    on_select: EventHandler<String>,
    /// Sign out.
    #[props(default)]
    on_sign_out: EventHandler<()>,
) -> Element {
    let labels: Vec<&str> = items.iter().map(|x| x.0.as_str()).collect();
    let mut m = use_signal(|| {
        let mut u = MenuModel::new(&labels);
        u.show_theme = show_theme;
        u.show_status = show_status;
        u.theme = ThemePref::from_id(&theme);
        u.status = Presence::from_id(&status);
        u.open = open;
        u
    });
    let mut user_opened = use_signal(|| false);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut panel = use_signal(|| None::<Rc<MountedData>>);
    let u = m.read().clone();
    let rows = u.rows();
    let mut close = move |restore: bool| {
        m.write().close();
        user_opened.set(false);
        if restore {
            focus(trig.peek().clone());
        }
    };
    let ini = initials(&name);
    let trig_label = account_label(&name, if show_status { Some(u.status) } else { None });
    rsx! {
        div { class: cls(&["oxyn-umenu", &align, &side, if u.open { "open" } else { "" }]),
            button { class: cls(&["um-trg", if show_name { "named" } else { "" }]), r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{u.open}",
                "aria-label": "{trig_label}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { if m.peek().open { close(false) } else { m.write().open_menu(false, false); user_opened.set(true); focus(panel.peek().clone()) } },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    Key::ArrowDown | Key::Enter => { e.prevent_default(); m.write().open_menu(true, false); user_opened.set(true); focus(panel.peek().clone()) }
                    Key::ArrowUp => { e.prevent_default(); m.write().open_menu(true, true); user_opened.set(true); focus(panel.peek().clone()) }
                    _ => {}
                },
                span { class: "um-av", "{ini}" if show_status { PresenceDot { status: u.status.id() } } }
                if show_name { span { class: "um-nm", "{name}" } Icon { name: "chevron", size: 12.0 } }
            }
            if u.open && user_opened() { div { class: "um-out", onclick: move |_| close(false) } }
            if u.open {
                div { class: "um-pnl", role: "menu", tabindex: "-1", "aria-label": "Account",
                    onmounted: move |e| panel.set(Some(e.data())),
                    onkeydown: move |e: KeyboardEvent| {
                        let k = e.key();
                        if let Some(nk) = nav_key(&k, true) { e.prevent_default(); m.write().key(nk); return }
                        match k {
                            Key::ArrowLeft => { e.prevent_default(); if m.write().cycle(false) { on_theme.call(m.peek().theme.id().to_string()) } }
                            Key::ArrowRight => { e.prevent_default(); if m.write().cycle(true) { on_theme.call(m.peek().theme.id().to_string()) } }
                            Key::Escape | Key::Tab => { e.prevent_default(); close(true) }
                            Key::Enter => {
                                e.prevent_default();
                                let r = m.peek().highlight.and_then(|h| m.peek().rows().get(h).cloned());
                                match r { Some(UserRow::Item(i)) => { on_select.call(m.peek().items[i].clone()); close(true) } Some(UserRow::SignOut) => { m.write().sign_out(ClockMs(0)); on_sign_out.call(()) } _ => {} }
                            }
                            Key::Character(c) if c.chars().count() == 1 && c != " " => { if let Some(ch) = c.chars().next() { m.write().type_char(ch, ClockMs(0)) } }
                            _ => {}
                        }
                    },
                    div { class: "um-id",
                        span { class: "um-av lg", "{ini}" }
                        div { class: "um-tx",
                            div { class: "um-n", "{name}" if let Some(p) = plan.clone() { span { class: "um-plan", "{p}" } } }
                            div { class: "um-e", "{email}" }
                        }
                    }
                    if let Some((used, lim)) = usage {
                        div { class: "um-use", "aria-label": "Agent usage {used:.0} of {lim:.0} credits",
                            div { class: "ul", span { "Agent credits" } span { "{used:.0} / {lim:.0}" } }
                            div { class: "ub", i { style: "width:{(used / lim * 100.0).min(100.0):.1}%" } }
                        }
                    }
                    div { class: "um-sep" }
                    for (ri, r) in rows.iter().cloned().enumerate() {
                        match r {
                            UserRow::Item(i) => {
                                let (lb, ic, kb) = items[i].clone();
                                let l2 = lb.clone();
                                rsx! {
                                    button { key: "i{i}", class: cls(&["um-row", if u.highlight == Some(ri) { "is-hl" } else { "" }]), r#type: "button", role: "menuitem", tabindex: "-1",
                                        onpointermove: move |_| if m.peek().highlight != Some(ri) { m.write().highlight = Some(ri) },
                                        onclick: move |_| { on_select.call(l2.clone()); close(true) },
                                        Icon { name: ic, size: 14.0 } span { class: "lb", "{lb}" }
                                        if !kb.is_empty() { span { class: "oxyn-keys", "aria-hidden": "true", for (k, part) in kb.split(' ').enumerate() { kbd { key: "{k}", "{part}" } } } }
                                    }
                                }
                            }
                            UserRow::Theme => rsx! {
                                div { key: "theme-{ri}", class: cls(&["um-row sw", if u.highlight == Some(ri) { "is-hl" } else { "" }]), role: "group", "aria-label": "Theme",
                                    onpointermove: move |_| if m.peek().highlight != Some(ri) { m.write().highlight = Some(ri) },
                                    Icon { name: u.theme.icon(), size: 14.0 } span { class: "lb", "Theme" }
                                    div { class: "um-seg",
                                        for t in ThemePref::ALL {
                                            button { key: "{t.id()}", r#type: "button", role: "menuitemradio", tabindex: "-1", "aria-checked": "{u.theme == t}", "aria-label": "{t.id()}", class: if u.theme == t { "on" } else { "" },
                                                onclick: move |_| { m.write().theme = t; on_theme.call(t.id().to_string()) }, Icon { name: t.icon(), size: 12.0 } }
                                        }
                                    }
                                }
                            },
                            UserRow::Status => rsx! {
                                div { key: "status-{ri}", class: cls(&["um-row sw", if u.highlight == Some(ri) { "is-hl" } else { "" }]), role: "group", "aria-label": "Status",
                                    onpointermove: move |_| if m.peek().highlight != Some(ri) { m.write().highlight = Some(ri) },
                                    PresenceDot { status: u.status.id() } span { class: "lb", "{u.status.label()}" }
                                    div { class: "um-seg",
                                        for p in Presence::CHOICES {
                                            button { key: "{p.id()}", r#type: "button", role: "menuitemradio", tabindex: "-1", "aria-checked": "{u.status == p}", "aria-label": "{p.label()}", class: if u.status == p { "on" } else { "" },
                                                onclick: move |_| m.write().status = p, PresenceDot { status: p.id() } }
                                        }
                                    }
                                }
                            },
                            UserRow::SignOut => rsx! {
                                div { key: "so-sep-{ri}", class: "um-sep" }
                                button { class: cls(&["um-row", if u.highlight == Some(ri) { "is-hl" } else { "" }]), r#type: "button", role: "menuitem", tabindex: "-1", "aria-busy": "{u.signing_out.is_some()}",
                                    onpointermove: move |_| if m.peek().highlight != Some(ri) { m.write().highlight = Some(ri) },
                                    onclick: move |_| { m.write().sign_out(ClockMs(0)); on_sign_out.call(()) },
                                    if u.signing_out.is_some() { span { class: "oxyn-spn", style: "--s:13px" } } else { Icon { name: "external", size: 14.0 } }
                                    span { class: "lb", if u.signing_out.is_some() { "Signing out…" } else { "Sign out" } }
                                    if u.signing_out.is_some() { span { class: "oxyn-timer", style: "animation-duration:1200ms", onanimationend: move |_| { m.write().signing_out = None; close(true) } } }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────── Team switcher ──

/// A workspace tile: the first letter on its identity hue (or a neutral tile).
#[component]
pub fn WorkspaceTile(
    /// Workspace.
    workspace: Workspace,
    /// Size px.
    #[props(default = 28.0)]
    size: f32,
) -> Element {
    let hue = workspace
        .tile
        .clone()
        .map(|h| format!("--hue:var(--oxyn-color-agent-{h});"))
        .unwrap_or_default();
    rsx! { span { class: cls(&["oxyn-wtile", if workspace.tile.is_some() { "hued" } else { "" }]), "aria-hidden": "true", style: "{hue}width:{size}px;height:{size}px;font-size:{size * 0.46}px;border-radius:{size * 0.28}px", "{workspace.letter()}" } }
}

/// Team switcher: the current workspace's tile, name and plan; a panel with a search combobox
/// (filters by name, plan or role; `aria-activedescendant`), roles, ⌘1–9 shortcuts, a check on the
/// current workspace, and inline Create / Join views. Switching crossfades the trigger through a
/// shimmer and spinner (`aria-busy`) until it settles. Escape goes back, then clears, then closes.
/// Oxyn plus: `compact` (tile only, for the icon rail) and member counts.
#[component]
pub fn TeamSwitcher(
    /// Workspaces.
    workspaces: Vec<Workspace>,
    /// Active id.
    #[props(default)]
    value: Option<String>,
    /// Tile only.
    #[props(default)]
    compact: bool,
    /// Start open.
    #[props(default)]
    open: bool,
    /// Simulated switch ms.
    #[props(default = 650)]
    delay: u32,
    /// Switched (id).
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = SwitcherModel::new(workspaces.clone(), value.as_deref());
        if open {
            s.open_menu();
        }
        s
    });
    let mut user_opened = use_signal(|| false);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut input = use_signal(|| None::<Rc<MountedData>>);
    let mut live = use_signal(String::new);
    let s = m.read().clone();
    let rows = s.rows();
    let cur = s.current().cloned();
    let shown = s
        .switching
        .as_ref()
        .and_then(|(id, _)| s.workspaces.iter().find(|w| &w.id == id).cloned())
        .or(cur.clone());
    let mut close = move || {
        m.write().open = false;
        user_opened.set(false);
        focus(trig.peek().clone());
    };
    let _ = SWITCH_DELAY;
    rsx! {
        div { class: cls(&["oxyn-team", if compact { "compact" } else { "" }, if s.open { "open" } else { "" }]),
            onkeydown: move |e: KeyboardEvent| {
                let md = e.modifiers().meta() || e.modifiers().ctrl();
                if let Key::Character(c) = e.key() && md && let Ok(n) = c.parse::<usize>() {
                    e.prevent_default();
                    if let Some(id) = m.write().shortcut(n, ClockMs(0)) { live.set(format!("Switching to {}", id)); }
                }
            },
            button { class: "tm-trg", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{s.open}", "aria-busy": "{s.switching.is_some()}", "aria-label": "{s.trigger_label()}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { if m.peek().open { close() } else { m.write().open_menu(); user_opened.set(true); focus(input.peek().clone()) } },
                onkeydown: move |e: KeyboardEvent| if matches!(e.key(), Key::ArrowDown) { e.prevent_default(); m.write().open_menu(); user_opened.set(true); focus(input.peek().clone()) },
                if let Some(w) = shown.clone() {
                    for _k in std::iter::once(w.id.clone()) {
                        span { key: "{_k}", class: cls(&["tm-cur", if s.switching.is_some() { "sw" } else { "" }]),
                            WorkspaceTile { workspace: w.clone(), size: 26.0 }
                            if !compact {
                                span { class: "tm-tx",
                                    span { class: "tm-n", "{w.name}" }
                                    span { class: "tm-p", if s.switching.is_some() { "Switching…" } else { {w.plan.clone().unwrap_or_default()} } }
                                }
                            }
                        }
                    }
                }
                if !compact { if s.switching.is_some() { span { class: "oxyn-spn", style: "--s:12px" } } else { Icon { name: "chevron", size: 12.0 } } }
            }
            if let Some((_, _)) = s.switching.clone() {
                span { class: "oxyn-timer", style: "animation-duration:{delay}ms", onanimationend: move |_| { let r = m.write().settle(ClockMs(u64::MAX / 2), std::time::Duration::from_millis(0)); if let Some(id) = r { live.set(format!("Switched to {id}")); on_change.call(id) } } }
            }
            if s.open && user_opened() { div { class: "um-out", onclick: move |_| close() } }
            if s.open {
                div { class: "tm-pnl", role: "dialog", "aria-label": "Switch workspace",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape {
                        e.prevent_default();
                        let esc = m.write().escape();
                        if esc && !m.peek().open { close() }
                    },
                    match s.view {
                        SwitcherView::List => rsx! {
                            div { class: "tm-search",
                                Icon { name: "search", size: 13.0 }
                                input { r#type: "text", role: "combobox", "aria-expanded": "true", "aria-controls": "oxyn-tm-list", "aria-autocomplete": "list",
                                    "aria-activedescendant": "oxyn-tm-r{s.highlight}", placeholder: "Find workspace…", value: "{s.query}", "aria-label": "Find workspace",
                                    onmounted: move |e| input.set(Some(e.data())),
                                    oninput: move |e| m.write().set_query(&e.value()),
                                    onkeydown: move |e: KeyboardEvent| {
                                        if let Some(nk) = nav_key(&e.key(), true) { if matches!(nk, NavKey::First | NavKey::Last) && !m.peek().query.is_empty() { return } e.prevent_default(); m.write().key(nk) }
                                        else if e.key() == Key::Enter { e.prevent_default(); if let Some(id) = m.write().enter(ClockMs(0)) { live.set(format!("Switching to {id}")); close() } }
                                    }
                                }
                            }
                            ul { class: "tm-list", id: "oxyn-tm-list", role: "listbox", "aria-label": "Workspaces",
                                for (ri, r) in rows.iter().cloned().enumerate() {
                                    match r {
                                        SwitcherRow::Workspace(i) => {
                                            let w = s.workspaces[i].clone();
                                            let idc = w.id.clone();
                                            let on = w.id == s.active;
                                            rsx! {
                                                li { key: "{w.id}", id: "oxyn-tm-r{ri}", role: "option", "aria-selected": "{on}", class: cls(&["tm-row", if s.highlight == ri { "is-hl" } else { "" }]),
                                                    onpointermove: move |_| if m.peek().highlight != ri { m.write().highlight = ri },
                                                    onclick: move |_| { if let Some(id) = m.write().switch_to(&idc, ClockMs(0)) { live.set(format!("Switching to {id}")) } close() },
                                                    WorkspaceTile { workspace: w.clone(), size: 22.0 }
                                                    span { class: "tm-tx", span { class: "tm-n", "{w.name}" } span { class: "tm-p", {[w.plan.clone(), w.role.clone(), w.members.map(|n| format!("{n} members"))].into_iter().flatten().collect::<Vec<_>>().join(" · ")} } }
                                                    if on { Icon { name: "check", size: 13.0, stroke: 2.0 } }
                                                    if i < 9 { span { class: "oxyn-keys", "aria-hidden": "true", kbd { "⌘" } kbd { "{i + 1}" } } }
                                                }
                                            }
                                        }
                                        SwitcherRow::Create => rsx! {
                                            li { key: "create-{ri}", id: "oxyn-tm-r{ri}", role: "option", "aria-selected": "false", class: cls(&["tm-row act", if s.highlight == ri { "is-hl" } else { "" }]),
                                                onpointermove: move |_| if m.peek().highlight != ri { m.write().highlight = ri },
                                                onclick: move |_| { let q = m.peek().query.clone(); m.write().draft = q; m.write().view = SwitcherView::Create; },
                                                Icon { name: "plus", size: 13.0 } span { class: "tm-n", if s.query.is_empty() { "Create workspace" } else { "Create “{s.query}”" } }
                                            }
                                        },
                                        SwitcherRow::Join => rsx! {
                                            li { key: "join-{ri}", id: "oxyn-tm-r{ri}", role: "option", "aria-selected": "false", class: cls(&["tm-row act", if s.highlight == ri { "is-hl" } else { "" }]),
                                                onpointermove: move |_| if m.peek().highlight != ri { m.write().highlight = ri },
                                                onclick: move |_| { m.write().draft.clear(); m.write().view = SwitcherView::Join; },
                                                Icon { name: "link", size: 13.0 } span { class: "tm-n", "Join with invite" }
                                            }
                                        },
                                    }
                                }
                            }
                        },
                        v => rsx! {
                            div { class: "tm-form",
                                button { class: "tm-back", r#type: "button", onclick: move |_| m.write().view = SwitcherView::List, Icon { name: "chevron-left", size: 13.0 } "Back" }
                                label { class: "tm-l", if v == SwitcherView::Create { "Workspace name" } else { "Invite link or code" } }
                                input { class: "tm-in", r#type: "text", value: "{s.draft}", placeholder: if v == SwitcherView::Create { "Acme Research" } else { "oxyn.dev/i/…" },
                                    oninput: move |e| m.write().draft = e.value(),
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); if m.peek().view == SwitcherView::Create { if let Some(id) = m.write().create(ClockMs(0)) { live.set(format!("Created {id}")); close() } } else { live.set("Simulated, nothing was sent".into()); m.write().view = SwitcherView::List } },
                                }
                                button { class: "oxyn-btn primary sm full", r#type: "button", disabled: s.draft.trim().is_empty(),
                                    onclick: move |_| { if m.peek().view == SwitcherView::Create { if let Some(id) = m.write().create(ClockMs(0)) { live.set(format!("Created {id}")); close() } } else { live.set("Simulated, nothing was sent".into()); m.write().view = SwitcherView::List } },
                                    if v == SwitcherView::Create { "Create workspace" } else { "Join" }
                                }
                            }
                        },
                    }
                }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}

// ─────────────────────────────── Notification center ──

/// Notification center: a bell with an unread badge (count in the label, badge `aria-hidden`)
/// opening a panel with All / Unread views (`aria-pressed`), grouped rows that expand in place
/// (marking read), Mark all read, Clear and per-row dismiss; focus moves to the next row after a
/// dismiss. The bell tilts while open; counts roll. Oxyn plus: kind icons (approval, run,
/// mention) and an `inline` panel mode for sidebars.
#[component]
pub fn NotificationCenter(
    /// Notifications.
    items: Vec<Note>,
    /// Start open.
    #[props(default)]
    open: bool,
    /// Panel inline (no trigger).
    #[props(default)]
    inline: bool,
    /// `all` · `unread`.
    #[props(default = "all".to_string())]
    view: String,
    /// Row opened (id).
    #[props(default)]
    on_open: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut c = CenterModel::new(items.clone());
        c.open = open || inline;
        c.view = if view == "unread" {
            NoteView::Unread
        } else {
            NoteView::All
        };
        c
    });
    let mut user_opened = use_signal(|| false);
    let mut rows = use_signal(Vec::<(String, Rc<MountedData>)>::new);
    let c = m.read().clone();
    let unread = c.unread();
    let groups = c.groups();
    let kind_icon = |k: &str| match k {
        "approval" => "hand",
        "run" => "play",
        "mention" => "comment",
        _ => "bell",
    };
    rsx! {
        div { class: cls(&["oxyn-ncenter", if c.open { "open" } else { "" }, if inline { "inline" } else { "" }]),
            if !inline {
                button { class: "nc-bell", r#type: "button", "aria-label": "{c.trigger_label()}", "aria-expanded": "{c.open}",
                    onclick: move |_| { let o = !m.peek().open; m.write().open = o; user_opened.set(o) },
                    Icon { name: "bell", size: 16.0 }
                    if unread > 0 { span { class: "nc-badge", key: "{unread}", "aria-hidden": "true", "{unread}" } }
                }
                if c.open && user_opened() { div { class: "um-out", onclick: move |_| { m.write().open = false; user_opened.set(false) } } }
            }
            if c.open {
                div { class: "nc-pnl", role: "region", "aria-label": "Notifications",
                    div { class: "nc-hd",
                        div { class: "nc-ttl", "Notifications" span { class: "nc-sum", "aria-live": "polite", key: "{unread}", "{c.summary()}" } }
                        div { class: "nc-views",
                            button { r#type: "button", "aria-pressed": "{c.view == NoteView::All}", class: if c.view == NoteView::All { "on" } else { "" }, onclick: move |_| m.write().view = NoteView::All, "All" }
                            button { r#type: "button", "aria-pressed": "{c.view == NoteView::Unread}", class: if c.view == NoteView::Unread { "on" } else { "" }, onclick: move |_| m.write().view = NoteView::Unread, "Unread" if unread > 0 { span { class: "n", "{unread}" } } }
                        }
                    }
                    div { class: "nc-list",
                        if groups.is_empty() {
                            div { class: "nc-empty", Icon { name: "inbox", size: 18.0 } span { "You're all caught up" } }
                        }
                        for (g, notes) in groups.iter().cloned() {
                            div { key: "{g}", class: "nc-grp",
                                div { class: "nc-gl", "{g}" }
                                for (k, n) in notes.iter().cloned().enumerate() {
                                    div { key: "{n.id}", class: cls(&["nc-row", if n.unread { "unread" } else { "" }, if c.expanded.as_ref() == Some(&n.id) { "open" } else { "" }]), style: "--i:{k}",
                                        button { class: "nc-main", r#type: "button", "aria-expanded": "{c.expanded.as_ref() == Some(&n.id)}",
                                            "aria-label": if n.unread { "{n.title}, unread" } else { "{n.title}" },
                                            onmounted: { let id = n.id.clone(); move |e: MountedEvent| { let mut r = rows.write(); r.retain(|(k, _)| *k != id); r.push((id.clone(), e.data())); } },
                                            onclick: { let id = n.id.clone(); move |_| { m.write().toggle(&id); on_open.call(id.clone()) } },
                                            span { class: "nc-ic k-{n.kind}", Icon { name: kind_icon(&n.kind), size: 13.0 } }
                                            span { class: "nc-tx",
                                                span { class: "nc-t", strong { "{n.actor}" } " {n.title}" }
                                                span { class: "nc-time", "{n.time}" }
                                            }
                                            if n.unread { span { class: "nc-dot", "aria-hidden": "true" } }
                                        }
                                        div { class: "nc-body", div { class: "nc-bi", p { "{n.body}" } } }
                                        button { class: "oxyn-x nc-x", r#type: "button", "aria-label": "Dismiss {n.title}",
                                            onclick: { let id = n.id.clone(); move |_| { let next = m.write().dismiss(&id); if let Some(nx) = next && let Some((_, d)) = rows.peek().iter().find(|(k, _)| *k == nx).cloned() { focus(Some(d)) } } },
                                            Icon { name: "x", size: 11.0, stroke: 2.0 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "nc-ft",
                        button { class: "oxyn-btn ghost xs", r#type: "button", disabled: unread == 0, onclick: move |_| { m.write().mark_all(); }, Icon { name: "check", size: 12.0 } "Mark all read" }
                        button { class: "oxyn-btn ghost xs", r#type: "button", disabled: c.items.is_empty(), onclick: move |_| m.write().clear(), "Clear" }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────── Code block ──

/// Code block: a source viewer with a filename header, language label, light highlighting, a
/// [`CopyButton`] and optional collapse (`max_lines` → Show all, `aria-expanded`). The `pre` is
/// focusable so it can scroll. Ports Beautiful UI's Code Block header (MIT, © 2026 Shane Levine)
/// and Arc UI's behaviour. Oxyn plus: line numbers, highlighted lines, `diff`, wrap toggle and a
/// streaming caret.
#[component]
pub fn CodeBlock(
    /// Source.
    code: String,
    /// File name.
    #[props(default)]
    filename: Option<String>,
    /// Language id (`rust` · `ts` · `tsx` · `py` · `sh` · `json` · `diff`).
    #[props(default = "tsx".to_string())]
    language: String,
    /// Collapse beyond this many lines (0 = never).
    #[props(default)]
    max_lines: usize,
    /// Line numbers.
    #[props(default)]
    line_numbers: bool,
    /// 1-based lines to highlight.
    #[props(default)]
    highlight_lines: Vec<usize>,
    /// Wrap long lines.
    #[props(default)]
    wrap: bool,
    /// Streaming caret after the last line.
    #[props(default)]
    streaming: bool,
    /// `default` · `bare` (no header).
    #[props(default = "default".to_string())]
    variant: String,
) -> Element {
    let mut open = use_signal(|| false);
    let mut wrapping = use_signal(|| wrap);
    let lines = highlight(&code, &language);
    let collapses = code_collapses(&code, max_lines);
    let shown = if collapses && !open() {
        max_lines
    } else {
        lines.len()
    };
    let name = filename.clone().unwrap_or_else(|| "Source code".into());
    let id = format!(
        "oxyn-code-{}",
        name.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
    );
    rsx! {
        section { class: cls(&["oxyn-code", &variant, if wrapping() { "cb-wrapped" } else { "" }, if collapses && !open() { "clamped" } else { "" }]), "aria-label": "{name} source code",
            if variant != "bare" {
                div { class: "cb-hd",
                    Icon { name: "code", size: 13.0 }
                    span { class: "cb-fn", "{name}" }
                    span { class: "cb-lang", "{language_label(&language)}" }
                    span { class: "cb-sp" }
                    button { class: "cb-wrap", r#type: "button", "aria-pressed": "{wrapping()}", "aria-label": "Wrap lines", title: "Wrap lines", onclick: move |_| { let w = !wrapping(); wrapping.set(w) }, Icon { name: "text", size: 13.0 } }
                    CopyButton { value: code.clone(), icon_only: true, variant: "plain", size: "sm" }
                }
            }
            pre { class: "cb-pre", id: "{id}", tabindex: "0",
                code {
                    for (i, ln) in lines.iter().take(shown).enumerate() {
                        span { key: "{i}", class: cls(&["ln", if highlight_lines.contains(&(i + 1)) { "is-hl" } else { "" }, ln.first().map(|t| match t.0 { oxyn_core::primitives::chrome::Tok::Add => "add", oxyn_core::primitives::chrome::Tok::Del => "del", _ => "" }).unwrap_or("")]),
                            if line_numbers { span { class: "no", "aria-hidden": "true", "{i + 1}" } }
                            span { class: "src",
                                for (k, (t, s)) in ln.iter().enumerate() { span { key: "{k}", class: "t-{t.id()}", "{s}" } }
                                if streaming && i + 1 == shown { span { class: "caret", "aria-hidden": "true" } }
                                "\n"
                            }
                        }
                    }
                }
            }
            if collapses {
                button { class: "cb-more", r#type: "button", "aria-expanded": "{open()}", "aria-controls": "{id}", onclick: move |_| { let o = !open(); open.set(o) },
                    for _k in std::iter::once(open()) { span { key: "{_k}", class: "cb-l", if open() { "Show less" } else { "Show all {lines.len()} lines" } } }
                    Icon { name: "chevron", size: 12.0 }
                }
            }
        }
    }
}
