//! Plan #32 (affectUI fidelity pass), overnight batch family B part 6 demos: the exact Autonomy
//! Dial and Away Summary, each with every affectUI style and state.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::autonomyfid::{AutonomyAction, autonomy_demo},
};
use oxyn_dioxus::{AutonomyDial, AwaySummary};

use super::{Code, mk, sel, var};
use crate::{Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn cstyle(v: &Vals) -> String {
    if v.s("style") == "classic" {
        "classic".into()
    } else {
        String::new()
    }
}

// ---------- Autonomy Dial ----------

fn detailed() -> Vec<AutonomyAction> {
    let details = [
        "Articles, macros and past answers",
        "Saved as drafts on the ticket",
        "Priority, tags and the assignee",
        "Replies go straight to the customer",
        "Files older than 90 days",
        "Up to the order's full amount",
    ];
    autonomy_demo()
        .into_iter()
        .zip(details)
        .map(|(a, d)| AutonomyAction {
            detail: d.into(),
            ..a
        })
        .collect()
}

fn autonomy_dial(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("level"),
        v.s("state"),
        v.s("width")
    );
    let level = match v.s("level") {
        "always" => 0,
        "full" => 2,
        _ => 1,
    };
    let state = v.s("state");
    let pins: Option<Vec<String>> = (state == "locked").then(|| {
        vec![
            "read".to_string(),
            "purge".to_string(),
            "refund".to_string(),
        ]
    });
    let point = if state == "pointed" { "draft" } else { "" };
    let actions = (state == "details").then(detailed);
    let width = if v.s("width") == "narrow" {
        "320px"
    } else {
        "432px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AutonomyDial { actions, default_level: level, default_pinned: pins, point: point.to_string(),
                title: "Support agent".to_string(), style: cstyle(v),
                on_level_change: move |l: u8| msg.set(format!("onLevelChange({l})")),
                on_pinned_change: move |p: Vec<String>| msg.set(format!("onPinnedChange({p:?})")) }
            {status(msg)}
        }
    }
}

fn autonomy_dial_code(_: &Vals) -> String {
    Code::new("AutonomyDial")
        .raw("actions", "Some(agent.actions())")
        .raw("default_level", "1")
        .raw("default_pinned", "Some(vec![\"refund\".into()])")
        .raw("title", "\"Support agent\".into()")
        .raw("on_level_change", "move |l| settings.set_level(l)")
        .raw("on_pinned_change", "move |p| settings.set_locks(p)")
        .done()
}

// ---------- Away Summary ----------

fn agent_glyph() -> Element {
    rsx! {
        svg { width: "13", height: "13", view_box: "0 0 12 12", "aria-hidden": "true",
            for (i, o) in [1.0, 0.45, 1.0, 0.45, 1.0, 0.45, 1.0, 0.45, 1.0].iter().enumerate() {
                rect { key: "{i}", "data-k": "{i}", x: "{(i % 3) * 4}", y: "{(i / 3) * 4}", width: "3.2", height: "3.2", rx: ".6", fill: "#8B5CF6", opacity: "{o}" }
            }
        }
    }
}

fn away_summary(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let resolved: Vec<String> = if state == "resolved" {
        vec!["export".into()]
    } else {
        vec![]
    };
    let since_clock = (state == "folded").then_some(9 * 60 + 40);
    let work_ms = if state == "slow" { 2_400 } else { 0 };
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "600px"
    };
    let mute = "var(--oxyn-color-fg-muted)";
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:14px;font-size:13.5px",
            div { style: "align-self:flex-end;max-width:85%;padding:6px 12px;border-radius:14px;background:color-mix(in srgb,currentColor 7%,transparent);line-height:1.35",
                "Move the nightly export to the new bucket – shout if anything's unclear." }
            div { style: "display:flex;align-items:center;gap:10px;line-height:1.35",
                {agent_glyph()}
                span { "Sure. I'll break it into steps and carry on while you're away." }
            }
            AwaySummary { now, resolved, since_clock, work_ms, expanded: state == "expanded", default_collapsed: state == "folded", style: cstyle(v),
                on_action: move |id: String| msg.set(format!("onAction({id:?})")),
                on_dismiss: move |_| msg.set("onDismiss()".into()),
                on_collapsed_change: move |c: bool| msg.set(format!("onCollapsedChange({c})")) }
            div { style: "display:flex;align-items:center;height:46px;padding:0 11px 0 14px;border:1px solid color-mix(in srgb,currentColor 10%,transparent);border-radius:12px;background:var(--oxaw-card);box-shadow:var(--oxaw-sh);color:{mute};font-size:13px",
                span { style: "flex:1", "Reply to the agent…" }
                span { "aria-hidden": "true", style: "display:inline-flex;align-items:center;justify-content:center;width:24px;height:24px;border-radius:50%;background:color-mix(in srgb,currentColor 30%,transparent)",
                    svg { width: "12", height: "12", view_box: "0 0 16 16", fill: "none", stroke: "#fff", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", path { d: "M8 13V3.5M4 7l4-4 4 4" } } }
            }
            {status(msg)}
        }
    }
}

fn away_summary_code(_: &Vals) -> String {
    Code::new("AwaySummary")
        .raw("items", "Some(thread.away_items())")
        .raw("away", "minutes_away")
        .raw("now", "clock.now()")
        .raw("on_action", "move |id| agent.act(id)")
        .raw("on_dismiss", "move |_| seen.set(true)")
        .done()
}

/// Family B part 6 demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
        Demo {
            controls: vec![
                style(),
                sel("level", "Level", &["risky", "always", "full"], "risky"),
                sel(
                    "state",
                    "State",
                    &["default", "locked", "pointed", "details"],
                    "default",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "if-risky",
                    "As affectUI ships it: If risky – the two safe actions run on their own, the risky ones and the two locked destructive ones ask first.",
                    &[],
                ),
                var(
                    "full-auto",
                    "Full auto: everything runs except what you locked.",
                    &[("level", "full")],
                ),
                var(
                    "always-ask",
                    "Always ask: every action waits for you, and the first group says so.",
                    &[("level", "always")],
                ),
                var(
                    "locked",
                    "A lock set on a safe action: it always asks, whatever the level.",
                    &[("state", "locked")],
                ),
                var(
                    "pointed",
                    "A row pointed at: the row tints and its lock shows.",
                    &[("state", "pointed")],
                ),
                var(
                    "details",
                    "Actions with a detail line under the name.",
                    &[("state", "details")],
                ),
                var(
                    "narrow",
                    "Under 340 px the detail lines are left out and the levels tighten.",
                    &[("width", "narrow"), ("state", "details")],
                ),
                var(
                    "classic",
                    "Classic style: the levels in one track inside a white card, each action its own card on a grey panel.",
                    &[("style", "classic")],
                ),
            ],
            render: autonomy_dial,
            code: autonomy_dial_code,
            keyboard: vec![
                ("← → ↑ ↓", "Picks the previous or next level"),
                ("Tab", "From the levels to the locks of the actions"),
                ("Space / Enter", "Sets or removes the focused lock"),
            ],
            a11y: vec![
                "The levels are a radio group named Autonomy, described by what the chosen level means.",
                "Both groups are labelled sections; every lock is a toggle button with aria-pressed that names its action.",
                "Every change is announced politely: the level and how many actions run on their own or ask first.",
            ],
            responsive: vec![
                "432 px up to its container.",
                "Under 340 px the detail lines go and the levels get tighter; without hover the locks always show and rows grow taller.",
            ],
            height: 560,
            ..mk(
                "autonomy-dial",
                "Autonomy Dial",
                "tools",
                "How much an agent may do on its own – three levels, and its real actions gliding between Runs on its own and Asks you first, with a lock for what always asks.",
                "AutonomyDial",
                "AutonomyDialEl",
                "autonomy-dial",
                "parity/autonomyfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["default", "resolved", "expanded", "folded", "slow"],
                    "default",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: two need you, one failed, five done – five rows, Show 3 more and Got it, in the thread where you left off.",
                    &[],
                ),
                var(
                    "resolved",
                    "A decision made: the row turns done in place, its tick draws and the counts roll.",
                    &[("state", "resolved")],
                ),
                var(
                    "expanded",
                    "Show 3 more: every row, and Show less.",
                    &[("state", "expanded")],
                ),
                var(
                    "folded",
                    "Since a time, folded at first: a one-line divider that opens the card again.",
                    &[("state", "folded")],
                ),
                var(
                    "slow-retry",
                    "A retry that takes a while: the button works for a moment before the row settles.",
                    &[("state", "slow")],
                ),
                var(
                    "narrow",
                    "Narrow: the counts drop under the title and long lines end in an ellipsis.",
                    &[("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: the header in a grey band, a stronger border and shadow.",
                    &[("style", "classic")],
                ),
            ],
            render: away_summary,
            code: away_summary_code,
            keyboard: vec![
                ("Tab", "Through the row actions, then Show more and Got it"),
                (
                    "Enter / Space",
                    "Runs the focused action; Got it folds the card and focuses the divider, which opens it again",
                ),
                ("Escape", "Folds the card from anywhere inside it"),
            ],
            a11y: vec![
                "A region named While you were away with a heading that says how long; the rows are a list.",
                "Every action names its task (\"Decide: Export job\") and each row says its state in words.",
                "Got it and the divider are buttons with aria-expanded; a working button stays focusable (aria-disabled); changes are announced politely.",
            ],
            responsive: vec![
                "600 px up to its container; rows keep one height in every state.",
                "Under 460 px the counts drop under the title; titles, details and long action labels end in an ellipsis.",
            ],
            height: 600,
            live: true,
            ..mk(
                "away-summary",
                "Away Summary",
                "tools",
                "The While you were away card for a chat thread – how long you were gone, what needs you, what failed and what's done as a short list with one action per row; Got it folds it into a one-line divider.",
                "AwaySummary",
                "AwaySummaryEl",
                "away-summary",
                "parity/awayfid.rs",
            )
        },
    ]
}
