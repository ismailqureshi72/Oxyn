//! Composer: the shell and its parts.

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::{
        BusyAction, ContextMeter, DisabledReason, EffortControl, EffortLevel, EffortSetting,
        ModelSelection, Picker, SendControl, SendState, SendVariant,
    },
};
use oxyn_dioxus::{
    AttachmentRow, Composer, ContextMeterChip, DropZone, EffortChip, ModelPicker, SendButton,
};

use super::{Code, q, sel, tog, txt, var};
use crate::{Align, Demo, Vals, data};

const SENDS: [&str; 12] = [
    "classic",
    "ring-progress",
    "queue-pill",
    "split",
    "steer-slide",
    "status-tab",
    "pause-stop",
    "glyph",
    "voice-confirm",
    "answer-card",
    "retry",
    "minimal",
];

fn level(id: &str) -> EffortLevel {
    match id {
        "low" => EffortLevel::Low,
        "high" => EffortLevel::High,
        "max" => EffortLevel::Max,
        _ => EffortLevel::Medium,
    }
}

fn composer(v: &Vals, now: ClockMs) -> Element {
    let st = v.s("agent");
    let attachments = match v.s("attachments") {
        "files" => data::attachments(),
        "context" => data::context_refs(),
        _ => vec![],
    };
    let att_variant = (v.s("attachments") == "context").then(|| "context-chip".to_string());
    let disabled = match v.s("disabled") {
        "offline" => Some(DisabledReason::Offline),
        "quota" => Some(DisabledReason::Quota {
            resets: "14:00".into(),
        }),
        _ => None,
    };
    let open = if v.s("open") == "none" {
        String::new()
    } else {
        v.string("open")
    };
    let history = if v.b("critical") { 118_000 } else { 72_000 };
    let tray = if v.s("layout") == "desktop" {
        vec!["oxyn".to_string(), "feat/site-v1".into(), "Local".into()]
    } else {
        vec![]
    };
    rsx! {
        div { class: "oxd-composer {v.s(\"layout\")}",
            Composer {
                layout: v.string("layout"),
                agent: data::state(st),
                now,
                elapsed: Duration::from_secs(42),
                catalog: data::catalog(),
                model: ModelSelection::Model("ember/swift-2".into()),
                favorites: data::favorites(),
                recent: vec!["slate/vision".to_string()],
                effort: EffortSetting::Level(level(v.s("effort"))),
                usage: Some(data::usage(history)),
                attachments,
                attachment_variant: att_variant,
                disabled,
                text: v.string("text"),
                open,
                tray,
                busy_action: if v.s("busy_action") == "steer" { BusyAction::Steer } else { BusyAction::Queue },
            }
        }
    }
}

fn composer_code(v: &Vals) -> String {
    let mut c = Code::new("Composer")
        .s("layout", v.s("layout"), "chat")
        .raw("agent", "agent.state()")
        .raw("now", "clock.now()")
        .raw("catalog", "models()")
        .raw("model", "ModelSelection::Model(\"ember/swift-2\".into())")
        .raw("usage", "Some(context_usage())");
    if v.s("attachments") != "none" {
        c = c.raw("attachments", "attachments()");
    }
    if v.s("attachments") == "context" {
        c = c.raw("attachment_variant", q("context-chip"));
    }
    match v.s("disabled") {
        "offline" => c = c.raw("disabled", "Some(DisabledReason::Offline)"),
        "quota" => {
            c = c.raw(
                "disabled",
                "Some(DisabledReason::Quota { resets: \"14:00\".into() })",
            )
        }
        _ => {}
    }
    if v.s("busy_action") == "steer" {
        c = c.raw("busy_action", "BusyAction::Steer");
    }
    c.raw("on_submit", "move |s: Submission| agent.send(s)")
        .raw("on_stop", "move |_| agent.stop()")
        .done()
}

fn send(v: &Vals, now: ClockMs) -> Element {
    let variant = SendVariant::ALL
        .into_iter()
        .find(|x| x.id() == v.s("variant"))
        .unwrap_or(SendVariant::Classic);
    let state = match v.s("state") {
        "empty" => SendState::Empty,
        "working" => SendState::Working,
        "working-draft" => SendState::WorkingWithDraft,
        "paused" => SendState::Paused,
        "error" => SendState::Error("Couldn't send".into()),
        "answering" => SendState::Answering,
        "dictating" => SendState::Dictating,
        _ => SendState::Ready,
    };
    let mut c = SendControl::new(variant);
    let view = c.view(state, BusyAction::Queue, Duration::from_secs(20), None, now);
    rsx! {
        div { class: "oxd-sendbox",
            span { class: "oxd-ph", "Ask the agent…" }
            SendButton { view, variant: variant.id().to_string() }
        }
    }
}

fn send_code(v: &Vals) -> String {
    Code::new("SendButton")
        .pre("let mut send = use_signal(|| SendControl::new(SendVariant::RingProgress));")
        .pre(
            "let view = send.write().view(state, BusyAction::Queue, elapsed, budget, clock.now());",
        )
        .raw("view", "view")
        .raw("variant", q(v.s("variant")))
        .raw(
            "on_action",
            "move |a: SendAction| agent.handle(a) // send, stop, queue, steer…",
        )
        .done()
}

fn picker(v: &Vals, now: ClockMs) -> Element {
    let cat = data::catalog();
    let favs = data::favorites();
    let recent = vec!["slate/vision".to_string()];
    let sel = if v.s("selected") == "auto" {
        ModelSelection::Auto { resolved: None }
    } else {
        ModelSelection::Model("ember/swift-2".into())
    };
    let mut p = Picker::default();
    if v.b("open") {
        let rows = p.rows(&cat, &favs, &recent, &sel);
        p.open(&rows, &sel);
        p.query = v.string("query");
    }
    let pending = v
        .b("pending")
        .then(|| ModelSelection::Model("ember/deep-2".into()));
    rsx! {
        div { class: "oxd-room-up",
            ModelPicker { catalog: cat, selected: sel, pending, picker: p, favorites: favs, recent, routing: v.b("routing"), variant: v.string("variant"), now }
        }
    }
}

fn picker_code(v: &Vals) -> String {
    Code::new("ModelPicker")
        .raw("catalog", "models()")
        .raw("selected", "model()")
        .raw(
            "picker",
            "picker() // popover state: open, query, highlight",
        )
        .raw("favorites", "favorites()")
        .s("variant", v.s("variant"), "pill-popover")
        .b("routing", v.b("routing"))
        .raw("now", "clock.now()")
        .raw("on_choose", "move |m: ModelSelection| model.set(m)")
        .done()
}

fn effort(v: &Vals, now: ClockMs) -> Element {
    let mut ec = EffortControl::new(EffortSetting::Level(level(v.s("level"))));
    if v.b("pending") {
        ec.commit(
            EffortSetting::Level(EffortLevel::Low),
            &data::state("streaming"),
            &data::levels(),
            crate::BASE,
        );
    }
    let view = ec.view(&data::levels(), now);
    rsx! {
        div { class: "oxd-room-up",
            EffortChip { view, support: data::levels(), variant: v.string("variant"), open: v.b("open"), model: "Ember Swift 2" }
        }
    }
}

fn effort_code(v: &Vals) -> String {
    Code::new("EffortChip")
        .pre("let mut effort = use_signal(|| EffortControl::new(EffortSetting::Level(EffortLevel::High)));")
        .pre("let view = effort.write().view(&model.caps.reasoning, clock.now());")
        .raw("view", "view")
        .raw("support", "model.caps.reasoning.clone()")
        .s("variant", v.s("variant"), "signal-bars")
        .raw(
            "on_pick",
            "move |e: EffortSetting| effort.write().commit(e, &agent.state(), &support, clock.now())",
        )
        .done()
}

fn context(v: &Vals, now: ClockMs) -> Element {
    let history = match v.s("level") {
        "warning" => 100_000,
        "critical" => 118_000,
        _ => 52_000,
    };
    let u = data::usage(history);
    let mut m = ContextMeter::default();
    let _ = m.view(&u, ClockMs(0));
    let view = m.view(&u, now);
    rsx! {
        div { class: "oxd-room-up",
            ContextMeterChip { view, usage: u, variant: v.string("variant"), open: v.b("open"), model: "Ember Swift 2" }
        }
    }
}

fn context_code(v: &Vals) -> String {
    Code::new("ContextMeterChip")
        .pre("let mut meter = use_signal(ContextMeter::default);")
        .pre("let view = meter.write().view(&usage, clock.now()); // eases between reports")
        .raw("view", "view")
        .raw("usage", "usage.clone()")
        .s("variant", v.s("variant"), "ring")
        .raw("model", q("Ember Swift 2"))
        .raw("on_compact", "move |_| agent.compact()")
        .done()
}

fn attachments(v: &Vals, now: ClockMs) -> Element {
    let variant = v.s("variant");
    let items = if variant == "context-chip" {
        data::context_refs()
    } else {
        data::attachments()
    };
    let drop = v.s("drop");
    rsx! {
        div { class: if variant == "terminal" { "oxd-term oxd-w560" } else { "oxd-w560" },
            AttachmentRow { items, now, variant: variant.to_string(), reduced: v.b("reduced") }
            if drop != "none" {
                div { class: "oxd-drop",
                    DropZone { state: drop.to_string(), label: if drop == "invalid" { "2 of 3 files can't be attached" } else { "Drop to attach · 3 files" } }
                }
            }
        }
    }
}

fn attachments_code(v: &Vals) -> String {
    Code::new("AttachmentRow")
        .pre("let items = vec![")
        .pre("    AttachmentItem::file(1, \"login-flow.png\", 1_400_000, AttachmentState::Uploading { progress: Some(0.64) }),")
        .pre("    AttachmentItem::file(2, \"spec.pdf\", 812_000, AttachmentState::Ready { meta: \"PDF · 12 pages\".into() }),")
        .pre("];")
        .raw("items", "items")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "tile-ring")
        .raw("on_remove", "move |id: u64| remove(id)")
        .raw("on_retry", "move |id: u64| retry(id)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "composer",
            name: "Composer",
            category: "composer",
            summary: "The prompt box for an agent, in six layouts: send and stop, queue or steer while it works, model, effort and context in reach, attachments, and a reason when sending is off.",
            dioxus: "Composer",
            gpui: "ComposerEl",
            spec: "composer",
            source: "composer/mod.rs",
            controls: vec![
                sel(
                    "layout",
                    "Layout",
                    &["chat", "desktop", "ide-panel", "terminal", "mobile", "node"],
                    "chat",
                ),
                sel(
                    "agent",
                    "Agent state",
                    &[
                        "idle",
                        "thinking",
                        "streaming",
                        "tool-running",
                        "awaiting-approval",
                        "stalled",
                        "error",
                    ],
                    "idle",
                ),
                txt(
                    "text",
                    "Draft",
                    "Why does login fail when two tabs refresh at once?",
                ),
                sel(
                    "attachments",
                    "Attachments",
                    &["none", "files", "context"],
                    "files",
                ),
                sel(
                    "open",
                    "Popover",
                    &["none", "model", "effort", "context"],
                    "none",
                ),
                sel(
                    "effort",
                    "Effort",
                    &["low", "medium", "high", "max"],
                    "medium",
                ),
                sel(
                    "busy_action",
                    "Enter while busy",
                    &["queue", "steer"],
                    "queue",
                ),
                sel(
                    "disabled",
                    "Sending off",
                    &["none", "offline", "quota"],
                    "none",
                ),
                tog("critical", "Context nearly full", false),
            ],
            variants: vec![
                var("chat", "Chat app layout with files.", &[]),
                var(
                    "busy",
                    "Working, with a draft to queue.",
                    &[
                        ("agent", "tool-running"),
                        ("text", "also check the refresh-token path"),
                        ("attachments", "none"),
                        ("critical", "true"),
                    ],
                ),
                var(
                    "desktop",
                    "Desktop with context chips and a repo tray.",
                    &[
                        ("layout", "desktop"),
                        ("attachments", "context"),
                        ("text", "Plan the migration to the new session store"),
                    ],
                ),
                var(
                    "ide-panel",
                    "IDE side panel, thinking.",
                    &[("layout", "ide-panel"), ("agent", "thinking"), ("text", "")],
                ),
                var(
                    "terminal",
                    "Terminal prompt.",
                    &[
                        ("layout", "terminal"),
                        ("text", "refactor the token cache"),
                        ("attachments", "none"),
                    ],
                ),
                var(
                    "offline",
                    "Mobile, offline: the draft is kept.",
                    &[
                        ("layout", "mobile"),
                        ("disabled", "offline"),
                        ("text", "Draft kept while offline"),
                    ],
                ),
            ],
            render: composer,
            code: composer_code,
            keyboard: vec![
                ("Enter", "Send (queue or steer while the agent works)"),
                ("Shift + Enter", "New line"),
                ("Esc", "Close a popover; otherwise stop the run"),
                ("Alt + . / Alt + ,", "Raise / lower effort"),
            ],
            a11y: vec![
                "A multi-line text box labelled \"Message the agent\"; the placeholder is never the label.",
                "aria-describedby points at the hint row: keyboard hint, counter and the reason sending is off.",
                "Busy changes, large pastes and send failures are announced; only send failure is assertive.",
            ],
            responsive: vec![
                "Six layouts: chat, desktop, ide-panel, terminal, mobile and node (canvas).",
                "Typing still works offline and the draft is kept.",
            ],
            height: 460,
            align: Align::End,
            live: true,
        },
        Demo {
            slug: "send-control",
            name: "Send control",
            category: "composer",
            summary: "Send, stop, queue and steer in one control, in twelve styles: a progress ring, a queue pill, a split, a steer slide, a status tab, pause and stop, a glyph, voice confirm, an answer card, retry and minimal.",
            dioxus: "SendButton",
            gpui: "SendButtonEl",
            spec: "composer-send-control",
            source: "composer/send.rs",
            controls: vec![
                sel("variant", "Variant", &SENDS, "ring-progress"),
                sel(
                    "state",
                    "State",
                    &[
                        "empty",
                        "ready",
                        "working",
                        "working-draft",
                        "paused",
                        "error",
                        "answering",
                        "dictating",
                    ],
                    "working",
                ),
            ],
            variants: vec![
                var("ring-progress", "Stop square with a progress ring.", &[]),
                var(
                    "classic",
                    "Arrow, then stop.",
                    &[("variant", "classic"), ("state", "ready")],
                ),
                var(
                    "queue-pill",
                    "Queue while it works.",
                    &[("variant", "queue-pill"), ("state", "working-draft")],
                ),
                var(
                    "split",
                    "Send and steer side by side.",
                    &[("variant", "split"), ("state", "working-draft")],
                ),
                var(
                    "pause-stop",
                    "Pause or stop.",
                    &[("variant", "pause-stop"), ("state", "paused")],
                ),
                var(
                    "retry",
                    "Retry after a failed send.",
                    &[("variant", "retry"), ("state", "error")],
                ),
            ],
            render: send,
            code: send_code,
            keyboard: vec![
                (
                    "Enter",
                    "Send, or queue while the agent works (in the composer)",
                ),
                ("Shift + Enter", "New line"),
                ("Esc", "Stop (with stop_confirm: press twice)"),
            ],
            a11y: vec![
                "Its name follows the state: \"Send message\", \"Stop generating\", \"Queue message\", \"Retry sending\".",
                "\"Message sent\", \"Message queued\" and \"Generation stopped\" are announced politely; errors assertively.",
            ],
            responsive: vec![
                "Round styles fit any composer; status-tab, split and queue-pill need more width.",
            ],
            height: 160,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "model-picker",
            name: "Model picker",
            category: "composer",
            summary: "The model menu of an agent app: providers, favourites and recents, search, capability badges, a model that needs a key or is rate limited, Auto routing and a change that waits for the next turn.",
            dioxus: "ModelPicker",
            gpui: "ModelPickerEl",
            spec: "composer-model-picker",
            source: "composer/model.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["pill-popover", "compact", "favorites-segmented", "glyph"],
                    "pill-popover",
                ),
                tog("open", "Open", true),
                txt("query", "Search", ""),
                sel("selected", "Selected", &["model", "auto"], "model"),
                tog("pending", "Change waits for next turn", false),
                tog("routing", "Auto routing", false),
            ],
            variants: vec![
                var("open", "The full list.", &[]),
                var("search", "Searching \"dp\".", &[("query", "dp")]),
                var(
                    "pending",
                    "Deep 2 from the next turn.",
                    &[("open", "false"), ("pending", "true")],
                ),
                var(
                    "favorites-segmented",
                    "Favourites as segments.",
                    &[("variant", "favorites-segmented"), ("open", "false")],
                ),
                var("glyph", "Terminal picker.", &[("variant", "glyph")]),
            ],
            render: picker,
            code: picker_code,
            keyboard: vec![
                ("↑ / ↓", "Move through models"),
                ("Enter", "Choose"),
                ("Esc", "Close"),
                ("Type", "Search"),
            ],
            a11y: vec![
                "The pill is a button with aria-haspopup=listbox and a full name (\"Model: Ember Swift 2. Change model\").",
                "Search is a combobox controlling a listbox; unavailable models are aria-disabled with the reason.",
                "Capability badges have text alternatives, never colour only.",
            ],
            responsive: vec![
                "compact shows only the short name; the list opens upward from a bottom composer.",
            ],
            height: 460,
            align: Align::End,
            live: true,
        },
        Demo {
            slug: "effort-control",
            name: "Effort control",
            category: "composer",
            summary: "How hard the model thinks: signal bars, segments, a slider, chevrons, a menu or a toggle; a change made mid-run applies to the next message.",
            dioxus: "EffortChip",
            gpui: "EffortChipEl",
            spec: "composer-effort-control",
            source: "composer/effort.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &[
                        "signal-bars",
                        "segmented",
                        "slider",
                        "chevrons",
                        "menu",
                        "toggle",
                    ],
                    "signal-bars",
                ),
                sel("level", "Level", &["low", "medium", "high", "max"], "high"),
                tog("open", "Menu open", false),
                tog("pending", "Pending next turn", false),
            ],
            variants: vec![
                var("signal-bars", "Bars that fill.", &[]),
                var(
                    "segmented",
                    "Low to Max segments.",
                    &[("variant", "segmented")],
                ),
                var(
                    "menu",
                    "A menu with descriptions.",
                    &[("variant", "menu"), ("open", "true")],
                ),
                var(
                    "pending",
                    "Lowered mid-run, applies next turn.",
                    &[("pending", "true"), ("level", "max")],
                ),
            ],
            render: effort,
            code: effort_code,
            keyboard: vec![
                ("Alt + . / Alt + ,", "Raise / lower (from the composer)"),
                ("← / →", "Move between levels (segmented, slider)"),
            ],
            a11y: vec![
                "Segmented and menu are a radiogroup; slider is a slider with aria-valuetext (\"High, deeper reasoning, slower\").",
                "Tier colours never carry meaning alone: label and glyph shape change too.",
            ],
            responsive: vec![
                "signal-bars and chevrons fit a dense toolbar; segmented needs the most width.",
            ],
            height: 300,
            align: Align::End,
            live: false,
        },
        Demo {
            slug: "context-meter",
            name: "Context meter",
            category: "composer",
            summary: "How full the context window is: a ring, a segmented ring, a bar, a grid, text or a card, warning and critical levels, and a breakdown by system, tools, files and history.",
            dioxus: "ContextMeterChip",
            gpui: "ContextMeterEl",
            spec: "composer-context-meter",
            source: "composer/context.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["ring", "ring-segmented", "bar", "grid", "text", "card"],
                    "ring",
                ),
                sel(
                    "level",
                    "Level",
                    &["normal", "warning", "critical"],
                    "normal",
                ),
                tog("open", "Breakdown open", false),
            ],
            variants: vec![
                var("ring", "A ring that fills.", &[]),
                var(
                    "critical",
                    "Nearly full; compact suggested.",
                    &[("level", "critical")],
                ),
                var(
                    "grid",
                    "A grid of cells by category.",
                    &[("variant", "grid")],
                ),
                var(
                    "card",
                    "The full breakdown.",
                    &[("variant", "card"), ("level", "warning")],
                ),
            ],
            render: context,
            code: context_code,
            keyboard: vec![("Enter / Space", "Open the breakdown"), ("Esc", "Close it")],
            a11y: vec![
                "A meter inside a button: aria-valuenow, aria-valuemax and a valuetext like \"112 thousand of 200 thousand tokens used\".",
                "Crossing warning or critical is announced politely; routine token changes never are.",
            ],
            responsive: vec!["ring and text fit a toolbar; card is for a side panel."],
            height: 340,
            align: Align::End,
            live: false,
        },
        Demo {
            slug: "attachments",
            name: "Attachments",
            category: "composer",
            summary: "Files and context in the draft: upload rings, ready, reading and failed states with retry, too-large rejections, context chips for files, repos, selections and URLs, and drop zones.",
            dioxus: "AttachmentRow, DropZone",
            gpui: "AttachmentRowEl",
            spec: "composer-attachments",
            source: "composer/attachments.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &[
                        "tile-ring",
                        "thumb-strip",
                        "chip",
                        "chip-bar",
                        "list",
                        "terminal",
                        "context-chip",
                    ],
                    "tile-ring",
                ),
                sel("drop", "Drop zone", &["none", "valid", "invalid"], "none"),
                tog("reduced", "Reduced motion", false),
            ],
            variants: vec![
                var("tile-ring", "Tiles with upload rings.", &[]),
                var("chip", "Compact chips.", &[("variant", "chip")]),
                var("list", "A list with sizes.", &[("variant", "list")]),
                var(
                    "context-chip",
                    "Files, repo, selection and URL.",
                    &[("variant", "context-chip")],
                ),
                var("drop", "Drop target over the row.", &[("drop", "valid")]),
            ],
            render: attachments,
            code: attachments_code,
            keyboard: vec![
                ("Tab", "To each attachment and its remove button"),
                ("Enter", "Retry a failed upload"),
            ],
            a11y: vec![
                "A list labelled \"Attachments, 4 items\"; each item reads its name, type, size and state.",
                "Progress is announced at 25 % steps at most; errors once, assertively.",
                "Drag and drop always has an alternative (attach button, paste).",
            ],
            responsive: vec![
                "max_visible slots, then \"+k more\"; chip and context-chip suit narrow composers.",
            ],
            height: 260,
            align: Align::Start,
            live: true,
        },
    ]
}
