//! Fleet overview and Fleet timeline (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::mission::{AGENTS, BUCKETS, FILTERS, Mission, SPAN_MIN, St};
use oxyn_core::blocks::usd;

use super::{avatar, status};
use crate::aikit::{cls, ico, key_of, live, mark};

/// Shared state and the parts both blocks draw (summary curve, filter, inbox).
fn parts(
    m: &Mission,
    n: u64,
    mut act: impl FnMut(String) + Copy + 'static,
) -> (Element, Element, Element) {
    let counts = m.counts(n);
    let curve = m.curve(n);
    let max = curve
        .iter()
        .map(|b| b.iter().sum::<u32>())
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    let w = 100.0 / BUCKETS as f64;
    let summary = rsx! {
        div { class: "oxbk-mc-sum", role: "region", "aria-label": "Since you last looked",
            p { class: "oxaik-sub", "{m.summary(n)}" }
            svg { class: "oxbk-mc-curve", view_box: "0 0 100 30", preserve_aspect_ratio: "none", role: "img", "aria-label": "{m.summary(n)}",
                for (i, b) in curve.iter().enumerate() {
                    {
                        let mut y = 30.0;
                        let segs: Vec<(f64, f64, &str)> = [(b[0], "green"), (b[1], "amber"), (b[2], "red")].iter().filter(|x| x.0 > 0).map(|&(v, h)| {
                            let hgt = v as f64 / max * 26.0;
                            y -= hgt;
                            (y, hgt, h)
                        }).collect();
                        rsx! {
                            g { key: "{i}",
                                rect { class: "oxbk-mc-base", x: "{i as f64 * w + 0.6}", y: "29", width: "{w - 1.2}", height: "1" }
                                for (k, (yy, hh, h)) in segs.into_iter().enumerate() {
                                    rect { key: "{k}", "data-hue": "{h}", x: "{i as f64 * w + 0.6}", y: "{yy}", width: "{w - 1.2}", height: "{hh}", rx: "0.6" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-row oxaik-between oxaik-mute oxbk-mc-axis", span { "8 h ago" } span { "4 h" } span { "now" } }
        }
    };
    let filter = rsx! {
        div { class: "oxbk-filter", role: "radiogroup", "aria-label": "Show agents",
            for (i, f) in FILTERS.iter().enumerate() {
                button { key: "{i}", class: cls("oxaik-sg", m.filter == i), r#type: "button", role: "radio", "aria-checked": "{m.filter == i}",
                    tabindex: if m.filter == i { "0" } else { "-1" },
                    "aria-label": "{f}, {counts[i]}",
                    onclick: move |_| act(format!("mi-f-{i}")),
                    "{f}" span { class: "oxbk-cnt oxaik-num", "{counts[i]}" }
                }
            }
        }
    };
    let inbox_list = m.inbox(n);
    let inbox = rsx! {
        aside { class: "oxbk-mc-inbox", "aria-label": "Needs you",
            p { class: "oxbk-cap", "Needs you " span { class: "oxbk-cnt oxaik-num", "{inbox_list.len()}" } }
            if inbox_list.is_empty() {
                p { class: "oxaik-mute oxbk-empty", {ico("check-circle", 14.0)} "Nothing waiting on you" }
            }
            for &i in inbox_list.iter() {
                {
                    let a = &AGENTS[i];
                    let st = m.st(i, n);
                    let open = m.open == Some(i);
                    rsx! {
                        div { key: "{i}", class: "oxbk-mc-card", "data-st": "{st.word()}",
                            button { class: "oxbk-mc-cardb", r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-mc-ans-{i}",
                                onclick: move |_| act(format!("mi-open-{i}")),
                                {avatar(a.brand, a.name)}
                                span { class: "oxbk-mc-ct", b { "{a.name}" } span { class: "oxaik-mute", "{a.task}" } }
                                {status(st.word(), st.hue())}
                            }
                            if open {
                                div { class: "oxbk-mc-ans oxaik-fade", id: "oxbk-mc-ans-{i}",
                                    p { "{m.note(i, n)}" }
                                    div { class: "oxaik-row",
                                        if st == St::Waiting {
                                            button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("mi-ok-{i}")), "Approve" }
                                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mi-no-{i}")), "Decline" }
                                        } else {
                                            button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("mi-retry-{i}")), {ico("retry", 12.0)} "Retry" }
                                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mi-dismiss-{i}")), "Dismiss" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    (summary, filter, inbox)
}

fn agent_actions(
    m: &Mission,
    i: usize,
    n: u64,
    mut act: impl FnMut(String) + Copy + 'static,
) -> Element {
    match m.st(i, n) {
        St::Waiting => rsx! {
            div { class: "oxaik-row oxbk-mc-acts",
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mi-ok-{i}")), "Approve" }
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mi-no-{i}")), "Decline" }
            }
        },
        St::Failed => rsx! {
            div { class: "oxaik-row oxbk-mc-acts",
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mi-retry-{i}")), {ico("retry", 12.0)} "Retry" }
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mi-dismiss-{i}")), "Dismiss" }
            }
        },
        _ => rsx! {},
    }
}

fn key_handler(
    mut ms: Signal<Mission>,
    mut said: Signal<String>,
    n: u64,
) -> impl FnMut(KeyboardEvent) {
    move |e: KeyboardEvent| {
        let k = key_of(&e);
        if matches!(
            k.as_str(),
            "ArrowLeft" | "ArrowRight" | "Home" | "End" | "Escape"
        ) {
            e.prevent_default();
            let x = ms.write().key(&k, n);
            if let Some(x) = x {
                said.set(x);
            }
        }
    }
}

/// Fleet overview: every agent at a glance. On top, what happened since you looked as a curve
/// of done, needs-you and failed events; in the middle the fleet (each lead with its sub-agents,
/// the picked lead's sub-agents beside it); on the side an inbox of asks and failures with their
/// answers. One filter (All, Running, Waiting, Failed; arrow keys) narrows all three. Approve,
/// Decline, Retry and Dismiss change the agent in place.
#[component]
pub fn FleetOverview(
    /// Host clock.
    now: ClockMs,
    /// Start filter (0 all, 1 running, 2 waiting, 3 failed).
    #[props(default)]
    filter: usize,
    /// A retried agent fails again (error scenario).
    #[props(default)]
    fail_retry: bool,
    /// Heading.
    #[props(default = "Fleet overview".to_string())]
    title: String,
) -> Element {
    let mut ms = use_signal(move || Mission::new(filter, fail_retry, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let m = ms.read().clone();
    let mut act = move |id: String| {
        let x = ms.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (summary, filt, inbox) = parts(&m, n, act);
    let leads: Vec<usize> = m.leads().into_iter().filter(|&l| m.shows(l, n)).collect();
    let lead = if leads.contains(&m.lead) {
        Some(m.lead)
    } else {
        leads.first().copied()
    };
    rsx! {
        section { class: "oxaik oxbk oxbk-mc", "aria-label": "{title}", onkeydown: key_handler(ms, said, n),
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "{title}" } p { class: "oxaik-sub", "{AGENTS.len()} agents · {usd(m.cost())} today" } }
                {filt}
            }
            {summary}
            div { class: "oxbk-mc-main",
                div { class: "oxbk-mc-fleet", role: "list", "aria-label": "Fleet",
                    div { class: "oxbk-mc-leads",
                        for &l in leads.iter() {
                            {
                                let a = &AGENTS[l];
                                let st = m.st(l, n);
                                let kids = m.children(l);
                                let on = lead == Some(l);
                                let kids_word = format!("{} sub-agent{}", kids.len(), if kids.len() == 1 { "" } else { "s" });
                                rsx! {
                                    div { key: "{l}", class: cls("oxbk-mc-lead", on), role: "listitem",
                                        button { class: "oxbk-mc-leadb", r#type: "button", "aria-pressed": "{on}", onclick: move |_| act(format!("mi-lead-{l}")),
                                            div { class: "oxaik-row oxaik-between",
                                                span { class: "oxaik-row", {avatar(a.brand, a.name)} b { "{a.name}" } }
                                                {status(st.word(), st.hue())}
                                            }
                                            p { class: "oxbk-mc-task", "{a.task}" }
                                            p { class: "oxaik-row oxaik-mute oxaik-num",
                                                {mark(a.brand, 11.0)} span { "{a.model}" } span { "· {m.time(l, n)}" }
                                            }
                                            div { class: "oxbk-mc-kids", "aria-label": "{kids.len()} sub-agents",
                                                for &c in kids.iter() { i { key: "{c}", "data-hue": "{m.st(c, n).hue()}", title: "{AGENTS[c].name}: {m.st(c, n).word()}" } }
                                                span { class: "oxaik-mute", "{kids_word}" }
                                            }
                                        }
                                        {agent_actions(&m, l, n, act)}
                                    }
                                }
                            }
                        }
                    }
                    if let Some(l) = lead {
                        div { class: "oxbk-mc-subs", "aria-label": "Sub-agents of {AGENTS[l].name}",
                            p { class: "oxbk-cap", "Under {AGENTS[l].name}" }
                            for c in m.children(l).into_iter().filter(|&c| m.passes(c, n) || m.filter == 0) {
                                {
                                    let a = &AGENTS[c];
                                    let st = m.st(c, n);
                                    let note = m.note(c, n);
                                    rsx! {
                                        div { key: "{c}", class: "oxbk-mc-sub", "data-st": "{st.word()}",
                                            div { class: "oxaik-row oxaik-between", span { class: "oxaik-row", {avatar(a.brand, a.name)} b { "{a.name}" } } {status(st.word(), st.hue())} }
                                            p { class: "oxbk-mc-task", "{a.task}" }
                                            if !note.is_empty() { p { class: "oxbk-mc-note", "data-hue": "{st.hue()}", "{note}" } }
                                            p { class: "oxaik-mute oxaik-num", "{m.time(c, n)}" }
                                            {agent_actions(&m, c, n, act)}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                {inbox}
            }
            {live(&said.read())}
        }
    }
}

/// Fleet timeline: Fleet overview on one clock. Every agent gets a lane (leads with their
/// sub-agents indented) on a shared 30-minute axis ending now; running blocks grow, finished ones
/// stop, waiting and failed ones carry their colour and word. The curve, the filter and the inbox
/// work as in Fleet overview.
#[component]
pub fn FleetTimeline(
    /// Host clock.
    now: ClockMs,
    /// Start filter (0 all, 1 running, 2 waiting, 3 failed).
    #[props(default)]
    filter: usize,
    /// A retried agent fails again (error scenario).
    #[props(default)]
    fail_retry: bool,
    /// Show the now line and axis ticks.
    #[props(default = true)]
    ticks: bool,
) -> Element {
    let mut ms = use_signal(move || Mission::new(filter, fail_retry, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let m = ms.read().clone();
    let mut act = move |id: String| {
        let x = ms.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (summary, filt, inbox) = parts(&m, n, act);
    let mut rows: Vec<usize> = Vec::new();
    for l in m.leads() {
        if m.shows(l, n) {
            rows.push(l);
            rows.extend(
                m.children(l)
                    .into_iter()
                    .filter(|&c| m.passes(c, n) || m.filter == 0),
            );
        }
    }
    rsx! {
        section { class: "oxaik oxbk oxbk-mc oxbk-mt", "aria-label": "Fleet timeline", onkeydown: key_handler(ms, said, n),
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Fleet timeline" } p { class: "oxaik-sub", "{AGENTS.len()} agents on one clock · last {SPAN_MIN} min" } }
                {filt}
            }
            {summary}
            div { class: "oxbk-mc-main",
                div { class: "oxbk-mt-lanes", role: "list", "aria-label": "Lanes",
                    if ticks {
                        div { class: "oxbk-mt-axis oxaik-mute oxaik-num", "aria-hidden": "true",
                            span { "{SPAN_MIN} min" } span { "20" } span { "10" } span { "now" }
                        }
                    }
                    for &i in rows.iter() {
                        {
                            let a = &AGENTS[i];
                            let st = m.st(i, n);
                            let sub = a.parent.is_some();
                            let lane = m.lane(i, n);
                            rsx! {
                                div { key: "{i}", class: "oxbk-mt-row", role: "listitem", "data-sub": "{sub}",
                                    "aria-label": "{a.name}, {st.word()}, {a.task}, {m.time(i, n)}",
                                    div { class: "oxbk-mt-name", {avatar(a.brand, a.name)} span { "{a.name}" } }
                                    div { class: "oxbk-mt-track",
                                        if ticks { i { class: "oxbk-mt-grid", style: "left:33.333%" } i { class: "oxbk-mt-grid", style: "left:66.666%" } }
                                        if let Some((s, e)) = lane {
                                            button { class: "oxbk-mt-blk", r#type: "button", "data-hue": "{st.hue()}", "data-st": "{st.word()}",
                                                style: "left:{s * 100.0}%;width:max(6px, {(e - s) * 100.0}%)",
                                                title: "{a.task}",
                                                onclick: move |_| if matches!(st, St::Waiting | St::Failed) { act(format!("mi-open-{i}")) },
                                                span { "{a.task}" }
                                            }
                                        }
                                    }
                                    span { class: "oxbk-mt-st", {status(st.word(), st.hue())} }
                                }
                            }
                        }
                    }
                }
                {inbox}
            }
            {live(&said.read())}
        }
    }
}
