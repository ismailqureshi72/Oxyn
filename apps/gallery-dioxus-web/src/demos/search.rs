//! Search and commands (plan #26): command palette / command menu, AI command, AI search bar,
//! settings command, search field, expanding search, shortcut recorder, pagination.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use std::collections::BTreeMap;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        command::{AiAction, Command, Setting, SettingKind, SettingValue, Source},
        search::{Binding, ShortcutGroup},
    },
};
use oxyn_dioxus::CommandPalette;
#[cfg(oxyn_pro)]
use oxyn_dioxus::{AiCommand, AiSearchBar, ExpandingSearch, Pagination, ScriptedAnswer, SearchField, SettingsCommand, ShortcutList, ShortcutRecorder};

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
            .logo("linear")
            .suggested()
            .hint("Linear"),
        Command::new("pr", "Open pull request")
            .group("Suggestions")
            .logo("github")
            .suggested()
            .hint("GitHub"),
        Command::new("slack", "Share to #eng-agents")
            .group("Suggestions")
            .logo("S")
            .suggested()
            .hint("Slack"),
        Command::new("doc", "Draft release notes")
            .group("Suggestions")
            .logo("notion")
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

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
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
                var("command-menu", "Menu: logos, ⌘1–⌘5, tray.", &[]),
                var(
                    "palette",
                    "A palette with matched letters.",
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
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 8] = ["command-palette", "ai-command", "ai-search-bar", "settings-command", "search-field", "expanding-search", "shortcut-recorder", "pagination"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
