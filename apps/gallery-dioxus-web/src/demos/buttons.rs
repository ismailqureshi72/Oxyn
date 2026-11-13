//! Buttons: button, icon button, action button, split button and the three groups.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::button::{ActionButton as ActionModel, GroupItem},
};
use oxyn_dioxus::{
    ActionButton, Button, ButtonGroup, ExpandingButtonGroup, FloatingButtonGroup, IconButton,
    SplitButton,
};

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

fn tray() -> Vec<GroupItem> {
    vec![
        GroupItem::new("copy", "Copy", Some("copy")),
        GroupItem::new("retry", "Regenerate", Some("retry")),
        GroupItem::new("good", "Good response", Some("check")),
        GroupItem::new("share", "Share", Some("share")),
    ]
}

fn expand() -> Vec<GroupItem> {
    vec![
        GroupItem::new("copy", "Copy", Some("copy")).done("Copied"),
        GroupItem::new("branch", "Branch", Some("branch")).done("Branched"),
        GroupItem::new("share", "Share", Some("share")).done("Link copied"),
        GroupItem::new("dl", "Export", Some("download")).done("Saved"),
    ]
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

fn icon_button(v: &Vals, _: ClockMs) -> Element {
    let pressed = v.b("toggle").then_some(false);
    let badge = match v.s("badge") {
        "dot" => Some(0),
        "count" => Some(3),
        _ => None,
    };
    let (icon, pi) = if v.b("toggle") {
        ("audio", Some("stop".to_string()))
    } else {
        (v.s("icon"), None)
    };
    rsx! {
        IconButton {
            icon: icon.to_string(),
            label: v.string("label"),
            variant: v.string("variant"),
            size: v.string("size"),
            pressed,
            pressed_icon: pi,
            badge,
            disabled: v.b("disabled"),
        }
    }
}

fn icon_button_code(v: &Vals) -> String {
    let mut c = Code::new("IconButton");
    if v.b("toggle") {
        c = c
            .raw("icon", q("audio"))
            .raw("pressed", "Some(muted())")
            .raw("pressed_icon", q("stop"));
    } else {
        c = c.raw("icon", q(v.s("icon")));
    }
    c = c
        .raw("label", q(v.s("label")))
        .s("variant", v.s("variant"), "ghost")
        .s("size", v.s("size"), "md");
    c = match v.s("badge") {
        "dot" => c.raw("badge", "Some(0)"),
        "count" => c.raw("badge", "Some(3)"),
        _ => c,
    };
    c.b("disabled", v.b("disabled"))
        .raw("onclick", "move |_| copy()")
        .done()
}

fn action(v: &Vals, now: ClockMs) -> Element {
    let mut m = ActionModel::new("Deploy", "Deploying…", "Deployed");
    match v.s("phase") {
        "pending" => {
            m.press(ClockMs(now.0 - 200));
        }
        "success" => {
            m.press(ClockMs(now.0 - 900));
            m.resolve(true, ClockMs(now.0 - 300));
        }
        "retry" => {
            m.backoff = std::time::Duration::from_secs(4);
            m.press(ClockMs(now.0 - 2_000));
            m.resolve(false, ClockMs(now.0 - 1_500));
        }
        _ => {}
    }
    let shortcut = v.b("shortcut").then(|| "Mod+Shift+D".to_string());
    rsx! { ActionButton { model: m, now, shortcut, icon: v.string("icon") } }
}

fn action_code(v: &Vals) -> String {
    Code::new("ActionButton")
        .pre("// The model walks idle → pending → success or error (with retry back-off).")
        .pre("let mut deploy = use_signal(|| ActionModel::new(\"Deploy\", \"Deploying…\", \"Deployed\"));")
        .raw("model", "deploy()")
        .raw("now", "clock.now()")
        .s("icon", v.s("icon"), "up")
        .opt(
            "shortcut",
            if v.b("shortcut") { "Mod+Shift+D" } else { "" },
        )
        .raw("on_press", "move |_| deploy.write().press(clock.now())")
        .done()
}

fn split(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-room-menu",
            SplitButton {
                actions: vec![("merge".into(), "Merge".into()), ("squash".into(), "Squash and merge".into()), ("rebase".into(), "Rebase and merge".into())],
                variant: v.string("variant"),
                icon: Some("merge".to_string()),
                remember: v.b("remember"),
                disabled: v.b("disabled"),
                open: v.b("open"),
            }
        }
    }
}

fn split_code(v: &Vals) -> String {
    Code::new("SplitButton")
        .raw(
            "actions",
            "vec![(\"merge\".into(), \"Merge\".into()), (\"squash\".into(), \"Squash and merge\".into()), (\"rebase\".into(), \"Rebase and merge\".into())]",
        )
        .s("variant", v.s("variant"), "primary")
        .raw("icon", q("merge"))
        .b("remember", v.b("remember"))
        .b("disabled", v.b("disabled"))
        .raw("on_action", "move |id: String| merge(id)")
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

fn floating(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-pad-tip",
            FloatingButtonGroup {
                items: tray(),
                variant: v.string("variant"),
                size: v.string("size"),
                vertical: v.b("vertical"),
                icon_only: !v.b("labels"),
                highlight_style: v.string("highlight_style"),
                auto_hide: v.b("auto_hide"),
            }
        }
    }
}

fn floating_code(v: &Vals) -> String {
    Code::new("FloatingButtonGroup")
        .pre("let items = vec![")
        .pre("    GroupItem::new(\"copy\", \"Copy\", Some(\"copy\")),")
        .pre("    GroupItem::new(\"retry\", \"Regenerate\", Some(\"retry\")),")
        .pre("    GroupItem::new(\"good\", \"Good response\", Some(\"check\")),")
        .pre("    GroupItem::new(\"share\", \"Share\", Some(\"share\")),")
        .pre("];")
        .raw("items", "items")
        .s("variant", v.s("variant"), "floating")
        .s("size", v.s("size"), "md")
        .b("vertical", v.b("vertical"))
        .raw("icon_only", (!v.b("labels")).to_string())
        .s("highlight_style", v.s("highlight_style"), "pill")
        .b("auto_hide", v.b("auto_hide"))
        .done()
}

fn expanding(v: &Vals, now: ClockMs) -> Element {
    let default_expanded = v.b("first_open").then_some(0);
    rsx! {
        ExpandingButtonGroup {
            items: expand(),
            size: v.string("size"),
            default_expanded,
            labels: v.b("labels"),
            delay_ms: v.f("delay") as u64,
            now,
        }
    }
}

fn expanding_code(v: &Vals) -> String {
    Code::new("ExpandingButtonGroup")
        .pre("let items = vec![")
        .pre("    GroupItem::new(\"copy\", \"Copy\", Some(\"copy\")).done(\"Copied\"),")
        .pre("    GroupItem::new(\"branch\", \"Branch\", Some(\"branch\")).done(\"Branched\"),")
        .pre("    GroupItem::new(\"share\", \"Share\", Some(\"share\")).done(\"Link copied\"),")
        .pre("];")
        .raw("items", "items")
        .s("size", v.s("size"), "md")
        .raw(
            "default_expanded",
            if v.b("first_open") { "Some(0)" } else { "None" },
        )
        .b("labels", v.b("labels"))
        .raw("delay_ms", v.s("delay"))
        .raw("now", "clock.now()")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
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
            slug: "icon-button",
            name: "Icon button",
            category: "buttons",
            summary: "A square button that is only an icon: ghost, secondary or primary, five sizes, a toggle that morphs its icon, a dot or count badge.",
            dioxus: "IconButton",
            gpui: "IconButtonEl",
            spec: "button",
            source: "primitives/buttons.rs",
            controls: vec![
                sel(
                    "icon",
                    "Icon",
                    &["copy", "search", "bell", "pencil", "trash", "share"],
                    "copy",
                ),
                txt("label", "Label (tooltip)", "Copy"),
                sel(
                    "variant",
                    "Variant",
                    &["ghost", "secondary", "primary"],
                    "ghost",
                ),
                sel("size", "Size", &["xs", "sm", "md", "lg", "xl"], "md"),
                tog("toggle", "Toggle (mute → stop)", false),
                sel("badge", "Badge", &["none", "dot", "count"], "none"),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("ghost", "Quiet toolbar action.", &[]),
                var(
                    "secondary-badge",
                    "Outlined, with an unread count.",
                    &[
                        ("variant", "secondary"),
                        ("icon", "bell"),
                        ("label", "Notifications"),
                        ("badge", "count"),
                    ],
                ),
                var(
                    "primary-xl",
                    "The large round action on mobile.",
                    &[
                        ("variant", "primary"),
                        ("size", "xl"),
                        ("icon", "pencil"),
                        ("label", "New thread"),
                    ],
                ),
                var(
                    "toggle",
                    "aria-pressed toggle whose icon morphs.",
                    &[("toggle", "true"), ("label", "Mute")],
                ),
            ],
            render: icon_button,
            code: icon_button_code,
            keyboard: vec![
                ("Enter / Space", "Press or toggle"),
                ("Tab", "Focus shows the label as a tooltip"),
            ],
            a11y: vec![
                "Always carries aria-label; the same text is its tooltip.",
                "Toggles expose aria-pressed.",
            ],
            responsive: vec!["Five sizes from 16 px (xs, dense IDE toolbars) to 40 px (xl)."],
            height: 180,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "action-button",
            name: "Action button",
            category: "buttons",
            summary: "A button that runs something and reports back: idle, pending with a spinner, a check on success, an error that retries with back-off.",
            dioxus: "ActionButton",
            gpui: "ActionButtonEl",
            spec: "button",
            source: "primitives/buttons.rs",
            controls: vec![
                sel(
                    "phase",
                    "Phase",
                    &["idle", "pending", "success", "retry"],
                    "idle",
                ),
                sel("icon", "Icon", &["up", "play", "spark"], "up"),
                tog("shortcut", "Shortcut chip", true),
            ],
            variants: vec![
                var("idle", "Ready to run.", &[]),
                var(
                    "pending",
                    "Spinner after a short delay.",
                    &[("phase", "pending")],
                ),
                var("success", "Check and past tense.", &[("phase", "success")]),
                var(
                    "retry-backoff",
                    "Failed; retries after a counted back-off.",
                    &[("phase", "retry")],
                ),
            ],
            render: action,
            code: action_code,
            keyboard: vec![
                ("Enter / Space", "Run"),
                (
                    "Shortcut",
                    "The chip's shortcut runs it from anywhere on the page",
                ),
            ],
            a11y: vec![
                "Results are announced through a polite live region inside the button.",
                "Pending sets aria-busy; the label changes to the present tense.",
            ],
            responsive: vec!["Sits inline in toolbars and forms like a regular button."],
            height: 180,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "split-button",
            name: "Split button",
            category: "buttons",
            summary: "A main action with a menu of alternatives; it can remember the last choice and promote it to the main half.",
            dioxus: "SplitButton",
            gpui: "SplitButtonEl",
            spec: "button-groups",
            source: "primitives/buttons.rs",
            controls: vec![
                sel("variant", "Variant", &["primary", "secondary"], "primary"),
                tog("remember", "Remember choice", true),
                tog("open", "Menu open", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("primary", "Merge with alternatives.", &[]),
                var(
                    "secondary",
                    "Quieter, next to a primary.",
                    &[("variant", "secondary")],
                ),
                var("open", "Menu open.", &[("open", "true")]),
            ],
            render: split,
            code: split_code,
            keyboard: vec![
                ("Enter / Space", "Main action"),
                (
                    "↓ / ↑ on the arrow",
                    "Open the menu on the first or last item",
                ),
                ("Esc", "Close the menu, focus back on the arrow"),
            ],
            a11y: vec![
                "Two real buttons; the arrow has aria-haspopup=menu and aria-expanded.",
                "The menu is a role=menu with roving focus.",
            ],
            responsive: vec!["The menu opens below and aligns to the button's start edge."],
            height: 260,
            align: Align::Start,
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
        Demo {
            slug: "floating-button-group",
            name: "Floating button group",
            category: "buttons",
            summary: "The quiet tray under a message: a highlight that glides between buttons as a pill, underline or glow, and a tray that can hide when idle.",
            dioxus: "FloatingButtonGroup",
            gpui: "ButtonGroupEl",
            spec: "button-groups",
            source: "primitives/buttons.rs",
            controls: vec![
                sel("variant", "Variant", &["floating", "muted"], "floating"),
                sel(
                    "highlight_style",
                    "Highlight",
                    &["pill", "underline", "glow"],
                    "pill",
                ),
                sel("size", "Size", &["sm", "md"], "md"),
                tog("labels", "Labels", false),
                tog("vertical", "Vertical", false),
                tog("auto_hide", "Auto-hide", false),
            ],
            variants: vec![
                var("floating", "Raised tray, pill highlight.", &[]),
                var(
                    "muted-underline",
                    "Flat tray, underline highlight.",
                    &[("variant", "muted"), ("highlight_style", "underline")],
                ),
                var(
                    "glow-labels",
                    "Labels on, glow highlight.",
                    &[
                        ("highlight_style", "glow"),
                        ("labels", "true"),
                        ("size", "sm"),
                    ],
                ),
            ],
            render: floating,
            code: floating_code,
            keyboard: vec![
                ("Tab", "Into the tray"),
                (
                    "Tab",
                    "Each button is a tab stop; focus moves the highlight",
                ),
            ],
            a11y: vec![
                "Icon-only buttons keep their labels as aria-label and tooltips.",
                "The gliding highlight is decorative (aria-hidden).",
            ],
            responsive: vec![
                "vertical turns the tray into a side rail; tooltips move to tooltip_side.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "expanding-button-group",
            name: "Expanding button group",
            category: "buttons",
            summary: "Icon buttons that open into labels one at a time, then confirm in place (Copy → Copied).",
            dioxus: "ExpandingButtonGroup",
            gpui: "ExpandingGroupEl",
            spec: "button-groups",
            source: "primitives/buttons.rs",
            controls: vec![
                sel("size", "Size", &["sm", "md"], "md"),
                tog("first_open", "First open", true),
                tog("labels", "Labels always on", false),
                rng("delay", "Hover delay ms", 0.0, 400.0, 50.0, "0"),
            ],
            variants: vec![
                var("default", "One open at a time.", &[]),
                var(
                    "labels",
                    "Labels always shown.",
                    &[("labels", "true"), ("first_open", "false")],
                ),
                var("small", "Small size.", &[("size", "sm")]),
            ],
            render: expanding,
            code: expanding_code,
            keyboard: vec![
                ("Tab", "Focus opens the button's label"),
                (
                    "Enter / Space",
                    "Run; the label confirms in place for a moment",
                ),
            ],
            a11y: vec!["Each button keeps its label as aria-label whether open or not."],
            responsive: vec!["labels keeps every label open where hover is not available."],
            height: 180,
            align: Align::Center,
            live: true,
        },
    ]
}
