//! Training run (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::training::{
    DATASET, ENDPOINT, EPOCHS, JOB, MODEL, Status, TOTAL, Training, dur, lr, sci, train_loss,
    train_smooth, val_loss,
};

use super::{dots, ico, key_of, live, mark};

/// Training run: a fine-tuning job card. A raw (faint) and smoothed train loss plus validation
/// loss draw across a fixed step axis whose bands double as epoch progress, so nothing rescales
/// mid-run. Stats show the learning rate, elapsed time and ETA; the newest three events list
/// evals, checkpoints and epochs. Pause and a two-press Cancel while running, Retry from the last
/// checkpoint after a failure, and Deploy on completion. Arrow keys inspect samples on the
/// focused chart (Shift jumps ten, Escape returns to the latest).
#[component]
pub fn TrainingRun(
    /// Host clock.
    now: ClockMs,
    /// Fails out of memory at step 940 (Retry demo).
    #[props(default)]
    oom: bool,
    /// Playback speed, percent.
    #[props(default = 100)]
    speed: u64,
    /// Show the event log.
    #[props(default = true)]
    events: bool,
) -> Element {
    let t0 = now.0;
    let mut tr = use_signal(move || Training::new(oom, speed, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let t = tr.read().clone();
    let mut act = move |id: String| {
        let x = tr.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let st = t.status(n);
    let step = t.step(n);
    let ss = t.samples(n);
    let vs = t.vals(n);
    let (w, h) = (600.0f64, 160.0f64);
    let (ymin, ymax) = (0.2f64, 2.9f64);
    let x = |s: u64| s as f64 / TOTAL as f64 * w;
    let y = |v: f64| h - (v - ymin) / (ymax - ymin) * h;
    let raw: Vec<String> = ss
        .iter()
        .map(|s| format!("{:.1},{:.1}", x(*s), y(train_loss(*s))))
        .collect();
    let smooth: Vec<String> = ss
        .iter()
        .map(|s| format!("{:.1},{:.1}", x(*s), y(train_smooth(*s))))
        .collect();
    let val: Vec<String> = vs
        .iter()
        .map(|s| format!("{:.1},{:.1}", x(*s), y(val_loss(*s))))
        .collect();
    let insp = t.inspect.and_then(|i| ss.get(i).copied());
    let confirming = t.confirming(n);
    let dep = t.deploy_state(n);
    let copied = t.copied.is_some_and(|c| n < c + 1_500);
    let evs = t.events(n);
    let recent: Vec<_> = evs.iter().rev().take(3).cloned().collect();
    let running = matches!(
        st,
        Status::Running | Status::Starting | Status::Queued | Status::Paused
    );
    rsx! {
        section { class: "oxaik oxaik-trn", "aria-label": "Fine-tuning job {JOB}",
            header { class: "oxaik-hd",
                div { class: "oxaik-row oxaik-trn-id",
                    {mark(MODEL.1, 24.0)}
                    div { class: "oxaik-hl",
                        h3 { class: "oxaik-h", "{JOB}" }
                        p { class: "oxaik-sub", "{MODEL.0} · {MODEL.2}" }
                    }
                }
                span { class: "oxaik-pill", "data-hue": "{st.hue()}",
                    {match st {
                        Status::Running | Status::Starting => rsx! { {dots()} },
                        Status::Completed => rsx! { {ico("check", 11.0)} },
                        Status::Failed => rsx! { {ico("alert", 11.0)} },
                        Status::Paused => rsx! { {ico("pause", 11.0)} },
                        _ => rsx! { {ico("clock", 11.0)} },
                    }}
                    span { key: "{st.word()}", class: "oxaik-fade", "{st.word()}" }
                }
            }
            div { class: "oxaik-trn-chart", tabindex: "0", role: "group", "aria-label": "Loss chart. {t.readout(n)}",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End" | "Escape") {
                        e.prevent_default();
                        let x = tr.write().key(&k, e.modifiers().shift(), n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                div { class: "oxaik-row oxaik-trn-leg",
                    span { class: "oxaik-row", i { class: "sw tr" } "Train" }
                    span { class: "oxaik-row", i { class: "sw val" } span { class: "full", "Validation" } span { class: "short", "Val" } }
                    span { class: "oxaik-mute oxaik-num oxaik-trn-read", "{t.readout(n)}" }
                }
                div { class: "oxaik-trn-plot",
                    for e in 1..EPOCHS { i { key: "{e}", class: "oxaik-trn-ep", style: "left:{e * 100 / EPOCHS}%" } }
                    svg { view_box: "0 0 600 160", preserve_aspect_ratio: "none", "aria-hidden": "true",
                        polyline { class: "raw", points: "{raw.join(\" \")}", "vector-effect": "non-scaling-stroke" }
                        polyline { class: "sm", points: "{smooth.join(\" \")}", "vector-effect": "non-scaling-stroke" }
                        polyline { class: "val", points: "{val.join(\" \")}", "vector-effect": "non-scaling-stroke" }
                    }
                    if let Some(s) = insp { i { class: "oxaik-trn-cur", style: "left:{s as f64 / TOTAL as f64 * 100.0}%" } }
                    if ss.is_empty() {
                        p { class: "oxaik-trn-wait oxaik-row oxaik-mute",
                            if st == Status::Queued { "Waiting for 8× H100 · 2nd in queue" } else if st == Status::Starting { {dots()} "Loading {MODEL.0} and the dataset" } else { "No samples" }
                        }
                    }
                }
                div { class: "oxaik-trn-epochs", "aria-hidden": "true",
                    for e in 0..EPOCHS {
                        {
                            let lo = e * TOTAL / EPOCHS;
                            let f = ((step.saturating_sub(lo)) as f64 / (TOTAL / EPOCHS) as f64).min(1.0);
                            rsx! { span { key: "{e}", i { style: "width:{f * 100.0}%" } } }
                        }
                    }
                }
                div { class: "oxaik-row oxaik-between oxaik-mute oxaik-num oxaik-trn-ax", span { "Step {step} of {TOTAL}" } span { "Epoch {t.epoch(n)} of {EPOCHS}" } }
            }
            if st == Status::Completed {
                dl { class: "oxaik-trn-stats",
                    for (k, v) in t.metrics() { div { key: "{k}", dt { "{k}" } dd { class: "oxaik-num", "{v}" } } }
                }
            } else {
                dl { class: "oxaik-trn-stats",
                    div { dt { "Learning rate" } dd { class: "oxaik-num", "{sci(lr(step))}" } }
                    div { dt { "Elapsed" } dd { class: "oxaik-num", "{dur(t.elapsed(n))}" } }
                    div { dt { "ETA" } dd { class: "oxaik-num", if st == Status::Queued { "About {dur(t.eta(n))}" } else if running { "{dur(t.eta(n))}" } else { "—" } } }
                    div { dt { "Train loss" } dd { class: "oxaik-num", if ss.is_empty() { "—" } else { "{train_smooth(step):.3}" } } }
                }
            }
            if events && !recent.is_empty() {
                ul { class: "oxaik-trn-ev",
                    for (k, s, txt) in recent {
                        li { key: "{k}-{s}", "data-kind": "{k}",
                            {ico(match k { "checkpoint" => "archive", "eval" => "table", "epoch" => "check", "error" => "alert", "warning" => "ban", _ => "info" }, 12.0)}
                            span { "{txt}" }
                            span { class: "oxaik-mute oxaik-num", "step {s}" }
                        }
                    }
                }
            }
            footer { class: "oxaik-row oxaik-between oxaik-trn-foot",
                span { class: "oxaik-row oxaik-mute oxaik-trn-ds",
                    if st == Status::Failed { span { class: "oxaik-ps-err oxaik-row", {ico("alert", 12.0)} "CUDA out of memory at step 940" } }
                    else if dep == "done" {
                        span { class: "oxaik-row", {ico("check", 12.0)} code { class: "oxaik-mono", "{ENDPOINT}" }
                            button { class: "oxaik-ib", r#type: "button", "aria-label": if copied { "Copied" } else { "Copy endpoint" }, onclick: move |_| act("tr-copy".into()), {ico(if copied { "check" } else { "copy" }, 12.0)} } }
                    } else {
                        {mark(DATASET.1, 14.0)} "{DATASET.0} · {oxyn_core::aikit::thousands(u64::from(DATASET.2))} examples"
                    }
                }
                div { class: "oxaik-ctl",
                    if st == Status::Running {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tr-pause".into()), {ico("pause", 11.0)} "Pause" }
                    }
                    if st == Status::Paused {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tr-resume".into()), {ico("play", 11.0)} "Resume" }
                    }
                    if running {
                        button { class: if confirming { "oxaik-btn sm danger" } else { "oxaik-btn sm" }, r#type: "button", onclick: move |_| act("tr-cancel".into()),
                            if confirming { "Confirm cancel" } else { "Cancel" } }
                    }
                    if st == Status::Failed {
                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("tr-retry".into()), {ico("retry", 12.0)} "Retry from checkpoint" }
                    }
                    if matches!(st, Status::Cancelled | Status::Failed) || (st == Status::Completed && dep == "done") {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tr-restart".into()), "Run again" }
                    }
                    if st == Status::Completed && dep != "done" {
                        button { class: "oxaik-btn sm pri", r#type: "button", disabled: dep == "loading", "aria-busy": "{dep == \"loading\"}", onclick: move |_| act("tr-deploy".into()),
                            if dep == "loading" { {dots()} "Deploying" } else { {ico("bolt", 12.0)} "Deploy" } }
                    }
                }
            }
            {live(&said())}
        }
    }
}
