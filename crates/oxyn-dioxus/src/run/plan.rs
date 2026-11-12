//! `TodoList` (checklist, dock), `PlanCard` (document) and `StepTimeline` (vertical) —
//! `plan-todo.md`.

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs,
    run::{
        ItemStatus, Plan, PlanItem, PlanState, TodoDock, ToolCall,
        plan::{item_window, plan_cta},
    },
};

use super::tool_call::ToolCallRow;
use crate::composer::icons::Icon;

fn status_class(s: &ItemStatus) -> &'static str {
    match s {
        ItemStatus::Pending => "pending",
        ItemStatus::Active => "active",
        ItemStatus::Done => "done",
        ItemStatus::Skipped { .. } => "skipped",
        ItemStatus::Failed { .. } => "failed",
        ItemStatus::Blocked { .. } => "blocked",
        ItemStatus::Cancelled => "cancelled",
    }
}

fn marker(s: &ItemStatus) -> Element {
    match s {
        ItemStatus::Active => rsx! { span { class: "ld" } },
        ItemStatus::Done => {
            rsx! { span { class: "draw", Icon { name: "check", size: 13.0, stroke: 2.0 } } }
        }
        ItemStatus::Failed { .. } => rsx! { Icon { name: "x", size: 13.0, stroke: 2.0 } },
        ItemStatus::Blocked { .. } => rsx! { Icon { name: "clock", size: 13.0 } },
        ItemStatus::Pending => rsx! { span { class: "hollow" } },
        s => rsx! { span { class: "g", "{s.glyph()}" } },
    }
}

fn item_row(i: &PlanItem, now: ClockMs, strike: bool, numbered: Option<usize>) -> Element {
    let st = status_class(&i.status);
    let changed = i
        .change
        .as_ref()
        .is_some_and(|(_, at)| now.since(*at) < oxyn_core::run::plan::CHANGE_HIGHLIGHT);
    let sub = match &i.status {
        ItemStatus::Failed { reason: Some(r) } => Some(r.clone()),
        ItemStatus::Blocked { on } => Some(format!("waiting on {on}")),
        ItemStatus::Skipped { reason: Some(r) } => Some(r.clone()),
        _ => None,
    };
    let label = i.label().to_string();
    let aria = format!("{label}, {}", i.status.word());
    let dur = i.duration(now);
    let cls = format!(
        "it {st}{}{}",
        if strike && i.status == ItemStatus::Done {
            " strike"
        } else {
            ""
        },
        if changed { " changed" } else { "" }
    );
    rsx! {
        li { class: "{cls}", "aria-label": "{aria}",
            span { class: "mk", {marker(&i.status)} }
            if let Some(n) = numbered { span { class: "num", "{n}." } }
            span { class: if i.status == ItemStatus::Active { "lb shim" } else { "lb" }, "{label}" }
            if matches!(i.status, ItemStatus::Skipped { .. }) { span { class: "tag", "skipped" } }
            span { class: "sp" }
            if let Some(d) = dur { span { class: "du", "{d}" } }
            if let Some(s) = sub { div { class: "why", "{s}" } }
        }
    }
}

/// A todo list: `checklist` inline, or `dock` above the composer.
#[component]
pub fn TodoList(
    /// The plan.
    plan: Plan,
    /// Agent state (dock: live / stopped).
    #[props(default = AgentState::Idle)]
    agent: AgentState,
    /// Shared clock.
    now: ClockMs,
    /// `checklist` or `dock`.
    #[props(default = "checklist".to_string())]
    variant: String,
    /// When the last item finished (dock fade).
    #[props(default)]
    done_at: Option<ClockMs>,
    /// Dock expanded.
    #[props(default)]
    expanded: bool,
    /// Strike done items through.
    #[props(default)]
    strike: bool,
) -> Element {
    let p = plan.progress();
    if variant == "dock" {
        let dock = TodoDock::derive(Some(&plan), &agent, done_at, now);
        let mut open = use_signal(|| expanded);
        let (label, current, cls, fade) = match &dock {
            TodoDock::Hidden => return rsx! {},
            TodoDock::Live { label, current } => (label.clone(), current.clone(), "live", 0.0),
            TodoDock::Completed { label, fade } => (label.clone(), None, "completed", *fade),
            TodoDock::Stopped { label } => (label.clone(), None, "stopped", 0.0),
        };
        let (a, b) = item_window(&plan.items, 2, 4);
        return rsx! {
            div { class: "oxyn-todo dock {cls}", style: "opacity:{1.0 - fade:.2}",
                button { class: "dh", r#type: "button", "aria-expanded": "{open()}", onclick: move |_| open.toggle(),
                    span { class: "mk", if cls == "completed" { Icon { name: "check", size: 13.0, stroke: 2.0 } } else if cls == "stopped" { Icon { name: "stop", size: 10.0 } } else { span { class: "ld" } } }
                    span { class: "pg", "{label}" }
                    if let Some(c) = current { span { class: "cur shim", "· {c}" } }
                    span { class: "sp" }
                    span { class: "bar", span { style: "width:{p.fraction() * 100.0:.0}%" } }
                    span { class: "chev", Icon { name: "chevron", size: 12.0 } }
                }
                if open() {
                    ul { class: "items",
                        if a > 0 { li { class: "fold", "{a} done" } }
                        for i in plan.items[a..b].iter() { {item_row(i, now, strike, None)} }
                        if b < plan.items.len() { li { class: "fold", "{plan.items.len() - b} more" } }
                    }
                }
            }
        };
    }
    let title = plan.title.clone().unwrap_or_else(|| "Tasks".into());
    rsx! {
        div { class: "oxyn-todo checklist",
            div { class: "th",
                Icon { name: "checklist", size: 14.0 }
                span { class: "tt", "{title}" }
                span { class: "sp" }
                span { class: "pg", "{p.label()}" }
            }
            ul { class: "items", for i in plan.items.iter() { {item_row(i, now, strike, None)} } }
        }
    }
}

/// The plan card (`plan-card/document`).
#[component]
pub fn PlanCard(
    /// The plan.
    plan: Plan,
    /// Shared clock.
    now: ClockMs,
    /// The plan needs sign-off (Approve instead of Build).
    #[props(default)]
    gated: bool,
    /// Build / Approve pressed.
    #[props(default)]
    on_build: EventHandler<()>,
    /// Request changes pressed.
    #[props(default)]
    on_revise: EventHandler<()>,
) -> Element {
    let p = plan.progress();
    let title = plan.title.clone().unwrap_or_else(|| "Plan".into());
    match &plan.state {
        PlanState::Executing | PlanState::Interrupted | PlanState::Done => {
            let txt = match plan.state {
                PlanState::Executing => format!("Plan · {}/{}", p.done, p.total),
                PlanState::Interrupted => format!("Plan · Stopped · {} left", p.total - p.done),
                _ => format!("Plan · Done · {}/{}", p.done, p.total),
            };
            return rsx! { button { class: "oxyn-plan pill", r#type: "button", Icon { name: "checklist", size: 13.0 } span { "{txt}" } span { class: "tt", "{title}" } } };
        }
        _ => {}
    }
    let drafting = plan.state == PlanState::Drafting;
    let dim = matches!(
        plan.state,
        PlanState::Superseded { .. } | PlanState::Rejected { .. } | PlanState::Revising
    );
    let cta = plan_cta(&plan.state, gated);
    let streaming = plan.state == PlanState::Streaming;
    let n = plan.items.len();
    rsx! {
        div { class: if dim { "oxyn-plan card dim" } else { "oxyn-plan card" },
            div { class: "ph",
                span { class: "kicker", "Plan" }
                if plan.version > 1 { span { class: "ver", "v{plan.version}" } }
                span { class: "sp" }
                if n > 0 && !drafting { span { class: "pg", "{n} steps" } }
            }
            if drafting {
                div { class: "sk t" } div { class: "sk" } div { class: "sk" } div { class: "sk s" }
                div { class: "foot", span { class: "shim", "Planning" } }
            } else {
                h3 { "{title}" }
                ol { class: "items",
                    for (k, i) in plan.items.iter().enumerate() {
                        div { class: if streaming && k + 1 == n { "enter shim-row" } else { "" }, {item_row(i, now, false, Some(k + 1))} }
                    }
                }
                match &plan.state {
                    PlanState::Superseded { by } => rsx! { div { class: "foot note", "Replaced by v{by}" } },
                    PlanState::Rejected { note } => rsx! { div { class: "foot note", "Rejected" if let Some(n) = note { " · {n}" } } },
                    PlanState::Revising => rsx! { div { class: "foot", span { class: "shim", "Revising plan…" } } },
                    PlanState::Approved => rsx! { div { class: "foot ok", Icon { name: "check", size: 13.0, stroke: 2.0 } "Approved" } },
                    _ => rsx! {
                        if let Some(c) = cta {
                            div { class: "foot",
                                button { class: "ghost", r#type: "button", onclick: move |_| on_revise.call(()), "Request changes" }
                                button { class: "primary", r#type: "button", onclick: move |_| on_build.call(()), "{c}" }
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Plan steps as a vertical timeline with their tool calls nested.
#[component]
pub fn StepTimeline(
    /// The plan.
    plan: Plan,
    /// Calls referenced by `PlanItem::calls`.
    #[props(default)]
    calls: Vec<ToolCall>,
    /// Shared clock.
    now: ClockMs,
    /// Fold to one line once every step is done.
    #[props(default)]
    fold_on_done: bool,
) -> Element {
    let p = plan.progress();
    if fold_on_done && p.total > 0 && p.done == p.total {
        let total: u64 = plan
            .items
            .iter()
            .filter_map(|i| {
                i.span
                    .and_then(|(s, e)| e.map(|e| e.since(s).as_millis() as u64))
            })
            .sum();
        let d = oxyn_core::verbs::format_summary(std::time::Duration::from_millis(total));
        return rsx! { div { class: "oxyn-steps folded", Icon { name: "check", size: 13.0, stroke: 2.0 } span { "{p.total} steps · {d}" } } };
    }
    rsx! {
        ol { class: "oxyn-steps",
            for i in plan.items.iter() {
                {
                    let st = status_class(&i.status);
                    let kids: Vec<ToolCall> = calls.iter().filter(|c| i.calls.contains(&c.id)).cloned().collect();
                    let show_kids = matches!(i.status, ItemStatus::Active | ItemStatus::Failed { .. }) && !kids.is_empty();
                    rsx! {
                        li { class: "st {st}",
                            div { class: "rail" , span { class: "mk", {marker(&i.status)} } span { class: "line" } }
                            div { class: "body",
                                div { class: "row",
                                    span { class: if i.status == ItemStatus::Active { "lb shim" } else { "lb" }, "{i.label()}" }
                                    span { class: "sp" }
                                    if let Some(d) = i.duration(now) { span { class: "du", "{d}" } }
                                }
                                if let ItemStatus::Failed { reason: Some(r) } = &i.status { div { class: "why", "{r}" } }
                                if show_kids {
                                    div { class: "calls", for c in kids { ToolCallRow { key: "{c.id}", call: c, now, variant: "minimal" } } }
                                } else if !kids.is_empty() {
                                    div { class: "count", if kids.len() == 1 { "1 tool call" } else { "{kids.len()} tool calls" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
