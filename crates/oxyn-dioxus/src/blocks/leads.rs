//! Lead scoring (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::ago;
use oxyn_core::blocks::leads::{DESTS, LEADS, Leads, Push, SORTS, tier};

use crate::aikit::{cls, dots, ico, key_of, live, mark};

fn ring(score: u32, hue: &str) -> Element {
    let c = 2.0 * std::f64::consts::PI * 14.0;
    let off = c * (1.0 - f64::from(score) / 100.0);
    rsx! {
        span { class: "oxbk-ld-ring", "data-hue": "{hue}", "aria-hidden": "true",
            svg { view_box: "0 0 36 36", width: "36", height: "36",
                circle { class: "oxbk-ld-rt", cx: "18", cy: "18", r: "14" }
                circle { class: "oxbk-ld-rv", cx: "18", cy: "18", r: "14", stroke_dasharray: "{c:.1}", stroke_dashoffset: "{off:.1}" }
            }
            b { class: "oxaik-num", "{score}" }
        }
    }
}

/// Lead scoring: new business sign-ups sorted by a model's fit score. A line holds the company,
/// its contact, a ring with the number (plus Hot, Warm or Cold) and a short why; expanding it
/// lists the signals, how the points add up and a send-to-CRM action (HubSpot or Airtable).
/// Rescore counts the rings up again and can move a lead; sort by score or newest.
#[component]
pub fn LeadScoring(
    /// Host clock.
    now: ClockMs,
    /// Count the rings up on first view.
    #[props(default = true)]
    intro: bool,
    /// The first push fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Lead open at start.
    #[props(default)]
    open: Option<usize>,
) -> Element {
    let mut ld = use_signal(move || Leads::new(intro, fail, open, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let l = ld.read().clone();
    let mut act = move |id: String| {
        let x = ld.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let order = l.order();
    rsx! {
        section { class: "oxaik oxbk oxbk-ld", "aria-label": "Lead scoring",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h2 { class: "oxaik-h", "Inbound leads" }
                    p { class: "oxaik-sub", "{LEADS.len()} new · ranked by AI fit score" }
                }
                div { class: "oxaik-row",
                    div { class: "oxaik-seg", role: "radiogroup", "aria-label": "Sort by",
                        for (i, s) in SORTS.iter().enumerate() {
                            button { key: "{i}", class: cls("oxaik-sg", l.sort == i), r#type: "button", role: "radio", "aria-checked": "{l.sort == i}", onclick: move |_| act(format!("ld-sort-{i}")), "{s}" }
                        }
                    }
                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ld-rescore".into()), {ico("retry", 12.0)} span { class: "oxbk-ld-rs", "Rescore" } }
                }
            }
            ul { class: "oxbk-ld-list", "aria-label": "Leads",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "Home" | "End" | "ArrowLeft" | "ArrowRight") {
                        e.prevent_default();
                        let x = ld.write().key(&k, n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                for (pos, i) in order.iter().copied().enumerate() {
                    {
                        let lead = LEADS[i];
                        let sc = l.shown(i, n);
                        let (tw, th) = tier(l.scores[i]);
                        let is_open = l.open == Some(i);
                        rsx! {
                            li { key: "{i}", class: "oxbk-ld-item", "data-open": "{is_open}",
                                button { class: "oxbk-ld-row", r#type: "button", "aria-expanded": "{is_open}", "aria-controls": "oxbk-ld-d{i}", tabindex: if pos == l.focus { "0" } else { "-1" },
                                    "aria-label": "{lead.0}, {lead.1}, score {l.scores[i]}, {tw}: {lead.4}",
                                    onclick: move |_| act(format!("ld-open-{i}")),
                                    span { class: "oxbk-ld-logo", "aria-hidden": "true", "{lead.0.chars().next().unwrap_or('?')}" }
                                    span { class: "oxbk-ld-who",
                                        span { class: "oxbk-ld-l1", b { "{lead.0}" } span { class: "oxaik-mute oxbk-ld-role", "{lead.1} · {lead.2}" } }
                                        span { class: "oxaik-mute oxbk-ld-why", "{lead.4}" }
                                    }
                                    span { class: "oxaik-mute oxbk-ld-ago", "{ago(u64::from(lead.5) * 60_000)}" }
                                    span { class: "oxbk-ld-tier", "data-hue": "{th}", "{tw}" }
                                    {ring(sc, th)}
                                }
                                if is_open {
                                    div { id: "oxbk-ld-d{i}", class: "oxbk-ld-det oxaik-fade",
                                        div { class: "oxaik-row", for (k, s) in lead.6.iter().enumerate() { span { key: "{k}", class: "oxaik-pill", "{s}" } } }
                                        ul { class: "oxbk-ld-bd", "aria-label": "Score breakdown",
                                            for (k, f) in lead.7.iter().enumerate() {
                                                li { key: "{k}", span { "{f.0}" } span { class: "oxaik-bar", span { style: "width:{f64::from(f.1) / f64::from(f.2) * 100.0}%" } } span { class: "oxaik-num", "{f.1} / {f.2}" } }
                                            }
                                        }
                                        div { class: "oxaik-row oxbk-ld-push",
                                            div { class: "oxaik-seg", role: "radiogroup", "aria-label": "Push to",
                                                for (k, d) in DESTS.iter().enumerate() {
                                                    button { key: "{k}", class: cls("oxaik-sg", l.dest == k), r#type: "button", role: "radio", "aria-checked": "{l.dest == k}", onclick: move |_| act(format!("ld-dest-{k}")), {mark(d.1, 12.0)} " {d.0}" }
                                                }
                                            }
                                            if let Some(d) = l.pushed(i, n) {
                                                span { class: "oxbk-st", "data-hue": "green", {ico("check-circle", 12.0)} span { "In {DESTS[d].0}" } }
                                            } else if l.pushing(i, n) {
                                                span { class: "oxaik-row oxaik-mute", {dots()} "Pushing to {DESTS[l.dest].0}" }
                                            } else if let Some(Push::Failed(d)) = l.push[i] {
                                                span { class: "oxbk-st", "data-hue": "red", {ico("x-circle", 12.0)} span { "Couldn't reach {DESTS[d].0}" } }
                                                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ld-retry".into()), {ico("retry", 12.0)} "Retry" }
                                            } else {
                                                button { class: "oxaik-btn sm pri oxbk-ld-go", r#type: "button", onclick: move |_| act("ld-push".into()), {ico("up", 12.0)} "Push to {DESTS[l.dest].0}" }
                                            }
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
