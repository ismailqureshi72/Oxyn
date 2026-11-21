//! AI feature launch (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::launch::{Launch, STEPS};

use super::switch;
use crate::aikit::{ico, key_of, live, mark};

/// AI feature launch: an in-product card introducing an AI feature. A looping mini demo (a typed
/// request, then the result) plays above a three-step tour whose dots autoplay until you take
/// over; each step has three benefits; a workspace switch only admins can turn on (pressing it
/// again while saving cancels); Try it now and Learn more finish the card.
#[component]
pub fn AiFeatureLaunch(
    /// Host clock.
    now: ClockMs,
    /// You are a workspace admin.
    #[props(default = true)]
    admin: bool,
    /// Already on for the workspace.
    #[props(default)]
    enabled: bool,
    /// Autoplay the tour.
    #[props(default = true)]
    auto_play: bool,
) -> Element {
    let mut fl = use_signal(move || Launch::new(admin, enabled, auto_play, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let l = fl.read().clone();
    let mut act = move |id: String| {
        let x = fl.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let step = l.step(n);
    let s = STEPS[step];
    let (typed, lines) = l.demo(n);
    let typed_text: String = s.2[0].chars().take(typed).collect();
    let saving = l.is_saving(n);
    rsx! {
        section { class: "oxaik oxbk oxbk-fl", "aria-label": "New: AI assist",
            div { class: "oxbk-fl-stage", "aria-hidden": "true",
                div { class: "oxbk-fl-demo",
                    p { class: "oxbk-fl-ask", {ico("spark", 12.0)} span { "{typed_text}" } if typed < s.2[0].chars().count() { i { class: "oxbk-fl-caret" } } }
                    if lines >= 1 { p { class: "oxbk-fl-out oxaik-fade", "{s.2[1]}" } }
                    if lines >= 2 { p { class: "oxbk-fl-meta oxaik-fade", {ico("check", 11.0)} "{s.2[2]}" } }
                }
            }
            div { class: "oxaik-row oxbk-fl-top",
                span { class: "oxaik-pill", "data-hue": "violet", {ico("spark", 11.0)} "New" }
                span { class: "oxaik-mute", "Step {step + 1} of 3" }
            }
            h2 { class: "oxaik-h oxbk-fl-title", "{s.0}" }
            p { class: "oxbk-fl-blurb", "{s.1}" }
            ul { class: "oxbk-fl-bens",
                for (k, b) in s.3.iter().enumerate() { li { key: "{step}-{k}", class: "oxaik-fade", {ico("check-circle", 13.0)} span { "{b}" } } }
            }
            div { class: "oxbk-fl-dots", role: "tablist", "aria-label": "Tour steps",
                onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") { e.prevent_default(); let x = fl.write().key(&k, n); if let Some(x) = x { said.set(x) } } },
                for i in 0..STEPS.len() {
                    button { key: "{i}", class: "oxbk-fl-dot", r#type: "button", role: "tab", "aria-selected": "{i == step}", "aria-label": "Step {i + 1}: {STEPS[i].0}", tabindex: if i == step { "0" } else { "-1" },
                        onclick: move |_| act(format!("fl-dot-{i}")),
                        i { style: if i == step { format!("--p:{}", l.progress(n)) } else { String::new() } } }
                }
            }
            div { class: "oxbk-fl-ws",
                {switch("Enable for my workspace", l.on(), EventHandler::new(move |_| act("fl-switch".into())))}
                div { class: "oxbk-fl-wst",
                    b { "Enable for my workspace" }
                    p { class: "oxaik-mute",
                        if !l.admin { {ico("lock", 11.0)} "Only admins can turn this on. Ask an admin." }
                        else if saving { "Saving… press again to cancel" }
                        else if l.on() { "On for everyone in Northwind" }
                        else { "Off · members can't use it yet" }
                    }
                }
            }
            footer { class: "oxaik-row oxbk-fl-foot",
                button { class: "oxaik-btn", r#type: "button", onclick: move |_| act("fl-learn".into()), "Learn more" }
                button { class: "oxaik-btn pri", r#type: "button", onclick: move |_| act("fl-try".into()), {ico("spark", 12.0)} "Try it now" }
                span { class: "oxaik-mute oxbk-fl-credit", {mark("claude", 12.0)} "Powered by Claude" }
            }
            {live(&said.read())}
        }
    }
}
