//! Test ticks, Triage metro and Loop pipeline (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::secs;
use oxyn_core::xpost::loops::{Loops, STATIONS as LP_STATIONS, TASK as LP_TASK};
use oxyn_core::xpost::metro::{
    FINDINGS, H as MH, Metro, OUTCOMES, STATIONS as M_STATIONS, TERM_X, TERM_Y, W as MW, path,
};
use oxyn_core::xpost::ticks::{FILES, FIX, SUITE, TestTicks as Ticks, Tick, file_of, total};

use super::{cls, ico, key_of, live, owned};

/// Test ticks: a test run drawn as one tick per test, a row per file. Ticks fill as the runner
/// reports (queued, running, passed, failed); the timer, counters and tests-per-second roll with
/// it. A run that ends with failures dims the passing rows, keeps the failures lit and opens a fix
/// card with a diff over the failing file; Apply fix re-runs only the failed tests.
#[component]
pub fn TestTicks(
    /// Host clock.
    now: ClockMs,
    /// Two tests fail (error scenario with the fix card).
    #[props(default)]
    fail: bool,
    /// Speed factor (1 = sample timing).
    #[props(default = 1.0)]
    speed: f64,
    /// Show the tests-per-second rate.
    #[props(default = true)]
    show_rate: bool,
    /// One dense grid without file rows.
    #[props(default)]
    compact: bool,
) -> Element {
    let t0 = now.0;
    let mut st = use_signal(move || Ticks::new(fail, speed, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let t = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let (p, f, done_n) = t.counts(n);
    let done = t.done(n);
    let ended = done && f > 0;
    let fix = t.fix_open(n);
    let rate = t.rate(n);
    let running = !done && t.stopped.is_none();
    let mut start = 0usize;
    let rows: Vec<(usize, &str, usize, usize)> = FILES
        .iter()
        .enumerate()
        .map(|(fi, (name, cnt))| {
            let s = start;
            start += cnt;
            (fi, *name, s, *cnt)
        })
        .collect();
    let fail_file = file_of(oxyn_core::xpost::ticks::FAILS[0]).0;
    let sel = t.sel;
    rsx! {
        section { class: "oxxp oxxp-tt", "aria-label": "Test run",
            header { class: "oxxp-hd",
                div { class: "oxxp-hl",
                    h3 { class: "oxxp-h", "Running tests" }
                    p { class: "oxxp-sub", "{SUITE.0} · {SUITE.1}" }
                }
                div { class: "oxxp-ctl",
                    if running { button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("tt-stop".into()), {ico("stop", 11.0)} "Stop" } }
                    else { button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("tt-run".into()), {ico("retry", 12.0)} "Run again" } }
                }
            }
            div { class: "oxxp-tt-stats", role: "group", "aria-label": "Totals",
                span { class: "oxxp-stat", b { class: "oxxp-num", "{secs(t.elapsed(n))}" } small { "elapsed" } }
                span { class: "oxxp-stat", "data-hue": "green", b { class: "oxxp-num", "{p}" } small { "passed" } }
                span { class: "oxxp-stat", "data-hue": if f > 0 { "red" } else { "neutral" }, b { class: "oxxp-num", "{f}" } small { "failed" } }
                if show_rate { span { class: "oxxp-stat", b { class: "oxxp-num", "{rate:.1}" } small { "tests/s" } } }
                span { class: "oxxp-stat oxxp-tt-of", b { class: "oxxp-num", "{done_n}/{total()}" } small { "done" } }
            }
            div { class: "oxxp-tt-grid", "data-ended": "{ended}", "data-compact": "{compact}", role: "grid", tabindex: "0",
                "aria-label": "Tests, arrows to move, Escape to clear", "aria-busy": "{running}",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if owned(&k) || k == "Enter" { e.prevent_default(); }
                    if let Some(x) = st.write().key(&k, n) { said.set(x); }
                },
                for (fi, name, s, cnt) in rows {
                    if !(t.only_failed && !(s..s + cnt).any(|i| t.tick(i, n) == Tick::Fail)) {
                        div { key: "{fi}", class: "oxxp-tt-row", role: "row", "data-lit": "{(s..s + cnt).any(|i| t.tick(i, n) == Tick::Fail)}",
                            if !compact { span { class: "oxxp-tt-file oxxp-mono", role: "rowheader", "{name}" } }
                            span { class: "oxxp-tt-ticks",
                                for i in s..s + cnt {
                                    button { key: "{i}", r#type: "button", tabindex: "-1", role: "gridcell",
                                        class: cls("oxxp-tick", sel == Some(i)), "data-st": "{t.tick(i, n).name()}",
                                        "aria-label": "{t.describe(i, n)}",
                                        onclick: move |_| act(format!("tt-tick-{i}")) }
                                }
                            }
                        }
                    }
                }
                if fix {
                    div { class: "oxxp-tt-fix", role: "dialog", "aria-label": "Suggested fix", style: "--row:{fail_file}",
                        div { class: "oxxp-row oxxp-between",
                            span { class: "oxxp-row", {ico("spark", 12.0)} b { "Fix for {FILES[fail_file].0}" } }
                            button { class: "oxxp-ib", r#type: "button", "aria-label": "Close fix", onclick: move |_| act("tt-fix-close".into()), {ico("x", 12.0)} }
                        }
                        pre { class: "oxxp-diff oxxp-mono",
                            for (k, (sign, line)) in FIX.iter().enumerate() {
                                span { key: "{k}", "data-sign": "{sign}", "{sign} {line}\n" }
                            }
                        }
                        div { class: "oxxp-row",
                            button { class: "oxxp-btn sm pri", r#type: "button", onclick: move |_| act("tt-fix".into()), "Apply fix and re-run" }
                            span { class: "oxxp-mute", "Enter" }
                        }
                    }
                }
            }
            footer { class: "oxxp-row oxxp-between",
                span { class: "oxxp-row oxxp-mute",
                    span { class: "oxxp-dot", "data-hue": if f > 0 { "red" } else if done { "green" } else { "neutral" } }
                    "{t.status(n)}"
                }
                button { r#type: "button", role: "switch", class: cls("oxxp-switch", t.only_failed), "aria-checked": "{t.only_failed}", onclick: move |_| act("tt-filter".into()), span { class: "oxxp-knob" } "Failed only" }
            }
            if let Some(i) = sel { p { class: "oxxp-mute oxxp-tt-sel", "{t.describe(i, n)}" } }
            {live(&said())}
        }
    }
}

fn poly(o: usize) -> String {
    let p = path(o);
    format!(
        "M{} {} L{} {} L{} {} L{} {}",
        p[0].0, p[0].1, p[1].0, p[1].1, p[2].0, p[2].1, p[3].0, p[3].1
    )
}

/// Triage metro: security findings ride a metro map from Scan to Triage and branch to Patched,
/// False positive or Needs you. Terminal counters roll as dots land; pick a dot (or use the
/// arrows) to open its card, and patch or dismiss what is waiting on you.
#[component]
pub fn TriageMetro(
    /// Host clock.
    now: ClockMs,
    /// Start over once every finding has landed (until someone interacts).
    #[props(default = true)]
    looping: bool,
    /// Station and terminal labels on the map.
    #[props(default = true)]
    labels: bool,
    /// Terminal counters under the map.
    #[props(default = true)]
    show_counts: bool,
) -> Element {
    let t0 = now.0;
    let mut st = use_signal(move || Metro::new(looping, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let m = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let counts = m.counts(n);
    let open = m.open;
    rsx! {
        section { class: "oxxp oxxp-tm", "aria-label": "Triage metro",
            header { class: "oxxp-hd",
                div { class: "oxxp-hl",
                    h3 { class: "oxxp-h", "Security triage" }
                    p { class: "oxxp-sub", "quillworks-api · 974 files · 6 checkers" }
                }
                div { class: "oxxp-ctl",
                    button { class: "oxxp-btn sm", r#type: "button", "aria-pressed": "{m.paused.is_some()}", onclick: move |_| act("tm-pause".into()),
                        {ico(if m.paused.is_some() { "play" } else { "pause" }, 11.0)} if m.paused.is_some() { "Resume" } else { "Pause" } }
                }
            }
            div { class: "oxxp-tm-map", tabindex: "0", role: "group", "aria-label": "Map of findings, arrows to move between them",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if owned(&k) { e.prevent_default(); }
                    if let Some(x) = st.write().key(&k, n) { said.set(x); }
                },
                svg { view_box: "0 0 {MW} {MH}", "aria-hidden": "true", preserve_aspect_ratio: "xMidYMid meet",
                    for o in 0..3usize {
                        path { key: "{o}", class: "oxxp-tm-line", "data-hue": "{OUTCOMES[o].1}", d: "{poly(o)}" }
                    }
                    for (k, (x, _)) in M_STATIONS.iter().enumerate() {
                        circle { key: "s{k}", class: "oxxp-tm-stn", cx: "{x}", cy: "100", r: "7" }
                    }
                    for o in 0..3usize {
                        circle { key: "t{o}", class: "oxxp-tm-term", "data-hue": "{OUTCOMES[o].1}", cx: "{TERM_X}", cy: "{TERM_Y[o]}", r: "9" }
                    }
                    if labels {
                        for (k, (x, l)) in M_STATIONS.iter().enumerate() {
                            text { key: "l{k}", class: "oxxp-tm-lab", x: "{x}", y: "128", text_anchor: "middle", "{l}" }
                        }
                    }
                    for i in 0..FINDINGS.len() {
                        if let Some((x, y)) = m.point(i, n) {
                            circle { key: "d{i}", class: cls("oxxp-tm-dot", open == Some(i)), "data-hue": if m.landed(i, n) { OUTCOMES[m.outcome(i)].1 } else { "ride" }, cx: "{x:.1}", cy: "{y:.1}", r: if open == Some(i) { "7" } else { "5" } }
                        }
                    }
                }
                div { class: "oxxp-tm-hits",
                    for i in 0..FINDINGS.len() {
                        if let Some((x, y)) = m.point(i, n) {
                            button { key: "{i}", r#type: "button", tabindex: "-1", class: "oxxp-tm-hit",
                                style: "left:{x / MW * 100.0:.2}%;top:{y / MH * 100.0:.2}%",
                                "aria-label": "{m.describe(i, n)}", onclick: move |_| act(format!("tm-dot-{i}")) }
                        }
                    }
                }
                if labels {
                    div { class: "oxxp-tm-terms", "aria-hidden": "true",
                        for o in 0..3usize { span { key: "{o}", style: "top:{TERM_Y[o] / MH * 100.0:.1}%", "data-hue": "{OUTCOMES[o].1}", "{OUTCOMES[o].0}" } }
                    }
                }
            }
            if show_counts {
                div { class: "oxxp-tm-counts",
                    for o in 0..3usize {
                        span { key: "{o}", class: "oxxp-stat", "data-hue": "{OUTCOMES[o].1}", b { class: "oxxp-num", key: "{counts[o]}", "{counts[o]}" } small { "{OUTCOMES[o].0}" } }
                    }
                }
            }
            if let Some(i) = open {
                div { class: "oxxp-tm-card oxxp-in", role: "dialog", "aria-label": "{FINDINGS[i].0}",
                    div { class: "oxxp-row oxxp-between",
                        b { "{FINDINGS[i].0}" }
                        button { class: "oxxp-ib", r#type: "button", "aria-label": "Close", onclick: move |_| act("tm-close".into()), {ico("x", 12.0)} }
                    }
                    p { class: "oxxp-row oxxp-mute",
                        span { class: "oxxp-pill", "data-hue": match FINDINGS[i].2 { "High" => "red", "Medium" => "amber", _ => "neutral" }, "{FINDINGS[i].2}" }
                        span { class: "oxxp-mono", "{FINDINGS[i].1}" }
                        span { if m.landed(i, n) { "{OUTCOMES[m.outcome(i)].0}" } else { "On its way" } }
                    }
                    if m.landed(i, n) && m.outcome(i) == 2 {
                        div { class: "oxxp-row",
                            button { class: "oxxp-btn sm pri", r#type: "button", onclick: move |_| act("tm-patch".into()), "Approve patch" }
                            button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("tm-dismiss".into()), "Not a risk" }
                        }
                    }
                }
            }
            footer { class: "oxxp-mute", "{m.status(n)}" }
            {live(&said())}
        }
    }
}

/// Loop pipeline: an agent working in rounds until a check passes. A marker travels Work → Check
/// → Improve round a loop, the round counter ticks and each round leaves a scored chip. A round at
/// or over the target exits to Done; running out of rounds stops with the best score and offers
/// one more round.
#[component]
pub fn LoopPipeline(
    /// Host clock.
    now: ClockMs,
    /// Target score (percent).
    #[props(default = 85)]
    target: u32,
    /// Most rounds.
    #[props(default = 5)]
    max_rounds: usize,
    /// Round chips with scores.
    #[props(default = true)]
    show_history: bool,
) -> Element {
    let t0 = now.0;
    let mut st = use_signal(move || Loops::new(target, max_rounds, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let l = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let at = l.at(n);
    let passed = l.passed(n);
    let out = l.out(n);
    let scores = l.scored(n);
    let round = at.map(|a| a.0 + 1).unwrap_or(scores.len());
    // Marker angle round the loop: Work at the top, Check right, Improve left.
    let ang = at
        .map(|(_, s, f)| (s as f64 + f) / 3.0 * 360.0)
        .unwrap_or(120.0);
    let sel = l.sel;
    rsx! {
        section { class: "oxxp oxxp-lp", "aria-label": "Loop pipeline", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if owned(&k) { e.prevent_default(); }
                if let Some(x) = st.write().key(&k, n) { said.set(x); }
            },
            header { class: "oxxp-hd",
                div { class: "oxxp-hl",
                    h3 { class: "oxxp-h", "{LP_TASK.0}" }
                    p { class: "oxxp-sub", "Loops until {LP_TASK.1} ≥ {l.target}% · up to {l.max} rounds" }
                }
                div { class: "oxxp-ctl",
                    if !passed && !out {
                        button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("lp-pause".into()), {ico(if l.paused.is_some() { "play" } else { "pause" }, 11.0)} if l.paused.is_some() { "Resume" } else { "Pause" } }
                        button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("lp-stop".into()), {ico("stop", 11.0)} "Stop" }
                    } else {
                        button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("lp-restart".into()), {ico("retry", 12.0)} "Start over" }
                    }
                }
            }
            div { class: "oxxp-lp-body",
                div { class: "oxxp-lp-ring", "data-state": if passed { "done" } else if out { "out" } else { "run" }, "aria-hidden": "true",
                    svg { view_box: "0 0 160 160",
                        circle { class: "oxxp-lp-track", cx: "80", cy: "80", r: "58" }
                        if at.is_some() { circle { class: "oxxp-lp-mark", cx: "{80.0 + 58.0 * (ang.to_radians()).sin():.1}", cy: "{80.0 - 58.0 * (ang.to_radians()).cos():.1}", r: "6" } }
                    }
                    for (k, s) in LP_STATIONS.iter().enumerate() {
                        span { key: "{k}", class: cls("oxxp-lp-stn", at.is_some_and(|a| a.1 == k)), "data-k": "{k}", "{s}" }
                    }
                    div { class: "oxxp-lp-mid",
                        b { class: "oxxp-num", key: "{round}", "{round}" }
                        small { if passed { "rounds · done" } else if out { "rounds · stopped" } else { "round" } }
                    }
                }
                div { class: "oxxp-lp-side",
                    div { class: "oxxp-lp-score",
                        span { class: "oxxp-mute", "Best" }
                        b { class: "oxxp-num", "data-hue": if passed { "green" } else if out { "amber" } else { "neutral" }, "{l.best(n)}%" }
                        div { class: "oxxp-bar", span { style: "width:{l.best(n)}%" } i { style: "left:{l.target}%" } }
                    }
                    if show_history {
                        div { class: "oxxp-lp-hist", role: "list", "aria-label": "Rounds",
                            for (i, s) in scores.iter().enumerate() {
                                button { key: "{i}", r#type: "button", role: "listitem", class: cls("oxxp-chip oxxp-in", sel == Some(i)),
                                    "data-hue": if *s >= l.target { "green" } else { "neutral" },
                                    onclick: move |_| act(format!("lp-round-{i}")),
                                    "#{i + 1} " b { class: "oxxp-num", "{s}%" }
                                }
                            }
                        }
                    }
                    if out && l.stopped.is_none() {
                        button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("lp-more".into()), {ico("plus", 12.0)} "One more round" }
                    }
                }
            }
            footer { class: "oxxp-row oxxp-mute",
                span { class: "oxxp-dot", "data-hue": if passed { "green" } else if out { "amber" } else { "blue" } }
                "{l.status(n)}"
            }
            {live(&said())}
        }
    }
}
