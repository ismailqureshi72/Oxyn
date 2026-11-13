//! Tool calls, plans and sub-agents.

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{ClockMs, run::PlanState};
use oxyn_dioxus::{PlanCard, StepTimeline, SubagentTree, TodoList, ToolCallRow, ToolStack};

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

fn stack(v: &Vals, now: ClockMs) -> Element {
    let (calls, entries) = data::explore();
    let (calls, entries) = if v.s("turn") == "parallel" {
        (data::parallel(), vec![])
    } else {
        (calls, entries)
    };
    let done = v.b("done").then(|| Duration::from_secs(31));
    let term = v.s("variant") == "terminal";
    rsx! {
        div { class: if term { "oxd-term oxd-w560" } else { "oxd-w560" },
            if !term { p { class: "oxd-msg", "I'll check how the refresh path takes the lock." } }
            ToolStack { calls, entries, now, variant: v.string("variant"), density: v.string("density"), turn_done: done, row: v.string("row") }
        }
    }
}

fn stack_code(v: &Vals) -> String {
    let mut c = Code::new("ToolStack")
        .raw("calls", "turn.calls.clone()")
        .raw(
            "entries",
            "turn.entries.clone() // order with breaks for prose",
        )
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "transcript")
        .s("density", v.s("density"), "normal")
        .s("row", v.s("row"), "row");
    if v.b("done") {
        c = c.raw("turn_done", "Some(turn.worked_for())");
    }
    c.done()
}

fn todo(v: &Vals, now: ClockMs) -> Element {
    let plan = if v.b("all_done") {
        data::plan_done()
    } else {
        data::plan()
    };
    let agent = data::state(v.s("agent"));
    rsx! {
        div { class: "oxd-w480",
            TodoList { plan, agent, now, variant: v.string("variant"), expanded: v.b("expanded"), strike: v.b("strike") }
        }
    }
}

fn todo_code(v: &Vals) -> String {
    Code::new("TodoList")
        .raw("plan", "plan()")
        .raw("agent", "agent.state()")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "checklist")
        .b("expanded", v.b("expanded"))
        .b("strike", v.b("strike"))
        .done()
}

fn timeline(v: &Vals, now: ClockMs) -> Element {
    let plan = if v.b("all_done") {
        data::plan_done()
    } else {
        data::plan()
    };
    let calls = if v.b("calls") {
        data::explore().0
    } else {
        vec![]
    };
    rsx! { div { class: "oxd-w520", StepTimeline { plan, calls, now, fold_on_done: v.b("fold") } } }
}

fn timeline_code(v: &Vals) -> String {
    Code::new("StepTimeline")
        .raw("plan", "plan()")
        .raw("calls", "calls() // shown under the step that made them")
        .raw("now", "clock.now()")
        .b("fold_on_done", v.b("fold"))
        .done()
}

fn plan_card(v: &Vals, now: ClockMs) -> Element {
    let mut plan = match v.s("state") {
        "proposed" => data::plan_proposed(),
        _ => data::plan(),
    };
    match v.s("state") {
        "drafting" => plan.state = PlanState::Drafting,
        "executing" => plan.state = PlanState::Executing,
        "superseded" => {
            plan = data::plan_proposed();
            plan.state = PlanState::Superseded { by: 3 };
            plan.version = 2;
        }
        _ => {}
    }
    rsx! { div { class: "oxd-w520", PlanCard { plan, now, gated: v.b("gated") } } }
}

fn plan_code(v: &Vals) -> String {
    Code::new("PlanCard")
        .raw("plan", "plan()")
        .raw("now", "clock.now()")
        .b("gated", v.b("gated"))
        .raw("on_build", "move |_| agent.run_plan()")
        .raw("on_revise", "move |_| composer.focus()")
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
    vec![
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
            slug: "tool-stack",
            name: "Tool stack",
            category: "tools",
            summary: "A turn's tool calls grouped the way a transcript reads them: runs of reads fold into one line, parallel calls show live, and a finished turn folds into \"Worked for 31s\".",
            dioxus: "ToolStack",
            gpui: "ToolStackEl",
            spec: "tools",
            source: "run/tool_call.rs",
            controls: vec![
                sel("turn", "Turn", &["explore", "parallel"], "explore"),
                sel(
                    "variant",
                    "Variant",
                    &["transcript", "worked-for", "terminal"],
                    "transcript",
                ),
                sel(
                    "density",
                    "Density",
                    &["hidden", "compact", "normal", "verbose"],
                    "normal",
                ),
                sel("row", "Row", &["row", "card", "minimal"], "row"),
                tog("done", "Turn finished", false),
            ],
            variants: vec![
                var("transcript", "Grouped with a prose break.", &[]),
                var(
                    "parallel",
                    "Three calls in flight.",
                    &[("turn", "parallel")],
                ),
                var(
                    "worked-for",
                    "Folded after the turn.",
                    &[("variant", "worked-for"), ("done", "true")],
                ),
                var(
                    "terminal",
                    "Terminal transcript.",
                    &[("variant", "terminal")],
                ),
            ],
            render: stack,
            code: stack_code,
            keyboard: vec![
                ("Enter / Space", "Open or close a group"),
                ("← / →", "Close / open a group row (tree semantics)"),
            ],
            a11y: vec![
                "Group rows use tree semantics (treeitem, ← / →); each call row is a button with aria-expanded.",
            ],
            responsive: vec!["Groups keep long turns short; density trades detail for space."],
            height: 340,
            align: Align::Start,
            live: true,
        },
        Demo {
            slug: "todo-list",
            name: "Todo list",
            category: "tools",
            summary: "The agent's task list: done, active, blocked, pending and skipped items, as a checklist or a dock above the composer that fades once everything is done.",
            dioxus: "TodoList",
            gpui: "TodoListEl",
            spec: "plan-todo",
            source: "run/plan.rs",
            controls: vec![
                sel("variant", "Variant", &["checklist", "dock"], "checklist"),
                sel(
                    "agent",
                    "Agent state",
                    &["streaming", "tool-running", "cancelled", "done"],
                    "streaming",
                ),
                tog("expanded", "Dock expanded", true),
                tog("strike", "Strike done items", false),
                tog("all_done", "All done", false),
            ],
            variants: vec![
                var("checklist", "Every item with its status.", &[]),
                var("dock", "Docked above the composer.", &[("variant", "dock")]),
                var(
                    "stopped",
                    "The run was stopped.",
                    &[
                        ("variant", "dock"),
                        ("agent", "cancelled"),
                        ("expanded", "false"),
                    ],
                ),
                var(
                    "strike",
                    "Done items struck through.",
                    &[("strike", "true")],
                ),
            ],
            render: todo,
            code: todo_code,
            keyboard: vec![("Enter / Space", "Expand or collapse the dock")],
            a11y: vec![
                "role=list labelled \"Agent tasks, 2 of 6 done\"; each item reads its label, status and duration.",
                "An item becoming active is announced only if it stays active for 2 s.",
            ],
            responsive: vec![
                "checklist fills a panel; dock sits above the composer and expands on demand.",
            ],
            height: 320,
            align: Align::Start,
            live: true,
        },
        Demo {
            slug: "step-timeline",
            name: "Step timeline",
            category: "tools",
            summary: "The plan as a vertical timeline, each step with its time and the tool calls it made; folds to one line once every step is done.",
            dioxus: "StepTimeline",
            gpui: "StepTimelineEl",
            spec: "plan-todo",
            source: "run/plan.rs",
            controls: vec![
                tog("calls", "Show calls under steps", true),
                tog("all_done", "All done", false),
                tog("fold", "Fold when done", true),
            ],
            variants: vec![
                var("vertical", "Steps with their calls.", &[]),
                var(
                    "folded",
                    "Everything done, folded.",
                    &[("all_done", "true")],
                ),
            ],
            render: timeline,
            code: timeline_code,
            keyboard: vec![("Enter / Space", "Open a folded timeline or a step's calls")],
            a11y: vec!["An ordered list; nested calls are a sub-list."],
            responsive: vec!["Fills its container; fold_on_done keeps finished plans to one line."],
            height: 360,
            align: Align::Start,
            live: true,
        },
        Demo {
            slug: "plan-card",
            name: "Plan card",
            category: "tools",
            summary: "A plan as a document you review before the agent runs it: drafting, proposed with Approve or Build, executing, or superseded by a newer version.",
            dioxus: "PlanCard",
            gpui: "PlanCardEl",
            spec: "plan-approval",
            source: "run/plan.rs",
            controls: vec![
                sel(
                    "state",
                    "State",
                    &["proposed", "drafting", "executing", "superseded"],
                    "proposed",
                ),
                tog("gated", "Needs sign-off", true),
            ],
            variants: vec![
                var("proposed", "Waiting for sign-off.", &[]),
                var(
                    "drafting",
                    "Still being written.",
                    &[("state", "drafting"), ("gated", "false")],
                ),
                var(
                    "executing",
                    "Running.",
                    &[("state", "executing"), ("gated", "false")],
                ),
                var(
                    "superseded",
                    "Replaced by v3.",
                    &[("state", "superseded"), ("gated", "false")],
                ),
            ],
            render: plan_card,
            code: plan_code,
            keyboard: vec![
                ("Tab", "To Approve / Build and Revise"),
                ("Enter", "Activate the focused action"),
            ],
            a11y: vec!["role=region labelled with the plan title and version."],
            responsive: vec!["Fills its container."],
            height: 380,
            align: Align::Start,
            live: false,
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
    ]
}
