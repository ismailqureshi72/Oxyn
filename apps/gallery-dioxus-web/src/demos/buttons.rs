//! Buttons: button, icon button, action button, split button and the three groups.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::button::{ActionButton as ActionModel, GroupItem},
};
use oxyn_dioxus::{Button, ButtonGroup};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{ActionButton, ExpandingButtonGroup, FloatingButtonGroup, IconButton, SplitButton};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

const ICONS: [&str; 7] = ["none", "spark", "play", "up", "trash", "check", "share"];

fn group_items(n: usize) -> Vec<GroupItem> {
    let mut v = vec![
        GroupItem::new("bold", "Bold", Some("text")),
        GroupItem::new("link", "Link", Some("link")),
        GroupItem::new("code", "Code", Some("code")),
        GroupItem::new("files", "Files", Some("folder")).count(12),
        GroupItem::new("img", "Image", Some("image")),
        GroupItem::new("table", "Table", Some("table")),
    ];
    v.truncate(n);
    v
}

fn button(v: &Vals, _: ClockMs) -> Element {
    let progress = (v.s("loading_style") == "progress").then_some(0.64);
    let reason = v
        .b("disabled")
        .then(|| "Waiting for your approval".to_string());
    rsx! {
        Button {
            label: v.string("label"),
            variant: v.string("variant"),
            size: v.string("size"),
            icon: v.opt("icon"),
            loading: v.b("loading"),
            loading_style: v.string("loading_style"),
            progress,
            disabled: v.b("disabled"),
            reason,
            hold: v.b("hold"),
            shortcut: v.opt("shortcut"),
        }
    }
}

fn button_code(v: &Vals) -> String {
    let mut c = Code::new("Button")
        .raw("label", q(v.s("label")))
        .s("variant", v.s("variant"), "primary")
        .s("size", v.s("size"), "md")
        .opt("icon", v.s("icon"))
        .b("loading", v.b("loading"));
    if v.b("loading") {
        c = c.s("loading_style", v.s("loading_style"), "shimmer");
        if v.s("loading_style") == "progress" {
            c = c.raw("progress", "0.64");
        }
    }
    c.b("disabled", v.b("disabled"))
        .b("hold", v.b("hold"))
        .opt("shortcut", v.s("shortcut"))
        .raw("onclick", "move |_| deploy()")
        .done()
}

fn group(v: &Vals, _: ClockMs) -> Element {
    let n = v.u("count");
    let overflow = v.b("overflow");
    let items = group_items(if overflow { 6 } else { n });
    let selected = v.b("toggle").then(|| "code".to_string());
    rsx! {
        div { class: "oxd-room-menu",
            ButtonGroup {
                items,
                variant: v.string("variant"),
                size: v.string("size"),
                vertical: v.b("vertical"),
                collapse_labels: v.b("icons_only"),
                selected,
                max_visible: overflow.then_some(4),
            }
        }
    }
}

fn group_code(v: &Vals) -> String {
    Code::new("ButtonGroup")
        .pre("let items = vec![")
        .pre("    GroupItem::new(\"bold\", \"Bold\", Some(\"text\")),")
        .pre("    GroupItem::new(\"link\", \"Link\", Some(\"link\")),")
        .pre("    GroupItem::new(\"code\", \"Code\", Some(\"code\")),")
        .pre("];")
        .raw("items", "items")
        .s("variant", v.s("variant"), "outline")
        .s("size", v.s("size"), "md")
        .b("vertical", v.b("vertical"))
        .b("collapse_labels", v.b("icons_only"))
        .opt("selected", if v.b("toggle") { "code" } else { "" })
        .raw(
            "max_visible",
            if v.b("overflow") { "Some(4)" } else { "None" },
        )
        .raw("on_action", "move |id: String| format(id)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
            slug: "button",
            name: "Button",
            category: "buttons",
            summary: "Eight variants and five sizes, with a shimmering loading state, a progress fill, press-and-hold for destructive actions and a shortcut chip.",
            dioxus: "Button",
            gpui: "ButtonEl",
            spec: "button",
            source: "primitives/buttons.rs",
            controls: vec![
                txt("label", "Label", "Deploy"),
                sel(
                    "variant",
                    "Variant",
                    &[
                        "primary",
                        "secondary",
                        "ghost",
                        "destructive",
                        "outline",
                        "link",
                        "quiet",
                        "success",
                    ],
                    "primary",
                ),
                sel("size", "Size", &["xs", "sm", "md", "lg", "xl"], "md"),
                sel("icon", "Icon", &ICONS, "none"),
                tog("loading", "Loading", false),
                sel(
                    "loading_style",
                    "Loading style",
                    &["shimmer", "spinner", "progress"],
                    "shimmer",
                ),
                tog("disabled", "Disabled with reason", false),
                tog("hold", "Hold to confirm", false),
                sel("shortcut", "Shortcut", &["none", "Mod+Enter"], "none"),
            ],
            variants: vec![
                var("primary", "The one main action on a surface.", &[]),
                var(
                    "secondary",
                    "Second action next to a primary.",
                    &[("variant", "secondary"), ("label", "Preview")],
                ),
                var(
                    "destructive-hold",
                    "Press and hold to confirm a delete.",
                    &[
                        ("variant", "destructive"),
                        ("label", "Hold to delete"),
                        ("icon", "trash"),
                        ("hold", "true"),
                    ],
                ),
                var(
                    "loading-progress",
                    "Keeps its width while it works and fills with progress.",
                    &[
                        ("variant", "secondary"),
                        ("label", "Uploading"),
                        ("loading", "true"),
                        ("loading_style", "progress"),
                    ],
                ),
                var(
                    "shortcut",
                    "Shortcut chip, spelled out for screen readers.",
                    &[
                        ("label", "Run"),
                        ("icon", "play"),
                        ("shortcut", "Mod+Enter"),
                    ],
                ),
            ],
            render: button,
            code: button_code,
            keyboard: vec![
                ("Enter / Space", "Press"),
                (
                    "Space (held)",
                    "Hold-to-confirm buttons fill while held and fire when full",
                ),
                (
                    "Tab",
                    "Disabled-with-reason buttons stay focusable so the reason can be read",
                ),
            ],
            a11y: vec![
                "Native button element with type=button.",
                "Loading sets aria-busy and aria-disabled; focus stays on the button.",
                "A disabled button with a reason keeps focus and exposes the reason through aria-describedby.",
                "Shortcuts are exposed with aria-keyshortcuts; the visible chip is hidden from screen readers.",
            ],
            responsive: vec![
                "Keeps its width while loading, so rows never jump.",
                "full stretches it to the container (mobile sheets, narrow side panels).",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "button-group",
            name: "Button group",
            category: "buttons",
            summary: "Joined buttons for a toolbar: outline or solid, horizontal or vertical, icon-only with tooltips, a toggle selection and an overflow menu.",
            dioxus: "ButtonGroup",
            gpui: "ButtonGroupEl",
            spec: "button-groups",
            source: "primitives/buttons.rs",
            controls: vec![
                sel("variant", "Variant", &["outline", "solid"], "outline"),
                sel("size", "Size", &["ide", "sm", "md"], "md"),
                rng("count", "Buttons", 2.0, 6.0, 1.0, "3"),
                tog("vertical", "Vertical", false),
                tog("icons_only", "Icons only", false),
                tog("toggle", "Toggle selection", false),
                tog("overflow", "Overflow menu", false),
            ],
            variants: vec![
                var("outline", "Default toolbar group.", &[]),
                var(
                    "solid-toggle",
                    "Solid, icon-only, one selected.",
                    &[
                        ("variant", "solid"),
                        ("icons_only", "true"),
                        ("toggle", "true"),
                    ],
                ),
                var(
                    "overflow",
                    "Six actions, four visible, the rest in a menu.",
                    &[("overflow", "true"), ("size", "sm")],
                ),
                var(
                    "vertical",
                    "Stacked for a side rail.",
                    &[("vertical", "true"), ("size", "sm")],
                ),
            ],
            render: group,
            code: group_code,
            keyboard: vec![
                ("Tab", "Each button is a tab stop"),
                ("↓ / ↑ on overflow", "Open the overflow menu"),
            ],
            a11y: vec![
                "role=group with a label; toggle groups expose aria-pressed per button.",
                "Icon-only buttons keep their labels as aria-label and tooltips.",
            ],
            responsive: vec![
                "max_visible moves the rest into an overflow menu instead of wrapping.",
                "ide size (24 px) is for dense editor toolbars.",
            ],
            height: 260,
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
const ORDER: [&str; 7] = ["button", "icon-button", "action-button", "split-button", "button-group", "floating-button-group", "expanding-button-group"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
