//! Reply draft (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::reply::{LENGTHS, MESSAGE, Phase, Reply, SUGGESTIONS, TONES};

use super::{cls, cmd, dots, ico, key_of, live, mark, seg};

/// Reply draft: a received email with an AI reply under it. Quick replies steer a fresh draft
/// that streams in; tone and length rewrite it with one crossfade (the old draft dims meanwhile);
/// the finished draft is editable; Regenerate writes another and Insert (Cmd/Ctrl+Enter) hands it
/// over. Past drafts can be restored (Oxyn extra).
#[component]
pub fn ReplyDraft(
    /// Host clock.
    now: ClockMs,
    /// Start the first draft on mount.
    #[props(default = true)]
    autostart: bool,
    /// Starting tone (0 friendly, 1 concise, 2 formal).
    #[props(default)]
    tone: usize,
    /// Starting length (0 short, 1 medium, 2 long).
    #[props(default = 1)]
    length: usize,
    /// Show the draft history (Oxyn extra).
    #[props(default)]
    history: bool,
) -> Element {
    let t0 = now.0;
    let mut rd = use_signal(move || {
        let mut r = Reply::new(false, t0);
        r.tone = tone.min(2);
        r.len = length.min(2);
        r.draft = oxyn_core::aikit::reply::compose(r.sug, r.tone, r.len);
        if autostart {
            r.press("rd-draft", t0);
        }
        r
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let r = rd.read().clone();
    let mut act = move |id: String| {
        let x = rd.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = r.phase(n);
    let (from, addr, subj, body, recv) = MESSAGE;
    let text = r.visible(n);
    let busy = matches!(ph, Phase::Thinking | Phase::Streaming | Phase::Rewriting);
    let label = match ph {
        Phase::Empty => "No draft yet",
        Phase::Thinking => "Drafting",
        Phase::Streaming => "Writing",
        Phase::Rewriting => "Rewriting",
        Phase::Ready => "Draft ready",
        Phase::Inserted => "Inserted",
    };
    rsx! {
        article { class: "oxaik oxaik-rd", "aria-label": "Email from {from}",
            onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Enter" && cmd(&e) { e.prevent_default(); let x = rd.write().key("insert", "Enter", n); if let Some(x) = x { said.set(x) } } },
            div { class: "oxaik-rd-mail",
                div { class: "oxaik-row oxaik-rd-from",
                    span { class: "oxaik-av", "MI" }
                    div { class: "oxaik-hl", b { "{from}" } span { class: "oxaik-mute", "{addr}" } }
                    span { class: "oxaik-row oxaik-mute oxaik-rd-recv", {mark("gmail", 13.0)} "{recv}" }
                }
                h3 { class: "oxaik-h", "{subj}" }
                for (i, p) in body.split("\n\n").enumerate() { p { key: "{i}", class: "oxaik-rd-p", "{p}" } }
            }
            section { class: "oxaik-rd-reply", "aria-label": "Reply", "aria-busy": "{busy}",
                div { class: "oxaik-row oxaik-rd-sugs",
                    for (i, s) in SUGGESTIONS.iter().enumerate() {
                        button { key: "{i}", class: cls("oxaik-chip", i == r.sug && ph != Phase::Empty), r#type: "button", "aria-pressed": "{i == r.sug && ph != Phase::Empty}",
                            onclick: move |_| act(format!("rd-sug-{i}")), "{s}"
                        }
                    }
                }
                div { class: "oxaik-rd-box", "data-ph": "{label}",
                    div { class: "oxaik-row oxaik-rd-meta",
                        {mark("claude", 13.0)} span { "Claude Sonnet 4.5" }
                        span { class: "oxaik-rd-st", if busy { {dots()} span { class: "oxaik-shim", "{label}" } } else { "{label}" } }
                    }
                    if ph == Phase::Empty {
                        div { class: "oxaik-rd-empty", button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("rd-draft".into()), {ico("spark", 12.0)} "Draft a reply" } }
                    } else if ph == Phase::Ready {
                        textarea { class: "oxaik-rd-text", "aria-label": "Draft reply", rows: "7", value: "{text}",
                            oninput: move |e| act(format!("rd-edit:{}", e.value())),
                        }
                    } else {
                        div { class: "oxaik-rd-text", "data-dim": "{ph == Phase::Rewriting}", "{text}" }
                    }
                }
                div { class: "oxaik-row oxaik-rd-opts",
                    {seg("Tone", TONES.iter().map(|x| x.to_string()).collect(), r.tone, EventHandler::new(move |i| act(format!("rd-tone-{i}"))), EventHandler::new(move |k: String| { let x = rd.write().key("tone", &k, n); if let Some(x) = x { said.set(x) } }))}
                    span { class: "oxaik-rd-div", "aria-hidden": "true" }
                    {seg("Length", LENGTHS.iter().map(|x| x.to_string()).collect(), r.len, EventHandler::new(move |i| act(format!("rd-len-{i}"))), EventHandler::new(move |k: String| { let x = rd.write().key("len", &k, n); if let Some(x) = x { said.set(x) } }))}
                }
                if history && !r.history.is_empty() {
                    div { class: "oxaik-row oxaik-rd-hist",
                        span { class: "oxaik-mute", "Earlier drafts" }
                        for (i, _) in r.history.iter().enumerate() {
                            button { key: "{i}", class: "oxaik-chip oxaik-num", r#type: "button", onclick: move |_| act(format!("rd-hist-{i}")), "{i + 1}" }
                        }
                    }
                }
                div { class: "oxaik-row oxaik-between",
                    span { class: "oxaik-mute", if ph == Phase::Inserted { "Inserted, simulated, nothing was sent" } else { "⌘ Enter to insert" } }
                    div { class: "oxaik-row",
                        button { class: "oxaik-btn sm", r#type: "button", disabled: busy, onclick: move |_| act("rd-regen".into()), {ico("retry", 12.0)} "Regenerate" }
                        button { class: "oxaik-btn sm pri", r#type: "button", disabled: ph != Phase::Ready, onclick: move |_| act("rd-insert".into()),
                            if ph == Phase::Inserted { {ico("check", 12.0)} "Inserted" } else { "Insert" }
                        }
                    }
                }
            }
            {live(&said())}
        }
    }
}
