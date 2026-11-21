//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Fan-out Grid**
//! ([`TaskGrid`]), the built form of the `fanout-grid` spec. Behaviour and look only; the model
//! is `oxyn_core::kit::fanfid`. Styles: `fanfid.css` (classes `oxfo-*`), in `PRIMITIVES_CSS`.
//! Follows the nearest `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as
//! `now`.

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::fanfid::{FoStatus, FoTask, fo_demo, fo_secs, fo_view},
};

static SEQ: AtomicUsize = AtomicUsize::new(0);
/// Rest after the job finishes before the demo loops, ms.
const REST_MS: u32 = 5_000;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// Fan-out Grid (affectUI parity): one cell per task of a job split across workers – queued grey,
/// running blue with a shine, retrying amber, done green, failed red – with live counts that
/// filter the grid, the cost so far, the time left, a details card per task and Retry failed.
#[component]
pub fn TaskGrid(
    /// The tasks (the demo's 240 when `None`).
    #[props(default)]
    tasks: Option<Vec<FoTask>>,
    /// Ms into the job; `None` replays the demo (looping).
    #[props(default)]
    at: Option<u32>,
    /// Workers (for the estimate).
    #[props(default = 12)]
    workers: u32,
    /// The job line.
    #[props(default = "Enrich 240 sign-up domains".to_string())]
    job: String,
    /// Heading.
    #[props(default = "Fan-out".to_string())]
    title: String,
    /// A status filter from the start.
    #[props(default)]
    filter: Option<FoStatus>,
    /// A task pinned from the start (index).
    #[props(default)]
    pinned: Option<usize>,
    /// The clock (drives the replay).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Retry failed was pressed (the failed task ids).
    #[props(default)]
    on_retry: EventHandler<Vec<String>>,
    /// A task was pinned (index) or released.
    #[props(default)]
    on_select: EventHandler<Option<usize>>,
) -> Element {
    let uid = use_hook(|| {
        format!(
            "{:x}{}",
            now.0 % 0xFFFFF,
            SEQ.fetch_add(1, Ordering::Relaxed)
        )
    });
    let t0 = use_signal(move || now);
    let mut flt = use_signal(move || filter);
    let mut pin = use_signal(move || pinned);
    let mut hover = use_signal(|| Option::<usize>::None);
    let mut cur = use_signal(|| 0usize);
    let mut retried = use_signal(|| Option::<u32>::None);
    let ts = tasks.clone().unwrap_or_else(|| fo_demo(240, 12));
    let n = ts.len();
    let job_end = ts.iter().map(|t| t.end).max().unwrap_or(0);
    let t = at.unwrap_or_else(|| {
        let el = now.since(t0()).as_millis() as u32;
        (el % (job_end + REST_MS)).min(job_end + REST_MS)
    });
    // A replay that loops forgets the hand retry.
    let rt = retried().filter(|r| *r <= t);
    let v = fo_view(&ts, t, rt, workers);
    let done = v.counts[0];
    let failed_ids: Vec<String> = ts
        .iter()
        .zip(&v.status)
        .filter(|(_, s)| **s == FoStatus::Failed)
        .map(|(x, _)| x.id.clone())
        .collect();
    let shown = pin().or(hover());
    let tip_id = format!("oxfo-{uid}-tip");
    let cell_id = |i: usize| format!("oxfo-{uid}-c{i}");
    let active = cell_id(cur().min(n.saturating_sub(1)));
    let cost_s = format!("${:.2}", v.cost);
    let right = if v.finished && failed_ids.is_empty() {
        format!("Finished in {}", fo_secs(v.ended_at))
    } else if v.finished {
        format!("{} failed", failed_ids.len())
    } else {
        format!("~{} left", fo_secs(v.left))
    };
    let pc = done * 100 / n.max(1);
    let status = if v.finished {
        format!(
            "Finished: {} done, {} failed, {cost_s}.",
            done,
            failed_ids.len()
        )
    } else {
        format!("{}% done.", (pc / 10) * 10)
    };
    let nfailed = failed_ids.len();
    let detail = shown.and_then(|i| ts.get(i).map(|x| (i, x.clone())));

    rsx! {
        section { class: "oxfo", "data-style": style_attr(&style), "data-done": v.finished, "aria-label": "{title}",
            header { class: "oxfo-bar",
                span { class: "oxfo-logo", {icon("M3 8h3M6 8l4-4.5M6 8h7M6 8l4 4.5M10 3.5h3M10 12.5h3", 15)} }
                h2 { class: "oxfo-ttl", "{title}" }
                span { class: "oxfo-job", "{job}" }
                span { class: "oxfo-flex" }
                span { class: "oxfo-nums",
                    span { b { "{done}" } " / {n}" }
                    span { "{cost_s}" }
                    span { class: "oxfo-left", "data-ok": v.finished && nfailed == 0, "{right}" }
                }
            }
            div { class: "oxfo-grid", role: "grid", tabindex: "0", "aria-label": "Tasks", "aria-activedescendant": "{active}",
                "data-filter": flt().map(|f| f.id()),
                onkeydown: move |e: KeyboardEvent| {
                    let c = cur();
                    let k = match e.key() {
                        Key::ArrowRight => Some((c + 1).min(n - 1)),
                        Key::ArrowLeft => Some(c.saturating_sub(1)),
                        Key::ArrowDown => Some((c + 20).min(n - 1)),
                        Key::ArrowUp => Some(c.saturating_sub(20)),
                        Key::Home => Some(0),
                        Key::End => Some(n - 1),
                        _ => None,
                    };
                    if let Some(k) = k { e.prevent_default(); cur.set(k); hover.set(Some(k)); return; }
                    match e.key() {
                        Key::Enter => { e.prevent_default(); let v = if pin() == Some(c) { None } else { Some(c) }; pin.set(v); on_select.call(v); }
                        Key::Character(s) if s == " " => { e.prevent_default(); let v = if pin() == Some(c) { None } else { Some(c) }; pin.set(v); on_select.call(v); }
                        Key::Escape => { pin.set(None); hover.set(None); on_select.call(None); }
                        _ => {}
                    }
                },
                onmouseleave: move |_| hover.set(None),
                for (i, (x, s)) in ts.iter().zip(v.status.iter()).enumerate() {
                    {
                        let dim = flt().is_some_and(|f| f != *s);
                        let on = shown == Some(i);
                        let delay = (i % 17) as f32 * 0.09;
                        let cid = cell_id(i);
                        let lab = format!("{}: {}", x.label, s.word());
                        rsx! {
                            i { class: "oxfo-cell", key: "{x.id}", "data-k": "{x.id}", id: "{cid}", role: "gridcell", "aria-label": "{lab}",
                                "aria-describedby": if on { Some(tip_id.clone()) } else { None },
                                "data-st": s.id(), "data-dim": dim, "data-on": on, "data-pin": pin() == Some(i),
                                style: "--d:{delay}s",
                                onmouseenter: move |_| hover.set(Some(i)),
                                onclick: move |_| { cur.set(i); let v = if pin() == Some(i) { None } else { Some(i) }; pin.set(v); on_select.call(v); },
                            }
                        }
                    }
                }
            }
            if let Some((i, x)) = detail {
                {
                    let s = v.status[i];
                    let took = x.took(t, rt);
                    let tries = x.attempts(t, rt);
                    let line = if s == FoStatus::Queued {
                        format!("Worker {} · waiting", x.worker)
                    } else {
                        let c = if s == FoStatus::Done { format!(" · ${:.2}", x.cost) } else { String::new() };
                        let a = if tries > 1 { format!(" · {tries} tries") } else { String::new() };
                        format!("Worker {} · {:.1}s{c}{a}", x.worker, took as f64 / 1000.0)
                    };
                    rsx! {
                        div { class: "oxfo-tip", id: "{tip_id}", role: "tooltip", "data-pin": pin().is_some(),
                            div { class: "oxfo-th", b { "{x.label}" } span { class: "oxfo-pill", "data-st": s.id(), "{s.word()}" } }
                            span { class: "oxfo-tl", "{line}" }
                            if matches!(s, FoStatus::Done | FoStatus::Failed) { span { class: "oxfo-tr", "data-st": s.id(), "{x.result}" } }
                        }
                    }
                }
            }
            if !v.finished || nfailed > 0 {
                div { class: "oxfo-counts",
                    for (k, st) in FoStatus::COUNTED.iter().enumerate() {
                        if *st != FoStatus::Failed || v.counts[3] > 0 {
                            {
                                let st = *st;
                                let on = flt() == Some(st);
                                let cnt = v.counts[k];
                                let word = st.id();
                                rsx! {
                                    button { class: "oxfo-cnt", key: "{word}", "data-k": "{word}", r#type: "button", "aria-pressed": on, "data-st": word,
                                        "aria-label": "Show only {word} ({cnt})",
                                        onclick: move |_| { let v = if flt() == Some(st) { None } else { Some(st) }; flt.set(v); },
                                        i {} b { "{cnt}" } span { "{word}" }
                                    }
                                }
                            }
                        }
                    }
                    span { class: "oxfo-flex" }
                    if nfailed > 0 {
                        button { class: "oxfo-retry", r#type: "button",
                            onclick: move |_| { retried.set(Some(t)); if flt() == Some(FoStatus::Failed) { flt.set(None); } on_retry.call(failed_ids.clone()); },
                            {icon("M13 8a5 5 0 1 1-1.5-3.6M13 2.5v3h-3", 12)} "Retry failed"
                        }
                    }
                }
            }
            p { class: "oxfo-status", role: "status", "{status}" }
        }
    }
}
