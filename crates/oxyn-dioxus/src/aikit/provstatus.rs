//! Provider status (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::provstatus::{
    DAYS, PROVIDERS, ProvStatus, day, day_label, latency, story, uptime,
};

use super::{dots, ico, key_of, live, mark};

/// Provider status: a card for the AI providers a product depends on. Each row shows the
/// provider's mark, its health as an icon and a word (never colour alone), p50 latency in
/// tabular numerals and a 30-day uptime bar whose days read out on hover or with the arrow keys.
/// A row with an incident is a button that opens its updates, newest first. The header sums up
/// health and checks again on a draining countdown ring, with a refresh button; a failed check
/// keeps the last results and says so. Narrow cards switch to two lines per row.
#[component]
pub fn ProviderStatus(
    /// Host clock.
    now: ClockMs,
    /// Seconds between checks.
    #[props(default = 12)]
    interval_s: u64,
    /// Story check to start from (0 outage, 9 a failed check, 10 all clear).
    #[props(default)]
    start_check: usize,
    /// Show the uptime bars.
    #[props(default = true)]
    uptime_bars: bool,
    /// Provider whose incident is open on mount.
    #[props(default)]
    open: Option<usize>,
) -> Element {
    let t0 = now.0;
    let mut ps = use_signal(move || ProvStatus::new(interval_s * 1000, start_check, open, t0));
    let mut said = use_signal(String::new);
    let mut hover = use_signal(|| None::<(usize, usize)>);
    let n = now.0;
    let s = ps.read().clone();
    let mut act = move |id: String| {
        let x = ps.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (chk, failed) = s.check(n);
    let checking = s.checking(n);
    let worst = s.worst(n);
    let ring = s.ring(n);
    let dash = 2.0 * std::f64::consts::PI * 7.0;
    let readout = hover().or(s.cursor);
    rsx! {
        section { class: "oxaik oxaik-ps", "aria-label": "AI providers",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", "AI providers" }
                    p { class: "oxaik-sub", "data-hue": "{worst.hue()}",
                        span { class: "oxaik-ps-sum", {ico(worst.icon(), 13.0)} span { key: "{s.summary(n)}", class: "oxaik-fade", "{s.summary(n)}" } }
                    }
                }
                div { class: "oxaik-row oxaik-ps-chk",
                    if checking {
                        span { class: "oxaik-row oxaik-mute", {dots()} "Checking" }
                    } else if failed {
                        span { class: "oxaik-row oxaik-ps-err", role: "alert", {ico("alert", 13.0)} "Check failed · last results" }
                    } else {
                        span { class: "oxaik-mute oxaik-num", "Next check in {s.countdown(n)}s" }
                    }
                    svg { class: "oxaik-ps-ring", width: "18", height: "18", view_box: "0 0 18 18", "aria-hidden": "true",
                        circle { cx: "9", cy: "9", r: "7", class: "bg" }
                        circle { cx: "9", cy: "9", r: "7", class: "fg", "stroke-dasharray": "{dash:.2}", "stroke-dashoffset": "{dash * (1.0 - ring):.2}" }
                    }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Check now", disabled: checking, onclick: move |_| act("ps-refresh".into()), {ico("retry", 13.0)} }
                }
            }
            ul { class: "oxaik-ps-list",
                for (p, (name, mk, detail, _)) in PROVIDERS.iter().enumerate() {
                    {
                        let (h, inc) = story(p, chk);
                        let lat = latency(p, chk);
                        let up = uptime(p, chk);
                        let is_open = s.open == Some(p) && inc.is_some();
                        let head = rsx! {
                            span { class: "oxaik-ps-name", {mark(mk, 16.0)} span { class: "oxaik-ps-nt", b { "{name}" } span { class: "oxaik-mute", "{detail}" } } }
                            span { class: "oxaik-ps-h", "data-hue": "{h.hue()}", {ico(h.icon(), 13.0)} span { key: "{h.word()}", class: "oxaik-fade", "{h.word()}" } }
                            span { class: "oxaik-ps-lat oxaik-num", "{lat} ms" }
                        };
                        rsx! {
                            li { key: "{p}", class: "oxaik-ps-row", "data-open": "{is_open}",
                                if let Some(inc) = &inc {
                                    button { class: "oxaik-ps-main", r#type: "button", "aria-expanded": "{is_open}", "aria-controls": "oxaik-ps-inc-{p}",
                                        "aria-label": "{name}, {h.word()}: {inc.title}",
                                        onclick: move |_| act(format!("ps-row-{p}")),
                                        {head}
                                        span { class: if is_open { "oxaik-ay-chev open" } else { "oxaik-ay-chev" }, {ico("chevron", 12.0)} }
                                    }
                                } else {
                                    div { class: "oxaik-ps-main", {head} span { class: "oxaik-ay-chev", "aria-hidden": "true" } }
                                }
                                if uptime_bars {
                                    div { class: "oxaik-ps-up",
                                        div { class: "oxaik-ps-bar", tabindex: "0", role: "img",
                                            "aria-label": "{name}, 30 days, {up:.2}% uptime",
                                            onkeydown: move |e: KeyboardEvent| {
                                                let k = key_of(&e);
                                                if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End" | "Escape") {
                                                    e.prevent_default();
                                                    let x = ps.write().key(p, &k, n);
                                                    if let Some(x) = x { said.set(x) }
                                                }
                                            },
                                            onmouseleave: move |_| hover.set(None),
                                            for d in 0..DAYS {
                                                i { key: "{d}", "data-hue": "{day(p, d, chk).hue()}", class: if readout == Some((p, d)) { "on" } else { "" },
                                                    onmouseenter: move |_| hover.set(Some((p, d))) }
                                            }
                                            if let Some((rp, rd)) = readout.filter(|r| r.0 == p) {
                                                span { class: "oxaik-ps-tip", style: "left:{(rd as f64 + 0.5) / DAYS as f64 * 100.0}%",
                                                    b { "{day_label(rd)}" } " · {day(rp, rd, chk).word()}" }
                                            }
                                        }
                                        span { class: "oxaik-ps-pct oxaik-num", "{up:.2}%" }
                                    }
                                }
                                if is_open {
                                    if let Some(inc) = &inc {
                                        div { id: "oxaik-ps-inc-{p}", class: "oxaik-ps-inc",
                                            b { "{inc.title}" }
                                            ol { class: "oxaik-ps-ups",
                                                for (st, t, txt) in inc.updates.iter() {
                                                    li { key: "{st}", "data-st": "{st}",
                                                        span { class: "oxaik-ps-stg", "{st}" }
                                                        span { class: "oxaik-mute oxaik-num", "{t}" }
                                                        span { "{txt}" }
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
            p { class: "oxaik-mute oxaik-ps-foot", "p50 latency over the last check · uptime from daily health · sample data" }
            {live(&said())}
        }
    }
}
