//! Screens demos (#42 Screens 1, Screens 2): app frames (Inset, Flat, three-pane) and the AI Chat screens (Start, Overview,
//! Agents, Double sidebar, Progress, Run) Folders (Overview, Detail), Integrations, Kanban, CRM, the Workflow builder and the Settings shell, then the agent screens (Orchestration; S3: Thread, Inbox, Inbox Board, Tasks, Activity, Memory; S4: Agents, Skills, Schedules, Usage, Permissions, Run Detail; S5: Evals and the upgraded Orchestration and Workflow Builder; S6: Settings Profile, Preferences, Notifications, Privacy and General; S7: Settings Members, Models, Security, Billing and Get started; S8: the Onboarding wizard, Website, Brand, Tone and Ready; S9, Agent apps: Taskdeck Start, Task and Board; S10: Command Center and Session Studio). Every screen has Sample / Loading / Empty data and an
//! Inset / Flat frame; light and dark follow the page theme.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    ActivityScreen, AgentThreadScreen, AgentsScreen, AnalyticsScreen, AppShellScreen,
    ChatAgentsScreen, ChatOverviewScreen, ChatProgressScreen, ChatRailScreen, ChatRunScreen,
    ChatStartScreen, CrmScreen, EvalsScreen, FoldersScreen, InboxBoardScreen, InboxScreen,
    IntegrationsScreen, KanbanScreen, MemoryScreen, OrchestrationScreen, PermissionsScreen,
    RailShellScreen, RunDetailScreen, SchedulesScreen, ScreenStage, SettingsScreen, SkillsScreen,
    TasksScreen, UsageScreen, WorkflowScreen, s2_states, s3_states, s4_states, s5_states,
};
use oxyn_dioxus::{
    BillingSettingsScreen, GeneralSettingsScreen, MembersSettingsScreen, ModelsSettingsScreen,
    NotificationsSettingsScreen, OnboardingSettingsScreen, PreferencesSettingsScreen,
    PrivacySettingsScreen, ProfileSettingsScreen, SecuritySettingsScreen, s6_states,
};
use oxyn_dioxus::{CommandCenterScreen, SessionStudioScreen, s10_states};
use oxyn_dioxus::{FleetWindowScreen, s11_fleet_states};
use oxyn_dioxus::{
    OnboardingBrandScreen, OnboardingReadyScreen, OnboardingToneScreen, OnboardingWebsiteScreen,
    s8_states,
};
use oxyn_dioxus::{
    TaskdeckBoardScreen, TaskdeckPluginsScreen, TaskdeckScheduledScreen, TaskdeckSitesScreen,
    TaskdeckStartScreen, TaskdeckTaskScreen, s9_states,
};
use oxyn_dioxus::{WorkspaceCoderScreen, s11_coder_states};

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

/// The state key for a data control label on an S3 screen (its own states by label).
fn s3_key(slug: &str, label: &str) -> String {
    s3_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
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
            "Three-pane shell (Oxyn extra)",
            CAT,
            "Oxyn-only extra, with no affectUI counterpart: an icon rail, a chats column with a project switcher, the page and a work area. Kept as our own frame beside the affectUI-matched shells.",
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
    let (_, lay, key) = frame(v);
    let st = s5_key("workflow", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s5_states("workflow"), screen: move |s: String| rsx! { WorkflowScreen { layout: lay.clone(), state: s, now, step_numbers: a, compact: b } } }
        }
    }
}

fn workflow_code(v: &Vals) -> String {
    let c = Code::new("WorkflowScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s5_key("workflow", v.s("data")), "sample");
    let c = c.b("step_numbers", v.b("x1")).b("compact", v.b("x2"));
    c.raw("now", "clock()").done()
}

fn workflow_demo() -> Demo {
    let mut c = controls_x("inset", &["Test run"]);
    c.push(super::tog("x1", "Step numbers", false));
    c.push(super::tog("x2", "Compact cards", false));
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
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "A test run under way: steps light up in turn and the run dock shows the timeline.",
                &[("data", "Test run")],
            ),
            var(
                "x1",
                "Oxyn: each step numbered in run order.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: compact cards without the setting line.",
                &[("x2", "true")],
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
            (
                "← / → in the run dock tabs",
                "Payload, Triggers, Errors, Timeline, Runs",
            ),
            ("Enter on Start test run", "Run with the sample payload"),
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
            "Steps laid out on a canvas and linked by curves: add them from a grouped palette (click or drag; official app marks), move them with the pointer or the arrow keys, connect and set each one up in the inspector, zoom and fit, then start a test run with the sample payload and follow it in the run dock (Payload · Triggers · Errors · Timeline · Runs) as every step lights up in turn.",
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
    let (_, lay, key) = frame(v);
    let st = s5_key("agent-orchestration", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s5_states("agent-orchestration"), screen: move |s: String| rsx! { OrchestrationScreen { layout: lay.clone(), state: s, now, cost_bars: a, dim_done: b } } }
        }
    }
}

fn agent_orchestration_code(v: &Vals) -> String {
    let c = Code::new("OrchestrationScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s(
            "state",
            &s5_key("agent-orchestration", v.s("data")),
            "sample",
        );
    let c = c.b("cost_bars", v.b("x1")).b("dim_done", v.b("x2"));
    c.raw("now", "clock()").done()
}

fn agent_orchestration_demo() -> Demo {
    let mut c = controls_x("inset", &["Paused"]);
    c.push(super::tog("x1", "Cost bars", false));
    c.push(super::tog("x2", "Dim finished", false));
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
                "Nothing yet: each region says what will appear.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "The run paused: the clock and cost hold, Resume picks it up again.",
                &[("data", "Paused")],
            ),
            var(
                "x1",
                "Oxyn: each card shows its share of the run's cost as a bar.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: finished agents fade so the busy ones stand out.",
                &[("x2", "true")],
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
            "One run shared out between agents, drawn as a branching tree of cards on a dotted canvas, each with its role tag (Orchestrator, Research, Analysis, Writing, Review), status and model (official marks): see which agent holds which piece, hop to whichever is busy, zoom and fit, pause or stop the run, and read every call in the trace, the tokens and cost per agent and the events in the dock below.",
            "OrchestrationScreen",
            "OrchestrationEl",
            "agent-orchestration",
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

// ---------- Agent · Thread ----------

fn agent_thread(v: &Vals, now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s3_key("agent-thread", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (to, sm) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s3_states("agent-thread"), screen: move |s: String| rsx! { AgentThreadScreen { layout: lay.clone(), state: s, now, tools_open: to, stamps: sm } } }
        }
    }
}

fn agent_thread_code(v: &Vals) -> String {
    let c = Code::new("AgentThreadScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s3_key("agent-thread", v.s("data")), "sample");
    let c = c.raw("now", "clock()");
    let c = c.b("tools_open", v.b("x1")).b("stamps", v.b("x2"));
    c.done()
}

fn agent_thread_demo() -> Demo {
    let mut c = controls_x(
        "inset",
        &[
            "Decision",
            "Scrolled up",
            "Offline",
            "Uploading",
            "Dictating",
            "Away summary",
            "Error",
        ],
    );
    c.push(super::tog("x1", "Tools unfolded", false));
    c.push(super::tog("x2", "Time stamps", false));
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
                "decision",
                "A decision card: press 1, 2 or 3 to answer.",
                &[("data", "Decision")],
            ),
            var(
                "scrolled",
                "Scrolled up with a Jump to latest pill.",
                &[("data", "Scrolled up")],
            ),
            var(
                "offline",
                "Offline: a banner, and messages queue until Reconnect.",
                &[("data", "Offline")],
            ),
            var(
                "uploading",
                "A file uploading in the composer.",
                &[("data", "Uploading")],
            ),
            var(
                "dictating",
                "Dictating: a live waveform and timer.",
                &[("data", "Dictating")],
            ),
            var(
                "away",
                "What finished while you were away, on top.",
                &[("data", "Away summary")],
            ),
            var(
                "error",
                "A reply that failed, with Retry.",
                &[("data", "Error")],
            ),
            var(
                "tools",
                "Oxyn: tool lines start unfolded.",
                &[("x1", "true")],
            ),
            var(
                "stamps",
                "Oxyn: a time on each of your messages.",
                &[("x2", "true")],
            ),
        ],
        render: agent_thread,
        code: agent_thread_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter", "Send (Shift+Enter: new line)"),
            ("1 / 2 / 3 on a decision", "Answer it"),
            ("Enter on a tool line", "Unfold the calls"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "The thread is a log; the decision card lists its answers with aria-keyshortcuts; upload progress is a progressbar.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "The column is 680 px wide and centred; under 520 px it takes the full width and the composer drops its labels.",
        ],
        height: 240,
        live: true,
        align: Align::Start,
        ..mk(
            "agent-thread",
            "Agent · Thread",
            CAT,
            "One full-width thread with the orchestrator: task cards, tools folded into one line, file chips and a composer, across all nine states (Live, Decision, Scrolled up, Offline, Uploading, Dictating, Away summary, Error, Loading). The Artifacts work area is folded behind its one toggle.",
            "AgentThreadScreen",
            "AgentThreadEl",
            "agent-thread",
            "parity/agentscr.rs",
        )
    }
}

// ---------- Agent · Inbox ----------

fn inbox_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s3_key("inbox", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (pk, hd) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s3_states("inbox"), screen: move |s: String| rsx! { InboxScreen { layout: lay.clone(), state: s, peek_next: pk, hold: hd } } }
        }
    }
}

fn inbox_screen_code(v: &Vals) -> String {
    let c = Code::new("InboxScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s3_key("inbox", v.s("data")), "sample");
    let c = c.b("peek_next", v.b("x1")).b("hold", v.b("x2"));
    c.done()
}

fn inbox_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Just answered"]);
    c.push(super::tog("x1", "Peek at next", false));
    c.push(super::tog("x2", "Hold after answer", false));
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
                "answered",
                "Just answered: the next question has glided in, with Undo.",
                &[("data", "Just answered")],
            ),
            var(
                "peek",
                "Oxyn: a peek at the next question.",
                &[("x1", "true")],
            ),
            var(
                "hold",
                "Oxyn: stay on the answered question.",
                &[("x2", "true")],
            ),
        ],
        render: inbox_screen,
        code: inbox_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            (
                "1–9 on a question",
                "Answer with that choice; the next question glides in",
            ),
            ("Y / N", "First or second answer"),
            ("↑ / ↓ then Enter", "Pick and send"),
            ("L", "Later: move it to the end"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Answers are a radiogroup; the question region names its shortcuts with aria-keyshortcuts; Undo sits in a status toast.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Under 760 px the list stacks above the question.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "inbox",
            "Agent · Inbox",
            CAT,
            "Questions agents are waiting on: a 320 px list tinted by project with Open / Done tabs, one question at a time with its answers, and the task behind it in the work area. One keystroke answers and the next question glides in.",
            "InboxScreen",
            "InboxScreenEl",
            "inbox",
            "parity/agentscr.rs",
        )
    }
}

// ---------- Agent · Inbox Board ----------

fn inbox_board(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s3_key("agent-inbox-board", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (cp, wp) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s3_states("agent-inbox-board"), screen: move |s: String| rsx! { InboxBoardScreen { layout: lay.clone(), state: s, compact: cp, wip: wp } } }
        }
    }
}

fn inbox_board_code(v: &Vals) -> String {
    let c = Code::new("InboxBoardScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s3_key("agent-inbox-board", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("wip", v.b("x2"));
    c.done()
}

fn inbox_board_demo() -> Demo {
    let mut c = controls_x("inset", &["Run failed"]);
    c.push(super::tog("x1", "Compact cards", false));
    c.push(super::tog("x2", "WIP limit", false));
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
                "A run that failed, with Retry in the work area.",
                &[("data", "Run failed")],
            ),
            var("compact", "Oxyn: one-line cards.", &[("x1", "true")]),
            var(
                "wip",
                "Oxyn: a limit on Running, shown as 3 / 4.",
                &[("x2", "true")],
            ),
        ],
        render: inbox_board,
        code: inbox_board_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter on a card", "Open it in the work area"),
            ("Enter in New task", "Start it (Escape cancels)"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Lanes are labelled regions with counts; cards are buttons with aria-current when open.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Lanes stack under 760 px.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-inbox-board",
            "Agent · Inbox Board",
            CAT,
            "Tasks as three lanes (Needs you, Running, Done) with the agent’s mark, time, sources and tool calls. Tabs narrow to a lane, Agents filters, board or list, New task, and Approve, Reject or Retry in the work area.",
            "InboxBoardScreen",
            "InboxBoardEl",
            "agent-inbox-board",
            "parity/agentscr.rs",
        )
    }
}

// ---------- Agent · Tasks ----------

fn tasks_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s3_key("agent-tasks", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (co, fo) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s3_states("agent-tasks"), screen: move |s: String| rsx! { TasksScreen { layout: lay.clone(), state: s, cost: co, folded: fo } } }
        }
    }
}

fn tasks_screen_code(v: &Vals) -> String {
    let c = Code::new("TasksScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s3_key("agent-tasks", v.s("data")), "sample");
    let c = c.b("cost", v.b("x1")).b("folded", v.b("x2"));
    c.done()
}

fn tasks_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Done tab"]);
    c.push(super::tog("x1", "Cost column", false));
    c.push(super::tog("x2", "Groups folded", false));
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
                "finished",
                "The Done tab with a finished task open.",
                &[("data", "Done tab")],
            ),
            var(
                "cost",
                "Oxyn: what each task has cost so far.",
                &[("x1", "true")],
            ),
            var("folded", "Oxyn: groups start folded.", &[("x2", "true")]),
        ],
        render: tasks_screen,
        code: tasks_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter on a group", "Fold or unfold it"),
            ("Enter on Decide", "Show the answers inline"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Progress rings are hidden; each row states its steps as text; group heads report aria-expanded.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Under 520 px the steps, files and age columns hide.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-tasks",
            "Agent · Tasks",
            CAT,
            "Tasks grouped by project with progress rings, what each is doing and a Needs you tag. The work area holds the decision waiting (Decide answers it inline), steps, what happened so far, artifacts and what was decided.",
            "TasksScreen",
            "TasksEl",
            "agent-tasks",
            "parity/agentscr.rs",
        )
    }
}

// ---------- Agent · Activity ----------

fn activity_screen(v: &Vals, now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s3_key("agent-activity", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (co, ck) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s3_states("agent-activity"), screen: move |s: String| rsx! { ActivityScreen { layout: lay.clone(), state: s, now, cost: co, clock: ck } } }
        }
    }
}

fn activity_screen_code(v: &Vals) -> String {
    let c = Code::new("ActivityScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s3_key("agent-activity", v.s("data")), "sample");
    let c = c.raw("now", "clock()");
    let c = c.b("cost", v.b("x1")).b("clock", v.b("x2"));
    c.done()
}

fn activity_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Undone"]);
    c.push(super::tog("x1", "Cost column", false));
    c.push(super::tog("x2", "Clock countdown", false));
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
                "undone",
                "The first step undone: struck through and counted under Undone.",
                &[("data", "Undone")],
            ),
            var(
                "cost",
                "Oxyn: each step’s cost on its row.",
                &[("x1", "true")],
            ),
            var(
                "clock",
                "Oxyn: the undo window as a clock.",
                &[("x2", "true")],
            ),
        ],
        render: activity_screen,
        code: activity_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter on a row", "Open the step"),
            ("Enter on Undo", "Take it back while the window is open"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Each Undo button names the step it reverses; the countdown is a timer.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Under 520 px the time column hides and Undo keeps its icon.",
        ],
        height: 240,
        live: true,
        align: Align::Start,
        ..mk(
            "agent-activity",
            "Agent · Activity",
            CAT,
            "What agents did in your apps, grouped by day: filter tabs with counts, a search, official app marks and Undo or Final on each row. The work area shows When, Task and Cost, What happened, Why it could and Taking it back with a live countdown.",
            "ActivityScreen",
            "ActivityEl",
            "agent-activity",
            "parity/agentscr.rs",
        )
    }
}

// ---------- Agent · Memory ----------

fn memory_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s3_key("agent-memory", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (sm, cp) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s3_states("agent-memory"), screen: move |s: String| rsx! { MemoryScreen { layout: lay.clone(), state: s, scope_menu: sm, compact: cp } } }
        }
    }
}

fn memory_screen_code(v: &Vals) -> String {
    let c = Code::new("MemoryScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s3_key("agent-memory", v.s("data")), "sample");
    let c = c.b("scope_menu", v.b("x1")).b("compact", v.b("x2"));
    c.done()
}

fn memory_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["To review"]);
    c.push(super::tog("x1", "Scope menu", false));
    c.push(super::tog("x2", "Compact rows", false));
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
                "review",
                "The To review tab: memories waiting for Keep or Forget.",
                &[("data", "To review")],
            ),
            var(
                "scope",
                "Oxyn: a scope menu narrows to one group.",
                &[("x1", "true")],
            ),
            var(
                "compact",
                "Oxyn: rows without source and use counts.",
                &[("x2", "true")],
            ),
        ],
        render: memory_screen,
        code: memory_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("← / → in a tab row", "Move between tabs"),
            ("Escape", "Close an open menu or popover; clear a search"),
            ("Enter in Remember", "Save it (Escape cancels)"),
            ("Enter on Forget", "Remove it; Undo brings it back"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Tabs are a tablist with arrow keys; menus report aria-expanded and close on Escape.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
            "Group heads report aria-expanded; Forget announces itself and offers Undo.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and top-bar actions keep their icons with hidden labels.",
            "Under 760 px search hides; under 520 px use counts hide.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-memory",
            "Agent · Memory",
            CAT,
            "What agents remember, grouped by scope (About you and each project), with where each came from, how often it was used and when. Saved / To review, a search and Remember; the work area shows its source, its uses and Forget.",
            "MemoryScreen",
            "MemoryEl",
            "agent-memory",
            "parity/agentscr.rs",
        )
    }
}

/// The state key for a data control label on an S4 screen.
fn s4_key(slug: &str, label: &str) -> String {
    s4_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
}

// ---------- Agent · Agents ----------

fn agents_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s4_key("agent-agents", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s4_states("agent-agents"), screen: move |s: String| rsx! { AgentsScreen { layout: lay.clone(), state: s, by_spend: a, budget_bars: b } } }
        }
    }
}

fn agents_screen_code(v: &Vals) -> String {
    let c = Code::new("AgentsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s4_key("agent-agents", v.s("data")), "sample");
    let c = c.b("by_spend", v.b("x1")).b("budget_bars", v.b("x2"));
    c.done()
}

fn agents_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["New agent"]);
    c.push(super::tog("x1", "Sort by spend", false));
    c.push(super::tog("x2", "Spend bars", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "New agent open: the mark draws itself from the typed name.",
                &[("data", "New agent")],
            ),
            var(
                "x1",
                "Oxyn: cards sorted by spend, most first.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: a spend bar against a €50 cap on each card.",
                &[("x2", "true")],
            ),
        ],
        render: agents_screen,
        code: agents_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("Enter on a card", "Open the agent in the work area"),
            ("Enter in New agent", "Create it (Escape cancels)"),
            ("Enter in Message", "Send it (Shift+Enter breaks a line)"),
            ("Escape", "Close an open menu or popover; clear a search"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Cards are buttons with aria-current; New agent is a dialog with the model as a radio group; Pause is a toggle button.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and the work area becomes a sheet.",
            "Under 560 px the grid is one column; under 640 px the view toggle hides.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-agents",
            "Agent · Agents",
            CAT,
            "The agent team as a grid of cards: a pixel mark, role, status tag, official app marks, model, runs and spend, with All / Working / Idle / Paused tabs and a grid or list view. Typing a name in New agent builds its mark live; the work area (348) shows status, model, runs over two weeks, instructions, skills, apps, recent tasks and Message.",
            "AgentsScreen",
            "AgentsEl",
            "agent-agents",
            "parity/opsscr.rs",
        )
    }
}

// ---------- Agent · Skills ----------

fn skills_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s4_key("agent-skills", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s4_states("agent-skills"), screen: move |s: String| rsx! { SkillsScreen { layout: lay.clone(), state: s, compact: a, by_use: b } } }
        }
    }
}

fn skills_screen_code(v: &Vals) -> String {
    let c = Code::new("SkillsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s4_key("agent-skills", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("by_use", v.b("x2"));
    c.done()
}

fn skills_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Installing"]);
    c.push(super::tog("x1", "Compact cards", false));
    c.push(super::tog("x2", "Sort by use", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "Install pressed on a Discover card: the ring is filling.",
                &[("data", "Installing")],
            ),
            var(
                "x1",
                "Oxyn: cards without the description line.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: Discover sorted by how many use each skill.",
                &[("x2", "true")],
            ),
        ],
        render: skills_screen,
        code: skills_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("Space on a switch", "Turn the skill on or off"),
            (
                "Enter on +",
                "Install it; the ring fills, then it moves to Installed",
            ),
            ("Enter in New skill", "Create it (Escape cancels)"),
            ("Escape", "Close an open menu or popover; clear a search"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Switches are role=switch with the skill's name; Install reports aria-busy while the ring fills.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and the work area becomes a sheet.",
            "Under 640 px search hides; under 560 px cards are one column.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-skills",
            "Agent · Skills",
            CAT,
            "Skills agents can use: installed skills as cards with a glyph in the skill's hue, a switch and who made it, category tabs and a search, then Discover cards whose Install fills a ring. The work area shows the skill's runs over two weeks, when it steps in, its steps, what it needs and what ran lately.",
            "SkillsScreen",
            "SkillsEl",
            "agent-skills",
            "parity/opsscr.rs",
        )
    }
}

// ---------- Agent · Schedules ----------

fn schedules_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s4_key("agent-schedules", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s4_states("agent-schedules"), screen: move |s: String| rsx! { SchedulesScreen { layout: lay.clone(), state: s, hide_paused: a, month: b } } }
        }
    }
}

fn schedules_screen_code(v: &Vals) -> String {
    let c = Code::new("SchedulesScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s4_key("agent-schedules", v.s("data")), "sample");
    let c = c.b("hide_paused", v.b("x1")).b("month", v.b("x2"));
    c.done()
}

fn schedules_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Running now"]);
    c.push(super::tog("x1", "Hide paused", false));
    c.push(super::tog("x2", "Month cost", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "Run now pressed: the first mark fills and the row says Running.",
                &[("data", "Running now")],
            ),
            var("x1", "Oxyn: paused schedules hidden.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: a footer with what the schedules cost in a month.",
                &[("x2", "true")],
            ),
        ],
        render: schedules_screen,
        code: schedules_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("Enter on a mark or row", "Open the schedule"),
            ("Space on a switch", "Pause or resume it"),
            ("Enter on Skip", "Skip that run (Undo brings it back)"),
            ("Escape", "Close an open menu or popover; clear a search"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Strip marks are buttons named by schedule and time; switches are role=switch; Run now announces itself.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and the work area becomes a sheet.",
            "Under 760 px the rhythm column hides; under 560 px cost and the relative time hide.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-schedules",
            "Agent · Schedules",
            CAT,
            "What agents do on a rhythm: the hours around now as a strip (activity before now, a mark per run after it that fills while it runs), then schedule rows with the rhythm in words, next run, last result, cost and a switch. The work area has Run now, Pause, what it does, next runs with Skip and the last runs.",
            "SchedulesScreen",
            "SchedulesEl",
            "agent-schedules",
            "parity/opsscr.rs",
        )
    }
}

// ---------- Agent · Usage ----------

fn usage_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s4_key("agent-usage", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s4_states("agent-usage"), screen: move |s: String| rsx! { UsageScreen { layout: lay.clone(), state: s, daily: a, alert: b } } }
        }
    }
}

fn usage_screen_code(v: &Vals) -> String {
    let c = Code::new("UsageScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s4_key("agent-usage", v.s("data")), "sample");
    let c = c.b("daily", v.b("x1")).b("alert", v.b("x2"));
    c.done()
}

fn usage_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Over budget"]);
    c.push(super::tog("x1", "Daily bars", false));
    c.push(super::tog("x2", "Alert line", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "The pace runs past the budget: the forecast says when it runs out.",
                &[("data", "Over budget")],
            ),
            var(
                "x1",
                "Oxyn: day-by-day bars in place of the running total.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: an alert line at 80 % of the budget.",
                &[("x2", "true")],
            ),
        ],
        render: usage_screen,
        code: usage_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("Enter on a project row", "Open it in the work area"),
            ("Enter on Projects / Models", "Switch the breakdown"),
            ("Enter on Open", "Open the item worth a look"),
            ("Escape", "Close an open menu or popover; clear a search"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "The chart is role=img; its label carries spend, budget and forecast. Projects / Models are pressed buttons.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and the work area becomes a sheet.",
            "Under 560 px the totals go two by two and the share bars hide.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-usage",
            "Agent · Usage",
            CAT,
            "What agents cost this month: the running total against a budget line with a plain-language forecast, tasks, runs, cost per task and tokens, where it went by project or model (official marks) and what is worth a look. The work area shows the project day by day and what cost the most.",
            "UsageScreen",
            "UsageEl",
            "agent-usage",
            "parity/opsscr.rs",
        )
    }
}

// ---------- Agent · Permissions ----------

fn permissions_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s4_key("agent-permissions", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s4_states("agent-permissions"), screen: move |s: String| rsx! { PermissionsScreen { layout: lay.clone(), state: s, compact: a, lock_red: b } } }
        }
    }
}

fn permissions_screen_code(v: &Vals) -> String {
    let c = Code::new("PermissionsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s4_key("agent-permissions", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("lock_red", v.b("x2"));
    c.done()
}

fn permissions_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Hands-off"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Lock red lines", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "Hands-off picked: everything on its own except the red lines.",
                &[("data", "Hands-off")],
            ),
            var(
                "x1",
                "Oxyn: app rows without the account line.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: red lines locked, with a lock.",
                &[("x2", "true")],
            ),
        ],
        render: permissions_screen,
        code: permissions_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            (
                "← / → in the levels",
                "Move between Careful, Balanced and Hands-off",
            ),
            ("Enter on an app", "Open its actions"),
            ("↑ / ↓ in a rule", "Pick Never, Asks first or On its own"),
            ("Escape", "Close an open menu or popover; clear a search"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "Careful, Balanced and Hands-off form a radiogroup (arrow keys move); each rule is a native select named by its action.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and the work area becomes a sheet.",
            "Under 560 px the levels stack and the per-app counts hide.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-permissions",
            "Agent · Permissions",
            CAT,
            "What the agent may do without asking: Careful / Balanced / Hands-off with the counts each leads to, the apps it works in (official marks) with their counts, and fixed rules that win over any level. The work area (388) lists one app's actions by kind, each with a select of three settings, a refund limit and recent events.",
            "PermissionsScreen",
            "PermissionsEl",
            "agent-permissions",
            "parity/opsscr.rs",
        )
    }
}

// ---------- Agent · Run Detail ----------

fn run_detail_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s4_key("agent-run-detail", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s4_states("agent-run-detail"), screen: move |s: String| rsx! { RunDetailScreen { layout: lay.clone(), state: s, errors_only: a, cost_labels: b } } }
        }
    }
}

fn run_detail_screen_code(v: &Vals) -> String {
    let c = Code::new("RunDetailScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s4_key("agent-run-detail", v.s("data")), "sample");
    let c = c.b("errors_only", v.b("x1")).b("cost_labels", v.b("x2"));
    c.done()
}

fn run_detail_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Stopped"]);
    c.push(super::tog("x1", "Errors only", false));
    c.push(super::tog("x2", "Cost labels", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "Stopped at the failed dry run, with Retry.",
                &[("data", "Stopped")],
            ),
            var(
                "x1",
                "Oxyn: only the failures and the steps around them.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: each bar labelled with its cost.",
                &[("x2", "true")],
            ),
        ],
        render: run_detail_screen,
        code: run_detail_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            ("↑ / ↓ in the trace", "Move through the steps"),
            ("Home / End", "First or last step"),
            ("Enter on Stop", "Stop the run (Resume or Retry restart it)"),
            ("Escape", "Close an open menu or popover; clear a search"),
        ],
        a11y: vec![
            "The Fieldwork sidebar is a labelled nav with aria-current on this page.",
            "The trace is a listbox with aria-activedescendant; each step names its kind and length.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the sidebar slides in from the menu button and the work area becomes a sheet.",
            "Under 760 px the label column narrows; under 560 px tiles go two by two and every other tick hides.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "agent-run-detail",
            "Agent · Run Detail",
            CAT,
            "One run on one time axis: task, agent, model and start with Running and Stop, tiles for time, cost, tokens and steps, then thinking, tool calls (official marks), sub-agents with their steps indented and failures as tinted bars with a now line. The work area (348) shows the step with its input and output.",
            "RunDetailScreen",
            "RunDetailEl",
            "agent-run-detail",
            "parity/opsscr.rs",
        )
    }
}

/// The state key for a data control label on an S5 screen.
fn s5_key(slug: &str, label: &str) -> String {
    s5_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
}

// ---------- Evals ----------

fn evals_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s5_key("evals", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s5_states("evals"), screen: move |s: String| rsx! { EvalsScreen { layout: lay.clone(), state: s, fails_first: a, cost_col: b } } }
        }
    }
}

fn evals_screen_code(v: &Vals) -> String {
    let c = Code::new("EvalsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s5_key("evals", v.s("data")), "sample");
    let c = c.b("fails_first", v.b("x1")).b("cost_col", v.b("x2"));
    c.done()
}

fn evals_screen_demo() -> Demo {
    let mut c = controls_x("inset", &["Running"]);
    c.push(super::tog("x1", "Failures first", false));
    c.push(super::tog("x2", "Cost per case", false));
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
                "Nothing yet: the page says what will appear and how to start.",
                &[("data", "Empty")],
            ),
            var(
                "own",
                "Run eval pressed: every case is scored in turn, with a progress bar and Stop.",
                &[("data", "Running")],
            ),
            var(
                "x1",
                "Oxyn: failures and partials first, so the cases to fix lead.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: a cost column per case and the run's total.",
                &[("x2", "true")],
            ),
        ],
        render: evals_screen,
        code: evals_screen_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            (
                "← / → in the chart",
                "Step through the versions (Home / End: first or last)",
            ),
            (
                "Enter on a case",
                "Open the full input, expected reply and output",
            ),
            ("Enter on ↻", "Re-run that case"),
            (
                "Enter on Run eval",
                "Score every case in turn (Stop ends it)",
            ),
            ("Escape", "Close an open menu or popover"),
        ],
        a11y: vec![
            "The Evalkit sidebar is a labelled nav with aria-current on Evals.",
            "The chart is a listbox of versions with aria-activedescendant; the diff uses del and ins, and the current change is outlined.",
            "The cases are a table with column headers; each input opens its own row (aria-expanded); the run reports its progress as a progressbar.",
            "Skeletons are hidden from screen readers; every action reports what it did in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 1000 px the chart and the diff stack and the Expected column hides.",
            "Under 700 px the tiles go two by two and only input, score and re-run stay; under 640 px the sidebar slides in from the menu button.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "evals",
            "Evals",
            CAT,
            "A prompt's eval: pass rate, average score, latency and cost against the version before; pass rate across every version as a chart you can step through; the system prompt's changes as a diff to review (step through, Accept all or Reject all); and the test cases with Pass / Partial / Fail filters, a row that opens to the full reply and Re-run on each. Pick another model and Run eval scores every case on it in turn.",
            "EvalsScreen",
            "EvalsEl",
            "evals",
            "parity/buildscr.rs",
        )
    }
}

fn s6_key(slug: &str, label: &str) -> String {
    s6_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
}
// ---------- Settings · Profile ----------

fn set_profile_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-profile", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-profile"), screen: move |s: String| rsx! { ProfileSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_profile_code(v: &Vals) -> String {
    let c = Code::new("ProfileSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-profile", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_profile_demo() -> Demo {
    let mut c = controls_x("inset", &["Unsaved"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new account: blank fields and the defaults.",
                &[("data", "Empty")],
            ),
            var(
                "unsaved",
                "Opened with edits pending: the save bar counts them; Discard or Save.",
                &[("data", "Unsaved")],
            ),
            var(
                "x1",
                "Oxyn: compact rows without hints, and a card that shows how others see you.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_profile_screen,
        code: set_profile_code,
        keyboard: vec![
            (
                "Tab",
                "Sidebar, search, page, then each field in reading order",
            ),
            ("Enter in search", "Open the first page with a match"),
            ("← / → in Time format", "Move between 12-hour and 24-hour"),
            (
                "Escape",
                "Close a menu or the password form; clear the search",
            ),
        ],
        a11y: vec![
            "The settings sidebar is a labelled nav with aria-current on this page; matches while searching show as counts.",
            "Every field has a label; a field with a problem is aria-invalid with its message beside it; the password meter is a polite live region.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px rows put their control under the label and tiles go one per row.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-profile",
            "Settings · Profile",
            CAT,
            "Settings / Profile in the settings shell: picture, full name, role and email rows (email changes confirm by link), time zone, 12- or 24-hour clock and first day of the week, personal context with a counter, and sign-in: Google connected (official mark) and Add password with a strength meter. Edits collect in a save bar.",
            "ProfileSettingsScreen",
            "ProfileSettingsEl",
            "settings-profile",
            "parity/setscr.rs",
        )
    }
}

// ---------- Settings · Preferences ----------

fn set_preferences_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-preferences", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-preferences"), screen: move |s: String| rsx! { PreferencesSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_preferences_code(v: &Vals) -> String {
    let c = Code::new("PreferencesSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s(
            "state",
            &s6_key("settings-preferences", v.s("data")),
            "sample",
        );
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_preferences_demo() -> Demo {
    let mut c = controls_x("inset", &["Memory off"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new account: blank fields and the defaults.",
                &[("data", "Empty")],
            ),
            var(
                "memoff",
                "Memory switched off: kept facts are not shown or used.",
                &[("data", "Memory off")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_preferences_screen,
        code: set_preferences_code,
        keyboard: vec![
            (
                "Tab",
                "Sidebar, search, page, then each control in reading order",
            ),
            ("Enter or Space on a tile", "Turn that tool on or off"),
            ("← / → in Appearance", "Move between System, Light and Dark"),
            ("Escape", "Close a menu or the Clear all confirmation"),
        ],
        a11y: vec![
            "Appearance is a radiogroup; each tool switch is named by its tool.",
            "Clear all asks first and moves focus to its confirm button.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px rows put their control under the label and tiles go one per row.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-preferences",
            "Settings · Preferences",
            CAT,
            "Settings / Preferences: the default model (Auto shows how it picks, official model marks), language, System / Light / Dark tiles, chat tools as tiles with switches (Web search, Image generation, Code runner, Canvas), custom instructions with a counter and answer length, and memory with each kept fact, Forget and Clear all (asks first).",
            "PreferencesSettingsScreen",
            "PreferencesSettingsEl",
            "settings-preferences",
            "parity/setscr.rs",
        )
    }
}

// ---------- Settings · Notifications ----------

fn set_notifications_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-notifications", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-notifications"), screen: move |s: String| rsx! { NotificationsSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_notifications_code(v: &Vals) -> String {
    let c = Code::new("NotificationsSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s(
            "state",
            &s6_key("settings-notifications", v.s("data")),
            "sample",
        );
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_notifications_demo() -> Demo {
    let mut c = controls_x("inset", &["Quiet now"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new account: blank fields and the defaults.",
                &[("data", "Empty")],
            ),
            var(
                "quiet",
                "Quiet hours in force, with End now.",
                &[("data", "Quiet now")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_notifications_screen,
        code: set_notifications_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, digest, then the table row by row"),
            (
                "Space on a cell",
                "Turn that channel on or off for the kind",
            ),
            (
                "Enter on a column header",
                "Turn the whole column on, or off when it is all on",
            ),
            ("Escape", "Close an open menu"),
        ],
        a11y: vec![
            "The grid is a table with column and row headers; every checkbox is named by kind and channel.",
            "Day pills are toggle buttons (aria-pressed); the quiet-hours line says the window in words.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px rows put their control under the label and tiles go one per row.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-notifications",
            "Settings · Notifications",
            CAT,
            "Settings / Notifications: a daily digest (switch, time, days as toggle pills, one line that says when), a table of six kinds (approvals amber, finished green, failed red, mentions blue) by Email / Mobile / Inbox where a column header toggles the whole column, quiet hours with from–to and whether approvals still come through, and a test send.",
            "NotificationsSettingsScreen",
            "NotificationsSettingsEl",
            "settings-notifications",
            "parity/setscr.rs",
        )
    }
}

// ---------- Settings · Privacy ----------

fn set_privacy_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-privacy", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-privacy"), screen: move |s: String| rsx! { PrivacySettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_privacy_code(v: &Vals) -> String {
    let c = Code::new("PrivacySettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-privacy", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_privacy_demo() -> Demo {
    let mut c = controls_x("inset", &["Exporting"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new account: blank fields and the defaults.",
                &[("data", "Empty")],
            ),
            var(
                "exporting",
                "An export running step by step to its link.",
                &[("data", "Exporting")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_privacy_screen,
        code: set_privacy_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each control in reading order"),
            (
                "← / → in the link tabs",
                "Move between workspace and public links",
            ),
            ("Enter on Export", "Start the export"),
            ("Escape", "Close the delete confirmation"),
        ],
        a11y: vec![
            "Link scopes are a tablist with counts; each session and link action names its target.",
            "The export is a progressbar with its current step in words; Delete all chats needs the word typed.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px rows put their control under the label and tiles go one per row.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-privacy",
            "Settings · Privacy",
            CAT,
            "Settings / Privacy: switches for model training, product analytics and ad measurement, cookies and product emails, active sessions with Sign out (one, or all others), shared links in two tabs (In the workspace, Public links) with Copy link and Stop sharing, Export your data as a stepped progress, and Delete all chats (type delete to confirm).",
            "PrivacySettingsScreen",
            "PrivacySettingsEl",
            "settings-privacy",
            "parity/setscr.rs",
        )
    }
}

// ---------- Settings · General ----------

fn set_general_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-general", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-general"), screen: move |s: String| rsx! { GeneralSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_general_code(v: &Vals) -> String {
    let c = Code::new("GeneralSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-general", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_general_demo() -> Demo {
    let mut c = controls_x("inset", &["Deleting"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new account: blank fields and the defaults.",
                &[("data", "Empty")],
            ),
            var(
                "deleting",
                "Delete workspace opened: the name has to be typed in full.",
                &[("data", "Deleting")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_general_screen,
        code: set_general_code,
        keyboard: vec![
            ("Tab", "Sidebar, page, then each field in reading order"),
            ("← / → in Icon colour", "Move between the six hues"),
            ("Enter on Use …", "Take the suggested address"),
            ("Escape", "Close the delete confirmation"),
        ],
        a11y: vec![
            "The address field says why it cannot be used, or that it is available, next to it.",
            "Icon colour is a radiogroup named by hue; Delete workspace needs the exact name typed.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px rows put their control under the label and tiles go one per row.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-general",
            "Settings · General",
            CAT,
            "Settings / General: the workspace icon (initials on an identity hue you pick), name, and address that follows the name until edited, with live checks (taken, too short, characters) and a suggestion; company context with a counter; who new chats are visible to; and a danger zone where Delete workspace asks for the name.",
            "GeneralSettingsScreen",
            "GeneralSettingsEl",
            "settings-general",
            "parity/setscr.rs",
        )
    }
}

// ---------- Settings · Members ----------

fn set_members_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-members", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-members"), screen: move |s: String| rsx! { MembersSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_members_code(v: &Vals) -> String {
    let c = Code::new("MembersSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-members", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_members_demo() -> Demo {
    let mut c = controls_x("inset", &["Inviting"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new workspace: just you, nothing set up yet.",
                &[("data", "Empty")],
            ),
            var(
                "inviting",
                "Invite members opened with two addresses typed: the seat count checks them as you type.",
                &[("data", "Inviting")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_members_screen,
        code: set_members_code,
        keyboard: vec![
            (
                "Tab",
                "Tabs, search, role filter, Invite, then each row's role and menu",
            ),
            (
                "Enter in the invite field",
                "New line; Send invites sends them all",
            ),
            ("Escape", "Close a menu, the invite form or a confirmation"),
        ],
        a11y: vec![
            "The member list is a table with column headers; each role menu is named by the person.",
            "Invite says why it cannot be sent (bad address, already here, no seats) next to the field.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px tables drop their extra columns, cards stack and rows put their control under the label.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-members",
            "Settings · Members",
            CAT,
            "Settings / Members: seats at a glance (members, a seat meter, admins), search and a role filter, Invite members (several addresses at once, a role, a seat check), a member table with a role menu (the last owner cannot be demoted), team, usage meter and a menu to resend or revoke an invite or remove someone (asks first), plus a Teams tab where people are added to a team.",
            "MembersSettingsScreen",
            "MembersSettingsEl",
            "settings-members",
            "parity/setscr2.rs",
        )
    }
}

// ---------- Settings · Models ----------

fn set_models_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-models", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-models"), screen: move |s: String| rsx! { ModelsSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_models_code(v: &Vals) -> String {
    let c = Code::new("ModelsSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-models", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_models_demo() -> Demo {
    let mut c = controls_x("inset", &["New key"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new workspace: just you, nothing set up yet.",
                &[("data", "Empty")],
            ),
            var(
                "new-key",
                "A key was just rotated: the new secret shows once, with Copy and hide.",
                &[("data", "New key")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_models_screen,
        code: set_models_code,
        keyboard: vec![
            (
                "Tab",
                "Defaults, tabs, search, then each provider's models and keys",
            ),
            ("Space on a switch", "Turn a model on or off"),
            (
                "Escape",
                "Close a menu, the add-key form or the rotate confirmation",
            ),
        ],
        a11y: vec![
            "Each switch and access menu is named by its model; the provider header says how many are on.",
            "A hidden key is announced as hidden; Reveal reports whether the key shows through aria-pressed.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px tables drop their extra columns, cards stack and rows put their control under the label.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-models",
            "Settings · Models",
            CAT,
            "Settings / Models: the chat and image defaults (official marks; Auto when nothing is picked), All / Chat / Image tabs and a search, providers with their official coloured marks and how many models are on, each model with an access menu and a switch (turning off a default falls back), and API keys: add (provider, name, key checks), reveal or hide the masked key, copy, rotate (asks first, shows the new key once) and delete.",
            "ModelsSettingsScreen",
            "ModelsSettingsEl",
            "settings-models",
            "parity/setscr2.rs",
        )
    }
}

// ---------- Settings · Security ----------

fn set_security_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-security", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-security"), screen: move |s: String| rsx! { SecuritySettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_security_code(v: &Vals) -> String {
    let c = Code::new("SecuritySettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-security", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_security_demo() -> Demo {
    let mut c = controls_x("inset", &["Setting up"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new workspace: just you, nothing set up yet.",
                &[("data", "Empty")],
            ),
            var(
                "two-step-setup",
                "Two-step setup opened: scan the QR code, enter the code, save the recovery codes.",
                &[("data", "Setting up")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_security_screen,
        code: set_security_code,
        keyboard: vec![
            ("Tab", "Each section in reading order"),
            (
                "Enter in the code field",
                "Verify once six digits are typed",
            ),
            ("Escape", "Close the setup, a form or a confirmation"),
        ],
        a11y: vec![
            "The setup steps are a list that marks the current one; the QR code has a text alternative and the key is shown as text.",
            "Audit filters are a radiogroup; each event names who, what and where.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px tables drop their extra columns, cards stack and rows put their control under the label.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-security",
            "Settings · Security",
            CAT,
            "Settings / Security: your two-step sign-in set up in three steps (a QR code and its key, a six-digit code check, ten recovery codes to copy or download), recovery codes and passkeys; the workspace rules (two-step for everyone, single sign-on that needs a verified domain, session length, join by domain) with the identity-provider values to copy; email domains with their TXT record and Verify; signed-in devices with Sign out and Sign out everyone else; and an audit log with filters.",
            "SecuritySettingsScreen",
            "SecuritySettingsEl",
            "settings-security",
            "parity/setscr2.rs",
        )
    }
}

// ---------- Settings · Billing ----------

fn set_billing_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-billing", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-billing"), screen: move |s: String| rsx! { BillingSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_billing_code(v: &Vals) -> String {
    let c = Code::new("BillingSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s("state", &s6_key("settings-billing", v.s("data")), "sample");
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_billing_demo() -> Demo {
    let mut c = controls_x("inset", &["Cancelling"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new workspace: just you, nothing set up yet.",
                &[("data", "Empty")],
            ),
            var(
                "cancelling",
                "The cancel flow opened at its first step: pick a reason, see what changes, then cancel at period end.",
                &[("data", "Cancelling")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_billing_screen,
        code: set_billing_code,
        keyboard: vec![
            (
                "Tab",
                "Plan, meters, plans, then each setting, card and invoice",
            ),
            ("← / → in the interval", "Move between Monthly and Yearly"),
            (
                "Escape",
                "Close the card form, a plan confirmation or the cancel flow",
            ),
        ],
        a11y: vec![
            "Meters are labelled with used and limit as text, not only colour; the invoices table has column headers.",
            "The cancel flow is a group with numbered steps; its reasons are a radiogroup.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px tables drop their extra columns, cards stack and rows put their control under the label.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-billing",
            "Settings · Billing",
            CAT,
            "Settings / Billing: the plan and the upcoming bill, four usage meters toned by how full they are, plan cards with Monthly / Yearly (−20 %) and Upgrade or Switch (asks first, shows the new bill), the extra-usage cap and invoice emails, the payment method with a card form that checks the number and expiry, an invoices table with Download, and Cancel plan: reason, what changes (with an offer), then cancelled at period end with Resume.",
            "BillingSettingsScreen",
            "BillingSettingsEl",
            "settings-billing",
            "parity/setscr2.rs",
        )
    }
}

// ---------- Settings · Get started ----------

fn set_onboarding_screen(v: &Vals, _now: ClockMs) -> Element {
    let (_, lay, key) = frame(v);
    let st = s6_key("settings-onboarding", v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s6_states("settings-onboarding"), screen: move |s: String| rsx! { OnboardingSettingsScreen { layout: lay.clone(), state: s, compact: a, autosave: b } } }
        }
    }
}

fn set_onboarding_code(v: &Vals) -> String {
    let c = Code::new("OnboardingSettingsScreen")
        .s("layout", &low(v.s("frame")), "inset")
        .s(
            "state",
            &s6_key("settings-onboarding", v.s("data")),
            "sample",
        );
    let c = c.b("compact", v.b("x1")).b("autosave", v.b("x2"));
    c.done()
}

fn set_onboarding_demo() -> Demo {
    let mut c = controls_x("inset", &["Complete"]);
    c.push(super::tog("x1", "Compact rows", false));
    c.push(super::tog("x2", "Save at once", false));
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
                "Skeletons where the settings will land.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A new workspace: just you, nothing set up yet.",
                &[("data", "Empty")],
            ),
            var(
                "complete",
                "Every step done: the ring turns green and the checklist can be hidden.",
                &[("data", "Complete")],
            ),
            var("x1", "Oxyn: compact rows without hints.", &[("x1", "true")]),
            var(
                "x2",
                "Oxyn: every valid change is saved at once, no save bar.",
                &[("x2", "true")],
            ),
        ],
        render: set_onboarding_screen,
        code: set_onboarding_code,
        keyboard: vec![
            ("Tab", "Continue, then each step's Open and Mark done"),
            ("Enter on Open", "Go to the page for that step"),
            ("Escape", "Close a menu"),
        ],
        a11y: vec![
            "Progress is a progressbar with its value; the ring has a text alternative.",
            "Each step's Mark done is a toggle button named by the step.",
            "Every action reports what it did in a polite live region; skeletons are aria-hidden.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas; the full view lays it out at the real width.",
            "Under 640 px the settings sidebar slides in from the menu button.",
            "Under 560 px tables drop their extra columns, cards stack and rows put their control under the label.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            "settings-onboarding",
            "Settings · Get started",
            CAT,
            "Settings / Get started, the onboarding checklist: a progress ring and bar with the next step and Continue, six steps (your profile, the workspace, invite your team, models, two-step sign-in, a plan) that each open the page that does them, Mark done and Undo, and whether the checklist shows in the sidebar.",
            "OnboardingSettingsScreen",
            "OnboardingSettingsEl",
            "settings-onboarding",
            "parity/setscr2.rs",
        )
    }
}

// ---------- Onboarding wizard (S8) ----------

fn s8_key(slug: &str, label: &str) -> String {
    s8_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
}

fn onb_el(slug: &'static str, lay: String, s: String, a: bool, b: bool) -> Element {
    match slug {
        "onboarding-brand" => {
            rsx! { OnboardingBrandScreen { layout: lay, state: s, named_steps: a, form_only: b } }
        }
        "onboarding-tone" => {
            rsx! { OnboardingToneScreen { layout: lay, state: s, named_steps: a, form_only: b } }
        }
        "onboarding-ready" => {
            rsx! { OnboardingReadyScreen { layout: lay, state: s, named_steps: a, form_only: b } }
        }
        _ => {
            rsx! { OnboardingWebsiteScreen { layout: lay, state: s, named_steps: a, form_only: b } }
        }
    }
}

fn onb_screen(slug: &'static str, v: &Vals) -> Element {
    let (_, lay, key) = frame(v);
    let st = s8_key(slug, v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s8_states(slug), screen: move |s: String| onb_el(slug, lay.clone(), s, a, b) }
        }
    }
}

fn onb_code(name: &str, slug: &str, v: &Vals) -> String {
    let c = Code::new(name).s("layout", &low(v.s("frame")), "inset").s(
        "state",
        &s8_key(slug, v.s("data")),
        "sample",
    );
    let c = c.b("named_steps", v.b("x1")).b("form_only", v.b("x2"));
    c.done()
}

fn onb_website_screen(v: &Vals, _now: ClockMs) -> Element {
    onb_screen("onboarding-website", v)
}
fn onb_brand_screen(v: &Vals, _now: ClockMs) -> Element {
    onb_screen("onboarding-brand", v)
}
fn onb_tone_screen(v: &Vals, _now: ClockMs) -> Element {
    onb_screen("onboarding-tone", v)
}
fn onb_ready_screen(v: &Vals, _now: ClockMs) -> Element {
    onb_screen("onboarding-ready", v)
}
fn onb_website_code(v: &Vals) -> String {
    onb_code("OnboardingWebsiteScreen", "onboarding-website", v)
}
fn onb_brand_code(v: &Vals) -> String {
    onb_code("OnboardingBrandScreen", "onboarding-brand", v)
}
fn onb_tone_code(v: &Vals) -> String {
    onb_code("OnboardingToneScreen", "onboarding-tone", v)
}
fn onb_ready_code(v: &Vals) -> String {
    onb_code("OnboardingReadyScreen", "onboarding-ready", v)
}

#[allow(clippy::too_many_arguments)]
fn onb_demo(
    slug: &'static str,
    name: &'static str,
    summary: &'static str,
    dx: &'static str,
    gp: &'static str,
    own: (&'static str, &'static str, &'static str),
    render: fn(&Vals, ClockMs) -> Element,
    code: fn(&Vals) -> String,
    keyboard: Vec<(&'static str, &'static str)>,
) -> Demo {
    let (own_key, own_label, own_note) = own;
    let mut c = controls_x("inset", &[own_label]);
    c.push(super::tog("x1", "Named steps", false));
    c.push(super::tog("x2", "Form only", false));
    Demo {
        controls: c,
        variants: vec![
            var(
                "sample",
                "Sample data, the dashboard preview as an inset panel.",
                &[],
            ),
            var(
                "flat",
                "The same step with the preview edge to edge.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons in the form and the preview.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A first visit: nothing filled in, Continue explains what is missing.",
                &[("data", "Empty")],
            ),
            var(own_key, own_note, &[("data", own_label)]),
            var(
                "x1",
                "Oxyn: each step named under its dash; done steps can be revisited.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: the form alone, centred, without the preview.",
                &[("x2", "true")],
            ),
        ],
        render,
        code,
        keyboard,
        a11y: vec![
            "The steps are an ordered list; the current one has aria-current=step and each is named “Step n of 4: …”.",
            "Fields have labels; a problem sets aria-invalid and shows its message as an alert beside the field.",
            "Colour, mark, tone and length are radiogroups; the handover is a switch.",
            "The preview is decorative (aria-hidden); everything it shows is also in the form. Every action reports in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas: a 720 px form column and a 720 px preview.",
            "Under 960 px the preview leaves the page and opens over the form from Preview in the top bar.",
            "Under 620 px the signed-in address hides, marks go two by two and the summary tightens.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(slug, name, CAT, summary, dx, gp, slug, "parity/onbscr.rs")
    }
}

fn onb_website_demo() -> Demo {
    onb_demo(
        "onboarding-website",
        "Onboarding · Website",
        "Onboarding / Website, step 1 of 4: the sites the assistant learns from. A field with Add and Continue cleans what you type (no https://, www. or paths) and checks it; each site is read with progress in the preview's Knowledge table, a private address fails with Retry, and Continue needs one readable site.",
        "OnboardingWebsiteScreen",
        "OnboardingWebsiteEl",
        (
            "fetching",
            "Reading site",
            "Two more sites being read: progress, then pages learned.",
        ),
        onb_website_screen,
        onb_website_code,
        vec![
            ("Enter", "Add the address; on an empty field, Continue"),
            ("Escape", "Clear the field"),
            ("Tab", "Add, Continue, then each site's Retry and Remove"),
        ],
    )
}

fn onb_brand_demo() -> Demo {
    onb_demo(
        "onboarding-brand",
        "Onboarding · Brand",
        "Onboarding / Brand, step 2 of 4: the company's name plus the shorter names people use for it, a brand colour (eight swatches or a checked hex, with a warning when it would vanish on white or in dark mode) and a mark (initials, square, round or an uploaded logo). The preview glides to the assistant card in the new colour.",
        "OnboardingBrandScreen",
        "OnboardingBrandEl",
        (
            "custom",
            "Own colour",
            "A typed hex colour and an uploaded logo.",
        ),
        onb_brand_screen,
        onb_brand_code,
        vec![
            ("Enter", "Add the name"),
            ("Arrow keys", "Move through swatches and marks"),
            ("Tab", "Remove, colours, hex, marks, Continue"),
        ],
    )
}

fn onb_tone_demo() -> Demo {
    onb_demo(
        "onboarding-tone",
        "Onboarding · Tone",
        "Onboarding / Tone, step 3 of 4: the assistant's name (checked, with a counter), tone (Friendly, Neutral, Formal), reply length (Short, Balanced, Detailed) and Hand over to a person with an address. The preview's test chat re-types its reply in the new voice and shows who it hands over to.",
        "OnboardingToneScreen",
        "OnboardingToneEl",
        (
            "testing",
            "Test reply",
            "The test chat typing its reply in the chosen voice.",
        ),
        onb_tone_screen,
        onb_tone_code,
        vec![
            (
                "Tab",
                "Name, tone, length, handover switch and address, Continue",
            ),
            ("Space", "Turn the handover on or off"),
        ],
    )
}

fn onb_ready_demo() -> Demo {
    onb_demo(
        "onboarding-ready",
        "Onboarding · Ready",
        "Onboarding / Ready, step 4 of 4: a summary of what the assistant learns from, its brand, name, voice and handover, each with Edit (a missing step shows Add and blocks), then Open dashboard turns the assistant from Draft to Live in the preview.",
        "OnboardingReadyScreen",
        "OnboardingReadyEl",
        (
            "live",
            "Live",
            "Published: the dashboard shows the assistant Live.",
        ),
        onb_ready_screen,
        onb_ready_code,
        vec![
            ("Tab", "Each Edit, then Open dashboard"),
            ("Enter on Edit", "Go back to that step"),
        ],
    )
}

// ---------- Agent apps I: Taskdeck (S9) ----------

fn s9_key(slug: &str, label: &str) -> String {
    s9_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
}

fn td_el(slug: &'static str, lay: String, s: String, a: bool, b: bool) -> Element {
    match slug {
        "taskdeck-task" => {
            rsx! { TaskdeckTaskScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        "taskdeck-board" => {
            rsx! { TaskdeckBoardScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        "taskdeck-plugins" => {
            rsx! { TaskdeckPluginsScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        "taskdeck-scheduled" => {
            rsx! { TaskdeckScheduledScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        "taskdeck-sites" => {
            rsx! { TaskdeckSitesScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        _ => {
            rsx! { TaskdeckStartScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
    }
}

fn td_screen(slug: &'static str, v: &Vals) -> Element {
    let (_, lay, key) = frame(v);
    let st = s9_key(slug, v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: s9_states(slug), screen: move |s: String| td_el(slug, lay.clone(), s, a, b) }
        }
    }
}

fn td_code(name: &str, slug: &str, v: &Vals) -> String {
    let c = Code::new(name).s("layout", &low(v.s("frame")), "inset").s(
        "state",
        &s9_key(slug, v.s("data")),
        "sample",
    );
    let c = c.b("rail", v.b("x1")).b("sidebar_hidden", v.b("x2"));
    c.done()
}

fn td_start_screen(v: &Vals, _now: ClockMs) -> Element {
    td_screen("taskdeck-start", v)
}
fn td_task_screen(v: &Vals, _now: ClockMs) -> Element {
    td_screen("taskdeck-task", v)
}
fn td_board_screen(v: &Vals, _now: ClockMs) -> Element {
    td_screen("taskdeck-board", v)
}
fn td_start_code(v: &Vals) -> String {
    td_code("TaskdeckStartScreen", "taskdeck-start", v)
}
fn td_task_code(v: &Vals) -> String {
    td_code("TaskdeckTaskScreen", "taskdeck-task", v)
}
fn td_board_code(v: &Vals) -> String {
    td_code("TaskdeckBoardScreen", "taskdeck-board", v)
}
fn td_plugins_screen(v: &Vals, _now: ClockMs) -> Element {
    td_screen("taskdeck-plugins", v)
}
fn td_scheduled_screen(v: &Vals, _now: ClockMs) -> Element {
    td_screen("taskdeck-scheduled", v)
}
fn td_sites_screen(v: &Vals, _now: ClockMs) -> Element {
    td_screen("taskdeck-sites", v)
}
fn td_plugins_code(v: &Vals) -> String {
    td_code("TaskdeckPluginsScreen", "taskdeck-plugins", v)
}
fn td_scheduled_code(v: &Vals) -> String {
    td_code("TaskdeckScheduledScreen", "taskdeck-scheduled", v)
}
fn td_sites_code(v: &Vals) -> String {
    td_code("TaskdeckSitesScreen", "taskdeck-sites", v)
}

#[allow(clippy::too_many_arguments)]
fn td_demo(
    slug: &'static str,
    name: &'static str,
    summary: &'static str,
    dx: &'static str,
    gp: &'static str,
    own: (&'static str, &'static str, &'static str),
    render: fn(&Vals, ClockMs) -> Element,
    code: fn(&Vals) -> String,
    keyboard: Vec<(&'static str, &'static str)>,
) -> Demo {
    let (own_key, own_label, own_note) = own;
    let mut c = controls_x("inset", &[own_label]);
    c.push(super::tog("x1", "Icon rail", false));
    c.push(super::tog("x2", "Sidebar hidden", false));
    Demo {
        controls: c,
        variants: vec![
            var(
                "sample",
                "Sample data, the main area as an inset card.",
                &[],
            ),
            var(
                "flat",
                "The same view with the main area edge to edge.",
                &[("frame", "Flat")],
            ),
            var(
                "loading",
                "Skeletons in the sidebar and the main area.",
                &[("data", "Loading")],
            ),
            var(
                "empty",
                "A first run: no tasks or projects yet.",
                &[("data", "Empty")],
            ),
            var(own_key, own_note, &[("data", own_label)]),
            var(
                "x1",
                "Oxyn: the sidebar as a 56 px icon rail; the top bar expands it.",
                &[("x1", "true")],
            ),
            var(
                "x2",
                "Oxyn: the sidebar hidden for focus; the top bar brings it back.",
                &[("x2", "true")],
            ),
        ],
        render,
        code,
        keyboard,
        a11y: vec![
            "The sidebar is a nav landmark; the open view and the open task carry aria-current=page.",
            "Local / Cloud, run mode, model and Board / List are radio groups or menus with menuitemradio; menus close on Escape.",
            "Steps are an ordered list; a step waiting for approval is a labelled group with Allow once, Always in this folder and Deny.",
            "The side panel is a tablist (arrow keys move between Artifacts, Files, Terminal and Browser). Every action reports in a polite live region.",
        ],
        responsive: vec![
            "Authored on the 1440 × 900 canvas: a 264 px sidebar, the main area and a 340 px side panel on the task.",
            "Under 960 px the sidebar becomes a drawer from the top bar and the side panel opens over the task.",
            "Under 620 px the composer chips shorten, the board scrolls sideways and the list keeps title and status.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(slug, name, CAT, summary, dx, gp, slug, "parity/taskdeck.rs")
    }
}

fn td_start_demo() -> Demo {
    td_demo(
        "taskdeck-start",
        "Taskdeck · Start",
        "Agent apps / Taskdeck: a desktop agent app's start screen. A sidebar with New task, Tasks, Plugins & skills, a Local / Cloud switch, projects and recent tasks, and the account row with the inbox bell; in the middle the task composer with + (files, folder, plugins, skills), run mode, model, dictation and send, the folder it works in and four starter ideas. Sending checks the prompt, starts the task and opens it.",
        "TaskdeckStartScreen",
        "TaskdeckStartEl",
        (
            "cloud",
            "Cloud",
            "Cloud mode: cloud tasks in the sidebar and a hand-off headline.",
        ),
        td_start_screen,
        td_start_code,
        vec![
            ("Enter", "Start the task (Shift + Enter for a new line)"),
            ("Escape", "Close a menu or the search field"),
            ("Tab", "Composer, its chips, then the ideas"),
        ],
    )
}

fn td_task_demo() -> Demo {
    td_demo(
        "taskdeck-task",
        "Taskdeck · Task",
        "Agent apps / Taskdeck: a running task. The prompt, then a step timeline that advances on its own: each step shows its tool call and output, a script waits for approval (Allow once, Always in this folder, Deny), Pause and Resume, follow-ups queue for the next step; the side panel has Artifacts (open the sheet, chart or note), Files, Terminal and Browser.",
        "TaskdeckTaskScreen",
        "TaskdeckTaskEl",
        (
            "approval",
            "Needs you",
            "The task stopped on an approval for a terminal command.",
        ),
        td_task_screen,
        td_task_code,
        vec![
            ("Arrow keys", "Move between the side panel's tabs"),
            ("Enter", "Send the follow-up"),
            ("Tab", "Steps, the approval buttons, Pause, the panel"),
        ],
    )
}

fn td_board_demo() -> Demo {
    td_demo(
        "taskdeck-board",
        "Taskdeck · Board",
        "Agent apps / Taskdeck: every task by status, Queued, Running, Needs you and Done, as a board or a list for the current Local or Cloud side. Cards show the project, progress and age; their menu opens, runs, re-queues, finishes or deletes them, and a click opens the task.",
        "TaskdeckBoardScreen",
        "TaskdeckBoardEl",
        (
            "menu",
            "Account menu",
            "The account menu open: Usage, Daily check-in, Feedback & help.",
        ),
        td_board_screen,
        td_board_code,
        vec![
            ("Tab", "Cards and their menus, column by column"),
            ("Escape", "Close a card menu or the account menu"),
            ("→ in the account menu", "Open Usage or Feedback & help"),
        ],
    )
}

fn td_plugins_demo() -> Demo {
    td_demo(
        "taskdeck-plugins",
        "Taskdeck · Plugins & skills",
        "Agent apps / Taskdeck: the Plugins & skills page. Plugins: a searchable catalogue by category (Browse, Work, Code, Create) with official marks, Install, an on/off switch, Details (what the plugin may do) and Remove. Skills: bundled and your own saved instructions with on/off, delete and New skill (name and when it applies, checked before saving).",
        "TaskdeckPluginsScreen",
        "TaskdeckPluginsEl",
        (
            "skills",
            "Skills tab",
            "The Skills tab: bundled and your own skills.",
        ),
        td_plugins_screen,
        td_plugins_code,
        vec![
            (
                "Tab",
                "Search, categories, then each plugin's switch and buttons",
            ),
            ("Space", "Turn a plugin or skill on or off"),
            ("Escape", "Close the New skill form"),
        ],
    )
}

fn td_scheduled_demo() -> Demo {
    td_demo(
        "taskdeck-scheduled",
        "Taskdeck · Scheduled",
        "Agent apps / Taskdeck: tasks that run on a schedule. Each row shows where it runs, how often, the next run and how the last one went; Run now runs it (Running, then Succeeded), Pause / Resume, Edit opens the form in place and Delete removes it. New schedule takes a name, what it does, how often and a 24-hour time, checked before saving.",
        "TaskdeckScheduledScreen",
        "TaskdeckScheduledEl",
        (
            "editing",
            "Editing",
            "A schedule open for editing in place.",
        ),
        td_scheduled_screen,
        td_scheduled_code,
        vec![
            ("Tab", "Each row's Run now, Pause, Edit and Delete"),
            ("Escape", "Close the schedule form"),
        ],
    )
}

fn td_sites_demo() -> Demo {
    td_demo(
        "taskdeck-sites",
        "Taskdeck · Sites",
        "Agent apps / Taskdeck: sites that tasks built and deployed. Cards show a thumbnail, the address, status (Live, Deploying, Deploy failed), the last deploy and the task that made it; Preview opens the page beside the list with its address bar, and Redeploy deploys again (Deploying, then Live).",
        "TaskdeckSitesScreen",
        "TaskdeckSitesEl",
        (
            "preview",
            "Preview",
            "A site's preview open beside the list.",
        ),
        td_sites_screen,
        td_sites_code,
        vec![
            ("Tab", "Each site's thumbnail, Preview and Redeploy"),
            ("Enter", "Open or close the preview"),
        ],
    )
}

// ---------- Agent apps II: Command Center and Session Studio (S10) ----------

/// The state bar of an Agent apps screen (S10 and S11).
fn ag_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    match slug {
        "fleet-window" => s11_fleet_states(),
        "workspace-coder" => s11_coder_states(),
        _ => s10_states(slug),
    }
}

fn s10_key(slug: &str, label: &str) -> String {
    ag_states(slug)
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(label))
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| low(label))
}

fn ag_el(slug: &'static str, lay: String, s: String, a: bool, b: bool) -> Element {
    match slug {
        "session-studio" => {
            rsx! { SessionStudioScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        "fleet-window" => {
            rsx! { FleetWindowScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        "workspace-coder" => {
            rsx! { WorkspaceCoderScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } }
        }
        _ => rsx! { CommandCenterScreen { layout: lay, state: s, rail: a, sidebar_hidden: b } },
    }
}

fn ag_screen(slug: &'static str, v: &Vals) -> Element {
    let (_, lay, key) = frame(v);
    let st = s10_key(slug, v.s("data"));
    let key = format!("{key}-{}-{}", v.b("x1"), v.b("x2"));
    let (a, b) = (v.b("x1"), v.b("x2"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:100%;min-width:0",
            ScreenStage { state: st, states: ag_states(slug), screen: move |s: String| ag_el(slug, lay.clone(), s, a, b) }
        }
    }
}

fn ag_code(name: &str, slug: &str, v: &Vals) -> String {
    let c = Code::new(name).s("layout", &low(v.s("frame")), "inset").s(
        "state",
        &s10_key(slug, v.s("data")),
        "sample",
    );
    let c = c.b("rail", v.b("x1")).b("sidebar_hidden", v.b("x2"));
    c.done()
}

fn cc_screen(v: &Vals, _now: ClockMs) -> Element {
    ag_screen("command-center", v)
}
fn ss_screen(v: &Vals, _now: ClockMs) -> Element {
    ag_screen("session-studio", v)
}
fn cc_code(v: &Vals) -> String {
    ag_code("CommandCenterScreen", "command-center", v)
}
fn ss_code(v: &Vals) -> String {
    ag_code("SessionStudioScreen", "session-studio", v)
}

#[allow(clippy::too_many_arguments)]
fn ag_demo(
    slug: &'static str,
    name: &'static str,
    summary: &'static str,
    dx: &'static str,
    gp: &'static str,
    own: (&'static str, &'static str, &'static str),
    render: fn(&Vals, ClockMs) -> Element,
    code: fn(&Vals) -> String,
    keyboard: Vec<(&'static str, &'static str)>,
    a11y: Vec<&'static str>,
    more: &[(&'static str, &'static str, &'static str)],
) -> Demo {
    let (own_key, own_label, own_note) = own;
    let mut labels = vec![own_label];
    labels.extend(more.iter().map(|m| m.1));
    let mut c = controls_x("inset", &labels);
    c.push(super::tog("x1", "Icon rail", false));
    c.push(super::tog("x2", "Sidebar hidden", false));
    let mut variants = vec![
        var(
            "sample",
            "Sample data, the main area as an inset card.",
            &[],
        ),
        var(
            "flat",
            "The same view with the main area edge to edge.",
            &[("frame", "Flat")],
        ),
        var(
            "loading",
            "Skeletons in the sidebar, the conversation and the pane.",
            &[("data", "Loading")],
        ),
        var(
            "empty",
            "A first run: nothing yet, the composer front and centre.",
            &[("data", "Empty")],
        ),
        var(own_key, own_note, &[("data", own_label)]),
        var(
            "x1",
            "Oxyn: the sidebar as a 56 px icon rail; the top bar expands it.",
            &[("x1", "true")],
        ),
        var(
            "x2",
            "Oxyn: the sidebar hidden for focus; the top bar brings it back.",
            &[("x2", "true")],
        ),
    ];
    for (k, l, n) in more.iter().copied() {
        variants.push(var(k, n, &[("data", l)]));
    }
    Demo {
        controls: c,
        variants,
        render,
        code,
        keyboard,
        a11y,
        responsive: vec![
            "Authored on the 1440 × 900 canvas: a 272 px sidebar, the conversation and a 440 px pane (380 px under 1180).",
            "Under 960 px the sidebar becomes a drawer from the top bar and the pane opens over the conversation from its top-bar button.",
            "Under 620 px the chips shorten and the composer and pane keep their actions; labels ellipsise.",
        ],
        height: 240,
        live: false,
        align: Align::Start,
        ..mk(
            slug,
            name,
            CAT,
            summary,
            dx,
            gp,
            slug,
            match slug {
                "fleet-window" => "parity/fleetwin.rs",
                "workspace-coder" => "parity/wscoder.rs",
                "command-board" => "parity/cmdboard.rs",
                _ => "parity/cmdstudio.rs",
            },
        )
    }
}

fn cc_demo() -> Demo {
    ag_demo(
        "command-center",
        "Command Center",
        "Agent apps / Command Center: a desktop coding-agent app in the threads-and-review pattern. Projects and their threads in the sidebar (status, lines changed), Automations, a running thread with its work log that finishes on its own, a pane with Review (each thread its own changes), Terminal tabs and a Browser; the review (files, hunks, Accept or Revert per hunk and file, Accept all), a Local / Worktree / Cloud switch with the branch, a task queue that fills while the thread runs, and a composer with model and reasoning pickers and Stop.",
        "CommandCenterScreen",
        "CommandCenterEl",
        (
            "worktree",
            "Worktree",
            "The thread finished in a worktree on its own branch, with Hand off to Local and the changes to review.",
        ),
        cc_screen,
        cc_code,
        vec![
            (
                "Enter",
                "Send, or queue while the thread runs (Shift + Enter for a new line)",
            ),
            ("Escape", "Close a picker menu"),
            (
                "Tab",
                "Threads, the conversation, the queue, the composer, then the review pane",
            ),
        ],
        vec![
            "The sidebar is a nav landmark; the open thread carries aria-current=page; project groups are disclosure buttons.",
            "Work in, model and reasoning are menus of menuitemradio that close on Escape.",
            "The review pane is a labelled complementary region; each file is a section with Accept and Revert buttons naming the file and hunk.",
            "The queue is an ordered list with Move up and Remove; every action reports in a polite live region.",
        ],
        &[
            (
                "automations",
                "Automations",
                "Automations: threads that start on a schedule or an event, with Run now, on/off and New automation.",
            ),
            (
                "terminal",
                "Terminals",
                "The pane on Terminal: tabs for the test run and a shell, a new tab, close, and a prompt that runs commands.",
            ),
            (
                "browser",
                "Browser",
                "The pane on Browser: back, reload, an address bar and the app's page.",
            ),
        ],
    )
}

fn ss_demo() -> Demo {
    ag_demo(
        "session-studio",
        "Session Studio",
        "Agent apps / Session Studio: a desktop coding-agent app in the session-and-tools pattern. A session list; a chat whose turns show Read, Edit (inline diff) and Bash blocks; permission prompts (Allow once, Always allow, Deny, keys 1 2 3); a permission mode (Ask before edits, Accept edits, Plan mode; Shift + Tab cycles) where Plan mode answers with a plan to approve that becomes the todo list; a todo list with progress; and side panes (Diff with each session's own changes, a side chat, Context) that collapse and can be dragged into a new order.",
        "SessionStudioScreen",
        "SessionStudioEl",
        (
            "permission",
            "Permission",
            "The session stopped to ask before running the test command.",
        ),
        ss_screen,
        ss_code,
        vec![
            (
                "Shift + Tab",
                "Cycle the permission mode (Ask, Accept edits, Plan)",
            ),
            ("Enter", "Send (Shift + Enter for a new line)"),
            (
                "Tab",
                "Sessions, the blocks and their buttons, todos, the composer, the diff",
            ),
        ],
        vec![
            "The sidebar is a nav landmark; the open session carries aria-current=page.",
            "A permission prompt and a plan are labelled groups; their buttons say what they do.",
            "Todos are real checkboxes; the running one shows a spinner and the waiting one an amber dot.",
            "Permission mode and model are menus of menuitemradio; every action reports in a polite live region.",
        ],
        &[
            (
                "chat",
                "Side chat",
                "The side chat pane under the diff: ask about the session without interrupting it.",
            ),
            (
                "arrange",
                "Rearrange",
                "Rearrange: drag the Diff, Side chat and Context panes by their handles, or move them with the arrow buttons.",
            ),
        ],
    )
}

// ---------- Agent apps III: Fleet Window, Workspace Coder, Command Board (S11) ----------

fn fw_screen(v: &Vals, _now: ClockMs) -> Element {
    ag_screen("fleet-window", v)
}
fn fw_code(v: &Vals) -> String {
    ag_code("FleetWindowScreen", "fleet-window", v)
}

fn fw_demo() -> Demo {
    ag_demo(
        "fleet-window",
        "Fleet Window",
        "Agent apps / Fleet Window: a desktop coding-agent app in the agents-window pattern. Agents in the sidebar, each on its own worktree with its model mark, status and lines changed; the open agent's steps finishing on their own into a Changes pane (accept or revert per hunk and file); a composer with Agent / Plan / Ask and a multi-model picker that runs one prompt on several models at once; Compare ranks the attempts by tests, size and time and applies one (the rest are discarded, with Undo); Plan mode returns a checklist to Build.",
        "FleetWindowScreen",
        "FleetWindowEl",
        (
            "compare",
            "Compare",
            "Three models ran the same prompt in parallel: tests, lines changed and time side by side, a suggested pick, View changes and Apply.",
        ),
        fw_screen,
        fw_code,
        vec![
            ("Enter", "Send (Shift + Enter for a new line)"),
            ("Escape", "Close the mode, model or account menu"),
            (
                "Tab",
                "Agents, the conversation, the composer, then the Changes pane",
            ),
        ],
        vec![
            "The sidebar is a nav landmark; the open agent carries aria-current=page; a parallel run is a button that opens Compare.",
            "Mode is a menu of menuitemradio; models are menuitemcheckbox, so several can be on at once.",
            "Compare is a list of attempts; Apply and View changes name the model; Changes is a labelled complementary region.",
            "Plan steps are real checkboxes; every action reports in a polite live region.",
        ],
        &[(
            "plan",
            "Plan",
            "Plan mode: the agent answered with a checklist to tick, Build or Dismiss.",
        )],
    )
}

fn wc_screen(v: &Vals, _now: ClockMs) -> Element {
    ag_screen("workspace-coder", v)
}
fn wc_code(v: &Vals) -> String {
    ag_code("WorkspaceCoderScreen", "workspace-coder", v)
}

fn wc_demo() -> Demo {
    ag_demo(
        "workspace-coder",
        "Workspace Coder",
        "Agent apps / Workspace Coder: a desktop agent app in the chat-plus-computer pattern. Chats by day in the sidebar; the agent's progress list and tool log while it works; a Workspace pane that is its own computer (Files with a file viewer, Terminal, Browser, Preview); a finished website to preview at desktop or phone width and publish to a link. Mode (Chat, Researcher, Workspace), Thinking and model in the composer.",
        "WorkspaceCoderScreen",
        "WorkspaceCoderEl",
        (
            "preview",
            "Preview",
            "The task finished: the Workspace pane shows the built page with Desktop / Phone and Publish.",
        ),
        wc_screen,
        wc_code,
        vec![
            ("Enter", "Send (Shift + Enter for a new line)"),
            ("← →", "Move between the Workspace tabs"),
            ("Escape", "Close the mode, model or account menu"),
        ],
        vec![
            "The sidebar is a nav landmark; the open chat carries aria-current=page.",
            "Progress is a labelled group with a disclosure; the tool log is an ordered list.",
            "The Workspace is a labelled complementary region with a tablist (roving tabindex, arrow keys).",
            "Desktop / Phone are toggle buttons (aria-pressed); every action reports in a polite live region.",
        ],
        &[(
            "files",
            "Files",
            "The Workspace pane on Files: the file list and a viewer with line numbers.",
        )],
    )
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
        inbox_screen_demo(),
        inbox_board_demo(),
        tasks_screen_demo(),
        activity_screen_demo(),
        memory_screen_demo(),
        agents_screen_demo(),
        skills_screen_demo(),
        schedules_screen_demo(),
        usage_screen_demo(),
        permissions_screen_demo(),
        run_detail_screen_demo(),
        evals_screen_demo(),
        set_profile_demo(),
        set_preferences_demo(),
        set_notifications_demo(),
        set_privacy_demo(),
        set_general_demo(),
        set_members_demo(),
        set_models_demo(),
        set_security_demo(),
        set_billing_demo(),
        set_onboarding_demo(),
        onb_website_demo(),
        onb_brand_demo(),
        onb_tone_demo(),
        onb_ready_demo(),
        td_start_demo(),
        td_task_demo(),
        td_board_demo(),
        td_plugins_demo(),
        td_scheduled_demo(),
        td_sites_demo(),
        cc_demo(),
        ss_demo(),
        fw_demo(),
        wc_demo(),
    ]
}
