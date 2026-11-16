//! Plan #32 (affectUI fidelity pass), overnight batch family B demos: the exact Agent Findings and
//! Agent Inbox, each with every affectUI style and state; part 2: Agent Crew; part 3: Agent Cursors, Agent Graph, Agent Handoff and Agent Lanes; part 4: Agent Ledger, Agent Monitor, Agent Queue, Agent Recap and Agent Sequence.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AgentCrew, AgentCursors, AgentFindings, AgentGraph, AgentHandoff, AgentInbox, AgentLanes,
    AgentLedger, AgentMonitor, AgentQueue, AgentRecap, AgentSequence,
};

use super::{Code, mk, sel, tog, var};
use crate::{Align, Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn cstyle(v: &Vals) -> String {
    if v.s("style") == "classic" {
        "classic".into()
    } else {
        String::new()
    }
}

// ---------- Agent Findings ----------

fn agent_findings(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("run"), v.s("filter"));
    let filter = match v.s("filter") {
        "auditor" => Some(0),
        "fuzzer" => Some(1),
        _ => None,
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:480px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentFindings { now, finished: v.s("run") == "finished", filter, style: cstyle(v),
                on_open: move |id: String| msg.set(format!("onOpen({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_findings_code(_: &Vals) -> String {
    Code::new("AgentFindings")
        .raw("agents", "agents()")
        .raw("findings", "findings()")
        .raw("title", "\"Verification\".into()")
        .raw("now", "clock.now()")
        .raw("on_open", "move |id| open(id)")
        .done()
}

// ---------- Agent Inbox ----------

fn agent_inbox(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("view"), v.b("open"));
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:740px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentInbox { now, view: v.string("view"), open: if v.b("open") { "t1".to_string() } else { String::new() }, style: cstyle(v),
                on_action: move |(a, t): (String, String)| msg.set(format!("on{}({t:?})", a[..1].to_uppercase() + &a[1..])) }
            {status(msg)}
        }
    }
}

fn agent_inbox_code(v: &Vals) -> String {
    let mut c = Code::new("AgentInbox").raw("tasks", "tasks()");
    if v.s("view") == "list" {
        c = c.raw("view", "\"list\".into()");
    }
    c.raw("now", "clock.now()")
        .raw("on_action", "move |(what, id)| handle(what, id)")
        .done()
}

// ---------- Agent Crew ----------

fn agent_crew(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("style"),
        v.s("project"),
        v.s("run"),
        v.b("open"),
        v.s("width")
    );
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "760px"
    };
    let at = (v.s("run") == "settled").then_some(26_500);
    let open = if v.b("open") {
        match v.s("project") {
            "launch" => "editor",
            "postgres" => "pg-lead",
            _ => "lead",
        }
    } else {
        ""
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentCrew { now, project: v.string("project"), at, open: open.to_string(), style: cstyle(v),
                on_steer: move |(id, a, m): (String, String, String)| msg.set(if m.is_empty() { format!("onSteer({id:?}, {a:?})") } else { format!("onSteer({id:?}, {a:?}, {m:?})") }),
                on_project: move |id: String| msg.set(format!("onProjectChange({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_crew_code(_: &Vals) -> String {
    Code::new("AgentCrew")
        .raw("root", "Some(crew())")
        .raw("handoffs", "handoffs()")
        .raw("projects", "projects()")
        .raw("now", "clock.now()")
        .raw(
            "on_steer",
            "move |(agent, action, message)| steer(agent, action, message)",
        )
        .raw("on_project", "move |id| switch_to(id)")
        .done()
}

// ---------- Agent Cursors ----------

fn agent_cursors(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("style"),
        v.s("tab"),
        v.s("run"),
        v.s("follow"),
        v.b("open")
    );
    let at = match v.s("run") {
        "settled" => Some(16_000),
        "writing" => Some(5_200),
        _ => None,
    };
    let following = match (v.s("follow"), v.s("tab")) {
        ("on", "table") => "ops",
        ("on", _) => "researcher",
        _ => "",
    };
    let open = if v.b("open") {
        if v.s("tab") == "table" {
            "s-ops"
        } else {
            "s-writer"
        }
    } else {
        ""
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:700px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentCursors { now, tab: v.string("tab"), at, following: following.to_string(), open: open.to_string(), style: cstyle(v),
                on_accept: move |s: oxyn_core::parity::cursorsfid::DocSuggestion| msg.set(format!("onAccept({:?})", s.id)),
                on_reject: move |s: oxyn_core::parity::cursorsfid::DocSuggestion| msg.set(format!("onReject({:?})", s.id)),
                on_accept_all: move |_| msg.set("onAcceptAll()".into()),
                on_following: move |id: String| msg.set(if id.is_empty() { "onFollowingChange(null)".into() } else { format!("onFollowingChange({id:?})") }),
                on_resolve: move |c: oxyn_core::parity::cursorsfid::DocComment| msg.set(format!("onResolveComment({:?})", c.id)),
                on_reply: move |(c, t): (oxyn_core::parity::cursorsfid::DocComment, String)| msg.set(format!("onReply({:?}, {t:?})", c.id)),
                on_tab: move |id: String| msg.set(format!("onTabChange({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_cursors_code(v: &Vals) -> String {
    let mut c = Code::new("AgentCursors")
        .raw("scene", "Some(scene())")
        .raw("tabs", "tabs()")
        .raw("tab", format!("{:?}.into()", v.s("tab")).as_str());
    if v.s("follow") == "on" {
        c = c.raw("following", "\"researcher\".into()");
    }
    c.raw("height", "470.0")
        .raw("now", "clock.now()")
        .raw("on_accept", "move |s| accept(s)")
        .raw("on_reject", "move |s| reject(s)")
        .raw("on_accept_all", "move |_| accept_all()")
        .raw("on_resolve", "move |c| resolve(c)")
        .done()
}

// ---------- Agent Graph ----------

fn agent_graph(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("run"),
        v.s("open"),
        v.s("width")
    );
    let at = match v.s("run") {
        "needs-you" => Some(15_000),
        "early" => Some(7_000),
        _ => None,
    };
    let open = match v.s("open") {
        "lead" => "lead",
        "writer" => "writer",
        _ => "",
    };
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "756px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentGraph { now, at, open: open.to_string(), style: cstyle(v),
                on_select: move |id: String| msg.set(if id.is_empty() { "onSelect(null)".into() } else { format!("onSelect({id:?})") }) }
            {status(msg)}
        }
    }
}

fn agent_graph_code(_: &Vals) -> String {
    Code::new("AgentGraph")
        .raw(
            "scene",
            "Some(GraphScene { nodes: nodes(), edges: edges(), calls: recent_calls() })",
        )
        .raw("now", "clock.now()")
        .raw("on_select", "move |id| show(id)")
        .done()
}

// ---------- Agent Handoff ----------

fn agent_handoff(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("run"),
        v.s("open"),
        v.s("width")
    );
    let at = match v.s("run") {
        "settled" => Some(18_000),
        "handing" => Some(8_000),
        _ => None,
    };
    let (open, item) = match v.s("open") {
        "card" => ("h-winback-1", ""),
        "lost" => ("h-winback-1", "h-winback-1/quotes"),
        _ => ("", ""),
    };
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "788px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentHandoff { now, at, open: open.to_string(), open_item: item.to_string(), style: cstyle(v),
                on_assign: move |(t, a): (String, String)| msg.set(format!("onAssign({t:?}, {a:?})")),
                on_restore: move |(h, i): (String, String)| msg.set(format!("onRestore({h:?}, {i:?})")) }
            {status(msg)}
        }
    }
}

fn agent_handoff_code(_: &Vals) -> String {
    Code::new("AgentHandoff")
        .raw(
            "scene",
            "Some(HandoffScene { agents: agents(), handoffs: handoffs(), tasks: tasks() })",
        )
        .raw("now", "clock.now()")
        .raw("on_assign", "move |(task, agent)| assign(task, agent)")
        .raw(
            "on_restore",
            "move |(handoff, item)| restore(handoff, item)",
        )
        .done()
}

// ---------- Agent Lanes ----------

fn agent_lanes(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("run"),
        v.s("open"),
        v.s("width")
    );
    let at = match v.s("run") {
        "settled" => Some(9_000),
        _ => None,
    };
    let open = match v.s("open") {
        "summary" => "scan",
        "decision" => "keys",
        _ => "",
    };
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "760px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentLanes { now, at, open: open.to_string(), style: cstyle(v),
                on_approve: move |id: String| msg.set(format!("onApprove({id:?})")),
                on_reject: move |id: String| msg.set(format!("onReject({id:?})")),
                on_open: move |id: String| msg.set(format!("onOpen({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_lanes_code(_: &Vals) -> String {
    Code::new("AgentLanes")
        .raw("scene", "Some(LaneScene { agents: agents(), tasks: tasks(), now: clock.ms(), span: 30 * 60_000 })")
        .raw("now", "clock.now()")
        .raw("on_approve", "move |id| approve(id)")
        .raw("on_reject", "move |id| reject(id)")
        .raw("on_open", "move |id| open(id)")
        .done()
}

// ---------- Agent Ledger ----------

fn agent_ledger(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let (at, point) = match v.s("state") {
        "settled" => (Some(9_000), None),
        "pointed" => (Some(9_000), Some("price".to_string())),
        "undone" => (Some(19_000), None),
        _ => (None, None),
    };
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "640px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentLedger { now, at, point_at: point, style: cstyle(v),
                on_undo: move |ids: Vec<String>| msg.set(format!("onUndo({ids:?})")),
                on_redo: move |ids: Vec<String>| msg.set(format!("onRedo({ids:?})")) }
            {status(msg)}
        }
    }
}

fn agent_ledger_code(_: &Vals) -> String {
    Code::new("AgentLedger")
        .raw("entries", "Some(changes())")
        .raw("undone", "undone()")
        .raw("agent", "\"Release agent\".into()")
        .raw("now", "clock.now()")
        .raw("clock", "clock.ms()")
        .raw("on_undo", "move |ids| revert(ids)")
        .raw("on_redo", "move |ids| reapply(ids)")
        .done()
}

// ---------- Agent Monitor ----------

fn agent_monitor(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("style"), v.s("state"));
    let (at, failed, folded, open) = match v.s("state") {
        "running" => (Some(5_200), false, false, ""),
        "waiting" => (Some(10_200), false, false, ""),
        "finished" => (Some(19_000), false, false, ""),
        "details" => (Some(19_000), false, false, "build"),
        "failed" => (None, true, false, ""),
        "folded" => (Some(19_000), false, true, ""),
        _ => (None, false, false, ""),
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:340px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentMonitor { now, at, failed, default_folded: folded, open: open.to_string(), style: cstyle(v),
                on_allow: move |_| msg.set("onAllow()".into()),
                on_decline: move |_| msg.set("onDecline()".into()),
                on_pause: move |p: bool| msg.set(if p { "onPause()".into() } else { "onResume()".into() }),
                on_cancel: move |_| msg.set("onCancel()".into()),
                on_retry: move |_| msg.set("onRetry()".into()),
                on_open_log: move |_| msg.set("onOpenLog()".into()) }
            {status(msg)}
        }
    }
}

fn agent_monitor_code(_: &Vals) -> String {
    Code::new("AgentMonitor")
        .raw("scene", "Some(run.scene())")
        .raw("now", "clock.now()")
        .raw("on_allow", "move |_| run.allow()")
        .raw("on_decline", "move |_| run.decline()")
        .raw("on_pause", "move |paused| run.pause(paused)")
        .raw("on_cancel", "move |_| run.cancel()")
        .raw("on_open_log", "move |_| open_trace()")
        .done()
}

// ---------- Agent Queue ----------

fn agent_queue(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let (at, paused, workers, open) = match v.s("state") {
        "limited" => (Some(17_000), false, 4, None),
        "paused" => (Some(9_000), true, 4, None),
        "one" => (Some(12_000), false, 1, None),
        "card" => (Some(17_000), false, 4, Some(1)),
        _ => (None, false, 4, None),
    };
    let width = if v.s("width") == "narrow" {
        "420px"
    } else {
        "756px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentQueue { now, at, paused, workers, open_at: open, style: cstyle(v),
                on_paused_change: move |p: bool| msg.set(format!("onPausedChange({p})")),
                on_concurrency_change: move |n: usize| msg.set(format!("onConcurrencyChange({n})")),
                on_promote: move |id: String| msg.set(format!("onPromote({id:?})")),
                on_cancel: move |id: String| msg.set(format!("onCancel({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_queue_code(_: &Vals) -> String {
    Code::new("AgentQueue")
        .raw("name", "\"agent-work\".into()")
        .raw("workers", "4")
        .raw("now", "clock.now()")
        .raw("on_paused_change", "move |p| queue.pause(p)")
        .raw("on_concurrency_change", "move |n| queue.set_workers(n)")
        .raw("on_promote", "move |id| queue.promote(id)")
        .raw("on_cancel", "move |id| queue.cancel(id)")
        .done()
}

// ---------- Agent Recap ----------

fn agent_recap(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let (replay, acted, point): (u64, Vec<String>, &str) = match v.s("state") {
        "settled" => (0, vec![], ""),
        "pointed" => (0, vec![], "sync"),
        "acted" => (0, vec!["refund".into(), "sync".into(), "prs".into()], ""),
        _ => (1_800, vec![], ""),
    };
    let width = if v.s("width") == "narrow" {
        "400px"
    } else {
        "804px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AgentRecap { now, replay, acted, point: point.to_string(), style: cstyle(v),
                on_action: move |(id, a): (String, String)| msg.set(format!("onAction({id:?}, {a:?})")),
                on_undo: move |id: String| msg.set(format!("onUndo({id:?})")),
                on_redo: move |id: String| msg.set(format!("onRedo({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_recap_code(_: &Vals) -> String {
    Code::new("AgentRecap")
        .raw("events", "Some(night())")
        .raw("since", "left_at_minutes")
        .raw("span", "away_minutes")
        .raw("now", "clock.now()")
        .raw("on_action", "move |(id, which)| act(id, which)")
        .raw("on_undo", "move |id| undo(id)")
        .raw("on_redo", "move |id| redo(id)")
        .done()
}

// ---------- Agent Sequence ----------

fn agent_sequence(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let (at, open, part): (Option<u64>, &str, &str) = match v.s("state") {
        "settled" => (Some(15_000), "", ""),
        "message" => (Some(15_000), "m11", ""),
        "participant" => (Some(15_000), "", "research"),
        _ => (None, "", ""),
    };
    let width = if v.s("width") == "narrow" {
        "420px"
    } else {
        "800px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;margin:0 auto;display:flex;flex-direction:column;gap:8px",
            AgentSequence { now, at, open: open.to_string(), open_participant: part.to_string(), style: cstyle(v),
                on_select: move |id: Option<String>| msg.set(format!("onSelect({id:?})")) }
            {status(msg)}
        }
    }
}

fn agent_sequence_code(_: &Vals) -> String {
    Code::new("AgentSequence")
        .raw("participants", "Some(run.participants())")
        .raw("messages", "Some(run.messages())")
        .raw("title", "run.title()")
        .raw("running", "run.is_running()")
        .raw("now", "clock.now()")
        .raw("on_select", "move |id| inspect(id)")
        .done()
}

/// Family B demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
        Demo {
            controls: vec![
                style(),
                sel(
                    "project",
                    "Project",
                    &["checkout", "launch", "postgres"],
                    "checkout",
                ),
                sel("run", "Run", &["live", "settled"], "live"),
                tog("open", "Lead details open", false),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: a lead and three sub-agents with their helpers; handoffs run along the connectors and light up the card they reach, contexts fill, and the lead ends up asking you to merge.",
                    &[],
                ),
                var(
                    "working-details",
                    "The lead's details while it works: task, model, usage, tools, last handoff and Message / Pause / Stop.",
                    &[("open", "true")],
                ),
                var(
                    "needs-you",
                    "The run settled with the lead waiting: \"1 needs you\" in the header and Message / Decline / Approve in its details.",
                    &[("run", "settled"), ("open", "true")],
                ),
                var(
                    "launch",
                    "Another project from the switcher: the launch post crew (Editor, Researcher, Writer with its Fact checker, SEO).",
                    &[("project", "launch")],
                ),
                var(
                    "narrow",
                    "Narrow: every level indented, the cards as wide as there is room.",
                    &[("width", "narrow"), ("run", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: elevated cards, a grey footer band and filled badges.",
                    &[("style", "classic"), ("run", "settled")],
                ),
            ],
            render: agent_crew,
            code: agent_crew_code,
            keyboard: vec![
                (
                    "Tab",
                    "Into the crew – one stop for all cards – then the project switcher",
                ),
                ("← → ↑ ↓", "Walk the cards"),
                ("Home / End", "The lead, the last card"),
                ("Enter / Space", "Open or close the card's details"),
                (
                    "Esc",
                    "Close the message field, then the details – focus goes back to the card",
                ),
                (
                    "↑ ↓ Enter",
                    "In the project switcher: pick a project; Esc closes it",
                ),
            ],
            a11y: vec![
                "A tree; every card is named with its status, activity and context – \"Coder: Working, Editing basket.ts, context 38%\" – and says it opens a dialog.",
                "The details are a dialog named after the agent; close, steering buttons and the message field are reachable by Tab.",
                "The project switcher is a button with a listbox; the current project is marked as selected.",
                "A polite live region reads each handoff and each steering action – \"Lead handed to Coder: Handle empty baskets\", \"Paused Coder\".",
            ],
            responsive: vec![
                "Wide (760 px): sub-agents side by side under the lead; narrower: every level indented.",
                "Rows are fixed, so a changing activity or context never moves a card or a connector.",
                "Long names, activities and tasks end in … with the full text on hover.",
            ],
            height: 600,
            live: true,
            ..mk(
                "agent-crew",
                "Agent Crew",
                "tools",
                "A lead and its sub-agents as one live tree of cards – what each does now, how full its context is, its tools and the last thing it was handed; handoffs run as a light along the connectors, a click opens the details with steering, and a switcher swaps projects.",
                "AgentCrew",
                "AgentCrewEl",
                "agent-crew",
                "parity/crewfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("tab", "Tab", &["doc", "table"], "doc"),
                sel("run", "Run", &["live", "writing", "settled"], "live"),
                sel("follow", "Follow", &["off", "on"], "off"),
                tog("open", "Suggestion window open", false),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: four agents arrive in the doc, read, select, type suggestions letter by letter, resize the hero image, fix the code and leave comment pins; the costs and the count in the header follow along.",
                    &[],
                ),
                var(
                    "writing",
                    "Frozen while the Researcher writes its comment: the open window beside the paragraph, the Writer's rewrite just typed.",
                    &[("run", "writing")],
                ),
                var(
                    "suggestion-window",
                    "A suggestion's window open: who suggests it, why, Reject and Accept.",
                    &[("run", "settled"), ("open", "true")],
                ),
                var(
                    "following",
                    "Following the Researcher: its face filled, every other pointer and selection faded back.",
                    &[("follow", "on")],
                ),
                var(
                    "table",
                    "Tables and tabs: the Table tab – status pills, an owner and a date being changed, a task row typed in.",
                    &[("tab", "table")],
                ),
                var(
                    "classic",
                    "Classic style: the page as raised paper on a grey header band.",
                    &[("style", "classic"), ("run", "settled")],
                ),
            ],
            render: agent_cursors,
            code: agent_cursors_code,
            keyboard: vec![
                (
                    "Tab",
                    "The tabs, the faces in the header, then from suggestion to suggestion – a focused suggestion opens its window",
                ),
                (
                    "Enter / Space",
                    "On a face: follow that agent, or stop following",
                ),
                ("Esc", "Dismisses an open suggestion"),
            ],
            a11y: vec![
                "The text is real text in headings, paragraphs, a list and a table – pointers, carets and selections are decoration, hidden from screen readers.",
                "Every suggestion is focusable and labelled – \"Researcher suggests removing: close to 30%\" – and its window is a dialog with Reject and Accept.",
                "The faces are toggle buttons (aria-pressed) named after the agent and what it does; a followed face fills with its tint.",
                "A polite live region reads what agents do – \"Coder is writing a comment\", new suggestions with their note.",
            ],
            responsive: vec![
                "Pointers are anchored on their characters, so they stay on their letters while the text wraps at any width, font size or zoom.",
                "Flags sit to the bottom right of their pointer; near the right edge they are cut off rather than moved.",
                "Below 520 px the page gets tighter margins and the suggestion count gives way to the faces.",
            ],
            height: 600,
            live: true,
            ..mk(
                "agent-cursors",
                "Agent Cursors",
                "tools",
                "Agents in a document like teammates in a multiplayer editor – live pointers with names and costs, suggestions typed letter by letter, an image they resize, code they fix and comment pins beside the page.",
                "AgentCursors",
                "AgentCursorsEl",
                "agent-cursors",
                "parity/cursorsfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("run", "Run", &["live", "finished"], "live"),
                sel("filter", "Only", &["all", "auditor", "fuzzer"], "all"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: four agents check in parallel, the bars fill and findings slide into the feed.",
                    &[],
                ),
                var(
                    "finished",
                    "Every agent done: the counters in red and amber.",
                    &[("run", "finished")],
                ),
                var(
                    "filtered",
                    "One agent pressed: only its findings, the others faded, an \"Auditor ×\" chip.",
                    &[("run", "finished"), ("filter", "auditor")],
                ),
                var(
                    "classic",
                    "Classic style: a softer card and the feed in a grey band.",
                    &[("style", "classic"), ("run", "finished")],
                ),
            ],
            render: agent_findings,
            code: agent_findings_code,
            keyboard: vec![
                ("Tab", "The agents, then the findings in the feed"),
                (
                    "Enter / Space",
                    "On an agent: only its findings (again: all); on a finding: open it",
                ),
            ],
            a11y: vec![
                "The feed is a polite live region: new findings are announced as they arrive.",
                "Every severity has a text label for screen readers, not just a colour.",
                "Agents are toggle buttons (aria-pressed) for the filter.",
            ],
            responsive: vec![
                "Below 480 px the check counts and locations drop so the rows keep one line.",
            ],
            height: 470,
            live: true,
            ..mk(
                "agent-findings",
                "Agent Findings",
                "tools",
                "A swarm of agents checking something – one calm row per agent with its progress, and their findings sliding into a short feed as they're found.",
                "AgentFindings",
                "AgentFindingsEl",
                "agent-findings",
                "parity/agentsfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("run", "Run", &["live", "early", "needs-you"], "live"),
                sel("open", "Card", &["none", "lead", "writer"], "none"),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: the lead in the middle, four agents around it and seven tools plus you on the outer orbit; every call runs a pulse out and its answer back, agents breathe, finish, fail and wait for you.",
                    &[],
                ),
                var(
                    "card",
                    "The lead's card: status, what it does, its latest calls with spinners and times, its cost and calls in the last minute.",
                    &[("run", "early"), ("open", "lead")],
                ),
                var(
                    "needs-you",
                    "Settled with the Writer waiting (amber, \"Writer needs you\") and the Coder failed (red, \"Coder failed\"); two agents done.",
                    &[("run", "needs-you"), ("open", "writer")],
                ),
                var(
                    "narrow",
                    "Narrow: taller than wide, smaller nodes, short labels, no model line; the card docks to the bottom.",
                    &[("width", "narrow"), ("run", "needs-you")],
                ),
                var(
                    "classic",
                    "Classic style: blue links that grow thicker with traffic, the bar on a grey band, the \"Busier link\" legend.",
                    &[("style", "classic"), ("run", "early")],
                ),
            ],
            render: agent_graph,
            code: agent_graph_code,
            keyboard: vec![
                (
                    "Tab",
                    "Between the \"needs you\" / \"failed\" chips and the nodes; a focused node lights up its neighbourhood",
                ),
                ("Enter / Space", "Opens or closes the focused node's card"),
                ("Esc", "Closes the card and returns focus to its node"),
            ],
            a11y: vec![
                "Every node is a button named with its name, kind, status, what it does and its calls in the last minute; aria-expanded says whether its card is open.",
                "The card is a dialog; its close button takes focus, Esc closes it.",
                "A visually hidden list gives the same picture as text: every node, its status, whom it called how often and its cost.",
                "The chips sit in a polite live region; status is never colour alone – a badge and a line under the name.",
            ],
            responsive: vec![
                "The layout is computed for a wide and a narrow box with x as a share of the width, so it fits its container.",
                "Below 520 px the graph turns taller than wide, nodes shrink, labels use short names and the card docks to the bottom.",
                "Labels and the outer orbit stay inside the panel – nothing is cut off and nothing scrolls.",
            ],
            height: 600,
            live: true,
            ..mk(
                "agent-graph",
                "Agent Graph",
                "tools",
                "Agents and the tools they call as a living network – a pulse along the link for every call and its answer, agents that breathe while they work and turn amber when they need you, and a card with each node's latest calls.",
                "AgentGraph",
                "AgentGraphEl",
                "agent-graph",
                "parity/graphfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("run", "Run", &["live", "handing", "settled"], "live"),
                sel("open", "Open", &["none", "card", "lost"], "none"),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: the Researcher picks up the next task, hands it to the Writer and on to the Reviewer – a light runs along the lines, the card drops into its column and the costs tick up.",
                    &[],
                ),
                var(
                    "handing-on",
                    "Frozen just after a handoff: the new card on top of the column, \"With Writer\" and the Writer drafting.",
                    &[("run", "handing")],
                ),
                var(
                    "card-open",
                    "A handoff card open in place: everything that went along, and the lost item struck through.",
                    &[("run", "settled"), ("open", "card")],
                ),
                var(
                    "lost-item",
                    "The lost item open: why it got lost, and Add back.",
                    &[("run", "settled"), ("open", "lost")],
                ),
                var(
                    "stacked",
                    "Below 620 px the relay stacks into one column: an agent, its handoff, the next agent.",
                    &[("width", "narrow"), ("run", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: visible borders, crisper corners and the column header in a filled band.",
                    &[("style", "classic"), ("run", "settled")],
                ),
            ],
            render: agent_handoff,
            code: agent_handoff_code,
            keyboard: vec![
                (
                    "Tab",
                    "Through the waiting tasks, the agents (while a task is picked up) and the handed-over items",
                ),
                (
                    "Enter / Space",
                    "Opens a card or an agent's task; on a lost item: shows why, and Add back",
                ),
                ("Esc", "Drops a picked-up card, then closes what is open"),
            ],
            a11y: vec![
                "Tasks can be given to an agent without dragging: double-click a card, then click an agent to its right.",
                "Each column and each handoff's items are labelled lists; cards and lost items are buttons with aria-expanded.",
                "Status is never colour alone: a spinner, a dot or a triangle beside the words.",
            ],
            responsive: vec![
                "Below about 760 px the status shows as an icon only.",
                "Below 620 px the relay stacks into one column – an agent, its handoff, the next agent.",
                "Cards keep their size; the columns fade out at the bottom instead of scrolling.",
            ],
            height: 560,
            live: true,
            ..mk(
                "agent-handoff",
                "Agent Handoff",
                "tools",
                "A task passed from agent to agent like a relay – at every handoff what went along to the next one, what got lost on the way, and a light running down the line as the card drops in.",
                "AgentHandoff",
                "AgentHandoffEl",
                "agent-handoff",
                "parity/handofffid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("view", "Layout", &["board", "list"], "board"),
                tog("open", "First card open", true),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: the board, with the waiting migration open on Reject / Approve; clocks tick and the release notes finish by themselves.",
                    &[],
                ),
                var(
                    "list",
                    "List layout: the same tasks as full-width rows under their headers.",
                    &[("view", "list")],
                ),
                var("closed", "Every card folded.", &[("open", "false")]),
                var(
                    "classic",
                    "Classic style: hairline cards and bold title-case headers.",
                    &[("style", "classic")],
                ),
            ],
            render: agent_inbox,
            code: agent_inbox_code,
            keyboard: vec![
                ("Tab", "The layout switch, then the cards and their buttons"),
                ("Enter / Space", "Unfold or fold a card"),
                ("Esc", "Fold the card"),
            ],
            a11y: vec![
                "Columns are labelled sections; cards are buttons with aria-expanded.",
                "The layout switch is a radiogroup; a status line reads \"2 need you, 2 running\".",
                "Status tags carry words as well as colour.",
            ],
            responsive: vec![
                "Fills its container up to 740 px; below 620 px the board stacks into one column.",
            ],
            height: 620,
            live: true,
            ..mk(
                "agent-inbox",
                "Agent Inbox",
                "tools",
                "All your background agent tasks in one calm list – live clocks and status tags, filter tabs with rolling counts, rows that unfold to the last steps with Approve, Reject, Retry or Open, and glide to their new place.",
                "AgentInbox",
                "AgentInboxEl",
                "agent-inbox",
                "parity/agentsfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("run", "Run", &["live", "settled"], "live"),
                sel("open", "Open", &["none", "summary", "decision"], "none"),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: four agents over the last half hour on a fast clock – running blocks grow into now, a draft hands over to the Reviewer along a dashed curve, the window slides and old blocks fade out at the left.",
                    &[],
                ),
                var(
                    "summary",
                    "A finished block clicked: the dark summary – who, state, how long and what got done.",
                    &[("run", "settled"), ("open", "summary")],
                ),
                var(
                    "decision",
                    "The waiting block's decision: the ask, Reject and Approve (Approve turns it into a running block).",
                    &[("run", "settled"), ("open", "decision")],
                ),
                var(
                    "narrow",
                    "Below 520 px the names column shrinks to the avatars.",
                    &[("width", "narrow"), ("run", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: outlined blocks, round avatar tiles, a clearer grid and a filled bar.",
                    &[("style", "classic"), ("run", "settled")],
                ),
            ],
            render: agent_lanes,
            code: agent_lanes_code,
            keyboard: vec![
                ("Tab", "From block to block, lane by lane"),
                (
                    "Enter",
                    "Opens a block's summary, or the decision on a waiting one",
                ),
                ("Esc", "Closes the decision or the summary"),
            ],
            a11y: vec![
                "Every lane is a list labelled with the agent's name; every block is a button naming the agent, task, state and time.",
                "A waiting block controls its decision (aria-expanded), a labelled dialog with Approve focused when you open it.",
                "How many tasks need someone is announced politely when it changes.",
            ],
            responsive: vec![
                "Fills its container up to 760 px.",
                "Below 520 px the names column shrinks to the avatars and every other tick label drops.",
            ],
            height: 420,
            live: true,
            ..mk(
                "agent-lanes",
                "Agent Lanes",
                "tools",
                "Where every agent is right now – a live timeline with a lane per agent, tasks growing into now, hand-offs drawn between agents and approvals right on the block.",
                "AgentLanes",
                "AgentLanesEl",
                "agent-lanes",
                "parity/lanesfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["live", "settled", "pointed", "undone"],
                    "live",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: changes land at the top one by one, then the price change's Undo is pointed at and taken – the coupon and the deals go with it, the sent email can't.",
                    &[],
                ),
                var(
                    "pointed",
                    "Pointing at Undo: the change and what builds on it light up – Also undone in blue, Can't undo in amber.",
                    &[("state", "pointed")],
                ),
                var(
                    "undone",
                    "After the undo: the chain struck through, logos grey, Undone in the slot and Redo on hover; the header counts what was undone.",
                    &[("state", "undone")],
                ),
                var(
                    "narrow",
                    "Below 480 px the system name drops and Undo / Redo become round icon buttons.",
                    &[("width", "narrow"), ("state", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: the header in its own band and the list a raised card.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: agent_ledger,
            code: agent_ledger_code,
            keyboard: vec![
                (
                    "Tab",
                    "Moves through Undo and Redo; focusing Undo shows what goes with it",
                ),
                ("Enter / Space", "Undoes or redoes the focused change"),
            ],
            a11y: vec![
                "An ordered list named after the ledger, latest entry on top.",
                "Undo is named with the change and how many build on it; Redo with the change.",
                "Before → after is plain text and the arrow is read as \"to\"; what was undone is announced.",
            ],
            responsive: vec![
                "640 px up to its container.",
                "Below 480 px the system name drops and Undo / Redo shrink to round icon buttons.",
                "On touch screens Undo and Redo show on every row in place of the time.",
            ],
            height: 600,
            live: true,
            ..mk(
                "agent-ledger",
                "Agent Ledger",
                "tools",
                "Everything an agent changed in your systems, each change with an undo – before and after per row, and pointing at Undo shows what would go with it and what can't be taken back.",
                "AgentLedger",
                "AgentLedgerEl",
                "agent-ledger",
                "parity/ledgerfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &[
                        "live", "running", "waiting", "finished", "details", "failed", "folded",
                    ],
                    "live",
                ),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: steps roll through the well with a clock and the bill so far, finished ones drop into the list, the card stops to ask before publishing (Enter allows, Esc declines), then finishes green.",
                    &[],
                ),
                var(
                    "waiting",
                    "Needs you: the exact command in the well, Decline and Allow in the foot.",
                    &[("state", "waiting")],
                ),
                var(
                    "details",
                    "A finished step clicked: when it started, how long it took, its share of the run and what it touched.",
                    &[("state", "details")],
                ),
                var(
                    "failed",
                    "Failed: the step in red, what went wrong and Try again.",
                    &[("state", "failed")],
                ),
                var(
                    "folded",
                    "Folded into the small bar with the tools it used and how long it took.",
                    &[("state", "folded")],
                ),
                var(
                    "classic",
                    "Classic style: rounder card, bordered tiles and the controls on a tray.",
                    &[("style", "classic"), ("state", "running")],
                ),
            ],
            render: agent_monitor,
            code: agent_monitor_code,
            keyboard: vec![
                (
                    "Tab",
                    "Fold away, the finished steps, then the controls in the foot",
                ),
                ("Enter", "While it waits and the card has focus: Allow"),
                (
                    "Esc",
                    "While it waits: Decline; with step details open, closes them",
                ),
                (
                    "Enter / Space",
                    "On a step: opens its details (focus moves to Close)",
                ),
            ],
            a11y: vec![
                "The card is a region named with the agent, its state and the current step; folded, the bar is one button with the same name.",
                "The time strip is one image labelled with every step and its time; finished steps are a list of buttons with aria-expanded.",
                "A step's details are a labelled dialog; the ask is announced when the card starts waiting.",
            ],
            responsive: vec![
                "A 340 px card that shrinks with its container; labels and the command ellipsize.",
            ],
            height: 480,
            live: true,
            ..mk(
                "agent-monitor",
                "Agent Monitor",
                "tools",
                "One agent at work as a card – the current step with a clock and the bill so far, a time strip, the finished steps with each tool's logo, the exact command when it needs an OK, and a fold into a small bar.",
                "AgentMonitor",
                "AgentMonitorEl",
                "agent-monitor",
                "parity/monitorfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["live", "limited", "paused", "one", "card"],
                    "live",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: jobs drop into the queue, glide onto free workers as growing bars and land in Done or Failed; every pass a rate limit stops the workers, the backlog climbs past the backpressure line and drains after.",
                    &[],
                ),
                var(
                    "limited",
                    "A rate limit with its countdown: running jobs paused, Waiting amber above the line, an amber stretch on the chart.",
                    &[("state", "limited")],
                ),
                var(
                    "paused",
                    "Paused by hand: a grey pill, Resume, the workers waiting.",
                    &[("state", "paused")],
                ),
                var(
                    "one-worker",
                    "One worker: the queue grows and the rest show as a grid of dots by kind.",
                    &[("state", "one")],
                ),
                var(
                    "card",
                    "A job's card: how long it waited, its place, priority and attempt, with Cancel and Move to front.",
                    &[("state", "card")],
                ),
                var(
                    "narrow",
                    "Under 600 px the flow stacks: queue, then workers (marks only), then Done and Failed side by side.",
                    &[("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: mixed-case labels, the numbers and the chart in inset panels.",
                    &[("style", "classic"), ("state", "limited")],
                ),
            ],
            render: agent_queue,
            code: agent_queue_code,
            keyboard: vec![
                (
                    "Tab",
                    "Into the jobs (the front job is the one stop), then the card's buttons",
                ),
                ("Enter / Space", "Opens or closes the focused job's card"),
                ("Esc", "Closes the card"),
            ],
            a11y: vec![
                "Every job is a button named with its kind, how long it waited or ran, its place in line, priority and attempt.",
                "Pause is a toggle with aria-pressed; the stepper's buttons say what they do and the count is announced.",
                "The chart is an image summarised in words; rate limits, pauses and crossing the backpressure line are announced politely.",
            ],
            responsive: vec![
                "756 px up to its container.",
                "Under 600 px the flow stacks and the four numbers wrap to two columns; under 440 px Pause and the stepper get their own row.",
            ],
            height: 640,
            live: true,
            ..mk(
                "agent-queue",
                "Agent Queue",
                "tools",
                "The work waiting for your agents and how fast it moves – a live queue, workers running jobs as growing bars, Done and Failed, and a depth chart that shows the backlog build up under a rate limit and drain after.",
                "AgentQueue",
                "AgentQueueEl",
                "agent-queue",
                "parity/queuefid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["replay", "settled", "pointed", "acted"],
                    "replay",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: the night replays – the line draws itself, a dot pops onto each peak and its card fades in as the line passes; Approve, Merge and Retry work for a moment, then settle with Undo.",
                    &[],
                ),
                var(
                    "pointed",
                    "Pointing at an event: a crosshair, the dot grows and a dark tooltip with its time; the card lights up too.",
                    &[("state", "pointed")],
                ),
                var(
                    "acted",
                    "After acting: the refund approved, the sync fixed, and one done card undone with Redo.",
                    &[("state", "acted")],
                ),
                var(
                    "narrow",
                    "Under 640 px the three lanes stack.",
                    &[("width", "narrow"), ("state", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: the chart in its own panel, lanes without a background, solid primary buttons.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: agent_recap,
            code: agent_recap_code,
            keyboard: vec![
                (
                    "Tab",
                    "Through each card's buttons, lane by lane; focusing a card lights its dot",
                ),
                ("Enter / Space", "Runs the focused action, Undo or Redo"),
            ],
            a11y: vec![
                "Each lane is a labelled section with its cards as a list; the chart is decorative and the summary says the counts in words.",
                "While an action works its buttons are disabled and the row says Working…; the result and every undo are announced.",
                "State is never colour alone: lanes are named, tags say Done, Approved, Fixed or Undone.",
            ],
            responsive: vec![
                "804 px up to its container.",
                "Under 640 px the three lanes stack and stop reserving room for three cards.",
            ],
            height: 560,
            live: true,
            ..mk(
                "agent-recap",
                "Agent Recap",
                "tools",
                "A catch-up card for when you come back: a tiny timeline of the night with a spike per event, then grouped items – waiting on you, broken, finished – each with its action in place.",
                "AgentRecap",
                "AgentRecapEl",
                "agent-recap",
                "parity/recapfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["live", "settled", "message", "participant"],
                    "live",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "live",
                    "As affectUI ships it: the run plays out message by message – each arrow draws across with its tag, bars grow on the lifelines while participants work, the parallel and retried messages gather in labelled frames, then it loops.",
                    &[],
                ),
                var(
                    "settled",
                    "The finished run: fifteen messages, one failure, every bar closed.",
                    &[("state", "settled")],
                ),
                var(
                    "message-card",
                    "A message's card: sender to receiver, the label, Call or Reply, its frame, what went along and how long the answer took.",
                    &[("state", "message")],
                ),
                var(
                    "participant-card",
                    "A participant's card: kind, Working or Idle, the model, what it does, and messages sent and received, tasks and busy time.",
                    &[("state", "participant")],
                ),
                var(
                    "narrow",
                    "Under 560 px the chips drop to their marks and the time gutter narrows.",
                    &[("width", "narrow"), ("state", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: a white card, pill chips and tags, tinted frames.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: agent_sequence,
            code: agent_sequence_code,
            keyboard: vec![
                (
                    "Tab",
                    "To the participant chips, then into the message list",
                ),
                (
                    "Enter / Space",
                    "Opens the focused participant's or message's card",
                ),
                ("Escape", "Closes the open card"),
            ],
            a11y: vec![
                "Messages are an ordered list; each is named \"Sender to Receiver: text, status, m:ss\" and says it opens a dialog.",
                "Participants are buttons; cards are dialogs with a close button you can reach by Tab.",
                "Colour never carries meaning alone: replies are dashed, failures say so and carry a cross, frames are labelled.",
            ],
            responsive: vec![
                "800 px up to its container; the columns share the width.",
                "Under 560 px the chips show only their marks and the time gutter and times shrink.",
            ],
            height: 720,
            live: true,
            align: Align::Start,
            ..mk(
                "agent-sequence",
                "Agent Sequence",
                "tools",
                "One agent run as a sequence diagram – a column for you, each agent and each tool, every message an arrow between lifelines, calls solid and replies dashed, bars while a participant works, and parallel or retried messages in labelled frames.",
                "AgentSequence",
                "AgentSequenceEl",
                "agent-sequence",
                "parity/seqfid.rs",
            )
        },
    ]
}
