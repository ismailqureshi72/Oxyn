//! Screens S5: orchestration and builders. [`EvalsScreen`] (new): a prompt's eval with pass rate
//! by version, a reviewable prompt diff and test cases you can re-run, on the Evalkit persona's
//! sidebar. The Orchestration and Workflow Builder screens were upgraded in place
//! (`screens2fid`, `screensfid`); their state bars come from [`s5_states`]. Styles in
//! `buildscr.css` (`oxev-` Evals). Chrome stays monochrome; colour is for official marks and
//! status.

use dioxus::prelude::*;
use oxyn_core::parity::buildscr::{
    COST_1K, DIFF, EVAL_MODELS, HISTORY, Piece, ScreenState, VERSIONS, Verdict, avg_tenths,
    case_cost_tenths, case_id, case_view, change_of, diff_changes, eur, eval_cases, p50_ms,
    pass_rate, prompt_text, run_next, score_on, signed_tenths, split, states_for, tenths,
    verdict_counts,
};

use super::screensfid::{AppFrame, MenuButton, MenuItem, ico, nico, use_threads};
use crate::brand::BrandIcon;

/// The state bar entries of each S5 screen (for a host's `ScreenStage { states }`).
pub fn s5_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

fn tone(v: Verdict) -> &'static str {
    match v {
        Verdict::Pass => "ok",
        Verdict::Partial => "warn",
        Verdict::Fail => "bad",
    }
}

fn delta(text: String, good: bool) -> Element {
    rsx! { span { class: "oxev-d", "data-good": "{good}", "{text}" } }
}

/// Screen – Evals: a prompt's eval. Tiles for pass rate, average score, latency and cost against
/// the version before; pass rate across every version as a chart whose points you can step
/// through; the system prompt's changes as a reviewable diff (step through the changes, Accept
/// all or Reject all); and the test cases with filters, a row that opens to the full reply, and
/// Re-run on each one. Run eval scores every case in turn. States: `sample`, `loading`, `empty`,
/// `running`.
#[component]
pub fn EvalsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: failures and partials first.
    #[props(default)]
    fails_first: bool,
    /// Oxyn: a cost column per case and the run's total.
    #[props(default)]
    cost_col: bool,
) -> Element {
    let (st, own) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let loading = st == ScreenState::Loading;
    let cases = use_signal(move || eval_cases(st == ScreenState::Sample));
    let mut ver = use_signal(|| 2usize);
    let mut model = use_signal(|| 0usize);
    // The model the shown scores were made on (None: not run on the picked model yet).
    let mut scored = use_signal(|| Some(0usize));
    let mut filter = use_signal(|| "all");
    let mut open = use_signal(|| None::<u32>);
    // A running eval: the case index being scored.
    let mut run = use_signal(move || if own == "running" { Some(8usize) } else { None });
    let mut one = use_signal(|| None::<u32>);
    let mut rerun = use_signal(Vec::<u32>::new);
    let mut change = use_signal(|| 0usize);
    let mut decided = use_signal(|| None::<bool>);
    let mut point = use_signal(|| 11usize);
    let mut said = use_signal(String::new);
    let mut last = use_signal(|| "2 hours ago".to_string());
    let all = cases();
    let n = all.len();
    let v = ver();
    let m = model();
    let fresh = scored() == Some(m);
    let changes = diff_changes();
    let nch = changes.len();
    let rate = pass_rate(&all, v, m);
    let prev = v.saturating_sub(1);
    let counts = verdict_counts(&all, v, m);
    let shown = case_view(&all, v, m, filter(), fails_first);
    let running = run();
    let total_cost: u32 = all.iter().map(case_cost_tenths).sum();
    let vlabel = VERSIONS[v];
    let models: Vec<MenuItem> = EVAL_MODELS.iter().map(|(l, _)| (*l, *l, "")).collect();
    let versions: Vec<MenuItem> = vec![
        ("v12", "v12 · current", ""),
        ("v11", "v11 · Nov 9", ""),
        ("v10", "v10 · Oct 28", ""),
    ];
    // Chart geometry: 12 points across 0..100, rate 0..100 % on 0..100 high (inverted).
    let px = |i: usize| (i as f32) * 100.0 / 11.0;
    let py = |r: u32| 100.0 - r as f32;
    let mut path = String::new();
    for (i, r) in HISTORY.iter().enumerate() {
        path.push_str(&format!(
            "{}{:.2} {:.2} ",
            if i == 0 { "M" } else { "L" },
            px(i),
            py(*r)
        ));
    }
    let pt = point();
    let tiles: Vec<(&str, String, &str, Element)> = if n == 0 {
        Vec::new()
    } else {
        let dr = rate as i32 - pass_rate(&all, prev, m) as i32;
        let da = avg_tenths(&all, v, m) as i32 - avg_tenths(&all, prev, m) as i32;
        let dl = p50_ms(&all, v) as i32 / 100 - p50_ms(&all, prev) as i32 / 100;
        let dc = COST_1K[v] as i32 - COST_1K[prev] as i32;
        let vs = VERSIONS[prev];
        let same = v == prev;
        vec![
            (
                "Pass rate",
                format!("{rate}%"),
                "",
                if same {
                    rsx! {}
                } else {
                    delta(
                        format!("{}{dr} vs {vs}", if dr >= 0 { "+" } else { "" }),
                        dr >= 0,
                    )
                },
            ),
            (
                "Avg score",
                tenths(avg_tenths(&all, v, m)),
                "/5",
                if same {
                    rsx! {}
                } else {
                    delta(format!("{} vs {vs}", signed_tenths(da)), da >= 0)
                },
            ),
            (
                "Latency p50",
                tenths(p50_ms(&all, v) / 100),
                "s",
                if same {
                    rsx! {}
                } else {
                    delta(format!("{} s vs {vs}", signed_tenths(dl)), dl <= 0)
                },
            ),
            (
                "Cost per 1k runs",
                eur(COST_1K[v]),
                "",
                if same {
                    rsx! {}
                } else {
                    delta(format!("+{} vs {vs}", eur(dc.unsigned_abs())), dc <= 0)
                },
            ),
        ]
    };
    let actions = rsx! {
        span { class: "oxv-q oxev-last", if running.is_some() { "Running…" } else { "Last run {last}" } }
        if running.is_some() {
            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { run.set(None); said.set("Eval stopped".to_string()); }, {ico("stop")} span { class: "oxscn-btn-l", "Stop" } }
        } else {
            button { r#type: "button", class: "oxscn-primary oxev-run", disabled: n == 0,
                onclick: move |_| { run.set(Some(0)); rerun.set(Vec::new()); said.set(format!("Running {n} cases on {}", EVAL_MODELS[model()].0)); },
                {ico("runs")} span { class: "oxscn-btn-l", "Run eval" } }
        }
    };
    let badge = rsx! {
        span { class: "oxev-crumbs",
            MenuButton { label: if v == 2 { format!("{vlabel} current") } else { vlabel.to_string() }, aria: "Prompt version {vlabel}", items: versions, checked: vlabel.to_string(), caret: true, class: "oxev-ver",
                on_pick: move |k: String| { let i = VERSIONS.iter().position(|x| *x == k).unwrap_or(2); ver.set(i); point.set(9 + i); said.set(format!("Showing {k}")); } }
            MenuButton { label: EVAL_MODELS[m].0.to_string(), aria: "Model {EVAL_MODELS[m].0}", items: models, checked: EVAL_MODELS[m].0.to_string(), brand: EVAL_MODELS[m].1.to_string(), caret: true, class: "oxev-model",
                on_pick: move |k: String| { let i = EVAL_MODELS.iter().position(|x| x.0 == k).unwrap_or(0); model.set(i); said.set(if scored() == Some(i) { format!("Scores on {k}") } else { format!("{k} picked – press Run eval to score it") }); } }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "evalkit", nav_on: "evals", crumb: "Evals", crumb_icon: "flask", title: "Support reply prompt",
            badge, actions,
            div { class: "oxev",
                if loading {
                    div { class: "oxev-tiles", "aria-hidden": "true", for i in 0..4 { div { key: "{i}", class: "oxev-tile", span { class: "oxv-skl" } span { class: "oxv-skb" } } } }
                    div { class: "oxev-mid", "aria-hidden": "true", div { class: "oxev-card oxev-sk" } div { class: "oxev-card oxev-sk" } }
                    div { class: "oxa-skrows", "aria-hidden": "true", for i in 0..5 { div { key: "{i}", class: "oxa-skrow", span { class: "oxv-skb" } span { class: "oxv-skl" } } } }
                } else if n == 0 {
                    div { class: "oxv-none",
                        span { class: "oxv-none-g", {nico("flask")} }
                        p { class: "oxv-none-t", "No eval runs yet" }
                        p { class: "oxv-none-l", "Add test cases or bring them in from a dataset, then press Run eval to score this prompt." }
                        div { class: "oxev-none-b",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| said.set("New test case".to_string()), {ico("plus")} span { "Add test case" } }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| said.set("Pick a dataset".to_string()), {nico("db")} span { "From a dataset" } }
                        }
                    }
                } else {
                    if let Some(i) = running {
                        div { class: "oxev-prog", role: "progressbar", "aria-label": "Eval progress", "aria-valuemin": "0", "aria-valuemax": "{n}", "aria-valuenow": "{i}",
                            span { class: "oxev-prog-t", "Scoring {case_id(i as u32 + 1)} · {i} of {n} done" }
                            span { class: "oxev-prog-bar", span { style: "width:{i * 100 / n}%" } }
                            span { key: "{i}", class: "oxev-tick",
                                onanimationend: move |_| {
                                    match run_next(i, n) {
                                        Some(j) => run.set(Some(j)),
                                        None => { run.set(None); scored.set(Some(model())); last.set("just now".to_string()); said.set(format!("Eval done: {}% pass", pass_rate(&cases.peek(), ver(), model()))); }
                                    }
                                } }
                        }
                    }
                    if !fresh && running.is_none() {
                        p { class: "oxev-stale", role: "note", {nico("help")} span { "Not run on {EVAL_MODELS[m].0} yet. Scores below are from {EVAL_MODELS[scored().unwrap_or(0)].0}; press Run eval to score this model." } }
                    }
                    div { class: "oxev-tiles",
                        for (l, val, unit, d) in tiles {
                            div { key: "{l}", class: "oxev-tile",
                                span { class: "oxv-q2 oxev-tl", "{l}" }
                                div { class: "oxev-tv", b { "{val}" } if !unit.is_empty() { span { class: "oxv-q2", "{unit}" } } span { class: "oxscn-grow" } {d} }
                            }
                        }
                    }
                    div { class: "oxev-mid",
                        section { class: "oxev-card oxev-chartc", "aria-label": "Pass rate by version",
                            p { class: "oxv-q2 oxev-ch-t", "Pass rate by version" }
                            div { class: "oxev-ch-big", b { "v{pt + 1} · {HISTORY[pt]}%" } if pt > 0 { {delta(format!("{}{}%", if HISTORY[pt] >= HISTORY[pt - 1] { "+" } else { "−" }, HISTORY[pt].abs_diff(HISTORY[pt - 1])), HISTORY[pt] >= HISTORY[pt - 1])} } }
                            div { class: "oxev-chart",
                                div { class: "oxev-y", span { "100%" } span { "50%" } span { "0%" } }
                                div { class: "oxev-plot", role: "listbox", "aria-label": "Versions", tabindex: "0", "aria-activedescendant": "oxev-p{pt}",
                                    onkeydown: move |e: KeyboardEvent| {
                                        let j = match e.key() { Key::ArrowRight => (pt + 1).min(11), Key::ArrowLeft => pt.saturating_sub(1), Key::Home => 0, Key::End => 11, _ => return };
                                        e.prevent_default();
                                        point.set(j);
                                        if j >= 9 { ver.set(j - 9); }
                                        said.set(format!("v{} · {}%", j + 1, HISTORY[j]));
                                    },
                                    svg { view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                                        line { x1: "0", x2: "100", y1: "50", y2: "50", class: "oxev-grid", vector_effect: "non-scaling-stroke" }
                                        line { x1: "{px(pt)}", x2: "{px(pt)}", y1: "0", y2: "100", class: "oxev-cur", vector_effect: "non-scaling-stroke" }
                                        path { d: "{path}", class: "oxev-line", vector_effect: "non-scaling-stroke" }
                                    }
                                    for (i, r) in HISTORY.iter().copied().enumerate() {
                                        span { key: "{i}", id: "oxev-p{i}", role: "option", class: "oxev-pt", "aria-selected": "{i == pt}", "aria-label": "v{i + 1}: {r}%", style: "left:{px(i)}%;top:{py(r)}%",
                                            onclick: move |_| { point.set(i); if i >= 9 { ver.set(i - 9); } said.set(format!("v{} · {r}%", i + 1)); } }
                                    }
                                }
                            }
                            div { class: "oxev-x", span { "v1" } span { "v7" } span { "v12" } }
                            p { class: "oxv-q oxev-foot", "Last 12 versions · {n} test cases · {EVAL_MODELS[m].0}" }
                        }
                        section { class: "oxev-card oxev-diff", "aria-label": "Changes in v12",
                            p { class: "oxev-diff-h", {nico("spark")} span { class: "oxv-ell", "Changes in v12 · system prompt, edited by Wren Ashby" } }
                            if let Some(acc) = decided() {
                                p { class: "oxev-text", "{prompt_text(acc)}" }
                                div { class: "oxev-diff-f",
                                    span { class: "oxv-q", if acc { "Accepted – v12 is the prompt in use" } else { "Rejected – back to v11's prompt" } }
                                    span { class: "oxscn-grow" }
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { decided.set(None); said.set("Undone".to_string()); }, {ico("undo")} span { "Undo" } }
                                }
                            } else {
                                p { class: "oxev-text",
                                    for (i, p) in DIFF.iter().copied().enumerate() {
                                        {
                                            let cur = change_of(i) == Some(change());
                                            match p {
                                                Piece::Same(t) => rsx! { span { key: "{i}", "{t}" } },
                                                Piece::Del(t) => rsx! { del { key: "{i}", class: "oxev-del", "data-cur": "{cur}", "{t}" } },
                                                Piece::Add(t) => rsx! { ins { key: "{i}", class: "oxev-add", "data-cur": "{cur}", "{t}" } },
                                            }
                                        }
                                    }
                                }
                                div { class: "oxev-diff-f",
                                    span { class: "oxev-cnt", "aria-label": "Change {change() + 1} of {nch}", "{change() + 1}/{nch}" }
                                    span { class: "oxscn-grow" }
                                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Previous change", onclick: move |_| { let c = (change() + nch - 1) % nch; change.set(c); said.set(format!("Change {} of {nch}", c + 1)); }, span { class: "oxev-flip", {nico("next")} } }
                                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Next change", onclick: move |_| { let c = (change() + 1) % nch; change.set(c); said.set(format!("Change {} of {nch}", c + 1)); }, {nico("next")} }
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { decided.set(Some(false)); said.set("All changes rejected".to_string()); }, "Reject all" }
                                    button { r#type: "button", class: "oxscn-primary", onclick: move |_| { decided.set(Some(true)); said.set("All changes accepted".to_string()); }, "Accept all" }
                                }
                            }
                        }
                    }
                    div { class: "oxev-th",
                        h2 { class: "oxev-h2", "Test cases " span { class: "oxv-q2", "{n}" } }
                        span { class: "oxscn-grow" }
                        if cost_col { span { class: "oxv-q oxev-tot", "Run cost {eur(total_cost / 10)}" } }
                        div { class: "oxv-seg", role: "group", "aria-label": "Show",
                            for (k, l, c) in [("all", "All", n as u32), ("pass", "Pass", counts[0]), ("partial", "Partial", counts[1]), ("fail", "Fail", counts[2])] {
                                button { key: "{k}", r#type: "button", class: "oxv-seg-t", "aria-pressed": "{filter() == k}", onclick: move |_| filter.set(k),
                                    "{l}" if k != "all" { span { class: "oxev-segn", "{c}" } } }
                            }
                        }
                    }
                    div { class: "oxev-table", role: "table", "aria-label": "Test cases", "data-cost": "{cost_col}",
                        div { class: "oxev-tr oxev-thd", role: "row",
                            span { role: "columnheader", "Input" } span { role: "columnheader", class: "oxev-c-exp", "Expected" } span { role: "columnheader", class: "oxev-c-out", "{vlabel} output" }
                            span { role: "columnheader", "Score" } span { role: "columnheader", class: "oxev-c-lat", "Latency" }
                            if cost_col { span { role: "columnheader", class: "oxev-c-cost", "Cost" } }
                            span { role: "columnheader", class: "oxscn-sr", "Re-run" }
                        }
                        if shown.is_empty() { p { class: "oxa-mute oxev-pad", "No {filter()} cases on {vlabel}." } }
                        for c in shown {
                            {
                                let id = c.n;
                                let idx = (id - 1) as usize;
                                let is_open = open() == Some(id);
                                let busy = running.is_some_and(|r| r == idx) || one() == Some(id);
                                let queued = running.is_some_and(|r| r < idx);
                                let sc = score_on(&c, v, m);
                                let vd = Verdict::of(sc);
                                let again = rerun().contains(&id);
                                rsx! {
                                    div { key: "{id}", class: "oxev-tr", role: "row", "data-open": "{is_open}",
                                        button { r#type: "button", class: "oxev-in", role: "cell", "aria-expanded": "{is_open}",
                                            onclick: move |_| open.set(if is_open { None } else { Some(id) }),
                                            span { class: "oxev-id", "{case_id(id)}" }
                                            BrandIcon { name: c.channel.to_string(), size: 12.0, coloured: true }
                                            span { class: "oxv-ell", "{c.input}" }
                                        }
                                        span { role: "cell", class: "oxev-c-exp oxv-ell", "{c.expected}" }
                                        span { role: "cell", class: "oxev-c-out oxv-ell", "{c.output}" }
                                        span { role: "cell", class: "oxev-sc",
                                            if busy { span { class: "oxev-spin", "aria-label": "Scoring" } span { class: "oxv-q", "Scoring" } }
                                            else if queued { span { class: "oxv-q", "Queued" } }
                                            else { span { class: "oxa-pill", "data-tone": "{tone(vd)}", span { class: "oxev-dot" } "{vd.label()} {sc}/5" } }
                                        }
                                        span { role: "cell", class: "oxev-c-lat oxev-num", if again { "{tenths(c.ms / 100)}s ↻" } else { "{tenths(c.ms / 100)}s" } }
                                        if cost_col { span { role: "cell", class: "oxev-c-cost oxev-num", "€0.{case_cost_tenths(&c):03}" } }
                                        span { role: "cell", class: "oxev-rr",
                                            if one() == Some(id) {
                                                span { key: "t{id}", class: "oxev-tick", onanimationend: move |_| { one.set(None); rerun.write().push(id); said.set(format!("{} re-run: {}", case_id(id), Verdict::of(score_on(&cases.peek()[idx], ver(), model())).label())); } }
                                            }
                                            button { r#type: "button", class: "oxscn-ib", "aria-label": "Re-run {case_id(id)}", disabled: busy || running.is_some(),
                                                onclick: move |_| { one.set(Some(id)); said.set(format!("Re-running {}", case_id(id))); }, {ico("redo")} }
                                        }
                                        if is_open {
                                            div { class: "oxev-more", role: "cell",
                                                div { span { class: "oxv-q2", "Input" } p { "{c.input}" } }
                                                div { span { class: "oxv-q2", "Expected" } p { "{c.expected}" } }
                                                div { span { class: "oxv-q2", "{vlabel} output" } p { "{c.output}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}
