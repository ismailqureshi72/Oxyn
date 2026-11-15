//! Plan #32 (affectUI fidelity pass), overnight batch family B demos: the exact Agent Findings and
//! Agent Inbox, each with every affectUI style and state; part 2: Agent Crew.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{AgentCrew, AgentFindings, AgentInbox};

use super::{Code, mk, sel, tog, var};
use crate::{Demo, Vals};

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
                "A tree; every card is named with its status, activity and context – \"Coder: Working, Editing cart.ts, context 38%\" – and says it opens a dialog.",
                "The details are a dialog named after the agent; close, steering buttons and the message field are reachable by Tab.",
                "The project switcher is a button with a listbox; the current project is marked as selected.",
                "A polite live region reads each handoff and each steering action – \"Lead handed to Coder: Guard empty carts\", \"Paused Coder\".",
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
    ]
}
