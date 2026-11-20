//! Checkpoints: lanes, side list, inline marker, restore toast, and the C1a variants (branch tree,
//! film strip, undo ladder, restore preview sheet). The live demos keep a real `CheckpointRun`, so
//! Rewind adds an attempt and moves the head.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    checkpoint::{Checkpoint, CheckpointKind, CheckpointRun, RestoreAction, RestoreFlow},
};
use oxyn_dioxus::{
    CheckpointFilmStrip, CheckpointLanes, CheckpointList, CheckpointMarker, CheckpointTree,
    RailTone, RestorePreviewSheet, RestoreToast, UndoLadder,
};

use super::{Code, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

fn iso_all(cps: &[Checkpoint]) -> Vec<String> {
    cps.iter().map(data::iso_of).collect()
}

/// What the agent does first after a rewind in the demo (the pool cycles).
const NEXT_STEPS: [&str; 4] = [
    "Switched to Stripe Elements",
    "Simplified the checkout flow",
    "Rewrote the failing tests",
    "Tried a smaller change",
];

/// The app each of [`NEXT_STEPS`] is done through.
const NEXT_TOOLS: [&str; 4] = [
    "mcp__stripe__create_intent",
    "mcp__stripe__update_session",
    "mcp__github__push_files",
    "mcp__github__push_files",
];

/// Lanes on a live run: Rewind (button, Enter or Space) adds an attempt in a new lane and moves
/// the head. With `autoplay` the run plays itself as in the affectUI reference: checkpoints
/// arrive one by one, the tests fail, the agent rewinds to the pricing table and attempt two
/// bends out and runs to the pull request; then it holds and starts over. Any Rewind press
/// takes over; with reduced motion it shows the finished run and does not play.
#[component]
fn LanesLive(
    cps: Vec<Checkpoint>,
    head: Option<String>,
    iso: bool,
    tools: bool,
    costs: bool,
    credits: bool,
    max_rows: usize,
    neutral: bool,
    autoplay: bool,
    empty: bool,
) -> Element {
    let full = cps.clone();
    // Autoplay starts from the first checkpoint, so the rows that arrive later draw in at once.
    let mut run = use_signal(move || {
        if autoplay && cfg!(target_arch = "wasm32") {
            CheckpointRun::new(cps.into_iter().take(1).collect(), None)
        } else {
            CheckpointRun::new(cps, head)
        }
    });
    let mut playing = use_signal(move || autoplay);
    let mut rewinds = use_signal(|| 0usize);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (&full, autoplay);
    #[cfg(target_arch = "wasm32")]
    use_future(move || {
        let full = full.clone();
        async move {
            use oxyn_core::checkpoint::CheckpointStatus;
            if !autoplay {
                return;
            }
            let reduced = web_sys::window()
                .and_then(|w| {
                    w.match_media("(prefers-reduced-motion: reduce)")
                        .ok()
                        .flatten()
                })
                .is_some_and(|m| m.matches());
            if reduced {
                run.set(CheckpointRun::new(full, None));
                return;
            }
            let mut k = 1usize;
            loop {
                if !*playing.peek() {
                    return;
                }
                let mut v: Vec<Checkpoint> = full.iter().take(k).cloned().collect();
                let failed = v
                    .last()
                    .is_some_and(|c| c.status == CheckpointStatus::Failed);
                if let Some(last) = v.last_mut()
                    && !failed
                    && k < full.len()
                {
                    last.status = CheckpointStatus::Running;
                }
                run.set(CheckpointRun::new(v, None));
                let wait = if k >= full.len() {
                    3500
                } else if failed {
                    1600
                } else {
                    1100
                };
                gloo_timers::future::TimeoutFuture::new(wait).await;
                k = if k >= full.len() { 1 } else { k + 1 };
            }
        }
    });
    let r = run.read().clone();
    let cps_now = if empty { vec![] } else { r.checkpoints.clone() };
    let times = if iso { iso_all(&cps_now) } else { vec![] };
    rsx! {
        CheckpointLanes { checkpoints: cps_now, head: r.head.clone(), times, max_rows,
            task: Some("Ship the pricing page".to_string()), tool_icons: tools, step_costs: costs,
            tone: if neutral { RailTone::Neutral } else { RailTone::Live },
            format: if credits { Some(Callback::new(|c: u32| format!("{} cr", c * 10))) } else { None },
            on_rewind: move |id: String| {
                playing.set(false);
                let i = run.peek().checkpoints.iter().position(|c| c.id == id);
                if let Some(i) = i {
                    let n = *rewinds.peek();
                    rewinds.set(n + 1);
                    let mut w = run.write();
                    w.rewind_with(i, NEXT_STEPS[n % NEXT_STEPS.len()]);
                    // The demo's next step is done through an app, so its tile shows the mark.
                    if let Some(c) = w.checkpoints.last_mut() {
                        c.tool = Some(NEXT_TOOLS[n % NEXT_TOOLS.len()].to_string());
                    }
                }
            },
        }
    }
}

fn lanes(v: &Vals, _: ClockMs) -> Element {
    let mut cps = data::lane_script();
    if !v.b("rewound") {
        cps.truncate(6);
    }
    let head = v.opt("head").filter(|h| cps.iter().any(|c| &c.id == h));
    // The stage remounts on every control change, so autoplay restarts from the first checkpoint.
    let auto = v.b("autoplay");
    rsx! {
        div { class: "oxd-w480",
            LanesLive { cps, head, iso: v.s("time") == "iso", tools: v.b("tools"), costs: v.b("costs"), credits: v.s("currency") == "credits", max_rows: v.u("rows"), neutral: v.s("rail") == "neutral", autoplay: auto, empty: v.b("empty") }
        }
    }
}

fn lanes_code(v: &Vals) -> String {
    let mut c = Code::new("CheckpointLanes")
        .pre("let checkpoints = vec![")
        .pre("    Checkpoint::new(\"c1\", None, \"Read the brief\", 9).tool(\"mcp__slack__read_thread\"),")
        .pre("    Checkpoint::new(\"c4\", Some(\"c1\"), \"Built the pricing table\", 81).tool(\"mcp__figma__get_design\"),")
        .pre("    Checkpoint::new(\"c5\", Some(\"c4\"), \"Added Stripe checkout\", 122),")
        .pre("    Checkpoint::new(\"c6\", Some(\"c5\"), \"Tests failing – 4 of 12\", 148).with(CheckpointStatus::Failed),")
        .pre("    // a rewind: the next attempt opens a new lane from c4")
        .pre("    Checkpoint::new(\"c7\", Some(\"c4\"), \"Switched to Stripe Elements\", 186),")
        .pre("];")
        .pre("let mut run = use_signal(|| CheckpointRun::new(checkpoints, None));")
        .raw("checkpoints", "run.read().checkpoints.clone()")
        .raw("head", "run.read().head.clone()");
    if v.s("time") == "iso" {
        c = c.raw(
            "times",
            "run.read().checkpoints.iter().map(|c| iso_at(\"2026-11-14\", (15, 30), c.at)).collect()",
        );
    }
    if !v.b("tools") {
        c = c.raw("tool_icons", "false");
    }
    if v.b("costs") {
        c = c.raw("step_costs", "true");
    }
    if v.s("rows") != "14" {
        c = c.raw("max_rows", v.s("rows"));
    }
    if v.s("rail") == "neutral" {
        c = c.raw("tone", "RailTone::Neutral");
    }
    if v.b("empty") {
        c = c.raw("checkpoints", "vec![]");
    }
    if v.s("currency") == "credits" {
        c = c.raw(
            "format",
            "Some(Callback::new(|cents: u32| format!(\"{} cr\", cents * 10)))",
        );
    }
    c.raw("task", q("Ship the pricing page"))
        .raw(
            "on_rewind",
            "move |id: String| { let i = run.peek().checkpoints.iter().position(|c| c.id == id); if let Some(i) = i { run.write().rewind_with(i, \"Switched to Stripe Elements\"); } }",
        )
        .done()
}

fn list(v: &Vals, _: ClockMs) -> Element {
    let term = v.s("variant") == "terminal";
    rsx! {
        div { class: if term { "oxd-term oxd-w480" } else { "oxd-w420" },
            CheckpointList { checkpoints: data::checkpoints(), times: data::checkpoint_times(), variant: v.string("variant"), selected: Some(v.u("selected")) }
        }
    }
}

fn list_code(v: &Vals) -> String {
    Code::new("CheckpointList")
        .raw("checkpoints", "checkpoints()")
        .raw("times", "times() // \"2026-11-14 15:31:04\", same order")
        .s("variant", v.s("variant"), "side-list")
        .raw("selected", format!("Some({})", v.s("selected")))
        .raw(
            "on_restore",
            "move |(i, action): (usize, RestoreAction)| agent.restore(i, action)",
        )
        .done()
}

fn marker(v: &Vals, _: ClockMs) -> Element {
    let cps = data::checkpoints();
    let cp = match v.s("kind") {
        "chat-only" => Checkpoint {
            chat_only: true,
            ..Checkpoint::new("m", None, "Asked for a summary", 0)
        },
        "manual" => {
            let mut m = Checkpoint::new("s", None, "Before the auth refactor", 0).files(3, 42, 17);
            m.kind = CheckpointKind::Manual;
            m
        }
        _ => cps[1].clone(),
    };
    let hover = v.s("variant") == "hover-revert";
    let time = "2026-11-14 15:42".to_string();
    rsx! {
        div { class: "oxd-w520 oxd-room-menu",
            div { class: "oxd-turn", b { "You" } " refactor the auth middleware to use sessions" }
            div { class: "oxd-turn", b { "Agent" } " Edited 3 files +42 −17" }
            if hover {
                div { class: "oxd-turn oxd-hov",
                    CheckpointMarker { checkpoint: cp, time, variant: "hover-revert", revealed: v.b("revealed"), menu: v.b("menu"), blocked: v.b("blocked") }
                    span { b { "You" } " now add rate limiting" }
                }
            } else {
                CheckpointMarker { checkpoint: cp, time, revealed: v.b("revealed"), menu: v.b("menu"), blocked: v.b("blocked"), current: v.b("current") }
                div { class: "oxd-turn", b { "You" } " now add rate limiting" }
            }
        }
    }
}

fn marker_code(v: &Vals) -> String {
    Code::new("CheckpointMarker")
        .raw("checkpoint", "checkpoint.clone()")
        .raw("time", q("2026-11-14 15:42"))
        .s("variant", v.s("variant"), "inline-marker")
        .b("blocked", v.b("blocked"))
        .b("current", v.b("current"))
        .raw(
            "on_action",
            "move |a: RestoreAction| agent.restore(a) // files, conversation, both or fork",
        )
        .done()
}

fn toast(v: &Vals, now: ClockMs) -> Element {
    let flow = match v.s("flow") {
        "restoring" => RestoreFlow::Restoring {
            done: v.f("done") as u32,
            total: 7,
        },
        "partial" => RestoreFlow::Partial {
            label: "Restored 6 of 7 files · rate.rs is locked".into(),
        },
        "failed" => RestoreFlow::Failed {
            reason: "the working tree has unsaved edits".into(),
        },
        _ => RestoreFlow::Restored {
            label: "Restored 3 files to 2026-11-14 15:42".into(),
            until: ClockMs(crate::BASE.0 + 8_000),
        },
    };
    rsx! { RestoreToast { flow, now } }
}

fn toast_code(v: &Vals) -> String {
    let flow = match v.s("flow") {
        "restoring" => format!("RestoreFlow::Restoring {{ done: {}, total: 7 }}", v.s("done")),
        "partial" => {
            "RestoreFlow::Partial { label: \"Restored 6 of 7 files · rate.rs is locked\".into() }"
                .into()
        }
        "failed" => {
            "RestoreFlow::Failed { reason: \"the working tree has unsaved edits\".into() }".into()
        }
        _ => "RestoreFlow::Restored { label: \"Restored 3 files to 2026-11-14 15:42\".into(), until: undo_deadline }".into(),
    };
    Code::new("RestoreToast")
        .raw("flow", flow)
        .raw("now", "clock.now()")
        .raw("on_action", "move |_| agent.undo_restore()")
        .done()
}

/// The tree on a live run, like [`LanesLive`].
#[component]
fn TreeLive(cps: Vec<Checkpoint>, head: Option<String>, folded: Vec<String>) -> Element {
    let mut run = use_signal(move || CheckpointRun::new(cps, head));
    let r = run.read().clone();
    rsx! {
        CheckpointTree { checkpoints: r.checkpoints.clone(), head: r.head.clone(), times: iso_all(&r.checkpoints), folded,
            on_rewind: move |id: String| {
                let i = run.peek().checkpoints.iter().position(|c| c.id == id);
                if let Some(i) = i { run.write().rewind(i); }
            },
        }
    }
}

fn tree_cps(v: &Vals) -> Vec<Checkpoint> {
    let mut cps = data::checkpoints();
    if v.b("third") {
        let mut x = Checkpoint::new("c7", Some("c2"), "Tried a hosted checkout page", 236)
            .files(2, 30, 12)
            .with(oxyn_core::checkpoint::CheckpointStatus::Stopped);
        x.tool = Some("Write".into());
        cps.push(x);
    }
    cps
}

fn tree(v: &Vals, _: ClockMs) -> Element {
    let cps = tree_cps(v);
    let head = v.opt("head").filter(|h| cps.iter().any(|c| &c.id == h));
    let folded = if v.b("folded") {
        vec!["c2".to_string()]
    } else {
        vec![]
    };
    rsx! {
        div { class: "oxd-w560",
            TreeLive { cps, head, folded }
        }
    }
}

fn tree_code(v: &Vals) -> String {
    let mut c = Code::new("CheckpointTree")
        .pre("let mut run = use_signal(|| CheckpointRun::new(checkpoints(), None));")
        .raw("checkpoints", "run.read().checkpoints.clone()")
        .raw("head", "run.read().head.clone()")
        .raw(
            "times",
            "run.read().checkpoints.iter().map(|c| iso_at(\"2026-11-14\", (15, 30), c.at)).collect()",
        );
    if v.b("folded") {
        c = c.raw("folded", "vec![\"c2\".into()]");
    }
    c.raw(
        "on_rewind",
        "move |id: String| { let i = run.peek().checkpoints.iter().position(|c| c.id == id); if let Some(i) = i { run.write().rewind(i); } }",
    )
    .done()
}

fn film(v: &Vals, _: ClockMs) -> Element {
    let cps = data::checkpoints();
    let times = data::checkpoint_times();
    rsx! {
        div { class: "oxd-w640",
            CheckpointFilmStrip { checkpoints: cps, times, selected: Some(v.u("selected")), thumb_rows: v.u("thumb") }
        }
    }
}

fn film_code(v: &Vals) -> String {
    let mut c = Code::new("CheckpointFilmStrip")
        .raw("checkpoints", "checkpoints()")
        .raw("times", "times() // absolute ISO, same order")
        .raw("selected", format!("Some({})", v.s("selected")));
    if v.s("thumb") != "6" {
        c = c.raw("thumb_rows", v.s("thumb"));
    }
    c.raw("on_select", "move |i: usize| preview.show(i)")
        .raw("on_restore", "move |i: usize| sheet.open(i)")
        .done()
}

fn ladder(v: &Vals, _: ClockMs) -> Element {
    let steps = data::undo_steps(v.b("fixed"));
    rsx! {
        div { class: "oxd-w520",
            UndoLadder { steps, applied: Some(v.u("applied")), title: "Agent steps · 2026-11-14".to_string() }
        }
    }
}

fn ladder_code(v: &Vals) -> String {
    Code::new("UndoLadder")
        .pre("let steps = vec![")
        .pre("    UndoStep::new(\"Added the pricing table\", \"3 files · +120 −4\"),")
        .pre("    UndoStep::new(\"Deployed the preview build\", \"preview environment\").fixed(),")
        .pre("    UndoStep::new(\"Moved prices to cents\", \"2 files · +21 −17\"),")
        .pre("];")
        .raw("steps", "steps")
        .raw("applied", format!("Some({})", v.s("applied")))
        .raw(
            "on_change",
            "move |applied: usize| agent.set_applied(applied)",
        )
        .done()
}

fn sheet(v: &Vals, _: ClockMs) -> Element {
    let action = match v.s("action") {
        "files" => RestoreAction::Files,
        "chat" => RestoreAction::Chat,
        _ => RestoreAction::Both,
    };
    let n = v.u("files").clamp(1, 5);
    let mut files = data::preview_files();
    files.truncate(n);
    rsx! {
        div { class: "oxd-w520",
            RestorePreviewSheet { files, turns: 3, action, title: "Built the pricing table".to_string(), time: "2026-11-14 15:31:04".to_string(), confirm: v.s("step") == "confirm" }
        }
    }
}

fn sheet_code(v: &Vals) -> String {
    let a = match v.s("action") {
        "files" => "RestoreAction::Files",
        "chat" => "RestoreAction::Chat",
        _ => "RestoreAction::Both",
    };
    let mut c = Code::new("RestorePreviewSheet")
        .raw("files", "preview_files() // path, FileChange, (+, −)")
        .raw("turns", "3")
        .raw("action", a)
        .raw("title", q("Built the pricing table"))
        .raw("time", q("2026-11-14 15:31:04"));
    if v.s("step") == "confirm" {
        c = c.raw("confirm", "true");
    }
    c.raw("on_restore", "move |a: RestoreAction| agent.restore(cp, a)")
        .raw("on_cancel", "move |_| sheet.close()")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "checkpoint-lanes",
            name: "Checkpoint lanes",
            category: "checkpoints-git",
            summary: "A run as a line of checkpoints, like git for agents: rewind to any of them and the new attempt branches out in its own lane, while the one it left fades back but stays.",
            dioxus: "CheckpointLanes",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                tog("rewound", "Rewound once (two attempts)", true),
                sel("head", "Head", &["none", "c4", "c6", "c7", "c9"], "none"),
                sel("time", "Time", &["run time", "iso"], "run time"),
                tog("tools", "Tool icon per checkpoint", true),
                tog("costs", "Step cost", false),
                sel("rail", "Rail colour", &["live", "neutral"], "live"),
                tog("autoplay", "Play the run", true),
                tog("empty", "No checkpoints yet", false),
                sel(
                    "currency",
                    "Cost format",
                    &["dollars", "credits"],
                    "dollars",
                ),
                rng("rows", "Rows before scrolling", 3.0, 14.0, 1.0, "14"),
            ],
            variants: vec![
                var(
                    "rewound",
                    "Attempt two after a rewind, settled.",
                    &[("autoplay", "false")],
                ),
                var(
                    "single",
                    "One attempt, failing tests.",
                    &[("rewound", "false"), ("autoplay", "false")],
                ),
                var(
                    "iso",
                    "Absolute ISO times (the day once in the bar) and per-step costs, both opt-in.",
                    &[("time", "iso"), ("costs", "true"), ("autoplay", "false")],
                ),
                var(
                    "credits",
                    "A custom cost formatter (credits), no tool icons.",
                    &[
                        ("currency", "credits"),
                        ("tools", "false"),
                        ("costs", "true"),
                        ("autoplay", "false"),
                    ],
                ),
                var(
                    "scrolling",
                    "Four rows tall: overflow scrolls in place, and the head is scrolled into sight.",
                    &[("rows", "4"), ("autoplay", "false")],
                ),
                var(
                    "neutral",
                    "Monochrome rail.",
                    &[("rail", "neutral"), ("autoplay", "false")],
                ),
            ],
            render: lanes,
            code: lanes_code,
            keyboard: vec![
                ("Tab", "Into the list: one tab stop, on the head"),
                ("↑ / ↓", "Previous or next checkpoint"),
                ("Home / End", "First or last checkpoint"),
                (
                    "Enter / Space",
                    "Rewind to the focused checkpoint: a new attempt starts from it",
                ),
            ],
            a11y: vec![
                "An ordered list with a roving tab stop; each row is named by its title, attempt, status, place on the path and time, and the head has aria-current=step.",
                "Rewind buttons are pointer shortcuts (hidden from assistive tech); Enter on the row does the same.",
                "A new attempt updates the run total in the bar, which screen readers hear politely.",
                "Status is never colour alone: Failed, Stopped and Attempt are written badges; the head is announced as current and drawn as the larger, lighter dot.",
                "Motion: a new checkpoint is revealed as the card grows and its edge draws down; a rewind opens a new lane, slides the rows right, draws the fork out of the parent dot and turns the abandoned attempt into grey rings (about 270 ms). The top demo plays the run on a loop; with reduced motion it shows the finished run and nothing animates.",
            ],
            responsive: vec![
                "Rows hold a fixed 30 px height beside one rail, so status changes never shift the layout; on hover or focus a Rewind pill swaps in where files and time sit.",
                "Beyond max_rows, rows scroll within the card (thin pill scrollbar).",
                "Under 460 px the task, the day, file counts and costs give way; the title ellipses before the totals do; the time always stays.",
                "With opt-in ISO times that share a day, the day shows once in the bar and rows show the time of day (full ISO in the tooltip and the accessible name).",
            ],
            height: 340,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "checkpoint-list",
            name: "Checkpoint list",
            category: "checkpoints-git",
            summary: "Every checkpoint of a run in a side list grouped by date, or as the terminal picker you open with Esc Esc, both driven from the keyboard.",
            dioxus: "CheckpointList",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["side-list", "terminal"],
                    "side-list",
                ),
                rng("selected", "Selected", 0.0, 5.0, 1.0, "4"),
            ],
            variants: vec![
                var("side-list", "A side panel list.", &[]),
                var("terminal", "Terminal picker.", &[("variant", "terminal")]),
            ],
            render: list,
            code: list_code,
            keyboard: vec![
                ("↑ / ↓ · Home / End", "Move the selection"),
                (
                    "Enter",
                    "Side list: restore files and chat · terminal: show the actions",
                ),
                (
                    "1 – 4",
                    "Terminal: restore files, rewind chat, both, or fork",
                ),
                ("Esc", "Terminal: back to choosing"),
            ],
            a11y: vec![
                "The side list is grouped under ISO date headings; each row is named by title, timestamp and file count and carries a machine-readable time element.",
                "The terminal picker is a listbox with aria-activedescendant on the selected option.",
            ],
            responsive: vec![
                "side-list fits a 280–360 px side panel; terminal fits an 80-column TUI.",
            ],
            height: 320,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "checkpoint-marker",
            name: "Checkpoint marker",
            category: "checkpoints-git",
            summary: "A quiet divider between chat turns that marks a checkpoint; on hover it offers Restore files, Rewind conversation, both or Fork, and says why restore is blocked while the agent runs.",
            dioxus: "CheckpointMarker",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["inline-marker", "hover-revert"],
                    "inline-marker",
                ),
                sel(
                    "kind",
                    "Checkpoint",
                    &["auto", "chat-only", "manual"],
                    "auto",
                ),
                tog("revealed", "Revealed (hover)", true),
                tog("menu", "Menu open", true),
                tog("blocked", "Agent running (blocked)", false),
                tog("current", "You are here", false),
            ],
            variants: vec![
                var("menu", "Restore choices open.", &[]),
                var(
                    "quiet",
                    "At rest between turns.",
                    &[("revealed", "false"), ("menu", "false")],
                ),
                var(
                    "blocked",
                    "Restore blocked while the agent runs.",
                    &[("blocked", "true")],
                ),
                var(
                    "hover-revert",
                    "A revert button on the user turn.",
                    &[("variant", "hover-revert"), ("menu", "false")],
                ),
            ],
            render: marker,
            code: marker_code,
            keyboard: vec![
                ("Tab", "To the marker; focus reveals Restore"),
                ("Enter / Space", "On Restore: open or close the menu"),
                ("R · C · B · F", "In the menu: pick that action"),
                ("↑ / ↓", "Move between menu items"),
                ("Esc", "Close the menu"),
            ],
            a11y: vec![
                "The marker is named \"Checkpoint, 2026-11-14 15:42, 3 files\"; Restore has aria-haspopup and aria-expanded.",
                "Menu items are menuitems with aria-keyshortcuts; blocked items are disabled and say why.",
            ],
            responsive: vec![
                "Sits between turns at the transcript width; the menu opens below it.",
            ],
            height: 320,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "restore-toast",
            name: "Restore toast",
            category: "checkpoints-git",
            summary: "What a restore did: progress while files are written, a done toast with an undo window that runs out, a partial result that names what failed, or a failure that changed nothing.",
            dioxus: "RestoreToast",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel(
                    "flow",
                    "Flow",
                    &["restored", "restoring", "partial", "failed"],
                    "restored",
                ),
                rng("done", "Files written", 0.0, 7.0, 1.0, "4"),
            ],
            variants: vec![
                var("restored", "Undo window counting down.", &[]),
                var("restoring", "Writing files.", &[("flow", "restoring")]),
                var("partial", "One file failed.", &[("flow", "partial")]),
                var("failed", "Nothing changed.", &[("flow", "failed")]),
            ],
            render: toast,
            code: toast_code,
            keyboard: vec![("Tab", "To Undo, Retry failed or Retry")],
            a11y: vec!["role=status, so the result is announced; the action is a real button."],
            responsive: vec!["Hugs its text; place it at the bottom of the transcript."],
            height: 160,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "checkpoint-tree",
            name: "Checkpoint tree",
            category: "checkpoints-git",
            summary: "Forked checkpoints as an indented tree: every rewind opens a fork under the checkpoint it came from, folds to a count, and the path to the head stays solid.",
            dioxus: "CheckpointTree",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                tog("third", "A third attempt (stopped)", false),
                sel("head", "Head", &["none", "c4", "c5", "c6"], "none"),
                tog("folded", "Forks folded", false),
            ],
            variants: vec![
                var("open", "Two lines, the fork open.", &[]),
                var(
                    "three",
                    "Three attempts from one checkpoint.",
                    &[("third", "true")],
                ),
                var("folded", "Forks folded to a count.", &[("folded", "true")]),
            ],
            render: tree,
            code: tree_code,
            keyboard: vec![
                ("↑ / ↓ · Home / End", "Move between visible checkpoints"),
                (
                    "← / →",
                    "Fold or unfold the forks of the focused checkpoint",
                ),
                ("Enter / Space", "Rewind to the focused checkpoint"),
            ],
            a11y: vec![
                "role=tree with treeitems carrying aria-level and aria-expanded; one tab stop.",
                "Each item says its title, status, ISO time, whether it is a fork, and current or left behind.",
                "Fold buttons are pointer shortcuts; ← and → do the same from the keyboard.",
            ],
            responsive: vec![
                "Each fork indents 16 px; titles truncate before badges.",
                "Under 460 px the line counts give way.",
            ],
            height: 340,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "checkpoint-film-strip",
            name: "Checkpoint film strip",
            category: "checkpoints-git",
            summary: "A thumbnail per checkpoint on a track you can scrub: drag the playhead or use the arrow keys to step through the run, then restore the frame you stopped on.",
            dioxus: "CheckpointFilmStrip",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                rng("selected", "Frame", 0.0, 5.0, 1.0, "4"),
                rng("thumb", "Lines per thumbnail", 3.0, 8.0, 1.0, "6"),
            ],
            variants: vec![
                var("latest", "On the newest finished frame.", &[]),
                var("failed", "On the failed attempt.", &[("selected", "3")]),
                var(
                    "sparse",
                    "Simpler thumbnails.",
                    &[("thumb", "3"), ("selected", "1")],
                ),
            ],
            render: film,
            code: film_code,
            keyboard: vec![
                (
                    "← / → · Home / End",
                    "In the frames or on the scrubber: move the playhead",
                ),
                ("Enter", "In the frames: restore the frame on the playhead"),
                ("Tab", "Frames → scrubber → Restore"),
            ],
            a11y: vec![
                "Frames are a horizontal listbox; each option says its number, title, status and ISO time.",
                "The scrubber is a native range input with aria-valuetext set to the frame's title.",
                "The caption under the track is a polite live region.",
            ],
            responsive: vec![
                "Frames keep 124 px and scroll sideways with a thin pill scrollbar; the track always fits.",
                "The caption wraps onto two lines when narrow.",
            ],
            height: 300,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "undo-ladder",
            name: "Undo ladder",
            category: "checkpoints-git",
            summary: "The agent's steps as a vertical stepper you can walk back down and up again; a step that can't be taken back locks every rung under it.",
            dioxus: "UndoLadder",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                rng("applied", "Steps in effect", 0.0, 6.0, 1.0, "6"),
                tog("fixed", "Deploy can't be undone", true),
            ],
            variants: vec![
                var("all", "Everything applied.", &[]),
                var("walked-back", "Two steps undone.", &[("applied", "4")]),
                var(
                    "free",
                    "Every step reversible, walked to the bottom.",
                    &[("fixed", "false"), ("applied", "0")],
                ),
            ],
            render: ladder,
            code: ladder_code,
            keyboard: vec![
                ("↑ / ↓ · Home / End", "Move between rungs"),
                (
                    "Enter / Space",
                    "Make the focused rung the newest step in effect",
                ),
                ("⌘Z / Ctrl+Z", "Undo one step"),
                ("⇧⌘Z / Ctrl+Y", "Redo one step"),
            ],
            a11y: vec![
                "An ordered list with one tab stop; each rung says its step, what it touched and applied, undone or locked.",
                "The newest step in effect has aria-current=step; the summary is a polite live region.",
                "Undo and Redo are real buttons with aria-keyshortcuts and disable at the ends.",
            ],
            responsive: vec![
                "Rungs grow with their text; the header buttons wrap under the summary when narrow.",
            ],
            height: 380,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "restore-preview",
            name: "Restore preview sheet",
            category: "checkpoints-git",
            summary: "Before a restore runs: the files it rewrites, brings back or removes with their line counts, the turns it rewinds, then Restore and a second, explicit confirm.",
            dioxus: "RestorePreviewSheet",
            gpui: "CheckpointsView",
            spec: "checkpoint-timeline",
            source: "checkpoint.rs",
            controls: vec![
                sel("action", "Restores", &["both", "files", "chat"], "both"),
                sel("step", "Step", &["review", "confirm"], "review"),
                rng("files", "Files", 1.0, 5.0, 1.0, "5"),
            ],
            variants: vec![
                var("review", "The diff summary.", &[]),
                var("confirm", "Asking to confirm.", &[("step", "confirm")]),
                var(
                    "files-only",
                    "Files only, two files.",
                    &[("action", "files"), ("files", "2")],
                ),
            ],
            render: sheet,
            code: sheet_code,
            keyboard: vec![
                (
                    "Enter / Space",
                    "Press the focused button (focus starts on the primary)",
                ),
                (
                    "Tab / Shift+Tab",
                    "Between Cancel and the primary; focus stays in the sheet",
                ),
                (
                    "Esc",
                    "From confirm back to the summary; from the summary, close",
                ),
            ],
            a11y: vec![
                "role=dialog with aria-modal, labelled by the checkpoint title and described by its ISO time and totals.",
                "Each file carries an M, A or D tag and +/− counts, so the change never rests on colour.",
                "The confirm note is role=alert; the done line is role=status.",
            ],
            responsive: vec![
                "Up to 520 px wide; paths truncate in the middle of the row and the file list scrolls past five.",
            ],
            height: 400,
            align: Align::Start,
            live: false,
        },
    ]
}
