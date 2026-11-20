//! TL;DR toggle (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::first_words;
use oxyn_core::aikit::tldr::{BYLINE, LENGTHS, MODES, PARAS, SUMMARIES, TITLE, Tldr};

use super::{dots, ico, live, mark, seg};

/// TL;DR toggle: an article card with a Full, Summary and Key points switch and a length
/// control. Switching streams an AI summary in place of the article (the body follows its
/// height); each key point jumps back to the paragraph it came from, which is highlighted for a
/// moment. A failed summary offers Try again. Keys: arrows, Home and End in both groups.
#[component]
pub fn TldrToggle(
    /// Host clock.
    now: ClockMs,
    /// Starting view (0 full, 1 summary, 2 key points).
    #[props(default = 1)]
    mode: usize,
    /// Starting length (0 short, 1 medium, 2 long).
    #[props(default = 1)]
    length: usize,
    /// The first summary fails.
    #[props(default)]
    fail: bool,
    /// Show the words-saved readout (Oxyn extra).
    #[props(default = true)]
    saved: bool,
) -> Element {
    let t0 = now.0;
    let mut td = use_signal(move || Tldr::new(mode, length, fail, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let t = td.read().clone();
    let mut act = move |id: String| {
        let x = td.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = t.phase(n);
    let vis = t.visible(n);
    let hl = t.highlighted(n);
    rsx! {
        article { class: "oxaik oxaik-td", "aria-label": "{TITLE}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "{TITLE}" } span { class: "oxaik-mute", "{BYLINE}" } }
            }
            div { class: "oxaik-row oxaik-between oxaik-td-ctl",
                {seg("View", MODES.iter().map(|x| x.to_string()).collect(), t.mode, EventHandler::new(move |i| act(format!("td-mode-{i}"))), EventHandler::new(move |k: String| { let x = td.write().key("mode", &k, n); if let Some(x) = x { said.set(x) } }))}
                if t.mode != 0 {
                    {seg("Length", LENGTHS.iter().map(|x| x.to_string()).collect(), t.len, EventHandler::new(move |i| act(format!("td-len-{i}"))), EventHandler::new(move |k: String| { let x = td.write().key("len", &k, n); if let Some(x) = x { said.set(x) } }))}
                }
            }
            div { class: "oxaik-td-body", "aria-busy": "{ph == \"thinking\" || ph == \"streaming\"}",
                {match ph {
                    "full" => rsx! {
                        for (i, p) in PARAS.iter().enumerate() {
                            p { key: "{i}", class: if hl == Some(i) { "oxaik-td-p hl" } else { "oxaik-td-p" }, "{p}" }
                        }
                    },
                    "thinking" => rsx! { p { class: "oxaik-row", {dots()} span { class: "oxaik-shim", if t.mode == 1 { "Reading the article" } else { "Finding the key points" } } } },
                    "error" => rsx! {
                        div { class: "oxaik-err", role: "alert", {ico("alert", 18.0)} b { "Couldn't summarize this article" }
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("td-retry".into()), {ico("retry", 12.0)} "Try again" }
                        }
                    },
                    _ if t.mode == 1 => rsx! { p { class: "oxaik-td-sum", "{first_words(SUMMARIES[t.len], vis)}" } },
                    _ => {
                        let mut left = vis;
                        let pts: Vec<(usize, &str, usize, usize, usize)> = t.points().iter().enumerate().map(|(i, (pt, para))| {
                            let w = pt.split_whitespace().count();
                            let k = left.min(w);
                            left = left.saturating_sub(w);
                            (i, *pt, *para, k, w)
                        }).filter(|x| x.3 > 0).collect();
                        rsx! {
                            ol { class: "oxaik-td-pts",
                                for (i, pt, para, k, w) in pts {
                                    li { key: "{i}",
                                        span { "{first_words(pt, k)}" }
                                        if k == w {
                                            button { class: "oxaik-ib oxaik-td-jump", r#type: "button", "aria-label": "Jump to paragraph {para + 1}", onclick: move |_| act(format!("td-jump-{i}")), {ico("link", 12.0)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }}
            }
            div { class: "oxaik-row oxaik-between oxaik-td-foot",
                span { class: "oxaik-row oxaik-mute",
                    if t.mode == 0 { "Full article" } else if ph == "done" { {mark("claude", 12.0)} "Summarized by Claude Haiku 4.5" } else if ph == "error" { "Summary failed" } else { "Summarizing…" }
                }
                if saved && t.mode != 0 && ph == "done" { span { class: "oxaik-mute oxaik-num", "{t.saved()}" } }
            }
            {live(&said())}
        }
    }
}
