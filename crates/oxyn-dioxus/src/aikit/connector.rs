//! Connector setup (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::connector::{APPS, ConnSt, Connectors};

use super::{dots, ico, key_of, live, mark};

/// Connector setup: the "Connect your tools" step of an assistant. Every app is linked without
/// leaving the card: a redirect, a wait for the grant, then an initial import with a thin bar and a
/// counter; afterwards it lists what access was given and can unlink the app after a confirming
/// press (Escape backs out). A refused grant offers Retry, and search narrows the list.
#[component]
pub fn ConnectorSetup(
    /// Host clock.
    now: ClockMs,
    /// The first HubSpot grant is denied (Retry demo).
    #[props(default)]
    deny: bool,
    /// Show the search field.
    #[props(default = true)]
    searchable: bool,
    /// Connect Notion on mount.
    #[props(default = true)]
    autoplay: bool,
    /// Assistant name in the heading.
    #[props(default = "Corvel Assistant".to_string())]
    assistant: String,
) -> Element {
    let t0 = now.0;
    let mut cs = use_signal(move || {
        let mut c = Connectors::new(deny);
        if autoplay {
            c.press("cs-connect-2", t0);
            if deny {
                c.press("cs-connect-6", t0);
            }
        }
        c
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let c = cs.read().clone();
    let mut act = move |id: String| {
        let x = cs.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    rsx! {
        section { class: "oxaik oxaik-cs", "aria-label": "Connect your tools",
            onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Escape" { let x = cs.write().key("Escape", n); if let Some(x) = x { said.set(x) } } },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Connect your tools" } p { class: "oxaik-sub", "{assistant} can read and act in the apps you connect. {c.connected(n)} of {APPS.len()} connected." } }
            }
            if searchable {
                input { class: "oxaik-in", r#type: "search", "aria-label": "Search apps", placeholder: "Search apps", value: "{c.query}", oninput: move |e| act(format!("cs-q:{}", e.value())) }
            }
            ul { class: "oxaik-cs-list",
                if c.rows().is_empty() { li { class: "oxaik-mute oxaik-cs-none", "No apps match" } }
                for i in c.rows() {
                    {
                        let a = APPS[i];
                        let st = c.st(i, n);
                        let line = c.line(i, n);
                        let prog = match st { ConnSt::Redirecting => 0.08, ConnSt::Authorizing => 0.18, ConnSt::Syncing(f) => 0.2 + f * 0.8, _ => 0.0 };
                        let open = c.open == Some(i) && matches!(st, ConnSt::On | ConnSt::Confirm);
                        rsx! {
                            li { key: "{a.0}", class: "oxaik-cs-row", "data-st": match st { ConnSt::On => "on", ConnSt::Denied => "bad", ConnSt::Off => "off", ConnSt::Confirm => "confirm", _ => "busy" },
                                div { class: "oxaik-cs-top",
                                    span { class: "oxaik-pk-mk", {mark(a.0, 18.0)} }
                                    div { class: "oxaik-hl",
                                        b { "{a.1}" }
                                        span { class: "oxaik-mute oxaik-num oxaik-cs-line", "{line}" }
                                    }
                                    div { class: "oxaik-row oxaik-cs-act",
                                        {match st {
                                            ConnSt::Off => rsx! { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("cs-connect-{i}")), "Connect" } },
                                            ConnSt::Redirecting | ConnSt::Authorizing | ConnSt::Syncing(_) => rsx! { span { class: "oxaik-pill", "data-hue": "blue", {dots()} "Connecting" } },
                                            ConnSt::On => rsx! {
                                                span { class: "oxaik-pill", "data-hue": "green", {ico("check", 11.0)} "Connected" }
                                                button { class: "oxaik-ib", r#type: "button", "aria-label": "Show {a.1} access", "aria-expanded": "{c.open == Some(i)}", onclick: move |_| act(format!("cs-open-{i}")), {ico("chevron", 12.0)} }
                                            },
                                            ConnSt::Denied => rsx! { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("cs-retry-{i}")), {ico("retry", 12.0)} "Retry" } },
                                            ConnSt::Confirm => rsx! {
                                                span { class: "oxaik-mute", "Disconnect?" }
                                                button { class: "oxaik-btn sm danger", r#type: "button", onclick: move |_| act(format!("cs-yes-{i}")), "Disconnect" }
                                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("cs-no".into()), "Keep" }
                                            },
                                        }}
                                    }
                                }
                                if prog > 0.0 { div { class: "oxaik-cs-hair", "aria-hidden": "true", span { style: "transform:scaleX({prog})" } } }
                                if st == ConnSt::Denied { p { class: "oxaik-bad oxaik-row oxaik-cs-note", role: "alert", {ico("alert", 12.0)} "{a.1} said access was not allowed. Ask an admin or try again." } }
                                if open {
                                    div { class: "oxaik-cs-scopes oxaik-fade",
                                        span { class: "oxaik-mute", "Granted" }
                                        ul { for s in a.5.iter() { li { key: "{s}", {ico("check", 11.0)} "{s}" } } }
                                        if st == ConnSt::On { button { class: "oxaik-btn sm danger", r#type: "button", onclick: move |_| act(format!("cs-disc-{i}")), "Disconnect" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxaik-mute", "Simulated: no real accounts are connected." }
            {live(&said())}
        }
    }
}
