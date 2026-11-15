//! Plan #28 parity demos, chunk 6: the X-post references (thread list, activity heat map and
//! home, token usage, attachment strip, run modes, experiment grid, document grid, polling cost).

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::home::{
        Doc, Thread, ThreadState, demo_counts, demo_experiments, demo_usage, year_days,
    },
};
use oxyn_dioxus::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};

use super::{Code, mk, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

// ---------- thread list ----------

fn threads(set: &str) -> Vec<Thread> {
    let t = |id, p, title, s| Thread::new(id, p, title, s);
    let mut v = vec![
        t(
            "t1",
            "orbit-web",
            "Refactor the auth middleware",
            ThreadState::Approval,
        ),
        t(
            "t2",
            "ledger-api",
            "Pick a retry strategy for webhooks",
            ThreadState::Input,
        )
        .agent("brain"),
        t(
            "t3",
            "orbit-web",
            "Clamp slider values in the editor",
            ThreadState::Idle(21 * 60),
        ),
        t(
            "t4",
            "orbit-web",
            "Plan the router migration",
            ThreadState::Idle(29 * 60),
        )
        .agent("brain"),
        t(
            "t5",
            "ledger-api",
            "Map the payouts state machine",
            ThreadState::Idle(29 * 60 + 20),
        ),
        t(
            "t6",
            "atlas-ui",
            "Fix focus ring on tabs",
            ThreadState::Done,
        )
        .agent("brain"),
        t(
            "t7",
            "orbit-web",
            "Clamp the zoom level on the canvas",
            ThreadState::Working(130),
        ),
        t(
            "t8",
            "ledger-api",
            "Validate payout amounts",
            ThreadState::Working(95),
        )
        .agent("brain"),
        t("t9", "atlas-ui", "Bump the icon set", ThreadState::Settled),
        t(
            "t10",
            "orbit-web",
            "Remove the old feature flags",
            ThreadState::Settled,
        ),
    ];
    if set == "calm" {
        for x in &mut v {
            if matches!(x.state, ThreadState::Approval | ThreadState::Input) {
                x.state = ThreadState::Idle(4 * 60);
            }
        }
    }
    v
}

fn thread_list(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let mut open = use_signal(|| Some("t1".to_string()));
    let list = threads(v.s("set"));
    let title = open
        .read()
        .as_ref()
        .and_then(|id| list.iter().find(|t| &t.id == id))
        .map(|t| (t.project.clone(), t.title.clone()));
    rsx! {
        div { class: "oxd-t3", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:0;width:100%;max-width:820px;border:1px solid var(--oxyn-color-border-default);border-radius:14px;overflow:hidden;background:var(--oxyn-color-surface-raised)",
            div { style: "padding:10px;box-shadow:1px 0 0 var(--oxyn-color-border-default);background:var(--oxyn-color-surface-sunken)",
                ThreadList { threads: list.clone(), selected: open(), search: v.b("search"), settled_open: v.b("settled"),
                    on_select: move |id: String| { msg.set(format!("onSelect({id})")); open.set(Some(id)); },
                    on_new: move |_| { msg.set("onNew()".into()); open.set(None); },
                }
            }
            div { style: "display:flex;flex-direction:column;align-items:center;justify-content:center;gap:10px;padding:24px;min-height:220px;text-align:center",
                if v.b("home") {
                    {
                        let (p, t) = title.clone().unwrap_or(("orbit-web".into(), "New thread".into()));
                        rsx! {
                            div { style: "font-size:12px;color:var(--oxyn-color-fg-muted)", "{p} / {t}" }
                            div { style: "font-size:22px;font-weight:500;letter-spacing:-.01em", "What should we build in " b { "{p}" } "?" }
                            div { style: "font-size:12.5px;color:var(--oxyn-color-fg-muted)", "or start without a project" }
                        }
                    }
                }
                {status(msg)}
            }
        }
    }
}

fn thread_list_code(v: &Vals) -> String {
    Code::new("ThreadList")
        .pre("let threads = vec![")
        .pre("    Thread::new(\"t1\", \"orbit-web\", \"Refactor the auth middleware\", ThreadState::Approval),")
        .pre("    Thread::new(\"t2\", \"ledger-api\", \"Pick a retry strategy\", ThreadState::Input),")
        .pre("    Thread::new(\"t7\", \"orbit-web\", \"Clamp the zoom level\", ThreadState::Working(130)),")
        .pre("    Thread::new(\"t6\", \"atlas-ui\", \"Fix focus ring on tabs\", ThreadState::Done),")
        .pre("];")
        .raw("threads", "threads")
        .raw("selected", "open()")
        .b("settled_open", v.b("settled"))
        .raw("on_select", "move |id: String| open.set(Some(id))")
        .done()
}

// ---------- activity heat map ----------

fn heatmap(v: &Vals, _: ClockMs) -> Element {
    let noun = v.string("noun");
    let years = vec![
        (2025, demo_counts(11, year_days(2025), 120)),
        (2026, demo_counts(5, 318, 290)),
    ];
    rsx! {
        div { style: "width:100%;max-width:760px;padding:14px 16px;border-radius:12px;border:1px solid var(--oxyn-color-border-default);background:var(--oxyn-color-surface-raised)",
            if v.s("form") == "compact" {
                ActivityHeatmap { years: years.clone(), noun: noun.clone(), weeks: Some(24), cell: 10.0 }
            } else {
                ActivityHeatmap { years, title: format!("{} activity", if noun == "thread" { "Threadbear" } else { "Chat" }), noun,
                    footnote: if v.b("foot") { Some("Threads in a private folder aren't counted".to_string()) } else { None },
                    cell: v.f("cell") as f32 }
            }
        }
    }
}

fn heatmap_code(v: &Vals) -> String {
    let mut c = Code::new("ActivityHeatmap")
        .raw("years", "vec![(2025, counts_2025), (2026, counts_2026)]")
        .s("noun", v.s("noun"), "chat");
    if v.s("form") == "compact" {
        c = c.raw("weeks", "Some(24)");
    } else {
        c = c.raw("title", q("Threadbear activity"));
    }
    c.done()
}

// ---------- activity home ----------

fn composer_box(placeholder: &str) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:8px;padding:10px 12px;border-radius:14px;border:1px solid var(--oxyn-color-border-default);background:var(--oxyn-color-surface-raised)",
            div { style: "font-size:13.5px;color:var(--oxyn-color-fg-muted);min-height:20px", "{placeholder}" }
            div { style: "display:flex;align-items:center;gap:8px;font-size:12px;color:var(--oxyn-color-fg-muted)",
                span { "+" } span { "Request approval" } span { style: "flex:1" } span { "Auto ⌄" }
                span { style: "width:24px;height:24px;border-radius:50%;background:var(--oxyn-color-fg-default);color:var(--oxyn-color-bg-base);display:grid;place-items:center", "↑" }
            }
        }
    }
}

fn activity_home(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let variant = v.string("variant");
    let lib = variant == "librarian";
    let used = v.f("used") as u32;
    rsx! {
        div { style: "width:100%;max-width:880px;display:flex;flex-direction:column;gap:8px",
            ActivityHome { variant: variant.clone(), now, credits: (used, 2000), counts: demo_counts(5, 318, if lib { 290 } else { 120 }),
                greeting: if v.b("greet") { if lib { "Good night, Wayne".to_string() } else { "Good afternoon".to_string() } } else { String::new() },
                date: if v.b("greet") { "Sunday, November 15 at 10:40 PM".to_string() } else { String::new() },
                on_open: move |s: String| msg.set(format!("onOpen({s})")),
                if !lib && v.b("composer") { {composer_box("Help me brainstorm ideas for…")} }
            }
            {status(msg)}
        }
    }
}

fn activity_home_code(v: &Vals) -> String {
    Code::new("ActivityHome")
        .s("variant", v.s("variant"), "overview")
        .raw("credits", format!("({}, 2000)", v.f("used") as u32))
        .raw("counts", "counts_per_day")
        .raw("now", "clock.now()")
        .raw("on_open", "move |id: String| open(id)")
        .done()
}

// ---------- token usage ----------

fn usage_chart(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        TokenUsageChart { days: demo_usage(v.f("days") as usize), per_m_cents: v.f("price") * 100.0, start: (2026, 177) }
    }
}

fn usage_chart_code(v: &Vals) -> String {
    Code::new("TokenUsageChart")
        .raw("days", "usage_by_day")
        .raw("per_m_cents", format!("{:.1}", v.f("price") * 100.0))
        .raw("start", "(2026, 177) // 27 Jun")
        .done()
}

// ---------- attachment strip ----------

const FILES: [&str; 6] = [
    "orange-cat.jpg",
    "payrollzone.pdf",
    "timetable.csv",
    "presentation.key",
    "reactypes.ts",
    "birthday-clip.mp4",
];

fn attach(v: &Vals, now: ClockMs) -> Element {
    let n = v.f("count") as usize;
    let mut added = use_signal(|| now);
    let mut removed = use_signal(Vec::<String>::new);
    let list: Vec<String> = FILES
        .iter()
        .take(n)
        .map(|s| s.to_string())
        .filter(|s| !removed.read().contains(s))
        .collect();
    let names = list.clone();
    let weights: Vec<f32> = list
        .iter()
        .map(|f| match f.rsplit('.').next() {
            Some("jpg") => 0.8,
            Some("mp4") => 1.6,
            Some("key") => 1.2,
            _ => 0.6,
        })
        .collect();
    let speed = v.f("speed");
    rsx! {
        div { style: "width:100%;max-width:560px;display:flex;flex-direction:column;gap:8px",
            div { style: "display:flex;flex-direction:column;border-radius:16px;border:1px solid var(--oxyn-color-border-default);background:var(--oxyn-color-surface-raised);overflow:hidden",
                div { style: "display:flex;align-items:center;gap:12px;padding:6px 12px;font-size:12px;color:var(--oxyn-color-fg-muted);background:var(--oxyn-color-surface-sunken)",
                    span { "Main" } span { "·" } span { "project-sea" }
                }
                div { style: "display:flex;flex-direction:column;gap:10px;padding:14px 14px 10px",
                    AttachmentStrip { files: list, weights, added: added(), now, dur_ms: (1400.0 / speed.max(0.1)) as u64, land: v.b("land"),
                        on_remove: move |i: usize| { if let Some(f) = names.get(i) { removed.write().push(f.clone()); } },
                    }
                    div { style: "font-size:14px;color:var(--oxyn-color-fg-muted)", "Hi, what do you need today?" }
                    div { style: "display:flex;align-items:center;gap:10px;font-size:12.5px;color:var(--oxyn-color-fg-muted)",
                        span { style: "width:28px;height:28px;border-radius:50%;border:1px solid var(--oxyn-color-border-default);display:grid;place-items:center", "+" }
                        span { "Bypass all" } span { style: "flex:1" } span { "GPT-5.6 Mini ⌄" }
                    }
                }
            }
            div { style: "display:flex;gap:8px",
                button { class: "oxh-btn", r#type: "button", onclick: move |_| { removed.write().clear(); added.set(now); }, "Drop the files again" }
            }
        }
    }
}

fn attach_code(v: &Vals) -> String {
    Code::new("AttachmentStrip")
        .raw("files", "files()")
        .raw("weights", "sizes // relative, same order")
        .raw("added", "dropped_at")
        .raw("now", "clock.now()")
        .b("land", !v.b("land"))
        .raw("on_remove", "move |i: usize| { files.write().remove(i); }")
        .done()
}

// ---------- run modes ----------

fn run_modes(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let mode = v.string("mode");
    let mut running = use_signal(|| false);
    let mut paused = use_signal(|| false);
    rsx! {
        div { style: "width:100%;max-width:560px;display:flex;flex-direction:column;gap:10px;padding-top:290px",
            div { style: "display:flex;flex-direction:column;gap:10px;padding:12px 14px;border-radius:16px;border:1px solid var(--oxyn-color-border-default);background:var(--oxyn-color-surface-raised)",
                div { style: "display:flex;gap:10px;align-items:flex-start",
                    RunModeMenu { key: "{mode}", mode: mode.clone(), budget: v.f("budget") as u32, on_change: move |(m, b): (String, u32)| msg.set(format!("onChange({m}, {b} min)")) }
                    span { style: "font-size:13.5px;color:var(--oxyn-color-fg-muted);padding-top:4px",
                        if running() { "Guide this conversation" } else { "In naklecha/simple-llm, replace the copied kernels…" }
                    }
                }
                div { style: "display:flex;align-items:center;gap:8px;font-size:12px;color:var(--oxyn-color-fg-muted)",
                    span { "+" } span { style: "flex:1" } span { "Claude Opus · High" }
                    if running() {
                        button { class: "oxh-btn", r#type: "button", "aria-pressed": "{paused()}", onclick: move |_| { paused.toggle(); msg.set(if paused() { "onPause()".into() } else { "onResume()".into() }); }, if paused() { "Resume" } else { "Pause" } }
                        button { class: "oxh-btn", r#type: "button", onclick: move |_| { running.set(false); paused.set(false); msg.set("onStop()".into()); }, "Stop" }
                    } else {
                        button { class: "oxh-btn", r#type: "button", onclick: move |_| { running.set(true); msg.set("onSend()".into()); }, "Send" }
                    }
                }
            }
            {status(msg)}
        }
    }
}

fn run_modes_code(v: &Vals) -> String {
    Code::new("RunModeMenu")
        .s("mode", v.s("mode"), "work")
        .raw("budget", format!("{}", v.f("budget") as u32))
        .raw(
            "on_change",
            "move |(mode, minutes): (String, u32)| run.set_mode(mode, minutes)",
        )
        .done()
}

// ---------- experiment grid ----------

fn experiments(v: &Vals, now: ClockMs) -> Element {
    let budget = v.f("budget") as u32;
    let n = v.f("count") as usize;
    let exps = demo_experiments(n, budget / 60);
    let mut start = use_signal(|| now);
    let speed = v.f("speed"); // run minutes per real second
    let span = u64::from(budget) * 60 + 1800;
    let elapsed = if v.b("live") {
        ((now.0.saturating_sub(start().0)) as f64 / 1000.0 * speed * 60.0) as u64 % span
    } else {
        u64::from(budget) * 60 * v.f("at") as u64 / 100
    };
    rsx! {
        div { style: "width:100%;max-width:460px;display:flex;flex-direction:column;gap:8px",
            ExperimentGrid { experiments: exps, budget, elapsed: elapsed.min(u64::from(budget) * 60), band: v.f("band") as u32, paused: !v.b("live") }
            if v.b("live") { button { class: "oxh-btn", r#type: "button", style: "align-self:flex-start", onclick: move |_| start.set(now), "Restart the run" } }
        }
    }
}

fn experiments_code(v: &Vals) -> String {
    Code::new("ExperimentGrid")
        .raw("experiments", "run.experiments()")
        .raw("budget", format!("{}", v.f("budget") as u32))
        .raw("elapsed", "run.elapsed_secs()")
        .raw("band", format!("{}", v.f("band") as u32))
        .done()
}

// ---------- document grid ----------

fn docs() -> Vec<Doc> {
    let d = |id: &str, title: &str, ex: &str, visited: u64, favorite: bool| Doc {
        id: id.into(),
        title: title.into(),
        excerpt: ex.into(),
        visited,
        favorite,
    };
    vec![
        d(
            "welcome",
            "Welcome",
            "A quick tour: blocks, headings, tables and the shortcuts that move between them.",
            120,
            true,
        ),
        d(
            "writing",
            "Writing: text, headings and paragraphs",
            "Headings give your writing structure; paragraphs keep the rhythm.",
            900,
            false,
        ),
        d(
            "blocks",
            "Blocks: lists, quotes and callouts",
            "Three ways to set a point apart from the text around it.",
            3_600,
            false,
        ),
        d(
            "tables",
            "Tables and columns",
            "Insert a table, resize the columns and sort by any of them.",
            7_200,
            true,
        ),
        d(
            "code",
            "Code and equations",
            "Inline code, fenced blocks with highlighting, and TeX equations.",
            86_400,
            false,
        ),
        d(
            "images",
            "Images, files and covers",
            "Drop an image, align it, and give the page a cover.",
            172_800,
            false,
        ),
        d(
            "organize",
            "Organize and find your notes",
            "Folders, favourites and the search that reads every page.",
            259_200,
            false,
        ),
        d(
            "reading",
            "Example: reading list",
            "Books read in 2026, with ratings and notes.",
            400_000,
            true,
        ),
    ]
}

fn doc_grid(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:100%;max-width:900px;display:flex;flex-direction:column;gap:8px",
            DocGrid { docs: docs(), sidebar: v.b("sidebar"), list: v.s("layout") == "list", on_open: move |id: String| msg.set(format!("onOpen({id})")) }
            {status(msg)}
        }
    }
}

fn doc_grid_code(v: &Vals) -> String {
    Code::new("DocGrid")
        .raw("docs", "library.docs()")
        .b("list", v.s("layout") == "list")
        .raw("on_open", "move |id: String| open(id)")
        .done()
}

// ---------- polling cost ----------

fn polling(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        PollingCost { agents: v.f("agents") as u32, every: v.f("every") as u32, minutes: v.f("mins") as u32, transcript: v.f("transcript") * 1000.0 }
    }
}

fn polling_code(v: &Vals) -> String {
    Code::new("PollingCost")
        .raw("agents", format!("{}", v.f("agents") as u32))
        .raw("every", format!("{}", v.f("every") as u32))
        .raw("minutes", format!("{}", v.f("mins") as u32))
        .raw("transcript", format!("{:.1}", v.f("transcript") * 1000.0))
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("set", "Threads", &["busy", "calm"], "busy"),
                tog("search", "Search row", true),
                tog("settled", "Settled open", false),
                tog("home", "Project home beside it", true),
            ],
            variants: vec![
                var(
                    "needs-you",
                    "Approval and Input rise to the top; Working and Settled group underneath.",
                    &[],
                ),
                var(
                    "calm",
                    "Nothing waiting: rows show their age.",
                    &[("set", "calm")],
                ),
                var(
                    "open",
                    "Settled group open, no search.",
                    &[("settled", "true"), ("search", "false")],
                ),
            ],
            render: thread_list,
            code: thread_list_code,
            keyboard: vec![
                ("↑ / ↓ / Home / End", "Move through rows and group headers"),
                ("Enter", "Open the thread, or open / close a group header"),
                ("← / →", "Close / open the group under the cursor"),
            ],
            a11y: vec![
                "The list is role=listbox with aria-activedescendant: one tab stop.",
                "Chips carry text (Approval, Input, Done, Working 2m), not colour alone; the 'need you' count is role=status.",
                "Reduced motion drops the row entrance and the working spinner.",
            ],
            responsive: vec!["Fills its column up to 300 px; titles ellipsize."],
            height: 460,
            align: Align::Start,
            ..mk(
                "thread-list",
                "Thread list",
                "shells",
                "Agent threads grouped by where they stand: what needs you rises with an Approval or Input chip, finished work says Done, Working and Settled fold into counted groups.",
                "ThreadList",
                "ThreadListEl",
                "thread-list",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                sel("form", "Form", &["year", "compact"], "year"),
                sel("noun", "Counts", &["thread", "chat"], "thread"),
                rng("cell", "Cell (px)", 8.0, 13.0, 1.0, "11"),
                tog("foot", "Footnote", true),
            ],
            variants: vec![
                var("year", "A year with ‹ › navigation and a day tooltip.", &[]),
                var(
                    "compact",
                    "The last 24 weeks for an overview card.",
                    &[("form", "compact"), ("noun", "chat")],
                ),
            ],
            render: heatmap,
            code: heatmap_code,
            keyboard: vec![
                ("← / →", "A week back / forward"),
                ("↑ / ↓", "A day back / forward"),
                ("Home / End", "First / last day"),
                ("Esc", "Hide the tooltip"),
            ],
            a11y: vec![
                "The grid is one focusable element with a summary label; the tooltip is role=status so arrowing reads each day.",
                "Levels step in lightness and the legend reads Less → More.",
                "Reduced motion drops the cell fade.",
            ],
            responsive: vec!["Scrolls sideways inside its card below the year's width."],
            height: 300,
            ..mk(
                "activity-heatmap",
                "Activity heat map",
                "data-display",
                "A year of activity as week columns in five levels, with a day tooltip, year navigation and a compact strip for overview cards.",
                "ActivityHeatmap",
                "ActivityHeatmapEl",
                "activity-heatmap",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["overview", "librarian"], "overview"),
                rng("used", "Credits used", 0.0, 2000.0, 1.0, "1284"),
                tog("greet", "Greeting", false),
                tog("composer", "Composer slot", true),
            ],
            variants: vec![
                var(
                    "overview",
                    "Composer over Activity, Recent chats, Usage, Files and This month.",
                    &[],
                ),
                var(
                    "near-limit",
                    "Usage past 90 %: the bar turns to the warning tone.",
                    &[("used", "1880")],
                ),
                var(
                    "librarian",
                    "A background agent's home: status, quote and the year heat map.",
                    &[("variant", "librarian"), ("greet", "true")],
                ),
            ],
            render: activity_home,
            code: activity_home_code,
            keyboard: vec![("Tab", "Through the cards' buttons and lists")],
            a11y: vec![
                "Cards are sections with headings; usage is role=meter with the percent in its label.",
                "The Librarian status is role=status.",
                "Reduced motion shows the cards at once.",
            ],
            responsive: vec![
                "Six columns; four under 860 px and two under 620 px of its own width (container query on a full-width root).",
            ],
            height: 560,
            align: Align::Start,
            live: true,
            ..mk(
                "activity-home",
                "Activity home",
                "shells",
                "A home under the composer: activity, recent chats, usage, files and this month, or a background agent's status with a quote and a year of activity.",
                "ActivityHome",
                "ActivityHomeEl",
                "activity-home",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                rng("days", "Days of data", 14.0, 120.0, 1.0, "93"),
                rng("price", "$ per million", 0.5, 15.0, 0.5, "3"),
            ],
            variants: vec![
                var("quarter", "Three months, three ranges.", &[]),
                var("short", "Two weeks of data.", &[("days", "14")]),
            ],
            render: usage_chart,
            code: usage_chart_code,
            keyboard: vec![
                ("← / →", "Read the previous / next day"),
                ("Esc", "Hide the tooltip"),
            ],
            a11y: vec![
                "The plot is one focusable image with the totals in its label; the day tooltip is role=status.",
                "Range buttons are aria-pressed toggles.",
            ],
            responsive: vec![
                "Fills its card up to 420 px; the area stretches, strokes stay 1.5 px.",
            ],
            height: 320,
            ..mk(
                "token-usage-chart",
                "Token usage chart",
                "data-display",
                "Daily tokens as an area with the cached share as a line: hover any day for its numbers, switch the range and the totals roll.",
                "TokenUsageChart",
                "TokenUsageChartEl",
                "token-usage-chart",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                rng("count", "Files", 1.0, 6.0, 1.0, "5"),
                rng("speed", "Speed", 0.25, 3.0, 0.25, "1"),
                tog("land", "Queued files land one after another", true),
            ],
            variants: vec![
                var(
                    "batch",
                    "Five files: each lands as the one before finishes.",
                    &[],
                ),
                var("one", "One image.", &[("count", "1")]),
                var(
                    "all-at-once",
                    "All tiles shown, queued ones waiting.",
                    &[("land", "false")],
                ),
            ],
            render: attach,
            code: attach_code,
            keyboard: vec![("Tab", "To each tile's dismiss button (shown on focus)")],
            a11y: vec![
                "Tiles are list items labelled with the file name and upload percent; a polite status says when all files are up.",
                "Reduced motion drops the landing and the ring transition; the percent still counts.",
            ],
            responsive: vec!["Tiles wrap onto a second row."],
            height: 300,
            align: Align::Start,
            live: true,
            ..mk(
                "attachment-strip",
                "Attachment strip",
                "composer",
                "Composer attachments as 56 px tiles: queued files land one after another while a ring draws round the uploading tile and a small percent counts up, then blurs away.",
                "AttachmentStrip",
                "AttachmentStripEl",
                "attachment-strip",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                sel("mode", "Mode", &["ask", "plan", "work", "auto"], "work"),
                rng("budget", "Budget (min)", 5.0, 1440.0, 5.0, "1440"),
            ],
            variants: vec![
                var("work", "Work mode; open the chip to switch.", &[]),
                var(
                    "autoresearch",
                    "Autoresearch with a 10-hour budget.",
                    &[("mode", "auto"), ("budget", "600")],
                ),
            ],
            render: run_modes,
            code: run_modes_code,
            keyboard: vec![
                ("Enter / Space", "Open the menu"),
                ("↑ / ↓", "Move between modes"),
                ("Enter", "Pick; in the minutes field, set the budget"),
                ("Esc", "Close"),
            ],
            a11y: vec![
                "The chip has aria-haspopup=menu; modes are menuitemradio with aria-checked.",
                "Each mode has its own shape, so the chip reads without colour.",
            ],
            responsive: vec!["The menu opens upwards above a bottom composer."],
            height: 470,
            align: Align::Start,
            ..mk(
                "run-mode-menu",
                "Run mode menu",
                "composer",
                "How much the agent may do, Ask, Plan, Work or Autoresearch, each with its own shape; Autoresearch takes a time budget from presets or typed minutes.",
                "RunModeMenu",
                "RunModeMenuEl",
                "run-mode-menu",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                tog("live", "Run live", true),
                rng("speed", "Run minutes per second", 5.0, 120.0, 5.0, "40"),
                rng("at", "Stopped at (%)", 0.0, 100.0, 1.0, "55"),
                rng("budget", "Budget (min)", 240.0, 1440.0, 60.0, "1440"),
                rng("count", "Experiments", 6.0, 80.0, 1.0, "48"),
                rng("band", "Hours per row", 1.0, 6.0, 1.0, "4"),
            ],
            variants: vec![
                var("live", "Experiments pop in as the run reaches them.", &[]),
                var(
                    "finished",
                    "The budget spent.",
                    &[("live", "false"), ("at", "100")],
                ),
                var(
                    "short",
                    "A 4-hour run in hour rows.",
                    &[("budget", "240"), ("band", "1"), ("count", "18")],
                ),
            ],
            render: experiments,
            code: experiments_code,
            keyboard: vec![
                ("Tab", "Through the legend and each mark"),
                ("Enter / Space", "On a legend item: pin that outcome"),
            ],
            a11y: vec![
                "Every mark is a button labelled with its id, outcome and note; the note line is role=status.",
                "Outcomes differ by shape (and crashes by tone), so the grid reads in monochrome.",
                "Budget use is a progressbar.",
            ],
            responsive: vec!["Marks wrap within each band row."],
            height: 340,
            live: true,
            ..mk(
                "experiment-grid",
                "Experiment grid",
                "chat",
                "A budgeted research run's experiments as shapes in bands of hours: kept, reverted and crashed read at a glance, new ones pop in, and the legend pins one outcome.",
                "ExperimentGrid",
                "ExperimentGridEl",
                "experiment-grid",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                sel("layout", "Layout", &["grid", "list"], "grid"),
                tog("sidebar", "Sidebar", true),
            ],
            variants: vec![
                var("grid", "Cards with the document list beside them.", &[]),
                var(
                    "list",
                    "List rows, no sidebar.",
                    &[("layout", "list"), ("sidebar", "false")],
                ),
            ],
            render: doc_grid,
            code: doc_grid_code,
            keyboard: vec![
                ("Tab", "Through tabs, sort, layout and cards"),
                ("Enter", "Open a focused card"),
            ],
            a11y: vec![
                "Tabs are role=tab with aria-selected; layout buttons are aria-pressed.",
                "Stars are toggle buttons that say what they will do.",
            ],
            responsive: vec![
                "Three columns; the sidebar hides and two columns remain under 560 px of its own width.",
            ],
            height: 520,
            align: Align::Start,
            ..mk(
                "doc-grid",
                "Document grid",
                "shells",
                "Local documents as a sidebar list and a card grid: My documents or Favorites, search, Last visited or Title, grid or list, stars in place.",
                "DocGrid",
                "DocGridEl",
                "doc-grid",
                "parity/home.rs",
            )
        },
        Demo {
            controls: vec![
                rng("agents", "Background subagents", 1.0, 8.0, 1.0, "3"),
                rng("every", "Poll every (s)", 10.0, 120.0, 5.0, "30"),
                rng("mins", "Window (min)", 10.0, 300.0, 10.0, "60"),
                rng("transcript", "Transcript (K tokens)", 5.0, 200.0, 5.0, "40"),
            ],
            variants: vec![
                var("hour", "Three subagents for an hour.", &[]),
                var(
                    "long",
                    "Six subagents over five hours.",
                    &[("agents", "6"), ("mins", "300")],
                ),
            ],
            render: polling,
            code: polling_code,
            keyboard: vec![],
            a11y: vec!["Numbers are text next to each bar; nothing relies on bar length alone."],
            responsive: vec!["Up to 440 px wide."],
            height: 240,
            ..mk(
                "polling-cost",
                "Polling cost",
                "data-display",
                "What background subagents spend checking in: re-reading the whole transcript every poll against a status ping, and how much of the limit that buys back.",
                "PollingCost",
                "PollingCostEl",
                "polling-cost",
                "parity/home.rs",
            )
        },
    ]
}
