//! Ask your data (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::ask::{Ask, CHARTS, Phase, SOURCE, SUGGESTIONS};
use oxyn_core::aikit::first_words;

use super::{dots, ico, key_of, live, mark, seg};

/// Ask your data: a question about a database becomes a chart. The SQL the model writes streams
/// into a foldable Generated SQL panel (tinted by token type), the query runs with a rolling row
/// count, and the result builds as stacked bars, lines or a table with a legend, a readout you
/// can move with the arrow keys and a one-line insight that streams in under it. Enter asks (and
/// stops while busy), Escape stops. A question that matches no table offers suggestions.
#[component]
pub fn AskYourData(
    /// Host clock.
    now: ClockMs,
    /// Starting question.
    #[props(default = SUGGESTIONS[0].to_string())]
    question: String,
    /// Ask the starting question on mount.
    #[props(default = true)]
    auto_run: bool,
    /// Keep the SQL open after the query starts.
    #[props(default)]
    keep_sql_open: bool,
    /// The query fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Show suggestions in the empty state.
    #[props(default = true)]
    suggestions: bool,
) -> Element {
    let t0 = now.0;
    let q0 = question.clone();
    let mut ay = use_signal(move || Ask::new(&q0, auto_run, fail, keep_sql_open, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let a = ay.read().clone();
    let mut act = move |id: String| {
        let x = ay.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut keyed = move |part: &'static str, k: String| {
        let x = ay.write().key(part, &k, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = a.phase(n);
    let busy = a.busy(n);
    let d = a.data();
    let sql_open = a.sql_shown(n);
    let view = a.view();
    let built = a.built(n);
    let copied = a.copied.is_some_and(|c| n < c + 1_500);
    // Announce the chart once it lands.
    let summary = (ph == Phase::Done && built >= 1.0)
        .then(|| a.summary())
        .flatten();
    let sr = if said().is_empty() {
        summary.clone().unwrap_or_default()
    } else {
        said()
    };
    rsx! {
        section { class: "oxaik oxaik-ay", "aria-label": "Ask your data",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", if let Some(d) = &d { "{d.title}" } else { "Ask your data" } }
                    p { class: "oxaik-sub", if let Some(d) = &d { "{d.subtitle}" } else { "Plain questions, answered from {SOURCE.0}" } }
                }
                if ph == Phase::Done {
                    div { class: "oxaik-ay-view", {seg("View", CHARTS.iter().map(|x| x.to_string()).collect(), view, EventHandler::new(move |i| act(format!("ay-chart-{i}"))), EventHandler::new(move |k| keyed("view", k)))} }
                }
            }
            form { class: "oxaik-ay-field", onsubmit: move |e| { e.prevent_default(); act("ay-ask".into()) },
                span { class: "oxaik-ay-src", title: "{SOURCE.0} · {SOURCE.1}", {mark("postgresql", 15.0)} span { class: "oxaik-ay-srcn", "{SOURCE.0}" } }
                input { class: "oxaik-ay-in", r#type: "text", "aria-label": "Question", placeholder: "Ask a question about your data", value: "{a.q}",
                    oninput: move |e| ay.write().set_q(&e.value()),
                    onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if k == "Escape" { e.prevent_default(); keyed("field", k) } },
                }
                button { class: if busy { "oxaik-btn" } else { "oxaik-btn pri" }, r#type: "submit", "aria-label": if busy { "Stop" } else { "Ask" },
                    if busy { {ico("stop", 12.0)} "Stop" } else { {ico("up", 12.0)} "Ask" }
                }
            }
            if a.run.is_some() {
                div { class: "oxaik-ay-sql",
                    div { class: "oxaik-ay-sqlrow",
                        button { class: "oxaik-ay-sqlbtn", r#type: "button", "aria-expanded": "{sql_open}", "aria-controls": "oxaik-ay-code",
                            onclick: move |_| act("ay-sql".into()),
                            span { class: "oxaik-ay-st", "data-ph": "{ph:?}",
                                {match ph {
                                    Phase::Writing | Phase::Running => rsx! { {dots()} },
                                    Phase::Error => rsx! { {ico("alert", 13.0)} },
                                    Phase::Stopped => rsx! { {ico("stop", 12.0)} },
                                    _ => rsx! { {ico("check", 13.0)} },
                                }}
                            }
                            b { "Generated SQL" }
                            span { class: "oxaik-mute oxaik-num", "{a.meta(n)}" }
                            span { class: if sql_open { "oxaik-ay-chev open" } else { "oxaik-ay-chev" }, {ico("chevron", 12.0)} }
                        }
                        button { class: "oxaik-ib", r#type: "button", "aria-label": if copied { "Copied" } else { "Copy SQL" }, onclick: move |_| act("ay-copy".into()), {ico(if copied { "check" } else { "copy" }, 13.0)} }
                    }
                    if sql_open {
                        pre { id: "oxaik-ay-code", class: "oxaik-ay-code oxaik-scroll", tabindex: "0", "aria-label": "SQL",
                            code { {sql_tokens(&first_words_keep_lines(d.as_ref().map(|d| d.sql).unwrap_or(""), a.sql_words(n)))} }
                        }
                    }
                }
            }
            div { class: "oxaik-ay-stage", "aria-busy": "{busy}",
                {match ph {
                    Phase::Idle | Phase::NoMatch => rsx! {
                        div { class: "oxaik-ay-empty",
                            if ph == Phase::NoMatch {
                                p { class: "oxaik-row", {ico("info", 14.0)} "No table in {SOURCE.0} matches that question. Try one of these:" }
                            } else {
                                p { class: "oxaik-mute", "Ask about signups, revenue or churn in {SOURCE.0}." }
                            }
                            if suggestions || ph == Phase::NoMatch {
                                div { class: "oxaik-row",
                                    for (i, s) in SUGGESTIONS.iter().enumerate() {
                                        button { class: "oxaik-chip", r#type: "button", onclick: move |_| act(format!("ay-sug-{i}")), "{s}" }
                                    }
                                }
                            }
                        }
                    },
                    Phase::Writing => rsx! { p { class: "oxaik-row oxaik-ay-msg", {dots()} span { class: "oxaik-shim", "Writing SQL for your question" } } },
                    Phase::Running => rsx! { p { class: "oxaik-row oxaik-ay-msg", {dots()} span { "Running query · " } span { class: "oxaik-num", "{oxyn_core::aikit::thousands(a.scanned(n))} rows scanned" } } },
                    Phase::Stopped => rsx! {
                        div { class: "oxaik-ay-empty", p { class: "oxaik-row", {ico("stop", 12.0)} "Stopped. Nothing else ran." }
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ay-ask".into()), {ico("retry", 12.0)} "Ask again" } }
                    },
                    Phase::Error => rsx! {
                        div { class: "oxaik-err", role: "alert", {ico("alert", 16.0)} b { "The query timed out on {SOURCE.0}" }
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ay-retry".into()), {ico("retry", 12.0)} "Try again" } }
                    },
                    Phase::Done => {
                        let d = d.clone().unwrap_or_else(|| oxyn_core::aikit::ask::dataset(0));
                        rsx! { {chart(&d, view, built, a.cursor, EventHandler::new(move |k| keyed("chart", k)))}
                            p { class: "oxaik-ay-ins", "{first_words(d.insight, a.insight_words(n))}" } }
                    }
                }}
            }
            {live(&sr)}
        }
    }
}

/// The first `n` words of `text`, keeping its line breaks.
fn first_words_keep_lines(text: &str, n: usize) -> String {
    let mut left = n;
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        if left == 0 {
            break;
        }
        if i > 0 {
            out.push('\n');
        }
        let lead = line.len() - line.trim_start().len();
        out.push_str(&line[..lead]);
        let w: Vec<&str> = line.split_whitespace().take(left).collect();
        left -= w.len();
        out.push_str(&w.join(" "));
    }
    out
}

/// SQL tinted by token: keywords, strings, numbers, the rest.
fn sql_tokens(sql: &str) -> Element {
    const KW: [&str; 24] = [
        "SELECT",
        "FROM",
        "WHERE",
        "GROUP",
        "BY",
        "ORDER",
        "AS",
        "AND",
        "JOIN",
        "USING",
        "ON",
        "SUM",
        "COUNT",
        "ROUND",
        "MAX",
        "DATE_TRUNC",
        "INTERVAL",
        "CURRENT_DATE",
        "LIMIT",
        "OR",
        "IN",
        "NOT",
        "NULL",
        "DESC",
    ];
    let mut toks: Vec<(String, &'static str)> = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    for ch in sql.chars() {
        if in_str {
            cur.push(ch);
            if ch == '\'' {
                toks.push((std::mem::take(&mut cur), "s"));
                in_str = false;
            }
            continue;
        }
        if ch == '\'' {
            if !cur.is_empty() {
                toks.push((std::mem::take(&mut cur), ""));
            }
            cur.push(ch);
            in_str = true;
            continue;
        }
        if ch.is_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else {
            if !cur.is_empty() {
                toks.push((std::mem::take(&mut cur), ""));
            }
            toks.push((ch.to_string(), "p"));
        }
    }
    if !cur.is_empty() {
        toks.push((cur, if in_str { "s" } else { "" }));
    }
    let toks: Vec<(String, &'static str)> = toks
        .into_iter()
        .map(|(t, k)| {
            if !k.is_empty() {
                (t, k)
            } else if KW.contains(&t.to_uppercase().as_str()) {
                (t, "k")
            } else if t.chars().all(|c| c.is_ascii_digit()) {
                (t, "n")
            } else {
                (t, "i")
            }
        })
        .collect();
    rsx! {
        for (i, (t, k)) in toks.into_iter().enumerate() {
            span { key: "{i}", "data-t": k, "{t}" }
        }
    }
}

fn chart(
    d: &oxyn_core::aikit::ask::Dataset,
    view: usize,
    built: f64,
    cursor: Option<usize>,
    on_key: EventHandler<String>,
) -> Element {
    let max = (d.max() * 1.1).max(1.0);
    let ticks: Vec<f64> = (0..=3).map(|i| max * i as f64 / 3.0).collect();
    let nc = d.cats.len();
    let legend = rsx! {
        ul { class: "oxaik-legend",
            for (i, (s, h)) in d.series.iter().enumerate() {
                li { key: "{i}", "data-hue": "{h}", i { class: "oxaik-ay-sw" } "{s}"
                    if let Some(v) = d.legend_value(i) { b { class: "oxaik-num", "{v}" } }
                }
            }
        }
    };
    let readout = cursor
        .map(|c| d.readout(c))
        .unwrap_or_else(|| "Use ← → to read values".into());
    match view {
        2 => rsx! {
            div { class: "oxaik-ay-chart",
                {legend}
                div { class: "oxaik-ay-tw oxaik-scroll", tabindex: "0", "aria-label": "{d.title} table",
                    table { class: "oxaik-ay-tbl",
                        thead { tr { th { scope: "col", "{d.category}" } for (s, _) in d.series.iter() { th { scope: "col", "{s}" } } if d.stacked { th { scope: "col", "Total" } } } }
                        tbody {
                            for c in 0..nc {
                                tr { key: "{c}", th { scope: "row", "{d.cats[c]}" }
                                    for s in 0..d.series.len() { td { class: "oxaik-num", "{d.fmt_v(d.values[c][s])}" } }
                                    if d.stacked { td { class: "oxaik-num", b { "{d.fmt_v(d.total(c))}" } } }
                                }
                            }
                        }
                    }
                }
            }
        },
        _ => {
            let w = 600.0;
            let h = 180.0;
            let lines: Vec<(String, String)> = d
                .series
                .iter()
                .enumerate()
                .map(|(s, (_, hue))| {
                    let pts: Vec<String> = (0..nc)
                        .map(|c| {
                            let x = (c as f64 + 0.5) / nc as f64 * w;
                            let y = h - d.values[c][s] / max * h * built;
                            format!("{x:.1},{y:.1}")
                        })
                        .collect();
                    (pts.join(" "), hue.to_string())
                })
                .collect();
            rsx! {
                div { class: "oxaik-ay-chart",
                    {legend}
                    div { class: "oxaik-ay-plotw",
                        div { class: "oxaik-ay-y oxaik-num", "aria-hidden": "true",
                            for t in ticks.iter().rev() { span { "{d.fmt_v(*t)}" } }
                        }
                        div { class: "oxaik-ay-plot", tabindex: "0", role: "group", "aria-label": "{d.title}. {readout}",
                            onkeydown: move |e: KeyboardEvent| {
                                let k = key_of(&e);
                                if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End" | "Escape") { e.prevent_default(); on_key.call(k) }
                            },
                            for i in 0..4 { i { class: "oxaik-ay-grid", style: "bottom:{i * 100 / 3}%" } }
                            if view == 0 {
                                div { class: "oxaik-ay-bars",
                                    for c in 0..nc {
                                        div { key: "{c}", class: if cursor == Some(c) { "oxaik-ay-col on" } else { "oxaik-ay-col" },
                                            if d.stacked {
                                                div { class: "oxaik-ay-stack", style: "height:{d.total(c) / max * 100.0 * built}%",
                                                    for (s, (_, hue)) in d.series.iter().enumerate().rev() {
                                                        i { key: "{s}", "data-hue": "{hue}", style: "flex:{d.values[c][s].max(0.0)} 0 0" }
                                                    }
                                                }
                                            } else {
                                                div { class: "oxaik-ay-group",
                                                    for (s, (_, hue)) in d.series.iter().enumerate() {
                                                        i { key: "{s}", "data-hue": "{hue}", style: "height:{d.values[c][s] / max * 100.0 * built}%" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                svg { class: "oxaik-ay-svg", view_box: "0 0 600 180", preserve_aspect_ratio: "none", "aria-hidden": "true",
                                    for (i, (p, hue)) in lines.into_iter().enumerate() {
                                        polyline { key: "{i}", points: "{p}", "data-hue": "{hue}", "vector-effect": "non-scaling-stroke" }
                                    }
                                }
                                if let Some(c) = cursor { i { class: "oxaik-ay-cur", style: "left:{(c as f64 + 0.5) / nc as f64 * 100.0}%" } }
                            }
                        }
                    }
                    div { class: "oxaik-ay-x oxaik-mute", "aria-hidden": "true",
                        for (c, l) in d.cats.iter().enumerate() { span { key: "{c}", class: if c % 2 == 1 { "odd" } else { "" }, "{l}" } }
                    }
                    p { class: "oxaik-ay-read oxaik-mute oxaik-num", "aria-hidden": "true", "{readout}" }
                }
            }
        }
    }
}
