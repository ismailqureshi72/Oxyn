//! Catch up (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::catchup::{CatchUp as Model, ITEMS, RANGES, SECTIONS, SOURCES};
use oxyn_core::blocks::thousands;

use crate::aikit::{cls, dots, ico, key_of, live, mark};

/// Catch up: a digest for when you return. One button reads Slack, Linear, GitHub and Gmail for
/// the range you pick (each source shows a spinner, then a check and how much it read) and
/// streams back a ranked summary in four sections: Needs you, Decisions made, Shipped and FYI.
/// Each line names its source, people and a link; marking lines as read clears them until you
/// are all caught up.
#[component]
pub fn CatchUp(
    /// Host clock.
    now: ClockMs,
    /// Range (0 since yesterday, 1 last 3 days, 2 last week).
    #[props(default = 1)]
    range: usize,
    /// Read at once.
    #[props(default = true)]
    auto_play: bool,
    /// GitHub can't be reached on the first read (error scenario).
    #[props(default)]
    fail: bool,
) -> Element {
    let mut cu = use_signal(move || Model::new(range, auto_play, fail, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let c = cu.read().clone();
    let mut act = move |id: String| {
        let x = cu.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let busy = c.busy(n);
    let items = c.items();
    let streamed = c.streamed(n);
    let left = c.left(n);
    let reading = c.run.is_some() && streamed == 0 && busy;
    let run_label = if busy {
        "Stop"
    } else if c.run.is_some() {
        "Refresh"
    } else {
        "Catch me up"
    };
    rsx! {
        section { class: "oxaik oxbk oxbk-cu", "aria-label": "Catch up",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h2 { class: "oxaik-h", "Catch up" }
                    p { class: "oxaik-sub", if c.run.is_some() && !busy { "{left.len()} of {items.len()} left · {RANGES[c.range].to_lowercase()}" } else { "What you missed, ranked" } }
                }
                div { class: "oxbk-filter", role: "radiogroup", "aria-label": "Range",
                    onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") { e.prevent_default(); let x = cu.write().key(&k, n); if let Some(x) = x { said.set(x) } } },
                    for (i, r) in RANGES.iter().enumerate() {
                        button { key: "{i}", class: cls("oxaik-sg", c.range == i), r#type: "button", role: "radio", "aria-checked": "{c.range == i}", tabindex: if c.range == i { "0" } else { "-1" }, disabled: busy,
                            onclick: move |_| act(format!("cu-range-{i}")), "{r}" }
                    }
                }
            }
            div { class: "oxbk-cu-srcs",
                for (i, s) in SOURCES.iter().enumerate() {
                    {
                        let st = c.source(i, n);
                        rsx! {
                            span { key: "{i}", class: "oxbk-cu-src", "data-st": "{st}",
                                {mark(s.1, 13.0)} span { "{s.0}" }
                                match st {
                                    1 => rsx! { {dots()} },
                                    2 => rsx! { span { class: "oxbk-st", "data-hue": "green", {ico("check", 11.0)} span { class: "oxaik-num", "{thousands(u64::from(s.2[c.range]))}" } } },
                                    3 => rsx! { span { class: "oxbk-st", "data-hue": "red", {ico("x-circle", 11.0)} span { "Skipped" } } },
                                    _ => rsx! {},
                                }
                            }
                        }
                    }
                }
            }
            if c.skipped && !reading { p { class: "oxbk-mcp-err", {ico("alert", 12.0)} "GitHub could not be reached, so pull requests are missing." button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("cu-run".into()), {ico("retry", 12.0)} "Retry" } } }
            if c.run.is_none() {
                p { class: "oxbk-empty", {ico("inbox", 13.0)} "Pick a range and catch up in one read." }
            } else if c.caught_up(n) {
                p { class: "oxbk-cu-done oxaik-fade", {ico("check-circle", 16.0)} b { "All caught up" } span { class: "oxaik-mute", "Nothing else since {RANGES[c.range].to_lowercase()}." } }
            } else if reading {
                p { class: "oxaik-mute oxaik-row", {dots()} span { class: "oxaik-shim", "Reading your tools" } }
            } else {
                for (si, sec) in SECTIONS.iter().enumerate() {
                    {
                        let rows: Vec<usize> = items.iter().take(streamed).copied().filter(|&i| ITEMS[i].0 == si && !c.read.contains(&i)).collect();
                        rsx! {
                            if !rows.is_empty() {
                                section { key: "{si}", class: "oxbk-cu-sec", "aria-labelledby": "oxbk-cu-s{si}",
                                    h3 { id: "oxbk-cu-s{si}", class: "oxbk-cu-sh", "data-hue": "{sec.2}", {ico(sec.1, 12.0)} "{sec.0}" span { class: "oxbk-cnt", "{rows.len()}" } }
                                    ul { class: "oxbk-cu-ul",
                                        for i in rows {
                                            {
                                                let it = ITEMS[i];
                                                let src = SOURCES[it.1];
                                                rsx! {
                                                    li { key: "{i}", class: "oxbk-cu-it oxaik-fade",
                                                        span { class: "oxbk-cu-mk", {mark(src.1, 14.0)} }
                                                        div { class: "oxbk-cu-body",
                                                            p { span { class: "oxaik-sr", "{src.0}: " } "{it.2}" }
                                                            p { class: "oxaik-mute oxbk-cu-meta", "{it.3} · {it.4}" }
                                                            if c.open == Some(i) { p { class: "oxbk-cu-open oxaik-fade", {ico("external", 11.0)} "Opens {it.4} in {src.0} (simulated)" } }
                                                        }
                                                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Open {it.4} in {src.0}", "aria-expanded": "{c.open == Some(i)}", onclick: move |_| act(format!("cu-open-{i}")), {ico("external", 12.0)} }
                                                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Mark as read: {it.2}", onclick: move |_| act(format!("cu-read-{i}")), {ico("check", 12.0)} }
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
            }
            footer { class: "oxaik-row oxbk-cu-foot",
                button { class: if c.run.is_none() { "oxaik-btn pri" } else { "oxaik-btn" }, r#type: "button", onclick: move |_| act("cu-run".into()),
                    {ico(if busy { "stop" } else if c.run.is_some() { "retry" } else { "spark" }, 12.0)} "{run_label}" }
                if !busy && !left.is_empty() { button { class: "oxaik-btn", r#type: "button", onclick: move |_| act("cu-all".into()), {ico("check", 12.0)} "Mark all as read" } }
            }
            {live(&said.read())}
        }
    }
}
