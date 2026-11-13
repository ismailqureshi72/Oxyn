//! Checkpoints: lanes, side list, inline marker, restore toast.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    checkpoint::{Checkpoint, CheckpointKind, RestoreFlow},
};
use oxyn_dioxus::{CheckpointLanes, CheckpointList, CheckpointMarker, RestoreToast};

use super::{Code, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

fn lanes(v: &Vals, _: ClockMs) -> Element {
    let mut cps = data::checkpoints();
    if !v.b("rewound") {
        cps.truncate(4);
    }
    let head = v.opt("head");
    rsx! {
        div { class: "oxd-w560",
            CheckpointLanes { checkpoints: cps, head, task: Some("Add Stripe checkout".to_string()) }
        }
    }
}

fn lanes_code(v: &Vals) -> String {
    let mut c = Code::new("CheckpointLanes")
        .pre("let checkpoints = vec![")
        .pre("    Checkpoint::new(\"c1\", None, \"Read the brief\", 9),")
        .pre("    Checkpoint::new(\"c2\", Some(\"c1\"), \"Built the pricing table\", 64).files(3, 120, 4),")
        .pre("    Checkpoint::new(\"c4\", Some(\"c2\"), \"Tests failing – 4 of 12\", 131).with(CheckpointStatus::Failed),")
        .pre("    // a rewind: the next attempt branches from c2")
        .pre("    Checkpoint::new(\"c5\", Some(\"c2\"), \"Switched to Stripe Elements\", 169),")
        .pre("];")
        .raw("checkpoints", "checkpoints");
    if let Some(h) = v.opt("head") {
        c = c.raw("head", q(&h));
    }
    c.raw("task", q("Add Stripe checkout"))
        .raw("on_rewind", "move |id: String| agent.rewind_to(id)")
        .done()
}

fn list(v: &Vals, _: ClockMs) -> Element {
    let term = v.s("variant") == "terminal";
    rsx! {
        div { class: if term { "oxd-term oxd-w480" } else { "oxd-w420" },
            CheckpointList { checkpoints: data::checkpoints(), times: data::checkpoint_times(), variant: v.string("variant"), selected: Some(v.u("selected")) }
        }
    }
}

fn list_code(v: &Vals) -> String {
    Code::new("CheckpointList")
        .raw("checkpoints", "checkpoints()")
        .raw("times", "times() // \"3:42 PM\", same order")
        .s("variant", v.s("variant"), "side-list")
        .raw("selected", format!("Some({})", v.s("selected")))
        .done()
}

fn marker(v: &Vals, _: ClockMs) -> Element {
    let cps = data::checkpoints();
    let cp = match v.s("kind") {
        "chat-only" => Checkpoint {
            chat_only: true,
            ..Checkpoint::new("m", None, "Asked for a summary", 0)
        },
        "manual" => {
            let mut m = Checkpoint::new("s", None, "Before the auth refactor", 0).files(3, 42, 17);
            m.kind = CheckpointKind::Manual;
            m
        }
        _ => cps[1].clone(),
    };
    let hover = v.s("variant") == "hover-revert";
    rsx! {
        div { class: "oxd-w520 oxd-room-menu",
            div { class: "oxd-turn", b { "You" } " refactor the auth middleware to use sessions" }
            div { class: "oxd-turn", b { "Agent" } " Edited 3 files +42 −17" }
            if hover {
                div { class: "oxd-turn oxd-hov",
                    CheckpointMarker { checkpoint: cp, time: "3:42 PM", variant: "hover-revert", revealed: v.b("revealed") }
                    span { b { "You" } " now add rate limiting" }
                }
            } else {
                CheckpointMarker { checkpoint: cp, time: "3:42 PM", revealed: v.b("revealed"), menu: v.b("menu"), blocked: v.b("blocked"), current: v.b("current") }
                div { class: "oxd-turn", b { "You" } " now add rate limiting" }
            }
        }
    }
}

fn marker_code(v: &Vals) -> String {
    Code::new("CheckpointMarker")
        .raw("checkpoint", "checkpoint.clone()")
        .raw("time", q("3:42 PM"))
        .s("variant", v.s("variant"), "inline-marker")
        .b("blocked", v.b("blocked"))
        .b("current", v.b("current"))
        .raw(
            "on_action",
            "move |a: RestoreAction| agent.restore(a) // files, conversation, both or fork",
        )
        .done()
}

fn toast(v: &Vals, now: ClockMs) -> Element {
    let flow = match v.s("flow") {
        "restoring" => RestoreFlow::Restoring {
            done: v.f("done") as u32,
            total: 7,
        },
        "partial" => RestoreFlow::Partial {
            label: "Restored 6 of 7 files · rate.rs is locked".into(),
        },
        _ => RestoreFlow::Restored {
            label: "Restored 3 files to 3:42 PM".into(),
            until: ClockMs(crate::BASE.0 + 8_000),
        },
    };
    rsx! { RestoreToast { flow, now } }
}

fn toast_code(v: &Vals) -> String {
    let flow = match v.s("flow") {
        "restoring" => format!("RestoreFlow::Restoring {{ done: {}, total: 7 }}", v.s("done")),
        "partial" => {
            "RestoreFlow::Partial { label: \"Restored 6 of 7 files · rate.rs is locked\".into() }"
                .into()
        }
        _ => "RestoreFlow::Restored { label: \"Restored 3 files to 3:42 PM\".into(), until: undo_deadline }".into(),
    };
    Code::new("RestoreToast")
        .raw("flow", flow)
        .raw("now", "clock.now()")
        .raw("on_action", "move |_| agent.undo_restore()")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "checkpoint-lanes",
            name: "Checkpoint lanes",
            category: "checkpoints-git",
            summary: "A run as a line of checkpoints, like git for agents: rewind to any of them and the new attempt branches out in its own lane, while the one it left fades back but stays.",
            dioxus: "CheckpointLanes",
            gpui: "CheckpointLanesEl",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                tog("rewound", "Rewound once (two attempts)", true),
                sel("head", "Head", &["none", "c2", "c4", "c5", "c6"], "none"),
            ],
            variants: vec![
                var("rewound", "Attempt two running after a rewind.", &[]),
                var(
                    "single",
                    "One attempt, failing tests.",
                    &[("rewound", "false")],
                ),
            ],
            render: lanes,
            code: lanes_code,
            keyboard: vec![
                (
                    "Tab",
                    "To the current checkpoint (one tab stop; the host moves focus between rows)",
                ),
                ("Enter / Space", "On a row's Rewind button: rewind there"),
            ],
            a11y: vec![
                "Each row carries a full aria-label (title, time, changes, status); the current one has aria-current=step.",
                "The run header sits in a polite live region.",
            ],
            responsive: vec!["Lanes stay narrow; titles take the rest of the width."],
            height: 340,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "checkpoint-list",
            name: "Checkpoint list",
            category: "checkpoints-git",
            summary: "Every checkpoint of a run in a side list with times and file counts, or as the terminal picker you open with Esc Esc.",
            dioxus: "CheckpointList",
            gpui: "",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["side-list", "terminal"],
                    "side-list",
                ),
                rng("selected", "Selected", 0.0, 5.0, 1.0, "4"),
            ],
            variants: vec![
                var("side-list", "A side panel list.", &[]),
                var("terminal", "Terminal picker.", &[("variant", "terminal")]),
            ],
            render: list,
            code: list_code,
            keyboard: vec![(
                "Esc Esc",
                "Opens the terminal picker in coding-agent TUIs (host binding)",
            )],
            a11y: vec!["Each row reads its title, time and file changes as text."],
            responsive: vec![
                "side-list fits a 280–360 px side panel; terminal fits an 80-column TUI.",
            ],
            height: 320,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "checkpoint-marker",
            name: "Checkpoint marker",
            category: "checkpoints-git",
            summary: "A quiet divider between chat turns that marks a checkpoint; on hover it offers Restore files, Rewind conversation, both or Fork, and says why restore is blocked while the agent runs.",
            dioxus: "CheckpointMarker",
            gpui: "",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["inline-marker", "hover-revert"],
                    "inline-marker",
                ),
                sel(
                    "kind",
                    "Checkpoint",
                    &["auto", "chat-only", "manual"],
                    "auto",
                ),
                tog("revealed", "Revealed (hover)", true),
                tog("menu", "Menu open", true),
                tog("blocked", "Agent running (blocked)", false),
                tog("current", "You are here", false),
            ],
            variants: vec![
                var("menu", "Restore choices open.", &[]),
                var(
                    "quiet",
                    "At rest between turns.",
                    &[("revealed", "false"), ("menu", "false")],
                ),
                var(
                    "blocked",
                    "Restore blocked while the agent runs.",
                    &[("blocked", "true")],
                ),
                var(
                    "hover-revert",
                    "A revert button on the user turn.",
                    &[("variant", "hover-revert")],
                ),
            ],
            render: marker,
            code: marker_code,
            keyboard: vec![
                ("Tab", "To the Restore button (aria-haspopup=menu)"),
                (
                    "Enter / Space",
                    "Open the restore menu; items show their shortcuts",
                ),
            ],
            a11y: vec![
                "The marker is named \"Checkpoint, 3:42 PM, 3 files changed\"; Restore has aria-haspopup and aria-expanded.",
                "Menu items are menuitems with aria-keyshortcuts; blocked items are disabled.",
            ],
            responsive: vec![
                "Sits between turns at the transcript width; the menu opens below it.",
            ],
            height: 320,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "restore-toast",
            name: "Restore toast",
            category: "checkpoints-git",
            summary: "What a restore did: progress while files are written, a done toast with an undo window that runs out, or a partial result that names what failed.",
            dioxus: "RestoreToast",
            gpui: "",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel(
                    "flow",
                    "Flow",
                    &["restored", "restoring", "partial"],
                    "restored",
                ),
                rng("done", "Files written", 0.0, 7.0, 1.0, "4"),
            ],
            variants: vec![
                var("restored", "Undo window counting down.", &[]),
                var("restoring", "Writing files.", &[("flow", "restoring")]),
                var("partial", "One file failed.", &[("flow", "partial")]),
            ],
            render: toast,
            code: toast_code,
            keyboard: vec![("Tab", "To Undo")],
            a11y: vec!["role=status, so the result is announced; Undo is a real button."],
            responsive: vec!["Hugs its text; place it at the bottom of the transcript."],
            height: 160,
            align: Align::Center,
            live: true,
        },
    ]
}
