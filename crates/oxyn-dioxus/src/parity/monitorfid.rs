//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32): the exact **Agent Monitor**
//! ([`AgentMonitor`]). Behaviour and look only; the model is `oxyn_core::parity::monitorfid`.
//! Styles: `monitorfid.css` (classes `oxmn-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::monitorfid::{
        MONITOR_LOOP_MS, MonStepState, MonitorScene, MonitorState, MonitorStep, monitor_at,
        monitor_clock, monitor_cost, monitor_failed, monitor_label, step_count, step_share,
        step_took, time_strip,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn kind_icon(kind: &str) -> Element {
    let d = match kind {
        "search" => "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 16-4.2-4.2",
        "read" => "M4 5h7a2 2 0 0 1 2 2v12a2 2 0 0 0-2-2H4zm16 0h-5a2 2 0 0 0-2 2",
        "edit" => "M4 20h4L19 9l-4-4L4 16zm9-13 4 4",
        "think" => "M9 18h6M10 21h4M12 3a6 6 0 0 0-3 11v2h6v-2a6 6 0 0 0-3-11z",
        "web" => {
            "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18"
        }
        _ => "M4 6l5 5-5 5M12 18h8",
    };
    rsx! {
        svg { width: "13", height: "13", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } }
    }
}

fn tool_tile(s: &MonitorStep, size: f32) -> Element {
    rsx! {
        span { class: "oxmn-tt",
            if s.brand.is_empty() { {kind_icon(&s.kind)} } else { BrandIcon { name: s.brand.clone(), size, coloured: true, fallback: true } }
        }
    }
}

/// Agent Monitor (affectUI): one agent at work as a card – its current step with a clock and the
/// bill so far, a time strip, the finished steps with each tool's logo (a click shows the
/// details), the exact command when it needs an OK, and a fold into a small bar.
#[component]
pub fn AgentMonitor(
    /// Your own run (the demo's when `None`).
    #[props(default)]
    scene: Option<MonitorScene>,
    /// Freeze the demo this many ms into a pass.
    #[props(default)]
    at: Option<u64>,
    /// Show the demo's failed run.
    #[props(default)]
    failed: bool,
    /// Start folded, as the small bar.
    #[props(default)]
    default_folded: bool,
    /// A step whose details are open at the start (id).
    #[props(default)]
    open: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Allow while waiting (Enter does the same while the card has focus).
    #[props(default)]
    on_allow: EventHandler<()>,
    /// Decline while waiting (Esc).
    #[props(default)]
    on_decline: EventHandler<()>,
    /// Pause (`true`) or resume (`false`).
    #[props(default)]
    on_pause: EventHandler<bool>,
    /// Cancel a running run.
    #[props(default)]
    on_cancel: EventHandler<()>,
    /// Try again after a failure.
    #[props(default)]
    on_retry: EventHandler<()>,
    /// Log: jump to the full trace; shown when set.
    #[props(default)]
    on_open_log: Option<EventHandler<()>>,
) -> Element {
    let demo = scene.is_none() && !failed;
    let start = use_signal(move || now);
    let mut folded = use_signal(move || default_folded);
    let mut opened = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    // Demo steering, in real ms of the pass: (lap, allowed, declined, cancelled).
    let mut steer = use_signal(|| (0u64, None::<u64>, None::<u64>, None::<u64>));
    let mut paused_at = use_signal(|| None::<u64>);
    let mut paused_ms = use_signal(|| 0u64);

    let raw = now.since(start()).as_millis() as u64;
    let raw = paused_at().unwrap_or(raw).saturating_sub(paused_ms());
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => (raw / MONITOR_LOOP_MS, raw % MONITOR_LOOP_MS),
        _ => (0, raw),
    };
    let (sl, allowed, declined, cancelled) = steer();
    let (allowed, declined, cancelled) = if sl == lap {
        (allowed, declined, cancelled)
    } else {
        (None, None, None)
    };
    let mut sc = match (&scene, failed) {
        (Some(s), _) => s.clone(),
        (None, true) => monitor_failed(),
        _ => monitor_at(el, allowed, declined),
    };
    if let Some(c) = cancelled.filter(|c| el >= *c)
        && sc.state.busy()
    {
        sc.state = MonitorState::Cancelled;
        sc.ask = None;
        for s in sc.steps.iter_mut() {
            if s.state == MonStepState::Running {
                s.state = MonStepState::Queued;
            }
        }
        sc.clock = c.saturating_sub(400);
    }
    let paused = paused_at().is_some() && sc.state.busy();
    let state = if paused {
        MonitorState::Paused
    } else {
        sc.state
    };
    let label = monitor_label(&sc);
    let (cost_a, cost_b) = monitor_cost(sc.cost);
    let running = sc
        .steps
        .iter()
        .rev()
        .find(|s| s.state == MonStepState::Running)
        .cloned();
    let strip = time_strip(&sc.steps, sc.clock);
    let mut done: Vec<(usize, MonitorStep)> = sc
        .steps
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, s)| matches!(s.state, MonStepState::Done | MonStepState::Failed))
        .collect();
    done.reverse();
    let count = step_count(&sc.steps);
    let total: u64 = sc.steps.iter().map(|s| s.took).sum();
    let tools: Vec<MonitorStep> = sc
        .steps
        .iter()
        .filter(|s| s.state != MonStepState::Queued)
        .cloned()
        .collect();
    let lap_k = format!("lap{lap}");
    let clock = monitor_clock(sc.clock);
    let steps_all = sc.steps.clone();

    let allow = move || {
        let mut st = steer;
        st.set((lap, Some(el), None, None));
        on_allow.call(());
    };
    let decline = move || {
        let mut st = steer;
        st.set((lap, None, Some(el), None));
        on_decline.call(());
    };
    let sc_state = sc.state;
    let strip_label = steps_all
        .iter()
        .filter(|s| s.state != MonStepState::Queued)
        .map(|s| {
            let d = if s.took > 0 {
                s.took
            } else {
                sc.clock.saturating_sub(s.since)
            };
            format!("{}, {}", s.label, step_took(d))
        })
        .collect::<Vec<_>>()
        .join("; ");

    if folded() {
        let shown: Vec<MonitorStep> = tools.iter().rev().take(4).rev().cloned().collect();
        let extra = tools.len().saturating_sub(4);
        return rsx! {
            div { class: "oxmn", "data-style": style_attr(&style), "data-folded": "true", "data-st": "{state.id()}",
                button { class: "oxmn-fold", r#type: "button", "aria-label": "{label} – open", "aria-expanded": "false",
                    onclick: move |_| folded.set(false),
                    span { class: "oxmn-logo", BrandIcon { name: sc.brand.clone(), size: 16.0, coloured: true, fallback: true } }
                    b { class: "oxmn-name", "{sc.agent}" }
                    if extra > 0 { span { class: "oxmn-more", "+{extra}" } }
                    span { class: "oxmn-stack",
                        for (i, s) in shown.iter().cloned().enumerate() {
                            span { key: "{s.id}", "data-k": "{s.id}", class: "oxmn-sk", "data-last": "{i + 1 == shown.len()}", {tool_tile(&s, 13.0)} }
                        }
                    }
                    span { class: "oxmn-sp" }
                    span { class: "oxmn-pill", "data-st": "{state.id()}",
                        i {}
                        if state == MonitorState::Finished { "{step_took(total)}" } else if state.busy() { "{clock}" } else { "{state.label()}" }
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "oxmn", "data-style": style_attr(&style), "data-st": "{state.id()}", role: "region", "aria-label": "{label}", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    if opened().is_some() { opened.set(None) } else if sc_state == MonitorState::Waiting && !paused { decline() }
                } else if e.key() == Key::Enter && sc_state == MonitorState::Waiting && !paused && opened().is_none() {
                    allow()
                }
            },
            div { class: "oxmn-card",
                div { class: "oxmn-head",
                    span { class: "oxmn-logo", BrandIcon { name: sc.brand.clone(), size: 16.0, coloured: true, fallback: true } }
                    span { class: "oxmn-who",
                        b { class: "oxmn-name", "{sc.agent}" }
                        span { class: "oxmn-role", "{sc.role}" }
                    }
                    span { class: "oxmn-cost", "data-busy": "{state.busy()}", "aria-label": "Cost so far {cost_a}{cost_b}",
                        "{cost_a}" span { class: "oxmn-c2", "{cost_b}" }
                    }
                    span { class: "oxmn-pill", "data-st": "{state.id()}", i {} "{state.label()}" }
                    button { class: "oxmn-ib", r#type: "button", "aria-label": "Fold away", "aria-expanded": "true", onclick: move |_| folded.set(true),
                        svg { width: "14", height: "14", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", "aria-hidden": "true",
                            path { d: "M7 10l5-5 5 5M7 19l5-5 5 5" } }
                    }
                }
                // The well.
                div { class: "oxmn-well", "data-st": "{state.id()}", key: "{lap_k}", "data-lap": "{lap_k}",
                    if let (MonitorState::Waiting, Some(a)) = (sc.state, sc.ask.clone()) {
                        span { class: "oxmn-tt", BrandIcon { name: a.brand.clone(), size: 13.0, coloured: true, fallback: true } }
                        span { class: "oxmn-wl", b { "{a.title}" } code { class: "oxmn-code", "{a.command}" } }
                        span { class: "oxmn-clk", "{clock}" }
                    } else if let Some(r) = running.clone() {
                        {tool_tile(&r, 13.0)}
                        span { class: "oxmn-wl", span { key: "{r.id}", "data-k": "{r.id}", class: "oxmn-roll", "{r.label}" } }
                        span { class: "oxmn-clk", "aria-label": "Running for {sc.clock / 1000}s", "{clock}" }
                    } else {
                        span { class: "oxmn-wl", "data-quiet": "true",
                            {match state {
                                MonitorState::Finished => rsx! { span { class: "oxmn-ok", "Finished in {step_took(sc.clock)}" } if !sc.note.is_empty() { span { class: "oxmn-note", "{sc.note}" } } },
                                MonitorState::Failed => rsx! { span { class: "oxmn-err", "{sc.error}" } },
                                MonitorState::Cancelled => rsx! { span { "Cancelled at {clock}" } },
                                MonitorState::Paused => rsx! { span { "Paused" } },
                                _ => rsx! { span { "Waiting to start" } },
                            }}
                        }
                    }
                }
                // The time strip.
                div { class: "oxmn-strip", role: "img",
                    "aria-label": "{strip_label}",
                    "data-fin": "{state == MonitorState::Finished}",
                    for (i, share) in strip.iter().cloned() {
                        {
                            let s = steps_all[i].clone();
                            let st = if s.state == MonStepState::Running && sc.state == MonitorState::Waiting { "waiting".to_string() } else { format!("{:?}", s.state).to_lowercase() };
                            let lit = opened().as_deref() == Some(s.id.as_str());
                            let sid = s.id.clone();
                            rsx! {
                                i { key: "{s.id}", "data-k": "{s.id}", class: "oxmn-seg", "data-st": "{st}", "data-lit": "{lit}", style: "flex-grow:{share:.4}",
                                    onclick: move |_| opened.set(if opened().as_deref() == Some(sid.as_str()) { None } else { Some(sid.clone()) }) }
                            }
                        }
                    }
                }
                if !done.is_empty() {
                    ul { class: "oxmn-rows", "aria-label": "Finished steps",
                        for (i, s) in done.iter().take(3).cloned() {
                            {
                                let is_open = opened().as_deref() == Some(s.id.as_str());
                                let sid = s.id.clone();
                                let share = step_share(&steps_all, i);
                                let row_st = if s.state == MonStepState::Failed { "failed" } else { "done" };
                                rsx! {
                                    li { key: "{s.id}", "data-k": "{s.id}", class: "oxmn-li", "data-open": "{is_open}",
                                        button { class: "oxmn-row", r#type: "button", "aria-expanded": "{is_open}", "aria-label": "{s.label} – details", "data-st": "{row_st}",
                                            onclick: move |_| opened.set(if opened().as_deref() == Some(sid.as_str()) { None } else { Some(sid.clone()) }),
                                            {tool_tile(&s, 12.0)}
                                            span { class: "oxmn-rl", "{s.label}" }
                                            if s.state == MonStepState::Failed { span { class: "oxmn-rf", "Failed" } } else { span { class: "oxmn-rt", "{step_took(s.took)}" } }
                                        }
                                        if is_open {
                                            div { class: "oxmn-info", role: "dialog", "aria-label": "{s.label}",
                                                div { class: "oxmn-ih",
                                                    {tool_tile(&s, 14.0)}
                                                    span { class: "oxmn-who", b { "{s.label}" } span { class: "oxmn-role", "{s.tool}" } }
                                                    button { class: "oxmn-ib", r#type: "button", "aria-label": "Close",
                                                        onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                        onclick: move |_| opened.set(None),
                                                        svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", "aria-hidden": "true", path { d: "M6 6l12 12M18 6 6 18" } }
                                                    }
                                                }
                                                dl { class: "oxmn-grid",
                                                    div { dt { "Started" } dd { "+{monitor_clock(s.since)}" } }
                                                    div { dt { "Took" } dd { "{step_took(s.took)}" } }
                                                    div { dt { "Share" } dd { "{share}" } }
                                                    div { dt { "State" } dd { if s.state == MonStepState::Failed { "Failed" } else { "Done" } } }
                                                }
                                                if !s.detail.is_empty() { p { "{s.detail}" } }
                                                if !s.output.is_empty() { code { class: "oxmn-code", "data-k": "out", "{s.output}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxmn-foot",
                    {match (sc.state, paused) {
                        (MonitorState::Waiting, false) => rsx! {
                            button { class: "oxmn-btn", r#type: "button", onclick: move |_| decline(), "Decline" kbd { "Esc" } }
                            button { class: "oxmn-btn", "data-k": "allow", r#type: "button", onclick: move |_| allow(), "Allow" kbd { "Enter" } }
                        },
                        (MonitorState::Running | MonitorState::Waiting, _) => rsx! {
                            button { class: "oxmn-btn", r#type: "button", "aria-pressed": "{paused}",
                                onclick: move |_| {
                                    if let Some(p) = paused_at() {
                                        let now_raw = now.since(start()).as_millis() as u64;
                                        paused_ms.set(paused_ms() + now_raw.saturating_sub(p));
                                        paused_at.set(None);
                                        on_pause.call(false);
                                    } else {
                                        paused_at.set(Some(now.since(start()).as_millis() as u64));
                                        on_pause.call(true);
                                    }
                                },
                                if paused {
                                    svg { width: "10", height: "10", view_box: "0 0 24 24", "aria-hidden": "true", path { d: "M7 4.5v15l12-7.5z", fill: "currentColor" } }
                                    "Resume"
                                } else {
                                    svg { width: "10", height: "10", view_box: "0 0 24 24", "aria-hidden": "true", fill: "currentColor", rect { x: "6", y: "5", width: "4", height: "14", rx: "1" } rect { x: "14", y: "5", width: "4", height: "14", rx: "1" } }
                                    "Pause"
                                }
                            }
                            button { class: "oxmn-btn", r#type: "button",
                                onclick: move |_| { steer.set((lap, None, None, Some(el))); paused_at.set(None); on_cancel.call(()) },
                                svg { width: "10", height: "10", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.4", stroke_linecap: "round", "aria-hidden": "true", path { d: "M6 6l12 12M18 6 6 18" } } "Cancel" }
                        },
                        (MonitorState::Failed, _) => rsx! {
                            button { class: "oxmn-btn", "data-k": "allow", r#type: "button", onclick: move |_| on_retry.call(()), "↻ Try again" }
                        },
                        _ => rsx! {},
                    }}
                    span { class: "oxmn-sp" }
                    span { class: "oxmn-count", "{count}" }
                    if let Some(h) = on_open_log {
                        button { class: "oxmn-log", r#type: "button", onclick: move |_| h.call(()), "Log ↗" }
                    }
                }
            }
            div { class: "oxmn-sr", role: "status", "aria-live": "polite", if sc.state == MonitorState::Waiting { "Needs you: {sc.ask.as_ref().map(|a| a.title.clone()).unwrap_or_default()}" } }
        }
    }
}
