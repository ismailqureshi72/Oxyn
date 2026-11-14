//! Chat (plan #27, chunk 2): streaming markdown, reasoning, thinking traces, source cards, AI and
//! cited answers, response feedback, message bubble, starter prompts, chat thread, AI chat,
//! document extraction, research run, support conversation, recommendation and context cards.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    chat::{
        sources::{Claim, ContextCard, Field, Source},
        thread::{ChatMessage, Delivery, Starter},
    },
};
use oxyn_dioxus::{
    AiAnswer, AiChat, ChatThread, ContextCards, DocumentExtraction, MessageBubble, ReasoningBlock,
    RecommendationCard, ResearchRun, ResponseFeedback, SourceCards, StarterPrompts,
    StreamingMarkdown, SupportConversation, ThinkingTraces,
};

use super::{Code, mk, q, rng, sel, tog, var};
use crate::{Align, BASE, Demo, Vals};

fn s(x: &str) -> String {
    x.to_string()
}

fn el(now: ClockMs, v: &Vals) -> u64 {
    if v.b("finished") {
        u64::MAX
    } else {
        now.0.saturating_sub(BASE.0)
    }
}

const MD: &str = "## Why the test is flaky\n\nTokens minted in the **last second** of a minute expire a few milliseconds before `verify` runs, so the strict check rejects them [1].\n\n### Fix\n\n1. Add a 30 s leeway to the expiry check [2].\n2. Freeze the clock at `:59.999` in a regression test.\n3. Rerun the suite three times.\n\n```rust\nlet mut v = Validation::new(Algorithm::RS256);\nv.leeway = 30;\n```\n\n| Run | Result | Time |\n|---|---|---|\n| 1 | pass | 41 s |\n| 2 | pass | 39 s |\n| 3 | pass | 40 s |\n\n> The change is two lines in `auth/token.rs` and one new test.";

fn sources() -> Vec<Source> {
    vec![
        Source::new("RFC 9068: JWT Profile for OAuth 2.0 Access Tokens", "https://www.rfc-editor.org/rfc/rfc9068").site("RFC Editor").published("Oct 2021").snippet("Resource servers MUST check the exp claim, allowing for a small leeway to account for clock skew."),
        Source::new("Validation in jsonwebtoken", "https://docs.rs/jsonwebtoken/latest/jsonwebtoken/struct.Validation.html").site("docs.rs").snippet("Add some leeway (in seconds) to the exp and nbf validation to account for clock skew."),
        Source::new("Flaky auth test at the minute boundary · Issue #482", "https://github.com/acme/atlas/issues/482").site("GitHub"),
        Source::new("Clock skew in distributed systems", "https://en.wikipedia.org/wiki/Clock_skew").site("Wikipedia").snippet("Clock skew is the phenomenon in which the same sourced clock signal arrives at different components at different times."),
        Source::new("Date.now() and timers", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/now").site("MDN"),
        Source::new("Testing time-dependent code", "https://stackoverflow.com/questions/2425721").site("Stack Overflow"),
    ]
}

// ─────────────────────────────── streaming markdown ──

fn smd(v: &Vals, now: ClockMs) -> Element {
    let cps = match v.s("pace") {
        "calm" => 60.0,
        "fast" => 400.0,
        "instant" => 0.0,
        _ => 140.0,
    };
    rsx! { div { class: "oxd-w560", StreamingMarkdown { text: MD, elapsed_ms: el(now, v), cps, first_token_ms: v.u("first_token") as u64, phase: v.string("phase"), caret: v.b("caret"), reveal: v.string("reveal") } } }
}

fn smd_code(v: &Vals) -> String {
    Code::new("StreamingMarkdown")
        .raw("text", "reply")
        .raw("elapsed_ms", "now - started")
        .s("phase", v.s("phase"), "streaming")
        .raw("first_token_ms", v.s("first_token"))
        .raw("on_cite", "move |n: u32| sources.highlight(n)")
        .done()
}

// ───────────────────────────────────────── reasoning ──

const REASONING: &str = "**Reading the failing test**\nauth_refresh fails about one run in twenty, always when the token is minted at second 59.\n\n**Checking the validator**\nThe expiry check is a strict less-than against now(), with no leeway for skew between minting and checking.\n\n**Considering fixes**\nA leeway of 30 seconds matches RFC 9068 guidance. Freezing the clock in the test makes the failure reproducible.\n\n**Plan**\nPatch token.rs, add a regression test, rerun three times.";

fn reasoning(v: &Vals, now: ClockMs) -> Element {
    let status = v.string("status");
    let e = if status == "thinking" {
        el(now, v).min(30_000)
    } else {
        0
    };
    rsx! {
        div { class: "oxd-w560",
            ReasoningBlock { reasoning: REASONING, status, seconds: if v.s("status") == "thinking" { None } else { Some(v.u("seconds") as u64) }, elapsed_ms: e, variant: v.string("variant"), level: v.string("level"), open: v.b("open"), error: Some(s("Rate limited by the provider after 3 retries.")) }
        }
    }
}

fn reasoning_code(v: &Vals) -> String {
    Code::new("ReasoningBlock")
        .raw("reasoning", "thoughts")
        .s("status", v.s("status"), "done")
        .raw("seconds", format!("Some({})", v.s("seconds")))
        .s("variant", v.s("variant"), "card")
        .s("level", v.s("level"), "normal")
        .raw("on_stop", "move |_| run.abort()")
        .done()
}

fn traces(v: &Vals, now: ClockMs) -> Element {
    let kind = v.string("kind");
    let rows: Vec<(String, String)> = match kind.as_str() {
        "search" => vec![
            (s("Searching the web"), s("jwt exp leeway clock skew")),
            (s("Reading RFC 9068"), s("rfc-editor.org")),
            (s("Reading jsonwebtoken docs"), s("docs.rs")),
            (s("Reading issue #482"), s("github.com")),
        ],
        "coding" => vec![
            (s("Read"), s("src/auth/token.rs")),
            (s("Edit"), s("src/auth/token.rs +2 −1")),
            (s("Write"), s("tests/auth_refresh.rs")),
            (s("Run"), s("cargo test auth")),
        ],
        "reasoning" => vec![
            (s("The failure clusters at second 59"), String::new()),
            (s("exp is compared strictly"), String::new()),
            (s("A 30 s leeway is standard"), String::new()),
        ],
        _ => vec![
            (s("Reproduce the failure"), String::new()),
            (s("Find the root cause"), String::new()),
            (s("Patch the validator"), String::new()),
            (s("Add a regression test"), String::new()),
            (s("Verify three runs"), String::new()),
        ],
    };
    rsx! { div { class: "oxd-w520", ThinkingTraces { kind, rows, elapsed_ms: el(now, v), step_ms: v.u("step") as u64, fail_at: if v.b("fail") { Some(2) } else { None } } } }
}

fn traces_code(v: &Vals) -> String {
    Code::new("ThinkingTraces")
        .s("kind", v.s("kind"), "steps")
        .raw(
            "rows",
            "vec![(\"Read\".into(), \"src/auth/token.rs\".into())]",
        )
        .raw("elapsed_ms", "now - started")
        .raw("step_ms", v.s("step"))
        .done()
}

// ──────────────────────────────────────── sources ──

fn source_cards(v: &Vals, now: ClockMs) -> Element {
    let status = v.string("status");
    let all = sources();
    let n = v.u("count").min(all.len());
    let e = if status == "searching" {
        el(now, v) % 4000
    } else {
        u64::MAX
    };
    rsx! {
        div { class: "oxd-w560", style: "padding-bottom:110px",
            SourceCards { sources: all[..n].to_vec(), status, query: Some(s("jwt expiry leeway clock skew")), duration_ms: Some(2100), max: v.u("max"), elapsed_ms: e, highlighted: if v.b("highlight") { Some(2) } else { None }, error: Some(s("The search provider timed out after 8s.")), expanded: v.b("expanded") }
        }
    }
}

fn source_cards_code(v: &Vals) -> String {
    Code::new("SourceCards")
        .raw(
            "sources",
            "vec![Source::new(\"RFC 9068\", \"https://www.rfc-editor.org/rfc/rfc9068\")]",
        )
        .s("status", v.s("status"), "done")
        .raw("max", v.s("max"))
        .raw("duration_ms", "Some(2100)")
        .done()
}

fn answer(v: &Vals, now: ClockMs) -> Element {
    let claims = vec![
        Claim::sourced(
            "Resource servers must check the exp claim and may allow a small leeway for clock skew.",
            &[1],
        ),
        Claim::sourced(
            "The jsonwebtoken crate exposes this as Validation::leeway, in seconds.",
            &[2],
        ),
        Claim::inferred("A 30 second leeway is enough to cover the skew seen in this test suite."),
        Claim::unverified("Other services in the repository use the same strict check."),
        Claim::sourced("The flaky failure was reported in issue #482.", &[3]),
    ];
    let ans = "Add a small **leeway** to the expiry check. RFC 9068 requires checking `exp` but allows for clock skew [1], and the jsonwebtoken crate exposes it as `Validation::leeway` [2]. The flaky test in #482 fails exactly at the minute boundary [3], so a 30 second leeway fixes it without weakening expiry in practice.";
    rsx! {
        div { class: "oxd-w560", style: "padding-bottom:40px",
            AiAnswer { question: "How should we fix the flaky token expiry test?", answer: ans, sources: sources()[..4].to_vec(), claims, variant: v.string("variant"), elapsed_ms: el(now, v), follow_ups: if v.b("follow_ups") { vec![s("Is 30 seconds too long for refresh tokens?"), s("Show me the regression test"), s("Which other services use a strict check?")] } else { vec![] } }
        }
    }
}

fn answer_code(v: &Vals) -> String {
    Code::new("AiAnswer")
        .raw(
            "question",
            q("How should we fix the flaky token expiry test?"),
        )
        .raw("answer", "answer_md")
        .raw("sources", "sources")
        .raw("claims", "claims")
        .s("variant", v.s("variant"), "answer")
        .raw("elapsed_ms", "now - started")
        .done()
}

// ──────────────────────────────── feedback / bubble ──

fn feedback(v: &Vals, _: ClockMs) -> Element {
    let rating = v.opt("rating");
    rsx! {
        div { class: "oxd-w520",
            div { class: "oxd-prose", style: "margin-bottom:10px", p { "Tokens minted in the last second of a minute expire before verify runs. Add a 30 s leeway and freeze the clock in a regression test." } }
            ResponseFeedback { rating, allow_comment: v.b("comment"), fail_send: v.b("fail"), positive: if v.b("positive") { oxyn_core::chat::thread::POSITIVE.iter().map(|x| x.to_string()).collect() } else { vec![] } }
        }
    }
}

fn feedback_code(v: &Vals) -> String {
    Code::new("ResponseFeedback")
        .opt("rating", v.s("rating"))
        .b("allow_comment", v.b("comment"))
        .raw("on_rating", "move |r: Option<String>| save(message_id, r)")
        .done()
}

fn bubble(v: &Vals, now: ClockMs) -> Element {
    let variant = v.string("variant");
    let streaming = v.b("streaming");
    rsx! {
        div { class: "oxd-w560", style: "display:flex;flex-direction:column;gap:16px",
            MessageBubble { role: "user", variant: variant.clone(), text: "Why does auth.rs fail at the minute boundary?", time: Some(s("14:02")), failed: v.b("failed"), files: if v.b("files") { vec![s("auth.rs"), s("ci-log.txt")] } else { vec![] } }
            MessageBubble { role: "assistant", variant: variant.clone(), text: "Tokens minted at **second 59** expire before `verify` runs. A 30 s leeway fixes it [1].", time: Some(s("14:02")), elapsed_ms: if streaming { Some(el(now, v) % 6000) } else { None }, edited: v.b("edited") }
            if v.b("system") { MessageBubble { role: "system", text: "Atlas switched to claude-sonnet · context compacted" } }
        }
    }
}

fn bubble_code(v: &Vals) -> String {
    Code::new("MessageBubble")
        .raw("role", q("assistant"))
        .raw("text", "reply")
        .s("variant", v.s("variant"), "bubble")
        .b("edited", v.b("edited"))
        .done()
}

// ──────────────────────────────────────── starters ──

fn starters() -> Vec<Starter> {
    vec![
        Starter::new(
            "w1",
            "write",
            "Draft release notes",
            "Draft release notes for v0.3 from the merged PRs this week.",
        )
        .app("GitHub"),
        Starter::new(
            "w2",
            "write",
            "Reply to the customer",
            "Draft a reply to Sam explaining the March overage and the new hard cap.",
        )
        .app("Mail"),
        Starter::new(
            "c1",
            "code",
            "Fix the flaky test",
            "Find out why auth_refresh fails at the minute boundary and fix it.",
        ),
        Starter::new(
            "c2",
            "code",
            "Bump dependencies",
            "Update all minor versions in Cargo.toml and run the test suite.",
        ),
        Starter::new(
            "c3",
            "code",
            "Explain this module",
            "Explain how src/auth works, with a diagram of the token flow.",
        ),
        Starter::new(
            "a1",
            "analyze",
            "Churn drivers",
            "What drove churn in Q3? Use the billing export and support tickets.",
        )
        .app("Drive"),
        Starter::new(
            "a2",
            "analyze",
            "Cost by agent",
            "Break down last month's spend by agent and model.",
        ),
        Starter::new(
            "p1",
            "plan",
            "Plan the migration",
            "Plan the move from Postgres 14 to 16 with zero downtime.",
        ),
        Starter::new(
            "p2",
            "plan",
            "Sprint plan",
            "Turn the open issues labelled p1 into a two-week sprint plan.",
        )
        .app("Linear"),
    ]
}

fn starters_demo(v: &Vals, _: ClockMs) -> Element {
    rsx! { div { class: "oxd-w560", StarterPrompts { greeting: "What are we working on, Ismail?", description: Some(s("Pick a starter or ask anything. Atlas can read the repo, run commands and open PRs.")), prompts: starters(), count: v.u("count"), loading: v.b("loading"), composer: v.b("composer") } } }
}

fn starters_code(v: &Vals) -> String {
    Code::new("StarterPrompts")
        .raw("greeting", q("What are we working on?"))
        .raw(
            "prompts",
            "vec![Starter::new(\"c1\", \"code\", \"Fix the flaky test\", \"…\")]",
        )
        .raw("count", v.s("count"))
        .b("loading", v.b("loading"))
        .raw("on_submit", "move |text: String| send(text)")
        .done()
}

// ──────────────────────────────────────── thread ──

fn thread_demo(v: &Vals, _: ClockMs) -> Element {
    let day = 86_400_000u64;
    let t0 = 20 * day + 9 * 3_600_000;
    let parts = vec![
        (s("me"), s("Ismail"), String::new()),
        (s("sam"), s("Sam"), s("blue")),
        (s("atlas"), s("Atlas"), s("violet")),
    ];
    let mut msgs = vec![
        ChatMessage::new("1", "sam", "Did the deploy go out?", t0 - day),
        ChatMessage::new(
            "2",
            "me",
            "Yes, 2.3.1 is live on staging.",
            t0 - day + 60_000,
        ),
        ChatMessage::new("3", "sam", "Auth tests are flaky again 😬", t0),
        ChatMessage::new("4", "sam", "Third time this week.", t0 + 40_000).react("😬", 2, false),
        ChatMessage::new(
            "5",
            "atlas",
            "I traced it to the expiry check at second 59. Fix is two lines; want me to open a PR?",
            t0 + 120_000,
        )
        .react("👍", 3, true)
        .file("auth-fix.diff"),
        ChatMessage::new(
            "6",
            "me",
            "Yes please, and add a regression test.",
            t0 + 180_000,
        )
        .status(if v.b("failed") {
            Delivery::Failed
        } else {
            Delivery::Read
        }),
    ];
    if !v.b("reactions") {
        for m in &mut msgs {
            m.reactions.clear();
        }
    }
    rsx! {
        ChatThread { participants: parts, me: "me", messages: msgs, typing: if v.b("typing") { vec![s("atlas")] } else { vec![] }, read_by: if v.b("receipts") { vec![(s("sam"), s("6")), (s("atlas"), s("6"))] } else { vec![] }, composer: v.b("composer"), height: v.f("height") as f32 }
    }
}

fn thread_code(v: &Vals) -> String {
    Code::new("ChatThread")
        .raw("participants", "vec![(\"me\".into(), \"Ismail\".into(), String::new()), (\"atlas\".into(), \"Atlas\".into(), \"violet\".into())]")
        .raw("me", q("me"))
        .raw("messages", "messages")
        .b("composer", v.b("composer"))
        .raw("on_send", "move |text: String| send(text)")
        .done()
}

// ─────────────────────────────── extraction / research ──

fn extraction(v: &Vals, now: ClockMs) -> Element {
    let fields = vec![
        Field::new("Vendor", "Lumen Labs Ltd", 0.98, 1),
        Field::new("Invoice number", "INV-2026-114", 0.96, 1),
        Field::new("Issue date", "2026-11-02", 0.91, 1),
        Field::new("Due date", "2026-12-02", 0.74, 1),
        Field::new("Subtotal", "$1,120.00", 0.88, 2),
        Field::new("Tax", "$120.40", 0.63, 2),
        Field::new("Total", "$1,240.40", 0.94, 2),
    ];
    rsx! { DocumentExtraction { fields, elapsed_ms: if v.b("scan") { el(now, v) % 6000 } else { u64::MAX }, threshold: v.f("threshold") as f32 } }
}

fn extraction_code(v: &Vals) -> String {
    Code::new("DocumentExtraction")
        .raw(
            "fields",
            "vec![Field::new(\"Total\", \"$1,240.40\", 0.94, 2)]",
        )
        .raw("threshold", v.s("threshold"))
        .done()
}

fn research(v: &Vals, now: ClockMs) -> Element {
    let qs = vec![
        s("Q3 churn by plan"),
        s("support ticket themes Q3"),
        s("pricing change August impact"),
        s("competitor launches Q3"),
    ];
    let ss = vec![
        (s("Billing export · Q3"), s("drive.google.com")),
        (s("Support tickets tagged churn"), s("zendesk.com")),
        (s("August pricing post-mortem"), s("notion.so")),
        (s("Competitor launch tracker"), s("airtable.com")),
        (s("NPS survey results"), s("typeform.com")),
    ];
    let e = if v.b("finished") {
        u64::MAX
    } else {
        (now.0.saturating_sub(BASE.0)) % 16_000
    };
    rsx! { ResearchRun { question: "What drove churn in Q3?", queries: qs, sources: ss, elapsed_ms: e } }
}

fn research_code(_: &Vals) -> String {
    Code::new("ResearchRun")
        .raw("question", q("What drove churn in Q3?"))
        .raw("queries", "queries")
        .raw("sources", "sources")
        .raw("elapsed_ms", "now - started")
        .done()
}

fn support(v: &Vals, _: ClockMs) -> Element {
    rsx! { SupportConversation { phase: v.string("phase"), height: 460.0 } }
}

fn support_code(v: &Vals) -> String {
    Code::new("SupportConversation")
        .s("phase", v.s("phase"), "bot")
        .done()
}

fn rec(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        RecommendationCard { title: "Move nightly evals to a smaller model", reason: "The nightly eval suite runs on the frontier model, but 92% of its checks pass identically on the small model. Keep the frontier model for the 40 checks that differ.", confidence: v.f("confidence") as f32, impact: if v.b("impact") { Some(s("−$1,240 / month · same pass rate")) } else { None }, evidence: if v.b("evidence") { vec![s("Last 30 nights: 1,184 of 1,288 checks identical"), s("Small model is 11× cheaper per token")] } else { vec![] } }
    }
}

fn rec_code(v: &Vals) -> String {
    Code::new("RecommendationCard")
        .raw("title", q("Move nightly evals to a smaller model"))
        .raw("reason", "reason")
        .raw("confidence", v.s("confidence"))
        .done()
}

fn ctx(v: &Vals, _: ClockMs) -> Element {
    let cards = vec![
        ContextCard::new("file", "src/auth/token.rs", "182 lines", 2400),
        ContextCard::new("file", "tests/auth_refresh.rs", "64 lines", 900),
        ContextCard::new("doc", "Auth design doc", "Notion · 6 pages", 7800),
        ContextCard::new("web", "RFC 9068", "rfc-editor.org", 5200),
        ContextCard::new("memory", "Prefers small PRs", "Saved Oct 12", 40),
        ContextCard::new("tool", "cargo test output", "148 passed · 1 failed", 3100),
    ];
    rsx! { ContextCards { cards, layout: v.string("layout"), budget: v.u("budget") as u32 } }
}

fn ctx_code(v: &Vals) -> String {
    Code::new("ContextCards")
        .raw(
            "cards",
            "vec![ContextCard::new(\"file\", \"src/auth/token.rs\", \"182 lines\", 2400)]",
        )
        .s("layout", v.s("layout"), "grid")
        .done()
}

fn aichat(v: &Vals, now: ClockMs) -> Element {
    let e = if v.b("finished") {
        u64::MAX
    } else {
        (now.0.saturating_sub(BASE.0)) % 16_000
    };
    rsx! {
        AiChat { elapsed_ms: e, reasoning: v.b("reasoning"), sources: v.b("sources"), height: 560.0,
            div { class: "oxc-ai-comp", textarea { rows: "1", placeholder: "Reply to Atlas…", "aria-label": "Message" } button { class: "stsend", r#type: "button", "aria-label": "Send", disabled: true, "↑" } }
        }
    }
}

fn aichat_code(v: &Vals) -> String {
    Code::new("AiChat")
        .raw("elapsed_ms", "now - started")
        .b("reasoning", v.b("reasoning"))
        .raw("children", "rsx! { Composer { .. } }")
        .done()
        .replace("children: rsx! { Composer { .. } },", "Composer { .. }")
}

fn finished() -> crate::Control {
    tog("finished", "Show finished", false)
}

/// Chunk-2 demos.
pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel(
                    "pace",
                    "Pace",
                    &["calm", "normal", "fast", "instant"],
                    "normal",
                ),
                sel(
                    "phase",
                    "Phase",
                    &["streaming", "stopped", "error"],
                    "streaming",
                ),
                rng("first_token", "First token ms", 0.0, 3000.0, 100.0, "600"),
                tog("caret", "Caret", true),
                sel("reveal", "Reveal", &["fade", "none"], "fade"),
                finished(),
            ],
            variants: vec![
                var(
                    "streaming",
                    "Headings, lists, code, a table and citations arriving.",
                    &[],
                ),
                var(
                    "stopped",
                    "Stopped mid-reply: what arrived stays.",
                    &[("phase", "stopped")],
                ),
                var(
                    "error",
                    "Connection dropped with Retry.",
                    &[("phase", "error")],
                ),
                var(
                    "instant",
                    "Everything at once (Oxyn plus).",
                    &[("pace", "instant")],
                ),
            ],
            render: smd,
            code: smd_code,
            keyboard: vec![("Tab", "Citation chips, links, code copy")],
            a11y: vec![
                "The container is aria-busy while streaming; the caret is aria-hidden.",
                "Partial markers (**, `, [1) are held back so screen readers never hear them.",
                "Citation chips are buttons named “Source n”.",
            ],
            responsive: vec!["Tables and code scroll horizontally inside their own frames."],
            height: 420,
            align: Align::Start,
            live: true,
            ..mk(
                "streaming-markdown",
                "Streaming markdown",
                "chat",
                "Markdown that renders correctly while it streams: held-back partial markers, open code fences that grow, tables that wait for whole rows, a caret, and stop or error states.",
                "StreamingMarkdown, Markdown",
                "MarkdownEl",
                "streaming-markdown",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "status",
                    "Status",
                    &["thinking", "done", "stopped", "error"],
                    "thinking",
                ),
                rng("seconds", "Seconds", 1.0, 120.0, 1.0, "7"),
                sel("variant", "Variant", &["card", "plain"], "card"),
                sel(
                    "level",
                    "Display level",
                    &["normal", "thinking", "verbose"],
                    "normal",
                ),
                tog("open", "Open", false),
                finished(),
            ],
            variants: vec![
                var(
                    "thinking",
                    "Shimmering label, seconds and the latest lines.",
                    &[],
                ),
                var("done", "Folded to “Thought for 7s”.", &[("status", "done")]),
                var(
                    "open",
                    "Open: steps parsed from headings.",
                    &[("status", "done"), ("open", "true")],
                ),
                var(
                    "error",
                    "Interrupted with Retry.",
                    &[("status", "error"), ("open", "true")],
                ),
                var(
                    "verbose",
                    "Verbose display level stays open (Oxyn plus).",
                    &[("status", "done"), ("level", "verbose")],
                ),
            ],
            render: reasoning,
            code: reasoning_code,
            keyboard: vec![
                ("Enter / Space", "Open or close the reasoning"),
                ("Tab", "Header, Stop / Retry, Copy reasoning"),
            ],
            a11y: vec![
                "The header is a button with aria-expanded and the visible label as its name.",
                "A polite region announces status changes only, never each chunk or second.",
                "The live preview is aria-hidden; the full region is aria-busy while streaming.",
            ],
            responsive: vec!["The header is 44 px in every state; actions never shift it."],
            height: 340,
            align: Align::Start,
            live: true,
            ..mk(
                "reasoning-block",
                "Reasoning block",
                "chat",
                "The thinking of a reply: a shimmering label counts seconds while the newest lines stream, then it folds to “Thought for 7s” and opens to the steps.",
                "ReasoningBlock",
                "ReasoningEl",
                "reasoning-block",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "kind",
                    "Kind",
                    &["steps", "reasoning", "search", "coding"],
                    "steps",
                ),
                rng("step", "Step ms", 300.0, 2000.0, 100.0, "900"),
                tog("fail", "A row fails", false),
                finished(),
            ],
            variants: vec![
                var("steps", "Plan steps playing in.", &[]),
                var(
                    "search",
                    "Search trace with sources.",
                    &[("kind", "search")],
                ),
                var("coding", "Files and commands.", &[("kind", "coding")]),
                var(
                    "failed",
                    "A failed row (Oxyn plus).",
                    &[("kind", "coding"), ("fail", "true"), ("finished", "true")],
                ),
            ],
            render: traces,
            code: traces_code,
            keyboard: vec![("Enter / Space", "Expand or collapse")],
            a11y: vec![
                "Each row's mark carries its status as a label.",
                "The header is a disclosure button with aria-expanded.",
            ],
            responsive: vec!["Details ellipsize; rows never wrap their mark."],
            height: 300,
            align: Align::Start,
            live: true,
            ..mk(
                "thinking-traces",
                "Thinking",
                "chat",
                "Beautiful UI's Thinking traces ported: steps, reasoning, search and coding rows playing in with pending, active, done and failed marks.",
                "ThinkingTraces",
                "TraceEl",
                "reasoning-block",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel("status", "Status", &["searching", "done", "error"], "done"),
                rng("count", "Sources", 0.0, 6.0, 1.0, "6"),
                rng("max", "Cards before stack", 2.0, 5.0, 1.0, "3"),
                tog("highlight", "Highlight [2]", false),
                tog("expanded", "Stack open", false),
            ],
            variants: vec![
                var("done", "Three cards and a favicon stack.", &[]),
                var(
                    "searching",
                    "Cards streaming into reserved slots.",
                    &[("status", "searching")],
                ),
                var("expanded", "The stack opened.", &[("expanded", "true")]),
                var("none", "No sources found.", &[("count", "0")]),
                var(
                    "error",
                    "Search failed with Retry.",
                    &[("status", "error"), ("count", "2")],
                ),
            ],
            render: source_cards,
            code: source_cards_code,
            keyboard: vec![
                ("Tab", "Cards and the stack"),
                ("Enter", "Open a card or the stack"),
            ],
            a11y: vec![
                "A labelled list, aria-busy while searching; skeletons are hidden.",
                "Each card reads number, title and domain, and says it opens a new tab.",
                "A polite region announces the status once.",
            ],
            responsive: vec!["max + 1 columns; two below 540 px of container."],
            height: 300,
            align: Align::Start,
            live: true,
            ..mk(
                "source-cards",
                "Source cards",
                "chat",
                "Cited sources under an answer: cards stream into reserved slots, the overflow becomes a favicon stack, hover previews the snippet and citations highlight their card.",
                "SourceCards, SourceCitation",
                "SourceCardsEl",
                "citations-sources",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["answer", "cited"], "answer"),
                tog("follow_ups", "Follow-ups", true),
                finished(),
            ],
            variants: vec![
                var(
                    "answer",
                    "Search, then a streamed answer with citations.",
                    &[],
                ),
                var(
                    "cited",
                    "Claims with provenance and a grounding meter.",
                    &[("variant", "cited")],
                ),
                var("done", "Finished with follow-ups.", &[("finished", "true")]),
            ],
            render: answer,
            code: answer_code,
            keyboard: vec![("Tab", "Citations, cards, actions and follow-ups")],
            a11y: vec![
                "Provenance is shown with words and icons, not colour alone.",
                "Citation chips name their source.",
            ],
            responsive: vec!["Up to 680 px; cards drop to two columns."],
            height: 520,
            align: Align::Start,
            live: true,
            ..mk(
                "ai-answer",
                "AI answer",
                "chat",
                "An answer with its sources: search, stream, citations that highlight cards, actions and follow-ups; the cited variant shows each claim's provenance.",
                "AiAnswer",
                "AnswerEl",
                "citations-sources",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel("rating", "Rating", &["none", "up", "down"], "none"),
                tog("comment", "Note field", true),
                tog("positive", "Ask what was good", true),
                tog("fail", "First send fails", false),
            ],
            variants: vec![
                var("idle", "Thumbs only.", &[]),
                var(
                    "down",
                    "Thumbs down with reasons and a note.",
                    &[("rating", "down")],
                ),
                var("up", "Thumbs up asks what was good.", &[("rating", "up")]),
                var(
                    "retry",
                    "A failed send becomes Try again (Oxyn plus).",
                    &[("rating", "down"), ("fail", "true")],
                ),
            ],
            render: feedback,
            code: feedback_code,
            keyboard: vec![
                ("Enter / Space", "Press a thumb or chip"),
                ("Escape", "Close the panel, keep the rating"),
                ("⌘ / Ctrl + Enter", "Submit from the note"),
            ],
            a11y: vec![
                "Thumbs are toggle buttons (Good / Bad response) with aria-expanded while the panel is open.",
                "Chips use aria-pressed and invert when selected.",
                "A polite region announces rating, sending, thanks and errors.",
            ],
            responsive: vec!["The privacy line hides below 400 px of container."],
            height: 300,
            align: Align::Start,
            ..mk(
                "response-feedback",
                "Response feedback",
                "chat",
                "Feedback on a reply: thumbs that fill, reason chips and a note after thumbs down, a short thank-you that folds away, and Try again when sending fails.",
                "ResponseFeedback, MessageActions",
                "FeedbackEl",
                "message-bubble",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["bubble", "flat"], "bubble"),
                tog("streaming", "Streaming", false),
                tog("failed", "Failed send", false),
                tog("files", "Attachments", false),
                tog("edited", "Edited", false),
                tog("system", "System note", true),
            ],
            variants: vec![
                var("bubble", "User bubble, assistant prose with actions.", &[]),
                var(
                    "flat",
                    "Flat transcript (Oxyn plus).",
                    &[("variant", "flat")],
                ),
                var(
                    "streaming",
                    "The assistant still streaming.",
                    &[("streaming", "true")],
                ),
                var(
                    "failed",
                    "A failed send with Retry.",
                    &[("failed", "true"), ("files", "true")],
                ),
            ],
            render: bubble,
            code: bubble_code,
            keyboard: vec![("Tab", "Actions appear on focus within")],
            a11y: vec![
                "Each turn is an article named by its author.",
                "Actions are always reachable by keyboard; on touch they stay visible.",
            ],
            responsive: vec!["User bubbles cap at 80 % width."],
            height: 340,
            align: Align::Start,
            live: true,
            ..mk(
                "message-bubble",
                "Message bubble",
                "chat",
                "One turn of a conversation: user bubbles, assistant prose with a model mark, copy, regenerate and thumbs, system notes, failed sends and attachments.",
                "MessageBubble",
                "MessageEl",
                "message-bubble",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                rng("count", "Cards", 2.0, 6.0, 2.0, "4"),
                tog("loading", "Loading", false),
                tog("composer", "Composer", true),
            ],
            variants: vec![
                var("starters", "Greeting, tabs, cards and a composer.", &[]),
                var("loading", "Placeholder cards.", &[("loading", "true")]),
                var(
                    "six",
                    "Six cards without the composer.",
                    &[("count", "6"), ("composer", "false")],
                ),
            ],
            render: starters_demo,
            code: starters_code,
            keyboard: vec![
                ("Tab", "Tabs, Shuffle, cards, composer"),
                ("Enter / Space", "Fill the composer from a card"),
                ("Enter", "Send"),
                ("Shift + Enter", "New line"),
            ],
            a11y: vec![
                "A labelled section with the greeting as its heading.",
                "Card names include title, app and category.",
                "Picks and shuffles are announced politely.",
            ],
            responsive: vec!["Two columns; one below 480 px of container."],
            height: 520,
            align: Align::Start,
            ..mk(
                "starter-prompts",
                "Starter prompts",
                "chat",
                "The empty state of an AI app: a greeting, starter cards by category with a staggered shuffle, and a composer the cards fill.",
                "StarterPrompts",
                "StartersEl",
                "messages",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("reactions", "Reactions", true),
                tog("receipts", "Read receipts", true),
                tog("typing", "Typing", true),
                tog("failed", "Failed send", false),
                tog("composer", "Composer", true),
                rng("height", "Height", 320.0, 560.0, 20.0, "460"),
            ],
            variants: vec![
                var("team", "A team thread with an agent.", &[]),
                var("failed", "Your last message failed.", &[("failed", "true")]),
                var(
                    "transcript",
                    "Read-only transcript.",
                    &[("composer", "false"), ("typing", "false")],
                ),
            ],
            render: thread_demo,
            code: thread_code,
            keyboard: vec![
                ("Enter", "Send"),
                ("Shift + Enter", "New line"),
                ("Tab", "Reaction buttons show on focus"),
                ("Escape", "Close the reaction picker"),
            ],
            a11y: vec![
                "role=log, polite, additions; typing is a separate status.",
                "Reaction chips are toggles labelled “👍 3, including you”.",
                "Read receipts carry a label listing readers.",
            ],
            responsive: vec!["Messages cap at 78 % width."],
            height: 500,
            align: Align::Start,
            ..mk(
                "chat-thread",
                "Chat thread",
                "chat",
                "A conversation with grouped messages, day separators, reactions, read receipts, typing and a composer; agents join with their identity hue.",
                "ChatThread",
                "ChatThreadEl",
                "messages",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("reasoning", "Reasoning", true),
                tog("sources", "Sources", true),
                finished(),
            ],
            variants: vec![
                var("full", "Reasoning, sources and a streamed answer.", &[]),
                var(
                    "plain",
                    "Just the answer.",
                    &[("reasoning", "false"), ("sources", "false")],
                ),
                var("done", "Finished.", &[("finished", "true")]),
            ],
            render: aichat,
            code: aichat_code,
            keyboard: vec![("Tab", "Through the transcript to the composer")],
            a11y: vec!["The transcript is a polite log; each turn is an article."],
            responsive: vec!["Fills its container up to 760 px."],
            height: 600,
            align: Align::Start,
            live: true,
            ..mk(
                "ai-chat",
                "AI chat",
                "chat",
                "A full AI chat block: a user turn, the assistant thinking, searching and streaming a cited answer, then actions — Arc UI's AI chat and Beautiful UI's Chat.",
                "AiChat",
                "",
                "messages",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("scan", "Animate the scan", true),
                rng("threshold", "Review below", 0.5, 0.95, 0.05, "0.8"),
            ],
            variants: vec![
                var("scan", "Fields appear as the scan reaches them.", &[]),
                var(
                    "review",
                    "Finished, two fields to review.",
                    &[("scan", "false")],
                ),
                var(
                    "strict",
                    "A stricter threshold (Oxyn plus).",
                    &[("scan", "false"), ("threshold", "0.95")],
                ),
            ],
            render: extraction,
            code: extraction_code,
            keyboard: vec![("Tab", "Fields, Looks right, Confirm all")],
            a11y: vec![
                "Confidence has a label in percent, not just a bar.",
                "Fields are labelled inputs with their page.",
            ],
            responsive: vec!["Document and fields stack below 620 px of container."],
            height: 440,
            align: Align::Start,
            live: true,
            ..mk(
                "document-extraction",
                "Document extraction",
                "chat",
                "Fields pulled from a document with confidence and page provenance: hover a field to see where it came from, review the low-confidence ones, then confirm.",
                "DocumentExtraction",
                "",
                "citations-sources",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![finished()],
            variants: vec![
                var("running", "Planning, searching, reading, writing.", &[]),
                var("done", "The finished report card.", &[("finished", "true")]),
            ],
            render: research,
            code: research_code,
            keyboard: vec![("Tab", "Pause, Stop, Open report")],
            a11y: vec!["A progressbar with a value; the current phase has aria-current=step."],
            responsive: vec!["Columns stack below 520 px of container."],
            height: 420,
            align: Align::Start,
            live: true,
            ..mk(
                "research-run",
                "Research run",
                "chat",
                "A deep-research job: phases, the queries it ran, sources rolling in, progress and elapsed time, ending on a report.",
                "ResearchRun",
                "",
                "citations-sources",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![sel(
                "phase",
                "Start at",
                &["bot", "handoff", "human", "resolved"],
                "bot",
            )],
            variants: vec![
                var("bot", "The assistant answers.", &[]),
                var(
                    "handoff",
                    "Waiting in line for a person.",
                    &[("phase", "handoff")],
                ),
                var(
                    "resolved",
                    "Resolved, asking for a rating.",
                    &[("phase", "resolved")],
                ),
            ],
            render: support,
            code: support_code,
            keyboard: vec![("Tab", "Messages and the footer actions")],
            a11y: vec!["The transcript is a polite log; the banner states who is answering."],
            responsive: vec!["Up to 560 px."],
            height: 500,
            align: Align::Start,
            ..mk(
                "support-conversation",
                "Support conversation",
                "chat",
                "A support chat that starts with an AI assistant, hands off to a person with a queue position, and ends with a rating.",
                "SupportConversation",
                "",
                "messages",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                rng("confidence", "Confidence", 0.2, 1.0, 0.02, "0.86"),
                tog("impact", "Impact", true),
                tog("evidence", "Evidence", true),
            ],
            variants: vec![
                var("high", "High confidence.", &[]),
                var("medium", "Medium confidence.", &[("confidence", "0.6")]),
                var(
                    "bare",
                    "Without impact or evidence.",
                    &[("impact", "false"), ("evidence", "false")],
                ),
            ],
            render: rec,
            code: rec_code,
            keyboard: vec![("Tab", "Accept, Dismiss, Undo")],
            a11y: vec!["Confidence is text; the decision is announced as status."],
            responsive: vec!["Up to 440 px."],
            height: 340,
            ..mk(
                "recommendation-card",
                "Recommendation card",
                "chat",
                "Beautiful UI's Recommendation Card ported: a suggestion with its reason, confidence, impact and evidence, accepted or dismissed with Undo.",
                "RecommendationCard",
                "",
                "citations-sources",
                "chat/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel("layout", "Layout", &["grid", "row"], "grid"),
                rng(
                    "budget",
                    "Budget tokens",
                    20000.0,
                    200000.0,
                    10000.0,
                    "40000",
                ),
            ],
            variants: vec![
                var("grid", "Files, docs, web, memory and tools.", &[]),
                var("row", "One scrolling row.", &[("layout", "row")]),
            ],
            render: ctx,
            code: ctx_code,
            keyboard: vec![("Tab", "Pin and remove per card")],
            a11y: vec!["Pin is a toggle (aria-pressed); remove names its card."],
            responsive: vec!["Columns of at least 210 px; one column in narrow frames."],
            height: 300,
            align: Align::Start,
            ..mk(
                "context-cards",
                "Context cards",
                "chat",
                "Beautiful UI's Context Cards ported: what the agent used, with kind icons and token cost; pin to keep, remove to drop.",
                "ContextCards",
                "",
                "citations-sources",
                "chat/mod.rs",
            )
        },
    ]
}
