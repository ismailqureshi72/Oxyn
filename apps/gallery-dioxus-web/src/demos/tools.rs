//! Tool calls, plans and sub-agents.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{ClockMs, run::PlanState};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{PlanCard, StepTimeline, TodoList, ToolStack};
use oxyn_dioxus::{SubagentTree, ToolCallRow};

use super::{Code, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

fn call_row(v: &Vals, now: ClockMs) -> Element {
    let expanded = match v.s("expanded") {
        "open" => Some(true),
        "closed" => Some(false),
        _ => None,
    };
    let term = v.s("variant") == "terminal";
    rsx! {
        div { class: if term { "oxd-term oxd-w560" } else { "oxd-w560" },
            ToolCallRow { call: data::tool_call(v.s("state")), now, variant: v.string("variant"), expanded }
        }
    }
}

fn call_code(v: &Vals) -> String {
    let mut c = Code::new("ToolCallRow")
        .pre("let call = ToolCall::new(\"ru\", \"grep\", started_at)")
        .pre("    .with_kind(ToolKind::Search)")
        .pre("    .with_target(Target::Query(\"refresh_token\".into()))")
        .pre("    .with_state(ToolCallState::Running);")
        .raw("call", "call")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "row");
    c = match v.s("expanded") {
        "open" => c.raw("expanded", "Some(true)"),
        "closed" => c.raw("expanded", "Some(false)"),
        _ => c,
    };
    c.raw(
        "on_action",
        "move |(id, action): (String, RowAction)| host.act(id, action)",
    )
    .done()
}

fn tree(v: &Vals, now: ClockMs) -> Element {
    let collapsed = if v.b("collapsed") {
        vec!["t".to_string()]
    } else {
        vec![]
    };
    let term = v.s("variant") == "terminal";
    rsx! {
        div { class: if term { "oxd-term oxd-w560" } else { "oxd-w560" },
            SubagentTree { graph: data::graph(), now, variant: v.string("variant"), display_depth: v.u("depth"), max_children: v.u("max"), collapsed }
        }
    }
}

fn tree_code(v: &Vals) -> String {
    let mut c = Code::new("SubagentTree")
        .raw("graph", "run.graph()")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "indented");
    if v.s("depth") != "3" {
        c = c.raw("display_depth", v.s("depth"));
    }
    if v.s("max") != "8" {
        c = c.raw("max_children", v.s("max"));
    }
    if v.b("collapsed") {
        c = c.raw("collapsed", "vec![\"t\".to_string()]");
    }
    c.raw("on_review", "move |id: String| open_approval(id)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![
        Demo {
            slug: "tool-call",
            name: "Tool call",
            category: "tools",
            summary: "One tool call in one line: a live timer, status and summary, MCP server badge, and a fold that opens to arguments (secrets masked) and output; approval and failure open on their own.",
            dioxus: "ToolCallRow",
            gpui: "ToolCallRowEl",
            spec: "tool-call",
            source: "run/tool_call.rs",
            controls: vec![
                sel("state", "State", &data::CALL_STATES, "running"),
                sel(
                    "variant",
                    "Variant",
                    &["row", "card", "minimal", "terminal"],
                    "row",
                ),
                sel("expanded", "Expanded", &["auto", "open", "closed"], "auto"),
            ],
            variants: vec![
                var("row", "Running with a live timer.", &[]),
                var(
                    "card",
                    "MCP call expanded with arguments.",
                    &[
                        ("variant", "card"),
                        ("state", "succeeded"),
                        ("expanded", "open"),
                    ],
                ),
                var("failed", "Failure opens by itself.", &[("state", "failed")]),
                var(
                    "minimal",
                    "Edit summary in one quiet line.",
                    &[("variant", "minimal"), ("state", "edit")],
                ),
                var(
                    "terminal",
                    "Terminal transcript style.",
                    &[("variant", "terminal"), ("state", "retry")],
                ),
            ],
            render: call_row,
            code: call_code,
            keyboard: vec![
                ("Enter / Space", "Fold open or closed"),
                ("Tab", "Into the row's actions (retry, copy, open)"),
            ],
            a11y: vec![
                "The row is a button with aria-expanded and aria-controls pointing at the detail.",
                "Each state has a full name (\"Running cargo test, 12 seconds\", \"Linear needs sign-in\").",
                "The MCP server is part of the name (\"via Linear\").",
            ],
            responsive: vec![
                "minimal suits narrow side panels; card suits full-width transcripts; terminal suits TUIs.",
            ],
            height: 260,
            align: Align::Start,
            live: true,
        },
        Demo {
            slug: "subagent-tree",
            name: "Sub-agent tree",
            category: "tools",
            summary: "Who started whom: an orchestrator and its sub-agents with state, time and results, approvals surfaced in place, depth and sibling limits, as an indented tree, a rail, a dock or a terminal.",
            dioxus: "SubagentTree",
            gpui: "SubagentTreeEl",
            spec: "subagent-tree",
            source: "run/tree.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["indented", "rail", "dock", "inline-wrapper", "terminal"],
                    "indented",
                ),
                rng("depth", "Display depth", 1.0, 4.0, 1.0, "3"),
                rng("max", "Max children", 1.0, 8.0, 1.0, "8"),
                tog("collapsed", "Collapse Test runner", false),
            ],
            variants: vec![
                var("indented", "The full tree.", &[]),
                var("rail", "A side rail.", &[("variant", "rail")]),
                var("dock", "Docked summary.", &[("variant", "dock")]),
                var("terminal", "Terminal tree.", &[("variant", "terminal")]),
            ],
            render: tree,
            code: tree_code,
            keyboard: vec![
                ("↑ / ↓", "Move between agents"),
                ("← / →", "Collapse / expand"),
                (
                    "Enter",
                    "Focus that agent's transcript, or review its approval",
                ),
            ],
            a11y: vec![
                "role=tree with treeitems, aria-level, aria-setsize, aria-posinset and aria-expanded.",
                "Rows read in full (\"Test runner, sub-agent of Orchestrator, needs your approval\").",
                "Failures and approvals are announced assertively; at most one announcement per 5 s.",
            ],
            responsive: vec![
                "display_depth and max_children fold deep or wide trees into \"+ n more\".",
            ],
            height: 340,
            align: Align::Start,
            live: true,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 6] = [
    "tool-call",
    "tool-stack",
    "todo-list",
    "step-timeline",
    "plan-card",
    "subagent-tree",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
