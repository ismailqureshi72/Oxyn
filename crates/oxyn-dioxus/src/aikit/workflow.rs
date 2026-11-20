//! AI workflow (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::first_words;
use oxyn_core::aikit::workflow::{Output, Step, StepKind, StepSt, Workflow};

use super::{cls, dots, ico, key_of, live, mark};

fn status_label(st: StepSt) -> (String, &'static str) {
    match st {
        StepSt::Idle => (String::new(), ""),
        StepSt::Pending => ("Waiting".into(), "neutral"),
        StepSt::Travel | StepSt::Running => ("Running".into(), "blue"),
        StepSt::Approval => ("Needs you".into(), "amber"),
        StepSt::Done(ms) => (format!("{ms}ms"), "green"),
        StepSt::Failed => ("Failed".into(), "red"),
        StepSt::Skipped => ("Skipped".into(), "neutral"),
    }
}

/// AI workflow: a trigger, an AI step, a branch and actions on one hairline rail. Test run glides
/// a dot down each connector, runs each step with a shimmering activity line and opens its
/// output inline (the email, a label with a streamed reason, the posted message, the created
/// issue); the untaken path is skipped; a failing step shows Retry, which resumes from it. Steps
/// are added from a command menu (search, app marks, 1–9 shortcuts) and removed with a small
/// button. Keys: Enter/Space on buttons, ←/→ on branch tabs, ↑/↓ Enter Escape in the menu.
#[component]
pub fn AiWorkflow(
    /// Host clock.
    now: ClockMs,
    /// `passes`, `fails` (the Linear step fails once), `empty` or `loading`.
    #[props(default = "passes".to_string())]
    scenario: String,
    /// Start a test run on mount.
    #[props(default = true)]
    autoplay: bool,
    /// Show the add menu and remove buttons.
    #[props(default = true)]
    editable: bool,
    /// Ask for approval before the first action (Oxyn extra).
    #[props(default)]
    gate: bool,
    /// Heading.
    #[props(default = "Escalate urgent support email".to_string())]
    title: String,
) -> Element {
    let t0 = now.0;
    let sc = scenario.clone();
    let mut wf = use_signal(move || {
        let mut w = Workflow::new(&sc, gate);
        if autoplay && !w.steps.is_empty() && !w.loading {
            w.press("wf-run", t0);
        }
        w
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let w = wf.read().clone();
    let running = w.running(n);
    let mut act = move |id: String| {
        let x = wf.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let failed = w
        .sequence()
        .iter()
        .any(|s| w.status(s.id, n) == StepSt::Failed);
    let rows = w.menu_rows();
    let render_step = move |s: Step, nested: bool, w: &Workflow, next: Option<u32>| -> Element {
        let st = w.status(s.id, n);
        let (lab, hue) = status_label(st);
        let fill = next.map(|x| w.wire_fill(x, n));
        let moving = next.is_some_and(|x| w.status(x, n) == StepSt::Travel);
        let id = s.id;
        let out = match (&s.output, w.since_done(s.id, n)) {
            (o, Some(_)) => Some(o.clone()),
            _ => None,
        };
        let busy = matches!(st, StepSt::Running | StepSt::Travel);
        let name = format!(
            "{}, {}, {}",
            s.title,
            s.app,
            if lab.is_empty() {
                s.kind.label().to_string()
            } else {
                lab.clone()
            }
        );
        rsx! {
            li { key: "{id}", class: "oxaik-wf-step", "data-st": "{hue}", "data-nested": "{nested}",
                span { class: "oxaik-wf-col", "aria-hidden": "true",
                    span { class: "oxaik-wf-node", if s.kind == StepKind::Branch { {ico("branch", 13.0)} } else { {mark(s.app, 15.0)} } }
                    if let Some(f) = fill {
                        span { class: "oxaik-wf-wire", i { style: "transform:scaleY({f})" } if moving { b { style: "top:{f * 100.0}%" } } }
                    }
                }
                div { class: "oxaik-wf-main",
                    div { class: "oxaik-wf-line",
                        span { class: "oxaik-wf-title", "aria-label": "{name}", "{s.title}" }
                        if busy { span { class: "oxaik-wf-det oxaik-shim", "{s.activity}" } } else { span { class: "oxaik-wf-det", "{s.detail}" } }
                        span { class: "oxaik-wf-st oxaik-num", "data-hue": "{hue}",
                            {match st { StepSt::Running | StepSt::Travel => dots(), StepSt::Done(_) => ico("check", 11.0), StepSt::Failed => ico("x-circle", 11.0), StepSt::Approval => ico("hand", 11.0), _ => rsx! {} }}
                            "{lab}"
                        }
                        if editable && s.kind != StepKind::Trigger && !running {
                            button { class: "oxaik-wf-rm", r#type: "button", "aria-label": "Remove {s.title}", onclick: move |_| act(format!("wf-rm-{id}")), {ico("x", 12.0)} }
                        }
                    }
                    if let Some(o) = out {
                        div { class: "oxaik-wf-out oxaik-fade",
                            {match o {
                                Output::Email(from, subj, prev) => rsx! { span { b { "{from}" } " · {subj}" } span { class: "oxaik-wf-clamp", "{prev}" } },
                                Output::Label(l, c, reason) => {
                                    let k = w.stream_words(id, reason, n);
                                    rsx! { span { class: "oxaik-row", span { class: "oxaik-pill", "data-hue": "red", span { class: "oxaik-dot" } "{l}" } span { class: "oxaik-mono", "{c:.2}" } } span { "{first_words(reason, k)}" } }
                                }
                                Output::Route(p) => rsx! { span { "Took the {p} path" } },
                                Output::Message(ch, t) => rsx! { span { b { "{ch}" } " · {t}" } },
                                Output::Record(rid, t) => rsx! { span { b { class: "oxaik-mono", "{rid}" } " · {t}" } },
                                Output::Text(t) => { let k = w.stream_words(id, t, n); rsx! { span { "{first_words(t, k)}" } } }
                            }}
                        }
                    }
                    if st == StepSt::Failed {
                        div { class: "oxaik-wf-out oxaik-wf-err", role: "alert",
                            span { "Linear returned 503: service unavailable." }
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("wf-retry".into()), {ico("retry", 12.0)} "Retry" }
                        }
                    }
                    if st == StepSt::Approval {
                        div { class: "oxaik-wf-out oxaik-wf-gate",
                            span { "The next steps write to Slack and Linear." }
                            button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("wf-approve".into()), "Approve and run" }
                        }
                    }
                }
            }
        }
    };
    let top: Vec<Step> = w.steps.clone();
    let tab = w.tab;
    let paths = w.paths.clone();
    rsx! {
        section { class: "oxaik oxaik-wf", "aria-label": "{title}", "aria-busy": "{running || w.loading}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "{title}" } p { class: "oxaik-sub", "Runs on every new email to support@corvel.dev" } }
                button { class: if running { "oxaik-btn sm" } else { "oxaik-btn sm pri" }, r#type: "button", disabled: w.loading || w.steps.is_empty(),
                    onclick: move |_| act("wf-run".into()),
                    if running { {ico("stop", 12.0)} "Stop" } else { {ico("play", 12.0)} "Test run" }
                }
            }
            if w.loading {
                div { class: "oxaik-skel", "aria-hidden": "true", i { style: "width:80%" } i { style: "width:60%" } i { style: "width:70%" } }
            } else if w.steps.is_empty() {
                div { class: "oxaik-wf-empty",
                    p { class: "oxaik-mute", "Start with a trigger" }
                    for (i, s) in rows.iter().enumerate() {
                        button { key: "{i}", class: "oxaik-wf-pick", r#type: "button", onclick: move |_| act(format!("wf-pick-{i}")),
                            {mark(s.app, 15.0)} b { "{s.title}" } span { class: "oxaik-mute", "{s.detail}" }
                        }
                    }
                }
            } else {
                ol { class: "oxaik-wf-list",
                    for (k, s) in top.iter().enumerate() { {render_step(s.clone(), false, &w, top.get(k + 1).map(|x| x.id))} }
                }
                if !paths.is_empty() {
                    div { class: "oxaik-wf-paths",
                        div { class: "oxaik-seg", role: "radiogroup", "aria-label": "Branch paths",
                            onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowLeft"|"ArrowRight"|"Home"|"End") { e.prevent_default(); let x = wf.write().key("tabs", &k, n); if let Some(x) = x { said.set(x) } } },
                            for (i, p) in paths.iter().enumerate() {
                                button { key: "{i}", r#type: "button", role: "radio", class: cls("oxaik-sg", i == tab), "aria-checked": "{i == tab}", tabindex: if i == tab { "0" } else { "-1" },
                                    onclick: move |_| act(format!("wf-path-{i}")), "{p.label}"
                                }
                            }
                        }
                        ol { class: "oxaik-wf-list", "aria-label": "{paths[tab].label}",
                            for (k, s) in paths[tab].steps.iter().enumerate() { {render_step(s.clone(), true, &w, paths[tab].steps.get(k + 1).map(|x| x.id))} }
                        }
                    }
                }
            }
            div { class: "oxaik-row oxaik-between oxaik-wf-foot",
                span { class: if failed { "oxaik-mute oxaik-bad" } else { "oxaik-mute" }, "{w.outcome(n)}" }
                if editable && !w.loading {
                    div { class: "oxaik-menu-wrap",
                        button { class: "oxaik-btn sm", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{w.menu}", onclick: move |_| act("wf-add".into()),
                            {ico("plus", 12.0)} if paths.is_empty() { "Add step" } else { "Add to {paths[tab].label}" }
                        }
                        if w.menu {
                            div { class: "oxaik-menu oxaik-wf-menu", role: "dialog", "aria-label": "Add a step",
                                input { class: "oxaik-in", r#type: "text", role: "combobox", "aria-expanded": "true", "aria-controls": "oxaik-wf-lb", "aria-activedescendant": "oxaik-wf-o{w.hi}",
                                    placeholder: "Search steps", value: "{w.query}", autofocus: true,
                                    oninput: move |e| act(format!("wf-q:{}", e.value())),
                                    onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowDown"|"ArrowUp"|"Enter"|"Escape") || (k.len() == 1 && wf.peek().query.is_empty() && k.chars().all(|c| c.is_ascii_digit())) { e.prevent_default(); let x = wf.write().key("menu", &k, n); if let Some(x) = x { said.set(x) } } },
                                }
                                ul { id: "oxaik-wf-lb", class: "oxaik-wf-opts", role: "listbox",
                                    if rows.is_empty() { li { class: "oxaik-mute oxaik-opt", "No steps match" } }
                                    for (i, s) in rows.iter().enumerate() {
                                        li { key: "{i}", id: "oxaik-wf-o{i}", role: "option", "aria-selected": "{i == w.hi}", class: cls("oxaik-opt", i == w.hi),
                                            onclick: move |_| act(format!("wf-pick-{i}")),
                                            {mark(s.app, 14.0)} span { class: "oxaik-wf-ot", "{s.title}" }
                                            if i < 9 { kbd { class: "oxaik-kbd", "{i + 1}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            {live(&said())}
        }
    }
}
