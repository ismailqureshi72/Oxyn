//! Call insights (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::call::{CALL, CLIP_S, Call, KINDS, MOMENTS, RISK, SPEAKERS, STEPS, SUMMARY};

use crate::aikit::{ico, key_of, live, mark};

const BARS: usize = 48;

/// Call insights: a sales call in one card. The header carries the prospect, the call, the
/// meeting platform mark, date and length and a deal risk chip. Talk time is one bar split by
/// speaker and grouped by side; key moments (pricing, competitor, objection) sit on a rail for
/// the whole call; selecting one shows its quote with a play button and a seekable waveform.
/// The AI summary streams once with Copy; checking off next steps lowers the derived risk.
#[component]
pub fn CallInsights(
    /// Host clock.
    now: ClockMs,
    /// Tour the moments until someone interacts.
    #[props(default = true)]
    tour: bool,
    /// Next steps already done.
    #[props(default)]
    done: usize,
) -> Element {
    let mut ci = use_signal(move || Call::new(tour, done, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let c = ci.read().clone();
    let mut act = move |id: String| {
        let x = ci.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut keyed = move |t: &'static str, k: String| {
        let x = ci.write().key(t, &k, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let cur = c.current(n);
    let mo = MOMENTS[cur];
    let (pos, playing) = c.position(n);
    let (risk, _) = c.risk();
    let len = CALL.4;
    let summary = c.summary(n);
    let summary_done = summary.len() >= SUMMARY.len();
    rsx! {
        section { class: "oxaik oxbk oxbk-ci", "aria-label": "Call insights: {CALL.0}",
            header { class: "oxaik-hd",
                div { class: "oxaik-row oxbk-ci-id",
                    span { class: "oxbk-ci-logo", "aria-hidden": "true", "P" }
                    div { class: "oxaik-hl",
                        h2 { class: "oxaik-h", "{CALL.0} · {CALL.1}" }
                        p { class: "oxaik-sub", {mark(CALL.2, 13.0)} span { class: "oxbk-ci-plat", "Zoom" } "· {CALL.3} · {len / 60} min" }
                    }
                }
                div { class: "oxaik-row",
                    span { class: "oxbk-ci-ppl", "aria-label": "Attendees",
                        for (k, s) in SPEAKERS.iter().enumerate() { span { key: "{k}", class: "oxbk-av oxbk-av-you", "data-side": "{s.2}", title: "{s.0}", "{s.0.chars().next().unwrap_or('?')}" } }
                    }
                    span { class: "oxaik-pill", "data-hue": "{RISK[risk].1}", {ico(if risk == 0 { "check-circle" } else { "alert" }, 11.0)} "{RISK[risk].0}" }
                }
            }
            div { class: "oxbk-ci-talk",
                p { class: "oxbk-cap", "Talk time" }
                div { class: "oxbk-ci-tbar", "aria-hidden": "true",
                    for (k, s) in SPEAKERS.iter().enumerate() { i { key: "{k}", "data-sp": "{k}", style: "width:{s.3}%" } }
                }
                ul { class: "oxbk-ci-leg",
                    for (k, s) in SPEAKERS.iter().enumerate() {
                        li { key: "{k}", i { "data-sp": "{k}" } span { class: "oxbk-ci-nm", "{s.0}" } span { class: "oxaik-mute oxbk-ci-ti", "{s.1}" } span { class: "oxaik-num", "{s.3}%" } }
                    }
                }
            }
            div { class: "oxbk-ci-mom",
                p { class: "oxbk-cap", "Key moments" }
                div { class: "oxbk-ci-rail", role: "radiogroup", "aria-label": "Key moments",
                    onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if k.starts_with("Arrow") || k == "Home" || k == "End" { e.prevent_default(); keyed("markers", k) } },
                    i { class: "oxbk-ci-fill", style: "width:{f64::from(mo.0) / f64::from(len) * 100.0}%" }
                    for (k, m) in MOMENTS.iter().enumerate() {
                        button { key: "{k}", class: "oxbk-ci-mk", r#type: "button", role: "radio", "aria-checked": "{k == cur}", tabindex: if k == cur { "0" } else { "-1" },
                            "data-hue": "{KINDS[m.1].2}", style: "left:{f64::from(m.0) / f64::from(len) * 100.0}%",
                            "aria-label": "{KINDS[m.1].0} at {m.0 / 60}:{m.0 % 60:02}",
                            onclick: move |_| act(format!("ci-m-{k}")), {ico(KINDS[m.1].1, 12.0)} }
                    }
                }
                div { class: "oxbk-ci-well", key: "{cur}",
                    div { class: "oxaik-row oxbk-ci-wh",
                        span { class: "oxbk-st", "data-hue": "{KINDS[mo.1].2}", {ico(KINDS[mo.1].1, 11.0)} span { "{KINDS[mo.1].0}" } }
                        span { class: "oxaik-mute oxaik-num", "{SPEAKERS[mo.2].0} · {c.stamp(n)}" }
                    }
                    p { class: "oxbk-ci-q oxaik-fade", "“{mo.3}”" }
                    div { class: "oxaik-row oxbk-ci-player",
                        button { class: "oxaik-ib", r#type: "button", "aria-label": if playing { "Pause" } else { "Play clip" }, onclick: move |_| act("ci-play".into()), {ico(if playing { "pause" } else { "play" }, 12.0)} }
                        div { class: "oxbk-ci-wave", role: "slider", tabindex: "0", "aria-label": "Clip position", "aria-valuemin": "0", "aria-valuemax": "{CLIP_S}", "aria-valuenow": "{pos}", "aria-valuetext": "{c.stamp(n)}",
                            onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if k.starts_with("Arrow") || k == "Home" || k == "End" { e.prevent_default(); keyed("wave", k) } },
                            for b in 0..BARS {
                                {
                                    let h = 20 + ((b * 37 + cur * 11) % 70);
                                    let on = b * CLIP_S as usize <= pos as usize * BARS;
                                    rsx! { i { key: "{b}", "data-on": "{on}", style: "height:{h}%" } }
                                }
                            }
                        }
                        span { class: "oxaik-mute oxaik-num", "0:{pos:02} / 0:{CLIP_S:02}" }
                    }
                }
            }
            div { class: "oxbk-ci-sum",
                div { class: "oxaik-row oxbk-ci-sh", p { class: "oxbk-cap", {mark("claude", 11.0)} "AI summary" }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Copy summary", disabled: !summary_done, onclick: move |_| act("ci-copy".into()), {ico(if c.copied { "check" } else { "copy" }, 12.0)} } }
                p { "{summary}" }
            }
            div { class: "oxbk-ci-next",
                p { class: "oxbk-cap", "Next steps" }
                ul {
                    for (k, s) in STEPS.iter().enumerate() {
                        li { key: "{k}", class: "oxbk-ci-step", "data-done": "{c.done[k]}",
                            label {
                                input { r#type: "checkbox", checked: c.done[k], onchange: move |_| act(format!("ci-step-{k}")) }
                                span { class: "oxbk-ci-stt", "{s.0}" }
                            }
                            span { class: "oxaik-mute oxbk-ci-own", "{s.1}" }
                            span { class: "oxaik-mute oxaik-num", "{s.2}" }
                        }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
