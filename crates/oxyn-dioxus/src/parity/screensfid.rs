//! Screens 1: whole app frames. [`AppFrame`] is the shared frame – a sidebar (workspace switcher,
//! live search, Pinned / Projects / Chats with pin and unread, a dismissible notice, a setup
//! checklist and the account menu), the page and an optional work area, laid out Inset (the page
//! a rounded card on a grey canvas) or Flat (edge to edge, hairlines), both resizable. [`RailFrame`]
//! is the three-pane frame: an icon rail, a chats column, the page and a work area. On top of them:
//! [`AppShellScreen`], [`RailShellScreen`] and the AI Chat screens [`ChatStartScreen`],
//! [`ChatOverviewScreen`], [`ChatAgentsScreen`], [`ChatRailScreen`], [`ChatProgressScreen`] and
//! [`ChatRunScreen`], the Folders screens ([`FoldersScreen`], Overview and Detail) and
//! [`IntegrationsScreen`], [`KanbanScreen`], [`CrmScreen`], [`WorkflowScreen`] and the
//! Settings shell [`SettingsScreen`], each with
//! Sample, Loading and Empty data. Models: `oxyn_core::parity::screensfid`. Styles:
//! `screensfid.css` (`oxscn-` frame, `oxsrl-` rail frame, `oxcst-` start, `oxcov-` overview,
//! `oxcag-` agents, `oxcdr-` double sidebar, `oxcpr-` progress, `oxcrn-` run, `oxsfo-` / `oxsfd-` /
//! `oxsfi-` folders, `oxsin-` integrations, `oxskb-` kanban, `oxscr-` CRM, `oxswf-` workflow, `oxsst-` settings). Every person, workspace and line of copy is fictional and our own.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::parity::screensfid::{
    APP_TABS, AppInfo, CRM_STAGES, CRM_VIEWS, Deal, DealSort, FileRow, FileSort, Folder, KB_COLS,
    KB_PRI, PERSON, PERSON_INI, Review, ScreenState, ShellLayout, Task, Thread, WORKSPACE,
    add_deal, add_task, ago, app_view, apps, board, can_send, chat_title, close_words, col_label,
    column, connected_count, deal_view, deals, due_words, folder_activity, folder_files,
    folder_view, folders, invite_initials, member_name, move_task, next_col, open_thread,
    over_limit, parse_money, pct_done, pipeline, projects, setup_steps, size_label, sort_files,
    split_pinned, stage_key, stage_label, stage_totals, synced_words, task_key, threads,
    toggle_pin, total_size, untitled, usd, view_count,
};
use oxyn_core::parity::screensfid::{STAGE_H, STAGE_STATES, STAGE_W};

use super::efflistfid::{check_glyph, chevron};
use crate::brand::BrandIcon;

pub(crate) fn ico(name: &str) -> Element {
    glyph(name, name)
}

/// A file glyph toned by the file's type (`report.pdf` red, `data.csv` green, `notes.md` blue).
pub(crate) fn file_ico(file: &str) -> Element {
    glyph("file", oxyn_core::icon_tone::IconTone::for_file(file).key())
}

/// The stroke glyph `name`, tagged `data-ic = tone` for icontone.css.
fn glyph(name: &str, tone: &str) -> Element {
    let d: &[&str] = match name {
        "plus" => &["M12 5v14", "M5 12h14"],
        "chat" => &["M5 5h14v10H9l-4 4z"],
        "folder" => &["M3 7h7l2 2h9v10H3z"],
        "apps" => &[
            "M4 4h6v6H4z",
            "M14 4h6v6h-6z",
            "M4 14h6v6H4z",
            "M14 14h6v6h-6z",
        ],
        "search" => &[
            "M10.5 4a6.5 6.5 0 1 0 0 13 6.5 6.5 0 1 0 0-13",
            "m15.5 15.5 4.5 4.5",
        ],
        "side" => &["M4 5h16v14H4z", "M10 5v14"],
        "pin" => &["M9 4h6l-1 6 3 3H7l3-3z", "M12 13v7"],
        "more" => &["M6 12h.01", "M12 12h.01", "M18 12h.01"],
        "close" => &["m6 6 12 12", "M18 6 6 18"],
        "mic" => &[
            "M9 5a3 3 0 0 1 6 0v6a3 3 0 0 1-6 0z",
            "M6 11a6 6 0 0 0 12 0",
            "M12 17v3",
        ],
        "send" => &["M12 19V5", "m6 11 6-6 6 6"],
        "clip" => &["M16 7 9 14a2 2 0 0 0 3 3l7-7a4 4 0 0 0-6-6l-7 7a6 6 0 0 0 8 8l5-5"],
        "shield" => &["M12 3 5 6v6c0 4 3 7 7 9 4-2 7-5 7-9V6z"],
        "spark" => &[
            "M12 4v4",
            "M12 16v4",
            "M4 12h4",
            "M16 12h4",
            "m7 7 2 2",
            "m15 15 2 2",
            "m17 7-2 2",
            "m7 17 2-2",
        ],
        "globe" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18",
            "M3 12h18",
            "M12 3c3 3 3 15 0 18",
            "M12 3c-3 3-3 15 0 18",
        ],
        "term" => &["M4 5h16v14H4z", "m8 10 2 2-2 2", "M12 14h4"],
        "note" => &["M6 4h9l3 3v13H6z", "M9 11h6", "M9 15h6"],
        "bell" => &["M6 16V11a6 6 0 0 1 12 0v5l1 2H5z", "M10 20h4"],
        "gear" => &[
            "M12 9a3 3 0 1 0 0 6 3 3 0 1 0 0-6",
            "M12 3v3",
            "M12 18v3",
            "M3 12h3",
            "M18 12h3",
            "m5.6 5.6 2.1 2.1",
            "m16.3 16.3 2.1 2.1",
            "m18.4 5.6-2.1 2.1",
            "m7.7 16.3-2.1 2.1",
        ],
        "tasks" => &["M5 5h14v14H5z", "m8.5 12 2.5 2.5 4.5-5"],
        "runs" => &["M8 5v14l11-7z"],
        "home" => &["M4 11 12 4l8 7", "M6 10v10h12V10"],
        "chart" => &["M5 19V9", "M12 19V5", "M19 19v-7"],
        "file" => &["M7 3h7l4 4v14H7z", "M14 3v4h4"],
        "back" => &["m14 6-6 6 6 6"],
        "stop" => &["M7 7h10v10H7z"],
        "undo" => &["M9 7 4 12l5 5", "M4 12h10a5 5 0 0 1 0 10h-2"],
        "panel" => &["M4 5h16v14H4z", "M14 5v14"],
        "check" => &["m5 12.5 4.5 4.5L19 7.5"],
        "sort" => &["M7 4v16", "m4 7 3-3 3 3", "M17 20V4", "m14 17 3 3 3-3"],
        "bold" => &[
            "M7 5h6a3.5 3.5 0 0 1 0 7H7z",
            "M7 12h7a3.5 3.5 0 0 1 0 7H7z",
        ],
        "row" => &["M4 6h16", "M4 12h16", "M4 18h16"],
        "inbox" => &["M4 13h4l2 3h4l2-3h4", "M5 5h14l1 8v6H4v-6z"],
        "bot" => &[
            "M6 9h12v10H6z",
            "M12 5v4",
            "M9.5 13h.01",
            "M14.5 13h.01",
            "M10 16h4",
        ],
        "bolt" => &["M13 3 5 14h6l-1 7 8-11h-6z"],
        "clock" => &["M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18", "M12 7v5l3 2"],
        "pulse" => &["M3 12h4l2-6 4 12 2-6h6"],
        "coin" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18",
            "M15 9.5c-.5-1-1.6-1.5-3-1.5-1.7 0-3 .8-3 2s1.3 1.7 3 2 3 .8 3 2-1.3 2-3 2c-1.4 0-2.5-.5-3-1.5",
            "M12 6v2",
            "M12 16v2",
        ],
        "tree" => &[
            "M9 4h6v4H9z",
            "M4 16h6v4H4z",
            "M14 16h6v4h-6z",
            "M12 8v4",
            "M7 16v-4h10v4",
        ],
        "pause" => &["M8 5v14", "M16 5v14"],
        "zoomin" => &[
            "M10.5 4a6.5 6.5 0 1 0 0 13 6.5 6.5 0 1 0 0-13",
            "m15.5 15.5 4.5 4.5",
            "M10.5 8v5",
            "M8 10.5h5",
        ],
        "zoomout" => &[
            "M10.5 4a6.5 6.5 0 1 0 0 13 6.5 6.5 0 1 0 0-13",
            "m15.5 15.5 4.5 4.5",
            "M8 10.5h5",
        ],
        "fit" => &["M4 9V4h5", "M15 4h5v5", "M20 15v5h-5", "M9 20H4v-5"],
        "next" => &["m9 6 6 6-6 6"],
        "copy" => &["M8 8h11v11H8z", "M5 16V5h11"],
        _ => &["M12 12h.01"],
    };
    rsx! {
        svg { class: "oxscn-ico", "data-ic": "{tone}", width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            for (i, p) in d.iter().enumerate() { path { key: "{i}", d: "{p}" } }
        }
    }
}

/// Skeleton lines for a loading region (`n` bars of the given widths, cycled).
pub(crate) fn skel(n: usize) -> Element {
    const W: [u32; 5] = [92, 64, 78, 48, 70];
    rsx! {
        div { class: "oxscn-skel", "aria-hidden": "true",
            for i in 0..n {
                { let s = format!("width:{}%", W[i % 5]); rsx! { span { key: "{i}", class: "oxscn-bar", style: "{s}" } } }
            }
        }
    }
}

/// An empty region: a title, one line and an optional action.
pub(crate) fn empty(title: &str, line: &str) -> Element {
    rsx! {
        div { class: "oxscn-empty",
            span { class: "oxscn-empty-ico", {ico("spark")} }
            p { class: "oxscn-empty-t", "{title}" }
            p { class: "oxscn-empty-l", "{line}" }
        }
    }
}

/// One menu entry: key, label, optional brand mark.
pub(crate) type MenuItem = (&'static str, &'static str, &'static str);

/// A button with a small menu under (or over) it. Escape or picking closes it; the picked key is
/// sent through `on_pick` and shown checked when `checked` matches.
#[component]
pub fn MenuButton(
    /// Button label (empty for icon-only, then `aria` names it).
    label: String,
    /// Accessible name.
    aria: String,
    /// Leading icon.
    #[props(default)]
    icon: String,
    /// Entries.
    items: Vec<MenuItem>,
    /// The checked entry.
    #[props(default)]
    checked: String,
    /// Open upward.
    #[props(default)]
    up: bool,
    /// Align the menu to the right edge.
    #[props(default)]
    right: bool,
    /// Extra class on the trigger.
    #[props(default)]
    class: String,
    /// Picked.
    #[props(default)]
    on_pick: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let is_open = open();
    let up_s = if up { "true" } else { "false" };
    let right_s = if right { "true" } else { "false" };
    rsx! {
        div { class: "oxscn-mb", "data-up": "{up_s}", "data-right": "{right_s}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && open() { e.stop_propagation(); open.set(false); },
            button { r#type: "button", class: "oxscn-btn {class}", "aria-haspopup": "menu", "aria-expanded": "{is_open}", "aria-label": "{aria}",
                onclick: move |_| open.toggle(),
                if !icon.is_empty() { {ico(&icon)} }
                if !label.is_empty() { span { "{label}" } }
            }
            if is_open {
                div { class: "oxscn-menu", role: "menu", "aria-label": "{aria}",
                    for (k, l, b) in items.iter().copied() {
                        button { key: "{k}", r#type: "button", class: "oxscn-mi", role: "menuitemradio",
                            "aria-checked": if checked == k { "true" } else { "false" },
                            onclick: move |_| { open.set(false); on_pick.call(k.to_string()); },
                            if !b.is_empty() { BrandIcon { name: b.to_string(), size: 15.0, coloured: true } }
                            span { class: "oxscn-mi-l", "{l}" }
                            if checked == k { span { class: "oxscn-mi-c", {check_glyph()} } }
                        }
                    }
                }
            }
        }
    }
}

/// The composer every chat screen uses: a field, Add, approval and model menus, voice and send.
/// Enter sends (Shift+Enter breaks a line); Send stays disabled until there is text.
#[component]
pub fn ScreenComposer(
    /// Text (controlled).
    text: Signal<String>,
    /// Placeholder.
    #[props(default = "Ask anything, or type / for tools".to_string())]
    placeholder: String,
    /// Show the model menu.
    #[props(default = true)]
    model: bool,
    /// Busy: Send turns into Stop.
    #[props(default)]
    busy: bool,
    /// Sent (the trimmed text).
    on_send: EventHandler<String>,
    /// Stop pressed while busy.
    #[props(default)]
    on_stop: EventHandler<()>,
) -> Element {
    let mut text = text;
    let mut ask = use_signal(|| "big".to_string());
    let mut mdl = use_signal(|| "auto".to_string());
    let mut rec = use_signal(|| false);
    let mut files = use_signal(Vec::<String>::new);
    let t = text();
    let ok = can_send(&t);
    let mut send = move || {
        let v = text.peek().trim().to_string();
        if can_send(&v) {
            on_send.call(v);
            text.set(String::new());
            files.set(vec![]);
        }
    };
    let ask_l = match ask().as_str() {
        "always" => "Ask first",
        "never" => "On its own",
        _ => "Ask for big steps",
    };
    let mdl_l = match mdl().as_str() {
        "fast" => "Fast",
        "deep" => "Deep",
        _ => "Auto",
    };
    let recording = rec();
    rsx! {
        div { class: "oxscn-cmp", "data-busy": "{busy}",
            if !files().is_empty() {
                div { class: "oxscn-cmp-files",
                    for (i, f) in files().iter().enumerate() {
                        span { key: "{i}", class: "oxscn-chip", {file_ico(f)} "{f}"
                            button { r#type: "button", class: "oxscn-chip-x", "aria-label": "Remove {f}",
                                onclick: move |_| { files.write().remove(i); }, {ico("close")} }
                        }
                    }
                }
            }
            textarea { class: "oxscn-cmp-in", rows: "2", placeholder: "{placeholder}", "aria-label": "Message", value: "{t}",
                oninput: move |e| text.set(e.value()),
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send(); },
            }
            div { class: "oxscn-cmp-row",
                MenuButton { label: "", aria: "Add", icon: "plus", up: true, class: "oxscn-ib",
                    items: vec![("upload", "Upload a file", ""), ("drive", "From Google Drive", "googledrive"), ("notion", "From Notion", "notion"), ("shot", "Take a screenshot", "")],
                    on_pick: move |k: String| { let n = match k.as_str() { "drive" => "budget-2026.sheet", "notion" => "Launch notes", "shot" => "screenshot.png", _ => "brief.pdf" }; files.write().push(n.to_string()); } }
                MenuButton { label: ask_l, aria: "When to ask before acting", icon: "shield", up: true, checked: ask(),
                    items: vec![("always", "Ask first", ""), ("big", "Ask for big steps", ""), ("never", "On its own", "")],
                    on_pick: move |k| ask.set(k) }
                if model {
                    MenuButton { label: mdl_l, aria: "Model", icon: "spark", up: true, checked: mdl(),
                        items: vec![("auto", "Auto", ""), ("fast", "Fast", ""), ("deep", "Deep", "")],
                        on_pick: move |k| mdl.set(k) }
                }
                span { class: "oxscn-grow" }
                button { r#type: "button", class: "oxscn-ib", "aria-label": if recording { "Stop dictation" } else { "Dictate" }, "aria-pressed": "{recording}",
                    "data-on": "{recording}", onclick: move |_| rec.toggle(), {ico("mic")} }
                if busy {
                    button { r#type: "button", class: "oxscn-send", "aria-label": "Stop", onclick: move |_| on_stop.call(()), {ico("stop")} }
                } else {
                    button { r#type: "button", class: "oxscn-send", "aria-label": "Send", disabled: !ok, onclick: move |_| send(), {ico("send")} }
                }
            }
        }
    }
}

/// Sidebar flavour.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Side {
    Chat,
    Runs,
    Agent,
}

/// The agent screens' sidebar: primary pages, then the workspace pages (key, label, icon, badge).
pub(crate) const AGENT_NAV: [(&str, &str, &str, u32); 7] = [
    ("thread", "Thread", "chat", 0),
    ("inbox", "Inbox", "inbox", 5),
    ("tasks", "Tasks", "tasks", 0),
    ("agents", "Agents", "bot", 0),
    ("runs", "Runs", "runs", 0),
    ("memory", "Memory", "note", 0),
    ("skills", "Skills", "bolt", 0),
];

/// The agent sidebar's Workspace group.
pub(crate) const AGENT_WS: [(&str, &str, &str); 5] = [
    ("schedules", "Schedules", "clock"),
    ("activity", "Activity", "pulse"),
    ("usage", "Usage", "coin"),
    ("permissions", "Permissions", "shield"),
    ("settings", "Settings", "gear"),
];

#[allow(clippy::too_many_arguments)]
fn sidebar(
    side: Side,
    state: ScreenState,
    mut thr: Signal<Vec<Thread>>,
    mut active: Signal<u32>,
    mut query: Signal<String>,
    mut rail: Signal<bool>,
    mut peek: Signal<bool>,
    mut ws_open: Signal<bool>,
    mut acct_open: Signal<bool>,
    mut notice: Signal<bool>,
    mut setup_open: Signal<bool>,
    mut steps: Signal<Vec<bool>>,
    mut open_proj: Signal<String>,
    on_nav: EventHandler<String>,
    nav_on: String,
) -> Element {
    let q = query().to_lowercase();
    let hit = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);
    let list = thr();
    let (pinned, rest) = split_pinned(&list);
    let pinned: Vec<Thread> = pinned.into_iter().filter(|t| hit(&t.title)).collect();
    let rest: Vec<Thread> = rest.into_iter().filter(|t| hit(&t.title)).collect();
    let projs: Vec<_> = projects()
        .into_iter()
        .filter(|p| hit(p.label) || p.chats.iter().any(|c| hit(c)))
        .collect();
    let cur = active();
    let st = steps();
    let done = st.iter().filter(|x| **x).count();
    let pct = pct_done(done, st.len());
    let pct_s = format!("{pct}% done");
    let bar = format!("width:{pct}%");
    let is_rail = rail();
    let nav: Vec<(&str, &str, &str, u32)> = match side {
        Side::Chat => vec![
            ("new", "New chat", "plus", 0),
            ("files", "Files", "folder", 0),
            ("apps", "Apps", "apps", 0),
        ],
        Side::Runs => vec![
            ("home", "Home", "home", 0),
            ("chat", "Chat", "chat", 0),
            ("runs", "Runs", "runs", 3),
            ("stats", "Numbers", "chart", 0),
            ("made", "Made for you", "file", 0),
        ],
        Side::Agent => AGENT_NAV.to_vec(),
    };
    let loading = state == ScreenState::Loading;
    let blank = state == ScreenState::Empty;
    let ws = ws_open();
    let ac = acct_open();
    let peek_s = if peek() { "true" } else { "false" };
    let row = move |t: Thread| {
        let id = t.id;
        let on = id == cur;
        let pin_l = if t.pinned {
            format!("Unpin {}", t.title)
        } else {
            format!("Pin {}", t.title)
        };
        let age = ago(t.mins);
        rsx! {
            div { key: "{id}", class: "oxscn-row", "data-on": "{on}",
                button { r#type: "button", class: "oxscn-row-b", "aria-current": if on { "page" } else { "false" },
                    onclick: move |_| { active.set(id); open_thread(&mut thr.write(), id); peek.set(false); },
                    if t.unread { span { class: "oxscn-dot", "aria-label": "Unread" } }
                    span { class: "oxscn-row-t", "{t.title}" }
                    span { class: "oxscn-row-a", "{age}" }
                }
                button { r#type: "button", class: "oxscn-row-pin", "aria-label": "{pin_l}", "aria-pressed": "{t.pinned}",
                    onclick: move |_| { toggle_pin(&mut thr.write(), id); }, {ico("pin")} }
            }
        }
    };
    rsx! {
        nav { class: "oxscn-side", "aria-label": "Sidebar", "data-rail": "{is_rail}", "data-peek": "{peek_s}",
            div { class: "oxscn-ws",
                button { r#type: "button", class: "oxscn-ws-b", "aria-haspopup": "menu", "aria-expanded": "{ws}", onclick: move |_| ws_open.toggle(),
                    span { class: "oxscn-mono", "K" } span { class: "oxscn-ws-n", "{WORKSPACE}" } {chevron(ws)} }
                button { r#type: "button", class: "oxscn-ib oxscn-fold", "aria-label": if is_rail { "Expand sidebar" } else { "Collapse sidebar" },
                    onclick: move |_| { rail.toggle(); peek.set(false); }, {ico("side")} }
                if ws {
                    div { class: "oxscn-menu oxscn-ws-menu", role: "menu", "aria-label": "Workspaces",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ws_open.set(false); },
                        button { r#type: "button", class: "oxscn-mi", role: "menuitemradio", "aria-checked": "true", onclick: move |_| ws_open.set(false),
                            span { class: "oxscn-mono", "K" } span { class: "oxscn-mi-l", "{WORKSPACE}" } span { class: "oxscn-mi-c", {check_glyph()} } }
                        button { r#type: "button", class: "oxscn-mi", role: "menuitemradio", "aria-checked": "false", onclick: move |_| ws_open.set(false),
                            span { class: "oxscn-mono", "Q" } span { class: "oxscn-mi-l", "Quiet Hours Lab" } }
                        button { r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| ws_open.set(false), {ico("plus")} span { class: "oxscn-mi-l", "New workspace" } }
                    }
                }
            }
            label { class: "oxscn-search",
                {ico("search")}
                input { r#type: "search", placeholder: "Search", "aria-label": "Search chats and projects", value: "{query}",
                    oninput: move |e| query.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { query.set(String::new()); } }
                kbd { class: "oxscn-kbd", "⌘K" }
            }
            div { class: "oxscn-nav",
                for (k, l, i, n) in nav {
                    button { key: "{k}", r#type: "button", class: "oxscn-nav-b", title: "{l}", "data-on": if (side == Side::Runs && k == "runs") || k == nav_on { "true" } else { "false" },
                        "aria-current": if k == nav_on { "page" } else { "false" },
                        onclick: move |_| { on_nav.call(k.to_string()); peek.set(false); },
                        {ico(i)} span { class: "oxscn-lbl", "{l}" }
                        if n > 0 { span { class: "oxscn-badge", "{n}" } }
                    }
                }
            }
            div { class: "oxscn-lists",
                if side == Side::Agent {
                    p { class: "oxscn-gh", "Workspace" }
                    div { class: "oxscn-nav",
                        for (k, l, i) in AGENT_WS {
                            button { key: "{k}", r#type: "button", class: "oxscn-nav-b", title: "{l}", "data-on": if k == nav_on { "true" } else { "false" },
                                "aria-current": if k == nav_on { "page" } else { "false" },
                                onclick: move |_| { on_nav.call(k.to_string()); peek.set(false); },
                                {ico(i)} span { class: "oxscn-lbl", "{l}" }
                            }
                        }
                    }
                } else if loading {
                    p { class: "oxscn-gh", "Chats" }
                    {skel(6)}
                } else if blank {
                    p { class: "oxscn-gh", "Chats" }
                    p { class: "oxscn-hint", "Chats you start land here. Pin the ones you come back to." }
                } else if side == Side::Runs {
                    p { class: "oxscn-gh", "Agents" }
                    for (i, a) in ["Member update", "Revenue desk", "Inbox sweeper", "Flows"].iter().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxscn-row-b oxscn-agent", span { class: "oxscn-mono oxscn-mono-s", "{&a[..1]}" } span { class: "oxscn-row-t", "{a}" } }
                    }
                } else {
                    if !pinned.is_empty() {
                        p { class: "oxscn-gh", "Pinned" }
                        for t in pinned { {row(t)} }
                    }
                    p { class: "oxscn-gh", "Projects" }
                    for p in projs {
                        {
                            let k = p.key;
                            let o = open_proj() == k;
                            let n = p.chats.len();
                            rsx! {
                                div { key: "{k}", class: "oxscn-proj",
                                    button { r#type: "button", class: "oxscn-row-b", "aria-expanded": "{o}",
                                        onclick: move |_| { let cur = open_proj.peek().clone(); open_proj.set(if cur == k { String::new() } else { k.to_string() }); },
                                        {ico("folder")} span { class: "oxscn-row-t", "{p.label}" }
                                        span { class: "oxscn-row-a", "{n}" }
                                    }
                                    if o {
                                        div { class: "oxscn-sub",
                                            if p.chats.is_empty() { span { class: "oxscn-hint", "No chats yet" } }
                                            for c in p.chats.iter() { button { key: "{c}", r#type: "button", class: "oxscn-row-b oxscn-subrow", span { class: "oxscn-row-t", "{c}" } } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    p { class: "oxscn-gh", "Chats" }
                    for t in rest { {row(t)} }
                    if !q.is_empty() && thr().iter().all(|t| !hit(&t.title)) && projects().iter().all(|p| !hit(p.label)) {
                        p { class: "oxscn-hint", role: "status", "Nothing matches “{query}”." }
                    }
                }
            }
            div { class: "oxscn-foot",
                if notice() && !is_rail {
                    div { class: "oxscn-notice",
                        p { class: "oxscn-notice-t", "New: shared folders" }
                        p { class: "oxscn-notice-l", "Give a folder to the team; agents read it too." }
                        button { r#type: "button", class: "oxscn-link", "aria-label": "Dismiss the notice", onclick: move |_| notice.set(false), "Dismiss" }
                    }
                }
                if !is_rail {
                    div { class: "oxscn-setup",
                        button { r#type: "button", class: "oxscn-setup-b", "aria-expanded": "{setup_open}", onclick: move |_| setup_open.toggle(),
                            span { "Set up Kestrel" } span { class: "oxscn-row-a", "{pct_s}" } {chevron(setup_open())} }
                        div { class: "oxscn-prog", role: "progressbar", "aria-valuenow": "{pct}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "Setup", span { style: "{bar}" } }
                        if setup_open() {
                            ul { class: "oxscn-steps",
                                for (i, (l, _)) in setup_steps().into_iter().enumerate() {
                                    li { key: "{i}",
                                        label { class: "oxscn-step",
                                            input { r#type: "checkbox", checked: st.get(i).copied().unwrap_or(false), onchange: move |e| { steps.write()[i] = e.checked(); } }
                                            span { "{l}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxscn-acct",
                    button { r#type: "button", class: "oxscn-acct-b", "aria-haspopup": "menu", "aria-expanded": "{ac}", "aria-label": "Account, {PERSON}", onclick: move |_| acct_open.toggle(),
                        span { class: "oxscn-av", "{PERSON_INI}" } span { class: "oxscn-lbl", "{PERSON}" } }
                    if ac {
                        div { class: "oxscn-menu oxscn-acct-menu", role: "menu", "aria-label": "Account",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { acct_open.set(false); },
                            for (i, l) in ["Profile", "Settings", "Keyboard shortcuts", "Sign out"].iter().enumerate() {
                                button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| acct_open.set(false), span { class: "oxscn-mi-l", "{l}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The resize handle between two regions (drag or arrow keys).
#[allow(clippy::too_many_arguments)]
fn separator(
    label: &'static str,
    which: &'static str,
    val: u32,
    min: u32,
    max: u32,
    mut w: Signal<u32>,
    mut drag: Signal<Option<(&'static str, f64, u32)>>,
    sign: i32,
) -> Element {
    rsx! {
        div { class: "oxscn-sep", "data-for": "{which}", role: "separator", tabindex: "0", "aria-orientation": "vertical", "aria-label": "{label}",
            "aria-valuenow": "{val}", "aria-valuemin": "{min}", "aria-valuemax": "{max}",
            onpointerdown: move |e: PointerEvent| { e.prevent_default(); drag.set(Some((which, e.client_coordinates().x, w()))); },
            onkeydown: move |e: KeyboardEvent| {
                let step: i32 = match e.key() { Key::ArrowLeft => -16, Key::ArrowRight => 16, _ => 0 } * sign;
                if step != 0 { e.prevent_default(); let v = (w() as i32 + step).clamp(min as i32, max as i32); w.set(v as u32); }
            },
        }
    }
}

/// The shared app frame: sidebar, page and an optional work area, Inset or Flat.
///
/// The frame owns the sidebar's state; the screen passes the threads and the open thread so its
/// page can add and switch chats. Under 860 px of width the work area becomes a sheet opened from
/// the top bar; under 640 px the sidebar slides in from a menu button.
#[component]
pub fn AppFrame(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Sidebar flavour: `chat` or `runs`.
    #[props(default = "chat".to_string())]
    side: String,
    /// Threads in the sidebar (owned by the screen).
    threads: Signal<Vec<Thread>>,
    /// The open thread id (0: none).
    active: Signal<u32>,
    /// Breadcrumb / title in the top bar.
    #[props(default)]
    title: String,
    /// Actions at the right of the top bar.
    #[props(default)]
    actions: Option<Element>,
    /// The work area (none: no right region).
    #[props(default)]
    work: Option<Element>,
    /// The work area's name ("Workspace", "Brief"…).
    #[props(default = "Workspace".to_string())]
    work_label: String,
    /// Start with the work area closed.
    #[props(default)]
    work_closed: bool,
    /// Sidebar nav pressed (`new`, `files`, `apps`…).
    #[props(default)]
    on_nav: EventHandler<String>,
    /// A sidebar of the screen's own in place of the chats sidebar (the Settings shell). Inside it,
    /// [`SidePeek`] from context closes the narrow slide-in sheet.
    #[props(default)]
    side_el: Option<Element>,
    /// The sidebar entry for this page (`inbox`, `tasks`…), shown current.
    #[props(default)]
    nav_on: String,
    /// The page.
    children: Element,
) -> Element {
    let lay = ShellLayout::from_key(&layout);
    let st = ScreenState::from_key(&state);
    let query = use_signal(String::new);
    let rail = use_signal(|| false);
    let mut peek = use_signal(|| false);
    use_context_provider(|| SidePeek(peek));
    let ws_open = use_signal(|| false);
    let acct_open = use_signal(|| false);
    let notice = use_signal(|| true);
    let setup_open = use_signal(|| false);
    let steps = use_signal(|| {
        setup_steps()
            .into_iter()
            .map(|(_, d)| d)
            .collect::<Vec<_>>()
    });
    let open_proj = use_signal(|| "autumn".to_string());
    let sw = use_signal(|| 248u32);
    let ww = use_signal(|| 340u32);
    let mut drag = use_signal(|| None::<(&'static str, f64, u32)>);
    let mut wopen = use_signal(move || !work_closed);
    let mut wpeek = use_signal(|| false);
    let has_work = work.is_some();
    let side_k = match side.as_str() {
        "runs" => Side::Runs,
        "agent" => Side::Agent,
        _ => Side::Chat,
    };
    let style = format!(
        "--oxscn-h:{height}px;--oxscn-sw:{}px;--oxscn-ww:{}px",
        sw(),
        ww()
    );
    let lay_k = lay.key();
    let st_k = st.key();
    let dragging = drag().is_some();
    let is_rail = rail();
    let wo = wopen() && has_work;
    let wp = wpeek() && has_work;
    let scrim = peek() || wp;
    let wl = work_label.clone();
    rsx! {
        div { class: "oxscn", "data-layout": "{lay_k}", "data-state": "{st_k}", "data-drag": "{dragging}", "data-rail": "{is_rail}", style: "{style}",
            onpointermove: move |e: PointerEvent| {
                if let Some((which, x0, w0)) = drag() {
                    let dx = (e.client_coordinates().x - x0) / stage_scale();
                    if which == "side" { let v = (w0 as f64 + dx).clamp(200.0, 340.0); let mut s = sw; s.set(v as u32); }
                    else { let v = (w0 as f64 - dx).clamp(260.0, 480.0); let mut s = ww; s.set(v as u32); }
                }
            },
            onpointerup: move |_| if drag.peek().is_some() { drag.set(None) },
            onpointerleave: move |_| if drag.peek().is_some() { drag.set(None) },
            if let Some(se) = side_el {
                nav { class: "oxscn-side oxsst-side", "aria-label": "Settings", "data-rail": "false", "data-peek": "{peek}", {se} }
            } else {
                {sidebar(side_k, st, threads, active, query, rail, peek, ws_open, acct_open, notice, setup_open, steps, open_proj, on_nav, nav_on)}
            }
            if !is_rail { {separator("Resize sidebar", "side", sw(), 200, 340, sw, drag, 1)} }
            div { class: "oxscn-page",
                main { class: "oxscn-main",
                    header { class: "oxscn-top",
                        button { r#type: "button", class: "oxscn-ib oxscn-menu-b", "aria-label": "Open the sidebar", "aria-expanded": "{peek}", onclick: move |_| peek.toggle(), {ico("side")} }
                        if !title.is_empty() { h2 { class: "oxscn-title", "{title}" } }
                        span { class: "oxscn-grow" }
                        {actions}
                        if has_work {
                            button { r#type: "button", class: "oxscn-ib oxscn-wtog-w", "aria-label": if wo { format!("Hide {wl}") } else { format!("Show {wl}") }, "aria-pressed": "{wo}",
                                onclick: move |_| wopen.toggle(), {ico("panel")} }
                            button { r#type: "button", class: "oxscn-btn oxscn-wtog-n", "aria-expanded": "{wp}", onclick: move |_| wpeek.toggle(), {ico("panel")} span { "{work_label}" } }
                        }
                    }
                    div { class: "oxscn-body", {children} }
                }
                if wo { {separator("Resize the work area", "work", ww(), 260, 480, ww, drag, -1)} }
                if has_work {
                    aside { class: "oxscn-work", "aria-label": "{work_label}", "data-open": "{wo}", "data-peek": "{wp}",
                        button { r#type: "button", class: "oxscn-ib oxscn-work-x", "aria-label": "Close {work_label}", onclick: move |_| { wpeek.set(false); wopen.set(false); }, {ico("close")} }
                        {work}
                    }
                }
            }
            if scrim { div { class: "oxscn-scrim", onclick: move |_| { peek.set(false); wpeek.set(false); } } }
        }
    }
}

/// The frame's narrow-sidebar flag, in context for a custom sidebar (`side_el`) to close it.
#[derive(Clone, Copy)]
pub(crate) struct SidePeek(pub Signal<bool>);

/// Thread signal seeded for a state: sample threads, or none when empty or loading.
pub(crate) fn use_threads(st: ScreenState) -> Signal<Vec<Thread>> {
    use_signal(move || {
        if st == ScreenState::Sample {
            threads()
        } else {
            vec![]
        }
    })
}

/// A message in a screen's conversation.
#[derive(Clone, PartialEq)]
pub(crate) struct Msg {
    pub me: bool,
    pub text: String,
    pub at: u64,
}

/// The reply a screen gives a message (canned, our own words).
pub(crate) fn reply_for(text: &str) -> String {
    let t = chat_title(text);
    format!(
        "On it. I'll start with “{t}”, show you the plan first and ask before anything goes out."
    )
}

/// Adds a new thread for a first message and opens it.
pub(crate) fn start_thread(mut thr: Signal<Vec<Thread>>, mut active: Signal<u32>, text: &str) {
    let id = thr.peek().iter().map(|t| t.id).max().unwrap_or(0) + 1;
    thr.write().insert(
        0,
        Thread {
            id,
            title: chat_title(text),
            pinned: false,
            unread: false,
            mins: 0,
        },
    );
    active.set(id);
}

/// Count of open suggestions as words.
pub(crate) fn open_words(r: &[Review]) -> String {
    match oxyn_core::parity::screensfid::open_count(r) {
        0 => "All reviewed".to_string(),
        1 => "1 to review".to_string(),
        n => format!("{n} to review"),
    }
}

/// Milliseconds between two clock readings.
pub(crate) fn since(now: ClockMs, then: u64) -> u64 {
    now.0.saturating_sub(then)
}

// ───────────────────────────────────────────────────────────── App frame demos ──

fn page_placeholder(st: ScreenState) -> Element {
    rsx! {
        div { class: "oxscn-ph",
            match st {
                ScreenState::Loading => rsx! { {skel(7)} },
                ScreenState::Empty => rsx! { {empty("Your page goes here", "Everything a screen shows sits in this pane; the sidebar and work area stay put.")} },
                ScreenState::Sample => rsx! {
                    h1 { class: "oxscn-h1", "Your page goes here" }
                    p { class: "oxscn-lead", "The frame keeps the sidebar, this page and the work area in step: drag the hairlines to resize, fold the sidebar to a rail, close the work area." }
                    div { class: "oxscn-ph-grid",
                        for i in 0..6 { div { key: "{i}", class: "oxscn-ph-card" } }
                    }
                },
            }
        }
    }
}

/// The work area of the App shell: browser-style tabs, an address field, tools and sources.
#[component]
fn Workspace(state: String) -> Element {
    let st = ScreenState::from_key(&state);
    let mut tabs = use_signal(|| vec!["New tab".to_string()]);
    let mut cur = use_signal(|| 0usize);
    let mut addr = use_signal(String::new);
    let mut said = use_signal(String::new);
    let list = tabs();
    let c = cur().min(list.len().saturating_sub(1));
    let mut open_tab = move |t: String| {
        tabs.write().push(t);
        let n = tabs.peek().len();
        cur.set(n - 1);
    };
    rsx! {
        div { class: "oxscn-wk",
            div { class: "oxscn-tabs", role: "tablist", "aria-label": "Open tabs",
                for (i, t) in list.iter().enumerate() {
                    div { key: "{i}-{t}", class: "oxscn-tab", "data-on": "{i == c}",
                        button { r#type: "button", role: "tab", class: "oxscn-tab-b", "aria-selected": "{i == c}", onclick: move |_| cur.set(i), "{t}" }
                        button { r#type: "button", class: "oxscn-tab-x", "aria-label": "Close {t}",
                            onclick: move |_| { let mut v = tabs.write(); if v.len() > 1 { v.remove(i); } else { v[0] = "New tab".to_string(); } drop(v); let n = tabs.peek().len(); if cur() >= n { cur.set(n - 1); } },
                            {ico("close")} }
                    }
                }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "New tab", onclick: move |_| open_tab("New tab".to_string()), {ico("plus")} }
            }
            label { class: "oxscn-search oxscn-addr",
                {ico("globe")}
                input { r#type: "text", placeholder: "Search or type an address", "aria-label": "Address", value: "{addr}",
                    oninput: move |e| addr.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { let v = addr.peek().trim().to_string(); if !v.is_empty() { tabs.write()[c] = v; addr.set(String::new()); } } }
            }
            if st == ScreenState::Loading {
                {skel(6)}
            } else {
                p { class: "oxscn-gh", "Tools" }
                for (k, l, i, kb) in [("term", "Terminal", "term", "⌃`"), ("web", "Browser", "globe", "⌘⇧B"), ("notes", "Notes", "note", "⌘⇧N")] {
                    div { key: "{k}", class: "oxscn-tool",
                        button { r#type: "button", class: "oxscn-row-b", onclick: move |_| open_tab(l.to_string()), {ico(i)} span { class: "oxscn-row-t", "{l}" } kbd { class: "oxscn-kbd", "{kb}" } }
                        MenuButton { label: "", aria: "More for {l}", icon: "more", right: true, class: "oxscn-ib",
                            items: vec![("pin", "Keep in the sidebar", ""), ("new", "Open in a new tab", ""), ("hide", "Hide", "")],
                            on_pick: move |k: String| said.set(format!("{l}: {k}")) }
                    }
                }
                p { class: "oxscn-gh", "Suggested" }
                if st == ScreenState::Empty {
                    p { class: "oxscn-hint", "Pages you open often show up here." }
                } else {
                    for (i, (d, w)) in [("papers.example.org", "Two papers on dye-free linen"), ("wiki.kestrel.example", "Launch checklist"), ("tracker.example.com", "Open tasks for the launch"), ("sheets.example.net", "Budget by channel")].into_iter().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxscn-src", onclick: move |_| open_tab(d.to_string()),
                            span { class: "oxscn-fav", "{&d[..1]}" } span { class: "oxscn-src-t", span { class: "oxscn-row-t", "{w}" } span { class: "oxscn-row-a", "{d}" } } }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

/// The stage's scale (canvas px → screen px), in context for frames that measure the pointer
/// (resize handles divide by it). 0 until the stage has been measured.
#[derive(Clone, Copy)]
pub struct StageScale(pub Signal<f64>);

/// The current stage scale, 1.0 outside a stage or before it is measured.
pub(crate) fn stage_scale() -> f64 {
    try_consume_context::<StageScale>()
        .map(|s| s.0.peek().to_owned())
        .filter(|v| *v > 0.0)
        .unwrap_or(1.0)
}

/// The screen stage (rule R1): every screen is authored on one fixed 1440 × 900 (16:10) canvas,
/// and the stage scales it to fit its column with the aspect ratio kept, so a screen never turns
/// taller than wide. Under the canvas, a 52 px state bar (Data · Loading · Empty, rule R2)
/// switches the data state; it shows in the full view (`data-screen-full` on an ancestor, or
/// `bar: true`) and stays out of the gallery thumbnail. `screen` renders the screen for a state.
#[component]
pub fn ScreenStage(
    /// `sample`, `loading` or `empty`: the state the screen opens in.
    #[props(default = "sample".to_string())]
    state: String,
    /// Always show the state bar (it also shows under `data-screen-full`).
    #[props(default)]
    bar: bool,
    /// Renders the screen for a state key.
    screen: Callback<String, Element>,
    /// The screen's name, for the canvas region label.
    #[props(default = "Screen".to_string())]
    label: String,
) -> Element {
    let mut st = use_signal(|| ScreenState::from_key(&state).key().to_string());
    let mut scale = use_signal(|| 0.0f64);
    use_context_provider(|| StageScale(scale));
    let cur = st();
    let canvas = format!("width:{STAGE_W}px;height:{STAGE_H}px");
    rsx! {
        div { class: "oxstage", "data-bar": "{bar}",
            div { class: "oxstage-box", role: "region", "aria-label": "{label}",
                onresize: move |e: Event<ResizeData>| {
                    if let Ok(sz) = e.get_border_box_size() { scale.set(sz.width / STAGE_W as f64); }
                },
                div { class: "oxstage-canvas", style: "{canvas}",
                    for k in [cur.clone()] { div { key: "{k}", class: "oxstage-k", {screen.call(k.clone())} } }
                }
            }
            div { class: "oxstage-bar", role: "toolbar", "aria-label": "Data state",
                span { class: "oxstage-bar-l", "State" }
                div { class: "oxstage-seg", role: "radiogroup", "aria-label": "Data",
                    for (k, l) in STAGE_STATES {
                        button { key: "{k}", r#type: "button", role: "radio", class: "oxstage-opt", "aria-checked": "{cur == k}",
                            onclick: move |_| st.set(k.to_string()), "{l}" }
                    }
                }
                span { class: "oxstage-bar-r", "{STAGE_W} × {STAGE_H}" }
            }
        }
    }
}

/// Screen – App shell: the shared frame with a placeholder page and a workspace on the right
/// (tabs, an address field, tools with shortcuts and their menus, suggested sources).
#[component]
pub fn AppShellScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let s2 = state.clone();
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Home", work: rsx! { Workspace { state: s2 } },
            {page_placeholder(st)}
        }
    }
}

// ───────────────────────────────────────────────────────────── Three-pane frame ──

/// The three-pane frame: an icon rail, a chats column (project switcher, search, new chat,
/// Pinned and Chats), the page and a work area. The column folds away from the rail's Chats icon.
#[component]
pub fn RailFrame(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Threads.
    threads: Signal<Vec<Thread>>,
    /// Open thread.
    active: Signal<u32>,
    /// Work area.
    #[props(default)]
    work: Option<Element>,
    /// Work area name.
    #[props(default = "Artifacts".to_string())]
    work_label: String,
    /// New chat pressed.
    #[props(default)]
    on_new: EventHandler<()>,
    /// The page.
    children: Element,
) -> Element {
    let lay = ShellLayout::from_key(&layout);
    let st = ScreenState::from_key(&state);
    let mut thr = threads;
    let mut active = active;
    let mut col = use_signal(|| true);
    let mut cpeek = use_signal(|| false);
    let mut sect = use_signal(|| "chats".to_string());
    let mut proj = use_signal(|| "all".to_string());
    let mut q = use_signal(String::new);
    let mut wopen = use_signal(|| true);
    let mut wpeek = use_signal(|| false);
    let has_work = work.is_some();
    let wo = wopen() && has_work;
    let wp = wpeek() && has_work;
    let ql = q().to_lowercase();
    let list = thr();
    let (pinned, rest) = split_pinned(&list);
    let keep = |t: &Thread| ql.is_empty() || t.title.to_lowercase().contains(&ql);
    let pinned: Vec<Thread> = pinned.into_iter().filter(keep).collect();
    let rest: Vec<Thread> = rest.into_iter().filter(keep).collect();
    let cur = active();
    let co = col();
    let cp = cpeek();
    let proj_l = match proj().as_str() {
        "all" => "All projects".to_string(),
        k => projects()
            .into_iter()
            .find(|p| p.key == k)
            .map(|p| p.label.to_string())
            .unwrap_or_default(),
    };
    let mut items: Vec<MenuItem> = vec![("all", "All projects", "")];
    items.extend(projects().into_iter().map(|p| (p.key, p.label, "")));
    let lay_k = lay.key();
    let st_k = st.key();
    let wl = work_label.clone();
    let row = move |t: Thread| {
        let id = t.id;
        let on = id == cur;
        rsx! {
            div { key: "{id}", class: "oxscn-row", "data-on": "{on}",
                button { r#type: "button", class: "oxscn-row-b", "aria-current": if on { "page" } else { "false" },
                    onclick: move |_| { active.set(id); open_thread(&mut thr.write(), id); },
                    if t.unread { span { class: "oxscn-dot", "aria-label": "Unread" } }
                    span { class: "oxscn-row-t", "{t.title}" }
                }
                button { r#type: "button", class: "oxscn-row-pin", "aria-label": if t.pinned { "Unpin" } else { "Pin" }, "aria-pressed": "{t.pinned}",
                    onclick: move |_| { toggle_pin(&mut thr.write(), id); }, {ico("pin")} }
            }
        }
    };
    rsx! {
        div { class: "oxscn oxsrl", "data-layout": "{lay_k}", "data-state": "{st_k}", "data-col": "{co}", "data-cpeek": "{cp}", style: "--oxscn-h:{height}px",
            nav { class: "oxsrl-rail", "aria-label": "Sections",
                span { class: "oxscn-mono", title: "{WORKSPACE}", "K" }
                for (k, l, i) in [("chats", "Chats", "chat"), ("tasks", "Tasks", "tasks"), ("runs", "Runs", "runs"), ("files", "Files", "folder"), ("apps", "Apps", "apps")] {
                    button { key: "{k}", r#type: "button", class: "oxsrl-rb", "aria-label": "{l}", title: "{l}", "aria-current": if sect() == k { "page" } else { "false" },
                        onclick: move |_| { if k == "chats" && sect() == "chats" { col.toggle(); cpeek.toggle(); } else { col.set(true); cpeek.set(true); } sect.set(k.to_string()); }, {ico(i)} }
                }
                span { class: "oxscn-grow" }
                button { r#type: "button", class: "oxsrl-rb", "aria-label": "Alerts", title: "Alerts", {ico("bell")} }
                button { r#type: "button", class: "oxsrl-rb", "aria-label": "Settings", title: "Settings", {ico("gear")} }
                span { class: "oxscn-av", title: "{PERSON}", "{PERSON_INI}" }
            }
            div { class: "oxsrl-col",
                MenuButton { label: proj_l, aria: "Project", icon: "folder", items, checked: proj(), on_pick: move |k| proj.set(k), class: "oxsrl-proj" }
                label { class: "oxscn-search",
                    {ico("search")}
                    input { r#type: "search", placeholder: "Search", "aria-label": "Search chats", value: "{q}", oninput: move |e| q.set(e.value()) }
                }
                button { r#type: "button", class: "oxscn-nav-b", onclick: move |_| on_new.call(()), {ico("plus")} span { "New chat" } }
                div { class: "oxscn-lists",
                    if st == ScreenState::Loading { {skel(7)} }
                    else if list.is_empty() { p { class: "oxscn-hint", "No chats yet. Start one with New chat." } }
                    else {
                        if !pinned.is_empty() { p { class: "oxscn-gh", "Pinned" } for t in pinned { {row(t)} } }
                        p { class: "oxscn-gh", "Chats" }
                        for t in rest { {row(t)} }
                    }
                }
            }
            div { class: "oxscn-page",
                main { class: "oxscn-main",
                    header { class: "oxscn-top",
                        button { r#type: "button", class: "oxscn-ib oxsrl-colb", "aria-label": if co { "Hide chats" } else { "Show chats" }, "aria-pressed": "{co}", onclick: move |_| col.toggle(), {ico("side")} }
                        button { r#type: "button", class: "oxscn-ib oxscn-menu-b", "aria-label": "Open chats", "aria-expanded": "{cp}", onclick: move |_| cpeek.toggle(), {ico("side")} }
                        span { class: "oxscn-grow" }
                        if has_work {
                            button { r#type: "button", class: "oxscn-ib oxscn-wtog-w", "aria-label": if wo { format!("Hide {wl}") } else { format!("Show {wl}") }, "aria-pressed": "{wo}", onclick: move |_| wopen.toggle(), {ico("panel")} }
                            button { r#type: "button", class: "oxscn-btn oxscn-wtog-n", "aria-expanded": "{wp}", onclick: move |_| wpeek.toggle(), {ico("panel")} span { "{work_label}" } }
                        }
                    }
                    div { class: "oxscn-body", {children} }
                }
                if has_work {
                    aside { class: "oxscn-work oxsrl-work", "aria-label": "{work_label}", "data-open": "{wo}", "data-peek": "{wp}",
                        button { r#type: "button", class: "oxscn-ib oxscn-work-x", "aria-label": "Close {work_label}", onclick: move |_| { wpeek.set(false); wopen.set(false); }, {ico("close")} }
                        {work}
                    }
                }
            }
            if wp { div { class: "oxscn-scrim", onclick: move |_| wpeek.set(false) } }
            if cp && !wp { div { class: "oxscn-scrim oxsrl-cscrim", onclick: move |_| cpeek.set(false) } }
        }
    }
}

/// Screen – Three-pane shell: the rail frame with a placeholder page and the workspace.
#[component]
pub fn RailShellScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 2u32);
    let s2 = state.clone();
    rsx! {
        RailFrame { layout, state: state.clone(), height, threads: thr, active, work_label: "Workspace", work: rsx! { Workspace { state: s2 } },
            {page_placeholder(st)}
        }
    }
}

// ───────────────────────────────────────────────────────────── AI Chat screens ──

/// A conversation log with a pending reply (thinking until `reply_ms` after the last message).
fn convo(msgs: &[Msg], now: ClockMs, reply_ms: u64) -> Element {
    let pending = msgs
        .last()
        .map(|m| m.me && since(now, m.at) < reply_ms)
        .unwrap_or(false);
    rsx! {
        div { class: "oxscn-log", role: "log", "aria-live": "polite",
            for (i, m) in msgs.iter().enumerate() {
                if m.me {
                    div { key: "{i}", class: "oxscn-msg", "data-me": "true", p { "{m.text}" } }
                } else if since(now, m.at) < reply_ms {
                    div { key: "{i}", class: "oxscn-msg oxscn-hide" }
                } else {
                    div { key: "{i}", class: "oxscn-msg", "data-me": "false", span { class: "oxscn-mono oxscn-mono-s", "K" } p { "{m.text}" } }
                }
            }
            if pending { div { class: "oxscn-thinking", role: "status", span { class: "oxscn-pulse" } "Thinking…" } }
        }
    }
}

/// Sends a message: appends it and a reply that shows `reply_ms` later.
fn push_pair(mut msgs: Signal<Vec<Msg>>, now: ClockMs, text: String) {
    let r = reply_for(&text);
    msgs.write().push(Msg {
        me: true,
        text,
        at: now.0,
    });
    msgs.write().push(Msg {
        me: false,
        text: r,
        at: now.0,
    });
}

fn apps_menu() -> Vec<MenuItem> {
    vec![
        ("notion", "Notion", "notion"),
        ("linear", "Linear", "linear"),
        ("gmail", "Gmail", "gmail"),
        ("slack", "Slack", "slack"),
        ("github", "GitHub", "github"),
        ("drive", "Google Drive", "googledrive"),
    ]
}

/// Screen – AI Chat / Start: a greeting, the composer with Add, approval, model and voice, a
/// folder and apps to work with, and starter suggestions that fill the composer. Sending opens a
/// new chat in the sidebar and the conversation takes over the page.
#[component]
pub fn ChatStartScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Hour of day for the greeting (0–23).
    #[props(default = 19)]
    hour: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let mut active = use_signal(|| 0u32);
    let mut text = use_signal(String::new);
    let mut msgs = use_signal(Vec::<Msg>::new);
    let mut folder = use_signal(String::new);
    let mut apps = use_signal(move || {
        if st == ScreenState::Sample {
            vec!["notion".to_string(), "slack".to_string()]
        } else {
            vec![]
        }
    });
    let hi = format!(
        "{}, Rowan. What's next?",
        oxyn_core::parity::screensfid::greeting(hour)
    );
    let started = !msgs().is_empty();
    let folder_l = if folder().is_empty() {
        "Work in a folder".to_string()
    } else {
        projects()
            .into_iter()
            .find(|p| p.key == folder())
            .map(|p| p.label.to_string())
            .unwrap_or_default()
    };
    let mut fitems: Vec<MenuItem> = vec![];
    fitems.extend(projects().into_iter().map(|p| (p.key, p.label, "")));
    let conn = apps();
    let conn_n = conn.len();
    let apps_l = if conn_n == 0 {
        "Connect apps".to_string()
    } else {
        format!("{conn_n} apps on")
    };
    let send = move |t: String| {
        if msgs.peek().is_empty() {
            start_thread(thr, active, &t);
        }
        push_pair(msgs, now, t);
    };
    let title = if started {
        thr()
            .iter()
            .find(|t| t.id == active())
            .map(|t| t.title.clone())
            .unwrap_or_default()
    } else {
        String::new()
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title,
            on_nav: move |k: String| if k == "new" { msgs.set(vec![]); active.set(0); },
            if started {
                div { class: "oxcst-chat",
                    {convo(&msgs(), now, 1200)}
                    ScreenComposer { text, on_send: send }
                }
            } else {
                div { class: "oxcst",
                    h1 { class: "oxcst-hi", "{hi}" }
                    ScreenComposer { text, on_send: send }
                    div { class: "oxcst-ctx",
                        MenuButton { label: folder_l, aria: "Work in a folder", icon: "folder", items: fitems, checked: folder(), on_pick: move |k| folder.set(k) }
                        MenuButton { label: apps_l, aria: "Connect apps", icon: "apps", items: apps_menu(), checked: "",
                            on_pick: move |k: String| { let mut a = apps.write(); if let Some(i) = a.iter().position(|x| *x == k) { a.remove(i); } else { a.push(k); } } }
                        for k in conn.iter() {
                            { let b = if k == "drive" { "googledrive".to_string() } else { k.clone() }; rsx! { span { key: "{k}", class: "oxcst-app", title: "{k}", BrandIcon { name: b, size: 14.0, coloured: true } } } }
                        }
                    }
                    if st == ScreenState::Loading {
                        div { class: "oxcst-starters", "aria-busy": "true", for i in 0..7 { span { key: "{i}", class: "oxcst-st oxscn-shim" } } }
                    } else {
                        div { class: "oxcst-starters",
                            for (k, l, p) in oxyn_core::parity::screensfid::starters() {
                                button { key: "{k}", r#type: "button", class: "oxcst-st", onclick: move |_| text.set(p.to_string()), "{l}" }
                            }
                        }
                        if st == ScreenState::Empty { p { class: "oxscn-hint oxcst-note", "Your first chat will show up in the sidebar." } }
                    }
                }
            }
        }
    }
}

/// Screen – AI Chat / Overview: the composer with the numbers under it – activity per day (a heat
/// map you step through with the arrow keys), recent chats, credits used, files made for you and
/// this month against last.
#[component]
pub fn ChatOverviewScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::parity::screensfid::{delta, heat, heat_total, thousands, used_pct};
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let text = use_signal(String::new);
    let msgs = use_signal(Vec::<Msg>::new);
    let mut cell = use_signal(|| 139usize);
    let mut more_chats = use_signal(|| false);
    let mut more_files = use_signal(|| false);
    const WEEKS: usize = 20;
    let levels = if st == ScreenState::Sample {
        heat(WEEKS)
    } else {
        vec![0; WEEKS * 7]
    };
    let total = heat_total(&levels);
    let c = cell();
    const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let lv = levels.get(c).copied().unwrap_or(0);
    let cell_said = format!(
        "Week {} of {WEEKS}, {}: {} chats",
        c / 7 + 1,
        DAYS[c % 7],
        lv as u32 * 3
    );
    let used = if st == ScreenState::Sample { 1284 } else { 0 };
    let pct = used_pct(used, 2000);
    let used_s = thousands(used);
    let bar = format!("width:{pct}%");
    let recent: Vec<(String, String)> = thr()
        .iter()
        .map(|t| (t.title.clone(), ago(t.mins)))
        .collect();
    let nshow = if more_chats() {
        recent.len()
    } else {
        3.min(recent.len())
    };
    let more_n = recent.len().saturating_sub(3);
    let files = [
        ("member-letter.md", "Autumn launch", "2h"),
        ("fabric-compare.csv", "Field research", "1d"),
        ("logo-grid.md", "Brand kit", "3d"),
        ("hiring-plan.md", "Hiring loop", "5d"),
    ];
    let fshow = if more_files() { 4 } else { 3 };
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let send = move |t: String| push_pair(msgs, now, t);
    let started = !msgs().is_empty();
    let d1 = delta(12);
    let d2 = delta(-3);
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Overview",
            div { class: "oxcov",
                p { class: "oxcov-mark", {ico("spark")} "Kestrel" }
                ScreenComposer { text, on_send: send }
                if started { {convo(&msgs(), now, 1200)} }
                div { class: "oxcov-grid",
                    section { class: "oxcov-card oxcov-act", "aria-label": "Activity",
                        div { class: "oxcov-h", h3 { "Activity" } span { class: "oxscn-row-a", if loading { "…" } else { "{total} chats · 20 weeks" } } }
                        if loading { {skel(3)} } else {
                            div { class: "oxcov-heat", role: "grid", tabindex: "0", "aria-label": "Chats per day, arrow keys move",
                                onkeydown: move |e: KeyboardEvent| {
                                    let d: i64 = match e.key() { Key::ArrowUp => -1, Key::ArrowDown => 1, Key::ArrowLeft => -7, Key::ArrowRight => 7, _ => 0 };
                                    if d != 0 { e.prevent_default(); let n = (cell() as i64 + d).clamp(0, (WEEKS * 7 - 1) as i64); cell.set(n as usize); }
                                },
                                for (i, l) in levels.iter().enumerate() {
                                    span { key: "{i}", class: "oxcov-c", "data-l": "{l}", "data-on": "{i == c}", onclick: move |_| cell.set(i) }
                                }
                            }
                            p { class: "oxcov-said", role: "status", if blank { "Nothing yet – your first chat lights up today's square." } else { "{cell_said}" } }
                        }
                    }
                    section { class: "oxcov-card", "aria-label": "Recent chats",
                        div { class: "oxcov-h", h3 { "Recent chats" } }
                        if loading { {skel(3)} } else if recent.is_empty() { p { class: "oxscn-hint", "No chats yet." } } else {
                            for (i, (t, a)) in recent.iter().take(nshow).enumerate() {
                                div { key: "{i}", class: "oxcov-li", span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-row-a", "{a}" } }
                            }
                            if more_n > 0 {
                                button { r#type: "button", class: "oxscn-link", "aria-expanded": "{more_chats}", onclick: move |_| more_chats.toggle(),
                                    if more_chats() { "Show less" } else { "{more_n} more" } }
                            }
                        }
                    }
                    section { class: "oxcov-card", "aria-label": "Credits",
                        div { class: "oxcov-h", h3 { "Credits" } span { class: "oxscn-row-a", "Resets 1 Dec" } }
                        if loading { {skel(2)} } else {
                            p { class: "oxcov-big", "{used_s}" span { class: "oxscn-row-a", " of 2,000" } }
                            div { class: "oxscn-prog", role: "meter", "aria-valuenow": "{used}", "aria-valuemin": "0", "aria-valuemax": "2000", "aria-label": "Credits used", span { style: "{bar}" } }
                        }
                    }
                    section { class: "oxcov-card", "aria-label": "Made for you",
                        div { class: "oxcov-h", h3 { "Made for you" } }
                        if loading { {skel(3)} } else if blank { p { class: "oxscn-hint", "Docs and sheets the assistant makes land here." } } else {
                            for (i, (f, p, a)) in files.iter().take(fshow).enumerate() {
                                div { key: "{i}", class: "oxcov-li", title: "{p}", {file_ico(f)} span { class: "oxscn-row-t", "{f}" } span { class: "oxscn-row-a", "{a}" } }
                            }
                            button { r#type: "button", class: "oxscn-link", "aria-expanded": "{more_files}", onclick: move |_| more_files.toggle(), if more_files() { "Show less" } else { "1 more" } }
                        }
                    }
                    section { class: "oxcov-card", "aria-label": "This month",
                        div { class: "oxcov-h", h3 { "This month" } }
                        if loading { {skel(2)} } else {
                            dl { class: "oxcov-dl",
                                div { dt { "Chats" } dd { if blank { "0" } else { "36" } span { class: "oxcov-d", "data-up": "true", if !blank { "{d1}" } } } }
                                div { dt { "Files made" } dd { if blank { "0" } else { "14" } span { class: "oxcov-d", "data-up": "false", if !blank { "{d2}" } } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Screen – AI Chat / Agents: a chat with the orchestrator and the crew at work, and a brief
/// beside it – angle, audience, channels, palette, formats and details – that turns into ads.
#[component]
pub fn ChatAgentsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::parity::screensfid::{ad_estimate, angles, channels, crew};
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 2u32);
    let text = use_signal(String::new);
    let msgs = use_signal(Vec::<Msg>::new);
    let mut who = use_signal(String::new);
    let mut tab = use_signal(|| "brief".to_string());
    let mut angle = use_signal(|| "feel".to_string());
    let mut chans = use_signal(|| channels().into_iter().map(|(_, on)| on).collect::<Vec<_>>());
    let mut fmts = use_signal(|| vec![true, true, false]);
    let mut gen_at = use_signal(|| None::<u64>);
    let on_n = chans().iter().filter(|x| **x).count();
    let f_n = fmts().iter().filter(|x| **x).count();
    let (ads, mins) = ad_estimate(on_n, f_n);
    let foot = format!("{ads} ads · ~{mins} min");
    let made = gen_at().map(|t| (since(now, t) / 700) as usize);
    let gen_s = match made {
        Some(n) if n < ads => format!("Making {} of {ads}…", n + 1),
        Some(_) => format!("{ads} ads ready"),
        None => String::new(),
    };
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let send = move |t: String| push_pair(msgs, now, t);
    let w = who();
    let brief = rsx! {
        div { class: "oxcag-brief",
            div { class: "oxscn-seg", role: "tablist", "aria-label": "Brief sections",
                for (k, l) in [("brief", "Brief"), ("assets", "Assets"), ("activity", "Activity")] {
                    button { key: "{k}", r#type: "button", role: "tab", class: "oxscn-seg-b", "aria-selected": "{tab() == k}", onclick: move |_| tab.set(k.to_string()), "{l}" }
                }
            }
            if loading { {skel(9)} }
            else if blank { {empty("No brief yet", "Tell the orchestrator what you are launching; the brief fills in as it learns.")} }
            else if tab() == "brief" {
                div { class: "oxcag-prod",
                    span { class: "oxcag-thumb", "aria-hidden": "true" }
                    div { p { class: "oxcag-pn", "Field Overshirt" } p { class: "oxscn-row-a", "$118 · 640 sold" } }
                    span { class: "oxscn-tag", "data-tone": "ok", "Top seller" }
                }
                p { class: "oxscn-gh", "Angle" }
                div { class: "oxcag-angles", role: "radiogroup", "aria-label": "Selling angle",
                    for a in angles() {
                        button { key: "{a.key}", r#type: "button", role: "radio", class: "oxcag-angle", "aria-checked": "{angle() == a.key}", onclick: move |_| angle.set(a.key.to_string()),
                            span { class: "oxcag-sig", "data-n": "{a.signal}", span {} span {} span {} }
                            span { span { class: "oxcag-al", "{a.label}" } span { class: "oxscn-row-a", "{a.why}" } }
                        }
                    }
                }
                p { class: "oxscn-gh", "Audience" }
                div { class: "oxcag-tags", span { class: "oxscn-chip", "Commuters 25–40" } span { class: "oxscn-chip", "Weekend hikers" } }
                p { class: "oxscn-gh", "Channels" }
                for (i, (l, _)) in channels().into_iter().enumerate() {
                    { let on = chans().get(i).copied().unwrap_or(false); rsx! {
                        div { key: "{i}", class: "oxcag-sw",
                            span { "{l}" }
                            button { r#type: "button", role: "switch", class: "oxscn-switch", "aria-checked": "{on}", "aria-label": "{l}", onclick: move |_| { chans.write()[i] = !on; }, span {} }
                        }
                    } }
                }
                p { class: "oxscn-gh", "Palette" }
                div { class: "oxcag-pal",
                    for (i, (n, h)) in [("Oat", "#D9CDB8"), ("Moss", "#6E7D57"), ("Chalk", "#F1EFEA"), ("Night", "#22262B")].into_iter().enumerate() {
                        span { key: "{i}", class: "oxcag-swatch", title: "{n}", style: "background:{h}", "aria-label": "{n}" }
                    }
                }
                p { class: "oxscn-gh", "Formats" }
                div { class: "oxcag-tags",
                    for (i, l) in ["Square", "Story", "Wide"].into_iter().enumerate() {
                        { let on = fmts().get(i).copied().unwrap_or(false); rsx! {
                            button { key: "{i}", r#type: "button", class: "oxscn-chip oxscn-chip-b", "aria-pressed": "{on}", onclick: move |_| { fmts.write()[i] = !on; }, "{l}" }
                        } }
                    }
                }
                dl { class: "oxcag-dl", div { dt { "Launch" } dd { "3 Nov" } } div { dt { "Budget" } dd { "$4,000" } } div { dt { "Markets" } dd { "US, CA" } } }
            } else if tab() == "assets" {
                for (i, f) in ["overshirt-front.png", "overshirt-detail.png", "style-guide.pdf"].iter().enumerate() {
                    div { key: "{i}", class: "oxcov-li", {file_ico(f)} span { class: "oxscn-row-t", "{f}" } }
                }
            } else {
                ul { class: "oxcag-act",
                    li { "Brand scout read 42 past posts" } li { "Audience desk sized two groups" } li { "Studio started six frames" }
                }
            }
            div { class: "oxcag-foot",
                span { class: "oxscn-row-a", role: "status", if gen_s.is_empty() { "{foot}" } else { "{gen_s}" } }
                button { r#type: "button", class: "oxscn-primary", disabled: loading || blank || ads == 0, onclick: move |_| gen_at.set(Some(now.0)), "Make ads" }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Ads for the overshirt", work_label: "Brief", work: brief,
            actions: rsx! { MenuButton { label: "", aria: "Chat options", icon: "more", right: true, items: vec![("rename", "Rename", ""), ("share", "Share", ""), ("archive", "Archive", "")] } },
            div { class: "oxcag",
                if loading { {skel(8)} }
                else if blank { {empty("Start with a goal", "Say what you want made; the orchestrator brings in the agents it needs.")} }
                else {
                    div { class: "oxscn-log",
                        div { class: "oxscn-msg", "data-me": "true", p { "Make launch ads for the overshirt. Lead with how it feels." } }
                        div { class: "oxscn-msg", "data-me": "false", span { class: "oxscn-mono oxscn-mono-s", "K" }
                            p { "I split this across five agents. The brief on the right is filling in; numbers are provisional until the sales pull finishes." } }
                        p { class: "oxscn-row-a oxcag-took", "Worked for 3m 12s" }
                    }
                    div { class: "oxcag-crew", "aria-label": "Agents working",
                        p { class: "oxscn-gh", "Agents working" }
                        div { class: "oxcag-row",
                            for m in crew() {
                                button { key: "{m.key}", r#type: "button", class: "oxcag-ag", "data-done": "{m.done}", "aria-pressed": "{w == m.key}", title: "{m.name}",
                                    onclick: move |_| { let c = who.peek().clone(); who.set(if c == m.key { String::new() } else { m.key.to_string() }); },
                                    span { class: "oxscn-mono", "{&m.name[..1]}" } span { class: "oxcag-agn", "{m.name}" } }
                            }
                        }
                        if let Some(m) = crew().into_iter().find(|m| m.key == w) {
                            p { class: "oxcag-said", role: "status", "{m.name}: {m.doing} · " if m.done { "done" } else { "working" } }
                        }
                        div { class: "oxcag-orch", span { class: "oxscn-mono", "O" } span { b { "Orchestrator" } span { class: "oxscn-row-a", " · routes each request" } } }
                    }
                    {convo(&msgs(), now, 1200)}
                }
                ScreenComposer { text, on_send: send }
            }
        }
    }
}

/// The sheet editor in the Double sidebar screen's artifacts: toolbar, cell reference, contents
/// and a grid you click or arrow through; typing in Contents edits the cell, Enter moves down.
#[component]
fn SheetEditor() -> Element {
    use oxyn_core::parity::screensfid::{cell_ref, col_name, sheet, sort_sheet};
    let mut data = use_signal(sheet);
    let mut undo = use_signal(Vec::<Vec<Vec<String>>>::new);
    let mut sel = use_signal(|| (1usize, 1usize));
    let mut bold = use_signal(Vec::<(usize, usize)>::new);
    let d = data();
    let (r, c) = sel();
    let rows = d.len();
    let cols = d.first().map(|x| x.len()).unwrap_or(0);
    let val = d.get(r).and_then(|x| x.get(c)).cloned().unwrap_or_default();
    let refs = cell_ref(c, r);
    let can_undo = !undo().is_empty();
    let mut snap = move || {
        let s = data.peek().clone();
        undo.write().push(s);
    };
    let heads: Vec<String> = (0..cols).map(|i| col_name(i).to_string()).collect();
    rsx! {
        div { class: "oxcdr-sheet",
            div { class: "oxcdr-tb", role: "toolbar", "aria-label": "Sheet",
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Undo", disabled: !can_undo, onclick: move |_| { if let Some(s) = undo.write().pop() { data.set(s); } }, {ico("undo")} }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Bold", "aria-pressed": "{bold().contains(&(r, c))}",
                    onclick: move |_| { let k = sel(); let mut b = bold.write(); if let Some(i) = b.iter().position(|x| *x == k) { b.remove(i); } else { b.push(k); } }, {ico("bold")} }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Sort A to Z by this column", onclick: move |_| { snap(); sort_sheet(&mut data.write(), sel().1, true); }, {ico("sort")} }
                button { r#type: "button", class: "oxscn-btn", onclick: move |_| { snap(); sort_sheet(&mut data.write(), sel().1, false); }, "Z–A" }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Insert a row below", onclick: move |_| { snap(); let at = sel().0 + 1; let n = data.peek()[0].len(); data.write().insert(at, vec![String::new(); n]); sel.set((at, 0)); }, {ico("row")} }
            }
            div { class: "oxcdr-fx",
                input { class: "oxcdr-ref", "aria-label": "Cell", readonly: true, value: "{refs}" }
                input { class: "oxcdr-val", "aria-label": "Contents of {refs}", value: "{val}",
                    oninput: move |e| { let (r, c) = sel(); if let Some(x) = data.write().get_mut(r).and_then(|x| x.get_mut(c)) { *x = e.value(); } },
                    onfocus: move |_| snap(),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { let (r, c) = sel(); sel.set(((r + 1).min(rows - 1), c)); } }
            }
            div { class: "oxcdr-grid", role: "grid", tabindex: "0", "aria-label": "Fabric comparison",
                onkeydown: move |e: KeyboardEvent| {
                    let (r, c) = sel();
                    let n = match e.key() { Key::ArrowUp => (r.saturating_sub(1), c), Key::ArrowDown => ((r + 1).min(rows - 1), c), Key::ArrowLeft => (r, c.saturating_sub(1)), Key::ArrowRight => (r, (c + 1).min(cols - 1)), _ => (r, c) };
                    if n != (r, c) { e.prevent_default(); sel.set(n); }
                },
                div { class: "oxcdr-tr oxcdr-hd", span { class: "oxcdr-rh" } for (i, h) in heads.iter().enumerate() { span { key: "{i}", class: "oxcdr-ch", "{h}" } } }
                for (ri, row) in d.iter().enumerate() {
                    div { key: "{ri}", class: "oxcdr-tr", role: "row",
                        span { class: "oxcdr-rh", "{ri + 1}" }
                        for (ci, v) in row.iter().enumerate() {
                            span { key: "{ci}", class: "oxcdr-td", role: "gridcell", "aria-selected": "{(ri, ci) == (r, c)}", "data-b": "{bold().contains(&(ri, ci))}", "data-h": "{ri == 0}",
                                onclick: move |_| sel.set((ri, ci)), "{v}" }
                        }
                    }
                }
            }
        }
    }
}

/// Screen – AI Chat / Double sidebar: an icon rail and a chats column, a plain chat with web search
/// (each answer folds its tool calls into one line and links what it made), and the artifacts on
/// the right – a sheet you can edit and the sources list – as closable tabs.
#[component]
pub fn ChatRailScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let mut active = use_signal(|| 3u32);
    let text = use_signal(String::new);
    let mut msgs = use_signal(Vec::<Msg>::new);
    let mut tools_open = use_signal(|| false);
    let mut arts = use_signal(move || {
        if st == ScreenState::Sample {
            vec!["fabric-compare.csv", "sources.md"]
        } else {
            vec![]
        }
    });
    let mut art = use_signal(|| 0usize);
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let a = arts();
    let ai = art().min(a.len().saturating_sub(1));
    let cur = a.get(ai).copied().unwrap_or("");
    let mut open_art = move |n: &'static str| {
        let p = arts.peek().iter().position(|x| *x == n);
        match p {
            Some(i) => art.set(i),
            None => {
                arts.write().push(n);
                let l = arts.peek().len();
                art.set(l - 1);
            }
        }
    };
    let send = move |t: String| {
        if thr.peek().is_empty() {
            start_thread(thr, active, &t);
        }
        push_pair(msgs, now, t);
    };
    let title = thr()
        .iter()
        .find(|t| t.id == active())
        .map(|t| t.title.clone())
        .unwrap_or_else(|| "New chat".to_string());
    let to = tools_open();
    let work = rsx! {
        div { class: "oxcdr-arts",
            div { class: "oxscn-tabs", role: "tablist", "aria-label": "Artifacts",
                for (i, n) in a.iter().copied().enumerate() {
                    div { key: "{n}", class: "oxscn-tab", "data-on": "{i == ai}",
                        button { r#type: "button", role: "tab", class: "oxscn-tab-b", "aria-selected": "{i == ai}", onclick: move |_| art.set(i), "{n}" }
                        button { r#type: "button", class: "oxscn-tab-x", "aria-label": "Close {n}", onclick: move |_| { arts.write().remove(i); art.set(0); }, {ico("close")} }
                    }
                }
            }
            if loading { {skel(10)} }
            else if a.is_empty() { {empty("Nothing open", "Files the assistant makes open here as tabs.")} }
            else if cur == "fabric-compare.csv" { p { class: "oxscn-row-a oxcdr-meta", "fabric-compare.csv · just now" } SheetEditor {} }
            else {
                ol { class: "oxcdr-src",
                    for (i, (t, d)) in [("Linen weights explained", "textiles.example.org"), ("Hemp twill lead times", "mills.example.com"), ("Tencel care notes", "fibre.example.net")].into_iter().enumerate() {
                        li { key: "{i}", span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-row-a", "{d}" } }
                    }
                }
            }
        }
    };
    rsx! {
        RailFrame { layout, state: state.clone(), height, threads: thr, active, work, work_label: "Artifacts",
            on_new: move |_| { msgs.set(vec![]); active.set(0); },
            div { class: "oxcdr",
                h1 { class: "oxcdr-t", "{title}" }
                if loading { {skel(8)} }
                else if blank && msgs().is_empty() { {empty("Ask with the web", "Answers search the web, then fold the searches into one line you can open.")} }
                else {
                    div { class: "oxscn-log",
                        if !blank {
                            div { class: "oxscn-msg", "data-me": "true", p { "Compare five fabrics for the overshirt: weight, price per metre and lead time." } }
                            div { class: "oxscn-msg", "data-me": "false", span { class: "oxscn-mono oxscn-mono-s", "K" }
                                div { class: "oxcdr-turn",
                                    button { r#type: "button", class: "oxcdr-ran", "aria-expanded": "{to}", onclick: move |_| tools_open.toggle(), {ico("globe")} span { "Ran 3 searches · 4.1s" } {chevron(to)} }
                                    if to {
                                        ul { class: "oxcdr-calls", li { "web.search “linen weight gsm” · 1.2s" } li { "web.search “hemp twill lead time” · 1.6s" } li { "web.read 3 pages · 1.3s" } }
                                    }
                                    p { "Washed linen and hemp twill are the sturdiest; organic cotton is cheapest and quickest to arrive. I put all five side by side." }
                                    div { class: "oxcdr-chips",
                                        button { r#type: "button", class: "oxscn-chip oxscn-chip-b", onclick: move |_| open_art("fabric-compare.csv"), {file_ico("fabric-compare.csv")} "fabric-compare.csv" }
                                        button { r#type: "button", class: "oxscn-chip oxscn-chip-b", onclick: move |_| open_art("sources.md"), {file_ico("sources.md")} "sources.md" }
                                    }
                                }
                            }
                        }
                    }
                    {convo(&msgs(), now, 1200)}
                }
                ScreenComposer { text, on_send: send }
            }
        }
    }
}

/// Screen – AI Chat / Progress: a chat whose tool calls open into cards, with its working notes
/// alongside, in the page itself – the steps ticking off, what it made and what it used.
#[component]
pub fn ChatProgressScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::parity::screensfid::{progress_steps, progress_tools, step_state};
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 1u32);
    let text = use_signal(String::new);
    let msgs = use_signal(Vec::<Msg>::new);
    let start = use_signal(|| now.0);
    let mut open_tool = use_signal(|| usize::MAX);
    let mut sec = use_signal(|| vec![true, true, true]);
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let steps = progress_steps();
    let done = if st == ScreenState::Sample {
        (2 + (since(now, start()) / 1500) as usize).min(steps.len())
    } else {
        0
    };
    let said = if done >= steps.len() {
        "All steps done".to_string()
    } else {
        format!("Working on: {}", steps[done])
    };
    let send = move |t: String| push_pair(msgs, now, t);
    let s = sec();
    let ot = open_tool();
    let s2 = s.clone();
    let head = move |i: usize, l: &'static str, n: String| {
        let o = s2[i];
        rsx! { button { r#type: "button", class: "oxcpr-sh", "aria-expanded": "{o}", onclick: move |_| { sec.write()[i] = !o; }, span { "{l}" } span { class: "oxscn-row-a", "{n}" } {chevron(o)} } }
    };
    let n_steps = format!("{done} of {}", steps.len());
    let work = rsx! {
        div { class: "oxcpr-pane",
            if loading { {skel(10)} }
            else if blank { {empty("Nothing running", "Steps, outputs and the apps it uses show here while it works.")} }
            else {
                {head(0, "Progress", n_steps)}
                if s[0] {
                    ul { class: "oxcpr-steps",
                        for (i, l) in steps.iter().enumerate() {
                            { let k = step_state(i, done).key(); rsx! { li { key: "{i}", "data-s": "{k}", span { class: "oxcpr-tick", if k == "done" { {check_glyph()} } } "{l}" } } }
                        }
                    }
                    p { class: "oxscn-sr", role: "status", "{said}" }
                }
                {head(1, "Outputs", "3".to_string())}
                if s[1] {
                    for (i, (f, b, k)) in [("Welcome plan", "notion", "Notion page"), ("Buddy checklist", "googledrive", "Sheet"), ("First-week notes.md", "", "Markdown")].into_iter().enumerate() {
                        div { key: "{i}", class: "oxcov-li", if b.is_empty() { {ico("file")} } else { BrandIcon { name: b.to_string(), size: 15.0, coloured: true } } span { class: "oxscn-row-t", "{f}" } span { class: "oxscn-row-a", "{k}" } }
                    }
                }
                {head(2, "Used in this chat", String::new())}
                if s[2] {
                    dl { class: "oxcag-dl",
                        div { dt { "Uploads" } dd { "2 files" } }
                        div { dt { "Apps" } dd { class: "oxcpr-apps", BrandIcon { name: "notion".to_string(), size: 14.0, coloured: true } BrandIcon { name: "googledrive".to_string(), size: 14.0, coloured: true } BrandIcon { name: "slack".to_string(), size: 14.0, coloured: true } } }
                        div { dt { "Web" } dd { "6 pages" } }
                        div { dt { "Skills" } dd { "Onboarding plan" } }
                    }
                }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Plan the new starters' week", work, work_label: "Progress",
            div { class: "oxcpr",
                if loading { {skel(9)} }
                else if blank { {empty("Ask for something that takes a few steps", "You'll see each step and every tool it calls as it goes.")} }
                else {
                    div { class: "oxscn-log",
                        div { class: "oxscn-msg", "data-me": "true", p { "Three people start Monday. Plan their first week and pair each with a buddy." } }
                        for (i, (t, d, secs)) in progress_tools().into_iter().enumerate() {
                            {
                                let o = ot == i;
                                let secs_s = format!("{secs:.1}s");
                                rsx! {
                                    div { key: "{i}", class: "oxcpr-tool", "data-open": "{o}",
                                        button { r#type: "button", class: "oxcpr-tool-b", "aria-expanded": "{o}", onclick: move |_| open_tool.set(if o { usize::MAX } else { i }),
                                            span { class: "oxcpr-ok", {check_glyph()} } span { class: "oxcpr-tn", "{t}" } span { class: "oxscn-row-t", "{d}" } span { class: "oxscn-row-a", "{secs_s}" } {chevron(o)} }
                                        if o { pre { class: "oxcpr-io", "in: {t}(team: \"design\")\nout: 3 results" } }
                                    }
                                }
                            }
                        }
                        div { class: "oxscn-msg", "data-me": "false", span { class: "oxscn-mono oxscn-mono-s", "K" }
                            div { p { "Here is the week at a glance:" }
                                ul { li { "Monday: laptops, accounts and lunch with their buddy" } li { "Tuesday to Thursday: one shadow day per team" } li { "Friday: a small first task, shipped together" } } }
                        }
                    }
                    {convo(&msgs(), now, 1200)}
                }
                ScreenComposer { text, on_send: send }
            }
        }
    }
}

/// Screen – AI Chat / Run: one agent run – the plan ticking through its steps (rename with Enter,
/// move with Alt+arrows, Delete removes), live time and cost, a step that waits for your yes, and
/// the draft it wrote with suggestions to accept (Y) or reject (N).
#[component]
pub fn ChatRunScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::parity::screensfid::{
        clock, money, move_step, next_open, run_cents, run_plan, step_state, suggestions,
    };
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut plan = use_signal(run_plan);
    let start = use_signal(|| now.0);
    let mut stopped = use_signal(|| None::<u32>);
    let mut approved = use_signal(|| None::<(bool, u32)>);
    let mut edit = use_signal(|| None::<usize>);
    let mut focus_i = use_signal(|| 0usize);
    let mut rev = use_signal(|| vec![Review::Open; suggestions().len()]);
    let mut cur_s = use_signal(|| 0usize);
    let mut act_open = use_signal(|| usize::MAX);
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let p = plan();
    // Elapsed run time: frozen when stopped; the gate step waits for approval.
    let live = (since(now, start()) / 1000) as u32 + 7;
    let mut t = stopped().unwrap_or(live);
    let gate = p
        .iter()
        .position(|s| s.tool == "mail.send")
        .unwrap_or(usize::MAX);
    let mut acc = 0u32;
    let mut done = 0usize;
    let mut waiting = false;
    for (i, s) in p.iter().enumerate() {
        if i == gate {
            match approved() {
                None => {
                    if t >= acc {
                        waiting = true;
                        t = t.min(acc + 1);
                    }
                    break;
                }
                Some((false, _)) => {
                    waiting = false;
                    break;
                }
                Some((true, at)) => {
                    acc = acc.max(at);
                }
            }
        }
        if t >= acc + s.secs {
            acc += s.secs;
            done = i + 1;
        } else {
            break;
        }
    }
    let held = matches!(approved(), Some((false, _)));
    let finished = done >= p.len();
    let status = if stopped().is_some() {
        "Stopped"
    } else if held {
        "On hold"
    } else if waiting {
        "Needs you"
    } else if finished {
        "Done"
    } else {
        "Running"
    };
    let tone = match status {
        "Done" => "ok",
        "Needs you" | "On hold" => "warn",
        "Stopped" => "muted",
        _ => "run",
    };
    let calls = done * 2 + usize::from(!finished);
    let time_s = clock(t);
    let cost_s = money(run_cents(t));
    let steps_s = format!("{done}/{}", p.len());
    let n = p.len();
    let sug = suggestions();
    let r = rev();
    let open = oxyn_core::parity::screensfid::open_count(&r);
    let cs = cur_s().min(sug.len() - 1);
    let mut decide = move |ok: bool| {
        let c = cur_s();
        rev.write()[c] = if ok {
            Review::Accepted
        } else {
            Review::Rejected
        };
        if let Some(nx) = next_open(&rev.peek(), c) {
            cur_s.set(nx);
        }
    };
    let count_s = open_words(&r);
    let work = rsx! {
        div { class: "oxcrn-side",
            if loading { {skel(10)} }
            else if blank { {empty("Nothing needs you yet", "When a run reaches a step that needs a yes, it waits here.")} }
            else {
                if waiting && approved().is_none() && stopped().is_none() {
                    div { class: "oxcrn-gate", role: "alert",
                        p { class: "oxcrn-gate-t", "Send the member update?" }
                        p { class: "oxscn-row-a", "212 members get it by email. You can still edit the draft below." }
                        div { class: "oxcrn-gate-b",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| approved.set(Some((false, 0))), "Hold" }
                            button { r#type: "button", class: "oxscn-primary", onclick: move |_| { let l = (since(now, start()) / 1000) as u32 + 7; approved.set(Some((true, l))); }, "Send it" }
                        }
                    }
                } else {
                    p { class: "oxscn-hint", match approved() { Some((true, _)) => "You said yes to sending. Nothing else needs you.", Some((false, _)) => "On hold: the run waits until you send it.", None => "Nothing needs you yet." } }
                }
                div { class: "oxcrn-draft",
                    div { class: "oxcov-h", h3 { "Draft" } span { class: "oxscn-row-a", role: "status", "{count_s}" } }
                    div { class: "oxcrn-doc", tabindex: "0", "aria-label": "Draft with suggestions; Y accepts, N rejects",
                        onkeydown: move |e: KeyboardEvent| match e.key() { Key::Character(ref c) if c == "y" => decide(true), Key::Character(ref c) if c == "n" => decide(false), _ => {} },
                        "In October membership "
                        for (i, s) in sug.iter().enumerate() {
                            {
                                let k = match r[i] { Review::Open => "open", Review::Accepted => "yes", Review::Rejected => "no" };
                                let on = i == cs && r[i] == Review::Open;
                                rsx! {
                                    span { key: "{i}", class: "oxcrn-sug", "data-r": "{k}", "data-on": "{on}", onclick: move |_| cur_s.set(i),
                                        if k != "yes" { del { "{s.del}" } } if k != "no" { ins { "{s.ins}" } }
                                    }
                                    if i < 4 { " · " }
                                }
                            }
                        }
                    }
                    div { class: "oxcrn-rv",
                        button { r#type: "button", class: "oxscn-btn", disabled: open == 0, onclick: move |_| decide(false), "Reject" kbd { class: "oxscn-kbd", "N" } }
                        button { r#type: "button", class: "oxscn-btn", disabled: open == 0, onclick: move |_| decide(true), "Accept" kbd { class: "oxscn-kbd", "Y" } }
                        span { class: "oxscn-grow" }
                        button { r#type: "button", class: "oxscn-link", disabled: open == 0, onclick: move |_| { for x in rev.write().iter_mut() { if *x == Review::Open { *x = Review::Rejected; } } }, "Reject all" }
                        button { r#type: "button", class: "oxscn-link", disabled: open == 0, onclick: move |_| { for x in rev.write().iter_mut() { if *x == Review::Open { *x = Review::Accepted; } } }, "Accept all" }
                    }
                }
                p { class: "oxscn-gh", "Files" }
                for (i, f) in ["member-update.md", "october-sales.csv", "signups-chart.png"].iter().enumerate() { div { key: "{i}", class: "oxcov-li", {file_ico(f)} span { class: "oxscn-row-t", "{f}" } } }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, side: "runs", title: "Runs / Member update", work, work_label: "Run details",
            actions: rsx! {
                span { class: "oxscn-tag", "data-tone": "{tone}", "{status}" }
                button { r#type: "button", class: "oxscn-btn", disabled: stopped().is_some() || finished || loading || blank, onclick: move |_| stopped.set(Some(t)), {ico("stop")} span { "Stop" } }
            },
            div { class: "oxcrn",
                if loading { {skel(10)} }
                else if blank { {empty("No runs yet", "Give an agent a goal; its plan, tool calls and the draft it writes show up here.")} }
                else {
                    p { class: "oxscn-row-a", span { class: "oxscn-mono oxscn-mono-s", "M" } " Started by {PERSON} at 7:48 PM" }
                    h1 { class: "oxcrn-t", "Write the November member update" }
                    dl { class: "oxcrn-stats",
                        div { dt { "Steps" } dd { "{steps_s}" } } div { dt { "Tool calls" } dd { "{calls}" } }
                        div { dt { "Time" } dd { "{time_s}" } } div { dt { "Cost" } dd { "{cost_s}" } }
                    }
                    div { class: "oxcov-h", h3 { "Plan" } span { class: "oxscn-row-a oxcrn-keys", "Enter renames · Alt+↑↓ moves · Delete removes" } }
                    ol { class: "oxcrn-plan", "aria-label": "Plan",
                        for (i, s) in p.iter().enumerate() {
                            {
                                let k = if i == gate && waiting { "wait" } else { step_state(i, done).key() };
                                let lbl = match k { "wait" => "Needs you", "done" => "Done", "active" => "Working", _ => "Up next" };
                                let ed = edit() == Some(i);
                                let title = s.title.clone();
                                let secs_s = format!("{}s", s.secs);
                                rsx! {
                                    li { key: "{i}-{title}", class: "oxcrn-step", "data-s": "{k}", tabindex: "0", "aria-label": "{title}, {lbl}",
                                        onfocus: move |_| focus_i.set(i),
                                        onkeydown: move |e: KeyboardEvent| {
                                            if edit().is_some() { return; }
                                            match e.key() {
                                                Key::Enter => { e.prevent_default(); edit.set(Some(i)); }
                                                Key::ArrowUp if e.modifiers().alt() => { e.prevent_default(); let j = move_step(&mut plan.write(), i, true); focus_i.set(j); }
                                                Key::ArrowDown if e.modifiers().alt() => { e.prevent_default(); let j = move_step(&mut plan.write(), i, false); focus_i.set(j); }
                                                Key::Delete | Key::Backspace if n > 1 => { e.prevent_default(); plan.write().remove(i); }
                                                _ => {}
                                            }
                                        },
                                        span { class: "oxcrn-dot" }
                                        span { class: "oxcrn-st",
                                            if ed {
                                                input { class: "oxcrn-in", "aria-label": "Step name", value: "{title}",
                                                    onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                    oninput: move |e| { plan.write()[i].title = e.value(); },
                                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter || e.key() == Key::Escape { e.prevent_default(); e.stop_propagation(); edit.set(None); },
                                                    onblur: move |_| edit.set(None) }
                                            } else { span { class: "oxcrn-sn", "{title}" } }
                                            span { class: "oxscn-row-a", "{s.why}" }
                                        }
                                        code { class: "oxcrn-tool", "{s.tool}" }
                                        span { class: "oxcrn-lbl", "{lbl}" }
                                        span { class: "oxscn-row-a", "{secs_s}" }
                                    }
                                }
                            }
                        }
                    }
                    p { class: "oxscn-sr", role: "status", "{status} · {steps_s}" }
                    div { class: "oxcov-h", h3 { "Activity" } }
                    for (i, s) in p.iter().enumerate().take(done) {
                        {
                            let o = act_open() == i;
                            let tl = s.title.clone();
                            let tool = s.tool;
                            rsx! {
                                div { key: "{i}", class: "oxcrn-act",
                                    button { r#type: "button", class: "oxcdr-ran", "aria-expanded": "{o}", onclick: move |_| act_open.set(if o { usize::MAX } else { i }),
                                        span { class: "oxcpr-ok", {check_glyph()} } span { "{tl} · used 1 tool" } {chevron(o)} }
                                    if o { pre { class: "oxcpr-io", "{tool} → ok" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── Folders ──

/// Folders seeded for a state: the sample set, one empty folder when the screen opens inside a
/// folder with no data, or none.
fn seed_folders(st: ScreenState, detail: bool) -> Vec<Folder> {
    match st {
        ScreenState::Sample => folders(),
        _ if detail => {
            let mut f = folders().remove(0);
            f.members = vec![PERSON_INI.to_string()];
            f.starred = false;
            vec![f]
        }
        _ => vec![],
    }
}

/// Up to four member initials, then `+n`.
fn faces(members: &[String]) -> Element {
    let more = members.len().saturating_sub(4);
    rsx! {
        span { class: "oxsfo-faces",
            for (i, m) in members.iter().take(4).enumerate() {
                span { key: "{i}", class: "oxscn-av oxsfo-face", title: "{member_name(m)}", "{m}" }
            }
            if more > 0 { span { class: "oxsfo-more", "+{more}" } }
        }
    }
}

/// Screen – Folders: the folders you share with people and agents (Overview), and one folder open
/// with its files and details (Detail). `detail` starts inside the first folder.
#[component]
pub fn FoldersScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Start inside the first folder.
    #[props(default)]
    detail: bool,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let list = use_signal(move || seed_folders(st, detail));
    let mut open = use_signal(move || if detail { Some(1u32) } else { None });
    let cur = open().and_then(|id| list().into_iter().find(|f| f.id == id));
    let title = match &cur {
        Some(f) => format!("Folders / {}", f.name),
        None => "Folders".to_string(),
    };
    let work = cur.as_ref().map(|f| {
        let id = f.id;
        rsx! { FolderInfo { key: "{id}", state: state.clone(), list, id } }
    });
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title, work, work_label: "Details",
            on_nav: move |k: String| if k == "files" { open.set(None) },
            if let Some(f) = cur {
                FolderFiles { key: "{f.id}", state: state.clone(), name: f.name.clone(), note: f.note.clone(), members: f.members.clone(), on_back: move |_| open.set(None) }
            } else {
                FolderGrid { state: state.clone(), list, on_open: move |id: u32| open.set(Some(id)) }
            }
        }
    }
}

/// The Overview page: search, sort, grid or list, star, and a new folder typed in place.
#[component]
fn FolderGrid(state: String, list: Signal<Vec<Folder>>, on_open: EventHandler<u32>) -> Element {
    let st = ScreenState::from_key(&state);
    let mut q = use_signal(String::new);
    let mut by_name = use_signal(|| false);
    let mut as_list = use_signal(|| false);
    let mut adding = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = list();
    let view = folder_view(&all, &q(), by_name());
    let shown = view.len();
    let mut create = move || {
        let name = draft.peek().trim().to_string();
        if name.is_empty() {
            return;
        }
        let id = list.peek().iter().map(|f| f.id).max().unwrap_or(0) + 1;
        list.write().push(Folder {
            id,
            name: name.clone(),
            note: "Empty so far".to_string(),
            members: vec![PERSON_INI.to_string()],
            mins: 0,
            starred: false,
            agents: false,
        });
        draft.set(String::new());
        adding.set(false);
        said.set(format!("Made {name}"));
    };
    let lay = if as_list() { "list" } else { "grid" };
    rsx! {
        div { class: "oxsfo",
            div { class: "oxsfo-head",
                div { class: "oxsfo-ht",
                    h1 { class: "oxscn-h1", "Folders" }
                    p { class: "oxscn-row-a", if st == ScreenState::Sample { "{all.len()} folders · shared with people and agents" } else { "Shared with people and agents" } }
                }
                button { r#type: "button", class: "oxscn-primary oxsfo-new", onclick: move |_| adding.set(true), {ico("plus")} span { "New folder" } }
            }
            div { class: "oxsfo-bar",
                label { class: "oxscn-search oxsfo-q",
                    {ico("search")}
                    input { r#type: "search", placeholder: "Find a folder", "aria-label": "Find a folder", value: "{q}",
                        oninput: move |e| q.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { q.set(String::new()) } }
                }
                MenuButton { label: if by_name() { "Name".to_string() } else { "Recent".to_string() }, aria: "Sort folders", icon: "sort", right: true,
                    items: vec![("recent", "Recently changed", ""), ("name", "Name", "")], checked: if by_name() { "name".to_string() } else { "recent".to_string() },
                    on_pick: move |k: String| by_name.set(k == "name") }
                div { class: "oxscn-seg oxsfo-view", role: "group", "aria-label": "View",
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Grid", "aria-pressed": "{!as_list()}", onclick: move |_| as_list.set(false), {ico("apps")} }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "List", "aria-pressed": "{as_list()}", onclick: move |_| as_list.set(true), {ico("row")} }
                }
            }
            if st == ScreenState::Loading {
                div { class: "oxsfo-grid", "data-view": "grid", "aria-hidden": "true",
                    for i in 0..6 { div { key: "{i}", class: "oxsfo-card oxsfo-sk", {skel(3)} } }
                }
            } else {
                if all.is_empty() && !adding() {
                    {empty("No folders yet", "Make one for a project: files, chats and agents can all share it.")}
                }
                ul { class: "oxsfo-grid", "data-view": "{lay}", "aria-label": "Folders",
                    if adding() {
                        li { class: "oxsfo-card oxsfo-add",
                            span { class: "oxsfo-fi", {ico("folder")} }
                            input { class: "oxsfo-in", r#type: "text", placeholder: "Folder name", "aria-label": "New folder name", value: "{draft}",
                                onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                oninput: move |e| draft.set(e.value()),
                                onkeydown: move |e: KeyboardEvent| match e.key() { Key::Enter => create(), Key::Escape => { adding.set(false); draft.set(String::new()); } _ => {} } }
                            div { class: "oxsfo-add-a",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| { adding.set(false); draft.set(String::new()); }, "Cancel" }
                                button { r#type: "button", class: "oxscn-primary", disabled: draft().trim().is_empty(), onclick: move |_| create(), "Create" }
                            }
                        }
                    }
                    for f in view {
                        {
                            let id = f.id;
                            let star = f.starred;
                            let nm = f.name.clone();
                            let ago_s = if f.mins == 0 { "Just now".to_string() } else { ago(f.mins) };
                            rsx! {
                                li { key: "{id}", class: "oxsfo-card", "data-star": "{star}",
                                    button { r#type: "button", class: "oxsfo-open", onclick: move |_| on_open.call(id),
                                        span { class: "oxsfo-fi", {ico("folder")} }
                                        span { class: "oxsfo-t",
                                            span { class: "oxsfo-n", "{f.name}" }
                                            span { class: "oxsfo-note", "{f.note}" }
                                        }
                                    }
                                    div { class: "oxsfo-foot",
                                        {faces(&f.members)}
                                        if f.agents { span { class: "oxsfo-ag", title: "Agents can read this folder", {ico("spark")} span { "Agents" } } }
                                        span { class: "oxscn-row-a", "{ago_s}" }
                                    }
                                    button { r#type: "button", class: "oxscn-ib oxsfo-star", "aria-label": "Star {nm}", "aria-pressed": "{star}",
                                        onclick: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.starred = !x.starred; } },
                                        {star_glyph(star)} }
                                }
                            }
                        }
                    }
                }
                if !all.is_empty() && shown == 0 {
                    p { class: "oxscn-hint", role: "status", "No folder matches “{q}”." }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

fn star_glyph(on: bool) -> Element {
    let fill = if on { "currentColor" } else { "none" };
    rsx! {
        svg { class: "oxscn-ico", width: "16", height: "16", view_box: "0 0 24 24", fill: "{fill}", stroke: "currentColor", stroke_width: "1.8", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "m12 4 2.4 5 5.4.6-4 3.7 1.1 5.4L12 16l-4.9 2.7 1.1-5.4-4-3.7 5.4-.6z" }
        }
    }
}

/// The Detail page: a folder's files with sort, select, delete with undo, and add.
#[component]
fn FolderFiles(
    state: String,
    name: String,
    note: String,
    members: Vec<String>,
    on_back: EventHandler<()>,
) -> Element {
    let st = ScreenState::from_key(&state);
    let mut files = use_signal(move || {
        if st == ScreenState::Sample {
            folder_files()
        } else {
            vec![]
        }
    });
    let mut sort = use_signal(|| (FileSort::Edited, false));
    let mut picked = use_signal(Vec::<u32>::new);
    let mut undo = use_signal(|| None::<Vec<FileRow>>);
    let mut said = use_signal(String::new);
    let (by, rev) = sort();
    let mut rows = files();
    sort_files(&mut rows, by, rev);
    let sel = picked();
    let n_sel = sel.len();
    let all_on = !rows.is_empty() && n_sel == rows.len();
    let total = total_size(&rows);
    let count = rows.len();
    let mut del = move || {
        let p = picked.peek().clone();
        if p.is_empty() {
            return;
        }
        let before = files.peek().clone();
        files.write().retain(|f| !p.contains(&f.id));
        picked.set(vec![]);
        undo.set(Some(before));
        said.set(if p.len() == 1 {
            "Deleted 1 file".to_string()
        } else {
            format!("Deleted {} files", p.len())
        });
    };
    let add = move |_| {
        let names: Vec<String> = files.peek().iter().map(|f| f.name.clone()).collect();
        let nm = untitled(&names, "untitled", ".md");
        let id = files.peek().iter().map(|f| f.id).max().unwrap_or(0) + 1;
        files.write().push(FileRow {
            id,
            name: nm.clone(),
            kind: "Doc",
            owner: PERSON_INI,
            mins: 0,
            kb: 1,
        });
        sort.set((FileSort::Edited, false));
        said.set(format!("Added {nm}"));
    };
    let th = move |k: FileSort, l: &'static str| {
        let on = by == k;
        // Edited (newest first) and Size (largest first) start descending.
        let desc = matches!(k, FileSort::Edited | FileSort::Size) != rev;
        let aria = if !on {
            "none"
        } else if desc {
            "descending"
        } else {
            "ascending"
        };
        rsx! {
            th { scope: "col", "aria-sort": "{aria}",
                button { r#type: "button", class: "oxsfd-sort", "data-on": "{on}", onclick: move |_| { let (b, r) = sort(); sort.set((k, if b == k { !r } else { false })); },
                    "{l}" if on { span { class: "oxsfd-arr", "aria-hidden": "true", if desc { "↓" } else { "↑" } } } }
            }
        }
    };
    rsx! {
        div { class: "oxsfd",
            button { r#type: "button", class: "oxscn-link oxsfd-back", onclick: move |_| on_back.call(()), {ico("back")} "All folders" }
            div { class: "oxsfo-head",
                div { class: "oxsfo-ht",
                    h1 { class: "oxscn-h1", "{name}" }
                    p { class: "oxscn-row-a", "{note}" }
                }
                {faces(&members)}
                button { r#type: "button", class: "oxscn-primary", disabled: st == ScreenState::Loading, onclick: add, {ico("plus")} span { "Add file" } }
            }
            if n_sel > 0 {
                div { class: "oxsfd-selbar", role: "toolbar", "aria-label": "Selected files",
                    span { "{n_sel} selected" }
                    span { class: "oxscn-grow" }
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| picked.set(vec![]), "Clear" }
                    button { r#type: "button", class: "oxscn-btn oxsfd-del", onclick: move |_| del(), "Delete" }
                }
            } else if undo().is_some() {
                div { class: "oxsfd-selbar", role: "status",
                    span { "{said}" }
                    span { class: "oxscn-grow" }
                    button { r#type: "button", class: "oxscn-link", onclick: move |_| { if let Some(b) = undo.take() { files.set(b); said.set("Restored".to_string()); } }, "Undo" }
                }
            }
            if st == ScreenState::Loading { {skel(8)} }
            else if rows.is_empty() {
                {empty("Nothing in this folder", "Add a file and everyone it is shared with sees it; agents too, if you let them.")}
            } else {
                div { class: "oxsfd-wrap",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Delete { del() },
                    table { class: "oxsfd-tb",
                        thead {
                            tr {
                                th { scope: "col", class: "oxsfd-ck",
                                    input { r#type: "checkbox", "aria-label": "Select all files", checked: all_on,
                                        onchange: move |_| { if all_on { picked.set(vec![]); } else { picked.set(files.peek().iter().map(|f| f.id).collect()); } } } }
                                {th(FileSort::Name, "Name")}
                                {th(FileSort::Kind, "Kind")}
                                th { scope: "col", class: "oxsfd-own", "Owner" }
                                {th(FileSort::Edited, "Edited")}
                                {th(FileSort::Size, "Size")}
                            }
                        }
                        tbody {
                            for f in rows {
                                {
                                    let id = f.id;
                                    let on = sel.contains(&id);
                                    let ed = if f.mins == 0 { "Just now".to_string() } else { ago(f.mins) };
                                    rsx! {
                                        tr { key: "{id}", "data-on": "{on}",
                                            td { class: "oxsfd-ck", input { r#type: "checkbox", "aria-label": "Select {f.name}", checked: on,
                                                onchange: move |_| { let mut p = picked.write(); if let Some(i) = p.iter().position(|x| *x == id) { p.remove(i); } else { p.push(id); } } } }
                                            td { class: "oxsfd-name", {glyph(if f.kind == "Notes" { "note" } else { "file" }, kind_icon(f.kind))} span { class: "oxscn-row-t", "{f.name}" } }
                                            td { class: "oxscn-row-a", "{f.kind}" }
                                            td { class: "oxsfd-own", span { class: "oxscn-av oxsfo-face", title: "{member_name(f.owner)}", "{f.owner}" } }
                                            td { class: "oxscn-row-a", "{ed}" }
                                            td { class: "oxscn-row-a oxsfd-num", "{size_label(f.kb)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "oxsfd-total oxscn-row-a", "{count} files · {total}" }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

fn kind_icon(kind: &str) -> &'static str {
    match kind {
        "Sheet" => "sheet",
        "Image" => "image",
        "Notes" => "note",
        _ => "file",
    }
}

/// The Details area of an open folder: about, who it is shared with (role menu, remove, invite),
/// whether agents may read it, and recent activity.
#[component]
fn FolderInfo(state: String, list: Signal<Vec<Folder>>, id: u32) -> Element {
    let st = ScreenState::from_key(&state);
    let f = list().into_iter().find(|f| f.id == id);
    let mut roles = use_signal(Vec::<(String, &'static str)>::new);
    let mut invite = use_signal(String::new);
    let mut bad = use_signal(|| false);
    let Some(f) = f else { return rsx! {} };
    let agents = f.agents;
    let members = f.members.clone();
    let role_of = move |m: &str| {
        roles()
            .iter()
            .find(|(k, _)| k == m)
            .map(|(_, r)| *r)
            .unwrap_or(if m == PERSON_INI { "owner" } else { "edit" })
    };
    let mut send_invite = move || {
        let v = invite.peek().trim().to_string();
        match invite_initials(&v) {
            Some(ini) => {
                if let Some(x) = list.write().iter_mut().find(|x| x.id == id)
                    && !x.members.contains(&ini)
                {
                    x.members.push(ini);
                }
                invite.set(String::new());
                bad.set(false);
            }
            None => bad.set(true),
        }
    };
    rsx! {
        div { class: "oxsfi",
            if st == ScreenState::Loading { {skel(9)} } else {
                p { class: "oxscn-gh", "About" }
                p { class: "oxsfi-about", "{f.note}" }
                p { class: "oxscn-gh", "Shared with" }
                ul { class: "oxsfi-people",
                    for m in members {
                        {
                            let r = role_of(&m);
                            let nm = member_name(&m);
                            let m2 = m.clone();
                            rsx! {
                                li { key: "{m}", class: "oxsfi-p",
                                    span { class: "oxscn-av", "{m}" }
                                    span { class: "oxscn-row-t", "{nm}" }
                                    if r == "owner" { span { class: "oxscn-row-a", "Owner" } } else {
                                        MenuButton { label: if r == "view" { "Can view".to_string() } else { "Can edit".to_string() }, aria: "Access for {nm}", right: true, class: "oxsfi-role".to_string(),
                                            items: vec![("edit", "Can edit", ""), ("view", "Can view", ""), ("remove", "Remove", "")], checked: r.to_string(),
                                            on_pick: move |k: String| {
                                                if k == "remove" { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.members.retain(|y| *y != m2); } }
                                                else { let k: &'static str = if k == "view" { "view" } else { "edit" }; let mut v = roles.write(); v.retain(|(a, _)| *a != m2); v.push((m2.clone(), k)); }
                                            } }
                                    }
                                }
                            }
                        }
                    }
                }
                label { class: "oxscn-search oxsfi-inv", "data-bad": "{bad}",
                    {ico("plus")}
                    input { r#type: "email", placeholder: "Invite by email", "aria-label": "Invite by email", "aria-invalid": "{bad}", value: "{invite}",
                        oninput: move |e| { invite.set(e.value()); bad.set(false); },
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { send_invite() } }
                }
                if bad() { p { class: "oxscn-hint", role: "status", "That does not look like an email address." } }
                div { class: "oxcag-sw oxsfi-sw",
                    span { "Agents can read this folder" }
                    button { r#type: "button", role: "switch", class: "oxscn-switch", "aria-checked": "{agents}", "aria-label": "Agents can read this folder",
                        onclick: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.agents = !x.agents; } }, span {} }
                }
                p { class: "oxscn-gh", "Activity" }
                if st == ScreenState::Empty { p { class: "oxscn-hint", "Edits, uploads and agent reads show up here." } } else {
                    for (i, (w, t, a)) in folder_activity().into_iter().enumerate() {
                        div { key: "{i}", class: "oxsfi-act", span { class: "oxscn-av oxsfo-face", "{w}" } span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-row-a", "{a}" } }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── Integrations ──

/// Screen – Integrations: an app directory with category tabs and search; Connect asks for the
/// permissions first, a connected app opens its details beside the list (permissions, sync,
/// disconnect).
#[component]
pub fn IntegrationsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut list = use_signal(move || apps(st == ScreenState::Sample));
    let mut tab = use_signal(|| "all".to_string());
    let mut q = use_signal(String::new);
    let mut pick = use_signal(move || {
        if st == ScreenState::Sample {
            Some("slack")
        } else {
            None
        }
    });
    let mut ask = use_signal(|| None::<&'static str>);
    let mut grant = use_signal(Vec::<bool>::new);
    let mut said = use_signal(String::new);
    let all = list();
    let view = app_view(&all, &tab(), &q());
    let n_on = connected_count(&all);
    let loading = st == ScreenState::Loading;
    let cur_tab = tab();
    let asking = ask().and_then(|id| all.iter().find(|a| a.id == id).cloned());
    let picked = pick().and_then(|id| all.iter().find(|a| a.id == id && a.connected).cloned());
    let mut open_ask = move |id: &'static str| {
        let n = list
            .peek()
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.scopes.len())
            .unwrap_or(0);
        grant.set(vec![true; n]);
        ask.set(Some(id));
    };
    let mut allow = move || {
        let Some(id) = *ask.peek() else { return };
        let g = grant.peek().clone();
        let mut name = "";
        if let Some(a) = list.write().iter_mut().find(|a| a.id == id) {
            a.connected = true;
            a.synced = 0;
            for (s, on) in a.scopes.iter_mut().zip(g) {
                s.1 = on;
            }
            name = a.name;
        }
        ask.set(None);
        pick.set(Some(id));
        said.set(format!("{name} connected"));
    };
    let tabs_k: Vec<&'static str> = APP_TABS.iter().map(|(k, _)| *k).collect();
    let work = picked.clone().map(|a| rsx! { AppDetail { key: "{a.id}", app: a, list, on_gone: move |n: String| { pick.set(None); said.set(format!("{n} disconnected")); } } });
    let lead = if loading {
        "Loading your apps…".to_string()
    } else if n_on == 0 {
        "Nothing connected yet".to_string()
    } else if n_on == 1 {
        "1 app connected".to_string()
    } else {
        format!("{n_on} apps connected")
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Integrations", work, work_label: "App",
            div { class: "oxsin",
                div { class: "oxsfo-head",
                    div { class: "oxsfo-ht",
                        h1 { class: "oxscn-h1", "Integrations" }
                        p { class: "oxscn-row-a", "{lead} · the assistant only touches what you allow" }
                    }
                }
                label { class: "oxscn-search oxsin-q",
                    {ico("search")}
                    input { r#type: "search", placeholder: "Find an app", "aria-label": "Find an app", value: "{q}",
                        oninput: move |e| q.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { q.set(String::new()) } }
                }
                div { class: "oxsin-tabs", role: "tablist", "aria-label": "Categories",
                    onkeydown: move |e: KeyboardEvent| {
                        let d: i32 = match e.key() { Key::ArrowRight => 1, Key::ArrowLeft => -1, _ => 0 };
                        if d != 0 {
                            e.prevent_default();
                            let i = tabs_k.iter().position(|k| *k == tab()).unwrap_or(0) as i32;
                            let n = (i + d).rem_euclid(tabs_k.len() as i32) as usize;
                            tab.set(tabs_k[n].to_string());
                        }
                    },
                    for (k, l) in APP_TABS {
                        {
                            let on = cur_tab == k;
                            let n = if k == "on" { Some(n_on) } else { None };
                            rsx! {
                                button { key: "{k}", r#type: "button", role: "tab", class: "oxsin-tab", "aria-selected": "{on}", tabindex: if on { "0" } else { "-1" },
                                    onclick: move |_| tab.set(k.to_string()),
                                    "{l}"
                                    if let Some(n) = n { span { class: "oxsin-n", "{n}" } }
                                }
                            }
                        }
                    }
                }
                if loading {
                    div { class: "oxsin-grid", "aria-hidden": "true", for i in 0..6 { div { key: "{i}", class: "oxsin-card", {skel(3)} } } }
                } else if view.is_empty() {
                    if cur_tab == "on" && q().trim().is_empty() {
                        {empty("No apps connected", "Connect one from All: you choose what it may read and do, and can switch any of it off later.")}
                    } else {
                        p { class: "oxscn-hint", role: "status", "No app matches “{q}”." }
                    }
                } else {
                    ul { class: "oxsin-grid", "aria-label": "Apps",
                        for a in view {
                            {
                                let id = a.id;
                                let on = a.connected;
                                let sel = pick() == Some(id) && on;
                                let nm = a.name;
                                rsx! {
                                    li { key: "{id}", class: "oxsin-card", "data-on": "{on}", "data-sel": "{sel}",
                                        div { class: "oxsin-top",
                                            span { class: "oxsin-mark", BrandIcon { name: id.to_string(), size: 20.0, coloured: true, fallback: true } }
                                            span { class: "oxsin-t", span { class: "oxsfo-n", "{nm}" } span { class: "oxscn-row-a", "{kind_label(a.kind)}" } }
                                        }
                                        p { class: "oxsin-b", "{a.blurb}" }
                                        div { class: "oxsin-foot",
                                            if on {
                                                span { class: "oxscn-tag", "data-tone": "ok", "Connected" }
                                                button { r#type: "button", class: "oxscn-btn", "aria-pressed": "{sel}", "aria-label": "Manage {nm}", onclick: move |_| pick.set(Some(id)), "Manage" }
                                            } else {
                                                span { class: "oxscn-row-a", "Not connected" }
                                                button { r#type: "button", class: "oxscn-btn oxsin-conn", "aria-label": "Connect {nm}", onclick: move |_| open_ask(id), {ico("plus")} span { "Connect" } }
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
            if let Some(a) = asking {
                div { class: "oxsin-scrim", onclick: move |_| ask.set(None) }
                div { class: "oxsin-dlg", role: "dialog", "aria-modal": "true", "aria-labelledby": "oxsin-dlg-t",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); ask.set(None); },
                    div { class: "oxsin-top",
                        span { class: "oxsin-mark", BrandIcon { name: a.id.to_string(), size: 20.0, coloured: true, fallback: true } }
                        h2 { id: "oxsin-dlg-t", class: "oxsin-dt", "Connect {a.name}" }
                    }
                    p { class: "oxsin-b", "Pick what it may do. You can change this any time from Manage." }
                    ul { class: "oxsin-scopes",
                        for (i, (s, _)) in a.scopes.iter().enumerate() {
                            {
                                let g = grant().get(i).copied().unwrap_or(true);
                                rsx! {
                                    li { key: "{i}",
                                        label { class: "oxsin-scope",
                                            input { r#type: "checkbox", checked: g, onchange: move |_| { if let Some(x) = grant.write().get_mut(i) { *x = !*x; } } }
                                            span { "{s}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxsfo-add-a",
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| ask.set(None), "Cancel" }
                        button { r#type: "button", class: "oxscn-primary", disabled: !grant().iter().any(|x| *x),
                            onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                            onclick: move |_| allow(), "Allow" }
                    }
                }
            }
        }
    }
}

fn kind_label(k: &str) -> &'static str {
    APP_TABS
        .iter()
        .find(|(x, _)| *x == k)
        .map(|(_, l)| *l)
        .unwrap_or("Other")
}

/// A connected app beside the directory: what it may do (switches), sync and disconnect.
#[component]
fn AppDetail(app: AppInfo, list: Signal<Vec<AppInfo>>, on_gone: EventHandler<String>) -> Element {
    let mut sure = use_signal(|| false);
    let id = app.id;
    let nm = app.name;
    let synced = synced_words(app.synced);
    rsx! {
        div { class: "oxsfi oxsin-det",
            div { class: "oxsin-top",
                span { class: "oxsin-mark", BrandIcon { name: id.to_string(), size: 22.0, coloured: true, fallback: true } }
                span { class: "oxsin-t", span { class: "oxsfo-n", "{nm}" } span { class: "oxscn-row-a", "{synced}" } }
            }
            p { class: "oxsfi-about", "{app.blurb}" }
            p { class: "oxscn-gh", "It may" }
            for (i, (s, on)) in app.scopes.iter().enumerate() {
                div { key: "{i}", class: "oxcag-sw oxsfi-sw",
                    span { "{s}" }
                    button { r#type: "button", role: "switch", class: "oxscn-switch", "aria-checked": "{on}", "aria-label": "{s}",
                        onclick: move |_| { if let Some(a) = list.write().iter_mut().find(|a| a.id == id) && let Some(x) = a.scopes.get_mut(i) { x.1 = !x.1; } }, span {} }
                }
            }
            div { class: "oxsin-acts",
                button { r#type: "button", class: "oxscn-btn", onclick: move |_| { if let Some(a) = list.write().iter_mut().find(|a| a.id == id) { a.synced = 0; } }, {ico("undo")} span { "Sync now" } }
                if sure() {
                    span { class: "oxsin-sure", role: "group", "aria-label": "Confirm disconnect",
                        span { "Disconnect?" }
                        button { r#type: "button", class: "oxscn-btn oxsfd-del", onclick: move |_| { if let Some(a) = list.write().iter_mut().find(|a| a.id == id) { a.connected = false; } on_gone.call(nm.to_string()); }, "Yes" }
                        button { r#type: "button", class: "oxscn-link", onclick: move |_| sure.set(false), "Keep" }
                    }
                } else {
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| sure.set(true), "Disconnect" }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────── Kanban ──

/// Kanban board: columns with counts and limits, cards you drag between columns (or move with
/// Alt + arrows), find, an Only mine filter, add a card in place, and the open card beside the
/// board with its status, priority and assignee.
#[component]
pub fn KanbanScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut list = use_signal(move || board(st == ScreenState::Sample));
    let mut q = use_signal(String::new);
    let mut mine = use_signal(|| false);
    let mut pick = use_signal(move || {
        if st == ScreenState::Sample {
            Some(11u32)
        } else {
            None
        }
    });
    let mut drag = use_signal(|| None::<u32>);
    let mut over = use_signal(|| None::<&'static str>);
    let mut adding = use_signal(|| None::<&'static str>);
    let mut draft = use_signal(String::new);
    let mut said = use_signal(String::new);
    let mut refocus = use_signal(|| None::<u32>);
    let all = list();
    let loading = st == ScreenState::Loading;
    let who = if mine() { Some(PERSON_INI) } else { None };
    let qs = q();
    let shown: usize = KB_COLS
        .iter()
        .map(|(k, _, _)| column(&all, k, &qs, who).len())
        .sum();
    let filtering = !qs.trim().is_empty() || who.is_some();
    let done = all.iter().filter(|t| t.col == "done").count();
    let lead = if loading {
        "Loading the board…".to_string()
    } else if all.is_empty() {
        "No cards yet".to_string()
    } else {
        format!("{} cards · {done} done", all.len())
    };
    let mut shift = move |id: u32, col: &'static str, at: usize| {
        if move_task(&mut list.write(), id, col, at) {
            said.set(format!("{} moved to {}", task_key(id), col_label(col)));
        }
    };
    let picked = pick().and_then(|id| all.iter().find(|t| t.id == id).cloned());
    let work = picked.map(|t| {
        rsx! { KanbanCardDetail { key: "{t.id}", task: t, list, on_gone: move |k: String| { pick.set(None); said.set(format!("{k} deleted")); } } }
    });
    let is_mine = mine();
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Autumn launch / Board", work, work_label: "Card",
            div { class: "oxskb",
                div { class: "oxsfo-head",
                    div { class: "oxsfo-ht",
                        h1 { class: "oxscn-h1", "Board" }
                        p { class: "oxscn-row-a", "{lead}" }
                    }
                }
                div { class: "oxskb-bar",
                    label { class: "oxscn-search oxskb-q",
                        {ico("search")}
                        input { r#type: "search", placeholder: "Find a card", "aria-label": "Find a card", value: "{q}",
                            oninput: move |e| q.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { q.set(String::new()) } }
                    }
                    button { r#type: "button", class: "oxscn-btn oxskb-mine", "aria-pressed": "{is_mine}", onclick: move |_| mine.toggle(),
                        span { class: "oxscn-av oxsfo-face", "{PERSON_INI}" } span { "Only mine" } }
                }
                if !loading && filtering && shown == 0 && !all.is_empty() {
                    p { class: "oxscn-hint", role: "status", "No card matches." }
                }
                if loading {
                    div { class: "oxskb-cols", "aria-hidden": "true",
                        for i in 0..4 { div { key: "{i}", class: "oxskb-col", {skel(4)} } }
                    }
                } else {
                    div { class: "oxskb-cols",
                        for (k, l, lim) in KB_COLS {
                            {
                                let cards = column(&all, k, &qs, who);
                                let n = all.iter().filter(|t| t.col == k).count();
                                let hot = over_limit(&all, k);
                                let is_drop = over() == Some(k);
                                let is_adding = adding() == Some(k);
                                let count = if lim > 0 { format!("{n}/{lim}") } else { n.to_string() };
                                let none = cards.is_empty();
                                rsx! {
                                    section { key: "{k}", class: "oxskb-col", "data-col": "{k}", "data-drop": "{is_drop}", "aria-label": "{l}, {n} cards",
                                        ondragover: move |e: DragEvent| { e.prevent_default(); if *over.peek() != Some(k) { over.set(Some(k)); } },
                                        ondrop: move |e: DragEvent| {
                                            e.prevent_default();
                                            over.set(None);
                                            if let Some(id) = *drag.peek() { let at = column(&list.peek(), k, "", None).len(); shift(id, k, at); }
                                            drag.set(None);
                                        },
                                        header { class: "oxskb-ch",
                                            h2 { class: "oxskb-cn", "{l}" }
                                            span { class: "oxskb-n", "data-over": "{hot}", title: if lim > 0 { "Limit {lim}" } else { "" }, "{count}" }
                                            button { r#type: "button", class: "oxscn-ib oxskb-add", "aria-label": "Add a card to {l}",
                                                onclick: move |_| { draft.set(String::new()); adding.set(Some(k)); }, {ico("plus")} }
                                        }
                                        if hot { p { class: "oxskb-warn", "Over the limit of {lim}" } }
                                        ul { class: "oxskb-list", "aria-label": "{l}",
                                            for t in cards {
                                                {
                                                    let id = t.id;
                                                    let key_s = task_key(id);
                                                    let sel = pick() == Some(id);
                                                    let dragging = drag() == Some(id);
                                                    let pri = usize::from(t.pri.min(3));
                                                    let pri_l = KB_PRI[pri];
                                                    let who_n = member_name(t.who);
                                                    rsx! {
                                                        li { key: "{id}", class: "oxskb-card", draggable: "true", tabindex: "0", "data-sel": "{sel}", "data-drag": "{dragging}",
                                                            "aria-label": "{key_s} {t.title}", "aria-keyshortcuts": "Alt+ArrowLeft Alt+ArrowRight Alt+ArrowUp Alt+ArrowDown",
                                                            onmounted: move |e: MountedEvent| { if *refocus.peek() == Some(id) { refocus.set(None); let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); } },
                                                            ondragstart: move |_| drag.set(Some(id)),
                                                            ondragend: move |_| { drag.set(None); over.set(None); },
                                                            ondrop: move |e: DragEvent| {
                                                                e.prevent_default();
                                                                e.stop_propagation();
                                                                over.set(None);
                                                                if let Some(d) = *drag.peek() && d != id {
                                                                    let at = column(&list.peek(), k, "", None).iter().position(|x| x.id == id).unwrap_or(0);
                                                                    shift(d, k, at);
                                                                }
                                                                drag.set(None);
                                                            },
                                                            onclick: move |_| pick.set(Some(id)),
                                                            onkeydown: move |e: KeyboardEvent| {
                                                                let alt = e.modifiers().alt();
                                                                let key = e.key();
                                                                if key == Key::Enter || key == Key::Character(" ".into()) {
                                                                    e.prevent_default();
                                                                    pick.set(Some(id));
                                                                } else if alt && (key == Key::ArrowLeft || key == Key::ArrowRight) {
                                                                    e.prevent_default();
                                                                    if let Some(c) = next_col(k, if key == Key::ArrowLeft { -1 } else { 1 }) {
                                                                        let at = column(&list.peek(), c, "", None).len();
                                                                        refocus.set(Some(id));
                                                                        shift(id, c, at);
                                                                    }
                                                                } else if alt && (key == Key::ArrowUp || key == Key::ArrowDown) {
                                                                    e.prevent_default();
                                                                    let pos = column(&list.peek(), k, "", None).iter().position(|x| x.id == id).unwrap_or(0);
                                                                    let to = if key == Key::ArrowUp { pos.saturating_sub(1) } else { pos + 1 };
                                                                    shift(id, k, to);
                                                                }
                                                            },
                                                            div { class: "oxskb-top",
                                                                span { class: "oxskb-key", "{key_s}" }
                                                                span { class: "oxskb-pri", "data-p": "{pri}", role: "img", "aria-label": "{pri_l}", title: "{pri_l}", span {} span {} span {} }
                                                            }
                                                            p { class: "oxskb-t", "{t.title}" }
                                                            div { class: "oxskb-meta",
                                                                span { class: "oxskb-tag", "{t.tag}" }
                                                                if t.sub.1 > 0 { span { class: "oxskb-sub", "data-all": "{t.sub.0 == t.sub.1}", {ico("check")} "{t.sub.0}/{t.sub.1}" } }
                                                                if let Some(d) = t.due { span { class: "oxskb-due", "data-late": "{d < 0}", "{due_words(d)}" } }
                                                                if !t.who.is_empty() { span { class: "oxscn-av oxsfo-face oxskb-who", title: "{who_n}", "{t.who}" } }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if is_adding {
                                            div { class: "oxskb-new",
                                                input { class: "oxskb-in", placeholder: "Card title", "aria-label": "New card in {l}", value: "{draft}",
                                                    onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                    oninput: move |e| draft.set(e.value()),
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        if e.key() == Key::Escape { adding.set(None); }
                                                        if e.key() == Key::Enter {
                                                            let v = draft.peek().clone();
                                                            if let Some(id) = add_task(&mut list.write(), k, &v) { said.set(format!("{} added to {l}", task_key(id))); draft.set(String::new()); }
                                                        }
                                                    } }
                                                div { class: "oxsfo-add-a",
                                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(None), "Cancel" }
                                                    button { r#type: "button", class: "oxscn-primary", disabled: draft().trim().is_empty(),
                                                        onclick: move |_| { let v = draft.peek().clone(); if let Some(id) = add_task(&mut list.write(), k, &v) { said.set(format!("{} added to {l}", task_key(id))); adding.set(None); } }, "Add" }
                                                }
                                            }
                                        } else if none {
                                            p { class: "oxskb-none", if all.is_empty() && k == "todo" { "Add the first card with +." } else if filtering { "None match" } else { "Drop cards here" } }
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
}

/// The open card beside the board: title (editable), status, priority and assignee menus,
/// sub-tasks, notes and Delete.
#[component]
fn KanbanCardDetail(task: Task, list: Signal<Vec<Task>>, on_gone: EventHandler<String>) -> Element {
    let id = task.id;
    let key_s = task_key(id);
    let pri = usize::from(task.pri.min(3));
    let cols: Vec<MenuItem> = KB_COLS.iter().map(|(k, l, _)| (*k, *l, "")).collect();
    let pris: Vec<MenuItem> = vec![
        ("0", KB_PRI[0], ""),
        ("1", KB_PRI[1], ""),
        ("2", KB_PRI[2], ""),
        ("3", KB_PRI[3], ""),
    ];
    let people: Vec<MenuItem> = vec![
        ("none", "Nobody", ""),
        ("RV", member_name("RV"), ""),
        ("JO", member_name("JO"), ""),
        ("TA", member_name("TA"), ""),
        ("SE", member_name("SE"), ""),
    ];
    let who_k = if task.who.is_empty() {
        "none".to_string()
    } else {
        task.who.to_string()
    };
    let who_l = if task.who.is_empty() {
        "Nobody".to_string()
    } else {
        member_name(task.who).to_string()
    };
    let (a, b) = task.sub;
    let pct = if b == 0 {
        0
    } else {
        u32::from(a) * 100 / u32::from(b)
    };
    rsx! {
        div { class: "oxsfi oxskb-det",
            p { class: "oxskb-key", "{key_s} · {col_label(task.col)}" }
            input { class: "oxskb-title", "aria-label": "Card title", value: "{task.title}",
                oninput: move |e| { if let Some(t) = list.write().iter_mut().find(|t| t.id == id) { t.title = e.value(); } } }
            dl { class: "oxskb-dl",
                div { dt { "Status" } dd { MenuButton { label: col_label(task.col).to_string(), aria: "Status", class: "oxskb-st".to_string(), items: cols, checked: task.col.to_string(), right: true,
                    on_pick: move |k: String| { if let Some(c) = KB_COLS.iter().map(|(x, _, _)| *x).find(|x| *x == k) { let at = column(&list.peek(), c, "", None).len(); let _ = move_task(&mut list.write(), id, c, at); } } } } }
                div { dt { "Priority" } dd { MenuButton { label: KB_PRI[pri].to_string(), aria: "Priority", class: "oxskb-pr".to_string(), items: pris, checked: pri.to_string(), right: true,
                    on_pick: move |k: String| { if let Ok(p) = k.parse::<u8>() && let Some(t) = list.write().iter_mut().find(|t| t.id == id) { t.pri = p; } } } } }
                div { dt { "Assignee" } dd { MenuButton { label: who_l, aria: "Assignee", class: "oxskb-as".to_string(), items: people, checked: who_k, right: true,
                    on_pick: move |k: String| {
                        let w: &'static str = match k.as_str() { "RV" => "RV", "JO" => "JO", "TA" => "TA", "SE" => "SE", _ => "" };
                        if let Some(t) = list.write().iter_mut().find(|t| t.id == id) { t.who = w; }
                    } } } }
                div { dt { "Due" } dd { if let Some(d) = task.due { "{due_words(d)}" } else { "No date" } } }
            }
            if b > 0 {
                p { class: "oxscn-gh", "Sub-tasks" }
                div { class: "oxskb-prog", role: "progressbar", "aria-label": "Sub-tasks done", "aria-valuenow": "{a}", "aria-valuemin": "0", "aria-valuemax": "{b}",
                    span { style: "width:{pct}%" } }
                p { class: "oxscn-row-a oxskb-pl", "{a} of {b} done" }
            }
            if !task.note.is_empty() {
                p { class: "oxscn-gh", "Notes" }
                p { class: "oxsfi-about", "{task.note}" }
            }
            div { class: "oxsin-acts",
                button { r#type: "button", class: "oxscn-btn oxsfd-del",
                    onclick: move |_| { list.write().retain(|t| t.id != id); on_gone.call(task_key(id)); }, "Delete card" }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────── CRM ──

/// CRM: saved views as tabs with counts, find, a deals table you sort by column and edit in
/// place (company and value fields, a stage menu), New deal, and the open deal beside the table.
#[component]
pub fn CrmScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut list = use_signal(move || deals(st == ScreenState::Sample));
    let mut view = use_signal(|| "all");
    let mut q = use_signal(String::new);
    let mut sort = use_signal(|| (DealSort::Value, false));
    let mut pick = use_signal(move || {
        if st == ScreenState::Sample {
            Some(1u32)
        } else {
            None
        }
    });
    // The cell being edited: deal id and field (`company` or `value`), with its draft.
    let mut edit = use_signal(|| None::<(u32, &'static str)>);
    let mut draft = use_signal(String::new);
    let mut bad = use_signal(|| false);
    let mut said = use_signal(String::new);
    let all = list();
    let loading = st == ScreenState::Loading;
    let (by, rev) = sort();
    let cur = view();
    let qs = q();
    let rows = deal_view(&all, cur, &qs, by, rev);
    let shown_total: u32 = rows.iter().map(|d| d.value).sum();
    let lead = if loading {
        "Loading deals…".to_string()
    } else if all.is_empty() {
        "No deals yet".to_string()
    } else {
        format!(
            "{} deals · {} open pipeline",
            all.len(),
            usd(pipeline(&all))
        )
    };
    let mut commit = move || {
        let Some((id, field)) = *edit.peek() else {
            return;
        };
        let v = draft.peek().trim().to_string();
        if field == "value" {
            match parse_money(&v) {
                Some(n) => {
                    if let Some(d) = list.write().iter_mut().find(|d| d.id == id) {
                        d.value = n;
                    }
                    said.set(format!("Value set to {}", usd(n)));
                }
                None => {
                    bad.set(true);
                    said.set("Type an amount like 12,400 or 12k".to_string());
                    return;
                }
            }
        } else if !v.is_empty()
            && let Some(d) = list.write().iter_mut().find(|d| d.id == id)
        {
            d.company = v;
        }
        bad.set(false);
        edit.set(None);
    };
    let mut start = move |id: u32, field: &'static str, now: String| {
        draft.set(now);
        bad.set(false);
        edit.set(Some((id, field)));
    };
    let th = move |k: DealSort, l: &'static str, num: bool| {
        let on = by == k;
        // Value starts largest first.
        let desc = (k == DealSort::Value) != rev;
        let aria = if !on {
            "none"
        } else if desc {
            "descending"
        } else {
            "ascending"
        };
        rsx! {
            th { scope: "col", class: if num { "oxscr-num" } else { "" }, "aria-sort": "{aria}",
                button { r#type: "button", class: "oxsfd-sort", "data-on": "{on}", onclick: move |_| { let (b, r) = sort(); sort.set((k, if b == k { !r } else { false })); },
                    "{l}" if on { span { class: "oxsfd-arr", "aria-hidden": "true", if desc { "↓" } else { "↑" } } } }
            }
        }
    };
    let picked = pick().and_then(|id| all.iter().find(|d| d.id == id).cloned());
    let work = picked.map(|d| {
        rsx! { CrmDealDetail { key: "{d.id}", deal: d, list, on_gone: move |n: String| { pick.set(None); said.set(format!("{n} deleted")); } } }
    });
    let views_k: Vec<&'static str> = CRM_VIEWS.iter().map(|(k, _)| *k).collect();
    let stages: Vec<MenuItem> = CRM_STAGES.iter().map(|(k, l)| (*k, *l, "")).collect();
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Sales / Deals", work, work_label: "Deal",
            div { class: "oxscr",
                div { class: "oxsfo-head",
                    div { class: "oxsfo-ht",
                        h1 { class: "oxscn-h1", "Deals" }
                        p { class: "oxscn-row-a", "{lead}" }
                    }
                    if !loading {
                        button { r#type: "button", class: "oxscn-primary oxscr-new",
                            onclick: move |_| {
                                let n = list.peek().len() + 1;
                                if let Some(id) = add_deal(&mut list.write(), &format!("New company {n}"), PERSON_INI) {
                                    view.set("all");
                                    q.set(String::new());
                                    pick.set(Some(id));
                                    start(id, "company", String::new());
                                    said.set("New deal added as a lead".to_string());
                                }
                            },
                            {ico("plus")} "New deal" }
                    }
                }
                div { class: "oxsin-tabs oxscr-tabs", role: "tablist", "aria-label": "Views",
                    onkeydown: move |e: KeyboardEvent| {
                        let d: i32 = match e.key() { Key::ArrowRight => 1, Key::ArrowLeft => -1, _ => 0 };
                        if d != 0 {
                            e.prevent_default();
                            let i = views_k.iter().position(|k| *k == view()).unwrap_or(0) as i32;
                            let n = (i + d).rem_euclid(views_k.len() as i32) as usize;
                            view.set(views_k[n]);
                        }
                    },
                    for (k, l) in CRM_VIEWS {
                        {
                            let on = cur == k;
                            let n = view_count(&all, k);
                            rsx! {
                                button { key: "{k}", r#type: "button", role: "tab", class: "oxsin-tab", "aria-selected": "{on}", tabindex: if on { "0" } else { "-1" },
                                    onclick: move |_| view.set(k),
                                    "{l}"
                                    if !loading { span { class: "oxsin-n", "{n}" } }
                                }
                            }
                        }
                    }
                }
                label { class: "oxscn-search oxscr-q",
                    {ico("search")}
                    input { r#type: "search", placeholder: "Find a company, contact or note", "aria-label": "Find a deal", value: "{q}",
                        oninput: move |e| q.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { q.set(String::new()) } }
                }
                if loading {
                    div { class: "oxsfd-wrap", "aria-hidden": "true", div { class: "oxscr-sk", {skel(6)} } }
                } else if all.is_empty() {
                    {empty("No deals yet", "Add the first with New deal: it starts as a lead you own, and every cell edits in place.")}
                } else if rows.is_empty() {
                    p { class: "oxscn-hint", role: "status", if qs.trim().is_empty() { "Nothing in this view yet." } else { "No deal matches “{qs}”." } }
                } else {
                    div { class: "oxsfd-wrap oxscr-wrap",
                        table { class: "oxsfd-tb oxscr-tb", "aria-label": "Deals",
                            thead { tr {
                                {th(DealSort::Company, "Company", false)}
                                {th(DealSort::Stage, "Stage", false)}
                                {th(DealSort::Value, "Value", true)}
                                th { scope: "col", class: "oxscr-own", "Owner" }
                                {th(DealSort::Close, "Close", false)}
                            } }
                            tbody {
                                for d in rows {
                                    {
                                        let id = d.id;
                                        let sel = pick() == Some(id);
                                        let ed = edit();
                                        let ed_co = ed == Some((id, "company"));
                                        let ed_v = ed == Some((id, "value"));
                                        let co = d.company.clone();
                                        let co2 = d.company.clone();
                                        let val = d.value;
                                        let late = d.close < 0 && d.stage != "won" && d.stage != "lost";
                                        let st_items = stages.clone();
                                        rsx! {
                                            tr { key: "{id}", "data-on": "{sel}", "data-stage": "{d.stage}",
                                                td { class: "oxscr-co",
                                                    if ed_co {
                                                        input { class: "oxscr-in", "aria-label": "Company", value: "{draft}", placeholder: "Company name",
                                                            onmounted: move |e: MountedEvent| { let m = e.data(); spawn(async move { let _ = m.set_focus(true).await; }); },
                                                            oninput: move |e| draft.set(e.value()),
                                                            onblur: move |_| commit(),
                                                            onkeydown: move |e: KeyboardEvent| { if e.key() == Key::Enter { commit(); } else if e.key() == Key::Escape { edit.set(None); } } }
                                                    } else {
                                                        button { r#type: "button", class: "oxscr-cell", "aria-label": "{co}, open", onclick: move |_| pick.set(Some(id)),
                                                            ondoubleclick: move |_| start(id, "company", co.clone()),
                                                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::F2 { e.prevent_default(); start(id, "company", co2.clone()); },
                                                            span { class: "oxscn-row-t", "{d.company}" }
                                                            if !d.contact.is_empty() { span { class: "oxscr-sub", "{d.contact}" } }
                                                        }
                                                    }
                                                }
                                                td { class: "oxscr-st",
                                                    MenuButton { label: stage_label(d.stage).to_string(), aria: "Stage of {d.company}", class: "oxscr-stb".to_string(), items: st_items, checked: d.stage.to_string(),
                                                        on_pick: move |k: String| {
                                                            if let Some(s) = stage_key(&k) && let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.stage = s; }
                                                            said.set(format!("Moved to {}", stage_label(&k)));
                                                        } }
                                                }
                                                td { class: "oxscr-num",
                                                    if ed_v {
                                                        input { class: "oxscr-in oxscr-vin", "aria-label": "Value", "aria-invalid": "{bad}", "data-bad": "{bad}", inputmode: "decimal", value: "{draft}",
                                                            onmounted: move |e: MountedEvent| { let m = e.data(); spawn(async move { let _ = m.set_focus(true).await; }); },
                                                            oninput: move |e| { draft.set(e.value()); bad.set(false); },
                                                            onblur: move |_| { if *bad.peek() { edit.set(None); bad.set(false); } else { commit(); } },
                                                            onkeydown: move |e: KeyboardEvent| { if e.key() == Key::Enter { commit(); } else if e.key() == Key::Escape { bad.set(false); edit.set(None); } } }
                                                    } else {
                                                        button { r#type: "button", class: "oxscr-cell oxscr-val", "aria-label": "Value {usd(val)}, edit",
                                                            onclick: move |_| start(id, "value", val.to_string()), "{usd(val)}" }
                                                    }
                                                }
                                                td { class: "oxscr-own",
                                                    if d.owner.is_empty() { span { class: "oxscr-sub", "–" } } else { span { class: "oxscn-av oxsfo-face", title: "{member_name(d.owner)}", "{d.owner}" } }
                                                }
                                                td { class: "oxscn-row-a oxscr-close", "data-late": "{late}", "{close_words(d.close, d.stage)}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    p { class: "oxsfd-total oxscn-row-a", "{deal_view(&all, cur, &qs, by, rev).len()} shown · {usd(shown_total)} · double-click a company or press F2 to rename" }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}

/// The open deal beside the table: company, contact, stage and owner menus, value, close,
/// a stage-by-stage pipeline, notes, Mark won and Delete.
#[component]
fn CrmDealDetail(deal: Deal, list: Signal<Vec<Deal>>, on_gone: EventHandler<String>) -> Element {
    let id = deal.id;
    let stages: Vec<MenuItem> = CRM_STAGES.iter().map(|(k, l)| (*k, *l, "")).collect();
    let people: Vec<MenuItem> = vec![
        ("none", "Nobody", ""),
        ("RV", member_name("RV"), ""),
        ("JO", member_name("JO"), ""),
        ("TA", member_name("TA"), ""),
        ("SE", member_name("SE"), ""),
    ];
    let own_k = if deal.owner.is_empty() {
        "none".to_string()
    } else {
        deal.owner.to_string()
    };
    let own_l = if deal.owner.is_empty() {
        "Nobody".to_string()
    } else {
        member_name(deal.owner).to_string()
    };
    let all = list();
    let totals = stage_totals(&all);
    let top = totals.iter().map(|t| t.1).max().unwrap_or(0).max(1);
    let name = deal.company.clone();
    let won = deal.stage == "won";
    rsx! {
        div { class: "oxsfi oxscr-det",
            p { class: "oxscr-kick", "{stage_label(deal.stage)} · {usd(deal.value)}" }
            h2 { class: "oxscr-h2", "{deal.company}" }
            dl { class: "oxskb-dl",
                div { dt { "Contact" } dd { if deal.contact.is_empty() { "Not set" } else { "{deal.contact}" } } }
                div { dt { "Stage" } dd { MenuButton { label: stage_label(deal.stage).to_string(), aria: "Stage", class: "oxscr-dst".to_string(), items: stages, checked: deal.stage.to_string(), right: true,
                    on_pick: move |k: String| { if let Some(s) = stage_key(&k) && let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.stage = s; } } } } }
                div { dt { "Owner" } dd { MenuButton { label: own_l, aria: "Owner", class: "oxscr-down".to_string(), items: people, checked: own_k, right: true,
                    on_pick: move |k: String| {
                        let w: &'static str = match k.as_str() { "RV" => "RV", "JO" => "JO", "TA" => "TA", "SE" => "SE", _ => "" };
                        if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.owner = w; }
                    } } } }
                div { dt { "Close" } dd { "{close_words(deal.close, deal.stage)}" } }
            }
            if !deal.note.is_empty() {
                p { class: "oxscn-gh", "Notes" }
                p { class: "oxsfi-about", "{deal.note}" }
            }
            p { class: "oxscn-gh", "Pipeline" }
            ul { class: "oxscr-pipe", "aria-label": "Value by stage",
                for (k, v, n) in totals {
                    li { key: "{k}", "data-on": "{k == deal.stage}",
                        span { class: "oxscr-pl", "{stage_label(k)}" }
                        span { class: "oxscr-bar", span { style: "width:{v * 100 / top}%" } }
                        span { class: "oxscn-row-a oxscr-pv", "{usd(v)} · {n}" }
                    }
                }
            }
            div { class: "oxsin-acts",
                button { r#type: "button", class: "oxscn-btn", disabled: won,
                    onclick: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.stage = "won"; } },
                    if won { "Won" } else { "Mark won" } }
                button { r#type: "button", class: "oxscn-btn oxsfd-del",
                    onclick: move |_| { list.write().retain(|x| x.id != id); on_gone.call(name.clone()); }, "Delete deal" }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── Workflow builder ──

/// Workflow builder: a palette of nodes, a canvas you drag them around on (or move with the arrow
/// keys), connections drawn as curves, an inspector for the selected node, and a test run that
/// lights each step up in turn.
#[component]
pub fn WorkflowScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock (drives the test run).
    now: ClockMs,
) -> Element {
    use oxyn_core::parity::screensfid::{
        WF_GRID, WF_NODE_H, WF_NODE_W, WF_PALETTE, WfRun, wf_add, wf_canvas, wf_clamp, wf_finished,
        wf_fit, wf_kind_label, wf_order, wf_outcome, wf_path, wf_place, wf_remove, wf_states,
        wf_zoom, workflow,
    };
    use std::rc::Rc;
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let seed = workflow(st == ScreenState::Sample);
    let mut nodes = use_signal(|| seed.0.clone());
    let mut edges = use_signal(|| seed.1.clone());
    let mut sel = use_signal(move || {
        if st == ScreenState::Sample {
            Some(5u32)
        } else {
            None
        }
    });
    let mut sel_edge = use_signal(|| None::<(u32, u32)>);
    let mut zoom = use_signal(|| 100u32);
    let mut pal = use_signal(|| true);
    let mut pal_n = use_signal(|| false);
    let mut run_at = use_signal(|| None::<u64>);
    let mut published = use_signal(|| false);
    let mut said = use_signal(String::new);
    // (node id, pointer x0, y0, node x0, y0, moved)
    let mut grab = use_signal(|| None::<(u32, f64, f64, i32, i32, bool)>);
    let mut pal_drag = use_signal(|| None::<(&'static str, &'static str, &'static str)>);
    let mut cv = use_signal(|| None::<Rc<MountedData>>);
    let mut refocus = use_signal(|| None::<u32>);
    let loading = st == ScreenState::Loading;
    let ns = nodes();
    let es = edges();
    let z = zoom();
    let tick = run_at().map(|t| (since(now, t) / 900) as usize);
    let states = wf_states(&ns, &es, tick);
    let state_of = |id: u32| {
        states
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, s)| *s)
            .unwrap_or(WfRun::Idle)
    };
    let running = tick.is_some_and(|t| !wf_finished(&ns, &es, t));
    let outcome = tick.map(|t| wf_outcome(&ns, &es, t));
    let ready_n = ns.iter().filter(|n| n.ready()).count();
    let all_ready = !ns.is_empty() && ready_n == ns.len();
    let (cw, ch) = wf_canvas(&ns);
    let plane = format!(
        "width:{cw}px;height:{ch}px;transform:scale({})",
        f64::from(z) / 100.0
    );
    let wrap = format!(
        "width:{}px;height:{}px",
        cw * z as i32 / 100,
        ch * z as i32 / 100
    );
    let title_of = move |id: u32| {
        nodes
            .peek()
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.title.clone())
            .unwrap_or_default()
    };
    let mut add = move |kind: &'static str, title: &'static str, ask: &'static str| {
        let after = *sel.peek();
        let id = wf_add(
            &mut nodes.write(),
            &mut edges.write(),
            kind,
            title,
            ask,
            after,
        );
        sel.set(Some(id));
        sel_edge.set(None);
        refocus.set(Some(id));
        run_at.set(None);
        published.set(false);
        said.set(match after {
            Some(a) => format!("{title} added after {}", title_of(a)),
            None => format!("{title} added"),
        });
    };
    let fit = move || {
        if let Some(r) = cv.peek().clone() {
            spawn(async move {
                if let Ok(rc) = r.get_client_rect().await {
                    let f = wf_fit(&nodes.peek(), rc.size.width as i32, rc.size.height as i32);
                    zoom.set(f);
                    said.set(format!("Zoom {f}%"));
                }
            });
        }
    };
    let items: Vec<MenuItem> = WF_PALETTE
        .iter()
        .flat_map(|(_, _, xs)| xs.iter().map(|(t, _)| (*t, *t, "")))
        .collect();
    let find_pal = |t: &str| {
        WF_PALETTE
            .iter()
            .find_map(|(_, k, xs)| xs.iter().find(|(x, _)| *x == t).map(|(x, a)| (*k, *x, *a)))
    };
    let picked = sel().and_then(|id| ns.iter().find(|n| n.id == id).cloned());
    let order = wf_order(&ns, &es);
    let work = if loading {
        Some(rsx! { div { class: "oxswf-insp", {skel(6)} } })
    } else if let Some(n) = picked {
        Some(
            rsx! { WorkflowInspector { key: "{n.id}", node: n.clone(), nodes, edges, run: state_of(n.id).key().to_string(),
            on_gone: move |t: String| { sel.set(None); run_at.set(None); said.set(format!("{t} removed")); },
            on_said: move |t: String| said.set(t) } },
        )
    } else if let Some((a, b)) = sel_edge() {
        let (ta, tb) = (title_of(a), title_of(b));
        Some(rsx! {
            div { class: "oxswf-insp",
                p { class: "oxswf-kind", "Connection" }
                h2 { class: "oxswf-h2", "{ta} → {tb}" }
                p { class: "oxscn-hint oxswf-pl", "Runs “{tb}” after “{ta}” finishes." }
                div { class: "oxsin-acts",
                    button { r#type: "button", class: "oxscn-btn oxsfd-del",
                        onclick: move |_| { edges.write().retain(|e| *e != (a, b)); sel_edge.set(None); run_at.set(None); said.set(format!("Connection from {ta} to {tb} removed")); }, "Remove connection" }
                }
            }
        })
    } else {
        Some(rsx! {
            div { class: "oxswf-insp",
                p { class: "oxswf-kind", "Workflow" }
                h2 { class: "oxswf-h2", "Member welcome" }
                dl { class: "oxskb-dl",
                    div { dt { "Steps" } dd { "{ns.len()}" } }
                    div { dt { "Ready" } dd { "{ready_n} of {ns.len()}" } }
                    div { dt { "Last test" } dd { if let Some(o) = outcome.clone() { "{o}" } else { "Not run yet" } } }
                }
                if order.is_empty() {
                    p { class: "oxscn-hint oxswf-pl", "Add a trigger, then the steps after it. Select a step to set it up." }
                } else {
                    p { class: "oxscn-gh", "Run order" }
                    ol { class: "oxswf-log",
                        for id in order.iter().copied() {
                            {
                                let s = state_of(id);
                                let t = title_of(id);
                                rsx! { li { key: "{id}", "data-run": "{s.key()}", span { class: "oxswf-dot" } span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-row-a", "{s.label()}" } } }
                            }
                        }
                    }
                }
            }
        })
    };
    let pal_open = pal();
    let pal_nar = pal_n();
    let actions = rsx! {
        if published() { span { class: "oxscn-tag", "data-tone": "ok", "Published" } }
        button { r#type: "button", class: "oxscn-btn oxswf-pub", disabled: !all_ready || loading, title: if all_ready { "Publish this version" } else { "Every step needs its setting first" },
            onclick: move |_| { published.set(true); said.set("Published".to_string()); }, "Publish" }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Flows / Member welcome", actions, work, work_label: "Inspector",
            div { class: "oxswf", "data-pal": "{pal_open}",
                div { class: "oxswf-bar", role: "toolbar", "aria-label": "Canvas",
                    button { r#type: "button", class: "oxscn-ib oxswf-pt-w", "aria-label": if pal_open { "Hide the node palette" } else { "Show the node palette" }, "aria-pressed": "{pal_open}",
                        onclick: move |_| pal.toggle(), {ico("side")} }
                    button { r#type: "button", class: "oxscn-ib oxswf-pt-n", "aria-label": if pal_nar { "Hide the node palette" } else { "Show the node palette" }, "aria-pressed": "{pal_nar}",
                        onclick: move |_| pal_n.toggle(), {ico("side")} }
                    MenuButton { label: "Add node", aria: "Add node", icon: "plus", items,
                        on_pick: move |k: String| { if let Some((kind, t, a)) = find_pal(&k) { add(kind, t, a); } } }
                    span { class: "oxscn-grow" }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Fit to view", title: "Fit to view", disabled: loading, onclick: move |_| fit(), {ico("panel")} }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Zoom out", disabled: z <= 50, onclick: move |_| zoom.set(wf_zoom(z, -1)), "−" }
                    button { r#type: "button", class: "oxscn-btn oxswf-z", "aria-label": "Zoom {z}%, reset to 100%", onclick: move |_| zoom.set(100), "{z}%" }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Zoom in", disabled: z >= 150, onclick: move |_| zoom.set(wf_zoom(z, 1)), "+" }
                    if running {
                        button { r#type: "button", class: "oxscn-primary oxswf-run", onclick: move |_| { run_at.set(None); said.set("Test run stopped".to_string()); }, {ico("stop")} span { "Stop" } }
                    } else {
                        button { r#type: "button", class: "oxscn-primary oxswf-run", disabled: order.is_empty() || loading,
                            onclick: move |_| { run_at.set(Some(now.0)); sel_edge.set(None); said.set("Test run started".to_string()); }, {ico("runs")} span { class: "oxswf-rl-w", if tick.is_some() { "Run again" } else { "Run workflow" } } span { class: "oxswf-rl-n", "Run" } }
                    }
                }
                if let Some(o) = outcome.clone() {
                    p { class: "oxswf-out", role: "status", "data-fail": "{o.starts_with(\"Stopped\")}", "{o}" }
                }
                div { class: "oxswf-main",
                    if pal_open || pal_nar {
                        aside { class: "oxswf-pal", "aria-label": "Node palette", "data-w": "{pal_open}", "data-n": "{pal_nar}",
                            div { class: "oxswf-pal-h", span { "Add nodes" }
                                button { r#type: "button", class: "oxscn-ib", "aria-label": "Close the palette", onclick: move |_| { pal.set(false); pal_n.set(false); }, {ico("close")} } }
                            if loading { {skel(8)} } else {
                                p { class: "oxswf-pal-hint", "Click to add after the selected step, or drag onto the canvas." }
                                for (g, kind, xs) in WF_PALETTE {
                                    div { key: "{g}", class: "oxswf-grp",
                                        p { class: "oxscn-gh", "{g} " span { class: "oxswf-n", "{xs.len()}" } }
                                        for (t, a) in xs.iter().copied() {
                                            button { key: "{t}", r#type: "button", class: "oxswf-pi", draggable: "true", "data-kind": "{kind}",
                                                ondragstart: move |_| pal_drag.set(Some((kind, t, a))),
                                                ondragend: move |_| pal_drag.set(None),
                                                onclick: move |_| { pal_n.set(false); add(kind, t, a); },
                                                span { class: "oxswf-chip", "data-kind": "{kind}" } span { class: "oxscn-row-t", "{t}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxswf-cv", tabindex: "-1", "aria-label": "Workflow canvas",
                        onmounted: move |e: MountedEvent| {
                            let d = e.data();
                            cv.set(Some(d.clone()));
                            // open fitted, so the whole flow shows in a narrow frame
                            spawn(async move {
                                if let Ok(rc) = d.get_client_rect().await && rc.size.width > 0.0 {
                                    let f = wf_fit(&nodes.peek(), rc.size.width as i32, rc.size.height as i32);
                                    zoom.set(f.clamp(67, 100));
                                }
                            });
                        },
                        ondragover: move |e: DragEvent| e.prevent_default(),
                        ondrop: move |e: DragEvent| {
                            e.prevent_default();
                            let Some((kind, t, a)) = *pal_drag.peek() else { return };
                            pal_drag.set(None);
                            let p = e.client_coordinates();
                            let zz = f64::from(*zoom.peek()) / 100.0 * stage_scale();
                            if let Some(r) = cv.peek().clone() {
                                spawn(async move {
                                    let rc = r.get_client_rect().await.ok();
                                    let so = r.get_scroll_offset().await.ok();
                                    let (ox, oy) = rc.map(|rc| (rc.origin.x, rc.origin.y)).unwrap_or((0.0, 0.0));
                                    let (sx, sy) = so.map(|s| (s.x, s.y)).unwrap_or((0.0, 0.0));
                                    let x = ((p.x - ox + sx) / zz) as i32 - WF_NODE_W / 2;
                                    let y = ((p.y - oy + sy) / zz) as i32 - WF_NODE_H / 2;
                                    let id = wf_add(&mut nodes.write(), &mut edges.write(), kind, t, a, None);
                                    let _ = wf_place(&mut nodes.write(), id, x, y);
                                    sel.set(Some(id)); sel_edge.set(None); run_at.set(None); published.set(false);
                                    said.set(format!("{t} added"));
                                });
                            }
                        },
                        onpointermove: move |e: PointerEvent| {
                            if let Some((id, x0, y0, nx, ny, _)) = grab() {
                                let zz = f64::from(*zoom.peek()) / 100.0 * stage_scale();
                                let p = e.client_coordinates();
                                let (dx, dy) = ((p.x - x0) / zz, (p.y - y0) / zz);
                                if dx.abs() + dy.abs() > 3.0 {
                                    let (x, y) = wf_clamp(nx + dx as i32, ny + dy as i32);
                                    if let Some(n) = nodes.write().iter_mut().find(|n| n.id == id) { n.x = x; n.y = y; }
                                    grab.set(Some((id, x0, y0, nx, ny, true)));
                                }
                            }
                        },
                        onpointerup: move |_| {
                            if let Some((id, _, _, _, _, moved)) = grab() {
                                if moved {
                                    let (x, y) = nodes.peek().iter().find(|n| n.id == id).map(|n| (n.x, n.y)).unwrap_or((0, 0));
                                    let _ = wf_place(&mut nodes.write(), id, x, y);
                                    said.set(format!("{} moved", title_of(id)));
                                }
                                grab.set(None);
                            }
                        },
                        onpointerleave: move |_| grab.set(None),
                        if loading {
                            div { class: "oxswf-skel", "aria-hidden": "true", for i in 0..4 { span { key: "{i}", class: "oxscn-shim oxswf-sk" } } }
                        } else {
                            div { class: "oxswf-wrap", style: "{wrap}",
                                div { class: "oxswf-plane", style: "{plane}",
                                    onclick: move |_| { sel.set(None); sel_edge.set(None); },
                                    svg { class: "oxswf-edges", width: "{cw}", height: "{ch}", view_box: "0 0 {cw} {ch}", "aria-label": "Connections",
                                        for (a, b) in es.iter().copied() {
                                            {
                                                let na = ns.iter().find(|n| n.id == a).cloned();
                                                let nb = ns.iter().find(|n| n.id == b).cloned();
                                                match (na, nb) {
                                                    (Some(na), Some(nb)) => {
                                                        let d = wf_path(&na, &nb);
                                                        let on = sel_edge() == Some((a, b));
                                                        let lit = matches!(state_of(a), WfRun::Done) && !matches!(state_of(b), WfRun::Idle | WfRun::Queued | WfRun::Skipped);
                                                        let lbl = format!("Connection from {} to {}", na.title, nb.title);
                                                        rsx! {
                                                            g { key: "{a}-{b}", class: "oxswf-edge", "data-on": "{on}", "data-lit": "{lit}", tabindex: "0", role: "button", "aria-label": "{lbl}", "aria-pressed": "{on}",
                                                                onclick: move |e: MouseEvent| { e.stop_propagation(); sel.set(None); sel_edge.set(Some((a, b))); },
                                                                onkeydown: move |e: KeyboardEvent| {
                                                                    let k = e.key();
                                                                    if k == Key::Enter || k == Key::Character(" ".into()) { e.prevent_default(); sel.set(None); sel_edge.set(Some((a, b))); }
                                                                    if k == Key::Delete || k == Key::Backspace { e.prevent_default(); edges.write().retain(|x| *x != (a, b)); sel_edge.set(None); run_at.set(None); said.set("Connection removed".to_string()); }
                                                                },
                                                                path { class: "oxswf-hit", d: "{d}" }
                                                                path { class: "oxswf-line", d: "{d}" }
                                                            }
                                                        }
                                                    }
                                                    _ => rsx! {},
                                                }
                                            }
                                        }
                                    }
                                    for n in ns.iter().cloned() {
                                        {
                                            let id = n.id;
                                            let on = sel() == Some(id);
                                            let rs = state_of(id);
                                            let ready = n.ready();
                                            let pos = format!("left:{}px;top:{}px;width:{WF_NODE_W}px;height:{WF_NODE_H}px", n.x, n.y);
                                            let kl = wf_kind_label(n.kind);
                                            let dragging = grab().is_some_and(|g| g.0 == id && g.5);
                                            let summary = if ready { n.value.clone() } else { format!("{} not set", n.ask) };
                                            let lbl = format!("{kl}: {}, {}{}", n.title, if ready { "ready" } else { "not configured" }, if rs.label().is_empty() { String::new() } else { format!(", {}", rs.label().to_lowercase()) });
                                            rsx! {
                                                div { key: "{id}", class: "oxswf-node", style: "{pos}", "data-kind": "{n.kind}", "data-sel": "{on}", "data-run": "{rs.key()}", "data-ready": "{ready}", "data-drag": "{dragging}",
                                                    tabindex: "0", role: "button", "aria-pressed": "{on}", "aria-label": "{lbl}", "aria-keyshortcuts": "ArrowLeft ArrowRight ArrowUp ArrowDown Delete",
                                                    onmounted: move |e: MountedEvent| { if *refocus.peek() == Some(id) { refocus.set(None); let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); } },
                                                    onclick: move |e: MouseEvent| { e.stop_propagation(); },
                                                    onpointerdown: move |e: PointerEvent| {
                                                        e.stop_propagation();
                                                        let p = e.client_coordinates();
                                                        let (nx, ny) = nodes.peek().iter().find(|n| n.id == id).map(|n| (n.x, n.y)).unwrap_or((0, 0));
                                                        sel.set(Some(id)); sel_edge.set(None);
                                                        grab.set(Some((id, p.x, p.y, nx, ny, false)));
                                                    },
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        let k = e.key();
                                                        let step = if e.modifiers().shift() { WF_GRID * 5 } else { WF_GRID };
                                                        let (dx, dy) = match k { Key::ArrowLeft => (-step, 0), Key::ArrowRight => (step, 0), Key::ArrowUp => (0, -step), Key::ArrowDown => (0, step), _ => (0, 0) };
                                                        if dx != 0 || dy != 0 {
                                                            e.prevent_default();
                                                            let (x, y) = nodes.peek().iter().find(|n| n.id == id).map(|n| (n.x, n.y)).unwrap_or((0, 0));
                                                            let _ = wf_place(&mut nodes.write(), id, x + dx, y + dy);
                                                            let (x, y) = nodes.peek().iter().find(|n| n.id == id).map(|n| (n.x, n.y)).unwrap_or((0, 0));
                                                            sel.set(Some(id));
                                                            said.set(format!("{} at {x}, {y}", title_of(id)));
                                                        } else if k == Key::Enter || k == Key::Character(" ".into()) {
                                                            e.prevent_default(); sel.set(Some(id)); sel_edge.set(None);
                                                        } else if k == Key::Delete || k == Key::Backspace {
                                                            e.prevent_default();
                                                            let t = title_of(id);
                                                            wf_remove(&mut nodes.write(), &mut edges.write(), id);
                                                            sel.set(None); run_at.set(None);
                                                            said.set(format!("{t} removed"));
                                                        } else if k == Key::Escape {
                                                            sel.set(None);
                                                        }
                                                    },
                                                    div { class: "oxswf-nt",
                                                        span { class: "oxswf-chip", "data-kind": "{n.kind}" }
                                                        span { class: "oxswf-kl", "{kl}" }
                                                        span { class: "oxscn-grow" }
                                                        if !rs.label().is_empty() {
                                                            span { class: "oxswf-rs", "data-run": "{rs.key()}", "{rs.label()}" }
                                                        } else if !ready {
                                                            span { class: "oxswf-rs", "data-run": "unset", "Not configured" }
                                                        }
                                                    }
                                                    p { class: "oxswf-title", "{n.title}" }
                                                    p { class: "oxswf-sum", "data-unset": "{!ready}", "{summary}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if ns.is_empty() {
                                div { class: "oxswf-blank",
                                    {empty("Start with a trigger", "Pick New member, a schedule or a webhook from the palette; each step you add after it joins on.")}
                                    div { class: "oxswf-blank-a",
                                        button { r#type: "button", class: "oxscn-primary", onclick: move |_| add("trigger", "New member", "Which list"), {ico("plus")} span { "Add New member" } }
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
}

/// The inspector beside the canvas for one node: its title and setting, ready state, where it
/// comes from and goes to (remove, or connect to another step) and Delete.
#[component]
fn WorkflowInspector(
    node: oxyn_core::parity::screensfid::WfNode,
    nodes: Signal<Vec<oxyn_core::parity::screensfid::WfNode>>,
    edges: Signal<Vec<(u32, u32)>>,
    run: String,
    on_gone: EventHandler<String>,
    on_said: EventHandler<String>,
) -> Element {
    use oxyn_core::parity::screensfid::{wf_connect, wf_kind_label, wf_remove};
    let id = node.id;
    let ready = node.ready();
    let all = nodes();
    let es = edges();
    let name = |i: u32| {
        all.iter()
            .find(|n| n.id == i)
            .map(|n| n.title.clone())
            .unwrap_or_default()
    };
    let ins: Vec<(u32, String)> = es
        .iter()
        .filter(|(_, b)| *b == id)
        .map(|(a, _)| (*a, name(*a)))
        .collect();
    let outs: Vec<(u32, String)> = es
        .iter()
        .filter(|(a, _)| *a == id)
        .map(|(_, b)| (*b, name(*b)))
        .collect();
    let targets: Vec<(u32, String)> = all
        .iter()
        .filter(|n| n.id != id && n.kind != "trigger" && !es.contains(&(id, n.id)))
        .map(|n| (n.id, n.title.clone()))
        .collect();
    let mut connecting = use_signal(|| false);
    let kl = wf_kind_label(node.kind);
    let title = node.title.clone();
    let run_l = match run.as_str() {
        "running" => "Running",
        "done" => "Done",
        "failed" => "Failed: needs its setting",
        "skipped" => "Skipped",
        "queued" => "Queued",
        _ => "",
    };
    rsx! {
        div { class: "oxswf-insp",
            p { class: "oxswf-kind", span { class: "oxswf-chip", "data-kind": "{node.kind}" } "{kl}" }
            input { class: "oxskb-title", "aria-label": "Step name", value: "{node.title}",
                oninput: move |e| { if let Some(n) = nodes.write().iter_mut().find(|n| n.id == id) { n.title = e.value(); } } }
            div { class: "oxswf-st",
                if ready { span { class: "oxscn-tag", "data-tone": "ok", "Ready" } } else { span { class: "oxscn-tag", "data-tone": "warn", "Not configured" } }
                if !run_l.is_empty() { span { class: "oxswf-rs", "data-run": "{run}", "{run_l}" } }
            }
            label { class: "oxswf-field",
                span { class: "oxswf-fl", "{node.ask}" }
                input { class: "oxskb-in", value: "{node.value}", placeholder: "Needed to run", "aria-invalid": "{!ready}",
                    oninput: move |e| { if let Some(n) = nodes.write().iter_mut().find(|n| n.id == id) { n.value = e.value(); } } }
            }
            dl { class: "oxskb-dl",
                div { dt { "Position" } dd { "{node.x}, {node.y}" } }
            }
            p { class: "oxscn-gh", "Comes from" }
            if ins.is_empty() { p { class: "oxscn-hint", if node.kind == "trigger" { "Starts the workflow" } else { "Nothing yet: this step will not run" } } }
            ul { class: "oxswf-links",
                for (a, t) in ins {
                    li { key: "{a}", span { class: "oxscn-row-t", "{t}" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Disconnect {t}", onclick: move |_| { edges.write().retain(|e| *e != (a, id)); on_said.call("Connection removed".to_string()); }, {ico("close")} } }
                }
            }
            p { class: "oxscn-gh", "Goes to" }
            if outs.is_empty() { p { class: "oxscn-hint", "Nothing: the workflow ends here" } }
            ul { class: "oxswf-links",
                for (b, t) in outs {
                    li { key: "{b}", span { class: "oxscn-row-t", "{t}" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Disconnect {t}", onclick: move |_| { edges.write().retain(|e| *e != (id, b)); on_said.call("Connection removed".to_string()); }, {ico("close")} } }
                }
            }
            if !targets.is_empty() {
                div { class: "oxscn-mb oxswf-con",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { connecting.set(false); },
                    button { r#type: "button", class: "oxscn-btn", "aria-haspopup": "menu", "aria-expanded": "{connecting}", onclick: move |_| connecting.toggle(), {ico("plus")} span { "Connect to…" } }
                    if connecting() {
                        div { class: "oxscn-menu", role: "menu", "aria-label": "Connect to",
                            for (b, t) in targets {
                                button { key: "{b}", r#type: "button", class: "oxscn-mi", role: "menuitem",
                                    onclick: move |_| {
                                        connecting.set(false);
                                        let ok = wf_connect(&nodes.peek(), &mut edges.write(), id, b);
                                        on_said.call(if ok { format!("Connected to {}", t) } else { format!("Can't connect to {}: that would make a loop", t) });
                                    },
                                    span { class: "oxscn-mi-l", "{t}" } }
                            }
                        }
                    }
                }
            }
            div { class: "oxsin-acts",
                button { r#type: "button", class: "oxscn-btn oxsfd-del",
                    onclick: move |_| { wf_remove(&mut nodes.write(), &mut edges.write(), id); on_gone.call(title.clone()); }, "Delete step" }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────── Settings shell ──

/// The settings shell: its own sidebar (Back to app, search that finds any setting, Personal and
/// Workspace pages, the account) in the same Inset or Flat frame, and a centred settings column of
/// switches, menus, fields with checks and counters, actions, and destructive actions confirmed inline.
/// Edits collect in a save bar (Discard / Save changes).
#[component]
pub fn SettingsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The page to open (`notifications` when unknown).
    #[props(default = "notifications".to_string())]
    page: String,
    /// Back to app pressed.
    #[props(default)]
    on_back: EventHandler<()>,
) -> Element {
    use oxyn_core::parity::screensfid::{set_changes, set_find, set_valid, settings};
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st != ScreenState::Empty;
    let mut saved = use_signal(move || settings(sample));
    let mut cur = use_signal(move || settings(sample));
    let start = if settings(true).iter().any(|p| p.key == page) {
        page.clone()
    } else {
        "notifications".to_string()
    };
    let mut open = use_signal(move || start.clone());
    let q = use_signal(String::new);
    let mut said = use_signal(String::new);
    let acct = use_signal(|| false);
    let loading = st == ScreenState::Loading;
    let pages = cur();
    let qs = q();
    let hits = set_find(&pages, &qs);
    let searching = !qs.trim().is_empty();
    let changes = set_changes(&saved(), &pages);
    let valid = set_valid(&pages);
    let pg = pages
        .iter()
        .find(|p| p.key == open())
        .cloned()
        .unwrap_or_else(|| pages[0].clone());
    let title = format!("Settings / {}", pg.label);
    let first_hit = hits.first().map(|(p, _)| *p);
    let side = rsx! {
        SettingsNav { pages: pages.clone(), open, q, hits: hits.clone(), acct, loading,
            on_back: move |_| { said.set("Back to the app".to_string()); on_back.call(()); } }
    };
    let n_hits = hits.len();
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title, side_el: side,
            div { class: "oxsst",
                if loading {
                    div { class: "oxsst-col", {skel(3)} div { class: "oxsst-card", {skel(6)} } div { class: "oxsst-card", {skel(4)} } }
                } else {
                    div { class: "oxsst-col",
                        header { class: "oxsst-head",
                            h1 { class: "oxscn-h1", "{pg.label}" }
                            p { class: "oxscn-lead", "{pg.lead}" }
                            if searching {
                                p { class: "oxsst-found", role: "status",
                                    if n_hits == 0 { "No setting matches “{qs}”." } else if n_hits == 1 { "1 setting matches “{qs}”." } else { "{n_hits} settings match “{qs}”." }
                                    if let Some(f) = first_hit { if f != pg.key { button { r#type: "button", class: "oxscn-link", onclick: move |_| open.set(f.to_string()), "Go to the first" } } }
                                }
                            }
                        }
                        if st == ScreenState::Empty {
                            p { class: "oxsst-new", "A new workspace: nothing is set yet. Fill in what you need; the defaults cover the rest." }
                        }
                        for (sec, rows) in pg.sections.iter().cloned() {
                            section { key: "{sec}", class: "oxsst-sec", "data-danger": "{sec == \"Danger zone\"}",
                                h2 { class: "oxsst-h2", "{sec}" }
                                div { class: "oxsst-card",
                                    for r in rows {
                                        {
                                            let hit = hits.iter().any(|(_, k)| *k == r.key);
                                            rsx! { SettingsRow { key: "{r.key}", row: r.clone(), cur, saved, hit, on_said: move |t: String| said.set(t) } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if changes > 0 {
                        div { class: "oxsst-save", role: "region", "aria-label": "Unsaved changes",
                            span { class: "oxsst-save-t", if changes == 1 { "1 unsaved change" } else { "{changes} unsaved changes" } }
                            if !valid { span { class: "oxsst-save-e", "Fix the marked field first" } }
                            span { class: "oxscn-grow" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { let s = saved.peek().clone(); cur.set(s); said.set("Changes discarded".to_string()); }, "Discard" }
                            button { r#type: "button", class: "oxscn-primary", disabled: !valid,
                                onclick: move |_| { let c = cur.peek().clone(); saved.set(c); said.set(if changes == 1 { "1 change saved".to_string() } else { format!("{changes} changes saved") }); }, "Save changes" }
                        }
                    }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}

/// The settings sidebar: Back to app, a search over every setting, the two groups of pages (with
/// how many settings match on each while searching) and the account menu.
#[component]
fn SettingsNav(
    pages: Vec<oxyn_core::parity::screensfid::SetPage>,
    open: Signal<String>,
    q: Signal<String>,
    hits: Vec<(&'static str, &'static str)>,
    acct: Signal<bool>,
    loading: bool,
    on_back: EventHandler<()>,
) -> Element {
    use oxyn_core::parity::screensfid::SET_GROUPS;
    let mut open = open;
    let mut q = q;
    let mut acct = acct;
    let mut peek = try_use_context::<SidePeek>().map(|s| s.0);
    let searching = !q().trim().is_empty();
    let first = hits.first().map(|(p, _)| *p);
    let ac = acct();
    rsx! {
        div { class: "oxsst-top",
            button { r#type: "button", class: "oxscn-btn oxsst-back", onclick: move |_| on_back.call(()), {ico("back")} span { "Back to app" } }
        }
        p { class: "oxsst-st", "Settings" }
        label { class: "oxscn-search",
            {ico("search")}
            input { r#type: "search", placeholder: "Search settings", "aria-label": "Search settings", value: "{q}",
                oninput: move |e| q.set(e.value()),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Escape { q.set(String::new()); }
                    if e.key() == Key::Enter && let Some(f) = first { open.set(f.to_string()); if let Some(p) = peek.as_mut() { p.set(false); } }
                } }
        }
        div { class: "oxscn-lists oxsst-nav",
            if loading { {skel(8)} } else {
                for (g, gl) in SET_GROUPS {
                    p { key: "{g}", class: "oxscn-gh", "{gl}" }
                    for pg in pages.iter().filter(|p| p.group == g) {
                        {
                            let k = pg.key;
                            let n = hits.iter().filter(|(p, _)| *p == k).count();
                            let on = open() == k;
                            let dim = searching && n == 0;
                            rsx! {
                                button { key: "{k}", r#type: "button", class: "oxscn-nav-b oxsst-nb", "data-on": "{on}", "data-dim": "{dim}", "aria-current": if on { "page" } else { "false" },
                                    onclick: move |_| { open.set(k.to_string()); if let Some(p) = peek.as_mut() { p.set(false); } },
                                    {ico(pg.icon)} span { class: "oxscn-lbl", "{pg.label}" }
                                    if searching && n > 0 { span { class: "oxscn-badge", "aria-label": "{n} matches", "{n}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
        div { class: "oxscn-foot",
            div { class: "oxscn-acct",
                button { r#type: "button", class: "oxscn-acct-b", "aria-haspopup": "menu", "aria-expanded": "{ac}", "aria-label": "Account, {PERSON}", onclick: move |_| acct.toggle(),
                    span { class: "oxscn-av", "{PERSON_INI}" } span { class: "oxscn-lbl", "{PERSON}" } }
                if ac {
                    div { class: "oxscn-menu oxscn-acct-menu", role: "menu", "aria-label": "Account",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { acct.set(false); },
                        for (i, l) in ["Switch workspace", "Keyboard shortcuts", "Sign out"].iter().enumerate() {
                            button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| acct.set(false), span { class: "oxscn-mi-l", "{l}" } }
                        }
                    }
                }
            }
        }
    }
}

/// One settings row: label and hint, then its control. Switches, menus and fields edit the
/// working copy; a changed field is checked (and counted against its cap); an action reports
/// what it did; a danger action asks first (Delete workspace wants the name typed).
#[component]
fn SettingsRow(
    row: oxyn_core::parity::screensfid::SetRow,
    cur: Signal<Vec<oxyn_core::parity::screensfid::SetPage>>,
    saved: Signal<Vec<oxyn_core::parity::screensfid::SetPage>>,
    hit: bool,
    on_said: EventHandler<String>,
) -> Element {
    use oxyn_core::parity::screensfid::{
        SetCtl, set_enabled, set_needs_name, set_row, set_row_mut, set_text_error,
    };
    let key = row.key;
    let mut asking = use_signal(|| false);
    let mut typed = use_signal(String::new);
    let mut done = use_signal(|| false);
    let on = set_enabled(&cur(), &row);
    let was = set_row(&saved(), key).map(|(_, r)| r.ctl.clone());
    let dirty = was.as_ref() != Some(&row.ctl);
    let fid = format!("oxsst-{key}");
    let label = row.label;
    let ctl = match row.ctl.clone() {
        SetCtl::Switch(v) => rsx! {
            button { r#type: "button", class: "oxscn-switch", role: "switch", id: "{fid}", "aria-checked": "{v}", "aria-label": "{label}", disabled: !on,
                onclick: move |_| { if let Some(r) = set_row_mut(&mut cur.write(), key) { r.ctl = SetCtl::Switch(!v); } }, span {} }
        },
        SetCtl::Choice(opts, i) => {
            let items: Vec<MenuItem> = opts.iter().map(|o| (*o, *o, "")).collect();
            let cur_l = opts.get(i).copied().unwrap_or("");
            if on {
                rsx! { MenuButton { label: cur_l.to_string(), aria: "{label}: {cur_l}", items, checked: cur_l.to_string(), right: true, class: "oxsst-mb".to_string(),
                on_pick: move |k: String| { if let Some(j) = opts.iter().position(|o| *o == k) && let Some(r) = set_row_mut(&mut cur.write(), key) { r.ctl = SetCtl::Choice(opts, j); } } } }
            } else {
                rsx! { button { r#type: "button", class: "oxscn-btn oxsst-mb", disabled: true, "aria-label": "{label}: {cur_l}", span { "{cur_l}" } } }
            }
        }
        SetCtl::Text(v, cap) => {
            let err = if dirty { set_text_error(key, &v) } else { None };
            let n = v.chars().count();
            let over = cap > 0 && n > cap;
            let long = cap > 120;
            let bad = err.is_some() || over;
            rsx! {
                div { class: "oxsst-tf", "data-long": "{long}",
                    if long {
                        textarea { class: "oxsst-in oxsst-ta", id: "{fid}", rows: "3", value: "{v}", disabled: !on, "aria-invalid": "{bad}", placeholder: "Write a few lines",
                            oninput: move |e| { if let Some(r) = set_row_mut(&mut cur.write(), key) { r.ctl = SetCtl::Text(e.value(), cap); } } }
                    } else {
                        input { class: "oxsst-in", id: "{fid}", value: "{v}", disabled: !on, "aria-invalid": "{bad}", placeholder: "Not set",
                            oninput: move |e| { if let Some(r) = set_row_mut(&mut cur.write(), key) { r.ctl = SetCtl::Text(e.value(), cap); } } }
                    }
                    div { class: "oxsst-tfm",
                        if let Some(m) = err { span { class: "oxsst-err", "{m}" } }
                        if cap > 0 && (long || over) { span { class: "oxsst-cnt", "data-over": "{over}", "{n}/{cap}" } }
                    }
                }
            }
        }
        SetCtl::Action(l) => rsx! {
            button { r#type: "button", class: "oxscn-btn", disabled: !on,
                onclick: move |_| { done.set(true); on_said.call(match key { "export" => "Export started: the link arrives by email".to_string(), "invite-link" => "Invite link copied".to_string(), _ => format!("{label}: done") }); },
                if done() && key == "invite-link" { "Copied" } else { "{l}" } }
        },
        SetCtl::Danger(l) => rsx! {
            button { r#type: "button", class: "oxscn-btn oxsst-danger", disabled: !on || done(), "aria-expanded": "{asking}",
                onclick: move |_| { typed.set(String::new()); asking.set(true); }, if done() { "Done" } else { "{l}" } }
        },
    };
    let needs_name = set_needs_name(key);
    let name_ok = typed().trim() == WORKSPACE;
    let verb = match &row.ctl {
        SetCtl::Danger(l) => *l,
        _ => "",
    };
    rsx! {
        div { class: "oxsst-row", "data-hit": "{hit}", "data-off": "{!on}", "data-dirty": "{dirty}",
            div { class: "oxsst-rl",
                label { class: "oxsst-lb", r#for: "{fid}", "{row.label}" }
                if !row.hint.is_empty() { p { class: "oxsst-hint", "{row.hint}" } }
            }
            div { class: "oxsst-rc", {ctl} }
            if asking() {
                div { class: "oxsst-ask", role: "group", "aria-label": "Confirm {verb}",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { asking.set(false); },
                    if needs_name {
                        label { class: "oxsst-fl", "Type {WORKSPACE} to confirm"
                            input { class: "oxsst-in", value: "{typed}", "aria-label": "Workspace name",
                                onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                oninput: move |e| typed.set(e.value()) } }
                    } else {
                        p { class: "oxsst-fl", "This cannot be undone." }
                    }
                    div { class: "oxsin-acts",
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| asking.set(false), "Cancel" }
                        button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go", disabled: needs_name && !name_ok,
                            onmounted: move |e: MountedEvent| { if !needs_name { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); } },
                            onclick: move |_| { asking.set(false); done.set(true); on_said.call(format!("{verb}: done (sample, nothing was removed)")); }, "{verb}" }
                    }
                }
            }
        }
    }
}
