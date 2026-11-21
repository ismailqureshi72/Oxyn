//! Agent gallery (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::gallery::{
    Flow, Gallery, TEAMS, TEMPLATES, WORKSPACES, app_name, rating_words,
};

use super::status;
use crate::aikit::{cls, dots, ico, key_of, live, mark};

/// Agent gallery: a marketplace of agent templates. Team chips (a radio group, arrow keys) and
/// a search over name, job, apps and model narrow the cards; each card shows the job, the apps
/// it connects to (marks, read as one sentence), the model and its rating. Preview plays a
/// three-step sample run; Install picks a workspace (radio group), grants access app by app as
/// each connects, and confirms with a check. Escape steps back, or cancels while connecting.
#[component]
pub fn AgentGallery(
    /// Host clock.
    now: ClockMs,
    /// Start team (0 Sales … 4 Ops).
    #[props(default = 2)]
    team: usize,
    /// The second app fails to connect (error scenario).
    #[props(default)]
    fail: bool,
    /// Start with this search.
    #[props(default)]
    query: String,
) -> Element {
    let mut ga = use_signal(move || {
        let mut g = Gallery::new(team, fail);
        g.query = query.clone();
        g
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let g = ga.read().clone();
    let mut act = move |id: String| {
        let x = ga.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let shown = g.shown();
    let conn = g.connect(n);
    rsx! {
        section { class: "oxaik oxbk oxbk-ag", "aria-label": "Agent gallery",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if k == "Escape" {
                    let x = ga.write().key(&k, n);
                    if let Some(x) = x { said.set(x) }
                }
            },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Agent gallery" } p { class: "oxaik-sub", "Ready-made agents for every team." } }
                div { class: "oxbk-ag-search",
                    {ico("search", 13.0)}
                    input { class: "oxaik-in", r#type: "search", value: "{g.query}", placeholder: "Search agents or apps", "aria-label": "Search agents or apps",
                        oninput: move |e| { ga.write().query = e.value(); let l = ga.read().result_line(); said.set(l) } }
                }
            }
            div { class: "oxbk-filter oxaik-scroll", role: "radiogroup", "aria-label": "Team",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") && ga.read().flow.is_none() {
                        e.prevent_default();
                        let x = ga.write().key(&k, n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                for (i, t) in TEAMS.iter().enumerate() {
                    button { key: "{i}", class: cls("oxaik-sg", g.team == i), r#type: "button", role: "radio", "aria-checked": "{g.team == i}", tabindex: if g.team == i { "0" } else { "-1" },
                        onclick: move |_| act(format!("ag-team-{i}")), "{t}" }
                }
            }
            p { class: "oxaik-mute oxbk-ag-res", "{g.result_line()}" }
            if shown.is_empty() {
                div { class: "oxbk-empty oxbk-ag-empty",
                    p { "No agents match “{g.query}” in {TEAMS[g.team]}." }
                    if let Some(t) = g.elsewhere() {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ag-team-{t}")), "Show in {TEAMS[t]}" }
                    }
                }
            }
            div { class: "oxbk-ag-grid",
                for &i in shown.iter() {
                    {
                        let tp = TEMPLATES[i];
                        let flow = g.flow.filter(|f| f.0 == i).map(|f| f.1);
                        let installed = g.installed_in(i);
                        let apps: Vec<&str> = tp.apps.iter().map(|a| app_name(a)).collect();
                        let apps_line = format!("Connects to {}", apps.join(", "));
                        let rw = rating_words(&tp);
                        let rating = format!("{}.{}", tp.rating / 10, tp.rating % 10);
                        let inst = if tp.installs >= 1000 { format!("{:.1}k", tp.installs as f64 / 1000.0) } else { tp.installs.to_string() };
                        rsx! {
                            article { key: "{i}", class: "oxbk-ag-card", "aria-label": "{tp.name}", "data-flow": "{flow.is_some()}",
                                div { class: "oxaik-row oxaik-between",
                                    b { class: "oxbk-ag-nm", "{tp.name}" }
                                    span { class: "oxaik-row oxaik-mute oxaik-num", "aria-label": "{rw}", "★ {rating}" span { "· {inst}" } }
                                }
                                p { class: "oxbk-ag-job", "{tp.job}" }
                                div { class: "oxaik-row oxaik-between",
                                    span { class: "oxbk-ag-apps",
                                        span { class: "oxaik-sr", "{apps_line}" }
                                        span { class: "oxaik-row", "aria-hidden": "true", for a in tp.apps.iter() { {mark(a, 15.0)} } }
                                    }
                                    span { class: "oxaik-row oxaik-mute oxbk-ag-model", {mark(tp.model.0, 12.0)} "{tp.model.1}" }
                                }
                                {match (flow, installed) {
                                    (_, Some(ws)) => rsx! { p { class: "oxbk-ag-done", {status("Installed", "green")} span { class: "oxaik-mute", "in {ws}" } } },
                                    (None, None) => rsx! {
                                        div { class: "oxaik-row oxbk-ag-acts",
                                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ag-preview-{i}")), {ico("play", 11.0)} "Preview" }
                                            button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("ag-install-{i}")), "Install" }
                                        }
                                    },
                                    (Some(Flow::Preview), None) => rsx! {
                                        div { class: "oxbk-ag-flow oxaik-fade",
                                            p { class: "oxbk-cap", "Sample run" }
                                            ol { class: "oxbk-ag-steps", for (k, s) in tp.preview.iter().enumerate() { li { key: "{k}", style: "--d:{k * 220}ms", span { class: "oxaik-num", "{k + 1}" } "{s}" } } }
                                            div { class: "oxaik-row",
                                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ag-back".into()), "Back" }
                                                button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("ag-install-{i}")), "Install" }
                                            }
                                        }
                                    },
                                    (Some(Flow::Workspace(w)), None) => rsx! {
                                        div { class: "oxbk-ag-flow oxaik-fade",
                                            p { class: "oxbk-cap", "Install in" }
                                            div { class: "oxbk-ag-ws", role: "radiogroup", "aria-label": "Workspace",
                                                onkeydown: move |e: KeyboardEvent| {
                                                    let k = key_of(&e);
                                                    if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                                                        e.prevent_default();
                                                        ga.write().key(&k, n);
                                                    }
                                                },
                                                for (k, (ws, det)) in WORKSPACES.iter().enumerate() {
                                                    button { key: "{k}", class: cls("oxbk-ag-wsb", w == k), r#type: "button", role: "radio", "aria-checked": "{w == k}", tabindex: if w == k { "0" } else { "-1" },
                                                        onclick: move |_| act(format!("ag-ws-{k}")),
                                                        span { class: "oxbk-ag-radio" } b { "{ws}" } span { class: "oxaik-mute", "{det}" }
                                                    }
                                                }
                                            }
                                            div { class: "oxaik-row",
                                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ag-back".into()), "Back" }
                                                button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("ag-grant".into()), "Grant access" }
                                            }
                                        }
                                    },
                                    (Some(Flow::Connect(w, _)), None) => {
                                        let (got, failed, done) = conn.unwrap_or((0, None, false));
                                        rsx! {
                                            div { class: "oxbk-ag-flow oxaik-fade",
                                                p { class: "oxbk-cap", "Connecting to {WORKSPACES[w].0}" }
                                                ul { class: "oxbk-ag-conn",
                                                    for (k, a) in tp.apps.iter().enumerate() {
                                                        li { key: "{k}",
                                                            {mark(a, 14.0)} span { "{app_name(a)}" }
                                                            if failed == Some(k) { {status("Could not connect", "red")} }
                                                            else if k < got || done { {status("Connected", "green")} }
                                                            else if k == got { span { class: "oxaik-mute", {dots()} " Connecting" } }
                                                            else { span { class: "oxaik-mute", "Waiting" } }
                                                        }
                                                    }
                                                }
                                                if failed.is_some() {
                                                    div { class: "oxaik-row",
                                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ag-back".into()), "Cancel" }
                                                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("ag-retry".into()), {ico("retry", 12.0)} "Retry" }
                                                    }
                                                } else if done {
                                                    div { class: "oxaik-row",
                                                        {status("Installed", "green")}
                                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ag-back".into()), "Done" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    (Some(Flow::Card), None) => rsx! {},
                                }}
                            }
                        }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
