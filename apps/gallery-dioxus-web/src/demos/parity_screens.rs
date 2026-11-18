//! Screens demos (#42 Screens 1, Screens 2): app frames (Inset, Flat, three-pane) and the AI Chat screens (Start, Overview,
//! Agents, Double sidebar, Progress, Run) Folders (Overview, Detail), Integrations, Kanban, CRM, the Workflow builder and the Settings shell, then the agent screens (Orchestration, Thread…). Every screen has Sample / Loading / Empty data and an
//! Inset / Flat frame; light and dark follow the page theme.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AgentThreadScreen, AnalyticsScreen, AppShellScreen, ChatAgentsScreen, ChatOverviewScreen,
    ChatProgressScreen, ChatRailScreen, ChatRunScreen, ChatStartScreen, CrmScreen, FoldersScreen,
    IntegrationsScreen, KanbanScreen, OrchestrationScreen, RailShellScreen, ScreenStage,
    SettingsScreen, WorkflowScreen, s2_states,
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
    controls_x(lay, &[])
}

/// The data control with a screen's own states after Sample, Loading and Empty.
fn controls_x(lay: &'static str, extra: &[&'static str]) -> Vec<crate::Control> {
    let mut data: Vec<&'static str> = vec!["Sample", "Loading", "Empty"];
    data.extend_from_slice(extra);
    vec![
        sel("data", "Data", &data, "Sample"),
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
            ScreenStage { state: st, screen: move |s: String| rsx! { AppShellScreen { layout: lay.clone(), state: s } } }
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
        height: 240,
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
            ScreenStage { state: st, screen: move |s: String| rsx! { AppShellScreen { layout: lay.clone(), state: s } } }
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
        height: 240,
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
            ScreenStage { state: st, screen: move |s: String| rsx! { RailShellScreen { layout: lay.clone(), state: s } } }
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
        height: 240,
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
            ScreenStage { state: st, states: oxyn_core::parity::screensfid::START_STATES.to_vec(), screen: move |s: String| rsx! { ChatStartScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn chat_start_code(v: &Vals) -> String {
    code("ChatStartScreen", v, true, false)
}

fn chat_start_demo() -> Demo {
    Demo {
        controls: controls_x("inset", &["Attached", "Dictating"]),
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
            var(
                "attached",
                "Files, a sheet and an app page attached in the composer.",
                &[("data", "Attached")],
            ),
            var(
                "dictating",
                "The composer taking dictation: a live waveform in place of the field.",
                &[("data", "Dictating")],
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
        height: 240,
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
            ScreenStage { state: st, states: oxyn_core::parity::screensfid::OVERVIEW_STATES.to_vec(), screen: move |s: String| rsx! { ChatOverviewScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn chat_overview_code(v: &Vals) -> String {
    code("ChatOverviewScreen", v, true, false)
}

fn chat_overview_demo() -> Demo {
    Demo {
        controls: controls_x("inset", &["Near limit"]),
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
            var(
                "limit",
                "Credits near the monthly cap: the meter turns amber with Upgrade.",
                &[("data", "Near limit")],
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
        height: 240,
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
            ScreenStage { state: st, states: oxyn_core::parity::screensfid::AGENTS_STATES.to_vec(), screen: move |s: String| rsx! { ChatAgentsScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn chat_agents_code(v: &Vals) -> String {
    code("ChatAgentsScreen", v, true, false)
}

fn chat_agents_demo() -> Demo {
    Demo {
        controls: controls_x("inset", &["Generating"]),
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
            var(
                "making",
                "The brief turning into ads, one at a time.",
                &[("data", "Generating")],
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
        height: 240,
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
            ScreenStage { state: st, states: oxyn_core::parity::screensfid::RAIL_STATES.to_vec(), screen: move |s: String| rsx! { ChatRailScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn chat_double_sidebar_code(v: &Vals) -> String {
    code("ChatRailScreen", v, true, false)
}

fn chat_double_sidebar_demo() -> Demo {
    Demo {
        controls: controls_x("inset", &["Chats folded"]),
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
            var(
                "folded",
                "The chats column folded to the rail; the thread takes the room.",
                &[("data", "Chats folded")],
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
        height: 240,
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
            ScreenStage { state: st, states: oxyn_core::parity::screensfid::PROGRESS_STATES.to_vec(), screen: move |s: String| rsx! { ChatProgressScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn chat_progress_code(v: &Vals) -> String {
    code("ChatProgressScreen", v, true, false)
}

fn chat_progress_demo() -> Demo {
    Demo {
        controls: controls_x("inset", &["Working"]),
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
            var(
                "working",
                "A turn still working: the column ticks steps off live.",
                &[("data", "Working")],
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
        height: 240,
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
            ScreenStage { state: st, states: oxyn_core::parity::screensfid::RUN_STATES.to_vec(), screen: move |s: String| rsx! { ChatRunScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn chat_run_code(v: &Vals) -> String {
    code("ChatRunScreen", v, true, false)
}

fn chat_run_demo() -> Demo {
    Demo {
        controls: controls_x("inset", &["Needs input", "Done"]),
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
            var(
                "input",
                "The run waiting on a yes: Needs your input asks before it sends.",
                &[("data", "Needs input")],
            ),
            var(
                "done",
                "Every step done and the draft reviewed.",
                &[("data", "Done")],
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
        height: 240,
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
    let key = format!("{key}-{}-{}-{}", v.s("view"), v.s("range"), v.s("x1"));
    let view = low(v.s("view"));
    let starred = v.b("x1");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s2_states("folders-overview"), screen: move |s: String| rsx! { FoldersScreen { layout: lay.clone(), state: s, view: view.clone(), starred } } }
        }
    }
}

fn folders_overview_code(v: &Vals) -> String {
    let c = Code::new("FoldersScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &low(v.s("data")), "sample");
    let c = if v.s("view") == "List" {
        c.s("view", "list", "grid")
    } else {
        c
    };
    let c = c.b("starred", v.b("x1"));
    c.done()
}

fn folders_overview_demo() -> Demo {
    let mut c = controls_x("inset", &["Creating"]);
    c.push(sel("view", "View", &["Grid", "List"], "Grid"));
    c.push(super::tog("x1", "Star a folder", false));
    Demo {
        controls: c,
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
                "Nothing yet: each region says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "creating",
                "The Create folder field open in the grid, focused.",
                &[("data", "Creating")],
            ),
            var(
                "list",
                "Folders as a list with when each changed.",
                &[("view", "List")],
            ),
            var(
                "starred",
                "Oxyn: a starred folder stays first.",
                &[("x1", "true")],
            ),
        ],
        render: folders_overview,
        code: folders_overview_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter in Create folder", "Create it (Escape cancels)"),
            ("Escape in a search", "Clear it"),
        ],
        a11y: vec![
            "The persona sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Folder tiles are buttons that name the folder and who can see it; the star is a toggle button.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Folder tiles go 4 → 2 → 1 columns; templates keep two columns on a phone.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "folders-overview",
            "Folders · Overview",
            CAT,
            "Files and the folders you keep them in: tabs, search, sort, grid or list and Create folder over the folder tiles, then file templates filtered by type.",
            "FoldersScreen",
            "FoldersEl",
            "folders-overview",
            "parity/workscr.rs",
        )
    }
}

// ---------- Folders · Detail ----------

fn folders_detail(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    let key = format!("{key}-{}-{}-{}", v.s("view"), v.s("range"), v.s("x1"));
    let view = low(v.s("view"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s2_states("folders-detail"), screen: move |s: String| rsx! { FoldersScreen { layout: lay.clone(), state: s, detail: true, view: view.clone() } } }
        }
    }
}

fn folders_detail_code(v: &Vals) -> String {
    let c = Code::new("FoldersScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &low(v.s("data")), "sample");
    let c = c.raw("detail", "true");
    let c = if v.s("view") == "List" {
        c.s("view", "list", "grid")
    } else {
        c
    };
    c.done()
}

fn folders_detail_demo() -> Demo {
    let mut c = controls_x("inset", &["Uploading"]);
    c.push(sel("view", "View", &["Grid", "List"], "Grid"));
    Demo {
        controls: c,
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
                "Nothing yet: each region says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "uploading",
                "Two files on their way in with progress bars.",
                &[("data", "Uploading")],
            ),
            var("list", "Files as a list.", &[("view", "List")]),
        ],
        render: folders_detail,
        code: folders_detail_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter in Share", "Invite the name typed"),
            ("Escape in Share", "Close the popover"),
        ],
        a11y: vec![
            "The persona sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Upload progress is a progressbar with a value; Share is a labelled dialog.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "The title row wraps under the name on a phone and sort steps aside.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "folders-detail",
            "Folders · Detail",
            CAT,
            "One folder open: breadcrumbs and a back link, the title row with More, sort, grid or list, Share and Upload files, then file cards with page previews, or an empty drop area.",
            "FoldersScreen",
            "FolderDetailEl",
            "folders-detail",
            "parity/workscr.rs",
        )
    }
}

// ---------- Integrations ----------

fn integrations(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    let key = format!("{key}-{}-{}-{}", v.s("view"), v.s("range"), v.s("x1"));
    let cm = v.b("x1");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s2_states("integrations"), screen: move |s: String| rsx! { IntegrationsScreen { layout: lay.clone(), state: s, cat_menu: cm } } }
        }
    }
}

fn integrations_code(v: &Vals) -> String {
    let c = Code::new("IntegrationsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &low(v.s("data")), "sample");
    let c = c.b("cat_menu", v.b("x1"));
    c.done()
}

fn integrations_demo() -> Demo {
    let mut c = controls_x("inset", &["Failed"]);
    c.push(super::tog("x1", "Category menu", false));
    Demo {
        controls: c,
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
                "Nothing yet: each region says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "failed",
                "A link that did not go through, with Try again.",
                &[("data", "Failed")],
            ),
            var(
                "categories",
                "Oxyn: a category menu narrows the page to one group.",
                &[("x1", "true")],
            ),
        ],
        render: integrations,
        code: integrations_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter on +", "Ask before linking; Link is focused"),
            ("Enter on ✓", "Settings or Unlink"),
        ],
        a11y: vec![
            "The persona sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Each app names its state in its button label (Link Slack, Slack is linked); errors are alerts.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "The two-column directory turns to one column under 900 px; the sticky header wraps its tabs on a phone.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "integrations",
            "Integrations",
            CAT,
            "A directory of apps to link: a sticky header with tabs, search and Request, apps grouped by category with official marks, a confirm before each link, and a linked app’s Settings or Unlink.",
            "IntegrationsScreen",
            "IntegrationsEl",
            "integrations",
            "parity/workscr.rs",
        )
    }
}

// ---------- Kanban ----------

fn kanban(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    let key = format!("{key}-{}-{}-{}", v.s("view"), v.s("range"), v.s("x1"));
    let view = low(v.s("view"));
    let labels = !v.b("x1");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s2_states("kanban"), screen: move |s: String| rsx! { KanbanScreen { layout: lay.clone(), state: s, view: view.clone(), labels } } }
        }
    }
}

fn kanban_code(v: &Vals) -> String {
    let c = Code::new("KanbanScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &low(v.s("data")), "sample");
    let c = if v.s("view") == "List" {
        c.s("view", "list", "board")
    } else {
        c
    };
    let c = if v.b("x1") {
        c.raw("labels", "false")
    } else {
        c
    };
    c.done()
}

fn kanban_demo() -> Demo {
    let mut c = controls_x("inset", &["Dragging"]);
    c.push(sel("view", "View", &["Board", "List"], "Board"));
    c.push(super::tog("x1", "Hide labels", false));
    Demo {
        controls: c,
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
                "Nothing yet: each region says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "dragging",
                "A card held over High priority, tilted, the lane lit.",
                &[("data", "Dragging")],
            ),
            var(
                "list",
                "The same tasks as a list grouped by lane.",
                &[("view", "List")],
            ),
            var(
                "plain",
                "Oxyn: cards without their label chips.",
                &[("x1", "true")],
            ),
        ],
        render: kanban,
        code: kanban_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Space / Enter on a card", "Pick it up or put it down"),
            ("← / → with a card up", "Carry it to the next lane"),
            ("Enter in New task", "Add it (Escape cancels)"),
        ],
        a11y: vec![
            "The persona sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Cards announce Space to pick up; each move is read out (Moved to High priority).",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Lanes keep 276 px and scroll sideways on narrow screens.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "kanban",
            "Kanban",
            CAT,
            "Inbox / Overview as a board: Filter, Sort by and Board | List, task tabs with counts, New task, and four 276 px lanes by priority with labelled cards you drag (with a tilt) or carry by keyboard.",
            "KanbanScreen",
            "KanbanEl",
            "kanban",
            "parity/workscr.rs",
        )
    }
}

// ---------- Workflow builder ----------

fn workflow(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, screen: move |s: String| rsx! { WorkflowScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn workflow_code(v: &Vals) -> String {
    code("WorkflowScreen", v, true, false)
}

fn workflow_demo() -> Demo {
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
        render: workflow,
        code: workflow_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            (
                "Arrow keys on a node",
                "Move it one grid step (Shift: five)",
            ),
            (
                "Enter / Space on a node",
                "Select it and open the inspector",
            ),
            ("Delete on a node or connection", "Remove it"),
            ("Escape on a node", "Clear the selection"),
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
        height: 240,
        live: true,
        align: Align::Start,
        ..mk(
            "workflow",
            "Workflow builder",
            CAT,
            "Nodes on a canvas joined by curves: add them from a palette (click or drag), move them with the pointer or the arrow keys, connect and set each one up in the inspector, zoom and fit, then run a test and watch every step light up in turn.",
            "WorkflowScreen",
            "WorkflowEl",
            "workflow",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Settings shell ----------

fn settings(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, screen: move |s: String| rsx! { SettingsScreen { layout: lay.clone(), state: s } } }
        }
    }
}

fn settings_code(v: &Vals) -> String {
    code("SettingsScreen", v, false, false)
}

fn settings_demo() -> Demo {
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
        render: settings,
        code: settings_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            (
                "Enter in Search settings",
                "Open the first page with a match",
            ),
            ("Escape in Search settings", "Clear the search"),
            ("Space on a switch", "Turn it on or off"),
            ("Escape in a confirm", "Cancel the destructive action"),
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
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings",
            "Settings shell",
            CAT,
            "The frame for settings: Back to app, a search that finds any setting, Personal and Workspace pages, and a centred column of switches, menus and checked fields, destructive actions confirmed in place, and a save bar that collects every change.",
            "SettingsScreen",
            "SettingsEl",
            "settings",
            "parity/screensfid.rs",
        )
    }
}

// ---------- CRM ----------

fn crm(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    let key = format!("{key}-{}-{}-{}", v.s("view"), v.s("range"), v.s("x1"));
    let totals = v.b("x1");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s2_states("crm"), screen: move |s: String| rsx! { CrmScreen { layout: lay.clone(), state: s, totals } } }
        }
    }
}

fn crm_code(v: &Vals) -> String {
    let c = Code::new("CrmScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &low(v.s("data")), "sample");
    let c = c.b("totals", v.b("x1"));
    c.done()
}

fn crm_demo() -> Demo {
    let mut c = controls_x("inset", &["Editing"]);
    c.push(super::tog("x1", "Pipeline strip", false));
    Demo {
        controls: c,
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
                "Nothing yet: each region says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "editing",
                "A Value cell open for editing.",
                &[("data", "Editing")],
            ),
            var(
                "totals",
                "Oxyn: a pipeline strip with rows and value per stage that filters the table.",
                &[("x1", "true")],
            ),
        ],
        render: crm,
        code: crm_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter or double-click on a cell", "Edit it"),
            ("Enter / Tab in a cell", "Save (Tab moves to the next cell)"),
            ("Escape in a cell", "Cancel the edit"),
        ],
        a11y: vec![
            "The persona sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "The table sorts from header buttons with aria-sort; a value that does not parse marks the cell aria-invalid.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Under 760 px the owner and last-contact columns hide and the table scrolls sideways.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "crm",
            "CRM",
            CAT,
            "Sales / Head of Sales: open views as tabs with their marks, stage and owner filters with search, and the Company CRM table with sortable headers, row select and Delete, Add row and cells you edit in place.",
            "CrmScreen",
            "CrmEl",
            "crm",
            "parity/workscr.rs",
        )
    }
}

// ---------- Agent · Orchestration ----------

fn agent_orchestration(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, screen: move |s: String| rsx! { OrchestrationScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn agent_orchestration_code(v: &Vals) -> String {
    code("OrchestrationScreen", v, true, false)
}

fn agent_orchestration_demo() -> Demo {
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
        render: agent_orchestration,
        code: agent_orchestration_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            (
                "← / → in the run panel tabs",
                "Trace, Tokens & cost, Events",
            ),
            (
                "↑ / ↓ on the panel edge",
                "Make the run panel taller or shorter",
            ),
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
        height: 240,
        live: true,
        align: Align::Start,
        ..mk(
            "agent-orchestration",
            "Agent · Orchestration",
            CAT,
            "One run shared out between agents, drawn as a branching outline on a dotted canvas: see which agent holds which piece, hop to whichever is busy, zoom and fit, pause or stop the run, and read every call in the trace, the tokens and cost per agent and the events below.",
            "OrchestrationScreen",
            "OrchestrationEl",
            "agent-orchestration",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Agent · Thread ----------

fn agent_thread(v: &Vals, now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, screen: move |s: String| rsx! { AgentThreadScreen { layout: lay.clone(), state: s, now } } }
        }
    }
}

fn agent_thread_code(v: &Vals) -> String {
    code("AgentThreadScreen", v, true, false)
}

fn agent_thread_demo() -> Demo {
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
        render: agent_thread,
        code: agent_thread_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then the work area"),
            ("← / → on a hairline", "Resize the sidebar or work area"),
            ("Escape", "Close an open menu; clear search"),
            ("Enter", "Send (Shift+Enter: new line)"),
            ("← / → in the work area tabs", "Artifacts or Tasks"),
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
        height: 240,
        live: true,
        align: Align::Start,
        ..mk(
            "agent-thread",
            "Agent · Thread",
            CAT,
            "One conversation with the orchestrator: plain answers, tools that ran side by side folded into one line, task cards and files, counts that open the work, and a reply that keeps streaming through a dropped connection, a slow upload and time away.",
            "AgentThreadScreen",
            "AgentThreadEl",
            "agent-thread",
            "parity/screensfid.rs",
        )
    }
}

// ---------- Analytics ----------

fn analytics(v: &Vals, _now: ClockMs) -> Element {
    let (st, lay, key) = frame(v);
    let key = format!("{key}-{}-{}-{}", v.s("view"), v.s("range"), v.s("x1"));
    let range = match v.s("range") {
        "30 days" => "30d",
        "90 days" => "90d",
        _ => "7d",
    }
    .to_string();
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s2_states("analytics"), screen: move |s: String| rsx! { AnalyticsScreen { layout: lay.clone(), state: s, range: range.clone() } } }
        }
    }
}

fn analytics_code(v: &Vals) -> String {
    let c = Code::new("AnalyticsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &low(v.s("data")), "sample");
    let c = match v.s("range") {
        "30 days" => c.s("range", "30d", "7d"),
        "90 days" => c.s("range", "90d", "7d"),
        _ => c,
    };
    c.done()
}

fn analytics_demo() -> Demo {
    let mut c = controls_x("inset", &["Compare"]);
    c.push(sel(
        "range",
        "Range",
        &["7 days", "30 days", "90 days"],
        "7 days",
    ));
    Demo {
        controls: c,
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
                "Nothing yet: each region says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "compare",
                "The previous period dashed under each channel.",
                &[("data", "Compare")],
            ),
            var(
                "month",
                "Opened on the last 30 days.",
                &[("range", "30 days")],
            ),
        ],
        render: analytics,
        code: analytics_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            (
                "← / → on a chart",
                "Move the crosshair a day; the readout is announced",
            ),
            ("Escape on a chart", "Hide the crosshair"),
        ],
        a11y: vec![
            "The persona sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Charts are labelled images with a live readout; the legend toggles are buttons with aria-pressed.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "KPI tiles go 4 → 2 columns and the chart rows stack under 980 px.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "analytics",
            "Analytics",
            CAT,
            "Analytics / Revenue: a date range, Compare, Export and Save as report over KPI tiles, revenue by channel with a legend you toggle, the revenue breakdown, top products and two weekly trends.",
            "AnalyticsScreen",
            "AnalyticsEl",
            "analytics",
            "parity/workscr.rs",
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
        workflow_demo(),
        settings_demo(),
        crm_demo(),
        analytics_demo(),
        agent_orchestration_demo(),
        agent_thread_demo(),
    ]
}
