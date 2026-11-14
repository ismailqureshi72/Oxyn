//! App shells and screens (plan #27, chunk 4): sidebars, the rail, the inbox, app shells, the AI
//! side panel, terminal tabs, Beautiful UI's Sidebar Nav, the harness and the seven affectUI AI
//! Chat screens. Every shell renders inside the demo app frame.

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{ClockMs, chat::thread::Starter, shell::Screen};
use oxyn_dioxus::{
    AiSidePanel, AppShell, ApprovalCard, ChatShell, Harness, InboxSidebar, MessageBubble, PlanCard,
    Sidebar, SidebarNav, SidebarRail, StarterPrompts, StatusIndicator, StreamingMarkdown,
    SubagentTree, TerminalTabs, TodoList, ToolCallRow, ToolStack, WorkspaceSidebar,
};

use super::{Code, mk, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

fn key(v: &Vals, keys: &[&str]) -> String {
    keys.iter().map(|k| v.s(k)).collect::<Vec<_>>().join("|")
}

fn s(x: &str) -> String {
    x.to_string()
}

/// The demo app frame (window dots and a title) around a shell.
fn app_frame(title: &str, body: Element) -> Element {
    rsx! {
        div { class: "oxd-app oxs-frame",
            div { class: "oxd-app-bar",
                span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" }
                span { class: "oxd-app-title", "{title}" }
                span { class: "oxd-app-sp" }
            }
            div { class: "oxs-body", {body} }
        }
    }
}

fn page_body() -> Element {
    rsx! {
        div { class: "oxs-page",
            h3 { "Checkout redesign" }
            p { "Three flows to review before Thursday. The address step still drops 14% of mobile sessions." }
            div { class: "oxs-cards",
                div { class: "oxs-card", b { "Address" } small { "In review · Marcus" } }
                div { class: "oxs-card", b { "Payment" } small { "Ready · Emma" } }
                div { class: "oxs-card", b { "Receipt" } small { "Draft · Ava" } }
            }
        }
    }
}

// ───────────────────────────── Sidebar ──

fn sidebar(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["collapsed", "active", "height"]);
    rsx! {
        {app_frame("Northline", rsx! {
            div { key: "{k}", class: "oxs-row",
                Sidebar { collapsed: v.b("collapsed"), active: v.string("active"), height: v.f("height") as f32 }
                {page_body()}
            }
        })}
    }
}

fn sidebar_code(v: &Vals) -> String {
    Code::new("Sidebar")
        .raw("sections", "sections()")
        .s("active", v.s("active"), "checkout")
        .b("collapsed", v.b("collapsed"))
        .raw("on_select", "move |id: String| open(id)")
        .done()
}

// ───────────────────────────── Workspace sidebar ──

fn workspace(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["menu", "query", "collapsed", "storage"]);
    rsx! {
        {app_frame("Northwind", rsx! {
            div { key: "{k}", class: "oxs-row",
                WorkspaceSidebar { menu_open: v.b("menu"), query: v.string("query"), collapsed: v.b("collapsed"), storage: v.f("storage") as f32 / 100.0, height: 500.0 }
                {page_body()}
            }
        })}
    }
}

fn workspace_code(v: &Vals) -> String {
    Code::new("WorkspaceSidebar")
        .raw("workspaces", "workspaces()")
        .raw("favorites", "favorites")
        .raw("projects", "projects")
        .raw("storage", format!("{:.2}", v.f("storage") / 100.0))
        .raw("on_navigate", "move |name: String| open(name)")
        .done()
}

// ───────────────────────────── Sidebar rail ──

fn rail(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["collapsed", "width", "transition"]);
    rsx! {
        {app_frame("Northline", rsx! {
            div { key: "{k}", style: "width:100%",
                SidebarRail { collapsed: v.b("collapsed"), width: v.f("width") as f32, transition: v.string("transition"), height: 470.0, {page_body()} }
            }
        })}
    }
}

fn rail_code(v: &Vals) -> String {
    Code::new("SidebarRail")
        .raw("sections", "sections()")
        .raw("width", format!("{:.0}.0", v.f("width")))
        .b("collapsed", v.b("collapsed"))
        .s("transition", v.s("transition"), "ease")
        .raw("on_resize", "move |w: u32| save_width(w)")
        .raw("children", "rsx! { Page {} }")
        .done()
}

// ───────────────────────────── Inbox ──

fn inbox(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["collapsed", "composing"]);
    rsx! {
        {app_frame("Fieldnote Mail", rsx! {
            div { key: "{k}", style: "width:100%",
                InboxSidebar { collapsed: v.b("collapsed"), composing: v.b("composing"), height: 500.0 }
            }
        })}
    }
}

fn inbox_code(v: &Vals) -> String {
    Code::new("InboxSidebar")
        .raw("inbox", "use_signal(sample_inbox)()")
        .b("collapsed", v.b("collapsed"))
        .b("composing", v.b("composing"))
        .done()
}

// ───────────────────────────── App shell ──

fn appshell(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["variant", "rail"]);
    let variant = v.string("variant");
    let railed = v.b("rail");
    rsx! {
        {app_frame("Northline", rsx! {
            div { key: "{k}", style: "width:100%",
                AppShell { variant: variant.clone(), height: 480.0,
                    sidebar: rsx! { Sidebar { collapsed: railed, height: 480.0 } },
                    {page_body()}
                }
            }
        })}
    }
}

fn appshell_code(v: &Vals) -> String {
    Code::new("AppShell")
        .s("variant", v.s("variant"), "inset")
        .raw("title", q("Checkout redesign"))
        .raw("sidebar", "rsx! { Sidebar {} }")
        .raw("children", "rsx! { Page {} }")
        .done()
}

// ───────────────────────────── AI side panel ──

fn aipanel(v: &Vals, now: ClockMs) -> Element {
    let k = key(v, &["open", "proposed"]);
    rsx! {
        {app_frame("Team workspaces · Spec", rsx! {
            div { key: "{k}", style: "width:100%",
                AiSidePanel { now_ms: now.0, open: v.b("open"), proposed: v.b("proposed"), height: 480.0 }
            }
        })}
    }
}

fn aipanel_code(v: &Vals) -> String {
    Code::new("AiSidePanel")
        .raw("now_ms", "clock.now()")
        .raw("page", "blocks")
        .b("open", v.b("open"))
        .raw("on_apply", "move |block: String| save(block)")
        .done()
}

// ───────────────────────────── Terminal tabs ──

fn term(v: &Vals, now: ClockMs) -> Element {
    let k = key(v, &["tabs", "activity"]);
    let titles: Vec<String> = ["dev server", "cargo test", "zsh", "logs", "db shell"]
        .iter()
        .take(v.u("tabs"))
        .map(|x| s(x))
        .collect();
    rsx! {
        {app_frame("atlas · terminal", rsx! {
            div { key: "{k}", style: "width:100%",
                TerminalTabs { now_ms: now.0, titles, activity_ms: v.f("activity") as u64, height: 320.0 }
            }
        })}
    }
}

fn term_code(v: &Vals) -> String {
    Code::new("TerminalTabs")
        .raw("now_ms", "clock.now()")
        .raw(
            "titles",
            "vec![\"dev server\".into(), \"cargo test\".into()]",
        )
        .raw("activity_ms", format!("{}", v.f("activity") as u64))
        .done()
}

// ───────────────────────────── Sidebar Nav (Beautiful UI) ──

fn snav(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["compact", "active"]);
    rsx! {
        {app_frame("Northline", rsx! {
            div { key: "{k}", class: "oxs-row pad",
                SidebarNav { compact: v.b("compact"), active: v.string("active") }
                {page_body()}
            }
        })}
    }
}

fn snav_code(v: &Vals) -> String {
    Code::new("SidebarNav")
        .raw("sections", "sections()")
        .s("active", v.s("active"), "home")
        .b("compact", v.b("compact"))
        .done()
}

// ───────────────────────────── Harness ──

fn harness(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["viewport", "dark", "variant"]);
    let fork = oxyn_core::shell::fork_url(
        "/components/shells",
        "app-shell",
        &[("variant", v.s("variant"))],
    );
    let variant = v.string("variant");
    rsx! {
        div { key: "{k}", style: "width:100%",
            Harness { title: "App shell", viewport: v.string("viewport"), dark: v.b("dark"), fork_href: fork,
                AppShell { variant: variant.clone(), height: 380.0, sidebar: rsx! { Sidebar { height: 380.0 } }, {page_body()} }
            }
        }
    }
}

fn harness_code(v: &Vals) -> String {
    Code::new("Harness")
        .raw("title", q("App shell"))
        .s("viewport", v.s("viewport"), "desktop")
        .b("dark", v.b("dark"))
        .raw("fork_href", "fork_url(base, \"app-shell\", &vals)")
        .raw("children", "rsx! { AppShell { .. } }")
        .done()
}

// ───────────────────────────── AI Chat screens ──

fn threads() -> Vec<(String, String)> {
    vec![
        (
            s("Fix the refresh-token race"),
            s("Atlas · running · 4 min"),
        ),
        (s("Release notes for v0.3"), s("Atlas · done · 1 h")),
        (s("Audit the billing webhooks"), s("Scout · waiting on you")),
        (s("Onboarding copy pass"), s("Quill · done · yesterday")),
    ]
}

fn starter_list() -> Vec<Starter> {
    vec![
        Starter::new(
            "c1",
            "code",
            "Fix the flaky test",
            "Find why auth_refresh flakes and fix it.",
        ),
        Starter::new(
            "w1",
            "write",
            "Draft release notes",
            "Draft release notes for v0.3 from this week's PRs.",
        ),
        Starter::new(
            "r1",
            "research",
            "Compare vector stores",
            "Compare three vector stores for 10M rows.",
        ),
        Starter::new(
            "p1",
            "plan",
            "Plan the migration",
            "Plan the Postgres 17 migration with rollback.",
        ),
    ]
}

const ANSWER: &str = "The race is in `refresh()`: two requests read the same expiry, both refresh, and the second write wins with a stale token.\n\nI'll take the lock **before** reading the expiry and add a 30 s leeway, then rerun the auth suite.";

fn mini_comp(placeholder: &str) -> Element {
    rsx! {
        div { class: "oxc-ai-comp oxs-comp", textarea { rows: "1", placeholder: "{placeholder}", "aria-label": "Message" } button { class: "stsend", r#type: "button", "aria-label": "Send", disabled: true, "↑" } }
    }
}

fn screen_el(sc: Screen, now: ClockMs, el: u64) -> Element {
    let cps = 60.0;
    match sc {
        Screen::Start => rsx! {
            ChatShell { threads: threads(), active: 0, height: 540.0,
                div { class: "oxs-center",
                    div { class: "oxd-w560",
                        StarterPrompts { greeting: "Good afternoon, Ismail", description: Some(s("Ask anything, or start from a prompt. Atlas can read the repo, run commands and open PRs.")), prompts: starter_list(), count: 4, composer: true }
                    }
                }
            }
        },
        Screen::Overview => rsx! {
            ChatShell { title: "Overview", height: 540.0,
                div { class: "oxs-pad",
                    h3 { class: "oxs-h", "Your agents" }
                    div { class: "oxs-grid",
                        for (name, st, what) in [("Atlas", "tool-running", "Running tests in auth/"), ("Scout", "awaiting-approval", "Needs approval: deploy to staging"), ("Quill", "done", "Finished the onboarding copy"), ("Relay", "thinking", "Reading the billing webhooks")] {
                            div { key: "{name}", class: "oxs-agent",
                                StatusIndicator { state: data::state(st), now }
                                div { b { "{name}" } small { "{what}" } }
                            }
                        }
                    }
                    h3 { class: "oxs-h", "Recent threads" }
                    ul { class: "oxs-recent",
                        for (t, m) in threads() { li { key: "{t}", b { "{t}" } small { "{m}" } } }
                    }
                    {mini_comp("Ask across all agents…")}
                }
            }
        },
        Screen::Agents => rsx! {
            ChatShell { threads: threads(), active: 0, title: "Fix the refresh-token race", height: 540.0,
                side: rsx! { h4 { class: "oxs-sh", "Agents" } SubagentTree { graph: data::graph(), now } },
                div { class: "oxs-thread",
                    MessageBubble { role: "user", text: "Why does auth fail at the minute boundary?", time: Some(s("14:02")) }
                    div { class: "oxs-bot", StreamingMarkdown { text: ANSWER, elapsed_ms: el, cps } }
                }
                {mini_comp("Reply to Atlas…")}
            }
        },
        Screen::DoubleSidebar => rsx! {
            ChatShell { rail: true, threads: threads(), active: 1, title: "Release notes for v0.3", height: 540.0,
                div { class: "oxs-thread",
                    MessageBubble { role: "user", text: "Draft release notes for v0.3 from this week's merged PRs.", time: Some(s("10:12")) }
                    div { class: "oxs-bot", StreamingMarkdown { text: "**v0.3** brings the composer parts: voice recording, dictation, mentions, mode and permission controls.\n\n- Faster thread loading\n- Fixed the token refresh race\n- New terminal tabs", elapsed_ms: el, cps } }
                }
                {mini_comp("Reply…")}
            }
        },
        Screen::Progress => rsx! {
            ChatShell { threads: threads(), active: 0, title: "Fix the refresh-token race", height: 540.0,
                side: rsx! { h4 { class: "oxs-sh", "Progress" } TodoList { plan: data::plan(), agent: data::state("tool-running"), now, variant: "card".to_string(), expanded: true } },
                div { class: "oxs-thread",
                    MessageBubble { role: "user", text: "Fix the refresh race and keep me posted.", time: Some(s("14:02")) }
                    div { class: "oxs-bot", StreamingMarkdown { text: ANSWER, elapsed_ms: el, cps } }
                }
                {mini_comp("Reply to Atlas…")}
            }
        },
        Screen::Run => {
            let (calls, entries) = data::explore();
            rsx! {
                ChatShell { threads: threads(), active: 0, title: "Run 248 · Fix the refresh-token race", height: 560.0,
                    div { class: "oxs-thread",
                        PlanCard { plan: data::plan(), now }
                        ToolStack { calls, entries, now, turn_done: Some(Duration::from_secs(31)) }
                        ApprovalCard { card: data::approval("high", false), now, variant: "card".to_string() }
                    }
                }
            }
        }
        Screen::Thread => rsx! {
            ChatShell { threads: threads(), active: 2, title: "Scout · Audit the billing webhooks", height: 540.0,
                div { class: "oxs-thread",
                    MessageBubble { role: "user", text: "Audit the billing webhooks for replay attacks.", time: Some(s("09:40")) }
                    ToolCallRow { call: data::tool_call("succeeded"), now }
                    ToolCallRow { call: data::tool_call("streaming-output"), now }
                    div { class: "oxs-bot", StreamingMarkdown { text: "Two handlers skip the signature timestamp check. I'll add a 5-minute tolerance and a replay cache keyed on the event id.", elapsed_ms: el, cps } }
                }
                {mini_comp("Reply to Scout…")}
            }
        },
    }
}

fn screens(v: &Vals, now: ClockMs) -> Element {
    let sc = Screen::from_id(v.s("screen"));
    let el = if v.b("finished") {
        u64::MAX
    } else {
        (now.0 % 14_000).saturating_sub(600)
    };
    let k = v.string("screen");
    rsx! {
        div { key: "{k}", style: "width:100%",
            {app_frame(&format!("Atlas · {}", sc.title()), screen_el(sc, now, el))}
        }
    }
}

fn screens_code(v: &Vals) -> String {
    Code::new("ChatShell")
        .raw("threads", "threads")
        .raw("side", "rsx! { TodoList { .. } }")
        .raw(
            "children",
            "rsx! { MessageBubble { .. } StreamingMarkdown { .. } }",
        )
        .done()
        .replace(
            "ChatShell",
            &format!("// screen: {}\nChatShell", v.s("screen")),
        )
}

pub fn demos() -> Vec<Demo> {
    let shell_resp = vec![
        "The frame fills the stage width; the sidebar keeps its width and the content column takes the rest.",
        "Below 640 px fold the sidebar to the rail with `[`.",
    ];
    vec![
        Demo {
            controls: vec![
                tog("collapsed", "Icon rail", false),
                sel(
                    "active",
                    "Active",
                    &["checkout", "home", "inbox", "roadmap", "q4"],
                    "checkout",
                ),
                rng("height", "Height", 360.0, 560.0, 20.0, "460"),
            ],
            variants: vec![
                var(
                    "open",
                    "Brand, nested Projects open on the active page.",
                    &[],
                ),
                var(
                    "rail",
                    "Folded to a 64 px icon rail with tooltips and an unread dot.",
                    &[("collapsed", "true")],
                ),
                var(
                    "home",
                    "Top-level page active; groups closed.",
                    &[("active", "home")],
                ),
            ],
            render: sidebar,
            code: sidebar_code,
            keyboard: vec![
                ("[", "Fold or expand"),
                ("Enter / Space", "Open a row or toggle a group"),
                ("Tab", "Move between rows"),
            ],
            a11y: vec![
                "A labelled nav; the open page has aria-current.",
                "Groups expose aria-expanded; rail rows carry the label and unread count.",
                "Fold and expand are announced politely.",
            ],
            responsive: shell_resp.clone(),
            height: 560,
            align: Align::Start,
            ..mk(
                "sidebar",
                "Sidebar",
                "shells",
                "An app sidebar: brand, sections with nested groups, pinned pages, settings and the account; folds to an icon rail.",
                "Sidebar",
                "SidebarEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("menu", "Switcher open", false),
                sel("query", "Search", &["", "check", "zzz"], ""),
                tog("collapsed", "Icon rail", false),
                rng("storage", "Storage %", 0.0, 100.0, 1.0, "38"),
            ],
            variants: vec![
                var("default", "Favourites, projects, storage and account.", &[]),
                var(
                    "switcher",
                    "The workspace switcher open.",
                    &[("menu", "true")],
                ),
                var(
                    "search",
                    "Live search filters both lists.",
                    &[("query", "check")],
                ),
                var("no-match", "Nothing matches.", &[("query", "zzz")]),
                var("rail", "Folded to the rail.", &[("collapsed", "true")]),
            ],
            render: workspace,
            code: workspace_code,
            keyboard: vec![
                ("[", "Fold or expand"),
                ("Alt + ↑ / ↓", "Reorder a favourite"),
                ("Enter / Escape", "Create or cancel a new project"),
                ("Escape", "Close the switcher or clear search"),
            ],
            a11y: vec![
                "The switcher is a menu button with radio items.",
                "Reorders and new projects are announced.",
                "Storage is a meter.",
            ],
            responsive: shell_resp.clone(),
            height: 580,
            align: Align::Start,
            ..mk(
                "workspace-sidebar",
                "Workspace sidebar",
                "shells",
                "A workspace sidebar: switcher, live search, reorderable favourites, inline new projects, a storage meter and the account.",
                "WorkspaceSidebar",
                "WorkspaceSidebarEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("collapsed", "Start folded", false),
                rng("width", "Width", 200.0, 360.0, 4.0, "248"),
                sel(
                    "transition",
                    "Fold transition",
                    &["ease", "cascade", "scrub"],
                    "ease",
                ),
            ],
            variants: vec![
                var(
                    "ease",
                    "Labels fade in 160 ms while the width glides over 400 ms; icons stay put.",
                    &[],
                ),
                var(
                    "cascade",
                    "Labels drift left and fade from the top down.",
                    &[("transition", "cascade")],
                ),
                var(
                    "scrub",
                    "Labels fade as you drag, then settle to icons.",
                    &[("transition", "scrub")],
                ),
                var("rail", "Folded to the icon rail.", &[("collapsed", "true")]),
                var("wide", "Held at the 360 px maximum.", &[("width", "360")]),
            ],
            render: rail,
            code: rail_code,
            keyboard: vec![
                ("[", "Fold or expand"),
                ("← / →", "Resize by 16 px on the separator"),
                ("Enter", "Fold on the separator"),
                ("Double-click", "Fold on the separator"),
            ],
            a11y: vec![
                "The separator is a focusable splitter with value and text.",
                "The breadcrumb follows the open page.",
            ],
            responsive: shell_resp.clone(),
            height: 560,
            align: Align::Start,
            ..mk(
                "sidebar-rail",
                "Sidebar rail",
                "shells",
                "A resizable sidebar that snaps to a 64 px icon rail, with a keyboard splitter and a breadcrumb that follows the page.",
                "SidebarRail",
                "RailShellEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("collapsed", "Folders as rail", false),
                tog("composing", "Composer open", false),
            ],
            variants: vec![
                var("default", "Folders, labels and the message list.", &[]),
                var(
                    "compose",
                    "Compose grows into a composer.",
                    &[("composing", "true")],
                ),
                var(
                    "rail",
                    "Folders folded to the rail.",
                    &[("collapsed", "true")],
                ),
            ],
            render: inbox,
            code: inbox_code,
            keyboard: vec![
                ("S", "Star the focused message"),
                ("E", "Archive the focused message"),
                ("⌘ / Ctrl + Enter", "Send from the composer"),
                ("Escape", "Save a draft and close"),
            ],
            a11y: vec![
                "Folder rows name their unread counts.",
                "Each message row reads sender, subject, time and state.",
                "Moves, stars and undo are announced.",
            ],
            responsive: shell_resp.clone(),
            height: 580,
            align: Align::Start,
            ..mk(
                "inbox-sidebar",
                "Inbox sidebar",
                "shells",
                "Mail folders with counts and labels beside a message list: drag onto folders or Move to, star, archive, undo, and a Compose button that grows into a composer.",
                "InboxSidebar",
                "InboxEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["inset", "flat"], "inset"),
                tog("rail", "Sidebar as rail", false),
            ],
            variants: vec![
                var(
                    "inset",
                    "Content floats on a raised card in the sidebar's plane.",
                    &[],
                ),
                var(
                    "flat",
                    "One plane split by hairlines.",
                    &[("variant", "flat")],
                ),
                var(
                    "inset-rail",
                    "Inset with the sidebar folded.",
                    &[("rail", "true")],
                ),
            ],
            render: appshell,
            code: appshell_code,
            keyboard: vec![
                ("[", "Fold the sidebar"),
                ("Tab", "Sidebar, header actions, content"),
            ],
            a11y: vec![
                "Landmarks: nav, header, main content.",
                "Header actions are labelled icon buttons.",
            ],
            responsive: shell_resp.clone(),
            height: 560,
            align: Align::Start,
            ..mk(
                "app-shell",
                "App shell",
                "shells",
                "The app frame: sidebar, header with title and actions, and content, flat or inset.",
                "AppShell",
                "AppShellEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Panel open", true),
                tog("proposed", "Edit proposed", false),
            ],
            variants: vec![
                var("ask", "Suggestions; ask and watch the answer stream.", &[]),
                var(
                    "proposed",
                    "A word diff with Apply and Discard.",
                    &[("proposed", "true")],
                ),
                var(
                    "closed",
                    "Closed; the Assistant button reopens it.",
                    &[("open", "false")],
                ),
            ],
            render: aipanel,
            code: aipanel_code,
            keyboard: vec![
                ("Enter", "Ask"),
                ("Escape", "Close the panel"),
                ("Tab", "Apply, Discard, Undo"),
            ],
            a11y: vec![
                "The panel is a labelled complementary region; the thread is a polite log.",
                "Apply moves focus to Undo; changes are announced.",
            ],
            responsive: vec!["The panel takes up to 48% of the frame and at most 340 px."],
            height: 560,
            align: Align::Start,
            live: true,
            ..mk(
                "ai-side-panel",
                "AI side panel",
                "shells",
                "An assistant docked beside a page: it streams an answer, proposes an edit as a word diff, applies it with a tint and offers Undo.",
                "AiSidePanel",
                "AiPanelEl",
                "side-panels",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                rng("tabs", "Tabs", 1.0, 5.0, 1.0, "3"),
                rng(
                    "activity",
                    "Background output ms",
                    0.0,
                    6000.0,
                    500.0,
                    "2500",
                ),
            ],
            variants: vec![
                var("three", "Dev server, tests (exit 0) and a shell.", &[]),
                var(
                    "busy",
                    "Background tabs print often and show dots.",
                    &[("activity", "800"), ("tabs", "5")],
                ),
                var(
                    "one",
                    "A single quiet tab.",
                    &[("tabs", "1"), ("activity", "0")],
                ),
            ],
            render: term,
            code: term_code,
            keyboard: vec![
                ("Ctrl + Tab", "Next tab (Shift for previous)"),
                ("← / →", "Move between tabs"),
                ("Double-click", "Rename"),
                ("Enter / Escape", "Save or cancel a rename"),
            ],
            a11y: vec![
                "A tablist with one tabpanel; background output adds “new output” to the tab name.",
                "Close buttons are labelled per tab.",
            ],
            responsive: vec!["The strip scrolls sideways when tabs overflow."],
            height: 420,
            align: Align::Start,
            live: true,
            ..mk(
                "terminal-tabs",
                "Terminal tabs",
                "shells",
                "Terminal tabs: add, close, rename, cycle with Ctrl+Tab, activity dots on background tabs and exit codes.",
                "TerminalTabs",
                "TermTabsEl",
                "terminal-tabs",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                tog("compact", "Compact", false),
                sel(
                    "active",
                    "Active",
                    &["home", "inbox", "projects", "roadmap", "q4"],
                    "home",
                ),
            ],
            variants: vec![
                var("default", "A sliding active indicator and badges.", &[]),
                var(
                    "compact",
                    "Compact rows.",
                    &[("compact", "true"), ("active", "roadmap")],
                ),
                var("pinned", "A pinned page active.", &[("active", "q4")]),
            ],
            render: snav,
            code: snav_code,
            keyboard: vec![
                ("Tab", "Move between links"),
                ("Enter", "Open a link or toggle a group"),
            ],
            a11y: vec![
                "Links carry aria-current; group headings expose aria-expanded.",
                "Port of Beautiful UI Sidebar Nav (MIT, © 2026 Shane Levine); its paid Central Icons are replaced with Oxyn icons.",
            ],
            responsive: vec!["Fixed 240 px column."],
            height: 520,
            align: Align::Start,
            ..mk(
                "sidebar-nav",
                "Sidebar Nav",
                "shells",
                "Grouped sidebar links with a sliding active indicator, collapsible groups and badges (Beautiful UI, MIT).",
                "SidebarNav",
                "SidebarNavEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "viewport",
                    "Viewport",
                    &["desktop", "tablet", "phone"],
                    "desktop",
                ),
                tog("dark", "Dark canvas", false),
                sel("variant", "Shell", &["inset", "flat"], "inset"),
            ],
            variants: vec![
                var("desktop", "A demo in the harness with Fork this.", &[]),
                var("tablet", "Tablet width.", &[("viewport", "tablet")]),
                var(
                    "dark",
                    "Dark canvas, flat shell.",
                    &[("dark", "true"), ("variant", "flat")],
                ),
            ],
            render: harness,
            code: harness_code,
            keyboard: vec![("Tab", "Viewport, theme, Fork this")],
            a11y: vec![
                "The viewport switch is a radio group.",
                "Fork this announces the fork politely.",
            ],
            responsive: vec!["Phone and tablet widths clamp to the stage."],
            height: 540,
            align: Align::Start,
            ..mk(
                "harness",
                "Harness",
                "shells",
                "A demo frame with viewport widths, a dark canvas and a Fork this button that hands over the current demo and its settings.",
                "Harness",
                "HarnessEl",
                "workspace-chrome",
                "shell/mod.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "screen",
                    "Screen",
                    &[
                        "start", "overview", "agents", "double", "progress", "run", "thread",
                    ],
                    "start",
                ),
                tog("finished", "Show finished", false),
            ],
            variants: Screen::ALL
                .iter()
                .map(|sc| {
                    let id = sc.id();
                    var(id, sc.title(), &[("screen", id)])
                })
                .collect(),
            render: screens,
            code: screens_code,
            keyboard: vec![
                ("Tab", "Rail, threads, main column, side panel"),
                ("Enter", "Open a thread or send"),
            ],
            a11y: vec![
                "Each screen has nav, main and complementary landmarks.",
                "Streamed answers announce politely; approvals keep their own keys.",
            ],
            responsive: vec![
                "The side panel and thread list keep fixed widths; the main column flexes.",
            ],
            height: 640,
            align: Align::Start,
            live: true,
            ..mk(
                "ai-chat-screens",
                "AI Chat screens",
                "shells",
                "The seven AI Chat screens (Start, Overview, Agents, Double sidebar, Progress, Run, Agent thread) composed from the chat, composer, plan, tool and approval parts.",
                "ChatShell",
                "ChatShellEl",
                "ai-chat-screens",
                "shell/mod.rs",
            )
        },
    ]
}
