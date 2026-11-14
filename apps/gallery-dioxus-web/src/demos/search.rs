//! Search and commands (plan #26): command palette / command menu, AI command, AI search bar,
//! settings command, search field, expanding search, shortcut recorder, pagination.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        command::{AiAction, Command, Setting, SettingKind, SettingValue, Source},
        search::{Binding, ShortcutGroup},
    },
};
use oxyn_dioxus::{
    AiCommand, AiSearchBar, CommandPalette, ExpandingSearch, Pagination, ScriptedAnswer,
    SearchField, SettingsCommand, ShortcutList, ShortcutRecorder,
};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

fn commands() -> Vec<Command> {
    vec![
        Command::new("sum", "Summarize this thread")
            .group("Suggestions")
            .logo("AI")
            .suggested()
            .keywords(&["tldr"]),
        Command::new("issue", "Create Linear issue")
            .group("Suggestions")
            .logo("L")
            .suggested()
            .hint("Linear"),
        Command::new("pr", "Open pull request")
            .group("Suggestions")
            .logo("GH")
            .suggested()
            .hint("GitHub"),
        Command::new("slack", "Share to #eng-agents")
            .group("Suggestions")
            .logo("S")
            .suggested()
            .hint("Slack"),
        Command::new("doc", "Draft release notes")
            .group("Suggestions")
            .logo("N")
            .suggested()
            .hint("Notion"),
        Command::new("new", "New thread")
            .group("General")
            .icon("plus")
            .shortcut("Mod+N")
            .suggested(),
        Command::new("stop", "Stop the agent")
            .group("General")
            .icon("stop")
            .shortcut("Mod+.")
            .keywords(&["cancel", "halt"]),
        Command::new("model", "Switch model")
            .group("General")
            .icon("spark")
            .desc("Sonnet 4.5, Opus, GPT-5")
            .keywords(&["llm"]),
        Command::new("rewind", "Rewind to checkpoint")
            .group("General")
            .icon("undo")
            .keywords(&["restore", "undo"]),
        Command::new("share", "Share project")
            .group("General")
            .icon("share")
            .keywords(&["invite", "link"]),
        Command::new("settings", "Open settings")
            .group("Go to")
            .icon("cog")
            .shortcut("Mod+,")
            .desc("Preferences and API keys"),
        Command::new("runs", "Runs")
            .group("Go to")
            .icon("agents")
            .hint("Jump to"),
        Command::new("approvals", "Approvals")
            .group("Go to")
            .icon("hand")
            .hint("Jump to"),
        Command::new("session", "auth/session.ts")
            .group("Files")
            .icon("file")
            .scope('@'),
        Command::new("client", "api/client.ts")
            .group("Files")
            .icon("file")
            .scope('@'),
        Command::new("t-auth", "Fix refresh-token race")
            .group("Threads")
            .icon("thread")
            .scope('#'),
        Command::new("t-docs", "Docs for the retry policy")
            .group("Threads")
            .icon("thread")
            .scope('#'),
    ]
}

fn palette(v: &Vals, now: ClockMs) -> Element {
    let menu = v.s("variant") == "menu";
    let mut items = commands();
    if !menu {
        for c in items
            .iter_mut()
            .filter(|c| c.group.as_deref() == Some("Suggestions"))
        {
            c.logo = None;
            c.icon = Some("bolt".into());
        }
    }
    rsx! {
        CommandPalette {
            items,
            variant: v.string("variant"),
            query: v.string("query"),
            scopes: v.b("scopes"),
            ask_fallback: v.b("ask"),
            footer: v.b("footer"),
            elevated: v.b("elevated"),
            width: 560.0,
            max_height: v.f("max_height") as f32,
            now,
        }
    }
}

fn palette_code(v: &Vals) -> String {
    Code::new("CommandPalette")
        .pre("let items = vec![")
        .pre("    Command::new(\"sum\", \"Summarize this thread\").group(\"Suggestions\").logo(\"AI\").suggested(),")
        .pre("    Command::new(\"new\", \"New thread\").group(\"General\").icon(\"plus\").shortcut(\"Mod+N\"),")
        .pre("    Command::new(\"settings\", \"Open settings\").group(\"Go to\").icon(\"cog\").desc(\"Preferences and API keys\"),")
        .pre("];")
        .raw("items", "items")
        .s("variant", v.s("variant"), "palette")
        .b("auto_focus", true)
        .raw("on_select", "move |(id, query)| run(id, query)")
        .raw("on_close", "move |_| open.set(false)")
        .done()
}

fn answer() -> ScriptedAnswer {
    ScriptedAnswer {
        text: "Webhook deliveries retry with exponential backoff: 1 min, 5 min, 30 min, then hourly for 24 hours [1]. Each retry reuses the same `Idempotency-Key`, so your handler can skip duplicates [2]. Acme's failures since Tuesday come from a 504 on their endpoint, not from us [3].".into(),
        sources: vec![
            Source::new("doc-retries", "Webhook retries").mark("D"),
            Source::new("doc-idem", "Idempotency keys").mark("D"),
            Source::new("iss-912", "AUTH-912").mark("L"),
        ],
        actions: vec![
            AiAction::new("lin", "Create issue in Linear", "Created ENG-2481").mark("L"),
            AiAction::new("slack", "Reply to Acme in Slack", "Draft saved to #acme-support").mark("S"),
        ],
    }
}

fn ai_command(v: &Vals, now: ClockMs) -> Element {
    let items = vec![
        Command::new("p-webhooks", "Webhooks")
            .group("Pages")
            .icon("link")
            .hint("Docs"),
        Command::new("p-keys", "API keys")
            .group("Pages")
            .icon("key")
            .hint("Settings"),
        Command::new("p-usage", "Usage and billing")
            .group("Pages")
            .icon("table")
            .hint("Settings"),
        Command::new("a-invite", "Invite a teammate")
            .group("Actions")
            .icon("users")
            .shortcut("Mod+I"),
        Command::new("a-rotate", "Rotate signing secret")
            .group("Actions")
            .icon("retry"),
        Command::new("a-replay", "Replay failed deliveries")
            .group("Actions")
            .icon("play"),
    ];
    rsx! {
        AiCommand {
            items,
            suggested_questions: vec!["How do webhook retries work?".to_string(), "Why are Acme's deliveries failing?".to_string(), "Which keys expire this month?".to_string()],
            model: "Claude Sonnet 4.5",
            default_mode: v.string("mode"),
            default_query: v.string("query"),
            height: v.f("height") as f32,
            elevated: v.b("elevated"),
            answer: answer(),
            autoplay: v.b("autoplay"),
            now,
            width: 600.0,
        }
    }
}

fn ai_command_code(v: &Vals) -> String {
    Code::new("AiCommand")
        .raw("items", "pages_and_actions()")
        .raw(
            "suggested_questions",
            "vec![\"How do webhook retries work?\".into()]",
        )
        .raw("model", q("Claude Sonnet 4.5"))
        .s("default_mode", v.s("mode"), "search")
        .raw("answer", "answer()")
        .raw("now", "clock.now()")
        .raw("on_select", "move |id: String| go(id)")
        .done()
}

fn results() -> Vec<Command> {
    vec![
        Command::new("doc-retries", "Webhook retries")
            .group("Docs")
            .icon("page")
            .desc("Backoff schedule and limits"),
        Command::new("doc-idem", "Idempotency keys")
            .group("Docs")
            .icon("page")
            .desc("Safely retrying requests"),
        Command::new("doc-events", "Webhook event types")
            .group("Docs")
            .icon("page")
            .desc("Payloads for every event"),
        Command::new("cus-acme", "Acme Corp")
            .group("Customers")
            .icon("building")
            .desc("Enterprise · 3 webhook endpoints")
            .hint("$48k ARR"),
        Command::new("cus-globex", "Globex")
            .group("Customers")
            .icon("building")
            .desc("Growth · 1 endpoint")
            .hint("$9k ARR"),
        Command::new("iss-912", "AUTH-912 Webhooks to Acme return 504")
            .group("Issues")
            .icon("comment")
            .hint("In progress"),
        Command::new("iss-877", "ENG-877 Retry storms after outage")
            .group("Issues")
            .icon("comment")
            .hint("Done"),
    ]
}

fn ai_search(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        AiSearchBar {
            results: results(),
            recent: vec!["webhook retries".to_string(), "Figma".to_string(), "AUTH-912".to_string()],
            suggestions: vec!["Why are Acme's webhooks failing?".to_string(), "How do I rotate a signing secret?".to_string()],
            model: "Claude Sonnet 4.5",
            query: v.string("query"),
            autoplay: v.b("autoplay"),
            answer: answer(),
            height: v.f("height") as f32,
            width: 600.0,
            now,
        }
    }
}

fn ai_search_code(v: &Vals) -> String {
    Code::new("AiSearchBar")
        .raw("results", "search(query())")
        .raw("recent", "vec![\"webhook retries\".into()]")
        .raw(
            "suggestions",
            "vec![\"Why are Acme's webhooks failing?\".into()]",
        )
        .raw("model", q("Claude Sonnet 4.5"))
        .opt("query", v.s("query"))
        .raw("answer", "answer()")
        .raw("now", "clock.now()")
        .done()
}

type Sections = Vec<(String, String, String)>;

fn settings_defs() -> (Sections, Vec<Setting>, BTreeMap<String, SettingValue>) {
    let sec = |a: &str, b: &str, c: &str| (a.to_string(), b.to_string(), c.to_string());
    let sections = vec![
        sec("general", "General", "Your workspace"),
        sec("agents", "Agents", "How agents run by default"),
        sec("notify", "Notifications", "When we reach you"),
    ];
    let settings = vec![
        Setting::new(
            "name",
            "general",
            "Workspace name",
            "Shown in the sidebar and emails",
            SettingKind::Text {
                prefix: None,
                required: true,
                max: Some(32),
            },
        )
        .keywords(&["title", "rename"]),
        Setting::new(
            "url",
            "general",
            "Workspace URL",
            "Letters, numbers and dashes",
            SettingKind::Text {
                prefix: Some("oxyn.dev/".into()),
                required: true,
                max: Some(24),
            },
        )
        .keywords(&["slug", "link"]),
        Setting::new(
            "theme",
            "general",
            "Theme",
            "Light, dark or follow the system",
            SettingKind::Segmented(vec!["Light".into(), "Dark".into(), "System".into()]),
        )
        .keywords(&["appearance", "mode"]),
        Setting::new(
            "model",
            "agents",
            "Default model",
            "Used for new threads",
            SettingKind::Select(vec!["Sonnet 4.5".into(), "Opus 4.1".into(), "GPT-5".into()]),
        )
        .keywords(&["llm"]),
        Setting::new(
            "approve",
            "agents",
            "Ask before risky actions",
            "Writes outside the worktree, installs, pushes",
            SettingKind::Switch,
        )
        .keywords(&["approval", "permission"]),
        Setting::new(
            "effort",
            "agents",
            "Effort",
            "How long agents think before acting",
            SettingKind::Segmented(vec!["Low".into(), "Medium".into(), "High".into()]),
        )
        .keywords(&["reasoning"]),
        Setting::new(
            "digest",
            "notify",
            "Daily digest",
            "A summary of runs each morning",
            SettingKind::Switch,
        )
        .keywords(&["email", "summary"]),
        Setting::new(
            "done",
            "notify",
            "Finished runs",
            "Notify when a background run ends",
            SettingKind::Switch,
        )
        .keywords(&["complete"]),
    ];
    let mut saved = BTreeMap::new();
    for (k, v) in [
        ("name", "Atlas Labs"),
        ("url", "atlas"),
        ("theme", "Dark"),
        ("model", "Sonnet 4.5"),
        ("effort", "Medium"),
    ] {
        saved.insert(k.to_string(), SettingValue::Text(v.into()));
    }
    for (k, b) in [("approve", true), ("digest", false), ("done", true)] {
        saved.insert(k.to_string(), SettingValue::Bool(b));
    }
    (sections, settings, saved)
}

fn settings(v: &Vals, _: ClockMs) -> Element {
    let (sections, settings, saved) = settings_defs();
    rsx! {
        SettingsCommand {
            sections, settings, saved,
            suggested: vec!["theme".to_string(), "approve".to_string(), "digest".to_string()],
            fail_first: v.b("fail_first"),
            save_ms: v.u("save_ms") as u32,
            open_query: v.b("open").then(|| v.string("query")),
            width: 640.0,
        }
    }
}

fn settings_code(v: &Vals) -> String {
    Code::new("SettingsCommand")
        .raw("sections", "sections()")
        .raw("settings", "settings()")
        .raw("saved", "saved_values()")
        .raw("suggested", "vec![\"theme\".into(), \"approve\".into()]")
        .b("fail_first", v.b("fail_first"))
        .raw("on_save", "move |ids: Vec<String>| persist(ids)")
        .done()
}

fn search_field(v: &Vals, _: ClockMs) -> Element {
    let items: Vec<String> = if v.b("list") {
        [
            "Billing",
            "Team members",
            "API keys",
            "Webhooks",
            "Usage",
            "Audit log",
            "Integrations",
            "Single sign-on",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    } else {
        vec![]
    };
    rsx! {
        SearchField {
            label: "Search settings",
            value: v.string("value"),
            placeholder: "Search",
            hide_label: v.b("hide_label"),
            size: v.string("size"),
            disabled: v.b("disabled"),
            items,
            scopes: if v.b("scopes") { vec!["All".to_string(), "Docs".to_string(), "Chats".to_string()] } else { vec![] },
            ask_ai: v.b("ask_ai"),
            width: 340.0,
        }
    }
}

fn search_field_code(v: &Vals) -> String {
    Code::new("SearchField")
        .raw("label", q("Search settings"))
        .raw("value", "query()")
        .b("hide_label", v.b("hide_label"))
        .s("size", v.s("size"), "md")
        .raw("items", if v.b("list") { "pages()" } else { "vec![]" })
        .b("ask_ai", v.b("ask_ai"))
        .raw("on_change", "move |q: String| query.set(q)")
        .done()
}

fn expanding(v: &Vals, _: ClockMs) -> Element {
    let items = vec![
        Command::new("p-atlas", "Atlas roadmap")
            .group("Projects")
            .icon("folder")
            .hint("12 runs"),
        Command::new("p-auth", "Auth hardening")
            .group("Projects")
            .icon("folder")
            .hint("4 runs"),
        Command::new("d-agents", "Agent API reference")
            .group("Docs")
            .icon("page"),
        Command::new("d-atlas", "Atlas onboarding guide")
            .group("Docs")
            .icon("page"),
        Command::new("d-hooks", "Webhook retries")
            .group("Docs")
            .icon("page"),
        Command::new("t-auth", "Fix refresh-token race")
            .group("Threads")
            .icon("thread")
            .hint("Today"),
    ];
    let sugg = vec![
        Command::new("r-1", "atlas roadmap"),
        Command::new("r-2", "webhook retries"),
    ];
    rsx! {
        div { class: "oxd-xs-lane",
            ExpandingSearch {
                label: "Search projects and docs",
                items,
                suggestions: sugg,
                anchor: v.string("anchor"),
                expanded_width: v.f("width") as f32,
                max_results: v.u("max_results"),
                default_expanded: v.b("expanded"),
                query: v.string("query"),
                hotkey: true,
                lane: 440.0,
            }
        }
    }
}

fn expanding_code(v: &Vals) -> String {
    Code::new("ExpandingSearch")
        .raw("label", q("Search projects and docs"))
        .raw("items", "items()")
        .raw("suggestions", "recent()")
        .s("anchor", v.s("anchor"), "end")
        .raw("expanded_width", format!("{:.0}.0", v.f("width")))
        .raw("on_select", "move |id: String| open(id)")
        .done()
}

fn recorder(v: &Vals, _: ClockMs) -> Element {
    let groups = vec![
        ShortcutGroup {
            label: "General".into(),
            items: vec![
                ("Command palette".into(), "mod+k".into()),
                ("New thread".into(), "mod+shift+n".into()),
                ("Settings".into(), "mod+,".into()),
            ],
        },
        ShortcutGroup {
            label: "Agent".into(),
            items: vec![
                ("Stop".into(), "mod+.".into()),
                ("Approve".into(), "mod+enter".into()),
                ("Rewind".into(), "mod+alt+z".into()),
            ],
        },
    ];
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:22px;width:560px;max-width:100%",
            ShortcutRecorder {
                label: "Open command palette",
                value: v.opt("value"),
                reset_value: Some("mod+k".to_string()),
                bindings: vec![Binding::new("mod+shift+n", "New thread"), Binding::new("mod+.", "Stop the agent")],
                warn_reserved: v.b("warn_reserved"),
                require_modifier: v.b("require_modifier"),
                platform: v.string("platform"),
                description: Some("Used everywhere in the app".to_string()),
                size: v.string("size"),
                sequences: v.b("sequences"),
                recording: v.b("recording"),
                disabled: v.b("disabled"),
            }
            if v.b("cheatsheet") {
                ShortcutList { groups, platform: v.string("platform"), width: 560.0 }
            }
        }
    }
}

fn recorder_code(v: &Vals) -> String {
    Code::new("ShortcutRecorder")
        .raw("label", q("Open command palette"))
        .raw("value", "shortcut()")
        .raw(
            "bindings",
            "vec![Binding::new(\"mod+shift+n\", \"New thread\")]",
        )
        .s("platform", v.s("platform"), "mac")
        .b("sequences", v.b("sequences"))
        .raw("on_change", "move |(v, _replaced)| shortcut.set(v)")
        .done()
}

fn pagination(v: &Vals, _: ClockMs) -> Element {
    let with_total = v.b("total");
    rsx! {
        Pagination {
            page: v.u("page"),
            count: v.u("count"),
            mode: v.string("mode"),
            total: with_total.then_some(312),
            sizes: if with_total { vec![20, 50, 100] } else { vec![] },
        }
    }
}

fn pagination_code(v: &Vals) -> String {
    Code::new("Pagination")
        .raw("page", "page()")
        .raw("count", v.s("count"))
        .s("mode", v.s("mode"), "window")
        .raw("on_page_change", "move |p: usize| page.set(p)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "command-palette",
            name: "Command palette",
            category: "search",
            summary: "Search and run anything from one field: every word must match, matched letters in medium weight, a gliding highlight, Tab jumps sections. The menu variant adds tool logos, ⌘1–⌘5 and a tray.",
            dioxus: "CommandPalette",
            gpui: "CommandPaletteEl",
            spec: "command-palette",
            source: "primitives/command.rs",
            controls: vec![
                sel("variant", "Variant", &["palette", "menu"], "menu"),
                txt("query", "Query", ""),
                tog("scopes", "Scopes (> @ #)", true),
                tog("ask", "Ask fallback", true),
                tog("footer", "Footer", true),
                tog("elevated", "Elevated", true),
                rng("max_height", "Max height", 200.0, 420.0, 20.0, "320"),
            ],
            variants: vec![
                var("command-menu", "affectUI's menu: logos, ⌘1–⌘5, tray.", &[]),
                var(
                    "palette",
                    "Arc's palette with matched letters.",
                    &[("variant", "palette"), ("query", "sh pr")],
                ),
                var("files-scope", "@ narrows to files.", &[("query", "@a")]),
                var(
                    "no-results",
                    "Ask the assistant instead.",
                    &[("query", "why did run 248 fail")],
                ),
            ],
            render: palette,
            code: palette_code,
            keyboard: vec![
                ("↑ / ↓", "Move through all sections (wraps)"),
                ("Home / End", "First / last"),
                ("Tab / Shift+Tab", "Next / previous section"),
                ("Enter", "Run the highlighted command"),
                ("⌘1–⌘5", "Run a suggestion"),
                ("Esc", "Clear the field, then close"),
                ("> @ #", "Commands, files, threads"),
            ],
            a11y: vec![
                "The field is a combobox controlling a listbox; the highlighted row is aria-activedescendant, so focus never leaves the field.",
                "Groups are labelled; shortcut caps are hidden from screen readers; the result count and what ran are announced politely.",
            ],
            responsive: vec![
                "Fills its container up to its width; under 480 px shortcut keys and hints are hidden.",
            ],
            height: 500,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "ai-command",
            name: "AI command",
            category: "search",
            summary: "A ⌘K palette with an AI mode: search pages and actions, or press Tab to ask; the answer streams in place with numbered sources and next steps you can run, and the palette never changes size.",
            dioxus: "AiCommand, ScriptedAnswer",
            gpui: "AiCommandEl",
            spec: "ai-command",
            source: "primitives/command.rs",
            controls: vec![
                sel("mode", "Mode", &["search", "ask"], "ask"),
                txt("query", "Query", "How do webhook retries work?"),
                tog("autoplay", "Ask on load", true),
                rng("height", "Results height", 280.0, 440.0, 20.0, "360"),
                tog("elevated", "Elevated", true),
            ],
            variants: vec![
                var(
                    "answer",
                    "Streams an answer with sources and next steps.",
                    &[],
                ),
                var(
                    "search",
                    "Pages and actions; Tab asks.",
                    &[("mode", "search"), ("query", "web"), ("autoplay", "false")],
                ),
                var(
                    "suggested",
                    "Ask mode, empty: suggested questions.",
                    &[("query", ""), ("autoplay", "false")],
                ),
            ],
            render: ai_command,
            code: ai_command_code,
            keyboard: vec![
                ("↑ / ↓", "Move"),
                ("Enter", "Open, ask, or run the highlighted step"),
                ("Tab", "Switch between Search and Ask AI"),
                ("Esc", "Close the answer, back to search, then close"),
            ],
            a11y: vec![
                "The answer is a region marked aria-busy while it streams; the footer status is a polite live region.",
                "Citations are buttons named by their source.",
            ],
            responsive: vec![
                "Fixed results height so typing, switching modes and streaming never resize it.",
            ],
            height: 520,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "ai-search-bar",
            name: "AI search bar",
            category: "search",
            summary: "One search card for an app or docs: grouped results as you type, an Ask AI row first; Tab streams a short answer whose numbered citations light their result below.",
            dioxus: "AiSearchBar",
            gpui: "AiSearchBarEl",
            spec: "ai-search-bar",
            source: "primitives/command.rs",
            controls: vec![
                txt("query", "Query", "webhook"),
                tog("autoplay", "Ask on load", true),
                rng("height", "Body height", 240.0, 400.0, 20.0, "320"),
            ],
            variants: vec![
                var("answer", "Answer with citation badges.", &[]),
                var(
                    "results",
                    "Results with the Ask AI row.",
                    &[("autoplay", "false")],
                ),
                var(
                    "empty",
                    "Recent searches and suggested questions.",
                    &[("query", ""), ("autoplay", "false")],
                ),
            ],
            render: ai_search,
            code: ai_search_code,
            keyboard: vec![
                ("↑ / ↓", "Move (wraps)"),
                ("Enter", "Open, fill a recent search or ask"),
                ("Tab", "Ask AI about the query"),
                ("Esc", "Close the answer, then clear"),
            ],
            a11y: vec![
                "Combobox with grouped options; each result's citation badge is labelled \"Cited as n\".",
                "Hovering or focusing a citation lights its result.",
            ],
            responsive: vec!["100 % wide up to its width; the body has a fixed height."],
            height: 460,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "settings-command",
            name: "Settings command",
            category: "search",
            summary: "Workspace settings on one page: the search grows into a palette that flies to any setting (the row breathes twice), and an unsaved-changes bar counts edits, validates, saves or retries.",
            dioxus: "SettingsCommand",
            gpui: "",
            spec: "settings-command",
            source: "primitives/command.rs",
            controls: vec![
                tog("open", "Palette open", true),
                txt("query", "Query", "email"),
                rng("save_ms", "Save takes ms", 200.0, 2000.0, 100.0, "900"),
                tog("fail_first", "First save fails", false),
            ],
            variants: vec![
                var(
                    "search",
                    "The palette finds Daily digest by its keyword.",
                    &[],
                ),
                var(
                    "page",
                    "The settings page; edit to see the bar.",
                    &[("open", "false")],
                ),
                var(
                    "retry",
                    "The first save fails with Retry.",
                    &[("open", "false"), ("fail_first", "true")],
                ),
            ],
            render: settings,
            code: settings_code,
            keyboard: vec![
                ("⌘K / Ctrl+K / /", "Open the settings search"),
                ("↑ / ↓", "Move through results"),
                ("Enter", "Jump to the setting and focus its control"),
                ("Esc", "Close the palette"),
            ],
            a11y: vec![
                "Each control is labelled by its setting and described by its hint or error.",
                "Edited rows carry a dot plus hidden \"Edited\" text; saves and errors are announced.",
            ],
            responsive: vec![
                "640 px wide by default; the section nav sits beside the scrolling page.",
            ],
            height: 480,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "search-field",
            name: "Search field",
            category: "search",
            summary: "A search input whose clear button scales in inside a reserved slot so the width never shifts; with items it filters live under a gliding highlight and shows an empty state.",
            dioxus: "SearchField",
            gpui: "SearchFieldEl",
            spec: "search-field",
            source: "primitives/command.rs",
            controls: vec![
                txt("value", "Value", "key"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("list", "Live results", true),
                tog("scopes", "Scope chips", false),
                tog("ask_ai", "Ask AI row", false),
                tog("hide_label", "Hide label", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("filter", "Live filtering (Beautiful UI).", &[]),
                var(
                    "plain",
                    "Arc's field with the clear button.",
                    &[("list", "false")],
                ),
                var(
                    "scoped-ai",
                    "Scope chips and an Ask AI row.",
                    &[("scopes", "true"), ("ask_ai", "true"), ("value", "sso")],
                ),
                var("empty", "No results.", &[("value", "zzz")]),
            ],
            render: search_field,
            code: search_field_code,
            keyboard: vec![("Esc", "Clear the field")],
            a11y: vec![
                "A native search input with a real label; the clear button is labelled and hands focus back.",
                "Result counts are announced politely.",
            ],
            responsive: vec!["The clear slot is reserved; the field shrinks with min-width 0."],
            height: 340,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "expanding-search",
            name: "Expanding search",
            category: "search",
            summary: "An icon that morphs into a search field with grouped, highlighted results beneath; the panel tracks the field's width; Enter picks and folds it back.",
            dioxus: "ExpandingSearch",
            gpui: "ExpandingSearchEl",
            spec: "expanding-search",
            source: "primitives/command.rs",
            controls: vec![
                tog("expanded", "Expanded", true),
                txt("query", "Query", "atlas"),
                sel("anchor", "Anchor", &["end", "start", "center"], "end"),
                rng("width", "Expanded width", 220.0, 440.0, 20.0, "360"),
                rng("max_results", "Max results", 2.0, 8.0, 1.0, "6"),
            ],
            variants: vec![
                var("results", "Expanded with results.", &[]),
                var("recent", "Expanded, recent searches.", &[("query", "")]),
                var("icon", "Folded into its icon.", &[("expanded", "false")]),
                var("start", "Grows to the right.", &[("anchor", "start")]),
            ],
            render: expanding,
            code: expanding_code,
            keyboard: vec![
                ("Enter / Space", "On the icon: expand"),
                ("↑ / ↓", "Move (wraps)"),
                ("Enter", "Pick"),
                ("Esc", "Fold back"),
                ("⌘K", "Expand (Oxyn plus)"),
            ],
            a11y: vec![
                "Combobox with aria-activedescendant over a listbox of labelled groups.",
                "Counts and empty states are announced; the folded field is not in the tab order.",
            ],
            responsive: vec![
                "Grows to the smaller of its width and its container; the panel caps at min(22 rem, 60 vh).",
            ],
            height: 340,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "shortcut-recorder",
            name: "Shortcut recorder",
            category: "search",
            summary: "Record a key combination into caps: held modifiers show as you build it, reserved and taken shortcuts ask first, plus a searchable cheatsheet that lights held keys.",
            dioxus: "ShortcutRecorder, ShortcutList",
            gpui: "ShortcutRecorderEl",
            spec: "shortcut-recorder",
            source: "primitives/command.rs",
            controls: vec![
                sel(
                    "value",
                    "Value",
                    &["mod+k", "mod+shift+p", "f5", "g i", "none"],
                    "mod+shift+p",
                ),
                sel("platform", "Platform", &["mac", "other"], "mac"),
                sel("size", "Size", &["md", "sm"], "md"),
                tog("recording", "Recording", false),
                tog("require_modifier", "Require a modifier", true),
                tog("warn_reserved", "Warn about reserved", true),
                tog("sequences", "Two-key sequences", false),
                tog("cheatsheet", "Cheatsheet", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("set", "A custom shortcut with Reset.", &[]),
                var("recording", "Listening for keys.", &[("recording", "true")]),
                var(
                    "sequence",
                    "G then I.",
                    &[
                        ("value", "g i"),
                        ("sequences", "true"),
                        ("cheatsheet", "false"),
                    ],
                ),
                var("windows", "Ctrl glyphs.", &[("platform", "other")]),
            ],
            render: recorder,
            code: recorder_code,
            keyboard: vec![
                ("Enter / Space", "Start recording"),
                ("Any combination", "Record it"),
                ("Esc", "Cancel recording"),
                ("Backspace / Delete", "Clear"),
                ("Tab", "Stop and move on"),
            ],
            a11y: vec![
                "A button with aria-pressed while recording, named by the label and the spoken shortcut.",
                "Conflicts offer Use anyway and Cancel as real buttons; results are announced.",
            ],
            responsive: vec!["The cheatsheet goes two columns from 520 px."],
            height: 420,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "pagination",
            name: "Pagination",
            category: "search",
            summary: "Previous and next around a sliding five-page window with a travelling current mark; numbers slide like a belt when the window shifts. Oxyn adds ellipsis, cursor, load-more and page sizes.",
            dioxus: "Pagination",
            gpui: "PaginationEl",
            spec: "pagination",
            source: "primitives/surfaces.rs",
            controls: vec![
                rng("page", "Page", 1.0, 20.0, 1.0, "6"),
                rng("count", "Pages", 0.0, 40.0, 1.0, "20"),
                sel(
                    "mode",
                    "Layout",
                    &["window", "ellipsis", "cursor", "load-more"],
                    "window",
                ),
                tog("total", "Range + page size", false),
            ],
            variants: vec![
                var("window", "Arc's five-page window.", &[]),
                var(
                    "ellipsis",
                    "First, last and neighbours.",
                    &[("mode", "ellipsis"), ("page", "10")],
                ),
                var(
                    "table",
                    "With a range and rows per page.",
                    &[("total", "true"), ("page", "2")],
                ),
                var(
                    "load-more",
                    "A Load more button.",
                    &[("mode", "load-more"), ("total", "true")],
                ),
            ],
            render: pagination,
            code: pagination_code,
            keyboard: vec![
                ("Tab", "Previous, pages, next"),
                ("Enter / Space", "Go to that page"),
            ],
            a11y: vec![
                "A nav landmark; the current page has aria-current=page; previous and next are disabled at the ends.",
            ],
            responsive: vec![
                "Window mode keeps the same width on any screen; the row wraps when it must.",
            ],
            height: 160,
            align: Align::Center,
            live: false,
        },
    ]
}
