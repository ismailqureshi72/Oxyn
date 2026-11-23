//! Plan #28 parity demos, plans and runs: agent plan, trail, task rows, stepper, onboarding
//! checklist, launch checklist, task breakdown, agent tree, fleet, orchestrator, tree table,
//! activity feed and timeline, countdown, animated counter, agent status and agent run.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
        fleet::{
            Countdown as CountdownModel, CrewAgent, EventKind, Feed, FeedEvent, Lane, OrchPhase,
            Orchestration, Prereq, Readiness, Resolve, TRow, TreeTable as TreeModel,
        },
        plan::{
            BTask, Breakdown, CheckItem, Checklist, FlowStep, PlanDraft, PlanStep, StepState,
            Stepper, TaskRow, Trail, TrailStep,
        },
    },
};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTree,
    AnimatedCounter, Countdown, LaunchChecklist, OnboardingChecklist, TaskBreakdown, TaskRows,
    TreeTable,
};
use oxyn_dioxus::{AgentTrail, FlowStepper};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

fn ago(now: ClockMs, ms: u64) -> Option<ClockMs> {
    Some(ClockMs(now.0.saturating_sub(ms)))
}

macro_rules! keyed {
    ($sig:ident, $key:expr, $build:expr) => {
        let key = $key;
        let mut $sig = use_signal(|| $build);
        let mut k = use_signal(|| key.clone());
        if k() != key {
            k.set(key.clone());
            $sig.set($build);
        }
    };
}

// ---------- agent trail ----------

fn sample_trail(stage: &str, long: bool, now: ClockMs) -> Trail {
    if stage == "empty" {
        return Trail {
            title: "Fix the checkout crash".into(),
            agent: Some("Claude".into()),
            steps: vec![],
        };
    }
    let mut sub: Vec<(String, StepState)> = vec![
        ("Found the empty-cart case".into(), StepState::Done),
        ("Added a guard and 2 tests".into(), StepState::Done),
        ("Running the test suite".into(), StepState::Running),
        ("Check the e2e checkout".into(), StepState::Pending),
    ];
    let mut details: Vec<String> = vec![
        "src/components/checkout/CartWrapper.tsx:1284".into(),
        "+18 −3".into(),
        "182/214".into(),
        String::new(),
    ];
    if long {
        let pre = [
            ("Read the stack trace", "sentry #9921"),
            ("Opened checkout/form.tsx", "412 lines"),
            ("Searched for totals()", "6 hits"),
            ("Read cart.ts", "cart.ts:142"),
            ("Ran the failing test", "1 failed"),
            ("Bisected to 4f2a91c", "7 steps"),
        ];
        for (i, (t, d)) in pre.iter().enumerate() {
            sub.insert(i, ((*t).into(), StepState::Done));
            details.insert(i, (*d).into());
        }
        sub.push(("Update the snapshot".into(), StepState::Pending));
        details.push(String::new());
    }
    let mut s = vec![
        TrailStep {
            took_ms: Some(8_000),
            tool: Some("Linear".into()),
            ..TrailStep::new("Read the bug report", StepState::Done)
        },
        TrailStep {
            took_ms: Some(34_000),
            tool: Some("Sentry".into()),
            ..TrailStep::new("Reproduce the crash", StepState::Done)
        },
        TrailStep {
            status: Some("Testing".into()),
            started: ago(now, 23_000),
            tool: Some("GitHub".into()),
            sub,
            details,
            ..TrailStep::new("Patch the checkout form", StepState::Running)
        },
        TrailStep {
            tool: Some("Notion".into()),
            ..TrailStep::new("Write the release note", StepState::Pending)
        },
        TrailStep {
            gate: true,
            tool: Some("GitHub".into()),
            ask: Some("Open PR #482 against main – 3 files, 214 tests passing".into()),
            ..TrailStep::new("Open a pull request", StepState::Pending)
        },
    ];
    let finish = |x: &mut TrailStep, ms: u64| {
        x.state = StepState::Done;
        x.took_ms = Some(ms);
        x.sub.iter_mut().for_each(|y| y.1 = StepState::Done);
    };
    match stage {
        "waiting" => {
            finish(&mut s[2], 41_000);
            s[3].state = StepState::Done;
            s[3].took_ms = Some(12_000);
            s[4].state = StepState::Waiting;
        }
        "failed" => {
            s[2].state = StepState::Failed;
            s[2].took_ms = Some(23_000);
        }
        "declined" => {
            finish(&mut s[2], 41_000);
            s[3].state = StepState::Done;
            s[3].took_ms = Some(12_000);
            s[4].state = StepState::Declined;
        }
        "done" => {
            finish(&mut s[2], 41_000);
            s[3].state = StepState::Done;
            s[3].took_ms = Some(12_000);
            s[4].state = StepState::Done;
            s[4].took_ms = Some(900);
        }
        _ => {}
    }
    Trail {
        title: "Fix the checkout crash".into(),
        agent: Some("Claude".into()),
        steps: s,
    }
}

fn trail(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        LiveTrail {
            stage: v.string("stage"),
            long: v.s("subs") == "long",
            sub: v.b("sub"),
            dense: v.b("dense"),
            variant: v.string("variant"),
            rtl: v.s("dir") == "rtl",
            now,
        }
    }
}

#[component]
fn LiveTrail(
    stage: String,
    long: bool,
    sub: bool,
    dense: bool,
    variant: String,
    rtl: bool,
    now: ClockMs,
) -> Element {
    keyed!(
        t,
        format!("{stage}-{long}"),
        sample_trail(&stage, long, now)
    );
    rsx! {
        div { class: "oxd-w520",
            AgentTrail {
                trail: t(),
                now,
                substeps: sub,
                dense,
                variant,
                rtl,
                on_approve: move |i: usize| { let _ = t.write().approve(i, now); },
                on_decline: move |i: usize| { let _ = t.write().decline(i); },
            }
        }
    }
}

fn trail_code(v: &Vals) -> String {
    Code::new("AgentTrail")
        .raw("trail", "trail() // Trail { title, agent, steps: [TrailStep { tool, gate, ask, sub, details, .. }] }")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "plain")
        .b("dense", v.b("dense"))
        .b("rtl", v.s("dir") == "rtl")
        .raw("on_approve", "move |i| trail.write().approve(i, now)")
        .raw("on_decline", "move |i| trail.write().decline(i)")
        .done()
}

// ---------- stepper ----------

fn sample_stepper(current: usize, error: bool) -> Stepper {
    let s = |t: &str, d: &str| FlowStep {
        title: t.into(),
        description: Some(d.into()),
        error: None,
        optional: false,
    };
    let mut st = Stepper {
        steps: vec![
            s("Account", "Name and email"),
            s("Workspace", "Pick a name and URL"),
            s("Billing", "Card or invoice"),
            FlowStep {
                optional: true,
                ..s("Invite", "Bring your team")
            },
        ],
        current,
    };
    if error && current < st.steps.len() {
        st.steps[current].error = Some("Card declined, try another".into());
    }
    st
}

fn stepper(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveStepper { current: v.u("current"), error: v.b("error"), orientation: v.string("orientation"), details: v.string("details"), compact: v.b("compact"), select: v.b("select") } }
}

#[component]
fn LiveStepper(
    current: usize,
    error: bool,
    orientation: String,
    details: String,
    compact: bool,
    select: bool,
) -> Element {
    keyed!(
        st,
        format!("{current}{error}"),
        sample_stepper(current, error)
    );
    let on = select.then(|| EventHandler::new(move |i: usize| st.write().current = i));
    rsx! {
        div { class: "oxd-w560 oxp-cq",
            FlowStepper { stepper: st(), orientation, details, compact, on_select: on }
        }
    }
}

fn stepper_code(v: &Vals) -> String {
    Code::new("FlowStepper")
        .raw("stepper", "Stepper { steps, current }")
        .s("orientation", v.s("orientation"), "horizontal")
        .s("details", v.s("details"), "all")
        .b("compact", v.b("compact"))
        .done()
}

/// The plan and run demos added for plan #28.
pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![
        Demo {
            controls: vec![
                sel(
                    "stage",
                    "Stage",
                    &["running", "waiting", "failed", "declined", "done", "empty"],
                    "running",
                ),
                sel("subs", "Sub-steps", &["short", "long"], "short"),
                sel("variant", "Variant", &["plain", "card", "inline"], "plain"),
                sel("dir", "Direction", &["ltr", "rtl"], "ltr"),
                tog("sub", "Show sub-steps", true),
                tog("dense", "Dense", false),
            ],
            variants: vec![
                var(
                    "running",
                    "Step 3 testing: the light runs down to the running sub-step, live timer.",
                    &[],
                ),
                var(
                    "needs-you",
                    "The gated PR step waits: Needs you opens Approve and Decline.",
                    &[("stage", "waiting")],
                ),
                var(
                    "declined",
                    "You said no: the step is struck through.",
                    &[("stage", "declined")],
                ),
                var("failed", "The patch failed.", &[("stage", "failed")]),
                var(
                    "folded",
                    "Twelve sub-steps fold to five around the running one: +N earlier, +N more.",
                    &[("subs", "long")],
                ),
                var(
                    "card",
                    "As a card, every step done.",
                    &[("variant", "card"), ("stage", "done")],
                ),
                var(
                    "inline",
                    "Inline, no header rule, dense.",
                    &[("variant", "inline"), ("dense", "true")],
                ),
                var(
                    "rtl",
                    "Right to left: the rail moves to the right edge.",
                    &[("dir", "rtl")],
                ),
                var("empty", "No steps yet.", &[("stage", "empty")]),
            ],
            render: trail,
            code: trail_code,
            keyboard: vec![
                (
                    "Tab",
                    "Each step with sub-steps, +N earlier / +N more, Needs you",
                ),
                (
                    "Enter / Space",
                    "Open or close a step's sub-steps; show every sub-step",
                ),
                (
                    "Enter",
                    "On Needs you: open Approve and Decline, focus on Approve",
                ),
                ("Escape", "Close the answer panel"),
            ],
            a11y: vec![
                "An ordered list named after the task; the running step and sub-step carry aria-current=step.",
                "The header count reads 3 of 5 steps done; each step reads its title, state and time.",
                "A step with sub-steps toggles them and reports aria-expanded; times and details keep their direction in RTL.",
                "A polite live region says Approved or Declined; the focus outline appears only when moving by keyboard.",
            ],
            responsive: vec![
                "Rows are fixed: 32 px a step, 24 px a sub-step, 40 and 28 px on touch screens.",
                "Long sub-step lists fold to five around the running one; the right side never takes more than 40%.",
                "Right to left: the rail moves to the right edge and the branch bends the other way.",
            ],
            height: 440,
            align: Align::Start,
            live: true,
            ..mk(
                "agent-trail",
                "Agent trail",
                "tools",
                "The steps an agent works through as one quiet trail: a header with the agent's mark, the task and 3/5; done steps with a check and their time; the running step shimmering what it does with a live timer, a short fading light running down the trail into the sub-step it is on; long sub-step lists fold to five (+N earlier, +N more); a gated step waits with a dashed ring and Approval, then Needs you opens Approve and Decline; declined steps are struck through; each step shows its tool's coloured logo. RTL, keyboard and live-region built in.",
                "AgentTrail",
                "AgentTrailEl",
                "agent-trail",
                "kit/run.rs",
            )
        },
        Demo {
            controls: vec![
                rng("current", "Current step", 0.0, 4.0, 1.0, "2"),
                sel(
                    "orientation",
                    "Orientation",
                    &["horizontal", "vertical"],
                    "horizontal",
                ),
                sel("details", "Descriptions", &["all", "current"], "all"),
                tog("compact", "Compact", false),
                tog("error", "Error on current", false),
                tog("select", "Completed steps selectable", true),
            ],
            variants: vec![
                var("horizontal", "Step 3 of 4.", &[]),
                var(
                    "vertical",
                    "Vertical with current-only descriptions.",
                    &[("orientation", "vertical"), ("details", "current")],
                ),
                var("compact", "Markers and a caption.", &[("compact", "true")]),
                var("error", "Billing failed.", &[("error", "true")]),
                var("complete", "All steps complete.", &[("current", "4")]),
            ],
            render: stepper,
            code: stepper_code,
            keyboard: vec![
                ("← → ↑ ↓", "Move between reachable steps"),
                ("Home / End", "First / current step"),
                ("Enter / Space", "Select a completed step"),
            ],
            a11y: vec![
                "nav with an accessible name; the current step has aria-current=step.",
                "All complete is announced in a status region.",
            ],
            responsive: vec![
                "Horizontal steppers hide their text below 30 rem (container query) and keep the markers.",
            ],
            height: 260,
            align: Align::Start,
            ..mk(
                "stepper",
                "Stepper",
                "tools",
                "Where a person is in a multi-step flow: numbers morph into checks or alerts, connectors fill, descriptions for every step or just the current one, compact markers with a caption, and arrow-key selection of completed steps.",
                "FlowStepper",
                "FlowStepperEl",
                "stepper",
                "kit/run.rs",
            )
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 16] = [
    "agent-plan",
    "agent-trail",
    "task-rows",
    "stepper",
    "onboarding-checklist",
    "launch-checklist",
    "task-breakdown",
    "agent-tree",
    "agent-fleet",
    "agent-orchestrator",
    "tree-table",
    "activity-feed",
    "countdown",
    "animated-counter",
    "agent-status",
    "agent-run",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
