//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32): the exact **Agent Queue**
//! ([`AgentQueue`]). Behaviour and look only; the model is `oxyn_core::parity::queuefid`.
//! Styles: `queuefid.css` (classes `oxqu-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::queuefid::{
        JobKind, JobStatus, QUEUE_LOOP_MS, QueueEvent, QueueJob, depth_top, job_label, queue_at,
        queue_said, queue_secs,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn kind_icon(k: JobKind) -> Element {
    let d = match k {
        JobKind::Llm => {
            "M12 3l1.8 4.7L18.5 9.5l-4.7 1.8L12 16l-1.8-4.7L5.5 9.5l4.7-1.8zM18 15l.8 2.2 2.2.8-2.2.8L18 21l-.8-2.2-2.2-.8 2.2-.8z"
        }
        JobKind::Tool => {
            "M14.5 6.5a4 4 0 0 0-5.6 4.9L4 16.3V20h3.7l4.9-4.9a4 4 0 0 0 4.9-5.6l-2.5 2.5-2.4-.6-.6-2.4z"
        }
        JobKind::Agent => "M12 4a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM5 20a7 7 0 0 1 14 0",
        JobKind::Human => "M9 11l3 3 8-8M20 12v6a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h9",
    };
    rsx! {
        svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.9",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } }
    }
}

fn job_icon(j: &QueueJob) -> Element {
    rsx! {
        span { class: "oxqu-ji", "data-kind": "{j.kind.id()}",
            if j.brand.is_empty() { {kind_icon(j.kind)} } else { BrandIcon { name: j.brand.clone(), size: 12.0, coloured: true, fallback: true } }
        }
    }
}

/// Agent Queue (affectUI): a live job queue – waiting jobs, workers running them as growing bars,
/// Done and Failed, the four numbers and a queue-depth chart where a rate limit shows as an amber
/// stretch; Pause / Resume, a worker stepper and a card per job with Move to front and Cancel.
#[component]
pub fn AgentQueue(
    /// Heading.
    #[props(default = "Queue".to_string())]
    title: String,
    /// The queue's name, muted next to the title.
    #[props(default = "agent-work".to_string())]
    name: String,
    /// Workers at the start (1–8).
    #[props(default = 4)]
    workers: usize,
    /// Freeze the demo this many ms into a pass.
    #[props(default)]
    at: Option<u64>,
    /// Start paused.
    #[props(default)]
    paused: bool,
    /// A job whose card is open at the start (its place in the queue, 0 = front).
    #[props(default)]
    open_at: Option<usize>,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Pause (`true`) or resume.
    #[props(default)]
    on_paused_change: EventHandler<bool>,
    /// The number of workers changed.
    #[props(default)]
    on_concurrency_change: EventHandler<usize>,
    /// Move to front (job id).
    #[props(default)]
    on_promote: EventHandler<String>,
    /// Cancel a waiting job (job id).
    #[props(default)]
    on_cancel: EventHandler<String>,
) -> Element {
    let start = use_signal(move || now);
    // Steering: (lap, ms into the pass, event).
    let mut events = use_signal(move || {
        if paused {
            vec![(0u64, 0i64, QueueEvent::Pause(true))]
        } else {
            vec![]
        }
    });
    let mut n_workers = use_signal(move || workers.clamp(1, 8));
    // The worker count a pass started with.
    let mut lap_workers = use_signal(move || (0u64, workers.clamp(1, 8)));
    let mut open = use_signal(move || {
        open_at.and_then(|i| {
            queue_at(at.unwrap_or(0) as i64, workers, &[])
                .queued
                .get(i)
                .map(|j| j.id.clone())
        })
    });

    let raw = now.since(start()).as_millis() as u64;
    let (lap, el) = match at {
        Some(a) => (0, a),
        None => (raw / QUEUE_LOOP_MS, raw % QUEUE_LOOP_MS),
    };
    let all = events();
    // Paused at the end of an earlier pass: this one starts paused.
    let carried = all
        .iter()
        .filter(|e| e.0 < lap)
        .filter_map(|e| match e.2 {
            QueueEvent::Pause(p) => Some(p),
            _ => None,
        })
        .next_back()
        .unwrap_or(false);
    let mut evs: Vec<(i64, QueueEvent)> = vec![];
    if carried {
        evs.push((i64::MIN / 4, QueueEvent::Pause(true)));
    }
    evs.extend(
        all.iter()
            .filter(|e| e.0 == lap)
            .map(|e| (e.1, e.2.clone())),
    );
    let first = if lap_workers().0 == lap {
        lap_workers().1
    } else {
        n_workers()
    };
    let sc = queue_at(el as i64, first, &evs);
    let t = sc.now;
    let running = sc.slots.iter().filter(|s| s.is_some()).count();
    let waiting = sc.queued.len();
    let over = waiting as u32 > sc.backpressure;
    let said = queue_said(&sc);
    let top = depth_top(&sc.depth, sc.backpressure) as f32;
    // The depth chart, 1000 × 100 units.
    let n = sc.depth.len().max(2);
    let pts: Vec<(f32, f32)> = sc
        .depth
        .iter()
        .enumerate()
        .map(|(i, d)| {
            (
                i as f32 / (n - 1) as f32 * 1000.0,
                100.0 - *d as f32 / top * 100.0,
            )
        })
        .collect();
    let mut line = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        line.push_str(&format!("{}{x:.1} {y:.1}", if i == 0 { "M" } else { "L" }));
    }
    let area = format!("{line}L1000 100L0 100Z");
    let last_y = pts.last().map_or(100.0, |p| p.1);
    let bp_y = 100.0 - sc.backpressure as f32 / top * 100.0;
    let peak = sc.depth.iter().copied().max().unwrap_or(0);
    let avg = sc.depth.iter().sum::<u32>() as f32 / sc.depth.len().max(1) as f32;
    let chart_label = format!(
        "Queue depth over the last 96s: {waiting} waiting now, peak {peak}, average {avg:.1}, backpressure line at {}, rate limited {} times.",
        sc.backpressure,
        sc.bands.len()
    );
    let pill = if sc.paused {
        ("paused", "Paused".to_string())
    } else if let Some(s) = sc.limited {
        ("limited", format!("Rate limited · {s}s"))
    } else {
        ("live", "Live".to_string())
    };
    let open_job: Option<(QueueJob, Option<(usize, usize)>)> = open().and_then(|id| {
        sc.queued
            .iter()
            .position(|j| j.id == id)
            .map(|p| (sc.queued[p].clone(), Some((p + 1, waiting))))
            .or_else(|| {
                sc.slots
                    .iter()
                    .flatten()
                    .find(|j| j.id == id)
                    .map(|j| (j.clone(), None))
            })
    });
    let per_min = sc.per_min;
    let p95 = sc.wait_p95.div_ceil(1000);
    let show_q: Vec<(usize, QueueJob)> = sc.queued.iter().cloned().enumerate().take(3).collect();
    let rest: Vec<QueueJob> = sc.queued.iter().skip(3).cloned().collect();
    let lap_k = format!("lap{lap}");
    let nw = n_workers();
    let is_paused = sc.paused;
    let stopped = sc.paused || sc.limited.is_some();

    let mut push = move |e: QueueEvent| {
        events.write().push((lap, el as i64, e));
    };
    let mut set_workers = move |v: usize| {
        if lap_workers().0 != lap {
            lap_workers.set((lap, first));
        }
        n_workers.set(v);
        events
            .write()
            .push((lap, el as i64, QueueEvent::Workers(v)));
        on_concurrency_change.call(v);
    };

    rsx! {
        div { class: "oxqu", "data-style": style_attr(&style), "data-pill": "{pill.0}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(None) },
            div { class: "oxqu-head",
                span { class: "oxqu-ic",
                    svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", "aria-hidden": "true",
                        path { d: "M4 6h16M4 12h16M4 18h10" } }
                }
                b { class: "oxqu-ttl", "{title}" }
                span { class: "oxqu-name", "{name}" }
                span { class: "oxqu-pill", "data-k": "{pill.0}", i {} "{pill.1}" }
                span { class: "oxqu-sp" }
                button { class: "oxqu-btn", r#type: "button", "aria-pressed": "{is_paused}", "aria-label": if is_paused { "Resume the queue" } else { "Pause the queue" },
                    onclick: move |_| { push(QueueEvent::Pause(!is_paused)); on_paused_change.call(!is_paused) },
                    if is_paused {
                        svg { width: "12", height: "12", view_box: "0 0 24 24", "aria-hidden": "true", path { d: "M7 4.5v15l12-7.5z", fill: "currentColor" } }
                        "Resume"
                    } else {
                        svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", "aria-hidden": "true", rect { x: "6", y: "5", width: "4", height: "14", rx: "1" } rect { x: "14", y: "5", width: "4", height: "14", rx: "1" } }
                        "Pause"
                    }
                }
                span { class: "oxqu-step", role: "group", "aria-label": "Workers",
                    button { r#type: "button", "aria-label": "One worker less", disabled: nw <= 1,
                        onclick: move |_| set_workers(n_workers().saturating_sub(1).max(1)), "−" }
                    b { role: "status", "aria-label": "{nw} workers", "{nw}" }
                    button { r#type: "button", "aria-label": "One worker more", disabled: nw >= 8,
                        onclick: move |_| set_workers((n_workers() + 1).min(8)), "+" }
                }
            }
            dl { class: "oxqu-stats",
                div { dt { "Waiting" } dd { "data-over": "{over}", "{waiting}" } }
                div { dt { "Running" } dd { "{running}" small { "/{nw}" } } }
                div { dt { "Throughput" } dd { "{per_min}" small { "/min" } } }
                div { dt { "Wait p95" } dd { "{p95}" small { "s" } } }
            }
            div { class: "oxqu-main", key: "{lap_k}", "data-lap": "{lap_k}",
                div { class: "oxqu-col", "data-k": "queue",
                    div { class: "oxqu-ch", "Queue " b { "{waiting}" } }
                    div { class: "oxqu-jobs", role: "group", "aria-label": "{said}",
                        if show_q.is_empty() { span { class: "oxqu-empty", "Nothing waiting" } }
                        for (i, j) in show_q.iter().cloned() {
                            {
                                let label = job_label(&j, t, Some((i + 1, waiting)));
                                let id = j.id.clone();
                                let is_open = open().as_deref() == Some(j.id.as_str());
                                rsx! {
                                    button { key: "{j.id}", "data-k": "{j.id}", class: "oxqu-job", "data-kind": "{j.kind.id()}", r#type: "button", "aria-label": "{label}", "aria-expanded": "{is_open}",
                                        tabindex: if i == 0 { "0" } else { "-1" },
                                        onclick: move |_| open.set(if open().as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }),
                                        {job_icon(&j)}
                                        span { class: "oxqu-jn", "{j.name}" }
                                        if j.high { span { class: "oxqu-chip", "data-k": "high", "↑" } }
                                        if j.attempt > 1 { span { class: "oxqu-chip", "data-k": "retry", "↻ {j.attempt}" } }
                                        span { class: "oxqu-jt", "{queue_secs(t - j.enqueued_at)}" }
                                    }
                                }
                            }
                        }
                        if !rest.is_empty() {
                            div { class: "oxqu-dots", "aria-hidden": "true",
                                for j in rest.iter().take(48).cloned() {
                                    i { key: "{j.id}", "data-k": "{j.id}", class: "oxqu-dot", "data-kind": "{j.kind.id()}" }
                                }
                            }
                        }
                    }
                }
                div { class: "oxqu-col", "data-k": "workers",
                    div { class: "oxqu-ch", "Workers " b { "{running}/{nw}" } }
                    div { class: "oxqu-ws",
                        for (w, (wk, slot)) in sc.workers.iter().cloned().zip(sc.slots.iter().cloned()).enumerate() {
                            div { key: "{w}", "data-k": "{w}", class: "oxqu-w",
                                span { class: "oxqu-wt", BrandIcon { name: wk.brand.clone(), size: 14.0, coloured: true, fallback: true } }
                                b { class: "oxqu-wn", "{wk.name}" }
                                if let Some(j) = slot {
                                    {
                                        let label = job_label(&j, t, None);
                                        let id = j.id.clone();
                                        let pct = (j.progress * 100.0).clamp(2.0, 100.0);
                                        let el_ms = t - j.started_at.unwrap_or(t);
                                        rsx! {
                                            button { key: "{j.id}", "data-k": "{j.id}", class: "oxqu-run", "data-kind": "{j.kind.id()}", "data-stop": "{stopped}", r#type: "button", "aria-label": "{label}",
                                                tabindex: "-1",
                                                onclick: move |_| open.set(if open().as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }),
                                                i { class: "oxqu-bar", style: "width:{pct:.1}%" }
                                                {job_icon(&j)}
                                                span { class: "oxqu-jn", "{j.name}" }
                                                if stopped { span { class: "oxqu-jt", "Paused" } } else { span { class: "oxqu-jt", "data-k": "run", "{queue_secs(el_ms)}" } }
                                            }
                                        }
                                    }
                                } else {
                                    span { class: "oxqu-idle", if is_paused { "Paused" } else { "Idle" } }
                                }
                            }
                        }
                    }
                }
                div { class: "oxqu-tot",
                    div { class: "oxqu-tc", "data-k": "done",
                        span { class: "oxqu-tl", i { "✓" } "Done" }
                        b { "{sc.done}" }
                    }
                    div { class: "oxqu-tc", "data-k": "failed",
                        span { class: "oxqu-tl", i { "✕" } "Failed" }
                        b { "{sc.failed}" }
                    }
                }
                if let Some((j, pos)) = open_job {
                    {
                        let (ida, idc) = (j.id.clone(), j.id.clone());
                        let waited = queue_secs(j.started_at.unwrap_or(t) - j.enqueued_at);
                        rsx! {
                            div { class: "oxqu-card", role: "dialog", "aria-label": "{j.name}",
                                div { class: "oxqu-cardh",
                                    span { class: "oxqu-wt", {job_icon(&j)} }
                                    span { class: "oxqu-who", b { "{j.name}" } span { "{j.kind.label()} · {j.detail}" } }
                                    span { class: "oxqu-st", if j.status == JobStatus::Running { "Running" } else { "Waiting" } }
                                    button { class: "oxqu-x", r#type: "button", "aria-label": "Close", onclick: move |_| open.set(None), "✕" }
                                }
                                dl { class: "oxqu-cg",
                                    div { dt { "Waiting" } dd { "{waited}" } }
                                    div { dt { "Position" } dd { if let Some((p, n)) = pos { "{p} of {n}" } else { "On a worker" } } }
                                    div { dt { "Priority" } dd { if j.high { "High" } else { "Normal" } } }
                                    div { dt { "Attempt" } dd { "{j.attempt} of {j.max_attempts}" } }
                                }
                                if pos.is_some() {
                                    div { class: "oxqu-acts",
                                        button { class: "oxqu-btn", r#type: "button", onclick: move |_| { push(QueueEvent::Cancel(idc.clone())); open.set(None); on_cancel.call(idc.clone()) }, "Cancel" }
                                        button { class: "oxqu-btn", "data-k": "primary", r#type: "button", onclick: move |_| { push(QueueEvent::Promote(ida.clone())); on_promote.call(ida.clone()) }, "Move to front" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxqu-chart",
                div { class: "oxqu-ch", "Queue depth" }
                div { class: "oxqu-plot",
                    div { class: "oxqu-svgw",
                        span { class: "oxqu-y", style: "top:0", "{top as u32}" }
                        span { class: "oxqu-y", style: "top:{bp_y:.1}%", "{sc.backpressure}" }
                        span { class: "oxqu-y", style: "top:100%", "0" }
                        for (k, (a, b)) in sc.bands.iter().cloned().enumerate() {
                            i { key: "{k}", "data-k": "{k}", class: "oxqu-band", style: "left:{a * 100.0:.2}%;width:{(b - a) * 100.0:.2}%" }
                        }
                        svg { class: "oxqu-svg", view_box: "0 0 1000 100", preserve_aspect_ratio: "none", role: "img", "aria-label": "{chart_label}",
                            defs { linearGradient { id: "oxqu-g", x1: "0", y1: "0", x2: "0", y2: "1",
                                stop { offset: "0", stop_color: "var(--oxqu-violet)", stop_opacity: ".28" }
                                stop { offset: "1", stop_color: "var(--oxqu-violet)", stop_opacity: "0" } } }
                            line { class: "oxqu-bp", x1: "0", x2: "1000", y1: "{bp_y:.1}", y2: "{bp_y:.1}" }
                            path { d: "{area}", fill: "url(#oxqu-g)" }
                            path { class: "oxqu-line", d: "{line}" }
                        }
                        i { class: "oxqu-now", style: "top:{last_y:.1}%" }
                    }
                    span { class: "oxqu-x0", "96s ago" }
                    span { class: "oxqu-x1", "now" }
                }
            }
            div { class: "oxqu-sr", role: "status", "aria-live": "polite",
                if is_paused { "Queue paused" } else if let Some(s) = sc.limited { "Rate limited – workers paused for {s} seconds" } else if over { "{waiting} jobs waiting – above the backpressure line of {sc.backpressure}" }
            }
        }
    }
}
