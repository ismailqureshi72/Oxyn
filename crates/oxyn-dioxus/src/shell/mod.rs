//! App shells and screens (plan #27, chunk 4): `Sidebar`, `SidebarRail`, `WorkspaceSidebar`,
//! `InboxSidebar`, `AppShell` (flat · inset), `AiSidePanel`, `TerminalTabs`, `SidebarNav`,
//! `ChatShell` (the frame of the AI Chat screens) and `Harness` (a demo frame with Fork this).
//!
//! Logic lives in `oxyn_core::shell`; styles in [`SHELL_CSS`], bundled into `PRIMITIVES_CSS`.
//! Behaviour follows Arc UI (Sidebar, Workspace sidebar, Sidebar rail, Inbox sidebar, AI side
//! panel) and affectUI (App Shell Flat / Inset, AI Chat screens), rebuilt from their docs with no
//! source used, and Beautiful UI's Sidebar Nav (MIT, © 2026 Shane Levine) without its paid icons.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use dioxus::prelude::*;
use oxyn_core::shell::{
    Block, DiffOp, EditPhase, Folder, Inbox, Mail, NavItem, NavSection, PageEdit, RAIL_W, Rail,
    TermTabs, filter, parent_of, shift, trail,
};

use crate::composer::Icon;

/// Stylesheet for the shells.
pub const SHELL_CSS: &str = include_str!("shell.css");

fn cls(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect()
}

/// Keys that fold a rail: `[` when focus is not in a field.
fn is_fold_key(e: &KeyboardEvent) -> bool {
    e.key() == Key::Character("[".into())
        && !e.modifiers().ctrl()
        && !e.modifiers().meta()
        && !e.modifiers().alt()
}

/// Workspace sidebar (Arc UI Workspace sidebar): a workspace switcher menu, live search,
/// favourites you reorder with Alt+↑/↓, inline new projects (Enter creates, Escape cancels), a
/// storage meter and an account row; folds to the rail with `[`.
#[component]
pub fn WorkspaceSidebar(
    /// Workspaces (name, plan line).
    #[props(default = vec![("Northwind".to_string(), "Business · 24 members".to_string()), ("Fieldnote".to_string(), "Team · 8 members".to_string()), ("Personal".to_string(), "Free".to_string())])]
    workspaces: Vec<(String, String)>,
    /// Favourites.
    #[props(default = vec!["Checkout flows".to_string(), "Critique notes".to_string(), "Q4 roadmap".to_string(), "Component audit".to_string()])]
    favorites: Vec<String>,
    /// Projects.
    #[props(default = vec!["Checkout redesign".to_string(), "Mobile app 4.0".to_string(), "Design system".to_string(), "Onboarding".to_string(), "Help center".to_string()])]
    projects: Vec<String>,
    /// Initial search text.
    #[props(default)]
    query: String,
    /// Start with the switcher open.
    #[props(default)]
    menu_open: bool,
    /// Start folded.
    #[props(default)]
    collapsed: bool,
    /// Storage used (0–1).
    #[props(default = 0.38)]
    storage: f32,
    /// Height (px).
    #[props(default = 480.0)]
    height: f32,
    /// A favourite or project was opened.
    #[props(default)]
    on_navigate: EventHandler<String>,
) -> Element {
    let mut ws = use_signal(|| 0usize);
    let mut menu = use_signal(|| menu_open);
    let mut q = use_signal(|| query.clone());
    let mut favs = use_signal(|| favorites.clone());
    let mut projs = use_signal(|| projects.clone());
    let mut adding = use_signal(|| None::<String>);
    let mut current = use_signal(|| "Checkout redesign".to_string());
    let mut rail = use_signal(|| collapsed);
    let mut live = use_signal(String::new);
    let r = rail();
    let (wname, wplan) = workspaces.get(ws()).cloned().unwrap_or_default();
    let fv = favs.read().clone();
    let pv = projs.read().clone();
    let ff = filter(&fv, &q());
    let pf = filter(&pv, &q());
    rsx! {
        nav { class: cls(&["oxyn-sidebar", "oxyn-wsbar", if r { "rail" } else { "" }]), "aria-label": "Workspace", style: "height:{height}px", tabindex: "-1",
            onkeydown: move |e: KeyboardEvent| if is_fold_key(&e) { e.prevent_default(); let n = !rail(); rail.set(n); live.set(if n { "Sidebar folded".into() } else { "Sidebar expanded".into() }) },
            div { class: "ws-head",
                button { class: "ws-switch", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu()}", "aria-label": "Switch workspace, current {wname}",
                    onclick: move |_| { let n = !menu(); menu.set(n) },
                    span { class: "sh-logo", "aria-hidden": "true", "{initials(&wname)}" }
                    if !r { span { class: "sh-bn", b { "{wname}" } small { "{wplan}" } } span { class: "sh-chev open", "aria-hidden": "true", Icon { name: "chevron", size: 12.0 } } }
                }
                if menu() {
                    div { class: "ws-menu", role: "menu", "aria-label": "Workspaces",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { menu.set(false) },
                        for (i, (n, p)) in workspaces.iter().cloned().enumerate() {
                            button { key: "{n}", class: cls(&["ws-mi", if i == ws() { "on" } else { "" }]), r#type: "button", role: "menuitemradio", "aria-checked": "{i == ws()}",
                                onclick: move |_| { ws.set(i); menu.set(false); live.set(format!("Switched to {n}")) },
                                span { class: "sh-logo sm", "aria-hidden": "true", "{initials(&n)}" }
                                span { class: "sh-bn", b { "{n}" } small { "{p}" } }
                                if i == ws() { Icon { name: "check", size: 13.0 } }
                            }
                        }
                    }
                }
            }
            if !r {
                label { class: "ws-search",
                    Icon { name: "search", size: 13.0 }
                    input { r#type: "search", placeholder: "Search favorites and projects", "aria-label": "Search favorites and projects", value: "{q}",
                        oninput: move |e| q.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| { if e.key() == Key::Escape { q.set(String::new()) } e.stop_propagation() },
                    }
                }
            }
            div { class: "sh-scroll",
                div { class: "sh-sec",
                    if !r { div { class: "sh-sech", "Favorites" } }
                    ul { class: "sh-list", role: "list", "aria-label": "Favorites", "aria-describedby": "oxyn-ws-hint",
                        for (i, f) in ff.iter().map(|(i, f)| (*i, (*f).clone())) {
                            li { key: "{f}",
                                button { class: cls(&["sh-row", if current() == f { "on" } else { "" }]), r#type: "button", title: if r { "{f}" }, "aria-current": if current() == f { "page" },
                                    onclick: { let f = f.clone(); move |_| { current.set(f.clone()); on_navigate.call(f.clone()) } },
                                    onkeydown: { let f = f.clone(); move |e: KeyboardEvent| if e.modifiers().alt() && (e.key() == Key::ArrowUp || e.key() == Key::ArrowDown) {
                                        e.prevent_default();
                                        let d = if e.key() == Key::ArrowUp { -1 } else { 1 };
                                        let j = shift(&mut favs.write(), i, d);
                                        live.set(format!("{f} moved to position {}", j + 1));
                                    } },
                                    span { class: "sh-ic", Icon { name: "star", size: 14.0 } }
                                    if !r { span { class: "sh-lb", "{f}" } }
                                }
                            }
                        }
                    }
                    if !r { p { class: "sh-hint", id: "oxyn-ws-hint", "Alt + ↑ / ↓ reorders a favorite." } }
                }
                div { class: "sh-sec",
                    if !r { div { class: "sh-sech", "Projects" } }
                    ul { class: "sh-list", role: "list", "aria-label": "Projects",
                        for (_, p) in pf.iter().map(|(i, p)| (*i, (*p).clone())) {
                            li { key: "{p}",
                                button { class: cls(&["sh-row", if current() == p { "on" } else { "" }]), r#type: "button", title: if r { "{p}" }, "aria-current": if current() == p { "page" },
                                    onclick: { let p = p.clone(); move |_| { current.set(p.clone()); on_navigate.call(p.clone()) } },
                                    span { class: "sh-ic", Icon { name: "folder", size: 14.0 } }
                                    if !r { span { class: "sh-lb", "{p}" } }
                                }
                            }
                        }
                        if ff.is_empty() && pf.is_empty() && !r { li { class: "sh-empty", "Nothing matches “{q}”" } }
                    }
                    if !r {
                        if let Some(name) = adding() {
                            input { class: "ws-new", value: "{name}", placeholder: "Project name", "aria-label": "New project name", "aria-describedby": "oxyn-ws-new-hint", autofocus: true,
                                oninput: move |e| adding.set(Some(e.value())),
                                onkeydown: move |e: KeyboardEvent| { e.stop_propagation(); match e.key() {
                                    Key::Enter => { let n = adding().unwrap_or_default(); if !n.trim().is_empty() { projs.write().push(n.trim().to_string()); live.set(format!("Project {} created", n.trim())) } adding.set(None) }
                                    Key::Escape => { adding.set(None); live.set("New project cancelled".into()) }
                                    _ => {}
                                } },
                            }
                            p { class: "sh-hint", id: "oxyn-ws-new-hint", "Enter creates the project, Escape cancels." }
                        } else {
                            button { class: "sh-row add", r#type: "button", onclick: move |_| adding.set(Some(String::new())), span { class: "sh-ic", Icon { name: "plus", size: 14.0 } } span { class: "sh-lb", "New project" } }
                        }
                    }
                }
            }
            div { class: "sh-foot",
                if !r {
                    div { class: "ws-store", role: "meter", "aria-label": "Storage", "aria-valuenow": "{(storage * 100.0).round()}", "aria-valuemin": "0", "aria-valuemax": "100",
                        div { class: "ws-sl", span { "Storage" } span { "{(storage * 100.0).round()}%" } }
                        div { class: "ws-bar", i { style: "width:{storage * 100.0}%" } }
                        small { "{storage * 100.0:.1} GB of 100 GB" }
                    }
                }
                div { class: "sh-user", span { class: "sh-av", "aria-hidden": "true", "EC" } if !r { span { class: "sh-un", b { "Emma Collins" } small { span { class: "ws-on" } "Active" } } } }
            }
            div { class: "sr", role: "status", "{live}" }
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
