//! Overlays: tooltip, popover, dropdown menu.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{Button, DropdownMenu, IconButton, Popover, TextField, Tooltip};

use super::{Code, q, sel, tog, txt, var};
use crate::{Align, Demo, Vals, data};

fn tooltip(v: &Vals, _: ClockMs) -> Element {
    let open = v.b("open").then_some(true);
    let rich = v.b("rich");
    rsx! {
        div { class: "oxd-pad-tip",
            Tooltip {
                content: v.string("content"),
                title: rich.then(|| "Run tests".to_string()),
                shortcut: v.opt("shortcut"),
                side: v.string("side"),
                align: v.string("tip_align"),
                offset: v.f("offset") as f32,
                disabled: v.b("disabled"),
                delay: v.f("delay") as u32,
                open,
                IconButton { icon: "search", label: "Search", variant: "secondary" }
            }
        }
    }
}

fn tooltip_code(v: &Vals) -> String {
    let mut c = Code::new("Tooltip")
        .raw("content", q(v.s("content")))
        .opt("title", if v.b("rich") { "Run tests" } else { "" })
        .opt("shortcut", v.s("shortcut"))
        .s("side", v.s("side"), "top");
    if v.s("delay") != "500" {
        c = c.raw("delay", v.s("delay"));
    }
    let s = c
        .raw("IconButton", "{ icon: \"search\", label: \"Search\" }")
        .done();
    // The child is not a prop; print it as one.
    s.replace(
        "IconButton: { icon: \"search\", label: \"Search\" },",
        "IconButton { icon: \"search\", label: \"Search\" }",
    )
}

fn popover(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-room-pop",
            Popover {
                trigger: rsx! { Button { label: "Share", variant: "secondary", size: "sm", icon: "share" } },
                trigger_label: "Share",
                open: v.b("open"),
                side: v.string("side"),
                align: v.string("align"),
                title: Some("Share this thread".to_string()),
                description: Some("Anyone with the link can read it.".to_string()),
                width: Some(280.0),
                arrow: v.b("arrow"),
                mode: v.string("mode"),
                modal: v.b("modal"),
                collision_padding: v.f("padding") as f32,
                TextField { label: "Link", value: "oxyn.dev/t/8f2a", copyable: true, size: "sm" }
            }
        }
    }
}

fn popover_code(v: &Vals) -> String {
    let mut s = String::from("rsx! {\n    Popover {\n");
    s.push_str("        trigger: rsx! { Button { label: \"Share\", variant: \"secondary\", icon: \"share\" } },\n");
    s.push_str("        trigger_label: \"Share\",\n");
    if v.s("side") != "bottom" {
        s.push_str(&format!("        side: {},\n", q(v.s("side"))));
    }
    if v.s("align") != "start" {
        s.push_str(&format!("        align: {},\n", q(v.s("align"))));
    }
    s.push_str("        title: \"Share this thread\",\n");
    s.push_str("        description: \"Anyone with the link can read it.\",\n");
    if v.b("arrow") {
        s.push_str("        arrow: true,\n");
    }
    if v.s("mode") != "press" {
        s.push_str(&format!("        mode: {},\n", q(v.s("mode"))));
    }
    s.push_str("        TextField { label: \"Link\", value: link(), copyable: true }\n    }\n}");
    s
}

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
    vec![
        Demo {
            slug: "tooltip",
            name: "Tooltip",
            category: "overlays",
            summary: "A small label for one control: shows after a short rest and at once on keyboard focus, neighbours follow instantly, a shortcut inside, a rich form with a title.",
            dioxus: "Tooltip, TooltipProvider",
            gpui: "TooltipEl",
            spec: "tooltip-popover",
            source: "primitives/overlays.rs",
            controls: vec![
                txt("content", "Text", "Search runs"),
                sel("shortcut", "Shortcut", &["none", "Mod+K", "/"], "Mod+K"),
                sel("side", "Side", &["top", "bottom", "left", "right"], "top"),
                super::rng("delay", "Delay ms", 0.0, 1000.0, 100.0, "500"),
                tog("rich", "Rich (title)", false),
                tog("open", "Force open", true),
                sel("tip_align", "Align", &["center", "start", "end"], "center"),
                super::rng("offset", "Offset px", 0.0, 16.0, 1.0, "6"),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("shortcut", "Label with its shortcut.", &[]),
                var(
                    "bottom",
                    "Below the control.",
                    &[("side", "bottom"), ("shortcut", "none")],
                ),
                var(
                    "rich",
                    "A title line and text.",
                    &[
                        ("rich", "true"),
                        ("content", "Runs the full suite on a fresh worktree."),
                        ("shortcut", "none"),
                    ],
                ),
                var(
                    "hover",
                    "Real hover: rest the pointer on it.",
                    &[("open", "false")],
                ),
            ],
            render: tooltip,
            code: tooltip_code,
            keyboard: vec![("Tab", "Focus shows it at once"), ("Esc", "Hide it")],
            a11y: vec![
                "The text becomes the trigger's description (aria-describedby).",
                "Wrap the app in TooltipProvider so neighbouring tooltips open without a second delay.",
            ],
            responsive: vec!["side picks where it opens; it never takes focus."],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "popover",
            name: "Popover",
            category: "overlays",
            summary: "A small floating card attached to its trigger: a title and quiet line, any content, an arrow, opened by press or hover intent; Escape and a press outside close it.",
            dioxus: "Popover",
            gpui: "PopoverEl",
            spec: "tooltip-popover",
            source: "primitives/overlays.rs",
            controls: vec![
                tog("open", "Open", true),
                sel(
                    "side",
                    "Side",
                    &["bottom", "top", "right", "left"],
                    "bottom",
                ),
                sel("align", "Align", &["start", "center", "end"], "center"),
                tog("arrow", "Arrow", true),
                sel("mode", "Opens on", &["press", "hover"], "press"),
                tog("modal", "Modal", false),
                super::rng("padding", "Collision padding", 0.0, 24.0, 2.0, "10"),
            ],
            variants: vec![
                var("press", "Opens on press.", &[]),
                var(
                    "hover",
                    "Hover intent.",
                    &[("mode", "hover"), ("open", "false")],
                ),
                var("no-arrow", "Without the arrow.", &[("arrow", "false")]),
            ],
            render: popover,
            code: popover_code,
            keyboard: vec![
                ("Enter / Space", "Open; focus moves into the card"),
                ("Esc", "Close; focus goes back to the trigger"),
            ],
            a11y: vec![
                "role=dialog named by its title; the trigger has aria-expanded and aria-controls.",
                "focus_on_open moves focus into the card; closing hands it back.",
            ],
            responsive: vec![
                "width sets the card width; side and align place it around the trigger.",
            ],
            height: 300,
            align: Align::Start,
            live: false,
        },
        Demo {
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
    ]
}
