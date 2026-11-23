//! Overlays: tooltip, popover, dropdown menu.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{Button, DropdownMenu, TextField};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{IconButton, Popover, Tooltip};

use super::{Code, q, sel, tog, txt, var};
use crate::{Align, Demo, Vals, data};

fn menu(v: &Vals, now: ClockMs) -> Element {
    let items = if v.b("empty") {
        vec![]
    } else if v.b("logos") {
        data::menu_items_logos()
    } else {
        data::menu_items()
    };
    let ctx = v.b("context");
    rsx! {
        div { class: "oxd-room-menu-lg",
            if ctx {
                DropdownMenu { items, context: true, open: v.b("open"), now,
                    div { class: "oxd-ctxbox", "Right-click this message (or Shift+F10)" }
                }
            } else {
                DropdownMenu {
                    items,
                    trigger: v.string("trigger"),
                    icon: Some("thread".to_string()),
                    title: v.b("title").then(|| "Thread actions".to_string()),
                    align: v.string("align"),
                    searchable: v.b("searchable"),
                    density: v.string("density"),
                    open: v.b("open"),
                    now,
                }
            }
        }
    }
}

fn menu_code(v: &Vals) -> String {
    let c = Code::new("DropdownMenu")
        .pre("let items = vec![")
        .pre("    MenuItem::action(\"rename\", \"Rename\").icon(\"pencil\").shortcut(\"Mod+R\"),")
        .pre("    MenuItem::submenu(\"move\", \"Move to\", vec![MenuItem::action(\"archive\", \"Archive\")]),")
        .pre("    MenuItem::divider(),")
        .pre("    MenuItem::checkbox(\"wrap\", \"Wrap long lines\", true),")
        .pre("    MenuItem::radio(\"fast\", \"Fast model\", \"model\", true),")
        .pre("    MenuItem::divider(),")
        .pre("    MenuItem::action(\"del\", \"Delete thread\").icon(\"trash\").danger(),")
        .pre("];")
        .raw("items", "items");
    let c = if v.b("context") {
        c.raw("context", "true")
            .raw("div", "{ \"Right-click this message\" }")
    } else {
        c.raw("trigger", q(v.s("trigger")))
            .raw("icon", q("thread"))
            .opt("title", if v.b("title") { "Thread actions" } else { "" })
            .s("align", v.s("align"), "start")
            .b("searchable", v.b("searchable"))
            .s("density", v.s("density"), "comfortable")
    };
    c.raw("on_action", "move |id: String| run(id)")
        .done()
        .replace(
            "div: { \"Right-click this message\" },",
            "div { \"Right-click this message\" }",
        )
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
            slug: "dropdown-menu",
            name: "Dropdown menu",
            category: "overlays",
            summary: "Actions and choices behind any trigger: icons, shortcuts, checks and radios, red items and submenus, a gliding highlight, type-ahead, search and a context-menu mode.",
            dioxus: "DropdownMenu",
            gpui: "MenuEl",
            spec: "dropdown-menu",
            source: "primitives/overlays.rs",
            controls: vec![
                txt("trigger", "Trigger", "Thread"),
                tog("open", "Open", true),
                tog("title", "Heading", true),
                sel("align", "Align", &["start", "end"], "start"),
                sel(
                    "density",
                    "Density",
                    &["comfortable", "compact"],
                    "comfortable",
                ),
                tog("searchable", "Searchable", false),
                tog("logos", "Tool logos", false),
                tog("context", "Context-menu mode", false),
                tog("empty", "Empty", false),
            ],
            variants: vec![
                var("menu", "Every row type.", &[]),
                var(
                    "searchable-compact",
                    "A filter field, compact rows.",
                    &[
                        ("searchable", "true"),
                        ("density", "compact"),
                        ("title", "false"),
                    ],
                ),
                var(
                    "context",
                    "Opens on right-click.",
                    &[("context", "true"), ("open", "false")],
                ),
                var(
                    "empty",
                    "Nothing to show.",
                    &[("empty", "true"), ("title", "false")],
                ),
            ],
            render: menu,
            code: menu_code,
            keyboard: vec![
                (
                    "Enter / Space / ↓",
                    "Open on the first item (↑ opens on the last)",
                ),
                ("↑ / ↓, Home / End", "Move"),
                ("→ / ←", "Open / close a submenu"),
                ("Type", "Jump to the next item starting with that letter"),
                ("Enter / Space", "Run, check or pick"),
                ("Esc / Tab", "Close"),
                ("Shift+F10 / ContextMenu", "Open in context-menu mode"),
            ],
            a11y: vec![
                "role=menu with menuitem, menuitemcheckbox and menuitemradio rows.",
                "Disabled items stay readable and are skipped by the keyboard.",
                "Shortcuts are spelled out for screen readers.",
            ],
            responsive: vec![
                "align end keeps it inside the right edge; compact density for dense apps.",
            ],
            height: 460,
            align: Align::Start,
            live: false,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 3] = ["tooltip", "popover", "dropdown-menu"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
