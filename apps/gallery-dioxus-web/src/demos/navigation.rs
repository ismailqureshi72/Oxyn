//! Navigation & layout, choice & feedback, and app chrome (plan #27, chunk 1).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        chrome::{Note, ReactionCount, ReactionOption, Workspace},
        layout::{Crumb, DockItem, Pane, RadioCard, Tab, TocItem, ToggleItem, TreeNode},
    },
};
use oxyn_dioxus::{CodeBlock, CopyButton, Tabs};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{Accordion, AccordionItem, Breadcrumb, ChipGroup, Dock, Drawer, EmptyPreset, ExpandableText, HoverCard, NotificationCenter, PageHeader, PaneView, Profile, RadioCards, Rating, Reactions, ReorderItem, ReorderableList, ResizablePanels, TeamSwitcher, TocProgress, TocSection, ToggleGroup, TreeView, UserMenu};

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

/// Chunk-1 demos.
pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
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
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 23] = ["tabs", "breadcrumb", "accordion", "resizable-panels", "drawer", "tree-view", "toc-progress", "page-header", "dock", "expandable-text", "toggle-group", "chip-group", "radio-cards", "rating", "reorderable-list", "reactions", "user-menu", "team-switcher", "notification-center", "hover-card", "copy-button", "code-block", "empty-states"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
