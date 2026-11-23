//! Destructive confirmations and run-level approvals (plan #28): `HoldToConfirm`,
//! `InlineConfirm`, `ConfirmPopover`, `ApprovalGates`, `PermissionRequest`, `AgentApprovals`.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
        code_spans,
        confirm::{
            ConfirmMode, ConfirmPopover as PopModel, HoldConfirm, InlineConfirm as MorphModel,
            MorphFace, PopoverPhase,
        },
        gates::{
            AgentApprovals as AgentModel, GateDecision, GateStatus, Gates, Outcome, PermRisk,
            PermissionRequest as PermModel, RuleScope, Scope,
        },
    },
};

use crate::composer::icons::Icon;

/// Confirm popover: the trigger grows into a card with the consequence, an optional
/// type-to-confirm phrase, progress on Confirm, then a check (or the failure with Retry).
#[component]
pub fn ConfirmPopover(
    /// Model.
    model: PopModel,
    /// Shared clock.
    now: ClockMs,
    /// Trigger label.
    #[props(default = "Delete project".to_string())]
    label: String,
    /// Card heading.
    #[props(default = "Delete this project?".to_string())]
    title: String,
    /// The consequence (describes the dialog).
    #[props(default)]
    description: Option<String>,
    /// Trigger icon.
    #[props(default = "trash".to_string())]
    icon: String,
    /// Confirm label.
    #[props(default = "Delete".to_string())]
    confirm_label: String,
    /// Cancel label.
    #[props(default = "Cancel".to_string())]
    cancel_label: String,
    /// Progress label.
    #[props(default = "Deleting".to_string())]
    pending_label: String,
    /// Success label (no longer than the trigger label).
    #[props(default = "Deleted".to_string())]
    success_label: String,
    /// Failure heading.
    #[props(default = "That didn't work".to_string())]
    error_title: String,
    /// Retry label.
    #[props(default = "Try again".to_string())]
    retry_label: String,
    /// Trigger pressed.
    #[props(default)]
    on_show: EventHandler<()>,
    /// Cancel or Escape.
    #[props(default)]
    on_dismiss: EventHandler<()>,
    /// Phrase typed.
    #[props(default)]
    on_type: EventHandler<String>,
    /// Confirm pressed.
    #[props(default)]
    on_submit: EventHandler<()>,
) -> Element {
    let phase = model.phase(now);
    let open = model.open(now);
    let tone = model.tone.id();
    let fill = model.match_fill();
    let shake = model.shake(now);
    let sweep = model.sweep(now);
    let trig = match phase {
        PopoverPhase::Success => {
            rsx! { span { class: "oxp-cp-ok", svg { class: "oxp-draw", view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3 8.5l3.2 3L13 4.5" } } "{success_label}" } }
        }
        _ => rsx! { Icon { name: icon.clone(), size: 14.0 } "{label}" },
    };
    let btn = match phase {
        PopoverPhase::Working(p) => {
            let (w, pct) = match p {
                Some(p) => (
                    format!("--w:{:.0}%", p * 100.0),
                    format!(" {:.0}%", p * 100.0),
                ),
                None => (
                    format!("--w:30%;--x:{:.0}%", sweep * 130.0 - 30.0),
                    String::new(),
                ),
            };
            rsx! { button { class: "oxp-cp-go {tone} working", r#type: "button", "aria-disabled": "true", style: "{w}",
            span { class: "oxp-cp-bar", "data-ind": "{p.is_none()}" } span { class: "oxp-cp-lb", "{pending_label}{pct}" } } }
        }
        PopoverPhase::Error => {
            rsx! { button { class: "oxp-cp-go {tone}", r#type: "button", onclick: move |_| on_submit.call(()), "{retry_label}" } }
        }
        _ => {
            rsx! { button { class: "oxp-cp-go {tone}", r#type: "button", "aria-disabled": "{!model.unlocked()}",
            style: "--m:{fill * 100.0:.0}%", onclick: move |_| on_submit.call(()),
            span { class: "oxp-cp-m" } span { class: "oxp-cp-lb", "{confirm_label}" } } }
        }
    };
    rsx! {
        div { class: "oxp-cp {model.align.id()}",
            button { class: "oxp-cp-trig {tone}", r#type: "button", "aria-expanded": "{open}", "aria-haspopup": "dialog",
                "data-phase": phase.id(), onclick: move |_| on_show.call(()), {trig} }
            if open {
                div { class: "oxp-cp-card", role: "alertdialog", "aria-label": "{title}", "aria-describedby": "oxp-cp-desc",
                    style: "transform:translateX({shake:.1}px)",
                    onkeydown: move |e: KeyboardEvent| match e.key() { Key::Escape => on_dismiss.call(()), Key::Enter => on_submit.call(()), _ => {} },
                    if phase == PopoverPhase::Error {
                        div { class: "oxp-cp-t err", Icon { name: "alert", size: 14.0 } "{error_title}" }
                        div { id: "oxp-cp-desc", class: "oxp-cp-d", "{model.error.clone().unwrap_or_default()}" }
                    } else {
                        div { class: "oxp-cp-t", "{title}" }
                        if let Some(d) = description.clone() { div { id: "oxp-cp-desc", class: "oxp-cp-d", "{d}" } }
                    }
                    if let Some(phrase) = model.confirm_text.clone() {
                        if phase == PopoverPhase::Confirming {
                            label { class: "oxp-cp-ty", "Type " code { "{phrase}" } " to confirm"
                                input { r#type: "text", value: "{model.typed}", autocomplete: "off", spellcheck: "false",
                                    "aria-invalid": "{fill == 0.0 && !model.typed.is_empty()}",
                                    oninput: move |e| on_type.call(e.value()) }
                            }
                        }
                    }
                    div { class: "oxp-cp-btns",
                        button { class: "oxp-cp-no", r#type: "button", onclick: move |_| on_dismiss.call(()), "{cancel_label}" }
                        {btn}
                    }
                }
            }
        }
    }
}

/// Status of a gate as a dot class (for hosts drawing their own rows).
pub fn gate_dot(s: GateStatus) -> &'static str {
    match s {
        GateStatus::Open => "wait",
        GateStatus::Approved | GateStatus::Edited => "ok",
        GateStatus::Rejected => "no",
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
