//! Renders every free family-F variant (tool calls, stacks, plans, sub-agent tree) to static HTML.
//! `cargo run -p oxyn-dioxus --example run_sheet -- tokens/dist/oxyn.css > sheet.html`

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ActivityKind, AgentState, ClockMs, RequestRef, ToolRef,
    run::{
        AgentNode, ArgValue, Attempt, CancelledBy, ItemStatus, McpServer, McpStatus, Plan,
        PlanItem, PlanState, RunGraph, Target, ToolCall, ToolCallState as S, ToolErrorKind,
        ToolKind, plan::ItemChange, stack::Entry,
    },
};
use oxyn_dioxus::{
    PlanCard, RUN_CSS, StepTimeline, SubagentTree, TodoList, ToolCallRow, ToolStack,
};

const NOW: ClockMs = ClockMs(1_000_000);

fn at(ms_ago: u64) -> ClockMs {
    ClockMs(NOW.0 - ms_ago)
}

fn call(id: &str, name: &str, kind: ToolKind, target: Target) -> ToolCall {
    ToolCall::new(id, name, at(60_000))
        .with_kind(kind)
        .with_target(target)
}

fn ok(c: ToolCall, ago: u64, took: u64, summary: Option<&str>) -> ToolCall {
    c.with_state(S::Succeeded {
        summary: summary.map(str::to_string),
    })
    .timed(at(ago + took), Some(at(ago)))
}

fn running(c: ToolCall, for_ms: u64) -> ToolCall {
    c.with_state(S::Running).timed(at(for_ms), None)
}

fn states() -> Vec<(&'static str, ToolCall)> {
    let p = |s: &str| Target::Path(s.into());
    let mut mcp = call(
        "m",
        "mcp__linear__create_issue",
        ToolKind::Mcp,
        Target::Label("create_issue".into()),
    );
    mcp.server = Some(McpServer {
        name: "Linear".into(),
        status: McpStatus::Connected,
    });
    mcp.args = Some(ArgValue::obj([
        ("team", ArgValue::s("ENG")),
        (
            "title",
            ArgValue::s("Refresh-token race in auth middleware"),
        ),
        ("api_token", ArgValue::s("lin_123")),
    ]));
    let mut auth = mcp.clone();
    auth.server = Some(McpServer {
        name: "Linear".into(),
        status: McpStatus::NeedsAuth,
    });
    let mut retry = call(
        "r",
        "bash",
        ToolKind::Shell,
        Target::Command("cargo test -p oxyn-core".into()),
    )
    .with_state(S::Failed {
        kind: ToolErrorKind::Exit,
        code: Some(101),
        message: Some("2 tests failed".into()),
    })
    .timed(at(9_000), Some(at(2_000)));
    retry.attempt = Attempt {
        n: 1,
        max: 3,
        next_at: Some(ClockMs(NOW.0 + 3_000)),
    };
    let mut edit = ok(
        call("e", "str_replace", ToolKind::Edit, p("src/auth/refresh.rs")),
        5_000,
        300,
        None,
    );
    edit.lines = Some((12, 3));
    let mut stream_out = call(
        "so",
        "web_fetch",
        ToolKind::Fetch,
        Target::Url("https://docs.rs/tokio/latest".into()),
    )
    .with_state(S::StreamingOutput {
        progress: Some((3, 5)),
    })
    .timed(at(4_200), None);
    stream_out.last_event_at = at(200);
    let mut stalled = running(
        call(
            "st",
            "bash",
            ToolKind::Shell,
            Target::Command("npm run e2e".into()),
        ),
        160_000,
    );
    stalled.last_event_at = at(130_000);
    vec![
        (
            "pending",
            call("pe", "read_file", ToolKind::Read, p("src/main.rs")),
        ),
        ("streaming input", mcp.clone().with_state(S::StreamingInput)),
        (
            "running",
            running(
                call(
                    "ru",
                    "grep",
                    ToolKind::Search,
                    Target::Query("refresh_token".into()),
                ),
                2_400,
            ),
        ),
        ("streaming output", stream_out),
        (
            "awaiting approval",
            call(
                "aw",
                "bash",
                ToolKind::Shell,
                Target::Command("rm -rf target/".into()),
            )
            .with_state(S::AwaitingApproval)
            .timed(at(6_000), None),
        ),
        (
            "succeeded · ✓ hold",
            ok(
                call(
                    "s1",
                    "read_file",
                    ToolKind::Read,
                    p("src/middleware/auth.rs"),
                ),
                300,
                820,
                Some("212 lines"),
            ),
        ),
        ("succeeded", ok(mcp.clone(), 8_000, 1_400, Some("ENG-482"))),
        ("edit", edit),
        (
            "failed",
            call(
                "f",
                "bash",
                ToolKind::Shell,
                Target::Command("cargo build --release".into()),
            )
            .with_state(S::Failed {
                kind: ToolErrorKind::Exit,
                code: Some(101),
                message: Some("error[E0425]: cannot find value `tok`".into()),
            })
            .timed(at(14_000), Some(at(4_000))),
        ),
        ("retry scheduled", retry),
        (
            "cancelled",
            call(
                "c",
                "bash",
                ToolKind::Shell,
                Target::Command("cargo bench".into()),
            )
            .with_state(S::Cancelled {
                by: CancelledBy::User,
            })
            .timed(at(9_000), Some(at(5_000))),
        ),
        (
            "denied",
            call("d", "write_file", ToolKind::Create, p("/etc/hosts")).with_state(S::Denied {
                reason: Some("outside workspace".into()),
            }),
        ),
        ("auth needed", auth.with_state(S::AuthNeeded)),
        ("stalled", stalled),
    ]
}

fn explore() -> Vec<ToolCall> {
    let p = |s: &str| Target::Path(s.into());
    vec![
        ok(
            call("x1", "read_file", ToolKind::Read, p("src/auth.rs")),
            30_000,
            400,
            None,
        ),
        ok(
            call("x2", "read_file", ToolKind::Read, p("src/session.rs")),
            29_000,
            300,
            None,
        ),
        ok(
            call(
                "x3",
                "grep",
                ToolKind::Search,
                Target::Query("refresh_token".into()),
            ),
            28_000,
            200,
            Some("12 matches"),
        ),
        ok(
            call("x4", "read_file", ToolKind::Read, p("src/store.rs")),
            27_000,
            300,
            None,
        ),
        ok(
            call(
                "x5",
                "bash",
                ToolKind::Shell,
                Target::Command("cargo test auth".into()),
            ),
            20_000,
            6_400,
            Some("14 passed"),
        ),
        ok(
            call(
                "x6",
                "web_search",
                ToolKind::WebSearch,
                Target::Query("tokio mutex across await".into()),
            ),
            12_000,
            900,
            None,
        ),
        call(
            "x7",
            "web_fetch",
            ToolKind::Fetch,
            Target::Url("https://docs.rs/tokio".into()),
        )
        .with_state(S::Failed {
            kind: ToolErrorKind::Timeout,
            code: None,
            message: None,
        })
        .timed(at(11_000), Some(at(1_000))),
    ]
}

fn parallel() -> Vec<ToolCall> {
    vec![
        running(
            call(
                "p1",
                "bash",
                ToolKind::Shell,
                Target::Command("cargo test".into()),
            ),
            3_200,
        ),
        running(
            call(
                "p2",
                "web_search",
                ToolKind::WebSearch,
                Target::Query("axum middleware order".into()),
            ),
            3_190,
        ),
        ok(
            call(
                "p3",
                "read_file",
                ToolKind::Read,
                Target::Path("Cargo.toml".into()),
            ),
            2_000,
            1_170,
            None,
        ),
    ]
}

fn plan() -> Plan {
    let mut p = Plan::new(
        Some("Fix the refresh-token race"),
        vec![
            PlanItem::new("1", "Read the auth middleware")
                .verbs("Reading the auth middleware", "Read the auth middleware")
                .with(ItemStatus::Done)
                .span(NOW.0 - 60_000, Some(NOW.0 - 52_000)),
            PlanItem::new("2", "Write a failing test")
                .verbs("Writing a failing test", "Wrote a failing test")
                .with(ItemStatus::Done)
                .span(NOW.0 - 52_000, Some(NOW.0 - 31_000)),
            PlanItem::new("3", "Hold the lock across the refresh")
                .verbs(
                    "Holding the lock across the refresh",
                    "Held the lock across the refresh",
                )
                .with(ItemStatus::Active)
                .span(NOW.0 - 31_000, None),
            PlanItem::new("4", "Run the auth tests").with(ItemStatus::Blocked { on: "#3".into() }),
            PlanItem::new("5", "Update the changelog").with(ItemStatus::Pending),
            PlanItem::new("6", "Bump the session schema").with(ItemStatus::Skipped {
                reason: Some("no schema change needed".into()),
            }),
        ],
    );
    p.items[4].change = Some((ItemChange::Added, at(500)));
    p.items[2].calls = vec!["x1".into(), "x3".into(), "x5".into()];
    p.items[0].calls = vec!["x2".into()];
    p
}

fn graph() -> RunGraph {
    let tool = |a, t: &str| AgentState::ToolRunning {
        tool: ToolRef {
            id: "t".into(),
            name: "x".into(),
            activity: a,
            target: Some(t.into()),
        },
        concurrent: 1,
    };
    let mut o = AgentNode::new(
        "o",
        None,
        "Orchestrator",
        4,
        AgentState::Thinking {
            header: Some("Planning the migration".into()),
        },
    );
    o.started = Some(at(134_000));
    let mut r = AgentNode::new(
        "r",
        Some("o"),
        "Researcher",
        2,
        tool(ActivityKind::Read, "14 files"),
    );
    r.started = Some(at(48_000));
    let mut t = AgentNode::new(
        "t",
        Some("o"),
        "Test runner",
        1,
        AgentState::AwaitingApproval {
            request: RequestRef {
                id: "q".into(),
                tool: None,
            },
        },
    );
    t.started = Some(at(31_000));
    t.request = Some("run `cargo test`".into());
    let mut f = AgentNode::new(
        "f",
        Some("t"),
        "Fixture gen",
        6,
        AgentState::Done {
            elapsed: Duration::from_secs(12),
            waited: Duration::ZERO,
        },
    );
    f.started = Some(at(40_000));
    f.ended = Some(at(28_000));
    f.result = Some("3 fixtures".into());
    let mut v = AgentNode::new(
        "v",
        Some("o"),
        "Reviewer",
        5,
        AgentState::Done {
            elapsed: Duration::from_secs(65),
            waited: Duration::ZERO,
        },
    );
    v.started = Some(at(120_000));
    v.ended = Some(at(55_000));
    v.result = Some("2 findings".into());
    let mut d = AgentNode::new(
        "d",
        Some("o"),
        "Docs writer",
        7,
        AgentState::Error {
            kind: oxyn_core::ErrorKind::Tool,
        },
    );
    d.started = Some(at(90_000));
    d.ended = Some(at(70_000));
    RunGraph {
        nodes: vec![o, r, t, f, v, d],
    }
}

#[component]
fn Sheet() -> Element {
    let st = states();
    let ex = explore();
    let ex_entries = vec![
        Entry::Call(0),
        Entry::Call(1),
        Entry::Call(2),
        Entry::Call(3),
        Entry::Call(4),
        Entry::Break,
        Entry::Call(5),
        Entry::Call(6),
    ];
    let par = parallel();
    let pl = plan();
    let mut drafting = pl.clone();
    drafting.state = PlanState::Drafting;
    let mut proposed = pl.clone();
    for i in &mut proposed.items {
        i.status = ItemStatus::Pending;
        i.span = None;
        i.change = None;
    }
    let mut exec = pl.clone();
    exec.state = PlanState::Executing;
    let mut sup = proposed.clone();
    sup.state = PlanState::Superseded { by: 3 };
    sup.version = 2;
    let mut done = pl.clone();
    for i in &mut done.items {
        if !matches!(i.status, ItemStatus::Skipped { .. }) {
            i.status = ItemStatus::Done;
        }
    }
    let g = graph();
    rsx! {
        div { class: "page",
            header { h1 { "Oxyn · tool calls, plans, sub-agents" } p { "Plan #20 · family F free variants · Dioxus renderer · monochrome chrome, semantic colour" } }
            div { class: "cols",
                div { class: "card",
                    h2 { "tool-call/row · every state" }
                    for (label, c) in st.iter().cloned() {
                        div { class: "srow", span { class: "lbl", "{label}" } div { class: "v", ToolCallRow { call: c, now: NOW } } }
                    }
                }
                div { class: "col",
                    div { class: "card",
                        h2 { "tool-call/card · expanded" }
                        ToolCallRow { call: st[6].1.clone(), now: NOW, variant: "card", expanded: Some(true) }
                        div { class: "gap8" }
                        ToolCallRow { call: st[8].1.clone(), now: NOW, variant: "card" }
                        h2 { class: "gap", "tool-call/minimal" }
                        for i in [2usize, 5, 7, 8, 10] { ToolCallRow { call: st[i].1.clone(), now: NOW, variant: "minimal" } }
                        h2 { class: "gap", "tool-call/terminal" }
                        div { class: "term", for i in [2usize, 5, 8, 9, 13] { ToolCallRow { call: st[i].1.clone(), now: NOW, variant: "terminal" } } }
                    }
                    div { class: "card",
                        h2 { "tool-stack/transcript · groups" }
                        p { class: "prose", "I'll check how the refresh path takes the lock." }
                        ToolStack { calls: ex.clone(), entries: ex_entries.clone(), now: NOW }
                        h2 { class: "gap", "parallel · live" }
                        ToolStack { calls: par.clone(), now: NOW }
                        h2 { class: "gap", "tool-stack/worked-for" }
                        ToolStack { calls: ex.clone(), entries: ex_entries.clone(), now: NOW, variant: "worked-for", turn_done: Some(Duration::from_secs(31)) }
                        h2 { class: "gap", "density hidden" }
                        ToolStack { calls: par.clone(), now: NOW, density: "hidden" }
                        h2 { class: "gap", "tool-stack/terminal" }
                        div { class: "term", ToolStack { calls: ex.clone(), entries: ex_entries.clone(), now: NOW, variant: "terminal" } }
                    }
                }
            }
            div { class: "cols",
                div { class: "col",
                    div { class: "card",
                        h2 { "todo-list/checklist" }
                        TodoList { plan: pl.clone(), now: NOW }
                        h2 { class: "gap", "todo-list/dock · live, expanded" }
                        TodoList { plan: pl.clone(), now: NOW, variant: "dock", agent: AgentState::Streaming, expanded: true }
                        h2 { class: "gap", "dock · stopped · done" }
                        div { class: "stack",
                            TodoList { plan: pl.clone(), now: NOW, variant: "dock", agent: AgentState::Cancelled { after: Duration::from_secs(42) } }
                            TodoList { plan: done.clone(), now: NOW, variant: "dock", done_at: Some(at(1_000)) }
                        }
                    }
                    div { class: "card",
                        h2 { "step-timeline/vertical" }
                        StepTimeline { plan: pl.clone(), calls: ex.clone(), now: NOW }
                        h2 { class: "gap", "folded" }
                        StepTimeline { plan: done.clone(), now: NOW, fold_on_done: true }
                    }
                }
                div { class: "col",
                    div { class: "card",
                        h2 { "plan-card/document · proposed" }
                        PlanCard { plan: proposed.clone(), now: NOW, gated: true }
                        h2 { class: "gap", "drafting · executing · superseded" }
                        div { class: "two",
                            PlanCard { plan: drafting, now: NOW }
                            div { class: "stack", PlanCard { plan: exec, now: NOW } PlanCard { plan: sup, now: NOW } }
                        }
                    }
                    div { class: "card",
                        h2 { "subagent-tree/indented" }
                        SubagentTree { graph: g.clone(), now: NOW }
                        h2 { class: "gap", "subagent-tree/rail" }
                        SubagentTree { graph: g.clone(), now: NOW, variant: "rail" }
                        h2 { class: "gap", "subagent-tree/dock · collapsed root" }
                        div { class: "two",
                            SubagentTree { graph: g.clone(), now: NOW, variant: "dock" }
                            SubagentTree { graph: g.clone(), now: NOW, collapsed: vec!["o".to_string()] }
                        }
                        h2 { class: "gap", "subagent-tree/inline-wrapper" }
                        SubagentTree { graph: g.clone(), now: NOW, variant: "inline-wrapper" }
                        h2 { class: "gap", "subagent-tree/terminal" }
                        SubagentTree { graph: g.clone(), now: NOW, variant: "terminal" }
                    }
                }
            }
        }
    }
}

const PAGE_CSS: &str = r#"
html,body{margin:0;background:#0B0B0C}
.page{padding:32px;width:1460px;font-family:Inter,system-ui,sans-serif;color:var(--oxyn-color-fg-default)}
header h1{margin:0;font-size:24px;font-weight:600}
header p{margin:4px 0 22px;color:var(--oxyn-color-fg-subtle);font-size:13px}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:18px;margin-bottom:18px;align-items:start}
.col{display:flex;flex-direction:column;gap:18px}
.card{background:var(--oxyn-color-bg-base);border:1px solid var(--oxyn-color-border-subtle);border-radius:16px;padding:18px}
.card h2{margin:0 0 10px;font-size:11.5px;letter-spacing:.06em;text-transform:uppercase;color:var(--oxyn-color-fg-muted);font-weight:600}
.card h2.gap{margin-top:22px}
.srow{display:grid;grid-template-columns:130px 1fr;align-items:start;padding:2px 0}
.srow .lbl{font-family:"DejaVu Sans Mono",monospace;font-size:11px;color:var(--oxyn-color-fg-subtle);padding-top:9px}
.srow .v{min-width:0}
.term{background:var(--oxyn-color-surface-sunken);border-radius:10px;padding:10px 12px}
.prose{margin:0 0 6px 8px;color:var(--oxyn-color-fg-muted)}
.gap8{height:8px}
.stack{display:flex;flex-direction:column;gap:10px;align-items:flex-start}
.stack>.oxyn-todo{align-self:stretch}
.two{display:grid;grid-template-columns:1fr 1fr;gap:12px;align-items:start}
"#;

fn main() {
    let tokens = std::env::args()
        .nth(1)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let mut dom = VirtualDom::new(Sheet);
    dom.rebuild_in_place();
    let body = dioxus_ssr::render(&dom);
    println!(
        "<!doctype html><html><head><meta charset=utf-8><style>{tokens}{RUN_CSS}{PAGE_CSS}</style></head><body>{body}</body></html>"
    );
}
