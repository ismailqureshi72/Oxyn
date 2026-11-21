//! Home and run surfaces from the X-post references (plan #28, chunk 6): `ThreadList` (T3 Code,
//! Threadbear), `ActivityHeatmap` and `ActivityHome` (affectUI-style overview by @flohoeller,
//! Threadbear's Librarian home), `TokenUsageChart` (@pitus_), `AttachmentStrip` (@sitenley for
//! componest), `RunModeMenu` and `ExperimentGrid` (Cloudy), `DocGrid` (quiet.md) and
//! `PollingCost` (@TekonBurns). Behaviour-only rebuilds; models in `oxyn_core::kit::home`.
//! Every class is prefixed `oxh-`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::home::{
        BUDGETS, Doc, DocSort, Experiment, Outcome, RunMode, Thread, ThreadState, UsageDay,
        budget_line, compact, day_label, delta, doc_view, experiment_counts, experiment_grid,
        file_icon, group_threads, heat_recent, heat_tip, heat_year, hm, hover_index, initials,
        month_cols, needs_you, parse_budget, poll_cost, spark_path, tile_name, tile_upload,
        usage_totals,
    },
};

use crate::composer::icons::Icon;

/// Characters keyed by place from the end, so only changed places remount and roll.
#[component]
fn Roll(text: String) -> Element {
    let n = text.chars().count();
    rsx! {
        span { class: "oxh-roll",
            for (i, c) in text.chars().enumerate() {
                span { key: "{n - i}-{c}", class: if c.is_ascii_digit() { "oxh-d" } else { "" }, "{c}" }
            }
        }
    }
}

/// A shape mark (circle, square, square-o, triangle, diamond) drawn in 10 px.
#[component]
fn Shape(shape: String, #[props(default = 10.0)] size: f32) -> Element {
    rsx! {
        svg { class: "oxh-shape", "data-shape": "{shape}", width: "{size}", height: "{size}", view_box: "0 0 10 10", "aria-hidden": "true",
            match shape.as_str() {
                "circle" => rsx! { circle { cx: "5", cy: "5", r: "4.2" } },
                "triangle" => rsx! { path { d: "M5 .8 9.4 9H.6z" } },
                "diamond" => rsx! { path { d: "M5 .4 9.6 5 5 9.6.4 5z" } },
                "square-o" => rsx! { rect { x: "1.2", y: "1.2", width: "7.6", height: "7.6", rx: "1.5", class: "o" } },
                _ => rsx! { rect { x: ".8", y: ".8", width: "8.4", height: "8.4", rx: "1.6" } },
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Thread list
// ---------------------------------------------------------------------------------------------

/// Thread list (T3 Code / Threadbear behaviour): threads grouped by where they stand. What needs
/// you rises to the top with an Approval or Input chip, unseen results carry Done, the rest show
/// their age; Working and Settled sit in collapsible groups with counts (Settled starts closed).
/// The list is one tab stop: ↑ / ↓ move, Enter opens, ← / → close or open a group, and typing in
/// the search narrows every group.
#[component]
pub fn ThreadList(
    /// Threads.
    threads: Vec<Thread>,
    /// Open thread id.
    #[props(default)]
    selected: Option<String>,
    /// Show the search row.
    #[props(default = true)]
    search: bool,
    /// Start with the Settled group open.
    #[props(default)]
    settled_open: bool,
    /// A thread was opened.
    #[props(default)]
    on_select: EventHandler<String>,
    /// The new-thread button.
    #[props(default)]
    on_new: EventHandler<()>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut closed = use_signal(move || {
        if settled_open {
            vec![]
        } else {
            vec!["settled"]
        }
    });
    let mut cursor = use_signal(|| 0usize);
    let mut sel = use_signal(|| selected.clone());
    let groups = group_threads(&threads, &query.read());
    // Flattened visible rows: (thread index or usize::MAX for a group header, group id).
    let mut flat: Vec<(usize, &'static str)> = Vec::new();
    for g in &groups {
        if !g.label.is_empty() {
            flat.push((usize::MAX, g.id));
        }
        if !closed.read().contains(&g.id) {
            for &r in &g.rows {
                flat.push((r, g.id));
            }
        }
    }
    let n = flat.len();
    let cur = cursor().min(n.saturating_sub(1));
    let ids: Vec<String> = threads.iter().map(|t| t.id.clone()).collect();
    let active_id = flat.get(cur).map(|(r, g)| {
        if *r == usize::MAX {
            format!("oxh-g-{g}")
        } else {
            format!("oxh-t-{}", ids[*r])
        }
    });
    let flat_k = flat.clone();
    let ids_k = ids.clone();
    let need = needs_you(&threads);
    rsx! {
        div { class: "oxh-threads",
            if search {
                div { class: "oxh-tsearch",
                    span { class: "oxh-tsicon", Icon { name: "search", size: 13.0 } }
                    input { r#type: "search", placeholder: "Search", "aria-label": "Search threads", value: "{query}",
                        oninput: move |e| { query.set(e.value()); cursor.set(0); }
                    }
                    button { class: "oxh-ib", r#type: "button", "aria-label": "New thread", onclick: move |_| on_new.call(()), Icon { name: "pencil", size: 13.0 } }
                }
            }
            if let Some(nd) = need { div { class: "oxh-need", role: "status", "{nd}" } }
            div { class: "oxh-tlist", role: "listbox", tabindex: "0", "aria-label": "Threads",
                "aria-activedescendant": active_id.clone().unwrap_or_default(),
                onkeydown: move |e: KeyboardEvent| {
                    let Some(&(r, g)) = flat_k.get(cur) else { return };
                    match e.key() {
                        Key::ArrowDown => { e.prevent_default(); cursor.set((cur + 1).min(n.saturating_sub(1))); }
                        Key::ArrowUp => { e.prevent_default(); cursor.set(cur.saturating_sub(1)); }
                        Key::Home => cursor.set(0),
                        Key::End => cursor.set(n.saturating_sub(1)),
                        Key::ArrowLeft if r == usize::MAX => { if !closed.read().contains(&g) { closed.write().push(g); } }
                        Key::ArrowRight if r == usize::MAX => closed.write().retain(|x| *x != g),
                        Key::Enter if r == usize::MAX => {
                            e.prevent_default();
                            let c = closed.read().contains(&g);
                            if c { closed.write().retain(|x| *x != g) } else { closed.write().push(g) }
                        }
                        Key::Enter => { let id = ids_k[r].clone(); sel.set(Some(id.clone())); on_select.call(id); }
                        _ => {}
                    }
                },
                if groups.is_empty() { div { class: "oxh-tempty", "No threads match" } }
                for g in groups {
                    {
                        let gid = g.id;
                        let is_closed = closed.read().contains(&gid);
                        let count = g.rows.len();
                        rsx! {
                            div { key: "{gid}", class: "oxh-tgroup", "data-group": "{gid}",
                                if !g.label.is_empty() {
                                    {
                                        let hi = flat.iter().position(|x| x.0 == usize::MAX && x.1 == gid);
                                        rsx! {
                                            div { id: "oxh-g-{gid}", class: "oxh-thead", role: "option", "aria-selected": "false",
                                                "aria-expanded": "{!is_closed}", "data-active": "{hi == Some(cur)}",
                                                onclick: move |_| {
                                                    if let Some(h) = hi { cursor.set(h); }
                                                    let c = closed.read().contains(&gid);
                                                    if c { closed.write().retain(|x| *x != gid) } else { closed.write().push(gid) }
                                                },
                                                span { "{g.label}" }
                                                span { class: "oxh-tcount", "{count}" }
                                                span { class: if is_closed { "oxh-chev" } else { "oxh-chev open" }, Icon { name: "chevron", size: 12.0 } }
                                            }
                                        }
                                    }
                                }
                                if !is_closed {
                                    for r in g.rows.clone() {
                                        {
                                            let t = threads[r].clone();
                                            let id = t.id.clone();
                                            let fi = flat.iter().position(|x| x.0 == r);
                                            let chip = t.state.chip();
                                            let st = t.state.id();
                                            let on = sel.read().as_deref() == Some(t.id.as_str());
                                            rsx! {
                                                div { key: "{id}", id: "oxh-t-{id}", class: "oxh-trow", role: "option", "aria-selected": "{on}",
                                                    "data-active": "{fi == Some(cur)}", "data-state": "{st}",
                                                    onclick: move |_| { if let Some(f) = fi { cursor.set(f); } sel.set(Some(id.clone())); on_select.call(id.clone()); },
                                                    div { class: "oxh-tl1",
                                                        span { class: "oxh-badge", "aria-hidden": "true", "{initials(&t.project)}" }
                                                        span { class: "oxh-tproj", "{t.project}" }
                                                        if let Some(c) = chip {
                                                            span { key: "{c}", class: "oxh-chip", "data-state": "{st}",
                                                                if matches!(t.state, ThreadState::Working(_)) { span { class: "oxh-spin", "aria-hidden": "true" } } else { Icon { name: t.state.icon(), size: 12.0 } }
                                                                "{c}"
                                                            }
                                                        } else {
                                                            span { class: "oxh-age", "{t.meta()}" }
                                                        }
                                                    }
                                                    div { class: "oxh-tl2",
                                                        span { class: "oxh-ttitle", "{t.title}" }
                                                        span { class: "oxh-agent", "aria-hidden": "true", Icon { name: t.agent.clone(), size: 13.0 } }
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
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Activity heat map
// ---------------------------------------------------------------------------------------------

/// Activity heat map (Threadbear / affectUI overview): a year of days as week columns, five
/// levels split at the quartiles of the busy days. Hover or arrow keys show "5 chats · Mon,
/// Oct 5"; the year steps with ‹ › and the cells fade in again. `weeks` gives the compact strip
/// of the last weeks instead (overview card).
#[component]
pub fn ActivityHeatmap(
    /// `(year, counts per day of the year)`, oldest first.
    years: Vec<(i32, Vec<u32>)>,
    /// Card title ("Threadbear activity").
    #[props(default)]
    title: String,
    /// Singular noun for counts ("thread").
    #[props(default = "chat".to_string())]
    noun: String,
    /// Verb after the total ("started").
    #[props(default = "started".to_string())]
    verb: String,
    /// Footnote under the grid.
    #[props(default)]
    footnote: Option<String>,
    /// Compact strip of the last `weeks` weeks (no labels, no year nav).
    #[props(default)]
    weeks: Option<u32>,
    /// Cell size in px.
    #[props(default = 11.0)]
    cell: f32,
) -> Element {
    let last = years.len().saturating_sub(1);
    let mut yi = use_signal(move || last);
    let mut hover = use_signal(|| None::<u32>);
    let i = yi().min(last);
    let Some((year, counts)) = years.get(i).cloned() else {
        return rsx! {};
    };
    let gap = 3.0;
    let step = cell + gap;
    if let Some(w) = weeks {
        let cells = heat_recent(&counts, w);
        let total: u32 = counts.iter().rev().take((w * 7) as usize).sum();
        return rsx! {
            div { class: "oxh-heat compact", role: "img", "aria-label": "{total} {noun}s in the last {w} weeks",
                svg { class: "oxh-hsvg", width: "{w as f32 * step}", height: "{7.0 * step}", view_box: "0 0 {w as f32 * step} {7.0 * step}",
                    for (c, r, l) in cells.into_iter() {
                        rect { class: "oxh-cell", "data-l": "{l}", x: "{c as f32 * step}", y: "{r as f32 * step}", width: "{cell}", height: "{cell}", rx: "2.5",
                            style: "animation-delay:{c * 18}ms" }
                    }
                }
            }
        };
    }
    let cells = heat_year(year, &counts);
    let total: u32 = cells.iter().map(|c| c.count).sum();
    let cols = cells.last().map_or(53, |c| c.col + 1);
    let lab_w = 30.0;
    let w = lab_w + cols as f32 * step;
    let h = 16.0 + 7.0 * step;
    let tip = hover().and_then(|d| cells.get(d as usize).copied());
    let ndays = cells.len() as u32;
    rsx! {
        div { class: "oxh-heat", style: "--cell:{cell}px",
            div { class: "oxh-hhead",
                div {
                    div { class: "oxh-htitle", "{title}" }
                    div { class: "oxh-hsub", key: "{year}", Roll { text: format!("{total}") } " {noun}s {verb} in {year}" }
                }
                if years.len() > 1 {
                    div { class: "oxh-ynav",
                        button { class: "oxh-ib", r#type: "button", "aria-label": "Previous year", disabled: i == 0, onclick: move |_| { yi.set(i.saturating_sub(1)); hover.set(None); }, span { class: "oxh-rot", Icon { name: "chevron", size: 12.0 } } }
                        span { class: "oxh-year", key: "{year}", "{year}" }
                        button { class: "oxh-ib", r#type: "button", "aria-label": "Next year", disabled: i >= last, onclick: move |_| { yi.set((i + 1).min(last)); hover.set(None); }, span { class: "oxh-rot r", Icon { name: "chevron", size: 12.0 } } }
                    }
                }
            }
            div { class: "oxh-hwrap",
                svg { key: "{year}", class: "oxh-hsvg", width: "{w}", height: "{h}", view_box: "0 0 {w} {h}", role: "grid", tabindex: "0",
                    "aria-label": "{title} {year}: {total} {noun}s. Arrow keys move between days.",
                    onkeydown: move |e: KeyboardEvent| {
                        let d = hover().unwrap_or(0) as i64;
                        let nd = match e.key() {
                            Key::ArrowRight => d + 7,
                            Key::ArrowLeft => d - 7,
                            Key::ArrowDown => d + 1,
                            Key::ArrowUp => d - 1,
                            Key::Home => 0,
                            Key::End => ndays as i64 - 1,
                            Key::Escape => { hover.set(None); return; }
                            _ => return,
                        };
                        e.prevent_default();
                        hover.set(Some(nd.clamp(0, ndays as i64 - 1) as u32));
                    },
                    onmouseleave: move |_| hover.set(None),
                    for (c, m) in month_cols(year) {
                        text { key: "m{m}", class: "oxh-mlab", x: "{lab_w + c as f32 * step}", y: "10", "{m}" }
                    }
                    for (r, l) in [(1u32, "Mon"), (3, "Wed"), (5, "Fri")] {
                        text { key: "d{l}", class: "oxh-mlab", x: "0", y: "{16.0 + r as f32 * step + cell - 2.0}", "{l}" }
                    }
                    for c in cells.iter().copied() {
                        rect { key: "{c.doy}", class: "oxh-cell", "data-l": "{c.level}", "data-on": "{tip.map(|t| t.doy) == Some(c.doy)}",
                            x: "{lab_w + c.col as f32 * step}", y: "{16.0 + c.row as f32 * step}", width: "{cell}", height: "{cell}", rx: "2.5",
                            style: "animation-delay:{c.col * 9}ms",
                            onmouseenter: move |_| hover.set(Some(c.doy)),
                        }
                    }
                }
                if let Some(t) = tip {
                    div { class: "oxh-tip", role: "status",
                        style: "left:{lab_w + t.col as f32 * step + cell / 2.0}px;top:{16.0 + t.row as f32 * step - 6.0}px",
                        "{heat_tip(t.count, &noun, year, t.doy)}"
                    }
                }
            }
            div { class: "oxh-hfoot",
                if let Some(f) = footnote { span { "{f}" } }
                span { class: "oxh-legend", "aria-hidden": "true",
                    "Less"
                    for l in 0..5 { span { class: "oxh-sw", "data-l": "{l}" } }
                    "More"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Activity home
// ---------------------------------------------------------------------------------------------

const QUOTES: [(&str, &str); 4] = [
    (
        "Well begun is half done.",
        "Aristotle, philosopher (attributed)",
    ),
    (
        "It is not the mountain we conquer, but ourselves.",
        "Edmund Hillary, mountaineer",
    ),
    ("Nothing will work unless you do.", "Maya Angelou, writer"),
    (
        "Simple can be harder than complex.",
        "Steve Jobs, entrepreneur",
    ),
];

/// Activity home (affectUI-style overview / Threadbear Librarian): the composer slot over a
/// grid of cards. `overview` shows Activity (a 12-week strip with totals), Recent chats, Usage
/// against the plan, Files made for you and This month with rolling deltas. `librarian` shows a
/// background agent's status with Pause and Look at everything, a quote that refreshes, and the
/// year heat map. Cards stagger in once; reduced motion shows them at once.
#[component]
pub fn ActivityHome(
    /// `overview` or `librarian`.
    #[props(default = "overview".to_string())]
    variant: String,
    /// Greeting line ("Good night, Wayne"); empty hides it.
    #[props(default)]
    greeting: String,
    /// Date line under the greeting.
    #[props(default)]
    date: String,
    /// Credits used and the allowance.
    #[props(default = (1284, 2000))]
    credits: (u32, u32),
    /// Activity counts per day (oldest first; the last day is today).
    #[props(default)]
    counts: Vec<u32>,
    /// Year for the Librarian heat map.
    #[props(default = 2026)]
    year: i32,
    /// Clock (Librarian look timing).
    #[props(default)]
    now: ClockMs,
    /// Composer or other content above the cards.
    #[props(default)]
    children: Element,
    /// A recent chat or file was opened.
    #[props(default)]
    on_open: EventHandler<String>,
) -> Element {
    let mut paused = use_signal(|| false);
    let mut look_at = use_signal(|| None::<ClockMs>);
    let mut quote = use_signal(|| 0usize);
    let (used, of) = credits;
    let frac = (used as f32 / of.max(1) as f32).min(1.0);
    if variant == "librarian" {
        let looking = look_at().is_some_and(|t| now.0.saturating_sub(t.0) < 2400);
        let status = if paused() {
            "Paused"
        } else if looking {
            "Looking at everything…"
        } else {
            "Idle"
        };
        let (q, who) = QUOTES[quote() % QUOTES.len()];
        let looks = 137
            + if look_at().is_some() && !looking {
                12
            } else {
                0
            };
        return rsx! {
            div { class: "oxh-home lib",
                if !greeting.is_empty() {
                    div { class: "oxh-greet", h2 { "{greeting}" } if !date.is_empty() { p { "{date}" } } }
                }
                {children}
                section { class: "oxh-card oxh-libcard", style: "--i:0",
                    div { class: "oxh-ctitle", "Librarian" }
                    p { class: "oxh-cdim", "Keeps your projects tidy as your threads go." }
                    div { class: "oxh-status", "data-s": "{status.split(' ').next().unwrap_or_default().to_lowercase()}", role: "status",
                        span { class: "oxh-sdot" } span { key: "{status}", class: "oxh-swap", "{status}" }
                    }
                    p { class: "oxh-cdim", Roll { text: format!("{looks}") } " looks today · 174 skipped · 695k tokens" }
                    div { class: "oxh-row",
                        button { class: "oxh-btn", r#type: "button", "aria-pressed": "{paused()}", onclick: move |_| paused.toggle(), if paused() { "Resume" } else { "Pause" } }
                        button { class: "oxh-btn", r#type: "button", disabled: paused() || looking, onclick: move |_| look_at.set(Some(now)), "Look at everything" }
                    }
                }
                section { class: "oxh-card oxh-quote", style: "--i:1",
                    blockquote { key: "{q}", class: "oxh-swap", p { "“{q}”" } footer { "{who}" } }
                    button { class: "oxh-ib", r#type: "button", "aria-label": "Another quote", onclick: move |_| quote += 1, Icon { name: "retry", size: 13.0 } }
                }
                section { class: "oxh-card", style: "--i:2",
                    ActivityHeatmap { years: vec![(year - 1, oxyn_core::kit::home::demo_counts(7, 365, 200)), (year, counts.clone())], title: "Threadbear activity", noun: "thread", footnote: Some("Threads in a private folder aren't counted".to_string()) }
                }
            }
        };
    }
    let recent = [
        ("Q3 launch messaging", "2 hours ago"),
        ("Ad hooks for the linen overshirt", "Yesterday"),
        ("Logo usage on partner sites", "3 days ago"),
    ];
    let files = [
        ("q3-launch-plan.md", "1 hour ago"),
        ("budget-by-channel.csv", "Yesterday"),
        ("competitor-pricing.csv", "Tue"),
    ];
    let pct = (frac * 100.0).round();
    let of_txt = oxyn_core::kit::home::grouped(u64::from(of));
    rsx! {
        div { class: "oxh-home",
            if !greeting.is_empty() {
                div { class: "oxh-greet", h2 { "{greeting}" } if !date.is_empty() { p { "{date}" } } }
            }
            {children}
            div { class: "oxh-grid",
                section { class: "oxh-card wide", style: "--i:0",
                    div { class: "oxh-ch", Icon { name: "clock", size: 13.0 } span { "Activity" } }
                    ActivityHeatmap { years: vec![(year, counts.clone())], weeks: Some(24), cell: 9.0 }
                    div { class: "oxh-cfoot", span { "872 chats · 214 files · 38 automations" } span { "Last 24 weeks" } }
                }
                section { class: "oxh-card", style: "--i:1",
                    div { class: "oxh-ch", Icon { name: "thread", size: 13.0 } span { "Recent chats" } span { class: "oxh-more", "6 more" } }
                    ul { class: "oxh-list",
                        for (t, w) in recent {
                            li { button { r#type: "button", onclick: move |_| on_open.call(t.to_string()), span { class: "oxh-lt", "{t}" } span { class: "oxh-lw", "{w}" } } }
                        }
                    }
                }
                section { class: "oxh-card", style: "--i:2",
                    div { class: "oxh-ch", Icon { name: "spark", size: 13.0 } span { "Usage" } }
                    div { class: "oxh-big", Roll { text: oxyn_core::kit::home::grouped(u64::from(used)) } span { class: "oxh-of", " / {of_txt} credits" } }
                    div { class: "oxh-bar", role: "meter", "aria-valuemin": "0", "aria-valuemax": "{of}", "aria-valuenow": "{used}", "aria-label": "Credits used, {pct}%",
                        span { style: "width:{frac * 100.0}%", "data-warn": "{frac >= 0.9}" }
                    }
                    div { class: "oxh-cdim", "Resets Dec 1" }
                }
                section { class: "oxh-card", style: "--i:3",
                    div { class: "oxh-ch", Icon { name: "file", size: 13.0 } span { "Files made for you" } span { class: "oxh-more", "11 more" } }
                    ul { class: "oxh-list",
                        for (f, w) in files {
                            li { button { r#type: "button", onclick: move |_| on_open.call(f.to_string()), span { class: "oxh-fi", Icon { name: file_icon(f), size: 12.0 } } span { class: "oxh-lt", "{f}" } span { class: "oxh-lw", "{w}" } } }
                        }
                    }
                }
                section { class: "oxh-card", style: "--i:4",
                    div { class: "oxh-ch", Icon { name: "table", size: 13.0 } span { "This month" } }
                    div { class: "oxh-stats",
                        for (l, v, d) in [("Chats", 36i64, 12i64), ("Files made", 14, -3)] {
                            div { class: "oxh-stat",
                                span { class: "oxh-cdim", "{l}" }
                                span { class: "oxh-sv", Roll { text: format!("{v}") } span { class: "oxh-delta", "data-up": "{d > 0}", "{delta(d)}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Token usage chart
// ---------------------------------------------------------------------------------------------

/// Token usage chart (@pitus_ behaviour): daily tokens as an area with the cached share as a
/// line. Hovering or arrowing through the plot shows the day with Tokens and Cached; the range
/// buttons re-window the plot and the totals (Total usage, Avg daily, Est. cost) roll to the new
/// numbers. Uncached tokens cost `per_m_cents` a million, cached a tenth.
#[component]
pub fn TokenUsageChart(
    /// Days, oldest first; the last is today.
    days: Vec<UsageDay>,
    /// Year and day of the year of the first day (for the date labels).
    #[props(default = (2026, 177))]
    start: (i32, u32),
    /// Price of a million uncached tokens, cents.
    #[props(default = 300.0)]
    per_m_cents: f64,
    /// Title.
    #[props(default = "Token usage".to_string())]
    title: String,
    /// Range options in days (0 = all).
    #[props(default = vec![30, 90, 0])]
    ranges: Vec<usize>,
) -> Element {
    let mut range = use_signal(|| 0usize);
    let mut hover = use_signal(|| None::<usize>);
    let r = ranges.get(range()).copied().unwrap_or(0);
    let n = days.len();
    let from = if r == 0 { 0 } else { n.saturating_sub(r) };
    let win: Vec<UsageDay> = days[from..].to_vec();
    let tot = usage_totals(&win, per_m_cents);
    let max = win.iter().map(|d| d.tokens).fold(0.0, f64::max) * 1.08;
    let (w, h) = (320.0f32, 96.0f32);
    let toks: Vec<f64> = win.iter().map(|d| d.tokens).collect();
    let cached: Vec<f64> = win.iter().map(|d| d.cached).collect();
    let area = spark_path(&toks, max, w, h, true);
    let line = spark_path(&toks, max, w, h, false);
    let cline = spark_path(&cached, max, w, h, false);
    let date = |i: usize| {
        let (y, d0) = start;
        day_label(y, d0 + (from + i) as u32)
    };
    let first = date(0);
    let last = date(win.len().saturating_sub(1));
    let hv = hover().filter(|&i| i < win.len());
    let m = win.len();
    rsx! {
        div { class: "oxh-usage",
            div { class: "oxh-uhead",
                div {
                    div { class: "oxh-ctitle", "{title}" }
                    div { class: "oxh-cdim", key: "{first}", "See your token usage from {first} – {last}." }
                }
                div { class: "oxh-seg", role: "group", "aria-label": "Range",
                    for (k, rr) in ranges.iter().copied().enumerate() {
                        button { r#type: "button", "aria-pressed": "{k == range()}", onclick: move |_| { range.set(k); hover.set(None); },
                            if rr == 0 { "All" } else { "{rr}d" }
                        }
                    }
                }
            }
            div { class: "oxh-plot", tabindex: "0", role: "img",
                "aria-label": "{title}: {tot.total} tokens, {tot.avg} a day, about {tot.cost}. Arrow keys read each day.",
                onkeydown: move |e: KeyboardEvent| {
                    let i = hover().unwrap_or(m.saturating_sub(1));
                    match e.key() {
                        Key::ArrowLeft => hover.set(Some(i.saturating_sub(1))),
                        Key::ArrowRight => hover.set(Some((i + 1).min(m.saturating_sub(1)))),
                        Key::Escape => hover.set(None),
                        _ => return,
                    }
                    e.prevent_default();
                },
                onmouseleave: move |_| hover.set(None),
                svg { key: "{r}", class: "oxh-usvg", view_box: "0 0 {w} {h}", "preserveAspectRatio": "none", "aria-hidden": "true",
                    path { class: "oxh-uarea", d: "{area}" }
                    path { class: "oxh-uline", d: "{line}", fill: "none", "vector-effect": "non-scaling-stroke" }
                    path { class: "oxh-ucache", d: "{cline}", fill: "none", "vector-effect": "non-scaling-stroke" }
                    if let Some(i) = hv {
                        line { class: "oxh-urule", x1: "{i as f32 * w / (m.max(2) - 1) as f32}", x2: "{i as f32 * w / (m.max(2) - 1) as f32}", y1: "0", y2: "{h}", "vector-effect": "non-scaling-stroke" }
                    }
                }
                div { class: "oxh-hits",
                    for i in 0..m {
                        span { key: "{i}", onmouseenter: move |_| hover.set(hover_index(m, i as f32 / (m.max(2) - 1) as f32)) }
                    }
                }
                if let Some(i) = hv {
                    {
                        let d = win[i];
                        let left = i as f32 / (m.max(2) - 1) as f32 * 100.0;
                        rsx! {
                            div { class: "oxh-utip", role: "status", style: "left:clamp(60px,{left}%,calc(100% - 60px))",
                                div { class: "oxh-cdim", "{date(i)}" }
                                div { class: "oxh-tr", span { class: "oxh-k a" } "Tokens" b { "{compact(d.tokens)}" } }
                                div { class: "oxh-tr", span { class: "oxh-k c" } "Cached" b { "{compact(d.cached)}" } }
                            }
                        }
                    }
                }
            }
            div { class: "oxh-totals",
                for (l, v) in [("Total usage", tot.total.clone()), ("Avg. daily usage", tot.avg.clone()), ("Est. cost", tot.cost.clone())] {
                    div { span { class: "oxh-cdim", "{l}" } b { Roll { text: v } } }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Attachment strip
// ---------------------------------------------------------------------------------------------

/// Attachment strip (@sitenley behaviour for componest): a row of 56 px tiles above the prompt.
/// Images show a thumbnail, everything else a document mark with the name; each has a dismiss in
/// the corner. Queued files land one after another as the previous upload finishes; a ring
/// draws clockwise around the uploading tile while a 9 px percent counts up in the corner, then
/// at 100 the percent blurs out on the same spot.
#[component]
pub fn AttachmentStrip(
    /// File names.
    files: Vec<String>,
    /// Relative upload weights (size), same order; missing = 1.
    #[props(default)]
    weights: Vec<f32>,
    /// Thumbnail URLs for image files, same order (`None` draws a placeholder).
    #[props(default)]
    thumbs: Vec<Option<String>>,
    /// When the batch was added.
    #[props(default)]
    added: ClockMs,
    /// Shared clock.
    now: ClockMs,
    /// Upload time of a weight-1 file, ms.
    #[props(default = 1400)]
    dur_ms: u64,
    /// Show only tiles whose upload has started (queued files land one after another).
    #[props(default = true)]
    land: bool,
    /// Dismiss pressed.
    #[props(default)]
    on_remove: EventHandler<usize>,
) -> Element {
    let ws: Vec<f32> = (0..files.len())
        .map(|i| weights.get(i).copied().unwrap_or(1.0))
        .collect();
    let states: Vec<_> = (0..files.len())
        .map(|i| tile_upload(&ws, i, added, now, dur_ms, 320))
        .collect();
    let queued = states.iter().filter(|s| s.queued).count();
    let busy = states
        .iter()
        .filter(|s| !s.queued && s.progress < 1.0)
        .count();
    rsx! {
        div { class: "oxh-strip", role: "list", "aria-label": "Attachments",
            for (i, name) in files.iter().enumerate() {
                if !(land && states[i].queued) {
                    {
                        let s = states[i];
                        let icon = file_icon(name);
                        let img = icon == "image";
                        let thumb = thumbs.get(i).cloned().flatten();
                        let label = if s.progress < 1.0 { format!("{name}, uploading {}%", s.pct) } else { format!("{name}, uploaded") };
                        rsx! {
                            div { key: "{name}", class: "oxh-tile", role: "listitem", "aria-label": "{label}", "data-img": "{img}", "data-busy": "{s.progress < 1.0}", "data-q": "{s.queued}",
                                if img {
                                    if let Some(u) = thumb { img { class: "oxh-thumb", src: "{u}", alt: "" } } else { span { class: "oxh-thumb ph", "aria-hidden": "true" } }
                                } else {
                                    span { class: "oxh-doc", "aria-hidden": "true", Icon { name: icon, size: 16.0 } }
                                    span { class: "oxh-tname", "aria-hidden": "true", "{tile_name(name, 7)}" }
                                }
                                if !s.done {
                                    svg { class: "oxh-ring", view_box: "0 0 56 56", "aria-hidden": "true", style: "opacity:{1.0 - s.fade}",
                                        rect { x: "1", y: "1", width: "54", height: "54", rx: "12", "pathLength": "100", stroke_dasharray: "{s.progress * 100.0} 100" }
                                    }
                                    span { class: "oxh-pct", "aria-hidden": "true", style: "filter:blur({s.fade * 4.0}px);opacity:{1.0 - s.fade}", "{s.pct}%" }
                                }
                                button { class: "oxh-x", r#type: "button", "aria-label": "Remove {name}", onclick: move |_| on_remove.call(i), Icon { name: "x", size: 10.0 } }
                            }
                        }
                    }
                }
            }
            if land && queued > 0 {
                div { class: "oxh-tile oxh-queued", role: "listitem", "aria-label": "{queued} queued", "+{queued}" }
            }
            span { class: "oxh-sr", role: "status", if busy > 0 { "Uploading {busy} file" if busy > 1 { "s" } } else if !files.is_empty() { "All files uploaded" } }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Run mode menu and the experiment grid
// ---------------------------------------------------------------------------------------------

/// Run mode menu (Cloudy behaviour): the composer chip that sets how much the agent may do.
/// Ask, Plan, Work and Autoresearch each carry their own shape; Autoresearch opens a time budget
/// row (1h, 5h, 10h, 24h or typed minutes). ↑ / ↓ move, Enter picks, Esc closes.
#[component]
pub fn RunModeMenu(
    /// Mode id (`ask`, `plan`, `work`, `auto`).
    #[props(default = "work".to_string())]
    mode: String,
    /// Budget in minutes for Autoresearch.
    #[props(default = 1440)]
    budget: u32,
    /// Open the menu upwards.
    #[props(default = true)]
    up: bool,
    /// Mode or budget changed: `(mode id, minutes)`.
    #[props(default)]
    on_change: EventHandler<(String, u32)>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut cur = use_signal(|| RunMode::from_id(&mode));
    let mut bud = use_signal(|| budget);
    let mut hi = use_signal(|| 0usize);
    let mut draft = use_signal(|| budget.to_string());
    let m = cur();
    let mut pick = move |k: usize| {
        let mm = RunMode::ALL[k];
        cur.set(mm);
        open.set(false);
        on_change.call((mm.id().to_string(), bud()));
    };
    rsx! {
        div { class: "oxh-mode", "data-up": "{up}",
            onkeydown: move |e: KeyboardEvent| {
                if !open() { return; }
                match e.key() {
                    Key::ArrowDown => { e.prevent_default(); hi.set((hi() + 1) % 4); }
                    Key::ArrowUp => { e.prevent_default(); hi.set((hi() + 3) % 4); }
                    Key::Escape => open.set(false),
                    Key::Enter => { e.prevent_default(); pick(hi()); }
                    _ => {}
                }
            },
            button { class: "oxh-mchip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{open()}", "data-mode": "{m.id()}",
                onclick: move |_| { let o = !open(); open.set(o); if o { hi.set(RunMode::ALL.iter().position(|x| *x == cur()).unwrap_or(0)); } },
                Shape { shape: m.shape() }
                span { key: "{m.id()}", class: "oxh-swap", "{m.label()}" }
                if m.budgeted() { span { class: "oxh-mb", "{hm(bud())}" } }
            }
            if open() {
                div { class: "oxh-catch", onclick: move |_| open.set(false) }
                div { class: "oxh-menu", role: "menu", "aria-label": "Run mode",
                    for (k, rm) in RunMode::ALL.into_iter().enumerate() {
                        div { key: "{rm.id()}", class: "oxh-mi", role: "menuitemradio", tabindex: "-1", "aria-checked": "{rm == m}", "data-hi": "{k == hi()}",
                            onmouseenter: move |_| hi.set(k), onclick: move |_| pick(k),
                            Shape { shape: rm.shape() }
                            div { class: "oxh-mtx", b { "{rm.label()}" } span { "{rm.blurb()}" } }
                            if rm == m { span { class: "oxh-check", Icon { name: "check", size: 12.0 } } }
                        }
                        if rm.budgeted() && (k == hi() || m == rm) {
                            div { class: "oxh-budget", role: "group", "aria-label": "Time budget",
                                for b in BUDGETS {
                                    button { r#type: "button", "aria-pressed": "{bud() == b}",
                                        onclick: move |e: MouseEvent| { e.stop_propagation(); bud.set(b); draft.set(b.to_string()); cur.set(RunMode::Auto); on_change.call(("auto".into(), b)); },
                                        "{hm(b)}"
                                    }
                                }
                                label { class: "oxh-bmin",
                                    input { r#type: "text", inputmode: "numeric", "aria-label": "Budget in minutes", value: "{draft}",
                                        onclick: move |e: MouseEvent| e.stop_propagation(),
                                        oninput: move |e| draft.set(e.value()),
                                        onkeydown: move |e: KeyboardEvent| {
                                            e.stop_propagation();
                                            if e.key() == Key::Enter {
                                                if let Some(b) = parse_budget(&draft()) { bud.set(b); draft.set(b.to_string()); cur.set(RunMode::Auto); open.set(false); on_change.call(("auto".into(), b)); }
                                                else { draft.set(bud().to_string()); }
                                            }
                                        },
                                    }
                                    "min"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Experiment grid (Cloudy behaviour): a budgeted run's experiments as shape marks in bands of
/// hours, so kept results, reverts and crashes read without colour. The header shows the budget
/// used ("2h 12m of 24h") with a hairline; new experiments pop in as the clock reaches them; the
/// legend counts each outcome and pins one (the others dim); hover or focus a mark to read it.
#[component]
pub fn ExperimentGrid(
    /// Experiments, in time order.
    experiments: Vec<Experiment>,
    /// Budget in minutes.
    #[props(default = 1440)]
    budget: u32,
    /// Run time so far, seconds (marks after it are not shown yet).
    elapsed: u64,
    /// Hours per row.
    #[props(default = 4)]
    band: u32,
    /// Paused: the hairline stops pulsing.
    #[props(default)]
    paused: bool,
) -> Element {
    let mut pin = use_signal(|| None::<Outcome>);
    let mut read = use_signal(|| None::<usize>);
    let shown: Vec<Experiment> = experiments
        .iter()
        .filter(|e| e.at <= elapsed)
        .cloned()
        .collect();
    let grid = experiment_grid(&shown, band);
    let rows = (budget.div_ceil(60)).div_ceil(band.max(1));
    let (line, f) = budget_line(elapsed, budget);
    let note = read().and_then(|i| shown.get(i)).cloned();
    rsx! {
        div { class: "oxh-exp",
            div { class: "oxh-ehead",
                Shape { shape: "square" }
                span { "Autoresearch" }
                span { class: "oxh-cdim", key: "{line}", "{line}" }
                span { class: "oxh-ebar", "data-paused": "{paused}", role: "progressbar", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{(f * 100.0).round()}", "aria-label": "Budget used", span { style: "width:{f * 100.0}%" } }
            }
            div { class: "oxh-ehead2",
                b { "Experiments" }
                span { class: "oxh-cdim", "{experiment_counts(&shown)}" }
            }
            div { class: "oxh-elegend", role: "group", "aria-label": "Filter by outcome",
                for o in Outcome::ALL {
                    {
                        let n = shown.iter().filter(|e| e.outcome == o).count();
                        rsx! {
                            button { r#type: "button", "aria-pressed": "{pin() == Some(o)}", "data-o": "{o.id()}",
                                onclick: move |_| pin.set(if pin() == Some(o) { None } else { Some(o) }),
                                Shape { shape: o.shape() } "{o.label()}" span { class: "oxh-cdim", "{n}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxh-egrid", style: "--rows:{rows}",
                for r in 0..rows {
                    div { key: "{r}", class: "oxh-erow",
                        span { class: "oxh-elab", "{r * band}h" }
                        div { class: "oxh-emarks",
                            for (i, rr, _) in grid.iter().copied().filter(|g| g.1 == r) {
                                {
                                    let e = shown[i].clone();
                                    let dim = pin().is_some_and(|p| p != e.outcome);
                                    rsx! {
                                        button { key: "{e.id}", class: "oxh-emark", r#type: "button", "data-o": "{e.outcome.id()}", "data-dim": "{dim}", "data-row": "{rr}",
                                            "aria-label": "{e.id}, {e.outcome.label()}: {e.note}",
                                            onmouseenter: move |_| read.set(Some(i)), onfocus: move |_| read.set(Some(i)),
                                            Shape { shape: e.outcome.shape(), size: 9.0 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxh-enote", role: "status",
                if let Some(e) = note { span { key: "{e.id}", class: "oxh-swap", b { "{e.id}" } " · {e.note}" } } else { span { class: "oxh-cdim", "Hover a mark to read the experiment." } }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Document grid
// ---------------------------------------------------------------------------------------------

/// Document grid (quiet.md behaviour): local documents as a sidebar list and a card grid. Tabs
/// switch My documents and Favorites, the search narrows both, the sort flips Last visited and
/// Title, and the grid / list toggle re-lays the same cards. Stars toggle in place; cards keep
/// their keys so they glide rather than remount.
#[component]
pub fn DocGrid(
    /// Documents.
    docs: Vec<Doc>,
    /// Show the sidebar list.
    #[props(default = true)]
    sidebar: bool,
    /// Start in list view.
    #[props(default)]
    list: bool,
    /// A document was opened.
    #[props(default)]
    on_open: EventHandler<String>,
) -> Element {
    let mut favs = use_signal(|| docs.iter().map(|d| d.favorite).collect::<Vec<_>>());
    let mut tab = use_signal(|| false);
    let mut q = use_signal(String::new);
    let mut sort = use_signal(|| DocSort::LastVisited);
    let mut as_list = use_signal(|| list);
    let mut open_id = use_signal(|| None::<String>);
    let cur: Vec<Doc> = docs
        .iter()
        .zip(favs.read().iter())
        .map(|(d, f)| Doc {
            favorite: *f,
            ..d.clone()
        })
        .collect();
    let view = doc_view(&cur, tab(), &q.read(), sort());
    let side = doc_view(&cur, false, "", DocSort::Title);
    rsx! {
        div { class: "oxh-docs",
          div { class: "oxh-dwrap", "data-side": "{sidebar}",
            if sidebar {
                nav { class: "oxh-dside", "aria-label": "Documents",
                    div { class: "oxh-dsh", "My documents" }
                    for i in side {
                        {
                            let d = cur[i].clone();
                            let id = d.id.clone();
                            rsx! {
                                button { key: "{d.id}", r#type: "button", class: "oxh-dsi", "aria-current": "{open_id.read().as_deref() == Some(d.id.as_str())}",
                                    onclick: move |_| { open_id.set(Some(id.clone())); on_open.call(id.clone()); },
                                    Icon { name: "page", size: 13.0 } span { "{d.title}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxh-dmain",
                div { class: "oxh-dtop",
                    b { "Home" }
                    label { class: "oxh-dsearch", Icon { name: "search", size: 13.0 }
                        input { r#type: "search", placeholder: "Search documents…", "aria-label": "Search documents", value: "{q}", oninput: move |e| q.set(e.value()) }
                    }
                }
                div { class: "oxh-dbar",
                    div { class: "oxh-seg", role: "tablist",
                        button { role: "tab", r#type: "button", "aria-selected": "{!tab()}", onclick: move |_| tab.set(false), "My documents" }
                        button { role: "tab", r#type: "button", "aria-selected": "{tab()}", onclick: move |_| tab.set(true), "Favorites" }
                    }
                    span { class: "oxh-sp" }
                    button { class: "oxh-btn q", r#type: "button", "aria-label": "Sort: {sort().label()}",
                        onclick: move |_| sort.set(if sort() == DocSort::LastVisited { DocSort::Title } else { DocSort::LastVisited }),
                        span { key: "{sort().label()}", class: "oxh-swap", "↓ {sort().label()}" }
                    }
                    div { class: "oxh-seg", role: "group", "aria-label": "Layout",
                        button { r#type: "button", "aria-pressed": "{!as_list()}", "aria-label": "Grid", onclick: move |_| as_list.set(false), Icon { name: "table", size: 13.0 } }
                        button { r#type: "button", "aria-pressed": "{as_list()}", "aria-label": "List", onclick: move |_| as_list.set(true), Icon { name: "checklist", size: 13.0 } }
                    }
                }
                if view.is_empty() {
                    div { class: "oxh-tempty", if tab() { "No favourites yet. Star a document to keep it here." } else { "No documents match." } }
                }
                div { class: "oxh-dgrid", "data-list": "{as_list()}",
                    for i in view {
                        {
                            let d = cur[i].clone();
                            let id = d.id.clone();
                            let fav = d.favorite;
                            rsx! {
                                article { key: "{d.id}", class: "oxh-dcard", tabindex: "0",
                                    onclick: move |_| { open_id.set(Some(id.clone())); on_open.call(id.clone()); },
                                    div { class: "oxh-dct", h4 { "{d.title}" }
                                        button { class: "oxh-star", r#type: "button", "aria-pressed": "{fav}", "aria-label": if fav { "Remove from favorites" } else { "Add to favorites" },
                                            onclick: move |e: MouseEvent| { e.stop_propagation(); let mut f = favs.write(); f[i] = !f[i]; },
                                            Icon { name: "star", size: 12.0 }
                                        }
                                    }
                                    p { "{d.excerpt}" }
                                    span { class: "oxh-cdim", "{oxyn_core::kit::home::ago(d.visited)} ago" }
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

// ---------------------------------------------------------------------------------------------
// Polling cost
// ---------------------------------------------------------------------------------------------

/// Polling cost (@TekonBurns): what background subagents spend checking on each other. A full
/// poll re-reads the whole (growing) transcript every interval; a status ping costs a few
/// hundred tokens. The two bars compare a window, the saving rolls, and the share of a usage
/// limit says how many prompts it buys back.
#[component]
pub fn PollingCost(
    /// Background subagents.
    #[props(default = 3)]
    agents: u32,
    /// Seconds between polls.
    #[props(default = 30)]
    every: u32,
    /// Window, minutes.
    #[props(default = 60)]
    minutes: u32,
    /// Transcript tokens at the start.
    #[props(default = 40_000.0)]
    transcript: f64,
    /// Transcript growth, tokens a minute.
    #[props(default = 600.0)]
    growth: f64,
    /// Tokens a status ping costs.
    #[props(default = 300.0)]
    ping: f64,
    /// Usage limit for the window, tokens.
    #[props(default = 300_000_000.0)]
    limit: f64,
) -> Element {
    let p = poll_cost(agents, every, minutes, transcript, growth, ping);
    let back = ((p.full - p.ping) / limit.max(1.0) * 100.0).clamp(0.0, 100.0);
    let pw = (p.ping / p.full.max(1.0) * 100.0).max(0.6);
    rsx! {
        div { class: "oxh-poll",
            div { class: "oxh-ch", Icon { name: "agents", size: 13.0 } span { "Background status polling" } span { class: "oxh-more", "{agents} subagents · every {every}s · {hm(minutes)}" } }
            div { class: "oxh-prow",
                span { class: "oxh-pl", "Re-read transcript" }
                span { class: "oxh-pbar", span { class: "full", style: "width:100%" } }
                b { Roll { text: compact(p.full) } }
            }
            div { class: "oxh-prow",
                span { class: "oxh-pl", "Status ping" }
                span { class: "oxh-pbar", span { style: "width:{pw}%" } }
                b { Roll { text: compact(p.ping) } }
            }
            div { class: "oxh-pfoot",
                span { class: "oxh-big", Roll { text: format!("{:.0}%", p.saved * 100.0) } }
                span { class: "oxh-cdim", "less polling cost over {p.polls} polls, about {back:.1}% of the limit back for prompts." }
            }
        }
    }
}
