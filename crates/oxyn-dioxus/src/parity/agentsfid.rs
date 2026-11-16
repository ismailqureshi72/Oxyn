//! affectUI fidelity pass, overnight batch, family B (plan #32): exact rebuilds of affectUI's
//! **Agent Findings** ([`AgentFindings`]) and **Agent Inbox** ([`AgentInbox`]). Behaviour and look
//! only; models in `oxyn_core::parity::agentsfid`. Styles: `agentsfid.css` (classes `oxfd-*`,
//! `oxib-*`), in `PRIMITIVES_CSS`. Both follow the nearest `data-cstyle` (Clean / Classic) unless
//! `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::agentsfid::{
        AgentFinding, FindingAgent, INBOX_COLUMNS, InboxTask, TaskState, age, clock, demo_findings,
        demo_inbox, findings_at, inbox_summary, severity_counts,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn ms(now: ClockMs, then: ClockMs) -> u64 {
    now.since(then).as_millis() as u64
}

/// The four-point sparkle affectUI marks agents and columns with.
#[component]
fn Sparkle(#[props(default = 14.0)] size: f32) -> Element {
    rsx! {
        svg { class: "oxfd-spk", width: "{size}", height: "{size}", view_box: "0 0 24 24", "aria-hidden": "true",
            path { d: "M12 2c.7 5.3 3.3 8 9 10-5.7 2-8.3 4.7-9 10-.7-5.3-3.3-8-9-10 5.7-2 8.3-4.7 9-10z", fill: "currentColor" }
        }
    }
}

/// A 2×2 severity mark.
#[component]
fn SevMark(sev: String) -> Element {
    rsx! {
        svg { class: "oxfd-sev-i", "data-sev": "{sev}", width: "11", height: "11", view_box: "0 0 11 11", "aria-hidden": "true",
            rect { x: "0", y: "0", width: "4.5", height: "4.5", rx: "1" }
            rect { x: "6.5", y: "0", width: "4.5", height: "4.5", rx: "1", opacity: ".55" }
            rect { x: "0", y: "6.5", width: "4.5", height: "4.5", rx: "1", opacity: ".55" }
            rect { x: "6.5", y: "6.5", width: "4.5", height: "4.5", rx: "1" }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Agent Findings ──

/// Agent Findings (affectUI): a swarm of agents checking something – one calm row per agent with
/// its coloured model mark, current step, a blue progress bar, its check count and critical /
/// warning counters, and their findings sliding into a short "Latest findings" feed (severity
/// mark, title, agent · location, age). The bar counts every check and the criticals and warnings
/// (red and amber once there are any). Pressing an agent shows only its findings (the others fade,
/// a "name ×" chip appears); pressing a finding selects it. Clean: one card, an uppercase feed
/// label; Classic: a softer card with the feed in a grey band and a title-case label.
#[component]
pub fn AgentFindings(
    /// Agents (the demo when empty).
    #[props(default)]
    agents: Vec<FindingAgent>,
    /// Findings with the time each is found (the demo when empty).
    #[props(default)]
    findings: Vec<AgentFinding>,
    /// Heading in the bar.
    #[props(default = "Verification".to_string())]
    title: String,
    /// Findings the feed shows at most.
    #[props(default = 4)]
    rows: usize,
    /// Show the run finished from the start.
    #[props(default)]
    finished: bool,
    /// Start over a few seconds after every agent is done.
    #[props(default = true)]
    looped: bool,
    /// Agent filter at the start.
    #[props(default)]
    filter: Option<usize>,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A finding was opened (its id).
    #[props(default)]
    on_open: EventHandler<String>,
) -> Element {
    let (da, df) = use_hook(demo_findings);
    let agents = if agents.is_empty() { da } else { agents };
    let findings = if findings.is_empty() { df } else { findings };
    let start = use_signal(move || now);
    let mut only = use_signal(move || filter);
    let mut sel = use_signal(|| None::<String>);
    let end = agents
        .iter()
        .map(|a| a.ms_per_check * a.total as u64)
        .max()
        .unwrap_or(0)
        .max(findings.iter().map(|f| f.at_ms).max().unwrap_or(0));
    let mut el = if finished { end } else { ms(now, start()) };
    if looped && !finished {
        el %= end + 5000;
    }
    let found = findings_at(&findings, el);
    let done: u32 = agents.iter().map(|a| a.done_at(el)).sum();
    let total: u32 = agents.iter().map(|a| a.total).sum();
    let (crit, warn) = severity_counts(&found, None);
    let feed: Vec<AgentFinding> = found
        .iter()
        .filter(|f| only().is_none_or(|a| f.agent == a))
        .take(rows)
        .cloned()
        .collect();
    let names: Vec<String> = agents.iter().map(|a| a.name.clone()).collect();
    let agent_rows: Vec<(usize, FindingAgent, u32, usize, usize, f32, bool)> = agents
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, a)| {
            let d = a.done_at(el);
            let (c, w) = severity_counts(&found, Some(i));
            let pct = d as f32 * 100.0 / a.total.max(1) as f32;
            (i, a, d, c, w, pct, only() == Some(i))
        })
        .collect();
    rsx! {
        div { class: "oxfd", "data-style": style_attr(&style), "data-filter": "{only().is_some()}",
            div { class: "oxfd-card",
                div { class: "oxfd-hd",
                    span { class: "oxfd-hi", Sparkle { size: 13.0 } }
                    b { "{title}" }
                    span { class: "oxfd-hc", "{done} of {total} checks" }
                    span { class: "oxfd-sp" }
                    span { class: "oxfd-tot", "data-sev": "critical", "data-on": "{crit > 0}", i {} "{crit} critical" }
                    span { class: "oxfd-tot", "data-sev": "warning", "data-on": "{warn > 0}", i {} "{warn} warning" if warn != 1 { "s" } }
                }
                div { class: "oxfd-agents", role: "group", "aria-label": "Agents",
                    for (i, a, d, c, w, pct, pressed) in agent_rows.into_iter() {
                                button { key: "{i}", class: "oxfd-row", r#type: "button", "aria-pressed": "{pressed}", "data-dim": "{only().is_some() && !pressed}",
                                    title: "{a.name} · {a.model}",
                                    "aria-label": if pressed { format!("Show all agents (now only {})", a.name) } else { format!("{}: {} of {} checks, {} critical, {} warnings. Show only its findings", a.name, d, a.total, c, w) },
                                    onclick: move |_| only.set(if only() == Some(i) { None } else { Some(i) }),
                                    span { class: "oxfd-tile", BrandIcon { name: a.brand.clone(), size: 14.0, coloured: true, fallback: true } }
                                    span { class: "oxfd-nm", b { "{a.name}" } span { key: "{d}", "{a.step_at(d)}" } }
                                    span { class: "oxfd-bar", "data-done": "{d >= a.total}", i { style: "width:{pct:.1}%" } }
                                    span { class: "oxfd-ct", "{d}/{a.total}" }
                                    span { class: "oxfd-mini", "data-sev": "critical", "data-on": "{c > 0}", SevMark { sev: "critical" } "{c}" }
                                    span { class: "oxfd-mini", "data-sev": "warning", "data-on": "{w > 0}", SevMark { sev: "warning" } "{w}" }
                                }
                    }
                }
                div { class: "oxfd-feed",
                    div { class: "oxfd-fl",
                        span { class: "oxfd-fll", "Latest findings" }
                        if let Some(a) = only() {
                            button { class: "oxfd-chip", r#type: "button", "aria-label": "Show all agents", onclick: move |_| only.set(None),
                                "{names.get(a).cloned().unwrap_or_default()}" Icon { name: "x", size: 11.0, stroke: 2.0 } }
                        }
                    }
                    ul { class: "oxfd-list", "aria-live": "polite", "aria-label": "Latest findings", style: "--rows:{rows}",
                        if feed.is_empty() { li { class: "oxfd-none", "Nothing found yet" } }
                        for f in feed.into_iter() {
                            li { key: "{f.id}", class: "oxfd-li",
                                button { class: "oxfd-f", r#type: "button", "data-sel": "{sel() == Some(f.id.clone())}",
                                    onclick: { let id = f.id.clone(); move |_| { sel.set(Some(id.clone())); on_open.call(id.clone()) } },
                                    SevMark { sev: f.severity.id().to_string() }
                                    span { class: "oxfd-sr", "{f.severity.label()}: " }
                                    span { class: "oxfd-ft", "{f.title}" }
                                    span { class: "oxfd-fm", "{agents.get(f.agent).map(|a| a.name.clone()).unwrap_or_default()}" span { class: "oxfd-loc", " · " code { "{f.location}" } } }
                                    span { class: "oxfd-sp" }
                                    span { class: "oxfd-age", "{age(el.saturating_sub(f.at_ms))}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────── Agent Inbox ──

/// (state, clock seconds, steps added since the start).
type TaskView = (TaskState, u64, Vec<(String, String)>);
/// (column, its name, its tasks with their indices).
type Column<'a> = (usize, &'a str, Vec<(usize, InboxTask)>);

#[derive(Clone, PartialEq)]
struct TaskNow {
    state: TaskState,
    since: ClockMs,
    base_s: u64,
    extra: Vec<(String, String)>,
}

/// Agent Inbox (affectUI): every background agent task in one calm board – Needs you, Running and
/// Done columns with rolling counts – or a list. Cards carry a coloured sparkle, the title, two
/// lines of description and a status tag with a live clock (Waiting for you, a blue running ring,
/// a green Done, a red Failed) with tool and source counts; a card unfolds in place to its model,
/// last steps and Approve / Reject, Retry or Open, and glides to its new column. Clean: cards with
/// soft shadows under grey column bars; Classic: hairline cards and bold title-case headers.
#[component]
pub fn AgentInbox(
    /// Tasks (the demo when empty).
    #[props(default)]
    tasks: Vec<InboxTask>,
    /// Heading after "Agents /".
    #[props(default = "Agent inbox".to_string())]
    title: String,
    /// `board` or `list`.
    #[props(default = "board".to_string())]
    view: String,
    /// The card open at the start.
    #[props(default)]
    open: String,
    /// Show an Open button on running and done tasks.
    #[props(default = true)]
    openable: bool,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// (`approve` | `reject` | `retry` | `open` | `view`, task id or view).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let tasks = if tasks.is_empty() {
        demo_inbox()
    } else {
        tasks
    };
    let first = tasks.clone();
    let ts = use_signal(move || first.clone());
    let mut live = use_signal(move || {
        tasks
            .iter()
            .map(|t| TaskNow {
                state: t.state,
                since: now,
                base_s: t.elapsed_s,
                extra: vec![],
            })
            .collect::<Vec<_>>()
    });
    let mut layout = use_signal(move || view.clone());
    let mut expanded = use_signal(move || {
        if open.is_empty() {
            None
        } else {
            Some(open.clone())
        }
    });
    let all = ts();
    // Derived state: a running task with a finish time is done once it has run that long.
    let cur: Vec<TaskView> = live()
        .iter()
        .zip(all.iter())
        .map(|(l, t)| {
            let run = ms(now, l.since) / 1000;
            match l.state {
                TaskState::Running => match t.finishes_after_s {
                    Some(f) if l.extra.is_empty() && run >= f => {
                        let mut ex = l.extra.clone();
                        ex.push(("Finished – ready to read".into(), "now".into()));
                        (TaskState::Done, l.base_s + f, ex)
                    }
                    _ => (TaskState::Running, l.base_s + run, l.extra.clone()),
                },
                s => (s, l.base_s, l.extra.clone()),
            }
        })
        .collect();
    let states: Vec<TaskState> = cur.iter().map(|c| c.0).collect();
    let summary = inbox_summary(&states);
    let board = layout() == "board";
    let mut act = move |i: usize, what: &str| {
        let id = ts.peek()[i].id.clone();
        let mut l = live.write();
        let t = &mut l[i];
        match what {
            "approve" => {
                t.state = TaskState::Running;
                t.since = now;
                t.extra.push((
                    "Approved by you – running on production".into(),
                    "now".into(),
                ));
            }
            "reject" => {
                t.state = TaskState::Failed;
                t.extra.push(("Rejected by you".into(), "now".into()));
            }
            "retry" => {
                t.state = TaskState::Running;
                t.since = now;
                t.base_s = 0;
                t.extra
                    .push(("Retrying with a slower pace".into(), "now".into()));
            }
            _ => {}
        }
        on_action.call((what.to_string(), id));
    };
    let card = move |i: usize, t: InboxTask, list: bool| -> Element {
        let (st, secs, extra) = cur[i].clone();
        let is_open = expanded() == Some(t.id.clone());
        let tag = match st {
            TaskState::Waiting => format!("Waiting for you · {}", clock(secs)),
            TaskState::Running => {
                if list {
                    format!("Running · {}", clock(secs))
                } else {
                    clock(secs)
                }
            }
            TaskState::Done => {
                if list {
                    format!("Done in {}", clock(secs))
                } else {
                    clock(secs)
                }
            }
            TaskState::Failed => "Failed".to_string(),
        };
        let id = t.id.clone();
        rsx! {
            div { key: "{t.id}", class: "oxib-card", "data-state": "{st.id()}", "data-open": "{is_open}",
                button { class: "oxib-head", r#type: "button", "aria-expanded": "{is_open}",
                    onclick: move |_| expanded.set(if expanded() == Some(id.clone()) { None } else { Some(id.clone()) }),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { expanded.set(None) },
                    span { class: "oxib-tt", Sparkle { size: 13.0 } b { "{t.title}" } }
                    span { class: "oxib-desc", "{t.description}" }
                    span { class: "oxib-meta",
                        span { class: "oxib-tag", "data-state": "{st.id()}",
                            match st {
                                TaskState::Running => rsx! { span { class: "oxib-ring" } },
                                TaskState::Done => rsx! { span { class: "oxib-dot" } },
                                TaskState::Failed => rsx! { span { class: "oxib-bang", "!" } },
                                TaskState::Waiting => rsx! {},
                            }
                            span { key: "{secs}", "{tag}" }
                        }
                        span { class: "oxib-sp" }
                        span { class: "oxib-n", title: "{t.tools} tools", Icon { name: "search", size: 12.0 } "{t.tools}" }
                        span { class: "oxib-div" }
                        span { class: "oxib-n", title: "{t.sources} sources", Icon { name: "link", size: 12.0 } "{t.sources}" }
                    }
                }
                if is_open {
                    div { class: "oxib-more",
                        span { class: "oxib-agent", "{t.agent}" }
                        ol { class: "oxib-steps",
                            for (k, (l, w)) in t.steps.iter().chain(extra.iter()).cloned().enumerate() {
                                li { key: "{k}", span { class: "oxib-sl", "{l}" } span { class: "oxib-sw", "{w}" } }
                            }
                        }
                        div { class: "oxib-acts",
                            match st {
                                TaskState::Waiting => rsx! {
                                    button { class: "oxib-btn ghost", r#type: "button", onclick: move |_| act(i, "reject"), "Reject" }
                                    button { class: "oxib-btn solid", r#type: "button", onclick: move |_| act(i, "approve"), "Approve" }
                                },
                                TaskState::Failed => rsx! { button { class: "oxib-btn solid", r#type: "button", onclick: move |_| act(i, "retry"), "Retry" } },
                                _ => rsx! { if openable { button { class: "oxib-btn", r#type: "button", onclick: move |_| act(i, "open"), "Open" } } },
                            }
                        }
                    }
                }
            }
        }
    };
    let cols: Vec<Column> = INBOX_COLUMNS
        .iter()
        .enumerate()
        .map(|(c, name)| {
            (
                c,
                *name,
                all.iter()
                    .cloned()
                    .enumerate()
                    .filter(|(i, _)| states[*i].column() == c)
                    .collect(),
            )
        })
        .collect();
    rsx! {
        div { class: "oxib", "data-style": style_attr(&style), "data-view": "{layout()}",
            div { class: "oxib-bar",
                span { class: "oxib-crumb", "Agents" } span { class: "oxib-slash", "/" } b { "{title}" }
                span { class: "oxib-sp" }
                div { class: "oxib-seg", role: "radiogroup", "aria-label": "Layout",
                    button { r#type: "button", role: "radio", "aria-checked": "{board}", onclick: move |_| { layout.set("board".into()); on_action.call(("view".into(), "board".into())) },
                        svg { width: "13", height: "13", view_box: "0 0 14 14", fill: "none", stroke: "currentColor", stroke_width: "1.4", "aria-hidden": "true", rect { x: "1.5", y: "2", width: "3", height: "10", rx: "1" } rect { x: "5.5", y: "2", width: "3", height: "7", rx: "1" } rect { x: "9.5", y: "2", width: "3", height: "5", rx: "1" } }
                        "Board" }
                    button { r#type: "button", role: "radio", "aria-checked": "{!board}", onclick: move |_| { layout.set("list".into()); on_action.call(("view".into(), "list".into())) },
                        svg { width: "13", height: "13", view_box: "0 0 14 14", fill: "none", stroke: "currentColor", stroke_width: "1.4", stroke_linecap: "round", "aria-hidden": "true", path { d: "M2 3.5h10M2 7h10M2 10.5h10" } }
                        "List" }
                }
            }
            div { class: "oxib-cols",
                for (c, name, items) in cols.into_iter() {
                    section { key: "{c}", class: "oxib-col", "data-col": "{c}", "aria-label": "{name}",
                        div { class: "oxib-ch", Sparkle { size: 12.0 } span { class: "oxib-cl", "{name}" } span { class: "oxib-cc", span { key: "{items.len()}", "{items.len()}" } } }
                        div { class: "oxib-list",
                            if items.is_empty() { div { class: "oxib-empty", "Nothing here" } }
                            for (i, t) in items.into_iter() { {card(i, t, !board)} }
                        }
                    }
                }
            }
            div { class: "oxib-live", role: "status", "{summary}" }
        }
    }
}
