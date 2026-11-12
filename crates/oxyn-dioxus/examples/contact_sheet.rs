//! Writes a static HTML contact sheet of every `StatusIndicator` state, variant and theme.
//!
//! `cargo run -p oxyn-dioxus --example contact_sheet > sheet.html`, then open it next to
//! `tokens/dist/oxyn.css` (or pass the CSS path as the first argument to inline it).

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs, ErrorKind, RequestRef, StallLevel, ToolRef, verbs::ActivityKind,
};
use oxyn_dioxus::{IndicatorSize, IndicatorVariant, STATUS_INDICATOR_CSS, StatusIndicator};

fn states() -> Vec<(&'static str, AgentState)> {
    let tool = ToolRef {
        id: "t1".into(),
        name: "bash".into(),
        activity: ActivityKind::Run,
        target: Some("cargo test".into()),
    };
    vec![
        ("Idle", AgentState::Idle),
        ("Composing", AgentState::Composing),
        ("Submitted", AgentState::Submitted),
        ("Thinking", AgentState::Thinking { header: None }),
        (
            "Thinking + header",
            AgentState::Thinking {
                header: Some("Checking the auth flow".into()),
            },
        ),
        ("Streaming", AgentState::Streaming),
        (
            "Tool running",
            AgentState::ToolRunning {
                tool: tool.clone(),
                concurrent: 1,
            },
        ),
        (
            "Tools ×3",
            AgentState::ToolRunning {
                tool: tool.clone(),
                concurrent: 3,
            },
        ),
        (
            "Awaiting approval",
            AgentState::AwaitingApproval {
                request: RequestRef {
                    id: "r1".into(),
                    tool: None,
                },
            },
        ),
        ("Queued", AgentState::Queued { count: 2 }),
        ("Steering", AgentState::Steering),
        (
            "Stalled (soft)",
            AgentState::Stalled {
                since: ClockMs(0),
                level: StallLevel::Soft,
                was: Box::new(AgentState::Thinking { header: None }),
            },
        ),
        (
            "Stalled (hard)",
            AgentState::Stalled {
                since: ClockMs(0),
                level: StallLevel::Hard,
                was: Box::new(AgentState::ToolRunning {
                    tool,
                    concurrent: 1,
                }),
            },
        ),
        (
            "Done",
            AgentState::Done {
                elapsed: Duration::from_secs(72),
                waited: Duration::ZERO,
            },
        ),
        (
            "Error",
            AgentState::Error {
                kind: ErrorKind::Timeout,
            },
        ),
        (
            "Cancelled",
            AgentState::Cancelled {
                after: Duration::from_secs(42),
            },
        ),
    ]
}

#[component]
fn Sheet() -> Element {
    let now = ClockMs(48_000);
    rsx! {
        div { class: "sheet",
            h1 { "StatusIndicator" span { "oxyn · plan #15 · 16 states × 3 variants × 2 sizes × 3 themes" } }
            for theme in ["dark", "light", "hc-dark"] {
                section { "data-oxyn-theme": theme, class: "theme",
                    h2 { "{theme}" }
                    div { class: "grid",
                        div { class: "hd", "state" }
                        div { class: "hd", "pill" }
                        div { class: "hd", "inline" }
                        div { class: "hd", "sm" }
                        div { class: "hd", "dot" }
                        for (name, s) in states() {
                            div { class: "nm", "{name}" }
                            div { StatusIndicator { state: s.clone(), now } }
                            div { StatusIndicator { state: s.clone(), now, variant: IndicatorVariant::Inline } }
                            div { StatusIndicator { state: s.clone(), now, size: IndicatorSize::Small, pack: "terse" } }
                            div { StatusIndicator { state: s, now, variant: IndicatorVariant::Dot } }
                        }
                    }
                }
            }
        }
    }
}

const PAGE_CSS: &str = r#"
html,body{margin:0;background:#0B0B0C}
.sheet{padding:28px;font-family:Inter,system-ui,sans-serif;display:grid;grid-template-columns:repeat(3,auto);gap:20px;width:max-content}
.sheet h1{grid-column:1/-1;margin:0 0 4px;color:#F2EFEA;font-size:22px;font-weight:600}
.sheet h1 span{margin-left:12px;color:#8A867F;font-size:13px;font-weight:400}
.theme{background:var(--oxyn-color-bg-base);border-radius:14px;padding:16px 18px;border:1px solid var(--oxyn-color-border-subtle)}
.theme h2{margin:0 0 10px;font-size:12px;letter-spacing:.06em;text-transform:uppercase;color:var(--oxyn-color-fg-muted)}
.grid{display:grid;grid-template-columns:auto auto auto auto auto;gap:9px 16px;align-items:center}
.hd{font-size:11px;color:var(--oxyn-color-fg-muted)}
.nm{font-size:12px;color:var(--oxyn-color-fg-muted)}
"#;

fn main() {
    let tokens = std::env::args()
        .nth(1)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let body = dioxus_ssr::render_element(rsx! { Sheet {} });
    println!(
        "<!doctype html><html><head><meta charset=utf-8><style>{tokens}{STATUS_INDICATOR_CSS}{PAGE_CSS}</style></head><body>{body}</body></html>"
    );
}
