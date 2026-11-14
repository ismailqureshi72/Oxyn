//! Navigation & layout, choice & feedback, and app chrome (plan #27, chunk 1).

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        chrome::{Note, ReactionCount, ReactionOption, Workspace},
        layout::{Crumb, DockItem, Pane, RadioCard, Tab, TocItem, ToggleItem, TreeNode},
    },
};
use oxyn_dioxus::{
    Accordion, AccordionItem, Breadcrumb, ChipGroup, CodeBlock, CopyButton, Dock, Drawer,
    EmptyPreset, ExpandableText, HoverCard, NotificationCenter, PageHeader, PaneView, Profile,
    RadioCards, Rating, Reactions, ReorderItem, ReorderableList, ResizablePanels, Tabs,
    TeamSwitcher, TocProgress, TocSection, ToggleGroup, TreeView, UserMenu,
};

use super::{Code, mk, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

fn s(x: &str) -> String {
    x.to_string()
}

// ─────────────────────────────────────────── tabs ──

fn tab_set(v: &Vals) -> (Vec<Tab>, Vec<String>) {
    let counts = v.b("counts");
    let many = v.b("many");
    let mut t = vec![
        Tab::new("overview", "Overview"),
        Tab::new("runs", "Runs"),
        Tab::new("approvals", "Approvals"),
        Tab::new("settings", "Settings"),
    ];
    if counts {
        t[1] = t[1].clone().count(24);
        t[2] = t[2].clone().count(3);
    }
    if v.b("disabled") {
        t[3] = t[3].clone().disabled();
    }
    let mut p = vec![
        s("Atlas is healthy. 12 runs today, 2 waiting on review, $18.40 spent."),
        s("Run 251 is streaming. Run 250 finished with 148 tests passing."),
        s("Three approvals waiting: push to main, send 40 emails, delete a branch."),
        s("Models, tools and permission rules for this project."),
    ];
    if many {
        for (k, l) in ["Files", "Memory", "Billing", "Members", "Audit log"]
            .iter()
            .enumerate()
        {
            t.push(Tab::new(&format!("x{k}"), l));
            p.push(format!("{l} for Atlas."));
        }
    }
    (t, p)
}

fn tabs(v: &Vals, _: ClockMs) -> Element {
    let variant = v.string("variant");
    if variant == "terminal" {
        let t = vec![
            Tab::new("t1", "cargo test")
                .closable()
                .icon("prompt")
                .busy(true),
            Tab::new("t2", "npm run dev")
                .closable()
                .icon("prompt")
                .busy(true),
            Tab::new("t3", "zsh").closable().icon("prompt"),
        ];
        let p = vec![
            s(
                "$ cargo test -p oxyn-core\n   Compiling oxyn-core v0.3.0\n    Running unittests src/lib.rs\ntest primitives::layout::tabs ... ok\ntest primitives::layout::tree ... ok\n▌",
            ),
            s(
                "$ npm run dev\n  VITE v6.2  ready in 412 ms\n  ➜  Local:   http://localhost:5173/\n▌",
            ),
            s(
                "$ git status\nOn branch feat/app-shell-chat\nnothing to commit, working tree clean\n$ ▌",
            ),
        ];
        return rsx! { div { class: "oxd-w560", Tabs { tabs: t, panels: p, variant: "terminal", addable: true, label: "Terminals" } } };
    }
    let (t, p) = tab_set(v);
    rsx! {
        div { class: "oxd-w520",
            Tabs { tabs: t, panels: p, variant, size: v.string("size"), manual: v.b("manual"), full: v.b("full"), label: "Project" }
        }
    }
}

fn tabs_code(v: &Vals) -> String {
    Code::new("Tabs")
        .pre("let tabs = vec![Tab::new(\"overview\", \"Overview\"), Tab::new(\"runs\", \"Runs\").count(24)];")
        .raw("tabs", "tabs")
        .raw("panels", "vec![\"Summary\".into(), \"Recent runs\".into()]")
        .s("variant", v.s("variant"), "pill")
        .s("size", v.s("size"), "md")
        .b("manual", v.b("manual"))
        .b("full", v.b("full"))
        .raw("on_change", "move |v: String| route.set(v)")
        .done()
}

// ───────────────────────────────────── breadcrumb ──

fn crumbs(v: &Vals, _: ClockMs) -> Element {
    let icons = v.b("icons");
    let mut items = [
        Crumb::new("Acme").href("#"),
        Crumb::new("Projects").href("#"),
        Crumb::new("Atlas").href("#"),
        Crumb::new("Runs").href("#"),
        Crumb::new("Run 251"),
    ];
    if icons {
        items[0] = items[0].clone().icon("building");
        items[2] = items[2].clone().icon("folder");
        items[4] = items[4].clone().icon("play");
    }
    let depth = v.u("depth").clamp(2, 5);
    let items: Vec<Crumb> = items[5 - depth..].to_vec();
    rsx! { Breadcrumb { items, max: v.u("max"), separator: v.string("separator") } }
}

fn crumbs_code(v: &Vals) -> String {
    Code::new("Breadcrumb")
        .raw("items", "vec![Crumb::new(\"Acme\").href(\"/\"), Crumb::new(\"Atlas\").href(\"/atlas\"), Crumb::new(\"Run 251\")]")
        .raw("max", v.s("max"))
        .s("separator", v.s("separator"), "chevron")
        .done()
}

// ─────────────────────────────────────── accordion ──

fn acc_items(meta: bool) -> Vec<AccordionItem> {
    let mut v = vec![
        AccordionItem::new(
            "What can the agent change without asking?",
            "Reads, searches and edits inside the workspace run freely. Pushing, deleting, spending and sending messages always stop for an approval.",
        ),
        AccordionItem::new(
            "Where do runs execute?",
            "Each run gets a fresh sandbox with your repository mounted read-write and the network limited to the domains you allow.",
        ),
        AccordionItem::new(
            "How is cost counted?",
            "Tokens in and out per model, plus tool time. The run meter shows a live total and stops the run at your budget.",
        ),
        AccordionItem::new(
            "Can I rewind a run?",
            "Yes. Every tool call is a checkpoint; restore one and the next attempt branches from it.",
        ),
    ];
    if meta {
        v[0] = v[0].clone().meta("Policy");
        v[1] = v[1].clone().meta("Sandbox");
        v[2] = v[2].clone().meta("Billing");
        v[3] = v[3].clone().meta("Runs");
    }
    v
}

fn accordion(v: &Vals, _: ClockMs) -> Element {
    let open = match v.s("open") {
        "none" => None,
        x => x.parse().ok(),
    };
    rsx! {
        div { class: "oxd-w560",
            Accordion { items: acc_items(v.b("meta")), default_open: open, multiple: v.b("multiple"), size: v.string("size"), variant: v.string("variant"), controls: v.b("multiple") }
        }
    }
}

fn accordion_code(v: &Vals) -> String {
    Code::new("Accordion")
        .raw(
            "items",
            "vec![AccordionItem::new(\"Question\", \"Answer\")]",
        )
        .b("multiple", v.b("multiple"))
        .s("size", v.s("size"), "md")
        .s("variant", v.s("variant"), "plain")
        .done()
}

// ─────────────────────────────── resizable panels ──

fn resizable(v: &Vals, _: ClockMs) -> Element {
    let three = v.b("three");
    let mut panes = vec![
        PaneView {
            pane: Pane::new("files", "Files", 24.0).min(140.0).collapsible(),
            title: s("Files"),
            lines: ["src/", "  main.rs", "  auth.rs", "tests/", "Cargo.toml"]
                .iter()
                .map(|x| s(x))
                .collect(),
        },
        PaneView {
            pane: Pane::new("editor", "Editor", 52.0).min(200.0),
            title: s("auth.rs"),
            lines: [
                "fn verify(token: &str) -> Result<Claims> {",
                "    let key = keys::current()?;",
                "    decode(token, &key)",
                "}",
            ]
            .iter()
            .map(|x| s(x))
            .collect(),
        },
    ];
    if three {
        panes.push(PaneView {
            pane: Pane::new("agent", "Agent", 24.0).min(160.0).collapsible(),
            title: s("Agent"),
            lines: ["Reading auth.rs", "Found 2 call sites", "Proposing a fix…"]
                .iter()
                .map(|x| s(x))
                .collect(),
        });
    }
    rsx! {
        div { style: "width:100%;max-width:760px",
            ResizablePanels { panes, label: "Editor layout", height: v.f("height") as f32, vertical: v.b("vertical"), handle: v.string("handle"), readout: v.b("readout") }
        }
    }
}

fn resizable_code(v: &Vals) -> String {
    Code::new("ResizablePanels")
        .pre("let panes = vec![")
        .pre("    PaneView { pane: Pane::new(\"files\", \"Files\", 24.0).min(140.0).collapsible(), title: \"Files\".into(), lines: vec![] },")
        .pre("    PaneView { pane: Pane::new(\"editor\", \"Editor\", 76.0).min(200.0), title: \"Editor\".into(), lines: vec![] },")
        .pre("];")
        .raw("panes", "panes")
        .raw("height", v.s("height"))
        .b("vertical", v.b("vertical"))
        .s("handle", v.s("handle"), "line")
        .raw("on_layout_change", "move |sizes: Vec<f32>| save(sizes)")
        .done()
}

// ─────────────────────────────────────────── drawer ──

fn drawer(v: &Vals, _: ClockMs) -> Element {
    let side = v.string("side");
    rsx! {
        div { class: "oxd-app",
            div { class: "oxd-app-bar", span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-title", "Atlas · Runs" } }
            div { class: "oxd-app-body",
                p { class: "oxd-note", "The drawer opens inside this window. Drag its header toward the edge, press Escape or the close button." }
                Drawer { title: "Run 251", description: Some(s("Streaming · started 2 min ago")), side, trigger_label: "Open run details", size: v.f("size") as f32, inset: v.b("inset"), contained: true, footer: if v.b("footer") { Some(s("Open full run")) } else { None },
                    div { class: "oxd-prose",
                        p { "Fixing the flaky auth test in tests/auth.rs. 4 tool calls so far, $0.42 spent." }
                        p { "Next: rerun the suite three times to confirm the fix holds." }
                    }
                }
            }
        }
    }
}

fn drawer_code(v: &Vals) -> String {
    Code::new("Drawer")
        .raw("title", q("Run 251"))
        .s("side", v.s("side"), "right")
        .b("inset", v.b("inset"))
        .raw("children", "rsx! { RunDetails {} }")
        .done()
        .replace("children: rsx! { RunDetails {} },", "RunDetails {}")
}

// ─────────────────────────────────────── hover card ──

fn hover(v: &Vals, _: ClockMs) -> Element {
    let open = v.b("open");
    let kind = v.string("kind");
    let mut p = Profile::new("Maya Patel");
    p.role = Some(s("Staff engineer · Platform"));
    p.bio = Some(s("Owns the sandbox fleet and the agent permission model."));
    p.stats = vec![
        (s("Runs"), s("1,204")),
        (s("Approvals"), s("318")),
        (s("Reviews"), s("92")),
    ];
    p.meta = Some(s("Karachi · 4:12 PM local"));
    let mut a = Profile::new("Atlas Coder");
    a.role = Some(s("Agent · claude-sonnet · 14 tools"));
    a.bio = Some(s(
        "Writes and fixes code in the Atlas repository. Asks before pushing.",
    ));
    a.stats = vec![(s("Runs today"), s("12")), (s("Success"), s("94%"))];
    a.agent_hue = Some(s("violet"));
    rsx! {
        div { class: "oxd-prose", style: "max-width:440px;min-height:220px",
            p {
                "Reviewed by "
                if kind == "source" {
                    HoverCard { trigger: "[1] RFC 9068", source: Some((s("JSON Web Token Profile for OAuth 2.0 Access Tokens"), s("rfc-editor.org"), s("Defines a profile for issuing OAuth access tokens in JWT format…"))), open, side: v.string("side"), open_delay: v.u("delay") as u64 }
                } else if kind == "agent" {
                    HoverCard { trigger: "Atlas Coder", profile: Some(a), open, side: v.string("side"), open_delay: v.u("delay") as u64 }
                } else {
                    HoverCard { trigger: "Maya", profile: Some(p), open, side: v.string("side"), open_delay: v.u("delay") as u64 }
                }
                " before the agent merged the fix."
            }
        }
    }
}

fn hover_code(v: &Vals) -> String {
    Code::new("HoverCard")
        .raw("trigger", q("Maya"))
        .raw("profile", "Some(profile)")
        .s("side", v.s("side"), "bottom")
        .raw("open_delay", v.s("delay"))
        .done()
}

// ──────────────────────────────────────── tree view ──

fn tree_nodes(badges: bool) -> Vec<TreeNode> {
    let b = |n: TreeNode, x: &str| if badges { n.badge(x) } else { n };
    vec![
        TreeNode::folder(
            "src",
            "src",
            vec![
                TreeNode::folder(
                    "auth",
                    "auth",
                    vec![
                        b(TreeNode::leaf("auth/mod", "mod.rs"), "M"),
                        b(TreeNode::leaf("auth/token", "token.rs"), "A"),
                    ],
                ),
                b(TreeNode::leaf("main", "main.rs"), "●"),
                TreeNode::leaf("lib", "lib.rs"),
            ],
        ),
        TreeNode::folder(
            "tests",
            "tests",
            vec![
                b(TreeNode::leaf("t/auth", "auth.rs"), "M"),
                b(TreeNode::leaf("t/old", "legacy.rs"), "D"),
            ],
        ),
        TreeNode::leaf("cargo", "Cargo.toml"),
        TreeNode::leaf("readme", "README.md"),
    ]
}

fn tree(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w320",
            TreeView { nodes: tree_nodes(v.b("badges")), expanded: vec![s("src"), s("auth")], selected: Some(s("auth/token")), guides: v.b("guides"), density: v.string("density") }
        }
    }
}

fn tree_code(v: &Vals) -> String {
    Code::new("TreeView")
        .raw("nodes", "vec![TreeNode::folder(\"src\", \"src\", vec![TreeNode::leaf(\"main\", \"main.rs\").badge(\"M\")])]")
        .raw("expanded", "vec![\"src\".into()]")
        .s("density", v.s("density"), "comfortable")
        .raw("on_select", "move |id: String| open(id)")
        .done()
}

// ─────────────────────────────────── reorderable ──

fn reorder(v: &Vals, _: ClockMs) -> Element {
    let items = vec![
        ReorderItem::new(
            "a",
            "Ship pricing page",
            Some("Queued after the current run"),
        ),
        ReorderItem::new("b", "Fix flaky auth test", Some("tests/auth.rs")),
        ReorderItem::new("c", "Write release notes", Some("v0.3.0")),
        ReorderItem::new("d", "Bump dependencies", None),
    ];
    rsx! { div { class: "oxd-w420", ReorderableList { items, label: "Agent queue", numbered: v.b("numbered"), removable: v.b("removable") } } }
}

fn reorder_code(v: &Vals) -> String {
    Code::new("ReorderableList")
        .raw(
            "items",
            "vec![ReorderItem::new(\"a\", \"Ship pricing page\", None)]",
        )
        .b("removable", v.b("removable"))
        .raw("on_reorder", "move |ids: Vec<String>| queue.set(ids)")
        .done()
}

// ───────────────────────────────────── toggle group ──

fn toggle(v: &Vals, _: ClockMs) -> Element {
    let multiple = v.b("multiple");
    let set = v.string("set");
    let (items, value, label) = if set == "text" {
        (
            vec![
                ToggleItem::text("chat", "Chat"),
                ToggleItem::text("agent", "Agent"),
                ToggleItem::text("plan", "Plan"),
            ],
            vec![s("agent")],
            "Mode",
        )
    } else {
        (
            vec![
                ToggleItem::icon("b", "Bold", "type"),
                ToggleItem::icon("i", "Italic", "pencil"),
                ToggleItem::icon("code", "Code", "code"),
                ToggleItem::icon("link", "Link", "link"),
                ToggleItem::icon("quote", "Quote", "comment"),
            ],
            if multiple {
                vec![s("b"), s("i")]
            } else {
                vec![s("b")]
            },
            "Text style",
        )
    };
    rsx! { ToggleGroup { items, label, multiple: multiple && set != "text", value, size: v.string("size"), disabled: v.b("disabled") } }
}

fn toggle_code(v: &Vals) -> String {
    Code::new("ToggleGroup")
        .raw("items", "vec![ToggleItem::icon(\"b\", \"Bold\", \"type\"), ToggleItem::icon(\"i\", \"Italic\", \"pencil\")]")
        .raw("label", q("Text style"))
        .b("multiple", v.b("multiple"))
        .s("size", v.s("size"), "md")
        .done()
}

// ─────────────────────────────────────── chip group ──

fn chips(v: &Vals, _: ClockMs) -> Element {
    let options: Vec<(String, String)> = [
        ("web", "Web search"),
        ("code", "Code"),
        ("docs", "Docs"),
        ("issues", "Issues"),
        ("slack", "Slack"),
        ("drive", "Drive"),
        ("mail", "Mail"),
        ("calendar", "Calendar"),
    ]
    .iter()
    .map(|(a, b)| (s(a), s(b)))
    .collect();
    let counts = if v.b("counts") {
        vec![12, 48, 7, 31, 220, 15, 96, 4]
    } else {
        vec![]
    };
    rsx! {
        div { class: "oxd-w420",
            ChipGroup { options, label: "Sources", value: vec![s("code"), s("mail")], multiple: v.b("multiple"), max_visible: v.u("max_visible"), counts, variant: v.string("variant"), clearable: v.b("clearable") }
        }
    }
}

fn chips_code(v: &Vals) -> String {
    Code::new("ChipGroup")
        .raw(
            "options",
            "vec![(\"web\".into(), \"Web search\".into()), (\"code\".into(), \"Code\".into())]",
        )
        .raw("label", q("Sources"))
        .raw("max_visible", v.s("max_visible"))
        .s("variant", v.s("variant"), "outline")
        .raw("on_change", "move |v: Vec<String>| filters.set(v)")
        .done()
}

// ────────────────────────────────────── radio cards ──

fn rcards(v: &Vals, _: ClockMs) -> Element {
    let set = v.string("set");
    let options = if set == "permissions" {
        vec![
            RadioCard::new("ask", "Ask every time")
                .desc("The agent stops before each edit or command.")
                .icon("hand")
                .tone("low"),
            RadioCard::new("edits", "Auto-accept edits")
                .desc("Edits apply; commands and network still ask.")
                .icon("pencil")
                .tone("medium"),
            RadioCard::new("full", "Full access")
                .desc("No prompts. Use only in a disposable sandbox.")
                .icon("bolt")
                .tone("high"),
        ]
    } else {
        vec![
            RadioCard::new("free", "Hobby")
                .desc("1 agent, community models")
                .meta("$0"),
            RadioCard::new("pro", "Pro")
                .desc("5 agents, frontier models, approvals")
                .meta("$20/mo"),
            RadioCard::new("team", "Team")
                .desc("Shared runs, audit log, SSO")
                .meta("$40/seat"),
            RadioCard::new("ent", "Enterprise")
                .disabled("Talk to sales to enable")
                .meta("Custom"),
        ]
    };
    let value = if set == "permissions" {
        s("edits")
    } else {
        s("pro")
    };
    rsx! {
        div { style: "width:100%;max-width:640px",
            RadioCards { options, value: Some(value), layout: v.string("layout"), min_column: v.f("min_column") as f32, label: "Plan", disabled: v.b("disabled") }
        }
    }
}

fn rcards_code(v: &Vals) -> String {
    Code::new("RadioCards")
        .raw(
            "options",
            "vec![RadioCard::new(\"pro\", \"Pro\").desc(\"5 agents\").meta(\"$20/mo\")]",
        )
        .s("layout", v.s("layout"), "grid")
        .raw("min_column", v.s("min_column"))
        .raw("name", "Some(\"plan\".into())")
        .done()
}

// ─────────────────────────────────────────── rating ──

fn rating(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        Rating { label: "How was this answer?", value: v.f("value") as f32, max: v.u("max") as u32, allow_half: v.b("half"), read_only: v.b("read_only"), size: v.string("size"), glyph: v.string("glyph"), show_value: v.b("show_value"), description: if v.b("hint") { Some(s("Your rating trains nothing; it helps us spot bad answers.")) } else { None } }
    }
}

fn rating_code(v: &Vals) -> String {
    Code::new("Rating")
        .raw("label", q("How was this answer?"))
        .raw("value", v.s("value"))
        .b("allow_half", v.b("half"))
        .b("read_only", v.b("read_only"))
        .s("glyph", v.s("glyph"), "star")
        .raw("on_change", "move |v: f32| send_feedback(v)")
        .done()
}

// ─────────────────────────────────────── copy button ──

fn copy(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-btnrow",
            CopyButton { value: v.string("value"), label: v.string("label"), icon_only: v.b("icon_only"), variant: v.string("variant"), size: v.string("size"), fail: v.b("fail") }
            code { class: "oxd-muted", "{v.s(\"value\")}" }
        }
    }
}

fn copy_code(v: &Vals) -> String {
    Code::new("CopyButton")
        .raw("value", q(v.s("value")))
        .s("label", v.s("label"), "Copy")
        .b("icon_only", v.b("icon_only"))
        .s("variant", v.s("variant"), "outline")
        .raw("on_copy", "move |text: String| clipboard.write(text)")
        .done()
}

// ───────────────────────────────────────── reactions ──

fn react_opts() -> Vec<ReactionOption> {
    vec![
        ReactionOption::new("up", "Thumbs up", "👍"),
        ReactionOption::new("love", "Love", "❤️"),
        ReactionOption::new("laugh", "Laugh", "😄"),
        ReactionOption::new("fire", "Fire", "🔥"),
        ReactionOption::new("eyes", "Looking", "👀"),
        ReactionOption::new("ship", "Ship it", "🚀"),
    ]
}

fn reactions(v: &Vals, _: ClockMs) -> Element {
    let value = vec![
        ReactionCount {
            id: s("up"),
            count: 4,
            reacted: true,
        },
        ReactionCount {
            id: s("fire"),
            count: 2,
            reacted: false,
        },
        ReactionCount {
            id: s("ship"),
            count: 1,
            reacted: false,
        },
    ];
    rsx! { div { style: "padding-top:56px", Reactions { options: react_opts(), value, compact: v.b("compact"), picker_open: v.b("picker") } } }
}

fn reactions_code(v: &Vals) -> String {
    Code::new("Reactions")
        .raw(
            "options",
            "vec![ReactionOption::new(\"up\", \"Thumbs up\", \"👍\")]",
        )
        .raw(
            "value",
            "vec![ReactionCount { id: \"up\".into(), count: 4, reacted: true }]",
        )
        .b("compact", v.b("compact"))
        .raw("on_change", "move |(id, on): (String, bool)| save(id, on)")
        .done()
}

// ───────────────────────────────────── expandable text ──

const LONG: &str = "The agent read 14 files, ran the test suite three times and traced the flaky failure to a clock skew in the token validator: tokens minted in the last second of a minute were rejected when the check ran a few milliseconds later.\n\nThe fix adds a 30 second leeway to the expiry check and a regression test that freezes the clock at 59.999 seconds. All 148 tests pass, including the new one, across three consecutive runs.\n\nNo other call sites depend on the strict check. The change is two lines in auth/token.rs and one new test.";

fn xtext(v: &Vals, _: ClockMs) -> Element {
    rsx! { ExpandableText { text: LONG, lines: v.u("lines") as u32, fade: v.b("fade"), more_label: v.string("more"), less_label: v.string("less") } }
}

fn xtext_code(v: &Vals) -> String {
    Code::new("ExpandableText")
        .raw("text", "summary")
        .raw("lines", v.s("lines"))
        .b("fade", v.b("fade"))
        .done()
}

// ──────────────────────────────────────────── dock ──

fn dock_items() -> Vec<DockItem> {
    vec![
        DockItem::new("select", "Select", "cursor").key("V"),
        DockItem::new("pen", "Pen", "pencil").key("P"),
        DockItem::new("shapes", "Shapes", "symbol").group(vec![
            DockItem::new("rect", "Rectangle", "symbol").key("R"),
            DockItem::new("ell", "Ellipse", "globe").key("O"),
            DockItem::new("line", "Line", "minus").key("L"),
        ]),
        DockItem::new("text", "Text", "type").key("T"),
        DockItem::new("comments", "Comments", "comment").badge(3),
        DockItem::new("agent", "Ask agent", "spark").key("⌘J"),
    ]
}

fn dock(v: &Vals, _: ClockMs) -> Element {
    rsx! { div { style: "padding-top:150px", Dock { items: dock_items(), value: Some(s("pen")), magnify: v.b("magnify"), reorder: v.b("reorder"), position: v.string("position") } } }
}

fn dock_code(v: &Vals) -> String {
    Code::new("Dock")
        .raw("items", "vec![DockItem::new(\"pen\", \"Pen\", \"pencil\").key(\"P\"), DockItem::new(\"shapes\", \"Shapes\", \"symbol\").group(members)]")
        .raw("value", "Some(\"pen\".into())")
        .b("magnify", v.b("magnify"))
        .b("reorder", v.b("reorder"))
        .done()
}

// ─────────────────────────────────────────── toc ──

fn toc_sections() -> Vec<TocSection> {
    let para = |t: &str| {
        vec![
            s(t),
            s(
                "Agents keep a transcript of every tool call, so you can see what changed and why. Each step records its inputs, its output and the time it took, and anything that touched the outside world asks first.",
            ),
            s("Long runs stream their progress; you can steer, pause or rewind at any point."),
        ]
    };
    vec![
        TocSection {
            item: TocItem::new("intro", "Introduction", 1),
            body: para(
                "Oxyn gives agent apps the parts users actually look at: the composer, the transcript, approvals and run state.",
            ),
        },
        TocSection {
            item: TocItem::new("install", "Install", 1),
            body: para(
                "Add oxyn-core and the renderer you use, then include the token stylesheet once.",
            ),
        },
        TocSection {
            item: TocItem::new("dioxus", "Dioxus", 2),
            body: para(
                "Components hold their own state in signals and report changes through events.",
            ),
        },
        TocSection {
            item: TocItem::new("gpui", "GPUI", 2),
            body: para("Elements are RenderOnce; the host owns the model and the clock."),
        },
        TocSection {
            item: TocItem::new("theming", "Theming", 1),
            body: para(
                "Monochrome by default, semantic colour only for status, risk and agent identity.",
            ),
        },
        TocSection {
            item: TocItem::new("tokens", "Tokens", 3),
            body: para("DTCG tokens compile to CSS variables and Rust constants."),
        },
        TocSection {
            item: TocItem::new("next", "Next steps", 1),
            body: para("Browse the components, then build your first agent screen."),
        },
    ]
}

fn toc(v: &Vals, _: ClockMs) -> Element {
    rsx! { div { style: "width:100%;max-width:760px", TocProgress { sections: toc_sections(), side: v.string("side"), show_time_left: v.b("time"), height: v.f("height") as f32, reading_time: v.f("minutes") as f32 } } }
}

fn toc_code(v: &Vals) -> String {
    Code::new("TocProgress")
        .raw("sections", "sections")
        .s("side", v.s("side"), "right")
        .raw("reading_time", v.s("minutes"))
        .raw("on_active_change", "move |id: String| set_hash(id)")
        .done()
}

// ───────────────────────────────────── page header ──

fn phead(v: &Vals, _: ClockMs) -> Element {
    let tabs = vec![
        Tab::new("activity", "Activity").count(14),
        Tab::new("runs", "Runs").count(251),
        Tab::new("prs", "Pull requests").count(6),
        Tab::new("settings", "Settings"),
    ];
    rsx! {
        div { style: "width:100%;max-width:720px",
            PageHeader { title: "Atlas", crumbs: vec![Crumb::new("Acme").href("#"), Crumb::new("Projects").href("#"), Crumb::new("Atlas")], meta: "Rust · 3,412 files · updated 2 min ago by Atlas Coder", tabs, status: if v.b("status") { Some(s("Agent running")) } else { None }, folded: v.b("folded"), fold_at: v.f("fold_at") as f32, height: 380.0 }
        }
    }
}

fn phead_code(v: &Vals) -> String {
    Code::new("PageHeader")
        .raw("title", q("Atlas"))
        .raw("crumbs", "crumbs")
        .raw("tabs", "tabs")
        .raw("fold_at", v.s("fold_at"))
        .done()
}

// ──────────────────────────────────────── user menu ──

fn umenu(v: &Vals, _: ClockMs) -> Element {
    let items = vec![
        (s("Profile"), s("users"), s("⌘ P")),
        (s("Settings"), s("cog"), s("⌘ ,")),
        (s("Billing"), s("table"), String::new()),
        (s("Keyboard shortcuts"), s("keyboard"), s("?")),
    ];
    let up = v.s("side") == "up";
    rsx! {
        div { class: "oxd-app", style: "height:440px",
            div { class: "oxd-app-bar", span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-title", "Atlas" } span { class: "oxd-app-sp" }
                if !up { UserMenu { name: "Ismail Qureshi", email: "ismail@oxyn.dev", plan: Some(s("Pro")), items: items.clone(), show_name: v.b("show_name"), show_theme: v.b("theme"), show_status: v.b("status"), usage: if v.b("usage") { Some((312.0, 500.0)) } else { None }, open: v.b("open") } }
            }
            div { class: "oxd-app-body", style: "flex:1;justify-content:flex-end",
                if up { UserMenu { name: "Ismail Qureshi", email: "ismail@oxyn.dev", plan: Some(s("Pro")), items, show_name: true, show_theme: v.b("theme"), show_status: v.b("status"), usage: if v.b("usage") { Some((312.0, 500.0)) } else { None }, side: "up", align: "start", open: v.b("open") } }
            }
        }
    }
}

fn umenu_code(v: &Vals) -> String {
    Code::new("UserMenu")
        .raw("name", q("Ismail Qureshi"))
        .raw("email", q("ismail@oxyn.dev"))
        .raw("plan", "Some(\"Pro\".into())")
        .raw(
            "items",
            "vec![(\"Settings\".into(), \"cog\".into(), \"⌘ ,\".into())]",
        )
        .b("show_status", v.b("status"))
        .raw("on_sign_out", "move |_| sign_out()")
        .done()
}

// ──────────────────────────────────── team switcher ──

fn workspaces() -> Vec<Workspace> {
    vec![
        Workspace::new("acme", "Acme")
            .plan("Pro")
            .role("Owner")
            .members(12)
            .tile("violet"),
        Workspace::new("lumen", "Lumen Labs")
            .plan("Team")
            .role("Admin")
            .members(48)
            .tile("blue"),
        Workspace::new("oss", "Oxyn OSS")
            .plan("Free")
            .role("Member")
            .members(5)
            .tile("green"),
        Workspace::new("personal", "Personal")
            .plan("Free")
            .role("Owner"),
    ]
}

fn team(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-app", style: "height:420px;width:420px",
            div { class: "oxd-app-body", style: "padding:12px",
                div { style: "width:240px", TeamSwitcher { workspaces: workspaces(), value: Some(s("acme")), compact: v.b("compact"), open: v.b("open"), delay: v.u("delay") as u32 } }
            }
        }
    }
}

fn team_code(v: &Vals) -> String {
    Code::new("TeamSwitcher")
        .raw("workspaces", "vec![Workspace::new(\"acme\", \"Acme\").plan(\"Pro\").role(\"Owner\").tile(\"violet\")]")
        .raw("value", "Some(\"acme\".into())")
        .b("compact", v.b("compact"))
        .raw("on_change", "move |id: String| switch_to(id)")
        .done()
}

// ─────────────────────────────── notification center ──

fn notes() -> Vec<Note> {
    vec![
        Note::new(
            "n1",
            "Atlas Coder",
            "needs approval to push to main",
            "4 files changed in auth/. Tests pass. Review the diff before it pushes.",
            "Today",
            "2m",
        )
        .kind("approval"),
        Note::new(
            "n2",
            "Run 251",
            "finished",
            "148 tests passed · $0.84 · 6 min 12 s.",
            "Today",
            "14m",
        )
        .kind("run"),
        Note::new(
            "n3",
            "Maya",
            "mentioned you in #platform",
            "@ismail can you check the sandbox quota before Friday?",
            "Today",
            "1h",
        )
        .kind("mention")
        .read(),
        Note::new(
            "n4",
            "Billing",
            "usage passed 60% of the monthly budget",
            "312 of 500 credits used. Runs keep going until 100%.",
            "Earlier",
            "Yesterday",
        )
        .read(),
        Note::new(
            "n5",
            "Research agent",
            "finished “Q3 churn drivers”",
            "12 sources, 3 claims flagged as unverified.",
            "Earlier",
            "2d",
        )
        .kind("run"),
    ]
}

fn ncenter(v: &Vals, _: ClockMs) -> Element {
    if v.b("inline") {
        return rsx! { div { class: "oxd-w420", NotificationCenter { items: notes(), inline: true, view: v.string("view") } } };
    }
    rsx! {
        div { class: "oxd-app", style: "height:480px",
            div { class: "oxd-app-bar", span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-title", "Atlas" } span { class: "oxd-app-sp" }
                NotificationCenter { items: notes(), open: v.b("open"), view: v.string("view") }
            }
        }
    }
}

fn ncenter_code(v: &Vals) -> String {
    Code::new("NotificationCenter")
        .raw("items", "vec![Note::new(\"n1\", \"Atlas\", \"needs approval\", \"…\", \"Today\", \"2m\").kind(\"approval\")]")
        .b("inline", v.b("inline"))
        .s("view", v.s("view"), "all")
        .done()
}

// ─────────────────────────────────────── code block ──

const RUST_SRC: &str = "use oxyn_dioxus::Composer;\n\n// The prompt box, wired to your agent.\n#[component]\nfn App() -> Element {\n    let mut runs = use_signal(Vec::new);\n    rsx! {\n        Composer {\n            placeholder: \"Ask Atlas…\",\n            on_submit: move |s: Submission| runs.write().push(s.text),\n        }\n    }\n}\n\nfn budget(tokens: u32) -> f64 {\n    tokens as f64 * 0.000_003\n}";
const DIFF_SRC: &str = " fn verify(token: &str) -> Result<Claims> {\n-    let exp = claims.exp;\n-    if exp < now() { return Err(Expired) }\n+    let exp = claims.exp + LEEWAY_SECS;\n+    if exp < now() {\n+        return Err(Expired);\n+    }\n     Ok(claims)\n }";
const TS_SRC: &str = "import { createAgent } from \"@acme/agent\";\n\nconst agent = createAgent({ model: \"sonnet\", tools: [\"search\", \"shell\"] });\n// Stream tokens into the transcript\nfor await (const chunk of agent.run(\"Fix the flaky test\")) {\n  render(chunk);\n}";

fn code(v: &Vals, _: ClockMs) -> Element {
    let lang = v.string("language");
    let (src, file) = match lang.as_str() {
        "diff" => (DIFF_SRC, "auth/token.rs"),
        "ts" => (TS_SRC, "agent.ts"),
        _ => (RUST_SRC, "src/main.rs"),
    };
    rsx! {
        div { class: "oxd-w560",
            CodeBlock { code: src, filename: Some(s(file)), language: lang, max_lines: v.u("max_lines"), line_numbers: v.b("line_numbers"), highlight_lines: if v.b("highlight") { vec![6, 7] } else { vec![] }, wrap: v.b("wrap"), streaming: v.b("streaming"), variant: v.string("variant") }
        }
    }
}

fn code_code(v: &Vals) -> String {
    Code::new("CodeBlock")
        .raw("code", "source")
        .raw("filename", "Some(\"src/main.rs\".into())")
        .s("language", v.s("language"), "tsx")
        .raw("max_lines", v.s("max_lines"))
        .b("line_numbers", v.b("line_numbers"))
        .done()
}

// ───────────────────────────────────── empty states ──

fn empties(v: &Vals, _: ClockMs) -> Element {
    let prompts = if v.b("prompts") {
        vec![
            s("Summarise this repo"),
            s("Find flaky tests"),
            s("Draft release notes"),
        ]
    } else {
        vec![]
    };
    rsx! { EmptyPreset { preset: v.string("preset"), query: v.string("query"), layout: v.string("layout"), prompts } }
}

fn empties_code(v: &Vals) -> String {
    Code::new("EmptyPreset")
        .s("preset", v.s("preset"), "first-run")
        .opt("query", v.s("query"))
        .s("layout", v.s("layout"), "framed")
        .done()
}

/// Chunk-1 demos.
pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["pill", "underline", "enclosed", "terminal"],
                    "pill",
                ),
                sel("size", "Size", &["sm", "md"], "md"),
                tog("counts", "Counts", true),
                tog("manual", "Manual activation", false),
                tog("many", "Overflowing tabs", false),
                tog("full", "Fill width", false),
                tog("disabled", "Disabled tab", false),
            ],
            variants: vec![
                var("pill", "A gliding pill over a sunken strip.", &[]),
                var(
                    "underline",
                    "An underline that slides.",
                    &[("variant", "underline")],
                ),
                var(
                    "enclosed",
                    "Folder tabs over a rule (Oxyn plus).",
                    &[("variant", "enclosed")],
                ),
                var(
                    "overflow",
                    "Too many tabs: the strip scrolls with edge buttons.",
                    &[("many", "true")],
                ),
                var(
                    "terminal",
                    "Closable terminal tabs with busy dots and a + (Oxyn plus).",
                    &[("variant", "terminal")],
                ),
            ],
            render: tabs,
            code: tabs_code,
            keyboard: vec![
                ("← →", "Move between tabs (and select, unless manual)"),
                ("Home / End", "First / last tab"),
                ("Enter / Space", "Select the focused tab (manual)"),
                ("Delete", "Close the focused terminal tab"),
                ("Tab", "Into the panel"),
            ],
            a11y: vec![
                "tablist / tab / tabpanel with aria-selected and aria-controls; one tab stop.",
                "Overflow buttons are labelled “Scroll tabs left / right”.",
                "The pill is aria-hidden; panels re-key so the old one never takes focus.",
            ],
            responsive: vec![
                "The strip scrolls horizontally; edge buttons appear only when it overflows.",
                "Tabs keep a 5.5rem minimum width (sm drops it).",
            ],
            height: 260,
            ..mk(
                "tabs",
                "Tabs",
                "navigation",
                "Peer views of one thing: a pill or underline that glides between tabs, panels that slide the way you moved, overflow edge buttons, counts that roll, and closable terminal tabs.",
                "Tabs",
                "TabsEl",
                "tabs",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                rng("depth", "Depth", 2.0, 5.0, 1.0, "5"),
                rng("max", "Collapse beyond (0 = wrap)", 0.0, 5.0, 1.0, "0"),
                sel("separator", "Separator", &["chevron", "slash"], "chevron"),
                tog("icons", "Icons", false),
            ],
            variants: vec![
                var(
                    "path",
                    "A full path; the last crumb is the current page.",
                    &[],
                ),
                var(
                    "collapsed",
                    "The middle folds behind “…” (Oxyn plus).",
                    &[("max", "3")],
                ),
                var(
                    "slash",
                    "Slash separators with icons.",
                    &[("separator", "slash"), ("icons", "true")],
                ),
            ],
            render: crumbs,
            code: crumbs_code,
            keyboard: vec![
                ("Tab", "Between crumb links"),
                ("Enter", "Follow the focused crumb or reveal hidden ones"),
            ],
            a11y: vec![
                "nav landmark with an ordered list.",
                "The last item has aria-current=page and is never a link.",
                "Separators are aria-hidden.",
            ],
            responsive: vec![
                "Wraps onto new lines; crumbs never wrap inside.",
                "Use max to collapse long paths on narrow screens.",
            ],
            height: 160,
            ..mk(
                "breadcrumb",
                "Breadcrumb",
                "navigation",
                "Where a page sits: links from the root to the current page, new crumbs sliding in from a blur, and an optional “…” that folds the middle.",
                "Breadcrumb",
                "BreadcrumbEl",
                "breadcrumb",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                sel("open", "Open at first", &["none", "0", "1", "2"], "0"),
                tog("multiple", "Several open", false),
                sel("size", "Size", &["md", "lg"], "md"),
                sel(
                    "variant",
                    "Variant",
                    &["plain", "separated", "card"],
                    "plain",
                ),
                tog("meta", "Meta column", false),
            ],
            variants: vec![
                var("faq", "Single-open questions.", &[]),
                var(
                    "multiple",
                    "Several open, with Expand all (Oxyn plus).",
                    &[("multiple", "true"), ("variant", "separated")],
                ),
                var("page", "Large page-level FAQ.", &[("size", "lg")]),
                var(
                    "card",
                    "Inside a card with a meta column.",
                    &[("variant", "card"), ("meta", "true")],
                ),
            ],
            render: accordion,
            code: accordion_code,
            keyboard: vec![
                ("Enter / Space", "Toggle the focused row"),
                ("↑ ↓", "Between triggers"),
                ("Home / End", "First / last trigger"),
            ],
            a11y: vec![
                "Triggers sit in h3 with aria-expanded and aria-controls.",
                "Panels are labelled regions and become visibility:hidden once closed.",
            ],
            responsive: vec![
                "Rows fill the width; lg caps answers at 62ch and tightens below 520px.",
            ],
            height: 340,
            align: Align::Start,
            ..mk(
                "accordion",
                "Accordion",
                "navigation",
                "Supporting detail revealed in place: the panel height grows while the answer slides down out of a blur, the chevron springs.",
                "Accordion, AccordionItem",
                "AccordionEl",
                "accordion",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("three", "Third pane", true),
                tog("vertical", "Vertical", false),
                sel("handle", "Handle", &["line", "grip"], "line"),
                tog("readout", "Size readout", false),
                rng("height", "Height", 220.0, 420.0, 20.0, "300"),
            ],
            variants: vec![
                var("ide", "Files, editor and agent panes.", &[]),
                var(
                    "two",
                    "Two panes with a grip handle.",
                    &[("three", "false"), ("handle", "grip")],
                ),
                var(
                    "vertical",
                    "Stacked editor and terminal (Oxyn plus).",
                    &[("vertical", "true"), ("three", "false")],
                ),
                var(
                    "readout",
                    "Shares shown in each pane header.",
                    &[("readout", "true")],
                ),
            ],
            render: resizable,
            code: resizable_code,
            keyboard: vec![
                ("← →", "Move the divider 16 px (Shift: 64 px)"),
                ("Home / End", "To a limit"),
                ("Enter", "Collapse or restore the neighbour"),
                ("Escape", "Cancel a drag"),
                ("Double-click", "Reset the layout"),
            ],
            a11y: vec![
                "Dividers are focusable separators with aria-valuenow, min, max and valuetext.",
                "Collapsed panes are inert; the group is labelled.",
            ],
            responsive: vec![
                "Needs a definite height; panes do not stack on mobile (switch to tabs).",
                "Pointer capture with touch-action none for touch, pen and mouse.",
            ],
            height: 360,
            ..mk(
                "resizable-panels",
                "Resizable panels",
                "navigation",
                "Trade space between panes by dragging or with the keyboard: rubber-band limits, squeezed content fades, collapsible panes leave a restore tab.",
                "ResizablePanels, PaneView",
                "ResizableEl",
                "resizable-panels",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                sel("side", "Side", &["right", "left", "top", "bottom"], "right"),
                rng("size", "Size", 260.0, 460.0, 20.0, "340"),
                tog("inset", "Inset", false),
                tog("footer", "Footer", true),
            ],
            variants: vec![
                var("right", "From the right edge.", &[]),
                var(
                    "bottom",
                    "A bottom drawer.",
                    &[("side", "bottom"), ("size", "260")],
                ),
                var("inset", "Floating inset panel.", &[("inset", "true")]),
            ],
            render: drawer,
            code: drawer_code,
            keyboard: vec![
                ("Escape", "Close and return focus to the trigger"),
                ("Tab / Shift+Tab", "Cycle inside the drawer"),
            ],
            a11y: vec![
                "role=dialog with aria-modal, title and description wired.",
                "Close button labelled; dragging is never the only way to close.",
            ],
            responsive: vec![
                "Side drawers cap at the window width minus a gutter; top / bottom span the width.",
            ],
            height: 460,
            ..mk(
                "drawer",
                "Drawer",
                "navigation",
                "A temporary side surface built on the sheet: titled header that doubles as the drag handle, flick or drag past a third to close, any edge.",
                "Drawer",
                "SheetEl",
                "drawer",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("guides", "Guide lines", true),
                sel(
                    "density",
                    "Density",
                    &["comfortable", "compact"],
                    "comfortable",
                ),
                tog("badges", "Git badges", true),
            ],
            variants: vec![
                var("files", "A repository tree with git badges.", &[]),
                var(
                    "compact",
                    "Compact rows without guides.",
                    &[("density", "compact"), ("guides", "false")],
                ),
                var("plain", "No badges.", &[("badges", "false")]),
            ],
            render: tree,
            code: tree_code,
            keyboard: vec![
                ("↑ ↓", "Next / previous visible row"),
                ("→", "Open a folder, then into it"),
                ("←", "Close a folder, then to its parent"),
                ("Home / End", "First / last row"),
                ("Enter / Space", "Select (and toggle a folder)"),
                ("*", "Expand sibling folders"),
            ],
            a11y: vec![
                "tree / treeitem with aria-level, posinset, setsize, expanded and selected.",
                "Roving tabindex; icons, chevrons and guides are aria-hidden.",
            ],
            responsive: vec!["36 px rows (28 compact), labels ellipsize, 18 px per level."],
            height: 340,
            ..mk(
                "tree-view",
                "Tree view",
                "navigation",
                "Nested folders with keyboard navigation, rows that stagger in when a folder opens, guide lines and git or agent status badges.",
                "TreeView",
                "TreeViewEl",
                "tree-view",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                sel("side", "TOC side", &["right", "left", "bar"], "right"),
                tog("time", "Time left", true),
                rng(
                    "minutes",
                    "Reading minutes (0 = estimate)",
                    0.0,
                    20.0,
                    1.0,
                    "6",
                ),
                rng("height", "Height", 280.0, 460.0, 20.0, "360"),
            ],
            variants: vec![
                var("reading", "Line fills as you scroll the article.", &[]),
                var("left", "Contents on the left.", &[("side", "left")]),
                var(
                    "bar",
                    "A compact progress bar for narrow screens (Oxyn plus).",
                    &[("side", "bar")],
                ),
            ],
            render: toc,
            code: toc_code,
            keyboard: vec![
                ("Tab", "Through the headings"),
                ("Enter", "Scroll the article to the heading"),
            ],
            a11y: vec![
                "nav landmark named by the title, an ordered list of anchor links.",
                "aria-current=location on the current heading; contrast differs, not only the line.",
                "The line and marker are decorative.",
            ],
            responsive: vec!["Below 560px of container width the contents move above the article."],
            height: 420,
            align: Align::Start,
            ..mk(
                "toc-progress",
                "Table of contents progress",
                "navigation",
                "Contents that read along with you: a line fills through the headings, a marker rides to the current section and the time left rolls down to Finished.",
                "TocProgress, TocSection",
                "TocProgressEl",
                "toc-progress",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("status", "Agent status", true),
                tog("folded", "Start folded", false),
                rng("fold_at", "Fold at px", 20.0, 120.0, 4.0, "56"),
            ],
            variants: vec![
                var(
                    "project",
                    "A project header; scroll the body to fold it.",
                    &[],
                ),
                var(
                    "folded",
                    "Already folded into the compact bar.",
                    &[("folded", "true")],
                ),
                var("plain", "No status pill.", &[("status", "false")]),
            ],
            render: phead,
            code: phead_code,
            keyboard: vec![
                ("Tab", "Breadcrumb, actions, tabs"),
                ("← →", "Between tabs"),
            ],
            a11y: vec![
                "Tabs use tab roles; the current crumb carries aria-current.",
                "The icon-only actions trigger is labelled More actions.",
                "Follow, Share, Copy link and Archive report through a polite status region.",
            ],
            responsive: vec!["Action labels hide in the folded bar and in narrow containers."],
            height: 440,
            ..mk(
                "page-header",
                "Page header",
                "navigation",
                "A project header that folds into a compact bar as the page scrolls, with breadcrumb, actions and tabs whose counts roll.",
                "PageHeader",
                "PageHeaderEl",
                "page-header",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("magnify", "Magnify", false),
                tog("reorder", "Alt + arrows reorder", true),
                sel("position", "Position", &["bottom", "left"], "bottom"),
            ],
            variants: vec![
                var("tools", "A canvas tool dock with a group tray.", &[]),
                var(
                    "magnify",
                    "macOS-style magnify (Oxyn plus).",
                    &[("magnify", "true")],
                ),
                var(
                    "vertical",
                    "Vertical dock on the left edge (Oxyn plus).",
                    &[("position", "left")],
                ),
            ],
            render: dock,
            code: dock_code,
            keyboard: vec![
                ("← →", "Along the dock (or inside a tray)"),
                ("Enter / Space", "Choose; on a group, toggle its tray"),
                ("↑", "Open a group's tray"),
                ("↓ / Escape", "Close the tray"),
                ("Alt + ← →", "Move the focused item"),
            ],
            a11y: vec![
                "toolbar with roving tabindex; plain items use aria-pressed.",
                "Groups have aria-haspopup, aria-expanded and aria-controls; closed trays are inert.",
                "Names include the shortcut and badge (“Pen, P”).",
            ],
            responsive: vec!["The floating label shows on hover and focus, not touch."],
            height: 300,
            ..mk(
                "dock",
                "Dock",
                "navigation",
                "A floating tool dock: a label glides between items, a tint springs to the chosen tool, groups grow a tray and items reorder with Alt + arrows.",
                "Dock",
                "DockEl",
                "dock",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                rng("lines", "Lines", 1.0, 6.0, 1.0, "3"),
                tog("fade", "Fade", true),
                txt("more", "More label", "Read more"),
                txt("less", "Less label", "Show less"),
            ],
            variants: vec![
                var("summary", "A run summary clamped to three lines.", &[]),
                var(
                    "one-line",
                    "One line without the fade.",
                    &[("lines", "1"), ("fade", "false")],
                ),
                var(
                    "labels",
                    "Custom labels.",
                    &[("more", "Show full summary"), ("less", "Collapse")],
                ),
            ],
            render: xtext,
            code: xtext_code,
            keyboard: vec![("Enter / Space", "Open or close from the toggle")],
            a11y: vec![
                "The clamp is visual: the whole text stays in the accessibility tree.",
                "The toggle has aria-expanded and aria-controls; labels crossfade in one cell.",
            ],
            responsive: vec!["The clamp is plain CSS, so width and font changes need no script."],
            height: 260,
            ..mk(
                "expandable-text",
                "Expandable text",
                "navigation",
                "Long text clamped to a few lines with a soft fade; Read more glides open, Show less closes, and the toggle only appears when the text overflows.",
                "ExpandableText",
                "ExpandableTextEl",
                "expandable-text",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                sel("set", "Items", &["format", "text"], "format"),
                tog("multiple", "Multiple", true),
                sel("size", "Size", &["sm", "md"], "md"),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "toolbar",
                    "Formatting toolbar: pressed neighbours join.",
                    &[],
                ),
                var(
                    "single",
                    "Single choice: one fill glides.",
                    &[("multiple", "false")],
                ),
                var(
                    "text",
                    "Text options for agent mode.",
                    &[("set", "text"), ("multiple", "false")],
                ),
                var(
                    "dense",
                    "Small, for dense desktop toolbars.",
                    &[("size", "sm")],
                ),
            ],
            render: toggle,
            code: toggle_code,
            keyboard: vec![
                ("Tab", "In and out on one item"),
                ("← →", "Move focus (no press), wrapping"),
                ("Home / End", "First / last"),
                ("Space / Enter", "Toggle or select"),
            ],
            a11y: vec![
                "Single: radiogroup with radios and aria-checked. Multiple: group with aria-pressed.",
                "Icon-only items take their name and tooltip from label.",
            ],
            responsive: vec!["Stays on one line; md items are 36 px for touch."],
            height: 200,
            ..mk(
                "toggle-group",
                "Toggle group",
                "choice",
                "A row of toggles, single or multiple: one fill glides between options, or each fades in and neighbours join like a formatting toolbar.",
                "ToggleGroup",
                "ToggleGroupEl",
                "toggle-group",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("multiple", "Multiple", true),
                rng(
                    "max_visible",
                    "Visible before folding (0 = all)",
                    0.0,
                    8.0,
                    1.0,
                    "5",
                ),
                sel("variant", "Variant", &["outline", "soft"], "outline"),
                tog("counts", "Counts", false),
                tog("clearable", "Clear chip", true),
            ],
            variants: vec![
                var("filters", "Source filters folding behind +N more.", &[]),
                var(
                    "single",
                    "Single choice that can still clear.",
                    &[("multiple", "false"), ("max_visible", "0")],
                ),
                var(
                    "soft",
                    "Soft chips with counts (Oxyn plus).",
                    &[
                        ("variant", "soft"),
                        ("counts", "true"),
                        ("max_visible", "0"),
                    ],
                ),
            ],
            render: chips,
            code: chips_code,
            keyboard: vec![
                ("Arrows", "Between chips, wrapping"),
                ("Home / End", "First / last"),
                ("Enter / Space", "Toggle or open the overflow"),
            ],
            a11y: vec![
                "group with aria-label; chips are buttons with aria-pressed.",
                "Selected chips always stay visible.",
            ],
            responsive: vec!["Chips wrap onto new lines."],
            height: 200,
            ..mk(
                "chip-group",
                "Chip group",
                "choice",
                "Filter chips that morph as you pick them: a check grows in, the label slides, long sets fold behind +N more and selected chips never hide.",
                "ChipGroup",
                "ChipGroupEl",
                "chip-group",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                sel("set", "Options", &["plans", "permissions"], "plans"),
                sel("layout", "Layout", &["grid", "list"], "grid"),
                rng("min_column", "Min column px", 140.0, 260.0, 10.0, "150"),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "plans",
                    "Plans with prices; one card disabled with a reason.",
                    &[],
                ),
                var(
                    "list",
                    "Stacked rows with meta at the end.",
                    &[("layout", "list")],
                ),
                var(
                    "permissions",
                    "Permission modes with risk tones (Oxyn plus).",
                    &[("set", "permissions"), ("layout", "list")],
                ),
            ],
            render: rcards,
            code: rcards_code,
            keyboard: vec![
                ("Tab", "One tab stop"),
                ("Arrows", "Select the next enabled card, wrapping"),
                ("Home / End", "First / last enabled"),
                ("Enter / Space", "Select"),
            ],
            a11y: vec![
                "radiogroup / radio with aria-checked and aria-disabled; described by description or the disabled reason.",
                "Selection shows a ring and a filled dot, not colour alone.",
            ],
            responsive: vec!["auto-fill columns down to one in narrow containers."],
            height: 300,
            ..mk(
                "radio-cards",
                "Radio cards",
                "choice",
                "Option cards that need more than a label: one ring glides to the selected card, price and description slots, radio keyboard behaviour.",
                "RadioCards",
                "RadioCardsEl",
                "radio-cards",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                rng("value", "Value", 0.0, 5.0, 0.5, "4"),
                rng("max", "Stars", 3.0, 10.0, 1.0, "5"),
                tog("half", "Half stars", false),
                tog("read_only", "Read only", false),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                sel("glyph", "Glyph", &["star", "heart", "dot"], "star"),
                tog("show_value", "Value word", true),
                tog("hint", "Hint", false),
            ],
            variants: vec![
                var("stars", "Five stars with a word.", &[]),
                var("half", "Half steps.", &[("half", "true"), ("value", "3.5")]),
                var(
                    "hearts",
                    "Hearts (Oxyn plus).",
                    &[("glyph", "heart"), ("size", "lg")],
                ),
                var(
                    "read-only",
                    "A static image for averages.",
                    &[("read_only", "true"), ("value", "4.5"), ("half", "true")],
                ),
            ],
            render: rating,
            code: rating_code,
            keyboard: vec![
                ("→ ↑ / ← ↓", "Up / down one step"),
                ("Home / End", "Lowest / highest"),
                ("1–9", "That many stars"),
                ("Backspace / Delete", "Clear"),
            ],
            a11y: vec![
                "radiogroup with one radio per step labelled “Great, 4 of 5 stars”; commits announced politely.",
                "Read-only renders role=img with an aria-label.",
            ],
            responsive: vec![
                "36 px touch cells; the word wraps under the stars when space runs out.",
            ],
            height: 200,
            ..mk(
                "rating",
                "Rating",
                "choice",
                "Capture a numeric opinion: stars as a radio group with half steps, scrubbing, clearing and a value word that rolls.",
                "Rating",
                "RatingEl",
                "rating",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("numbered", "Numbers", true),
                tog("removable", "Remove buttons", false),
            ],
            variants: vec![
                var("queue", "An agent queue you can reorder.", &[]),
                var(
                    "editable",
                    "Removable rows (Oxyn plus).",
                    &[("removable", "true")],
                ),
                var("plain", "Without numbers.", &[("numbered", "false")]),
            ],
            render: reorder,
            code: reorder_code,
            keyboard: vec![("↑ ↓ on a handle", "Move that item one place")],
            a11y: vec![
                "Handles are labelled “Reorder X. Position 1 of 4. Use arrow keys.”",
                "Focus stays on the moved handle; moves are announced.",
            ],
            responsive: vec![
                "Only the grip starts a drag (touch-action none), so rows still scroll the page.",
            ],
            height: 320,
            ..mk(
                "reorderable-list",
                "Reorderable list",
                "choice",
                "Reorder by dragging the grip or with arrow keys: rows lift, neighbours slide and the position number rolls.",
                "ReorderableList, ReorderItem",
                "ReorderListEl",
                "reorderable-list",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("compact", "Compact", false),
                tog("picker", "Picker open", false),
            ],
            variants: vec![
                var("bar", "Pills with counts and an add button.", &[]),
                var("picker", "The picker tray open.", &[("picker", "true")]),
                var(
                    "compact",
                    "Compact pills (Oxyn plus).",
                    &[("compact", "true")],
                ),
            ],
            render: reactions,
            code: reactions_code,
            keyboard: vec![
                ("Enter / Space", "Toggle a pill or open the picker"),
                ("Arrows", "Between picker options"),
                ("Escape", "Close the picker"),
            ],
            a11y: vec![
                "group of buttons with aria-pressed and labels like “Love, 4 reactions, including yours”.",
                "The picker is a labelled toolbar; changes are announced.",
            ],
            responsive: vec!["Pills wrap; the tray grows out of the add button."],
            height: 220,
            ..mk(
                "reactions",
                "Reactions",
                "choice",
                "Acknowledge in one tap: reaction pills whose counts roll, your own highlighted, and a picker tray that grows out of the add button.",
                "Reactions",
                "ReactionsEl",
                "reactions",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Start open", true),
                tog("show_name", "Name beside avatar", true),
                tog("theme", "Theme switch", true),
                tog("status", "Status switch", true),
                tog("usage", "Usage meter", true),
                sel("side", "Opens", &["down", "up"], "down"),
            ],
            variants: vec![
                var("header", "In the top bar.", &[]),
                var(
                    "sidebar",
                    "From a sidebar footer, opening upward.",
                    &[("side", "up")],
                ),
                var(
                    "minimal",
                    "Avatar only, links and sign out.",
                    &[
                        ("show_name", "false"),
                        ("status", "false"),
                        ("usage", "false"),
                        ("theme", "false"),
                    ],
                ),
            ],
            render: umenu,
            code: umenu_code,
            keyboard: vec![
                ("Enter / Space / ↓", "Open on the first row"),
                ("↑", "Open on the last row"),
                ("↑ ↓ Home End", "Move, looping"),
                ("← →", "Change theme or status"),
                ("A–Z", "Typeahead"),
                ("Escape / Tab", "Close and return focus"),
            ],
            a11y: vec![
                "Trigger has aria-haspopup=menu, aria-expanded and “Account menu, name, status”.",
                "Rows are menuitems; switches are menuitemradio groups.",
                "Sign out sets aria-busy while it runs.",
            ],
            responsive: vec!["Below 640px the panel becomes a bottom sheet."],
            height: 480,
            ..mk(
                "user-menu",
                "User menu",
                "chrome",
                "Your account behind the avatar: identity, a few links, an inline theme and status switch that keep the menu open, usage, and sign out with a spinner.",
                "UserMenu, PresenceDot",
                "UserMenuEl",
                "user-menu",
                "primitives/chrome.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Start open", true),
                tog("compact", "Tile only (rail)", false),
                rng("delay", "Switch ms", 200.0, 2000.0, 50.0, "650"),
            ],
            variants: vec![
                var("menu", "Search, roles, shortcuts, create and join.", &[]),
                var("closed", "The trigger alone.", &[("open", "false")]),
                var(
                    "rail",
                    "Tile only, for an icon rail (Oxyn plus).",
                    &[("compact", "true")],
                ),
            ],
            render: team,
            code: team_code,
            keyboard: vec![
                ("Enter / ↓", "Open and focus the search"),
                ("Type", "Filter by name, plan or role"),
                ("↑ ↓", "Move the highlight"),
                ("Enter", "Switch, or open Create / Join"),
                ("⌘ / Ctrl + 1–9", "Switch to that workspace"),
                ("Escape", "Back, clear, then close"),
            ],
            a11y: vec![
                "Trigger: aria-haspopup=dialog, aria-expanded, aria-busy while switching.",
                "Search is a combobox with aria-activedescendant; the current workspace is aria-selected with a check.",
                "A polite region announces switches.",
            ],
            responsive: vec!["The panel caps at the viewport minus a gutter."],
            height: 460,
            align: Align::Start,
            ..mk(
                "team-switcher",
                "Team switcher",
                "chrome",
                "A workspace switcher like Linear or Slack: the current tile and plan, search, roles, ⌘1–9, inline create and join, and a shimmering switch.",
                "TeamSwitcher, WorkspaceTile",
                "TeamSwitcherEl",
                "team-switcher",
                "primitives/chrome.rs",
            )
        },
        Demo {
            controls: vec![
                tog("open", "Start open", true),
                tog("inline", "Inline panel", false),
                sel("view", "View", &["all", "unread"], "all"),
            ],
            variants: vec![
                var("bell", "A bell with the panel open.", &[]),
                var("unread", "Unread only.", &[("view", "unread")]),
                var(
                    "inline",
                    "Inline panel for a sidebar (Oxyn plus).",
                    &[("inline", "true")],
                ),
            ],
            render: ncenter,
            code: ncenter_code,
            keyboard: vec![
                ("Enter", "Expand a row (marks it read)"),
                ("Tab", "Rows, dismiss buttons, views and bulk actions"),
            ],
            a11y: vec![
                "The bell label includes the unread count; the badge is aria-hidden.",
                "Rows use aria-expanded; focus moves to the next row after a dismiss.",
                "The summary line is polite.",
            ],
            responsive: vec!["The panel caps at the viewport minus a gutter."],
            height: 520,
            ..mk(
                "notification-center",
                "Notification center",
                "chrome",
                "A home for updates: unread badge, All and Unread views, grouped rows that expand in place, mark all read and clear, with a bell that tilts.",
                "NotificationCenter",
                "NotificationCenterEl",
                "notification-center",
                "primitives/chrome.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Preview", &["person", "agent", "source"], "person"),
                tog("open", "Start open", true),
                sel("side", "Side", &["bottom", "top"], "bottom"),
                rng("delay", "Open delay ms", 0.0, 1000.0, 50.0, "500"),
            ],
            variants: vec![
                var("person", "A person preview.", &[]),
                var(
                    "agent",
                    "An agent with its identity ring (Oxyn plus).",
                    &[("kind", "agent")],
                ),
                var(
                    "source",
                    "A citation source preview (Oxyn plus).",
                    &[("kind", "source")],
                ),
            ],
            render: hover,
            code: hover_code,
            keyboard: vec![
                ("Tab", "Focus opens the card at once"),
                ("Enter", "Open without the delay"),
                ("Escape", "Close and stay closed while hovering"),
            ],
            a11y: vec![
                "role=tooltip; the trigger gets aria-describedby while open.",
                "Focus never moves into the card.",
            ],
            responsive: vec!["18.5rem wide, capped to the viewport; taps toggle on touch."],
            height: 320,
            align: Align::Start,
            ..mk(
                "hover-card",
                "Hover card",
                "chrome",
                "Preview a person, agent or source on hover or focus: a short delay, a grace period while you travel into it, and a fast warm re-open.",
                "HoverCard, Profile",
                "HoverCardEl",
                "hover-card",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                txt("value", "Value", "npx oxyn add composer"),
                txt("label", "Label", "Copy"),
                tog("icon_only", "Icon only", false),
                sel(
                    "variant",
                    "Variant",
                    &["outline", "plain", "solid"],
                    "outline",
                ),
                sel("size", "Size", &["sm", "md"], "md"),
                tog("fail", "Simulate failure", false),
            ],
            variants: vec![
                var("outline", "Bordered with a label.", &[]),
                var(
                    "icon",
                    "Icon only.",
                    &[("icon_only", "true"), ("variant", "plain")],
                ),
                var("solid", "Solid (Oxyn plus).", &[("variant", "solid")]),
                var("failed", "The failure state.", &[("fail", "true")]),
            ],
            render: copy,
            code: copy_code,
            keyboard: vec![("Enter / Space", "Copy")],
            a11y: vec![
                "Named by label even when icon-only.",
                "A polite status announces “Copy: Copied” or “Could not copy”.",
            ],
            responsive: vec!["The label cell reserves its widest state so nothing shifts."],
            height: 180,
            ..mk(
                "copy-button",
                "Copy button",
                "chrome",
                "Copy a value with immediate confirmation: icon and label crossfade to Copied, the check draws in, and it resets after a moment.",
                "CopyButton",
                "CopyButtonEl",
                "copy-button",
                "primitives/navigation.rs",
            )
        },
        Demo {
            controls: vec![
                sel("language", "Language", &["rust", "ts", "diff"], "rust"),
                rng(
                    "max_lines",
                    "Collapse beyond (0 = never)",
                    0.0,
                    20.0,
                    1.0,
                    "10",
                ),
                tog("line_numbers", "Line numbers", true),
                tog("highlight", "Highlight lines", false),
                tog("wrap", "Wrap", false),
                tog("streaming", "Streaming caret", false),
                sel("variant", "Variant", &["default", "bare"], "default"),
            ],
            variants: vec![
                var("source", "Rust with line numbers, collapsed.", &[]),
                var(
                    "diff",
                    "A diff with added and removed lines.",
                    &[("language", "diff"), ("max_lines", "0")],
                ),
                var(
                    "streaming",
                    "A block still streaming (Oxyn plus).",
                    &[
                        ("language", "ts"),
                        ("streaming", "true"),
                        ("max_lines", "0"),
                    ],
                ),
                var(
                    "highlighted",
                    "Highlighted lines.",
                    &[("highlight", "true"), ("max_lines", "0")],
                ),
            ],
            render: code,
            code: code_code,
            keyboard: vec![
                ("Tab", "Copy, the scrollable source, Show all"),
                ("Enter / Space", "Copy or toggle"),
            ],
            a11y: vec![
                "A section labelled “file source code”; the pre is focusable.",
                "Show all has aria-expanded and aria-controls.",
            ],
            responsive: vec!["Scrolls horizontally, or wraps with the wrap toggle."],
            height: 420,
            ..mk(
                "code-block",
                "Code block",
                "chrome",
                "Code with a filename header, light highlighting, copy, wrap and Show all, line numbers, highlighted lines, diffs and a streaming caret.",
                "CodeBlock",
                "CodeBlockEl",
                "code-block",
                "primitives/chrome.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "preset",
                    "Preset",
                    &[
                        "first-run",
                        "no-results",
                        "no-runs",
                        "inbox-zero",
                        "offline",
                        "error",
                        "no-access",
                    ],
                    "first-run",
                ),
                txt("query", "Query (no-results)", "flaky auth"),
                sel("layout", "Layout", &["framed", "bare", "inline"], "framed"),
                tog("prompts", "Starter prompts", false),
            ],
            variants: vec![
                var("first-run", "Nothing created yet.", &[]),
                var(
                    "no-results",
                    "A search that matched nothing.",
                    &[("preset", "no-results")],
                ),
                var(
                    "offline",
                    "Offline.",
                    &[("preset", "offline"), ("layout", "bare")],
                ),
                var(
                    "starter",
                    "First run with starter prompts (Oxyn plus).",
                    &[("prompts", "true")],
                ),
                var(
                    "inline",
                    "One line in a list.",
                    &[("preset", "inbox-zero"), ("layout", "inline")],
                ),
            ],
            render: empties,
            code: empties_code,
            keyboard: vec![("Tab", "To the actions")],
            a11y: vec![
                "role=status with a title; the icon is aria-hidden.",
                "Copy changes roll in place.",
            ],
            responsive: vec!["Centred up to 520px."],
            height: 320,
            ..mk(
                "empty-states",
                "Empty states",
                "chrome",
                "A useful next step when there is nothing to show: first run, no results, no runs, inbox zero, offline, error and no access.",
                "EmptyPreset, EmptyState",
                "EmptyStateEl",
                "empty-states",
                "primitives/navigation.rs",
            )
        },
    ]
}
