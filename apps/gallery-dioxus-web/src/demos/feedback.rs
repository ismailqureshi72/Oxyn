//! Feedback (plan #26): toaster, notification stack, alert, skeleton (+ shimmer loader),
//! skeleton morph.

use dioxus::prelude::*;
use oxyn_core::{ClockMs, primitives::toast::Notice};
use oxyn_dioxus::{
    Alert, MorphBlock, NotificationStack, SeedToast, ShimmerLabel, Skeleton, SkeletonMorph,
    Toaster, use_toasts,
};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

#[component]
fn ToastButtons(promise_ms: u32) -> Element {
    let toasts = use_toasts();
    let mut pending = use_signal(|| None::<u64>);
    let mut n = use_signal(|| 0u32);
    rsx! {
        div { class: "oxd-btnrow",
            button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| { toasts.success("Draft saved", Some("Q3 board report · all changes kept")); }, "Save" }
            button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| { toasts.show("File moved to trash", Some("auth/legacy.ts"), oxyn_core::primitives::toast::ToastKind::Default, Some("Undo")); }, "Delete" }
            button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| { let id = toasts.promise("Uploading report…", Some("q3-board.pdf · 2.4 MB")); pending.set(Some(id)); n += 1; }, "Upload" }
            button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| { toasts.error("Sync failed", Some("GitHub returned 502. We'll retry in a minute.")); }, "Fail" }
            if let Some(id) = pending() {
                span { key: "{n}", class: "oxyn-timer", style: "animation-duration:{promise_ms}ms", onanimationend: move |_| { toasts.resolve(id, true, "Report uploaded", Some("Shared with 4 people")); pending.set(None) } }
            }
        }
    }
}

fn toaster(v: &Vals, _: ClockMs) -> Element {
    let seed = if v.b("seed") {
        vec![
            SeedToast::new(
                "Run 247 finished",
                "12 files changed · 148 tests passed",
                "success",
            ),
            SeedToast::new("Approval needed", "Atlas wants to push to main", "warning")
                .action("Review"),
            SeedToast::new("Indexing repository…", "atlas · 3,412 files", "loading"),
        ]
    } else {
        vec![]
    };
    rsx! {
        div { class: "oxd-app oxd-app-toast",
            div { class: "oxd-app-bar", span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-dot" } span { class: "oxd-app-title", "Atlas" } }
            Toaster {
                position: v.string("position"),
                inline: true,
                duration: v.u("duration") as u32,
                visible: v.u("visible"),
                expand: v.b("expand"),
                hairline: v.b("hairline"),
                close_button: v.b("close_button"),
                seed,
                div { class: "oxd-app-body", ToastButtons { promise_ms: 1800 } p { class: "oxd-note", "Hover the stack to fan it out · F8 focuses the newest" } }
            }
        }
    }
}

fn toaster_code(v: &Vals) -> String {
    let mut c = Code::new("Toaster")
        .s("position", v.s("position"), "bottom-right")
        .raw("duration", v.s("duration"))
        .raw("visible", v.s("visible"))
        .b("expand", v.b("expand"));
    if !v.b("hairline") {
        c = c.raw("hairline", "false");
    }
    let mut s = c.raw("App", "{}").done().replace("App: {},", "App {}");
    s.push_str("\n\n// Anywhere inside App:\nlet toasts = use_toasts();\ntoasts.success(\"Draft saved\", Some(\"Q3 board report\"));\nlet id = toasts.promise(\"Uploading report…\", None);\n// later\ntoasts.resolve(id, true, \"Report uploaded\", Some(\"Shared with 4 people\"));");
    s
}

fn notices() -> Vec<Notice> {
    vec![
        Notice::new(
            "maint",
            "Scheduled maintenance",
            "Agents pause on Sunday from 02:00 to 03:00 UTC while we upgrade the sandbox fleet.",
        )
        .badge("Notice")
        .action("View status", true)
        .pattern("grid")
        .priority(2),
        Notice::new(
            "invite",
            "Sam invited you to Atlas",
            "You'll see their agent runs and can approve on their behalf.",
        )
        .badge("Invite")
        .action("Accept", false)
        .pattern("dots")
        .priority(5),
        Notice::new(
            "release",
            "Agents can open pull requests",
            "Turn it on per project under Permissions.",
        )
        .badge("New")
        .action("Learn more", true)
        .pattern("lines")
        .priority(1),
    ]
}

fn notification_stack(v: &Vals, _: ClockMs) -> Element {
    let mut ns = notices();
    if !v.b("patterns") {
        for n in &mut ns {
            n.pattern = None;
        }
    }
    rsx! {
        NotificationStack {
            notices: ns,
            depth: v.u("depth"),
            by_priority: v.b("by_priority"),
            undo: v.b("undo"),
        }
    }
}

fn notification_stack_code(v: &Vals) -> String {
    Code::new("NotificationStack")
        .pre("let notices = vec![")
        .pre("    Notice::new(\"maint\", \"Scheduled maintenance\", \"Agents pause on Sunday…\").badge(\"Notice\").action(\"View status\", true),")
        .pre("    Notice::new(\"invite\", \"Sam invited you to Atlas\", \"…\").badge(\"Invite\").priority(5),")
        .pre("];")
        .raw("notices", "notices")
        .raw("depth", v.s("depth"))
        .b("by_priority", v.b("by_priority"))
        .raw("on_dismiss", "move |id: String| mark_read(id)")
        .done()
}

fn alert(v: &Vals, _: ClockMs) -> Element {
    let tone = v.string("tone");
    let (title, body) = match tone.as_str() {
        "success" => (
            "Deployed to production",
            "Run 248 shipped in 3 min 12 s. Health checks are green.",
        ),
        "warning" => (
            "Context is 86 % full",
            "The agent will compact older turns soon. Pin anything it must keep.",
        ),
        "danger" => (
            "Deploy failed",
            "Build exited with code 1 in step 3 of 5. Nothing was released.",
        ),
        "agent" => (
            "Atlas is working in the background",
            "It's refactoring auth/session.ts and will ask before pushing.",
        ),
        _ => (
            "Read-only mode",
            "This workspace is archived. Restore it to run agents again.",
        ),
    };
    let actions: Vec<String> = if v.b("actions") {
        if tone == "danger" {
            vec!["Retry".into(), "Show logs".into()]
        } else {
            vec!["View".into()]
        }
    } else {
        vec![]
    };
    rsx! {
        div { class: "oxd-w480",
            Alert {
                title: if v.s("title").is_empty() { title.to_string() } else { v.string("title") },
                tone,
                body: Some(body.to_string()),
                open: v.b("open"),
                dismissible: v.b("dismissible"),
                actions,
                details: (v.b("details")).then(|| "error[E0308]: mismatched types\n  --> src/auth/session.rs:42:18\n   |\n42 |     let token: Token = refresh().await;\n   |                ^^^^^ expected `Token`, found `Result<Token, Error>`".to_string()),
                variant: v.string("variant"),
            }
            p { class: "oxd-note", "Content below closes the gap when the alert is dismissed." }
        }
    }
}

fn alert_code(v: &Vals) -> String {
    Code::new("Alert")
        .raw("title", q("Deploy failed"))
        .s("tone", v.s("tone"), "info")
        .raw("body", q("Build exited with code 1…"))
        .b("dismissible", v.b("dismissible"))
        .raw(
            "actions",
            if v.b("actions") {
                "vec![\"Retry\".into(), \"Show logs\".into()]"
            } else {
                "vec![]"
            },
        )
        .opt(
            "details",
            if v.b("details") {
                "error[E0308]: mismatched types…"
            } else {
                ""
            },
        )
        .s("variant", v.s("variant"), "soft")
        .raw("on_action", "move |a: String| run(a)")
        .done()
}

fn skeleton(v: &Vals, _: ClockMs) -> Element {
    let loading = v.b("loading");
    rsx! {
        div { class: "oxd-w360", style: "display:flex;flex-direction:column;gap:18px",
            Skeleton {
                lines: v.u("lines"),
                avatar: v.b("avatar"),
                shape: v.string("shape"),
                sweep: v.string("sweep"),
                speed: v.f("speed") as f32,
                angle: v.f("angle") as f32,
                loading,
                delay: v.u("delay") as u32,
                width: 360.0,
                div { class: "oxd-loaded",
                    div { class: "oxd-loaded-h", span { class: "oxd-av", "MC" } b { "Maya Chen" } span { class: "oxd-muted", "· 2 min ago" } }
                    p { "Both tabs call refresh() with the same token; the second one gets 401. A single-flight lock fixes it." }
                }
            }
            if v.b("shimmer_text") && loading {
                ShimmerLabel { text: "Thinking… reading 14 files", duration: v.f("speed") as f32 + 0.4 }
            }
        }
    }
}

fn skeleton_code(v: &Vals) -> String {
    Code::new("Skeleton")
        .raw("lines", v.s("lines"))
        .b("avatar", v.b("avatar"))
        .s("shape", v.s("shape"), "text")
        .s("sweep", v.s("sweep"), "shimmer")
        .raw("loading", "loading()")
        .raw("Message", "{ message }")
        .done()
        .replace("Message: { message },", "Message { message }")
}

fn skeleton_morph(v: &Vals, _: ClockMs) -> Element {
    let loading = v.b("loading");
    let s = v.u("stagger") as u32;
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:12px;align-items:center",
            SkeletonMorph { loading, stagger: s, width: 360.0,
                div { class: "mrow",
                    MorphBlock { index: 0, width: "36px", height: 36.0, radius: "circle", span { class: "oxd-av lg", "AT" } }
                    div { style: "flex:1;display:flex;flex-direction:column;gap:4px;min-width:0",
                        MorphBlock { index: 1, width: "45%", height: 14.0, b { "Atlas · run 248" } }
                        MorphBlock { index: 2, width: "30%", height: 11.0, span { class: "oxd-muted", style: "font-size:12px", "refactor/auth-refresh · 4 min" } }
                    }
                }
                MorphBlock { index: 3, lines: Some(v.u("lines")), line_height: 20.0,
                    p { style: "margin:0;line-height:20px", "Added a single-flight lock around refresh() so two tabs share one request, and a retry for the call that waited. 148 tests pass." }
                }
                if v.b("chart") {
                    MorphBlock { index: 4, height: 72.0, radius: "10",
                        div { class: "oxd-bars", for (i, h) in [30, 52, 41, 66, 48, 72, 58, 80].iter().enumerate() { span { key: "{i}", style: "height:{h}%" } } }
                    }
                }
                div { class: "mrow",
                    MorphBlock { index: 5, width: "88px", height: 28.0, radius: "8", button { class: "oxyn-btn primary sm", r#type: "button", "Review" } }
                    MorphBlock { index: 6, width: "72px", height: 28.0, radius: "8", button { class: "oxyn-btn ghost sm", r#type: "button", "Dismiss" } }
                }
            }
            p { class: "oxd-note", "Toggle Loading to watch blocks resolve one after another." }
        }
    }
}

fn skeleton_morph_code(v: &Vals) -> String {
    let mut s = String::from("rsx! {\n    SkeletonMorph {\n        loading: loading(),\n");
    if v.s("stagger") != "40" {
        s.push_str(&format!("        stagger: {},\n", v.s("stagger")));
    }
    s.push_str("        MorphBlock { index: 0, width: \"36px\", height: 36.0, radius: \"circle\", Avatar { … } }\n");
    s.push_str("        MorphBlock { index: 1, width: \"45%\", height: 14.0, b { \"Atlas · run 248\" } }\n");
    s.push_str(&format!(
        "        MorphBlock {{ index: 2, lines: Some({}), p {{ \"{{summary}}\" }} }}\n",
        v.s("lines")
    ));
    s.push_str("    }\n}");
    s
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "toaster",
            name: "Toaster",
            category: "feedback",
            summary: "Calm toasts from anywhere: a stack that fans out on hover, promise toasts that morph from spinner to check, a timer hairline that pauses while you read, swipe or Escape to dismiss.",
            dioxus: "Toaster, use_toasts, SeedToast",
            gpui: "ToasterEl",
            spec: "toaster",
            source: "primitives/surfaces.rs",
            controls: vec![
                sel(
                    "position",
                    "Position",
                    &[
                        "bottom-right",
                        "bottom-center",
                        "bottom-left",
                        "top-right",
                        "top-center",
                        "top-left",
                    ],
                    "bottom-right",
                ),
                rng(
                    "duration",
                    "Duration ms (0 = stays)",
                    0.0,
                    10000.0,
                    500.0,
                    "6000",
                ),
                rng("visible", "Visible", 1.0, 5.0, 1.0, "3"),
                tog("expand", "Always fanned out", false),
                tog("hairline", "Timer hairline", true),
                tog("close_button", "Close button", true),
                tog("seed", "Start with toasts", true),
            ],
            variants: vec![
                var("stack", "Three toasts, collapsed.", &[]),
                var("expanded", "Fanned out into a list.", &[("expand", "true")]),
                var(
                    "top-center",
                    "From the top, staying until dismissed.",
                    &[("position", "top-center"), ("duration", "0")],
                ),
                var(
                    "empty",
                    "Press the buttons to send toasts.",
                    &[("seed", "false")],
                ),
            ],
            render: toaster,
            code: toaster_code,
            keyboard: vec![
                ("F8", "Focus the newest toast and fan the stack out"),
                ("Tab", "Toast to toast, through action and close"),
                ("Esc", "Close the focused toast and move to the next"),
            ],
            a11y: vec![
                "The stack is a labelled region (\"Notifications (F8)\"); every toast is focusable.",
                "New and updated toasts are announced politely, errors assertively.",
                "Timers pause while the pointer or focus is in the stack; swiping is never the only way out.",
            ],
            responsive: vec!["356 px wide; under 600 px it spans the width with 12 px gutters."],
            height: 440,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "notification-stack",
            name: "Notification stack",
            category: "feedback",
            summary: "Notice cards stacked behind each other: dismiss the front one to reveal the next. Oxyn adds stack depth, priority ordering, an Undo and background patterns.",
            dioxus: "NotificationStack",
            gpui: "NotificationStackEl",
            spec: "notification-stack",
            source: "primitives/surfaces.rs",
            controls: vec![
                rng("depth", "Cards drawn", 1.0, 3.0, 1.0, "3"),
                tog("by_priority", "Order by priority", false),
                tog("undo", "Undo", true),
                tog("patterns", "Patterns", true),
            ],
            variants: vec![
                var("stack", "Three notices.", &[]),
                var(
                    "priority",
                    "The invite jumps to the front.",
                    &[("by_priority", "true")],
                ),
                var(
                    "flat",
                    "One card, no patterns.",
                    &[("depth", "1"), ("patterns", "false")],
                ),
            ],
            render: notification_stack,
            code: notification_stack_code,
            keyboard: vec![
                ("Tab", "The front card's buttons"),
                ("Enter / Space", "Run the action or dismiss"),
            ],
            a11y: vec![
                "A labelled region; each card is an article named by its title.",
                "Only the front card is read out and focusable; the next takes its place after a dismiss.",
            ],
            responsive: vec!["340 px wide, never wider than its container."],
            height: 260,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "alert",
            name: "Alert",
            category: "feedback",
            summary: "A persistent message that helps people recover or continue: tone icon, title and details; dismissing collapses its height; tone and copy changes morph in place.",
            dioxus: "Alert",
            gpui: "AlertEl",
            spec: "alert",
            source: "primitives/surfaces.rs",
            controls: vec![
                sel(
                    "tone",
                    "Tone",
                    &["info", "success", "warning", "danger", "agent"],
                    "danger",
                ),
                sel("variant", "Style", &["soft", "outline", "solid"], "soft"),
                tog("open", "Open", true),
                tog("dismissible", "Dismissible", true),
                tog("actions", "Actions", true),
                tog("details", "Details", true),
                txt("title", "Title", ""),
            ],
            variants: vec![
                var("agent-error", "Retry, logs and details.", &[]),
                var(
                    "success",
                    "A quiet confirmation.",
                    &[
                        ("tone", "success"),
                        ("actions", "false"),
                        ("details", "false"),
                    ],
                ),
                var(
                    "warning",
                    "Context almost full.",
                    &[("tone", "warning"), ("details", "false")],
                ),
                var(
                    "agent",
                    "The agent's identity colour.",
                    &[
                        ("tone", "agent"),
                        ("variant", "solid"),
                        ("details", "false"),
                        ("actions", "false"),
                    ],
                ),
                var(
                    "outline",
                    "Monochrome outline.",
                    &[
                        ("tone", "info"),
                        ("variant", "outline"),
                        ("details", "false"),
                    ],
                ),
            ],
            render: alert,
            code: alert_code,
            keyboard: vec![("Enter / Space", "Dismiss, actions and Show logs")],
            a11y: vec![
                "Danger is role=alert; the rest role=status.",
                "The dismiss button is named \"Dismiss: <title>\"; the icon is decorative.",
            ],
            responsive: vec![
                "Fills its container; the copy wraps, the icon and button keep their size.",
            ],
            height: 300,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "skeleton",
            name: "Skeleton",
            category: "feedback",
            summary: "Grey shapes the size of what they stand in for, one sheen sweeping across all of them in step (or a staggered pulse), shown only after a short delay, then a crossfade into the content.",
            dioxus: "Skeleton, ShimmerLabel",
            gpui: "SkeletonEl",
            spec: "skeletons",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("loading", "Loading", true),
                sel(
                    "shape",
                    "Shape",
                    &[
                        "text", "message", "tool-row", "diff", "card", "table", "form",
                    ],
                    "message",
                ),
                rng("lines", "Lines", 1.0, 6.0, 1.0, "3"),
                tog("avatar", "Avatar", true),
                sel(
                    "sweep",
                    "Sweep",
                    &["shimmer", "pulse", "wave", "none"],
                    "shimmer",
                ),
                rng("speed", "Period s", 0.8, 3.0, 0.2, "1.6"),
                rng("angle", "Sweep angle", 60.0, 140.0, 10.0, "100"),
                rng("delay", "Reveal after ms", 0.0, 600.0, 50.0, "150"),
                tog("shimmer_text", "Shimmer text", true),
            ],
            variants: vec![
                var("message", "A chat message.", &[]),
                var(
                    "tool-rows",
                    "Tool calls in a run.",
                    &[
                        ("shape", "tool-row"),
                        ("lines", "4"),
                        ("shimmer_text", "false"),
                    ],
                ),
                var(
                    "wave",
                    "Arc's staggered pulse.",
                    &[("shape", "text"), ("sweep", "wave"), ("lines", "5")],
                ),
                var(
                    "loaded",
                    "Crossfaded into the content.",
                    &[("loading", "false")],
                ),
            ],
            render: skeleton,
            code: skeleton_code,
            keyboard: vec![],
            a11y: vec![
                "role=status with aria-busy and a label; the shapes are aria-hidden.",
                "Reduced motion stops the sheen and pulse.",
            ],
            responsive: vec![
                "Bars are percentages of the slot width, so it scales with its container.",
            ],
            height: 260,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "skeleton-morph",
            name: "Skeleton morph",
            category: "feedback",
            summary: "Loading skeletons that grow into the real content block by block: each placeholder stretches to the content's box while it fades and the content sharpens out of a blur.",
            dioxus: "SkeletonMorph, MorphBlock",
            gpui: "SkeletonMorphEl",
            spec: "skeleton-morph",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("loading", "Loading", false),
                rng("stagger", "Stagger ms", 0.0, 160.0, 10.0, "60"),
                rng("lines", "Text lines", 1.0, 5.0, 1.0, "3"),
                tog("chart", "Chart block", true),
            ],
            variants: vec![
                var("resolve", "Blocks resolving in order.", &[]),
                var("loading", "Size-matched skeletons.", &[("loading", "true")]),
                var(
                    "fast",
                    "Arc's 40 ms stagger, no chart.",
                    &[("stagger", "40"), ("chart", "false")],
                ),
            ],
            render: skeleton_morph,
            code: skeleton_morph_code,
            keyboard: vec![],
            a11y: vec![
                "aria-busy with a hidden status while loading; content isn't rendered until ready, so nothing half-loaded is read.",
                "Blocks keep reading order.",
            ],
            responsive: vec![
                "Blocks size to their content once loaded; skeleton widths take percentages.",
            ],
            height: 340,
            align: Align::Center,
            live: false,
        },
    ]
}
