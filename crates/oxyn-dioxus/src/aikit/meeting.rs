//! Meeting summary (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::first_words;
use oxyn_core::aikit::meeting::{ATTENDEES, LINES, MEETING, Meeting, SendSt, Status};

use super::{cls, dots, ico, key_of, live, mark};

fn initials(n: &str) -> String {
    n.split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect()
}

/// Meeting summary: AI notes for one call. The header shows the platform mark, the attendees and
/// the duration; after a short summarising pass, Summary, Decisions and Action items stream in.
/// Each line's timestamp opens its transcript quote (hover, focus or press; Escape closes);
/// action items can be checked off or sent to Linear or Notion (simulated), with a failed send
/// offering retry. Brief drops the decisions (Oxyn extra).
#[component]
pub fn MeetingSummary(
    /// Host clock.
    now: ClockMs,
    /// `play` (summarise on mount), `ready`, `error` or `empty`.
    #[props(default = "play".to_string())]
    scenario: String,
    /// Show the Linear and Notion send buttons.
    #[props(default = true)]
    send: bool,
    /// The first Notion send fails.
    #[props(default)]
    fail_send: bool,
    /// Brief notes (summary and action items only).
    #[props(default)]
    brief: bool,
) -> Element {
    let t0 = now.0;
    let sc = scenario.clone();
    let mut ms = use_signal(move || {
        let mut m = Meeting::new(&sc, t0);
        m.fail_send = fail_send;
        m.brief = brief;
        m
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let m = ms.read().clone();
    let mut act = move |id: String| {
        let x = ms.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let st = m.status_at(n);
    let (title, platform, when, mins, words) = MEETING;
    let (done, total) = m.progress();
    let sections = [("Summary", 0u8), ("Decisions", 1u8), ("Action items", 2u8)];
    rsx! {
        article { class: "oxaik oxaik-ms", "aria-label": "{title}", "aria-busy": "{st == Status::Summarizing}",
            onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Escape" { let x = ms.write().key("Escape"); if let Some(x) = x { said.set(x) } } },
            header { class: "oxaik-hd",
                div { class: "oxaik-row oxaik-ms-top",
                    {mark(platform, 22.0)}
                    div { class: "oxaik-hl",
                        h2 { class: "oxaik-h", "{title}" }
                        span { class: "oxaik-mute oxaik-num", "Zoom · {when} · {mins} min · {oxyn_core::aikit::thousands(words as u64)} words" }
                    }
                }
                div { class: "oxaik-ms-av", "aria-label": "Attendees: {ATTENDEES.join(\", \")}",
                    for a in ATTENDEES.iter() { span { key: "{a}", class: "oxaik-av", title: "{a}", tabindex: "0", "{initials(a)}" } }
                }
            }
            div { class: "oxaik-ms-body",
                {match st {
                    Status::Summarizing => rsx! {
                        div { class: "oxaik-ms-think", {dots()} span { class: "oxaik-shim", "Summarizing 34 minutes of transcript" } }
                        div { class: "oxaik-skel", "aria-hidden": "true", i { style: "width:92%" } i { style: "width:74%" } i { style: "width:84%" } }
                    },
                    Status::Error => rsx! {
                        div { class: "oxaik-err oxaik-ms-err", role: "alert",
                            {ico("alert", 18.0)} b { "Couldn't summarize this meeting" }
                            span { class: "oxaik-mute", "The transcript service timed out." }
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ms-regen".into()), {ico("retry", 12.0)} "Try again" }
                        }
                    },
                    Status::Empty => rsx! {
                        div { class: "oxaik-err oxaik-ms-err", {ico("mic", 18.0)} b { "No transcript yet" } span { class: "oxaik-mute", "Notes appear here after the call is recorded." } }
                    },
                    Status::Ready => rsx! {
                        for (name, sec) in sections.iter().filter(|s| !(m.brief && s.1 == 1)) {
                            section { key: "{name}", class: "oxaik-ms-sec",
                                h3 { class: "oxaik-h4", "{name}" if *sec == 2 { span { class: "oxaik-mute oxaik-num", " · {done} of {total} done" } } }
                                ul { class: "oxaik-ms-list",
                                    for i in m.lines().into_iter().filter(|&i| LINES[i].0 == *sec) {
                                        {
                                            let l = LINES[i];
                                            let vis = m.visible(i, n);
                                            let text = first_words(l.1, vis);
                                            let full = vis >= l.1.split_whitespace().count();
                                            let open = m.quote == Some(i);
                                            let send_st = m.send_st(i, n);
                                            let checked = m.checked[i];
                                            rsx! {
                                                li { key: "{i}", class: cls("oxaik-ms-li", checked), "data-sec": "{sec}",
                                                    if l.0 == 2 {
                                                        button { class: cls("oxaik-ms-cb", checked), r#type: "button", role: "checkbox", "aria-checked": "{checked}", "aria-label": "{l.1}",
                                                            onclick: move |_| act(format!("ms-check-{i}")), if checked { {ico("check", 11.0)} }
                                                        }
                                                    } else { span { class: "oxaik-ms-bul", "aria-hidden": "true" } }
                                                    div { class: "oxaik-ms-txt",
                                                        span { class: "oxaik-ms-line", "{text}" }
                                                        if l.0 == 2 && full { span { class: "oxaik-mute oxaik-ms-meta", "{l.5} · due {l.6}" } }
                                                    }
                                                    if full {
                                                        div { class: "oxaik-ms-side",
                                                            div { class: "oxaik-menu-wrap",
                                                                button { class: "oxaik-ms-ts oxaik-mono", r#type: "button", "aria-expanded": "{open}", "aria-describedby": if open { "oxaik-ms-q" } else { "" },
                                                                    onclick: move |_| act(format!("ms-quote-{i}")), "{l.2}"
                                                                }
                                                                if open {
                                                                    div { id: "oxaik-ms-q", class: "oxaik-ms-quote", role: "tooltip",
                                                                        b { "{l.3}" } span { "“{l.4}”" }
                                                                    }
                                                                }
                                                            }
                                                            if l.0 == 2 && send {
                                                                {match send_st {
                                                                    SendSt::None => rsx! {
                                                                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Send {l.1} to Linear", onclick: move |_| act(format!("ms-linear-{i}")), {mark("linear", 13.0)} }
                                                                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Send {l.1} to Notion", onclick: move |_| act(format!("ms-notion-{i}")), {mark("notion", 13.0)} }
                                                                    },
                                                                    SendSt::Sending(_) => rsx! { span { class: "oxaik-pill", {dots()} "Sending" } },
                                                                    SendSt::Sent(t, r) => rsx! { span { class: "oxaik-pill", "data-hue": "green", {mark(if t == 0 { "linear" } else { "notion" }, 12.0)} "{Meeting::reference(t, r)}" } },
                                                                    SendSt::Failed(t) => rsx! {
                                                                        button { class: "oxaik-pill oxaik-ms-retry", "data-hue": "red", r#type: "button", onclick: move |_| act(format!("ms-{}-{i}", if t == 0 { "linear" } else { "notion" })), {ico("alert", 12.0)} "Retry" }
                                                                    },
                                                                }}
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }}
            }
            div { class: "oxaik-row oxaik-between oxaik-ms-foot",
                span { class: "oxaik-row oxaik-mute", {mark("claude", 13.0)} "Summarized by Claude Sonnet 4.5" }
                div { class: "oxaik-row",
                    button { class: cls("oxaik-chip", m.brief), r#type: "button", "aria-pressed": "{m.brief}", onclick: move |_| act("ms-brief".into()), "Brief" }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Copy notes", onclick: move |_| act("ms-copy".into()), {ico("copy", 13.0)} }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Regenerate", onclick: move |_| act("ms-regen".into()), {ico("retry", 13.0)} }
                }
            }
            {live(&said())}
        }
    }
}
