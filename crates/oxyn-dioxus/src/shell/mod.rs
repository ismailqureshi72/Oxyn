//! App shells and screens (plan #27, chunk 4): `Sidebar`, `SidebarRail`, `WorkspaceSidebar`,
//! `InboxSidebar`, `AppShell` (flat · inset), `AiSidePanel`, `TerminalTabs`, `SidebarNav`,
//! `ChatShell` (the frame of the AI Chat screens) and `Harness` (a demo frame with Fork this).
//!
//! Logic lives in `oxyn_core::shell`; styles in [`SHELL_CSS`], bundled into `PRIMITIVES_CSS`.
//! Behaviour follows Arc UI (Sidebar, Workspace sidebar, Sidebar rail, Inbox sidebar, AI side
//! panel) and affectUI (App Shell Flat / Inset, AI Chat screens), rebuilt from their docs with no
//! source used, and Beautiful UI's Sidebar Nav (MIT, © 2026 Shane Levine) without its paid icons.

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

/// Sample navigation (Northline, product design).
pub fn sample_sections() -> Vec<NavSection> {
    vec![
        NavSection::new(
            "",
            vec![
                NavItem::new("home", "Home", "home"),
                NavItem::new("inbox", "Inbox", "inbox").count(4),
            ],
        ),
        NavSection::new(
            "Workspace",
            vec![
                NavItem::new("projects", "Projects", "folder").with(vec![
                    NavItem::new("checkout", "Checkout redesign", ""),
                    NavItem::new("onboarding", "Mobile onboarding", ""),
                    NavItem::new("ds", "Design system 2.0", ""),
                ]),
                NavItem::new("roadmap", "Roadmap", "calendar"),
                NavItem::new("research", "Research", "search"),
            ],
        ),
        NavSection::new(
            "Pinned",
            vec![
                NavItem::new("q4", "Q4 launch plan", "star"),
                NavItem::new("critique", "Critique notes", "text"),
            ],
        ),
    ]
}

/// Keys that fold a rail: `[` when focus is not in a field.
fn is_fold_key(e: &KeyboardEvent) -> bool {
    e.key() == Key::Character("[".into())
        && !e.modifiers().ctrl()
        && !e.modifiers().meta()
        && !e.modifiers().alt()
}

#[component]
fn NavRows(
    sections: Vec<NavSection>,
    active: Signal<String>,
    open: Signal<Vec<String>>,
    rail: bool,
    live: Signal<String>,
    on_select: EventHandler<String>,
    /// Keep labels in the DOM while folded, so a rail transition can fade them.
    #[props(default)]
    keep: bool,
) -> Element {
    let show = !rail || keep;
    rsx! {
        for (si, s) in sections.iter().enumerate() {
            div { key: "{si}", class: "sh-sec",
                if !s.title.is_empty() && !rail { div { class: "sh-sech", "{s.title}" } }
                if !s.title.is_empty() && rail && si > 0 { div { class: "sh-secr", "aria-hidden": "true" } }
                ul { class: "sh-list", role: "list",
                    for (ii, it) in s.items.iter().cloned().enumerate() {
                        {
                            let has = !it.children.is_empty();
                            let is_open = open.read().contains(&it.id);
                            let cur = active() == it.id;
                            let child_cur = it.children.iter().any(|c| c.id == active());
                            let id = it.id.clone();
                            let label = it.label.clone();
                            rsx! {
                                li { key: "{it.id}",
                                    button { class: cls(&["sh-row", if cur { "on" } else { "" }, if child_cur && rail { "on" } else { "" }]), r#type: "button", style: "--i:{si * 6 + ii}",
                                        title: if rail { "{it.label}" },
                                        "aria-current": if cur { "page" },
                                        "aria-expanded": if has && !rail { if is_open { "true" } else { "false" } },
                                        "aria-label": if rail { if it.count > 0 { "{it.label}, {it.count} unread" } else { "{it.label}" } },
                                        onclick: move |_| {
                                            if has && !rail {
                                                let mut o = open.write();
                                                if let Some(p) = o.iter().position(|x| *x == id) { o.remove(p); } else { o.push(id.clone()); }
                                            } else {
                                                active.set(id.clone());
                                                live.set(format!("{label} opened"));
                                                on_select.call(id.clone());
                                            }
                                        },
                                        span { class: "sh-ic", Icon { name: if it.icon.is_empty() { "page".to_string() } else { it.icon.clone() }, size: 15.0 } }
                                        if show {
                                            span { class: "sh-lb", "aria-hidden": if rail { "true" }, "{it.label}" }
                                            if it.count > 0 { span { class: "sh-ct", "aria-label": "{it.count} unread", "{it.count}" } }
                                            if has { span { class: cls(&["sh-chev", if is_open { "open" } else { "" }]), "aria-hidden": "true", Icon { name: "chevron-right", size: 12.0 } } }
                                        }
                                        if rail && it.count > 0 { span { class: "sh-dot", "aria-hidden": "true" } }
                                    }
                                    if has && is_open && !rail {
                                        ul { class: "sh-sub", role: "list",
                                            for c in it.children.iter().cloned() {
                                                {
                                                    let cid = c.id.clone();
                                                    let cl = c.label.clone();
                                                    let ccur = active() == c.id;
                                                    rsx! {
                                                        li { key: "{c.id}",
                                                            button { class: cls(&["sh-row", "sub", if ccur { "on" } else { "" }]), r#type: "button", "aria-current": if ccur { "page" },
                                                                onclick: move |_| { active.set(cid.clone()); live.set(format!("{cl} opened")); on_select.call(cid.clone()) },
                                                                span { class: "sh-lb", "{c.label}" }
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
                    }
                }
            }
        }
    }
}

/// Sidebar (Arc UI Sidebar): brand header, sections with nested groups, pinned items, settings and
/// the account row; folds to an icon rail from its button or `[`.
#[component]
pub fn Sidebar(
    /// Sections.
    #[props(default = sample_sections())]
    sections: Vec<NavSection>,
    /// Active row id.
    #[props(default = "checkout".to_string())]
    active: String,
    /// Start folded to the rail.
    #[props(default)]
    collapsed: bool,
    /// Brand name.
    #[props(default = "Northline".to_string())]
    brand: String,
    /// Account name.
    #[props(default = "Emma Collins".to_string())]
    user: String,
    /// Height (px).
    #[props(default = 460.0)]
    height: f32,
    /// A row was opened.
    #[props(default)]
    on_select: EventHandler<String>,
) -> Element {
    let act = use_signal(|| active.clone());
    let open = use_signal(|| {
        parent_of(&sections, &active)
            .into_iter()
            .collect::<Vec<_>>()
    });
    let mut rail = use_signal(|| collapsed);
    let mut live = use_signal(String::new);
    let r = rail();
    rsx! {
        nav { class: cls(&["oxyn-sidebar", if r { "rail" } else { "" }]), "aria-label": "Main", style: "height:{height}px", tabindex: "-1",
            onkeydown: move |e: KeyboardEvent| if is_fold_key(&e) { e.prevent_default(); let n = !rail(); rail.set(n); live.set(if n { "Sidebar folded".into() } else { "Sidebar expanded".into() }) },
            div { class: "sh-brand",
                span { class: "sh-logo", "aria-hidden": "true", "{initials(&brand)}" }
                if !r { span { class: "sh-bn", b { "{brand}" } small { "Product design" } } }
                button { class: "sh-fold", r#type: "button", "aria-label": if r { "Expand sidebar" } else { "Fold sidebar" }, "aria-expanded": "{!r}", title: "Fold  [",
                    onclick: move |_| { let n = !rail(); rail.set(n); live.set(if n { "Sidebar folded".into() } else { "Sidebar expanded".into() }) },
                    Icon { name: if r { "chevron-right" } else { "chevron-left" }, size: 14.0 }
                }
            }
            div { class: "sh-scroll", NavRows { sections: sections.clone(), active: act, open, rail: r, live, on_select } }
            div { class: "sh-foot",
                button { class: "sh-row", r#type: "button", title: if r { "Settings" }, span { class: "sh-ic", Icon { name: "cog", size: 15.0 } } if !r { span { class: "sh-lb", "Settings" } } }
                div { class: "sh-user",
                    span { class: "sh-av", "aria-hidden": "true", "{initials(&user)}" }
                    if !r { span { class: "sh-un", b { "{user}" } small { "Product designer" } } }
                }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}

/// Sidebar rail (Arc UI Sidebar · App rail): an app shell whose sidebar drags wider or narrower
/// and snaps to a 64 px icon rail below its minimum; `[` folds it, the separator takes arrows,
/// and the breadcrumb follows the open page.
#[component]
pub fn SidebarRail(
    /// Sections.
    #[props(default = sample_sections())]
    sections: Vec<NavSection>,
    /// Active row id.
    #[props(default = "checkout".to_string())]
    active: String,
    /// Start folded.
    #[props(default)]
    collapsed: bool,
    /// Open width (px).
    #[props(default = 248.0)]
    width: f32,
    /// Height (px).
    #[props(default = 460.0)]
    height: f32,
    /// How labels leave on fold: `ease` (fade 160 ms while the width glides 400 ms), `cascade`
    /// (labels drift left and fade from the top down) or `scrub` (labels fade as you drag).
    #[props(default = "ease".to_string())]
    transition: String,
    /// Content.
    children: Element,
    /// Width changed (0 = folded to the rail).
    #[props(default)]
    on_resize: EventHandler<u32>,
) -> Element {
    let act = use_signal(|| active.clone());
    let open = use_signal(|| {
        parent_of(&sections, &active)
            .into_iter()
            .collect::<Vec<_>>()
    });
    let mut rail = use_signal(|| Rail {
        width,
        collapsed,
        ..Rail::default()
    });
    let mut drag = use_signal(|| None::<(f64, f32)>);
    let mut px_now = use_signal(|| width);
    let mut live = use_signal(String::new);
    let r = rail();
    let crumbs = trail(&sections, &act());
    let lo = ((px_now() - r.snap) / (r.min - r.snap)).clamp(0.0, 1.0);
    rsx! {
        div { class: cls(&["oxyn-railshell", &transition, if drag().is_some() { "dragging" } else { "" }]), style: "height:{height}px;--sw:{r.shown()}px;--lo:{lo:.2}",
            onkeydown: move |e: KeyboardEvent| if is_fold_key(&e) { e.prevent_default(); rail.write().toggle(); let c = rail.peek().collapsed; live.set(if c { "Sidebar folded to rail".into() } else { "Sidebar expanded".into() }); on_resize.call(rail.peek().value()) },
            onpointermove: move |e| if let Some((x0, w0)) = drag() { let x = w0 + (e.client_coordinates().x - x0) as f32; px_now.set(x); rail.write().drag(x) },
            onpointerup: move |_| if drag().is_some() { drag.set(None); on_resize.call(rail.peek().value()) },
            onpointerleave: move |_| if drag().is_some() { drag.set(None); on_resize.call(rail.peek().value()) },
            nav { class: cls(&["oxyn-sidebar", "inshell", if r.collapsed { "rail" } else { "" }]), "aria-label": "Main", id: "oxyn-railnav",
                div { class: "sh-brand",
                    span { class: "sh-logo", "aria-hidden": "true", "N" }
                    span { class: "sh-bn", "aria-hidden": if r.collapsed { "true" }, b { "Northline" } small { "Product design" } }
                }
                div { class: "sh-scroll", NavRows { sections: sections.clone(), active: act, open, rail: r.collapsed, live, keep: true, on_select: move |_| {} } }
            }
            div { class: cls(&["sh-sep", if drag().is_some() { "drag" } else { "" }]), role: "separator", tabindex: "0", "aria-orientation": "vertical", "aria-controls": "oxyn-railnav",
                "aria-label": "Resize sidebar", "aria-valuenow": "{r.value()}", "aria-valuemin": "0", "aria-valuemax": "{r.max}",
                "aria-valuetext": if r.collapsed { "Folded to icon rail".to_string() } else { format!("{} pixels", r.value()) },
                onpointerdown: move |e| { e.prevent_default(); let w0 = rail.peek().shown(); px_now.set(w0); drag.set(Some((e.client_coordinates().x, w0))) },
                ondoubleclick: move |_| { rail.write().toggle(); on_resize.call(rail.peek().value()) },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    Key::ArrowLeft => { e.prevent_default(); rail.write().nudge(-1); on_resize.call(rail.peek().value()) }
                    Key::ArrowRight => { e.prevent_default(); rail.write().nudge(1); on_resize.call(rail.peek().value()) }
                    Key::Enter => { rail.write().toggle(); on_resize.call(rail.peek().value()) }
                    _ => {}
                },
                span { class: "sh-tip", "aria-hidden": "true", "Drag to resize sidebar" }
            }
            main { class: "sh-main",
                header { class: "sh-top",
                    button { class: "sh-ib", r#type: "button", "aria-label": if r.collapsed { "Expand sidebar" } else { "Fold sidebar" }, "aria-expanded": "{!r.collapsed}", title: "[",
                        onclick: move |_| { rail.write().toggle(); on_resize.call(rail.peek().value()) },
                        Icon { name: "grip", size: 14.0 }
                    }
                    ol { class: "sh-crumbs", "aria-label": "Breadcrumb",
                        li { "Northline" }
                        for (i, c) in crumbs.iter().enumerate() {
                            li { key: "{act}-{i}", class: if i + 1 == crumbs.len() { "cur" } else { "" }, "aria-current": if i + 1 == crumbs.len() { "page" }, "{c}" }
                        }
                    }
                    span { class: "sh-w", "aria-hidden": "true", if r.collapsed { "rail · {RAIL_W as u32}px" } else { "{r.value()}px" } }
                }
                div { class: "sh-body", {children} }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
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

/// Sample mail.
pub fn sample_inbox() -> Inbox {
    Inbox::new(
        vec![
            Folder::new("inbox", "Inbox", "inbox"),
            Folder::new("starred", "Starred", "star"),
            Folder::new("snoozed", "Snoozed", "clock"),
            Folder::new("sent", "Sent", "up"),
            Folder::new("drafts", "Drafts", "pencil"),
            Folder::new("archive", "Archive", "archive"),
        ],
        vec![
            Mail::new(
                "m1",
                "Emma Collins",
                "Onboarding prototype, round two",
                "I tightened the handoff between steps two and three.",
                "9:42 AM",
            )
            .unread()
            .label("Design review"),
            Mail::new(
                "m2",
                "Ava Mitchell",
                "Launch checklist: four items left",
                "Pricing copy and the status page are the last blockers.",
                "9:10 AM",
            )
            .unread()
            .starred()
            .label("Launch"),
            Mail::new(
                "m3",
                "Marcus Johnson",
                "Menu and table states for Thursday",
                "Two interaction questions before the review.",
                "8:18 AM",
            )
            .unread()
            .label("Design review"),
            Mail::new(
                "m4",
                "Jasmine Brooks",
                "Preview build 412 is live",
                "Keyboard fixes and the reduced motion pass are in.",
                "Yesterday",
            )
            .label("Launch"),
            Mail::new(
                "m5",
                "Emma Collins",
                "Interview notes from Lisbon",
                "Six sessions, three strong patterns.",
                "Yesterday",
            )
            .label("Research"),
            Mail::new(
                "m6",
                "Marcus Johnson",
                "Portfolio review for the senior role",
                "Two candidates stood out.",
                "Sun",
            )
            .unread()
            .label("Hiring"),
        ],
    )
}

/// Inbox sidebar (Arc UI Inbox sidebar): folders with unread counts, labels, a message list you
/// drag onto folders (or move from the row menu), S stars and E archives the focused row, an undo
/// for the last move, and a Compose button that grows into a composer (⌘/Ctrl+Enter sends,
/// Escape saves a draft).
#[component]
pub fn InboxSidebar(
    /// Mail.
    #[props(default = sample_inbox())]
    inbox: Inbox,
    /// Start folded to the rail.
    #[props(default)]
    collapsed: bool,
    /// Start with the composer open.
    #[props(default)]
    composing: bool,
    /// Height (px).
    #[props(default = 480.0)]
    height: f32,
) -> Element {
    let mut ib = use_signal(|| inbox.clone());
    let mut rail = use_signal(|| collapsed);
    let mut compose = use_signal(|| composing);
    let mut draft = use_signal(String::new);
    let mut drafts = use_signal(|| 1usize);
    let mut dragging = use_signal(|| None::<String>);
    let mut over = use_signal(|| None::<String>);
    let mut menu = use_signal(|| None::<String>);
    let mut live = use_signal(String::new);
    let r = rail();
    let m = ib.read().clone();
    let open_label = m
        .folders
        .iter()
        .find(|f| f.id == m.open)
        .map(|f| f.label.clone())
        .unwrap_or_default();
    let shown: Vec<Mail> = m.visible().into_iter().cloned().collect();
    let unread = shown.iter().filter(|x| x.unread).count();
    let labels = [
        ("Design review", "#7c5cff"),
        ("Launch", "#e5484d"),
        ("Research", "#12a594"),
        ("Hiring", "#f5a524"),
    ];
    rsx! {
        div { class: cls(&["oxyn-inbox", if r { "rail" } else { "" }]), style: "height:{height}px",
            onkeydown: move |e: KeyboardEvent| if is_fold_key(&e) { e.prevent_default(); let n = !rail(); rail.set(n) },
            nav { class: cls(&["oxyn-sidebar", "inshell", if r { "rail" } else { "" }]), "aria-label": "Mail folders",
                div { class: "ib-compose-slot",
                    if !compose() {
                        button { class: "ib-compose", r#type: "button", "aria-haspopup": "dialog", title: if r { "Compose" }, onclick: move |_| compose.set(true),
                            Icon { name: "pencil", size: 14.0 } if !r { span { "Compose" } }
                        }
                    }
                }
                div { class: "sh-scroll",
                    ul { class: "sh-list", role: "list",
                        for f in m.folders.iter().cloned() {
                            {
                                let n = if f.id == "drafts" { drafts() } else { m.count(&f.id) };
                                let fid = f.id.clone();
                                let fid2 = f.id.clone();
                                let fl = f.label.clone();
                                let on = m.open == f.id;
                                let tgt = over().as_deref() == Some(f.id.as_str());
                                rsx! {
                                    li { key: "{f.id}",
                                        button { class: cls(&["sh-row", if on { "on" } else { "" }, if tgt { "drop" } else { "" }]), r#type: "button", title: if r { "{f.label}" }, "aria-current": if on { "page" },
                                            "aria-label": if n > 0 { format!("{}, {n} {}", f.label, if f.id == "inbox" { "unread" } else { "messages" }) } else { f.label.clone() },
                                            onclick: move |_| { ib.write().open = fid.clone(); live.set(format!("{fl} opened")) },
                                            ondragover: move |e| { e.prevent_default(); if over.peek().as_deref() != Some(fid2.as_str()) { over.set(Some(fid2.clone())) } },
                                            ondragleave: move |_| over.set(None),
                                            ondrop: { let fid = f.id.clone(); let fl = f.label.clone(); move |e: DragEvent| { e.prevent_default(); over.set(None); if let Some(id) = dragging() && ib.write().move_to(&id, &fid) { live.set(format!("Moved to {fl}")) } dragging.set(None) } },
                                            span { class: "sh-ic", Icon { name: f.icon.clone(), size: 15.0 } }
                                            if !r { span { class: "sh-lb", "{f.label}" } if n > 0 { span { class: "sh-ct", "{n}" } } } else if n > 0 && f.id == "inbox" { span { class: "sh-dot" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !r {
                        div { class: "sh-sech", "Labels" }
                        ul { class: "sh-list", role: "list",
                            for (l, c) in labels {
                                li { key: "{l}", div { class: "sh-row static", span { class: "ib-lc", style: "background:{c}" } span { class: "sh-lb", "{l}" } span { class: "sh-ct", "{m.mail.iter().filter(|x| x.label == l).count()}" } } }
                            }
                        }
                    }
                }
                div { class: "sh-foot", div { class: "sh-user", span { class: "sh-av", "aria-hidden": "true", "SR" } if !r { span { class: "sh-un", b { "Sofia Ramirez" } small { "sofia@fieldnote.studio" } } } } }
            }
            section { class: "ib-list", "aria-label": "{open_label}",
                header { class: "ib-lh", h3 { "{open_label}" } small { "{shown.len()} messages · {unread} unread" }
                    if m.last_move.is_some() { button { class: "ib-undo", r#type: "button", onclick: move |_| { if ib.write().undo() { live.set("Move undone".into()) } }, Icon { name: "undo", size: 12.0 } "Undo" } }
                }
                ul { class: "ib-rows", role: "list",
                    for x in shown.iter().cloned() {
                        {
                            let id = x.id.clone();
                            let id2 = x.id.clone();
                            let id3 = x.id.clone();
                            let id4 = x.id.clone();
                            let color = labels.iter().find(|(l, _)| *l == x.label).map(|(_, c)| *c).unwrap_or("transparent");
                            rsx! {
                                li { key: "{x.id}", class: cls(&["ib-row", if x.unread { "unread" } else { "" }, if dragging().as_deref() == Some(x.id.as_str()) { "dragging" } else { "" }]),
                                    draggable: "true", tabindex: "0",
                                    "aria-label": format!("{}, {}, {}{}{}", x.from, x.subject, x.when, if x.unread { ", unread" } else { "" }, if x.starred { ", starred" } else { "" }),
                                    ondragstart: move |_| dragging.set(Some(id.clone())),
                                    ondragend: move |_| { dragging.set(None); over.set(None) },
                                    onclick: move |_| ib.write().read(&id2),
                                    onkeydown: move |e: KeyboardEvent| match e.key() {
                                        Key::Character(c) if c == "s" || c == "S" => { let on = ib.write().star(&id3); live.set(if on { "Starred".into() } else { "Star removed".into() }) }
                                        Key::Character(c) if c == "e" || c == "E" => { if ib.write().move_to(&id3, "archive") { live.set("Archived".into()) } }
                                        Key::Escape => { dragging.set(None); menu.set(None) }
                                        _ => {}
                                    },
                                    span { class: "sh-av sm", "aria-hidden": "true", "{initials(&x.from)}" }
                                    div { class: "ib-tx",
                                        div { class: "ib-l1", b { "{x.from}" } span { class: "ib-when", "{x.when}" } }
                                        div { class: "ib-sub", if !x.label.is_empty() { span { class: "ib-lc", style: "background:{color}" } } "{x.subject}" }
                                        div { class: "ib-pre", "{x.preview}" }
                                    }
                                    div { class: "ib-acts",
                                        button { class: cls(&["sh-ib", if x.starred { "on" } else { "" }]), r#type: "button", "aria-label": if x.starred { "Remove star" } else { "Star" }, "aria-pressed": "{x.starred}",
                                            onclick: { let id = x.id.clone(); move |e: MouseEvent| { e.stop_propagation(); let on = ib.write().star(&id); live.set(if on { "Starred".into() } else { "Star removed".into() }) } },
                                            Icon { name: "star", size: 13.0 }
                                        }
                                        button { class: "sh-ib", r#type: "button", "aria-label": "Archive",
                                            onclick: { let id = x.id.clone(); move |e: MouseEvent| { e.stop_propagation(); if ib.write().move_to(&id, "archive") { live.set("Archived".into()) } } },
                                            Icon { name: "archive", size: 13.0 }
                                        }
                                        button { class: "sh-ib", r#type: "button", "aria-label": "Move to", "aria-haspopup": "menu", "aria-expanded": "{menu().as_deref() == Some(x.id.as_str())}",
                                            onclick: move |e: MouseEvent| { e.stop_propagation(); let open = menu.peek().as_deref() == Some(id4.as_str()); menu.set(if open { None } else { Some(id4.clone()) }) },
                                            Icon { name: "folder", size: 13.0 }
                                        }
                                        if menu().as_deref() == Some(x.id.as_str()) {
                                            div { class: "ib-menu", role: "menu", "aria-label": "Move to",
                                                for f in m.folders.iter().filter(|f| f.id != "starred" && f.id != x.folder).cloned() {
                                                    button { key: "{f.id}", class: "ws-mi", r#type: "button", role: "menuitem",
                                                        onclick: { let id = x.id.clone(); move |e: MouseEvent| { e.stop_propagation(); if ib.write().move_to(&id, &f.id) { live.set(format!("Moved to {}", f.label)) } menu.set(None) } },
                                                        Icon { name: f.icon.clone(), size: 13.0 } "{f.label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if shown.is_empty() { li { class: "sh-empty", "Nothing in {open_label}" } }
                }
                p { class: "sh-hint", "Drag a message onto a folder, or use Move to. S stars, E archives." }
            }
            if compose() {
                div { class: "ib-composer", role: "dialog", "aria-label": "New message",
                    onkeydown: move |e: KeyboardEvent| {
                        e.stop_propagation();
                        if e.key() == Key::Escape { if !draft().trim().is_empty() { drafts += 1; live.set("Draft saved".into()) } compose.set(false); draft.set(String::new()) }
                        if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) { e.prevent_default(); compose.set(false); draft.set(String::new()); live.set("Message sent (simulated)".into()) }
                    },
                    header { b { "New message" } button { class: "sh-ib", r#type: "button", "aria-label": "Close and save draft", onclick: move |_| { if !draft().trim().is_empty() { drafts += 1; live.set("Draft saved".into()) } compose.set(false); draft.set(String::new()) }, Icon { name: "x", size: 13.0 } } }
                    input { class: "ib-to", placeholder: "To", "aria-label": "To" }
                    textarea { rows: "4", placeholder: "Write a message…", "aria-label": "Message", value: "{draft}", autofocus: true, oninput: move |e| draft.set(e.value()) }
                    footer { small { "⌘ Enter sends · Esc saves a draft" } button { class: "ib-send", r#type: "button", onclick: move |_| { compose.set(false); draft.set(String::new()); live.set("Message sent (simulated)".into()) }, "Send" } }
                }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}

/// App shell (affectUI App Shell Flat / Inset): a sidebar, a header with the page title and
/// actions, and the content. `flat` keeps one plane split by hairlines; `inset` floats the
/// content on a rounded raised card inside the sidebar's tinted plane.
#[component]
pub fn AppShell(
    /// `flat` or `inset`.
    #[props(default = "inset".to_string())]
    variant: String,
    /// Page title.
    #[props(default = "Checkout redesign".to_string())]
    title: String,
    /// Sidebar (defaults to [`Sidebar`]).
    #[props(default)]
    sidebar: Option<Element>,
    /// Header actions.
    #[props(default)]
    actions: Option<Element>,
    /// Height (px).
    #[props(default = 460.0)]
    height: f32,
    /// Content.
    children: Element,
) -> Element {
    rsx! {
        div { class: cls(&["oxyn-appshell", &variant]), style: "height:{height}px",
            div { class: "as-side",
                if let Some(s) = sidebar { {s} } else { Sidebar { height } }
            }
            div { class: "as-main",
                header { class: "as-head",
                    h2 { "{title}" }
                    span { class: "sh-sp" }
                    if let Some(a) = actions { {a} } else {
                        button { class: "sh-ib", r#type: "button", "aria-label": "Search", Icon { name: "search", size: 14.0 } }
                        button { class: "sh-ib", r#type: "button", "aria-label": "Notifications", Icon { name: "bell", size: 14.0 } }
                    }
                }
                div { class: "as-body", {children} }
            }
        }
    }
}

/// Sample spec page for the AI side panel.
pub fn sample_page() -> Vec<Block> {
    vec![
        Block::new(
            "intro",
            "Intro",
            "Team workspaces are a new feature that will make it possible for people who work together to share their projects, files and billing in one single place, so that they no longer need to forward invoices around.",
        ),
        Block::new(
            "goals",
            "Goals",
            "Cut the time it takes to invite a team from six minutes to under one. Move 30% of paid solo accounts to a team plan by March.",
        ),
        Block::new(
            "pricing",
            "Pricing",
            "Per seat, billed monthly. Admins add or remove seats at any time and changes are prorated on the next invoice.",
        ),
    ]
}

/// AI side panel (Arc UI AI side panel): an assistant docked beside a page. Suggestions or a
/// prompt stream an answer; an answer that edits the page shows a word diff with Apply and
/// Discard; Apply tints the changed block and offers Undo. Escape closes the panel and the reopen
/// button brings it back.
#[component]
pub fn AiSidePanel(
    /// Host clock (ms).
    now_ms: u64,
    /// Page blocks.
    #[props(default = sample_page())]
    page: Vec<Block>,
    /// Start open.
    #[props(default = true)]
    open: bool,
    /// Start with the intro rewrite proposed (for demos).
    #[props(default)]
    proposed: bool,
    /// Assistant name.
    #[props(default = "Assistant".to_string())]
    assistant: String,
    /// Height (px).
    #[props(default = 480.0)]
    height: f32,
    /// A change was applied (block id).
    #[props(default)]
    on_apply: EventHandler<String>,
) -> Element {
    let tight = "Team workspaces let people who work together share projects, files and billing in one place, so nobody forwards invoices again.";
    let mut pe = use_signal(|| {
        let mut p = PageEdit::new(page.clone());
        if proposed {
            p.propose("intro", tight);
        }
        p
    });
    let mut is_open = use_signal(|| open);
    let mut asked = use_signal(|| {
        if proposed {
            Some((0u64, "Tighten the intro".to_string()))
        } else {
            None
        }
    });
    let mut tint = use_signal(|| None::<(String, u64)>);
    let mut draft = use_signal(String::new);
    let mut live = use_signal(String::new);
    let p = pe.read().clone();
    let answer = "The intro repeats itself. Here is a tighter version that keeps the promise and drops the filler.";
    let streamed = asked().map(|(t0, _)| {
        let words: Vec<&str> = answer.split(' ').collect();
        let n = if proposed && t0 == 0 {
            words.len()
        } else {
            ((now_ms.saturating_sub(t0)) / 60) as usize
        };
        (words[..n.min(words.len())].join(" "), n >= words.len())
    });
    if let Some((_, done)) = &streamed
        && *done
        && p.phase == EditPhase::None
    {
        pe.write().propose("intro", tight);
        live.set("Answer ready; it changes Intro".into());
    }
    let mut ask = move |q: String| {
        asked.set(Some((now_ms, q)));
        pe.write().phase = EditPhase::None;
        live.set("Reading the page".into());
    };
    rsx! {
        div { class: cls(&["oxyn-aipanel", if is_open() { "open" } else { "" }]), style: "height:{height}px",
            article { class: "ap-page", "aria-label": "Team workspaces",
                h2 { "Team workspaces" }
                small { "Product spec · Maya Chen · Edited Oct 18" }
                for b in p.blocks.iter() {
                    section { key: "{b.id}", class: cls(&["ap-block", if tint().is_some_and(|(id, t)| id == b.id && now_ms.saturating_sub(t) < 1400) { "tint" } else { "" }]),
                        h4 { "{b.label}" } p { key: "{b.text.len()}", class: "ap-tx", "{b.text}" }
                    }
                }
                if !is_open() {
                    button { class: "ap-reopen", r#type: "button", "aria-expanded": "false", "aria-controls": "oxyn-aipanel", onclick: move |_| is_open.set(true), Icon { name: "spark", size: 14.0 } "{assistant}" }
                }
            }
            if is_open() {
                aside { class: "ap-panel", id: "oxyn-aipanel", "aria-label": "{assistant}",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { is_open.set(false); live.set("Assistant closed".into()) },
                    header { class: "ap-h",
                        span { class: "ap-mark", "aria-hidden": "true", Icon { name: "spark", size: 14.0 } }
                        span { class: "sh-bn", b { "{assistant}" } small { "Claude Sonnet" } }
                        span { class: "sh-sp" }
                        button { class: "sh-ib", r#type: "button", "aria-label": "Close assistant", "aria-expanded": "true", "aria-controls": "oxyn-aipanel", onclick: move |_| is_open.set(false), Icon { name: "x", size: 13.0 } }
                    }
                    div { class: "ap-chips", span { class: "ap-chip", Icon { name: "page", size: 12.0 } "This page" } }
                    div { class: "ap-thread", role: "log", "aria-live": "polite",
                        if let Some((_, q)) = asked() {
                            div { class: "ap-q", "{q}" }
                            if let Some((t, done)) = &streamed { p { class: "ap-a", "aria-busy": "{!done}", "{t}" if !done { span { class: "ap-caret", "aria-hidden": "true" } } } }
                            if matches!(p.phase, EditPhase::Proposed | EditPhase::Applied | EditPhase::Discarded | EditPhase::Undone) {
                                div { class: cls(&["ap-card", match p.phase { EditPhase::Applied => "applied", EditPhase::Discarded => "discarded", _ => "" }]),
                                    div { class: "ap-ch", Icon { name: "pencil", size: 12.0 } "Intro" }
                                    p { class: "ap-diff",
                                        for (i, (op, w)) in p.diff().into_iter().enumerate() {
                                            match op { DiffOp::Same => rsx! { span { key: "{i}", "{w} " } }, DiffOp::Del => rsx! { del { key: "{i}", "{w}" } " " }, DiffOp::Ins => rsx! { ins { key: "{i}", "{w}" } " " } }
                                        }
                                    }
                                    footer {
                                        match p.phase {
                                            EditPhase::Proposed => rsx! {
                                                button { class: "ap-apply", r#type: "button", onclick: move |_| { if let Some(id) = pe.write().apply() { tint.set(Some((id.clone(), now_ms))); live.set("Applied to the page; Undo available".into()); on_apply.call(id) } }, Icon { name: "check", size: 12.0 } "Apply" }
                                                button { class: "ap-ghost", r#type: "button", onclick: move |_| { pe.write().discard(); live.set("Discarded".into()) }, "Discard" }
                                            },
                                            EditPhase::Applied => rsx! { span { class: "ap-st", Icon { name: "check-circle", size: 12.0 } "Applied to the page" } },
                                            EditPhase::Discarded => rsx! { span { class: "ap-st", Icon { name: "x-circle", size: 12.0 } "Discarded" } },
                                            _ => rsx! { span { class: "ap-st", Icon { name: "undo", size: 12.0 } "Undone" } },
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "ap-empty", "{assistant} reads this page and proposes edits. Nothing changes until you apply it." }
                        }
                    }
                    if asked().is_none() || matches!(p.phase, EditPhase::Applied | EditPhase::Discarded | EditPhase::Undone) {
                        div { class: "ap-sugs",
                            for s in ["Summarize this page", "Tighten the intro", "Add an annual price column"] {
                                button { key: "{s}", class: "ap-sug", r#type: "button", onclick: move |_| ask(s.to_string()), "{s}" }
                            }
                        }
                    }
                    if p.phase == EditPhase::Applied {
                        button { class: "ap-undo", r#type: "button", autofocus: true, onclick: move |_| { if pe.write().undo() { live.set("Undone".into()) } }, Icon { name: "undo", size: 12.0 } "Undo" }
                    }
                    div { class: "ap-comp",
                        input { placeholder: "Ask about Team workspaces", "aria-label": "Ask the assistant", value: "{draft}", oninput: move |e| draft.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !draft().trim().is_empty() { let q = draft(); draft.set(String::new()); ask(q) } }
                        button { class: "stsend", r#type: "button", "aria-label": "Send", disabled: draft().trim().is_empty(), onclick: move |_| { let q = draft(); draft.set(String::new()); ask(q) }, Icon { name: "up", size: 14.0, stroke: 2.0 } }
                    }
                }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}

/// Terminal tabs: a tab strip of shells with add, close, double-click rename, Ctrl+Tab cycling,
/// activity dots on background tabs that print, and exit codes.
#[component]
pub fn TerminalTabs(
    /// Host clock (ms).
    now_ms: u64,
    /// Tab titles.
    #[props(default = vec!["dev server".to_string(), "cargo test".to_string(), "zsh".to_string()])]
    titles: Vec<String>,
    /// Background tabs print a line every this many ms (0 = never).
    #[props(default = 2500)]
    activity_ms: u64,
    /// Height (px).
    #[props(default = 320.0)]
    height: f32,
) -> Element {
    let mut tt = use_signal(|| {
        let refs: Vec<&str> = titles.iter().map(|s| s.as_str()).collect();
        let mut t = TermTabs::new(&refs);
        let seed: [&[&str]; 3] = [
            &[
                "$ npm run dev",
                "  ▲ Next.js ready on http://localhost:3000",
                "  ✓ compiled in 412 ms",
            ],
            &[
                "$ cargo test -p auth",
                "running 48 tests",
                "test token::leeway ... ok",
            ],
            &[
                "$ git status",
                "On branch fix/token-leeway",
                "nothing to commit",
            ],
        ];
        for (i, tab) in t.tabs.iter_mut().enumerate() {
            tab.lines = seed[i % 3].iter().map(|s| s.to_string()).collect();
        }
        if let Some(tab) = t.tabs.get_mut(1) {
            tab.exit = Some(0);
        }
        t
    });
    let mut ticks = use_signal(|| 0u64);
    let mut renaming = use_signal(|| None::<(u32, String)>);
    let mut live = use_signal(String::new);
    if let Some(due) = now_ms.checked_div(activity_ms)
        && due > ticks()
    {
        ticks.set(due);
        let bg: Vec<u32> = tt
            .peek()
            .tabs
            .iter()
            .filter(|t| t.id != tt.peek().active && t.exit.is_none())
            .map(|t| t.id)
            .collect();
        if let Some(&id) = bg.get((due as usize) % bg.len().max(1)) {
            tt.write()
                .output(id, &format!("  GET /api/runs 200 in {} ms", 20 + due % 70));
        }
    }
    let t = tt.read().clone();
    let active = t.tabs.iter().find(|x| x.id == t.active).cloned();
    let panel_label = active.as_ref().map(|a| a.title.clone()).unwrap_or_default();
    rsx! {
        div { class: "oxyn-term", style: "height:{height}px",
            div { class: "tm-strip", role: "tablist", "aria-label": "Terminals",
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Tab && e.modifiers().ctrl() { e.prevent_default(); tt.write().cycle(if e.modifiers().shift() { -1 } else { 1 }) }
                    if e.key() == Key::ArrowRight { tt.write().cycle(1) }
                    if e.key() == Key::ArrowLeft { tt.write().cycle(-1) }
                },
                for tab in t.tabs.iter().cloned() {
                    {
                        let on = tab.id == t.active;
                        let id = tab.id;
                        let title = tab.title.clone();
                        rsx! {
                            div { key: "{tab.id}", class: cls(&["tm-tab", if on { "on" } else { "" }]),
                                if renaming().is_some_and(|(r, _)| r == tab.id) {
                                    input { class: "tm-ren", value: "{renaming().map(|r| r.1).unwrap_or_default()}", "aria-label": "Rename terminal", autofocus: true,
                                        oninput: move |e| renaming.set(Some((id, e.value()))),
                                        onkeydown: move |e: KeyboardEvent| { e.stop_propagation(); match e.key() {
                                            Key::Enter => { let n = renaming().map(|r| r.1).unwrap_or_default(); tt.write().rename(id, &n); renaming.set(None); live.set("Terminal renamed".into()) }
                                            Key::Escape => renaming.set(None),
                                            _ => {}
                                        } },
                                        onblur: move |_| { let n = renaming().map(|r| r.1).unwrap_or_default(); tt.write().rename(id, &n); renaming.set(None) },
                                    }
                                } else {
                                    button { class: "tm-tb", r#type: "button", role: "tab", "aria-selected": "{on}", tabindex: if on { "0" } else { "-1" }, "aria-controls": "oxyn-term-panel",
                                        "aria-label": format!("{}{}", tab.title, if tab.activity { ", new output" } else { "" }),
                                        onclick: move |_| tt.write().select(id),
                                        ondoubleclick: move |_| renaming.set(Some((id, title.clone()))),
                                        Icon { name: "prompt", size: 12.0 }
                                        span { "{tab.title}" }
                                        if tab.activity { span { class: "tm-act", "aria-hidden": "true" } }
                                        if let Some(code) = tab.exit { span { class: cls(&["tm-exit", if code == 0 { "ok" } else { "bad" }]), "{code}" } }
                                    }
                                    button { class: "tm-x", r#type: "button", "aria-label": "Close {tab.title}", tabindex: "-1", onclick: move |_| { tt.write().close(id); live.set("Terminal closed".into()) }, Icon { name: "x", size: 11.0 } }
                                }
                            }
                        }
                    }
                }
                button { class: "tm-add", r#type: "button", "aria-label": "New terminal", onclick: move |_| { tt.write().add(); live.set("New terminal".into()) }, Icon { name: "plus", size: 13.0 } }
            }
            div { class: "tm-out", id: "oxyn-term-panel", role: "tabpanel", "aria-label": "{panel_label}",
                if let Some(a) = active {
                    for (i, l) in a.lines.iter().enumerate() { div { key: "{i}", class: if l.starts_with('$') { "tm-cmd" } else { "" }, "{l}" } }
                    div { class: "tm-cmd", "$ " span { class: "tm-cur", "aria-hidden": "true" } }
                } else {
                    div { class: "sh-empty", "No terminals. Press + to open one." }
                }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}

/// Sidebar Nav (Beautiful UI, MIT, © 2026 Shane Levine): grouped links with a sliding active
/// indicator, hover highlight, collapsible groups and badges.
#[component]
pub fn SidebarNav(
    /// Sections.
    #[props(default = sample_sections())]
    sections: Vec<NavSection>,
    /// Active id.
    #[props(default = "home".to_string())]
    active: String,
    /// Compact density.
    #[props(default)]
    compact: bool,
    /// Changed.
    #[props(default)]
    on_select: EventHandler<String>,
) -> Element {
    let mut act = use_signal(|| active.clone());
    let mut closed = use_signal(Vec::<String>::new);
    let mut live = use_signal(String::new);
    let rows: Vec<(String, NavItem)> = sections
        .iter()
        .flat_map(|s| s.items.iter().map(move |i| (s.title.clone(), i.clone())))
        .collect();
    let row_h = if compact { 30.0 } else { 36.0 };
    let mut y = 0.0f32;
    let mut ind = None;
    let mut layout: Vec<(String, Vec<(NavItem, f32)>)> = Vec::new();
    for s in &sections {
        if !s.title.is_empty() {
            y += 28.0;
        }
        let hidden = closed.read().contains(&s.title);
        let mut items = Vec::new();
        if !hidden {
            for it in &s.items {
                if it.id == act() {
                    ind = Some(y);
                }
                items.push((it.clone(), y));
                y += row_h;
            }
        }
        layout.push((s.title.clone(), items));
        y += 8.0;
    }
    let _ = rows;
    rsx! {
        nav { class: cls(&["oxyn-snav", if compact { "compact" } else { "" }]), "aria-label": "Sidebar", style: "--rh:{row_h}px",
            if let Some(top) = ind { span { class: "sn-ind", "aria-hidden": "true", style: "transform:translateY({top}px)" } }
            for (title, items) in layout.into_iter() {
                div { key: "{title}", class: "sn-grp",
                    if !title.is_empty() {
                        button { class: "sn-gh", r#type: "button", "aria-expanded": "{!closed.read().contains(&title)}",
                            onclick: { let t = title.clone(); move |_| { let mut c = closed.write(); if let Some(i) = c.iter().position(|x| *x == t) { c.remove(i); } else { c.push(t.clone()); } } },
                            "{title}" span { class: cls(&["sh-chev", if closed.read().contains(&title) { "" } else { "open" }]), Icon { name: "chevron-right", size: 11.0 } }
                        }
                    }
                    ul { role: "list",
                        for (it, _) in items {
                            li { key: "{it.id}",
                                a { class: cls(&["sn-a", if act() == it.id { "on" } else { "" }]), href: "#{it.id}", "aria-current": if act() == it.id { "page" },
                                    onclick: { let id = it.id.clone(); let l = it.label.clone(); move |e: MouseEvent| { e.prevent_default(); act.set(id.clone()); live.set(l.clone()); on_select.call(id.clone()) } },
                                    span { class: "sh-ic", Icon { name: if it.icon.is_empty() { "page".to_string() } else { it.icon.clone() }, size: 15.0 } }
                                    span { class: "sh-lb", "{it.label}" }
                                    if it.count > 0 { span { class: "sh-ct", "{it.count}" } }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "sr", role: "status", "{live}" }
        }
    }
}

/// The frame of an AI Chat screen (affectUI): an optional app rail, a thread list, the main
/// column and an optional side panel.
#[component]
pub fn ChatShell(
    /// Show the narrow app rail (Double sidebar).
    #[props(default)]
    rail: bool,
    /// Thread list rows (title, meta); empty hides the list.
    #[props(default)]
    threads: Vec<(String, String)>,
    /// Active thread index.
    #[props(default)]
    active: usize,
    /// Header title.
    #[props(default)]
    title: String,
    /// Side panel.
    #[props(default)]
    side: Option<Element>,
    /// Height (px).
    #[props(default = 520.0)]
    height: f32,
    /// Main column.
    children: Element,
) -> Element {
    let mut cur = use_signal(|| active);
    rsx! {
        div { class: "oxyn-chatshell", style: "height:{height}px",
            if rail {
                nav { class: "cs-rail", "aria-label": "Apps",
                    span { class: "sh-logo", "aria-hidden": "true", "A" }
                    for (ic, l) in [("thread", "Chats"), ("agents", "Agents"), ("folder", "Projects"), ("cog", "Settings")] {
                        button { key: "{l}", class: cls(&["sh-ib", if l == "Chats" { "on" } else { "" }]), r#type: "button", "aria-label": "{l}", title: "{l}", Icon { name: ic, size: 15.0 } }
                    }
                }
            }
            if !threads.is_empty() {
                nav { class: "cs-list", "aria-label": "Threads",
                    button { class: "cs-new", r#type: "button", Icon { name: "plus", size: 13.0 } "New chat" }
                    ul { role: "list",
                        for (i, (t, m)) in threads.iter().cloned().enumerate() {
                            li { key: "{i}", button { class: cls(&["cs-th", if cur() == i { "on" } else { "" }]), r#type: "button", "aria-current": if cur() == i { "page" }, onclick: move |_| cur.set(i), b { "{t}" } small { "{m}" } } }
                        }
                    }
                }
            }
            main { class: "cs-main",
                if !title.is_empty() { header { class: "cs-h", h2 { "{title}" } span { class: "sh-sp" } button { class: "sh-ib", r#type: "button", "aria-label": "Share", Icon { name: "share", size: 14.0 } } } }
                div { class: "cs-body", {children} }
            }
            if let Some(s) = side { aside { class: "cs-side", "aria-label": "Details", {s} } }
        }
    }
}

/// Harness: a demo frame with a toolbar (title, viewport width, theme) and a **Fork this** button
/// that hands the host a fork of the current demo.
#[component]
pub fn Harness(
    /// Title.
    #[props(default = "App shell".to_string())]
    title: String,
    /// Fork link (shown after forking).
    #[props(default = "/fork/app-shell".to_string())]
    fork_href: String,
    /// Starting viewport: `desktop` · `tablet` · `phone`.
    #[props(default = "desktop".to_string())]
    viewport: String,
    /// Dark canvas.
    #[props(default)]
    dark: bool,
    /// Fork pressed.
    #[props(default)]
    on_fork: EventHandler<String>,
    /// The demo.
    children: Element,
) -> Element {
    let mut vp = use_signal(|| viewport.clone());
    let mut dk = use_signal(|| dark);
    let mut forked = use_signal(|| false);
    let w = match vp().as_str() {
        "phone" => "390px",
        "tablet" => "640px",
        _ => "100%",
    };
    rsx! {
        div { class: cls(&["oxyn-harness", if dk() { "dark" } else { "" }]),
            header { class: "hn-bar",
                b { "{title}" }
                span { class: "sh-sp" }
                div { class: "hn-seg", role: "radiogroup", "aria-label": "Viewport",
                    for (id, ic) in [("desktop", "monitor"), ("tablet", "page"), ("phone", "phone")] {
                        button { key: "{id}", class: cls(&["sh-ib", if vp() == id { "on" } else { "" }]), r#type: "button", role: "radio", "aria-checked": "{vp() == id}", "aria-label": "{id}", onclick: move |_| vp.set(id.to_string()), Icon { name: ic, size: 13.0 } }
                    }
                }
                button { class: cls(&["sh-ib", if dk() { "on" } else { "" }]), r#type: "button", "aria-label": "Dark canvas", "aria-pressed": "{dk()}", onclick: move |_| { let n = !dk(); dk.set(n) }, Icon { name: if dk() { "sun" } else { "moon" }, size: 13.0 } }
                if forked() {
                    a { class: "hn-fork done", href: "{fork_href}", Icon { name: "check", size: 13.0 } "Forked · open" }
                } else {
                    button { class: "hn-fork", r#type: "button", onclick: move |_| { forked.set(true); on_fork.call(fork_href.clone()) }, Icon { name: "branch", size: 13.0 } "Fork this" }
                }
            }
            div { class: "hn-stage", div { class: "hn-vp", style: "width:{w}", {children} } }
            div { class: "sr", role: "status", if forked() { "Forked. A copy of this demo is ready to edit." } }
        }
    }
}
