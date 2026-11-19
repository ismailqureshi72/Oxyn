//! Screens S15 (Arc screens IV, the admin demo app): **Overview**, **Audit log**, **Status**,
//! **API keys and webhooks**, **Members** and **Billing**, and **Agent / Tool calls**, all on the
//! Arc frame shared with S12 (`arcscr.rs`). Rules in `oxyn_core::parity::arcscr4`; styles in
//! `arcscr4.css` (`oxs15-`). Chrome stays monochrome; colour is for statuses, uptime, roles and
//! the official brand and model marks.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::parity::arcscr4::{
    AU_TYPES, Audit, Billing, COMPONENTS, CallSt, HOOK_EVENTS, Keys, Members, Mode, OV_KPIS,
    OV_RANGES, Overview, PAST, PEOPLE, PLANS, ROLES, S15_SLUGS, SCOPES, STATUS_DAYS, ScreenState,
    Sel, Status, TC_AGENT, TC_RANGES, TC_TOOLS, TODAY, ToolCalls, arc4_split, day_health, dur, hm,
    hms, invite_error, iso, member_email, money, money0, open_incident, ov_delta, ov_label,
    ov_point, ov_series, ov_value, pct, pct2, s15_states as core_states, short_n, thousands,
    today_health, uptime,
};

use super::arcscr::{ArcFrame, av, btn, group, none, nrow, seg, skl};
use super::screensfid::nico;
use crate::brand::BrandIcon;

/// The state bar entries of an S15 screen (for a host's `ScreenStage { states }`).
pub fn s15_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    core_states(slug).to_vec()
}

// ───────────────────────────────────────────────────────────── shared parts ──

const NAV: [(&str, &str, &str); 6] = [
    ("admin-overview", "home", "Overview"),
    ("audit-log", "list", "Audit log"),
    ("status-board", "pulse", "Status"),
    ("api-keys-webhooks", "key", "API keys"),
    ("members-roles", "people", "Members"),
    ("billing-usage", "card", "Billing"),
];

fn side(cur: &'static str, said: Signal<String>) -> Element {
    let mut said = said;
    rsx! {
        for (slug, ic, label) in NAV {
            {nrow(nico(ic), label.to_string(), String::new(), slug == cur, EventHandler::new(move |_| said.set(label.to_string())))}
        }
        {group("Agents", true)}
        {nrow(rsx! { BrandIcon { name: "claude".to_string(), size: 14.0 } }, TC_AGENT.0.to_string(), String::new(), cur == "agent-tool-calls", EventHandler::new(move |_| said.set(format!("{} tool calls", TC_AGENT.0))))}
        {group("Recent", false)}
        {nrow(nico("doc"), "Refund policy".to_string(), String::new(), false, EventHandler::new(move |_| said.set("Refund policy".to_string())))}
    }
}

fn plan_card() -> Element {
    rsx! {
        div { class: "oxs15-pc",
            span { class: "oxs12-q", "Team plan · 9 of 10 seats" }
            div { class: "oxs15-meter", "aria-hidden": "true", span { style: "width:90%" } }
            span { class: "oxs12-q", "Renews {iso(TODAY + 18)}" }
        }
    }
}

fn pill(hue: &str, t: &str) -> Element {
    rsx! { span { class: "oxs15-pill", "data-hue": "{hue}", span { class: "oxs12-dot" } "{t}" } }
}

fn crumbs(last: &str) -> Vec<String> {
    vec!["Corvel Labs".to_string(), last.to_string()]
}

fn meter(label: &str, used: String, cap: String, frac: f64) -> Element {
    let w = (frac * 100.0).clamp(0.0, 100.0);
    let hue = if frac >= 0.9 { "amber" } else { "mute" };
    rsx! {
        div { class: "oxs15-use",
            div { class: "oxs15-useh", span { "{label}" } span { class: "oxs12-q oxs14-num", "{used} of {cap}" } }
            div { class: "oxs15-meter", "data-hue": "{hue}", role: "meter", "aria-label": "{label}", "aria-valuenow": "{w:.0}", "aria-valuemin": "0", "aria-valuemax": "100", span { style: "width:{w:.1}%" } }
        }
    }
}

// ───────────────────────────────────────────────────────────── overview ──

/// Screen – Overview: KPI cards over 24h, 7d or 30d (each opens its chart), the chart with a
/// readout per point, component health, recent activity from the audit log and an incident
/// banner to dismiss. States: `sample`, `loading`, `empty`, `week`, `incident`, `errors`.
#[component]
pub fn AdminOverviewScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut ov = use_signal(move || {
        let mut o = Overview::new(st != ScreenState::Empty);
        match own {
            "week" => o.range = 1,
            "incident" => o.incident = true,
            "errors" => {
                o.kpi = 1;
                o.range = 2;
            }
            _ => o.range = 2,
        }
        o
    });
    let o = ov();
    let loading = st == ScreenState::Loading;
    let empty = st == ScreenState::Empty;
    let ser = ov_series(o.kpi, o.range);
    let hi = ser.iter().copied().max().unwrap_or(1).max(1);
    let ranges: Vec<(&'static str, String)> =
        OV_RANGES.iter().map(|r| (r.0, r.0.to_string())).collect();
    let recent = oxyn_core::parity::arcscr4::events(true);
    let n = ser.len();
    let readout = match o.point {
        Some(i) => format!(
            "{}: {}",
            ov_label(o.range, i),
            ov_point(o.kpi, ser[i.min(n - 1)])
        ),
        None => "Press a bar to read it".to_string(),
    };
    let (first, last) = (ov_label(o.range, 0), ov_label(o.range, n.saturating_sub(1)));
    rsx! {
        ArcFrame { slug: "admin-overview", layout, said,
            crumbs: crumbs("Overview"),
            meta: format!("Updated {} 09:41 UTC", iso(TODAY)),
            side: side("admin-overview", said),
            card: plan_card(),
            actions: rsx! {
                {seg("Range", ranges, OV_RANGES[o.range].0, EventHandler::new(move |k: &'static str| { if let Some(i) = OV_RANGES.iter().position(|r| r.0 == k) { let r = ov.write().set_range(i); said.set(r); } }))}
                {btn("Export", "download", false, EventHandler::new(move |_| said.set(format!("Exported overview-{}.csv", iso(TODAY)))))}
            },
            if loading {
                div { class: "oxs15-pad", div { class: "oxs15-kpis", for i in 0..4 { div { key: "{i}", class: "oxs12-card oxs15-kpi", {skl(3, "")} } } } div { class: "oxs12-card oxs15-chart", {skl(8, "")} } }
            } else if empty {
                div { class: "oxs15-pad",
                    {none("chart", "No traffic yet", "Send your first API request and the numbers land here within a minute.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| said.set("Create an API key".to_string()), {nico("key")} span { class: "oxs12-btl", "Create an API key" } } })}
                }
            } else {
                div { class: "oxs15-pad",
                    if o.incident {
                        div { class: "oxs14-ban", "data-hue": "amber", role: "alert",
                            {nico("bell")}
                            span { class: "oxs12-grow", "Webhook deliveries are delayed (since 09:14 UTC). Events are queued, not lost." }
                            button { r#type: "button", class: "oxs14-lnk", onclick: move |_| said.set("Status opened".to_string()), "View status" }
                            button { r#type: "button", class: "oxs12-ib", "aria-label": "Dismiss", onclick: move |_| { ov.write().incident = false; said.set("Incident banner dismissed".to_string()); }, {nico("close")} }
                        }
                    }
                    div { class: "oxs15-kpis", role: "tablist", "aria-label": "Metrics",
                        for (k, (_, label)) in OV_KPIS.iter().copied().enumerate() {
                            { let (bp, good) = ov_delta(k, o.range); let hue = if good { "green" } else { "rose" }; let on = k == o.kpi;
                              rsx! { button { key: "{k}", r#type: "button", role: "tab", class: "oxs12-card oxs15-kpi", "aria-selected": "{on}",
                                onclick: move |_| { let r = ov.write().set_kpi(k); said.set(r); },
                                span { class: "oxs12-q", "{label}" }
                                b { class: "oxs14-num", "{ov_value(k, o.range)}" }
                                span { class: "oxs14-dl", "data-hue": "{hue}", "{pct(bp)}" span { class: "oxs12-q", " vs first half" } }
                              } } }
                        }
                    }
                    section { class: "oxs12-card oxs15-chart", "aria-label": "{OV_KPIS[o.kpi].1} chart",
                        div { class: "oxs15-ch",
                            h2 { class: "oxs12-h2", "{OV_KPIS[o.kpi].1} · last {OV_RANGES[o.range].0}" }
                            span { class: "oxs12-grow" }
                            span { class: "oxs12-q", role: "status", "{readout}" }
                        }
                        div { class: "oxs15-bars",
                            for (i, v) in ser.iter().copied().enumerate() {
                                { let h = (v as f64 / hi as f64 * 100.0).max(3.0); let on = o.point == Some(i);
                                  rsx! { button { key: "{i}", r#type: "button", class: "oxs15-bar", "aria-pressed": "{on}", "aria-label": "{ov_label(o.range, i)}: {ov_point(o.kpi, v)}",
                                    onclick: move |_| { let r = ov.write().read(i); said.set(r); },
                                    span { style: "height:{h:.1}%" } } } }
                            }
                        }
                        div { class: "oxs15-axis", span { "{first}" } span { "{last}" } }
                    }
                    div { class: "oxs15-two",
                        section { class: "oxs12-card oxs15-box",
                            h2 { class: "oxs12-h2", "Components" }
                            ul { class: "oxs15-ul",
                                for (c, (_, name, _)) in COMPONENTS.iter().copied().enumerate() {
                                    { let h = today_health(c, if o.incident { "degraded" } else { "" }); let up = pct2(uptime(c, ""));
                                      rsx! { li { key: "{c}", span { class: "oxs12-ell oxs12-grow", "{name}" } span { class: "oxs12-q oxs14-num", "{up}" } {pill(h.hue(), h.word())} } } }
                                }
                            }
                        }
                        section { class: "oxs12-card oxs15-box",
                            h2 { class: "oxs12-h2", "Recent activity" }
                            ul { class: "oxs15-ul",
                                for e in recent.iter().take(6).cloned() {
                                    li { key: "{e.id}",
                                        if let Some(a) = e.actor { {av(a, true)} } else { span { class: "oxs15-sys", {nico("bot")} } }
                                        span { class: "oxs12-an oxs12-grow", b { class: "oxs12-ell", "{e.action}" } span { class: "oxs12-ell", "{e.who()} · {e.target}" } }
                                        span { class: "oxs12-q oxs14-num", "{hm(e.at)}" }
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

// ───────────────────────────────────────────────────────────── audit ──

/// Screen – Audit log: every action in the workspace with its time (UTC, ISO), who, what, where
/// from and the result; filter by type and failures, search, open an event for its before and
/// after, show more and export. States: `sample`, `loading`, `empty`, `filtered`, `detail`,
/// `nomatch`.
#[component]
pub fn AuditLogScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut au = use_signal(move || {
        let mut a = Audit::new(st != ScreenState::Empty);
        match own {
            "filtered" => a.kind = 2,
            "detail" => a.open = Some(1),
            "nomatch" => a.q = "staging-cluster".into(),
            _ => {}
        }
        a
    });
    let a = au();
    let list = a.list();
    let total = a.matching().len();
    let cur = a.current();
    let loading = st == ScreenState::Loading;
    let q = a.q.clone();
    rsx! {
        ArcFrame { slug: "audit-log", layout, said,
            crumbs: crumbs("Audit log"),
            meta: "Kept for 365 days · times in UTC".to_string(),
            side: side("audit-log", said),
            card: plan_card(),
            actions: rsx! {
                {btn("Export CSV", "download", false, EventHandler::new(move |_| { let r = au.read().export(); said.set(r); }))}
            },
            right: rsx! {
                if let Some(e) = cur.clone() {
                    div { class: "oxs15-rp", role: "region", "aria-label": "Event details",
                        div { class: "oxs15-rph", h2 { class: "oxs12-h2 oxs12-grow", "{e.action}" } button { r#type: "button", class: "oxs12-ib", "aria-label": "Close event", onclick: move |_| { au.write().open = None; said.set("Event closed".to_string()); }, {nico("close")} } }
                        {pill(if e.ok { "green" } else { "rose" }, if e.ok { "Succeeded" } else { "Blocked" })}
                        dl { class: "oxs15-dl",
                            dt { "When" } dd { class: "oxs14-num", "{e.when()}" }
                            dt { "Actor" } dd { "{e.who()}" }
                            dt { "Target" } dd { "{e.target}" }
                            dt { "Type" } dd { "{AU_TYPES[e.kind].1}" }
                            dt { "IP address" } dd { class: "oxs14-num", "{e.ip}" }
                        }
                        h3 { class: "oxs12-q", "Change" }
                        pre { class: "oxs15-code", span { class: "oxs15-del", "- {e.before}" } "\n" span { class: "oxs15-add", "+ {e.after}" } }
                    }
                }
            },
            show_right: cur.is_some(),
            bar: rsx! {
                div { class: "oxs15-fbar",
                    for (k, (_, l)) in AU_TYPES.iter().copied().enumerate() {
                        button { key: "{k}", r#type: "button", class: "oxs12-chip", "aria-pressed": "{a.kind == k}", onclick: move |_| { let r = au.write().set_kind(k); said.set(r); }, "{l}" }
                    }
                    button { r#type: "button", class: "oxs12-chip", "aria-pressed": "{a.failed}", onclick: move |_| { let mut w = au.write(); w.failed = !w.failed; said.set(if w.failed { "Only failures".to_string() } else { "All results".to_string() }); }, "Failures only" }
                    span { class: "oxs12-grow" }
                    label { class: "oxs12-find",
                        {nico("search")}
                        input { r#type: "search", placeholder: "Search actor, action, IP", "aria-label": "Search events", value: "{q}",
                            oninput: move |e| au.write().q = e.value(),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); au.write().q.clear(); } }
                    }
                }
            },
            if loading {
                div { class: "oxs15-pad", {skl(10, "")} }
            } else if a.all.is_empty() {
                div { class: "oxs15-pad", {none("list", "Nothing logged yet", "Sign-ins, key changes, member changes and billing events land here as they happen.", rsx! {})} }
            } else if list.is_empty() {
                div { class: "oxs15-pad", {none("search", "No events match", "Try another type or search, or clear the filters.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { let r = au.write().clear(); said.set(r); }, "Clear filters" } })} }
            } else {
                div { class: "oxs15-pad",
                    p { class: "oxs12-q", role: "status", "{total} events" if a.filtered() { button { r#type: "button", class: "oxs14-lnk", onclick: move |_| { let r = au.write().clear(); said.set(r); }, "Clear filters" } } }
                    table { class: "oxs14-t oxs15-t",
                        thead { tr { th { "Time (UTC)" } th { "Actor" } th { "Action" } th { class: "oxs14-hide", "Target" } th { class: "oxs15-hide", "IP address" } th { "Result" } } }
                        tbody {
                            for e in list.iter().cloned() {
                                { let id = e.id; let on = a.open == Some(id);
                                  rsx! { tr { key: "{id}", "aria-selected": "{on}", class: "oxs15-tr",
                                    td { class: "oxs14-num oxs15-time", "{iso(e.day)} {hms(e.at)}" }
                                    td { span { class: "oxs15-who", if let Some(x) = e.actor { {av(x, true)} } else { span { class: "oxs15-sys", {nico("bot")} } } span { class: "oxs12-ell", "{e.who()}" } } }
                                    td { button { r#type: "button", class: "oxs15-rowb", "aria-expanded": "{on}", "aria-label": "{e.action} {e.target}, {iso(e.day)} {hms(e.at)}", onclick: move |_| { let r = au.write().toggle(id); said.set(r); }, "{e.action}" } }
                                    td { class: "oxs14-hide oxs15-ellc", span { class: "oxs12-ell", "{e.target}" } }
                                    td { class: "oxs15-hide oxs14-num", "{e.ip}" }
                                    td { {pill(if e.ok { "green" } else { "rose" }, if e.ok { "OK" } else { "Blocked" })} }
                                  } } }
                            }
                        }
                    }
                    if list.len() < total {
                        button { r#type: "button", class: "oxs12-btn oxs12-sm oxs15-more", onclick: move |_| { if let Some(r) = au.write().more() { said.set(r); } }, "Show more ({total - list.len()})" }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── status ──

/// Screen – Status: the overall state, each component with 90 days of uptime (press a day for
/// its record), the open incident with its updates, past incidents and Subscribe. States:
/// `sample`, `loading`, `empty`, `degraded`, `outage`, `maint`.
#[component]
pub fn StatusBoardScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut s = use_signal(move || Status::new(st != ScreenState::Empty, own));
    let v = s();
    let (word, hue) = v.overall();
    let inc = open_incident(v.mode);
    let loading = st == ScreenState::Loading;
    let avg = pct2((0..6).map(|c| uptime(c, v.mode)).sum::<i64>() / 6);
    let start = iso(TODAY - 89);
    let readout = v.day.map(|d| {
        format!(
            "{} · {}",
            iso(TODAY - (STATUS_DAYS - 1 - d) as i32),
            day_health(v.comp, d, v.mode).word()
        )
    });
    rsx! {
        ArcFrame { slug: "status-board", layout, said,
            crumbs: crumbs("Status"),
            meta: format!("status.corvel.example · checked {} 09:41 UTC", iso(TODAY)),
            side: side("status-board", said),
            card: plan_card(),
            actions: rsx! {
                button { r#type: "button", class: "oxs12-btn", "data-pri": "{!v.subscribed}", "aria-pressed": "{v.subscribed}", onclick: move |_| { let r = s.write().subscribe(); said.set(r); }, {nico(if v.subscribed { "check" } else { "bell" })} span { class: "oxs12-btl", if v.subscribed { "Subscribed" } else { "Subscribe" } } }
            },
            if loading {
                div { class: "oxs15-pad", {skl(4, "")} {skl(8, "")} }
            } else if !v.sample {
                div { class: "oxs15-pad", {none("pulse", "No components yet", "Add the parts of your product people depend on, and their uptime shows here.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| said.set("Add a component".to_string()), {nico("plus")} span { class: "oxs12-btl", "Add component" } } })} }
            } else {
                div { class: "oxs15-pad",
                    div { class: "oxs15-hero", "data-hue": "{hue}", role: "status", span { class: "oxs15-hdot" } b { "{word}" } span { class: "oxs12-grow" } span { class: "oxs12-q", "90-day uptime {avg}" } }
                    if let Some((title, comp, h, ups)) = inc {
                        section { class: "oxs12-card oxs15-inc", "data-hue": "{h.hue()}",
                            div { class: "oxs15-ch", {pill(h.hue(), h.word())} h2 { class: "oxs12-h2", "{title}" } span { class: "oxs12-grow" } span { class: "oxs12-q", "{COMPONENTS[comp].1}" } }
                            ol { class: "oxs15-ups",
                                for (i, (t, stage, text)) in ups.iter().copied().enumerate() {
                                    li { key: "{i}", b { "{stage}" } span { class: "oxs12-q oxs14-num", "{iso(TODAY)} {t} UTC" } p { "{text}" } }
                                }
                            }
                        }
                    }
                    section { class: "oxs12-card oxs15-box", "aria-label": "Components",
                        for (c, (_, name, what)) in COMPONENTS.iter().copied().enumerate() {
                            { let now = today_health(c, v.mode); let up = pct2(uptime(c, v.mode)); let rd = if v.comp == c { readout.clone() } else { None };
                              rsx! { div { key: "{c}", class: "oxs15-comp",
                                div { class: "oxs15-ch", span { class: "oxs12-an", b { "{name}" } span { class: "oxs12-ell", "{what}" } } span { class: "oxs12-grow" } span { class: "oxs12-q oxs14-num oxs15-hide", "{up}" } {pill(now.hue(), now.word())} }
                                div { class: "oxs15-days", role: "group", "aria-label": "{name}, last 90 days",
                                    for d in 0..STATUS_DAYS {
                                        { let h = day_health(c, d, v.mode); let on = v.day == Some(d) && v.comp == c;
                                          let date = iso(TODAY - (STATUS_DAYS - 1 - d) as i32);
                                          rsx! { button { key: "{d}", r#type: "button", class: "oxs15-day", "data-h": "{h.key()}", "aria-pressed": "{on}", "aria-label": "{name} {date}: {h.word()}", tabindex: if d + 1 == STATUS_DAYS { "0" } else { "-1" },
                                            onclick: move |_| { let r = s.write().read(c, d); said.set(r); } } } }
                                    }
                                }
                                div { class: "oxs15-axis", span { "{start}" } if let Some(r) = rd { span { class: "oxs15-read", role: "status", "{r}" } } span { "Today" } }
                              } } }
                        }
                    }
                    section { class: "oxs12-card oxs15-box",
                        h2 { class: "oxs12-h2", "Past incidents" }
                        ul { class: "oxs15-ul",
                            for (i, (c, ago, h, title, mins)) in PAST.iter().copied().enumerate() {
                                { let open = v.past == Some(i); let when = iso(TODAY - ago as i32); let cname = COMPONENTS[c].1;
                                  rsx! { li { key: "{i}", class: "oxs15-pi",
                                    button { r#type: "button", class: "oxs15-rowb oxs12-grow", "aria-expanded": "{open}", onclick: move |_| { let mut w = s.write(); w.past = if w.past == Some(i) { None } else { Some(i) }; said.set(title.to_string()); },
                                        {pill(h.hue(), h.word())} span { class: "oxs12-ell", "{title}" } }
                                    span { class: "oxs12-q oxs14-num", "{when}" }
                                    if open { p { class: "oxs15-pd", "{cname} · {mins} minutes · resolved. We added an alert so we see it before you do." } }
                                  } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── keys ──

/// Screen – API keys and webhooks: keys with scope, created and last used (ISO); New key with a
/// name and scope, the secret shown once with Copy; Roll and Revoke (press twice). Webhooks:
/// endpoints with their events and success rate, deliveries with status codes, Send test,
/// Retry and turn off. States: `sample`, `loading`, `empty`, `created`, `hooks`, `failing`.
#[component]
pub fn ApiKeysWebhooksScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut ak = use_signal(move || {
        let mut k = Keys::new(st != ScreenState::Empty);
        match own {
            "created" => {
                k.draft = Some(("support-bot".into(), 1));
                let _ = k.create();
            }
            "hooks" => k.tab = 1,
            "failing" => {
                k.tab = 1;
                if let Some(h) = k.hooks.get_mut(0) {
                    for d in h.deliveries.iter_mut().take(3) {
                        d.code = 500;
                    }
                }
            }
            _ => {}
        }
        k
    });
    let k = ak();
    let loading = st == ScreenState::Loading;
    let check = k.check();
    let tabs: Vec<(&'static str, String)> = vec![
        ("keys", format!("API keys ({})", k.keys.len())),
        ("hooks", format!("Webhooks ({})", k.hooks.len())),
    ];
    let hook = k.hooks.get(k.hook).cloned();
    rsx! {
        ArcFrame { slug: "api-keys-webhooks", layout, said,
            crumbs: crumbs(if k.tab == 0 { "API keys" } else { "Webhooks" }),
            meta: "Developers".to_string(),
            side: side("api-keys-webhooks", said),
            card: plan_card(),
            actions: rsx! {
                div { class: "oxs15-hseg", {seg("Section", tabs, if k.tab == 0 { "keys" } else { "hooks" }, EventHandler::new(move |x: &'static str| { ak.write().tab = if x == "keys" { 0 } else { 1 }; said.set(if x == "keys" { "API keys".to_string() } else { "Webhooks".to_string() }); }))} }
                if k.tab == 0 { {btn("New key", "plus", true, EventHandler::new(move |_| { let r = ak.write().open_new(); said.set(r); }))} }
                else { {btn("Add endpoint", "plus", true, EventHandler::new(move |_| said.set("Add endpoint".to_string())))} }
            },
            if loading {
                div { class: "oxs15-pad", {skl(8, "")} }
            } else if k.tab == 0 {
                div { class: "oxs15-pad",
                    if let Some(sec) = k.secret.clone() {
                        section { class: "oxs12-card oxs15-secret", role: "alert",
                            div { class: "oxs15-ch", {nico("key")} b { "Copy your secret now" } span { class: "oxs12-q", "It will not show again." } }
                            div { class: "oxs15-sec", code { class: "oxs14-num", "{sec}" }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { ak.write().copied = true; said.set("Secret copied".to_string()); }, {nico(if k.copied { "check" } else { "copy" })} span { class: "oxs12-btl", if k.copied { "Copied" } else { "Copy" } } }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { ak.write().secret = None; said.set("Secret hidden for good".to_string()); }, "Done" }
                            }
                        }
                    }
                    if let Some((name, scope)) = k.draft.clone() {
                        form { class: "oxs12-card oxs15-form", "aria-label": "New API key", onsubmit: move |e: FormEvent| { e.prevent_default(); let r = ak.write().create(); said.set(match r { Ok(x) | Err(x) => x }); },
                            label { class: "oxs15-fl", span { class: "oxs12-q", "Name" }
                                input { class: "oxs15-in", r#type: "text", placeholder: "e.g. support-bot", value: "{name}", "aria-invalid": "{check.is_some() && !name.is_empty()}", oninput: move |e| { if let Some(d) = ak.write().draft.as_mut() { d.0 = e.value(); } } } }
                            div { class: "oxs15-fl", span { class: "oxs12-q", "Scope" }
                                div { class: "oxs12-seg", role: "radiogroup", "aria-label": "Scope",
                                    for (i, (_, l)) in SCOPES.iter().copied().enumerate() {
                                        button { key: "{i}", r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{scope == i}", onclick: move |_| { if let Some(d) = ak.write().draft.as_mut() { d.1 = i; } }, "{l}" }
                                    }
                                }
                            }
                            if let (Some(e), false) = (check, name.is_empty()) { p { class: "oxs14-err", role: "alert", {nico("help")} span { "{e}" } } }
                            div { class: "oxs15-ch", span { class: "oxs12-grow" }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { ak.write().draft = None; said.set("Cancelled".to_string()); }, "Cancel" }
                                button { r#type: "submit", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: check.is_some(), "Create key" }
                            }
                        }
                    }
                    if k.keys.is_empty() {
                        {none("key", "No API keys", "Create a key to call the API from your servers. Keep it secret; roll it if it leaks.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| { let r = ak.write().open_new(); said.set(r); }, {nico("plus")} span { class: "oxs12-btl", "New key" } } })}
                    } else {
                        table { class: "oxs14-t oxs15-t",
                            thead { tr { th { "Name" } th { "Key" } th { class: "oxs14-hide", "Scope" } th { class: "oxs15-hide", "Created" } th { class: "oxs14-hide", "Last used" } th { class: "oxs14-r", "Actions" } } }
                            tbody {
                                for x in k.keys.iter().cloned() {
                                    { let id = x.id; let ask = k.confirm == Some(id); let roll_l = format!("Roll {}", x.name); let rev_l = if ask { format!("Confirm revoke {}", x.name) } else { format!("Revoke {}", x.name) };
                                      rsx! { tr { key: "{id}",
                                        td { span { class: "oxs15-who", {av(x.by, true)} b { class: "oxs12-ell", "{x.name}" } } }
                                        td { code { class: "oxs15-mono", "ox_live_…{x.tail}" } }
                                        td { class: "oxs14-hide", {pill(if x.scope == 2 { "amber" } else { "mute" }, SCOPES[x.scope].1)} }
                                        td { class: "oxs15-hide oxs14-num", "{iso(x.created)}" }
                                        td { class: "oxs14-hide oxs14-num", if let Some(u) = x.used { "{iso(u)}" } else { span { class: "oxs12-q", "Never" } } }
                                        td { class: "oxs14-r", div { class: "oxs15-acts",
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-label": "{roll_l}", onclick: move |_| { if let Some(r) = ak.write().roll(id) { said.set(r); } }, {nico("refresh")} span { class: "oxs12-btl", "Roll" } }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-danger": "{ask}", "aria-label": "{rev_l}", onclick: move |_| { if let Some(r) = ak.write().revoke(id) { said.set(r); } }, if ask { "Revoke?" } else { "Revoke" } }
                                        } }
                                      } } }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs15-pad",
                    if k.hooks.is_empty() {
                        {none("link", "No endpoints", "Add a URL and pick events; we POST them signed, and retry for 3 days.", rsx! {})}
                    } else {
                        div { class: "oxs15-wh",
                            ul { class: "oxs15-whl", "aria-label": "Endpoints",
                                for (i, h) in k.hooks.iter().cloned().enumerate() {
                                    { let on = i == k.hook; let fail = h.failing();
                                      rsx! { li { key: "{h.id}", button { r#type: "button", class: "oxs15-whb", "aria-current": "{on}", onclick: move |_| { ak.write().hook = i; said.set(h.url.to_string()); },
                                        span { class: "oxs12-an", b { class: "oxs12-ell oxs15-mono", "{h.url}" } span { class: "oxs12-ell", "{h.events.len()} events · {h.success()}% delivered" } }
                                        if !h.on { {pill("mute", "Off")} } else if fail { {pill("rose", "Failing")} } else { {pill("green", "On")} }
                                      } } } }
                                }
                            }
                            if let Some(h) = hook {
                                section { class: "oxs12-card oxs15-box",
                                    div { class: "oxs15-ch", b { class: "oxs12-ell oxs15-mono", "{h.url}" } span { class: "oxs12-grow" }
                                        button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { if let Some(r) = ak.write().test() { said.set(r); } }, {nico("send")} span { class: "oxs12-btl", "Send test" } }
                                        button { r#type: "button", role: "switch", class: "oxs15-sw", "aria-checked": "{h.on}", "aria-label": "Endpoint on", onclick: move |_| { if let Some(r) = ak.write().toggle() { said.set(r); } }, span {} }
                                    }
                                    if h.failing() { p { class: "oxs14-ban", "data-hue": "rose", role: "alert", {nico("help")} span { "The last deliveries failed with 500. We retry for 3 days, then turn the endpoint off." } } }
                                    div { class: "oxs15-tags", for e in h.events.iter().copied() { code { key: "{e}", class: "oxs15-tag", "{HOOK_EVENTS[e]}" } } }
                                    h3 { class: "oxs12-q", "Recent deliveries" }
                                    if h.deliveries.is_empty() { p { class: "oxs12-q", "Nothing sent yet." } }
                                    ul { class: "oxs15-ul",
                                        for d in h.deliveries.iter().cloned() {
                                            { let id = d.id; rsx! { li { key: "{id}",
                                                {pill(if d.ok() { "green" } else { "rose" }, &d.code_word())}
                                                code { class: "oxs15-mono oxs12-grow oxs12-ell", "{HOOK_EVENTS[d.event]}" }
                                                span { class: "oxs12-q oxs14-num", "{dur(d.ms)} · {iso(TODAY)} {hms(d.at)}" }
                                                if !d.ok() { button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { if let Some(r) = ak.write().retry(id) { said.set(r); } }, "Retry" } }
                                            } } }
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

// ───────────────────────────────────────────────────────────── members ──

/// Screen – Members: people with role, two-step sign-in and last active (ISO); Invite with an
/// email and role (checked, seats counted), pending invites with Resend and Revoke, change a
/// role (the last owner stays), Remove (press twice), filter by role. States: `sample`,
/// `loading`, `empty`, `invite`, `full`, `admins`.
#[component]
pub fn MembersRolesScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut mb = use_signal(move || {
        let mut m = Members::new(st != ScreenState::Empty, own == "full");
        match own {
            "invite" => {
                m.form = true;
                m.email = "sana@corvel".into();
            }
            "full" => m.form = true,
            "admins" => m.filter = Some(1),
            _ => {}
        }
        m
    });
    let m = mb();
    let loading = st == ScreenState::Loading;
    let err = if m.email.is_empty() && m.used() < m.seats {
        None
    } else {
        invite_error(&m.email, &m)
    };
    let shown = m.shown();
    let filters: Vec<(&'static str, String)> = vec![
        ("all", "Everyone".into()),
        ("1", "Admins".into()),
        ("2", "Members".into()),
        ("3", "Viewers".into()),
    ];
    let fkey = match m.filter {
        Some(1) => "1",
        Some(2) => "2",
        Some(3) => "3",
        _ => "all",
    };
    rsx! {
        ArcFrame { slug: "members-roles", layout, said,
            crumbs: crumbs("Members"),
            meta: format!("{} of {} seats", m.used(), m.seats),
            side: side("members-roles", said),
            card: plan_card(),
            actions: rsx! {
                div { class: "oxs15-hseg", {seg("Role", filters, fkey, EventHandler::new(move |x: &'static str| { let mut w = mb.write(); w.filter = x.parse().ok(); said.set(if x == "all" { "Everyone".to_string() } else { format!("{}s", ROLES[x.parse::<usize>().unwrap_or(2)]) }); }))} }
                {btn("Invite", "plus", true, EventHandler::new(move |_| { let mut w = mb.write(); w.form = !w.form; said.set("Invite a member".to_string()); }))}
            },
            if loading {
                div { class: "oxs15-pad", {skl(9, "")} }
            } else {
                div { class: "oxs15-pad",
                    {meter("Seats", m.used().to_string(), m.seats.to_string(), m.used() as f64 / m.seats.max(1) as f64)}
                    if m.form {
                        form { class: "oxs12-card oxs15-form", "aria-label": "Invite a member", onsubmit: move |e: FormEvent| { e.prevent_default(); let r = mb.write().send(); said.set(match r { Ok(x) | Err(x) => x }); },
                            div { class: "oxs15-inv",
                                label { class: "oxs15-fl oxs12-grow", span { class: "oxs12-q", "Email" }
                                    input { class: "oxs15-in", r#type: "email", placeholder: "name@company.com", value: "{m.email}", "aria-invalid": "{err.is_some()}", oninput: move |e| mb.write().email = e.value() } }
                                div { class: "oxs15-fl", span { class: "oxs12-q", "Role" }
                                    div { class: "oxs12-seg", role: "radiogroup", "aria-label": "Invite role",
                                        for r in 1..4usize { button { key: "{r}", r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{m.irole == r}", onclick: move |_| mb.write().irole = r, "{ROLES[r]}" } }
                                    }
                                }
                            }
                            if let Some(e) = err { p { class: "oxs14-err", role: "alert", {nico("help")} span { "{e}" } } }
                            div { class: "oxs15-ch", span { class: "oxs12-q", "They get an email with a link that works for 7 days." } span { class: "oxs12-grow" }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| mb.write().form = false, "Cancel" }
                                button { r#type: "submit", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: err.is_some() || m.email.is_empty(), "Send invite" }
                            }
                        }
                    }
                    if m.list.len() <= 1 && m.invites.is_empty() {
                        {none("people", "Just you so far", "Invite your team; admins manage keys and billing, members build, viewers read.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| mb.write().form = true, {nico("plus")} span { class: "oxs12-btl", "Invite people" } } })}
                    }
                    table { class: "oxs14-t oxs15-t oxs15-mt",
                        thead { tr { th { "Name" } th { "Role" } th { class: "oxs14-hide", "Two-step" } th { class: "oxs15-hide", "Last active" } th { class: "oxs14-r", "Actions" } } }
                        tbody {
                            for (i, x) in shown.iter().cloned() {
                                { let ask = m.confirm == Some(i); let name = PEOPLE[x.who % PEOPLE.len()].0;
                                  let rh = match x.role { 0 => "violet", 1 => "blue", _ => "mute" }; let rm_l = if ask { format!("Confirm remove {name}") } else { format!("Remove {name}") };
                                  rsx! { tr { key: "{x.who}",
                                    td { span { class: "oxs15-who", {av(x.who, false)} span { class: "oxs12-an", b { class: "oxs12-ell", "{name}" } span { class: "oxs12-ell", "{member_email(x.who)}" } } } }
                                    td { button { r#type: "button", class: "oxs15-role", "data-hue": "{rh}", "aria-label": "{name} is {ROLES[x.role]}; change role", onclick: move |_| { if let Some(r) = mb.write().cycle_role(i) { said.set(r); } }, "{ROLES[x.role]}" {nico("updown")} } }
                                    td { class: "oxs14-hide", if x.mfa { {pill("green", "On")} } else { {pill("amber", "Off")} } }
                                    td { class: "oxs15-hide oxs14-num", "{iso(x.active)}" }
                                    td { class: "oxs14-r", if x.role != 0 { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-danger": "{ask}", "aria-label": "{rm_l}", onclick: move |_| { if let Some(r) = mb.write().remove(i) { said.set(r); } }, if ask { "Remove?" } else { "Remove" } } } else { span { class: "oxs12-q", "Owner" } } }
                                  } } }
                            }
                        }
                    }
                    if !m.invites.is_empty() {
                        h2 { class: "oxs12-h2 oxs15-h", "Pending invites ({m.invites.len()})" }
                        ul { class: "oxs15-ul oxs12-card oxs15-box",
                            for v in m.invites.iter().cloned() {
                                { let id = v.id; rsx! { li { key: "{id}",
                                    span { class: "oxs15-sys", {nico("mail")} }
                                    span { class: "oxs12-an oxs12-grow", b { class: "oxs12-ell", "{v.email}" } span { class: "oxs12-ell", "{ROLES[v.role]} · sent {iso(v.sent)}" } }
                                    button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { if let Some(r) = mb.write().resend(id) { said.set(r); } }, "Resend" }
                                    button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-label": "Revoke invite to {v.email}", onclick: move |_| { if let Some(r) = mb.write().revoke(id) { said.set(r); } }, "Revoke" }
                                } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── billing ──

/// Screen – Billing: the plan with renewal, Monthly or Yearly, three plans with the prorated
/// change to confirm (refused when seats do not fit), usage meters, the payment card and
/// invoices with download; a past-due banner with Retry. States: `sample`, `loading`, `empty`,
/// `yearly`, `change`, `pastdue`.
#[component]
pub fn BillingUsageScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut bl = use_signal(move || {
        let mut b = Billing::new(st != ScreenState::Empty);
        match own {
            "yearly" => b.yearly = true,
            "change" => {
                let _ = b.pick(2);
            }
            "pastdue" => {
                if let Some(i) = b.invoices.first_mut() {
                    i.status = 2;
                }
            }
            _ => {}
        }
        b
    });
    let b = bl();
    let loading = st == ScreenState::Loading;
    let cur = PLANS[b.plan];
    let cycles: Vec<(&'static str, String)> =
        vec![("m", "Monthly".into()), ("y", "Yearly · save 17%".into())];
    rsx! {
        ArcFrame { slug: "billing-usage", layout, said,
            crumbs: crumbs("Billing"),
            meta: format!("Period {} – {}", iso(TODAY - 12), iso(TODAY + 18)),
            side: side("billing-usage", said),
            card: plan_card(),
            actions: rsx! {
                {seg("Billing cycle", cycles, if b.yearly { "y" } else { "m" }, EventHandler::new(move |x: &'static str| { if (x == "y") != bl.read().yearly { let r = bl.write().toggle_cycle(); said.set(r); } }))}
            },
            if loading {
                div { class: "oxs15-pad", div { class: "oxs15-plans", for i in 0..3 { div { key: "{i}", class: "oxs12-card oxs15-plan", {skl(4, "")} } } } {skl(6, "")} }
            } else {
                div { class: "oxs15-pad",
                    if b.past_due() {
                        div { class: "oxs14-ban", "data-hue": "rose", role: "alert", {nico("card")}
                            span { class: "oxs12-grow", "Your last payment failed on {iso(TODAY - 1)}. Update the card or retry before {iso(TODAY + 6)} to keep the Team plan." }
                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| { if let Some(r) = bl.write().retry() { said.set(r); } }, "Retry payment" }
                        }
                    }
                    div { class: "oxs15-plans", role: "radiogroup", "aria-label": "Plans",
                        for (i, (name, _, _, seats, calls)) in PLANS.iter().copied().enumerate() {
                            { let on = i == b.plan; let picked = b.pick == Some(i); let price = b.price(i);
                              rsx! { div { key: "{i}", class: "oxs12-card oxs15-plan", "data-on": "{on}", "data-pick": "{picked}",
                                div { class: "oxs15-ch", b { "{name}" } span { class: "oxs12-grow" } if on { {pill("green", "Current")} } }
                                p { class: "oxs15-price", b { class: "oxs14-num", "{money0(price)}" } span { class: "oxs12-q", " / month" } }
                                p { class: "oxs12-q", if b.yearly && price > 0 { "Billed {money0(price * 12)} yearly" } else if price > 0 { "Billed monthly" } else { "Free forever" } }
                                ul { class: "oxs15-feat", li { "{seats} seats" } li { "{short_n(calls)} API calls a month" } li { if i == 0 { "7-day audit log" } else { "365-day audit log" } } }
                                button { r#type: "button", role: "radio", class: "oxs12-btn oxs15-pb", "aria-checked": "{on}", "data-pri": "{picked}", disabled: on, onclick: move |_| { if let Some(r) = bl.write().pick(i) { said.set(r); } }, if on { "Your plan" } else if i > b.plan { "Upgrade" } else { "Downgrade" } }
                              } } }
                        }
                    }
                    if let Some(p) = b.pick {
                        { let c = b.prorate(p);
                          rsx! { section { class: "oxs12-card oxs15-conf", role: "dialog", "aria-label": "Change plan",
                            div { class: "oxs12-an oxs12-grow", b { "Change to {PLANS[p].0}" }
                                span { if c >= 0 { "{money(c)} today for the {b.left} days left, then {money0(b.price(p))} a month from {iso(TODAY + 18)}." } else { "{money(-c)} credit on your next invoice; {money0(b.price(p))} a month from {iso(TODAY + 18)}." } } }
                            button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { bl.write().pick = None; said.set("Plan unchanged".to_string()); }, "Cancel" }
                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| { if let Some(r) = bl.write().confirm() { said.set(r); } }, "Confirm change" }
                          } } }
                    }
                    div { class: "oxs15-two",
                        section { class: "oxs12-card oxs15-box",
                            h2 { class: "oxs12-h2", "Usage this period" }
                            {meter("Seats", b.seats.to_string(), cur.3.to_string(), b.seats as f64 / cur.3 as f64)}
                            {meter("API calls", short_n(b.calls), short_n(cur.4), b.calls as f64 / cur.4 as f64)}
                            {meter("Storage", format!("{}.{} GB", b.storage / 1000, (b.storage % 1000) / 100), "50 GB".to_string(), b.storage as f64 / 50_000.0)}
                        }
                        section { class: "oxs12-card oxs15-box",
                            h2 { class: "oxs12-h2", "Payment method" }
                            if b.plan == 0 && b.invoices.is_empty() {
                                p { class: "oxs12-q", "No card on file. Add one when you upgrade." }
                            } else {
                                div { class: "oxs15-card",
                                    BrandIcon { name: "visa".to_string(), size: 28.0 }
                                    span { class: "oxs12-an oxs12-grow", b { "Visa ending 4242" } span { "Expires 2028-04" } }
                                    button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| said.set("Update card".to_string()), "Update" }
                                }
                            }
                            p { class: "oxs12-q", "Invoices go to billing@corvel.example" }
                        }
                    }
                    section { class: "oxs12-card oxs15-box",
                        h2 { class: "oxs12-h2", "Invoices" }
                        if b.invoices.is_empty() { p { class: "oxs12-q", "No invoices yet. The Starter plan is free." } }
                        ul { class: "oxs15-ul",
                            for (i, v) in b.invoices.iter().cloned().enumerate() {
                                li { key: "{v.no}",
                                    code { class: "oxs15-mono", "{v.no}" }
                                    span { class: "oxs12-q oxs14-num oxs12-grow", "{iso(v.day)}" }
                                    b { class: "oxs14-num", "{money(v.cents)}" }
                                    {match v.status { 0 => pill("green", "Paid"), 1 => pill("amber", "Open"), _ => pill("rose", "Failed") }}
                                    button { r#type: "button", class: "oxs12-ib", "aria-label": "Download {v.no}", onclick: move |_| said.set(format!("Downloaded {}.pdf", bl.read().invoices.get(i).map(|x| x.no.clone()).unwrap_or_default())), {nico("download")} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── tool calls ──

fn tool_logo(t: usize, size: f32) -> Element {
    let b = TC_TOOLS[t.min(TC_TOOLS.len() - 1)].2;
    rsx! { span { class: "oxs15-logo", "aria-hidden": "true", BrandIcon { name: b.to_string(), size } } }
}

fn json(s: &str) -> String {
    // Pretty-print a flat JSON object: one key per line.
    let t = s.trim();
    if !(t.starts_with('{') && t.ends_with('}')) {
        return t.to_string();
    }
    let inner = &t[1..t.len() - 1];
    let mut out = String::from("{\n");
    let mut depth = 0i32;
    let mut q = false;
    let mut cur = String::new();
    for ch in inner.chars() {
        match ch {
            '"' => q = !q,
            '{' | '[' if !q => depth += 1,
            '}' | ']' if !q => depth -= 1,
            _ => {}
        }
        if ch == ',' && !q && depth == 0 {
            out.push_str(&format!("  {},\n", cur.trim()));
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    if !cur.trim().is_empty() {
        out.push_str(&format!("  {}\n", cur.trim()));
    }
    out.push('}');
    out.replace("\":", "\": ")
}

/// Screen – Agent / Tool calls: one agent and its tools as an org chart (the live call lights
/// its line), the calls table with time, tool, status, duration, tokens and input; open the
/// agent (Pause, Connect tool), a tool (Allowed / Ask first / Paused, its parameters,
/// Disconnect pressed twice) or a call (input and output, Approve or Reject a waiting one, Run
/// again for a failed one); Today, 7 days or 30 days. The agent keeps calling with the clock.
/// States: `sample`, `loading`, `empty`, `waiting`, `failed`, `paused`.
#[component]
pub fn AgentToolCallsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// The host clock (the agent takes a step each second).
    #[props(default)]
    now: ClockMs,
    /// Run the clock.
    #[props(default = true)]
    live: bool,
) -> Element {
    let (st, own) = arc4_split(&state);
    let mut said = use_signal(String::new);
    let mut tc = use_signal(move || ToolCalls::new(st != ScreenState::Empty, own));
    let mut applied = use_signal(|| None::<u64>);
    use_effect(use_reactive!(|now, live| {
        let t = now.0 / 1000;
        let a = *applied.peek();
        match a {
            None => applied.set(Some(t)),
            Some(a) if live && t > a => {
                let mut w = tc.write();
                for _ in 0..(t - a).min(3) {
                    if let Some(s) = w.step() {
                        said.set(s);
                    }
                }
                drop(w);
                applied.set(Some(t));
            }
            _ => {}
        }
    }));
    let t = tc();
    let loading = st == ScreenState::Loading;
    let shown = t.shown();
    let (n, nf, nw, tok) = t.totals();
    let live_tool = t.live();
    let ranges: Vec<(&'static str, String)> =
        TC_RANGES.iter().map(|r| (r.0, r.0.to_string())).collect();
    let sel = t.sel;
    let open_call = match sel {
        Some(Sel::Call(id)) => t.calls.iter().find(|c| c.id == id).cloned(),
        _ => None,
    };
    let (agent, task, model, maker) = TC_AGENT;
    let row_a: Vec<(usize, Mode)> = t
        .tools
        .iter()
        .copied()
        .take(t.tools.len().div_ceil(2))
        .collect();
    let row_b: Vec<(usize, Mode)> = t
        .tools
        .iter()
        .copied()
        .skip(t.tools.len().div_ceil(2))
        .collect();
    let card = move |ti: usize, m: Mode, t: &ToolCalls| -> Element {
        let (name, f, _, src, _, _, _) = TC_TOOLS[ti];
        let (c, fl, avg) = t.stats(ti);
        let on = t.sel == Some(Sel::Tool(ti));
        let lit = live_tool == Some(ti);
        rsx! { li { key: "{ti}", class: "oxs15-tl", "data-live": "{lit}",
            button { r#type: "button", class: "oxs15-tool", "aria-pressed": "{on}", "data-mode": "{m.key()}", "aria-label": "{name}, {src}, {m.word()}, {c} calls, {fl} failed",
                onclick: move |_| { let r = tc.write().select(Sel::Tool(ti)); said.set(r); },
                div { class: "oxs15-ch", {tool_logo(ti, 16.0)} b { class: "oxs12-ell", "{name}" } span { class: "oxs12-grow" } if lit { span { class: "oxs15-calling", span { class: "oxs12-dot" } "Calling" } } }
                code { class: "oxs15-mono oxs12-ell", "{f}" }
                div { class: "oxs15-ch", span { class: "oxs15-src", "{src}" } span { class: "oxs12-grow" } span { class: "oxs15-mode", "data-mode": "{m.key()}", "{m.word()}" } }
                div { class: "oxs15-ch oxs12-q", span { class: "oxs14-num", "{c} calls" } if fl > 0 { span { class: "oxs15-bad", "{fl} failed" } } span { class: "oxs12-grow" } if avg > 0 { span { class: "oxs14-num", "{dur(avg)}" } } }
            }
        } }
    };
    let tref = t.clone();
    let panel: Element = match sel {
        Some(Sel::Agent) => rsx! {
            div { class: "oxs15-rph", BrandIcon { name: maker.to_string(), size: 18.0 } h2 { class: "oxs12-h2 oxs12-grow", "{agent}" } button { r#type: "button", class: "oxs12-ib", "aria-label": "Close", onclick: move |_| tc.write().sel = None, {nico("close")} } }
            p { class: "oxs12-q", "{task}" }
            dl { class: "oxs15-dl", dt { "Model" } dd { "{model}" } dt { "Tools" } dd { "{tref.tools.len()} connected" } dt { "Calls" } dd { class: "oxs14-num", "{n} in range" } dt { "Tokens" } dd { class: "oxs14-num", "{thousands(tok as u64)}" } }
            div { class: "oxs15-ch",
                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { let r = tc.write().pause(); said.set(r); }, if tref.paused { "Resume" } else { "Pause agent" } }
                button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| tc.write().picker = true, "Connect tool" }
            }
        },
        Some(Sel::Tool(ti)) => {
            let (name, f, _, src, desc, _, params) = TC_TOOLS[ti];
            let m = tref.mode(ti).unwrap_or(Mode::Off);
            let ask = tref.disc == Some(ti);
            rsx! {
                div { class: "oxs15-rph", {tool_logo(ti, 18.0)} h2 { class: "oxs12-h2 oxs12-grow", "{name}" } button { r#type: "button", class: "oxs12-ib", "aria-label": "Close", onclick: move |_| tc.write().sel = None, {nico("close")} } }
                p { class: "oxs12-q", "{desc} · {src}" }
                code { class: "oxs15-mono", "{f}" }
                div { class: "oxs15-fl", span { class: "oxs12-q", "May run" }
                    div { class: "oxs12-seg", role: "radiogroup", "aria-label": "Access",
                        for mm in [Mode::Auto, Mode::Ask, Mode::Off] {
                            button { key: "{mm.key()}", r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{m == mm}", onclick: move |_| { if let Some(r) = tc.write().set_mode(mm) { said.set(r); } }, "{mm.word()}" }
                        }
                    }
                }
                h3 { class: "oxs12-q", "Parameters" }
                ul { class: "oxs15-ul", for (pn, pt, req) in params.iter().copied() { li { key: "{pn}", code { class: "oxs15-mono oxs12-grow", "{pn}" } span { class: "oxs12-q", "{pt}" } if req { {pill("mute", "required")} } } } }
                button { r#type: "button", class: "oxs12-btn oxs12-sm oxs15-disc", "data-danger": "{ask}", onclick: move |_| { if let Some(r) = tc.write().disconnect() { said.set(r); } }, if ask { "Press again to disconnect" } else { "Disconnect" } }
            }
        }
        Some(Sel::Call(_)) => {
            if let Some(c) = open_call.clone() {
                rsx! {
                    div { class: "oxs15-rph", {tool_logo(c.tool, 18.0)} h2 { class: "oxs12-h2 oxs12-grow oxs15-mono", "{TC_TOOLS[c.tool].1}" } button { r#type: "button", class: "oxs12-ib", "aria-label": "Close", onclick: move |_| tc.write().sel = None, {nico("close")} } }
                    div { class: "oxs15-ch", {pill(c.st.hue(), c.st.word())} span { class: "oxs12-q oxs14-num", "{c.when()} · {c.run}" } }
                    if c.st == CallSt::Waiting {
                        div { class: "oxs15-ask", role: "alert",
                            p { "{agent} wants to run {TC_TOOLS[c.tool].1}. {TC_TOOLS[c.tool].0} asks first." }
                            div { class: "oxs15-ch",
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| { if let Some(r) = tc.write().approve() { said.set(r); } }, "Approve" }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { if let Some(r) = tc.write().reject() { said.set(r); } }, "Reject" }
                            }
                        }
                    }
                    h3 { class: "oxs12-q", "Input" }
                    pre { class: "oxs15-code", "{json(&c.input)}" }
                    if let Some(o) = c.output.clone() { h3 { class: "oxs12-q", "Output · {c.tokens} tokens" } pre { class: "oxs15-code", "{json(&o)}" } }
                    if let Some(e) = c.error.clone() { h3 { class: "oxs12-q", "Error" } pre { class: "oxs15-code", "data-err": "true", "{e}" } }
                    if c.st == CallSt::Failed {
                        div { class: "oxs15-ask", "data-hue": "rose",
                            p { if c.tool == 5 { "The bot is not in that channel. Run it again in #support, where it is a member." } else { "Run the call again with the same input." } }
                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| { if let Some(r) = tc.write().solve() { said.set(r); } }, {nico("refresh")} span { class: "oxs12-btl", "Run again" } }
                        }
                    }
                    if c.st == CallSt::Running { p { class: "oxs12-q", role: "status", "Running for {dur(c.ms)}…" } }
                }
            } else {
                rsx! {}
            }
        }
        None => rsx! {},
    };
    rsx! {
        ArcFrame { slug: "agent-tool-calls", layout, said,
            crumbs: vec!["Agents".to_string(), agent.to_string(), "Tool calls".to_string()],
            meta: if t.paused { "Paused".to_string() } else { format!("Live · {}", iso(TODAY)) },
            side: side("agent-tool-calls", said),
            card: plan_card(),
            actions: rsx! {
                div { class: "oxs15-hseg", {seg("Range", ranges, TC_RANGES[t.range].0, EventHandler::new(move |k: &'static str| { if let Some(i) = TC_RANGES.iter().position(|r| r.0 == k) { tc.write().range = i; said.set(k.to_string()); } }))} }
                button { r#type: "button", class: "oxs12-btn", "aria-pressed": "{t.paused}", onclick: move |_| { let r = tc.write().pause(); said.set(r); }, {nico(if t.paused { "flash" } else { "pause" })} span { class: "oxs12-btl", if t.paused { "Resume agent" } else { "Pause agent" } } }
                {btn("Connect tool", "plug", true, EventHandler::new(move |_| { let mut w = tc.write(); w.picker = !w.picker; said.set("Connect a tool".to_string()); }))}
            },
            right: rsx! { div { class: "oxs15-rp", role: "region", "aria-label": "Details", {panel} } },
            show_right: sel.is_some() && !loading,
            if loading {
                div { class: "oxs15-pad", div { class: "oxs15-org", div { class: "oxs12-card oxs15-agent", {skl(3, "")} } div { class: "oxs15-tools", for i in 0..4 { div { key: "{i}", class: "oxs12-card oxs15-tool", {skl(3, "")} } } } } {skl(6, "")} }
            } else {
                div { class: "oxs15-pad",
                    if t.picker {
                        div { class: "oxs12-card oxs15-picker", role: "dialog", "aria-label": "Connect a tool",
                            div { class: "oxs15-ch", b { "Connect a tool" } span { class: "oxs12-grow" } button { r#type: "button", class: "oxs12-ib", "aria-label": "Close", onclick: move |_| tc.write().picker = false, {nico("close")} } }
                            if t.unconnected().is_empty() { p { class: "oxs12-q", "Every tool in the catalogue is connected." } }
                            ul { class: "oxs15-ul",
                                for ti in t.unconnected() {
                                    li { key: "{ti}", {tool_logo(ti, 16.0)} span { class: "oxs12-an oxs12-grow", b { "{TC_TOOLS[ti].0}" } span { class: "oxs12-ell", "{TC_TOOLS[ti].4}" } }
                                        button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { if let Some(r) = tc.write().connect(ti) { said.set(r); } }, "Connect" } }
                                }
                            }
                        }
                    }
                    section { class: "oxs15-org", "aria-label": "{agent} and its tools", "data-paused": "{t.paused}",
                        button { r#type: "button", class: "oxs12-card oxs15-agent", "aria-pressed": "{sel == Some(Sel::Agent)}", onclick: move |_| { let r = tc.write().select(Sel::Agent); said.set(r); },
                            span { class: "oxs15-amark", BrandIcon { name: maker.to_string(), size: 20.0 } }
                            span { class: "oxs12-an", b { "{agent}" } span { class: "oxs12-ell", "{model}" } }
                            span { class: "oxs12-grow" }
                            if t.paused { {pill("amber", "Paused")} } else if live_tool.is_some() { {pill("blue", "Working")} } else { {pill("green", "Idle")} }
                        }
                        if t.tools.is_empty() {
                            div { class: "oxs15-stem" }
                            button { r#type: "button", class: "oxs15-slot", onclick: move |_| tc.write().picker = true, {nico("plus")} "Connect the first tool" }
                        } else {
                            div { class: "oxs15-stem" }
                            ul { class: "oxs15-tools", "data-row": "1", for (ti, m) in row_a.iter().copied() { {card(ti, m, &t)} } }
                            if !row_b.is_empty() { div { class: "oxs15-stem2" } ul { class: "oxs15-tools", "data-row": "2", for (ti, m) in row_b.iter().copied() { {card(ti, m, &t)} } } }
                        }
                    }
                    div { class: "oxs15-sum", role: "status",
                        span { b { class: "oxs14-num", "{n}" } " calls" } span { b { class: "oxs14-num", "{nf}" } " failed" } span { b { class: "oxs14-num", "{nw}" } " waiting" } span { b { class: "oxs14-num", "{thousands(tok as u64)}" } " tokens" }
                    }
                    if shown.is_empty() {
                        {none("bolt", "No calls yet", "Connect a tool and the agent's calls show here with what went in and came back.", rsx! { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| tc.write().picker = true, {nico("plug")} span { class: "oxs12-btl", "Connect tool" } } })}
                    } else {
                        table { class: "oxs14-t oxs15-t", "aria-label": "Calls",
                            thead { tr { th { "Time" } th { "Tool" } th { "Status" } th { class: "oxs14-r", "Duration" } th { class: "oxs14-r oxs14-hide", "Tokens" } th { class: "oxs15-hide", "Input" } th { class: "oxs15-hide", "Run" } } }
                            tbody {
                                for c in shown.iter().cloned() {
                                    { let id = c.id; let on = sel == Some(Sel::Call(id));
                                      rsx! { tr { key: "{id}", class: "oxs15-tr", "aria-selected": "{on}", "data-st": "{c.st.key()}",
                                        td { class: "oxs14-num oxs15-time", if c.ago == 0 { "{hms(c.at)}" } else { "{c.when()}" } }
                                        td { button { r#type: "button", class: "oxs15-rowb", "aria-label": "{TC_TOOLS[c.tool].1} at {c.when()}, {c.st.word()}", onclick: move |_| { let r = tc.write().select(Sel::Call(id)); said.set(r); }, {tool_logo(c.tool, 14.0)} span { class: "oxs15-mono oxs12-ell", "{TC_TOOLS[c.tool].1}" } } }
                                        td { {pill(c.st.hue(), c.st.word())} }
                                        td { class: "oxs14-r oxs14-num", if c.ms > 0 { "{dur(c.ms)}" } else { "—" } }
                                        td { class: "oxs14-r oxs14-num oxs14-hide", if c.tokens > 0 { "{thousands(c.tokens as u64)}" } else { "—" } }
                                        td { class: "oxs15-hide oxs15-ellc", code { class: "oxs15-mono oxs12-ell", "{c.input}" } }
                                        td { class: "oxs15-hide", code { class: "oxs15-mono", "{c.run}" } }
                                      } } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The S15 slugs (for hosts).
pub fn s15_slugs() -> &'static [&'static str] {
    &S15_SLUGS
}
