//! Agent doc cursor (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::doccursor::{AGENT, BlockKind, DocCursor, MATE, META, Phase, TITLE};

use super::{dots, ico, live, mark};

fn cursor(name: &str, who: &str) -> Element {
    rsx! {
        span { class: "oxaik-dc-cur", "data-who": "{who}", "aria-hidden": "true",
            svg { width: "12", height: "14", view_box: "0 0 12 14", path { d: "M1 1 L11 7 L6.5 8 L4.5 13 Z" } }
            span { class: "oxaik-dc-tag", "{name}" }
        }
    }
}

/// Agent doc cursor (Arc "Agent cursor"): an agent working on a shared document like a coworker.
/// Its labelled pointer marks a paragraph and retypes it, adds a new section and corrects a
/// misspelling while a colleague reads along. Edits stay highlighted until someone keeps or reverts
/// them from the review strip; Stop keeps finished edits for review. While the pointer is over the
/// card or focus is inside it, a repeating demo waits in review. The text is plain headings and
/// paragraphs; pointers and highlights are hidden from assistive tech, and each step is spoken.
#[component]
pub fn AgentDocCursor(
    /// Host clock.
    now: ClockMs,
    /// Start a run on mount (off shows an Ask button).
    #[props(default = true)]
    auto_play: bool,
    /// Replay: accept on its own after a pause, then run again.
    #[props(default = true)]
    looping: bool,
    /// Playback speed, percent.
    #[props(default = 100)]
    speed: u64,
    /// The agent can't be reached (error scenario).
    #[props(default)]
    fail: bool,
    /// Show the teammate's cursor.
    #[props(default = true)]
    teammate: bool,
) -> Element {
    let t0 = now.0;
    let mut dc = use_signal(move || DocCursor::new(auto_play, looping, speed, fail, teammate, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let d = dc.read().rolled(n);
    let mut act = move |id: String| {
        let x = {
            let mut w = dc.write();
            *w = w.rolled(n);
            w.press(&id, n)
        };
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = d.phase(n);
    let blocks = d.blocks(n);
    let pending = d.edits_done(n);
    let review = pending > 0 && matches!(ph, Phase::Review | Phase::Stopped);
    let busy = matches!(ph, Phase::Thinking | Phase::Editing(_));
    let changes = if pending == 1 { "change" } else { "changes" };
    let sr = if said().is_empty() {
        d.status(n)
    } else {
        said()
    };
    rsx! {
        article { class: "oxaik oxaik-dc", "aria-label": "{TITLE}",
            onmouseenter: move |_| dc.write().hold = true,
            onmouseleave: move |_| dc.write().hold = false,
            onfocusin: move |_| dc.write().hold = true,
            onfocusout: move |_| dc.write().hold = false,
            header { class: "oxaik-hd oxaik-dc-hd",
                div { class: "oxaik-row oxaik-dc-st", "data-ph": "{ph:?}",
                    {mark(AGENT.1, 16.0)}
                    if busy { {dots()} }
                    {match ph {
                        Phase::Accepted | Phase::Review => rsx! { {ico("check", 13.0)} },
                        Phase::Error => rsx! { {ico("alert", 13.0)} },
                        Phase::Stopped => rsx! { {ico("stop", 12.0)} },
                        _ => rsx! {},
                    }}
                    span { key: "{d.status(n)}", class: "oxaik-fade", "{d.status(n)}" }
                }
                div { class: "oxaik-ctl",
                    if busy {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("dc-stop".into()), {ico("stop", 11.0)} "Stop" }
                    } else if ph == Phase::Idle {
                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("dc-ask".into()), {ico("spark", 12.0)} "Ask {AGENT.0}" }
                    } else if ph == Phase::Error {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("dc-retry".into()), {ico("retry", 12.0)} "Try again" }
                    } else {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("dc-ask".into()), {ico("retry", 12.0)} "Run again" }
                    }
                }
            }
            div { class: "oxaik-dc-doc",
                h3 { class: "oxaik-dc-title", "{TITLE}" }
                p { class: "oxaik-mute oxaik-dc-meta", "{META}" }
                for b in blocks {
                    {
                        let body = rsx! {
                            if b.mate { {cursor(MATE, "mate")} }
                            "{b.text}"
                            if !b.tint.is_empty() { mark { class: "oxaik-dc-tint", "data-mark": "{b.mark}", "{b.tint}" } }
                            if b.agent { {cursor(AGENT.0, "agent")} }
                            "{b.tail}"
                        };
                        match b.kind {
                            BlockKind::Heading => rsx! { h4 { key: "{b.id}", class: "oxaik-dc-h", "data-new": "{b.id.starts_with('n')}", {body} } },
                            BlockKind::Para => rsx! { p { key: "{b.id}", class: "oxaik-dc-p", "data-new": "{b.id.starts_with('n')}", {body} } },
                        }
                    }
                }
            }
            if review {
                div { class: "oxaik-dc-bar", role: "group", "aria-label": "Review changes",
                    span { class: "oxaik-row",
                        i { class: "oxaik-dc-sw" }
                        b { "{pending} {changes}" }
                        span { class: "oxaik-mute oxaik-dc-by", "by {AGENT.0}" }
                    }
                    div { class: "oxaik-ctl",
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("dc-undo".into()), {ico("undo", 12.0)} "Undo" }
                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("dc-accept".into()), {ico("check", 12.0)} "Accept changes" }
                    }
                }
            }
            {live(&sr)}
        }
    }
}
