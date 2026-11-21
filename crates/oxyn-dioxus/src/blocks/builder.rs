//! Agent builder (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::builder::{Builder, FILES, MODELS, PROMPTS, TOOLS};

use super::{status, switch};
use crate::aikit::{cls, dots, ico, key_of, live, mark, seg};

/// Agent builder: set up one agent in a card. The name edits inline (Enter keeps, Escape
/// restores) over its workspace and version; a model picker (menu with radio items), the
/// instructions with a character count, tools as switch rows with their app marks, and knowledge
/// files with upload progress (a failed upload names its error and retries). The Configure / Test
/// switch swaps the body for a small chat that answers with the model and the tools you turned
/// on (Stop while it streams). Publish ships the setup; any later change reads as unsaved until
/// it is published again.
#[component]
pub fn AgentBuilder(
    /// Host clock.
    now: ClockMs,
    /// Start on the Test view.
    #[props(default)]
    testing: bool,
    /// The first publish fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Start with unsaved changes (Slack turned on).
    #[props(default)]
    unsaved: bool,
) -> Element {
    let mut ab = use_signal(move || {
        let mut b = Builder::new(fail, now.0);
        b.testing = testing;
        if unsaved {
            b.press("ab-tool-3", now.0);
        }
        b
    });
    let mut said = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut name_before = use_signal(String::new);
    let n = now.0;
    let b = ab.read().clone();
    let mut act = move |id: String| {
        let x = ab.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut send = move || {
        let t = draft.read().clone();
        let x = ab.write().send(&t, n);
        if let Some(x) = x {
            said.set(x);
            draft.set(String::new());
        }
    };
    let ps = b.publish_state(n);
    let ps_hue = match ps {
        "Published" => "green",
        "Publishing" => "blue",
        "Publish failed" => "red",
        _ => "amber",
    };
    let (mb, ml, mn) = MODELS[b.config.model];
    let streaming = b.streaming(n);
    let ini: String = b.config.name.chars().take(1).collect();
    let busy = ps == "Publishing";
    rsx! {
        section { class: "oxaik oxbk oxbk-ab", "aria-label": "Agent builder",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if matches!(k.as_str(), "Escape" | "ArrowUp" | "ArrowDown") && ab.read().menu {
                    e.prevent_default();
                    let x = ab.write().key(&k, n);
                    if let Some(x) = x { said.set(x) }
                }
            },
            header { class: "oxaik-hd oxbk-ab-hd",
                div { class: "oxaik-row oxbk-ab-id",
                    span { class: "oxbk-ab-av", "aria-hidden": "true", "{ini}" }
                    div { class: "oxaik-hl",
                        input { class: "oxbk-ab-name", r#type: "text", value: "{b.config.name}", "aria-label": "Agent name", maxlength: "40",
                            onfocus: move |_| { let v = ab.read().config.name.clone(); name_before.set(v) },
                            oninput: move |e| ab.write().rename(&e.value()),
                            onkeydown: move |e: KeyboardEvent| match key_of(&e).as_str() {
                                "Escape" => { let v = name_before.read().clone(); ab.write().rename(&v); }
                                "Enter" => { said.set(format!("Name set to {}", ab.read().config.name)); }
                                _ => {}
                            },
                        }
                        p { class: "oxaik-sub", "Northwind · v{b.version}" }
                    }
                }
                {seg("View", vec!["Configure".into(), "Test".into()], usize::from(b.testing),
                    EventHandler::new(move |v| act(format!("ab-view-{v}"))),
                    EventHandler::new(move |k: String| { let x = ab.write().key(&k, n); if let Some(x) = x { said.set(x) } }))}
            }
            if !b.testing {
                div { class: "oxbk-ab-body",
                    div { class: "oxbk-ab-f",
                        span { class: "oxbk-cap", "Model" }
                        div { class: "oxbk-cm",
                            button { class: "oxaik-chip oxbk-ab-model", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{b.menu}", "aria-label": "Model: {ml}",
                                onclick: move |_| act("ab-menu".into()),
                                {mark(mb, 13.0)} span { "{ml}" } span { class: "oxaik-mute", "{mn}" } {ico("chevron", 11.0)}
                            }
                            if b.menu {
                                ul { class: "oxbk-menu", role: "menu", "aria-label": "Model",
                                    for (i, (br, l, note)) in MODELS.iter().enumerate() {
                                        li { key: "{i}", role: "none",
                                            button { class: cls("oxbk-mi", i == b.config.model), r#type: "button", role: "menuitemradio", "aria-checked": "{i == b.config.model}",
                                                onclick: move |_| act(format!("ab-model-{i}")),
                                                {mark(br, 13.0)} span { "{l}" } span { class: "oxaik-mute", "{note}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxbk-ab-f",
                        label { class: "oxbk-cap oxaik-row oxaik-between", r#for: "oxbk-ab-ins", span { "Instructions" } span { class: "oxaik-mute oxaik-num", "{b.config.instructions.chars().count()} / 1,200" } }
                        textarea { class: "oxaik-ta", id: "oxbk-ab-ins", rows: "3", value: "{b.config.instructions}", oninput: move |e| ab.write().instruct(&e.value()) }
                    }
                    div { class: "oxbk-ab-f",
                        span { class: "oxbk-cap", "Tools" }
                        ul { class: "oxbk-ab-tools",
                            for (i, (br, name, what)) in TOOLS.iter().enumerate() {
                                li { key: "{i}", class: "oxbk-ab-tool", "data-on": "{b.config.tools[i]}",
                                    {mark(br, 18.0)}
                                    span { class: "oxbk-ab-tt", b { "{name}" } span { class: "oxaik-mute", "{what}" } }
                                    {switch(&format!("{name}: {what}"), b.config.tools[i], EventHandler::new(move |_| act(format!("ab-tool-{i}"))))}
                                }
                            }
                        }
                    }
                    div { class: "oxbk-ab-f",
                        span { class: "oxbk-cap", "Knowledge" }
                        ul { class: "oxbk-ab-files",
                            for (i, (f, kb)) in FILES.iter().enumerate() {
                                {
                                    let (p, failed) = b.upload(i, n);
                                    rsx! {
                                        li { key: "{i}", class: "oxbk-ab-file", "data-failed": "{failed}",
                                            {ico("file", 14.0)}
                                            span { class: "oxbk-ab-fn", b { title: "{f}", "{f}" } span { class: "oxaik-mute oxaik-num", "{kb} KB" } }
                                            if failed {
                                                span { class: "oxbk-ab-err", {ico("alert", 12.0)} "Upload failed: the connection dropped" }
                                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ab-retry-{i}")), {ico("retry", 12.0)} "Retry" }
                                            } else if p < 1.0 {
                                                span { class: "oxaik-bar oxbk-ab-up", role: "progressbar", "aria-label": "Uploading {f}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{(p * 100.0) as u32}", span { style: "width:{p * 100.0}%" } }
                                            } else {
                                                {status("Ready", "green")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxbk-ab-test",
                    ol { class: "oxbk-ab-thread oxaik-scroll", "aria-live": "polite",
                        if b.chat.is_empty() {
                            li { class: "oxaik-mute oxbk-empty", "Try {b.config.name} with one of these:" }
                        }
                        for (i, (q, _)) in b.chat.iter().enumerate() {
                            {
                                let (_, shown) = b.reply(i, n);
                                rsx! {
                                    li { class: "oxbk-ab-me", "{q}" }
                                    li { class: "oxbk-ab-bot",
                                        span { class: "oxbk-ab-av sm", "aria-hidden": "true", "{ini}" }
                                        if shown.is_empty() { span { class: "oxaik-mute", {dots()} } } else { span { "{shown}" } }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxaik-row oxbk-kb-sugs",
                        for (i, p) in PROMPTS.iter().enumerate() {
                            button { key: "{i}", class: "oxaik-chip", r#type: "button", disabled: streaming, onclick: move |_| act(format!("ab-prompt-{i}")), "{p}" }
                        }
                    }
                    div { class: "oxbk-ls-comp",
                        input { class: "oxaik-in", r#type: "text", value: "{draft}", placeholder: "Message {b.config.name}", "aria-label": "Test message",
                            oninput: move |e| draft.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Enter" { e.prevent_default(); send() },
                        }
                        if streaming {
                            button { class: "oxaik-ib", r#type: "button", "aria-label": "Stop", onclick: move |_| act("ab-stop".into()), {ico("stop", 13.0)} }
                        } else {
                            button { class: "oxaik-ib", r#type: "button", "aria-label": "Send", onclick: move |_| send(), {ico("up", 14.0)} }
                        }
                    }
                }
            }
            footer { class: "oxaik-row oxaik-between oxbk-ab-foot",
                {status(ps, ps_hue)}
                button { class: "oxaik-btn sm pri", r#type: "button", disabled: ps == "Published" || ps == "Publishing", "aria-busy": "{busy}",
                    onclick: move |_| act("ab-publish".into()),
                    if ps == "Publishing" { {dots()} "Publishing" } else if ps == "Publish failed" { "Try again" } else { "Publish" }
                }
            }
            {live(&said.read())}
        }
    }
}
