//! Composer parts (plan #27, chunk 3): voice recorder, dictation button, mention input, mode
//! switch, permission selector, tools menu, queue stack and agent steering.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::parts::{Suggestion, Tool},
};
use oxyn_dioxus::{
    AgentSteering, DictationButton, MentionInput, ModeSwitch, PermissionSelector, QueueStack,
    ToolsMenu, VoiceRecorder,
};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

/// A key that changes with every control, so a new variant remounts the part with its props.
fn key(v: &Vals, keys: &[&str]) -> String {
    keys.iter().map(|k| v.s(k)).collect::<Vec<_>>().join("|")
}

fn people() -> Vec<Suggestion> {
    vec![
        Suggestion::new("maya", "Maya Chen", "People", "Backend lead"),
        Suggestion::new("sam", "Sam Ortiz", "People", "On call"),
        Suggestion::new("priya", "Priya Rao", "People", "Design"),
        Suggestion::new("src/auth/token.rs", "token.rs", "Files", "src/auth"),
        Suggestion::new("tests/auth_refresh.rs", "auth_refresh.rs", "Files", "tests"),
        Suggestion::new("reviewer", "Reviewer", "Agents", "Reads the diff"),
        Suggestion::new("tester", "Test runner", "Agents", "Runs cargo test"),
    ]
}

fn commands() -> Vec<Suggestion> {
    vec![
        Suggestion::new("review", "/review", "Commands", "Review the open diff"),
        Suggestion::new("test", "/test", "Commands", "Run the test suite"),
        Suggestion::new("plan", "/plan", "Commands", "Plan before editing"),
        Suggestion::new("compact", "/compact", "Commands", "Summarise the context"),
        Suggestion::new("clear", "/clear", "Commands", "Start a new thread"),
    ]
}

fn tools(auth: bool) -> Vec<Tool> {
    let mut v = vec![
        Tool::new("read", "Read files", "Workspace", true),
        Tool::new("edit", "Edit files", "Workspace", true),
        Tool::new("shell", "Run commands", "Workspace", true),
        Tool::new("search", "Web search", "Web", true),
        Tool::new("fetch", "Fetch pages", "Web", false),
        Tool::new("browser", "Browser", "Web", false),
    ];
    let gh = Tool::new("github", "GitHub", "Connectors", false);
    let linear = Tool::new("linear", "Linear", "Connectors", false);
    if auth {
        v.push(gh.auth());
        v.push(linear.auth());
    } else {
        v.push(gh);
        v.push(linear);
    }
    v
}

fn queue_items(n: usize) -> Vec<String> {
    [
        "Also bump the jsonwebtoken crate",
        "Then rerun the auth suite three times",
        "Open a draft PR when it passes",
        "Ping Maya for review",
        "Update the changelog",
    ]
    .iter()
    .take(n)
    .map(|s| s.to_string())
    .collect()
}

fn voice(v: &Vals, now: ClockMs) -> Element {
    let k = key(v, &["phase", "variant", "max", "deny", "bars"]);
    rsx! {
        div { key: "{k}", style: "width:100%;display:flex;justify-content:center",
            VoiceRecorder { now_ms: now.0, phase: v.string("phase"), variant: v.string("variant"), max_s: v.f("max") as u64, deny: v.b("deny"), bars: v.u("bars") }
        }
    }
}

fn voice_code(v: &Vals) -> String {
    Code::new("VoiceRecorder")
        .raw("now_ms", "clock.now()")
        .s("variant", v.s("variant"), "inline")
        .raw("max_s", format!("{}", v.f("max") as u64))
        .raw("on_send", "move |ms: u64| upload(ms)")
        .done()
}

fn dictation(v: &Vals, now: ClockMs) -> Element {
    let k = key(v, &["variant", "wpm", "fail", "seed"]);
    let value = if v.b("seed") {
        "Fix the flaky auth test: ".to_string()
    } else {
        String::new()
    };
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:520px",
            DictationButton { now_ms: now.0, variant: v.string("variant"), wpm: v.f("wpm") as f32, fail: v.b("fail"), value }
        }
    }
}

fn dictation_code(v: &Vals) -> String {
    Code::new("DictationButton")
        .raw("now_ms", "clock.now()")
        .s("variant", v.s("variant"), "field")
        .raw("on_change", "move |text: String| draft.set(text)")
        .done()
}

fn mention(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["value", "limit", "up", "cmds"]);
    let value = match v.s("value") {
        "@ma" => "Ask @ma".to_string(),
        "/" => "/".to_string(),
        "pills" => "@maya can you check @src/auth/token.rs ".to_string(),
        _ => String::new(),
    };
    let cmds = if v.b("cmds") { commands() } else { vec![] };
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:560px",
            MentionInput { suggestions: people(), commands: cmds, value, limit: v.u("limit"), up: v.b("up") }
        }
    }
}

fn mention_code(v: &Vals) -> String {
    Code::new("MentionInput")
        .raw("suggestions", "people_files_agents")
        .raw("commands", if v.b("cmds") { "commands" } else { "vec![]" })
        .b("up", v.b("up"))
        .raw("on_submit", "move |text: String| send(text)")
        .done()
}

fn mode(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["variant", "value", "busy", "preview"]);
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:560px",
            ModeSwitch { variant: v.string("variant"), value: v.string("value"), busy: v.b("busy"), preview: v.b("preview") }
        }
    }
}

fn mode_code(v: &Vals) -> String {
    Code::new("ModeSwitch")
        .s("variant", v.s("variant"), "segmented")
        .s("value", v.s("value"), "agent")
        .b("busy", v.b("busy"))
        .raw(
            "on_change",
            "move |(mode, now): (String, bool)| set_mode(mode, now)",
        )
        .done()
}

fn perm(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["variant", "value", "ceiling", "open"]);
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:560px",
            PermissionSelector { variant: v.string("variant"), value: v.string("value"), ceiling: v.string("ceiling"), open: v.b("open") }
        }
    }
}

fn perm_code(v: &Vals) -> String {
    Code::new("PermissionSelector")
        .s("variant", v.s("variant"), "menu")
        .s("value", v.s("value"), "ask")
        .s("ceiling", v.s("ceiling"), "high")
        .raw("on_change", "move |id: String| policy.set(id)")
        .done()
}

fn tools_demo(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["open", "groups", "search", "auth"]);
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:420px",
            ToolsMenu { tools: tools(v.b("auth")), open: v.b("open"), group_toggles: v.b("groups"), search: v.b("search") }
        }
    }
}

fn tools_code(v: &Vals) -> String {
    Code::new("ToolsMenu")
        .raw(
            "tools",
            "vec![Tool::new(\"read\", \"Read files\", \"Workspace\", true)]",
        )
        .b("open", v.b("open"))
        .raw("on_change", "move |ids: Vec<String>| enabled.set(ids)")
        .done()
}

fn queue(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["n", "busy", "paused", "rows", "turn"]);
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:560px",
            QueueStack { items: queue_items(v.u("n")), busy: v.b("busy"), paused: v.b("paused"), max_rows: v.u("rows"), turn_ms: v.f("turn") as u64 }
        }
    }
}

fn queue_code(v: &Vals) -> String {
    Code::new("QueueStack")
        .raw("items", "queued")
        .b("paused", v.b("paused"))
        .raw("max_rows", v.s("rows").to_string())
        .raw(
            "on_send",
            "move |(text, now): (String, bool)| agent.send(text, now)",
        )
        .done()
}

fn steer(v: &Vals, now: ClockMs) -> Element {
    let k = key(v, &["step", "notes", "interrupt"]);
    let notes = if v.b("notes") {
        vec![
            ("Skip the docs folder".to_string(), 9_000),
            ("Use a 30 s leeway, not 60".to_string(), 2_500),
        ]
    } else {
        vec![]
    };
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:560px",
            AgentSteering { now_ms: now.0, step_ms: v.f("step") as u64, notes, interrupt: v.b("interrupt") }
        }
    }
}

fn steer_code(v: &Vals) -> String {
    Code::new("AgentSteering")
        .raw("now_ms", "clock.now()")
        .b("interrupt", v.b("interrupt"))
        .raw(
            "on_steer",
            "move |(note, interrupt): (String, bool)| agent.steer(note, interrupt)",
        )
        .done()
}

/// Chunk-3 demos.
pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel(
                    "phase",
                    "Start in",
                    &["idle", "recording", "paused", "review", "denied"],
                    "idle",
                ),
                sel("variant", "Variant", &["inline", "card"], "inline"),
                rng("max", "Limit s", 10.0, 300.0, 5.0, "120"),
                rng("bars", "Bars", 12.0, 48.0, 2.0, "28"),
                tog("deny", "Permission denied", false),
            ],
            variants: vec![
                var("idle", "A mic in the composer row.", &[]),
                var(
                    "recording",
                    "Live level bars, timer and pause.",
                    &[("phase", "recording")],
                ),
                var("review", "Play, discard or send.", &[("phase", "review")]),
                var(
                    "card",
                    "Card variant near its limit.",
                    &[("variant", "card"), ("phase", "recording"), ("max", "15")],
                ),
                var("denied", "The microphone was refused.", &[("deny", "true")]),
            ],
            render: voice,
            code: voice_code,
            keyboard: vec![
                ("Enter / Space", "Start, pause, resume, stop or send"),
                ("Tab", "Pause, stop, discard, send"),
            ],
            a11y: vec![
                "Every button names its action; the bars are aria-hidden.",
                "A polite region announces start, pause, the last ten seconds and send.",
            ],
            responsive: vec!["The inline variant fits a 320 px composer row."],
            height: 260,
            live: true,
            ..mk(
                "voice-recorder",
                "Voice recorder",
                "composer",
                "Record a voice message in the composer: permission, live level bars, a timer that counts down near the limit, pause, then play, discard or send.",
                "VoiceRecorder",
                "VoiceRecorderEl",
                "composer-voice-input",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["field", "pill"], "field"),
                rng("wpm", "Words per minute", 60.0, 300.0, 10.0, "150"),
                tog("seed", "Existing text", false),
                tog("fail", "Recognition fails", false),
            ],
            variants: vec![
                var("field", "Mic inside the field; words stream in grey.", &[]),
                var(
                    "pill",
                    "Round mic beside the field.",
                    &[("variant", "pill")],
                ),
                var(
                    "insert",
                    "Dictation inserts at the caret.",
                    &[("seed", "true")],
                ),
                var("error", "Recognition failed.", &[("fail", "true")]),
            ],
            render: dictation,
            code: dictation_code,
            keyboard: vec![
                ("Enter / Space on the mic", "Start or stop dictation"),
                ("Escape", "Cancel and drop the interim words"),
            ],
            a11y: vec![
                "The mic button is named for its action: “Dictate” or “Stop dictation”.",
                "Interim words are aria-hidden; a polite region announces listening, done and cancel.",
            ],
            responsive: vec!["The field grows to its container; the mic never wraps."],
            height: 220,
            live: true,
            ..mk(
                "dictation-button",
                "Dictation button",
                "composer",
                "Speech to text in the field: a mic that listens, interim words in grey that settle when final, inserted at the caret, with cancel and an error state.",
                "DictationButton",
                "DictationEl",
                "composer-voice-input",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "value",
                    "Start with",
                    &["empty", "@ma", "/", "pills"],
                    "@ma",
                ),
                rng("limit", "Rows", 3.0, 10.0, 1.0, "6"),
                tog("up", "Open upward", false),
                tog("cmds", "Slash commands", true),
            ],
            variants: vec![
                var("mention", "@ suggestions grouped by kind.", &[]),
                var(
                    "commands",
                    "/ commands at the line start.",
                    &[("value", "/")],
                ),
                var(
                    "pills",
                    "Accepted mentions render as pills.",
                    &[("value", "pills")],
                ),
                var(
                    "upward",
                    "Opens above, for a bottom composer.",
                    &[("up", "true"), ("value", "@ma")],
                ),
            ],
            render: mention,
            code: mention_code,
            keyboard: vec![
                ("↑ / ↓", "Move through suggestions"),
                ("Enter / Tab", "Insert the suggestion"),
                ("Escape", "Close the list"),
                ("Enter", "Send when the list is closed"),
            ],
            a11y: vec![
                "The field is a combobox with aria-activedescendant on the open listbox.",
                "Options carry aria-selected; a polite region announces the result count.",
            ],
            responsive: vec!["The list matches the field width, capped at 360 px."],
            height: 380,
            align: Align::Start,
            ..mk(
                "mention-input",
                "Mention input",
                "composer",
                "Type @ to mention people, files and agents or / for commands: ranked, grouped suggestions at the caret, keyboard picking, and pills for accepted mentions.",
                "MentionInput",
                "MentionListEl",
                "composer-slash-mentions",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["segmented", "menu"], "segmented"),
                sel("value", "Mode", &["ask", "plan", "agent"], "agent"),
                tog("busy", "Agent running", false),
                tog("preview", "Composer preview", true),
            ],
            variants: vec![
                var("segmented", "Ask · Plan · Agent segments.", &[]),
                var(
                    "menu",
                    "A compact chip with a list.",
                    &[("variant", "menu")],
                ),
                var(
                    "busy",
                    "Mid-run: the change waits for the next turn.",
                    &[("busy", "true")],
                ),
                var(
                    "plan",
                    "Plan mode changes the placeholder.",
                    &[("value", "plan")],
                ),
            ],
            render: mode,
            code: mode_code,
            keyboard: vec![
                ("Tab / Enter", "Reach and choose a mode"),
                ("Shift + Tab in the preview", "Cycle modes"),
                ("Escape", "Close the menu variant"),
            ],
            a11y: vec![
                "Segments are a radiogroup with aria-checked; the menu is a listbox.",
                "A queued change is announced: “Plan from the next turn”.",
            ],
            responsive: vec!["Use the menu variant where the segments do not fit."],
            height: 240,
            ..mk(
                "mode-switch",
                "Mode switch",
                "composer",
                "Switch the agent between Ask, Plan and Agent: segmented or as a menu, a placeholder that follows the mode, and a change that waits for the next turn while it runs.",
                "ModeSwitch",
                "ModeSwitchEl",
                "composer-mode-switch",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["menu", "cards"], "menu"),
                sel("value", "Preset", &["read", "ask", "auto", "full"], "ask"),
                sel(
                    "ceiling",
                    "Policy allows up to",
                    &["low", "medium", "high"],
                    "high",
                ),
                tog("open", "Open", false),
            ],
            variants: vec![
                var("menu", "A chip that opens the presets.", &[]),
                var("open", "Open with risk on each row.", &[("open", "true")]),
                var("cards", "Presets as cards.", &[("variant", "cards")]),
                var(
                    "policy",
                    "Workspace policy blocks high risk.",
                    &[("variant", "cards"), ("ceiling", "medium")],
                ),
            ],
            render: perm,
            code: perm_code,
            keyboard: vec![
                ("Enter / Space", "Open the presets or choose one"),
                ("Tab", "Move through presets"),
                ("Escape", "Close, or cancel a confirm"),
            ],
            a11y: vec![
                "Presets are a radiogroup; blocked ones are aria-disabled with the reason.",
                "Full access asks in an alertdialog before it applies.",
            ],
            responsive: vec!["Cards wrap to one column in narrow frames."],
            height: 340,
            align: Align::Start,
            ..mk(
                "permission-selector",
                "Permission selector",
                "composer",
                "How much the agent may do without asking: Read only, Ask before edits, Auto-edit or Full access, with risk levels, a confirm for full access and a workspace ceiling.",
                "PermissionSelector",
                "PermissionEl",
                "composer-permission-selector",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Open", true),
                tog("groups", "Group switches", true),
                tog("search", "Search", true),
                tog("auth", "Connectors need sign-in", true),
            ],
            variants: vec![
                var("open", "Grouped tools with switches.", &[]),
                var(
                    "closed",
                    "The tools chip in the composer.",
                    &[("open", "false")],
                ),
                var(
                    "minimal",
                    "No search or group switches.",
                    &[("search", "false"), ("groups", "false")],
                ),
                var(
                    "connected",
                    "Connectors already signed in.",
                    &[("auth", "false")],
                ),
            ],
            render: tools_demo,
            code: tools_code,
            keyboard: vec![
                ("Tab", "Move through tools"),
                ("Enter / Space", "Toggle a tool"),
                ("Escape", "Close the menu"),
            ],
            a11y: vec![
                "Each tool is a switch (role switch, aria-checked); groups are labelled.",
                "Group buttons read “Turn all on” or “Turn all off”; changes are announced politely.",
            ],
            responsive: vec!["The menu scrolls past 360 px of height."],
            height: 560,
            align: Align::Start,
            ..mk(
                "tools-menu",
                "Tools menu",
                "composer",
                "Choose which tools the agent may use: grouped switches, group toggles, a filter, and connectors that ask you to sign in first.",
                "ToolsMenu",
                "ToolsMenuEl",
                "composer-tools-menu",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                rng("n", "Queued", 0.0, 5.0, 1.0, "3"),
                rng("rows", "Collapse after", 0.0, 5.0, 1.0, "3"),
                rng("turn", "Turn ms", 2000.0, 12000.0, 500.0, "6000"),
                tog("busy", "Agent running", true),
                tog("paused", "Paused", false),
            ],
            variants: vec![
                var("queue", "Messages wait for the turn to end.", &[]),
                var(
                    "collapsed",
                    "Long queues collapse.",
                    &[("n", "5"), ("rows", "2")],
                ),
                var("paused", "Queue paused.", &[("paused", "true")]),
                var(
                    "idle",
                    "Agent idle: the next item sends.",
                    &[("busy", "false")],
                ),
            ],
            render: queue,
            code: queue_code,
            keyboard: vec![
                ("Tab", "Each row: send now, edit, remove"),
                ("Alt + ↑ / ↓", "Reorder a row"),
                ("Enter / Escape", "Save or cancel an edit"),
            ],
            a11y: vec![
                "The stack is a list labelled “Queued messages”.",
                "Sending, removing and reordering are announced politely.",
            ],
            responsive: vec!["Rows truncate to one line; edit opens the full text."],
            height: 320,
            align: Align::End,
            live: true,
            ..mk(
                "queue-stack",
                "Queue stack",
                "composer",
                "Messages typed while the agent works wait above the composer: they send in order as turns end, and each can be sent now, edited, reordered or removed.",
                "QueueStack",
                "QueueStackEl",
                "composer-queue-stack",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                rng("step", "Step ms", 1000.0, 8000.0, 500.0, "3000"),
                tog("notes", "Earlier notes", true),
                tog("interrupt", "Interrupt by default", false),
            ],
            variants: vec![
                var("steer", "Notes the agent picks up at its next step.", &[]),
                var("empty", "No notes yet.", &[("notes", "false")]),
                var(
                    "interrupt",
                    "Notes stop the current step.",
                    &[("interrupt", "true")],
                ),
            ],
            render: steer,
            code: steer_code,
            keyboard: vec![
                ("Enter", "Send a note at the next step"),
                ("Ctrl / ⌘ + Enter", "Send and interrupt now"),
            ],
            a11y: vec![
                "The current step is a status region; picked-up notes are announced.",
                "Interrupt is a toggle (aria-pressed).",
            ],
            responsive: vec!["The step line truncates; the note field keeps its width."],
            height: 340,
            live: true,
            ..mk(
                "agent-steering",
                "Agent steering",
                "composer",
                "Guide a running agent without stopping it: notes queue for its next step and show when picked up, or interrupt the current step.",
                "AgentSteering",
                "SteeringEl",
                "composer-queue-stack",
                "composer/parts.rs",
            )
        },
    ]
}
