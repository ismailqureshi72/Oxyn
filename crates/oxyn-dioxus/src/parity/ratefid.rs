//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Rate Limit Planner**
//! ([`RateLimitPlanner`]). Behaviour and look only; the model is `oxyn_core::parity::ratefid`.
//! Styles: `ratefid.css` (classes `oxrl-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set.

use dioxus::prelude::*;
use oxyn_core::parity::ratefid::{
    RL_BASE, RL_DAY, RL_LAST, RL_LIMIT, RL_STEP, RlRun, RlState, rl_blame, rl_clock, rl_curve,
    rl_demo, rl_fit, rl_fmt_tokens, rl_new, rl_now, rl_over, rl_parse, rl_peak, rl_run_at,
    rl_spread, rl_state, rl_tokens,
};

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

fn pct(t: u32) -> f64 {
    t as f64 / RL_DAY as f64 * 100.0
}

/// Where a run's load has faded out (chart minutes).
fn run_end(r: &RlRun) -> u32 {
    let mut t = r.start + r.climb;
    while t < r.start + RL_DAY && rl_run_at(r, t) > 0.0 {
        t += RL_STEP;
    }
    t.min(RL_DAY)
}

/// Rate Limit Planner (affectUI parity): the day's token load against an API's per-minute limit,
/// one draggable row per planned run. Spread out lowers the peak; a run that tips the day over
/// the limit is named with the nearest start where it fits.
#[component]
pub fn RateLimitPlanner(
    /// The runs (the demo's when `None`).
    #[props(default)]
    runs: Option<Vec<RlRun>>,
    /// Limit, thousand tokens per minute.
    #[props(default = RL_LIMIT)]
    limit: f64,
    /// Safety line as a share of the limit (0.8 draws it at 96k of 120k).
    #[props(default = 0.8)]
    buffer: f64,
    /// Background load, thousand tokens per minute.
    #[props(default = RL_BASE)]
    base: f64,
    /// Chart minute of "now" (0 = 06:00); the demo's 10:30 when `None`.
    #[props(default)]
    now: Option<u32>,
    /// A run selected from the start (id).
    #[props(default)]
    selected: Option<String>,
    /// The edit dialog open from the start for this run (id).
    #[props(default)]
    editing: Option<String>,
    /// Heading.
    #[props(default = "Rate limit".to_string())]
    title: String,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// The runs changed (moved, spread, added, edited or removed).
    #[props(default)]
    on_runs_change: EventHandler<Vec<RlRun>>,
) -> Element {
    let mut rs = use_signal(move || runs.clone().unwrap_or_else(rl_demo));
    let mut undo = use_signal(|| Option::<Vec<RlRun>>::None);
    let mut sel = use_signal(move || selected.clone());
    let mut edit = use_signal(move || editing.clone());
    let mut drag = use_signal(|| Option::<(usize, f64, u32)>::None);
    let mut track_w = use_signal(|| 300.0f64);
    let mut next_id = use_signal(|| 1u32);
    let now = now.unwrap_or_else(rl_now);
    let list = rs();
    let curve = rl_curve(&list, base);
    let (pt, pv) = rl_peak(&list, base);
    let state = rl_state(pv, limit);
    let over = rl_over(&list, base, limit);
    let ymax = (limit * 1.18).max(pv * 1.06);
    const H: f64 = 110.0;
    let y = |v: f64| H - v / ymax * H;
    let xs = |t: u32| t as f64 / RL_DAY as f64 * 1000.0;
    let line: String = curve
        .iter()
        .enumerate()
        .map(|(i, (t, v))| {
            format!(
                "{}{:.1} {:.1}",
                if i == 0 { "M" } else { "L" },
                xs(*t),
                y(*v)
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let area = format!("{line} L1000 {H} L0 {H} Z");
    // The part over the limit: a tinted polygon per stretch and slanted hatch strokes (no
    // pattern or clip ids, so many planners on one page never share them).
    let ly0 = H - limit / ymax * H;
    let mut over_d = String::new();
    let mut hatch_d = String::new();
    let mut k = 0;
    while k < curve.len() {
        if curve[k].1 <= limit {
            k += 1;
            continue;
        }
        let a = k;
        while k < curve.len() && curve[k].1 > limit {
            k += 1;
        }
        let x0 = xs(curve[a]
            .0
            .saturating_sub(if a > 0 { RL_STEP / 2 } else { 0 }));
        let x1 = xs(curve[k - 1].0 + if k < curve.len() { RL_STEP / 2 } else { 0 });
        over_d.push_str(&format!("M{x0:.1} {ly0:.1}"));
        for p in &curve[a..k] {
            over_d.push_str(&format!(" L{:.1} {:.1}", xs(p.0), y(p.1)));
        }
        over_d.push_str(&format!(" L{x1:.1} {ly0:.1} Z "));
        for p in &curve[a..k] {
            let x = xs(p.0);
            hatch_d.push_str(&format!(
                "M{:.1} {:.1} L{:.1} {:.1} ",
                x - 2.5,
                ly0,
                x + 2.5,
                y(p.1)
            ));
        }
    }
    let ly = y(limit);
    let by = y(limit * buffer);
    let peak_pc = (pv / limit * 100.0).round() as i64;
    let status = match state {
        RlState::Under => format!("Under limit · peak {peak_pc}%"),
        RlState::Close => format!("Close to limit · peak {peak_pc}%"),
        RlState::Over => {
            let (a, b) = over.unwrap_or((pt, pt));
            format!("Over limit · {}–{}", rl_clock(a), rl_clock(b))
        }
    };
    let tokens: f64 = list.iter().map(rl_tokens).sum();
    let sub = format!(
        "{} runs · {} tokens · {}k / min",
        list.len(),
        rl_fmt_tokens(tokens),
        limit.round()
    );
    let sel_i = sel().and_then(|id| list.iter().position(|r| r.id == id));
    // The run to fix: the selected one if it adds to an overrun, else the biggest share of it.
    let culprit = over.and_then(|(a, _)| {
        let at = curve
            .iter()
            .filter(|p| p.0 >= a && p.1 > limit)
            .map(|p| p.0)
            .next()
            .unwrap_or(a);
        match sel_i {
            Some(i) if rl_run_at(&list[i], at) > 0.0 => Some((i, a)),
            _ => rl_blame(&list, at)
                .filter(|&i| !list[i].started)
                .map(|i| (i, a)),
        }
    });
    let fix = culprit.and_then(|(i, a)| rl_fit(&list, base, limit, i, now).map(|s| (i, a, s)));
    let foot = if let Some((i, a, s)) = fix {
        format!(
            "{} takes the day over the limit at {}. It fits from {}.",
            list[i].name,
            rl_clock(a),
            rl_clock(s)
        )
    } else if let Some((i, a)) = culprit {
        format!(
            "{} takes the day over the limit at {} and fits nowhere later today.",
            list[i].name,
            rl_clock(a)
        )
    } else if let Some(i) = sel_i {
        let r = &list[i];
        format!(
            "{} starts {}, peaks at {}k / min and fades out by {}.",
            r.name,
            rl_clock(r.start),
            r.peak,
            rl_clock(run_end(r))
        )
    } else {
        format!(
            "Busiest minute {} at {:.0}k – {}% of the limit.",
            rl_clock(pt),
            pv,
            peak_pc
        )
    };
    let ticks: Vec<u32> = (0..6).map(|k| k * 240).collect();
    // The now label steps aside when the peak chip sits over it.
    let now_clash = (pct(now) - pct(pt)).abs() < 16.0;
    let now_lab = format!("Now {}", rl_clock(now));
    let peak_lab = format!("Peak {:.0}k · {}", pv, rl_clock(pt));
    let lim_lab = format!("{:.0}k", limit);
    let buf_lab = format!("{:.0}k", limit * buffer);
    let grad = "oxrl-grad";
    let st = state.id();
    let mut set = move |v: Vec<RlRun>| {
        rs.set(v.clone());
        on_runs_change.call(v);
    };
    let can_undo = undo().is_some();

    rsx! {
        section { class: "oxrl", "data-style": style_attr(&style), "data-state": st, "data-drag": drag().is_some(), "aria-label": "{title}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { if edit().is_some() { edit.set(None); } else { sel.set(None); } },
            onpointermove: move |e: PointerEvent| {
                if let Some((i, x0, s0)) = drag() {
                    let dx = e.client_coordinates().x - x0;
                    let dm = dx / track_w().max(1.0) * RL_DAY as f64;
                    let s = ((s0 as f64 + dm) / RL_STEP as f64).round() as i64 * RL_STEP as i64;
                    let s = s.clamp(now.div_ceil(RL_STEP) as i64 * RL_STEP as i64, RL_LAST as i64) as u32;
                    if rs.read()[i].start != s { rs.with_mut(|v| v[i].start = s); }
                }
            },
            onpointerup: move |_| if drag().is_some() { drag.set(None); on_runs_change.call(rs()); },
            onpointerleave: move |_| if drag().is_some() { drag.set(None); on_runs_change.call(rs()); },
            header { class: "oxrl-bar",
                div { class: "oxrl-hd",
                    h2 { class: "oxrl-ttl", "{title}" }
                    span { class: "oxrl-sub", "{sub}" }
                }
                span { class: "oxrl-flex" }
                span { class: "oxrl-pill", "data-state": st, role: "status", "{status}" }
                if can_undo {
                    button { class: "oxrl-btn", r#type: "button", onclick: move |_| { if let Some(v) = undo() { set(v); undo.set(None); } }, "Undo" }
                }
                button { class: "oxrl-btn", r#type: "button", "aria-label": "Spread out: move runs apart so the busiest minute drops",
                    onclick: move |_| { let cur = rs(); let v = rl_spread(&cur, base, now); if v != cur { undo.set(Some(cur)); set(v); } },
                    {icon("M2.5 8h11M5 5.5L2.5 8 5 10.5M11 5.5l2.5 2.5-2.5 2.5", 13)}
                    "Spread out"
                }
            }
            div { class: "oxrl-plot",
            div { class: "oxrl-chart",
                svg { class: "oxrl-svg", view_box: "0 0 1000 {H}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    defs {
                        linearGradient { id: "{grad}", x1: "0", y1: "0", x2: "0", y2: "1",
                            stop { offset: "0", class: "oxrl-g0" }
                            stop { offset: "1", class: "oxrl-g1" }
                        }
                    }
                    for t in ticks.iter() { line { key: "{t}", "data-k": "{t}", class: "oxrl-gl", x1: "{xs(*t)}", x2: "{xs(*t)}", y1: "0", y2: "{H}", "vector-effect": "non-scaling-stroke" } }
                    line { class: "oxrl-buf", x1: "0", x2: "1000", y1: "{by}", y2: "{by}", "vector-effect": "non-scaling-stroke" }
                    path { class: "oxrl-area", d: "{area}", fill: "url(#{grad})" }
                    if !over_d.is_empty() {
                        path { class: "oxrl-overa", d: "{over_d}" }
                        path { class: "oxrl-hatch", d: "{hatch_d}", "vector-effect": "non-scaling-stroke" }
                    }
                    path { class: "oxrl-line", d: "{line}", "vector-effect": "non-scaling-stroke" }
                    line { class: "oxrl-lim", x1: "0", x2: "1000", y1: "{ly}", y2: "{ly}", "vector-effect": "non-scaling-stroke" }
                    line { class: "oxrl-nowl", x1: "{xs(now)}", x2: "{xs(now)}", y1: "0", y2: "{H}", "vector-effect": "non-scaling-stroke" }
                }
                span { class: "oxrl-ylab oxrl-limlab", style: "top:{ly / H * 100.0}%", "{lim_lab}" }
                span { class: "oxrl-ylab", style: "top:{by / H * 100.0}%", "{buf_lab}" }
                if !now_clash { span { class: "oxrl-now", style: "left:{pct(now)}%", "{now_lab}" } }
                span { class: "oxrl-peak", "data-state": st, style: "left:clamp(48px, {pct(pt)}%, calc(100% - 48px));top:{y(pv) / H * 100.0}%", "{peak_lab}" }
            }
            div { class: "oxrl-axis", "aria-hidden": "true",
                for t in ticks.iter() { span { key: "{t}", "data-k": "{t}", style: "left:{pct(*t)}%", "{rl_clock(*t)}" } }
            }
            }
            div { class: "oxrl-rows", role: "list", "aria-label": "Planned runs",
                for (i, r) in list.iter().enumerate() {
                    {
                        let id = r.id.clone();
                        let id_e = r.id.clone();
                        let on = sel_i == Some(i);
                        let end = run_end(r);
                        let late = fix.is_some_and(|f| f.0 == i) || culprit.is_some_and(|c| c.0 == i);
                        let when = if r.started { "Running".to_string() } else { rl_clock(r.start) };
                        let val = rl_clock(r.start);
                        let peak_s = format!("{}k", r.peak);
                        let w = pct(end - r.start);
                        let editing_this = edit().as_deref() == Some(r.id.as_str());
                        let at_val = rl_clock(r.start);
                        let rk = r.id.clone();
                        rsx! {
                            div { class: "oxrl-row", role: "listitem", key: "{rk}", "data-k": "{rk}", "data-sel": on, "data-tone": "{r.tone}", "data-started": r.started,
                                button { class: "oxrl-name", r#type: "button", "aria-pressed": on,
                                    onclick: move |_| { let v = if sel().as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }; sel.set(v); },
                                    span { class: "oxrl-gly", i {} i {} }
                                    span { class: "oxrl-nm", "{r.name}" }
                                }
                                span { class: "oxrl-when", "data-late": late, "{when}" }
                                div { class: "oxrl-track",
                                    onmounted: move |e: MountedEvent| async move { if let Ok(rc) = e.get_client_rect().await { track_w.set(rc.size.width.max(1.0)); } },
                                    div { class: "oxrl-run", role: "slider", tabindex: if r.started { "-1" } else { "0" },
                                        "aria-label": "{r.name} start", "aria-valuemin": "0", "aria-valuemax": "{RL_LAST}", "aria-valuenow": "{r.start}", "aria-valuetext": "{val}",
                                        "aria-disabled": r.started,
                                        style: "left:{pct(r.start)}%;width:{w}%",
                                        onpointerdown: move |e: PointerEvent| {
                                            let cur = rs.read()[i].clone();
                                            if !cur.started { e.prevent_default(); sel.set(Some(cur.id.clone())); drag.set(Some((i, e.client_coordinates().x, cur.start))); }
                                        },
                                        onkeydown: move |e: KeyboardEvent| {
                                            let cur = rs.read()[i].clone();
                                            if cur.started { return; }
                                            let step = if e.modifiers().shift() { 30 } else { RL_STEP } as i64;
                                            let d = match e.key() { Key::ArrowRight | Key::ArrowUp => step, Key::ArrowLeft | Key::ArrowDown => -step, _ => 0 };
                                            if d != 0 {
                                                e.prevent_default();
                                                let lo = now.div_ceil(RL_STEP) * RL_STEP;
                                                let s = (cur.start as i64 + d).clamp(lo as i64, RL_LAST as i64) as u32;
                                                let mut v = rs(); v[i].start = s; set(v);
                                            }
                                        },
                                        i { class: "oxrl-fill" }
                                    }
                                    span { class: "oxrl-pk", style: "left:calc({pct(end)}% + 4px)", "{peak_s}" }
                                }
                                div { class: "oxrl-acts",
                                    button { class: "oxrl-ic", r#type: "button", "aria-label": "Remove {r.name}",
                                        onclick: move |_| { let mut v = rs(); v.remove(i); sel.set(None); edit.set(None); set(v); },
                                        {icon("M4.5 4.5l7 7M11.5 4.5l-7 7", 12)}
                                    }
                                    button { class: "oxrl-ic", r#type: "button", "aria-label": "Edit {r.name}", "aria-expanded": editing_this,
                                        onclick: move |_| { let v = if edit().as_deref() == Some(id_e.as_str()) { None } else { Some(id_e.clone()) }; edit.set(v); },
                                        {icon("M4 8h.01M8 8h.01M12 8h.01", 14)}
                                    }
                                }
                                if editing_this {
                                    div { class: "oxrl-dlg", role: "dialog", "aria-label": "Edit {r.name}",
                                        b { class: "oxrl-dt", "Edit {r.name}" }
                                        label { class: "oxrl-f", span { "Name" }
                                            input { r#type: "text", value: "{r.name}", autofocus: true,
                                                oninput: move |e| { let mut v = rs(); v[i].name = e.value(); set(v); } }
                                        }
                                        label { class: "oxrl-f", span { "Starts" }
                                            input { r#type: "time", value: "{at_val}", step: "300",
                                                oninput: move |e| { if let Some(m) = rl_parse(&e.value()) { let mut v = rs(); v[i].start = m.min(RL_LAST); set(v); } } }
                                        }
                                        label { class: "oxrl-f", span { "Peak, k / min" }
                                            input { r#type: "number", min: "1", max: "999", value: "{r.peak}",
                                                oninput: move |e| { if let Ok(n) = e.value().parse::<u32>() { let mut v = rs(); v[i].peak = n.clamp(1, 999); set(v); } } }
                                        }
                                        label { class: "oxrl-f", span { "Climbs for, min" }
                                            input { r#type: "number", min: "1", max: "240", value: "{r.climb}",
                                                oninput: move |e| { if let Ok(n) = e.value().parse::<u32>() { let mut v = rs(); v[i].climb = n.clamp(1, 240); set(v); } } }
                                        }
                                        label { class: "oxrl-f", span { "Halves every, min" }
                                            input { r#type: "number", min: "5", max: "240", value: "{r.half}",
                                                oninput: move |e| { if let Ok(n) = e.value().parse::<u32>() { let mut v = rs(); v[i].half = n.clamp(5, 240); set(v); } } }
                                        }
                                        button { class: "oxrl-done", r#type: "button", onclick: move |_| edit.set(None), "Done" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button { class: "oxrl-add", r#type: "button",
                onclick: move |_| {
                    let n = next_id(); next_id.set(n + 1);
                    let mut v = rs();
                    let mut r = rl_new(&format!("new-{n}"), now.div_ceil(RL_STEP) * RL_STEP);
                    r.tone = (v.len() % 4) as u8;
                    let id = r.id.clone();
                    v.push(r); set(v);
                    sel.set(Some(id.clone())); edit.set(Some(id));
                },
                {icon("M8 3.5v9M3.5 8h9", 12)} "Add run"
            }
            footer { class: "oxrl-foot", "data-state": st,
                span { class: "oxrl-ins", "{foot}" }
                if let Some((i, _, s)) = fix {
                    button { class: "oxrl-move", r#type: "button",
                        onclick: move |_| { let cur = rs(); let mut v = cur.clone(); v[i].start = s; undo.set(Some(cur)); set(v); },
                        "Move to {rl_clock(s)}"
                    }
                }
            }
        }
    }
}
