//! Prompt studio (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::prompt::{
    CASES, EFFORT, MODELS, Menu, Part, Studio, VARS, VERSIONS, diff, parts,
};

use crate::aikit::{cls, cmd, dots, ico, key_of, live, mark};

/// Prompt studio: one card to edit, test and ship a prompt template. A version chip (a menu of
/// versions with production marked, promote and undo) and a model chip (model and effort) sit in
/// the header; System and User tabs show the template with `{{variable}}` placeholders as chips
/// in each variable's series colour; a coloured field per variable fills them from saved test
/// cases; Run streams the reply with latency, tokens and cost; Compare shows a word diff against
/// production.
#[component]
pub fn PromptStudio(
    /// Host clock.
    now: ClockMs,
    /// Run the first test case at once.
    #[props(default = true)]
    auto_play: bool,
    /// The first run fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Open in compare mode.
    #[props(default)]
    compare: bool,
) -> Element {
    let mut st = use_signal(move || {
        let mut s = Studio::new(auto_play, fail, now.0);
        s.compare = compare;
        s
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = st.read().clone();
    let mut act = move |id: String| {
        let x = st.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut keyed = move |k: String| {
        let x = st.write().key(&k, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let running = s.running(n);
    let used = s.used();
    let empty = s.empty();
    let v = VERSIONS[s.version];
    let is_prod = s.version == s.prod;
    let latest = s.version == VERSIONS.len() - 1;
    let tpl = s.template(s.version);
    rsx! {
        section { class: "oxaik oxbk oxbk-ps", "aria-label": "Prompt studio",
            onkeydown: move |e: KeyboardEvent| if cmd(&e) && key_of(&e) == "Enter" { e.prevent_default(); keyed("run".into()) },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h2 { class: "oxaik-h", "Support reply" }
                    p { class: "oxaik-sub", "Prompt template · production serves {VERSIONS[s.prod].0}" }
                }
                div { class: "oxaik-row oxbk-ps-chips",
                    div { class: "oxbk-ps-mw",
                        button { class: "oxaik-chip oxbk-ps-chip", r#type: "button", "aria-haspopup": "true", "aria-expanded": "{s.menu == Some(Menu::Version)}",
                            onclick: move |_| act("ps-ver".into()),
                            b { "{v.0}" } span { class: "oxaik-pill", "data-hue": if is_prod { "green" } else { "neutral" }, if is_prod { "Production" } else if latest { "Latest" } else { "Draft" } } {ico("chevron", 11.0)}
                        }
                        if s.menu == Some(Menu::Version) {
                            div { class: "oxbk-menu oxbk-ps-menu", role: "radiogroup", "aria-label": "Versions",
                                onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "Escape") { e.prevent_default(); keyed(k) } },
                                for (i, ver) in VERSIONS.iter().enumerate() {
                                    button { key: "{i}", class: cls("oxbk-ps-mi", i == s.version), r#type: "button", role: "radio", "aria-checked": "{i == s.version}",
                                        onclick: move |_| act(format!("ps-ver-{i}")),
                                        b { "{ver.0}" } span { class: "oxaik-mute", "{ver.1}" } if i == s.prod { span { class: "oxaik-pill", "data-hue": "green", "Production" } }
                                    }
                                }
                                div { class: "oxaik-row oxbk-ps-macts",
                                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ps-compare".into()), {ico("eye", 12.0)} "Compare" }
                                    button { class: "oxaik-btn sm pri", r#type: "button", disabled: is_prod, onclick: move |_| act("ps-promote".into()), {ico("up", 12.0)} "Promote" }
                                }
                            }
                        }
                    }
                    div { class: "oxbk-ps-mw",
                        button { class: "oxaik-chip oxbk-ps-chip", r#type: "button", "aria-haspopup": "true", "aria-expanded": "{s.menu == Some(Menu::Model)}",
                            onclick: move |_| act("ps-model".into()),
                            {mark(MODELS[s.model].1, 13.0)} span { "{MODELS[s.model].0}" } span { class: "oxaik-mute", "· {EFFORT[s.effort]}" } {ico("chevron", 11.0)}
                        }
                        if s.menu == Some(Menu::Model) {
                            div { class: "oxbk-menu oxbk-ps-menu oxbk-ps-mr",
                                onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Escape") { e.prevent_default(); keyed(k) } },
                                div { role: "radiogroup", "aria-label": "Model",
                                    for (i, m) in MODELS.iter().enumerate() {
                                        button { key: "{i}", class: cls("oxbk-ps-mi", i == s.model), r#type: "button", role: "radio", "aria-checked": "{i == s.model}",
                                            onclick: move |_| act(format!("ps-model-{i}")), {mark(m.1, 14.0)} span { "{m.0}" } }
                                    }
                                }
                                p { class: "oxbk-cap", "Effort" }
                                div { class: "oxaik-seg", role: "radiogroup", "aria-label": "Effort",
                                    for (i, e) in EFFORT.iter().enumerate() {
                                        button { key: "{i}", class: cls("oxaik-sg", i == s.effort), r#type: "button", role: "radio", "aria-checked": "{i == s.effort}", onclick: move |_| act(format!("ps-effort-{i}")), "{e}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(u) = s.undo {
                div { class: "oxbk-undo oxaik-row oxaik-fade", {ico("check-circle", 12.0)} span { "{VERSIONS[s.prod].0} is in production (was {VERSIONS[u].0})." }
                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ps-undo".into()), {ico("undo", 12.0)} "Undo" } }
            }
            div { class: "oxaik-row oxbk-ps-bar",
                div { class: "oxbk-tabs oxbk-ps-tabs", role: "tablist", "aria-label": "Message",
                    onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") { e.prevent_default(); keyed(k) } },
                    for (i, t) in ["System", "User"].iter().enumerate() {
                        button { key: "{i}", class: cls("oxbk-tab", s.tab == i), r#type: "button", role: "tab", "aria-selected": "{s.tab == i}", "aria-controls": "oxbk-ps-ed", tabindex: if s.tab == i { "0" } else { "-1" },
                            onclick: move |_| act(format!("ps-tab-{i}")), "{t}" }
                    }
                }
                button { class: cls("oxaik-chip", s.compare), r#type: "button", "aria-pressed": "{s.compare}", onclick: move |_| act("ps-compare".into()),
                    {ico("eye", 11.0)} if s.compare { "Comparing with {VERSIONS[s.prod].0}" } else { "Compare" } }
            }
            div { id: "oxbk-ps-ed", class: "oxbk-ps-ed", role: "tabpanel", tabindex: "0", "aria-label": if s.tab == 0 { "System prompt" } else { "User prompt" },
                if s.compare {
                    p { class: "oxbk-ps-diff",
                        for (k, (op, w)) in diff(s.template(s.prod), tpl).into_iter().enumerate() {
                            if op > 0 { ins { key: "{k}", span { class: "oxaik-sr", "Added " } "+{w} " } }
                            else if op < 0 { del { key: "{k}", span { class: "oxaik-sr", "Removed " } "−{w} " } }
                            else { span { key: "{k}", "{w} " } }
                        }
                    }
                } else {
                    p { class: "oxbk-ps-tpl",
                        for (k, p) in parts(tpl).into_iter().enumerate() {
                            match p {
                                Part::Text(t) => rsx! { span { key: "{k}", "{t}" } },
                                Part::Var(i) => rsx! { span { key: "{k}", class: "oxbk-ps-var", "data-hue": "{VARS[i].1}", "data-empty": "{empty.contains(&i)}", "{{{{{VARS[i].0}}}}}" } },
                            }
                        }
                    }
                }
            }
            div { class: "oxbk-ps-fields",
                for i in used.iter().copied() {
                    label { key: "{i}", class: "oxbk-ps-field", "data-hue": "{VARS[i].1}",
                        span { class: "oxbk-ps-fl", i { class: "oxaik-dot" } "{VARS[i].0}" }
                        input { class: "oxaik-in", value: "{s.values[i]}", "aria-invalid": "{empty.contains(&i)}",
                            oninput: move |e| st.write().set_value(i, &e.value()) }
                    }
                }
            }
            div { class: "oxaik-row oxbk-ps-run",
                div { class: "oxaik-seg", role: "radiogroup", "aria-label": "Test case",
                    for (i, c) in CASES.iter().enumerate() {
                        button { key: "{i}", class: cls("oxaik-sg", i == s.case), r#type: "button", role: "radio", "aria-checked": "{i == s.case}", tabindex: if i == s.case { "0" } else { "-1" },
                            onclick: move |_| act(format!("ps-case-{i}")), "{c.0}" }
                    }
                }
                button { class: if running { "oxaik-btn" } else { "oxaik-btn pri" }, r#type: "button", "aria-keyshortcuts": "Control+Enter", onclick: move |_| act("ps-run".into()),
                    {ico(if running { "stop" } else { "play" }, 12.0)} if running { "Stop" } else { "Run" } span { class: "oxaik-kbd oxbk-ps-kbd", "⌘↵" } }
            }
            if s.run.is_some() {
                div { class: "oxbk-ps-out oxaik-fade",
                    if s.failed {
                        p { class: "oxbk-mcp-err", {ico("x-circle", 12.0)} "Run failed: the model returned an error."
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ps-retry".into()), {ico("retry", 12.0)} "Retry" } }
                    } else {
                        div { class: "oxaik-row oxbk-ps-oh",
                            span { class: "oxbk-st", "data-hue": if running { "blue" } else if s.stopped.is_some() { "neutral" } else { "green" },
                                {ico(if running { "pencil" } else if s.stopped.is_some() { "stop" } else { "check-circle" }, 12.0)}
                                span { if running { "Writing" } else if s.stopped.is_some() { "Stopped" } else { "Reply" } } }
                            if let Some((lat, toks, cost)) = s.stats(n) { span { class: "oxaik-mute oxaik-num", "{lat} · {toks} tokens · {cost}" } }
                            button { class: "oxaik-ib oxbk-ps-copy", r#type: "button", "aria-label": "Copy reply", onclick: move |_| act("ps-copy".into()), {ico("copy", 12.0)} }
                        }
                        p { class: "oxbk-ps-reply", {mark(MODELS[s.model].1, 13.0)} " {s.reply(n)}" if running { {dots()} } }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
