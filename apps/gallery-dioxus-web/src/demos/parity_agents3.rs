//! Plan #32 (affectUI fidelity pass), family B part 8 demos: the exact Decision Tree and the
//! Agents rows after it, each with every affectUI style and state.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        delegatefid::{db_agents, db_auto_plan, db_tasks},
        dtreefid::{dt_demo, dt_extra},
        fallbackfid::{FbOutcome, fb_demo, fb_demo_all_fail},
        judgefid::{jb_demo, jb_demo_verdicts},
        parallelfid::{PaStatus, pa_at, pa_demo},
        planfid::{pv_actual_total, pv_demo},
        runningfid::{RnState, rn_demo},
    },
};
use oxyn_dioxus::{
    DecisionTree, DelegateBoard, FallbackChain, JudgeBracket, MemoryPanel, ParallelAttempts,
    PlanVsActual, RunningNow,
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

// ---------- Delegate Board ----------

fn delegate_board(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let pair = |t: &str, a: &str| (t.to_string(), a.to_string());
    let assigned: Vec<(String, String)> = match state {
        "queued" => vec![
            pair("t1", "coder"),
            pair("t2", "coder"),
            pair("t3", "coder"),
        ],
        "done" => db_auto_plan(&db_agents(), &db_tasks(), &[]),
        _ => vec![],
    };
    let suggestion = (state == "suggest").then(|| pair("t3", "ops"));
    let auto = state == "auto";
    let w = match v.s("width") {
        "narrow" => "380px",
        _ => "720px",
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            DelegateBoard { now, assigned, auto, suggestion, style: cstyle(v),
                on_assign: move |(t, a): (String, Option<String>)| msg.set(format!("onAssign({t:?}, {a:?})")),
                on_mode_change: move |m: bool| msg.set(format!("onModeChange({:?})", if m { "auto" } else { "manual" })) }
            {status(msg)}
        }
    }
}

fn delegate_board_code(_: &Vals) -> String {
    Code::new("DelegateBoard")
        .raw("agents", "Some(team.agents())")
        .raw("tasks", "Some(backlog.tasks())")
        .raw("now", "clock.now()")
        .raw("on_assign", "move |(task, agent)| team.give(task, agent)")
        .raw("on_mode_change", "move |auto| team.set_auto(auto)")
        .done()
}

// ---------- Parallel Attempts ----------

fn five() -> Vec<oxyn_core::parity::parallelfid::PaScript> {
    let mut d = pa_demo();
    let mut x = d[0].clone();
    x.end.id = "d".into();
    x.end.model = "Grok 4".into();
    x.end.brand = "grok".into();
    x.end.approach = "Mock the clock".into();
    x.end.score = Some(71);
    x.end.note = "18 of 20 runs pass".into();
    x.ms = 6900;
    let mut y = d[1].clone();
    y.end.id = "e".into();
    y.end.model = "Mistral Large".into();
    y.end.brand = "mistral".into();
    y.end.approach = "Pin the test data".into();
    y.end.score = Some(86);
    y.end.note = "20 of 20 runs pass · touches 6 files".into();
    y.ms = 7200;
    d.push(x);
    d.push(y);
    d
}

fn parallel_attempts(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("state"),
        v.s("attempts"),
        v.s("width")
    );
    let state = v.s("state");
    let scripts = if v.s("attempts") == "five" {
        five()
    } else {
        pa_demo()
    };
    let attempts = match state {
        "running" => Some(pa_at(&scripts, 2600)),
        "start" => Some(pa_at(&scripts, 0)),
        _ => None,
    };
    let picked = (state == "picked").then(|| "a".to_string());
    let live = state == "live";
    let w = match v.s("width") {
        "narrow" => "380px",
        _ => "720px",
    };
    let _ = PaStatus::Done;
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            ParallelAttempts { now, live, attempts, scripts: Some(scripts), picked, style: cstyle(v),
                on_pick: move |id: String| msg.set(format!("onPick({id:?})")) }
            {status(msg)}
        }
    }
}

fn parallel_attempts_code(_: &Vals) -> String {
    Code::new("ParallelAttempts")
        .raw("attempts", "Some(race.attempts())")
        .raw("task", "\"Steady the flaky payment test\"")
        .raw("now", "clock.now()")
        .raw("on_pick", "move |id| race.use_attempt(id)")
        .done()
}

fn parallel_attempts_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &["live", "start", "running", "done", "picked"],
                "live",
            ),
            sel("attempts", "Attempts", &["three", "five"], "three"),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "live",
                "The race plays and loops: bars grow with a shine, the step under each name changes, cost counts up; then scores land, the best is marked and used.",
                &[],
            ),
            var(
                "start",
                "Just started: every bar at a sliver, no scores, nothing to pick yet.",
                &[("state", "start")],
            ),
            var(
                "running",
                "Mid-race: one attempt has already failed where it stopped; the others are still going.",
                &[("state", "running")],
            ),
            var(
                "done",
                "Finished: the best score gets the Best tag, a green bar and a soft green wash; the failure stays dimmed.",
                &[("state", "done")],
            ),
            var(
                "own-pick",
                "Your own pick: the lower-scoring attempt is in use, the best keeps its tag and offers Use this one.",
                &[("state", "picked")],
            ),
            var(
                "best-of-5",
                "Best of 5 with more models and approaches.",
                &[("state", "done"), ("attempts", "five")],
            ),
            var(
                "narrow",
                "Below 560 px each progress bar drops to its own line and the approach is hidden.",
                &[("state", "done"), ("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: header band, rows joined in one framed list, a green edge on the one in use.",
                &[("style", "classic"), ("state", "done")],
            ),
        ],
        render: parallel_attempts,
        code: parallel_attempts_code,
        keyboard: vec![
            ("Tab", "Moves between the finished attempts' Use buttons"),
            ("Enter / Space", "Uses the focused attempt"),
        ],
        a11y: vec![
            "The rows form a radiogroup labelled by the heading; a finished attempt's Use control is a radio whose name gives the model and its score.",
            "Bars are decorative; progress, failure and score are all in text.",
            "Reduced motion drops the stagger, the shine and the glides.",
        ],
        responsive: vec![
            "Wide: tile, name and approach, bar, score and the use button in one line.",
            "Under 560 px the bar goes under the name, the approach and the task drop out.",
        ],
        height: 300,
        live: true,
        align: Align::Start,
        ..mk(
            "parallel-attempts",
            "Parallel Attempts",
            "tools",
            "One task tried several ways at once: every attempt races as a row with its model, approach and a live bar, finishes with a score, and the best one is marked and used – switch to another if you prefer.",
            "ParallelAttempts",
            "ParallelAttemptsEl",
            "parallel-attempts",
            "parity/parallelfid.rs",
        )
    }
}

// ---------- Judge Bracket ----------

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
            JudgeBracket { now, live, contenders, verdicts: if live { None } else { verdicts }, open, style: cstyle(v),
                on_pick: move |id: String| msg.set(format!("onPick({id:?})")) }
            {status(msg)}
        }
    }
}

fn judge_bracket_code(_: &Vals) -> String {
    Code::new("JudgeBracket")
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
            "judge-bracket",
            "Judge Bracket",
            "tools",
            "Answers from several models in a knockout bracket judged by another model: open any match to compare both answers, their scores and why one won; once the final is settled the champion's route turns blue.",
            "JudgeBracket",
            "JudgeBracketEl",
            "judge-bracket",
            "parity/judgefid.rs",
        )
    }
}

// ---------- Memory Panel ----------

fn memory_panel(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("style"), v.s("state"));
    let state = v.s("state");
    let filter = (state == "projects").then(|| "Projects".to_string());
    let search = (state == "search").then(|| "tokens".to_string());
    let pointed = (state == "tray").then(|| "m4".to_string());
    let forgotten = (state == "forgot").then(|| "m2".to_string());
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "display:flex;flex-direction:column;gap:8px;max-width:100%",
            MemoryPanel { now, filter, search, pointed, forgotten, undo_ms: if state == "forgot" { 600_000 } else { 5000 },
                enabled: state != "off", adding: state == "new", style: cstyle(v),
                on_change: move |m: Vec<oxyn_core::parity::memoryfid::MpMemory>| msg.set(format!("onChange({} memories)", m.len())),
                on_enabled_change: move |b: bool| msg.set(format!("onEnabledChange({b})")) }
            {status(msg)}
        }
    }
}

fn memory_panel_code(_: &Vals) -> String {
    Code::new("MemoryPanel")
        .raw("memories", "Some(store.memories())")
        .raw("now", "clock.now()")
        .raw("on_change", "move |m| store.save(m)")
        .raw("on_enabled_change", "move |on| store.set_enabled(on)")
        .done()
}

fn memory_panel_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &["all", "projects", "search", "tray", "forgot", "off", "new"],
                "all",
            ),
        ],
        variants: vec![
            var(
                "default",
                "Eight memories in three groups, one paused. Point at a row for Edit, Pause and Forget.",
                &[],
            ),
            var(
                "filtered",
                "The Projects chip picked: other groups fold away.",
                &[("state", "projects")],
            ),
            var(
                "search",
                "Search open: the chips give way to a field and the list filters as you type.",
                &[("state", "search")],
            ),
            var(
                "tray",
                "A row pointed at: light fill, the three actions, and the status line names its category.",
                &[("state", "tray")],
            ),
            var(
                "undo",
                "A memory forgotten: the counts drop and the last line offers Undo for a few seconds.",
                &[("state", "forgot")],
            ),
            var(
                "off",
                "Memory switched off: a note unfolds, the list dims, nothing is deleted.",
                &[("state", "off")],
            ),
            var(
                "new",
                "New memory: a field, a category and Save (disabled until there is text).",
                &[("state", "new")],
            ),
            var(
                "classic",
                "Classic style: header and status bands, squarer card.",
                &[("style", "classic")],
            ),
        ],
        render: memory_panel,
        code: memory_panel_code,
        keyboard: vec![
            (
                "Tab",
                "Switch, chips, search, each memory's actions, New memory",
            ),
            ("← / →", "Moves between the category chips"),
            (
                "Enter",
                "Saves an edit or a new memory (Shift+Enter: new line)",
            ),
            ("Escape", "Cancels an edit, the new memory or the search"),
        ],
        a11y: vec![
            "Use memory is a role=switch; the chips are a radiogroup with roving focus; Pause is a toggle (aria-pressed).",
            "Each row's actions also appear on keyboard focus, not only on hover.",
            "Saves, pauses, forgets and undos are read out by a polite status region.",
        ],
        responsive: vec![
            "A narrow card (320 px, shrinks with its container); the chip row scrolls sideways instead of wrapping.",
            "Past 360 px the rows scroll within the card.",
        ],
        height: 600,
        live: true,
        align: Align::Start,
        ..mk(
            "memory-panel",
            "Memory Panel",
            "tools",
            "What the assistant remembers about you, by category: filter or search it, pause a memory, edit or forget it with a short undo, add your own, or switch memory off without losing anything.",
            "MemoryPanel",
            "MemoryPanelEl",
            "memory-panel",
            "parity/memoryfid.rs",
        )
    }
}

// ---------- Running Now ----------

fn running_now(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("place"));
    let state = v.s("state");
    let side = v.s("place") == "sidebar";
    let mut tasks = rn_demo();
    if state == "paused" {
        for t in tasks.iter_mut() {
            if t.state == RnState::Running {
                t.state = RnState::Paused;
            }
        }
    }
    if state == "done" {
        for t in tasks.iter_mut() {
            t.state = RnState::Done;
            t.done = t.total();
        }
    }
    let default_open = state != "closed";
    let title = if state == "words" {
        "Agent jobs".to_string()
    } else {
        "Background tasks".to_string()
    };
    let rn = rsx! {
        RunningNow { now, tasks: Some(tasks), default_open, title, placement: if side { "right" } else { "below-end" }, style: cstyle(v),
            on_open: move |id: String| msg.set(format!("onOpen({id:?})")),
            on_pause: move |(id, p): (String, bool)| msg.set(format!("{}({id:?})", if p { "onPause" } else { "onResume" })),
            on_open_all: move |_| msg.set("onOpenAll()".into()) }
    };
    let bar = "display:flex;align-items:center;gap:8px;padding:6px 8px;border:1px solid var(--oxyn-color-border-default);border-radius:10px;background:var(--oxyn-color-surface-raised)";
    let mark = "width:18px;height:18px;border-radius:5px;background:var(--oxyn-color-fg-default);flex:none";
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:520px;max-width:100%;display:flex;flex-direction:column;gap:8px;min-height:300px",
            if side {
                div { style: "display:flex;flex-direction:column;align-items:flex-start;gap:10px;width:230px;padding:10px;border:1px solid var(--oxyn-color-border-default);border-radius:10px;background:var(--oxyn-color-surface-raised)",
                    span { style: "font-size:12px;font-weight:600", "Workspace" }
                    {rn}
                }
            } else {
                div { style: "{bar}",
                    span { style: "{mark}" }
                    span { style: "font-size:12px;color:var(--oxyn-color-fg-muted)", "Acme / " b { style: "color:var(--oxyn-color-fg-default)", "Agents" } }
                    span { style: "flex:1" }
                    {rn}
                }
            }
            {status(msg)}
        }
    }
}

fn running_now_code(_: &Vals) -> String {
    Code::new("RunningNow")
        .raw("tasks", "Some(jobs.background())")
        .raw("now", "clock.now()")
        .raw("on_open", "move |id| nav.open_task(id)")
        .raw(
            "on_pause",
            "move |(id, paused)| jobs.set_paused(id, paused)",
        )
        .raw("on_open_all", "move |_| nav.tasks()")
        .done()
}

fn running_now_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &["closed", "open", "paused", "done", "words"],
                "open",
            ),
            sel("place", "Placement", &["header", "sidebar"], "header"),
        ],
        variants: vec![
            var(
                "open",
                "Open over an app bar: rings fill as tasks move through their steps; Pause, Resume and Open work, and the pill's counts follow.",
                &[],
            ),
            var(
                "closed",
                "Closed: just the quiet pill – only the groups that aren't zero.",
                &[("state", "closed")],
            ),
            var(
                "paused",
                "Everything that was running paused: grey frozen rings, Resume buttons, the pill down to what needs you.",
                &[("state", "paused")],
            ),
            var(
                "done",
                "All finished: full rings with a check, the pill says so.",
                &[("state", "done")],
            ),
            var(
                "sidebar",
                "In a sidebar, opening to the right.",
                &[("place", "sidebar")],
            ),
            var(
                "words",
                "Your own words for the heading.",
                &[("state", "words")],
            ),
            var(
                "classic",
                "Classic style: a squarer pill, a header band and footer band in the popover.",
                &[("style", "classic")],
            ),
        ],
        render: running_now,
        code: running_now_code,
        keyboard: vec![
            ("Enter / Space", "Opens or closes the list from the pill"),
            (
                "Tab",
                "Each task, its Pause / Resume or Open, then Open tasks",
            ),
            ("Escape", "Closes the list"),
        ],
        a11y: vec![
            "The pill is a popup button (aria-expanded) whose name carries the counts; the list is a labelled dialog.",
            "Each row's main button names the task and its step; Pause and Resume are named per task.",
            "Finishing tasks are read out politely; reduced motion stops the ring sweep.",
        ],
        responsive: vec![
            "The pill is inline and fits any header; the 284 px list opens below and aligned to the pill's end, or to the right in a sidebar.",
        ],
        height: 360,
        live: true,
        align: Align::Start,
        ..mk(
            "running-now",
            "Running Now",
            "tools",
            "A compact pill for an app bar or side rail counting background agent work – how much is running, how much waits on you – that opens to each task's ring, current step, Pause / Resume and Open.",
            "RunningNow",
            "RunningNowEl",
            "running-now",
            "parity/runningfid.rs",
        )
    }
}

// ---------- Plan vs Actual ----------

fn plan_vs_actual(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let mut steps = pv_demo();
    if state == "ahead" {
        steps[1].actual = 24;
        steps[3].actual = 30;
    }
    let total = pv_actual_total(&steps);
    let at = match state {
        "live" => None,
        "early" => Some(30),
        "mid" | "selected" => Some(62),
        _ => Some(total),
    };
    let selected = (state == "selected").then(|| "checkout".to_string());
    let w = if v.s("width") == "narrow" {
        "380px"
    } else {
        "640px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
            PlanVsActual { now, steps: Some(steps), at, selected, style: cstyle(v),
                on_select: move |id: Option<String>| msg.set(format!("onSelect({id:?})")) }
            {status(msg)}
        }
    }
}

fn plan_vs_actual_code(_: &Vals) -> String {
    Code::new("PlanVsActual")
        .raw("steps", "Some(agent.plan_with_timings())")
        .raw("at", "Some(run.elapsed_secs())")
        .raw("task", "\"Launch the pricing page\"")
        .raw("now", "clock.now()")
        .done()
}

fn plan_vs_actual_demo(style: crate::Control) -> Demo {
    Demo {
        controls: vec![
            style,
            sel(
                "state",
                "State",
                &["live", "early", "mid", "selected", "done", "ahead"],
                "live",
            ),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "live",
                "The run plays and loops: bars grow over their dashed plans, the copy step overruns and turns amber, everything after it slides right and the ETA drifts past the planned finish.",
                &[],
            ),
            var(
                "early",
                "Thirty seconds in: the brief done early, two steps running side by side.",
                &[("state", "early")],
            ),
            var(
                "mid",
                "Mid-run: one step over plan, the now line, the planned finish and the amber gap to the ETA.",
                &[("state", "mid")],
            ),
            var(
                "selected",
                "A step picked: its row tints, its bar gets a ring, curves join it to what it waits on and what waits on it, and its cost shows.",
                &[("state", "selected")],
            ),
            var(
                "done",
                "Finished behind plan: every step checked, the final deltas, an amber ETA.",
                &[("state", "done")],
            ),
            var(
                "ahead",
                "Finished ahead: the status and ETA turn green.",
                &[("state", "ahead")],
            ),
            var(
                "narrow",
                "Under 560 px the name column narrows; the task, glyphs and axis labels drop out.",
                &[("state", "mid"), ("width", "narrow")],
            ),
            var(
                "classic",
                "Classic style: thin plain bars, a dot on the running edge, ETA before Plan.",
                &[("style", "classic"), ("state", "mid")],
            ),
        ],
        render: plan_vs_actual,
        code: plan_vs_actual_code,
        keyboard: vec![
            ("Tab", "Moves through the steps"),
            ("Enter / Space", "Selects a step to show its links and cost"),
            ("Escape", "Clears the selection"),
        ],
        a11y: vec![
            "Each step is a toggle button whose name says how long it took against plan, or when it is expected to start.",
            "The header status (behind / ahead / on plan) is a status region; the chart marks are aria-hidden.",
            "Overruns are named in text (+14s), never by colour alone; reduced motion stops the sheen, spin and glides.",
        ],
        responsive: vec![
            "Rows keep their 28 px height at every width; the bars share the space after the 150 px name column.",
            "Under 560 px the name column narrows to 96 px and the task, glyphs and axis labels drop out.",
        ],
        height: 380,
        live: true,
        align: Align::Start,
        ..mk(
            "plan-vs-actual",
            "Plan vs Actual",
            "tools",
            "How an agent's run is going against its plan: each step's real bar over a dashed planned one, overruns in amber, later steps pushed right and the expected finish drifting from the planned finish.",
            "PlanVsActual",
            "PlanVsActualEl",
            "plan-vs-actual",
            "parity/planfid.rs",
        )
    }
}

/// Family B part 8 demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
        decision_tree_demo(style()),
        parallel_attempts_demo(style()),
        judge_bracket_demo(style()),
        memory_panel_demo(style()),
        running_now_demo(style()),
        plan_vs_actual_demo(style()),
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["manual", "suggest", "queued", "auto", "done"],
                    "manual",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "Four tasks in the pile, two agents busy, one free. Drag a card onto an agent – or press Enter on it, choose with ↑ ↓ and press Enter, or press the agent's digit.",
                    &[],
                ),
                var(
                    "suggest",
                    "A suggested hand-over: the agent lights up with the time it would be busy until, and the task is marked in the pile.",
                    &[("state", "suggest")],
                ),
                var(
                    "queued",
                    "Three tasks queued on one agent: the queue folds past three into a +N chip with the rest in its tooltip; click a waiting chip to put it back.",
                    &[("state", "queued")],
                ),
                var(
                    "auto",
                    "Auto switched on: one task a second goes to whoever frees up first, until the pile is empty; queued counts show beside the names.",
                    &[("state", "auto")],
                ),
                var(
                    "done",
                    "Everything handed out: an empty pile and the money waiting in queues.",
                    &[("state", "done")],
                ),
                var(
                    "narrow",
                    "Under 560 px the pile sits above the agents.",
                    &[("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: header and footer bands, each agent framed.",
                    &[("style", "classic"), ("state", "queued")],
                ),
            ],
            render: delegate_board,
            code: delegate_board_code,
            keyboard: vec![
                (
                    "Enter",
                    "Picks up the focused task; Enter again hands it to the chosen agent",
                ),
                ("↑ / ↓", "Chooses the agent while a task is held"),
                ("1 – 9", "Hands the held task to that agent at once"),
                ("Escape", "Puts the held task back"),
            ],
            a11y: vec![
                "Pile cards are toggle buttons (aria-pressed while held); each agent is a named group whose digit button carries aria-keyshortcuts.",
                "Auto is a role=switch; each hand-over and put-back is read out by a polite status region.",
                "Dragging is optional – Enter, the arrows and the digits do the same; the +N chip is focusable and lists the rest in a tooltip.",
            ],
            responsive: vec![
                "Fills up to 720 px: pile left (170 px), agents right.",
                "Under 560 px the pile sits above the agents and the hint line drops out of the header.",
            ],
            height: 460,
            live: true,
            align: Align::Start,
            ..mk(
                "delegate-board",
                "Delegate Board",
                "tools",
                "Hand tasks to agents yourself: a pile of unassigned work beside each agent's model, rate, busy-until time and queue, with the money waiting in queues – or switch on Auto and let whoever frees up first take the next one.",
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
                "Fallback Chain",
                "tools",
                "Which model actually served a request: the fallback order drawn as a small flow with a running clock and cost, each failed try broken red with its reason, the one that answered carried through to the response.",
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
