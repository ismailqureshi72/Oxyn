//! Approvals and checkpoints sheet (plan #21).
//! `cargo run -p oxyn-dioxus --example hitl_sheet -- tokens/dist/oxyn.css > sheet.html`

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    approval::{ApprovalCard as Card, ApprovalQueue, ApprovalRequest, Decision, Risk},
    checkpoint::{Checkpoint, CheckpointKind, CheckpointStatus, RestoreFlow},
};
use oxyn_dioxus::{
    ApprovalCard, ApprovalPager, CheckpointLanes, CheckpointList, CheckpointMarker, HITL_CSS,
    RestoreToast,
};

const NOW: ClockMs = ClockMs(1_000_000);

fn req(id: &str, title: &str, risk: Risk, opened_ago: u64) -> ApprovalRequest {
    ApprovalRequest::new(id, title, risk, ClockMs(NOW.0 - opened_ago))
}

fn email() -> Card {
    let mut r = req("e", "Send campaign email", Risk::Medium, 18_000);
    r.description = Some("October product update · Newsletter".into());
    r.label = Some("Step 4 · gmail.send".into());
    r.details = vec![
        ("To".into(), "Customers – EU (42)".into()),
        ("Subject".into(), "What's new in October".into()),
        ("Send".into(), "Today, 9:00 AM".into()),
    ];
    r.expires_in = Some(Duration::from_secs(60));
    Card::new(r)
}

fn shell_low() -> Card {
    let mut r = req("l", "Run the test suite", Risk::Low, 5_000);
    r.description = Some("Reads only · inside the workspace".into());
    r.command = Some("cargo test -p oxyn-core".into());
    r.label = Some("Step 2 · bash".into());
    Card::new(r)
}

fn high() -> Card {
    let mut r = req("h", "Delete build artefacts", Risk::High, 700);
    r.description = Some("Outside git · 1.2 GB".into());
    r.command = Some("rm -rf target/ dist/".into());
    r.reasons = vec!["Deletes files that git does not track".into()];
    Card::new(r)
}

fn critical() -> Card {
    let mut r = req("c", "Charge the customer card", Risk::Critical, 4_000);
    r.description = Some("Stripe · live mode".into());
    r.amount = Some("$420.00".into());
    r.details = vec![
        ("Payee".into(), "Northwind Ltd".into()),
        ("Card".into(), "Visa ···· 4242".into()),
    ];
    r.reasons = vec![
        "Moves real money".into(),
        "Can't be undone from here".into(),
    ];
    let mut c = Card::new(r);
    c.hold_start(ClockMs(NOW.0 - 780)).unwrap();
    c
}

fn resolved(d: Decision, title: &str) -> Card {
    let mut c = Card::new(req("r", title, Risk::Low, 7_000));
    c.press(d, ClockMs(NOW.0 - 400)).unwrap();
    c.ack(ClockMs(NOW.0 - 300));
    c
}

fn queue() -> ApprovalQueue {
    let mut q = ApprovalQueue::default();
    for (i, (t, r)) in [
        ("Read .env.example", Risk::Low),
        ("Edit src/auth/refresh.rs", Risk::Medium),
        ("Run npm install", Risk::Medium),
    ]
    .into_iter()
    .enumerate()
    {
        let mut x = req(&i.to_string(), t, r, 3_000);
        x.description = Some("Coder · this session".into());
        q.requests.push(x);
    }
    q
}

fn cps() -> Vec<Checkpoint> {
    let mut v = vec![
        Checkpoint::new("c1", None, "Read the brief", 9),
        Checkpoint::new("c2", Some("c1"), "Built the pricing table", 64).files(3, 120, 4),
        Checkpoint::new("c3", Some("c2"), "Added the Stripe checkout", 98).files(2, 61, 9),
        Checkpoint::new("c4", Some("c3"), "Tests failing – 4 of 12", 131)
            .with(CheckpointStatus::Failed)
            .files(1, 12, 3),
        Checkpoint::new("c5", Some("c2"), "Switched to Stripe Elements", 169).files(4, 88, 41),
        Checkpoint::new("c6", Some("c5"), "Wiring the webhooks", 212)
            .with(CheckpointStatus::Running),
    ];
    v[2].cost_cents = Some(31);
    v[5].cost_cents = Some(42);
    v
}

#[component]
fn Sheet() -> Element {
    let mut medium_hover = email();
    medium_hover.set_hover(true, ClockMs(NOW.0 - 2_000));
    let mut sending = shell_low();
    sending
        .press(Decision::AllowOnce, ClockMs(NOW.0 - 600))
        .unwrap();
    let mut stale = shell_low();
    stale.stale();
    let mut expiring = email();
    expiring.request.expires_in = Some(Duration::from_secs(25));
    let chat = Checkpoint {
        chat_only: true,
        ..Checkpoint::new("m", None, "Asked for a summary", 0)
    };
    let mut manual = Checkpoint::new("s", None, "Before the auth refactor", 0).files(3, 42, 17);
    manual.kind = CheckpointKind::Manual;
    let list = cps();
    let times: Vec<String> = [
        "3:31 PM", "3:33 PM", "3:36 PM", "3:38 PM", "3:42 PM", "3:45 PM",
    ]
    .map(String::from)
    .to_vec();
    rsx! {
        div { class: "page",
            header { h1 { "Oxyn · approvals and checkpoints" } p { "Plan #21 · free variants · Dioxus renderer · review sheet, not the site layout" } }
            div { class: "cols",
                div { class: "col",
                    div { class: "card",
                        h2 { "approval-card/inline · medium, countdown, always" }
                        ApprovalCard { card: email(), now: NOW }
                        h2 { class: "gap", "low · command · sending (spinner after 300 ms)" }
                        ApprovalCard { card: sending, now: NOW }
                        h2 { class: "gap", "high · arming (Allow fills 600 ms, Deny is default)" }
                        ApprovalCard { card: high(), now: NOW }
                        h2 { class: "gap", "critical · hold to allow (65 %)" }
                        ApprovalCard { card: critical(), now: NOW }
                    }
                    div { class: "card",
                        h2 { "countdown · last 10 s turns amber" }
                        ApprovalCard { card: expiring, now: NOW, actions: "card" }
                        h2 { class: "gap", "resolved rows" }
                        div { class: "stack",
                            ApprovalCard { card: resolved(Decision::AllowOnce, "Run the test suite"), now: NOW }
                            ApprovalCard { card: resolved(Decision::AllowAlways { pattern: "cargo test *".into() }, "Run cargo test"), now: NOW }
                            ApprovalCard { card: resolved(Decision::Deny { reason: Some("use --dry-run first".into()) }, "Deploy to staging"), now: NOW }
                            ApprovalCard { card: stale, now: NOW }
                        }
                    }
                }
                div { class: "col",
                    div { class: "card",
                        h2 { "approval-card/paged · queue with batch" }
                        ApprovalPager { queue: queue(), now: NOW }
                        h2 { class: "gap", "approval-card/footer · inside a tool-call card" }
                        div { class: "toolcard",
                            div { class: "tch", span { class: "hand", "✋" } b { "Running" } code { "rm -rf target/" } span { class: "w", "Waiting for you" } }
                            ApprovalCard { card: high(), now: ClockMs(NOW.0 + 2_000), variant: "footer" }
                        }
                        h2 { class: "gap", "approval-card/terminal" }
                        ApprovalCard { card: shell_low(), now: NOW, variant: "terminal" }
                        div { class: "gap8" }
                        ApprovalCard { card: critical(), now: NOW, variant: "terminal" }
                        h2 { class: "gap", "approval-card/modal · critical" }
                        ApprovalCard { card: critical(), now: NOW, variant: "modal" }
                    }
                }
            }
            div { class: "cols",
                div { class: "col",
                    div { class: "card",
                        h2 { "checkpoint-timeline/lanes · rewound once, attempt 2 running" }
                        CheckpointLanes { checkpoints: list.clone(), head: Some("c6".to_string()), task: Some("Add Stripe checkout".to_string()) }
                    }
                    div { class: "card",
                        h2 { "checkpoint-timeline/side-list" }
                        CheckpointList { checkpoints: list.clone(), times: times.clone(), selected: Some(4) }
                        h2 { class: "gap", "checkpoint-timeline/terminal · Esc Esc" }
                        CheckpointList { checkpoints: list.clone(), times: times.clone(), variant: "terminal", selected: Some(4) }
                    }
                }
                div { class: "col",
                    div { class: "card markers",
                        h2 { "checkpoint-timeline/inline-marker" }
                        div { class: "msg", b { "You" } " refactor the auth middleware to use sessions" }
                        div { class: "msg", b { "Agent" } " … Edited 3 files +42 −17" }
                        CheckpointMarker { checkpoint: list[1].clone(), time: "3:42 PM" }
                        div { class: "msg", b { "You" } " now add rate limiting" }
                        h2 { class: "gap", "revealed · menu open · agent running" }
                        div { class: "menuroom", CheckpointMarker { checkpoint: list[1].clone(), time: "3:42 PM", revealed: true, menu: true, blocked: true } }
                        h2 { class: "gap", "current · chat-only · saved" }
                        CheckpointMarker { checkpoint: list[4].clone(), time: "3:45 PM", current: true }
                        CheckpointMarker { checkpoint: chat, time: "3:47 PM" }
                        CheckpointMarker { checkpoint: manual, time: "3:50 PM", revealed: true }
                        h2 { class: "gap", "checkpoint-timeline/hover-revert" }
                        div { class: "msg hov", CheckpointMarker { checkpoint: list[2].clone(), time: "3:36 PM", variant: "hover-revert", revealed: true } span { b { "You" } " switch to Stripe Elements" } }
                        h2 { class: "gap", "restore toasts" }
                        div { class: "stack",
                            RestoreToast { flow: RestoreFlow::Restored { label: "Restored 3 files to 3:42 PM".into(), until: ClockMs(NOW.0 + 6_000) }, now: NOW }
                            RestoreToast { flow: RestoreFlow::Restoring { done: 4, total: 7 }, now: NOW }
                            RestoreToast { flow: RestoreFlow::Partial { label: "Restored 6 of 7 files · rate.rs is locked".into() }, now: NOW }
                        }
                    }
                }
            }
        }
    }
}

const PAGE_CSS: &str = r#"
html,body{margin:0;background:#0B0B0C}
.page{padding:32px;width:1460px;font-family:Inter,system-ui,sans-serif;color:var(--oxyn-color-fg-default)}
header h1{margin:0;font-size:24px;font-weight:600}
header p{margin:4px 0 22px;color:var(--oxyn-color-fg-subtle);font-size:13px}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:18px;margin-bottom:18px;align-items:start}
.col{display:flex;flex-direction:column;gap:18px}
.card{background:var(--oxyn-color-bg-base);border:1px solid var(--oxyn-color-border-subtle);border-radius:16px;padding:18px}
.card h2{margin:0 0 10px;font-size:11.5px;letter-spacing:.06em;text-transform:uppercase;color:var(--oxyn-color-fg-muted);font-weight:600}
.card h2.gap{margin-top:22px}
.gap8{height:8px}
.stack{display:flex;flex-direction:column;gap:8px;align-items:flex-start}
.stack>*{align-self:stretch}.stack>.oxyn-cp.toast{align-self:flex-start}
.toolcard{border:1px solid var(--oxyn-color-border-default);border-radius:10px;background:var(--oxyn-color-surface-raised)}
.tch{display:flex;align-items:center;gap:8px;height:36px;padding:0 10px;font-size:13px}
.tch code{font-family:"JetBrains Mono",monospace;font-size:12px;color:var(--oxyn-color-fg-muted)}
.tch .w,.tch .hand{color:var(--oxyn-color-status-waiting-fg)}
.msg{display:flex;gap:6px;padding:6px 4px;color:var(--oxyn-color-fg-muted)}.msg b{color:var(--oxyn-color-fg-default);font-weight:600}
.msg.hov{align-items:center;background:var(--oxyn-color-surface-hover);border-radius:8px;min-height:30px;position:relative}
.menuroom{min-height:200px}
"#;

fn main() {
    let tokens = std::env::args()
        .nth(1)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let mut dom = VirtualDom::new(Sheet);
    dom.rebuild_in_place();
    let body = dioxus_ssr::render(&dom);
    println!(
        "<!doctype html><html><head><meta charset=utf-8><style>{tokens}{HITL_CSS}{PAGE_CSS}</style></head><body>{body}</body></html>"
    );
}
