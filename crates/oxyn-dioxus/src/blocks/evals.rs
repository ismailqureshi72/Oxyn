//! Eval suite (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::evals::{
    BASELINE, CANDIDATE, CATEGORIES, Evals, FEATURE, FILTERS, HISTORY, JUDGE, Out, RUBRIC,
};

use super::status;
use crate::aikit::{cls, ico, key_of, live, mark};

fn glyph(o: Option<Out>) -> &'static str {
    match o {
        Some(Out::Pass) => "✓",
        Some(Out::Fail) => "×",
        Some(Out::Error) => "!",
        None => "",
    }
}

/// Eval suite: score a release candidate of a model-backed feature against the shipped
/// baseline. The header names the feature, both versions with their model marks and the judge,
/// with Run (Stop while running). Results stream in: each category row fills a cell per case
/// (pass, fail and error with their own glyphs, a regression solid), its bar shows a ghost tick at
/// the baseline, and the score counts up beside the baseline with the change in points. The case
/// list filters to All, Failed, Regressions or Improved (arrow keys); opening a case shows the
/// input, expected behaviour, both outputs and the judge's scores, with buttons to accept it or file
/// it as an issue (and Undo).
#[component]
pub fn EvalSuite(
    /// Host clock.
    now: ClockMs,
    /// Open with the run finished.
    #[props(default)]
    done: bool,
    /// The judge fails halfway (error scenario).
    #[props(default)]
    fail: bool,
    /// Start filter (0 all, 1 failed, 2 regressions, 3 improved).
    #[props(default)]
    filter: usize,
    /// Open this case at the start.
    #[props(default)]
    open: Option<usize>,
) -> Element {
    let mut ev = use_signal(move || {
        let mut e = Evals::new(done, fail, now.0);
        e.filter = filter.min(3);
        e.open = open;
        e
    });
    let mut said = use_signal(String::new);
    let mut more = use_signal(|| false);
    let n = now.0;
    let e = ev.read().clone();
    let mut act = move |id: String| {
        let x = ev.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let state = e.state(n);
    let (score, base) = e.score(n);
    let delta = score as i64 - base as i64;
    let (cost, time) = e.cost_time(n);
    let counts = e.counts(n);
    let list = e.list(n);
    let shown: Vec<usize> = if *more.read() {
        list.clone()
    } else {
        list.iter().copied().take(8).collect()
    };
    let hmax = HISTORY.iter().map(|h| h.1).max().unwrap_or(100).max(score) as f64;
    let st_hue = match state {
        "Finished" => "green",
        "Running" => "blue",
        "Judge failed" => "red",
        _ => "neutral",
    };
    rsx! {
        section { class: "oxaik oxbk oxbk-ev", "aria-label": "Eval suite: {FEATURE}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", "{FEATURE}" }
                    p { class: "oxaik-sub oxbk-ev-models",
                        span { class: "oxaik-pill", {mark(CANDIDATE.1, 12.0)} "{CANDIDATE.0} · {CANDIDATE.2}" }
                        span { class: "oxaik-mute", "vs" }
                        span { class: "oxaik-pill", {mark(BASELINE.1, 12.0)} "{BASELINE.0} · {BASELINE.2}" }
                        span { class: "oxaik-mute oxaik-row", "Judge" {mark(JUDGE.0, 12.0)} "{JUDGE.1}" }
                    }
                }
                button { class: if state == "Running" { "oxaik-btn sm" } else { "oxaik-btn sm pri" }, r#type: "button", onclick: move |_| act("ev-run".into()),
                    {ico(if state == "Running" { "stop" } else { "play" }, 11.0)}
                    if state == "Running" { "Stop" } else if e.run.is_some() { "Run again" } else { "Run {e.cases.len()} cases" }
                }
            }
            div { class: "oxbk-ev-sum",
                div { class: "oxbk-ev-score",
                    span { class: "oxbk-ev-big oxaik-num", "{score}%" span { class: "oxaik-sr", " pass rate" } }
                    span { class: "oxaik-mute oxaik-num", "baseline {base}%" }
                    if e.run.is_some() { span { class: "oxaik-pill oxaik-num", "data-hue": if delta >= 0 { "green" } else { "red" }, "{delta:+} pts" } }
                    {status(state, st_hue)}
                }
                dl { class: "oxbk-ev-ct", div { dt { "Cost" } dd { class: "oxaik-num", "{cost}" } } div { dt { "Time" } dd { class: "oxaik-num", "{time}" } } }
                div { class: "oxbk-ev-hist", role: "group", "aria-label": "Earlier runs",
                    for (k, (v, s)) in HISTORY.iter().enumerate() {
                        {
                            let val = if *s == 0 { score } else { *s };
                            rsx! {
                                button { key: "{k}", class: cls("oxbk-ev-hb", e.hist == k), r#type: "button", "aria-label": "{v}: {val}%", "aria-pressed": "{e.hist == k}",
                                    onclick: move |_| act(format!("ev-hist-{k}")),
                                    i { style: "height:{val as f64 / hmax * 100.0}%" } small { "{v}" }
                                }
                            }
                        }
                    }
                }
            }
            if state == "Judge failed" {
                p { class: "oxbk-mcp-err", {ico("alert", 12.0)} "The judge stopped responding at case {e.scored(n)}. Scores so far are kept; run again to finish." }
            }
            div { class: "oxbk-ev-cats",
                for (c, (name, bpct)) in CATEGORIES.iter().enumerate() {
                    {
                        let (_, _, _, _, pct) = e.cat(c, n);
                        let d = pct as i64 - *bpct as i64;
                        let cells: Vec<(usize, Option<Out>, Option<bool>, &str)> = (0..e.cases.len()).filter(|&i| e.cases[i].cat == c).map(|i| (i, e.out(i, n), e.change(i, n), e.out(i, n).map_or("", |x| x.word()))).collect();
                        let scored = cells.iter().any(|x| x.1.is_some());
                        rsx! {
                            div { key: "{c}", class: "oxbk-ev-cat",
                                span { class: "oxbk-ev-cn", "{name}" }
                                span { class: "oxbk-ev-cells", role: "img", "aria-label": "{e.cat_label(c, n)}",
                                    for (i, o, ch, ow) in cells.into_iter() {
                                        i { key: "{i}", "data-o": "{ow}", "data-reg": "{ch == Some(false)}", "{glyph(o)}" }
                                    }
                                }
                                span { class: "oxbk-ev-bar", "aria-hidden": "true", span { class: "oxaik-bar", span { style: "width:{pct}%" } } i { style: "left:{bpct}%" } }
                                span { class: "oxaik-num oxbk-ev-pct", if scored { "{pct}%" } else { "–" } }
                                span { class: "oxaik-num oxbk-ev-d", "data-hue": if d >= 0 { "green" } else { "red" }, if scored { "{d:+}" } else { "" } }
                            }
                        }
                    }
                }
            }
            div { class: "oxbk-filter oxaik-scroll", role: "radiogroup", "aria-label": "Cases",
                onkeydown: move |k: KeyboardEvent| {
                    let key = key_of(&k);
                    if matches!(key.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                        k.prevent_default();
                        let x = ev.write().key(&key, n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                for (i, f) in FILTERS.iter().enumerate() {
                    button { key: "{i}", class: cls("oxaik-sg", e.filter == i), r#type: "button", role: "radio", "aria-checked": "{e.filter == i}", tabindex: if e.filter == i { "0" } else { "-1" },
                        onclick: move |_| act(format!("ev-f-{i}")),
                        "{f}" span { class: "oxbk-cnt oxaik-num", "{counts[i]}" }
                    }
                }
            }
            ul { class: "oxbk-ev-list",
                if list.is_empty() { li { class: "oxaik-mute oxbk-empty", if e.run.is_none() { "Run the suite to see cases." } else { "No cases here." } } }
                for &i in shown.iter() {
                    {
                        let c = &e.cases[i];
                        let o = e.out(i, n);
                        let ch = e.change(i, n);
                        let open = e.open == Some(i);
                        let tri = e.triage[i];
                        let ow = o.map_or("", |x| x.word());
                        let ows = o.map_or("pending", |x| x.word());
                        let tag = match ch { Some(false) => "Regression", Some(true) => "Improved", None => "" };
                        rsx! {
                            li { key: "{i}", class: "oxbk-ev-case",
                                button { class: "oxbk-ev-cb", r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-ev-d-{i}",
                                    onclick: move |_| act(format!("ev-open-{i}")),
                                    span { class: "oxbk-ev-oo", "aria-hidden": "true", i { "data-o": "{c.base.word()}", "{glyph(Some(c.base))}" } {ico("chevron-right", 10.0)} i { "data-o": "{ow}", "{glyph(o)}" } }
                                    span { class: "oxaik-sr", "Baseline {c.base.word()}, new {ows}. " }
                                    span { class: "oxbk-ev-ct2", "{c.title}" }
                                    span { class: "oxaik-mute oxbk-ev-cc", "{CATEGORIES[c.cat].0}" }
                                    if !tag.is_empty() { span { class: "oxaik-pill", "data-hue": if ch == Some(false) { "red" } else { "green" }, "{tag}" } }
                                    if tri == 1 { span { class: "oxaik-pill", {ico("check", 11.0)} span { class: "oxbk-ev-tw", "Expected" } } }
                                    if tri == 2 { span { class: "oxaik-pill", {ico("external", 11.0)} span { class: "oxbk-ev-tw", "Issue" } } }
                                }
                                if open {
                                    {
                                        let (input, expected, bo, no, scores, why) = e.detail(i);
                                        rsx! {
                                            div { class: "oxbk-ev-det oxaik-fade", id: "oxbk-ev-d-{i}", role: "region", "aria-label": "{c.title}",
                                                dl { class: "oxbk-ev-io",
                                                    div { dt { "Input" } dd { "{input}" } }
                                                    div { dt { "Expected" } dd { "{expected}" } }
                                                }
                                                div { class: "oxbk-ev-outs",
                                                    div { p { class: "oxbk-cap oxaik-row", {mark(BASELINE.1, 12.0)} "{BASELINE.0}" } p { "{bo}" } }
                                                    div { p { class: "oxbk-cap oxaik-row", {mark(CANDIDATE.1, 12.0)} "{CANDIDATE.0}" } p { "{no}" } }
                                                }
                                                ul { class: "oxbk-ev-rub",
                                                    for (r, (b, nw)) in scores.iter().enumerate() {
                                                        li { key: "{r}", span { "{RUBRIC[r]}" } span { class: "oxaik-bar", span { style: "width:{*nw as f64 * 20.0}%" } } span { class: "oxaik-num", "{nw}/5" } span { class: "oxaik-num oxaik-mute", "was {b}" } }
                                                    }
                                                }
                                                p { class: "oxaik-mute", "{why}" }
                                                div { class: "oxaik-row",
                                                    if tri == 0 {
                                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ev-expected-{i}")), {ico("check", 12.0)} "Mark as expected" }
                                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ev-issue-{i}")), {ico("external", 12.0)} "Open issue" }
                                                    } else {
                                                        span { class: "oxaik-mute", if tri == 1 { "Marked as expected." } else { "Issue opened (simulated)." } }
                                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ev-undo-{i}")), {ico("undo", 12.0)} "Undo" }
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
            if list.len() > 8 {
                button { class: "oxaik-btn sm oxbk-ghost", r#type: "button", onclick: move |_| { let v = !*more.read(); more.set(v) },
                    if *more.read() { "Show fewer" } else { "Show all {list.len()}" }
                }
            }
            {live(&said.read())}
        }
    }
}
