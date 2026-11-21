//! Run inspector (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::inspector::{Inspector, Kind, SPANS, SUPERVISOR, SpanSt};
use oxyn_core::blocks::thousands;

use super::{avatar, status};
use crate::aikit::{cls, ico, key_of, live, mark};

/// Run inspector: one multi-agent run on one screen. The orchestrator and the sub-agents it
/// started form a live tree of cards (a supervisor watches beside it, joined by a dashed line);
/// every card lists its model and tool calls as rows with a small bar on the agent's own time
/// axis. Clicking a card picks the agent (the others fade, the numbers on top sum up its part);
/// clicking a call opens its arguments and result, or its error, right under the row (Escape
/// closes). The run plays on its own clock and can be replayed.
#[component]
pub fn RunInspector(
    /// Host clock.
    now: ClockMs,
    /// Start this many ms into the run (24000 opens it finished).
    #[props(default)]
    skip: u64,
    /// Speed ×100 (100 = real time).
    #[props(default = 100)]
    speed: u64,
    /// Show the supervisor beside the orchestrator.
    #[props(default = true)]
    supervisor: bool,
    /// Open this call at the start (span index).
    #[props(default)]
    call: Option<usize>,
) -> Element {
    let mut ri = use_signal(move || {
        let mut r = Inspector::new(skip, speed, supervisor, now.0);
        r.call = call;
        r
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let r = ri.read().clone();
    let mut act = move |id: String| {
        let x = ri.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let nums = r.numbers(n);
    let agents = r.agents(n);
    rsx! {
        section { class: "oxaik oxbk oxbk-ri", "aria-label": "Run inspector",
            onkeydown: move |e: KeyboardEvent| {
                if key_of(&e) == "Escape" {
                    let x = ri.write().key("Escape", n);
                    if let Some(x) = x { said.set(x) }
                }
            },
            header { class: "oxaik-hd oxbk-ri-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", "{SPANS[0].args}" }
                    p { class: "oxaik-sub", "aria-live": "polite", "{r.title(n)}" }
                }
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ri-replay".into()), {ico("retry", 12.0)} "Replay" }
            }
            dl { class: "oxbk-ri-nums", "aria-label": "Numbers for {nums.of}",
                div { dt { "Scope" } dd { "{nums.of}" } }
                div { dt { "Time" } dd { class: "oxaik-num", "{nums.time}" } }
                div { dt { "Calls" } dd { class: "oxaik-num", "{nums.calls}" } }
                div { dt { "Tokens" } dd { class: "oxaik-num", "{thousands(nums.tokens.0)} in · {thousands(nums.tokens.1)} out" } }
                div { dt { "Cost" } dd { class: "oxaik-num", "{nums.cost}" } }
                div { dt { "Errors" } dd { class: "oxaik-num", "data-hue": if nums.errors > 0 { "red" } else { "neutral" }, "{nums.errors}" } }
            }
            ul { class: "oxbk-ri-tree", "aria-label": "Agents",
                for &a in agents.iter() {
                    {
                        let sp = &SPANS[a];
                        let depth = Inspector::depth(a);
                        let (w, h) = r.agent_st(a, n);
                        let fade = r.selected.is_some_and(|s| s != a);
                        let picked = r.selected == Some(a);
                        let calls = r.calls(a, n);
                        rsx! {
                            li { key: "{a}", class: "oxbk-ri-node", "data-depth": "{depth}", "data-fade": "{fade}",
                                div { class: "oxbk-ri-pair",
                                    div { class: cls("oxbk-ri-card", picked), "data-st": "{w}",
                                        button { class: "oxbk-ri-cardb", r#type: "button", "aria-pressed": "{picked}",
                                            onclick: move |_| act(format!("ri-agent-{a}")),
                                            {avatar(sp.brand, sp.name)}
                                            span { class: "oxbk-ri-cn", b { "{sp.name}" } span { class: "oxaik-row oxaik-mute", {mark(sp.brand, 11.0)} "{sp.model}" } }
                                            {status(w, h)}
                                            span { class: "oxaik-mute oxaik-num oxbk-ri-t", "{r.time(a, n)}" }
                                        }
                                        p { class: "oxbk-ri-task", "{sp.args}" }
                                        if !calls.is_empty() {
                                            ol { class: "oxbk-ri-calls", "aria-label": "Calls by {sp.name}",
                                                for &c in calls.iter() {
                                                    {
                                                        let cs = &SPANS[c];
                                                        let st = r.st(c, n);
                                                        let (l, bw) = r.bar(c, n);
                                                        let open = r.call == Some(c);
                                                        let hue = match st { SpanSt::Failed => "red", SpanSt::Running => "blue", _ => if cs.kind == Kind::Llm { "violet" } else { "neutral" } };
                                                        rsx! {
                                                            li { key: "{c}", class: "oxbk-ri-call", "data-st": "{st:?}",
                                                                button { class: "oxbk-ri-callb", r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-ri-cd-{c}",
                                                                    "aria-label": "{r.call_label(c, n)}",
                                                                    onclick: move |_| act(format!("ri-call-{c}")),
                                                                    span { class: "oxbk-ri-ck", if cs.kind == Kind::Llm { {mark(cs.brand, 12.0)} } else { {ico("tool", 12.0)} } }
                                                                    span { class: if cs.kind == Kind::Tool { "oxbk-ri-cnm oxaik-mono" } else { "oxbk-ri-cnm" }, "{cs.name}" }
                                                                    span { class: "oxbk-ri-bar", "aria-hidden": "true", i { "data-hue": "{hue}", style: "left:{l * 100.0}%;width:max(3px, {bw * 100.0}%)" } }
                                                                    span { class: "oxaik-mute oxaik-num oxbk-ri-cd", "{r.time(c, n)}" }
                                                                    if st == SpanSt::Failed { span { class: "oxbk-st", "data-hue": "red", {ico("x-circle", 12.0)} } }
                                                                }
                                                                if open {
                                                                    div { class: "oxbk-ri-det oxaik-fade", id: "oxbk-ri-cd-{c}", role: "region", "aria-label": "{cs.name} details",
                                                                        p { span { class: "oxaik-mute", "Input " } code { class: "oxaik-mono", "{cs.args}" } }
                                                                        if st == SpanSt::Failed {
                                                                            p { class: "oxbk-ri-err", {ico("alert", 12.0)} "{cs.error}" }
                                                                        } else if st == SpanSt::Running {
                                                                            p { class: "oxaik-mute", "Still running…" }
                                                                        } else {
                                                                            p { span { class: "oxaik-mute", "Output " } "{cs.result}" }
                                                                        }
                                                                        if cs.tin > 0 { p { class: "oxaik-mute oxaik-num", "{thousands(cs.tin)} tokens in · {thousands(cs.tout)} out" } }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if a == 0 && r.supervisor {
                                        div { class: "oxbk-ri-sup", "aria-label": "Supervisor",
                                            div { class: "oxaik-row", {avatar(SUPERVISOR.1, SUPERVISOR.0)} b { "{SUPERVISOR.0}" } span { class: "oxaik-pill", "Supervisor" } }
                                            p { class: "oxaik-row oxaik-mute", {mark(SUPERVISOR.1, 11.0)} "{SUPERVISOR.2}" }
                                            p { class: "oxbk-ri-task", "{SUPERVISOR.3}" }
                                        }
                                    }
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
