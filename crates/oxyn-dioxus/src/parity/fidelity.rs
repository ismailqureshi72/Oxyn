//! affectUI fidelity pass (plan #32): exact rebuilds of affectUI's **Chat Composer**, **Comment
//! Thread** (`AgentCommentThread`) and **Comments Panel**, plus the drawn file-type icons and
//! portraits they use. Behaviour and look only; models in `oxyn_core::parity::fidelity`.
//! Styles: `fidelity.css` (classes `oxcc-*`, `oxct-*`, `oxcp-*`, `oxfi-*`), in `PRIMITIVES_CSS`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        compose::dictated,
        fidelity::{
            Approval, Attachment, EditRun, FileKind, PanelComment, PanelFilter, ago, attach,
            avatar_data_uri, is_edit, meter_levels, open_edits, panel_view, split_command,
        },
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// Small stroke icons the shared set does not have.
fn glyph(name: &str) -> &'static str {
    match name {
        "shield" => {
            "M12 3.5 5 6.2v5.3c0 4.3 2.9 7.9 7 9 4.1-1.1 7-4.7 7-9V6.2zM12 8.5v4.5M12 16v.01"
        }
        "target" => {
            "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 16.5a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9zM12 12.5a.5.5 0 1 0 0-1 .5.5 0 0 0 0 1z"
        }
        "bulb" => {
            "M9 18h6M10 21h4M12 3a6 6 0 0 0-3.6 10.8c.7.6 1.1 1.3 1.1 2.2h5c0-.9.4-1.6 1.1-2.2A6 6 0 0 0 12 3z"
        }
        "record" => {
            "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7z"
        }
        "scribble" => "M3 15c2.5-4 4-6 5-6s.5 4 2 4 3.5-6 5-6 1 5 2.5 5 2-2 3.5-3M4 19h16",
        "sort" => "M4 6h16M7 12h10M10 18h4",
        _ => "",
    }
}

#[component]
fn Glyph(name: String, #[props(default = 16.0)] size: f32) -> Element {
    rsx! {
        svg { class: "oxfi-g", width: "{size}", height: "{size}", view_box: "0 0 24 24", fill: "none", stroke: "currentColor",
            stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "{glyph(&name)}" }
        }
    }
}

/// A file-type icon: a page with a folded corner in the type's colour and its badge (PDF, XLS,
/// DOC, `</>`), a landscape for images and a filled folder for folders.
#[component]
pub fn FileTypeIcon(
    /// `pdf`, `image`, `sheet`, `doc`, `code` or `folder`.
    kind: String,
    /// Size px.
    #[props(default = 20.0)]
    size: f32,
    /// Text lines on the page instead of the badge (the plugin marks in the plus menu).
    #[props(default)]
    plain: bool,
) -> Element {
    let k = FileKind::parse(&kind).unwrap_or(FileKind::Doc);
    let c = format!("#{}", k.hex());
    match k {
        FileKind::Folder => rsx! {
            svg { class: "oxfi", width: "{size}", height: "{size}", view_box: "0 0 24 24", "aria-hidden": "true",
                path { d: "M2.5 6.5A2 2 0 0 1 4.5 4.5h4.2l2 2.2h8.8a2 2 0 0 1 2 2v1H2.5z", fill: "{c}", opacity: ".75" }
                path { d: "M2.5 9.2h19v8.8a2 2 0 0 1-2 2h-15a2 2 0 0 1-2-2z", fill: "{c}" }
            }
        },
        FileKind::Image => rsx! {
            svg { class: "oxfi", width: "{size}", height: "{size}", view_box: "0 0 24 24", "aria-hidden": "true",
                rect { x: "3", y: "3.5", width: "18", height: "17", rx: "4", fill: "{c}" }
                circle { cx: "9", cy: "9.5", r: "2", fill: "#fff", opacity: ".95" }
                path { d: "M3 17.5 8.5 13l3.5 3 3-2.5 6 4.5v1a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 18.5z", fill: "#fff", opacity: ".9" }
            }
        },
        _ => rsx! {
            svg { class: "oxfi", width: "{size}", height: "{size}", view_box: "0 0 24 24", "aria-hidden": "true",
                path { d: "M6 2.5h8.5L20 8v12a1.5 1.5 0 0 1-1.5 1.5h-12A1.5 1.5 0 0 1 5 20V4A1.5 1.5 0 0 1 6 2.5z", fill: "{c}" }
                path { d: "M14.5 2.5V7a1 1 0 0 0 1 1H20z", fill: "#fff", opacity: ".45" }
                if plain {
                    path { d: "M8.5 12.5h7M8.5 15.5h7M8.5 18.5h4.5", stroke: "#fff", stroke_width: "1.5", stroke_linecap: "round", opacity: ".95" }
                } else {
                    text { x: "12.5", y: "17.2", text_anchor: "middle", font_size: if k == FileKind::Code { "5.6" } else { "5" }, font_weight: "700", fill: "#fff", font_family: "ui-sans-serif,system-ui,sans-serif", "{k.badge()}" }
                }
            }
        },
    }
}

/// A drawn portrait avatar (head and shoulders on a soft backdrop); `seed` picks the palette.
#[component]
pub fn Portrait(
    /// Palette seed.
    seed: u8,
    /// Size px.
    #[props(default = 24.0)]
    size: f32,
) -> Element {
    const BG: [(&str, &str); 6] = [
        ("#F3D9C9", "#E9B8A0"),
        ("#CFE0F2", "#9DBEE3"),
        ("#E2D6F3", "#BCA6E3"),
        ("#D6EBDD", "#A5CFB2"),
        ("#F2E3C2", "#E2C485"),
        ("#E8D3D9", "#CFA2AE"),
    ];
    const SKIN: [&str; 6] = [
        "#E9B99A", "#C98E6C", "#F1C9A8", "#8D5A3F", "#DDA77F", "#B5774F",
    ];
    const HAIR: [&str; 6] = [
        // warm-ok: illustration hair, not chrome
        "#3B2A22", "#1F1A17", "#A0652E", "#2B2320", "#5A3A26", "#14110F",
    ];
    const TOP: [&str; 6] = [
        "#4B5563", "#1F2937", "#5C5C66", "#2F4858", "#6B4E71", "#3F3F46",
    ];
    let i = seed as usize % 6;
    let (a, b) = BG[i];
    let gid = format!("oxpt{i}");
    let long = seed % 2 == 0;
    rsx! {
        svg { class: "oxpt", width: "{size}", height: "{size}", view_box: "0 0 32 32", "aria-hidden": "true",
            defs { linearGradient { id: "{gid}", x1: "0", y1: "0", x2: "0", y2: "1", stop { offset: "0", stop_color: "{a}" } stop { offset: "1", stop_color: "{b}" } } }
            circle { cx: "16", cy: "16", r: "16", fill: "url(#{gid})" }
            if long { path { d: "M8.6 20c-.6-6 1-11.6 7.4-11.6S24 14 23.4 20z", fill: "{HAIR[i]}" } }
            path { d: "M5 32c.6-5.6 4.8-8.6 11-8.6S26.4 26.4 27 32z", fill: "{TOP[i]}" }
            rect { x: "13.4", y: "17.5", width: "5.2", height: "6.5", rx: "2.4", fill: "{SKIN[i]}" }
            ellipse { cx: "16", cy: "14", rx: "5.2", ry: "5.8", fill: "{SKIN[i]}" }
            path { d: if long { "M10.6 13.4c.2-4 2.4-6.2 5.4-6.2s5.4 2.2 5.4 6.2c-1.6-1.8-3.6-2.8-5.4-2.8s-3.8 1-5.4 2.8z" } else { "M10.8 12.6c.4-3.4 2.4-5 5.2-5s4.8 1.6 5.2 5c-1.4-1-3.2-1.6-5.2-1.6s-3.8.6-5.2 1.6z" }, fill: "{HAIR[i]}" }
            circle { cx: "16", cy: "16", r: "15.5", fill: "none", stroke: "rgba(0,0,0,.08)" }
        }
    }
}

/// A round photo avatar (generated portraits of fictional people, `seed` picks one of six), or
/// the drawn [`Portrait`] when `drawn`.
#[component]
pub fn PhotoAvatar(
    /// Portrait seed.
    seed: u8,
    /// Size px.
    #[props(default = 24.0)]
    size: f32,
    /// Draw the illustrated portrait instead of the photo.
    #[props(default)]
    drawn: bool,
) -> Element {
    if drawn {
        return rsx! { Portrait { seed, size } };
    }
    let src = avatar_data_uri(seed);
    rsx! { img { class: "oxpa", src: "{src}", width: "{size}", height: "{size}", alt: "", draggable: "false" } }
}

/// The agent's gradient orb avatar.
#[component]
pub fn AgentOrb(
    /// Size px.
    #[props(default = 24.0)]
    size: f32,
) -> Element {
    rsx! {
        svg { class: "oxorb", width: "{size}", height: "{size}", view_box: "0 0 32 32", "aria-hidden": "true",
            defs { radialGradient { id: "oxorbg", cx: "35%", cy: "30%", r: "75%",
                stop { offset: "0", stop_color: "#9AD0FF" } stop { offset: ".45", stop_color: "#4C7DFF" } stop { offset: "1", stop_color: "#7B3FF2" } } }
            circle { cx: "16", cy: "16", r: "16", fill: "url(#oxorbg)" }
        }
    }
}

fn default_models() -> Vec<String> {
    [
        "Auto",
        "Claude Fable 5.1",
        "Claude Opus 5.5",
        "Claude Sonnet 5.5",
        "Claude Haiku 5.5",
        "GPT-5",
        "GPT-5 mini",
    ]
    .map(String::from)
    .to_vec()
}

fn default_folders() -> Vec<String> {
    ["Q3 Campaign", "Brand assets", "Hiring", "Research"]
        .map(String::from)
        .to_vec()
}

fn default_apps() -> Vec<(String, String)> {
    [
        ("gmail", "Gmail"),
        ("slack", "Slack"),
        ("clickup", "ClickUp"),
    ]
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .to_vec()
}

/// The plus menu's "Add" group: key (`onAttach` value), label, hint shown after the label.
const PLUS: [(&str, &str, &str); 7] = [
    ("files", "Files and folders", ""),
    ("claude", "Attach Claude", ""),
    (
        "folder",
        "Work in a folder",
        "Choose a folder for this chat",
    ),
    ("goal", "Goal", "Set a goal to keep working toward"),
    ("plan", "Plan mode", "Turn on plan mode"),
    ("skill", "Record a skill", ""),
    ("draw", "Draw", "Sketch a drawing"),
];

/// The plus menu's "Plugins" group.
const PLUGINS: [(&str, &str, &str); 2] = [
    ("documents", "Documents", "Create and edit documents"),
    ("pdf", "PDF", "Read, create and verify PDFs"),
];

fn plus_icon(key: &str) -> Element {
    match key {
        "files" => rsx! { Icon { name: "clip", size: 17.0 } },
        "claude" => rsx! { BrandIcon { name: "claude".to_string(), size: 17.0, coloured: true } },
        "folder" => rsx! { Icon { name: "folder", size: 17.0 } },
        "goal" => rsx! { Glyph { name: "target".to_string(), size: 17.0 } },
        "plan" => rsx! { Glyph { name: "bulb".to_string(), size: 17.0 } },
        "skill" => rsx! { Glyph { name: "record".to_string(), size: 17.0 } },
        _ => rsx! { Glyph { name: "scribble".to_string(), size: 17.0 } },
    }
}

/// Chat Composer (affectUI): a large message field; a row with **+** (a capped, scrolling drop-up
/// list spanning the composer: Files and folders, Attach Claude, Work in a folder · Goal, Plan
/// mode, Record a skill, Draw · Plugins: Documents, PDF), the approval chip (Request
/// approval, Approve for me, or the orange **Unrestricted access**), the **Auto** model menu with
/// coloured brand marks, the mic (recording mode: a live trace, the words arriving in grey, ✕ to
/// discard and ✓ to keep them) and a white send button. Under it, the tray: a blue folder for
/// "Work in a folder" and "Connect apps" with coloured Gmail, Slack and ClickUp marks and a check
/// menu. `style` picks Clean (one continuous container, the tray under a hairline) or Classic
/// (an inset card on a darker band); empty follows the nearest `data-cstyle`.
#[component]
pub fn ChatComposer(
    /// Placeholder.
    #[props(default = "Start by typing…".to_string())]
    placeholder: String,
    /// Starting text.
    #[props(default)]
    text: String,
    /// Model names; the first is selected (Auto draws a sparkle, known brands their mark).
    #[props(default = default_models())]
    models: Vec<String>,
    /// Demo transcript that arrives word by word while recording.
    #[props(default = "Draft a short update for the team about this week's progress".to_string())]
    dictation: String,
    /// Starting approval mode: `ask`, `auto` or `full`.
    #[props(default = "ask".to_string())]
    approval: String,
    /// Folders in "Work in a folder"; empty leaves the picker out.
    #[props(default = default_folders())]
    folders: Vec<String>,
    /// Starting folder.
    #[props(default)]
    folder: Option<String>,
    /// Apps (id, name) in "Connect apps"; the first three show their marks.
    #[props(default = default_apps())]
    apps: Vec<(String, String)>,
    /// Connected app ids at the start.
    #[props(default = vec!["gmail".to_string()])]
    connected: Vec<String>,
    /// The tray under the card (folder and apps). Off once a chat has started: the apps move
    /// into the plus menu.
    #[props(default = true)]
    tray: bool,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Attachment kinds to start with (`pdf`, `image`, …).
    #[props(default)]
    attachments: Vec<String>,
    /// Start in recording mode (the words already arriving).
    #[props(default)]
    recording: bool,
    /// A menu open from the start: `plus`, `approval`, `model`, `folder` or `apps`.
    #[props(default)]
    default_open: Option<String>,
    /// Shared clock (recording trace and words).
    now: ClockMs,
    /// Sent: text and model.
    #[props(default)]
    on_send: EventHandler<(String, String)>,
    /// Model picked.
    #[props(default)]
    on_model_change: EventHandler<String>,
    /// Plus menu entry picked: `files`, `claude`, `folder` (also opens the folder picker),
    /// `goal`, `plan` (toggles plan mode), `skill`, `draw`, `documents` or `pdf`.
    #[props(default)]
    on_attach: EventHandler<String>,
    /// Approval mode picked.
    #[props(default)]
    on_approval_change: EventHandler<String>,
    /// Folder picked (`None` when the picked folder is picked again).
    #[props(default)]
    on_folder_change: EventHandler<Option<String>>,
    /// Connected app ids.
    #[props(default)]
    on_apps_change: EventHandler<Vec<String>>,
) -> Element {
    let mut draft = use_signal(move || text.clone());
    let mut model = use_signal(|| 0usize);
    let start_open = default_open.clone();
    let mut open = use_signal(move || start_open.clone().unwrap_or_default());
    let mut veil = use_signal(|| false);
    let mut rec = use_signal(move || recording.then(|| ClockMs(now.0.saturating_sub(2_600))));
    let mut appr = use_signal(move || Approval::parse(&approval));
    let mut fold = use_signal(move || folder.clone());
    let mut conn = use_signal(move || connected.clone());
    let mut plan = use_signal(|| false);
    let mut atts = use_signal(move || {
        let mut v: Vec<Attachment> = vec![];
        for k in &attachments {
            if let Some(k) = FileKind::parse(k) {
                attach(&mut v, k);
            }
        }
        v
    });
    let mut live = use_signal(String::new);
    let mut field = use_signal(|| None::<Rc<MountedData>>);
    let names = use_signal(move || models.clone());
    let menu = open();
    let heard = rec().map(|t| dictated(&dictation, now.since(t).as_millis() as u64, 190));
    let trace = rec()
        .map(|t| meter_levels(now.since(t).as_millis() as u64, 16))
        .unwrap_or_default();
    let can_send = !draft().trim().is_empty() || !atts().is_empty();
    let cur = names.peek().get(model()).cloned().unwrap_or_default();
    let short = cur.strip_prefix("Claude ").unwrap_or(&cur).to_string();
    let mut toggle = move |m: &str| {
        if open() == m {
            open.set(String::new());
        } else {
            open.set(m.to_string());
            veil.set(true);
        }
    };
    let mut close = move || {
        open.set(String::new());
        veil.set(false);
    };
    let mut send = move || {
        let t = draft().trim().to_string();
        if t.is_empty() && atts().is_empty() {
            return;
        }
        let m = names.peek().get(model()).cloned().unwrap_or_default();
        on_send.call((t, m));
        draft.set(String::new());
        atts.set(vec![]);
        live.set("Message sent.".into());
    };
    let dict2 = dictation.clone();
    let mut keep = move || {
        if let Some(t) = rec() {
            let h = dictated(&dict2, now.since(t).as_millis() as u64, 190);
            if !h.is_empty() {
                let d = draft();
                draft.set(if d.trim().is_empty() {
                    h
                } else {
                    format!("{} {h}", d.trim_end())
                });
            }
            rec.set(None);
            live.set("Recording added to your message.".into());
            focus(field());
        }
    };
    let style_attr = if style.is_empty() {
        None
    } else {
        Some(style.clone())
    };
    let mode = appr();
    rsx! {
        div { class: "oxcc", "data-style": style_attr, "data-tray": "{tray}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape {
                if !open().is_empty() { close() } else if rec().is_some() { rec.set(None); live.set("Recording discarded.".into()) }
            },
            if veil() && !menu.is_empty() { div { class: "oxcc-veil", onclick: move |_| close() } }
            div { class: "oxcc-band",
                div { class: "oxcc-card", onclick: move |_| focus(field()),
                    if !atts().is_empty() {
                        div { class: "oxcc-atts", role: "list", "aria-label": "Attachments",
                            for a in atts() {
                                {
                                    let id = a.id;
                                    rsx! {
                                        div { key: "{a.id}", class: "oxcc-att", role: "listitem", "data-kind": "{a.kind.key()}",
                                            if a.kind == FileKind::Image {
                                                span { class: "oxcc-thumb", "aria-hidden": "true" }
                                            } else {
                                                FileTypeIcon { kind: a.kind.key().to_string(), size: 22.0 }
                                            }
                                            span { class: "oxcc-attn",
                                                span { class: "oxcc-attt", "{a.name}" }
                                                span { class: "oxcc-attk", "{a.kind.label()}" }
                                            }
                                            button { class: "oxcc-attx", r#type: "button", "aria-label": "Remove {a.name}",
                                                onclick: move |e| { e.stop_propagation(); atts.write().retain(|x| x.id != id) },
                                                Icon { name: "x", size: 11.0, stroke: 2.0 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxcc-field",
                        if let Some(h) = &heard {
                            div { class: "oxcc-heard", "aria-live": "polite",
                                if !draft().is_empty() { span { class: "oxcc-had", "{draft} " } }
                                if h.is_empty() { span { class: "oxcc-ph", "Listening…" } } else { "{h}" }
                            }
                        } else {
                            textarea { class: "oxcc-ta", rows: "1", value: "{draft}", placeholder: "{placeholder}", "aria-label": "Message",
                                onmounted: move |e| field.set(Some(e.data())),
                                oninput: move |e| draft.set(e.value()),
                                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send() },
                            }
                        }
                    }
                    div { class: "oxcc-bar",
                        div { class: "oxcc-wrap oxcc-plusw",
                            button { class: "oxcc-ib", r#type: "button", "aria-label": "Add", "aria-haspopup": "menu", "aria-expanded": "{menu == \"plus\"}",
                                onclick: move |e| { e.stop_propagation(); toggle("plus") },
                                Icon { name: "plus", size: 18.0 }
                            }
                        }
                        if menu == "plus" {
                            div { class: "oxcc-menu oxcc-plus", role: "menu", "aria-label": "Add to the chat", onclick: move |e| e.stop_propagation(),
                                div { class: "oxcc-scroll",
                                    div { class: "oxcc-mh", "Add" }
                                    for (key, label, hint) in PLUS {
                                        button { key: "{key}", class: "oxcc-mi", role: if key == "plan" { "menuitemcheckbox" } else { "menuitem" }, r#type: "button",
                                            "aria-checked": if key == "plan" { Some(plan().to_string()) } else { None },
                                            "data-tool": "{key}",
                                            onclick: move |_| {
                                                on_attach.call(key.to_string());
                                                match key {
                                                    "plan" => { let p = !plan(); plan.set(p); live.set(if p { "Plan mode on.".into() } else { "Plan mode off.".into() }) }
                                                    "folder" => { open.set("folder".into()); }
                                                    "files" => { attach(&mut atts.write(), FileKind::Doc); close(); live.set("File added.".into()) }
                                                    _ => close(),
                                                }
                                            },
                                            span { class: "oxcc-mic", {plus_icon(key)} }
                                            span { class: "oxcc-mt", "{label}" }
                                            if !hint.is_empty() { span { class: "oxcc-md", "{hint}" } }
                                            if key == "plan" { span { class: if plan() { "oxcc-sw on" } else { "oxcc-sw" }, span {} } }
                                        }
                                        if key == "folder" { div { class: "oxcc-sep", role: "separator" } }
                                    }
                                    div { class: "oxcc-sep", role: "separator" }
                                    div { class: "oxcc-mh", "Plugins" }
                                    for (key, label, hint) in PLUGINS {
                                        button { key: "{key}", class: "oxcc-mi", role: "menuitem", r#type: "button", "data-tool": "{key}",
                                            onclick: move |_| { on_attach.call(key.to_string()); close(); live.set(format!("{label} plugin added.")) },
                                            span { class: "oxcc-mic", FileTypeIcon { kind: if key == "pdf" { "pdf".to_string() } else { "doc".to_string() }, size: 17.0, plain: true } }
                                            span { class: "oxcc-mt", "{label}" }
                                            span { class: "oxcc-md", "{hint}" }
                                        }
                                    }
                                    if !tray {
                                        div { class: "oxcc-sep", role: "separator" }
                                        div { class: "oxcc-mh", "Apps" }
                                        for (id, name) in apps.iter() {
                                            {
                                                let id2 = id.clone();
                                                let on = conn().contains(id);
                                                rsx! {
                                                    button { key: "{id}", class: "oxcc-mi", role: "menuitemcheckbox", "aria-checked": "{on}", r#type: "button",
                                                        onclick: move |_| { let mut c = conn.write(); if let Some(p) = c.iter().position(|x| *x == id2) { c.remove(p); } else { c.push(id2.clone()); } on_apps_change.call(c.clone()); },
                                                        span { class: "oxcc-mic", BrandIcon { name: name.clone(), size: 16.0, coloured: true, fallback: true } }
                                                        span { class: "oxcc-mt", "{name}" }
                                                        if on { span { class: "oxcc-ck", Icon { name: "check", size: 14.0, stroke: 2.0 } } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "oxcc-wrap",
                            button { key: "{mode.key()}", class: "oxcc-chip", "data-mode": "{mode.key()}", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{menu == \"approval\"}",
                                onclick: move |e| { e.stop_propagation(); toggle("approval") },
                                span { class: "oxcc-ci",
                                    match mode {
                                        Approval::Ask => rsx! { Icon { name: "hand", size: 17.0 } },
                                        Approval::Auto => rsx! { Icon { name: "check-circle", size: 17.0 } },
                                        Approval::Full => rsx! { Glyph { name: "shield".to_string(), size: 17.0 } },
                                    }
                                }
                                span { class: "oxcc-cl", "{mode.chip()}" }
                            }
                            if menu == "approval" {
                                div { class: "oxcc-menu oxcc-appr", role: "listbox", "aria-label": "How should actions be approved?", onclick: move |e| e.stop_propagation(),
                                    div { class: "oxcc-mh row", span { "How should actions be approved?" } a { class: "oxcc-more", href: "#approvals", onclick: move |e| e.prevent_default(), "Learn more" } }
                                    for m in Approval::ALL {
                                        button { key: "{m.key()}", class: "oxcc-mi tall", "data-mode": "{m.key()}", role: "option", "aria-selected": "{mode == m}", r#type: "button",
                                            onclick: move |_| { appr.set(m); on_approval_change.call(m.key().to_string()); close(); live.set(format!("{}.", m.chip())) },
                                            span { class: "oxcc-mic",
                                                match m {
                                                    Approval::Ask => rsx! { Icon { name: "hand", size: 17.0 } },
                                                    Approval::Auto => rsx! { Icon { name: "check-circle", size: 17.0 } },
                                                    Approval::Full => rsx! { Glyph { name: "shield".to_string(), size: 17.0 } },
                                                }
                                            }
                                            span { class: "oxcc-ml", span { class: "oxcc-mt", "{m.label()}" } span { class: "oxcc-md", "{m.desc()}" } }
                                            if mode == m { span { class: "oxcc-ck", Icon { name: "check", size: 14.0, stroke: 2.0 } } }
                                        }
                                    }
                                }
                            }
                        }
                        span { class: "oxcc-sp" }
                        if rec().is_some() {
                            div { class: "oxcc-trace", "aria-hidden": "true",
                                for (i, l) in trace.iter().enumerate() {
                                    span { key: "{i}", style: "height:{(l * 16.0).max(3.5):.1}px" }
                                }
                            }
                            button { class: "oxcc-rb", r#type: "button", "aria-label": "Discard recording", onclick: move |e| { e.stop_propagation(); rec.set(None); live.set("Recording discarded.".into()) }, Icon { name: "x", size: 15.0, stroke: 1.8 } }
                            button { class: "oxcc-rb ok", r#type: "button", "aria-label": "Use recording", onclick: move |e| { e.stop_propagation(); keep() }, Icon { name: "check", size: 15.0, stroke: 1.8 } }
                        } else {
                            div { class: "oxcc-wrap",
                                button { class: "oxcc-model", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{menu == \"model\"}", "aria-label": "Model: {cur}",
                                    onclick: move |e| { e.stop_propagation(); toggle("model") },
                                    span { key: "{cur}", class: "oxcc-mname", "{short}" }
                                    Icon { name: "chevron", size: 13.0 }
                                }
                                if menu == "model" {
                                    div { class: "oxcc-menu oxcc-models", role: "listbox", "aria-label": "Select a model", onclick: move |e| e.stop_propagation(),
                                        div { class: "oxcc-scroll",
                                        for (i, m) in names().into_iter().enumerate() {
                                            button { key: "{m}", class: "oxcc-mi", role: "option", "aria-selected": "{model() == i}", r#type: "button",
                                                onclick: { let m = m.clone(); move |_| { model.set(i); on_model_change.call(m.clone()); close(); live.set(format!("{m} selected.")) } },
                                                span { class: "oxcc-mic", if m == "Auto" { Icon { name: "spark", size: 16.0 } } else { BrandIcon { name: m.clone(), size: 16.0, coloured: true, fallback: true } } }
                                                span { class: "oxcc-mt", "{m}" }
                                                if model() == i { span { class: "oxcc-ck", Icon { name: "check", size: 14.0, stroke: 2.0 } } }
                                            }
                                        }
                                        }
                                    }
                                }
                            }
                            button { class: "oxcc-ib", r#type: "button", "aria-label": "Voice input", onclick: move |e| { e.stop_propagation(); close(); rec.set(Some(now)); live.set("Recording. Press Escape to discard.".into()) }, Icon { name: "mic", size: 18.0 } }
                            button { class: "oxcc-send", r#type: "button", disabled: !can_send, "aria-label": "Send message", onclick: move |e| { e.stop_propagation(); send() }, Icon { name: "up", size: 16.0, stroke: 2.0 } }
                        }
                    }
                }
                if tray {
                    div { class: "oxcc-tray",
                        if !folders.is_empty() {
                            div { class: "oxcc-wrap",
                                button { class: "oxcc-tb", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{menu == \"folder\"}", title: fold().unwrap_or_default(),
                                    onclick: move |e| { e.stop_propagation(); toggle("folder") },
                                    FileTypeIcon { kind: "folder", size: 17.0 }
                                    span { class: "oxcc-tl", {fold().unwrap_or_else(|| "Work in a folder".to_string())} }
                                }
                                if menu == "folder" {
                                    div { class: "oxcc-menu oxcc-down oxcc-folders", role: "listbox", "aria-label": "Work in a folder", onclick: move |e| e.stop_propagation(),
                                        div { class: "oxcc-mh", "Work in a folder" }
                                        for f in folders.iter() {
                                            {
                                                let f2 = f.clone();
                                                let on = fold().as_deref() == Some(f.as_str());
                                                rsx! {
                                                    button { key: "{f}", class: "oxcc-mi", role: "option", "aria-selected": "{on}", r#type: "button",
                                                        onclick: move |_| { let n = if fold().as_deref() == Some(f2.as_str()) { None } else { Some(f2.clone()) }; fold.set(n.clone()); on_folder_change.call(n); close() },
                                                        span { class: "oxcc-mic", Icon { name: "folder", size: 16.0 } }
                                                        span { class: "oxcc-mt", "{f}" }
                                                        if on { span { class: "oxcc-ck", Icon { name: "check", size: 14.0, stroke: 2.0 } } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        span { class: "oxcc-sp" }
                        div { class: "oxcc-wrap",
                            button { class: "oxcc-tb", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu == \"apps\"}",
                                onclick: move |e| { e.stop_propagation(); toggle("apps") },
                                span { class: "oxcc-tl", "Connect apps" }
                                span { class: "oxcc-marks",
                                    for (id, name) in apps.iter().take(3) {
                                        span { key: "{id}", class: "oxcc-mark", BrandIcon { name: name.clone(), size: 15.0, coloured: true, fallback: true } }
                                    }
                                }
                            }
                            if menu == "apps" {
                                div { class: "oxcc-menu oxcc-down oxcc-end oxcc-apps", role: "menu", "aria-label": "Apps", onclick: move |e| e.stop_propagation(),
                                    div { class: "oxcc-mh", "Apps" }
                                    for (id, name) in apps.iter() {
                                        {
                                            let id2 = id.clone();
                                            let on = conn().contains(id);
                                            rsx! {
                                                button { key: "{id}", class: "oxcc-mi", role: "menuitemcheckbox", "aria-checked": "{on}", r#type: "button",
                                                    onclick: move |_| { let mut c = conn.write(); if let Some(p) = c.iter().position(|x| *x == id2) { c.remove(p); } else { c.push(id2.clone()); } on_apps_change.call(c.clone()); },
                                                    span { class: "oxcc-mic", BrandIcon { name: name.clone(), size: 16.0, coloured: true, fallback: true } }
                                                    span { class: "oxcc-mt", "{name}" }
                                                    span { class: "oxcc-ck", if on { Icon { name: "check", size: 14.0, stroke: 2.0 } } }
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
            div { class: "oxcc-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Comment Thread ──

/// One message in [`AgentCommentThread`].
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadNote {
    /// Author name.
    pub author: String,
    /// Portrait seed; `None` draws the agent orb.
    pub avatar: Option<u8>,
    /// Minutes ago (0 = just now).
    pub minutes: u32,
    /// Text (a leading `/edit` is greyed as a command).
    pub text: String,
    /// The agent's status line ("Edit completed") and its folded details.
    pub status: Option<(String, Vec<String>)>,
}

impl ThreadNote {
    /// A person's message.
    pub fn person(author: &str, avatar: u8, minutes: u32, text: &str) -> Self {
        Self {
            author: author.into(),
            avatar: Some(avatar),
            minutes,
            text: text.into(),
            status: None,
        }
    }
    /// The agent's message with a status line and details.
    pub fn agent(author: &str, minutes: u32, status: &str, details: &[&str]) -> Self {
        Self {
            author: author.into(),
            avatar: None,
            minutes,
            text: String::new(),
            status: Some((
                status.into(),
                details.iter().map(|s| s.to_string()).collect(),
            )),
        }
    }
}

/// Comment Thread (affectUI): one comment with replies. People show a portrait, name and time;
/// a leading `/Edit` is greyed as a command. The agent has a gradient orb and a "✓ Edit
/// completed" line that folds its details open. "Reply, or ask @Atlas…" posts your reply as
/// "just now"; the agent answers with a shimmering "Thinking…" first.
#[component]
pub fn AgentCommentThread(
    /// Messages.
    notes: Vec<ThreadNote>,
    /// Your name and portrait seed.
    #[props(default = ("Alex Rivera".to_string(), 3))]
    me: (String, u8),
    /// The agent's name.
    #[props(default = "Atlas".to_string())]
    agent: String,
    /// The agent's answer to a reply (empty: it does not answer).
    #[props(default = "Done – I also updated the meta description.".to_string())]
    answer: String,
    /// Card title.
    #[props(default = "Comment".to_string())]
    title: String,
    /// Fold the first status line open at the start.
    #[props(default)]
    open: bool,
    /// `photo` (portrait photos, as affectUI) or `drawn` (illustrated portraits).
    #[props(default = "photo".to_string())]
    avatars: String,
    /// Shared clock (thinking delay).
    now: ClockMs,
    /// A reply was posted.
    #[props(default)]
    on_reply: EventHandler<String>,
    /// Close pressed (shows the ✕).
    #[props(default)]
    on_close: Option<EventHandler<()>>,
) -> Element {
    // Each note carries the time it shows from (the agent's answer waits 1.4 s behind a reply).
    let mut list = use_signal(move || notes.iter().cloned().map(|n| (n, 0u64)).collect::<Vec<_>>());
    let mut unfold = use_signal(move || if open { vec![0usize] } else { vec![] });
    let mut draft = use_signal(String::new);
    let shown: Vec<ThreadNote> = list()
        .into_iter()
        .filter(|(_, at)| *at <= now.0)
        .map(|x| x.0)
        .collect();
    let thinking = list().iter().any(|(_, at)| *at > now.0);
    let n = shown.len();
    let me2 = use_signal(move || me.clone());
    let ag = use_signal(move || agent.clone());
    let answer2 = use_signal(move || answer.clone());
    let mut post = move || {
        let t = draft().trim().to_string();
        if t.is_empty() {
            return;
        }
        let (who, seed) = me2.peek().clone();
        let ans = answer2.peek().clone();
        let mut l = list.write();
        l.push((ThreadNote::person(&who, seed, 0, &t), 0));
        if !ans.is_empty() {
            l.push((
                ThreadNote {
                    author: ag.peek().clone(),
                    avatar: None,
                    minutes: 0,
                    text: ans,
                    status: None,
                },
                now.0 + 1_400,
            ));
        }
        draft.set(String::new());
        on_reply.call(t);
    };
    let ph = format!("Reply, or ask @{}…", ag());
    rsx! {
        section { class: "oxct", "aria-label": "{title}",
            header { class: "oxct-h",
                h3 { "{title}" }
                span { class: "oxct-n", if n == 1 { "1 message" } else { "{n} messages" } }
                span { class: "oxct-sp" }
                if let Some(c) = on_close { button { class: "oxct-x", r#type: "button", "aria-label": "Close comment", onclick: move |_| c.call(()), Icon { name: "x", size: 15.0 } } }
            }
            div { class: "oxct-body", role: "log", "aria-live": "polite",
                for (i, m) in shown.into_iter().enumerate() {
                    article { key: "{i}", class: "oxct-m",
                        div { class: "oxct-who",
                            match m.avatar { Some(s) => rsx! { PhotoAvatar { seed: s, size: 24.0, drawn: avatars == "drawn" } }, None => rsx! { AgentOrb { size: 24.0 } } }
                            b { "{m.author}" }
                            span { class: "oxct-sp" }
                            time { class: "oxct-t", "{ago(m.minutes)}" }
                        }
                        if !m.text.is_empty() {
                            {
                                let (cmd, rest) = split_command(&m.text);
                                rsx! { p { class: "oxct-p", if let Some(c) = cmd { span { class: "oxct-cmd", "{c} " } } "{rest}" } }
                            }
                        }
                        if let Some((st, det)) = m.status.clone() {
                            {
                                let on = unfold().contains(&i);
                                rsx! {
                                    button { class: "oxct-st", r#type: "button", "aria-expanded": "{on}",
                                        onclick: move |_| { let mut u = unfold.write(); if let Some(p) = u.iter().position(|x| *x == i) { u.remove(p); } else { u.push(i); } },
                                        span { class: "oxct-ok", Icon { name: "check", size: 10.0, stroke: 2.6 } }
                                        "{st}"
                                        span { class: if on { "oxct-chev up" } else { "oxct-chev" }, Icon { name: "chevron", size: 12.0 } }
                                    }
                                    if on {
                                        ul { class: "oxct-det",
                                            for (j, d) in det.iter().enumerate() { li { key: "{j}", "{d}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if thinking {
                    article { class: "oxct-m",
                        div { class: "oxct-who", AgentOrb { size: 24.0 } b { "{ag}" } span { class: "oxct-sp" } time { class: "oxct-t", "just now" } }
                        p { class: "oxct-p oxct-think", "Thinking…" }
                    }
                }
            }
            div { class: "oxct-reply",
                input { class: "oxct-in", value: "{draft}", placeholder: "{ph}", "aria-label": "Reply",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); post() },
                }
                button { class: "oxct-send", r#type: "button", disabled: draft().trim().is_empty(), "aria-label": "Send reply", onclick: move |_| post(), Icon { name: "up", size: 14.0, stroke: 2.0 } }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Comments Panel ──

/// Comments Panel (affectUI): a comment feed with All / Questions / Edits tabs (Edits counts the
/// open edits), a sort menu (Newest / Oldest first), a Resolve toggle per comment, runnable
/// `/edit` comments (Run edit → Running… → "Edit applied" with Undo) and a composer whose new
/// comment lands on top as "just now".
#[component]
pub fn CommentsPanel(
    /// Comments.
    comments: Vec<PanelComment>,
    /// Your name and portrait seed.
    #[props(default = ("Alex Rivera".to_string(), 3))]
    me: (String, u8),
    /// Title.
    #[props(default = "Comments".to_string())]
    title: String,
    /// Starting tab: `all`, `questions` or `edits`.
    #[props(default = "all".to_string())]
    tab: String,
    /// How long an edit runs, ms.
    #[props(default = 1_200)]
    run_ms: u64,
    /// `photo` (portrait photos, as affectUI) or `drawn` (illustrated portraits).
    #[props(default = "photo".to_string())]
    avatars: String,
    /// The list's height cap in px; the list scrolls inside it (0: no cap).
    #[props(default = 330.0)]
    max_list: f32,
    /// Shared clock (edit runs).
    now: ClockMs,
    /// An edit ran (comment id).
    #[props(default)]
    on_run_edit: EventHandler<u32>,
    /// A comment was added.
    #[props(default)]
    on_add: EventHandler<String>,
    /// Close pressed (shows the ✕).
    #[props(default)]
    on_close: Option<EventHandler<()>>,
) -> Element {
    let mut list = use_signal(move || comments.clone());
    let mut filt = use_signal(move || match tab.as_str() {
        "questions" => PanelFilter::Questions,
        "edits" => PanelFilter::Edits,
        _ => PanelFilter::All,
    });
    let mut oldest = use_signal(|| false);
    let mut sort_open = use_signal(|| false);
    let mut runs = use_signal(Vec::<(u32, ClockMs)>::new);
    let mut draft = use_signal(String::new);
    // An edit runs for `run_ms` from its press, then reads as applied (until Undo).
    let eff: Vec<PanelComment> = list()
        .into_iter()
        .map(|mut c| {
            if let Some((_, t)) = runs().iter().find(|x| x.0 == c.id) {
                c.run = if (now.since(*t).as_millis() as u64) < run_ms {
                    EditRun::Running
                } else {
                    EditRun::Applied
                };
            }
            c
        })
        .collect();
    let view = panel_view(&eff, filt(), oldest());
    let edits = open_edits(&eff);
    let me2 = use_signal(move || me.clone());
    let mut add = move || {
        let t = draft().trim().to_string();
        if t.is_empty() {
            return;
        }
        let id = list().iter().map(|c| c.id).max().unwrap_or(0) + 1;
        let (who, seed) = me2.peek().clone();
        list.write().push(PanelComment::new(id, &who, seed, 0, &t));
        draft.set(String::new());
        filt.set(PanelFilter::All);
        oldest.set(false);
        on_add.call(t);
    };
    let tabs = [
        (PanelFilter::All, "All"),
        (PanelFilter::Questions, "Questions"),
        (PanelFilter::Edits, "Edits"),
    ];
    rsx! {
        section { class: "oxcp", "aria-label": "{title}",
            header { class: "oxcp-h",
                h3 { "{title}" }
                span { class: "oxcp-sp" }
                if let Some(c) = on_close { button { class: "oxcp-x", r#type: "button", "aria-label": "Close comments", onclick: move |_| c.call(()), Icon { name: "x", size: 15.0 } } }
            }
            div { class: "oxcp-bar",
                div { class: "oxcp-tabs", role: "tablist", "aria-label": "Filter comments",
                    for (f, l) in tabs {
                        button { key: "{l}", class: "oxcp-tab", role: "tab", "aria-selected": "{filt() == f}", r#type: "button", onclick: move |_| filt.set(f),
                            "{l}"
                            if f == PanelFilter::Edits && edits > 0 { span { key: "{edits}", class: "oxcp-badge", "{edits}" } }
                        }
                    }
                }
                span { class: "oxcp-sp" }
                div { class: "oxcp-sortw",
                    button { class: "oxcp-ib", r#type: "button", "aria-label": "Sort comments", "aria-haspopup": "menu", "aria-expanded": "{sort_open()}", onclick: move |_| sort_open.set(!sort_open()), Glyph { name: "sort".to_string(), size: 15.0 } }
                    if sort_open() {
                        div { class: "oxcp-menu", role: "menu",
                            for (o, l) in [(false, "Newest first"), (true, "Oldest first")] {
                                button { key: "{l}", class: "oxcp-mi", role: "menuitemradio", "aria-checked": "{oldest() == o}", r#type: "button", onclick: move |_| { oldest.set(o); sort_open.set(false) },
                                    "{l}" if oldest() == o { span { class: "oxcp-ck", Icon { name: "check", size: 13.0, stroke: 2.0 } } }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxcp-list", role: "log", "aria-live": "polite", tabindex: "0", "aria-label": "Comment list",
                style: if max_list > 0.0 { format!("max-height:{max_list}px") } else { String::new() },
                if view.is_empty() { p { class: "oxcp-empty", "Nothing here yet" } }
                for i in view {
                    {
                        let c = eff[i].clone();
                        let id = c.id;
                        let (cmd, rest) = split_command(&c.text);
                        let running = c.run == EditRun::Running;
                        rsx! {
                            article { key: "{id}", class: if c.resolved { "oxcp-c done" } else { "oxcp-c" },
                                div { class: "oxcp-who",
                                    PhotoAvatar { seed: c.avatar, size: 24.0, drawn: avatars == "drawn" }
                                    b { "{c.author}" }
                                    time { class: "oxcp-t", "{ago(c.minutes)}" }
                                    span { class: "oxcp-sp" }
                                    button { class: if c.resolved { "oxcp-res on" } else { "oxcp-res" }, r#type: "button", "aria-pressed": "{c.resolved}",
                                        "aria-label": if c.resolved { "Reopen comment" } else { "Resolve comment" },
                                        onclick: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.resolved = !x.resolved } },
                                        Icon { name: "check-circle", size: 16.0 }
                                    }
                                }
                                p { class: "oxcp-p", if let Some(cm) = cmd { span { class: "oxcp-cmd", "{cm} " } } "{rest}" }
                                if is_edit(&c.text) && !c.resolved {
                                    div { class: "oxcp-act",
                                        if running {
                                            button { class: "oxcp-run", r#type: "button", disabled: true, span { class: "oxcp-spin" } "Running…" }
                                        } else if c.run == EditRun::Applied {
                                            span { class: "oxcp-ok", Icon { name: "check", size: 12.0, stroke: 2.2 } "Edit applied" }
                                            button { class: "oxcp-undo", r#type: "button", onclick: move |_| { runs.write().retain(|x| x.0 != id); if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.run = EditRun::Idle } }, "Undo" }
                                        } else {
                                            button { class: "oxcp-run", r#type: "button", onclick: move |_| { runs.write().retain(|x| x.0 != id); runs.write().push((id, now)); on_run_edit.call(id) }, Icon { name: "play", size: 10.0 } "Run edit" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxcp-comp",
                PhotoAvatar { seed: me2().1, size: 24.0, drawn: avatars == "drawn" }
                input { class: "oxcp-in", value: "{draft}", placeholder: "Add a comment or /edit…", "aria-label": "Add a comment",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); add() },
                }
                button { class: "oxcp-send", r#type: "button", disabled: draft().trim().is_empty(), "aria-label": "Send comment", onclick: move |_| add(), Icon { name: "up", size: 13.0, stroke: 2.0 } }
            }
        }
    }
}
