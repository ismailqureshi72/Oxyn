//! Screens S12 (Arc screens I): **Metrics dashboard**, **Kanban board**, **Project board**,
//! **Gantt timeline** and **Team directory** on the Arc screen frame: a 248 px sidebar (workspace,
//! search, nav with counts, labelled groups, a context card and an account row with the email and
//! its menu), an inset page with a one- or two-row header, and an optional right card. Rules in
//! `oxyn_core::parity::arcscr`; styles in `arcscr.css` (`oxs12-`). Chrome stays monochrome;
//! colour is for statuses, labels, project dots, availability and official marks.

use dioxus::prelude::*;
use oxyn_core::parity::arcscr::{
    ACTIVITY, Avail, Bar, CYCLE, Col, GT_NAV, GT_VIEWS, Health, Issue, KB_NAV, KB_PROJECTS, KPIS,
    LABELS, LINKS, MILESTONES, MX_NAV, MX_VIEWS, Member, OFFICES, ONLINE, PB_NAV, PB_PROJECTS,
    PEOPLE, PHASES, PLAN, PRIOS, PROJECT, Plan, RANGES, STATUSES, ScreenState, TABLES, TD_NAV,
    TEAM, TEAMS, TODAY, Task, ZOOMS, acct_menu, add_issue, arc_split, assignees, axis_value,
    bar_dates, col_counts, column, conflicts, end_of, find_people, initials, iso, issue_key,
    issues, kpi_value, load_of, local_time, members, month_day, move_issue, ms_done, ms_progress,
    my_email, nice_top, nudge, office_count, office_open, phase_span, plan_done, point_day, points,
    range_span, s12_states as core_states, series, stats, table, tasks, thousands, toggle_task,
    weekday, ws, zoom_window,
};

use super::screensfid::{MenuButton, nico, stage_scale};
use crate::brand::BrandIcon;

/// The state bar entries of an S12 screen (for a host's `ScreenStage { states }`).
pub fn s12_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    core_states(slug).to_vec()
}

// ───────────────────────────────────────────────────────────── shared parts ──

fn av(who: usize, small: bool) -> Element {
    let (n, h) = PEOPLE[who % PEOPLE.len()];
    let hue = oxyn_core::parity::arcscr::HUES[h];
    let ini = initials(who);
    rsx! { span { class: "oxs12-av", "data-hue": "{hue}", "data-sm": "{small}", title: "{n}", "aria-hidden": "true", "{ini}" } }
}

fn hue_ico(hue: &str, icon: &str) -> Element {
    rsx! { span { class: "oxs12-hi", "data-hue": "{hue}", {nico(icon)} } }
}

fn prio_bars(p: usize) -> Element {
    let (l, n) = PRIOS[p.min(4)];
    if n == 4 {
        return rsx! { span { class: "oxs12-urg", title: "{l}", role: "img", "aria-label": "Priority {l}", "!" } };
    }
    rsx! {
        svg { class: "oxs12-pri", view_box: "0 0 14 14", width: "14", height: "14", role: "img", "aria-label": "Priority {l}",
            for i in 0..3u8 {
                { let x = 2 + i as u32 * 4; let h = 4 + i as u32 * 3; let y = 12 - h; let on = i < n;
                  rsx! { rect { key: "{i}", x: "{x}", y: "{y}", width: "2.6", height: "{h}", rx: "1", "data-on": "{on}" } } }
            }
            if n == 0 { line { x1: "2", y1: "7", x2: "12", y2: "7", "data-on": "true", stroke_width: "1.6", stroke_dasharray: "1.5 2" } }
        }
    }
}

fn ring(pct: u32, hue: &str) -> Element {
    let c = 2.0 * std::f32::consts::PI * 6.0;
    let off = c * (1.0 - pct.min(100) as f32 / 100.0);
    rsx! {
        svg { class: "oxs12-ring", "data-hue": "{hue}", view_box: "0 0 16 16", width: "16", height: "16", role: "img", "aria-label": "{pct}% done",
            circle { cx: "8", cy: "8", r: "6", "data-bg": "true" }
            circle { cx: "8", cy: "8", r: "6", stroke_dasharray: "{c:.2}", stroke_dashoffset: "{off:.2}", transform: "rotate(-90 8 8)" }
        }
    }
}

fn skl(n: usize, class: &'static str) -> Element {
    rsx! {
        div { class: "oxs12-skl {class}", "aria-hidden": "true",
            for i in 0..n { { let w = [92, 64, 78, 48, 70][i % 5]; rsx! { span { key: "{i}", class: "oxs12-sb", style: "width:{w}%" } } } }
        }
    }
}

fn none(icon: &str, title: &str, line: &str, act: Element) -> Element {
    rsx! {
        div { class: "oxs12-none",
            span { class: "oxs12-none-g", {nico(icon)} }
            p { class: "oxs12-none-t", "{title}" }
            p { class: "oxs12-none-l", "{line}" }
            {act}
        }
    }
}

/// A sidebar row: icon, label, count; `on` marks the open one.
fn nrow(
    icon: Element,
    label: String,
    count: String,
    on: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button { r#type: "button", class: "oxs12-nr", "aria-current": if on { "page" } else { "false" }, onclick: move |e| onclick.call(e),
            span { class: "oxs12-nri", {icon} }
            span { class: "oxs12-ell oxs12-nrl", "{label}" }
            if !count.is_empty() { span { class: "oxs12-nrc", "{count}" } }
        }
    }
}

fn group(label: &str, plus: bool) -> Element {
    rsx! {
        div { class: "oxs12-gl",
            span { class: "oxs12-ell", "{label}" }
            if plus { span { class: "oxs12-gp", "aria-hidden": "true", {nico("plus")} } }
        }
    }
}

fn ws_mark(slug: &str) -> Element {
    let d = match slug {
        "kanban-board" => "M4 15 L10 5 L16 15 Z",
        "project-board" => "M10 3 a7 7 0 1 0 0.01 0 Z",
        "gantt-timeline" => "M3 5 h14 v3 h-14 Z M3 11 h9 v3 h-9 Z",
        "team-directory" => "M4 16 L16 4 M4 10 L10 4 M10 16 L16 10",
        _ => "M10 3 a7 7 0 0 1 7 7 h-7 Z M10 10 v7 a7 7 0 0 1 -7 -7 Z",
    };
    let stroke = slug == "team-directory";
    rsx! {
        span { class: "oxs12-wsm", "aria-hidden": "true",
            svg { view_box: "0 0 20 20", width: "14", height: "14",
                if stroke { path { d: "{d}", fill: "none", stroke: "currentColor", stroke_width: "2.2", stroke_linecap: "round" } }
                else { path { d: "{d}", fill: "currentColor" } }
            }
        }
    }
}

/// The Arc frame: sidebar (workspace, search, `side`, `card`, account), page (header rows,
/// body) and an optional right card. Under 960 px (container) the sidebar is a drawer from the
/// header's menu button.
#[component]
fn ArcFrame(
    slug: &'static str,
    layout: String,
    side: Element,
    #[props(default)] card: Option<Element>,
    crumbs: Vec<String>,
    #[props(default)] meta: String,
    actions: Element,
    #[props(default)] bar: Option<Element>,
    #[props(default)] right: Option<Element>,
    #[props(default = true)] show_right: bool,
    #[props(default = "Search".to_string())] search: String,
    #[props(default)] on_search: Option<EventHandler<()>>,
    said: Signal<String>,
    #[props(default)] foot: Option<Element>,
    children: Element,
) -> Element {
    let mut menu = use_signal(|| false);
    let mut drawer = use_signal(|| false);
    let mut hidden = use_signal(|| false);
    let mut said = said;
    let w = ws(slug);
    let email = my_email(slug);
    let items = acct_menu(slug);
    let last = crumbs.len().saturating_sub(1);
    let has_right = right.is_some() && show_right;
    rsx! {
        div { class: "oxs12", "data-layout": "{layout}", "data-drawer": "{drawer()}", "data-side": if hidden() { "hidden" } else { "shown" }, "data-right": "{has_right}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { if menu() { menu.set(false); } else if drawer() { drawer.set(false); } },
            div { class: "oxs12-scrim", onclick: move |_| drawer.set(false) }
            nav { class: "oxs12-side", "aria-label": "{w.name}",
                div { class: "oxs12-ws",
                    button { r#type: "button", class: "oxs12-wsb", "aria-label": "Workspace {w.name}", onclick: move |_| said.set(format!("{} workspace", w.name)),
                        {ws_mark(slug)}
                        span { class: "oxs12-ell oxs12-wsn", "{w.name}" }
                        span { class: "oxs12-car", {nico("caret")} }
                    }
                    button { r#type: "button", class: "oxs12-ib", "aria-label": "Hide sidebar", title: "Hide sidebar",
                        onclick: move |_| { if drawer() { drawer.set(false); } else { hidden.set(true); } }, {nico("side")} }
                }
                button { r#type: "button", class: "oxs12-search", onclick: move |_| { drawer.set(false); match on_search { Some(h) => h.call(()), None => said.set("Search opened".to_string()) } },
                    {nico("search")} span { class: "oxs12-ell", "{search}" } kbd { "⌘K" } }
                div { class: "oxs12-sscroll", {side} }
                div { class: "oxs12-sfoot",
                    if let Some(c) = card { div { class: "oxs12-ccard", {c} } }
                    div { class: "oxs12-acct",
                        button { r#type: "button", class: "oxs12-acb", "aria-haspopup": "menu", "aria-expanded": "{menu()}", "aria-label": "Account, {oxyn_core::parity::arcscr::ME}, {email}",
                            onclick: move |_| menu.toggle(),
                            span { class: "oxs12-av", "data-hue": "blue", "aria-hidden": "true", "{oxyn_core::parity::arcscr::ME_INI}" }
                            span { class: "oxs12-an", b { class: "oxs12-ell", "{oxyn_core::parity::arcscr::ME}" } span { class: "oxs12-ell", "{email}" } }
                            span { class: "oxs12-ud", {nico("updown")} }
                        }
                        if menu() {
                            div { class: "oxs12-catch", onclick: move |_| menu.set(false) }
                            div { class: "oxs12-menu", role: "menu", "aria-label": "Account",
                                div { class: "oxs12-mh", span { class: "oxs12-av", "data-hue": "blue", "aria-hidden": "true", "{oxyn_core::parity::arcscr::ME_INI}" }
                                    span { class: "oxs12-an", b { class: "oxs12-ell", "{oxyn_core::parity::arcscr::ME}" } span { class: "oxs12-ell", "{email}" } } }
                                for (i, (g, l, k)) in items.iter().copied().enumerate() {
                                    button { key: "{i}", r#type: "button", class: "oxs12-mi", role: "menuitem",
                                        onclick: move |_| { menu.set(false); said.set(l.to_string()); },
                                        {nico(g)} span { class: "oxs12-ell", "{l}" } if !k.is_empty() { kbd { "{k}" } } }
                                }
                            }
                        }
                    }
                }
            }
            main { class: "oxs12-page",
                header { class: "oxs12-hd",
                    button { r#type: "button", class: "oxs12-ib oxs12-menub", "aria-label": "Open sidebar", onclick: move |_| drawer.set(true), {nico("side")} }
                    if hidden() { button { r#type: "button", class: "oxs12-ib oxs12-showb", "aria-label": "Show sidebar", onclick: move |_| hidden.set(false), {nico("side")} } }
                    ol { class: "oxs12-crumbs", "aria-label": "Breadcrumb",
                        for (i, c) in crumbs.iter().enumerate() {
                            li { key: "{i}", class: "oxs12-cr", "aria-current": if i == last { "page" } else { "false" },
                                if i > 0 { span { class: "oxs12-sep", "aria-hidden": "true", "›" } }
                                span { class: "oxs12-ell", "{c}" }
                            }
                        }
                    }
                    if !meta.is_empty() { span { class: "oxs12-meta oxs12-ell", "{meta}" } }
                    span { class: "oxs12-grow" }
                    div { class: "oxs12-acts", {actions} }
                }
                if let Some(b) = bar { div { class: "oxs12-bar", {b} } }
                div { class: "oxs12-body", {children} }
                if let Some(f) = foot { div { class: "oxs12-foot", {f} } }
            }
            if has_right && let Some(r) = right { aside { class: "oxs12-rc", {r} } }
            div { class: "oxs12-live", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

fn btn(label: &str, icon: &str, primary: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button { r#type: "button", class: "oxs12-btn", "data-pri": "{primary}", "aria-label": "{label}", onclick: move |e| onclick.call(e),
            {nico(icon)} span { class: "oxs12-btl", "{label}" }
        }
    }
}

fn seg(
    aria: &str,
    items: Vec<(&'static str, String)>,
    cur: &str,
    on: EventHandler<&'static str>,
) -> Element {
    rsx! {
        div { class: "oxs12-seg", role: "radiogroup", "aria-label": "{aria}",
            for (k, l) in items {
                button { key: "{k}", r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{cur == k}", onclick: move |_| on.call(k), "{l}" }
            }
        }
    }
}

fn find_field(
    aria: &'static str,
    ph: &'static str,
    mut q: Signal<String>,
    mounted: Option<Signal<Option<std::rc::Rc<MountedData>>>>,
) -> Element {
    // Long placeholders (the people search) get room to show in full.
    let basis = if ph.len() > 20 { 280 } else { 250 };
    rsx! {
        label { class: "oxs12-find", style: "--fb:{basis}px",
            {nico("search")}
            input { r#type: "search", placeholder: "{ph}", "aria-label": "{aria}", value: "{q}",
                onmounted: move |e: MountedEvent| if let Some(mut m) = mounted { m.set(Some(e.data())); },
                oninput: move |e| q.set(e.value()),
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); q.set(String::new()); } }
            kbd { "/" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Metrics ──

/// Screen – Metrics dashboard: a date range, Export, KPI tabs over one chart (hover for a day's
/// value, drag across to zoom, Compare with the period before) and three breakdown cards (pages,
/// sources, places) with inline bars. States: `sample`, `loading`, `empty`, `compare`, `zoomed`,
/// `exported`.
#[component]
pub fn MetricsDashboardScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc_split(&state);
    let mut said = use_signal(String::new);
    let mut range = use_signal(|| 1usize);
    let mut kpi = use_signal(|| 0usize);
    let mut compare = use_signal(move || own == "compare");
    let mut zoom = use_signal(move || {
        if own == "zoomed" {
            Some((8usize, 21usize))
        } else {
            None
        }
    });
    let mut exported = use_signal(move || own == "exported");
    let mut tabs = use_signal(|| [false; 3]);
    let mut pick = use_signal(|| None::<(usize, usize)>);
    let mut view = use_signal(|| None::<usize>);
    let loading = st == ScreenState::Loading;
    let empty = st == ScreenState::Empty;
    let r = range();
    let (a, z) = range_span(r);
    let crumbs = vec!["Lumenfield".to_string(), "Overview".to_string()];
    let range_items: Vec<(&'static str, &'static str, &'static str)> =
        RANGES.iter().map(|x| (x.0, x.1, "")).collect();
    let view_items: Vec<(&'static str, &'static str, &'static str)> =
        MX_VIEWS.iter().map(|x| (x.1, x.1, "")).collect();
    let fname = format!("metrics-{a}-to-{z}.csv");
    let fname2 = fname.clone();
    rsx! {
        ArcFrame { slug: "metrics-dashboard", layout, said, crumbs,
            side: rsx! {
                for (i, (k, ic, l, n)) in MX_NAV.iter().copied().enumerate() {
                    {nrow(if ic == "grid" { nico("apps") } else { nico(ic) }, l.to_string(), n.to_string(), i == 0, EventHandler::new(move |_| said.set(format!("{l} opened"))))}
                    if k == "goals" { {group("Saved views", true)} }
                }
                for (i, (ic, l, rk)) in MX_VIEWS.iter().copied().enumerate() {
                    {
                        let ri = RANGES.iter().position(|x| x.0 == rk).unwrap_or(1);
                        let on = view() == Some(i);
                        let icon = if ic == "google" { rsx! { BrandIcon { name: "google".to_string(), size: 15.0, coloured: true } } } else { nico(ic) };
                        rsx! { {nrow(icon, l.to_string(), rk.to_string(), on, EventHandler::new(move |_| { view.set(Some(i)); range.set(ri); zoom.set(None); said.set(format!("{l}: {}", RANGES[ri].1)); }))} }
                    }
                }
            },
            card: rsx! {
                div { class: "oxs12-online",
                    div { class: "oxs12-cr1", span { class: "oxs12-hi", "data-hue": "green", {nico("pulse")} } span { class: "oxs12-ell", "Online now" } b { "{ONLINE.1}" } }
                    div { class: "oxs12-spark", "aria-hidden": "true",
                        for (i, v) in ONLINE.0.iter().enumerate() { { let h = v * 100 / 17; let last = i == ONLINE.0.len() - 1; rsx! { span { key: "{i}", "data-last": "{last}", style: "height:{h}%" } } } }
                    }
                    div { class: "oxs12-cr2", span { "30 min ago" } span { "Now" } }
                }
            },
            actions: rsx! {
                MenuButton { label: RANGES[r].1.to_string(), aria: "Date range", icon: "cal", caret: true, right: true, items: range_items, checked: RANGES[r].0.to_string(),
                    on_pick: move |k: String| { if let Some(i) = RANGES.iter().position(|x| x.0 == k) { range.set(i); zoom.set(None); view.set(None); said.set(RANGES[i].1.to_string()); } } }
                button { r#type: "button", class: "oxs12-btn", "data-done": "{exported()}", "aria-label": if exported() { "Exported" } else { "Export" },
                    onclick: move |_| { exported.set(true); said.set(format!("{fname} exported")); },
                    if exported() { {nico("check")} } else { {nico("download")} }
                    span { class: "oxs12-btl", if exported() { "Exported" } else { "Export" } }
                }
            },
            bar: rsx! {
                MenuButton { label: "Filter".to_string(), aria: "Filter: saved views", icon: "filter", items: view_items, checked: view().map(|i| MX_VIEWS[i].1.to_string()).unwrap_or_default(),
                    on_pick: move |k: String| { if let Some(i) = MX_VIEWS.iter().position(|x| x.1 == k) { let ri = RANGES.iter().position(|x| x.0 == MX_VIEWS[i].2).unwrap_or(1); view.set(Some(i)); range.set(ri); zoom.set(None); said.set(format!("{k}: {}", RANGES[ri].1)); } } }
                if let Some(i) = view() { span { class: "oxs12-chip", "Saved view: {MX_VIEWS[i].1}" button { r#type: "button", class: "oxs12-chx", "aria-label": "Clear the saved view", onclick: move |_| view.set(None), {nico("close")} } } }
                span { class: "oxs12-grow" }
                span { class: "oxs12-meta oxs12-ell", if exported() { "{fname2} · " } "{a} to {z}" }
            },
            if loading {
                div { class: "oxs12-mx",
                    div { class: "oxs12-kpis oxs12-card", for i in 0..5 { div { key: "{i}", class: "oxs12-kpi", {skl(3, "")} } } }
                    div { class: "oxs12-card oxs12-chartc", {skl(6, "oxs12-skc")} }
                    div { class: "oxs12-tbls", for i in 0..3 { div { key: "{i}", class: "oxs12-card oxs12-tbl", {skl(7, "")} } } }
                }
            } else {
                div { class: "oxs12-mx",
                    div { class: "oxs12-kpis oxs12-card", role: "tablist", "aria-label": "Metric",
                        for (i, (_, l)) in KPIS.iter().copied().enumerate() {
                            {
                                let (v, ch, good) = if empty { ("0".to_string(), String::new(), true) } else { kpi_value(i, r) };
                                let on = kpi() == i;
                                let up = ch.starts_with('+');
                                let chs = ch.trim_start_matches(['+', '-']).to_string();
                                rsx! {
                                    button { key: "{i}", r#type: "button", role: "tab", class: "oxs12-kpi", "aria-selected": "{on}",
                                        onclick: move |_| { kpi.set(i); said.set(format!("{l} on the chart")); },
                                        span { class: "oxs12-kl oxs12-ell", "{l}" }
                                        b { class: "oxs12-kv oxs12-ell", "{v}" }
                                        if !ch.is_empty() { span { class: "oxs12-kd", "data-good": "{good}", if up { "↗ " } else { "↘ " } "{chs}" } }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxs12-card oxs12-chartc",
                        if empty {
                            {none("chart", "No visits yet", "Add the tracking snippet to your site; visits show here within a minute.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| said.set("Snippet copied".to_string()), {nico("copy")} span { class: "oxs12-btl", "Copy snippet" } } })}
                        } else {
                            Chart { k: kpi(), r, compare: compare(), zoom: zoom(), on_zoom: move |w: Option<(usize, usize)>| { zoom.set(w); if let Some((a2, b2)) = w { said.set(format!("Zoomed to {} – {}", iso(point_day(r, a2)), iso(point_day(r, b2)))); } },
                                head: rsx! {
                                    span { class: "oxs12-leg", span { class: "oxs12-legl" } "{KPIS[kpi()].1}" }
                                    if compare() { span { class: "oxs12-leg", span { class: "oxs12-legl", "data-prev": "true" } "Previous period" } }
                                    span { class: "oxs12-grow" }
                                    if zoom().is_some() {
                                        button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { zoom.set(None); said.set("Zoom reset".to_string()); }, {nico("zoomout")} span { class: "oxs12-btl", "Reset zoom" } }
                                    } else { span { class: "oxs12-hint oxs12-ell", "Drag across the chart to zoom" } }
                                    label { class: "oxs12-sw",
                                        input { r#type: "checkbox", role: "switch", checked: compare(), "aria-label": "Compare with the previous period", onchange: move |e| compare.set(e.checked()) }
                                        span { class: "oxs12-swt", "aria-hidden": "true" }
                                        span { "Compare" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxs12-tbls",
                        for (t, (_, ta, tb)) in TABLES.iter().copied().enumerate() {
                            {
                                let b = tabs()[t];
                                let rows = if empty { vec![] } else { table(t, b, r) };
                                let max = rows.iter().map(|x| x.n).max().unwrap_or(1).max(1);
                                let what = match (t, b) { (0, _) => "pages", (1, false) => "referrers", (1, true) => "campaigns", (_, false) => "countries", _ => "cities" };
                                rsx! {
                                    section { key: "{t}", class: "oxs12-card oxs12-tbl", "aria-label": if b { tb } else { ta },
                                        div { class: "oxs12-tbh",
                                            div { class: "oxs12-seg", role: "tablist",
                                                button { r#type: "button", role: "tab", class: "oxs12-sego", "aria-selected": "{!b}", onclick: move |_| tabs.write()[t] = false, "{ta}" }
                                                button { r#type: "button", role: "tab", class: "oxs12-sego", "aria-selected": "{b}", onclick: move |_| tabs.write()[t] = true, "{tb}" }
                                            }
                                            span { class: "oxs12-grow" }
                                            span { class: "oxs12-q", "Visitors" }
                                        }
                                        if rows.is_empty() { p { class: "oxs12-faint", "No {what} yet" } }
                                        ul { class: "oxs12-rows",
                                            for (j, x) in rows.into_iter().enumerate() {
                                                {
                                                    let w = (x.n * 100 / max).max(2);
                                                    let on = pick() == Some((t, j));
                                                    let icon = match x.icon {
                                                        "" => rsx! {},
                                                        "link" | "tag" => rsx! { span { class: "oxs12-ri", {nico(x.icon)} } },
                                                        c if c.len() == 2 => rsx! { span { class: "oxs12-cc", "{c}" } },
                                                        bnd => rsx! { span { class: "oxs12-ri", BrandIcon { name: bnd.to_string(), size: 14.0, coloured: true } } },
                                                    };
                                                    let n = thousands(x.n);
                                                    let lab = x.label;
                                                    rsx! {
                                                        li { key: "{j}",
                                                            button { r#type: "button", class: "oxs12-row", "aria-pressed": "{on}", onclick: move |_| { pick.set(Some((t, j))); said.set(format!("{lab}: {n} visitors")); },
                                                                span { class: "oxs12-rbar", style: "width:{w}%" }
                                                                {icon}
                                                                span { class: "oxs12-ell oxs12-rl", title: "{lab}", "{lab}" }
                                                                span { class: "oxs12-rn", "{thousands(x.n)}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if !empty { button { r#type: "button", class: "oxs12-more", onclick: move |_| said.set(format!("All {what}")), "View all {what} ›" } }
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

/// The chart: grid, axis labels, the area and line, the previous period dashed, hover readout and
/// drag to zoom.
#[component]
fn Chart(
    k: usize,
    r: usize,
    compare: bool,
    zoom: Option<(usize, usize)>,
    on_zoom: EventHandler<Option<(usize, usize)>>,
    head: Element,
) -> Element {
    let mut wpx = use_signal(|| 1000.0f64);
    let mut hover = use_signal(|| None::<usize>);
    let mut drag = use_signal(|| None::<(usize, usize)>);
    let full = series(k, r, false);
    let fullp = series(k, r, true);
    let (lo, hi) = zoom.unwrap_or((0, full.len() - 1));
    let cur: Vec<f64> = full[lo..=hi].to_vec();
    let prev: Vec<f64> = fullp[lo..=hi].to_vec();
    let n = cur.len();
    let mut max = cur.iter().cloned().fold(0.0, f64::max);
    if compare {
        max = prev.iter().cloned().fold(max, f64::max);
    }
    let (top, step) = nice_top(max);
    let x = |i: usize| {
        if n < 2 {
            0.0
        } else {
            i as f64 * 1000.0 / (n - 1) as f64
        }
    };
    let y = |v: f64| 200.0 - v / top * 200.0;
    let line: String = cur
        .iter()
        .enumerate()
        .map(|(i, v)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, x(i), y(*v)))
        .collect();
    let area = format!("{line}L1000,200L0,200Z");
    let pline: String = prev
        .iter()
        .enumerate()
        .map(|(i, v)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, x(i), y(*v)))
        .collect();
    let ticks: Vec<(f64, String)> = (0..=4)
        .map(|i| (i as f64 * step, axis_value(k, i as f64 * step)))
        .collect();
    let nl = 6.min(n);
    let xl: Vec<String> = (0..nl)
        .map(|j| {
            iso(point_day(
                r,
                lo + if nl < 2 { 0 } else { j * (n - 1) / (nl - 1) },
            ))
        })
        .collect();
    let idx = move |px: f64| -> usize {
        let w = *wpx.peek();
        ((px / w.max(1.0)) * (n.saturating_sub(1)) as f64)
            .round()
            .clamp(0.0, n.saturating_sub(1) as f64) as usize
    };
    let hv = hover();
    let tip = hv.map(|i| {
        let v = cur[i.min(n - 1)];
        let vs = match k {
            3 => format!("{v:.1}%"),
            4 => format!("{}m {:02}s", v as u32 / 60, v as u32 % 60),
            _ => thousands(v.round() as u64),
        };
        (i, format!("{} · {vs}", iso(point_day(r, lo + i))))
    });
    let tip_x = tip.as_ref().map(|(i, _)| x(*i));
    let tip_l = tip
        .as_ref()
        .map(|(i, t)| ((x(*i) / 10.0).clamp(8.0, 92.0), t.clone()));
    let sel = drag().map(|(a, b)| (x(a.min(b)) / 10.0, (x(a.max(b)) - x(a.min(b))) / 10.0));
    rsx! {
        div { class: "oxs12-chart",
            div { class: "oxs12-chh", {head} }
            div { class: "oxs12-plot",
                div { class: "oxs12-yax", "aria-hidden": "true",
                    for (i, (_, l)) in ticks.iter().enumerate().rev() { span { key: "{i}", "{l}" } }
                }
                div { class: "oxs12-pa",
                    svg { class: "oxs12-svg", view_box: "0 0 1000 200", preserve_aspect_ratio: "none", role: "img", "aria-label": "{KPIS[k].1} from {xl.first().cloned().unwrap_or_default()} to {xl.last().cloned().unwrap_or_default()}",
                        for (i, (v, _)) in ticks.iter().enumerate() { { let yy = y(*v); rsx! { line { key: "{i}", class: "oxs12-gl0", x1: "0", x2: "1000", y1: "{yy:.1}", y2: "{yy:.1}" } } } }
                        path { class: "oxs12-area", d: "{area}" }
                        if compare { path { class: "oxs12-pline", d: "{pline}" } }
                        path { class: "oxs12-line", d: "{line}" }
                        if let Some(xx) = tip_x { line { class: "oxs12-hl", x1: "{xx:.1}", x2: "{xx:.1}", y1: "0", y2: "200" } }
                    }
                    if let Some((l, w)) = sel { span { class: "oxs12-sel", style: "left:{l:.2}%;width:{w:.2}%" } }
                    if let Some((l, t)) = tip_l { span { class: "oxs12-tip", style: "left:{l:.2}%", "{t}" } }
                    div { class: "oxs12-catchc", "aria-hidden": "true",
                        onresize: move |e: Event<ResizeData>| if let Ok(s) = e.get_border_box_size() { wpx.set(s.width); },
                        onmousedown: move |e: MouseEvent| { let i = idx(e.element_coordinates().x); drag.set(Some((i, i))); },
                        onmousemove: move |e: MouseEvent| { let i = idx(e.element_coordinates().x); hover.set(Some(i)); if let Some((a, _)) = drag() { drag.set(Some((a, i))); } },
                        onmouseup: move |_| { if let Some((a, b)) = drag() && let Some((za, zb)) = zoom_window(n, a, b) { on_zoom.call(Some((lo + za, lo + zb))); } drag.set(None); },
                        onmouseleave: move |_| { hover.set(None); drag.set(None); },
                    }
                }
            }
            div { class: "oxs12-xax", "aria-hidden": "true", for (i, l) in xl.iter().enumerate() { span { key: "{i}", "{l}" } } }
        }
    }
}

// ───────────────────────────────────────────────────────────── Kanban ──

#[derive(Clone, Copy, PartialEq)]
struct KDrag {
    id: u32,
    x0: f64,
    y0: f64,
    dx: f64,
    dy: f64,
}

fn label_chip(l: usize) -> Element {
    let (_, n, h) = LABELS[l.min(5)];
    rsx! { span { class: "oxs12-lab", "data-hue": "{h}", span { class: "oxs12-dot" } span { class: "oxs12-ell", "{n}" } } }
}

fn col_glyph(c: Col) -> Element {
    let t = c.tone();
    let d = match c {
        Col::Backlog => {
            rsx! { circle { cx: "7", cy: "7", r: "5.2", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_dasharray: "1.6 1.8" } }
        }
        Col::Todo => {
            rsx! { circle { cx: "7", cy: "7", r: "5.2", fill: "none", stroke: "currentColor", stroke_width: "1.5" } }
        }
        Col::Progress => {
            rsx! { circle { cx: "7", cy: "7", r: "5.2", fill: "none", stroke: "currentColor", stroke_width: "1.5" } path { d: "M7 3.6 A3.4 3.4 0 0 1 7 10.4 Z", fill: "currentColor" } }
        }
        Col::Review => {
            rsx! { circle { cx: "7", cy: "7", r: "5.2", fill: "none", stroke: "currentColor", stroke_width: "1.5" } path { d: "M7 3.6 A3.4 3.4 0 1 1 3.6 7 L7 7 Z", fill: "currentColor" } }
        }
        Col::Done => {
            rsx! { circle { cx: "7", cy: "7", r: "6", fill: "currentColor" } path { d: "M4.4 7.2l1.8 1.8 3.4-3.6", fill: "none", stroke: "var(--oxth-card)", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round" } }
        }
    };
    rsx! { svg { class: "oxs12-cg", "data-hue": "{t}", view_box: "0 0 14 14", width: "14", height: "14", "aria-hidden": "true", {d} } }
}

/// Screen – Kanban board (the cycle board): issues in Backlog, Todo, In progress, In review and
/// Done with work-in-progress limits; drag a card, or pick it with Space and move it with the
/// arrows (Alt + arrows also work); text, label and priority filters; group by status or
/// assignee; + New issue per column; the sidebar's cycle card counts done points. States:
/// `sample`, `loading`, `empty`, `dragging`, `filtered`, `assignee`.
#[component]
pub fn KanbanBoardScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// The issues (`None` = the sample board for the state).
    #[props(default)]
    data: Option<Vec<Issue>>,
) -> Element {
    let (st, own) = arc_split(&state);
    let mut said = use_signal(String::new);
    let mut board = use_signal(move || {
        data.clone()
            .unwrap_or_else(|| issues(st == ScreenState::Sample))
    });
    let q = use_signal(String::new);
    let mut label = use_signal(move || {
        if own == "filtered" {
            Some(1usize)
        } else {
            None
        }
    });
    let mut prio = use_signal(|| None::<usize>);
    let mut by_person = use_signal(move || own == "assignee");
    let mut lift = use_signal(|| None::<u32>);
    let mut drag = use_signal(move || {
        if own == "dragging" {
            Some(KDrag {
                id: 214,
                x0: 0.0,
                y0: 0.0,
                dx: 286.0,
                dy: 34.0,
            })
        } else {
            None
        }
    });
    let mut over = use_signal(move || {
        if own == "dragging" {
            Some(Col::Review)
        } else {
            None
        }
    });
    let mut adding = use_signal(|| None::<Col>);
    let mut draft = use_signal(String::new);
    let loading = st == ScreenState::Loading;
    let b = board();
    let (done, total) = points(&b);
    let counts = col_counts(&b);
    let qq = q();
    let lb = label();
    let pr = prio();
    let shown = |c: Col| -> Vec<Issue> {
        column(&b, c, &qq, lb)
            .into_iter()
            .filter(|i| pr.is_none_or(|p| i.prio == p))
            .collect()
    };
    let mut drop_it = move || {
        let d = *drag.peek();
        let o = *over.peek();
        if let (Some(d), Some(c)) = (d, o) {
            let moved = (d.dx.abs() + d.dy.abs()) > 6.0;
            if moved && move_issue(&mut board.write(), d.id, c, None) {
                said.set(format!("{} moved to {}", issue_key(d.id), c.label()));
            }
        }
        drag.set(None);
        over.set(None);
    };
    let mut add = move || {
        if let Some(c) = adding() {
            let t = draft.peek().clone();
            if let Some(id) = add_issue(&mut board.write(), c, &t) {
                said.set(format!("{} added to {}", issue_key(id), c.label()));
                draft.set(String::new());
                adding.set(None);
            }
        }
    };
    let lab_items: Vec<(&'static str, &'static str, &'static str)> =
        std::iter::once(("all", "All labels", ""))
            .chain(LABELS.iter().map(|x| (x.0, x.1, "")))
            .collect();
    let pri_items: Vec<(&'static str, &'static str, &'static str)> =
        std::iter::once(("all", "Any priority", ""))
            .chain(PRIOS.iter().map(|x| (x.0, x.0, "")).rev())
            .collect();
    let days_left = CYCLE.2 - TODAY;
    let day_n = TODAY - CYCLE.1 + 1;
    let span = CYCLE.2 - CYCLE.1 + 1;
    let pct = (done * 100).checked_div(total).unwrap_or(0);
    let card_el = move |i: Issue| {
        let id = i.id;
        let d = drag();
        let lifted = d.map(|d| d.id == id).unwrap_or(false);
        let kb = lift() == Some(id);
        let style = match d {
            Some(d) if d.id == id => format!("--dx:{:.0}px;--dy:{:.0}px", d.dx, d.dy),
            _ => String::new(),
        };
        let key = issue_key(id);
        let t = i.title.clone();
        let col = i.col;
        let due = i.due.map(|d| {
            (
                iso(d),
                if d < TODAY {
                    "rose"
                } else if d <= TODAY + 2 {
                    "amber"
                } else {
                    "mute"
                },
            )
        });
        rsx! {
            li { key: "{id}", class: "oxs12-kc", "data-lift": "{lifted}", "data-kb": "{kb}", style: "{style}",
                onpointerdown: move |e: PointerEvent| {
                    if e.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
                        let p = e.client_coordinates();
                        drag.set(Some(KDrag { id, x0: p.x, y0: p.y, dx: 0.0, dy: 0.0 }));
                        over.set(Some(col));
                    }
                },
                button { r#type: "button", class: "oxs12-kcb", "aria-roledescription": "Draggable issue", "aria-pressed": "{kb}",
                    "aria-label": "{key} {t}, {col.label()}. Space to pick up, then arrows to move.",
                    onkeydown: move |e: KeyboardEvent| {
                        let k = e.key();
                        if k == Key::Character(" ".into()) || k == Key::Enter {
                            e.prevent_default();
                            if lift() == Some(id) { lift.set(None); said.set(format!("{} put down", issue_key(id))); } else { lift.set(Some(id)); said.set(format!("{} picked up. Arrows move it, Escape puts it down.", issue_key(id))); }
                        } else if lift() == Some(id) || e.modifiers().alt() {
                            let (dx, dy) = match k { Key::ArrowLeft => (-1, 0), Key::ArrowRight => (1, 0), Key::ArrowUp => (0, -1), Key::ArrowDown => (0, 1), Key::Escape => { lift.set(None); return; } _ => return };
                            e.prevent_default();
                            if let Some(s) = nudge(&mut board.write(), id, dx, dy) { said.set(s); }
                        }
                    },
                    span { class: "oxs12-kch",
                        span { class: "oxs12-key", "{key}" }
                        if let Some(p) = i.pr { span { class: "oxs12-pr", BrandIcon { name: "github".to_string(), size: 12.0, coloured: true } "#{p}" } }
                        if let Some((ds, h)) = due { span { class: "oxs12-due", "data-hue": "{h}", title: "Due {ds}", "aria-label": "Due {ds}", {nico("cal")} "{&ds[5..]}" } }
                        span { class: "oxs12-grow" }
                        {av(i.who, true)}
                    }
                    span { class: "oxs12-kt", title: "{t}", "{t}" }
                    span { class: "oxs12-kf",
                        {prio_bars(i.prio)}
                        {label_chip(i.label)}
                        span { class: "oxs12-grow" }
                        if let Some((a, b)) = i.subs { span { class: "oxs12-n", "data-full": "{a == b}", {ring(a * 100 / b.max(1), if a == b { "green" } else { "mute" })} "{a}/{b}" } }
                        if i.comments > 0 { span { class: "oxs12-n", title: "Comments", {nico("chat")} "{i.comments}" } }
                    }
                }
            }
        }
    };
    let lab_l = lb.map(|l| LABELS[l].1).unwrap_or("Labels");
    let pri_l = pr.map(|p| PRIOS[p].0).unwrap_or("Priority");
    rsx! {
        ArcFrame { slug: "kanban-board", layout, said,
            crumbs: vec!["Brindle".to_string(), "Projects".to_string(), "Storefront v2".to_string()],
            meta: format!("{} issues · {} points", b.len(), total),
            side: rsx! {
                for (i, (_, ic, l, n)) in KB_NAV.iter().copied().enumerate() {
                    { let c = if i == 1 { b.len().to_string() } else { n.to_string() }; nrow(nico(ic), l.to_string(), c, i == 1, EventHandler::new(move |_| said.set(format!("{l} opened")))) }
                }
                {group("Projects", true)}
                for (i, (ic, l, n, h)) in KB_PROJECTS.iter().copied().enumerate() {
                    { let c = if i == 0 { b.len().to_string() } else { n.to_string() }; nrow(hue_ico(h, ic), l.to_string(), c, i == 0, EventHandler::new(move |_| said.set(format!("{l} opened")))) }
                }
            },
            card: rsx! {
                div { class: "oxs12-cyc",
                    div { class: "oxs12-cr1", span { class: "oxs12-hi", "data-hue": "blue", {nico("cycle")} } b { class: "oxs12-ell", "{CYCLE.0}" } span { class: "oxs12-q", "{days_left} days left" } }
                    div { class: "oxs12-prog", role: "progressbar", "aria-valuenow": "{pct}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "Cycle points done", span { style: "width:{pct}%" } }
                    div { class: "oxs12-cr2", span { "{done} of {total} points" } span { "Day {day_n} of {span}" } }
                }
            },
            actions: rsx! {
                {btn("Share", "link", false, EventHandler::new(move |_| said.set("Link to Storefront v2 copied".to_string())))}
                {btn("New issue", "plus", true, EventHandler::new(move |_| { adding.set(Some(Col::Todo)); draft.set(String::new()); }))}
            },
            bar: rsx! {
                {find_field("Filter issues", "Filter issues", q, None)}
                MenuButton { label: pri_l.to_string(), aria: "Priority", icon: "flash", items: pri_items, checked: pr.map(|p| PRIOS[p].0.to_string()).unwrap_or("all".to_string()),
                    on_pick: move |k: String| prio.set(PRIOS.iter().position(|x| x.0 == k)) }
                MenuButton { label: lab_l.to_string(), aria: "Labels", icon: "tag", items: lab_items, checked: lb.map(|l| LABELS[l].0.to_string()).unwrap_or("all".to_string()),
                    on_pick: move |k: String| { let v = LABELS.iter().position(|x| x.0 == k); label.set(v); said.set(match v { Some(l) => format!("Showing {} issues", LABELS[l].1), None => "All labels".to_string() }); } }
                if let Some(l) = lb { span { class: "oxs12-chip", "Label is {LABELS[l].1}" button { r#type: "button", class: "oxs12-chx", "aria-label": "Clear the label filter", onclick: move |_| label.set(None), {nico("close")} } } }
                span { class: "oxs12-grow" }
                {seg("Group by", vec![("status", "Status".to_string()), ("assignee", "Assignee".to_string())], if by_person() { "assignee" } else { "status" }, EventHandler::new(move |k: &'static str| by_person.set(k == "assignee")))}
            },
            if loading {
                div { class: "oxs12-kb", "aria-hidden": "true",
                    for i in 0..5 { div { key: "{i}", class: "oxs12-kcol", span { class: "oxs12-sb oxs12-skh" } for j in 0..(3 - i % 2) { div { key: "{j}", class: "oxs12-kc oxs12-kcsk", {skl(3, "")} } } } }
                }
            } else if b.is_empty() && adding().is_none() {
                {none("kanban", "No issues in this cycle", "Create the first one; drag it across the board as it moves.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| adding.set(Some(Col::Todo)), {nico("plus")} span { class: "oxs12-btl", "New issue" } } })}
            } else if by_person() {
                div { class: "oxs12-kb",
                    for w in assignees(&b) {
                        {
                            let mine: Vec<Issue> = b.iter().filter(|i| i.who == w && i.col != Col::Done).filter(|i| oxyn_core::parity::arcscr::shows(i, &qq, lb)).cloned().collect();
                            let nm = mine.len();
                            rsx! {
                                section { key: "{w}", class: "oxs12-kcol", "aria-label": "{PEOPLE[w].0}, {nm} open issues",
                                    header { class: "oxs12-kh", {av(w, true)} h2 { class: "oxs12-ell", "{PEOPLE[w].0}" } span { class: "oxs12-q", "{nm}" } }
                                    ul { class: "oxs12-kcs",
                                        for i in mine { li { key: "{i.id}", class: "oxs12-kc oxs12-kcp", span { class: "oxs12-kch", {col_glyph(i.col)} span { class: "oxs12-key", "{issue_key(i.id)}" } span { class: "oxs12-grow" } {label_chip(i.label)} } span { class: "oxs12-kt", "{i.title}" } } }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs12-kb",
                    onpointermove: move |e: PointerEvent| {
                        if let Some(mut d) = drag() {
                            let p = e.client_coordinates();
                            let s = stage_scale();
                            d.dx = (p.x - d.x0) / s;
                            d.dy = (p.y - d.y0) / s;
                            drag.set(Some(d));
                        }
                    },
                    onpointerup: move |_| drop_it(),
                    onpointerleave: move |_| if drag.peek().is_some() && own != "dragging" { drag.set(None); over.set(None); },
                    for c in Col::ALL {
                        {
                            let cs = shown(c);
                            let n = counts[c.index()];
                            let lim = c.limit();
                            let overl = lim > 0 && n > lim;
                            let hot = over() == Some(c) && drag().is_some();
                            let cnt = if lim > 0 { format!("{n}/{lim}") } else { n.to_string() };
                            let ltip = if overl { format!("Over the limit of {lim}") } else { String::new() };
                            rsx! {
                                section { key: "{c.key()}", class: "oxs12-kcol", "data-over": "{hot}", "data-col": "{c.key()}", "aria-label": "{c.label()}, {n} issues",
                                    onpointerenter: move |_| if drag.peek().is_some() { over.set(Some(c)) },
                                    header { class: "oxs12-kh",
                                        {col_glyph(c)}
                                        h2 { class: "oxs12-ell", "{c.label()}" }
                                        span { class: "oxs12-q", "data-over": "{overl}", title: "{ltip}", "{cnt}" }
                                        span { class: "oxs12-grow" }
                                        button { r#type: "button", class: "oxs12-ib", "aria-label": "New issue in {c.label()}", onclick: move |_| { adding.set(Some(c)); draft.set(String::new()); }, {nico("plus")} }
                                    }
                                    ul { class: "oxs12-kcs",
                                        for i in cs { {card_el(i)} }
                                    }
                                    if adding() == Some(c) {
                                        div { class: "oxs12-knew",
                                            input { class: "oxs12-in", r#type: "text", placeholder: "Issue title", "aria-label": "New issue title", value: "{draft}",
                                                onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                oninput: move |e| draft.set(e.value()),
                                                onkeydown: move |e: KeyboardEvent| match e.key() { Key::Enter => add(), Key::Escape => { e.stop_propagation(); adding.set(None); } _ => {} } }
                                            div { class: "oxs12-knr",
                                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| adding.set(None), "Cancel" }
                                                button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: draft().trim().len() < 3, onclick: move |_| add(), "Create" }
                                            }
                                        }
                                    } else {
                                        button { r#type: "button", class: "oxs12-kadd", onclick: move |_| { adding.set(Some(c)); draft.set(String::new()); }, {nico("plus")} span { "New issue" } }
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

// ───────────────────────────────────────────────────────────── Project board ──

fn task_glyph(st: u8) -> Element {
    let c = match st {
        1 => Col::Progress,
        2 => Col::Review,
        3 => Col::Done,
        _ => Col::Todo,
    };
    col_glyph(c)
}

/// Screen – Project board: the project's head (status menu, lead, dates, people), four stat
/// tiles, a milestone track that picks the task list, tasks to complete or reopen, Tasks and
/// Activity tabs, and a right card with the team's load and the project's links (copy). States:
/// `sample`, `loading`, `empty`, `atrisk`, `activity`.
#[component]
pub fn ProjectBoardScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// The tasks (`None` = the sample for the state).
    #[props(default)]
    data: Option<Vec<Task>>,
) -> Element {
    let (st, own) = arc_split(&state);
    let mut said = use_signal(String::new);
    let mut list = use_signal(move || {
        data.clone()
            .unwrap_or_else(|| tasks(st == ScreenState::Sample))
    });
    let mut ms = use_signal(|| 2usize);
    let mut tab = use_signal(move || if own == "activity" { 2usize } else { 0 });
    let mut health = use_signal(move || {
        if own == "atrisk" {
            Health::AtRisk
        } else {
            Health::OnTrack
        }
    });
    let mut panel = use_signal(|| true);
    let mut copied = use_signal(|| None::<usize>);
    let mut adding = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut hmenu = use_signal(|| false);
    let loading = st == ScreenState::Loading;
    let empty = st == ScreenState::Empty;
    let t = list();
    let (pct, open, busy, week, late, until) = stats(&t);
    let h = health();
    let mut add = move || {
        let v = draft.peek().trim().to_string();
        if v.len() >= 3 {
            let m = ms();
            list.write().push(Task {
                title: v.clone(),
                ms: m,
                st: 0,
                prio: 0,
                due: MILESTONES[m].1,
                comments: 0,
                who: 0,
            });
            said.set(format!("{v} added to {}", MILESTONES[m].0));
            draft.set(String::new());
            adding.set(false);
        }
    };
    let row = move |i: usize, x: Task| {
        let done = x.st == 3;
        let title = x.title.clone();
        let dh = if !done && x.due < TODAY {
            "rose"
        } else {
            "mute"
        };
        let aria = if done {
            format!("Reopen {title}")
        } else {
            format!("Complete {title}")
        };
        rsx! {
            li { key: "{i}", class: "oxs12-tr", "data-done": "{done}",
                button { r#type: "button", class: "oxs12-tck", role: "checkbox", "aria-checked": "{done}", "aria-label": "{aria}",
                    onclick: move |_| { let w = toggle_task(&mut list.write(), i); if let Some(w) = w { let t = list.peek()[i].title.clone(); said.set(format!("{t} {w}")); } },
                    {task_glyph(x.st)} }
                span { class: "oxs12-ell oxs12-tt", title: "{x.title}", "{x.title}" }
                if x.comments > 0 { span { class: "oxs12-n", {nico("chat")} "{x.comments}" } }
                {prio_bars(x.prio)}
                span { class: "oxs12-due", "data-hue": "{dh}", "{iso(x.due)}" }
                {av(x.who, true)}
            }
        }
    };
    let right = rsx! {
        div { class: "oxs12-rcin",
            div { class: "oxs12-rch", b { "Team" } span { class: "oxs12-q", "{TEAM.len()}" } }
            ul { class: "oxs12-team",
                for (w, role) in TEAM {
                    { let n = if empty { 0 } else { load_of(&t, w) }; let lw = (n * 25).min(100);
                      rsx! { li { key: "{w}", {av(w, false)}
                        span { class: "oxs12-an", b { class: "oxs12-ell", "{PEOPLE[w].0}" if w == PROJECT.2 { span { class: "oxs12-q", " · lead" } } } span { class: "oxs12-ell", "{role}" } }
                        span { class: "oxs12-load", role: "img", "aria-label": "{n} open tasks", span { style: "width:{lw}%" } }
                        span { class: "oxs12-q", "{n}" } } } }
                }
            }
            div { class: "oxs12-rch", b { "Links" } }
            ul { class: "oxs12-links",
                for (i, (bnd, l, s)) in LINKS.iter().copied().enumerate() {
                    li { key: "{i}",
                        span { class: "oxs12-ri", BrandIcon { name: bnd.to_string(), size: 16.0, coloured: true } }
                        span { class: "oxs12-an", b { class: "oxs12-ell", "{l}" } span { class: "oxs12-ell", "{s}" } }
                        button { r#type: "button", class: "oxs12-ib", "aria-label": if copied() == Some(i) { "Copied".to_string() } else { format!("Copy link to {l}") },
                            onclick: move |_| { copied.set(Some(i)); said.set(format!("Link to {l} copied")); },
                            if copied() == Some(i) { {nico("check")} } else { {nico("copy")} } }
                    }
                }
            }
        }
    };
    rsx! {
        ArcFrame { slug: "project-board", layout, said,
            crumbs: vec!["Projects".to_string(), PROJECT.0.to_string()],
            right, show_right: panel(),
            side: rsx! {
                for (i, (_, ic, l, n)) in PB_NAV.iter().copied().enumerate() {
                    {nrow(nico(ic), l.to_string(), n.to_string(), false, EventHandler::new(move |_| said.set(format!("{l} opened"))))}
                    if i == 2 { {group("Projects", true)} }
                }
                for (i, (ic, l, d, m, p)) in PB_PROJECTS.iter().copied().enumerate() {
                    { let pp = if i == 0 && !empty { pct } else { p };
                      let sub = format!("{} · {m}", iso(d));
                      rsx! {
                        button { key: "{i}", r#type: "button", class: "oxs12-nr oxs12-nr2", "aria-current": if i == 0 { "page" } else { "false" }, onclick: move |_| said.set(format!("{l} opened")),
                            span { class: "oxs12-nri", {nico(ic)} }
                            span { class: "oxs12-an", span { class: "oxs12-ell oxs12-nrl", "{l}" } span { class: "oxs12-ell", "{sub}" } }
                            {ring(pp, if i == 0 { "green" } else { "mute" })}
                        } } }
                }
            },
            actions: rsx! {
                {btn("Share", "link", false, EventHandler::new(move |_| said.set("Link to Returns portal copied".to_string())))}
                {btn("New task", "plus", true, EventHandler::new(move |_| { adding.set(true); tab.set(0); }))}
                button { r#type: "button", class: "oxs12-ib oxs12-pt", "aria-pressed": "{panel()}", "aria-label": if panel() { "Hide the side panel" } else { "Show the side panel" }, onclick: move |_| panel.toggle(), {nico("panel")} }
            },
            bar: rsx! {
                div { class: "oxs12-seg", role: "tablist", "aria-label": "Project",
                    for (k, l) in [(0usize, "Overview".to_string()), (1, format!("Tasks {open}")), (2, "Activity".to_string())] {
                        button { key: "{k}", r#type: "button", role: "tab", class: "oxs12-sego", "aria-selected": "{tab() == k}", onclick: move |_| tab.set(k), "{l}" }
                    }
                }
            },
            if loading {
                div { class: "oxs12-pb", {skl(3, "")} div { class: "oxs12-stats", for i in 0..4 { div { key: "{i}", class: "oxs12-stat", {skl(3, "")} } } } {skl(8, "")} }
            } else if tab() == 2 {
                div { class: "oxs12-pb",
                    h2 { class: "oxs12-h2", "Activity" }
                    if empty { p { class: "oxs12-faint", "Nothing has happened on this project yet." } }
                    ul { class: "oxs12-act",
                        for (i, (w, what, d, tm)) in ACTIVITY.iter().copied().enumerate() {
                            if !empty { li { key: "{i}", {av(w, true)} span { class: "oxs12-ell", b { "{PEOPLE[w].0}" } " {what}" } span { class: "oxs12-q", "{iso(d)} {tm}" } } }
                        }
                    }
                }
            } else if tab() == 1 {
                div { class: "oxs12-pb",
                    if t.is_empty() { {none("tasks", "No tasks yet", "Add the first task; it joins the open milestone.", rsx! {})} }
                    for (m, (mn, _, _)) in MILESTONES.iter().copied().enumerate() {
                        { let rows: Vec<(usize, Task)> = t.iter().cloned().enumerate().filter(|(_, x)| x.ms == m).collect();
                          let (d, a) = ms_progress(&t, m);
                          rsx! { if !rows.is_empty() { section { key: "{m}", class: "oxs12-tg",
                              h2 { class: "oxs12-h2", "{mn}" span { class: "oxs12-q", " {d} of {a} done" } }
                              ul { class: "oxs12-tasks", for (i, x) in rows { {row(i, x)} } } } } } }
                    }
                }
            } else {
                div { class: "oxs12-pb",
                    div { class: "oxs12-ph",
                        span { class: "oxs12-pic", "data-hue": "teal", {nico("box")} }
                        h1 { class: "oxs12-ell", "{PROJECT.0}" }
                    }
                    p { class: "oxs12-sum", "{PROJECT.1}" }
                    div { class: "oxs12-pmeta",
                        div { class: "oxs12-hm",
                            button { r#type: "button", class: "oxs12-health", "data-hue": "{h.hue()}", "aria-haspopup": "menu", "aria-expanded": "{hmenu()}", "aria-label": "Status: {h.label()}", onclick: move |_| hmenu.toggle(),
                                span { class: "oxs12-dot" } "{h.label()}" {nico("caret")} }
                            if hmenu() {
                                div { class: "oxs12-catch", onclick: move |_| hmenu.set(false) }
                                div { class: "oxs12-menu oxs12-menud", role: "menu", "aria-label": "Status",
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); hmenu.set(false); },
                                    for x in Health::ALL {
                                        button { key: "{x.key()}", r#type: "button", class: "oxs12-mi", role: "menuitemradio", "aria-checked": "{x == h}",
                                            onclick: move |_| { health.set(x); hmenu.set(false); said.set(format!("Status set to {}", x.label())); },
                                            span { class: "oxs12-health", "data-hue": "{x.hue()}", span { class: "oxs12-dot" } } span { class: "oxs12-ell", "{x.label()}" } }
                                    }
                                }
                            }
                        }
                        span { class: "oxs12-pm", {av(PROJECT.2, true)} span { class: "oxs12-ell", "{PEOPLE[PROJECT.2].0}" } span { class: "oxs12-q", "lead" } }
                        span { class: "oxs12-pm", {nico("cal")} span { class: "oxs12-ell", "{iso(PROJECT.3)} → {iso(PROJECT.4)}" } }
                        span { class: "oxs12-stack", for (w, _) in TEAM.iter().copied().take(5) { {av(w, true)} } span { class: "oxs12-av oxs12-more1", "data-sm": "true", "+1" } }
                    }
                    if h != Health::OnTrack { p { class: "oxs12-note", "data-hue": "{h.hue()}", {nico("flash")} span { "{late} task overdue and {week} due this week; the next milestone is {iso(MILESTONES[2].1)}." } } }
                    div { class: "oxs12-stats",
                        div { class: "oxs12-stat", span { class: "oxs12-q", "Complete" } b { "{pct}%" } span { class: "oxs12-prog", role: "progressbar", "aria-valuenow": "{pct}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "Complete", span { style: "width:{pct}%" } } }
                        div { class: "oxs12-stat", span { class: "oxs12-q", "Open tasks" } b { "{open}" } span { class: "oxs12-q", "{busy} in progress or review" } }
                        div { class: "oxs12-stat", span { class: "oxs12-q", "Due this week" } b { "{week}" } span { class: "oxs12-q", "data-hue": if late > 0 { "rose" } else { "mute" }, "{late} overdue" } }
                        div { class: "oxs12-stat", span { class: "oxs12-q", "Until launch" } b { "{until} days" } span { class: "oxs12-q", "{iso(PROJECT.4)}" } }
                    }
                    h2 { class: "oxs12-h2", "Milestones" span { class: "oxs12-q", " {ms_done(&t)} of {MILESTONES.len()} done" } }
                    div { class: "oxs12-mst", role: "tablist", "aria-label": "Milestones",
                        for (m, (mn, d, _)) in MILESTONES.iter().copied().enumerate() {
                            { let (dn, a) = ms_progress(&t, m); let p = (dn * 100).checked_div(a).unwrap_or(0); let full = a > 0 && dn == a;
                              let line = if full { "Done".to_string() } else if dn > 0 { format!("{p}%") } else { format!("{a} tasks") };
                              rsx! { button { key: "{m}", r#type: "button", role: "tab", class: "oxs12-ms", "aria-selected": "{ms() == m}", onclick: move |_| { ms.set(m); said.set(format!("{mn}: {a} tasks")); },
                                span { class: "oxs12-msb", "data-full": "{full}", span { style: "width:{p}%" } }
                                b { class: "oxs12-ell", "{mn}" } span { class: "oxs12-ell oxs12-q", "{iso(d)}" } span { class: "oxs12-ell oxs12-q", "data-full": "{full}", "{line}" } } } }
                        }
                    }
                    { let m = ms(); let rows: Vec<(usize, Task)> = t.iter().cloned().enumerate().filter(|(_, x)| x.ms == m).collect(); let open_n = rows.iter().filter(|(_, x)| x.st < 3).count();
                      rsx! {
                        div { class: "oxs12-tlh", h2 { class: "oxs12-h2", "{MILESTONES[m].0}" span { class: "oxs12-q", " {open_n} open" } } span { class: "oxs12-grow" }
                            button { r#type: "button", class: "oxs12-more", onclick: move |_| tab.set(1), "All tasks ›" } }
                        if adding() {
                            div { class: "oxs12-knew oxs12-tnew",
                                input { class: "oxs12-in", r#type: "text", placeholder: "Task title", "aria-label": "New task title", value: "{draft}",
                                    onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                    oninput: move |e| draft.set(e.value()),
                                    onkeydown: move |e: KeyboardEvent| match e.key() { Key::Enter => add(), Key::Escape => { e.stop_propagation(); adding.set(false); } _ => {} } }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| adding.set(false), "Cancel" }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: draft().trim().len() < 3, onclick: move |_| add(), "Add task" }
                            }
                        }
                        if rows.is_empty() && !adding() { {none("tasks", "No tasks in this milestone", "Add a task with New task; it lands here.", rsx! {})} }
                        ul { class: "oxs12-tasks", for (i, x) in rows { {row(i, x)} } }
                      } }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── Gantt ──

#[derive(Clone, Copy, PartialEq)]
struct GDrag {
    id: u32,
    /// 0 move, 1 left edge, 2 right edge.
    edge: u8,
    x0: f64,
    dx: f64,
}

const FIRST: i32 = 0;
const DAYS: i32 = 106;
const ROW_H: f64 = 36.0;

fn phase_hue(p: usize) -> &'static str {
    ["teal", "magenta", "blue", "amber"][p.min(3)]
}

/// Screen – Gantt timeline: phases of bars with progress, milestones and dependency arrows (red
/// when a task starts before the one it waits on ends); drag a bar to move it or its edges to
/// change dates, or focus it and use ← / → (Shift for the end); Day / Week / Month zoom, Today,
/// collapse phases, filter and owner, Undo and Reset. States: `sample`, `loading`, `empty`,
/// `dragging`, `month`.
#[component]
pub fn GanttTimelineScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// The bars (`None` = the sample plan for the state).
    #[props(default)]
    data: Option<Vec<Bar>>,
) -> Element {
    let (st, own) = arc_split(&state);
    let mut said = use_signal(String::new);
    let mut plan = use_signal(move || match data.clone() {
        Some(b) => Plan {
            bars: b,
            undo: vec![],
        },
        None => Plan::new(st == ScreenState::Sample),
    });
    let mut zoom = use_signal(move || if own == "month" { 2usize } else { 1 });
    let mut folded = use_signal(|| [false; 4]);
    let q = use_signal(String::new);
    let mut owner = use_signal(|| None::<usize>);
    let mut drag = use_signal(move || {
        if own == "dragging" {
            Some(GDrag {
                id: 10,
                edge: 0,
                x0: 0.0,
                dx: 4.0 * 15.0,
            })
        } else {
            None
        }
    });
    let mut focus = use_signal(|| None::<u32>);
    let mut today_el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let loading = st == ScreenState::Loading;
    let p = plan();
    let ppd = ZOOMS[zoom()].2 as f64;
    let qq = q().to_lowercase();
    let ow = owner();
    let vis = |b: &Bar| {
        (qq.is_empty() || b.name.to_lowercase().contains(&qq)) && ow.is_none_or(|o| b.who == o)
    };
    let bad = conflicts(&p.bars);
    let dg = drag();
    let ddays = |d: &GDrag| (d.dx / ppd).round() as i32;
    // rows: (phase header) then its bars
    let mut rows: Vec<(Option<usize>, Option<Bar>)> = Vec::new();
    for (ph, _) in PHASES.iter().enumerate() {
        let bs: Vec<Bar> = p
            .bars
            .iter()
            .filter(|b| b.phase == ph && vis(b))
            .cloned()
            .collect();
        if bs.is_empty() && !p.bars.is_empty() && (!qq.is_empty() || ow.is_some()) {
            continue;
        }
        rows.push((Some(ph), None));
        if !folded()[ph] {
            for b in bs {
                rows.push((None, Some(b)));
            }
        }
    }
    let width = DAYS as f64 * ppd;
    let xof = |d: i32| (d - FIRST) as f64 * ppd;
    let row_of = |id: u32| {
        rows.iter()
            .position(|r| r.1.as_ref().is_some_and(|b| b.id == id))
    };
    // live geometry of a bar (with the drag applied)
    let geo = |b: &Bar| -> (i32, i32) {
        match dg {
            Some(d) if d.id == b.id => {
                let n = ddays(&d);
                match d.edge {
                    1 => {
                        let n = n.min(b.len - 1);
                        (b.start + n, b.len - n)
                    }
                    2 => (b.start, (b.len + n).max(1)),
                    _ => (b.start + n, b.len),
                }
            }
            _ => (b.start, b.len),
        }
    };
    let mut arrows: Vec<(String, bool)> = Vec::new();
    for (ri, r) in rows.iter().enumerate() {
        if let Some(b) = &r.1 {
            for a in &b.after {
                if let (Some(pi), Some(pb)) = (row_of(*a), p.bars.iter().find(|x| x.id == *a)) {
                    let (ps, pl) = geo(pb);
                    let (bs, _) = geo(b);
                    let x1 = xof(ps + pl.max(1)) - if pl == 0 { ppd / 2.0 } else { 0.0 };
                    let y1 = pi as f64 * ROW_H + ROW_H / 2.0;
                    let x2 = xof(bs) + if b.len == 0 { ppd / 2.0 - 7.0 } else { 0.0 };
                    let y2 = ri as f64 * ROW_H + ROW_H / 2.0;
                    let mx = x1 + 8.0;
                    let d = format!(
                        "M{x1:.1},{y1:.1} H{mx:.1} V{:.1} H{:.1} V{y2:.1} H{x2:.1}",
                        (y1 + y2) / 2.0,
                        (x2 - 8.0).min(mx)
                    );
                    arrows.push((d, bad.contains(&(*a, b.id))));
                }
            }
        }
    }
    let height = rows.len() as f64 * ROW_H;
    // month labels and ticks
    let mut months: Vec<(f64, String)> = Vec::new();
    let mut ticks: Vec<(f64, String, bool)> = Vec::new();
    for d in FIRST..FIRST + DAYS {
        let (m, day) = month_day(d);
        if day == 1 || d == FIRST {
            months.push((xof(d), m));
        }
        let show = match zoom() {
            0 => true,
            1 => weekday(d) == 0,
            _ => day == 1 || day == 15,
        };
        if show {
            ticks.push((xof(d), format!("{day:02}"), weekday(d) >= 5));
        }
    }
    let end_drag = move |_: PointerEvent| {
        if let Some(d) = *drag.peek() {
            let n = (d.dx / ppd).round() as i32;
            let ok = {
                let mut pw = plan.write();
                match d.edge {
                    1 => pw.stretch_start(d.id, n),
                    2 => pw.stretch(d.id, n),
                    _ => pw.shift(d.id, n),
                }
            };
            if ok && let Some(b) = plan.peek().bars.iter().find(|b| b.id == d.id) {
                said.set(format!("{} · {}", b.name, bar_dates(b)));
            }
        }
        drag.set(None);
    };
    let owners: Vec<usize> = {
        let mut v: Vec<usize> = p.bars.iter().map(|b| b.who).collect();
        v.sort();
        v.dedup();
        v
    };
    let own_items: Vec<(&'static str, &'static str, &'static str)> =
        std::iter::once(("all", "Everyone", ""))
            .chain(owners.iter().map(|w| (PEOPLE[*w].0, PEOPLE[*w].0, "")))
            .collect();
    let undo_n = p.undo.len();
    rsx! {
        ArcFrame { slug: "gantt-timeline", layout, said,
            crumbs: vec!["Northgate".to_string(), PLAN.0.to_string(), "Timeline".to_string()],
            meta: format!("{} tasks · {}% done", p.bars.len(), plan_done(&p.bars)),
            side: rsx! {
                for (i, (_, ic, l, n)) in GT_NAV.iter().copied().enumerate() {
                    {nrow(nico(ic), l.to_string(), n.to_string(), i == 0, EventHandler::new(move |_| said.set(format!("{l} opened"))))}
                }
                {group("Milestones", false)}
                for b in p.bars.iter().filter(|b| b.len == 0).cloned() {
                    { let h = phase_hue(b.phase); let nm = b.name; let d = iso(b.start);
                      rsx! { button { key: "{b.id}", r#type: "button", class: "oxs12-nr", onclick: move |_| { focus.set(Some(b.id)); said.set(format!("{nm} · {d}")); },
                        span { class: "oxs12-nri", span { class: "oxs12-dia", "data-hue": "{h}" } }
                        span { class: "oxs12-ell oxs12-nrl", "{b.name}" } span { class: "oxs12-nrc", "{iso(b.start)}" } } } }
                }
                {group("Saved views", false)}
                for (ic, l, n) in GT_VIEWS.iter().copied() { {nrow(nico(ic), l.to_string(), n.to_string(), false, EventHandler::new(move |_| said.set(format!("{l} opened"))))} }
            },
            actions: rsx! {
                {btn("Share", "link", false, EventHandler::new(move |_| said.set("Link to the Harbor 2.0 timeline copied".to_string())))}
                {btn("New task", "plus", true, EventHandler::new(move |_| {
                    let id = plan.peek().bars.iter().map(|b| b.id).max().unwrap_or(0) + 1;
                    let mut pw = plan.write(); let snap = pw.bars.clone(); pw.undo.push(snap); pw.bars.push(Bar { id, name: "New task", phase: 2, start: TODAY, len: 5, done: 0, who: 0, after: vec![] });
                    said.set(format!("New task added to Build, {}", iso(TODAY)));
                }))}
            },
            bar: rsx! {
                {find_field("Filter tasks", "Filter tasks", q, None)}
                MenuButton { label: ow.map(|o| PEOPLE[o].0).unwrap_or("Owner").to_string(), aria: "Owner", icon: "people", caret: true, items: own_items, checked: ow.map(|o| PEOPLE[o].0.to_string()).unwrap_or("all".to_string()),
                    on_pick: move |k: String| owner.set(PEOPLE.iter().position(|x| x.0 == k)) }
                span { class: "oxs12-grow" }
                {seg("Zoom", ZOOMS.iter().map(|z| (z.0, z.1.to_string())).collect(), ZOOMS[zoom()].0, EventHandler::new(move |k: &'static str| { if let Some(i) = ZOOMS.iter().position(|z| z.0 == k) { zoom.set(i); said.set(format!("{} view", ZOOMS[i].1)); } }))}
                button { r#type: "button", class: "oxs12-btn", onclick: move |_| { if let Some(e) = today_el() { spawn(async move { let _ = e.scroll_to(ScrollBehavior::Smooth).await; }); } said.set(format!("Today, {}", iso(TODAY))); },
                    {nico("target")} span { class: "oxs12-btl", "Today" } }
            },
            foot: rsx! {
                span { class: "oxs12-ell oxs12-q", "Drag a bar to move it, or its edges to change dates. Focus a bar: ← → move it, Shift + ← → change its end." }
                span { class: "oxs12-grow" }
                button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", disabled: undo_n == 0, onclick: move |_| { if plan.write().undo() { said.set("Undone".to_string()); } }, {nico("undo")} span { class: "oxs12-btl", "Undo" } }
                button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| { if plan.write().reset() { said.set("Plan reset".to_string()); } }, {nico("refresh")} span { class: "oxs12-btl", "Reset" } }
            },
            if loading {
                div { class: "oxs12-gt", div { class: "oxs12-glst", {skl(12, "")} } div { class: "oxs12-gr", {skl(12, "oxs12-skg")} } }
            } else if p.bars.is_empty() {
                {none("chart", "No tasks on the timeline", "Add a task to start the plan; drag it to set its dates.", rsx! {})}
            } else {
                div { class: "oxs12-gt",
                    div { class: "oxs12-glst", role: "treegrid", "aria-label": "Tasks",
                        div { class: "oxs12-glh", "{p.bars.len()} tasks" }
                        for (ri, r) in rows.iter().cloned().enumerate() {
                            if let Some(ph) = r.0 {
                                { let n = p.bars.iter().filter(|b| b.phase == ph).count(); let f = folded()[ph];
                                  rsx! { button { key: "p{ri}", r#type: "button", class: "oxs12-gph", "aria-expanded": "{!f}", onclick: move |_| folded.write()[ph] = !f,
                                    span { class: "oxs12-chev", "data-open": "{!f}", {nico("next")} } b { class: "oxs12-ell", "{PHASES[ph].1}" } span { class: "oxs12-grow" } span { class: "oxs12-q", "{n}" } } } }
                            } else if let Some(b) = r.1 {
                                { let late = bad.iter().any(|x| x.1 == b.id);
                                  rsx! { div { key: "b{b.id}", class: "oxs12-gtr", role: "row", {av(b.who, true)} span { class: "oxs12-ell", "{b.name}" } if late { span { class: "oxs12-urg", title: "Starts before what it waits on ends", "!" } } span { class: "oxs12-grow" } span { class: "oxs12-q", "{iso(end_of(&b))}" } } } }
                            }
                        }
                    }
                    div { class: "oxs12-gr",
                        onpointermove: move |e: PointerEvent| { if let Some(mut d) = drag() { d.dx = (e.client_coordinates().x - d.x0) / stage_scale(); if own != "dragging" { drag.set(Some(d)); } } },
                        onpointerup: end_drag,
                        onpointerleave: move |_| if own != "dragging" { drag.set(None) },
                        div { class: "oxs12-gw", style: "width:{width:.0}px",
                            div { class: "oxs12-gmh", "aria-hidden": "true",
                                for (i, (x, m)) in months.iter().enumerate() { span { key: "{i}", class: "oxs12-gm", style: "left:{x:.0}px", "{m}" } }
                                for (i, (x, d, _)) in ticks.iter().enumerate() { span { key: "t{i}", class: "oxs12-gd", style: "left:{x:.0}px", "{d}" } }
                            }
                            div { class: "oxs12-gg", style: "height:{height:.0}px",
                                if zoom() < 2 { for (i, (x, _, we)) in ticks.iter().enumerate() { if !*we || zoom() == 0 { span { key: "g{i}", class: "oxs12-gv", "data-we": "{we}", style: "left:{x:.0}px;width:{ppd:.0}px" } } } }
                                span { class: "oxs12-tl", style: "left:{xof(TODAY) + ppd / 2.0:.0}px" }
                                span { class: "oxs12-today", style: "left:{xof(TODAY) + ppd / 2.0:.0}px", onmounted: move |e: MountedEvent| today_el.set(Some(e.data())), "Today {iso(TODAY)}" }
                                svg { class: "oxs12-garr", width: "{width:.0}", height: "{height:.0}", "aria-hidden": "true",
                                    defs { marker { id: "oxs12-ah", "viewBox": "0 0 8 8", "refX": "7", "refY": "4", "markerWidth": "6", "markerHeight": "6", "orient": "auto", path { d: "M0,0 L8,4 L0,8 z", fill: "context-stroke" } } }
                                    for (i, (d, bad)) in arrows.iter().enumerate() { path { key: "{i}", d: "{d}", "data-bad": "{bad}", "marker-end": "url(#oxs12-ah)" } }
                                }
                                for (ri, r) in rows.iter().cloned().enumerate() {
                                    { let top = ri as f64 * ROW_H;
                                      if let Some(ph) = r.0 {
                                        match phase_span(&p.bars, ph) {
                                            Some((a, z, pc)) => { let l = xof(a); let w = xof(z + 1) - l; rsx! { span { key: "s{ri}", class: "oxs12-gsum", style: "top:{top + 15.0:.0}px;left:{l:.0}px;width:{w:.0}px", span { style: "width:{pc}%" } } } }
                                            None => rsx! {},
                                        }
                                      } else if let Some(b) = r.1 {
                                        let (s, len) = geo(&b);
                                        let h = phase_hue(b.phase);
                                        let id = b.id;
                                        let nm = b.name;
                                        let live = dg.is_some_and(|d| d.id == id);
                                        let late = bad.iter().any(|x| x.1 == id);
                                        let tip = if live { let mut c = b.clone(); c.start = s; c.len = len; Some(bar_dates(&c)) } else { None };
                                        let kbd = move |e: KeyboardEvent| {
                                            let k = e.key();
                                            let dd = match k { Key::ArrowLeft => -1, Key::ArrowRight => 1, _ => return };
                                            e.prevent_default();
                                            let ok = { let mut pw = plan.write(); if e.modifiers().shift() { pw.stretch(id, dd) } else { pw.shift(id, dd) } };
                                            if ok && let Some(b) = plan.peek().bars.iter().find(|b| b.id == id) { said.set(format!("{} · {}", b.name, bar_dates(b))); }
                                        };
                                        if b.len == 0 {
                                            let x = xof(s) + ppd / 2.0;
                                            rsx! {
                                                button { key: "m{id}", r#type: "button", class: "oxs12-gms", "data-hue": "{h}", "data-focus": "{focus() == Some(id)}", style: "top:{top:.0}px;left:{x - 7.0:.0}px", "aria-label": "{nm}, milestone {iso(s)}",
                                                    onkeydown: kbd, onclick: move |_| focus.set(Some(id)),
                                                    onpointerdown: move |e: PointerEvent| { let x0 = e.client_coordinates().x; drag.set(Some(GDrag { id, edge: 0, x0, dx: 0.0 })); },
                                                    span { class: "oxs12-dia", "data-hue": "{h}" } b { class: "oxs12-gml", "{nm}" } span { class: "oxs12-q", "{iso(s)}" }
                                                    if let Some(t) = tip { span { class: "oxs12-gtip", role: "tooltip", "{t}" } }
                                                }
                                            }
                                        } else {
                                            let l = xof(s);
                                            let w = len as f64 * ppd;
                                            let inside = w > 120.0;
                                            rsx! {
                                                div { key: "b{id}", class: "oxs12-gb", "data-hue": "{h}", "data-live": "{live}", "data-late": "{late}", "data-focus": "{focus() == Some(id)}", style: "top:{top + 6.0:.0}px;left:{l:.0}px;width:{w:.0}px",
                                                    span { class: "oxs12-gbp", style: "width:{b.done}%" }
                                                    button { r#type: "button", class: "oxs12-gbb", "aria-label": "{nm}, {bar_dates(&b)}, {b.done}% done. Arrows move it; Shift + arrows change its end.",
                                                        onkeydown: kbd, onfocus: move |_| focus.set(Some(id)),
                                                        onpointerdown: move |e: PointerEvent| { let x0 = e.client_coordinates().x; drag.set(Some(GDrag { id, edge: 0, x0, dx: 0.0 })); },
                                                        if inside { span { class: "oxs12-ell oxs12-gbl", "{nm}" } }
                                                        span { class: "oxs12-grow" }
                                                        if w > 44.0 { span { class: "oxs12-gbn", "{b.done}%" } }
                                                    }
                                                    span { class: "oxs12-ge", "data-e": "l", "aria-hidden": "true", onpointerdown: move |e: PointerEvent| { e.stop_propagation(); let x0 = e.client_coordinates().x; drag.set(Some(GDrag { id, edge: 1, x0, dx: 0.0 })); } }
                                                    span { class: "oxs12-ge", "data-e": "r", "aria-hidden": "true", onpointerdown: move |e: PointerEvent| { e.stop_propagation(); let x0 = e.client_coordinates().x; drag.set(Some(GDrag { id, edge: 2, x0, dx: 0.0 })); } }
                                                    if !inside { span { class: "oxs12-gout", "{nm}" } }
                                                    if let Some(t) = tip { span { class: "oxs12-gtip", role: "tooltip", "{t}" } }
                                                }
                                            }
                                        }
                                      } else { rsx! {} } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── Team directory ──

fn avail_el(a: Avail) -> Element {
    rsx! { span { class: "oxs12-avl", "data-hue": "{a.hue()}", {nico(a.icon())} span { class: "oxs12-ell", "{a.words()}" } } }
}

/// Screen – Team directory: people by team with their office's local time and availability;
/// search names, roles and skills (`/` focuses, Escape clears), filter by team, status or office
/// (sidebar), favourite with the star, Grid or List, Invite by email. States: `sample`,
/// `loading`, `empty`, `list`, `noresults`, `inviting`.
#[component]
pub fn TeamDirectoryScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// The people (`None` = the sample for the state).
    #[props(default)]
    data: Option<Vec<Member>>,
) -> Element {
    let (st, own) = arc_split(&state);
    let mut said = use_signal(String::new);
    let all = use_signal(move || {
        data.clone()
            .unwrap_or_else(|| members(st == ScreenState::Sample))
    });
    let mut q = use_signal(move || {
        if own == "noresults" {
            "astrophysics".to_string()
        } else {
            String::new()
        }
    });
    let mut team = use_signal(|| None::<usize>);
    let mut status = use_signal(|| None::<&'static str>);
    let mut office = use_signal(|| None::<usize>);
    let mut list = use_signal(move || own == "list");
    let mut favs = use_signal(|| vec![10usize, 3]);
    let mut inviting = use_signal(move || own == "inviting");
    let mut email = use_signal(move || {
        if own == "inviting" {
            "noor@saltwick.example".to_string()
        } else {
            String::new()
        }
    });
    let mut sent = use_signal(Vec::<String>::new);
    let mut err = use_signal(String::new);
    let search_el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let loading = st == ScreenState::Loading;
    let v = all();
    let hits = find_people(&v, &q(), team(), status(), office());
    let total = v.len();
    let n_off = {
        let mut o: Vec<usize> = v.iter().map(|m| m.office).collect();
        o.sort();
        o.dedup();
        o.len()
    };
    let team_items: Vec<(&'static str, &'static str, &'static str)> =
        std::iter::once(("all", "All teams", ""))
            .chain(TEAMS.iter().map(|t| (t.0, t.1, "")))
            .collect();
    let st_items: Vec<(&'static str, &'static str, &'static str)> =
        std::iter::once(("all", "Any status", ""))
            .chain(STATUSES.iter().map(|s| (s.0, s.1, "")))
            .collect();
    let mut send = move || {
        let e = email.peek().trim().to_string();
        let ok = e.len() > 5
            && e.contains('@')
            && e.rsplit('@').next().is_some_and(|d| d.contains('.'))
            && !e.contains(' ');
        if ok {
            sent.write().push(e.clone());
            said.set(format!("Invite sent to {e}"));
            email.set(String::new());
            err.set(String::new());
            inviting.set(false);
        } else {
            err.set("Enter an email address like name@company.example".to_string());
        }
    };
    let pend = sent().join(", ");
    let focus_search = move |_: ()| {
        if let Some(m) = search_el() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let card = move |i: usize, m: Member, as_row: bool| {
        let fav = favs().contains(&m.who);
        let who = m.who;
        let name = PEOPLE[who].0;
        let city = OFFICES[m.office].1;
        let tm = local_time(m.office);
        let star_l = if fav {
            format!("Remove {name} from favourites")
        } else {
            format!("Add {name} to favourites")
        };
        rsx! {
            li { key: "{i}", class: if as_row { "oxs12-pr2" } else { "oxs12-pc" },
                div { class: "oxs12-pch",
                    {av(who, false)}
                    span { class: "oxs12-an", b { class: "oxs12-ell", "{name}" if m.lead { span { class: "oxs12-lead", "Lead" } } } span { class: "oxs12-ell", "{m.role}" } }
                    button { r#type: "button", class: "oxs12-ib oxs12-star", "aria-pressed": "{fav}", "aria-label": "{star_l}",
                        onclick: move |_| { let mut f = favs.write(); if let Some(p) = f.iter().position(|x| *x == who) { f.remove(p); said.set(format!("{name} removed from favourites")); } else { f.push(who); said.set(format!("{name} added to favourites")); } },
                        {nico("star")} }
                }
                if as_row { span { class: "oxs12-ell oxs12-q oxs12-rt", "{TEAMS[m.team].1}" } }
                div { class: "oxs12-pcf",
                    span { class: "oxs12-ell oxs12-q", "{city} {tm}" }
                    span { class: "oxs12-grow" }
                    {avail_el(m.avail)}
                }
            }
        }
    };
    rsx! {
        ArcFrame { slug: "team-directory", layout, said, search: "Find a person".to_string(), on_search: EventHandler::new(focus_search),
            crumbs: vec!["Saltwick".to_string(), "People".to_string()],
            meta: format!("{total} people in {n_off} offices"),
            side: rsx! {
                for (i, (_, ic, l, n)) in TD_NAV.iter().copied().enumerate() {
                    { let c = if i == 0 { total.to_string() } else { n.to_string() }; nrow(nico(ic), l.to_string(), c, i == 0, EventHandler::new(move |_| said.set(format!("{l} opened")))) }
                }
                {group("Offices", false)}
                for (o, (_, city, _)) in OFFICES.iter().copied().enumerate() {
                    { let (wk, here) = office_count(&v, o); let open = office_open(o); let on = office() == Some(o);
                      rsx! { button { key: "{o}", r#type: "button", class: "oxs12-nr oxs12-nr2", "aria-pressed": "{on}", "aria-label": "{city}, {local_time(o)}, {wk} of {here} working now",
                        onclick: move |_| { office.set(if on { None } else { Some(o) }); said.set(if on { "All offices".to_string() } else { format!("People in {city}") }); },
                        span { class: "oxs12-nri", span { class: "oxs12-hi", "data-hue": if open { "amber" } else { "mute" }, {nico(if open { "sun" } else { "clock" })} } }
                        span { class: "oxs12-an", span { class: "oxs12-ell oxs12-nrl", "{city}" } span { class: "oxs12-ell", "{wk} of {here} working now" } }
                        span { class: "oxs12-nrc", "{local_time(o)}" } } } }
                }
                {group("Favorites", false)}
                for w in favs() { { let nm = PEOPLE[w].0; nrow(av(w, true), nm.to_string(), String::new(), false, EventHandler::new(move |_| { q.set(nm.to_string()); said.set(nm.to_string()); })) } }
            },
            actions: rsx! {
                {btn("Invite", "user", true, EventHandler::new(move |_| { inviting.set(true); err.set(String::new()); }))}
            },
            bar: rsx! {
                {find_field("Search people, roles, skills", "Search people, roles, skills", q, Some(search_el))}
                MenuButton { label: team().map(|t| TEAMS[t].1).unwrap_or("Team").to_string(), aria: "Team", icon: "people", caret: true, items: team_items, checked: team().map(|t| TEAMS[t].0.to_string()).unwrap_or("all".to_string()),
                    on_pick: move |k: String| team.set(TEAMS.iter().position(|t| t.0 == k)) }
                MenuButton { label: status().and_then(|s| STATUSES.iter().find(|x| x.0 == s)).map(|x| x.1).unwrap_or("Status").to_string(), aria: "Status", icon: "pulse", caret: true, items: st_items, checked: status().unwrap_or("all").to_string(),
                    on_pick: move |k: String| status.set(STATUSES.iter().find(|x| x.0 == k).map(|x| x.0)) }
                if let Some(o) = office() { span { class: "oxs12-chip", "Office is {OFFICES[o].1}" button { r#type: "button", class: "oxs12-chx", "aria-label": "Clear the office filter", onclick: move |_| office.set(None), {nico("close")} } } }
                span { class: "oxs12-grow" }
                {seg("View", vec![("grid", "Grid".to_string()), ("list", "List".to_string())], if list() { "list" } else { "grid" }, EventHandler::new(move |k: &'static str| list.set(k == "list")))}
            },
            div { class: "oxs12-td",
                onkeydown: move |e: KeyboardEvent| { if e.key() == Key::Character("/".into()) && let Some(m) = search_el() { e.prevent_default(); spawn(async move { let _ = m.set_focus(true).await; }); } },
                if inviting() {
                    div { class: "oxs12-inv", role: "group", "aria-label": "Invite people",
                        {nico("mail")}
                        input { class: "oxs12-in", r#type: "email", placeholder: "name@company.example", "aria-label": "Email to invite", "aria-invalid": "{!err().is_empty()}", value: "{email}",
                            onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                            oninput: move |e| { email.set(e.value()); err.set(String::new()); },
                            onkeydown: move |e: KeyboardEvent| match e.key() { Key::Enter => send(), Key::Escape => { e.stop_propagation(); inviting.set(false); } _ => {} } }
                        if !err().is_empty() { span { class: "oxs12-err", role: "alert", "{err}" } }
                        span { class: "oxs12-grow" }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| inviting.set(false), "Cancel" }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| send(), "Send invite" }
                    }
                }
                if !sent().is_empty() { p { class: "oxs12-pend", {nico("clock")} span { class: "oxs12-ell", "Pending: {pend}" } } }
                if loading {
                    div { class: "oxs12-pgrid", for i in 0..8 { div { key: "{i}", class: "oxs12-pc", {skl(3, "")} } } }
                } else if v.is_empty() {
                    {none("people", "No one here yet", "Invite your team; they show here with their office and hours.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| inviting.set(true), {nico("user")} span { class: "oxs12-btl", "Invite" } } })}
                } else if hits.is_empty() {
                    {none("search", "No one matches “{q}”", "Try a name, a role or a skill, or clear the filters.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| { q.set(String::new()); team.set(None); status.set(None); office.set(None); }, {nico("close")} span { class: "oxs12-btl", "Clear search" } } })}
                } else {
                    for (t, (_, tl, ic, h)) in TEAMS.iter().copied().enumerate() {
                        { let mine: Vec<usize> = hits.iter().copied().filter(|i| v[*i].team == t).collect();
                          rsx! { if !mine.is_empty() { section { key: "{t}", class: "oxs12-tgp", "aria-label": "{tl}",
                            h2 { class: "oxs12-h2", {hue_ico(h, ic)} "{tl}" span { class: "oxs12-q", " {mine.len()}" } }
                            ul { class: if list() { "oxs12-plist" } else { "oxs12-pgrid" },
                                for i in mine { {card(i, v[i].clone(), list())} }
                            } } } } }
                    }
                }
            }
        }
    }
}
