//! Provider keys (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::keys::{KeySt, Keys, PROVIDERS};
use oxyn_core::aikit::usd;

use super::{dots, ico, key_of, live, mark};

/// Provider keys: a bring-your-own-key settings card. Each row shows the provider's mark, the
/// masked key, a status and when it was last used; Reveal shows the key for five seconds, Copy,
/// Rotate and Remove (with Undo) act on it. Add key opens an inline form that tests the key
/// before Save. Spend, rate-limit use and project scope per key are Oxyn extras.
#[component]
pub fn ProviderKeys(
    /// Host clock.
    now: ClockMs,
    /// Start with no keys (empty state).
    #[props(default)]
    empty: bool,
    /// Show spend and rate-limit use per key (Oxyn extra).
    #[props(default = true)]
    usage: bool,
    /// Open the add form on mount.
    #[props(default)]
    adding: bool,
) -> Element {
    let t0 = now.0;
    let mut pk = use_signal(move || {
        let mut k = Keys::new(empty);
        if adding {
            k.press("pk-add", t0);
        }
        k
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let k = pk.read().clone();
    let mut act = move |id: String| {
        let x = pk.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let undo = k.can_undo(n);
    rsx! {
        section { class: "oxaik oxaik-pk", "aria-label": "Provider keys",
            onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Escape" { act("pk-cancel".into()) } },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Provider keys" } p { class: "oxaik-sub", "Bring your own keys. They are encrypted and only used for your workspace." } }
                button { class: "oxaik-btn sm", r#type: "button", disabled: k.draft.is_some() || k.free_providers().is_empty(), onclick: move |_| act("pk-add".into()), {ico("plus", 12.0)} "Add key" }
            }
            if k.keys.is_empty() && k.draft.is_none() {
                div { class: "oxaik-err oxaik-pk-empty", {ico("key", 18.0)} b { "No keys yet" } span { class: "oxaik-mute", "Add a key from OpenAI, Anthropic, Google and others to use your own quota." } }
            }
            ul { class: "oxaik-pk-list",
                for (i, key) in k.keys.iter().enumerate() {
                    {
                        let p = PROVIDERS[key.provider];
                        let shown = k.revealed(i, n);
                        let secret = if shown { key.secret.clone() } else { key.masked() };
                        let (hue, st) = match key.status { KeySt::Active => ("green", "Active"), KeySt::Invalid => ("red", "Invalid"), KeySt::Limited => ("amber", "Near limit") };
                        let rot = k.rotating_now(i, n);
                        let left = k.reveal_left(n);
                        rsx! {
                            li { key: "{p.0}", class: "oxaik-pk-row",
                                span { class: "oxaik-pk-mk", {mark(p.0, 18.0)} }
                                div { class: "oxaik-pk-main",
                                    div { class: "oxaik-row", b { "{p.1}" } span { class: "oxaik-pill", "data-hue": "{hue}", if key.status == KeySt::Invalid { {ico("alert", 11.0)} } else { span { class: "oxaik-dot" } } "{st}" } }
                                    code { class: "oxaik-mono oxaik-pk-key", "aria-label": if shown { format!("{} key, shown", p.1) } else { format!("{} key, hidden", p.1) }, "{secret}" }
                                    span { class: "oxaik-mute oxaik-num",
                                        if rot { "Rotated just now · the old key stops in 24h" } else { "Used {key.used}" }
                                        if usage { " · {usd(key.spend)} this month · {key.rpm.0}/{key.rpm.1} rpm · {key.scope}" }
                                    }
                                }
                                div { class: "oxaik-pk-acts",
                                    button { class: "oxaik-ib", r#type: "button", "aria-label": if shown { "Hide key" } else { "Reveal key for 5 seconds" }, "aria-pressed": "{shown}", onclick: move |_| act(format!("pk-reveal-{i}")),
                                        if shown { span { class: "oxaik-num oxaik-pk-left", "{left}" } } else { {ico("eye", 13.0)} }
                                    }
                                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Copy {p.1} key", onclick: move |_| act(format!("pk-copy-{i}")), {ico("copy", 13.0)} }
                                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Rotate {p.1} key", onclick: move |_| act(format!("pk-rotate-{i}")), {ico("retry", 13.0)} }
                                    button { class: "oxaik-ib oxaik-pk-rm", r#type: "button", "aria-label": "Remove {p.1} key", onclick: move |_| act(format!("pk-remove-{i}")), {ico("trash", 13.0)} }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(d) = k.draft.clone() {
                {
                    let testing = k.testing(n);
                    let res = k.test_result(n);
                    let p = PROVIDERS[d.provider];
                    let free = k.free_providers();
                    rsx! {
                        form { class: "oxaik-pk-form oxaik-fade", onsubmit: move |e| { e.prevent_default(); act("pk-save".into()) },
                            div { class: "oxaik-row", role: "radiogroup", "aria-label": "Provider",
                                for q in free {
                                    button { key: "{q}", class: if q == d.provider { "oxaik-chip on" } else { "oxaik-chip" }, r#type: "button", role: "radio", "aria-checked": "{q == d.provider}", onclick: move |_| act(format!("pk-prov-{q}")),
                                        {mark(PROVIDERS[q].0, 13.0)} "{PROVIDERS[q].1}"
                                    }
                                }
                            }
                            input { class: "oxaik-in oxaik-mono", r#type: "password", "aria-label": "{p.1} API key", placeholder: "{p.2}…", value: "{d.secret}", autocomplete: "off",
                                "aria-invalid": "{res == Some(false)}",
                                oninput: move |e| act(format!("pk-input:{}", e.value())),
                            }
                            div { class: "oxaik-row oxaik-between",
                                span { class: "oxaik-mute oxaik-pk-msg",
                                    if testing { {dots()} "Testing with a 1-token request…" }
                                    else if res == Some(true) { span { class: "oxaik-ok oxaik-row", {ico("check-circle", 12.0)} "Key works" } }
                                    else if res == Some(false) { span { class: "oxaik-bad oxaik-row", {ico("alert", 12.0)} "{p.1} rejected this key" } }
                                    else { "Keys start with {p.2}" }
                                }
                                div { class: "oxaik-row",
                                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("pk-cancel".into()), "Cancel" }
                                    button { class: "oxaik-btn sm", r#type: "button", disabled: d.secret.is_empty() || testing, onclick: move |_| act("pk-test".into()), "Test" }
                                    button { class: "oxaik-btn sm pri", r#type: "submit", disabled: res != Some(true), "Save" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-row oxaik-between oxaik-pk-foot",
                span { class: "oxaik-mute oxaik-num", "{k.footer()}" }
                if undo {
                    span { class: "oxaik-row oxaik-pk-undo", role: "status",
                        span { "Key removed" }
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("pk-undo".into()), {ico("undo", 12.0)} "Undo" }
                    }
                }
            }
            {live(&said())}
        }
    }
}
