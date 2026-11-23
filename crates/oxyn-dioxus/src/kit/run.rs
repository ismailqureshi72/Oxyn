//! Plans, progress and agents (plan #28, chunk 2): `AgentPlan`, `AgentTrail`, `TaskRows`,
//! `FlowStepper`, `OnboardingChecklist`, `LaunchChecklist`, `TaskBreakdown`, `AgentTree`,
//! `AgentFleet`, `AgentOrchestrator`, `TreeTable`, `ActivityFeed`, `Countdown`,
//! `AnimatedCounter`, `AgentStatus`, `AgentRun`.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
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

/// Agent trail (affectUI Agent Trail spec): a header with the agent's mark, the task and "3/5";
/// done steps with a tinted check and their time; the running step's status shimmering with a
/// live timer and its sub-steps, a short fading light running down the rail into the sub-step it
/// is on; long sub-step lists fold to five around it ("+N earlier", "+N more", "Show fewer");
/// gated steps with a dashed ring and Approval, then Needs you, a button that opens Approve and
/// Decline (Esc closes); declined steps struck through; the tool's coloured logo before each
/// title, faded while still to come. RTL, keyboard and a polite live region built in.
/// Variants: `plain` (default), `card` (a bordered card) and `inline` (no header rule, tighter).
#[component]
pub fn AgentTrail(
    /// Trail.
    trail: Trail,
    /// Shared clock.
    now: ClockMs,
    /// Show sub-steps (the running step opens; others open with a click).
    #[props(default = true)]
    substeps: bool,
    /// Dense rows.
    #[props(default)]
    dense: bool,
    /// `plain` · `card` · `inline`.
    #[props(default = "plain".to_string())]
    variant: String,
    /// Right-to-left.
    #[props(default)]
    rtl: bool,
    /// A waiting step was approved (index). With this or `on_decline`, Needs you is a button.
    #[props(default)]
    on_approve: Option<EventHandler<usize>>,
    /// A waiting step was declined (index).
    #[props(default)]
    on_decline: Option<EventHandler<usize>>,
) -> Element {
    let mut opened = use_signal(|| None::<Vec<usize>>);
    let mut unfold = use_signal(Vec::<usize>::new);
    let mut asking = use_signal(|| None::<usize>);
    let mut said = use_signal(String::new);
    let cur = trail.current();
    let open_set: Vec<usize> = opened().unwrap_or_else(|| cur.into_iter().collect());
    let answer = on_approve.is_some() || on_decline.is_some();
    let step_h = if dense { 28.0 } else { 32.0 };
    let sub_h = 24.0;
    // Where the light runs to: the running step's current sub-step, else the step itself.
    let mut y = 0.0f32;
    let mut light_to = None::<f32>;
    for (i, s) in trail.steps.iter().enumerate() {
        let show = substeps && open_set.contains(&i) && !s.sub.is_empty();
        if Some(i) == cur && s.state == StepState::Running {
            let mut to = y + step_h / 2.0;
            if show {
                let f =
                    oxyn_core::kit::plan::fold(s.sub.len(), s.current_sub(), unfold().contains(&i));
                let k = s.current_sub().unwrap_or(0);
                let rows = (if f.earlier > 0 { 1 } else { 0 }) + k.saturating_sub(f.start);
                to = y + step_h + rows as f32 * sub_h + sub_h / 2.0;
            }
            light_to = Some(to);
        }
        y += step_h;
        if show {
            let f = oxyn_core::kit::plan::fold(s.sub.len(), s.current_sub(), unfold().contains(&i));
            let extra = (f.earlier > 0) as usize
                + (f.more > 0 || (unfold().contains(&i) && s.sub.len() > 6)) as usize;
            y += (f.end - f.start + extra) as f32 * sub_h;
        }
        if asking() == Some(i) {
            y += 84.0;
        }
    }
    let vclass = format!("oxt oxt-{variant}{}", if dense { " dense" } else { "" });
    let count = trail.count();
    let count_l = trail.count_label();
    let list_name = if trail.title.is_empty() {
        "Agent steps".to_string()
    } else {
        trail.title.clone()
    };
    rsx! {
        section { class: "{vclass}", dir: if rtl { "rtl" } else { "ltr" }, style: "--oxt-step:{step_h}px;--oxt-sub:{sub_h}px",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && asking().is_some() { asking.set(None); },
            if !trail.title.is_empty() {
                header { class: "oxt-hd",
                    if let Some(a) = trail.agent.clone() {
                        span { class: "oxt-ag", crate::brand::BrandIcon { name: a.clone(), size: 14.0, fallback: true } span { class: "oxt-agn", "{a}" } }
                    }
                    h3 { class: "oxt-task", title: "{trail.title}", "{trail.title}" }
                    span { class: "oxt-count", "aria-label": "{count_l}", "{count}" }
                }
            }
            if trail.steps.is_empty() {
                p { class: "oxt-empty", "No steps yet" }
            } else {
                ol { class: "oxt-list", "aria-label": "{list_name}",
                    if let Some(to) = light_to { span { class: "oxt-light", "aria-hidden": "true", style: "--oxt-to:{to}px" } }
                    for (i, s) in trail.steps.iter().cloned().enumerate() {
                        { let has_sub = substeps && !s.sub.is_empty();
                          let open = has_sub && open_set.contains(&i);
                          let all = unfold().contains(&i);
                          let f = oxyn_core::kit::plan::fold(s.sub.len(), s.current_sub(), all);
                          let k = s.current_sub();
                          let label = trail.step_label(i, now);
                          let time = oxyn_core::kit::plan::trail_time(&s, now);
                          let title = if s.title.is_empty() { "Untitled step".to_string() } else { s.title.clone() };
                          let is_cur = Some(i) == cur;
                          let ask_open = asking() == Some(i);
                          let badge = s.badge();
                          let run_step = s.state == StepState::Running;
                          let ok_said = format!("Approved: {title}");
                          let ask_text = s.ask.clone().unwrap_or_else(|| format!("Let the agent {}", title.to_lowercase()));
                          let row = rsx! {
                              {mark(s.state)}
                              if let Some(t) = s.tool.clone() { span { class: "oxt-tool", title: "{t}", crate::brand::BrandIcon { name: t.clone(), size: 14.0, fallback: true } } }
                              span { class: "oxt-title", title: "{title}", "{title}" }
                              span { class: "oxt-right",
                                  if run_step { if let Some(st) = s.status.clone() { span { class: "oxr-shim oxt-status", "{st}" } } }
                                  if let Some(t) = time.clone() { span { class: "oxt-time", dir: "ltr", "{t}" } }
                              }
                          };
                          rsx! {
                            li { key: "{i}", class: "oxt-step", "data-state": s.state.id(), "data-gate": "{s.gate}", "aria-current": if is_cur { "step" } else { "false" },
                                if has_sub {
                                    button { r#type: "button", class: "oxt-row", "aria-expanded": "{open}", "aria-label": "{label}",
                                        onclick: move |_| { let mut v = opened().unwrap_or_else(|| cur.into_iter().collect()); if let Some(p) = v.iter().position(|x| *x == i) { v.remove(p); } else { v.push(i); } opened.set(Some(v)); },
                                        {row}
                                    }
                                } else {
                                    div { class: "oxt-row", "aria-label": "{label}", {row} }
                                }
                                if let Some(b) = badge {
                                    if s.state == StepState::Waiting && answer {
                                        button { r#type: "button", class: "oxt-badge waiting oxt-needs", "aria-expanded": "{ask_open}", "aria-haspopup": "true",
                                            onclick: move |_| { asking.set(if asking() == Some(i) { None } else { Some(i) }); },
                                            "{b}" }
                                    } else {
                                        span { class: "oxt-badge {s.state.id()}", "{b}" }
                                    }
                                }
                                if ask_open {
                                    div { class: "oxt-ask", role: "group", "aria-label": "Answer {title}",
                                        p { "{ask_text}" }
                                        div { class: "oxt-ask-b",
                                            button { r#type: "button", class: "oxt-btn go", onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                onclick: move |_| { asking.set(None); said.set(ok_said.clone()); if let Some(h) = on_approve { h.call(i); } }, "Approve" }
                                            button { r#type: "button", class: "oxt-btn", onclick: move |_| { asking.set(None); said.set("Declined".to_string()); if let Some(h) = on_decline { h.call(i); } }, "Decline" }
                                        }
                                    }
                                }
                                if open {
                                    ul { class: "oxt-subs",
                                        if f.earlier > 0 { li { class: "oxt-fold", button { r#type: "button", onclick: move |_| unfold.write().push(i), "+{f.earlier} earlier" } } }
                                        for j in f.start..f.end {
                                            { let (t, st) = s.sub[j].clone(); let d = s.detail(j, 24);
                                              rsx! { li { key: "{j}", class: "oxt-sub", "data-state": st.id(), "aria-current": if run_step && Some(j) == k && st == StepState::Running { "step" } else { "false" },
                                                {mark(st)} span { class: "oxt-title", title: "{t}", "{t}" }
                                                if let Some(d) = d { span { class: "oxt-detail", dir: "ltr", "{d}" } }
                                              } } }
                                        }
                                        if f.more > 0 { li { class: "oxt-fold", button { r#type: "button", onclick: move |_| unfold.write().push(i), "+{f.more} more" } } }
                                        if all && s.sub.len() > 6 { li { class: "oxt-fold", button { r#type: "button", onclick: move |_| unfold.write().retain(|x| *x != i), "Show fewer" } } }
                                    }
                                }
                            }
                          } }
                    }
                }
            }
            div { class: "oxt-live", role: "status", "aria-live": "polite", "{said}" }
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

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
