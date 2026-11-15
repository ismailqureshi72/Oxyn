//! Plan #32 (affectUI fidelity pass), overnight batch family B demos: the exact Agent Findings and
//! Agent Inbox, each with every affectUI style and state.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{AgentFindings, AgentInbox};

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

/// Family B demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
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
