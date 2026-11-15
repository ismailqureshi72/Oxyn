//! Plan #32 (affectUI fidelity pass) demos: the exact Chat Composer with every affectUI variant
//! (and Oxyn's own AI composer as an extra), Comment Thread and Comments Panel.

use dioxus::prelude::*;
use oxyn_core::{ClockMs, parity::fidelity::PanelComment};
use oxyn_dioxus::{AgentCommentThread, AiComposer, ChatComposer, CommentsPanel, ThreadNote};

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
        div { style: "width:560px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:10px;padding:206px 0 166px",
            ChatComposer { now, style: v.string("style"), approval: v.string("approval"), tray: v.b("tray"), recording: v.b("recording"),
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
            AgentCommentThread { notes: notes(), now, open: v.b("open"),
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
    ]
}

fn comments_panel(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:420px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:8px;padding-bottom:40px",
            CommentsPanel { key: "{v.s(\"tab\")}", comments: comments(), now, tab: v.string("tab"),
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

/// The plan #32 demos.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("kind", "Component", &["affectui", "oxyn"], "affectui"),
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
                    "Clean, as affectUI ships it: type, pick approval and model, dictate, add files, connect apps.",
                    &[],
                ),
                var(
                    "classic",
                    "Classic style: one surface, the tray under a hairline.",
                    &[("style", "classic")],
                ),
                var(
                    "unrestricted",
                    "Full access picked: the orange Unrestricted access chip.",
                    &[("approval", "full")],
                ),
                var(
                    "recording",
                    "Recording: the live trace, words arriving in grey, ✕ discards and ✓ keeps them.",
                    &[("recording", "true")],
                ),
                var(
                    "attachments",
                    "Attachment chips from the plus menu: PDF, image thumbnail, spreadsheet.",
                    &[("attach", "true")],
                ),
                var(
                    "plus-menu",
                    "The plus menu: file types with their icons, then Goal, Plan mode, Record a skill, Draw.",
                    &[("open", "plus")],
                ),
                var(
                    "models",
                    "The model menu with coloured brand marks.",
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
                    "Oxyn extra (beyond affectUI): the AI composer with / commands and @ mentions.",
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
                "Fills its container up to 540 px; under 460 px the file tiles wrap to 3 × 2 and the tools to one column; under 380 px the approval chip shows its icon only.",
            ],
            height: 600,
            live: true,
            ..mk(
                "chat-composer",
                "Chat composer",
                "composer",
                "Prompt composer with model picker, voice input, app connections and approval request: a plus drop-up with file types that add attachment chips, the orange Unrestricted access chip, recording with a live trace, and a tray with a folder picker and Connect apps.",
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
                "AgentCommentThread, Portrait, AgentOrb",
                "AgentCommentThreadEl",
                "comment-thread",
                "parity/fidelity.rs",
            )
        },
        Demo {
            controls: vec![sel("tab", "Tab", &["all", "questions", "edits"], "all")],
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
            responsive: vec!["Fills its container up to 400 px."],
            height: 600,
            live: true,
            ..mk(
                "comments-panel",
                "Comments panel",
                "chat",
                "Comment feed with filters, runnable /edit commands, resolve and a composer.",
                "CommentsPanel",
                "CommentsPanelEl",
                "comments-panel",
                "parity/fidelity.rs",
            )
        },
    ]
}
