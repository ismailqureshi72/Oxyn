//! Renders the composer in its six free layouts and its parts' variants as one static HTML page
//! (server-side render). Screenshot it with Chromium for a contact sheet.
//!
//! ```text
//! cargo run -p oxyn-dioxus --example composer_sheet -- tokens/dist/oxyn.css > composer.html
//! ```

use std::{cell::Cell, time::Duration};

use dioxus::prelude::*;
use oxyn_core::{
    ActivityKind, AgentState, ClockMs, ToolRef,
    composer::{
        AttachmentItem, AttachmentState, Capabilities, CategoryUsage, ContextCategory as C,
        ContextMeter, ContextRef, ContextUsage, DisabledReason, EffortControl, EffortLevel,
        EffortSetting, ModelCatalog, ModelInfo, ModelSelection, ModelStatus, Picker,
        ReasoningSupport, attachments::Reject, model::Tier3,
    },
};
use oxyn_dioxus::{
    AttachmentRow, COMPOSER_CSS, Composer, ContextMeterChip, DropZone, EffortChip,
    GLYPH_LOADER_CSS, ModelPicker, STATUS_TEXT_CSS,
};

const NOW: ClockMs = ClockMs(1_000_000);

thread_local! {
    static T: Cell<u64> = const { Cell::new(NOW.0) };
}

fn levels() -> ReasoningSupport {
    ReasoningSupport::Levels {
        allowed: vec![
            EffortLevel::Low,
            EffortLevel::Medium,
            EffortLevel::High,
            EffortLevel::Max,
        ],
        default: EffortLevel::Medium,
    }
}

fn catalog() -> ModelCatalog {
    let m =
        |id: &str, p: &str, name: &str, vision, ctx, speed, cost: Option<&str>, status| ModelInfo {
            id: id.into(),
            provider: p.into(),
            name: name.into(),
            short_name: None,
            caps: Capabilities {
                vision,
                tools: true,
                reasoning: if p == "local" {
                    ReasoningSupport::Toggle
                } else {
                    levels()
                },
            },
            context_tokens: ctx,
            speed: Some(speed),
            cost: cost.map(str::to_string),
            status,
        };
    ModelCatalog {
        providers: vec![
            ("ember".into(), "Ember".into()),
            ("slate".into(), "Slate".into()),
            ("local".into(), "On device".into()),
        ],
        models: vec![
            m(
                "ember/swift-2",
                "ember",
                "Ember Swift 2",
                true,
                200_000,
                Tier3::Fast,
                Some("$1 / $5 per Mtok"),
                ModelStatus::Available,
            ),
            m(
                "ember/deep-2",
                "ember",
                "Ember Deep 2",
                true,
                1_000_000,
                Tier3::Slow,
                Some("$5 / $25 per Mtok"),
                ModelStatus::Available,
            ),
            m(
                "slate/one",
                "slate",
                "Slate One",
                false,
                128_000,
                Tier3::Medium,
                None,
                ModelStatus::NeedsKey,
            ),
            m(
                "slate/vision",
                "slate",
                "Slate Vision",
                true,
                256_000,
                Tier3::Medium,
                None,
                ModelStatus::RateLimited {
                    until: ClockMs(NOW.0 + 252_000),
                },
            ),
            m(
                "local/q7",
                "local",
                "Local Q 7B",
                false,
                32_000,
                Tier3::Fast,
                Some("Free · on device"),
                ModelStatus::Available,
            ),
        ],
        auto: true,
    }
}

fn usage(history: u32) -> ContextUsage {
    let c = |category, tokens| CategoryUsage { category, tokens };
    ContextUsage {
        window: Some(200_000),
        categories: vec![
            c(C::System, 9_200),
            c(C::Tools, 14_600),
            c(C::Memory, 3_100),
            c(C::Skills, 600),
            c(C::Files, 18_400),
            c(C::History, history),
            c(C::Reserved, 20_000),
        ],
        exact: true,
        at: NOW,
    }
}

fn atts() -> Vec<AttachmentItem> {
    vec![
        AttachmentItem::file(
            1,
            "login-flow.png",
            1_400_000,
            AttachmentState::Uploading {
                progress: Some(0.64),
            },
        ),
        AttachmentItem::file(
            2,
            "spec.pdf",
            812_000,
            AttachmentState::Ready {
                meta: "PDF · 12 pages".into(),
            },
        ),
        AttachmentItem::file(
            3,
            "auth.rs",
            18_000,
            AttachmentState::Ready {
                meta: String::new(),
            },
        ),
        AttachmentItem::file(
            4,
            "q3-revenue.csv",
            2_400_000,
            AttachmentState::Error {
                reason: "Upload failed".into(),
                retryable: true,
            },
        ),
    ]
}

fn tool() -> AgentState {
    AgentState::ToolRunning {
        tool: ToolRef {
            id: "t".into(),
            name: "edit".into(),
            activity: ActivityKind::Edit,
            target: Some("auth.rs".into()),
        },
        concurrent: 1,
    }
}

fn all_atts() -> Vec<AttachmentItem> {
    vec![
        AttachmentItem::file(
            1,
            "hero.png",
            1_400_000,
            AttachmentState::Uploading {
                progress: Some(0.42),
            },
        ),
        AttachmentItem::file(
            2,
            "spec.pdf",
            812_000,
            AttachmentState::Ready {
                meta: "PDF · 12 pages".into(),
            },
        ),
        AttachmentItem::file(
            3,
            "export.csv",
            32_000_000,
            AttachmentState::Rejected(Reject::TooLarge {
                bytes: 32_000_000,
                limit: 20_000_000,
            }),
        ),
        AttachmentItem::file(
            4,
            "main.rs",
            9_000,
            AttachmentState::Reading {
                label: "Reading…".into(),
            },
        ),
    ]
}

fn ctx_refs() -> Vec<AttachmentItem> {
    vec![
        AttachmentItem::context(10, ContextRef::File("src/middleware/auth.rs".into())),
        AttachmentItem::context(
            11,
            ContextRef::Repo {
                branch: "main".into(),
                changes: 3,
            },
        ),
        AttachmentItem::context(
            12,
            ContextRef::Selection {
                path: "src/token.rs".into(),
                from: 12,
                to: 40,
            },
        ),
        AttachmentItem::context(13, ContextRef::Url("https://docs.rs/tokio".into())),
    ]
}

#[component]
fn Sheet() -> Element {
    let now = ClockMs(T.with(Cell::get));
    let cat = catalog();
    let favs = vec![
        "ember/swift-2".to_string(),
        "ember/deep-2".into(),
        "local/q7".into(),
    ];
    let recent = vec!["slate/vision".to_string()];
    let swift = ModelSelection::Model("ember/swift-2".into());
    let elapsed = Duration::from_secs(42);
    let mut meter = ContextMeter::default();
    let u = usage(72_000);
    let _ = meter.view(&u, ClockMs(0));
    let cv = meter.view(&u, now);
    let mut m2 = ContextMeter::default();
    let crit = usage(118_000);
    let _ = m2.view(&crit, ClockMs(0));
    let cv_crit = m2.view(&crit, now);
    let mut ec = EffortControl::new(EffortSetting::Level(EffortLevel::High));
    let ev = ec.view(&levels(), now);
    let mut ec2 = EffortControl::new(EffortSetting::Level(EffortLevel::Max));
    ec2.commit(
        EffortSetting::Level(EffortLevel::Low),
        &AgentState::Streaming,
        &levels(),
        NOW,
    );
    let ev_pending = ec2.view(&levels(), now);
    let open_picker = |sel: &ModelSelection, q: &str| {
        let mut p = Picker::default();
        let rows = p.rows(&cat, &favs, &recent, sel);
        p.open(&rows, sel);
        p.query = q.into();
        p
    };
    rsx! {
        div { class: "page",
            header {
                h1 { "Oxyn · composer" }
                p { "Plan #18 · input shell, attachments, model picker, effort, context meter · Dioxus renderer · dark theme" }
            }
            section { class: "layouts",
                div { class: "cell tall",
                    span { class: "tag", "composer/chat · model picker open" }
                    Composer { layout: "chat", agent: AgentState::Idle, now, catalog: cat.clone(), model: swift.clone(),
                        favorites: favs.clone(), recent: recent.clone(), usage: Some(u.clone()), attachments: atts(),
                        text: "Why does login fail when two tabs refresh at once? Check the token path.", open: "model", focused: true }
                }
                div { class: "cell tall",
                    span { class: "tag", "composer/chat · busy with draft · context open" }
                    Composer { layout: "chat", agent: tool(), now, elapsed, catalog: cat.clone(), model: swift.clone(),
                        favorites: favs.clone(), usage: Some(crit.clone()), text: "also check the refresh-token path", open: "context",
                        effort: EffortSetting::Level(EffortLevel::Max) }
                }
                div { class: "cell mid",
                    span { class: "tag", "composer/desktop · effort menu open" }
                    Composer { layout: "desktop", agent: AgentState::Idle, now, catalog: cat.clone(), model: ModelSelection::Model("ember/deep-2".into()),
                        favorites: favs.clone(), usage: Some(u.clone()), attachments: ctx_refs(), attachment_variant: "context-chip".to_string(),
                        text: "Plan the migration to the new session store", open: "effort", effort_variant: "signal-bars".to_string(),
                        tray: vec!["oxyn".into(), "feat/composer-shell".into(), "Local".into()] }
                }
                div { class: "cell mid",
                    span { class: "tag", "composer/ide-panel · thinking" }
                    Composer { layout: "ide-panel", agent: AgentState::Thinking { header: None }, now, elapsed, catalog: cat.clone(),
                        model: swift.clone(), usage: Some(u.clone()), attachments: atts().into_iter().take(3).collect::<Vec<_>>() }
                    span { class: "tag", "composer/terminal" }
                    Composer { layout: "terminal", agent: AgentState::Idle, now, catalog: cat.clone(), model: swift.clone(),
                        usage: Some(u.clone()), attachments: all_atts().into_iter().take(2).collect::<Vec<_>>(), text: "refactor the token cache", effort: EffortSetting::Level(EffortLevel::Max) }
                }
                div { class: "cell mid",
                    span { class: "tag", "composer/mobile · offline" }
                    Composer { layout: "mobile", agent: AgentState::Idle, now, catalog: cat.clone(), model: ModelSelection::Auto { resolved: None },
                        usage: Some(u.clone()), attachments: atts().into_iter().take(2).collect::<Vec<_>>(), disabled: Some(DisabledReason::Offline), text: "Draft kept while offline" }
                    span { class: "tag", "composer/node · drop target" }
                    Composer { layout: "node", agent: AgentState::Idle, now, catalog: cat.clone(), model: swift.clone(), usage: Some(u.clone()),
                        drop: Some(("valid".to_string(), "Drop to attach · 3 files".to_string())) }
                }
            }
            section { class: "parts",
                div { class: "card",
                    h2 { "Attachments · 10" }
                    for v in ["tile-ring", "thumb-strip", "chip", "chip-bar", "list", "terminal"] {
                        div { class: "prow", span { class: "pnm", "{v}" } div { class: "v", AttachmentRow { items: all_atts(), now, variant: v.to_string() } } }
                    }
                    div { class: "prow", span { class: "pnm", "context-chip" } div { class: "v", AttachmentRow { items: ctx_refs(), now, variant: "context-chip".to_string() } } }
                    div { class: "prow", span { class: "pnm", "paste-chip" } div { class: "v",
                        AttachmentRow { items: vec![AttachmentItem { id: 30, name: "Pasted text".into(), kind: oxyn_core::composer::AttachmentKind::PastedText { lines: 142 }, state: AttachmentState::Ready { meta: "142 lines".into() } }], now, variant: "paste-chip".to_string() } } }
                    div { class: "prow", span { class: "pnm", "drop-inline · drop-overlay" } div { class: "v drops",
                        div { class: "dz", DropZone { state: "valid", label: "Drop to attach · 3 files" } }
                        div { class: "dz", DropZone { state: "invalid", label: "2 of 3 files can't be attached" } }
                    } }
                    h2 { class: "gap", "Effort · 6" }
                    for v in ["signal-bars", "segmented", "slider", "chevrons", "menu", "toggle"] {
                        div { class: "prow", span { class: "pnm", "{v}" } div { class: "v", EffortChip { view: ev.clone(), support: levels(), variant: v.to_string() } } }
                    }
                    div { class: "prow", span { class: "pnm", "pending next turn" } div { class: "v", EffortChip { view: ev_pending.clone(), support: levels() } } }
                }
                div { class: "card",
                    h2 { "Model picker · 6" }
                    div { class: "prow", span { class: "pnm", "pill-popover" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: swift.clone(), picker: Picker::default(), now } } }
                    div { class: "prow", span { class: "pnm", "pending next turn" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: swift.clone(), pending: Some(ModelSelection::Model("ember/deep-2".into())), picker: Picker::default(), now } } }
                    div { class: "prow", span { class: "pnm", "auto routing" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: ModelSelection::Auto { resolved: None }, routing: true, picker: Picker::default(), now } } }
                    div { class: "prow", span { class: "pnm", "compact" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: swift.clone(), picker: Picker::default(), variant: "compact".to_string(), now } } }
                    div { class: "prow", span { class: "pnm", "favorites-segmented" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: swift.clone(), favorites: favs.clone(), picker: Picker::default(), variant: "favorites-segmented".to_string(), now } } }
                    div { class: "prow tallrow", span { class: "pnm", "search · 'dp'" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: swift.clone(), favorites: favs.clone(), recent: recent.clone(), picker: open_picker(&swift, "dp"), now } } }
                    div { class: "prow glyphrow", span { class: "pnm", "glyph (terminal)" } div { class: "v", ModelPicker { catalog: cat.clone(), selected: swift.clone(), favorites: favs.clone(), recent: recent.clone(), picker: open_picker(&swift, ""), variant: "glyph".to_string(), now } } }
                    h2 { class: "gap", "Context meter · 6" }
                    for v in ["ring", "ring-segmented", "bar", "grid", "text"] {
                        div { class: "prow", span { class: "pnm", "{v}" } div { class: "v",
                            ContextMeterChip { view: cv.clone(), usage: u.clone(), variant: v.to_string() }
                            ContextMeterChip { view: cv_crit.clone(), usage: crit.clone(), variant: v.to_string() }
                        } }
                    }
                    div { class: "prow", span { class: "pnm", "card" } div { class: "v", ContextMeterChip { view: cv.clone(), usage: u.clone(), variant: "card".to_string(), model: "Ember Swift 2".to_string() } } }
                }
            }
        }
    }
}

const PAGE_CSS: &str = r#"
html,body{margin:0;background:#0B0B0C}
.page{padding:32px;width:1460px;font-family:Inter,system-ui,sans-serif;color:var(--oxyn-color-fg-default)}
header h1{margin:0;font-size:24px;font-weight:600;color:#F2EFEA}
header p{margin:4px 0 22px;color:#8A867F;font-size:13px}
.layouts{display:grid;grid-template-columns:1fr 1fr;gap:18px}
.cell{position:relative;background:var(--oxyn-color-bg-base);border:1px solid var(--oxyn-color-border-subtle);border-radius:16px;padding:18px;
  display:flex;flex-direction:column;justify-content:flex-end;gap:10px}
.cell.tall{min-height:560px}.cell.mid{min-height:430px}
.tag{font-size:11px;letter-spacing:.05em;text-transform:uppercase;color:var(--oxyn-color-fg-subtle);font-weight:600}
.cell>.tag:first-child{position:absolute;top:16px;left:18px}
.parts{display:grid;grid-template-columns:1fr 1fr;gap:18px;margin-top:18px}
.card{background:var(--oxyn-color-bg-base);border:1px solid var(--oxyn-color-border-subtle);border-radius:16px;padding:18px}
.card h2{margin:0 0 12px;font-size:11.5px;letter-spacing:.06em;text-transform:uppercase;color:var(--oxyn-color-fg-muted)}
.card h2.gap{margin-top:22px}
.prow{display:grid;grid-template-columns:150px 1fr;align-items:center;min-height:40px;padding:5px 0}
.prow .pnm{font-family:"DejaVu Sans Mono",monospace;font-size:11.5px;color:var(--oxyn-color-fg-muted)}
.prow .v{position:relative;display:flex;gap:14px;align-items:center;flex-wrap:wrap}
.prow.tallrow{min-height:420px;align-items:end}
.prow.glyphrow{min-height:200px;align-items:end}
.drops{gap:10px}
.dz{position:relative;width:230px;height:64px;border-radius:12px;--c-bg:var(--oxyn-color-surface-raised);background:var(--c-bg)}
"#;

fn main() {
    let tokens = std::env::args()
        .nth(1)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    // Render a few frames apart so time-based parts (status tab loader, upload rings) have
    // history, the way a live page would.
    let mut dom = VirtualDom::new(Sheet);
    T.with(|c| c.set(NOW.0 - 2_000));
    dom.rebuild_in_place();
    for t in [NOW.0 - 1_000, NOW.0 - 400, NOW.0] {
        T.with(|c| c.set(t));
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
    }
    let body = dioxus_ssr::render(&dom);
    println!(
        "<!doctype html><html><head><meta charset=utf-8><style>{tokens}{GLYPH_LOADER_CSS}{STATUS_TEXT_CSS}{COMPOSER_CSS}{PAGE_CSS}</style></head><body>{body}</body></html>"
    );
}
