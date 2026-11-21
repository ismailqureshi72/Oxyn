//! AI setup wizard (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::thousands;
use oxyn_core::blocks::wizard::{GUARDS, MODELS, QUESTIONS, SOURCES, STEPS, Src, TONES, Wizard};

use super::switch;
use crate::aikit::{cls, dots, ico, key_of, live, mark};

/// AI setup wizard: three steps to turn on an AI assistant. Connect sources (each syncs with a
/// live document count and a progress line; a failed one offers Retry), pick the model from
/// three cards, a tone and guardrails, then ask a test question whose answer streams with
/// numbered citations that match the connected sources. Go live flips the header from Draft to
/// Live and shows a summary.
#[component]
pub fn AiSetupWizard(
    /// Host clock.
    now: ClockMs,
    /// Starting step (0 to 2; 3 opens live).
    #[props(default)]
    start: usize,
    /// Intercom fails on its first sync (error scenario).
    #[props(default)]
    fail: bool,
) -> Element {
    let mut wz = use_signal(move || {
        let mut w = Wizard::new(start, fail, now.0);
        if start > 0 {
            for s in w.src.iter_mut() {
                if matches!(s, Src::Syncing(_)) {
                    *s = Src::Syncing(now.0.saturating_sub(10_000));
                }
            }
        }
        if let Some(a) = w.ask.as_mut() {
            a.1 = now.0.saturating_sub(10_000);
        }
        w
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let w = wz.read().clone();
    let mut act = move |id: String| {
        let x = wz.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ready = w.ready(n);
    let streaming = w.streaming(n);
    let cited = w.cited();
    rsx! {
        section { class: "oxaik oxbk oxbk-wz", "aria-label": "Set up Ask Northwind",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h2 { class: "oxaik-h", "Ask Northwind" }
                    p { class: "oxaik-sub", "Support assistant setup" }
                }
                span { class: "oxaik-pill", "data-hue": if w.live { "green" } else { "neutral" }, {ico(if w.live { "check-circle" } else { "pencil" }, 11.0)} if w.live { "Live" } else { "Draft" } }
            }
            ol { class: "oxbk-wz-steps", "aria-label": "Steps",
                for (i, st) in STEPS.iter().enumerate() {
                    li { key: "{i}",
                        button { class: cls("oxbk-wz-step", i == w.step && !w.live), r#type: "button", disabled: i > w.reached || w.live, "aria-current": if i == w.step && !w.live { "step" },
                            onclick: move |_| act(format!("wz-step-{i}")),
                            span { class: "oxbk-wz-n", "data-done": "{i < w.reached || w.live}", if i < w.reached || w.live { {ico("check", 11.0)} } else { "{i + 1}" } }
                            span { class: "oxbk-wz-sl", "{st}" }
                        }
                    }
                }
            }
            div { class: "oxbk-wz-view",
                if w.live {
                    div { class: "oxbk-wz-live oxaik-fade",
                        span { class: "oxbk-wz-check", {ico("check-circle", 28.0)} }
                        b { "Ask Northwind is live" }
                        ul { class: "oxbk-wz-sum",
                            li { span { class: "oxaik-mute", "Sources" } span { class: "oxaik-row", for i in ready.iter().copied() { {mark(SOURCES[i].1, 13.0)} } "{ready.len()} connected" } }
                            li { span { class: "oxaik-mute", "Model" } span { class: "oxaik-row", {mark(MODELS[w.model].1, 13.0)} "{MODELS[w.model].0}" } }
                            li { span { class: "oxaik-mute", "Tone" } span { "{TONES[w.tone]}" } }
                            li { span { class: "oxaik-mute", "Guardrails" } span { "{w.guards.iter().filter(|g| **g).count()} on" } }
                        }
                    }
                } else if w.step == 0 {
                    ul { class: "oxbk-wz-srcs",
                        for (i, s) in SOURCES.iter().enumerate() {
                            {
                                let (cnt, done) = w.synced(i, n);
                                let st = w.src[i];
                                rsx! {
                                    li { key: "{i}", class: "oxbk-wz-src",
                                        span { class: "oxbk-mcp-mk", {mark(s.1, 16.0)} }
                                        div { class: "oxbk-wz-sb",
                                            b { "{s.0}" }
                                            match st {
                                                Src::Off => rsx! { span { class: "oxaik-mute", "{thousands(u64::from(s.2))} documents" } },
                                                Src::Failed => rsx! { span { class: "oxbk-st", "data-hue": "red", {ico("x-circle", 11.0)} span { "Token expired" } } },
                                                Src::Syncing(_) => rsx! {
                                                    span { class: "oxbk-st", "data-hue": if done { "green" } else { "blue" }, {ico(if done { "check-circle" } else { "retry" }, 11.0)}
                                                        span { class: "oxaik-num", if done { "Synced {thousands(u64::from(cnt))}" } else { "Syncing {thousands(u64::from(cnt))} of {thousands(u64::from(s.2))}" } } }
                                                    span { class: "oxaik-bar", role: "progressbar", "aria-label": "{s.0} sync", "aria-valuenow": "{cnt}", "aria-valuemax": "{s.2}", span { style: "width:{f64::from(cnt) / f64::from(s.2) * 100.0}%", "data-hue": if done { "green" } else { "blue" } } }
                                                },
                                            }
                                        }
                                        match st {
                                            Src::Failed => rsx! { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("wz-retry-{i}")), {ico("retry", 12.0)} "Retry" } },
                                            Src::Syncing(_) => rsx! { button { class: "oxaik-btn sm", r#type: "button", "aria-label": "Disconnect {s.0}", onclick: move |_| act(format!("wz-connect-{i}")), "Disconnect" } },
                                            Src::Off => rsx! { button { class: "oxaik-btn sm pri", r#type: "button", "aria-label": "Connect {s.0}", onclick: move |_| act(format!("wz-connect-{i}")), {ico("plug", 12.0)} "Connect" } },
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if w.step == 1 {
                    div { class: "oxbk-wz-models", role: "radiogroup", "aria-label": "Model",
                        onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if k.starts_with("Arrow") || k == "Home" || k == "End" { e.prevent_default(); let x = wz.write().key(&k, n); if let Some(x) = x { said.set(x) } } },
                        for (i, m) in MODELS.iter().enumerate() {
                            button { key: "{i}", class: cls("oxbk-wz-model", i == w.model), r#type: "button", role: "radio", "aria-checked": "{i == w.model}", tabindex: if i == w.model { "0" } else { "-1" },
                                onclick: move |_| act(format!("wz-model-{i}")),
                                {mark(m.1, 18.0)} b { "{m.0}" } span { class: "oxaik-mute", "{m.2} · {m.3}" } }
                        }
                    }
                    p { class: "oxbk-cap", "Tone" }
                    div { class: "oxaik-seg", role: "radiogroup", "aria-label": "Tone",
                        for (i, t) in TONES.iter().enumerate() {
                            button { key: "{i}", class: cls("oxaik-sg", i == w.tone), r#type: "button", role: "radio", "aria-checked": "{i == w.tone}", onclick: move |_| act(format!("wz-tone-{i}")), "{t}" }
                        }
                    }
                    p { class: "oxbk-cap", "Guardrails" }
                    ul { class: "oxbk-wz-guards",
                        for (i, g) in GUARDS.iter().enumerate() {
                            li { key: "{i}", {switch(g.0, w.guards[i], EventHandler::new(move |_| act(format!("wz-guard-{i}"))))} div { b { "{g.0}" } p { class: "oxaik-mute", "{g.1}" } } }
                        }
                    }
                } else {
                    div { class: "oxaik-row oxbk-wz-qs",
                        for (i, q) in QUESTIONS.iter().enumerate() {
                            button { key: "{i}", class: "oxaik-chip", r#type: "button", disabled: streaming, "aria-pressed": "{w.ask.map(|a| a.0) == Some(i)}", onclick: move |_| act(format!("wz-ask-{i}")), "{q.0}" }
                        }
                        if streaming { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("wz-stop".into()), {ico("stop", 12.0)} "Stop" } }
                    }
                    if w.ask.is_some() {
                        div { class: "oxbk-wz-ans oxaik-fade",
                            p { {mark(MODELS[w.model].1, 13.0)} " "
                                for (k, (t, c)) in w.answer(n).into_iter().enumerate() {
                                    span { key: "{k}", "{t} " if let Some(c) = c { sup { class: "oxbk-in-ref", "aria-label": "source {c}", "{c}" } " " } }
                                }
                                if streaming { {dots()} }
                            }
                            if !streaming {
                                ol { class: "oxbk-wz-cites", "aria-label": "Sources",
                                    for (k, s) in cited.iter().enumerate() {
                                        li { key: "{k}", span { class: "oxbk-in-ref", "{k + 1}" } {mark(SOURCES[*s].1, 12.0)} span { class: "oxbk-wz-cn", "{SOURCES[*s].0}" } span { class: "oxaik-mute oxbk-wz-ct", "Help article" } }
                                    }
                                }
                            }
                        }
                    } else {
                        p { class: "oxbk-empty", {ico("comment", 13.0)} "Ask a test question to see how it answers." }
                    }
                }
            }
            footer { class: "oxaik-row oxbk-wz-foot",
                if w.live {
                    button { class: "oxaik-btn", r#type: "button", onclick: move |_| act("wz-edit".into()), {ico("pencil", 12.0)} "Edit setup" }
                } else {
                    span { class: "oxaik-mute oxbk-wz-meta", "{ready.len()} sources · {MODELS[w.model].0}" }
                    button { class: "oxaik-btn", r#type: "button", disabled: w.step == 0, onclick: move |_| act("wz-back".into()), "Back" }
                    button { class: "oxaik-btn pri", r#type: "button", disabled: !w.can_next(n), onclick: move |_| act("wz-next".into()),
                        if w.step == 2 { {ico("bolt", 12.0)} "Go live" } else { "Continue" } }
                }
            }
            {live(&said.read())}
        }
    }
}
