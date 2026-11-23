//! Chat (plan #27, chunk 2): streaming markdown, reasoning, thinking traces, source cards, AI and
//! cited answers, response feedback, message bubble, starter prompts, chat thread, AI chat,
//! document extraction, research run, support conversation, recommendation and context cards.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    chat::{
        sources::{Claim, ContextCard, Field, Source},
        thread::{ChatMessage, Delivery, Starter},
    },
};
use oxyn_dioxus::{ChatThread, MessageBubble, ReasoningBlock, ResponseFeedback, StreamingMarkdown};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{AiAnswer, AiChat, ContextCards, DocumentExtraction, RecommendationCard, ResearchRun, SourceCards, StarterPrompts, SupportConversation, ThinkingTraces};

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

// ──────────────────────────────────────── thread ──

fn thread_demo(v: &Vals, _: ClockMs) -> Element {
    let day = 86_400_000u64;
    let t0 = 20 * day + 9 * 3_600_000;
    let parts = vec![
        (s("me"), s("Alex Rivera"), String::new()),
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
        .raw("participants", "vec![(\"me\".into(), \"Alex Rivera\".into(), String::new()), (\"atlas\".into(), \"Atlas\".into(), \"violet\".into())]")
        .raw("me", q("me"))
        .raw("messages", "messages")
        .b("composer", v.b("composer"))
        .raw("on_send", "move |text: String| send(text)")
        .done()
}

fn finished() -> crate::Control {
    tog("finished", "Show finished", false)
}

/// Chunk-2 demos.
pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
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
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 15] = ["streaming-markdown", "reasoning-block", "thinking-traces", "source-cards", "ai-answer-oxyn", "response-feedback", "message-bubble", "starter-prompts", "chat-thread", "ai-chat", "document-extraction", "research-run", "support-conversation", "recommendation-card", "context-cards"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
