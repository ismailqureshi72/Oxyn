//! Screens 1 demos: app frames (Inset, Flat, three-pane) and the AI Chat screens (Start, Overview,
//! Agents, Double sidebar, Progress, Run) Folders (Overview, Detail), Integrations, Kanban and CRM. Every screen has Sample / Loading / Empty data and an
//! Inset / Flat frame; light and dark follow the page theme.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AppShellScreen, ChatAgentsScreen, ChatOverviewScreen, ChatProgressScreen, ChatRailScreen,
    ChatRunScreen, ChatStartScreen, CrmScreen, FoldersScreen, IntegrationsScreen, KanbanScreen,
    RailShellScreen,
};

use super::{Code, mk, sel, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "screens";

fn low(s: &str) -> String {
    s.to_lowercase()
}

fn frame(v: &Vals) -> (String, String, String) {
    let st = low(v.s("data"));
    let lay = low(v.s("frame"));
    let key = format!("{st}-{lay}");
    (st, lay, key)
}

fn code(name: &str, v: &Vals, live: bool, detail: bool) -> String {
    let c = Code::new(name).s("layout", &low(v.s("frame")), "inset").s(
        "state",
        &low(v.s("data")),
        "sample",
    );
    let c = if detail { c.raw("detail", "true") } else { c };
    if live {
        c.raw("now", "clock()").done()
    } else {
        c.done()
    }
}

fn controls(lay: &'static str) -> Vec<crate::Control> {
    vec![
        sel("data", "Data", &["Sample", "Loading", "Empty"], "Sample"),
        sel(
            "frame",
            "Frame",
            &["Inset", "Flat"],
            if lay == "flat" { "Flat" } else { "Inset" },
        ),
    ]
}

// ---------- App shell · Inset ----------

fn shell_inset(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            AppShellScreen { layout: lay, state: st }
        }
    }
}

fn shell_inset_code(v: &Vals) -> String {
    code("AppShellScreen", v, false, false)
}

fn shell_inset_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: shell_inset,
        code: shell_inset_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "shell-inset",
            "App shell · Inset",
            CAT,
            "The frame every screen sits in: a sidebar with workspace, search, projects, chats, a notice, setup and account, the page as an inset card, and a workspace on the right with tabs, tools and sources.",
            "AppShellScreen",
            "ScreenShellEl",
            "shell-inset",
            "parity/screensfid.rs",
        )
    }
}

// ---------- App shell · Flat ----------

fn shell_flat(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            AppShellScreen { layout: lay, state: st }
        }
    }
}

fn shell_flat_code(v: &Vals) -> String {
    code("AppShellScreen", v, false, false)
}

fn shell_flat_demo() -> Demo {
    Demo {
        controls: controls("flat"),
        variants: vec![
            var("sample", "Sample data in the Flat frame.", &[]),
            var(
                "inset",
                "The same screen in the Inset frame.",
                &[("frame", "Inset")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: shell_flat,
        code: shell_flat_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "shell-flat",
            "App shell · Flat",
            CAT,
            "The same frame laid flat: grey sidebar, edge-to-edge page and work area split by single hairlines; every region resizes.",
            "AppShellScreen",
            "ScreenShellEl",
            "shell-flat",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Three-pane shell ----------

fn shell_three_pane(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            RailShellScreen { layout: lay, state: st }
        }
    }
}

fn shell_three_pane_code(v: &Vals) -> String {
    code("RailShellScreen", v, false, false)
}

fn shell_three_pane_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: shell_three_pane,
        code: shell_three_pane_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "shell-three-pane",
            "Three-pane shell",
            CAT,
            "An icon rail, a chats column with a project switcher, the page and a work area: the double-sidebar frame.",
            "RailShellScreen",
            "ScreenRailEl",
            "shell-three-pane",
            "parity/screensfid.rs",
        )
    }
}

// ---------- AI Chat · Start ----------

fn chat_start(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ChatStartScreen { layout: lay, state: st, now }
        }
    }
}

fn chat_start_code(v: &Vals) -> String {
    code("ChatStartScreen", v, true, false)
}

fn chat_start_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: chat_start,
        code: chat_start_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter", "Send (Shift+Enter: new line)"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: true,
        align: Align::Start,
        ..mk(
            "chat-start",
            "AI Chat · Start",
            CAT,
            "A greeting over the composer – Add, ask-before-acting, model and voice – with a folder and apps to work in and starters that fill the prompt; sending opens a new chat.",
            "ChatStartScreen",
            "ChatStartEl",
            "chat-start",
            "parity/screensfid.rs",
        )
    }
}

// ---------- AI Chat · Overview ----------

fn chat_overview(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ChatOverviewScreen { layout: lay, state: st, now }
        }
    }
}

fn chat_overview_code(v: &Vals) -> String {
    code("ChatOverviewScreen", v, true, false)
}

fn chat_overview_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: chat_overview,
        code: chat_overview_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Arrow keys on Activity", "Step through days and weeks"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: true,
        align: Align::Start,
        ..mk(
            "chat-overview",
            "AI Chat · Overview",
            CAT,
            "The composer with the numbers under it: activity per day you step through by keyboard, recent chats, credits, files made for you and this month against last.",
            "ChatOverviewScreen",
            "ChatOverviewEl",
            "chat-overview",
            "parity/screensfid.rs",
        )
    }
}

// ---------- AI Chat · Agents ----------

fn chat_agents(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ChatAgentsScreen { layout: lay, state: st, now }
        }
    }
}

fn chat_agents_code(v: &Vals) -> String {
    code("ChatAgentsScreen", v, true, false)
}

fn chat_agents_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: chat_agents,
        code: chat_agents_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter / Space on an agent", "Show what it is doing"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: true,
        align: Align::Start,
        ..mk(
            "chat-agents",
            "AI Chat · Agents",
            CAT,
            "A chat with the orchestrator and the crew at work, beside a brief – angle, audience, channels, palette and formats – that turns into ads.",
            "ChatAgentsScreen",
            "ChatAgentsEl",
            "chat-agents",
            "parity/screensfid.rs",
        )
    }
}

// ---------- AI Chat · Double sidebar ----------

fn chat_double_sidebar(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ChatRailScreen { layout: lay, state: st, now }
        }
    }
}

fn chat_double_sidebar_code(v: &Vals) -> String {
    code("ChatRailScreen", v, true, false)
}

fn chat_double_sidebar_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: chat_double_sidebar,
        code: chat_double_sidebar_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Arrow keys in the sheet", "Move the selected cell"),
            ("Enter in Contents", "Keep the edit, move down"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: true,
        align: Align::Start,
        ..mk(
            "chat-double-sidebar",
            "AI Chat · Double sidebar",
            CAT,
            "Rail and chats column, a plain chat with web search folded into one line per answer, and artifacts on the right: an editable sheet and the sources.",
            "ChatRailScreen",
            "ChatRailEl",
            "chat-double-sidebar",
            "parity/screensfid.rs",
        )
    }
}

// ---------- AI Chat · Progress ----------

fn chat_progress(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ChatProgressScreen { layout: lay, state: st, now }
        }
    }
}

fn chat_progress_code(v: &Vals) -> String {
    code("ChatProgressScreen", v, true, false)
}

fn chat_progress_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: chat_progress,
        code: chat_progress_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter / Space on a tool card", "Open its input and output"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: true,
        align: Align::Start,
        ..mk(
            "chat-progress",
            "AI Chat · Progress",
            CAT,
            "A chat whose tool calls open into cards, with the work beside it: steps ticking off, what it made and the uploads, apps, web pages and skills it used.",
            "ChatProgressScreen",
            "ChatProgressEl",
            "chat-progress",
            "parity/screensfid.rs",
        )
    }
}

// ---------- AI Chat · Run ----------

fn chat_run(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ChatRunScreen { layout: lay, state: st, now }
        }
    }
}

fn chat_run_code(v: &Vals) -> String {
    code("ChatRunScreen", v, true, false)
}

fn chat_run_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: chat_run,
        code: chat_run_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter on a step", "Rename it"),
            ("Alt + ↑ / ↓", "Move a step"),
            ("Delete", "Remove a step"),
            ("Y / N in the draft", "Accept or reject the suggestion"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: true,
        align: Align::Start,
        ..mk(
            "chat-run",
            "AI Chat · Run",
            CAT,
            "One agent run: an editable plan ticking through its steps, live time and cost, a step that waits for your yes, and the draft with suggestions to accept or reject.",
            "ChatRunScreen",
            "ChatRunEl",
            "chat-run",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Folders · Overview ----------

fn folders_overview(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            FoldersScreen { layout: lay, state: st }
        }
    }
}

fn folders_overview_code(v: &Vals) -> String {
    code("FoldersScreen", v, false, false)
}

fn folders_overview_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: folders_overview,
        code: folders_overview_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter in New folder", "Create it (Escape cancels)"),
            ("Escape in Find", "Clear the search"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "folders-overview",
            "Folders · Overview",
            CAT,
            "The folders you share with people and agents: find one, sort, switch grid or list, star it to the top, make a new one in place, and open it.",
            "FoldersScreen",
            "FoldersEl",
            "folders-overview",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Folders · Detail ----------

fn folders_detail(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            FoldersScreen { layout: lay, state: st, detail: true }
        }
    }
}

fn folders_detail_code(v: &Vals) -> String {
    code("FoldersScreen", v, false, true)
}

fn folders_detail_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: folders_detail,
        code: folders_detail_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Space on a checkbox", "Select the file"),
            (
                "Delete",
                "Delete the selected files (Undo brings them back)",
            ),
            ("Enter in Invite", "Invite the address"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "folders-detail",
            "Folders · Detail",
            CAT,
            "One folder open: its files in a table you sort by any column, select, delete with undo and add to, with who it is shared with, their access, an invite field and agent access beside it.",
            "FoldersScreen",
            "FolderDetailEl",
            "folders-detail",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Integrations ----------

fn integrations(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            IntegrationsScreen { layout: lay, state: st }
        }
    }
}

fn integrations_code(v: &Vals) -> String {
    code("IntegrationsScreen", v, false, false)
}

fn integrations_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: integrations,
        code: integrations_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("← / → in the tabs", "Move between categories"),
            ("Escape in the dialog", "Cancel connecting"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "integrations",
            "Integrations",
            CAT,
            "An app directory with category tabs and search: Connect asks which permissions to grant first, and a connected app opens beside the list with switches for each permission, Sync now and a confirmed Disconnect.",
            "IntegrationsScreen",
            "IntegrationsEl",
            "integrations",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Kanban ----------

fn kanban(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            KanbanScreen { layout: lay, state: st }
        }
    }
}

fn kanban_code(v: &Vals) -> String {
    code("KanbanScreen", v, false, false)
}

fn kanban_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: kanban,
        code: kanban_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter / Space on a card", "Open it beside the board"),
            ("Alt + ← / →", "Move the card to the next column"),
            ("Alt + ↑ / ↓", "Move the card up or down"),
            ("Enter in a new card", "Add it (Escape cancels)"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "kanban",
            "Kanban",
            CAT,
            "A board in four columns with counts and work limits: drag cards between and within columns or move them with Alt and the arrows, find, show only yours, add a card in place, and open one beside the board to change its status, priority and assignee.",
            "KanbanScreen",
            "KanbanEl",
            "kanban",
            "parity/screensfid.rs",
        )
    }
}

// ---------- CRM ----------

fn crm(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            CrmScreen { layout: lay, state: st }
        }
    }
}

fn crm_code(v: &Vals) -> String {
    code("CrmScreen", v, false, false)
}

fn crm_demo() -> Demo {
    Demo {
        controls: controls("inset"),
        variants: vec![
            var("sample", "Sample data in the Inset frame.", &[]),
            var(
                "flat",
                "The same screen in the Flat frame.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons where the data will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
        ],
        render: crm,
        code: crm_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("← / → in the tabs", "Move between views"),
            ("Enter on a value", "Edit it (Enter keeps, Escape cancels)"),
            ("F2 on a company", "Rename it in place"),
            ("Escape in Find", "Clear the search"),
        ],
        a11y: vec![
            "The sidebar is a labelled nav; the open chat has aria-current and unread chats say so.",
            "Menus open from a trigger that reports aria-expanded; their items are menuitemradio and Escape shuts them.",
            "Hairlines are separators with a value and arrow keys; skeletons are hidden from screen readers.",
        ],
        responsive: vec![
            "Under 860 px of width the work area becomes a sheet opened from the top bar.",
            "Under 640 px the sidebar slides in from the menu button and the composer drops its labels.",
        ],
        height: 660,
        live: false,
        align: Align::Start,
        ..mk(
            "crm",
            "CRM",
            CAT,
            "Deals in a table with saved views as tabs: find, sort any column, edit a company or value in place, move a deal through its stages from the row, add a deal, and open one beside the table with its owner, notes and the pipeline by stage.",
            "CrmScreen",
            "CrmEl",
            "crm",
            "parity/screensfid.rs",
        )
    }
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        shell_inset_demo(),
        shell_flat_demo(),
        shell_three_pane_demo(),
        chat_start_demo(),
        chat_overview_demo(),
        chat_agents_demo(),
        chat_double_sidebar_demo(),
        chat_progress_demo(),
        chat_run_demo(),
        folders_overview_demo(),
        folders_detail_demo(),
        integrations_demo(),
        kanban_demo(),
        crm_demo(),
    ]
}
