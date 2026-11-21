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
//! Sample, Loading and Empty data. Models: `oxyn_core::kit::screensfid`. Styles:
//! `screensfid.css` (`oxscn-` frame, `oxsrl-` rail frame, `oxcst-` start, `oxcov-` overview,
//! `oxcag-` agents, `oxcdr-` double sidebar, `oxcpr-` progress, `oxcrn-` run, `oxsfo-` / `oxsfd-` /
//! `oxsfi-` folders, `oxsin-` integrations, `oxskb-` kanban, `oxscr-` CRM, `oxswf-` workflow, `oxsst-` settings). Every person, workspace and line of copy is fictional and our own.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::kit::screensfid::{
    EnvMode, PERSON_EMAIL, PERSON_PLAN, SIDE_W, STAGE_H, STAGE_STATES, STAGE_W, WORK_W,
    account_menu, context_card, thousands, usage,
};
use oxyn_core::kit::screensfid::{
    PERSON, PERSON_INI, Review, ScreenState, ShellLayout, Thread, WORKSPACE, ago, can_send,
    chat_title, open_thread, pct_done, projects, setup_steps, split_pinned, threads, toggle_pin,
};

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
        "flask" => &[
            "M9 3h6",
            "M10 3v6l-5 9a2 2 0 0 0 2 3h10a2 2 0 0 0 2-3l-5-9V3",
            "M7 15h10",
        ],
        "db" => &[
            "M5 6c0-1.7 3-3 7-3s7 1.3 7 3-3 3-7 3-7-1.3-7-3z",
            "M5 6v12c0 1.7 3 3 7 3s7-1.3 7-3V6",
            "M5 12c0 1.7 3 3 7 3s7-1.3 7-3",
        ],
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
        "sun" => &[
            "M12 8a4 4 0 1 0 0 8 4 4 0 1 0 0-8",
            "M12 3v2",
            "M12 19v2",
            "M3 12h2",
            "M19 12h2",
            "m5.6 5.6 1.4 1.4",
            "m17 17 1.4 1.4",
            "m5.6 18.4 1.4-1.4",
            "m17 7 1.4-1.4",
        ],
        "kbd" => &[
            "M3 7h18v10H3z",
            "M7 11h.01",
            "M11 11h.01",
            "M15 11h.01",
            "M8 14h8",
        ],
        "download" => &["M12 4v11", "m7 10 5 5 5-5", "M5 20h14"],
        "help" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18",
            "M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .8-1 1.5v.4",
            "M12 17h.01",
        ],
        "logout" => &["M14 4h5v16h-5", "M10 8l-4 4 4 4", "M6 12h10"],
        "laptop" => &["M5 6h14v9H5z", "M3 18h18"],
        "phone" => &[
            "M8 3h8a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H8a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z",
            "M11 18h2",
        ],
        "cloud" => &["M7 18h10a4 4 0 0 0 .5-8A6 6 0 0 0 6 9.5 4.3 4.3 0 0 0 7 18z"],
        "updown" => &["m8 9 4-4 4 4", "m8 15 4 4 4-4"],
        "caret" => &["m7 10 5 5 5-5"],
        "flash" => &["M13 3 5 14h6l-1 7 8-11h-6z"],
        "cal" => &["M4 6h16v14H4z", "M4 10h16", "M8 3v5", "M16 3v5"],
        "hand" => &[
            "M8 13V6.5a1.5 1.5 0 0 1 3 0V12",
            "M11 11V5a1.5 1.5 0 0 1 3 0v6",
            "M14 11V6.5a1.5 1.5 0 0 1 3 0V14a6 6 0 0 1-6 6h-.5a6 6 0 0 1-4.6-2.2L3.6 14.6a1.5 1.5 0 0 1 2.3-1.9L8 15",
        ],
        "target" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18",
            "M12 8a4 4 0 1 0 0 8 4 4 0 1 0 0-8",
            "M12 12h.01",
        ],
        "rec" => &[
            "M12 4a8 8 0 1 0 0 16 8 8 0 1 0 0-16",
            "M12 9a3 3 0 1 0 0 6 3 3 0 1 0 0-6",
        ],
        "pen" => &["M4 20l4-1 11-11-3-3L5 16z", "M14 7l3 3"],
        "plug" => &["M9 3v5", "M15 3v5", "M6 8h12v3a6 6 0 0 1-12 0z", "M12 17v4"],
        "book" => &[
            "M5 4h11a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3z",
            "M5 17a3 3 0 0 1 3-3h11",
        ],
        "doc" => &["M7 3h7l4 4v14H7z", "M14 3v4h4", "M10 12h5", "M10 16h5"],
        "pdf" => &["M7 3h7l4 4v14H7z", "M14 3v4h4", "M9.5 16h5"],
        "sheet" => &["M4 5h16v14H4z", "M4 10h16", "M4 15h16", "M10 5v14"],
        "slides" => &["M4 5h16v11H4z", "M12 16v4", "M8 20h8"],
        "table" => &["M4 5h16v14H4z", "M4 10h16", "M10 10v9"],
        "image" => &["M4 5h16v14H4z", "m4 16 5-5 4 4 2-2 5 5", "M15 9h.01"],
        "scope" => &[
            "m4 15 9-9 4 4-9 9z",
            "m13 6 2-2 4 4-2 2",
            "M8 19l-2 2",
            "M11 21l-3-5",
        ],
        "dots" => &[
            "M6 6h.01",
            "M12 6h.01",
            "M18 6h.01",
            "M6 12h.01",
            "M12 12h.01",
            "M18 12h.01",
            "M6 18h.01",
            "M12 18h.01",
            "M18 18h.01",
        ],
        "people" => &[
            "M9 11a3 3 0 1 0 0-6 3 3 0 1 0 0 6",
            "M3 19a6 6 0 0 1 12 0",
            "M16 5a3 3 0 0 1 0 6",
            "M17 14a5 5 0 0 1 4 5",
        ],
        "mega" => &["M4 10v4h3l7 4V6L7 10z", "M17 9a4 4 0 0 1 0 6"],
        "palette" => &[
            "M12 3a9 9 0 1 0 0 18c1 0 1.5-.8 1.2-1.6-.4-1.1.3-2.4 1.6-2.4H17a4 4 0 0 0 4-4c0-5.5-4-10-9-10",
            "M7.5 11h.01",
            "M10 7.5h.01",
            "M14.5 7.5h.01",
        ],
        "brain" => &[
            "M9 4a3 3 0 0 0-3 3 3 3 0 0 0-2 5 3 3 0 0 0 2 5 3 3 0 0 0 6 1V5a2 2 0 0 0-3-1",
            "M15 4a3 3 0 0 1 3 3 3 3 0 0 1 2 5 3 3 0 0 1-2 5 3 3 0 0 1-6 1",
        ],
        "route" => &[
            "M6 4a2 2 0 1 0 0 4 2 2 0 1 0 0-4",
            "M18 16a2 2 0 1 0 0 4 2 2 0 1 0 0-4",
            "M6 8v3a3 3 0 0 0 3 3h6a3 3 0 0 1 3 3",
        ],
        "insta" => &[
            "M5 5h14v14H5z",
            "M12 9a3 3 0 1 0 0 6 3 3 0 1 0 0-6",
            "M16 8h.01",
        ],
        "video" => &["M4 6h11v12H4z", "m15 10 5-3v10l-5-3"],
        "link" => &[
            "M10 14a4 4 0 0 0 6 0l3-3a4 4 0 0 0-6-6l-1 1",
            "M14 10a4 4 0 0 0-6 0l-3 3a4 4 0 0 0 6 6l1-1",
        ],
        "shirt" => &["M8 4 4 7l2 4 2-1v10h8V10l2 1 2-4-4-3a4 4 0 0 1-8 0"],
        "fx" => &["M9 20c2 0 2-2 3-8s1-8 3-8", "M8 11h7"],
        "redo" => &["M15 7l5 5-5 5", "M20 12H10a5 5 0 0 0 0 10h2"],
        "trash" => &["M5 7h14", "M10 7V4h4v3", "M7 7l1 13h8l1-13"],
        "upload" => &["M12 20V9", "m7 14 5-5 5 5", "M5 4h14"],
        "web" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18",
            "M3 12h18",
            "M12 3c3 3 3 15 0 18",
            "M12 3c-3 3-3 15 0 18",
        ],
        "folderopen" => &["M3 7h7l2 2h8v3", "M3 7v12h15l3-7H6l-3 7"],
        "user" => &[
            "M12 4a4 4 0 1 0 0 8 4 4 0 1 0 0-8",
            "M5 20c1-4 4-6 7-6s6 2 7 6",
        ],
        "layers" => &["m12 4 8 4-8 4-8-4z", "m4 12 8 4 8-4", "m4 16 8 4 8-4"],
        "box" => &["m12 3 8 4v10l-8 4-8-4V7z", "m4 7 8 4 8-4", "M12 11v10"],
        "mail" => &["M4 6h16v12H4z", "m4 7 8 6 8-6"],
        "card" => &["M3 6h18v12H3z", "M3 10h18", "M7 15h4"],
        "bug" => &[
            "M9 8a3 3 0 0 1 6 0",
            "M8 9h8v6a4 4 0 0 1-8 0z",
            "M4 12h4",
            "M16 12h4",
            "M5 18l3-2",
            "M19 18l-3-2",
            "M12 11v8",
        ],
        "building" => &[
            "M5 21V4h9v17",
            "M14 9h5v12",
            "M3 21h18",
            "M8 8h3",
            "M8 12h3",
            "M8 16h3",
        ],
        "dollar" => &[
            "M12 3v18",
            "M16 7c-1-1.5-2.5-2-4-2-2.2 0-4 1.2-4 3s1.8 2.7 4 3 4 1.2 4 3-1.8 3-4 3c-1.6 0-3.2-.6-4.2-2",
        ],
        "cycle" => &[
            "M20 11a8 8 0 0 0-14.5-4.5L4 8",
            "M4 4v4h4",
            "M4 13a8 8 0 0 0 14.5 4.5L20 16",
            "M20 20v-4h-4",
        ],
        "wallet" => &["M4 7h15v12H4z", "M4 7l11-3v3", "M15 13h2"],
        "store" => &[
            "M4 9 5 4h14l1 5",
            "M4 9h16a3 3 0 0 1-5 2 3 3 0 0 1-6 0 3 3 0 0 1-5-2",
            "M5 11v9h14v-9",
            "M10 20v-5h4v5",
        ],
        "bag" => &["M5 8h14l-1 12H6z", "M9 8a3 3 0 0 1 6 0"],
        "star" => &["m12 4 2.4 5 5.4.6-4 3.7 1.1 5.4L12 16l-4.9 2.7 1.1-5.4-4-3.7 5.4-.6z"],
        "share" => &[
            "M9 11a3 3 0 1 0 0-6 3 3 0 1 0 0 6",
            "M3 19c.8-3 3.2-5 6-5s5.2 2 6 5",
            "M17 8h4",
            "M19 6v4",
        ],
        "filter" => &["M4 5h16l-6 7v6l-4 2v-8z"],
        "lock" => &["M6 11h12v9H6z", "M8 11V8a4 4 0 0 1 8 0v3"],
        "tag" => &["M4 4h7l9 9-7 7-9-9z", "M8 8h.01"],
        "text" => &["M4 6h16", "M4 10h10", "M4 14h16", "M4 18h10"],
        "kanban" => &["M4 4h16v16H4z", "M9 4v16", "M15 4v10"],
        "verified" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18",
            "m8.5 12 2.5 2.5 4.5-5",
        ],
        "grip" => &[
            "M9 6h.01",
            "M15 6h.01",
            "M9 12h.01",
            "M15 12h.01",
            "M9 18h.01",
            "M15 18h.01",
        ],
        "trendup" => &["m4 16 5-5 4 4 7-7", "M15 8h5v5"],
        "key" => &[
            "M8 15a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
            "M11 10h9",
            "M17 10v3",
            "M20 10v2",
        ],
        "eye" => &[
            "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z",
            "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
        ],
        "eyeoff" => &[
            "M10.6 5.1A10 10 0 0 1 12 5c6.5 0 10 7 10 7a17 17 0 0 1-2.6 3.4",
            "M6.6 6.6A17 17 0 0 0 2 12s3.5 7 10 7a9.7 9.7 0 0 0 5.4-1.6",
            "M9.9 9.9a3 3 0 0 0 4.2 4.2",
            "m3 3 18 18",
        ],
        "finger" => &[
            "M6 10a6 6 0 0 1 12 0v2",
            "M9 10a3 3 0 0 1 6 0v3a7 7 0 0 1-1.5 4.5",
            "M12 10v4a10 10 0 0 1-2 6",
            "M6 14a12 12 0 0 1-1 4",
            "M18 15a14 14 0 0 1-1 4",
        ],
        "refresh" => &[
            "M20 11a8 8 0 0 0-14.6-4.4L4 8",
            "M4 4v4h4",
            "M4 13a8 8 0 0 0 14.6 4.4L20 16",
            "M20 20v-4h-4",
        ],
        "list" => &[
            "M8 6h12",
            "M8 12h12",
            "M8 18h12",
            "M4 6h.01",
            "M4 12h.01",
            "M4 18h.01",
        ],
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
    /// A brand mark before the label (coloured, official), in place of `icon`.
    #[props(default)]
    brand: String,
    /// A ⌄ after the label.
    #[props(default)]
    caret: bool,
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
                if !brand.is_empty() { BrandIcon { name: brand.clone(), size: 15.0, coloured: true } }
                else if !icon.is_empty() { {ico(&icon)} }
                if !label.is_empty() { span { class: "oxscn-btn-l", "{label}" } }
                if caret { span { class: "oxscn-caret", {nico("caret")} } }
            }
            if is_open {
                div { class: "oxscn-catch", onclick: move |_| open.set(false) }
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

/// A neutral (chrome) glyph: never toned, whatever its name.
pub(crate) fn nico(name: &str) -> Element {
    glyph(name, "n")
}

static COMPOSER_IDS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// The composer's Add (+) menu: two groups (Add, Plugins), arrow keys move the highlight
/// (aria-activedescendant), Enter or a click picks, → opens Attach app's list and ← or Escape goes
/// back. `pick` gets the item key, or `app:<id>` for an app from the second list.
#[component]
fn PlusMenu(
    /// Ids prefix (unique per composer).
    uid: u32,
    /// Apps connected (for the Link apps hint).
    connected: usize,
    /// Picked.
    on_pick: EventHandler<String>,
    /// Close asked (Escape on the first list, or a pick).
    on_close: EventHandler<()>,
) -> Element {
    use oxyn_core::kit::screensfid::{
        COMPOSER_APPS, PLUS_ADD, PLUS_PLUGINS, connected_words, plus_items,
    };
    let mut hi = use_signal(|| 0usize);
    let mut apps = use_signal(|| false);
    let items = plus_items();
    let n = if apps() {
        COMPOSER_APPS.len()
    } else {
        items.len()
    };
    let h = hi().min(n.saturating_sub(1));
    let active = if apps() {
        format!("oxpm{uid}-a{h}")
    } else {
        format!("oxpm{uid}-{h}")
    };
    let conn = connected_words(connected);
    let mut choose = move |i: usize| {
        let it = plus_items()[i];
        if apps() {
            on_pick.call(format!("app:{}", COMPOSER_APPS[i].0));
            on_close.call(());
        } else if it.more {
            apps.set(true);
            hi.set(0);
        } else {
            on_pick.call(it.key.to_string());
            on_close.call(());
        }
    };
    let row = move |i: usize, it: oxyn_core::kit::screensfid::PlusItem, conn: &str| {
        let hint = if it.key == "connect" {
            conn.to_string()
        } else {
            it.hint.to_string()
        };
        let icon = if it.key == "folder" {
            ico("folder")
        } else if matches!(it.key, "docs" | "pdf" | "sheets") {
            ico(it.icon)
        } else {
            nico(it.icon)
        };
        rsx! {
            div { key: "{it.key}", id: "oxpm{uid}-{i}", class: "oxpm-i", role: "menuitem", "data-hi": "{i == h}", "aria-haspopup": if it.more { "menu" } else { "false" },
                onmouseenter: move |_| hi.set(i),
                onclick: move |e: MouseEvent| { e.stop_propagation(); choose(i); },
                {icon}
                span { class: "oxpm-l", "{it.label}" }
                if !hint.is_empty() { span { class: "oxpm-h", "{hint}" } }
                if it.more { span { class: "oxpm-more", {nico("next")} } }
            }
        }
    };
    rsx! {
        div { class: "oxpm", role: "menu", "aria-label": if apps() { "Attach app" } else { "Add" }, tabindex: "-1", "aria-activedescendant": "{active}",
            onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
            onkeydown: move |e: KeyboardEvent| {
                match e.key() {
                    Key::ArrowDown => { e.prevent_default(); hi.set((h + 1) % n.max(1)); }
                    Key::ArrowUp => { e.prevent_default(); hi.set((h + n - 1) % n.max(1)); }
                    Key::Home => { e.prevent_default(); hi.set(0); }
                    Key::End => { e.prevent_default(); hi.set(n - 1); }
                    Key::Enter => { e.prevent_default(); choose(h); }
                    Key::Character(ref c) if c == " " => { e.prevent_default(); choose(h); }
                    Key::ArrowRight if !apps() && plus_items()[h].more => { e.prevent_default(); apps.set(true); hi.set(0); }
                    Key::ArrowLeft if apps() => { e.prevent_default(); apps.set(false); hi.set(1); }
                    Key::Escape => { e.prevent_default(); e.stop_propagation(); if apps() { apps.set(false); hi.set(1); } else { on_close.call(()); } }
                    Key::Tab => on_close.call(()),
                    _ => {}
                }
            },
            if apps() {
                div { class: "oxpm-back", role: "presentation", onclick: move |e: MouseEvent| { e.stop_propagation(); apps.set(false); hi.set(1); }, {nico("back")} span { "Attach app" } }
                for (i, (k, l)) in COMPOSER_APPS.into_iter().enumerate() {
                    div { key: "{k}", id: "oxpm{uid}-a{i}", class: "oxpm-i", role: "menuitem", "data-hi": "{i == h}",
                        onmouseenter: move |_| hi.set(i),
                        onclick: move |e: MouseEvent| { e.stop_propagation(); choose(i); },
                        BrandIcon { name: k.to_string(), size: 15.0, coloured: true }
                        span { class: "oxpm-l", "{l}" }
                        span { class: "oxpm-h", "{oxyn_core::kit::screensfid::attach_label(k)}" }
                    }
                }
            } else {
                for (i, it) in PLUS_ADD.into_iter().enumerate() { {row(i, it, &conn)} }
                div { class: "oxpm-sep", role: "separator" }
                p { class: "oxpm-g", "Plugins" }
                for (j, it) in PLUS_PLUGINS.into_iter().enumerate() { {row(PLUS_ADD.len() + j, it, &conn)} }
            }
        }
    }
}

/// The composer every chat screen uses (affectUI's composer, our build): the field, then on the
/// left the Add (+) menu and the approval chip (Request approval, or Oxyn's two steps beyond it),
/// on the right the model picker (Auto ⌄, coloured official marks), the mic and a round send
/// that stays grey until there is text. With `foot`, a strip under it picks a folder to work in
/// and connects apps. Enter sends (Shift+Enter breaks a line).
#[component]
pub fn ScreenComposer(
    /// Text (controlled).
    text: Signal<String>,
    /// Placeholder.
    #[props(default = "Ask anything, or @ a file or app".to_string())]
    placeholder: String,
    /// Show the model menu.
    #[props(default = true)]
    model: bool,
    /// Busy: Send turns into Stop.
    #[props(default)]
    busy: bool,
    /// The Work in a folder / Link apps strip under the box (Start, Overview).
    #[props(default)]
    foot: bool,
    /// Open with these files attached (the Attached state).
    #[props(default)]
    files_init: Vec<String>,
    /// Open taking dictation (the Dictating state).
    #[props(default)]
    dictating: bool,
    /// Apps on at the start (brand ids).
    #[props(default = vec!["gmail".to_string(), "slack".to_string(), "notion".to_string()])]
    apps_init: Vec<String>,
    /// Sent (the trimmed text).
    on_send: EventHandler<String>,
    /// Stop pressed while busy.
    #[props(default)]
    on_stop: EventHandler<()>,
) -> Element {
    use oxyn_core::kit::screensfid::{
        APPROVAL_MODES, COMPOSER_APPS, MODELS, attach_label, model as model_for,
    };
    let uid = use_hook(|| COMPOSER_IDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let mut text = text;
    let mut ask = use_signal(|| "ask".to_string());
    let mut mdl = use_signal(|| "auto".to_string());
    let mut rec = use_signal(move || dictating);
    let mut files = use_signal(move || files_init.clone());
    let mut plus_open = use_signal(|| false);
    let mut folder = use_signal(String::new);
    let mut apps = use_signal(move || apps_init.clone());
    let mut goal = use_signal(|| false);
    let mut skill = use_signal(|| false);
    let t = text();
    let ok = can_send(&t);
    let mut send = move || {
        let v = text.peek().trim().to_string();
        if can_send(&v) {
            on_send.call(v);
            text.set(String::new());
            files.set(vec![]);
            goal.set(false);
        }
    };
    let ask_k = ask();
    let (_, ask_l, _, ask_i) = APPROVAL_MODES
        .iter()
        .copied()
        .find(|m| m.0 == ask_k)
        .unwrap_or(APPROVAL_MODES[0]);
    let m = model_for(&mdl());
    let recording = rec();
    let po = plus_open();
    let conn = apps();
    let conn_n = conn.len();
    let mut add_app = move |k: String| {
        let mut a = apps.write();
        if !a.contains(&k) {
            a.push(k);
        }
    };
    let pick = move |k: String| match k.as_str() {
        "files" => files.write().push("brief.pdf".to_string()),
        "draw" => files.write().push("sketch.png".to_string()),
        "docs" => files.write().push("new-doc.docx".to_string()),
        "pdf" => files.write().push("form.pdf".to_string()),
        "sheets" => files.write().push("new-sheet.csv".to_string()),
        "folder" => folder.set("autumn".to_string()),
        "goal" => goal.set(true),
        "skill" => skill.set(true),
        "connect" => {
            let next = COMPOSER_APPS
                .iter()
                .find(|(id, _)| !apps.peek().iter().any(|a| a == id))
                .map(|(id, _)| id.to_string());
            if let Some(n) = next {
                add_app(n);
            }
        }
        a if a.starts_with("app:") => {
            let id = a[4..].to_string();
            files.write().push(format!("@{id}"));
            add_app(id);
        }
        _ => {}
    };
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
    let app_items: Vec<MenuItem> = COMPOSER_APPS.iter().map(|(k, l)| (*k, *l, *k)).collect();
    let model_items: Vec<MenuItem> = MODELS.iter().map(|m| (m.key, m.name, m.mark)).collect();
    let ask_items: Vec<MenuItem> = APPROVAL_MODES.iter().map(|m| (m.0, m.1, "")).collect();
    let has_chips = !files().is_empty() || goal() || skill();
    rsx! {
        div { class: "oxscn-cmpw", "data-foot": "{foot}",
        div { class: "oxscn-cmp", "data-busy": "{busy}", "data-rec": "{recording}",
            if has_chips {
                div { class: "oxscn-cmp-files",
                    if skill() {
                        span { class: "oxscn-chip oxscn-chip-rec", span { class: "oxscn-recdot" } "Recording a skill"
                            button { r#type: "button", class: "oxscn-chip-x", "aria-label": "Stop recording", onclick: move |_| skill.set(false), {ico("close")} } }
                    }
                    if goal() {
                        span { class: "oxscn-chip", {nico("target")} "Goal: keep going until it's done"
                            button { r#type: "button", class: "oxscn-chip-x", "aria-label": "Remove the goal", onclick: move |_| goal.set(false), {ico("close")} } }
                    }
                    for (i, f) in files().iter().enumerate() {
                        {
                            let app = f.strip_prefix('@').map(str::to_string);
                            let lbl = match &app { Some(a) => attach_label(a).to_string(), None => f.clone() };
                            rsx! {
                                span { key: "{i}-{f}", class: "oxscn-chip", title: "{lbl}",
                                    if let Some(a) = app { BrandIcon { name: a, size: 14.0, coloured: true } } else { {file_ico(f)} }
                                    span { class: "oxscn-chip-t", "{lbl}" }
                                    button { r#type: "button", class: "oxscn-chip-x", "aria-label": "Remove {lbl}",
                                        onclick: move |_| { files.write().remove(i); }, {ico("close")} }
                                }
                            }
                        }
                    }
                }
            }
            if recording {
                div { class: "oxscn-dict", role: "status",
                    span { class: "oxscn-wave", "aria-hidden": "true", for i in 0..24 { span { key: "{i}", style: "animation-delay:-{(i * 37) % 600}ms" } } }
                    span { class: "oxscn-row-a", "Listening… speak, then press the mic to stop" }
                }
            } else {
                textarea { class: "oxscn-cmp-in", rows: "2", placeholder: "{placeholder}", "aria-label": "Message", value: "{t}",
                    oninput: move |e| text.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send(); },
                }
            }
            div { class: "oxscn-cmp-row",
                div { class: "oxscn-mb", "data-up": "{!foot}",
                    button { r#type: "button", class: "oxscn-ib oxscn-plus", "aria-label": "Add", "aria-haspopup": "menu", "aria-expanded": "{po}",
                        onclick: move |_| plus_open.toggle(), {nico("plus")} }
                    if po {
                        div { class: "oxscn-menu oxpm-pop",
                            PlusMenu { uid, connected: conn_n, on_pick: pick, on_close: move |_| plus_open.set(false) }
                        }
                        div { class: "oxscn-catch", onclick: move |_| plus_open.set(false) }
                    }
                }
                MenuButton { label: ask_l, aria: "Approval: {ask_l}", icon: ask_i, up: !foot, checked: ask_k.clone(), class: "oxscn-appr",
                    items: ask_items, on_pick: move |k| ask.set(k) }
                span { class: "oxscn-grow" }
                if model {
                    MenuButton { label: m.name, aria: "Model: {m.name}", brand: m.mark, caret: true, up: !foot, right: true, checked: mdl(), class: "oxscn-model",
                        items: model_items, on_pick: move |k| mdl.set(k) }
                }
                button { r#type: "button", class: "oxscn-ib", "aria-label": if recording { "Stop dictation" } else { "Dictate" }, "aria-pressed": "{recording}",
                    "data-on": "{recording}", onclick: move |_| rec.toggle(), {nico("mic")} }
                if busy {
                    button { r#type: "button", class: "oxscn-send", "aria-label": "Stop", onclick: move |_| on_stop.call(()), {nico("stop")} }
                } else {
                    button { r#type: "button", class: "oxscn-send", "aria-label": "Send", disabled: !ok, onclick: move |_| send(), {nico("send")} }
                }
            }
        }
        if foot {
            div { class: "oxscn-cfoot",
                MenuButton { label: folder_l, aria: "Work in a folder", icon: "folder", items: fitems, checked: folder(), class: "oxscn-cf-b", on_pick: move |k| folder.set(k) }
                span { class: "oxscn-grow" }
                MenuButton { label: "Link apps", aria: "Link apps, {conn_n} on", right: true, checked: "", class: "oxscn-cf-b", items: app_items,
                    on_pick: move |k: String| { let mut a = apps.write(); if let Some(i) = a.iter().position(|x| *x == k) { a.remove(i); } else { a.push(k); } } }
                span { class: "oxscn-cf-marks", "aria-hidden": "true",
                    for k in conn.iter().take(4) { BrandIcon { key: "{k}", name: k.clone(), size: 14.0, coloured: true } }
                }
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
    /// A workspace persona's nav and group (`oxyn_core::kit::workscr::persona`).
    Domain(&'static str),
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
    foot: Foot,
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
        Side::Domain(k) => oxyn_core::kit::workscr::persona(k).nav.to_vec(),
    };
    let loading = state == ScreenState::Loading;
    let blank = state == ScreenState::Empty;
    let ws = ws_open();
    let ac = acct_open();
    let peek_s = if peek() { "true" } else { "false" };
    let Foot {
        mut env,
        mut gone,
        mut sub,
        mut bell,
    } = foot;
    let env_k = env();
    let gone_v = gone();
    let card = context_card(&gone_v);
    let sub_k = sub();
    let bell_o = bell();
    let use_v = usage();
    let use_pct = use_v.pct();
    let use_w = format!("width:{use_pct}%");
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
                        if matches!(side, Side::Domain(_)) { {nico(i)} } else { {ico(i)} } span { class: "oxscn-lbl", "{l}" }
                        if n > 0 { span { class: "oxscn-badge", "{n}" } }
                    }
                }
            }
            if matches!(side, Side::Chat | Side::Agent) {
                div { class: "oxscn-env", role: "radiogroup", "aria-label": "Where agents work",
                    for m in EnvMode::ALL {
                        button { key: "{m.key()}", r#type: "button", role: "radio", class: "oxscn-env-o", title: "{m.hint()}",
                            "aria-checked": "{env_k == m}", onclick: move |_| env.set(m),
                            {ico(if m == EnvMode::Cloud { "cloud" } else { "laptop" })} span { class: "oxscn-lbl", "{m.label()}" } }
                    }
                }
            }
            div { class: "oxscn-lists",
                if let Side::Domain(pk) = side {
                    {
                        let pe = oxyn_core::kit::workscr::persona(pk);
                        rsx! {
                            p { class: "oxscn-gh", "{pe.group}" }
                            if loading { {skel(4)} } else {
                                div { class: "oxscn-nav",
                                    for (k, l, i) in pe.items.iter().copied() {
                                        button { key: "{k}", r#type: "button", class: "oxscn-nav-b", title: "{l}", "data-on": if k == nav_on { "true" } else { "false" },
                                            "aria-current": if k == nav_on { "page" } else { "false" },
                                            onclick: move |_| { on_nav.call(k.to_string()); peek.set(false); },
                                            {nico(i)} span { class: "oxscn-lbl", "{l}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if side == Side::Agent {
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
                if !is_rail {
                    if let Some((ck, ct, cl)) = card {
                        div { key: "{ck}", class: "oxscn-card", "data-card": "{ck}", role: "status",
                            button { r#type: "button", class: "oxscn-card-x", "aria-label": "Dismiss {ct}", onclick: move |_| gone.write().push(ck), {ico("close")} }
                            if ck == "setup" {
                                button { r#type: "button", class: "oxscn-setup-b", "aria-expanded": "{setup_open}", onclick: move |_| setup_open.toggle(),
                                    span { class: "oxscn-card-t", "{ct}" } span { class: "oxscn-row-a", "{pct_s}" } {chevron(setup_open())} }
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
                            } else {
                                p { class: "oxscn-card-t",
                                    if ck == "running" { span { class: "oxscn-live", "aria-hidden": "true" } }
                                    span { class: "oxscn-row-t", "{ct}" }
                                }
                                p { class: "oxscn-card-l", "{cl}" }
                                button { r#type: "button", class: "oxscn-card-a", onclick: move |_| on_nav.call(if ck == "running" { "runs".to_string() } else { "changelog".to_string() }),
                                    if ck == "running" { "Open runs" } else { "Full changelog" } {ico("next")} }
                            }
                        }
                    }
                }
                div { class: "oxscn-acct",
                    button { r#type: "button", class: "oxscn-acct-b", "aria-haspopup": "menu", "aria-expanded": "{ac}", "aria-label": "Account, {PERSON}, {PERSON_PLAN}",
                        onclick: move |_| { acct_open.toggle(); sub.set(""); bell.set(false); },
                        span { class: "oxscn-av oxscn-av-id", "{PERSON_INI}" }
                        span { class: "oxscn-acct-txt",
                            span { class: "oxscn-acct-n", "{PERSON}" }
                            span { class: "oxscn-acct-p", "{PERSON_PLAN}" }
                        }
                        span { class: "oxscn-acct-ud", {ico("updown")} }
                    }
                    button { r#type: "button", class: "oxscn-ib oxscn-bell", "aria-label": "Notifications, 3 new", "aria-expanded": "{bell_o}", "aria-haspopup": "dialog",
                        onclick: move |_| { bell.toggle(); acct_open.set(false); },
                        {ico("bell")} span { class: "oxscn-bell-n", "aria-hidden": "true", "3" } }
                    if bell_o {
                        div { class: "oxscn-menu oxscn-bell-pop", role: "dialog", "aria-label": "Notifications",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { bell.set(false); },
                            p { class: "oxscn-pop-h", "Notifications" }
                            for (i, (t, w)) in [("Draft agent needs your approval", "2m"), ("Weekly digest is ready", "1h"), ("Shared folder Autumn launch", "3h")].iter().enumerate() {
                                button { key: "{i}", r#type: "button", class: "oxscn-mi oxscn-note", onclick: move |_| bell.set(false),
                                    span { class: "oxscn-dot" } span { class: "oxscn-mi-l", "{t}" } span { class: "oxscn-mi-k", "{w}" } }
                            }
                        }
                    }
                    if ac {
                        div { class: "oxscn-menu oxscn-acct-menu", role: "menu", "aria-label": "Account",
                            onkeydown: move |e: KeyboardEvent| match e.key() { Key::Escape => { if sub.peek().is_empty() { acct_open.set(false); } else { sub.set(""); } } Key::ArrowLeft => sub.set(""), _ => {} },
                            div { class: "oxscn-acct-hd",
                                span { class: "oxscn-av oxscn-av-id oxscn-av-l", "{PERSON_INI}" }
                                span { class: "oxscn-acct-txt",
                                    span { class: "oxscn-acct-n", "{PERSON}" }
                                    span { class: "oxscn-acct-p", title: "{PERSON_EMAIL}", "{PERSON_EMAIL}" }
                                }
                                span { class: "oxscn-plan", "{use_v.plan}" }
                            }
                            for (i, (k, l, g, h)) in account_menu().iter().copied().enumerate() {
                                if k.is_empty() {
                                    div { key: "{i}", class: "oxscn-msep", role: "separator" }
                                } else {
                                    button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", "data-k": "{k}",
                                        "aria-haspopup": if h == "›" { "menu" } else { "false" }, "aria-expanded": if h == "›" { if sub_k == k { "true" } else { "false" } } else { "" },
                                        onmouseenter: move |_| sub.set(if h == "›" { k } else { "" }),
                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::ArrowRight && h == "›" { sub.set(k); },
                                        onclick: move |_| if h == "›" { sub.set(k); } else { acct_open.set(false); sub.set(""); on_nav.call(k.to_string()); },
                                        {ico(g)} span { class: "oxscn-mi-l", "{l}" }
                                        if h == "›" { span { class: "oxscn-mi-k oxscn-mi-sub", {ico("next")} } } else if !h.is_empty() { span { class: "oxscn-mi-k", "{h}" } }
                                    }
                                }
                            }
                            if sub_k == "usage" {
                                div { class: "oxscn-menu oxscn-fly", role: "menu", "aria-label": "Usage", "data-k": "usage",
                                    div { class: "oxscn-fly-hd",
                                        span { class: "oxscn-acct-n", "{use_v.plan} plan" }
                                        span { class: "oxscn-acct-p", "{use_v.renews}" }
                                    }
                                    div { class: "oxscn-meter", "data-near": "{use_v.near()}",
                                        div { class: "oxscn-meter-l", span { "Credits" } span { class: "oxscn-row-a", "{thousands(use_v.used)} of {thousands(use_v.total)} · {use_pct}%" } }
                                        div { class: "oxscn-prog", role: "progressbar", "aria-valuenow": "{use_pct}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "Credits used", span { style: "{use_w}" } }
                                    }
                                    for (i, (l, u, m)) in use_v.quotas.iter().copied().enumerate() {
                                        div { key: "{i}", class: "oxscn-quota", span { class: "oxscn-row-t", "{l}" } span { class: "oxscn-row-a", "{u} / {m}" } }
                                    }
                                    div { class: "oxscn-msep", role: "separator" }
                                    button { r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| { acct_open.set(false); sub.set(""); on_nav.call("usage".to_string()); }, {ico("chart")} span { class: "oxscn-mi-l", "View usage" } }
                                }
                            }
                            if sub_k == "feedback" {
                                div { class: "oxscn-menu oxscn-fly oxscn-fly-b", role: "menu", "aria-label": "Feedback & help", "data-k": "feedback",
                                    for (i, (l, g)) in [("Send feedback", "chat"), ("Help centre", "help"), ("Contact support", "inbox")].iter().copied().enumerate() {
                                        button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| { acct_open.set(false); sub.set(""); }, {ico(g)} span { class: "oxscn-mi-l", "{l}" } }
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

/// The sidebar bottom's own state: Local / Cloud, dismissed context cards, the open account
/// submenu (`usage`, `feedback` or none) and the notifications popover.
#[derive(Clone, Copy)]
pub(crate) struct Foot {
    env: Signal<EnvMode>,
    gone: Signal<Vec<&'static str>>,
    sub: Signal<&'static str>,
    bell: Signal<bool>,
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
    def: u32,
) -> Element {
    rsx! {
        div { class: "oxscn-sep", "data-for": "{which}", role: "separator", tabindex: "0", "aria-orientation": "vertical", "aria-label": "{label}",
            "aria-valuenow": "{val}", "aria-valuemin": "{min}", "aria-valuemax": "{max}",
            title: "Drag to resize, double-click to reset",
            onpointerdown: move |e: PointerEvent| { e.prevent_default(); drag.set(Some((which, e.client_coordinates().x, w()))); },
            ondoubleclick: move |_| w.set(def),
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
    /// The work area's opening width (360 by default; Agents 368, Run 348).
    #[props(default = WORK_W.0)]
    work_w: u32,
    /// A breadcrumb root before the title ("Home", "Runs"), with its glyph.
    #[props(default)]
    crumb: String,
    /// The crumb's glyph.
    #[props(default)]
    crumb_icon: String,
    /// A bare top bar: no hairline, no title (Start, Overview, Progress).
    #[props(default)]
    bare: bool,
    /// Beside the title (a status pill).
    #[props(default)]
    badge: Option<Element>,
    /// A column inside the page, right of the main column and full height (Progress).
    #[props(default)]
    side_col: Option<Element>,
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
    let foot = Foot {
        env: use_signal(|| EnvMode::Local),
        gone: use_signal(Vec::new),
        sub: use_signal(|| ""),
        bell: use_signal(|| false),
    };
    let setup_open = use_signal(|| false);
    let steps = use_signal(|| {
        setup_steps()
            .into_iter()
            .map(|(_, d)| d)
            .collect::<Vec<_>>()
    });
    let open_proj = use_signal(|| "autumn".to_string());
    let sw = use_signal(|| SIDE_W.0);
    let ww = use_signal(move || work_w.clamp(WORK_W.1, WORK_W.2));
    let mut drag = use_signal(|| None::<(&'static str, f64, u32)>);
    let mut wopen = use_signal(move || !work_closed);
    let mut wpeek = use_signal(|| false);
    let has_work = work.is_some();
    let has_col = side_col.is_some();
    let side_k = match side.as_str() {
        "runs" => Side::Runs,
        "agent" => Side::Agent,
        "files" => Side::Domain("files"),
        "board" => Side::Domain("board"),
        "sales" => Side::Domain("sales"),
        "revenue" => Side::Domain("revenue"),
        "fieldwork" => Side::Domain("fieldwork"),
        "evalkit" => Side::Domain("evalkit"),
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
                    if which == "side" { let v = (w0 as f64 + dx).clamp(f64::from(SIDE_W.1), f64::from(SIDE_W.2)); let mut s = sw; s.set(v as u32); }
                    else { let v = (w0 as f64 - dx).clamp(f64::from(WORK_W.1), f64::from(WORK_W.2)); let mut s = ww; s.set(v as u32); }
                }
            },
            onpointerup: move |_| if drag.peek().is_some() { drag.set(None) },
            onpointerleave: move |_| if drag.peek().is_some() { drag.set(None) },
            if let Some(se) = side_el {
                nav { class: "oxscn-side oxsst-side", "aria-label": "Settings", "data-rail": "false", "data-peek": "{peek}", {se} }
            } else {
                {sidebar(side_k, st, threads, active, query, rail, peek, ws_open, acct_open, foot, setup_open, steps, open_proj, on_nav, nav_on)}
            }
            if !is_rail { {separator("Resize sidebar", "side", sw(), SIDE_W.1, SIDE_W.2, sw, drag, 1, SIDE_W.0)} }
            div { class: "oxscn-page",
                main { class: "oxscn-main", "data-col": "{has_col}",
                    div { class: "oxscn-mcol",
                    header { class: "oxscn-top", "data-bare": "{bare}",
                        button { r#type: "button", class: "oxscn-ib oxscn-menu-b", "aria-label": "Open the sidebar", "aria-expanded": "{peek}", onclick: move |_| peek.toggle(), {ico("side")} }
                        if !crumb.is_empty() {
                            nav { class: "oxscn-crumb", "aria-label": "Breadcrumb",
                                if !crumb_icon.is_empty() { {nico(&crumb_icon)} }
                                span { class: "oxscn-crumb-r", "{crumb}" }
                                span { class: "oxscn-crumb-s", "aria-hidden": "true", "/" }
                            }
                        }
                        if !title.is_empty() && !bare { h2 { class: "oxscn-title", "{title}" } }
                        {badge}
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
                    {side_col}
                }
                if wo { {separator("Resize the work area", "work", ww(), WORK_W.1, WORK_W.2, ww, drag, -1, work_w)} }
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
    match oxyn_core::kit::screensfid::open_count(r) {
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
    /// The states the bar offers: Data · Loading · Empty, then the screen's own (key, label).
    #[props(default = STAGE_STATES.to_vec())]
    states: Vec<(&'static str, &'static str)>,
) -> Element {
    let known = states.iter().any(|(k, _)| *k == state);
    let mut st = use_signal(move || {
        if known {
            state.clone()
        } else {
            ScreenState::from_key(&state).key().to_string()
        }
    });
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
                    for (k, l) in states.iter().copied() {
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

/// The three-pane frame (AI Chat / Double Sidebar): a 52 px icon rail on the canvas, then one card
/// holding a 272 px chats column (the All projects switcher, search, New chat, Pinned, Projects
/// with their chats, Chats) beside the page, and the work area as its own card. The column folds
/// away from the rail's Chats icon or the page's sidebar button.
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
    /// The work area's opening width.
    #[props(default = WORK_W.0)]
    work_w: u32,
    /// Open with the chats column shown.
    #[props(default = true)]
    col_init: bool,
    /// A chat inside a project that is open (shown current under its project), as (project key, title).
    #[props(default)]
    open_in: Option<(&'static str, &'static str)>,
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
    let mut col = use_signal(move || col_init);
    let mut cpeek = use_signal(|| false);
    let mut sect = use_signal(|| "chats".to_string());
    let mut proj = use_signal(|| "all".to_string());
    let mut q = use_signal(String::new);
    let mut finding = use_signal(|| false);
    let mut wopen = use_signal(|| true);
    let mut wpeek = use_signal(|| false);
    let mut open_p = use_signal(move || open_in.map(|o| o.0.to_string()).unwrap_or_default());
    let mut in_chat = use_signal(move || open_in.map(|o| o.1.to_string()).unwrap_or_default());
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
    let ic = in_chat();
    let ic_empty = ic.is_empty();
    let row = move |t: Thread| {
        let id = t.id;
        let on = id == cur && ic_empty;
        rsx! {
            div { key: "{id}", class: "oxscn-row", "data-on": "{on}",
                button { r#type: "button", class: "oxscn-row-b", "aria-current": if on { "page" } else { "false" },
                    onclick: move |_| { active.set(id); in_chat.set(String::new()); open_thread(&mut thr.write(), id); },
                    span { class: "oxscn-row-t", "{t.title}" }
                    if t.unread { span { class: "oxscn-udot", "aria-label": "Unread" } }
                }
                button { r#type: "button", class: "oxscn-row-pin", "aria-label": if t.pinned { "Unpin" } else { "Pin" }, "aria-pressed": "{t.pinned}",
                    onclick: move |_| { toggle_pin(&mut thr.write(), id); }, {nico("pin")} }
            }
        }
    };
    let pf = proj();
    let shown: Vec<_> = projects()
        .into_iter()
        .filter(|p| pf == "all" || p.key == pf)
        .collect();
    let style = format!(
        "--oxscn-h:{height}px;--oxscn-ww:{}px",
        work_w.clamp(WORK_W.1, WORK_W.2)
    );
    rsx! {
        div { class: "oxscn oxsrl", "data-layout": "{lay_k}", "data-state": "{st_k}", "data-col": "{co}", "data-cpeek": "{cp}", style: "{style}",
            nav { class: "oxsrl-rail", "aria-label": "Sections",
                span { class: "oxsrl-ws", title: "{WORKSPACE}", "K" }
                for (k, l, i) in [("chats", "Chats", "chat"), ("tasks", "Tasks", "tasks"), ("runs", "Runs", "runs"), ("files", "Files", "folder"), ("apps", "Apps", "plug")] {
                    button { key: "{k}", r#type: "button", class: "oxsrl-rb", "aria-label": "{l}", title: "{l}", "aria-current": if sect() == k { "page" } else { "false" },
                        onclick: move |_| { if k == "chats" && sect() == "chats" { col.toggle(); cpeek.toggle(); } else { col.set(true); cpeek.set(true); } sect.set(k.to_string()); }, {nico(i)} }
                }
                span { class: "oxscn-grow" }
                button { r#type: "button", class: "oxsrl-rb oxsrl-bell", "aria-label": "Notifications, 2 new", title: "Notifications", {nico("bell")} span { class: "oxsrl-bdot" } }
                button { r#type: "button", class: "oxsrl-rb", "aria-label": "Settings", title: "Settings", {nico("gear")} }
                span { class: "oxscn-av oxsrl-av", title: "{PERSON}", "{PERSON_INI}" }
            }
            div { class: "oxsrl-card",
            div { class: "oxsrl-col",
                div { class: "oxsrl-head",
                    MenuButton { label: proj_l, aria: "Project", caret: true, items, checked: proj(), on_pick: move |k| proj.set(k), class: "oxsrl-proj" }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Search chats", "aria-pressed": "{finding}", onclick: move |_| { finding.toggle(); q.set(String::new()); }, {nico("search")} }
                }
                if finding() {
                    label { class: "oxscn-search",
                        {nico("search")}
                        input { r#type: "search", placeholder: "Search chats", "aria-label": "Search chats", value: "{q}", oninput: move |e| q.set(e.value()),
                            onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { finding.set(false); q.set(String::new()); } }
                    }
                }
                button { r#type: "button", class: "oxscn-nav-b", onclick: move |_| { in_chat.set(String::new()); on_new.call(()); }, {nico("pen")} span { "New chat" } }
                div { class: "oxscn-lists",
                    if st == ScreenState::Loading { {skel(7)} }
                    else if list.is_empty() { p { class: "oxscn-hint oxsrl-none", "No projects or chats yet" } }
                    else {
                        if !pinned.is_empty() { p { class: "oxscn-gh", "Pinned" } for t in pinned { {row(t)} } }
                        if ql.is_empty() {
                            p { class: "oxscn-gh", "Projects" }
                            for pr in shown {
                                {
                                    let k = pr.key;
                                    let open = open_p() == k;
                                    rsx! {
                                        div { key: "{k}", class: "oxsrl-pj",
                                            button { r#type: "button", class: "oxscn-row-b oxsrl-pjb", "aria-expanded": "{open}", onclick: move |_| open_p.set(if open { String::new() } else { k.to_string() }),
                                                {ico("folder")} span { class: "oxscn-row-t", "{pr.label}" } }
                                            if open {
                                                div { class: "oxsrl-sub",
                                                    for c in open_in.filter(|o| o.0 == k).map(|o| o.1).into_iter().chain(pr.chats.iter().copied()) {
                                                        { let on = ic == c; rsx! {
                                                            button { key: "{c}", r#type: "button", class: "oxscn-row-b oxsrl-subr", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                                onclick: move |_| in_chat.set(c.to_string()), span { class: "oxscn-row-t", "{c}" } }
                                                        } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        p { class: "oxscn-gh", "Chats" }
                        for t in rest { {row(t)} }
                    }
                }
            }
            div { class: "oxscn-page",
                main { class: "oxscn-main",
                    header { class: "oxscn-top", "data-bare": "true",
                        button { r#type: "button", class: "oxscn-ib oxsrl-colb", "aria-label": if co { "Hide chats" } else { "Show chats" }, "aria-pressed": "{co}", onclick: move |_| col.toggle(), {nico("side")} }
                        button { r#type: "button", class: "oxscn-ib oxscn-menu-b", "aria-label": "Open chats", "aria-expanded": "{cp}", onclick: move |_| cpeek.toggle(), {nico("side")} }
                        span { class: "oxscn-grow" }
                        if has_work {
                            button { r#type: "button", class: "oxscn-ib oxscn-wtog-w", "aria-label": if wo { format!("Hide {wl}") } else { format!("Show {wl}") }, "aria-pressed": "{wo}", onclick: move |_| wopen.toggle(), {nico("panel")} }
                            button { r#type: "button", class: "oxscn-btn oxscn-wtog-n", "aria-expanded": "{wp}", onclick: move |_| wpeek.toggle(), {nico("panel")} span { "{work_label}" } }
                        }
                    }
                    div { class: "oxscn-body", {children} }
                }
            }
            }
            if has_work {
                aside { class: "oxscn-work oxsrl-work", "aria-label": "{work_label}", "data-open": "{wo}", "data-peek": "{wp}",
                    button { r#type: "button", class: "oxscn-ib oxscn-work-x", "aria-label": "Close {work_label}", onclick: move |_| { wpeek.set(false); wopen.set(false); }, {nico("panel")} }
                    {work}
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

/// Screen – AI Chat / Start: a centred 672 px column – a greeting, the composer (Add menu,
/// Request approval, Auto ⌄, mic, send) with its Work in a folder / Link apps strip, and two
/// rows of quiet suggestions that fill the composer. Sending opens a new chat in the sidebar and
/// the conversation takes over the page. States: Data, Loading, Empty, Attached, Dictating.
#[component]
pub fn ChatStartScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading`, `empty`, `attached` or `dictating`.
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
    use oxyn_core::kit::screensfid::start_suggestions;
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let mut active = use_signal(|| 0u32);
    let mut text = use_signal(String::new);
    let mut msgs = use_signal(Vec::<Msg>::new);
    let hi = format!(
        "{}, Rowan. What are we working on?",
        oxyn_core::kit::screensfid::greeting(hour)
    );
    let started = !msgs().is_empty();
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
    let files_init = if state == "attached" {
        vec![
            "brief.pdf".to_string(),
            "budget-by-team.csv".to_string(),
            "@notion".to_string(),
        ]
    } else {
        vec![]
    };
    let dict = state == "dictating";
    let loading = st == ScreenState::Loading;
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title, bare: !started,
            on_nav: move |k: String| if k == "new" { msgs.set(vec![]); active.set(0); },
            if started {
                div { class: "oxcst-chat",
                    {convo(&msgs(), now, 1200)}
                    ScreenComposer { text, on_send: send }
                }
            } else {
                div { class: "oxcst",
                    if loading {
                        span { class: "oxscn-shim oxcst-hi-sk", "aria-hidden": "true" }
                    } else {
                        h1 { class: "oxcst-hi", "{hi}" }
                    }
                    ScreenComposer { text, on_send: send, foot: true, files_init, dictating: dict,
                        placeholder: "Help me think through…".to_string() }
                    if loading {
                        div { class: "oxcst-starters", "aria-busy": "true", for i in 0..7 { span { key: "{i}", class: "oxcst-st oxscn-shim" } } }
                    } else {
                        div { class: "oxcst-starters", role: "group", "aria-label": "Suggestions",
                            for (k, l, i, p) in start_suggestions() {
                                button { key: "{k}", r#type: "button", class: "oxcst-st", onclick: move |_| text.set(p.to_string()), {nico(i)} span { "{l}" } }
                            }
                        }
                        if st == ScreenState::Empty { p { class: "oxscn-hint oxcst-note", "Your first chat will show up in the sidebar." } }
                    }
                }
            }
        }
    }
}

/// Screen – AI Chat / Overview: a centred 880 px column – the mark, the start composer, then the
/// numbers: activity day by day (a heat map you step through with the arrow keys, over the whole
/// year or the last 12 weeks), recent chats, credits used, the files it made and this month
/// against last. States: Data, Loading, Empty, Near limit.
#[component]
pub fn ChatOverviewScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading`, `empty` or `limit`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::kit::screensfid::{
        CREDITS_CAP, OVERVIEW_WEEKS, credits_used, delta, heat, made_files, near_limit,
        overview_counts, overview_line, used_pct,
    };
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let text = use_signal(String::new);
    let msgs = use_signal(Vec::<Msg>::new);
    let mut range = use_signal(|| "all".to_string());
    let weeks = if range() == "12w" { 12 } else { OVERVIEW_WEEKS };
    let mut cell = use_signal(|| usize::MAX);
    let mut more_chats = use_signal(|| false);
    let mut more_files = use_signal(|| false);
    let filled = st == ScreenState::Sample;
    let all = if filled {
        heat(OVERVIEW_WEEKS)
    } else {
        vec![0; OVERVIEW_WEEKS * 7]
    };
    let levels: Vec<u8> = all[(OVERVIEW_WEEKS - weeks) * 7..].to_vec();
    let n = levels.len();
    let c = cell().min(n - 1);
    const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let lv = levels.get(c).copied().unwrap_or(0);
    let cell_said = format!(
        "Week {} of {weeks}, {}: {} chats",
        c / 7 + 1,
        DAYS[c % 7],
        lv as u32 * 3
    );
    let counts = overview_counts(filled);
    let line = overview_line(counts);
    let used = if st == ScreenState::Sample {
        credits_used(&state)
    } else {
        0
    };
    let warn = near_limit(used, CREDITS_CAP);
    let pct = used_pct(used, CREDITS_CAP);
    let used_s = thousands(used);
    let cap_s = thousands(CREDITS_CAP);
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
    let files = made_files();
    let fshow = if more_files() { files.len() } else { 3 };
    let fmore = files.len() - 3;
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let send = move |t: String| push_pair(msgs, now, t);
    let started = !msgs().is_empty();
    let d1 = delta(12);
    let d2 = delta(-3);
    let range_l = if range() == "12w" {
        "Last 12 weeks"
    } else {
        "Feb – Nov"
    };
    let heat_style = format!("grid-template-columns:repeat({weeks},minmax(0,1fr))");
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Overview", bare: true,
            div { class: "oxcov",
                p { class: "oxcov-mark", span { class: "oxscn-mono oxcov-logo", "K" } "Kestrel" }
                ScreenComposer { text, on_send: send, foot: true, placeholder: "Help me think through…".to_string() }
                if started { {convo(&msgs(), now, 1200)} }
                div { class: "oxcov-grid",
                    section { class: "oxcov-card oxcov-w2", "aria-label": "Activity",
                        div { class: "oxcov-h", {nico("clock")} h3 { "Activity" } }
                        if loading { {skel(3)} } else {
                            div { class: "oxcov-heat", role: "grid", tabindex: "0", style: "{heat_style}", "aria-label": "Chats per day, arrow keys move",
                                onkeydown: move |e: KeyboardEvent| {
                                    let d: i64 = match e.key() { Key::ArrowUp => -1, Key::ArrowDown => 1, Key::ArrowLeft => -7, Key::ArrowRight => 7, _ => 0 };
                                    if d != 0 { e.prevent_default(); let base = if cell() == usize::MAX { n as i64 - 1 } else { cell() as i64 }; let v = (base + d).clamp(0, n as i64 - 1); cell.set(v as usize); }
                                },
                                for (i, l) in levels.iter().enumerate() {
                                    { let row = i % 7 + 1; let col = i / 7 + 1; let pos = format!("grid-row:{row};grid-column:{col}"); rsx! {
                                        span { key: "{i}", class: "oxcov-c", style: "{pos}", "data-l": "{l}", "data-on": "{cell() == i}", onclick: move |_| cell.set(i) }
                                    } }
                                }
                            }
                            div { class: "oxcov-foot",
                                span { class: "oxscn-row-t", role: "status",
                                    if blank { "Nothing yet – your first chat lights up today's square." } else if cell() != usize::MAX { "{cell_said}" } else { "{line}" } }
                                MenuButton { label: range_l, aria: "Range: {range_l}", right: true, caret: true, checked: range(), class: "oxcov-range",
                                    items: vec![("all", "Feb – Nov", ""), ("12w", "Last 12 weeks", "")], on_pick: move |k| { range.set(k); cell.set(usize::MAX); } }
                            }
                        }
                    }
                    section { class: "oxcov-card oxcov-w2", "aria-label": "Recent chats",
                        div { class: "oxcov-h", {nico("chat")} h3 { "Recent chats" }
                            if more_n > 0 && !loading {
                                button { r#type: "button", class: "oxcov-more", "aria-expanded": "{more_chats}", onclick: move |_| more_chats.toggle(),
                                    if more_chats() { "Show less" } else { "{more_n} more" } }
                            }
                        }
                        if loading { {skel(3)} } else if recent.is_empty() { p { class: "oxcov-none", "No chats yet. Ask anything above to start one." } } else {
                            for (i, (t, a)) in recent.iter().take(nshow).enumerate() {
                                div { key: "{i}", class: "oxcov-li", span { class: "oxcov-bub", {nico("chat")} } span { class: "oxscn-row-t", title: "{t}", "{t}" } span { class: "oxscn-row-a", "{a}" } }
                            }
                        }
                    }
                    section { class: "oxcov-card", "aria-label": "Usage", "data-warn": "{warn}",
                        div { class: "oxcov-h", {nico("pulse")} h3 { "Usage" } }
                        if loading { {skel(2)} } else {
                            p { class: "oxcov-big", "{used_s}" span { class: "oxcov-of", " / {cap_s} credits" } }
                            div { class: "oxscn-prog oxcov-prog", "data-warn": "{warn}", role: "meter", "aria-valuenow": "{used}", "aria-valuemin": "0", "aria-valuemax": "{CREDITS_CAP}", "aria-label": "Credits used", span { style: "{bar}" } }
                            div { class: "oxcov-reset",
                                span { class: "oxscn-row-t", if warn { "{pct}% used · resets 1 Dec" } else { "Resets 1 Dec" } }
                                if warn { button { r#type: "button", class: "oxcov-up", "Upgrade" } }
                            }
                        }
                    }
                    section { class: "oxcov-card oxcov-w2", "aria-label": "Files made for you",
                        div { class: "oxcov-h", {nico("doc")} h3 { "Files made for you" }
                            if !loading && !blank {
                                button { r#type: "button", class: "oxcov-more", "aria-expanded": "{more_files}", onclick: move |_| more_files.toggle(),
                                    if more_files() { "Show less" } else { "{fmore} more" } }
                            }
                        }
                        if loading { {skel(3)} } else if blank { p { class: "oxcov-none", "Docs and sheets it makes for you land here." } } else {
                            for (i, (f, p, a)) in files.iter().take(fshow).enumerate() {
                                div { key: "{i}", class: "oxcov-li", title: "{f} · {p}", span { class: "oxcov-ft", {file_ico(f)} } span { class: "oxscn-row-t", "{f}" } span { class: "oxscn-row-a", "{a}" } }
                            }
                        }
                    }
                    section { class: "oxcov-card", "aria-label": "This month",
                        div { class: "oxcov-h", {nico("cal")} h3 { "This month" } }
                        if loading { {skel(2)} } else {
                            dl { class: "oxcov-dl",
                                div { dt { "Chats" } dd { if blank { "0" } else { "36" } if !blank { span { class: "oxcov-d", "data-up": "true", "aria-label": "{d1} on last month", "{d1}" } } } }
                                div { dt { "Files made" } dd { if blank { "0" } else { "14" } if !blank { span { class: "oxcov-d", "data-up": "false", "aria-label": "{d2} on last month", "{d2}" } } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Screen – AI Chat / Agents: a chat with the orchestrator and the crew at work ("Worked for…",
/// the Agents working chips, the Orchestrator line over the composer) and, in a 368 px work area,
/// the launch brief – product, selling angle, audience, channels, palette and formats – that turns
/// into ads. States: Data, Loading, Empty, Generating.
#[component]
pub fn ChatAgentsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading`, `empty` or `making`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::kit::screensfid::{AGENTS_WORK_W, ad_estimate, angles, channels, crew};
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 2u32);
    let text = use_signal(String::new);
    let msgs = use_signal(Vec::<Msg>::new);
    let mut who = use_signal(|| "lead".to_string());
    let mut tab = use_signal(|| "brief".to_string());
    let mut angle = use_signal(|| "feel".to_string());
    let mut chans = use_signal(|| channels().into_iter().map(|(_, on)| on).collect::<Vec<_>>());
    let mut fmts = use_signal(|| vec![true, true, false]);
    let mut aud = use_signal(|| vec![true, true, true, false]);
    let making = state == "making";
    let mut gen_at = use_signal(move || {
        if making {
            Some(now.0.saturating_sub(1500))
        } else {
            None
        }
    });
    let on_n = chans().iter().filter(|x| **x).count();
    let f_n = fmts().iter().filter(|x| **x).count();
    let (ads, mins) = ad_estimate(on_n, f_n);
    let made = gen_at().map(|t| ((since(now, t) / 900) as usize).min(ads));
    let gen_s = match made {
        Some(n) if n < ads => format!("Making {} of {ads}…", n + 1),
        Some(_) => format!("{ads} ads ready"),
        None => String::new(),
    };
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let send = move |t: String| push_pair(msgs, now, t);
    let w = who();
    let crew_icon = |k: &str| match k {
        "lead" => "mega",
        "brand" => "target",
        "studio" => "palette",
        "people" => "people",
        _ => "pen",
    };
    let audiences = ["Commuters 25–40", "City", "Mild winters", "Natural fibres"];
    let ads_s = format!("{ads} ads");
    let mins_s = format!("~{mins} min");
    let gen_pct = made
        .map(|n| format!("width:{}%", n * 100 / ads.max(1)))
        .unwrap_or_default();
    let brief = rsx! {
        div { class: "oxcag-brief",
            div { class: "oxscn-tabs oxcag-wtabs", role: "tablist", "aria-label": "Open briefs",
                div { class: "oxscn-tab", "data-on": "true",
                    span { class: "oxcag-tabi", {nico("tasks")} }
                    button { r#type: "button", role: "tab", class: "oxscn-tab-b", "aria-selected": "true", "Overshirt launch" }
                    button { r#type: "button", class: "oxscn-tab-x", "aria-label": "Close Overshirt launch", {ico("close")} }
                }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "New brief", {nico("plus")} }
            }
            div { class: "oxcag-in",
            div { class: "oxscn-seg", role: "tablist", "aria-label": "Brief sections",
                for (k, l) in [("brief", "Brief"), ("assets", "Assets"), ("activity", "Activity")] {
                    button { key: "{k}", r#type: "button", role: "tab", class: "oxscn-seg-b", "aria-selected": "{tab() == k}", onclick: move |_| tab.set(k.to_string()), "{l}" }
                }
            }
            if loading { {skel(9)} }
            else if blank { {empty("No brief yet", "Tell the orchestrator what you are launching; the brief fills in as it learns.")} }
            else if tab() == "brief" {
                div { class: "oxcag-prod",
                    span { class: "oxcag-thumb", "aria-hidden": "true", {nico("shirt")} }
                    div { p { class: "oxcag-pn", "Field Overshirt" } p { class: "oxscn-row-a", "$118 · 640 sold this month" } }
                    span { class: "oxcag-best", "Top seller" }
                }
                p { class: "oxcag-gh", "Selling angle" }
                div { class: "oxcag-card oxcag-angles", role: "radiogroup", "aria-label": "Selling angle",
                    for a in angles() {
                        button { key: "{a.key}", r#type: "button", role: "radio", class: "oxcag-angle", "aria-checked": "{angle() == a.key}", onclick: move |_| angle.set(a.key.to_string()),
                            span { class: "oxcag-radio" }
                            span { class: "oxcag-at", span { class: "oxcag-al", "{a.label}" } span { class: "oxcag-aw", "{a.why}" } }
                            span { class: "oxcag-sig", "data-n": "{a.signal}", "aria-label": "Signal {a.signal} of 3", span {} span {} span {} }
                        }
                    }
                }
                p { class: "oxcag-gh", "Audience" }
                div { class: "oxcag-card oxcag-tags",
                    for (i, l) in audiences.into_iter().enumerate() {
                        { let on = aud().get(i).copied().unwrap_or(false); rsx! {
                            button { key: "{i}", r#type: "button", class: "oxcag-tag", "aria-pressed": "{on}", onclick: move |_| { aud.write()[i] = !on; }, "{l}" }
                        } }
                    }
                }
                p { class: "oxcag-gh", "Channels" }
                div { class: "oxcag-card",
                    for (i, (l, _)) in channels().into_iter().enumerate() {
                        {
                            let on = chans().get(i).copied().unwrap_or(false);
                            let (ic, hint) = match i { 0 => ("insta", "Feed and stories"), 1 => ("video", "Vertical, 15 seconds"), _ => ("link", "Search and partner sites") };
                            rsx! {
                                div { key: "{i}", class: "oxcag-sw",
                                    span { class: "oxcag-swi", {nico(ic)} }
                                    span { class: "oxcag-at", span { class: "oxcag-al", "{l}" } span { class: "oxcag-aw", "{hint}" } }
                                    button { r#type: "button", role: "switch", class: "oxscn-switch", "aria-checked": "{on}", "aria-label": "{l}", onclick: move |_| { chans.write()[i] = !on; }, span {} }
                                }
                            }
                        }
                    }
                }
                div { class: "oxcag-card oxcag-palc",
                    p { class: "oxcag-gh oxcag-gh-in", "Palette" }
                    div { class: "oxcag-pal",
                        // warm-ok: the swatches are the product's colours (content, not chrome)
                        for (i, (n, h)) in [("Oat", "#D9CDB8"), ("Mustard", "#E2A817"), ("Chalk", "#F1EFEA"), ("Night", "#22262B")].into_iter().enumerate() {
                            span { key: "{i}", class: "oxcag-sw2", span { class: "oxcag-swatch", title: "{n}", style: "background:{h}", "aria-label": "{n}" } span { class: "oxcag-swn", "{n}" } }
                        }
                    }
                }
                p { class: "oxcag-gh", "Formats" }
                div { class: "oxcag-tags oxcag-fmts",
                    for (i, l) in ["Square", "Story", "Wide"].into_iter().enumerate() {
                        { let on = fmts().get(i).copied().unwrap_or(false); rsx! {
                            button { key: "{i}", r#type: "button", class: "oxcag-tag", "aria-pressed": "{on}", onclick: move |_| { fmts.write()[i] = !on; }, "{l}" }
                        } }
                    }
                }
            } else if tab() == "assets" {
                if let Some(n) = made {
                    div { class: "oxcag-ads",
                        for i in 0..ads { span { key: "{i}", class: "oxcag-ad", "data-ready": "{i < n}", "aria-label": if i < n { "Ad ready" } else { "Ad waiting" } } }
                    }
                }
                for (i, f) in ["overshirt-front.png", "overshirt-detail.png", "style-guide.pdf"].iter().enumerate() {
                    div { key: "{i}", class: "oxcov-li", span { class: "oxcov-ft", {file_ico(f)} } span { class: "oxscn-row-t", "{f}" } }
                }
            } else {
                ul { class: "oxcag-act",
                    for m in crew() { li { key: "{m.key}", span { class: "oxcag-ad-d", "data-done": "{m.done}" } span { class: "oxscn-row-t", "{m.name}: {m.doing}" } } }
                }
            }
            }
            div { class: "oxcag-foot",
                if gen_s.is_empty() {
                    span { class: "oxcag-pill", "{ads_s}" } span { class: "oxcag-pill", "{mins_s}" }
                } else {
                    div { class: "oxcag-genw", role: "status",
                        span { class: "oxscn-row-t", "{gen_s}" }
                        div { class: "oxscn-prog oxcov-prog", span { style: "{gen_pct}" } }
                    }
                }
                span { class: "oxscn-grow" }
                button { r#type: "button", class: "oxscn-primary oxcag-go", disabled: loading || blank || ads == 0, onclick: move |_| { gen_at.set(Some(now.0)); tab.set("assets".to_string()); }, "Generate ads" }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Ads for the overshirt", crumb: "Home", crumb_icon: "pen",
            work_label: "Brief", work: brief, work_w: AGENTS_WORK_W,
            actions: rsx! { MenuButton { label: "", aria: "Chat options", icon: "more", right: true, items: vec![("rename", "Rename", ""), ("share", "Share", ""), ("archive", "Archive", "")] } },
            div { class: "oxcag",
                if loading { {skel(8)} }
                else if blank { {empty("Start with a goal", "Say what you want made; the orchestrator brings in the agents it needs.")} }
                else {
                    div { class: "oxscn-log oxcag-log",
                        div { class: "oxscn-msg", "data-me": "true", p { "Lead with how it feels" } }
                        p { class: "oxcag-p", "I'll pull your top seller first, then line up the ad frames." }
                        p { class: "oxcag-took", span { class: "oxcag-took-i", {ico("dots")} } "Worked for 3m 38s" }
                        p { class: "oxcag-p", "Your top seller is the Field Overshirt. It's the one product in so far while the shop keeps importing, so read that ranking as early." }
                        p { class: "oxcag-p", "Before the ads go out I need one call I can't make for you, because it shapes every headline: which angle should lead?" }
                        div { class: "oxcag-crew", role: "group", "aria-label": "Agents working",
                            p { class: "oxcag-crew-l", "Agents working" }
                            div { class: "oxcag-row",
                                for m in crew() {
                                    button { key: "{m.key}", r#type: "button", class: "oxcag-ag", "data-done": "{m.done}", "aria-pressed": "{w == m.key}", title: "{m.name}",
                                        onclick: move |_| { let c = who.peek().clone(); who.set(if c == m.key { String::new() } else { m.key.to_string() }); },
                                        {nico(crew_icon(m.key))} span { class: "oxcag-agn", "{m.name}" } }
                                }
                            }
                            if let Some(m) = crew().into_iter().find(|m| m.key == w) {
                                p { class: "oxcag-said", role: "status", "{m.name}: {m.doing} · " if m.done { "done" } else { "working" } }
                            }
                        }
                    }
                    {convo(&msgs(), now, 1200)}
                }
                div { class: "oxcag-orch", span { class: "oxcag-orch-i", {nico("route")} } b { "Orchestrator" } span { class: "oxscn-row-t oxcag-orch-l", "Routes each request" } }
                ScreenComposer { text, on_send: send }
            }
        }
    }
}

/// The sheet editor in the Double sidebar screen's artifacts: toolbar, cell reference, contents
/// and a grid you click or arrow through; typing in Contents edits the cell, Enter moves down.
#[component]
fn SheetEditor() -> Element {
    use oxyn_core::kit::screensfid::{cell_ref, col_name, sheet, sort_sheet};
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
                span { class: "oxcdr-fxl", "aria-hidden": "true", {nico("fx")} }
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
                div { class: "oxcdr-tr oxcdr-hd", span { class: "oxcdr-rh" } for (i, h) in heads.iter().enumerate() { span { key: "{i}", class: "oxcdr-ch", "data-on": "{i == c}", "{h}" } } }
                for (ri, row) in d.iter().enumerate() {
                    div { key: "{ri}", class: "oxcdr-tr", role: "row",
                        span { class: "oxcdr-rh", "data-on": "{ri == r}", "{ri + 1}" }
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

/// Screen – AI Chat / Double Sidebar: a 52 px rail and a 272 px chats column (All projects ⌄), a
/// plain chat whose answers search the web (each folds its tool calls into one line and links the
/// files it made), and the artifacts in a 408 px work area – a sheet you can edit and the sources –
/// as closable tabs. States: Data, Loading, Empty, Chats folded.
#[component]
pub fn ChatRailScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading`, `empty` or `folded`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::kit::screensfid::RAIL_WORK_W;
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let mut active = use_signal(|| 0u32);
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
    let to = tools_open();
    let who = |t: &'static str| rsx! { div { class: "oxcdr-who", span { class: "oxcdr-mark", {glyph("dots", "spark")} } b { "Kestrel" } span { class: "oxscn-row-a", "{t}" } } };
    let work = rsx! {
        div { class: "oxcdr-arts",
            div { class: "oxscn-tabs", role: "tablist", "aria-label": "Artifacts",
                for (i, n) in a.iter().copied().enumerate() {
                    div { key: "{n}", class: "oxscn-tab", "data-on": "{i == ai}",
                        span { class: "oxcag-tabi", {file_ico(n)} }
                        button { r#type: "button", role: "tab", class: "oxscn-tab-b", "aria-selected": "{i == ai}", title: "{n}", onclick: move |_| art.set(i), "{n}" }
                        button { r#type: "button", class: "oxscn-tab-x", "aria-label": "Close {n}", onclick: move |_| { arts.write().remove(i); art.set(0); }, {ico("close")} }
                    }
                }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Open sources", onclick: move |_| open_art("sources.md"), {nico("plus")} }
            }
            if loading { {skel(10)} }
            else if a.is_empty() { {empty("Nothing open", "Files the assistant makes open here as tabs.")} }
            else if cur == "fabric-compare.csv" {
                div { class: "oxcdr-fh", span { class: "oxcdr-ft", {file_ico("x.csv")} } div { p { class: "oxcdr-fn", "fabric-compare.csv" } p { class: "oxscn-row-a", "Just now" } } }
                SheetEditor {}
            }
            else {
                div { class: "oxcdr-fh", span { class: "oxcdr-ft", {file_ico("x.md")} } div { p { class: "oxcdr-fn", "sources.md" } p { class: "oxscn-row-a", "3 pages read" } } }
                ol { class: "oxcdr-src",
                    for (i, (t, d)) in [("Linen weights explained", "textiles.example.org"), ("Hemp twill lead times", "mills.example.com"), ("Tencel care notes", "fibre.example.net")].into_iter().enumerate() {
                        li { key: "{i}", span { class: "oxscn-fav", {nico("web")} } span { class: "oxcdr-st", span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-row-a", "{d}" } } }
                    }
                }
            }
        }
    };
    rsx! {
        RailFrame { layout, state: state.clone(), height, threads: thr, active, work, work_label: "Artifacts", work_w: RAIL_WORK_W,
            col_init: state != "folded", open_in: if st == ScreenState::Sample { Some(("research", "Fabric weights for the overshirt")) } else { None },
            on_new: move |_| { msgs.set(vec![]); active.set(0); },
            div { class: "oxcdr",
                if loading { {skel(8)} }
                else if blank && msgs().is_empty() { {empty("Ask with the web", "Answers search the web, then fold the searches into one line you can open.")} }
                else {
                    div { class: "oxscn-log",
                        if !blank {
                            div { class: "oxscn-msg", "data-me": "true", p { "Compare five fabrics for the overshirt: weight, price per metre and lead time." } }
                            div { class: "oxcdr-turn",
                                {who("7:22 PM")}
                                button { r#type: "button", class: "oxcdr-ran", "aria-expanded": "{to}", onclick: move |_| tools_open.toggle(), span { class: "oxcdr-ok", {check_glyph()} } span { "Ran 3 searches · 4.1s" } {chevron(to)} }
                                if to {
                                    ul { class: "oxcdr-calls", li { "web.search “linen weight gsm” · 1.2s" } li { "web.search “hemp twill lead time” · 1.6s" } li { "web.read 3 pages · 1.3s" } }
                                }
                                p { "Washed linen and hemp twill are the sturdiest; organic cotton is the cheapest and the quickest to arrive. Tencel drapes best but needs the gentlest wash." }
                                p { "For a shirt worn open over a tee I'd go hemp twill at about 240 gsm – it holds its shape, softens with each wash and lands in two weeks. I put all five side by side." }
                                div { class: "oxcdr-chips",
                                    button { r#type: "button", class: "oxcdr-chip", onclick: move |_| open_art("fabric-compare.csv"), {file_ico("fabric-compare.csv")} "fabric-compare.csv" }
                                    button { r#type: "button", class: "oxcdr-chip", onclick: move |_| open_art("sources.md"), {file_ico("sources.md")} "sources.md" }
                                }
                            }
                            div { class: "oxscn-msg", "data-me": "true", p { "Which one holds up best if it's washed every week?" } }
                            div { class: "oxcdr-turn",
                                {who("7:26 PM")}
                                p { class: "oxcdr-ran oxcdr-ran-s", span { class: "oxcdr-ok", {check_glyph()} } span { "Ran 1 search · 1.2s" } }
                                p { "Hemp twill. It loses the least strength over fifty washes and pills less than the cotton blends; linen comes second but creases more each time." }
                            }
                        }
                    }
                    {convo(&msgs(), now, 1200)}
                }
                ScreenComposer { text, on_send: send, placeholder: "Ask anything; it can search the web".to_string() }
            }
        }
    }
}

/// Screen – AI Chat / Progress: a chat (858 px) whose tool calls open into cards, with its working
/// notes in a 300 px column inside the page – the steps ticking off (earlier ones folded), what it
/// made (a preview, then each output with its kind) and what it used this session (uploads, the
/// apps' official marks, web pages, skills). States: Data, Loading, Empty, Working.
#[component]
pub fn ChatProgressScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading`, `empty` or `working`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::kit::screensfid::{
        PROGRESS_FOLDED, progress_outputs, progress_steps, progress_tools, step_state, tool_mark,
    };
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 1u32);
    let text = use_signal(String::new);
    let msgs = use_signal(Vec::<Msg>::new);
    let start = use_signal(|| now.0);
    let mut open_tool = use_signal(|| usize::MAX);
    let mut sec = use_signal(|| vec![true, true, true]);
    let mut col = use_signal(|| true);
    let mut cpeek = use_signal(|| false);
    let mut early = use_signal(|| false);
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let working = state == "working";
    let steps = progress_steps();
    let done = if working {
        (2 + (since(now, start()) / 1600) as usize).min(steps.len())
    } else if st == ScreenState::Sample {
        steps.len()
    } else {
        0
    };
    let live = working && done < steps.len();
    let said = if done >= steps.len() {
        "All steps done".to_string()
    } else {
        format!("Working on: {}", steps[done])
    };
    let send = move |t: String| push_pair(msgs, now, t);
    let s = sec();
    let ot = open_tool();
    let s2 = s.clone();
    let head = move |i: usize, l: &'static str, close: bool| {
        let o = s2[i];
        rsx! {
            div { class: "oxcpr-sh",
                button { r#type: "button", class: "oxcpr-shb", "aria-expanded": "{o}", onclick: move |_| { sec.write()[i] = !o; }, span { "{l}" } {chevron(o)} }
                if close { button { r#type: "button", class: "oxscn-ib oxcpr-x", "aria-label": "Close progress", onclick: move |_| { col.set(false); cpeek.set(false); }, {nico("close")} } }
            }
        }
    };
    let fold = PROGRESS_FOLDED.min(done);
    let ex = early();
    let tools = progress_tools();
    let tool_card = move |i: usize| {
        let (t, d, secs) = tools[i];
        let o = ot == i;
        let run = live && i == 2 && done < 4;
        let secs_s = format!("{secs:.1}s");
        let m = tool_mark(t);
        rsx! {
            div { key: "{i}", class: "oxcpr-tool", "data-open": "{o}",
                button { r#type: "button", class: "oxcpr-tool-b", "aria-expanded": "{o}", onclick: move |_| open_tool.set(if o { usize::MAX } else { i }),
                    span { class: "oxcpr-tm", if m.is_empty() { {nico("people")} } else { BrandIcon { name: m.to_string(), size: 15.0, coloured: true } } }
                    span { class: "oxcpr-tn", "{t}" } span { class: "oxscn-row-t oxcpr-td", "{d}" }
                    if run { span { class: "oxcpr-spin", role: "status", "aria-label": "Running" } } else { span { class: "oxcpr-ok", "aria-label": "Done", {check_glyph()} } span { class: "oxscn-row-a", "{secs_s}" } }
                    {chevron(o)} }
                if o { pre { class: "oxcpr-io", "in: {t}(team: \"design\")\nout: 3 results" } }
            }
        }
    };
    let side = rsx! {
        aside { class: "oxcpr-col", "aria-label": "Progress", "data-open": "{col}", "data-peek": "{cpeek}",
            if loading { {skel(12)} }
            else if blank { {empty("Nothing running", "Steps, outputs and the apps it uses show here while it works.")} }
            else {
                {head(0, "Progress", true)}
                if s[0] {
                    ol { class: "oxcpr-steps",
                        if fold > 0 && !ex {
                            li { class: "oxcpr-early", button { r#type: "button", class: "oxcpr-eb", "aria-expanded": "false", onclick: move |_| early.set(true), "{fold} earlier steps" } }
                        }
                        for (i, l) in steps.iter().enumerate() {
                            if ex || i >= fold {
                                { let k = step_state(i, done).key(); rsx! { li { key: "{i}", "data-s": "{k}", span { class: "oxcpr-tick", if k == "done" { {check_glyph()} } } span { class: "oxcpr-sl", "{l}" } } } }
                            }
                        }
                    }
                    p { class: "oxscn-sr", role: "status", "{said}" }
                }
                div { class: "oxcpr-rule" }
                {head(1, "Outputs", false)}
                if s[1] {
                    div { class: "oxcpr-doc", "aria-label": "Preview of Welcome plan",
                        p { class: "oxcpr-doct", "Welcome plan" }
                        p { class: "oxcpr-docp", "Monday is laptops, accounts and lunch with a buddy; here is how the rest of the week runs." }
                        span { class: "oxcpr-docl" } span { class: "oxcpr-docl", style: "width:76%" } span { class: "oxcpr-docl", style: "width:58%" }
                    }
                    for (i, (n, k, f)) in progress_outputs().into_iter().enumerate() {
                        div { key: "{i}", class: "oxcpr-out", span { class: "oxcpr-oi", {file_ico(f)} } span { class: "oxscn-row-t", title: "{n}", "{n}" } span { class: "oxscn-row-a", "{k}" } }
                    }
                }
                div { class: "oxcpr-rule" }
                {head(2, "Used in this session", false)}
                if s[2] {
                    div { class: "oxcpr-used",
                        div { span { class: "oxcpr-ui", {nico("clip")} } span { class: "oxscn-row-t", "Uploads" } span { class: "oxscn-row-a", "intake-form.pdf +1" } }
                        div { span { class: "oxcpr-ui", {nico("plug")} } span { class: "oxscn-row-t", "Connectors" }
                            span { class: "oxcpr-apps", BrandIcon { name: "notion".to_string(), size: 14.0, coloured: true } BrandIcon { name: "googledrive".to_string(), size: 14.0, coloured: true } BrandIcon { name: "slack".to_string(), size: 14.0, coloured: true } } }
                        div { span { class: "oxcpr-ui", {nico("web")} } span { class: "oxscn-row-t", "Web search" } span { class: "oxscn-row-a", "6 pages" } }
                        div { span { class: "oxcpr-ui", {nico("pen")} } span { class: "oxscn-row-t", "Skills" } span { class: "oxscn-row-a", "onboarding-plan" } }
                    }
                }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, title: "Plan the new starters' week", bare: true, side_col: side,
            actions: rsx! { button { r#type: "button", class: "oxscn-btn oxcpr-pbtn", "data-col": "{col}", "aria-expanded": "{cpeek}", onclick: move |_| { col.set(true); cpeek.toggle(); }, {nico("tasks")} span { "Progress" } } },
            div { class: "oxcpr",
                if loading { {skel(9)} }
                else if blank { {empty("Ask for something that takes a few steps", "You'll see each step and every tool it calls as it goes.")} }
                else {
                    div { class: "oxscn-log oxcpr-log",
                        div { class: "oxscn-msg", "data-me": "true", p { "Three people start Monday. Plan their first week and pair each with a buddy – the intake form is in Drive, the wiki in Notion." } }
                        {tool_card(0)}
                        p { class: "oxcpr-p", "I read the first-week pages and last spring's notes. The weeks that went well all had the same three parts: accounts on day one, one buddy, and a small task shipped by Friday." }
                        {tool_card(1)}
                        div { class: "oxcpr-a",
                            p { class: "oxcpr-ah", "Here is the week now:" }
                            ul { li { "Monday: laptops, accounts and lunch with their buddy" } li { "Tuesday to Thursday: one shadow day per team" } li { "Friday: a small first task, shipped together" } }
                        }
                        div { class: "oxscn-msg", "data-me": "true", p { "Why is the new designer paired with someone from sales?" } }
                        {tool_card(2)}
                        if !live || done >= 4 {
                            p { class: "oxcpr-p", "The intake form lists the designer under the team that hired the role, which is sales this quarter. I matched by the work instead, so the new pair is with the brand designer; the sheet is updated." }
                        }
                    }
                    {convo(&msgs(), now, 1200)}
                }
                ScreenComposer { text, on_send: send, busy: live, placeholder: "Reply…".to_string() }
            }
        }
    }
}

/// Screen – AI Chat / Run: one agent run – the plan ticking through its steps (rename with Enter,
/// move with Alt+arrows, Delete removes), the apps it uses, live time and cost, and the activity;
/// the 348 px work area is Needs your input: a step that waits for your yes, then the draft it
/// wrote with suggestions to accept (Y) or reject (N). States: Data, Loading, Empty, Needs input,
/// Done.
#[component]
pub fn ChatRunScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading`, `empty`, `input` or `done`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    use oxyn_core::kit::screensfid::{
        RUN_WORK_W, clock, money, move_step, next_open, run_cents, run_line, run_marks, run_plan,
        step_state, suggestions, tool_mark,
    };
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut plan = use_signal(run_plan);
    let back = match state.as_str() {
        "input" => 30_000,
        "done" => 60_000,
        _ => 0,
    };
    let start = use_signal(move || now.0.saturating_sub(back));
    let is_done = state == "done";
    let mut stopped = use_signal(|| None::<u32>);
    let mut approved = use_signal(move || {
        if is_done {
            Some((true, 0u32))
        } else {
            None::<(bool, u32)>
        }
    });
    let mut edit = use_signal(|| None::<usize>);
    let mut focus_i = use_signal(|| 0usize);
    let mut rev = use_signal(move || {
        vec![
            if is_done {
                Review::Accepted
            } else {
                Review::Open
            };
            suggestions().len()
        ]
    });
    let mut cur_s = use_signal(|| 0usize);
    let mut act_open = use_signal(|| usize::MAX);
    let mut wtab = use_signal(|| true);
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
    let steps_s = format!("{} / {}", done.min(p.len()), p.len());
    let n = p.len();
    let sug = suggestions();
    let r = rev();
    let open = oxyn_core::kit::screensfid::open_count(&r);
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
    let marks = run_marks(&p);
    let pct = format!("width:{}%", done * 100 / n.max(1));
    let line = run_line(
        if finished {
            "Done"
        } else if waiting {
            "Needs you"
        } else {
            status
        },
        done,
        n,
    );
    let asking = waiting && approved().is_none() && stopped().is_none();
    let open_s = format!("{open}");
    let work = rsx! {
        div { class: "oxcrn-side",
            div { class: "oxscn-tabs oxcag-wtabs", role: "tablist", "aria-label": "Run panels",
                if wtab() {
                    div { class: "oxscn-tab", "data-on": "true",
                        span { class: "oxcag-tabi", {nico("hand")} }
                        button { r#type: "button", role: "tab", class: "oxscn-tab-b", "aria-selected": "true", "Needs your input" }
                        button { r#type: "button", class: "oxscn-tab-x", "aria-label": "Close Needs your input", onclick: move |_| wtab.set(false), {ico("close")} }
                    }
                }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Open Needs your input", onclick: move |_| wtab.set(true), {nico("plus")} }
            }
            div { class: "oxcrn-pane",
            if loading { {skel(10)} }
            else if blank {
                div { class: "oxcrn-none", span { class: "oxcrn-bell", {nico("bell")} } p { class: "oxscn-empty-t", "Nothing needs you yet" } p { class: "oxscn-empty-l", "It stops here and asks first whenever something would go out." } }
            }
            else {
                if asking {
                    div { class: "oxcrn-gate", role: "alert",
                        div { class: "oxcrn-gate-h", span { class: "oxcrn-gate-i", {nico("hand")} } p { class: "oxcrn-gate-t", "Send the member update?" } }
                        p { class: "oxcrn-gate-l", "212 members get it by email. You can still edit the draft below before it goes." }
                        div { class: "oxcrn-gate-m", BrandIcon { name: "gmail".to_string(), size: 14.0, coloured: true } code { "mail.send" } span { class: "oxscn-row-t", "to members@kestrel.example" } }
                        div { class: "oxcrn-gate-b",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| approved.set(Some((false, 0))), "Hold" }
                            button { r#type: "button", class: "oxscn-primary", onclick: move |_| { let l = (since(now, start()) / 1000) as u32 + 7; approved.set(Some((true, l))); }, "Send it" }
                        }
                    }
                } else {
                    div { class: "oxcrn-none",
                        span { class: "oxcrn-bell", {nico("bell")} }
                        p { class: "oxscn-empty-t", match approved() { Some((true, _)) => "You said yes to sending", Some((false, _)) => "On hold", None => "Nothing needs you yet" } }
                        p { class: "oxscn-empty-l", match approved() { Some((true, _)) => "Nothing else waits on you.", Some((false, _)) => "The run waits until you send it.", None => "It stops here and asks first whenever something would go out." } }
                    }
                }
                p { class: "oxcag-gh", "Draft" }
                div { class: "oxcrn-draft",
                    div { class: "oxcrn-dh", span { class: "oxcrn-dhi", {ico("spark")} } span { class: "oxscn-row-t", "Member update – November" } }
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
                        span { class: "oxcrn-cnt", title: "{open_words(&r)}", "{open_s}" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Reject this suggestion (N)", disabled: open == 0, onclick: move |_| decide(false), {nico("close")} }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Accept this suggestion (Y)", disabled: open == 0, onclick: move |_| decide(true), {nico("check")} }
                        span { class: "oxscn-grow" }
                        button { r#type: "button", class: "oxscn-btn", disabled: open == 0, onclick: move |_| { for x in rev.write().iter_mut() { if *x == Review::Open { *x = Review::Rejected; } } }, "Reject all" }
                        button { r#type: "button", class: "oxscn-primary oxcrn-acc", disabled: open == 0, onclick: move |_| { for x in rev.write().iter_mut() { if *x == Review::Open { *x = Review::Accepted; } } }, "Accept all" }
                    }
                }
                p { class: "oxcag-gh", "Files" }
                div { class: "oxcrn-files",
                    for (i, f) in ["member-update.md", "october-sales.csv", "signups-chart.png"].iter().enumerate() { span { key: "{i}", class: "oxcdr-chip", {file_ico(f)} "{f}" } }
                }
            }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: state.clone(), height, threads: thr, active, side: "runs", crumb: "Runs", crumb_icon: "runs", title: "Write the November member update", work, work_label: "Needs your input", work_w: RUN_WORK_W,
            badge: rsx! { if !loading && !blank { span { class: "oxscn-tag oxcrn-pill", "data-tone": "{tone}", "{status}" } } },
            actions: rsx! {
                button { r#type: "button", class: "oxscn-btn", disabled: stopped().is_some() || finished || loading || blank, onclick: move |_| stopped.set(Some(t)), {nico("stop")} span { "Stop run" } }
                MenuButton { label: "", aria: "Run options", icon: "more", right: true, items: vec![("rerun", "Run again", ""), ("copy", "Copy the plan", ""), ("logs", "Open the logs", "")] }
            },
            div { class: "oxcrn",
                if loading { {skel(10)} }
                else if blank { {empty("No runs yet", "Give an agent a goal; its plan, tool calls and the draft it writes show up here.")} }
                else {
                    p { class: "oxcrn-by", span { class: "oxcrn-byi", {nico("bot")} } span { class: "oxscn-row-t", "Member update · started by {PERSON} at 7:48 PM" } }
                    h1 { class: "oxcrn-t", "Write the November member update" }
                    div { class: "oxcrn-bar", role: "progressbar", "aria-valuenow": "{done}", "aria-valuemin": "0", "aria-valuemax": "{n}", "aria-label": "Steps done", span { style: "{pct}" } }
                    div { class: "oxcrn-uses", span { class: "oxscn-row-a", "Uses" }
                        for m in marks.iter() { span { key: "{m}", class: "oxcrn-mk", title: "{m}", BrandIcon { name: m.to_string(), size: 13.0, coloured: true } } }
                    }
                    dl { class: "oxcrn-stats",
                        div { dt { "Steps" } dd { "{steps_s}" } } div { dt { "Tool calls" } dd { "{calls}" } }
                        div { dt { "Time" } dd { "{time_s}" } } div { dt { "Cost" } dd { "{cost_s}" } }
                    }
                    section { class: "oxcrn-card", "aria-label": "Plan",
                        div { class: "oxcrn-ch", {nico("tasks")} h3 { "Plan" } span { class: "oxscn-grow" } span { class: "oxscn-row-a oxcrn-keys", title: "Enter renames · Alt+↑↓ moves · Delete removes", "{n} steps" } }
                        ol { class: "oxcrn-plan", "aria-label": "Plan",
                            for (i, s) in p.iter().enumerate() {
                                {
                                    let k = if i == gate && waiting { "wait" } else { step_state(i, done).key() };
                                    let lbl = match k { "wait" => "Needs you", "done" => "Done", "active" => "Working", _ => "Up next" };
                                    let ed = edit() == Some(i);
                                    let title = s.title.clone();
                                    let secs_s = if k == "done" { format!("{}.0s", s.secs) } else if k == "active" { format!("{}s", t.saturating_sub(acc)) } else { String::new() };
                                    let num = i + 1;
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
                                            span { class: "oxcrn-dot", match k { "done" => rsx! { {check_glyph()} }, "active" => rsx! { span { class: "oxcpr-spin" } }, "wait" => rsx! { {nico("hand")} }, _ => rsx! { "{num}" } } }
                                            span { class: "oxcrn-st",
                                                if ed {
                                                    input { class: "oxcrn-in", "aria-label": "Step name", value: "{title}",
                                                        onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                        oninput: move |e| { plan.write()[i].title = e.value(); },
                                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter || e.key() == Key::Escape { e.prevent_default(); e.stop_propagation(); edit.set(None); },
                                                        onblur: move |_| edit.set(None) }
                                                } else { span { class: "oxcrn-sn", "{title}" } }
                                                span { class: "oxcrn-why", span { class: "oxscn-row-t", "{s.why}" } code { class: "oxcrn-tool", "{s.tool}" } }
                                            }
                                            if k == "wait" { span { class: "oxcrn-lbl", "{lbl}" } } else { span { class: "oxcrn-secs", "{secs_s}" } }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "oxcrn-cf", role: "status", if !finished && !waiting { span { class: "oxcpr-spin" } } span { class: "oxscn-row-t", "{line}" } }
                    }
                    section { class: "oxcrn-card", "aria-label": "Activity",
                        div { class: "oxcrn-ch", {nico("pulse")} h3 { "Activity" } span { class: "oxscn-grow" } span { class: "oxscn-row-a", "{done.min(n)} of {n}" } }
                        for (i, s) in p.iter().enumerate().take(done + usize::from(!finished && !waiting)) {
                            {
                                let o = act_open() == i;
                                let tl = s.title.clone();
                                let tool = s.tool;
                                let m = tool_mark(tool);
                                let doing = i >= done;
                                let num = i + 1;
                                rsx! {
                                    div { key: "{i}", class: "oxcrn-act",
                                        div { class: "oxcrn-ah", span { class: "oxcrn-dot", "data-s": if doing { "active" } else { "done" }, if doing { "{num}" } else { {check_glyph()} } } span { class: "oxscn-row-t", "{tl}" } }
                                        button { r#type: "button", class: "oxcrn-ab", "aria-expanded": "{o}", onclick: move |_| act_open.set(if o { usize::MAX } else { i }),
                                            if !m.is_empty() { BrandIcon { name: m.to_string(), size: 14.0, coloured: true } }
                                            span { if doing { "Using a tool…" } else { "Used 1 tool" } } span { class: "oxscn-row-a", "· {s.secs}.0s" } {chevron(o)} }
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
}

// ───────────────────────────────────────────────────────────── Workflow builder ──

/// The official mark of the app (or model) a workflow step works in, if it has one.
fn wf_brand(title: &str, kind: &str) -> Option<&'static str> {
    match title {
        "Send email" => Some("gmail"),
        "Post a message" => Some("slack"),
        "Save a row" => Some("airtable"),
        _ if kind == "agent" => Some("claude"),
        _ => None,
    }
}

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
    /// Oxyn: each step shows its number in run order.
    #[props(default)]
    step_numbers: bool,
    /// Oxyn: compact cards without the setting line.
    #[props(default)]
    compact: bool,
) -> Element {
    use oxyn_core::kit::screensfid::{
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
    let testing0 = state == "testing";
    let mut run_at = use_signal(move || {
        if testing0 {
            Some(now.0.saturating_sub(1_900))
        } else {
            None
        }
    });
    let mut published = use_signal(|| false);
    let dock_tab = use_signal(|| "payload");
    let mut dock_fold = use_signal(|| false);
    // Earlier test runs (outcome lines), newest first.
    let mut past = use_signal(Vec::<String>::new);
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
    let out1 = outcome.clone();
    let out2 = outcome.clone();
    let actions = rsx! {
        if published() { span { class: "oxscn-tag", "data-tone": "ok", "Published" } }
        button { r#type: "button", class: "oxscn-btn oxswf-pub", disabled: !all_ready || loading, title: if all_ready { "Publish this version" } else { "Every step needs its setting first" },
            onclick: move |_| { published.set(true); said.set("Published".to_string()); }, "Publish" }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, title: "Flows / Member welcome", actions, work, work_label: "Inspector",
            div { class: "oxswf", "data-pal": "{pal_open}", "data-num": "{step_numbers}", "data-compact": "{compact}",
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
                            onclick: move |_| { if let Some(o) = out1.clone() { past.write().insert(0, o); } run_at.set(Some(now.0)); sel_edge.set(None); said.set("Test run started".to_string()); }, {ico("runs")} span { class: "oxswf-rl-w", if tick.is_some() { "Run again" } else { "Run workflow" } } span { class: "oxswf-rl-n", "Run" } }
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
                                                span { class: "oxswf-chip", "data-kind": "{kind}" } span { class: "oxscn-row-t", "{t}" } if let Some(b) = wf_brand(t, kind) { BrandIcon { name: b.to_string(), size: 12.0, coloured: true } } }
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
                                                        if step_numbers { if let Some(i) = order.iter().position(|x| *x == id) { span { class: "oxswf-num", "aria-hidden": "true", "{i + 1}" } } }
                                                        span { class: "oxswf-chip", "data-kind": "{n.kind}" }
                                                        span { class: "oxswf-kl", "{kl}" }
                                                        span { class: "oxscn-grow" }
                                                        if !rs.label().is_empty() {
                                                            span { class: "oxswf-rs", "data-run": "{rs.key()}", "{rs.label()}" }
                                                        } else if !ready {
                                                            span { class: "oxswf-rs", "data-run": "unset", "Not configured" }
                                                        }
                                                    }
                                                    p { class: "oxswf-title", if let Some(b) = wf_brand(&n.title, n.kind) { BrandIcon { name: b.to_string(), size: 13.0, coloured: true } } span { class: "oxswf-tt", "{n.title}" } }
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
                if !loading {
                    {
                        let unset: Vec<String> = ns.iter().filter(|n| !n.ready()).map(|n| format!("{}: {} not set", n.title, n.ask.to_lowercase())).collect();
                        let failed: Vec<String> = states.iter().filter(|(_, s)| *s == WfRun::Failed).filter_map(|(i, _)| ns.iter().find(|n| n.id == *i)).map(|n| format!("{}: failed in the test run", n.title)).collect();
                        let errs: Vec<String> = failed.into_iter().chain(unset).collect();
                        let ne = errs.len();
                        let trig: Vec<(String, bool)> = ns.iter().filter(|n| n.kind == "trigger").map(|n| (n.title.clone(), n.ready())).collect();
                        let tl: Vec<(String, WfRun)> = order.iter().filter_map(|i| ns.iter().find(|n| n.id == *i)).map(|n| (n.title.clone(), state_of(n.id))).collect();
                        let mut runs: Vec<String> = past();
                        if let Some(o) = outcome.clone() { runs.insert(0, if running { "Running now".to_string() } else { o }); }
                        let df = dock_fold();
                        rsx! {
                            section { class: "oxswf-dock", "aria-label": "Run dock", "data-fold": "{df}",
                                div { class: "oxswf-dh",
                                    {super::screens2fid::tabs("Run dock", vec![("payload", "Payload".to_string()), ("triggers", "Triggers".to_string()), ("errors", if ne > 0 { format!("Errors {ne}") } else { "Errors".to_string() }), ("timeline", "Timeline".to_string()), ("runs", "Runs".to_string())], dock_tab)}
                                    span { class: "oxscn-grow" }
                                    if !running {
                                        button { r#type: "button", class: "oxscn-btn oxswf-start", disabled: order.is_empty(),
                                            onclick: move |_| { if let Some(o) = out2.clone() { past.write().insert(0, o); } run_at.set(Some(now.0)); said.set("Test run started with the sample payload".to_string()); },
                                            {ico("runs")} span { class: "oxswf-start-l", "Start test run" } }
                                    }
                                    button { r#type: "button", class: "oxscn-ib", "aria-label": if df { "Unfold the run dock" } else { "Fold the run dock" }, "aria-expanded": "{!df}", onclick: move |_| dock_fold.toggle(), {super::efflistfid::chevron(df)} }
                                }
                                if !df {
                                    div { class: "oxswf-db", role: "tabpanel",
                                        {match dock_tab() {
                                            "payload" => rsx! { pre { class: "oxswf-pre", "{{\n  \"member\": {{ \"name\": \"Ines Marlow\", \"plan\": \"Pro\", \"email\": \"ines@example.com\" }},\n  \"source\": \"sign-up form\"\n}}" } },
                                            "triggers" => rsx! {
                                                ul { class: "oxswf-dl",
                                                    if trig.is_empty() { li { class: "oxscn-hint", "No trigger yet: add one from the palette." } }
                                                    for (t, ok) in trig { li { key: "{t}", span { class: "oxswf-chip", "data-kind": "trigger" } span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-grow" } span { class: "oxswf-rs", "data-run": if ok { "done" } else { "unset" }, if ok { "Ready" } else { "Not configured" } } } }
                                                }
                                            },
                                            "errors" => rsx! {
                                                ul { class: "oxswf-dl",
                                                    if errs.is_empty() { li { class: "oxscn-hint", "No errors: every step is set up." } }
                                                    for e in errs { li { key: "{e}", span { class: "oxswf-err", "!" } span { class: "oxscn-row-t", "{e}" } } }
                                                }
                                            },
                                            "timeline" => rsx! {
                                                if tick.is_none() { p { class: "oxscn-hint", "Each step lights up here in order once a test run starts." } }
                                                else {
                                                    ol { class: "oxswf-tl",
                                                        for (i, (t, r)) in tl.into_iter().enumerate() { li { key: "{i}", "data-run": "{r.key()}", span { class: "oxswf-tl-n", "{i + 1}" } span { class: "oxscn-row-t", "{t}" } span { class: "oxscn-grow" } span { class: "oxswf-rs", "data-run": "{r.key()}", if r.label().is_empty() { "–" } else { "{r.label()}" } } } }
                                                    }
                                                }
                                            },
                                            _ => rsx! {
                                                if runs.is_empty() { p { class: "oxscn-hint", "No runs yet – press Start test run to try the workflow with the sample payload." } }
                                                else { ul { class: "oxswf-dl", for (i, r) in runs.into_iter().enumerate() { li { key: "{i}", {ico("runs")} span { class: "oxscn-row-t", "{r}" } span { class: "oxscn-grow" } span { class: "oxv-q", if i == 0 { "just now" } else { "earlier" } } } } } }
                                            },
                                        }}
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
    node: oxyn_core::kit::screensfid::WfNode,
    nodes: Signal<Vec<oxyn_core::kit::screensfid::WfNode>>,
    edges: Signal<Vec<(u32, u32)>>,
    run: String,
    on_gone: EventHandler<String>,
    on_said: EventHandler<String>,
) -> Element {
    use oxyn_core::kit::screensfid::{wf_connect, wf_kind_label, wf_remove};
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
    use oxyn_core::kit::screensfid::{set_changes, set_find, set_valid, settings};
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
    pages: Vec<oxyn_core::kit::screensfid::SetPage>,
    open: Signal<String>,
    q: Signal<String>,
    hits: Vec<(&'static str, &'static str)>,
    acct: Signal<bool>,
    loading: bool,
    on_back: EventHandler<()>,
) -> Element {
    use oxyn_core::kit::screensfid::SET_GROUPS;
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
    row: oxyn_core::kit::screensfid::SetRow,
    cur: Signal<Vec<oxyn_core::kit::screensfid::SetPage>>,
    saved: Signal<Vec<oxyn_core::kit::screensfid::SetPage>>,
    hit: bool,
    on_said: EventHandler<String>,
) -> Element {
    use oxyn_core::kit::screensfid::{
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
