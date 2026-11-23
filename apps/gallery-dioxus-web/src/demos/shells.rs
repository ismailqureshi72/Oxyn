//! App shells and screens (plan #27, chunk 4): sidebars, the rail, the inbox, app shells, the AI
//! side panel, terminal tabs, Beautiful UI's Sidebar Nav, the harness and the seven affectUI AI
//! Chat screens. Every shell renders inside the demo app frame.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{ClockMs, chat::thread::Starter, shell::Screen};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    AiSidePanel, AppShell, ChatShell, Harness, InboxSidebar, PlanCard, Sidebar, SidebarNav,
    SidebarRail, StarterPrompts, StatusIndicator, TerminalTabs, TodoList, ToolStack,
};
use oxyn_dioxus::{
    ApprovalCard, MessageBubble, StreamingMarkdown, SubagentTree, ToolCallRow, WorkspaceSidebar,
};

use super::{Code, mk, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

fn key(v: &Vals, keys: &[&str]) -> String {
    keys.iter().map(|k| v.s(k)).collect::<Vec<_>>().join("|")
}

fn s(x: &str) -> String {
    x.to_string()
}

/// The demo app frame (window dots and a title) around a shell.
fn app_frame(title: &str, body: Element) -> Element {
    rsx! {
        div { class: "oxd-app oxs-frame",
            div { class: "oxd-app-bar",
                span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" }
                span { class: "oxd-app-title", "{title}" }
                span { class: "oxd-app-sp" }
            }
            div { class: "oxs-body", {body} }
        }
    }
}

fn page_body() -> Element {
    rsx! {
        div { class: "oxs-page",
            h3 { "Checkout redesign" }
            p { "Three flows to review before Thursday. The address step still drops 14% of mobile sessions." }
            div { class: "oxs-cards",
                div { class: "oxs-card", b { "Address" } small { "In review · Marcus" } }
                div { class: "oxs-card", b { "Payment" } small { "Ready · Emma" } }
                div { class: "oxs-card", b { "Receipt" } small { "Draft · Ava" } }
            }
        }
    }
}

// ───────────────────────────── Workspace sidebar ──

fn workspace(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["menu", "query", "collapsed", "storage"]);
    rsx! {
        {app_frame("Northwind", rsx! {
            div { key: "{k}", class: "oxs-row",
                WorkspaceSidebar { menu_open: v.b("menu"), query: if v.s("query") == "none" { String::new() } else { v.string("query") }, collapsed: v.b("collapsed"), storage: v.f("storage") as f32 / 100.0, height: 500.0 }
                {page_body()}
            }
        })}
    }
}

fn workspace_code(v: &Vals) -> String {
    Code::new("WorkspaceSidebar")
        .raw("workspaces", "workspaces()")
        .raw("favorites", "favorites")
        .raw("projects", "projects")
        .raw("storage", format!("{:.2}", v.f("storage") / 100.0))
        .raw("on_navigate", "move |name: String| open(name)")
        .done()
}

pub fn demos() -> Vec<Demo> {
    let shell_resp = vec![
        "The frame fills the stage width; the sidebar keeps its width and the content column takes the rest.",
        "Below 640 px fold the sidebar to the rail with `[`.",
    ];
    #[allow(unused_mut)]
    let mut v = vec![Demo {
        controls: vec![
            tog("menu", "Switcher open", false),
            sel("query", "Search", &["none", "check", "zzz"], "none"),
            tog("collapsed", "Icon rail", false),
            rng("storage", "Storage %", 0.0, 100.0, 1.0, "38"),
        ],
        variants: vec![
            var("default", "Favourites, projects, storage and account.", &[]),
            var(
                "switcher",
                "The workspace switcher open.",
                &[("menu", "true")],
            ),
            var(
                "search",
                "Live search filters both lists.",
                &[("query", "check")],
            ),
            var("no-match", "Nothing matches.", &[("query", "zzz")]),
            var("rail", "Folded to the rail.", &[("collapsed", "true")]),
        ],
        render: workspace,
        code: workspace_code,
        keyboard: vec![
            ("[", "Fold or expand"),
            ("Alt + ↑ / ↓", "Reorder a favourite"),
            ("Enter / Escape", "Create or cancel a new project"),
            ("Escape", "Close the switcher or clear search"),
        ],
        a11y: vec![
            "The switcher is a menu button with radio items.",
            "Reorders and new projects are announced.",
            "Storage is a meter.",
        ],
        responsive: shell_resp.clone(),
        height: 580,
        align: Align::Start,
        ..mk(
            "workspace-sidebar",
            "Workspace sidebar",
            "shells",
            "A workspace sidebar: switcher, live search, reorderable favourites, inline new projects, a storage meter and the account.",
            "WorkspaceSidebar",
            "WorkspaceSidebarEl",
            "workspace-chrome",
            "shell/mod.rs",
        )
    }];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 10] = [
    "sidebar",
    "workspace-sidebar",
    "sidebar-rail",
    "inbox-sidebar",
    "app-shell",
    "ai-side-panel",
    "terminal-tabs",
    "sidebar-nav",
    "harness",
    "ai-chat-screens",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
