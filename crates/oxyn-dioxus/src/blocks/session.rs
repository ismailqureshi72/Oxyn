//! Live session (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::session::{
    AGENTS, ANSWERS, CONTEXT_MODEL, LIMIT, STEPS, Session, StepSt, TASK,
};
use oxyn_core::blocks::{dur, thousands};

use super::{avatar, status};
use crate::aikit::{cls, dots, ico, key_of, live, mark};

const PART_HUES: [&str; 4] = ["neutral", "violet", "teal", "blue"];

/// Live session: sit in on a crew of agents mid-run. The conversation streams in the
/// middle (who wrote to whom, a typing line, an agent's question with quick answers); the team's
/// plan ticks off beside it with the running step marked current; the shared context fills
/// underneath. Agent tabs (arrow keys) narrow the stream to one agent and point at its steps;
/// Escape lets go. The composer sends to everyone or to the agents named with `@` (a menu opens
/// on `@`); Pause freezes the session clock.
#[component]
pub fn LiveSession(
    /// Host clock.
    now: ClockMs,
    /// Start this many ms into the session.
    #[props(default)]
    skip: u64,
    /// Show the shared context under the plan.
    #[props(default = true)]
    context: bool,
    /// Start narrowed to this agent (1–3).
    #[props(default)]
    focus: Option<usize>,
    /// Start paused.
    #[props(default)]
    paused: bool,
) -> Element {
    let mut ses = use_signal(move || {
        let mut s = Session::new(skip, now.0);
        s.focus = focus;
        if paused {
            s.paused = Some(now.0);
        }
        s
    });
    let mut said = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut msel = use_signal(|| 0usize);
    let n = now.0;
    let s = ses.read().clone();
    let mut act = move |id: String| {
        let x = ses.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let msgs = s.messages(n);
    let typing = s.typing(n);
    let stat = s.status(n);
    let hue = match stat {
        "Done" => "green",
        "Paused" => "neutral",
        "Waiting on you" => "amber",
        _ => "blue",
    };
    let menu = Session::mention_menu(&draft.read());
    let parts = s.context(n);
    let used = s.used(n);
    rsx! {
        section { class: "oxaik oxbk oxbk-ls", "aria-label": "{TASK}",
            onkeydown: move |e: KeyboardEvent| {
                if key_of(&e) == "Escape" {
                    let x = ses.write().key("", "Escape", n);
                    if let Some(x) = x { said.set(x) }
                }
            },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", "{TASK}" }
                    p { class: "oxaik-sub", {status(stat, hue)} span { class: "oxaik-num", "{s.done(n)} of {STEPS.len()} steps · {dur(s.t(n))}" } }
                }
                button { class: "oxaik-btn sm", r#type: "button", disabled: stat == "Done", onclick: move |_| act("ls-pause".into()),
                    {ico(if s.paused.is_some() { "play" } else { "pause" }, 12.0)}
                    if s.paused.is_some() { "Resume" } else { "Pause" }
                }
            }
            div { class: "oxbk-tabs oxaik-scroll", role: "tablist", "aria-label": "Agents",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                        e.prevent_default();
                        let x = ses.write().key("tabs", &k, n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                for (i, (name, brand, model)) in AGENTS.iter().enumerate() {
                    {
                        let on = s.focus.unwrap_or(0) == i;
                        let label = if i == 0 { "Everyone" } else { name };
                        rsx! {
                            button { key: "{i}", class: cls("oxbk-tab", on), r#type: "button", role: "tab", "aria-selected": "{on}", tabindex: if on { "0" } else { "-1" },
                                title: "{model}",
                                onclick: move |_| act(format!("ls-agent-{i}")),
                                if i > 0 { {avatar(brand, name)} } else { {ico("users", 13.0)} }
                                span { "{label}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxbk-ls-body",
                div { class: "oxbk-ls-stream",
                    ol { class: "oxbk-ls-msgs oxaik-scroll", "aria-label": "Conversation", "aria-live": "polite",
                        onkeydown: move |e: KeyboardEvent| {
                            let k = key_of(&e);
                            if matches!(k.as_str(), "ArrowUp" | "ArrowDown") {
                                e.prevent_default();
                                ses.write().key("msgs", &k, n);
                            }
                        },
                        for (i, m) in msgs.iter().enumerate() {
                            {
                                let fade = s.focus.is_some_and(|f| m.from != f && m.to != Some(f));
                                let picked = s.picked == Some(i);
                                let to = m.to.map_or("everyone".to_string(), |t| AGENTS[t].0.to_string());
                                let (name, brand, _) = AGENTS[m.from];
                                rsx! {
                                    li { key: "{i}-{m.at}", class: cls("oxbk-ls-msg", picked), "data-fade": "{fade}", "data-you": "{m.from == 0}", tabindex: "0",
                                        "aria-label": "{Session::label(m)}",
                                        onclick: move |_| act(format!("ls-msg-{i}")),
                                        onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Enter" { act(format!("ls-msg-{i}")) },
                                        {avatar(brand, name)}
                                        div { class: "oxbk-ls-mb",
                                            p { class: "oxbk-ls-who", b { "{name}" } span { class: "oxaik-mute", " to {to}" } span { class: "oxaik-mute oxaik-num oxbk-ls-at", "{dur(m.at)}" } }
                                            p { class: "oxbk-ls-txt", "{m.text}" }
                                            if m.asks {
                                                div { class: "oxaik-row oxbk-ls-ans", role: "group", "aria-label": "Answer",
                                                    for (a, t) in ANSWERS.iter().enumerate() {
                                                        button { key: "{a}", class: "oxaik-chip", r#type: "button", onclick: move |e| { e.stop_propagation(); act(format!("ls-answer-{a}")) }, "{t}" }
                                                    }
                                                }
                                            }
                                            if picked {
                                                p { class: "oxaik-mute oxbk-ls-det", "Sent {dur(m.at)} into the session · {m.text.split_whitespace().count()} words · to {to}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(t) = typing {
                            li { class: "oxbk-ls-typing oxaik-mute", {dots()} " {AGENTS[t].0} is writing" }
                        }
                    }
                    div { class: "oxbk-ls-comp",
                        if let Some(list) = menu.clone().filter(|l| !l.is_empty()) {
                            ul { class: "oxbk-menu", role: "listbox", "aria-label": "Name an agent",
                                for (k, &i) in list.iter().enumerate() {
                                    li { key: "{i}", role: "option", "aria-selected": "{k == *msel.read() % list.len()}", class: cls("oxbk-mi", k == *msel.read() % list.len()),
                                        onmousedown: move |e| {
                                            e.prevent_default();
                                            let d = draft.read().clone();
                                            let at = d.rfind('@').unwrap_or(d.len());
                                            draft.set(format!("{}@{} ", &d[..at], AGENTS[i].0));
                                        },
                                        {avatar(AGENTS[i].1, AGENTS[i].0)} "{AGENTS[i].0}"
                                    }
                                }
                            }
                        }
                        input { class: "oxaik-in", r#type: "text", value: "{draft}", placeholder: "Message the team, @ to name an agent", "aria-label": "Message",
                            oninput: move |e| { draft.set(e.value()); msel.set(0); },
                            onkeydown: move |e: KeyboardEvent| {
                                let k = key_of(&e);
                                let list = Session::mention_menu(&draft.read()).filter(|l| !l.is_empty());
                                match (k.as_str(), list) {
                                    ("ArrowDown", Some(_)) => { e.prevent_default(); let v = *msel.read() + 1; msel.set(v) }
                                    ("ArrowUp", Some(l)) => { e.prevent_default(); let v = (*msel.read() + l.len() - 1) % l.len(); msel.set(v) }
                                    ("Enter", Some(l)) => {
                                        e.prevent_default();
                                        let i = l[*msel.read() % l.len()];
                                        let d = draft.read().clone();
                                        let at = d.rfind('@').unwrap_or(d.len());
                                        draft.set(format!("{}@{} ", &d[..at], AGENTS[i].0));
                                    }
                                    ("Enter", None) if !e.modifiers().shift() => {
                                        e.prevent_default();
                                        let t = draft.read().clone();
                                        let x = ses.write().send(&t, n);
                                        if let Some(x) = x { said.set(x); draft.set(String::new()) }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Send",
                            onclick: move |_| {
                                let t = draft.read().clone();
                                let x = ses.write().send(&t, n);
                                if let Some(x) = x { said.set(x); draft.set(String::new()) }
                            },
                            {ico("up", 14.0)}
                        }
                    }
                }
                aside { class: "oxbk-ls-side",
                    div { class: "oxbk-card",
                        p { class: "oxbk-cap", "Plan" }
                        ol { class: "oxbk-ls-plan",
                            for (i, (title, ag, _, _)) in STEPS.iter().enumerate() {
                                {
                                    let st = s.step(i, n);
                                    let dim = s.focus.is_some_and(|f| *ag != f);
                                    let (ic, h, w) = match st {
                                        StepSt::Done => ("check-circle", "green", "Done"),
                                        StepSt::Active => ("play", "blue", "Running"),
                                        StepSt::Yours => ("hand", "amber", "Your turn"),
                                        StepSt::Todo => ("minus", "neutral", "To do"),
                                    };
                                    let t = s.step_time(i, n).unwrap_or_default();
                                    rsx! {
                                        li { key: "{i}", class: "oxbk-ls-step", "data-st": "{w}", "data-dim": "{dim}",
                                            "aria-current": if matches!(st, StepSt::Active | StepSt::Yours) { "step" } else { "false" },
                                            span { class: "oxbk-st", "data-hue": "{h}", {ico(ic, 13.0)} }
                                            span { class: "oxbk-ls-stt", span { class: if st == StepSt::Active { "oxaik-shim" } else { "" }, "{title}" }
                                                span { class: "oxaik-sr", ", {w}" }
                                            }
                                            {avatar(AGENTS[*ag].1, AGENTS[*ag].0)}
                                            span { class: "oxaik-mute oxaik-num oxbk-ls-time", "{t}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if context {
                        div { class: "oxbk-card",
                            p { class: "oxbk-cap oxaik-row oxaik-between", span { "Context" } span { class: "oxaik-row oxaik-mute", {mark(CONTEXT_MODEL.0, 12.0)} "{CONTEXT_MODEL.1}" } }
                            div { class: "oxbk-stack", role: "meter", "aria-label": "Context window", "aria-valuemin": "0", "aria-valuemax": "{LIMIT}", "aria-valuenow": "{used}", "aria-valuetext": "{s.context_line(n)}",
                                for (k, (l, tk)) in parts.iter().enumerate() {
                                    span { key: "{l}", "data-hue": "{PART_HUES[k]}", style: "width:{*tk as f64 / LIMIT as f64 * 100.0}%" }
                                }
                            }
                            p { class: "oxaik-mute oxaik-num", "{s.context_line(n)}" }
                            ul { class: "oxbk-legend",
                                for (k, (l, tk)) in parts.iter().enumerate() {
                                    li { key: "{l}", i { "data-hue": "{PART_HUES[k]}" } span { "{l}" } span { class: "oxaik-num oxaik-mute", "{thousands(*tk)}" } }
                                }
                            }
                        }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
