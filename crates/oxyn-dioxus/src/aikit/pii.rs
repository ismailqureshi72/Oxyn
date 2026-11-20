//! PII redaction (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::first_words;
use oxyn_core::aikit::pii::{Phase, Pii, TYPES, placeholder};

use super::{cls, cmd, dots, ico, key_of, live, mark};

/// PII redaction: a guardrail card where personal data is redacted before the prompt reaches
/// the model. Scan lights each name, email, phone number and card in turn with its type; Redact
/// swaps them for placeholders like `[EMAIL_1]` in one crossfade; Send passes the safe prompt and
/// the reply streams back with the values restored for the reader (Show original off keeps the
/// placeholders). Category chips choose what is redacted (Oxyn extra). Cmd/Ctrl+Enter advances.
#[component]
pub fn PiiRedaction(
    /// Host clock.
    now: ClockMs,
    /// Scan on mount.
    #[props(default = true)]
    autoplay: bool,
    /// Let the reader edit the message first.
    #[props(default = true)]
    editable: bool,
    /// Show the category toggles (Oxyn extra).
    #[props(default)]
    categories: bool,
) -> Element {
    let t0 = now.0;
    let mut pi = use_signal(move || Pii::new(autoplay, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let p = pi.read().clone();
    let mut act = move |id: String| {
        let x = pi.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = p.phase_at(n);
    let lit = p.lit(n);
    let counts = p.counts();
    let segs = p.segments();
    let mut seen = 0usize;
    let ords: Vec<usize> = segs
        .iter()
        .map(|(_, e)| {
            let o = seen;
            if e.is_some() {
                seen += 1;
            }
            o
        })
        .collect();
    let redacted = matches!(ph, Phase::Redacted | Phase::Replying | Phase::Done);
    let next = match ph {
        Phase::Edit => Some(("pii-scan", "Scan")),
        Phase::Found => Some(("pii-redact", "Redact")),
        Phase::Redacted => Some(("pii-send", "Send safe prompt")),
        Phase::Done => Some(("pii-reset", "Edit message")),
        _ => None,
    };
    let reply = if p.show_original {
        p.reply_restored()
    } else {
        p.reply_safe()
    };
    let rk = p.reply_words(n);
    rsx! {
        section { class: "oxaik oxaik-pii", "aria-label": "PII redaction",
            onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Enter" && cmd(&e) { e.prevent_default(); if let Some((id, _)) = next { act(id.into()) } } },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Redact before sending" } p { class: "oxaik-sub", "Personal data never reaches the model; the reply is restored for you." } }
                button { class: cls("oxaik-switch", p.show_original), r#type: "button", role: "switch", "aria-checked": "{p.show_original}", onclick: move |_| act("pii-orig".into()),
                    span { class: "oxaik-knob" } "Show original"
                }
            }
            if categories {
                div { class: "oxaik-row", role: "group", "aria-label": "What to redact",
                    for (i, t) in TYPES.iter().enumerate() {
                        button { key: "{i}", class: cls("oxaik-chip", p.on[i]), r#type: "button", "aria-pressed": "{p.on[i]}", "data-hue": "{t.3}", onclick: move |_| act(format!("pii-type-{i}")),
                            span { class: "oxaik-dot" } "{t.1}s"
                        }
                    }
                }
            }
            div { class: "oxaik-pii-msg",
                span { class: "oxaik-mute oxaik-pii-lab", "Your message" }
                if ph == Phase::Edit && editable {
                    textarea { class: "oxaik-ta", "aria-label": "Message", rows: "4", value: "{p.text}", oninput: move |e| act(format!("pii-edit:{}", e.value())) }
                } else {
                    p { class: "oxaik-pii-text", "data-red": "{redacted}",
                        for (k, (t, e)) in segs.iter().enumerate() {
                            {match e {
                                Some(e) => {
                                    let on = lit == usize::MAX || ords[k] < lit;
                                    let ph_text = placeholder(e);
                                    let ty = TYPES[e.kind];
                                    rsx! {
                                        span { key: "{k}", class: cls("oxaik-pii-ent", on), "data-hue": "{ty.3}", title: "{ty.1}", "aria-label": if redacted { format!("{}, redacted", ty.1) } else { format!("{t}, {}", ty.1) },
                                            span { class: "oxaik-pii-a", "{t}" }
                                            span { class: "oxaik-pii-b oxaik-mono", "{ph_text}" }
                                        }
                                    }
                                }
                                None => rsx! { span { key: "{k}", "{t}" } },
                            }}
                        }
                    }
                }
            }
            ul { class: "oxaik-pii-types",
                for (i, t) in TYPES.iter().enumerate() {
                    li { key: "{i}", "data-hue": "{t.3}",
                        span { class: "oxaik-dot" } span { "{t.1}" }
                        b { class: "oxaik-num",
                            if ph == Phase::Scanning && lit != usize::MAX { span { class: "oxaik-shim", "Checking" } }
                            else if ph == Phase::Edit { "—" }
                            else if !p.on[i] { "Kept" }
                            else { "{counts[i]}" }
                        }
                    }
                }
            }
            if matches!(ph, Phase::Replying | Phase::Done) {
                div { class: "oxaik-pii-reply oxaik-fade",
                    div { class: "oxaik-row oxaik-mute", {mark("claude", 13.0)} "Claude Haiku 4.5 saw only placeholders" }
                    if rk == 0 { p { class: "oxaik-row", {dots()} span { class: "oxaik-shim", "Thinking" } } }
                    else { p { "{first_words(&reply, rk)}" } }
                }
            }
            div { class: "oxaik-row oxaik-between",
                span { class: "oxaik-mute oxaik-mono oxaik-pii-safe", if redacted { "{p.safe()}" } else if ph == Phase::Scanning { "Scanning…" } else { "Nothing sent yet" } }
                if let Some((id, label)) = next {
                    button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(id.into()), if id == "pii-redact" { {ico("eye-off", 12.0)} } else if id == "pii-send" { {ico("up", 12.0)} } else if id == "pii-scan" { {ico("search", 12.0)} } "{label}" }
                } else { span { class: "oxaik-row oxaik-mute", {dots()} "Working" } }
            }
            {live(&said())}
        }
    }
}
