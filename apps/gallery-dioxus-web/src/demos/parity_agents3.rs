//! Plan #32 (affectUI fidelity pass), family B part 8 demos: the exact Decision Tree and the
//! Agents rows after it, each with every affectUI style and state.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        delegatefid::{db_agents, db_tasks},
        dtreefid::{dt_demo, dt_extra},
        fallbackfid::{FbOutcome, fb_demo, fb_demo_all_fail},
    },
};
use oxyn_dioxus::{DecisionTree, DelegateBoard, FallbackChain};

use super::{Code, mk, sel, var};
use crate::{Align, Demo, Vals};

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

fn width(v: &Vals) -> &'static str {
    match v.s("width") {
        "narrow" => "380px",
        _ => "820px",
    }
}

// ---------- Decision Tree ----------

fn decision_tree(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("style"),
        v.s("state"),
        v.s("queue"),
        v.s("other"),
        v.s("width")
    );
    let state = v.s("state");
    let auto = state == "auto";
    let (branch, then, hover, other_open, sent, flag, start) = match state {
        "picked" => (Some(0), None, None, false, String::new(), None, 0),
        "outcome" => (Some(0), Some(0), None, false, String::new(), None, 0),
        "hover" => (None, None, Some(0), false, String::new(), None, 0),
        "other" => (None, None, None, true, String::new(), None, 0),
        "sent" => (
            None,
            None,
            None,
            false,
            "Revert first, then page whoever owns the build".to_string(),
            None,
            0,
        ),
        "flag" => (None, None, None, false, String::new(), Some(0), 1),
        _ => (None, None, None, false, String::new(), None, 0),
    };
    let decisions = match v.s("queue") {
        "one" => Some(vec![dt_extra()]),
        "four" => {
            let mut d = vec![dt_extra()];
            d.extend(dt_demo());
            Some(d)
        }
        _ => None,
    };
    let allow_other = v.s("other") != "off";
    let w = width(v);
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:10px",
            DecisionTree { now, auto, decisions, allow_other, branch, then, hover, other_open, sent, flag, start, style: cstyle(v),
                on_choose: move |(id, b, t): (String, usize, Option<usize>)| msg.set(format!("onChoose({id:?}, {{ index: {b}, then: {t:?} }})")),
                on_other: move |(id, t): (String, String)| msg.set(format!("onChoose({id:?}, {{ other: {t:?} }})")) }
            {status(msg)}
        }
    }
}

fn decision_tree_code(_: &Vals) -> String {
    Code::new("DecisionTree")
        .raw("decisions", "Some(agent.pending_decisions())")
        .raw("now", "clock.now()")
        .raw(
            "on_choose",
            "move |(id, branch, then)| agent.resolve(id, branch, then)",
        )
        .raw("on_other", "move |(id, text)| agent.instruct(id, text)")
        .done()
}

// ---------- Fallback Chain ----------

fn fallback_chain(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("state"),
        v.s("chain"),
        v.s("width")
    );
    let state = v.s("state");
    let live = state == "live";
    let open = (state == "details").then_some(0);
    let attempts = match v.s("chain") {
        "fail" => fb_demo_all_fail(),
        "first" => {
            let mut a = fb_demo();
            a[0].outcome = FbOutcome::Ok;
            a
        }
        "long" => {
            let mut a = fb_demo();
            a[1].outcome = FbOutcome::Timeout;
            a[2].outcome = FbOutcome::Error("400 · context too long".into());
            a
        }
        _ => fb_demo(),
    };
    let w = match v.s("width") {
        "narrow" => "380px",
        _ => "760px",
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            FallbackChain { now, live, open, attempts: Some(attempts), style: cstyle(v),
                on_replay: move |_| msg.set("onReplay()".into()),
                on_open_change: move |i: Option<usize>| msg.set(format!("onOpenChange({i:?})")) }
            {status(msg)}
        }
    }
}

fn fallback_chain_code(_: &Vals) -> String {
    Code::new("FallbackChain")
        .raw("attempts", "Some(router.attempts())")
        .raw("request", "req.summary()")
        .raw("now", "clock.now()")
        .done()
}

// ---------- Delegate Board ----------

fn delegate_board(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let assigned: Vec<(String, String)> = match state {
        "busy" | "full" => {
            let mut a = vec![
                ("t5".to_string(), "code".to_string()),
                ("t2".to_string(), "research".to_string()),
            ];
            if state == "full" {
                a.push(("t1".into(), "code".into()));
            }
            a
        }
        "done" => db_tasks()
            .iter()
            .map(|t| {
                let a = db_agents()
                    .into_iter()
                    .find(|a| a.skills.contains(&t.skill))
                    .map(|a| a.id)
                    .unwrap_or_default();
                (t.id.clone(), a)
            })
            .collect(),
        _ => vec![],
    };
    let selected = match state {
        "selected" => "t5",
        "full" => "t3",
        _ => "",
    }
    .to_string();
    let assigned_ago_ms = if state == "done" { 600_000 } else { 0 };
    let w = match v.s("width") {
        "narrow" => "380px",
        _ => "780px",
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            DelegateBoard { now, assigned, assigned_ago_ms, selected, ms_per_s: if state == "full" { 20_000 } else { 450 }, style: cstyle(v),
                on_assign: move |(t, a): (String, String)| msg.set(format!("onAssign({t:?}, {a:?})")),
                on_unassign: move |t: String| msg.set(format!("onUnassign({t:?})")) }
            {status(msg)}
        }
    }
}

fn delegate_board_code(_: &Vals) -> String {
    Code::new("DelegateBoard")
        .raw("agents", "Some(team.agents())")
        .raw("tasks", "Some(backlog.unassigned())")
        .raw("now", "clock.now()")
        .raw("on_assign", "move |(task, agent)| team.give(task, agent)")
        .done()
}

/// Family B part 8 demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
        decision_tree_demo(style()),
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["empty", "selected", "busy", "full", "done"],
                    "empty",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "Five tasks waiting, three agents idle. Pick a task, then click an agent, press its digit or drop the card on it.",
                    &[],
                ),
                var(
                    "selected",
                    "A coding task picked: the agents that can code light up and the one with the most free slots says Best fit.",
                    &[("state", "selected")],
                ),
                var(
                    "busy",
                    "Two tasks already handed over and running in their agents' columns; queued ones can be taken back.",
                    &[("state", "busy")],
                ),
                var(
                    "full",
                    "The code agent is at capacity and an infra task is picked: only the ops agent can take it, so it is the best fit.",
                    &[("state", "full")],
                ),
                var(
                    "done",
                    "Everything handed out and finished – the tray is empty.",
                    &[("state", "done")],
                ),
                var(
                    "narrow",
                    "Under 620 px the tray sits on top and the agents stack.",
                    &[("state", "selected"), ("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: grey header band, framed, tinted agent columns.",
                    &[("style", "classic"), ("state", "busy")],
                ),
            ],
            render: delegate_board,
            code: delegate_board_code,
            keyboard: vec![
                ("Tab / Enter", "Selects a task in the tray"),
                (
                    "1 – 3",
                    "Hands the selected task to that agent (when it has the skill)",
                ),
                ("Escape", "Clears the selection"),
            ],
            a11y: vec![
                "Tray tasks are toggle buttons (aria-pressed); each agent column is a named group with a Hand over button carrying aria-keyshortcuts.",
                "Agents without the skill keep a disabled button that says what they can't do.",
                "A polite status region reads out each hand-over; dragging is optional – clicks and digits do the same.",
            ],
            responsive: vec![
                "Wide: tray left, agent columns right (auto-fit, 150 px minimum).",
                "Under 620 px the tray goes on top and the columns stack.",
            ],
            height: 460,
            live: true,
            align: Align::Start,
            ..mk(
                "delegate-board",
                "Delegate Board",
                "tools",
                "Give a task to the agent you choose: unassigned work in a tray, each agent with its skills and free slots, the right ones lit when you pick a task, and the job queuing, running and finishing in that agent's column.",
                "DelegateBoard",
                "DelegateBoardEl",
                "delegate-board",
                "parity/delegatefid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("state", "State", &["live", "settled", "details"], "live"),
                sel(
                    "chain",
                    "Chain",
                    &["second", "first", "long", "fail"],
                    "second",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "live",
                    "The request leaves, a light runs to the first model, it is rate limited and the branch breaks red; the second one answers and runs on to the response.",
                    &[],
                ),
                var(
                    "settled",
                    "The finished chain: one failure, the answer, two models never asked.",
                    &[("state", "settled")],
                ),
                var(
                    "first-try",
                    "The first model answers – everything after it stays dashed and unused.",
                    &[("state", "settled"), ("chain", "first")],
                ),
                var(
                    "third-try",
                    "Two failures with different reasons before the third model answers.",
                    &[("state", "settled"), ("chain", "long")],
                ),
                var(
                    "all-failed",
                    "Every model fails: red branches, a red status and no response.",
                    &[("state", "settled"), ("chain", "fail")],
                ),
                var(
                    "details",
                    "A model clicked: how long its try took and what happened next.",
                    &[("state", "details")],
                ),
                var(
                    "narrow",
                    "Under 600 px the diagram becomes a list on a rail.",
                    &[("state", "settled"), ("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: grey header band, a stronger frame, flat nodes.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: fallback_chain,
            code: fallback_chain_code,
            keyboard: vec![
                ("Tab", "Replay, then each model in order"),
                ("Enter / Space", "Opens or closes a model's details"),
            ],
            a11y: vec![
                "Each model is a button whose name carries its place in the order, the model and what happened to it.",
                "The headline (trying, answered by, every model failed) is a status region.",
                "The links are an aria-hidden SVG; the order is also the list order.",
            ],
            responsive: vec![
                "Wide: request, links, models, links, response in one row.",
                "Under 600 px the links give way to a rail and the request, models and response stack.",
            ],
            height: 420,
            live: true,
            align: Align::Start,
            ..mk(
                "fallback-chain",
                "Fallback Chain",
                "tools",
                "Which model actually served a request: the fallback order drawn as a small flow, each failed try marked with why, the one that answered carried through to the reply.",
                "FallbackChain",
                "FallbackChainEl",
                "fallback-chain",
                "parity/fallbackfid.rs",
            )
        },
    ]
}

fn decision_tree_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &[
                    "auto", "settled", "hover", "picked", "outcome", "other", "sent", "flag",
                ],
                "auto",
            ),
            sel("queue", "Queue", &["three", "four", "one"], "three"),
            sel("other", "Other…", &["on", "off"], "on"),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "auto",
                "As affectUI ships it: the tree plays itself – the agent's pick lights up, gets picked, its follow-up answered, then the next decision grows in. Touch it to take over.",
                &[],
            ),
            var(
                "settled",
                "Waiting for you: three branches and Other…, the follow-ups dimmed until their branch is picked.",
                &[("state", "settled")],
            ),
            var(
                "hover",
                "Hover a branch: the route from the question to it turns blue and the rest fade to 40 %.",
                &[("state", "hover")],
            ),
            var(
                "picked",
                "A branch picked: the rest fade away and its follow-up wakes up – press 1 or 2.",
                &[("state", "picked")],
            ),
            var(
                "outcome",
                "Both levels picked: the path runs on to Approved · running.",
                &[("state", "outcome")],
            ),
            var(
                "other",
                "Other… turned into a small text box; Enter sends it, Shift+Enter adds a line break.",
                &[("state", "other")],
            ),
            var(
                "sent",
                "An instruction sent instead of a branch, with the agent's short answer.",
                &[("state", "sent")],
            ),
            var(
                "flagged",
                "A flagged consequence with the agent's note open (F cycles through them).",
                &[("state", "flag")],
            ),
            var(
                "no-other",
                "Without Other…: the branches are the only way forward.",
                &[("state", "settled"), ("other", "off")],
            ),
            var(
                "single",
                "One decision, two branches, no follow-ups.",
                &[("state", "settled"), ("queue", "one")],
            ),
            var(
                "narrow",
                "At 380 px the tree turns vertical: question first, then each branch on a line down the left with its consequences beneath.",
                &[("state", "settled"), ("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: framed, grey title strip, the risk tag moves into the card and each set of consequences gets its own box.",
                &[
                    ("style", "classic"),
                    ("state", "settled"),
                    ("queue", "four"),
                ],
            ),
        ],
        render: decision_tree,
        code: decision_tree_code,
        keyboard: vec![
            (
                "1 – 4",
                "Chooses that branch; one past the last number opens Other…",
            ),
            (
                "Enter",
                "In Other…: sends the instruction (Shift+Enter: new line)",
            ),
            ("→ / ← or J / K", "Next and previous decision"),
            (
                "F",
                "Cycles through the amber flags and shows why the agent raised each",
            ),
            ("Escape", "Closes Other… or a note"),
        ],
        a11y: vec![
            "Branches are real buttons named after their option (the preferred one says so in its name); aria-keyshortcuts lists the digit.",
            "Amber flags take focus; aria-describedby ties each to its role=tooltip explanation.",
            "Other… is a labelled textarea whose border darkens on focus; picks, instructions and answers go to a polite live region.",
            "Connector lines are CSS decoration and never reach assistive tech.",
        ],
        responsive: vec![
            "A container query: wide, question → branches → consequences run left to right.",
            "Under 520 px it runs top to bottom with the consequences under each branch; nothing scrolls sideways.",
        ],
        height: 560,
        live: true,
        align: Align::Start,
        ..mk(
            "decision-tree",
            "Decision Tree",
            "tools",
            "An agent at a fork shows each way forward as a branch – what it changes, which parts worry it, which one it would take – and waits for you to pick a path or tell it something else.",
            "DecisionTree",
            "DecisionTreeEl",
            "decision-tree",
            "parity/dtreefid.rs",
        )
    }
}
