//! Agent routines (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::routines::{FREQS, Routines, Tick, schedule};

use super::{status, switch};
use crate::aikit::{cls, ico, key_of, live, mark, seg};

/// Agent routines: scheduled agents in one card. Each row reads as a sentence: the routine's
/// name and an on/off switch; its schedule in plain words with a live countdown and the last
/// run; then the sources it reads, where it posts and the model as small marks (one sentence for
/// screen readers) beside its last 14 runs as ticks. Run now plays the steps on a thin progress
/// line in the routine's colour and slides a new tick in; a schedule that comes due runs by
/// itself. Opening a row (Enter; arrow keys move between rows) shows the latest report, the
/// instruction and a frequency switch.
#[component]
pub fn AgentRoutines(
    /// Host clock.
    now: ClockMs,
    /// The next manual run of Error triage fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Open this row at the start.
    #[props(default)]
    open: Option<usize>,
    /// Show the run ticks.
    #[props(default = true)]
    ticks: bool,
) -> Element {
    let mut ro = use_signal(move || {
        let mut r = Routines::new(fail, now.0);
        r.open = open;
        r
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = ro.read().clone();
    let mut act = move |id: String| {
        let x = ro.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    rsx! {
        section { class: "oxaik oxbk oxbk-ro", "aria-label": "Agent routines",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Routines" } p { class: "oxaik-sub", "Timed agents and where their results land." } }
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ro-new".into()), {ico("plus", 12.0)} "New routine" }
            }
            ul { class: "oxbk-ro-list",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "Home" | "End") {
                        e.prevent_default();
                        ro.write().key(&k, n);
                    }
                },
                for (i, r) in s.list.iter().enumerate() {
                    {
                        let prog = s.progress(i, n);
                        let (last, lt) = s.last_line(i, n);
                        let hist = s.history(i, n);
                        let is_open = s.open == Some(i);
                        let running = s.active(i, n).is_some();
                        let enabled = r.enabled;
                        let run_label = if running { format!("Stop {}", r.name) } else { format!("Run {} now", r.name) };
                        let prog_hue = if prog.as_ref().is_some_and(|p| p.3) { "red" } else { r.hue };
                        rsx! {
                            li { key: "{i}-{r.name}", class: cls("oxbk-ro-row", is_open), "data-off": "{!enabled}",
                                div { class: "oxbk-ro-l1",
                                    button { class: "oxbk-ro-name", r#type: "button", "aria-expanded": "{is_open}", tabindex: if s.focus == i { "0" } else { "-1" },
                                        onclick: move |_| act(format!("ro-open-{i}")),
                                        "{r.name}"
                                    }
                                    button { class: "oxaik-btn sm oxbk-ro-run", r#type: "button", "aria-label": "{run_label}",
                                        onclick: move |_| act(format!("ro-run-{i}")),
                                        {ico(if running { "stop" } else { "play" }, 11.0)} span { class: "oxbk-ro-runw", if running { "Stop" } else { "Run now" } }
                                    }
                                    {switch(&format!("{} on", r.name), enabled, EventHandler::new(move |_| act(format!("ro-toggle-{i}"))))}
                                }
                                p { class: "oxbk-ro-l2 oxaik-mute",
                                    span { "{schedule(r)}" }
                                    span { class: "oxaik-num", "· {s.next_line(i, n)}" }
                                    span { class: "oxbk-ro-last", "data-hue": "{lt.hue()}", {ico(match lt { Tick::Ok => "check", Tick::Failed => "x", Tick::Skipped => "minus" }, 11.0)} "{last}" }
                                }
                                div { class: "oxbk-ro-l3",
                                    span { class: "oxaik-sr", "{s.marks_line(i)}" }
                                    span { class: "oxbk-ro-marks", "aria-hidden": "true",
                                        for m in r.sources.iter() { {mark(m, 13.0)} }
                                        {ico("chevron-right", 11.0)}
                                        {mark("slack", 13.0)} span { class: "oxbk-ro-dest", "{r.dest}" }
                                        span { class: "oxbk-ro-sep" }
                                        {mark(r.model.0, 13.0)}
                                    }
                                    if ticks {
                                        span { class: "oxbk-ro-ticks", role: "group", "aria-label": "Last runs: {s.strip_label(i, n)}",
                                            for (k, t) in hist.iter().enumerate() {
                                                i { key: "{k}", "data-hue": "{t.hue()}", "data-t": "{t.word()}", title: "{t.word()}" }
                                            }
                                        }
                                    }
                                }
                                if let Some((k, label, f, failed)) = prog.clone() {
                                    div { class: "oxbk-ro-prog", "data-hue": "{prog_hue}",
                                        div { class: "oxaik-bar", role: "progressbar", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{(f * 100.0) as u32}", "aria-valuetext": "{label}", span { style: "width:{f * 100.0}%" } }
                                        p { class: "oxaik-row",
                                            if failed { {status("Failed", "red")} span { "{label}: the token for this source expired" } }
                                            else { span { class: "oxaik-shim", "{label}" } span { class: "oxaik-mute oxaik-num", "step {k + 1}" } }
                                        }
                                    }
                                }
                                if is_open {
                                    div { class: "oxbk-ro-open oxaik-fade",
                                        div { class: "oxbk-card",
                                            p { class: "oxbk-cap", "Latest report" }
                                            ul { class: "oxbk-ro-rep", for (k, l) in s.report(i).into_iter().enumerate() { li { key: "{k}", "{l}" } } }
                                        }
                                        div { class: "oxbk-card",
                                            p { class: "oxbk-cap", "Instruction" }
                                            p { "{r.instruction}" }
                                        }
                                        div { class: "oxaik-row oxaik-between",
                                            span { class: "oxaik-mute", "Runs" }
                                            {seg("Frequency", FREQS.iter().map(|f| f.to_string()).collect(), r.freq,
                                                EventHandler::new(move |f| act(format!("ro-freq-{i}-{f}"))),
                                                EventHandler::new(move |k: String| { let x = ro.write().key(&k, n); if let Some(x) = x { said.set(x) } }))}
                                        }
                                        if running { p { class: "oxaik-mute", "Simulated: nothing is read or posted in this preview." } }
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
