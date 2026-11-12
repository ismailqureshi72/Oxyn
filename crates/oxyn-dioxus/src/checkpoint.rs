//! Checkpoints (`checkpoint-timeline.md`): `CheckpointLanes` (attempts as lanes, like git for
//! agents), `CheckpointMarker` (inline marker and hover-revert), `CheckpointList` (side list and
//! terminal), and `RestoreToast` (undo window).

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    checkpoint::{
        Checkpoint, CheckpointKind, CheckpointStatus, RestoreAction, RestoreFlow, header, lanes,
    },
};

use crate::composer::icons::Icon;

fn status_cls(s: CheckpointStatus) -> &'static str {
    match s {
        CheckpointStatus::Running => "running",
        CheckpointStatus::Done => "done",
        CheckpointStatus::Failed => "failed",
        CheckpointStatus::Stopped => "stopped",
    }
}

/// Run checkpoints as lanes: rewinding branches a new attempt; the old one fades but stays.
#[component]
pub fn CheckpointLanes(
    /// Checkpoints in the order taken.
    checkpoints: Vec<Checkpoint>,
    /// Where the agent stands (default: last).
    #[props(default)]
    head: Option<String>,
    /// What the run is for.
    #[props(default)]
    task: Option<String>,
    /// Rewind pressed.
    #[props(default)]
    on_rewind: EventHandler<String>,
) -> Element {
    let rows = lanes(&checkpoints, head.as_deref());
    let n_lanes = rows.iter().map(|r| r.lane).max().unwrap_or(0) + 1;
    let gutter = 18 * n_lanes + 8;
    rsx! {
        div { class: "oxyn-cp lanes",
            div { class: "bar",
                span { class: "t", "Checkpoints" }
                if let Some(t) = task { span { class: "task", "· {t}" } }
                span { class: "sp" }
                span { class: "n", "aria-live": "polite", "{header(&checkpoints)}" }
            }
            ol { class: "rows", style: "--g:{gutter}px",
                for r in rows {
                    {
                        let c = checkpoints[r.index].clone();
                        let id = c.id.clone();
                        let st = status_cls(c.status);
                        let cls = format!("row {st}{}{}", if r.on_path { " path" } else { " off" }, if r.current { " cur" } else { "" });
                        rsx! {
                            li { class: "{cls}", "aria-current": if r.current { "step" } else { "false" }, "aria-label": "{r.aria}", tabindex: if r.current { "0" } else { "-1" },
                                svg { class: "lane", width: "{gutter}", height: "36", view_box: "0 0 {gutter} 36",
                                    for l in 0..n_lanes {
                                        if l == r.lane || r.branch_from == Some(l) {
                                            line { x1: "{9 + l * 18}", y1: "0", x2: "{9 + l * 18}", y2: if l == r.lane { "36" } else { "8" }, class: if r.on_path && l == r.lane { "ln on" } else { "ln" } }
                                        }
                                    }
                                    if let Some(f) = r.branch_from {
                                        path { d: "M{9 + f * 18} 6 C {9 + f * 18} 18, {9 + r.lane * 18} 6, {9 + r.lane * 18} 18", class: if r.on_path { "ln on" } else { "ln" }, fill: "none" }
                                    }
                                    circle { cx: "{9 + r.lane * 18}", cy: "18", r: if r.current { "5" } else { "4" }, class: "dot {st}" }
                                }
                                span { class: if c.status == CheckpointStatus::Running { "tt shim" } else { "tt" }, "{c.title}" }
                                if c.status == CheckpointStatus::Failed { span { class: "badge failed", "Failed" } }
                                if c.status == CheckpointStatus::Stopped { span { class: "badge", "Stopped" } }
                                if r.lane > 0 && r.branch_from.is_some() { span { class: "badge att", "Attempt {r.lane + 1}" } }
                                span { class: "sp" }
                                span { class: "meta",
                                    if c.files == 1 { span { class: "files", "1 file" } } else if c.files > 1 { span { class: "files", "{c.files} files" } }
                                    span { class: "du", "{r.time}" }
                                }
                                if !r.current && c.status != CheckpointStatus::Running {
                                    button { class: "rw", r#type: "button", tabindex: "-1", onclick: move |_| on_rewind.call(id.clone()), Icon { name: "retry", size: 12.0 } "Rewind" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A checkpoint between turns (`inline-marker`) or on a user message (`hover-revert`).
#[component]
pub fn CheckpointMarker(
    /// The checkpoint.
    checkpoint: Checkpoint,
    /// Wall-clock label ("3:42 PM").
    time: String,
    /// `inline-marker` or `hover-revert`.
    #[props(default = "inline-marker".to_string())]
    variant: String,
    /// Show actions (hovered or focused).
    #[props(default)]
    revealed: bool,
    /// Menu open.
    #[props(default)]
    menu: bool,
    /// The agent is running (restore blocked).
    #[props(default)]
    blocked: bool,
    /// Current checkpoint ("You are here").
    #[props(default)]
    current: bool,
    /// Action chosen.
    #[props(default)]
    on_action: EventHandler<RestoreAction>,
) -> Element {
    let c = checkpoint;
    let glyph = if c.chat_only { "◇" } else { "◆" };
    let what = if c.chat_only {
        "no file changes".to_string()
    } else {
        format!(
            "{} {}",
            c.files,
            if c.files == 1 { "file" } else { "files" }
        )
    };
    let kind = match c.kind {
        CheckpointKind::Manual => " · saved",
        CheckpointKind::Milestone => " · milestone",
        CheckpointKind::Auto => "",
    };
    let menu_el = rsx! {
        if menu {
            div { class: "menu", role: "menu",
                for (a, key) in RestoreAction::ALL {
                    {
                        let off = !a.offered(&c) || (blocked && a != RestoreAction::Fork);
                        rsx! {
                            button { class: "mi", role: "menuitem", r#type: "button", disabled: off, "aria-keyshortcuts": "{key}",
                                onclick: move |_| on_action.call(a),
                                span { "{a.menu()}" }
                                if off && blocked && a != RestoreAction::Fork { span { class: "why", "Stop the agent to restore" } }
                                span { class: "sp" }
                                kbd { "{key}" }
                            }
                        }
                    }
                }
            }
        }
    };
    if variant == "hover-revert" {
        return rsx! {
            div { class: "oxyn-cp hover", style: "position:relative",
                button { class: if revealed { "rev on" } else { "rev" }, r#type: "button", "aria-label": "Restore checkpoint from {time}", "aria-haspopup": "menu",
                    Icon { name: "retry", size: 13.0 }
                }
                {menu_el}
            }
        };
    }
    rsx! {
        div { class: if current { "oxyn-cp marker cur" } else if revealed { "oxyn-cp marker rev" } else { "oxyn-cp marker" }, tabindex: "0",
            "aria-label": "Checkpoint, {time}, {what}",
            span { class: "hl" }
            span { class: "dm", "{glyph}" }
            span { class: "lb", if current { "You are here · " } "Checkpoint · {time} · {what}{kind}" }
            span { class: "hl" }
            if revealed || menu {
                span { class: "acts",
                    button { class: "rs", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu}", Icon { name: "retry", size: 12.0 } "Restore" Icon { name: "chevron", size: 11.0 } }
                }
            }
            {menu_el}
        }
    }
}

/// Checkpoints as a side list grouped by day (`side-list`) or a rewind list (`terminal`).
#[component]
pub fn CheckpointList(
    /// Checkpoints, newest last.
    checkpoints: Vec<Checkpoint>,
    /// Wall-clock labels, same order.
    times: Vec<String>,
    /// `side-list` or `terminal`.
    #[props(default = "side-list".to_string())]
    variant: String,
    /// Selected index.
    #[props(default)]
    selected: Option<usize>,
) -> Element {
    if variant == "terminal" {
        return rsx! {
            div { class: "oxyn-cp term",
                div { class: "dim", "Rewind to before…" }
                for (i, c) in checkpoints.iter().enumerate().rev() {
                    div { class: if selected == Some(i) { "opt sel" } else { "opt" },
                        if selected == Some(i) { "› " } else { "  " }
                        "{checkpoints.len() - i}. {c.title}"
                        span { class: "dim", "  {times.get(i).cloned().unwrap_or_default()}" }
                        if c.files > 0 { span { class: "dim", "  +{c.lines.0} −{c.lines.1}" } }
                    }
                }
                if selected.is_some() {
                    div { class: "dim", "  1 Restore files · 2 Rewind chat · 3 Both · 4 Fork · esc never mind" }
                }
            }
        };
    }
    rsx! {
        div { class: "oxyn-cp side",
            div { class: "day", "Today" }
            ul {
                for (i, c) in checkpoints.iter().enumerate().rev() {
                    li { class: if selected == Some(i) { "it sel" } else { "it" },
                        span { class: "dm", if c.chat_only { "◇" } else { "◆" } }
                        div { class: "tx",
                            div { class: "tt", "{c.title}" }
                            div { class: "mt",
                                "{times.get(i).cloned().unwrap_or_default()}"
                                if c.kind == CheckpointKind::Manual { " · saved" }
                            }
                        }
                        if c.files > 0 { span { class: "ln", span { class: "add", "+{c.lines.0}" } span { class: "del", "\u{2212}{c.lines.1}" } } }
                    }
                }
            }
        }
    }
}

/// The restore toast with its undo hairline.
#[component]
pub fn RestoreToast(
    /// The flow.
    flow: RestoreFlow,
    /// Shared clock.
    now: ClockMs,
    /// Toast action pressed.
    #[props(default)]
    on_action: EventHandler<()>,
) -> Element {
    let Some((text, action, left)) = flow.toast(now) else {
        return rsx! {};
    };
    let tone = match flow {
        RestoreFlow::Partial { .. } => "warn",
        RestoreFlow::Failed { .. } => "danger",
        _ => "",
    };
    rsx! {
        div { class: "oxyn-cp toast {tone}", role: "status",
            Icon { name: "retry", size: 13.0 }
            span { "{text}" }
            if let Some(a) = action { button { r#type: "button", onclick: move |_| on_action.call(()), "{a}" } }
            if matches!(flow, RestoreFlow::Restored { .. }) { span { class: "hair", style: "width:{left * 100.0:.0}%" } }
        }
    }
}
