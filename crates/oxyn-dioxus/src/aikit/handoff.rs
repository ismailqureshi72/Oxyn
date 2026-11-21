//! Human handoff (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::first_words;
use oxyn_core::aikit::handoff::{ASSISTANT, Handoff, PERSON, Stage, Who};

use super::{cls, dots, ico, key_of, live, mark};

/// Human handoff (Arc "Agent handoff"): a help chat in which the bot passes the conversation to a
/// human. Talk to a person opens a transfer card with the brief the bot hands over (editable first,
/// an Oxyn extra) and the usual wait with a queue position; Cancel returns to the bot; a teammate
/// arrives and carries on; Resolve ends it. A slim trail records each step. If nobody is online the
/// card offers to try later or stay with the bot.
#[component]
pub fn HumanHandoff(
    /// Host clock.
    now: ClockMs,
    /// `handoff` (starts with the assistant), `connected`, `unavailable` or `resolved`.
    #[props(default = "handoff".to_string())]
    scenario: String,
    /// Let the reader edit the summary before transfer (Oxyn extra).
    #[props(default)]
    edit_summary: bool,
    /// Show the stage timeline.
    #[props(default = true)]
    timeline: bool,
) -> Element {
    let t0 = now.0;
    let sc = scenario.clone();
    let mut ho = use_signal(move || Handoff::new(&sc, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let h = ho.read().clone();
    let mut act = move |id: String| {
        let x = ho.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let st = h.stage(n);
    let typing = h.typing(n);
    let marks = h.marks(n);
    let shown: Vec<usize> = (0..h.msgs.len()).filter(|&i| h.msgs[i].at <= n).collect();
    let header = match st {
        Stage::Connected | Stage::Resolved => (PERSON.0, PERSON.1),
        _ => (ASSISTANT.0, "Answers in seconds"),
    };
    rsx! {
        section { class: "oxaik oxaik-ho", "aria-label": "Support chat",
            header { class: "oxaik-hd",
                div { class: "oxaik-row oxaik-ho-who",
                    if matches!(st, Stage::Connected | Stage::Resolved) { span { class: "oxaik-av", "MI" } } else { span { class: "oxaik-ho-bot", {mark(ASSISTANT.1, 16.0)} } }
                    div { class: "oxaik-hl", b { "{header.0}" } span { class: "oxaik-mute", "{header.1}" } }
                }
                span { class: "oxaik-row oxaik-mute oxaik-ho-via", {mark("intercom", 13.0)} "via Intercom" }
            }
            div { class: "oxaik-ho-main",
                if timeline {
                    ol { class: "oxaik-ho-tl", "aria-label": "Stages",
                        for (s, on) in marks.iter() {
                            li { key: "{s.label()}", class: cls("oxaik-ho-mk", *on), "aria-current": if *s == st || (st == Stage::Unavailable && *s == Stage::Transferring) { "step" } else { "false" },
                                i {} span { if st == Stage::Unavailable && *s == Stage::Transferring { "Unavailable" } else { "{s.label()}" } }
                            }
                        }
                    }
                }
                div { class: "oxaik-ho-thread oxaik-scroll", role: "log", "aria-label": "Messages",
                    for i in shown {
                        {
                            let m = &h.msgs[i];
                            let k = h.words(i, n);
                            let txt = if m.who == Who::Bot { first_words(&m.text, k) } else { m.text.clone() };
                            let who = match m.who { Who::User => "user", Who::Bot => "bot", Who::Agent => "agent", Who::System => "sys" };
                            rsx! {
                                div { key: "{i}", class: "oxaik-ho-msg oxaik-fade", "data-who": "{who}",
                                    if m.who == Who::System { span { class: "oxaik-ho-sys", "{txt}" } } else { p { "{txt}" } }
                                }
                            }
                        }
                    }
                    if let Some(w) = typing {
                        div { class: "oxaik-ho-msg", "data-who": if w == Who::Bot { "bot" } else { "agent" }, "aria-label": "typing",
                            p { class: "oxaik-ho-typing", {dots()} span { class: "oxaik-mute", if w == Who::Bot { "Assistant is thinking" } else { "Mara is typing" } } }
                        }
                    }
                    if st == Stage::Transferring {
                        div { class: "oxaik-ho-card oxaik-fade",
                            div { class: "oxaik-row", {dots()} b { "Connecting you to {PERSON.1}" } }
                            span { class: "oxaik-mute oxaik-num", "{h.wait()}" }
                            div { class: "oxaik-ho-sum",
                                span { class: "oxaik-mute", "Shared with the team" }
                                if edit_summary && h.edit_summary {
                                    textarea { class: "oxaik-ta", "aria-label": "Summary for the team", value: "{h.summary}", oninput: move |e| act(format!("ho-sum:{}", e.value())) }
                                } else { p { "{h.summary}" } }
                            }
                            div { class: "oxaik-row",
                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ho-cancel".into()), "Cancel transfer" }
                                if edit_summary { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ho-editsum".into()), {ico("pencil", 12.0)} if h.edit_summary { "Done" } else { "Edit summary" } } }
                            }
                        }
                    }
                    if st == Stage::Unavailable {
                        div { class: "oxaik-ho-card oxaik-fade", role: "alert",
                            div { class: "oxaik-row", {ico("clock", 14.0)} b { "Nobody from billing is online" } }
                            span { class: "oxaik-mute", "They usually reply within a day. Your summary was emailed to them." }
                            div { class: "oxaik-row",
                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ho-retry".into()), {ico("retry", 12.0)} "Try again" }
                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ho-keep".into()), "Keep chatting with the assistant" }
                            }
                        }
                    }
                }
            }
            if st == Stage::Assistant {
                div { class: "oxaik-row oxaik-ho-chips",
                    for (i, c) in ["Yes, please", "Can you refund it?", "Thanks!"].iter().enumerate() {
                        button { key: "{i}", class: "oxaik-chip", r#type: "button", onclick: move |_| act(format!("ho-chip-{i}")), "{c}" }
                    }
                    button { class: "oxaik-btn sm pri oxaik-ho-human", r#type: "button", onclick: move |_| act("ho-human".into()), {ico("users", 12.0)} "Talk to a person" }
                }
            }
            if st == Stage::Connected {
                div { class: "oxaik-row oxaik-ho-chips",
                    span { class: "oxaik-mute", "Mara has the full conversation." }
                    button { class: "oxaik-btn sm oxaik-ho-human", r#type: "button", onclick: move |_| act("ho-resolve".into()), {ico("check", 12.0)} "Mark resolved" }
                }
            }
            if st == Stage::Resolved {
                div { class: "oxaik-row oxaik-ho-chips", span { class: "oxaik-row oxaik-ok", {ico("check-circle", 13.0)} "Resolved by {PERSON.0}" } }
            }
            form { class: "oxaik-ho-comp", onsubmit: move |e| { e.prevent_default(); act("ho-send".into()) },
                input { class: "oxaik-in", r#type: "text", "aria-label": "Message", placeholder: if st == Stage::Connected { "Reply to Mara" } else { "Message the assistant" }, value: "{h.draft}",
                    disabled: st == Stage::Resolved,
                    oninput: move |e| act(format!("ho-draft:{}", e.value())),
                    onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Enter" { e.prevent_default(); act("ho-send".into()) } },
                }
                button { class: "oxaik-ib", r#type: "submit", "aria-label": "Send", disabled: h.draft.trim().is_empty(), {ico("up", 14.0)} }
            }
            {live(&said())}
        }
    }
}
