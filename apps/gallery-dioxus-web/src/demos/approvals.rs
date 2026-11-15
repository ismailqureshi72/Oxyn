//! Approvals: approval card and pager.

use dioxus::prelude::*;
use oxyn_core::primitives::questions::{Question, QuestionKind, QuestionOption};
use oxyn_core::{ClockMs, approval::Decision};
use oxyn_dioxus::{ApprovalCard, ApprovalPager, ApprovalQuestions, DecisionCard};

use super::{Code, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

fn card(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        LiveApproval {
            risk: v.string("risk"),
            countdown: v.b("countdown"),
            phase: v.string("phase"),
            holding: v.b("holding"),
            variant: v.string("variant"),
            actions: v.string("actions"),
            shortcuts: v.b("shortcuts"),
            always: v.b("always"),
            edit: v.b("edit"),
            labels: v.string("labels"),
            now,
        }
    }
}

/// The card as a host would drive it: decisions press the model, the host acknowledges 700 ms
/// later, holds start and end with the pointer.
#[component]
fn LiveApproval(
    risk: String,
    countdown: bool,
    phase: String,
    holding: bool,
    variant: String,
    actions: String,
    shortcuts: bool,
    always: bool,
    edit: bool,
    labels: String,
    now: ClockMs,
) -> Element {
    let start = now;
    let init_phase = phase.clone();
    let init_risk = risk.clone();
    let mut card = use_signal(move || {
        let mut c = data::approval(&init_risk, countdown);
        match init_phase.as_str() {
            "sending" => {
                let _ = c.press(Decision::AllowOnce, ClockMs(start.0.saturating_sub(600)));
            }
            "approved" => {
                let _ = c.press(Decision::AllowOnce, ClockMs(start.0.saturating_sub(900)));
                c.ack(ClockMs(start.0.saturating_sub(800)));
            }
            "denied" => {
                let _ = c.press(
                    Decision::Deny {
                        reason: Some("use --dry-run first".into()),
                    },
                    ClockMs(start.0.saturating_sub(900)),
                );
                c.ack(ClockMs(start.0.saturating_sub(800)));
            }
            _ => {
                if init_risk == "critical" && holding {
                    let _ = c.hold_start(ClockMs(start.0.saturating_sub(780)));
                }
            }
        }
        c
    });
    let mut pressed = use_signal(|| None::<u64>);
    let mut shown = card.read().clone();
    if let Some(p) = pressed()
        && now.0 >= p + 700
    {
        shown.ack(ClockMs(p + 700));
    }
    shown.tick(now);
    let room = match variant.as_str() {
        "terminal" => "oxd-term oxd-w560",
        "modal" => "oxd-modal-room",
        _ => "oxd-w520",
    };
    rsx! {
        div { class: room,
            ApprovalCard {
                card: shown,
                now,
                variant,
                actions,
                shortcuts,
                always,
                on_edit: if edit { Some(EventHandler::new(|_| {})) } else { None },
                on_hover: move |h: bool| card.write().set_hover(h, now),
                approve_label: (labels == "custom").then(|| "Send now".to_string()),
                deny_label: if labels == "custom" { "Not yet".to_string() } else { "Deny".to_string() },
                always_label: if labels == "custom" { "Always allow sends to this list".to_string() } else { "Always allow this action".to_string() },
                icon: (labels == "custom").then(|| "mail".to_string()),
                on_decide: move |d: Decision| {
                    if card.write().press(d, now).is_ok() {
                        pressed.set(Some(now.0));
                    }
                },
                on_hold: move |down: bool| {
                    if down {
                        let _ = card.write().hold_start(now);
                    } else {
                        card.write().hold_end(now);
                    }
                },
            }
        }
    }
}

fn card_code(v: &Vals) -> String {
    let risk = match v.s("risk") {
        "low" => "Risk::Low",
        "high" => "Risk::High",
        "critical" => "Risk::Critical",
        _ => "Risk::Medium",
    };
    let mut c = Code::new("ApprovalCard")
        .pre(format!(
            "let mut req = ApprovalRequest::new(\"e\", \"Send campaign email\", {risk}, opened_at);"
        ))
        .pre("req.details = vec![(\"To\".into(), \"Customers – EU (42)\".into())];");
    if v.b("countdown") {
        c = c.pre("req.expires_in = Some(Duration::from_secs(60)); // pauses while hovered");
    }
    c.pre("let card = use_signal(|| ApprovalCard::new(req));")
        .raw("card", "card()")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "inline")
        .s("actions", v.s("actions"), "tray")
        .raw("shortcuts", v.b("shortcuts").to_string())
        .raw(
            "on_decide",
            "move |d: Decision| agent.answer(d) // AllowOnce, AllowAlways, Deny…",
        )
        .done()
        .replace("        shortcuts: true,\n", "")
}

fn pager(v: &Vals, now: ClockMs) -> Element {
    rsx! { div { class: "oxd-w520", ApprovalPager { queue: data::queue(), now, page: v.u("page") } } }
}

fn pager_code(v: &Vals) -> String {
    Code::new("ApprovalPager")
        .raw("queue", "approvals() // ApprovalQueue")
        .raw("now", "clock.now()")
        .raw("page", v.s("page"))
        .done()
}

fn decisions(kind: &str) -> Vec<Question> {
    let db = Question::new(
        "cache",
        "Where should the dashboard cache live?",
        QuestionKind::Single,
        vec![
            QuestionOption::new("redis", "Redis").summary("Fastest; one more service to run"),
            QuestionOption::new("pg", "Postgres table")
                .summary("Uses what we have; ~40 ms slower")
                .recommended(),
            QuestionOption::new("edge", "Edge KV").summary("Global, eventually consistent"),
            QuestionOption::new("none", "No cache").summary("Ship the query fix only"),
        ],
    )
    .task("Speed up the usage dashboard");
    let pick = Question::new(
        "hero",
        "Which hero image for the launch post?",
        QuestionKind::Pick,
        vec![
            QuestionOption::new("a", "Moth over code").summary("Dark, on brand"),
            QuestionOption::new("b", "Team at the board")
                .recommended()
                .summary("Warm, human"),
            QuestionOption::new("c", "Product screenshot").summary("Plain, clear"),
        ],
    )
    .task("Draft the launch post");
    let appr = Question::new(
        "deploy",
        "Deploy the billing fix to production?",
        QuestionKind::Approval,
        vec![],
    )
    .facts(&[
        ("Branch", "fix/proration"),
        ("Changes", "+214 −38 in 6 files"),
        ("Checks", "All 128 passed"),
    ])
    .task("Fix proration on upgrades");
    match kind {
        "pick" => vec![pick, db, appr],
        "approval" => vec![appr, db],
        _ => vec![db, pick, appr],
    }
}

fn decision(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w560",
            DecisionCard {
                decisions: decisions(v.s("kind")),
                default_folded: v.b("folded"),
                timeout_s: v.b("timeout").then_some(15),
                always: v.b("always"),
            }
        }
    }
}

fn decision_code(v: &Vals) -> String {
    Code::new("DecisionCard")
        .pre("let decisions = vec![")
        .pre("    Question::new(\"cache\", \"Where should the cache live?\", QuestionKind::Single, vec![")
        .pre("        QuestionOption::new(\"redis\", \"Redis\"),")
        .pre("        QuestionOption::new(\"pg\", \"Postgres table\").recommended(),")
        .pre("    ]).task(\"Speed up the dashboard\"),")
        .pre("];")
        .raw("decisions", "decisions")
        .b("default_folded", v.b("folded"))
        .raw("timeout_s", if v.b("timeout") { "Some(15)" } else { "None" })
        .b("always", v.b("always"))
        .raw("on_decide", "move |(id, answer)| agent.answer(id, answer)")
        .raw("on_later", "move |_| composer.focus()")
        .done()
}

fn flavours() -> Vec<Question> {
    vec![
        Question::new(
            "count",
            "How many flavors should we launch?",
            QuestionKind::Single,
            vec![
                QuestionOption::new("3", "Three (core line)").recommended(),
                QuestionOption::new("5", "Five (full case)"),
                QuestionOption::new("1", "Just one hero"),
            ],
        ),
        Question::new(
            "mix",
            "Which mix-ins should we stock?",
            QuestionKind::Multi,
            vec![
                QuestionOption::new("choc", "Chocolate chips"),
                QuestionOption::new("waffle", "Waffle bits"),
                QuestionOption::new("sprinkles", "Sprinkles"),
            ],
        ),
        Question::new(
            "market",
            "Which market do we enter first?",
            QuestionKind::Single,
            vec![
                QuestionOption::new("trucks", "Food trucks"),
                QuestionOption::new("grocery", "Grocery freezers"),
                QuestionOption::new("shops", "Scoop shops"),
            ],
        ),
    ]
}

fn questions(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w480",
            ApprovalQuestions {
                questions: flavours(),
                review: v.b("review"),
                resettable: v.b("resettable"),
                timeout_s: v.b("timeout").then_some(20),
            }
        }
    }
}

fn questions_code(v: &Vals) -> String {
    Code::new("ApprovalQuestions")
        .pre("let questions = vec![")
        .pre("    Question::new(\"count\", \"How many flavors should we launch?\", QuestionKind::Single, vec![")
        .pre("        QuestionOption::new(\"3\", \"Three (core line)\"),")
        .pre("        QuestionOption::new(\"5\", \"Five (full case)\"),")
        .pre("    ]),")
        .pre("];")
        .raw("questions", "questions")
        .b("review", v.b("review"))
        .raw("timeout_s", if v.b("timeout") { "Some(20)" } else { "None" })
        .raw("on_submitted", "move |answers: Vec<String>| agent.resume(answers)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "approval-card",
            name: "Approval card",
            category: "approvals",
            summary: "An agent asks before it acts: a risk tier, what exactly will happen, Allow and Deny with shortcuts, a countdown that pauses while you read, arming for high risk, hold-to-allow for critical, and a resolved row.",
            dioxus: "ApprovalCard",
            gpui: "ApprovalCardEl",
            spec: "approval-card",
            source: "approval.rs",
            controls: vec![
                sel(
                    "risk",
                    "Risk",
                    &["low", "medium", "high", "critical"],
                    "medium",
                ),
                sel(
                    "variant",
                    "Variant",
                    &["inline", "footer", "terminal", "modal"],
                    "inline",
                ),
                sel(
                    "phase",
                    "Phase",
                    &["pending", "sending", "approved", "denied"],
                    "pending",
                ),
                sel("actions", "Buttons", &["tray", "card"], "tray"),
                tog("countdown", "Countdown (60 s)", true),
                tog("edit", "Edit button", false),
                sel("labels", "Labels", &["default", "custom"], "default"),
                tog("shortcuts", "Key hints", true),
                tog("always", "Always allow ticked", false),
                tog("holding", "Holding (critical)", true),
            ],
            variants: vec![
                var("inline", "Medium risk with a countdown.", &[]),
                var(
                    "edit-custom",
                    "Edit before Deny, custom labels and icon.",
                    &[("edit", "true"), ("labels", "custom")],
                ),
                var(
                    "card-actions",
                    "Buttons inside the card, Edit shown.",
                    &[("actions", "card"), ("edit", "true")],
                ),
                var(
                    "high-arming",
                    "High risk: Allow arms first, Deny is the default.",
                    &[("risk", "high"), ("countdown", "false")],
                ),
                var(
                    "critical-hold",
                    "Hold to allow.",
                    &[("risk", "critical"), ("countdown", "false")],
                ),
                var(
                    "terminal",
                    "Terminal prompt.",
                    &[
                        ("risk", "low"),
                        ("variant", "terminal"),
                        ("countdown", "false"),
                    ],
                ),
                var(
                    "resolved",
                    "Folded into a resolved row.",
                    &[("phase", "approved"), ("risk", "low")],
                ),
            ],
            render: card,
            code: card_code,
            keyboard: vec![
                (
                    "⌘ / Ctrl + Enter",
                    "Allow (declared with aria-keyshortcuts; the host binds it)",
                ),
                ("Esc", "Deny (declared with aria-keyshortcuts)"),
                ("Tab", "Through Deny, For this session and Allow"),
                ("Terminal", "y allows, p allows always, Esc says no"),
            ],
            a11y: vec![
                "Deny and Allow are real buttons that declare aria-keyshortcuts (Escape, Meta+Enter); the visible key hints are kbd elements.",
                "The countdown text is aria-hidden; it turns amber in the last 10 seconds.",
                "Risk is carried by shape and label, never colour alone.",
            ],
            responsive: vec![
                "inline in a transcript, footer inside a tool-call card, terminal in a TUI, modal for critical actions.",
                "actions card keeps the buttons inside the card for narrow side panels.",
            ],
            height: 360,
            align: Align::Start,
            live: true,
        },
        Demo {
            slug: "approval-pager",
            name: "Approval pager",
            category: "approvals",
            summary: "Several open requests in one card: page through them with ‹ › and allow the whole batch once.",
            dioxus: "ApprovalPager",
            gpui: "",
            spec: "approvals",
            source: "approval.rs",
            controls: vec![rng("page", "Page", 0.0, 2.0, 1.0, "0")],
            variants: vec![
                var("first", "Request 1 of 3.", &[]),
                var("last", "Request 3 of 3.", &[("page", "2")]),
            ],
            render: pager,
            code: pager_code,
            keyboard: vec![("Tab", "Through the pager and the decision buttons")],
            a11y: vec![
                "The pager text (\"1 of 3\") and the batch row are real text next to the card's buttons.",
            ],
            responsive: vec!["Fills its container; the batch row stays under the card."],
            height: 300,
            align: Align::Start,
            live: true,
        },
        Demo {
            slug: "decision-card",
            name: "Decision card",
            category: "approvals",
            summary: "An agent's question in its own card above the composer: numbered options with one Recommended, tiles for a pick, Approve / Decline with facts, or your own words.",
            dioxus: "DecisionCard",
            gpui: "DecisionCardEl",
            spec: "decision-card",
            source: "primitives/flows.rs",
            controls: vec![
                sel(
                    "kind",
                    "First decision",
                    &["decision", "pick", "approval"],
                    "decision",
                ),
                tog("folded", "Folded", false),
                tog("timeout", "Timeout (15 s)", false),
                tog("always", "Always answer this way", false),
            ],
            variants: vec![
                var("decision", "Numbered rows, one Recommended.", &[]),
                var("pick", "Three tiles.", &[("kind", "pick")]),
                var(
                    "approval",
                    "Approve or Decline with facts.",
                    &[("kind", "approval")],
                ),
                var("folded", "Folded to one line.", &[("folded", "true")]),
                var(
                    "timeout",
                    "Picks Recommended after 15 s (Oxyn plus).",
                    &[("timeout", "true"), ("always", "true")],
                ),
            ],
            render: decision,
            code: decision_code,
            keyboard: vec![
                (
                    "1 – 5",
                    "Answer with that row at once; the next number goes into the field",
                ),
                ("Home / End", "First / last row"),
                (
                    "↑ / ↓ (← → on tiles)",
                    "Move between rows; ↓ from the last row goes into the field",
                ),
                (
                    "Enter / Space",
                    "Answer with the focused row; in the field Enter sends your words",
                ),
                ("Esc", "Later: fold the card to one line"),
            ],
            a11y: vec![
                "The card is a group named by the question; rows are a listbox with one tab stop.",
                "Each row says its number key (aria-keyshortcuts); Later says Escape.",
                "Answers and Later are announced politely with how many still wait.",
            ],
            responsive: vec![
                "Takes the composer's width; long titles wrap to two lines then end in ….",
                "Below 420 px tiles turn into rows; touch rows are 44 px.",
            ],
            height: 380,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "approval-questions",
            name: "Approval questions",
            category: "approvals",
            summary: "Paged questions the agent asks before acting: an odometer counter, single choice that moves on by itself, multi-select, your own answer, Skip and Send.",
            dioxus: "ApprovalQuestions",
            gpui: "ApprovalQuestionsEl",
            spec: "approval-questions",
            source: "primitives/flows.rs",
            controls: vec![
                tog("review", "Review before send", false),
                tog("resettable", "Start over after send", true),
                tog("timeout", "Timeout (20 s)", false),
            ],
            variants: vec![
                var("paged", "Three questions, 1 / 3.", &[]),
                var(
                    "review",
                    "A review page before Send (Oxyn plus).",
                    &[("review", "true")],
                ),
                var(
                    "timeout",
                    "Answers Recommended after 20 s (Oxyn plus).",
                    &[("timeout", "true")],
                ),
            ],
            render: questions,
            code: questions_code,
            keyboard: vec![
                ("Enter", "Continue (Send on the last question)"),
                (
                    "Click / Space on an option",
                    "Pick; single choice moves on after 480 ms",
                ),
                ("Esc", "Dismiss"),
            ],
            a11y: vec![
                "Single choice is a radiogroup, multi-select a group of checkboxes; the counter is named “Question 1 of 3”.",
                "The card is aria-live=polite so each new question is read.",
                "Prev, Next and Dismiss are labelled icon buttons.",
            ],
            responsive: vec!["Fills its container up to 460 px; option rows wrap their text."],
            height: 340,
            align: Align::Center,
            live: false,
        },
    ]
}
