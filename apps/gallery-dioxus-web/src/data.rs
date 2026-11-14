//! Sample data for the demos. Invented, but shaped like a real agent run: a coding agent fixing a
//! refresh-token race, a newsletter send waiting for approval, a checkout built in two attempts.

use std::time::Duration;

use oxyn_core::{
    ActivityKind, AgentState, ClockMs, ErrorKind, RequestRef, StallLevel, ToolRef,
    approval::{ApprovalCard as Card, ApprovalQueue, ApprovalRequest, Risk},
    checkpoint::{Checkpoint, CheckpointStatus},
    composer::{
        AttachmentItem, AttachmentState, Capabilities, CategoryUsage, ContextCategory as C,
        ContextRef, ContextUsage, EffortLevel, ModelCatalog, ModelInfo, ModelStatus,
        ReasoningSupport, model::Tier3,
    },
    primitives::{menu::MenuItem, select::SelectOption},
    run::{
        AgentNode, ArgValue, Attempt, CancelledBy, ItemStatus, McpServer, McpStatus, Plan,
        PlanItem, RunGraph, Target, ToolCall, ToolCallState as S, ToolErrorKind, ToolKind,
        plan::ItemChange, stack::Entry,
    },
};

/// The clock every demo starts at.
pub const NOW: ClockMs = ClockMs(1_000_000);

/// `ms_ago` before [`NOW`].
pub fn at(ms_ago: u64) -> ClockMs {
    ClockMs(NOW.0 - ms_ago)
}

/// The 13 agent states, kebab ids as in the tokens.
pub const STATES: [&str; 13] = [
    "idle",
    "composing",
    "submitted",
    "thinking",
    "streaming",
    "tool-running",
    "awaiting-approval",
    "queued",
    "steering",
    "stalled",
    "done",
    "error",
    "cancelled",
];

/// The busy states (what a loader shows).
pub const BUSY: [&str; 6] = [
    "thinking",
    "tool-running",
    "awaiting-approval",
    "stalled",
    "done",
    "error",
];

fn tool_ref(name: &str, activity: ActivityKind, target: &str) -> ToolRef {
    ToolRef {
        id: "t1".into(),
        name: name.into(),
        activity,
        target: Some(target.into()),
    }
}

/// An agent state by id.
pub fn state(id: &str) -> AgentState {
    match id {
        "composing" => AgentState::Composing,
        "submitted" => AgentState::Submitted,
        "thinking" => AgentState::Thinking { header: None },
        "thinking-header" => AgentState::Thinking {
            header: Some("Checking the auth flow".into()),
        },
        "streaming" => AgentState::Streaming,
        "tool-running" => AgentState::ToolRunning {
            tool: tool_ref("bash", ActivityKind::Run, "cargo test"),
            concurrent: 1,
        },
        "awaiting-approval" => AgentState::AwaitingApproval {
            request: RequestRef {
                id: "r1".into(),
                tool: None,
            },
        },
        "queued" => AgentState::Queued { count: 2 },
        "steering" => AgentState::Steering,
        "stalled" => AgentState::Stalled {
            since: at(6_000),
            level: StallLevel::Soft,
            was: Box::new(AgentState::Thinking { header: None }),
        },
        "done" => AgentState::Done {
            elapsed: Duration::from_secs(72),
            waited: Duration::ZERO,
        },
        "error" => AgentState::Error {
            kind: ErrorKind::Timeout,
        },
        "cancelled" => AgentState::Cancelled {
            after: Duration::from_secs(42),
        },
        _ => AgentState::Idle,
    }
}

/// Rust source for an agent state (for generated code).
pub fn state_code(id: &str) -> &'static str {
    match id {
        "composing" => "AgentState::Composing",
        "submitted" => "AgentState::Submitted",
        "thinking" => "AgentState::Thinking { header: None }",
        "streaming" => "AgentState::Streaming",
        "tool-running" => "AgentState::ToolRunning { tool, concurrent: 1 }",
        "awaiting-approval" => "AgentState::AwaitingApproval { request }",
        "queued" => "AgentState::Queued { count: 2 }",
        "steering" => "AgentState::Steering",
        "stalled" => "AgentState::Stalled { since, level: StallLevel::Soft, was }",
        "done" => "AgentState::Done { elapsed, waited }",
        "error" => "AgentState::Error { kind: ErrorKind::Timeout }",
        "cancelled" => "AgentState::Cancelled { after }",
        _ => "AgentState::Idle",
    }
}

// ── Composer ────────────────────────────────────────────────────────────────

/// Effort levels most models allow.
pub fn levels() -> ReasoningSupport {
    ReasoningSupport::Levels {
        allowed: vec![
            EffortLevel::Low,
            EffortLevel::Medium,
            EffortLevel::High,
            EffortLevel::Max,
        ],
        default: EffortLevel::Medium,
    }
}

/// Five models from three providers (one needs a key, one is rate limited).
pub fn catalog() -> ModelCatalog {
    let m =
        |id: &str, p: &str, name: &str, vision, ctx, speed, cost: Option<&str>, status| ModelInfo {
            id: id.into(),
            provider: p.into(),
            name: name.into(),
            short_name: None,
            caps: Capabilities {
                vision,
                tools: true,
                reasoning: if p == "local" {
                    ReasoningSupport::Toggle
                } else {
                    levels()
                },
            },
            context_tokens: ctx,
            speed: Some(speed),
            cost: cost.map(str::to_string),
            status,
        };
    ModelCatalog {
        providers: vec![
            ("ember".into(), "Ember".into()),
            ("slate".into(), "Slate".into()),
            ("local".into(), "On device".into()),
        ],
        models: vec![
            m(
                "ember/swift-2",
                "ember",
                "Ember Swift 2",
                true,
                200_000,
                Tier3::Fast,
                Some("$1 / $5 per Mtok"),
                ModelStatus::Available,
            ),
            m(
                "ember/deep-2",
                "ember",
                "Ember Deep 2",
                true,
                1_000_000,
                Tier3::Slow,
                Some("$5 / $25 per Mtok"),
                ModelStatus::Available,
            ),
            m(
                "slate/one",
                "slate",
                "Slate One",
                false,
                128_000,
                Tier3::Medium,
                None,
                ModelStatus::NeedsKey,
            ),
            m(
                "slate/vision",
                "slate",
                "Slate Vision",
                true,
                256_000,
                Tier3::Medium,
                None,
                ModelStatus::RateLimited {
                    until: ClockMs(NOW.0 + 252_000),
                },
            ),
            m(
                "local/q7",
                "local",
                "Local Q 7B",
                false,
                32_000,
                Tier3::Fast,
                Some("Free · on device"),
                ModelStatus::Available,
            ),
        ],
        auto: true,
    }
}

/// Favourite model ids.
pub fn favorites() -> Vec<String> {
    vec![
        "ember/swift-2".into(),
        "ember/deep-2".into(),
        "local/q7".into(),
    ]
}

/// Context window usage with `history` tokens of conversation.
pub fn usage(history: u32) -> ContextUsage {
    let c = |category, tokens| CategoryUsage { category, tokens };
    ContextUsage {
        window: Some(200_000),
        categories: vec![
            c(C::System, 9_200),
            c(C::Tools, 14_600),
            c(C::Memory, 3_100),
            c(C::Skills, 600),
            c(C::Files, 18_400),
            c(C::History, history),
            c(C::Reserved, 20_000),
        ],
        exact: true,
        at: NOW,
    }
}

/// Files in every upload state.
pub fn attachments() -> Vec<AttachmentItem> {
    vec![
        AttachmentItem::file(
            1,
            "login-flow.png",
            1_400_000,
            AttachmentState::Uploading {
                progress: Some(0.64),
            },
        ),
        AttachmentItem::file(
            2,
            "spec.pdf",
            812_000,
            AttachmentState::Ready {
                meta: "PDF · 12 pages".into(),
            },
        ),
        AttachmentItem::file(
            3,
            "auth.rs",
            18_000,
            AttachmentState::Ready {
                meta: String::new(),
            },
        ),
        AttachmentItem::file(
            4,
            "q3-revenue.csv",
            2_400_000,
            AttachmentState::Error {
                reason: "Upload failed".into(),
                retryable: true,
            },
        ),
    ]
}

/// Context references (file, repo, selection, URL).
pub fn context_refs() -> Vec<AttachmentItem> {
    vec![
        AttachmentItem::context(10, ContextRef::File("src/middleware/auth.rs".into())),
        AttachmentItem::context(
            11,
            ContextRef::Repo {
                branch: "main".into(),
                changes: 3,
            },
        ),
        AttachmentItem::context(
            12,
            ContextRef::Selection {
                path: "src/token.rs".into(),
                from: 12,
                to: 40,
            },
        ),
        AttachmentItem::context(13, ContextRef::Url("https://docs.rs/tokio".into())),
    ]
}

// ── Tool calls, plans, sub-agents ───────────────────────────────────────────

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

/// Tool-call state ids for the controls.
pub const CALL_STATES: [&str; 12] = [
    "pending",
    "streaming-input",
    "running",
    "streaming-output",
    "awaiting-approval",
    "succeeded",
    "edit",
    "failed",
    "retry",
    "cancelled",
    "denied",
    "auth-needed",
];

/// One tool call in the given state.
pub fn tool_call(id: &str) -> ToolCall {
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
    match id {
        "pending" => call("pe", "read_file", ToolKind::Read, p("src/main.rs")),
        "streaming-input" => mcp.with_state(S::StreamingInput),
        "streaming-output" => {
            let mut c = call(
                "so",
                "web_fetch",
                ToolKind::Fetch,
                Target::Url("https://docs.rs/tokio/latest".into()),
            )
            .with_state(S::StreamingOutput {
                progress: Some((3, 5)),
            })
            .timed(at(4_200), None);
            c.last_event_at = at(200);
            c
        }
        "awaiting-approval" => call(
            "aw",
            "bash",
            ToolKind::Shell,
            Target::Command("rm -rf target/".into()),
        )
        .with_state(S::AwaitingApproval)
        .timed(at(6_000), None),
        "succeeded" => ok(mcp, 8_000, 1_400, Some("ENG-482")),
        "edit" => {
            let mut e = ok(
                call("e", "str_replace", ToolKind::Edit, p("src/auth/refresh.rs")),
                5_000,
                300,
                None,
            );
            e.lines = Some((12, 3));
            e
        }
        "failed" => call(
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
        "retry" => {
            let mut r = call(
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
            r.attempt = Attempt {
                n: 1,
                max: 3,
                next_at: Some(ClockMs(NOW.0 + 3_000)),
            };
            r
        }
        "cancelled" => call(
            "c",
            "bash",
            ToolKind::Shell,
            Target::Command("cargo bench".into()),
        )
        .with_state(S::Cancelled {
            by: CancelledBy::User,
        })
        .timed(at(9_000), Some(at(5_000))),
        "denied" => {
            call("d", "write_file", ToolKind::Create, p("/etc/hosts")).with_state(S::Denied {
                reason: Some("outside workspace".into()),
            })
        }
        "auth-needed" => {
            let mut a = mcp.with_state(S::AuthNeeded);
            a.server = Some(McpServer {
                name: "Linear".into(),
                status: McpStatus::NeedsAuth,
            });
            a
        }
        _ => running(
            call(
                "ru",
                "grep",
                ToolKind::Search,
                Target::Query("refresh_token".into()),
            ),
            2_400,
        ),
    }
}

/// An exploration turn: reads, a search, a test run, a web search and a timed-out fetch.
pub fn explore() -> (Vec<ToolCall>, Vec<Entry>) {
    let p = |s: &str| Target::Path(s.into());
    let calls = vec![
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
    ];
    let entries = vec![
        Entry::Call(0),
        Entry::Call(1),
        Entry::Call(2),
        Entry::Call(3),
        Entry::Call(4),
        Entry::Break,
        Entry::Call(5),
        Entry::Call(6),
    ];
    (calls, entries)
}

/// Three calls in flight at once.
pub fn parallel() -> Vec<ToolCall> {
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

/// A six-step plan: two done, one active, one blocked, one pending, one skipped.
pub fn plan() -> Plan {
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

/// The plan with every step still pending.
pub fn plan_proposed() -> Plan {
    let mut p = plan();
    for i in &mut p.items {
        i.status = ItemStatus::Pending;
        i.span = None;
        i.change = None;
    }
    p
}

/// The plan with every step done.
pub fn plan_done() -> Plan {
    let mut p = plan();
    for i in &mut p.items {
        if !matches!(i.status, ItemStatus::Skipped { .. }) {
            i.status = ItemStatus::Done;
        }
    }
    p
}

/// An orchestrator with five sub-agents in mixed states.
pub fn graph() -> RunGraph {
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
            kind: ErrorKind::Tool,
        },
    );
    d.started = Some(at(90_000));
    d.ended = Some(at(70_000));
    RunGraph {
        nodes: vec![o, r, t, f, v, d],
    }
}

// ── Approvals and checkpoints ───────────────────────────────────────────────

fn req(id: &str, title: &str, risk: Risk, opened_ago: u64) -> ApprovalRequest {
    ApprovalRequest::new(id, title, risk, at(opened_ago))
}

/// An approval request by risk tier.
pub fn approval(risk: &str, countdown: bool) -> Card {
    let mut r = match risk {
        "low" => {
            let mut r = req("l", "Run the test suite", Risk::Low, 5_000);
            r.description = Some("Reads only · inside the workspace".into());
            r.command = Some("cargo test -p oxyn-core".into());
            r.label = Some("Step 2 · bash".into());
            r
        }
        "high" => {
            let mut r = req("h", "Delete build artefacts", Risk::High, 700);
            r.description = Some("Outside git · 1.2 GB".into());
            r.command = Some("rm -rf target/ dist/".into());
            r.reasons = vec!["Deletes files that git does not track".into()];
            r
        }
        "critical" => {
            let mut r = req("c", "Charge the customer card", Risk::Critical, 4_000);
            r.description = Some("Stripe · live mode".into());
            r.amount = Some("$420.00".into());
            r.details = vec![
                ("Payee".into(), "Northwind Ltd".into()),
                ("Card".into(), "Visa ···· 4242".into()),
            ];
            r.reasons = vec![
                "Moves real money".into(),
                "Can't be undone from here".into(),
            ];
            r
        }
        _ => {
            let mut r = req("e", "Send campaign email", Risk::Medium, 18_000);
            r.description = Some("October product update · Newsletter".into());
            r.label = Some("Step 4 · gmail.send".into());
            r.details = vec![
                ("To".into(), "Customers – EU (42)".into()),
                ("Subject".into(), "What's new in October".into()),
                ("Send".into(), "Today, 9:00 AM".into()),
            ];
            r
        }
    };
    if countdown {
        r.expires_in = Some(Duration::from_secs(60));
    }
    Card::new(r)
}

/// Three open requests.
pub fn queue() -> ApprovalQueue {
    let mut q = ApprovalQueue::default();
    for (i, (t, r)) in [
        ("Read .env.example", Risk::Low),
        ("Edit src/auth/refresh.rs", Risk::Medium),
        ("Run npm install", Risk::Medium),
    ]
    .into_iter()
    .enumerate()
    {
        let mut x = req(&i.to_string(), t, r, 3_000);
        x.description = Some("Coder · this session".into());
        q.requests.push(x);
    }
    q
}

/// A run rewound once: attempt one failed, attempt two is running.
pub fn checkpoints() -> Vec<Checkpoint> {
    let mut v = vec![
        Checkpoint::new("c1", None, "Read the brief", 9),
        Checkpoint::new("c2", Some("c1"), "Built the pricing table", 64).files(3, 120, 4),
        Checkpoint::new("c3", Some("c2"), "Added the Stripe checkout", 98).files(2, 61, 9),
        Checkpoint::new("c4", Some("c3"), "Tests failing – 4 of 12", 131)
            .with(CheckpointStatus::Failed)
            .files(1, 12, 3),
        Checkpoint::new("c5", Some("c2"), "Switched to Stripe Elements", 169).files(4, 88, 41),
        Checkpoint::new("c6", Some("c5"), "Wiring the webhooks", 212)
            .with(CheckpointStatus::Running),
    ];
    v[2].cost_cents = Some(31);
    v[5].cost_cents = Some(42);
    v
}

/// Wall-clock labels for [`checkpoints`].
pub fn checkpoint_times() -> Vec<String> {
    [
        "3:31 PM", "3:33 PM", "3:36 PM", "3:38 PM", "3:42 PM", "3:45 PM",
    ]
    .map(String::from)
    .to_vec()
}

// ── Primitives ──────────────────────────────────────────────────────────────

/// A thread menu: label, actions with icons and shortcuts, a disabled item, a submenu, check and
/// radio items and a danger item.
pub fn menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::label("Thread"),
        MenuItem::action("rename", "Rename")
            .icon("pencil")
            .shortcut("Mod+R"),
        MenuItem::action("dup", "Duplicate")
            .icon("copy")
            .shortcut("Mod+D"),
        MenuItem::action("pin", "Pin to top")
            .icon("star")
            .disabled(),
        MenuItem::submenu(
            "move",
            "Move to",
            vec![
                MenuItem::action("inbox", "Inbox").icon("inbox"),
                MenuItem::action("archive", "Archive").icon("archive"),
            ],
        )
        .icon("folder"),
        MenuItem::divider(),
        MenuItem::checkbox("wrap", "Wrap long lines", true),
        MenuItem::radio("fast", "Fast model", "model", true),
        MenuItem::radio("careful", "Careful model", "model", false),
        MenuItem::divider(),
        MenuItem::action("del", "Delete thread")
            .icon("trash")
            .shortcut("Mod+Backspace")
            .danger(),
    ]
}

/// Menu items with brand logos (affectUI `item.logo`) and a keep-open toggle.
pub fn menu_items_logos() -> Vec<MenuItem> {
    vec![
        MenuItem::label("Send to"),
        MenuItem::action("linear", "Create Linear issue")
            .logo("L")
            .shortcut("Mod+Shift+L"),
        MenuItem::action("github", "Open GitHub issue").logo("GH"),
        MenuItem::action("slack", "Share to Slack").logo("S"),
        MenuItem::action("notion", "Save to Notion").logo("N"),
        MenuItem::divider(),
        MenuItem::checkbox("follow", "Follow thread", true),
        MenuItem::action("pin", "Pin to sidebar")
            .icon("star")
            .keep_open(),
    ]
}

/// Model options grouped by provider.
pub fn model_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("swift", "Ember Swift 2")
            .desc("Fastest · $1 / Mtok")
            .icon("bolt")
            .group("Ember"),
        SelectOption::new("deep", "Ember Deep 2")
            .desc("Deepest · $5 / Mtok")
            .icon("brain")
            .group("Ember"),
        SelectOption::new("one", "Slate One")
            .desc("Balanced · needs a key")
            .icon("spark")
            .group("Slate"),
        SelectOption::new("q7", "Local Q 7B")
            .desc("Free · on device")
            .icon("bolt")
            .group("On device"),
    ]
}
