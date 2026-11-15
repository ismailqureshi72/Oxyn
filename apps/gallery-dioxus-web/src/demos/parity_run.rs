//! Plan #28 parity demos, plans and runs: agent plan, trail, task rows, stepper, onboarding
//! checklist, release readiness, task breakdown, agent tree, fleet, orchestrator, tree table,
//! activity feed and timeline, countdown, animated counter, agent status and agent run.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
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
use oxyn_dioxus::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTrail,
    AgentTree, AnimatedCounter, Countdown, FlowStepper, OnboardingChecklist, ReleaseReadiness,
    TaskBreakdown, TaskRows, TreeTable,
};

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

// ---------- agent plan ----------

fn sample_plan(phase: &str, now: ClockMs) -> PlanDraft {
    let mut p = PlanDraft::new(
        "Add rate limiting to the public API",
        vec![
            PlanStep::new("s1", "Read the API router and middleware")
                .tool("read")
                .detail("src/api/router.rs, src/api/middleware/*"),
            PlanStep::new("s2", "Add a token-bucket limiter keyed by API key").tool("edit"),
            PlanStep::new("s3", "Return 429 with Retry-After").tool("edit"),
            PlanStep::new("s4", "Write tests for burst and refill").tool("bash"),
            PlanStep::new("s5", "Update the API docs").tool("edit"),
        ],
    );
    p.estimate = Some("~3 min".into());
    match phase {
        "running" => {
            p.run(ClockMs(now.0.saturating_sub(40_000)));
            p.finish_step(Ok(()), ClockMs(now.0.saturating_sub(31_000)));
            p.finish_step(Ok(()), ClockMs(now.0.saturating_sub(12_000)));
        }
        "failed" => {
            p.run(ClockMs(now.0.saturating_sub(60_000)));
            for t in [52_000, 30_000, 21_000] {
                p.finish_step(Ok(()), ClockMs(now.0.saturating_sub(t)));
            }
            p.finish_step(
                Err("2 tests failed: refill_after_burst, retry_after_header".into()),
                ClockMs(now.0.saturating_sub(4_000)),
            );
        }
        "done" => {
            p.run(ClockMs(now.0.saturating_sub(90_000)));
            for t in [80_000, 60_000, 44_000, 20_000, 5_000] {
                p.finish_step(Ok(()), ClockMs(now.0.saturating_sub(t)));
            }
        }
        _ => {}
    }
    p
}

fn plan(v: &Vals, now: ClockMs) -> Element {
    rsx! { LivePlan { phase: v.string("phase"), cancel: v.b("cancel"), now } }
}

#[component]
fn LivePlan(phase: String, cancel: bool, now: ClockMs) -> Element {
    keyed!(p, phase.clone(), sample_plan(&phase, now));
    rsx! {
        div { class: "oxd-w560",
            AgentPlan {
                plan: p(),
                now,
                cancellable: cancel,
                on_rename: move |(i, t): (usize, String)| p.write().rename(i, &t),
                on_move: move |(a, b)| p.write().reorder(a, b),
                on_remove: move |i| p.write().remove(i),
                on_add: move |t: String| { p.write().add(&t); },
                on_run: move |_| p.write().run(now),
                on_cancel: move |_| p.set(sample_plan("draft", now)),
                on_retry: move |id: String| p.write().retry_from(&id, now),
            }
        }
    }
}

fn plan_code(_: &Vals) -> String {
    Code::new("AgentPlan")
        .pre("let mut plan = use_signal(|| PlanDraft::new(\"Add rate limiting\", steps));")
        .raw("plan", "plan()")
        .raw("now", "clock.now()")
        .raw("on_rename", "move |(i, t)| plan.write().rename(i, &t)")
        .raw("on_move", "move |(a, b)| plan.write().reorder(a, b)")
        .raw(
            "on_run",
            "move |_| { plan.write().run(now); agent.start(plan()) }",
        )
        .raw("on_retry", "move |id| plan.write().retry_from(&id, now)")
        .done()
}

// ---------- agent trail ----------

fn sample_trail(stage: &str, now: ClockMs) -> Trail {
    let mut s = vec![
        TrailStep {
            took_ms: Some(4_000),
            tool: Some("GitHub".into()),
            ..TrailStep::new("Read issue #482", StepState::Done)
        },
        TrailStep {
            took_ms: Some(64_000),
            tool: Some("Code".into()),
            ..TrailStep::new("Reproduce the race", StepState::Done)
        },
        TrailStep {
            status: Some("Testing".into()),
            started: ago(now, 23_000),
            tool: Some("Code".into()),
            sub: vec![
                ("Patch refresh()".into(), StepState::Done),
                ("Run auth tests".into(), StepState::Running),
                ("Run e2e login".into(), StepState::Pending),
            ],
            ..TrailStep::new("Fix the refresh-token race", StepState::Running)
        },
        TrailStep {
            gate: true,
            tool: Some("GitHub".into()),
            ..TrailStep::new("Open a pull request", StepState::Pending)
        },
        TrailStep {
            tool: Some("Linear".into()),
            ..TrailStep::new("Close the Linear ticket", StepState::Pending)
        },
    ];
    match stage {
        "waiting" => {
            s[2] = TrailStep {
                took_ms: Some(41_000),
                ..TrailStep::new("Fix the refresh-token race", StepState::Done)
            };
            s[3].state = StepState::Waiting;
        }
        "failed" => {
            s[2].state = StepState::Failed;
            s[2].took_ms = Some(23_000);
        }
        "declined" => {
            s[2].state = StepState::Done;
            s[2].took_ms = Some(41_000);
            s[3].state = StepState::Declined;
        }
        _ => {}
    }
    Trail {
        title: "Fix login loop (#482)".into(),
        agent: Some("Claude".into()),
        steps: s,
    }
}

fn trail(v: &Vals, now: ClockMs) -> Element {
    let t = sample_trail(v.s("stage"), now);
    rsx! { div { class: "oxd-w520", AgentTrail { trail: t, now, substeps: v.b("sub"), dense: v.b("dense") } } }
}

fn trail_code(v: &Vals) -> String {
    Code::new("AgentTrail")
        .raw("trail", "trail() // Trail { title, agent, steps }")
        .raw("now", "clock.now()")
        .b("dense", v.b("dense"))
        .done()
}

// ---------- task rows ----------

fn sample_rows(open: bool) -> Vec<TaskRow> {
    let d = |a: &str, b: &str| (a.to_string(), b.to_string());
    vec![
        TaskRow {
            label: "Reconcile Stripe payouts".into(),
            amount: Some("$12,480".into()),
            state: StepState::Done,
            step: (3, 3),
            details: vec![d("Fetched 214 payouts", "2s"), d("Matched invoices", "5s")],
            open: false,
        },
        TaskRow {
            label: "Categorise card spend".into(),
            amount: Some("318 rows".into()),
            state: StepState::Running,
            step: (1, 3),
            details: vec![
                d("Loaded statements", "done"),
                d("Classifying merchants", "running"),
                d("Write to ledger", "queued"),
            ],
            open,
        },
        TaskRow {
            label: "Flag duplicate invoices".into(),
            amount: Some("4 found".into()),
            state: StepState::Failed,
            step: (2, 3),
            details: vec![d("Vendor API timed out", "retry")],
            open: false,
        },
        TaskRow {
            label: "Draft the monthly summary".into(),
            amount: None,
            state: StepState::Pending,
            step: (0, 2),
            details: vec![],
            open: false,
        },
    ]
}

fn rows(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveRows { variant: v.string("variant"), open: v.b("open") } }
}

#[component]
fn LiveRows(variant: String, open: bool) -> Element {
    keyed!(r, format!("{open}"), sample_rows(open));
    rsx! {
        div { class: "oxd-w520",
            TaskRows {
                rows: r(),
                variant,
                on_toggle: move |i: usize| { let mut w = r.write(); w[i].open = !w[i].open; },
                on_retry: move |i: usize| { let mut w = r.write(); w[i].state = StepState::Running; },
            }
        }
    }
}

fn rows_code(v: &Vals) -> String {
    Code::new("TaskRows")
        .raw("rows", "rows()")
        .s("variant", v.s("variant"), "capsules")
        .raw("on_toggle", "move |i| rows.write()[i].open ^= true")
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

// ---------- onboarding checklist ----------

fn sample_check(done: usize, hidden: bool) -> Checklist {
    let it = |id: &str, t: &str, b: &str, a: &str| CheckItem {
        id: id.into(),
        title: t.into(),
        body: b.into(),
        action: Some(a.into()),
        done: false,
    };
    let mut c = Checklist {
        items: vec![
            it(
                "profile",
                "Add your profile photo",
                "Helps your team recognise you in threads and approvals.",
                "Upload photo",
            ),
            it(
                "repo",
                "Connect a repository",
                "Agents read and branch from it. Read-only by default.",
                "Connect GitHub",
            ),
            it(
                "invite",
                "Invite your team",
                "Approvals can route to anyone in the workspace.",
                "Invite",
            ),
            it(
                "rules",
                "Set approval rules",
                "Choose what agents may do without asking.",
                "Open rules",
            ),
            it(
                "run",
                "Run your first agent",
                "Try “Fix the flaky login test”.",
                "Start a run",
            ),
        ],
        open: None,
        hidden,
    };
    for i in c.items.iter_mut().take(done) {
        i.done = true;
    }
    c.open = c.items.iter().find(|i| !i.done).map(|i| i.id.clone());
    c
}

fn check(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveCheck { done: v.u("done"), hidden: v.b("hidden") } }
}

#[component]
fn LiveCheck(done: usize, hidden: bool) -> Element {
    keyed!(c, format!("{done}{hidden}"), sample_check(done, hidden));
    rsx! {
        div { class: "oxd-w480",
            OnboardingChecklist {
                list: c(),
                on_toggle: move |id: String| c.write().toggle(&id),
                on_open: move |id: String| c.write().open = Some(id),
                on_hide: move |h: bool| c.write().hidden = h,
            }
        }
    }
}

fn check_code(_: &Vals) -> String {
    Code::new("OnboardingChecklist")
        .raw("list", "list()")
        .raw("on_toggle", "move |id| list.write().toggle(&id)")
        .raw("on_hide", "move |h| list.write().hidden = h")
        .done()
}

// ---------- release readiness ----------

fn sample_ready(done: bool, fail: bool) -> Readiness {
    let p = |g: &str, t: &str, o: &str, d: bool, b: bool| Prereq {
        group: g.into(),
        title: t.into(),
        owner: o.into(),
        done: d,
        blocking: b,
    };
    Readiness {
        version: "v2.4.0".into(),
        items: vec![
            p("Quality", "All CI checks green", "CI", true, true),
            p("Quality", "No open P0 bugs", "Maya", done, true),
            p("Quality", "Load test at 2× peak", "Ravi", true, false),
            p("Docs", "Changelog merged", "Agent", true, true),
            p("Docs", "Migration guide reviewed", "Lea", done, false),
            p("Launch", "Status page scheduled", "Ops", done, false),
        ],
        release: if fail {
            StepState::Failed
        } else {
            StepState::Pending
        },
        error: fail.then(|| "Deploy hook returned 503. Nothing was released.".into()),
    }
}

fn ready(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveReady { done: v.b("done"), fail: v.b("fail"), now } }
}

#[component]
fn LiveReady(done: bool, fail: bool, now: ClockMs) -> Element {
    keyed!(r, format!("{done}{fail}"), sample_ready(done, fail));
    let mut at = use_signal(|| None::<u64>);
    let mut shown = r();
    if let Some(a) = at() {
        shown.release = if now.0 >= a + 1500 {
            StepState::Done
        } else {
            StepState::Running
        };
        shown.error = None;
    }
    rsx! {
        div { class: "oxd-w520",
            ReleaseReadiness {
                model: shown,
                on_toggle: move |i: usize| { let mut w = r.write(); w.items[i].done ^= true; },
                on_release: move |_| at.set(Some(now.0)),
            }
        }
    }
}

fn ready_code(_: &Vals) -> String {
    Code::new("ReleaseReadiness")
        .raw("model", "readiness()")
        .raw("on_release", "move |_| spawn(release())")
        .done()
}

// ---------- task breakdown ----------

fn sample_breakdown(deadline: f32) -> Breakdown {
    let t = |id: &str, title: &str, d: f32, o: &str, deps: &[&str]| BTask {
        id: id.into(),
        title: title.into(),
        days: d,
        owner: o.into(),
        deps: deps.iter().map(|s| s.to_string()).collect(),
    };
    Breakdown {
        goal: "Launch usage-based billing by Dec 1".into(),
        deadline_days: deadline,
        tasks: vec![
            t("meter", "Emit usage events from the API", 2.0, "RV", &[]),
            t(
                "agg",
                "Aggregate usage per customer per day",
                1.5,
                "RV",
                &["meter"],
            ),
            t("price", "Model the price tiers in Stripe", 1.0, "MK", &[]),
            t(
                "inv",
                "Generate invoices from usage",
                2.0,
                "MK",
                &["agg", "price"],
            ),
            t("ui", "Usage page in settings", 2.5, "LE", &["agg"]),
            t("qa", "End-to-end billing test", 1.0, "LE", &["inv"]),
        ],
    }
}

fn breakdown(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveBreakdown { deadline: v.f("deadline") as f32, stream: v.b("stream"), now } }
}

#[component]
fn LiveBreakdown(deadline: f32, stream: bool, now: ClockMs) -> Element {
    keyed!(b, format!("{deadline}"), sample_breakdown(deadline));
    let start = use_signal(|| now.0);
    let shown = if stream {
        ((now.0.saturating_sub(start()) / 450) as usize).min(b().tasks.len())
    } else {
        usize::MAX
    };
    rsx! {
        div { class: "oxd-w560",
            TaskBreakdown {
                model: b(),
                shown,
                on_shift: move |(i, up): (usize, bool)| b.write().shift(i, up),
            }
        }
    }
}

fn breakdown_code(_: &Vals) -> String {
    Code::new("TaskBreakdown")
        .raw(
            "model",
            "plan() // Breakdown { goal, deadline_days, tasks }",
        )
        .raw("on_shift", "move |(i, up)| plan.write().shift(i, up)")
        .done()
}

// ---------- agent tree and fleet ----------

fn crew(now: ClockMs, problem: &str) -> CrewAgent {
    let a = |id: &str, n: &str, task: &str, s: StepState, cost: f64, ms: u64| {
        let mut x = CrewAgent::new(id, n, task, s);
        x.cost = cost;
        x.started = ago(now, ms);
        if s.settled() {
            x.ended = ago(now, ms / 3);
        }
        x
    };
    let mut root = a(
        "lead",
        "Lead",
        "Ship the pricing page refresh",
        StepState::Running,
        0.31,
        240_000,
    );
    root.model = Some("Opus 5.5".into());
    let mut res = a(
        "research",
        "Researcher",
        "Compare 4 competitor pricing pages",
        StepState::Done,
        0.42,
        200_000,
    );
    res.children = vec![
        a(
            "r1",
            "Browser 1",
            "Read linear.app/pricing",
            StepState::Done,
            0.06,
            180_000,
        ),
        a(
            "r2",
            "Browser 2",
            "Read vercel.com/pricing",
            StepState::Done,
            0.07,
            170_000,
        ),
    ];
    let mut wr = a(
        "writer",
        "Writer",
        "Draft new plan copy",
        StepState::Running,
        0.18,
        90_000,
    );
    wr.model = Some("Sonnet 5.5".into());
    let mut rev = a(
        "review",
        "Reviewer",
        "Check copy against brand rules",
        StepState::Waiting,
        0.04,
        30_000,
    );
    rev.ask = Some("“Unlimited seats” conflicts with the Team plan limit of 50. Keep it?".into());
    let mut code = a(
        "coder",
        "Coder",
        "Update pricing.tsx",
        StepState::Pending,
        0.0,
        0,
    );
    code.started = None;
    if problem == "failed" {
        rev.state = StepState::Failed;
        rev.ask = None;
        rev.error = Some("Brand rules doc is private (403).".into());
    }
    wr.children = vec![rev];
    root.children = vec![res, wr, code];
    root
}

fn tree(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveTree { problem: v.string("problem"), fold: v.b("fold"), now } }
}

#[component]
fn LiveTree(problem: String, fold: bool, now: ClockMs) -> Element {
    keyed!(r, problem.clone(), crew(now, &problem));
    keyed!(
        f,
        format!("{fold}"),
        if fold {
            vec!["research".to_string()]
        } else {
            vec![]
        }
    );
    rsx! {
        div { class: "oxd-w560",
            AgentTree {
                root: r(),
                now,
                folded: f(),
                on_fold: move |id: String| { let mut w = f.write(); if let Some(i) = w.iter().position(|x| *x == id) { w.remove(i); } else { w.push(id); } },
                on_resolve: move |(id, x): (String, Resolve)| { if let Some(a) = r.write().find_mut(&id) { a.resolve(x) } },
            }
        }
    }
}

fn tree_code(_: &Vals) -> String {
    Code::new("AgentTree")
        .raw("root", "crew() // CrewAgent with children")
        .raw("now", "clock.now()")
        .raw("folded", "folded()")
        .raw("on_resolve", "move |(id, r)| agent.answer(&id, r)")
        .done()
}

fn fleet(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveFleet { open: v.string("open"), now } }
}

#[component]
fn LiveFleet(open: String, now: ClockMs) -> Element {
    let make = move || {
        let a = crew(now, "waiting");
        let mut b = CrewAgent::new(
            "support",
            "Support",
            "Answer 12 open tickets",
            StepState::Running,
        );
        b.cost = 0.12;
        b.started = ago(now, 400_000);
        b.children = vec![
            CrewAgent::new("t1", "Ticket #881", "Refund request", StepState::Done),
            CrewAgent::new("t2", "Ticket #884", "SSO setup", StepState::Running),
        ];
        let mut c = CrewAgent::new("ops", "Ops", "Rotate staging secrets", StepState::Failed);
        c.cost = 0.03;
        c.error = Some("Vault token expired".into());
        vec![a, b, c]
    };
    keyed!(ag, open.clone(), make());
    keyed!(
        o,
        open.clone(),
        if open == "none" {
            None
        } else {
            Some(open.clone())
        }
    );
    rsx! {
        div { class: "oxd-w760 oxp-cq",
            AgentFleet {
                agents: ag(),
                now,
                open: o(),
                on_open: move |x| o.set(x),
                on_resolve: move |(id, x): (String, Resolve)| { for a in ag.write().iter_mut() { if let Some(n) = a.find_mut(&id) { n.resolve(x) } } },
            }
        }
    }
}

fn fleet_code(_: &Vals) -> String {
    Code::new("AgentFleet")
        .raw("agents", "agents()")
        .raw("now", "clock.now()")
        .raw("open", "open()")
        .raw("on_open", "move |id| open.set(id)")
        .done()
}

// ---------- orchestrator ----------

fn sample_orch(conc: usize, stage: &str) -> Orchestration {
    let l = |w: &str, t: &str, i: &str| Lane {
        worker: w.into(),
        title: t.into(),
        instruction: i.into(),
        state: StepState::Pending,
        progress: 0.0,
        result: None,
    };
    let mut o = Orchestration {
        goal: "Compare our pricing page with four competitors and suggest changes".into(),
        lead: "Claude Opus 5.5".into(),
        lanes: vec![
            l(
                "Sonnet 5.5",
                "Linear",
                "Read linear.app/pricing; list tiers and limits",
            ),
            l(
                "Haiku 5.5",
                "Vercel",
                "Read vercel.com/pricing; note usage pricing",
            ),
            l(
                "Gemini 3.8 Flash",
                "Notion",
                "Read notion.so/pricing; seat rules",
            ),
            l(
                "GPT-6 Luna",
                "Figma",
                "Read figma.com/pricing; education plans",
            ),
        ],
        concurrency: conc,
        phase: OrchPhase::Planning,
        summary: None,
    };
    if stage != "planning" {
        o.dispatch();
        for (i, l) in o.lanes.iter_mut().enumerate() {
            if l.state == StepState::Running {
                l.progress = 0.3 + i as f32 * 0.2;
            }
        }
    }
    if stage == "done" {
        for i in 0..4 {
            o.complete(i, "4 tiers · annual −20 %");
        }
        o.phase = OrchPhase::Done;
        o.summary = Some("All four price annual plans ~20 % lower; three cap free seats. Suggest: add annual billing and a 3-seat free tier.".into());
    }
    o
}

fn orch(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveOrch { conc: v.u("conc"), stage: v.string("stage") } }
}

#[component]
fn LiveOrch(conc: usize, stage: String) -> Element {
    keyed!(o, format!("{conc}{stage}"), sample_orch(conc, &stage));
    rsx! {
        div { class: "oxd-w760",
            AgentOrchestrator {
                model: o(),
                on_concurrency: move |n: usize| { let mut w = o.write(); w.concurrency = n; w.dispatch(); },
            }
        }
    }
}

fn orch_code(_: &Vals) -> String {
    Code::new("AgentOrchestrator")
        .raw("model", "run() // Orchestration")
        .raw(
            "on_concurrency",
            "move |n| { run.write().concurrency = n; run.write().dispatch() }",
        )
        .done()
}

// ---------- tree table ----------

fn sample_table(lazy_fail: bool) -> TreeModel {
    let r = |id: &str, name: &str, size: f64, owner: &str, kids: Vec<TRow>| TRow {
        id: id.into(),
        cells: vec![name.into(), format!("{size} KB"), owner.into()],
        keys: vec![None, Some(size), None],
        children: kids,
        lazy: false,
        failed: false,
    };
    TreeModel {
        rows: vec![
            r(
                "src",
                "src",
                412.0,
                "team",
                vec![
                    r(
                        "api",
                        "api",
                        120.0,
                        "Ravi",
                        vec![
                            r("router", "router.rs", 18.0, "Ravi", vec![]),
                            r("limit", "limit.rs", 6.0, "Agent", vec![]),
                        ],
                    ),
                    r(
                        "ui",
                        "ui",
                        280.0,
                        "Lea",
                        vec![r("pricing", "pricing.tsx", 22.0, "Lea", vec![])],
                    ),
                    r("main", "main.rs", 4.0, "Ravi", vec![]),
                ],
            ),
            TRow {
                lazy: true,
                failed: lazy_fail,
                ..r("vendor", "vendor", 9200.0, "—", vec![])
            },
            r("readme", "README.md", 8.0, "Maya", vec![]),
        ],
        expanded: vec!["src".into(), "api".into()],
        selected: vec!["router".into()],
        sort: None,
    }
}

fn table(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveTable { select: v.b("select"), fail: v.b("fail"), capped: v.b("capped") } }
}

#[component]
fn LiveTable(select: bool, fail: bool, capped: bool) -> Element {
    keyed!(t, format!("{fail}"), sample_table(fail));
    rsx! {
        div { class: "oxd-w560",
            TreeTable {
                model: t(),
                columns: vec!["Name".to_string(), "Size".to_string(), "Owner".to_string()],
                selectable: select,
                max_height: if capped { 220 } else { 0 },
                on_toggle: move |id: String| t.write().toggle_open(&id),
                on_select: move |id: String| t.write().toggle_select(&id),
                on_sort: move |c: usize| t.write().cycle_sort(c),
                on_retry: move |id: String| { let mut w = t.write(); if let Some(r) = w.rows.iter_mut().find(|r| r.id == id) { r.failed = false; } },
            }
        }
    }
}

fn table_code(v: &Vals) -> String {
    Code::new("TreeTable")
        .raw("model", "table()")
        .raw(
            "columns",
            "vec![\"Name\".into(), \"Size\".into(), \"Owner\".into()]",
        )
        .b("selectable", v.b("select"))
        .raw("on_select", "move |id| table.write().toggle_select(&id)")
        .raw("on_sort", "move |c| table.write().cycle_sort(c)")
        .done()
}

// ---------- activity feed / timeline ----------

#[allow(clippy::too_many_arguments)]
fn ev(
    id: &str,
    k: EventKind,
    actor: &str,
    action: &str,
    target: &str,
    day: u32,
    time: &str,
    detail: Option<&str>,
) -> FeedEvent {
    FeedEvent {
        id: id.into(),
        kind: k,
        actor: actor.into(),
        action: action.into(),
        target: target.into(),
        day,
        time: time.into(),
        unread: day == 0,
        detail: detail.map(Into::into),
    }
}

fn sample_feed(incoming: bool) -> Feed {
    Feed {
        events: vec![
            ev(
                "1",
                EventKind::Agent,
                "Claude",
                "opened",
                "#491 Rate limits for the public API",
                0,
                "14:05",
                Some("5 files · +212 −18 · all checks passed"),
            ),
            ev(
                "2",
                EventKind::Comment,
                "Maya",
                "commented on",
                "#488 Pricing copy",
                0,
                "13:40",
                Some("“Can we keep the 3-seat free tier?”"),
            ),
            ev(
                "3",
                EventKind::Deploy,
                "Vercel",
                "deployed",
                "northwind-web to production",
                0,
                "11:12",
                None,
            ),
            ev(
                "4",
                EventKind::Code,
                "Ravi",
                "merged",
                "#482 Fix login loop",
                1,
                "17:58",
                Some("Squash merged into main"),
            ),
            ev(
                "5",
                EventKind::Alert,
                "Sentry",
                "reported",
                "TypeError in checkout (12 users)",
                1,
                "09:31",
                Some("First seen 09:31 · 44 events"),
            ),
            ev(
                "6",
                EventKind::Agent,
                "Codex",
                "finished",
                "Nightly dependency bumps",
                3,
                "02:00",
                None,
            ),
        ],
        incoming: if incoming {
            vec![ev(
                "0",
                EventKind::Code,
                "Lea",
                "pushed",
                "3 commits to feat/usage-page",
                0,
                "14:12",
                None,
            )]
        } else {
            vec![]
        },
        filter: None,
        range: 7,
        expanded: vec!["1".into()],
    }
}

fn feed(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveFeed { variant: v.string("variant"), incoming: v.b("incoming"), range: v.u("range") } }
}

#[component]
fn LiveFeed(variant: String, incoming: bool, range: usize) -> Element {
    keyed!(f, format!("{incoming}{range}"), {
        let mut x = sample_feed(incoming);
        x.range = range as u32;
        x
    });
    rsx! {
        div { class: "oxd-w520",
            ActivityFeed {
                feed: f(),
                variant: variant.clone(),
                title: if variant == "timeline" { "Timeline".to_string() } else { "Activity".to_string() },
                on_filter: move |k| f.write().filter = k,
                on_toggle: move |id: String| f.write().toggle(&id),
                on_show_new: move |_| f.write().show_new(),
                on_read_all: move |_| f.write().mark_all_read(),
            }
        }
    }
}

fn feed_code(v: &Vals) -> String {
    Code::new("ActivityFeed")
        .raw("feed", "feed()")
        .s("variant", v.s("variant"), "feed")
        .raw("on_show_new", "move |_| feed.write().show_new()")
        .raw("on_read_all", "move |_| feed.write().mark_all_read()")
        .done()
}

// ---------- countdown, counter ----------

fn countdown(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveCountdown { secs: v.u("secs") as u64, variant: v.string("variant"), days: v.b("days"), now } }
}

#[component]
fn LiveCountdown(secs: u64, variant: String, days: bool, now: ClockMs) -> Element {
    keyed!(target, format!("{secs}"), ClockMs(now.0 + secs * 1000));
    rsx! {
        Countdown { model: CountdownModel { target: target() }, now, variant, days, label: "Keynote starts in".to_string() }
    }
}

fn countdown_code(v: &Vals) -> String {
    Code::new("Countdown")
        .raw("model", "Countdown { target }")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "large")
        .b("days", !v.b("days"))
        .done()
        .replace("days: true", "days: false")
}

fn counter(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveCounter { value: v.f("value"), prefix: v.string("prefix"), decimals: v.u("decimals"), now } }
}

#[component]
fn LiveCounter(value: f64, prefix: String, decimals: usize, now: ClockMs) -> Element {
    let mut prev = use_signal(|| value);
    let mut cur = use_signal(|| value);
    let mut at = use_signal(|| now.0);
    if cur() != value {
        prev.set(cur());
        cur.set(value);
        at.set(now.0);
    }
    let t = ((now.0.saturating_sub(at())) as f32 / 600.0).min(1.0);
    let (p, s) = match prefix.as_str() {
        "$" => ("$".to_string(), String::new()),
        "%" => (String::new(), "%".to_string()),
        _ => (String::new(), " tokens".to_string()),
    };
    rsx! {
        AnimatedCounter { value, from: Some(prev()), t, label: "Run cost".to_string(), prefix: p, suffix: s, decimals }
    }
}

fn counter_code(v: &Vals) -> String {
    Code::new("AnimatedCounter")
        .raw("value", v.s("value"))
        .raw("from", "Some(previous)")
        .raw("t", "roll.progress()")
        .raw("decimals", v.s("decimals"))
        .done()
}

// ---------- agent status, agent run ----------

fn status(v: &Vals, now: ClockMs) -> Element {
    let act = match v.s("state") {
        "thinking" => Some("Planning the migration"),
        "acting" => Some("Running cargo test -p auth"),
        "waiting" => Some("Approve pushing to main?"),
        "error" => Some("Rate limited by GitHub"),
        "done" => Some("Opened #491"),
        _ => None,
    };
    rsx! {
        div { class: "oxp-demo-col",
            AgentStatus { name: "Claude".to_string(), state: v.string("state"), activity: act.map(String::from), started: ago(now, 83_000), now, variant: v.string("variant") }
        }
    }
}

fn status_code(v: &Vals) -> String {
    Code::new("AgentStatus")
        .raw("name", "\"Claude\".into()")
        .raw("state", format!("{:?}.into()", v.s("state")))
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "pill")
        .done()
}

fn run(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveRun { stage: v.string("stage"), now } }
}

#[component]
fn LiveRun(stage: String, now: ClockMs) -> Element {
    keyed!(st, stage.clone(), stage.clone());
    let s = st();
    let step = |t: &str, ms: u64| TrailStep {
        took_ms: Some(ms),
        ..TrailStep::new(t, StepState::Done)
    };
    let mut steps = vec![
        step("Read src/api/router.rs", 3_000),
        step("Searched for “middleware”", 2_000),
        step("Planned a token bucket per API key", 9_000),
    ];
    let change = (s == "review").then(|| {
        (
            "src/api/limit.rs".to_string(),
            vec![
                "+pub struct Bucket { tokens: f64, last: Instant }".to_string(),
                "+impl Bucket {".to_string(),
                "+    pub fn take(&mut self, now: Instant) -> bool {".to_string(),
                "  // …".to_string(),
                "-    Ok(next.run(req).await)".to_string(),
                "+    if !bucket.take(now) { return Err(too_many(retry)) }".to_string(),
            ],
        )
    });
    match s.as_str() {
        "running" => steps.push(TrailStep {
            status: Some("Writing limit.rs".into()),
            started: ago(now, 6_000),
            ..TrailStep::new("Implement the limiter", StepState::Running)
        }),
        "review" => steps.push(TrailStep::new(
            "Waiting for your review",
            StepState::Waiting,
        )),
        _ => {
            steps.push(step("Implemented the limiter", 21_000));
            steps.push(step("Ran 14 tests", 8_000));
        }
    }
    rsx! {
        div { class: "oxd-w560",
            AgentRun {
                task: "Add rate limiting to the public API".to_string(),
                model: "Claude Sonnet 5.5".to_string(),
                steps,
                change,
                result: (s == "done").then(|| "14 tests passed · 2 files changed".to_string()),
                now,
                on_approve: move |_| st.set("done".into()),
                on_reject: move |_| st.set("running".into()),
            }
        }
    }
}

fn run_code(_: &Vals) -> String {
    Code::new("AgentRun")
        .raw("task", "task.clone()")
        .raw("model", "\"Claude Sonnet 5.5\".into()")
        .raw("steps", "run.steps()")
        .raw("change", "run.pending_change()")
        .raw("on_approve", "move |_| run.approve()")
        .raw("on_reject", "move |note| run.revise(note)")
        .done()
}

/// The plan and run demos added for plan #28.
pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel(
                    "phase",
                    "Phase",
                    &["draft", "running", "failed", "done"],
                    "draft",
                ),
                tog("cancel", "Cancel button", true),
            ],
            variants: vec![
                var("review", "Draft: rename, drag, remove, add, Run plan.", &[]),
                var(
                    "running",
                    "Running with live timers.",
                    &[("phase", "running")],
                ),
                var(
                    "failed",
                    "A step failed: Retry from here.",
                    &[("phase", "failed")],
                ),
                var("done", "Finished with Run again.", &[("phase", "done")]),
            ],
            render: plan,
            code: plan_code,
            keyboard: vec![
                (
                    "Tab",
                    "Step titles (editable), remove buttons, Add, Cancel, Run plan",
                ),
                ("⌥↑ / ⌥↓", "Move the focused step"),
                ("Enter", "Commit a rename"),
            ],
            a11y: vec![
                "An ordered list named by the goal; each title is a labelled text input in draft.",
                "Remove buttons are named “Remove step n”; failed steps show their error as text.",
            ],
            responsive: vec!["Fills its container; tool tags and timers stay on the right."],
            height: 460,
            align: Align::Start,
            live: true,
            ..mk(
                "agent-plan",
                "Agent plan",
                "tools",
                "An agent's step-by-step plan to review before it runs: rename, drag to reorder, remove or add steps, then watch each run with live timers and retry from a failed step.",
                "AgentPlan",
                "AgentPlanEl",
                "agent-plan",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "stage",
                    "Stage",
                    &["running", "waiting", "failed", "declined"],
                    "running",
                ),
                tog("sub", "Sub-steps", true),
                tog("dense", "Dense", false),
            ],
            variants: vec![
                var(
                    "running",
                    "Step 3 testing, with sub-steps and a live timer.",
                    &[],
                ),
                var(
                    "needs-you",
                    "The gated PR step needs you.",
                    &[("stage", "waiting")],
                ),
                var("failed", "The fix failed.", &[("stage", "failed")]),
                var(
                    "dense",
                    "Dense, no sub-steps.",
                    &[("dense", "true"), ("sub", "false")],
                ),
            ],
            render: trail,
            code: trail_code,
            keyboard: vec![("—", "Read-only; the host links steps where needed")],
            a11y: vec![
                "Ordered list; state is in each step's badge text and data-state.",
                "The live timer is text; shimmer is decorative.",
            ],
            responsive: vec!["Tool marks and times keep their width; titles take the rest."],
            height: 360,
            align: Align::Start,
            live: true,
            ..mk(
                "agent-trail",
                "Agent trail",
                "tools",
                "The steps an agent works through as one quiet trail: a header with the task and 3/5, done steps with a check and their time, the current one shimmering what it does with a live timer and its sub-steps, gated steps marked Approval, then Needs you.",
                "AgentTrail",
                "AgentTrailEl",
                "agent-trail",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["capsules", "list"], "capsules"),
                tog("open", "Running row open", true),
            ],
            variants: vec![
                var("capsules", "Pill rows with a ring.", &[]),
                var("list", "Hairline list.", &[("variant", "list")]),
                var("closed", "All rows closed.", &[("open", "false")]),
            ],
            render: rows,
            code: rows_code,
            keyboard: vec![
                ("Enter / Space", "Open or close a row"),
                ("Tab", "Rows, Retry"),
            ],
            a11y: vec!["Each row is a button with aria-expanded; the pill text carries the state."],
            responsive: vec!["Amount hides first when narrow (it truncates)."],
            height: 360,
            align: Align::Start,
            ..mk(
                "task-rows",
                "Task rows",
                "tools",
                "Live agent tasks as compact rows: a ring with the step for running rows, Completed and Failed pills with Retry, details that open per row, rows entering 80 ms apart. Capsules or list.",
                "TaskRows",
                "TaskRowsEl",
                "task-rows",
                "parity/run.rs",
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
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                rng("done", "Done", 0.0, 5.0, 1.0, "2"),
                tog("hidden", "Hidden", false),
            ],
            variants: vec![
                var("progress", "2 of 5 with the next one open.", &[]),
                var("fresh", "Nothing done.", &[("done", "0")]),
                var("done", "You’re all set.", &[("done", "5")]),
                var("hidden", "Folded to its header.", &[("hidden", "true")]),
            ],
            render: check,
            code: check_code,
            keyboard: vec![
                ("Tab", "Checkboxes, titles, actions, Hide"),
                ("Enter / Space", "Toggle, open, run the action"),
            ],
            a11y: vec![
                "Checkboxes are role=checkbox buttons named by the item.",
                "Hide/Show has aria-expanded.",
            ],
            responsive: vec!["Max 480 px."],
            height: 420,
            align: Align::Start,
            ..mk(
                "onboarding-checklist",
                "Onboarding checklist",
                "tools",
                "Guide a new workspace through setup: a ring and count, one open step with its action, checks that draw in, Hide / Show, and a done title once everything is complete.",
                "OnboardingChecklist",
                "ChecklistEl",
                "onboarding-checklist",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                tog("done", "Blockers cleared", false),
                tog("fail", "Last release failed", false),
            ],
            variants: vec![
                var("blocked", "One blocker left.", &[]),
                var("ready", "Ready to ship.", &[("done", "true")]),
                var(
                    "failed",
                    "The release failed.",
                    &[("done", "true"), ("fail", "true")],
                ),
            ],
            render: ready,
            code: ready_code,
            keyboard: vec![("Tab", "Checkboxes, Release")],
            a11y: vec![
                "Progress bar with aria-valuenow; the failure is role=alert.",
                "Release is disabled while blockers remain and aria-busy while it runs.",
            ],
            responsive: vec!["Owners stay right; titles wrap."],
            height: 420,
            align: Align::Start,
            live: true,
            ..mk(
                "release-readiness",
                "Release readiness",
                "tools",
                "A launch checklist that keeps prerequisites, progress and the final action together: grouped items with owners, blockers called out, a release button that waits for them, a spinner while it runs and an inline alert if it fails.",
                "ReleaseReadiness",
                "ReadinessEl",
                "release-readiness",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                rng("deadline", "Deadline (working days)", 4.0, 12.0, 0.5, "8"),
                tog("stream", "Stream tasks in", false),
            ],
            variants: vec![
                var("fits", "8 days: fits.", &[]),
                var("late", "5 days: misses.", &[("deadline", "5")]),
                var("stream", "Tasks stream in.", &[("stream", "true")]),
            ],
            render: breakdown,
            code: breakdown_code,
            keyboard: vec![("Tab", "Regenerate, move up / down")],
            a11y: vec!["The verdict is a status region; move buttons are named."],
            responsive: vec!["The Gantt strip scales to the row."],
            height: 460,
            align: Align::Start,
            live: true,
            ..mk(
                "task-breakdown",
                "Task breakdown",
                "tools",
                "One goal broken into tasks with estimates, owners and dependencies; a schedule strip per task and a verdict that says whether the deadline holds. Reorder and regenerate.",
                "TaskBreakdown",
                "BreakdownEl",
                "task-breakdown",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel("problem", "Problem", &["waiting", "failed"], "waiting"),
                tog("fold", "Fold the research branch", false),
            ],
            variants: vec![
                var("needs-you", "The reviewer needs you.", &[]),
                var(
                    "failed",
                    "The reviewer failed: Retry / Dismiss.",
                    &[("problem", "failed")],
                ),
                var("folded", "A folded branch.", &[("fold", "true")]),
            ],
            render: tree,
            code: tree_code,
            keyboard: vec![("Tab", "Fold buttons, Approve / Reject, Retry / Dismiss")],
            a11y: vec![
                "role=tree with treeitems carrying aria-level and aria-expanded.",
                "The needs-you window is a dialog named “<agent> needs you”.",
            ],
            responsive: vec!["22 px indent per level; costs and times stay right."],
            height: 520,
            align: Align::Start,
            live: true,
            ..mk(
                "agent-tree",
                "Agent tree",
                "tools",
                "Who started whom: an orchestrator and its sub-agents as a live tree with status, cost and time on every card, costs rolling up each branch, branches that fold, and Approve / Reject or Retry / Dismiss for agents that need you or failed.",
                "AgentTree",
                "AgentTreeEl",
                "agent-tree",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![sel("open", "Open", &["lead", "support", "none"], "lead")],
            variants: vec![
                var("open", "Lead's tree open.", &[]),
                var("support", "Support open.", &[("open", "support")]),
                var("folded", "All folded.", &[("open", "none")]),
            ],
            render: fleet,
            code: fleet_code,
            keyboard: vec![("Enter / Space", "Open or fold an agent's tree")],
            a11y: vec!["Agent cards are toggle buttons (aria-pressed)."],
            responsive: vec!["Cards stack above the tree below 560 px (container query)."],
            height: 560,
            align: Align::Start,
            live: true,
            ..mk(
                "agent-fleet",
                "Agent fleet",
                "tools",
                "Several agents side by side: each card sums up its sub-agents (running, need you, done, failed) and its branch cost; a click opens its tree to the right while the last one folds back.",
                "AgentFleet",
                "FleetEl",
                "agent-fleet",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                rng("conc", "Concurrency", 1.0, 4.0, 1.0, "2"),
                sel(
                    "stage",
                    "Stage",
                    &["planning", "working", "done"],
                    "working",
                ),
            ],
            variants: vec![
                var("working", "Two lanes at a time.", &[]),
                var("wide", "All four in parallel.", &[("conc", "4")]),
                var("merged", "Merged summary.", &[("stage", "done")]),
                var("planning", "Lead still planning.", &[("stage", "planning")]),
            ],
            render: orch,
            code: orch_code,
            keyboard: vec![("Tab", "Concurrency radios")],
            a11y: vec!["Concurrency is a radiogroup; lane states are text."],
            responsive: vec!["Lanes wrap at 150 px minimum."],
            height: 420,
            align: Align::Start,
            ..mk(
                "agent-orchestrator",
                "Agent orchestrator",
                "tools",
                "A lead agent splits a goal into a plan, fans the subtasks out to parallel worker lanes under a 1–4 concurrency limit and merges what comes back, with each lane's model, progress and result in view.",
                "AgentOrchestrator",
                "OrchestratorEl",
                "agent-orchestrator",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                tog("select", "Selectable", true),
                tog("fail", "Lazy load fails", false),
                tog("capped", "Max height", false),
            ],
            variants: vec![
                var("select", "Tri-state selection.", &[]),
                var("plain", "Read-only.", &[("select", "false")]),
                var("retry", "Lazy children failed.", &[("fail", "true")]),
            ],
            render: table,
            code: table_code,
            keyboard: vec![("Tab", "Header sort buttons, row toggles, checkboxes")],
            a11y: vec![
                "role=treegrid; rows carry aria-level and aria-selected; checkboxes report mixed.",
                "Headers carry aria-sort.",
            ],
            responsive: vec![
                "Scrolls inside its frame; the header stays pinned with a max height.",
            ],
            height: 360,
            align: Align::Start,
            ..mk(
                "tree-table",
                "Tree table",
                "tools",
                "A table with nested rows, tri-state selection (checking a parent checks everything under it), sort that keeps the hierarchy, lazy children with a retry row, and a pinned header.",
                "TreeTable",
                "TreeTableEl",
                "tree-table",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["feed", "timeline"], "feed"),
                tog("incoming", "New updates waiting", true),
                sel("range", "Range (days)", &["1", "7", "30"], "7"),
            ],
            variants: vec![
                var("feed", "Filter, unread dots, new updates.", &[]),
                var(
                    "timeline",
                    "Quiet timeline.",
                    &[("variant", "timeline"), ("incoming", "false")],
                ),
                var("today", "Today only.", &[("range", "1")]),
            ],
            render: feed,
            code: feed_code,
            keyboard: vec![
                ("Tab", "Filter, new updates, events"),
                ("Enter / Space", "Expand an event (marks it read)"),
            ],
            a11y: vec![
                "Day labels are headings; events are buttons with aria-expanded.",
                "The filter is a radiogroup.",
            ],
            responsive: vec!["Targets truncate; times stay right."],
            height: 520,
            align: Align::Start,
            ..mk(
                "activity-feed",
                "Activity feed & timeline",
                "tools",
                "Events grouped by day, newest first, on a quiet avatar line: a type filter, new updates held behind a button so nothing jumps, expandable details, unread dots and Mark all read. The timeline variant drops the controls.",
                "ActivityFeed",
                "ActivityFeedEl",
                "activity-feed",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel("secs", "Starts in", &["5", "3725", "200000"], "200000"),
                sel("variant", "Variant", &["large", "compact"], "large"),
                tog("days", "Show days", true),
            ],
            variants: vec![
                var("large", "Days to seconds.", &[]),
                var("compact", "One line.", &[("variant", "compact")]),
                var("live", "Reaches zero, then Live now.", &[("secs", "5")]),
                var("hours", "Hours absorb days.", &[("days", "false")]),
            ],
            render: countdown,
            code: countdown_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec!["role=timer with an accessible name (“Keynote starts in”)."],
            responsive: vec!["Large digits wrap by unit; compact fits table cells."],
            height: 200,
            live: true,
            ..mk(
                "countdown",
                "Countdown",
                "data-display",
                "A launch countdown with digits that roll per unit, a compact one-line variant, hours absorbing days when days are hidden, and a live state with a pulsing dot at zero.",
                "Countdown",
                "CountdownEl",
                "countdown",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel("value", "Value", &["0.46", "12.05", "1284.5"], "1284.5"),
                sel("prefix", "Unit", &["$", "%", "tokens"], "$"),
                sel("decimals", "Decimals", &["0", "2"], "2"),
            ],
            variants: vec![
                var("money", "$1,284.50.", &[]),
                var(
                    "percent",
                    "Percent, no decimals.",
                    &[("prefix", "%"), ("value", "12.05"), ("decimals", "0")],
                ),
                var(
                    "tokens",
                    "Tokens.",
                    &[("prefix", "tokens"), ("decimals", "0")],
                ),
            ],
            render: counter,
            code: counter_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec![
                "The whole value is the accessible name (role=img); rolling digits are aria-hidden.",
            ],
            responsive: vec!["Font size is a prop."],
            height: 200,
            live: true,
            ..mk(
                "animated-counter",
                "Animated counter",
                "data-display",
                "A total that moves clearly: a label, prefix and suffix, fixed decimals and thousands separators, each digit column rolling the direction the number moved with a slight stagger.",
                "AnimatedCounter",
                "CounterEl",
                "animated-counter",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "state",
                    "State",
                    &["idle", "thinking", "acting", "waiting", "error", "done"],
                    "acting",
                ),
                sel("variant", "Variant", &["pill", "card", "inline"], "pill"),
            ],
            variants: vec![
                var("acting", "Working with a live timer.", &[]),
                var("waiting", "Needs you.", &[("state", "waiting")]),
                var(
                    "card",
                    "Card.",
                    &[("variant", "card"), ("state", "thinking")],
                ),
                var("error", "Error.", &[("state", "error")]),
            ],
            render: status,
            code: status_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec!["role=status, polite: state changes are announced once."],
            responsive: vec!["Activity truncates first."],
            height: 180,
            live: true,
            ..mk(
                "agent-status",
                "Agent status",
                "tools",
                "Who is working and how: a state dot on the agent's avatar (idle, thinking, working, needs you, error, done), what it does right now shimmering, and for how long.",
                "AgentStatus",
                "AgentStatusEl",
                "agent-status",
                "parity/run.rs",
            )
        },
        Demo {
            controls: vec![sel(
                "stage",
                "Stage",
                &["running", "review", "done"],
                "review",
            )],
            variants: vec![
                var("review", "Stops on a diff for review.", &[]),
                var("running", "Steps streaming.", &[("stage", "running")]),
                var(
                    "done",
                    "Result with Open pull request.",
                    &[("stage", "done")],
                ),
            ],
            render: run,
            code: run_code,
            keyboard: vec![("Tab", "Request changes, Approve & write, Open pull request")],
            a11y: vec!["The change is a labelled region; Request changes opens a note field."],
            responsive: vec!["Diff lines scroll inside their frame."],
            height: 520,
            align: Align::Start,
            live: true,
            ..mk(
                "agent-run",
                "Agent run",
                "tools",
                "One card that follows a coding agent through a task: steps stream in with the live one shimmering, the agent stops on a proposed change for you to approve or send back with a note, then the result with Open pull request.",
                "AgentRun",
                "AgentRunEl",
                "agent-run",
                "parity/run.rs",
            )
        },
    ]
}
