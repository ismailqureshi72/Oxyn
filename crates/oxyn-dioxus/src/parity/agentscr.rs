//! Screens S3: agent ops I on the Screens 1 frame with the Fieldwork persona's sidebar. Thread (nine
//! states), Inbox (list 320 and one question at a time), Inbox Board, Tasks, Activity and Memory,
//! each with full-view states (`oxyn_core::parity::agentscr::states_for`) and real interactions.
//! Styles in `agentscr.css` (`oxa-` shared, `oxat-` Thread, `oxai-` Inbox, `oxab-` Board, `oxak-`
//! Tasks, `oxay-` Activity, `oxam-` Memory). Chrome stays monochrome; colour is for official marks,
//! file glyphs, project hues and status.

use std::rc::Rc;

use dioxus::html::geometry::PixelsVector2D;
use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::parity::agentscr::{
    ACT_TABS, AWAY, Act, BoardCard, LANES, Line, Memo, Mode, Question, ScreenState, Step, Task,
    TaskState, act_counts, act_view, activity, age, board, board_counts, dict_clock, eur,
    key_choice, memo_counts, memo_groups, memory, mmss, move_to, next_question, project, questions,
    remember, reply, ring_pct, split, states_for, task_counts, task_groups, tasks, thread,
    thread_counts, tools_words, undo_clock, undo_left, undo_words,
};
use oxyn_core::parity::screensfid::PERSON_EMAIL;

use super::efflistfid::{check_glyph, chevron};
use super::screensfid::{AppFrame, MenuButton, file_ico, ico, nico, since, skel, use_threads};
use super::workscr::{find, tabs};
use crate::brand::BrandIcon;

/// The state bar entries of each S3 screen (for a host's `ScreenStage { states }`).
pub fn s3_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

/// Focus an element when it mounts.
fn focus_on(e: MountedEvent) {
    let d = e.data();
    spawn(async move {
        let _ = d.set_focus(true).await;
    });
}

/// A progress ring toned by state (`run`, `warn`, `ok`, `bad`).
fn ring(done: u8, of: u8, tone: &'static str) -> Element {
    let pct = ring_pct(done, of);
    let dash = format!("{:.1} 100", f64::from(pct) * 0.94);
    rsx! {
        svg { class: "oxa-ring", "data-tone": "{tone}", width: "16", height: "16", view_box: "0 0 36 36", "aria-hidden": "true",
            circle { cx: "18", cy: "18", r: "15", fill: "none", stroke: "currentColor", stroke_width: "4", opacity: ".25" }
            circle { cx: "18", cy: "18", r: "15", fill: "none", stroke: "currentColor", stroke_width: "4", stroke_dasharray: "{dash}", path_length: "94", transform: "rotate(-90 18 18)", stroke_linecap: "round" }
        }
    }
}

/// A status dot.
fn dot(tone: &'static str) -> Element {
    rsx! { span { class: "oxa-dot", "data-tone": "{tone}", "aria-hidden": "true" } }
}

/// The small pill for a state.
fn pill(label: &str, tone: &'static str) -> Element {
    rsx! { span { class: "oxa-pill", "data-tone": "{tone}", "{label}" } }
}

/// The work area's tab strip: one open item with a close button.
fn wtab(icon: &'static str, label: String, mut sel: Signal<Option<u32>>) -> Element {
    rsx! {
        div { class: "oxa-wtabs", role: "tablist", "aria-label": "Open items",
            span { class: "oxa-wtab", role: "tab", "aria-selected": "true",
                {nico(icon)} span { class: "oxv-ell", "{label}" }
                button { r#type: "button", class: "oxa-wtab-x", "aria-label": "Close {label}", onclick: move |_| sel.set(None), {nico("close")} }
            }
        }
    }
}

/// A label and value row in a detail list.
fn kv(k: &'static str, v: Element) -> Element {
    rsx! { div { class: "oxa-kv", span { class: "oxa-k", "{k}" } span { class: "oxa-v", {v} } } }
}

/// The steps list.
fn steps_el(steps: &'static [Step], done: u8, warn_now: bool) -> Element {
    rsx! {
        ol { class: "oxa-steps", "aria-label": "Steps",
            for (i, (l, _)) in steps.iter().enumerate() {
                {
                    let s = if (i as u8) < done { "done" } else if i as u8 == done { "now" } else { "todo" };
                    rsx! {
                        li { key: "{i}", class: "oxa-step", "data-s": "{s}",
                            span { class: "oxa-step-g", "aria-hidden": "true", if s == "done" { {check_glyph()} } }
                            span { class: "oxv-ell", "{l}" }
                            if s == "now" && warn_now { {pill("Needs you", "warn")} }
                            span { class: "oxscn-sr", ", {s}" }
                        }
                    }
                }
            }
        }
    }
}

/// File chips with toned glyphs.
fn file_chips(files: &'static [&'static str], mut said: Signal<String>) -> Element {
    rsx! {
        div { class: "oxa-files", role: "group", "aria-label": "Files",
            for f in files.iter().copied() {
                button { key: "{f}", r#type: "button", class: "oxa-file", onclick: move |_| said.set(format!("Opened {f}")),
                    if f.contains('.') { {file_ico(f)} } else { {ico("folder")} } span { class: "oxv-ell", "{f}" } }
            }
        }
    }
}

/// An empty region.
fn none(icon: &'static str, t: &'static str, l: &'static str) -> Element {
    rsx! {
        div { class: "oxv-none",
            span { class: "oxv-none-g", {nico(icon)} }
            p { class: "oxv-none-t", "{t}" }
            p { class: "oxv-none-l", "{l}" }
        }
    }
}

/// Skeleton rows.
fn sk_rows(n: usize) -> Element {
    rsx! {
        div { class: "oxa-skrows", "aria-hidden": "true",
            for i in 0..n { div { key: "{i}", class: "oxa-skrow", span { class: "oxv-skb" } span { class: "oxv-skl" } } }
        }
    }
}

/// The pixel mark of an agent (2×2 dots in its hue).
fn agent_mark(hue: &'static str) -> Element {
    rsx! { span { class: "oxa-mark", "data-hue": "{hue}", "aria-hidden": "true", for i in 0..5 { span { key: "{i}" } } } }
}

// ───────────────────────────────────────────────────────────── Thread ──

/// Screen – Agent / Thread: one full-width conversation with the orchestrator (task cards, tools
/// folded into one line, file chips), the running / need-you counts top right and a composer with
/// + · Request approval · mic · send. The work area (Artifacts) is folded; its one toggle sits top
/// right. States: `sample` (Live), `decision`, `scrolled`, `offline`, `uploading`, `dictating`,
/// `away`, `error`, `loading`, `empty`.
#[component]
pub fn AgentThreadScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock (streaming, upload and dictation timers).
    now: ClockMs,
    /// Oxyn: tool lines start unfolded.
    #[props(default)]
    tools_open: bool,
    /// Oxyn: a time stamp on every message of yours.
    #[props(default)]
    stamps: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let all_tasks = use_signal(move || tasks(sample));
    let mut lines = use_signal(move || thread(sample));
    let mut open_tools = use_signal(move || if tools_open { vec![2usize, 8] } else { vec![] });
    let mut text = use_signal(String::new);
    let mut stream = use_signal(|| None::<(String, u64)>);
    let mut decide = use_signal(move || sk == "decision");
    let mut away = use_signal(move || sk == "away");
    let mut offline = use_signal(move || sk == "offline");
    let mut queued = use_signal(Vec::<String>::new);
    let mut failed = use_signal(move || sk == "error");
    let mut newer = use_signal(move || if sk == "scrolled" { 2u32 } else { 0 });
    let mut upload = use_signal(move || sk == "uploading");
    let mut rec = use_signal(move || sk == "dictating");
    let mut approve = use_signal(|| false);
    let start = use_signal(|| now.0);
    let mut said = use_signal(String::new);
    let mut art = use_signal(|| 0usize);
    let mut log_el = use_signal(|| None::<Rc<MountedData>>);
    let at = all_tasks();
    let (running, need) = thread_counts(&at);
    let el = since(now, start());
    let up_pct = oxyn_core::parity::screens2fid::upload_pct(el, 16_000);
    let shown = stream().map(|(t, s)| {
        let words: Vec<&str> = t.split_whitespace().collect();
        let n = (since(now, s).saturating_sub(500) / 70) as usize;
        (words[..n.min(words.len())].join(" "), n >= words.len())
    });
    if let Some((full, true)) = shown.clone() {
        spawn(async move {
            stream.set(None);
            lines.write().push(Line::Agent(full, "now"));
            said.set("Reply finished".into());
        });
    }
    let busy = shown.is_some();
    let mut send = move |t: String| {
        let t = t.trim().to_string();
        if t.is_empty() {
            return;
        }
        text.set(String::new());
        if *offline.peek() {
            queued.write().push(t);
            said.set("Queued. It sends when you're back online.".into());
            return;
        }
        lines.write().push(Line::Me(t.clone()));
        stream.set(Some((reply(&t), now.0)));
    };
    let scroll_end = move || {
        if let Some(m) = log_el.peek().clone() {
            spawn(async move {
                let _ = m
                    .scroll(PixelsVector2D::new(0.0, 1.0e6), ScrollBehavior::Smooth)
                    .await;
            });
        }
    };
    let actions = rsx! {
        if sample {
            div { class: "oxat-counts", role: "group", "aria-label": "Tasks",
                button { r#type: "button", class: "oxat-count", onclick: move |_| said.set(format!("{running} tasks running")), {dot("run")} b { "{running}" } span { "running" } }
                span { class: "oxv-dotsep", "aria-hidden": "true", "·" }
                button { r#type: "button", class: "oxat-count", onclick: move |_| said.set(format!("{need} tasks need you")), {dot("warn")} b { "{need}" } span { "need you" } }
            }
        }
    };
    const ARTS: [&str; 3] = ["load-test.csv", "pipeline-design.md", "event-schema.json"];
    let work = rsx! {
        div { class: "oxa-work",
            div { class: "oxa-wtabs", role: "tablist", "aria-label": "Artifacts",
                for (i, f) in ARTS.iter().copied().enumerate() {
                    button { key: "{f}", r#type: "button", role: "tab", class: "oxa-wtab", "aria-selected": "{art() == i}", onclick: move |_| art.set(i),
                        {file_ico(f)} span { class: "oxv-ell", "{f}" } }
                }
            }
            if !sample { p { class: "oxa-mute", "Files the orchestrator makes land here." } }
            else {
                pre { class: "oxat-pre", "aria-label": "{ARTS[art()]}",
                    match art() {
                        0 => "rate_per_min,p95_ms,errors\n5000,41,0.00%\n10000,58,0.01%\n20000,94,0.02%\n30000,212,0.40%",
                        1 => "# Event pipeline\n\nQueue: to be chosen\nReplay: one day\nRetries: 3, backoff",
                        _ => "{\n  \"version\": 2,\n  \"order_id\": \"string\",\n  \"at\": \"timestamp\"\n}",
                    }
                }
            }
        }
    };
    let list = lines();
    let me_n = std::cell::Cell::new(0u32);
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "thread", bare: true,
            actions, work, work_label: "Artifacts", work_closed: true, work_w: 420,
            div { class: "oxat", "data-state": "{sk}",
                if offline() {
                    div { class: "oxat-banner", role: "status", "data-tone": "warn",
                        {dot("warn")} span { class: "oxv-ell", "You're offline. Messages wait here and send when you're back." }
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| {
                            offline.set(false);
                            let q: Vec<String> = queued.write().drain(..).collect();
                            for t in q { lines.write().push(Line::Me(t)); }
                            said.set("Back online. Queued messages sent.".into());
                        }, "Reconnect" }
                    }
                }
                div { class: "oxat-log", role: "log", "aria-label": "Thread with the orchestrator", tabindex: "0",
                    onmounted: move |e: MountedEvent| { let d = e.data(); log_el.set(Some(d.clone())); if sk != "scrolled" && sk != "away" { spawn(async move { let _ = d.scroll(PixelsVector2D::new(0.0, 1.0e6), ScrollBehavior::Instant).await; }); } },
                    if loading { div { class: "oxat-col", {skel(8)} } }
                    else if list.is_empty() && shown.is_none() {
                        div { class: "oxat-col oxat-start",
                            {none("chat", "Ask for an outcome", "The orchestrator plans the tasks, runs them side by side and checks in here when it needs you.")}
                        }
                    } else {
                        div { class: "oxat-col",
                            if away() {
                                section { class: "oxat-away", "aria-label": "While you were away",
                                    div { class: "oxat-away-h", {nico("clock")} b { "While you were away" } span { class: "oxv-q", "3h · 3 updates" }
                                        button { r#type: "button", class: "oxa-x", "aria-label": "Dismiss the summary", onclick: move |_| away.set(false), {nico("close")} } }
                                    ul { for (i, l) in AWAY.iter().enumerate() { li { key: "{i}", "{l}" } } }
                                }
                            }
                            for (i, it) in list.into_iter().enumerate() {
                                {
                                    match it {
                                        Line::Me(t) => {
                                            me_n.set(me_n.get() + 1);
                                            let stamp = ["08:31", "11:12", "11:40", "now"][(me_n.get() as usize - 1).min(3)];
                                            rsx! { div { key: "{i}", class: "oxat-me", p { "{t}" } if stamps { span { class: "oxv-q", "{stamp}" } } } }
                                        }
                                        Line::Agent(t, at) if t.is_empty() => rsx! {
                                            div { key: "{i}", class: "oxat-who", {agent_mark("purple")} b { "Orchestrator" } span { class: "oxv-q", "{at}" } }
                                        },
                                        Line::Agent(t, _) => rsx! { p { key: "{i}", class: "oxat-ag", "{t}" } },
                                        Line::Tools(n, ds) => {
                                            let o = open_tools().contains(&i);
                                            let w = tools_words(n, ds);
                                            rsx! {
                                                div { key: "{i}", class: "oxat-tools",
                                                    button { r#type: "button", class: "oxat-tb", "aria-expanded": "{o}",
                                                        onclick: move |_| { let mut v = open_tools.write(); if let Some(p) = v.iter().position(|x| *x == i) { v.remove(p); } else { v.push(i); } },
                                                        span { class: "oxat-ok", {check_glyph()} } span { "{w}" } {chevron(o)} }
                                                    if o {
                                                        ul { class: "oxat-calls",
                                                            for (j, c) in ["tasks.list", "files.read", "calendar.find"].iter().take(n as usize).enumerate() {
                                                                li { key: "{j}", {nico("term")} span { class: "oxv-ell", "{c}" } span { class: "oxv-q", "{0.4 + j as f32 * 0.7:.1}s" } }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Line::Task(id) => {
                                            let t = at.iter().find(|t| t.id == id).cloned();
                                            rsx! {
                                                if let Some(t) = t {
                                                    div { key: "{i}", class: "oxat-task",
                                                        {ring(t.done, t.of, t.state.key())}
                                                        span { class: "oxat-task-t", b { class: "oxv-ell", "{t.name}" } span { class: "oxv-ell oxv-q", "{t.now}" } }
                                                        span { class: "oxv-q", "{t.done}/{t.of}" }
                                                        button { r#type: "button", class: "oxat-open", "aria-label": "Open {t.name}", onclick: move |_| said.set(format!("{} opened in Tasks", t.name)), "Open" }
                                                    }
                                                }
                                            }
                                        }
                                        Line::Files(fs) => rsx! { div { key: "{i}", {file_chips(fs, said)} } },
                                    }
                                }
                            }
                            if let Some((t, _)) = shown.clone() {
                                div { class: "oxat-who", {agent_mark("purple")} b { "Orchestrator" } span { class: "oxv-q", "now" } }
                                if t.is_empty() { p { class: "oxat-ag oxat-think", span { class: "oxscn-pulse" } "Thinking…" } } else { p { class: "oxat-ag", "{t}" } }
                            }
                            if failed() {
                                div { class: "oxat-who", {agent_mark("purple")} b { "Orchestrator" } span { class: "oxv-q", "now" } }
                                div { class: "oxat-err", role: "alert",
                                    span { class: "oxat-err-g", "aria-hidden": "true", "!" }
                                    span { class: "oxat-task-t", b { "The reply didn't come through" } span { class: "oxv-q oxv-ell", "The model timed out after 30s. Your message is safe." } }
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { failed.set(false); stream.set(Some((reply("Get the pipeline onto staging"), now.0))); }, "Retry" }
                                }
                            }
                            if decide() {
                                div { class: "oxat-who", {agent_mark("purple")} b { "Orchestrator" } span { class: "oxv-q", "now" } }
                                section { class: "oxat-dec", "aria-label": "Decision", tabindex: "0",
                                    onkeydown: move |e: KeyboardEvent| {
                                        let k = match e.key() { Key::Character(c) => key_choice(&c, 3), _ => None };
                                        if let Some(k) = k {
                                            let l = ["Kafka", "SQS", "Postgres queue"][k];
                                            decide.set(false); lines.write().push(Line::Me(format!("Go with {l}."))); stream.set(Some((format!("Going with {l}. The pipeline moves to staging next."), now.0)));
                                        }
                                    },
                                    div { class: "oxat-dec-h", {pill("Needs you", "warn")} span { class: "oxv-q", "Event pipeline · step 3 of 4" } }
                                    p { class: "oxat-dec-q", "Which queue should the event service use?" }
                                    div { class: "oxat-dec-o", role: "group", "aria-label": "Answers",
                                        for (k, (l, s)) in [("Kafka", "Replay built in"), ("SQS", "Nothing to run"), ("Postgres queue", "Reuses the database")].into_iter().enumerate() {
                                            button { key: "{k}", r#type: "button", class: "oxat-opt", "aria-keyshortcuts": "{k + 1}",
                                                onclick: move |_| { decide.set(false); lines.write().push(Line::Me(format!("Go with {l}."))); stream.set(Some((format!("Going with {l}. The pipeline moves to staging next."), now.0))); },
                                                kbd { "{k + 1}" } b { "{l}" } span { class: "oxv-q oxv-ell", "{s}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if newer() > 0 {
                    button { r#type: "button", class: "oxat-jump", onclick: move |_| { newer.set(0); scroll_end(); said.set("Jumped to the latest".into()); },
                        span { class: "oxat-jump-d", "{newer} new" } {nico("updown")} span { "Jump to latest" } }
                }
                div { class: "oxat-cmp", "data-busy": "{busy}",
                    if upload() {
                        div { class: "oxat-up",
                            {file_ico("floorplan.pdf")} span { class: "oxv-ell", "venue-floorplan.pdf" }
                            span { class: "oxv-q", if up_pct >= 100 { "Ready" } else { "{up_pct}% · 4.2 MB" } }
                            span { class: "oxat-up-bar", role: "progressbar", "aria-label": "Upload", "aria-valuenow": "{up_pct}", "aria-valuemin": "0", "aria-valuemax": "100", span { style: "width:{up_pct}%" } }
                            button { r#type: "button", class: "oxa-x", "aria-label": "Remove the file", onclick: move |_| upload.set(false), {nico("close")} }
                        }
                    }
                    if !queued().is_empty() {
                        p { class: "oxat-queued", role: "status", "{queued().len()} waiting to send" }
                    }
                    if rec() {
                        div { class: "oxat-rec", role: "status", "aria-label": "Dictating",
                            span { class: "oxat-rec-d" }
                            span { class: "oxat-wave", "aria-hidden": "true", for i in 0..28u32 { span { key: "{i}", style: "height:{6 + (i * 7) % 17}px;animation-delay:-{i * 70}ms" } } }
                            span { class: "oxv-q", "{dict_clock(el / 1000)}" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { rec.set(false); text.set("Check the staging numbers before lunch".into()); said.set("Dictation added to the message".into()); }, "Done" }
                        }
                    } else {
                        textarea { class: "oxat-ta", rows: "2", placeholder: if offline() { "Write now; it sends when you're back" } else { "Ask for anything" }, "aria-label": "Message the orchestrator", value: "{text}",
                            oninput: move |e| text.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); let t = text.peek().clone(); send(t); } }
                    }
                    div { class: "oxat-row",
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Attach a file", onclick: move |_| { upload.set(true); said.set("Uploading venue-floorplan.pdf".into()); }, {nico("plus")} }
                        button { r#type: "button", class: "oxat-appr", "aria-pressed": "{approve}", onclick: move |_| approve.toggle(), {nico("hand")} span { "Request approval" } }
                        span { class: "oxscn-grow" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": if rec() { "Stop dictating" } else { "Dictate" }, "aria-pressed": "{rec}", onclick: move |_| rec.toggle(), {nico("mic")} }
                        if busy {
                            button { r#type: "button", class: "oxat-send", "aria-label": "Stop", onclick: move |_| { let s = shown.clone(); stream.set(None); if let Some((t, _)) = s { lines.write().push(Line::Agent(format!("{t} (stopped)"), "now")); } said.set("Stopped".into()); }, {nico("stop")} }
                        } else {
                            button { r#type: "button", class: "oxat-send", "aria-label": "Send", disabled: text().trim().is_empty(), onclick: move |_| { let t = text.peek().clone(); send(t); }, {nico("send")} }
                        }
                    }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── Inbox ──

/// The task context a question belongs to (the Inbox work area).
fn task_ctx(t: &Task, said: Signal<String>) -> Element {
    let pr = project(t.project).1;
    rsx! {
        p { class: "oxa-brief", "{t.brief}" }
        div { class: "oxa-kvs",
            {kv("Status", rsx! { {ring(t.done, t.of, t.state.key())} span { if t.state == TaskState::NeedsYou { "Waiting on you" } else { "{t.now}" } } span { class: "oxv-q", "{t.done}/{t.of}" } })}
            {kv("Project", rsx! { "{pr}" })}
            {kv("Started", rsx! { "Mon, 3 Nov" })}
        }
        h3 { class: "oxa-h3", "Steps" }
        {steps_el(t.steps, t.done, t.state == TaskState::NeedsYou)}
        h3 { class: "oxa-h3", "Artifacts " span { class: "oxv-q", "{t.files.len()}" } }
        {file_chips(t.files, said)}
    }
}

/// Screen – Agent / Inbox: the list (320) of questions agents are waiting on, tinted by project,
/// with Open / Done tabs; the detail (466) shows one question at a time with its answers, and one
/// keystroke (1–9, Y/N, L for later) answers it while the next question glides in. The work area
/// shows the task it belongs to. States: `sample`, `loading`, `empty`, `answered`.
#[component]
pub fn InboxScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: a peek at the next question under the answer bar.
    #[props(default)]
    peek_next: bool,
    /// Oxyn: stay on the answered question instead of gliding to the next.
    #[props(default)]
    hold: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let all_tasks = use_signal(move || tasks(sample));
    let mut open = use_signal(move || {
        let mut q = questions(sample);
        if sk == "answered" {
            q.remove(0);
        }
        q
    });
    let mut done = use_signal(move || {
        if sk == "answered" {
            vec![(questions(true).remove(0), "Kafka".to_string())]
        } else {
            vec![]
        }
    });
    let mut sel = use_signal(move || {
        if sample {
            Some(if sk == "answered" { 2 } else { 1 })
        } else {
            None
        }
    });
    let mut pick = use_signal(|| 0usize);
    let tab = use_signal(|| "open");
    let mut other = use_signal(String::new);
    let mut glide = use_signal(move || sk == "answered");
    let mut refocus = use_signal(|| false);
    let mut toast = use_signal(move || {
        if sk == "answered" {
            Some((1u32, "Kafka".to_string()))
        } else {
            None
        }
    });
    let mut said = use_signal(String::new);
    let qs = open();
    let cur = sel().and_then(|id| qs.iter().find(|q| q.id == id).cloned());
    let mut answer = move |q: Question, label: String| {
        let nx = next_question(&open.peek(), q.id);
        open.write().retain(|x| x.id != q.id);
        done.write().insert(0, (q.clone(), label.clone()));
        if !hold {
            sel.set(nx);
            glide.set(true);
            refocus.set(true);
        }
        pick.set(0);
        other.set(String::new());
        toast.set(Some((q.id, label.clone())));
        said.set(format!(
            "Sent “{label}” to {}. {}",
            q.from,
            if nx.is_some() {
                "Next question."
            } else {
                "Inbox clear."
            }
        ));
    };
    let mut later = move |q: Question| {
        let mut v = open.write();
        if let Some(p) = v.iter().position(|x| x.id == q.id) {
            let x = v.remove(p);
            v.push(x);
        }
        let nx = v.first().map(|x| x.id);
        drop(v);
        sel.set(nx);
        glide.set(true);
        refocus.set(true);
        said.set(format!("{} moved to the end", q.from));
    };
    let mut undo = move |id: u32| {
        let mut d = done.write();
        if let Some(p) = d.iter().position(|(q, _)| q.id == id) {
            let (q, _) = d.remove(p);
            drop(d);
            open.write().insert(0, q);
            sel.set(Some(id));
            toast.set(None);
            said.set("Answer taken back".into());
        }
    };
    let n_open = qs.len();
    let n_done = done().len();
    let next_title = cur
        .as_ref()
        .and_then(|q| next_question(&qs, q.id))
        .and_then(|id| qs.iter().find(|x| x.id == id).map(|x| x.title));
    let ctx = cur
        .as_ref()
        .and_then(|q| all_tasks().into_iter().find(|t| t.id == q.task));
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(t) = ctx.clone() {
                {wtab("tasks", t.name.to_string(), sel)}
                div { class: "oxa-wbody", {task_ctx(&t, said)} }
            } else {
                p { class: "oxa-mute", "The task behind a question opens here." }
            }
        }
    };
    let row = move |q: Question, is_done: Option<String>| {
        let on = sel() == Some(q.id);
        let hue = project(q.project).2;
        let id = q.id;
        let sub = match &is_done {
            Some(l) => format!("Answered: {l}"),
            None => format!("{} · {}", q.from, age(q.mins)),
        };
        let g = match q.kind {
            "approve" => "check",
            "variants" => "apps",
            _ => "route",
        };
        rsx! {
            li { key: "{id}",
                button { r#type: "button", class: "oxai-row", "data-on": "{on}", "aria-current": if on { "true" } else { "false" },
                    onclick: move |_| { sel.set(Some(id)); glide.set(false); },
                    span { class: "oxai-g", "data-hue": "{hue}", {nico(g)} }
                    span { class: "oxai-t", span { class: "oxv-ell oxai-q", "{q.title}" } span { class: "oxv-ell oxv-q", "{sub}" } }
                    if q.unread && is_done.is_none() { span { class: "oxai-unread", "aria-label": "Unread" } }
                }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "inbox", title: "Inbox",
            badge: rsx! { span { class: "oxa-n", "{n_open}" } {tabs("Show", vec![("open", "Open".to_string(), None), ("done", "Done".to_string(), if n_done > 0 { Some(n_done) } else { None })], tab)} },
            work, work_label: "Task", work_w: 368,
            div { class: "oxai",
                div { class: "oxai-list",
                    if loading { {sk_rows(5)} }
                    else if tab() == "done" {
                        if done().is_empty() { p { class: "oxa-mute", "Questions you answer move here." } }
                        ul { class: "oxai-ul", "aria-label": "Answered", for (q, l) in done() { {row(q, Some(l))} } }
                    } else if qs.is_empty() {
                        p { class: "oxa-mute", if sample { "Nothing open." } else { "No questions yet." } }
                    } else {
                        ul { class: "oxai-ul", "aria-label": "Open questions", for q in qs.clone() { {row(q, None)} } }
                    }
                }
                div { class: "oxai-det",
                    if loading { div { class: "oxai-qw", {skel(6)} } }
                    else if let Some(q) = cur.clone() {
                        {
                            let n = q.choices.len();
                            let hue = project(q.project).2;
                            let q2 = q.clone();
                            let q3 = q.clone();
                            let q4 = q.clone();
                            let q5 = q.clone();
                            rsx! {
                                div { key: "{q.id}", class: "oxai-qw", "data-glide": "{glide}", tabindex: "0", role: "group", "aria-label": "{q.title}",
                                    "aria-keyshortcuts": "1 2 3 Y N L Enter",
                                    onmounted: move |e: MountedEvent| if *refocus.peek() { refocus.set(false); focus_on(e) },
                                    onkeydown: move |e: KeyboardEvent| {
                                        if let Key::Character(c) = e.key() {
                                            if c == "l" || c == "L" { e.prevent_default(); later(q3.clone()); return; }
                                            if let Some(k) = key_choice(&c, n) { e.prevent_default(); answer(q3.clone(), q3.choices[k].label.to_string()); return; }
                                        }
                                        match e.key() {
                                            Key::ArrowDown => { e.prevent_default(); let k = (*pick.peek() + 1) % n; pick.set(k); }
                                            Key::ArrowUp => { e.prevent_default(); let k = (*pick.peek() + n - 1) % n; pick.set(k); }
                                            Key::Enter => { e.prevent_default(); let k = *pick.peek(); answer(q3.clone(), q3.choices[k].label.to_string()); }
                                            _ => {}
                                        }
                                    },
                                    p { class: "oxai-from", span { class: "oxai-g", "data-hue": "{hue}", {nico("route")} } span { class: "oxv-ell", "{q.from}" } span { class: "oxv-q", "· {age(q.mins)} ago" } }
                                    h2 { class: "oxai-h", "{q.title}" }
                                    p { class: "oxai-lead", "{q.lead}" }
                                    div { class: "oxai-opts", role: "radiogroup", "aria-label": "Answers",
                                        for (k, c) in q.choices.iter().cloned().enumerate() {
                                            {
                                                let on = pick() == k;
                                                let qq = q2.clone();
                                                rsx! {
                                                    button { key: "{k}", r#type: "button", role: "radio", class: "oxai-opt", "aria-checked": "{on}", tabindex: "-1",
                                                        onclick: move |_| { if *pick.peek() == k { answer(qq.clone(), c.label.to_string()); } else { pick.set(k); } },
                                                        kbd { "{k + 1}" }
                                                        span { class: "oxai-ot",
                                                            span { class: "oxai-ol", b { "{c.label}" } if c.pick { {pill("Suggested", "run")} } }
                                                            span { class: "oxv-q2", "{c.sub}" }
                                                            if on && !c.why.is_empty() {
                                                                ul { class: "oxai-why", for (w, plus) in c.why.iter().copied() { li { key: "{w}", "data-plus": "{plus}", span { "aria-hidden": "true", if plus { "+" } else { "−" } } "{w}" } } }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "oxai-bar",
                                        input { class: "oxv-in", r#type: "text", placeholder: "Or tell it something else", "aria-label": "Another answer", value: "{other}",
                                            oninput: move |e| other.set(e.value()),
                                            onkeydown: move |e: KeyboardEvent| { e.stop_propagation(); if e.key() == Key::Enter { let t = other.peek().trim().to_string(); if !t.is_empty() { answer(q4.clone(), t); } } } }
                                        button { r#type: "button", class: "oxscn-btn oxv-ghost", "aria-keyshortcuts": "L", onclick: move |_| later(q5.clone()), "Later" }
                                        button { r#type: "button", class: "oxv-pri", "aria-keyshortcuts": "Enter",
                                            onclick: move |_| { let k = *pick.peek(); let t = other.peek().trim().to_string(); let l = if t.is_empty() { q.choices[k].label.to_string() } else { t }; answer(q.clone(), l); }, "Send" }
                                    }
                                    p { class: "oxai-keys", "aria-hidden": "true", kbd { "1" } "–" kbd { "{n}" } " answer · " kbd { "↵" } " send · " kbd { "L" } " later" }
                                    if peek_next { if let Some(nt) = next_title { p { class: "oxai-next", span { class: "oxv-q", "Next" } span { class: "oxv-ell", "{nt}" } } } }
                                }
                            }
                        }
                    } else if sample || !done().is_empty() {
                        {none("check", "All caught up", "Agents ask here before they send or book anything. New questions appear on the left.")}
                    } else {
                        {none("inbox", "Nothing needs you yet", "When an agent needs a decision, the question waits here with its options.")}
                    }
                    if let Some((id, l)) = toast() {
                        div { class: "oxa-toast", role: "status", {check_glyph()} span { class: "oxv-ell", "Sent “{l}”" }
                            button { r#type: "button", class: "oxv-link", onclick: move |_| undo(id), "Undo" }
                            button { r#type: "button", class: "oxa-x", "aria-label": "Dismiss", onclick: move |_| toast.set(None), {nico("close")} } }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Board ──

fn lane_ico(lane: &str) -> Element {
    match lane {
        "needs" => rsx! { span { class: "oxab-li", "data-tone": "warn", {nico("hand")} } },
        "running" => {
            rsx! { span { class: "oxab-li oxa-spin", "data-tone": "run", "aria-hidden": "true" } }
        }
        _ => rsx! { span { class: "oxab-li", "data-tone": "ok", {check_glyph()} } },
    }
}

/// Screen – Agent / Inbox Board: three lanes (Needs you · Running · Done) of task cards with the
/// agent's mark, elapsed time, sources read and tool calls. Tabs narrow to a lane; Agents filters;
/// board or list view; New task adds a card. The work area approves, rejects, retries or opens the
/// selected card. States: `sample`, `loading`, `empty`, `failed`.
#[component]
pub fn InboxBoardScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: one-line cards.
    #[props(default)]
    compact: bool,
    /// Oxyn: a work-in-progress limit on Running (shown as 3 / 4).
    #[props(default)]
    wip: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut cards = use_signal(move || board(sample, sk == "failed"));
    let mut sel = use_signal(move || {
        if sample {
            Some(if sk == "failed" { 2 } else { 1 })
        } else {
            None
        }
    });
    let tab = use_signal(|| "all");
    let mut agent = use_signal(String::new);
    let mut list_view = use_signal(|| false);
    let mut adding = use_signal(|| false);
    let mut new_t = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = cards();
    let c = board_counts(&all);
    let mut agents: Vec<(&'static str, &'static str, &'static str)> = vec![("", "All agents", "")];
    for a in [
        "Builder", "Keeper", "Reviewer", "Writer", "Guard", "Analyst",
    ] {
        agents.push((a, a, ""));
    }
    let shown: Vec<BoardCard> = all
        .iter()
        .filter(|x| agent().is_empty() || x.agent.0 == agent())
        .cloned()
        .collect();
    let lanes: Vec<(&str, &str)> = LANES
        .iter()
        .copied()
        .filter(|(k, _)| tab() == "all" || tab() == *k)
        .collect();
    let cur = sel().and_then(|id| all.iter().find(|x| x.id == id).cloned());
    let mut add = move || {
        let t = new_t.peek().trim().to_string();
        if t.is_empty() {
            return;
        }
        let id = cards.peek().iter().map(|c| c.id).max().unwrap_or(0) + 1;
        cards.write().insert(
            0,
            BoardCard {
                id,
                title: Box::leak(t.clone().into_boxed_str()),
                blurb: "Queued. An agent picks it up in a moment.",
                lane: "running",
                agent: ("Builder", "blue"),
                secs: 0,
                reads: 0,
                calls: 0,
                failed: false,
                did: &[],
            },
        );
        new_t.set(String::new());
        adding.set(false);
        sel.set(Some(id));
        said.set(format!("Started {t}"));
    };
    let card = move |x: BoardCard| {
        let id = x.id;
        let on = sel() == Some(id);
        let tone = if x.failed {
            "bad"
        } else {
            match x.lane {
                "needs" => "warn",
                "running" => "run",
                _ => "ok",
            }
        };
        rsx! {
            li { key: "{id}",
                button { r#type: "button", class: "oxab-card", "data-on": "{on}", "data-compact": "{compact}", "aria-current": if on { "true" } else { "false" },
                    onclick: move |_| sel.set(Some(id)),
                    span { class: "oxab-ct", {agent_mark(x.agent.1)} b { class: "oxv-ell", "{x.title}" } }
                    if !compact { span { class: "oxab-cb", "{x.blurb}" } }
                    span { class: "oxab-cf",
                        {dot(tone)}
                        span { class: "oxab-time", if x.failed { "Failed" } else { "{mmss(x.secs)}" } }
                        span { class: "oxscn-grow" }
                        span { class: "oxab-m", title: "Sources read", {nico("search")} "{x.reads}" }
                        span { class: "oxab-m", title: "Tool calls", {nico("term")} "{x.calls}" }
                    }
                }
            }
        }
    };
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(x) = cur.clone() {
                {wtab("inbox", x.title.to_string(), sel)}
                div { class: "oxa-wbody",
                    h2 { class: "oxa-h2", "{x.title}" }
                    p { class: "oxa-brief", "{x.blurb}" }
                    div { class: "oxa-acts",
                        if x.failed {
                            button { r#type: "button", class: "oxv-pri", onclick: move |_| { move_to(&mut cards.write(), x.id, "running"); said.set(format!("Retrying {}", x.title)); }, "Retry" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| said.set("Opened the run log".into()), "Open log" }
                        } else if x.lane == "needs" {
                            button { r#type: "button", class: "oxv-pri", onclick: move |_| { move_to(&mut cards.write(), x.id, "running"); said.set(format!("Approved {}", x.title)); }, "Approve" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { move_to(&mut cards.write(), x.id, "done"); said.set(format!("Rejected {}", x.title)); }, "Reject" }
                        } else {
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| said.set(format!("Opened {}", x.title)), "Open" }
                        }
                    }
                    div { class: "oxa-kvs",
                        {kv("Status", rsx! { {dot(if x.failed { "bad" } else { match x.lane { "needs" => "warn", "running" => "run", _ => "ok" } })} span { if x.failed { "Stopped" } else { match x.lane { "needs" => "Waiting for you", "running" => "Running", _ => "Done" } } } span { class: "oxv-q", "{mmss(x.secs)}" } })}
                        {kv("Agent", rsx! { {agent_mark(x.agent.1)} span { "{x.agent.0}" } })}
                        {kv("Sources", rsx! { "{x.reads} read" })}
                        {kv("Tools", rsx! { "{x.calls} calls" })}
                    }
                    if !x.did.is_empty() {
                        h3 { class: "oxa-h3", "What it did " span { class: "oxv-q", "{x.did.len()}" } }
                        ol { class: "oxab-did", for (l, a) in x.did.iter().copied() { li { key: "{l}", span { "{l}" } span { class: "oxv-q", "{a}" } } } }
                    }
                }
            } else {
                p { class: "oxa-mute", "Pick a card to approve, retry or open it." }
            }
        }
    };
    let counts = [
        ("all", "All tasks", c[0]),
        ("needs", "Needs you", c[1]),
        ("running", "Running", c[2]),
        ("done", "Done", c[3]),
    ];
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "inbox",
            badge: rsx! { {tabs("Lanes", counts.iter().map(|(k, l, n)| (*k, l.to_string(), Some(*n))).collect(), tab)} },
            actions: rsx! {
                MenuButton { label: if agent().is_empty() { "Agents".to_string() } else { agent() }, aria: "Filter by agent", icon: "filter", right: true, caret: false, items: agents, checked: agent(), on_pick: move |k: String| agent.set(k) }
                div { class: "oxv-seg", role: "group", "aria-label": "View",
                    button { r#type: "button", class: "oxv-seg-b", "aria-label": "Board", "aria-pressed": "{!list_view()}", onclick: move |_| list_view.set(false), {nico("kanban")} }
                    button { r#type: "button", class: "oxv-seg-b", "aria-label": "List", "aria-pressed": "{list_view}", onclick: move |_| list_view.set(true), {nico("list")} }
                }
                div { class: "oxv-pop-w",
                    button { r#type: "button", class: "oxv-pri", "aria-label": "New task", "aria-haspopup": "dialog", "aria-expanded": "{adding}", onclick: move |_| adding.toggle(), {nico("plus")} span { class: "oxa-bl", "New task" } }
                    if adding() {
                        div { class: "oxscn-catch", onclick: move |_| adding.set(false) }
                        div { class: "oxv-pop", role: "dialog", "aria-label": "New task",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false) },
                            p { class: "oxv-pop-h", "What should an agent do?" }
                            input { class: "oxv-in", r#type: "text", placeholder: "Describe the task", "aria-label": "Task", value: "{new_t}", onmounted: focus_on,
                                oninput: move |e| new_t.set(e.value()), onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { add() } }
                            div { class: "oxv-pop-a",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                                button { r#type: "button", class: "oxv-pri", disabled: new_t().trim().is_empty(), onclick: move |_| add(), "Start" }
                            }
                        }
                    }
                }
            },
            work, work_label: "Task", work_w: 348,
            div { class: "oxab", "data-view": if list_view() { "list" } else { "board" },
                if loading {
                    for l in LANES { div { key: "{l.0}", class: "oxab-lane", div { class: "oxab-lh", span { class: "oxv-skl" } } {sk_rows(2)} } }
                } else if all.is_empty() {
                    div { class: "oxab-none", {none("kanban", "No tasks on the board", "Start a task and it moves from Running to Done here, stopping in Needs you when it wants a decision.")} }
                } else {
                    for (k, l) in lanes {
                        {
                            let items: Vec<BoardCard> = shown.iter().filter(|x| x.lane == k).cloned().collect();
                            let n = items.len();
                            rsx! {
                                section { key: "{k}", class: "oxab-lane", "aria-label": "{l}",
                                    div { class: "oxab-lh", {lane_ico(k)} span { class: "oxab-lt", "{l}" } span { class: "oxab-ln", if wip && k == "running" { "{n} / 4" } else { "{n}" } } }
                                    if items.is_empty() { p { class: "oxa-mute oxab-empty", "Nothing here" } }
                                    ul { class: "oxab-ul", for x in items { {card(x)} } }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Tasks ──

/// Screen – Agent / Tasks: rows grouped by project (folding), each with a progress ring, its name,
/// what it is doing, a Needs you tag, steps, files and age. Active / Done tabs and New task; the
/// work area shows the decision waiting (Decide answers it inline), status, steps, what happened so
/// far, artifacts and what was decided. States: `sample`, `loading`, `empty`, `finished` (Done).
#[component]
pub fn TasksScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: a cost column on each row.
    #[props(default)]
    cost: bool,
    /// Oxyn: groups start folded.
    #[props(default)]
    folded: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut list = use_signal(move || tasks(sample));
    let mut tab = use_signal(move || if sk == "finished" { "done" } else { "active" });
    let mut sel = use_signal(move || {
        if sample {
            Some(if sk == "finished" { 10 } else { 1 })
        } else {
            None
        }
    });
    let mut shut = use_signal(move || {
        if folded {
            vec!["core", "bench"]
        } else {
            vec![]
        }
    });
    let mut deciding = use_signal(|| false);
    let mut adding = use_signal(|| false);
    let mut new_t = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = list();
    let (na, nd) = task_counts(&all);
    let groups = task_groups(&all, tab() == "done");
    let cur = sel().and_then(|id| all.iter().find(|t| t.id == id).cloned());
    let qs = questions(true);
    let mut add = move || {
        let t = new_t.peek().trim().to_string();
        if t.is_empty() {
            return;
        }
        let id = list.peek().iter().map(|x| x.id).max().unwrap_or(0) + 1;
        let mut base = tasks(true).remove(3);
        base.id = id;
        base.name = Box::leak(t.clone().into_boxed_str());
        base.now = "Planning the steps";
        base.done = 0;
        base.mins = 0;
        list.write().insert(0, base);
        tab.set("active");
        sel.set(Some(id));
        new_t.set(String::new());
        adding.set(false);
        said.set(format!("Started {t}"));
    };
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(t) = cur.clone() {
                {wtab("tasks", t.name.to_string(), sel)}
                div { class: "oxa-wbody",
                    p { class: "oxa-brief", "{t.brief}" }
                    if let Some(ask) = t.ask.filter(|_| t.state == TaskState::NeedsYou) {
                        {
                            let q = qs.iter().find(|q| q.task == t.id).cloned();
                            let tid = t.id;
                            rsx! {
                                div { class: "oxak-dec",
                                    div { class: "oxak-dec-r", span { class: "oxak-dec-t", span { {pill("Needs you", "warn")} span { class: "oxv-q", "1 decision waiting" } } b { "{ask}" } }
                                        button { r#type: "button", class: "oxv-pri", "aria-expanded": "{deciding}", onclick: move |_| deciding.toggle(), "Decide" } }
                                    if deciding() { if let Some(q) = q {
                                        div { class: "oxak-dec-o", role: "group", "aria-label": "Answers",
                                            for (k, c) in q.choices.iter().cloned().enumerate() {
                                                button { key: "{k}", r#type: "button", class: "oxscn-btn",
                                                    onclick: move |_| {
                                                        if let Some(x) = list.write().iter_mut().find(|x| x.id == tid) { x.state = TaskState::Running; x.done += 1; x.ask = None; x.now = "Back at work"; }
                                                        deciding.set(false);
                                                        said.set(format!("Decided: {}", c.label));
                                                    },
                                                    kbd { "{k + 1}" } "{c.label}" }
                                            }
                                        }
                                    } }
                                }
                            }
                        }
                    }
                    div { class: "oxa-kvs",
                        {kv("Status", rsx! { {ring(t.done, t.of, t.state.key())} span { "{t.now}" } span { class: "oxv-q", "{t.done}/{t.of}" } })}
                        {kv("Project", rsx! { "{project(t.project).1}" })}
                        {kv("Started", rsx! { "Mon, 3 Nov" })}
                    }
                    h3 { class: "oxa-h3", "Steps" }
                    {steps_el(t.steps, t.done.min(t.of), t.state == TaskState::NeedsYou)}
                    h3 { class: "oxa-h3", "So far" }
                    ul { class: "oxak-log", for l in t.log.iter().copied() { li { key: "{l}", "{l}" } } }
                    h3 { class: "oxa-h3", "Artifacts " span { class: "oxv-q", "{t.files.len()}" } }
                    {file_chips(t.files, said)}
                    h3 { class: "oxa-h3", "Decided" }
                    div { class: "oxak-decided", span { class: "oxat-ok", {check_glyph()} } span { class: "oxak-dec-t", b { "Which format for the events?" } span { class: "oxv-q", "JSON with a version field · 3 days ago" } } }
                }
            } else {
                p { class: "oxa-mute", "Pick a task to see its steps, files and decisions." }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "tasks", title: "Tasks",
            badge: rsx! { span { class: "oxa-n", "{na}" } {tabs("Show", vec![("active", "Active".to_string(), None), ("done", "Done".to_string(), if nd > 0 { Some(nd) } else { None })], tab)} },
            actions: rsx! {
                div { class: "oxv-pop-w",
                    button { r#type: "button", class: "oxscn-btn", "aria-label": "New task", "aria-haspopup": "dialog", "aria-expanded": "{adding}", onclick: move |_| adding.toggle(), {nico("plus")} span { class: "oxa-bl", "New task" } }
                    if adding() {
                        div { class: "oxscn-catch", onclick: move |_| adding.set(false) }
                        div { class: "oxv-pop", role: "dialog", "aria-label": "New task",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false) },
                            p { class: "oxv-pop-h", "What should it get done?" }
                            input { class: "oxv-in", r#type: "text", placeholder: "Describe the outcome", "aria-label": "Task", value: "{new_t}", onmounted: focus_on,
                                oninput: move |e| new_t.set(e.value()), onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { add() } }
                            div { class: "oxv-pop-a",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                                button { r#type: "button", class: "oxv-pri", disabled: new_t().trim().is_empty(), onclick: move |_| add(), "Start" }
                            }
                        }
                    }
                }
            },
            work, work_label: "Task", work_w: 368,
            div { class: "oxak",
                if loading { {sk_rows(9)} }
                else if groups.is_empty() {
                    if tab() == "done" { {none("tasks", "Nothing finished yet", "Tasks land here once every step is done.")} }
                    else { {none("tasks", "No tasks yet", "Ask for an outcome in the thread, or start one with New task.")} }
                } else {
                    for ((pk, pn, hue), items) in groups {
                        {
                            let open = !shut().contains(&pk);
                            let n = items.len();
                            rsx! {
                                section { key: "{pk}", class: "oxak-g", "aria-label": "{pn}",
                                    button { r#type: "button", class: "oxak-gh", "aria-expanded": "{open}",
                                        onclick: move |_| { let mut v = shut.write(); if let Some(p) = v.iter().position(|x| *x == pk) { v.remove(p); } else { v.push(pk); } },
                                        {chevron(open)} span { class: "oxak-fg", "data-hue": "{hue}", {nico("folder")} } b { "{pn}" } span { class: "oxv-q", "{n}" } }
                                    if open {
                                        ul { class: "oxak-ul",
                                            for t in items {
                                                {
                                                    let id = t.id;
                                                    let on = sel() == Some(id);
                                                    rsx! {
                                                        li { key: "{id}",
                                                            button { r#type: "button", class: "oxak-row", "data-on": "{on}", "aria-current": if on { "true" } else { "false" }, onclick: move |_| { sel.set(Some(id)); deciding.set(false); },
                                                                {ring(t.done, t.of, t.state.key())}
                                                                span { class: "oxak-rt", b { class: "oxv-ell", "{t.name}" } span { class: "oxv-ell oxv-q2", "{t.now}" } }
                                                                if t.state == TaskState::NeedsYou { {pill("Needs you", "warn")} }
                                                                if cost { span { class: "oxak-c", "{eur(18 + id * 7)}" } }
                                                                span { class: "oxak-c", "{t.done}/{t.of}" }
                                                                span { class: "oxak-c", {nico("doc")} "{t.files.len()}" }
                                                                span { class: "oxak-c oxak-age", "{age(t.mins)}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Activity ──

/// Screen – Agent / Activity: what agents did in your apps, grouped by day, with filter tabs and
/// counts (All · On its own · You approved · Undone) and a search. Each row has its time, the app's
/// official mark, the sentence, how it was allowed and Undo or Final. The work area shows When /
/// Task / Cost, What happened, Why it could and Taking it back with Undo and a live countdown.
/// States: `sample`, `loading`, `empty`, `undone`.
#[component]
pub fn ActivityScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock (the undo countdown).
    now: ClockMs,
    /// Oxyn: a cost column on each row.
    #[props(default)]
    cost: bool,
    /// Oxyn: the countdown as a clock (17:42) instead of words.
    #[props(default)]
    clock: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut rows = use_signal(move || {
        let mut v = activity(sample);
        if sk == "undone" {
            v[0].undone = true;
        }
        v
    });
    let tab = use_signal(|| "all");
    let q = use_signal(String::new);
    let mut sel = use_signal(move || if sample { Some(1u32) } else { None });
    let start = use_signal(|| now.0);
    let mut said = use_signal(String::new);
    let all = rows();
    let n = act_counts(&all);
    let view = act_view(&all, tab(), &q());
    let cur = sel().and_then(|id| all.iter().find(|a| a.id == id).cloned());
    let el_s = since(now, start()) / 1000;
    let mut undo = move |a: Act| {
        if let Some(x) = rows.write().iter_mut().find(|x| x.id == a.id) {
            x.undone = true;
        }
        said.set(format!("Undone: {}", a.text));
    };
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(a) = cur.clone() {
                {wtab("doc", a.text.to_string(), sel)}
                div { class: "oxa-wbody",
                    div { class: "oxay-hd",
                        span { class: "oxay-logo oxay-logo-l", BrandIcon { name: a.app.to_string(), size: 22.0, coloured: true, fallback: true } }
                        span { class: "oxak-rt", b { class: "oxay-ht", "{a.text}" } span { class: "oxv-q oxv-ell", "{a.app_name} · {PERSON_EMAIL}" } }
                    }
                    div { class: "oxa-kvs",
                        {kv("When", rsx! { span { if a.day == 0 { "Today {a.at}" } else { "Yesterday {a.at}" } } span { class: "oxv-q", "· {age(a.mins)} ago" } })}
                        {kv("Task", rsx! { button { r#type: "button", class: "oxv-link", onclick: move |_| said.set(format!("Opened {}", a.task)), "{a.task}" } })}
                        {kv("Cost", rsx! { "{eur(a.cents)}" })}
                    }
                    h3 { class: "oxa-h3", "What happened" }
                    div { class: "oxa-card",
                        for (k, v) in a.what.iter().copied() { div { key: "{k}", class: "oxa-kv", span { class: "oxa-k", "{k}" } span { class: "oxa-v oxv-ell", "{v}" } } }
                        if !a.body.is_empty() { p { class: "oxay-body", "{a.body}" } }
                    }
                    h3 { class: "oxa-h3", "Why it could" }
                    div { class: "oxa-card",
                        p { class: "oxay-why", {dot(if a.mode == Mode::Approved { "warn" } else { "run" })} b { "{a.mode.label()}" } }
                        p { class: "oxa-mute oxay-wl", if a.mode == Mode::Approved { "You approved it {age(a.mins + 2)} ago." } else { "Your permissions let it do this without asking." } }
                        div { class: "oxay-rule", {nico("shield")} span { class: "oxv-ell", "{a.rule}" } span { class: "oxscn-grow" }
                            button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Opened Permissions".into()), "Permissions" } }
                    }
                    h3 { class: "oxa-h3", "Taking it back" }
                    {
                        let left = undo_left(&a, el_s);
                        let words = if a.undone { "Already undone".to_string() } else if a.undo_min == 0 { "This one is final".to_string() } else if clock { undo_clock(left) } else { undo_words(left) };
                        let a2 = a.clone();
                        rsx! {
                            div { class: "oxa-card oxay-back",
                                span { class: "oxak-rt", b { class: "oxay-bt", "{a.undo_does}" } span { class: "oxv-q", role: "timer", "aria-live": "off", "{words}" } }
                                button { r#type: "button", class: "oxv-pri", disabled: left == 0, onclick: move |_| undo(a2.clone()), {nico("undo")} span { "Undo" } }
                            }
                        }
                    }
                }
            } else {
                p { class: "oxa-mute", "Pick a step to see what happened and how to take it back." }
            }
        }
    };
    let tab_items: Vec<(&'static str, String, Option<usize>)> = ACT_TABS
        .iter()
        .enumerate()
        .map(|(i, (k, l))| (*k, l.to_string(), Some(n[i])))
        .collect();
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "activity", title: "Activity",
            badge: rsx! { {tabs("Filter", tab_items, tab)} },
            actions: rsx! { {find("Search activity", "Search activity", q, "oxay-q")} },
            work, work_label: "Step", work_w: 388,
            div { class: "oxay",
                if loading { {sk_rows(9)} }
                else if all.is_empty() {
                    {none("pulse", "Nothing done yet", "Every email, refund or post an agent makes shows here, with a way to take it back.")}
                } else if view.is_empty() {
                    p { class: "oxa-mute", role: "status", "Nothing matches." }
                } else {
                    for (day, items) in view {
                        section { key: "{day}", class: "oxay-day", "aria-label": "{day}",
                            h3 { class: "oxay-dh", "{day}" span { class: "oxa-n", "{items.len()}" } }
                            ul { class: "oxay-ul",
                                for a in items {
                                    {
                                        let id = a.id;
                                        let on = sel() == Some(id);
                                        let left = undo_left(&a, el_s);
                                        let a2 = a.clone();
                                        rsx! {
                                            li { key: "{id}", class: "oxay-row", "data-on": "{on}", "data-undone": "{a.undone}",
                                                button { r#type: "button", class: "oxay-sel", "aria-current": if on { "true" } else { "false" }, onclick: move |_| sel.set(Some(id)),
                                                    span { class: "oxay-at", "{a.at}" }
                                                    span { class: "oxay-logo", BrandIcon { name: a.app.to_string(), size: 16.0, coloured: true, fallback: true } }
                                                    span { class: "oxak-rt",
                                                        span { class: "oxv-ell oxay-t", "{a.text}" }
                                                        span { class: "oxay-sub", {dot(if a.mode == Mode::Approved { "warn" } else { "run" })} span { class: "oxv-ell", "{a.mode.label()}" } span { class: "oxv-q oxv-ell", "· {a.task}" } }
                                                    }
                                                    if cost { span { class: "oxak-c", "{eur(a.cents)}" } }
                                                    if a.undone { span { class: "oxay-tag", "Undone" } }
                                                    else if left == 0 { span { class: "oxay-final", {nico("lock")} "Final" } }
                                                }
                                                if !a.undone && left > 0 {
                                                    button { r#type: "button", class: "oxscn-btn oxay-undo", "aria-label": "Undo: {a.text}", onclick: move |_| undo(a2.clone()), {nico("undo")} span { "Undo" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Memory ──

fn memo_glyph(from: &str) -> &'static str {
    match from {
        "thread" => "chat",
        "you" => "user",
        "decision" => "inbox",
        _ => "doc",
    }
}

/// Screen – Agent / Memory: what the agents remember, grouped by scope (About you and each
/// project), each with where it came from, how often it was used and when. Saved / To review tabs,
/// a search and Remember; the work area shows the memory with where it came from, where it was
/// used and Forget (with Undo). States: `sample`, `loading`, `empty`, `review`.
#[component]
pub fn MemoryScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: a scope menu that narrows to one group.
    #[props(default)]
    scope_menu: bool,
    /// Oxyn: hide the source and use columns.
    #[props(default)]
    compact: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut list = use_signal(move || memory(sample));
    let mut tab = use_signal(move || if sk == "review" { "review" } else { "saved" });
    let q = use_signal(String::new);
    let mut scope = use_signal(String::new);
    let mut sel = use_signal(move || {
        if sample {
            Some(if sk == "review" { 12u32 } else { 3 })
        } else {
            None
        }
    });
    let mut shut = use_signal(Vec::<&'static str>::new);
    let mut adding = use_signal(|| false);
    let mut new_t = use_signal(String::new);
    let mut gone = use_signal(|| None::<Memo>);
    let mut said = use_signal(String::new);
    let all = list();
    let (ns, nr) = memo_counts(&all);
    let groups: Vec<_> = memo_groups(&all, tab() == "review", &q())
        .into_iter()
        .filter(|g| scope().is_empty() || g.0.0 == scope())
        .collect();
    let cur = sel().and_then(|id| all.iter().find(|m| m.id == id).cloned());
    let mut forget = move |m: Memo| {
        list.write().retain(|x| x.id != m.id);
        sel.set(None);
        said.set(format!("Forgot: {}", m.text));
        gone.set(Some(m));
    };
    let mut keep = move |id: u32| {
        if let Some(x) = list.write().iter_mut().find(|x| x.id == id) {
            x.review = false;
        }
        said.set("Saved to memory".into());
    };
    let mut add = move || {
        let t = new_t.peek().clone();
        if let Some(id) = remember(&mut list.write(), &t) {
            tab.set("saved");
            sel.set(Some(id));
            new_t.set(String::new());
            adding.set(false);
            said.set("Remembered".into());
        }
    };
    let mut scopes: Vec<(&'static str, &'static str, &'static str)> = vec![("", "All scopes", "")];
    for p in oxyn_core::parity::agentscr::PROJECTS {
        scopes.push((p.0, p.1, ""));
    }
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(m) = cur.clone() {
                {wtab("brain", m.text.to_string(), sel)}
                div { class: "oxa-wbody",
                    h2 { class: "oxa-h2", "{m.text}" }
                    div { class: "oxa-kvs",
                        {kv("Applies to", rsx! { span { class: "oxak-fg", "data-hue": "{project(m.scope).2}", {nico(if m.scope == "you" { "user" } else { "folder" })} } span { if m.scope == "you" { "Everywhere" } else { "{project(m.scope).1}" } } })}
                        {kv("Learned", rsx! { {nico(memo_glyph(m.from))} span { "From {m.source}" } span { class: "oxv-q", "· 16 days ago" } })}
                        {kv("Used", rsx! { span { "{m.uses} times" } span { class: "oxv-q", "· last {age(m.mins)} ago" } })}
                    }
                    h3 { class: "oxa-h3", "Where it came from" }
                    div { class: "oxa-card",
                        p { class: "oxa-mute", "Should the weekly note go out now?" }
                        p { class: "oxam-quote", "Not after three on a Friday. Monday at 9:00 instead." }
                        div { class: "oxay-rule", {nico("inbox")} span { class: "oxv-q oxv-ell", "You, in the Inbox · 16 days ago" } span { class: "oxscn-grow" }
                            button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Opened the source".into()), "Open" } }
                    }
                    h3 { class: "oxa-h3", "Where it was used " span { class: "oxv-q", "2" } }
                    div { class: "oxa-card oxam-used",
                        for (t, l) in [("Weekly note", "Held until Monday 9:00 · 2 days ago"), ("Client dashboard", "Moved the launch email to Monday · 9 days ago")] {
                            div { key: "{t}", class: "oxam-u", {nico("tasks")} span { class: "oxak-rt", b { "{t}" } span { class: "oxv-q oxv-ell", "{l}" } } }
                        }
                    }
                    div { class: "oxa-acts",
                        if m.review { button { r#type: "button", class: "oxv-pri", onclick: move |_| keep(m.id), "Keep" } }
                        button { r#type: "button", class: "oxscn-btn oxv-ghost", onclick: move |_| forget(m.clone()), {nico("trash")} span { "Forget" } }
                    }
                }
            } else {
                p { class: "oxa-mute", "Pick a memory to see where it came from and where it was used." }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "memory", title: "Memory",
            badge: rsx! { span { class: "oxa-n", "{ns}" } {tabs("Show", vec![("saved", "Saved".to_string(), None), ("review", "To review".to_string(), if nr > 0 { Some(nr) } else { None })], tab)} },
            actions: rsx! {
                if scope_menu {
                    MenuButton { label: if scope().is_empty() { "All scopes".to_string() } else { project(&scope()).1.to_string() }, aria: "Scope", icon: "filter", right: true, caret: true, items: scopes, checked: scope(), on_pick: move |k: String| scope.set(k) }
                }
                {find("Search memory", "Search memory", q, "oxam-q")}
                div { class: "oxv-pop-w",
                    button { r#type: "button", class: "oxscn-btn", "aria-label": "Remember", "aria-haspopup": "dialog", "aria-expanded": "{adding}", onclick: move |_| adding.toggle(), {nico("plus")} span { class: "oxa-bl", "Remember" } }
                    if adding() {
                        div { class: "oxscn-catch", onclick: move |_| adding.set(false) }
                        div { class: "oxv-pop", role: "dialog", "aria-label": "Remember",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false) },
                            p { class: "oxv-pop-h", "What should agents remember?" }
                            input { class: "oxv-in", r#type: "text", placeholder: "One fact or preference", "aria-label": "Memory", value: "{new_t}", onmounted: focus_on,
                                oninput: move |e| new_t.set(e.value()), onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { add() } }
                            div { class: "oxv-pop-a",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                                button { r#type: "button", class: "oxv-pri", disabled: new_t().trim().is_empty(), onclick: move |_| add(), "Remember" }
                            }
                        }
                    }
                }
            },
            work, work_label: "Memory", work_w: 368,
            div { class: "oxam", "data-compact": "{compact}",
                if loading { {sk_rows(10)} }
                else if all.is_empty() {
                    {none("brain", "Nothing remembered yet", "Agents keep what you tell them and what you decide, and show you here where each fact was used.")}
                } else if groups.is_empty() {
                    p { class: "oxa-mute", role: "status", if tab() == "review" { "Nothing to review." } else { "Nothing matches." } }
                } else {
                    for ((pk, pn, hue), items) in groups {
                        {
                            let open = !shut().contains(&pk);
                            let n = items.len();
                            rsx! {
                                section { key: "{pk}", class: "oxam-g", "aria-label": "{pn}",
                                    button { r#type: "button", class: "oxam-gh", "aria-expanded": "{open}",
                                        onclick: move |_| { let mut v = shut.write(); if let Some(p) = v.iter().position(|x| *x == pk) { v.remove(p); } else { v.push(pk); } },
                                        span { class: "oxak-fg", "data-hue": "{hue}", {nico(if pk == "you" { "user" } else { "folder" })} } b { "{pn}" } span { class: "oxa-n", "{n}" } span { class: "oxscn-grow" } {chevron(open)} }
                                    if open {
                                        ul { class: "oxam-ul",
                                            for m in items {
                                                {
                                                    let id = m.id;
                                                    let on = sel() == Some(id);
                                                    rsx! {
                                                        li { key: "{id}", class: "oxam-row", "data-on": "{on}",
                                                            button { r#type: "button", class: "oxam-sel", "aria-current": if on { "true" } else { "false" }, onclick: move |_| sel.set(Some(id)),
                                                                span { class: "oxam-g2", {nico(memo_glyph(m.from))} }
                                                                span { class: "oxv-ell oxam-t", "{m.text}" }
                                                                if !compact {
                                                                    span { class: "oxam-src oxv-ell", "{m.source}" }
                                                                    span { class: "oxak-c", "{m.uses}×" }
                                                                }
                                                                span { class: "oxak-c oxak-age", "{age(m.mins)}" }
                                                            }
                                                            if m.review {
                                                                button { r#type: "button", class: "oxscn-btn oxam-keep", onclick: move |_| keep(id), "Keep" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(m) = gone() {
                    div { class: "oxa-toast", role: "status", {nico("trash")} span { class: "oxv-ell", "Forgot it" }
                        button { r#type: "button", class: "oxv-link", onclick: move |_| { let id = m.id; list.write().insert(0, m.clone()); sel.set(Some(id)); gone.set(None); said.set("Memory restored".into()); }, "Undo" }
                        button { r#type: "button", class: "oxa-x", "aria-label": "Dismiss", onclick: move |_| gone.set(None), {nico("close")} } }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}
