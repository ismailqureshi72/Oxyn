//! Plan #28 parity demos, tool calls: tool call card and group, tool call transcript rows,
//! tool chips, JSON viewer, expandable card, link unfurl and the tool-kind map.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::tools::{
        CallCard, CallStatus, Json, JsonTree, Preview, Unfurl, UnfurlState, find_urls,
    },
};
use oxyn_dioxus::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};

use super::{Code, mk, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

macro_rules! keyed {
    ($sig:ident, $key:expr, $build:expr) => {
        let key = $key;
        let mut $sig = use_signal(|| $build);
        let mut k = use_signal(|| key.clone());
        if k() != key {
            k.set(key.clone());
            $sig.set($build);
        }
    };
}

fn ago(now: ClockMs, ms: u64) -> Option<ClockMs> {
    Some(ClockMs(now.0.saturating_sub(ms)))
}

// ---------- tool call card ----------

const ARGS: &str =
    r#"{"query":"usage-based pricing pages","limit":5,"site":["stripe.com","vercel.com"]}"#;
const RESULT: &str = r#"{"results":[{"title":"Stripe Billing pricing","url":"https://stripe.com/billing/pricing","score":0.94},{"title":"Vercel pricing","url":"https://vercel.com/pricing","score":0.91},{"title":"Linear pricing","url":"https://linear.app/pricing","score":0.88},{"title":"Notion pricing","url":"https://notion.so/pricing","score":0.81},{"title":"Figma pricing","url":"https://figma.com/pricing","score":0.77}],"took_ms":1180}"#;

fn sample_card(status: &str, open: bool, now: ClockMs) -> CallCard {
    let mut c = CallCard::new("search_web", "usage-based pricing pages");
    c.args = Some(ARGS.into());
    c.open = open;
    match status {
        "success" => {
            c.status = CallStatus::Success;
            c.result = Some(RESULT.into());
            c.duration_ms = Some(1_180);
        }
        "error" => {
            c.started = ago(now, 3_000);
            c.fail("Request timed out after 3.0s (search provider 504)", 3_000);
            if !open {
                c.open = true;
            }
        }
        _ => c.started = ago(now, 1_400),
    }
    c
}

/// A call started (retried) after the demo mounted settles on its own after 2.6 s, so Retry
/// plays out; the preset running calls keep running.
fn settle(mut c: CallCard, now: ClockMs, t0: ClockMs) -> CallCard {
    if c.status == CallStatus::Running
        && let Some(s) = c.started
        && s.0 >= t0.0
        && now.0.saturating_sub(s.0) > 2_600
    {
        c.status = CallStatus::Success;
        c.result = Some(RESULT.into());
        c.duration_ms = Some(2_600);
    }
    c
}

fn card(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        LiveCard {
            status: v.string("status"),
            view: v.string("view"),
            open: v.b("open"),
            cap: v.f("cap") as u32,
            now,
        }
    }
}

#[component]
fn LiveCard(status: String, view: String, open: bool, cap: u32, now: ClockMs) -> Element {
    keyed!(c, (status.clone(), open), sample_card(&status, open, now));
    keyed!(g, view.clone(), {
        let mut a = CallCard::new("read_file", "src/billing/meter.rs");
        a.status = CallStatus::Success;
        a.duration_ms = Some(120);
        a.args = Some(r#"{"path":"src/billing/meter.rs"}"#.into());
        a.result = Some("pub struct Meter { events: Vec<Event> }".into());
        let mut b = sample_card("success", false, now);
        b.name = "grep".into();
        b.summary = "\"invoice\" in src/".into();
        b.duration_ms = Some(640);
        let mut d = sample_card("error", false, now);
        d.name = "bash".into();
        d.summary = "cargo test billing".into();
        d.error = Some("2 tests failed: proration_rounds_down, refunds_cap".into());
        let mut e = CallCard::new("edit", "src/billing/invoice.rs");
        e.started = ago(now, 900);
        vec![a, b, d, e]
    });
    let mut gopen = use_signal(|| true);
    keyed!(t0, (status.clone(), open, view.clone()), now);
    let cards: Vec<CallCard> = g().into_iter().map(|x| settle(x, now, t0())).collect();
    rsx! {
        div { class: "oxd-w520",
            if view == "group" {
                ToolCallGroup {
                    cards,
                    now,
                    open: gopen(),
                    on_open: move |o| gopen.set(o),
                    on_toggle: move |i: usize| g.write()[i].toggle(),
                    on_retry: move |i: usize| g.write()[i].retry(now),
                }
            } else {
                ToolCallCard {
                    card: settle(c(), now, t0()),
                    now,
                    icon: "globe".to_string(),
                    max_code: cap,
                    on_toggle: move |_| c.write().toggle(),
                    on_retry: EventHandler::new(move |_| c.write().retry(now)),
                }
            }
        }
    }
}

fn card_code(v: &Vals) -> String {
    if v.string("view") == "group" {
        return Code::new("ToolCallGroup")
            .raw("cards", "calls()")
            .raw("now", "clock.now()")
            .raw("on_toggle", "move |i| calls.write()[i].toggle()")
            .raw(
                "on_retry",
                "move |i| { calls.write()[i].retry(now); agent.retry(i) }",
            )
            .done();
    }
    Code::new("ToolCallCard")
        .pre("let mut call = use_signal(|| CallCard::new(\"search_web\", \"usage-based pricing pages\"));")
        .raw("card", "call()")
        .raw("now", "clock.now()")
        .raw("icon", q("globe"))
        .raw("max_code", format!("{}", v.f("cap") as u32))
        .raw("on_toggle", "move |_| call.write().toggle()")
        .raw("on_retry", "EventHandler::new(move |_| call.write().retry(now))")
        .done()
}

// ---------- tool call (transcript rows) ----------

fn out_for(kind: &str) -> RowOutput {
    match kind {
        "table" => RowOutput::Table(
            vec!["id".into(), "email".into(), "plan".into()],
            (1..=7)
                .map(|i| {
                    vec![
                        format!("{i}"),
                        format!("user{i}@acme.dev"),
                        if i % 3 == 0 {
                            "team".into()
                        } else {
                            "pro".into()
                        },
                    ]
                })
                .collect(),
        ),
        "json" => RowOutput::Json(
            r#"{"id":4821,"state":"open","labels":["billing","p1"],"assignee":null}"#.into(),
        ),
        "text" => RowOutput::Text("Posted to #release at 14:02.".into()),
        _ => RowOutput::List(vec![
            ("Usage meter drops events on retry".into(), "#412".into()),
            ("Invoice PDF shows wrong currency".into(), "#398".into()),
            ("Proration rounds the wrong way".into(), "#377".into()),
            ("Add usage alerts".into(), "#350".into()),
            ("Seats sync lags by a day".into(), "#341".into()),
            ("Trial end email missing".into(), "#336".into()),
        ]),
    }
}

fn calls(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        LiveCalls {
            status: v.string("status"),
            output: v.string("output"),
            collapse: v.b("collapse"),
            now,
        }
    }
}

/// tool, title, running title, status, message, input, output, duration.
struct Row(
    &'static str,
    &'static str,
    Option<&'static str>,
    CallStatus,
    Option<&'static str>,
    Vec<(String, String)>,
    Option<RowOutput>,
    Option<u64>,
);

#[component]
fn LiveCalls(status: String, output: String, collapse: bool, now: ClockMs) -> Element {
    // The fourth call's status, and when it last changed (approve / retry simulate the call).
    keyed!(
        st,
        (status.clone(), output.clone()),
        (CallStatus::parse(&status), now)
    );
    let mut open_row = use_signal(|| Some(0usize));
    let mut gopen = use_signal(|| None::<bool>);
    let mut touched = use_signal(|| false);
    let (s, since) = st();
    let elapsed = now.0.saturating_sub(since.0);
    let s4 = if s == CallStatus::Running && status != "running" && elapsed > 1_400 {
        CallStatus::Success
    } else {
        s
    };
    let active = s4 == CallStatus::Running || s4 == CallStatus::Approval;
    // Folds itself 450 ms after everything settled, unless someone toggled.
    let auto_open = active || s4 == CallStatus::Error || !(collapse && elapsed > 450 + 1_400);
    let is_open = gopen().unwrap_or(auto_open);
    let rows: Vec<Row> = vec![
        Row(
            "mcp__github__search_issues",
            "Searched issues in acme/billing",
            Some("Searching issues in acme/billing"),
            CallStatus::Success,
            None,
            vec![
                ("repo".into(), "acme/billing".into()),
                ("query".into(), "is:open label:billing".into()),
            ],
            Some(out_for(&output)),
            Some(640),
        ),
        Row(
            "read",
            "Read src/billing/meter.rs",
            None,
            CallStatus::Success,
            None,
            vec![("path".into(), "src/billing/meter.rs".into())],
            Some(RowOutput::Text("212 lines".into())),
            Some(120),
        ),
        Row(
            "postgres_query",
            "Queried 7 trial accounts",
            Some("Querying trial accounts"),
            CallStatus::Success,
            None,
            vec![(
                "sql".into(),
                "select id, email, plan from accounts where trial".into(),
            )],
            Some(out_for("table")),
            Some(1_200),
        ),
        Row(
            "slack_post",
            "Posted a summary to #release",
            Some("Posting a summary to #release"),
            s4,
            match s4 {
                CallStatus::Approval => Some("Posts a 6-line summary to #release (42 members)."),
                CallStatus::Error => Some("Slack returned 429: rate limited, retry in 30s."),
                CallStatus::Denied => Some("You denied this call."),
                _ => None,
            },
            vec![
                ("channel".into(), "#release".into()),
                ("text".into(), "Billing fixes are in review…".into()),
            ],
            if s4 == CallStatus::Success {
                Some(out_for("text"))
            } else {
                None
            },
            if s4 == CallStatus::Success {
                Some(380)
            } else {
                None
            },
        ),
    ];
    let tools: Vec<String> = rows.iter().map(|r| r.0.to_string()).collect();
    let total: u64 = rows.iter().filter_map(|r| r.7).sum();
    rsx! {
        div { class: "oxd-w560",
            ToolCallList { tools, total_ms: total, active, open: is_open, on_open: move |o| { touched.set(true); gopen.set(Some(o)) },
                for (i, r) in rows.into_iter().enumerate() {
                    ToolCall {
                        key: "{i}",
                        tool: r.0.to_string(),
                        title: r.1.to_string(),
                        running_title: r.2.map(|x| x.to_string()),
                        status: r.3,
                        message: r.4.map(|x| x.to_string()),
                        input: r.5,
                        output: r.6,
                        duration_ms: r.7,
                        open: open_row() == Some(i),
                        on_toggle: move |_| { touched.set(true); open_row.set(if open_row() == Some(i) { None } else { Some(i) }) },
                        on_allow: move |_| st.set((CallStatus::Running, now)),
                        on_deny: move |_| st.set((CallStatus::Denied, now)),
                        on_retry: move |_| st.set((CallStatus::Running, now)),
                    }
                }
            }
        }
    }
}

fn calls_code(_: &Vals) -> String {
    Code::new("ToolCallList")
        .raw("tools", "calls.iter().map(|c| c.tool.clone()).collect()")
        .raw("total_ms", "calls.iter().filter_map(|c| c.ms).sum()")
        .raw("active", "turn.is_running()")
        .raw("open", "open()")
        .raw("on_open", "move |o| open.set(o)")
        .raw("// children", "ToolCall { tool, title, running_title, status, message, input, output, duration_ms, on_allow, on_deny, on_retry }")
        .done()
}

// ---------- tool chips ----------

fn chips(v: &Vals, _: ClockMs) -> Element {
    let live = v.b("live");
    let st = |d: CallStatus| if live { d } else { CallStatus::Success };
    let steps = vec![
        ChipStep {
            tool: "grep".into(),
            label: "Searched".into(),
            chip: "\"waffle cone\"".into(),
            mono: false,
            status: CallStatus::Success,
            detail: vec![
                "12 matches in 4 files".into(),
                "src/suppliers/cone_king.ts".into(),
            ],
        },
        ChipStep {
            tool: "read".into(),
            label: "Read".into(),
            chip: "reorder.ts".into(),
            mono: true,
            status: CallStatus::Success,
            detail: vec!["148 lines".into()],
        },
        ChipStep {
            tool: "edit".into(),
            label: "Edited".into(),
            chip: "reorder.ts".into(),
            mono: true,
            status: st(CallStatus::Success),
            detail: vec!["+18 −4 · lead time now 7 days".into()],
        },
        ChipStep {
            tool: "bash".into(),
            label: if live { "Running".into() } else { "Ran".into() },
            chip: "npm test reorder".into(),
            mono: true,
            status: st(CallStatus::Running),
            detail: vec!["14 passed".into()],
        },
    ];
    let diffs = vec![
        DiffChip {
            file: "reorder.ts".into(),
            add: 18,
            del: 4,
            lines: vec![
                " export function leadTime(s: Supplier) {".into(),
                "-  return 5".into(),
                "+  return s.days ?? 7".into(),
                " }".into(),
            ],
        },
        DiffChip {
            file: "suppliers.json".into(),
            add: 3,
            del: 1,
            lines: vec![
                " \"cone-king\": {".into(),
                "-  \"days\": 5".into(),
                "+  \"days\": 7,".into(),
                "+  \"min\": 400".into(),
                " }".into(),
            ],
        },
    ];
    let key = format!("{}-{}", v.string("density"), v.b("open"));
    rsx! {
        div { class: "oxd-w520", key: "{key}",
            ToolChips { steps, diffs, messages: 2, density: v.string("density"), open: v.b("open") }
        }
    }
}

fn chips_code(v: &Vals) -> String {
    Code::new("ToolChips")
        .raw("steps", "run.steps() // Vec<ChipStep>")
        .raw("diffs", "run.diffs() // Vec<DiffChip>")
        .raw("messages", "2")
        .s("density", &v.string("density"), "rows")
        .done()
}

// ---------- json viewer ----------

fn sample_json() -> Json {
    let users: Vec<String> = [
        ("Ada", 36, "admin"),
        ("Lin", 29, "member"),
        ("Sam", 41, "member"),
        ("Noor", 33, "owner"),
        ("Kai", 25, "member"),
        ("Ines", 38, "member"),
        ("Tom", 45, "billing"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (n, a, r))| {
        format!(
            r#"{{"id":{},"name":"{n}","age":{a},"role":"{r}","active":{}}}"#,
            100 + i,
            i % 3 != 2
        )
    })
    .collect();
    Json::parse(&format!(
        r#"{{"workspace":"acme","plan":{{"name":"Team","seats":50,"renews":"2027-01-01","usage":{{"events":182044,"limit":250000}}}},"users":[{}],"flags":{{"beta":true,"sso":false,"region":null}}}}"#,
        users.join(",")
    ))
    .unwrap_or(Json::Null)
}

fn json(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        LiveJson {
            depth: v.string("depth").parse().unwrap_or(1),
            page: v.string("page").parse().unwrap_or(50),
            query: v.string("query"),
            searchable: v.b("search"),
            copyable: v.b("copy"),
            show_path: v.b("path"),
            max_height: v.f("height") as u32,
        }
    }
}

#[component]
fn LiveJson(
    depth: usize,
    page: usize,
    query: String,
    searchable: bool,
    copyable: bool,
    show_path: bool,
    max_height: u32,
) -> Element {
    keyed!(t, (depth, page, query.clone()), {
        let mut t = JsonTree::with_depth(sample_json(), depth);
        t.page = page;
        t.query = query.clone();
        t
    });
    rsx! {
        div { class: "oxd-w520",
            JsonViewer {
                tree: t(),
                searchable,
                copyable,
                show_path,
                max_height,
                label: "Workspace export".to_string(),
                on_toggle: move |p: String| t.write().toggle(&p),
                on_query: move |q: String| { let mut w = t.write(); w.query = q; w.current = 0; },
                on_more: move |p: String| t.write().more(&p),
                on_key: move |k: String| t.write().key(&k),
            }
        }
    }
}

fn json_code(v: &Vals) -> String {
    Code::new("JsonViewer")
        .pre(format!(
            "let mut tree = use_signal(|| JsonTree::with_depth(Json::parse(&text).unwrap(), {}));",
            v.string("depth")
        ))
        .raw("tree", "tree()")
        .b("searchable", v.b("search"))
        .raw("on_toggle", "move |p| tree.write().toggle(&p)")
        .raw("on_query", "move |q| tree.write().query = q")
        .raw("on_more", "move |p| tree.write().more(&p)")
        .raw("on_key", "move |k| tree.write().key(&k)")
        .done()
}

// ---------- expandable card ----------

fn xcard(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        LiveX { open: v.b("open"), width: v.f("width") as u32, expanded_width: v.f("ewidth") as u32, state: v.string("state") }
    }
}

#[component]
fn LiveX(open: bool, width: u32, expanded_width: u32, state: String) -> Element {
    keyed!(o, open, open);
    let desc = match state.as_str() {
        "done" => "Deployed 3 services in 2m 14s",
        "failed" => "Deploy failed at the api service",
        _ => "Deploying 3 services to production",
    };
    let api = if state == "failed" {
        "✕  migration 0042 failed"
    } else {
        "✓  58s"
    };
    let log = format!("web      ✓  41s\napi      {api}\nworker   ✓  35s");
    rsx! {
        div { class: "oxd-w560",
            ExpandableCard {
                title: "Production deploy".to_string(),
                description: desc.to_string(),
                icon: "bolt".to_string(),
                expanded: o(),
                width,
                expanded_width,
                on_toggle: move |x| o.set(x),
                div { class: "oxt-sec",
                    div { class: "oxt-sh", "Services" }
                    pre { class: "oxt-code", "{log}" }
                    div { class: "oxr-dim oxr-small", "Commit 9f2c1ab · triggered by Ada · region iad1" }
                }
            }
        }
    }
}

fn xcard_code(v: &Vals) -> String {
    let w = v.f("width") as u32;
    let ew = v.f("ewidth") as u32;
    let mut c = Code::new("ExpandableCard")
        .raw("title", q("Production deploy"))
        .raw("description", "status_line()")
        .raw("expanded", "open()")
        .raw("on_toggle", "move |o| open.set(o)");
    if w > 0 {
        c = c.raw("width", format!("{w}"));
    }
    if ew > 0 {
        c = c.raw("expanded_width", format!("{ew}"));
    }
    c.raw("// children", "the details").done()
}

// ---------- link unfurl ----------

fn preview_for(url: &str) -> Option<Preview> {
    let p = |site: &str, title: &str, d: &str, image: bool| {
        Some(Preview {
            site: site.into(),
            title: title.into(),
            description: d.into(),
            image,
        })
    };
    if url.contains("airbnb") {
        p(
            "Airbnb",
            "Alfama loft with river view · 8 guests",
            "4 bedrooms · ★ 4.92 · €310 / night",
            true,
        )
    } else if url.contains("github") {
        p(
            "GitHub",
            "acme/offsite-planner",
            "Shared itinerary and budget for team offsites",
            false,
        )
    } else if url.contains("maps") {
        p(
            "Google Maps",
            "Time Out Market Lisboa",
            "Av. 24 de Julho 49 · 12 min walk",
            true,
        )
    } else {
        None
    }
}

const SAMPLES: [&str; 3] = [
    "https://airbnb.com/rooms/48213",
    "https://github.com/acme/offsite-planner",
    "https://maps.google.com/?q=time+out+market",
];

fn unfurl(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        LiveUnfurl {
            auto_expand: v.b("expand"),
            autoplay: v.b("autoplay"),
            speed: v.string("speed").parse().unwrap_or(1.0),
            preset: v.string("preset"),
            now,
        }
    }
}

/// Links resolve after 900 ms / speed: the shimmer, then the card (or the inline title).
fn resolved(links: &[Unfurl], now: ClockMs, auto: bool, speed: f64) -> Vec<Unfurl> {
    let wait = (900.0 / speed.max(0.1)) as u64;
    links
        .iter()
        .cloned()
        .map(|mut l| {
            if l.state == UnfurlState::Loading && now.0.saturating_sub(l.at.0) >= wait {
                l.resolve(preview_for(&l.url), auto);
            }
            l
        })
        .collect()
}

#[component]
fn LiveUnfurl(
    auto_expand: bool,
    autoplay: bool,
    speed: f64,
    preset: String,
    now: ClockMs,
) -> Element {
    let pre = preset.clone();
    let start = move || -> (String, Vec<Unfurl>) {
        let base = "Found a place for the offsite, take a look".to_string();
        match pre.as_str() {
            "card" if autoplay => (
                format!("{base} {}", SAMPLES[0]),
                vec![Unfurl {
                    url: SAMPLES[0].into(),
                    state: UnfurlState::Loading,
                    at: now,
                }],
            ),
            "inline" => (
                format!("{base} {}", SAMPLES[1]),
                vec![Unfurl {
                    url: SAMPLES[1].into(),
                    state: UnfurlState::Inline(preview_for(SAMPLES[1]).unwrap_or_else(|| {
                        Preview {
                            site: String::new(),
                            title: String::new(),
                            description: String::new(),
                            image: false,
                        }
                    })),
                    at: now,
                }],
            ),
            "none" => (
                format!("{base} https://example.org/brief"),
                vec![Unfurl {
                    url: "https://example.org/brief".into(),
                    state: UnfurlState::Loading,
                    at: now,
                }],
            ),
            _ => (base, vec![]),
        }
    };
    keyed!(m, (auto_expand, autoplay, preset.clone()), start());
    let (text, links) = m();
    let shown = resolved(&links, now, auto_expand, speed);
    let mut sent = use_signal(|| None::<String>);
    let mut add = move |url: &str| {
        let mut w = m.write();
        if !w.1.iter().any(|l| l.url == url) {
            w.0 = format!("{} {url}", w.0.trim_end());
            w.1.push(Unfurl {
                url: url.into(),
                state: UnfurlState::Loading,
                at: now,
            });
        }
    };
    let mut settle_into = move |shown: Vec<Unfurl>| m.write().1 = shown;
    rsx! {
        div { class: "oxd-w520",
            LinkUnfurl {
                text,
                links: shown.clone(),
                now,
                on_input: move |t: String| {
                    let urls = find_urls(&t);
                    let mut w = m.write();
                    w.0 = t;
                    for u in urls { if !w.1.iter().any(|l| l.url == u) { w.1.push(Unfurl { url: u, state: UnfurlState::Loading, at: now }); } }
                },
                on_toggle: { let shown = shown.clone(); move |i: usize| { let mut s = shown.clone(); if let Some(l) = s.get_mut(i) { l.toggle(); } settle_into(s) } },
                on_remove: { let shown = shown.clone(); move |i: usize| { let mut s = shown.clone(); if let Some(l) = s.get_mut(i) { l.state = UnfurlState::Plain; } settle_into(s) } },
                on_send: move |_| { let (t, _) = m(); sent.set(Some(t)); m.set((String::new(), vec![])) },
            }
            div { class: "oxt-demo-samples",
                for (u, host) in SAMPLES.iter().map(|u| (*u, u.split('/').nth(2).unwrap_or(u))) {
                    button { class: "oxr-btn", r#type: "button", onclick: move |_| add(u), "Paste {host}" }
                }
            }
            if let Some(t) = sent() { p { class: "oxr-dim oxr-small", role: "status", "Sent: {t}" } }
        }
    }
}

fn unfurl_code(v: &Vals) -> String {
    Code::new("LinkUnfurl")
        .pre("// links: Vec<Unfurl>; resolve each pasted URL with your unfurl endpoint, then Unfurl::resolve(preview, auto_expand)")
        .raw("text", "text()")
        .raw("links", "links()")
        .raw("now", "clock.now()")
        .raw("on_input", "move |t| { text.set(t.clone()); for u in find_urls(&t) { links.write().push(Unfurl::loading(u, now)) } }")
        .raw("on_toggle", "move |i| links.write()[i].toggle()")
        .raw("on_remove", "move |i| links.write()[i].state = UnfurlState::Plain")
        .raw("on_send", "move |_| send(text(), links())")
        .raw("// auto_expand", format!("{}", v.b("expand")))
        .done()
}

// ---------- tool-kind map ----------

fn kinds(v: &Vals, _: ClockMs) -> Element {
    let mut names: Vec<String> = match v.string("agent").as_str() {
        "codex" => [
            "shell",
            "apply_patch",
            "update_plan",
            "web_search",
            "view_image",
            "local_shell",
        ]
        .map(String::from)
        .to_vec(),
        "mcp" => [
            "mcp__github__create_issue",
            "mcp__linear__list_issues",
            "mcp__postgres__query",
            "mcp__slack__post_message",
        ]
        .map(String::from)
        .to_vec(),
        "opencode" => [
            "read",
            "write",
            "edit",
            "glob",
            "grep",
            "bash",
            "todowrite",
            "webfetch",
        ]
        .map(String::from)
        .to_vec(),
        _ => [
            "Read",
            "Edit",
            "MultiEdit",
            "Write",
            "Glob",
            "Grep",
            "Bash",
            "WebFetch",
            "WebSearch",
            "TodoWrite",
            "Task",
        ]
        .map(String::from)
        .to_vec(),
    };
    let extra = v.string("name");
    if !extra.trim().is_empty() {
        names.insert(0, extra.trim().to_string());
    }
    rsx! { div { class: "oxd-w560", ToolKindLegend { names } } }
}

fn kinds_code(_: &Vals) -> String {
    Code::new("ToolKindLegend")
        .pre("let map = ToolKindMap::standard().with(\"deploy_vercel\", ToolKind::Shell).with_prefix(\"db_\", ToolKind::Search);")
        .pre("let info = map.info(\"mcp__github__create_issue\", VerbPack::neutral()); // kind, icon, present, past, server")
        .raw("names", "vec![\"Read\".into(), \"apply_patch\".into(), \"mcp__github__create_issue\".into()]")
        .done()
}

pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("view", "View", &["card", "group"], "card"),
                sel(
                    "status",
                    "Status",
                    &["running", "success", "error"],
                    "success",
                ),
                tog("open", "Open", true),
                rng("cap", "Max code height", 80.0, 320.0, 20.0, "140"),
            ],
            variants: vec![
                var(
                    "result",
                    "Success, open on arguments and a capped result with Show all.",
                    &[],
                ),
                var(
                    "running",
                    "Live timer and Waiting for result…; settles after 2.6 s.",
                    &[("status", "running"), ("open", "false")],
                ),
                var(
                    "error",
                    "Failed: opened on error, Retry runs it again.",
                    &[("status", "error"), ("open", "false")],
                ),
                var(
                    "group",
                    "A group header that sums the calls (shimmers while one runs).",
                    &[("view", "group")],
                ),
            ],
            render: card,
            code: card_code,
            keyboard: vec![
                ("Tab", "Header, copy buttons, Retry, Show all"),
                ("Enter / Space", "Open or close the card or the group"),
            ],
            a11y: vec![
                "The header is a button with aria-expanded; status is text (Failed, a check with its duration).",
                "Copy buttons are named Copy arguments / Copy result.",
            ],
            responsive: vec![
                "The summary truncates first; name, status and time keep their width.",
            ],
            height: 460,
            align: Align::Start,
            live: true,
            ..mk(
                "tool-call-card",
                "Tool call card",
                "tools",
                "An agent's tool call in one line: live timer, status and summary; folds open to highlighted arguments and result with copy, Retry on errors, and a group that sums them up.",
                "ToolCallCard, ToolCallGroup",
                "ToolCallCardEl",
                "tool-call-card",
                "parity/tools.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "status",
                    "Last call",
                    &["success", "running", "approval", "error", "denied"],
                    "approval",
                ),
                sel(
                    "output",
                    "First output",
                    &["list", "table", "json", "text"],
                    "list",
                ),
                tog("collapse", "Auto collapse", true),
            ],
            variants: vec![
                var(
                    "approval",
                    "A call waits for Allow / Deny on a warning outline.",
                    &[],
                ),
                var(
                    "settled",
                    "Four settled calls; folds itself 450 ms later.",
                    &[("status", "success")],
                ),
                var(
                    "error",
                    "Rate limited: the reason and Retry.",
                    &[("status", "error")],
                ),
                var(
                    "table",
                    "Table output folding after five rows.",
                    &[
                        ("status", "success"),
                        ("output", "table"),
                        ("collapse", "false"),
                    ],
                ),
                var(
                    "running",
                    "Using tools… with the title shimmering.",
                    &[("status", "running")],
                ),
            ],
            render: calls,
            code: calls_code,
            keyboard: vec![
                (
                    "Tab",
                    "Group header, each row, Allow / Deny / Retry, show more",
                ),
                (
                    "Enter / Space",
                    "Open the group or a row's Input and Output",
                ),
            ],
            a11y: vec![
                "Rows are buttons with aria-expanded; the input is a definition list.",
                "Approval and error messages are text under the title.",
            ],
            responsive: vec!["Titles truncate; tables scroll inside their well."],
            height: 520,
            align: Align::Start,
            live: true,
            ..mk(
                "tool-call-transcript",
                "Tool call",
                "tools",
                "Tool call rows for agent transcripts: the tool's mark and the call in plain words, a live state while it runs, Input and Output a click away (list, table, JSON or text), with approval and retry when needed.",
                "ToolCallList, ToolCall",
                "ToolCallEl",
                "tool-call-transcript",
                "parity/tools.rs",
            )
        },
        Demo {
            controls: vec![
                sel("density", "Density", &["rows", "chips", "cards"], "rows"),
                tog("live", "Live (a call running)", false),
                tog("open", "Open", true),
            ],
            variants: vec![
                var(
                    "rows",
                    "Rows with inline chips, diff chips that preview on hover.",
                    &[],
                ),
                var("chips", "Only the chips, wrapped.", &[("density", "chips")]),
                var("cards", "Each call in a card.", &[("density", "cards")]),
                var("live", "A test run in progress.", &[("live", "true")]),
            ],
            render: chips,
            code: chips_code,
            keyboard: vec![
                ("Tab", "Header, each call, each diff chip"),
                ("Enter / Space", "Open a call's details; pin a diff"),
                ("Focus", "A focused diff chip previews its diff"),
            ],
            a11y: vec![
                "Diff chips are toggle buttons (aria-pressed) whose preview is a labelled region.",
            ],
            responsive: vec!["Chips wrap; the diff preview scrolls sideways."],
            height: 380,
            align: Align::Start,
            live: false,
            ..mk(
                "tool-chips",
                "Tool chips",
                "tools",
                "Code edits and tool calls as compact chips: “4 tool calls, 2 messages”, a live state per chip, then file-diff chips whose diff opens on hover or focus. Beautiful UI (MIT © 2026 Shane Levine), behaviour rebuilt.",
                "ToolChips",
                "ToolChipsEl",
                "tool-chips",
                "parity/tools.rs",
            )
        },
        Demo {
            controls: vec![
                sel("depth", "Open depth", &["1", "2", "3"], "2"),
                sel("page", "Page size", &["50", "3"], "50"),
                txt("query", "Query", ""),
                tog("search", "Searchable", true),
                tog("copy", "Copyable", true),
                tog("path", "Show path", true),
                rng("height", "Max height", 160.0, 420.0, 20.0, "300"),
            ],
            variants: vec![
                var(
                    "tree",
                    "Two levels open; arrow keys, Enter, / to search.",
                    &[],
                ),
                var(
                    "search",
                    "Search “member”: matches open their branches.",
                    &[("query", "member")],
                ),
                var(
                    "paged",
                    "Three per page with Show more.",
                    &[("page", "3"), ("depth", "3")],
                ),
                var(
                    "bare",
                    "No search, no copy, root only.",
                    &[("search", "false"), ("copy", "false"), ("depth", "1")],
                ),
            ],
            render: json,
            code: json_code,
            keyboard: vec![
                ("↓ / ↑", "Next / previous visible row"),
                ("→ / ←", "Open or step in / close or go to parent"),
                ("Home / End", "First / last row"),
                ("Enter / Space", "Toggle a branch or load the next page"),
                ("⌘/Ctrl C, ⇧ for path", "Copy the current value or path"),
                (
                    "/",
                    "Focus search; Enter / ⇧Enter next / previous match; Esc clears; ↓ back to the tree",
                ),
            ],
            a11y: vec![
                "role=tree with treeitems (aria-level, aria-expanded, aria-selected) and aria-activedescendant.",
                "Copy confirmations are a status.",
            ],
            responsive: vec!["Values truncate; the tree scrolls inside max height."],
            height: 460,
            align: Align::Start,
            live: false,
            ..mk(
                "json-viewer",
                "JSON viewer",
                "data-display",
                "A collapsible JSON tree with search, paging for long arrays, copy value or path, the current path under the tree and full keyboard moves.",
                "JsonViewer",
                "JsonViewerEl",
                "json-viewer",
                "parity/tools.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Expanded", false),
                sel(
                    "state",
                    "Description",
                    &["running", "done", "failed"],
                    "running",
                ),
                rng("width", "Width (0 fills)", 0.0, 520.0, 20.0, "360"),
                rng("ewidth", "Expanded width", 0.0, 560.0, 20.0, "520"),
            ],
            variants: vec![
                var(
                    "collapsed",
                    "Dense card that grows sideways and down when opened.",
                    &[],
                ),
                var("open", "Started open.", &[("open", "true")]),
                var(
                    "fill",
                    "Fills its container.",
                    &[("width", "0"), ("ewidth", "0"), ("state", "done")],
                ),
                var(
                    "failed",
                    "A changed description rolls in.",
                    &[("state", "failed"), ("open", "true")],
                ),
            ],
            render: xcard,
            code: xcard_code,
            keyboard: vec![
                ("Enter / Space", "Toggle from the header"),
                ("Escape", "Close from inside, focus stays on the header"),
            ],
            a11y: vec!["Header button with aria-expanded; the closed body is aria-hidden."],
            responsive: vec!["Width caps never exceed the container; it centres."],
            height: 300,
            align: Align::Start,
            live: false,
            ..mk(
                "expandable-card",
                "Expandable card",
                "surfaces",
                "A dense card that gives itself more room on request: width and height morph together, details fade in after the box has made room, Escape closes from inside.",
                "ExpandableCard",
                "ExpandableCardEl",
                "expandable-card",
                "parity/tools.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "preset",
                    "Start with",
                    &["card", "inline", "none", "empty"],
                    "card",
                ),
                tog("expand", "Auto expand", true),
                tog("autoplay", "Autoplay", true),
                sel("speed", "Speed", &["0.5", "1", "2"], "1"),
            ],
            variants: vec![
                var(
                    "unfurl",
                    "A pasted link shimmers, then unfurls into a card.",
                    &[],
                ),
                var(
                    "inline",
                    "Folded to the inline title; click to open.",
                    &[("preset", "inline")],
                ),
                var(
                    "no-preview",
                    "No preview: the link stays plain.",
                    &[("preset", "none")],
                ),
                var(
                    "manual",
                    "Auto expand off: links gain only their title.",
                    &[("preset", "empty"), ("expand", "false")],
                ),
            ],
            render: unfurl,
            code: unfurl_code,
            keyboard: vec![
                ("Enter", "Send"),
                ("Shift + Enter", "New line"),
                ("Tab", "Inline links, Collapse, Remove, Send"),
            ],
            a11y: vec![
                "The field is a labelled textarea; card buttons are named Collapse preview / Remove preview.",
                "Sending announces in a status line.",
            ],
            responsive: vec!["Cards cap at 420 px and shrink with the composer."],
            height: 360,
            align: Align::Start,
            live: true,
            ..mk(
                "link-unfurl",
                "Link unfurl",
                "composer",
                "A message field where a pasted link shimmers while it resolves, then unfurls into a preview card (site, title, description, image) that folds back to an inline link or is removed.",
                "LinkUnfurl",
                "LinkUnfurlEl",
                "link-unfurl",
                "parity/tools.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "agent",
                    "Agent",
                    &["claude-code", "codex", "opencode", "mcp"],
                    "claude-code",
                ),
                txt("name", "Try a tool name", "db_query"),
            ],
            variants: vec![
                var("claude-code", "Claude Code names.", &[]),
                var("codex", "Codex names.", &[("agent", "codex"), ("name", "")]),
                var(
                    "mcp",
                    "MCP names split into server and tool.",
                    &[("agent", "mcp"), ("name", "")],
                ),
            ],
            render: kinds,
            code: kinds_code,
            keyboard: vec![("—", "Read-only table")],
            a11y: vec!["A plain table with header cells."],
            responsive: vec!["Cells keep one line; the table scrolls sideways when narrow."],
            height: 420,
            align: Align::Start,
            live: false,
            ..mk(
                "tool-kind-map",
                "Tool-kind map",
                "tools",
                "Raw tool names from any agent (Claude Code, Codex, OpenCode, MCP) mapped to a kind, an icon and running / done verbs, with exact-name and prefix overrides.",
                "ToolKindLegend",
                "",
                "tool-kind-map",
                "parity/tools.rs",
            )
        },
    ]
}
