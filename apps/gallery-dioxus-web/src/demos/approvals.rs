//! Approvals: approval card and pager.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::primitives::questions::{Question, QuestionKind, QuestionOption};
use oxyn_core::{ClockMs, approval::Decision};
use oxyn_dioxus::ApprovalCard;
#[cfg(oxyn_pro)]
use oxyn_dioxus::{ApprovalPager, ApprovalQuestions, DecisionCard};

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

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![Demo {
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
    }];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 4] = [
    "approval-card",
    "approval-pager",
    "decision-card",
    "approval-questions",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
