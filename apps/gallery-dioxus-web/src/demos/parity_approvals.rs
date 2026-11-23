//! Plan #28 parity demos, approvals: hold to confirm, inline confirm, confirm popover, approval
//! gates, permission request and the agent approvals block.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
        confirm::{
            ConfirmMode, ConfirmPopover as PopModel, ConfirmTone, HoldConfirm,
            InlineConfirm as MorphModel, MorphFace, PopoverAlign, PopoverPhase,
        },
        gates::{
            AgentApprovals as AgentModel, Arg, Gate, GateDecision, GateKind, GateStatus, Gates,
            PermRisk, PermissionRequest as PermModel, PreviewRow, Rule, RuleScope, Scope, ToolAsk,
        },
    },
};
use oxyn_dioxus::ConfirmPopover;
#[cfg(oxyn_pro)]
use oxyn_dioxus::{AgentApprovals, ApprovalGates, HoldToConfirm, InlineConfirm, PermissionRequest};

use super::{Code, mk, q, sel, tog, var};
use crate::{Align, Demo, Vals};

// ---------- confirm popover ----------

fn pop(v: &Vals, now: ClockMs) -> Element {
    rsx! { LivePop { tone: v.string("tone"), phrase: v.b("phrase"), align: v.string("align"), result: v.string("result"), progress: v.b("progress"), open: v.b("open"), now } }
}

fn settle_pop(m: &mut PopModel, work: Option<u64>, ok: bool, progress: bool, now: ClockMs) {
    if let Some(w) = work {
        if progress {
            m.progress((now.0.saturating_sub(w)) as f32 / 1600.0);
        }
        if now.0 >= w + 1600 {
            m.finish(
                if ok {
                    Ok(())
                } else {
                    Err("You don't have permission to delete acme-web.".into())
                },
                ClockMs(w + 1600),
            );
        }
    }
}

#[component]
fn LivePop(
    tone: String,
    phrase: bool,
    align: String,
    result: String,
    progress: bool,
    open: bool,
    now: ClockMs,
) -> Element {
    let at = now;
    let key = format!("{tone}{phrase}{align}{open}");
    let align_cls = align.clone();
    let build = move || {
        let mut p = PopModel::new();
        p.tone = ConfirmTone::parse(&tone);
        p.align = PopoverAlign::parse(&align);
        if phrase {
            p = p.require("acme-web");
        }
        if open {
            p = p.with_phase(PopoverPhase::Confirming, at);
        }
        p
    };
    let mut m = use_signal(build.clone());
    let mut k = use_signal(|| key.clone());
    let mut work = use_signal(|| None::<u64>);
    if k() != key {
        k.set(key.clone());
        m.set(build());
        work.set(None);
    }
    let ok = result != "error";
    let mut shown = m.read().clone();
    settle_pop(&mut shown, work(), ok, progress, now);
    rsx! {
        div { class: "oxp-demo-pop {align_cls}",
            ConfirmPopover {
                model: shown,
                now,
                description: "This removes the acme-web project, its 14 deployments and its domains. It can't be undone.".to_string(),
                on_show: move |_| { let mut w = m.write(); settle_pop(&mut w, work(), ok, progress, now); work.set(None); w.show(now) },
                on_dismiss: move |_| m.write().dismiss(now),
                on_type: move |s: String| m.write().type_text(&s),
                on_submit: move |_| {
                    let mut w = m.write();
                    settle_pop(&mut w, work(), ok, progress, now);
                    if w.submit(now) { work.set(Some(now.0)) }
                },
            }
        }
    }
}

fn pop_code(v: &Vals) -> String {
    let mut c = Code::new("ConfirmPopover").pre("let mut p = use_signal(ConfirmPopover::new);");
    if v.b("phrase") {
        c = c.pre("p.write().confirm_text = Some(\"acme-web\".into()); // type to confirm");
    }
    c.raw("model", "p()")
        .raw("now", "clock.now()")
        .raw("description", q("This removes the acme-web project…"))
        .raw(
            "on_submit",
            "move |_| if p.write().submit(now) { spawn(delete(p)) }",
        )
        .raw("on_type", "move |s| p.write().type_text(&s)")
        .done()
}

/// The approval demos added for plan #28.
pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![Demo {
        controls: vec![
            tog("phrase", "Type to confirm", true),
            sel("tone", "Tone", &["danger", "neutral"], "danger"),
            sel("align", "Align", &["start", "center", "end"], "start"),
            sel("result", "Action result", &["ok", "error"], "ok"),
            tog("progress", "Reports progress", true),
            tog("open", "Start open", true),
        ],
        variants: vec![
            var("phrase", "Type the project name to unlock Delete.", &[]),
            var(
                "plain",
                "Consequence and two buttons.",
                &[("phrase", "false")],
            ),
            var(
                "neutral",
                "Reversible: foreground fill.",
                &[("tone", "neutral"), ("phrase", "false")],
            ),
            var(
                "error",
                "Fails: the message and Try again.",
                &[("result", "error"), ("phrase", "false")],
            ),
            var(
                "sweep",
                "No progress reported: an indeterminate sweep.",
                &[("progress", "false"), ("phrase", "false")],
            ),
        ],
        render: pop,
        code: pop_code,
        keyboard: vec![
            ("Enter", "Confirm (shakes when the phrase does not match)"),
            ("Escape", "Dismiss"),
            ("Tab", "Phrase field, Cancel, Delete"),
        ],
        a11y: vec![
            "The card is an alertdialog named by its heading and described by the consequence.",
            "The trigger has aria-expanded; Delete is aria-disabled until the phrase matches (case and outer spaces ignored).",
        ],
        responsive: vec![
            "300 px card, max 100 %; `end` keeps the right edge for buttons at the end of a row.",
        ],
        height: 320,
        align: Align::Start,
        live: true,
        ..mk(
            "confirm-popover",
            "Confirm popover",
            "approvals",
            "A destructive button that grows into a small card: the consequence, an optional type-to-confirm phrase that fills Delete as you type, progress on the button, then a check in the button's shape or the error with Try again.",
            "ConfirmPopover",
            "ConfirmPopoverEl",
            "confirm-popover",
            "kit/approvals.rs",
        )
    }];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 6] = [
    "hold-to-confirm",
    "inline-confirm",
    "confirm-popover",
    "approval-gates",
    "permission-request",
    "agent-approvals",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
