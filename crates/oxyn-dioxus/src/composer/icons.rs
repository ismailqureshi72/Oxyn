//! Oxyn's own outlined icons (24 px grid, 1.5 px stroke, `currentColor`). Drawn in-house.

use dioxus::prelude::*;
use oxyn_core::composer::FileType;

/// Path data for an icon by name.
pub fn path(name: &str) -> &'static str {
    match name {
        "plus" => "M12 5v14M5 12h14",
        "clip" => {
            "M20.5 11.5 12.3 19.7a5 5 0 0 1-7.1-7.1l8.5-8.5a3.3 3.3 0 0 1 4.7 4.7l-8.5 8.5a1.7 1.7 0 0 1-2.4-2.4l7.8-7.8"
        }
        "up" => "M12 19V5M6 11l6-6 6 6",
        "stop" => "M7 7h10v10H7z",
        "pause" => "M8 6v12M16 6v12",
        "play" => "M8 5.5v13l10-6.5z",
        "steer" => "M6 5v6a4 4 0 0 0 4 4h8M14 11l4 4-4 4",
        "chevron" => "m7 10 5 5 5-5",
        "x" => "M6 6l12 12M18 6 6 18",
        "page" => "M7 3h7l4 4v14H7zM14 3v4h4M10 12h5M10 16h5",
        "pencil" => "M15.5 5.5l3 3L8 19H5v-3zM13.5 7.5l3 3",
        "prompt" => "M5 7l5 5-5 5M12 18h7",
        "globe" => {
            "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM3.5 12h17M12 3c2.5 2.6 3.5 5.6 3.5 9s-1 6.4-3.5 9c-2.5-2.6-3.5-5.6-3.5-9s1-6.4 3.5-9z"
        }
        "plug" => "M9 3v5M15 3v5M6 8h12v3a6 6 0 0 1-12 0zM12 17v4",
        "checklist" => {
            "M4 6.5l1.5 1.5L8.5 5M4 12.5 5.5 14l3-3M4 18.5 5.5 20l3-3M11 7h9M11 13h9M11 19h9"
        }
        "cog" => {
            "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM12 2.8v2.4M12 18.8v2.4M2.8 12h2.4M18.8 12h2.4M5.5 5.5l1.7 1.7M16.8 16.8l1.7 1.7M5.5 18.5l1.7-1.7M16.8 7.2l1.7-1.7"
        }
        "hand" => {
            "M8 13V6.5a1.5 1.5 0 0 1 3 0V12M11 11V4.5a1.5 1.5 0 0 1 3 0V12M14 11.5V6a1.5 1.5 0 0 1 3 0v8a7 7 0 0 1-7 7h-.5a6 6 0 0 1-4.4-1.9L3 16.6a1.5 1.5 0 0 1 2.2-2.1L8 17"
        }
        "copy" => "M9 9h11v11H9zM5 15H4V4h11v1",
        "ban" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM5.6 5.6l12.8 12.8",
        "agents" => {
            "M12 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM5 22a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19 22a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM12 8v4M12 12l-5.5 4.5M12 12l5.5 4.5"
        }
        "check" => "m5 12.5 4.5 4.5L19 7.5",
        "search" => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4-4",
        "star" => "m12 4 2.5 5.2 5.6.8-4 4 1 5.6-5.1-2.7L6.9 19.6l1-5.6-4-4 5.6-.8z",
        "lock" => "M7 11V8a5 5 0 0 1 10 0v3M6 11h12v9H6z",
        "key" => "M14.5 9.5a4 4 0 1 1-1.2 2.9L4 21.7V18h2.5v-2.5H9l2.3-2.3",
        "clock" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",
        "eye" => {
            "M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"
        }
        "tool" => "M14.7 6.3a4 4 0 0 0 5 5L12 19a2.1 2.1 0 0 1-3-3l7.7-7.7zM14.7 6.3 17.5 3.5",
        "brain" => {
            "M9 4a3 3 0 0 0-3 3 3 3 0 0 0-2 5 3 3 0 0 0 2 5 3 3 0 0 0 6 1V5a2 2 0 0 0-3-1zM15 4a3 3 0 0 1 3 3 3 3 0 0 1 2 5 3 3 0 0 1-2 5 3 3 0 0 1-6 1"
        }
        "retry" => "M4 12a8 8 0 1 0 2.3-5.7M4 4v4.5h4.5",
        "file" => "M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8zM14 3v5h5",
        "image" => {
            "M5 4h14a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zM4 16l5-5 4 4 3-3 4 4M15.5 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z"
        }
        "table" => "M4 5h16v14H4zM4 10h16M4 15h16M10 5v14",
        "folder" => "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
        "link" => {
            "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"
        }
        "branch" => {
            "M6 3v12M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM18 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM18 9a9 9 0 0 1-9 9"
        }
        "code" => "m9 8-4 4 4 4M15 8l4 4-4 4",
        "archive" => "M4 4h16v4H4zM5 8v12h14V8M10 12h4",
        "audio" => {
            "M9 18V6l10-2v12M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM19 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z"
        }
        "mic" => {
            "M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM5 11a7 7 0 0 0 14 0M12 18v3M9 21h6"
        }
        "video" => "M4 6h11v12H4zM15 10l5-3v10l-5-3",
        "text" => "M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8zM9 13h6M9 17h4",
        "symbol" => {
            "M10 4c-2 0-3 1-3 3v2c0 1.5-1 3-3 3 2 0 3 1.5 3 3v2c0 2 1 3 3 3M14 4c2 0 3 1 3 3v2c0 1.5 1 3 3 3-2 0-3 1.5-3 3v2c0 2-1 3-3 3"
        }
        "thread" => "M4 5h16v10H9l-5 4z",
        "cursor" => "M9 4h6M9 20h6M12 4v16",
        "spark" => {
            "M12 3v4M12 17v4M3 12h4M17 12h4M6 6l2.5 2.5M15.5 15.5 18 18M6 18l2.5-2.5M15.5 8.5 18 6"
        }
        "compact" => "M4 14h6v6M20 10h-6V4M14 10l7-7M3 21l7-7",
        "heart" => "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
        "repost" => "M7 7h10l-3-3M17 17H7l3 3M17 7v4M7 17v-4",
        "share" => "M12 4v11M8 8l4-4 4 4M5 13v6h14v-6",
        "comment" => "M5 5h14v10H10l-5 4z",
        "more" => "M6 12h.01M12 12h.01M18 12h.01",
        "minus" => "M6 12h12",
        "chevron-right" => "m10 7 5 5-5 5",
        "chevron-left" => "m14 7-5 5 5 5",
        "calendar" => "M5 6h14v14H5zM5 10h14M9 3v4M15 3v4",
        "phone" => {
            "M6.5 3.5h3l1.5 4-2 1.5a11 11 0 0 0 6 6l1.5-2 4 1.5v3a2 2 0 0 1-2 2A16 16 0 0 1 4.5 5.5a2 2 0 0 1 2-2z"
        }
        "sun" => {
            "M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4"
        }
        "moon" => "M19 14.5A7.5 7.5 0 0 1 9.5 5a7.5 7.5 0 1 0 9.5 9.5z",
        "undo" => "M9 7 5 11l4 4M5 11h9a5 5 0 0 1 0 10h-2",
        "building" => "M5 21V4h10v17M15 9h4v12M8 8h2M8 12h2M8 16h2M3 21h18",
        "mail" => "M4 6h16v12H4zM4 7l8 6 8-6",
        "eye-off" => {
            "M3 3l18 18M10.6 6.1A8.6 8.6 0 0 1 12 6c6 0 9.5 6 9.5 6a16 16 0 0 1-2.7 3.4M6.5 7.6C4 9.3 2.5 12 2.5 12S6 18.5 12 18.5c1.6 0 3-.4 4.2-1M9.9 9.9a3 3 0 0 0 4.2 4.2"
        }
        "trash" => "M5 7h14M10 7V4h4v3M7 7l1 13h8l1-13",
        "inbox" => "M4 13l2-8h12l2 8M4 13v6h16v-6M4 13h5l1 2h4l1-2h5",
        "bell" => "M6 16V11a6 6 0 0 1 12 0v5l2 2H4zM10 20a2 2 0 0 0 4 0",
        "filter" => "M4 5h16l-6 8v5l-4 2v-7z",
        "external" => "M14 4h6v6M20 4l-9 9M18 14v6H4V6h6",
        "bolt" => "M13 3 5 14h6l-1 7 8-11h-6z",
        "alert" => "M12 4 2.8 19.5h18.4zM12 10v4.5M12 17h.01",
        "x-circle" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM9 9l6 6M15 9l-6 6",
        "check-circle" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM8.5 12.2l2.4 2.3 4.6-5",
        "volume" => "M4 9.5h3.5L12 5.5v13l-4.5-4H4zM15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11",
        "monitor" => "M3.5 5h17v11h-17zM9 20h6M12 16v4",
        "type" => "M5 7V5h14v2M12 5v14M9 19h6",
        "users" => {
            "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM2.5 20a6.5 6.5 0 0 1 13 0M16 4.5a3.5 3.5 0 0 1 0 6.5M18 14a6 6 0 0 1 3.5 6"
        }
        "command" => {
            "M9 9V6.5A2.5 2.5 0 1 0 6.5 9H9zm0 0h6m-6 0v6m6-6V6.5A2.5 2.5 0 1 1 17.5 9H15zm0 0v6m0 0h2.5a2.5 2.5 0 1 1-2.5 2.5V15zm0 0H9m0 0v2.5A2.5 2.5 0 1 1 6.5 15H9z"
        }
        "home" => "M4 10.5 12 4l8 6.5V20h-5.5v-5.5h-5V20H4z",
        "grip" => "M9 6h.01M15 6h.01M9 12h.01M15 12h.01M9 18h.01M15 18h.01",
        "keyboard" => "M3 6.5h18v11H3zM7 10h.01M11 10h.01M15 10h.01M17 13.5H7",
        "info" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v5M12 8h.01",
        "merge" => "M6 3v18M6 9a6 6 0 0 0 6 6h6M15 12l3 3-3 3",
        "download" => "M12 4v11M8 11l4 4 4-4M5 19h14",
        _ => "M12 12h.01",
    }
}

/// The icon for a file type.
pub fn file_icon(t: FileType) -> &'static str {
    match t {
        FileType::Image => "image",
        FileType::Spreadsheet => "table",
        FileType::Code(_) => "code",
        FileType::Archive => "archive",
        FileType::Audio => "audio",
        FileType::Video => "video",
        FileType::Text | FileType::Pdf | FileType::Document | FileType::Slides => "text",
        FileType::Folder => "folder",
        FileType::Repo => "branch",
        FileType::Url => "link",
        FileType::Symbol => "symbol",
        FileType::Thread => "thread",
        FileType::Selection => "cursor",
        FileType::Unknown => "file",
    }
}

/// An inline icon.
#[component]
pub fn Icon(
    /// Icon name.
    name: String,
    /// Size in px.
    #[props(default = 16.0)]
    size: f32,
    /// Stroke width.
    #[props(default = 1.5)]
    stroke: f32,
) -> Element {
    let d = path(&name);
    let fill = if name == "stop" || name == "play" {
        "currentColor"
    } else {
        "none"
    };
    rsx! {
        svg {
            class: "oxyn-ic",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "{fill}",
            stroke: "currentColor",
            stroke_width: "{stroke}",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "{d}" }
        }
    }
}
