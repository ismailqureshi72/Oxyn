//! Surfaces (plan #26): dialog, sheet, sheet stack, bottom sheet, card, scroll area, control
//! center. Overlays render `contained` inside a small app frame so they open in the stage.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::control_center::{Channel, FocusMode},
};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    BottomSheet, ControlCenter, Line, ScrollArea, Sheet, SheetStack, StackPage, StackRow,
};
use oxyn_dioxus::{Card, Dialog, Switch};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

/// A small app frame behind an overlay: a title bar with the trigger and a few faint rows.
pub(crate) fn frame(bar: Element, overlay: Element) -> Element {
    rsx! {
        div { class: "oxd-app",
            div { class: "oxd-app-bar",
                span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" }
                span { class: "oxd-app-title", "Atlas · Run 248" }
                span { class: "oxd-app-sp" }
                {bar}
            }
            div { class: "oxd-app-body", "aria-hidden": "true",
                div { class: "oxd-fl w70" } div { class: "oxd-fl w90" } div { class: "oxd-fl w50" }
                div { class: "oxd-fl w80" } div { class: "oxd-fl w60" } div { class: "oxd-fl w85" }
            }
            {overlay}
        }
    }
}

fn dialog(v: &Vals, _: ClockMs) -> Element {
    let danger = v.s("tone") == "danger";
    let body = if danger {
        rsx! { p { style: "margin:0", "The agent's worktree, 14 checkpoints and the run history for " b { "atlas-refactor" } " will be deleted. This can't be undone." } }
    } else {
        rsx! {
            div { style: "display:flex;flex-direction:column;gap:10px",
                p { style: "margin:0", "Pick what the agent may do without asking. You can change this per run." }
                Switch { label: "Edit files in the worktree", checked: true, size: "sm" }
                Switch { label: "Run tests and linters", checked: true, size: "sm" }
                Switch { label: "Open pull requests", size: "sm" }
            }
        }
    };
    let (title, desc, confirm) = if danger {
        (
            "Delete project?",
            "Atlas · 3 agents · 214 runs",
            "Delete project",
        )
    } else {
        ("Agent permissions", "Applies to new runs in Atlas", "Save")
    };
    rsx! {
        {frame(rsx! {
            Dialog {
                title: if v.s("title").is_empty() { title.to_string() } else { v.string("title") },
                description: Some(desc.to_string()),
                open: v.b("open"),
                trigger_label: Some(if danger { "Delete…".to_string() } else { "Permissions".to_string() }),
                size: v.string("size"),
                tone: v.string("tone"),
                close_on_backdrop: v.b("close_on_backdrop"),
                close_on_escape: v.b("close_on_escape"),
                close_button: v.b("close_button"),
                modal: v.b("modal"),
                contained: true,
                confirm_label: v.b("footer").then(|| confirm.to_string()),
                busy_ms: v.u("busy_ms") as u32,
                {body}
            }
        }, rsx! {})}
    }
}

fn dialog_code(v: &Vals) -> String {
    let mut c = Code::new("Dialog")
        .raw("title", q("Agent permissions"))
        .raw("description", q("Applies to new runs in Atlas"))
        .raw("trigger_label", q("Permissions"))
        .s("size", v.s("size"), "md")
        .s("tone", v.s("tone"), "default");
    if !v.b("close_on_backdrop") {
        c = c.raw("close_on_backdrop", "false");
    }
    if !v.b("modal") {
        c = c.raw("modal", "false");
    }
    c.opt("confirm_label", if v.b("footer") { "Save" } else { "" })
        .raw("busy_ms", v.s("busy_ms"))
        .raw("on_confirm", "move |_| save()")
        .done()
}

fn card(v: &Vals, _: ClockMs) -> Element {
    let row = v.s("layout") == "row";
    rsx! {
        div { class: if row { "oxd-cards rows" } else { "oxd-cards" },
            Card {
                title: "Refresh-token race",
                description: Some("Two tabs refreshed at once and logged the user out. The agent added a single-flight lock.".to_string()),
                media: v.opt("media"),
                meta: Some("Maya Chen".to_string()),
                status: Some(v.string("status")),
                action: v.b("action").then(|| "Review".to_string()),
                details: v.b("quick_look").then(|| "Fix: wrap refresh() in a mutex keyed by account; retry the waiting request with the new token. 3 files changed, 2 tests added, all 148 tests pass.".to_string()),
                open: v.b("open"),
                edge: v.string("edge"),
                selectable: v.b("selectable"),
                layout: v.string("layout"),
                width: 290.0,
            }
            if !row {
                Card {
                    title: "Newsletter send",
                    description: Some("Waiting for your approval to email 12,480 subscribers.".to_string()),
                    media: Some("dots".to_string()),
                    meta: Some("Outreach agent".to_string()),
                    status: Some("Needs approval".to_string()),
                    action: v.b("action").then(|| "Open".to_string()),
                    edge: "attention",
                    selectable: v.b("selectable"),
                    width: 290.0,
                }
            }
        }
    }
}

fn card_code(v: &Vals) -> String {
    Code::new("Card")
        .raw("title", q("Refresh-token race"))
        .raw("description", q("Two tabs refreshed at once…"))
        .opt("media", v.s("media"))
        .raw("meta", q("Maya Chen"))
        .raw("status", q(v.s("status")))
        .opt("action", if v.b("action") { "Review" } else { "" })
        .opt(
            "details",
            if v.b("quick_look") {
                "Fix: wrap refresh() in a mutex…"
            } else {
                ""
            },
        )
        .s("edge", v.s("edge"), "none")
        .b("selectable", v.b("selectable"))
        .s("layout", v.s("layout"), "card")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![
        Demo {
            slug: "dialog",
            name: "Dialog",
            category: "surfaces",
            summary: "A modal window on a dimmed backdrop: title and quiet line, a body that scrolls under a fixed header and footer, Escape and backdrop close, focus kept inside and handed back.",
            dioxus: "Dialog",
            gpui: "DialogEl",
            spec: "dialog",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("open", "Open", true),
                sel("size", "Size", &["sm", "md", "lg", "full"], "md"),
                sel("tone", "Tone", &["default", "danger"], "default"),
                tog("footer", "Footer", true),
                tog("close_button", "Close button", true),
                tog("close_on_backdrop", "Backdrop closes", true),
                tog("close_on_escape", "Escape closes", true),
                tog("modal", "Modal", true),
                rng("busy_ms", "Confirm runs ms", 0.0, 3000.0, 100.0, "1200"),
                txt("title", "Title", ""),
            ],
            variants: vec![
                var("form", "Settings with a Save footer.", &[]),
                var(
                    "confirm-danger",
                    "An alert dialog with a red confirm that runs before closing.",
                    &[("tone", "danger"), ("size", "sm")],
                ),
                var(
                    "sticky",
                    "Backdrop and Escape don't close it.",
                    &[("close_on_backdrop", "false"), ("close_on_escape", "false")],
                ),
                var(
                    "floating",
                    "Non-modal: the page stays live behind it.",
                    &[("modal", "false")],
                ),
                var("closed", "Just the trigger.", &[("open", "false")]),
            ],
            render: dialog,
            code: dialog_code,
            keyboard: vec![
                ("Esc", "Close and return focus to the trigger"),
                ("Tab / Shift+Tab", "Cycle focus inside"),
                ("Enter", "Run the focused button"),
            ],
            a11y: vec![
                "role=dialog (alertdialog when danger) with aria-modal, named by the title and described by the quiet line.",
                "Focus moves into the dialog and wraps at both ends; closing hands it back to the trigger.",
                "While a confirm runs the dialog is aria-busy and can't be dismissed.",
            ],
            responsive: vec![
                "min(viewport − 32 px, size) wide and capped at the viewport height; the body scrolls.",
            ],
            height: 460,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "card",
            name: "Card",
            category: "surfaces",
            summary: "A content card with media, byline, status and an action that opens into a quick look; hover lifts it and zooms the media; status changes roll in word by word.",
            dioxus: "Card",
            gpui: "CardEl",
            spec: "card",
            source: "primitives/surfaces.rs",
            controls: vec![
                sel("media", "Media", &["grid", "dots", "waves", "none"], "grid"),
                txt("status", "Status", "Updated 2 minutes ago"),
                tog("action", "Action", true),
                tog("quick_look", "Quick look", true),
                tog("open", "Quick look open", false),
                sel(
                    "edge",
                    "Status edge",
                    &["none", "working", "attention", "done", "error"],
                    "working",
                ),
                tog("selectable", "Selectable", false),
                sel("layout", "Layout", &["card", "row"], "card"),
            ],
            variants: vec![
                var(
                    "working",
                    "An agent is working on it: a beam runs round the border.",
                    &[],
                ),
                var(
                    "quick-look",
                    "Opened into the quick look.",
                    &[("open", "true"), ("edge", "none")],
                ),
                var(
                    "selectable",
                    "Pick cards for a batch action.",
                    &[("selectable", "true"), ("edge", "none")],
                ),
                var(
                    "row",
                    "Compact list row.",
                    &[("layout", "row"), ("edge", "done")],
                ),
            ],
            render: card,
            code: card_code,
            keyboard: vec![
                ("Enter / Space", "On the title: open the quick look"),
                ("Esc", "Close it; focus returns to the title"),
                ("Tab", "Trapped inside the open quick look"),
            ],
            a11y: vec![
                "An article named by its h3 title; only the title is a button, so nested actions stay separate.",
                "Status changes are read through a polite status region.",
            ],
            responsive: vec![
                "A definite width that never exceeds its cell; hover lift and zoom only for a fine pointer.",
            ],
            height: 380,
            align: Align::Center,
            live: false,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 7] = [
    "dialog",
    "sheet",
    "sheet-stack",
    "bottom-sheet",
    "card",
    "scroll-area",
    "control-center",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
