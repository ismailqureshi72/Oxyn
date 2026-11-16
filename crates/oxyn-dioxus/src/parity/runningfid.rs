//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Running Now**
//! ([`RunningNow`]). Behaviour and look only; the model is `oxyn_core::parity::runningfid`.
//! Styles: `runningfid.css` (classes `oxrn-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::runningfid::{RnState, RnTask, rn_demo, rn_done_at, rn_label},
};

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

fn ring(t: &RnTask) -> Element {
    let total = t.total().max(1) as f32;
    let f = if t.state == RnState::Done {
        1.0
    } else {
        t.done as f32 / total
    };
    let c = 2.0 * std::f32::consts::PI * 6.0;
    let dash = format!("{:.2} {:.2}", c * f, c);
    rsx! {
        svg { class: "oxrn-ring", "data-st": t.state.id(), width: "16", height: "16", view_box: "0 0 16 16", "aria-hidden": "true",
            circle { class: "oxrn-rt", cx: "8", cy: "8", r: "6" }
            circle { class: "oxrn-rf", cx: "8", cy: "8", r: "6", style: "stroke-dasharray:{dash}" }
            if t.state == RnState::Done { path { class: "oxrn-rk", d: "M5.3 8.2l1.8 1.8 3.4-3.8" } }
            if t.state == RnState::NeedsYou { path { class: "oxrn-rk", d: "M8 5.3v3.2M8 10.6h.01" } }
        }
    }
}

/// Running Now (affectUI): background agent tasks as one quiet pill for a header or sidebar –
/// "3 running · 1 needs you". Open it for each task with a filling ring, the step it is on and
/// its step count, Pause / Resume, and Open on the ones waiting for you.
#[component]
pub fn RunningNow(
    /// The tasks (the demo's when `None`).
    #[props(default)]
    tasks: Option<Vec<RnTask>>,
    /// Popover open from the start.
    #[props(default)]
    default_open: bool,
    /// Where the popover opens: `below-end` (header, the default), `below-start`, or `right`
    /// (a sidebar).
    #[props(default = "below-end".to_string())]
    placement: String,
    /// One step per this many ms while running (0 freezes).
    #[props(default = 2600)]
    step_ms: u64,
    /// Popover heading.
    #[props(default = "Background tasks".to_string())]
    title: String,
    /// Show the "Open tasks" footer.
    #[props(default = true)]
    open_all: bool,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A task opened (id).
    #[props(default)]
    on_open: EventHandler<String>,
    /// A task paused (`true`) or resumed (`false`).
    #[props(default)]
    on_pause: EventHandler<(String, bool)>,
    /// "Open tasks" pressed.
    #[props(default)]
    on_open_all: EventHandler<()>,
) -> Element {
    let base = tasks.clone().unwrap_or_else(rn_demo);
    let b2 = base.clone();
    // Per task: (state, steps done at `since`, since).
    let mut st = use_signal(move || {
        b2.iter()
            .map(|t| (t.state, t.done, now))
            .collect::<Vec<_>>()
    });
    let mut open = use_signal(move || default_open);
    let mut live_msg = use_signal(String::new);
    let s = st();
    let cur: Vec<RnTask> = base
        .iter()
        .zip(
            s.iter()
                .chain(std::iter::repeat(&(RnState::Running, 0, now))),
        )
        .map(|(t, (state, done, since))| {
            let mut t = t.clone();
            t.state = *state;
            t.done = *done;
            if *state == RnState::Running && step_ms > 0 {
                t.done = rn_done_at(
                    *done,
                    t.total(),
                    now.since(*since).as_millis() as u64,
                    step_ms,
                );
                if t.done >= t.total() {
                    t.state = RnState::Done;
                }
            }
            t
        })
        .collect();
    let label = rn_label(&cur);
    let running = cur.iter().filter(|t| t.state == RnState::Running).count();
    let needs = cur.iter().filter(|t| t.state == RnState::NeedsYou).count();
    let finished: Vec<String> = cur
        .iter()
        .filter(|t| t.state == RnState::Done)
        .map(|t| format!("{}: done", t.title))
        .collect();
    let announce = if live_msg().is_empty() {
        finished.last().cloned().unwrap_or_default()
    } else {
        live_msg()
    };

    let mut set = move |i: usize, state: RnState, done: u32| {
        let mut v = st();
        if let Some(x) = v.get_mut(i) {
            *x = (state, done, now);
        }
        st.set(v);
    };

    rsx! {
        div { class: "oxrn", "data-style": style_attr(&style), "data-place": "{placement}", "data-open": open(),
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false); },
            button { class: "oxrn-pill", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": open(), "aria-label": "{title}: {label}",
                onclick: move |_| open.set(!open()),
                if running > 0 {
                    span { class: "oxrn-g", i { class: "oxrn-dot", "data-st": "running" } b { "{running}" } " running" }
                }
                if needs > 0 {
                    span { class: "oxrn-g", i { class: "oxrn-dot", "data-st": "needs-you" } b { "{needs}" } if needs == 1 { " needs you" } else { " need you" } }
                }
                if running == 0 && needs == 0 { span { class: "oxrn-g", "{label}" } }
                span { class: "oxrn-chev", {icon("M4.5 6.5 8 10l3.5-3.5", 12)} }
            }
            if open() {
                div { class: "oxrn-pop", role: "dialog", "aria-label": "{title}",
                    div { class: "oxrn-ph", "{title}" span { "{cur.len()}" } }
                    ul { class: "oxrn-list",
                        for (i, t) in cur.iter().enumerate() {
                            {
                                let id = t.id.clone();
                                let id2 = t.id.clone();
                                let id3 = t.id.clone();
                                let title2 = t.title.clone();
                                let state = t.state;
                                let done = t.done;
                                rsx! {
                                    li { class: "oxrn-row", key: "{t.id}", "data-k": "{t.id}", "data-st": t.state.id(),
                                        button { class: "oxrn-main", r#type: "button", "aria-label": "Open {t.title} – {t.line()}",
                                            onclick: move |_| {
                                                on_open.call(id.clone());
                                                if state == RnState::NeedsYou { set(i, RnState::Running, done); live_msg.set(format!("{title2}: running again")); }
                                            },
                                            {ring(t)}
                                            span { class: "oxrn-tx",
                                                b { "{t.title}" }
                                                span { class: "oxrn-step", "{t.line()}" }
                                            }
                                        }
                                        span { class: "oxrn-cnt", "{t.done.min(t.total())}/{t.total()}" }
                                        match t.state {
                                            RnState::Running => rsx! {
                                                button { class: "oxrn-ib", r#type: "button", "aria-label": "Pause {t.title}", title: "Pause",
                                                    onclick: move |_| { set(i, RnState::Paused, done); on_pause.call((id2.clone(), true)); },
                                                    {icon("M6 4v8M10 4v8", 12)} }
                                            },
                                            RnState::Paused => rsx! {
                                                button { class: "oxrn-ib", r#type: "button", "aria-label": "Resume {t.title}", title: "Resume",
                                                    onclick: move |_| { set(i, RnState::Running, done); on_pause.call((id2.clone(), false)); },
                                                    {icon("M5.5 3.8v8.4l6.5-4.2z", 12)} }
                                            },
                                            RnState::NeedsYou => rsx! {
                                                span { class: "oxrn-need", "Needs you" }
                                                {
                                                    let t2 = t.title.clone();
                                                    rsx! {
                                                        button { class: "oxrn-open", r#type: "button",
                                                            onclick: move |_| { on_open.call(id3.clone()); set(i, RnState::Running, done); live_msg.set(format!("{t2}: running again")); },
                                                            "Open" }
                                                    }
                                                }
                                            },
                                            RnState::Done => rsx! { span { class: "oxrn-ib oxrn-ghost" } },
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if open_all {
                        button { class: "oxrn-all", r#type: "button", onclick: move |_| on_open_all.call(()),
                            {icon("M3 4.5h10M3 8h10M3 11.5h6", 13)} span { "Open tasks" } {icon("M6.5 4.5 10 8l-3.5 3.5", 12)} }
                    }
                }
            }
            div { class: "oxrn-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
