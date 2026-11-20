//! Ticket router (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::router::{LANES, Router, TICKETS, TicketSt};

use super::{cls, dots, ico, live, mark};

/// Ticket router: support messages from Gmail, Intercom, Slack and GitHub arrive one at a time;
/// an AI classifier labels each with intent, sentiment and confidence (labels fade in one after
/// another) and a pulse carries it into a team lane with a rolling count. Below the confidence
/// threshold a ticket waits in Needs review, where a person moves it. Pause, restart, speed and
/// the threshold are controls.
#[component]
pub fn TicketRouter(
    /// Host clock.
    now: ClockMs,
    /// Start paused.
    #[props(default)]
    paused: bool,
    /// Review below this confidence, %.
    #[props(default = 70)]
    threshold: u32,
    /// Show the threshold control (Oxyn extra).
    #[props(default = true)]
    tuning: bool,
) -> Element {
    let t0 = now.0;
    let mut rt = use_signal(move || Router::new(paused, threshold, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let r = rt.read().clone();
    let mut act = move |id: String| {
        let x = rt.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let cur = r.current(n);
    let counts = r.counts(n);
    rsx! {
        section { class: "oxaik oxaik-rt", "aria-label": "Ticket router",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Support inbox router" }
                    p { class: "oxaik-sub oxaik-row", {mark("claude", 12.0)} "Classified by Claude Haiku 4.5 · review below {r.threshold}%" }
                }
                div { class: "oxaik-ctl",
                    button { class: "oxaik-ib", r#type: "button", "aria-label": if r.paused.is_some() { "Resume" } else { "Pause" }, "aria-pressed": "{r.paused.is_some()}", onclick: move |_| act("rt-pause".into()),
                        {ico(if r.paused.is_some() { "play" } else { "pause" }, 13.0)}
                    }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Restart", onclick: move |_| act("rt-restart".into()), {ico("undo", 13.0)} }
                    button { class: "oxaik-chip oxaik-num", r#type: "button", "aria-label": "Speed {r.speed}x", onclick: move |_| act("rt-speed".into()), "{r.speed}x" }
                }
            }
            div { class: "oxaik-rt-in",
                if let Some(i) = cur {
                    {
                        let t = TICKETS[i];
                        let st = r.st(i, n);
                        let lab = st != TicketSt::Arriving;
                        let lane = r.lane(i);
                        rsx! {
                            div { key: "{i}", class: "oxaik-rt-ticket oxaik-fade", "data-st": match st { TicketSt::Arriving => "arrive", TicketSt::Classifying => "class", _ => "route" },
                                div { class: "oxaik-row oxaik-rt-from", {mark(t.0, 15.0)} b { "{t.1}" } span { class: "oxaik-mute", "{t.2}" } }
                                div { class: "oxaik-row oxaik-rt-labels",
                                    if st == TicketSt::Classifying { span { class: "oxaik-row oxaik-mute", {dots()} span { class: "oxaik-shim", "Classifying" } } }
                                    if lab && st != TicketSt::Classifying {
                                        span { class: "oxaik-pill", style: "animation-delay:0ms", "{t.3}" }
                                        span { class: "oxaik-pill", "data-hue": if t.4 == "Positive" { "green" } else if t.4 == "Neutral" { "neutral" } else { "red" }, style: "animation-delay:50ms", "{t.4}" }
                                        span { class: "oxaik-pill oxaik-num", "data-hue": if t.5 < r.threshold { "amber" } else { "neutral" }, style: "animation-delay:100ms", "{t.5}%" }
                                        span { class: "oxaik-row oxaik-rt-to", {ico("chevron-right", 12.0)} "{LANES[lane].0}" }
                                    }
                                }
                            }
                        }
                    }
                } else if r.finished(n) {
                    p { class: "oxaik-row oxaik-mute", {ico("check-circle", 13.0)} "All {TICKETS.len()} tickets routed" }
                } else {
                    p { class: "oxaik-row oxaik-mute", {dots()} "Waiting for tickets" }
                }
            }
            div { class: "oxaik-rt-lanes",
                for (l, (name, hue)) in LANES.iter().enumerate() {
                    div { key: "{name}", class: cls("oxaik-rt-lane", cur.is_some_and(|i| r.lane(i) == l && r.st(i, n) == TicketSt::Routing)), "data-hue": "{hue}",
                        div { class: "oxaik-row oxaik-between", span { class: "oxaik-row", span { class: "oxaik-dot" } b { "{name}" } } span { class: "oxaik-num oxaik-mute", "{counts[l]}" } }
                        ul { class: "oxaik-rt-items",
                            for i in r.in_lane(l, n).into_iter().take(3) {
                                li { key: "{i}", class: "oxaik-fade",
                                    span { class: "oxaik-row oxaik-rt-it", {mark(TICKETS[i].0, 12.0)} span { "{TICKETS[i].2}" } }
                                    if l == 3 {
                                        span { class: "oxaik-row oxaik-rt-rev",
                                            for (k, ln) in LANES.iter().take(3).enumerate() {
                                                button { key: "{k}", class: "oxaik-chip", r#type: "button", "aria-label": "Move {TICKETS[i].2} to {ln.0}", onclick: move |_| act(format!("rt-review-{i}-{k}")), "{ln.0}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if tuning {
                div { class: "oxaik-row oxaik-rt-tune",
                    label { class: "oxaik-mute", r#for: "oxaik-rt-th", "Review below" }
                    input { id: "oxaik-rt-th", class: "oxaik-range", r#type: "range", min: "40", max: "95", step: "5", value: "{r.threshold}", "aria-valuetext": "{r.threshold}% confidence",
                        oninput: move |e| { if let Ok(v) = e.value().parse::<u32>() { act(format!("rt-th-{v}")) } },
                    }
                    b { class: "oxaik-num", "{r.threshold}%" }
                }
            }
            {live(&said())}
        }
    }
}
