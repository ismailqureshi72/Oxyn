//! Surfaces (plan #26): dialog, sheet, sheet stack, bottom sheet, card, scroll area, control
//! center. Overlays render `contained` inside a small app frame so they open in the stage.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::control_center::{Channel, FocusMode},
};
use oxyn_dioxus::{
    BottomSheet, Card, ControlCenter, Dialog, Line, ScrollArea, Sheet, SheetStack, StackPage,
    StackRow, Switch,
};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

/// A small app frame behind an overlay: a title bar with the trigger and a few faint rows.
pub(crate) fn frame(bar: Element, overlay: Element) -> Element {
    rsx! {
        div { class: "oxd-app",
            div { class: "oxd-app-bar",
                span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" }
                span { class: "oxd-app-title", "Atlas · Run 248" }
                span { class: "oxd-app-sp" }
                {bar}
            }
            div { class: "oxd-app-body", "aria-hidden": "true",
                div { class: "oxd-fl w70" } div { class: "oxd-fl w90" } div { class: "oxd-fl w50" }
                div { class: "oxd-fl w80" } div { class: "oxd-fl w60" } div { class: "oxd-fl w85" }
            }
            {overlay}
        }
    }
}

fn dialog(v: &Vals, _: ClockMs) -> Element {
    let danger = v.s("tone") == "danger";
    let body = if danger {
        rsx! { p { style: "margin:0", "The agent's worktree, 14 checkpoints and the run history for " b { "atlas-refactor" } " will be deleted. This can't be undone." } }
    } else {
        rsx! {
            div { style: "display:flex;flex-direction:column;gap:10px",
                p { style: "margin:0", "Pick what the agent may do without asking. You can change this per run." }
                Switch { label: "Edit files in the worktree", checked: true, size: "sm" }
                Switch { label: "Run tests and linters", checked: true, size: "sm" }
                Switch { label: "Open pull requests", size: "sm" }
            }
        }
    };
    let (title, desc, confirm) = if danger {
        (
            "Delete project?",
            "Atlas · 3 agents · 214 runs",
            "Delete project",
        )
    } else {
        ("Agent permissions", "Applies to new runs in Atlas", "Save")
    };
    rsx! {
        {frame(rsx! {
            Dialog {
                title: if v.s("title").is_empty() { title.to_string() } else { v.string("title") },
                description: Some(desc.to_string()),
                open: v.b("open"),
                trigger_label: Some(if danger { "Delete…".to_string() } else { "Permissions".to_string() }),
                size: v.string("size"),
                tone: v.string("tone"),
                close_on_backdrop: v.b("close_on_backdrop"),
                close_on_escape: v.b("close_on_escape"),
                close_button: v.b("close_button"),
                modal: v.b("modal"),
                contained: true,
                confirm_label: v.b("footer").then(|| confirm.to_string()),
                busy_ms: v.u("busy_ms") as u32,
                {body}
            }
        }, rsx! {})}
    }
}

fn dialog_code(v: &Vals) -> String {
    let mut c = Code::new("Dialog")
        .raw("title", q("Agent permissions"))
        .raw("description", q("Applies to new runs in Atlas"))
        .raw("trigger_label", q("Permissions"))
        .s("size", v.s("size"), "md")
        .s("tone", v.s("tone"), "default");
    if !v.b("close_on_backdrop") {
        c = c.raw("close_on_backdrop", "false");
    }
    if !v.b("modal") {
        c = c.raw("modal", "false");
    }
    c.opt("confirm_label", if v.b("footer") { "Save" } else { "" })
        .raw("busy_ms", v.s("busy_ms"))
        .raw("on_confirm", "move |_| save()")
        .done()
}

fn sheet(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        {frame(rsx! {
            Sheet {
                title: "Run details",
                description: Some("refactor/auth-refresh · started 4 min ago".to_string()),
                open: v.b("open"),
                trigger_label: Some("Details".to_string()),
                side: v.string("side"),
                size: v.f("size") as f32,
                inset: v.b("inset"),
                draggable: v.b("draggable"),
                contained: true,
                footer: v.b("footer").then(|| "Rerun".to_string()),
                div { class: "oxd-kv",
                    div { span { "Model" } b { "Sonnet 4.5 · high effort" } }
                    div { span { "Branch" } b { "refactor/auth-refresh" } }
                    div { span { "Tool calls" } b { "38 (2 failed)" } }
                    div { span { "Tokens" } b { "182k in · 21k out" } }
                    div { span { "Cost" } b { "$1.42" } }
                    div { span { "Checkpoints" } b { "7" } }
                }
            }
        }, rsx! {})}
    }
}

fn sheet_code(v: &Vals) -> String {
    Code::new("Sheet")
        .raw("title", q("Run details"))
        .raw("trigger_label", q("Details"))
        .s("side", v.s("side"), "right")
        .raw("size", format!("{:.0}.0", v.f("size")))
        .b("inset", v.b("inset"))
        .opt("footer", if v.b("footer") { "Rerun" } else { "" })
        .raw("RunDetails", "{}")
        .done()
        .replace("RunDetails: {},", "RunDetails {}")
}

fn pages() -> Vec<StackPage> {
    vec![
        StackPage::new(
            "run",
            "Run settings",
            "How this agent run behaves",
            vec![
                StackRow::push("Tools", "5 on", "tools"),
                StackRow::push("Approvals", "Ask for risky", "approvals"),
                StackRow::text("Model", "Sonnet 4.5"),
                StackRow::switch("Stream reasoning", true),
            ],
        ),
        StackPage::new(
            "tools",
            "Tools",
            "What the agent can call",
            vec![
                StackRow::switch("File search", true),
                StackRow::switch("Edit files", true),
                StackRow::push("Shell", "Sandboxed", "shell"),
                StackRow::switch("Web browse", false),
            ],
        ),
        StackPage::new(
            "shell",
            "Shell access",
            "Commands run in a sandbox with no network",
            vec![
                StackRow::text("Working directory", "~/atlas"),
                StackRow::switch("Allow network", false),
                StackRow::text("Timeout", "120 s"),
            ],
        )
        .footer("Done"),
        StackPage::new(
            "approvals",
            "Approvals",
            "When the agent stops to ask",
            vec![
                StackRow::switch("Writes outside the worktree", true),
                StackRow::switch("Package installs", true),
                StackRow::switch("Pushes and deploys", true),
            ],
        ),
    ]
}

fn sheet_stack(v: &Vals, _: ClockMs) -> Element {
    let depth = v.u("depth");
    let stack: Vec<String> = ["run", "tools", "shell"]
        .iter()
        .take(depth)
        .map(|s| s.to_string())
        .collect();
    rsx! {
        {frame(rsx! {
            SheetStack {
                pages: pages(),
                default_stack: stack,
                mode: v.string("mode"),
                contained: true,
                breadcrumb: v.b("breadcrumb"),
                trigger_label: Some("Run settings".to_string()),
            }
        }, rsx! {})}
    }
}

fn sheet_stack_code(v: &Vals) -> String {
    Code::new("SheetStack")
        .pre("let pages = vec![")
        .pre("    StackPage::new(\"run\", \"Run settings\", \"\", vec![StackRow::push(\"Tools\", \"5 on\", \"tools\")]),")
        .pre("    StackPage::new(\"tools\", \"Tools\", \"\", vec![StackRow::push(\"Shell\", \"Sandboxed\", \"shell\")]),")
        .pre("    StackPage::new(\"shell\", \"Shell access\", \"\", vec![StackRow::switch(\"Allow network\", false)]).footer(\"Done\"),")
        .pre("];")
        .raw("pages", "pages")
        .raw("trigger_label", q("Run settings"))
        .s("mode", v.s("mode"), "auto")
        .b("breadcrumb", v.b("breadcrumb"))
        .raw("on_stack_change", "move |ids: Vec<String>| stack.set(ids)")
        .done()
}

fn bottom_sheet(v: &Vals, _: ClockMs) -> Element {
    let detents: Vec<f32> = match v.s("detents") {
        "single" => vec![0.6],
        "three" => vec![0.3, 0.6, 0.92],
        _ => vec![0.45, 0.92],
    };
    rsx! {
        div { class: "oxd-phone",
            div { class: "oxd-phone-bar", span { "9:41" } span { class: "oxd-app-sp" } Icon2 {} }
            div { class: "oxd-app-body", "aria-hidden": "true", div { class: "oxd-fl w70" } div { class: "oxd-fl w90" } div { class: "oxd-fl w50" } div { class: "oxd-fl w80" } }
            div { class: "oxd-phone-trig",
                BottomSheet {
                    title: "Filter runs",
                    description: Some("12 of 214 runs match".to_string()),
                    open: v.b("open"),
                    trigger_label: Some("Filters".to_string()),
                    detents,
                    initial_detent: v.u("initial"),
                    contained: true,
                    div { class: "oxd-chips",
                        for (i, c) in ["Failed", "Needs approval", "Running", "Done", "This week", "Mine", "Sonnet", "Over $1"].iter().enumerate() {
                            span { key: "{i}", class: if i < 2 { "oxd-chip on" } else { "oxd-chip" }, "{c}" }
                        }
                    }
                    div { class: "oxd-kv", style: "margin-top:12px",
                        div { span { "Agent" } b { "Any" } }
                        div { span { "Branch" } b { "refactor/*" } }
                        div { span { "Started" } b { "Last 7 days" } }
                        div { span { "Sort" } b { "Newest first" } }
                    }
                }
            }
        }
    }
}

#[component]
fn Icon2() -> Element {
    rsx! { span { class: "oxd-batt", "aria-hidden": "true" } }
}

fn bottom_sheet_code(v: &Vals) -> String {
    let d = match v.s("detents") {
        "single" => "vec![0.6]",
        "three" => "vec![0.3, 0.6, 0.92]",
        _ => "vec![0.45, 0.92]",
    };
    Code::new("BottomSheet")
        .raw("title", q("Filter runs"))
        .raw("trigger_label", q("Filters"))
        .raw("detents", d)
        .raw("initial_detent", v.s("initial"))
        .raw("on_detent_change", "move |i: usize| detent.set(i)")
        .raw("Filters", "{}")
        .done()
        .replace("Filters: {},", "Filters {}")
}

fn card(v: &Vals, _: ClockMs) -> Element {
    let row = v.s("layout") == "row";
    rsx! {
        div { class: if row { "oxd-cards rows" } else { "oxd-cards" },
            Card {
                title: "Refresh-token race",
                description: Some("Two tabs refreshed at once and logged the user out. The agent added a single-flight lock.".to_string()),
                media: v.opt("media"),
                meta: Some("Maya Chen".to_string()),
                status: Some(v.string("status")),
                action: v.b("action").then(|| "Review".to_string()),
                details: v.b("quick_look").then(|| "Fix: wrap refresh() in a mutex keyed by account; retry the waiting request with the new token. 3 files changed, 2 tests added, all 148 tests pass.".to_string()),
                open: v.b("open"),
                edge: v.string("edge"),
                selectable: v.b("selectable"),
                layout: v.string("layout"),
                width: 290.0,
            }
            if !row {
                Card {
                    title: "Newsletter send",
                    description: Some("Waiting for your approval to email 12,480 subscribers.".to_string()),
                    media: Some("dots".to_string()),
                    meta: Some("Outreach agent".to_string()),
                    status: Some("Needs approval".to_string()),
                    action: v.b("action").then(|| "Open".to_string()),
                    edge: "attention",
                    selectable: v.b("selectable"),
                    width: 290.0,
                }
            }
        }
    }
}

fn card_code(v: &Vals) -> String {
    Code::new("Card")
        .raw("title", q("Refresh-token race"))
        .raw("description", q("Two tabs refreshed at once…"))
        .opt("media", v.s("media"))
        .raw("meta", q("Maya Chen"))
        .raw("status", q(v.s("status")))
        .opt("action", if v.b("action") { "Review" } else { "" })
        .opt(
            "details",
            if v.b("quick_look") {
                "Fix: wrap refresh() in a mutex…"
            } else {
                ""
            },
        )
        .s("edge", v.s("edge"), "none")
        .b("selectable", v.b("selectable"))
        .s("layout", v.s("layout"), "card")
        .done()
}

fn transcript() -> Vec<Line> {
    let l = |w: &str, t: &str| Line {
        who: w.into(),
        text: t.into(),
    };
    vec![
        l(
            "You",
            "Users get logged out when two tabs refresh at once. Can you find out why?",
        ),
        l("Atlas", "Reading auth/session.ts and the refresh handler."),
        l(
            "Atlas",
            "Both tabs call refresh() with the same refresh token. The second call gets 401 because the first one rotated it.",
        ),
        l(
            "Atlas",
            "Plan: a single-flight lock around refresh(), keyed by account, and a retry for the request that waited.",
        ),
        l("You", "Sounds right. Go ahead."),
        l(
            "Atlas",
            "Edited auth/session.ts: added refreshOnce() with a per-account promise.",
        ),
        l(
            "Atlas",
            "Edited api/client.ts: retries once with the new token after a 401.",
        ),
        l(
            "Atlas",
            "Added two tests: concurrent refresh, and refresh after expiry.",
        ),
        l("Atlas", "Running the test suite… 148 passed."),
        l(
            "Atlas",
            "Opened a pull request: fix(auth): single-flight token refresh.",
        ),
        l("You", "Thanks. Can you also add a note to the changelog?"),
        l(
            "Atlas",
            "Added under Fixed: two tabs no longer log you out on refresh.",
        ),
    ]
}

fn scroll_area(v: &Vals, _: ClockMs) -> Element {
    let kind = v.s("content");
    let orient = if kind == "strip" {
        "horizontal".to_string()
    } else {
        v.string("orientation")
    };
    rsx! {
        ScrollArea {
            orientation: orient,
            fade: v.f("fade") as f32,
            scrollbars: v.string("scrollbars"),
            hide_delay: v.u("hide_delay") as u32,
            height: if kind == "strip" { 140.0 } else { 260.0 },
            width: 400.0,
            label: Some(match kind { "transcript" => "Transcript", "strip" => "Recent runs", _ => "Files" }.to_string()),
            stick_to_bottom: kind == "transcript" && v.b("stick"),
            stream: kind == "transcript" && v.b("stream"),
            lines: if kind == "transcript" { transcript() } else { vec![] },
            snap: (kind == "strip").then(|| "x mandatory".to_string()),
            if kind == "strip" {
                div { class: "strip",
                    for (i, r) in ["Run 248 · fix auth", "Run 247 · docs", "Run 246 · deps", "Run 245 · tests", "Run 244 · perf", "Run 243 · lint"].iter().enumerate() {
                        div { key: "{i}", class: "tile", "{r}" }
                    }
                }
            }
            if kind == "list" {
                div { class: "list",
                    for (i, (f, n)) in [("auth/session.ts", "+42 −8"), ("api/client.ts", "+17 −3"), ("auth/session.test.ts", "+64"), ("CHANGELOG.md", "+2"), ("api/retry.ts", "+11 −11"), ("auth/token.ts", "+3 −1"), ("README.md", "+1"), ("package.json", "+1 −1"), ("api/types.ts", "+6"), ("auth/index.ts", "+2 −2"), ("e2e/login.spec.ts", "+28"), ("tsconfig.json", "+1")].iter().enumerate() {
                        div { key: "{i}", class: "li", span { "{f}" } span { "{n}" } }
                    }
                }
            }
        }
    }
}

fn scroll_area_code(v: &Vals) -> String {
    let c = Code::new("ScrollArea")
        .s("orientation", v.s("orientation"), "vertical")
        .raw("fade", format!("{:.0}.0", v.f("fade")))
        .s("scrollbars", v.s("scrollbars"), "auto")
        .raw("height", "260.0");
    let c = if v.s("content") == "transcript" {
        c.b("stick_to_bottom", v.b("stick"))
            .raw("lines", "transcript()")
            .b("stream", v.b("stream"))
    } else {
        c.raw("FileList", "{}")
    };
    c.done().replace("FileList: {},", "FileList {}")
}

fn modes() -> Vec<FocusMode> {
    vec![
        FocusMode::new(
            "deep",
            "Deep work",
            "brain",
            "Silences everything but mentions",
            50,
        ),
        FocusMode::new("review", "Review", "eye", "Only approvals and failures", 25),
        FocusMode::new("rest", "Rest", "moon", "Silences all alerts", 30),
    ]
}

fn channels() -> Vec<Channel> {
    vec![
        Channel::new("mentions", "Mentions", true),
        Channel::new("approvals", "Approval requests", true),
        Channel::new("runs", "Finished runs", false),
    ]
}

fn control_center(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        ControlCenter {
            modes: modes(),
            channels: channels(),
            people: if v.b("people") { vec!["Maya".to_string(), "Sam".to_string()] } else { vec![] },
            default_volume: v.f("volume") as f32,
            default_text_size: v.f("text_size") as f32,
            agent_tiles: v.b("agent"),
            open_detail: v.opt("detail"),
        }
    }
}

fn control_center_code(v: &Vals) -> String {
    Code::new("ControlCenter")
        .pre("let modes = vec![FocusMode::new(\"deep\", \"Deep work\", \"brain\", \"Silences everything but mentions\", 50)];")
        .pre("let channels = vec![Channel::new(\"mentions\", \"Mentions\", true), Channel::new(\"runs\", \"Finished runs\", false)];")
        .raw("modes", "modes")
        .raw("channels", "channels")
        .raw("default_volume", format!("{:.0}.0", v.f("volume")))
        .raw("default_text_size", format!("{:.0}.0", v.f("text_size")))
        .b("agent_tiles", v.b("agent"))
        .raw("on_state_change", "move |(focus, notifications)| save(focus, notifications)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "dialog",
            name: "Dialog",
            category: "surfaces",
            summary: "A modal window on a dimmed backdrop: title and quiet line, a body that scrolls under a fixed header and footer, Escape and backdrop close, focus kept inside and handed back.",
            dioxus: "Dialog",
            gpui: "DialogEl",
            spec: "dialog",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("open", "Open", true),
                sel("size", "Size", &["sm", "md", "lg", "full"], "md"),
                sel("tone", "Tone", &["default", "danger"], "default"),
                tog("footer", "Footer", true),
                tog("close_button", "Close button", true),
                tog("close_on_backdrop", "Backdrop closes", true),
                tog("close_on_escape", "Escape closes", true),
                tog("modal", "Modal", true),
                rng("busy_ms", "Confirm runs ms", 0.0, 3000.0, 100.0, "1200"),
                txt("title", "Title", ""),
            ],
            variants: vec![
                var("form", "Settings with a Save footer.", &[]),
                var(
                    "confirm-danger",
                    "An alert dialog with a red confirm that runs before closing.",
                    &[("tone", "danger"), ("size", "sm")],
                ),
                var(
                    "sticky",
                    "Backdrop and Escape don't close it.",
                    &[("close_on_backdrop", "false"), ("close_on_escape", "false")],
                ),
                var(
                    "floating",
                    "Non-modal: the page stays live behind it.",
                    &[("modal", "false")],
                ),
                var("closed", "Just the trigger.", &[("open", "false")]),
            ],
            render: dialog,
            code: dialog_code,
            keyboard: vec![
                ("Esc", "Close and return focus to the trigger"),
                ("Tab / Shift+Tab", "Cycle focus inside"),
                ("Enter", "Run the focused button"),
            ],
            a11y: vec![
                "role=dialog (alertdialog when danger) with aria-modal, named by the title and described by the quiet line.",
                "Focus moves into the dialog and wraps at both ends; closing hands it back to the trigger.",
                "While a confirm runs the dialog is aria-busy and can't be dismissed.",
            ],
            responsive: vec![
                "min(viewport − 32 px, size) wide and capped at the viewport height; the body scrolls.",
            ],
            height: 460,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "sheet",
            name: "Sheet",
            category: "surfaces",
            summary: "The dialog docked to an edge for inspectors and run details: slides in from its side, a drag handle to fling it away, an inset floating style and a pinned footer.",
            dioxus: "Sheet",
            gpui: "SheetEl",
            spec: "sheet",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("open", "Open", true),
                sel("side", "Side", &["right", "left", "bottom", "top"], "right"),
                rng("size", "Size px", 240.0, 480.0, 20.0, "320"),
                tog("inset", "Inset", false),
                tog("draggable", "Drag handle", true),
                tog("footer", "Footer", true),
            ],
            variants: vec![
                var("right", "An inspector on the right.", &[]),
                var(
                    "inset-left",
                    "Floating on the left.",
                    &[("side", "left"), ("inset", "true")],
                ),
                var(
                    "bottom",
                    "A tray from the bottom.",
                    &[("side", "bottom"), ("size", "240")],
                ),
            ],
            render: sheet,
            code: sheet_code,
            keyboard: vec![("Esc", "Close"), ("Tab", "Move inside the sheet")],
            a11y: vec![
                "role=dialog with aria-modal, named by its title; the scrim and × close it and hand focus back.",
            ],
            responsive: vec![
                "size is the width (left / right) or height (top / bottom), never wider than the viewport.",
            ],
            height: 460,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "sheet-stack",
            name: "Sheet stack",
            category: "surfaces",
            summary: "Nested sheets with depth: a new sheet rises while the one below scales back 5 %, dims and peeks; back names the sheet below; drag or fling the top one away; stacked dialogs on wide screens.",
            dioxus: "SheetStack, StackPage, StackRow",
            gpui: "SheetStackEl",
            spec: "sheet-stack",
            source: "primitives/surfaces.rs",
            controls: vec![
                rng("depth", "Open sheets", 0.0, 3.0, 1.0, "2"),
                sel(
                    "mode",
                    "Mode",
                    &["auto", "sheet", "dialog", "side"],
                    "sheet",
                ),
                tog("breadcrumb", "Breadcrumb", false),
            ],
            variants: vec![
                var("sheets", "Two sheets, the lower one peeking.", &[]),
                var(
                    "deep",
                    "Three levels with a breadcrumb.",
                    &[("depth", "3"), ("breadcrumb", "true")],
                ),
                var(
                    "dialogs",
                    "Stacked dialogs (wide screens).",
                    &[("mode", "dialog")],
                ),
                var("side", "Inspectors stacking sideways.", &[("mode", "side")]),
            ],
            render: sheet_stack,
            code: sheet_stack_code,
            keyboard: vec![
                ("Esc", "Close the top sheet only"),
                ("Tab / Shift+Tab", "Cycle inside the top sheet"),
                ("Enter / Space", "Back, close and rows"),
            ],
            a11y: vec![
                "Each sheet is a dialog named by its title; only the top one is aria-modal, the rest are inert.",
                "The back button names the sheet it returns to; the close button reads Close all when more than one is open.",
            ],
            responsive: vec![
                "auto: bottom sheets under 640 px, centred dialogs at or above; each level loses room for the peek.",
            ],
            height: 460,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "bottom-sheet",
            name: "Bottom sheet",
            category: "surfaces",
            summary: "A sheet that rests at a peek or full height and follows your finger: rubber-bands past the top, projects a flick to the nearest detent, and the backdrop dims with its height.",
            dioxus: "BottomSheet",
            gpui: "BottomSheetEl",
            spec: "bottom-sheet",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("open", "Open", true),
                sel("detents", "Detents", &["two", "three", "single"], "two"),
                rng("initial", "Opens at", 0.0, 2.0, 1.0, "0"),
            ],
            variants: vec![
                var("peek", "Rests at 45 %.", &[]),
                var("full", "Opens at full height.", &[("initial", "1")]),
                var(
                    "three-detents",
                    "30 / 60 / 92 %.",
                    &[("detents", "three"), ("initial", "1")],
                ),
            ],
            render: bottom_sheet,
            code: bottom_sheet_code,
            keyboard: vec![
                ("↑ / ↓", "On the grabber: one detent up / down"),
                ("Home / End", "Tallest / smallest"),
                ("Enter / Space", "Toggle smallest and tallest"),
                ("↓ / Page Down", "On the sheet: expand first"),
                ("Esc", "Close"),
            ],
            a11y: vec![
                "role=dialog with aria-modal; the grabber is a button with aria-expanded and an Expand / Collapse label.",
                "Detent changes are announced politely (\"Sheet at 45 percent\").",
            ],
            responsive: vec![
                "min(100 %, 36 rem) wide; detents are fractions of the container or viewport height.",
            ],
            height: 480,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "card",
            name: "Card",
            category: "surfaces",
            summary: "A content card with media, byline, status and an action that opens into a quick look; hover lifts it and zooms the media; status changes roll in word by word.",
            dioxus: "Card",
            gpui: "CardEl",
            spec: "card",
            source: "primitives/surfaces.rs",
            controls: vec![
                sel("media", "Media", &["grid", "dots", "waves", "none"], "grid"),
                txt("status", "Status", "Updated 2 minutes ago"),
                tog("action", "Action", true),
                tog("quick_look", "Quick look", true),
                tog("open", "Quick look open", false),
                sel(
                    "edge",
                    "Status edge",
                    &["none", "working", "attention", "done", "error"],
                    "working",
                ),
                tog("selectable", "Selectable", false),
                sel("layout", "Layout", &["card", "row"], "card"),
            ],
            variants: vec![
                var(
                    "working",
                    "An agent is working on it: a beam runs round the border.",
                    &[],
                ),
                var(
                    "quick-look",
                    "Opened into the quick look.",
                    &[("open", "true"), ("edge", "none")],
                ),
                var(
                    "selectable",
                    "Pick cards for a batch action.",
                    &[("selectable", "true"), ("edge", "none")],
                ),
                var(
                    "row",
                    "Compact list row.",
                    &[("layout", "row"), ("edge", "done")],
                ),
            ],
            render: card,
            code: card_code,
            keyboard: vec![
                ("Enter / Space", "On the title: open the quick look"),
                ("Esc", "Close it; focus returns to the title"),
                ("Tab", "Trapped inside the open quick look"),
            ],
            a11y: vec![
                "An article named by its h3 title; only the title is a button, so nested actions stay separate.",
                "Status changes are read through a polite status region.",
            ],
            responsive: vec![
                "A definite width that never exceeds its cell; hover lift and zoom only for a fine pointer.",
            ],
            height: 380,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "scroll-area",
            name: "Scroll area",
            category: "surfaces",
            summary: "Native scrolling with thin overlay scrollbars and edge fades that grow with what's left; Oxyn's stick-to-bottom follows a streaming transcript and offers Jump to latest when you scroll up.",
            dioxus: "ScrollArea, Line",
            gpui: "ScrollAreaEl",
            spec: "scroll-area",
            source: "primitives/surfaces.rs",
            controls: vec![
                sel(
                    "content",
                    "Content",
                    &["transcript", "list", "strip"],
                    "transcript",
                ),
                tog("stick", "Stick to bottom", true),
                tog("stream", "Stream new lines", true),
                sel(
                    "orientation",
                    "Orientation",
                    &["vertical", "both"],
                    "vertical",
                ),
                rng("fade", "Edge fade px", 0.0, 48.0, 4.0, "28"),
                sel(
                    "scrollbars",
                    "Scrollbars",
                    &["auto", "always", "hidden"],
                    "auto",
                ),
                rng("hide_delay", "Hide after ms", 300.0, 2000.0, 100.0, "900"),
            ],
            variants: vec![
                var("transcript", "Follows new lines; scroll up to let go.", &[]),
                var(
                    "list",
                    "A file list with edge fades.",
                    &[("content", "list"), ("scrollbars", "always")],
                ),
                var(
                    "strip",
                    "A horizontal strip that snaps.",
                    &[("content", "strip")],
                ),
            ],
            render: scroll_area,
            code: scroll_area_code,
            keyboard: vec![
                ("Tab", "Focus the viewport"),
                (
                    "Arrows, Page Up / Down, Home / End, Space",
                    "Scroll natively",
                ),
            ],
            a11y: vec![
                "Scrolling stays native; with a label the viewport is a named region.",
                "The overlay bars are aria-hidden; the Jump to latest pill is a real button.",
            ],
            responsive: vec![
                "Touch keeps native momentum; thumbs thicken only under a fine pointer.",
            ],
            height: 320,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "control-center",
            name: "Control center",
            category: "surfaces",
            summary: "Workspace quick settings: tiles that open into detail, a session dial that counts past twelve and snaps to 5 minutes, and meters that follow the pointer with rubber-banded ends.",
            dioxus: "ControlCenter",
            gpui: "ControlCenterEl",
            spec: "control-center",
            source: "primitives/command.rs",
            controls: vec![
                sel(
                    "detail",
                    "Open detail",
                    &["none", "focus", "notifications", "agent"],
                    "none",
                ),
                rng("volume", "Volume", 0.0, 100.0, 1.0, "64"),
                rng("text_size", "Text size", 85.0, 130.0, 5.0, "100"),
                tog("people", "People who can reach you", true),
                tog("agent", "Agent tiles", true),
            ],
            variants: vec![
                var("tiles", "Tiles and meters.", &[]),
                var(
                    "focus",
                    "The focus detail with the session dial.",
                    &[("detail", "focus")],
                ),
                var(
                    "channels",
                    "Notification channels.",
                    &[("detail", "notifications")],
                ),
                var("agents", "Agent quick settings.", &[("detail", "agent")]),
            ],
            render: control_center,
            code: control_center_code,
            keyboard: vec![
                (
                    "Enter / Space",
                    "Toggle a tile or open its detail from the chevron",
                ),
                ("Arrows", "Meters ±1 step; the dial ±5 minutes"),
                ("Page Up / Down", "Meters ±4 steps; the dial ±15 minutes"),
                ("Home / End", "Limits"),
                ("Esc", "Close the detail"),
            ],
            a11y: vec![
                "Tiles are toggle buttons with aria-pressed; chevrons open a modal detail and the grid behind is inert.",
                "Meters and the dial are sliders with formatted aria-valuetext; channels are switches.",
            ],
            responsive: vec![
                "min(100 %, 22.5 rem) wide with a fixed height; tighter padding and a smaller dial under 360 px.",
            ],
            height: 380,
            align: Align::Center,
            live: false,
        },
    ]
}
