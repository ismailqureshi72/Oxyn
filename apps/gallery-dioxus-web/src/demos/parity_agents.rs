//! Plan #32 (affectUI fidelity pass), overnight batch family B demos: the exact Agent Findings and
//! Agent Inbox, each with every affectUI style and state; part 2: Agent Crew; part 3: Agent Cursors.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{AgentCrew, AgentCursors, AgentFindings, AgentInbox};

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
                ("Esc", "Closes the window of a suggestion"),
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
