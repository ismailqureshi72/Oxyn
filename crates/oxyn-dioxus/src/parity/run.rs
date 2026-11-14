//! Plans, progress and agents (plan #28, chunk 2): `AgentPlan`, `AgentTrail`, `TaskRows`,
//! `FlowStepper`, `OnboardingChecklist`, `ReleaseReadiness`, `TaskBreakdown`, `AgentTree`,
//! `AgentFleet`, `AgentOrchestrator`, `TreeTable`, `ActivityFeed`, `Countdown`,
//! `AnimatedCounter`, `AgentStatus`, `AgentRun`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        fleet::{
            Countdown as CountdownModel, CrewAgent, EventKind, Feed, OrchPhase, Orchestration,
            Readiness, Resolve, TreeTable as TreeModel, Tri, digits_roll, dollars, group,
        },
        plan::{
            Breakdown, Checklist, PlanDraft, PlanPhase, StepState, Stepper, TaskRow, Trail,
            TrailStep, short_duration,
        },
    },
};

use crate::composer::icons::Icon;

fn mark(state: StepState) -> Element {
    match state {
        StepState::Done => {
            rsx! { span { class: "oxr-mk done", Icon { name: "check", size: 11.0, stroke: 2.2 } } }
        }
        StepState::Running => rsx! { span { class: "oxr-mk run" } },
        StepState::Waiting => {
            rsx! { span { class: "oxr-mk wait", Icon { name: "pause", size: 9.0 } } }
        }
        StepState::Failed => {
            rsx! { span { class: "oxr-mk fail", Icon { name: "x", size: 10.0, stroke: 2.2 } } }
        }
        StepState::Declined => {
            rsx! { span { class: "oxr-mk decl", Icon { name: "ban", size: 10.0 } } }
        }
        StepState::Skipped => {
            rsx! { span { class: "oxr-mk skip", Icon { name: "minus", size: 10.0 } } }
        }
        StepState::Pending => rsx! { span { class: "oxr-mk pend" } },
    }
}

fn ring(p: f32, size: f32) -> Element {
    let r = size / 2.0 - 2.0;
    let c = std::f32::consts::TAU * r;
    let off = c * (1.0 - p.clamp(0.0, 1.0));
    rsx! {
        svg { class: "oxr-ring", width: "{size}", height: "{size}", view_box: "0 0 {size} {size}", "aria-hidden": "true",
            circle { cx: "{size / 2.0}", cy: "{size / 2.0}", r: "{r}", class: "bg" }
            circle { cx: "{size / 2.0}", cy: "{size / 2.0}", r: "{r}", class: "fg", stroke_dasharray: "{c}", stroke_dashoffset: "{off}",
                transform: "rotate(-90 {size / 2.0} {size / 2.0})" }
        }
    }
}

/// Agent plan: review before it runs (rename, reorder, remove, add), then live timers per step
/// and Retry from a failed step.
#[component]
pub fn AgentPlan(
    /// Plan.
    plan: PlanDraft,
    /// Shared clock.
    now: ClockMs,
    /// Show Cancel next to Run plan.
    #[props(default = true)]
    cancellable: bool,
    /// Rename step.
    #[props(default)]
    on_rename: EventHandler<(usize, String)>,
    /// Move step from → to.
    #[props(default)]
    on_move: EventHandler<(usize, usize)>,
    /// Remove step.
    #[props(default)]
    on_remove: EventHandler<usize>,
    /// Add a step.
    #[props(default)]
    on_add: EventHandler<String>,
    /// Run plan / Run again.
    #[props(default)]
    on_run: EventHandler<()>,
    /// Cancel.
    #[props(default)]
    on_cancel: EventHandler<()>,
    /// Retry from a step id.
    #[props(default)]
    on_retry: EventHandler<String>,
) -> Element {
    let mut drag = use_signal(|| None::<usize>);
    let mut adding = use_signal(String::new);
    let edit = plan.editable();
    let (done, total) = plan.progress();
    let n = plan.steps.len();
    rsx! {
        section { class: "oxr-plan", "data-phase": match plan.phase { PlanPhase::Draft => "draft", PlanPhase::Running => "running", PlanPhase::Done => "done" },
            header { class: "oxr-hd",
                Icon { name: "checklist", size: 15.0 }
                h3 { "{plan.title}" }
                span { class: "oxr-sp" }
                if edit { if let Some(e) = plan.estimate.clone() { span { class: "oxr-dim", "{e}" } } }
                else { span { class: "oxr-dim", "{done}/{total}" } }
            }
            ol { class: "oxr-steps", "aria-label": "{plan.title}",
                for (i, s) in plan.steps.iter().cloned().enumerate() {
                    li { class: "oxr-step", "data-state": s.state.id(), draggable: "{edit}",
                        ondragstart: move |_| drag.set(Some(i)),
                        ondragover: move |e| e.prevent_default(),
                        ondrop: move |_| { if let Some(f) = drag() { on_move.call((f, i)); } drag.set(None); },
                        if edit { span { class: "oxr-grip", "aria-hidden": "true", Icon { name: "grip", size: 12.0 } } } else { {mark(s.state)} }
                        span { class: "oxr-n", "{i + 1}" }
                        div { class: "oxr-body",
                            if edit {
                                input { class: "oxr-title-in", value: "{s.title}", "aria-label": "Step {i + 1}",
                                    onchange: move |e| on_rename.call((i, e.value())),
                                    onkeydown: move |e: KeyboardEvent| {
                                        let m = e.modifiers();
                                        if m.alt() && e.key() == Key::ArrowUp && i > 0 { e.prevent_default(); on_move.call((i, i - 1)) }
                                        if m.alt() && e.key() == Key::ArrowDown && i + 1 < n { e.prevent_default(); on_move.call((i, i + 1)) }
                                    } }
                            } else { div { class: "oxr-title", "{s.title}" } }
                            if let Some(d) = s.detail.clone() { div { class: "oxr-dim oxr-small", "{d}" } }
                            if let Some(err) = s.error.clone() {
                                div { class: "oxr-err", Icon { name: "alert", size: 12.0 } "{err}"
                                    button { class: "oxr-lnk", r#type: "button", onclick: { let id = s.id.clone(); move |_| on_retry.call(id.clone()) }, "Retry from here" } }
                            }
                        }
                        if let Some(t) = s.tool.clone() { span { class: "oxr-tool", "{t}" } }
                        if let Some(tm) = s.timer(now) { span { class: "oxr-time", "{tm}" } }
                        if edit && n > 1 { button { class: "oxr-x", r#type: "button", "aria-label": "Remove step {i + 1}", onclick: move |_| on_remove.call(i), Icon { name: "x", size: 12.0 } } }
                    }
                }
            }
            if edit {
                form { class: "oxr-add", onsubmit: move |e| { e.prevent_default(); if !adding().trim().is_empty() { on_add.call(adding()); adding.set(String::new()) } },
                    Icon { name: "plus", size: 13.0 }
                    input { placeholder: "Add a step", value: "{adding}", oninput: move |e| adding.set(e.value()), "aria-label": "Add a step" }
                }
            }
            footer { class: "oxr-tray",
                span { class: "oxr-dim oxr-small", if edit { "Drag or ⌥↑ ⌥↓ to reorder" } else if plan.phase == PlanPhase::Done { "Plan finished" } else { "Running" } }
                span { class: "oxr-sp" }
                if edit && cancellable { button { class: "oxr-btn", r#type: "button", onclick: move |_| on_cancel.call(()), "Cancel" } }
                if edit { button { class: "oxr-btn go", r#type: "button", onclick: move |_| on_run.call(()), "Run plan" } }
                if plan.phase == PlanPhase::Done { button { class: "oxr-btn", r#type: "button", onclick: move |_| on_run.call(()), "Run again" } }
            }
        }
    }
}

/// Agent trail: one quiet column of steps with a header ("3/5"), checks and times for done
/// steps, the current one shimmering with a live timer and its sub-steps, gates and Needs you.
#[component]
pub fn AgentTrail(
    /// Trail.
    trail: Trail,
    /// Shared clock.
    now: ClockMs,
    /// Show sub-steps of the running step.
    #[props(default = true)]
    substeps: bool,
    /// Dense rows.
    #[props(default)]
    dense: bool,
) -> Element {
    rsx! {
        section { class: if dense { "oxr-trail dense" } else { "oxr-trail" },
            if !trail.title.is_empty() {
                header { class: "oxr-hd",
                    if let Some(a) = trail.agent.clone() { span { class: "oxr-av", "{a.chars().next().unwrap_or('A')}" } span { class: "oxr-dim", "{a}" } }
                    h3 { "{trail.title}" }
                    span { class: "oxr-sp" }
                    span { class: "oxr-count", "{trail.count()}" }
                }
            }
            ol { class: "oxr-tl",
                for s in trail.steps.iter() {
                    li { class: "oxr-tli", "data-state": s.state.id(), "data-gate": "{s.gate}",
                        {mark(s.state)}
                        div { class: "oxr-body",
                            div { class: "oxr-line",
                                if let Some(t) = s.tool.clone() { span { class: "oxr-tool", "{t}" } }
                                span { class: "oxr-title", "{s.title}" }
                                if let Some(b) = s.badge() { span { class: "oxr-badge {s.state.id()}", "{b}" } }
                                span { class: "oxr-sp" }
                                if let (StepState::Running, Some(st)) = (s.state, s.started) {
                                    span { class: "oxr-time live", "{short_duration(now.0.saturating_sub(st.0))}" }
                                } else if let Some(t) = s.took_ms { span { class: "oxr-time", "{short_duration(t)}" } }
                            }
                            if s.state == StepState::Running { if let Some(st) = s.status.clone() { div { class: "oxr-shim", "{st}…" } } }
                            if substeps && s.state == StepState::Running && !s.sub.is_empty() {
                                ul { class: "oxr-sub",
                                    for (t, st) in s.sub.iter() { li { "data-state": st.id(), {mark(*st)} span { "{t}" } } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Task rows (Beautiful UI): capsules or a list; a ring with the step for running rows, a
/// Completed or Failed pill with Retry, details that open per row.
#[component]
pub fn TaskRows(
    /// Rows.
    rows: Vec<TaskRow>,
    /// `capsules` · `list`.
    #[props(default = "capsules".to_string())]
    variant: String,
    /// Toggle a row open.
    #[props(default)]
    on_toggle: EventHandler<usize>,
    /// Retry a failed row.
    #[props(default)]
    on_retry: EventHandler<usize>,
) -> Element {
    rsx! {
        ul { class: "oxr-rows {variant}", role: "list",
            for (i, r) in rows.iter().cloned().enumerate() {
                li { class: "oxr-row", "data-state": r.state.id(), style: "--d:{i * 80}ms",
                    button { class: "oxr-rowbtn", r#type: "button", "aria-expanded": "{r.open}", onclick: move |_| on_toggle.call(i),
                        if r.state == StepState::Running { {ring(r.ring(), 18.0)} } else { {mark(r.state)} }
                        span { class: "oxr-title", "{r.label}" }
                        span { class: "oxr-sp" }
                        if let Some(a) = r.amount.clone() { span { class: "oxr-amt", "{a}" } }
                        if let Some(p) = r.pill() { span { class: "oxr-pill {r.state.id()}", "{p}" } }
                        span { class: "oxr-chev", Icon { name: "chevron", size: 12.0 } }
                    }
                    if r.state == StepState::Failed { button { class: "oxr-lnk oxr-retry", r#type: "button", onclick: move |_| on_retry.call(i), Icon { name: "retry", size: 12.0 } "Retry" } }
                    if r.open {
                        ul { class: "oxr-det",
                            for (l, m) in r.details.iter() { li { span { "{l}" } span { class: "oxr-dim", "{m}" } } }
                        }
                    }
                }
            }
        }
    }
}

/// Stepper: where a person is in a multi-step flow. Horizontal or vertical, compact markers,
/// descriptions for all steps or only the current one, errors, and arrow-key selection of
/// completed steps when `on_select` is set.
#[component]
pub fn FlowStepper(
    /// Steps and current index.
    stepper: Stepper,
    /// `horizontal` · `vertical`.
    #[props(default = "horizontal".to_string())]
    orientation: String,
    /// `all` · `current`.
    #[props(default = "all".to_string())]
    details: String,
    /// Markers only with a caption.
    #[props(default)]
    compact: bool,
    /// Accessible name.
    #[props(default = "Progress".to_string())]
    label: String,
    /// Shown when every step is complete.
    #[props(default = "All steps complete".to_string())]
    complete_label: String,
    /// A completed step was chosen (makes them interactive).
    #[props(default)]
    on_select: Option<EventHandler<usize>>,
) -> Element {
    let st = stepper.clone();
    let caption = st.caption(&complete_label);
    rsx! {
        nav { class: "oxr-stepper {orientation}", "data-compact": "{compact}", "aria-label": "{label}",
            ol {
                for (i, s) in st.steps.iter().cloned().enumerate() {
                    li { class: "oxr-fs", "data-state": st.state(i).id(), "aria-current": if i == st.current { "step" } else { "false" },
                        style: "--c:{st.connector(i)}",
                        {
                            let reach = i <= st.current;
                            let sel = on_select;
                            let stc = st.clone();
                            rsx! {
                                button { class: "oxr-fsm", r#type: "button", disabled: sel.is_none() || !reach, tabindex: if i == st.current.min(st.steps.len().saturating_sub(1)) { "0" } else { "-1" },
                                    onclick: move |_| if let Some(f) = sel { f.call(i) },
                                    onkeydown: move |e: KeyboardEvent| {
                                        let d = match e.key() { Key::ArrowRight | Key::ArrowDown => 1, Key::ArrowLeft | Key::ArrowUp => -1, Key::Home => -99, Key::End => 99, _ => 0 };
                                        if d != 0 { e.prevent_default(); if let Some(f) = sel { f.call(stc.move_focus(i, d)) } }
                                    },
                                    match stc.state(i) {
                                        StepState::Done => rsx! { Icon { name: "check", size: 12.0, stroke: 2.2 } },
                                        StepState::Failed => rsx! { Icon { name: "alert", size: 12.0 } },
                                        _ => rsx! { "{i + 1}" },
                                    }
                                }
                            }
                        }
                        if !compact {
                            div { class: "oxr-fst",
                                div { class: "oxr-title", "{s.title}" if s.optional { span { class: "oxr-dim", " · optional" } } }
                                if let Some(e) = s.error.clone() { div { class: "oxr-err", "{e}" } }
                                else if let Some(d) = s.description.clone() { if details == "all" || i == st.current { div { class: "oxr-dim oxr-small", "{d}" } } }
                            }
                        }
                        span { class: "oxr-conn", "aria-hidden": "true" }
                    }
                }
            }
            if compact { div { class: "oxr-cap", role: "status", "{caption}" } }
            else if st.complete() { div { class: "oxr-sr", role: "status", "{complete_label}" } }
        }
    }
}

/// Onboarding checklist: a ring and count, expandable steps with an action, Hide / Show, and a
/// done title once everything is complete.
#[component]
pub fn OnboardingChecklist(
    /// Model.
    list: Checklist,
    /// Heading.
    #[props(default = "Set up your workspace".to_string())]
    title: String,
    /// Title once done.
    #[props(default = "You’re all set".to_string())]
    done_title: String,
    /// Toggle an item done.
    #[props(default)]
    on_toggle: EventHandler<String>,
    /// Open an item.
    #[props(default)]
    on_open: EventHandler<String>,
    /// Hide / Show.
    #[props(default)]
    on_hide: EventHandler<bool>,
) -> Element {
    let (d, t) = list.count();
    let all = list.all_done();
    let hidden = list.hidden;
    rsx! {
        section { class: "oxr-check", "data-done": "{all}",
            header { class: "oxr-hd",
                {ring(list.ring(), 30.0)}
                div { class: "oxr-body",
                    h3 { if all { "{done_title}" } else { "{title}" } }
                    div { class: "oxr-dim oxr-small", "{d} of {t} done" }
                }
                span { class: "oxr-sp" }
                button { class: "oxr-lnk", r#type: "button", "aria-expanded": "{!hidden}", onclick: move |_| on_hide.call(!hidden), if hidden { "Show" } else { "Hide" } }
            }
            if !hidden {
                ul { class: "oxr-cl",
                    for it in list.items.iter().cloned() {
                        li { class: "oxr-cli", "data-done": "{it.done}", "data-open": "{list.open.as_deref() == Some(it.id.as_str())}",
                            div { class: "oxr-cli-h",
                                button { class: "oxr-box", r#type: "button", role: "checkbox", "aria-checked": "{it.done}", "aria-label": "{it.title}",
                                    onclick: { let id = it.id.clone(); move |_| on_toggle.call(id.clone()) },
                                    if it.done { svg { class: "oxp-draw", view_box: "0 0 16 16", width: "12", height: "12", path { d: "M3 8.5l3.2 3L13 4.5" } } } }
                                button { class: "oxr-cli-t", r#type: "button", onclick: { let id = it.id.clone(); move |_| on_open.call(id.clone()) }, "{it.title}" }
                            }
                            if list.open.as_deref() == Some(it.id.as_str()) && !it.done {
                                div { class: "oxr-cli-b",
                                    p { "{it.body}" }
                                    if let Some(a) = it.action.clone() { button { class: "oxr-btn go", r#type: "button", onclick: { let id = it.id.clone(); move |_| on_toggle.call(id.clone()) }, "{a}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Release readiness: prerequisites grouped with owners, a progress bar, blockers called out and
/// a release button that waits for them (spinner while it runs, an inline alert on failure).
#[component]
pub fn ReleaseReadiness(
    /// Model.
    model: Readiness,
    /// Toggle item `i`.
    #[props(default)]
    on_toggle: EventHandler<usize>,
    /// Release pressed.
    #[props(default)]
    on_release: EventHandler<()>,
) -> Element {
    let (d, t) = model.count();
    let b = model.blockers();
    rsx! {
        section { class: "oxr-ready",
            header { class: "oxr-hd",
                h3 { "Release {model.version}" }
                span { class: "oxr-sp" }
                span { class: "oxr-count", key: "{d}", "{d}/{t}" }
            }
            div { class: "oxr-bar", role: "progressbar", "aria-valuemin": "0", "aria-valuemax": "{t}", "aria-valuenow": "{d}", span { style: "width:{d as f32 / t.max(1) as f32 * 100.0}%" } }
            for g in model.groups() {
                div { class: "oxr-grp", "{g}" }
                for (i, it) in model.items.iter().enumerate().filter(|(_, x)| x.group == g) {
                    label { class: "oxr-pre", "data-done": "{it.done}",
                        input { r#type: "checkbox", checked: it.done, onchange: move |_| on_toggle.call(i) }
                        span { class: "oxr-title", "{it.title}" }
                        if it.blocking && !it.done { span { class: "oxr-badge failed", "Blocker" } }
                        span { class: "oxr-sp" }
                        span { class: "oxr-dim oxr-small", "{it.owner}" }
                    }
                }
            }
            if let Some(e) = model.error.clone() { div { class: "oxr-alert", role: "alert", Icon { name: "alert", size: 13.0 } "{e}" } }
            footer { class: "oxr-tray",
                span { class: "oxr-dim oxr-small", if b > 0 { "{b} blocker(s) left" } else if model.release == StepState::Done { "Released" } else { "Ready to ship" } }
                span { class: "oxr-sp" }
                button { class: "oxr-btn go", r#type: "button", disabled: !model.can_release(), "aria-busy": "{model.release == StepState::Running}", onclick: move |_| on_release.call(()),
                    if model.release == StepState::Running { span { class: "oxp-spin" } "Releasing" } else if model.release == StepState::Done { Icon { name: "check", size: 13.0 } "Released" } else { "Release {model.version}" }
                }
            }
        }
    }
}

/// Task breakdown: a goal broken into tasks with estimates, owners and dependencies, and a
/// timeline that says whether the deadline holds. Reorder, regenerate, send.
#[component]
pub fn TaskBreakdown(
    /// Model.
    model: Breakdown,
    /// Tasks streamed so far (for the streaming reveal).
    #[props(default = usize::MAX)]
    shown: usize,
    /// Move task up (`true`) or down.
    #[props(default)]
    on_shift: EventHandler<(usize, bool)>,
    /// Regenerate.
    #[props(default)]
    on_regenerate: EventHandler<()>,
) -> Element {
    let sched = model.schedule();
    let span = model.span().max(model.deadline_days).max(1.0);
    let ok = model.on_track();
    let n = model.tasks.len();
    rsx! {
        section { class: "oxr-bd",
            div { class: "oxr-goal", Icon { name: "spark", size: 14.0 } span { "{model.goal}" } span { class: "oxr-sp" }
                button { class: "oxr-btn", r#type: "button", onclick: move |_| on_regenerate.call(()), Icon { name: "retry", size: 12.0 } "Regenerate" } }
            ol { class: "oxr-bdl",
                for (i, t) in model.tasks.iter().enumerate().take(shown) {
                    li { class: "oxr-bdi", style: "--d:{i * 70}ms",
                        span { class: "oxr-n", "{i + 1}" }
                        div { class: "oxr-body",
                            div { class: "oxr-title", "{t.title}" }
                            div { class: "oxr-dim oxr-small", "{t.days}d" if !t.deps.is_empty() { " · after {t.deps.join(\", \")}" } }
                            div { class: "oxr-gantt", span { style: "left:{sched[i].0 / span * 100.0}%;width:{t.days / span * 100.0}%" } }
                        }
                        span { class: "oxr-av", "{t.owner}" }
                        div { class: "oxr-mv",
                            button { r#type: "button", "aria-label": "Move up", disabled: i == 0, onclick: move |_| on_shift.call((i, true)), "↑" }
                            button { r#type: "button", "aria-label": "Move down", disabled: i + 1 == n, onclick: move |_| on_shift.call((i, false)), "↓" }
                        }
                    }
                }
            }
            div { class: if ok { "oxr-verdict ok" } else { "oxr-verdict late" }, role: "status",
                span { class: "oxr-dl", style: "left:{model.deadline_days / span * 100.0}%" }
                "{model.verdict()}"
            }
        }
    }
}

fn crew_card(a: &CrewAgent, now: ClockMs, depth: usize, has_kids: bool, folded: bool) -> Element {
    let cost = if a.children.is_empty() {
        a.cost
    } else {
        a.branch_cost()
    };
    rsx! {
        div { class: "oxr-agent", "data-state": a.state.id(), style: "--depth:{depth}",
            {mark(a.state)}
            div { class: "oxr-body",
                div { class: "oxr-line",
                    span { class: "oxr-title", "{a.name}" }
                    if let Some(m) = a.model.clone() { span { class: "oxr-dim oxr-small", "{m}" } }
                    if a.state == StepState::Waiting { span { class: "oxr-badge waiting", "Needs you" } }
                    if a.state == StepState::Failed { span { class: "oxr-badge failed", "Failed" } }
                }
                div { class: "oxr-dim oxr-small oxr-clip", "{a.task}" }
                if folded && has_kids { div { class: "oxr-dim oxr-small", "{a.children.len()} sub-agents folded" } }
            }
            div { class: "oxr-meta",
                span { class: "oxr-mono", "{dollars(cost)}" }
                if let Some(t) = a.time(now) { span { class: "oxr-time", "{t}" } }
            }
        }
    }
}

/// Agent tree: an orchestrator and its sub-agents as a live tree. Status, cost and time on every
/// card, costs rolling up each branch, folding branches, and a window with Approve / Reject or
/// Retry / Dismiss for agents that need you or failed.
#[component]
pub fn AgentTree(
    /// Root agent.
    root: CrewAgent,
    /// Shared clock.
    now: ClockMs,
    /// Heading.
    #[props(default = "Agents".to_string())]
    title: String,
    /// Folded branch ids.
    #[props(default)]
    folded: Vec<String>,
    /// Fold / open a branch.
    #[props(default)]
    on_fold: EventHandler<String>,
    /// Answer for a waiting or failed agent.
    #[props(default)]
    on_resolve: EventHandler<(String, Resolve)>,
) -> Element {
    let mut rows = Vec::new();
    root.rows(&folded, 0, &mut rows);
    let (run, wait, done, fail) = root.tally();
    rsx! {
        section { class: "oxr-tree",
            header { class: "oxr-hd",
                h3 { "{title}" }
                span { class: "oxr-dim oxr-small", "{run} running · {wait} need you · {done} done" if fail > 0 { " · {fail} failed" } }
                span { class: "oxr-sp" }
                span { class: "oxr-mono", "{dollars(root.branch_cost())}" }
            }
            div { class: "oxr-tree-b", role: "tree",
                for (d, a) in rows.iter() {
                    div { class: "oxr-tn", role: "treeitem", "aria-level": "{d + 1}", "aria-expanded": if a.children.is_empty() { "" } else if folded.contains(&a.id) { "false" } else { "true" },
                        if !a.children.is_empty() {
                            button { class: "oxr-fold", r#type: "button", style: "--depth:{d}", "aria-label": "Fold {a.name}", "data-folded": "{folded.contains(&a.id)}",
                                onclick: { let id = a.id.clone(); move |_| on_fold.call(id.clone()) }, Icon { name: "chevron", size: 11.0 } }
                        }
                        {crew_card(a, now, *d, !a.children.is_empty(), folded.contains(&a.id))}
                        if let Some(ask) = a.ask.clone().filter(|_| a.state == StepState::Waiting) {
                            div { class: "oxr-win", style: "--depth:{d}", role: "dialog", "aria-label": "{a.name} needs you",
                                p { "{ask}" }
                                button { class: "oxr-btn", r#type: "button", onclick: { let id = a.id.clone(); move |_| on_resolve.call((id.clone(), Resolve::Reject)) }, "Reject" }
                                button { class: "oxr-btn go", r#type: "button", onclick: { let id = a.id.clone(); move |_| on_resolve.call((id.clone(), Resolve::Approve)) }, "Approve" }
                            }
                        }
                        if let Some(err) = a.error.clone().filter(|_| a.state == StepState::Failed) {
                            div { class: "oxr-win fail", style: "--depth:{d}", role: "dialog", "aria-label": "{a.name} failed",
                                p { "{err}" }
                                button { class: "oxr-btn", r#type: "button", onclick: { let id = a.id.clone(); move |_| on_resolve.call((id.clone(), Resolve::Dismiss)) }, "Dismiss" }
                                button { class: "oxr-btn go", r#type: "button", onclick: { let id = a.id.clone(); move |_| on_resolve.call((id.clone(), Resolve::Retry)) }, "Retry" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Agent fleet: several agents side by side; each card sums up its sub-agents and a click opens
/// its tree to the right while the previous one folds back into its card.
#[component]
pub fn AgentFleet(
    /// Main agents.
    agents: Vec<CrewAgent>,
    /// Shared clock.
    now: ClockMs,
    /// Open agent id.
    #[props(default)]
    open: Option<String>,
    /// Open / fold an agent.
    #[props(default)]
    on_open: EventHandler<Option<String>>,
    /// Answer for a sub-agent.
    #[props(default)]
    on_resolve: EventHandler<(String, Resolve)>,
) -> Element {
    let total: f64 = agents.iter().map(|a| a.branch_cost()).sum();
    let opened = agents
        .iter()
        .find(|a| Some(&a.id) == open.as_ref())
        .cloned();
    rsx! {
        section { class: "oxr-fleet",
            header { class: "oxr-hd", h3 { "Fleet" } span { class: "oxr-dim oxr-small", "{agents.len()} agents" } span { class: "oxr-sp" } span { class: "oxr-mono", "{dollars(total)}" } }
            div { class: "oxr-fleet-b",
                div { class: "oxr-fleet-cards",
                    for a in agents.iter() {
                        {
                            let (r, w, d, f) = a.tally();
                            let is_open = Some(&a.id) == open.as_ref();
                            let id = a.id.clone();
                            rsx! {
                                button { class: "oxr-fcard", r#type: "button", "aria-pressed": "{is_open}", "data-state": a.state.id(),
                                    onclick: move |_| on_open.call(if is_open { None } else { Some(id.clone()) }),
                                    div { class: "oxr-line", {mark(a.state)} span { class: "oxr-title", "{a.name}" } span { class: "oxr-sp" } span { class: "oxr-mono", "{dollars(a.branch_cost())}" } }
                                    div { class: "oxr-dim oxr-small oxr-clip", "{a.task}" }
                                    div { class: "oxr-tally",
                                        if r > 0 { span { class: "run", "{r} running" } }
                                        if w > 0 { span { class: "wait", "{w} need you" } }
                                        if d > 0 { span { "{d} done" } }
                                        if f > 0 { span { class: "fail", "{f} failed" } }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(a) = opened {
                    div { class: "oxr-fleet-tree", key: "{a.id}",
                        AgentTree { root: a, now, title: "Tree".to_string(), on_resolve: move |x| on_resolve.call(x) }
                    }
                }
            }
        }
    }
}

/// Agent orchestrator: a lead turns a goal into a plan, fans subtasks out to worker lanes under a
/// concurrency limit (1–4) and merges what comes back.
#[component]
pub fn AgentOrchestrator(
    /// Model.
    model: Orchestration,
    /// Concurrency changed.
    #[props(default)]
    on_concurrency: EventHandler<usize>,
) -> Element {
    let phase = match model.phase {
        OrchPhase::Planning => "Planning",
        OrchPhase::FanOut => "Working",
        OrchPhase::Merging => "Merging",
        OrchPhase::Done => "Done",
    };
    rsx! {
        section { class: "oxr-orch",
            header { class: "oxr-hd", h3 { "{model.goal}" } }
            div { class: "oxr-lead", span { class: "oxr-av", "L" } span { class: "oxr-title", "{model.lead}" } span { class: "oxr-badge", "{phase}" }
                span { class: "oxr-sp" }
                span { class: "oxr-dim oxr-small", "Parallel" }
                div { class: "oxp-seg", role: "radiogroup", "aria-label": "Concurrency",
                    for n in 1..=4usize { button { r#type: "button", role: "radio", "aria-checked": "{model.concurrency == n}", onclick: move |_| on_concurrency.call(n), "{n}" } }
                }
            }
            div { class: "oxr-lanes",
                for l in model.lanes.iter() {
                    div { class: "oxr-lane", "data-state": l.state.id(),
                        div { class: "oxr-line", {mark(l.state)} span { class: "oxr-title", "{l.title}" } span { class: "oxr-sp" } span { class: "oxr-dim oxr-small", "{l.worker}" } }
                        div { class: "oxr-dim oxr-small oxr-clip", "{l.instruction}" }
                        div { class: "oxr-bar", span { style: "width:{l.progress * 100.0}%" } }
                        if let Some(r) = l.result.clone() { div { class: "oxr-small", "{r}" } }
                    }
                }
            }
            div { class: "oxr-tray", span { class: "oxr-dim oxr-small", "{model.status()}" } }
            if let Some(s) = model.summary.clone() { div { class: "oxr-sum", Icon { name: "merge", size: 13.0 } "{s}" } }
        }
    }
}

/// Tree table: nested rows, tri-state selection, sort that keeps the hierarchy, lazy children
/// with a retry row, a pinned header and a capped height.
#[component]
pub fn TreeTable(
    /// Model.
    model: TreeModel,
    /// Column headers.
    columns: Vec<String>,
    /// Checkbox column.
    #[props(default)]
    selectable: bool,
    /// Max body height px (0 = none).
    #[props(default)]
    max_height: u32,
    /// Expand / collapse.
    #[props(default)]
    on_toggle: EventHandler<String>,
    /// Select.
    #[props(default)]
    on_select: EventHandler<String>,
    /// Sort column.
    #[props(default)]
    on_sort: EventHandler<usize>,
    /// Retry a lazy load.
    #[props(default)]
    on_retry: EventHandler<String>,
) -> Element {
    let rows = model.visible();
    let style = if max_height > 0 {
        format!("max-height:{max_height}px")
    } else {
        String::new()
    };
    rsx! {
        div { class: "oxr-tt", style: "{style}",
            table { role: "treegrid", "aria-multiselectable": "{selectable}",
                thead { tr {
                    if selectable { th { class: "oxr-tt-c" } }
                    for (i, c) in columns.iter().enumerate() {
                        th { "aria-sort": match model.sort { Some((x, true)) if x == i => "ascending", Some((x, false)) if x == i => "descending", _ => "none" },
                            button { r#type: "button", onclick: move |_| on_sort.call(i), "{c}"
                                span { class: "oxr-sort", match model.sort { Some((x, true)) if x == i => "↑", Some((x, false)) if x == i => "↓", _ => "" } } }
                        }
                    }
                } }
                tbody {
                    if rows.is_empty() { tr { td { colspan: "9", class: "oxr-dim", "Nothing to show" } } }
                    for (d, r) in rows.iter().cloned() {
                        tr { "aria-level": "{d + 1}", "aria-selected": "{model.tri(&r.id) == Tri::On}",
                            if selectable {
                                td { class: "oxr-tt-c",
                                    button { class: "oxr-box", r#type: "button", role: "checkbox",
                                        "aria-checked": match model.tri(&r.id) { Tri::On => "true", Tri::Mixed => "mixed", Tri::Off => "false" },
                                        "aria-label": "Select {r.cells[0]}",
                                        onclick: { let id = r.id.clone(); move |_| on_select.call(id.clone()) },
                                        match model.tri(&r.id) { Tri::On => rsx! { Icon { name: "check", size: 10.0, stroke: 2.4 } }, Tri::Mixed => rsx! { Icon { name: "minus", size: 10.0, stroke: 2.4 } }, Tri::Off => rsx! {} }
                                    }
                                }
                            }
                            for (ci, cell) in r.cells.iter().enumerate() {
                                td { class: if ci == 0 { "oxr-tt-n" } else { "oxr-tt-v" },
                                    if ci == 0 {
                                        span { style: "width:{d * 18}px", class: "oxr-ind" }
                                        if !r.children.is_empty() || r.lazy {
                                            button { class: "oxr-fold", r#type: "button", "data-folded": "{!model.expanded.contains(&r.id)}", "aria-label": "Toggle {cell}",
                                                onclick: { let id = r.id.clone(); move |_| on_toggle.call(id.clone()) }, Icon { name: "chevron", size: 11.0 } }
                                        } else { span { class: "oxr-ind", style: "width:20px" } }
                                    }
                                    "{cell}"
                                }
                            }
                        }
                        if r.lazy && model.expanded.contains(&r.id) && r.children.is_empty() {
                            tr { class: "oxr-tt-load", td { colspan: "9",
                                span { style: "display:inline-block;width:{(d + 1) * 18 + 20}px" }
                                if r.failed { span { class: "oxr-err", "Couldn't load" } button { class: "oxr-lnk", r#type: "button", onclick: { let id = r.id.clone(); move |_| on_retry.call(id.clone()) }, "Retry" } }
                                else { span { class: "oxp-spin" } span { class: "oxr-dim", " Loading…" } }
                            } }
                        }
                    }
                }
            }
        }
    }
}

/// Activity feed and timeline: events grouped by day, newest first, a type filter, held-back new
/// updates behind a button, expandable details and Mark all read.
#[component]
pub fn ActivityFeed(
    /// Model.
    feed: Feed,
    /// `feed` (filter, unread, mark read) · `timeline` (quiet line with markers).
    #[props(default = "feed".to_string())]
    variant: String,
    /// Heading.
    #[props(default = "Activity".to_string())]
    title: String,
    /// Filter changed.
    #[props(default)]
    on_filter: EventHandler<Option<EventKind>>,
    /// Toggle an event.
    #[props(default)]
    on_toggle: EventHandler<String>,
    /// Show new updates.
    #[props(default)]
    on_show_new: EventHandler<()>,
    /// Mark all read.
    #[props(default)]
    on_read_all: EventHandler<()>,
) -> Element {
    let groups = feed.groups();
    let unread = feed.unread();
    let kinds = [
        None,
        Some(EventKind::Code),
        Some(EventKind::Deploy),
        Some(EventKind::Agent),
        Some(EventKind::Comment),
    ];
    let tl = variant == "timeline";
    rsx! {
        section { class: "oxr-feed {variant}",
            header { class: "oxr-hd",
                h3 { "{title}" }
                if !tl && unread > 0 { span { class: "oxr-pill run", "{unread} new" } }
                span { class: "oxr-sp" }
                if !tl { button { class: "oxr-lnk", r#type: "button", disabled: unread == 0, onclick: move |_| on_read_all.call(()), "Mark all read" } }
            }
            if !tl {
                div { class: "oxp-seg", role: "radiogroup", "aria-label": "Type",
                    for k in kinds { button { r#type: "button", role: "radio", "aria-checked": "{feed.filter == k}", onclick: move |_| on_filter.call(k), "{k.map(|x| x.id()).unwrap_or(\"all\")}" } }
                }
            }
            if !feed.incoming.is_empty() {
                button { class: "oxr-new", r#type: "button", onclick: move |_| on_show_new.call(()), Icon { name: "up", size: 12.0 } "{feed.incoming.len()} new updates" }
            }
            for (day, evs) in groups.iter() {
                h4 { class: "oxr-day", "{day}" }
                ol { class: "oxr-evs",
                    for e in evs.iter() {
                        li { class: "oxr-ev", "data-unread": "{e.unread}", "data-kind": e.kind.id(),
                            span { class: "oxr-evm", Icon { name: e.kind.icon(), size: 12.0 } }
                            div { class: "oxr-body",
                                button { class: "oxr-evb", r#type: "button", "aria-expanded": "{feed.expanded.contains(&e.id)}", disabled: e.detail.is_none(),
                                    onclick: { let id = e.id.clone(); move |_| on_toggle.call(id.clone()) },
                                    b { "{e.actor}" } " {e.action} " span { class: "oxr-tgt", "{e.target}" }
                                    span { class: "oxr-sp" }
                                    span { class: "oxr-time", "{e.time}" }
                                }
                                if feed.expanded.contains(&e.id) { if let Some(d) = e.detail.clone() { div { class: "oxr-evd", "{d}" } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Countdown: rolling digits with unit labels (large) or one compact line; morphs into a live
/// state with a pulsing dot at zero.
#[component]
pub fn Countdown(
    /// Model.
    model: CountdownModel,
    /// Shared clock.
    now: ClockMs,
    /// `large` · `compact`.
    #[props(default = "large".to_string())]
    variant: String,
    /// Show days (when false, hours absorb them).
    #[props(default = true)]
    days: bool,
    /// Shown after zero.
    #[props(default = "Live now".to_string())]
    complete_label: String,
    /// Accessible name.
    #[props(default = "Time remaining".to_string())]
    label: String,
) -> Element {
    let Some(p) = model.parts(now, !days) else {
        return rsx! { div { class: "oxr-cd live {variant}", role: "timer", "aria-label": "{label}", span { class: "oxr-live" } "{complete_label}" } };
    };
    if variant == "compact" {
        let s = model.compact(now).unwrap_or_default();
        return rsx! { div { class: "oxr-cd compact", role: "timer", "aria-label": "{label}", Icon { name: "clock", size: 12.0 } span { class: "oxr-mono", "{s}" } } };
    }
    let units = [
        ("days", p[0]),
        ("hours", p[1]),
        ("minutes", p[2]),
        ("seconds", p[3]),
    ];
    rsx! {
        div { class: "oxr-cd large", role: "timer", "aria-label": "{label}",
            for (u, v) in units.iter().filter(|(u, _)| days || *u != "days") {
                div { class: "oxr-cdu",
                    span { class: "oxr-cdd", key: "{v}", "{v:02}" }
                    span { class: "oxr-dim oxr-small", "{u}" }
                }
            }
        }
    }
}

/// Animated counter: label, prefix and suffix, fixed decimals, thousands separators, each digit
/// column rolling in the direction the number moved.
#[component]
pub fn AnimatedCounter(
    /// Value.
    value: f64,
    /// Previous value (for the roll).
    #[props(default)]
    from: Option<f64>,
    /// Roll progress 0..1 (the host animates it).
    #[props(default = 1.0)]
    t: f32,
    /// Small label above.
    #[props(default)]
    label: Option<String>,
    /// Prefix ("$").
    #[props(default)]
    prefix: String,
    /// Suffix ("%").
    #[props(default)]
    suffix: String,
    /// Decimals.
    #[props(default)]
    decimals: usize,
    /// Size px.
    #[props(default = 32.0)]
    size: f32,
) -> Element {
    let text = group(value, decimals);
    let (cols, up) = digits_roll(from.unwrap_or(value), value, decimals, t);
    let rolling = t < 1.0 && from.is_some();
    let cols: Vec<_> = cols
        .into_iter()
        .map(|(a, b, k)| (a, b, if up { -k } else { k } * 100.0))
        .collect();
    rsx! {
        div { class: "oxr-ctr", style: "--fs:{size}px",
            if let Some(l) = label { div { class: "oxr-dim oxr-small", "{l}" } }
            div { class: "oxr-ctr-n", "aria-label": "{prefix}{text}{suffix}", role: "img",
                if !prefix.is_empty() { span { class: "oxr-aff", "{prefix}" } }
                if rolling {
                    for (a, b, off) in cols.iter() {
                        span { class: "oxr-col", "aria-hidden": "true",
                            span { class: "oxr-roll", style: "transform:translateY({off}%)",
                                span { "{a}" } span { class: if up { "nx dn" } else { "nx up" }, "{b}" } }
                        }
                    }
                } else { span { "aria-hidden": "true", "{text}" } }
                if !suffix.is_empty() { span { class: "oxr-aff", "{suffix}" } }
            }
        }
    }
}

/// Agent status: who is working, what state it is in (idle, thinking, acting, waiting, error,
/// done), what it does right now (shimmering) and for how long.
#[component]
pub fn AgentStatus(
    /// Agent name.
    name: String,
    /// `idle` · `thinking` · `acting` · `waiting` · `error` · `done`.
    state: String,
    /// What it does right now.
    #[props(default)]
    activity: Option<String>,
    /// Started at (live timer).
    #[props(default)]
    started: Option<ClockMs>,
    /// Shared clock.
    now: ClockMs,
    /// `pill` · `card` · `inline`.
    #[props(default = "pill".to_string())]
    variant: String,
) -> Element {
    let label = match state.as_str() {
        "thinking" => "Thinking",
        "acting" => "Working",
        "waiting" => "Needs you",
        "error" => "Error",
        "done" => "Done",
        _ => "Idle",
    };
    let t = started.map(|s| short_duration(now.0.saturating_sub(s.0)));
    let busy = matches!(state.as_str(), "thinking" | "acting");
    rsx! {
        div { class: "oxr-as {variant}", "data-state": "{state}", role: "status", "aria-live": "polite",
            span { class: "oxr-as-av", "{name.chars().next().unwrap_or('A')}" span { class: "oxr-as-dot" } }
            span { class: "oxr-title", "{name}" }
            span { class: "oxr-as-st", "{label}" }
            if let Some(a) = activity { span { class: if busy { "oxr-shim oxr-clip" } else { "oxr-dim oxr-clip" }, "{a}" } }
            if let Some(t) = t { span { class: "oxr-time", "{t}" } }
        }
    }
}

/// Agent run: one card following a coding agent through a task: steps stream in with the live
/// one shimmering, the agent stops on a proposed change for you to approve or send back, then a
/// result with tests and Open pull request.
#[component]
pub fn AgentRun(
    /// Task (title).
    task: String,
    /// Model name.
    model: String,
    /// Steps so far.
    steps: Vec<TrailStep>,
    /// Proposed change `(file, diff lines)` waiting for review.
    #[props(default)]
    change: Option<(String, Vec<String>)>,
    /// Result line once done ("12 tests passed").
    #[props(default)]
    result: Option<String>,
    /// Shared clock.
    now: ClockMs,
    /// Approve the change.
    #[props(default)]
    on_approve: EventHandler<()>,
    /// Send it back with a note.
    #[props(default)]
    on_reject: EventHandler<String>,
    /// Open a pull request.
    #[props(default)]
    on_pr: EventHandler<()>,
) -> Element {
    let mut note = use_signal(String::new);
    let mut rejecting = use_signal(|| false);
    let trail = Trail {
        title: String::new(),
        agent: None,
        steps,
    };
    rsx! {
        section { class: "oxr-run",
            header { class: "oxr-hd", span { class: "oxr-av", "{model.chars().next().unwrap_or('M')}" } div { class: "oxr-body", h3 { "{task}" } div { class: "oxr-dim oxr-small", "{model}" } } }
            AgentTrail { trail, now, dense: true }
            if let Some((file, diff)) = change {
                div { class: "oxr-change",
                    div { class: "oxr-line", Icon { name: "file", size: 12.0 } span { class: "oxr-mono", "{file}" } span { class: "oxr-sp" } span { class: "oxr-badge waiting", "Review" } }
                    div { class: "oxp-g-diff", for l in diff.iter() { div { class: if l.starts_with('+') { "add" } else if l.starts_with('-') { "del" } else { "ctx" }, "{l}" } } }
                    if rejecting() {
                        form { class: "oxr-add", onsubmit: move |e| { e.prevent_default(); on_reject.call(note()); rejecting.set(false) },
                            input { placeholder: "What should change?", value: "{note}", oninput: move |e| note.set(e.value()) }
                            button { class: "oxr-btn", r#type: "submit", "Send back" }
                        }
                    } else {
                        div { class: "oxr-tray", span { class: "oxr-sp" }
                            button { class: "oxr-btn", r#type: "button", onclick: move |_| rejecting.set(true), "Request changes" }
                            button { class: "oxr-btn go", r#type: "button", onclick: move |_| on_approve.call(()), "Approve & write" }
                        }
                    }
                }
            }
            if let Some(r) = result {
                div { class: "oxr-tray", Icon { name: "check-circle", size: 14.0 } span { "{r}" } span { class: "oxr-sp" }
                    button { class: "oxr-btn go", r#type: "button", onclick: move |_| on_pr.call(()), Icon { name: "branch", size: 12.0 } "Open pull request" } }
            }
        }
    }
}
