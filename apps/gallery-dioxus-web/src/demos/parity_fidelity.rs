//! Plan #32 (affectUI fidelity pass) demos: the exact Chat Composer with every affectUI variant
//! (and Oxyn's own AI composer as an extra), Comment Thread and Comments Panel.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{chatfid::RunTurn, fidelity::PanelComment},
};
use oxyn_dioxus::{
    AgentCommentThread, AgentRunPanel, AiComposer, ChatComposer, CommentsPanel, MenuModel,
    ModelMenu, ModelPickerMenu, RunControls, ThreadNote,
};

use super::{Code, mk, sel, tog, var};
use crate::{Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

// ---------- chat composer ----------

fn chat_composer(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    if v.s("kind") == "oxyn" {
        return rsx! {
            div { style: "width:100%;display:flex;flex-direction:column;align-items:center;gap:8px;padding:40px 0 60px",
                div { style: "width:560px;max-width:100%",
                    AiComposer { now,
                        models: vec![
                            ("opus".to_string(), "Opus 5.5".to_string(), "Most capable".to_string()),
                            ("sonnet".to_string(), "Sonnet 5.5".to_string(), "Fast and capable".to_string()),
                        ],
                        suggestions: vec![("Summarize the thread".to_string(), "/summarize the last 20 messages".to_string()), ("Draft a reply".to_string(), "Draft a reply about the launch".to_string())],
                        mentions: vec!["Q3 report.pdf".into(), "roadmap.md".into(), "api/rate_limit.rs".into()],
                    }
                }
            }
        };
    }
    let open = match v.s("open") {
        "none" => None,
        o => Some(o.to_string()),
    };
    // Room above for the drop-ups (plus, approval, model) and below for the tray menus.
    rsx! {
        div { style: "width:680px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:10px;padding:270px 0 166px",
            ChatComposer { now, style: if v.s("style") == "classic" { "classic".to_string() } else { String::new() }, approval: v.string("approval"), tray: v.b("tray"), recording: v.b("recording"),
                attachments: if v.b("attach") { vec!["pdf".to_string(), "image".to_string(), "sheet".to_string()] } else { vec![] },
                default_open: open,
                on_send: move |(t, m): (String, String)| msg.set(format!("onSend({t:?}, {{ model: {m:?} }})")),
                on_model_change: move |m: String| msg.set(format!("onModelChange({m:?})")),
                on_attach: move |a: String| msg.set(format!("onAttach({a:?})")),
                on_approval_change: move |a: String| msg.set(format!("onApprovalChange({a:?})")),
                on_folder_change: move |f: Option<String>| msg.set(format!("onFolderChange({f:?})")),
                on_apps_change: move |a: Vec<String>| msg.set(format!("onAppsChange({a:?})")),
            }
            {status(msg)}
        }
    }
}

fn chat_composer_code(v: &Vals) -> String {
    if v.s("kind") == "oxyn" {
        return Code::new("AiComposer").raw("now", "clock.now()").done();
    }
    let mut c = Code::new("ChatComposer").raw("now", "clock.now()");
    if v.s("style") == "classic" {
        c = c.raw("style", "\"classic\".into()");
    }
    if v.s("approval") != "ask" {
        c = c.raw("approval", format!("{:?}.into()", v.s("approval")));
    }
    if !v.b("tray") {
        c = c.raw("tray", "false");
    }
    c.raw("on_send", "move |(text, model)| send(text, model)")
        .raw("on_attach", "move |kind| open_picker(kind)")
        .raw("on_apps_change", "move |ids| save_connections(ids)")
        .done()
}

// ---------- comment thread ----------

fn notes() -> Vec<ThreadNote> {
    vec![
        ThreadNote::person(
            "Mara Lind",
            0,
            12,
            "/Edit Rename every “customers” to “members” on the pricing page.",
        ),
        ThreadNote::agent(
            "Atlas",
            10,
            "Edit completed",
            &[
                "Renamed 12 mentions in 3 sections",
                "Kept “customer support” as it is",
                "Left the legal pages untouched",
            ],
        ),
    ]
}

fn comment_thread(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:440px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:8px",
            AgentCommentThread { notes: notes(), now, open: v.b("open"), avatars: v.string("avatars"),
                answer: if v.b("answer") { "Done – I also updated the meta description.".to_string() } else { String::new() },
                on_reply: move |t: String| msg.set(format!("onReply({t:?})")),
                on_close: move |_| msg.set("onClose()".into()),
            }
            {status(msg)}
        }
    }
}

fn comment_thread_code(_: &Vals) -> String {
    Code::new("AgentCommentThread")
        .raw("notes", "thread.notes()")
        .raw("now", "clock.now()")
        .raw("on_reply", "move |text| api.reply(text)")
        .raw("on_close", "move |_| close()")
        .done()
}

// ---------- comments panel ----------

fn comments() -> Vec<PanelComment> {
    vec![
        PanelComment::new(
            1,
            "Mara Lind",
            0,
            10,
            "/Edit Rename every “customers” to “members” on the pricing page.",
        ),
        PanelComment::new(
            2,
            "Theo Marsh",
            1,
            26,
            "/Edit Shorten every FAQ answer to two sentences.",
        ),
        PanelComment::new(
            3,
            "Priya Sen",
            2,
            34,
            "Can we keep the old plan names for existing members?",
        ),
        PanelComment::new(
            4,
            "Noor Ali",
            4,
            52,
            "The hero image finally looks right on mobile.",
        ),
        PanelComment::new(5, "Ines Duarte", 5, 61, "Can we ship this before Friday?"),
    ]
}

fn comments_panel(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:420px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:8px;padding-bottom:40px",
            CommentsPanel { key: "{v.s(\"tab\")}", comments: comments(), now, tab: v.string("tab"), avatars: v.string("avatars"), max_list: if v.b("cap") { 330.0 } else { 0.0 },
                on_run_edit: move |id: u32| msg.set(format!("onRunEdit({id})")),
                on_add: move |t: String| msg.set(format!("onAdd({t:?})")),
                on_close: move |_| msg.set("onClose()".into()),
            }
            {status(msg)}
        }
    }
}

fn comments_panel_code(v: &Vals) -> String {
    Code::new("CommentsPanel")
        .raw("comments", "feed.comments()")
        .s("tab", v.s("tab"), "all")
        .raw("now", "clock.now()")
        .raw("on_run_edit", "move |id| api.run_edit(id)")
        .raw("on_add", "move |text| api.add(text)")
        .done()
}

/// The plan #32 demos (chunk 1, then chunk 2).
pub(crate) fn demos() -> Vec<Demo> {
    let mut v = demos1();
    v.extend(demos2());
    v
}

/// Chunk 1 demos: Chat Composer, Comment Thread, Comments Panel.
pub(crate) fn demos1() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                sel("style", "Style", &["clean", "classic"], "clean"),
                sel("approval", "Approval", &["ask", "auto", "full"], "ask"),
                tog("tray", "Folder and apps tray", true),
                tog("attach", "Start with attachments", false),
                tog("recording", "Start recording", false),
                sel(
                    "open",
                    "Open menu",
                    &["none", "plus", "approval", "model", "folder", "apps"],
                    "none",
                ),
            ],
            variants: vec![
                var(
                    "default",
                    "Clean: one continuous container with the tray under a hairline; type, pick approval and model, dictate, connect apps.",
                    &[],
                ),
                var(
                    "classic",
                    "Classic style: the writing card inset on a darker band, the tray on the band.",
                    &[("style", "classic")],
                ),
                var(
                    "unrestricted",
                    "Full access picked: the orange Unrestricted access chip.",
                    &[("approval", "full")],
                ),
                var(
                    "recording",
                    "Recording: sixteen blue capsules resting as dots and rising with the voice, words arriving in grey, ✕ discards and ✓ keeps them.",
                    &[("recording", "true")],
                ),
                var(
                    "attachments",
                    "Oxyn extra: attachment chips (PDF, image thumbnail, spreadsheet) above the field.",
                    &[("attach", "true")],
                ),
                var(
                    "plus-menu",
                    "The plus menu: a capped, scrolling list – Files and folders, Attach Claude, Work in a folder, Goal, Plan mode, Record a skill, Draw, then Plugins: Documents, PDF.",
                    &[("open", "plus")],
                ),
                var(
                    "models",
                    "The model menu with coloured brand marks, capped at six rows with a thin scroll.",
                    &[("open", "model")],
                ),
                var(
                    "approvals-menu",
                    "How should actions be approved? Request approval, Approve for me, Full access.",
                    &[("open", "approval")],
                ),
                var(
                    "in-chat",
                    "Once a chat has started: no tray, the apps move into the plus menu.",
                    &[("tray", "false")],
                ),
                var(
                    "oxyn-ai-composer",
                    "Extended: the AI composer with / commands and @ mentions.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: chat_composer,
            code: chat_composer_code,
            keyboard: vec![
                ("Enter", "Send"),
                ("Shift + Enter", "New line"),
                (
                    "Enter / Space / ↓",
                    "Open the menu of the focused button (plus, approval, model, folder, apps)",
                ),
                ("Esc", "Close the menu, or discard a recording"),
                (
                    "Tab",
                    "Move on (closes nothing silently: menus close on pick or Esc)",
                ),
            ],
            a11y: vec![
                "Plus is a menu (Plan mode a menuitemcheckbox); approval, model and folder are listboxes; apps are menuitemcheckboxes; every trigger carries aria-expanded.",
                "A polite live region announces modes, models, attachments, recording and sends.",
                "Clicking anywhere on the card focuses the field.",
            ],
            responsive: vec![
                "Fills its container up to 680 px; under 460 px the plus menu drops its hints; under 380 px the approval chip shows its icon only.",
            ],
            height: 600,
            live: true,
            ..mk(
                "chat-composer",
                "Chat composer",
                "composer",
                "Prompt composer with model picker, voice input, app connections and approval request: a scrolling plus menu with Plugins, the orange Unrestricted access chip, recording with a dotted level meter, and a tray with a folder picker and Connect apps.",
                "ChatComposer, FileTypeIcon",
                "ChatComposerEl",
                "chat-composer",
                "parity/fidelity.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Details open", false),
                tog("answer", "Agent answers replies", true),
                sel("avatars", "Avatars", &["photo", "drawn"], "photo"),
            ],
            variants: vec![
                var(
                    "default",
                    "Reply, or ask @Atlas: your reply lands as just now, then Atlas thinks and answers.",
                    &[],
                ),
                var(
                    "details",
                    "Edit completed folded open.",
                    &[("open", "true")],
                ),
                var(
                    "people-only",
                    "The agent does not answer replies.",
                    &[("answer", "false")],
                ),
                var(
                    "drawn-avatars",
                    "Oxyn extra: illustrated portraits instead of photos.",
                    &[("avatars", "drawn")],
                ),
            ],
            render: comment_thread,
            code: comment_thread_code,
            keyboard: vec![
                ("Enter", "Send the reply"),
                ("Tab", "Reach the status line to fold its details open"),
            ],
            a11y: vec![
                "Status lines are buttons with aria-expanded; the thread is a polite log that announces new replies.",
            ],
            responsive: vec!["Fills its container up to 420 px."],
            height: 520,
            live: true,
            ..mk(
                "comment-thread",
                "Comment thread",
                "chat",
                "One comment with replies – an assistant reports what it did, folds open details and answers @mentions.",
                "AgentCommentThread, PhotoAvatar, AgentOrb",
                "AgentCommentThreadEl",
                "comment-thread",
                "parity/fidelity.rs",
            )
        },
        Demo {
            controls: vec![
                sel("tab", "Tab", &["all", "questions", "edits"], "all"),
                sel("avatars", "Avatars", &["photo", "drawn"], "photo"),
                tog("cap", "List scrolls inside a height cap", true),
            ],
            variants: vec![
                var(
                    "all",
                    "Run edit, resolve, sort, and add a comment or /edit.",
                    &[],
                ),
                var(
                    "edits",
                    "The Edits tab: runnable /edit comments.",
                    &[("tab", "edits")],
                ),
                var("questions", "The Questions tab.", &[("tab", "questions")]),
                var(
                    "drawn-avatars",
                    "Oxyn extra: illustrated portraits, the list at full height.",
                    &[("avatars", "drawn"), ("cap", "false")],
                ),
            ],
            render: comments_panel,
            code: comments_panel_code,
            keyboard: vec![
                ("Tab", "Tabs, sort, comments, composer"),
                ("Enter", "Send the comment"),
            ],
            a11y: vec![
                "Filters are tabs, sorting is a menu of radio items, Resolve is a toggle button; the list announces new comments politely.",
            ],
            responsive: vec![
                "Fills its container up to 400 px; the list scrolls inside a 330 px cap with a thin pill thumb while the header, tabs and composer stay put.",
            ],
            height: 600,
            live: true,
            ..mk(
                "comments-panel",
                "Comments panel",
                "chat",
                "Comment feed with filters, runnable /edit commands, resolve and a composer.",
                "CommentsPanel, PhotoAvatar",
                "CommentsPanelEl",
                "comments-panel",
                "parity/fidelity.rs",
            )
        },
    ]
}

// ---------- model picker (affectUI) and the Oxyn model menu ----------

fn menu_models() -> Vec<MenuModel> {
    let m = |id: &str,
             name: &str,
             d: &str,
             s: Option<u8>,
             q: Option<u8>,
             c: Option<u8>,
             l: Option<u32>,
             p: Option<f64>| MenuModel {
        id: id.into(),
        name: name.into(),
        description: d.into(),
        speed: s,
        quality: q,
        cost: c,
        latency_ms: l,
        price_per_m: p,
    };
    vec![
        m(
            "auto",
            "Auto",
            "Routes each message to the best model",
            None,
            None,
            None,
            None,
            None,
        ),
        m(
            "opus",
            "Opus 5.5",
            "Hardest problems and long agent runs",
            Some(2),
            Some(5),
            Some(5),
            Some(1400),
            Some(15.0),
        ),
        m(
            "sonnet",
            "Sonnet 5.5",
            "Everyday coding and writing",
            Some(4),
            Some(4),
            Some(3),
            Some(620),
            Some(3.0),
        ),
    ]
}

fn model_picker(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    if v.s("kind") == "oxyn" {
        return rsx! {
            div { style: "width:362px;max-width:100%;min-height:440px;display:flex;flex-direction:column;gap:6px;padding-top:24px",
                ModelMenu { models: menu_models(), value: "sonnet", thinking: v.b("thinking"),
                    on_change: move |id: String| msg.set(format!("onChange({id:?})")),
                    on_option: move |(k, x): (String, String)| msg.set(format!("{k} → {x}")) }
                {status(msg)}
            }
        };
    }
    let key = format!(
        "{}{}{}{}",
        v.s("marks"),
        v.b("thinking"),
        v.b("web"),
        v.b("open")
    );
    rsx! {
        div { key: "{key}", style: "width:362px;max-width:100%;min-height:520px;display:flex;flex-direction:column;gap:10px;padding-top:24px",
            ModelPickerMenu { marks: v.string("marks"), thinking: v.b("thinking"), web_search: v.b("web"), default_open: v.b("open"),
                style: if v.s("style") == "classic" { "classic".to_string() } else { String::new() },
                on_change: move |id: String| msg.set(format!("onChange({id:?})")),
                on_thinking_change: move |on: bool| msg.set(format!("onThinkingChange({on})")),
                on_web_search_change: move |on: bool| msg.set(format!("onWebSearchChange({on})")),
                on_effort_change: move |e: String| msg.set(format!("onEffortChange({e:?})")),
            }
            {status(msg)}
        }
    }
}

fn model_picker_code(v: &Vals) -> String {
    if v.s("kind") == "oxyn" {
        return Code::new("ModelMenu")
            .raw("models", "models")
            .raw("value", "\"sonnet\"")
            .b("thinking", v.b("thinking"))
            .raw("on_change", "move |id| model.set(id)")
            .done();
    }
    let mut c = Code::new("ModelPickerMenu").raw("models", "models");
    if v.s("marks") == "brands" {
        c = c.raw("marks", "\"brands\".into()");
    }
    if v.b("thinking") {
        c = c.b("thinking", true);
    }
    c.raw("on_change", "move |id| model.set(id)")
        .raw("on_effort_change", "move |e| effort.set(e)")
        .done()
}

// ---------- run controls (affectUI) and the Oxyn run controls ----------

const ANSWER: &str = "I'll add a token bucket per API key in the gateway middleware. Each key gets 60 requests a minute with a burst of 20; when the bucket is empty the request gets a 429 with a Retry-After header. Then I'll add two integration tests that hammer one key and check the header, and update the API docs with the new limits.";

fn run_turns() -> Vec<RunTurn> {
    vec![
        RunTurn::parse(
            "Draft the release notes for 2.4",
            "Release 2.4 – faster syncs and a calmer inbox.\n\nSync – large workspaces now load about twice as fast, and the first sync after an update no longer blocks the editor. [[steer:Keep it to three sections]] Three sections it is – the small fixes fold into one line.\n\nInbox – mentions and replies are grouped by thread, and muted threads stay muted on every device.\n\nFixes – exports keep image captions, the date picker respects the week start, and shortcuts work again on non-US keyboards.",
        ),
        RunTurn::parse(
            "Add an upgrade note for self-hosted teams",
            "Self-hosted teams should update the sync service before the apps – the new clients expect the 2.4 protocol.\n\n· Back up the database and stop the sync service\n\n· Pull the 2.4 image and run the migration\n\n· Restart it, then update the desktop apps",
        ),
    ]
}

fn run_controls(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let on = move |(a, t): (String, String)| {
        msg.set(if t.is_empty() {
            format!("on{a}")
        } else {
            format!("on{a}({t:?})")
        })
    };
    if v.s("kind") == "oxyn" {
        let perm = if v.s("permission") == "auto" {
            "edits".to_string()
        } else {
            v.string("permission")
        };
        return rsx! {
            div { style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:6px",
                RunControls { agent: "Builder", agent_id: "builder-7", prompt: "Add rate limiting to the public API", answer: ANSWER,
                    status: v.string("status"), mode: v.string("mode"), permission: perm, context: 0.42,
                    queue: if v.b("queued") { vec!["Then update the changelog".to_string()] } else { vec![] }, now,
                    on_action: on }
                {status(msg)}
            }
        };
    }
    let st = v.s("status");
    let key = format!(
        "{st}{}{}{}{}",
        v.s("mode"),
        v.s("permission"),
        v.b("queued"),
        v.b("menu")
    );
    rsx! {
        div { key: "{key}", style: "width:520px;max-width:100%;display:flex;flex-direction:column;gap:8px;padding-top:12px",
            AgentRunPanel { turns: run_turns(), now,
                queue: if v.b("queued") { vec!["Add an upgrade note for self-hosted teams".to_string()] } else { vec![] },
                autoplay: st != "idle", stopped_at: if st == "stopped" { Some(29) } else { None }, elapsed: if st == "done" { 60_000 } else { 0 },
                mode: v.string("mode"), permission: v.string("permission"), menu_open: v.b("menu"),
                style: if v.s("style") == "classic" { "classic".to_string() } else { String::new() },
                on_action: on }
            {status(msg)}
        }
    }
}

fn run_controls_code(v: &Vals) -> String {
    if v.s("kind") == "oxyn" {
        return Code::new("RunControls")
            .raw("agent", "\"Builder\"")
            .raw("prompt", "run.prompt.clone()")
            .raw("answer", "run.answer.clone()")
            .s("status", &v.string("status"), "idle")
            .raw("now", "clock.now()")
            .raw("on_action", "move |(what, text)| dispatch(what, text)")
            .done();
    }
    let mut c = Code::new("AgentRunPanel")
        .raw("agent", "\"Release writer\".into()")
        .raw("turns", "run.turns()")
        .raw("queue", "run.queue()");
    if v.s("mode") == "chat" {
        c = c.raw("mode", "\"chat\".into()");
    }
    c.raw("now", "clock.now()")
        .raw("on_action", "move |(what, text)| dispatch(what, text)")
        .done()
}

/// Chunk 2 demos: Model Picker and Run Controls.
pub(crate) fn demos2() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                sel("style", "Style", &["clean", "classic"], "clean"),
                sel("marks", "Model marks", &["letters", "brands"], "letters"),
                tog("open", "Menu open", true),
                tog("thinking", "Extended thinking", false),
                tog("web", "Web search", false),
            ],
            variants: vec![
                var(
                    "default",
                    "Clean: point at a model and its dashes fill blue while the tray rolls its first-token time and price.",
                    &[],
                ),
                var(
                    "thinking",
                    "Extended thinking on: the Low / Medium / High effort unfolds and the time multiplies.",
                    &[("thinking", "true")],
                ),
                var(
                    "web-search",
                    "Web search on: the tray says it looks things up.",
                    &[("web", "true")],
                ),
                var(
                    "classic",
                    "Classic style: title-case headers, monochrome dot-dashes, a round send and the tray on its own shelf.",
                    &[("style", "classic")],
                ),
                var(
                    "brand-marks",
                    "Oxyn extra: coloured brand marks on the rows and the trigger instead of letter tiles.",
                    &[("marks", "brands")],
                ),
                var(
                    "closed",
                    "Closed: the trigger pill with the model, a blue bulb when thinking is on, and the chevron.",
                    &[("open", "false"), ("thinking", "true")],
                ),
                var(
                    "oxyn-model-menu",
                    "Extended: the compact model menu whose stats unfold under the pointed row.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: model_picker,
            code: model_picker_code,
            keyboard: vec![
                ("Enter / Space / ↓", "Open the menu from the trigger"),
                ("↑ / ↓", "Move through the models (the tray follows)"),
                ("Home / End", "First / last model"),
                ("Enter", "Pick and close"),
                ("Esc", "Close, focus back on the trigger"),
            ],
            a11y: vec![
                "The menu is a dialog \"Model settings\" holding a listbox with aria-activedescendant; each option names its speed, quality and cost (\"Speed 2, quality 5, cost 5 of 5\").",
                "Extended thinking and Web search are switches; Effort is a radiogroup that is only tabbable while thinking is on.",
                "The trigger's name follows the settings (\"Model: Sonnet 5.5, extended thinking\"); the tray is a polite live region.",
            ],
            responsive: vec![
                "The composer is 362 px and the menu 320 px; under 340 px the descriptions drop and rows shrink to 40 px.",
            ],
            height: 560,
            live: true,
            ..mk(
                "model-picker",
                "Model picker",
                "composer",
                "The model menu of an AI chat – point at a model and its speed, quality and cost dashes fill, switch on extended thinking and its effort unfolds, while the tray rolls the first-token time and price.",
                "ModelPickerMenu",
                "ModelPickerMenuEl",
                "model-picker",
                "parity/chatfid.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                sel("style", "Style", &["clean", "classic"], "clean"),
                sel(
                    "status",
                    "Start",
                    &["running", "stopped", "done", "idle", "error"],
                    "running",
                ),
                sel("mode", "Mode", &["act", "chat"], "act"),
                sel("permission", "Permission", &["ask", "auto", "full"], "ask"),
                tog("queued", "A queued message", true),
                tog("menu", "Permissions menu open", false),
            ],
            variants: vec![
                var(
                    "running",
                    "Streaming: a steer lands mid-sentence as a blue pill, the queued message waits as Next and runs when the turn is done.",
                    &[],
                ),
                var(
                    "stopped",
                    "Stopped at a word: Resume, Retry, Dismiss.",
                    &[("status", "stopped"), ("queued", "false")],
                ),
                var(
                    "done",
                    "Done: the time it took and Retry.",
                    &[("status", "done"), ("queued", "false")],
                ),
                var(
                    "permissions",
                    "What the agent may do: Ask first, Auto-edit, Full access in red.",
                    &[("status", "idle"), ("menu", "true"), ("queued", "false")],
                ),
                var(
                    "chat-mode",
                    "Chat mode: the permissions are disabled – chat only answers.",
                    &[("mode", "chat"), ("status", "idle"), ("queued", "false")],
                ),
                var(
                    "classic",
                    "Classic style: the run as a white panel on a grey tray, the queue on the tray.",
                    &[("style", "classic")],
                ),
                var(
                    "oxyn-run-controls",
                    "Extended: the run composer with a send menu (Send now, Steer, Queue) and an error state.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: run_controls,
            code: run_controls_code,
            keyboard: vec![
                ("Enter", "Send when idle; steer while running"),
                ("⌥ Enter", "Queue"),
                ("Shift + Enter", "New line"),
                ("Esc", "Stop a running stream (or close the menu)"),
            ],
            a11y: vec![
                "The run is a polite log, busy while streaming; a status region reads \"Release writer: Writing · steered, 1 queued\".",
                "Mode is a radiogroup; the permissions button is aria-disabled in Chat mode with the reason in its title; the menu is a listbox.",
                "Reduced motion stops the caret blink and the pill and roll animations.",
            ],
            responsive: vec![
                "Fills its container up to 520 px; under 440 px the composer's controls wrap and the answer drops its indent.",
            ],
            height: 470,
            live: true,
            ..mk(
                "run-controls",
                "Run controls",
                "composer",
                "Stop, steer and queue a running agent – the answer streams with a caret, a steer lands mid-sentence as a pill, a queued message runs next, and a stopped run resumes from its word.",
                "AgentRunPanel",
                "AgentRunPanelEl",
                "run-controls",
                "parity/chatfid.rs",
            )
        },
    ]
}
