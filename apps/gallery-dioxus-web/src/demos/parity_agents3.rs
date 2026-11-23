//! Plan #32 (affectUI fidelity pass), family B part 8 demos: the exact Decision Tree and the
//! Agents rows after it, each with every affectUI style and state.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
        delegatefid::{db_agents, db_auto_plan, db_tasks},
        dtreefid::{dt_demo, dt_extra},
        fallbackfid::{FbOutcome, fb_demo, fb_demo_all_fail},
        fanfid::FoStatus,
        judgefid::{jb_demo, jb_demo_verdicts},
        parallelfid::{PaStatus, pa_at, pa_demo},
        planfid::{pv_actual_total, pv_demo},
        ratefid::{RL_BASE, RlRun, rl_demo, rl_now, rl_parse, rl_spread},
        rootfid::{rc_demo, rc_len},
        runningfid::{RnState, rn_demo},
        schedfid::SrMode,
        toolmapfid::{tm_demo_agents, tm_replay},
        tracefid::tw_demo,
    },
};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    ActivityPill, AssignBoard, AttemptRace, LoadPlanner, MemoryPanel, PlanVsActual, RootCauseTree,
};
use oxyn_dioxus::{
    DecisionTree, FallbackChain, ModelKnockout, ScheduledRuns, TaskGrid, ToolReach, TraceWaterfall,
};

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
            a[0].tokens = 412;
            a[0].cost = 0.0027;
            a
        }
        "second" => {
            let mut a = fb_demo();
            a[1].outcome = FbOutcome::Ok;
            a[1].tokens = 388;
            a[1].cost = 0.0031;
            a
        }
        _ => fb_demo(),
    };
    let w = match v.s("width") {
        "narrow" => "380px",
        "medium" => "560px",
        _ => "680px",
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            FallbackChain { now, live, open, attempts: Some(attempts), style: cstyle(v),
                on_retry: move |_| msg.set("onRetry()".into()),
                on_open_change: move |i: Option<usize>| msg.set(format!("onOpenChange({i:?})")) }
            {status(msg)}
        }
    }
}

fn fallback_chain_code(_: &Vals) -> String {
    Code::new("FallbackChain")
        .raw("attempts", "Some(router.attempts())")
        .raw("request", "req.summary()")
        .raw("route", "\"router/auto\"")
        .raw("on_retry", "move |_| router.retry(req)")
        .raw("now", "clock.now()")
        .done()
}

// ---------- Model knockout ----------

fn judge_bracket(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("state"),
        v.s("size"),
        v.s("width")
    );
    let state = v.s("state");
    let four = v.s("size") == "four";
    let all = jb_demo_verdicts();
    let contenders = if four {
        let c = jb_demo();
        Some(vec![c[0].clone(), c[2].clone(), c[4].clone(), c[6].clone()])
    } else {
        None
    };
    let verdicts = if four {
        // Re-seat the semifinal and final verdicts as a four-contender bracket.
        let mut x: Vec<_> = all[4..].to_vec();
        for vd in &mut x {
            vd.round -= 1;
        }
        let k = match state {
            "judging" | "pending" => 2,
            "quarter" => 1,
            _ => 3,
        };
        x.truncate(k);
        Some(x)
    } else {
        let k = match state {
            "judging" => 4,
            "quarter" => 5,
            "pending" => 2,
            _ => 7,
        };
        Some(all[..k].to_vec())
    };
    let open = match (state, four) {
        ("quarter", false) => Some((0, 1)),
        ("pending", false) => Some((1, 1)),
        ("quarter", true) => Some((0, 0)),
        _ => None,
    };
    let live = state == "live";
    let w = match v.s("width") {
        "narrow" => "380px",
        _ => "640px",
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            ModelKnockout { now, live, contenders, verdicts: if live { None } else { verdicts }, open, style: cstyle(v),
                on_pick: move |id: String| msg.set(format!("onPick({id:?})")) }
            {status(msg)}
        }
    }
}

fn judge_bracket_code(_: &Vals) -> String {
    Code::new("ModelKnockout")
        .raw("contenders", "Some(answers.seeded())")
        .raw("verdicts", "Some(judge.verdicts())")
        .raw("judge", "\"Opus 4.1\"")
        .raw("now", "clock.now()")
        .raw("on_pick", "move |id| apply(id)")
        .done()
}

fn judge_bracket_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &["live", "judging", "quarter", "pending", "decided"],
                "live",
            ),
            sel("size", "Contenders", &["eight", "four"], "eight"),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "live",
                "Plays and loops: each match shimmers while the judge compares, the preferred answer moves on, the other fades but stays; at the end the winner's way lights up and a cup appears.",
                &[],
            ),
            var(
                "judging",
                "Quarterfinals decided, a semifinal being judged: its names shimmer and the panel says the judge is comparing.",
                &[("state", "judging")],
            ),
            var(
                "match-open",
                "A decided quarterfinal opened: both answers with their meta line, code and score, the Winner tag and the judge's reason.",
                &[("state", "quarter")],
            ),
            var(
                "not-judged",
                "A semifinal whose contenders aren't known yet: dashed empty slots, Not judged yet.",
                &[("state", "pending")],
            ),
            var(
                "decided",
                "The final decided: the winner's way in blue round after round, a cup in the final, and a Use button.",
                &[("state", "decided")],
            ),
            var(
                "four",
                "Four contenders: semifinals and a final.",
                &[("state", "decided"), ("size", "four")],
            ),
            var(
                "narrow",
                "When the rounds don't fit, each round becomes a list with one match a row and the answers stack.",
                &[("state", "decided"), ("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: header band, framed answers panel, squarer slots.",
                &[("style", "classic"), ("state", "quarter")],
            ),
        ],
        render: judge_bracket,
        code: judge_bracket_code,
        keyboard: vec![
            ("Tab", "Match by match, then the answers and Use"),
            (
                "→ / ←",
                "Forward to the next-round match, or back to the first feeder",
            ),
            ("↑ / ↓", "Previous and next match in the round"),
            ("Enter / Space", "Opens or closes a match's answers"),
            ("Escape", "Back to the match being judged"),
        ],
        a11y: vec![
            "Each match is a button named with both contenders and the result (or that it is being judged).",
            "The winner keeps full colour and its score; the loser fades and its outline turns dashed – never colour alone.",
            "Progress (round · decided/total) is a status region; reduced motion makes everything simply appear.",
        ],
        responsive: vec![
            "Rounds share the width with fixed slot heights; the elbows join each pair into the next round.",
            "Under 560 px each round is a list with one match a row and the two answers stack.",
        ],
        height: 640,
        live: true,
        align: Align::Start,
        ..mk(
            "model-knockout",
            "Model knockout",
            "tools",
            "Answers from several models in a knockout bracket judged by another model: open any match to compare both answers, their scores and why one won; once the final is settled the champion's route turns blue.",
            "ModelKnockout",
            "ModelKnockoutEl",
            "judge-bracket",
            "kit/judgefid.rs",
        )
    }
}

// ---------- Tool reach ----------

fn tool_access_map(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let mut agents = tm_demo_agents();
    if state == "replaying" {
        let _ = tm_replay(
            &mut agents[0],
            "github",
            "{ \"pull_request\": \"Pricing v3\", \"base\": \"main\" }",
            240,
        );
    }
    let agent = match state {
        "scout" | "failed" => Some("scout".to_string()),
        "scribe" => Some("scribe".to_string()),
        _ => None,
    };
    let open = match state {
        "calls" | "replaying" | "editing" => Some("github".to_string()),
        "failed" => Some("search".to_string()),
        _ => None,
    };
    let call = (state == "editing").then(|| "gh2".to_string());
    let editing = state == "editing";
    let w = if v.s("width") == "narrow" {
        "380px"
    } else {
        "720px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            ToolReach { now, agents: Some(agents), agent, open, call, editing, style: cstyle(v),
                on_agent_change: move |id: String| msg.set(format!("onAgentChange({id:?})")),
                on_replay: move |(a, t, _): (String, String, String)| msg.set(format!("onReplay({a:?}, {t:?}, …)")) }
            {status(msg)}
        }
    }
}

fn tool_access_map_code(_: &Vals) -> String {
    Code::new("ToolReach")
        .raw("tools", "Some(run.tools())")
        .raw("agents", "Some(run.agents())")
        .raw("now", "clock.now()")
        .raw(
            "on_replay",
            "move |(agent, tool, input)| run.replay(agent, tool, input)",
        )
        .done()
}

fn tool_access_map_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &[
                    "map",
                    "scout",
                    "scribe",
                    "calls",
                    "replaying",
                    "editing",
                    "failed",
                ],
                "map",
            ),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "map",
                "The builder agent in the middle: six of nine tools in reach, the shell busy with a build (its wire shimmers), the rest used once or more, and three tools it cannot touch.",
                &[],
            ),
            var(
                "other-agent",
                "Another agent on the same layout: most cards drop to No access, a query is in flight and the last web search failed.",
                &[("state", "scout")],
            ),
            var(
                "needs-you",
                "An agent waiting on a person: the amber Needs you badge on its card.",
                &[("state", "scribe")],
            ),
            var(
                "calls",
                "A tool opened: its calls newest first, the picked call's input and output, Edit input and Replay. Escape or the back arrow returns to the map.",
                &[("state", "calls")],
            ),
            var(
                "replaying",
                "A replay in flight: the new call is marked Replay, the output waits and both buttons hold until it lands.",
                &[("state", "replaying")],
            ),
            var(
                "editing",
                "Editing the input before a replay: Cmd or Ctrl + Enter runs it.",
                &[("state", "editing")],
            ),
            var(
                "failed",
                "A failed call: the error text in red in the output.",
                &[("state", "failed")],
            ),
            var(
                "narrow",
                "Under 540 px the agent sits on top and the tools fall into two columns below; the wires go.",
                &[("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: one framed card with a header band and squarer cards.",
                &[("style", "classic")],
            ),
        ],
        render: tool_access_map,
        code: tool_access_map_code,
        keyboard: vec![
            ("← / →", "Moves between the agent tabs"),
            (
                "Tab / Enter",
                "Through the tools; Enter opens a tool's calls",
            ),
            (
                "↑ / ↓",
                "Moves through the calls (the list is one tab stop)",
            ),
            ("Escape", "Back to the map"),
            ("Cmd / Ctrl + Enter", "Runs the edited input"),
        ],
        a11y: vec![
            "The agents are a tablist; each tool is a button named with its state and call count; tools out of reach are disabled.",
            "States are words (Running, Failed, Not used yet, No access), never colour alone; the counts are a status region.",
            "Reduced motion stops the wire shimmer and the slide-in.",
        ],
        responsive: vec![
            "Tool cards have one fixed size, so nothing shifts when a call starts, ends or fails.",
            "Below 540 px the agent card leads, two columns of tools follow with no wires, and the calls view stacks its list over the details.",
        ],
        height: 470,
        live: true,
        align: Align::Start,
        ..mk(
            "tool-reach",
            "Tool reach",
            "tools",
            "Which tools an agent can reach and what it did with them: the agent in the middle, every tool of the run around it with a wire where access exists, cards for used, busy, failed or unused, and each tool's calls one click away with a replay.",
            "ToolReach",
            "ToolReachEl",
            "tool-access-map",
            "kit/toolmapfid.rs",
        )
    }
}

// ---------- Scheduled Runs ----------

fn scheduled_runs(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let mode = if matches!(state, "manual" | "overbooked" | "open") {
        SrMode::Manual
    } else {
        SrMode::Auto
    };
    let agent = (state == "filtered").then(|| "openai".to_string());
    let open = match state {
        "open" => Some("apiref".to_string()),
        "booked-open" => Some("search".to_string()),
        _ => None,
    };
    let booked = if state == "overbooked" {
        vec![("sessions".to_string(), 4)]
    } else {
        vec![]
    };
    let w = if v.s("width") == "narrow" {
        "380px"
    } else {
        "640px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px;padding:14px;border-radius:16px;background:var(--oxsr-bg)",
            ScheduledRuns { mode, agent, open, booked, style: cstyle(v),
                on_book: move |(id, c): (String, usize)| msg.set(format!("onBook({id}, {c})")),
                on_mode: move |m: SrMode| msg.set(format!("onMode({})", m.id())),
                on_filter: move |a: Option<String>| msg.set(format!("onFilter({a:?})")) }
            {status(msg)}
        }
    }
}

fn scheduled_runs_code(_: &Vals) -> String {
    Code::new("ScheduledRuns")
        .raw("tasks", "Some(queue.tasks())")
        .raw("mode", "SrMode::Auto")
        .raw("on_book", "move |(id, cell)| queue.book(id, cell)")
        .done()
}

fn scheduled_runs_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &[
                    "auto",
                    "manual",
                    "filtered",
                    "open",
                    "booked-open",
                    "overbooked",
                ],
                "auto",
            ),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "auto",
                "Auto: the whole list is booked into the week, biggest tasks first, small ones into the gaps; the one-and-a-half-session task lands in two sessions as 1/2 and 2/2.",
                &[],
            ),
            var(
                "manual",
                "Manual: only the five hand-booked runs stay; the other six wait in the list, one of them needing 150 % of a session.",
                &[("state", "manual")],
            ),
            var(
                "filtered",
                "One agent picked: its pill and count show in the header and every other agent's card fades.",
                &[("state", "filtered")],
            ),
            var(
                "open",
                "A waiting task opened: its details, what it needs, and buttons for the sessions it still fits in, so booking works without a drag.",
                &[("state", "open")],
            ),
            var(
                "booked-open",
                "A booked task opened: where it runs, and a button that takes it back to the list (the board switches to Manual so it stays there).",
                &[("state", "booked-open")],
            ),
            var(
                "overbooked",
                "A hand drop that overfills a session: its bar and figure turn red and the status line names the session.",
                &[("state", "overbooked")],
            ),
            var(
                "narrow",
                "Under 560 px the list moves above the week and cards in the sessions keep only their title and share.",
                &[("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: square cards with an agent stripe, a dashed list column.",
                &[("style", "classic")],
            ),
        ],
        render: scheduled_runs,
        code: scheduled_runs_code,
        keyboard: vec![
            (
                "Tab",
                "Moves through the filter, the mode switch and the cards",
            ),
            (
                "Enter / Space",
                "Opens a card's details; inside, books it into a session or takes it back",
            ),
            ("Escape", "Closes the details"),
        ],
        a11y: vec![
            "Auto | Manual is a radio group; the agent buttons are toggles named after the agent.",
            "Each card's name gives its title, agent and share of a session; each session names how much of its limit is booked.",
            "Booking never needs a drag: the details list the sessions that still have room. Overbooking is said in the status line, not only in red.",
        ],
        responsive: vec![
            "The three days share the width after a 136 px list column.",
            "Under 560 px the list sits above the week and cards in the sessions drop their description and figures.",
        ],
        height: 520,
        live: false,
        align: Align::Start,
        ..mk(
            "scheduled-runs",
            "Scheduled runs",
            "tools",
            "Agent tasks booked into a week of sessions that each have a usage limit: auto booking fills the gaps with small tasks and cuts big ones across sessions, manual keeps what you booked by hand, and one agent can be picked out.",
            "ScheduledRuns",
            "ScheduledRunsEl",
            "scheduled-runs",
            "kit/schedfid.rs",
        )
    }
}

// ---------- Trace Waterfall ----------

fn trace_waterfall(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let end = tw_demo().end;
    let at = match state {
        "live" => None,
        "early" => Some(14_000),
        "mid" | "folded" => Some(38_000),
        _ => Some(end),
    };
    let folded = if state == "folded" {
        vec!["research".to_string(), "writer".to_string()]
    } else {
        vec![]
    };
    let open = match state {
        "details" => Some("edit".to_string()),
        "failed" => Some("tests1".to_string()),
        _ => None,
    };
    let w = if v.s("width") == "narrow" {
        "380px"
    } else {
        "640px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px;padding-bottom:40px",
            TraceWaterfall { now, at, folded, open, style: cstyle(v),
                on_open: move |id: Option<String>| msg.set(format!("onOpen({id:?})")) }
            {status(msg)}
        }
    }
}

fn trace_waterfall_code(_: &Vals) -> String {
    Code::new("TraceWaterfall")
        .raw("trace", "Some(run.trace())")
        .raw("at", "Some(run.elapsed_ms())")
        .raw("now", "clock.now()")
        .done()
}

fn trace_waterfall_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &[
                    "live", "early", "mid", "folded", "done", "details", "failed",
                ],
                "live",
            ),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "live",
                "The run replays and loops: spans slide in and grow with a sheen, the totals climb, the axis glides to the next round range, then the pill turns Done.",
                &[],
            ),
            var(
                "early",
                "Fourteen seconds in: the plan done, the researcher reading pages on a 15 s axis.",
                &[("state", "early")],
            ),
            var(
                "mid",
                "Mid-run: writer and coder side by side, a failed test run in red and the error count up.",
                &[("state", "mid")],
            ),
            var(
                "folded",
                "Finished branches folded: their calls shrink to marks on the parent bar with a +n count.",
                &[("state", "folded")],
            ),
            var(
                "done",
                "Finished: every span ended, the slowest call marked, the pill Done.",
                &[("state", "done")],
            ),
            var(
                "details",
                "A model call opened: duration, start, cost, tokens in and out, and what went in and came out.",
                &[("state", "details")],
            ),
            var(
                "failed",
                "The failed tool call opened: a red status and the error text.",
                &[("state", "failed")],
            ),
            var(
                "narrow",
                "Under 560 px each row puts its bar under the name and the totals wrap to three columns.",
                &[("state", "done"), ("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: a grey header band, sentence-case labels, roomier rows.",
                &[("style", "classic"), ("state", "mid")],
            ),
        ],
        render: trace_waterfall,
        code: trace_waterfall_code,
        keyboard: vec![
            ("Tab", "The span tree is one stop"),
            ("Up / Down, Home / End", "Move through the spans"),
            ("Right / Left", "Unfold / fold a branch"),
            ("Enter / Space", "Opens the span's details"),
            ("Escape", "Closes the details"),
        ],
        a11y: vec![
            "The spans are a tree (treeitems with levels and expanded state) with one tab stop and an active descendant.",
            "The Live / Done pill is a status region; a failed span carries a warning icon and the word Failed in its details, not only red.",
            "Reduced motion stops the sheen, the shimmer on the totals and the glides.",
        ],
        responsive: vec![
            "Wide: the span names on the left, the time axis on the right.",
            "Under 560 px each row stacks its bar under the name and the totals wrap to three columns; under 360 px the muted details drop.",
        ],
        height: 560,
        live: true,
        align: Align::Start,
        ..mk(
            "trace-waterfall",
            "Trace waterfall",
            "tools",
            "An agent run as a tree of spans on a time axis: agents, model calls and tool calls as tinted bars that grow while the run goes, the run's totals on top, finished branches folded into marks, and every span's details one click away.",
            "TraceWaterfall",
            "TraceWaterfallEl",
            "trace-waterfall",
            "kit/tracefid.rs",
        )
    }
}

// ---------- Task grid ----------

fn fanout_grid(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let at = match state {
        "live" => None,
        "early" => Some(3_000),
        "mid" | "filtered" | "pinned" => Some(11_000),
        _ => Some(40_000),
    };
    let filter = (state == "filtered").then_some(FoStatus::Running);
    let pinned = (state == "pinned").then_some(41);
    let w = if v.s("width") == "narrow" {
        "380px"
    } else {
        "620px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            TaskGrid { now, at, filter, pinned, style: cstyle(v),
                on_retry: move |ids: Vec<String>| msg.set(format!("onRetry({} tasks)", ids.len())),
                on_select: move |i: Option<usize>| msg.set(format!("onSelect({i:?})")) }
            {status(msg)}
        }
    }
}

fn fanout_grid_code(_: &Vals) -> String {
    Code::new("TaskGrid")
        .raw("tasks", "Some(job.tasks())")
        .raw("at", "Some(job.elapsed_ms())")
        .raw("workers", "12")
        .raw("now", "clock.now()")
        .done()
}

fn fanout_grid_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &["live", "early", "mid", "filtered", "pinned", "failed"],
                "live",
            ),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "live",
                "The job replays and loops: cells turn blue with a shine, then green in a wave; one retries in amber, two fail; done, cost and time left keep moving.",
                &[],
            ),
            var(
                "early",
                "Three seconds in: the first wave of twelve workers busy, everything else queued.",
                &[("state", "early")],
            ),
            var(
                "mid",
                "Mid-job: green spreading, a retry breathing amber, the time left counting down.",
                &[("state", "mid")],
            ),
            var(
                "filtered",
                "A count pressed: only running cells stay vivid, the rest fade back.",
                &[("state", "filtered")],
            ),
            var(
                "pinned",
                "A cell pinned: a ring on it and its card – worker, time, tries, cost and result.",
                &[("state", "pinned")],
            ),
            var(
                "failed",
                "Ended with two failures: a red count and Retry failed, which runs them again until the job finishes green.",
                &[("state", "failed")],
            ),
            var(
                "narrow",
                "Below 520 px the cells get smaller and done / cost / time left wrap under the title.",
                &[("state", "mid"), ("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: square-ish cells; the count buttons sit in a grey strip along the bottom of the card.",
                &[("style", "classic"), ("state", "failed")],
            ),
        ],
        render: fanout_grid,
        code: fanout_grid_code,
        keyboard: vec![
            (
                "Tab",
                "The grid is one stop, then the counts and Retry failed",
            ),
            (
                "Arrow keys, Home / End",
                "Walk the cells and show each one's card",
            ),
            ("Enter / Space", "Pin or release the card"),
            ("Escape", "Releases the card"),
        ],
        a11y: vec![
            "The grid is a single tab stop with an active cell; each cell is named by its task and status and described by its card.",
            "Each count is a pressed / not-pressed toggle; a polite status line reports progress every tenth and once at the end.",
            "Status is never colour alone: the card and the counts say it in words; reduced motion stops the wave, shine and breathing.",
        ],
        responsive: vec![
            "The cells fill the width and wrap.",
            "Below 520 px cells are 8 px and the header figures wrap onto their own line.",
        ],
        height: 420,
        live: true,
        align: Align::Start,
        ..mk(
            "task-grid",
            "Task grid",
            "tools",
            "One job split into hundreds of small tasks across a pool of workers: a cell per task tinted by status, counts that filter the grid, the cost and time left, each task's card one hover away and Retry failed.",
            "TaskGrid",
            "TaskGridEl",
            "fanout-grid",
            "kit/fanfid.rs",
        )
    }
}

/// Family B part 8 demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    #[allow(unused_mut)]
    let mut v = vec![
        decision_tree_demo(style()),
        judge_bracket_demo(style()),
        tool_access_map_demo(style()),
        scheduled_runs_demo(style()),
        trace_waterfall_demo(style()),
        fanout_grid_demo(style()),
        Demo {
            controls: vec![
                style(),
                sel("state", "State", &["live", "settled", "details"], "live"),
                sel(
                    "chain",
                    "Chain",
                    &["third", "first", "second", "fail"],
                    "third",
                ),
                sel("width", "Width", &["wide", "medium", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "live",
                    "Plays and loops: a dotted light grows along each branch while that model is tried; the first is rate limited, the second runs out of time, the third answers and runs on to the response. Clock and cost tick in the header.",
                    &[],
                ),
                var(
                    "settled",
                    "The finished run: two red breaks, the answer in green, one model never asked.",
                    &[("state", "settled")],
                ),
                var(
                    "first-try",
                    "The first model answers – everything after it stays dashed and is marked not needed.",
                    &[("state", "settled"), ("chain", "first")],
                ),
                var(
                    "second-try",
                    "One rate-limit break before the second model answers.",
                    &[("state", "settled"), ("chain", "second")],
                ),
                var(
                    "all-failed",
                    "Every model fails: red branches, a red status, no response, and a Retry button in the header.",
                    &[("state", "settled"), ("chain", "fail")],
                ),
                var(
                    "details",
                    "A model opened: provider, its tag, then model, time against its budget, cost, status code, list price and the reason.",
                    &[("state", "details")],
                ),
                var(
                    "medium",
                    "Under 600 px the provider leaves the row and lives only in the details card.",
                    &[("state", "settled"), ("width", "medium")],
                ),
                var(
                    "narrow",
                    "Under 520 px the diagram becomes a vertical tree on a dashed rail.",
                    &[("state", "settled"), ("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: header band over a grey field, flatter nodes.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: fallback_chain,
            code: fallback_chain_code,
            keyboard: vec![
                (
                    "Tab",
                    "Request, then each model in order, then the response – each shows its details card on focus",
                ),
                (
                    "Enter / Space",
                    "Pins a node's details card open or closes it",
                ),
                ("Escape", "Closes a pinned card"),
            ],
            a11y: vec![
                "Each model is a button whose name carries its place in the order, model, provider and what happened to it; its details card is a role=tooltip that opens on hover, focus or click.",
                "The header status (attempt k of n, answered, all failed) is a status region; failures are named in words, never colour alone.",
                "The links are an aria-hidden SVG; the order is also the list order. Reduced motion stops the travelling light and the drawing.",
            ],
            responsive: vec![
                "Wide: request, links, models, links, response in one row.",
                "Under 600 px the provider moves into the details card.",
                "Under 520 px the links give way to a dashed rail and request, models and response stack.",
            ],
            height: 420,
            live: true,
            align: Align::Start,
            ..mk(
                "fallback-chain",
                "Fallback chain",
                "tools",
                "Which model actually served a request: the fallback order drawn as a small flow with a running clock and cost, each failed try broken red with its reason, the one that answered carried through to the response.",
                "FallbackChain",
                "FallbackChainEl",
                "fallback-chain",
                "kit/fallbackfid.rs",
            )
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 14] = [
    "decision-tree",
    "attempt-race",
    "model-knockout",
    "memory-panel",
    "activity-pill",
    "plan-vs-actual",
    "root-cause-tree",
    "tool-reach",
    "scheduled-runs",
    "load-planner",
    "trace-waterfall",
    "task-grid",
    "assign-board",
    "fallback-chain",
];

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
                "The tree plays itself – the agent's pick lights up, gets picked, its follow-up answered, then the next decision grows in. Touch it to take over.",
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
            "Decision tree",
            "tools",
            "An agent at a fork shows each way forward as a branch – what it changes, which parts worry it, which one it would take – and waits for you to pick a path or tell it something else.",
            "DecisionTree",
            "DecisionTreeEl",
            "decision-tree",
            "kit/dtreefid.rs",
        )
    }
}

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
